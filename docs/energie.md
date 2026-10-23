# Energie: AXP2101

Der AXP2101 (I2C `0x34`) übernimmt Laden, Akku-Überwachung und die
Spannungsschienen des Boards. Treiber: `src/pmic.rs`.

## Was das PoC macht

- liest die Chip-ID (Register `0x03`, erwartet `0x4A`),
- aktiviert die ADC-Kanäle für Akku-, USB- und Systemspannung sowie die Chip-Temperatur,
- zeigt auf der Seite **Energie** USB-Status, Spannungen, Ladezustand in Prozent, Ladestatus und Chip-Temperatur.

## Spannungsschienen

Die Waveshare-Beispiele schalten **keine** Schiene ein: Display, Codec und
Sensoren laufen mit den Standardwerten des Chips nach dem Einschalten. Der
Treiber liest nur die Freigaberegister (`0x80`, `0x90`, `0x91`) und gibt sie
beim Start über die serielle Konsole aus. Eine Schiene abzuschalten ist daher
erst für Energiesparmodi relevant (siehe [Stromverbrauch](stromverbrauch.md)).

## Code

### ADC aktivieren

```rust
--8<-- "src/pmic.rs:adc"
```

### Status lesen

```rust
--8<-- "src/pmic.rs:status"
```

## Stolpersteine

- Die Akkuspannung ist ein 13-Bit-Wert (Register `0x34/0x35`), VBUS und VSYS
  sind 14 Bit; die oberen Bits der Highbytes müssen maskiert werden.
- Ohne Akku liefert der Prozentwert keinen Sinn; der Treiber gibt dann `None`
  zurück.
- Die PWR-Taste hängt am AXP2101. Ein langer Druck schaltet das Gerät aus.

## Herstellerunterlagen

- [AXP2101-Datenblatt](https://www.waveshare.net/w/upload/e/ed/X-power-AXP2101_SWcharge_V1.0.pdf)
- [Waveshare-Beispiel `05_LVGL_AXP2101_ADC_Data`](https://github.com/waveshareteam/ESP32-S3-Touch-AMOLED-1.75/tree/main/examples/arduino/05_LVGL_AXP2101_ADC_Data)
- [Registerdefinitionen in XPowersLib](https://github.com/waveshareteam/ESP32-S3-Touch-AMOLED-1.75/blob/main/examples/arduino/libraries/XPowersLib/src/REG/AXP2101Constants.h)
- [Waveshare-Produktseite](https://docs.waveshare.com/ESP32-S3-Touch-AMOLED-1.75)
