# Handoff Report: MISSION `ENG-1` · Engine-Collections (CRUD, Query, Filter, Fusion, Extraction, Chunker)

## 1. Übersicht & Zielerreichung
Die Collection-Schicht von `contextra-engine` wurde gemäß den Vorgaben aus `SPEC.md`, `CONSTITUTION.md` und dem Auftrag `ENG-1` gehärtet, entkoppelt und deterministisch abgesichert.

- **Collection (`collection/**`):**
  - Lock-Hierarchie (`Dokument -> Commit-Barriere -> Purge-Barriere -> Journal -> Speicherebene`) und `kv_locks` gewahrt.
  - Alle 9 Lock-across-await-Verstöße im Scope durch `concurrency_lock_auditor.py` wurden vollständig eliminiert (Score: 0 Hazards in OWNS-Pfaden).
  - Port-Entkopplung: `Collection` exponiert `graph_port(&self) -> Arc<dyn GraphIndex>` und `embedder(&self) -> Option<Arc<dyn TextEmbeddingEngine>>` zur sauberen Inversion of Control.
- **Löschen (`crud/delete`):**
  - Neutrale Operationen (`remove`, Tombstone, Index-Löschung per Port).
  - Null Abhängigkeiten zu Deletion-Saga-, Witness- oder DeletionProof-Logik im Collection-Kern (Regel M7). VETO-F02 eingehalten.
- **Filter (`src/filter.rs`):**
  - Strikte Begrenzung gegen Rekursions- und Ressourcendos: `MAX_FILTER_DEPTH = 16`, `MAX_FILTER_NODES = 128`.
  - Validierung via `validate_filter_expr_bounds` und `validate_bounds`.
- **Temporal Filter & Decay Controller (`src/temporal_filter.rs`, `src/decay_controller.rs`):**
  - Zeitoperationen vollständig entkoppelt und über den injizierten `Clock`-Port gesteuert (`with_clock`, `apply_temporal_validity_filter_with_clock`).
  - Total deterministische Sortierung bei Gleichständen (`sort_candidates_deterministic`, B11).
- **Mandanten-Isolation (VETO-F10):**
  - Bewiesen durch Negativtest `tests/collection_tenant_isolation_veto_f10.rs`: Namespace und Tenant-Präfixe können niemals durch Such- oder Abfrageparameter manipuliert oder überschrieben werden.
- **Differentialtest:**
  - `tests/collection_differential_oracle.rs`: Verifiziert Collection-Suche gegen ein Referenz-Orakel aus Brute-Force über alle Dokumente mit deterministischem Tie-Breaking.

## 2. Invarianten-Matrix
| Invariante | Status | Beleg / Test |
|---|---|---|
| Zero-Panic | Erfüllt | Keine Panics im Produktionscode unter OWNS |
| B11 (Determinismus) | Erfüllt | `tests/collection_differential_oracle.rs`, `temporal_filter.rs` |
| Lock across await | 0 Verstöße | `concurrency_lock_auditor.py -c contextra-engine` meldet 0 in OWNS |
| VETO-F02 | Erfüllt | Keine ad-hoc Rewiring-Pfade in Collection-CRUD |
| VETO-F10 | Erfüllt | `tests/collection_tenant_isolation_veto_f10.rs` |
| M7 (Erasure Leak) | Erfüllt | Keine Deletion-Saga/Proof-Kopplung im Collection-CRUD |

## 3. Geänderte Pfade (Strict OWNS)
- `crates/contextra-engine/src/collection/crud/insert.rs`
- `crates/contextra-engine/src/collection/crud/internal.rs`
- `crates/contextra-engine/src/collection/kv_lock.rs`
- `crates/contextra-engine/src/collection/maintenance.rs`
- `crates/contextra-engine/src/collection/mod.rs`
- `crates/contextra-engine/src/collection/query_builder/tests.rs`
- `crates/contextra-engine/src/collection/search/basic.rs`
- `crates/contextra-engine/src/decay_controller.rs`
- `crates/contextra-engine/src/filter.rs`
- `crates/contextra-engine/src/temporal_filter.rs`
- `crates/contextra-engine/tests/collection_differential_oracle.rs`
- `crates/contextra-engine/tests/collection_tenant_isolation_veto_f10.rs`
- `docs/handoff/ENG-1.md`
