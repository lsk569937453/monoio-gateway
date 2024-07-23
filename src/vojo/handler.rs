use crate::vojo::app_config::AppConfig;
use futures::channel::oneshot::Sender;
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;
#[derive(Clone)]
pub struct Handler {
    pub shared_app_config: Arc<RwLock<AppConfig>>,
    pub senders: Arc<Mutex<HashMap<i32, Vec<Sender<i32>>>>>,
}
impl Handler {
    pub fn new() -> Self {
        Self {
            shared_app_config: Arc::new(RwLock::new(Default::default())),
            senders: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}
