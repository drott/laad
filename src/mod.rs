//!
//! # Laad
//!
//! This library decodes frames from the TBS battery monitor and TBS charger products.
//!
//! Example usage (see also **examples/laadreader/main.rs**):
//!
//! ```rust
//! use tokio::sync::mpsc;
//! use laad::{decoder::Decoder,frameparser::FrameParser,protocol::TbsPg,types::Bytes};
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
//!   # // Avoid stalling in doctests.
//!   # tokio::spawn(async move {
//!   #   std::process::exit(0);
//!   # });
//!   while let Some(frame) = frames_rx.recv().await {
//!       let decoder = Decoder::new();
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
//! 
//! }
//! ```

/// Decoder decodes frames into protocol types.
pub mod decoder;
/// FrameParser identifies frames in a stream of bytes and sends the frames to a Tokio channel.
pub mod frameparser;
/// Protocol defines the TBS protocol and decoded information for frame types that are understood.
pub mod protocol;
/// Basic types for bytes and frames.
pub mod types;
