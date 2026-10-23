# Spec Delta

## Purpose

Stellt nachvollziehbare Verbrauchswerte für das Board bereit, damit die Akkulaufzeit abgeschätzt und Entwurfsentscheidungen belegt werden können.

## ADDED Requirements

### Requirement: Verbrauchstabelle
Die Dokumentation SHALL den Stromverbrauch je Betriebszustand und je Komponente tabellarisch auflisten, mit Einheit, Versorgungsspannung und Quelle der Zahl.

#### Scenario: Tabelle prüfen
- **WHEN** die Verbrauchsseite gelesen wird
- **THEN** hat jede Zeile einen Wert, eine Einheit, die Betriebsbedingungen und entweder einen Datenblatt-Link oder einen Verweis auf eine Messung

### Requirement: Datenblattwerte und Messwerte getrennt
Die Dokumentation SHALL Datenblattwerte und eigene Messwerte getrennt ausweisen und Abweichungen kommentieren.

#### Scenario: Abweichung
- **WHEN** ein gemessener Wert mehr als das Doppelte des Datenblattwerts beträgt
- **THEN** nennt die Doku die vermutete Ursache (z. B. Verbraucher auf dem Board)

### Requirement: Messaufbau dokumentiert
Die Dokumentation SHALL den Messaufbau beschreiben: Messgerät, Messpunkt, Versorgung (USB oder Akku), Firmware-Version und Umgebungsbedingungen.

#### Scenario: Messung wiederholen
- **WHEN** eine andere Person die Beschreibung befolgt
- **THEN** kann sie die Messung ohne weitere Angaben wiederholen

### Requirement: Verbrauch einzelner Bausteine
Die Dokumentation SHALL Display (an, gedimmt, aus), Audio, IMU, Touch, den Energie-Chip und den Funk (WLAN, BLE Advertising, BLE verbunden) einzeln ausweisen.

#### Scenario: Vollständigkeit
- **WHEN** die Tabelle geprüft wird
- **THEN** ist jeder genannte Baustein in mindestens einem Zustand enthalten

### Requirement: Akkulaufzeit-Abschätzung
Die Dokumentation SHALL für typische BLE-Nutzungsprofile eine Akkulaufzeit anhand der gemessenen Werte und einer genannten Akkukapazität berechnen.

#### Scenario: Profil
- **WHEN** ein Profil "alle 60 s Aufwachen und BLE senden" betrachtet wird
- **THEN** nennt die Doku den mittleren Strom und die Laufzeit samt Rechenweg

### Requirement: Empfehlungen
Die Dokumentation SHALL aus den Messwerten konkrete Empfehlungen für einen stromsparenden Betrieb ableiten.

#### Scenario: Empfehlungsliste
- **WHEN** die Seite zu Empfehlungen gelesen wird
- **THEN** enthält sie priorisierte Maßnahmen mit dem erwarteten Einsparpotenzial aus den Messungen
