# Entwicklung

## Toolchain (Ubuntu)

```sh
sudo apt install build-essential curl git pkg-config libudev-dev libssl-dev
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
. ~/.cargo/env
cd ~ && cargo install espup espflash --locked      # nicht im Projektordner
espup install -v 1.95.0.0 -t esp32s3
. ~/export-esp.sh                                  # in jedem neuen Terminal
sudo usermod -aG dialout $USER                     # danach neu einloggen
```

## Bauen, flashen, Konsole

```sh
cargo run --release                 # Produkt-Firmware: baut, flasht, öffnet die Konsole
cargo run --release --features poc  # Hardware-PoC-Seiten statt Produkt
```

Die Produkt-Firmware zeigt die Farbfläche, siehe [Farbanzeige](farbanzeige.md).
Mit dem Cargo-Feature `poc` startet stattdessen das Test-Programm mit den
Seiten Testbild, Energie, IMU, Audio, Funk und Info.

## Aufbau des Quellcodes

| Pfad | Inhalt |
|---|---|
| `crates/amulett-core` | Reine Logik ohne Hardware: Freundestabelle, Farbregler, Beacon-Format, HCI-Auswertung, Parameter. Mit Unit-Tests. |
| `src/product.rs` | Produkt-Hauptprogramm |
| `src/poc.rs` | Hardware-PoC (Feature `poc`) |
| `src/display.rs`, `touch.rs`, `pmic.rs`, … | Treiber je Baustein |
| `src/wireless.rs` | BLE (Beacon + Scanner) und WLAN-Scan |

## Tests auf dem Entwicklungsrechner

Die Logik in `crates/amulett-core` lässt sich ohne Board testen. Weil das
Projekt standardmäßig für den ESP32-S3 baut, muss das Ziel angegeben werden:

```sh
cd crates/amulett-core
cargo test --target x86_64-unknown-linux-gnu
```

!!! note "Tests ohne `alloc` und `std`"
    Die Projektkonfiguration baut `core` und `alloc` neu (`build-std`).
    Tests dürfen deshalb weder `extern crate std` noch `extern crate alloc`
    verwenden, sonst meldet der Compiler doppelte Sprachelemente. Testdaten
    liegen in festen Feldern.

## Dokumentation lokal bauen

```sh
pip install -r docs/requirements.txt
mkdocs serve          # http://127.0.0.1:8000
mkdocs build --strict # wie in der CI
```
