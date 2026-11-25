# API: contextra-ports

> Erzeugt aus dem Quellcode am 2026-11-25, Repository HEAD ce4579a56. Eine Zeile je Item: Datei:Zeile und die **erste Zeile** der Signatur. Mehrzeilige Signaturen sind nur angeschnitten. Nicht kompiliert, daher ohne Typprüfung. Testcode (`tests`, `testing`, `test_doubles`) ist ausgeschlossen.

Umfang: 322 öffentliche Items, 5 crate-interne Sichtbarkeitsgrenzen.

## Innen: Abhängigkeiten zu anderen Workspace-Crates

- contextra-types

## Außen: öffentliche Schnittstelle (`pub`)

```rust
src/attention.rs:7  pub struct RequestId(pub u64)
src/attention.rs:11  pub trait AttentionExporter: Send + Sync
src/barrier.rs:19  pub struct BarrierShared
src/barrier.rs:25  pub fn new(token: Option<Arc<()>>) -> Self
src/barrier.rs:37  pub struct BarrierExclusive
src/barrier.rs:43  pub fn new(token: Option<Arc<()>>) -> Self
src/barrier.rs:55  pub struct CompactionPermit
src/barrier.rs:61  pub fn new(token: Option<Arc<()>>) -> Self
src/barrier.rs:73  pub struct PurgePermit
src/barrier.rs:79  pub fn new(token: Option<Arc<()>>) -> Self
src/barrier.rs:91  pub trait CommitBarrier: Send + Sync
src/barrier.rs:112  pub trait PurgeBarrier: Send + Sync
src/checkpoint.rs:9  pub trait Checkpoint: Send + Sync + 'static
src/checkpoint.rs:26  pub trait CheckpointCoordinator: Send + Sync + 'static
src/clock.rs:8  pub trait Clock: Send + Sync + 'static
src/clock.rs:30  pub struct SystemClock
src/clock.rs:37  pub fn new() -> Self
src/conformance.rs:10  pub mod check
src/conformance.rs:38  pub fn assert_checkpoint_contract<T: Checkpoint>(t: &T, tx: TxId, state: &WorkflowState)
src/conformance.rs:72  pub fn assert_attention_exporter_contract<T: AttentionExporter>(t: &T, req: RequestId)
src/conformance.rs:77  pub fn assert_graph_index_contract<T: GraphIndex + CommunityResolver>(
src/conformance.rs:91  pub fn assert_lifecycle_contract<
src/conformance.rs:107  pub fn assert_embedding_contract<
src/conformance.rs:129  pub fn assert_observability_contract<T: DriftStatusProvider>(t: &T)
src/conformance.rs:134  pub fn assert_license_contract<T: LicenseGate>(t: &T, ring: FeatureRing)
src/conformance.rs:140  pub fn assert_reranker_contract<T: Reranker>(t: &T, results: &[RerankResult])
src/conformance.rs:147  pub fn assert_text_index_contract<T: TextIndex + SegmentSynthesizer>(t: &T)
src/conformance.rs:153  pub fn assert_storage_engine_contract<T: StorageEngine>(
src/conformance.rs:175  pub fn assert_purger_contract<T: Purger>(t: &T, ctx: &RequestContext, target: &ScanTarget)
src/conformance.rs:180  pub fn assert_vfs_file_contract<F: crate::vfs::VfsFile>(f: &F, buf: &mut [u8])
src/conformance.rs:188  pub fn assert_read_gate_contract<G: crate::gate::ReadGate>(
src/conformance.rs:204  pub fn assert_keys_contract<S: crate::keys::RecordSealer, K: crate::keys::FileKeyring>(
src/conformance.rs:217  pub fn assert_ledger_contract<L: crate::ledger::LedgerSink>(l: &L)
src/conformance.rs:223  pub fn assert_erasure_layer_contract<E: crate::erasure::ErasureLayer>(
src/conformance.rs:234  pub fn assert_egress_policy_contract<T: crate::policy::EgressPolicy>(
src/conformance.rs:243  pub fn assert_visitor_methods()
src/embedding.rs:7  pub struct ContextSegment<'a>
src/embedding.rs:20  pub fn new(chunk_id: u64, text: &'a str) -> Self
src/embedding.rs:30  pub fn with_fingerprint(mut self, fingerprint: &'a ModelFingerprint) -> Self
src/embedding.rs:36  pub fn with_rope_offset(mut self, offset: usize) -> Self
src/embedding.rs:43  pub trait TextEmbeddingEngine: Send + Sync + 'static
src/embedding.rs:65  pub enum EmbeddingError
src/embedding.rs:83  pub trait EmbeddingProvider: Send + Sync + 'static
src/embedding.rs:122  pub trait TextGenerator: Send + Sync + 'static
src/embedding.rs:128  pub trait LlmTextGenerator: Send + Sync + 'static
src/embedding.rs:153  pub trait LlmTextGeneratorStreaming: LlmTextGenerator
src/env.rs:6  pub use crate::clock::{Clock, SystemClock}
src/env.rs:7  pub use crate::id_gen::{IdGen, SequentialIdGen}
src/env.rs:8  pub use crate::metrics::
src/env.rs:11  pub use crate::rng::{Rng, SeededRng}
src/env.rs:14  pub mod prelude
src/env.rs:15  pub use super::{Clock, IdGen, MetricsSink, Rng}
src/erasure.rs:13  pub struct LayerId(pub [u8; 16])
src/erasure.rs:17  pub const fn new(raw: [u8; 16]) -> Self
src/erasure.rs:22  pub fn as_bytes(&self) -> &[u8; 16]
src/erasure.rs:44  pub enum EraseTarget
src/erasure.rs:70  pub struct LayerScope
src/erasure.rs:79  pub const fn empty() -> Self
src/erasure.rs:87  pub fn new(scope_id: [u8; 16], affected_units: Vec<u64>) -> Self
src/erasure.rs:95  pub fn scope_id(&self) -> &[u8; 16]
src/erasure.rs:100  pub fn affected_units(&self) -> &[u64]
src/erasure.rs:118  pub struct PurgeCx
src/erasure.rs:125  pub const fn new(deadline_ms: u64) -> Self
src/erasure.rs:132  pub struct PurgeOutcome
src/erasure.rs:141  pub const fn new(purged_count: u64, bytes_reclaimed: u64) -> Self
src/erasure.rs:151  pub struct Armed
src/erasure.rs:156  pub struct Scanner<State>
src/erasure.rs:162  pub const fn new() -> Self
src/erasure.rs:185  pub use crate::scanner::LayerWitness
src/erasure.rs:189  pub enum KeyCoverage
src/erasure.rs:206  pub trait ErasureLayer: Send + Sync
src/erasure.rs:269  pub trait StorageBackend: Send + Sync + 'static
src/erasure.rs:272  pub trait ErasureTarget: Send + Sync + 'static
src/fs.rs:6  pub use crate::vfs::{VerifiedVia, Vfs, VfsFile, VfsOpenOptions}
src/fs.rs:9  pub mod prelude
src/fs.rs:10  pub use super::{VerifiedVia, Vfs, VfsFile, VfsOpenOptions}
src/gate.rs:10  pub struct Seq(pub u64)
src/gate.rs:14  pub const fn new(val: u64) -> Self
src/gate.rs:19  pub const fn as_u64(&self) -> u64
src/gate.rs:26  pub struct KeyBuf(pub [u8; 16])
src/gate.rs:30  pub const fn new(raw: [u8; 16]) -> Self
src/gate.rs:35  pub fn as_bytes(&self) -> &[u8; 16]
src/gate.rs:49  pub enum KeyBound
src/gate.rs:60  pub const fn exact(key: KeyBuf) -> Self
src/gate.rs:77  pub struct KeyRange
src/gate.rs:86  pub const fn new(start: KeyBuf, end: KeyBuf) -> Self
src/gate.rs:91  pub fn contains(&self, key: &KeyBuf) -> bool
src/gate.rs:104  pub enum GateDecision
src/gate.rs:115  pub enum RangeGate
src/gate.rs:128  pub trait ReadGate: Send + Sync
src/graph.rs:3  pub use crate::graph_index::{GraphIndex, GraphIndexStats}
src/graph.rs:14  pub struct RoleId(pub u32)
src/graph.rs:19  pub const fn new(id: u32) -> Self
src/graph.rs:25  pub const fn inner(self) -> u32
src/graph.rs:44  pub struct RoleBinding
src/graph.rs:53  pub fn new(role: RoleId, entity: EntityId) -> Self
src/graph.rs:63  pub struct HyperEdgeId(pub u64)
src/graph.rs:68  pub const fn new(id: u64) -> Self
src/graph.rs:74  pub const fn inner(self) -> u64
src/graph.rs:93  pub enum GraphMutationError
src/graph.rs:149  pub trait GraphCollectionMutation: Send + Sync + 'static
src/graph_index.rs:49  pub trait GraphIndex: Send + Sync + 'static
src/graph_index.rs:304  pub struct GraphIndexStats
src/graph_index.rs:315  pub trait CommunityResolver: Send + Sync
src/id_gen.rs:7  pub trait IdGen: Send + Sync + 'static
src/id_gen.rs:21  pub struct SequentialIdGen
src/id_gen.rs:27  pub fn new(start: u64) -> Self
src/index.rs:3  pub use crate::graph::{GraphCollectionMutation, GraphMutationError}
src/index.rs:4  pub use crate::graph_index::{CommunityResolver, GraphIndex, GraphIndexStats}
src/index.rs:5  pub use crate::text_index::{SegmentSynthesizer, TextIndex, TextIndexStats}
src/index.rs:6  pub use crate::vector_index::{HybridSearchProvider, VectorIndex, VectorIndexStats}
src/index.rs:9  pub mod prelude
src/index.rs:10  pub use super::{GraphCollectionMutation, GraphIndex, TextIndex, VectorIndex}
src/inference.rs:3  pub use crate::attention::{AttentionExporter, RequestId}
src/inference.rs:4  pub use crate::embedding::
src/inference.rs:8  pub use crate::query_rewriter::QueryRewriter
src/inference.rs:9  pub use crate::reranker::Reranker
src/inference.rs:12  pub mod prelude
src/inference.rs:13  pub use super::
src/keys.rs:12  pub struct FileId(pub u64)
src/keys.rs:16  pub const fn new(id: u64) -> Self
src/keys.rs:21  pub const fn as_u64(&self) -> u64
src/keys.rs:28  pub enum FileKind
src/keys.rs:42  pub struct FileKey
src/keys.rs:49  pub fn new(raw: Vec<u8>, file_id: FileId) -> Self
src/keys.rs:54  pub fn as_bytes(&self) -> &[u8]
src/keys.rs:59  pub fn file_id(&self) -> FileId
src/keys.rs:84  pub struct SealCtx
src/keys.rs:91  pub const fn new(epoch: u64) -> Self
src/keys.rs:104  pub struct Envelope
src/keys.rs:110  pub fn new(payload: Vec<u8>) -> Self
src/keys.rs:115  pub fn payload(&self) -> &[u8]
src/keys.rs:128  pub trait RecordSealer: Send + Sync
src/keys.rs:177  pub trait FileKeyring: Send + Sync
src/kv.rs:11  pub struct KvLayout
src/kv.rs:24  pub struct RopeConfig
src/kv.rs:44  pub struct PrefixKey
src/kv.rs:115  pub struct KvBlock
src/kv.rs:128  pub struct KvPrefixHit
src/kv.rs:139  pub trait KvPrefixStore: Send + Sync + 'static
src/kv_bridge_port.rs:14  pub trait KvBridgeStorage: Send + Sync
src/ledger.rs:11  pub struct Timestamp(pub u64)
src/ledger.rs:15  pub const fn new(val: u64) -> Self
src/ledger.rs:20  pub const fn as_u64(&self) -> u64
src/ledger.rs:27  pub struct RangeSet
src/ledger.rs:33  pub const fn empty() -> Self
src/ledger.rs:38  pub fn new(ranges: Vec<(KeyBuf, KeyBuf)>) -> Self
src/ledger.rs:43  pub fn ranges(&self) -> &[(KeyBuf, KeyBuf)]
src/ledger.rs:59  pub struct LedgerEntry
src/ledger.rs:76  pub const fn new(
src/ledger.rs:95  pub fn compute_chained(
src/ledger.rs:125  pub fn verify_chain_hash(&self, prev_hash: &[u8; 32]) -> bool
src/ledger.rs:152  pub trait LedgerSink: Send + Sync
src/lib.rs:23  pub use contextra_types::
src/lib.rs:33  pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>
src/lib.rs:36  pub type BoxStream<'a, T> = Pin<Box<dyn futures_util::stream::Stream<Item = T> + Send + 'a>>
src/lib.rs:39  pub mod attention
src/lib.rs:41  pub mod checkpoint
src/lib.rs:43  pub mod clock
src/lib.rs:45  pub mod conformance
src/lib.rs:47  pub mod embedding
src/lib.rs:49  pub mod graph
src/lib.rs:51  pub mod graph_index
src/lib.rs:53  pub mod id_gen
src/lib.rs:55  pub mod kv
src/lib.rs:57  pub mod kv_bridge_port
src/lib.rs:59  pub mod license
src/lib.rs:61  pub mod lifecycle
src/lib.rs:63  pub mod metrics
src/lib.rs:65  pub mod observability
src/lib.rs:67  pub mod plugin
src/lib.rs:69  pub mod query_rewriter
src/lib.rs:71  pub mod reranker
src/lib.rs:73  pub mod rng
src/lib.rs:75  pub mod scanner
src/lib.rs:77  pub mod storage
src/lib.rs:79  pub mod text_index
src/lib.rs:81  pub mod vector_index
src/lib.rs:83  pub mod vfs
src/lib.rs:86  pub mod env
src/lib.rs:88  pub mod fs
src/lib.rs:90  pub mod index
src/lib.rs:92  pub mod inference
src/lib.rs:94  pub mod policy
src/lib.rs:96  pub use attention::*
src/lib.rs:97  pub use checkpoint::*
src/lib.rs:98  pub use clock::*
src/lib.rs:99  pub use conformance::*
src/lib.rs:100  pub use embedding::*
src/lib.rs:101  pub use graph::*
src/lib.rs:102  pub use graph_index::*
src/lib.rs:103  pub use id_gen::*
src/lib.rs:104  pub use kv::*
src/lib.rs:105  pub use kv_bridge_port::*
src/lib.rs:106  pub use license::*
src/lib.rs:107  pub use lifecycle::*
src/lib.rs:108  pub use metrics::*
src/lib.rs:109  pub use observability::*
src/lib.rs:110  pub use plugin::*
src/lib.rs:111  pub use query_rewriter::*
src/lib.rs:112  pub use reranker::*
src/lib.rs:113  pub use rng::*
src/lib.rs:114  pub use scanner::
src/lib.rs:118  pub use storage::*
src/lib.rs:119  pub use text_index::*
src/lib.rs:120  pub use vector_index::*
src/lib.rs:121  pub use vfs::*
src/lib.rs:132  pub mod contract_tests
src/lib.rs:134  pub use metrics::
src/lib.rs:185  pub mod barrier
src/lib.rs:186  pub mod erasure
src/lib.rs:187  pub mod gate
src/lib.rs:188  pub mod keys
src/lib.rs:189  pub mod ledger
src/lib.rs:190  pub mod observer
src/lib.rs:191  pub mod report_signer
src/lib.rs:193  pub use barrier::
src/lib.rs:196  pub use erasure::
src/lib.rs:200  pub use gate::{GateDecision, KeyBound, KeyRange, RangeGate, ReadGate}
src/lib.rs:201  pub use keys::{Envelope, FileKey, FileKeyring, FileKind, RecordSealer, SealCtx}
src/lib.rs:202  pub use ledger::{LedgerEntry, LedgerSink, RangeSet}
src/lib.rs:203  pub use observer::{ErasureEvent, ErasureObserver, ObservedPhase, TargetKind}
src/lib.rs:204  pub use report_signer::ReportSigner
src/license.rs:6  pub enum VectorDeleteMode
src/license.rs:22  pub enum FeatureRing
src/license.rs:36  pub struct AuthorizedRing
src/license.rs:47  pub fn ring(&self) -> FeatureRing
src/license.rs:55  pub enum LicenseError
src/license.rs:70  pub trait LicenseGate: Send + Sync
src/license.rs:93  pub struct OpenFastGate
src/lifecycle.rs:14  pub struct GroundingAssessment
src/lifecycle.rs:24  pub trait GroundingValidator: Send + Sync
src/lifecycle.rs:38  pub trait ResponseGroundingValidator: Send + Sync
src/lifecycle.rs:51  pub trait ContextPreparer: Send + Sync
src/metrics.rs:19  pub trait MetricsSink: Send + Sync + 'static
src/metrics.rs:47  pub struct NoopMetricsSink
src/metrics.rs:59  pub struct MetricKey
src/metrics.rs:68  pub fn new(name: impl Into<String>, labels: &[(&str, &str)]) -> Self
src/metrics.rs:83  pub struct HistogramSnapshot
src/metrics.rs:99  pub struct MetricSnapshot
src/metrics.rs:183  pub struct InMemoryMetricsSink
src/metrics.rs:191  pub fn new() -> Self
src/metrics.rs:196  pub fn snapshot(&self) -> MetricSnapshot
src/observability.rs:7  pub use super::lifecycle::{GroundingAssessment, GroundingValidator, ResponseGroundingValidator}
src/observability.rs:10  pub trait DriftStatusProvider: Send + Sync
src/observer.rs:15  pub type Timestamp = u64
src/observer.rs:19  pub enum TargetKind
src/observer.rs:36  pub enum ObservedPhase
src/observer.rs:49  pub struct ErasureEvent
src/observer.rs:62  pub enum ObserverError
src/observer.rs:79  pub trait ErasureObserver: Send + Sync + 'static
src/plugin.rs:11  pub struct PluginCapability
src/plugin.rs:24  pub fn new(
src/plugin.rs:40  pub trait PluginManifest: Send + Sync
src/plugin.rs:98  pub enum PluginError
src/plugin.rs:119  pub struct PluginRegistry
src/plugin.rs:127  pub fn new(gate: Arc<dyn LicenseGate>) -> Self
src/plugin.rs:136  pub fn is_active(&self, name: &str) -> bool
src/plugin.rs:141  pub fn snapshot(&self) -> Vec<PluginCapability>
src/plugin.rs:146  pub fn current_feature_ring(&self) -> FeatureRing
src/plugin.rs:163  pub fn activate_all(
src/policy.rs:3  pub use crate::license::
src/policy.rs:9  pub enum EgressDecision
src/policy.rs:19  pub trait EgressPolicy: Send + Sync
src/policy.rs:30  pub mod prelude
src/policy.rs:31  pub use super::
src/query_rewriter.rs:7  pub trait QueryRewriter: Send + Sync
src/report_signer.rs:15  pub type SignerId = u8
src/report_signer.rs:19  pub struct ReportSignature
src/report_signer.rs:30  pub enum SignerError
src/report_signer.rs:45  pub trait ReportSigner: Send + Sync
src/reranker.rs:7  pub trait Reranker: Send + Sync
src/rng.rs:12  pub trait Rng: Send + Sync + 'static
src/rng.rs:49  pub struct SeededRng
src/rng.rs:57  pub fn new(seed: u64) -> Self
src/rng.rs:74  pub fn from_entropy() -> Self
src/scanner/layer.rs:13  pub enum WitnessLayer
src/scanner/layer.rs:34  pub fn as_str(&self) -> &str
src/scanner/layer.rs:71  pub enum ScanTechnique
src/scanner/layer.rs:88  pub struct ScanTarget
src/scanner/layer.rs:97  pub fn new(target_id: impl Into<String>, target_hash: [u8; 32]) -> Self
src/scanner/layer.rs:107  pub struct PurgeStats
src/scanner/layer.rs:115  pub trait Purger: Send + Sync
src/scanner/mod.rs:9  pub mod layer
src/scanner/mod.rs:11  pub mod session
src/scanner/mod.rs:13  pub mod witness
src/scanner/mod.rs:15  pub use layer::*
src/scanner/mod.rs:16  pub use session::*
src/scanner/mod.rs:17  pub use witness::*
src/scanner/mod.rs:52  pub trait LayerScanner: Send + Sync
src/scanner/session.rs:15  pub struct ScanSession
src/scanner/session.rs:40  pub fn record_unit(&mut self, unit_hash: [u8; 32], is_live_residual: bool)
src/scanner/session.rs:49  pub fn record_control_match(&mut self)
src/scanner/witness.rs:17  pub struct LayerWitness
src/scanner/witness.rs:57  pub fn layer(&self) -> &str
src/scanner/witness.rs:62  pub fn witness_layer(&self) -> &WitnessLayer
src/scanner/witness.rs:67  pub fn scanned(&self) -> u64
src/scanner/witness.rs:72  pub fn remaining(&self) -> u64
src/scanner/witness.rs:77  pub fn technique(&self) -> &ScanTechnique
src/scanner/witness.rs:82  pub fn method(&self) -> &ScanMethod
src/scanner/witness.rs:87  pub fn digest(&self) -> &[u8; 32]
src/scanner/witness.rs:92  pub fn measured_at(&self) -> u64
src/scanner/witness.rs:99  pub fn mint_witness_for_testkit(
src/storage.rs:11  pub struct StorageStats
src/storage.rs:29  pub const MAX_SCAN_MERGE_ACCUMULATOR: usize = 100_000
src/storage.rs:35  pub trait StorageEngine: Send + Sync + 'static
src/storage.rs:268  pub mod prelude
src/storage.rs:269  pub use super::{StorageEngine, StorageStats}
src/storage.rs:270  pub use crate::kv::KvPrefixStore
src/text_index.rs:13  pub use crate::embedding::TextEmbeddingEngine
src/text_index.rs:16  pub trait SegmentSynthesizer: Send + Sync
src/text_index.rs:33  pub struct TextIndexStats
src/text_index.rs:46  pub trait TextIndex: Send + Sync + 'static
src/vector_index.rs:15  pub struct VectorIndexStats
src/vector_index.rs:37  pub trait VectorIndex: Send + Sync + 'static
src/vector_index.rs:215  pub trait HybridSearchProvider: Send + Sync
src/vfs.rs:30  pub enum VerifiedVia
src/vfs.rs:41  pub struct VfsOpenOptions
src/vfs.rs:58  pub fn read_only() -> Self
src/vfs.rs:66  pub fn write_create() -> Self
src/vfs.rs:76  pub trait VfsFile: Send + Sync
src/vfs.rs:165  pub trait Vfs: Send + Sync + 'static
```

## Innen: crate-interne Grenzen (`pub(crate)`, `pub(super)`, `pub(in …)`)

```rust
src/kv.rs:73  pub(crate) fn deserialize_bytes<'de, D>(deserializer: D) -> Result<Bytes, D::Error>
src/license.rs:42  pub(crate) fn new(ring: FeatureRing) -> Self
src/scanner/session.rs:27  pub(crate) fn begin(layer: WitnessLayer, target: ScanTarget, technique: ScanTechnique) -> Self
src/scanner/session.rs:59  pub(crate) fn finish(mut self, clock: &dyn Clock) -> Result<LayerWitness>
src/scanner/witness.rs:29  pub(crate) fn new(
```
