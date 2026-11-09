# Baseline Kontext & Test-Inventar: `contextra-db` (WP-13 / WP-100 Vorbereitung)

Stand: 2026-11-09
Basis: `restructure/main` (HEAD: 65e1d68a5aadbb6903e1b768a07e3fd7f048b6b3)
Branch: `wp/P13`
Toolchain: rustc / cargo 1.89.0 (c24e10642 2025-06-23)

---

## 1. Übersicht & Status

`contextra-db` ist die High-Level Collection- und Hybridsuch-API, die im späteren Verlauf (Phase 4 / WP-100 / P42) in die Produktfassade `contextra` eingefaltet wird.

### Testbestand (Stand nach Archivierung)
- **Archivierte Tests (nach `archive/tests-legacy/contextra-db/` verschoben via git mv):** 3 Dateien (Beweis-, Zeugen- und Compliance-Szenarien ohne mechanischen Wert für das Kernretrieval):
  - `e2e_tenant_deletion_proof.rs` (Beweis-Persistenz & Tenant-Isolation)
  - `deletion_proof_integration.rs` (Proof-Generierung bei `drop_collection`)
  - `e2e_compliance_scenario.rs` (Compliance E2E-Szenario mit DeletionProof)
- **Verbleibende Testdateien in `crates/contextra-db/tests/`:** 50 Testdateien (insgesamt ~11.243 Zeilen Testcode).
- **Produktionscode in `crates/contextra-db/src/`:** 5 Quelldateien (~2.134 Zeilen).

---

## 2. Testdateien-Inventar (50 aktive Testdateien)

| Nr. | Testdatei | Zeilen | Fokus / Mechanik |
|---|---|---|---|
| 1 | `atomic_commit.rs` | 116 | Atomare Writes / Transaktions-Commit |
| 2 | `auto_community_detection_test.rs` | 149 | Graph-Community-Erkennung |
| 3 | `backpressure_test.rs` | 97 | Backpressure & Überlaststeuerung |
| 4 | `cascade_invalidation_integration.rs` | 156 | Kaskadierende Graph-Kanteninvalidierung |
| 5 | `cascade_invalidation_test.rs` | 130 | Supersedes Kaskadeninvalidierung |
| 6 | `checkpoint_layer_bounds.rs` | 154 | Checkpoint Schichtengrenzen |
| 7 | `checkpoint_stress.rs` | 88 | Checkpoint Stress-Tests |
| 8 | `collection_contract.rs` | 152 | Collection API Verträge |
| 9 | `compound_split_recall_impact.rs` | 429 | Compound Splitting Recall Analyse |
| 10 | `compound_split_recall_impact_test.rs` | 484 | Recall Impact durch Splitter |
| 11 | `concurrent_collection_stress.rs` | 75 | Parallele Collection-Zugriffe |
| 12 | `consolidation_double_trigger_test.rs` | 162 | Doppelte Triggerung von Konsolidierung |
| 13 | `consolidation_integration_test.rs` | 270 | Hintergrund-Konsolidierungsdurchläufe |
| 14 | `context_compaction_retry_test.rs` | 241 | Retry-Verhalten bei Kontextkompaktierung |
| 15 | `cross_domain_chaos_matrix_test.rs` | 319 | Chaos-Matrix über Speicher & Indizes |
| 16 | `cross_signal_isolation_test.rs` | 302 | Snapshot-Isolation über Signale |
| 17 | `dimension_validation.rs` | 59 | Validierung der Embedding-Dimensionen |
| 18 | `diskann_collection_fallback.rs` | 110 | DiskANN Fallback-Strategie |
| 19 | `docid_128_consolidation.rs` | 46 | 128-Bit DocId Konsolidierung |
| 20 | `embedding_dimension_switch.rs` | 105 | Mismatch-Prüfung bei Modellwechsel |
| 21 | `fault_injection_2pc.rs` | 905 | 2-Phase-Commit & Fehlereinschleusung |
| 22 | `filter_tests.rs` | 145 | Metadaten- und Tag-Filterung |
| 23 | `full_stack_e2e.rs` | 131 | Vollständiger E2E-Suchfluss |
| 24 | `fusion_edge_cases_test.rs` | 652 | Randfälle bei Rank-Fusion (RRF) |
| 25 | `graph_entity_delete_test.rs` | 193 | Mechanisches Löschen von Graph-Entitäten |
| 26 | `graph_hybrid_search.rs` | 338 | Hybrid-Suche inkl. Graph-Signale |
| 27 | `hnsw_delete_and_backfill.rs` | 256 | HNSW Vektor-Löschung & Nachindizierung |
| 28 | `instance_orphan_registry_test.rs` | 158 | Erkennung verwaister Instanzen |
| 29 | `integration_temporal_filter.rs` | 230 | Zeitbasierte Filterung |
| 30 | `layer_bounds.rs` | 116 | Schichtengrenzen & API-Kapselung |
| 31 | `loom_relate_n_ary.rs` | 97 | Loom Concurrency-Test für N-Ary Relate |
| 32 | `multistep_query_rewriter_modalities.rs` | 229 | Multi-Step Query Rewriting Modalitäten |
| 33 | `multistep_retrieval_tests.rs` | 343 | Multi-Step Retrieval Abfragen |
| 34 | `no_blanket_allow_deprecated.rs` | 36 | Verbot von pauschalen allow(deprecated) |
| 35 | `pid_pool_production_caller.rs` | 140 | PID-Regler Aufrufe im Pool |
| 36 | `proptest_search_invariants.rs` | 234 | Property-Tests für Suchinvarianten |
| 37 | `search_result_bound.rs` | 244 | K-Bound Invarianten der Suche |
| 38 | `semantic_recall.rs` | 381 | Semantischer Recall Benchmark/Test |
| 39 | `signal_kind_no_new_variant.rs` | 62 | Invarianz von SignalKind |
| 40 | `snapshot_api.rs` | 198 | MVCC Read Snapshot Schnittstelle |
| 41 | `snapshot_recovery.rs` | 246 | Wiederherstellung aus Snapshots |
| 42 | `split_brain_vector_orphan_test.rs` | 582 | Split-Brain Vektor-Isolierung |
| 43 | `sprint2_acid_tests.rs` | 398 | ACID Garantien & Drop Collection |
| 44 | `stabilization_sprint_tests.rs` | 166 | Stabilitätsprüfungen |
| 45 | `stress.rs` | 78 | Paralleler Orchestrator Stress-Test |
| 46 | `supersedes_cascading_tombstone_test.rs` | 203 | Kaskadierende Tombstones bei Ersetzung |
| 47 | `text_integration.rs` | 112 | BM25 Textindex Integration |
| 48 | `transaction_isolation.rs` | 88 | Transaktions-Isolation |
| 49 | `truncation_filter_recall_test.rs` | 321 | Truncation & Filter Recall |
| 50 | `zettelkasten_links_test.rs` | 317 | Zettelkasten Hyperlink-Auflösung |

---

## 3. Testausführung & Externe Build-Blockaden

### Befehl:
```bash
cargo test -p contextra-db --no-run
```

### Gemessenes Ergebnis:
- **Build-Status:** Fehlschlag vor Testausführung (`rot`) aufgrund von Compiler-Fehlern in externen Abhängigkeiten (`contextra-crypto` und `contextra-vector`), die VOR Arbeitsbeginn in `restructure/main` vorlagen.
- **Grün / Rot / Ignoriert:** 0 ausgeführt (Kompilierung der Workspace-Test-Targets blockiert durch externe Crates).
- **Laufzeit:** Nicht messbar bis zur Behebung der vorgeschalteten Crates.

### Externe Ursachenanalyse (dokumentiert in `OPEN_ISSUES.md` gemäß Regel 5):
1. **`contextra-crypto` (lib):**
   - E0599 / E0433: Unerwarteter `cfg(serde)` Zustand und geänderte Typen (`HnswDeletionError::VerificationFailed` vs aktueller `contextra_types` Stand).
2. **`contextra-vector` (lib):**
   - E0599: Fehlende Methode `GenericHnswIndex::try_new_with_rng_and_kernel` (aktuell nur `try_new_with_rng_and_clock` oder `try_new_with_kernel`).
3. **`contextra-graph` (lib):**
   - E0433: Fehlende Imports bzw. optionale Features von `contextra_types` / `contextra_ports` in Abhängigkeitsketten.

Sobald die upstream Pakete (`crypto`, `vector`, `graph`) in `restructure/main` stabilisiert sind, baut `contextra-db` inklusive seiner 50 Testsuiten direkt durch.
