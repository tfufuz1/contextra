# Scheduled Proactive Audit & Mutation Testing Rotation

This document describes the automated weekly proactive audit pipeline and mutation testing rotation configured in `.github/workflows/scheduled-audit.yml`.

---

## 1. Setup & Requirements

*(Hinweis: Das Label `jules-audit` ist historisch und wird im automatisierten Workflow nicht mehr aktiv verwendet, da Audit-Aufträge nun als Repository-Artefakt via PR übermittelt werden. Die PR-Erstellung erfolgt über die GitHub Action `peter-evans/create-pull-request@v6`.)*

---

## 2. Funktionsweise des Workflows (`.github/workflows/scheduled-audit.yml`)

Der Workflow wird automatisch jeden **Freitag um 22:00 UTC** (`0 22 * * 5`) sowie manuell via `workflow_dispatch` ausgeführt.

Er besteht aus zwei unabhängigen Jobs:

### Job 1: `prepare-audit-context`
- Sammelt alle Commit-Logs der letzten 7 Tage (`git log --since="7 days ago" --oneline --no-merges`).
- Prüft vor Erstellung, ob bereits eine unbearbeitete `.jules/pending-audit-task.md` existiert:
  - Falls existiert und nicht mit `STATUS: DONE` markiert ist, wird eine Workflow-Log-Warnung (`::warning::`) ausgegeben und der neue Auftrag wird als weiterer Abschnitt angehängt (kein Fehlschlag des Jobs, verhindert Datenverlust).
  - Falls nicht existiert oder mit `STATUS: DONE` markiert ist, wird die Datei mit dem Header (`STATUS: OPEN`, `CREATED: YYYY-MM-DD`) neu erstellt.
- Öffnet automatisch einen Pull Request gegen `main` auf dem Branch `automated/weekly-audit-task-<Datum>` via `peter-evans/create-pull-request@v6`.
- Der Auftragstext verweist strikt auf `.jules/AUDIT_INTAKE_PROTOCOL.md` und fordert die Prüfung auf:
  1. Race Conditions / TOCTOU-Fehler
  2. Neue `.unwrap()` / `.expect()` außerhalb der Baseline (`.unwrap-baseline.json`)
  3. Silent-Failure-Pattern (`let _ = ...` bei I/O-Operationen)
  4. DAG-Grenzverletzungen in `Cargo.toml`-Dependencies

### Job 2: `trigger-mutation-testing`
- Berechnet die ISO-Kalenderwoche (`KW % 4`).
- Rotiert wöchentlich durch die 4 Fokus-Crates:
  - **Woche 0**: `memfuse-graph`
  - **Woche 1**: `memfuse-vector`
  - **Woche 2**: `memfuse-agent`
  - **Woche 3**: `memfuse-db`
- Löst für den berechneten Fokus-Crate den `mutation-testing.yml`-Workflow via `workflow_dispatch` aus.

---

## 3. Workflow für die Bearbeitung (Artefakt-Handoff)

Statt manueller Issue-Zuweisung wird der Audit-Auftrag direkt als Artefakt im Repository verarbeitet:
1. Der automatische PR mit Branch `automated/weekly-audit-task-<Datum>` wird gemergt.
2. Eine neue Jules-Session prüft beim Empfehlungsschritt in Phase 1 von `.jules/SESSION_BOOTSTRAP.md`, ob `.jules/pending-audit-task.md` mit `STATUS: OPEN` existiert.
3. Jules arbeitet den Auftrag gemäß `.jules/AUDIT_INTAKE_PROTOCOL.md` ab.
4. Nach Abschluss (oder wenn kein Fund verifiziert werden kann) wird der Header der Datei `.jules/pending-audit-task.md` auf `STATUS: DONE` gesetzt.

---

## Audit-Workflow v2 (nach Review 2026-09)

Siehe `.jules/AUDIT_INTAKE_PROTOCOL.md` Abschnitt "🔒 Verdict-Unabhängigkeitspflicht (Audit-Workflow v2)" — dort kanonisch definiert.
