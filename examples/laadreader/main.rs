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
use replay_sender::ReplaySender;
use tokio::sync::mpsc::Sender;

use ble_receiver::BleReceiver;
use tokio::sync::mpsc;

mod ble_receiver;
mod random_sender;
mod replay_sender;

use laad::{decoder, frameparser::FrameParser, protocol::TbsPg, types::Bytes};
use random_sender::RandomSender;
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

fn configure_and_run_source(bytes_tx: Sender<Bytes>) {
    let matches = clap::Command::new("laadreader")
        .arg(
            clap::Arg::new("ble")
                .long("ble")
                .help("Use BLE receiver instead of random sender")
                .required(false)
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            clap::Arg::new("replay")
                .help("Use a replay file instead of random sender")
                .long("replay")
                .required(false)
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches_from(std::env::args());

    if matches.get_flag("ble") {
        let mut sender = BleReceiver::new(bytes_tx);
        tokio::spawn(async move {
            sender.start_receiving().await;
        });
    } else if matches.get_flag("replay") {
        let mut sender = ReplaySender::new(bytes_tx);
        tokio::spawn(async move {
            sender.send_bytes().await;
        });
    } else {
        let mut sender = RandomSender::new(bytes_tx);
        tokio::spawn(async move {
            sender.send_bytes().await;
        });
    }
}

#[tokio::main]
async fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    info!("Tracing initialized");

    let (bytes_tx, bytes_rx) = mpsc::channel(5);
    let (frames_tx, mut frames_rx) = mpsc::channel(5);

    configure_and_run_source(bytes_tx);

    // Source sends bytes to bytes_tx using bytes_tx.send(Bytes(bytes)).await.

    let mut frame_parser = FrameParser::new();

    tokio::spawn(async move {
        frame_parser.parse_frames(bytes_rx, frames_tx).await;
    });

    while let Some(frame) = frames_rx.recv().await {
        let decoder = decoder::Decoder {};
        let decoded = decoder.decode_frame(frame);
        match decoded {
            TbsPg::Unknown => {
                error!("Received unknown frame");
            }
            _ => {
                info!("Decoded frame: {:?}", decoded);
            }
        }
    }
}
