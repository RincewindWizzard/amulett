# Design

## Context

Aufbau auf `hardware-poc-and-docs`: Modulstruktur je Peripherie, AXP2101-Modul, Display, BLE, MkDocs-Seite. Die Hardware (ESP32-S3R8 mit 8 MB Octal-PSRAM, 32 MB Flash, AMOLED, AXP2101, Codec, QMI8658) ist nicht verifiziert in Bezug auf Schaltung und Pinbelegung. Der Chip selbst erreicht laut Datenblatt im Deep Sleep einige Mikroampere, Light Sleep einige hundert Mikroampere und mit aktivem Funk zig Milliampere; konkrete Zahlen werden beim Schreiben der Doku aus dem Datenblatt mit Link übernommen, nicht aus dem Gedächtnis. Der Board-Verbrauch liegt wegen PSRAM, Flash, PMIC-Ruhestrom und nicht abschaltbarer Bausteine wahrscheinlich deutlich höher.

## Goals / Non-Goals

**Goals:**
- Reproduzierbare Verbrauchsmessung je Zustand und Komponente.
- Klare Antwort, was das Board im tiefsten erreichbaren Schlaf und bei BLE-Betrieb braucht.

**Non-Goals:**
- Optimieren auf ein konkretes Produktprofil; eigene Hardware-Modifikationen am Board (z. B. Bauteile auslöten).

## Decisions

- **Messpunkt:** Versorgung über den Akku-Anschluss (MX1.25) mit einem Strommessgerät, das µA bis 100 mA abdeckt (z. B. Nordic PPK2 oder Otii; alternativ INA226-Modul mit Mittelwertbildung). USB-Messung ist ungeeignet, weil Ladeelektronik und USB-Seite mitmessen. Alternative Shunt + Oszilloskop wird nur für BLE-Strombursts genutzt. Akku-Strom bei angeschlossenem USB ist nicht messbar; Messungen laufen daher ohne USB.
- **Zustandsmatrix:** Jede Messung nennt Zustand (Aktiv, Modem-Sleep, Light, Deep), CPU-Takt, Display-Zustand, Funk-Zustand. Bausteine werden schrittweise zugeschaltet, damit Differenzen den Beitrag je Komponente ergeben.
- **Deep Sleep mit RTC-Daten:** esp-hal Sleep-APIs (`Rtc`, `TimerWakeupSource`, `Ext1WakeupSource`/GPIO-Wakeup); Zustand im RTC-Speicher über `#[ram(unstable(rtc_fast))]`-Attribute oder gleichwertig. Genaue API wird gegen die verwendete esp-hal-Version geprüft.
- **BLE-Stack:** `esp-radio` mit `trouble`, falls Powermanagement/Sleep zwischen Funkereignissen dort unterstützt wird; sonst Messung ohne Modem-Sleep und Vermerk in der Doku. Alternative NimBLE über ESP-IDF (std) wird nicht verfolgt.
- **Peripherie abschalten:** Display über CO5300-Sleep-Befehl und anschließend Abschalten der AMOLED-Schiene im AXP2101; Audio-Codec-Powerdown; IMU in Low-Power oder Powerdown mit Interrupt-Wakeup. Was am AXP2101 nicht abschaltbar ist, wird gemessen und als Restverbrauch ausgewiesen.
- **Doku:** Seiten `docs/stromverbrauch.md` (Tabelle, Messaufbau, Rechenweg), `docs/schlafmodi.md` (Modi, Wakeup-Quellen, Code), `docs/ble-energie.md` (Intervalle, Bursts, Profile). Verbrauchstabelle als Datei (z. B. CSV/YAML) aus der die Seite generiert oder eingebunden wird, damit Zahlen nicht auseinanderlaufen. Datenblatt-Links zu ESP32-S3, AXP2101, CO5300, QMI8658, Codec.

## Risks / Trade-offs

- [Kein Messgerät vorhanden] → Messaufgaben bleiben offen; Doku weist Datenblattwerte aus und markiert fehlende Messwerte klar als "nicht gemessen".
- [Board hat hohen Ruhestrom durch nicht abschaltbare Bausteine] → Ergebnis dokumentieren und Gegenmaßnahmen (z. B. anderes Board, Hardware-Anpassung) als Empfehlung nennen.
- [PSRAM, Flash und AMOLED-Treiber schlafen nicht von allein] → Vor Sleep explizit in Low-Power versetzen und das Ergebnis messen.
- [BLE-Sleep in Rust-Stack unreif] → Verbrauch auch ohne optimalen Sleep messen und Stand der Software-Unterstützung dokumentieren.
- [Messwerte schwanken (Akkuspannung, Temperatur, Funkumgebung)] → Mittelwerte über definierte Zeiträume, Umgebung notieren.

## Open Questions

- Welcher Akku (Kapazität) wird für die Laufzeitrechnung angenommen? Platzhalter: 3,7 V / 500 mAh, anpassbar.
- Ob ein Messgerät vorhanden ist, klärt sich vor der Umsetzung der Messaufgaben.
