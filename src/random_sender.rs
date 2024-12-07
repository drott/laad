use rand::Rng;
use rand::prelude::SliceRandom;
use tokio::sync::mpsc;

use crate::types::Bytes;

enum Command {
    SamePacket(usize, usize),
    Open(usize),
    Close(usize),
    None
}

pub struct RandomSender {
    tx: mpsc::Sender<Bytes>,
    command: Command,
}

fn next_command(previous_command : &Command, packet_length: usize) -> Command {
    let mut rng = rand::thread_rng();
    let idx1 = rng.gen_range(0..packet_length);
    let idx2 = loop {
        let idx = rng.gen_range(0..packet_length);
        if idx != idx1 {
            break idx;
        }
    };
    let (smaller, larger) = if idx1 < idx2 { (idx1, idx2) } else { (idx2, idx1) };

    match previous_command {
        Command::Open(_) => {
            Command::Close(larger)
        },
        _ => {
            match rng.gen_range(0..3) {
                0 => Command::SamePacket(smaller, larger),
                1 => Command::Open(smaller),
                _ => Command::None
            }

        }
    }
    
}

impl RandomSender {

    pub fn new(tx: mpsc::Sender<Bytes>) -> Self {
        Self { tx, command: Command::None }
    }

    pub async fn send_bytes(&mut self) {
        loop {
            let mut bytes;
            {
                let mut rng = rand::thread_rng();
                let size = rng.gen_range(10..=20);
                bytes = vec![0u8; size];
                let possible_values = [0x11, 0x22, 0x33];
                for byte in &mut bytes {
                    *byte = *possible_values.choose(&mut rng).unwrap();
                }
                self.command = next_command(&self.command, size);
                match self.command {
                    Command::SamePacket(idx1, idx2) => {
                        bytes[idx1.min(idx2)] = 0xaa;
                        bytes[idx1.max(idx2)] = 0xff;
                    },
                    Command::Open(idx) => {
                        bytes[idx] = 0xaa;
                    },
                    Command::Close(idx) => {
                        bytes[idx] = 0xff;
                    },
                    Command::None => {}
                }
            }
            if self.tx.send(crate::types::Bytes(bytes)).await.is_err() {
                break;
            }
            // tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }
}
