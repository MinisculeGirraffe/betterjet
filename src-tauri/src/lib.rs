pub mod commands;
pub mod proto;
pub mod state;

use std::error::Error;

use commands::{
    connect_device, disconnect_device, get_btle_adapters, get_status, scan_devices, send_command,
};
use directories::ProjectDirs;
use state::AppState;
use tauri::{App, Manager};
use tokio::sync::RwLock;

pub fn setup_state(app: &mut App) -> Result<(), Box<dyn Error>> {
    let handle = app.handle().to_owned();
    let dirs = ProjectDirs::from("com.betterjet", "", "").expect("Could not get project dirs");
    println!("dir: {:?}", dirs);
    let db = sled::open(dirs.data_dir())?;
    tauri::async_runtime::block_on(async move {
        let state = AppState::new(handle.clone(), db).await;
        handle.manage(RwLock::new(state));
    });
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(setup_state)
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
