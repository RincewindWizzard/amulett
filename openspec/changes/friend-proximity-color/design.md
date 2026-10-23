# Design

## Context

Stand nach `hardware-poc-and-docs` (Code vorhanden, Geräteprüfung offen): `no_std`-Firmware mit esp-hal 1.2, blockierende Hauptschleife (~30 ms pro Bild), Framebuffer im PSRAM, QSPI-Display mit 4-KB-DMA-Blöcken (theoretisch ~22 ms pro Vollbild bei 40 MHz), BLE-Advertising über rohe HCI-Kommandos (`src/wireless.rs`), Scheduler `esp-rtos` läuft, BOOT-Taste an GPIO0. BLE-Empfang kostet laut Datenblatt bis 93 mA Spitzenstrom (siehe `docs/stromverbrauch.md`). Es gibt noch keine Animation; die Anzeige ist deshalb eine Farbfläche. Motivation und Umfang: siehe proposal.md.

## Goals / Non-Goals

**Goals:**
- Zwei oder mehr Amulette erkennen sich, und die Farbfläche zeigt Anzahl (Sättigung) und Nähe (Helligkeit).
- Alle Parameter (Intervalle, RSSI-Schwellen, Farbbereiche, Übergangszeiten) an einer Stelle einstellbar, damit sich im Feldtest tunen lässt.
- Die Logik (Freundestabelle, Maße, Farbberechnung) ist hardwareunabhängig und auf dem Entwicklungsrechner testbar.
- Mit nur einem Gerät prüfbar (simulierte Freunde).

**Non-Goals:**
- Frame-Animation und deren Speicher (Flash-Partition, Bildformat, Konverter) — später als eigene Änderung. Die hier berechneten Maße lassen sich dafür unverändert verwenden (z. B. als Frame-Position).
- Authentifizierung, Kopplung, Verbindungen/GATT; Abstand in Metern (nur relative Nähe).

## Decisions

- **Beacon als nicht verbindbare Advertising-PDU** (`ADV_NONCONN_IND`) mit *Manufacturer Specific Data* (AD-Typ `0xFF`): Company-ID `0xFFFF` (für Tests reserviert), 2 Byte Kennung `"AM"`, 1 Byte Version, 4 Byte Geräte-Kennung (aus der eFuse-MAC), 1 Byte Zähler. Passives Scannen genügt. Alternative GATT/Verbindung: verworfen (Aufwand, Strom, wenige Verbindungen).
- **Scannen über rohe HCI-Kommandos**, wie das Advertising: `LE Set Scan Parameters` (passiv), `LE Set Scan Enable` mit *Filter Duplicates = aus* (sonst werden Wiederholungen nicht gezählt), Auswertung des Ereignisses *LE Advertising Report* (`0x3E`/`0x02`). Alternative `trouble-host`: schwerer, für diesen Zweck nicht nötig. Ob Senden und Scannen gleichzeitig laufen, klärt Spike 1.
- **Nähe je Freund aus RSSI.** Je Freund exponentiell geglätteter RSSI (EMA), abgebildet auf `c = clamp((rssi − R_fern) / (R_nah − R_fern), 0, 1)` mit Startwerten `R_fern = −90 dBm`, `R_nah = −50 dBm`. RSSI beschreibt Nähe direkter als die Beacon-Anzahl, die vom Scan-Tastverhältnis und von Kollisionen abhängt. Die Beacon-Rate dient als Frische-/Ablaufkriterium (Ablauf nach 10 s ohne Beacon) und wird im Debug-Bild und in der Konsole mitgeloggt, damit beide Verfahren im Feldtest vergleichbar sind.
- **Zwei Freundesmaße.** `n_eff = Σ c_i` (gewichtete Freundesanzahl) und `c_max = max c_i` (Nähe des nächsten Freundes). Alternative: ein Gesamtwert per noisy-OR `1 − Π(1 − c_i)`; vermischt Anzahl und Nähe und passt schlechter zur gewünschten Trennung in zwei Farbkanäle.
- **Farbabbildung (HSV → RGB565).** Farbton `H` fest (Startwert 210°, später je Fraktion), `S = min(1, n_eff / N_VOLL)` mit `N_VOLL = 4` als Startwert, `V = V_MIN + (1 − V_MIN) · c_max` mit `V_MIN = 0,08`. Ohne Freunde: `S = 0` und `V = V_MIN` (dunkelgrau). Alternative `S` und `V` beide aus einem Wert: weniger Information. Die Umrechnung `hsv_to_rgb565` liegt in `amulett-core` (ganzzahlig, ohne Gleitkomma).
- **Weicher Übergang.** `S` und `V` folgen ihren Zielwerten mit begrenzter Änderungsrate (Startwerte: aufwärts 0,5/s, abwärts 0,25/s). Das Intervall der Anzeige beträgt 50 ms. Das Display wird nur geschrieben, wenn sich der RGB565-Wert ändert.
- **Einfarbige Fläche ohne Framebuffer.** Der Treiber bekommt eine Methode `fill(color)`, die einen 4-KB-Block mit dem Farbwert wiederholt sendet (kein PSRAM-Zugriff). Die Debug-Ansicht nutzt weiterhin den Framebuffer mit Text, dann liegt die Farbfläche als Hintergrund darunter.
- **Debug-Ansicht per BOOT-Taste:** schaltet zwischen reiner Farbfläche und Übersicht (Freunde mit Kennung, RSSI, Rate, `n_eff`, `c_max`) um.
- **Reine Logik in eigenem Crate** `crates/amulett-core` (`no_std`, keine Abhängigkeiten): Freundestabelle, EMA, Maße, Farbregler, HSV-Umrechnung, Beacon-Kodierung. Unit-Tests laufen auf dem Entwicklungsrechner. Die Firmware hängt per Pfad daran.
- **Demo-Modus mit einem Gerät:** virtuelle Freunde mit fester Nähe; Tippen auf dem Display erhöht die Anzahl, ein langer Druck entfernt sie.

## Risks / Trade-offs

- [RSSI schwankt stark (Körperabschirmung, Ausrichtung, Reflexionen, ±10 dB)] → Glättung, großzügige Schwellen, weicher Farbübergang, Feldtest mit Protokoll; Farbe zeigt nur grobe Nähe.
- [Dauerhaftes Scannen ist teuer: Empfang laut Datenblatt bis 93 mA Spitze] → Tastverhältnis konfigurierbar (z. B. 10 % ≙ im Mittel rund 9 mA nur für den Empfang, nach Datenblatt gerechnet, nicht gemessen); Akkulaufzeit in `power-consumption-study` messen.
- [Senden und Scannen gleichzeitig evtl. nicht stabil im ESP-Controller / rohen HCI] → Spike 1; Fallback: abwechselnd senden und scannen.
- [Helle Vollfläche verbraucht auf dem AMOLED spürbar Strom] → `V` oben begrenzbar (Parameter `V_MAX`), Helligkeitsregister des Displays; Messung in `power-consumption-study`.
- [Sättigungsunterschiede sind bei geringem Helligkeitswert schwer zu sehen] → `V_MIN` so wählen, dass Grau und Farbe unterscheidbar bleiben; im Feldtest anpassen.
- [Blockierende Hauptschleife verpasst HCI-Ereignisse beim Schreiben des Displays] → Ereignisse zwischen den Blöcken abholen; sonst Funk in eigenem `esp-rtos`-Thread.
- [Geräte-Kennung ist dauerhaft sichtbar und erlaubt Wiedererkennung durch Fremde] → bewusst akzeptiert; rotierende Kennungen bei späterer Kopplung.
- [Host-Tests im Projekt mit `build-std`/Xtensa-Standardtarget] → Spike 0; Fallback: Tests über ein separates, nicht eingebundenes Cargo-Projekt.

## Open Questions

- Welcher Farbton soll gelten, solange es keine Fraktionen gibt? Annahme: Blau (210°); mit `faction-selection-and-danger` bestimmt die Fraktion den Farbton.
- Sollen Anzahl und Nähe dauerhaft auf getrennte Farbkanäle gelegt werden oder reicht ein Gesamtwert? Annahme: getrennt (Sättigung = Anzahl, Helligkeit = Nähe); der Feldtest entscheidet.
