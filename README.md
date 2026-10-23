# amulett

Rust (`no_std`, [esp-hal](https://github.com/esp-rs/esp-hal)) firmware for the
Waveshare **ESP32-S3-Touch-AMOLED-1.75** (ESP32-S3R8, 8 MB PSRAM, 32 MB flash).

Onboard peripherals: CO5300 AMOLED (QSPI, 466×466), CST9217 touch (I2C),
QMI8658 IMU, AXP2101 PMIC, audio codec + dual mic, PWR/BOOT buttons.

## Setup

```sh
cargo install espup espflash
espup install        # Xtensa Rust toolchain ("esp" channel)
. ~/export-esp.sh
```

## Run

```sh
cargo run --release   # builds, flashes and opens the serial monitor
```

The current firmware only boots and scans the shared I2C bus, which is a quick
check that the PMIC, touch controller and IMU are reachable. Next steps:
AXP2101 power rails, CO5300 init over QSPI, CST9217 touch.

Note: GPIO numbers in `src/main.rs` come from the vendor documentation and
should be verified against your board revision. The scaffold has not been
compiled yet (no Xtensa toolchain in the authoring environment).
