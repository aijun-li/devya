use std::sync::Mutex;

use tokio::task::JoinHandle;

#[derive(Default)]
pub struct AppState {
    pub proxy_handle: Mutex<Option<JoinHandle<()>>>,
}
