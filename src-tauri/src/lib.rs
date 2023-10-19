pub mod commands;
pub mod proto;
pub mod state;

use commands::{
    connect_device, disconnect_device, get_btle_adapters, get_status, scan_devices, send_command,
};
use state::AppState;
use tauri::{App, Manager};
use tokio::sync::RwLock;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let handle = app.handle().to_owned();
            tauri::async_runtime::block_on(async move {
                let state = AppState::new(handle.clone()).await;
                handle.manage(RwLock::new(state));
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_btle_adapters,
            scan_devices,
            connect_device,
            disconnect_device,
            send_command,
            get_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
