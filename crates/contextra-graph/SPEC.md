# SPEC: `contextra-graph` (Klasse A Baustein)

CSR-Graph für Wissensnetze, Relationen, Hyperkanten und Pfad-Traversierung (Signal 3 in 5-Signal Fusion).

---

## 1. Baukasten-Eigenschaften (Klasse A, SPEC v4.0 §2, B1–B12)

- **B1 (Keine internen Abhängigkeiten im Kern):** `contextra-graph` besitzt im Kern keine internen Abhängigkeiten. Übergangsweise Anbindungen an `contextra-types` und `contextra-ports` sind vollständig hinter die transitionalen Features `types-compat` und `ports-compat` gekapselt (Entfernung in Welle 6).
- **B2 (Keine internen Test-/Dev-Abhängigkeiten):** Die Dev-Abhängigkeit auf `contextra-testkit` ist eliminiert; autarke Test-Doubles (`InMemoryStorageEngine`, `FaultVfs`) liegen in `src/test_doubles.rs`.
- **B3 (API nur in Standard- und Crate-eigenen Typen):** Alle APIs operieren auf Standardtypen (`u64`, `f32`, `String`, `Option`, `Vec`) sowie crate-lokalen Typen (`DocId`, `EntityId`, `TxId`, `Entity`, `Edge`, `HyperEdgeId`, `RoleId`, `RoleBinding`, `PprConfig`, `PprAlgorithm`, `TlHfdParams`).
- **B4 (Ein `thiserror`-Fehlertyp):** Zentraler `GraphError` mit `#[non_exhaustive]` und abgeleiteter `GraphMutationError`-Taxonomie.
- **B5 (Build aus leerem Verzeichnis):** Gate G3 (`python3 tools/export_crate.py contextra-graph`) baut autark ohne Workspace-Kontext.
- **B6 (Schwere Deps hinter Features, minimales Default):** Alle Diffusions- und Analyse-Algorithmen (`ppr-forward-push`, `k-path-diffusion`, `apprh-diffusion`, `graph-connectivity-health`, `edge-reinforcement-learning`) sind additive Cargo-Features.
- **B7 (Keine Fremdfachlogik):** Keine Abhängigkeit zu LLM-Netzwerk-Clients, Vektorisierungs-Runtimes oder SQL-Engines im Bausteinkern.
- **B8 (Vollständige Dokumentation & Benchmarks):** `README.md`, `SPEC.md`, `//!`-Moduldokumentation und Criterion-Benchmarks (`benches/csr_traversal_bench.rs`, `benches/binary_edge_regression.rs`).
- **B9 (Zero-Panic):** Kein unkontrolliertes `unwrap`, `expect` oder ungesichertes Indexing in Produktionspfaden.
- **B10 (Unsafe-Disziplin):** `#![forbid(unsafe_code)]` in allen Produktionsmodulen der Crate.
- **B11 (Determinismus):** Zeit, Zufall und IDs laufen deterministisch über explizite Parameter oder Seed-Generatoren.
- **B12 (Eigene Lizenz & Rust-Version):** `edition = "2021"`, `rust-version = "1.89.0"`, `license = "MIT OR Apache-2.0"`.

---

## 2. Invarianten und Verträge

1. **Lock-freier Lesepfad:** CSR-Segmente sind unveränderlich (`ArcSwap<GraphInner>`); Mutationen akkumulieren in Delta-Puffern und werden atomar publiziert.
2. **Knotenbudgetierte Traversierung:** Traversierungs-Routinen terminieren in zyklischen Graphen garantiert deterministisch durch Hop-Limits (`MAX_TRAVERSAL_HOPS`) und Visited-Sets ohne Endlosschleifen.
3. **Bitemporale Sichtbarkeit:** Snapshots und Traversierungen respektieren Transaktions-Gültigkeitsgrenzen (`tx_valid_from`, `tx_valid_to`).
4. **Löschlogik-Separation:** Physische Lösch- und Beweisführung liegt gemäß SPEC §6 in `contextra-erasure`. Bis Welle 5 verbleiben Kaskaden-Routinen (`cascade_invalidate_*`, `process_cascade_queue`) deprecation-markiert für Abwärtskompatibilität.

---

## 3. Concurrency- und Durability-Garantien

- **Keine Locks über `.await`:** Mutex- und RwLock-Guards werden niemals über asynchrone Suspensionspunkte gehalten.
- **Hierarchie bei Locks:** In `SessionBranchTree` gilt zwingend die Erwerbsreihenfolge `nodes -> edges/active_head`.
- **Persistenz-Entkopplung:** Storage-I/O erfolgt über den neutralen `GraphStorage`-Trait oder transitional über `StorageEngine`.
