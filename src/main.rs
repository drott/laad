use tokio::sync::mpsc;

mod random_sender;
mod types;
mod frames;

use random_sender::RandomSender;
use frames::FrameParser;

#[tokio::main]
async fn main() {
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
        println!("Received frame: {:?}", frame.0.iter().map(|b| format!("0x{:02x} ", b)).collect::<String>());
    }
}
