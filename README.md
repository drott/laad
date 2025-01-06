### Laad

Rust library for parsing telemetry from a TBS battery monitor, such as the [Battery Monitor Expert Modular](https://tbs-electronics.com/product/expert-modular-battery-monitor-12v-24v-48v/).

So far it's been tested to decode most PGNs of a dump from the TBS Battery Monitor Expert Modular, except for those that dump internal registers and state.

### Examples

The `examples/` directory contains the `laadreader` command line tool that can be started against

* a packet sender that sends random packets (default),
* a packet sender that sends packets from a replay of communication with the battery monitor (argument `--replay`),
* and one (argument `--ble` that connects to a TBS battery monitor using Bluetooth Low Energy (BLE).

The `laadreader` tool connects via BLE to a device with "TBS" in its name and connects to the serial port characteristic on the TBS device, then waits for a moment for unsolicited packets, then sends a request all packet to retrieve more information.

#### Try reading from BLE

```bash
cargo build
cargo run --bin laadreader -- --ble
```

Typical output may look like this:

```shell
2025-01-06T18:21:18.070745Z  INFO laadreader: Decoded frame: Bb1dc(BasicQuantities { voltage: Some(11.309999), current: Some(-0.0625), temperature: NoSensorDetected })
2025-01-06T18:21:18.070755Z  INFO laadreader: Decoded frame: Bb1dc(BasicQuantities { voltage: Some(11.3), current: Some(-0.0546875), temperature: NoSensorDetected })
2025-01-06T18:21:18.070763Z  INFO laadreader: Decoded frame: Bb1dc(BasicQuantities { voltage: Some(11.309999), current: Some(-0.0546875), temperature: NoSensorDetected })
2025-01-06T18:21:18.070781Z  INFO laadreader: Decoded frame: Bb1dc(BasicQuantities { voltage: Some(11.3), current: Some(-0.0625), temperature: NoSensorDetected })
2025-01-06T18:21:18.071205Z ERROR laad::decoder: Checksum not valid for PGN tag: [18, F0], 0x4F vs 0xE6?
2025-01-06T18:21:18.071212Z ERROR laadreader: Received unknown frame
2025-01-06T18:21:18.071604Z  INFO laadreader: Decoded frame: Bb1dc(BasicQuantities { voltage: Some(11.309999), current: Some(-0.0546875), temperature: NoSensorDetected })
2025-01-06T18:21:18.071844Z  INFO laadreader: Decoded frame: AddressClaimed(AddressClaimed { device_id: ExpertModular, brand_id: TbsElectronics, serial_number: 227190006 })
2025-01-06T18:21:18.072241Z  INFO laadreader: Decoded frame: Acknowledgement(Acknowledgement { ack_type: PositiveAcknowledgement, pgn: 0x01F0 })
2025-01-06T18:21:18.072482Z  INFO laadreader: Decoded frame: DeviceName(DeviceName { name: "Akkumonitori" })
2025-01-06T18:21:18.072705Z  INFO laadreader: Decoded frame: VersionInfo(VersionInfo { firmware_version: Version { major: 1, minor: 0, maintenance: 5 }, hardware_version: Version { major: 1, minor: 0, maintenance: 0 }, bootloader_version: Version { major: 1, minor: 0, maintenance: 0 }, auxiliary_version: Version { major: 1, minor: 0, maintenance: 0 } })
2025-01-06T18:21:18.072722Z  INFO laadreader: Decoded frame: OperatingModeStatus(OperatingModeStatus { mode: DeviceOff, installer_lock: InstallerLockOff })
2025-01-06T18:21:18.072970Z  INFO laadreader: Decoded frame: Bb1dc(BasicQuantities { voltage: Some(11.3), current: Some(-0.0546875), temperature: NoSensorDetected })
2025-01-06T18:21:18.072987Z  INFO laadreader: Decoded frame: Bb1pc(PowerAndCharge { power: Some(419464930.0), consumed_amp_hours: Some(5964697.5) })
2025-01-06T18:21:18.073211Z  INFO laadreader: Decoded frame: Bb1st(BankStatus { state_of_charge: ChargePercentage(73.0), state_of_health: HealthPercentage(100.0), time_remaining: Minutes(32767) })
2025-01-06T18:21:18.073229Z  INFO laadreader: Decoded frame: Bb1bs(BasicSetup { bank_enable: Enabled, bank_name: MainBatteryBank, bank_capacity: CapacityAh(200), battery_type: AGM })
2025-01-06T18:21:18.073450Z  INFO laadreader: Decoded frame: Bb2dc(BasicQuantities { voltage: Some(11.37), current: None, temperature: Unavailable })
2025-01-06T18:21:18.073468Z  INFO laadreader: Decoded frame: Bb2st(BankStatus { state_of_charge: ChargePercentage(100.0), state_of_health: Unavailable, time_remaining: Unavailable })
2025-01-06T18:21:18.073691Z  INFO laadreader: Decoded frame: Bb2bs(BasicSetup { bank_enable: Enabled, bank_name: StarterBattery, bank_capacity: ParameterNotAvailable, battery_type: ParameterNotAvailable })
2025-01-06T18:21:18.073709Z  INFO laadreader: Decoded frame: Bb3dc(BasicQuantities { voltage: None, current: None, temperature: Unavailable })
2025-01-06T18:21:18.073944Z  INFO laadreader: Decoded frame: Bb3st(BankStatus { state_of_charge: Unavailable, state_of_health: Unavailable, time_remaining: Unavailable })
2025-01-06T18:21:18.073963Z  INFO laadreader: Decoded frame: Bb3bs(BasicSetup { bank_enable: Disabled, bank_name: ParameterNotAvailable, bank_capacity: ParameterNotAvailable, battery_type: ParameterNotAvailable })
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

*Laad* as the library name is a play on the Dutch word for charging.

### Acknowledgement

Thanks to Daniel Schouten and [TBS Electronics B.V.](https://tbs-electronics.com/) for providing integration documentation for the protocol of the TBS Battery Monitor and charger devices.

### License

MIT
