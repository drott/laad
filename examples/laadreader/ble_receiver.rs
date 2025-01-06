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
use btleplug::api::{Central, CharPropFlags, Manager as _, Peripheral, ScanFilter, WriteType};
use btleplug::platform::Manager;
use futures::StreamExt;
use laad::types::Bytes;
use std::error::Error;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time;
use tracing::{debug, error, info};
use uuid::Uuid;

// From Android hci snoop log, the TBS characteristic data updates can be extracted
// using tshark with the following command:
// tshark -r btlog_pre_filter.log -Y "(bthci_acl.src.bd_addr[4:2] == 31:d8) && (btatt.opcode == 0x1d)" -T fields -e btatt.value

pub struct BleReceiver {
    tx: mpsc::Sender<Bytes>,
}

const PERIPHERAL_NAME_MATCH_FILTER: &str = "TBS";
const TX_RX_CHARACTERISTIC_UUID: Uuid = Uuid::from_u128(0x65333333_A115_11E2_9E9A_0800200CA102);
const TIMEOUT_SECS: u64 = 20;
const REQUEST_FOR_ADDRESS_CLAIMED: [u8; 11] = [
    0xAA, 0xFD, 0x00, 0x00, 0xEA, 0x03, 0x00, 0xEE, 0x00, 0x28, 0x99,
];
const SEND_ALL_COMMAND: [u8; 16] = [
    0xAA, 0xFD, 0x00, 0x03, 0xF0, 0x08, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x10, 0x99,
];

impl BleReceiver {
    pub fn new(tx: mpsc::Sender<Bytes>) -> Self {
        Self { tx }
    }

    async fn send_requests(peripheral: &impl Peripheral) {
        let characteristic = peripheral
            .characteristics()
            .into_iter()
            .find(|c| c.uuid == TX_RX_CHARACTERISTIC_UUID)
            .unwrap();
        if let Err(err) = peripheral
            .write(
                &characteristic,
                &REQUEST_FOR_ADDRESS_CLAIMED,
                WriteType::WithoutResponse,
            )
            .await
        {
            error!("Error sending REQUEST_FOR_ADDRESS_CLAIMED: {:?}", err);
        } else {
            debug!("Sent REQUEST_FOR_ADDRESS_CLAIMED");
        }
        time::sleep(Duration::from_secs(3)).await;
        loop {
            if let Err(err) = peripheral
                .write(
                    &characteristic,
                    &SEND_ALL_COMMAND,
                    WriteType::WithoutResponse,
                )
                .await
            {
                error!("Error sending SEND_ALL_COMMAND: {:?}", err);
            } else {
                debug!("Sent SEND_ALL_COMMAND");
            }
            time::sleep(Duration::from_secs(10)).await;
        }
    }

    async fn connect_notify() -> Result<btleplug::platform::Peripheral, Box<dyn Error>> {
        let manager = Manager::new().await?;
        let adapter_list = manager.adapters().await?;
        if adapter_list.is_empty() {
            error!("No Bluetooth adapters found");
        }

        let adapter = adapter_list.into_iter().next().unwrap();
        adapter
            .start_scan(ScanFilter::default())
            .await
            .expect("Can't scan BLE adapter for connected devices...");
        info!("Scanning for peripherals.");
        time::sleep(Duration::from_secs(TIMEOUT_SECS)).await;
        let peripherals = adapter.peripherals().await?;

        if peripherals.is_empty() {
            error!("BLE peripheral devices were not found, sorry. Exiting...");
        } else {
            for peripheral in peripherals.iter() {
                let properties = peripheral.properties().await?;
                let is_connected = peripheral.is_connected().await?;
                let local_name = properties
                    .unwrap()
                    .local_name
                    .unwrap_or(String::from("(peripheral name unknown)"));
                debug!(
                    "Peripheral {:?} is connected: {:?}",
                    &local_name, is_connected
                );
                // Check if it's the peripheral we want.
                if local_name.contains(PERIPHERAL_NAME_MATCH_FILTER) {
                    println!("Found matching peripheral {:?}...", &local_name);
                    if !is_connected {
                        // Connect if we aren't already connected.
                        if let Err(err) = peripheral.connect().await {
                            error!("Error connecting to peripheral, skipping: {}", err);
                            continue;
                        }
                    }
                    let is_connected = peripheral.is_connected().await?;
                    info!(
                        "Now connected ({:?}) to peripheral {:?}.",
                        is_connected, &local_name
                    );
                    if is_connected {
                        debug!("Discover peripheral {:?} services...", local_name);
                        peripheral.discover_services().await?;
                        for characteristic in peripheral.characteristics() {
                            info!("Checking characteristic {:?}", characteristic);
                            // Subscribe to notifications from the characteristic with the selected
                            // UUID.
                            if characteristic.uuid == TX_RX_CHARACTERISTIC_UUID
                                && characteristic.properties.contains(CharPropFlags::NOTIFY)
                            {
                                info!("Subscribing to characteristic {:?}", characteristic.uuid);
                                peripheral.subscribe(&characteristic).await?;
                                return Ok(peripheral.clone());
                            }
                        }
                        info!("Disconnecting from peripheral {:?}...", local_name);
                        peripheral.disconnect().await?;
                    }
                } else {
                    debug!("Skipping unknown peripheral {:?}", peripheral);
                }
            }
        }
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No matching peripheral found",
        )))
    }

    pub async fn start_receiving(&mut self) {
        let peripheral = match Self::connect_notify().await {
            Ok(peripheral) => peripheral,
            Err(err) => {
                error!("Error connecting to peripheral: {:?}", err);
                return;
            }
        };

        let mut notification_stream = match peripheral.notifications().await {
            Ok(stream) => stream,
            Err(err) => {
                error!("Error getting notifications stream: {:?}", err);
                return;
            }
        };

        let peripheral_clone = peripheral.clone();
        tokio::spawn(async move {
            Self::send_requests(&peripheral_clone).await;
        });

        while let Some(notification) = notification_stream.next().await {
            let hex_string: String =
                notification
                    .value
                    .iter()
                    .fold(String::new(), |mut acc, byte| {
                        acc.push_str(&format!("{:02X}", byte));
                        acc
                    });
            debug!(
                "Received notification: 0x{}, sending to parser.",
                hex_string
            );
            let bytes = Bytes(notification.value);
            self.tx.send(bytes).await.unwrap();
        }
    }
}
