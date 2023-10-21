use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::FromPrimitive;
use serde::{Deserialize, Serialize};
use std::{
    io::{self, Read},
    time::Duration,
};
use thiserror::Error;

use typeshare::typeshare;

#[derive(Error, Debug)]
pub enum InterfaceError {
    #[error("Invalid Data provided to protocol")]
    InvalidParameter,
    #[error("I/O Error: {0}")]
    IOError(#[from] io::Error),
    #[error("Invalid Data provided to protocol")]
    NotEnoughData,
}

pub trait Encode
where
    Self: Sized,
{
    fn encode(&self) -> Result<Vec<u8>, InterfaceError> {
        let mut bytes = Vec::new();
        self.write_to(&mut bytes)?;
        Ok(bytes)
    }

    fn write_to<W: io::Write>(&self, writer: &mut W) -> Result<(), InterfaceError>;
}

pub trait Decode: Sized {
    fn read_from<R: Read>(reader: R) -> Result<Self, InterfaceError>;
}

#[typeshare]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, FromPrimitive, ToPrimitive, Serialize)]
pub enum OperatingMode {
    Standby = 0,
    NormalHeat = 1,
    TurboHeat = 2,
    ExtendedHeat = 3,
    Cool = 4,
    Dry = 5,
    Wait = 6,
}

#[typeshare]
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct DeviceStatus {
    /// The total runtime left on the device
    pub remaining_hours: u8,
    pub remaining_minutes: u8,
    pub remaining_seconds: u8,
    /// Stored in units of 0.5 degrees celsius
    pub actual_temp: u8,
    /// Stored in units of 0.5 degrees celsius
    pub target_temp: u8,
    pub operating_mode: OperatingMode,
    /// Represented as a number between 0-19
    pub fan_step: u8,
    /// Maximum runtime for the current mode
    pub max_duration_hours: u8,
    pub max_duration_minutes: u8,
    /// Stored in units of 0.5 degrees celsius
    pub min_target_temp: u8,
    /// Stored in units of 0.5 degrees celsius
    pub max_target_temp: u8,
    /// Stored in units of 0.5 degrees celsius
    pub ambient_temp: u8,
    pub shutdown_code: ShutDownCode,
    pub update_status: UpdateStatus,
}

impl Decode for DeviceStatus {
    fn read_from<R: Read>(mut reader: R) -> Result<Self, InterfaceError> {
        let mut packet = [0u8; 27];
        reader.read_exact(&mut packet)?;
        let operating_mode =
            OperatingMode::from_u8(packet[8]).ok_or_else(|| InterfaceError::InvalidParameter)?;
        let shutdown_code =
            ShutDownCode::from_u8(packet[17]).ok_or_else(|| InterfaceError::InvalidParameter)?;
        let update_status =
            UpdateStatus::from_u8(packet[25]).ok_or_else(|| InterfaceError::InvalidParameter)?;

        Ok(Self {
            remaining_hours: packet[3],
            remaining_minutes: packet[4],
            remaining_seconds: packet[5],
            actual_temp: packet[6],
            target_temp: packet[7],
            operating_mode,
            fan_step: packet[9],
            max_duration_hours: packet[10],
            max_duration_minutes: packet[11],
            min_target_temp: packet[12],
            max_target_temp: packet[13],
            ambient_temp: packet[16],
            shutdown_code,
            update_status,
        })
    }
}

#[typeshare]
#[serde_with::serde_as]
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct ParsedDeviceStatus {
    #[typeshare(serialized_as = u64)]
    #[serde_as(as = "serde_with::DurationSeconds<u64>")]
    remaining_duration: Duration,
    /// As degrees C
    actual_temp: f32,
    /// As degreesC
    target_temp: f32,
    operating_mode: OperatingMode,
    ///As a percent 0 - 100
    fan_step: u8,
    #[typeshare(serialized_as = u64)]
    #[serde_as(as = "serde_with::DurationSeconds<u64>")]
    max_duration: Duration,
    min_target_temp: f32,
    max_target_temp: f32,
    ambient_temp: f32,
    shutdown_code: ShutDownCode,
    update_status: UpdateStatus,
}

impl From<DeviceStatus> for ParsedDeviceStatus {
    fn from(value: DeviceStatus) -> Self {
        let remaining_duration = Duration::from_secs(
            (value.remaining_hours as u64 * 3600)
                + (value.remaining_minutes as u64 * 60)
                + (value.remaining_seconds as u64),
        );

        let max_duration = Duration::from_secs(
            (value.max_duration_hours as u64 * 3600) + value.max_duration_minutes as u64 * 60,
        );
        Self {
            remaining_duration,
            actual_temp: value.actual_temp as f32 / 2.0,
            target_temp: value.target_temp as f32 / 2.0,
            operating_mode: value.operating_mode,
            fan_step: value.fan_step.saturating_add(1).saturating_mul(5),
            max_duration,
            min_target_temp: value.min_target_temp as f32 / 2.0,
            max_target_temp: value.max_target_temp as f32 / 2.0,
            ambient_temp: value.ambient_temp as f32 / 2.0,
            shutdown_code: value.shutdown_code,
            update_status: value.update_status,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[typeshare]
pub struct DeviceStatusEvent {
    pub id: String,
    pub status: DeviceStatus,
}

#[typeshare]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, FromPrimitive, ToPrimitive, Serialize)]
pub enum ShutDownCode {
    Normal = 0,
    InvalidADC = 1,
    ThermistorTrackingError = 2,
    FastOverTempTrip = 3,
    SlowOverTempTrip = 4,
    FanFailure = 5,
    HeaterPowerStandby = 6,
    ExtenderThermalTrip = 7,
}

#[typeshare]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, FromPrimitive, ToPrimitive, Serialize)]
pub enum UpdateStatus {
    Idle = 0,
    Starting = 1,
    ConnectingToAP = 2,
    GotIPAddress = 3,
    CheckingConnection = 4,
    CheckingForUpdate = 5,
    Updating = 6,
    RestartingBedJet = 7,
    NoWiFiConfig = 20,
    UnableToConnect = 21,
    DHCPFailure = 22,
    UnableToContactServer = 23,
    ConnectionTestOK = 24,
    ConnectionTestFailed = 25,
    NoUpdateNeeded = 26,
    RadioDisabled = 27,
    RestartingBedJetTerminal = 28,
    UpdateFailed = 29,
}

#[typeshare]
#[repr(u8)]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, FromPrimitive, ToPrimitive, Serialize, Deserialize,
)]
pub enum ButtonCode {
    Stop = 0x01,
    Cool = 0x02,
    Heat = 0x03,
    Turbo = 0x04,
    Dry = 0x05,
    ExternalHeat = 0x06,
    FanUp = 0x10,
    FanDown = 0x11,
    TempUp1C = 0x12,
    TempDown1C = 0x13,
    TempUp1F = 0x14,
    TempDown1F = 0x15,
    Memory1Recall = 0x20,
    Memory2Recall = 0x21,
    Memory3Recall = 0x22,
    Memory1Store = 0x28,
    Memory2Store = 0x29,
    Memory3Store = 0x2a,
    StartConnectionTest = 0x42,
    StartFirmwareUpdate = 0x43,
    SetLowPowerMode = 0x44,
    SetNormalPowerMode = 0x45,
    EnableRingOfLight = 0x46,
    DisableRingOfLight = 0x47,
    MuteBeeper = 0x48,
    UnmuteBeeper = 0x49,
    ResetToFactorySettings = 0x4c,
    EnableWiFiBT = 0x4d,
    DisableWiFiBT = 0x4e,
    SetConfigCompleteFlag = 0x4f,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, FromPrimitive, ToPrimitive)]
pub enum ParameterCode {
    DeviceName = 0x00,
    MemoryName1 = 0x01,
    MemoryName2 = 0x02,
    MemoryName3 = 0x03,
    BiorhythmName1 = 0x04,
    BiorhythmName2 = 0x05,
    BiorhythmName3 = 0x06,
    Biorhythm1Fragment1 = 0x07,
    Biorhythm1Fragment2 = 0x08,
    Biorhythm1Fragment3 = 0x09,
    Biorhythm1Fragment4 = 0x0a,
    Biorhythm2Fragment1 = 0x0b,
    Biorhythm2Fragment2 = 0x0c,
    Biorhythm2Fragment3 = 0x0d,
    Biorhythm2Fragment4 = 0x0e,
    Biorhythm3Fragment1 = 0x0f,
    Biorhythm3Fragment2 = 0x10,
    Biorhythm3Fragment3 = 0x11,
    Biorhythm3Fragment4 = 0x12,
    FirmwareVersionCodes = 0x20,
}

#[typeshare]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, FromPrimitive, ToPrimitive)]
pub enum CommandClass {
    Button = 0x01,
    SetTime = 0x02,
    SetTemp = 0x03,
    SetFan = 0x07,
    SetClock = 0x08,
    SetParameter = 0x40,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
#[typeshare]
pub enum TempParam {
    /// The temperature in degrees Celsius
    Celsius(f32),
    /// The temperature in degrees Fahrenheit
    Fahrenheit(f32),
}

impl Encode for TempParam {
    fn write_to<W: io::Write>(&self, writer: &mut W) -> Result<(), InterfaceError> {
        // The actual value we need to write is stored in units of 0.5 Celsius, so we multiply by 2
        // or convert to Celsius and multiply by 2

        let value = match self {
            TempParam::Celsius(val) => val * 2.0,
            TempParam::Fahrenheit(val) => (val - 32.0) * 5.0 / 9.0 * 2.0,
        };
        let value = value.clamp(0.0, 255.0) as u8;
        writer.write_all(&[value])?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
#[typeshare]
pub enum FanParam {
    Step(u8),
    Percent(u8),
}

impl FanParam {
    fn validate(&self) -> Result<(), InterfaceError> {
        match self {
            FanParam::Step(val) if *val > 19 => Err(InterfaceError::InvalidParameter),
            FanParam::Percent(val) if *val > 100 => Err(InterfaceError::InvalidParameter),
            _ => Ok(()),
        }
    }
}

impl Encode for FanParam {
    fn write_to<W: io::Write>(&self, writer: &mut W) -> Result<(), InterfaceError> {
        self.validate()?;

        let value = match self {
            FanParam::Step(val) => *val,
            FanParam::Percent(val) => val.saturating_div(5).saturating_sub(1),
        };

        writer.write_all(&[value])?;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[typeshare]
#[serde(tag = "type", content = "content")]
/// A higher level enum containing the commands that can be sent to the device, and the parameters to those commands
pub enum Command {
    Button(ButtonCode),
    SetTime { hours: u8, minutes: u8 },
    SetTemp(TempParam),
    SetFan(FanParam),
    SetClock { hours: u8, minutes: u8 },
    SetParam(SetParamKind),
}

impl Encode for Command {
    fn write_to<W: io::Write>(&self, writer: &mut W) -> Result<(), InterfaceError> {
        match self {
            Command::Button(code) => {
                writer.write_all(&[CommandClass::Button as u8, *code as u8])?
            }
            Command::SetTime { hours, minutes } => {
                writer.write_all(&[CommandClass::SetTime as u8, *hours, *minutes])?;
            }
            Command::SetTemp(temp) => {
                writer.write_all(&[CommandClass::SetTemp as u8])?;
                temp.write_to(writer)?
            }
            Command::SetFan(fan) => {
                writer.write_all(&[CommandClass::SetFan as u8])?;
                fan.write_to(writer)?
            }
            Command::SetClock { hours, minutes } => {
                writer.write_all(&[CommandClass::SetClock as u8, *hours, *minutes])?
            }
            Command::SetParam(param) => {
                writer.write_all(&[CommandClass::SetParameter as u8])?;
                param.write_to(writer)?
            }
        };

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
#[typeshare]
pub enum SetParamKind {
    /// Cannot contain a String longer than 15 bytes.
    DeviceName(String),
}

impl Encode for SetParamKind {
    fn write_to<W: io::Write>(&self, writer: &mut W) -> Result<(), InterfaceError> {
        match self {
            SetParamKind::DeviceName(name) => {
                // Validate that the string is within the allowed limit
                if name.len() > 15 {
                    return Err(InterfaceError::InvalidParameter);
                }
                // Write the header data
                writer.write_all(&[ParameterCode::DeviceName as u8, 0x10])?;
                // And then write the string
                writer.write_all(name.as_bytes())?;

                // Calculate the number of bytes to zero pad with
                let padding = 16 - name.len();

                // And write those bytes out
                io::copy(&mut io::repeat(0).take(padding as u64), writer)?;
            }
        }
        Ok(())
    }
}
