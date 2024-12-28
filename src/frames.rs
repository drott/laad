use crate::types::{Bytes, Frame};
use tokio::sync::mpsc;
use tracing::*;

pub struct FrameParser {
    opened_frame: Vec<u8>,
}

impl FrameParser {
    pub fn new() -> Self {
        Self {
            opened_frame: Vec::new(),
        }
    }

    pub async fn parse_frames(&mut self, mut rx: mpsc::Receiver<Bytes>, tx: mpsc::Sender<Frame>) {
        while let Some(bytes) = rx.recv().await {
            const START_BYTE: u8 = 0xaa;
            const END_BYTE: u8 = 0x99;

            // TODO: Loop this to account for multiple frames in a single buffer.
            let (mut start_byte_at, mut end_byte_at) = (None, None);
            for (i, &byte) in bytes.0.iter().enumerate() {
                if byte == START_BYTE {
                    start_byte_at = Some(i);
                }
                if byte == END_BYTE {
                    end_byte_at = Some(i);
                }
            }

            match (self.opened_frame.is_empty(), start_byte_at, end_byte_at) {
                (true, Some(start), Some(end)) if start < end => {
                    if let Err(e) = tx
                        .send(Frame(bytes.0[start..=end].to_vec().into_boxed_slice()))
                        .await
                    {
                        error!("Failed to send frame: {:?}", e);
                    }
                }
                (true, Some(start), Some(end)) if start > end => {
                    error!("Invalid frame: start_byte_at > end_byte_at");
                    self.opened_frame.clear();
                }
                (true, Some(start), None) => {
                    self.opened_frame.extend_from_slice(&bytes.0[start..]);
                }
                (false, None, None) => {
                    self.opened_frame.extend_from_slice(&bytes.0);
                }
                (false, None, Some(end)) => {
                    self.opened_frame.extend_from_slice(&bytes.0[..=end]);
                    if let Err(e) = tx
                        .send(Frame(self.opened_frame.clone().into_boxed_slice()))
                        .await
                    {
                        error!("Failed to send frame: {:?}", e);
                    }
                    self.opened_frame.clear();
                }
                (true, None, None) => {
                    // No start or end byte in this buffer, skip packet.
                }
                _ => {
                    error!(
                        "Invalid frame state. opened_frame.is_empty: {}, start_byte_at: {:?}, end_byte_at: {:?}",
                        self.opened_frame.is_empty(),
                        start_byte_at,
                        end_byte_at
                    );
                    self.opened_frame.clear();
                }
            }
        }
    }
}
