# API: contextra-engine

> Erzeugt aus dem Quellcode am 2026-11-25, Repository HEAD ce4579a56. Eine Zeile je Item: Datei:Zeile und die **erste Zeile** der Signatur. Mehrzeilige Signaturen sind nur angeschnitten. Nicht kompiliert, daher ohne Typprüfung. Testcode (`tests`, `testing`, `test_doubles`) ist ausgeschlossen.

Umfang: 368 öffentliche Items, 118 crate-interne Sichtbarkeitsgrenzen.

## Innen: Abhängigkeiten zu anderen Workspace-Crates

- contextra-checkpoint
- contextra-graph
- contextra-mvcc
- contextra-ports
- contextra-rank
- contextra-text
- contextra-types
- contextra-vector

## Außen: öffentliche Schnittstelle (`pub`)

```rust
src/adapters/mod.rs:6  pub mod text
src/adapters/mod.rs:7  pub mod privacy
src/adapters/mod.rs:8  pub mod ollama
src/adapters/ollama.rs:7  pub trait OllamaBridge: Send + Sync
src/adapters/privacy.rs:6  pub fn to_canonical_tenant_id(raw: u64) -> TenantId
src/adapters/text.rs:9  pub struct Bm25Scorer<S: StorageEngine>
src/adapters/text.rs:15  pub fn new(storage: Arc<S>, namespace: &str) -> Self
src/backends.rs:12  pub trait Backends: Send + Sync + 'static
src/backends.rs:27  pub struct DefaultBackends
src/background_workers/config.rs:16  pub struct OrphanCleanupBackoffConfig
src/background_workers/config.rs:36  pub fn calculate_rebuild_cooldown(
src/background_workers/config.rs:52  pub trait OrphanCleanupIndex: Send + Sync
src/background_workers/expiry_workers.rs:22  pub const MAX_EXPIRED_PER_TICK: usize = 100
src/background_workers/expiry_workers.rs:25  pub fn start_expiry_cleanup_worker<S: StorageEngine>(
src/background_workers/expiry_workers.rs:75  pub fn start_expiry_reaper<S: StorageEngine>(
src/background_workers/expiry_workers.rs:86  pub fn start_decay_cleanup_worker<S: StorageEngine, V: VectorIndex>(
src/background_workers/expiry_workers.rs:114  pub fn start_thermostat_reaper<S: StorageEngine, V: VectorIndex>(
src/background_workers/hyperedge_worker.rs:21  pub const MAX_DEFERRED_HYPEREDGES_PER_TICK: usize = 1_000
src/background_workers/hyperedge_worker.rs:26  pub struct DeferredHyperedgeQueue
src/background_workers/hyperedge_worker.rs:32  pub fn new() -> Self
src/background_workers/hyperedge_worker.rs:53  pub fn start_hyperedge_cascade_deferred_worker<S: StorageEngine>(
src/background_workers/mod.rs:22  pub use config::{calculate_rebuild_cooldown, OrphanCleanupBackoffConfig, OrphanCleanupIndex}
src/background_workers/mod.rs:25  pub use expiry_workers::{start_decay_cleanup_worker, start_thermostat_reaper}
src/background_workers/mod.rs:27  pub use expiry_workers::{start_expiry_cleanup_worker, start_expiry_reaper, MAX_EXPIRED_PER_TICK}
src/background_workers/mod.rs:28  pub use hyperedge_worker::
src/background_workers/mod.rs:33  pub use orphan_workers::
src/background_workers/orphan_workers.rs:21  pub const MAX_ORPHANS_PER_TICK: usize = 100
src/background_workers/orphan_workers.rs:24  pub fn reap_orphans_step<T: Clone + Send + Sync + 'static>(
src/background_workers/orphan_workers.rs:32  pub fn start_orphan_cleanup_worker_with_config<
src/background_workers/orphan_workers.rs:171  pub fn start_orphan_cleanup_worker<T: Clone + Send + Sync + 'static>(
src/background_workers/orphan_workers.rs:189  pub fn start_orphan_reaper<T: Clone + Send + Sync + 'static>(
src/chunker/config.rs:16  pub struct ChunkerConfig
src/chunker/estimate.rs:20  pub fn estimate_tokens(text: &str) -> usize
src/chunker/mod.rs:23  pub use config::ChunkerConfig
src/chunker/mod.rs:24  pub use estimate::estimate_tokens
src/chunker/mod.rs:25  pub use windows::{chunk_text, chunk_text_with_overlap}
src/chunker/mod.rs:37  pub struct MarkdownChunker
src/chunker/mod.rs:134  pub fn new(config: ChunkerConfig) -> Self
src/chunker/mod.rs:139  pub fn with_defaults() -> Self
src/chunker/mod.rs:144  pub fn chunk(&self, doc_id: DocId, markdown: &str) -> Vec<ContextChunk>
src/chunker/windows.rs:16  pub fn chunk_text_with_overlap<'a>(
src/chunker/windows.rs:63  pub fn chunk_text(text: &str, chunk_size: usize) -> Vec<&str>
src/collection/crud/auto_extraction.rs:16  pub use crate::extraction::{EntityExtractionConfig, ExtractorMode}
src/collection/crud/auto_extraction.rs:18  pub use contextra_types::AutoExtractionMode
src/collection/crud/auto_extraction.rs:27  pub enum ExtractorMode
src/collection/crud/auto_extraction.rs:43  pub struct EntityExtractionConfig
src/collection/crud/auto_extraction.rs:86  pub fn recommended_mode_for_regulated(is_regulated: bool) -> AutoExtractionMode
src/collection/crud/auto_extraction.rs:96  pub struct AutoExtractionConfig
src/collection/crud/auto_extraction.rs:108  pub fn new(mode: AutoExtractionMode) -> Self
src/collection/crud/auto_extraction.rs:117  pub fn with_mode(mut self, mode: AutoExtractionMode) -> Self
src/collection/crud/auto_extraction.rs:130  pub fn is_enabled(&self) -> bool
src/collection/crud/auto_extraction.rs:170  pub fn with_auto_extraction(
src/collection/crud/auto_extraction.rs:180  pub fn set_auto_extraction(
src/collection/crud/auto_extraction.rs:189  pub fn set_auto_extraction_with_mode(
src/collection/crud/auto_extraction.rs:202  pub fn set_auto_extraction_config(&self, config: AutoExtractionConfig)
src/collection/crud/auto_extraction.rs:208  pub fn auto_extraction_config(
src/collection/crud/auto_extraction.rs:219  pub fn auto_extraction_config_with_extractor(
src/collection/crud/mod.rs:26  pub use auto_extraction::{AutoExtractionConfig, AutoExtractionMode, EntityExtractionConfig}
src/collection/crud/mod.rs:28  pub use read::{DEFAULT_SCAN_LIMIT, HARD_SCAN_CEILING, MAX_SCAN_RESULTS}
src/collection/crud/read.rs:15  pub const DEFAULT_SCAN_LIMIT: usize = 10_000
src/collection/crud/read.rs:19  pub const HARD_SCAN_CEILING: usize = 100_000
src/collection/crud/read.rs:22  pub const MAX_SCAN_RESULTS: usize = DEFAULT_SCAN_LIMIT
src/collection/kv_lock.rs:33  pub fn new() -> Self
src/collection/kv_lock.rs:44  pub fn shard_idx(&self, key: &str) -> usize
src/collection/kv_lock.rs:66  pub fn try_lock_shard<'a>(&'a self, idx: usize) -> Option<KeyGuard<'a>>
src/collection/kv_lock.rs:76  pub fn try_lock_for<'a>(&'a self, key: &str) -> Option<KeyGuard<'a>>
src/collection/kv_lock.rs:101  pub fn sorted_unique_shards<'a>(
src/collection/maintenance.rs:879  pub use contextra_graph::percolation::PercolationConfig
src/collection/maintenance.rs:883  pub struct PercolationResult
src/collection/mod.rs:18  pub mod kv_lock
src/collection/mod.rs:21  pub mod crud
src/collection/mod.rs:23  pub mod maintenance
src/collection/mod.rs:25  pub mod query_builder
src/collection/mod.rs:27  pub mod relate
src/collection/mod.rs:29  pub mod search
src/collection/mod.rs:31  pub mod tx
src/collection/mod.rs:62  pub struct CollectionConfig
src/collection/mod.rs:79  pub struct StoredDocument
src/collection/mod.rs:88  pub struct StoredDocumentMeta
src/collection/mod.rs:104  pub fn parse_importance_score(response: &str) -> f32
src/collection/mod.rs:124  pub fn compute_default_importance(text_opt: Option<&str>) -> contextra_types::ImportanceScore
src/collection/mod.rs:141  pub fn ensure_importance_metadata(
src/collection/mod.rs:211  pub fn extract_effective_importance(metadata: &Option<serde_json::Value>, now_tx: TxId) -> f32
src/collection/mod.rs:273  pub struct Collection<S: StorageEngine = LsmStorage, V: VectorIndex = HnswIndex>
src/collection/mod.rs:333  pub fn with_hnsw(
src/collection/mod.rs:357  pub const DECAY_DELETION_THRESHOLD: f32 = 0.05
src/collection/mod.rs:364  pub fn new(
src/collection/mod.rs:408  pub fn with_metrics_sink(self, metrics: Arc<dyn MetricsSink>) -> Self
src/collection/mod.rs:414  pub fn set_metrics_sink(&self, metrics: Arc<dyn MetricsSink>)
src/collection/mod.rs:419  pub fn metrics_sink(&self) -> Arc<dyn MetricsSink>
src/collection/mod.rs:424  pub fn with_clock(self, clock: Arc<dyn Clock>) -> Self
src/collection/mod.rs:430  pub fn set_clock(&self, clock: Arc<dyn Clock>)
src/collection/mod.rs:435  pub fn clock(&self) -> Arc<dyn Clock>
src/collection/mod.rs:440  pub fn with_config(self, config: CollectionConfig) -> Self
src/collection/mod.rs:446  pub fn set_config(&self, config: CollectionConfig)
src/collection/mod.rs:451  pub fn config(&self) -> CollectionConfig
src/collection/mod.rs:456  pub fn with_pressure_receiver(
src/collection/mod.rs:465  pub fn set_pressure_receiver(
src/collection/mod.rs:473  pub fn consolidation_in_progress(&self) -> Arc<AtomicBool>
src/collection/mod.rs:478  pub fn graph_index(&self) -> Arc<CsrGraph>
src/collection/mod.rs:483  pub fn graph_port(&self) -> Arc<dyn contextra_ports::GraphIndex>
src/collection/mod.rs:488  pub fn embedder(&self) -> Option<Arc<dyn TextEmbeddingEngine>>
src/collection/mod.rs:494  pub fn with_embedder(self, embedder: Arc<dyn TextEmbeddingEngine>) -> Self
src/collection/mod.rs:511  pub fn vector_index(&self) -> &Arc<V>
src/collection/mod.rs:517  pub fn namespaced_key(&self, key: &[u8], key_type: u8) -> Vec<u8>
src/collection/mod.rs:550  pub fn storage(&self) -> &Arc<S>
src/collection/mod.rs:555  pub fn user_key_prefix(&self) -> Vec<u8>
src/collection/mod.rs:561  pub fn name(&self) -> &str
src/collection/mod.rs:573  pub fn dimension(&self) -> usize
src/collection/mod.rs:661  pub fn set_community_detection_trigger_threshold(&self, threshold: u64)
src/collection/mod.rs:667  pub fn with_community_detection_trigger_threshold(self, threshold: u64) -> Self
src/collection/mod.rs:673  pub fn community_detection_trigger_threshold(&self) -> u64
src/collection/mod.rs:679  pub fn mutations_since_community_detection(&self) -> u64
src/collection/query_builder/builder.rs:23  pub const DEFAULT_RERANK_POOL_MULTIPLIER: usize = 10
src/collection/query_builder/builder.rs:28  pub const DEFAULT_RERANK_POOL_MAX: usize = 200
src/collection/query_builder/builder.rs:31  pub struct HybridQueryBuilder<'a, S: StorageEngine, V: VectorIndex>
src/collection/query_builder/builder.rs:62  pub fn new(collection: &'a Collection<S, V>) -> Self
src/collection/query_builder/builder.rs:94  pub fn text(mut self, text: impl Into<String>) -> Self
src/collection/query_builder/builder.rs:100  pub fn embedding(mut self, embedding: impl AsRef<[f32]>) -> Self
src/collection/query_builder/builder.rs:106  pub fn vector(self, vector: impl AsRef<[f32]>) -> Self
src/collection/query_builder/builder.rs:111  pub fn k(mut self, k: usize) -> Self
src/collection/query_builder/builder.rs:117  pub fn weights<W>(mut self, weights: W) -> Self
src/collection/query_builder/builder.rs:128  pub fn fusion_weights(mut self, weights: FusionWeights) -> Self
src/collection/query_builder/builder.rs:134  pub fn fusion_strategy(mut self, strategy: impl Into<SignalFusionStrategies>) -> Self
src/collection/query_builder/builder.rs:140  pub fn filter(mut self, filter: FilterExpr) -> Self
src/collection/query_builder/builder.rs:147  pub fn metadata_filter(mut self, filter: MetadataFilter) -> Self
src/collection/query_builder/builder.rs:155  pub fn strategy(mut self, strategy: impl Into<SearchStrategy>) -> Self
src/collection/query_builder/builder.rs:161  pub fn anchor_entities(mut self, anchors: impl IntoIterator<Item = EntityId>) -> Self
src/collection/query_builder/builder.rs:167  pub fn anchors(self, anchors: impl IntoIterator<Item = EntityId>) -> Self
src/collection/query_builder/builder.rs:172  pub fn same_community_as(mut self, entity_id: EntityId) -> Self
src/collection/query_builder/builder.rs:178  pub fn memory_type_filter(mut self, types: impl IntoIterator<Item = MemoryType>) -> Self
src/collection/query_builder/builder.rs:184  pub fn include_provenance(mut self, include: bool) -> Self
src/collection/query_builder/builder.rs:190  pub fn include_superseded(mut self, include: bool) -> Self
src/collection/query_builder/builder.rs:196  pub fn memory_types(self, types: impl IntoIterator<Item = MemoryType>) -> Self
src/collection/query_builder/builder.rs:201  pub fn filter_fn(mut self, f: impl Fn(DocId) -> bool + Send + Sync + 'static) -> Self
src/collection/query_builder/builder.rs:207  pub fn reranker(mut self, reranker: &'a (dyn Reranker + 'a)) -> Self
src/collection/query_builder/builder.rs:216  pub fn rerank_pool_multiplier(mut self, multiplier: usize) -> Self
src/collection/query_builder/builder.rs:224  pub fn rerank_pool_max(mut self, max: usize) -> Self
src/collection/query_builder/builder.rs:231  pub fn pid_controller(
src/collection/query_builder/builder.rs:240  pub fn seq(mut self, seq_no: u64) -> Self
src/collection/query_builder/builder.rs:246  pub fn as_of(mut self, timestamp: u64) -> Self
src/collection/query_builder/builder.rs:252  pub fn query_timestamp(mut self, timestamp: u64) -> Self
src/collection/query_builder/builder.rs:258  pub fn current_tx(mut self, tx: contextra_types::TxId) -> Self
src/collection/query_builder/builder.rs:267  pub fn on_signal_failure(mut self, policy: SignalFailurePolicy) -> Self
src/collection/query_builder/builder.rs:273  pub fn with_metrics_sink(mut self, sink: Arc<dyn MetricsSink>) -> Self
src/collection/query_builder/builder.rs:279  pub fn query_config(mut self, query: &contextra_types::HybridQuery) -> Self
src/collection/query_builder/builder.rs:309  pub fn query(&self) -> HybridQueryBuilder<'_, S, V>
src/collection/query_builder/mod.rs:24  pub const METRIC_SEARCH_LATENCY_VECTOR: &str = "search.latency.vector"
src/collection/query_builder/mod.rs:27  pub const METRIC_SEARCH_LATENCY_TEXT: &str = "search.latency.text"
src/collection/query_builder/mod.rs:30  pub const METRIC_SEARCH_LATENCY_GRAPH: &str = "search.latency.graph"
src/collection/query_builder/mod.rs:32  pub use builder::{HybridQueryBuilder, DEFAULT_RERANK_POOL_MAX, DEFAULT_RERANK_POOL_MULTIPLIER}
src/collection/query_builder/mod.rs:33  pub use scope::ScopeConstraint
src/collection/query_builder/mod.rs:34  pub use strategy::SearchStrategy
src/collection/query_builder/mod.rs:35  pub use weights::SignalWeights
src/collection/query_builder/scope.rs:20  pub struct ScopeConstraint
src/collection/query_builder/scope.rs:30  pub fn new(allowed_doc_ids: BTreeSet<DocId>, gamma: u32) -> Self
src/collection/query_builder/scope.rs:38  pub fn from_allowed_ids(allowed_doc_ids: BTreeSet<DocId>) -> Self
src/collection/query_builder/scope.rs:61  pub fn scope(mut self, constraint: ScopeConstraint) -> Self
src/collection/query_builder/strategy.rs:12  pub enum SearchStrategy
src/collection/query_builder/strategy.rs:35  pub fn to_graph_strategy(&self) -> GraphTraversalStrategy
src/collection/query_builder/strategy.rs:57  pub fn to_fusion_strategy(&self) -> FusionStrategy
src/collection/query_builder/strategy.rs:66  pub fn to_signal_fusion_strategies(&self) -> SignalFusionStrategies
src/collection/query_builder/weights.rs:12  pub struct SignalWeights
src/collection/query_builder/weights.rs:23  pub fn new(vector: f32, text: f32, graph: f32) -> Result<Self>
src/collection/search/checkpoint.rs:22  pub struct CheckpointPinGuard<'a, S: StorageEngine + ?Sized>
src/collection/search/filtered.rs:17  pub const OVERFETCH_STAGE_MULTIPLIERS: [usize; 3] = [3, 6, 12]
src/collection/search/filtered.rs:20  pub const MAX_OVERFETCH_SCAN: usize = contextra_types::MAX_SEARCH_K
src/collection/search/hybrid/mod.rs:38  pub const DEFAULT_MIN_RERANK_CANDIDATES: usize = 100
src/collection/search/hybrid/mod.rs:284  pub const OVERFETCH_FACTOR: usize = 3
src/collection/search/hybrid/mod.rs:287  pub const DEFAULT_COMMUNITY_BOOST: f32 = 1.2
src/collection/search/hybrid/mod.rs:385  pub fn filter_by_importance(
src/collection/search/mod.rs:25  pub use checkpoint::
src/collection/search/mod.rs:30  pub use crate::temporal_filter::
src/collection/tx.rs:19  pub fn next_tx(&self) -> Result<TxId>
src/collection/tx.rs:26  pub fn allocate_tx(&self) -> Result<TxId>
src/collection/tx.rs:38  pub fn begin_transaction(&self) -> Result<crate::transaction::DbTransaction<S, V>>
src/contextra_impl/collections.rs:232  pub fn allocate_tx(&self) -> Result<TxId>
src/contextra_impl/diagnostics.rs:64  pub fn orphan_registry(&self) -> &Arc<contextra_checkpoint::InstanceOrphanRegistry>
src/contextra_impl/lifecycle.rs:441  pub fn shutdown(&self)
src/contextra_impl/wiring.rs:13  pub fn set_metrics_sink(&self, sink: Arc<dyn contextra_ports::MetricsSink>)
src/contextra_impl/wiring.rs:18  pub fn metrics_sink(&self) -> Arc<dyn contextra_ports::MetricsSink>
src/contextra_impl/wiring.rs:24  pub fn set_router(&self, router: std::sync::Weak<dyn DriftStatusProvider>)
src/contextra_impl/wiring.rs:30  pub fn set_calibrator(
src/decay_controller.rs:23  pub struct DecaySignalInputs
src/decay_controller.rs:47  pub struct DecayControllerConfig
src/decay_controller.rs:71  pub struct AdaptiveDecayController
src/decay_controller.rs:88  pub fn new(config: DecayControllerConfig) -> Self
src/decay_controller.rs:96  pub fn with_clock(config: DecayControllerConfig, clock: Arc<dyn Clock>) -> Self
src/decay_controller.rs:105  pub fn with_defaults() -> Self
src/decay_controller.rs:110  pub fn set_clock(&mut self, clock: Arc<dyn Clock>)
src/decay_controller.rs:117  pub fn system_temperature(&self, inputs: &DecaySignalInputs) -> f32
src/decay_controller.rs:125  pub fn effective_half_life(&self, temperature: f32) -> f32
src/decay_controller.rs:131  pub fn effective_half_life_ms(&self, temperature: f32) -> f32
src/decay_controller.rs:138  pub fn effective_score(
src/decay_controller.rs:151  pub fn effective_score_wall_clock(
src/decay_controller.rs:172  pub fn should_evict(
src/decay_controller.rs:183  pub fn should_evict_wall_clock(
src/export.rs:21  pub const SCHEMA_VERSION_V1: &str = "1.0"
src/export.rs:25  pub struct ExportDocumentV1
src/export.rs:37  pub struct ExportCollectionV1
src/export.rs:48  pub struct ExportMemoryV1
src/export.rs:78  pub struct ExportRelationV1
src/extraction/mod.rs:17  pub mod open_ie
src/extraction/mod.rs:18  pub mod rule_based
src/extraction/mod.rs:20  pub mod tests
src/extraction/mod.rs:21  pub mod types
src/extraction/mod.rs:23  pub use open_ie::extract_triples
src/extraction/mod.rs:24  pub use rule_based::{RuleBasedExtractor, MAX_EXTRACTION_TEXT_LENGTH}
src/extraction/mod.rs:25  pub use types::{EntityExtractionConfig, ExtractedTriple, ExtractorMode}
src/extraction/mod.rs:31  pub trait EntityExtractor: Send + Sync
src/extraction/mod.rs:43  pub struct OpenIeExtractor
src/extraction/mod.rs:75  pub fn create_extractor(mode: ExtractorMode) -> Box<dyn EntityExtractor>
src/extraction/rule_based.rs:19  pub const MAX_EXTRACTION_TEXT_LENGTH: usize = 500_000
src/extraction/rule_based.rs:22  pub const MAX_EXTRACTED_TRIPLES: usize = 5_000
src/extraction/rule_based.rs:125  pub struct RuleBasedExtractor
src/extraction/rule_based.rs:129  pub fn new() -> Self
src/extraction/rule_based.rs:134  pub fn extract(
src/extraction/types.rs:21  pub struct ExtractedTriple
src/extraction/types.rs:36  pub struct EntityExtractionConfig
src/extraction/types.rs:57  pub enum ExtractorMode
src/filter.rs:20  pub const MAX_FILTER_DEPTH: usize = 16
src/filter.rs:23  pub const MAX_FILTER_NODES: usize = 128
src/filter.rs:26  pub fn validate_filter_expr_bounds(expr: &FilterExpr) -> Result<()>
src/filter.rs:68  pub fn extract_memory_type(metadata: &Option<Value>) -> MemoryType
src/filter.rs:84  pub enum FilterOp
src/filter.rs:113  pub enum MetadataFilter
src/filter.rs:132  pub fn validate_bounds(&self) -> Result<()>
src/filter.rs:176  pub fn matches(&self, metadata: &Value) -> bool
src/fusion.rs:5  pub use contextra_rank::fusion::
src/fusion.rs:13  pub use contextra_types::{FusionStrategy, SignalFusionStrategies}
src/fusion.rs:16  pub fn fuse_search_results_with_strategy(
src/fusion.rs:35  pub fn fuse_search_results_with_signal_strategies(
src/fusion.rs:54  pub fn fuse_signals(
src/import.rs:22  pub struct ImportSummary
src/import.rs:111  pub const MAX_IMPORT_COLLECTIONS: usize = 10_000
src/lib.rs:15  pub mod orchestration
src/lib.rs:21  pub use contextra_ports::TextEmbeddingEngine
src/lib.rs:22  pub use contextra_ports::VectorDeleteMode
src/lib.rs:24  pub use contextra_store::lsm::DurabilityMode
src/lib.rs:34  pub mod adapters
src/lib.rs:35  pub mod backends
src/lib.rs:37  pub mod background_workers
src/lib.rs:39  pub mod chunker
src/lib.rs:40  pub mod collection
src/lib.rs:42  pub mod decay_controller
src/lib.rs:44  pub mod export
src/lib.rs:47  pub mod extraction
src/lib.rs:49  pub mod filter
src/lib.rs:51  pub mod fusion
src/lib.rs:53  pub mod import
src/lib.rs:55  pub mod metrics_names
src/lib.rs:57  pub mod temporal_filter
src/lib.rs:59  pub mod transaction
src/lib.rs:62  pub use decay_controller::{AdaptiveDecayController, DecayControllerConfig, DecaySignalInputs}
src/lib.rs:65  pub use export::
src/lib.rs:69  pub use import::ImportSummary
src/lib.rs:73  pub use background_workers::start_decay_cleanup_worker
src/lib.rs:75  pub use background_workers::{start_expiry_cleanup_worker, start_orphan_cleanup_worker}
src/lib.rs:78  pub use backends::Backends
src/lib.rs:80  pub use backends::DefaultBackends
src/lib.rs:82  pub use collection::crud::MAX_SCAN_RESULTS
src/lib.rs:84  pub use collection::maintenance::PercolationConfig
src/lib.rs:87  pub use collection::maintenance::PercolationResult
src/lib.rs:89  pub use collection::query_builder::{HybridQueryBuilder, SearchStrategy, SignalWeights}
src/lib.rs:90  pub use collection::{Collection, CollectionConfig}
src/lib.rs:91  pub use contextra_checkpoint
src/lib.rs:92  pub use contextra_text::Language
src/lib.rs:95  pub use filter::MetadataFilter
src/lib.rs:101  pub type ConsolidationLauncher = Arc<
src/lib.rs:118  pub fn register_consolidation_launcher<F>(_f: F)
src/lib.rs:136  pub use fusion::{ProvenanceRecord, SearchResult, SignalContribution}
src/lib.rs:142  pub struct Document
src/lib.rs:150  pub struct ContextraStats
src/lib.rs:163  pub use contextra_ports::DriftStatusProvider
src/lib.rs:166  pub type DbStats = ContextraStats
src/lib.rs:171  pub struct CommunityDetectionConfig
src/lib.rs:186  pub enum EmbeddingBackend
src/lib.rs:205  pub enum TenantPolicy
src/lib.rs:217  pub struct ContextraConfig
src/lib.rs:264  pub fn with_consolidation_launcher(mut self, launcher: ConsolidationLauncher) -> Self
src/lib.rs:271  pub fn with_tenant_policy(mut self, policy: TenantPolicy) -> Self
src/lib.rs:278  pub fn with_metrics_sink(mut self, sink: Arc<dyn contextra_ports::MetricsSink>) -> Self
src/lib.rs:355  pub type TenantCollectionMap = ahash::AHashMap<
src/lib.rs:363  pub struct Contextra
src/lib.rs:385  pub mod contextra_impl
src/lib.rs:387  pub use contextra_types::DistanceMetric
src/lib.rs:388  pub use serde_json::json
src/lib.rs:395  pub fn inner_storage(&self) -> Arc<LsmStorage>
src/lib.rs:400  pub fn config(&self) -> &ContextraConfig
src/lib.rs:405  pub fn license_gate(&self) -> Arc<dyn LicenseGate>
src/lib.rs:413  pub trait SandboxBridge: Send + Sync
src/metrics_names.rs:6  pub const COMMIT_LATENCY: &str = "commit.latency"
src/metrics_names.rs:9  pub const REBUILD_DURATION: &str = "rebuild.duration"
src/metrics_names.rs:12  pub const BACKPRESSURE_LEVEL: &str = "backpressure.level"
src/metrics_names.rs:15  pub const CHECKPOINT_DURATION: &str = "checkpoint.duration"
src/metrics_names.rs:18  pub const SEARCH_LATENCY_VECTOR: &str = "search.latency.vector"
src/metrics_names.rs:21  pub const SEARCH_LATENCY_TEXT: &str = "search.latency.text"
src/metrics_names.rs:24  pub const SEARCH_LATENCY_GRAPH: &str = "search.latency.graph"
src/metrics_names.rs:27  pub const INSERT_LATENCY: &str = "insert.latency"
src/metrics_names.rs:30  pub const REAPER_DOCS_REAPED: &str = "reaper.docs_reaped"
src/metrics_names.rs:33  pub fn record_histogram_safe(
src/metrics_names.rs:45  pub fn record_gauge_safe(
src/metrics_names.rs:57  pub fn record_counter_safe(
src/orchestration/mod.rs:14  pub struct EngineServiceContainer<S, V, T>
src/orchestration/mod.rs:32  pub fn new(storage: Arc<S>, vector_index: Arc<V>, text_index: Arc<T>) -> Self
src/orchestration/mod.rs:41  pub fn storage(&self) -> &Arc<S>
src/orchestration/mod.rs:46  pub fn vector_index(&self) -> &Arc<V>
src/orchestration/mod.rs:51  pub fn text_index(&self) -> &Arc<T>
src/orchestration/mod.rs:58  pub struct WorkflowContext
src/orchestration/mod.rs:64  pub fn new() -> Self
src/orchestration/mod.rs:71  pub fn set(&mut self, key: impl Into<String>, val: impl Into<String>)
src/orchestration/mod.rs:76  pub fn get(&self, key: &str) -> Option<&str>
src/orchestration/mod.rs:82  pub trait WorkflowStep: Send + Sync
src/orchestration/mod.rs:94  pub struct Draft
src/orchestration/mod.rs:95  pub struct Prepared
src/orchestration/mod.rs:96  pub struct Executing
src/orchestration/mod.rs:97  pub struct Completed
src/orchestration/mod.rs:98  pub struct Failed
src/orchestration/mod.rs:101  pub struct WorkflowState<State>
src/orchestration/mod.rs:112  pub fn new(workflow_id: impl Into<String>) -> Self
src/orchestration/mod.rs:124  pub fn add_step(mut self, step: Box<dyn WorkflowStep>) -> Self
src/orchestration/mod.rs:130  pub fn prepare(self) -> Result<WorkflowState<Prepared>>
src/orchestration/mod.rs:149  pub fn start_execution(self) -> WorkflowState<Executing>
src/orchestration/mod.rs:193  pub fn workflow_id(&self) -> &str
src/orchestration/mod.rs:198  pub fn completed_steps(&self) -> usize
src/orchestration/mod.rs:203  pub fn context(&self) -> &WorkflowContext
src/orchestration/mod.rs:210  pub fn workflow_id(&self) -> &str
src/orchestration/mod.rs:215  pub fn completed_steps(&self) -> usize
src/orchestration/mod.rs:220  pub fn error(&self) -> Option<&ContextraError>
src/temporal_filter.rs:22  pub type FusionResult = contextra_rank::SearchResult
src/temporal_filter.rs:26  pub struct ValidityWindow
src/temporal_filter.rs:43  pub fn is_empty(&self) -> bool
src/temporal_filter.rs:53  pub fn is_valid_bitemporal(&self, as_of_tx: TxId, as_of_business: Option<u64>) -> bool
src/temporal_filter.rs:120  pub fn extract_validity_window(metadata: Option<&Value>) -> ValidityWindow
src/temporal_filter.rs:163  pub fn sort_candidates_deterministic(results: &mut [contextra_rank::SearchResult])
src/temporal_filter.rs:173  pub fn apply_temporal_validity_filter_with_clock(
src/temporal_filter.rs:187  pub fn apply_temporal_validity_filter(
src/temporal_filter.rs:207  pub fn apply_temporal_validity_filter_at(
src/transaction/compensating_actions.rs:15  pub trait CompensatingAction: Send + Sync
src/transaction/compensating_actions.rs:21  pub struct CommitLedger
src/transaction/compensating_actions.rs:28  pub fn new() -> Self
src/transaction/compensating_actions.rs:36  pub fn push<A: CompensatingAction + 'static>(&mut self, action: A)
src/transaction/compensating_actions.rs:53  pub struct CompensateLsmAction<S: StorageEngine, V: VectorIndex>
src/transaction/compensating_actions.rs:64  pub fn new(
src/transaction/compensating_actions.rs:197  pub struct CompensateGraphAction<S: StorageEngine, V: VectorIndex>
src/transaction/compensating_actions.rs:206  pub fn new(collection: Collection<S, V>, doc_ids: Arc<Vec<DocId>>) -> Self
src/transaction/compensating_actions.rs:241  pub struct CompensateHnswAction<S: StorageEngine, V: VectorIndex>
src/transaction/compensating_actions.rs:250  pub fn new(
src/transaction/compensating_actions.rs:322  pub struct CompensateTextAction<S: StorageEngine, V: VectorIndex>
src/transaction/compensating_actions.rs:330  pub fn new(collection: Collection<S, V>, doc_ids: Arc<Vec<DocId>>) -> Self
src/transaction/compensating_actions.rs:371  pub struct RollbackStagedAction<S: StorageEngine, V: VectorIndex>
src/transaction/compensating_actions.rs:379  pub fn new(collection: Collection<S, V>, tx_id: TxId) -> Self
src/transaction/db_transaction.rs:18  pub struct DbTransaction<S: StorageEngine, V: VectorIndex = contextra_vector::HnswIndex>
src/transaction/db_transaction.rs:37  pub fn new(collection: Collection<S, V>, tx_id: TxId) -> Self
src/transaction/db_transaction.rs:56  pub fn record_keys(&self, forward: Vec<u8>, reverse: Vec<u8>, doc_id: DocId)
src/transaction/db_transaction.rs:61  pub fn record_keys_with_old_values(
src/transaction/db_transaction.rs:90  pub fn stage_text_insert(&self, doc_id: DocId, text: String)
src/transaction/db_transaction.rs:100  pub fn stage_text_delete(&self, doc_id: DocId)
src/transaction/db_transaction.rs:110  pub fn stage_graph_entity(&self, entity: Entity)
src/transaction/db_transaction.rs:120  pub fn stage_graph_edge(&self, edge: Edge)
src/transaction/db_transaction.rs:130  pub fn stage_graph_entity_delete(&self, entity_id: EntityId)
src/transaction/db_transaction.rs:140  pub fn stage_graph_edge_delete(&self, from: EntityId, to: EntityId)
src/transaction/db_transaction.rs:150  pub fn stage_hyperedge(&self, hyperedge: contextra_graph::hyperedge::HyperEdge)
src/transaction/intent.rs:18  pub enum CommitIntent
src/transaction/mod.rs:89  pub use cleanup::{cleanup_orphaned_consolidation_intents, recover_pending_intents}
src/transaction/mod.rs:90  pub use compensating_actions::
src/transaction/mod.rs:94  pub use db_transaction::DbTransaction
src/transaction/mod.rs:95  pub use intent::CommitIntent
```

## Innen: crate-interne Grenzen (`pub(crate)`, `pub(super)`, `pub(in …)`)

```rust
src/chunker/blocks.rs:19  pub(crate) struct CodeFence
src/chunker/blocks.rs:20  pub(crate) char_type: char,
src/chunker/blocks.rs:21  pub(crate) length: usize,
src/chunker/blocks.rs:26  pub(crate) fn parse_opening_fence(line: &str) -> Option<CodeFence>
src/chunker/blocks.rs:50  pub(crate) fn is_closing_fence(line: &str, active_fence: &CodeFence) -> bool
src/chunker/blocks.rs:62  pub(crate) enum MarkdownBlock
src/chunker/blocks.rs:80  pub(crate) fn to_text(&self) -> String
src/chunker/blocks.rs:113  pub(crate) fn estimate_tokens(&self) -> usize
src/chunker/blocks.rs:120  pub(crate) fn parse_markdown_blocks(lines: &[String]) -> Vec<MarkdownBlock>
src/chunker/blocks.rs:216  pub(crate) fn is_table_row(line: &str) -> bool
src/chunker/blocks.rs:226  pub(crate) fn is_table_delimiter_row(line: &str) -> bool
src/chunker/estimate.rs:17  pub(crate) const CHARS_PER_TOKEN: usize = 4
src/chunker/sections.rs:17  pub(crate) struct RawSection
src/chunker/sections.rs:18  pub(crate) lines: Vec<String>,
src/chunker/sections.rs:19  pub(crate) breadcrumb: String,
src/chunker/sections.rs:20  pub(crate) heading_level: u8,
src/chunker/sections.rs:21  pub(crate) source_line: usize,
src/chunker/sections.rs:22  pub(crate) source_line_end: usize,
src/chunker/sections.rs:23  pub(crate) start_byte: usize,
src/chunker/sections.rs:24  pub(crate) end_byte: usize,
src/chunker/sections.rs:25  pub(crate) tokens: usize,
src/chunker/split.rs:21  pub(crate) fn split_raw_section(
src/collection/crud/auto_extraction.rs:240  pub(crate) async fn auto_extract_and_relate<S: StorageEngine, V: VectorIndex>(
src/collection/crud/insert.rs:191  pub(crate) async fn insert_inner_unlocked(
src/collection/crud/insert.rs:251  pub(crate) async fn check_doc_id_collision(&self, doc_id: DocId, id: &str) -> Result<()>
src/collection/crud/insert.rs:276  pub(crate) async fn check_doc_id_collision_tracked(
src/collection/crud/internal.rs:14  pub(crate) fn validate_doc_id(id: &str) -> Result<()>
src/collection/crud/internal.rs:35  pub(crate) fn validate_embedding(embedding: &[f32]) -> Result<()>
src/collection/crud/internal.rs:56  pub(crate) async fn apply_insert_backpressure(&self)
src/collection/crud/internal.rs:98  pub(crate) async fn lock_keys_sorted<'a>(
src/collection/crud/mod.rs:27  pub(super) use internal::validate_doc_id
src/collection/kv_lock.rs:19  pub(crate) struct KeyGuard<'a>
src/collection/kv_lock.rs:25  pub(crate) struct KvKeyLocks
src/collection/mod.rs:232  pub(super) fn extract_text(metadata: &Option<serde_json::Value>) -> Option<String>
src/collection/mod.rs:274  pub(super) name: String,
src/collection/mod.rs:275  pub(super) prefix: Vec<u8>,
src/collection/mod.rs:276  pub(super) index: Arc<V>,
src/collection/mod.rs:277  pub(super) text_index: InvertedIndex<S>,
src/collection/mod.rs:278  pub(super) graph_index: Arc<CsrGraph>,
src/collection/mod.rs:279  pub(super) storage: Arc<S>,
src/collection/mod.rs:280  pub(super) next_tx: Arc<AtomicU64>,
src/collection/mod.rs:281  pub(super) dimension: usize,
src/collection/mod.rs:282  pub(super) embedder: parking_lot::RwLock<Option<Arc<dyn TextEmbeddingEngine>>>,
src/collection/mod.rs:284  pub(super) kv_locks: Arc<kv_lock::KvKeyLocks>,
src/collection/mod.rs:286  pub(super) mutations_since_community_detection: Arc<AtomicU64>,
src/collection/mod.rs:288  pub(super) community_detection_trigger_threshold: Arc<AtomicU64>,
src/collection/mod.rs:290  pub(super) consolidation_in_progress: Arc<AtomicBool>,
src/collection/mod.rs:292  pub(super) config: parking_lot::RwLock<CollectionConfig>,
src/collection/mod.rs:294  pub(super) pressure_rx:
src/collection/mod.rs:297  pub(super) clock: parking_lot::RwLock<Arc<dyn Clock>>,
src/collection/mod.rs:299  pub(super) metrics: parking_lot::RwLock<Arc<dyn MetricsSink>>,
src/collection/mod.rs:685  pub(super) fn check_and_trigger_community_detection(&self, count: usize)
src/collection/query_builder/builder.rs:32  pub(super) collection: &'a Collection<S, V>,
src/collection/query_builder/builder.rs:33  pub(super) text: Option<String>,
src/collection/query_builder/builder.rs:34  pub(super) vector: Option<Vec<f32>>,
src/collection/query_builder/builder.rs:35  pub(super) k: Option<usize>,
src/collection/query_builder/builder.rs:36  pub(super) weights: Option<FusionWeights>,
src/collection/query_builder/builder.rs:37  pub(super) fusion_strategy: Option<SignalFusionStrategies>,
src/collection/query_builder/builder.rs:38  pub(super) strategy: Option<SearchStrategy>,
src/collection/query_builder/builder.rs:39  pub(super) filter: Option<FilterExpr>,
src/collection/query_builder/builder.rs:40  pub(super) anchor_entities: Option<Vec<EntityId>>,
src/collection/query_builder/builder.rs:41  pub(super) same_community_as: Option<EntityId>,
src/collection/query_builder/builder.rs:42  pub(super) memory_type_filter: Option<Vec<MemoryType>>,
src/collection/query_builder/builder.rs:43  pub(super) include_superseded: bool,
src/collection/query_builder/builder.rs:44  pub(super) include_provenance: bool,
src/collection/query_builder/builder.rs:45  pub(super) filter_fn: Option<Box<dyn Fn(DocId) -> bool + Send + Sync>>,
src/collection/query_builder/builder.rs:46  pub(super) hard_scope: Option<super::scope::ScopeConstraint>,
src/collection/query_builder/builder.rs:47  pub(super) reranker: Option<&'a (dyn Reranker + 'a)>,
src/collection/query_builder/builder.rs:48  pub(super) rerank_pool_multiplier: Option<usize>,
src/collection/query_builder/builder.rs:49  pub(super) rerank_pool_max: Option<usize>,
src/collection/query_builder/builder.rs:50  pub(super) on_signal_failure: SignalFailurePolicy,
src/collection/query_builder/builder.rs:52  pub(super) pid_controller: Option<Arc<parking_lot::Mutex<contextra_adapt::PidController>>>,
src/collection/query_builder/builder.rs:53  pub(super) seq: Option<u64>,
src/collection/query_builder/builder.rs:54  pub(super) as_of_timestamp: Option<u64>,
src/collection/query_builder/builder.rs:55  pub(super) query_timestamp: Option<u64>,
src/collection/query_builder/builder.rs:56  pub(super) current_tx: Option<contextra_types::TxId>,
src/collection/query_builder/builder.rs:57  pub(super) metrics: Option<Arc<dyn MetricsSink>>,
src/collection/search/filtered.rs:27  pub(super) async fn estimate_filter_selectivity(
src/collection/search/filtered.rs:87  pub(super) async fn get_matching_doc_ids_at(
src/collection/search/filtered.rs:123  pub(super) async fn get_matching_doc_ids_for_query_at(
src/collection/search/hybrid/mod.rs:294  pub(super) async fn apply_community_boost_post_fusion(
src/collection/search/hybrid/query.rs:24  pub(super) fn calc_hybrid_candidate_k(query: &contextra_types::HybridQuery, k: usize) -> usize
src/collection/search/hydrate.rs:18  pub(super) async fn hydrate_from_scored_at(
src/collection/search/hydrate.rs:67  pub(super) async fn hydrate_from_tuples_at(
src/contextra_impl/crud.rs:17  pub(crate) async fn default_col(&self) -> Result<Arc<Collection<LsmStorage>>>
src/transaction/compensating_actions.rs:54  pub(super) collection: Collection<S, V>,
src/transaction/compensating_actions.rs:55  pub(super) intent_key: Vec<u8>,
src/transaction/compensating_actions.rs:56  pub(super) doc_ids: Arc<Vec<DocId>>,
src/transaction/compensating_actions.rs:57  pub(super) f_keys: Vec<StagedKeyOp>,
src/transaction/compensating_actions.rs:58  pub(super) r_keys: Vec<StagedKeyOp>,
src/transaction/compensating_actions.rs:198  pub(super) collection: Collection<S, V>,
src/transaction/compensating_actions.rs:199  pub(super) doc_ids: Arc<Vec<DocId>>,
src/transaction/compensating_actions.rs:242  pub(super) collection: Collection<S, V>,
src/transaction/compensating_actions.rs:243  pub(super) tenant_id: TenantId,
src/transaction/compensating_actions.rs:244  pub(super) doc_ids: Arc<Vec<DocId>>,
src/transaction/compensating_actions.rs:323  pub(super) collection: Collection<S, V>,
src/transaction/compensating_actions.rs:324  pub(super) doc_ids: Arc<Vec<DocId>>,
src/transaction/compensating_actions.rs:372  pub(super) collection: Collection<S, V>,
src/transaction/compensating_actions.rs:373  pub(super) tx_id: TxId,
src/transaction/db_transaction.rs:20  pub(super) collection: Collection<S, V>,
src/transaction/db_transaction.rs:21  pub(super) staged_forward_keys: Mutex<Vec<StagedKeyOp>>,
src/transaction/db_transaction.rs:22  pub(super) staged_reverse_keys: Mutex<Vec<StagedKeyOp>>,
src/transaction/db_transaction.rs:23  pub(super) staged_doc_ids: Mutex<Arc<Vec<DocId>>>,
src/transaction/db_transaction.rs:24  pub(super) staged_text_ops: Mutex<Vec<(DocId, String)>>,
src/transaction/db_transaction.rs:25  pub(super) staged_text_deletes: Mutex<Vec<DocId>>,
src/transaction/db_transaction.rs:26  pub(super) staged_graph_entities: Mutex<Vec<Entity>>,
src/transaction/db_transaction.rs:27  pub(super) staged_graph_edges: Mutex<Vec<Edge>>,
src/transaction/db_transaction.rs:28  pub(super) staged_graph_entity_deletes: Mutex<Vec<EntityId>>,
src/transaction/db_transaction.rs:29  pub(super) staged_graph_edge_deletes: Mutex<Vec<(EntityId, EntityId)>>,
src/transaction/db_transaction.rs:30  pub(super) staged_hyperedges: Mutex<Vec<contextra_graph::hyperedge::HyperEdge>>,
src/transaction/db_transaction.rs:31  pub(super) committed: AtomicBool,
src/transaction/db_transaction_commit.rs:23  pub(super) async fn commit_text_staged(&self) -> Result<()>
src/transaction/db_transaction_commit.rs:58  pub(super) async fn commit_graph_staged(&self) -> Result<()>
src/transaction/db_transaction_rollback.rs:18  pub(super) async fn compensate_hnsw(&self, doc_ids: &[DocId])
src/transaction/db_transaction_rollback.rs:40  pub(super) async fn compensate_text(&self, doc_ids: &[DocId])
src/transaction/db_transaction_rollback.rs:67  pub(super) async fn compensate_lsm(&self, intent_key: &[u8], doc_ids: &[DocId])
src/transaction/db_transaction_rollback.rs:183  pub(super) async fn rollback_internal(&self)
src/transaction/intent.rs:14  pub(super) type StagedKeyOp = (Vec<u8>, Option<Bytes>)
```
