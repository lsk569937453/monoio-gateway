use crate::middleware::log_service::LogService;
use anyhow::anyhow;
use http::{response::Builder, HeaderMap, StatusCode};
use monoio::io::AsyncReadRent;
use monoio::io::AsyncWriteRentExt;
use monoio::{
    io::{
        sink::{Sink, SinkExt},
        stream::Stream,
        Splitable,
    },
    net::{TcpListener, TcpStream},
};
use monoio_http::{
    common::{error::HttpError, request::Request, response::Response},
    h1::{
        codec::{decoder::RequestDecoder, encoder::GenericEncoder},
        payload::{FixedPayload, Payload},
    },
    util::spsc::{spsc_pair, SPSCReceiver},
};
use tower::layer::layer_fn;
use tower::Layer;
use tower::ServiceBuilder;
use tower::{service_fn, BoxError, Service, ServiceExt};
use vojo::app_error::AppError;
mod constants;
mod control_plane;
mod middleware;
mod vojo;
#[macro_use]
extern crate tracing;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate async_trait;
use crate::control_plane::rest_api::start_control_plane;
use crate::middleware::ip_allow_service::IpAllowService;
use tracing_subscriber::FmtSubscriber;
use vojo::gateway_request;
use vojo::gateway_request::GatewayRequest;
fn main() -> Result<(), anyhow::Error> {
    std::sync::mpsc::channel();
    std::thread::spawn(move || {
        starts_control_plane();
    });
    let cpus = num_cpus::get();
    println!("Cpu core is {}", cpus);

    std::thread::scope(|s| {
        // let addr_clone = addr.clone();
        // let database_clone = database_holder.clone();
        for i in 0..cpus {
            println!("thread is {}", i);
            // let addr_clone1 = addr_clone.clone();
            // let database_clone1 = database_clone.clone();
            s.spawn(move || {
                let mut rt = monoio::RuntimeBuilder::<monoio::IoUringDriver>::new()
                    .with_entries(256)
                    .enable_timer()
                    .build()
                    .unwrap();
                rt.block_on(async {
                    main_with_error().await;
                });
            });
        }
    });
    Ok(())
}
fn starts_control_plane() -> Result<(), AppError> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .map_err(|e| AppError(e.to_string()))?;
    rt.block_on(async {
        start_control_plane(8888).await;
    });
    Ok(())
}
async fn main_with_error() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(tracing::Level::DEBUG)
        .finish();
    // Initialize the tracing subscriber
    let _ = tracing::subscriber::set_global_default(subscriber);
    let listener = TcpListener::bind("0.0.0.0:8080").unwrap();
    info!("Listening 0.0.0.0:8080");
    loop {
        let incoming = listener.accept().await;
        match incoming {
            Ok((stream, addr)) => {
                // println!("accepted a connection from {}", addr);
                monoio::spawn(handle_connection(stream, addr.to_string()));
            }
            Err(e) => {
                // println!("accepted connection failed: {}", e);
            }
        }
    }
}
async fn handle_connection(stream: TcpStream, addr: String) {
    let (r, w) = stream.into_split();
    let sender = GenericEncoder::new(w);
    let mut receiver = RequestDecoder::new(r);
    let (mut tx, rx) = spsc_pair();
    monoio::spawn(handle_task(rx, sender, addr));

    loop {
        match receiver.next().await {
            None => {
                // println!("connection closed, connection handler exit");
                return;
            }
            Some(Err(_)) => {
                // println!("receive request failed, connection handler exit");
                return;
            }
            Some(Ok(item)) => match tx.send(item).await {
                Err(_) => {
                    // println!("request handler dropped, connection handler exit");
                    return;
                }
                Ok(_) => {
                    // println!("request handled success");
                }
            },
        }
    }
}

async fn handle_task(
    mut receiver: SPSCReceiver<Request>,
    mut sender: impl Sink<Response, Error = impl Into<HttpError>>,
    remote_addr: String,
) -> Result<(), anyhow::Error> {
    let service_fn = service_fn(handle_request);
    let log_service_fn = layer_fn(|service| LogService {
        service,
        target: "tower-docs",
    });
    let ip_allow_service_fn = layer_fn(|service| IpAllowService {
        service,
        target: "tower-docs",
    });

    let mut tower_service = ServiceBuilder::new()
        .layer(ip_allow_service_fn)
        .layer(log_service_fn)
        .service(service_fn);
    loop {
        let request = match receiver.recv().await {
            Some(r) => r,
            None => {
                return Ok(());
            }
        };
        let gateway_request = GatewayRequest::new(request, remote_addr.clone());

        let resp = tower_service.call(gateway_request).await?;

        sender.send_and_flush(resp).await.map_err(Into::into)?;
    }
}

async fn handle_request(gateway_request: GatewayRequest) -> Result<Response, anyhow::Error> {
    let req = gateway_request.request;
    let mut headers = HeaderMap::new();
    headers.insert("Server", "monoio-http-demo".parse().unwrap());
    let mut has_error = false;
    let mut has_payload = false;
    let payload = match req.into_body() {
        Payload::None => Payload::None,
        Payload::Fixed(mut p) => match p.next().await.unwrap() {
            Ok(data) => {
                has_payload = true;
                Payload::Fixed(FixedPayload::new(data))
            }
            Err(_) => {
                has_error = true;
                Payload::None
            }
        },
        Payload::Stream(_) => unimplemented!(),
    };

    let status = if has_error {
        StatusCode::INTERNAL_SERVER_ERROR
    } else if has_payload {
        StatusCode::OK
    } else {
        StatusCode::NO_CONTENT
    };
    Ok(Builder::new()
        .status(status)
        .header("Server", "monoio-http-demo")
        .body(payload)
        .unwrap())
}
