# Typen-Register & Kollisions-Audit (K3 / A2)

> Generiert durch `tools/type_registry_sync.py`.
> Einzigartige Typnamen: 1532 | Gesamte Definitionen: 1751
> Erkannte typografische Crate-Übergreifende Kollisionen: **155**

## Crate Typen-Statistik

| Crate | Definierte Typen (Structs/Enums/Traits/Aliases) |
|---|---|
| `contextra` | 20 |
| `contextra-adapt` | 49 |
| `contextra-agent` | 45 |
| `contextra-audit-export` | 10 |
| `contextra-avv-generator` | 4 |
| `contextra-checkpoint` | 26 |
| `contextra-cognition` | 50 |
| `contextra-crypto` | 73 |
| `contextra-db` | 19 |
| `contextra-durable-fs` | 19 |
| `contextra-engine` | 101 |
| `contextra-erasure` | 197 |
| `contextra-graph` | 116 |
| `contextra-infer-candle` | 37 |
| `contextra-infer-ollama` | 38 |
| `contextra-infer-onnx` | 13 |
| `contextra-ingest` | 116 |
| `contextra-kvcache` | 48 |
| `contextra-license` | 6 |
| `contextra-mcp` | 36 |
| `contextra-mvcc` | 18 |
| `contextra-ports` | 150 |
| `contextra-privacy` | 51 |
| `contextra-py` | 8 |
| `contextra-rank` | 47 |
| `contextra-router` | 26 |
| `contextra-sandbox` | 16 |
| `contextra-simd` | 9 |
| `contextra-store` | 111 |
| `contextra-sys` | 9 |
| `contextra-testkit` | 27 |
| `contextra-text` | 44 |
| `contextra-types` | 96 |
| `contextra-vector` | 67 |
| `contextra-wire` | 49 |

## Kollisionen über Crate-Grenzen (Identische Typnamen)

| Typname | Beteiligte Crates | Definitionen |
|---|---|---|
| `AdaptiveDecayController` | `contextra-adapt`, `contextra-engine` | 2 |
| `AncestorSalts` | `contextra-crypto`, `contextra-erasure` | 2 |
| `Armed` | `contextra-erasure`, `contextra-ports`, `contextra-types` | 3 |
| `BarrierExclusive` | `contextra-erasure`, `contextra-ports` | 2 |
| `BarrierShared` | `contextra-erasure`, `contextra-ports` | 2 |
| `BoundedTopK` | `contextra-rank`, `contextra-text` | 2 |
| `BoxFuture` | `contextra-ports`, `contextra-privacy` | 2 |
| `CommitBarrier` | `contextra-erasure`, `contextra-ports` | 2 |
| `CommunityDetectionConfig` | `contextra-engine`, `contextra-graph` | 2 |
| `CompactionPermit` | `contextra-erasure`, `contextra-ports` | 2 |
| `CompactionStrategy` | `contextra-cognition`, `contextra-store` | 2 |
| `Confidence` | `contextra-infer-ollama`, `contextra-ingest` | 2 |
| `ConfigFingerprint` | `contextra-rank`, `contextra-types` | 2 |
| `ConformalCalibrator` | `contextra-rank`, `contextra-router` | 2 |
| `ConsolidationNodesGuard` | `contextra-cognition`, `contextra-graph` | 2 |
| `ContextChunk` | `contextra-rank`, `contextra-types` | 2 |
| `ContextEditAuditRecord` | `contextra-agent`, `contextra-privacy` | 2 |
| `ControlEvidence` | `contextra-erasure`, `contextra-types` | 2 |
| `CustodyLevel` | `contextra-crypto`, `contextra-erasure` | 2 |
| `DecayControllerConfig` | `contextra-adapt`, `contextra-engine` | 2 |
| `DecaySignalInputs` | `contextra-adapt`, `contextra-engine` | 2 |
| `Dek` | `contextra-crypto`, `contextra-erasure` | 2 |
| `DeletionLayer` | `contextra-crypto`, `contextra-erasure` | 2 |
| `DeletionProof` | `contextra-crypto`, `contextra-erasure` | 2 |
| `DeletionProofKeyPair` | `contextra-crypto`, `contextra-erasure` | 2 |
| `DeletionScope` | `contextra-crypto`, `contextra-erasure` | 2 |
| `DirLock` | `contextra-durable-fs`, `contextra-store` | 2 |
| `DistanceMetric` | `contextra-simd`, `contextra-types` | 2 |
| `DocId` | `contextra-rank`, `contextra-types` | 3 |
| `DriftDetector` | `contextra-adapt`, `contextra-rank` | 2 |
| `DurabilityConfigError` | `contextra`, `contextra-store` | 2 |
| `DurabilityMode` | `contextra`, `contextra-store` | 2 |
| `Embedding` | `contextra-types`, `contextra-wire` | 2 |
| `EncryptedSegmentPayload` | `contextra-crypto`, `contextra-kvcache` | 2 |
| `EntityExtractor` | `contextra-engine`, `contextra-graph` | 2 |
| `Envelope` | `contextra-erasure`, `contextra-ports` | 2 |
| `EraseMode` | `contextra-erasure`, `contextra-types` | 2 |
| `EraseReceipt` | `contextra-erasure`, `contextra-types` | 2 |
| `EraseRequest` | `contextra-erasure`, `contextra-types` | 2 |
| `EraseTarget` | `contextra-erasure`, `contextra-ports`, `contextra-types` | 3 |
| `ErasureEvent` | `contextra-erasure`, `contextra-ports` | 2 |
| `ErasureId` | `contextra-erasure`, `contextra-types` | 2 |
| `ErasureLayer` | `contextra-erasure`, `contextra-ports` | 2 |
| `ErasureObserver` | `contextra-erasure`, `contextra-ports` | 2 |
| `ErasureReport` | `contextra-engine`, `contextra-erasure` | 2 |
| `ErasureStatus` | `contextra-erasure`, `contextra-types` | 2 |
| `ErasureTarget` | `contextra-erasure`, `contextra-ports` | 2 |
| `Error` | `contextra-crypto`, `contextra-engine`, `contextra-rank`, `contextra-types` | 4 |
| `EvictionCommand` | `contextra-crypto`, `contextra-kvcache` | 2 |
| `EvictionWorker` | `contextra-crypto`, `contextra-kvcache` | 2 |
| `ExcludedScope` | `contextra-crypto`, `contextra-erasure` | 2 |
| `FileId` | `contextra-erasure`, `contextra-ports`, `contextra-types` | 3 |
| `FileKey` | `contextra-erasure`, `contextra-ports` | 2 |
| `FileKeyring` | `contextra-erasure`, `contextra-ports` | 2 |
| `FileKind` | `contextra-erasure`, `contextra-ports` | 2 |
| `FixedTestClock` | `contextra-mcp`, `contextra-privacy`, `contextra-rank` | 3 |
| `FusionStrategy` | `contextra-rank`, `contextra-types` | 2 |
| `FusionWeights` | `contextra-rank`, `contextra-types` | 2 |
| `FuzzInput` | `contextra-graph`, `contextra-mcp`, `contextra-store` | 4 |
| `GateDecision` | `contextra-erasure`, `contextra-ports` | 2 |
| `GraphRepairAttestation` | `contextra-crypto`, `contextra-erasure` | 2 |
| `HeapItem` | `contextra-graph`, `contextra-store` | 2 |
| `HybridQueryBuilder` | `contextra-engine`, `contextra-types` | 2 |
| `HyperEdge` | `contextra-graph`, `contextra-wire` | 2 |
| `HyperEdgeId` | `contextra-ports`, `contextra-wire` | 2 |
| `InMemoryStorageEngine` | `contextra-store`, `contextra-testkit` | 2 |
| `KeyBound` | `contextra-erasure`, `contextra-ports` | 2 |
| `KeyBuf` | `contextra-erasure`, `contextra-ports` | 2 |
| `KeyCoverage` | `contextra-erasure`, `contextra-ports`, `contextra-types` | 3 |
| `KeyRange` | `contextra-erasure`, `contextra-ports` | 2 |
| `KvDeleteMode` | `contextra`, `contextra-store` | 2 |
| `KvSegment` | `contextra-crypto`, `contextra-kvcache` | 2 |
| `LatencyBudgetGuard` | `contextra-adapt`, `contextra-db` | 2 |
| `LayerCleanupProof` | `contextra-crypto`, `contextra-erasure` | 2 |
| `LayerId` | `contextra-erasure`, `contextra-ports`, `contextra-types` | 3 |
| `LayerScope` | `contextra-erasure`, `contextra-ports` | 2 |
| `LayerWitness` | `contextra-erasure`, `contextra-ports`, `contextra-types` | 4 |
| `LayerWitnessExt` | `contextra-crypto`, `contextra-erasure` | 2 |
| `LayerWitnessMeasurement` | `contextra-crypto`, `contextra-erasure` | 2 |
| `LedgerEntry` | `contextra-erasure`, `contextra-ports` | 3 |
| `LedgerSink` | `contextra-erasure`, `contextra-ports` | 2 |
| `LineInfo` | `contextra-engine`, `contextra-ingest` | 2 |
| `LoomMetadata` | `contextra-checkpoint`, `contextra-store` | 2 |
| `MeasureClass` | `contextra-erasure`, `contextra-types` | 2 |
| `MemoryLink` | `contextra-rank`, `contextra-types` | 2 |
| `Meta` | `contextra-checkpoint`, `contextra-ports` | 2 |
| `MockEmbedder` | `contextra-kvcache`, `contextra-mcp`, `contextra-testkit` | 5 |
| `MockLlmGenerator` | `contextra-cognition`, `contextra-engine`, `contextra-mcp` | 4 |
| `MockStorage` | `contextra-checkpoint`, `contextra-engine`, `contextra-text` | 6 |
| `ObservedPhase` | `contextra-erasure`, `contextra-ports` | 2 |
| `ObserverError` | `contextra-erasure`, `contextra-ports` | 2 |
| `PendingReason` | `contextra-erasure`, `contextra-types` | 2 |
| `PercolationConfig` | `contextra-cognition`, `contextra-graph` | 2 |
| `Phase` | `contextra-erasure`, `contextra-types` | 2 |
| `PidLatencyController` | `contextra-adapt`, `contextra-db` | 2 |
| `PurgeBarrier` | `contextra-erasure`, `contextra-ports` | 2 |
| `PurgeCx` | `contextra-erasure`, `contextra-ports` | 3 |
| `PurgeOutcome` | `contextra-erasure`, `contextra-ports` | 2 |
| `PurgePermit` | `contextra-erasure`, `contextra-ports` | 2 |
| `RangeGate` | `contextra-erasure`, `contextra-ports` | 2 |
| `RangeSet` | `contextra-erasure`, `contextra-ports` | 2 |
| `ReadGate` | `contextra-erasure`, `contextra-ports` | 2 |
| `RecordSealer` | `contextra-erasure`, `contextra-ports` | 2 |
| `RefusalReason` | `contextra-erasure`, `contextra-types` | 2 |
| `ReportSignature` | `contextra-erasure`, `contextra-ports` | 2 |
| `ReportSigner` | `contextra-erasure`, `contextra-ports` | 3 |
| `RerankDeadline` | `contextra-adapt`, `contextra-db` | 2 |
| `RerankPidController` | `contextra-adapt`, `contextra-db` | 2 |
| `Residue` | `contextra-erasure`, `contextra-types` | 2 |
| `ResourceBudget` | `contextra-ingest`, `contextra-types` | 2 |
| `Result` | `contextra-crypto`, `contextra-mcp`, `contextra-rank`, `contextra-simd`, `contextra-types` | 5 |
| `RetentionAttestation` | `contextra-erasure`, `contextra-types` | 2 |
| `RetentionCode` | `contextra-erasure`, `contextra-types` | 2 |
| `RoleBinding` | `contextra-ports`, `contextra-wire` | 2 |
| `RoleId` | `contextra-ports`, `contextra-wire` | 2 |
| `ScanTechniqueSerde` | `contextra-crypto`, `contextra-erasure` | 2 |
| `Scanner` | `contextra-erasure`, `contextra-ports`, `contextra-types` | 3 |
| `ScoredDocument` | `contextra-types`, `contextra-wire` | 2 |
| `ScrubOutcome` | `contextra-durable-fs`, `contextra-erasure` | 2 |
| `SealCtx` | `contextra-erasure`, `contextra-ports` | 2 |
| `SecretKey` | `contextra-crypto`, `contextra-erasure` | 2 |
| `Seq` | `contextra-erasure`, `contextra-ports`, `contextra-types` | 3 |
| `ShadowDiscrepancy` | `contextra-adapt`, `contextra-graph` | 2 |
| `ShadowSink` | `contextra-adapt`, `contextra-graph` | 2 |
| `Shard` | `contextra-crypto`, `contextra-kvcache` | 2 |
| `Signal` | `contextra-rank`, `contextra-types` | 2 |
| `SignalFusionStrategies` | `contextra-rank`, `contextra-types` | 2 |
| `SignerError` | `contextra-erasure`, `contextra-ports` | 2 |
| `SignerId` | `contextra-erasure`, `contextra-ports` | 2 |
| `SigningKeyHandle` | `contextra-crypto`, `contextra-erasure` | 2 |
| `SimpleRng` | `contextra-graph`, `contextra-router` | 2 |
| `SpillHandler` | `contextra-crypto`, `contextra-kvcache` | 2 |
| `SplitMix64` | `contextra-adapt`, `contextra-cognition` | 2 |
| `StagedOp` | `contextra-store`, `contextra-testkit` | 2 |
| `StagedTxMap` | `contextra-store`, `contextra-testkit` | 2 |
| `StorageBackend` | `contextra-erasure`, `contextra-ports` | 2 |
| `StoreDataMap` | `contextra-store`, `contextra-testkit` | 2 |
| `Target` | `contextra-adapt`, `contextra-graph`, `contextra-sys` | 5 |
| `TargetKind` | `contextra-erasure`, `contextra-ports` | 2 |
| `TenantIsolatedKvStore` | `contextra-crypto`, `contextra-kvcache` | 2 |
| `TenantState` | `contextra-crypto`, `contextra-kvcache` | 2 |
| `TestClock` | `contextra-cognition`, `contextra-vector` | 2 |
| `TestMetricsSink` | `contextra-testkit`, `contextra-vector` | 2 |
| `Timestamp` | `contextra-erasure`, `contextra-ports`, `contextra-types` | 4 |
| `Timing` | `contextra-erasure`, `contextra-types` | 2 |
| `TlHfdParams` | `contextra-graph`, `contextra-types` | 2 |
| `Unarmed` | `contextra-erasure`, `contextra-types` | 2 |
| `UncoveredLayer` | `contextra-crypto`, `contextra-erasure` | 2 |
| `UnitKey` | `contextra-crypto`, `contextra-erasure` | 2 |
| `Value` | `contextra-ingest`, `contextra-ports` | 4 |
| `VerificationKey` | `contextra-crypto`, `contextra-erasure` | 2 |
| `VerifiedVia` | `contextra-erasure`, `contextra-ports` | 4 |
| `VfsFile` | `contextra-erasure`, `contextra-ports` | 2 |
| `VfsOpenOptions` | `contextra-erasure`, `contextra-ports` | 2 |
| `WorkflowState` | `contextra-engine`, `contextra-types` | 2 |

## Detailierte Kollisionsorte

### `AdaptiveDecayController`

- `contextra-adapt`: `/app/crates/contextra-adapt/src/decay_controller.rs:67` -> `pub struct AdaptiveDecayController`
- `contextra-engine`: `/app/crates/contextra-engine/src/decay_controller.rs:58` -> `pub struct AdaptiveDecayController`
### `AncestorSalts`

- `contextra-erasure`: `/app/crates/contextra-erasure/src/crypto_local.rs:47` -> `pub struct AncestorSalts`
- `contextra-crypto`: `/app/crates/contextra-crypto/src/dek.rs:21` -> `pub struct AncestorSalts`
### `Armed`

- `contextra-ports`: `/app/crates/contextra-ports/src/erasure.rs:151` -> `pub struct Armed`
- `contextra-erasure`: `/app/crates/contextra-erasure/src/types_local.rs:724` -> `pub struct Armed`
- `contextra-types`: `/app/crates/contextra-types/src/erasure.rs:434` -> `pub struct Armed`
### `BarrierExclusive`

- `contextra-ports`: `/app/crates/contextra-ports/src/barrier.rs:37` -> `pub struct BarrierExclusive`
- `contextra-erasure`: `/app/crates/contextra-erasure/src/types_local.rs:286` -> `pub struct BarrierExclusive`
### `BarrierShared`

- `contextra-ports`: `/app/crates/contextra-ports/src/barrier.rs:19` -> `pub struct BarrierShared`
- `contextra-erasure`: `/app/crates/contextra-erasure/src/types_local.rs:268` -> `pub struct BarrierShared`
### `BoundedTopK`

- `contextra-rank`: `/app/crates/contextra-rank/src/fusion/topk.rs:25` -> `pub struct BoundedTopK`
- `contextra-text`: `/app/crates/contextra-text/src/wand.rs:57` -> `pub struct BoundedTopK`
### `BoxFuture`

- `contextra-ports`: `/app/crates/contextra-ports/src/lib.rs:32` -> `pub type_alias BoxFuture`
- `contextra-privacy`: `/app/crates/contextra-privacy/src/egress_vault.rs:157` -> `pub type_alias BoxFuture`
### `CommitBarrier`

- `contextra-ports`: `/app/crates/contextra-ports/src/barrier.rs:91` -> `pub trait CommitBarrier`
- `contextra-erasure`: `/app/crates/contextra-erasure/src/types_local.rs:340` -> `pub trait CommitBarrier`
### `CommunityDetectionConfig`

- `contextra-graph`: `/app/crates/contextra-graph/src/community.rs:29` -> `pub struct CommunityDetectionConfig`
- `contextra-engine`: `/app/crates/contextra-engine/src/lib.rs:181` -> `pub struct CommunityDetectionConfig`
### `CompactionPermit`

- `contextra-ports`: `/app/crates/contextra-ports/src/barrier.rs:55` -> `pub struct CompactionPermit`
- `contextra-erasure`: `/app/crates/contextra-erasure/src/types_local.rs:304` -> `pub struct CompactionPermit`
### `CompactionStrategy`

- `contextra-store`: `/app/crates/contextra-store/src/compaction/adaptive.rs:77` -> `pub enum CompactionStrategy`
- `contextra-cognition`: `/app/crates/contextra-cognition/src/context_compaction/types.rs:15` -> `pub enum CompactionStrategy`
### `Confidence`

- `contextra-ingest`: `/app/crates/contextra-ingest/src/detect/mod.rs:26` -> `pub enum Confidence`
- `contextra-infer-ollama`: `/app/crates/contextra-infer-ollama/src/importance.rs:94` -> `pub enum Confidence`
### `ConfigFingerprint`

- `contextra-rank`: `/app/crates/contextra-rank/src/local_types.rs:23` -> `pub struct ConfigFingerprint`
- `contextra-types`: `/app/crates/contextra-types/src/types/domain/misc.rs:507` -> `pub struct ConfigFingerprint`
### `ConformalCalibrator`

- `contextra-rank`: `/app/crates/contextra-rank/src/calibration/conformal.rs:40` -> `pub trait ConformalCalibrator`
- `contextra-router`: `/app/crates/contextra-router/src/profile.rs:227` -> `pub type_alias ConformalCalibrator`
### `ConsolidationNodesGuard`

- `contextra-graph`: `/app/crates/contextra-graph/src/hyperedge.rs:312` -> `pub struct ConsolidationNodesGuard`
- `contextra-cognition`: `/app/crates/contextra-cognition/src/consolidation_locks.rs:45` -> `pub struct ConsolidationNodesGuard`
### `ContextChunk`

- `contextra-rank`: `/app/crates/contextra-rank/src/local_types.rs:43` -> `pub struct ContextChunk`
- `contextra-types`: `/app/crates/contextra-types/src/types/saos.rs:230` -> `pub struct ContextChunk`
### `ContextEditAuditRecord`

- `contextra-privacy`: `/app/crates/contextra-privacy/src/context_edit_audit.rs:69` -> `pub struct ContextEditAuditRecord`
- `contextra-agent`: `/app/crates/contextra-agent/src/clm_scratchpad.rs:29` -> `pub struct ContextEditAuditRecord`
### `ControlEvidence`

- `contextra-erasure`: `/app/crates/contextra-erasure/src/types_local.rs:728` -> `pub struct ControlEvidence`
- `contextra-types`: `/app/crates/contextra-types/src/erasure.rs:442` -> `pub struct ControlEvidence`
### `CustodyLevel`

- `contextra-erasure`: `/app/crates/contextra-erasure/src/deletion_proof.rs:150` -> `pub enum CustodyLevel`
- `contextra-crypto`: `/app/crates/contextra-crypto/src/deletion_proof/types.rs:97` -> `pub enum CustodyLevel`
### `DecayControllerConfig`

- `contextra-adapt`: `/app/crates/contextra-adapt/src/decay_controller.rs:46` -> `pub struct DecayControllerConfig`
- `contextra-engine`: `/app/crates/contextra-engine/src/decay_controller.rs:40` -> `pub struct DecayControllerConfig`
### `DecaySignalInputs`

- `contextra-adapt`: `/app/crates/contextra-adapt/src/decay_controller.rs:20` -> `pub struct DecaySignalInputs`
- `contextra-engine`: `/app/crates/contextra-engine/src/decay_controller.rs:20` -> `pub struct DecaySignalInputs`
### `Dek`

- `contextra-erasure`: `/app/crates/contextra-erasure/src/crypto_local.rs:39` -> `pub struct Dek`
- `contextra-crypto`: `/app/crates/contextra-crypto/src/dek.rs:16` -> `pub struct Dek`
### `DeletionLayer`

- `contextra-erasure`: `/app/crates/contextra-erasure/src/deletion_proof.rs:295` -> `pub enum DeletionLayer`
- `contextra-crypto`: `/app/crates/contextra-crypto/src/deletion_proof/types.rs:190` -> `pub enum DeletionLayer`
### `DeletionProof`

- `contextra-erasure`: `/app/crates/contextra-erasure/src/deletion_proof.rs:425` -> `pub struct DeletionProof`
- `contextra-crypto`: `/app/crates/contextra-crypto/src/deletion_proof/types.rs:321` -> `pub struct DeletionProof`
### `DeletionProofKeyPair`

- `contextra-erasure`: `/app/crates/contextra-erasure/src/deletion_proof.rs:120` -> `pub struct DeletionProofKeyPair`
- `contextra-crypto`: `/app/crates/contextra-crypto/src/ed25519_proof.rs:47` -> `pub struct DeletionProofKeyPair`
### `DeletionScope`

- `contextra-erasure`: `/app/crates/contextra-erasure/src/deletion_proof.rs:400` -> `pub enum DeletionScope`
- `contextra-crypto`: `/app/crates/contextra-crypto/src/deletion_proof/types.rs:295` -> `pub enum DeletionScope`
### `DirLock`

- `contextra-store`: `/app/crates/contextra-store/src/util.rs:24` -> `pub struct DirLock`
- `contextra-durable-fs`: `/app/crates/contextra-durable-fs/src/lib.rs:323` -> `pub struct DirLock`
### `DistanceMetric`

- `contextra-simd`: `/app/crates/contextra-simd/src/metric.rs:5` -> `pub enum DistanceMetric`
- `contextra-types`: `/app/crates/contextra-types/src/types/domain/document.rs:20` -> `pub enum DistanceMetric`
### `DocId`

- `contextra-rank`: `/app/crates/contextra-rank/src/local_types.rs:8` -> `pub struct DocId`
- `contextra-types`: `/app/crates/contextra-types/src/types/domain/ids.rs:173` -> `pub struct DocId`
- `contextra-types`: `/app/crates/contextra-types/src/types/domain/ids.rs:180` -> `pub struct DocId`
### `DriftDetector`

- `contextra-rank`: `/app/crates/contextra-rank/src/drift.rs:61` -> `pub struct DriftDetector`
- `contextra-adapt`: `/app/crates/contextra-adapt/src/drift.rs:27` -> `pub trait DriftDetector`
### `DurabilityConfigError`

- `contextra-store`: `/app/crates/contextra-store/src/lsm/config.rs:25` -> `pub enum DurabilityConfigError`
- `contextra`: `/app/crates/contextra/src/performance_profile.rs:51` -> `pub enum DurabilityConfigError`
### `DurabilityMode`

- `contextra-store`: `/app/crates/contextra-store/src/lsm/config.rs:12` -> `pub enum DurabilityMode`
- `contextra`: `/app/crates/contextra/src/performance_profile.rs:37` -> `pub enum DurabilityMode`
### `Embedding`

- `contextra-wire`: `/app/crates/contextra-wire/src/contextra_generated.rs:51` -> `pub struct Embedding`
- `contextra-types`: `/app/crates/contextra-types/src/types/domain/document.rs:174` -> `pub struct Embedding`
### `EncryptedSegmentPayload`

- `contextra-crypto`: `/app/crates/contextra-crypto/src/kv_segment/segment.rs:26` -> `pub struct EncryptedSegmentPayload`
- `contextra-kvcache`: `/app/crates/contextra-kvcache/src/segment.rs:223` -> `pub struct EncryptedSegmentPayload`
### `EntityExtractor`

- `contextra-graph`: `/app/crates/contextra-graph/src/entity_extraction.rs:64` -> `pub trait EntityExtractor`
- `contextra-engine`: `/app/crates/contextra-engine/src/extraction/mod.rs:31` -> `pub trait EntityExtractor`
### `Envelope`

- `contextra-ports`: `/app/crates/contextra-ports/src/keys.rs:104` -> `pub struct Envelope`
- `contextra-erasure`: `/app/crates/contextra-erasure/src/types_local.rs:474` -> `pub struct Envelope`
### `EraseMode`

- `contextra-erasure`: `/app/crates/contextra-erasure/src/types_local.rs:1183` -> `pub enum EraseMode`
- `contextra-types`: `/app/crates/contextra-types/src/erasure.rs:67` -> `pub enum EraseMode`
### `EraseReceipt`

- `contextra-erasure`: `/app/crates/contextra-erasure/src/types_local.rs:1229` -> `pub struct EraseReceipt`
- `contextra-types`: `/app/crates/contextra-types/src/erasure.rs:133` -> `pub struct EraseReceipt`
### `EraseRequest`

- `contextra-erasure`: `/app/crates/contextra-erasure/src/saga/machine.rs:18` -> `pub struct EraseRequest`
- `contextra-types`: `/app/crates/contextra-types/src/erasure.rs:116` -> `pub struct EraseRequest`
### `EraseTarget`

- `contextra-ports`: `/app/crates/contextra-ports/src/erasure.rs:44` -> `pub enum EraseTarget`
- `contextra-erasure`: `/app/crates/contextra-erasure/src/types_local.rs:801` -> `pub enum EraseTarget`
- `contextra-types`: `/app/crates/contextra-types/src/erasure.rs:17` -> `pub enum EraseTarget`
### `ErasureEvent`

- `contextra-ports`: `/app/crates/contextra-ports/src/observer.rs:49` -> `pub struct ErasureEvent`
- `contextra-erasure`: `/app/crates/contextra-erasure/src/types_local.rs:1140` -> `pub struct ErasureEvent`
### `ErasureId`

- `contextra-erasure`: `/app/crates/contextra-erasure/src/types_local.rs:19` -> `pub struct ErasureId`
- `contextra-types`: `/app/crates/contextra-types/src/ids.rs:77` -> `pub struct ErasureId`
### `ErasureLayer`

- `contextra-ports`: `/app/crates/contextra-ports/src/erasure.rs:236` -> `pub trait ErasureLayer`
- `contextra-erasure`: `/app/crates/contextra-erasure/src/types_local.rs:1034` -> `pub trait ErasureLayer`
### `ErasureObserver`

- `contextra-ports`: `/app/crates/contextra-ports/src/observer.rs:79` -> `pub trait ErasureObserver`
- `contextra-erasure`: `/app/crates/contextra-erasure/src/types_local.rs:1166` -> `pub trait ErasureObserver`
### `ErasureReport`

- `contextra-erasure`: `/app/crates/contextra-erasure/src/report/build.rs:152` -> `pub struct ErasureReport`
- `contextra-engine`: `/app/crates/contextra-engine/src/contextra_impl/erasure.rs:90` -> `pub struct ErasureReport`
### `ErasureStatus`

- `contextra-erasure`: `/app/crates/contextra-erasure/src/types_local.rs:1287` -> `pub enum ErasureStatus`
- `contextra-types`: `/app/crates/contextra-types/src/erasure.rs:235` -> `pub enum ErasureStatus`
### `ErasureTarget`

- `contextra-ports`: `/app/crates/contextra-ports/src/erasure.rs:302` -> `pub trait ErasureTarget`
- `contextra-erasure`: `/app/crates/contextra-erasure/src/backend.rs:21` -> `pub trait ErasureTarget`
### `Error`

- `contextra-rank`: `/app/crates/contextra-rank/src/fusion/types.rs:34` -> `type_alias Error`
- `contextra-crypto`: `/app/crates/contextra-crypto/src/ed25519_proof.rs:164` -> `type_alias Error`
- `contextra-engine`: `/app/crates/contextra-engine/src/filter.rs:101` -> `type_alias Error`
- `contextra-types`: `/app/crates/contextra-types/src/types/domain/ids.rs:98` -> `type_alias Error`
### `EvictionCommand`

- `contextra-crypto`: `/app/crates/contextra-crypto/src/kv_segment/eviction_worker.rs:30` -> `enum EvictionCommand`
- `contextra-kvcache`: `/app/crates/contextra-kvcache/src/eviction_worker.rs:25` -> `enum EvictionCommand`
### `EvictionWorker`

- `contextra-crypto`: `/app/crates/contextra-crypto/src/kv_segment/eviction_worker.rs:40` -> `pub struct EvictionWorker`
- `contextra-kvcache`: `/app/crates/contextra-kvcache/src/eviction_worker.rs:85` -> `pub struct EvictionWorker`
### `ExcludedScope`

- `contextra-erasure`: `/app/crates/contextra-erasure/src/deletion_proof.rs:391` -> `pub enum ExcludedScope`
- `contextra-crypto`: `/app/crates/contextra-crypto/src/deletion_proof/types.rs:286` -> `pub enum ExcludedScope`
### `FileId`

- `contextra-ports`: `/app/crates/contextra-ports/src/keys.rs:12` -> `pub struct FileId`
- `contextra-erasure`: `/app/crates/contextra-erasure/src/types_local.rs:384` -> `pub struct FileId`
- `contextra-types`: `/app/crates/contextra-types/src/ids.rs:103` -> `pub struct FileId`
### `FileKey`

- `contextra-ports`: `/app/crates/contextra-ports/src/keys.rs:42` -> `pub struct FileKey`
- `contextra-erasure`: `/app/crates/contextra-erasure/src/types_local.rs:414` -> `pub struct FileKey`
### `FileKeyring`

- `contextra-ports`: `/app/crates/contextra-ports/src/keys.rs:177` -> `pub trait FileKeyring`
- `contextra-erasure`: `/app/crates/contextra-erasure/src/types_local.rs:518` -> `pub trait FileKeyring`
### `FileKind`

- `contextra-ports`: `/app/crates/contextra-ports/src/keys.rs:28` -> `pub enum FileKind`
- `contextra-erasure`: `/app/crates/contextra-erasure/src/types_local.rs:400` -> `pub enum FileKind`
### `FixedTestClock`

- `contextra-rank`: `/app/crates/contextra-rank/src/calibration/isotonic.rs:332` -> `struct FixedTestClock`
- `contextra-privacy`: `/app/crates/contextra-privacy/src/context_edit_audit.rs:281` -> `struct FixedTestClock`
- `contextra-mcp`: `/app/crates/contextra-mcp/src/prompt_injection/tests.rs:353` -> `struct FixedTestClock`
### `FusionStrategy`

- `contextra-rank`: `/app/crates/contextra-rank/src/local_types.rs:30` -> `pub enum FusionStrategy`
- `contextra-types`: `/app/crates/contextra-types/src/types/saos.rs:30` -> `pub enum FusionStrategy`
### `FusionWeights`

- `contextra-rank`: `/app/crates/contextra-rank/src/local_types.rs:62` -> `pub struct FusionWeights`
- `contextra-types`: `/app/crates/contextra-types/src/types/saos.rs:152` -> `pub struct FusionWeights`
### `FuzzInput`

- `contextra-store`: `/app/crates/contextra-store/fuzz/fuzz_targets/wal_mutation_chaos.rs:42` -> `enum FuzzInput`
- `contextra-store`: `/app/crates/contextra-store/fuzz/fuzz_targets/fuzz_sstable_binary_search.rs:7` -> `struct FuzzInput`
- `contextra-mcp`: `/app/crates/contextra-mcp/fuzz/fuzz_targets/fuzz_prompt_injection_guard.rs:9` -> `struct FuzzInput`
- `contextra-graph`: `/app/crates/contextra-graph/fuzz/fuzz_targets/fuzz_graph_persist.rs:178` -> `struct FuzzInput`
### `GateDecision`

- `contextra-ports`: `/app/crates/contextra-ports/src/gate.rs:104` -> `pub enum GateDecision`
- `contextra-erasure`: `/app/crates/contextra-erasure/src/types_local.rs:205` -> `pub enum GateDecision`
### `GraphRepairAttestation`

- `contextra-erasure`: `/app/crates/contextra-erasure/src/deletion_proof.rs:92` -> `pub struct GraphRepairAttestation`
- `contextra-crypto`: `/app/crates/contextra-crypto/src/deletion_proof/types.rs:76` -> `pub struct GraphRepairAttestation`
### `HeapItem`

- `contextra-store`: `/app/crates/contextra-store/src/compaction/engine.rs:455` -> `struct HeapItem`
- `contextra-graph`: `/app/crates/contextra-graph/src/path_rag/k_path.rs:59` -> `struct HeapItem`
### `HybridQueryBuilder`

- `contextra-engine`: `/app/crates/contextra-engine/src/collection/query_builder/builder.rs:31` -> `pub struct HybridQueryBuilder`
- `contextra-types`: `/app/crates/contextra-types/src/types/saos.rs:491` -> `pub struct HybridQueryBuilder`
### `HyperEdge`

- `contextra-wire`: `/app/crates/contextra-wire/src/contextra_generated.rs:1304` -> `pub struct HyperEdge`
- `contextra-graph`: `/app/crates/contextra-graph/src/hyperedge.rs:44` -> `pub struct HyperEdge`
### `HyperEdgeId`

- `contextra-ports`: `/app/crates/contextra-ports/src/graph.rs:63` -> `pub struct HyperEdgeId`
- `contextra-wire`: `/app/crates/contextra-wire/src/contextra_generated.rs:1030` -> `pub struct HyperEdgeId`
### `InMemoryStorageEngine`

- `contextra-store`: `/app/crates/contextra-store/src/engine/memory_only.rs:32` -> `pub struct InMemoryStorageEngine`
- `contextra-testkit`: `/app/crates/contextra-testkit/src/in_memory_store.rs:64` -> `pub struct InMemoryStorageEngine`
### `KeyBound`

- `contextra-ports`: `/app/crates/contextra-ports/src/gate.rs:49` -> `pub enum KeyBound`
- `contextra-erasure`: `/app/crates/contextra-erasure/src/types_local.rs:150` -> `pub enum KeyBound`
### `KeyBuf`

- `contextra-ports`: `/app/crates/contextra-ports/src/gate.rs:26` -> `pub struct KeyBuf`
- `contextra-erasure`: `/app/crates/contextra-erasure/src/types_local.rs:128` -> `pub struct KeyBuf`
### `KeyCoverage`

- `contextra-ports`: `/app/crates/contextra-ports/src/erasure.rs:219` -> `pub enum KeyCoverage`
- `contextra-erasure`: `/app/crates/contextra-erasure/src/types_local.rs:698` -> `pub enum KeyCoverage`
- `contextra-types`: `/app/crates/contextra-types/src/erasure.rs:269` -> `pub enum KeyCoverage`
### `KeyRange`

- `contextra-ports`: `/app/crates/contextra-ports/src/gate.rs:77` -> `pub struct KeyRange`
- `contextra-erasure`: `/app/crates/contextra-erasure/src/types_local.rs:178` -> `pub struct KeyRange`
### `KvDeleteMode`

- `contextra-store`: `/app/crates/contextra-store/src/kv/delete_mode.rs:18` -> `pub enum KvDeleteMode`
- `contextra`: `/app/crates/contextra/src/collection_profile.rs:39` -> `pub enum KvDeleteMode`
### `KvSegment`

- `contextra-crypto`: `/app/crates/contextra-crypto/src/kv_segment/segment.rs:45` -> `pub struct KvSegment`
- `contextra-kvcache`: `/app/crates/contextra-kvcache/src/segment.rs:237` -> `pub struct KvSegment`
### `LatencyBudgetGuard`

- `contextra-db`: `/app/crates/contextra-db/src/pid_latency_controller.rs:88` -> `pub struct LatencyBudgetGuard`
- `contextra-adapt`: `/app/crates/contextra-adapt/src/pid_latency_controller.rs:151` -> `pub struct LatencyBudgetGuard`
### `LayerCleanupProof`

- `contextra-erasure`: `/app/crates/contextra-erasure/src/deletion_proof.rs:224` -> `pub struct LayerCleanupProof`
- `contextra-crypto`: `/app/crates/contextra-crypto/src/deletion_proof/types.rs:168` -> `pub struct LayerCleanupProof`
### `LayerId`

- `contextra-ports`: `/app/crates/contextra-ports/src/erasure.rs:13` -> `pub struct LayerId`
- `contextra-erasure`: `/app/crates/contextra-erasure/src/types_local.rs:668` -> `pub struct LayerId`
- `contextra-types`: `/app/crates/contextra-types/src/erasure.rs:148` -> `pub struct LayerId`
### `LayerScope`

- `contextra-ports`: `/app/crates/contextra-ports/src/erasure.rs:70` -> `pub struct LayerScope`
- `contextra-erasure`: `/app/crates/contextra-erasure/src/types_local.rs:759` -> `pub struct LayerScope`
### `LayerWitness`

- `contextra-ports`: `/app/crates/contextra-ports/src/erasure.rs:183` -> `pub struct LayerWitness`
- `contextra-ports`: `/app/crates/contextra-ports/src/scanner/witness.rs:17` -> `pub struct LayerWitness`
- `contextra-erasure`: `/app/crates/contextra-erasure/src/types_local.rs:944` -> `pub struct LayerWitness`
- `contextra-types`: `/app/crates/contextra-types/src/erasure.rs:555` -> `pub struct LayerWitness`
### `LayerWitnessExt`

- `contextra-erasure`: `/app/crates/contextra-erasure/src/deletion_proof.rs:35` -> `pub trait LayerWitnessExt`
- `contextra-crypto`: `/app/crates/contextra-crypto/src/deletion_proof/types.rs:19` -> `pub trait LayerWitnessExt`
### `LayerWitnessMeasurement`

- `contextra-erasure`: `/app/crates/contextra-erasure/src/deletion_proof.rs:373` -> `pub struct LayerWitnessMeasurement`
- `contextra-crypto`: `/app/crates/contextra-crypto/src/deletion_proof/types.rs:268` -> `pub struct LayerWitnessMeasurement`
### `LedgerEntry`

- `contextra-ports`: `/app/crates/contextra-ports/src/ledger.rs:59` -> `pub struct LedgerEntry`
- `contextra-erasure`: `/app/crates/contextra-erasure/src/types_local.rs:562` -> `pub struct LedgerEntry`
- `contextra-erasure`: `/app/crates/contextra-erasure/src/ledger/entry.rs:10` -> `pub struct LedgerEntry`
### `LedgerSink`

- `contextra-ports`: `/app/crates/contextra-ports/src/ledger.rs:152` -> `pub trait LedgerSink`
- `contextra-erasure`: `/app/crates/contextra-erasure/src/types_local.rs:653` -> `pub trait LedgerSink`
### `LineInfo`

- `contextra-ingest`: `/app/crates/contextra-ingest/src/chunking/prose/markdown_parse.rs:79` -> `struct LineInfo`
- `contextra-engine`: `/app/crates/contextra-engine/src/chunker/mod.rs:42` -> `struct LineInfo`
### `LoomMetadata`

- `contextra-store`: `/app/crates/contextra-store/src/wal/mod.rs:76` -> `pub struct LoomMetadata`
- `contextra-checkpoint`: `/app/crates/contextra-checkpoint/src/hardlink_cloner.rs:112` -> `pub struct LoomMetadata`
### `MeasureClass`

- `contextra-erasure`: `/app/crates/contextra-erasure/src/types_local.rs:709` -> `pub enum MeasureClass`
- `contextra-types`: `/app/crates/contextra-types/src/erasure.rs:297` -> `pub enum MeasureClass`
### `MemoryLink`

- `contextra-rank`: `/app/crates/contextra-rank/src/local_types.rs:51` -> `pub struct MemoryLink`
- `contextra-types`: `/app/crates/contextra-types/src/types/domain/misc.rs:109` -> `pub struct MemoryLink`
### `Meta`

- `contextra-ports`: `/app/crates/contextra-ports/src/checkpoint.rs:28` -> `type_alias Meta`
- `contextra-checkpoint`: `/app/crates/contextra-checkpoint/src/store.rs:952` -> `type_alias Meta`
### `MockEmbedder`

- `contextra-mcp`: `/app/crates/contextra-mcp/src/config.rs:532` -> `pub struct MockEmbedder`
- `contextra-mcp`: `/app/crates/contextra-mcp/src/explain.rs:127` -> `struct MockEmbedder`
- `contextra-mcp`: `/app/crates/contextra-mcp/src/tests.rs:13` -> `struct MockEmbedder`
- `contextra-kvcache`: `/app/crates/contextra-kvcache/src/radix.rs:567` -> `struct MockEmbedder`
- `contextra-testkit`: `/app/crates/contextra-testkit/src/mocks.rs:35` -> `pub struct MockEmbedder`
### `MockLlmGenerator`

- `contextra-mcp`: `/app/crates/contextra-mcp/src/config.rs:592` -> `pub struct MockLlmGenerator`
- `contextra-engine`: `/app/crates/contextra-engine/src/extraction/tests.rs:20` -> `struct MockLlmGenerator`
- `contextra-cognition`: `/app/crates/contextra-cognition/src/memory_consolidation.rs:1053` -> `struct MockLlmGenerator`
- `contextra-cognition`: `/app/crates/contextra-cognition/src/context_compaction/tests.rs:235` -> `struct MockLlmGenerator`
### `MockStorage`

- `contextra-checkpoint`: `/app/crates/contextra-checkpoint/src/guard.rs:518` -> `struct MockStorage`
- `contextra-checkpoint`: `/app/crates/contextra-checkpoint/src/guard/tests.rs:10` -> `struct MockStorage`
- `contextra-checkpoint`: `/app/crates/contextra-checkpoint/src/store/tests.rs:20` -> `struct MockStorage`
- `contextra-text`: `/app/crates/contextra-text/src/wand.rs:631` -> `struct MockStorage`
- `contextra-text`: `/app/crates/contextra-text/src/lib.rs:177` -> `struct MockStorage`
- `contextra-engine`: `/app/crates/contextra-engine/src/orchestration/mod.rs:316` -> `struct MockStorage`
### `ObservedPhase`

- `contextra-ports`: `/app/crates/contextra-ports/src/observer.rs:36` -> `pub enum ObservedPhase`
- `contextra-erasure`: `/app/crates/contextra-erasure/src/types_local.rs:1127` -> `pub enum ObservedPhase`
### `ObserverError`

- `contextra-ports`: `/app/crates/contextra-ports/src/observer.rs:62` -> `pub enum ObserverError`
- `contextra-erasure`: `/app/crates/contextra-erasure/src/types_local.rs:1153` -> `pub enum ObserverError`
### `PendingReason`

- `contextra-erasure`: `/app/crates/contextra-erasure/src/types_local.rs:1240` -> `pub enum PendingReason`
- `contextra-types`: `/app/crates/contextra-types/src/erasure.rs:178` -> `pub enum PendingReason`
### `PercolationConfig`

- `contextra-graph`: `/app/crates/contextra-graph/src/percolation.rs:18` -> `pub struct PercolationConfig`
- `contextra-cognition`: `/app/crates/contextra-cognition/src/maintenance_config.rs:28` -> `pub struct PercolationConfig`
### `Phase`

- `contextra-erasure`: `/app/crates/contextra-erasure/src/types_local.rs:1266` -> `pub enum Phase`
- `contextra-types`: `/app/crates/contextra-types/src/erasure.rs:210` -> `pub enum Phase`
### `PidLatencyController`

- `contextra-db`: `/app/crates/contextra-db/src/pid_latency_controller.rs:14` -> `pub struct PidLatencyController`
- `contextra-adapt`: `/app/crates/contextra-adapt/src/pid_latency_controller.rs:38` -> `pub struct PidLatencyController`
### `PurgeBarrier`

- `contextra-ports`: `/app/crates/contextra-ports/src/barrier.rs:112` -> `pub trait PurgeBarrier`
- `contextra-erasure`: `/app/crates/contextra-erasure/src/types_local.rs:348` -> `pub trait PurgeBarrier`
### `PurgeCx`

- `contextra-ports`: `/app/crates/contextra-ports/src/erasure.rs:118` -> `pub struct PurgeCx`
- `contextra-erasure`: `/app/crates/contextra-erasure/src/types_local.rs:843` -> `pub struct PurgeCx`
- `contextra-erasure`: `/app/crates/contextra-erasure/src/purge/cx.rs:10` -> `pub struct PurgeCx`
### `PurgeOutcome`

- `contextra-ports`: `/app/crates/contextra-ports/src/erasure.rs:132` -> `pub struct PurgeOutcome`
- `contextra-erasure`: `/app/crates/contextra-erasure/src/types_local.rs:824` -> `pub struct PurgeOutcome`
### `PurgePermit`

- `contextra-ports`: `/app/crates/contextra-ports/src/barrier.rs:73` -> `pub struct PurgePermit`
- `contextra-erasure`: `/app/crates/contextra-erasure/src/types_local.rs:322` -> `pub struct PurgePermit`
### `RangeGate`

- `contextra-ports`: `/app/crates/contextra-ports/src/gate.rs:115` -> `pub enum RangeGate`
- `contextra-erasure`: `/app/crates/contextra-erasure/src/types_local.rs:216` -> `pub enum RangeGate`
### `RangeSet`

- `contextra-ports`: `/app/crates/contextra-ports/src/ledger.rs:27` -> `pub struct RangeSet`
- `contextra-erasure`: `/app/crates/contextra-erasure/src/types_local.rs:533` -> `pub struct RangeSet`
### `ReadGate`

- `contextra-ports`: `/app/crates/contextra-ports/src/gate.rs:128` -> `pub trait ReadGate`
- `contextra-erasure`: `/app/crates/contextra-erasure/src/types_local.rs:226` -> `pub trait ReadGate`
### `RecordSealer`

- `contextra-ports`: `/app/crates/contextra-ports/src/keys.rs:128` -> `pub trait RecordSealer`
- `contextra-erasure`: `/app/crates/contextra-erasure/src/types_local.rs:497` -> `pub trait RecordSealer`
### `RefusalReason`

- `contextra-erasure`: `/app/crates/contextra-erasure/src/types_local.rs:1257` -> `pub enum RefusalReason`
- `contextra-types`: `/app/crates/contextra-types/src/erasure.rs:197` -> `pub enum RefusalReason`
### `ReportSignature`

- `contextra-ports`: `/app/crates/contextra-ports/src/report_signer.rs:19` -> `pub struct ReportSignature`
- `contextra-erasure`: `/app/crates/contextra-erasure/src/types_local.rs:1062` -> `pub struct ReportSignature`
### `ReportSigner`

- `contextra-ports`: `/app/crates/contextra-ports/src/report_signer.rs:45` -> `pub trait ReportSigner`
- `contextra-erasure`: `/app/crates/contextra-erasure/src/types_local.rs:1086` -> `pub trait ReportSigner`
- `contextra-erasure`: `/app/crates/contextra-erasure/src/report/signing.rs:232` -> `pub struct ReportSigner`
### `RerankDeadline`

- `contextra-db`: `/app/crates/contextra-db/src/homeostat.rs:10` -> `pub struct RerankDeadline`
- `contextra-adapt`: `/app/crates/contextra-adapt/src/homeostat.rs:102` -> `pub struct RerankDeadline`
### `RerankPidController`

- `contextra-db`: `/app/crates/contextra-db/src/homeostat.rs:32` -> `pub struct RerankPidController`
- `contextra-adapt`: `/app/crates/contextra-adapt/src/homeostat.rs:21` -> `pub struct RerankPidController`
### `Residue`

- `contextra-erasure`: `/app/crates/contextra-erasure/src/types_local.rs:740` -> `pub struct Residue`
- `contextra-types`: `/app/crates/contextra-types/src/erasure.rs:458` -> `pub struct Residue`
### `ResourceBudget`

- `contextra-ingest`: `/app/crates/contextra-ingest/src/extractor.rs:39` -> `pub struct ResourceBudget`
- `contextra-types`: `/app/crates/contextra-types/src/types/budget.rs:341` -> `pub struct ResourceBudget`
### `Result`

- `contextra-simd`: `/app/crates/contextra-simd/src/error.rs:18` -> `pub type_alias Result`
- `contextra-rank`: `/app/crates/contextra-rank/src/error.rs:18` -> `pub type_alias Result`
- `contextra-crypto`: `/app/crates/contextra-crypto/src/error.rs:16` -> `pub type_alias Result`
- `contextra-mcp`: `/app/crates/contextra-mcp/src/sandbox.rs:12` -> `type_alias Result`
- `contextra-types`: `/app/crates/contextra-types/src/error.rs:115` -> `pub type_alias Result`
### `RetentionAttestation`

- `contextra-erasure`: `/app/crates/contextra-erasure/src/types_local.rs:1218` -> `pub struct RetentionAttestation`
- `contextra-types`: `/app/crates/contextra-types/src/erasure.rs:101` -> `pub struct RetentionAttestation`
### `RetentionCode`

- `contextra-erasure`: `/app/crates/contextra-erasure/src/types_local.rs:1201` -> `pub enum RetentionCode`
- `contextra-types`: `/app/crates/contextra-types/src/erasure.rs:80` -> `pub enum RetentionCode`
### `RoleBinding`

- `contextra-ports`: `/app/crates/contextra-ports/src/graph.rs:44` -> `pub struct RoleBinding`
- `contextra-wire`: `/app/crates/contextra-wire/src/contextra_generated.rs:1153` -> `pub struct RoleBinding`
### `RoleId`

- `contextra-ports`: `/app/crates/contextra-ports/src/graph.rs:14` -> `pub struct RoleId`
- `contextra-wire`: `/app/crates/contextra-wire/src/contextra_generated.rs:909` -> `pub struct RoleId`
### `ScanTechniqueSerde`

- `contextra-erasure`: `/app/crates/contextra-erasure/src/deletion_proof.rs:331` -> `enum ScanTechniqueSerde`
- `contextra-crypto`: `/app/crates/contextra-crypto/src/deletion_proof/types.rs:226` -> `enum ScanTechniqueSerde`
### `Scanner`

- `contextra-ports`: `/app/crates/contextra-ports/src/erasure.rs:156` -> `pub struct Scanner`
- `contextra-erasure`: `/app/crates/contextra-erasure/src/types_local.rs:856` -> `pub struct Scanner`
- `contextra-types`: `/app/crates/contextra-types/src/erasure.rs:481` -> `pub struct Scanner`
### `ScoredDocument`

- `contextra-wire`: `/app/crates/contextra-wire/src/contextra_generated.rs:255` -> `pub struct ScoredDocument`
- `contextra-types`: `/app/crates/contextra-types/src/types/domain/document.rs:221` -> `pub struct ScoredDocument`
### `ScrubOutcome`

- `contextra-erasure`: `/app/crates/contextra-erasure/src/vault/scrub.rs:27` -> `pub struct ScrubOutcome`
- `contextra-durable-fs`: `/app/crates/contextra-durable-fs/src/scrub.rs:72` -> `pub enum ScrubOutcome`
### `SealCtx`

- `contextra-ports`: `/app/crates/contextra-ports/src/keys.rs:84` -> `pub struct SealCtx`
- `contextra-erasure`: `/app/crates/contextra-erasure/src/types_local.rs:454` -> `pub struct SealCtx`
### `SecretKey`

- `contextra-erasure`: `/app/crates/contextra-erasure/src/crypto_local.rs:15` -> `pub struct SecretKey`
- `contextra-crypto`: `/app/crates/contextra-crypto/src/secret.rs:8` -> `pub struct SecretKey`
### `Seq`

- `contextra-ports`: `/app/crates/contextra-ports/src/gate.rs:10` -> `pub struct Seq`
- `contextra-erasure`: `/app/crates/contextra-erasure/src/types_local.rs:54` -> `pub struct Seq`
- `contextra-types`: `/app/crates/contextra-types/src/ids.rs:185` -> `pub struct Seq`
### `ShadowDiscrepancy`

- `contextra-graph`: `/app/crates/contextra-graph/src/ppr/shadow_hook.rs:8` -> `pub struct ShadowDiscrepancy`
- `contextra-adapt`: `/app/crates/contextra-adapt/src/shadow_mode.rs:11` -> `pub struct ShadowDiscrepancy`
### `ShadowSink`

- `contextra-graph`: `/app/crates/contextra-graph/src/ppr/shadow_hook.rs:18` -> `pub trait ShadowSink`
- `contextra-adapt`: `/app/crates/contextra-adapt/src/shadow_mode.rs:21` -> `pub trait ShadowSink`
### `Shard`

- `contextra-crypto`: `/app/crates/contextra-crypto/src/kv_segment/store.rs:100` -> `struct Shard`
- `contextra-kvcache`: `/app/crates/contextra-kvcache/src/store.rs:311` -> `struct Shard`
### `Signal`

- `contextra-rank`: `/app/crates/contextra-rank/src/fusion/types.rs:171` -> `pub enum Signal`
- `contextra-types`: `/app/crates/contextra-types/src/types/saos.rs:319` -> `pub enum Signal`
### `SignalFusionStrategies`

- `contextra-rank`: `/app/crates/contextra-rank/src/local_types.rs:57` -> `pub struct SignalFusionStrategies`
- `contextra-types`: `/app/crates/contextra-types/src/types/saos.rs:43` -> `pub struct SignalFusionStrategies`
### `SignerError`

- `contextra-ports`: `/app/crates/contextra-ports/src/report_signer.rs:30` -> `pub enum SignerError`
- `contextra-erasure`: `/app/crates/contextra-erasure/src/types_local.rs:1073` -> `pub enum SignerError`
### `SignerId`

- `contextra-ports`: `/app/crates/contextra-ports/src/report_signer.rs:15` -> `pub type_alias SignerId`
- `contextra-erasure`: `/app/crates/contextra-erasure/src/types_local.rs:1058` -> `pub type_alias SignerId`
### `SigningKeyHandle`

- `contextra-erasure`: `/app/crates/contextra-erasure/src/deletion_proof.rs:161` -> `pub enum SigningKeyHandle`
- `contextra-crypto`: `/app/crates/contextra-crypto/src/deletion_proof/types.rs:108` -> `pub enum SigningKeyHandle`
### `SimpleRng`

- `contextra-router`: `/app/crates/contextra-router/src/bandit_regret_tests.rs:18` -> `struct SimpleRng`
- `contextra-graph`: `/app/crates/contextra-graph/src/community.rs:151` -> `struct SimpleRng`
### `SpillHandler`

- `contextra-crypto`: `/app/crates/contextra-crypto/src/kv_segment/store.rs:26` -> `pub type_alias SpillHandler`
- `contextra-kvcache`: `/app/crates/contextra-kvcache/src/store.rs:44` -> `pub type_alias SpillHandler`
### `SplitMix64`

- `contextra-adapt`: `/app/crates/contextra-adapt/src/flow_thompson.rs:90` -> `pub struct SplitMix64`
- `contextra-cognition`: `/app/crates/contextra-cognition/src/aggregation_phase.rs:230` -> `struct SplitMix64`
### `StagedOp`

- `contextra-store`: `/app/crates/contextra-store/src/engine/memory_only.rs:15` -> `enum StagedOp`
- `contextra-testkit`: `/app/crates/contextra-testkit/src/in_memory_store.rs:39` -> `enum StagedOp`
### `StagedTxMap`

- `contextra-store`: `/app/crates/contextra-store/src/engine/memory_only.rs:28` -> `type_alias StagedTxMap`
- `contextra-testkit`: `/app/crates/contextra-testkit/src/in_memory_store.rs:49` -> `type_alias StagedTxMap`
### `StorageBackend`

- `contextra-ports`: `/app/crates/contextra-ports/src/erasure.rs:299` -> `pub trait StorageBackend`
- `contextra-erasure`: `/app/crates/contextra-erasure/src/backend.rs:12` -> `pub trait StorageBackend`
### `StoreDataMap`

- `contextra-store`: `/app/crates/contextra-store/src/engine/memory_only.rs:27` -> `type_alias StoreDataMap`
- `contextra-testkit`: `/app/crates/contextra-testkit/src/in_memory_store.rs:47` -> `type_alias StoreDataMap`
### `Target`

- `contextra-sys`: `/app/crates/contextra-sys/src/mmap.rs:185` -> `type_alias Target`
- `contextra-graph`: `/app/crates/contextra-graph/src/arc_slice.rs:115` -> `type_alias Target`
- `contextra-graph`: `/app/crates/contextra-graph/src/hyperedge.rs:257` -> `type_alias Target`
- `contextra-graph`: `/app/crates/contextra-graph/src/csr/inner/mod.rs:158` -> `type_alias Target`
- `contextra-adapt`: `/app/crates/contextra-adapt/src/bandit.rs:102` -> `type_alias Target`
### `TargetKind`

- `contextra-ports`: `/app/crates/contextra-ports/src/observer.rs:19` -> `pub enum TargetKind`
- `contextra-erasure`: `/app/crates/contextra-erasure/src/types_local.rs:1110` -> `pub enum TargetKind`
### `TenantIsolatedKvStore`

- `contextra-crypto`: `/app/crates/contextra-crypto/src/kv_segment/store.rs:117` -> `pub struct TenantIsolatedKvStore`
- `contextra-kvcache`: `/app/crates/contextra-kvcache/src/store.rs:328` -> `pub struct TenantIsolatedKvStore`
### `TenantState`

- `contextra-crypto`: `/app/crates/contextra-crypto/src/kv_segment/store.rs:31` -> `struct TenantState`
- `contextra-kvcache`: `/app/crates/contextra-kvcache/src/store.rs:59` -> `struct TenantState`
### `TestClock`

- `contextra-vector`: `/app/crates/contextra-vector/src/hnsw/core_rebuild.rs:981` -> `struct TestClock`
- `contextra-cognition`: `/app/crates/contextra-cognition/src/maintenance_scheduler.rs:511` -> `struct TestClock`
### `TestMetricsSink`

- `contextra-testkit`: `/app/crates/contextra-testkit/src/mocks.rs:136` -> `pub struct TestMetricsSink`
- `contextra-vector`: `/app/crates/contextra-vector/src/hnsw/tests.rs:230` -> `struct TestMetricsSink`
### `Timestamp`

- `contextra-ports`: `/app/crates/contextra-ports/src/ledger.rs:11` -> `pub struct Timestamp`
- `contextra-ports`: `/app/crates/contextra-ports/src/observer.rs:15` -> `pub type_alias Timestamp`
- `contextra-erasure`: `/app/crates/contextra-erasure/src/types_local.rs:70` -> `pub struct Timestamp`
- `contextra-types`: `/app/crates/contextra-types/src/ids.rs:201` -> `pub struct Timestamp`
### `Timing`

- `contextra-erasure`: `/app/crates/contextra-erasure/src/types_local.rs:1192` -> `pub enum Timing`
- `contextra-types`: `/app/crates/contextra-types/src/erasure.rs:54` -> `pub enum Timing`
### `TlHfdParams`

- `contextra-graph`: `/app/crates/contextra-graph/src/tl_hfd/params.rs:11` -> `pub struct TlHfdParams`
- `contextra-types`: `/app/crates/contextra-types/src/types/domain/misc.rs:385` -> `pub struct TlHfdParams`
### `Unarmed`

- `contextra-erasure`: `/app/crates/contextra-erasure/src/types_local.rs:720` -> `pub struct Unarmed`
- `contextra-types`: `/app/crates/contextra-types/src/erasure.rs:427` -> `pub struct Unarmed`
### `UncoveredLayer`

- `contextra-erasure`: `/app/crates/contextra-erasure/src/deletion_proof.rs:319` -> `pub struct UncoveredLayer`
- `contextra-crypto`: `/app/crates/contextra-crypto/src/deletion_proof/types.rs:214` -> `pub struct UncoveredLayer`
### `UnitKey`

- `contextra-erasure`: `/app/crates/contextra-erasure/src/crypto_local.rs:43` -> `pub struct UnitKey`
- `contextra-crypto`: `/app/crates/contextra-crypto/src/dek.rs:8` -> `pub struct UnitKey`
### `Value`

- `contextra-ports`: `/app/crates/contextra-ports/src/kv.rs:79` -> `type_alias Value`
- `contextra-ports`: `/app/crates/contextra-ports/src/conformance.rs:193` -> `type_alias Value`
- `contextra-ingest`: `/app/crates/contextra-ingest/src/formats/structured/json.rs:44` -> `type_alias Value`
- `contextra-ingest`: `/app/crates/contextra-ingest/src/formats/structured/json.rs:55` -> `type_alias Value`
### `VerificationKey`

- `contextra-erasure`: `/app/crates/contextra-erasure/src/deletion_proof.rs:188` -> `pub enum VerificationKey`
- `contextra-crypto`: `/app/crates/contextra-crypto/src/deletion_proof/types.rs:135` -> `pub enum VerificationKey`
### `VerifiedVia`

- `contextra-ports`: `/app/crates/contextra-ports/src/vfs.rs:30` -> `pub enum VerifiedVia`
- `contextra-erasure`: `/app/crates/contextra-erasure/src/types_local.rs:1321` -> `pub enum VerifiedVia`
- `contextra-erasure`: `/app/crates/contextra-erasure/src/vault/table.rs:25` -> `pub enum VerifiedVia`
- `contextra-erasure`: `/app/crates/contextra-erasure/src/measure/field.rs:14` -> `pub enum VerifiedVia`
### `VfsFile`

- `contextra-ports`: `/app/crates/contextra-ports/src/vfs.rs:76` -> `pub trait VfsFile`
- `contextra-erasure`: `/app/crates/contextra-erasure/src/types_local.rs:1500` -> `pub trait VfsFile`
### `VfsOpenOptions`

- `contextra-ports`: `/app/crates/contextra-ports/src/vfs.rs:41` -> `pub struct VfsOpenOptions`
- `contextra-erasure`: `/app/crates/contextra-erasure/src/types_local.rs:1332` -> `pub struct VfsOpenOptions`
### `WorkflowState`

- `contextra-engine`: `/app/crates/contextra-engine/src/orchestration/mod.rs:101` -> `pub struct WorkflowState`
- `contextra-types`: `/app/crates/contextra-types/src/types/domain/misc.rs:61` -> `pub struct WorkflowState`
