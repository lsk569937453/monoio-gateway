use crate::vojo::app_config::AppConfig;
use std::sync::Arc;
use std::sync::Mutex;
#[derive(Clone)]
pub struct Handler {
    pub shared_app_config: Arc<Mutex<AppConfig>>,
}
