use std::sync::Arc;

use async_trait::async_trait;
use serde::Serialize;
use tauri::{ipc::Channel, AppHandle, Listener};

use crate::mitm::{self, proxy::RequestHandler};

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
pub async fn start_proxy(app: AppHandle, channel: Channel<Captured>, port: Option<u16>) {
    let handler = tokio::spawn(mitm::init(
        port.unwrap_or(7777),
        MyHandler {
            channel: Arc::new(channel),
        },
    ));

    // handler.abort();

    app.once("stop_proxy", move |_| {
        tracing::info!("HTTP Proxy is stopped");
        handler.abort();
    });
}
