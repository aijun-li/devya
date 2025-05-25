use crate::mitm::RootCA;
use anyhow::anyhow;
use std::path::{Path, PathBuf};
use tauri::Manager;

pub fn get_app_data_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|err| anyhow!("{}", err).to_string())?;
    Ok(app_data_dir)
}

pub fn get_cert_path<T>(local_data_dir: T) -> (PathBuf, PathBuf)
where
    T: AsRef<Path>,
{
    let ca_cert_path = local_data_dir.as_ref().join("cert/ca.crt");
    let ca_key_path = local_data_dir.as_ref().join("cert/ca.key");
    (ca_cert_path, ca_key_path)
}

pub async fn create_ca_if_not_exists<T>(local_data_dir: T) -> anyhow::Result<()>
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
