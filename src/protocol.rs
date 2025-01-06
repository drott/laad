/*
 * Copyright (c) 2024 Dominik Röttsches
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */

//! This module defines known enums, structs, and implementations related to the protocol
//! for the TBS battery monitors and chargers.

/// Represents the different stages of charging, used in the [charge state](ChargeState) message.
#[derive(Debug, Default)]
#[allow(dead_code)]
#[repr(u8)]
pub enum ChargeStage {
    Waiting = 0,
    SoftStart = 1,
    Bulk = 2,
    ExtendedBulk = 3,
    Absorption = 4,
    Analyze = 6,
    Float = 8,
    Pulse = 9,
    Equalize = 11,
    Stop = 13,
    Error = 15,
    Unavailable = 255,
    #[default]
    Unknown = 5,
}

/// Represents the state of an indicator.
#[derive(Debug, Default)]
#[allow(dead_code)]
pub enum IndicatorState {
    On,
    #[default]
    Off,
    Blinking,
    NotAvailable,
}

/// Represents the state of charge including indicators for ranges of charge levels.
#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct ChargeState {
    pub stage: ChargeStage,
    pub indicator_0_49: IndicatorState,
    pub indicator_50_79: IndicatorState,
    pub indicator_80_99: IndicatorState,
    pub indicator_100: IndicatorState,
}

/// Represents the remaining time for charging or other operations, used in the [bank status](BankStatus) message.
#[derive(Debug, Default)]
#[allow(dead_code)]
pub enum RemainingTime {
    Minutes(u16),
    Charging,
    #[default]
    Unavailable,
}

/// Represents the state of health of the battery, , used in the [bank status](BankStatus) message.
#[derive(Debug, Default)]
#[allow(dead_code)]
pub enum StateOfHealth {
    HealthPercentage(f32),
    #[default]
    Unavailable,
    Initializing,
}

/// Represents the state of charge of the battery, used in the [bank status](BankStatus) message.
#[derive(Debug, Default)]
#[allow(dead_code)]
pub enum StateOfCharge {
    ChargePercentage(f32),
    #[default]
    Unavailable,
    Initializing,
}

/// Represents the status of a battery bank.
#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct BankStatus {
    pub state_of_charge: StateOfCharge,
    pub state_of_health: StateOfHealth,
    // in minutes, not available when charging.
    pub time_remaining: RemainingTime,
}

/// Represents the version information of firmware, hardware, bootloader, and auxiliary components, used in the [version info](VersionInfo) message.
#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct Version {
    pub major: u32,
    pub minor: u8,
    pub maintenance: u8,
}

/// Contains version information for firmware, hardware, bootloader, and auxiliary components.
#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct VersionInfo {
    pub firmware_version: Version,
    pub hardware_version: Version,
    pub bootloader_version: Version,
    pub auxiliary_version: Version,
}

/// Represents the temperature in degrees Celsius or other states, used in the [basic quantities](BasicQuantities) message.
#[derive(Debug)]
#[allow(dead_code)]
pub enum Temperature {
    DegreesCelsius(f32),
    Unavailable,
    NoSensorDetected,
}

/// Represents so called "basic quantities" such as voltage, current, and temperature.
#[derive(Debug)]
#[allow(dead_code)]
pub struct BasicQuantities {
    // TODO: Flag state.
    pub voltage: Option<f32>,
    pub current: Option<f32>,
    pub temperature: Temperature,
}

/// Represents power and charge information.
#[derive(Debug)]
#[allow(dead_code)]
pub struct PowerAndCharge {
    // in W.
    pub power: Option<f32>,
    // in Ah.
    pub consumed_amp_hours: Option<f32>,
}

/// Represents the device ID, used in the [address claimed](AddressClaimed) message.
#[derive(Debug, Default)]
#[allow(dead_code)]
pub enum DeviceId {
    ExpertModular = 0x0A24,
    #[default]
    Unknown,
}

/// Represents the brand ID, used in the [address claimed](AddressClaimed) message.
#[derive(Debug, Default)]
#[allow(dead_code)]
pub enum BrandId {
    TbsElectronics = 0x32,
    #[default]
    Unknown,
}

/// Represents the address claimed by a device.
#[derive(Debug)]
#[allow(dead_code)]
pub struct AddressClaimed {
    pub device_id: DeviceId,
    pub brand_id: BrandId,
    pub serial_number: u32,
}

/// Represents the enablement state of a bank, used in the [basic setup](BasicSetup) message.
#[derive(Debug, Default)]
#[allow(dead_code)]
pub enum BankEnable {
    Disabled = 0,
    Enabled = 1,
    #[default]
    ParameterUnavailable,
}

/// Represents the chosen name of a battery bank, as configured by the user.
#[derive(Debug)]
#[allow(dead_code)]
pub enum BankName {
    BatteryBank1 = 0,
    BatteryBank2 = 1,
    BatteryBank3 = 2,
    MainBatteryBank = 3,
    AuxiliaryBatteryBank = 4,
    AuxiliaryBatteryBank1 = 5,
    AuxiliaryBatteryBank2 = 6,
    PrimaryBatteryBank = 7,
    SecondaryBatteryBank = 8,
    StarterBattery = 9,
    ServiceBatteryBank = 10,
    AccessoryBatteryBank = 11,
    HouseBatteryBank = 12,
    PortBattery = 13,
    StarboardBatteryBank = 14,
    PowerBatteryBank = 15,
    GeneratorStarterBattery = 16,
    BowThrusterBattery = 17,
    RadioBattery = 18,
    VehicleBattery = 19,
    TrailerBattery = 20,
    DrivetrainBattery = 21,
    BrakeBattery = 22,
    SolarBattery = 23,
    OtherBattery = 24,
    ParameterNotAvailable = 255,
}

/// Represents the capacity of a bank in ampere-hours.
#[derive(Debug)]
#[allow(dead_code)]
pub enum BankCapacity {
    CapacityAh(u16),
    ParameterNotAvailable,
}

/// Represents the type of battery.
#[derive(Debug)]
#[allow(dead_code)]
pub enum BatteryType {
    Flooded = 2000,
    Gel = 3000,
    AGM = 3200,
    LiFePo4 = 5000,
    ParameterNotAvailable = 65535,
}

/// Represents the basic setup of a battery bank, whether it is enabled or not, its name (from [BankName]), and the battery type, from [BatteryType].
#[derive(Debug)]
#[allow(dead_code)]
pub struct BasicSetup {
    pub bank_enable: BankEnable,
    pub bank_name: BankName,
    pub bank_capacity: BankCapacity,
    pub battery_type: BatteryType,
}

/// Represents the type of acknowledgement, used in the [acknowledgement](Acknowledgement) message.
#[derive(Debug)]
#[allow(dead_code)]
pub enum AcknowledgementType {
    PositiveAcknowledgement = 0,
    NegativeAcknowledgement = 1,
    AccessDenied = 2,
    CannotRespond = 3,
    Reserved,
}

/// Represents an acknowledgement message, received when a request for a PGN was sent.
#[allow(dead_code)]
pub struct Acknowledgement {
    pub ack_type: AcknowledgementType,
    pub pgn: u16,
}

impl std::fmt::Debug for Acknowledgement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Acknowledgement")
            .field("ack_type", &self.ack_type)
            .field("pgn", &format_args!("0x{:04X}", self.pgn))
            .finish()
    }
}

/// Represents the name of a device, as null-terminated string.
#[allow(dead_code)]
pub struct DeviceName {
    pub name: [u8; 32],
}

impl std::fmt::Debug for DeviceName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let null_terminated_name = std::str::from_utf8(&self.name)
            .unwrap_or("")
            .trim_end_matches('\0');
        f.debug_struct("DeviceName")
            .field("name", &null_terminated_name)
            .finish()
    }
}

/// Represents the operating mode of a device, used in the [operating mode status](OperatingModeStatus).
#[derive(Debug)]
#[allow(dead_code)]
pub enum OperatingMode {
    DeviceOff = 0,
    DeviceBooting = 1,
    DeviceWaitingForSlaves = 2,
    DeviceWaitingForMaster = 3,
    DeviceOn = 10,
    DeviceOnNightMode = 11,
    DeviceInError = 127,
    ParameterNotAvailable = 255,
}

/// Represents the installer lock state, used in the [operating mode status](OperatingModeStatus).
#[derive(Debug)]
#[allow(dead_code)]
pub enum InstallerLock {
    InstallerLockOff = 0,
    InstallerLockOn = 1,
    ParameterNotAvailable,
}

/// Operating mode status of a device.
#[derive(Debug)]
#[allow(dead_code)]
pub struct OperatingModeStatus {
    pub mode: OperatingMode,
    pub installer_lock: InstallerLock,
}

/// TBS protocol messages.
#[derive(Debug)]
#[allow(dead_code)]
pub enum TbsPg {
    Bb1dc(BasicQuantities),
    Bb2dc(BasicQuantities),
    Bb3dc(BasicQuantities),
    Bb1pc(PowerAndCharge),
    Bb2pc(PowerAndCharge),
    Bb3pc(PowerAndCharge),
    Bb1st(BankStatus),
    Bb2st(BankStatus),
    Bb3st(BankStatus),
    Bb1cs(ChargeState),
    Bb2cs(ChargeState),
    Bb3cs(ChargeState),
    Bb1bs(BasicSetup),
    Bb2bs(BasicSetup),
    Bb3bs(BasicSetup),
    AddressClaimed(AddressClaimed),
    VersionInfo(VersionInfo),
    Heartbeat,
    Acknowledgement(Acknowledgement),
    DeviceName(DeviceName),
    OperatingModeStatus(OperatingModeStatus),
    Unknown,
}
