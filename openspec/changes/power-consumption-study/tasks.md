# Tasks

## 1. Grundlagen und Datenblattwerte

- [ ] 1.1 Datenblattwerte (ESP32-S3 Schlafmodi und Funk, AXP2101-Ruhestrom, CO5300, QMI8658, Codec) mit Quelle in `docs/stromverbrauch.md` erfassen; prüfen, dass jede Zahl einen Link hat
- [ ] 1.2 Schaltplan auf nicht abschaltbare Verbraucher prüfen (PSRAM, Flash, Pull-ups, LDOs) und Ergebnis in der Doku festhalten
- [ ] 1.3 Messaufbau beschreiben und aufbauen (Messgerät am Akku-Anschluss); Referenzmessung "Board aus/Leerlauf" liegt vor
- [ ] 1.4 MkDocs-Seitengerüst und Navigation für `stromverbrauch`, `schlafmodi`, `ble-energie` anlegen; `mkdocs build --strict` läuft fehlerfrei

## 2. Schlafmodi in der Firmware

- [ ] 2.1 `sleep`-Modul mit Zuständen Aktiv, Modem-Sleep, Light Sleep, Deep Sleep; Wechsel per Konsole belegen
- [ ] 2.2 Aufwachquellen Timer und Taste umsetzen, Aufwachgrund melden; per Test auf Gerät (10-s-Timer, Tastendruck) prüfen
- [ ] 2.3 Wakeup per Touch- oder IMU-Interrupt umsetzen und auf Gerät prüfen
- [ ] 2.4 RTC-Zähler über Deep Sleep hinweg; Wert steigt nach jedem Aufwachen
- [ ] 2.5 Doku `docs/schlafmodi.md` mit Codeausschnitten, Wakeup-Tabelle und Stolpersteinen schreiben

## 3. Peripherie abschalten

- [ ] 3.1 Display-Sleep und Abschalten der Schiene am AXP2101; Display ist dunkel, Stromaufnahme sinkt messbar
- [ ] 3.2 Audio-Codec und IMU in Low-Power versetzen; Wirkung per Messung belegen
- [ ] 3.3 Messreihe je Komponente (an/aus) durchführen und Werte in die Verbrauchstabelle (Datei) eintragen

## 4. BLE-Energie

- [ ] 4.1 Advertising mit konfigurierbaren Intervallen (100 ms, 1 s, 5 s); Sichtbarkeit in BLE-App prüfen
- [ ] 4.2 Verbindung mit konfigurierbarem Connection-Intervall und Slave Latency; Datenübertragung prüfen
- [ ] 4.3 Zyklus "Deep Sleep, aufwachen, BLE senden, schlafen" mit 60 s Intervall; Nachricht kommt bei jedem Zyklus an
- [ ] 4.4 BLE-Messreihe (Advertising, verbunden, Zyklus) durchführen; Werte in die Tabelle eintragen
- [ ] 4.5 Doku `docs/ble-energie.md` mit Intervall-Trade-offs und Messkurven schreiben

## 5. Auswertung

- [ ] 5.1 Verbrauchstabelle vervollständigen (Datenblatt vs. gemessen, Abweichungen kommentiert) und mit `mkdocs build --strict` prüfen
- [ ] 5.2 Akkulaufzeit für mindestens drei Nutzungsprofile berechnen (mit Rechenweg und Akkukapazität)
- [ ] 5.3 Empfehlungen priorisiert mit Einsparpotenzial dokumentieren
- [ ] 5.4 Gesamtprüfung: Doku-Build in CI grün, alle Links erreichbar

## Workflow follow-up

- Archive the change after the project's review requirements are satisfied.
