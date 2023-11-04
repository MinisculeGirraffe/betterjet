use std::sync::Arc;

use crate::proto::{Command, ParsedDeviceStatus};
use crate::state::{AppState, PeripheralResult, UserPreferences};
use btleplug::api::{Central, Peripheral as _};
use serde::Serialize;
use tauri::State;
use tokio::sync::RwLock;
use typeshare::typeshare;

type AppStateHandle<'a> = State<'a, Arc<RwLock<AppState>>>;

#[typeshare]
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AdapterResult {
    selected: String,
    adapters: Vec<String>,
}

#[tauri::command]
pub async fn get_btle_adapters(state: AppStateHandle<'_>) -> Result<AdapterResult, ()> {
    let state = state.read().await;
    let selected = state.selected_adapter.adapter_info().await.unwrap();
    let mut adapters = Vec::new();
    for adapter in &state.all_adapters {
        let name = adapter.adapter_info().await.unwrap().clone();
        adapters.push(name);
    }
    Ok(AdapterResult { selected, adapters })
}

#[tauri::command]
pub async fn scan_devices(state: AppStateHandle<'_>) -> Result<Vec<PeripheralResult>, ()> {
    let state = state.read().await;

    state.scan_devices().await.unwrap();
    let peripherals = state.get_peripherals().await.unwrap();

    let mut result: Vec<PeripheralResult> = Vec::new();

    for periph in peripherals.iter() {
        let id = periph.id().to_string();
        let connected = periph.is_connected().await.unwrap_or(false);
        let name = state.db.get_cached_name(&id);

        result.push(PeripheralResult {
            id,
            name,
            connected,
        });
    }

    Ok(result)
}

#[tauri::command]
pub async fn connect_device(state: AppStateHandle<'_>, id: String) -> Result<(), ()> {
    state.write().await.connect_peripheral(&id).await.unwrap();
    Ok(())
}

#[tauri::command]
pub async fn disconnect_device(state: AppStateHandle<'_>, id: String) -> Result<(), ()> {
    state.write().await.disconnect_peripheral(&id).await;
    Ok(())
}

#[tauri::command]
pub async fn get_config(state: AppStateHandle<'_>) -> Result<UserPreferences, ()> {
    let state = state.read().await.db.get_config().unwrap_or_default();
    Ok(state)
}

#[tauri::command]
pub async fn set_config(state: AppStateHandle<'_>, config: UserPreferences) -> Result<(), ()> {
    state.read().await.db.set_config(&config);
    Ok(())
}

#[tauri::command]
pub async fn send_command(
    state: AppStateHandle<'_>,
    id: String,
    command: Command,
) -> Result<(), ()> {
    state
        .read()
        .await
        .find_device_by_id(&id)
        .ok_or(())?
        .send_command(command)
        .await
        .map_err(|_| ())?;

    Ok(())
}

#[tauri::command]
pub async fn get_status(
    state: AppStateHandle<'_>,
    id: String,
) -> Result<Option<ParsedDeviceStatus>, ()> {
    println!("id: {:?}", id);
    let status = state
        .read()
        .await
        .find_device_by_id(&id)
        .ok_or(())?
        .get_status()
        .await
        .ok()
        .map(|i| i.into());
    Ok(status)
}
