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

#[monoio::main(threads = 0)]
//very powerful
async fn main() {
    let listener = TcpListener::bind("0.0.0.0:8080").unwrap();
    println!("Listening");
    loop {
        let incoming = listener.accept().await;
        match incoming {
            Ok((stream, addr)) => {
                // println!("accepted a connection from {}", addr);
                monoio::spawn(handle_connection(stream));
            }
            Err(e) => {
                // println!("accepted connection failed: {}", e);
            }
        }
    }
}
async fn echo(mut stream: TcpStream) -> std::io::Result<()> {
    let mut buf: Vec<u8> = Vec::with_capacity(8 * 1024);
    let mut res;
    let res_str = "HTTP/1.1 200 OK\r\n";
    loop {
        // read
        (res, buf) = stream.read(buf).await;
        if res? == 0 {
            return Ok(());
        }
        let res_bytes = res_str.as_bytes();
        // write all
        (res, _) = stream.write_all(res_bytes).await;
        res?;

        // clear
        buf.clear();
    }
}
async fn handle_connection(stream: TcpStream) {
    let (r, w) = stream.into_split();
    let sender = GenericEncoder::new(w);
    let mut receiver = RequestDecoder::new(r);
    let (mut tx, rx) = spsc_pair();
    monoio::spawn(handle_task(rx, sender));

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
) -> Result<(), HttpError> {
    loop {
        let request = match receiver.recv().await {
            Some(r) => r,
            None => {
                return Ok(());
            }
        };
        let resp = handle_request(request).await;
        sender.send_and_flush(resp).await.map_err(Into::into)?;
    }
}

async fn handle_request(req: Request) -> Response {
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
    Builder::new()
        .status(status)
        .header("Server", "monoio-http-demo")
        .body(payload)
        .unwrap()
}
