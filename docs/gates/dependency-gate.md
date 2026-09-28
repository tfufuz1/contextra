# Harness Command: dependency-gate

`dependency-gate` prüft neu hinzugefügte Abhängigkeiten in `Cargo.toml` und `Cargo.lock`.

## CLI-Syntax

```bash
cargo run --manifest-path xtask/Cargo.toml -- dependency-gate [--base <rev>] [--head <rev>] [--json]
```

## Funktionsweise
- Prüft Lizenzen via `deny.toml` und Audit-Einträge via `supply-chain/`.
- Zählt neue transitive Pakete und validiert Ring-Invarianten aus `capabilities.toml`.
- Erfordert zwingend einen `Dependency-Reason: <text>` Trailer im Commit-Text bei neuen Abhängigkeiten.
