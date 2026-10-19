# Policy: Issue-Automation-Abgrenzung

## 1. Übersicht der Workflows mit Issue-Erstellung

| Workflow | Zweck | Trigger-Bedingung | Dedup-Mechanismus | Aktueller Status |
| :--- | :--- | :--- | :--- | :--- |
| `.github/workflows/scheduled-audit.yml` | Routineauftrag | Schedule (`0 22 * * 5`) / `workflow_dispatch` | Task-Datei Prüf-Header (`pending-audit-task.md`) <!-- doc-ref-ignore --> | Deaktiviert (PR-basiert) |
| `.github/workflows/post-merge-verification.yml` | Fehler-Eskalation | Push auf `main` | Keiner | Deaktiviert |
| `.github/workflows/mutation-testing.yml` | Fehler-Eskalation | `workflow_dispatch` / PR (`event_name !== 'pull_request'`) | Keiner | Deaktiviert |
| `.github/workflows/chaos.yml` | Fehler-Eskalation | Schedule (`0 2 * * *`) / PR (`event_name === 'schedule'`) | Keiner | Deaktiviert |
| `.github/workflows/tsan.yml` | Fehler-Eskalation | Schedule / PR / Dispatch | Keiner | Deaktiviert |

## 2. Abgrenzungsregel für automatisierte und manuelle Issues (ADR-N30)

> **Manuelle Issues:** Vom Menschen angelegte Issues über die vordefinierten Issue-Formulare (`.github/ISSUE_TEMPLATE/*.yml`) sind ausdrücklich erwünscht und dienen als primäre Eingabe für LLM-Agenten (Jules).
>
> **Automatisierte Issues:** Automatisierte Issue-Erstellung durch Workflows/Gates/PRs ist grundsätzlich **DEAKTIVIERT**, um Spam im Repository-Issue-Tracker zu verhindern.
> **Ausnahme:** Die einzige zulässige automatische Issue-Erzeugung durch Workflows ist eine deduplizierte Meldung bei einem fehlerhaften `main`-Branch ("main rot"). Die Deduplizierung MUSS über einen festen Titel-Präfix (z. B. `[MAIN ROT]`) und einen eindeutigen HTML-Kommentar-Marker im Body erfolgen.

### Begründung
* **Manuell erstellte Tasks:** Bieten volle Kontrolle und Transparenz über geplante Aufgaben im GitHub-UI.
* **Fehler-Eskalationen ("main rot"):** Indizieren akute Störungen im Hauptzweig und benötigen sofortige Sichtbarkeit.
* **Proaktive Routineaufträge ohne Fehlersignal:** Erzeugen unnötigen Noise im Issue-Tracker und müssen stattdessen Datei- oder PR-basiert abgewickelt werden.

## 3. Governance für neue Workflows

Jeder **NEUE** GitHub Actions Workflow, der vorhat, `github.rest.issues.create` oder vergleichbare Mechanismen zur automatisierten Issue-Erstellung einzusetzen, MUSS diese Policy vorab lesen.

* Automatische Issues sind ausnahmslos auf deduplizierte "main rot"-Meldungen beschränkt.
* PRs, die proaktive Routineaufträge ohne Fehlersignal über GitHub-Issues automatisieren wollen, sind abzulehnen.
