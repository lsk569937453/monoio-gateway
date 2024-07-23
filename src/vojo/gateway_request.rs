use super::handler::Handler;
use crate::vojo::thread_local_info::ThreadLocalInfo;
use crate::AppError;
use monoio_http::common::request::Request;
use monoio_http_client::Client;
use std::sync::Arc;
use std::sync::Mutex;
pub struct GatewayRequest {
    pub port: i32,
    pub request: Request,
    pub remote_ip: String,
    pub client: Client,
    pub handler: Handler,
    pub thread_local_info_mutex: Arc<Mutex<ThreadLocalInfo>>,
}
impl GatewayRequest {
    pub fn new(
        port: i32,
        request: Request,
        remote_ip: String,
        client: Client,
        handler: Handler,
        thread_local_info_mutex: Arc<Mutex<ThreadLocalInfo>>,
    ) -> Self {
        Self {
            port,
            request,
            remote_ip,
            client,
            handler,
            thread_local_info_mutex,
        }
    }
    pub fn get_route(&self) -> Result<String, AppError> {
        let mut app_config = self
            .handler
            .shared_app_config
            .read()
            .map_err(|e| AppError(e.to_string()))?
            .clone();
        let api_service = app_config
            .api_service_config
            .get(&self.port)
            .ok_or(AppError("Can not find port in config.".to_string()))?;

        Ok("a".to_string())
    }
}
