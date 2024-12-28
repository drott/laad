use tokio::sync::mpsc;

mod decoder;
mod frames;
mod protocol;
mod random_sender;
mod types;

use frames::FrameParser;
use random_sender::RandomSender;
use tracing::{info, Level};
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

    let mut sender = RandomSender::new(bytes_tx);

    tokio::spawn(async move {
        sender.send_bytes().await;
    });

    let mut frame_parser = FrameParser::new();

    tokio::spawn(async move {
        frame_parser.parse_frames(bytes_rx, frames_tx).await;
    });

    while let Some(frame) = frames_rx.recv().await {
        let decoder = decoder::Decoder {};
        let decoded = decoder.decode_frame(frame);
        match decoded {
            protocol::TbsPg::Unknown => {
                println!("Received unknown frame");
            }
            _ => {
                info!("Decoded frame: {:?}", decoded);
            }
        }
    }
}
