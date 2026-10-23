# Display: CO5300 AMOLED

1,75 Zoll rund, 466×466 Pixel, 16,7 Mio. Farben, Anschluss per **QSPI**
(SPI2: Takt GPIO38, CS GPIO12, Daten GPIO4–7, Reset GPIO39). Treiber:
`src/display.rs`.

## Protokoll

Jeder Zugriff beginnt mit einem Befehlsbyte `0x02` (Register schreiben) oder
`0x32` (Pixel schreiben), gefolgt von einer 24-Bit-Adresse. Befehle und Adresse
gehen über eine Leitung, Pixeldaten über vier.

| Zugriff | Befehl | Adresse | Daten |
|---|---|---|---|
| Register schreiben | `0x02` | `00 <reg> 00` | 0–n Bytes, 1 Leitung |
| Pixel schreiben | `0x32` | `00 3C 00` (Write Memory Continue) | RGB565 big-endian, 4 Leitungen |

Vor den Pixeln wird mit `CASET` (0x2A), `PASET` (0x2B) und `RAMWR` (0x2C) das
Fenster gesetzt. Die sichtbare Fläche beginnt bei **Spalte 6**.

## Initialisierung

```rust
--8<-- "src/display.rs:init"
```

`0xC4 = 0x80` schaltet den QSPI-Pixelmodus des Controllers ein.

## Pixel senden

```rust
--8<-- "src/display.rs:blit"
```

Ein vollständiges Bild (466×466×2 = 434 KB) passt nicht in den internen RAM.
Das PoC hält einen **Framebuffer im PSRAM** (`Framebuffer`, implementiert
`embedded_graphics::DrawTarget`) und schickt ihn in 4-KB-Blöcken per DMA.

## Helligkeit

Das AMOLED hat keine PWM-Hintergrundbeleuchtung. Die Helligkeit steht im
Register `0x51` (0–255). Die Seite **Testbild** wechselt per Tipp zwischen drei
Stufen. Für den Schlaf bietet der Treiber `sleep()`/`wake()`.

## Stolpersteine

- PSRAM-Zugriffe sind langsam; der Framebuffer wird deshalb zeilenweise
  gezeichnet und am Stück gesendet.
- Display-Reset (GPIO39) und Touch-Reset (GPIO40) sind getrennte Leitungen.
- Das Panel ist rund: Pixel außerhalb des Kreises werden nicht angezeigt.

## Herstellerunterlagen

- [Waveshare-Produktseite](https://docs.waveshare.com/ESP32-S3-Touch-AMOLED-1.75)
- [Arduino-Treiber `Arduino_CO5300`](https://github.com/waveshareteam/ESP32-S3-Touch-AMOLED-1.75/blob/main/examples/arduino/libraries/GFX_Library_for_Arduino/src/display/Arduino_CO5300.cpp)
  (Initialisierungsfolge, Registerliste)
- [QSPI-Bus `Arduino_ESP32QSPI`](https://github.com/waveshareteam/ESP32-S3-Touch-AMOLED-1.75/blob/main/examples/arduino/libraries/GFX_Library_for_Arduino/src/databus/Arduino_ESP32QSPI.cpp)
- [Beispiel `01_HelloWorld`](https://github.com/waveshareteam/ESP32-S3-Touch-AMOLED-1.75/tree/main/examples/arduino/01_HelloWorld)
