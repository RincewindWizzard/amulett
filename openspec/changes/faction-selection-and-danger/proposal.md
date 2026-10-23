# Proposal

## Why

Aus den Freundschaftsamuletten wird ein Spiel mit zwei Lagern: Wer zur eigenen Fraktion gehört, ist Freund, wer zur anderen gehört, ist Feind. Damit wird aus der reinen Nähe-Anzeige eine Spielmechanik: Das Amulett zeigt, wer verbündet ist, und warnt, wenn ein Feind in der Nähe ist. Die Änderung `friend-proximity-color` erkennt bisher alle Amulette gleichermaßen als Freunde. Diese Änderung führt die Unterscheidung ein.

## What Changes

- **Auswahl beim Start:** Nach dem Einschalten wählt man mit den Tasten die Fraktion A oder B (BOOT wechselt die Auswahl, PWR bestätigt). Erst danach startet das normale Programm.
- **Fraktion im Beacon:** Der Beacon enthält die Fraktion des Senders. Amulette gleicher Fraktion sind Freunde, Amulette der anderen Fraktion Feinde.
- **Nur Freunde zählen:** Die Freundesmaße und damit die Farbfläche (Sättigung = Anzahl, Helligkeit = Nähe) berücksichtigen nur Amulette der eigenen Fraktion. Der **Farbton** der Fläche ist die Fraktionsfarbe (A blau, B orange).
- **Gefahrenanzeige:** Werden Beacons der fremden Fraktion nah genug empfangen, zeigt das Amulett deutlich "Gefahr" (rot pulsierend, stärker je näher der Feind). Sie endet, wenn der Feind nicht mehr empfangen wird.
- **Fraktionsanzeige:** Die gewählte Fraktion ist dauerhaft am Bildschirm erkennbar (Farbe und Buchstabe).
- **Doku:** Neue Seite zu Fraktionen, Beacon-Format, Gefahr-Parametern und Grenzen (BLE-Beacons sind fälschbar).

## Capabilities

### New Capabilities
- `faction-selection`: Wahl der Fraktion beim Start per Tasten, Anzeige der eigenen Fraktion und Übertragung im Beacon.
- `faction-danger-alert`: Unterscheidung Freund/Feind beim Empfang, Beschränkung der Freundesmaße auf die eigene Fraktion und Gefahrenanzeige bei Feinden in der Nähe.

### Modified Capabilities
<!-- keine: die Capabilities aus friend-proximity-color sind noch nicht archiviert; ihre Anpassungen stehen hier als neue Anforderungen -->

## Impact

- Code: Beacon-Nutzdaten (`amulett-core`, `src/wireless.rs`) um ein Fraktionsfeld erweitert; Freundestabelle trennt Freunde und Feinde; neuer Startbildschirm und Gefahrenbildschirm in `src/main.rs`; Zustandsautomat Start → Auswahl → Normalbetrieb ⇄ Gefahr.
- Abhängigkeit: Setzt `friend-proximity-color` voraus (Beacon, Scanner, Freundestabelle, Freundesmaße, Farbanzeige). Der Beacon-Version wird erhöht.
- Betrieb: Feinde werden wie Freunde per Scan erkannt, der Stromverbrauch ändert sich nicht.
- Nicht Teil dieser Änderung: mehr als zwei Fraktionen (das Format lässt sie zu), Speichern der Fraktion über den Neustart, Fälschungssicherheit (Signaturen), Spielziele/Punkte, Vibration (keine Hardware), Ton als Standard.
