# Baseline contextra-rank

Erfasst am HEAD `882aada72` vor Eingriffen.

## 1. Lokale Befehle (contextra-rank)
- `cargo check -p contextra-rank --all-targets --locked`: GRÜN (ca. 0.3s)
- `cargo test -p contextra-rank --locked`: GRÜN (21 doc/unit/integration targets passed, 0 failed, 1 ignored)
- `cargo clippy -p contextra-rank --all-targets --locked -- -D warnings`: ROT (2 Fehler in `src/compat/error.rs` und `src/compat/types.rs`)

## 2. Workspace-Baseline (Referenz)
- `cargo check --workspace --lib --bins --locked --keep-going`: 1 Target fehlerhaft (`contextra-infer-candle` mit 37 Fehlern)
- `cargo check --workspace --all-targets --locked --keep-going`: 35 fremde Targets fehlerhaft (`contextra-erasure`, `contextra-infer-candle`, `contextra-store`), 23 eindeutige Fehlermeldungen in `/tmp/ws_all_baseline.txt`.
- `contextra-rank` ist in der Workspace-Baseline fehlerfrei.

## 3. Tool-Baseline
- `gate_layering.py --crate contextra-rank`: GRÜN
- `count_bridge_refs.py -c contextra-rank`: 0 Bridge-Refs
- `zero_panic_analyzer.py --crate contextra-rank`: 0 Prod-Verstöße
- `determinism_guard.py --crate contextra-rank`: 0 Nicht-deterministische Stellen
- `unsafe_boundary_auditor.py --crate contextra-rank`: 0 Unsafe-Stellen
