use std::path::{Path, PathBuf};

use anyhow::anyhow;
use quick_cache::sync::Cache;
use tauri::{Emitter, Manager, State};
use tokio::{net::TcpListener, sync::broadcast};

use crate::{
    handler::ProxyHandler,
    mitm::{MitmProxy, RootCA},
    state::ProxyState,
};

fn get_app_local_data_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let app_local_data_dir = app
        .path()
        .app_local_data_dir()
        .map_err(|err| anyhow!("{}", err).to_string())?;
    Ok(app_local_data_dir)
}

fn get_cert_path<T>(local_data_dir: T) -> (PathBuf, PathBuf)
where
    T: AsRef<Path>,
{
    let ca_cert_path = local_data_dir.as_ref().join("cert/ca.crt");
    let ca_key_path = local_data_dir.as_ref().join("cert/ca.key");
    (ca_cert_path, ca_key_path)
}

async fn create_ca_if_not_exists<T>(local_data_dir: T) -> anyhow::Result<()>
where
    T: AsRef<Path>,
{
    let (ca_cert_path, ca_key_path) = get_cert_path(local_data_dir);
    if !ca_cert_path.exists() || !ca_key_path.exists() {
        let root_ca = RootCA::new("Devya CA", 3650)?;
        root_ca.save_to_file(ca_cert_path, ca_key_path).await?;
    }
    Ok(())
}

#[tauri::command]
pub async fn check_ca_installed(app: tauri::AppHandle) -> Result<bool, String> {
    let app_local_data_dir = get_app_local_data_dir(&app)?;

    create_ca_if_not_exists(&app_local_data_dir)
        .await
        .map_err(|err| err.to_string())?;
    let (ca_cert_path, _) = get_cert_path(&app_local_data_dir);
    let installed = RootCA::check_installed(ca_cert_path).map_err(|err| err.to_string())?;
    Ok(installed)
}

#[tauri::command]
pub async fn install_ca(app: tauri::AppHandle) -> Result<(), String> {
    let app_local_data_dir = get_app_local_data_dir(&app)?;

    let (ca_cert_path, _) = get_cert_path(&app_local_data_dir);

    RootCA::install(ca_cert_path).map_err(|err| err.to_string())?;

    Ok(())
}

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

    let app_local_data_dir = get_app_local_data_dir(&app)?;
    let (ca_cert_path, ca_key_path) = get_cert_path(&app_local_data_dir);

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
