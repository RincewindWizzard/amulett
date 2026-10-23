# Stromverbrauch (Datenblattwerte)

Diese Seite sammelt, was die Hersteller-Dokumentation zum Stromverbrauch des
Boards **Waveshare ESP32-S3-Touch-AMOLED-1.75** hergibt. **Alle Werte stammen
aus Datenblättern, nichts ist gemessen.** Eigene Messungen kommen später in eine
getrennte Tabelle (siehe [Offene Punkte](#offene-punkte)).

!!! warning "Chip ist nicht Board"
    Die Chip-Werte gelten für den ESP32-S3 allein. Auf dem Board kommen
    Spannungsregler, PSRAM, Display, Codec, Sensoren und Pull-ups dazu. Der
    Board-Verbrauch im Schlaf liegt daher voraussichtlich deutlich über den
    Mikroampere-Werten des Chips.

## Was die Waveshare-Doku sagt

Die [Produktseite](https://docs.waveshare.com/ESP32-S3-Touch-AMOLED-1.75)
nennt **keine** Stromverbrauchs-, Schlafmodus- oder Akkulaufzeitwerte. Sie
beschreibt nur die Ausstattung:

| Baustein | Funktion | Schnittstelle |
|---|---|---|
| ESP32-S3R8 | SoC mit 8 MB PSRAM (gestapelt) | – |
| CO5300 | AMOLED-Treiber | QSPI |
| CST9217 | Touch-Controller | I2C |
| QMI8658 | 6-Achsen-IMU | I2C |
| PCF85063 | Echtzeituhr (RTC) mit Batterieanschluss | I2C |
| AXP2101 | Energiemanagement, Laden, Akku-Anschluss MX1.25 | I2C |
| TCA9554 | I/O-Expander | I2C |
| ES7210 | Audio-Erfassung / Echounterdrückung (Mikrofone) | I2S/I2C |
| ES8311 | Audio-Codec (Lautsprecher) – laut Ressourcenseite | I2S/I2C |
| TF-Kartenslot, PWR/BOOT-Taste, 8-Pin-Stiftleiste | – | – |

Hinweis: Die Waveshare-Seite nennt **16 MB Flash**, die Produktbeschreibung des
Händlers 32 MB. Für den Stromverbrauch relevant ist das nicht, aber die
Flash-Größe sollte am Gerät geprüft werden (`espflash board-info`).

Weitere Unterlagen: [Ressourcen und Dokumente](https://docs.waveshare.net/ESP32-S3-Touch-AMOLED-1.75/Resources-And-Documents),
[Schaltplan (PDF)](https://www.waveshare.net/w/upload/4/42/ESP32-S3-Touch-AMOLED-1.75.pdf),
[Beispielcode](https://github.com/waveshareteam/ESP32-S3-Touch-AMOLED-1.75).

## ESP32-S3: Schlafmodi

Quelle: [ESP32-S3 Datasheet v2.2](https://documentation.espressif.com/esp32-s3_datasheet_en.pdf),
Abschnitt 5.6.2, Tabelle 5-10. Der Chip ist laut Datenblatt für Varianten
**ohne** PSRAM gemessen; für den S3R8 kommt der PSRAM-Anteil dazu.

| Modus | Bedingung | Typ. Strom |
|---|---|---|
| Light-Sleep | VDD_SPI und WLAN aus, alle GPIOs hochohmig | 240 µA |
| Deep-Sleep + ULP-FSM | ULP-Koprozessor eingeschaltet ¹ | 170 µA |
| Deep-Sleep + ULP-RISC-V | ULP-Koprozessor eingeschaltet ¹ | 190 µA |
| Deep-Sleep, ULP-Sensorüberwachung | z. B. Touch-Sensor mit 1 % Tastverhältnis | 18 µA |
| Deep-Sleep | RTC-Speicher und RTC-Peripherie an | 8 µA |
| Deep-Sleep | nur RTC-Speicher an | 7 µA |
| Power off | `CHIP_PU` auf Low | 1 µA |

¹ Fußnote 2 des Datenblatts: Bei eingeschaltetem ULP-Koprozessor arbeiten Peripherie wie GPIO und I2C auch im Deep-Sleep.

**PSRAM-Zuschlag im Light-Sleep** (Fußnote 1 zu Tabelle 5-10): 140 µA für
8 MB 8-Bit-PSRAM bei 3,3 V, 200 µA bei 1,8 V. Der S3R8 hat 8 MB Octal-PSRAM,
also gilt vermutlich 240 µA + 140 µA ≈ **380 µA** im Light-Sleep (Rechnung aus
Datenblattwerten, nicht gemessen; Versorgungsspannung des PSRAM prüfen).

## ESP32-S3: Aktiver Betrieb

Messbedingungen laut Datenblatt: 3,3 V, 25 °C.

### Modem-Sleep (CPU läuft, Funk aus)

Tabelle 5-9, typische Werte in mA. "Typ1": alle Peripherietakte aus, "Typ2": alle an.
Bei Flash-Zugriffen kann der Wert höher sein.

| CPU-Takt | CPU im Leerlauf (WAITI) | 1 Kern aktiv, 32-Bit-Zugriffe | 2 Kerne aktiv, 32-Bit-Zugriffe | 2 Kerne aktiv, 128-Bit-Zugriffe |
|---|---|---|---|---|
| 40 MHz | 13,2 / 18,8 | 16,2 / 21,8 | 18,7 / 24,4 | 23,0 / 28,8 |
| 80 MHz | 22,0 / 36,1 | 28,4 / 42,6 | 33,1 / 47,3 | 41,8 / 56,3 |
| 160 MHz | 27,6 / 42,3 | 39,9 / 54,6 | 49,6 / 64,1 | 66,7 / 81,1 |
| 240 MHz | 32,9 / 47,6 | 51,2 / 65,9 | 66,2 / 81,3 | 91,7 / 107,9 |

Format: Typ1 / Typ2 in mA.

### Bluetooth LE (Spitzenströme)

Tabelle 5-8. **Spitzenwerte** (TX bei 100 % Tastverhältnis), keine Mittelwerte.

| Modus | Bedingung | Spitze |
|---|---|---|
| TX | +21 dBm | 335 mA |
| TX | +9 dBm | 193 mA |
| TX | 0 dBm | 176 mA |
| TX | +15 dBm (laut Tabelle) | 116 mA |
| RX | – | 93 mA |

Das Datenblatt nennt weder Dauer noch Häufigkeit der Funkereignisse. Der
mittlere BLE-Strom hängt von Advertising-Intervall, Connection-Intervall,
Slave Latency und Nutzdaten ab und ist **nicht** im Datenblatt angegeben.

### WLAN (Spitzenströme)

Tabelle 5-7: TX 802.11b 1 Mbit/s @21 dBm: 340 mA; 802.11g 54 Mbit/s @19 dBm:
291 mA; 802.11n HT20 MCS7 @18,5 dBm: 283 mA; HT40 MCS7 @18 dBm: 286 mA;
RX 802.11b/g/n HT20: 88 mA; HT40: 91 mA.

## Weitere Bausteine auf dem Board

| Baustein | Zustand | Wert | Quelle |
|---|---|---|---|
| AXP2101 | Ausgeschaltet, nur Akku, nur RTC-LDO an (BATFET aus) | 40 µA | [Datenblatt](https://www.waveshare.net/w/upload/e/ed/X-power-AXP2101_SWcharge_V1.0.pdf), Tabelle 5-5 / Abschnitt 6.5.4 |
| AXP2101 | Eingeschaltet, Ruhestrom der Regler | keine Angabe | – |
| ES8311 (Codec) | Normalbetrieb (DVDD 1,8 V, PVDD 1,8 V, AVDD 3,3 V) | 8 mA | [Datenblatt](https://www.waveshare.net/w/upload/6/65/ES8311.DS.pdf), DC Characteristics |
| ES8311 (Codec) | Power Down | 0 µA (laut Tabelle) | wie oben |
| QMI8658 | nur Beschleunigungssensor, Low-Power, 3 Hz | 30 µA | [Datenblatt](https://www.waveshare.net/w/upload/5/5f/QMI8658C.pdf), Tabelle 14 |
| QMI8658 | nur Beschleunigungssensor, Low-Power, 11 / 21 / 128 Hz | 35 / 42 / 55 µA | wie oben |
| QMI8658 | nur Beschleunigungssensor, hohe Auflösung, 31,25 Hz … 1 kHz | 132 … 182 µA | wie oben |
| QMI8658 | nur Gyroskop, 31,25 Hz … 8 kHz | 651 … 908 µA | Tabelle 15 |
| QMI8658 | Beschleunigung + Gyro (6DOF), 31,25 Hz … 8 kHz | 750 … 1004 µA | Tabelle 16 (VDD = 1,8 V) |
| QMI8658 | 6DOF Attitude Engine, 1 … 64 Hz | 783 µA | Tabelle 17 |
| PCF85063 (RTC) | VDD 3,3 V, 25 °C, Schnittstelle inaktiv, CLKOUT aus | 220 nA (max. 450 nA) | [Datenblatt](https://www.waveshare.net/w/upload/9/97/PCF85063A.pdf), Tabelle 41 |
| PCF85063 (RTC) | I2C aktiv, 400 kHz | 18 µA (max. 50 µA) | wie oben |
| CO5300 (AMOLED) | – | **keine Angabe** (Datenblatt nicht in der Waveshare-Ressourcenliste) | – |
| CST9217 (Touch) | – | **keine Angabe** (Datenblatt nicht verlinkt) | – |
| ES7210 (Mikrofon-ADC) | – | **keine Angabe** (Datenblatt nicht verlinkt) | – |
| TCA9554 (I/O-Expander) | – | **keine Angabe** (Datenblatt nicht verlinkt) | – |

Das QMI8658-Datenblatt ist als "Advance Information, Rev 0.6" gekennzeichnet;
die Werte können sich geändert haben. Das AMOLED-Panel selbst dürfte im
eingeschalteten Zustand der größte Einzelverbraucher sein (abhängig von
Helligkeit und Bildinhalt, da AMOLED-Pixel einzeln leuchten); dazu liefert die
Herstellerdoku keine Zahl.

## Was daraus folgt (nur aus Datenblättern abgeleitet)

- **Chip allein**: 7–8 µA im Deep-Sleep; mit PSRAM im Light-Sleep etwa 380 µA.
- **BLE-Mittelwert** lässt sich nur mit Funkereignis-Dauer und -Häufigkeit
  berechnen. Formel für einen periodischen Zyklus:
  `I_mittel = (I_schlaf · t_schlaf + Σ I_phase · t_phase) / T`
  mit der Zykluszeit `T`. Beispielhaft bestimmen die Spitzenwerte (≈ 100–200 mA)
  nur in den kurzen Funkphasen den Verbrauch; im Schlaf zählen Mikroampere.
- **Nicht abschaltbare Verbraucher** auf dem Board (Regler-Ruhestrom, PSRAM,
  Flash, Pull-ups, AMOLED-Versorgung) sind aus den Datenblättern nicht
  quantifizierbar. Der Schaltplan zeigt, welche Schienen der AXP2101
  abschalten kann.

## Offene Punkte

- Schaltplan auswerten: Welche Bausteine hängen an welcher AXP2101-Schiene und sind abschaltbar?
- Datenblätter für CO5300, CST9217, ES7210 und TCA9554 beschaffen oder Herstellerangabe anfragen.
- **Später**: Messungen am Akku-Anschluss (Deep-Sleep, Light-Sleep, BLE-Advertising, verbunden, Display an/aus) und Vergleich mit den hier genannten Datenblattwerten.
