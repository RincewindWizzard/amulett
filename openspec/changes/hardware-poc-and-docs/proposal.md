# Proposal

## Why

Das Board (Waveshare ESP32-S3-Touch-AMOLED-1.75) hat viele Peripheriegeräte, aber es gibt noch keinen Code, der zeigt, wie man sie in Rust (esp-hal, `no_std`) ansteuert. Bevor Anforderungen für das eigentliche Produkt (Amulett) gesammelt werden, muss jede Hardware einmal initialisiert, benutzt und dokumentiert sein, damit klar ist, was möglich ist und wo die Stolpersteine liegen.

## What Changes

- Proof-of-Concept-Firmware, die jede verbaute Hardware einmal initialisiert und sichtbar/hörbar benutzt: AXP2101 (Energie), CO5300-AMOLED (QSPI), CST9217-Touch, PWR-/BOOT-Taste, QMI8658-IMU, Audio (Codec, Mikrofone, Lautsprecher), WLAN/BLE sowie weitere auf dem Board gefundene Bausteine (z. B. RTC, IO-Expander, SD-Karte, soweit vorhanden).
- Jede Peripherie bekommt ein eigenes Modul und eine kleine Demo; ein Demo-Menü/Status auf dem Display zeigt die Ergebnisse (Touch-Koordinaten, IMU-Werte, Tastenstatus).
- MkDocs-Dokumentation (Deutsch) mit einer Seite pro Peripherie: Funktionsweise, Pinbelegung, Codebeispiel, Stolpersteine und Link auf die passende Seite der Herstellerdoku.
- GitHub Action, die die Doku baut und auf GitHub Pages veröffentlicht.
- README verlinkt auf die gehostete Doku.

## Capabilities

### New Capabilities
- `peripheral-poc-display`: Initialisierung des CO5300-AMOLED per QSPI und sichtbare Ausgabe auf dem runden 466×466-Display.
- `peripheral-poc-input`: Erkennung von Touch-Berührungen (CST9217) und der PWR-/BOOT-Taste.
- `peripheral-poc-sensors-power`: Auslesen des QMI8658-IMU sowie Energieverwaltung und Batteriestatus über den AXP2101.
- `peripheral-poc-audio-wireless`: Audio-Ein-/Ausgabe (Mikrofone, Codec, Lautsprecher) und WLAN-/BLE-Grundfunktionen.
- `project-documentation`: MkDocs-Doku mit Herstellerlinks, automatische Veröffentlichung auf GitHub Pages, Link im README.

### Modified Capabilities
<!-- keine -->

## Impact

- Code: `src/` wird in Module je Peripherie aufgeteilt; neue Abhängigkeiten (z. B. `embedded-hal`, ein Grafik-Crate wie `embedded-graphics`, ggf. `esp-radio`, `esp-alloc`).
- Neu: `mkdocs.yml`, `docs/`, `.github/workflows/docs.yml`, Anpassung von `README.md`.
- Die Pinbelegung muss gegen Schaltplan/Herstellerdoku verifiziert werden; bisher angenommene GPIOs im Gerüst sind ungeprüft.
- Nicht Teil dieser Änderung: Produktfunktionen (Sprachassistent, UI-Design, Cloud-Anbindung).
