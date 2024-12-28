use crate::{
    protocol::{BankStatus, BasicQuantities, TbsPg, Version, VersionInfo},
    types::Frame,
};

use tracing::error;

pub struct Decoder {}

type PgnTag = [u8; 2];

const PGN_TAG_BB1ST: (PgnTag, usize) = ([0x1A, 0xF0], 16);
const PGN_TAG_VERSION_INFO: (PgnTag, usize) = ([0x02, 0xF0], 16);
const PGN_TAG_HEARTBEAT: (PgnTag, usize) = ([0xFF, 0xFF], 8);
const PGN_TAG_BB1DC: (PgnTag, usize) = ([0x18, 0xF0], 16);

impl Decoder {
    pub fn decode_frame(&self, frame: Frame) -> TbsPg {
        // TODO: Byte stuff, check checksum.

        let frame_len = frame.0.len();
        if frame_len < 8 {
            return TbsPg::Unknown;
        }

        let pgn_tag = [frame.0[3], frame.0[4]];

        if !self.validate_checksum(&frame) {
            error!("Checksum not valid for PGN tag: {:02X?}", pgn_tag);
            return TbsPg::Unknown;
        }

        match (pgn_tag, frame_len) {
            PGN_TAG_BB1ST => self.decode_bb1st(frame),
            PGN_TAG_VERSION_INFO => self.decode_version_info(frame),
            PGN_TAG_HEARTBEAT => TbsPg::Heartbeat,
            PGN_TAG_BB1DC => self.decode_bb1dc(frame),
            _ => TbsPg::Unknown,
        }
    }

    fn validate_checksum(&self, frame: &Frame) -> bool {
        if frame.0.len() < 8 {
            return false;
        }
        let checksum = frame.0[frame.0.len() - 2];
        let sum: u8 = frame.0[1..frame.0.len() - 2]
            .iter()
            .fold(0, |acc, &b| acc.wrapping_add(b));
        sum.wrapping_neg() == checksum
    }

    fn decode_bb1st(&self, frame: Frame) -> TbsPg {
        let _flags = u16::from_le_bytes([frame.0[6], frame.0[7]]);
        let soc: f32 = u16::from_le_bytes([frame.0[8], frame.0[9]]) as f32 * 0.01;
        let soh: f32 = u16::from_le_bytes([frame.0[10], frame.0[11]]) as f32 * 0.01;
        let _time_remaining = u16::from_le_bytes([frame.0[12], frame.0[13]]);
        TbsPg::Bb1st(BankStatus {
            state_of_charge: soc,
            state_of_health: soh,
        })
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

    fn decode_bb1dc(&self, frame: Frame) -> TbsPg {
        let _flags = u16::from_le_bytes([frame.0[6], frame.0[7]]);
        let voltage = u16::from_le_bytes([frame.0[8], frame.0[9]]) as f32 * 0.01;
        let current = if frame.0[10..13] == [0xFF, 0xFF, 0xFF] {
            None
        } else {
            Some(
                u32::from_le_bytes([frame.0[10], frame.0[11], frame.0[12], 0]) as f32 * 0.01
                    - 80000.0,
            )
        };
        let temperature = if frame.0[13] == 0xFF {
            None
        } else {
            Some(frame.0[13] as f32 * 0.5 - 40.0)
        };
        TbsPg::Bb1dc(BasicQuantities {
            voltage,
            current,
            temperature,
        })
    }
}
