# Workspace Kompilierbarkeit & Gate-Status Report (2026-09-25)

**Datum:** 2026-09-25
**Toolchain:** `rustc 1.89.0 (29483883e 2025-08-04)`
**Gegenstand:** Rein diagnostischer Statusbericht zur aktuellen Kompilierbarkeit und zum Gate-Status des Workspaces.

---

## 1. Übersicht & Zusammenfassung

| Metrik / Gate | Ergebnis | Anmerkung |
| :--- | :--- | :--- |
| **Cargo Check Workspace** | ❌ **23 Fehler / 29 Warnings** | Alle 23 Fehler betreffen Test/Example/Bench-Targets von `contextra-db` (v.a. Veraltete `contextra_core`-Imports) |
| **`check-result-dropped-io`** | ✅ **PASSED** | 0 Violations (`xtask check-result-dropped-io`) |
| **Tech-Debt Audit (`debt-audit`)** | ⚠️ **Soft-Warnings / Test-Unwraps** | Kein `.unwrap()` in Prod-Code außerhalb genehmigter Pfade; `unsafe` isoliert; `std::fs` Soft-Warnings vorhanden |

---

## 2. Compiler-Fehler (23 Fehler)

Alle 23 Compiler-Fehler treten in Test-, Example- oder Benchmark-Dateien des Crates `contextra-db` auf. Die Fehlerursache ist überwiegend die Migration von `contextra_core` auf Ring-0 Crates (`contextra-types`, `contextra-ports`), wodurch alte Test-/Example-Dateien noch unbehobene `contextra_core`-Importe oder alte Trait-Signaturen referenzieren.

### Detaillierte Fehlerliste

| Nr. | Datei : Zeile | Target | Fehlercode | Exakte Fehlermeldung |
| :--- | :--- | :--- | :--- | :--- |
| 1 | `benches/migration_benchmarks.rs:6` | `migration_benchmarks` (bench) | `E0432` | unresolved import `contextra_core` |
| 2 | `benches/competitive_bench.rs:4` | `competitive_bench` (bench) | `E0432` | unresolved import `contextra_core` |
| 3 | `crates/contextra-db/examples/quickstart.rs:11` | `quickstart` (example) | `E0433` | failed to resolve: use of unresolved module or unlinked crate `contextra_core` |
| 4 | `crates/contextra-db/examples/hybrid_search.rs:11` | `hybrid_search` (example) | `E0433` | failed to resolve: use of unresolved module or unlinked crate `contextra_core` |
| 5 | `crates/contextra-db/examples/collections.rs:14` | `collections` (example) | `E0433` | failed to resolve: use of unresolved module or unlinked crate `contextra_core` |
| 6 | `crates/contextra-db/examples/collections.rs:113` | `collections` (example) | `E0433` | failed to resolve: use of unresolved module or unlinked crate `contextra_core` |
| 7 | `crates/contextra-db/tests/concurrent_collection_stress.rs:4` | `concurrent_collection_stress` (test) | `E0432` | unresolved import `contextra_core` |
| 8 | `crates/contextra-db/tests/context_compaction_retry_test.rs:1` | `context_compaction_retry_test` (test) | `E0432` | unresolved import `contextra_core` |
| 9 | `crates/contextra-db/tests/context_compaction_retry_test.rs:29` | `context_compaction_retry_test` (test) | `E0405` | cannot find trait `VectorIndex` in this scope |
| 10 | `crates/contextra-db/tests/context_compaction_retry_test.rs:36` | `context_compaction_retry_test` (test) | `E0405` | cannot find trait `VectorIndex` in this scope |
| 11 | `crates/contextra-db/tests/hnsw_delete_and_backfill.rs:4` | `hnsw_delete_and_backfill` (test) | `E0432` | unresolved import `contextra_core` |
| 12 | `crates/contextra-db/tests/stabilization_sprint_tests.rs:1` | `stabilization_sprint_tests` (test) | `E0432` | unresolved import `contextra_core` |
| 13 | `crates/contextra-db/tests/sprint2_acid_tests.rs:11` | `sprint2_acid_tests` (test) | `E0432` | unresolved import `contextra_core` |
| 14 | `crates/contextra-db/tests/sprint2_acid_tests.rs:108` | `sprint2_acid_tests` (test) | `E0599` | no method named `scan_prefix` found for struct `Arc<LsmStorage>` in the current scope |
| 15 | `crates/contextra-db/tests/sprint2_acid_tests.rs:122` | `sprint2_acid_tests` (test) | `E0599` | no method named `scan_prefix` found for struct `Arc<LsmStorage>` in the current scope |
| 16 | `crates/contextra-db/tests/fault_injection_2pc.rs:5` | `fault_injection_2pc` (test) | `E0432` | unresolved import `contextra_core` |
| 17 | `crates/contextra-db/tests/fault_injection_2pc.rs:312` | `fault_injection_2pc` (test) | `E0277` | the trait bound `FaultyStorage: contextra_ports::storage::StorageEngine` is not satisfied |
| 18 | `crates/contextra-db/tests/fault_injection_2pc.rs:312` | `fault_injection_2pc` (test) | `E0277` | the trait bound `FaultyVectorIndex: contextra_ports::vector_index::VectorIndex` is not satisfied |
| 19 | `crates/contextra-db/tests/graph_hybrid_search.rs:3` | `graph_hybrid_search` (test) | `E0432` | unresolved import `contextra_core` |
| 20 | `crates/contextra-db/tests/graph_hybrid_search.rs:48` | `graph_hybrid_search` (test) | `E0599` | no method named `add_entity` found for struct `Arc<contextra_graph::csr::graph_write::CsrGraph>` in the current scope |
| 21 | `crates/contextra-db/tests/graph_hybrid_search.rs:52` | `graph_hybrid_search` (test) | `E0599` | no method named `add_entity` found for struct `Arc<contextra_graph::csr::graph_write::CsrGraph>` in the current scope |
| 22 | `crates/contextra-db/tests/graph_hybrid_search.rs:64` | `graph_hybrid_search` (test) | `E0599` | no method named `commit` found for struct `Arc<contextra_graph::csr::graph_write::CsrGraph>` in the current scope |
| 23 | `crates/contextra-db/tests/compound_split_recall_impact.rs:4` | `compound_split_recall_impact` (test) | `E0432` | unresolved import `contextra_core` |

---

## 3. Compiler-Warnings (29 Warnings)

Gegenwärtig existieren 29 Warnings, gruppiert nach Crate und Datei.

### Crate: `contextra-db` (3 Warnings)
- `crates/contextra-db/tests/search_result_bound.rs:181` — `unused_variables`: unused variable: `file_content`
- `crates/contextra-db/tests/search_result_bound.rs:46` — `dead_code`: function `read_search_rs` is never used
- `crates/contextra-db/tests/cross_signal_isolation_test.rs:70` — `unused_imports`: unused import: `contextra_types::GraphTraversalStrategy`

### Crate: `contextra-store` (5 Warnings)
- `crates/contextra-store/tests/chaos_dropped_write.rs:15` — `dead_code`: function `open` is never used
- `crates/contextra-store/tests/chaos_dropped_write.rs:16` — `dead_code`: function `dup2` is never used
- `crates/contextra-store/tests/chaos_dropped_write.rs:17` — `dead_code`: function `close` is never used
- `crates/contextra-store/src/compaction/tests/basic.rs:4` — `unused_imports`: unused import: `contextra_core::StorageEngine`
- `crates/contextra-store/src/sstable/tests.rs:3` — `unused_imports`: unused import: `StorageEngine`

### Crate: `contextra-vector` (4 Warnings)
- `crates/contextra-vector/src/diskann/tests.rs:22` — `duplicate_macro_attributes`: duplicated attribute
- `crates/contextra-vector/src/diskann/tests.rs:19` — `unused_imports`: unused import: `super::*`
- `crates/contextra-vector/tests/../src/distance.rs:11` — `unused_imports`: unused imports: `cosine_distance_scalar`, `dot_product_scalar`, and `euclidean_distance_scalar`
- `crates/contextra-vector/tests/../src/distance.rs:17` — `unused_imports`: unused import: `validate_vector`

### Crate: `contextra-graph` (17 Warnings)
- `crates/contextra-graph/src/community/tests.rs:4` — `unused_imports`: unused import: `ContextraError`
- `crates/contextra-graph/src/community/tests.rs:6` — `unused_imports`: unused import: `super::*`
- `crates/contextra-graph/src/csr/tests/basic_tests.rs:4` — `unused_imports`: unused import: `crate::GraphIndexExt`
- `crates/contextra-graph/src/csr/tests/basic_tests.rs:9` — `unused_imports`: unused import: `super::*`
- `crates/contextra-graph/src/csr/tests/bitemporal_tests.rs:4` — `unused_imports`: unused import: `crate::GraphIndexExt`
- `crates/contextra-graph/src/csr/tests/bitemporal_tests.rs:6` — `unused_imports`: unused import: `ContextraError`
- `crates/contextra-graph/src/csr/tests/graph_index_tests.rs:4` — `unused_imports`: unused import: `crate::GraphIndexExt`
- `crates/contextra-graph/src/csr/tests/graph_index_tests.rs:6` — `unused_imports`: unused import: `DocId`
- `crates/contextra-graph/src/csr/tests/persistence_tests.rs:2` — `unused_imports`: unused import: `PersistedEdgePayload`
- `crates/contextra-graph/src/csr/tests/persistence_tests.rs:4` — `unused_imports`: unused import: `crate::GraphIndexExt`
- `crates/contextra-graph/src/csr/tests/persistence_tests.rs:6` — `unused_imports`: unused imports: `ContextraError` and `DocId`
- `crates/contextra-graph/src/ppr/tests/algo_tests.rs:4` — `unused_imports`: unused imports: `ContextraError` and `DocId`
- `crates/contextra-graph/src/ppr/tests/algo_tests.rs:6` — `unused_imports`: unused import: `super::*`
- `crates/contextra-graph/src/ppr/tests/algo_tests.rs:7` — `unused_imports`: unused import: `std::sync::Arc`
- `crates/contextra-graph/src/ppr/tests/context_tests.rs:4` — `unused_imports`: unused imports: `ContextraError` and `DocId`
- `crates/contextra-graph/src/ppr/tests/algo_tests.rs:813` — `dead_code`: struct `LogCaptureLayer` is never constructed
- `crates/contextra-graph/src/ppr/tests/algo_tests.rs:827` — `dead_code`: struct `StringVisitor` is never constructed

---

## 4. Gate-Ergebnisse & Tech-Debt Status

### 4.1 Dropped-IO Gate (`check-result-dropped-io`)
Command: `cargo xtask check-result-dropped-io`
```
✅ No violations found (check-result-dropped-io)
```
**Status:** BESTANDEN (0 Violations).

### 4.2 Tech-Debt Audit (`debt-audit`)
- **.unwrap() / .expect() in Produktionscode:** Keine unzulässigen `.unwrap()`-Aufrufe in Produktionscode vorhanden (alle Vorkommen befinden sich in Test-/Bench-Dateien oder isolierten Hilfsstrukturen).
- **Unsafe-Code:** Ausnahmslos auf erlaubte Crate-Inseln (`contextra-simd`, `contextra-sys`, `contextra-wire`) und `distance.rs` beschränkt. `#![forbid(unsafe_code)]` wird in allen übrigen Crates durchgesetzt.
- **`std::fs` in Produktionscode:** Enthalten in `contextra-store/src/sstable/reader.rs` (innerhalb von `spawn_blocking`) und `contextra-infer-candle` sowie `contextra-infer-onnx` zur Einlesung lokaler Modellgewichte.

---

## 5. Status-Abgleich der fünf gepatchten Dateien / Bereiche

Abgleich mit den in parallelen Tasks gepatchten / behandelten Bereichen:

| Datei / Bereich | Status laut diesem Lauf | Details |
| :--- | :--- | :--- |
| **fuzz-Targets** (`crates/*/fuzz/fuzz_targets/`) | ✅ **Konform** | Null Compiler-Errors oder Warnings |
| `crates/contextra-store/src/lsm/mod.rs` | ✅ **Konform** | Null Compiler-Errors oder Warnings |
| `crates/contextra-store/src/lsm/ops/compaction.rs` | ✅ **Konform** | Null Compiler-Errors oder Warnings |
| `crates/contextra-cognition/src/lib.rs` | ✅ **Konform** | Null Compiler-Errors oder Warnings |

---

## 6. Rohdaten-Ausgabe (Befehl & Auszüge)

### Befehl 1: `cargo check --workspace --all-targets --keep-going --message-format=short 2>&1`
```text
warning: contextra-wire@0.1.0: flatc not found, using existing generated code.
    Checking contextra-db v0.1.0 (/app/crates/contextra-db)
crates/contextra-db/../../benches/migration_benchmarks.rs:6:5: error[E0432]: unresolved import `contextra_core`: use of unresolved module or unlinked crate `contextra_core`
crates/contextra-db/tests/sprint2_acid_tests.rs:11:5: error[E0432]: unresolved import `contextra_core`: use of unresolved module or unlinked crate `contextra_core`
crates/contextra-db/../../benches/competitive_bench.rs:4:5: error[E0432]: unresolved import `contextra_core`: use of unresolved module or unlinked crate `contextra_core`
crates/contextra-db/tests/fault_injection_2pc.rs:5:5: error[E0432]: unresolved import `contextra_core`: use of unresolved module or unlinked crate `contextra_core`
crates/contextra-db/tests/fault_injection_2pc.rs:312:11: error[E0277]: the trait bound `FaultyStorage: contextra_ports::storage::StorageEngine` is not satisfied: the trait `contextra_ports::storage::StorageEngine` is not implemented for `FaultyStorage`
crates/contextra-db/tests/fault_injection_2pc.rs:312:11: error[E0277]: the trait bound `FaultyVectorIndex: contextra_ports::vector_index::VectorIndex` is not satisfied: the trait `contextra_ports::vector_index::VectorIndex` is not implemented for `FaultyVectorIndex`
crates/contextra-db/tests/sprint2_acid_tests.rs:108:10: error[E0599]: no method named `scan_prefix` found for struct `Arc<LsmStorage>` in the current scope
crates/contextra-db/tests/sprint2_acid_tests.rs:122:10: error[E0599]: no method named `scan_prefix` found for struct `Arc<LsmStorage>` in the current scope
crates/contextra-db/tests/concurrent_collection_stress.rs:4:5: error[E0432]: unresolved import `contextra_core`: use of unresolved module or unlinked crate `contextra_core`
crates/contextra-db/examples/quickstart.rs:11:20: error[E0433]: failed to resolve: use of unresolved module or unlinked crate `contextra_core`: use of unresolved module or unlinked crate `contextra_core`
crates/contextra-db/tests/hnsw_delete_and_backfill.rs:4:5: error[E0432]: unresolved import `contextra_core`: use of unresolved module or unlinked crate `contextra_core`
crates/contextra-db/tests/stabilization_sprint_tests.rs:1:5: error[E0432]: unresolved import `contextra_core`: use of unresolved module or unlinked crate `contextra_core`
crates/contextra-db/tests/graph_hybrid_search.rs:3:5: error[E0432]: unresolved import `contextra_core`: use of unresolved module or unlinked crate `contextra_core`
crates/contextra-db/examples/hybrid_search.rs:11:20: error[E0433]: failed to resolve: use of unresolved module or unlinked crate `contextra_core`: use of unresolved module or unlinked crate `contextra_core`
crates/contextra-db/examples/collections.rs:113:21: error[E0433]: failed to resolve: use of unresolved module or unlinked crate `contextra_core`: use of unresolved module or unlinked crate `contextra_core`
crates/contextra-db/tests/graph_hybrid_search.rs:48:10: error[E0599]: no method named `add_entity` found for struct `Arc<contextra_graph::csr::graph_write::CsrGraph>` in the current scope
crates/contextra-db/tests/graph_hybrid_search.rs:52:10: error[E0599]: no method named `add_entity` found for struct `Arc<contextra_graph::csr::graph_write::CsrGraph>` in the current scope
crates/contextra-db/examples/collections.rs:14:20: error[E0433]: failed to resolve: use of unresolved module or unlinked crate `contextra_core`: use of unresolved module or unlinked crate `contextra_core`
crates/contextra-db/tests/graph_hybrid_search.rs:64:11: error[E0599]: no method named `commit` found for struct `Arc<contextra_graph::csr::graph_write::CsrGraph>` in the current scope
crates/contextra-db/tests/compound_split_recall_impact.rs:4:5: error[E0432]: unresolved import `contextra_core`: use of unresolved module or unlinked crate `contextra_core`
crates/contextra-db/tests/context_compaction_retry_test.rs:1:5: error[E0432]: unresolved import `contextra_core`: use of unresolved module or unlinked crate `contextra_core`
crates/contextra-db/tests/context_compaction_retry_test.rs:29:50: error[E0405]: cannot find trait `VectorIndex` in this scope
crates/contextra-db/tests/context_compaction_retry_test.rs:36:27: error[E0405]: cannot find trait `VectorIndex` in this scope
```

### Befehl 2: `cargo xtask check-result-dropped-io`
```text
✅ No violations found (check-result-dropped-io)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.19s
     Running `xtask/target/debug/xtask check-result-dropped-io`
```
