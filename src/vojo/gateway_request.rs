use super::handler::Handler;
use crate::vojo::thread_local_info::ThreadLocalInfo;
use monoio_http::common::request::Request;
use monoio_http_client::Client;
use std::sync::Arc;
use std::sync::Mutex;
pub struct GatewayRequest {
    pub request: Request,
    pub remote_ip: String,
    pub client: Client,
    pub handler: Handler,
    pub thread_local_info_mutex: Arc<Mutex<ThreadLocalInfo>>,
}
impl GatewayRequest {
    pub fn new(
        request: Request,
        remote_ip: String,
        client: Client,
        handler: Handler,
        thread_local_info_mutex: Arc<Mutex<ThreadLocalInfo>>,
    ) -> Self {
        Self {
            request,
            remote_ip,
            client,
            handler,
            thread_local_info_mutex,
        }
    }
}
