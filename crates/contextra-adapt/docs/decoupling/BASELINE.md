# Baseline: contextra-adapt (HEAD 882aada72)

## 1. Messdatum & Umgebung
- **Datum:** 2026-11-11
- **Toolchain:** `rustc 1.89.0` (workspace default)
- **Crate:** `contextra-adapt` (Ebene 0, Tier `beta`, Gruppe `llm`)

## 2. Baseline-Messung (vor erstem Umbau)

### 2.1 Crate-Build & Test
- `cargo check -p contextra-adapt --all-targets --locked`: **GRÜN** (0.3s)
- `cargo test -p contextra-adapt --locked`: **GRÜN** (52 Unit-Tests, 13 Integrations-Tests, 3 Doctests bestanden)
- `cargo test -p contextra-adapt --all-features --locked`: **GRÜN** (75 Unit-Tests, 26 Integrations-Tests, 3 Doctests bestanden)

### 2.2 Workspace-Konsistenz
- `cargo check --workspace --lib --bins --locked --keep-going`:
  - 1 bekannte rote Stelle in externem Crate: `contextra-infer-candle` (fehlende `contextra_types` Kanten / ungelöste Typen)
  - Abgelegt in `/tmp/ws_lib_baseline.txt`
- `cargo check --workspace --all-targets --locked --keep-going`:
  - 23 eindeutige Fehlerursachen in externen Targets (`contextra-infer-candle`, `contextra-db`, `contextra-engine`, etc. laut `BUILD_REPARATUR.md`)
  - Abgelegt in `/tmp/ws_all_baseline.txt`

### 2.3 Werkzeug-Audit-Baseline
- `tools/gate_layering.py --crate contextra-adapt`: **GRÜN** (Warnung: Feature `workspace` noch nicht deklariert)
- `tools/zero_panic_analyzer.py --crate contextra-adapt --fail-on-prod`: **0 Verstöße**
- `tools/determinism_guard.py --crate contextra-adapt`: **0 Verstöße**
- `tools/unsafe_boundary_auditor.py --crate contextra-adapt`: **0 Verstöße** (`#![forbid(unsafe_code)]` aktiv)
- `tools/modularity_score.py -c contextra-adapt`: **Score 100 / 100** (0 Bridge Refs)
- `tools/isolation_proof.py -c contextra-adapt --modes no-default,default,each-feature`: **GRÜN**
