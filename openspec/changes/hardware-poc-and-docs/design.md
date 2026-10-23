# Design

## Context

Das Repo enthält ein `no_std`-Gerüst mit esp-hal 1.x (nur I2C-Scan, siehe `src/main.rs`), CI-Build und Release. Hardware laut Spezifikation: ESP32-S3R8 (8 MB PSRAM, 32 MB Flash), CO5300 (QSPI), CST9217 (I2C), QMI8658 (I2C), AXP2101 (I2C), Audio-Codec mit Dual-Mikrofon und Echounterdrückung, PWR-/BOOT-Taste, WLAN/BLE. Pinbelegung und weitere Bausteine (RTC, IO-Expander, SD, Codec-Typ) sind nicht verifiziert und müssen zuerst aus Schaltplan und Herstellerdoku (Waveshare-Wiki) ermittelt werden. Motivation: siehe proposal.md.

## Goals / Non-Goals

**Goals:**
- Pro Peripherie ein eigenes Modul mit kleiner, lesbarer Demo, die als Referenz für spätere Arbeit dient.
- Dokumentation entsteht parallel zum Code und enthält Codeausschnitte direkt aus den Modulen.

**Non-Goals:**
- Produktive Treiber-Crates, Energiesparoptimierung, UI-Design, Sprachassistent, Cloud.

## Decisions

- **`no_std` + esp-hal bleibt**, Async mit Embassy (`esp-rtos`) für die gleichzeitige Nutzung von Touch, IMU und Anzeige. Alternative: esp-idf-hal (std) bietet fertige Audio/WLAN-Komponenten, wurde aber bereits abgelehnt (größerer Build).
- **Treiber:** vorhandene `embedded-hal`-Treiber-Crates nutzen (z. B. für QMI8658, AXP2101), wenn sie mit esp-hal 1.x funktionieren; sonst kleine eigene Register-Zugriffe im Modul, damit die Doku das Protokoll erklären kann. CO5300 und CST9217 vermutlich eigene Implementierung, angelehnt an Herstellerbeispiele (C/Arduino).
- **Grafik:** `embedded-graphics` mit einem Framebuffer im PSRAM (466×466×16 bit ≈ 434 KB, passt nicht in internen SRAM). Alternative: zeilenweise zeichnen ohne Framebuffer — weniger Speicher, aber komplexer für Demos.
- **Gemeinsamer I2C-Bus** wird über einen Shared-Bus-Wrapper (`embassy-embedded-hal` bzw. RefCell-Mutex) auf Touch, IMU, PMIC verteilt.
- **Start-Reihenfolge:** PMIC-Schienen → Display-Reset → Display-Init → übrige Peripherie. Fehler einer Peripherie stoppen die anderen Demos nicht.
- **Demo-Navigation:** Wischgesten/Taste wechseln zwischen Demo-Seiten auf dem Display.
- **Doku:** MkDocs mit Material-Theme, Deutsch, Deploy per `actions/deploy-pages` (Pages-Quelle "GitHub Actions"). Codebeispiele werden per Snippet-Include aus `src/` eingebunden, damit sie nicht veralten.
- **Herstellerlinks:** Waveshare-Wiki-Seite des Boards plus Datenblätter der Bausteine; Links werden beim Schreiben der Seite geprüft.

## Risks / Trade-offs

- [Pinbelegung falsch geraten] → Zuerst Schaltplan/Herstellerdoku auswerten und in der Doku mit Quelle festhalten.
- [Kein ausgereifter CO5300-Treiber in Rust] → Init-Sequenz aus Herstellerbeispiel übernehmen, Quelle verlinken.
- [Audio-Codec/Echounterdrückung aufwändig (I2S, I2C-Konfiguration)] → Erst Ton/Pegel, keine Signalverarbeitung.
- [WLAN/BLE brauchen `esp-radio`, Heap und Scheduler] → Als letzte Demo, falls Aufwand zu hoch separat abspaltbar.
- [Ohne Hardware in der Entwicklungsumgebung nicht testbar] → CI prüft nur den Build; Verifikation der Demos erfolgt manuell auf dem Gerät und wird in der Doku protokolliert.
- [GitHub Pages muss im Repo aktiviert werden] → Einmalige manuelle Einstellung, in der Doku/README vermerkt.

## Open Questions

- Exakte Modellbezeichnung der Zusatzbausteine (Codec, RTC, IO-Expander) — wird beim Auswerten des Schaltplans geklärt.
