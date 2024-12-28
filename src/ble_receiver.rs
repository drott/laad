use crate::types::Bytes;
use btleplug::api::{Central, CharPropFlags, Manager as _, Peripheral, ScanFilter};
use btleplug::platform::Manager;
use futures::StreamExt;
use std::error::Error;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time;
use tracing::{debug, error, info};
use uuid::Uuid;

pub struct BleReceiver {
    tx: mpsc::Sender<Bytes>,
}

const PERIPHERAL_NAME_MATCH_FILTER: &str = "TBS";
const TX_RX_CHARACTERISTIC_UUID: Uuid = Uuid::from_u128(0x65333333_A115_11E2_9E9A_0800200CA102);
const TIMEOUT_SECS: u64 = 10;

impl BleReceiver {
    pub fn new(tx: mpsc::Sender<Bytes>) -> Self {
        Self { tx }
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
