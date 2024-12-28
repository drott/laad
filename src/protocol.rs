#[derive(Debug, Default)]
#[allow(dead_code)]
enum ChargeStage {
    Waiting,
    SoftStart,
    Bulk,
    ExtendedBulk,
    Absorption,
    Analyze,
    Float,
    Pulse,
    Equalize,
    Stop,
    Error,
    #[default]
    Unknown,
}

#[derive(Debug, Default)]
#[allow(dead_code)]
enum IndicatorState {
    On,
    #[default]
    Off,
    Blinking,
}

#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct ChargeState {
    stage: ChargeStage,
    indicator_0_49: IndicatorState,
    indicator_50_79: IndicatorState,
    indicator_80_99: IndicatorState,
    indicator_100: IndicatorState,
}

#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct BankStatus {
    pub state_of_charge: f32,
    pub state_of_health: f32,
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
pub enum TbsPg {
    Bb1st(BankStatus),
    Bb1cs(ChargeState),
    Bb1dc(BasicQuantities),
    VersionInfo(VersionInfo),
    Heartbeat,
    Unknown,
}
