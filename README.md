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
cargo run --release   # builds, flashes and opens the serial monitor
```

Releases (Tag `v*`) enthalten ein fertiges Flash-Image `amulett-esp32s3.bin`:
`espflash write-bin 0x0 amulett-esp32s3.bin`.

The current firmware only boots and scans the shared I2C bus, which is a quick
check that the PMIC, touch controller and IMU are reachable. Next steps:
AXP2101 power rails, CO5300 init over QSPI, CST9217 touch.

Note: GPIO numbers in `src/main.rs` come from the vendor documentation and
should be verified against your board revision.
