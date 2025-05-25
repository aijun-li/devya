use sea_orm::DbConn;
use tokio::sync::{broadcast, Mutex};

pub struct ProxyStateInner {
    pub shutdown_tx: Option<broadcast::Sender<()>>,
    pub port: Option<u16>,
    pub running_count: usize,
}

pub type ProxyState = Mutex<ProxyStateInner>;

pub struct DbState {
    pub conn: DbConn,
}
