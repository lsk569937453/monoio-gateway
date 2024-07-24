use crate::constants::common_constants::DEFAULT_TEMPORARY_DIR;

use crate::proxy::http_proxy::create_monoio_runtime;
use crate::vojo::app_config::ApiService;
use crate::vojo::app_config::Route;

use crate::vojo::app_config::AppConfig;
use crate::vojo::app_error::AppError;
use crate::vojo::base_response::BaseResponse;
use crate::vojo::handler::Handler;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::delete;
use axum::routing::{get, post, put};
use axum::Router;
use futures::channel::mpsc::UnboundedSender as Sender;
use futures::SinkExt;
use std::convert::Infallible;
use std::env;
use std::net::SocketAddr;
use std::path::Path;
use tokio::io::AsyncWriteExt;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
static INTERNAL_SERVER_ERROR: &str = "Internal Server Error";

async fn get_app_config(
    State(state): State<Handler>,
) -> Result<impl axum::response::IntoResponse, Infallible> {
    let data = BaseResponse {
        response_code: 0,
        response_object: 0,
    };
    let res = match serde_json::to_string(&data) {
        Ok(json) => (axum::http::StatusCode::OK, json),
        Err(_) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("No route {}", INTERNAL_SERVER_ERROR),
        ),
    };
    Ok(res)
}

async fn post_app_config(
    State(state): State<Handler>,
    axum::extract::Json(api_services): axum::extract::Json<ApiService>,
) -> Result<impl axum::response::IntoResponse, Infallible> {
    let t = match post_app_config_with_error(api_services, state).await {
        Ok(r) => r.into_response(),
        Err(err) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            err.to_string(),
        )
            .into_response(),
    };
    Ok(t)
}
async fn post_app_config_with_error(
    api_service: ApiService,
    handler: Handler,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let port = api_service.listen_port;
    let app_config = handler
        .shared_app_config
        .read()
        .map_err(|e| AppError(e.to_string()))?
        .clone();
    if !app_config.api_service_config.contains_key(&port) {
        create_monoio_runtime(port, handler);
        let data = BaseResponse {
            response_code: 0,
            response_object: 0,
        };
        let json_str = serde_json::to_string(&data).unwrap();
        Ok((axum::http::StatusCode::OK, json_str))
    } else {
        Err(AppError(format!("The port {} is already in use", port)))
    }
}
async fn delete_route(
    axum::extract::Path(_route_id): axum::extract::Path<String>,
    State(state): State<Handler>,
) -> Result<impl axum::response::IntoResponse, Infallible> {
    let data = BaseResponse {
        response_code: 0,
        response_object: 0,
    };
    let json_str = serde_json::to_string(&data).unwrap();
    Ok((axum::http::StatusCode::OK, json_str))
}

async fn put_route(
    State(state): State<Handler>,
    axum::extract::Json(route_vistor): axum::extract::Json<Route>,
) -> Result<impl axum::response::IntoResponse, Infallible> {
    match put_route_with_error(route_vistor, state).await {
        Ok(r) => Ok((axum::http::StatusCode::OK, r)),
        Err(e) => Ok((axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}
async fn put_route_with_error(_route_vistor: Route, handler: Handler) -> Result<String, AppError> {
    let data = BaseResponse {
        response_code: 0,
        response_object: 0,
    };
    Ok(serde_json::to_string(&data).unwrap())
}
async fn save_config_to_file(data: AppConfig) -> Result<(), AppError> {
    let result: bool = Path::new(DEFAULT_TEMPORARY_DIR).is_dir();
    if !result {
        let path = env::current_dir().map_err(|e| AppError(e.to_string()))?;
        let absolute_path = path.join(DEFAULT_TEMPORARY_DIR);
        std::fs::create_dir_all(absolute_path).map_err(|e| AppError(e.to_string()))?;
    }

    let mut f = tokio::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("temporary/new_silverwind_config.yml")
        .await
        .map_err(|e| AppError(e.to_string()))?;
    let api_service_str = serde_yaml::to_string(&data).map_err(|e| AppError(e.to_string()))?;
    f.write_all(api_service_str.as_bytes())
        .await
        .map_err(|e| AppError(e.to_string()))?;
    Ok(())
}

pub fn get_router(handler: Handler) -> Router {
    axum::Router::new()
        .route("/appConfig", get(get_app_config).post(post_app_config))
        .route("/route/:id", delete(delete_route))
        .route("/route", put(put_route))
        .with_state(handler)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
}
pub async fn start_control_plane(handler: Handler, port: i32) -> Result<(), AppError> {
    let app = get_router(handler);

    let addr = SocketAddr::from(([0, 0, 0, 0], port as u16));

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| AppError(e.to_string()))?;
    axum::serve(listener, app)
        .await
        .map_err(|e| AppError(e.to_string()))?;
    info!("The admin port is {}", port);
    println!("Listening on http://{}", addr);
    Ok(())
}
