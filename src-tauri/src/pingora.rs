use axum::async_trait;
use pingora::{prelude::*, proxy::ProxyHttp};

struct Proxy();

#[async_trait]
impl ProxyHttp for Proxy {
    type CTX = ();

    fn new_ctx(&self) -> Self::CTX {
        ()
    }

    async fn upstream_peer(
        &self,
        session: &mut pingora::prelude::Session,
        _ctx: &mut Self::CTX,
    ) -> pingora::Result<Box<HttpPeer>> {
        tracing::debug!("{}", session.request_summary());

        let host = session
            .get_header("host")
            .expect("No Host Found!")
            .to_str()
            .expect("Invalid Host Str!");

        let addr = match host.split_once(':') {
            Some((host, port)) => (host, port.parse::<u16>().unwrap_or(80)),
            None => (host, 80),
        };

        let peer = HttpPeer::new(addr, addr.1 == 443, host.to_string());
        Ok(Box::new(peer))
    }
}

pub fn start_proxy() {
    let mut server = Server::new(None).expect("Failed to create server");
    server.bootstrap();

    let mut service = http_proxy_service(&server.configuration, Proxy());
    service.add_tcp("127.0.0.1:7777");
    service
        .add_tls(
            "127.0.0.1:7777",
            &format!("{}/certs/myCA.pem", env!("CARGO_MANIFEST_DIR")),
            &format!("{}/certs/myCA.key", env!("CARGO_MANIFEST_DIR")),
        )
        .unwrap();

    server.add_service(service);
    server.run_forever();
}
