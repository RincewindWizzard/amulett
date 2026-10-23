# Spec Delta

## Purpose

Legt fest, wie das Amulett Freunde und Feinde unterscheidet, welche Amulette in die Freundesmaße eingehen und wann und wie es vor Feinden warnt.

## ADDED Requirements

### Requirement: Freund oder Feind
Das Amulett SHALL ein empfangenes Amulett als Freund einstufen, wenn dessen Fraktion der eigenen entspricht, und als Feind, wenn sie der anderen bekannten Fraktion entspricht.

#### Scenario: Gleiche Fraktion
- **WHEN** ein Beacon mit der eigenen Fraktion empfangen wird
- **THEN** wird der Sender als Freund geführt

#### Scenario: Andere Fraktion
- **WHEN** ein Beacon der anderen Fraktion empfangen wird
- **THEN** wird der Sender als Feind geführt

#### Scenario: Unbekannte Fraktion
- **WHEN** ein Beacon mit unbekanntem Fraktionswert empfangen wird
- **THEN** wird er weder als Freund noch als Feind gezählt

### Requirement: Freundesmaße zählen nur Freunde
Das Amulett SHALL bei der Berechnung der Freundesmaße und der Farbfläche ausschließlich Freunde berücksichtigen.

#### Scenario: Nur Feinde in der Nähe
- **WHEN** nur Feinde, aber keine Freunde empfangen werden
- **THEN** bleiben die Freundesmaße 0 und die Fläche bleibt grau und dunkel

### Requirement: Gefahr bei Feind in der Nähe
Das Amulett SHALL eine Gefahrenanzeige starten, wenn die geglättete Nähe eines Feindes den Einschaltwert erreicht.

#### Scenario: Feind nähert sich
- **WHEN** ein Feind so nah kommt, dass seine geglättete Nähe den Einschaltwert erreicht
- **THEN** wechselt das Display zur Gefahrenanzeige

#### Scenario: Feind weit weg
- **WHEN** ein Feind nur schwach empfangen wird
- **THEN** erscheint keine Gefahrenanzeige

### Requirement: Gefahrenanzeige
Das Amulett SHALL bei Gefahr den Bildschirm rot und pulsierend mit dem Text "GEFAHR" darstellen, wobei Pulsgeschwindigkeit und Helligkeit mit der Nähe des nächsten Feindes zunehmen.

#### Scenario: Näherer Feind
- **WHEN** sich ein Feind bei laufender Gefahrenanzeige weiter annähert
- **THEN** pulsiert die Anzeige schneller als zuvor

### Requirement: Gefahr hat Vorrang
Das Amulett SHALL die Gefahrenanzeige über der Farbfläche darstellen.

#### Scenario: Gleichzeitig Freund und Feind
- **WHEN** ein Freund und ein Feind in der Nähe sind
- **THEN** zeigt das Display die Gefahrenanzeige; der Freund wird im Hintergrund weiter erfasst

### Requirement: Gefahr endet mit Hysterese
Das Amulett SHALL die Gefahrenanzeige erst beenden, wenn die geglättete Nähe aller Feinde unter den Ausschaltwert gefallen ist oder kein Beacon eines Feindes mehr innerhalb der Ablaufzeit empfangen wurde; der Ausschaltwert SHALL kleiner sein als der Einschaltwert.

#### Scenario: Feind geht weg
- **WHEN** nach der Ablaufzeit kein Beacon des Feindes mehr eintrifft
- **THEN** endet die Gefahrenanzeige und die Farbfläche wird wieder angezeigt

#### Scenario: Schwankendes Signal
- **WHEN** die Nähe eines Feindes knapp um den Einschaltwert schwankt
- **THEN** flackert die Gefahrenanzeige nicht

### Requirement: Eigene Beacons lösen keine Gefahr aus
Das Amulett SHALL Beacons mit der eigenen Geräte-Kennung nicht als Feind werten.

#### Scenario: Echo
- **WHEN** ein Beacon mit der eigenen Kennung empfangen wird
- **THEN** wird keine Gefahr ausgelöst

### Requirement: Warnton ist optional
Das Amulett SHALL einen Warnton nur ausgeben, wenn er in den Parametern eingeschaltet ist; standardmäßig ist er aus.

#### Scenario: Standard
- **WHEN** Gefahr eintritt und der Ton nicht eingeschaltet ist
- **THEN** bleibt der Lautsprecher still

### Requirement: Schwellen konfigurierbar
Das Amulett SHALL Einschaltwert, Ausschaltwert, Ablaufzeit und Pulsgeschwindigkeiten der Gefahr an der zentralen Stelle der Parameter bündeln.

#### Scenario: Tuning
- **WHEN** ein Schwellwert geändert und neu gebaut wird
- **THEN** wirkt die Änderung ohne weitere Codeänderungen
