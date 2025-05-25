use crate::mitm::RootCA;

use super::utils::{create_ca_if_not_exists, get_app_data_dir, get_cert_path};

#[tauri::command]
pub async fn check_ca_installed(app: tauri::AppHandle) -> Result<bool, String> {
    let app_data_dir = get_app_data_dir(&app)?;

    create_ca_if_not_exists(&app_data_dir)
        .await
        .map_err(|err| err.to_string())?;
    let (ca_cert_path, _) = get_cert_path(&app_data_dir);
    let installed = RootCA::check_installed(ca_cert_path).map_err(|err| err.to_string())?;
    Ok(installed)
}

#[tauri::command]
pub async fn install_ca(app: tauri::AppHandle) -> Result<(), String> {
    let app_data_dir = get_app_data_dir(&app)?;

    let (ca_cert_path, _) = get_cert_path(&app_data_dir);

    RootCA::install(ca_cert_path).map_err(|err| err.to_string())?;

    Ok(())
}
