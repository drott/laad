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

use crate::{
    protocol::{
        Acknowledgement, AcknowledgementType, AddressClaimed, BankCapacity, BankEnable, BankName,
        BankStatus, BasicQuantities, BasicSetup, BatteryType, BrandId, ChargeStage, ChargeState,
        DeviceId, DeviceName, IndicatorState, InstallerLock, OperatingMode, OperatingModeStatus,
        PowerAndCharge, RemainingTime, StateOfCharge, StateOfHealth, TbsPg, Temperature, Version,
        VersionInfo,
    },
    types::Frame,
};

use tracing::error;

pub struct Decoder {}

type PgnTag = [u8; 2];

const PGN_TAG_BB1DC: (PgnTag, usize) = ([0x18, 0xF0], 16);
const PGN_TAG_BB1PC: (PgnTag, usize) = ([0x19, 0xF0], 16);
const PGN_TAG_BB1ST: (PgnTag, usize) = ([0x1A, 0xF0], 16);
const PGN_TAG_BB1CS: (PgnTag, usize) = ([0x1E, 0xF0], 16);
const PGN_TAG_BB1BS: (PgnTag, usize) = ([0x20, 0xF0], 16);
const PGN_TAG_BB2DC: (PgnTag, usize) = ([0x22, 0xF0], 16);
const PGN_TAG_BB2PC: (PgnTag, usize) = ([0x23, 0xF0], 16);
const PGN_TAG_BB2ST: (PgnTag, usize) = ([0x24, 0xF0], 16);
const PGN_TAG_BB2CS: (PgnTag, usize) = ([0x28, 0xF0], 16);
const PGN_TAG_BB2BS: (PgnTag, usize) = ([0x2A, 0xF0], 16);
const PGN_TAG_BB3DC: (PgnTag, usize) = ([0x2C, 0xF0], 16);
const PGN_TAG_BB3PC: (PgnTag, usize) = ([0x2D, 0xF0], 16);
const PGN_TAG_BB3ST: (PgnTag, usize) = ([0x2E, 0xF0], 16);
const PGN_TAG_BB3CS: (PgnTag, usize) = ([0x32, 0xF0], 16);
const PGN_TAG_BB3BS: (PgnTag, usize) = ([0x34, 0xF0], 16);

const PGN_TAG_ADDRESS_CLAIMED: (PgnTag, usize) = ([0x00, 0xEE], 16);
const PGN_TAG_VERSION_INFO: (PgnTag, usize) = ([0x02, 0xF0], 16);
const PGN_TAG_HEARTBEAT: (PgnTag, usize) = ([0xFF, 0xFF], 8);
const PGN_TAG_ACKNOWLEDGEMENT: (PgnTag, usize) = ([0x00, 0xe8], 16);
const PGN_TAG_DEVICE_NAME: (PgnTag, usize) = ([0x00, 0xF0], 40);
const PGN_TAG_OPERATION_MODE: (PgnTag, usize) = ([0x0E, 0xF0], 16);

enum BankId {
    Bank1,
    Bank2,
    Bank3,
}

impl BankName {
    fn from_u8(byte: u8) -> Self {
        match byte {
            0 => BankName::BatteryBank1,
            1 => BankName::BatteryBank2,
            2 => BankName::BatteryBank3,
            3 => BankName::MainBatteryBank,
            4 => BankName::AuxiliaryBatteryBank,
            5 => BankName::AuxiliaryBatteryBank1,
            6 => BankName::AuxiliaryBatteryBank2,
            7 => BankName::PrimaryBatteryBank,
            8 => BankName::SecondaryBatteryBank,
            9 => BankName::StarterBattery,
            10 => BankName::ServiceBatteryBank,
            11 => BankName::AccessoryBatteryBank,
            12 => BankName::HouseBatteryBank,
            13 => BankName::PortBattery,
            14 => BankName::StarboardBatteryBank,
            15 => BankName::PowerBatteryBank,
            16 => BankName::GeneratorStarterBattery,
            17 => BankName::BowThrusterBattery,
            18 => BankName::RadioBattery,
            19 => BankName::VehicleBattery,
            20 => BankName::TrailerBattery,
            21 => BankName::DrivetrainBattery,
            22 => BankName::BrakeBattery,
            23 => BankName::SolarBattery,
            24 => BankName::OtherBattery,
            _ => BankName::ParameterNotAvailable,
        }
    }
}

impl ChargeStage {
    fn from_u8(charge_stage: u8) -> Self {
        match charge_stage {
            1 => ChargeStage::SoftStart,
            2 => ChargeStage::Bulk,
            3 => ChargeStage::ExtendedBulk,
            4 => ChargeStage::Absorption,
            6 => ChargeStage::Analyze,
            8 => ChargeStage::Float,
            9 => ChargeStage::Pulse,
            11 => ChargeStage::Equalize,
            13 => ChargeStage::Stop,
            15 => ChargeStage::Error,
            255 => ChargeStage::Unavailable,
            _ => ChargeStage::Unknown,
        }
    }
}

impl OperatingMode {
    fn from_u8(mode: u8) -> Self {
        match mode {
            0 => OperatingMode::DeviceOff,
            1 => OperatingMode::DeviceBooting,
            2 => OperatingMode::DeviceWaitingForSlaves,
            3 => OperatingMode::DeviceWaitingForMaster,
            10 => OperatingMode::DeviceOn,
            11 => OperatingMode::DeviceOnNightMode,
            127 => OperatingMode::DeviceInError,
            _ => OperatingMode::ParameterNotAvailable,
        }
    }
}

impl IndicatorState {
    fn from_u8(indicator_state: u8) -> Self {
        match indicator_state {
            0 => IndicatorState::Off,
            1 => IndicatorState::On,
            2 => IndicatorState::Blinking,
            3 => IndicatorState::NotAvailable,
            _ => IndicatorState::NotAvailable,
        }
    }
}

impl AcknowledgementType {
    fn from_u8(acknowledgment: u8) -> Self {
        match acknowledgment {
            0 => AcknowledgementType::PositiveAcknowledgement,
            1 => AcknowledgementType::NegativeAcknowledgement,
            2 => AcknowledgementType::AccessDenied,
            3 => AcknowledgementType::CannotRespond,
            _ => AcknowledgementType::Reserved,
        }
    }
}

impl Default for Decoder {
    fn default() -> Self {
        Self::new()
    }
}

impl Decoder {
    pub fn new() -> Self {
        Self {}
    }
    /// Decodes a given frame into a `TbsPg` type.
    ///
    /// Receives frames with bytestuffing reverted and:
    /// 1. Ensures the frame is at least 8 bytes long.
    /// 2. Extracts the PGN tag and checksum, and validates it.
    /// 4. Matches the PGN tag and frame length for known PGNs and decodes them.
    ///
    /// # Parameters
    /// - `frame`: The frame to be decoded, with bytestuffing reverted.
    ///
    /// # Returns
    /// - `TbsPg`: The decoded frame as a `TbsPg` type. If the frame is invalid or the PGN tag is unknown, it returns `TbsPg::Unknown`.
    ///
    /// # Errors
    /// - Logs an error if the checksum calculation fails or if the PGN tag is unknown.
    pub fn decode_frame(&self, frame: Frame) -> TbsPg {
        let frame_len = frame.0.len();
        if frame_len < 8 {
            return TbsPg::Unknown;
        }

        let pgn_tag = [frame.0[3], frame.0[4]];
        let checksum = frame.0[frame.0.len() - 2];
        let calculated_checksum = self.calculate_checksum(&frame);

        if let Some(calculated_checksum) = calculated_checksum {
            if checksum != calculated_checksum {
                error!(
                    "Checksum not valid for PGN tag: {:02X?}, 0x{:02X?} vs 0x{:02X}?",
                    pgn_tag, checksum, calculated_checksum
                );
                return TbsPg::Unknown;
            }
        } else {
            error!("Failed to calculate checksum for PGN tag: {:02X?}", pgn_tag);
            return TbsPg::Unknown;
        }

        match (pgn_tag, frame_len) {
            PGN_TAG_BB1ST => self.decode_bbst(BankId::Bank1, frame),
            PGN_TAG_BB2ST => self.decode_bbst(BankId::Bank2, frame),
            PGN_TAG_BB3ST => self.decode_bbst(BankId::Bank3, frame),
            PGN_TAG_VERSION_INFO => self.decode_version_info(frame),
            PGN_TAG_HEARTBEAT => TbsPg::Heartbeat,
            PGN_TAG_BB1DC => self.decode_bbdc(BankId::Bank1, frame),
            PGN_TAG_BB2DC => self.decode_bbdc(BankId::Bank2, frame),
            PGN_TAG_BB3DC => self.decode_bbdc(BankId::Bank3, frame),
            PGN_TAG_BB1PC => self.decode_bbpc(BankId::Bank1, frame),
            PGN_TAG_BB2PC => self.decode_bbpc(BankId::Bank2, frame),
            PGN_TAG_BB3PC => self.decode_bbpc(BankId::Bank3, frame),
            PGN_TAG_BB1CS => self.decode_bbcs(BankId::Bank1, &frame),
            PGN_TAG_BB2CS => self.decode_bbcs(BankId::Bank2, &frame),
            PGN_TAG_BB3CS => self.decode_bbcs(BankId::Bank3, &frame),
            PGN_TAG_BB1BS => self.decode_bbbs(BankId::Bank1, frame),
            PGN_TAG_BB2BS => self.decode_bbbs(BankId::Bank2, frame),
            PGN_TAG_BB3BS => self.decode_bbbs(BankId::Bank3, frame),
            PGN_TAG_ADDRESS_CLAIMED => self.decode_address_claimed(frame),
            PGN_TAG_ACKNOWLEDGEMENT => self.decode_acknowledgement(frame),
            PGN_TAG_DEVICE_NAME => self.decode_device_name(frame),
            PGN_TAG_OPERATION_MODE => self.decode_operating_mode(frame),
            _ => {
                error!(
                    "Unknown PGN tag: {:02X?}, frame length {:?}",
                    pgn_tag, frame_len
                );
                TbsPg::Unknown
            }
        }
    }

    fn calculate_checksum(&self, frame: &Frame) -> Option<u8> {
        if frame.0.len() < 8 {
            return None;
        }
        let sum: u8 = frame.0[1..frame.0.len() - 2]
            .iter()
            .fold(0, |acc, &b| acc.wrapping_add(b));
        Some(sum.wrapping_neg())
    }

    fn decode_bbst(&self, bank_id: BankId, frame: Frame) -> TbsPg {
        let _flags = u16::from_le_bytes([frame.0[6], frame.0[7]]);
        let soc = u16::from_le_bytes([frame.0[8], frame.0[9]]);
        let soc = if soc == 65535 {
            StateOfCharge::Unavailable
        } else if soc == 65533 {
            StateOfCharge::Initializing
        } else {
            StateOfCharge::ChargePercentage(soc as f32 / 100.0)
        };
        let soh = u16::from_le_bytes([frame.0[10], frame.0[11]]);
        let soh = if soh == 65535 {
            StateOfHealth::Unavailable
        } else if soh == 65533 {
            StateOfHealth::Initializing
        } else {
            StateOfHealth::HealthPercentage(soh as f32 / 100.0)
        };
        let time_remaining = u16::from_le_bytes([frame.0[12], frame.0[13]]);
        let time_remaining = if time_remaining == 65535 {
            RemainingTime::Unavailable
        } else if time_remaining == 65533 {
            RemainingTime::Charging
        } else {
            RemainingTime::Minutes(time_remaining)
        };
        let bank_status = BankStatus {
            state_of_charge: soc,
            state_of_health: soh,
            time_remaining,
        };
        match bank_id {
            BankId::Bank1 => TbsPg::Bb1st(bank_status),
            BankId::Bank2 => TbsPg::Bb2st(bank_status),
            BankId::Bank3 => TbsPg::Bb3st(bank_status),
        }
    }

    fn decode_bbcs(&self, bank_id: BankId, frame: &Frame) -> TbsPg {
        let _flags = u16::from_le_bytes([frame.0[6], frame.0[7]]);
        let charge_stage = frame.0[8];
        let charge_stage = ChargeStage::from_u8(charge_stage);
        let indicator_flags = frame.0[9];
        let indicator_0_49 = IndicatorState::from_u8(indicator_flags & 0b11);
        let indicator_50_79 = IndicatorState::from_u8((indicator_flags >> 2) & 0b11);
        let indicator_80_99 = IndicatorState::from_u8((indicator_flags >> 4) & 0b11);
        let indicator_100 = IndicatorState::from_u8((indicator_flags >> 6) & 0b11);
        let charge_state = ChargeState {
            stage: charge_stage,
            indicator_0_49,
            indicator_50_79,
            indicator_80_99,
            indicator_100,
        };

        match bank_id {
            BankId::Bank1 => TbsPg::Bb1cs(charge_state),
            BankId::Bank2 => TbsPg::Bb2cs(charge_state),
            BankId::Bank3 => TbsPg::Bb3cs(charge_state),
        }
    }

    fn decode_bbpc(&self, bank_id: BankId, frame: Frame) -> TbsPg {
        let _flags = u16::from_le_bytes([frame.0[6], frame.0[7]]);
        const UNAVAILABLE: u32 = 0x00FFFFFF;
        let power = u32::from_be_bytes([frame.0[8], frame.0[9], frame.0[10], 0]);
        let power = if power != UNAVAILABLE {
            Some(power as f32 * 0.1 - 80000.0)
        } else {
            None
        };
        let charge = u32::from_be_bytes([frame.0[11], frame.0[12], frame.0[13], 0]);
        let charge = if charge != UNAVAILABLE {
            Some(charge as f32 * 0.01 - 80000.0)
        } else {
            None
        };

        let power_and_charge = PowerAndCharge {
            power,
            consumed_amp_hours: charge,
        };
        match bank_id {
            BankId::Bank1 => TbsPg::Bb1pc(power_and_charge),
            BankId::Bank2 => TbsPg::Bb2pc(power_and_charge),
            BankId::Bank3 => TbsPg::Bb3pc(power_and_charge),
        }
    }

    fn decode_version_info(&self, frame: Frame) -> TbsPg {
        fn convert_version(version: u16) -> Version {
            Version {
                major: (version / 100) as u32,
                minor: ((version / 10) % 10) as u8,
                maintenance: (version % 10) as u8,
            }
        }
        let firmware_version = u16::from_le_bytes([frame.0[6], frame.0[7]]);
        let hardware_version = u16::from_le_bytes([frame.0[8], frame.0[9]]);
        let bootloader_version = u16::from_le_bytes([frame.0[8], frame.0[9]]);
        let auxiliary_version = u16::from_le_bytes([frame.0[10], frame.0[11]]);

        TbsPg::VersionInfo(VersionInfo {
            firmware_version: convert_version(firmware_version),
            hardware_version: convert_version(hardware_version),
            bootloader_version: convert_version(bootloader_version),
            auxiliary_version: convert_version(auxiliary_version),
        })
    }

    fn decode_bbdc(&self, bank: BankId, frame: Frame) -> TbsPg {
        let _flags = u16::from_le_bytes([frame.0[6], frame.0[7]]);
        let voltage = u16::from_le_bytes([frame.0[8], frame.0[9]]);
        let voltage = if voltage == 0xFFFF {
            None
        } else {
            Some(voltage as f32 * 0.01)
        };
        let current = if frame.0[10..13] == [0xFF, 0xFF, 0xFF] {
            None
        } else {
            Some(
                u32::from_le_bytes([frame.0[10], frame.0[11], frame.0[12], 0]) as f32 * 0.01
                    - 80000.0,
            )
        };
        let temperature = if frame.0[13] == 0xFE {
            Temperature::NoSensorDetected
        } else if frame.0[13] == 0xFF {
            Temperature::Unavailable
        } else {
            Temperature::DegreesCelsius(frame.0[13] as f32 * 0.5 - 40.0)
        };
        let quantities = BasicQuantities {
            voltage,
            current,
            temperature,
        };
        match bank {
            BankId::Bank1 => TbsPg::Bb1dc(quantities),
            BankId::Bank2 => TbsPg::Bb2dc(quantities),
            BankId::Bank3 => TbsPg::Bb3dc(quantities),
        }
    }

    fn decode_bbbs(&self, bank: BankId, frame: Frame) -> TbsPg {
        let flags = u16::from_le_bytes([frame.0[6], frame.0[7]]);
        let bank_enable = match flags & 0b11 {
            0 => BankEnable::Disabled,
            1 => BankEnable::Enabled,
            _ => BankEnable::ParameterUnavailable,
        };
        let battery_type = u16::from_le_bytes([frame.0[8], frame.0[9]]);
        let battery_type = match battery_type {
            2000 => BatteryType::Flooded,
            3000 => BatteryType::Gel,
            3200 => BatteryType::AGM,
            5000 => BatteryType::LiFePo4,
            _ => BatteryType::ParameterNotAvailable,
        };
        let bank_capacity = u16::from_le_bytes([frame.0[10], frame.0[11]]);
        let bank_capacity = if bank_capacity == 0xFFFF {
            BankCapacity::ParameterNotAvailable
        } else {
            BankCapacity::CapacityAh(bank_capacity)
        };
        let bank_name = BankName::from_u8(frame.0[13]);
        let basic_setup = BasicSetup {
            bank_enable,
            bank_name,
            bank_capacity,
            battery_type,
        };
        match bank {
            BankId::Bank1 => TbsPg::Bb1bs(basic_setup),
            BankId::Bank2 => TbsPg::Bb2bs(basic_setup),
            BankId::Bank3 => TbsPg::Bb3bs(basic_setup),
        }
    }

    fn decode_address_claimed(&self, frame: Frame) -> TbsPg {
        let device_id = u16::from_le_bytes([frame.0[12], frame.0[13]]);
        let device_id = if device_id != 0x0A24 {
            error!("Unknown device ID: {:04X}", device_id);
            DeviceId::Unknown
        } else {
            DeviceId::ExpertModular
        };
        let brand_id = frame.0[11];
        let brand_id = if brand_id != 0x32 {
            error!("Unknown brand ID: {:02X}", brand_id);
            BrandId::Unknown
        } else {
            BrandId::TbsElectronics
        };
        let serial = u32::from_le_bytes([frame.0[6], frame.0[7], frame.0[8], frame.0[9]]);
        TbsPg::AddressClaimed(AddressClaimed {
            device_id,
            brand_id,
            serial_number: serial.wrapping_neg(),
        })
    }

    fn decode_acknowledgement(&self, frame: Frame) -> TbsPg {
        let ack_type = AcknowledgementType::from_u8(frame.0[6]);
        let pgn = u16::from_le_bytes([frame.0[12], frame.0[13]]);
        TbsPg::Acknowledgement(Acknowledgement { ack_type, pgn })
    }

    fn decode_device_name(&self, frame: Frame) -> TbsPg {
        let mut name = [0; 32];
        name.copy_from_slice(&frame.0[6..38]);
        TbsPg::DeviceName(DeviceName { name })
    }

    fn decode_operating_mode(&self, frame: Frame) -> TbsPg {
        let mode = OperatingMode::from_u8(frame.0[6]);
        let lock_flag = u16::from_le_bytes([frame.0[8], frame.0[9]]) >> 14;
        let installer_lock = match lock_flag {
            0 => InstallerLock::InstallerLockOff,
            1 => InstallerLock::InstallerLockOn,
            _ => InstallerLock::ParameterNotAvailable,
        };
        TbsPg::OperatingModeStatus(OperatingModeStatus {
            mode,
            installer_lock,
        })
    }
}
