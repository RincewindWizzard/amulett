# Design

## Context

Aufbauend auf `friend-proximity-color` (noch nicht umgesetzt): nicht verbindbarer Beacon mit Hersteller-Daten (Company-ID `0xFFFF`, Kennung `"AM"`, Version, 4 Byte Geräte-Kennung, Zähler), Scanner mit Freundestabelle (EMA-geglätteter RSSI je Gerät, Ablauf nach 10 s, Nähe `c` je Freund, Maße `n_eff` und `c_max`), Farbfläche mit Sättigung = Anzahl und Helligkeit = Nähe, reine Logik in `crates/amulett-core`. Eingaben im vorhandenen PoC: BOOT-Taste an GPIO0 (Low = gedrückt), PWR-Taste über TCA9554 P4 (High = gedrückt); ein langer PWR-Druck schaltet über den AXP2101 ab. Motivation: siehe proposal.md.

## Goals / Non-Goals

**Goals:**
- Zwei Fraktionen mit symmetrischer Feindschaft, Auswahl mit den Tasten, gut erkennbare Gefahr.
- Die Entscheidungslogik (Freund/Feind, Gefahr mit Hysterese) liegt in `amulett-core` und ist auf dem Rechner testbar.

**Non-Goals:**
- Fälschungssicherheit, Speichern der Wahl, mehr als zwei Fraktionen, Spielstand/Punkte.

## Decisions

- **Fraktionsfeld im Beacon:** Beacon-Version 2 fügt ein Byte `flags` hinzu: Bits 0–3 Fraktions-ID (`0` = A, `1` = B, übrige Werte reserviert), Bits 4–7 reserviert (0). Unbekannte Werte werden ignoriert (weder Freund noch Feind). Das Format trägt bis zu 16 Fraktionen, die Regel "andere bekannte Fraktion = Feind" gilt nur für zwei. Alternative: Fraktion in der Geräte-Kennung codieren: verworfen (unklar, nicht erweiterbar). Beacons mit älterer Version werden verworfen.
- **Beziehungsregel an einer Stelle:** `relation(own, other) -> Friend | Enemy | Ignore` in `amulett-core`. Für zwei Fraktionen: gleich = Freund, verschieden = Feind. Spätere Allianzen lassen sich dort ändern.
- **Freundestabelle führt Beziehung je Eintrag.** Die Freundesmaße `n_eff` und `c_max` rechnen nur über Freunde. Für Feinde gilt dieselbe Glättung und Nähe-Abbildung `c`, daraus wird die Gefährdung `g = max(c_feind)`.
- **Gefahr mit Hysterese:** Gefahr beginnt bei `g ≥ G_EIN` (Startwert 0,45 ≙ etwa −68 dBm bei den Standardschwellen −90/−50 dBm) und endet bei `g < G_AUS` (Startwert 0,30) oder sobald alle Feinde abgelaufen sind (10 s). Mindestens zwei Beacons des Feindes innerhalb von 3 s vor dem Auslösen, damit ein einzelner Ausreißer keinen Alarm auslöst.
- **Zustandsautomat:** `Auswahl → Normal ⇄ Gefahr`. In `Auswahl` sind Sender und Scanner aus. In `Normal` laufen Farbfläche (Farbton = Fraktionsfarbe) und Fraktionszeichen; in `Gefahr` übernimmt die Gefahrenanzeige den ganzen Bildschirm, die Freundesauswertung läuft im Hintergrund weiter.
- **Bedienung:** BOOT wechselt A/B, kurzer PWR-Druck bestätigt. Beide Tasten werden flankengetriggert (wie im PoC) mit 30 ms Entprellung. Langer PWR-Druck schaltet weiterhin ab (Hardware) und wird nicht abgefangen. Alternative Touch: vorerst nicht, die Aufgabe nennt Tasten.
- **Farben:** A = Blau (`Rgb565` 0x1E9F-ähnlich, Cyan-Blau), B = Orange/Gelb; beide mit Buchstaben, damit Farbenblindheit keine Rolle spielt. Rot gehört der Gefahr. Genaue Werte als Konstanten.
- **Gefahrenanzeige:** Vollbild, rotes Pulsieren (Helligkeit als Sinus, Periode von 1,2 s bei `G_EIN` bis 0,3 s bei `g = 1`), Text "GEFAHR" und eine Balkenanzeige für `g`. Frame-Streifen werden dafür nicht gebraucht, es genügt der Framebuffer. Kein Ton im Standard; ein optionaler Piepton nutzt den vorhandenen Testton-Pfad (`AudioIo`) und ist per Konstante abschaltbar.
- **Keine Persistenz:** Die Wahl lebt im RAM und wird bei jedem Start neu gefragt (so beschrieben).

## Risks / Trade-offs

- [Beacons sind mit jedem BLE-Gerät fälschbar, Fraktion lässt sich vortäuschen] → bewusst akzeptiert, in der Doku vermerkt; spätere Signatur mit geteiltem Schlüssel je Fraktion möglich.
- [RSSI schwankt, Feindnähe wird falsch eingeschätzt] → Glättung, Hysterese, Mindestanzahl Beacons; Schwellen im Feldtest anpassen.
- [Fehlalarm durch Feinde hinter Wänden / Nachbarn] → Einschaltwert lässt sich anheben; Feldtest bestimmt die Reichweite.
- [Verpasste Gefahr, wenn Scanner nur mit kleinem Tastverhältnis läuft] → Latenz der Erkennung hängt von Scan-Fenster und Advertising-Intervall ab; in der Doku nennen und im Feldtest messen.
- [Fraktion geht beim Strom-Aus verloren und man wählt versehentlich falsch] → bewusst gewünscht ("beim Start wählen"); sonst später Speicher im Flash.
- [PWR-Taste kann bei sehr langem Druck das Gerät ausschalten] → Hinweis auf dem Auswahlbildschirm: "PWR kurz".
- [Zwei Fraktionen fest verdrahtet, spätere Erweiterung ändert Regeln] → `relation()` und Format sind bereits allgemein gehalten.

## Open Questions

- Soll die Wahl optional gespeichert werden (z. B. langer BOOT-Druck zum Wechseln)? Annahme: nein, jedes Mal fragen.
- Gefahr nur sichtbar oder zusätzlich hörbar? Annahme: nur sichtbar, Ton per Konstante.
