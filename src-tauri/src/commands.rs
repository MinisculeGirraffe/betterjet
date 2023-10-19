use crate::proto::{Command, ParsedDeviceStatus};
use crate::state::AppState;
use btleplug::api::{Central, Peripheral as _};
use serde::Serialize;
use tauri::State;
use tokio::sync::RwLock;
use typeshare::typeshare;

type AppStateHandle<'a> = State<'a, RwLock<AppState>>;

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

#[typeshare]
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PeripheralResult {
    id: String,
    name: Option<String>,
    connected: bool,
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
        let mut name: Option<String> = None;

        let device = state.find_device_by_id(&id);
        if let Some(device) = device {
            name = device.get_friendly_name().await.ok();
        }

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

    // tokio::spawn(async move { update_stream(app, bedjetid, recv).await });

    Ok(())
}

#[tauri::command]
pub async fn disconnect_device(state: AppStateHandle<'_>, id: String) -> Result<(), ()> {
    state.write().await.disconnect_peripheral(&id).await;
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
    println!("status: {:?}", status);
    Ok(status.into())
}
