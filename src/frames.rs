use crate::types::{Bytes, Frame};
use regex::bytes::Regex;
use tokio::sync::mpsc;
use tracing::*;

const MAX_BUFFERED_BYTES: usize = 1024;

pub struct FrameParser {
    buffered_bytes: Vec<u8>,
}

impl FrameParser {
    pub fn new() -> Self {
        Self {
            buffered_bytes: Vec::new(),
        }
    }

    pub async fn parse_frames(&mut self, mut rx: mpsc::Receiver<Bytes>, tx: mpsc::Sender<Frame>) {
        while let Some(bytes) = rx.recv().await {
            const START_BYTE: u8 = 0xaa;
            const END_BYTE: u8 = 0x99;

            self.buffered_bytes.extend_from_slice(&bytes.0);

            // This is less efficient than it could be, because it restarts the search on previous
            // packets, but it's simpler to understand to use regexes here.
            let re = Regex::new(&format!(
                r"(?-u)\x{:02X}(.*?)\x{:02X}",
                START_BYTE, END_BYTE
            ))
            .unwrap();
            let mut last_match_end = 0;
            for cap in re.captures_iter(&self.buffered_bytes) {
                if let Some(matched) = cap.get(0) {
                    last_match_end = matched.end();
                    if let Err(e) = tx
                        .send(Frame(matched.as_bytes().to_vec().into_boxed_slice()))
                        .await
                    {
                        error!("Failed to send frame: {:?}", e);
                    }
                }
            }
            self.buffered_bytes.drain(..last_match_end);
            debug!(
                "Buffered bytes ({} bytes): {:?}",
                self.buffered_bytes.len(),
                self.buffered_bytes
                    .iter()
                    .map(|b| format!("0x{:02X}", b))
                    .collect::<Vec<_>>()
                    .join(" ")
            );

            if self.buffered_bytes.len() > MAX_BUFFERED_BYTES {
                if let Some(pos) = self.buffered_bytes.iter().position(|&x| x == START_BYTE) {
                    self.buffered_bytes.drain(..pos);
                } else {
                    self.buffered_bytes.clear();
                }
            }
        }
    }
}
