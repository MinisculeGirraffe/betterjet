use crate::proto::{Command, Decode, DeviceStatus, Encode, InterfaceError, ParsedDeviceStatus};
use btleplug::{
    api::{
        Central, CentralEvent, Characteristic, Manager as ManagerTrait,
        Peripheral as PeripheralTrait, ScanFilter, WriteType,
    },
    platform::{Adapter, Manager, Peripheral},
};
use futures::{Future, FutureExt, Stream, StreamExt};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    io::{Cursor, Read},
    pin::Pin,
    sync::Arc,
    task::Poll,
};
use tauri::{AppHandle, Manager as TauriManager};
use thiserror::Error;
use tokio::{
    sync::{watch, Mutex, RwLock},
    task::{JoinError, JoinHandle},
};
use typeshare::typeshare;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum DeviceError {
    #[error("Failed to convert protocol stuff into bytes")]
    InterfaceError(#[from] InterfaceError),
    #[error("Bluetooth Error {0}")]
    BluetoothError(#[from] btleplug::Error),
    #[error("Failed to join Task")]
    TaskError(#[from] JoinError),
    #[error("Failed to recieve the current device status")]
    RecvError(#[from] watch::error::RecvError),
    #[error("Device did not have one of the required BT characteristics")]
    MissingCharacteristic,
    #[error("No Device by the specified Peripheral ID was found")]
    DeviceNotFound,
}

pub struct AppState {
    handle: AppHandle,
    btle_manager: Manager,
    pub selected_adapter: Adapter,
    pub event_task: Option<tokio::task::JoinHandle<()>>,
    pub all_adapters: Vec<Adapter>,
    connected_devices: Vec<BedJet>,
    pub db: DBState,
}

impl AppState {
    pub async fn new(handle: AppHandle, db: sled::Db) -> AppState {
        let manager = btleplug::platform::Manager::new().await.unwrap();
        let adapters = manager.adapters().await.unwrap();
        let value = AppState {
            handle,
            btle_manager: manager,
            selected_adapter: adapters.first().unwrap().clone(),
            event_task: None,
            all_adapters: adapters,
            connected_devices: Vec::new(),
            db: DBState::new(db),
        };

        value
    }

    async fn set_adapter(&mut self, adapter: Adapter) {
        self.selected_adapter = adapter;

        //Disconnect all the disconnected devices
        let mut connected_devices = self.connected_devices.clone();
        for device in connected_devices.iter_mut() {
            let _ = device.disconnect().await;
        }
        self.connected_devices.clear();
    }

    pub async fn scan_devices(&self) -> Result<(), btleplug::Error> {
        self.selected_adapter
            .start_scan(ScanFilter {
                services: vec![BedJet::SERVICE_UUID],
            })
            .await
    }

    pub async fn get_peripherals(&self) -> Result<Vec<Peripheral>, btleplug::Error> {
        self.selected_adapter.peripherals().await
    }

    pub fn find_device_by_id(&self, id: &str) -> Option<BedJet> {
        self.connected_devices.iter().find(|i| i.id == id).cloned()
    }

    pub async fn connect_peripheral(&mut self, id: &str) -> Result<(), DeviceError> {
        let device = self.find_device_by_id(id);
        if let Some(device) = device {
            let is_connected = device.peripheral.is_connected().await?;
            if !is_connected {
                device.connect(Some(self.handle.clone())).await?;
            }
            return Ok(());
        }

        let peripherals = self.get_peripherals().await?;
        let peripheral = peripherals
            .iter()
            .find(|i| i.id().to_string() == id)
            .cloned()
            .ok_or(DeviceError::DeviceNotFound)?;

        let bedjet = BedJet::new(peripheral, Some(self.handle.clone())).await?;
        bedjet.listen_status().await?;
        let name = bedjet.get_friendly_name().await?;
        self.db.set_cached_name(id, &name);

        self.connected_devices.push(bedjet);
        println!("Successfully added device");
        Ok(())
    }

    pub async fn disconnect_peripheral(&mut self, id: &str) {
        let device = self.find_device_by_id(id);
        if let Some(device) = device {
            let _ = device.disconnect().await;
        }
        self.connected_devices.retain(|i| i.id != id);
    }
}

#[typeshare]
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PeripheralResult {
    pub id: String,
    pub name: Option<String>,
    pub connected: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "value")]
#[typeshare]
enum DeviceEvent {
    Discovered(PeripheralResult),
    Disconnected(PeripheralResult),
    Connected(PeripheralResult),
}

pub async fn handle_events(state: Arc<RwLock<AppState>>) -> Result<(), btleplug::Error> {
    let (handle, mut events) = {
        let state = state.read().await;
        let handle = state.handle.clone();
        let events = state.selected_adapter.events().await?;
        (handle, events)
    };

    while let Some(event) = events.next().await {
        match event {
            CentralEvent::DeviceDiscovered(id) => {
                let id = id.to_string();
                let name = { state.read().await.db.get_cached_name(&id) };
                let event = DeviceEvent::Discovered(PeripheralResult {
                    id,
                    name,
                    connected: false,
                });
                println!("Emitting: {:#?}", event);
                let _ = handle.emit("DeviceEvent", event);
            }
            CentralEvent::DeviceDisconnected(id) => {
                let id = id.to_string();
                let name = { state.read().await.db.get_cached_name(&id) };
                let event = DeviceEvent::Disconnected(PeripheralResult {
                    id,
                    name,
                    connected: false,
                });
                println!("Emitting: {:#?}", event);
                let _ = handle.emit("DeviceEvent", event);
            }
            CentralEvent::DeviceConnected(id) => {
                let id = id.to_string();
                let name = { state.read().await.db.get_cached_name(&id) };
                let event = DeviceEvent::Connected(PeripheralResult {
                    id,
                    name,
                    connected: true,
                });
                println!("Emitting: {:#?}", event);
                let _ = handle.emit("DeviceEvent", event);
            }
            _ => {}
        };
    }

    Ok(())
}

#[derive(Debug, Clone)]
pub struct DBState {
    db: sled::Db,
}

impl DBState {
    pub const DEVICE_KEY: &'static str = "devices";
    pub const CONFIG_KEY: &'static str = "config";

    pub fn new(db: sled::Db) -> DBState {
        DBState { db }
    }
    pub fn flush(&self) -> Result<usize, sled::Error> {
        self.db.flush()
    }
    pub fn get_cached_name(&self, id: &str) -> Option<String> {
        self.db
            .get(format!("{}:{}", Self::DEVICE_KEY, id))
            .ok()
            .flatten()
            .as_deref()
            .map(String::from_utf8_lossy)
            .map(|i| i.to_string())
    }

    pub fn set_cached_name(&self, id: &str, name: &str) {
        self.db
            .insert(format!("{}:{}", Self::DEVICE_KEY, id), name)
            .unwrap();
    }

    pub fn get_config(&self) -> Option<UserPreferences> {
        self.db
            .get(Self::CONFIG_KEY)
            .ok()
            .flatten()
            .as_deref()
            .and_then(|i| rmp_serde::from_slice(i).ok())
    }
    pub fn set_config(&self, config: &UserPreferences) {
        println!("Setting config: {:#?}", config);
        let data = rmp_serde::to_vec(config).unwrap();
        self.db.insert(Self::CONFIG_KEY, data).unwrap();
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[typeshare]
enum TemperatureUnit {
    Fahrenheit,
    Celsius,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare]
pub struct UserPreferences {
    adapter: String,
    unit: TemperatureUnit,
    autoconnect_last_device: bool,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            adapter: String::new(),
            unit: TemperatureUnit::Fahrenheit,
            autoconnect_last_device: false,
        }
    }
}

pub struct WatchStream<T>
where
    T: Clone,
{
    reciever: watch::Receiver<T>,
    future: Option<Pin<Box<dyn Future<Output = Result<(), watch::error::RecvError>> + Send>>>,
}
impl<T> WatchStream<T>
where
    T: Clone,
{
    pub fn new(reciever: watch::Receiver<T>) -> WatchStream<T> {
        Self {
            reciever,
            future: None,
        }
    }
}

impl<T> Future for WatchStream<T>
where
    T: Clone + Send + Sync + 'static,
{
    type Output = T;

    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        {
            if self.future.is_none() {
                let mut recv = self.reciever.clone();
                let future = Box::pin(async move { recv.changed().await });
                self.future = Some(future);
            }
        }

        let poll = self.future.as_mut().unwrap().poll_unpin(cx);

        match poll {
            Poll::Ready(value) => match value {
                Ok(_) => {
                    let value = self.reciever.borrow().clone();
                    self.future = None;
                    Poll::Ready(value.clone())
                }
                Err(_) => {
                    self.future = None;
                    Poll::Pending
                }
            },
            Poll::Pending => Poll::Pending,
        }
    }
}
impl<T> Stream for WatchStream<T>
where
    T: Clone + Send + Sync + 'static,
{
    type Item = T;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        match self.poll(cx) {
            Poll::Ready(value) => Poll::Ready(Some(value)),
            Poll::Pending => Poll::Pending,
        }
    }
}

#[derive(Debug, Clone)]
/// The primary interface for interacting with the device.
pub struct BedJet {
    pub id: String,
    peripheral: Peripheral,
    device_status: Characteristic,
    friendly_name: Characteristic,
    command: Characteristic,
    extended_data: Characteristic,
    device_status_send: Arc<watch::Sender<Option<DeviceStatus>>>,
    notification_task: Arc<Mutex<Option<JoinHandle<Result<(), DeviceError>>>>>,
}

impl BedJet {
    pub const SERVICE_UUID: Uuid = Uuid::from_u128(324577607269236719219879600350580);
    pub const DEVICE_STATUS_UUID: Uuid = Uuid::from_u128(649096160927663446003035620926836);
    pub const FRIENDLY_NAME_UUID: Uuid = Uuid::from_u128(649175389090177710340629164877172);
    pub const WIFI_SSID_UUID: Uuid = Uuid::from_u128(649254617252691974678222708827508);
    pub const WIFI_PASSWORD_UUID: Uuid = Uuid::from_u128(649333845415206239015816252777844);
    pub const COMMANDS_UUID: Uuid = Uuid::from_u128(649413073577720503353409796728180);
    pub const EXTENDED_DATA_UUID: Uuid = Uuid::from_u128(649492301740234767691003340678516);

    pub async fn new(
        peripheral: Peripheral,
        handle: Option<AppHandle>,
    ) -> Result<Self, DeviceError> {
        peripheral.connect().await?;
        peripheral.discover_services().await?;

        let mut map: HashMap<Uuid, Characteristic> = peripheral
            .characteristics()
            .into_iter()
            .map(|c| (c.uuid, c))
            .collect();

        let (device_status_send, _) = watch::channel(None);
        let val = Self {
            id: peripheral.id().to_string(),
            peripheral,
            device_status: map
                .remove(&Self::DEVICE_STATUS_UUID)
                .ok_or(DeviceError::MissingCharacteristic)?,
            friendly_name: map
                .remove(&Self::FRIENDLY_NAME_UUID)
                .ok_or(DeviceError::MissingCharacteristic)?,
            command: map
                .remove(&Self::COMMANDS_UUID)
                .ok_or(DeviceError::MissingCharacteristic)?,
            extended_data: map
                .remove(&Self::EXTENDED_DATA_UUID)
                .ok_or(DeviceError::MissingCharacteristic)?,
            device_status_send: Arc::new(device_status_send),
            notification_task: Arc::new(Mutex::new(None)),
        };

        val.connect(handle).await?;

        Ok(val)
    }

    pub async fn connect(&self, handle: Option<AppHandle>) -> Result<(), DeviceError> {
        self.peripheral.connect().await?;
        self.peripheral.discover_services().await?;
        let mut task = self.notification_task.lock().await;
        println!("Task: {:?}", task);
        if let Some(handle) = task.as_ref() {
            if handle.is_finished() {
                task.take().unwrap().await??;
            } else {
                return Ok(());
            }
        }

        let inner = self.clone();
        task.replace(tokio::task::spawn(async move {
            inner.handle_notifications(handle).await
        }));

        Ok(())
    }

    pub async fn disconnect(&self) -> Result<(), DeviceError> {
        self.peripheral.disconnect().await?;
        Ok(())
    }

    async fn handle_notifications(&self, handle: Option<AppHandle>) -> Result<(), DeviceError> {
        let mut stream = self.peripheral.notifications().await?;
        while let Some(msg) = stream.next().await {
            let _ = match msg.uuid {
                BedJet::DEVICE_STATUS_UUID => {
                    self.handle_device_status(msg.value, handle.as_ref()).await
                }
                _ => Ok(()),
            };
        }
        Ok(())
    }

    pub async fn listen_status(&self) -> Result<(), btleplug::Error> {
        println!("Listening to status");
        self.peripheral.subscribe(&self.device_status).await
    }

    pub async fn unlisten_status(&self) -> Result<(), btleplug::Error> {
        println!("Unlistening to status");
        self.peripheral.unsubscribe(&self.device_status).await
    }
    pub fn subscribe_status(&self) -> watch::Receiver<Option<DeviceStatus>> {
        self.device_status_send.subscribe()
    }

    pub async fn get_status(&self) -> Result<DeviceStatus, DeviceError> {
        let mut recv = self.device_status_send.subscribe();

        let status = recv
            .wait_for(|val| val.is_some())
            .await?
            .to_owned()
            .expect("Value was checked as Some, and was actually None. This is impossible");

        Ok(status)
    }
    async fn handle_device_status(
        &self,
        message: Vec<u8>,
        handle: Option<&AppHandle>,
    ) -> Result<(), DeviceError> {
        // Calculate this up here before the cursor takes ownership
        let has_enough_bytes = message.first().is_some_and(|val| *val == 0);
        let mut cursor = Cursor::new(message);
        // We want to skip that first byte since it's just informational
        cursor.set_position(1);

        let status: DeviceStatus;
        // If the entire packet isn't contained in the message we received
        if !has_enough_bytes {
            // grab the rest of it
            let rest: Vec<u8> = self.peripheral.read(&self.device_status).await?;
            // and decode it
            status = DeviceStatus::read_from(cursor.chain(Cursor::new(rest)))?;
        } else {
            // otherwise just decode it with what we have
            status = DeviceStatus::read_from(cursor)?;
        }

        let prev = self.device_status_send.send_replace(Some(status));

        if let Some(handle) = handle {
            if prev != Some(status) {
                let status = ParsedDeviceStatus::from(status);
                let _ = handle.emit(&self.id, status);
                //  println!("Emitting: {:#?}", status);
            }
        }

        Ok(())
    }

    pub async fn get_friendly_name(&self) -> Result<String, DeviceError> {
        let data = self.peripheral.read(&self.friendly_name).await?;
        Ok(String::from_utf8_lossy(&data).to_string())
    }
    pub async fn send_command(&self, command: Command) -> Result<(), DeviceError> {
        let data = command.encode()?;
        self.peripheral
            .write(&self.command, &data, WriteType::WithoutResponse)
            .await?;

        Ok(())
    }
}
