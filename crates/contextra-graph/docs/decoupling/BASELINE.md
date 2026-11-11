# Baseline-Erhebung S-34 `contextra-graph` (Entkopplung)

Datum: 2026-11-15
Branch: jules-13242595197714389149-b82bf81c
Toolchain: rustc 1.89.0 (29483883e 2025-08-04) / cargo 1.89.0 (c24e10642 2025-06-23)

## 1. Kompilierungs- und Workspace-Status (Baseline)
- `cargo check --workspace --lib --bins --locked`: GRÜN
- `cargo check --workspace --all-targets --locked`: Vorbestehend ROT (2 Fehler in `contextra-store`: `MemtableScanner.runtime` unbenutztes Feld)
  - `contextra-store` (lib): error: field `runtime` is never read (crates/contextra-store/src/scanner/memtable.rs:32:5)
  - `contextra-store` (lib test): error: field `runtime` is never read
- `cargo check -p contextra-graph --all-targets`: GRÜN (2 Deprecation-Warnungen bzgl. `compat::purger::graph_purger` und `GraphPurger`)
- `cargo check -p contextra-graph --no-default-features`: 51 Fehler (unresolved imports `contextra_types`, `contextra_ports`, `GraphProvider`)

## 2. Tests (Baseline mit unverändertem Code)
- `cargo test -p contextra-graph --lib`: 218 passed, 0 failed, 0 ignored (Dauer: ~54s)
- `cargo test -p contextra-graph --test golden_corpus`: 2 passed, 0 failed, 0 ignored

## 3. Referenzinventur
- Referenzen auf `types`, `ports`, `testkit` laut `python3 tools/count_bridge_refs.py -c contextra-graph`: 135 Treffer
- Aufteilung:
  - `cascade.rs`: `ContextraError::InvalidInput`
  - `csr/`: `csr/graph_persist.rs`, `csr/graph_read/`, `csr/inner/`, `csr/tests/`
  - `ppr/`: `ppr/cost.rs`, `ppr/shadow_hook.rs`, `ppr/mod.rs`
  - `apprh/`: `apprh/mod.rs`, `apprh/error.rs`, etc.
  - `tl_hfd/`: `tl_hfd/diffusion.rs`, `tl_hfd/shadow.rs`, `tl_hfd/error.rs`
  - `path_rag/`: `path_rag/mod.rs`
  - `community.rs`: `CommunityDetectionConfig`, `CommunityAssignment`
  - `hyperedge.rs`: `ContextraError`, Re-Exports von Ports
  - `entity_extraction.rs`: `DocId`, `EntityId`, `ContextraError`
  - `provenance.rs`: `DocId`, `TxId`
  - `session_dag.rs`: `ContextraError`

## 4. Feature-Befund & Fremdnutzer
- Bestehende Features in `contextra-graph/Cargo.toml`:
  - `default = ["workspace", "experimental-graph"]`
  - `workspace = ["ports-compat", "types-compat"]`
  - `types-compat = ["dep:contextra-types", "dep:contextra-ports"]`
  - `ports-compat = ["types-compat"]`
  - `experimental-graph = []`
  - `docid-128 = []`
  - `graph-connectivity-health = []`
  - `edge-reinforcement-learning = []`
  - `ppr-forward-push = []`
  - `k-path-diffusion = []`
  - `apprh-diffusion = []`
  - `physio-percolation = ["graph-connectivity-health"]`
- Feature-Weiterleitungen durch Fremdnutzer:
  - `contextra-engine`:
    - `graph-connectivity-health = ["contextra-graph/graph-connectivity-health"]`
  - `contextra-cognition`:
    - `graph-connectivity-health = ["contextra-graph/physio-percolation", "contextra-engine/graph-connectivity-health"]`
    - `edge-reinforcement-learning = ["contextra-graph/edge-reinforcement-learning", "contextra-engine/edge-reinforcement-learning"]`
- Abhängigkeiten von `contextra-graph`:
  - `contextra-db` (`workspace = true`)
  - `contextra-engine` (`workspace = true`)
  - `contextra-agent` (`workspace = true`)
  - `contextra-cognition` (`workspace = true`)
  - `contextra-graph-fuzz` (`path = ".."`)

## 5. Zielvorgabe S-34
- Entkopplung des Kerns: Kern-Closure nur `contextra-ids`, `contextra-env`.
- `contextra-types` und `contextra-ports` nur hinter `workspace` in `src/compat/`.
- Fehlerzahl ohne Default-Features von 51 auf 0 reduzieren.
- Brücken-Referenzen im Kern auf 0 reduzieren.
