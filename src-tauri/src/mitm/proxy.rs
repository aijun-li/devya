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
use rustls::{
    pki_types::{CertificateDer, PrivateKeyDer},
    ClientConfig, RootCertStore, ServerConfig,
};
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tokio_rustls::{TlsAcceptor, TlsConnector};
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

    async fn handle_http(self: Arc<Self>, req: Request<Body>) -> anyhow::Result<Response<Body>> {
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

    async fn handle_connect(self: Arc<Self>, req: Request<Body>) -> anyhow::Result<Response<Body>> {
        let host = req
            .uri()
            .host()
            .ok_or_else(|| anyhow!("CONNECT request has no host"))?
            .to_string();

        if let Some(addr) = host_addr(req.uri()) {
            tokio::spawn(async move {
                match hyper::upgrade::on(req).await {
                    Ok(upgraded) => {
                        if self.root_cert.is_none() {
                            if let Err(e) = tunnel(upgraded, addr).await {
                                error!("Failed to tunnel: {}", e);
                            }
                        } else {
                            if let Err(e) = self.handle_tls(upgraded, addr, host).await {
                                error!("Failed to handle TLS: {}", e);
                            }
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

    async fn handle_tls(
        self: Arc<Self>,
        upgraded: Upgraded,
        target_addr: String,
        host_for_cert: String,
    ) -> anyhow::Result<()> {
        info!("Initiating TLS interception for {}", target_addr);

        let root_ca = self
            .root_cert
            .as_ref()
            .ok_or_else(|| anyhow!("No root CA"))?;
        let signed_cert = root_ca.sign(&host_for_cert)?;

        let server_cert = CertificateDer::from(signed_cert.cert.der().as_ref().to_owned());
        let server_key = PrivateKeyDer::Pkcs8(signed_cert.key_pair.serialize_der().into());

        let server_config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(vec![server_cert.clone()], server_key)?;
        let acceptor = TlsAcceptor::from(Arc::new(server_config));

        let client_io = TokioIo::new(upgraded);
        let client_tls_stream = acceptor.accept(client_io).await?;
        info!("Client TLS handshake successful for {}", host_for_cert);

        let root_cert_store = RootCertStore {
            roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
        };
        let client_config = ClientConfig::builder()
            .with_root_certificates(root_cert_store)
            .with_no_client_auth();
        let connector = TlsConnector::from(Arc::new(client_config));

        let server_tcp_stream = TcpStream::connect(&target_addr).await?;
        let server_name = host_for_cert.clone().try_into()?;
        let server_tls_stream = connector.connect(server_name, server_tcp_stream).await?;
        info!("Server TLS handshake successful for {}", host_for_cert);

        let server_io = TokioIo::new(server_tls_stream);
        let (sender, conn) = client_http1::handshake(server_io).await?;

        let sender = Arc::new(tokio::sync::Mutex::new(sender));

        tokio::spawn(async move {
            if let Err(e) = conn.await {
                warn!("Upstream connection error: {}", e);
            }
        });

        let proxy = self.clone();

        let service = service_fn(move |mut req: Request<Incoming>| {
            let proxy_clone = proxy.clone();
            let sender_clone = sender.clone();
            let host_for_cert = host_for_cert.clone();

            async move {
                info!("Intercepting request for {} {}", req.method(), req.uri());

                let original_uri = req.uri().clone();
                info!(
                    "Original URI: {:?} {:?} {:?}",
                    original_uri.scheme(),
                    original_uri.authority(),
                    original_uri
                );
                if original_uri.scheme().is_none() || original_uri.authority().is_none() {
                    let new_uri_string = format!(
                        "https://{}{}",
                        host_for_cert,
                        original_uri
                            .path_and_query()
                            .map(|pq| pq.as_str())
                            .unwrap_or("/")
                    );
                    if let Ok(new_uri) = Uri::try_from(new_uri_string) {
                        *req.uri_mut() = new_uri;
                    } else {
                        error!("Failed to parse URI");
                        let resp = Response::builder()
                            .status(StatusCode::BAD_REQUEST)
                            .body(empty_body())
                            .unwrap();
                        return Ok::<Response<Body>, anyhow::Error>(resp);
                    }
                }

                let req = req.map(|b| b.map_err(|e| anyhow!(e)).boxed());

                let req_or_resp = if let Some(handler) = &proxy_clone.handler {
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

                let mut sender = sender_clone.lock().await;

                if let Err(e) = sender.ready().await {
                    error!("Failed to send request to upstream: {}", e);
                    let resp = Response::builder()
                        .status(StatusCode::SERVICE_UNAVAILABLE)
                        .body(empty_body())
                        .unwrap();
                    return Ok(resp);
                }

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

                let final_res = if let Some(handler) = &proxy_clone.handler {
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
        });

        let client_tls_stream_io = TokioIo::new(client_tls_stream);
        if let Err(err) = http1::Builder::new()
            .preserve_header_case(true)
            .title_case_headers(true)
            .serve_connection(client_tls_stream_io, service)
            .with_upgrades()
            .await
        {
            error!("Error serving client TLS connection {}", err);
        }

        Ok(())
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
