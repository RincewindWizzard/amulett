# Freunde erkennen

Jedes Amulett sendet einen BLE-Beacon und scannt gleichzeitig nach den Beacons
anderer Amulette. Code: `src/wireless.rs` (Funk), `crates/amulett-core`
(Logik).

## Beacon

Ein nicht verbindbares Advertising-Paket (`ADV_NONCONN_IND`) mit einer
*Manufacturer Specific Data*-Struktur. Es ist weder verbindbar noch abfragbar;
es kennzeichnet nur das Amulett.

| Offset | Größe | Inhalt |
|---:|---:|---|
| 0 | 1 | Länge (10) |
| 1 | 1 | AD-Typ `0xFF` |
| 2 | 2 | Company-ID `0xFFFF` (für Tests reserviert), little-endian |
| 4 | 2 | Kennung `"AM"` |
| 6 | 1 | Format-Version (1) |
| 7 | 4 | Geräte-Kennung (untere vier Bytes der MAC-Adresse), little-endian |
| 11 | 1 | Zähler, steigt jede Sekunde |

Davor steht die übliche Flags-Struktur (`02 01 06`). Beacons mit anderer
Company-ID, Kennung oder Version werden verworfen.

```rust
--8<-- "crates/amulett-core/src/beacon.rs:decode"
```

## Funk

Die HCI-Kommandos gehen direkt an den BLE-Controller. Wichtig sind:

- *Set Event Mask* mit gesetztem Bit 61, sonst liefert der Controller keine
  *LE Meta Events* und damit keine Advertising-Reports,
- *Set Scan Enable* mit **Duplikatfilter aus**, sonst würde jedes Amulett nur
  einmal gemeldet und die Beacon-Rate wäre nicht messbar.

```rust
--8<-- "src/wireless.rs:radio_start"
```

Die Reports kommen als Ereignis mit allen Feldern *je Feld* statt *je Report*
angeordnet; die Auswertung steht in `amulett_core::hci` und ist per Unit-Test
abgedeckt.

## Freundestabelle

Pro Geräte-Kennung (bis zu 16) speichert das Amulett den geglätteten RSSI
(exponentieller Mittelwert), den Zeitpunkt des letzten Beacons und die
Beacon-Rate. Nach 10 s ohne Beacon läuft ein Eintrag ab. Beacons mit der
eigenen Kennung werden ignoriert.

Aus dem RSSI wird je Freund eine **Nähe** zwischen 0 und 1:

```
Nähe = clamp((RSSI − RSSI_FERN) / (RSSI_NAH − RSSI_FERN), 0, 1)
```

mit den Startwerten −90 dBm (fern) und −50 dBm (nah). Daraus entstehen zwei
Maße:

| Maß | Berechnung | Wirkung |
|---|---|---|
| Gewichtete Freundesanzahl `n_eff` | Summe der Nähe aller Freunde | Sättigung |
| Nähe des nächsten Freundes `c_max` | größte Nähe | Helligkeit |

## Parameter

Alle Werte stehen in `crates/amulett-core/src/params.rs`:

| Parameter | Startwert | Bedeutung |
|---|---:|---|
| `ADV_INTERVAL_MS` | 300 | Abstand der eigenen Beacons |
| `SCAN_INTERVAL_MS` / `SCAN_WINDOW_MS` | 100 / 30 | Scan-Tastverhältnis 30 % |
| `EXPIRE_MS` | 10 000 | Freund gilt danach als weg |
| `RSSI_EMA_ALPHA` | 0,25 | Glättung (größer = schneller) |
| `RSSI_NEAR_DBM` / `RSSI_FAR_DBM` | −50 / −90 | Abbildung auf Nähe 1 / 0 |

## Grenzen

- **RSSI ist ungenau.** Körperabschirmung, Ausrichtung der Antenne und
  Reflexionen verändern den Wert um ±10 dB. Die Anzeige zeigt nur grobe Nähe.
- **Stromverbrauch.** Laut Datenblatt braucht der Empfang bis zu 93 mA
  (Spitze). Das Scan-Tastverhältnis bestimmt den Mittelwert, siehe
  [Stromverbrauch](stromverbrauch.md).
- **Keine Absicherung.** Jedes BLE-Gerät kann einen Beacon senden; die
  Geräte-Kennung ist dauerhaft sichtbar.
- **Noch nicht auf dem Gerät erprobt.** Ob Senden und Scannen gleichzeitig
  stabil laufen, ist durch einen Test mit zwei Geräten zu bestätigen.

## Herstellerunterlagen

- [Bluetooth Core Specification, HCI-Kommandos und -Ereignisse](https://www.bluetooth.com/specifications/specs/core-specification-6-0/)
  (Set Event Mask, LE Set Scan Parameters/Enable, LE Advertising Report)
- [ESP32-S3-Datenblatt (Strom beim Empfang)](https://documentation.espressif.com/esp32-s3_datasheet_en.pdf)
- [`esp-radio`](https://docs.espressif.com/projects/rust/esp-radio/latest/)
