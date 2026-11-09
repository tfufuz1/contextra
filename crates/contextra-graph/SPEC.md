# SPEC: `contextra-graph` (Klasse A Baustein)

CSR-Graph für Wissensnetze, Relationen, Hyperkanten und Pfad-Traversierung (Signal 3 in 5-Signal Fusion).

---

## 1. Baukasten-Eigenschaften (Klasse A, SPEC v4.0 §2, B1–B12)

- **B1 (Keine internen Abhängigkeiten im Kern):** `contextra-graph` besitzt im Kern keine internen Abhängigkeiten. Übergangsweise Anbindungen an `contextra-types` und `contextra-ports` sind vollständig hinter die transitionalen Features `types-compat` und `ports-compat` gekapselt.
- **B2 (Keine internen Test-/Dev-Abhängigkeiten):** Keine internen Dev-Abhängigkeiten; autarke Test-Doubles liegen in `src/test_doubles.rs` und `tests/`.
- **B3 (API nur in Standard- und Crate-eigenen Typen):** Alle APIs operieren auf Standardtypen (`u64`, `f32`, `String`, `Option`, `Vec`, `&[u8]`) sowie crate-lokalen Typen (`DocId`, `EntityId`, `TxId`, `Entity`, `Edge`, `HyperEdgeId`, `RoleId`, `RoleBinding`, `PprConfig`, `PprAlgorithm`, `TlHfdParams`).
- **B4 (Ein `thiserror`-Fehlertyp):** Primärer `Error`-Typ (`GraphError`, `#[non_exhaustive]`) mit Subtypen (`GraphMutationError`) und Crate-Vertragswandlung (`contract::Error`).
- **B5 (Build aus leerem Verzeichnis):** Gate G3 (`python3 tools/export_crate.py contextra-graph`) baut autark ohne Workspace-Kontext.
- **B6 (Schwere Deps hinter Features, minimales Default):** `default = ["std"]`. Standard-Features `serde`, `tracing`, `inspect`, `experimental` sowie Algorithmen-Features (`ppr-forward-push`, `k-path-diffusion`, `apprh-diffusion`, `graph-connectivity-health`, `edge-reinforcement-learning`).
- **B7 (Keine Fremdfachlogik / Domänenreinheit):** Keine DSGVO-, Saga- oder DeletionProof-Logik. Neutrale `remove`/`compact`-Methoden und lesende `inspect`-Sicht.
- **B8 (Vollständige Dokumentation & Benchmarks):** `README.md`, `SPEC.md`, `//!`-Moduldokumentation und Criterion-Benchmarks (`benches/csr_traversal_bench.rs`, `benches/binary_edge_regression.rs`).
- **B9 (Zero-Panic):** Kein unkontrolliertes `unwrap`, `expect` oder ungesichertes Indexing in Produktionspfaden.
- **B10 (Unsafe-Disziplin):** `#![forbid(unsafe_code)]` in allen Produktionsmodulen der Crate.
- **B11 (Determinismus):** Zeit, Zufall und IDs laufen deterministisch über explizite Parameter oder injizierte Traits (`Clock`, `Rng`). Iterationsreihenfolge über `BTreeMap`/`BTreeSet`.
- **B12 (Eigene Lizenz & Rust-Version):** `edition = "2021"`, `rust-version = "1.89"`, `license = "MIT OR Apache-2.0"`.

---

## 2. Invarianten und Test-Zuordnung

| Invariante | Beschreibung | Test / Nachweis |
|---|---|---|
| `INV-GRAPH-001` | Lock-freier Lesepfad via `ArcSwap<GraphInner>`, Mutationen akkumulieren in Delta-Puffern. | `tests/csr_benchmark.rs`, `tests/proptest_csr_invariants.rs` |
| `INV-GRAPH-002` | Traversierungs-Routinen terminieren in zyklischen Graphen garantiert deterministisch durch Hop-Limits. | `src/csr/tests/persistence_tests.rs::test_node_budgeted_cyclic_traversal_termination` |
| `INV-GRAPH-003` | Bitemporale Snapshot-Isolation und TxId-Konsistenz. | `tests/ppr_snapshot_isolation.rs`, `tests/bitemporal_tests.rs` |
| `INV-GRAPH-004` | Deterministische Community Detection und PageRank-Ordnung. | `tests/provider_tests.rs`, `tests/leiden_reference_validation.rs` |
| `INV-GRAPH-005` | Zero-Panic bei unvollständigen oder malformierten Eingaben in Regel-Extraktion. | `tests/entity_extraction_edge_cases.rs`, `tests/entity_extraction_tests.rs` |
| `INV-GRAPH-006` | Physische Kantenbereinigung via neutralem `GraphMaintainer`. | `src/purger.rs::tests::test_graph_maintainer_existing_edges` |

---

## 3. Concurrency- und Lock-Modell

- **CsrGraph:** `parking_lot::RwLock` über `inner`. Keine Locks werden über `.await`-Punkte gehalten.
- **SessionBranchTree:** Strikte Erwerbsreihenfolge `nodes -> edges/active_head`. Verhindert Deadlocks bei nebenläufigen Branch- und Merge-Operationen.
- **Budgetübersicht:** Maximale Kanten pro Traversierung (`MAX_TRAVERSAL_HOPS = 16`), maximales $k$ für PPR (`MAX_SEARCH_K = 1_000`), Hyperedge-Fanout-Limit (`MAX_HYPEREDGE_CASCADE_FANOUT = 10_000`).
