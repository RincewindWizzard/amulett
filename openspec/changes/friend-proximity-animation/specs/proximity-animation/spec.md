# Spec Delta

## Purpose

Beschreibt, wie das Display eine Frame-Animation zeigt, deren Fortschritt vom Nähe-Wert der Freunde abhängt.

## ADDED Requirements

### Requirement: Animation folgt dem Nähe-Wert
Das Amulett SHALL eine Frame-Animation anzeigen, bei der ein höherer Nähe-Wert die Animation Richtung letztem Frame bewegt und ein niedrigerer Richtung erstem Frame.

#### Scenario: Freunde kommen
- **WHEN** mehrere Freunde in die Nähe kommen
- **THEN** schreitet die Animation voran, bis ihr Fortschritt dem Nähe-Wert entspricht

#### Scenario: Freunde gehen
- **WHEN** alle Freunde außer Reichweite sind
- **THEN** läuft die Animation zurück bis zum ersten Frame und bleibt dort stehen

### Requirement: Fortschritt ändert sich gleichmäßig
Das Amulett SHALL den Animationsfortschritt mit begrenzter Geschwindigkeit dem Zielwert nachführen, sodass keine Frames übersprungen werden und die Bewegung flüssig wirkt.

#### Scenario: Sprunghafter Zielwert
- **WHEN** sich der Nähe-Wert von 0 auf 1 ändert
- **THEN** werden die Frames nacheinander gezeigt und nicht in einem Schritt übersprungen

### Requirement: Genügende Bildrate
Das Amulett SHALL die Animation mit mindestens 15 Bildern pro Sekunde ausgeben, solange sich der Fortschritt ändert.

#### Scenario: Laufende Animation
- **WHEN** die Animation vorwärts oder rückwärts läuft
- **THEN** werden pro Sekunde mindestens 15 Bilder auf das Display geschrieben

### Requirement: Ruhezustand ohne Änderung
Das Amulett SHALL das Display nicht neu schreiben, solange sich der Fortschritt nicht ändert.

#### Scenario: Stillstand
- **WHEN** der Fortschritt unverändert bleibt
- **THEN** werden keine neuen Bilddaten an das Display gesendet

### Requirement: Frames kommen aus dem Speicher
Das Amulett SHALL jedes Frame aus dem abgelegten Animationsbestand lesen und unverändert auf dem Display darstellen.

#### Scenario: Frame-Index
- **WHEN** der Fortschritt einem Frame-Index entspricht
- **THEN** zeigt das Display exakt dieses Frame

### Requirement: Fehlende Animation
Das Amulett SHALL bei fehlendem oder ungültigem Animationsbestand eine Fehlermeldung auf dem Display und der seriellen Konsole ausgeben statt abzustürzen.

#### Scenario: Leere Partition
- **WHEN** die Frame-Partition keinen gültigen Kopf enthält
- **THEN** zeigt das Display eine Meldung und die Firmware läuft weiter
