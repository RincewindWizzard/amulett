# Fraktionen und Gefahr

Es gibt zwei Fraktionen, **A** und **B**. Amulette derselben Fraktion sind
Freunde, Amulette der anderen Fraktion Feinde. Das Amulett zeigt Freunde als
[Farbe](farbanzeige.md) und warnt vor Feinden mit einer roten Gefahrenanzeige.

## Fraktion wählen

Nach jedem Einschalten erscheint die Auswahl. Bis zur Bestätigung sendet und
scannt das Amulett nicht.

| Taste | Wirkung |
|---|---|
| BOOT | zwischen A und B wechseln |
| PWR (kurz) | Auswahl bestätigen |

Die Auswahl wird **nicht gespeichert**, nach jedem Start wird neu gewählt. Ein
langer Druck auf PWR schaltet das Gerät aus (das erledigt der AXP2101).

```rust
--8<-- "src/product.rs:select"
```

Fraktion A ist blau, B orange; Rot ist der Gefahr vorbehalten. Zusätzlich
steht der Buchstabe der Fraktion in einer dunklen Scheibe am oberen Rand der
Farbfläche, damit er auch bei grauer Fläche lesbar bleibt.

## Freund oder Feind

Die Fraktion steht im Beacon (Bits 0–3 des Flags-Bytes, siehe
[Freunde erkennen](freunde-erkennen.md#beacon)). Die Entscheidung trifft eine
einzige Funktion:

```rust
--8<-- "crates/amulett-core/src/faction.rs:relation"
```

| Beacon | Beziehung | Wirkung |
|---|---|---|
| gleiche Fraktion | Freund | geht in Sättigung und Helligkeit ein |
| andere Fraktion | Feind | kann Gefahr auslösen, ändert die Farbfläche nicht |
| unbekannte Fraktions-ID | – | wird verworfen |
| eigene Geräte-Kennung | – | wird verworfen |

## Gefahr

Pro Feind gilt dieselbe RSSI-Glättung und Nähe wie bei Freunden. Ein Feind
zählt erst, wenn er **bestätigt** ist: zwei seiner Beacons innerhalb von 3 s.
Ein einzelner Ausreißer löst keinen Alarm aus.

Die Nähe des nächsten bestätigten Feindes steuert den Alarm mit zwei
Schwellen (Hysterese):

| Schwelle | Startwert | Bedeutung |
|---|---:|---|
| `DANGER_ON` | 0,45 (≈ −68 dBm) | Alarm beginnt |
| `DANGER_OFF` | 0,30 (≈ −78 dBm) | Alarm endet |

Der Alarm endet außerdem, wenn 3 s lang kein Feind-Beacon mehr kam
(Bestätigung verfällt) oder der Feind nach 10 s abläuft.

```rust
--8<-- "crates/amulett-core/src/danger.rs:update"
```

### Anzeige

Der ganze Bildschirm pulsiert rot mit dem Text **GEFAHR** und einem Balken für
die Nähe des Feindes. Je näher der Feind, desto schneller das Pulsieren
(1,2 s Periode bei `DANGER_ON`, 0,3 s bei höchster Nähe). Die Gefahr hat
Vorrang vor der Farbfläche; Freunde werden im Hintergrund weiter erfasst.

Ein Warnton ist vorbereitet, aber ausgeschaltet (`WARN_TONE = false` in den
Parametern). Eingeschaltet spielt das Amulett einen Dauerton über den
Lautsprecher, solange die Gefahr anhält; dafür werden Codec und I2S
initialisiert.

## Demo-Modus

PWR (kurz) schaltet im laufenden Betrieb den Demo-Modus ein oder aus. Tippen
auf die **obere** Displayhälfte fügt einen virtuellen Freund hinzu, Tippen auf
die **untere** Hälfte schaltet einen virtuellen Feind durch (keiner, fern,
mittel, nah). Langes Drücken entfernt alles. So lassen sich Farbe und Gefahr mit
nur einem Gerät prüfen.

## Grenzen

- **Beacons sind fälschbar.** Jedes BLE-Gerät kann sich als Fraktion A oder B
  ausgeben; es gibt keine Signatur. Das ist bewusst offen gelassen.
- **Erkennungsdauer.** Bis ein Feind bestätigt ist, vergehen mindestens zwei
  Beacons plus Scan-Lücken (Advertising-Intervall 300 ms, Scan 30 von 100 ms).
- **RSSI schwankt.** Die Schwellen sind Startwerte und müssen im Feldtest mit
  mehreren Geräten angepasst werden.
- **Noch nicht auf dem Gerät erprobt.**

## Parameter

Alle Werte stehen in `crates/amulett-core/src/params.rs`: `DANGER_ON`,
`DANGER_OFF`, `CONFIRM_WINDOW_MS`, `PULSE_SLOW_MS`, `PULSE_FAST_MS`,
`PULSE_MIN`, `WARN_TONE`.
