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
use rand::prelude::SliceRandom;
use rand::Rng;
use tokio::sync::mpsc;

use laad::types::Bytes;

const EXAMPLE_PACKETS: &[&[u8]] = &[
    &[0xAA, 0x00, 0xFF, 0xFF, 0xFF, 0x00, 0x03, 0x99], // [HRTBT]
    &[
        0xAA, 0x00, 0xFF, 0x18, 0xF0, 0x08, 0x00, 0xB0, 0x32, 0x05, 0xFD, 0x11, 0x7A, 0xFE, 0x84,
        0x99,
    ], // [BB1DC]
    &[
        0xAA, 0xFD, 0x00, 0x00, 0xEA, 0x03, 0x00, 0xEE, 0x00, 0x28, 0x99,
    ], // [Request for address claimed.]
    &[
        0xAA, 0x00, 0xFF, 0x00, 0xEE, 0x08, 0xD2, 0x66, 0x2F, 0xF4, 0xFF, 0x32, 0x24, 0x0A, 0x51,
        0x99,
    ], // [Address claimed.]
    &[
        0xAA, 0xFD, 0x00, 0x03, 0xF0, 0x08, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x10,
        0x99,
    ], // [Send all command.]
    &[
        0xAA, 0x00, 0xFF, 0x1A, 0xF0, 0x08, 0xC0, 0xB0, 0x10, 0x27, 0x10, 0x27, 0xFD, 0xFF, 0x15,
        0x99,
    ], // [BB1ST]
    &[
        0xAA, 0x00, 0xFF, 0x02, 0xF0, 0x08, 0x67, 0x00, 0x64, 0x00, 0x64, 0x00, 0xFF, 0xFF, 0xDA,
        0x99,
    ], // [Version Info]
];

const BROKEN_PACKETS: &[&[u8]] = &[
    &[0x67, 0x00, 0x64, 0x00, 0x64, 0x00, 0xFF, 0xFF, 0xDA, 0x99],
    &[0xAA, 0x00, 0xFF, 0x01, 0xEE, 0x08, 0xD2, 0x66, 0x2F],
    &[0xAA],
    &[0x99],
    &[0xFF, 0xFF, 0xFF, 0xFF],
];

const WORKING_BUFFER_SIZE: usize = 20;

pub struct RandomSender {
    tx: mpsc::Sender<Bytes>,
    buffer: std::collections::VecDeque<u8>,
}

impl RandomSender {
    pub fn new(tx: mpsc::Sender<Bytes>) -> Self {
        Self {
            tx,
            buffer: std::collections::VecDeque::new(),
        }
    }

    pub fn fill_buffer_if_nedeed(&mut self) {
        while self.buffer.len() < WORKING_BUFFER_SIZE {
            let packet = {
                let mut rng = rand::thread_rng();
                if rng.gen_bool(0.5) {
                    *EXAMPLE_PACKETS.choose(&mut rng).unwrap()
                } else {
                    *BROKEN_PACKETS.choose(&mut rng).unwrap()
                }
            };
            self.buffer.extend(packet.iter().copied());
        }
    }

    pub async fn send_bytes(&mut self) {
        loop {
            self.fill_buffer_if_nedeed();
            let chunk_size =
                rand::thread_rng().gen_range(WORKING_BUFFER_SIZE * 40 / 100..=WORKING_BUFFER_SIZE);
            let bytes: Vec<u8> = self.buffer.drain(..chunk_size).collect();
            self.tx
                .send(Bytes(bytes))
                .await
                .expect("Failed to send bytes");
            // tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }
}
