use std::path::{Path, PathBuf};

use anyhow::anyhow;
use quick_cache::sync::Cache;
use tauri::Manager;

use crate::{
    handler::ProxyHandler,
    mitm::{MitmProxy, RootCA},
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
pub async fn start_proxy(app: tauri::AppHandle) -> Result<(), String> {
    let app_local_data_dir = get_app_local_data_dir(&app)?;
    let (ca_cert_path, ca_key_path) = get_cert_path(&app_local_data_dir);

    let root_ca = match check_ca_installed(app).await {
        Ok(true) => RootCA::read_from_file(ca_cert_path, ca_key_path).await,
        _ => None,
    };

    tokio::spawn(async move {
        let proxy = MitmProxy::builder()
            .with_handler(ProxyHandler)
            .with_root_ca(root_ca)
            .with_cert_cache(Cache::new(128))
            .with_addr("127.0.0.1:7777")
            .build();
        let _ = proxy.start().await;
    });

    Ok(())
}
