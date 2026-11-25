# Harness Command: risk

`risk` berechnet einen deterministischen Risiko-Score (0–100) für eine Änderung zwischen zwei Git-Revisionen.

## CLI-Syntax

```bash
cargo run --manifest-path xtask/Cargo.toml -- risk [--base <rev>] [--head <rev>] [--json]
```

## Funktionsweise
- Analysiert berührte Crates, Diff-Größe, Unsafe-Inseln, WAL/Crypto-Pfade und geschützte Pfade.
- Berechnet Score, Risikostufe (niedrig/mittel/hoch/kritisch) und erzeugt die Liste der erforderlichen Pflicht-Gates.
