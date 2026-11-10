# Baseline-Erhebung `contextra-graph` (GR6.0)

Datum: 2026-11-10
Git Branch: jules/graph-6-harden
Toolchain: rustc 1.89.0 / cargo

## 1. Kompilierungs- und Workspace-Status
- `cargo check -p contextra-graph --all-targets --locked`: **GRÜN** (2 Deprecation-Warnungen bzgl. `purger::graph_purger`/`GraphPurger`)
- `cargo check --workspace --all-targets --locked`: ROT (10 Fehler in `contextra-store`, Baseline-Problem)
- Verbraucher-Crates (`contextra-engine`, `contextra-db`, `contextra-cognition`): unbeeinflusst

## 2. Metriken contextra-graph
- LOC (`tools/loc.py`): 13.382 Zeilen über 71 Dateien
- Referenzen `contextra_types`/`ports`: 167 in `src/` (141 Prod, 26 Test)

## 3. Feature-Befund
- `default = ["types-compat", "ports-compat", "experimental-graph"]`
- `experimental-graph`: 2 cfg-Stellen.
- `docid-128`: 4 cfg-Stellen.
- Weiterleitungen (`graph-connectivity-health`, `edge-reinforcement-learning`, `physio-percolation`, `ppr-forward-push`) intakt.
