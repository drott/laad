### Laad

Rust library for parsing telemetry from a TBS battery monitor, such as the [Battery Monitor Expert Modular](https://tbs-electronics.com/product/expert-modular-battery-monitor-12v-24v-48v/).

### Examples

The `examples/` directory contains an executable `laadreader` that can be started against a packet sender that sends random packets (default), a packet sender that sends packets from a replay of communication with the battery monitor (argument `--replay`), and one (argument `--ble` that connects to a TBS battery monitor using Bluetooth Low Energy (BLE).

#### Try reading from BLE

```
cargo build
cargo run --bin laadreader -- --ble
```

#### Integrate laad into your project

To integrate the library into your own project, an example to use it looks like this.

```Rust
let (bytes_tx, bytes_rx) = mpsc::channel(5);
    let (frames_tx, mut frames_rx) = mpsc::channel(5);

    configure_and_run_source(bytes_tx);

    // Source sends bytes to bytes_tx using bytes_tx.send(Bytes(bytes)).await.

    let mut frame_parser = FrameParser::new();

    tokio::spawn(async move {
        frame_parser.parse_frames(bytes_rx, frames_tx).await;
    });

    while let Some(frame) = frames_rx.recv().await {
        let decoder = decoder::Decoder {};
        let decoded = decoder.decode_frame(frame);
        match decoded {
            TbsPg::Unknown => {
                error!("Received unknown frame");
            }
            _ => {
                info!("Decoded frame: {:?}", decoded);
            }
        }
    }
```

### Name

*Laad* is a play on the Dutch word for charging.

### Acknowledgement

Thanks to Daniel Schouten and [TBS Electronics B.V.](https://tbs-electronics.com/) for providing integration documentation for the protocol of the TBS Battery Monitor and charger devices.

### License

MIT
