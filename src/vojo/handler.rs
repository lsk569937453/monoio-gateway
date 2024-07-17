#[derive(Clone)]
pub struct Handler {
    pub shared_app_config: Arc<Mutex<AppConfig>>,
}