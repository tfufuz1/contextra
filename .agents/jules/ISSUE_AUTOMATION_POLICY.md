# Richtlinie: Automatisierte Issues

Nicht normativ. Die Richtlinie beschreibt, wann Workflows Issues anlegen dürfen. Die Angaben zum Ist-Zustand stammen aus der Vorfassung und sind vor einer Änderung im Repository zu prüfen.

## Grundsatz

Nach der Vorfassung ist die automatische Erstellung von Issues durch Workflows, Gates und PRs deaktiviert. Der Grund ist, dass sonst Rauschen im Issue-Tracker entsteht. Fehler melden Workflows im Status der GitHub-Actions-Läufe.

## Workflows mit Bezug zur Richtlinie

| Workflow (in `.github/workflows/`) | Zweck | Stand laut Vorfassung |
|---|---|---|
| `scheduled-audit.yml` | wöchentlicher Routineauftrag, Freitag 22:00 UTC | PR-basiert, keine Issues |
| `chaos.yml` | nächtlicher Chaos-Lauf | keine Issues |
| `tsan.yml` | Thread-Sanitizer | keine Issues |

Die Vorfassung nannte außerdem `mutation-testing.yml` und `post-merge-verification.yml`. Diese Dateien existieren im Repository nicht; ihre Einträge entfallen.

## Regeln für neue Workflows

- Ein neuer Workflow, der Issues anlegt, liest diese Richtlinie vorab.
- Die Einordnung als Fehler-Eskalation oder als Routineauftrag wird im PR-Text begründet.
- Routineaufträge ohne echtes Fehlersignal werden über PR oder Datei übergeben, nicht über Issues.
- Ein Workflow darf keine Sicherheitsgates mit `continue-on-error` durchwinken (SPEC 17.3).
