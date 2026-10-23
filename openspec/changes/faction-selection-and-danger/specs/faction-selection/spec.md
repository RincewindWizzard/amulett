# Spec Delta

## Purpose

Legt fest, wie der Träger beim Start seine Fraktion wählt, wie sie am Gerät angezeigt wird und wie andere Amulette sie per Beacon erfahren.

## ADDED Requirements

### Requirement: Auswahl beim Start
Das Amulett SHALL nach dem Einschalten einen Auswahlbildschirm für die Fraktion zeigen und erst nach Bestätigung mit dem Normalbetrieb beginnen.

#### Scenario: Start
- **WHEN** das Amulett eingeschaltet wird
- **THEN** zeigt das Display die Auswahl mit den Fraktionen A und B und eine vorausgewählte Fraktion

#### Scenario: Noch nicht bestätigt
- **WHEN** die Auswahl noch nicht bestätigt wurde
- **THEN** sendet das Amulett keinen Beacon und wertet keine Beacons aus

### Requirement: Bedienung mit den Tasten
Das Amulett SHALL die Auswahl mit der BOOT-Taste zwischen A und B wechseln und mit einem kurzen Druck auf die PWR-Taste bestätigen.

#### Scenario: Wechseln
- **WHEN** die BOOT-Taste gedrückt wird
- **THEN** wechselt die markierte Fraktion von A auf B oder von B auf A

#### Scenario: Bestätigen
- **WHEN** die PWR-Taste kurz gedrückt wird
- **THEN** wird die markierte Fraktion übernommen und der Normalbetrieb startet

### Requirement: Fraktion gut erkennbar
Das Amulett SHALL jede Fraktion durch Buchstabe und Farbe darstellen, wobei die Farben nicht Rot sind (Rot ist der Gefahr vorbehalten).

#### Scenario: Auswahlansicht
- **WHEN** die Auswahl angezeigt wird
- **THEN** sind A und B jeweils mit Buchstabe und eigener Farbe dargestellt und die markierte Fraktion ist hervorgehoben

### Requirement: Eigene Fraktion im Normalbetrieb sichtbar
Das Amulett SHALL im Normalbetrieb die eigene Fraktion dauerhaft am Bildschirm erkennbar machen, ohne die Farbfläche zu verdecken.

#### Scenario: Anzeige im Normalbetrieb
- **WHEN** die Farbfläche angezeigt wird
- **THEN** ist ein Fraktionszeichen (Buchstabe in Fraktionsfarbe) am Rand des Displays sichtbar

### Requirement: Fraktion im Beacon
Das Amulett SHALL im Beacon seine gewählte Fraktion übertragen.

#### Scenario: Beacon-Inhalt
- **WHEN** das Amulett mit Fraktion B im Normalbetrieb sendet
- **THEN** enthält der Beacon das Fraktionsfeld mit dem Wert für B

### Requirement: Neustart wählt neu
Das Amulett SHALL die Fraktion nach jedem Neustart erneut abfragen und sie nicht dauerhaft speichern.

#### Scenario: Neustart
- **WHEN** das Amulett aus- und wieder eingeschaltet wird
- **THEN** erscheint wieder die Auswahl
