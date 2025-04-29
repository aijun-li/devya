use std::io::{Read, Write};

use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use http_body_util::BodyExt;
use hyper::Response;
use mitm::{HttpHandler, MitmProxy, RootCA};
use tracing::{debug, error};

mod mitm;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

struct Proxy;

impl HttpHandler for Proxy {
    async fn handle_request(
        &self,
        req: hyper::Request<mitm::Body>,
    ) -> anyhow::Result<mitm::RequestOrResponse> {
        println!("{:?}", req);
        Ok(mitm::RequestOrResponse::Request(req))
    }

    async fn handle_response(
        &self,
        res: hyper::Response<mitm::Body>,
    ) -> anyhow::Result<hyper::Response<mitm::Body>> {
        let (parts, body) = res.into_parts();

        let collected_body = body.collect().await?;

        let body_bytes = collected_body.to_bytes();

        // println!(
        //     "{:?}",
        //     String::from_utf8_lossy(&decompress_gzip_data(&body_bytes).unwrap())
        // );

        let new_body = mitm::full_body(compress_gzip_data(b"<div>12345</div>").unwrap());

        let new_res = Response::from_parts(parts, new_body);

        Ok(new_res)
    }
}

fn compress_gzip_data(original_data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(original_data)?;
    encoder.finish() // finish() 返回 Result<Vec<u8>, io::Error>
}

fn decompress_gzip_data(compressed_data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    let mut decoder = GzDecoder::new(compressed_data);
    let mut decompressed_data = Vec::new();
    decoder.read_to_end(&mut decompressed_data)?;
    Ok(decompressed_data)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .setup(|_| {
            tokio::spawn(async move {
                let root_ca = RootCA::new("devya").unwrap();
                let proxy = MitmProxy::builder()
                    .with_handler(Proxy)
                    .with_root_ca(root_ca)
                    .build();
                let _ = proxy.bind("127.0.0.1:8080").await;
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
