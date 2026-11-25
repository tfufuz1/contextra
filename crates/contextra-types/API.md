# API: contextra-types

> Erzeugt aus dem Quellcode am 2026-11-25, Repository HEAD ce4579a56. Eine Zeile je Item: Datei:Zeile und die **erste Zeile** der Signatur. Mehrzeilige Signaturen sind nur angeschnitten. Nicht kompiliert, daher ohne Typprüfung. Testcode (`tests`, `testing`, `test_doubles`) ist ausgeschlossen.

Umfang: 325 öffentliche Items, 0 crate-interne Sichtbarkeitsgrenzen.

## Innen: Abhängigkeiten zu anderen Workspace-Crates

- keine

## Außen: öffentliche Schnittstelle (`pub`)

```rust
src/erasure.rs:16  pub enum EraseTarget
src/erasure.rs:49  pub enum Timing
src/erasure.rs:58  pub enum EraseMode
src/erasure.rs:67  pub enum RetentionCode
src/erasure.rs:84  pub struct RetentionAttestation
src/erasure.rs:95  pub struct EraseRequest
src/erasure.rs:108  pub struct EraseReceipt
src/erasure.rs:119  pub struct LayerId(pub [u8; 16])
src/erasure.rs:123  pub const fn new(bytes: [u8; 16]) -> Self
src/erasure.rs:128  pub const fn as_bytes(&self) -> &[u8; 16]
src/erasure.rs:145  pub enum PendingReason
src/erasure.rs:160  pub enum RefusalReason
src/erasure.rs:169  pub enum Phase
src/erasure.rs:190  pub enum ErasureStatus
src/erasure.rs:220  pub enum KeyCoverage
src/erasure.rs:231  pub enum GateVariant
src/erasure.rs:240  pub enum MeasureClass
src/erasure.rs:251  pub enum ScanMethod
src/erasure.rs:262  pub const fn class(&self) -> MeasureClass
src/erasure.rs:273  pub enum RiskCondition
src/erasure.rs:284  pub enum ResidualRisk
src/erasure.rs:319  pub fn id(&self) -> &'static str
src/erasure.rs:340  pub fn condition(&self) -> RiskCondition
src/erasure.rs:354  pub struct Unarmed
src/erasure.rs:357  pub struct Armed(())
src/erasure.rs:361  pub struct ControlEvidence(())
src/erasure.rs:366  pub const fn observed() -> Self
src/erasure.rs:373  pub struct Residue
src/erasure.rs:392  pub struct Scanner<S>
src/erasure.rs:403  pub fn new(layer: LayerId, class: MeasureClass, method_id: u16) -> Self
src/erasure.rs:415  pub fn arm(self, _ev: ControlEvidence) -> Scanner<Armed>
src/erasure.rs:429  pub fn record(&mut self, files: u64, bytes: u64)
src/erasure.rs:437  pub fn finish(
src/erasure.rs:462  pub struct LayerWitness
src/erasure.rs:475  pub fn layer(&self) -> LayerId
src/erasure.rs:481  pub fn class(&self) -> MeasureClass
src/erasure.rs:487  pub fn method_id(&self) -> u16
src/erasure.rs:493  pub fn files(&self) -> u64
src/erasure.rs:499  pub fn bytes(&self) -> u64
src/erasure.rs:505  pub fn duration(&self) -> Duration
src/erasure.rs:511  pub fn hash(&self) -> &[u8; 32]
src/erasure.rs:517  pub const fn matches(&self) -> u64
src/error.rs:38  pub enum HnswDeletionError
src/error.rs:61  pub struct StepId(pub u64)
src/error.rs:65  pub const fn new(id: u64) -> Self
src/error.rs:70  pub const fn inner(self) -> u64
src/error.rs:90  pub enum CacheDirective
src/error.rs:115  pub type Result<T> = std::result::Result<T, ContextraError>
src/error.rs:129  pub enum ContextraError
src/error.rs:392  pub fn limit_exceeded(limit: usize, context: impl Into<String>) -> Self
src/error.rs:401  pub fn model_load(path: impl Into<String>, reason: impl Into<String>) -> Self
src/error.rs:410  pub fn snapshot_unsupported_for_signal(strategy: impl Into<String>) -> Self
src/error.rs:416  pub fn orphaned_vector_reference(
src/error.rs:428  pub fn capability_unsupported(
src/error.rs:439  pub fn invalid_input(msg: impl Into<String>) -> Self
src/error.rs:445  pub fn wal_corruption(offset: u64, reason: impl Into<String>) -> Self
src/error.rs:454  pub fn checksum_mismatch(path: impl Into<String>, block_id: u64) -> Self
src/error.rs:463  pub fn kv_quantization(msg: impl Into<String>) -> Self
src/error.rs:469  pub fn durability_config(msg: impl Into<String>) -> Self
src/error.rs:475  pub fn pin_budget_exceeded(msg: impl Into<String>) -> Self
src/error.rs:481  pub fn cross_device_link(
src/error.rs:493  pub fn wal_truncation_detected(expected_hmac: [u8; 32], actual_hmac: [u8; 32]) -> Self
src/error.rs:502  pub fn is_occ_conflict(&self) -> bool
src/error_dto.rs:32  pub struct ContextraErrorDto
src/error_dto.rs:44  pub fn new(kind: impl Into<String>, message: impl Into<String>) -> Self
src/error_dto.rs:54  pub fn with_details(
src/ids.rs:10  pub const MAX_REF_LEN: usize = 256
src/ids.rs:16  pub struct $name(Vec<u8>)
src/ids.rs:20  pub fn try_new(bytes: Vec<u8>) -> Result<Self>
src/ids.rs:32  pub fn as_bytes(&self) -> &[u8]
src/ids.rs:38  pub fn into_inner(self) -> Vec<u8>
src/ids.rs:64  pub struct DocRef
src/ids.rs:73  pub struct ErasureId(pub [u8; 16])
src/ids.rs:77  pub const fn new(bytes: [u8; 16]) -> Self
src/ids.rs:82  pub const fn as_bytes(&self) -> &[u8; 16]
src/ids.rs:99  pub struct FileId(pub [u8; 16])
src/ids.rs:103  pub const fn new(bytes: [u8; 16]) -> Self
src/ids.rs:108  pub const fn as_bytes(&self) -> &[u8; 16]
src/ids.rs:125  pub struct StoreId(pub [u8; 16])
src/ids.rs:129  pub const fn new(bytes: [u8; 16]) -> Self
src/ids.rs:134  pub const fn as_bytes(&self) -> &[u8; 16]
src/ids.rs:151  pub struct GateId(pub [u8; 16])
src/ids.rs:155  pub const fn new(bytes: [u8; 16]) -> Self
src/ids.rs:160  pub const fn as_bytes(&self) -> &[u8; 16]
src/ids.rs:177  pub struct Seq(pub u64)
src/ids.rs:181  pub const fn new(val: u64) -> Self
src/ids.rs:186  pub const fn as_u64(&self) -> u64
src/ids.rs:193  pub struct Timestamp(pub u64)
src/ids.rs:197  pub const fn from_micros(micros: u64) -> Self
src/ids.rs:202  pub const fn as_micros(&self) -> u64
src/lib.rs:36  pub mod error
src/lib.rs:39  pub mod error_dto
src/lib.rs:42  pub mod model_fingerprint
src/lib.rs:45  pub mod retrieval_strategy
src/lib.rs:48  pub mod schema
src/lib.rs:51  pub mod tenant_scope
src/lib.rs:54  pub mod tombstone
src/lib.rs:57  pub mod types
src/lib.rs:60  pub mod erasure
src/lib.rs:61  pub mod ids
src/lib.rs:62  pub mod refs
src/lib.rs:63  pub mod witness
src/lib.rs:66  pub use erasure::*
src/lib.rs:68  pub use ids::{ErasureId, FileId, GateId, Seq, StoreId, Timestamp}
src/lib.rs:69  pub use refs::{CollectionRef, DocKey, OperatorRef, SubjectRef, TenantRef, MAX_REF_LEN}
src/lib.rs:72  pub use error::{ContextraError, HnswDeletionError, Result}
src/lib.rs:73  pub use error_dto::ContextraErrorDto
src/lib.rs:74  pub use model_fingerprint::ModelFingerprint
src/lib.rs:75  pub use retrieval_strategy::RetrievalStrategy
src/lib.rs:76  pub use schema::{DocIdWidth, ManifestSchemaVersion}
src/lib.rs:77  pub use tenant_scope::{TenantScopeViolation, TenantScoped}
src/lib.rs:78  pub use tombstone::{SeqBitTombstone, TombstoneSemanticsCheck}
src/lib.rs:79  pub use types::*
src/lib.rs:81  pub use witness::{RequestContext, SagaRecord, ScanMethod}
src/model_fingerprint.rs:33  pub struct ModelFingerprint
src/model_fingerprint.rs:48  pub fn new(
src/refs.rs:3  pub use crate::ids::
src/refs.rs:6  pub use crate::tenant_scope::TenantScoped
src/retrieval_strategy.rs:22  pub enum RetrievalStrategy
src/schema.rs:19  pub enum DocIdWidth
src/schema.rs:32  pub fn current() -> Self
src/schema.rs:42  pub fn bytes_len(self) -> usize
src/schema.rs:58  pub enum ManifestSchemaVersion
src/schema.rs:69  pub fn doc_id_width(self) -> DocIdWidth
src/schema.rs:78  pub fn as_u8(self) -> u8
src/schema.rs:87  pub fn from_u8(version_byte: u8) -> Result<Self>
src/schema.rs:99  pub fn is_compatible_with_current_build(self) -> bool
src/tenant_scope.rs:20  pub struct TenantScoped<T>
src/tenant_scope.rs:32  pub fn new(tenant_id: TenantId, value: T) -> Self
src/tenant_scope.rs:39  pub fn tenant_id(&self) -> &TenantId
src/tenant_scope.rs:51  pub fn into_inner_checked(self, expected: &TenantId) -> Result<T, TenantScopeViolation>
src/tenant_scope.rs:67  pub fn map<U>(self, f: impl FnOnce(T) -> U) -> TenantScoped<U>
src/tenant_scope.rs:79  pub enum TenantScopeViolation
src/tombstone.rs:37  pub trait TombstoneSemanticsCheck
src/tombstone.rs:52  pub struct SeqBitTombstone
src/types.rs:15  pub mod budget
src/types.rs:19  pub mod domain
src/types.rs:23  pub mod filter
src/types.rs:27  pub mod importance
src/types.rs:31  pub mod saos
src/types.rs:33  pub use budget::*
src/types.rs:34  pub use domain::*
src/types.rs:35  pub use filter::*
src/types.rs:36  pub use importance::*
src/types.rs:37  pub use saos::*
src/types/budget.rs:39  pub enum BudgetStrategy
src/types/budget.rs:51  pub struct TokenBudget
src/types/budget.rs:131  pub struct Reservation<'a>
src/types/budget.rs:139  pub fn new(max_tokens: usize, reserve_tokens: usize) -> Self
src/types/budget.rs:154  pub fn for_model(model: &str) -> Self
src/types/budget.rs:180  pub fn with_reserved(mut self, system_tokens: usize, answer_tokens: usize) -> Self
src/types/budget.rs:186  pub fn with_strategy(mut self, strategy: BudgetStrategy) -> Self
src/types/budget.rs:192  pub fn effective_limit(&self) -> usize
src/types/budget.rs:201  pub fn available(&self) -> usize
src/types/budget.rs:208  pub fn consume(&self, tokens: usize)
src/types/budget.rs:224  pub fn reserve(&self, amount: usize) -> Result<Reservation<'_>>
src/types/budget.rs:262  pub fn try_reserve(&self, tokens: usize) -> Result<()>
src/types/budget.rs:275  pub fn refund(&self, tokens: usize)
src/types/budget.rs:290  pub fn consumed(&self) -> usize
src/types/budget.rs:301  pub fn settle(self)
src/types/budget.rs:341  pub struct ResourceBudget
src/types/budget.rs:357  pub struct ResourceTracker
src/types/budget.rs:372  pub fn new(budget: ResourceBudget) -> Self
src/types/budget.rs:384  pub fn consume_memory(&self, bytes: u64) -> Result<()>
src/types/budget.rs:413  pub fn release_memory(&self, bytes: u64)
src/types/budget.rs:437  pub fn memory_used(&self) -> u64
src/types/budget.rs:442  pub fn budget(&self) -> &ResourceBudget
src/types/budget.rs:447  pub fn has_memory_capacity(&self) -> bool
src/types/domain/document.rs:20  pub enum DistanceMetric
src/types/domain/document.rs:32  pub fn compute(&self, a: &[f32], b: &[f32]) -> Result<f32>
src/types/domain/document.rs:114  pub fn compute_u8(&self, a: &[u8], b: &[u8]) -> Result<u32>
src/types/domain/document.rs:174  pub struct Embedding
src/types/domain/document.rs:181  pub fn new(data: Vec<f32>) -> Self
src/types/domain/document.rs:187  pub fn dim(&self) -> usize
src/types/domain/document.rs:193  pub fn as_slice(&self) -> &[f32]
src/types/domain/document.rs:198  pub fn l2_norm(&self) -> f32
src/types/domain/document.rs:207  pub fn normalize(&self) -> Self
src/types/domain/document.rs:221  pub struct ScoredDocument
src/types/domain/document.rs:230  pub fn new(doc_id: DocId, score: f32) -> Self
src/types/domain/document.rs:238  pub struct RerankResult
src/types/domain/ids.rs:32  pub struct TenantId(pub u64)
src/types/domain/ids.rs:36  pub const SYSTEM: Self = Self(0)
src/types/domain/ids.rs:39  pub const DEFAULT: Self = Self(0)
src/types/domain/ids.rs:42  pub const INVALID: Self = Self(0)
src/types/domain/ids.rs:57  pub const fn new(id: u64) -> Self
src/types/domain/ids.rs:62  pub fn try_new(id: u64) -> Result<Self>
src/types/domain/ids.rs:74  pub const fn inner(self) -> u64
src/types/domain/ids.rs:80  pub const fn as_u64(self) -> u64
src/types/domain/ids.rs:86  pub fn is_system(self) -> bool
src/types/domain/ids.rs:115  pub struct CollectionId(pub u64)
src/types/domain/ids.rs:119  pub const INVALID: Self = Self(0)
src/types/domain/ids.rs:123  pub const fn new(id: u64) -> Self
src/types/domain/ids.rs:128  pub fn try_new(id: u64) -> Result<Self>
src/types/domain/ids.rs:140  pub const fn inner(self) -> u64
src/types/domain/ids.rs:147  pub const fn as_u64(self) -> u64
src/types/domain/ids.rs:173  pub struct DocId(pub u64)
src/types/domain/ids.rs:180  pub struct DocId(pub u128)
src/types/domain/ids.rs:185  pub const MAX: Self = Self(u64::MAX)
src/types/domain/ids.rs:192  pub const MIN: Self = Self(0)
src/types/domain/ids.rs:196  pub const fn new(id: u64) -> Self
src/types/domain/ids.rs:202  pub const fn inner(self) -> u64
src/types/domain/ids.rs:209  pub const fn as_u64(self) -> u64
src/types/domain/ids.rs:225  pub fn from_key(key: &str) -> Result<Self>
src/types/domain/ids.rs:238  pub const MAX: Self = Self(u128::MAX)
src/types/domain/ids.rs:240  pub const MIN: Self = Self(0)
src/types/domain/ids.rs:244  pub const fn new(id: u128) -> Self
src/types/domain/ids.rs:250  pub const fn inner(self) -> u128
src/types/domain/ids.rs:257  pub const fn as_u64(self) -> u64
src/types/domain/ids.rs:267  pub fn from_key(key: &str) -> Result<Self>
src/types/domain/ids.rs:300  pub struct EntityId(pub u64)
src/types/domain/ids.rs:305  pub const fn new(id: u64) -> Self
src/types/domain/ids.rs:311  pub const fn inner(self) -> u64
src/types/domain/ids.rs:318  pub const fn as_u64(self) -> u64
src/types/domain/ids.rs:323  pub fn as_bytes(&self) -> Vec<u8>
src/types/domain/ids.rs:329  pub fn from_doc_id(doc_id: DocId) -> Self
src/types/domain/ids.rs:335  pub fn from_doc_id(doc_id: DocId) -> Self
src/types/domain/ids.rs:348  pub fn from_key(key: &str) -> Result<Self>
src/types/domain/ids.rs:361  pub fn from_key(key: &str) -> Result<Self>
src/types/domain/ids.rs:417  pub struct TxId(pub u64)
src/types/domain/ids.rs:425  pub const INVALID: Self = Self(0)
src/types/domain/ids.rs:431  pub const MAX_COLLECTION_SEQUENCE: u64 = 1_000_000_000_000
src/types/domain/ids.rs:447  pub const INTERNAL_BASE: u64 = u64::MAX - 1_000_000
src/types/domain/ids.rs:451  pub const fn new(id: u64) -> Self
src/types/domain/ids.rs:457  pub const fn inner(self) -> u64
src/types/domain/ids.rs:464  pub const fn as_u64(self) -> u64
src/types/domain/ids.rs:470  pub const fn internal() -> Self
src/types/domain/ids.rs:483  pub fn try_from_internal_offset(offset: u64) -> Result<Self>
src/types/domain/ids.rs:501  pub fn is_valid_origin(&self) -> bool
src/types/domain/misc.rs:20  pub fn serialize<S>(bytes: &[u8; 32], serializer: S) -> Result<S::Ok, S::Error>
src/types/domain/misc.rs:34  pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 32], D::Error>
src/types/domain/misc.rs:61  pub struct WorkflowState
src/types/domain/misc.rs:72  pub const TOMBSTONE_BIT: u64 = 1 << 63
src/types/domain/misc.rs:76  pub const EXPIRY_METADATA_KEY: &str = "__expires_at_seq"
src/types/domain/misc.rs:90  pub const MAX_SEARCH_K: usize = 1_000
src/types/domain/misc.rs:95  pub enum LinkRelation
src/types/domain/misc.rs:109  pub struct MemoryLink
src/types/domain/misc.rs:121  pub struct Entity
src/types/domain/misc.rs:135  pub fn new(id: EntityId, name: impl Into<String>, entity_type: impl Into<String>) -> Self
src/types/domain/misc.rs:148  pub fn try_new(
src/types/domain/misc.rs:184  pub struct Edge
src/types/domain/misc.rs:212  pub fn new(from: EntityId, to: EntityId, label: impl Into<String>) -> Self
src/types/domain/misc.rs:227  pub fn with_source_doc_id(mut self, doc_id: DocId) -> Self
src/types/domain/misc.rs:236  pub fn try_new(
src/types/domain/misc.rs:267  pub fn with_weight(mut self, weight: f32) -> Self
src/types/domain/misc.rs:273  pub fn with_tx_validity(mut self, from: Option<TxId>, to: Option<TxId>) -> Self
src/types/domain/misc.rs:280  pub fn with_business_validity(mut self, from: Option<i64>, to: Option<i64>) -> Self
src/types/domain/misc.rs:287  pub fn with_validity(self, from: Option<TxId>, to: Option<TxId>) -> Self
src/types/domain/misc.rs:307  pub enum MemoryType
src/types/domain/misc.rs:336  pub fn default_decay(&self) -> crate::types::importance::DecayFunction
src/types/domain/misc.rs:353  pub fn default_ttl_tx(&self) -> Option<u64>
src/types/domain/misc.rs:361  pub fn as_metadata_key(&self) -> &'static str
src/types/domain/misc.rs:385  pub struct TlHfdParams
src/types/domain/misc.rs:442  pub enum PprAlgorithm
src/types/domain/misc.rs:463  pub struct PprConfig
src/types/domain/misc.rs:507  pub struct ConfigFingerprint
src/types/domain/misc.rs:527  pub fn new(
src/types/domain/misc.rs:548  pub fn with_threshold(mut self, threshold: f32) -> Self
src/types/domain/misc.rs:555  pub fn threshold(&self) -> f32
src/types/domain/misc.rs:561  pub fn temperature(&self) -> f32
src/types/domain/misc.rs:576  pub enum AutoExtractionMode
src/types/domain/misc.rs:586  pub fn is_enabled(self) -> bool
src/types/domain/mod.rs:38  pub use document::*
src/types/domain/mod.rs:39  pub use ids::*
src/types/domain/mod.rs:40  pub use misc::*
src/types/filter.rs:26  pub enum FilterExpr
src/types/filter.rs:116  pub fn evaluate(&self, metadata: &serde_json::Value) -> bool
src/types/importance.rs:37  pub struct ImportanceScore(f32)
src/types/importance.rs:44  pub fn new(raw: f32) -> Self
src/types/importance.rs:53  pub fn value(&self) -> f32
src/types/importance.rs:68  pub enum DecayFunction
src/types/importance.rs:102  pub fn decay_factor(&self, created_at_tx: TxId, now_tx: TxId) -> f32
src/types/importance.rs:155  pub fn decay_factor_wallclock(
src/types/importance.rs:186  pub struct MemoryImportance
src/types/importance.rs:197  pub fn new(base_score: ImportanceScore, decay: DecayFunction, created_at_tx: TxId) -> Self
src/types/importance.rs:209  pub fn effective_score(&self, now_tx: TxId) -> f32
src/types/saos.rs:30  pub enum FusionStrategy
src/types/saos.rs:43  pub struct SignalFusionStrategies
src/types/saos.rs:74  pub fn uniform(strategy: FusionStrategy) -> Self
src/types/saos.rs:79  pub fn strategy_for_signal(&self, signal_name: &str) -> FusionStrategy
src/types/saos.rs:126  pub enum GraphTraversalStrategy
src/types/saos.rs:152  pub struct FusionWeights
src/types/saos.rs:173  pub fn new(vector: f32, text: f32, graph: f32) -> Result<Self>
src/types/saos.rs:206  pub fn vector(&self) -> f32
src/types/saos.rs:211  pub fn text(&self) -> f32
src/types/saos.rs:216  pub fn graph(&self) -> f32
src/types/saos.rs:222  pub fn metadata(&self) -> f32
src/types/saos.rs:230  pub struct ContextChunk
src/types/saos.rs:256  pub fn combined_text_owned(&self) -> String
src/types/saos.rs:271  pub fn combined_token_count(&self) -> usize
src/types/saos.rs:279  pub fn has_context_prefix(&self) -> bool
src/types/saos.rs:289  pub struct ContextWindow
src/types/saos.rs:319  pub enum Signal
src/types/saos.rs:352  pub enum OnSignalFailure
src/types/saos.rs:363  pub struct QueryRewriteOutput
src/types/saos.rs:375  pub fn new(
src/types/saos.rs:388  pub fn text_only(text: impl Into<String>) -> Self
src/types/saos.rs:397  pub fn is_empty(&self) -> bool
src/types/saos.rs:410  pub struct ScoredEntry
src/types/saos.rs:422  pub struct HybridQuery
src/types/saos.rs:472  pub type QueryOptions = HybridQuery
src/types/saos.rs:476  pub fn builder() -> HybridQueryBuilder
src/types/saos.rs:491  pub struct HybridQueryBuilder
src/types/saos.rs:510  pub type QueryOptionsBuilder = HybridQueryBuilder
src/types/saos.rs:514  pub fn new() -> Self
src/types/saos.rs:519  pub fn with_text_query(mut self, q: impl Into<String>) -> Self
src/types/saos.rs:525  pub fn with_vector_query(mut self, v: Vec<f32>) -> Self
src/types/saos.rs:531  pub fn with_graph_start_node(mut self, start: impl Into<String>) -> Self
src/types/saos.rs:537  pub fn with_graph_strategy(mut self, strategy: GraphTraversalStrategy) -> Self
src/types/saos.rs:543  pub fn with_fusion_weights(mut self, weights: FusionWeights) -> Self
src/types/saos.rs:549  pub fn with_fusion_strategy(mut self, strategy: impl Into<SignalFusionStrategies>) -> Self
src/types/saos.rs:555  pub fn with_filter(mut self, filter: FilterExpr) -> Self
src/types/saos.rs:561  pub fn with_same_community_as(mut self, entity_id: EntityId) -> Self
src/types/saos.rs:567  pub fn with_memory_type_filter(mut self, types: Vec<MemoryType>) -> Self
src/types/saos.rs:573  pub fn with_include_superseded(mut self, include: bool) -> Self
src/types/saos.rs:579  pub fn with_include_provenance(mut self, include: bool) -> Self
src/types/saos.rs:585  pub fn with_rerank_pool_multiplier(mut self, multiplier: usize) -> Self
src/types/saos.rs:591  pub fn with_rerank_pool_max(mut self, max: usize) -> Self
src/types/saos.rs:597  pub fn with_on_signal_failure(mut self, policy: OnSignalFailure) -> Self
src/types/saos.rs:603  pub fn with_k(mut self, k: usize) -> Self
src/types/saos.rs:612  pub fn build(self) -> Result<HybridQuery>
src/witness.rs:39  pub enum ScanMethod
src/witness.rs:52  pub struct RequestContext
src/witness.rs:66  pub fn new(tenant_id: crate::TenantId, tx_id: crate::TxId) -> Self
src/witness.rs:75  pub struct SagaRecord
src/witness.rs:86  pub fn new(deleted_keys_hash: [u8; 32]) -> Self
```

## Innen: crate-interne Grenzen (`pub(crate)`, `pub(super)`, `pub(in …)`)

```rust
```
