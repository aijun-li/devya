use std::path::{Path, PathBuf};

use anyhow::Context;
use tauri::Manager;
use tracing::info;

use crate::mitm::RootCA;

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
    let app_local_data_dir = app
        .path()
        .app_local_data_dir()
        .map_err(|err| err.to_string())?;
    info!("app_local_data_dir {:?}", app_local_data_dir);

    create_ca_if_not_exists(&app_local_data_dir)
        .await
        .map_err(|err| err.to_string())?;
    let (ca_cert_path, _) = get_cert_path(&app_local_data_dir);
    let installed = RootCA::check_installed(ca_cert_path).map_err(|err| err.to_string())?;
    Ok(installed)
}

#[tauri::command]
pub async fn install_ca(app: tauri::AppHandle) -> Result<(), String> {
    let app_local_data_dir = app
        .path()
        .app_local_data_dir()
        .map_err(|err| err.to_string())?;

    let (ca_cert_path, _) = get_cert_path(&app_local_data_dir);

    RootCA::install(ca_cert_path).map_err(|err| err.to_string())?;

    Ok(())
}
