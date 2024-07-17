use monoio_http::common::request::Request;
use monoio_http_client::Client;
pub struct GatewayRequest {
    pub request: Request,
    pub remote_ip: String,
    pub client: Client,
}
impl GatewayRequest {
    pub fn new(request: Request, remote_ip: String, client: Client) -> Self {
        Self {
            request,
            remote_ip,
            client,
        }
    }
}
