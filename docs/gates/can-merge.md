# Harness Command: can-merge

`can-merge` analysiert Gate-Ergebnisse eines PRs oder lokalen Testlaufs und entscheidet, ob ein Merge zulässig ist.

## CLI-Syntax

```bash
cargo run --manifest-path xtask/Cargo.toml -- can-merge [--results-dir <dir> | --pr <nr>] [--json]
```

## Funktionsweise
- Wertet JSON-Dateien aus `--results-dir` oder GitHub PR Check-Runs via `gh` aus.
- Gibt „JA“ oder „NEIN“ aus mit maximal 3 Hauptgründen.
- Wendet Fail-Closed-Semantik an (fehlende oder ungültige Eingaben führen zu „NEIN“).
