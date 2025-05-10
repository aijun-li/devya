mod commands;
mod mitm;

use std::io::{Read, Write};

use commands::{check_ca_installed, install_ca};
use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use mitm::{HttpHandler, MitmProxy, RootCA};
use quick_cache::sync::Cache;

struct Proxy;

impl HttpHandler for Proxy {
    async fn handle_request(
        &self,
        req: hyper::Request<mitm::Body>,
    ) -> anyhow::Result<mitm::RequestOrResponse> {
        // println!("{:?}", req);
        Ok(mitm::RequestOrResponse::Request(req))
    }

    async fn handle_response(
        &self,
        res: hyper::Response<mitm::Body>,
    ) -> anyhow::Result<hyper::Response<mitm::Body>> {
        // let (parts, body) = res.into_parts();

        // let collected_body = body.collect().await?;

        // let body_bytes = collected_body.to_bytes();

        // // println!(
        // //     "{:?}",
        // //     String::from_utf8_lossy(&decompress_gzip_data(&body_bytes).unwrap())
        // // );

        // let new_body = mitm::full_body(compress_gzip_data(b"<div>12345</div>").unwrap());

        // let new_res = Response::from_parts(parts, new_body);

        // Ok(new_res)
        Ok(res)
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
        .invoke_handler(tauri::generate_handler![check_ca_installed, install_ca])
        .setup(|_| {
            // tokio::spawn(async move {
            //     let root_ca = RootCA::read_from_file("./ca.crt", "./ca.key")
            //         .await
            //         .unwrap();
            //     let proxy = MitmProxy::builder()
            //         .with_handler(Proxy)
            //         .with_root_ca(root_ca)
            //         .with_cert_cache(Cache::new(128))
            //         .with_addr("127.0.0.1:7777")
            //         .build();
            //     let _ = proxy.start().await;
            // });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
