use tokio::sync::mpsc;

mod random_sender;
use random_sender::send_bytes;
mod types;

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(100);

    // Spawn a task to send byte sequences
    tokio::spawn(send_bytes(tx));
    // Convert the receiver to a stream
    // Read directly from the receiver
    while let Some(bytes) = rx.recv().await {
        println!("Received: {:?}", bytes.0.iter().map(|b| format!("0x{:02x} ", b)).collect::<String>());
    }
}
