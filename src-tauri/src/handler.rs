use crate::mitm::{self, HttpHandler};

pub struct ProxyHandler {
    channel: tauri::ipc::Channel<String>,
}

impl ProxyHandler {
    pub fn new(channel: tauri::ipc::Channel<String>) -> Self {
        Self { channel }
    }
}

impl HttpHandler for ProxyHandler {
    async fn handle_request(
        &self,
        req: hyper::Request<mitm::Body>,
    ) -> anyhow::Result<mitm::RequestOrResponse> {
        let _ = self.channel.send(req.uri().to_string());
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
