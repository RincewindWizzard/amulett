# Audio: ES8311 und ES7210

Zwei Codecs teilen sich eine I2S-Schnittstelle:

| Signal | GPIO | Richtung |
|---|---:|---|
| MCLK | 42 | Ausgang (256 × Abtastrate) |
| BCLK | 9 | Ausgang |
| WS | 45 | Ausgang |
| DOUT | 8 | zum ES8311 (Wiedergabe) |
| DIN | 10 | vom ES7210 (Mikrofone) |
| PA_CTRL | 46 | Lautsprecher-Verstärker NS4150B |

Beide Chips laufen als I2S-**Slave**: Der ESP32-S3 liefert MCLK, BCLK und WS
und wird über I2C konfiguriert. Der Code liegt in `src/audio.rs`.

## Wiedergabe: ES8311

I2C `0x18`. Die Initialisierung stammt aus dem Waveshare-Beispiel
`08_ES8311` und ist für 16 kHz, 16 Bit und MCLK = 4,096 MHz fest eingestellt.

```rust
--8<-- "src/audio.rs:es8311_init"
```

Der Verstärker (GPIO46) wird nur während der Wiedergabe aktiviert. Das PoC
spielt einen 400-Hz-Sinus in Dauerschleife (DMA-Loop-Puffer); Tippen auf der
Seite **Audio** schaltet ihn ein und aus.

## Aufnahme: ES7210

I2C `0x40`, vier Eingänge. Auf dem Board sind MIC1 und MIC2 die beiden
Mikrofone; MIC3 führt das Wiedergabesignal (Referenz für Echounterdrückung),
MIC4 ist nicht verbunden. Das PoC nutzt MIC1/MIC2 als normales Stereosignal
(L/R), 30 dB Verstärkung. Die Herstellersoftware betreibt dieselben Chips im
4-Slot-TDM-Betrieb, um zusätzlich die Referenz aufzunehmen.

Die Registerfolge ist aus Espressifs `esp_codec_dev` übernommen:

```rust
--8<-- "src/audio.rs:es7210_init"
```

Die Seite **Audio** zeigt den Spitzenpegel beider Kanäle als Balken.

```rust
--8<-- "src/audio.rs:levels"
```

## Stolpersteine

- MCLK muss laufen, **bevor** die Codecs per I2C konfiguriert werden.
- Die 8-Bit-Adressen aus Espressif-Bibliotheken (`0x30`, `0x80`) sind
  7-Bit-Adressen `0x18` und `0x40`.
- TX und RX teilen BCLK/WS (`signal_loopback`), deshalb gleiche Abtastrate und
  Datenbreite für beide Richtungen.
- Echounterdrückung und Rauschfilter sind Software (z. B. Espressif AFE) und
  nicht Teil des PoC.

## Herstellerunterlagen

- [ES8311-Datenblatt](https://www.waveshare.net/w/upload/6/65/ES8311.DS.pdf) und
  [User Guide](https://www.waveshare.net/w/upload/5/56/ES8311.user.Guide.pdf)
- [Beispiel `08_ES8311`](https://github.com/waveshareteam/ESP32-S3-Touch-AMOLED-1.75/tree/main/examples/arduino/08_ES8311)
- [ES7210-Treiber in `esp_codec_dev`](https://github.com/espressif/esp-adf/tree/master/components/esp_codec_dev/device/es7210)
- [Hardware-Referenz (I2S, TDM-Belegung)](https://github.com/waveshareteam/ESP32-S3-Touch-AMOLED-1.75/blob/main/HARDWARE_REFERENCE.md)
