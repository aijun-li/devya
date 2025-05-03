use std::{net::SocketAddr, sync::Arc};

use super::RootCA;
use anyhow::anyhow;
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
use tracing::{debug, error, info, warn};

pub type Body = BoxBody<Bytes, anyhow::Error>;

pub struct MitmProxy<A: ToSocketAddrs, H: HttpHandler> {
    bind_addr: Option<A>,
    root_cert: Option<RootCA>,
    handler: Option<H>,
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
            handler: None,
        }
    }
}

impl<A, H> MitmProxy<A, H>
where
    A: ToSocketAddrs + Send + Sync + 'static,
    H: HttpHandler + Send + Sync + 'static,
{
    pub async fn start(self) -> anyhow::Result<()> {
        let Some(addr) = &self.bind_addr else {
            warn!("No bind address");
            return Ok(());
        };

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

        let Some(host) = final_req.uri().host() else {
            error!("URI has no host");
            return Ok(error_response(StatusCode::BAD_REQUEST, "URI has no host"));
        };
        let port = final_req.uri().port_u16().unwrap_or(80);
        let addr = format!("{}:{}", host, port);

        let stream = match TcpStream::connect(&addr).await {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to connect to upstream {}: {}", addr, e);
                return Ok(error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    &format!("Failed to connect to upstream {}", addr),
                ));
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
        debug!("Client TLS handshake successful for {}", host_for_cert);

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
        debug!("Server TLS handshake successful for {}", host_for_cert);

        let server_io = TokioIo::new(server_tls_stream);
        let (sender, conn) = client_http1::handshake(server_io).await?;

        tokio::spawn(async move {
            if let Err(e) = conn.await {
                warn!("Upstream connection error: {}", e);
            }
        });

        let sender = Arc::new(tokio::sync::Mutex::new(sender));

        let proxy = self.clone();

        let service = service_fn(move |mut req: Request<Incoming>| {
            let proxy = proxy.clone();
            let sender = sender.clone();
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

                let mut sender = sender.lock().await;

                let res = match sender.send_request(final_req).await {
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

    async fn get_final_req(&self, req: Request<Incoming>) -> RequestOrResponse {
        let req = req.map(|b| b.map_err(|e| anyhow!(e)).boxed());

        let Some(handler) = &self.handler else {
            return RequestOrResponse::Request(req);
        };

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
}

#[derive(Default)]
pub struct MitmProxyBuilder<A: ToSocketAddrs, H: HttpHandler> {
    bind_addr: Option<A>,
    root_ca: Option<RootCA>,
    handler: Option<H>,
}

impl<A, H> MitmProxyBuilder<A, H>
where
    A: ToSocketAddrs,
    H: HttpHandler,
{
    pub fn with_root_ca(mut self, root_ca: RootCA) -> Self {
        self.root_ca = Some(root_ca);
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

    pub fn build(self) -> MitmProxy<A, H> {
        MitmProxy {
            bind_addr: self.bind_addr,
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
