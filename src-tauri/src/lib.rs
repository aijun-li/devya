mod commands;
mod handler;
mod mitm;
mod state;
mod utils;

use migration::{Migrator, MigratorTrait};
use sea_orm::{Database, DbConn};
use state::{DbState, ProxyStateInner};
use tauri::{App, Manager};
use tokio::sync::Mutex;

async fn setup_db(app: &App) -> anyhow::Result<DbConn> {
    let db_url = if cfg!(debug_assertions) {
        "sqlite://dev.db".to_string()
    } else {
        let dir_path = app.path().app_data_dir()?;
        format!("sqlite://{}/devya.db", dir_path.to_string_lossy())
    };

    let conn = Database::connect(db_url).await?;
    Migrator::up(&conn, None).await?;

    Ok(conn)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::cert::check_ca_installed,
            commands::cert::install_ca,
            commands::proxy::start_proxy,
            commands::proxy::stop_proxy,
            commands::proxy::check_proxy_running,
            commands::db::get_rule_dirs,
            commands::db::upsert_rule_dir,
        ])
        .setup(|app| {
            let conn = tauri::async_runtime::block_on(setup_db(app))?;
            app.manage(DbState { conn });
            app.manage(Mutex::new(ProxyStateInner {
                shutdown_tx: None,
                port: None,
                running_count: 0,
            }));
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
