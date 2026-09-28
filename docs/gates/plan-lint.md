# Gate: Plan Lint (`plan-lint`)

## Zweck
Prüft den von Jules vorgeschlagenen Plan vor der Freigabe gegen die Regeln der Task-Karte.

## Befehle
`cargo xtask plan-lint --plan-file <JSON|TXT> --card <DATEI> [--approve]`
`cargo xtask plan-lint --session <SESSION_ID> --card <DATEI> [--approve]`

## Invarianten
- Alle im Plan genannten Pfade müssen im `scope` der Karte liegen.
- Kein geplanter Zugriff auf `forbidden` oder geschützte Pfade.
- Akzeptanzbefehle müssen im Plan vorkommen.
- Bei 3 aufeinanderfolgenden Ablehnungen erfolgt Eskalation (Status `error`, Aufforderung zur Task-Teilung).
