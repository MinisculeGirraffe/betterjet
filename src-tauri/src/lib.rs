pub mod commands;
pub mod proto;
pub mod state;

use std::{error::Error, sync::Arc};

use commands::{
    connect_device, disconnect_device, get_btle_adapters, get_status, scan_devices, send_command,
};
use directories::ProjectDirs;
use state::AppState;
use tauri::{App, Manager, Runtime, Window, WindowEvent};
use tokio::sync::RwLock;

use crate::{
    commands::{get_config, set_config},
    state::handle_events,
};

pub fn setup_state(app: &mut App) -> Result<(), Box<dyn Error>> {
    let handle = app.handle().to_owned();
    let dirs = ProjectDirs::from("com.betterjet", "", "").expect("Could not get project dirs");
    println!("dir: {:?}", dirs);
    let db = sled::open(dirs.data_dir())?;
    tauri::async_runtime::block_on(async move {
        let state = AppState::new(handle.clone(), db).await;
        let _ = state.scan_devices().await;
        let state = Arc::new(RwLock::new(state));

        let task = {
            let state = state.clone();
            tokio::spawn(async move {
                let _ = handle_events(state).await;
            })
        };
        state.write().await.event_task = Some(task);
        handle.manage(state);
    });
    Ok(())
}

pub fn handle_window_event<R: Runtime>(window:&Window<R>, event: &WindowEvent) {
    if let tauri::WindowEvent::CloseRequested { .. } = event {
        let Some(state) = window.app_handle().try_state::<AppState>() else {
            return;
        };
        let _ = state.db.flush();
    }
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
            get_status,
            get_config,
            set_config,
        ])
        .on_window_event(handle_window_event)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
