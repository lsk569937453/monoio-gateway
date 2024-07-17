use futures::task::Context;
use futures::task::Poll;
use monoio_http::common::request::Request;

use tower::Service;

use crate::vojo::gateway_request::GatewayRequest;
// A middleware that logs requests before forwarding them to another service
pub struct LogService<S> {
    pub target: &'static str,
    pub service: S,
}

impl<S> Service<GatewayRequest> for LogService<S>
where
    S: Service<GatewayRequest>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, request: GatewayRequest) -> Self::Future {
        // info!("ip is:{}", request.remote_ip);
        self.service.call(request)
    }
}
