# Farbanzeige

Solange es keine Animation gibt, zeigt das Display die Freundesmenge als Farbe.
Die ganze Displayfläche hat eine Farbe, deren Farbton die gewählte
[Fraktion](fraktionen.md) vorgibt (A: 210°, Blau; B: 30°, Orange). Zwei Eigenschaften der Farbe tragen die Information:

| Eigenschaft | Folgt | Bedeutung |
|---|---|---|
| **Sättigung** | gewichtete Freundesanzahl `n_eff` | viele Freunde → kräftige Farbe, keiner → grau |
| **Helligkeit** | Nähe des nächsten Freundes `c_max` | Freund direkt neben mir → hell, keiner → dunkel |

```
Sättigung  = min(1, n_eff / FULL_SATURATION_COUNT)       (Start: 4 Freunde = voll)
Helligkeit = V_MIN + (V_MAX − V_MIN) · c_max             (Start: 0,08 … 1,0)
```

Ohne Freunde ist die Fläche dunkelgrau (nicht ganz schwarz, damit man sieht,
dass das Gerät läuft). Wie `n_eff` und `c_max` entstehen, steht auf der Seite
[Freunde erkennen](freunde-erkennen.md).

## Weicher Übergang

Sättigung und Helligkeit folgen ihren Zielwerten nur mit begrenzter
Geschwindigkeit (`RISE_PER_S` = 0,5/s aufwärts, `FALL_PER_S` = 0,25/s abwärts):
Ein neuer Freund lässt die Farbe in etwa zwei Sekunden aufleuchten, ein
verschwundener blendet in vier Sekunden aus. Dadurch wirken schwankende
RSSI-Werte nicht wie Flackern.

Das Display wird nur beschrieben, wenn sich der RGB565-Farbwert ändert; eine
einfarbige Fläche wird ohne Framebuffer direkt an das Display gesendet
(`Co5300::fill`).

## Bedienung

| Eingabe | Wirkung |
|---|---|
| BOOT-Taste | Debug-Übersicht ein-/ausblenden (Freunde, Kennung, RSSI, Beacon-Rate, `n_eff`, `c_max`, Sättigung und Helligkeit mit Zielwert) |
| PWR-Taste (kurz) | Demo-Modus ein/aus |
| Tippen obere Hälfte (nur Demo) | virtuellen Freund hinzufügen (bis zu 6, unterschiedliche Nähe) |
| Tippen untere Hälfte (nur Demo) | virtuellen Feind schalten: keiner → fern → mittel → nah |
| Langes Drücken (nur Demo) | alle virtuellen Freunde und Feinde entfernen |

Der **Demo-Modus** speist virtuelle Freunde in dieselbe Tabelle ein wie echte
Beacons. So lässt sich die Anzeige mit nur einem Gerät prüfen.

Die serielle Konsole gibt jede Sekunde eine Zeile aus:

```
friends=2 n_eff=1.35 c_max=0.78 S=0.34 V=0.62 beacons/s=8 display_writes=14
  id=0x1A2B3C4D rssi=-58 dBm rate=3.4/s
```

## Tuning

Alle Werte in `crates/amulett-core/src/params.rs` (siehe
[Freunde erkennen](freunde-erkennen.md#parameter)); Farbwerte:

| Parameter | Startwert |
|---|---:|
| `HUE_A_DEG` / `HUE_B_DEG` | 210 / 30 |
| `FULL_SATURATION_COUNT` | 4,0 |
| `V_MIN` / `V_MAX` | 0,08 / 1,0 |
| `RISE_PER_S` / `FALL_PER_S` | 0,5 / 0,25 |
| `DISPLAY_INTERVAL_MS` | 50 |

!!! note "Strom"
    Ein hell leuchtendes AMOLED verbraucht deutlich mehr als ein dunkles.
    `V_MAX` begrenzt die höchste Helligkeit; Messwerte folgen in der Studie
    zum [Stromverbrauch](stromverbrauch.md).

## Feldtest

1. Zwei Amulette einschalten und nebeneinander legen: Sättigung und Helligkeit
   steigen innerhalb weniger Sekunden.
2. Eines in 5 m, 10 m Abstand und durch eine Wand tragen; Konsole beobachten
   (RSSI, Beacon-Rate) und die Schwellen `RSSI_NEAR_DBM`/`RSSI_FAR_DBM`
   anpassen.
3. Mit drei Geräten prüfen, dass die Sättigung mit der Anzahl steigt.
4. Ergebnisse hier eintragen. *(Noch keine Messwerte, der Test auf dem Gerät
   steht aus.)*
