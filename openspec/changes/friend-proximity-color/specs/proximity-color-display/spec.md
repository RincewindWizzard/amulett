# Spec Delta

## Purpose

Beschreibt, wie das Display die Menge und Nähe der Freunde über Sättigung und Helligkeit einer Farbe darstellt.

## ADDED Requirements

### Requirement: Farbfläche als Anzeige
Das Amulett SHALL im Normalbetrieb die gesamte sichtbare Displayfläche mit einer Farbe füllen, deren Farbton fest ist und deren Sättigung und Helligkeit aus den Freundesmaßen berechnet werden.

#### Scenario: Normalbetrieb
- **WHEN** das Amulett läuft
- **THEN** zeigt das Display eine einfarbige Fläche in der berechneten Farbe

### Requirement: Sättigung folgt der Freundesanzahl
Das Amulett SHALL die Sättigung mit der gewichteten Freundesanzahl monoton steigern; ohne Freunde ist die Sättigung 0, ab der Volldarstellungsanzahl ist sie 1.

#### Scenario: Kein Freund
- **WHEN** kein Freund erkannt wird
- **THEN** ist die Fläche entsättigt (grau)

#### Scenario: Mehr Freunde
- **WHEN** zu einem erkannten Freund ein weiterer gleich naher hinzukommt
- **THEN** ist die Sättigung höher als zuvor

#### Scenario: Viele Freunde
- **WHEN** mindestens so viele Freunde nahe sind wie die Volldarstellungsanzahl
- **THEN** ist die Sättigung 1

### Requirement: Helligkeit folgt der Nähe des nächsten Freundes
Das Amulett SHALL die Helligkeit mit der Nähe des nächsten Freundes monoton steigern; ohne Freunde ist sie auf dem Mindestwert, bei höchster Nähe auf dem Höchstwert.

#### Scenario: Freund nähert sich
- **WHEN** sich ein Freund annähert
- **THEN** wird die Fläche heller

#### Scenario: Kein Freund
- **WHEN** kein Freund erkannt wird
- **THEN** ist die Fläche auf der Mindesthelligkeit (dunkel, aber nicht vollständig aus)

### Requirement: Weicher Farbübergang
Das Amulett SHALL Sättigung und Helligkeit mit begrenzter Änderungsgeschwindigkeit dem Zielwert nachführen, sodass kein sichtbarer Sprung entsteht.

#### Scenario: Plötzlicher Freund
- **WHEN** ein Freund plötzlich in unmittelbarer Nähe erscheint
- **THEN** erreicht die Anzeige die Zielfarbe erst nach der Übergangszeit und nicht in einem Bild

#### Scenario: Freund geht
- **WHEN** der letzte Freund verschwindet
- **THEN** blendet die Fläche langsam zu Grau und dunkel aus

### Requirement: Display nur bei Änderung beschreiben
Das Amulett SHALL das Display nur dann neu beschreiben, wenn sich die darzustellende Farbe um mehr als eine Stufe geändert hat.

#### Scenario: Ruhe
- **WHEN** sich die Zielfarbe nicht ändert
- **THEN** werden keine neuen Bilddaten gesendet

### Requirement: Debug-Ansicht
Das Amulett SHALL mit der BOOT-Taste eine Übersicht ein- und ausblenden, die Freunde mit Geräte-Kennung, geglättetem RSSI und Beacon-Rate sowie die beiden Freundesmaße zeigt.

#### Scenario: Einblenden
- **WHEN** die BOOT-Taste gedrückt wird
- **THEN** erscheint die Übersicht über der Farbfläche, ein weiterer Druck blendet sie aus

### Requirement: Simulierte Freunde
Das Amulett SHALL einen Demo-Modus anbieten, in dem virtuelle Freunde mit wählbarer Nähe hinzugefügt oder entfernt werden, damit die Anzeige mit nur einem Gerät prüfbar ist.

#### Scenario: Virtueller Freund
- **WHEN** im Demo-Modus ein virtueller Freund hinzugefügt wird
- **THEN** ändern sich Sättigung und Helligkeit wie bei einem echten Freund
