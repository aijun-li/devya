use std::{net::SocketAddr, sync::Arc};

use super::RootCA;
use anyhow::{anyhow, bail};
use http_body_util::{combinators::BoxBody, BodyExt, Empty, Full};
use hyper::{
    body::{Bytes, Incoming},
    client::conn::http1 as client_http1,
    server::conn::http1,
    service::service_fn,
    upgrade::Upgraded,
    Method, Request, Response, StatusCode, Uri,
};
use hyper_util::rt::TokioIo;
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tracing::{error, info, warn};

pub type Body = BoxBody<Bytes, anyhow::Error>;

pub struct MitmProxy<H: HttpHandler> {
    root_cert: Option<RootCA>,
    handler: Option<H>,
}

impl<H: HttpHandler> MitmProxy<H> {
    pub fn builder() -> MitmProxyBuilder<H> {
        MitmProxyBuilder {
            root_ca: None,
            handler: None,
        }
    }
}

impl<H: HttpHandler + Send + Sync + 'static> MitmProxy<H> {
    pub async fn bind<T>(self, addr: T) -> anyhow::Result<()>
    where
        T: ToSocketAddrs,
    {
        let listener = TcpListener::bind(addr).await?;
        info!("Proxy listening on {}", listener.local_addr()?);

        let proxy = Arc::new(self);

        loop {
            let (stream, client_addr) = listener.accept().await?;
            let proxy = proxy.clone();

            tokio::spawn(async move {
                let io = TokioIo::new(stream);
                if let Err(err) = http1::Builder::new()
                    .preserve_header_case(true)
                    .title_case_headers(true)
                    .serve_connection(
                        io,
                        service_fn(move |req| proxy.clone().handle_connection(req, client_addr)),
                    )
                    .with_upgrades()
                    .await
                {
                    error!("Error serving connection from {}: {}", client_addr, err)
                }
            });
        }
    }

    async fn handle_connection(
        self: Arc<Self>,
        req: Request<Incoming>,
        client_addr: SocketAddr,
    ) -> Result<Response<Body>, anyhow::Error> {
        info!(
            "Handling request from {}: {} {}",
            client_addr,
            req.method(),
            req.uri()
        );

        let req = req.map(|b| b.map_err(|e| anyhow!(e)).boxed());

        if req.method() == Method::CONNECT {
            self.handle_connect(req).await
        } else {
            self.handle_http(req).await
        }
    }

    async fn handle_http(
        self: Arc<Self>,
        req: Request<Body>,
    ) -> Result<Response<Body>, anyhow::Error> {
        let req_or_resp = if let Some(handler) = &self.handler {
            match handler.handle_request(req).await {
                Ok(r) => r,
                Err(e) => {
                    error!("Handler error on request: {}", e);
                    let resp = Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(empty_body())
                        .unwrap();
                    return Ok(resp);
                }
            }
        } else {
            RequestOrResponse::Request(req)
        };

        let final_req = match req_or_resp {
            RequestOrResponse::Request(r) => r,
            RequestOrResponse::Response(resp) => return Ok(resp),
        };

        let Some(host) = final_req.uri().host() else {
            bail!("URI has no host");
        };
        let port = final_req.uri().port_u16().unwrap_or(80);
        let addr = format!("{}:{}", host, port);

        let stream = match TcpStream::connect(&addr).await {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to connect to upstream {}: {}", addr, e);
                let resp = Response::builder()
                    .status(StatusCode::SERVICE_UNAVAILABLE)
                    .body(empty_body())
                    .unwrap();
                return Ok(resp);
            }
        };
        let io = TokioIo::new(stream);

        let (mut sender, conn) = client_http1::handshake(io).await?;

        tokio::spawn(async move {
            if let Err(e) = conn.await {
                warn!("Upstream connection error: {}", e);
            }
        });

        let res = match sender.send_request(final_req).await {
            Ok(r) => r,
            Err(e) => {
                error!("Failed to send request to upstream: {}", e);
                let resp = Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(empty_body())
                    .unwrap();
                return Ok(resp);
            }
        };

        let res = res.map(|b| b.map_err(|e| anyhow!(e)).boxed());

        let final_res = if let Some(handler) = &self.handler {
            match handler.handle_response(res).await {
                Ok(r) => r,
                Err(e) => {
                    error!("Handler error on response: {}", e);
                    let resp = Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(empty_body())
                        .unwrap();
                    return Ok(resp);
                }
            }
        } else {
            res
        };

        Ok(final_res)
    }

    async fn handle_connect(
        self: Arc<Self>,
        req: Request<Body>,
    ) -> Result<Response<Body>, anyhow::Error> {
        if let Some(addr) = host_addr(req.uri()) {
            tokio::spawn(async move {
                match hyper::upgrade::on(req).await {
                    Ok(upgraded) => {
                        if let Err(e) = tunnel(upgraded, addr).await {
                            error!("Failed to tunnel: {}", e);
                        }
                    }
                    Err(e) => error!("Failed to upgrade: {}", e),
                }
            });
            Ok(Response::new(empty_body()))
        } else {
            error!("CONNECT request has no host: {:?}", req.uri());
            let resp = Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(full_body("CONNECT must be to a socket address"))
                .unwrap();
            Ok(resp)
        }
    }
}

#[derive(Default)]
pub struct MitmProxyBuilder<H: HttpHandler> {
    root_ca: Option<RootCA>,
    handler: Option<H>,
}

impl<H: HttpHandler> MitmProxyBuilder<H> {
    pub fn with_root_ca(mut self, root_ca: RootCA) -> Self {
        self.root_ca = Some(root_ca);
        self
    }

    pub fn with_handler(mut self, handler: H) -> Self {
        self.handler = Some(handler);
        self
    }

    pub fn build(self) -> MitmProxy<H> {
        MitmProxy {
            root_cert: self.root_ca,
            handler: self.handler,
        }
    }
}

pub enum RequestOrResponse {
    Request(Request<Body>),
    Response(Response<Body>),
}

#[trait_variant::make(Send)]
pub trait HttpHandler {
    async fn handle_request(&self, req: Request<Body>) -> anyhow::Result<RequestOrResponse> {
        async { Ok(RequestOrResponse::Request(req)) }
    }

    async fn handle_response(&self, res: Response<Body>) -> anyhow::Result<Response<Body>> {
        async { Ok(res) }
    }
}

pub fn empty_body() -> Body {
    Empty::<Bytes>::new()
        .map_err(|never| match never {})
        .boxed()
}

pub fn full_body<T: Into<Bytes>>(data: T) -> Body {
    Full::new(data.into())
        .map_err(|never| match never {})
        .boxed()
}

fn host_addr(uri: &Uri) -> Option<String> {
    uri.authority().map(|auth| auth.to_string())
}

async fn tunnel(upgraded: Upgraded, addr: String) -> anyhow::Result<()> {
    info!("Tunneling to {}", addr);
    let mut server = TcpStream::connect(addr).await?;
    let mut upgraded = TokioIo::new(upgraded);

    tokio::io::copy_bidirectional(&mut upgraded, &mut server).await?;

    Ok(())
}
