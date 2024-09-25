use tauri::ipc::Channel;

pub mod proxy;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn start_proxy(on_event: Channel<String>) {
    proxy::start_proxy(on_event).await;
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![greet, start_proxy])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
