# amulett

Rust (`no_std`, [esp-hal](https://github.com/esp-rs/esp-hal)) firmware for the
Waveshare **ESP32-S3-Touch-AMOLED-1.75** (ESP32-S3R8, 8 MB PSRAM, 16 MB flash).

📖 **Dokumentation:** <https://rincewindwizzard.github.io/amulett/>

Onboard peripherals: CO5300 AMOLED (QSPI, 466×466), CST9217 touch (I2C),
QMI8658 IMU, AXP2101 PMIC, audio codec + dual mic, PWR/BOOT buttons.

## Setup (Ubuntu)

```sh
sudo apt install build-essential curl git pkg-config libudev-dev libssl-dev
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh   # falls Rust fehlt
. ~/.cargo/env
cargo install espup espflash --locked      # nicht im Projektordner ausführen
espup install -v 1.95.0.0 -t esp32s3       # Xtensa-Toolchain ("esp")
. ~/export-esp.sh                          # in jedem neuen Terminal
sudo usermod -aG dialout $USER             # Serial-Zugriff, danach neu einloggen
```

## Run

```sh
cargo run --release                 # product firmware: builds, flashes, opens the serial monitor
cargo run --release --features poc  # hardware proof-of-concept pages instead
```

Releases (Tag `v*`) enthalten ein fertiges Flash-Image `amulett-esp32s3.bin`:
`espflash write-bin 0x0 amulett-esp32s3.bin`.

After switching on, choose a faction with the buttons (BOOT switches between A and B,
a short press on PWR confirms). Amulets of the same faction are friends, the other
faction is the enemy and triggers a red danger display.

The product firmware shows the presence of friends as a colour: every amulet
sends a BLE beacon and scans for the others; saturation shows how many friends
are around, brightness how close the nearest one is. The `poc` feature builds
the hardware test pages (display, touch, power, IMU, audio, radio) instead.
Details are in the documentation linked above.

Run the unit tests of the hardware independent logic on the development
machine with `cd crates/amulett-core && cargo test --target x86_64-unknown-linux-gnu`.
