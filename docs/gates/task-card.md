# Gate: Task Card (`task-card`)

## Zweck
Das `task-card` Gate stellt sicher, dass jede Arbeitsaufgabe schemakonform strukturiert, scope-begrenzt und budgetiert ist.

## Befehle
- `cargo xtask task-card new --crate <CRATE> --title <TITLE>`: Erzeugt eine neue Task-Karte unter `.jules/tasks/T-YYYY-NNNN.toml`.
- `cargo xtask task-card lint <DATEI>`: Prüft eine spezifische Task-Karte.
- `cargo xtask task-card lint --pr`: Prüft im PR-Kontext (`PR_LABELS`), ob eine referenzierte Karte existiert und gültig ist.

## Invarianten
- Pfad-Scope ohne `**`-Vollglobs.
- Geschützte Pfade bleiben unangetastet (außer mit explizitem `protected_change = "ADR-NNN"`).
- Budget-Obergrenzen: Files <= 25, Lines <= 1000, Iterations <= 12.
