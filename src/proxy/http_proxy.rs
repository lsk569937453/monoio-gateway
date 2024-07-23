use crate::middleware::log_service::LogService;
use crate::vojo::app_error::AppError;
use crate::vojo::handler::Handler;
use crate::vojo::thread_local_info::ThreadLocalInfo;
use axum::handler;
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
use crate::middleware::route_service::handle_request;
use crate::vojo::gateway_request;
use crate::vojo::gateway_request::GatewayRequest;
use crossbeam::channel::{bounded, select};
use futures::channel::mpsc::unbounded;
use futures::channel::oneshot::channel;
use monoio::io::Canceller;
use monoio_http_client::Client;
use std::pin::pin;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;
use tracing_subscriber::FmtSubscriber;
pub fn create_monoio_runtime(port: i32, handler: Handler) {
    let lock = handler.clone();

    let mut handler_write_lock = lock.senders.lock().unwrap();

    let first_key = handler_write_lock.keys().cloned().next().unwrap();
    let removed_list = handler_write_lock.remove(&first_key).unwrap();
    drop(handler_write_lock);
    for item in removed_list {
        item.send(1).unwrap();
    }
    let cpus = num_cpus::get();
    println!("Cpu core is {}", cpus);
    for i in 0..cpus {
        let handle_clone1 = handler.clone();

        println!("thread is {}", i);
        std::thread::spawn(move || {
            let (stop_tx, stop_rx) = channel();
            let lock = handle_clone1.clone();
            let mut handler_write_lock = lock.senders.lock().unwrap();
            handler_write_lock
                .entry(port)
                .or_insert(Vec::new())
                .push(stop_tx);
            drop(handler_write_lock);
            let mut rt = monoio::RuntimeBuilder::<monoio::IoUringDriver>::new()
                .with_entries(256)
                .enable_timer()
                .build()
                .unwrap();
            rt.block_on(async { main_with_error(port, handle_clone1).await });
        });
    }
}
pub async fn main_with_error(port: i32, handler: Handler) {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(tracing::Level::DEBUG)
        .finish();
    let _ = tracing::subscriber::set_global_default(subscriber);
    let addr = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(addr.clone()).unwrap();
    let client = Client::default();
    let thread_local_infos = Arc::new(Mutex::new(ThreadLocalInfo::new()));
    info!("Listening {}", addr);
    loop {
        if let Ok((stream, addr)) = listener.accept().await {
            monoio::spawn(handle_connection(
                port,
                client.clone(),
                handler.clone(),
                stream,
                addr.to_string(),
                thread_local_infos.clone(),
            ));
        }
    }
}
async fn handle_connection(
    port: i32,
    client: Client,
    handler: Handler,
    stream: TcpStream,
    addr: String,
    thread_local_info_mutex: Arc<Mutex<ThreadLocalInfo>>,
) {
    let (r, w) = stream.into_split();
    let sender = GenericEncoder::new(w);
    let mut receiver = RequestDecoder::new(r);
    let (mut tx, rx) = spsc_pair();
    monoio::spawn(handle_task(
        port,
        client,
        handler,
        rx,
        sender,
        addr,
        thread_local_info_mutex,
    ));

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
    port: i32,

    client: Client,
    handler: Handler,
    mut receiver: SPSCReceiver<Request>,
    mut sender: impl Sink<Response, Error = impl Into<HttpError>>,
    remote_addr: String,
    thread_local_info_mutex: Arc<Mutex<ThreadLocalInfo>>,
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

        let gateway_request = GatewayRequest::new(
            port,
            request,
            remote_addr.clone(),
            client.clone(),
            handler.clone(),
            thread_local_info_mutex.clone(),
        );

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
