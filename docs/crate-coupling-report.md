# Crate-Kopplungsbericht

- Erzeugt: 2026-11-27 02:22
- Workspace-Root: `/home/freddy/Projekte/contextra`
- Analysierte Crates: 19 · Komponenten-Tiefe: 1 · Zeilen (ohne Leerzeilen): 238.490
- Legende: **prod** = src/bin/build ohne `cfg(test)`; **dev** = tests/benches/examples/fuzz und `cfg(test)`-Code.
- Hinweis: Textuelle Analyse ohne Compiler. Glob-Importe (`*`) sind nicht auflösbar. Siehe Kopf des Skripts für Grenzen.

## 1. Überblick

| Crate | Ring | Zeilen | Fan-out prod | Fan-in prod | Cargo-Deps intern (n/d/b) | Pub-Items | API-Lecks | Zyklus |
|---|---|---:|---:|---:|---|---:|---:|---|
| [contextra](#contextra) | 4 | 3352 | 3 | 1 | 3/0/0 | 25 | 14 | – |
| [contextra-bench](#contextra-bench) | tooling | 3936 | 3 | 0 | 7/1/0 | 49 | 1 | – |
| [contextra-checkpoint](#contextra-checkpoint) | 1 | 8557 | 2 | 1 | 2/1/0 | 1 | 0 | – |
| [contextra-crypto](#contextra-crypto) | 0 | 16231 | 2 | 2 | 2/1/0 | 110 | 10 | – |
| [contextra-db](#contextra-db) | 3 | 11466 | 3 | 1 | 3/7/0 | 20 | 6 | – |
| [contextra-durable-fs](#contextra-durable-fs) | 0 | 2750 | 0 | 0 | 0/0/0 | 49 | 0 | – |
| [contextra-engine](#contextra-engine) | 3 | 25640 | 9 | 1 | 9/2/0 | 97 | 33 | – |
| [contextra-graph](#contextra-graph) | 0 | 28667 | 2 | 1 | 2/0/0 | 138 | 22 | – |
| [contextra-mcp](#contextra-mcp) | 4 | 7712 | 5 | 0 | 5/0/0 | 32 | 5 | – |
| [contextra-mvcc](#contextra-mvcc) | 0 | 6532 | 1 | 4 | 2/1/0 | 22 | 0 | – |
| [contextra-ports](#contextra-ports) | 0 | 6796 | 1 | 14 | 1/0/0 | 145 | 1 | – |
| [contextra-rank](#contextra-rank) | 0 | 7660 | 2 | 3 | 2/0/0 | 39 | 7 | – |
| [contextra-simd](#contextra-simd) | 0 | 5137 | 1 | 0 | 1/0/0 | 35 | 1 | – |
| [contextra-store](#contextra-store) | 1 | 48169 | 5 | 1 | 5/1/0 | 142 | 5 | – |
| [contextra-sys](#contextra-sys) | 0 | 2321 | 0 | 2 | 0/0/0 | 29 | 0 | – |
| [contextra-testkit](#contextra-testkit) | tooling | 4856 | 2 | 0 | 2/0/0 | 23 | 9 | – |
| [contextra-text](#contextra-text) | 0 | 15219 | 2 | 1 | 2/0/0 | 78 | 8 | – |
| [contextra-types](#contextra-types) | 0 | 8287 | 0 | 14 | 0/0/0 | 74 | 0 | – |
| [contextra-vector](#contextra-vector) | 0 | 25202 | 4 | 1 | 4/0/0 | 100 | 9 | – |

## 2. Workspace-weite Auffälligkeiten

### 2.1 Zyklen

- Kein Zyklus im prod-Code-Graphen gefunden.
- Kein Zyklus in den normalen Cargo-Abhängigkeiten.

### 2.2 Abweichungen Cargo.toml ↔ Code

| Crate | Ziel | Deklariert als | Befund |
|---|---|---|---|
| `contextra-bench` | `contextra-db` | normal | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |
| `contextra-bench` | `contextra-store` | normal | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |
| `contextra-bench` | `contextra-text` | normal | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |
| `contextra-bench` | `contextra-vector` | normal | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |
| `contextra-checkpoint` | `contextra-types` | normal | nur in Tests/Benches genutzt, aber als normale Abhängigkeit deklariert → dev-dependency? |
| `contextra-checkpoint` | `contextra-mvcc` | – | genutzt, aber nicht in Cargo.toml deklariert (cfg-/Feature-Pfad oder Fehler) |
| `contextra-db` | `contextra-testkit` | dev | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |
| `contextra-mvcc` | `contextra-types` | normal | nur in Tests/Benches genutzt, aber als normale Abhängigkeit deklariert → dev-dependency? |

### 2.3 Schwache Kanten (billig zu trennen)

Kanten im prod-Code mit höchstens 3 verschiedenen Symbolen. Reihenfolge: wenigste Symbole, dann wenigste Fundstellen.

| Von | Nach | Symbole | Fundstellen | Dateien | Symbole (prod) |
|---|---|---:|---:|---:|---|
| `contextra-db` | `contextra-ports` | 1 | 1 | 1 | `QueryRewriter` |
| `contextra-checkpoint` | `contextra-mvcc` | 1 | 1 | 1 | `SnapshotRegistry` |
| `contextra-mcp` | `contextra-rank` | 1 | 1 | 1 | `fusion::ProvenanceRecord` |
| `contextra-bench` | `contextra-ports` | 1 | 1 | 1 | `VectorIndex` |
| `contextra-bench` | `contextra-rank` | 1 | 1 | 1 | `RankError` |
| `contextra-engine` | `contextra-mvcc` | 1 | 1 | 1 | `tx_buffer::TxBuffer` |
| `contextra-vector` | `contextra-sys` | 1 | 2 | 2 | `mmap_readonly`×2 |
| `contextra-rank` | `contextra-ports` | 1 | 2 | 1 | `Clock`×2 |
| `contextra-mvcc` | `contextra-ports` | 1 | 3 | 1 | `MetricsSink`×3 |
| `contextra-bench` | `contextra-types` | 2 | 2 | 1 | `Result`, `ContextraError::CapabilityUnsupported` |
| `contextra-store` | `contextra-sys` | 2 | 3 | 2 | `mmap_readonly`×2, `set_restrictive_file_acl` |
| `contextra-mcp` | `contextra-ports` | 2 | 3 | 1 | `plugin::PluginRegistry`×2, `EmbeddingProvider` |
| `contextra-ports` | `contextra-types` | 2 | 3 | 3 | `RequestContext`×2, `ids::ErasureId` |
| `contextra-mcp` | `contextra-crypto` | 2 | 4 | 1 | `CryptoKey`×3, `CryptoKey::try_new` |
| `contextra-engine` | `contextra-text` | 2 | 4 | 3 | `inverted::InvertedIndex`×2, `Language`×2 |
| `contextra-text` | `contextra-ports` | 2 | 8 | 5 | `StorageEngine`×6, `scanner`×2 |
| `contextra-engine` | `contextra-checkpoint` | 3 | 4 | 3 | `InstanceOrphanRegistry`×2, `InstanceOrphanRegistry::new`, `(crate)` |
| `contextra-mcp` | `contextra-types` | 3 | 6 | 3 | `ContextraError`×4, `ContextraError::InvalidInput`, `ContextraError::NotFound` |

**Nur in Tests/Benches/Beispielen** (für das Produkt irrelevant, aber sie koppeln den Test-Build):

| Von | Nach | Fundstellen | Symbole |
|---|---|---:|---|
| `contextra-bench` | `contextra-sys` | 1 | `ensure_fd_limit` |
| `contextra-checkpoint` | `contextra-testkit` | 3 | `InMemoryStorageEngine`×2, `FaultVfs` |
| `contextra-checkpoint` | `contextra-types` | 13 | `TxId`×4, `Result`×3, `ContextraError`×2, `ContextraError::Internal`×2 … (+2) |
| `contextra-crypto` | `contextra-testkit` | 5 | `mint_witness_for_tests`×4, `FaultVfs` |
| `contextra-db` | `contextra-checkpoint` | 4 | `PersistentCheckpointStore`×2, `clear_all_orphaned_checkpoints`, `get_orphaned_checkpoints` |
| `contextra-db` | `contextra-graph` | 13 | `CsrGraph`×9, `csr::CsrGraph`×2, `path_rag::PathRAGEngine`, `PathRAGEngine` |
| `contextra-db` | `contextra-rank` | 1 | `IsotonicCalibrator::new` |
| `contextra-db` | `contextra-store` | 8 | `LsmConfig`×3, `LsmStorage::new`×3, `lsm`, `LsmStorage` |
| `contextra-db` | `contextra-text` | 10 | `Language::English`×6, `Language`×2, `bm25::score_term`, `inverted::InvertedIndex` |
| `contextra-db` | `contextra-vector` | 4 | `HnswConfig`×2, `HnswIndex`, `HnswIndex::try_new` |
| `contextra-engine` | `contextra-testkit` | 1 | `InMemoryStorageEngine` |
| `contextra-mvcc` | `contextra-testkit` | 6 | `ManualClock`×6 |
| `contextra-mvcc` | `contextra-types` | 7 | `TxId`×5, `DocId`, `types` |
| `contextra-store` | `contextra-testkit` | 8 | `ManualClock`×4, `ReferenceModel`×4 |

### 2.4 Hub-Symbole (Kandidaten für types/ports)

Symbole, die aus einem Nicht-Basis-Crate von mindestens 3 verschiedenen Crates im prod-Code importiert werden. Wer sie in ein schmales Basis-Crate verschiebt, kann viele Kanten kappen.

Keine.

### 2.5 API-Lecks

Öffentliche Items, deren Signatur oder öffentliche Felder Typen anderer Workspace-Crates enthalten, sowie `pub use` fremder Crates. Diese Stellen machen eine Entkopplung am schwersten, weil Nutzer des Crates die fremde Abhängigkeit mitsehen.

| Crate | Komponente | Item | Fremde Crates | Fundstelle |
|---|---|---|---|---|
| `contextra` | `boot` | `fn execute_boot_sequence` | `contextra-types` | `src/boot.rs:39` |
| `contextra` | `collection_profile` | `const DEFAULT_AUTO_EXTRACTION_MODE` | `contextra-types` | `src/collection_profile.rs:55` |
| `contextra` | `collection_profile` | `const ENTERPRISE_REGULATED_AUTO_EXTRACTION_DEFAULT` | `contextra-types` | `src/collection_profile.rs:63` |
| `contextra` | `collection_profile` | `struct CollectionProfile` | `contextra-types` | `src/collection_profile.rs:82` |
| `contextra` | `collection_profile` | `pub use contextra_types::types::domain::AutoExtractionMode` | `contextra-types` | `src/collection_profile.rs:34` |
| `contextra` | `(crate-root)` | `fn open` | `contextra-types` | `src/lib.rs:127` |
| `contextra` | `(crate-root)` | `fn open_with_config` | `contextra-types` | `src/lib.rs:159` |
| `contextra` | `(crate-root)` | `pub use contextra_types::error::ContextraError` | `contextra-types` | `src/lib.rs:53` |
| `contextra` | `(crate-root)` | `pub use contextra_types::DistanceMetric` | `contextra-types` | `src/lib.rs:55` |
| `contextra` | `performance_profile` | `enum DurabilityMode` | `contextra-db` | `src/performance_profile.rs:37` |
| `contextra` | `performance_profile` | `enum DurabilityConfigError` | `contextra-db` | `src/performance_profile.rs:52` |
| `contextra` | `performance_profile` | `struct ResolvedProfileConfig` | `contextra-db`, `contextra-ports` | `src/performance_profile.rs:117` |
| `contextra` | `performance_profile` | `enum PerformanceProfileError` | `contextra-ports` | `src/performance_profile.rs:127` |
| `contextra` | `performance_profile` | `pub use contextra_ports::license::VectorDeleteMode` | `contextra-ports` | `src/performance_profile.rs:33` |
| `contextra-bench` | `matrix` | `fn fuse_via_rank_rrf` | `contextra-rank` | `src/matrix.rs:102` |
| `contextra-crypto` | `deletion_proof::chain` | `fn verify_proof_sequence` | `contextra-types` | `src/deletion_proof/chain.rs:12` |
| `contextra-crypto` | `deletion_proof::types` | `fn serialize` | `contextra-ports` | `src/deletion_proof/types.rs:235` |
| `contextra-crypto` | `deletion_proof::types` | `fn deserialize` | `contextra-ports` | `src/deletion_proof/types.rs:250` |
| `contextra-crypto` | `deletion_proof::types` | `struct LayerWitnessMeasurement` | `contextra-ports` | `src/deletion_proof/types.rs:267` |
| `contextra-crypto` | `deletion_proof::verifier` | `enum VerificationErrorReason` | `contextra-types` | `src/deletion_proof/verifier.rs:13` |
| `contextra-crypto` | `error` | `enum CryptoError` | `contextra-types` | `src/error.rs:19` |
| `contextra-crypto` | `kv_cipher` | `struct EncryptedKvLayer` | `contextra-types` | `src/kv_cipher.rs:52` |
| `contextra-crypto` | `kv_cipher` | `pub use contextra_types::ModelFingerprint` | `contextra-types` | `src/kv_cipher.rs:42` |
| `contextra-crypto` | `kv_segment::segment` | `struct KvSegment` | `contextra-types` | `src/kv_segment/segment.rs:42` |
| `contextra-crypto` | `kv_segment::store` | `type SpillHandler` | `contextra-types` | `src/kv_segment/store.rs:26` |
| `contextra-db` | `(crate-root)` | `pub use contextra_engine` | `contextra-engine` | `src/lib.rs:72` |
| `contextra-db` | `(crate-root)` | `pub use contextra_engine::DurabilityMode` | `contextra-engine` | `src/lib.rs:73` |
| `contextra-db` | `(crate-root)` | `pub use contextra_engine::decay_controller::*` | `contextra-engine` | `src/lib.rs:89` |
| `contextra-db` | `(crate-root)` | `pub use contextra_engine::fusion` | `contextra-engine` | `src/lib.rs:91` |
| `contextra-db` | `(crate-root)` | `pub use contextra_types::DistanceMetric` | `contextra-types` | `src/lib.rs:120` |
| `contextra-db` | `multistep` | `pub use contextra_ports::QueryRewriter` | `contextra-ports` | `src/multistep.rs:28` |
| `contextra-engine` | `background_workers::expiry_workers` | `fn start_decay_cleanup_worker` | `contextra-ports` | `src/background_workers/expiry_workers.rs:85` |
| `contextra-engine` | `collection::crud::auto_extraction` | `pub use contextra_types::AutoExtractionMode` | `contextra-types` | `src/collection/crud/auto_extraction.rs:18` |
| `contextra-engine` | `collection::maintenance` | `pub use contextra_graph::percolation::PercolationConfig` | `contextra-graph` | `src/collection/maintenance.rs:878` |
| `contextra-engine` | `collection` | `fn compute_default_importance` | `contextra-types` | `src/collection/mod.rs:124` |
| `contextra-engine` | `collection` | `struct Collection` | `contextra-store`, `contextra-vector` | `src/collection/mod.rs:272` |
| `contextra-engine` | `collection::query_builder::scope` | `struct ScopeConstraint` | `contextra-types` | `src/collection/query_builder/scope.rs:19` |
| `contextra-engine` | `collection::query_builder::strategy` | `enum SearchStrategy` | `contextra-types` | `src/collection/query_builder/strategy.rs:11` |
| `contextra-engine` | `extraction::open_ie` | `fn extract_triples` | `contextra-ports` | `src/extraction/open_ie.rs:80` |
| `contextra-engine` | `fusion` | `fn weights_to_signal_factors` | `contextra-types` | `src/fusion.rs:16` |
| `contextra-engine` | `fusion` | `fn fuse_search_results_with_strategy` | `contextra-types` | `src/fusion.rs:26` |
| `contextra-engine` | `fusion` | `fn fuse_search_results_with_signal_strategies` | `contextra-types` | `src/fusion.rs:45` |
| `contextra-engine` | `(crate-root)` | `type ConsolidationLauncher` | `contextra-store` | `src/lib.rs:95` |
| `contextra-engine` | `(crate-root)` | `fn register_consolidation_launcher` | `contextra-store` | `src/lib.rs:112` |
| `contextra-engine` | `(crate-root)` | `struct ContextraStats` | `contextra-ports` | `src/lib.rs:145` |
| `contextra-engine` | `(crate-root)` | `struct ContextraConfig` | `contextra-ports`, `contextra-store`, `contextra-types` | `src/lib.rs:211` |
| `contextra-engine` | `(crate-root)` | `type TenantCollectionMap` | `contextra-store`, `contextra-types` | `src/lib.rs:350` |
| `contextra-engine` | `(crate-root)` | `pub use contextra_ports::TextEmbeddingEngine` | `contextra-ports` | `src/lib.rs:19` |
| `contextra-engine` | `(crate-root)` | `pub use contextra_ports::VectorDeleteMode` | `contextra-ports` | `src/lib.rs:20` |
| `contextra-engine` | `(crate-root)` | `pub use contextra_store::lsm::DurabilityMode` | `contextra-store` | `src/lib.rs:21` |
| `contextra-engine` | `(crate-root)` | `pub use contextra_checkpoint` | `contextra-checkpoint` | `src/lib.rs:89` |
| `contextra-engine` | `(crate-root)` | `pub use contextra_text::Language` | `contextra-text` | `src/lib.rs:90` |
| `contextra-engine` | `(crate-root)` | `pub use contextra_ports::DriftStatusProvider` | `contextra-ports` | `src/lib.rs:161` |
| `contextra-engine` | `(crate-root)` | `pub use contextra_types::DistanceMetric` | `contextra-types` | `src/lib.rs:385` |
| `contextra-engine` | `metrics_names` | `fn record_histogram_safe` | `contextra-ports` | `src/metrics_names.rs:33` |
| `contextra-engine` | `metrics_names` | `fn record_gauge_safe` | `contextra-ports` | `src/metrics_names.rs:45` |
| `contextra-engine` | `metrics_names` | `fn record_counter_safe` | `contextra-ports` | `src/metrics_names.rs:57` |
| `contextra-engine` | `temporal_filter` | `type FusionResult` | `contextra-rank` | `src/temporal_filter.rs:22` |
| `contextra-engine` | `temporal_filter` | `struct ValidityWindow` | `contextra-types` | `src/temporal_filter.rs:25` |
| `contextra-engine` | `temporal_filter` | `fn sort_candidates_deterministic` | `contextra-rank` | `src/temporal_filter.rs:162` |
| `contextra-engine` | `temporal_filter` | `fn apply_temporal_validity_filter_with_clock` | `contextra-ports`, `contextra-rank`, `contextra-types` | `src/temporal_filter.rs:173` |
| `contextra-engine` | `temporal_filter` | `fn apply_temporal_validity_filter` | `contextra-rank`, `contextra-types` | `src/temporal_filter.rs:187` |
| `contextra-engine` | `temporal_filter` | `fn apply_temporal_validity_filter_at` | `contextra-rank` | `src/temporal_filter.rs:207` |
| `contextra-engine` | `transaction::db_transaction` | `struct DbTransaction` | `contextra-vector` | `src/transaction/db_transaction.rs:18` |
| `contextra-graph` | `apprh::diffusion` | `fn run_apprh_push` | `contextra-types` | `src/apprh/diffusion.rs:23` |
| `contextra-graph` | `apprh` | `fn apprh_local` | `contextra-types` | `src/apprh/mod.rs:35` |
| `contextra-graph` | `apprh` | `fn forward_push_apprh` | `contextra-types` | `src/apprh/mod.rs:51` |
| `contextra-graph` | `apprh::shadow` | `struct ApprhShadowComparison` | `contextra-types` | `src/apprh/shadow.rs:30` |
| `contextra-graph` | `apprh::shadow` | `fn shadow_compare_forward_push_vs_apprh` | `contextra-types` | `src/apprh/shadow.rs:54` |
| `contextra-graph` | `edge_reinforcement_buffer` | `struct CooccurrenceSignal` | `contextra-types` | `src/edge_reinforcement_buffer.rs:26` |
| `contextra-graph` | `edge_reinforcement_buffer` | `struct TraversalSignal` | `contextra-types` | `src/edge_reinforcement_buffer.rs:36` |
| `contextra-graph` | `hyperedge_suggest` | `fn validate_candidates_with_llm` | `contextra-ports` | `src/hyperedge_suggest.rs:118` |
| `contextra-graph` | `(crate-root)` | `trait GraphIndexExt` | `contextra-ports` | `src/lib.rs:132` |
| `contextra-graph` | `path_rag::k_path` | `struct KPathResult` | `contextra-types` | `src/path_rag/k_path.rs:35` |
| `contextra-graph` | `path_rag` | `struct GraphPath` | `contextra-types` | `src/path_rag/mod.rs:31` |
| `contextra-graph` | `path_rag` | `fn forward_push_ppr` | `contextra-types` | `src/path_rag/mod.rs:96` |
| `contextra-graph` | `path_rag` | `pub use contextra_types::EntityId` | `contextra-types` | `src/path_rag/mod.rs:25` |
| `contextra-graph` | `percolation` | `fn find_rebonding_candidates` | `contextra-ports`, `contextra-types` | `src/percolation.rs:79` |
| `contextra-graph` | `ppr::shadow_hook` | `fn log_tl_hfd_vs_baseline_discrepancy` | `contextra-types` | `src/ppr/shadow_hook.rs:26` |
| `contextra-graph` | `ppr_stream` | `struct PprCandidateStream` | `contextra-ports` | `src/ppr_stream.rs:23` |
| `contextra-graph` | `tl_hfd::diffusion` | `fn run_diffusion` | `contextra-types` | `src/tl_hfd/diffusion.rs:28` |
| `contextra-graph` | `tl_hfd::lovasz` | `fn truncate_participants` | `contextra-types` | `src/tl_hfd/lovasz.rs:14` |
| `contextra-graph` | `tl_hfd::lovasz` | `fn compute_lovasz_extension` | `contextra-types` | `src/tl_hfd/lovasz.rs:56` |
| `contextra-graph` | `tl_hfd` | `fn tl_hfd_local` | `contextra-types` | `src/tl_hfd/mod.rs:31` |
| `contextra-graph` | `tl_hfd::shadow` | `struct ShadowComparison` | `contextra-types` | `src/tl_hfd/shadow.rs:55` |
| `contextra-graph` | `tl_hfd::shadow` | `fn shadow_compare_forward_push_vs_tl_hfd` | `contextra-types` | `src/tl_hfd/shadow.rs:79` |
| `contextra-mcp` | `config` | `fn verify_embedding_provider` | `contextra-types` | `src/config.rs:72` |
| `contextra-mcp` | `config` | `fn create_embedding_provider` | `contextra-types` | `src/config.rs:266` |
| `contextra-mcp` | `config` | `fn create_llm_text_generator` | `contextra-types` | `src/config.rs:377` |
| `contextra-mcp` | `explain` | `struct ExplainResponse` | `contextra-rank` | `src/explain.rs:23` |
| `contextra-mcp` | `server` | `struct McpServer` | `contextra`, `contextra-ports` | `src/server.rs:13` |
| `contextra-ports` | `observer` | `struct ErasureEvent` | `contextra-types` | `src/observer.rs:48` |
| `contextra-rank` | `calibration::isotonic` | `trait Clock` | `contextra-ports` | `src/calibration/isotonic.rs:14` |
| `contextra-rank` | `fusion::normalized` | `fn weights_to_signal_factors_types_compat` | `contextra-types` | `src/fusion/normalized.rs:197` |
| `contextra-rank` | `local_types` | `struct DocId` | `contextra-types` | `src/local_types.rs:9` |
| `contextra-rank` | `local_types` | `struct ConfigFingerprint` | `contextra-types` | `src/local_types.rs:68` |
| `contextra-rank` | `local_types` | `enum FusionStrategy` | `contextra-types` | `src/local_types.rs:121` |
| `contextra-rank` | `local_types` | `struct SignalFusionStrategies` | `contextra-types` | `src/local_types.rs:183` |
| `contextra-rank` | `local_types` | `struct FusionWeights` | `contextra-types` | `src/local_types.rs:238` |
| `contextra-simd` | `metric` | `enum DistanceMetric` | `contextra-types` | `src/metric.rs:3` |
| `contextra-store` | `lsm::engine` | `struct LsmStorage` | `contextra-mvcc` | `src/lsm/engine.rs:33` |
| `contextra-store` | `lsm::observer` | `struct CommitContext` | `contextra-types` | `src/lsm/observer.rs:66` |
| `contextra-store` | `lsm::tests` | `pub use contextra_types::TOMBSTONE_BIT` | `contextra-types` | `src/lsm/tests.rs:7` |
| `contextra-store` | `wal` | `struct WalConfig` | `contextra-crypto`, `contextra-ports` | `src/wal/mod.rs:373` |
| `contextra-store` | `wal` | `pub use contextra_crypto::crypto::KeyManager` | `contextra-crypto` | `src/wal/mod.rs:343` |
| `contextra-testkit` | `conformance::clock` | `fn run_clock_suite` | `contextra-ports` | `src/conformance/clock.rs:24` |
| `contextra-testkit` | `conformance::graph` | `fn run_graph_index_suite` | `contextra-ports` | `src/conformance/graph.rs:30` |
| `contextra-testkit` | `conformance::metrics` | `fn run_metrics_sink_suite` | `contextra-ports` | `src/conformance/metrics.rs:22` |
| `contextra-testkit` | `conformance::rng` | `fn run_rng_suite` | `contextra-ports` | `src/conformance/rng.rs:26` |
| `contextra-testkit` | `conformance::storage` | `fn run_storage_engine_suite` | `contextra-ports` | `src/conformance/storage.rs:38` |
| `contextra-testkit` | `conformance::text` | `fn run_text_index_suite` | `contextra-ports` | `src/conformance/text.rs:30` |
| `contextra-testkit` | `conformance::vector` | `fn run_vector_index_suite` | `contextra-ports` | `src/conformance/vector.rs:36` |
| `contextra-testkit` | `conformance::vfs` | `fn run_vfs_suite` | `contextra-ports` | `src/conformance/vfs.rs:24` |
| `contextra-testkit` | `(crate-root)` | `fn mint_witness_for_tests` | `contextra-ports` | `src/lib.rs:125` |
| `contextra-text` | `inverted::purge` | `struct TextPurger` | `contextra-ports` | `src/inverted/purge.rs:94` |
| `contextra-text` | `inverted::purge` | `fn text_purger` | `contextra-ports` | `src/inverted/purge.rs:493` |
| `contextra-text` | `(crate-root)` | `struct Bm25Scorer` | `contextra-ports` | `src/lib.rs:93` |
| `contextra-text` | `stream` | `struct Bm25CandidateStream` | `contextra-ports` | `src/stream.rs:42` |
| `contextra-text` | `types` | `struct DocId` | `contextra-types` | `src/types.rs:5` |
| `contextra-text` | `types` | `struct DocId` | `contextra-types` | `src/types.rs:11` |
| `contextra-text` | `types` | `struct TextHit` | `contextra-types` | `src/types.rs:101` |
| `contextra-text` | `wand` | `fn block_max_wand_search` | `contextra-ports` | `src/wand.rs:261` |
| `contextra-vector` | `candidate_stream` | `struct VectorCandidateStream` | `contextra-ports` | `src/candidate_stream.rs:40` |
| `contextra-vector` | `distance` | `fn compute_distance` | `contextra-types` | `src/distance.rs:537` |
| `contextra-vector` | `distance` | `fn compute_distance_trusted` | `contextra-types` | `src/distance.rs:542` |
| `contextra-vector` | `distance` | `fn compute_distance_f32_bytes_trusted` | `contextra-types` | `src/distance.rs:547` |
| `contextra-vector` | `types` | `struct DocId` | `contextra-types` | `src/types.rs:13` |
| `contextra-vector` | `types` | `struct DocId` | `contextra-types` | `src/types.rs:20` |
| `contextra-vector` | `types` | `struct TxId` | `contextra-types` | `src/types.rs:93` |
| `contextra-vector` | `types` | `enum DistanceMetric` | `contextra-types` | `src/types.rs:127` |
| `contextra-vector` | `types` | `struct ScoredDocument` | `contextra-types` | `src/types.rs:141` |

## 3. Kantenliste

| Von | Nach | prod Fundstellen | prod Symbole | dev Fundstellen | Cargo-Art |
|---|---|---:|---:|---:|---|
| `contextra-engine` | `contextra-types` | 227 | 50 | 178 | normal |
| `contextra-vector` | `contextra-types` | 97 | 21 | 40 | normal? |
| `contextra-checkpoint` | `contextra-ports` | 55 | 8 | 8 | normal |
| `contextra-store` | `contextra-ports` | 54 | 17 | 149 | normal |
| `contextra-graph` | `contextra-types` | 46 | 13 | 60 | normal? |
| `contextra-store` | `contextra-types` | 39 | 14 | 123 | normal |
| `contextra-rank` | `contextra-types` | 38 | 17 | 0 | normal? |
| `contextra-crypto` | `contextra-types` | 35 | 14 | 18 | normal |
| `contextra-engine` | `contextra-ports` | 33 | 16 | 19 | normal |
| `contextra-vector` | `contextra-ports` | 33 | 7 | 78 | normal? |
| `contextra-graph` | `contextra-ports` | 29 | 20 | 44 | normal? |
| `contextra-text` | `contextra-types` | 22 | 9 | 137 | normal? |
| `contextra-engine` | `contextra-store` | 21 | 9 | 99 | normal |
| `contextra-engine` | `contextra-graph` | 20 | 16 | 75 | normal |
| `contextra-testkit` | `contextra-types` | 20 | 4 | 10 | normal |
| `contextra-engine` | `contextra-rank` | 14 | 6 | 2 | normal |
| `contextra-testkit` | `contextra-ports` | 14 | 12 | 4 | normal |
| `contextra` | `contextra-types` | 13 | 9 | 20 | normal |
| `contextra-simd` | `contextra-types` | 13 | 6 | 0 | normal? |
| `contextra-mcp` | `contextra` | 12 | 11 | 9 | normal |
| `contextra-vector` | `contextra-mvcc` | 11 | 8 | 0 | normal? |
| `contextra` | `contextra-db` | 10 | 5 | 1 | normal |
| `contextra-store` | `contextra-mvcc` | 10 | 6 | 7 | normal |
| `contextra-engine` | `contextra-vector` | 9 | 4 | 117 | dev/normal |
| `contextra` | `contextra-ports` | 8 | 6 | 9 | normal |
| `contextra-text` | `contextra-ports` | 8 | 2 | 87 | normal? |
| `contextra-crypto` | `contextra-ports` | 6 | 6 | 34 | normal |
| `contextra-mcp` | `contextra-types` | 6 | 3 | 4 | normal |
| `contextra-db` | `contextra-engine` | 5 | 5 | 0 | normal |
| `contextra-store` | `contextra-crypto` | 5 | 4 | 2 | normal |
| `contextra-db` | `contextra-types` | 4 | 4 | 40 | normal |
| `contextra-engine` | `contextra-checkpoint` | 4 | 3 | 0 | normal |
| `contextra-engine` | `contextra-text` | 4 | 2 | 68 | normal |
| `contextra-mcp` | `contextra-crypto` | 4 | 2 | 3 | normal |
| `contextra-mcp` | `contextra-ports` | 3 | 2 | 39 | normal |
| `contextra-mvcc` | `contextra-ports` | 3 | 1 | 1 | normal |
| `contextra-ports` | `contextra-types` | 3 | 2 | 0 | normal |
| `contextra-store` | `contextra-sys` | 3 | 2 | 4 | normal |
| `contextra-bench` | `contextra-types` | 2 | 2 | 1 | normal |
| `contextra-rank` | `contextra-ports` | 2 | 1 | 0 | normal? |
| `contextra-vector` | `contextra-sys` | 2 | 1 | 0 | normal? |
| `contextra-bench` | `contextra-ports` | 1 | 1 | 1 | normal |
| `contextra-bench` | `contextra-rank` | 1 | 1 | 0 | normal |
| `contextra-checkpoint` | `contextra-mvcc` | 1 | 1 | 0 | – |
| `contextra-db` | `contextra-ports` | 1 | 1 | 9 | normal |
| `contextra-engine` | `contextra-mvcc` | 1 | 1 | 3 | normal |
| `contextra-mcp` | `contextra-rank` | 1 | 1 | 0 | normal |
| `contextra-bench` | `contextra-sys` | 0 | 0 | 1 | dev |
| `contextra-checkpoint` | `contextra-testkit` | 0 | 0 | 3 | dev |
| `contextra-checkpoint` | `contextra-types` | 0 | 0 | 13 | normal |
| `contextra-crypto` | `contextra-testkit` | 0 | 0 | 5 | dev |
| `contextra-db` | `contextra-checkpoint` | 0 | 0 | 4 | dev |
| `contextra-db` | `contextra-graph` | 0 | 0 | 13 | dev |
| `contextra-db` | `contextra-rank` | 0 | 0 | 1 | dev |
| `contextra-db` | `contextra-store` | 0 | 0 | 8 | dev |
| `contextra-db` | `contextra-text` | 0 | 0 | 10 | dev |
| `contextra-db` | `contextra-vector` | 0 | 0 | 4 | dev |
| `contextra-engine` | `contextra-testkit` | 0 | 0 | 1 | dev |
| `contextra-mvcc` | `contextra-testkit` | 0 | 0 | 6 | dev |
| `contextra-mvcc` | `contextra-types` | 0 | 0 | 7 | normal |
| `contextra-store` | `contextra-testkit` | 0 | 0 | 8 | dev |

## 4. Crates im Detail

### contextra

- Pfad: `crates/contextra` · Zeilen: 3352 · Rust-Dateien: 22 · Ring: 4
- Interne Cargo-Abhängigkeiten: `contextra-db` (normal), `contextra-ports` (normal), `contextra-types` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-types` | 13 | 20 | 7 | `error::ContextraError`×4, `DistanceMetric`×2, `ContextraError::Internal`, `types::domain::AutoExtractionMode`, `types::domain`, `ContextraError`, `ContextraError::DurabilityConfig`, `error::Result`, `ScoredEntry` | `DocId`×2, `DocId::new`×2, `ContextraError::InvalidInput`, `ScoredDocument`, `DistanceMetric::Cosine` |
| `contextra-db` | 10 | 1 | 2 | `DurabilityMode`×3, `DurabilityMode::Full`×2, `DurabilityMode::WalNoHmac`×2, `DurabilityMode::MemoryOnly`×2, `QueryRewriter` | – |
| `contextra-ports` | 8 | 9 | 4 | `license::FeatureRing`×2, `license::LicenseGate`×2, `IdGen`, `license`, `license::VectorDeleteMode`, `license::LicenseError` | `BoxFuture`×3, `license::AuthorizedRing`, `license::OpenFastGate`, `license::LicenseError::NotActivated`, `SequentialIdGen` |

**Pro Komponente**

- `(crate-root)` (188 Zeilen)
  - → `contextra-types`: prod: `error::ContextraError`, `DistanceMetric`, `types::domain`
- `agent_memory` (374 Zeilen)
  - → `contextra-ports`: prod: `IdGen`
  - → `contextra-types`: prod: `error::ContextraError`
- `boot` (90 Zeilen)
  - → `contextra-types`: prod: `error::ContextraError`
- `builder` (458 Zeilen)
  - → `contextra-ports`: prod: `license::FeatureRing`, `license`
  - → `contextra-types`: prod: `error::ContextraError`, `DistanceMetric`, `ContextraError::Internal`
- `collection_profile` (186 Zeilen)
  - → `contextra-ports`: prod: `license::LicenseGate`
  - → `contextra-types`: prod: `types::domain::AutoExtractionMode`
- `performance_profile` (294 Zeilen)
  - → `contextra-db`: prod: `DurabilityMode`×3, `DurabilityMode::Full`×2, `DurabilityMode::WalNoHmac`×2, `DurabilityMode::MemoryOnly`×2
  - → `contextra-ports`: prod: `license::FeatureRing`, `license::LicenseGate`, `license::VectorDeleteMode`, `license::LicenseError` · dev: `license::LicenseError`×2, `license::AuthorizedRing`, `license::OpenFastGate`, `license::LicenseError::NotActivated`
  - → `contextra-types`: prod: `ContextraError`, `ContextraError::DurabilityConfig`
- `query_rewriter` (250 Zeilen)
  - → `contextra-db`: prod: `QueryRewriter`
  - → `contextra-types`: prod: `error::Result`, `ScoredEntry`
- `examples/embedded_facade.rs` (47 Zeilen)
  - → `contextra-types`: dev: `DistanceMetric`
- `tests/agent_memory_facade.rs` (84 Zeilen)
  - → `contextra-ports`: dev: `BoxFuture`
  - → `contextra-types`: dev: `error::Result`
- `tests/agent_memory_hyperedges.rs` (149 Zeilen)
  - → `contextra-ports`: dev: `BoxFuture`
  - → `contextra-types`: dev: `error::Result`
- `tests/agent_memory_idgen_test.rs` (83 Zeilen)
  - → `contextra-ports`: dev: `BoxFuture`, `SequentialIdGen`
  - → `contextra-types`: dev: `error::Result`
- `tests/llm_query_rewriter.rs` (206 Zeilen)
  - → `contextra-db`: dev: `QueryRewriter`
  - → `contextra-types`: dev: `error::ContextraError`, `ScoredEntry`, `error::Result`
- `tests/type_identity_test.rs` (49 Zeilen)
  - → `contextra-types`: dev: `ContextraError`×2, `DocId`×2, `DocId::new`×2, `DistanceMetric`×2 … (+4)

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `(crate-root)`
  - 3 fn
    - `fn builder`
    - `fn open` ⚠ fremd: `contextra-types`
    - `fn open_with_config` ⚠ fremd: `contextra-types`
- `agent_memory`
  - 4 struct
    - `struct MemoryId` · 2 Methoden
    - `struct RelationId`
    - `struct Memory`
    - `struct AgentMemory` · 10 Methoden
- `boot`
  - 1 enum, 1 fn
    - `enum BootPhase`
    - `fn execute_boot_sequence` ⚠ fremd: `contextra-types`
- `builder`
  - 1 enum, 1 struct, 1 trait
    - `enum EncryptionMode`
    - `trait ContextraConfigExt` · Methoden: is_encrypted
    - `struct ContextraBuilder` · 14 Methoden
- `collection_profile`
  - 2 const, 3 enum, 2 struct
    - `enum KvDeleteMode`
    - `const DEFAULT_AUTO_EXTRACTION_MODE` ⚠ fremd: `contextra-types`
    - `const ENTERPRISE_REGULATED_AUTO_EXTRACTION_DEFAULT` ⚠ fremd: `contextra-types`
    - `struct LsmTuning`
    - `struct CollectionProfile` · 2 Methoden ⚠ fremd: `contextra-types`
    - `enum CollectionProfileError`
    - `enum DeploymentTier` · 1 Methoden
- `performance_profile`
  - 4 enum, 1 struct
    - `enum DurabilityMode` · 1 Methoden ⚠ fremd: `contextra-db`
    - `enum DurabilityConfigError` ⚠ fremd: `contextra-db`
    - `enum PerformanceProfile` · 1 Methoden
    - `struct ResolvedProfileConfig` · 2 Methoden ⚠ fremd: `contextra-db`, `contextra-ports`
    - `enum PerformanceProfileError` ⚠ fremd: `contextra-ports`
- `query_rewriter`
  - 1 struct
    - `struct LlmQueryRewriter` · 5 Methoden
- Zusätzlich 3 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).
- ⚠ Re-Exporte **fremder Crates**: `contextra_types::types::domain::AutoExtractionMode`, `contextra_types::error::ContextraError`, `contextra_types::DistanceMetric`, `contextra_ports::license::VectorDeleteMode`

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-mcp` | `Contextra`×2, `chunker`, `chunker::MarkdownChunker::new`, `chunker::ChunkerConfig::default`, `collection_profile`, `collection_profile::DeploymentTier::EdgeMinimal`, `collection_profile::DeploymentTier::PowerUserLocal`, `collection_profile::DeploymentTier::EnterpriseShared`, `collection_profile::DeploymentTier::EnterpriseRegulated`, `collection_profile::AutoExtractionMode::Enabled` … (+1) | `collection_profile::DeploymentTier` |

**Nach außen ungenutzte öffentliche Items (Heuristik, 24 von 25)**: `MemoryId`, `RelationId`, `Memory`, `AgentMemory`, `BootPhase`, `execute_boot_sequence`, `EncryptionMode`, `ContextraConfigExt`, `ContextraBuilder`, `KvDeleteMode`, `DEFAULT_AUTO_EXTRACTION_MODE`, `ENTERPRISE_REGULATED_AUTO_EXTRACTION_DEFAULT`, `LsmTuning`, `CollectionProfile`, `CollectionProfileError`, `builder`, `open`, `open_with_config`, `DurabilityMode`, `DurabilityConfigError`, `PerformanceProfile`, `ResolvedProfileConfig`, `PerformanceProfileError`, `LlmQueryRewriter`

### contextra-bench

- Pfad: `benchmarks/contextra-bench` · Zeilen: 3936 · Rust-Dateien: 14 · Ring: tooling
- Interne Cargo-Abhängigkeiten: `contextra-sys` (dev), `contextra-db` (normal), `contextra-ports` (normal), `contextra-rank` (normal), `contextra-store` (normal), `contextra-text` (normal), `contextra-types` (normal), `contextra-vector` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-types` | 2 | 1 | 1 | `Result`, `ContextraError::CapabilityUnsupported` | `TxId` |
| `contextra-ports` | 1 | 1 | 1 | `VectorIndex` | `StorageEngine` |
| `contextra-rank` | 1 | 0 | 1 | `RankError` | – |
| `contextra-sys` | 0 | 1 | 0 | – | `ensure_fd_limit` |

**Pro Komponente**

- `(crate-root)` (864 Zeilen)
  - → `contextra-types`: prod: `Result`, `ContextraError::CapabilityUnsupported`
- `ann_benchmarks` (244 Zeilen)
  - → `contextra-ports`: prod: `VectorIndex`
- `matrix` (185 Zeilen)
  - → `contextra-rank`: prod: `RankError`
- `benches/competitive_kv.rs` (554 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-sys`: dev: `ensure_fd_limit`
  - → `contextra-types`: dev: `TxId`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `adapters`
  - 5 struct, 2 trait
    - `struct BenchmarkDocument`
    - `struct EngineHit`
    - `struct QueryLatencyStats`
    - `trait FullTextEngineAdapter` · Methoden: name, index_batch, search, commit
    - `trait VectorEngineAdapter` · Methoden: name, dimension, insert_vectors, search_knn
    - `struct TantivyAdapterSketch` · 1 Methoden
    - `struct LanceDbAdapterSketch` · 1 Methoden
- `ann_benchmarks`
  - 5 fn, 1 struct
    - `struct AnnBenchResult`
    - `fn generate_synthetic_vectors`
    - `fn compute_ground_truth`
    - `fn recall_at_k`
    - `fn run_ann_benchmark`
    - `fn run_ann_benchmark_synthetic_ci`
- `beir_eval`
  - 6 fn, 3 struct
    - `struct BeirDocument`
    - `struct BeirQuery`
    - `struct BeirMetrics`
    - `fn load_beir_corpus`
    - `fn load_beir_queries`
    - `fn ndcg_at_k`
    - `fn map_at_k`
    - `fn recall_at_k`
    - `fn run_beir_eval`
- `compare`
  - 2 fn, 4 struct
    - `struct LongMemEvalMetricsSummary`
    - `struct LocomoMetricsSummary`
    - `struct CombinedMetrics`
    - `struct MetricComparisonResult`
    - `fn compare_metrics`
    - `fn compare_metrics_files`
- `matrix`
  - 1 enum, 6 fn, 2 struct
    - `enum EvaluatedStrategy` · 1 Methoden
    - `struct StrategyMatrixRow`
    - `struct ComparisonMatrixReport`
    - `fn to_rank_items`
    - `fn from_rank_items`
    - `fn to_search_results`
    - `fn fuse_via_rank_rrf` ⚠ fremd: `contextra-rank`
    - `fn fuse_via_rank_weighted_rrf`
    - `fn fuse_via_rank_score_normalized`
- `metrics`
  - 5 fn, 4 struct
    - `struct RelevanceGrade`
    - `struct RankedResult`
    - `struct QueryEvaluation`
    - `struct AggregateMetrics`
    - `fn calculate_recall_at_k`
    - `fn calculate_mrr_at_k`
    - `fn calculate_ndcg_at_k`
    - `fn calculate_latency_percentiles`
    - `fn read_process_rss_bytes`
- `regression_gate`
  - 1 const, 1 fn, 1 struct
    - `const DEFAULT_TOLERANCE_THRESHOLD`
    - `struct RegressionGateResult`
    - `fn run_regression_gate`
- Zusätzlich 7 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).

**Eingehend (wer nutzt dieses Crate)**

Wird von keinem anderen analysierten Workspace-Crate importiert.

**Nach außen ungenutzte öffentliche Items (Heuristik, 49 von 49)**: `BenchmarkDocument`, `EngineHit`, `QueryLatencyStats`, `FullTextEngineAdapter`, `VectorEngineAdapter`, `TantivyAdapterSketch`, `LanceDbAdapterSketch`, `AnnBenchResult`, `generate_synthetic_vectors`, `compute_ground_truth`, `recall_at_k`, `run_ann_benchmark`, `run_ann_benchmark_synthetic_ci`, `BeirDocument`, `BeirQuery`, `BeirMetrics`, `load_beir_corpus`, `load_beir_queries`, `ndcg_at_k`, `map_at_k`, `recall_at_k`, `run_beir_eval`, `LongMemEvalMetricsSummary`, `LocomoMetricsSummary`, `CombinedMetrics`, `MetricComparisonResult`, `compare_metrics`, `compare_metrics_files`, `EvaluatedStrategy`, `StrategyMatrixRow` …

**Auffälligkeiten**

- contextra-db: kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen)
- contextra-store: kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen)
- contextra-text: kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen)
- contextra-vector: kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen)

### contextra-checkpoint

- Pfad: `crates/contextra-checkpoint` · Zeilen: 8557 · Rust-Dateien: 27 · Ring: 1
- Interne Cargo-Abhängigkeiten: `contextra-testkit` (dev), `contextra-ports` (normal), `contextra-types` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-ports` | 55 | 8 | 4 | `Clock`×20, `StorageEngine`×14, `Vfs`×7, `MetricsSink`×6, `BoxFuture`×3, `SystemClock`×2, `Checkpoint`×2, `CheckpointCoordinator` | – |
| `contextra-mvcc` | 1 | 0 | 1 | `SnapshotRegistry` | – |
| `contextra-testkit` | 0 | 3 | 0 | – | `InMemoryStorageEngine`×2, `FaultVfs` |
| `contextra-types` | 0 | 13 | 0 | – | `TxId`×4, `Result`×3, `ContextraError`×2, `ContextraError::Internal`×2, `TxId::new` … (+1) |

**Pro Komponente**

- `guard` (1100 Zeilen)
  - → `contextra-ports`: prod: `Clock`×9, `StorageEngine`×6, `MetricsSink`×3, `SystemClock` · dev: `MetricsSink`×2, `Clock`
- `hardlink_cloner` (259 Zeilen)
  - → `contextra-mvcc`: prod: `SnapshotRegistry`
  - → `contextra-ports`: prod: `BoxFuture`×2
- `manifest` (210 Zeilen)
  - → `contextra-types`: dev: `TxId`
- `orphan` (845 Zeilen)
  - → `contextra-ports`: prod: `Clock`×8, `Vfs`×7, `SystemClock`, `StorageEngine`
  - → `contextra-testkit`: dev: `FaultVfs`
- `store` (1674 Zeilen)
  - → `contextra-ports`: prod: `StorageEngine`×7, `Clock`×3, `MetricsSink`×3, `Checkpoint`×2, `BoxFuture`, `CheckpointCoordinator` · dev: `CheckpointCoordinator`×5
- `examples/checkpoint_memvfs.rs` (34 Zeilen)
  - → `contextra-testkit`: dev: `InMemoryStorageEngine`
  - → `contextra-types`: dev: `TxId::new`, `TxId::INTERNAL_BASE`
- `tests/checkpoint_clock_determinism.rs` (178 Zeilen)
  - → `contextra-types`: dev: `TxId`
- `tests/checkpoint_systematic_crash.rs` (225 Zeilen)
  - → `contextra-testkit`: dev: `InMemoryStorageEngine`
- `tests/clock_determinism.rs` (114 Zeilen)
  - → `contextra-types`: dev: `TxId`
- `tests/hardlink_cloner_tests.rs` (151 Zeilen)
  - → `contextra-types`: dev: `Result`×3, `ContextraError`
- `tests/manifest_fault_injection.rs` (82 Zeilen)
  - → `contextra-types`: dev: `TxId`
- `tests/time_travel_correctness.rs` (737 Zeilen)
  - → `contextra-types`: dev: `ContextraError::Internal`×2, `ContextraError`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `manifest`
  - 1 struct
    - `struct CheckpointManifest` *(re-exportiert)* · 2 Methoden
- Zusätzlich 37 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-engine` | `InstanceOrphanRegistry`×2, `InstanceOrphanRegistry::new`, `(crate)` | – |
| `contextra-db` | – | `PersistentCheckpointStore`×2, `clear_all_orphaned_checkpoints`, `get_orphaned_checkpoints` |

**Nach außen ungenutzte öffentliche Items (Heuristik, 1 von 1)**: `CheckpointManifest`

**Auffälligkeiten**

- contextra-types: nur in Tests/Benches genutzt, aber als normale Abhängigkeit deklariert → dev-dependency?
- contextra-mvcc: genutzt, aber nicht in Cargo.toml deklariert (cfg-/Feature-Pfad oder Fehler)

### contextra-crypto

- Pfad: `crates/contextra-crypto` · Zeilen: 16231 · Rust-Dateien: 74 · Ring: 0
- Interne Cargo-Abhängigkeiten: `contextra-testkit` (dev), `contextra-ports` (normal), `contextra-types` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-types` | 35 | 18 | 10 | `TenantId`×8, `Result`×7, `ScanMethod::FeatureDisabled`×3, `ContextraError::Crypto`×3, `TenantScoped`×2, `ScanMethod::Structural`×2, `ScanMethod::NotMeasured`×2, `ContextraError`×2, `RequestContext`, `SagaRecord` … (+4) | `ContextraError::Io`×4, `ContextraError::NotFound` |
| `contextra-ports` | 6 | 34 | 2 | `scanner::WitnessLayer`, `scanner::LayerWitness`, `Clock`, `scanner::ScanTechnique::NotMeasured`, `ScanTechnique`, `scanner` | `Result`×24, `SystemClock`×6, `scanner::ScanTechnique`×4 |
| `contextra-testkit` | 0 | 5 | 0 | – | `mint_witness_for_tests`×4, `FaultVfs` |

**Pro Komponente**

- `crypto` (821 Zeilen)
  - → `contextra-types`: prod: `TenantId`×4, `TenantScoped`×2 · dev: `TenantId`×2
- `deletion_proof` (1315 Zeilen)
  - → `contextra-ports`: prod: `scanner::WitnessLayer`, `scanner::LayerWitness`, `Clock`, `scanner::ScanTechnique::NotMeasured`, `ScanTechnique`, `scanner`
  - → `contextra-types`: prod: `ScanMethod::FeatureDisabled`×3, `ScanMethod::Structural`×2, `ScanMethod::NotMeasured`×2, `Result`, `RequestContext`, `SagaRecord`, `error::HnswDeletionError::VerificationFailed`, `ContextraError::Internal`, `TenantId`
- `error` (121 Zeilen)
  - → `contextra-types`: prod: `ContextraError`×2 · dev: `ContextraError`
- `kv_cipher` (350 Zeilen)
  - → `contextra-types`: prod: `Result`×6, `ContextraError::Crypto`×3, `TenantId`, `ModelFingerprint`, `ContextraError::ParseError`
- `kv_segment` (1437 Zeilen)
  - → `contextra-types`: prod: `TenantId`×2 · dev: `TenantId`
- `revocation_log` (450 Zeilen)
  - → `contextra-ports`: dev: `Result`×8, `SystemClock`
  - → `contextra-testkit`: dev: `FaultVfs`
  - → `contextra-types`: dev: `ContextraError::Io`×4, `ContextraError::NotFound`
- `benches/crypto_benchmarks.rs` (127 Zeilen)
  - → `contextra-types`: dev: `TenantId`
- `fuzz/fuzz_targets/deletion_proof_tamper.rs` (137 Zeilen)
  - → `contextra-ports`: dev: `scanner::ScanTechnique`, `SystemClock`
  - → `contextra-testkit`: dev: `mint_witness_for_tests`
- `tests/audit_chain_tests.rs` (213 Zeilen)
  - → `contextra-ports`: dev: `scanner::ScanTechnique`, `SystemClock`
  - → `contextra-testkit`: dev: `mint_witness_for_tests`
- `tests/campaign_cov_revocation_log.rs` (38 Zeilen)
  - → `contextra-ports`: dev: `SystemClock`
- `tests/campaign_j25.rs` (426 Zeilen)
  - → `contextra-types`: dev: `TenantId`
- `tests/campaign_j26_hypotheses.rs` (784 Zeilen)
  - → `contextra-ports`: dev: `Result`×8, `scanner::ScanTechnique`, `SystemClock`
  - → `contextra-testkit`: dev: `mint_witness_for_tests`
- `tests/deletion_proof_signature_version_serde.rs` (197 Zeilen)
  - → `contextra-ports`: dev: `scanner::ScanTechnique`, `SystemClock`
  - → `contextra-testkit`: dev: `mint_witness_for_tests`
- `tests/eviction_worker_tokio_runtime_nonblocking_regression.rs` (65 Zeilen)
  - → `contextra-types`: dev: `TenantId`
- `tests/kv_segment_concurrency.rs` (271 Zeilen)
  - → `contextra-types`: dev: `TenantId`
- `tests/kv_segment_integration.rs` (120 Zeilen)
  - → `contextra-types`: dev: `TenantId`
- `tests/kv_segment_proptests.rs` (82 Zeilen)
  - → `contextra-types`: dev: `TenantId`
- `tests/loom_kv_deferred_zeroize.rs` (161 Zeilen)
  - → `contextra-types`: dev: `TenantId`
- `tests/proptests.rs` (130 Zeilen)
  - → `contextra-types`: dev: `TenantId`
- `tests/revocation_log_vfs.rs` (201 Zeilen)
  - → `contextra-ports`: dev: `Result`×8
  - → `contextra-types`: dev: `ContextraError`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `aead`
  - 1 const, 4 fn
    - `const REC_AAD`
    - `fn seal_record_with_nonce`
    - `fn seal_record_with_rng`
    - `fn seal_record`
    - `fn open_record`
- `anti_tamper`
  - 2 fn, 2 struct
    - `struct VolatileEncryptionKey` · 3 Methoden
    - `fn constant_time_compare`
    - `fn verify_tag_constant_time`
    - `struct Blake3Chain` · 5 Methoden
- `audit_chain`
  - 1 enum, 1 fn, 5 struct
    - `enum DataClass`
    - `struct AuditChainEntry` · 1 Methoden
    - `struct AuditChainHeadSignature`
    - `struct AuditHeadAnchor` · 2 Methoden
    - `struct EncryptedCommitmentSalt` · 2 Methoden
    - `fn compute_record_commitment`
    - `struct AuditChain` · 10 Methoden
- `crypto`
  - 1 struct
    - `struct KeyManager` · 15 Methoden
- `dek`
  - 3 const, 3 fn, 3 struct, 2 type
    - `struct UnitKey` · 2 Methoden
    - `struct Dek` · 2 Methoden
    - `type Salt`
    - `type DocTag`
    - `struct AncestorSalts`
    - `const DEK_LABEL`
    - `const DEK_SALT_LABEL`
    - `const WRAP_AAD`
    - `fn derive_dek`
    - `fn wrap_dek`
    - `fn unwrap_dek`
- `deletion_proof`
  - 1 const, 7 enum, 8 fn, 7 struct, 1 trait
    - `fn verify_proof_sequence` ⚠ fremd: `contextra-types`
    - `fn compute_wal_delete_receipt`
    - `const DELETION_PROOF_KEY_PURPOSE`
    - `fn derive_deletion_proof_key_for_tenant`
    - `trait LayerWitnessExt` · Methoden: wal_seq_after, control_matches
    - `fn map_witness_layer_to_deletion_layer`
    - `struct GraphRepairAttestation`
    - `fn hash_deleted_keys_length_prefixed`
    - `enum CustodyLevel`
    - `enum SigningKeyHandle`
    - `enum VerificationKey`
    - `struct LayerCleanupProof` · 2 Methoden
    - `enum DeletionLayer`
    - `struct UncoveredLayer`
    - `fn serialize` *(re-exportiert)* ⚠ fremd: `contextra-ports`
    - `fn deserialize` *(re-exportiert)* ⚠ fremd: `contextra-ports`
    - `struct LayerWitnessMeasurement` ⚠ fremd: `contextra-ports`
    - `enum ExcludedScope`
    - `enum DeletionScope`
    - `struct DeletionProof`
    - `fn verify_wal_delete_receipt`
    - `enum VerificationErrorReason` ⚠ fremd: `contextra-types`
    - `struct VerificationResult` · 2 Methoden
    - `struct DeletionProofVerifier` · 3 Methoden
- `ed25519_proof`
  - 2 enum, 2 fn, 1 struct
    - `enum DeletionProofError`
    - `struct DeletionProofKeyPair` · 6 Methoden
    - `fn sign_deletion_proof_v3`
    - `fn verify_deletion_proof_v3`
    - `enum SignatureVersion` · 1 Methoden
- `error`
  - 1 enum, 1 type
    - `type Result`
    - `enum CryptoError` · 1 Methoden ⚠ fremd: `contextra-types`
- `kdf`
  - 10 const, 2 fn, 3 struct
    - `const KDF_HEADER_MAGIC`
    - `const KDF_HEADER_VERSION_1`
    - `const KDF_ID_ARGON2ID`
    - `const MIN_SALT_LEN`
    - `const MIN_M_COST_KIB`
    - `const MIN_T_COST`
    - `const MIN_P_COST`
    - `const DEFAULT_M_COST_KIB`
    - `const DEFAULT_T_COST`
    - `const DEFAULT_P_COST`
    - `struct KdfParams` · 1 Methoden
    - `struct KdfHeader` · 4 Methoden
    - `struct DerivedKey`
    - `fn derive_key_argon2id`
    - `fn hkdf_expand`
- `kv_cipher`
  - 1 const, 2 struct, 1 trait
    - `const CURRENT_KV_FORMAT_VERSION`
    - `struct EncryptedKvLayer` ⚠ fremd: `contextra-types`
    - `trait KvCipher` · Methoden: seal, open
    - `struct KvSegmentCipher` · 6 Methoden
- `kv_segment`
  - 1 const, 1 enum, 1 fn, 7 struct, 1 type
    - `enum KvDeleteMode`
    - `struct EvictionWorker` · 3 Methoden
- `kv_shredding`
  - 1 const, 1 fn, 5 struct
- `revocation_log`
  - 1 enum, 1 fn, 2 struct
- `secret`
  - 2 struct
- `wal_completeness`
  - 1 fn
- `wal_crypto`
  - 6 struct, 1 trait
  - … 30 weitere Items nicht aufgelistet (`--max-items`)
- Zusätzlich 9 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).
- Interne Re-Exporte (`pub use`): `DeletionProofKeyPair`, `hash_deleted_keys_length_prefixed`
- ⚠ Re-Exporte **fremder Crates**: `contextra_types::ModelFingerprint`

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-store` | `wal_crypto`×2, `verify_wal_chain_completeness`, `wal_crypto::WalHmac`, `crypto::KeyManager` | – |
| `contextra-mcp` | `CryptoKey`×3, `CryptoKey::try_new` | `AuditChain` |

**Nach außen ungenutzte öffentliche Items (Heuristik, 106 von 110)**: `REC_AAD`, `seal_record_with_nonce`, `seal_record_with_rng`, `seal_record`, `open_record`, `VolatileEncryptionKey`, `constant_time_compare`, `verify_tag_constant_time`, `Blake3Chain`, `DataClass`, `AuditChainEntry`, `AuditChainHeadSignature`, `AuditHeadAnchor`, `EncryptedCommitmentSalt`, `compute_record_commitment`, `UnitKey`, `Dek`, `Salt`, `DocTag`, `AncestorSalts`, `DEK_LABEL`, `DEK_SALT_LABEL`, `WRAP_AAD`, `derive_dek`, `wrap_dek`, `unwrap_dek`, `verify_proof_sequence`, `compute_wal_delete_receipt`, `DELETION_PROOF_KEY_PURPOSE`, `derive_deletion_proof_key_for_tenant` …

### contextra-db

- Pfad: `crates/contextra-db` · Zeilen: 11466 · Rust-Dateien: 55 · Ring: 3
- Interne Cargo-Abhängigkeiten: `contextra-checkpoint` (dev), `contextra-graph` (dev), `contextra-rank` (dev), `contextra-store` (dev), `contextra-testkit` (dev), `contextra-text` (dev), `contextra-vector` (dev), `contextra-engine` (normal), `contextra-ports` (normal), `contextra-types` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-engine` | 5 | 0 | 1 | `(crate)`, `DurabilityMode`, `decay_controller::*`, `fusion`, `collection` | – |
| `contextra-types` | 4 | 40 | 2 | `DistanceMetric`, `MAX_SEARCH_K`, `EntityId`, `EntityId::from_key` | `Result`×5, `TxId`×5, `FusionWeights`×3, `TenantId`×2, `DocId`×2 … (+15) |
| `contextra-ports` | 1 | 9 | 1 | `QueryRewriter` | `GraphIndex`×3, `StorageEngine`×2, `EmbeddingError`, `VectorIndex`, `BoxFuture` … (+1) |
| `contextra-checkpoint` | 0 | 4 | 0 | – | `PersistentCheckpointStore`×2, `clear_all_orphaned_checkpoints`, `get_orphaned_checkpoints` |
| `contextra-graph` | 0 | 13 | 0 | – | `CsrGraph`×9, `csr::CsrGraph`×2, `path_rag::PathRAGEngine`, `PathRAGEngine` |
| `contextra-rank` | 0 | 1 | 0 | – | `IsotonicCalibrator::new` |
| `contextra-store` | 0 | 8 | 0 | – | `LsmConfig`×3, `LsmStorage::new`×3, `lsm`, `LsmStorage` |
| `contextra-text` | 0 | 10 | 0 | – | `Language::English`×6, `Language`×2, `bm25::score_term`, `inverted::InvertedIndex` |
| `contextra-vector` | 0 | 4 | 0 | – | `HnswConfig`×2, `HnswIndex`, `HnswIndex::try_new` |

**Pro Komponente**

- `(crate-root)` (794 Zeilen)
  - → `contextra-engine`: prod: `(crate)`, `DurabilityMode`, `decay_controller::*`, `fusion`, `collection`
  - → `contextra-rank`: dev: `IsotonicCalibrator::new`
  - → `contextra-types`: prod: `DistanceMetric` · dev: `TenantId`, `Result`, `ContextraError::InvalidInput`
- `multistep` (403 Zeilen)
  - → `contextra-ports`: prod: `QueryRewriter`
  - → `contextra-types`: prod: `MAX_SEARCH_K`, `EntityId`, `EntityId::from_key`
- `examples/collections.rs` (114 Zeilen)
  - → `contextra-types`: dev: `Result`, `TenantId::new`
- `examples/db_standalone.rs` (36 Zeilen)
  - → `contextra-types`: dev: `Result`, `ContextraError::Io`
- `examples/hybrid_search.rs` (97 Zeilen)
  - → `contextra-types`: dev: `Result`
- `examples/quickstart.rs` (65 Zeilen)
  - → `contextra-types`: dev: `Result`
- `tests/atomic_commit.rs` (100 Zeilen)
  - → `contextra-types`: dev: `EntityId`
- `tests/auto_community_detection_test.rs` (127 Zeilen)
  - → `contextra-types`: dev: `EntityId`
- `tests/cascade_invalidation_test.rs` (116 Zeilen)
  - → `contextra-graph`: dev: `path_rag::PathRAGEngine`
  - → `contextra-ports`: dev: `GraphIndex`
  - → `contextra-types`: dev: `LinkRelation`
- `tests/checkpoint_layer_bounds.rs` (137 Zeilen)
  - → `contextra-checkpoint`: dev: `PersistentCheckpointStore`
  - → `contextra-store`: dev: `LsmConfig`×2, `LsmStorage::new`×2
  - → `contextra-types`: dev: `TxId`
- `tests/checkpoint_stress.rs` (78 Zeilen)
  - → `contextra-checkpoint`: dev: `PersistentCheckpointStore`
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/compound_split_recall_impact.rs` (402 Zeilen)
  - → `contextra-types`: dev: `FusionWeights`
- `tests/compound_split_recall_impact_test.rs` (456 Zeilen)
  - → `contextra-types`: dev: `FusionWeights`
- `tests/cross_domain_chaos_matrix_test.rs` (281 Zeilen)
  - → `contextra-graph`: dev: `csr::CsrGraph`
  - → `contextra-ports`: dev: `EmbeddingError`
  - → `contextra-store`: dev: `lsm`
  - → `contextra-text`: dev: `Language`
  - → `contextra-types`: dev: `DocId`
- `tests/diskann_collection_fallback.rs` (97 Zeilen)
  - → `contextra-graph`: dev: `csr::CsrGraph`
  - → `contextra-ports`: dev: `VectorIndex`
  - → `contextra-store`: dev: `LsmStorage`, `LsmConfig`
  - → `contextra-text`: dev: `Language`
  - → `contextra-types`: dev: `TxId::new`
- `tests/embedding_dimension_switch.rs` (91 Zeilen)
  - → `contextra-types`: dev: `ContextraError`
- `tests/fault_injection_2pc.rs` (783 Zeilen)
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-text`: dev: `Language::English`
- `tests/graph_entity_delete_test.rs` (161 Zeilen)
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-ports`: dev: `GraphIndex`
  - → `contextra-text`: dev: `Language::English`
  - → `contextra-vector`: dev: `HnswIndex`, `HnswConfig`
- `tests/graph_hybrid_search.rs` (299 Zeilen)
  - → `contextra-ports`: dev: `GraphIndex`
  - → `contextra-types`: dev: `ContextraError::SnapshotUnsupportedForSignal`×2, `GraphTraversalStrategy`, `EntityId::from_key`, `PprAlgorithm::Auto` … (+1)
- `tests/hnsw_delete_and_backfill.rs` (223 Zeilen)
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-text`: dev: `Language::English`
- `tests/instance_orphan_registry_test.rs` (133 Zeilen)
  - → `contextra-checkpoint`: dev: `clear_all_orphaned_checkpoints`, `get_orphaned_checkpoints`
  - → `contextra-types`: dev: `TxId`
- `tests/integration_temporal_filter.rs` (206 Zeilen)
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-types`: dev: `TxId`
- `tests/multistep_query_rewriter_modalities.rs` (204 Zeilen)
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-text`: dev: `Language::English`
  - → `contextra-types`: dev: `ContextraError::Internal`
- `tests/multistep_retrieval_tests.rs` (299 Zeilen)
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-ports`: dev: `BoxFuture`
  - → `contextra-text`: dev: `Language::English`
  - → `contextra-types`: dev: `ContextraError::Internal`
- `tests/pid_pool_production_caller.rs` (121 Zeilen)
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-ports`: dev: `Clock`
  - → `contextra-text`: dev: `Language::English`
- `tests/proptest_search_invariants.rs` (211 Zeilen)
  - → `contextra-text`: dev: `bm25::score_term`
- `tests/search_result_bound.rs` (221 Zeilen)
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-types`: dev: `DistanceMetric`
- `tests/snapshot_api.rs` (165 Zeilen)
  - → `contextra-types`: dev: `ContextraError::CapabilityUnsupported`×2, `EntityId::new`
  - → `contextra-vector`: dev: `HnswIndex::try_new`, `HnswConfig`
- `tests/split_brain_vector_orphan_test.rs` (506 Zeilen)
  - → `contextra-types`: dev: `DocId`
- `tests/sprint2_acid_tests.rs` (349 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-store`: dev: `LsmStorage::new`
  - → `contextra-types`: dev: `TenantId`, `TxId`
- `tests/stabilization_sprint_tests.rs` (145 Zeilen)
  - → `contextra-types`: dev: `FusionWeights`
- `tests/supersedes_cascading_tombstone_test.rs` (171 Zeilen)
  - → `contextra-graph`: dev: `PathRAGEngine`
- `tests/text_integration.rs` (102 Zeilen)
  - → `contextra-text`: dev: `inverted::InvertedIndex`
- `tests/truncation_filter_recall_test.rs` (284 Zeilen)
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-types`: dev: `MemoryType`
- `tests/zettelkasten_links_test.rs` (276 Zeilen)
  - → `contextra-types`: dev: `HybridQuery`, `FusionWeights::new`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `homeostat`
  - 2 struct
    - `struct RerankDeadline` · 3 Methoden
    - `struct RerankPidController` · 3 Methoden
- `multistep`
  - 2 const, 3 struct
    - `const DEFAULT_MAX_SUB_QUERIES`
    - `const DEFAULT_RERANK_POOL_MULTIPLIER`
    - `struct MultiStepConfig`
    - `struct MultiStepResult`
    - `struct MultiStepEngine` · 4 Methoden
- `pid_latency_controller`
  - 3 const, 2 struct
    - `const DEFAULT_TARGET_LATENCY_MS`
    - `const MIN_SCALING_FACTOR`
    - `const MAX_SCALING_FACTOR`
    - `struct PidLatencyController` · 5 Methoden
    - `struct LatencyBudgetGuard` · 6 Methoden
- `volatile_vault`
  - 2 enum, 6 struct
    - `enum SignalModality`
    - `struct VaultChunk` · 3 Methoden
    - `struct VaultConfig`
    - `struct VolatileDrainReceipt`
    - `struct CommitReceipt`
    - `enum VaultError`
    - `struct VolatileContextVault` · 10 Methoden
    - `struct VaultChunkMetadata`
- Zusätzlich 7 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).
- Interne Re-Exporte (`pub use`): `*`, `RerankDeadline`, `RerankPidController`
- ⚠ Re-Exporte **fremder Crates**: `contextra_engine`, `contextra_engine::DurabilityMode`, `contextra_engine::decay_controller::*`, `contextra_engine::fusion`, `contextra_types::DistanceMetric`, `contextra_ports::QueryRewriter`

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra` | `DurabilityMode`×3, `DurabilityMode::Full`×2, `DurabilityMode::WalNoHmac`×2, `DurabilityMode::MemoryOnly`×2, `QueryRewriter` | – |

**Nach außen ungenutzte öffentliche Items (Heuristik, 20 von 20)**: `RerankDeadline`, `RerankPidController`, `DEFAULT_MAX_SUB_QUERIES`, `DEFAULT_RERANK_POOL_MULTIPLIER`, `MultiStepConfig`, `MultiStepResult`, `MultiStepEngine`, `DEFAULT_TARGET_LATENCY_MS`, `MIN_SCALING_FACTOR`, `MAX_SCALING_FACTOR`, `PidLatencyController`, `LatencyBudgetGuard`, `SignalModality`, `VaultChunk`, `VaultConfig`, `VolatileDrainReceipt`, `CommitReceipt`, `VaultError`, `VolatileContextVault`, `VaultChunkMetadata`

**Auffälligkeiten**

- contextra-testkit: kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen)

### contextra-durable-fs

- Pfad: `crates/contextra-durable-fs` · Zeilen: 2750 · Rust-Dateien: 21 · Ring: 0
- Interne Cargo-Abhängigkeiten: keine

**Ausgehend (Importe aus anderen Workspace-Crates)**

Keine.

**Pro Komponente**

Keine Importe aus anderen Workspace-Crates.

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `(crate-root)`
  - 2 enum, 12 fn, 3 struct
    - `enum FsClass` · 1 Methoden
    - `fn flush_file`
    - `fn flush_dir`
    - `fn fs_class`
    - `fn rename_durable`
    - `struct PoisonedHandle`
    - `struct DurableHandle` · 8 Methoden
    - `enum LockMode`
    - `struct DirLock` · 2 Methoden
    - `fn sync_dir`
    - `fn sync_dir_with_injector`
    - `fn atomic_replace`
    - `fn atomic_replace_with_injector`
    - `fn durable_remove`
    - `fn durable_remove_with_injector`
    - `fn scrub_and_remove`
    - `fn scrub_and_remove_with_injector`
- `error`
  - 1 enum, 2 type
    - `enum DurableFsError` · 1 Methoden
    - `type Error`
    - `type Result`
- `fault_injector`
  - 1 enum, 2 struct, 1 trait
    - `trait FaultInjector` · Methoden: on_write, on_fsync, on_rename, on_dir_fsync
    - `struct NoFault`
    - `enum InjectedFault`
    - `struct FaultInjectingVfs` · 6 Methoden
- `health`
  - 2 enum, 1 struct
    - `enum HealthState` · 4 Methoden
    - `enum HealthEvent`
    - `struct AtomicHealthState` · 3 Methoden
- `journal`
  - 2 const, 2 enum, 1 fn, 3 struct
    - `enum JournalSyncPolicy`
    - `enum RecordRecoveryPolicy`
    - `const JOURNAL_MAGIC`
    - `const RECORD_HEADER_SIZE`
    - `fn calculate_crc32`
    - `struct JournalRecord`
    - `struct AppendOnlyJournal` · 8 Methoden
    - `struct PendingDeleteJournal` · 6 Methoden
- `platform`
  - 1 enum, 2 fn
    - `enum FlushKind`
    - `fn sync_file`
    - `fn sync_dir_path`
- `scrub`
  - 3 enum, 3 fn, 2 struct
    - `struct ScrubReceipt`
    - `enum ScrubPolicy`
    - `enum ScrubRefusal`
    - `enum ScrubOutcome`
    - `struct ScrubProgress`
    - `fn scrub_file_with_rate_limit`
    - `fn scrub_if_safe`
    - `fn scrub_if_safe_with_injector`
- `vfs`
  - 1 struct, 1 trait, 1 type
    - `trait Vfs` · Methoden: read, write_atomic, append, sync_file, sync_dir, remove…
    - `struct StdVfs`
    - `type RealFs`

**Eingehend (wer nutzt dieses Crate)**

Wird von keinem anderen analysierten Workspace-Crate importiert.

**Nach außen ungenutzte öffentliche Items (Heuristik, 49 von 49)**: `DurableFsError`, `Error`, `Result`, `FaultInjector`, `NoFault`, `InjectedFault`, `FaultInjectingVfs`, `HealthState`, `HealthEvent`, `AtomicHealthState`, `JournalSyncPolicy`, `RecordRecoveryPolicy`, `JOURNAL_MAGIC`, `RECORD_HEADER_SIZE`, `calculate_crc32`, `JournalRecord`, `AppendOnlyJournal`, `PendingDeleteJournal`, `FsClass`, `flush_file`, `flush_dir`, `fs_class`, `rename_durable`, `PoisonedHandle`, `DurableHandle`, `LockMode`, `DirLock`, `sync_dir`, `sync_dir_with_injector`, `atomic_replace` …

### contextra-engine

- Pfad: `crates/contextra-engine` · Zeilen: 25640 · Rust-Dateien: 123 · Ring: 3
- Interne Cargo-Abhängigkeiten: `contextra-testkit` (dev), `contextra-vector` (dev), `contextra-checkpoint` (normal), `contextra-graph` (normal), `contextra-mvcc` (normal), `contextra-ports` (normal), `contextra-rank` (normal), `contextra-store` (normal), `contextra-text` (normal), `contextra-types` (normal), `contextra-vector` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-types` | 227 | 178 | 34 | `ContextraError::invalid_input`×50, `EntityId`×15, `HybridQuery`×13, `Result`×11, `MAX_SEARCH_K`×11, `ContextraError::Internal`×9, `ImportanceScore::new`×8, `domain::MemoryLink`×7, `EntityId::from_key`×7, `ContextraError::Serialization`×6 … (+40) | `ContextraError::from`×10, `domain::LinkRelation::Elaborates`×2, `ContextraError::HnswConnectivityDegraded`, `FusionWeights::new`, `PprConfig::default` … (+7) |
| `contextra-ports` | 33 | 19 | 16 | `MetricsSink`×9, `StorageEngine`×5, `Clock`×3, `VectorIndexStats`×2, `Reranker`×2, `DriftStatusProvider`×2, `VectorIndex`, `SystemClock::new`, `GraphIndex`, `license` … (+6) | `BoxFuture` |
| `contextra-store` | 21 | 99 | 7 | `LsmStorage`×6, `SystemPressure`×3, `tenant_codec::TenantScopedStorage`×3, `tenant_codec::TenantScopedStorage::new`×3, `PressureLevel::Critical`×2, `PressureLevel::Normal`, `PressureLevel::Elevated`, `LsmConfig`, `lsm::DurabilityMode` | `lsm`×10 |
| `contextra-graph` | 20 | 75 | 7 | `hyperedge::HyperEdgeId`×2, `percolation::PercolationConfig`×2, `CsrGraph::load_from_storage`×2, `hyperedge::HyperEdge`×2, `cascade_invalidate_edges_for_superseded_doc`, `percolation::compute_percolation_health`, `percolation::should_trigger_rebonding`, `percolation::find_rebonding_candidates`, `CsrGraph`, `hyperedge::RoleId::new` … (+6) | `csr::CsrGraph`×15, `csr`, `CsrGraph::new` |
| `contextra-rank` | 14 | 2 | 4 | `SearchResult`×8, `IsotonicCalibrator`×2, `fusion`, `fusion::fuse_search_results_with_strategy`, `fusion::fuse_search_results_with_signal_strategies`, `fusion::fuse_signals` | – |
| `contextra-vector` | 9 | 117 | 8 | `HnswIndex`×4, `hnsw::HnswIndex`×3, `acorn::FilteredIndex`, `HnswConfig` | `hnsw::HnswConfig::default`, `hnsw::HnswIndex::try_new`, `acorn`, `HnswIndex::try_new` |
| `contextra-checkpoint` | 4 | 0 | 3 | `InstanceOrphanRegistry`×2, `InstanceOrphanRegistry::new`, `(crate)` | – |
| `contextra-text` | 4 | 68 | 3 | `inverted::InvertedIndex`×2, `Language`×2 | `Language::English`×63, `Language::German`×4 |
| `contextra-mvcc` | 1 | 3 | 1 | `tx_buffer::TxBuffer` | `tx_buffer::IndexOp` |
| `contextra-testkit` | 0 | 1 | 0 | – | `InMemoryStorageEngine` |

**Pro Komponente**

- `(crate-root)` (368 Zeilen)
  - → `contextra-checkpoint`: prod: `(crate)`, `InstanceOrphanRegistry`
  - → `contextra-ports`: prod: `MetricsSink`×3, `license::LicenseGate`, `StorageEngine`, `TextEmbeddingEngine`, `VectorDeleteMode`, `DriftStatusProvider`, `VectorIndexStats`, `StorageStats`
  - → `contextra-rank`: prod: `IsotonicCalibrator`
  - → `contextra-store`: prod: `lsm::DurabilityMode`, `LsmStorage`, `tenant_codec::TenantScopedStorage`
  - → `contextra-text`: prod: `Language`
  - → `contextra-types`: prod: `DistanceMetric`×2, `TenantId`, `DistanceMetric::Cosine`
  - → `contextra-vector`: prod: `HnswIndex`
- `adapters` (54 Zeilen)
  - → `contextra-text`: prod: `inverted::InvertedIndex`
- `backends` (28 Zeilen)
  - → `contextra-store`: prod: `LsmStorage`
  - → `contextra-vector`: prod: `HnswIndex`
- `background_workers` (1158 Zeilen)
  - → `contextra-graph`: prod: `hyperedge::HyperEdgeId` · dev: `CsrGraph`×8, `hyperedge::HyperEdgeId`
  - → `contextra-mvcc`: prod: `tx_buffer::TxBuffer` · dev: `tx_buffer::TxBuffer`×2, `tx_buffer::IndexOp`
  - → `contextra-ports`: prod: `StorageEngine`×2, `VectorIndex`, `Clock`, `SystemClock::new`
  - → `contextra-store`: dev: `LsmStorage`×8, `LsmConfig`×8
  - → `contextra-text`: dev: `Language::English`×8
  - → `contextra-types`: prod: `Result`×4, `TxId` · dev: `Result`×2, `ContextraError::HnswConnectivityDegraded`
  - → `contextra-vector`: prod: `hnsw::HnswIndex`×3 · dev: `HnswIndex`×8, `HnswConfig`×8, `hnsw::HnswConfig::default`, `hnsw::HnswIndex::try_new`
- `collection` (11178 Zeilen)
  - → `contextra-graph`: prod: `percolation::PercolationConfig`×2, `cascade_invalidate_edges_for_superseded_doc`, `percolation::compute_percolation_health`, `percolation::should_trigger_rebonding`, `percolation::find_rebonding_candidates`, `CsrGraph`, `hyperedge::HyperEdgeId`, `hyperedge::RoleId::new`, `hyperedge::HyperEdgeId::new`, `hyperedge::RoleBinding` … (+4) · dev: `CsrGraph`×40, `csr::CsrGraph`×15, `csr`, `percolation::PercolationConfig`
  - → `contextra-ports`: prod: `VectorIndexStats`, `GraphIndex`, `Reranker`, `StorageEngine` · dev: `StorageStats`×5, `StorageEngine`×3, `TextEmbeddingEngine`, `BoxFuture`
  - → `contextra-store`: prod: `SystemPressure`×3, `PressureLevel::Critical`×2, `PressureLevel::Normal`, `PressureLevel::Elevated`, `LsmStorage` · dev: `LsmStorage`×36, `LsmConfig`×35, `lsm`×10
  - → `contextra-text`: prod: `inverted::InvertedIndex`, `Language` · dev: `Language::English`×50, `Language::German`×4, `Language`
  - → `contextra-types`: prod: `ContextraError::invalid_input`×37, `HybridQuery`×12, `MAX_SEARCH_K`×11, `EntityId`×11, `ContextraError::Internal`×9, `ImportanceScore::new`×8, `domain::MemoryLink`×7, `EntityId::from_key`×7, `Entity::new`×6, `DecayFunction::None`×6 … (+34) · dev: `Result`×25, `ContextraError::InvalidInput`×15, `ContextraError::from`×10, `MAX_SEARCH_K`×4 … (+27)
  - → `contextra-vector`: prod: `HnswIndex`, `acorn::FilteredIndex` · dev: `HnswIndex`×44, `HnswConfig`×44
- `contextra_impl` (1269 Zeilen)
  - → `contextra-checkpoint`: prod: `InstanceOrphanRegistry`, `InstanceOrphanRegistry::new`
  - → `contextra-graph`: prod: `CsrGraph::load_from_storage`×2
  - → `contextra-ports`: prod: `MetricsSink`×3, `Reranker`, `license`, `NoopMetricsSink`, `DriftStatusProvider`
  - → `contextra-rank`: prod: `IsotonicCalibrator`
  - → `contextra-store`: prod: `LsmStorage`×3, `tenant_codec::TenantScopedStorage::new`×3, `tenant_codec::TenantScopedStorage`×2, `LsmConfig`
  - → `contextra-types`: prod: `ContextraError::invalid_input`×8, `EntityId`×4, `FusionWeights`×2, `ContextraError::PolicyViolation`×2, `ContextraError::Storage`×2, `ContextraError::Transaction`, `MemoryType`, `GraphTraversalStrategy`, `HybridQuery`, `Result` · dev: `Result`×3
  - → `contextra-vector`: prod: `HnswConfig`
- `decay_controller` (172 Zeilen)
  - → `contextra-ports`: prod: `Clock`
- `export` (343 Zeilen)
  - → `contextra-types`: prod: `TxId::new` · dev: `DocId::from_key`×4
- `extraction` (556 Zeilen)
  - → `contextra-ports`: prod: `LlmTextGenerator`
- `filter` (384 Zeilen)
  - → `contextra-types`: prod: `ContextraError::invalid_input`×2, `ContextraError` · dev: `ContextraError::InvalidInput`×2
- `fusion` (63 Zeilen)
  - → `contextra-rank`: prod: `fusion`, `fusion::fuse_search_results_with_strategy`, `fusion::fuse_search_results_with_signal_strategies`, `fusion::fuse_signals`
  - → `contextra-types`: prod: `FusionWeights`, `FusionStrategy`, `SignalFusionStrategies`
- `import` (129 Zeilen)
  - → `contextra-types`: prod: `ContextraError::invalid_input`×3, `ContextraError::Serialization`×2, `Result`
- `metrics_names` (54 Zeilen)
  - → `contextra-ports`: prod: `MetricsSink`×3
- `temporal_filter` (339 Zeilen)
  - → `contextra-ports`: prod: `Clock`
  - → `contextra-rank`: prod: `SearchResult`×8 · dev: `SearchResult`×2
  - → `contextra-types`: prod: `TxId`
- `transaction` (1505 Zeilen)
  - → `contextra-graph`: prod: `hyperedge::HyperEdge`×2 · dev: `CsrGraph`
  - → `contextra-ports`: prod: `StorageEngine`
  - → `contextra-store`: dev: `LsmStorage`, `LsmConfig`
  - → `contextra-text`: dev: `Language::English`
  - → `contextra-types`: dev: `DocId::new`×4, `DocId`×2, `EntityId`
  - → `contextra-vector`: prod: `HnswIndex` · dev: `HnswIndex`, `HnswConfig`
- `tests/auto_extraction_runtime_mode.rs` (236 Zeilen)
  - → `contextra-types`: dev: `Result`
- `tests/auto_extraction_wiring.rs` (292 Zeilen)
  - → `contextra-types`: dev: `EntityId::from_key`×5
- `tests/auto_triple_extraction.rs` (118 Zeilen)
  - → `contextra-types`: dev: `EntityId::from_key`×2
- `tests/chunker_golden_tests.rs` (618 Zeilen)
  - → `contextra-types`: dev: `DocId`
- `tests/chunker_hardening_tests.rs` (145 Zeilen)
  - → `contextra-types`: dev: `DocId`
- `tests/chunker_robustness.rs` (65 Zeilen)
  - → `contextra-types`: dev: `DocId`
- `tests/collection_tenant_isolation_veto_f10.rs` (26 Zeilen)
  - → `contextra-types`: dev: `DocId`
- `tests/commit_crash_consistency.rs` (296 Zeilen)
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-text`: dev: `Language::English`
  - → `contextra-vector`: dev: `HnswIndex`, `HnswConfig`
- `tests/crud_error_robustness.rs` (110 Zeilen)
  - → `contextra-types`: dev: `ContextraError`, `Result`
- `tests/decay_and_expiry.rs` (159 Zeilen)
  - → `contextra-ports`: dev: `Clock`
  - → `contextra-types`: dev: `Result`×2, `MemoryType`
- `tests/engine_commit_crash_points.rs` (225 Zeilen)
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-text`: dev: `Language::English`
  - → `contextra-vector`: dev: `HnswIndex`, `HnswConfig`
- `tests/engine_recovery_crash_windows.rs` (375 Zeilen)
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-text`: dev: `Language::English`
  - → `contextra-vector`: dev: `HnswIndex`, `HnswConfig`
- `tests/export_import_roundtrip.rs` (92 Zeilen)
  - → `contextra-types`: dev: `ContextraError`, `Result`
- `tests/filter_metadata_eval.rs` (106 Zeilen)
  - → `contextra-types`: dev: `FilterExpr`, `Result`
- `tests/hybrid_adaptive_overfetch.rs` (134 Zeilen)
  - → `contextra-graph`: dev: `CsrGraph`
- `tests/hybrid_pathrag_snapshot.rs` (112 Zeilen)
  - → `contextra-graph`: dev: `CsrGraph`
- `tests/hybrid_query_fusion.rs` (95 Zeilen)
  - → `contextra-types`: dev: `Result`
- `tests/hybrid_signal_failure_semantics.rs` (110 Zeilen)
  - → `contextra-graph`: dev: `CsrGraph`
- `tests/invariant_determinism.rs` (81 Zeilen)
  - → `contextra-types`: dev: `Result`×2
- `tests/invariant_durability.rs` (107 Zeilen)
  - → `contextra-types`: dev: `Result`
- `tests/invariant_isolation.rs` (129 Zeilen)
  - → `contextra-types`: dev: `Result`, `ContextraError::Internal`
- `tests/invariant_tenant_isolation.rs` (127 Zeilen)
  - → `contextra-types`: dev: `Result`×2
- `tests/metrics_instrumentation.rs` (203 Zeilen)
  - → `contextra-ports`: dev: `MetricsSink`
  - → `contextra-types`: dev: `Result`×5
- `tests/multi_index_transaction.rs` (106 Zeilen)
  - → `contextra-types`: dev: `Result`
- `tests/query_builder_scope_acorn.rs` (249 Zeilen)
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-vector`: dev: `acorn`
- `tests/query_builder_time_travel.rs` (82 Zeilen)
  - → `contextra-types`: dev: `Result`
- `tests/rule_based_extraction_determinism.rs` (176 Zeilen)
  - → `contextra-types`: dev: `Result`
- `tests/ssi_read_path_coverage.rs` (171 Zeilen)
  - → `contextra-graph`: dev: `CsrGraph::new`
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-testkit`: dev: `InMemoryStorageEngine`
  - → `contextra-text`: dev: `Language::English`
  - → `contextra-types`: dev: `domain::LinkRelation::Elaborates`, `DistanceMetric::Cosine`
  - → `contextra-vector`: dev: `HnswIndex::try_new`, `HnswConfig`
- `tests/tenant_collection_handle_sharing.rs` (70 Zeilen)
  - → `contextra-types`: dev: `Result`×2, `TenantId`
- `tests/tenant_index_persistence_reload.rs` (93 Zeilen)
  - → `contextra-types`: dev: `Result`×2, `TenantId`
- `tests/tenant_policy_required.rs` (59 Zeilen)
  - → `contextra-types`: dev: `Result`×2
- `tests/tenant_wrapper_coverage.rs` (214 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `Result`
- `tests/tx_fault_injection.rs` (283 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`, `VectorIndex`
  - → `contextra-types`: dev: `Result`×7
- `tests/tx_recovery_and_erasure.rs` (295 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`, `VectorIndex`
  - → `contextra-types`: dev: `Result`×7
- `tests/write_skew_via_engine_api.rs` (207 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `Result`×2, `ContextraError`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `(crate-root)`
  - 2 enum, 1 fn, 5 struct, 3 type
    - `type ConsolidationLauncher` ⚠ fremd: `contextra-store`
    - `fn register_consolidation_launcher` ⚠ fremd: `contextra-store`
    - `struct Document`
    - `struct ContextraStats` ⚠ fremd: `contextra-ports`
    - `type DbStats`
    - `struct CommunityDetectionConfig`
    - `enum EmbeddingBackend`
    - `enum TenantPolicy`
    - `struct ContextraConfig` · 3 Methoden ⚠ fremd: `contextra-ports`, `contextra-store`, `contextra-types`
    - `type TenantCollectionMap` ⚠ fremd: `contextra-store`, `contextra-types`
    - `struct Contextra` · 3 Methoden
- `adapters`
  - 1 struct
    - `struct Bm25Scorer` · 1 Methoden
- `backends`
  - 1 struct, 1 trait
    - `trait Backends`
    - `struct DefaultBackends`
- `background_workers`
  - 1 fn
    - `fn start_decay_cleanup_worker` *(re-exportiert)* ⚠ fremd: `contextra-ports`
- `chunker`
  - 1 fn, 2 struct
    - `struct ChunkerConfig` *(re-exportiert)*
    - `fn estimate_tokens` *(re-exportiert)*
    - `struct MarkdownChunker` · 3 Methoden
- `collection`
  - 4 const, 1 enum, 5 fn, 7 struct
    - `const MAX_SCAN_RESULTS` *(re-exportiert)*
    - `fn sorted_unique_shards`
    - `struct PercolationResult`
    - `struct CollectionConfig`
    - `struct StoredDocument`
    - `struct StoredDocumentMeta`
    - `fn parse_importance_score`
    - `fn compute_default_importance` ⚠ fremd: `contextra-types`
    - `fn ensure_importance_metadata`
    - `fn extract_effective_importance`
    - `struct Collection` · 34 Methoden ⚠ fremd: `contextra-store`, `contextra-vector`
    - `const METRIC_SEARCH_LATENCY_VECTOR`
    - `const METRIC_SEARCH_LATENCY_TEXT`
    - `const METRIC_SEARCH_LATENCY_GRAPH`
    - `struct ScopeConstraint` *(re-exportiert)* · 2 Methoden ⚠ fremd: `contextra-types`
    - `enum SearchStrategy` *(re-exportiert)* · 3 Methoden ⚠ fremd: `contextra-types`
    - `struct SignalWeights` *(re-exportiert)* · 1 Methoden
- `decay_controller`
  - 3 struct
    - `struct DecaySignalInputs`
    - `struct DecayControllerConfig`
    - `struct AdaptiveDecayController` · 11 Methoden
- `export`
  - 1 const, 4 struct
    - `const SCHEMA_VERSION_V1`
    - `struct ExportDocumentV1`
    - `struct ExportCollectionV1`
    - `struct ExportMemoryV1`
    - `struct ExportRelationV1`
- `extraction`
  - 2 const, 1 enum, 4 fn, 4 struct, 1 trait
    - `trait EntityExtractor` · Methoden: extract_triples
    - `struct OpenIeExtractor`
    - `fn create_extractor`
    - `fn extract_triples_for_mode`
    - `fn extract_triples_for_config`
    - `fn extract_triples` ⚠ fremd: `contextra-ports`
    - `const MAX_EXTRACTION_TEXT_LENGTH`
    - `const MAX_EXTRACTED_TRIPLES`
    - `struct RuleBasedExtractor` · 2 Methoden
    - `struct ExtractedTriple`
    - `struct EntityExtractionConfig`
    - `enum ExtractorMode`
- `filter`
  - 2 const, 2 enum, 2 fn
    - `const MAX_FILTER_DEPTH`
    - `const MAX_FILTER_NODES`
    - `fn validate_filter_expr_bounds`
    - `fn extract_memory_type`
    - `enum FilterOp`
    - `enum MetadataFilter` · 2 Methoden
- `fusion`
  - 4 fn
    - `fn weights_to_signal_factors` ⚠ fremd: `contextra-types`
    - `fn fuse_search_results_with_strategy` ⚠ fremd: `contextra-types`
    - `fn fuse_search_results_with_signal_strategies` ⚠ fremd: `contextra-types`
    - `fn fuse_signals`
- `import`
  - 1 const, 1 struct
    - `struct ImportSummary`
    - `const MAX_IMPORT_COLLECTIONS`
- `metrics_names`
  - 9 const, 3 fn
    - `const COMMIT_LATENCY`
    - `const REBUILD_DURATION`
    - `const BACKPRESSURE_LEVEL`
    - `const CHECKPOINT_DURATION`
    - `const SEARCH_LATENCY_VECTOR`
    - `const SEARCH_LATENCY_TEXT`
    - `const SEARCH_LATENCY_GRAPH`
    - `const INSERT_LATENCY`
    - `const REAPER_DOCS_REAPED`
    - `fn record_histogram_safe` ⚠ fremd: `contextra-ports`
    - `fn record_gauge_safe` ⚠ fremd: `contextra-ports`
    - `fn record_counter_safe` ⚠ fremd: `contextra-ports`
- `orchestration`
  - 8 struct, 1 trait
    - `struct EngineServiceContainer` · 4 Methoden
- `temporal_filter`
  - 5 fn, 1 struct, 1 type
- `transaction`
  - 1 enum, 1 struct
  - … 17 weitere Items nicht aufgelistet (`--max-items`)
- Zusätzlich 40 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).
- ⚠ Re-Exporte **fremder Crates**: `contextra_types::AutoExtractionMode`, `contextra_graph::percolation::PercolationConfig`, `contextra_ports::TextEmbeddingEngine`, `contextra_ports::VectorDeleteMode`, `contextra_store::lsm::DurabilityMode`, `contextra_checkpoint`, `contextra_text::Language`, `contextra_ports::DriftStatusProvider`, `contextra_types::DistanceMetric`

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-db` | `(crate)`, `DurabilityMode`, `decay_controller::*`, `fusion`, `collection` | – |

*Hinweis: Mindestens ein Crate importiert per Glob (`*`); ungenutzte Items sind deshalb nicht bestimmbar.*

### contextra-graph

- Pfad: `crates/contextra-graph` · Zeilen: 28667 · Rust-Dateien: 133 · Ring: 0
- Interne Cargo-Abhängigkeiten: `contextra-ports` (normal/optional via types-compat), `contextra-types` (normal/optional via types-compat)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-types` | 46 | 60 | 28 | `EntityId`×18, `ContextraError::InvalidInput`×5, `Result`×5, `ContextraError`×3, `TxId`×3, `Edge`×2, `Edge::new`×2, `PprConfig`×2, `ContextraError::Internal`×2, `MemoryLink` … (+3) | `Entity::new`×10, `TlHfdParams::default`×4, `PprAlgorithm::Auto`×4, `Entity`×2, `TenantId::new`×2 |
| `contextra-ports` | 29 | 44 | 12 | `BoxFuture`×5, `GraphIndex`×4, `StorageEngine`×2, `graph::HyperEdgeId::new`×2, `GraphIndex::personalized_page_rank_at`, `GraphIndex::rollback_to_tx`, `graph::RoleBinding`, `graph::HyperEdgeId`, `graph::GraphMutationError`, `graph::GraphMutationError::LockAcquisitionTimeout` … (+10) | `Result`×3, `graph_index::GraphIndex` |

**Pro Komponente**

- `(crate-root)` (133 Zeilen)
  - → `contextra-ports`: prod: `GraphIndex`, `BoxFuture`
  - → `contextra-types`: prod: `TxId`, `Result`
- `apprh` (982 Zeilen)
  - → `contextra-types`: prod: `EntityId`×4, `ContextraError`×2, `ContextraError::InvalidInput`×2 · dev: `Entity`
- `cascade` (939 Zeilen)
  - → `contextra-ports`: dev: `GraphIndex`
  - → `contextra-types`: prod: `ContextraError::InvalidInput`×2 · dev: `Entity::new`×10
- `community` (1292 Zeilen)
  - → `contextra-ports`: dev: `GraphIndex`
- `csr` (6098 Zeilen)
  - → `contextra-ports`: prod: `BoxFuture`×4, `graph::HyperEdgeId::new`×2, `StorageEngine`, `GraphIndex::personalized_page_rank_at`, `GraphIndex::rollback_to_tx`, `graph::RoleBinding`, `graph::HyperEdgeId`, `graph::GraphMutationError`, `graph::GraphMutationError::LockAcquisitionTimeout`, `graph::GraphMutationError::RoleBindingInvalid` … (+7) · dev: `GraphIndex`×3
  - → `contextra-types`: prod: `EntityId`×5, `Edge`×2, `Edge::new`×2, `PprConfig`×2, `Result`×2, `TxId`×2, `MemoryLink`, `LinkRelation::Supersedes` · dev: `Result`
- `edge_reinforcement` (148 Zeilen)
  - → `contextra-types`: dev: `EntityId`
- `edge_reinforcement_buffer` (191 Zeilen)
  - → `contextra-ports`: dev: `GraphIndex`
  - → `contextra-types`: prod: `EntityId` · dev: `EntityId`
- `entity_extraction` (285 Zeilen)
  - → `contextra-types`: prod: `ContextraError::Internal`
- `hyperedge` (711 Zeilen)
  - → `contextra-ports`: prod: `graph`
- `hyperedge_suggest` (160 Zeilen)
  - → `contextra-ports`: prod: `LlmTextGenerator`
- `path_rag` (1418 Zeilen)
  - → `contextra-types`: prod: `EntityId`×2, `Result`×2, `DocId`
- `percolation` (285 Zeilen)
  - → `contextra-ports`: prod: `GraphIndex` · dev: `GraphIndex`
  - → `contextra-types`: prod: `EntityId`
- `ppr` (3586 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`×3, `GraphIndex`×2
  - → `contextra-types`: prod: `TlHfdParams`, `EntityId` · dev: `TlHfdParams::default`×4, `TlHfdParams`
- `ppr_stream` (145 Zeilen)
  - → `contextra-ports`: prod: `GraphIndex`
  - → `contextra-types`: prod: `ContextraError::Internal`
- `provenance` (165 Zeilen)
  - → `contextra-types`: dev: `EntityId`
- `purger` (96 Zeilen)
  - → `contextra-ports`: prod: `scanner` · dev: `GraphIndex`
  - → `contextra-types`: dev: `TenantId::new`×2
- `session_dag` (883 Zeilen)
  - → `contextra-ports`: prod: `StorageEngine`
- `tl_hfd` (748 Zeilen)
  - → `contextra-types`: prod: `EntityId`×4, `ContextraError`, `ContextraError::InvalidInput` · dev: `Entity`
- `benches/csr_traversal_bench.rs` (201 Zeilen)
  - → `contextra-ports`: dev: `GraphIndex`
  - → `contextra-types`: dev: `Result`×4, `Edge::new`×2
- `examples/isolated_graph_traversal.rs` (54 Zeilen)
  - → `contextra-ports`: dev: `GraphIndex`
- `tests/algorithm_test_doubles.rs` (55 Zeilen)
  - → `contextra-types`: dev: `EntityId`
- `tests/apprh_determinism_permutation.rs` (129 Zeilen)
  - → `contextra-types`: dev: `EntityId`
- `tests/apprh_gate_monitor.rs` (291 Zeilen)
  - → `contextra-types`: dev: `EntityId`
- `tests/apprh_selector_shadow_to_production.rs` (220 Zeilen)
  - → `contextra-types`: dev: `EntityId`
- `tests/apprh_tests.rs` (215 Zeilen)
  - → `contextra-types`: dev: `EntityId`
- `tests/budgeted_k_path_hub_entity.rs` (151 Zeilen)
  - → `contextra-types`: dev: `EntityId`
- `tests/community_hyperedges_included_flag.rs` (127 Zeilen)
  - → `contextra-ports`: dev: `GraphIndex`
- `tests/csr_benchmark.rs` (163 Zeilen)
  - → `contextra-ports`: dev: `GraphIndex`
- `tests/csr_complexity_bench.rs` (152 Zeilen)
  - → `contextra-ports`: dev: `GraphIndex`
- `tests/dangling_nodes_audit_test.rs` (245 Zeilen)
  - → `contextra-ports`: dev: `GraphIndex`
  - → `contextra-types`: dev: `PprAlgorithm::Auto`×3
- `tests/diffusion_shadow_gate_thresholds.rs` (87 Zeilen)
  - → `contextra-types`: dev: `EntityId`
- `tests/entity_extraction_edge_cases.rs` (127 Zeilen)
  - → `contextra-types`: dev: `DocId`
- `tests/entity_extraction_tests.rs` (81 Zeilen)
  - → `contextra-types`: dev: `DocId`
- `tests/golden_ports_type_compatibility.rs` (60 Zeilen)
  - → `contextra-types`: dev: `EntityId`
- `tests/graph_collection_mutation_trait_test.rs` (68 Zeilen)
  - → `contextra-ports`: dev: `graph`
- `tests/hub_node_benchmark.rs` (96 Zeilen)
  - → `contextra-ports`: dev: `GraphIndex`
  - → `contextra-types`: dev: `Edge::new`×2
- `tests/hyperedge_cascade_crash_recovery.rs` (151 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/hyperedge_cascade_queue_persistence.rs` (69 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/hyperedge_memory_budget.rs` (96 Zeilen)
  - → `contextra-types`: dev: `EntityId`
- `tests/hyperedge_payload_sharing.rs` (37 Zeilen)
  - → `contextra-types`: dev: `EntityId`
- `tests/hyperedge_persistence_survives_restart.rs` (100 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/hyperedge_suggest.rs` (161 Zeilen)
  - → `contextra-ports`: dev: `Result`
- `tests/hyperedge_suggest_llm_test.rs` (117 Zeilen)
  - → `contextra-ports`: dev: `Result`
- `tests/hyperedge_suggest_tests.rs` (329 Zeilen)
  - → `contextra-ports`: dev: `Result`
- `tests/integration_graph.rs` (116 Zeilen)
  - → `contextra-ports`: dev: `GraphIndex`
- `tests/leiden_reference_validation.rs` (155 Zeilen)
  - → `contextra-ports`: dev: `GraphIndex`
- `tests/path_rag_at_snapshot.rs` (311 Zeilen)
  - → `contextra-ports`: dev: `GraphIndex`
- `tests/ppr_alloc_test.rs` (112 Zeilen)
  - → `contextra-types`: dev: `PprAlgorithm::Auto`
- `tests/ppr_candidate_stream.rs` (315 Zeilen)
  - → `contextra-ports`: dev: `BoxFuture`×9
  - → `contextra-types`: dev: `Result`×8
- `tests/ppr_reference_validation.rs` (265 Zeilen)
  - → `contextra-ports`: dev: `GraphIndex`
- `tests/ppr_snapshot_isolation.rs` (356 Zeilen)
  - → `contextra-ports`: dev: `GraphIndex`
- `tests/ppr_strategy_equivalence.rs` (97 Zeilen)
  - → `contextra-ports`: dev: `graph_index::GraphIndex`
- `tests/proptest_csr_invariants.rs` (334 Zeilen)
  - → `contextra-ports`: dev: `GraphIndex`
- `tests/proptest_tl_hfd_diffusion.rs` (276 Zeilen)
  - → `contextra-types`: dev: `EntityId`
- `tests/proptest_tl_hfd_lovasz.rs` (267 Zeilen)
  - → `contextra-types`: dev: `EntityId`
- `tests/provider_tests.rs` (115 Zeilen)
  - → `contextra-types`: dev: `EntityId`
- `tests/source_doc_ids_populated.rs` (154 Zeilen)
  - → `contextra-ports`: dev: `GraphIndex`
- `tests/star_expansion_equals_clique.rs` (48 Zeilen)
  - → `contextra-types`: dev: `EntityId`
- `tests/tlhfd_shadow_mode_parity.rs` (288 Zeilen)
  - → `contextra-types`: dev: `EntityId`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `(crate-root)`
  - 1 trait
    - `trait GraphIndexExt` · Methoden: remove_entity ⚠ fremd: `contextra-ports`
- `apprh`
  - 2 enum, 4 fn, 9 struct, 1 trait
    - `fn run_apprh_push` ⚠ fremd: `contextra-types`
    - `enum ApprhError`
    - `struct ApprhGateMonitorConfig` · 1 Methoden
    - `struct ApprhObservationRecord`
    - `struct ApprhGateMonitorSnapshot`
    - `struct ApprhGateMonitor` · 7 Methoden
    - `fn apprh_local` ⚠ fremd: `contextra-types`
    - `fn forward_push_apprh` ⚠ fremd: `contextra-types`
    - `struct ApprhParams` · 1 Methoden
    - `enum ApprhMode`
    - `struct ApprhSelectorConfig`
    - `struct ApprhSelector` · 8 Methoden
    - `trait ApprhFlipGate` · Methoden: should_flip
    - `struct DefaultApprhFlipGate`
    - `struct ApprhShadowComparison` ⚠ fremd: `contextra-types`
    - `fn shadow_compare_forward_push_vs_apprh` ⚠ fremd: `contextra-types`
- `arc_slice`
  - 1 struct
    - `struct ArcSlice` · 9 Methoden
- `cascade`
  - 3 const, 1 enum, 4 fn, 4 struct, 1 trait, 1 type
    - `const MAX_HYPEREDGE_CASCADE_FANOUT`
    - `const DEFAULT_HYPEREDGE_CASCADE_FANOUT_LIMIT`
    - `struct CascadeInvalidationReport`
    - `fn cascade_invalidate_edges_for_superseded_doc`
    - `const CASCADE_QUEUE_PREFIX`
    - `enum CascadeStatus`
    - `struct CascadeDeletionRecord`
    - `struct CascadeTicket`
    - `trait GraphGarbageCollection` · Methoden: sweep_orphans
    - `struct HyperedgeCascadeReport`
    - `type CascadeReport`
    - `fn topological_sort_hyperedge_closure`
    - `fn cascade_invalidate_hyperedges_for_superseded_doc`
    - `fn verify_doc_hyperedge_deletion_completeness`
- `community`
  - 1 fn, 4 struct
    - `struct CommunityDetectionConfig`
    - `struct VirtualHyperedgeNode` · 1 Methoden
    - `struct StarExpansionIterator` · 2 Methoden
    - `struct CommunityAssignment`
    - `fn detect_communities`
- `consistency_enforcement`
  - 4 struct, 1 trait, 1 type
    - `type EdgeId`
    - `struct ConflictPattern`
    - `struct EdgeAssertion` · 1 Methoden
    - `trait ContradictionDetector` · Methoden: conflicts
    - `struct ExactPredicateConflictDetector`
    - `struct ConsistencyEnforcer` · 9 Methoden
- `contract`
  - 1 enum
    - `enum Error` *(re-exportiert)*
- `csr`
  - 2 struct
    - `struct CsrGraph` *(re-exportiert)* · 16 Methoden
    - `struct MemoryEstimate` *(re-exportiert)* · 1 Methoden
- `edge_reinforcement`
  - 4 fn, 1 struct
    - `struct EdgeReinforcementConfig`
    - `fn compute_edge_weight`
    - `fn apply_cooccurrence_reinforcement`
    - `fn apply_traversal_reinforcement`
    - `fn apply_weight_normalization`
- `edge_reinforcement_buffer`
  - 3 struct
    - `struct CooccurrenceSignal` ⚠ fremd: `contextra-types`
    - `struct TraversalSignal` ⚠ fremd: `contextra-types`
    - `struct EdgeReinforcementBuffer` · 6 Methoden
- `entity_extraction`
  - 1 fn, 2 struct, 1 trait
    - `struct ExtractedRelation` · 1 Methoden
    - `trait EntityExtractor` · Methoden: extract
    - `struct RuleBasedEntityExtractor` · 1 Methoden
    - `fn ingest_extracted_relations`
- `error`
  - 2 enum, 1 type
    - `type Result`
    - `enum GraphError`
    - `enum GraphMutationError`
- `hyperedge`
  - 2 const, 2 fn, 4 struct
    - `const HYPEREDGE_PREFIX`
    - `const HYPEREDGE_BY_ENTITY_PREFIX`
    - `fn star_weight`
    - `struct HyperEdge` · 9 Methoden
    - `struct HyperEdgeView` · 2 Methoden
    - `struct RoleInterner` · 8 Methoden
    - `fn sort_dedup_entities`
    - `struct ConsolidationNodesGuard` · 4 Methoden
- `hyperedge_suggest`
  - 1 const, 1 enum, 2 fn, 2 struct
    - `const MAX_RELATE_PARTICIPANTS`
    - `struct HyperEdgeCandidate`
    - `struct ValidatedHyperEdgeCandidate`
    - `enum HyperedgeSuggestError`
    - `fn compute_co_occurrence_candidates`
    - `fn validate_candidates_with_llm` ⚠ fremd: `contextra-ports`
- `path_rag`
  - 1 const, 1 fn, 7 struct, 2 trait
    - `struct KPathConfig`
    - `struct KPathResult` ⚠ fremd: `contextra-types`
    - `trait KPathDiffusion` · Methoden: find_k_paths
    - `struct GraphPath` ⚠ fremd: `contextra-types`
    - `trait PathGraph` · Methoden: neighbors_with_weights, predecessors_with_weights, hyperedges_for_entity, get_hyperedge, hyperedge_participants
- `percolation`
  - 3 fn, 1 struct
- `ppr`
  - 3 fn, 7 struct, 1 trait
- `ppr_stream`
  - 1 const, 1 struct
- `provenance`
  - 2 struct
- `provider`
  - 2 fn, 1 struct, 1 trait
- `purger`
  - 1 fn, 1 struct
- `session_dag`
  - 2 const, 6 struct, 1 type
- `test_doubles`
  - 3 struct
- `tl_hfd`
  - 2 const, 1 enum, 5 fn, 4 struct, 1 trait
- `types`
  - 2 struct
  - … 58 weitere Items nicht aufgelistet (`--max-items`)
- Zusätzlich 35 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).
- ⚠ Re-Exporte **fremder Crates**: `contextra_types::EntityId`

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-engine` | `hyperedge::HyperEdgeId`×2, `percolation::PercolationConfig`×2, `CsrGraph::load_from_storage`×2, `hyperedge::HyperEdge`×2, `cascade_invalidate_edges_for_superseded_doc`, `percolation::compute_percolation_health`, `percolation::should_trigger_rebonding`, `percolation::find_rebonding_candidates`, `CsrGraph`, `hyperedge::RoleId::new` … (+6) | `csr::CsrGraph`×15, `csr`, `CsrGraph::new` |
| `contextra-db` | – | `CsrGraph`×9, `csr::CsrGraph`×2, `path_rag::PathRAGEngine`, `PathRAGEngine` |

**Nach außen ungenutzte öffentliche Items (Heuristik, 129 von 138)**: `run_apprh_push`, `ApprhError`, `ApprhGateMonitorConfig`, `ApprhObservationRecord`, `ApprhGateMonitorSnapshot`, `ApprhGateMonitor`, `apprh_local`, `forward_push_apprh`, `ApprhParams`, `ApprhMode`, `ApprhSelectorConfig`, `ApprhSelector`, `ApprhFlipGate`, `DefaultApprhFlipGate`, `ApprhShadowComparison`, `shadow_compare_forward_push_vs_apprh`, `ArcSlice`, `MAX_HYPEREDGE_CASCADE_FANOUT`, `DEFAULT_HYPEREDGE_CASCADE_FANOUT_LIMIT`, `CascadeInvalidationReport`, `CASCADE_QUEUE_PREFIX`, `CascadeStatus`, `CascadeDeletionRecord`, `CascadeTicket`, `GraphGarbageCollection`, `HyperedgeCascadeReport`, `CascadeReport`, `topological_sort_hyperedge_closure`, `cascade_invalidate_hyperedges_for_superseded_doc`, `verify_doc_hyperedge_deletion_completeness` …

### contextra-mcp

- Pfad: `crates/contextra-mcp` · Zeilen: 7712 · Rust-Dateien: 30 · Ring: 4
- Interne Cargo-Abhängigkeiten: `contextra` (normal), `contextra-crypto` (normal), `contextra-ports` (normal), `contextra-rank` (normal), `contextra-types` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra` | 12 | 9 | 4 | `Contextra`×2, `chunker`, `chunker::MarkdownChunker::new`, `chunker::ChunkerConfig::default`, `collection_profile`, `collection_profile::DeploymentTier::EdgeMinimal`, `collection_profile::DeploymentTier::PowerUserLocal`, `collection_profile::DeploymentTier::EnterpriseShared`, `collection_profile::DeploymentTier::EnterpriseRegulated`, `collection_profile::AutoExtractionMode::Enabled` … (+1) | `collection_profile::DeploymentTier` |
| `contextra-types` | 6 | 4 | 3 | `ContextraError`×4, `ContextraError::InvalidInput`, `ContextraError::NotFound` | `TenantId::try_new`, `AutoExtractionMode`, `TenantId`, `MAX_SEARCH_K` |
| `contextra-crypto` | 4 | 3 | 1 | `CryptoKey`×3, `CryptoKey::try_new` | `AuditChain` |
| `contextra-ports` | 3 | 39 | 1 | `plugin::PluginRegistry`×2, `EmbeddingProvider` | `EmbeddingError`×17, `BoxFuture`×12, `Clock`×2 |
| `contextra-rank` | 1 | 0 | 1 | `fusion::ProvenanceRecord` | – |

**Pro Komponente**

- `bin` (62 Zeilen)
  - → `contextra`: prod: `Contextra`
- `config` (672 Zeilen)
  - → `contextra-ports`: dev: `BoxFuture`×2, `EmbeddingError`
  - → `contextra-types`: prod: `ContextraError`
- `explain` (239 Zeilen)
  - → `contextra`: dev: `Contextra`
  - → `contextra-ports`: dev: `EmbeddingError`×2, `BoxFuture`, `EmbeddingProvider`
  - → `contextra-rank`: prod: `fusion::ProvenanceRecord`
- `prompt_injection` (1088 Zeilen)
  - → `contextra-ports`: dev: `Clock`×2
- `protocol` (173 Zeilen)
  - → `contextra-types`: prod: `ContextraError`×2, `ContextraError::InvalidInput`, `ContextraError::NotFound`
- `sandbox` (697 Zeilen)
  - → `contextra-crypto`: prod: `CryptoKey`×3, `CryptoKey::try_new` · dev: `CryptoKey::try_new`×2
  - → `contextra-types`: prod: `ContextraError`
- `server` (139 Zeilen)
  - → `contextra`: prod: `Contextra`
  - → `contextra-ports`: prod: `plugin::PluginRegistry`×2, `EmbeddingProvider`
- `server_tools` (344 Zeilen)
  - → `contextra`: prod: `chunker`, `chunker::MarkdownChunker::new`, `chunker::ChunkerConfig::default`
- `tests` (872 Zeilen)
  - → `contextra`: dev: `Contextra`
  - → `contextra-ports`: dev: `EmbeddingError`×2, `BoxFuture`, `EmbeddingProvider`
  - → `contextra-types`: dev: `TenantId::try_new`
- `tools_crud` (171 Zeilen)
  - → `contextra`: prod: `collection_profile`, `collection_profile::DeploymentTier::EdgeMinimal`, `collection_profile::DeploymentTier::PowerUserLocal`, `collection_profile::DeploymentTier::EnterpriseShared`, `collection_profile::DeploymentTier::EnterpriseRegulated`, `collection_profile::AutoExtractionMode::Enabled`, `collection_profile::AutoExtractionMode::Disabled`
- `tests/campaign_j42_mcp_audit.rs` (56 Zeilen)
  - → `contextra`: dev: `collection_profile::DeploymentTier`
  - → `contextra-crypto`: dev: `AuditChain`
  - → `contextra-types`: dev: `AutoExtractionMode`
- `tests/confirm_required_semantics.rs` (143 Zeilen)
  - → `contextra`: dev: `Contextra`
  - → `contextra-ports`: dev: `BoxFuture`×2, `EmbeddingError`×2, `EmbeddingProvider`
- `tests/dispatch_direct_tests.rs` (326 Zeilen)
  - → `contextra`: dev: `Contextra`
  - → `contextra-ports`: dev: `EmbeddingError`×2, `BoxFuture`, `EmbeddingProvider`
- `tests/mcp_test.rs` (1085 Zeilen)
  - → `contextra`: dev: `Contextra`
  - → `contextra-ports`: dev: `EmbeddingError`×2, `BoxFuture`, `EmbeddingProvider`
- `tests/server_lifecycle_tests.rs` (167 Zeilen)
  - → `contextra`: dev: `Contextra`
  - → `contextra-ports`: dev: `EmbeddingError`×2, `BoxFuture`, `EmbeddingProvider`
  - → `contextra-types`: dev: `TenantId`
- `tests/tool_registry_classification.rs` (158 Zeilen)
  - → `contextra`: dev: `Contextra`
  - → `contextra-ports`: dev: `BoxFuture`×2, `EmbeddingError`×2, `EmbeddingProvider`
- `tests/tools_crud_direct_tests.rs` (484 Zeilen)
  - → `contextra`: dev: `Contextra`
  - → `contextra-ports`: dev: `EmbeddingError`×2, `BoxFuture`, `EmbeddingProvider`
  - → `contextra-types`: dev: `MAX_SEARCH_K`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `config`
  - 4 fn, 2 struct
    - `fn available_compiled_providers`
    - `fn verify_embedding_provider` ⚠ fremd: `contextra-types`
    - `struct EmbeddingConfig` · 4 Methoden
    - `fn create_embedding_provider` ⚠ fremd: `contextra-types`
    - `struct LlmConfig` · 3 Methoden
    - `fn create_llm_text_generator` ⚠ fremd: `contextra-types`
- `explain`
  - 2 struct
    - `struct ExplainRequest`
    - `struct ExplainResponse` ⚠ fremd: `contextra-rank`
- `io`
  - 2 const, 1 fn
    - `const MAX_RPC_BYTES`
    - `const MAX_SEARCH_QUERY_BYTES`
    - `fn read_line_bounded`
- `protocol`
  - 2 const, 1 enum, 3 fn, 3 struct
    - `const MAX_JSON_DEPTH`
    - `const MAX_BATCH_SIZE`
    - `enum McpError` · 6 Methoden
    - `struct JsonRpcRequest`
    - `struct JsonRpcError`
    - `struct JsonRpcResponse` · 2 Methoden
    - `fn measure_json_depth`
    - `fn validate_json_depth`
    - `fn response_from_error`
- `sandbox`
  - 4 const, 1 enum, 4 struct
    - `const MAX_VOLATILE_RESULTS`
    - `const MAX_VOLATILE_KEY_BYTES`
    - `const MAX_VOLATILE_OUTPUT_BYTES`
    - `struct ToolDefinition`
    - `const TOOL_REGISTRY`
    - `enum ToolCategory`
    - `struct SandboxPolicy`
    - `struct VolatileToolResult` · 2 Methoden
    - `struct McpSandbox` · 8 Methoden
- `server`
  - 1 struct
    - `struct McpServer` · 7 Methoden ⚠ fremd: `contextra`, `contextra-ports`
- `validation`
  - 2 fn
    - `fn is_write_allowed_by_env`
    - `fn validate_collection_name`
- Zusätzlich 9 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).

**Eingehend (wer nutzt dieses Crate)**

Wird von keinem anderen analysierten Workspace-Crate importiert.

**Nach außen ungenutzte öffentliche Items (Heuristik, 32 von 32)**: `available_compiled_providers`, `verify_embedding_provider`, `EmbeddingConfig`, `create_embedding_provider`, `LlmConfig`, `create_llm_text_generator`, `ExplainRequest`, `ExplainResponse`, `MAX_RPC_BYTES`, `MAX_SEARCH_QUERY_BYTES`, `read_line_bounded`, `MAX_JSON_DEPTH`, `MAX_BATCH_SIZE`, `McpError`, `JsonRpcRequest`, `JsonRpcError`, `JsonRpcResponse`, `measure_json_depth`, `validate_json_depth`, `response_from_error`, `MAX_VOLATILE_RESULTS`, `MAX_VOLATILE_KEY_BYTES`, `MAX_VOLATILE_OUTPUT_BYTES`, `ToolDefinition`, `TOOL_REGISTRY`, `ToolCategory`, `SandboxPolicy`, `VolatileToolResult`, `McpSandbox`, `McpServer` …

### contextra-mvcc

- Pfad: `crates/contextra-mvcc` · Zeilen: 6532 · Rust-Dateien: 24 · Ring: 0
- Interne Cargo-Abhängigkeiten: `contextra-testkit` (dev), `contextra-ports` (normal), `contextra-types` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-ports` | 3 | 1 | 1 | `MetricsSink`×3 | – |
| `contextra-testkit` | 0 | 6 | 0 | – | `ManualClock`×6 |
| `contextra-types` | 0 | 7 | 0 | – | `TxId`×5, `DocId`, `types` |

**Pro Komponente**

- `seq_log` (738 Zeilen)
  - → `contextra-testkit`: dev: `ManualClock`
- `snapshot` (825 Zeilen)
  - → `contextra-testkit`: dev: `ManualClock`
- `ssi` (1069 Zeilen)
  - → `contextra-ports`: prod: `MetricsSink`×3
- `tx_buffer` (1281 Zeilen)
  - → `contextra-testkit`: dev: `ManualClock`
- `benches/mvcc_bench.rs` (125 Zeilen)
  - → `contextra-types`: dev: `TxId`×2, `DocId`
- `examples/mvcc_basic.rs` (63 Zeilen)
  - → `contextra-types`: dev: `types`
- `tests/b16_reproduction.rs` (113 Zeilen)
  - → `contextra-testkit`: dev: `ManualClock`
- `tests/campaign_cov_ssi_coarsening.rs` (51 Zeilen)
  - → `contextra-types`: dev: `TxId`
- `tests/campaign_cov_tx_buffer_reap.rs` (61 Zeilen)
  - → `contextra-testkit`: dev: `ManualClock`
  - → `contextra-types`: dev: `TxId`
- `tests/history_anomaly_checker.rs` (266 Zeilen)
  - → `contextra-types`: dev: `TxId`
- `tests/isolation_tests.rs` (712 Zeilen)
  - → `contextra-testkit`: dev: `ManualClock`
- `tests/ssi_metrics_observability.rs` (130 Zeilen)
  - → `contextra-ports`: dev: `MetricsSink`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `seq_log`
  - 1 const, 1 enum, 2 struct
    - `const DEFAULT_MAX_PIN_DURATION`
    - `struct SeqLogEntry` · 1 Methoden
    - `enum SeqLogChange` · 2 Methoden
    - `struct SequenceLog` · 25 Methoden
- `snapshot`
  - 3 struct
    - `struct SnapshotRegistryConfig`
    - `struct SnapshotRegistry` · 29 Methoden
    - `struct SnapshotGuard` · 2 Methoden
- `ssi`
  - 1 const, 3 struct, 1 trait
    - `const DEFAULT_MAX_TRACKED_COMMIT_KEYS`
    - `struct ReadSet` · 13 Methoden
    - `trait SsiValidator` · Methoden: validate
    - `struct PruningBlockerInfo`
    - `struct SequenceLogSsiValidator` · 25 Methoden
- `tx_buffer`
  - 6 const, 1 enum, 2 struct, 1 trait
    - `const STAGING_ENTRY_OVERHEAD_BYTES`
    - `const DEFAULT_SHARD_COUNT`
    - `const DEFAULT_MAX_OPS_PER_TX`
    - `const DEFAULT_MAX_READ_SET_KEYS`
    - `const DEFAULT_MAX_TX_STAGED_BYTES`
    - `const DEFAULT_MAX_TOTAL_STAGED_BYTES`
    - `struct TxBufferConfig`
    - `enum IndexOp` · 2 Methoden
    - `trait StagedOpSize` · Methoden: staged_bytes
    - `struct TxBuffer` · 37 Methoden
- Zusätzlich 2 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-vector` | `IndexOp::Insert`×2, `IndexOp::Delete`×2, `SequenceLog::new`×2, `SeqLogChange::Insert`, `SeqLogChange::Delete`, `TxBuffer`, `TxBuffer::new_with_config`, `SequenceLog` | – |
| `contextra-store` | `SsiValidator`×4, `tx_buffer::STAGING_ENTRY_OVERHEAD_BYTES`×2, `SnapshotRegistry`, `SequenceLogSsiValidator`, `IndexOp`, `SequenceLogSsiValidator::new_with_bounds` | `SnapshotRegistry::new` |
| `contextra-checkpoint` | `SnapshotRegistry` | – |
| `contextra-engine` | `tx_buffer::TxBuffer` | `tx_buffer::IndexOp` |

**Nach außen ungenutzte öffentliche Items (Heuristik, 15 von 22)**: `DEFAULT_MAX_PIN_DURATION`, `SeqLogEntry`, `SeqLogChange`, `SnapshotRegistryConfig`, `SnapshotGuard`, `DEFAULT_MAX_TRACKED_COMMIT_KEYS`, `ReadSet`, `PruningBlockerInfo`, `DEFAULT_SHARD_COUNT`, `DEFAULT_MAX_OPS_PER_TX`, `DEFAULT_MAX_READ_SET_KEYS`, `DEFAULT_MAX_TX_STAGED_BYTES`, `DEFAULT_MAX_TOTAL_STAGED_BYTES`, `TxBufferConfig`, `StagedOpSize`

**Auffälligkeiten**

- contextra-types: nur in Tests/Benches genutzt, aber als normale Abhängigkeit deklariert → dev-dependency?

### contextra-ports

- Pfad: `crates/contextra-ports` · Zeilen: 6796 · Rust-Dateien: 42 · Ring: 0
- Interne Cargo-Abhängigkeiten: `contextra-types` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-types` | 3 | 0 | 3 | `RequestContext`×2, `ids::ErasureId` | – |

**Pro Komponente**

- `observer` (141 Zeilen)
  - → `contextra-types`: prod: `ids::ErasureId`
- `policy` (30 Zeilen)
  - → `contextra-types`: prod: `RequestContext`
- `scanner` (483 Zeilen)
  - → `contextra-types`: prod: `RequestContext`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `(crate-root)`
  - 2 type
    - `type BoxFuture`
    - `type BoxStream`
- `attention`
  - 1 struct, 1 trait
    - `struct RequestId`
    - `trait AttentionExporter` · Methoden: export_attention_weights
- `barrier`
  - 4 struct, 2 trait
    - `struct BarrierShared` · 1 Methoden
    - `struct BarrierExclusive` · 1 Methoden
    - `struct CompactionPermit` · 1 Methoden
    - `struct PurgePermit` · 1 Methoden
    - `trait CommitBarrier` · Methoden: shared, exclusive
    - `trait PurgeBarrier` · Methoden: compaction, purge, purge_waiting
- `checkpoint`
  - 2 trait
    - `trait Checkpoint` · Methoden: take_snapshot, restore
    - `trait CheckpointCoordinator` · Methoden: create_named_checkpoint, restore_named_checkpoint, drop_named_checkpoint, list_named_checkpoints
- `clock`
  - 1 struct, 1 trait
    - `trait Clock` · Methoden: now_unix_nanos, monotonic_nanos
    - `struct SystemClock` · 1 Methoden
- `conformance`
  - 20 fn
    - `fn assert_checkpoint_contract`
    - `fn assert_checkpoint_coordinator_contract`
    - `fn assert_vector_index_contract`
    - `fn assert_attention_exporter_contract`
    - `fn assert_graph_index_contract`
    - `fn assert_lifecycle_contract`
    - `fn assert_embedding_contract`
    - `fn assert_observability_contract`
    - `fn assert_license_contract`
    - `fn assert_reranker_contract`
    - `fn assert_text_index_contract`
    - `fn assert_storage_engine_contract`
    - `fn assert_purger_contract`
    - `fn assert_vfs_file_contract`
    - `fn assert_read_gate_contract`
    - `fn assert_keys_contract`
    - `fn assert_ledger_contract`
    - `fn assert_erasure_layer_contract`
    - `fn assert_egress_policy_contract`
    - `fn assert_visitor_methods`
- `embedding`
  - 1 enum, 1 struct, 5 trait
    - `struct ContextSegment` · 3 Methoden
    - `trait TextEmbeddingEngine` · Methoden: embed, embed_batch
    - `enum EmbeddingError`
    - `trait EmbeddingProvider` · Methoden: provider_name, embed, embedding_dim, embed_batch
    - `trait TextGenerator` · Methoden: generate_text
    - `trait LlmTextGenerator` · Methoden: generate, generate_with_context
    - `trait LlmTextGeneratorStreaming` · Methoden: generate_stream
- `erasure`
  - 2 enum, 6 struct, 3 trait
    - `struct LayerId` · 2 Methoden
    - `enum EraseTarget`
    - `struct LayerScope` · 4 Methoden
    - `struct PurgeCx` · 1 Methoden
    - `struct PurgeOutcome` · 1 Methoden
    - `struct Armed`
    - `struct Scanner` · 1 Methoden
    - `enum KeyCoverage`
    - `trait ErasureLayer` · Methoden: id, resolve, on_gate, purge, witness, key_coverage
    - `trait StorageBackend`
    - `trait ErasureTarget`
- `gate`
  - 3 enum, 3 struct, 1 trait
    - `struct Seq` · 2 Methoden
    - `struct KeyBuf` · 2 Methoden
    - `enum KeyBound` · 1 Methoden
    - `struct KeyRange` · 2 Methoden
    - `enum GateDecision`
    - `enum RangeGate`
    - `trait ReadGate` · Methoden: check, check_bound, variant_for_range, gated_seq, gate_epoch, covers…
- `graph`
  - 1 enum, 3 struct, 1 trait
    - `struct RoleId` · 2 Methoden
    - `struct RoleBinding` · 1 Methoden
    - `struct HyperEdgeId` · 2 Methoden
    - `enum GraphMutationError`
    - `trait GraphCollectionMutation` · Methoden: relate_n_ary
- `graph_index`
  - 1 struct, 2 trait
    - `trait GraphIndex` · Methoden: traverse, neighbors, remove_edge, add_bidirectional, multi_traverse, multi_traverse_at…
    - `struct GraphIndexStats`
    - `trait CommunityResolver` · Methoden: get_community
- `id_gen`
  - 1 struct, 1 trait
    - `trait IdGen` · Methoden: next_id
    - `struct SequentialIdGen` · 1 Methoden
- `keys`
  - 1 enum, 4 struct, 2 trait
    - `struct FileId` · 2 Methoden
    - `enum FileKind`
    - `struct FileKey` · 3 Methoden
    - `struct SealCtx` · 1 Methoden
    - `struct Envelope` · 2 Methoden
    - `trait RecordSealer` · Methoden: seal, open, evict, make_slots_durable
    - `trait FileKeyring` · Methoden: create, open, destroy
- `kv`
  - 5 struct, 1 trait
    - `struct KvLayout`
    - `struct RopeConfig`
    - `struct PrefixKey`
    - `struct KvBlock`
- `kv_bridge_port`
  - 1 trait
- `ledger`
  - 3 struct, 1 trait
- `license`
  - 3 enum, 2 struct, 1 trait
- `lifecycle`
  - 1 struct, 3 trait
- `metrics`
  - 5 struct, 1 trait
- `observability`
  - 1 trait
- `observer`
  - 3 enum, 1 struct, 1 trait, 1 type
- `plugin`
  - 1 enum, 2 struct, 1 trait
- `policy`
  - 1 enum, 1 trait
- `query_rewriter`
  - 1 trait
- `report_signer`
  - 1 enum, 1 struct, 1 trait, 1 type
- `reranker`
  - 1 trait
- `rng`
  - 1 struct, 1 trait
- `scanner`
  - 2 enum, 4 struct, 2 trait
- `storage`
  - 1 const, 1 struct, 1 trait
- `text_index`
  - 1 struct, 2 trait
- `vector_index`
  - 1 struct, 2 trait
- `vfs`
  - 1 enum, 1 struct, 2 trait
  - … 65 weitere Items nicht aufgelistet (`--max-items`)
- Zusätzlich 10 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).
- Interne Re-Exporte (`pub use`): `LayerWitness`, `QueryRewriter`, `Reranker`, `KvPrefixStore`, `TextEmbeddingEngine`

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-checkpoint` | `Clock`×20, `StorageEngine`×14, `Vfs`×7, `MetricsSink`×6, `BoxFuture`×3, `SystemClock`×2, `Checkpoint`×2, `CheckpointCoordinator` | – |
| `contextra-store` | `Clock`×11, `MetricsSink`×6, `MAX_SCAN_MERGE_ACCUMULATOR`×6, `StorageStats`×5, `scanner`×5, `SystemClock::new`×4, `Vfs`×3, `IdGen`×3, `ResourceTracker`×2, `StorageEngine`×2 … (+7) | `ResourceBudget`×28, `ResourceTracker::new`×27, `clock::SystemClock`, `storage::StorageEngine` |
| `contextra-engine` | `MetricsSink`×9, `StorageEngine`×5, `Clock`×3, `VectorIndexStats`×2, `Reranker`×2, `DriftStatusProvider`×2, `VectorIndex`, `SystemClock::new`, `GraphIndex`, `license` … (+6) | `BoxFuture` |
| `contextra-vector` | `Clock`×15, `MetricsSink`×6, `Rng`×4, `VectorIndex`×2, `scanner`×2, `NoopMetricsSink`×2, `SeededRng::new`×2 | `SeededRng`×5, `vector_index::VectorIndex`, `clock::SystemClock` |
| `contextra-graph` | `BoxFuture`×5, `GraphIndex`×4, `StorageEngine`×2, `graph::HyperEdgeId::new`×2, `GraphIndex::personalized_page_rank_at`, `GraphIndex::rollback_to_tx`, `graph::RoleBinding`, `graph::HyperEdgeId`, `graph::GraphMutationError`, `graph::GraphMutationError::LockAcquisitionTimeout` … (+10) | `Result`×3, `graph_index::GraphIndex` |
| `contextra-testkit` | `Clock`×2, `Vfs`×2, `GraphIndex`, `MetricsSink`, `Rng`, `StorageEngine`, `TextIndex`, `VectorIndex`, `scanner::WitnessLayer`, `scanner::ScanTechnique` … (+2) | `TextEmbeddingEngine` |
| `contextra` | `license::FeatureRing`×2, `license::LicenseGate`×2, `IdGen`, `license`, `license::VectorDeleteMode`, `license::LicenseError` | `BoxFuture`×3, `license::AuthorizedRing`, `license::OpenFastGate`, `license::LicenseError::NotActivated` … (+1) |
| `contextra-text` | `StorageEngine`×6, `scanner`×2 | `BoxFuture`×50, `StorageStats`×35, `clock::SystemClock` |
| `contextra-crypto` | `scanner::WitnessLayer`, `scanner::LayerWitness`, `Clock`, `scanner::ScanTechnique::NotMeasured`, `ScanTechnique`, `scanner` | `Result`×24, `SystemClock`×6, `scanner::ScanTechnique`×4 |
| `contextra-mcp` | `plugin::PluginRegistry`×2, `EmbeddingProvider` | `EmbeddingError`×17, `BoxFuture`×12, `Clock`×2 |
| `contextra-mvcc` | `MetricsSink`×3 | – |
| `contextra-rank` | `Clock`×2 | – |
| `contextra-bench` | `VectorIndex` | `StorageEngine` |
| `contextra-db` | `QueryRewriter` | `GraphIndex`×3, `StorageEngine`×2, `EmbeddingError`, `VectorIndex` … (+2) |

**Nach außen ungenutzte öffentliche Items (Heuristik, 105 von 145)**: `RequestId`, `AttentionExporter`, `BarrierShared`, `BarrierExclusive`, `CompactionPermit`, `PurgePermit`, `CommitBarrier`, `PurgeBarrier`, `assert_checkpoint_contract`, `assert_checkpoint_coordinator_contract`, `assert_vector_index_contract`, `assert_attention_exporter_contract`, `assert_graph_index_contract`, `assert_lifecycle_contract`, `assert_embedding_contract`, `assert_observability_contract`, `assert_license_contract`, `assert_reranker_contract`, `assert_text_index_contract`, `assert_storage_engine_contract`, `assert_purger_contract`, `assert_vfs_file_contract`, `assert_read_gate_contract`, `assert_keys_contract`, `assert_ledger_contract`, `assert_erasure_layer_contract`, `assert_egress_policy_contract`, `assert_visitor_methods`, `ContextSegment`, `TextGenerator` …

### contextra-rank

- Pfad: `crates/contextra-rank` · Zeilen: 7660 · Rust-Dateien: 42 · Ring: 0
- Interne Cargo-Abhängigkeiten: `contextra-ports` (normal/optional via types-compat), `contextra-types` (normal/optional via types-compat)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-types` | 38 | 0 | 4 | `DocId`×4, `SignalFusionStrategies`×4, `ContextraError::invalid_input`×3, `FusionWeights`×3, `FusionStrategy`×3, `OnSignalFailure`×3, `ContextraError`×2, `ContextChunk`×2, `ConfigFingerprint`×2, `FusionStrategy::Rrf`×2 … (+7) | – |
| `contextra-ports` | 2 | 0 | 1 | `Clock`×2 | – |

**Pro Komponente**

- `calibration` (843 Zeilen)
  - → `contextra-ports`: prod: `Clock`×2
- `error` (28 Zeilen)
  - → `contextra-types`: prod: `ContextraError::invalid_input`×3, `ContextraError`
- `fusion` (2133 Zeilen)
  - → `contextra-types`: prod: `ContextChunk`×2, `FusionWeights`, `ContextraError`, `DocId::from_key`, `ContextraError::InvalidInput`, `MemoryLink`, `DocId`
- `local_types` (242 Zeilen)
  - → `contextra-types`: prod: `SignalFusionStrategies`×4, `DocId`×3, `FusionStrategy`×3, `OnSignalFailure`×3, `ConfigFingerprint`×2, `FusionStrategy::Rrf`×2, `FusionStrategy::ScoreNormalized`×2, `OnSignalFailure::Fail`×2, `OnSignalFailure::Degrade`×2, `FusionWeights`×2 … (+1)

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `calibration`
  - 2 enum, 1 fn, 4 struct, 2 trait
    - `enum ConformalError`
    - `trait ConformalCalibrator` · Methoden: update, threshold
    - `struct AdaptiveConformalCalibrator` · 7 Methoden
    - `trait Clock` · Methoden: now_unix_nanos, monotonic_nanos ⚠ fremd: `contextra-ports`
    - `struct SystemClock` · 1 Methoden
    - `struct IsotonicCalibrator` · 13 Methoden
    - `enum Calibrator` · 3 Methoden
    - `fn calibrate_score`
    - `struct PlattScaler` · 10 Methoden
- `dibud`
  - 2 enum, 2 fn, 3 struct
    - `fn fuse_exact_prefix`
    - `fn fuse_exact_prefix_async`
    - `enum DiBudStep`
    - `struct DiBudFusionState` · 7 Methoden
    - `enum BudgetedChannel` · 2 Methoden
    - `struct FusionBudget` · 1 Methoden
    - `struct DiBudOutcome`
- `drift`
  - 1 enum, 1 struct
    - `enum DriftStatus` · 1 Methoden
    - `struct DriftDetector` · 6 Methoden
- `error`
  - 1 enum, 1 type
    - `enum RankError`
    - `type Result`
- `explain`
  - 1 fn, 2 struct
    - `struct ExplanationEntry`
    - `struct RetrievalExplanation` · 1 Methoden
    - `fn explain`
- `fusion`
  - 1 enum, 1 fn, 2 struct
    - `fn weights_to_signal_factors_types_compat` *(re-exportiert)* ⚠ fremd: `contextra-types`
    - `struct ProvenanceBuilder` *(re-exportiert)* · 10 Methoden
    - `enum SignalKind` *(re-exportiert)* · 3 Methoden
    - `struct BoundedTopK` *(re-exportiert)* · 6 Methoden
- `local_types`
  - 1 const, 2 enum, 4 struct
    - `const MAX_SEARCH_K`
    - `struct DocId` · 1 Methoden ⚠ fremd: `contextra-types`
    - `struct ConfigFingerprint` · 3 Methoden ⚠ fremd: `contextra-types`
    - `enum FusionStrategy` ⚠ fremd: `contextra-types`
    - `enum SignalFailurePolicy`
    - `struct SignalFusionStrategies` · 1 Methoden ⚠ fremd: `contextra-types`
    - `struct FusionWeights` ⚠ fremd: `contextra-types`
- `metrics`
  - 5 fn
    - `fn reciprocal_rank`
    - `fn mean_reciprocal_rank`
    - `fn precision_at_k`
    - `fn recall_at_k`
    - `fn ndcg_at_k`
- Zusätzlich 27 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).
- Interne Re-Exporte (`pub use`): `SignalKind`, `FusionStrategy`, `SignalFailurePolicy`

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-engine` | `SearchResult`×8, `IsotonicCalibrator`×2, `fusion`, `fusion::fuse_search_results_with_strategy`, `fusion::fuse_search_results_with_signal_strategies`, `fusion::fuse_signals` | – |
| `contextra-bench` | `RankError` | – |
| `contextra-mcp` | `fusion::ProvenanceRecord` | – |
| `contextra-db` | – | `IsotonicCalibrator::new` |

**Nach außen ungenutzte öffentliche Items (Heuristik, 37 von 39)**: `ConformalError`, `ConformalCalibrator`, `AdaptiveConformalCalibrator`, `Clock`, `SystemClock`, `Calibrator`, `calibrate_score`, `PlattScaler`, `fuse_exact_prefix`, `fuse_exact_prefix_async`, `DiBudStep`, `DiBudFusionState`, `BudgetedChannel`, `FusionBudget`, `DiBudOutcome`, `DriftStatus`, `DriftDetector`, `Result`, `ExplanationEntry`, `RetrievalExplanation`, `explain`, `weights_to_signal_factors_types_compat`, `ProvenanceBuilder`, `SignalKind`, `BoundedTopK`, `MAX_SEARCH_K`, `DocId`, `ConfigFingerprint`, `FusionStrategy`, `SignalFailurePolicy` …

### contextra-simd

- Pfad: `crates/contextra-simd` · Zeilen: 5137 · Rust-Dateien: 17 · Ring: 0
- Interne Cargo-Abhängigkeiten: `contextra-types` (normal/optional via types-compat)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-types` | 13 | 0 | 2 | `ContextraError::invalid_input`×3, `DistanceMetric`×3, `DistanceMetric::Cosine`×2, `DistanceMetric::Euclidean`×2, `DistanceMetric::DotProduct`×2, `ContextraError` | – |

**Pro Komponente**

- `error` (32 Zeilen)
  - → `contextra-types`: prod: `ContextraError::invalid_input`×3, `ContextraError`
- `metric` (29 Zeilen)
  - → `contextra-types`: prod: `DistanceMetric`×3, `DistanceMetric::Cosine`×2, `DistanceMetric::Euclidean`×2, `DistanceMetric::DotProduct`×2

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `(crate-root)`
  - 4 fn
    - `fn validate_vector`
    - `fn compute_distance`
    - `fn compute_distance_trusted`
    - `fn compute_distance_f32_bytes_trusted`
- `dispatch`
  - 1 enum, 10 fn, 1 struct
    - `enum SimdTarget`
    - `struct Dispatcher` · 13 Methoden
    - `fn detect_best_target`
    - `fn cosine_distance`
    - `fn euclidean_distance`
    - `fn dot_product_distance`
    - `fn cosine_distance_f32_bytes`
    - `fn euclidean_distance_f32_bytes`
    - `fn dot_product_distance_f32_bytes`
    - `fn dot_product_u8`
    - `fn euclidean_distance_sq_u8`
    - `fn cosine_similarity_parts_u8`
- `error`
  - 1 enum, 2 type
    - `enum SimdError`
    - `type Result`
    - `type Error`
- `kernels`
  - 13 fn, 2 struct
    - `struct CosineSimilarityPartsU8`
    - `struct CosineSimilarityPartsF32U8`
    - `fn cosine_distance_scalar`
    - `fn euclidean_distance_scalar`
    - `fn dot_product_scalar`
    - `fn cosine_distance_f32_bytes_scalar`
    - `fn euclidean_distance_f32_bytes_scalar`
    - `fn dot_product_f32_bytes_scalar`
    - `fn dot_product_u8_scalar`
    - `fn euclidean_distance_sq_u8_scalar`
    - `fn cosine_similarity_parts_u8_scalar`
    - `fn dot_product_f32_u8`
    - `fn euclidean_distance_sq_f32_u8`
    - `fn cosine_similarity_parts_f32_u8`
    - `fn normalize_inplace`
- `metric`
  - 1 enum
    - `enum DistanceMetric` ⚠ fremd: `contextra-types`

**Eingehend (wer nutzt dieses Crate)**

Wird von keinem anderen analysierten Workspace-Crate importiert.

**Nach außen ungenutzte öffentliche Items (Heuristik, 35 von 35)**: `SimdTarget`, `Dispatcher`, `detect_best_target`, `cosine_distance`, `euclidean_distance`, `dot_product_distance`, `cosine_distance_f32_bytes`, `euclidean_distance_f32_bytes`, `dot_product_distance_f32_bytes`, `dot_product_u8`, `euclidean_distance_sq_u8`, `cosine_similarity_parts_u8`, `SimdError`, `Result`, `Error`, `CosineSimilarityPartsU8`, `CosineSimilarityPartsF32U8`, `cosine_distance_scalar`, `euclidean_distance_scalar`, `dot_product_scalar`, `cosine_distance_f32_bytes_scalar`, `euclidean_distance_f32_bytes_scalar`, `dot_product_f32_bytes_scalar`, `dot_product_u8_scalar`, `euclidean_distance_sq_u8_scalar`, `cosine_similarity_parts_u8_scalar`, `dot_product_f32_u8`, `euclidean_distance_sq_f32_u8`, `cosine_similarity_parts_f32_u8`, `normalize_inplace` …

### contextra-store

- Pfad: `crates/contextra-store` · Zeilen: 48169 · Rust-Dateien: 234 · Ring: 1
- Interne Cargo-Abhängigkeiten: `contextra-testkit` (dev), `contextra-crypto` (normal), `contextra-mvcc` (normal), `contextra-ports` (normal), `contextra-sys` (normal), `contextra-types` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-ports` | 54 | 149 | 20 | `Clock`×11, `MetricsSink`×6, `MAX_SCAN_MERGE_ACCUMULATOR`×6, `StorageStats`×5, `scanner`×5, `SystemClock::new`×4, `Vfs`×3, `IdGen`×3, `ResourceTracker`×2, `StorageEngine`×2 … (+7) | `ResourceBudget`×28, `ResourceTracker::new`×27, `clock::SystemClock`, `storage::StorageEngine` |
| `contextra-types` | 39 | 123 | 15 | `ContextraError::Storage`×8, `Result`×5, `ContextraError`×5, `TOMBSTONE_BIT`×4, `TxId`×4, `ContextraError::MemoryBudgetExceeded`×2, `ContextraError::Internal`×2, `TxId::new`×2, `TxId::INTERNAL_BASE`×2, `ContextraError::Sandbox` … (+4) | `CollectionId`×6, `DocId`×6, `ContextraError::InvalidInput`×2, `TenantId`, `*` |
| `contextra-mvcc` | 10 | 7 | 6 | `SsiValidator`×4, `tx_buffer::STAGING_ENTRY_OVERHEAD_BYTES`×2, `SnapshotRegistry`, `SequenceLogSsiValidator`, `IndexOp`, `SequenceLogSsiValidator::new_with_bounds` | `SnapshotRegistry::new` |
| `contextra-crypto` | 5 | 2 | 5 | `wal_crypto`×2, `verify_wal_chain_completeness`, `wal_crypto::WalHmac`, `crypto::KeyManager` | – |
| `contextra-sys` | 3 | 4 | 2 | `mmap_readonly`×2, `set_restrictive_file_acl` | `reopen_and_dup2`×2, `verify_file_acl_owner_only`, `set_fd_limit` |
| `contextra-testkit` | 0 | 8 | 0 | – | `ManualClock`×4, `ReferenceModel`×4 |

**Pro Komponente**

- `compaction` (3432 Zeilen)
  - → `contextra-mvcc`: prod: `SnapshotRegistry` · dev: `SnapshotRegistry`
  - → `contextra-ports`: prod: `StorageStats`×2, `ResourceTracker`×2, `Clock`×2, `Vfs`, `SystemClock::new`, `ContextraError::NotFound` · dev: `ResourceTracker::new`×27, `ResourceBudget`×27, `StorageEngine`×2
  - → `contextra-types`: prod: `ContextraError::MemoryBudgetExceeded`×2, `ContextraError::Internal`×2, `ContextraError::Sandbox`, `Result`, `TOMBSTONE_BIT` · dev: `TxId::new`×3, `TOMBSTONE_BIT`×2, `ContextraError::MemoryBudgetExceeded`×2, `TxId`
- `kv` (48 Zeilen)
  - → `contextra-types`: prod: `Result`
- `lsm` (6598 Zeilen)
  - → `contextra-crypto`: prod: `verify_wal_chain_completeness`
  - → `contextra-mvcc`: prod: `SsiValidator`×4, `tx_buffer::STAGING_ENTRY_OVERHEAD_BYTES`, `SequenceLogSsiValidator`, `IndexOp`, `SequenceLogSsiValidator::new_with_bounds` · dev: `tx_buffer::STAGING_ENTRY_OVERHEAD_BYTES`
  - → `contextra-ports`: prod: `Clock`×6, `MAX_SCAN_MERGE_ACCUMULATOR`×6, `MetricsSink`×5, `Vfs`×2, `StorageStats`×2, `StorageEngine`×2, `license::FeatureRing`, `license::FeatureRing::Sovereign`, `SystemClock::new`, `NoopMetricsSink` · dev: `Clock`
  - → `contextra-testkit`: dev: `ManualClock`
  - → `contextra-types`: prod: `ContextraError::Storage`×5, `TxId`×4, `ContextraError`, `ContextraError::DurabilityConfig`, `Result`, `TxId::new`, `ContextraError::NotFound`, `TxId::INTERNAL_BASE` · dev: `TOMBSTONE_BIT`
- `memtable` (969 Zeilen)
  - → `contextra-mvcc`: prod: `tx_buffer::STAGING_ENTRY_OVERHEAD_BYTES`
  - → `contextra-types`: dev: `TOMBSTONE_BIT`
- `scanner` (1028 Zeilen)
  - → `contextra-ports`: prod: `scanner`×5, `scanner::ScanTarget::new`, `scanner::ScanTechnique::Structural`, `SystemClock::new` · dev: `clock::SystemClock`, `StorageEngine`, `scanner`
  - → `contextra-types`: prod: `ContextraError`×4, `ContextraError::Storage`×3, `Result`×2, `RequestContext::new`, `TenantId::new`, `TxId::new` · dev: `CollectionId`×6, `DocId`×6, `ContextraError`
- `sstable` (3538 Zeilen)
  - → `contextra-ports`: prod: `VfsFile`
  - → `contextra-types`: prod: `TOMBSTONE_BIT`×3, `TxId::INTERNAL_BASE` · dev: `ContextraError`
- `system_pressure` (345 Zeilen)
  - → `contextra-ports`: prod: `Clock`×3, `SystemClock::new`, `MetricsSink`
- `tenant_codec` (837 Zeilen)
  - → `contextra-ports`: prod: `StorageStats`
- `wal` (7297 Zeilen)
  - → `contextra-crypto`: prod: `wal_crypto`×2, `wal_crypto::WalHmac`, `crypto::KeyManager` · dev: `wal_crypto::WalHmac`, `crypto::KeyManager`
  - → `contextra-ports`: prod: `IdGen`×3
  - → `contextra-sys`: prod: `mmap_readonly`×2, `set_restrictive_file_acl` · dev: `verify_file_acl_owner_only`
  - → `contextra-types`: dev: `TxId`×6
- `benches/lsm_concurrency_bench.rs` (129 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `benches/redb_comparative_bench.rs` (508 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `benches/wal_bench.rs` (143 Zeilen)
  - → `contextra-types`: dev: `TxId`
- `examples/chaos_writer.rs` (55 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `examples/store_standalone.rs` (36 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `fuzz/fuzz_targets/fuzz_compaction_interleave.rs` (110 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `fuzz/fuzz_targets/fuzz_recovery_arbitrary_state.rs` (52 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `fuzz/fuzz_targets/wal_mutation_chaos.rs` (270 Zeilen)
  - → `contextra-types`: dev: `TxId`
- `tests/adaptive_compaction_determinism.rs` (35 Zeilen)
  - → `contextra-ports`: dev: `StorageStats`
- `tests/adaptive_compaction_regression.rs` (35 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/adaptive_compaction_tombstone.rs` (40 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/amplification_benchmark.rs` (343 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/blocking_util_pool_measurement.rs` (90 Zeilen)
  - → `contextra-ports`: dev: `MetricsSink`
- `tests/budget_drift.rs` (54 Zeilen)
  - → `contextra-mvcc`: dev: `tx_buffer::STAGING_ENTRY_OVERHEAD_BYTES`
- `tests/campaign_fuzzregress_memtable_concurrent.rs` (105 Zeilen)
  - → `contextra-types`: dev: `TxId`
- `tests/campaign_j11_memory_deletion.rs` (48 Zeilen)
  - → `contextra-ports`: dev: `storage::StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/campaign_j21_hypotheses.rs` (433 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/campaign_j41_store.rs` (36 Zeilen)
  - → `contextra-types`: dev: `TenantId`
- `tests/campaign_prop_lsm_ops.rs` (407 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/campaign_stress_store.rs` (294 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/chaos_bitflip_sstable.rs` (405 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/chaos_dropped_write.rs` (146 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-sys`: dev: `reopen_and_dup2`×2
- `tests/chaos_matrix.rs` (278 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/chaos_memory_pressure.rs` (217 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/chaos_power_cut.rs` (159 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `ContextraError`
- `tests/chaos_task_massacre.rs` (163 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/commit_cancel_chain_consistency.rs` (49 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/commit_cancellation_atomicity.rs` (147 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/commit_core_batch_rollback.rs` (44 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/commit_core_cancel_safety.rs` (54 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/commit_paths_equivalence.rs` (95 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/commit_read_lock_concurrency.rs` (61 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/commit_rollback_batch_scoped.rs` (97 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/compaction_contiguous_selection.rs` (155 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`, `StorageStats`
  - → `contextra-types`: dev: `TxId`
- `tests/compaction_correctness_and_pinning.rs` (80 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/compaction_key_range_selection.rs` (101 Zeilen)
  - → `contextra-mvcc`: dev: `SnapshotRegistry::new`
  - → `contextra-ports`: dev: `ResourceTracker`, `ResourceBudget`
- `tests/compaction_merge_operator.rs` (482 Zeilen)
  - → `contextra-mvcc`: dev: `SnapshotRegistry`
  - → `contextra-testkit`: dev: `ManualClock`
- `tests/compaction_noncontiguous_shadowing.rs` (78 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/compaction_overlap_scan_and_tombstone.rs` (216 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/crash_kill_durability.rs` (441 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/crash_prefix_enumeration.rs` (133 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-testkit`: dev: `ReferenceModel`
  - → `contextra-types`: dev: `TxId`
- `tests/crash_recovery.rs` (111 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/determinism_injection_test.rs` (121 Zeilen)
  - → `contextra-ports`: dev: `scanner`
  - → `contextra-types`: dev: `TxId`
- `tests/differential/redb_operation_sequence.rs` (853 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/disk_full_during_write.rs` (76 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/docid_128_kv_engine.rs` (64 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/durability_mode_feature_gate.rs` (120 Zeilen)
  - → `contextra-ports`: dev: `license::FeatureRing`
- `tests/encryption_test.rs` (106 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/fault_injection_recovery.rs` (120 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/filename_uniqueness_test.rs` (88 Zeilen)
  - → `contextra-mvcc`: dev: `SnapshotRegistry`
  - → `contextra-types`: dev: `TxId`
- `tests/flush_crash_simulation.rs` (179 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/flush_durability.rs` (64 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/flush_mvcc_versions.rs` (184 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/flush_recovery_lifecycle.rs` (187 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/flush_threshold_boundary.rs` (68 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/fsync_syscall_verification.rs` (62 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/fsync_timeout_rollback.rs` (67 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/group_commit_fault_injection.rs` (97 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/group_commit_flush_race.rs` (171 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/group_commit_follower_timeout_chain.rs` (69 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/group_commit_leader_cancel_chain.rs` (65 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/group_commit_overlapping_batch_rollback.rs` (73 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/group_commit_stress.rs` (129 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/group_commit_test.rs` (223 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `ContextraError`
- `tests/group_commit_timeout.rs` (171 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/lsm_robustness_fixes.rs` (139 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/lsm_storage_merge_operator.rs` (112 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `ContextraError::InvalidInput`×2
- `tests/manifest_compaction_audit.rs` (117 Zeilen)
  - → `contextra-mvcc`: dev: `SnapshotRegistry`
  - → `contextra-types`: dev: `TOMBSTONE_BIT`
- `tests/manifest_crash_no_resurrection.rs` (206 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/manifest_replace_rank_compat.rs` (68 Zeilen)
  - → `contextra-types`: dev: `Result`
- `tests/memory_only_crash_semantics.rs` (50 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/metrics_sink_test.rs` (278 Zeilen)
  - → `contextra-ports`: dev: `MetricsSink`, `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/model_based_mvcc_flush.rs` (185 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-testkit`: dev: `ReferenceModel`
  - → `contextra-types`: dev: `TxId`
- `tests/observer_blocking_failopen.rs` (169 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/observer_deadline_no_runtime_block.rs` (67 Zeilen)
  - → `contextra-types`: dev: `TxId`
- `tests/proptest_model_based.rs` (237 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`, `Result`
- `tests/read_path_audit.rs` (210 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/recovery_fd_exhaustion_regression.rs` (93 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-sys`: dev: `set_fd_limit`
  - → `contextra-types`: dev: `TxId`
- `tests/resource_leaks.rs` (88 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/rollback_sstables.rs` (62 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/scan_limits_and_ranges.rs` (59 Zeilen)
  - → `contextra-types`: dev: `TxId`
- `tests/spec_invariants_agents_sec4.rs` (237 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/ssi_tracked_read_seq_clamp.rs` (122 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/ssi_tracked_reads_via_trait.rs` (120 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/ssi_unbounded_snapshot_seq.rs` (163 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/ssi_write_skew_integration.rs` (274 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/sstable_format_v3_to_v4_compat.rs` (158 Zeilen)
  - → `contextra-types`: dev: `TOMBSTONE_BIT`
- `tests/support/mod.rs` (79 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-testkit`: dev: `ReferenceModel`
- `tests/tenant_scoped_storage.rs` (131 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/test_wp02_flush_mvcc_bug.rs` (39 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-testkit`: dev: `ReferenceModel`
  - → `contextra-types`: dev: `TxId`, `Result`
- `tests/toctou_put_if_absent.rs` (209 Zeilen)
  - → `contextra-types`: dev: `ContextraError`, `Result`, `*`
- `tests/tombstone_scan.rs` (120 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/wal_backpressure.rs` (294 Zeilen)
  - → `contextra-types`: dev: `TxId`
- `tests/wal_flusher_rollback.rs` (102 Zeilen)
  - → `contextra-types`: dev: `Result`×2, `TxId`
- `tests/wal_hmac_chain_ordering.rs` (178 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/wal_hmac_rollback_race.rs` (122 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/wal_legacy_key_forced_migration.rs` (228 Zeilen)
  - → `contextra-types`: dev: `TxId`
- `tests/wal_legacy_key_migration_store.rs` (94 Zeilen)
  - → `contextra-types`: dev: `TxId`
- `tests/wal_legacy_key_retirement_marker.rs` (109 Zeilen)
  - → `contextra-types`: dev: `TxId`
- `tests/wal_legacy_migration_crash.rs` (44 Zeilen)
  - → `contextra-types`: dev: `TxId`
- `tests/wal_no_hmac_replay.rs` (52 Zeilen)
  - → `contextra-types`: dev: `TxId`
- `tests/wal_observer.rs` (178 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/wal_observer_contract.rs` (150 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-testkit`: dev: `ManualClock`
  - → `contextra-types`: dev: `TxId`
- `tests/wal_observer_integration.rs` (269 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-testkit`: dev: `ManualClock`
  - → `contextra-types`: dev: `TxId`
- `tests/wal_observer_test.rs` (181 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/wal_poison_causes.rs` (169 Zeilen)
  - → `contextra-types`: dev: `Result`×5, `TxId`
- `tests/wal_recovery_runtime_stall.rs` (116 Zeilen)
  - → `contextra-types`: dev: `TxId`
- `tests/wal_systematic_crash.rs` (199 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `ContextraError::Storage`×2, `TxId`
- `tests/wal_torn_write_fault_injection.rs` (113 Zeilen)
  - → `contextra-types`: dev: `TxId`
- `tests/wal_truncate_ordering.rs` (257 Zeilen)
  - → `contextra-types`: dev: `ContextraError`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `compaction`
  - 1 enum, 1 fn, 7 struct, 2 trait
    - `struct WorkloadMetrics` · 4 Methoden
    - `struct WorkloadMetricsSnapshot`
    - `enum CompactionStrategy`
    - `struct AdaptiveCompactionPlan`
    - `trait AdaptiveCompactionPlanner` · Methoden: plan_compaction
    - `struct CostBasedAdaptivePlanner` · 2 Methoden
    - `struct CompactionConfig` *(re-exportiert)*
    - `struct CompactionEngine` *(re-exportiert)* · 17 Methoden
    - `trait MergeOperator` · Methoden: merge
    - `fn retain_key_versions`
    - `struct TtlMetadata`
- `engine`
  - 1 struct
    - `struct InMemoryStorageEngine` · 1 Methoden
- `fsync`
  - 3 fn
    - `fn sync_file`
    - `fn sync_parent_dir`
    - `fn fsync_parent_dir`
- `io_seam`
  - 3 struct
    - `struct RealFs`
    - `struct RealFsFile` · 2 Methoden
    - `struct StoreIo` · 13 Methoden
- `key_source`
  - 2 struct, 1 trait
    - `trait KeyMaterialSource` · Methoden: fill_key_bytes
    - `struct OsKeyMaterialSource`
    - `struct StaticKeyMaterialSource` · 1 Methoden
- `lsm`
  - 17 const, 4 enum, 2 fn, 8 struct, 1 trait
    - `enum DurabilityMode` · 2 Methoden
    - `enum DurabilityConfigError`
    - `struct LsmConfig`
    - `enum StorageHealth` *(re-exportiert)*
    - `struct LsmStorage` *(re-exportiert)* · 33 Methoden ⚠ fremd: `contextra-mvcc`
    - `struct FlushPurgeFence` · 7 Methoden
    - `fn publish_visibility`
    - `fn load_snapshot_tx`
    - `const RANK_DOCUMENT_INTENT`
    - `const RANK_COMMIT_MUTEX`
    - `const RANK_FLUSH_MUTEX`
    - `const RANK_STATE_RWLOCK`
    - `const RANK_WAL_TRUNCATE`
    - `const RANK_SSTABLES_RWLOCK`
    - `const RANK_MEMTABLE_SHARDS`
    - `const RANK_MANIFEST_MUTEX`
    - `const RANK_OBSERVER_REGISTRY`
    - `const RANK_BLOCK_CACHE`
    - `const MAX_KEY_SIZE`
    - `const MAX_VALUE_SIZE`
    - `const MAX_BATCH_SIZE`
    - `const MAX_INTERNAL_MERGE_ENTRIES_FACTOR`
    - `const MAX_GROUP_COMMIT_BATCH_SIZE`
    - `const MIN_ENTRIES_FOR_SSTABLE_REBUILD`
    - `const DEFAULT_MAX_OBSERVER_LATENCY`
    - `enum WriteOrigin`
    - `struct WalEntryRef` · 6 Methoden
    - `struct CommitContext` ⚠ fremd: `contextra-types`
    - `struct CommittedBatch`
    - `trait WalObserver` · Methoden: requires_durability, on_commit
    - `struct ObserverRegistry` · 14 Methoden
    - `struct AsyncObserverAdapter` · 1 Methoden
- `manifest`
  - 2 const, 1 enum, 1 struct
    - `struct Manifest` *(re-exportiert)* · 9 Methoden
    - `enum ManifestEntry` *(re-exportiert)* · 2 Methoden
    - `const MAX_MANIFEST_ENTRY_SIZE`
    - `const DEFAULT_ROLLOVER_THRESHOLD_BYTES`
- `memtable`
  - 1 const, 2 fn, 1 struct
    - `fn is_tombstone`
    - `fn is_valid_range_bounds`
    - `const RESERVED_PREFIXES`
    - `struct MemTable` · 16 Methoden
- `scanner`
  - 6 fn, 7 struct
    - `struct MemtableScanner` · 2 Methoden
    - `struct ResidualInspectionReport` · 1 Methoden
    - `struct StoreInspector` · 4 Methoden
    - `fn lsm_scanners`
    - `fn lsm_scanners_with_handle`
    - `fn lsm_scanners_with_runtime`
    - `fn lsm_purgers`
    - `fn lsm_purgers_with_handle`
    - `fn lsm_purgers_with_runtime`
    - `struct LsmPurger` · 2 Methoden
    - `struct ScannerRuntime` · 5 Methoden
    - `struct SstableScanner` · 2 Methoden
    - `struct WalScanner` · 2 Methoden
- `sstable`
  - 3 const, 9 fn, 10 struct, 1 trait, 2 type
    - `const BLOCK_CACHE_SHARDS` *(re-exportiert)*
    - `struct SieveCacheBackend` *(re-exportiert)*
    - `trait BlockCacheBackend` *(re-exportiert)* · Methoden: new, get, insert, len, is_empty, contains…
    - `struct LruBlockCacheBackend` *(re-exportiert)*
    - `struct QuickCacheBlockCacheBackend` *(re-exportiert)*
    - `type BlockCacheShard` *(re-exportiert)*
- `system_pressure`
  - 6 const, 1 enum, 2 struct, 1 trait
- `tenant_codec`
  - 2 struct, 1 trait
- `util`
  - 1 struct, 2 trait
- `wal`
  - 5 const, 5 enum, 2 fn, 7 static, 6 struct, 1 trait, 1 type
  - … 62 weitere Items nicht aufgelistet (`--max-items`)
- Zusätzlich 29 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).
- ⚠ Re-Exporte **fremder Crates**: `contextra_crypto::crypto::KeyManager`

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-engine` | `LsmStorage`×6, `SystemPressure`×3, `tenant_codec::TenantScopedStorage`×3, `tenant_codec::TenantScopedStorage::new`×3, `PressureLevel::Critical`×2, `PressureLevel::Normal`, `PressureLevel::Elevated`, `LsmConfig`, `lsm::DurabilityMode` | `lsm`×10 |
| `contextra-db` | – | `LsmConfig`×3, `LsmStorage::new`×3, `lsm`, `LsmStorage` |

**Nach außen ungenutzte öffentliche Items (Heuristik, 137 von 142)**: `WorkloadMetrics`, `WorkloadMetricsSnapshot`, `CompactionStrategy`, `AdaptiveCompactionPlan`, `AdaptiveCompactionPlanner`, `CostBasedAdaptivePlanner`, `CompactionConfig`, `CompactionEngine`, `MergeOperator`, `retain_key_versions`, `TtlMetadata`, `InMemoryStorageEngine`, `sync_file`, `sync_parent_dir`, `fsync_parent_dir`, `RealFs`, `RealFsFile`, `StoreIo`, `KeyMaterialSource`, `OsKeyMaterialSource`, `StaticKeyMaterialSource`, `DurabilityConfigError`, `StorageHealth`, `FlushPurgeFence`, `publish_visibility`, `load_snapshot_tx`, `RANK_DOCUMENT_INTENT`, `RANK_COMMIT_MUTEX`, `RANK_FLUSH_MUTEX`, `RANK_STATE_RWLOCK` …

### contextra-sys

- Pfad: `crates/contextra-sys` · Zeilen: 2321 · Rust-Dateien: 11 · Ring: 0
- Interne Cargo-Abhängigkeiten: keine

**Ausgehend (Importe aus anderen Workspace-Crates)**

Keine.

**Pro Komponente**

Keine Importe aus anderen Workspace-Crates.

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `acl_win32`
  - 6 fn
    - `fn set_restrictive_file_acl`
    - `fn verify_file_acl_owner_only`
    - `fn set_restrictive_file_acl`
    - `fn verify_file_acl_owner_only`
    - `fn set_restrictive_file_acl`
    - `fn verify_file_acl_owner_only`
- `durability`
  - 3 fn, 1 struct
    - `fn durable_flush_file`
    - `fn durable_flush_dir`
    - `fn write_all_durable`
    - `struct DurableFile` · 6 Methoden
- `error`
  - 1 enum, 1 type
    - `enum Error`
    - `type Result`
- `mlock`
  - 1 enum, 5 fn
    - `enum MemLockPolicy`
    - `fn apply_dont_dump`
    - `fn mem_lock_typed`
    - `fn mem_lock_with_policy`
    - `fn mem_unlock`
    - `fn mem_lock`
- `mmap`
  - 1 fn, 4 struct
    - `fn mmap_readonly`
    - `struct SealedReadOnly`
    - `struct Unsealed`
    - `struct SealedFile` · 4 Methoden
    - `struct SealedMap` · 5 Methoden
- `posix`
  - 4 fn
    - `fn reopen_and_dup2`
    - `fn reopen_and_dup2`
    - `fn ensure_fd_limit`
    - `fn set_fd_limit`
- `vault`
  - 2 struct
    - `struct LockedRegions` · 7 Methoden
    - `struct VaultBuffer` · 8 Methoden

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-store` | `mmap_readonly`×2, `set_restrictive_file_acl` | `reopen_and_dup2`×2, `verify_file_acl_owner_only`, `set_fd_limit` |
| `contextra-vector` | `mmap_readonly`×2 | – |
| `contextra-bench` | – | `ensure_fd_limit` |

**Nach außen ungenutzte öffentliche Items (Heuristik, 18 von 29)**: `durable_flush_file`, `durable_flush_dir`, `write_all_durable`, `DurableFile`, `Error`, `Result`, `MemLockPolicy`, `apply_dont_dump`, `mem_lock_typed`, `mem_lock_with_policy`, `mem_unlock`, `mem_lock`, `SealedReadOnly`, `Unsealed`, `SealedFile`, `SealedMap`, `LockedRegions`, `VaultBuffer`

### contextra-testkit

- Pfad: `crates/contextra-testkit` · Zeilen: 4856 · Rust-Dateien: 24 · Ring: tooling
- Interne Cargo-Abhängigkeiten: `contextra-ports` (normal), `contextra-types` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-types` | 20 | 10 | 2 | `ContextraError::Io`×9, `Result`×8, `ContextraError::NotFound`×2, `TxId` | – |
| `contextra-ports` | 14 | 4 | 11 | `Clock`×2, `Vfs`×2, `GraphIndex`, `MetricsSink`, `Rng`, `StorageEngine`, `TextIndex`, `VectorIndex`, `scanner::WitnessLayer`, `scanner::ScanTechnique` … (+2) | `TextEmbeddingEngine` |

**Pro Komponente**

- `(crate-root)` (124 Zeilen)
  - → `contextra-ports`: prod: `scanner::WitnessLayer`, `scanner::ScanTechnique`, `scanner::LayerWitness`, `scanner::mint_witness_for_testkit`
- `conformance` (1486 Zeilen)
  - → `contextra-ports`: prod: `Clock`, `GraphIndex`, `MetricsSink`, `Rng`, `StorageEngine`, `TextIndex`, `VectorIndex`, `Vfs`
  - → `contextra-types`: prod: `TxId`
- `fault_vfs` (410 Zeilen)
  - → `contextra-ports`: prod: `Vfs` · dev: `Vfs`×2
  - → `contextra-types`: prod: `ContextraError::Io`×9, `Result`×8, `ContextraError::NotFound`×2 · dev: `Result`
- `manual_clock` (150 Zeilen)
  - → `contextra-ports`: prod: `Clock`
- `mocks` (276 Zeilen)
  - → `contextra-ports`: dev: `TextEmbeddingEngine`
- `benches/testkit_bench.rs` (28 Zeilen)
  - → `contextra-ports`: dev: `Clock`
- `examples/standalone_testkit.rs` (49 Zeilen)
  - → `contextra-types`: dev: `TxId`
- `tests/conformance_selftest.rs` (476 Zeilen)
  - → `contextra-types`: dev: `Result`×8

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `(crate-root)`
  - 1 fn
    - `fn mint_witness_for_tests` ⚠ fremd: `contextra-ports`
- `conformance`
  - 8 fn, 1 struct, 1 type
    - `fn run_clock_suite` ⚠ fremd: `contextra-ports`
    - `struct ConformanceFailure` · 1 Methoden
    - `type ConformanceResult`
    - `fn run_graph_index_suite` ⚠ fremd: `contextra-ports`
    - `fn run_metrics_sink_suite` ⚠ fremd: `contextra-ports`
    - `fn run_rng_suite` ⚠ fremd: `contextra-ports`
    - `fn run_storage_engine_suite` ⚠ fremd: `contextra-ports`
    - `fn run_text_index_suite` ⚠ fremd: `contextra-ports`
    - `fn run_vector_index_suite` ⚠ fremd: `contextra-ports`
    - `fn run_vfs_suite` ⚠ fremd: `contextra-ports`
- `fault_vfs`
  - 2 struct
    - `struct FaultConfig`
    - `struct FaultVfs` · 10 Methoden
- `in_memory_graph`
  - 1 struct
    - `struct InMemoryGraphIndex` · 1 Methoden
- `in_memory_store`
  - 1 struct
    - `struct InMemoryStorageEngine` · 1 Methoden
- `in_memory_text`
  - 1 struct
    - `struct InMemoryTextIndex` · 1 Methoden
- `in_memory_vector`
  - 1 struct
    - `struct InMemoryVectorIndex` · 1 Methoden
- `manual_clock`
  - 1 struct
    - `struct ManualClock` · 6 Methoden
- `mocks`
  - 1 enum, 2 struct
    - `struct MockEmbedder` · 2 Methoden
    - `enum MetricEvent`
    - `struct TestMetricsSink` · 3 Methoden
- `reference_model`
  - 1 enum, 1 struct
    - `enum RefOp`
    - `struct ReferenceModel` · 13 Methoden
- Interne Re-Exporte (`pub use`): `InMemoryGraphIndex`, `InMemoryStorageEngine`, `InMemoryTextIndex`, `InMemoryVectorIndex`, `ManualClock`, `*`

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-checkpoint` | – | `InMemoryStorageEngine`×2, `FaultVfs` |
| `contextra-crypto` | – | `mint_witness_for_tests`×4, `FaultVfs` |
| `contextra-engine` | – | `InMemoryStorageEngine` |
| `contextra-mvcc` | – | `ManualClock`×6 |
| `contextra-store` | – | `ManualClock`×4, `ReferenceModel`×4 |

**Nach außen ungenutzte öffentliche Items (Heuristik, 18 von 23)**: `run_clock_suite`, `ConformanceFailure`, `ConformanceResult`, `run_graph_index_suite`, `run_metrics_sink_suite`, `run_rng_suite`, `run_storage_engine_suite`, `run_text_index_suite`, `run_vector_index_suite`, `run_vfs_suite`, `FaultConfig`, `InMemoryGraphIndex`, `InMemoryTextIndex`, `InMemoryVectorIndex`, `MockEmbedder`, `MetricEvent`, `TestMetricsSink`, `RefOp`

### contextra-text

- Pfad: `crates/contextra-text` · Zeilen: 15219 · Rust-Dateien: 62 · Ring: 0
- Interne Cargo-Abhängigkeiten: `contextra-ports` (normal/optional via types-compat), `contextra-types` (normal/optional via types-compat)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-types` | 22 | 137 | 2 | `ContextraError`×3, `ContextraError::InvalidInput`×3, `ContextraError::Storage`×3, `DocId`×3, `ScoredDocument`×3, `ContextraError::Text`×2, `ContextraError::Io`×2, `DocId::new`×2, `ContextraError::Internal` | `Result`×67, `TOMBSTONE_BIT`×57, `TxId`×7, `TxId::new`×2, `ContextraError::CapabilityUnsupported`×2 … (+1) |
| `contextra-ports` | 8 | 87 | 5 | `StorageEngine`×6, `scanner`×2 | `BoxFuture`×50, `StorageStats`×35, `clock::SystemClock` |

**Pro Komponente**

- `(crate-root)` (318 Zeilen)
  - → `contextra-ports`: prod: `StorageEngine`×3 · dev: `BoxFuture`
- `error` (65 Zeilen)
  - → `contextra-types`: prod: `ContextraError`×3, `ContextraError::InvalidInput`×3, `ContextraError::Storage`×3, `ContextraError::Text`×2, `ContextraError::Io`×2, `ContextraError::Internal`
- `inverted` (2842 Zeilen)
  - → `contextra-ports`: prod: `scanner`×2, `StorageEngine` · dev: `StorageStats`×3
  - → `contextra-types`: dev: `TOMBSTONE_BIT`×6, `ContextChunk`
- `stream` (162 Zeilen)
  - → `contextra-ports`: prod: `StorageEngine`
- `types` (114 Zeilen)
  - → `contextra-types`: prod: `DocId`×3, `ScoredDocument`×3, `DocId::new`×2
- `wand` (814 Zeilen)
  - → `contextra-ports`: prod: `StorageEngine` · dev: `StorageStats`×2, `BoxFuture`
  - → `contextra-types`: dev: `TxId`×7, `TOMBSTONE_BIT`×6, `TxId::new`×2
- `tests/alloc_profiler.rs` (569 Zeilen)
  - → `contextra-ports`: dev: `StorageStats`×2
  - → `contextra-types`: dev: `Result`×16
- `tests/audit_inspection.rs` (578 Zeilen)
  - → `contextra-ports`: dev: `BoxFuture`×32, `StorageStats`×4
  - → `contextra-types`: dev: `Result`×34, `TOMBSTONE_BIT`×10
- `tests/bm25_candidate_stream.rs` (321 Zeilen)
  - → `contextra-ports`: dev: `StorageStats`×2
- `tests/concurrent_metadata.rs` (196 Zeilen)
  - → `contextra-ports`: dev: `StorageStats`×2
- `tests/docid_128_tombstone_resolution.rs` (249 Zeilen)
  - → `contextra-ports`: dev: `BoxFuture`×16, `StorageStats`×2
  - → `contextra-types`: dev: `Result`×17, `TOMBSTONE_BIT`×6
- `tests/pipeline_identity_regression.rs` (271 Zeilen)
  - → `contextra-ports`: dev: `StorageStats`×2
  - → `contextra-types`: dev: `TOMBSTONE_BIT`×6
- `tests/posting_roundtrip_tests.rs` (122 Zeilen)
  - → `contextra-types`: dev: `DocId`
- `tests/proptest_bm25_invariants.rs` (335 Zeilen)
  - → `contextra-ports`: dev: `StorageStats`×2
  - → `contextra-types`: dev: `TOMBSTONE_BIT`×6
- `tests/text_scanner_purger_tests.rs` (303 Zeilen)
  - → `contextra-ports`: dev: `StorageStats`×2, `clock::SystemClock`, `scanner`
- `tests/text_search_at_wrappers.rs` (331 Zeilen)
  - → `contextra-ports`: dev: `StorageStats`×2
  - → `contextra-types`: dev: `TOMBSTONE_BIT`×6, `ContextraError::CapabilityUnsupported`×2
- `tests/tombstone_read_your_writes.rs` (267 Zeilen)
  - → `contextra-ports`: dev: `StorageStats`×2
  - → `contextra-types`: dev: `TOMBSTONE_BIT`×6
- `tests/tombstone_update.rs` (221 Zeilen)
  - → `contextra-ports`: dev: `StorageStats`×2
- `tests/wand_differential.rs` (430 Zeilen)
  - → `contextra-ports`: dev: `StorageStats`×2
  - → `contextra-types`: dev: `TOMBSTONE_BIT`×5
- `tests/wand_tie_break_threshold.rs` (232 Zeilen)
  - → `contextra-ports`: dev: `StorageStats`×2
- `tests/write_amplification.rs` (219 Zeilen)
  - → `contextra-ports`: dev: `StorageStats`×2

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `(crate-root)`
  - 1 struct
    - `struct Bm25Scorer` · 1 Methoden ⚠ fremd: `contextra-ports`
- `bm25`
  - 2 const, 3 fn, 3 struct, 1 type
    - `const BM25_K1`
    - `const BM25_B`
    - `struct BM25` · 2 Methoden
    - `fn score_term`
    - `fn score_term_with_params`
    - `type FieldId`
    - `struct FieldWeight` · 1 Methoden
    - `struct BM25F` · 2 Methoden
    - `fn score_term_bm25f`
- `contract`
  - 1 enum, 2 struct, 2 trait, 1 type
    - `enum Error`
    - `type Result`
    - `trait Clock` · Methoden: now_unix_ms
    - `trait Rng` · Methoden: next_u64
    - `struct DocKey`
    - `struct Epoch`
- `domain`
  - 4 fn, 4 static, 2 struct, 1 trait
    - `struct LegalDomainVocabulary`
    - `fn get_legal_compound_stems`
    - `fn get_legal_protected_terms`
    - `static LEGAL_COMPOUND_STEMS`
    - `static LEGAL_PROTECTED_TERMS`
    - `struct MedicalDomainVocabulary`
    - `fn get_medical_compound_stems`
    - `fn get_medical_protected_terms`
    - `static MEDICAL_COMPOUND_STEMS`
    - `static MEDICAL_PROTECTED_TERMS`
    - `trait DomainVocabulary` · Methoden: compound_stems, protected_terms, domain_id
- `error`
  - 1 enum, 1 type
    - `enum TextError`
    - `type Result`
- `inverted`
  - 1 enum, 2 fn, 5 struct
    - `struct InvertedIndex` *(re-exportiert)* · 14 Methoden
    - `struct BM25MorphIndex` *(re-exportiert)* · 2 Methoden
    - `struct PurgeReport` · 3 Methoden
    - `struct TextPurger` · 5 Methoden ⚠ fremd: `contextra-ports`
    - `fn text_purger` ⚠ fremd: `contextra-ports`
    - `enum ParsedTextTarget` · 1 Methoden
    - `struct TextScanner` · 4 Methoden
    - `fn text_scanner`
- `memory_index`
  - 3 const, 4 struct
    - `const MAX_DOC_BYTES`
    - `const MAX_DOC_TERMS`
    - `const MAX_POSTINGS_PER_TERM`
    - `struct CompactStats`
    - `struct KeyInspection`
    - `struct IndexSnapshot` · 1 Methoden
    - `struct MemoryTextIndex` · 9 Methoden
- `morphology`
  - 2 struct
    - `struct TokenReductionMetrics` *(re-exportiert)* · 1 Methoden
    - `struct PassthroughTokenizer` *(re-exportiert)* · 1 Methoden
- `posting_list`
  - 2 const, 4 struct, 2 type
    - `const POSTING_LIST_V2_MAGIC`
    - `const BLOCK_SIZE`
    - `type DocIdRaw`
    - `type DocIdRaw`
    - `struct PostingBlockInfo`
    - `struct Posting` · 2 Methoden
    - `struct PostingList` · 10 Methoden
    - `struct ResidentPostingIndex` · 7 Methoden
- `stream`
  - 1 const, 1 struct
    - `const DEFAULT_STREAM_BATCH_SIZE`
    - `struct Bm25CandidateStream` · 7 Methoden ⚠ fremd: `contextra-ports`
- `tokenizer`
  - 2 enum, 3 fn, 5 struct, 3 trait
    - `fn compile_protected_regex`
    - `fn get_protected_regex`
    - `trait Tokenizer` · Methoden: tokenize
    - `struct DefaultTokenizer` · 2 Methoden
    - `struct GermanMorphTokenizer` · 2 Methoden
    - `fn tokenize`
    - `struct Token` · 1 Methoden
    - `enum OverflowPolicy`
    - `struct PipelineLimits`
    - `enum NormalizationPolicy` · 1 Methoden
    - `trait TokenFilter` · Methoden: filter, filter_id
    - `trait Analyzer` · Methoden: analyze, fingerprint
    - `struct AnalyzerPipeline` · 8 Methoden
- `traits`
  - 2 trait
    - `trait Index` · Methoden: insert, remove, search
    - `trait Maintain` · Methoden: compact, vacuum
- `types`
  - 1 const, 3 struct
    - `struct DocId` ⚠ fremd: `contextra-types`
    - `struct DocId` · 5 Methoden ⚠ fremd: `contextra-types`
    - `struct TextHit` ⚠ fremd: `contextra-types`
    - `const MAX_SEARCH_K`
- `wand`
  - 1 fn, 2 struct
    - `struct BoundedTopK` · 4 Methoden
    - `struct WandSearchResult`
    - `fn block_max_wand_search` ⚠ fremd: `contextra-ports`
- Zusätzlich 9 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-engine` | `inverted::InvertedIndex`×2, `Language`×2 | `Language::English`×63, `Language::German`×4 |
| `contextra-db` | – | `Language::English`×6, `Language`×2, `bm25::score_term`, `inverted::InvertedIndex` |

**Nach außen ungenutzte öffentliche Items (Heuristik, 76 von 78)**: `BM25_K1`, `BM25_B`, `BM25`, `score_term_with_params`, `FieldId`, `FieldWeight`, `BM25F`, `score_term_bm25f`, `Error`, `Result`, `Clock`, `Rng`, `DocKey`, `Epoch`, `LegalDomainVocabulary`, `get_legal_compound_stems`, `get_legal_protected_terms`, `LEGAL_COMPOUND_STEMS`, `LEGAL_PROTECTED_TERMS`, `MedicalDomainVocabulary`, `get_medical_compound_stems`, `get_medical_protected_terms`, `MEDICAL_COMPOUND_STEMS`, `MEDICAL_PROTECTED_TERMS`, `DomainVocabulary`, `TextError`, `Result`, `BM25MorphIndex`, `PurgeReport`, `TextPurger` …

### contextra-types

- Pfad: `crates/contextra-types` · Zeilen: 8287 · Rust-Dateien: 37 · Ring: 0
- Interne Cargo-Abhängigkeiten: keine

**Ausgehend (Importe aus anderen Workspace-Crates)**

Keine.

**Pro Komponente**

Keine Importe aus anderen Workspace-Crates.

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `error`
  - 3 enum, 1 struct, 1 type
    - `enum HnswDeletionError`
    - `struct StepId` · 2 Methoden
    - `enum CacheDirective`
    - `type Result`
    - `enum ContextraError` · 14 Methoden
- `error_dto`
  - 1 struct
    - `struct ContextraErrorDto` · 2 Methoden
- `ids`
  - 1 const, 7 struct
    - `const MAX_REF_LEN`
    - `struct DocRef`
    - `struct ErasureId` · 2 Methoden
    - `struct FileId` · 2 Methoden
    - `struct StoreId` · 2 Methoden
    - `struct GateId` · 2 Methoden
    - `struct Seq` · 2 Methoden
    - `struct Timestamp` · 2 Methoden
- `model_fingerprint`
  - 1 struct
    - `struct ModelFingerprint` · 1 Methoden
- `retrieval_strategy`
  - 1 enum
    - `enum RetrievalStrategy`
- `schema`
  - 2 enum
    - `enum DocIdWidth` · 2 Methoden
    - `enum ManifestSchemaVersion` · 4 Methoden
- `tenant_scope`
  - 1 enum, 1 struct
    - `struct TenantScoped` · 4 Methoden
    - `enum TenantScopeViolation`
- `tombstone`
  - 1 struct, 1 trait
    - `trait TombstoneSemanticsCheck` · Methoden: is_tombstone, make_tombstone
    - `struct SeqBitTombstone`
- `types`
  - 3 const, 12 enum, 2 fn, 30 struct, 2 type
    - `enum BudgetStrategy`
    - `struct TokenBudget` · 11 Methoden
    - `struct Reservation` · 1 Methoden
    - `struct ResourceBudget`
    - `struct ResourceTracker` · 6 Methoden
    - `enum DistanceMetric` *(re-exportiert)* · 2 Methoden
    - `struct Embedding` *(re-exportiert)* · 5 Methoden
    - `struct ScoredDocument` *(re-exportiert)* · 1 Methoden
    - `struct RerankResult` *(re-exportiert)*
    - `struct TenantId` *(re-exportiert)* · 5 Methoden
    - `struct CollectionId` *(re-exportiert)* · 4 Methoden
    - `struct DocId` *(re-exportiert)*
    - `struct DocId` *(re-exportiert)* · 8 Methoden
    - `struct EntityId` *(re-exportiert)* · 8 Methoden
    - `struct TxId` *(re-exportiert)* · 6 Methoden
    - `fn serialize` *(re-exportiert)*
    - `fn deserialize` *(re-exportiert)*
    - `struct WorkflowState` *(re-exportiert)*
    - `const TOMBSTONE_BIT` *(re-exportiert)*
    - `const EXPIRY_METADATA_KEY` *(re-exportiert)*
    - `const MAX_SEARCH_K` *(re-exportiert)*
    - `enum LinkRelation` *(re-exportiert)*
    - `struct MemoryLink` *(re-exportiert)*
    - `struct Entity` *(re-exportiert)* · 2 Methoden
    - `struct Edge` *(re-exportiert)* · 7 Methoden
    - `enum MemoryType` *(re-exportiert)* · 3 Methoden
    - `struct TlHfdParams` *(re-exportiert)*
    - `enum PprAlgorithm` *(re-exportiert)*
    - `struct PprConfig` *(re-exportiert)*
    - `struct ConfigFingerprint` *(re-exportiert)* · 4 Methoden
    - `enum AutoExtractionMode` *(re-exportiert)* · 1 Methoden
    - `enum FilterExpr` · 1 Methoden
    - `struct ImportanceScore` · 2 Methoden
    - `enum DecayFunction` · 2 Methoden
    - `struct MemoryImportance` · 2 Methoden
    - `enum FusionStrategy`
    - `struct SignalFusionStrategies` · 2 Methoden
    - `enum GraphTraversalStrategy`
    - `struct FusionWeights` · 5 Methoden
    - `struct ContextChunk` · 3 Methoden
    - `struct ContextWindow`
    - `enum Signal`
    - `enum OnSignalFailure`
    - `struct QueryRewriteOutput` · 3 Methoden
    - `struct ScoredEntry`
    - `struct HybridQuery` · 1 Methoden
    - `type QueryOptions`
    - `struct HybridQueryBuilder` · 17 Methoden
    - `type QueryOptionsBuilder`
- `witness`
  - 1 enum, 2 struct
    - `enum ScanMethod`
    - `struct RequestContext` · 1 Methoden
    - `struct SagaRecord` · 1 Methoden
- Zusätzlich 1 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).
- Interne Re-Exporte (`pub use`): `TenantScoped`

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-engine` | `ContextraError::invalid_input`×50, `EntityId`×15, `HybridQuery`×13, `Result`×11, `MAX_SEARCH_K`×11, `ContextraError::Internal`×9, `ImportanceScore::new`×8, `domain::MemoryLink`×7, `EntityId::from_key`×7, `ContextraError::Serialization`×6 … (+40) | `ContextraError::from`×10, `domain::LinkRelation::Elaborates`×2, `ContextraError::HnswConnectivityDegraded`, `FusionWeights::new` … (+8) |
| `contextra-vector` | `ContextraError::invalid_input`×21, `MAX_SEARCH_K`×15, `Result`×9, `ContextraError::Index`×7, `DocId`×6, `ContextraError::HnswConnectivityDegraded`×5, `DistanceMetric`×5, `TxId`×4, `ScoredDocument`×4, `ContextraError`×3 … (+11) | `TxId::new` |
| `contextra-graph` | `EntityId`×18, `ContextraError::InvalidInput`×5, `Result`×5, `ContextraError`×3, `TxId`×3, `Edge`×2, `Edge::new`×2, `PprConfig`×2, `ContextraError::Internal`×2, `MemoryLink` … (+3) | `Entity::new`×10, `TlHfdParams::default`×4, `PprAlgorithm::Auto`×4, `Entity`×2 … (+1) |
| `contextra-store` | `ContextraError::Storage`×8, `Result`×5, `ContextraError`×5, `TOMBSTONE_BIT`×4, `TxId`×4, `ContextraError::MemoryBudgetExceeded`×2, `ContextraError::Internal`×2, `TxId::new`×2, `TxId::INTERNAL_BASE`×2, `ContextraError::Sandbox` … (+4) | `CollectionId`×6, `DocId`×6, `ContextraError::InvalidInput`×2, `TenantId` … (+1) |
| `contextra-rank` | `DocId`×4, `SignalFusionStrategies`×4, `ContextraError::invalid_input`×3, `FusionWeights`×3, `FusionStrategy`×3, `OnSignalFailure`×3, `ContextraError`×2, `ContextChunk`×2, `ConfigFingerprint`×2, `FusionStrategy::Rrf`×2 … (+7) | – |
| `contextra-crypto` | `TenantId`×8, `Result`×7, `ScanMethod::FeatureDisabled`×3, `ContextraError::Crypto`×3, `TenantScoped`×2, `ScanMethod::Structural`×2, `ScanMethod::NotMeasured`×2, `ContextraError`×2, `RequestContext`, `SagaRecord` … (+4) | `ContextraError::Io`×4, `ContextraError::NotFound` |
| `contextra-text` | `ContextraError`×3, `ContextraError::InvalidInput`×3, `ContextraError::Storage`×3, `DocId`×3, `ScoredDocument`×3, `ContextraError::Text`×2, `ContextraError::Io`×2, `DocId::new`×2, `ContextraError::Internal` | `Result`×67, `TOMBSTONE_BIT`×57, `TxId`×7, `TxId::new`×2 … (+2) |
| `contextra-testkit` | `ContextraError::Io`×9, `Result`×8, `ContextraError::NotFound`×2, `TxId` | – |
| `contextra` | `error::ContextraError`×4, `DistanceMetric`×2, `ContextraError::Internal`, `types::domain::AutoExtractionMode`, `types::domain`, `ContextraError`, `ContextraError::DurabilityConfig`, `error::Result`, `ScoredEntry` | `DocId`×2, `DocId::new`×2, `ContextraError::InvalidInput`, `ScoredDocument` … (+1) |
| `contextra-simd` | `ContextraError::invalid_input`×3, `DistanceMetric`×3, `DistanceMetric::Cosine`×2, `DistanceMetric::Euclidean`×2, `DistanceMetric::DotProduct`×2, `ContextraError` | – |
| `contextra-mcp` | `ContextraError`×4, `ContextraError::InvalidInput`, `ContextraError::NotFound` | `TenantId::try_new`, `AutoExtractionMode`, `TenantId`, `MAX_SEARCH_K` |
| `contextra-db` | `DistanceMetric`, `MAX_SEARCH_K`, `EntityId`, `EntityId::from_key` | `Result`×5, `TxId`×5, `FusionWeights`×3, `TenantId`×2 … (+16) |
| `contextra-ports` | `RequestContext`×2, `ids::ErasureId` | – |
| `contextra-bench` | `Result`, `ContextraError::CapabilityUnsupported` | `TxId` |
| `contextra-checkpoint` | – | `TxId`×4, `Result`×3, `ContextraError`×2, `ContextraError::Internal`×2 … (+2) |
| `contextra-mvcc` | – | `TxId`×5, `DocId`, `types` |

*Hinweis: Mindestens ein Crate importiert per Glob (`*`); ungenutzte Items sind deshalb nicht bestimmbar.*

### contextra-vector

- Pfad: `crates/contextra-vector` · Zeilen: 25202 · Rust-Dateien: 127 · Ring: 0
- Interne Cargo-Abhängigkeiten: `contextra-mvcc` (normal/optional via ports-compat), `contextra-ports` (normal/optional via ports-compat), `contextra-sys` (normal/optional via ports-compat), `contextra-types` (normal/optional via ports-compat)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-types` | 97 | 40 | 16 | `ContextraError::invalid_input`×21, `MAX_SEARCH_K`×15, `Result`×9, `ContextraError::Index`×7, `DocId`×6, `ContextraError::HnswConnectivityDegraded`×5, `DistanceMetric`×5, `TxId`×4, `ScoredDocument`×4, `ContextraError`×3 … (+11) | `TxId::new` |
| `contextra-ports` | 33 | 78 | 7 | `Clock`×15, `MetricsSink`×6, `Rng`×4, `VectorIndex`×2, `scanner`×2, `NoopMetricsSink`×2, `SeededRng::new`×2 | `SeededRng`×5, `vector_index::VectorIndex`, `clock::SystemClock` |
| `contextra-mvcc` | 11 | 0 | 3 | `IndexOp::Insert`×2, `IndexOp::Delete`×2, `SequenceLog::new`×2, `SeqLogChange::Insert`, `SeqLogChange::Delete`, `TxBuffer`, `TxBuffer::new_with_config`, `SequenceLog` | – |
| `contextra-sys` | 2 | 0 | 2 | `mmap_readonly`×2 | – |

**Pro Komponente**

- `acorn` (180 Zeilen)
  - → `contextra-types`: prod: `DocId`
- `candidate_stream` (147 Zeilen)
  - → `contextra-ports`: prod: `VectorIndex`
- `diskann` (3661 Zeilen)
  - → `contextra-ports`: prod: `VectorIndex` · dev: `VectorIndex`×2
  - → `contextra-sys`: prod: `mmap_readonly`
  - → `contextra-types`: prod: `MAX_SEARCH_K`×9
- `distance` (617 Zeilen)
  - → `contextra-types`: dev: `DistanceMetric`
- `error` (97 Zeilen)
  - → `contextra-types`: prod: `ContextraError`×3, `ContextraError::Index`×3, `ContextraError::invalid_input`×2, `ContextraError::HnswConnectivityDegraded`×2, `ContextraError::GraphRepairFailed`×2, `ContextraError::Io`×2, `HnswDeletionError::VerificationFailed`, `ContextraError::capability_unsupported`, `ContextraError::InvalidInput`, `ContextraError::CapabilityUnsupported`
- `hnsw` (6344 Zeilen)
  - → `contextra-mvcc`: prod: `IndexOp::Insert`×2, `IndexOp::Delete`×2, `SequenceLog::new`×2, `SeqLogChange::Insert`, `SeqLogChange::Delete`, `TxBuffer`, `TxBuffer::new_with_config`, `SequenceLog`
  - → `contextra-ports`: prod: `Clock`×15, `MetricsSink`×6, `Rng`×4, `scanner`×2, `NoopMetricsSink`×2, `SeededRng::new`×2 · dev: `Rng`×3, `SeededRng::new`×3, `MetricsSink`×2, `VectorIndex`
  - → `contextra-types`: prod: `MAX_SEARCH_K`×6, `ContextraError::Index`×4, `Result`×3, `ContextraError::HnswConnectivityDegraded`×3, `DocId`×2, `ContextraError::Conflict`×2, `DistanceMetric` · dev: `DistanceMetric`
- `persistence` (1140 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`×3
  - → `contextra-sys`: prod: `mmap_readonly`
  - → `contextra-types`: dev: `Result`×2
- `quantize` (1022 Zeilen)
  - → `contextra-types`: prod: `ContextraError::invalid_input`×19, `Result`×6, `DistanceMetric` · dev: `ContextraError::InvalidInput`×6, `Result`×3
- `types` (307 Zeilen)
  - → `contextra-types`: prod: `TxId`×4, `ScoredDocument`×4, `DocId`×3, `DistanceMetric`×3, `DocId::new`×2, `DistanceMetric::Cosine`×2, `DistanceMetric::Euclidean`×2, `DistanceMetric::DotProduct`×2
- `benches/acorn_vs_post_filter.rs` (53 Zeilen)
  - → `contextra-types`: dev: `DocId`
- `benches/audit_benchmarks.rs` (430 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `benches/distance_nan_impact_bench.rs` (24 Zeilen)
  - → `contextra-types`: dev: `DistanceMetric`
- `benches/filtered_search_selectivity_bench.rs` (166 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `benches/flush_threshold_amplification.rs` (177 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `benches/hnsw_bench.rs` (90 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `benches/hnsw_delete_search_bench.rs` (86 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `benches/overfetch_selectivity.rs` (79 Zeilen)
  - → `contextra-ports`: dev: `SeededRng`, `VectorIndex`
- `benches/partial_rebuild_recall_bench.rs` (135 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `benches/sq8_bench.rs` (40 Zeilen)
  - → `contextra-types`: dev: `DistanceMetric`
- `examples/standalone_vector.rs` (41 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `fuzz/fuzz_targets/fuzz_hnsw_persistence.rs` (119 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `fuzz/fuzz_targets/hnsw_insert_search.rs` (79 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/acorn_hnsw_differential.rs` (253 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/acorn_naive_recall.rs` (70 Zeilen)
  - → `contextra-types`: dev: `DocId`
- `tests/acorn_selectivity_edge_cases_test.rs` (78 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/adaptive_ef_recall.rs` (279 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/campaign_hnsw_backoff.rs` (50 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`, `SeededRng::new`
- `tests/campaign_j23_vector_index.rs` (478 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
  - → `contextra-types`: dev: `ContextraError::Index`×6
- `tests/campaign_j24_test.rs` (609 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/connectivity_regression.rs` (301 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/deleted_nodes_lock_contention.rs` (176 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/deletion_ghost_pointer_proptest.rs` (47 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/deletion_zeroize_test.rs` (45 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/determinism_test.rs` (111 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/differential_testing.rs` (78 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/diskann_corruption_fallback_test.rs` (103 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
  - → `contextra-types`: dev: `TxId::new`
- `tests/diskann_docid128_tombstones.rs` (67 Zeilen)
  - → `contextra-ports`: dev: `vector_index::VectorIndex`
  - → `contextra-types`: dev: `ContextraError::Io`
- `tests/diskann_fault_injection_test.rs` (73 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/diskann_recall_test.rs` (219 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
  - → `contextra-types`: dev: `TxId`
- `tests/diskann_search_filtered.rs` (171 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
  - → `contextra-types`: dev: `MAX_SEARCH_K`
- `tests/distance_determinism.rs` (64 Zeilen)
  - → `contextra-types`: dev: `DistanceMetric`
- `tests/docid_128_hnsw.rs` (70 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
  - → `contextra-types`: dev: `Result`×2
- `tests/filtered_selectivity_recall.rs` (154 Zeilen)
  - → `contextra-ports`: dev: `SeededRng`, `VectorIndex`
- `tests/golden_distance_oracle.rs` (159 Zeilen)
  - → `contextra-types`: dev: `DistanceMetric`
- `tests/hnsw_edge_cases.rs` (321 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/hnsw_layer_determinism.rs` (110 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/hnsw_rebuild_connectivity_test.rs` (67 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/hnsw_rebuild_search_consistency.rs` (134 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
  - → `contextra-types`: dev: `DistanceMetric::Euclidean`
- `tests/hnsw_scanner_purge_test.rs` (228 Zeilen)
  - → `contextra-ports`: dev: `clock::SystemClock`, `VectorIndex`, `scanner`
- `tests/hnsw_seed_determinism.rs` (295 Zeilen)
  - → `contextra-ports`: dev: `SeededRng`, `VectorIndex`
- `tests/hnsw_snapshot_delete_test.rs` (129 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/k02_snapshot_rebuild_determinism.rs` (153 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
  - → `contextra-types`: dev: `DistanceMetric::Euclidean`
- `tests/loom_quantizer_race_test.rs` (222 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`×2
  - → `contextra-types`: dev: `ContextraError`
- `tests/mmap_toctou_test.rs` (228 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`×2
- `tests/nan_validation_policy.rs` (171 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/partial_rebuild_recall.rs` (146 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/partial_rebuild_recall_regression.rs` (325 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/partial_rebuild_veto_f02_guard.rs` (97 Zeilen)
  - → `contextra-ports`: dev: `SeededRng`, `VectorIndex`
- `tests/poisoning.rs` (109 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/proptest_distance_quantize.rs` (89 Zeilen)
  - → `contextra-types`: dev: `DistanceMetric`
- `tests/quantize_persistence_audit.rs` (302 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
  - → `contextra-types`: dev: `ContextraError`
- `tests/quantizer_encapsulation_regression.rs` (35 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/rabitq_recall_calibration.rs` (117 Zeilen)
  - → `contextra-types`: dev: `ContextraError`
- `tests/ram_reduction.rs` (59 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/recalibration.rs` (56 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
  - → `contextra-types`: dev: `DistanceMetric::Euclidean`
- `tests/recall.rs` (67 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/recall_audit.rs` (405 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/recall_vs_brute_force.rs` (155 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/reconstruction_attack_test.rs` (71 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/rollback.rs` (55 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/simd_numerical_audit.rs` (171 Zeilen)
  - → `contextra-types`: dev: `DistanceMetric`
- `tests/sq8_bias_calibration.rs` (20 Zeilen)
  - → `contextra-types`: dev: `DistanceMetric`
- `tests/sq8_rerank_recall.rs` (163 Zeilen)
  - → `contextra-ports`: dev: `SeededRng`, `VectorIndex`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `acorn`
  - 1 enum, 1 fn, 1 struct, 1 trait
    - `fn compute_gamma_edge_budget`
    - `trait FilteredIndex` · Methoden: search_knn_acorn
    - `enum AcornError`
    - `struct NaiveReferenceIndex` · 3 Methoden
- `candidate_stream`
  - 1 const, 1 struct
    - `const DEFAULT_VECTOR_STREAM_BATCH_SIZE`
    - `struct VectorCandidateStream` · 8 Methoden ⚠ fremd: `contextra-ports`
- `casting`
  - 8 fn
    - `fn to_u32`
    - `fn u32_to_usize`
    - `fn u64_to_usize`
    - `fn u64_to_u32`
    - `fn to_u8`
    - `fn to_u64`
    - `fn f32_to_usize`
    - `fn f64_to_usize`
- `compute_pool`
  - 2 struct
    - `struct TaskHandle` · 1 Methoden
    - `struct ComputePool` · 5 Methoden
- `contract`
  - 2 enum, 2 struct, 6 trait, 1 type
    - `enum Error`
    - `type Result`
    - `trait Clock` · Methoden: now_unix_ms
    - `trait Rng` · Methoden: next_u64
    - `struct ExternalId`
    - `enum Metric`
    - `struct ScoredPoint`
    - `trait DistanceFn` · Methoden: distance
    - `trait Index` · Methoden: insert, remove, search
    - `trait Maintain` · Methoden: compact, vacuum
    - `trait BlockTransform` · Methoden: transform, inverse_transform
- `diskann`
  - 1 struct
    - `struct DiskAnnIndex` *(re-exportiert)* · 2 Methoden
- `distance`
  - 16 fn, 4 struct, 1 trait
    - `struct CosineSimilarityPartsU8`
    - `trait DistanceKernel` · Methoden: distance, distance_f32_bytes, distance_trusted, distance_f32_bytes_trusted, cosine_distance, dot_product_distance…
    - `struct ScalarKernel`
    - `struct ScalarDistance` · 2 Methoden
    - `struct InjectedDistanceKernel` · 1 Methoden
    - `fn validate_vector`
    - `fn compute_distance` ⚠ fremd: `contextra-types`
    - `fn compute_distance_trusted` ⚠ fremd: `contextra-types`
    - `fn compute_distance_f32_bytes_trusted` ⚠ fremd: `contextra-types`
    - `fn cosine_distance`
    - `fn dot_product_distance`
    - `fn euclidean_distance`
    - `fn euclidean_distance_sq_u8`
    - `fn euclidean_distance_sq_f32_u8`
    - `fn dot_product_u8`
    - `fn dot_product_f32_u8`
    - `fn cosine_similarity_parts_u8`
    - `fn normalize_inplace`
    - `fn cosine_distance_scalar`
    - `fn dot_product_scalar`
    - `fn euclidean_distance_scalar`
- `error`
  - 1 enum, 1 type
    - `enum VectorError`
    - `type Result`
- `fixed_hasher`
  - 3 fn, 2 type
    - `fn fixed_hasher`
    - `type FixedHashSet`
    - `type FixedHashMap`
    - `fn new_fixed_hashset`
    - `fn new_fixed_hashmap`
- `hnsw`
  - 2 const, 1 enum, 2 fn, 12 struct, 1 trait, 1 type
    - `struct AdaptiveEfPolicy` · 2 Methoden
    - `struct AdaptiveEfStats`
    - `struct AdaptiveEfStateMachine` · 6 Methoden
    - `const ARENA_ALIGNMENT_BYTES`
    - `const ARENA_ALIGNMENT_U32`
    - `type BacklinkKey`
    - `struct BacklinkTable` · 4 Methoden
    - `struct HnswArena` · 10 Methoden
    - `struct MonotonicCounterClock` · 2 Methoden
    - `struct DeletionStats`
    - `trait GhostFreeVectorIndex` · Methoden: remove_with_graph_repair
    - `struct HnswPurger` · 1 Methoden
    - `fn hnsw_purger`
    - `struct HnswScanner` · 2 Methoden
    - `fn hnsw_scanner`
    - `struct ResidualReport` · 1 Methoden
    - `struct Sq8Bias` · 2 Methoden
    - `enum VectorStagingOp`
    - `struct VectorTxBuffer` · 4 Methoden
- `partial_rebuild`
  - 1 fn, 2 struct
    - `struct PartialRebuildConfig`
    - `struct TraversalTracker` · 4 Methoden
    - `fn should_trigger_partial_rebuild`
- `persistence`
  - 1 fn, 2 struct
    - `fn write_atomic`
    - `struct MmapIndex` *(re-exportiert)* · 5 Methoden
- `quantize`
  - 3 const, 1 struct
- `quantize_rabitq`
  - 1 struct
- `types`
  - 2 const, 1 enum, 7 struct, 4 trait
  - … 20 weitere Items nicht aufgelistet (`--max-items`)
- Zusätzlich 31 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-engine` | `HnswIndex`×4, `hnsw::HnswIndex`×3, `acorn::FilteredIndex`, `HnswConfig` | `hnsw::HnswConfig::default`, `hnsw::HnswIndex::try_new`, `acorn`, `HnswIndex::try_new` |
| `contextra-db` | – | `HnswConfig`×2, `HnswIndex`, `HnswIndex::try_new` |

**Nach außen ungenutzte öffentliche Items (Heuristik, 99 von 100)**: `compute_gamma_edge_budget`, `AcornError`, `NaiveReferenceIndex`, `DEFAULT_VECTOR_STREAM_BATCH_SIZE`, `VectorCandidateStream`, `to_u32`, `u32_to_usize`, `u64_to_usize`, `u64_to_u32`, `to_u8`, `to_u64`, `f32_to_usize`, `f64_to_usize`, `TaskHandle`, `ComputePool`, `Error`, `Result`, `Clock`, `Rng`, `ExternalId`, `Metric`, `ScoredPoint`, `DistanceFn`, `Index`, `Maintain`, `BlockTransform`, `DiskAnnIndex`, `CosineSimilarityPartsU8`, `DistanceKernel`, `ScalarKernel` …

