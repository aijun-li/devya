use quick_cache::sync::Cache;
use tauri::{Emitter, Manager, State};
use tokio::{net::TcpListener, sync::broadcast};

use crate::{
    handler::ProxyHandler,
    mitm::{MitmProxy, RootCA},
    state::ProxyState,
};

use super::{
    cert::check_ca_installed,
    utils::{get_app_data_dir, get_cert_path},
};

#[tauri::command]
pub async fn start_proxy(
    port: u16,
    channel: tauri::ipc::Channel<String>,
    app: tauri::AppHandle,
    proxy_state: State<'_, ProxyState>,
) -> Result<(), String> {
    let mut proxy_state = proxy_state.lock().await;

    if let Some(running_port) = proxy_state.port {
        if running_port == port {
            return Ok(());
        }
    }

    let listener = TcpListener::bind(("0.0.0.0", port)).await;

    let listener = listener.map_err(|e| e.to_string())?;

    proxy_state.port = Some(port);

    // shutdown previous proxy first
    let shutdown_tx = proxy_state.shutdown_tx.take();
    if let Some(tx) = shutdown_tx {
        let _ = tx.send(());
    }

    let app_data_dir = get_app_data_dir(&app)?;
    let (ca_cert_path, ca_key_path) = get_cert_path(&app_data_dir);

    let app_clone = app.clone();
    let root_ca = match check_ca_installed(app_clone).await {
        Ok(true) => RootCA::read_from_file(ca_cert_path, ca_key_path).await,
        _ => None,
    };

    let (tx, _) = broadcast::channel::<()>(1);

    proxy_state.shutdown_tx = Some(tx.clone());
    proxy_state.running_count += 1;

    tokio::spawn(async move {
        let _ = app.emit("proxy-started", ());
        let proxy = MitmProxy::builder()
            .with_handler(ProxyHandler::new(channel))
            .with_root_ca(root_ca)
            .with_cert_cache(Cache::new(128))
            .with_shutdown(tx)
            .build();
        let _ = proxy.start(listener).await;
        let _ = app.emit("proxy-stopped", ());

        let proxy_state = app.state::<ProxyState>();
        let mut proxy_state = proxy_state.lock().await;
        proxy_state.running_count -= 1;
    });

    Ok(())
}

#[tauri::command]
pub async fn stop_proxy(proxy_state: State<'_, ProxyState>) -> Result<(), String> {
    let mut proxy_state = proxy_state.lock().await;
    if let Some(tx) = proxy_state.shutdown_tx.take() {
        let _ = tx.send(());
    }
    Ok(())
}

#[derive(serde::Serialize)]
pub struct CheckProxyRunningResp {
    port: Option<u16>,
    running_count: usize,
}

#[tauri::command]
pub async fn check_proxy_running(
    proxy_state: State<'_, ProxyState>,
) -> Result<CheckProxyRunningResp, String> {
    let proxy_state = proxy_state.lock().await;
    Ok(CheckProxyRunningResp {
        port: proxy_state.port,
        running_count: proxy_state.running_count,
    })
}
