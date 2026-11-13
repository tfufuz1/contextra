# AGENTS.md — `contextra-vector`

> Werkzeug/Crate-Kontext für Agenten. Der obere Teil ist generiert; manuelle Notizen unterhalb ergänzen.

<!-- BEGIN GENERATED: crate_inventory.py -->

## Inventar (generiert)

- **Pfad:** `crates/contextra-vector` · **Ring:** 0 · **Version:** 0.1.0
- **Beschreibung:** HNSW vector index with SIMD distance computation for Contextra
- **src:** 107 Dateien, 15814 LOC · **unsafe:** 0 Vorkommen, Policy `forbid`
- **Interne Abhängigkeiten (normal, 7):** `contextra-env`, `contextra-ids`, `contextra-mvcc`, `contextra-ports`, `contextra-simd`, `contextra-sys`, `contextra-types`
- **Interne Dev/Build-Abhängigkeiten (0):** –
- **Externe Abhängigkeiten (normal, 11):** `ahash`, `blake3`, `memmap2`, `parking_lot`, `rand`, `roaring`, `serde`, `subtle`, `thiserror`, `tracing`, `zeroize`
- **Reverse-Abhängigkeiten (2):** `contextra-bench`, `contextra-db`
- **Kopplung an Hub-Crates:** ausgehend `contextra-ports`, `contextra-types`; eingehend `contextra-db`
- **Features (19):** `default`, `diskann`, `docid-128`, `experimental`, `experimental-all`, `experimental-diskann`, `experimental-predicate-augmented-search`, `experimental-rabitq`, `graph`, `inspect`, `mmap`, `partial-index-rebuild`, `ports-compat`, `quantize-rabitq`, `serde`, `simd`, `std`, `tracing`, `workspace`

### Öffentliche API (grob, aus src/)
- struct (48): `AdaptiveEfPolicy`, `AdaptiveEfStateMachine`, `AdaptiveEfStats`, `BacklinkTable`, `BackoffTracker`, `BatchContext`, `Candidate`, `ComputePool`, `CosineSimilarityPartsU8`, `DeletionStats`, `DiskAnnConfig`, `DiskAnnHmac`, `DiskAnnIndex`, `ExternalId`, `GenericHnswIndex` … (+33)
- enum (9): `AcornError`, `DiskAnnFallbackPolicy`, `DistanceMetric`, `Error`, `Metric`, `RebuildStatus`, `VectorData`, `VectorError`, `VectorStagingOp`
- trait (11): `BlockTransform`, `Clock`, `DistanceFn`, `DistanceKernel`, `FilteredIndex`, `GhostFreeVectorIndex`, `Index`, `IntoDocId`, `Maintain`, `Rng`, `VectorIndex`
- fn (225): `active_jobs`, `advance_nanos`, `align_capacity`, `all_doc_ids_and_layers`, `all_doc_ids_from_map`, `allocate_node`, `apply_insert`, `asymmetric_dist`, `asymmetric_distance`, `backoff_alert_threshold`, `backoff_base_delay`, `backoff_max_delay`, `build`, `build_sync`, `build_to_path_sync` … (+210)
- type (5): `BacklinkKey`, `FixedHashMap`, `FixedHashSet`, `HnswIndex`, `Result`
- const (14): `ARENA_ALIGNMENT_BYTES`, `ARENA_ALIGNMENT_U32`, `DEFAULT_LAYER_SEED`, `DEFAULT_P_HIGH`, `DEFAULT_P_LOW`, `DEFAULT_QUEUE_CAPACITY_PER_WORKER`, `DEFAULT_VECTOR_STREAM_BATCH_SIZE`, `HNSW_MAGIC`, `HNSW_REBUILD_DELETION_RATIO`, `HNSW_VERSION`, `MAX_SEARCH_K`, `SENTINEL_NO_ENTRY_POINT`, `SIZE`, `SQ8_MAX_ERROR_FACTOR_PER_DIM`
- mod (43): `acorn`, `adaptive_ef`, `arena`, `atomic`, `atomic_tests`, `backoff`, `candidate_stream`, `casting`, `clock`, `compat`, `compute_pool`, `contract`, `decode`, `deletion`, `diskann` … (+28)
- pub use: `adaptive_ef::{AdaptiveEfPolicy, AdaptiveEfStateMachine, AdaptiveEfStats}`, `arena::{BacklinkTable, HnswArena}`, `atomic::write_atomic`, `backoff::BackoffTracker`, `batch::{BatchContext, NeighborBacklink, PreparedInsert}`, `clock::MonotonicCounterClock`, `compat::candidate_stream::{VectorCandidateStream, DEFAULT_VECTOR_STREAM_BATCH_SIZE}`, `compute_pool::ComputePool`, `config::{DiskAnnConfig, DiskAnnFallbackPolicy}`, `config::{HnswConfig, HnswConfigBuilder}` … (+35)

Regeneriert mit `python3 tools/crate_inventory.py agents`. Manuelle Notizen stehen außerhalb dieses Blocks.

<!-- END GENERATED: crate_inventory.py -->`.

<!-- END GENERATED: crate_inventory.py -->
