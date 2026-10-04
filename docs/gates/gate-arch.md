# Aggregate Gate Arch (`gate-arch`)

## Zweck
Bündelt die Architektur- und Strukturprüfungen (`check-dag`, `check-ring-layering`, `check-vetoes`, `check-duplicate-symbols`, `check-unsafe-islands`, `check-ring0-async-purity`, `check-ring-capabilities-consistency`).

## Bestandteile & Trigger
- `check-dag`: Immer aktiv. Prüft Zyklusfreiheit des Crate-Abhängigkeitsgraphen.
- `check-ring-layering`: Immer aktiv. Prüft Einhaltung der Ring-Schichten.
- `check-vetoes`: Immer aktiv. Prüft Veto-Sperren und ADR-Fristen.
- `check-duplicate-symbols`: Immer aktiv. Erkennt doppelte Symbole in Modulen.
- `check-unsafe-islands`: Pfadgesteuert (`*.rs` mit `unsafe`, `Cargo.toml`, `capabilities.toml`).
- `check-ring0-async-purity`: Pfadgesteuert (Ring-0 Crates / `Cargo.toml`).
- `check-ring-capabilities-consistency`: Pfadgesteuert (`capabilities.toml` / `Cargo.toml`).

## CLI-Syntax
```bash
cargo xtask gate-arch [--root <dir>] [--base <rev>] [--head <rev>] [--strict] [--changed|--all] [--json]
```
- Exit-Codes: `0` wenn alle aktiven Bestandteile OK/SKIP; `1` bei jedem FAIL.
