# Typen-Register & Kollisions-Audit (K3 / A2)

> Generiert durch `tools/type_registry_sync.py`.
> Einzigartige Typnamen: 1284 | Gesamte Definitionen: 1608
> Erkannte typografische Crate-Übergreifende Kollisionen: **165**

## Crate Typen-Statistik

| Crate | Definierte Typen (Structs/Enums/Traits/Aliases) |
|---|---|
| `contextra` | 20 |
| `contextra-checkpoint` | 39 |
| `contextra-crypto` | 53 |
| `contextra-db` | 111 |
| `contextra-durable-fs` | 30 |
| `contextra-env` | 14 |
| `contextra-erasure` | 371 |
| `contextra-graph` | 142 |
| `contextra-ids` | 16 |
| `contextra-mcp` | 34 |
| `contextra-mvcc` | 29 |
| `contextra-ports` | 139 |
| `contextra-privacy` | 67 |
| `contextra-rank` | 57 |
| `contextra-simd` | 12 |
| `contextra-store` | 145 |
| `contextra-sys` | 18 |
| `contextra-testkit` | 37 |
| `contextra-text` | 75 |
| `contextra-types` | 82 |
| `contextra-vault` | 11 |
| `contextra-vector` | 106 |

## Kollisionen über Crate-Grenzen (Identische Typnamen)

| Typname | Beteiligte Crates | Definitionen |
|---|---|---|
| `AncestorSalts` | `contextra-crypto`, `contextra-erasure` | 2 |
| `Armed` | `contextra-erasure`, `contextra-ports`, `contextra-types` | 4 |
| `BarrierExclusive` | `contextra-erasure`, `contextra-ports` | 2 |
| `BarrierShared` | `contextra-erasure`, `contextra-ports` | 2 |
| `BoundedTopK` | `contextra-rank`, `contextra-text` | 2 |
| `BoxFuture` | `contextra-checkpoint`, `contextra-erasure`, `contextra-ports`, `contextra-privacy`, `contextra-text` | 11 |
| `Clock` | `contextra-env`, `contextra-erasure`, `contextra-mcp`, `contextra-privacy`, `contextra-rank`, `contextra-text`, `contextra-vector` | 8 |
| `CollectionId` | `contextra-erasure`, `contextra-ids` | 2 |
| `CommitBarrier` | `contextra-erasure`, `contextra-ports` | 2 |
| `CommunityDetectionConfig` | `contextra-db`, `contextra-graph` | 2 |
| `CompactionPermit` | `contextra-erasure`, `contextra-ports` | 2 |
| `ConfigFingerprint` | `contextra-rank`, `contextra-types` | 2 |
| `ControlEvidence` | `contextra-erasure`, `contextra-types` | 3 |
| `CosineSimilarityPartsU8` | `contextra-simd`, `contextra-vector` | 2 |
| `CryptoError` | `contextra-crypto`, `contextra-erasure` | 2 |
| `Dek` | `contextra-crypto`, `contextra-erasure` | 2 |
| `DirLock` | `contextra-durable-fs`, `contextra-store` | 2 |
| `DistanceFn` | `contextra-simd`, `contextra-vector` | 2 |
| `DistanceMetric` | `contextra-simd`, `contextra-types`, `contextra-vector` | 3 |
| `DocId` | `contextra-erasure`, `contextra-graph`, `contextra-ids`, `contextra-rank`, `contextra-text` | 7 |
| `DocKey` | `contextra-erasure`, `contextra-text` | 2 |
| `DocRef` | `contextra-erasure`, `contextra-ids` | 2 |
| `DurabilityConfigError` | `contextra`, `contextra-store` | 2 |
| `DurabilityMode` | `contextra`, `contextra-store` | 2 |
| `Edge` | `contextra-graph`, `contextra-types` | 2 |
| `EgressDecision` | `contextra-ports`, `contextra-privacy` | 2 |
| `EgressPolicy` | `contextra-ports`, `contextra-privacy` | 2 |
| `Entity` | `contextra-graph`, `contextra-types` | 2 |
| `EntityExtractor` | `contextra-db`, `contextra-graph` | 2 |
| `EntityId` | `contextra-graph`, `contextra-ids` | 2 |
| `Envelope` | `contextra-erasure`, `contextra-ports` | 2 |
| `Epoch` | `contextra-erasure`, `contextra-text` | 2 |
| `EraseMode` | `contextra-erasure`, `contextra-types` | 3 |
| `EraseReceipt` | `contextra-erasure`, `contextra-types` | 3 |
| `EraseRequest` | `contextra-erasure`, `contextra-types` | 3 |
| `EraseTarget` | `contextra-erasure`, `contextra-ports`, `contextra-types` | 4 |
| `ErasureEvent` | `contextra-erasure`, `contextra-ports` | 2 |
| `ErasureId` | `contextra-erasure`, `contextra-types` | 2 |
| `ErasureLayer` | `contextra-erasure`, `contextra-ports` | 2 |
| `ErasureObserver` | `contextra-erasure`, `contextra-ports` | 2 |
| `ErasureStatus` | `contextra-erasure`, `contextra-types` | 3 |
| `ErasureTarget` | `contextra-erasure`, `contextra-ports` | 2 |
| `Error` | `contextra-db`, `contextra-durable-fs`, `contextra-erasure`, `contextra-ids`, `contextra-privacy`, `contextra-rank`, `contextra-simd`, `contextra-sys`, `contextra-text`, `contextra-vector` | 11 |
| `FaultConfig` | `contextra-graph`, `contextra-testkit` | 2 |
| `FaultVfs` | `contextra-graph`, `contextra-testkit` | 2 |
| `FileId` | `contextra-erasure`, `contextra-ids`, `contextra-ports` | 3 |
| `FileKey` | `contextra-erasure`, `contextra-ports` | 2 |
| `FileKeyring` | `contextra-erasure`, `contextra-ports` | 2 |
| `FileKind` | `contextra-erasure`, `contextra-ports` | 2 |
| `FixedTestClock` | `contextra-privacy`, `contextra-rank` | 2 |
| `FusionResult` | `contextra-db`, `contextra-rank` | 2 |
| `FusionStrategy` | `contextra-rank`, `contextra-types` | 2 |
| `FusionWeights` | `contextra-rank`, `contextra-types` | 2 |
| `FuzzInput` | `contextra-graph`, `contextra-mcp`, `contextra-store` | 4 |
| `GateDecision` | `contextra-erasure`, `contextra-ports` | 2 |
| `GateVariant` | `contextra-erasure`, `contextra-types` | 2 |
| `GraphMutationError` | `contextra-graph`, `contextra-ports` | 2 |
| `HeapItem` | `contextra-graph`, `contextra-store` | 2 |
| `Hit` | `contextra-text`, `contextra-vector` | 6 |
| `HybridQueryBuilder` | `contextra-db`, `contextra-types` | 2 |
| `HyperEdgeId` | `contextra-graph`, `contextra-ports` | 2 |
| `IdGen` | `contextra-env`, `contextra-erasure` | 2 |
| `InMemoryStorageEngine` | `contextra-graph`, `contextra-store`, `contextra-testkit` | 3 |
| `Index` | `contextra-text`, `contextra-vector` | 3 |
| `JournalRecord` | `contextra-durable-fs`, `contextra-erasure` | 2 |
| `KeyBound` | `contextra-erasure`, `contextra-ports` | 2 |
| `KeyBuf` | `contextra-erasure`, `contextra-ports` | 2 |
| `KeyCoverage` | `contextra-erasure`, `contextra-ports`, `contextra-types` | 4 |
| `KeyMaterialSource` | `contextra-ports`, `contextra-store` | 2 |
| `KeyRange` | `contextra-erasure`, `contextra-ports` | 2 |
| `KvDeleteMode` | `contextra`, `contextra-crypto` | 2 |
| `LayerId` | `contextra-erasure`, `contextra-ports`, `contextra-types` | 4 |
| `LayerScanner` | `contextra-erasure`, `contextra-ports` | 2 |
| `LayerScope` | `contextra-erasure`, `contextra-ports` | 2 |
| `LayerWitness` | `contextra-erasure`, `contextra-ports`, `contextra-types` | 4 |
| `LedgerEntry` | `contextra-erasure`, `contextra-ports` | 3 |
| `LedgerSink` | `contextra-erasure`, `contextra-ports` | 2 |
| `LoomMetadata` | `contextra-checkpoint`, `contextra-store` | 2 |
| `Maintain` | `contextra-text`, `contextra-vector` | 4 |
| `ManifestEntry` | `contextra-erasure`, `contextra-store` | 2 |
| `ManualClock` | `contextra-env`, `contextra-testkit` | 2 |
| `MeasureClass` | `contextra-erasure`, `contextra-types` | 3 |
| `MemVfs` | `contextra-erasure`, `contextra-ports` | 2 |
| `Meta` | `contextra-checkpoint`, `contextra-ports` | 2 |
| `MockEmbedder` | `contextra-mcp`, `contextra-testkit` | 4 |
| `MockLlmGenerator` | `contextra-db`, `contextra-mcp` | 2 |
| `MockStorage` | `contextra-checkpoint`, `contextra-db`, `contextra-text` | 5 |
| `ModelFingerprint` | `contextra-crypto`, `contextra-types` | 2 |
| `ObservedPhase` | `contextra-erasure`, `contextra-ports` | 2 |
| `ObserverError` | `contextra-erasure`, `contextra-ports` | 2 |
| `PendingReason` | `contextra-erasure`, `contextra-types` | 3 |
| `Phase` | `contextra-erasure`, `contextra-types` | 3 |
| `PprAlgorithm` | `contextra-graph`, `contextra-types` | 2 |
| `PprConfig` | `contextra-graph`, `contextra-types` | 2 |
| `PurgeBarrier` | `contextra-erasure`, `contextra-ports` | 2 |
| `PurgeCx` | `contextra-erasure`, `contextra-ports` | 3 |
| `PurgeOutcome` | `contextra-erasure`, `contextra-ports` | 2 |
| `PurgePermit` | `contextra-erasure`, `contextra-ports` | 2 |
| `PurgeStats` | `contextra-erasure`, `contextra-ports` | 2 |
| `Purger` | `contextra-erasure`, `contextra-ports` | 2 |
| `RangeGate` | `contextra-erasure`, `contextra-ports` | 2 |
| `RangeSet` | `contextra-erasure`, `contextra-ports` | 2 |
| `ReadGate` | `contextra-erasure`, `contextra-ports` | 2 |
| `RealFs` | `contextra-durable-fs`, `contextra-store` | 3 |
| `RecordSealer` | `contextra-erasure`, `contextra-ports`, `contextra-store` | 3 |
| `RefusalReason` | `contextra-erasure`, `contextra-types` | 3 |
| `ReportSignature` | `contextra-erasure`, `contextra-ports` | 2 |
| `ReportSigner` | `contextra-erasure`, `contextra-ports` | 3 |
| `RequestContext` | `contextra-erasure`, `contextra-types` | 2 |
| `ResidualRisk` | `contextra-erasure`, `contextra-types` | 2 |
| `Residue` | `contextra-erasure`, `contextra-types` | 3 |
| `ResourceBudget` | `contextra-graph`, `contextra-store`, `contextra-types` | 3 |
| `ResourceTracker` | `contextra-graph`, `contextra-store`, `contextra-types` | 3 |
| `Result` | `contextra-checkpoint`, `contextra-crypto`, `contextra-durable-fs`, `contextra-erasure`, `contextra-graph`, `contextra-ids`, `contextra-mcp`, `contextra-mvcc`, `contextra-privacy`, `contextra-rank`, `contextra-simd`, `contextra-sys`, `contextra-text`, `contextra-types`, `contextra-vector` | 18 |
| `RetentionAttestation` | `contextra-erasure`, `contextra-types` | 3 |
| `RetentionCode` | `contextra-erasure`, `contextra-types` | 3 |
| `RiskCondition` | `contextra-erasure`, `contextra-types` | 2 |
| `Rng` | `contextra-env`, `contextra-text`, `contextra-vector` | 3 |
| `RoleBinding` | `contextra-graph`, `contextra-ports` | 2 |
| `RoleId` | `contextra-graph`, `contextra-ports` | 2 |
| `SagaRecord` | `contextra-erasure`, `contextra-types` | 2 |
| `ScanMethod` | `contextra-erasure`, `contextra-types` | 4 |
| `ScanSession` | `contextra-erasure`, `contextra-ports` | 2 |
| `ScanTarget` | `contextra-erasure`, `contextra-ports` | 2 |
| `ScanTechnique` | `contextra-erasure`, `contextra-ports` | 2 |
| `Scanner` | `contextra-erasure`, `contextra-ports`, `contextra-types` | 4 |
| `ScoredDocument` | `contextra-types`, `contextra-vector` | 2 |
| `ScrubOutcome` | `contextra-durable-fs`, `contextra-erasure` | 2 |
| `SealCtx` | `contextra-erasure`, `contextra-ports` | 2 |
| `SecretKey` | `contextra-crypto`, `contextra-erasure` | 2 |
| `Seq` | `contextra-erasure`, `contextra-ids`, `contextra-ports` | 3 |
| `SeqBitTombstone` | `contextra-mvcc`, `contextra-types` | 2 |
| `SequentialIdGen` | `contextra-env`, `contextra-erasure` | 2 |
| `Signal` | `contextra-rank`, `contextra-types` | 2 |
| `SignalFusionStrategies` | `contextra-rank`, `contextra-types` | 2 |
| `SignerError` | `contextra-erasure`, `contextra-ports` | 2 |
| `SignerId` | `contextra-erasure`, `contextra-ports` | 2 |
| `StagedOp` | `contextra-store`, `contextra-testkit` | 2 |
| `StagedTxMap` | `contextra-store`, `contextra-testkit` | 2 |
| `Stats` | `contextra-text`, `contextra-vector` | 5 |
| `StorageBackend` | `contextra-erasure`, `contextra-ports` | 2 |
| `StorageStats` | `contextra-ports`, `contextra-store` | 2 |
| `StoreDataMap` | `contextra-store`, `contextra-testkit` | 2 |
| `SystemClock` | `contextra-env`, `contextra-erasure`, `contextra-mcp`, `contextra-privacy`, `contextra-rank` | 5 |
| `Target` | `contextra-graph`, `contextra-sys` | 5 |
| `TargetKind` | `contextra-erasure`, `contextra-ports` | 2 |
| `TenantId` | `contextra-erasure`, `contextra-ids`, `contextra-privacy` | 3 |
| `TenantScopeViolation` | `contextra-ids`, `contextra-privacy` | 2 |
| `TenantScoped` | `contextra-ids`, `contextra-privacy` | 2 |
| `TextGenerator` | `contextra-graph`, `contextra-ports` | 2 |
| `Timestamp` | `contextra-erasure`, `contextra-ids`, `contextra-ports` | 4 |
| `Timing` | `contextra-erasure`, `contextra-types` | 3 |
| `TlHfdParams` | `contextra-graph`, `contextra-types` | 3 |
| `TombstoneSemanticsCheck` | `contextra-mvcc`, `contextra-types` | 2 |
| `TxId` | `contextra-erasure`, `contextra-graph`, `contextra-ids`, `contextra-rank` | 4 |
| `Unarmed` | `contextra-erasure`, `contextra-types` | 3 |
| `UnitKey` | `contextra-crypto`, `contextra-erasure` | 2 |
| `ValidityWindow` | `contextra-db`, `contextra-rank` | 2 |
| `VectorIndex` | `contextra-ports`, `contextra-vector` | 2 |
| `VerifiedVia` | `contextra-erasure`, `contextra-ports`, `contextra-store` | 6 |
| `Vfs` | `contextra-durable-fs`, `contextra-erasure`, `contextra-ports`, `contextra-store` | 5 |
| `VfsFile` | `contextra-erasure`, `contextra-ports`, `contextra-store` | 4 |
| `VfsOpenOptions` | `contextra-erasure`, `contextra-ports`, `contextra-store` | 4 |
| `WitnessLayer` | `contextra-erasure`, `contextra-ports` | 2 |
| `WorkflowState` | `contextra-checkpoint`, `contextra-db`, `contextra-types` | 3 |

## Detailierte Kollisionsorte

### `AncestorSalts`

- `contextra-crypto`: `/home/freddy/Projekte/contextra/crates/contextra-crypto/src/dek.rs:51` -> `pub struct AncestorSalts`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/crypto_local.rs:47` -> `pub struct AncestorSalts`
### `Armed`

- `contextra-ports`: `/home/freddy/Projekte/contextra/crates/contextra-ports/src/erasure.rs:151` -> `pub struct Armed`
- `contextra-types`: `/home/freddy/Projekte/contextra/crates/contextra-types/src/erasure.rs:438` -> `pub struct Armed`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:735` -> `pub struct Armed`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/erasure_domain.rs:454` -> `pub struct Armed`
### `BarrierExclusive`

- `contextra-ports`: `/home/freddy/Projekte/contextra/crates/contextra-ports/src/barrier.rs:37` -> `pub struct BarrierExclusive`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:297` -> `pub struct BarrierExclusive`
### `BarrierShared`

- `contextra-ports`: `/home/freddy/Projekte/contextra/crates/contextra-ports/src/barrier.rs:19` -> `pub struct BarrierShared`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:279` -> `pub struct BarrierShared`
### `BoundedTopK`

- `contextra-text`: `/home/freddy/Projekte/contextra/crates/contextra-text/src/wand.rs:50` -> `pub struct BoundedTopK`
- `contextra-rank`: `/home/freddy/Projekte/contextra/crates/contextra-rank/src/fusion/topk.rs:24` -> `pub struct BoundedTopK`
### `BoxFuture`

- `contextra-text`: `/home/freddy/Projekte/contextra/crates/contextra-text/src/storage.rs:10` -> `pub type_alias BoxFuture`
- `contextra-checkpoint`: `/home/freddy/Projekte/contextra/crates/contextra-checkpoint/src/snapshot_source.rs:12` -> `pub type_alias BoxFuture`
- `contextra-privacy`: `/home/freddy/Projekte/contextra/crates/contextra-privacy/src/egress_vault.rs:178` -> `pub type_alias BoxFuture`
- `contextra-ports`: `/home/freddy/Projekte/contextra/crates/contextra-ports/src/lib.rs:33` -> `pub type_alias BoxFuture`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/backend.rs:13` -> `pub type_alias BoxFuture`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:12` -> `pub type_alias BoxFuture`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/test_mocks.rs:11` -> `pub type_alias BoxFuture`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/vault/sealer.rs:21` -> `pub type_alias BoxFuture`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/vault/keyring.rs:18` -> `pub type_alias BoxFuture`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/report/signing.rs:19` -> `pub type_alias BoxFuture`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/saga/seams.rs:18` -> `pub type_alias BoxFuture`
### `Clock`

- `contextra-text`: `/home/freddy/Projekte/contextra/crates/contextra-text/src/contract.rs:33` -> `pub trait Clock`
- `contextra-env`: `/home/freddy/Projekte/contextra/crates/contextra-env/src/clock.rs:18` -> `pub trait Clock`
- `contextra-privacy`: `/home/freddy/Projekte/contextra/crates/contextra-privacy/src/types.rs:11` -> `pub trait Clock`
- `contextra-mcp`: `/home/freddy/Projekte/contextra/crates/contextra-mcp/src/prompt_injection.rs:17` -> `pub trait Clock`
- `contextra-vector`: `/home/freddy/Projekte/contextra/crates/contextra-vector/src/contract.rs:60` -> `pub trait Clock`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:2094` -> `pub trait Clock`
- `contextra-rank`: `/home/freddy/Projekte/contextra/crates/contextra-rank/src/contract.rs:7` -> `pub trait Clock`
- `contextra-rank`: `/home/freddy/Projekte/contextra/crates/contextra-rank/src/calibration/isotonic.rs:14` -> `pub trait Clock`
### `CollectionId`

- `contextra-ids`: `/home/freddy/Projekte/contextra/crates/contextra-ids/src/lib.rs:148` -> `pub struct CollectionId`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:1615` -> `pub struct CollectionId`
### `CommitBarrier`

- `contextra-ports`: `/home/freddy/Projekte/contextra/crates/contextra-ports/src/barrier.rs:91` -> `pub trait CommitBarrier`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:351` -> `pub trait CommitBarrier`
### `CommunityDetectionConfig`

- `contextra-graph`: `/home/freddy/Projekte/contextra/crates/contextra-graph/src/community.rs:33` -> `pub struct CommunityDetectionConfig`
- `contextra-db`: `/home/freddy/Projekte/contextra/crates/contextra-db/src/lib.rs:173` -> `pub struct CommunityDetectionConfig`
### `CompactionPermit`

- `contextra-ports`: `/home/freddy/Projekte/contextra/crates/contextra-ports/src/barrier.rs:55` -> `pub struct CompactionPermit`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:315` -> `pub struct CompactionPermit`
### `ConfigFingerprint`

- `contextra-types`: `/home/freddy/Projekte/contextra/crates/contextra-types/src/types/domain/misc.rs:529` -> `pub struct ConfigFingerprint`
- `contextra-rank`: `/home/freddy/Projekte/contextra/crates/contextra-rank/src/local_types.rs:56` -> `pub struct ConfigFingerprint`
### `ControlEvidence`

- `contextra-types`: `/home/freddy/Projekte/contextra/crates/contextra-types/src/erasure.rs:446` -> `pub struct ControlEvidence`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:739` -> `pub struct ControlEvidence`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/erasure_domain.rs:462` -> `pub struct ControlEvidence`
### `CosineSimilarityPartsU8`

- `contextra-simd`: `/home/freddy/Projekte/contextra/crates/contextra-simd/src/kernels/scalar.rs:28` -> `pub struct CosineSimilarityPartsU8`
- `contextra-vector`: `/home/freddy/Projekte/contextra/crates/contextra-vector/src/distance/kernel.rs:19` -> `pub struct CosineSimilarityPartsU8`
### `CryptoError`

- `contextra-crypto`: `/home/freddy/Projekte/contextra/crates/contextra-crypto/src/error.rs:27` -> `pub enum CryptoError`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/deletion_proof.rs:28` -> `pub type_alias CryptoError`
### `Dek`

- `contextra-crypto`: `/home/freddy/Projekte/contextra/crates/contextra-crypto/src/dek.rs:35` -> `pub struct Dek`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/crypto_local.rs:39` -> `pub struct Dek`
### `DirLock`

- `contextra-store`: `/home/freddy/Projekte/contextra/crates/contextra-store/src/util.rs:81` -> `pub struct DirLock`
- `contextra-durable-fs`: `/home/freddy/Projekte/contextra/crates/contextra-durable-fs/src/lib.rs:256` -> `pub struct DirLock`
### `DistanceFn`

- `contextra-simd`: `/home/freddy/Projekte/contextra/crates/contextra-simd/src/contract.rs:21` -> `pub trait DistanceFn`
- `contextra-vector`: `/home/freddy/Projekte/contextra/crates/contextra-vector/src/contract.rs:113` -> `pub trait DistanceFn`
### `DistanceMetric`

- `contextra-simd`: `/home/freddy/Projekte/contextra/crates/contextra-simd/src/metric.rs:14` -> `pub enum DistanceMetric`
- `contextra-vector`: `/home/freddy/Projekte/contextra/crates/contextra-vector/src/types.rs:27` -> `pub enum DistanceMetric`
- `contextra-types`: `/home/freddy/Projekte/contextra/crates/contextra-types/src/types/domain/document.rs:20` -> `pub enum DistanceMetric`
### `DocId`

- `contextra-text`: `/home/freddy/Projekte/contextra/crates/contextra-text/src/types.rs:9` -> `pub struct DocId`
- `contextra-text`: `/home/freddy/Projekte/contextra/crates/contextra-text/src/types.rs:15` -> `pub struct DocId`
- `contextra-graph`: `/home/freddy/Projekte/contextra/crates/contextra-graph/src/types.rs:55` -> `pub struct DocId`
- `contextra-ids`: `/home/freddy/Projekte/contextra/crates/contextra-ids/src/lib.rs:224` -> `pub struct DocId`
- `contextra-ids`: `/home/freddy/Projekte/contextra/crates/contextra-ids/src/lib.rs:230` -> `pub struct DocId`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:1639` -> `pub struct DocId`
- `contextra-rank`: `/home/freddy/Projekte/contextra/crates/contextra-rank/src/local_types.rs:10` -> `pub struct DocId`
### `DocKey`

- `contextra-text`: `/home/freddy/Projekte/contextra/crates/contextra-text/src/contract.rs:53` -> `pub struct DocKey`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/erasure_domain.rs:24` -> `pub struct DocKey`
### `DocRef`

- `contextra-ids`: `/home/freddy/Projekte/contextra/crates/contextra-ids/src/lib.rs:649` -> `pub struct DocRef`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/erasure_domain.rs:28` -> `pub struct DocRef`
### `DurabilityConfigError`

- `contextra-store`: `/home/freddy/Projekte/contextra/crates/contextra-store/src/lsm/config.rs:25` -> `pub enum DurabilityConfigError`
- `contextra`: `/home/freddy/Projekte/contextra/crates/contextra/src/performance_profile.rs:53` -> `pub enum DurabilityConfigError`
### `DurabilityMode`

- `contextra-store`: `/home/freddy/Projekte/contextra/crates/contextra-store/src/lsm/config.rs:12` -> `pub enum DurabilityMode`
- `contextra`: `/home/freddy/Projekte/contextra/crates/contextra/src/performance_profile.rs:39` -> `pub enum DurabilityMode`
### `Edge`

- `contextra-graph`: `/home/freddy/Projekte/contextra/crates/contextra-graph/src/types.rs:139` -> `pub struct Edge`
- `contextra-types`: `/home/freddy/Projekte/contextra/crates/contextra-types/src/types/domain/misc.rs:192` -> `pub struct Edge`
### `EgressDecision`

- `contextra-privacy`: `/home/freddy/Projekte/contextra/crates/contextra-privacy/src/types.rs:140` -> `pub enum EgressDecision`
- `contextra-ports`: `/home/freddy/Projekte/contextra/crates/contextra-ports/src/policy.rs:17` -> `pub enum EgressDecision`
### `EgressPolicy`

- `contextra-privacy`: `/home/freddy/Projekte/contextra/crates/contextra-privacy/src/types.rs:150` -> `pub trait EgressPolicy`
- `contextra-ports`: `/home/freddy/Projekte/contextra/crates/contextra-ports/src/policy.rs:28` -> `pub trait EgressPolicy`
### `Entity`

- `contextra-graph`: `/home/freddy/Projekte/contextra/crates/contextra-graph/src/types.rs:119` -> `pub struct Entity`
- `contextra-types`: `/home/freddy/Projekte/contextra/crates/contextra-types/src/types/domain/misc.rs:129` -> `pub struct Entity`
### `EntityExtractor`

- `contextra-graph`: `/home/freddy/Projekte/contextra/crates/contextra-graph/src/entity_extraction.rs:67` -> `pub trait EntityExtractor`
- `contextra-db`: `/home/freddy/Projekte/contextra/crates/contextra-db/src/extraction/mod.rs:31` -> `pub trait EntityExtractor`
### `EntityId`

- `contextra-graph`: `/home/freddy/Projekte/contextra/crates/contextra-graph/src/types.rs:21` -> `pub struct EntityId`
- `contextra-ids`: `/home/freddy/Projekte/contextra/crates/contextra-ids/src/lib.rs:331` -> `pub struct EntityId`
### `Envelope`

- `contextra-ports`: `/home/freddy/Projekte/contextra/crates/contextra-ports/src/keys.rs:104` -> `pub struct Envelope`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:485` -> `pub struct Envelope`
### `Epoch`

- `contextra-text`: `/home/freddy/Projekte/contextra/crates/contextra-text/src/contract.rs:71` -> `pub struct Epoch`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/journal/epoch.rs:30` -> `pub struct Epoch`
### `EraseMode`

- `contextra-types`: `/home/freddy/Projekte/contextra/crates/contextra-types/src/erasure.rs:71` -> `pub enum EraseMode`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:1194` -> `pub enum EraseMode`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/erasure_domain.rs:87` -> `pub enum EraseMode`
### `EraseReceipt`

- `contextra-types`: `/home/freddy/Projekte/contextra/crates/contextra-types/src/erasure.rs:137` -> `pub struct EraseReceipt`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:1240` -> `pub struct EraseReceipt`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/erasure_domain.rs:153` -> `pub struct EraseReceipt`
### `EraseRequest`

- `contextra-types`: `/home/freddy/Projekte/contextra/crates/contextra-types/src/erasure.rs:120` -> `pub struct EraseRequest`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/erasure_domain.rs:136` -> `pub struct EraseRequest`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/saga/machine.rs:18` -> `pub struct EraseRequest`
### `EraseTarget`

- `contextra-ports`: `/home/freddy/Projekte/contextra/crates/contextra-ports/src/erasure.rs:44` -> `pub enum EraseTarget`
- `contextra-types`: `/home/freddy/Projekte/contextra/crates/contextra-types/src/erasure.rs:21` -> `pub enum EraseTarget`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:812` -> `pub enum EraseTarget`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/erasure_domain.rs:37` -> `pub enum EraseTarget`
### `ErasureEvent`

- `contextra-ports`: `/home/freddy/Projekte/contextra/crates/contextra-ports/src/observer.rs:49` -> `pub struct ErasureEvent`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:1151` -> `pub struct ErasureEvent`
### `ErasureId`

- `contextra-types`: `/home/freddy/Projekte/contextra/crates/contextra-types/src/ids.rs:16` -> `pub struct ErasureId`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:24` -> `pub struct ErasureId`
### `ErasureLayer`

- `contextra-ports`: `/home/freddy/Projekte/contextra/crates/contextra-ports/src/erasure.rs:206` -> `pub trait ErasureLayer`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:1045` -> `pub trait ErasureLayer`
### `ErasureObserver`

- `contextra-ports`: `/home/freddy/Projekte/contextra/crates/contextra-ports/src/observer.rs:79` -> `pub trait ErasureObserver`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:1177` -> `pub trait ErasureObserver`
### `ErasureStatus`

- `contextra-types`: `/home/freddy/Projekte/contextra/crates/contextra-types/src/erasure.rs:239` -> `pub enum ErasureStatus`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:1298` -> `pub enum ErasureStatus`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/erasure_domain.rs:255` -> `pub enum ErasureStatus`
### `ErasureTarget`

- `contextra-ports`: `/home/freddy/Projekte/contextra/crates/contextra-ports/src/erasure.rs:272` -> `pub trait ErasureTarget`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/backend.rs:26` -> `pub trait ErasureTarget`
### `Error`

- `contextra-text`: `/home/freddy/Projekte/contextra/crates/contextra-text/src/contract.rs:6` -> `pub enum Error`
- `contextra-simd`: `/home/freddy/Projekte/contextra/crates/contextra-simd/src/error.rs:41` -> `pub type_alias Error`
- `contextra-sys`: `/home/freddy/Projekte/contextra/crates/contextra-sys/src/error.rs:24` -> `pub enum Error`
- `contextra-privacy`: `/home/freddy/Projekte/contextra/crates/contextra-privacy/src/error.rs:66` -> `pub type_alias Error`
- `contextra-ids`: `/home/freddy/Projekte/contextra/crates/contextra-ids/src/lib.rs:131` -> `type_alias Error`
- `contextra-db`: `/home/freddy/Projekte/contextra/crates/contextra-db/src/filter.rs:190` -> `type_alias Error`
- `contextra-vector`: `/home/freddy/Projekte/contextra/crates/contextra-vector/src/contract.rs:25` -> `pub enum Error`
- `contextra-durable-fs`: `/home/freddy/Projekte/contextra/crates/contextra-durable-fs/src/error.rs:45` -> `pub type_alias Error`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/ed25519_proof.rs:155` -> `type_alias Error`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/purge/phase.rs:277` -> `type_alias Error`
- `contextra-rank`: `/home/freddy/Projekte/contextra/crates/contextra-rank/src/compat/types.rs:109` -> `type_alias Error`
### `FaultConfig`

- `contextra-graph`: `/home/freddy/Projekte/contextra/crates/contextra-graph/src/test_doubles.rs:34` -> `pub struct FaultConfig`
- `contextra-testkit`: `/home/freddy/Projekte/contextra/crates/contextra-testkit/src/fault_vfs.rs:39` -> `pub struct FaultConfig`
### `FaultVfs`

- `contextra-graph`: `/home/freddy/Projekte/contextra/crates/contextra-graph/src/test_doubles.rs:27` -> `pub struct FaultVfs`
- `contextra-testkit`: `/home/freddy/Projekte/contextra/crates/contextra-testkit/src/fault_vfs.rs:61` -> `pub struct FaultVfs`
### `FileId`

- `contextra-ids`: `/home/freddy/Projekte/contextra/crates/contextra-ids/src/lib.rs:498` -> `pub struct FileId`
- `contextra-ports`: `/home/freddy/Projekte/contextra/crates/contextra-ports/src/keys.rs:12` -> `pub struct FileId`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:395` -> `pub struct FileId`
### `FileKey`

- `contextra-ports`: `/home/freddy/Projekte/contextra/crates/contextra-ports/src/keys.rs:42` -> `pub struct FileKey`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:425` -> `pub struct FileKey`
### `FileKeyring`

- `contextra-ports`: `/home/freddy/Projekte/contextra/crates/contextra-ports/src/keys.rs:177` -> `pub trait FileKeyring`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:529` -> `pub trait FileKeyring`
### `FileKind`

- `contextra-ports`: `/home/freddy/Projekte/contextra/crates/contextra-ports/src/keys.rs:28` -> `pub enum FileKind`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:411` -> `pub enum FileKind`
### `FixedTestClock`

- `contextra-privacy`: `/home/freddy/Projekte/contextra/crates/contextra-privacy/src/context_edit_audit.rs:284` -> `struct FixedTestClock`
- `contextra-rank`: `/home/freddy/Projekte/contextra/crates/contextra-rank/src/calibration/isotonic.rs:392` -> `struct FixedTestClock`
### `FusionResult`

- `contextra-db`: `/home/freddy/Projekte/contextra/crates/contextra-db/src/temporal_filter.rs:22` -> `pub type_alias FusionResult`
- `contextra-rank`: `/home/freddy/Projekte/contextra/crates/contextra-rank/src/temporal.rs:23` -> `pub type_alias FusionResult`
### `FusionStrategy`

- `contextra-types`: `/home/freddy/Projekte/contextra/crates/contextra-types/src/types/saos.rs:31` -> `pub enum FusionStrategy`
- `contextra-rank`: `/home/freddy/Projekte/contextra/crates/contextra-rank/src/local_types.rs:102` -> `pub enum FusionStrategy`
### `FusionWeights`

- `contextra-types`: `/home/freddy/Projekte/contextra/crates/contextra-types/src/types/saos.rs:154` -> `pub struct FusionWeights`
- `contextra-rank`: `/home/freddy/Projekte/contextra/crates/contextra-rank/src/local_types.rs:162` -> `pub struct FusionWeights`
### `FuzzInput`

- `contextra-graph`: `/home/freddy/Projekte/contextra/crates/contextra-graph/fuzz/fuzz_targets/fuzz_graph_persist.rs:178` -> `struct FuzzInput`
- `contextra-store`: `/home/freddy/Projekte/contextra/crates/contextra-store/fuzz/fuzz_targets/fuzz_sstable_binary_search.rs:7` -> `struct FuzzInput`
- `contextra-store`: `/home/freddy/Projekte/contextra/crates/contextra-store/fuzz/fuzz_targets/wal_mutation_chaos.rs:42` -> `enum FuzzInput`
- `contextra-mcp`: `/home/freddy/Projekte/contextra/crates/contextra-mcp/fuzz/fuzz_targets/fuzz_prompt_injection_guard.rs:9` -> `struct FuzzInput`
### `GateDecision`

- `contextra-ports`: `/home/freddy/Projekte/contextra/crates/contextra-ports/src/gate.rs:104` -> `pub enum GateDecision`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:216` -> `pub enum GateDecision`
### `GateVariant`

- `contextra-types`: `/home/freddy/Projekte/contextra/crates/contextra-types/src/erasure.rs:288` -> `pub enum GateVariant`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/erasure_domain.rs:304` -> `pub enum GateVariant`
### `GraphMutationError`

- `contextra-graph`: `/home/freddy/Projekte/contextra/crates/contextra-graph/src/error.rs:72` -> `pub enum GraphMutationError`
- `contextra-ports`: `/home/freddy/Projekte/contextra/crates/contextra-ports/src/graph.rs:93` -> `pub enum GraphMutationError`
### `HeapItem`

- `contextra-graph`: `/home/freddy/Projekte/contextra/crates/contextra-graph/src/path_rag/k_path.rs:69` -> `struct HeapItem`
- `contextra-store`: `/home/freddy/Projekte/contextra/crates/contextra-store/src/compaction/engine.rs:550` -> `struct HeapItem`
### `Hit`

- `contextra-text`: `/home/freddy/Projekte/contextra/crates/contextra-text/src/memory_index.rs:233` -> `type_alias Hit`
- `contextra-text`: `/home/freddy/Projekte/contextra/crates/contextra-text/src/memory_index.rs:388` -> `type_alias Hit`
- `contextra-text`: `/home/freddy/Projekte/contextra/crates/contextra-text/src/traits.rs:8` -> `type_alias Hit`
- `contextra-vector`: `/home/freddy/Projekte/contextra/crates/contextra-vector/src/contract.rs:122` -> `type_alias Hit`
- `contextra-vector`: `/home/freddy/Projekte/contextra/crates/contextra-vector/src/compat/hnsw_vector_index.rs:570` -> `type_alias Hit`
- `contextra-vector`: `/home/freddy/Projekte/contextra/crates/contextra-vector/src/compat/hnsw_vector_index.rs:604` -> `type_alias Hit`
### `HybridQueryBuilder`

- `contextra-db`: `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/query_builder/builder.rs:31` -> `pub struct HybridQueryBuilder`
- `contextra-types`: `/home/freddy/Projekte/contextra/crates/contextra-types/src/types/saos.rs:493` -> `pub struct HybridQueryBuilder`
### `HyperEdgeId`

- `contextra-graph`: `/home/freddy/Projekte/contextra/crates/contextra-graph/src/types.rs:182` -> `pub struct HyperEdgeId`
- `contextra-ports`: `/home/freddy/Projekte/contextra/crates/contextra-ports/src/graph.rs:63` -> `pub struct HyperEdgeId`
### `IdGen`

- `contextra-env`: `/home/freddy/Projekte/contextra/crates/contextra-env/src/id_gen.rs:14` -> `pub trait IdGen`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:2156` -> `pub trait IdGen`
### `InMemoryStorageEngine`

- `contextra-graph`: `/home/freddy/Projekte/contextra/crates/contextra-graph/src/test_doubles.rs:96` -> `pub struct InMemoryStorageEngine`
- `contextra-store`: `/home/freddy/Projekte/contextra/crates/contextra-store/src/compat/engine/memory_only.rs:32` -> `pub struct InMemoryStorageEngine`
- `contextra-testkit`: `/home/freddy/Projekte/contextra/crates/contextra-testkit/src/in_memory_store.rs:64` -> `pub struct InMemoryStorageEngine`
### `Index`

- `contextra-text`: `/home/freddy/Projekte/contextra/crates/contextra-text/src/traits.rs:6` -> `pub trait Index`
- `contextra-vector`: `/home/freddy/Projekte/contextra/crates/contextra-vector/src/contract.rs:120` -> `pub trait Index`
- `contextra-vector`: `/home/freddy/Projekte/contextra/crates/contextra-vector/src/compat/hnsw_vector_index.rs:568` -> `pub trait Index`
### `JournalRecord`

- `contextra-durable-fs`: `/home/freddy/Projekte/contextra/crates/contextra-durable-fs/src/journal.rs:62` -> `pub struct JournalRecord`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/journal/record.rs:124` -> `pub struct JournalRecord`
### `KeyBound`

- `contextra-ports`: `/home/freddy/Projekte/contextra/crates/contextra-ports/src/gate.rs:49` -> `pub enum KeyBound`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:161` -> `pub enum KeyBound`
### `KeyBuf`

- `contextra-ports`: `/home/freddy/Projekte/contextra/crates/contextra-ports/src/gate.rs:26` -> `pub struct KeyBuf`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:139` -> `pub struct KeyBuf`
### `KeyCoverage`

- `contextra-ports`: `/home/freddy/Projekte/contextra/crates/contextra-ports/src/erasure.rs:189` -> `pub enum KeyCoverage`
- `contextra-types`: `/home/freddy/Projekte/contextra/crates/contextra-types/src/erasure.rs:273` -> `pub enum KeyCoverage`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:709` -> `pub enum KeyCoverage`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/erasure_domain.rs:289` -> `pub enum KeyCoverage`
### `KeyMaterialSource`

- `contextra-store`: `/home/freddy/Projekte/contextra/crates/contextra-store/src/key_source.rs:7` -> `pub trait KeyMaterialSource`
- `contextra-ports`: `/home/freddy/Projekte/contextra/crates/contextra-ports/src/key_material.rs:31` -> `pub trait KeyMaterialSource`
### `KeyRange`

- `contextra-ports`: `/home/freddy/Projekte/contextra/crates/contextra-ports/src/gate.rs:77` -> `pub struct KeyRange`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:189` -> `pub struct KeyRange`
### `KvDeleteMode`

- `contextra-crypto`: `/home/freddy/Projekte/contextra/crates/contextra-crypto/src/kv_segment/delete_mode.rs:22` -> `pub enum KvDeleteMode`
- `contextra`: `/home/freddy/Projekte/contextra/crates/contextra/src/collection_profile.rs:40` -> `pub enum KvDeleteMode`
### `LayerId`

- `contextra-ports`: `/home/freddy/Projekte/contextra/crates/contextra-ports/src/erasure.rs:13` -> `pub struct LayerId`
- `contextra-types`: `/home/freddy/Projekte/contextra/crates/contextra-types/src/erasure.rs:152` -> `pub struct LayerId`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:679` -> `pub struct LayerId`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/erasure_domain.rs:168` -> `pub struct LayerId`
### `LayerScanner`

- `contextra-ports`: `/home/freddy/Projekte/contextra/crates/contextra-ports/src/scanner/mod.rs:60` -> `pub trait LayerScanner`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:2044` -> `pub trait LayerScanner`
### `LayerScope`

- `contextra-ports`: `/home/freddy/Projekte/contextra/crates/contextra-ports/src/erasure.rs:70` -> `pub struct LayerScope`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:770` -> `pub struct LayerScope`
### `LayerWitness`

- `contextra-ports`: `/home/freddy/Projekte/contextra/crates/contextra-ports/src/scanner/witness.rs:17` -> `pub struct LayerWitness`
- `contextra-types`: `/home/freddy/Projekte/contextra/crates/contextra-types/src/erasure.rs:559` -> `pub struct LayerWitness`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:955` -> `pub struct LayerWitness`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/erasure_domain.rs:575` -> `pub struct LayerWitness`
### `LedgerEntry`

- `contextra-ports`: `/home/freddy/Projekte/contextra/crates/contextra-ports/src/ledger.rs:59` -> `pub struct LedgerEntry`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:573` -> `pub struct LedgerEntry`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/ledger/entry.rs:10` -> `pub struct LedgerEntry`
### `LedgerSink`

- `contextra-ports`: `/home/freddy/Projekte/contextra/crates/contextra-ports/src/ledger.rs:152` -> `pub trait LedgerSink`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:664` -> `pub trait LedgerSink`
### `LoomMetadata`

- `contextra-checkpoint`: `/home/freddy/Projekte/contextra/crates/contextra-checkpoint/src/hardlink_cloner.rs:113` -> `pub struct LoomMetadata`
- `contextra-store`: `/home/freddy/Projekte/contextra/crates/contextra-store/src/wal/fs.rs:59` -> `pub struct LoomMetadata`
### `Maintain`

- `contextra-text`: `/home/freddy/Projekte/contextra/crates/contextra-text/src/traits.rs:23` -> `pub trait Maintain`
- `contextra-vector`: `/home/freddy/Projekte/contextra/crates/contextra-vector/src/contract.rs:136` -> `pub trait Maintain`
- `contextra-vector`: `/home/freddy/Projekte/contextra/crates/contextra-vector/src/types.rs:86` -> `pub trait Maintain`
- `contextra-vector`: `/home/freddy/Projekte/contextra/crates/contextra-vector/src/compat/hnsw_vector_index.rs:583` -> `pub trait Maintain`
### `ManifestEntry`

- `contextra-store`: `/home/freddy/Projekte/contextra/crates/contextra-store/src/manifest/entry.rs:21` -> `pub enum ManifestEntry`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/vault/sealer.rs:542` -> `pub struct ManifestEntry`
### `ManualClock`

- `contextra-env`: `/home/freddy/Projekte/contextra/crates/contextra-env/src/manual_clock.rs:17` -> `pub struct ManualClock`
- `contextra-testkit`: `/home/freddy/Projekte/contextra/crates/contextra-testkit/src/manual_clock.rs:41` -> `pub struct ManualClock`
### `MeasureClass`

- `contextra-types`: `/home/freddy/Projekte/contextra/crates/contextra-types/src/erasure.rs:301` -> `pub enum MeasureClass`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:720` -> `pub enum MeasureClass`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/erasure_domain.rs:317` -> `pub enum MeasureClass`
### `MemVfs`

- `contextra-ports`: `/home/freddy/Projekte/contextra/crates/contextra-ports/src/contract_tests.rs:371` -> `pub struct MemVfs`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/ledger/persistent.rs:138` -> `struct MemVfs`
### `Meta`

- `contextra-checkpoint`: `/home/freddy/Projekte/contextra/crates/contextra-checkpoint/src/compat/ports_impl.rs:14` -> `type_alias Meta`
- `contextra-ports`: `/home/freddy/Projekte/contextra/crates/contextra-ports/src/checkpoint.rs:28` -> `type_alias Meta`
### `MockEmbedder`

- `contextra-mcp`: `/home/freddy/Projekte/contextra/crates/contextra-mcp/src/explain.rs:131` -> `struct MockEmbedder`
- `contextra-mcp`: `/home/freddy/Projekte/contextra/crates/contextra-mcp/src/tests.rs:17` -> `struct MockEmbedder`
- `contextra-mcp`: `/home/freddy/Projekte/contextra/crates/contextra-mcp/src/config.rs:439` -> `pub struct MockEmbedder`
- `contextra-testkit`: `/home/freddy/Projekte/contextra/crates/contextra-testkit/src/mocks.rs:35` -> `pub struct MockEmbedder`
### `MockLlmGenerator`

- `contextra-db`: `/home/freddy/Projekte/contextra/crates/contextra-db/src/extraction/tests.rs:20` -> `struct MockLlmGenerator`
- `contextra-mcp`: `/home/freddy/Projekte/contextra/crates/contextra-mcp/src/config.rs:499` -> `pub struct MockLlmGenerator`
### `MockStorage`

- `contextra-text`: `/home/freddy/Projekte/contextra/crates/contextra-text/src/wand.rs:594` -> `struct MockStorage`
- `contextra-checkpoint`: `/home/freddy/Projekte/contextra/crates/contextra-checkpoint/src/guard.rs:517` -> `struct MockStorage`
- `contextra-checkpoint`: `/home/freddy/Projekte/contextra/crates/contextra-checkpoint/src/store/tests.rs:22` -> `struct MockStorage`
- `contextra-checkpoint`: `/home/freddy/Projekte/contextra/crates/contextra-checkpoint/src/guard/tests.rs:10` -> `struct MockStorage`
- `contextra-db`: `/home/freddy/Projekte/contextra/crates/contextra-db/src/orchestration/mod.rs:316` -> `struct MockStorage`
### `ModelFingerprint`

- `contextra-crypto`: `/home/freddy/Projekte/contextra/crates/contextra-crypto/src/kv_cipher.rs:43` -> `pub struct ModelFingerprint`
- `contextra-types`: `/home/freddy/Projekte/contextra/crates/contextra-types/src/model_fingerprint.rs:33` -> `pub struct ModelFingerprint`
### `ObservedPhase`

- `contextra-ports`: `/home/freddy/Projekte/contextra/crates/contextra-ports/src/observer.rs:36` -> `pub enum ObservedPhase`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:1138` -> `pub enum ObservedPhase`
### `ObserverError`

- `contextra-ports`: `/home/freddy/Projekte/contextra/crates/contextra-ports/src/observer.rs:62` -> `pub enum ObserverError`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:1164` -> `pub enum ObserverError`
### `PendingReason`

- `contextra-types`: `/home/freddy/Projekte/contextra/crates/contextra-types/src/erasure.rs:182` -> `pub enum PendingReason`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:1251` -> `pub enum PendingReason`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/erasure_domain.rs:198` -> `pub enum PendingReason`
### `Phase`

- `contextra-types`: `/home/freddy/Projekte/contextra/crates/contextra-types/src/erasure.rs:214` -> `pub enum Phase`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:1277` -> `pub enum Phase`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/erasure_domain.rs:230` -> `pub enum Phase`
### `PprAlgorithm`

- `contextra-graph`: `/home/freddy/Projekte/contextra/crates/contextra-graph/src/types.rs:270` -> `pub enum PprAlgorithm`
- `contextra-types`: `/home/freddy/Projekte/contextra/crates/contextra-types/src/types/domain/misc.rs:455` -> `pub enum PprAlgorithm`
### `PprConfig`

- `contextra-graph`: `/home/freddy/Projekte/contextra/crates/contextra-graph/src/types.rs:281` -> `pub struct PprConfig`
- `contextra-types`: `/home/freddy/Projekte/contextra/crates/contextra-types/src/types/domain/misc.rs:484` -> `pub struct PprConfig`
### `PurgeBarrier`

- `contextra-ports`: `/home/freddy/Projekte/contextra/crates/contextra-ports/src/barrier.rs:112` -> `pub trait PurgeBarrier`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:359` -> `pub trait PurgeBarrier`
### `PurgeCx`

- `contextra-ports`: `/home/freddy/Projekte/contextra/crates/contextra-ports/src/erasure.rs:118` -> `pub struct PurgeCx`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:854` -> `pub struct PurgeCx`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/purge/cx.rs:10` -> `pub struct PurgeCx`
### `PurgeOutcome`

- `contextra-ports`: `/home/freddy/Projekte/contextra/crates/contextra-ports/src/erasure.rs:132` -> `pub struct PurgeOutcome`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:835` -> `pub struct PurgeOutcome`
### `PurgePermit`

- `contextra-ports`: `/home/freddy/Projekte/contextra/crates/contextra-ports/src/barrier.rs:73` -> `pub struct PurgePermit`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:333` -> `pub struct PurgePermit`
### `PurgeStats`

- `contextra-ports`: `/home/freddy/Projekte/contextra/crates/contextra-ports/src/scanner/layer.rs:107` -> `pub struct PurgeStats`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:1833` -> `pub struct PurgeStats`
### `Purger`

- `contextra-ports`: `/home/freddy/Projekte/contextra/crates/contextra-ports/src/scanner/layer.rs:115` -> `pub trait Purger`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:1864` -> `pub trait Purger`
### `RangeGate`

- `contextra-ports`: `/home/freddy/Projekte/contextra/crates/contextra-ports/src/gate.rs:115` -> `pub enum RangeGate`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:227` -> `pub enum RangeGate`
### `RangeSet`

- `contextra-ports`: `/home/freddy/Projekte/contextra/crates/contextra-ports/src/ledger.rs:27` -> `pub struct RangeSet`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:544` -> `pub struct RangeSet`
### `ReadGate`

- `contextra-ports`: `/home/freddy/Projekte/contextra/crates/contextra-ports/src/gate.rs:128` -> `pub trait ReadGate`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:237` -> `pub trait ReadGate`
### `RealFs`

- `contextra-store`: `/home/freddy/Projekte/contextra/crates/contextra-store/src/io_seam.rs:66` -> `pub struct RealFs`
- `contextra-store`: `/home/freddy/Projekte/contextra/crates/contextra-store/src/io_seam/real_fs.rs:14` -> `pub struct RealFs`
- `contextra-durable-fs`: `/home/freddy/Projekte/contextra/crates/contextra-durable-fs/src/vfs.rs:60` -> `pub type_alias RealFs`
### `RecordSealer`

- `contextra-store`: `/home/freddy/Projekte/contextra/crates/contextra-store/src/util.rs:47` -> `pub trait RecordSealer`
- `contextra-ports`: `/home/freddy/Projekte/contextra/crates/contextra-ports/src/keys.rs:128` -> `pub trait RecordSealer`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:508` -> `pub trait RecordSealer`
### `RefusalReason`

- `contextra-types`: `/home/freddy/Projekte/contextra/crates/contextra-types/src/erasure.rs:201` -> `pub enum RefusalReason`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:1268` -> `pub enum RefusalReason`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/erasure_domain.rs:217` -> `pub enum RefusalReason`
### `ReportSignature`

- `contextra-ports`: `/home/freddy/Projekte/contextra/crates/contextra-ports/src/report_signer.rs:19` -> `pub struct ReportSignature`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:1073` -> `pub struct ReportSignature`
### `ReportSigner`

- `contextra-ports`: `/home/freddy/Projekte/contextra/crates/contextra-ports/src/report_signer.rs:45` -> `pub trait ReportSigner`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:1097` -> `pub trait ReportSigner`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/report/signing.rs:322` -> `pub struct ReportSigner`
### `RequestContext`

- `contextra-types`: `/home/freddy/Projekte/contextra/crates/contextra-types/src/witness.rs:57` -> `pub struct RequestContext`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:1698` -> `pub struct RequestContext`
### `ResidualRisk`

- `contextra-types`: `/home/freddy/Projekte/contextra/crates/contextra-types/src/erasure.rs:357` -> `pub enum ResidualRisk`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/erasure_domain.rs:373` -> `pub enum ResidualRisk`
### `Residue`

- `contextra-types`: `/home/freddy/Projekte/contextra/crates/contextra-types/src/erasure.rs:462` -> `pub struct Residue`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:751` -> `pub struct Residue`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/erasure_domain.rs:478` -> `pub struct Residue`
### `ResourceBudget`

- `contextra-graph`: `/home/freddy/Projekte/contextra/crates/contextra-graph/src/types.rs:306` -> `pub struct ResourceBudget`
- `contextra-store`: `/home/freddy/Projekte/contextra/crates/contextra-store/src/budget.rs:8` -> `pub struct ResourceBudget`
- `contextra-types`: `/home/freddy/Projekte/contextra/crates/contextra-types/src/types/budget.rs:341` -> `pub struct ResourceBudget`
### `ResourceTracker`

- `contextra-graph`: `/home/freddy/Projekte/contextra/crates/contextra-graph/src/types.rs:317` -> `pub struct ResourceTracker`
- `contextra-store`: `/home/freddy/Projekte/contextra/crates/contextra-store/src/budget.rs:15` -> `pub struct ResourceTracker`
- `contextra-types`: `/home/freddy/Projekte/contextra/crates/contextra-types/src/types/budget.rs:357` -> `pub struct ResourceTracker`
### `Result`

- `contextra-text`: `/home/freddy/Projekte/contextra/crates/contextra-text/src/contract.rs:30` -> `pub type_alias Result`
- `contextra-text`: `/home/freddy/Projekte/contextra/crates/contextra-text/src/error.rs:44` -> `pub type_alias Result`
- `contextra-crypto`: `/home/freddy/Projekte/contextra/crates/contextra-crypto/src/error.rs:17` -> `pub type_alias Result`
- `contextra-simd`: `/home/freddy/Projekte/contextra/crates/contextra-simd/src/error.rs:38` -> `pub type_alias Result`
- `contextra-graph`: `/home/freddy/Projekte/contextra/crates/contextra-graph/src/error.rs:16` -> `pub type_alias Result`
- `contextra-sys`: `/home/freddy/Projekte/contextra/crates/contextra-sys/src/error.rs:68` -> `pub type_alias Result`
- `contextra-checkpoint`: `/home/freddy/Projekte/contextra/crates/contextra-checkpoint/src/error.rs:12` -> `pub type_alias Result`
- `contextra-privacy`: `/home/freddy/Projekte/contextra/crates/contextra-privacy/src/error.rs:69` -> `pub type_alias Result`
- `contextra-ids`: `/home/freddy/Projekte/contextra/crates/contextra-ids/src/lib.rs:72` -> `pub type_alias Result`
- `contextra-mvcc`: `/home/freddy/Projekte/contextra/crates/contextra-mvcc/src/mvcc_error.rs:63` -> `pub type_alias Result`
- `contextra-mcp`: `/home/freddy/Projekte/contextra/crates/contextra-mcp/src/sandbox.rs:16` -> `type_alias Result`
- `contextra-vector`: `/home/freddy/Projekte/contextra/crates/contextra-vector/src/contract.rs:53` -> `pub type_alias Result`
- `contextra-vector`: `/home/freddy/Projekte/contextra/crates/contextra-vector/src/error.rs:87` -> `pub type_alias Result`
- `contextra-durable-fs`: `/home/freddy/Projekte/contextra/crates/contextra-durable-fs/src/error.rs:73` -> `pub type_alias Result`
- `contextra-types`: `/home/freddy/Projekte/contextra/crates/contextra-types/src/error.rs:115` -> `pub type_alias Result`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/revocation_log.rs:35` -> `pub type_alias Result`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/deletion_proof.rs:34` -> `pub type_alias Result`
- `contextra-rank`: `/home/freddy/Projekte/contextra/crates/contextra-rank/src/error.rs:23` -> `pub type_alias Result`
### `RetentionAttestation`

- `contextra-types`: `/home/freddy/Projekte/contextra/crates/contextra-types/src/erasure.rs:105` -> `pub struct RetentionAttestation`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:1229` -> `pub struct RetentionAttestation`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/erasure_domain.rs:121` -> `pub struct RetentionAttestation`
### `RetentionCode`

- `contextra-types`: `/home/freddy/Projekte/contextra/crates/contextra-types/src/erasure.rs:84` -> `pub enum RetentionCode`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:1212` -> `pub enum RetentionCode`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/erasure_domain.rs:100` -> `pub enum RetentionCode`
### `RiskCondition`

- `contextra-types`: `/home/freddy/Projekte/contextra/crates/contextra-types/src/erasure.rs:342` -> `pub enum RiskCondition`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/erasure_domain.rs:358` -> `pub enum RiskCondition`
### `Rng`

- `contextra-text`: `/home/freddy/Projekte/contextra/crates/contextra-text/src/contract.rs:46` -> `pub trait Rng`
- `contextra-env`: `/home/freddy/Projekte/contextra/crates/contextra-env/src/rng.rs:32` -> `pub trait Rng`
- `contextra-vector`: `/home/freddy/Projekte/contextra/crates/contextra-vector/src/contract.rs:67` -> `pub trait Rng`
### `RoleBinding`

- `contextra-graph`: `/home/freddy/Projekte/contextra/crates/contextra-graph/src/types.rs:221` -> `pub struct RoleBinding`
- `contextra-ports`: `/home/freddy/Projekte/contextra/crates/contextra-ports/src/graph.rs:44` -> `pub struct RoleBinding`
### `RoleId`

- `contextra-graph`: `/home/freddy/Projekte/contextra/crates/contextra-graph/src/types.rs:202` -> `pub struct RoleId`
- `contextra-ports`: `/home/freddy/Projekte/contextra/crates/contextra-ports/src/graph.rs:14` -> `pub struct RoleId`
### `SagaRecord`

- `contextra-types`: `/home/freddy/Projekte/contextra/crates/contextra-types/src/witness.rs:84` -> `pub struct SagaRecord`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:1714` -> `pub struct SagaRecord`
### `ScanMethod`

- `contextra-types`: `/home/freddy/Projekte/contextra/crates/contextra-types/src/witness.rs:44` -> `pub enum ScanMethod`
- `contextra-types`: `/home/freddy/Projekte/contextra/crates/contextra-types/src/erasure.rs:316` -> `pub enum ScanMethod`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:1803` -> `pub enum ScanMethod`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/erasure_domain.rs:332` -> `pub enum ScanMethod`
### `ScanSession`

- `contextra-ports`: `/home/freddy/Projekte/contextra/crates/contextra-ports/src/scanner/session.rs:15` -> `pub struct ScanSession`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:1875` -> `pub struct ScanSession`
### `ScanTarget`

- `contextra-ports`: `/home/freddy/Projekte/contextra/crates/contextra-ports/src/scanner/layer.rs:88` -> `pub struct ScanTarget`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:1814` -> `pub struct ScanTarget`
### `ScanTechnique`

- `contextra-ports`: `/home/freddy/Projekte/contextra/crates/contextra-ports/src/scanner/layer.rs:71` -> `pub enum ScanTechnique`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:1786` -> `pub enum ScanTechnique`
### `Scanner`

- `contextra-ports`: `/home/freddy/Projekte/contextra/crates/contextra-ports/src/erasure.rs:156` -> `pub struct Scanner`
- `contextra-types`: `/home/freddy/Projekte/contextra/crates/contextra-types/src/erasure.rs:485` -> `pub struct Scanner`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:867` -> `pub struct Scanner`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/erasure_domain.rs:501` -> `pub struct Scanner`
### `ScoredDocument`

- `contextra-vector`: `/home/freddy/Projekte/contextra/crates/contextra-vector/src/types.rs:40` -> `pub struct ScoredDocument`
- `contextra-types`: `/home/freddy/Projekte/contextra/crates/contextra-types/src/types/domain/document.rs:221` -> `pub struct ScoredDocument`
### `ScrubOutcome`

- `contextra-durable-fs`: `/home/freddy/Projekte/contextra/crates/contextra-durable-fs/src/scrub.rs:54` -> `pub enum ScrubOutcome`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/vault/scrub.rs:27` -> `pub struct ScrubOutcome`
### `SealCtx`

- `contextra-ports`: `/home/freddy/Projekte/contextra/crates/contextra-ports/src/keys.rs:84` -> `pub struct SealCtx`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:465` -> `pub struct SealCtx`
### `SecretKey`

- `contextra-crypto`: `/home/freddy/Projekte/contextra/crates/contextra-crypto/src/secret.rs:14` -> `pub struct SecretKey`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/crypto_local.rs:15` -> `pub struct SecretKey`
### `Seq`

- `contextra-ids`: `/home/freddy/Projekte/contextra/crates/contextra-ids/src/lib.rs:552` -> `pub struct Seq`
- `contextra-ports`: `/home/freddy/Projekte/contextra/crates/contextra-ports/src/gate.rs:10` -> `pub struct Seq`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:65` -> `pub struct Seq`
### `SeqBitTombstone`

- `contextra-mvcc`: `/home/freddy/Projekte/contextra/crates/contextra-mvcc/src/tombstone.rs:38` -> `pub struct SeqBitTombstone`
- `contextra-types`: `/home/freddy/Projekte/contextra/crates/contextra-types/src/tombstone.rs:53` -> `pub struct SeqBitTombstone`
### `SequentialIdGen`

- `contextra-env`: `/home/freddy/Projekte/contextra/crates/contextra-env/src/id_gen.rs:39` -> `pub struct SequentialIdGen`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:2163` -> `pub struct SequentialIdGen`
### `Signal`

- `contextra-types`: `/home/freddy/Projekte/contextra/crates/contextra-types/src/types/saos.rs:321` -> `pub enum Signal`
- `contextra-rank`: `/home/freddy/Projekte/contextra/crates/contextra-rank/src/fusion/types.rs:187` -> `pub enum Signal`
### `SignalFusionStrategies`

- `contextra-types`: `/home/freddy/Projekte/contextra/crates/contextra-types/src/types/saos.rs:44` -> `pub struct SignalFusionStrategies`
- `contextra-rank`: `/home/freddy/Projekte/contextra/crates/contextra-rank/src/local_types.rs:127` -> `pub struct SignalFusionStrategies`
### `SignerError`

- `contextra-ports`: `/home/freddy/Projekte/contextra/crates/contextra-ports/src/report_signer.rs:30` -> `pub enum SignerError`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:1084` -> `pub enum SignerError`
### `SignerId`

- `contextra-ports`: `/home/freddy/Projekte/contextra/crates/contextra-ports/src/report_signer.rs:15` -> `pub type_alias SignerId`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:1069` -> `pub type_alias SignerId`
### `StagedOp`

- `contextra-store`: `/home/freddy/Projekte/contextra/crates/contextra-store/src/compat/engine/memory_only.rs:15` -> `enum StagedOp`
- `contextra-testkit`: `/home/freddy/Projekte/contextra/crates/contextra-testkit/src/in_memory_store.rs:39` -> `enum StagedOp`
### `StagedTxMap`

- `contextra-store`: `/home/freddy/Projekte/contextra/crates/contextra-store/src/compat/engine/memory_only.rs:28` -> `type_alias StagedTxMap`
- `contextra-testkit`: `/home/freddy/Projekte/contextra/crates/contextra-testkit/src/in_memory_store.rs:49` -> `type_alias StagedTxMap`
### `Stats`

- `contextra-text`: `/home/freddy/Projekte/contextra/crates/contextra-text/src/memory_index.rs:404` -> `type_alias Stats`
- `contextra-text`: `/home/freddy/Projekte/contextra/crates/contextra-text/src/traits.rs:25` -> `type_alias Stats`
- `contextra-vector`: `/home/freddy/Projekte/contextra/crates/contextra-vector/src/contract.rs:138` -> `type_alias Stats`
- `contextra-vector`: `/home/freddy/Projekte/contextra/crates/contextra-vector/src/compat/hnsw_vector_index.rs:585` -> `type_alias Stats`
- `contextra-vector`: `/home/freddy/Projekte/contextra/crates/contextra-vector/src/compat/hnsw_vector_index.rs:809` -> `type_alias Stats`
### `StorageBackend`

- `contextra-ports`: `/home/freddy/Projekte/contextra/crates/contextra-ports/src/erasure.rs:269` -> `pub trait StorageBackend`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/backend.rs:17` -> `pub trait StorageBackend`
### `StorageStats`

- `contextra-store`: `/home/freddy/Projekte/contextra/crates/contextra-store/src/stats.rs:5` -> `pub struct StorageStats`
- `contextra-ports`: `/home/freddy/Projekte/contextra/crates/contextra-ports/src/storage.rs:26` -> `pub struct StorageStats`
### `StoreDataMap`

- `contextra-store`: `/home/freddy/Projekte/contextra/crates/contextra-store/src/compat/engine/memory_only.rs:27` -> `type_alias StoreDataMap`
- `contextra-testkit`: `/home/freddy/Projekte/contextra/crates/contextra-testkit/src/in_memory_store.rs:47` -> `type_alias StoreDataMap`
### `SystemClock`

- `contextra-env`: `/home/freddy/Projekte/contextra/crates/contextra-env/src/clock.rs:54` -> `pub struct SystemClock`
- `contextra-privacy`: `/home/freddy/Projekte/contextra/crates/contextra-privacy/src/types.rs:20` -> `pub struct SystemClock`
- `contextra-mcp`: `/home/freddy/Projekte/contextra/crates/contextra-mcp/src/prompt_injection.rs:25` -> `pub struct SystemClock`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:2104` -> `pub struct SystemClock`
- `contextra-rank`: `/home/freddy/Projekte/contextra/crates/contextra-rank/src/calibration/isotonic.rs:26` -> `pub struct SystemClock`
### `Target`

- `contextra-graph`: `/home/freddy/Projekte/contextra/crates/contextra-graph/src/hyperedge.rs:266` -> `type_alias Target`
- `contextra-graph`: `/home/freddy/Projekte/contextra/crates/contextra-graph/src/arc_slice.rs:124` -> `type_alias Target`
- `contextra-graph`: `/home/freddy/Projekte/contextra/crates/contextra-graph/src/csr/inner/mod.rs:167` -> `type_alias Target`
- `contextra-sys`: `/home/freddy/Projekte/contextra/crates/contextra-sys/src/mmap.rs:171` -> `type_alias Target`
- `contextra-sys`: `/home/freddy/Projekte/contextra/crates/contextra-sys/src/vault.rs:193` -> `type_alias Target`
### `TargetKind`

- `contextra-ports`: `/home/freddy/Projekte/contextra/crates/contextra-ports/src/observer.rs:19` -> `pub enum TargetKind`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:1121` -> `pub enum TargetKind`
### `TenantId`

- `contextra-privacy`: `/home/freddy/Projekte/contextra/crates/contextra-privacy/src/types.rs:48` -> `pub struct TenantId`
- `contextra-ids`: `/home/freddy/Projekte/contextra/crates/contextra-ids/src/lib.rs:79` -> `pub struct TenantId`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:1591` -> `pub struct TenantId`
### `TenantScopeViolation`

- `contextra-privacy`: `/home/freddy/Projekte/contextra/crates/contextra-privacy/src/types.rs:88` -> `pub enum TenantScopeViolation`
- `contextra-ids`: `/home/freddy/Projekte/contextra/crates/contextra-ids/src/lib.rs:660` -> `pub enum TenantScopeViolation`
### `TenantScoped`

- `contextra-privacy`: `/home/freddy/Projekte/contextra/crates/contextra-privacy/src/types.rs:104` -> `pub struct TenantScoped`
- `contextra-ids`: `/home/freddy/Projekte/contextra/crates/contextra-ids/src/lib.rs:674` -> `pub struct TenantScoped`
### `TextGenerator`

- `contextra-graph`: `/home/freddy/Projekte/contextra/crates/contextra-graph/src/text_gen.rs:6` -> `pub trait TextGenerator`
- `contextra-ports`: `/home/freddy/Projekte/contextra/crates/contextra-ports/src/embedding.rs:137` -> `pub trait TextGenerator`
### `Timestamp`

- `contextra-ids`: `/home/freddy/Projekte/contextra/crates/contextra-ids/src/lib.rs:578` -> `pub struct Timestamp`
- `contextra-ports`: `/home/freddy/Projekte/contextra/crates/contextra-ports/src/observer.rs:15` -> `pub type_alias Timestamp`
- `contextra-ports`: `/home/freddy/Projekte/contextra/crates/contextra-ports/src/ledger.rs:11` -> `pub struct Timestamp`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:81` -> `pub struct Timestamp`
### `Timing`

- `contextra-types`: `/home/freddy/Projekte/contextra/crates/contextra-types/src/erasure.rs:58` -> `pub enum Timing`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:1203` -> `pub enum Timing`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/erasure_domain.rs:74` -> `pub enum Timing`
### `TlHfdParams`

- `contextra-graph`: `/home/freddy/Projekte/contextra/crates/contextra-graph/src/types.rs:232` -> `pub struct TlHfdParams`
- `contextra-graph`: `/home/freddy/Projekte/contextra/crates/contextra-graph/src/tl_hfd/params.rs:20` -> `pub struct TlHfdParams`
- `contextra-types`: `/home/freddy/Projekte/contextra/crates/contextra-types/src/types/domain/misc.rs:389` -> `pub struct TlHfdParams`
### `TombstoneSemanticsCheck`

- `contextra-mvcc`: `/home/freddy/Projekte/contextra/crates/contextra-mvcc/src/tombstone.rs:23` -> `pub trait TombstoneSemanticsCheck`
- `contextra-types`: `/home/freddy/Projekte/contextra/crates/contextra-types/src/tombstone.rs:38` -> `pub trait TombstoneSemanticsCheck`
### `TxId`

- `contextra-graph`: `/home/freddy/Projekte/contextra/crates/contextra-graph/src/types.rs:91` -> `pub struct TxId`
- `contextra-ids`: `/home/freddy/Projekte/contextra/crates/contextra-ids/src/lib.rs:407` -> `pub struct TxId`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:1676` -> `pub struct TxId`
- `contextra-rank`: `/home/freddy/Projekte/contextra/crates/contextra-rank/src/local_types.rs:187` -> `pub struct TxId`
### `Unarmed`

- `contextra-types`: `/home/freddy/Projekte/contextra/crates/contextra-types/src/erasure.rs:431` -> `pub struct Unarmed`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:731` -> `pub struct Unarmed`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/erasure_domain.rs:447` -> `pub struct Unarmed`
### `UnitKey`

- `contextra-crypto`: `/home/freddy/Projekte/contextra/crates/contextra-crypto/src/dek.rs:20` -> `pub struct UnitKey`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/crypto_local.rs:43` -> `pub struct UnitKey`
### `ValidityWindow`

- `contextra-db`: `/home/freddy/Projekte/contextra/crates/contextra-db/src/temporal_filter.rs:26` -> `pub struct ValidityWindow`
- `contextra-rank`: `/home/freddy/Projekte/contextra/crates/contextra-rank/src/temporal.rs:27` -> `pub struct ValidityWindow`
### `VectorIndex`

- `contextra-ports`: `/home/freddy/Projekte/contextra/crates/contextra-ports/src/vector_index.rs:47` -> `pub trait VectorIndex`
- `contextra-vector`: `/home/freddy/Projekte/contextra/crates/contextra-vector/src/compat/candidate_stream.rs:11` -> `pub trait VectorIndex`
### `VerifiedVia`

- `contextra-store`: `/home/freddy/Projekte/contextra/crates/contextra-store/src/io_seam.rs:9` -> `pub enum VerifiedVia`
- `contextra-store`: `/home/freddy/Projekte/contextra/crates/contextra-store/src/io_seam/types.rs:12` -> `pub enum VerifiedVia`
- `contextra-ports`: `/home/freddy/Projekte/contextra/crates/contextra-ports/src/vfs.rs:26` -> `pub enum VerifiedVia`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:1332` -> `pub enum VerifiedVia`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/vault/table.rs:25` -> `pub enum VerifiedVia`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/measure/field.rs:14` -> `pub enum VerifiedVia`
### `Vfs`

- `contextra-store`: `/home/freddy/Projekte/contextra/crates/contextra-store/src/io_seam.rs:42` -> `pub trait Vfs`
- `contextra-store`: `/home/freddy/Projekte/contextra/crates/contextra-store/src/io_seam/types.rs:60` -> `pub trait Vfs`
- `contextra-ports`: `/home/freddy/Projekte/contextra/crates/contextra-ports/src/vfs.rs:163` -> `pub trait Vfs`
- `contextra-durable-fs`: `/home/freddy/Projekte/contextra/crates/contextra-durable-fs/src/vfs.rs:12` -> `pub trait Vfs`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:2195` -> `pub trait Vfs`
### `VfsFile`

- `contextra-store`: `/home/freddy/Projekte/contextra/crates/contextra-store/src/io_seam.rs:32` -> `pub trait VfsFile`
- `contextra-store`: `/home/freddy/Projekte/contextra/crates/contextra-store/src/io_seam/types.rs:44` -> `pub trait VfsFile`
- `contextra-ports`: `/home/freddy/Projekte/contextra/crates/contextra-ports/src/vfs.rs:73` -> `pub trait VfsFile`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:1511` -> `pub trait VfsFile`
### `VfsOpenOptions`

- `contextra-store`: `/home/freddy/Projekte/contextra/crates/contextra-store/src/io_seam.rs:17` -> `pub struct VfsOpenOptions`
- `contextra-store`: `/home/freddy/Projekte/contextra/crates/contextra-store/src/io_seam/types.rs:23` -> `pub struct VfsOpenOptions`
- `contextra-ports`: `/home/freddy/Projekte/contextra/crates/contextra-ports/src/vfs.rs:37` -> `pub struct VfsOpenOptions`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:1343` -> `pub struct VfsOpenOptions`
### `WitnessLayer`

- `contextra-ports`: `/home/freddy/Projekte/contextra/crates/contextra-ports/src/scanner/layer.rs:13` -> `pub enum WitnessLayer`
- `contextra-erasure`: `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:1728` -> `pub enum WitnessLayer`
### `WorkflowState`

- `contextra-checkpoint`: `/home/freddy/Projekte/contextra/crates/contextra-checkpoint/src/meta.rs:18` -> `pub struct WorkflowState`
- `contextra-db`: `/home/freddy/Projekte/contextra/crates/contextra-db/src/orchestration/mod.rs:101` -> `pub struct WorkflowState`
- `contextra-types`: `/home/freddy/Projekte/contextra/crates/contextra-types/src/types/domain/misc.rs:61` -> `pub struct WorkflowState`
