use rand::Rng;
use rand::prelude::SliceRandom;
use tokio::sync::mpsc;

use crate::types::Bytes;

pub async fn send_bytes(tx: mpsc::Sender<Bytes>) {
    loop {
        let mut bytes;
        {
            let mut rng = rand::thread_rng();
            let size = rng.gen_range(10..=20);
            bytes = vec![0u8; size];
            let possible_values = [0xAA, 0x11, 0x22, 0x33, 0xFF];
            for byte in &mut bytes {
                *byte = *possible_values.choose(&mut rng).unwrap();
            }
        }
        if tx.send(crate::types::Bytes(bytes)).await.is_err() {
            break;
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
}
