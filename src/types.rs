use std::fmt::{Display, Write};

/// Incoming bytes received for example over serial, the BLE receiver, or a replay, see examples.
#[derive(Debug)]
pub struct Bytes(pub Vec<u8>);

/// A struct that represents a frame of bytes, with a start byte (0xAA) and an end byte (0x99), de-bytestffed.
#[derive(Debug)]
pub struct Frame(pub Box<[u8]>);

impl Display for Frame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = self
            .0
            .iter()
            .fold(String::new(), |mut acc, &b| {
                write!(acc, "{:02X} ", b).unwrap();
                acc
            })
            .trim_end()
            .to_string();
        write!(f, "{}", s)
    }
}
