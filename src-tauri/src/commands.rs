use std::sync::Arc;

use async_trait::async_trait;
use tauri::{ipc::Channel, AppHandle, Listener};

use crate::mitm::{self, proxy::RequestHandler};

#[derive(Clone)]
struct MyHandler {
    channel: Arc<Channel<String>>,
}

#[async_trait]
impl RequestHandler for MyHandler {
    async fn handle_request<T: Send>(&self, req: hyper::Request<T>) -> hyper::Request<T> {
        self.channel.send(req.uri().to_string()).unwrap();
        req
    }

    async fn handle_response<T: Send>(&self, res: hyper::Response<T>) -> hyper::Response<T> {
        res
    }
}

#[tauri::command]
pub async fn start_proxy(app: AppHandle, channel: Channel<String>, port: Option<u16>) {
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
