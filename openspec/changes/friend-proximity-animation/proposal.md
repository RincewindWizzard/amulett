# Proposal

## Why

Das Produkt ist ein **Freundschaftsamulett**: Mehrere Amulette erkennen sich per BLE, und das Display zeigt an, wie viele Freunde in der Nähe sind und wie nah sie sind. Die Hardware-Demos aus `hardware-poc-and-docs` zeigen, dass Display, Touch und BLE ansprechbar sind, aber es gibt noch keine Produktfunktion. Diese Änderung legt die Kernidee fest und macht sie lauffähig, damit sich die offenen Fragen (Animation, Speicher, Reichweite, Akkulaufzeit) mit zwei oder mehr Geräten prüfen lassen.

## What Changes

- **Beacons:** Jedes Amulett sendet dauerhaft einen kleinen BLE-Beacon (Advertising) und scannt zugleich nach Beacons anderer Amulette.
- **Nähe-Wert:** Aus den empfangenen Beacons pro Zeitfenster (Anzahl und Signalstärke je Freund) entsteht ein Nähe-Wert, der Anzahl und Nähe der Freunde zusammenfasst.
- **Animation:** Eine Folge vorab erzeugter Einzelbilder (Frames) liegt im Flash. Der Nähe-Wert bestimmt, wie weit die Animation voranschreitet oder zurückläuft: Viele nahe Freunde treiben die Animation vorwärts bis zum Ende, wenige oder keine lassen sie zurücklaufen.
- **Bild-Pipeline:** Ein Werkzeug auf dem Entwicklungsrechner wandelt Bilder/GIFs in das Frame-Format der Firmware um und legt sie in einer eigenen Flash-Partition ab.
- **Doku:** Neue Seiten zu Funktionsweise, Speicherbudget, Frame-Format und Tuning-Parametern.

## Capabilities

### New Capabilities
- `friend-presence-detection`: Senden und Empfangen der Freundes-Beacons, Erkennen eigener Amulette, Glättung und Ablauf von Einträgen, Berechnung des Nähe-Werts.
- `proximity-animation`: Anzeige einer Frame-Animation, deren Position vom Nähe-Wert abhängt, einschließlich Glättung und Verhalten ohne Freunde.
- `animation-asset-storage`: Format, Speicherort und Ladeverfahren der Animationsframes sowie das Werkzeug, das sie erzeugt.

### Modified Capabilities
<!-- keine -->

## Impact

- Code: `src/wireless.rs` (Scannen zusätzlich zum Advertising, Auswertung der Advertising-Reports), neue Module für Nähe-Berechnung und Animation, Frame-Zugriff; `src/main.rs` (neuer Hauptbildschirm).
- Neu: Flash-Partitionstabelle, Konverter-Werkzeug (z. B. kleines Rust- oder Python-Skript unter `tools/`), Frames als Build-Artefakt, ggf. angepasste Release-Pipeline (Image enthält Frames).
- Baut auf `hardware-poc-and-docs` auf (Display-, BLE- und Flash-Image-Grundlagen). Beide Änderungen sollten nacheinander umgesetzt werden; die offenen Geräteprüfungen dort (BLE, Display) sind Voraussetzung für belastbare Tests hier.
- Stromverbrauch: Dauerhaftes Scannen und Display-Animation sind die größten Verbraucher; Betriebsdauer fällt in `power-consumption-study`.
- Nicht Teil dieser Änderung: Kopplung/Authentifizierung von Freunden, mehrere Animationen, Bedienung per Touch, Verbindungen oder Datenaustausch zwischen Amuletten, Gestaltung der endgültigen Animation.
