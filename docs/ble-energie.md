# BLE und Energie

Der Funk ist der größte Verbraucher, sobald das Display aus ist. Diese Seite
sammelt, was das Datenblatt dazu sagt, und beschreibt die BLE-Szenarien des
[Messprogramms](schlafmodi.md#messprogramm). **Messwerte gibt es noch nicht.**

## Was das Datenblatt sagt

Aus [Stromverbrauch](stromverbrauch.md) (ESP32-S3-Datenblatt v2.2, 3,3 V,
Spitzenwerte bei 100 % Tastverhältnis):

| Zustand | Strom |
|---|---:|
| BLE senden, 0 dBm | 176 mA |
| BLE senden, +9 dBm | 193 mA |
| BLE empfangen | 93 mA |
| Modem-Sleep, 80 MHz, CPU im Leerlauf | 22–36 mA |
| Light-Sleep (+ 140 µA PSRAM) | 240 µA |
| Deep-Sleep | 7–8 µA |

Das Datenblatt nennt **keine Mittelwerte** für Advertising oder Verbindungen;
sie hängen von Intervall und Nutzdaten ab.

## Faustformel

Ein Funkereignis dauert wenige Millisekunden. Der mittlere Strom eines Zyklus
mit Periode `T` ist

```
I_mittel = (I_schlaf · (T − t_aktiv) + I_aktiv · t_aktiv) / T
```

mit der aktiven Zeit `t_aktiv` (Aufwachen, Funk, Einschlafen) und dem Strom
`I_aktiv` in dieser Zeit. Beide Größen müssen gemessen werden; die Spitzenwerte
oben sind nur Obergrenzen für den Funkanteil.

## Einflussgrößen

| Größe | Wirkung |
|---|---|
| Advertising-Intervall (`ADV_INTERVAL_MS`) | längeres Intervall → weniger Funkereignisse, aber langsamere Erkennung |
| Scan-Tastverhältnis (`SCAN_WINDOW_MS` / `SCAN_INTERVAL_MS`) | bestimmt den Empfangsanteil (93 mA Spitze) |
| Sendeleistung | höhere Leistung → mehr Strom und Reichweite |
| Display | AMOLED hell → deutlich mehr als alle anderen Verbraucher |
| CPU-Takt | 80 statt 240 MHz spart laut Datenblatt im Leerlauf rund 11 mA |

## Szenarien im Messprogramm

| Szenario | Frage |
|---|---|
| BLE Advertising 100 ms | Strom beim häufigen Senden |
| BLE Advertising 1000 ms | Strom beim üblichen Intervall |
| BLE Advertising 5000 ms | Strom beim seltenen Senden |
| Beacon + Scan | Strom im Produktbetrieb ohne Display |
| Deep Sleep + BLE | Strom beim periodischen Aufwachen (60 s) |

Die **CPU läuft** in den Advertising-Szenarien durchgehend (der Funk-Stack der
Rust-Bibliothek schaltet nicht automatisch in Light-Sleep). Der Wert eines
Szenarios ist daher eine Obergrenze und nicht der Idealwert. Ob automatischer
Light-Sleep zwischen den Funkereignissen möglich ist, ist offen.

## Auswertung

Sobald Messwerte vorliegen, kommen hier die Messkurven, der Vergleich
Datenblatt/Messung und die Empfehlungen für Intervalle hin. Bis dahin bleibt
die Tabelle auf der Seite [Stromverbrauch](stromverbrauch.md#messwerte) leer.
