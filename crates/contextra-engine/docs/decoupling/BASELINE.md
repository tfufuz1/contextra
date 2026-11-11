# BASELINE — contextra-engine (S-64)

Erhebungszeitpunkt: 11. November 2026
Commit: f92caab (HEAD vor Umbau)

## 1. Werkzeugläufe

### 1.1 decouple_inspector.py
- Modularitäts-Score: 10/100
- Umfang: 17.617 LOC in 87 Dateien
- Schnittstellen: 7 Traits, 74 Structs, 9 Enums
- Funktionen: 357 pub / 704 gesamt
- Interne Abhängigkeiten (12): contextra-adapt, contextra-checkpoint, contextra-graph, contextra-infer-onnx, contextra-mvcc, contextra-ports, contextra-rank, contextra-sandbox, contextra-store, contextra-text, contextra-types, contextra-vector
- Schnittstellen-Kopplung:
  - contextra-rank: 17 Symbole
  - contextra-types: 27 Symbole
  - contextra-ports: 21 Symbole
  - contextra-graph: 10 Symbole
  - contextra-store: 5 Symbole
  - contextra-text: 6 Symbole
  - contextra-vector: 4 Symbole
  - contextra-sandbox: 3 Symbole
  - contextra-mvcc: 3 Symbole
- Direkte Kernel-Importe: 195 Stellen
- DSGVO-/Löschketten-Relikte: 8 Stellen (in `src/lib.rs` und `src/contextra_impl/lifecycle.rs`)

### 1.2 gate_layering.py
- Befehl: `python3 tools/gate_layering.py --crate contextra-engine`
- Ergebnis: 2 Befunde
  1. Kern-Closure: Unerlaubte interne Abhängigkeiten `contextra-crypto`, `contextra-env`, `contextra-ids`, `contextra-sys` (transitive Kern-Closure der genutzten Crates; in Root-`capabilities.toml` fehlen diese).
  2. Dev-Abhängigkeit auf `contextra-testkit` unzulässig (fehlt in `dev_allowed`).

### 1.3 abhaengigkeits_budget.py
- Kern-LOC: 21.935
- mit Abh.-LOC: 144.223
- Crates: 14

### 1.4 count_bridge_refs.py
- Bridge Refs (types|ports|testkit): 497

### 1.5 feature_audit.py
- Befund für contextra-engine: `default` (unbenutztes Default-Feature).

### 1.6 zero_panic_analyzer.py
- Produktion:
  - prod_unwrap: 111
  - prod_expect: 1
  - prod_panic: 6
  - prod_assert: 66
  - test_panics: 1442

### 1.7 determinism_guard.py
- Produktion: 19 Befunde (1 instant_now, 1 system_time_now, 7 random_hashmap, 10 random_hashset).

### 1.8 contract_diff_detector.py
- 42 port traits mit 96 Methoden inspiziert.

## 2. Build- und Feature-Matrix (Baseline)

| Modus / Feature | Befehl | Status |
|---|---|---|
| no-default | `cargo check -p contextra-engine --no-default-features --locked` | Grün |
| default | `cargo check -p contextra-engine --locked` | Grün |
| onnx | `cargo check -p contextra-engine --no-default-features --features onnx --locked` | Grün |
| sandbox | `cargo check -p contextra-engine --no-default-features --features sandbox --locked` | Grün |
| experimental-diskann | `cargo check -p contextra-engine --no-default-features --features experimental-diskann --locked` | Grün |
| adaptive-candidate-pool-sizing | `cargo check -p contextra-engine --no-default-features --features adaptive-candidate-pool-sizing --locked` | Grün |
| exp-extraction | `cargo check -p contextra-engine --no-default-features --features exp-extraction --locked` | Grün |
| entity-extraction | `cargo check -p contextra-engine --no-default-features --features entity-extraction --locked` | Grün |
| reranking | `cargo check -p contextra-engine --no-default-features --features reranking --locked` | Grün |
| background-maintenance | `cargo check -p contextra-engine --no-default-features --features background-maintenance --locked` | Grün |
| graph-connectivity-health | `cargo check -p contextra-engine --no-default-features --features graph-connectivity-health --locked` | Grün |
| edge-reinforcement-learning | `cargo check -p contextra-engine --no-default-features --features edge-reinforcement-learning --locked` | Grün |
| auto-extraction-opt-out | `cargo check -p contextra-engine --no-default-features --features auto-extraction-opt-out --locked` | Grün |

## 3. Test-Status (Baseline)

- Bestehende Unstimmigkeiten vor Umbau:
  - `src/background_workers/tests.rs`: `DecayControllerConfig` fehlte `base_half_life_ms: None` (behoben nach R0.9).
  - `tests/tenant_wrapper_coverage.rs`, `tests/invariant_tenant_isolation.rs`, `tests/tenant_index_persistence_reload.rs`: Veralteter Aufruf von `drop_collection` mit DeletionProof-Parametern nach WP-14 Rückbau (behoben nach R0.9).
  - `tests/characterization_consistency_and_determinism.rs`: Veraltete `SearchResult`- und `reciprocal_rank_fusion`-Signatur (behoben nach R0.9).
  - `tests/query_builder_scope_acorn.rs`: `ScoredDocument` Importpfad aus `contextra_types` (behoben nach R0.9).
  - `tests/decay_and_expiry.rs`: Nicht in unserem Eigentum (ENG-B); erfordert `base_half_life_ms` in `DecayControllerConfig` (dokumentiert in FINDINGS.md).
