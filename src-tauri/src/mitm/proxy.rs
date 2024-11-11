use std::sync::atomic::AtomicUsize;

use async_trait::async_trait;
use http_mitm_proxy::{DefaultClient, MitmProxy};
use hyper::service::service_fn;
use moka::sync::Cache;

static REQUEST_ID: AtomicUsize = AtomicUsize::new(1);

#[async_trait]
pub trait RequestHandler {
    async fn handle_request<T: Send>(
        &self,
        req: hyper::Request<T>,
        _id: usize,
    ) -> hyper::Request<T> {
        req
    }

    async fn handle_response<T: Send>(
        &self,
        res: hyper::Response<T>,
        _id: usize,
    ) -> hyper::Response<T> {
        res
    }
}

pub struct ProxyConfig {
    pub root_cert: Option<rcgen::CertifiedKey>,
    pub port: u16,
}

pub async fn start_proxy<T>(config: ProxyConfig, handler: T)
where
    T: RequestHandler + Send + Sync + Clone + 'static,
{
    let proxy = MitmProxy::new(
        // This is the root cert that will be used to sign the fake certificates
        config.root_cert,
        Some(Cache::new(128)),
    );

    let client = DefaultClient::new().unwrap();
    let server = proxy
        .bind(
            ("127.0.0.1", config.port),
            service_fn(move |req| {
                let client = client.clone();
                let handler = handler.clone();
                async move {
                    let uri = req.uri().clone();

                    // You can modify request here
                    // or You can just return response anywhere

                    let request_id = REQUEST_ID.fetch_add(1, std::sync::atomic::Ordering::Release);

                    let req = handler.handle_request(req, request_id).await;

                    let (res, _upgrade) = client.send_request(req).await?;

                    let res = handler.handle_response(res, request_id).await;

                    tracing::debug!("{} -> {}", uri, res.status());

                    // You can modify response here

                    Ok::<_, http_mitm_proxy::default_client::Error>(res)
                }
            }),
        )
        .await
        .unwrap();

    tracing::info!(
        "HTTP Proxy is listening on http://127.0.0.1:{}",
        config.port
    );

    server.await;
}
