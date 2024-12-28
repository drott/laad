use ble_peripheral_rust::uuid::ShortUuid;
use uuid::Uuid;
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


async fn handle_events(mut receiver_rx: tokio::sync::mpsc::Receiver<PeripheralEvent>) {
    while let Some(event) = receiver_rx.recv().await {
        match event {
            PeripheralEvent::CharacteristicSubscriptionUpdate {
                request,
                subscribed,
            } => {
                // Send notifications to subscribed clients
            }
            PeripheralEvent::ReadRequest {
                request,
                offset,
                responder,
            } => {
                // Respond to Read request
                responder.send(ReadRequestResponse {
                    value: String::from("Hello").into(),
                    response: RequestResponse::Success,
                });
            }
            PeripheralEvent::WriteRequest {
                request,
                offset,
                value,
                responder,
            } => {
                // Respond to Write request
                responder.send(WriteRequestResponse {
                    response: RequestResponse::Success,
                });
            }
            _ => {}
        }
    }
}

const SERVICE_UUID: u16 = 0x1800_u16;

#[tokio::main]
async fn main() {
    let (sender_tx, receiver_rx) = channel::<PeripheralEvent>(256);
    let mut peripheral = Peripheral::new(sender_tx).await.unwrap();
    while !peripheral.is_powered().await.unwrap() {}

    peripheral
        .add_service(&Service {
            uuid: Uuid::from_short(SERVICE_UUID),
            primary: true,
            characteristics: vec![Characteristic {
                uuid: <Uuid as ShortUuid>::from_string("65333333-A115-11E2-9E9A-0800200CA102"),
                ..Default::default()
            }],
        })
        .await;

    peripheral
        .start_advertising("TBS Battery Monitor", &[Uuid::from_short(SERVICE_UUID)])
        .await;

    tokio::spawn(async move {
        handle_events(receiver_rx).await;
    });
}
