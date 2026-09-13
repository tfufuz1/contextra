# Policy: Issue-Automation-Abgrenzung

## 1. Übersicht der Workflows mit Issue-Erstellung

| Workflow | Zweck | Trigger-Bedingung | Dedup-Mechanismus | Aktueller Status |
| :--- | :--- | :--- | :--- | :--- |
| `.github/workflows/scheduled-audit.yml` | Routineauftrag | Schedule (`0 22 * * 5`) / `workflow_dispatch` | Task-Datei Prüf-Header (`pending-audit-task.md`) | Nutzt kein Issue mehr (PR-basiert) |
| `.github/workflows/post-merge-verification.yml` | Fehler-Eskalation | Push auf `main` | API-Suche nach offenen Issues + Commit-SHA | Unverändert (`github.rest.issues.create`) |
| `.github/workflows/mutation-testing.yml` | Fehler-Eskalation | `workflow_dispatch` / PR (`event_name !== 'pull_request'`) | Keiner | Unverändert (`github.rest.issues.create`) |
| `.github/workflows/chaos.yml` | Fehler-Eskalation | Schedule (`0 2 * * *`) / PR (`event_name === 'schedule'`) | Keiner | Unverändert (`github.rest.issues.create`) |

## 2. Abgrenzungsregel für automatisierte Issues

> **Grundsatz:** Automatisierte Issue-Erstellung ist **NUR** für Fehler-Eskalation bei bereits eingetretenem Testversagen zulässig (Regression, Chaos-Test-Fail, Mutation-Test-Fail), **NICHT** für proaktive, turnusmäßige Arbeitsaufträge ohne vorheriges Fehlersignal.

### Begründung
* **Fehler-Eskalationen:** Sind zeitkritisch, indizieren akute Störungen im Hauptzweig oder in Integrationsläufen und benötigen sofortige menschliche und agentische Sichtbarkeit im Issue-Tracker.
* **Proaktive Routineaufträge:** Erzeugen ohne echtes Fehlersignal unnötigen Noise im Issue-Tracker. Turnusmäßige Aufträge müssen stattdessen über Datei- und PR-basierte Handoffs (z. B. `.jules/pending-audit-task.md` via Pull Request) abgewickelt werden.

## 3. Governance für neue Workflows

Jeder **NEUE** GitHub Actions Workflow, der vorhat, `github.rest.issues.create` oder vergleichbare Mechanismen zur automatisierten Issue-Erstellung einzusetzen, MUSS diese Policy vorab lesen.

* Die Einordnung (Fehler-Eskalation vs. proaktiver Routineauftrag) MUSS im PR-Body des einreichenden Pull Requests explizit begründet werden.
* PRs, die Routineaufträge ohne Fehlersignal über GitHub-Issues automatisieren wollen, sind abzulehnen und auf PR-/Datei-basierte Mechanismen umzustellen.
