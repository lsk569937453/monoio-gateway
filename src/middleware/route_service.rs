use crate::middleware::log_service::LogService;
use crate::vojo::app_error::AppError;
use crate::vojo::handler::Handler;

use bytes::Bytes;

use http::{response::Builder, HeaderMap, StatusCode};

use monoio_http::{
    common::{error::HttpError, request::Request, response::Response},
    h1::{
        codec::{decoder::RequestDecoder, encoder::GenericEncoder},
        payload::{FixedPayload, Payload},
    },
    util::spsc::{spsc_pair, SPSCReceiver},
};

use crate::vojo::gateway_request::GatewayRequest;
pub async fn handle_request(gateway_request: GatewayRequest) -> Result<Response, AppError> {
    let resp_result = gateway_request
        .client
        .get("http://backend:8080/get")
        .send()
        .await;

    match resp_result {
        Ok(resp) => {
            info!("has receive response,header is:{:?}", resp.headers());
            let http_resp = resp.bytes().await.unwrap();
            let res = Payload::Fixed(FixedPayload::new(http_resp));

            Ok(Builder::new()
                .status(StatusCode::OK)
                .header("Server", "monoio-http-demo")
                .body(res)
                .unwrap())
        }
        Err(e) => {
            return Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Payload::Fixed(FixedPayload::new(Bytes::from(
                    e.to_string(),
                ))))
                .unwrap());
        }
    }
}
