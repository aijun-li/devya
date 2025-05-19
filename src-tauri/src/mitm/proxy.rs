use std::{net::SocketAddr, sync::Arc};

use super::{RootCA, SignedCert};
use anyhow::{anyhow, Context};
use http_body_util::{combinators::BoxBody, BodyExt, Empty, Full};
use hyper::{
    body::{Bytes, Incoming},
    service::service_fn,
    upgrade::Upgraded,
    Method, Request, Response, StatusCode, Uri, Version,
};
use hyper_rustls::{ConfigBuilderExt, HttpsConnector};
use hyper_util::{
    client::legacy::{connect::HttpConnector, Client},
    rt::{TokioExecutor, TokioIo},
    server::conn::auto,
};
use quick_cache::sync::Cache;
use rustls::{
    pki_types::{CertificateDer, PrivateKeyDer},
    ClientConfig, ServerConfig,
};
use tokio::{
    net::{TcpListener, TcpStream, ToSocketAddrs},
    sync::broadcast,
};
use tokio_rustls::TlsAcceptor;
use tracing::{debug, error, info, warn};

pub type Body = BoxBody<Bytes, anyhow::Error>;

pub struct MitmProxy<A: ToSocketAddrs, H: HttpHandler> {
    bind_addr: Option<A>,
    root_cert: Option<RootCA>,
    cert_cache: Option<Cache<String, SignedCert>>,
    handler: Option<H>,
    shutdown_tx: Option<broadcast::Sender<()>>,
    http_client: Client<HttpsConnector<HttpConnector>, Body>,
}

impl<A, H> MitmProxy<A, H>
where
    A: ToSocketAddrs,
    H: HttpHandler,
{
    pub fn builder() -> MitmProxyBuilder<A, H> {
        MitmProxyBuilder {
            bind_addr: None,
            root_ca: None,
            cert_cache: None,
            handler: None,
            shutdown_tx: None,
        }
    }
}

impl<A, H> MitmProxy<A, H>
where
    A: ToSocketAddrs + Send + Sync + 'static,
    H: HttpHandler + Send + Sync + 'static,
{
    pub async fn start(mut self) -> anyhow::Result<()> {
        let Some(addr) = &self.bind_addr else {
            warn!("No bind address");
            return Ok(());
        };

        let listener = TcpListener::bind(addr).await?;
        info!("Proxy listening on {}", listener.local_addr()?);

        let (shutdown_tx, mut shutdown_rx) = match self.shutdown_tx {
            Some(ref tx) => (tx.clone(), tx.subscribe()),
            None => broadcast::channel::<()>(1),
        };

        let proxy = Arc::new(self);

        loop {
            tokio::select! {
                accept_result = listener.accept() => {
                    match accept_result {
                        Ok((stream, client_addr)) => {
                            let proxy = proxy.clone();
                            let mut shutdown_rx = shutdown_tx.subscribe();

                            tokio::spawn(async move {
                                let io = TokioIo::new(stream);
                                let mut server_builder = auto::Builder::new(TokioExecutor::new());
                                server_builder.http1()
                                    .preserve_header_case(true)
                                    .title_case_headers(true)
                                    .http2()
                                    .enable_connect_protocol();
                                let connection = server_builder
                                .serve_connection_with_upgrades(
                                    io,
                                    service_fn(move |req| proxy.clone().handle_connection(req, client_addr)),
                                );

                                tokio::select! {
                                    result = connection => {
                                        if let Err(err) = result {
                                            error!("Error serving connection from {}: {}", client_addr, err)
                                        }
                                    }
                                    _ = shutdown_rx.recv() => {
                                        info!("Shutting down connection from {}", client_addr);
                                    }
                                };
                            });
                        }
                        Err(e) => {
                            error!("Failed to accept connection: {}", e);
                        }
                    }
                }
                _ = shutdown_rx.recv() => {
                    info!("Shutting down proxy");
                    break;
                }
            }
        }

        Ok(())
    }

    async fn handle_connection(
        self: Arc<Self>,
        req: Request<Incoming>,
        client_addr: SocketAddr,
    ) -> Result<Response<Body>, anyhow::Error> {
        debug!(
            "Handling request from {}: {} {}",
            client_addr,
            req.method(),
            req.uri()
        );

        if req.method() == Method::CONNECT {
            self.handle_connect(req).await
        } else {
            self.handle_http(req).await
        }
    }

    async fn handle_http(
        self: Arc<Self>,
        req: Request<Incoming>,
    ) -> anyhow::Result<Response<Body>> {
        let final_req = match self.get_final_req(req).await {
            RequestOrResponse::Request(r) => r,
            RequestOrResponse::Response(resp) => return Ok(resp),
        };

        let res = match self.http_client.request(final_req).await {
            Ok(r) => r,
            Err(e) => {
                error!("Failed to send request to upstream: {}", e);
                return Ok(error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to send request to upstream",
                ));
            }
        };

        let final_res = self.get_final_res(res).await; // TODO: handle respones after handlin

        Ok(final_res)
    }

    async fn handle_connect(
        self: Arc<Self>,
        req: Request<Incoming>,
    ) -> anyhow::Result<Response<Body>> {
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
            Ok(error_response(
                StatusCode::BAD_REQUEST,
                "CONNECT request has no host",
            ))
        }
    }

    async fn handle_tls(
        self: Arc<Self>,
        upgraded: Upgraded,
        target_addr: String,
        host_for_cert: String,
    ) -> anyhow::Result<()> {
        debug!("Initiating TLS interception for {}", target_addr);

        let signed_cert = self
            .get_signed_cert(&host_for_cert)
            .context("Failed to sign cert")?;

        let server_cert = CertificateDer::from(signed_cert.cert);
        let server_key = PrivateKeyDer::Pkcs8(signed_cert.key_pair.into());

        let mut server_config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(vec![server_cert.clone()], server_key)?;
        server_config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
        let acceptor = TlsAcceptor::from(Arc::new(server_config));

        let client_io = TokioIo::new(upgraded);
        let client_tls_stream = acceptor.accept(client_io).await?;
        debug!("Client TLS handshake successful for {}", host_for_cert);

        let proxy = self.clone();

        let service = service_fn(move |mut req: Request<Incoming>| {
            let proxy = proxy.clone();
            let host_for_cert = host_for_cert.clone();

            async move {
                let original_uri = req.uri().clone();

                debug!(
                    "Intercepting request: {:?} {:?} {:?} {:?}",
                    req.method(),
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
                        return Ok::<Response<Body>, anyhow::Error>(error_response(
                            StatusCode::BAD_REQUEST,
                            "Failed to parse URI",
                        ));
                    }
                }

                let final_req = match proxy.get_final_req(req).await {
                    RequestOrResponse::Request(r) => r,
                    RequestOrResponse::Response(resp) => return Ok(resp),
                };

                let res = match proxy.http_client.request(final_req).await {
                    Ok(r) => r,
                    Err(e) => {
                        error!("Failed to send request to upstream: {}", e);
                        return Ok(error_response(
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "Failed to send request to upstream",
                        ));
                    }
                };

                let final_res = proxy.get_final_res(res).await;

                Ok(final_res)
            }
        });

        let client_tls_stream_io = TokioIo::new(client_tls_stream);
        let mut server_builder = auto::Builder::new(TokioExecutor::new());
        if let Err(err) = server_builder
            .http1()
            .preserve_header_case(true)
            .title_case_headers(true)
            .serve_connection_with_upgrades(client_tls_stream_io, service)
            .await
        {
            error!("Error serving client TLS connection {}", err);
        }

        Ok(())
    }

    async fn get_final_req(&self, req: Request<Incoming>) -> RequestOrResponse {
        let req = req.map(|b| b.map_err(|e| anyhow!(e)).boxed());

        let final_req = if let Some(handler) = &self.handler {
            match handler.handle_request(req).await {
                Ok(r) => r,
                Err(e) => {
                    error!("Failed to handle request: {}", e);
                    RequestOrResponse::Response(error_response(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Failed to handle request",
                    ))
                }
            }
        } else {
            return RequestOrResponse::Request(req);
        };

        match final_req {
            RequestOrResponse::Request(mut r) => {
                // reset version to enable optional HTTP2 upgrade
                *r.version_mut() = Version::default();
                RequestOrResponse::Request(r)
            }
            resp => resp,
        }
    }

    async fn get_final_res(&self, res: Response<Incoming>) -> Response<Body> {
        let res = res.map(|b| b.map_err(|e| anyhow!(e)).boxed());

        let Some(handler) = &self.handler else {
            return res;
        };

        match handler.handle_response(res).await {
            Ok(r) => r,
            Err(e) => {
                error!("Failed to handle response: {}", e);
                error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to handle response",
                )
            }
        }
    }

    fn get_signed_cert(&self, host: &str) -> anyhow::Result<SignedCert> {
        let root_cert = self
            .root_cert
            .as_ref()
            .ok_or_else(|| anyhow!("No ca root"))?;

        let Some(cache) = &self.cert_cache else {
            return root_cert.sign(host);
        };

        cache.get_or_insert_with(host, || root_cert.sign(host))
    }
}

#[derive(Default)]
pub struct MitmProxyBuilder<A: ToSocketAddrs, H: HttpHandler> {
    bind_addr: Option<A>,
    root_ca: Option<RootCA>,
    cert_cache: Option<Cache<String, SignedCert>>,
    handler: Option<H>,
    shutdown_tx: Option<broadcast::Sender<()>>,
}

impl<A, H> MitmProxyBuilder<A, H>
where
    A: ToSocketAddrs,
    H: HttpHandler,
{
    pub fn with_root_ca(mut self, root_ca: Option<RootCA>) -> Self {
        self.root_ca = root_ca;
        self
    }

    pub fn with_handler(mut self, handler: H) -> Self {
        self.handler = Some(handler);
        self
    }

    pub fn with_addr(mut self, addr: A) -> Self {
        self.bind_addr = Some(addr);
        self
    }

    pub fn with_cert_cache(mut self, cert_cache: Cache<String, SignedCert>) -> Self {
        self.cert_cache = Some(cert_cache);
        self
    }

    pub fn with_shutdown(mut self, shutdown_tx: broadcast::Sender<()>) -> Self {
        self.shutdown_tx = Some(shutdown_tx);
        self
    }

    pub fn build(self) -> MitmProxy<A, H> {
        MitmProxy {
            bind_addr: self.bind_addr,
            root_cert: self.root_ca,
            cert_cache: self.cert_cache,
            handler: self.handler,
            shutdown_tx: self.shutdown_tx,
            http_client: Self::make_http_client(),
        }
    }

    fn make_http_client() -> Client<HttpsConnector<HttpConnector>, Body> {
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

        let client_config = ClientConfig::builder()
            .with_webpki_roots()
            .with_no_client_auth();

        let https = hyper_rustls::HttpsConnectorBuilder::new()
            .with_tls_config(client_config)
            .https_or_http()
            .enable_http1()
            .enable_http2()
            .build();

        Client::builder(TokioExecutor::new()).build(https)
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

fn error_response<'a>(status: StatusCode, message: &'a str) -> Response<Body> {
    Response::builder()
        .status(status)
        .body(full_body(message.to_owned()))
        .unwrap_or_else(|_| {
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(empty_body())
                .unwrap()
        })
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
