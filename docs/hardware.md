# Hardware

Quelle für alle Angaben dieser Seite ist die
[Hardware-Referenz von Waveshare](https://github.com/waveshareteam/ESP32-S3-Touch-AMOLED-1.75/blob/main/HARDWARE_REFERENCE.md),
die laut Hersteller gegen den
[Schaltplan](https://github.com/waveshareteam/ESP32-S3-Touch-AMOLED-1.75/blob/main/Schematic/ESP32-S3-Touch-AMOLED-1.75-schematic.pdf)
geprüft wurde. Bei einer anderen Hardware-Revision gilt der Schaltplan dieser Revision.

## Varianten

| SKU | Produkt | Unterschied |
|---|---|---|
| 31261 | ESP32-S3-Touch-AMOLED-1.75 | Standard |
| 31262 | ESP32-S3-Touch-AMOLED-1.75-B | Standard mit Schutzgehäuse |
| 31264 | ESP32-S3-Touch-AMOLED-1.75-G | zusätzlich GPS-Modul LC76G |

## Bausteine

| Funktion | Baustein | Schnittstelle |
|---|---|---|
| SoC | ESP32-S3R8, 8 MB PSRAM, **16 MB** externer Flash | – |
| Display | CO5300, AMOLED 466×466 | QSPI |
| Touch | CST9217 | I2C, Interrupt, eigener Reset |
| Energie | AXP2101 (Laden, Akku, Spannungsschienen) | I2C |
| Bewegung | QMI8658 (Beschleunigung + Gyro) | I2C |
| Uhr | PCF85063 (RTC) | I2C |
| Audio-Ausgang | ES8311 + Verstärker NS4150B | I2S + I2C |
| Audio-Eingang | ES7210 (2 Mikrofone + Referenzkanal) | I2S (TDM) + I2C |
| I/O-Erweiterung | TCA9554 | I2C |
| Speicherkarte | TF/microSD | SDMMC |
| Erweiterung | 8-Pin-Stiftleiste 2,54 mm | UART0, 3 GPIO |

!!! note "Flash-Größe"
    Waveshare nennt 16 MB Flash. Manche Händlertexte sprechen von 32 MB.
    Am Gerät prüfen: `espflash board-info`.

## GPIO-Belegung

| GPIO | Signal | Verwendung |
|---:|---|---|
| 0 | BOOT | BOOT-Taste (Low = gedrückt, externer Pull-up) |
| 1 / 2 / 3 | SD_CMD / SD_CLK / SD_D0 | Speicherkarte (1-Bit-SDMMC) |
| 4 / 5 / 6 / 7 | LCD_DATA0…3 | CO5300 QSPI-Daten |
| 8 | I2S_DOUT | Wiedergabe zum ES8311 |
| 9 | I2S_BCLK | Bit-Takt (ES8311 und ES7210) |
| 10 | I2S_DIN | Aufnahme vom ES7210 |
| 11 | TP_INT | Touch-Interrupt |
| 12 | LCD_CS | Display Chip-Select |
| 13 | LCD_TE | Display Tearing-Effect-Signal |
| 14 | I2C_SCL | gemeinsamer I2C-Bus |
| 15 | I2C_SDA | gemeinsamer I2C-Bus |
| 16 / 17 / 18 | Stiftleiste | freie GPIOs (17/18 bei `-G` am GPS) |
| 19 / 20 | USB D− / D+ | natives USB |
| 21 | QMI_INT2 | IMU-Interrupt 2 |
| 38 | LCD_PCLK | Display-Takt |
| 39 | LCD_RST | Display-Reset |
| 40 | TP_RST | Touch-Reset |
| 41 | SD_D3/CS | Speicherkarte (nicht im 1-Bit-Modus genutzt) |
| 42 | I2S_MCLK | Master-Takt (ES8311 und ES7210) |
| 43 / 44 | U0TXD / U0RXD | UART0, Stiftleiste |
| 45 | I2S_WS | Wortauswahl-Takt |
| 46 | PA_CTRL | Lautsprecher-Verstärker |

## I2C-Bus (SDA = GPIO15, SCL = GPIO14, 400 kHz)

| Adresse (7 Bit) | Baustein |
|---:|---|
| `0x18` | ES8311 |
| `0x20` | TCA9554 |
| `0x34` | AXP2101 |
| `0x40` | ES7210 |
| `0x51` | PCF85063 |
| `0x5A` | CST9217 |
| `0x6B` | QMI8658 |

Manche Audio-Bibliotheken nennen 8-Bit-Adressen (`0x30` für ES8311, `0x80`
für ES7210). Diese dürfen nicht an eine API für 7-Bit-Adressen übergeben werden.

## TCA9554: Signale

| Pin | Signal | Funktion |
|---|---|---|
| P3 | RTC_INT | Interrupt der Uhr (Eingang) |
| P4 | SYS_OUT | PWR-Taste, High = gedrückt (Eingang) |
| P5 | AXP_IRQ | Interrupt des AXP2101 (Eingang) |
| P6 | QMI_INT1 | IMU-Interrupt 1 (Eingang) |
| P7 | GPS_RST | Reset des GPS (nur `-G`) |
| P0–P2 | EXIO0–2 | frei, an Testpunkten |

## Tasten

- **BOOT**: GPIO0, Low = gedrückt.
- **PWR**: hängt am Energiepfad des AXP2101, nicht an einem freien GPIO. Der
  entprellte Zustand ist über TCA9554 P4 lesbar (High = gedrückt).

## Hinweise

- Alle GPIOs arbeiten mit 3,3 V und sind **nicht** 5-V-tolerant.
- Das AMOLED hat keine PWM-Hintergrundbeleuchtung; die Helligkeit wird über
  CO5300-Befehle gesetzt.
- Display-Reset (GPIO39) und Touch-Reset (GPIO40) sind getrennte Leitungen.
- GPIO19/20 nicht zweckentfremden, sonst funktioniert USB-Flashen nicht mehr.
- Den Lautsprecher-Verstärker (GPIO46) nicht unabhängig vom Codec schalten.
