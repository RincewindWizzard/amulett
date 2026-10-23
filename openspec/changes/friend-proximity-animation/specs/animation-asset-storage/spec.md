# Spec Delta

## Purpose

Legt fest, wie Animationsframes erzeugt, im Flash gespeichert und von der Firmware gelesen werden, innerhalb der Speichergrenzen des Boards.

## ADDED Requirements

### Requirement: Frame-Format
Die Animationsframes SHALL in einem dokumentierten Binärformat mit Kopf (Kennung, Version, Breite, Höhe, Frame-Anzahl, Pixelformat) und anschließenden Frame-Daten gespeichert sein.

#### Scenario: Gültiger Kopf
- **WHEN** die Firmware den Bestand öffnet
- **THEN** prüft sie Kennung und Version und liest Abmessungen und Frame-Anzahl aus dem Kopf

#### Scenario: Ungültiger Kopf
- **WHEN** Kennung oder Version nicht passen
- **THEN** lehnt die Firmware den Bestand ab und meldet den Fehler

### Requirement: Eigene Flash-Partition
Die Frames SHALL in einer eigenen Flash-Partition abgelegt sein, die unabhängig von der Firmware beschrieben werden kann.

#### Scenario: Nur Frames aktualisieren
- **WHEN** ausschließlich die Frame-Partition neu geschrieben wird
- **THEN** startet die Firmware unverändert und zeigt die neue Animation

### Requirement: Konverter
Ein Werkzeug auf dem Entwicklungsrechner SHALL eine Bildfolge oder GIF-Datei in das Frame-Format umwandeln.

#### Scenario: Umwandlung
- **WHEN** das Werkzeug mit einer Bildfolge und den Zielabmessungen aufgerufen wird
- **THEN** entsteht eine Binärdatei im dokumentierten Format und das Werkzeug meldet Frame-Anzahl und Größe

#### Scenario: Zu groß
- **WHEN** der Bestand größer als die Frame-Partition wäre
- **THEN** bricht das Werkzeug mit einer Fehlermeldung ab

### Requirement: Speicherbudget dokumentiert
Die Dokumentation SHALL die maximale Frame-Anzahl für die unterstützten Abmessungen und Pixelformate ausweisen.

#### Scenario: Budgettabelle
- **WHEN** die Speicherseite gelesen wird
- **THEN** zeigt sie für jede Variante Bytes pro Frame und maximale Frame-Anzahl in der Partition

### Requirement: Lesen ohne vollständiges Laden
Die Firmware SHALL ein Frame lesen können, ohne den gesamten Bestand im RAM zu halten.

#### Scenario: Großer Bestand
- **WHEN** der Bestand größer als der verfügbare RAM ist
- **THEN** werden Frames bei Bedarf aus dem Flash in den Framebuffer geladen

### Requirement: Image enthält die Frames
Die Release-Pipeline SHALL ein Flash-Image erzeugen, das Firmware und Frames enthält, sodass ein einzelner Flash-Vorgang genügt.

#### Scenario: Release
- **WHEN** ein Tag `v*` gebaut wird
- **THEN** enthält das veröffentlichte Image die Frame-Partition mit der Standard-Animation
