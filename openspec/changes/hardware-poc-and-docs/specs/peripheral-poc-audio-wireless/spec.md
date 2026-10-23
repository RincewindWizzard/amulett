# Spec Delta

## Purpose

Zeigt, wie Audio (Dual-Mikrofon, Codec, Lautsprecher) und Funk (WLAN, Bluetooth LE) auf dem Board in Betrieb genommen werden.

## ADDED Requirements

### Requirement: Lautsprecher spielt Ton
Die Firmware SHALL über Codec und Lautsprecher einen Testton ausgeben.

#### Scenario: Testton
- **WHEN** die Audio-Demo gestartet wird
- **THEN** ist ein Ton hörbar

### Requirement: Mikrofone liefern Pegel
Die Firmware SHALL beide Mikrofonkanäle aufnehmen und einen Pegel je Kanal anzeigen.

#### Scenario: Geräusch
- **WHEN** in der Nähe des Boards gesprochen oder geklatscht wird
- **THEN** steigt der angezeigte Pegel beider Kanäle deutlich über den Ruhepegel

### Requirement: WLAN-Scan
Die Firmware SHALL WLAN-Netze in der Umgebung scannen und die gefundenen SSIDs ausgeben.

#### Scenario: Netze vorhanden
- **WHEN** der Scan ausgeführt wird und Netze in Reichweite sind
- **THEN** werden SSID und Signalstärke ausgegeben

### Requirement: BLE-Advertising
Die Firmware SHALL sich per Bluetooth LE mit einem Gerätenamen bewerben.

#### Scenario: Gerät sichtbar
- **WHEN** die BLE-Demo läuft
- **THEN** ist das Board in einer BLE-Scanner-App sichtbar
