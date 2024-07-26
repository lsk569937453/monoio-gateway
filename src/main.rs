use vojo::app_error::AppError;
use vojo::handler::Handler;
mod constants;
mod control_plane;
mod middleware;
mod proxy;
mod vojo;

#[macro_use]
extern crate tracing;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate async_trait;
use crate::control_plane::rest_api::start_control_plane;

fn main() -> Result<(), AppError> {
    std::thread::scope(|s| {
        let handler = Handler::new();
        let handle_clone = handler.clone();
        s.spawn(move || {
            let _ = starts_control_plane(handle_clone);
        });
    });
    Ok(())
}
fn starts_control_plane(hander: Handler) -> Result<(), AppError> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .map_err(|e| AppError(e.to_string()))?;

    rt.block_on(async {
        let _ = start_control_plane(hander, 8870).await;
    });
    Ok(())
}
