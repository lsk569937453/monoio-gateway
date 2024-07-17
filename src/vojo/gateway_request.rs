use monoio_http::common::request::Request;

pub struct GatewayRequest {
    pub request: Request,
    pub remote_ip: String,
}
impl GatewayRequest {
    pub fn new(request: Request, remote_ip: String) -> Self {
        Self { request, remote_ip }
    }
}
