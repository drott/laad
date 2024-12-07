use crate::types::{Bytes, Frame};
use tokio::sync::mpsc;

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
            const END_BYTE: u8 = 0xff;

            // TODO: Loop this to account for multiple frames in a single buffer.
            let (mut start_byte_at, mut end_byte_at) = (None, None);
            for (i, &byte) in bytes.0.iter().enumerate() {
                if byte == START_BYTE {
                    start_byte_at = Some(i);
                }
                if byte == END_BYTE  {
                    end_byte_at = Some(i);
                }
            }

            match (self.opened_frame.is_empty(), start_byte_at, end_byte_at) {
                (true, Some(start), Some(end)) => {
                    if let Err(e) = tx.send(Frame(bytes.0[start..=end].to_vec())).await {
                        println!("Failed to send frame: {:?}", e);
                    }
                }
                _ => {
                    todo!("Handle the case where the frame is split between two buffers");
                }
            }
        }
    }
}
