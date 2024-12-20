mod commands;
mod mitm;
mod state;
mod util;

use commands::{check_port, install_cert, start_proxy, stop_proxy};
use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            start_proxy,
            stop_proxy,
            install_cert,
            check_port
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
