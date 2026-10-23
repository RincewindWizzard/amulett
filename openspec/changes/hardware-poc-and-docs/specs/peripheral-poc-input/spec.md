# Spec Delta

## Purpose

Zeigt, wie Benutzereingaben über den kapazitiven CST9217-Touchscreen und die Seitentasten PWR und BOOT erkannt werden.

## ADDED Requirements

### Requirement: Touch-Berührungen werden erkannt
Die Firmware SHALL Berührungen des Touchscreens erkennen und die Koordinaten der Berührung ermitteln.

#### Scenario: Berührung
- **WHEN** der Finger den Touchscreen berührt
- **THEN** werden die Koordinaten im Bereich 0–465 pro Achse ausgegeben und auf dem Display markiert

#### Scenario: Loslassen
- **WHEN** der Finger den Touchscreen verlässt
- **THEN** meldet die Firmware das Ende der Berührung

### Requirement: Tasten werden erkannt
Die Firmware SHALL Drücken und Loslassen der Tasten PWR und BOOT einzeln erkennen und anzeigen.

#### Scenario: Tastendruck
- **WHEN** eine der beiden Tasten gedrückt wird
- **THEN** zeigt die Firmware an, welche Taste gedrückt ist, und meldet sie über die serielle Konsole

### Requirement: Eingabe ohne Aktives Warten
Die Firmware SHALL Touch- und Tastenereignisse über Interrupts oder Ereignisse erfassen, statt die Eingabe in einer Dauerschleife zu blockieren.

#### Scenario: Gleichzeitige Demos
- **WHEN** Touch-Eingabe und IMU-Anzeige gleichzeitig aktiv sind
- **THEN** werden keine Touch-Ereignisse verpasst
