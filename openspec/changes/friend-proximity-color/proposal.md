# Proposal

## Why

Das Produkt ist ein **Freundschaftsamulett**: Mehrere Amulette erkennen sich per BLE, und das Display zeigt, wie viele Freunde in der Nähe sind und wie nah sie sind. Die Hardware-Demos aus `hardware-poc-and-docs` zeigen, dass Display und BLE ansprechbar sind, aber es gibt noch keine Produktfunktion. Eine Animation gibt es noch nicht. Deshalb wird die Freundesmenge zunächst **über Farbe** dargestellt: Sättigung und Helligkeit der Displayfläche zeigen Anzahl und Nähe. So lassen sich Erkennung, Reichweite und Akkulaufzeit mit mehreren Geräten prüfen, bevor Bildmaterial existiert. (Die frühere Planung mit Frame-Animation und Flash-Speicher ist zurückgestellt.)

## What Changes

- **Beacons:** Jedes Amulett sendet dauerhaft einen kleinen BLE-Beacon (Advertising) und scannt zugleich nach Beacons anderer Amulette.
- **Freundesmaße:** Aus den empfangenen Beacons (Anzahl und Signalstärke je Freund, geglättet) entstehen zwei Werte: eine **gewichtete Freundesanzahl** und die **Nähe des nächsten Freundes**.
- **Farbanzeige:** Das Display füllt sich mit einer Grundfarbe (Farbton fest, später je Fraktion). Die **Sättigung** steigt mit der Anzahl der Freunde, die **Helligkeit** mit der Nähe des nächsten Freundes. Ohne Freunde ist die Fläche dunkel und grau, mit vielen nahen Freunden kräftig und hell.
- **Weicher Übergang:** Die Farbe ändert sich langsam und gleichmäßig; kein Flackern bei schwankendem Signal.
- **Debug-Ansicht:** Mit der BOOT-Taste lässt sich eine Übersicht mit Freunden, RSSI und Beacon-Rate ein- und ausblenden, um Werte im Feldtest zu beobachten.
- **Doku:** Neue Seite zu Funktionsweise, Beacon-Format und Parametern.

## Capabilities

### New Capabilities
- `friend-presence-detection`: Senden und Empfangen der Freundes-Beacons, Erkennen eigener Amulette, Glättung und Ablauf von Einträgen, Berechnung von Freundesanzahl und Nähe.
- `proximity-color-display`: Darstellung von Anzahl und Nähe der Freunde als Sättigung und Helligkeit einer Farbe, einschließlich Glättung, Debug-Ansicht und Demo-Modus.

### Modified Capabilities
<!-- keine -->

## Impact

- Code: `src/wireless.rs` (Scannen zusätzlich zum Advertising, Auswertung der Advertising-Reports), neues Crate `crates/amulett-core` für die reine Logik, neue Anzeige in `src/main.rs`.
- Baut auf `hardware-poc-and-docs` auf (Display- und BLE-Grundlagen). Die dort offenen Geräteprüfungen (BLE, Display) sind Voraussetzung für belastbare Tests hier.
- Stromverbrauch: Dauerhaftes Scannen ist der größte Verbraucher; Betriebsdauer fällt in `power-consumption-study`. Ein einfarbiges Display ist für das AMOLED günstig (dunkle Pixel verbrauchen kaum Strom); die Helligkeit wirkt direkt auf den Verbrauch.
- Nicht Teil dieser Änderung: Frame-Animation, Flash-Partition für Bilder, Bild-Werkzeug, Kopplung/Authentifizierung, Touch-Bedienung, Datenaustausch zwischen Amuletten. Die Fraktionen folgen in `faction-selection-and-danger`.
