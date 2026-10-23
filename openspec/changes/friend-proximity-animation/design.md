# Design

## Context

Stand nach `hardware-poc-and-docs` (Code vorhanden, Geräteprüfung offen): `no_std`-Firmware mit esp-hal 1.2, blockierende Hauptschleife (~30 ms pro Bild), Framebuffer im PSRAM, QSPI-Display mit 4-KB-DMA-Blöcken (theoretisch ~22 ms pro Vollbild bei 40 MHz), BLE-Advertising über rohe HCI-Kommandos (`src/wireless.rs`), Scheduler `esp-rtos` läuft. Hardware: 16 MB Flash (Waveshare-Angabe), 8 MB PSRAM, ESP32-S3 BLE-Empfang laut Datenblatt mit 93 mA Spitzenstrom (siehe `docs/stromverbrauch.md`). Motivation und Umfang: siehe proposal.md.

Speicherrechnung (aus den Abmessungen berechnet): Ein Vollbild 466×466 in RGB565 hat 434 312 Byte, bei 233×233 sind es 108 578 Byte, bei 233×233 mit 8-Bit-Palette 54 289 Byte (+ 512 Byte Palette). Das Display ist rund; nur etwa 79 % der Fläche sind sichtbar.

## Goals / Non-Goals

**Goals:**
- Zwei oder mehr Amulette erkennen sich und die Animation reagiert sichtbar auf Anzahl und Nähe.
- Alle Parameter (Intervalle, RSSI-Schwellen, Geschwindigkeit) an einer Stelle einstellbar, damit sich im Feld tunen lässt.
- Die Logik (Freundestabelle, Nähe-Wert, Animationsfortschritt) ist hardwareunabhängig und auf dem Entwicklungsrechner testbar.
- Mit nur einem Gerät prüfbar (simulierte Freunde).

**Non-Goals:**
- Authentifizierung, Kopplung oder Verschlüsselung; Verbindungen/GATT; mehrere Animationen; Touch-Bedienung; Abstandsmessung in Metern (nur relative Nähe).

## Decisions

- **Beacon als nicht verbindbare Advertising-PDU** (`ADV_NONCONN_IND`) mit *Manufacturer Specific Data* (AD-Typ `0xFF`): Company-ID `0xFFFF` (für Tests reserviert), 2 Byte Kennung `"AM"`, 1 Byte Version, 4 Byte Geräte-Kennung (aus der eFuse-MAC), 1 Byte Zähler. Passives Scannen genügt, die Kennung dient nur dem Filtern. Alternative GATT/Verbindung: verworfen (Aufwand, Strom, Begrenzung auf wenige Verbindungen).
- **Scannen über rohe HCI-Kommandos**, wie das Advertising: `LE Set Scan Parameters` (passiv), `LE Set Scan Enable` mit *Filter Duplicates = aus* (sonst werden Wiederholungen nicht gezählt) und Auswertung des Ereignisses *LE Advertising Report* (`0x3E`/`0x02`). Alternative `trouble-host`: mehr Funktionen, aber schwerer und für diesen Zweck nicht nötig. Ob Senden und Scannen gleichzeitig laufen, wird in Spike 1 geklärt.
- **Nähe-Wert aus RSSI statt nur aus der Beacon-Anzahl.** Die Idee "mehr Beacons pro Zeit bedeutet näher" hängt vom Scan-Tastverhältnis und von Kollisionen ab, RSSI beschreibt die Nähe direkter. Je Freund: exponentiell geglätteter RSSI (EMA), abgebildet auf `c = clamp((rssi − R_fern) / (R_nah − R_fern), 0, 1)` mit Startwerten `R_fern = −90 dBm`, `R_nah = −50 dBm`. Aggregation mit `P = 1 − Π(1 − c_i)` (noisy-OR): steigt mit Anzahl und Nähe, bleibt ≤ 1. Die Beacon-Rate dient als Frische-/Ablaufkriterium (Ablauf nach 10 s ohne Beacon) und wird zu Auswertungszwecken mitgeloggt. Alternative: Summe der Beacons im Zeitfenster; kann im Feldtest verglichen werden, weil beide Größen protokolliert werden.
- **Animationsfortschritt als Fließkomma-Position `p` in `[0, N−1]`.** Ziel `t = P·(N−1)`; pro Bild bewegt sich `p` höchstens `v_max·Δt` zum Ziel (vorwärts und rückwärts getrennt einstellbar, z. B. 12 bzw. 6 Frames/s). Angezeigt wird `round(p)`. Das Display wird nur beschrieben, wenn sich der Index ändert. Hysterese verhindert Flackern an Rundungsgrenzen.
- **Frames in eigener Flash-Partition** `frames` (Typ `data`, Subtyp `0x40`) statt `include_bytes!` in der Firmware, damit Firmware und Animation getrennt aktualisiert werden können. Partitionstabelle (`partitions.csv`): Bootloader/Tabelle, `factory` 2 MB ab `0x10000` (die Firmware hat derzeit ~0,6 MB), `frames` ab `0x210000` bis Flash-Ende (~13,9 MiB = 14 614 528 Byte). Lesen mit `esp-storage` (blockierend, ohne alles zu laden).
- **Frame-Format `AMFR` v1**: Kopf (4 Byte Kennung, 1 Byte Version, 1 Byte Pixelformat, 2 Byte Breite, 2 Byte Höhe, 1 Byte Skalierung, 2 Byte Frame-Anzahl, 2 Byte FPS-Hinweis, danach Frame-Daten ohne Lücken). Pixelformate: RGB565 (big-endian, direkt an das Display sendbar) und später Palette 8 Bit. Skalierung 1 oder 2 (Pixel-Verdopplung beim Zeichnen).
- **Startvariante: 233×233, RGB565, Skalierung 2** (108 578 Byte, ca. 134 Frames in der Partition). Vollauflösung 466×466 ergäbe ca. 33 Frames; Palette 8 Bit bei 233×233 ca. 268. Die Wahl ist per Header umschaltbar; die Entscheidung fällt nach Spike 2 (Lesegeschwindigkeit) und nach Sicht auf die Animation.
- **Zeichnen in Streifen** statt Vollbild-Framebuffer: Frame-Streifen aus dem Flash lesen, bei Skalierung 2 zeilenweise verdoppeln und direkt an das Display senden. Spart PSRAM-Zugriffe und Latenz; der vorhandene Framebuffer bleibt für Status-Overlays.
- **Reine Logik in eigenem Crate** `crates/amulett-core` (`no_std`, keine Abhängigkeiten): Freundestabelle, EMA, Nähe-Wert, Fortschrittsregler, Frame-Header-Parser. Unit-Tests laufen auf dem Entwicklungsrechner (`cargo test --target x86_64-unknown-linux-gnu`). Die Firmware hängt per Pfad daran.
- **Werkzeug `tools/make_frames.py`** (Python, Pillow): Bildfolge oder GIF → `frames.bin`; skaliert auf Zielgröße, maskiert optional außerhalb des Kreises, prüft Partitionsgröße. Alternative Rust-Tool: mehr Aufwand ohne Nutzen für den Anfang. Als Platzhalter erzeugt ein Skript prozedurale Frames (z. B. wachsender Ring), solange es keine echte Animation gibt.
- **Test mit einem Gerät**: Ein Demo-Modus erhöht per Tippen die Zahl simulierter Freunde mit festen RSSI-Werten.
- **Release-Image**: Aus Bootloader, Partitionstabelle, Firmware und `frames.bin` wird ein einziges Flash-Image zusammengesetzt (Offsets laut Partitionstabelle).

## Risks / Trade-offs

- [RSSI schwankt stark (Körperabschirmung, Ausrichtung, Reflexionen, ±10 dB)] → Glättung, Hysterese, großzügige Schwellen, Feldtest mit Protokoll; Animation zeigt nur grobe Nähe.
- [Dauerhaftes Scannen ist teuer: Empfang laut Datenblatt bis 93 mA Spitze] → Tastverhältnis konfigurierbar (z. B. 10 % ≙ im Mittel rund 9 mA nur für den Empfang, nach Datenblatt gerechnet, nicht gemessen); Akkulaufzeit in `power-consumption-study` messen.
- [Senden und Scannen gleichzeitig evtl. nicht stabil im ESP-Controller / rohen HCI] → Spike 1; Fallback: abwechselnd senden und scannen (Zeitscheiben).
- [Flash-Lesen zu langsam für 15 Bilder/s] → Spike 2 messen; Gegenmaßnahmen: kleinere Frames, Palette, RLE, Frames im PSRAM zwischenspeichern (8 MB ≙ ca. 73 Frames bei 233×233).
- [Display-Schreibzeit ~22 ms setzt Obergrenze von ~40 Bildern/s] → ausreichend für 15 Bilder/s.
- [Blockierende Hauptschleife verpasst HCI-Ereignisse während des Zeichnens] → Ereignisse zwischen den Streifen abholen; sonst Funk in eigenem `esp-rtos`-Thread.
- [Geräte-Kennung ist dauerhaft sichtbar und erlaubt Wiedererkennung durch Fremde] → bewusst vorerst akzeptiert; rotierende Kennungen bei späterer Kopplung.
- [Host-Tests im Projekt mit `build-std`/Xtensa-Standardtarget] → Spike 0; Fallback: Tests über ein separates, nicht eingebundenes Cargo-Projekt.
- [Urheberrecht der Animation] → eigene oder frei lizenzierte Bilder verwenden.

## Open Questions

- Welche Animation (Motiv, Anzahl Frames)? Annahme: Platzhalteranimation; echte Motive später, ändert Format und Code nicht.
- Sollen alle Amulette als "Freunde" zählen oder nur ein festgelegter Kreis? Annahme: alle Amulette; Kopplung ist eine spätere Änderung.
