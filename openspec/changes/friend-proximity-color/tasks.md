# Tasks

> **Stand der Umsetzung:** Kernlogik (Gruppe 2) mit 28 Unit-Tests, Funk-, Anzeige- und Demo-Code sind geschrieben und kompilieren (Produkt- und `poc`-Build); die Doku steht. Offen bleibt alles, was ein oder mehrere Geräte braucht (Funk-Spike, Beacons sichtbar, Anzeige, Feldtest) sowie der CI-Lauf der neuen Host-Tests (5.1).

## 1. Spikes (klären Risiken, bevor Funktionen gebaut werden)

- [x] 1.1 Host-Tests: `crates/amulett-core` mit Beispieltest anlegen und `cargo test --target x86_64-unknown-linux-gnu` im Projekt lauffähig machen (ggf. Konfiguration anpassen); Befehl und Ergebnis in `docs/entwicklung.md` festhalten
- [ ] 1.2 Spike Funk: gleichzeitig Advertising (nicht verbindbar, mit Hersteller-Daten) und passives Scannen starten; zwei Geräte sehen sich gegenseitig in der Konsole (Geräte-Kennung und RSSI), Ergebnis dokumentieren

## 2. Kernlogik (`crates/amulett-core`)

- [x] 2.1 Parameter-Modul mit allen Konstanten (Intervall, Scan-Fenster, Ablaufzeit, EMA-Faktor, R_nah/R_fern, N_VOLL, V_MIN/V_MAX, Farbton, Änderungsraten) und prüfen, dass es nur an dieser Stelle definiert ist
- [x] 2.2 Freundestabelle: Eintrag je Geräte-Kennung, EMA-Glättung, Ablauf, Eigene-Kennung ignorieren; Unit-Tests für Ausreißer, Ablauf und Eigenecho bestehen
- [x] 2.3 Freundesmaße `n_eff` und `c_max`; Unit-Tests für: kein Freund = 0, mehr Freunde = höheres `n_eff`, näher = höheres `c_max`, Obergrenzen
- [x] 2.4 Farbregler: Ziel aus `n_eff`/`c_max`, begrenzte Änderungsrate aufwärts/abwärts, Quantisierung; Unit-Tests für Sprung von 0 auf Voll, Ausblenden auf Grau und Stillstand
- [x] 2.5 `hsv_to_rgb565` (ganzzahlig) mit Unit-Tests für Grau (S=0), Volltöne und Monotonie der Helligkeit
- [x] 2.6 Beacon-Nutzdaten kodieren/dekodieren mit Unit-Tests (Rundlauf, Fremdgeräte werden verworfen)

## 3. Beacons senden und empfangen

- [ ] 3.1 Advertiser auf nicht verbindbare PDU mit Hersteller-Daten umstellen; in einer BLE-Scanner-App sind Kennung und Geräte-Kennung sichtbar
- [ ] 3.2 Scanner mit Auswertung der Advertising-Reports (ohne Duplikatfilter) und Tastverhältnis aus den Parametern; Reports erscheinen mit RSSI in der Konsole
- [ ] 3.3 Freundestabelle und Maße in die Hauptschleife einbinden; Konsole zeigt je Sekunde Freunde, `n_eff`, `c_max` und Beacon-Rate; mit zwei Geräten prüfen, dass Annähern `c_max` erhöht
- [x] 3.4 Doku `docs/freunde-erkennen.md`: Beacon-Format, Parameter, Nähe-Formel, Grenzen der RSSI-Messung

## 4. Farbanzeige

- [ ] 4.1 Display-Treiber: `fill(color)` ohne Framebuffer; einfarbige Fläche erscheint auf dem Display, Konsole meldet die Schreibdauer
- [ ] 4.2 Normalbetrieb: Farbregler → `fill`; Display wird nur bei geänderter Farbe beschrieben (Konsole zählt Schreibvorgänge)
- [ ] 4.3 Demo-Modus mit virtuellen Freunden (Tippen +1, langer Druck entfernt); Sättigung und Helligkeit ändern sich sichtbar, ohne Freund grau und dunkel
- [ ] 4.4 Debug-Ansicht per BOOT-Taste mit Übersicht (Freunde, RSSI, Rate, `n_eff`, `c_max`) über der Farbfläche
- [x] 4.5 Doku `docs/farbanzeige.md`: Abbildung auf Sättigung/Helligkeit, Tuning-Parameter, Demo-Modus, Feldtest-Anleitung mit zwei bis drei Geräten

## 5. Abschluss

- [x] 5.1 CI: Job für Host-Tests (`amulett-core`) läuft grün
- [ ] 5.2 Feldtest mit mindestens zwei Geräten (nah, fern, durch Körper abgeschirmt, drei Geräte); Messwerte und Schlussfolgerungen in `docs/farbanzeige.md` eintragen und Parameter anpassen

## Workflow follow-up

- Archive the change after the project's review requirements are satisfied.
