mod mitm;

use tauri::{
    plugin::{Builder, TauriPlugin},
    Runtime,
};

use mitm::start_proxy;

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("proxy")
        .setup(|_, _| {
            tauri::async_runtime::spawn(async { start_proxy().await });
            Ok(())
        })
        .build()
}
