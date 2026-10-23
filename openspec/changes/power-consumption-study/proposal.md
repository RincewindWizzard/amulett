# Proposal

## Why

Das Gerät soll später akkubetrieben laufen und per BLE kommunizieren. Ob das mit dem Board praktikabel ist, hängt davon ab, wie wenig Strom es im Schlaf und beim Funken braucht. Die Datenblattwerte des ESP32-S3 (Mikroampere im Deep Sleep) gelten nur für den Chip allein; auf dem Board ziehen AMOLED, Energie-Chip (AXP2101), Audio-Codec, Mikrofone und IMU zusätzlich Strom. Diese Zahlen kennt bisher niemand, und sie bestimmen die Architektur des Produkts.

## What Changes

- Firmware-Demos für die Stromsparmodi des ESP32-S3: Modem-Sleep, Light Sleep und Deep Sleep, mit Aufwachquellen (Timer, Taste, Touch-Interrupt, IMU-Interrupt).
- Abschaltbare Peripherie: Display (Sleep/Off, Versorgungsschienen am AXP2101), Audio, IMU im Low-Power-Modus; Ermittlung, was auf dem Board nicht abschaltbar ist.
- BLE-Energiedemos: Advertising mit verschiedenen Intervallen, Verbindung mit verschiedenen Connection-Intervallen und Slave Latency, Wake-up für periodische Kommunikation.
- Messreihe pro Zustand mit dokumentiertem Messaufbau; Ergebnisse als Tabelle (Datenblattwert vs. gemessener Wert).
- Doku-Erweiterung: Seiten zu Schlafmodi, BLE-Energie und eine Verbrauchstabelle je Komponente und Zustand, mit Links zu Datenblättern und Herstellerdoku, plus Hochrechnung der Akkulaufzeit.

## Capabilities

### New Capabilities
- `low-power-operation`: Betrieb in definierten Stromsparzuständen inklusive Aufwachquellen, Abschalten der Peripherie und BLE im Niedrigenergiebetrieb.
- `power-consumption-documentation`: Dokumentierte und nachvollziehbare Verbrauchswerte je Zustand und Komponente samt Messmethode und Akkulaufzeit-Abschätzung.

### Modified Capabilities
<!-- keine -->

## Impact

- Code: neue Module (`power`/`sleep`, BLE-Demo erweitert), ggf. Demo-Menü der Änderung `hardware-poc-and-docs` um Stromspar-Szenarien ergänzt.
- Doku: neue Seiten unter `docs/` und Eintrag in der MkDocs-Navigation.
- Abhängigkeit: baut auf `hardware-poc-and-docs` auf (AXP2101-, Display-, I2C-, BLE-Module und MkDocs-Seite). Beide Änderungen sollten nacheinander umgesetzt werden.
- Hardware: Messungen brauchen ein Messgerät (siehe design.md); ohne Messung bleiben nur Datenblattwerte.
- Nicht Teil dieser Änderung: Produktfunktionen und Optimierung der Firmware für einen konkreten Anwendungsfall.
