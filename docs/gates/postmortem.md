# Harness Command: postmortem

`postmortem` verwaltet Incident-Postmortems im Repository.

## CLI-Syntax

```bash
cargo run --manifest-path xtask/Cargo.toml -- postmortem new --title <t> [--incident <datei>]
cargo run --manifest-path xtask/Cargo.toml -- postmortem check [--json]
```

## Funktionsweise
- `new`: Anlegen von `docs/postmortems/PM-NNNN-<slug>.md` auf Basis von `_TEMPLATE.md`.
- `check`: Prüft alle Postmortems auf Vollständigkeit aller Pflichtabschnitte und die Existenz referenzierter Test-/Gate-Pfade sowie ADRs.
