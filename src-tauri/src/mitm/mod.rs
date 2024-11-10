mod cert;
mod proxy;

use cert::{make_root_cert, read_root_cert};
use home::home_dir;
use proxy::{start_proxy, ProxyConfig, RequestHandler};
use tokio::{fs, join};

#[derive(Clone)]
struct MyHandler {}

impl RequestHandler for MyHandler {}

pub async fn init() {
    let home = home_dir().expect("Failed to get home path");

    let cert_dir = home.join(".devya/certs");

    let cert_pair = (cert_dir.join("root.crt"), cert_dir.join("root.key"));

    let root_cert = match join!(fs::metadata(&cert_pair.0), fs::metadata(&cert_pair.1)) {
        (Ok(crt_meta), Ok(key_meta)) => {
            if crt_meta.is_file() && key_meta.is_file() {
                read_root_cert(&cert_pair)
            } else {
                make_root_cert(&cert_dir).await
            }
        }
        _ => make_root_cert(&cert_dir).await,
    };

    start_proxy(
        ProxyConfig {
            port: 7777,
            root_cert: Some(root_cert),
        },
        MyHandler {},
    )
    .await
}
