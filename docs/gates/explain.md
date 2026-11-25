# Harness Command: explain

`explain` liefert detaillierte Kontextinformationen zu Qualitätsgates, Invarianten und Architektureinschränkungen im Repository.

## CLI-Syntax

```bash
cargo run --manifest-path xtask/Cargo.toml -- explain <gate|invariante> [--json]
cargo run --manifest-path xtask/Cargo.toml -- explain --list [--json]
```

## Funktionsweise
- Liest zur Laufzeit dynamisch `docs/explain/<name>.toml` aus dem Repository.
- Gibt strukturiert aus: Was ist passiert, Warum gibt es die Regel, Typischer Fix und Nicht erlaubte Abkürzungen.
- Liefert bei unbekannten Namen eine Fehlermeldung mit ähnlich geschriebenen Gate-Namen.
