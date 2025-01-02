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

#[derive(Debug, Default)]
#[allow(dead_code)]
pub enum IndicatorState {
    On,
    #[default]
    Off,
    Blinking,
    NotAvailable,
}

#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct ChargeState {
    pub stage: ChargeStage,
    pub indicator_0_49: IndicatorState,
    pub indicator_50_79: IndicatorState,
    pub indicator_80_99: IndicatorState,
    pub indicator_100: IndicatorState,
}

#[derive(Debug, Default)]
#[allow(dead_code)]
pub enum RemainingTime {
    Minutes(u16),
    Charging,
    #[default]
    Unavailable,
}

#[derive(Debug, Default)]
#[allow(dead_code)]
pub enum StateOfHealth {
    HealthPercentage(f32),
    #[default]
    Unavailable,
    Initializing,
}

#[derive(Debug, Default)]
#[allow(dead_code)]
pub enum StateOfCharge {
    ChargePercentage(f32),
    #[default]
    Unavailable,
    Initializing,
}

#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct BankStatus {
    pub state_of_charge: StateOfCharge,
    pub state_of_health: StateOfHealth,
    // in minutes, not available when charging.
    pub time_remaining: RemainingTime,
}

#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct Version {
    pub major: u32,
    pub minor: u8,
    pub maintenance: u8,
}

#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct VersionInfo {
    pub firmware_version: Version,
    pub hardware_version: Version,
    pub bootloader_version: Version,
    pub auxiliary_version: Version,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct BasicQuantities {
    // TODO: Flag state.
    pub voltage: f32,
    pub current: Option<f32>,
    pub temperature: Option<f32>,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct PowerAndCharge {
    // in W.
    pub power: f32,
    // in Ah.
    pub consumed_amp_hours: f32,
}

#[derive(Debug, Default)]
#[allow(dead_code)]
pub enum DeviceId {
    ExpertModular = 0x0A24,
    #[default]
    Unknown,
}

#[derive(Debug, Default)]
#[allow(dead_code)]
pub enum BrandId {
    TbsElectronics = 0x32,
    #[default]
    Unknown,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct AddressClaimed {
    pub device_id: DeviceId,
    pub brand_id: BrandId,
    pub serial_number: u32,
}

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
    AddressClaimed(AddressClaimed),
    VersionInfo(VersionInfo),
    Heartbeat,
    Unknown,
}
