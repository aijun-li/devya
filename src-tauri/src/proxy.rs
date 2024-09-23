use std::net::SocketAddr;

use axum::{
    body::Body,
    extract::Request,
    http::Method,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use hyper::{body::Incoming, server::conn::http1, upgrade::Upgraded, StatusCode};
use hyper_util::rt::TokioIo;
use tokio::net::{TcpListener, TcpStream};
use tower::ServiceExt;

pub async fn start_proxy() {
    let router_service = Router::new().route("/", get(|| async { "Welcome to Devya" }));

    let hyper_service = hyper::service::service_fn(move |req: Request<Incoming>| {
        let router_service = router_service.clone();
        let req = req.map(Body::new);
        async move {
            if req.method() == Method::CONNECT {
                proxy(req).await
            } else {
                router_service
                    .oneshot(req)
                    .await
                    .map_err(|err| match err {})
            }
        }
    });

    let addr = SocketAddr::from(([127, 0, 0, 1], 7777));
    tracing::debug!("listening on {}", addr);

    let listener = TcpListener::bind(addr).await.unwrap();
    loop {
        let (stream, _) = listener.accept().await.unwrap();
        let io = TokioIo::new(stream);
        let hyper_service = hyper_service.clone();
        tokio::task::spawn(async move {
            if let Err(e) = http1::Builder::new()
                .preserve_header_case(true)
                .title_case_headers(true)
                .serve_connection(io, hyper_service)
                .with_upgrades()
                .await
            {
                tracing::error!("Failed to serve connection: {:?}", e);
            }
        });
    }
}

async fn proxy(req: Request) -> Result<Response, hyper::Error> {
    tracing::trace!(?req);

    if let Some(host_addr) = req.uri().authority().map(|auth| auth.to_string()) {
        tokio::task::spawn(async move {
            match hyper::upgrade::on(req).await {
                Ok(upgraded) => {
                    if let Err(e) = tunnel(upgraded, host_addr).await {
                        tracing::warn!("server io error: {}", e);
                    }
                }
                Err(e) => tracing::warn!("upgrade error: {}", e),
            }
        });

        Ok(Response::new(Body::empty()))
    } else {
        tracing::warn!("CONNECT host is not socket addr: {:?}", req.uri());
        Ok((
            StatusCode::BAD_REQUEST,
            "CONNECT must be to a socket address",
        )
            .into_response())
    }
}

async fn tunnel(upgraded: Upgraded, addr: String) -> std::io::Result<()> {
    let mut server = TcpStream::connect(addr).await?;
    let mut upgraded = TokioIo::new(upgraded);

    let (from_client, from_server) =
        tokio::io::copy_bidirectional(&mut upgraded, &mut server).await?;

    tracing::debug!(
        "client wrote {} bytes and server wrote {} bytes",
        from_client,
        from_server
    );

    Ok(())
}
