# Spec Delta

## Purpose

Zeigt, wie der QMI8658-Bewegungssensor ausgelesen und die Energieverwaltung (AXP2101) für Versorgungsschienen, Laden und Batteriestatus genutzt wird.

## ADDED Requirements

### Requirement: IMU liefert Messwerte
Die Firmware SHALL Beschleunigungs- und Gyroskopwerte des QMI8658 über I2C auslesen und darstellen.

#### Scenario: Board wird bewegt
- **WHEN** das Board gekippt oder geschüttelt wird
- **THEN** ändern sich die angezeigten Beschleunigungs- und Gyroskopwerte entsprechend

#### Scenario: Board liegt ruhig
- **WHEN** das Board flach und ruhig liegt
- **THEN** zeigt die Z-Achse etwa 1 g und das Gyroskop Werte nahe null

### Requirement: Versorgungsschienen werden geschaltet
Die Firmware SHALL über den AXP2101 die für Display und Peripherie benötigten Spannungsschienen aktivieren, bevor diese angesprochen werden.

#### Scenario: Start mit eingeschalteten Schienen
- **WHEN** die Firmware startet
- **THEN** werden die benötigten Schienen vor der Initialisierung von Display und Sensoren aktiviert

### Requirement: Batterie- und Ladestatus
Die Firmware SHALL Batteriespannung, Ladezustand und Ladestatus über den AXP2101 auslesen und anzeigen, sofern ein Akku angeschlossen ist.

#### Scenario: Akku angeschlossen
- **WHEN** ein Akku angeschlossen ist
- **THEN** werden Batteriespannung und Ladestatus angezeigt

#### Scenario: Kein Akku
- **WHEN** kein Akku angeschlossen ist
- **THEN** meldet die Firmware "kein Akku" statt fehlerhafter Werte

### Requirement: Weitere Bausteine werden erfasst
Die Firmware SHALL jeden weiteren auf dem Board vorhandenen Baustein, der über I2C erreichbar ist (z. B. RTC, IO-Expander), mindestens einmal initialisieren und einen Wert daraus ausgeben.

#### Scenario: I2C-Scan
- **WHEN** die Firmware den I2C-Bus scannt
- **THEN** werden alle gefundenen Adressen ausgegeben und den dokumentierten Bausteinen zugeordnet
