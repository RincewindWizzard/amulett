# Tasks

## 1. Spikes (klären Risiken, bevor Funktionen gebaut werden)

- [ ] 1.1 Host-Tests: `crates/amulett-core` mit Beispieltest anlegen und `cargo test --target x86_64-unknown-linux-gnu` im Projekt lauffähig machen (ggf. Konfiguration anpassen); Befehl und Ergebnis in `docs/entwicklung.md` festhalten
- [ ] 1.2 Spike Funk: gleichzeitig Advertising (nicht verbindbar, mit Hersteller-Daten) und passives Scannen starten; zwei Geräte sehen sich gegenseitig in der Konsole (Geräte-Kennung und RSSI), Ergebnis dokumentieren
- [ ] 1.3 Spike Flash: Lesegeschwindigkeit der Partition mit `esp-storage` für 108 KB und 434 KB messen und in der Doku eintragen; Entscheidung über Frame-Größe/Format begründen

## 2. Kernlogik (`crates/amulett-core`)

- [ ] 2.1 Parameter-Modul mit allen Konstanten (Intervall, Scan-Fenster, Ablaufzeit, EMA-Faktor, R_nah/R_fern, Geschwindigkeiten) und prüfen, dass es nur an dieser Stelle definiert ist
- [ ] 2.2 Freundestabelle: Eintrag je Geräte-Kennung, EMA-Glättung, Ablauf, Eigene-Kennung ignorieren; Unit-Tests für Ausreißer, Ablauf und Eigenecho bestehen
- [ ] 2.3 Nähe-Wert (noisy-OR) mit Unit-Tests für: kein Freund = 0, mehr Freunde = höher, näher = höher, Obergrenze 1
- [ ] 2.4 Fortschrittsregler (Ziel, begrenzte Geschwindigkeit vorwärts/rückwärts, Hysterese, Frame-Index) mit Unit-Tests für Sprung 0→1, Rücklauf auf Frame 0 und Stillstand
- [ ] 2.5 Frame-Header-Parser (Kennung, Version, Maße, Anzahl) mit Unit-Tests für gültige und ungültige Köpfe
- [ ] 2.6 Beacon-Nutzdaten kodieren/dekodieren mit Unit-Tests (Rundlauf, Fremdgeräte werden verworfen)

## 3. Beacons senden und empfangen

- [ ] 3.1 Advertiser auf nicht verbindbare PDU mit Hersteller-Daten umstellen; in einer BLE-Scanner-App sind Kennung und Geräte-Kennung sichtbar
- [ ] 3.2 Scanner mit Auswertung der Advertising-Reports (ohne Duplikatfilter) und Tastverhältnis aus den Parametern; Reports erscheinen mit RSSI in der Konsole
- [ ] 3.3 Freundestabelle und Nähe-Wert in die Hauptschleife einbinden; Debug-Seite zeigt Freunde, RSSI, Nähe-Wert und Beacon-Rate; mit zwei Geräten prüfen, dass Annähern den Wert erhöht
- [ ] 3.4 Beacon-Rate und RSSI je Freund protokollieren (Konsole), damit beide Verfahren im Feldtest verglichen werden können
- [ ] 3.5 Dokumentation `docs/freunde-erkennen.md`: Beacon-Format, Parameter, Nähe-Formel, Grenzen der RSSI-Messung

## 4. Frames: Format, Partition, Werkzeug

- [ ] 4.1 `partitions.csv` anlegen, in Runner und Image-Erzeugung einbinden; `espflash` schreibt die Tabelle und die Firmware startet weiterhin
- [ ] 4.2 `tools/make_frames.py` (Bildfolge/GIF → `frames.bin`, Zielgröße, Skalierung, Kreismaske, Größenprüfung) mit Tests (pytest) für Kopf, Größe und Fehlerfall
- [ ] 4.3 Platzhalteranimation erzeugen (prozedural, ca. 60 Frames) und Skript dafür ins Repo legen; erzeugte `frames.bin` passt in die Partition
- [ ] 4.4 Firmware-Leser: Kopf prüfen, Frame-Streifen lesen, Fehlermeldung bei leerer/ungültiger Partition; Test mit leerer und gefüllter Partition auf dem Gerät
- [ ] 4.5 Dokumentation `docs/animation-speicher.md`: Format, Budgettabelle je Variante, Flash-Ablauf, Umgang mit Lesezeiten

## 5. Animation anzeigen

- [ ] 5.1 Zeichnen in Streifen mit Skalierung 1/2 und Senden ohne Vollbild-Framebuffer; ein Frame erscheint korrekt auf dem Display
- [ ] 5.2 Hauptbildschirm "Amulett": Nähe-Wert → Fortschrittsregler → Frame; Display wird nur bei Indexwechsel beschrieben; Bildrate ≥ 15/s während der Bewegung (gemessen und in der Konsole ausgegeben)
- [ ] 5.3 Demo-Modus mit simulierten Freunden (Tippen erhöht/verringert die Anzahl); Animation läuft vor und zurück, ohne Freund zurück auf Frame 0
- [ ] 5.4 Dokumentation `docs/animation.md`: Funktionsweise, Tuning-Parameter, Demo-Modus, Feldtest-Anleitung mit zwei bis drei Geräten

## 6. Release und Abschluss

- [ ] 6.1 CI: Job für Host-Tests (`amulett-core`) und Test des Frame-Werkzeugs; beide laufen grün
- [ ] 6.2 Release-Pipeline setzt Bootloader, Partitionstabelle, Firmware und `frames.bin` zu einem Image zusammen; Image wird auf einem Gerät geflasht und startet mit Animation
- [ ] 6.3 Feldtest mit mindestens zwei Geräten (nah, fern, durch Körper abgeschirmt); Messwerte und Schlussfolgerungen in `docs/animation.md` eintragen, Parameter anpassen

## Workflow follow-up

- Archive the change after the project's review requirements are satisfied.
