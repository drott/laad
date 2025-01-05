//!
//! # TBS Library
//!
//! This library decodes frames from the TBS battery monitor and TBS charger products.
//!
//! Example usage (see also **examples/tbsreader/main.rs**):
//!
//! ```rust
//! use tokio::sync::mpsc;
//! use tbslib::{decoder,frames::FrameParser,protocol::TbsPg,types::Bytes};
//! #[tokio::main]
//! async fn main() {
//!   let (bytes_tx, bytes_rx) = mpsc::channel(5);
//!   let (frames_tx, mut frames_rx) = mpsc::channel(5);
//!
//!   // Configure source and send send bytes to bytes_tx using bytes_tx.send(Bytes(bytes)).await.
//!
//!   let mut frame_parser = FrameParser::new();
//!
//!   tokio::spawn(async move {
//!       frame_parser.parse_frames(bytes_rx, frames_tx).await;
//!   });
//!
//!   while let Some(frame) = frames_rx.recv().await {
//!       let decoder = decoder::Decoder {};
//!       let decoded = decoder.decode_frame(frame);
//!       match decoded {
//!           TbsPg::Unknown => {
//!               println!("Received unknown frame");
//!           }
//!           _ => {
//!               println!("Decoded frame: {:?}", decoded);
//!           }
//!       }
//!   }
//! }
//! ```
pub mod decoder;
pub mod frameparser;
pub mod protocol;
pub mod types;
