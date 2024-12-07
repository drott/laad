use rand::Rng;
use rand::prelude::SliceRandom;
use tokio::sync::mpsc;

use crate::types::Bytes;

pub struct RandomSender {
    tx: mpsc::Sender<Bytes>,
}

impl RandomSender {

    pub fn new(tx: mpsc::Sender<Bytes>) -> Self {
        Self { tx }
    }

    pub async fn send_bytes(&self) {
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
                let idx1 = rng.gen_range(0..bytes.len());
                let idx2 = loop {
                    let idx = rng.gen_range(0..bytes.len());
                    if idx != idx1 {
                        break idx;
                    }
                };
                bytes[idx1.min(idx2)] = 0xaa;
                bytes[idx1.max(idx2)] = 0xff;
            }
            if self.tx.send(crate::types::Bytes(bytes)).await.is_err() {
                break;
            }
            // tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }
}
