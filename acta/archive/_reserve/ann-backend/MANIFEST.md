# Extraktion fuer `_reserve/ann-backend`

**Aufgabe:** Reserve: ANN-Backend (HNSW/DiskANN/ACORN). Nur nach Messbericht zulaessig (ADR-12).  
**Umfang des Archivs:** 101 Dateien, 17494 Zeilen (inkl. Tests)  

## Herkunft und Umbauauftrag

| Altquelle (Crate) | Umbauauftrag |
|---|---|
| `vector` | NUR nach Messbericht (ADR-12): ANN als Kandidatenerzeuger, nicht deterministisch (Z7 gilt dann nicht) |

## Pruefhinweise (automatisch ermittelt)

| Befund | Treffer | Folge |
|---|---:|---|
| float | 442 | f32/f64 (Z7: Festkomma in Ranking-Pfaden) |
| serde | 4 | serde/bincode (G-NOSERDE-DISK: Plattenformate von Hand kodieren) |
| async | 248 | tokio/async (G-NOTOKIO: Kern synchron) |
| zeit_zufall | 15 | Zeit/Zufall direkt (G3: nur ueber Ports Clock/CryptoRng) |
| panic | 441 | unwrap/expect/panic (G-LINT) |
| std_fs | 33 | std::fs direkt (G-FS: nur acta-fs) |
| hr6 | 1 | include!/#[deprecated] (HR-6) |
| hub | 49 | Hub-Import ex types/ports (HR-2/HR-3) |
| alt_spec | 11 | Alt-Referenzen (D-/ER-/INV-Kennungen): durch SPEC-Verweise ersetzen |

**Externe Abhaengigkeiten der Altquelle ausserhalb der Whitelist (3.8):** `ahash`, `blake3`, `memmap2`, `parking_lot`, `rand`, `tracing` — ersetzen oder per ADR aufnehmen.

## Dateien

| Datei (relativ zu `archive/_reserve/ann-backend/`) | LOC | Befunde |
|---|---:|---|
| `vector/src/acorn/gamma_augmentation.rs` | 54 | float=6, async=1 |
| `vector/src/acorn/mod.rs` | 68 | float=2 |
| `vector/src/acorn/naive_reference.rs` | 90 | float=5 |
| `vector/src/diskann/build/flush.rs` | 224 | float=6 |
| `vector/src/diskann/build/mod.rs` | 148 | float=5, async=1 |
| `vector/src/diskann/build/pending_wal.rs` | 224 | float=5, async=1 |
| `vector/src/diskann/build/tombstone_wal.rs` | 179 | async=1 |
| `vector/src/diskann/config.rs` | 79 | float=1 |
| `vector/src/diskann/filtered.rs` | 75 | float=1, async=1 |
| `vector/src/diskann/format.rs` | 211 | float=6 |
| `vector/src/diskann/hnsw_bridge.rs` | 59 | float=1, async=1 |
| `vector/src/diskann/integrity.rs` | 44 |  |
| `vector/src/diskann/io.rs` | 134 | std_fs=12, alt_spec=1 |
| `vector/src/diskann/mod.rs` | 25 |  |
| `vector/src/diskann/persistence/delta.rs` | 73 | float=2, async=1, alt_spec=1 |
| `vector/src/diskann/persistence/fallback.rs` | 23 |  |
| `vector/src/diskann/persistence/loader.rs` | 269 | float=1, async=1 |
| `vector/src/diskann/persistence/mod.rs` | 11 |  |
| `vector/src/diskann/persistence/node.rs` | 173 | float=3 |
| `vector/src/diskann/persistence/writer.rs` | 116 | float=1 |
| `vector/src/diskann/predicate_augmented.rs` | 274 | float=4, async=5 |
| `vector/src/diskann/search.rs` | 399 | float=13, async=1 |
| `vector/src/diskann/tests/build_search_tests.rs` | 328 | float=6, async=17, panic=13 |
| `vector/src/diskann/tests/config_tests.rs` | 243 | float=3, async=17, panic=17, alt_spec=1 |
| `vector/src/diskann/tests/mod.rs` | 10 |  |
| `vector/src/diskann/tests/persistence_tests.rs` | 274 | async=20, panic=40 |
| `vector/src/diskann/tests/recall_tests.rs` | 169 | float=4, async=8, panic=8 |
| `vector/src/diskann/tests/wal_tests.rs` | 117 | async=12, panic=1 |
| `vector/src/diskann/types.rs` | 303 | float=3, panic=12 |
| `vector/src/hnsw/acorn_filtered.rs` | 298 | float=4 |
| `vector/src/hnsw/adaptive_ef.rs` | 311 | float=7 |
| `vector/src/hnsw/arena.rs` | 430 | panic=1 |
| `vector/src/hnsw/batch.rs` | 90 |  |
| `vector/src/hnsw/clock.rs` | 50 |  |
| `vector/src/hnsw/config.rs` | 229 | float=5 |
| `vector/src/hnsw/core_insert/apply.rs` | 73 | float=1 |
| `vector/src/hnsw/core_insert/compute.rs` | 377 | float=2 |
| `vector/src/hnsw/core_insert/connectivity.rs` | 115 | float=3 |
| `vector/src/hnsw/core_insert/mod.rs` | 17 |  |
| `vector/src/hnsw/core_rebuild/backoff.rs` | 133 |  |
| `vector/src/hnsw/core_rebuild/full.rs` | 437 | float=2 |
| `vector/src/hnsw/core_rebuild/mod.rs` | 139 | float=5, panic=5 |
| `vector/src/hnsw/core_rebuild/region.rs` | 119 |  |
| `vector/src/hnsw/core_rebuild/seq_replay.rs` | 196 |  |
| `vector/src/hnsw/core_rebuild/snapshot.rs` | 442 | float=19 |
| `vector/src/hnsw/core_search/adaptive.rs` | 264 | float=3, async=3 |
| `vector/src/hnsw/core_search/filtered.rs` | 238 | float=1, async=1 |
| `vector/src/hnsw/core_search/layer.rs` | 175 | float=2 |
| `vector/src/hnsw/core_search/mod.rs` | 21 |  |
| `vector/src/hnsw/core_search/rerank.rs` | 43 | float=2 |
| `vector/src/hnsw/core_search/validate.rs` | 48 | float=2 |
| `vector/src/hnsw/deletion.rs` | 475 | float=1, alt_spec=1 |
| `vector/src/hnsw/mod.rs` | 80 | alt_spec=1 |
| `vector/src/hnsw/sq8_bias.rs` | 109 | float=9, serde=1 |
| `vector/src/hnsw/staging.rs` | 64 | float=1 |
| `vector/src/hnsw/types/builder.rs` | 193 | float=2, hr6=1 |
| `vector/src/hnsw/types/core.rs` | 161 | float=1 |
| `vector/src/hnsw/types/data.rs` | 72 | float=3 |
| `vector/src/hnsw/types/guards.rs` | 38 |  |
| `vector/src/hnsw/types/index.rs` | 262 | float=3 |
| `vector/src/hnsw/types/mod.rs` | 23 |  |
| `vector/src/hnsw/types/persist.rs` | 292 | float=3 |
| `vector/src/hnsw/types/seqlog.rs` | 23 |  |
| `vector/src/partial_rebuild.rs` | 378 | float=11, panic=1, alt_spec=4 |
| `vector/src/persistence/atomic.rs` | 59 | std_fs=1, alt_spec=2 |
| `vector/src/persistence/header.rs` | 469 | float=22 |
| `vector/src/persistence/io.rs` | 61 | std_fs=8 |
| `vector/src/persistence/mmap.rs` | 274 | async=1 |
| `vector/src/persistence/mod.rs` | 20 |  |
| `vector/src/persistence/node.rs` | 95 |  |
| `vector/src/persistence/tests/atomic_tests.rs` | 35 | panic=7, std_fs=2 |
| `vector/src/persistence/tests/header_tests.rs` | 48 |  |
| `vector/src/persistence/tests/mmap_tests.rs` | 448 | float=10, async=8, panic=7, std_fs=7 |
| `vector/src/persistence/tests/mod.rs` | 11 |  |
| `vector/src/persistence/tests/node_tests.rs` | 40 |  |
| `vector/src/quantize_rabitq.rs` | 526 | float=48, serde=1, zeit_zufall=2, panic=4 |
| `vector/tests/acorn_hnsw_differential.rs` | 300 | float=5, async=6, zeit_zufall=2, panic=5, std_fs=1, hub=2 |
| `vector/tests/acorn_naive_recall.rs` | 98 | float=2, panic=1, hub=1 |
| `vector/tests/acorn_selectivity_edge_cases_test.rs` | 93 | float=1, async=2, panic=11, hub=2 |
| `vector/tests/diskann_corruption_fallback_test.rs` | 122 | async=8, panic=17, hub=3 |
| `vector/tests/diskann_docid128_tombstones.rs` | 94 | async=2, hub=3 |
| `vector/tests/diskann_fault_injection_test.rs` | 87 | float=2, async=5, panic=10, hub=2 |
| `vector/tests/diskann_recall_test.rs` | 258 | float=27, async=4, zeit_zufall=3, panic=13, hub=3 |
| `vector/tests/diskann_search_filtered.rs` | 218 | float=2, async=15, panic=12, hub=3 |
| `vector/tests/hnsw_edge_cases.rs` | 372 | float=8, async=26, hub=2 |
| `vector/tests/hnsw_layer_determinism.rs` | 134 | float=7, async=6, panic=22, hub=2 |
| `vector/tests/hnsw_rebuild_connectivity_test.rs` | 77 | float=4, async=2, panic=6, hub=2 |
| `vector/tests/hnsw_rebuild_search_consistency.rs` | 159 | float=1, async=6, panic=10, hub=3 |
| `vector/tests/hnsw_scanner_purge_test.rs` | 273 | float=1, async=10, panic=43, std_fs=2, hub=4 |
| `vector/tests/hnsw_seed_determinism.rs` | 349 | float=17, async=6, panic=34, hub=3 |
| `vector/tests/hnsw_snapshot_delete_test.rs` | 143 | async=4, panic=19, hub=2 |
| `vector/tests/hnsw_tests.rs` | 449 | float=7, async=14, panic=45, hub=1 |
| `vector/tests/rabitq_calibration_recall.rs` | 152 | float=20, async=2, panic=3 |
| `vector/tests/rabitq_recall_calibration.rs` | 112 | float=6, hub=1 |
| `vector/tests/rabitq_recall_calibration_test.rs` | 25 | float=6, panic=4 |
| `vector/tests/recall.rs` | 89 | float=8, async=2, panic=8, hub=2 |
| `vector/tests/recall_audit.rs` | 465 | float=25, async=13, zeit_zufall=4, panic=44, hub=2 |
| `vector/tests/recall_vs_brute_force.rs` | 261 | float=18, async=12, zeit_zufall=4, panic=1, hub=2 |
| `vector/tests/sq8_bias_calibration.rs` | 32 | serde=2, panic=2, hub=1 |
| `vector/tests/sq8_error_bound_proptest.rs` | 39 | panic=4 |
| `vector/tests/sq8_rerank_recall.rs` | 196 | float=25, async=2, panic=11, hub=3 |
