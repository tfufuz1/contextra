# API: contextra-vector

> Erzeugt aus dem Quellcode am 2026-11-25, Repository HEAD ce4579a56. Eine Zeile je Item: Datei:Zeile und die **erste Zeile** der Signatur. Mehrzeilige Signaturen sind nur angeschnitten. Nicht kompiliert, daher ohne Typprüfung. Testcode (`tests`, `testing`, `test_doubles`) ist ausgeschlossen.

Umfang: 425 öffentliche Items, 137 crate-interne Sichtbarkeitsgrenzen.

## Innen: Abhängigkeiten zu anderen Workspace-Crates

- contextra-ports
- contextra-sys
- contextra-types

## Außen: öffentliche Schnittstelle (`pub`)

```rust
src/acorn/gamma_augmentation.rs:29  pub fn compute_gamma_edge_budget(base_degree: usize, predicate_selectivity: f32) -> usize
src/acorn/mod.rs:5  pub mod gamma_augmentation
src/acorn/mod.rs:6  pub mod naive_reference
src/acorn/mod.rs:8  pub use gamma_augmentation::compute_gamma_edge_budget
src/acorn/mod.rs:9  pub use naive_reference::NaiveReferenceIndex
src/acorn/mod.rs:16  pub trait FilteredIndex
src/acorn/mod.rs:30  pub enum AcornError
src/acorn/naive_reference.rs:20  pub struct NaiveReferenceIndex
src/acorn/naive_reference.rs:27  pub fn new(metric: DistanceMetric) -> Self
src/acorn/naive_reference.rs:35  pub fn from_vectors(vectors: Vec<(DocId, Vec<f32>)>) -> Self
src/acorn/naive_reference.rs:43  pub fn insert(&mut self, id: DocId, vector: Vec<f32>)
src/candidate_stream.rs:11  pub const DEFAULT_VECTOR_STREAM_BATCH_SIZE: usize = 16
src/candidate_stream.rs:40  pub struct VectorCandidateStream<'a, V: VectorIndex>
src/candidate_stream.rs:54  pub fn new(index: &'a V, query: impl Into<Vec<f32>>, seq_no: Option<u64>) -> Self
src/candidate_stream.rs:69  pub fn with_batch_size(mut self, n: usize) -> Self
src/candidate_stream.rs:108  pub fn yielded(&self) -> usize
src/candidate_stream.rs:113  pub fn fetched_depth(&self) -> usize
src/candidate_stream.rs:118  pub fn is_exhausted(&self) -> bool
src/candidate_stream.rs:123  pub const fn max_depth() -> usize
src/casting.rs:9  pub fn to_u32(val: usize) -> Result<u32>
src/casting.rs:17  pub fn u32_to_usize(val: u32) -> usize
src/casting.rs:23  pub fn u64_to_usize(val: u64) -> Result<usize>
src/casting.rs:31  pub fn u64_to_u32(val: u64) -> Result<u32>
src/casting.rs:39  pub fn to_u8(val: usize) -> Result<u8>
src/casting.rs:47  pub fn to_u64(val: usize) -> u64
src/casting.rs:53  pub fn f32_to_usize(val: f32) -> Result<usize>
src/casting.rs:65  pub fn f64_to_usize(val: f64) -> Result<usize>
src/compute_pool.rs:14  pub struct TaskHandle<T>
src/compute_pool.rs:20  pub fn join(self) -> Result<T, std::sync::mpsc::RecvError>
src/compute_pool.rs:37  pub struct ComputePool
src/compute_pool.rs:43  pub fn new(max_workers: usize) -> Self
src/compute_pool.rs:105  pub fn max_workers(&self) -> usize
src/compute_pool.rs:110  pub fn active_jobs(&self) -> usize
src/compute_pool.rs:115  pub fn execute<F>(&self, f: F)
src/compute_pool.rs:126  pub fn spawn<F, T>(&self, f: F) -> TaskHandle<T>
src/contract.rs:8  pub enum Error
src/contract.rs:36  pub type Result<T> = core::result::Result<T, Error>
src/contract.rs:39  pub trait Clock: Send + Sync
src/contract.rs:45  pub trait Rng: Send
src/contract.rs:52  pub struct ExternalId(pub u128); // 128 Bit erhält vorhandene Fähigkeit (E-05); From<u64> vorhanden
src/contract.rs:62  pub enum Metric
src/contract.rs:73  pub struct ScoredPoint
src/contract.rs:81  pub trait DistanceFn: Send + Sync
src/contract.rs:87  pub trait Index<Id, Query>
src/contract.rs:102  pub trait Maintain
src/contract.rs:114  pub trait BlockTransform: Send + Sync
src/diskann/build.rs:432  pub fn recover_pending_delta(
src/diskann/build.rs:438  pub fn recover_pending_delta_sync(&self) -> Result<usize>
src/diskann/build.rs:632  pub fn build_to_path_sync(
src/diskann/build.rs:718  pub fn build_sync(&self, vectors: &[Vec<f32>], ids: &[DocId]) -> Result<()>
src/diskann/config.rs:12  pub enum DiskAnnFallbackPolicy
src/diskann/config.rs:22  pub struct DiskAnnConfig
src/diskann/integrity.rs:6  pub struct DiskAnnHmac
src/diskann/integrity.rs:12  pub fn new(integrity_key: &[u8]) -> Result<Self>
src/diskann/integrity.rs:37  pub fn update(&mut self, data: &[u8])
src/diskann/integrity.rs:42  pub fn finalize(self) -> [u8; 32]
src/diskann/mod.rs:14  pub mod predicate_augmented
src/diskann/mod.rs:22  pub use config::{DiskAnnConfig, DiskAnnFallbackPolicy}
src/diskann/mod.rs:23  pub use types::DiskAnnIndex
src/diskann/persistence.rs:18  pub fn trigger_background_persist_delta(&self)
src/diskann/persistence.rs:40  pub fn persist_delta_sync(&self) -> Result<()>
src/diskann/persistence.rs:249  pub fn load_sync(&self) -> Result<()>
src/diskann/types.rs:25  pub struct DiskAnnIndex
src/diskann/types.rs:186  pub fn try_new(config: DiskAnnConfig) -> Result<Self>
src/diskann/types.rs:281  pub fn len_sync(&self) -> usize
src/distance.rs:13  pub mod contract
src/distance.rs:20  pub struct CosineSimilarityPartsU8
src/distance.rs:30  pub trait DistanceKernel: Send + Sync + 'static
src/distance.rs:84  pub struct ScalarKernel
src/distance.rs:365  pub struct ScalarDistance
src/distance.rs:370  pub const fn new(metric: ContractMetric) -> Self
src/distance.rs:375  pub fn compute(&self, a: &[f32], b: &[f32]) -> core::result::Result<f32, ContractError>
src/distance.rs:428  pub struct InjectedDistanceKernel<D: DistanceFn>
src/distance.rs:433  pub fn new(distance_fn: D) -> Self
src/distance.rs:528  pub fn validate_vector(vec: &[f32]) -> Result<()>
src/distance.rs:538  pub fn compute_distance(a: &[f32], b: &[f32], metric: DistanceMetric) -> Result<f32>
src/distance.rs:543  pub fn compute_distance_trusted(a: &[f32], b: &[f32], metric: DistanceMetric) -> Result<f32>
src/distance.rs:548  pub fn compute_distance_f32_bytes_trusted(
src/distance.rs:557  pub fn cosine_distance(a: &[f32], b: &[f32]) -> Result<f32>
src/distance.rs:562  pub fn dot_product_distance(a: &[f32], b: &[f32]) -> Result<f32>
src/distance.rs:567  pub fn euclidean_distance(a: &[f32], b: &[f32]) -> Result<f32>
src/distance.rs:572  pub fn euclidean_distance_sq_u8(a: &[u8], b: &[u8]) -> u32
src/distance.rs:577  pub fn euclidean_distance_sq_f32_u8(a: &[f32], b: &[u8], alphas: &[f32], mins: &[f32]) -> f32
src/distance.rs:582  pub fn dot_product_u8(a: &[u8], b: &[u8]) -> u32
src/distance.rs:587  pub fn dot_product_f32_u8(a: &[f32], b: &[u8]) -> f32
src/distance.rs:592  pub fn cosine_similarity_parts_u8(a: &[u8], b: &[u8]) -> CosineSimilarityPartsU8
src/distance.rs:597  pub fn normalize_inplace(v: &mut [f32])
src/distance.rs:602  pub fn cosine_distance_scalar(a: &[f32], b: &[f32]) -> f32
src/distance.rs:607  pub fn dot_product_scalar(a: &[f32], b: &[f32]) -> f32
src/distance.rs:612  pub fn euclidean_distance_scalar(a: &[f32], b: &[f32]) -> f32
src/error.rs:10  pub enum VectorError
src/error.rs:53  pub type Result<T> = std::result::Result<T, VectorError>
src/fixed_hasher.rs:14  pub fn fixed_hasher() -> RandomState
src/fixed_hasher.rs:19  pub type FixedHashSet<T> = HashSet<T, RandomState>
src/fixed_hasher.rs:22  pub type FixedHashMap<K, V> = HashMap<K, V, RandomState>
src/fixed_hasher.rs:25  pub fn new_fixed_hashset<T>(capacity: usize) -> FixedHashSet<T>
src/fixed_hasher.rs:30  pub fn new_fixed_hashmap<K, V>(capacity: usize) -> FixedHashMap<K, V>
src/hnsw/adaptive_ef.rs:10  pub struct AdaptiveEfPolicy
src/hnsw/adaptive_ef.rs:25  pub fn new(
src/hnsw/adaptive_ef.rs:67  pub fn validate(&self) -> Result<()>
src/hnsw/adaptive_ef.rs:112  pub struct AdaptiveEfStats
src/hnsw/adaptive_ef.rs:123  pub struct AdaptiveEfStateMachine
src/hnsw/adaptive_ef.rs:136  pub fn new(policy: AdaptiveEfPolicy, k: usize) -> Self
src/hnsw/adaptive_ef.rs:151  pub fn current_ef(&self) -> usize
src/hnsw/adaptive_ef.rs:156  pub fn round_count(&self) -> usize
src/hnsw/adaptive_ef.rs:161  pub fn is_terminated(&self) -> bool
src/hnsw/adaptive_ef.rs:166  pub fn stats(&self) -> AdaptiveEfStats
src/hnsw/adaptive_ef.rs:178  pub fn step(&mut self, current_top_k: &[DocId]) -> bool
src/hnsw/arena.rs:13  pub const ARENA_ALIGNMENT_BYTES: usize = 64
src/hnsw/arena.rs:15  pub const ARENA_ALIGNMENT_U32: usize = ARENA_ALIGNMENT_BYTES / std::mem::size_of::<u32>()
src/hnsw/arena.rs:18  pub type BacklinkKey = (usize, usize)
src/hnsw/arena.rs:22  pub struct BacklinkTable
src/hnsw/arena.rs:28  pub fn new() -> Self
src/hnsw/arena.rs:36  pub fn get(&self, neighbor_ram_idx: usize, layer: usize) -> Option<&Vec<u32>>
src/hnsw/arena.rs:42  pub fn insert(&mut self, neighbor_ram_idx: usize, layer: usize, connections: Vec<u32>)
src/hnsw/arena.rs:47  pub fn clear(&mut self)
src/hnsw/arena.rs:54  pub struct HnswArena
src/hnsw/arena.rs:72  pub fn new() -> Self
src/hnsw/arena.rs:86  pub fn layer_offset(node_offset: usize, layer: usize, m: usize) -> usize
src/hnsw/arena.rs:96  pub fn align_capacity(raw_capacity: usize) -> usize
src/hnsw/arena.rs:102  pub fn allocate_node(
src/hnsw/arena.rs:183  pub fn get_ram_node_connections(&self, ram_idx: usize, layer: usize, m: usize) -> Vec<u32>
src/hnsw/arena.rs:220  pub fn update_backlink(
src/hnsw/arena.rs:260  pub fn try_relink_pruned_neighbors(
src/hnsw/arena.rs:298  pub fn zeroize_node_connections(&self, ram_idx: usize, max_layer: usize, m: usize)
src/hnsw/arena.rs:329  pub fn zeroize_and_free_node(&self, ram_idx: usize, max_layer: usize, m: usize)
src/hnsw/arena.rs:335  pub fn free_node(&self, ram_idx: usize)
src/hnsw/batch.rs:9  pub struct PreparedInsert
src/hnsw/batch.rs:22  pub struct NeighborBacklink
src/hnsw/batch.rs:30  pub struct BatchContext
src/hnsw/batch.rs:38  pub fn new(core: &HnswIndexCore) -> Self
src/hnsw/clock.rs:10  pub struct MonotonicCounterClock
src/hnsw/clock.rs:16  pub fn new(initial_nanos: u64) -> Self
src/hnsw/clock.rs:23  pub fn advance_nanos(&self, delta_nanos: u64)
src/hnsw/config.rs:6  pub struct HnswConfig
src/hnsw/config.rs:70  pub fn validate(&self) -> Result<()>
src/hnsw/config.rs:108  pub struct HnswConfigBuilder
src/hnsw/config.rs:114  pub fn new(dimension: usize) -> Self
src/hnsw/config.rs:124  pub fn max_elements(mut self, max: usize) -> Self
src/hnsw/config.rs:130  pub fn m(mut self, m: usize) -> Self
src/hnsw/config.rs:136  pub fn ef_construction(mut self, ef: usize) -> Self
src/hnsw/config.rs:142  pub fn ef_search(mut self, ef: usize) -> Self
src/hnsw/config.rs:148  pub fn distance_metric(mut self, metric: DistanceMetric) -> Self
src/hnsw/config.rs:154  pub fn quantize(mut self, quantize: bool) -> Self
src/hnsw/config.rs:160  pub fn quantizer_recalibration_sample_size(mut self, size: usize) -> Self
src/hnsw/config.rs:166  pub fn quantizer_drift_threshold(mut self, threshold: f32) -> Self
src/hnsw/config.rs:172  pub fn rebuild_threshold(mut self, threshold: f64) -> Self
src/hnsw/config.rs:178  pub fn backoff_base_delay(mut self, delay: std::time::Duration) -> Self
src/hnsw/config.rs:184  pub fn backoff_max_delay(mut self, delay: std::time::Duration) -> Self
src/hnsw/config.rs:190  pub fn backoff_alert_threshold(mut self, threshold: u32) -> Self
src/hnsw/config.rs:197  pub fn partial_rebuild_config(
src/hnsw/config.rs:206  pub fn build(self) -> Result<HnswConfig>
src/hnsw/core_insert.rs:60  pub fn compute_insert(&self, id: DocId, vector: &[f32]) -> Result<PreparedInsert>
src/hnsw/core_insert.rs:65  pub fn compute_insert_with_context(
src/hnsw/core_insert.rs:366  pub fn apply_insert(&self, prepared: PreparedInsert, tx_id: u64)
src/hnsw/core_insert.rs:420  pub fn connectivity_score(&self) -> f64
src/hnsw/core_insert.rs:444  pub fn check_connectivity(&self) -> contextra_types::Result<()>
src/hnsw/core_rebuild.rs:26  pub fn rebuild_status(&self) -> RebuildStatus
src/hnsw/core_rebuild.rs:34  pub fn wait_for_rebuild_blocking(
src/hnsw/core_rebuild.rs:52  pub fn wait_for_rebuild(&self) -> bool
src/hnsw/core_rebuild.rs:57  pub fn wait_for_rebuild_with_clock(&self, clock: &dyn contextra_ports::Clock) -> bool
src/hnsw/core_rebuild.rs:61  pub fn wait_for_rebuild_with_timeout(&self, timeout: std::time::Duration) -> bool
src/hnsw/core_rebuild.rs:66  pub fn wait_for_rebuild_with_timeout_and_clock(
src/hnsw/core_rebuild.rs:486  pub fn deleted_ratio(&self) -> f64
src/hnsw/core_rebuild.rs:490  pub fn rebuild_count(&self) -> u64
src/hnsw/core_rebuild.rs:494  pub fn visited_dead_nodes(&self) -> u64
src/hnsw/core_rebuild.rs:498  pub fn is_rebuild_required(&self) -> bool
src/hnsw/core_rebuild.rs:512  pub fn rebuild(&self) -> Result<()>
src/hnsw/core_rebuild.rs:516  pub fn rebuild_with_clock(&self, clock: &dyn contextra_ports::Clock) -> Result<()>
src/hnsw/core_rebuild.rs:520  pub fn rebuild_sync(&self) -> Result<()>
src/hnsw/core_rebuild.rs:525  pub fn rebuild_sync_with_clock(&self, clock: &dyn contextra_ports::Clock) -> Result<()>
src/hnsw/core_rebuild.rs:633  pub fn rebuild_region(&self, region_node_ids: Vec<u64>) -> Result<()>
src/hnsw/core_rebuild.rs:637  pub fn rebuild_region_sync(&self, region_node_ids: Vec<u64>) -> Result<()>
src/hnsw/core_rebuild.rs:988  pub fn verify_full_connectivity(&self) -> bool
src/hnsw/deletion.rs:16  pub struct DeletionStats
src/hnsw/deletion.rs:28  pub trait GhostFreeVectorIndex
src/hnsw/deletion.rs:387  pub fn remove(&self, doc_id: DocId) -> Result<bool>
src/hnsw/mod.rs:18  pub mod adaptive_ef
src/hnsw/mod.rs:19  pub mod arena
src/hnsw/mod.rs:20  pub mod clock
src/hnsw/mod.rs:21  pub mod deletion
src/hnsw/mod.rs:22  pub mod purge
src/hnsw/mod.rs:23  pub mod scanner
src/hnsw/mod.rs:24  pub mod sq8_bias
src/hnsw/mod.rs:27  pub use purge::{hnsw_purger, HnswPurger}
src/hnsw/mod.rs:29  pub use scanner::{hnsw_scanner, HnswScanner}
src/hnsw/mod.rs:37  pub mod staging
src/hnsw/mod.rs:44  pub use adaptive_ef::{AdaptiveEfPolicy, AdaptiveEfStateMachine, AdaptiveEfStats}
src/hnsw/mod.rs:45  pub use arena::{BacklinkTable, HnswArena}
src/hnsw/mod.rs:46  pub use batch::{BatchContext, NeighborBacklink, PreparedInsert}
src/hnsw/mod.rs:47  pub use clock::MonotonicCounterClock
src/hnsw/mod.rs:48  pub use config::{HnswConfig, HnswConfigBuilder}
src/hnsw/mod.rs:49  pub use deletion::{DeletionStats, GhostFreeVectorIndex}
src/hnsw/mod.rs:50  pub use sq8_bias::Sq8Bias
src/hnsw/mod.rs:51  pub use types::
src/hnsw/mod.rs:56  pub use vector_index_impl::{HnswCompactStats, Index, Maintain}
src/hnsw/mod.rs:58  pub use staging::{VectorStagingOp, VectorTxBuffer}
src/hnsw/purge.rs:21  pub struct HnswPurger
src/hnsw/purge.rs:28  pub fn new(index: Arc<HnswIndex>) -> Self
src/hnsw/purge.rs:38  pub fn hnsw_purger(index: Arc<HnswIndex>) -> Arc<dyn Purger>
src/hnsw/scanner.rs:20  pub struct HnswScanner
src/hnsw/scanner.rs:27  pub fn new(index: Arc<HnswIndex>) -> Self
src/hnsw/scanner.rs:205  pub fn hnsw_scanner(index: Arc<HnswIndex>) -> Arc<dyn LayerScanner>
src/hnsw/scanner.rs:212  pub fn scan_units(&self, session: &mut ScanSession, _ctx: &RequestContext) -> Result<()>
src/hnsw/scanner.rs:239  pub struct ResidualReport
src/hnsw/scanner.rs:252  pub fn has_residual(&self) -> bool
src/hnsw/scanner.rs:262  pub fn inspect_residual(&self, doc_id: DocId) -> ResidualReport
src/hnsw/sq8_bias.rs:11  pub struct Sq8Bias
src/hnsw/sq8_bias.rs:32  pub fn new(mean_bias: f32, variance_bias: f32, sample_count: u64) -> Self
src/hnsw/sq8_bias.rs:41  pub fn calibrate(
src/hnsw/staging.rs:12  pub enum VectorStagingOp
src/hnsw/staging.rs:36  pub struct VectorTxBuffer
src/hnsw/staging.rs:44  pub fn new(capacity: usize, timeout: Duration) -> Self
src/hnsw/staging.rs:53  pub fn stage(&self, tx: TxId, op: VectorStagingOp) -> contextra_types::Result<()>
src/hnsw/staging.rs:84  pub fn drain(&self, tx: TxId) -> Vec<VectorStagingOp>
src/hnsw/staging.rs:103  pub fn discard(&self, tx: TxId)
src/hnsw/staging.rs:110  pub enum SeqLogChange
src/hnsw/staging.rs:117  pub struct SequenceLog
src/hnsw/staging.rs:124  pub fn new() -> Self
src/hnsw/staging.rs:129  pub fn record_insert(&mut self, doc_id: DocId, seq: u64)
src/hnsw/staging.rs:141  pub fn record_delete(&mut self, doc_id: DocId, seq: u64)
src/hnsw/staging.rs:153  pub fn is_visible(&self, doc_id: DocId, snap_seq: u64) -> bool
src/hnsw/staging.rs:169  pub fn changes_since(&self, since_seq: u64) -> Vec<SeqLogChange>
src/hnsw/types.rs:14  pub const SENTINEL_NO_ENTRY_POINT: u32 = u32::MAX
src/hnsw/types.rs:17  pub const HNSW_REBUILD_DELETION_RATIO: f64 = 0.10
src/hnsw/types.rs:20  pub const DEFAULT_LAYER_SEED: u64 = 100
src/hnsw/types.rs:23  pub enum VectorData
src/hnsw/types.rs:32  pub struct HnswNode
src/hnsw/types.rs:41  pub struct Candidate
src/hnsw/types.rs:65  pub struct RebuildGuard<'a>(pub &'a AtomicBool, pub &'a parking_lot::Condvar)
src/hnsw/types.rs:75  pub struct SnapshotPinGuard<'a>(pub &'a HnswIndexCore, u64)
src/hnsw/types.rs:91  pub struct GenericHnswIndex<K: DistanceKernel = ScalarKernel>
src/hnsw/types.rs:97  pub type HnswIndex = GenericHnswIndex<ScalarKernel>
src/hnsw/types.rs:100  pub struct HnswHotCore
src/hnsw/types.rs:122  pub fn set_layer_seed(&self, seed: u64)
src/hnsw/types.rs:128  pub fn metrics_sink(&self) -> std::sync::Arc<dyn contextra_ports::MetricsSink>
src/hnsw/types.rs:133  pub fn set_metrics_sink(&self, sink: std::sync::Arc<dyn contextra_ports::MetricsSink>)
src/hnsw/types.rs:138  pub fn clock(&self) -> std::sync::Arc<dyn contextra_ports::Clock>
src/hnsw/types.rs:143  pub fn set_clock(&self, clock: std::sync::Arc<dyn contextra_ports::Clock>)
src/hnsw/types.rs:150  pub fn get_entry_point(&self) -> Option<usize>
src/hnsw/types.rs:161  pub fn set_entry_point(&self, ep: Option<usize>)
src/hnsw/types.rs:171  pub fn get_ram_entry_point(&self) -> Option<usize>
src/hnsw/types.rs:182  pub fn set_ram_entry_point(&self, ep: Option<usize>)
src/hnsw/types.rs:191  pub fn get_ram_node_connections(&self, ram_idx: usize, layer: usize, m: usize) -> Vec<u32>
src/hnsw/types.rs:196  pub struct HnswColdCore
src/hnsw/types.rs:222  pub struct HnswIndexCore
src/hnsw/types.rs:228  pub enum RebuildStatus
src/hnsw/types.rs:239  pub fn try_new_with_rng_and_clock(
src/hnsw/types.rs:311  pub fn try_new_with_rng(
src/hnsw/types.rs:323  pub fn try_new(config: HnswConfig) -> Result<Self>
src/hnsw/types.rs:329  pub fn try_new_with_kernel(
src/hnsw/types.rs:341  pub fn new(config: HnswConfig) -> Self
src/hnsw/types.rs:412  pub fn set_fault_injection_insert_target(&self, target: u64)
src/hnsw/types.rs:424  pub fn quantizer(&self) -> Option<crate::quantize::ScalarQuantizer>
src/hnsw/types.rs:429  pub fn sq8_bias(&self) -> Sq8Bias
src/hnsw/types.rs:434  pub fn metrics_sink(&self) -> std::sync::Arc<dyn contextra_ports::MetricsSink>
src/hnsw/types.rs:439  pub fn set_metrics_sink(&self, sink: std::sync::Arc<dyn contextra_ports::MetricsSink>)
src/hnsw/types.rs:444  pub fn with_metrics_sink(self, sink: std::sync::Arc<dyn contextra_ports::MetricsSink>) -> Self
src/hnsw/types.rs:450  pub fn set_clock(&self, clock: std::sync::Arc<dyn contextra_ports::Clock>)
src/hnsw/types.rs:455  pub fn with_clock(self, clock: std::sync::Arc<dyn contextra_ports::Clock>) -> Self
src/hnsw/types.rs:461  pub fn set_layer_seed(&self, seed: u64)
src/hnsw/types.rs:465  pub fn deleted_ratio(&self) -> f64
src/hnsw/types.rs:469  pub fn rebuild_count(&self) -> u64
src/hnsw/types.rs:473  pub fn visited_dead_nodes(&self) -> u64
src/hnsw/types.rs:477  pub fn connectivity_score(&self) -> f64
src/hnsw/types.rs:481  pub fn check_connectivity(&self) -> contextra_types::Result<()>
src/hnsw/types.rs:485  pub fn is_rebuild_required(&self) -> bool
src/hnsw/types.rs:489  pub fn rebuild(&self) -> Result<()>
src/hnsw/types.rs:493  pub fn rebuild_sync(&self) -> Result<()>
src/hnsw/types.rs:497  pub fn rebuild_region(&self, region_node_ids: Vec<u64>) -> Result<()>
src/hnsw/types.rs:501  pub fn rebuild_region_sync(&self, region_node_ids: Vec<u64>) -> Result<()>
src/hnsw/types.rs:506  pub fn check_and_trigger_partial_rebuild(&self)
src/hnsw/types.rs:542  pub fn compact_seq_log(&self, min_active_seqno: u64)
src/hnsw/types.rs:546  pub fn pin_snapshot(&self, seq_no: u64)
src/hnsw/types.rs:550  pub fn unpin_snapshot(&self, seq_no: u64)
src/hnsw/types.rs:555  pub fn entry_point(&self) -> Option<usize>
src/hnsw/types.rs:560  pub fn node_idx_for_doc(&self, doc_id: DocId) -> Option<usize>
src/hnsw/types.rs:569  pub fn layer0_connections(&self, node_idx: usize) -> Vec<u32>
src/hnsw/types.rs:596  pub fn all_doc_ids_from_map(&self) -> Vec<DocId>
src/hnsw/types.rs:606  pub fn all_doc_ids_and_layers(&self) -> Vec<(DocId, usize)>
src/hnsw/types.rs:619  pub fn trigger_rebuild_async(&self)
src/hnsw/types.rs:632  pub fn clock(&self) -> std::sync::Arc<dyn contextra_ports::Clock>
src/hnsw/types.rs:636  pub fn rebuild_status(&self) -> RebuildStatus
src/hnsw/types.rs:640  pub fn wait_for_rebuild_blocking(
src/hnsw/types.rs:648  pub fn wait_for_rebuild(&self) -> bool
src/hnsw/types.rs:652  pub fn wait_for_rebuild_with_timeout(&self, timeout: std::time::Duration) -> bool
src/hnsw/types.rs:656  pub fn load_mmap_sync(&self, path: impl AsRef<std::path::Path>) -> Result<()>
src/hnsw/types.rs:661  pub fn load_mmap(&self, path: impl AsRef<std::path::Path>) -> Result<()>
src/hnsw/types.rs:964  pub fn save_sync(&self, path: impl AsRef<std::path::Path>) -> Result<()>
src/hnsw/types.rs:969  pub fn save(&self, path: impl AsRef<std::path::Path>) -> Result<()>
src/hnsw/vector_index_impl.rs:566  pub trait Index<Id, Query: ?Sized>
src/hnsw/vector_index_impl.rs:581  pub trait Maintain
src/hnsw/vector_index_impl.rs:594  pub struct HnswCompactStats
src/lib.rs:39  pub mod casting
src/lib.rs:41  pub mod contract
src/lib.rs:42  pub mod error
src/lib.rs:43  pub mod types
src/lib.rs:46  pub mod diskann
src/lib.rs:47  pub mod distance
src/lib.rs:48  pub mod hnsw
src/lib.rs:49  pub mod persistence
src/lib.rs:50  pub mod quantize
src/lib.rs:52  pub mod acorn
src/lib.rs:53  pub mod candidate_stream
src/lib.rs:54  pub mod compute_pool
src/lib.rs:55  pub mod fixed_hasher
src/lib.rs:56  pub mod partial_rebuild
src/lib.rs:59  pub mod quantize_rabitq
src/lib.rs:61  pub use candidate_stream::{VectorCandidateStream, DEFAULT_VECTOR_STREAM_BATCH_SIZE}
src/lib.rs:62  pub use compute_pool::ComputePool
src/lib.rs:63  pub use contract::
src/lib.rs:69  pub use diskann::{DiskAnnConfig, DiskAnnFallbackPolicy, DiskAnnIndex}
src/lib.rs:80  pub use hnsw::
src/lib.rs:83  pub use persistence::{HnswHeader, MmapIndex}
src/lib.rs:84  pub use quantize::ScalarQuantizer
src/lib.rs:86  pub use quantize_rabitq::RaBitQQuantizer
src/lib.rs:87  pub use types::{Clock, DistanceMetric, DocId, Maintain, MetricsSink, Rng, ScoredDocument, TxId}
src/partial_rebuild.rs:41  pub struct PartialRebuildConfig
src/partial_rebuild.rs:62  pub struct TraversalTracker
src/partial_rebuild.rs:69  pub fn new(config: PartialRebuildConfig) -> Self
src/partial_rebuild.rs:77  pub fn record_traversal(&mut self, visited_node_ids: Vec<u64>)
src/partial_rebuild.rs:89  pub fn find_oversaturated_regions(
src/partial_rebuild.rs:129  pub fn hot_path_nodes(&self) -> impl Iterator<Item = u64> + '_
src/partial_rebuild.rs:150  pub fn should_trigger_partial_rebuild(
src/persistence/atomic.rs:11  pub fn write_atomic(destination: &Path, data: &[u8]) -> io::Result<()>
src/persistence/header.rs:9  pub const HNSW_MAGIC: u32 = 0x484E5357
src/persistence/header.rs:11  pub const HNSW_VERSION: u16 = 2
src/persistence/header.rs:15  pub struct HnswHeader
src/persistence/header.rs:36  pub const SIZE: usize = 84
src/persistence/header.rs:39  pub fn new(
src/persistence/header.rs:70  pub fn new_v2(
src/persistence/header.rs:105  pub fn new_v2_with_bias(
src/persistence/header.rs:143  pub fn magic(&self) -> u32
src/persistence/header.rs:147  pub fn version(&self) -> u16
src/persistence/header.rs:151  pub fn m(&self) -> u32
src/persistence/header.rs:155  pub fn metric(&self) -> u8
src/persistence/header.rs:159  pub fn quantized(&self) -> u8
src/persistence/header.rs:163  pub fn q_min(&self) -> f32
src/persistence/header.rs:167  pub fn q_max(&self) -> f32
src/persistence/header.rs:171  pub fn nodes_offset(&self) -> u64
src/persistence/header.rs:175  pub fn connections_offset(&self) -> u64
src/persistence/header.rs:179  pub fn set_connections_offset(&mut self, offset: u64)
src/persistence/header.rs:183  pub fn node_count(&self) -> u64
src/persistence/header.rs:187  pub fn last_tx_id(&self) -> u64
src/persistence/header.rs:191  pub fn quant_calibration_offset(&self) -> u64
src/persistence/header.rs:195  pub fn quant_calibration_len(&self) -> u32
src/persistence/header.rs:199  pub fn sq8_bias_mean(&self) -> f32
src/persistence/header.rs:203  pub fn sq8_bias_variance(&self) -> f32
src/persistence/header.rs:207  pub fn sq8_bias(&self) -> Sq8Bias
src/persistence/header.rs:215  pub fn dimension(&self) -> u32
src/persistence/header.rs:219  pub fn entry_point(&self) -> i64
src/persistence/header.rs:223  pub fn is_quantized(&self) -> bool
src/persistence/header.rs:227  pub fn q_range(&self) -> (f32, f32)
src/persistence/header.rs:231  pub fn try_from_bytes(bytes: &[u8]) -> Result<Self>
src/persistence/header.rs:416  pub fn to_bytes(&self) -> [u8; Self::SIZE]
src/persistence/mmap.rs:12  pub struct MmapIndex
src/persistence/mmap.rs:19  pub fn open(path: impl AsRef<std::path::Path>) -> Result<Self>
src/persistence/mmap.rs:95  pub fn get_node_record(&self, index: usize) -> Result<NodeRecord>
src/persistence/mmap.rs:113  pub fn get_vector(&self, record: &NodeRecord) -> Result<&[u8]>
src/persistence/mmap.rs:132  pub fn get_connections(&self, record: &NodeRecord, layer: usize) -> Result<Vec<u32>>
src/persistence/mod.rs:9  pub mod atomic
src/persistence/mod.rs:17  pub use atomic::write_atomic
src/persistence/mod.rs:18  pub use header::{HnswHeader, HNSW_MAGIC, HNSW_VERSION}
src/persistence/mod.rs:19  pub use mmap::MmapIndex
src/persistence/mod.rs:20  pub use node::NodeRecord
src/persistence/node.rs:9  pub struct NodeRecord
src/persistence/node.rs:21  pub const SIZE: usize = 8 + 1 + 8 + 8; // 25 bytes
src/persistence/node.rs:23  pub const SIZE: usize = 16 + 1 + 8 + 8; // 33 bytes
src/persistence/node.rs:25  pub fn from_bytes(bytes: &[u8]) -> Result<Self>
src/persistence/node.rs:72  pub fn to_bytes(&self) -> Vec<u8>
src/quantize/mod.rs:25  pub const DEFAULT_P_LOW: f32 = 0.005
src/quantize/mod.rs:27  pub const DEFAULT_P_HIGH: f32 = 0.995
src/quantize/mod.rs:30  pub const SQ8_MAX_ERROR_FACTOR_PER_DIM: f32 = 1.0 / 510.0
src/quantize/mod.rs:37  pub struct ScalarQuantizer
src/quantize/mod.rs:72  pub fn try_train(batch: &[&[f32]], dimension: usize) -> contextra_types::Result<Self>
src/quantize/mod.rs:78  pub fn try_train_with_percentiles(
src/quantize/mod.rs:182  pub fn train_with_percentiles(
src/quantize/mod.rs:205  pub fn train(batch: &[&[f32]], dimension: usize) -> Self
src/quantize/mod.rs:218  pub fn mins(&self) -> &[f32]
src/quantize/mod.rs:223  pub fn maxes(&self) -> &[f32]
src/quantize/mod.rs:228  pub fn scales(&self) -> &[f32]
src/quantize/mod.rs:233  pub fn inv_scales(&self) -> &[f32]
src/quantize/mod.rs:241  pub fn max_reconstruction_error_per_dim(&self, dim: usize) -> Option<f32>
src/quantize/mod.rs:251  pub fn dimension(&self) -> usize
src/quantize/mod.rs:262  pub fn calculate_vector_drift(&self, vector: &[f32]) -> f32
src/quantize/mod.rs:281  pub fn check_drift(&self, vector: &[f32]) -> f32
src/quantize/mod.rs:290  pub fn drift_ratio(&self) -> f32
src/quantize/mod.rs:302  pub fn is_rebuild_required(&self, threshold: f32) -> bool
src/quantize/mod.rs:311  pub fn quantize(&self, vector: &[f32]) -> contextra_types::Result<Vec<u8>>
src/quantize/mod.rs:385  pub fn dequantize(&self, vector: &[u8]) -> contextra_types::Result<Vec<f32>>
src/quantize/mod.rs:418  pub fn asymmetric_dist(
src/quantize/mod.rs:477  pub fn symmetric_dist(
src/quantize_rabitq.rs:29  pub struct RaBitQQuantizer
src/quantize_rabitq.rs:37  pub fn try_train(vectors: &[Vec<f32>], dimension: usize) -> Result<Self>
src/quantize_rabitq.rs:49  pub fn try_train_with_seed(vectors: &[Vec<f32>], dimension: usize, seed: u64) -> Result<Self>
src/quantize_rabitq.rs:81  pub fn dimension(&self) -> usize
src/quantize_rabitq.rs:86  pub fn code_length(&self) -> usize
src/quantize_rabitq.rs:96  pub fn quantize(&self, vector: &[f32]) -> Result<Vec<u8>>
src/quantize_rabitq.rs:163  pub fn asymmetric_distance(&self, query: &[f32], code: &[u8]) -> Result<f32>
src/types.rs:18  pub struct DocId(pub u64)
src/types.rs:25  pub struct DocId(pub u128)
src/types.rs:29  pub const MIN: Self = Self(0)
src/types.rs:33  pub const fn new(id: u64) -> Self
src/types.rs:38  pub const fn new(id: u128) -> Self
src/types.rs:44  pub const fn inner(&self) -> u64
src/types.rs:49  pub const fn inner(&self) -> u128
src/types.rs:54  pub fn from_key(key: &str) -> Result<Self, crate::error::VectorError>
src/types.rs:97  pub struct TxId(pub u64)
src/types.rs:101  pub const SYSTEM: Self = Self(0)
src/types.rs:104  pub const fn new(id: u64) -> Self
src/types.rs:109  pub const fn as_u64(&self) -> u64
src/types.rs:130  pub enum DistanceMetric
src/types.rs:142  pub struct ScoredDocument
src/types.rs:153  pub fn new(doc_id: DocId, score: f32, distance: f32) -> Self
src/types.rs:163  pub const MAX_SEARCH_K: usize = 10_000
src/types.rs:166  pub const TOMBSTONE_BIT: u64 = 1 << 63
src/types.rs:169  pub trait Clock: Send + Sync
src/types.rs:178  pub struct SystemClock
src/types.rs:182  pub fn new() -> Self
src/types.rs:203  pub trait Rng: Send + Sync
src/types.rs:212  pub struct SeededRng
src/types.rs:218  pub fn new(seed: u64) -> Self
src/types.rs:245  pub trait MetricsSink: Send + Sync
src/types.rs:264  pub struct NoopMetricsSink
src/types.rs:272  pub trait Maintain: Send + Sync
```

## Innen: crate-interne Grenzen (`pub(crate)`, `pub(super)`, `pub(in …)`)

```rust
src/diskann/build.rs:15  pub(crate) type TombstoneSet = roaring::RoaringTreemap
src/diskann/build.rs:19  pub(crate) struct TombstoneSet
src/diskann/build.rs:20  pub(crate) set: std::collections::BTreeSet<u128>,
src/diskann/build.rs:25  pub(crate) fn new() -> Self
src/diskann/build.rs:31  pub(crate) fn insert(&mut self, id: u128) -> bool
src/diskann/build.rs:35  pub(crate) fn contains(&self, id: u128) -> bool
src/diskann/build.rs:39  pub(crate) fn len(&self) -> usize
src/diskann/build.rs:44  pub(crate) fn is_empty(&self) -> bool
src/diskann/build.rs:48  pub(crate) fn clear(&mut self)
src/diskann/build.rs:56  pub(crate) fn prune_in_memory(
src/diskann/build.rs:103  pub(crate) fn append_to_tombstone_wal_blocking(
src/diskann/build.rs:145  pub(crate) async fn append_to_tombstone_wal(path: &std::path::Path, id: DocId) -> Result<()>
src/diskann/build.rs:150  pub(crate) fn read_tombstone_wal(path: &std::path::Path) -> Result<TombstoneSet>
src/diskann/build.rs:243  pub(crate) fn append_to_pending_wal_blocking(
src/diskann/build.rs:299  pub(crate) async fn append_to_pending_wal(
src/diskann/build.rs:314  pub(crate) fn read_pending_wal(path: &std::path::Path) -> Result<Vec<(DocId, Vec<f32>)>>
src/diskann/build.rs:468  pub(crate) fn prune_streaming(
src/diskann/build.rs:526  pub(crate) fn write_incremental_to_file_sync(
src/diskann/build.rs:747  pub(crate) fn verify_graph_integrity_debug(&self) -> Result<()>
src/diskann/config.rs:68  pub(crate) struct CachedNode
src/diskann/config.rs:69  pub(crate) vector: VectorData,
src/diskann/config.rs:70  pub(crate) neighbors: Vec<u32>,
src/diskann/config.rs:71  pub(crate) doc_id: DocId,
src/diskann/config.rs:75  pub(crate) enum VectorData
src/diskann/filtered.rs:21  pub(crate) async fn search_filtered_internal(
src/diskann/format.rs:8  pub(crate) const DISKANN_MAGIC: &[u8; 4] = b"DANN"
src/diskann/format.rs:9  pub(crate) const DISKANN_VERSION: u16 = 1
src/diskann/format.rs:10  pub(crate) const DISKANN_FOOTER_MAGIC: &[u8; 4] = b"DANF"
src/diskann/format.rs:11  pub(crate) const DISKANN_INTEGRITY_KEY: &[u8; 32] = b"contextra_diskann_integri_key_32"
src/diskann/format.rs:13  pub(crate) const PENDING_WAL_MAGIC: &[u8; 4] = b"PWAL"
src/diskann/format.rs:14  pub(crate) const PENDING_WAL_VERSION: u8 = 1
src/diskann/format.rs:15  pub(crate) const PENDING_WAL_HEADER_SIZE: usize = 5
src/diskann/format.rs:17  pub(crate) const TOMBSTONE_WAL_MAGIC: &[u8; 4] = b"TWAL"
src/diskann/format.rs:18  pub(crate) const TOMBSTONE_WAL_VERSION: u8 = 1
src/diskann/format.rs:19  pub(crate) const TOMBSTONE_WAL_HEADER_SIZE: usize = 5
src/diskann/format.rs:20  pub(crate) const MAX_PENDING_WAL_DIM: usize = 65_536
src/diskann/format.rs:22  pub(crate) const PENDING_FLUSH_THRESHOLD_MIN: u64 = 50
src/diskann/format.rs:23  pub(crate) const PENDING_FLUSH_THRESHOLD_MAX: u64 = 1_000
src/diskann/format.rs:24  pub(crate) const PENDING_FLUSH_THRESHOLD_FACTOR: f64 = 0.05
src/diskann/format.rs:27  pub(crate) fn compute_adaptive_flush_threshold(n_persisted: u64) -> u64
src/diskann/format.rs:34  pub(crate) struct DiskAnnHeader
src/diskann/format.rs:35  pub(crate) magic: [u8; 4],
src/diskann/format.rs:36  pub(crate) version: u16,
src/diskann/format.rs:37  pub(crate) node_count: u64,
src/diskann/format.rs:38  pub(crate) dimension: u32,
src/diskann/format.rs:39  pub(crate) max_degree: u32,
src/diskann/format.rs:40  pub(crate) sector_size: u32,
src/diskann/format.rs:41  pub(crate) entry_point: u32,
src/diskann/format.rs:42  pub(crate) metric: u8,
src/diskann/format.rs:43  pub(crate) quantized: u8,
src/diskann/format.rs:44  pub(crate) q_min: f32,
src/diskann/format.rs:45  pub(crate) q_max: f32,
src/diskann/format.rs:49  pub(crate) const SIZE: usize = 40
src/diskann/format.rs:51  pub(crate) fn to_bytes(self) -> [u8; Self::SIZE]
src/diskann/format.rs:67  pub(crate) fn try_from_bytes(bytes: &[u8]) -> Result<Self>
src/diskann/format.rs:156  pub(crate) struct DiskAnnFooter
src/diskann/format.rs:157  pub(crate) magic: [u8; 4],
src/diskann/format.rs:158  pub(crate) hmac: [u8; 32],
src/diskann/format.rs:162  pub(crate) const SIZE: usize = 36
src/diskann/format.rs:164  pub(crate) fn to_bytes(self) -> [u8; Self::SIZE]
src/diskann/format.rs:172  pub(crate) fn try_from_bytes(bytes: &[u8]) -> Result<Self>
src/diskann/mod.rs:7  pub(crate) mod build
src/diskann/mod.rs:8  pub(crate) mod config
src/diskann/mod.rs:9  pub(crate) mod filtered
src/diskann/mod.rs:10  pub(crate) mod format
src/diskann/mod.rs:11  pub(crate) mod integrity
src/diskann/mod.rs:12  pub(crate) mod persistence
src/diskann/mod.rs:15  pub(crate) mod search
src/diskann/mod.rs:16  pub(crate) mod types
src/diskann/mod.rs:17  pub(crate) mod vector_index_impl
src/diskann/persistence.rs:98  pub(crate) fn load_all_vectors_from_mmap(&self) -> Result<(Vec<Vec<f32>>, Vec<DocId>)>
src/diskann/persistence.rs:129  pub(crate) fn write_to_path_sync(
src/diskann/persistence.rs:232  pub(crate) fn init_hnsw_fallback(&self) -> Result<Arc<crate::hnsw::HnswIndex>>
src/diskann/persistence.rs:466  pub(crate) fn load_node(&self, index: u32) -> Result<CachedNode>
src/diskann/search.rs:17  pub(crate) fn get_dist_to_query(&self, query: &[f32], node_idx: u32) -> Result<f32>
src/diskann/search.rs:33  pub(crate) fn search_in_memory(
src/diskann/search.rs:112  pub(crate) fn get_dist_mixed(
src/diskann/search.rs:131  pub(crate) fn get_vec_mixed(
src/diskann/search.rs:154  pub(crate) fn search_streaming(
src/diskann/search.rs:290  pub(crate) fn search_blocking(
src/diskann/types.rs:26  pub(crate) inner: Arc<DiskAnnIndexInner>,
src/diskann/types.rs:29  pub(crate) struct DiskAnnIndexInner
src/diskann/types.rs:30  pub(crate) config: DiskAnnConfig,
src/diskann/types.rs:31  pub(crate) header: RwLock<Option<DiskAnnHeader>>,
src/diskann/types.rs:32  pub(crate) mmap: RwLock<Option<Mmap>>,
src/diskann/types.rs:33  pub(crate) node_size_bytes: AtomicU64,
src/diskann/types.rs:34  pub(crate) cache: RwLock<AHashMap<u32, CachedNode>>,
src/diskann/types.rs:35  pub(crate) doc_ids: RwLock<Vec<DocId>>,
src/diskann/types.rs:36  pub(crate) quantizer: RwLock<Option<crate::quantize::ScalarQuantizer>>,
src/diskann/types.rs:37  pub(crate) drift_warn_count: AtomicU64,
src/diskann/types.rs:40  pub(crate) pending_inserts: RwLock<Vec<(DocId, Vec<f32>)>>,
src/diskann/types.rs:42  pub(crate) pending_count: AtomicU64,
src/diskann/types.rs:44  pub(crate) flushing_in_progress: AtomicBool,
src/diskann/types.rs:45  pub(crate) hnsw_fallback: RwLock<Option<Arc<crate::hnsw::HnswIndex>>>,
src/diskann/types.rs:46  pub(crate) tombstones: RwLock<TombstoneSet>,
src/diskann/types.rs:47  pub(crate) compute_pool: ComputePool,
src/diskann/types.rs:163  pub(crate) struct SearchCandidate
src/diskann/types.rs:164  pub(crate) index: u32,
src/diskann/types.rs:165  pub(crate) distance: f32,
src/diskann/types.rs:285  pub(crate) fn check_quantizer_drift(&self, vector: &[f32])
src/hnsw/batch.rs:48  pub(super) fn get_neighbor_conns_in_batch(
src/hnsw/batch.rs:81  pub(super) struct SearchContext<'a>
src/hnsw/batch.rs:82  pub(super) nodes: &'a [HnswNode],
src/hnsw/batch.rs:83  pub(super) mmap: Option<&'a crate::persistence::MmapIndex>,
src/hnsw/batch.rs:84  pub(super) mmap_node_count: usize,
src/hnsw/batch.rs:85  pub(super) prior_prepared: &'a [PreparedInsert],
src/hnsw/batch.rs:86  pub(super) backlink_map: Option<&'a BacklinkTable>,
src/hnsw/batch.rs:87  pub(super) quantizer: Option<Cow<'a, crate::quantize::ScalarQuantizer>>,
src/hnsw/batch.rs:88  pub(super) arena: &'a HnswArena,
src/hnsw/config.rs:64  pub(super) fn validate_vector(vec: &[f32]) -> Result<()>
src/hnsw/core_insert.rs:13  pub(super) fn resolve_connections<'a>(
src/hnsw/core_insert.rs:414  pub(super) fn do_insert(&self, id: DocId, vector: &[f32], tx_id: u64) -> Result<()>
src/hnsw/core_rebuild.rs:80  pub(super) fn random_layer(&self) -> usize
src/hnsw/core_rebuild.rs:174  pub(super) fn resolve_dist(
src/hnsw/core_rebuild.rs:207  pub(super) fn resolve_doc_id(&self, idx: usize, ctx: &SearchContext) -> Result<DocId>
src/hnsw/core_rebuild.rs:281  pub(super) fn compute_symmetric_distance_hybrid_with_batch(
src/hnsw/core_rebuild.rs:307  pub(super) fn select_neighbors_heuristic_with_batch(
src/hnsw/core_rebuild.rs:395  pub(super) fn do_delete(&self, id: DocId) -> Result<()>
src/hnsw/core_search.rs:15  pub(super) fn search_layer(
src/hnsw/core_search.rs:26  pub(super) fn search_layer_with_context(
src/hnsw/core_search.rs:167  pub(super) async fn search_filtered_internal(
src/hnsw/types.rs:33  pub(super) doc_id: DocId,
src/hnsw/types.rs:34  pub(super) vector: VectorData,
src/hnsw/types.rs:35  pub(super) max_layer: usize,
src/hnsw/types.rs:36  pub(super) committed_tx: u64,
src/hnsw/types.rs:78  pub(super) fn new(core: &'a HnswIndexCore, seq_no: u64) -> Self
src/hnsw/types.rs:92  pub(super) inner: std::sync::Arc<HnswIndexCore>,
src/hnsw/types.rs:93  pub(super) _kernel: std::marker::PhantomData<K>,
src/hnsw/types.rs:665  pub(super) fn load_mmap_from_instance(
src/hnsw/types.rs:761  pub(super) fn save_internal(&self, path: impl AsRef<std::path::Path>) -> Result<()>
src/quantize/mod.rs:38  pub(crate) mins: Vec<f32>,
src/quantize/mod.rs:39  pub(crate) maxes: Vec<f32>,
src/quantize/mod.rs:40  pub(crate) scales: Vec<f32>,
src/quantize/mod.rs:41  pub(crate) inv_scales: Vec<f32>,
src/quantize/mod.rs:42  pub(crate) dimension: usize,
src/quantize/mod.rs:44  pub(crate) total_queries: AtomicU64,
src/quantize/mod.rs:46  pub(crate) out_of_range_queries: AtomicU64,
```
