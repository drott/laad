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

    fn de_bytestuff(&self, bytes: &[u8]) -> Vec<u8> {
        if !bytes.contains(&0xA9) {
            return bytes.to_vec();
        }
        let mut de_bytestuffed = Vec::new();
        let mut iter = bytes.iter();
        while let Some(byte) = iter.next() {
            if *byte == 0xA9 {
                if let Some(next_byte) = iter.next() {
                    de_bytestuffed.push(next_byte ^ 0x20);
                }
            } else {
                de_bytestuffed.push(*byte);
            }
        }
        de_bytestuffed
    }

    pub async fn parse_frames(&mut self, mut rx: mpsc::Receiver<Bytes>, tx: mpsc::Sender<Frame>) {
        while let Some(bytes) = rx.recv().await {
            const START_BYTE: u8 = 0xaa;
            const END_BYTE: u8 = 0x99;

            self.buffered_bytes.extend_from_slice(&bytes.0);

            // This is less efficient than it could be, because it restarts the search on previous
            // packets, but it's simpler to understand to use regexes here.
            let re = Regex::new(&format!(
                r"(?s-u)\x{:02X}(.*?)\x{:02X}",
                START_BYTE, END_BYTE
            ))
            .unwrap();
            let mut last_match_end = 0;
            for cap in re.captures_iter(self.buffered_bytes.as_slice()) {
                if let Some(matched) = cap.get(0) {
                    last_match_end = matched.end();
                    let matched_debytestuffed = self.de_bytestuff(&matched.as_bytes());
                    if let Err(e) = tx
                        .send(Frame(matched_debytestuffed.into_boxed_slice()))
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
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_de_bytestuff_no_stuffing() {
        let parser = FrameParser::new();
        let input = vec![0x01, 0x02, 0x03];
        let output = parser.de_bytestuff(&input);
        assert_eq!(output, input);
    }

    #[test]
    fn test_de_bytestuff_with_stuffing() {
        let parser = FrameParser::new();
        let input = vec![0x01, 0xA9, 0x20, 0x03];
        let expected_output = vec![0x01, 0x00, 0x03];
        let output = parser.de_bytestuff(&input);
        assert_eq!(output, expected_output);
    }

    #[test]
    fn test_de_bytestuff_multiple_stuffing() {
        let parser = FrameParser::new();
        let input = vec![0xA9, 0x20, 0xA9, 0x21, 0xA9, 0x22];
        let expected_output = vec![0x00, 0x01, 0x02];
        let output = parser.de_bytestuff(&input);
        assert_eq!(output, expected_output);
    }

    #[test]
    fn test_de_bytestuff_mixed_start_end_same_value() {
        let parser = FrameParser::new();
        let input = vec![0xAA, 0xA9, 0x8a, 0xBB, 0xCC, 0x99];
        let expected_output = vec![0xAA, 0xAA, 0xBB, 0xCC, 0x99];
        let output = parser.de_bytestuff(&input);
        assert_eq!(output, expected_output);
    }

    #[test]
    fn test_de_bytestuff_a9_escape() {
        let parser = FrameParser::new();
        let input = vec![0xAA, 0xA9, 0x89, 0xBB, 0xCC, 0x99];
        let expected_output = vec![0xAA, 0xA9, 0xBB, 0xCC, 0x99];
        let output = parser.de_bytestuff(&input);
        assert_eq!(output, expected_output);
    }

    #[test]
    fn test_de_bytestuff_99_escape() {
        let parser = FrameParser::new();
        let input = vec![0xAA, 0xA9, 0xB9, 0xBB, 0xCC, 0x99];
        let expected_output = vec![0xAA, 0x99, 0xBB, 0xCC, 0x99];
        let output = parser.de_bytestuff(&input);
        assert_eq!(output, expected_output);
    }

    #[test]
    fn test_de_bytestuff_incomplete_stuffing() {
        let parser = FrameParser::new();
        let input = vec![0x01, 0xA9];
        let expected_output = vec![0x01];
        let output = parser.de_bytestuff(&input);
        assert_eq!(output, expected_output);
    }
}
