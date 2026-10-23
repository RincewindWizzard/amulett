# Tasks

## 1. Hardware-Recherche und Projektstruktur

- [ ] 1.1 Schaltplan und Waveshare-Wiki auswerten; Bausteine, Pins und Adressen in `docs/hardware.md` festhalten und prüfen, dass jede Zeile eine Quelle verlinkt
- [ ] 1.2 `src/` in Module je Peripherie aufteilen (`display`, `touch`, `buttons`, `imu`, `pmic`, `audio`, `wireless`) und prüfen, dass `cargo build --release` weiterhin läuft
- [ ] 1.3 Abhängigkeiten hinzufügen (embedded-graphics, async-Runtime, Shared-I2C) und prüfen, dass der Build mit `--locked` nach Aktualisierung von `Cargo.lock` läuft

## 2. Energie und I2C-Basis

- [ ] 2.1 AXP2101-Modul: Schienen aktivieren, Batteriespannung/Ladestatus lesen; per Konsole auf Gerät verifizieren
- [ ] 2.2 Gemeinsamen I2C-Bus mit I2C-Scan umsetzen und prüfen, dass alle erwarteten Adressen erscheinen
- [ ] 2.3 Doku-Seite `docs/pmic.md` mit Pins, Codeausschnitt, Stolpersteinen und Herstellerlink schreiben und mit `mkdocs build --strict` prüfen

## 3. Display

- [ ] 3.1 CO5300-Init per QSPI mit Reset-Sequenz implementieren und verifizieren, dass das Display initialisiert wird
- [ ] 3.2 Testbild (RGB-Flächen, Randmarkierung) und Helligkeitsänderung umsetzen; auf Gerät sichtbar prüfen
- [ ] 3.3 Doku-Seite `docs/display.md` schreiben (Framebuffer/PSRAM, Init-Sequenz, Herstellerlink)

## 4. Eingaben

- [ ] 4.1 CST9217-Touch (I2C, Interrupt) implementieren; Koordinaten werden auf Display markiert und in der Konsole ausgegeben
- [ ] 4.2 PWR- und BOOT-Taste als Ereignisse erfassen und auf dem Display anzeigen
- [ ] 4.3 Doku-Seite `docs/input.md` mit Herstellerlinks schreiben

## 5. Bewegungssensor und weitere I2C-Bausteine

- [ ] 5.1 QMI8658 initialisieren, Beschleunigung und Gyro anzeigen; Ruhewert ≈ 1 g auf Z prüfen
- [ ] 5.2 Weitere gefundene I2C-Bausteine (z. B. RTC, IO-Expander) je einmal initialisieren und einen Wert ausgeben
- [ ] 5.3 Doku-Seite `docs/imu.md` und `docs/weitere-bausteine.md` schreiben

## 6. Audio und Funk

- [ ] 6.1 Codec über I2C/I2S konfigurieren, Testton ausgeben; Ton ist hörbar
- [ ] 6.2 Beide Mikrofone aufnehmen, Pegel je Kanal anzeigen; Pegel reagiert auf Klatschen
- [ ] 6.3 WLAN-Scan ausgeben und BLE-Advertising starten; Netze erscheinen in der Konsole, Gerät in BLE-App
- [ ] 6.4 Doku-Seiten `docs/audio.md` und `docs/funk.md` schreiben

## 7. Demo-Menü

- [ ] 7.1 Demo-Navigation (Wischen/Taste) zwischen den Peripherie-Seiten umsetzen und auf Gerät prüfen, dass jede Demo erreichbar ist

## 8. Dokumentationsseite und Veröffentlichung

- [ ] 8.1 `mkdocs.yml` (Material, Deutsch, Navigation, Snippet-Includes aus `src/`) und `docs/index.md` anlegen; `mkdocs build --strict` läuft fehlerfrei
- [ ] 8.2 `.github/workflows/docs.yml` für Build bei PR und Deploy auf GitHub Pages bei Push auf `main` anlegen; Workflow-Lauf ist grün und Seite erreichbar
- [ ] 8.3 README mit Link auf die gehostete Doku-URL ergänzen und prüfen, dass der Link auflöst
- [ ] 8.4 Gesamtprüfung: alle Herstellerlinks in der Doku erreichbar, CI-Build und Doku-Build grün

## Workflow follow-up

- GitHub Pages im Repo einmalig auf Quelle "GitHub Actions" stellen.
- Change nach Abschluss archivieren.
