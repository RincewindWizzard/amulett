# Spec Delta

## Purpose

Beschreibt, wie das Board in definierte Stromsparzustände versetzt wird, welche Ereignisse es aufwecken und wie BLE-Kommunikation mit minimalem Energiebedarf erfolgt.

## ADDED Requirements

### Requirement: Definierte Betriebszustände
Die Firmware SHALL die Zustände Aktiv, Modem-Sleep, Light Sleep und Deep Sleep einzeln und reproduzierbar einnehmen können.

#### Scenario: Zustand wählen
- **WHEN** über Demo-Menü oder Build-Konfiguration ein Zustand gewählt wird
- **THEN** wechselt das Gerät in diesen Zustand und meldet den Zustand vorher über die serielle Konsole

### Requirement: Aufwachquellen
Die Firmware SHALL das Gerät aus Light Sleep und Deep Sleep über einen Timer und über mindestens eine externe Quelle (Taste, Touch-Interrupt oder IMU-Interrupt) aufwecken können.

#### Scenario: Aufwachen per Timer
- **WHEN** das Gerät mit einem Timer von 10 Sekunden in Deep Sleep geht
- **THEN** startet es nach etwa 10 Sekunden neu und meldet "Aufwachgrund: Timer"

#### Scenario: Aufwachen per Taste
- **WHEN** im Deep Sleep eine konfigurierte Taste gedrückt wird
- **THEN** wacht das Gerät auf und meldet den Aufwachgrund

### Requirement: Zustand über Deep Sleep hinweg
Die Firmware SHALL Daten, die über Deep Sleep erhalten bleiben müssen, im RTC-Speicher ablegen.

#### Scenario: Zähler
- **WHEN** ein Zähler vor dem Deep Sleep erhöht wird
- **THEN** zeigt er nach dem Aufwachen den erhöhten Wert

### Requirement: Peripherie abschaltbar
Die Firmware SHALL vor dem Schlafen Display, Audio und nicht benötigte Sensoren in einen Zustand mit geringstmöglichem Verbrauch versetzen oder deren Versorgung abschalten.

#### Scenario: Display aus
- **WHEN** das Gerät in Light oder Deep Sleep geht
- **THEN** ist das Display dunkel und seine Versorgung ist, soweit auf dem Board möglich, abgeschaltet

#### Scenario: Nicht abschaltbare Verbraucher
- **WHEN** ein Baustein auf dem Board nicht abgeschaltet werden kann
- **THEN** ist dies in der Doku als Restverbrauch vermerkt

### Requirement: BLE im Niedrigenergiebetrieb
Die Firmware SHALL BLE-Advertising und BLE-Verbindungen mit konfigurierbaren Intervallen betreiben, wobei die CPU zwischen den Funkereignissen schläft.

#### Scenario: Advertising mit langem Intervall
- **WHEN** Advertising mit einem Intervall von 1000 ms läuft
- **THEN** ist das Gerät in einer BLE-App sichtbar und die CPU schläft zwischen den Advertising-Ereignissen

#### Scenario: Periodische Verbindung
- **WHEN** ein Zentralgerät verbunden ist
- **THEN** werden Connection-Intervall und Slave Latency konfiguriert und Daten können gesendet werden

### Requirement: Periodisches Aufwachen zum Senden
Die Firmware SHALL im Deep Sleep periodisch aufwachen, per BLE eine Nachricht senden und wieder schlafen gehen.

#### Scenario: Messzyklus
- **WHEN** der Zyklus mit einem Intervall von 60 s läuft
- **THEN** wird bei jedem Aufwachen eine BLE-Nachricht gesendet und das Gerät schläft danach wieder
