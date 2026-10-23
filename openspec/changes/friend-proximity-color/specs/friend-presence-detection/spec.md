# Spec Delta

## Purpose

Legt fest, wie ein Amulett andere Amulette in der Nähe per BLE erkennt und daraus Werte bildet, die beschreiben, wie viele Freunde wie nah sind.

## ADDED Requirements

### Requirement: Amulett sendet Beacons
Das Amulett SHALL dauerhaft einen BLE-Beacon senden, der es als Amulett kennzeichnet und eine Geräte-Kennung enthält.

#### Scenario: Beacon sichtbar
- **WHEN** das Amulett eingeschaltet ist
- **THEN** ist in einer BLE-Scanner-App ein Beacon mit Amulett-Kennzeichnung und Geräte-Kennung sichtbar

#### Scenario: Kein Dauerverbinden
- **WHEN** ein fremdes Gerät sich verbinden will
- **THEN** wird der Beacon nicht als verbindbar angeboten

### Requirement: Amulett scannt nach Beacons
Das Amulett SHALL gleichzeitig zum Senden nach Beacons anderer Amulette scannen und je Beacon Geräte-Kennung und Signalstärke (RSSI) erfassen.

#### Scenario: Freund in Reichweite
- **WHEN** ein zweites Amulett in Funkreichweite sendet
- **THEN** erfasst das erste Amulett dessen Geräte-Kennung und RSSI

#### Scenario: Fremde BLE-Geräte
- **WHEN** Beacons ohne Amulett-Kennzeichnung empfangen werden
- **THEN** werden sie ignoriert und beeinflussen die Freundesmaße nicht

### Requirement: Eigene Beacons werden ignoriert
Das Amulett SHALL Beacons mit der eigenen Geräte-Kennung nicht als Freund zählen.

#### Scenario: Eigenes Echo
- **WHEN** ein Beacon mit der eigenen Kennung empfangen wird
- **THEN** zählt er nicht für die Freundesmaße

### Requirement: Freunde laufen ab
Das Amulett SHALL einen Freund aus der Liste entfernen, wenn von ihm länger als eine konfigurierbare Zeitspanne kein Beacon empfangen wurde.

#### Scenario: Freund geht weg
- **WHEN** ein Freund nach der Ablaufzeit keinen Beacon mehr gesendet hat
- **THEN** wird er nicht mehr gezählt und sein Anteil an den Freundesmaßen entfällt

### Requirement: Signalstärke wird geglättet
Das Amulett SHALL die RSSI-Werte je Freund glätten, damit einzelne Ausreißer die Freundesmaße nicht sprunghaft verändern.

#### Scenario: Ausreißer
- **WHEN** ein einzelner Beacon eines nahen Freundes mit stark abweichender RSSI empfangen wird
- **THEN** ändern sich die Freundesmaße nur geringfügig

### Requirement: Freundesmaße
Das Amulett SHALL aus den erkannten Freunden zwei Werte berechnen: die gewichtete Freundesanzahl (Summe der Nähe aller Freunde) und die Nähe des nächsten Freundes (größte Nähe eines Freundes), wobei die Nähe je Freund zwischen 0 und 1 liegt.

#### Scenario: Kein Freund
- **WHEN** kein Freund erkannt wird
- **THEN** sind gewichtete Anzahl und Nähe des nächsten Freundes 0

#### Scenario: Mehr Freunde
- **WHEN** zu einem erkannten Freund ein weiterer mit gleicher Signalstärke hinzukommt
- **THEN** ist die gewichtete Anzahl größer als zuvor

#### Scenario: Näherer Freund
- **WHEN** sich ein Freund annähert (RSSI steigt)
- **THEN** steigt seine Nähe und damit die Nähe des nächsten Freundes

#### Scenario: Wertebereich
- **WHEN** beliebig viele Freunde sehr nah sind
- **THEN** bleibt die Nähe je Freund und die Nähe des nächsten Freundes höchstens 1

### Requirement: Parameter sind konfigurierbar
Das Amulett SHALL Advertising-Intervall, Scan-Fenster, Ablaufzeit, Glättung und die RSSI-Schwellen für "nah" und "fern" als Konstanten an einer Stelle bündeln.

#### Scenario: Tuning
- **WHEN** ein Parameter geändert und neu gebaut wird
- **THEN** wirkt die Änderung ohne weitere Codeänderungen
