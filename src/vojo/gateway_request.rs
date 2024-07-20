use monoio_http::common::request::Request;
use monoio_http_client::Client;

use super::handler::Handler;
pub struct GatewayRequest {
    pub request: Request,
    pub remote_ip: String,
    pub client: Client,
    pub handler: Handler,
}
impl GatewayRequest {
    pub fn new(request: Request, remote_ip: String, client: Client, handler: Handler) -> Self {
        Self {
            request,
            remote_ip,
            client,
            handler,
        }
    }
}
