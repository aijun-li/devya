use tokio::sync::{broadcast, Mutex};

pub struct ProxyStateInner {
    pub shutdown_tx: Option<broadcast::Sender<()>>,
    pub port: Option<u16>,
}

pub type ProxyState = Mutex<ProxyStateInner>;
