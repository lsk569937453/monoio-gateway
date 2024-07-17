use crate::constants::common_constants::DEFAULT_TEMPORARY_DIR;

use crate::vojo::app_config::ApiService;
use crate::vojo::app_config::Route;
use crate::vojo::app_config::ServiceType;

use crate::vojo::app_config::AppConfig;
use crate::vojo::app_error::AppError;
use crate::vojo::base_response::BaseResponse;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::delete;
use axum::routing::{get, post, put};
use axum::Router;
use std::convert::Infallible;
use std::env;
use std::net::SocketAddr;
use std::path::Path;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
static INTERNAL_SERVER_ERROR: &str = "Internal Server Error";

async fn get_app_config(
    State(state): State<Handler>,
) -> Result<impl axum::response::IntoResponse, Infallible> {
    let app_config = state.shared_app_config.lock().await;
    let cloned_config = app_config.clone();
    drop(app_config);
    let data = BaseResponse {
        response_code: 0,
        response_object: cloned_config,
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
    axum::extract::Json(api_services_vistor): axum::extract::Json<ApiService>,
) -> Result<impl axum::response::IntoResponse, Infallible> {
    let t = match post_app_config_with_error(api_services_vistor, state).await {
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
    mut api_service: ApiService,
    handler: Handler,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let current_type = api_service.service_config.server_type.clone();
    if current_type == ServiceType::Https || current_type == ServiceType::Http2Tls {
        validate_tls_config(
            api_service.service_config.cert_str.clone(),
            api_service.service_config.key_str.clone(),
        )?;
    }
    let cloned_port = api_service.listen_port;
    let (sender, receiver) = mpsc::channel::<()>(1);
    api_service.api_service_id.clone_from(&uuid);
    api_service.sender = sender;
    let mut rw_global_lock = handler.shared_app_config.lock().await;
    rw_global_lock
        .api_service_config
        .insert(uuid.clone(), api_service);
    drop(rw_global_lock);
    let mut cloned_handler = handler.clone();
    tokio::spawn(async move {
        let lock = cloned_handler.shared_app_config.lock().await;
        let cloned_config = lock.clone();
        if let Err(err) = save_config_to_file(cloned_config).await {
            error!("Save file error,the error is {}!", err);
        }
        drop(lock);
        let _ = cloned_handler
            .start_proxy(cloned_port, receiver, current_type, uuid)
            .await;
    });
    let data = BaseResponse {
        response_code: 0,
        response_object: 0,
    };
    let json_str = serde_json::to_string(&data).unwrap();
    Ok((axum::http::StatusCode::OK, json_str))
}
async fn delete_route(
    axum::extract::Path(_route_id): axum::extract::Path<String>,
    State(state): State<Handler>,
) -> Result<impl axum::response::IntoResponse, Infallible> {
    let rw_global_lock = state.shared_app_config.lock().await;

    let cloned_config = rw_global_lock.clone();

    tokio::spawn(async {
        if let Err(err) = save_config_to_file(cloned_config).await {
            error!("Save file error,the error is {}!", err);
        }
    });

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
    let rw_global_lock = handler.shared_app_config.lock().await;

    let cloned_config = rw_global_lock.clone();
    tokio::spawn(async {
        if let Err(err) = save_config_to_file(cloned_config).await {
            error!("Save file error,the error is {}!", err);
        }
    });
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
pub async fn start_control_plane(port: i32) -> Result<(), AppError> {
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
