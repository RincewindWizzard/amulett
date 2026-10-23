# Schlafmodi und Messszenarien

Diese Seite beschreibt, wie das Board in Stromsparzustände versetzt wird, und
das Messprogramm, mit dem sich der Verbrauch je Zustand ermitteln lässt. Die
Zahlen aus den Datenblättern stehen auf der Seite
[Stromverbrauch](stromverbrauch.md); **eigene Messwerte gibt es noch nicht**.

## Schlafmodi des ESP32-S3

| Modus | CPU | RAM | Aufwachen | Datenblatt |
|---|---|---|---|---|
| Aktiv / Modem-Sleep | läuft | erhalten | – | 13–108 mA (je Takt) |
| Light-Sleep | angehalten | erhalten, Programm läuft danach weiter | Timer, GPIO, … | 240 µA |
| Deep-Sleep | aus | nur RTC-Speicher | Timer, RTC-GPIO, Touch, ULP | 7–8 µA |

Beim Deep-Sleep startet das Programm nach dem Aufwachen **neu**; Zustand, der
erhalten bleiben soll, liegt im RTC-Speicher:

```rust
--8<-- "src/power_study.rs:rtc"
```

`persistent` initialisiert den Speicher nur beim ersten Einschalten mit 0 und
lässt ihn danach über Deep-Sleep und Software-Reset hinweg unverändert.

## Aufwachquellen

Im Messprogramm wecken zwei Quellen:

| Quelle | Pin / Zeit | Modus |
|---|---|---|
| Timer | 10 s (Deep/Light), 60 s im Zyklus-Szenario | Light und Deep |
| BOOT-Taste | GPIO0, Low-Pegel (externer Pull-up) | Light und Deep |

Weitere Pins sind möglich, weil sie alle RTC-Pads sind (GPIO0–21): Touch-Interrupt
GPIO11 und IMU-Interrupt GPIO21. Ein Pin weckt nur, wenn er **auf einen Pegel
hört** (`listen`) und der Low-Power-Pfad aktiv ist:

```rust
--8<-- "src/power_study.rs:wake"
```

!!! warning "Pegel halten"
    Liegt ein Pin beim Einschlafen schon auf dem Weck-Pegel, wacht der Chip
    sofort wieder auf. Deshalb weckt der Code auf Low-Pegel mit externem
    Pull-up (BOOT) und nicht auf einen offenen Eingang.

Nach dem Aufwachen meldet das Messprogramm die Ursache (`wakeup_cause()`).

## Light-Sleep

```rust
--8<-- "src/power_study.rs:light"
```

## Wiederaufnahme nach Deep-Sleep

Beim Start prüft die Firmware, ob eine Deep-Sleep-Szene läuft (gültige Marke
im RTC-Speicher und Aufwach-Ursache), zählt den Zyklus hoch und schläft wieder:

```rust
--8<-- "src/power_study.rs:resume"
```

## Messprogramm

```sh
cargo run --release --features power
```

Nach dem Start erscheint ein Menü: **BOOT** wechselt das Szenario, ein kurzer
Druck auf **PWR** startet es. Jedes Szenario gibt auf der Konsole eine Zeile
`== SCENARIO … ==` aus, damit Messungen dem Zustand zugeordnet werden können.

| Szenario | Zustand |
|---|---|
| Display an, hell | Display weiß mit Helligkeit 0xFF, CPU läuft |
| Display gedimmt | Display weiß mit Helligkeit 0x20, CPU läuft |
| Display aus, CPU läuft | Display im Sleep-Modus, CPU 240 MHz aktiv (Referenz ohne Display) |
| BLE Advertising 100 / 1000 / 5000 ms | verbindbares Advertising, Display aus, CPU läuft |
| Beacon + Scan | Produktbetrieb (Beacon senden, scannen), Display aus |
| Light Sleep | Display aus, 10 s Light-Sleep, Aufwachen per Timer oder BOOT |
| Deep Sleep | Display aus, 10 s Deep-Sleep, Aufwachen per Timer oder BOOT |
| Deep Sleep + BLE | alle 60 s aufwachen, 1 s Advertising, wieder schlafen |
| AXP2101-Schienen | schaltet die Ausgänge des AXP2101 nacheinander ab (8 s) und wieder an |

Der CPU-Takt steht in `src/main.rs` (`CpuClock::max()`); für Messungen mit 80
oder 160 MHz dort ändern.

### Schienen des AXP2101

Der Schaltplan ([PDF](https://github.com/waveshareteam/ESP32-S3-Touch-AMOLED-1.75/blob/main/Schematic/ESP32-S3-Touch-AMOLED-1.75-schematic.pdf))
zeigt für den AXP2101: `DCDC1` = `VCC3V3` (Hauptversorgung des ESP32-S3 und
der meisten Bausteine, **nicht abschaltbar**), `DCDC2` 0,9 V, `DCDC3` 1,2 V,
`DCDC4` 1,8 V, `DCDC5` nicht beschaltet sowie die Ausgänge ALDO1–4 und BLDO2
mit den Netznamen `VL1_3.3V`, `VL2_3.3V`, `VL3_3V`, `VL3_1.8V` und `VL_2.8V`.
Welche Last an welcher Schiene hängt, ist aus der PDF-Textebene nicht
ablesbar. Das Szenario **AXP2101-Schienen** schaltet deshalb jede
standardmäßig aktive Schiene einzeln ab: Wird dabei das Display dunkel,
versorgt sie das AMOLED, und an einem Strommessgerät ist die Wirkung jeder
Schiene sichtbar. Das Ergebnis gehört in die Tabelle auf der Seite
[Stromverbrauch](stromverbrauch.md#messwerte).

!!! warning "Vorsicht"
    Das Abschalten einer Schiene kann das Display schwarz schalten oder den
    Chip zurücksetzen. Die Schiene wird nach 8 s wieder eingeschaltet.

## Stolpersteine

- PSRAM, Flash, AMOLED-Treiber und die Pull-ups schlafen nicht von allein; ihr
  Anteil am Ruhestrom ist im Datenblatt nicht ausgewiesen.
- Vor dem Deep-Sleep sollte das Display in den Sleep-Modus (`Co5300::sleep`),
  sonst leuchtet das AMOLED weiter.
- Das Datenblatt nennt die Light-Sleep-Werte **ohne PSRAM**; für den S3R8
  kommen laut Fußnote 140 µA dazu.
- Aufwachen aus Deep-Sleep dauert länger als aus Light-Sleep, weil die Firmware
  neu startet.

## Herstellerunterlagen

- [ESP32-S3-Datenblatt, Abschnitt 5.6 „Current Consumption“](https://documentation.espressif.com/esp32-s3_datasheet_en.pdf)
- [`esp-hal`, Modul `rtc_cntl::sleep`](https://docs.espressif.com/projects/rust/esp-hal/latest/)
- [AXP2101-Datenblatt](https://www.waveshare.net/w/upload/e/ed/X-power-AXP2101_SWcharge_V1.0.pdf) (Ausgangsschienen, Abschnitt 6.5.4)
