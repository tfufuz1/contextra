# Harness Command: flake-report

`flake-report` spürt nicht-deterministische Test-Flakes auf.

## CLI-Syntax

```bash
cargo run --manifest-path xtask/Cargo.toml -- flake-report [--runs <n>] [--tests <liste>] [--crate <c>] [--json]
```

## Funktionsweise
- Führt `cargo nextest run` n-mal aus und berechnet die Erfolgsquote je Test.
- Markiert Tests mit 0 < Quote < 1 als flaky.
- Schreibt Ergebnisse nach stdout und `.jules/local/flake-report.json`.
