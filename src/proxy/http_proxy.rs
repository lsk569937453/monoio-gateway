use crate::middleware::log_service::LogService;
use crate::vojo::app_error::AppError;
use crate::vojo::handler::Handler;
use bytes::Bytes;
use futures::channel::mpsc::UnboundedReceiver;
use futures::channel::mpsc::UnboundedSender;
use futures::StreamExt;
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

use crate::control_plane::rest_api::start_control_plane;
use crate::middleware::ip_allow_service::IpAllowService;
use crate::vojo::gateway_request;
use futures::channel::mpsc::unbounded;
use monoio::io::Canceller;
use monoio_http_client::Client;
use std::pin::pin;
use std::sync::RwLock;
use tracing_subscriber::FmtSubscriber;

use crate::vojo::gateway_request::GatewayRequest;

pub fn create_monoio_runtime(port: i32, handler: Handler) {
    let cpus = num_cpus::get();
    println!("Cpu core is {}", cpus);
    for i in 0..cpus {
        let handle_clone1 = handler.clone();

        println!("thread is {}", i);
        std::thread::spawn(move || {
            let mut rt = monoio::RuntimeBuilder::<monoio::IoUringDriver>::new()
                .with_entries(256)
                .enable_timer()
                .build()
                .unwrap();

            rt.block_on(async {
                main_with_error(port, handle_clone1).await;
            });
        });
    }
}
pub async fn main_with_error(port: i32, handler: Handler) {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(tracing::Level::DEBUG)
        .finish();
    // Initialize the tracing subscriber
    let _ = tracing::subscriber::set_global_default(subscriber);
    let addr = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(addr.clone()).unwrap();
    let client = Client::default();

    info!("Listening {}", addr);
    loop {
        if let Ok((stream, addr)) = listener.accept().await {
            monoio::spawn(handle_connection(
                client.clone(),
                handler.clone(),
                stream,
                addr.to_string(),
            ));
        }
    }
}
async fn handle_connection(client: Client, handler: Handler, stream: TcpStream, addr: String) {
    let (r, w) = stream.into_split();
    let sender = GenericEncoder::new(w);
    let mut receiver = RequestDecoder::new(r);
    let (mut tx, rx) = spsc_pair();
    monoio::spawn(handle_task(client, handler, rx, sender, addr));

    loop {
        match receiver.next().await {
            None => {
                println!("connection closed, connection handler exit");
                return;
            }
            Some(Err(_)) => {
                println!("receive request failed, connection handler exit");
                return;
            }
            Some(Ok(item)) => match tx.send(item).await {
                Err(_) => {
                    println!("request handler dropped, connection handler exit");
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
    client: Client,
    handler: Handler,
    mut receiver: SPSCReceiver<Request>,
    mut sender: impl Sink<Response, Error = impl Into<HttpError>>,
    remote_addr: String,
) -> Result<(), AppError> {
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
        let data = handler
            .shared_app_config
            .read()
            .map_err(|e| AppError(e.to_string()))?
            .clone();

        let gateway_request = GatewayRequest::new(request, remote_addr.clone(), client.clone());

        let resp = tower_service.call(gateway_request).await;

        match resp {
            Ok(s) => sender
                .send_and_flush(s)
                .await
                .map_err(Into::into)
                .map_err(|e| AppError(e.to_string()))?,
            Err(e) => {
                error!("{}", e);
            }
        }
    }
}

async fn handle_request(gateway_request: GatewayRequest) -> Result<Response, AppError> {
    let resp_result = gateway_request
        .client
        .get("http://backend:8080/get")
        .send()
        .await;

    match resp_result {
        Ok(resp) => {
            info!("has receive response,header is:{:?}", resp.headers());
            let http_resp = resp.bytes().await.unwrap();
            let res = Payload::Fixed(FixedPayload::new(http_resp));

            // let req = gateway_request.request;
            // let mut headers = HeaderMap::new();
            // headers.insert("Server", "monoio-http-demo".parse().unwrap());
            // let mut has_error = false;
            // let mut has_payload = false;
            // let payload = match req.into_body() {
            //     Payload::None => Payload::None,
            //     Payload::Fixed(mut p) => match p.next().await.unwrap() {
            //         Ok(data) => {
            //             has_payload = true;
            //             Payload::Fixed(FixedPayload::new(data))
            //         }
            //         Err(_) => {
            //             has_error = true;
            //             Payload::None
            //         }
            //     },
            //     Payload::Stream(_) => unimplemented!(),
            // };

            // let status = if has_error {
            //     StatusCode::INTERNAL_SERVER_ERROR
            // } else if has_payload {
            //     StatusCode::OK
            // } else {
            //     StatusCode::NO_CONTENT
            // };
            Ok(Builder::new()
                .status(StatusCode::OK)
                .header("Server", "monoio-http-demo")
                .body(res)
                .unwrap())
        }
        Err(e) => {
            return Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Payload::Fixed(FixedPayload::new(Bytes::from(
                    e.to_string(),
                ))))
                .unwrap());
        }
    }
}
