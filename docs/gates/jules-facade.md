# Fassade: Jules (`jules`)

## Zweck
`jules` bietet eine 5-Phasen-Fassade über alle Einzelkommandos des Harnesses.

## Befehle
- `cargo xtask jules start --card <DATEI>`
- `cargo xtask jules check`
- `cargo xtask jules verify`
- `cargo xtask jules submit`
- `cargo xtask jules stop --reason <GRUND>`

## Konfiguration
Die Zuordnung Phase -> Einzelkommandos ist dynamisch in `.jules/harness-phases.toml` konfiguriert. Missing required commands lead to fail-closed Exit 2 with explicit session ownership hints.
