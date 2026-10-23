# Weitere Bausteine

## Echtzeituhr PCF85063 (`0x51`)

Zeitregister ab `0x04` (Sekunden, Minuten, Stunden, Tag, Wochentag, Monat,
Jahr), BCD-kodiert. Bit 7 der Sekunden meldet, dass der Oszillator
gestoppt war (Zeit unzuverlässig). Die Seite **Info** zeigt die Uhrzeit.
Treiber: `src/rtc.rs`. Der Interrupt liegt an TCA9554 P3.

```rust
--8<-- "src/rtc.rs:read"
```

- [PCF85063-Datenblatt](https://www.waveshare.net/w/upload/9/97/PCF85063A.pdf)
- [Beispiel `03_LVGL_PCF85063_simpleTime`](https://github.com/waveshareteam/ESP32-S3-Touch-AMOLED-1.75/tree/main/examples/arduino/03_LVGL_PCF85063_simpleTime)

## I/O-Expander TCA9554 (`0x20`)

Vom Board genutzt für Interrupt-Leitungen und die PWR-Taste (P3–P6) sowie das
GPS-Reset (P7, nur Variante `-G`). P0–P2 sind frei. Treiber: `src/expander.rs`
(Register `0x00` Eingang, `0x01` Ausgang, `0x03` Richtung).

- [TCA9554-Datenblatt (Texas Instruments)](https://www.ti.com/lit/ds/symlink/tca9554.pdf)
- [Hardware-Referenz](https://github.com/waveshareteam/ESP32-S3-Touch-AMOLED-1.75/blob/main/HARDWARE_REFERENCE.md)

## I2C-Scan

Beim Start fragt das PoC alle Adressen `0x08–0x77` ab und ordnet sie den
bekannten Bausteinen zu (Seite **Info**). Auf einem Standardboard erscheinen
sieben Geräte (siehe [Hardware](hardware.md)).

## Speicherkarte (TF)

Pins: CMD GPIO1, CLK GPIO2, D0 GPIO3 (1-Bit-SDMMC), D3/CS GPIO41. Das PoC
nutzt die Karte **nicht**. Eine Ansteuerung wäre mit dem SDMMC-Treiber von
esp-hal möglich. Siehe [Beispiel `07_LVGL_SD_Test`](https://github.com/waveshareteam/ESP32-S3-Touch-AMOLED-1.75/tree/main/examples/arduino/07_LVGL_SD_Test).

## GPS (nur Variante `-G`)

LC76G über I2C (`0x50`/`0x54`) oder UART (GPIO17/18), Reset über TCA9554 P7.
Nicht Teil des PoC. Siehe [Beispiel `09_LC76G_I2C`](https://github.com/waveshareteam/ESP32-S3-Touch-AMOLED-1.75/tree/main/examples/arduino/09_LC76G_I2C).
