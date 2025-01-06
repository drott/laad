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
