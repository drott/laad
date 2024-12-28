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
use uuid::Uuid;
use tracing::{info,error,Level};
use tracing_subscriber::FmtSubscriber;


const SERVICE_UUID: u16 = 0x1234_u16;
const TX_RX_CHARACTERISTIC_UUID: &str = "65333333-A115-11E2-9E9A-0800200CA102";

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

fn make_service() -> Service {
    Service {
        uuid: Uuid::from_short(SERVICE_UUID),
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

#[tokio::main]
async fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
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
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}
