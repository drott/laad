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

//! # Test server that creates a BLE peripheral with a single service and characteristic
//! emulating a TBS Battery Monitor, and sending a basic packets.
use ble_peripheral_rust::{
    gatt::properties::{AttributePermission, CharacteristicProperty},
    uuid::ShortUuid,
};
use ble_peripheral_rust::{
    gatt::{
        characteristic::Characteristic,
        peripheral_event::{
            PeripheralEvent, ReadRequestResponse, RequestResponse, WriteRequestResponse,
        },
        service::Service,
    },
    Peripheral,
};
use tokio::sync::mpsc::channel;
use tracing::{debug, error, info, Level};
use tracing_subscriber::FmtSubscriber;
use uuid::Uuid;

const SERVICE_UUID: &str = "65333333-A115-11E2-9E9A-0800200CA100";
const TX_RX_CHARACTERISTIC_UUID: &str = "65333333-A115-11E2-9E9A-0800200CA102";
const DEMO_PACKET: &[u8] = &[
    0xAA, 0x00, 0xFF, 0x1A, 0xF0, 0x08, 0xC0, 0xB0, 0x10, 0x27, 0x10, 0x27, 0xFD, 0xFF, 0x15, 0x99,
];

async fn handle_events(mut receiver_rx: tokio::sync::mpsc::Receiver<PeripheralEvent>) {
    while let Some(event) = receiver_rx.recv().await {
        match event {
            PeripheralEvent::CharacteristicSubscriptionUpdate { .. } => {}
            PeripheralEvent::ReadRequest { responder, .. } => {
                if let Err(e) = responder.send(ReadRequestResponse {
                    value: DEMO_PACKET.to_vec(),
                    response: RequestResponse::Success,
                }) {
                    debug!("Failed to send ReadRequestResponse: {:?}", e);
                } else {
                    debug!("Demo packet sent.");
                }
            }
            PeripheralEvent::WriteRequest {
                value, responder, ..
            } => {
                debug!("Received WriteRequest with value: {:X?}", value);
                if let Err(e) = responder.send(WriteRequestResponse {
                    response: RequestResponse::Success,
                }) {
                    debug!("Failed to send WriteRequestResponse: {:?}", e);
                }
            }
            _ => {}
        }
    }
}

fn make_service() -> Service {
    Service {
        uuid: Uuid::from_string(SERVICE_UUID),
        primary: true,
        characteristics: vec![Characteristic {
            uuid: <Uuid as ShortUuid>::from_string(TX_RX_CHARACTERISTIC_UUID),
            properties: vec![
                CharacteristicProperty::Read,
                CharacteristicProperty::Write,
                CharacteristicProperty::Notify,
            ],
            permissions: vec![
                AttributePermission::Readable,
                AttributePermission::Writeable,
            ],
            ..Default::default()
        }],
    }
}

async fn send_update(peripheral: &mut Peripheral) {
    peripheral
        .update_characteristic(
            Uuid::from_string(TX_RX_CHARACTERISTIC_UUID),
            DEMO_PACKET.to_vec(),
        )
        .await
        .expect("Failed to update characteristic");
}

#[tokio::main]
async fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    info!("Tracing initialized");

    let (sender_tx, receiver_rx) = channel::<PeripheralEvent>(256);

    let mut peripheral = Peripheral::new(sender_tx).await.unwrap();
    tokio::spawn(async move {
        handle_events(receiver_rx).await;
    });

    while !peripheral.is_powered().await.unwrap() {}

    let service = make_service();

    if let Err(err) = peripheral.add_service(&service).await {
        error!("Error adding service: {}", err);
        return;
    }
    info!("Service Added");

    if let Err(err) = peripheral
        .start_advertising("TBS Battery Monitor", &[service.uuid])
        .await
    {
        error!("Error starting advertising: {}", err);
        return;
    }
    info!("Advertising Started");

    loop {
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        send_update(&mut peripheral).await;
        debug!("Sent update");
    }
}
