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

use std::io::BufRead;

use tokio::sync::mpsc;

use laad::types::Bytes;

pub struct ReplaySender {
    tx: mpsc::Sender<Bytes>,
    replay_file: Option<std::iter::Cycle<std::vec::IntoIter<std::string::String>>>,
}

impl ReplaySender {
    pub fn new(tx: mpsc::Sender<Bytes>) -> Self {
        Self {
            tx,
            replay_file: None,
        }
    }

    pub fn open_file_if_needed(&mut self) {
        if self.replay_file.is_none() {
            let file = std::fs::File::open("dump/dumped_btatt_values.log")
                .expect("Failed to open replay file");
            let lines = std::io::BufReader::new(file)
                .lines()
                .collect::<Result<Vec<_>, _>>()
                .expect("Failed to read lines");
            self.replay_file = Some(lines.into_iter().cycle());
        }
    }

    pub async fn send_bytes(&mut self) {
        self.open_file_if_needed();
        loop {
            let line = self.replay_file.as_mut().unwrap().next().unwrap();
            let bytes = (0..line.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&line[i..i + 2], 16).expect("Failed to decode hex"))
                .collect::<Vec<u8>>();
            self.tx
                .send(Bytes(bytes))
                .await
                .expect("Failed to send bytes");
            // tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }
}
