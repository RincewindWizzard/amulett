# Eingabe: Touch und Tasten

## Touch: CST9217

I2C-Adresse `0x5A`, Interrupt GPIO11, Reset GPIO40. Treiber: `src/touch.rs`.

Ablauf pro Abfrage (aus der Herstellerbibliothek `TouchDrvCST92xx`):

1. Kommando `D0 00` schreiben und 15 Bytes lesen,
2. mit `D0 00 AB` quittieren,
3. Byte 6 muss `0xAB` sein, Byte 5 enthält die Anzahl der Finger (max. 2).

```rust
--8<-- "src/touch.rs:read"
```

Koordinaten liegen im Bereich 0–465. Das PoC fragt den Controller nur ab, wenn
die Interrupt-Leitung Low ist oder ein Finger aufliegt, und erkennt daraus
**Tippen** und **Wischen** (Wischen links/rechts blättert die Seiten).

## Tasten

| Taste | Anschluss | PoC |
|---|---|---|
| BOOT | GPIO0, Low = gedrückt | nächste Seite |
| PWR | AXP2101, entprellt über TCA9554 P4 lesbar, High = gedrückt | vorherige Seite |

```rust
--8<-- "src/expander.rs:pwr"
```

## Stolpersteine

- Das PoC fragt den Interrupt-Pegel ab statt echte GPIO-Interrupts zu nutzen;
  sehr kurze Berührungen zwischen zwei Bildern können verpasst werden.
- Der Controller braucht nach dem Reset einige Millisekunden, bevor er auf
  I2C antwortet.
- Ein langer Druck auf PWR schaltet das Gerät aus.

## Herstellerunterlagen

- [Beispiel `10_Touch_CST9217`](https://github.com/waveshareteam/ESP32-S3-Touch-AMOLED-1.75/tree/main/examples/arduino/10_Touch_CST9217)
- [Treiber `TouchDrvCST92xx` (SensorLib)](https://github.com/waveshareteam/ESP32-S3-Touch-AMOLED-1.75/blob/main/examples/arduino/libraries/SensorLib/src/touch/TouchDrvCST92xx.cpp)
- [Hardware-Referenz (Tasten, Pins)](https://github.com/waveshareteam/ESP32-S3-Touch-AMOLED-1.75/blob/main/HARDWARE_REFERENCE.md)
