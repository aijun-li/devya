use std::sync::Arc;

use async_trait::async_trait;
use serde::Serialize;
use tauri::{ipc::Channel, State};

use home::home_dir;

use crate::{
    mitm::{self, proxy::RequestHandler},
    state::AppState,
    util::is_port_free,
};

#[derive(Clone, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Captured {
    Request { id: String, content: String },
    Response { id: String, content: String },
}

#[derive(Clone)]
struct MyHandler {
    channel: Arc<Channel<Captured>>,
}

#[async_trait]
impl RequestHandler for MyHandler {
    async fn handle_request<T: Send>(
        &self,
        req: hyper::Request<T>,
        id: usize,
    ) -> hyper::Request<T> {
        self.channel
            .send(Captured::Request {
                id: id.to_string(),
                content: req.uri().to_string(),
            })
            .unwrap();
        req
    }

    async fn handle_response<T: Send>(
        &self,
        res: hyper::Response<T>,
        id: usize,
    ) -> hyper::Response<T> {
        self.channel
            .send(Captured::Response {
                id: id.to_string(),
                content: res.status().to_string(),
            })
            .unwrap();
        res
    }
}

#[tauri::command]
pub async fn start_proxy(
    channel: Channel<Captured>,
    port: Option<u16>,
    state: State<'_, AppState>,
) -> Result<(), ()> {
    let handler = tokio::spawn(mitm::init(
        port.unwrap_or(7777),
        MyHandler {
            channel: Arc::new(channel),
        },
    ));

    // TODO: error handling
    *state.proxy_handle.lock().unwrap() = Some(handler);

    Ok(())
}

#[tauri::command]
pub async fn stop_proxy(state: State<'_, AppState>) -> Result<(), ()> {
    if let Some(handle) = state.proxy_handle.lock().unwrap().take() {
        handle.abort();
    }

    tracing::info!("HTTP Proxy is stopped");

    Ok(())
}

#[tauri::command]
pub async fn install_cert() -> Result<(), String> {
    let home = home_dir().expect("Failed to get home path");
    let cert_dir = home.join(".devya/certs");

    mitm::cert::install_cert(&cert_dir).await?;

    Ok(())
}

#[tauri::command]
pub fn check_port(port: u16) -> bool {
    is_port_free(port)
}
