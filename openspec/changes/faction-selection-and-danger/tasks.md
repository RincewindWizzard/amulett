# Tasks

> Voraussetzung: `friend-proximity-color` ist umgesetzt (Beacon, Scanner, Freundestabelle, Freundesmaße, Farbanzeige, `amulett-core`).

> **Stand der Umsetzung:** Kernlogik (Gruppe 1) mit 47 Unit-Tests, Auswahlbildschirm, Gefahrenanzeige, Demo-Modus mit Feind, optionaler Warnton und Doku sind umgesetzt und kompilieren. Offen bleiben die Aufgaben, deren Prüfschritt ein oder mehrere Geräte braucht, und der CI-Lauf (4.3).

## 1. Kernlogik (`crates/amulett-core`)

- [x] 1.1 Fraktions-Typ (A, B, unbekannt) und `relation(own, other)`; Unit-Tests: gleiche Fraktion = Freund, verschiedene = Feind, unbekannt = ignoriert
- [x] 1.2 Beacon-Format Version 2 mit `flags`-Byte (Fraktion in Bits 0–3) kodieren und dekodieren; Unit-Tests für Rundlauf, unbekannte Fraktion und Verwerfen von Version 1
- [x] 1.3 Freundestabelle um Beziehung je Eintrag erweitern; Freundesmaße zählen nur Freunde (Unit-Test: nur Feinde ergibt 0) und Gefährdung `g` aus den Feinden (Unit-Test: nächster Feind bestimmt `g`)
- [x] 1.4 Gefahrenautomat mit `G_EIN`, `G_AUS`, Ablaufzeit und Mindestanzahl Beacons; Unit-Tests für Auslösen, Schwanken um die Schwelle (kein Flackern), Ende durch Ablauf und eigene Kennung wird ignoriert
- [x] 1.5 Parameter (Schwellen, Pulsperioden, Fraktionsfarben, Warnton ein/aus) an der zentralen Stelle ergänzen und prüfen, dass es keine Duplikate gibt

## 2. Auswahl und Anzeige

- [ ] 2.1 Zustandsautomat Auswahl → Normal ⇄ Gefahr in `src/main.rs`; Sender und Scanner starten erst nach Bestätigung (Konsole zeigt "Fraktion X bestätigt")
- [ ] 2.2 Auswahlbildschirm: A und B mit Buchstabe und Farbe, Markierung, Hinweis "BOOT wechseln, PWR kurz bestätigen"; BOOT wechselt, PWR bestätigt (auf dem Gerät prüfen)
- [ ] 2.3 Fraktionszeichen im Normalbetrieb am Displayrand, ohne die Farbfläche zu verdecken; Farbton der Fläche ist die Fraktionsfarbe (A blau, B orange)
- [ ] 2.4 Beacon sendet die gewählte Fraktion; in einer BLE-App oder am zweiten Amulett ist der Fraktionswert erkennbar

## 3. Gefahr

- [ ] 3.1 Scanner-Auswertung klassifiziert Freund/Feind/ignoriert; Konsole zeigt je Beacon die Beziehung; Freundesmaße und Farbfläche bleiben bei reinen Feinden unverändert
- [ ] 3.2 Gefahrenanzeige: rotes Pulsieren, Text, Balken, Pulsgeschwindigkeit abhängig von `g`; im Demo-Modus mit simuliertem Feind sichtbar
- [ ] 3.3 Simulierter Feind im Demo-Modus (Tippen schaltet virtuellen Feind mit wählbarer Nähe) für Tests mit einem Gerät
- [ ] 3.4 Optionaler Warnton über `AudioIo`, standardmäßig aus; mit Konstante an hörbar, mit Konstante aus still
- [ ] 3.5 Test mit zwei Geräten A/B: nah → Gefahr, entfernt → Gefahr endet; zwei Geräte gleicher Fraktion → keine Gefahr, Farbfläche zeigt Freunde; Ergebnisse notieren

## 4. Doku und Abschluss

- [x] 4.1 Seite `docs/fraktionen.md`: Spielregel, Bedienung, Beacon-Format Version 2, Gefahr-Parameter, Grenzen (Fälschbarkeit, Latenz, RSSI); `mkdocs build --strict` läuft fehlerfrei
- [x] 4.2 Bedienungsabschnitt in `docs/index.md` und README um Fraktionswahl ergänzen
- [ ] 4.3 CI: Host-Tests (`amulett-core`) laufen mit den neuen Tests grün

## Workflow follow-up

- Archive the change after the project's review requirements are satisfied.
