# amulett

Rust-Firmware (`no_std`, [esp-hal](https://github.com/esp-rs/esp-hal)) für das
**Waveshare ESP32-S3-Touch-AMOLED-1.75**. Dieses Projekt ist zunächst ein
Proof of Concept: Jede verbaute Hardware wird einmal angesprochen und
dokumentiert, damit klar ist, was möglich ist.

- [Hardware](hardware.md): Bausteine, Pinbelegung und I2C-Adressen
- Je Baustein eine Seite mit Funktionsweise, Code und Herstellerlinks:
  [Energie](energie.md), [Display](display.md), [Touch und Tasten](eingabe.md),
  [Bewegungssensor](imu.md), [Audio](audio.md), [Funk](funk.md),
  [Weitere Bausteine](weitere-bausteine.md)
- [Stromverbrauch](stromverbrauch.md): Datenblattwerte (noch ohne eigene Messung)

Quellcode: [github.com/RincewindWizzard/amulett](https://github.com/RincewindWizzard/amulett)

## Bedienung der Demo

Die Firmware zeigt sechs Seiten auf dem Display:
Testbild + Touch, Energie, IMU, Audio, Funk und Info.

| Eingabe | Wirkung |
|---|---|
| BOOT-Taste oder Wischen nach links | nächste Seite |
| PWR-Taste (kurz) oder Wischen nach rechts | vorherige Seite |
| Tippen auf *Testbild* | Helligkeit wechseln |
| Tippen auf *Audio* | Testton ein/aus (Mikrofonpegel laufen immer) |
| Tippen auf *Funk* | WLAN-Scan (BLE-Advertising läuft beim Betreten der Seite) |

Alle Ereignisse erscheinen zusätzlich auf der seriellen Konsole.
Bauen und Flashen siehe [README](https://github.com/RincewindWizzard/amulett#readme).

## Offizielle Herstellerunterlagen

- [Waveshare-Dokumentation](https://docs.waveshare.com/ESP32-S3-Touch-AMOLED-1.75)
- [Ressourcen und Dokumente](https://docs.waveshare.net/ESP32-S3-Touch-AMOLED-1.75/Resources-And-Documents)
- [Beispielcode und Hardware-Referenz (GitHub)](https://github.com/waveshareteam/ESP32-S3-Touch-AMOLED-1.75)
- [Schaltplan (PDF)](https://github.com/waveshareteam/ESP32-S3-Touch-AMOLED-1.75/blob/main/Schematic/ESP32-S3-Touch-AMOLED-1.75-schematic.pdf)
