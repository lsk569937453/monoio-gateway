use monoio_http::common::{request::Request, response::Response};
use std::future::Future;
pub trait Handler {
    async fn process(&self, req: Request) -> Result<Response, anyhow::Error>;

    fn set_next(&mut self, next: Box<dyn Handler>);
}
