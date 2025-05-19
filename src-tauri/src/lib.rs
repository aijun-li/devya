use state::{ProxyState, ProxyStateInner};
use tauri::Manager;
use tokio::sync::Mutex;

mod commands;
mod handler;
mod mitm;
mod state;
mod utils;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::check_ca_installed,
            commands::install_ca,
            commands::start_proxy
        ])
        .setup(|app| {
            app.manage(Mutex::new(ProxyStateInner {
                shutdown_tx: None,
                port: None,
            }));
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
