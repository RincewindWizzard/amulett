# Spec Delta

## Purpose

Zeigt, wie das CO5300-AMOLED-Display des Boards initialisiert und angesteuert wird, damit die Firmware sichtbar Inhalte auf dem runden Display ausgeben kann.

## ADDED Requirements

### Requirement: Display wird initialisiert
Die Firmware SHALL das CO5300-Display über QSPI nach dem Einschalten der Versorgung initialisieren und in einen darstellungsbereiten Zustand versetzen.

#### Scenario: Erfolgreicher Start
- **WHEN** das Board startet und die Stromversorgung des Displays aktiv ist
- **THEN** wird das Display ohne Fehler initialisiert und die Initialisierung wird über die serielle Konsole gemeldet

#### Scenario: Initialisierung schlägt fehl
- **WHEN** die Display-Initialisierung einen Fehler liefert
- **THEN** meldet die Firmware den Fehler über die serielle Konsole und die übrigen Demos laufen weiter

### Requirement: Display zeigt Testbild
Die Firmware SHALL nach der Initialisierung ein Testbild im vollen Bereich von 466×466 Pixeln anzeigen, das Farben und die runde Form des Displays erkennbar macht.

#### Scenario: Testbild sichtbar
- **WHEN** die Display-Initialisierung erfolgreich war
- **THEN** zeigt das Display ein Testbild mit mindestens den Farben Rot, Grün und Blau sowie einer Randmarkierung

### Requirement: Helligkeit ist einstellbar
Die Firmware SHALL die Display-Helligkeit zur Laufzeit ändern können.

#### Scenario: Helligkeit ändern
- **WHEN** die Demo die Helligkeit von niedrig auf hoch stellt
- **THEN** ist die Änderung am Display sichtbar
