# API: contextra-graph

> Erzeugt aus dem Quellcode am 2026-11-25, Repository HEAD ce4579a56. Eine Zeile je Item: Datei:Zeile und die **erste Zeile** der Signatur. Mehrzeilige Signaturen sind nur angeschnitten. Nicht kompiliert, daher ohne Typprüfung. Testcode (`tests`, `testing`, `test_doubles`) ist ausgeschlossen.

Umfang: 462 öffentliche Items, 104 crate-interne Sichtbarkeitsgrenzen.

## Innen: Abhängigkeiten zu anderen Workspace-Crates

- contextra-ports
- contextra-types

## Außen: öffentliche Schnittstelle (`pub`)

```rust
src/apprh/diffusion.rs:23  pub fn run_apprh_push<G: PathGraph>(
src/apprh/error.rs:8  pub enum ApprhError
src/apprh/gate_monitor.rs:20  pub struct ApprhGateMonitorConfig
src/apprh/gate_monitor.rs:42  pub fn validate(&self) -> Result<(), ApprhError>
src/apprh/gate_monitor.rs:65  pub struct ApprhObservationRecord
src/apprh/gate_monitor.rs:79  pub struct ApprhGateMonitorSnapshot
src/apprh/gate_monitor.rs:110  pub struct ApprhGateMonitor
src/apprh/gate_monitor.rs:122  pub fn new(config: ApprhGateMonitorConfig) -> Result<Self, ApprhError>
src/apprh/gate_monitor.rs:131  pub fn with_gate<G>(config: ApprhGateMonitorConfig, gate: G) -> Result<Self, ApprhError>
src/apprh/gate_monitor.rs:153  pub fn observe(&self, report: &ApprhShadowComparison)
src/apprh/gate_monitor.rs:184  pub fn production_ready(&self) -> bool
src/apprh/gate_monitor.rs:191  pub fn snapshot(&self) -> ApprhGateMonitorSnapshot
src/apprh/gate_monitor.rs:233  pub fn reset(&self)
src/apprh/gate_monitor.rs:242  pub fn config(&self) -> &ApprhGateMonitorConfig
src/apprh/mod.rs:3  pub mod diffusion
src/apprh/mod.rs:4  pub mod error
src/apprh/mod.rs:5  pub mod gate_monitor
src/apprh/mod.rs:6  pub mod params
src/apprh/mod.rs:7  pub mod selector
src/apprh/mod.rs:8  pub mod shadow
src/apprh/mod.rs:10  pub use diffusion::run_apprh_push
src/apprh/mod.rs:11  pub use error::ApprhError
src/apprh/mod.rs:12  pub use gate_monitor::
src/apprh/mod.rs:15  pub use params::ApprhParams
src/apprh/mod.rs:16  pub use selector::{ApprhMode, ApprhSelector, ApprhSelectorConfig}
src/apprh/mod.rs:17  pub use shadow::
src/apprh/mod.rs:35  pub fn apprh_local<G: PathGraph>(
src/apprh/mod.rs:51  pub fn forward_push_apprh<G: PathGraph>(
src/apprh/mod.rs:82  pub fn apprh_diffusion(
src/apprh/mod.rs:92  pub fn apprh_shadow_compare(
src/apprh/params.rs:12  pub struct ApprhParams
src/apprh/params.rs:40  pub fn validate(&self) -> Result<(), ApprhError>
src/apprh/selector.rs:25  pub enum ApprhMode
src/apprh/selector.rs:40  pub struct ApprhSelectorConfig
src/apprh/selector.rs:68  pub struct ApprhSelector
src/apprh/selector.rs:78  pub fn new(config: ApprhSelectorConfig, monitor: Arc<ApprhGateMonitor>) -> Self
src/apprh/selector.rs:92  pub fn with_default_monitor(config: ApprhSelectorConfig) -> Result<Self, ApprhError>
src/apprh/selector.rs:99  pub fn mode(&self) -> ApprhMode
src/apprh/selector.rs:109  pub fn set_mode(&self, new_mode: ApprhMode) -> Result<(), ApprhError>
src/apprh/selector.rs:125  pub fn promote_to_production(&self) -> Result<(), ApprhError>
src/apprh/selector.rs:131  pub fn demote_to_shadow(&self)
src/apprh/selector.rs:137  pub fn monitor(&self) -> &ApprhGateMonitor
src/apprh/selector.rs:146  pub fn evaluate_and_run<G: PathGraph>(
src/apprh/shadow.rs:13  pub trait ApprhFlipGate
src/apprh/shadow.rs:21  pub struct DefaultApprhFlipGate
src/apprh/shadow.rs:32  pub struct ApprhShadowComparison
src/apprh/shadow.rs:54  pub fn shadow_compare_forward_push_vs_apprh<G: PathGraph>(
src/arc_slice.rs:12  pub struct ArcSlice<T>
src/arc_slice.rs:22  pub fn new(shared: Arc<[T]>) -> Self
src/arc_slice.rs:34  pub fn from_vec(vec: Vec<T>) -> Self
src/arc_slice.rs:42  pub fn try_slice(&self, range: impl RangeBounds<usize>) -> Option<Self>
src/arc_slice.rs:70  pub fn slice(&self, range: impl RangeBounds<usize>) -> Self
src/arc_slice.rs:81  pub fn as_arc(&self) -> &Arc<[T]>
src/arc_slice.rs:88  pub fn offset(&self) -> usize
src/arc_slice.rs:95  pub fn len(&self) -> usize
src/arc_slice.rs:102  pub fn is_empty(&self) -> bool
src/arc_slice.rs:109  pub fn as_slice(&self) -> &[T]
src/cascade.rs:18  pub const MAX_HYPEREDGE_CASCADE_FANOUT: usize = 1_000
src/cascade.rs:22  pub const DEFAULT_HYPEREDGE_CASCADE_FANOUT_LIMIT: usize = MAX_HYPEREDGE_CASCADE_FANOUT
src/cascade.rs:27  pub struct CascadeInvalidationReport
src/cascade.rs:77  pub const CASCADE_QUEUE_PREFIX: &str = "__graph:cascade_queue:"
src/cascade.rs:82  pub enum CascadeStatus
src/cascade.rs:96  pub struct CascadeDeletionRecord
src/cascade.rs:105  pub struct CascadeTicket
src/cascade.rs:114  pub trait GraphGarbageCollection
src/cascade.rs:156  pub struct HyperedgeCascadeReport
src/cascade.rs:167  pub type CascadeReport = HyperedgeCascadeReport
src/cascade.rs:186  pub fn topological_sort_hyperedge_closure(
src/cascade.rs:379  pub fn verify_doc_hyperedge_deletion_completeness(
src/community.rs:29  pub struct CommunityDetectionConfig
src/community.rs:43  pub struct VirtualHyperedgeNode
src/community.rs:51  pub fn new(hyperedge_id: u64) -> Self
src/community.rs:66  pub struct StarExpansionIterator
src/community.rs:76  pub fn new(graph: &CsrGraph) -> Self
src/community.rs:96  pub fn next_with_weight(&mut self) -> Option<(EntityId, VirtualHyperedgeNode, f32)>
src/community.rs:140  pub struct CommunityAssignment
src/consistency_enforcement.rs:26  pub type EdgeId = (EntityId, EntityId)
src/consistency_enforcement.rs:31  pub struct ConflictPattern
src/consistency_enforcement.rs:47  pub struct EdgeAssertion
src/consistency_enforcement.rs:59  pub fn pattern_hash(&self) -> [u8; 32]
src/consistency_enforcement.rs:70  pub trait ContradictionDetector
src/consistency_enforcement.rs:80  pub struct ExactPredicateConflictDetector
src/consistency_enforcement.rs:93  pub struct ConsistencyEnforcer
src/consistency_enforcement.rs:101  pub const DEFAULT_SUPPRESSION_THRESHOLD: u32 = 3
src/consistency_enforcement.rs:105  pub fn new(suppression_threshold: u32) -> Self
src/consistency_enforcement.rs:115  pub fn check_before_insert(&mut self, assertion: &EdgeAssertion) -> Option<ConflictPattern>
src/consistency_enforcement.rs:124  pub fn detect_contradiction<D: ContradictionDetector>(
src/consistency_enforcement.rs:138  pub fn record_contradiction(
src/consistency_enforcement.rs:166  pub fn is_suppressed(&self, pattern_hash: [u8; 32]) -> bool
src/consistency_enforcement.rs:174  pub fn active_patterns(&self) -> impl Iterator<Item = &ConflictPattern>
src/consistency_enforcement.rs:180  pub fn get_pattern(&self, pattern_hash: &[u8; 32]) -> Option<&ConflictPattern>
src/consistency_enforcement.rs:186  pub fn suppression_threshold(&self) -> u32
src/consistency_enforcement.rs:198  pub fn suggest_tombstone_candidates(
src/contract.rs:4  pub enum Error
src/contract.rs:13  pub type Result<T> = core::result::Result<T, Error>
src/contract.rs:15  pub trait Clock: Send + Sync { fn now_unix_ms(&self) -> u64; }
src/contract.rs:17  pub trait Rng: Send { fn next_u64(&mut self) -> u64; }
src/contract.rs:19  pub type NodeId = u64
src/contract.rs:21  pub type Weight = f32;                      // endlich, >= 0; Validierung an der API-Grenze
src/csr.rs:16  pub use graph_write::CsrGraph
src/csr.rs:17  pub use inner::{GraphInner, MemoryEstimate}
src/csr.rs:18  pub use types::
src/csr.rs:22  pub use visibility::
src/csr/graph_index/mod.rs:7  pub mod mutations
src/csr/graph_index/mod.rs:8  pub mod traversal
src/csr/graph_read/communities.rs:13  pub fn set_communities_batch(&self, assignments: &[crate::CommunityAssignment])
src/csr/graph_read/mod.rs:6  pub mod communities
src/csr/graph_read/mod.rs:7  pub mod pagerank
src/csr/graph_read/mod.rs:8  pub mod path_rag
src/csr/graph_read/mod.rs:17  pub fn compact(&self)
src/csr/graph_read/mod.rs:129  pub fn entity_count(&self) -> usize
src/csr/graph_read/mod.rs:138  pub fn entity_exists(&self, id: EntityId) -> bool
src/csr/graph_read/mod.rs:156  pub fn edge_count(&self) -> usize
src/csr/graph_read/path_rag.rs:12  pub fn path_rag_at<'a>(
src/csr/graph_write/edge_mutation.rs:171  pub fn load_edge_direct(
src/csr/graph_write/entity.rs:12  pub fn insert_entity_direct(&self, entity: Entity) -> Result<()>
src/csr/graph_write/entity.rs:23  pub fn load_entity_direct(&self, entity: Entity) -> Result<()>
src/csr/graph_write/hyperedge.rs:42  pub fn enqueue_cascade_deferred(
src/csr/graph_write/hyperedge.rs:54  pub fn pending_cascade_queue_len(&self) -> usize
src/csr/graph_write/hyperedge.rs:106  pub fn relate_n_ary(
src/csr/graph_write/hyperedge.rs:156  pub fn insert_hyperedge_direct(&self, hyperedge: crate::hyperedge::HyperEdge)
src/csr/graph_write/hyperedge.rs:185  pub fn parent_hyperedges_of(
src/csr/graph_write/hyperedge.rs:210  pub fn hyperedges_for_doc(&self, doc_id: DocId) -> Vec<crate::hyperedge::HyperEdgeId>
src/csr/graph_write/hyperedge.rs:230  pub fn max_hyperedge_id(&self) -> u64
src/csr/graph_write/hyperedge.rs:241  pub fn get_hyperedge(
src/csr/graph_write/hyperedge.rs:254  pub fn hyperedges_for_entity(&self, entity_id: EntityId) -> Vec<crate::hyperedge::HyperEdgeId>
src/csr/graph_write/hyperedge.rs:357  pub fn commit_super_edge_batch(
src/csr/graph_write/hyperedge.rs:384  pub fn insert_hyperedge(&self, edge: crate::hyperedge::HyperEdge)
src/csr/graph_write/mod.rs:6  pub mod edge_mutation
src/csr/graph_write/mod.rs:7  pub mod entity
src/csr/graph_write/mod.rs:8  pub mod hyperedge
src/csr/graph_write/mod.rs:9  pub mod tombstone
src/csr/graph_write/mod.rs:26  pub struct CsrGraph
src/csr/graph_write/mod.rs:39  pub fn new() -> Self
src/csr/graph_write/mod.rs:44  pub fn with_config(config: CsrGraphConfig) -> Self
src/csr/graph_write/mod.rs:59  pub fn with_storage(storage: Arc<dyn StorageEngine>) -> Self
src/csr/graph_write/mod.rs:64  pub fn with_config_and_storage(
src/csr/graph_write/mod.rs:82  pub fn with_consistency_enforcer(suppression_threshold: u32) -> Self
src/csr/graph_write/mod.rs:100  pub fn personalized_page_rank_at<'a>(
src/csr/graph_write/mod.rs:113  pub fn rollback_to_tx<'a>(
src/csr/graph_write/mod.rs:120  pub fn inner_read(&self) -> arc_swap::Guard<Arc<GraphInner>>
src/csr/graph_write/mod.rs:132  pub fn set_storage(&mut self, storage: Arc<dyn StorageEngine>)
src/csr/graph_write/mod.rs:137  pub fn storage(&self) -> Option<Arc<dyn StorageEngine>>
src/csr/graph_write/mod.rs:142  pub fn estimate_memory(&self, presized: bool) -> MemoryEstimate
src/csr/graph_write/mod.rs:148  pub fn estimate_memory_bytes(&self) -> usize
src/csr/graph_write/mod.rs:154  pub fn estimate_compaction_peak_bytes(&self) -> usize
src/csr/graph_write/mod.rs:160  pub fn get_source_doc_id(&self, index: usize) -> Option<DocId>
src/csr/graph_write/mod.rs:166  pub fn source_doc_id_at(&self, from: EntityId, to: EntityId) -> Option<DocId>
src/csr/graph_write/mod.rs:202  pub fn edges_for_doc(&self, doc_id: DocId) -> Vec<(EntityId, EntityId)>
src/csr/graph_write/tombstone.rs:72  pub fn tombstone_hyperedge(&self, id: crate::hyperedge::HyperEdgeId, wal_tx: TxId) -> bool
src/csr/inner/compaction.rs:13  pub fn get_or_create_index(&mut self, id: EntityId) -> InternalIndex
src/csr/inner/compaction.rs:182  pub fn compact(&mut self)
src/csr/inner/estimate.rs:16  pub struct MemoryEstimate
src/csr/inner/estimate.rs:24  pub fn total_bytes(&self) -> usize
src/csr/inner/mod.rs:6  pub mod compaction
src/csr/inner/mod.rs:7  pub mod estimate
src/csr/inner/mod.rs:8  pub mod reinforcement
src/csr/inner/mod.rs:20  pub use estimate::MemoryEstimate
src/csr/inner/mod.rs:24  pub struct GraphInner
src/csr/inner/mod.rs:73  pub fn new() -> Self
src/csr/types.rs:18  pub enum EdgeType
src/csr/types.rs:26  pub struct CsrEdge
src/csr/types.rs:38  pub fn new(target: EntityId, weight: f32) -> Self
src/csr/types.rs:55  pub struct PersistedEdgePayload
src/csr/types.rs:71  pub const SCORE_DECAY: f32 = 0.7
src/csr/types.rs:75  pub const MAX_TRAVERSAL_HOPS: u8 = 3
src/csr/types.rs:79  pub const MAX_VISITED_NODES: usize = 10_000
src/csr/types.rs:83  pub struct CsrGraphConfig
src/csr/visibility.rs:17  pub const WALLCLOCK_TX_HEURISTIC_MIN: u64 = 1_400_000_000_000_000_000
src/csr/visibility.rs:38  pub fn is_suspicious_tx_id(tx: TxId) -> bool
src/csr/visibility.rs:46  pub fn is_edge_visible(
src/csr/visibility.rs:57  pub fn is_edge_visible_business(
src/csr/visibility.rs:72  pub fn is_edge_visible_bitemporal(
src/edge_reinforcement.rs:8  pub struct EdgeReinforcementConfig
src/edge_reinforcement.rs:132  pub fn compute_edge_weight(cooccurrence_weight: f32, traversal_weight: f32, alpha: f32) -> f32
src/edge_reinforcement.rs:137  pub fn apply_cooccurrence_reinforcement(
src/edge_reinforcement.rs:148  pub fn apply_traversal_reinforcement(
src/edge_reinforcement.rs:162  pub fn apply_weight_normalization(outgoing_edges: &mut [CsrEdge], w_max: f32)
src/edge_reinforcement_buffer.rs:29  pub struct CooccurrenceSignal
src/edge_reinforcement_buffer.rs:39  pub struct TraversalSignal
src/edge_reinforcement_buffer.rs:53  pub struct EdgeReinforcementBuffer
src/edge_reinforcement_buffer.rs:61  pub fn new() -> Self
src/edge_reinforcement_buffer.rs:68  pub fn push_cooccurrence(&self, from: EntityId, to: EntityId, co_activation: f32)
src/edge_reinforcement_buffer.rs:78  pub fn push_traversal(&self, from: EntityId, to: EntityId, path_length: usize)
src/edge_reinforcement_buffer.rs:88  pub fn cooccurrence_count(&self) -> usize
src/edge_reinforcement_buffer.rs:94  pub fn traversal_count(&self) -> usize
src/edge_reinforcement_buffer.rs:103  pub fn flush_to_graph(&self, graph: &CsrGraph, config: &EdgeReinforcementConfig)
src/entity_extraction.rs:27  pub struct ExtractedRelation
src/entity_extraction.rs:41  pub fn new(
src/entity_extraction.rs:64  pub trait EntityExtractor: Send + Sync
src/entity_extraction.rs:86  pub struct RuleBasedEntityExtractor
src/entity_extraction.rs:96  pub fn new(min_confidence: f32) -> Self
src/entity_extraction.rs:251  pub fn ingest_extracted_relations(
src/error.rs:6  pub type Result<T, E = GraphError> = std::result::Result<T, E>
src/error.rs:11  pub enum GraphError
src/error.rs:48  pub enum GraphMutationError
src/hyperedge.rs:18  pub const HYPEREDGE_PREFIX: &str = "__graph:hyperedge:"
src/hyperedge.rs:22  pub const HYPEREDGE_BY_ENTITY_PREFIX: &str = "__graph:hyperedge_by_entity:"
src/hyperedge.rs:30  pub fn star_weight(weight: f32, cardinality: usize) -> f32
src/hyperedge.rs:39  pub use contextra_ports::graph::{HyperEdgeId, RoleBinding, RoleId}
src/hyperedge.rs:44  pub struct HyperEdge
src/hyperedge.rs:97  pub fn new(
src/hyperedge.rs:119  pub fn with_tx_validity(mut self, valid_from: Option<TxId>, valid_to: Option<TxId>) -> Self
src/hyperedge.rs:127  pub fn with_business_validity(
src/hyperedge.rs:139  pub fn with_source_doc_id(mut self, source_doc_id: Option<DocId>) -> Self
src/hyperedge.rs:148  pub fn with_child_edge_ids(mut self, ids: impl Into<Arc<[HyperEdgeId]>>) -> Self
src/hyperedge.rs:168  pub fn validate(&self) -> std::result::Result<(), GraphMutationError>
src/hyperedge.rs:192  pub fn serialize(&self) -> Result<Vec<u8>>
src/hyperedge.rs:200  pub fn deserialize(bytes: &[u8]) -> Result<Self>
src/hyperedge.rs:223  pub fn view(&self) -> HyperEdgeView<'_>
src/hyperedge.rs:231  pub struct HyperEdgeView<'a>
src/hyperedge.rs:239  pub fn new(edge: &'a HyperEdge) -> Self
src/hyperedge.rs:248  pub fn slice_participants(
src/hyperedge.rs:271  pub struct RoleInterner
src/hyperedge.rs:289  pub fn sort_dedup_entities(entities: &[EntityId]) -> Vec<EntityId>
src/hyperedge.rs:312  pub struct ConsolidationNodesGuard
src/hyperedge.rs:320  pub fn acquire(entities: &[EntityId]) -> Self
src/hyperedge.rs:330  pub fn try_acquire(entities: &[EntityId]) -> std::result::Result<Self, GraphMutationError>
src/hyperedge.rs:338  pub fn entities(&self) -> &[EntityId]
src/hyperedge.rs:344  pub fn with_entities<F, R>(&self, f: F) -> R
src/hyperedge.rs:355  pub fn new() -> Self
src/hyperedge.rs:368  pub fn get_or_intern(&self, name: &str) -> RoleId
src/hyperedge.rs:389  pub fn resolve<F, R>(&self, id: RoleId, f: F) -> Option<R>
src/hyperedge.rs:398  pub fn resolve_string(&self, id: RoleId) -> Option<String>
src/hyperedge.rs:404  pub fn contains_role(&self, name: &str) -> bool
src/hyperedge.rs:410  pub fn contains_id(&self, id: RoleId) -> bool
src/hyperedge.rs:416  pub fn len(&self) -> usize
src/hyperedge.rs:422  pub fn is_empty(&self) -> bool
src/hyperedge_suggest.rs:14  pub const MAX_RELATE_PARTICIPANTS: usize = 64
src/hyperedge_suggest.rs:19  pub struct HyperEdgeCandidate
src/hyperedge_suggest.rs:31  pub struct ValidatedHyperEdgeCandidate
src/hyperedge_suggest.rs:45  pub enum HyperedgeSuggestError
src/hyperedge_suggest.rs:67  pub fn compute_co_occurrence_candidates(
src/lib.rs:35  pub mod apprh
src/lib.rs:36  pub mod arc_slice
src/lib.rs:37  pub mod cascade
src/lib.rs:38  pub mod community
src/lib.rs:39  pub mod consistency_enforcement
src/lib.rs:40  pub mod csr
src/lib.rs:42  pub mod edge_reinforcement
src/lib.rs:44  pub mod edge_reinforcement_buffer
src/lib.rs:45  pub mod entity_extraction
src/lib.rs:46  pub mod error
src/lib.rs:47  pub mod hyperedge
src/lib.rs:48  pub mod hyperedge_suggest
src/lib.rs:49  pub mod path_rag
src/lib.rs:51  pub mod percolation
src/lib.rs:52  pub mod ppr
src/lib.rs:53  pub mod ppr_stream
src/lib.rs:54  pub mod provenance
src/lib.rs:55  pub mod provider
src/lib.rs:56  pub mod purger
src/lib.rs:58  pub mod session_dag
src/lib.rs:59  pub mod test_doubles
src/lib.rs:60  pub mod tl_hfd
src/lib.rs:63  pub use apprh::
src/lib.rs:68  pub use arc_slice::ArcSlice
src/lib.rs:69  pub use cascade::
src/lib.rs:74  pub use community::
src/lib.rs:78  pub use consistency_enforcement::
src/lib.rs:82  pub use csr::CsrGraph
src/lib.rs:84  pub use edge_reinforcement::
src/lib.rs:89  pub use edge_reinforcement_buffer::{CooccurrenceSignal, EdgeReinforcementBuffer, TraversalSignal}
src/lib.rs:90  pub use entity_extraction::
src/lib.rs:93  pub use error::{GraphError, GraphMutationError, Result}
src/lib.rs:94  pub use error::GraphError as Error
src/lib.rs:95  pub use hyperedge::
src/lib.rs:99  pub use hyperedge_suggest::
src/lib.rs:103  pub use path_rag::{EntityId, GraphPath, PathGraph, PathRAGConfig, PathRAGEngine}
src/lib.rs:105  pub use path_rag::{KPathConfig, KPathDiffusion, KPathResult}
src/lib.rs:107  pub use percolation::
src/lib.rs:112  pub use ppr::compute_ppr_with_apprh_selector
src/lib.rs:113  pub use ppr::{DeletedView, PprContext}
src/lib.rs:114  pub use ppr_stream::{PprCandidateStream, DEFAULT_PPR_STREAM_BATCH_SIZE}
src/lib.rs:115  pub use provenance::{DocEdgeIndex, EdgeProvenance}
src/lib.rs:116  pub use purger::{graph_purger, GraphPurger}
src/lib.rs:117  pub use provider::
src/lib.rs:121  pub use session_dag::
src/lib.rs:124  pub use tl_hfd::
src/lib.rs:132  pub trait GraphIndexExt: contextra_ports::GraphIndex
src/path_rag/k_path.rs:15  pub struct KPathConfig
src/path_rag/k_path.rs:37  pub struct KPathResult
src/path_rag/k_path.rs:46  pub trait KPathDiffusion: Send + Sync
src/path_rag/mod.rs:14  pub mod k_path
src/path_rag/mod.rs:17  pub use k_path::{KPathConfig, KPathDiffusion, KPathResult}
src/path_rag/mod.rs:19  pub mod snapshot
src/path_rag/mod.rs:20  pub use snapshot::SnapshotPathGraph
src/path_rag/mod.rs:22  pub use crate::hyperedge::{HyperEdge, HyperEdgeId, RoleBinding, RoleId}
src/path_rag/mod.rs:25  pub use contextra_types::EntityId
src/path_rag/mod.rs:33  pub struct GraphPath
src/path_rag/mod.rs:47  pub trait PathGraph: Send + Sync
src/path_rag/mod.rs:73  pub struct PprParams
src/path_rag/mod.rs:96  pub fn forward_push_ppr<G: PathGraph>(
src/path_rag/mod.rs:166  pub const DEFAULT_SUFFICIENCY_THRESHOLD: f64 = 0.1
src/path_rag/mod.rs:171  pub struct PathRAGConfig
src/path_rag/mod.rs:194  pub struct PathRAGEngine<G: PathGraph>
src/path_rag/mod.rs:201  pub fn new(graph: G, max_hops: usize, sufficiency_threshold: f64) -> Self
src/path_rag/mod.rs:214  pub fn with_defaults(graph: G) -> Self
src/path_rag/mod.rs:219  pub fn with_config(graph: G, config: PathRAGConfig) -> Self
src/path_rag/mod.rs:224  pub fn max_hops(&self) -> usize
src/path_rag/mod.rs:229  pub fn sufficiency_threshold(&self) -> f64
src/path_rag/mod.rs:279  pub fn find_path(&self, source: EntityId, target: EntityId) -> Option<GraphPath>
src/path_rag/mod.rs:411  pub fn find_all_paths(&self, source: EntityId) -> Vec<GraphPath>
src/path_rag/mod.rs:441  pub fn sufficiency_check(&self, path: &GraphPath) -> bool
src/path_rag/mod.rs:451  pub fn to_rrf_signal(&self, paths: &[GraphPath]) -> Vec<(DocId, f32)>
src/path_rag/snapshot.rs:21  pub struct SnapshotPathGraph<'a>
src/path_rag/snapshot.rs:29  pub fn new(inner: &'a GraphInner, as_of_tx: TxId) -> Self
src/percolation.rs:18  pub struct PercolationConfig
src/percolation.rs:41  pub fn compute_percolation_health(
src/percolation.rs:54  pub fn should_trigger_rebonding(health: f32, config: &PercolationConfig) -> bool
src/ppr/cost.rs:10  pub struct GraphStats
src/ppr/cost.rs:22  pub fn new(node_count: usize, edge_count: usize) -> Self
src/ppr/cost.rs:39  pub struct PprCostEstimate
src/ppr/cost.rs:51  pub fn cheapest_strategy(&self) -> PprAlgorithm
src/ppr/cost.rs:63  pub struct PprCostSample
src/ppr/cost.rs:76  pub struct PprCostCalibrator
src/ppr/cost.rs:83  pub fn new() -> Self
src/ppr/cost.rs:89  pub fn record(&self, sample: PprCostSample)
src/ppr/cost.rs:96  pub fn sample_count(&self) -> usize
src/ppr/cost.rs:105  pub fn recommended_multiplier(&self) -> Option<f64>
src/ppr/cost.rs:131  pub fn clear(&self)
src/ppr/cost.rs:142  pub fn estimate_ppr_cost(
src/ppr/mod.rs:6  pub mod cost
src/ppr/mod.rs:7  pub mod dense
src/ppr/mod.rs:8  pub mod forward_push
src/ppr/mod.rs:9  pub mod shadow_hook
src/ppr/mod.rs:11  pub mod types
src/ppr/mod.rs:16  pub use self::types::{DeletedView, PprContext}
src/ppr/mod.rs:32  pub fn compute_ppr_with_apprh_selector(
src/ppr/shadow_hook.rs:8  pub struct ShadowDiscrepancy<T>
src/ppr/shadow_hook.rs:18  pub trait ShadowSink<T>: Send + Sync
src/ppr/shadow_hook.rs:26  pub fn log_tl_hfd_vs_baseline_discrepancy(
src/ppr/types.rs:10  pub struct DeletedView
src/ppr/types.rs:28  pub fn contains(&self, idx: usize) -> bool
src/ppr/types.rs:34  pub fn len(&self) -> usize
src/ppr/types.rs:40  pub fn is_empty(&self) -> bool
src/ppr/types.rs:47  pub struct PprContext
src/ppr/types.rs:56  pub fn new() -> Self
src/ppr_stream.rs:19  pub const DEFAULT_PPR_STREAM_BATCH_SIZE: usize = 16
src/ppr_stream.rs:23  pub struct PprCandidateStream<'a, G: GraphIndex + ?Sized>
src/ppr_stream.rs:36  pub fn new(graph: &'a G, seeds: impl Into<Vec<EntityId>>, config: PprConfig) -> Self
src/ppr_stream.rs:50  pub fn with_batch_size(mut self, n: usize) -> Self
src/ppr_stream.rs:57  pub fn with_exclude_seeds(mut self, exclude: bool) -> Self
src/ppr_stream.rs:114  pub fn yielded(&self) -> usize
src/ppr_stream.rs:120  pub fn is_exhausted(&self) -> bool
src/ppr_stream.rs:129  pub const fn max_depth() -> usize
src/provenance.rs:23  pub struct EdgeProvenance
src/provenance.rs:34  pub fn new(edge_id: EdgeId, source_doc_ids: Vec<DocId>, created_at_tx: TxId) -> Self
src/provenance.rs:47  pub struct DocEdgeIndex
src/provenance.rs:53  pub fn new() -> Self
src/provenance.rs:61  pub fn record(&self, doc_id: DocId, edge_id: EdgeId)
src/provenance.rs:68  pub fn record_provenance(&self, provenance: &EdgeProvenance)
src/provenance.rs:77  pub fn edges_for_doc(&self, doc_id: DocId) -> Vec<EdgeId>
src/provenance.rs:87  pub fn remove_doc(&self, doc_id: DocId)
src/provider.rs:10  pub trait GraphProvider: Send + Sync
src/provider.rs:67  pub struct InMemoryGraphProvider
src/provider.rs:76  pub fn new() -> Self
src/provider.rs:79  pub fn add_node(&mut self, node: EntityId)
src/provider.rs:84  pub fn add_edge(&mut self, source: EntityId, target: EntityId, weight: f32)
src/provider.rs:100  pub fn add_hyperedge(&mut self, hyperedge: HyperEdge)
src/provider.rs:141  pub fn compute_pagerank<G: GraphProvider>(
src/provider.rs:206  pub fn detect_communities_on_provider<G: GraphProvider>(
src/purger.rs:24  pub struct GraphPurger
src/purger.rs:30  pub fn new(graph: Arc<CsrGraph>) -> Self
src/purger.rs:39  pub fn graph_purger(graph: Arc<CsrGraph>) -> Arc<dyn Purger>
src/session_dag.rs:31  pub struct NodesGuard<'a>
src/session_dag.rs:38  pub fn nodes(&self) -> &HashMap<NodeIdx, AgentStateNode>
src/session_dag.rs:45  pub fn edges(&self) -> RwLockReadGuard<'_, Vec<DagEdge>>
src/session_dag.rs:50  pub fn edges_write(&self) -> RwLockWriteGuard<'_, Vec<DagEdge>>
src/session_dag.rs:55  pub fn active_head(&self) -> RwLockReadGuard<'_, NodeIdx>
src/session_dag.rs:60  pub fn active_head_write(&self) -> RwLockWriteGuard<'_, NodeIdx>
src/session_dag.rs:68  pub struct NodesWriteGuard<'a>
src/session_dag.rs:75  pub fn nodes(&self) -> &HashMap<NodeIdx, AgentStateNode>
src/session_dag.rs:80  pub fn nodes_mut(&mut self) -> &mut HashMap<NodeIdx, AgentStateNode>
src/session_dag.rs:85  pub fn edges(&self) -> RwLockReadGuard<'_, Vec<DagEdge>>
src/session_dag.rs:90  pub fn edges_write(&self) -> RwLockWriteGuard<'_, Vec<DagEdge>>
src/session_dag.rs:95  pub fn active_head(&self) -> RwLockReadGuard<'_, NodeIdx>
src/session_dag.rs:100  pub fn active_head_write(&self) -> RwLockWriteGuard<'_, NodeIdx>
src/session_dag.rs:107  pub type NodeIdx = u64
src/session_dag.rs:112  pub struct AgentStateNode
src/session_dag.rs:129  pub struct DagEdge
src/session_dag.rs:138  pub const MAX_DAG_STRING_BYTES: usize = 10 * 1024 * 1024
src/session_dag.rs:142  pub const MAX_DAG_TRAVERSAL_DEPTH: usize = 10_000
src/session_dag.rs:147  pub struct SessionDagStepOptions<'a>
src/session_dag.rs:162  pub struct SessionBranchTree
src/session_dag.rs:172  pub fn new(root_prompt: String, root_response: String) -> Self
src/session_dag.rs:195  pub fn lock_nodes(&self) -> NodesGuard<'_>
src/session_dag.rs:205  pub fn lock_nodes_write(&self) -> NodesWriteGuard<'_>
src/session_dag.rs:266  pub fn append_step(
src/session_dag.rs:297  pub fn branch_from(
src/session_dag.rs:322  pub fn set_active_head(&self, node_idx: NodeIdx) -> Result<()>
src/session_dag.rs:338  pub fn path_to_head(&self) -> Vec<AgentStateNode>
src/session_dag.rs:379  pub fn children_of(&self, node_idx: NodeIdx) -> Vec<NodeIdx>
src/session_dag.rs:391  pub fn active_head(&self) -> NodeIdx
src/session_dag.rs:399  pub fn node_count(&self) -> usize
src/session_dag.rs:405  pub fn get_node(&self, node_idx: NodeIdx) -> Option<AgentStateNode>
src/storage.rs:7  pub type StorageFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>
src/storage.rs:10  pub trait GraphStorage: Send + Sync
src/storage.rs:29  pub struct InMemoryGraphStorage
src/storage.rs:35  pub fn new() -> Self
src/tl_hfd/diffusion.rs:28  pub fn run_diffusion<G: PathGraph>(
src/tl_hfd/error.rs:8  pub enum TlHfdError
src/tl_hfd/lovasz.rs:14  pub fn truncate_participants(
src/tl_hfd/lovasz.rs:56  pub fn compute_lovasz_extension(
src/tl_hfd/mod.rs:3  pub mod diffusion
src/tl_hfd/mod.rs:4  pub mod error
src/tl_hfd/mod.rs:5  pub mod lovasz
src/tl_hfd/mod.rs:6  pub mod params
src/tl_hfd/mod.rs:7  pub mod shadow
src/tl_hfd/mod.rs:9  pub use diffusion::run_diffusion
src/tl_hfd/mod.rs:10  pub use error::TlHfdError
src/tl_hfd/mod.rs:11  pub use lovasz::{compute_lovasz_extension, truncate_participants}
src/tl_hfd/mod.rs:12  pub use params::TlHfdParams
src/tl_hfd/mod.rs:13  pub use shadow::
src/tl_hfd/mod.rs:31  pub fn tl_hfd_local<G: PathGraph>(
src/tl_hfd/mod.rs:48  pub fn thresholded_local_hfd(
src/tl_hfd/params.rs:11  pub struct TlHfdParams
src/tl_hfd/params.rs:48  pub fn validate(&self) -> Result<(), TlHfdError>
src/tl_hfd/shadow.rs:13  pub const MIN_SHADOW_SAMPLES: u64 = 10_000
src/tl_hfd/shadow.rs:17  pub const MIN_AGREEMENT_THRESHOLD: f32 = 0.85
src/tl_hfd/shadow.rs:22  pub struct ShadowDiscrepancyReport
src/tl_hfd/shadow.rs:35  pub trait DefaultFlipGate
src/tl_hfd/shadow.rs:50  pub struct TlHfdFlipGate
src/tl_hfd/shadow.rs:57  pub struct ShadowComparison
src/tl_hfd/shadow.rs:79  pub fn shadow_compare_forward_push_vs_tl_hfd<G: PathGraph>(
src/types.rs:8  pub const MAX_SEARCH_K: usize = 1_000
src/types.rs:12  pub struct EntityId(pub u64)
src/types.rs:16  pub const fn new(id: u64) -> Self
src/types.rs:21  pub const fn inner(self) -> u64
src/types.rs:26  pub fn from_key(key: impl AsRef<[u8]>) -> Result<Self, std::num::ParseIntError>
src/types.rs:58  pub struct DocId(pub u64)
src/types.rs:62  pub const fn new(id: u64) -> Self
src/types.rs:67  pub const fn inner(self) -> u64
src/types.rs:72  pub fn from_key(key: impl AsRef<[u8]>) -> Result<Self, std::num::ParseIntError>
src/types.rs:104  pub struct TxId(pub u64)
src/types.rs:108  pub const fn new(id: u64) -> Self
src/types.rs:113  pub const fn inner(self) -> u64
src/types.rs:138  pub struct Entity
src/types.rs:149  pub fn new(id: EntityId, name: impl Into<String>, entity_type: impl Into<String>) -> Self
src/types.rs:160  pub struct Edge
src/types.rs:175  pub fn new(from: EntityId, to: EntityId, label: impl Into<String>) -> Self
src/types.rs:186  pub fn with_weight(mut self, weight: f32) -> Self
src/types.rs:192  pub fn with_source_doc_id(mut self, doc_id: DocId) -> Self
src/types.rs:200  pub struct HyperEdgeId(pub u64)
src/types.rs:204  pub const fn new(id: u64) -> Self
src/types.rs:209  pub const fn inner(self) -> u64
src/types.rs:222  pub struct RoleId(pub u32)
src/types.rs:226  pub const fn new(id: u32) -> Self
src/types.rs:231  pub const fn inner(self) -> u32
src/types.rs:244  pub struct RoleBinding
src/types.rs:253  pub const fn new(role: RoleId, entity: EntityId) -> Self
src/types.rs:260  pub struct TlHfdParams
src/types.rs:291  pub enum PprAlgorithm
src/types.rs:309  pub struct PprConfig
src/types.rs:336  pub struct ResourceTracker
src/types.rs:343  pub fn unlimited() -> Self
src/types.rs:351  pub fn with_limit(max_bytes: usize) -> Self
src/types.rs:359  pub fn record_allocation(&mut self, bytes: usize)
src/types.rs:364  pub fn is_exhausted(&self) -> bool
src/types.rs:371  pub struct RequestContext
src/types.rs:378  pub fn new() -> Self
src/types.rs:383  pub fn with_tenant(tenant: impl Into<String>) -> Self
```

## Innen: crate-interne Grenzen (`pub(crate)`, `pub(super)`, `pub(in …)`)

```rust
src/apprh/shadow.rs:119  pub(crate) fn normalize_and_sort(map: &ahash::AHashMap<EntityId, f32>) -> Vec<(EntityId, f32)>
src/csr.rs:4  pub(crate) mod graph_index
src/csr.rs:5  pub(crate) mod graph_persist
src/csr.rs:6  pub(crate) mod graph_read
src/csr.rs:7  pub(crate) mod graph_write
src/csr.rs:8  pub(crate) mod inner
src/csr.rs:9  pub(crate) mod path_graph
src/csr.rs:10  pub(crate) mod types
src/csr.rs:11  pub(crate) mod visibility
src/csr/graph_index/mod.rs:20  pub(crate) fn collect_entity_incident_edges(
src/csr/graph_index/mod.rs:73  pub(crate) fn validate_traversal_params(max_hops: usize) -> Result<()>
src/csr/graph_index/mutations.rs:87  pub(crate) fn add_entity_impl<'a>(
src/csr/graph_index/mutations.rs:108  pub(crate) fn add_edge_impl<'a>(
src/csr/graph_index/mutations.rs:186  pub(crate) fn commit_impl<'a>(graph: &'a CsrGraph, tx: TxId) -> BoxFuture<'a, Result<()>>
src/csr/graph_index/mutations.rs:204  pub(crate) fn remove_edge_impl<'a>(
src/csr/graph_index/traversal.rs:17  pub(crate) fn traverse_at_bitemporal_impl<'a>(
src/csr/graph_index/traversal.rs:45  pub(crate) fn traverse_impl<'a>(
src/csr/graph_write/mod.rs:124  pub(crate) fn inner_write(&self) -> InnerWriteGuard<'_>
src/csr/graph_write/tombstone.rs:24  pub(crate) fn tombstone_edges_direct(
src/csr/inner/compaction.rs:26  pub(crate) fn add_to_out_weight_sum(&mut self, node_idx: usize, weight: f32)
src/csr/inner/compaction.rs:38  pub(crate) fn recompute_node_out_weight_sum(&mut self, node_idx: usize)
src/csr/inner/estimate.rs:229  pub(crate) fn estimate_memory(&self, presized: bool) -> MemoryEstimate
src/csr/inner/estimate.rs:249  pub(crate) fn estimate_memory_bytes(&self) -> usize
src/csr/inner/estimate.rs:253  pub(crate) fn estimate_compaction_peak_bytes(&self) -> usize
src/csr/inner/mod.rs:25  pub(crate) id_map: HashMap<EntityId, InternalIndex>,
src/csr/inner/mod.rs:26  pub(crate) reverse_map: Vec<EntityId>,
src/csr/inner/mod.rs:27  pub(crate) entities: Vec<Entity>,
src/csr/inner/mod.rs:28  pub(crate) communities: HashMap<EntityId, u64>,
src/csr/inner/mod.rs:29  pub(crate) communities_loaded: bool,
src/csr/inner/mod.rs:31  pub(crate) offsets: Vec<usize>,
src/csr/inner/mod.rs:32  pub(crate) targets: Vec<InternalIndex>,
src/csr/inner/mod.rs:33  pub(crate) weights: Vec<f32>,
src/csr/inner/mod.rs:34  pub(crate) tx_valid_froms: Vec<TxId>,
src/csr/inner/mod.rs:35  pub(crate) tx_valid_tos: Vec<TxId>,
src/csr/inner/mod.rs:36  pub(crate) business_valid_froms: Vec<i64>,
src/csr/inner/mod.rs:37  pub(crate) business_valid_tos: Vec<i64>,
src/csr/inner/mod.rs:38  pub(crate) source_doc_ids: Vec<DocId>,
src/csr/inner/mod.rs:39  pub(crate) out_weight_sums: Vec<f32>,
src/csr/inner/mod.rs:41  pub(crate) doc_to_edges: ahash::AHashMap<DocId, HashSet<(EntityId, EntityId)>>,
src/csr/inner/mod.rs:43  pub(crate) staged_entities: ahash::AHashMap<(TxId, EntityId), Entity>,
src/csr/inner/mod.rs:44  pub(crate) staged_edges: ahash::AHashMap<(TxId, EntityId), Vec<StagedEdgePayload>>,
src/csr/inner/mod.rs:45  pub(crate) staged_removals: ahash::AHashMap<TxId, Vec<(EntityId, EntityId)>>,
src/csr/inner/mod.rs:46  pub(crate) pending_edges: HashMap<InternalIndex, Vec<EdgePayload>>,
src/csr/inner/mod.rs:47  pub(crate) tombstoned_edges: HashSet<(InternalIndex, InternalIndex)>,
src/csr/inner/mod.rs:48  pub(crate) pending_edge_count: usize,
src/csr/inner/mod.rs:49  pub(crate) is_dirty: bool,
src/csr/inner/mod.rs:51  pub(crate) edge_store: HashMap<EntityId, Vec<CsrEdge>>,
src/csr/inner/mod.rs:53  pub(crate) hyperedges: HashMap<crate::hyperedge::HyperEdgeId, Arc<crate::hyperedge::HyperEdge>>,
src/csr/inner/mod.rs:54  pub(crate) doc_to_hyperedges: ahash::AHashMap<DocId, HashSet<crate::hyperedge::HyperEdgeId>>,
src/csr/inner/mod.rs:55  pub(crate) hyperedge_index: ahash::AHashMap<EntityId, HashSet<crate::hyperedge::HyperEdgeId>>,
src/csr/inner/mod.rs:56  pub(crate) child_to_parents:
src/csr/inner/mod.rs:61  pub(crate) fn sentinel_entity() -> Entity
src/csr/inner/mod.rs:107  pub(crate) fn entity_at(&self, idx: usize) -> Option<&Entity>
src/csr/inner/mod.rs:112  pub(crate) fn tx_valid_from_at(&self, idx: usize) -> Option<TxId>
src/csr/inner/mod.rs:120  pub(crate) fn tx_valid_to_at(&self, idx: usize) -> Option<TxId>
src/csr/inner/mod.rs:128  pub(crate) fn business_valid_from_at(&self, idx: usize) -> Option<i64>
src/csr/inner/mod.rs:136  pub(crate) fn business_valid_to_at(&self, idx: usize) -> Option<i64>
src/csr/inner/mod.rs:144  pub(crate) fn source_doc_id_at(&self, idx: usize) -> Option<DocId>
src/csr/inner/mod.rs:152  pub(crate) struct InnerWriteGuard<'a>
src/csr/inner/mod.rs:153  pub(crate) guard: parking_lot::MutexGuard<'a, GraphInner>,
src/csr/inner/mod.rs:154  pub(crate) arc_swap: &'a ArcSwap<GraphInner>,
src/csr/inner/reinforcement.rs:17  pub(crate) fn find_edge_mut_by_entities(
src/csr/inner/reinforcement.rs:76  pub(crate) fn all_entity_ids(&self) -> Vec<EntityId>
src/csr/inner/reinforcement.rs:80  pub(crate) fn outgoing_edges_mut(&mut self, entity_id: EntityId) -> &mut [CsrEdge]
src/csr/inner/reinforcement.rs:116  pub(crate) fn sync_edge_reinforcement_weights(
src/csr/types.rs:9  pub(crate) const GRAPH_ENTITY_PREFIX: &[u8] = b"__graph:entity:"
src/csr/types.rs:10  pub(crate) const GRAPH_EDGE_PREFIX: &[u8] = b"__graph:edge:"
src/csr/types.rs:11  pub(crate) const GRAPH_ENTITY_DELETED_PREFIX: &[u8] = b"graph:entity:deleted:"
src/csr/types.rs:12  pub(crate) const GRAPH_COMMUNITY_PREFIX: &[u8] = b"__graph:community:"
src/csr/types.rs:99  pub(crate) type InternalIndex = usize
src/csr/types.rs:102  pub(crate) struct EdgePayload
src/csr/types.rs:103  pub(crate) target: InternalIndex,
src/csr/types.rs:104  pub(crate) weight: f32,
src/csr/types.rs:105  pub(crate) tx_valid_from: Option<TxId>,
src/csr/types.rs:106  pub(crate) tx_valid_to: Option<TxId>,
src/csr/types.rs:107  pub(crate) business_valid_from: Option<i64>,
src/csr/types.rs:108  pub(crate) business_valid_to: Option<i64>,
src/csr/types.rs:109  pub(crate) source_doc_id: Option<DocId>,
src/csr/types.rs:113  pub(crate) struct StagedEdgePayload
src/csr/types.rs:114  pub(crate) target: EntityId,
src/csr/types.rs:115  pub(crate) weight: f32,
src/csr/types.rs:116  pub(crate) tx_valid_from: Option<TxId>,
src/csr/types.rs:117  pub(crate) tx_valid_to: Option<TxId>,
src/csr/types.rs:118  pub(crate) business_valid_from: Option<i64>,
src/csr/types.rs:119  pub(crate) business_valid_to: Option<i64>,
src/csr/types.rs:120  pub(crate) source_doc_id: Option<DocId>,
src/ppr/dense.rs:12  pub(crate) fn collect_valid_seeds(
src/ppr/dense.rs:108  pub(crate) fn compute_out_weight_sums(
src/ppr/dense.rs:144  pub(crate) fn distribute_dense_ranks(
src/ppr/dense.rs:371  pub(crate) fn compute_ppr_dense(
src/ppr/forward_push.rs:13  pub(crate) fn push_mass_to_neighbors(
src/ppr/forward_push.rs:202  pub(crate) fn forward_push_ppr(
src/ppr/mod.rs:10  pub(crate) mod snapshot
src/ppr/mod.rs:20  pub(crate) fn compute_ppr(
src/ppr/mod.rs:70  pub(crate) fn select_auto_ppr_algorithm(
src/ppr/mod.rs:176  pub(crate) fn compute_ppr_with_context(
src/ppr/snapshot.rs:98  pub(crate) fn compute_ppr_at(
src/ppr/types.rs:15  pub(crate) fn from_nodes(deleted_nodes: HashSet<usize>) -> Self
src/ppr/types.rs:20  pub(crate) fn empty() -> Self
src/ppr/types.rs:48  pub(crate) valid_seeds: Vec<usize>,
src/ppr/types.rs:49  pub(crate) out_weight_sums: Vec<f32>,
src/ppr/types.rs:50  pub(crate) ranks: Vec<f32>,
src/ppr/types.rs:51  pub(crate) next_ranks: Vec<f32>,
src/ppr/types.rs:60  pub(crate) fn prepare(&mut self, n: usize)
```
