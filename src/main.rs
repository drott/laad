use ble_receiver::BleReceiver;
use tokio::sync::mpsc;

mod ble_receiver;
mod decoder;
mod frames;
mod protocol;
mod random_sender;
mod types;

use frames::FrameParser;
use random_sender::RandomSender;
use tracing::{debug, info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    info!("Tracing initialized");

    let (bytes_tx, bytes_rx) = mpsc::channel(5);
    let (frames_tx, mut frames_rx) = mpsc::channel(5);

    let matches = clap::Command::new("tbslib")
        .arg(
            clap::Arg::new("ble")
                .long("ble")
                .help("Use BLE receiver instead of random sender")
                .required(false)
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches_from(std::env::args());

    if matches.get_flag("ble") {
        let mut sender = BleReceiver::new(bytes_tx);
        tokio::spawn(async move {
            sender.start_receiving().await;
        });
    } else {
        let mut sender = RandomSender::new(bytes_tx);
        tokio::spawn(async move {
            sender.send_bytes().await;
        });
    }

    let mut frame_parser = FrameParser::new();

    tokio::spawn(async move {
        frame_parser.parse_frames(bytes_rx, frames_tx).await;
    });

    while let Some(frame) = frames_rx.recv().await {
        let decoder = decoder::Decoder {};
        let decoded = decoder.decode_frame(frame);
        match decoded {
            protocol::TbsPg::Unknown => {
                debug!("Received unknown frame");
            }
            _ => {
                info!("Decoded frame: {:?}", decoded);
            }
        }
    }
}
