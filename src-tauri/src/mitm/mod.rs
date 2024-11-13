pub mod cert;
pub mod proxy;

use cert::{detect_cert, make_root_cert, read_root_cert};
use home::home_dir;
use proxy::{start_proxy, ProxyConfig, RequestHandler};

pub async fn init<T>(port: u16, handler: T)
where
    T: RequestHandler + Send + Sync + Clone + 'static,
{
    let home = home_dir().expect("Failed to get home path");

    let cert_dir = home.join(".devya/certs");

    let cert_pair = (cert_dir.join("root.crt"), cert_dir.join("root.key"));

    let root_cert = if detect_cert(&cert_dir).await {
        read_root_cert(&cert_pair)
    } else {
        make_root_cert(&cert_dir).await
    };

    start_proxy(
        ProxyConfig {
            port,
            root_cert: Some(root_cert),
        },
        handler,
    )
    .await
}
