# Spec Delta

## Purpose

Stellt eine gehostete MkDocs-Dokumentation bereit, die jede Peripherie erklärt und auf die passende Herstellerdokumentation verweist.

## ADDED Requirements

### Requirement: Seite je Peripherie
Die Dokumentation SHALL für jede Peripherie eine eigene Seite enthalten mit Funktionsweise, Pinbelegung, Codebeispiel, bekannten Stolpersteinen und einem Link auf die passende Seite der Herstellerdokumentation.

#### Scenario: Seite vollständig
- **WHEN** eine Peripherieseite geöffnet wird
- **THEN** enthält sie alle genannten Abschnitte und der Herstellerlink ist erreichbar

### Requirement: Doku ist lokal baubar
Die Dokumentation SHALL sich mit MkDocs ohne Fehler und Warnungen bauen lassen.

#### Scenario: Strikter Build
- **WHEN** `mkdocs build --strict` ausgeführt wird
- **THEN** endet der Build erfolgreich

### Requirement: Automatische Veröffentlichung
Eine GitHub Action SHALL die Dokumentation bei Änderungen auf `main` bauen und auf GitHub Pages veröffentlichen.

#### Scenario: Push auf main
- **WHEN** ein Commit mit Änderungen an der Doku auf `main` landet
- **THEN** wird die Doku gebaut und die Pages-Seite aktualisiert

#### Scenario: Pull Request
- **WHEN** ein Pull Request die Doku ändert
- **THEN** wird die Doku gebaut, aber nicht veröffentlicht

### Requirement: README verlinkt die Doku
Das README SHALL einen Link auf die gehostete Dokumentation enthalten.

#### Scenario: Link vorhanden
- **WHEN** das README gelesen wird
- **THEN** führt der Link zur veröffentlichten Doku-URL
