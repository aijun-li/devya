mod commands;
mod mitm;
mod state;

use commands::{start_proxy, stop_proxy};
use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![start_proxy, stop_proxy])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
