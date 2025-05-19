use std::time::Duration;

use hyper::Response;
use tokio::{net::TcpListener, sync::broadcast, time::sleep};

use crate::mitm::{MitmProxy, RootCA};

use super::{full_body, Body, HttpHandler, RequestOrResponse};

struct TestHandler;

impl HttpHandler for TestHandler {
    async fn handle_request(
        &self,
        _req: hyper::Request<Body>,
    ) -> anyhow::Result<RequestOrResponse> {
        Ok(RequestOrResponse::Response(Response::new(full_body(
            "test mitm",
        ))))
    }
}

#[tokio::test]
async fn test_proxy() {
    let (shutdown_tx, _) = broadcast::channel::<()>(1);
    let root_ca = RootCA::read_from_file("./ca.crt", "./ca.key")
        .await
        .unwrap();
    let proxy = MitmProxy::builder()
        .with_handler(TestHandler)
        .with_root_ca(Some(root_ca))
        .with_shutdown(shutdown_tx.clone())
        .build();

    let listener = TcpListener::bind(("127.0.0.1", 8080)).await.unwrap();

    let proxy_handle = tokio::spawn(async move { proxy.start(listener).await });

    sleep(Duration::from_secs(1)).await;

    let _ = shutdown_tx.send(());

    let _ = proxy_handle.await;
}
