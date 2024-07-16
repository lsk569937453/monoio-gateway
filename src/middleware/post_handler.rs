use super::common::Handler;
use anyhow::anyhow;
use http::{response::Builder, HeaderMap, StatusCode};
use monoio_http::common::request::Request;
use monoio_http::common::response::Response;
use monoio_http::h1::payload::Payload;
struct PostReqHandler {
    next: Option<Box<dyn Handler>>,
}
impl Handler for PostReqHandler {
    async fn process(&self, req: Request) -> Result<Response, anyhow::Error> {
        match &self.next {
            Some(handler) => handler.process(req),
            None => Err(anyhow!("Handler not found or end of chain")),
        }
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
