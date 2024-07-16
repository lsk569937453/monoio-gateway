use super::common::Handler;
use anyhow::anyhow;
use http::{response::Builder, HeaderMap, StatusCode};
use monoio::io::stream::Stream;
use monoio_http::common::request::Request;
use monoio_http::common::response::Response;
use monoio_http::h1::payload::FixedPayload;
use monoio_http::h1::payload::Payload;
struct CoreReqHandler {
    next: Option<Box<dyn Handler>>,
}
impl Handler for CoreReqHandler {
    async fn process(&self, req: Request) -> Result<Response, anyhow::Error> {
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
        // Builder::new()
        //     .status(400)
        //     .header("Server", "monoio-http-demo")
        //     .body(Payload::None)
        //     .unwrap()
    }
    fn set_next(&mut self, next: Box<dyn Handler>) {
        self.next = Some(next);
    }
}
