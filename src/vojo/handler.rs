use crate::vojo::app_config::AppConfig;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;
#[derive(Clone)]
pub struct Handler {
    pub shared_app_config: Arc<RwLock<AppConfig>>,
}
impl Handler {
    pub fn new() -> Self {
        Self {
            shared_app_config: Arc::new(RwLock::new(Default::default())),
        }
    }
}
