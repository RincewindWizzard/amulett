# Funk: WLAN und Bluetooth LE

Der ESP32-S3 bringt 2,4-GHz-WLAN (802.11 b/g/n) und Bluetooth 5 (LE) mit.
Das PoC nutzt [`esp-radio`](https://docs.espressif.com/projects/rust/esp-radio/latest/)
mit dem Scheduler [`esp-rtos`](https://docs.espressif.com/projects/rust/esp-rtos/latest/).
Code: `src/wireless.rs`.

## Voraussetzungen

- Der Scheduler muss **vor** dem Funk gestartet werden (`esp_rtos::start`).
- Der Funk braucht Heap im **internen RAM** (64 + 36 KB reserviert); der
  Framebuffer liegt im PSRAM.
- Die Funk-Binärdateien brauchen `opt-level = 3`; `Cargo.toml` setzt das für
  das Paket `esp-radio`.

## WLAN-Scan

```rust
--8<-- "src/wireless.rs:wifi_scan"
```

Auf der Seite **Funk** startet ein Tipp einen Scan. SSID, Signalstärke und
Kanal erscheinen auf dem Display und in der Konsole.

## BLE-Advertising

Das PoC sendet einen verbindbaren Namen (`amulett`) alle 1000 ms. Statt eines
Host-Stacks schickt es direkt HCI-Kommandos an den Controller:

```rust
--8<-- "src/wireless.rs:ble_start"
```

Das Gerät ist danach in jeder BLE-Scanner-App sichtbar. Das Advertising-Intervall
(`BLE_INTERVAL_MS` in `src/main.rs`) bestimmt maßgeblich den Stromverbrauch,
siehe [Stromverbrauch](stromverbrauch.md).

## Stolpersteine

- Das PoC startet BLE nur beim Betreten der Funk-Seite und stoppt es beim
  Verlassen. WLAN-Scan und BLE laufen nicht gleichzeitig (Coexistence ist nicht
  aktiviert).
- Ohne `opt-level 2/3` für `esp-radio` kann das Advertising ausbleiben.
- Das PoC verbindet sich mit keinem WLAN und bietet keine GATT-Dienste an.
  Dafür wäre ein Host-Stack wie [`trouble`](https://github.com/embassy-rs/trouble) nötig.

## Herstellerunterlagen

- [ESP32-S3-Datenblatt](https://documentation.espressif.com/esp32-s3_datasheet_en.pdf)
- [Waveshare-Produktseite](https://docs.waveshare.com/ESP32-S3-Touch-AMOLED-1.75) (Antenne: Chipantenne oder IPEX, per Widerstand umschaltbar)
- [`esp-radio`-Dokumentation](https://docs.espressif.com/projects/rust/esp-radio/latest/)
