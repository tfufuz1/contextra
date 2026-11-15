# Extraktion fuer `_reserve/ann-backend`

**Aufgabe:** Reserve: ANN-Backend (HNSW/DiskANN/ACORN). Nur nach Messbericht zulaessig (ADR-12).  
**Umfang des Archivs:** 103 Dateien, 17980 Zeilen (inkl. Tests)  

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
| `vector/src/acorn/gamma_augmentation.rs` | 53 | float=6, async=1 |
| `vector/src/acorn/mod.rs` | 67 | float=2 |
| `vector/src/acorn/naive_reference.rs` | 89 | float=5 |
| `vector/src/diskann/build/flush.rs` | 223 | float=6 |
| `vector/src/diskann/build/mod.rs` | 147 | float=5, async=1 |
| `vector/src/diskann/build/pending_wal.rs` | 223 | float=5, async=1 |
| `vector/src/diskann/build/tombstone_wal.rs` | 178 | async=1 |
| `vector/src/diskann/config.rs` | 78 | float=1 |
| `vector/src/diskann/filtered.rs` | 74 | float=1, async=1 |
| `vector/src/diskann/format.rs` | 210 | float=6 |
| `vector/src/diskann/hnsw_bridge.rs` | 58 | float=1, async=1 |
| `vector/src/diskann/integrity.rs` | 43 |  |
| `vector/src/diskann/io.rs` | 133 | std_fs=12, alt_spec=1 |
| `vector/src/diskann/mod.rs` | 24 |  |
| `vector/src/diskann/persistence/delta.rs` | 72 | float=2, async=1, alt_spec=1 |
| `vector/src/diskann/persistence/fallback.rs` | 22 |  |
| `vector/src/diskann/persistence/loader.rs` | 268 | float=1, async=1 |
| `vector/src/diskann/persistence/mod.rs` | 10 |  |
| `vector/src/diskann/persistence/node.rs` | 172 | float=3 |
| `vector/src/diskann/persistence/writer.rs` | 115 | float=1 |
| `vector/src/diskann/predicate_augmented.rs` | 273 | float=4, async=5 |
| `vector/src/diskann/search.rs` | 398 | float=13, async=1 |
| `vector/src/diskann/tests/build_search_tests.rs` | 327 | float=6, async=17, panic=13 |
| `vector/src/diskann/tests/config_tests.rs` | 242 | float=3, async=17, panic=17, alt_spec=1 |
| `vector/src/diskann/tests/mod.rs` | 9 |  |
| `vector/src/diskann/tests/persistence_tests.rs` | 273 | async=20, panic=40 |
| `vector/src/diskann/tests/recall_tests.rs` | 168 | float=4, async=8, panic=8 |
| `vector/src/diskann/tests/wal_tests.rs` | 116 | async=12, panic=1 |
| `vector/src/diskann/types.rs` | 302 | float=3, panic=12 |
| `vector/src/hnsw/acorn_filtered.rs` | 297 | float=4 |
| `vector/src/hnsw/adaptive_ef.rs` | 310 | float=7 |
| `vector/src/hnsw/arena.rs` | 429 | panic=1 |
| `vector/src/hnsw/batch.rs` | 89 |  |
| `vector/src/hnsw/clock.rs` | 49 |  |
| `vector/src/hnsw/config.rs` | 228 | float=5 |
| `vector/src/hnsw/core_insert/apply.rs` | 72 | float=1 |
| `vector/src/hnsw/core_insert/compute.rs` | 376 | float=2 |
| `vector/src/hnsw/core_insert/connectivity.rs` | 114 | float=3 |
| `vector/src/hnsw/core_insert/mod.rs` | 16 |  |
| `vector/src/hnsw/core_rebuild/backoff.rs` | 132 |  |
| `vector/src/hnsw/core_rebuild/full.rs` | 436 | float=2 |
| `vector/src/hnsw/core_rebuild/mod.rs` | 138 | float=5, panic=5 |
| `vector/src/hnsw/core_rebuild/region.rs` | 118 |  |
| `vector/src/hnsw/core_rebuild/seq_replay.rs` | 195 |  |
| `vector/src/hnsw/core_rebuild/snapshot.rs` | 441 | float=19 |
| `vector/src/hnsw/core_search/adaptive.rs` | 263 | float=3, async=3 |
| `vector/src/hnsw/core_search/filtered.rs` | 237 | float=1, async=1 |
| `vector/src/hnsw/core_search/layer.rs` | 174 | float=2 |
| `vector/src/hnsw/core_search/mod.rs` | 20 |  |
| `vector/src/hnsw/core_search/rerank.rs` | 42 | float=2 |
| `vector/src/hnsw/core_search/validate.rs` | 47 | float=2 |
| `vector/src/hnsw/deletion.rs` | 474 | float=1, alt_spec=1 |
| `vector/src/hnsw/mod.rs` | 79 | alt_spec=1 |
| `vector/src/hnsw/sq8_bias.rs` | 108 | float=9, serde=1 |
| `vector/src/hnsw/staging.rs` | 63 | float=1 |
| `vector/src/hnsw/types/builder.rs` | 192 | float=2, hr6=1 |
| `vector/src/hnsw/types/core.rs` | 160 | float=1 |
| `vector/src/hnsw/types/data.rs` | 71 | float=3 |
| `vector/src/hnsw/types/guards.rs` | 37 |  |
| `vector/src/hnsw/types/index.rs` | 261 | float=3 |
| `vector/src/hnsw/types/mod.rs` | 22 |  |
| `vector/src/hnsw/types/persist.rs` | 291 | float=3 |
| `vector/src/hnsw/types/seqlog.rs` | 22 |  |
| `vector/src/partial_rebuild.rs` | 377 | float=11, panic=1, alt_spec=4 |
| `vector/src/persistence/atomic.rs` | 58 | std_fs=1, alt_spec=2 |
| `vector/src/persistence/header.rs` | 468 | float=22 |
| `vector/src/persistence/io.rs` | 60 | std_fs=8 |
| `vector/src/persistence/mmap.rs` | 273 | async=1 |
| `vector/src/persistence/mod.rs` | 19 |  |
| `vector/src/persistence/node.rs` | 94 |  |
| `vector/src/persistence/tests/atomic_tests.rs` | 34 | panic=7, std_fs=2 |
| `vector/src/persistence/tests/header_tests.rs` | 47 |  |
| `vector/src/persistence/tests/mmap_tests.rs` | 447 | float=10, async=8, panic=7, std_fs=7 |
| `vector/src/persistence/tests/mod.rs` | 10 |  |
| `vector/src/persistence/tests/node_tests.rs` | 39 |  |
| `vector/src/quantize_rabitq.rs` | 525 | float=48, serde=1, zeit_zufall=2, panic=4 |
| `vector/tests/acorn_hnsw_differential.rs` | 299 | float=5, async=6, zeit_zufall=2, panic=5, std_fs=1, hub=2 |
| `vector/tests/acorn_naive_recall.rs` | 97 | float=2, panic=1, hub=1 |
| `vector/tests/acorn_selectivity_edge_cases_test.rs` | 92 | float=1, async=2, panic=11, hub=2 |
| `vector/tests/connectivity_regression.rs` | 379 |  |
| `vector/tests/diskann_corruption_fallback_test.rs` | 121 | async=8, panic=17, hub=3 |
| `vector/tests/diskann_docid128_tombstones.rs` | 93 | async=2, hub=3 |
| `vector/tests/diskann_fault_injection_test.rs` | 86 | float=2, async=5, panic=10, hub=2 |
| `vector/tests/diskann_recall_test.rs` | 257 | float=27, async=4, zeit_zufall=3, panic=13, hub=3 |
| `vector/tests/diskann_search_filtered.rs` | 217 | float=2, async=15, panic=12, hub=3 |
| `vector/tests/hnsw_edge_cases.rs` | 371 | float=8, async=26, hub=2 |
| `vector/tests/hnsw_layer_determinism.rs` | 133 | float=7, async=6, panic=22, hub=2 |
| `vector/tests/hnsw_rebuild_connectivity_test.rs` | 76 | float=4, async=2, panic=6, hub=2 |
| `vector/tests/hnsw_rebuild_search_consistency.rs` | 158 | float=1, async=6, panic=10, hub=3 |
| `vector/tests/hnsw_scanner_purge_test.rs` | 272 | float=1, async=10, panic=43, std_fs=2, hub=4 |
| `vector/tests/hnsw_seed_determinism.rs` | 348 | float=17, async=6, panic=34, hub=3 |
| `vector/tests/hnsw_snapshot_delete_test.rs` | 142 | async=4, panic=19, hub=2 |
| `vector/tests/hnsw_tests.rs` | 448 | float=7, async=14, panic=45, hub=1 |
| `vector/tests/mmap_malformed_data_test.rs` | 208 |  |
| `vector/tests/rabitq_calibration_recall.rs` | 151 | float=20, async=2, panic=3 |
| `vector/tests/rabitq_recall_calibration.rs` | 111 | float=6, hub=1 |
| `vector/tests/rabitq_recall_calibration_test.rs` | 24 | float=6, panic=4 |
| `vector/tests/recall.rs` | 88 | float=8, async=2, panic=8, hub=2 |
| `vector/tests/recall_audit.rs` | 464 | float=25, async=13, zeit_zufall=4, panic=44, hub=2 |
| `vector/tests/recall_vs_brute_force.rs` | 260 | float=18, async=12, zeit_zufall=4, panic=1, hub=2 |
| `vector/tests/sq8_bias_calibration.rs` | 31 | serde=2, panic=2, hub=1 |
| `vector/tests/sq8_error_bound_proptest.rs` | 38 | panic=4 |
| `vector/tests/sq8_rerank_recall.rs` | 195 | float=25, async=2, panic=11, hub=3 |
