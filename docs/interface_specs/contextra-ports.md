# Schnittstellenspezifikation: `contextra-ports`

> Erzeugt durch `tools/extract_interfaces.py` aus dem Quellcode. Keine Handpflege. Jede Angabe trägt Datei:Zeile. Maßgeblich bleibt der Quellcode.

## 1. Kopf

| Feld | Wert |
|---|---|
| Name | `contextra-ports` |
| Version | workspace |
| Pfad | `crates/contextra-ports` |
| Rolle | Domänen-Ports |
| Bibliothekspfad | src/lib.rs (Standard) |

**Crate-Dokumentation (//!):**

> Core trait definitions for Contextra subsystems.
>
> These traits define the abstract interfaces that concrete implementations
> must fulfill, enabling modularity and testability.

## 2. Abhängigkeiten

| Name | Art | Quelle | optional | verwendet von (normal/dev) |
|---|---|---|---|---|
| `ahash` | normal | workspace | nein |  |
| `blake3` | normal | workspace | nein |  |
| `bytes` | normal | workspace | nein |  |
| `contextra-env` | normal | workspace | nein | contextra-checkpoint, contextra-crypto, contextra-graph, contextra-infer-candle, contextra-kvcache, contextra-mvcc, contextra-ports, contextra-store, contextra-text, contextra-vector |
| `contextra-types` | normal | workspace | nein | contextra, contextra-adapt, contextra-agent, contextra-bench, contextra-checkpoint, contextra-cognition, contextra-crypto, contextra-crypto-fuzz, contextra-db, contextra-engine, contextra-graph, contextra-graph-fuzz, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-ingest, contextra-kvcache, contextra-mcp, contextra-mvcc, contextra-ports, contextra-py, contextra-rank, contextra-router, contextra-simd, contextra-store, contextra-store-fuzz, contextra-testkit, contextra-text, contextra-vault, contextra-vector, contextra-vector-fuzz |
| `futures-util` | normal | registry | nein |  |
| `proptest` | dev | registry | nein |  |
| `serde` | normal | workspace | nein |  |
| `serde_json` | normal | workspace | nein |  |
| `thiserror` | normal | workspace | nein |  |
| `tokio` | dev | workspace | nein |  |

**Verwendet von (Crate-Ebene):** `contextra`, `contextra-adapt`, `contextra-agent`, `contextra-checkpoint`, `contextra-cognition`, `contextra-crypto`, `contextra-crypto-fuzz`, `contextra-db`, `contextra-engine`, `contextra-graph`, `contextra-graph-fuzz`, `contextra-infer-candle`, `contextra-infer-ollama`, `contextra-infer-onnx`, `contextra-kvcache`, `contextra-mcp`, `contextra-mvcc`, `contextra-rank`, `contextra-router`, `contextra-store`, `contextra-store-fuzz`, `contextra-testkit`, `contextra-text`, `contextra-vector`, `contextra-vector-fuzz`

## 3. Features

| Feature | Aktiviert |
|---|---|
| `conformance` | `contract-tests` |
| `contract-tests` | – |
| `default` | – |
| `testkit` | – |

## 4. Metriken (Quellcode)

| Kennzahl | Wert |
|---|---|
| Quelldateien | 42 |
| Quellzeilen | 6755 |
| Öffentliche Items (Modul-Ebene) | 192 |
| davon `fn` | 30 |
| davon `struct` | 47 |
| davon `enum` | 20 |
| davon `trait` | 43 |
| davon `type` | 4 |
| davon `const` | 1 |
| davon `mod` | 47 |
| Öffentliche Makros (`#[macro_export]`) | 0 |
| Öffentliche Re-Exports (`pub use`) | 69 |
| `unsafe`-Vorkommen (Produktionscode) | 0 |
| `.unwrap()` (Produktionscode) | 2 |
| `.expect(` (Produktionscode) | 36 |
| `panic!` (Produktionscode) | 2 |
| `todo!/unimplemented!/unreachable!` | 0 |
| Indexierungen `x[…]` (grob, Produktionscode) | 1 |
| `#[deprecated]` | 17 |
| `#[cfg(feature…)]`-Gates | 0 |
| Parser-Abdeckung (geparste `pub`-Deklarationen / Grep-Treffer) | 258 / 258 = 100 % |

## 5. Modulbaum und Dateien

| Modul | Datei | Zeilen |
|---|---|---|
| `attention` | `crates/contextra-ports/src/attention.rs` | 21 |
| `barrier` | `crates/contextra-ports/src/barrier.rs` | 165 |
| `checkpoint` | `crates/contextra-ports/src/checkpoint.rs` | 61 |
| `clock` | `crates/contextra-ports/src/clock.rs` | 3 |
| `conformance` | `crates/contextra-ports/src/conformance.rs` | 278 |
| `contract_tests` | `crates/contextra-ports/src/contract_tests.rs` | 632 |
| `embedding` | `crates/contextra-ports/src/embedding.rs` | 272 |
| `env` | `crates/contextra-ports/src/env.rs` | 3 |
| `erasure` | `crates/contextra-ports/src/erasure.rs` | 272 |
| `erasure_legacy` | `crates/contextra-ports/src/erasure_legacy.rs` | 53 |
| `fs` | `crates/contextra-ports/src/fs.rs` | 11 |
| `gate` | `crates/contextra-ports/src/gate.rs` | 224 |
| `graph` | `crates/contextra-ports/src/graph.rs` | 219 |
| `graph_index` | `crates/contextra-ports/src/graph_index.rs` | 530 |
| `id_gen` | `crates/contextra-ports/src/id_gen.rs` | 3 |
| `index` | `crates/contextra-ports/src/index.rs` | 11 |
| `inference` | `crates/contextra-ports/src/inference.rs` | 17 |
| `key_material` | `crates/contextra-ports/src/key_material.rs` | 39 |
| `keys` | `crates/contextra-ports/src/keys.rs` | 204 |
| `kv` | `crates/contextra-ports/src/kv.rs` | 274 |
| `kv_bridge_port` | `crates/contextra-ports/src/kv_bridge_port.rs` | 23 |
| `ledger` | `crates/contextra-ports/src/ledger.rs` | 179 |
| `crate` | `crates/contextra-ports/src/lib.rs` | 211 |
| `license` | `crates/contextra-ports/src/license.rs` | 159 |
| `lifecycle` | `crates/contextra-ports/src/lifecycle.rs` | 69 |
| `metrics` | `crates/contextra-ports/src/metrics.rs` | 3 |
| `observability` | `crates/contextra-ports/src/observability.rs` | 18 |
| `observer` | `crates/contextra-ports/src/observer.rs` | 157 |
| `plugin` | `crates/contextra-ports/src/plugin.rs` | 464 |
| `policy` | `crates/contextra-ports/src/policy.rs` | 44 |
| `query_rewriter` | `crates/contextra-ports/src/query_rewriter.rs` | 46 |
| `report_signer` | `crates/contextra-ports/src/report_signer.rs` | 130 |
| `reranker` | `crates/contextra-ports/src/reranker.rs` | 32 |
| `rng` | `crates/contextra-ports/src/rng.rs` | 3 |
| `scanner::layer` | `crates/contextra-ports/src/scanner/layer.rs` | 123 |
| `scanner` | `crates/contextra-ports/src/scanner/mod.rs` | 219 |
| `scanner::session` | `crates/contextra-ports/src/scanner/session.rs` | 106 |
| `scanner::witness` | `crates/contextra-ports/src/scanner/witness.rs` | 108 |
| `storage` | `crates/contextra-ports/src/storage.rs` | 309 |
| `text_index` | `crates/contextra-ports/src/text_index.rs` | 276 |
| `vector_index` | `crates/contextra-ports/src/vector_index.rs` | 381 |
| `vfs` | `crates/contextra-ports/src/vfs.rs` | 403 |

## 6. Re-Exports (`pub use`)

- `pub use contextra_env::clock::*` — `crates/contextra-ports/src/clock.rs:3` (Modul `clock`)
- `pub use contextra_env::*` — `crates/contextra-ports/src/env.rs:3` (Modul `env`)
- `pub use crate::scanner::LayerWitness` — `crates/contextra-ports/src/erasure.rs:185` (Modul `erasure`)
- `pub use crate::barrier::` — `crates/contextra-ports/src/erasure_legacy.rs:10` (Modul `erasure_legacy`)
- `pub use crate::erasure::` — `crates/contextra-ports/src/erasure_legacy.rs:17` (Modul `erasure_legacy`)
- `pub use crate::gate::` — `crates/contextra-ports/src/erasure_legacy.rs:25` (Modul `erasure_legacy`)
- `pub use crate::keys::` — `crates/contextra-ports/src/erasure_legacy.rs:30` (Modul `erasure_legacy`)
- `pub use crate::ledger::` — `crates/contextra-ports/src/erasure_legacy.rs:35` (Modul `erasure_legacy`)
- `pub use crate::observer::` — `crates/contextra-ports/src/erasure_legacy.rs:40` (Modul `erasure_legacy`)
- `pub use crate::report_signer::ReportSigner` — `crates/contextra-ports/src/erasure_legacy.rs:45` (Modul `erasure_legacy`)
- `pub use crate::scanner::` — `crates/contextra-ports/src/erasure_legacy.rs:50` (Modul `erasure_legacy`)
- `pub use crate::vfs::` — `crates/contextra-ports/src/fs.rs:6` (Modul `fs`)
- `pub use super::` — `crates/contextra-ports/src/fs.rs:10` (Modul `fs::prelude`)
- `pub use crate::graph_index::` — `crates/contextra-ports/src/graph.rs:3` (Modul `graph`)
- `pub use contextra_env::id_gen::*` — `crates/contextra-ports/src/id_gen.rs:3` (Modul `id_gen`)
- `pub use crate::graph::` — `crates/contextra-ports/src/index.rs:3` (Modul `index`)
- `pub use crate::graph_index::` — `crates/contextra-ports/src/index.rs:4` (Modul `index`)
- `pub use crate::text_index::` — `crates/contextra-ports/src/index.rs:5` (Modul `index`)
- `pub use crate::vector_index::` — `crates/contextra-ports/src/index.rs:6` (Modul `index`)
- `pub use super::` — `crates/contextra-ports/src/index.rs:10` (Modul `index::prelude`)
- `pub use crate::attention::` — `crates/contextra-ports/src/inference.rs:3` (Modul `inference`)
- `pub use crate::embedding::` — `crates/contextra-ports/src/inference.rs:4` (Modul `inference`)
- `pub use crate::query_rewriter::QueryRewriter` — `crates/contextra-ports/src/inference.rs:8` (Modul `inference`)
- `pub use crate::reranker::Reranker` — `crates/contextra-ports/src/inference.rs:9` (Modul `inference`)
- `pub use super::` — `crates/contextra-ports/src/inference.rs:13` (Modul `inference::prelude`)
- `pub use contextra_types::` — `crates/contextra-ports/src/lib.rs:23` (Modul `crate`)
- `pub use attention::*` — `crates/contextra-ports/src/lib.rs:101` (Modul `crate`)
- `pub use checkpoint::*` — `crates/contextra-ports/src/lib.rs:102` (Modul `crate`)
- `pub use clock::*` — `crates/contextra-ports/src/lib.rs:103` (Modul `crate`)
- `pub use conformance::*` — `crates/contextra-ports/src/lib.rs:104` (Modul `crate`)
- `pub use embedding::*` — `crates/contextra-ports/src/lib.rs:105` (Modul `crate`)
- `pub use graph::*` — `crates/contextra-ports/src/lib.rs:106` (Modul `crate`)
- `pub use graph_index::*` — `crates/contextra-ports/src/lib.rs:107` (Modul `crate`)
- `pub use id_gen::*` — `crates/contextra-ports/src/lib.rs:108` (Modul `crate`)
- `pub use key_material::KeyMaterialSource` — `crates/contextra-ports/src/lib.rs:109` (Modul `crate`)
- `pub use kv::*` — `crates/contextra-ports/src/lib.rs:110` (Modul `crate`)
- `pub use kv_bridge_port::*` — `crates/contextra-ports/src/lib.rs:111` (Modul `crate`)
- `pub use license::*` — `crates/contextra-ports/src/lib.rs:112` (Modul `crate`)
- `pub use lifecycle::*` — `crates/contextra-ports/src/lib.rs:113` (Modul `crate`)
- `pub use metrics::*` — `crates/contextra-ports/src/lib.rs:114` (Modul `crate`)
- `pub use observability::*` — `crates/contextra-ports/src/lib.rs:115` (Modul `crate`)
- `pub use plugin::*` — `crates/contextra-ports/src/lib.rs:116` (Modul `crate`)
- `pub use query_rewriter::*` — `crates/contextra-ports/src/lib.rs:117` (Modul `crate`)
- `pub use reranker::*` — `crates/contextra-ports/src/lib.rs:118` (Modul `crate`)
- `pub use rng::*` — `crates/contextra-ports/src/lib.rs:119` (Modul `crate`)
- `pub use scanner::` — `crates/contextra-ports/src/lib.rs:120` (Modul `crate`)
- `pub use storage::` — `crates/contextra-ports/src/lib.rs:124` (Modul `crate`)
- `pub use text_index::*` — `crates/contextra-ports/src/lib.rs:125` (Modul `crate`)
- `pub use vector_index::*` — `crates/contextra-ports/src/lib.rs:126` (Modul `crate`)
- `pub use vfs::*` — `crates/contextra-ports/src/lib.rs:127` (Modul `crate`)
- `pub use metrics::` — `crates/contextra-ports/src/lib.rs:140` (Modul `crate`)
- `pub use barrier::` — `crates/contextra-ports/src/lib.rs:200` (Modul `crate`)
- `pub use erasure::` — `crates/contextra-ports/src/lib.rs:203` (Modul `crate`)
- `pub use gate::` — `crates/contextra-ports/src/lib.rs:207` (Modul `crate`)
- `pub use keys::` — `crates/contextra-ports/src/lib.rs:208` (Modul `crate`)
- `pub use ledger::` — `crates/contextra-ports/src/lib.rs:209` (Modul `crate`)
- `pub use observer::` — `crates/contextra-ports/src/lib.rs:210` (Modul `crate`)
- `pub use report_signer::ReportSigner` — `crates/contextra-ports/src/lib.rs:211` (Modul `crate`)
- `pub use contextra_env::metrics::*` — `crates/contextra-ports/src/metrics.rs:3` (Modul `metrics`)
- `pub use super::lifecycle::` — `crates/contextra-ports/src/observability.rs:7` (Modul `observability`)
- `pub use crate::license::` — `crates/contextra-ports/src/policy.rs:10` (Modul `policy`)
- `pub use super::` — `crates/contextra-ports/src/policy.rs:41` (Modul `policy::prelude`)
- `pub use contextra_env::rng::*` — `crates/contextra-ports/src/rng.rs:3` (Modul `rng`)
- `pub use layer::*` — `crates/contextra-ports/src/scanner/mod.rs:22` (Modul `scanner`)
- `pub use session::*` — `crates/contextra-ports/src/scanner/mod.rs:23` (Modul `scanner`)
- `pub use witness::*` — `crates/contextra-ports/src/scanner/mod.rs:24` (Modul `scanner`)
- `pub use super::` — `crates/contextra-ports/src/storage.rs:290` (Modul `storage::prelude`)
- `pub use crate::kv::KvPrefixStore` — `crates/contextra-ports/src/storage.rs:291` (Modul `storage::prelude`)
- `pub use crate::embedding::TextEmbeddingEngine` — `crates/contextra-ports/src/text_index.rs:21` (Modul `text_index`)

## 8. Öffentliche Schnittstelle je Item

### Modul `attention`

#### `struct RequestId`
- Ort: `crates/contextra-ports/src/attention.rs:7`
```rust
pub struct RequestId(pub u64)
```
Dokumentation: Unique identifier for an inference request within a process.

#### `trait AttentionExporter`
- Ort: `crates/contextra-ports/src/attention.rs:11`
```rust
pub trait AttentionExporter: Send + Sync
```
Dokumentation: Port implemented by inference backends (Ring 2) to export attention weights from the latest prefill step to the KV-cache eviction worker (Ring 1).

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn export_attention_weights(&self, request_id: RequestId) -> Option<Vec<f32>>` | `crates/contextra-ports/src/attention.rs:20` | Returns accumulated attention weights per token position for the given request ID. Returns `None` if no attention weights are tracked or available for the reque |

### Modul `barrier`

#### `struct BarrierExclusive`
- Ort: `crates/contextra-ports/src/barrier.rs:37`
```rust
pub struct BarrierExclusive
```
Dokumentation: RAII token guard representing an exclusive gate phase permit.
- Impl-Block `crates/contextra-ports/src/barrier.rs:41`
- Impl-Block (Trait `fmt::Debug`) `crates/contextra-ports/src/barrier.rs:48`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(token: Option<Arc<()>>) -> Self` | `crates/contextra-ports/src/barrier.rs:43` | Creates a new `BarrierExclusive` token. |

#### `struct BarrierShared`
- Ort: `crates/contextra-ports/src/barrier.rs:19`
```rust
pub struct BarrierShared
```
Dokumentation: RAII token guard representing a shared commit phase permit.
- Impl-Block `crates/contextra-ports/src/barrier.rs:23`
- Impl-Block (Trait `fmt::Debug`) `crates/contextra-ports/src/barrier.rs:30`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(token: Option<Arc<()>>) -> Self` | `crates/contextra-ports/src/barrier.rs:25` | Creates a new `BarrierShared` token. |

#### `struct CompactionPermit`
- Ort: `crates/contextra-ports/src/barrier.rs:55`
```rust
pub struct CompactionPermit
```
Dokumentation: RAII token guard representing a compaction permit.
- Impl-Block `crates/contextra-ports/src/barrier.rs:59`
- Impl-Block (Trait `fmt::Debug`) `crates/contextra-ports/src/barrier.rs:66`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(token: Option<Arc<()>>) -> Self` | `crates/contextra-ports/src/barrier.rs:61` | Creates a new `CompactionPermit` token. |

#### `struct PurgePermit`
- Ort: `crates/contextra-ports/src/barrier.rs:73`
```rust
pub struct PurgePermit
```
Dokumentation: RAII token guard representing an exclusive purge permit.
- Impl-Block `crates/contextra-ports/src/barrier.rs:77`
- Impl-Block (Trait `fmt::Debug`) `crates/contextra-ports/src/barrier.rs:84`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(token: Option<Arc<()>>) -> Self` | `crates/contextra-ports/src/barrier.rs:79` | Creates a new `PurgePermit` token. |

#### `trait CommitBarrier`
- Ort: `crates/contextra-ports/src/barrier.rs:91`
```rust
pub trait CommitBarrier: Send + Sync
```
Dokumentation: Barrier trait managing commit phases and exclusive gate operations (SPEC §11.3, D-57, D-78, L-7).

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn shared(&self) -> BoxFuture<'_, BarrierShared>` | `crates/contextra-ports/src/barrier.rs:99` | Acquires a shared permit for standard group commit operations. # Fehler Unfehlbar; wartet asynchron auf Commit-Freigabe. # Crash Reines Synchronisations-Token i |
| `fn exclusive(&self) -> BoxFuture<'_, BarrierExclusive>` | `crates/contextra-ports/src/barrier.rs:108` | Acquires an exclusive permit for gate group transitions. # Fehler Unfehlbar; blockiert nachfolgende Commits exklusiv. # Crash Exklusiver Token-Inhaber; cancel-s |

#### `trait PurgeBarrier`
- Ort: `crates/contextra-ports/src/barrier.rs:112`
```rust
pub trait PurgeBarrier: Send + Sync
```
Dokumentation: Barrier trait managing LSM compaction and physical purge mutual exclusion (SPEC §11.3, §11.4, I-16, L-7).

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn compaction(&self) -> BoxFuture<'_, CompactionPermit>` | `crates/contextra-ports/src/barrier.rs:120` | Acquires a shared permit for compaction. # Fehler Unfehlbar; wartet auf Purge-Freigabe. # Crash Kooperative Kompaktions-Sperre im RAM. |
| `fn purge(&self) -> BoxFuture<'_, PurgePermit>` | `crates/contextra-ports/src/barrier.rs:129` | Acquires an exclusive permit for physical purging. # Fehler Unfehlbar; exklusiver Purge-Zugriff. # Crash Erzwingt Abbruch kooperativer Kompaktionen; cancel-safe |
| `fn purge_waiting(&self) -> bool` | `crates/contextra-ports/src/barrier.rs:140` | Queries whether an exclusive purge operation is waiting for access. Background compactions inspect this periodically to cooperatively yield or abort. # Fehler K |

### Modul `checkpoint`

#### `trait Checkpoint`
- Ort: `crates/contextra-ports/src/checkpoint.rs:9`
```rust
pub trait Checkpoint: Send + Sync + 'static
```
Dokumentation: Abstract contract for generating consistent checkpoints.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn take_snapshot<'a>(&'a self, tx: TxId) -> BoxFuture<'a, Result<WorkflowState>>` | `crates/contextra-ports/src/checkpoint.rs:11` | Takes a deterministic snapshot of the current state. |
| `fn restore<'a>(&'a self, state: &'a WorkflowState) -> BoxFuture<'a, Result<()>>` | `crates/contextra-ports/src/checkpoint.rs:14` | Rolls the state back to the specified checkpoint. |

#### `trait CheckpointCoordinator`
- Ort: `crates/contextra-ports/src/checkpoint.rs:26`
```rust
pub trait CheckpointCoordinator: Send + Sync + 'static
```
Dokumentation: Unified Checkpoint Coordinator Trait combining named, TxId+seq_no-scoped, persistent checkpoints. # Dyn-Safety Note This trait is intentionally NOT dyn-compatible without specifying the associated type `Meta` (e.g. `Arc<dyn CheckpointCoordinator<Meta = ...>>`) or using a concrete type, due to the associated type `type Meta: Send + Sync`. # DECISION-REF ADR-011 — Consolidated Checkpoint Subsystem Architecture (resolving AGT-STORE-002).

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn create_named_checkpoint( &self, name: &str, collection_id: &str, seq_no: u64, tx_id: TxId, metadata: serde_json::Value, ) -> impl Future<Output = Result<Self::Meta>> + Send` | `crates/contextra-ports/src/checkpoint.rs:31` | Creates and persists a new named checkpoint. |
| `fn restore_named_checkpoint( &self, name: &str, ) -> impl Future<Output = Result<Self::Meta>> + Send` | `crates/contextra-ports/src/checkpoint.rs:41` | Restores database state to a named checkpoint. |
| `fn drop_named_checkpoint(&self, name: &str) -> impl Future<Output = Result<()>> + Send` | `crates/contextra-ports/src/checkpoint.rs:52` | Deletes a checkpoint by name. # Fehler Liefert `ContextraError::NotFound` falls der Checkpoint nicht existiert, oder I/O-Fehler. # Crash Atomare Manifest-/VFS-O |
| `fn list_named_checkpoints(&self) -> impl Future<Output = Result<Vec<Self::Meta>>> + Send` | `crates/contextra-ports/src/checkpoint.rs:60` | Lists all active checkpoints. # Fehler Liefert I/O- oder Deserialisierungsfehler beim Lesen der Checkpoint-Metadaten. # Crash Reiner Lesevorgang aus Manifest od |

Assoziierte Konstanten/Typen:

- `type Meta: Send + Sync` — `crates/contextra-ports/src/checkpoint.rs:28`

### Modul `conformance`

#### `mod check`
- Ort: `crates/contextra-ports/src/conformance.rs:10`
```rust
pub mod check
```
Dokumentation: Trait contract verification routines ensuring all trait methods are syntactically and semantically valid in generic contexts.

### Modul `conformance::check`

#### `fn assert_attention_exporter_contract`
- Ort: `crates/contextra-ports/src/conformance.rs:72`
```rust
pub fn assert_attention_exporter_contract<T: AttentionExporter>(t: &T, req: RequestId)
```
Dokumentation: Verifies that [`AttentionExporter`] methods can be invoked.

#### `fn assert_checkpoint_contract`
- Ort: `crates/contextra-ports/src/conformance.rs:38`
```rust
pub fn assert_checkpoint_contract<T: Checkpoint>(t: &T, tx: TxId, state: &WorkflowState)
```
Dokumentation: Verifies that [`Checkpoint`] methods can be invoked in generic contexts.

#### `fn assert_checkpoint_coordinator_contract`
- Ort: `crates/contextra-ports/src/conformance.rs:46`
```rust
pub async fn assert_checkpoint_coordinator_contract<T: CheckpointCoordinator>( t: &T, tx: TxId, metadata: serde_json::Value, )
```
Dokumentation: Verifies that [`CheckpointCoordinator`] methods can be invoked in generic contexts.

#### `fn assert_egress_policy_contract`
- Ort: `crates/contextra-ports/src/conformance.rs:234`
```rust
pub fn assert_egress_policy_contract<T: crate::policy::EgressPolicy>( t: &T, ctx: &RequestContext, payload: &str, )
```
Dokumentation: Verifies dynamic object-safety and evaluation conformance of [`crate::policy::EgressPolicy`].

#### `fn assert_embedding_contract`
- Ort: `crates/contextra-ports/src/conformance.rs:107`
```rust
pub fn assert_embedding_contract< T: EmbeddingProvider + LlmTextGeneratorStreaming + TextEmbeddingEngine + LlmTextGenerator + TextGenerator, >( t: &T, tenant: TenantId, seg: &[ContextSegment], )
```
Dokumentation: Verifies that [`EmbeddingProvider`], [`LlmTextGeneratorStreaming`], [`LlmTextGenerator`], and [`TextGenerator`] can be invoked.
- **deprecated**

#### `fn assert_erasure_layer_contract`
- Ort: `crates/contextra-ports/src/conformance.rs:223`
```rust
pub fn assert_erasure_layer_contract<E: crate::erasure::ErasureLayer>( e: &E, target: &crate::erasure::EraseTarget, scope: &crate::erasure::LayerScope, )
```
Dokumentation: Verifies that [`crate::erasure::ErasureLayer`] methods can be invoked.

#### `fn assert_graph_index_contract`
- Ort: `crates/contextra-ports/src/conformance.rs:77`
```rust
pub fn assert_graph_index_contract<T: GraphIndex + CommunityResolver>( t: &T, tx: TxId, e1: EntityId, e2: EntityId, )
```
Dokumentation: Verifies that [`GraphIndex`] and [`CommunityResolver`] methods can be invoked.

#### `fn assert_keys_contract`
- Ort: `crates/contextra-ports/src/conformance.rs:204`
```rust
pub fn assert_keys_contract<S: crate::keys::RecordSealer, K: crate::keys::FileKeyring>( s: &S, k: &K, file_id: &crate::keys::FileId, ctx: &crate::keys::SealCtx, key: &crate::gate::KeyBuf, )
```
Dokumentation: Verifies that [`crate::keys::RecordSealer`] and [`crate::keys::FileKeyring`] methods can be invoked.

#### `fn assert_ledger_contract`
- Ort: `crates/contextra-ports/src/conformance.rs:217`
```rust
pub fn assert_ledger_contract<L: crate::ledger::LedgerSink>(l: &L)
```
Dokumentation: Verifies that [`crate::ledger::LedgerSink`] methods can be invoked.

#### `fn assert_license_contract`
- Ort: `crates/contextra-ports/src/conformance.rs:134`
```rust
pub fn assert_license_contract<T: LicenseGate>(t: &T, ring: FeatureRing)
```
Dokumentation: Verifies that [`LicenseGate`] methods can be invoked.

#### `fn assert_lifecycle_contract`
- Ort: `crates/contextra-ports/src/conformance.rs:91`
```rust
pub fn assert_lifecycle_contract< T: GroundingValidator + ResponseGroundingValidator + ContextPreparer, >( t: &T, chunks: &[ContextChunk], assessment: &GroundingAssessment, budget: &TokenBudget, )
```
Dokumentation: Verifies that [`GroundingValidator`], [`ResponseGroundingValidator`], and [`ContextPreparer`] can be invoked.

#### `fn assert_observability_contract`
- Ort: `crates/contextra-ports/src/conformance.rs:129`
```rust
pub fn assert_observability_contract<T: DriftStatusProvider>(t: &T)
```
Dokumentation: Verifies that [`DriftStatusProvider`] methods can be invoked.

#### `fn assert_purger_contract`
- Ort: `crates/contextra-ports/src/conformance.rs:175`
```rust
pub fn assert_purger_contract<T: Purger>(t: &T, ctx: &RequestContext, target: &ScanTarget)
```
Dokumentation: Verifies that [`Purger`] methods can be invoked.

#### `fn assert_read_gate_contract`
- Ort: `crates/contextra-ports/src/conformance.rs:188`
```rust
pub fn assert_read_gate_contract<G: crate::gate::ReadGate>( g: &G, b: &crate::gate::KeyBound, r: &crate::gate::KeyRange, k: &crate::gate::KeyBuf, )
```
Dokumentation: Verifies that [`crate::gate::ReadGate`] methods can be invoked in generic contexts.

#### `fn assert_reranker_contract`
- Ort: `crates/contextra-ports/src/conformance.rs:140`
```rust
pub fn assert_reranker_contract<T: Reranker>(t: &T, results: &[RerankResult])
```
Dokumentation: Verifies that [`Reranker`] methods can be invoked.

#### `fn assert_storage_engine_contract`
- Ort: `crates/contextra-ports/src/conformance.rs:153`
```rust
pub fn assert_storage_engine_contract<T: StorageEngine>( t: &T, key: &[u8], val: &[u8], tx: TxId, )
```
Dokumentation: Verifies that [`StorageEngine`] methods can be invoked.

#### `fn assert_text_index_contract`
- Ort: `crates/contextra-ports/src/conformance.rs:147`
```rust
pub fn assert_text_index_contract<T: TextIndex + SegmentSynthesizer>(t: &T)
```
Dokumentation: Verifies that [`SegmentSynthesizer`] methods can be invoked.

#### `fn assert_vector_index_contract`
- Ort: `crates/contextra-ports/src/conformance.rs:61`
```rust
pub async fn assert_vector_index_contract<T: VectorIndex + HybridSearchProvider>( t: &T, doc: DocId, )
```
Dokumentation: Verifies that [`VectorIndex`] and [`HybridSearchProvider`] methods can be invoked.

#### `fn assert_vfs_file_contract`
- Ort: `crates/contextra-ports/src/conformance.rs:180`
```rust
pub fn assert_vfs_file_contract<F: crate::vfs::VfsFile>(f: &F, buf: &mut [u8])
```
Dokumentation: Verifies that [`crate::vfs::VfsFile`] methods can be invoked in generic contexts.

#### `fn assert_visitor_methods`
- Ort: `crates/contextra-ports/src/conformance.rs:243`
```rust
pub fn assert_visitor_methods()
```
Dokumentation: Internal visitor conformance verification for KV deserialization.

### Modul `contract_tests`

#### `struct MemVfs`
- Ort: `crates/contextra-ports/src/contract_tests.rs:371`
```rust
pub struct MemVfs
```
Dokumentation: In-memory VFS reference implementation used strictly for contract self-testing.
- Impl-Block `crates/contextra-ports/src/contract_tests.rs:376`
- Impl-Block (Trait `Vfs`) `crates/contextra-ports/src/contract_tests.rs:387`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-ports/src/contract_tests.rs:378` | Creates a new [`MemVfs`]. |

#### `fn verify_clock_contract`
- Ort: `crates/contextra-ports/src/contract_tests.rs:30`
```rust
pub fn verify_clock_contract<C: Clock>(clock: &C)
```
Dokumentation: Contract test helper verifying [`Clock`] trait implementation compliance. Checks monotonic non-decreasing behavior and valid wall-clock epoch timestamps.

#### `fn verify_layer_scanner_contract`
- Ort: `crates/contextra-ports/src/contract_tests.rs:349`
```rust
pub fn verify_layer_scanner_contract<L: LayerScanner>(scanner: &L, clock: &dyn Clock)
```
Dokumentation: Contract test helper verifying [`LayerScanner`] trait implementation compliance. Verifies positive control, scanned unit registration, and invariant `INV-WITNESS-MINT-1`.

#### `fn verify_metrics_sink_contract`
- Ort: `crates/contextra-ports/src/contract_tests.rs:167`
```rust
pub fn verify_metrics_sink_contract<M: MetricsSink>(sink: &M)
```
Dokumentation: Contract test helper verifying [`MetricsSink`] trait implementation compliance.

#### `fn verify_purger_contract`
- Ort: `crates/contextra-ports/src/contract_tests.rs:327`
```rust
pub fn verify_purger_contract<P: Purger>(purger: &P)
```
Dokumentation: Contract test helper verifying [`Purger`] trait implementation compliance. Verifies idempotency and safe fail-closed execution.

#### `fn verify_rng_contract`
- Ort: `crates/contextra-ports/src/contract_tests.rs:49`
```rust
pub fn verify_rng_contract<R: Rng>(rng: &mut R)
```
Dokumentation: Contract test helper verifying [`Rng`] trait implementation compliance. Checks non-zero outputs, correct buffer filling, and reproducible seeding if applicable.

#### `fn verify_storage_engine_contract`
- Ort: `crates/contextra-ports/src/contract_tests.rs:177`
```rust
pub async fn verify_storage_engine_contract<S: StorageEngine>(storage: &S)
```
Dokumentation: Contract test helper verifying [`StorageEngine`] trait implementation compliance. Verifies put/get, isolation, prefix scanning, delete, and commit lifecycle.

#### `fn verify_text_index_contract`
- Ort: `crates/contextra-ports/src/contract_tests.rs:288`
```rust
pub async fn verify_text_index_contract<T: TextIndex>(index: &T)
```
Dokumentation: Contract test helper verifying [`TextIndex`] trait implementation compliance. Verifies text insertion, search, delete, len, and commit.

#### `fn verify_vector_index_contract`
- Ort: `crates/contextra-ports/src/contract_tests.rs:249`
```rust
pub async fn verify_vector_index_contract<V: VectorIndex>(index: &V)
```
Dokumentation: Contract test helper verifying [`VectorIndex`] trait implementation compliance. Verifies insertion, search, delete, len, and transaction lifecycle.

#### `fn verify_vfs_contract`
- Ort: `crates/contextra-ports/src/contract_tests.rs:85`
```rust
pub fn verify_vfs_contract<V: Vfs>(vfs: &V, base_dir: &Path)
```
Dokumentation: Contract test helper verifying [`Vfs`] trait implementation compliance. Verifies atomic replacement, read roundtrips, directory creation, file removal, directory listing, and proper fail-closed errors on missing paths.

### Modul `crate`

#### `mod attention`
- Ort: `crates/contextra-ports/src/lib.rs:39`
```rust
pub mod attention
```
Dokumentation: Attention exporter trait and request ID types.

#### `mod barrier`
- Ort: `crates/contextra-ports/src/lib.rs:192`
```rust
pub mod barrier
```

#### `mod checkpoint`
- Ort: `crates/contextra-ports/src/lib.rs:41`
```rust
pub mod checkpoint
```
Dokumentation: Checkpoint and snapshot traits.

#### `mod clock`
- Ort: `crates/contextra-ports/src/lib.rs:43`
```rust
pub mod clock
```
Dokumentation: Clock port trait and system time implementation.

#### `mod conformance`
- Ort: `crates/contextra-ports/src/lib.rs:45`
```rust
pub mod conformance
```
Dokumentation: Trait conformance assertions and wiring verification matrix.

#### `mod contract_tests`
- Ort: `crates/contextra-ports/src/lib.rs:138`
```rust
pub mod contract_tests
```

#### `mod embedding`
- Ort: `crates/contextra-ports/src/lib.rs:47`
```rust
pub mod embedding
```
Dokumentation: Embedding provider and LLM generation traits.

#### `mod env`
- Ort: `crates/contextra-ports/src/lib.rs:89`
```rust
pub mod env
```
Dokumentation: Environment abstractions (Clock, Rng, IdGen, MetricsSink) with sub-prelude.

#### `mod erasure`
- Ort: `crates/contextra-ports/src/lib.rs:193`
```rust
pub mod erasure
```

#### `mod erasure_legacy`
- Ort: `crates/contextra-ports/src/lib.rs:91`
```rust
pub mod erasure_legacy
```
Dokumentation: Legacy erasure ports isolated under Decision D-E1 prior to relocation to `contextra-erasure-spi`.

#### `mod fs`
- Ort: `crates/contextra-ports/src/lib.rs:93`
```rust
pub mod fs
```
Dokumentation: Filesystem abstractions (Vfs, VfsFile) with sub-prelude.

#### `mod gate`
- Ort: `crates/contextra-ports/src/lib.rs:194`
```rust
pub mod gate
```

#### `mod graph`
- Ort: `crates/contextra-ports/src/lib.rs:49`
```rust
pub mod graph
```
Dokumentation: Graph mutation and CSR traversal port traits.

#### `mod graph_index`
- Ort: `crates/contextra-ports/src/lib.rs:51`
```rust
pub mod graph_index
```
Dokumentation: Graph index traits and CSR statistics.

#### `mod id_gen`
- Ort: `crates/contextra-ports/src/lib.rs:53`
```rust
pub mod id_gen
```
Dokumentation: Unique ID generator port trait and atomic implementation.

#### `mod index`
- Ort: `crates/contextra-ports/src/lib.rs:95`
```rust
pub mod index
```
Dokumentation: Indexing subsystem abstractions (VectorIndex, TextIndex, GraphIndex) with sub-prelude.

#### `mod inference`
- Ort: `crates/contextra-ports/src/lib.rs:97`
```rust
pub mod inference
```
Dokumentation: Inference subsystem abstractions (Embedding, Reranker, QueryRewriter) with sub-prelude.

#### `mod key_material`
- Ort: `crates/contextra-ports/src/lib.rs:86`
```rust
pub mod key_material
```
Dokumentation: Kryptografischer Port-Trait für Schlüssel- und Salt-Generierung (P28, ADR-098).

#### `mod keys`
- Ort: `crates/contextra-ports/src/lib.rs:195`
```rust
pub mod keys
```

#### `mod kv`
- Ort: `crates/contextra-ports/src/lib.rs:55`
```rust
pub mod kv
```
Dokumentation: Key-value prefix store traits and types.

#### `mod kv_bridge_port`
- Ort: `crates/contextra-ports/src/lib.rs:57`
```rust
pub mod kv_bridge_port
```
Dokumentation: Key-value bridge storage port trait.

#### `mod ledger`
- Ort: `crates/contextra-ports/src/lib.rs:196`
```rust
pub mod ledger
```

#### `mod license`
- Ort: `crates/contextra-ports/src/lib.rs:59`
```rust
pub mod license
```
Dokumentation: License and activation gate port traits.

#### `mod lifecycle`
- Ort: `crates/contextra-ports/src/lib.rs:61`
```rust
pub mod lifecycle
```
Dokumentation: Grounding validator traits and context preparer contracts.

#### `mod metrics`
- Ort: `crates/contextra-ports/src/lib.rs:63`
```rust
pub mod metrics
```
Dokumentation: Metrics reporting port trait.

#### `mod observability`
- Ort: `crates/contextra-ports/src/lib.rs:65`
```rust
pub mod observability
```
Dokumentation: Observability and lifecycle re-exports.

#### `mod observer`
- Ort: `crates/contextra-ports/src/lib.rs:197`
```rust
pub mod observer
```

#### `mod plugin`
- Ort: `crates/contextra-ports/src/lib.rs:67`
```rust
pub mod plugin
```
Dokumentation: Plugin registry and dependency resolution.

#### `mod policy`
- Ort: `crates/contextra-ports/src/lib.rs:99`
```rust
pub mod policy
```
Dokumentation: Policy subsystem abstractions (LicenseGate, EgressPolicy) with sub-prelude.

#### `mod query_rewriter`
- Ort: `crates/contextra-ports/src/lib.rs:69`
```rust
pub mod query_rewriter
```
Dokumentation: QueryRewriter port trait for multi-step retrieval sub-query generation.

#### `mod report_signer`
- Ort: `crates/contextra-ports/src/lib.rs:198`
```rust
pub mod report_signer
```

#### `mod reranker`
- Ort: `crates/contextra-ports/src/lib.rs:71`
```rust
pub mod reranker
```
Dokumentation: Reranker port trait for cross-encoder post-retrieval ranking.

#### `mod rng`
- Ort: `crates/contextra-ports/src/lib.rs:73`
```rust
pub mod rng
```
Dokumentation: Random number generator port trait and SplitMix64 implementation.

#### `mod scanner`
- Ort: `crates/contextra-ports/src/lib.rs:75`
```rust
pub mod scanner
```
Dokumentation: Storage layer scanners and physical purgers (Vertrag V1, SPEC §12.7.4).

#### `mod storage`
- Ort: `crates/contextra-ports/src/lib.rs:77`
```rust
pub mod storage
```
Dokumentation: Key-value storage engine traits.

#### `mod text_index`
- Ort: `crates/contextra-ports/src/lib.rs:79`
```rust
pub mod text_index
```
Dokumentation: Text retrieval and indexing traits (BM25 / Inverted Index).

#### `mod vector_index`
- Ort: `crates/contextra-ports/src/lib.rs:81`
```rust
pub mod vector_index
```
Dokumentation: Vector retrieval and HNSW indexing traits.

#### `mod vfs`
- Ort: `crates/contextra-ports/src/lib.rs:83`
```rust
pub mod vfs
```
Dokumentation: Virtual File System (VFS) port interface and contracts.

#### `type BoxFuture`
- Ort: `crates/contextra-ports/src/lib.rs:33`
```rust
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>
```
Dokumentation: Type alias for a pinned, heap-allocated `Future` that is `Send` and dyn-compatible.

#### `type BoxStream`
- Ort: `crates/contextra-ports/src/lib.rs:36`
```rust
pub type BoxStream<'a, T> = Pin<Box<dyn futures_util::stream::Stream<Item = T> + Send + 'a>>
```
Dokumentation: Type alias for a pinned, heap-allocated `Stream` that is `Send` and dyn-compatible.

### Modul `embedding`

#### `struct ContextSegment`
- Ort: `crates/contextra-ports/src/embedding.rs:20`
```rust
pub struct ContextSegment<'a>
```
Dokumentation: Representation of a context segment passed to context-aware text generation.
- Impl-Block `crates/contextra-ports/src/embedding.rs:31`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `chunk_id` | `u64` | `crates/contextra-ports/src/embedding.rs:22` |
| `text` | `&'a str` | `crates/contextra-ports/src/embedding.rs:24` |
| `model_fingerprint` | `Option<&'a ModelFingerprint>` | `crates/contextra-ports/src/embedding.rs:26` |
| `rope_offset` | `Option<usize>` | `crates/contextra-ports/src/embedding.rs:28` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(chunk_id: u64, text: &'a str) -> Self` | `crates/contextra-ports/src/embedding.rs:33` | Creates a new `ContextSegment` with default optional metadata set to `None`. |
| `pub fn with_fingerprint(mut self, fingerprint: &'a ModelFingerprint) -> Self` | `crates/contextra-ports/src/embedding.rs:43` | Builder method to set the model fingerprint. |
| `pub fn with_rope_offset(mut self, offset: usize) -> Self` | `crates/contextra-ports/src/embedding.rs:49` | Builder method to set the RoPE offset. |

#### `enum EmbeddingError`
- Ort: `crates/contextra-ports/src/embedding.rs:79`
```rust
pub enum EmbeddingError
```
Dokumentation: Error types encountered during embedding operations.

Varianten:

- `Unavailable(String)` — `crates/contextra-ports/src/embedding.rs:82`
- `ComputationFailed(String)` — `crates/contextra-ports/src/embedding.rs:85`
- `InputTooLong{` — `crates/contextra-ports/src/embedding.rs:88`

#### `trait EmbeddingProvider`
- Ort: `crates/contextra-ports/src/embedding.rs:98`
```rust
pub trait EmbeddingProvider: Send + Sync + 'static
```
Dokumentation: Abstraction for all embedding providers (e.g. OllamaEmbedder, OnnxEmbedder).

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn provider_name(&self) -> &str` | `crates/contextra-ports/src/embedding.rs:105` | Returns the provider identifier/name (e.g. for logging/debugging). # Fehler Keine; statische Identifikator-Abfrage. # Crash Reiner Statusabruf ohne Mutation ode |
| `fn embed<'a>(&'a self, text: &'a str) -> BoxFuture<'a, Result<Vec<f32>, EmbeddingError>>` | `crates/contextra-ports/src/embedding.rs:108` | Generates an embedding vector for the given text. |
| `fn embedding_dim(&self) -> usize` | `crates/contextra-ports/src/embedding.rs:116` | Returns the dimensionality of embeddings generated by this provider. # Fehler Keine; statische Dimensionsabfrage. # Crash Reiner Statusabruf ohne Mutation oder  |
| `fn embed_batch<'a>( &'a self, texts: &'a [&'a str], ) -> BoxFuture<'a, Result<Vec<Vec<f32>>, EmbeddingError>>` | `crates/contextra-ports/src/embedding.rs:119` | Generates embeddings for multiple texts. |

#### `trait LlmTextGenerator`
- Ort: `crates/contextra-ports/src/embedding.rs:144`
```rust
pub trait LlmTextGenerator: Send + Sync + 'static
```
Dokumentation: Abstract contract for LLM text generation (summarization, importance evaluation, query expansion).

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn generate<'a>(&'a self, prompt: &'a str) -> BoxFuture<'a, crate::Result<String>>` | `crates/contextra-ports/src/embedding.rs:146` | Generates text for a given prompt using an LLM. |
| `fn generate_with_context<'a>( &'a self, tenant: TenantId, segments: &'a [ContextSegment<'a>], ) -> BoxFuture<'a, crate::Result<String>>` | `crates/contextra-ports/src/embedding.rs:151` | Generates text for a given sequence of context segments using an LLM. Default implementation naively concatenates segment texts with `"\n\n"` and invokes `self. |

#### `trait LlmTextGeneratorStreaming`
- Ort: `crates/contextra-ports/src/embedding.rs:169`
```rust
pub trait LlmTextGeneratorStreaming: LlmTextGenerator
```
Dokumentation: Abstract contract for streaming LLM text generation (token by token / chunk by chunk).

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn generate_stream<'a>( &'a self, prompt: &'a str, config: &'a ConfigFingerprint, ) -> super::BoxStream<'a, crate::Result<String>>` | `crates/contextra-ports/src/embedding.rs:171` | Generates text stream for a given prompt using an LLM configuration. |

#### `trait TextEmbeddingEngine`
- Ort: `crates/contextra-ports/src/embedding.rs:56`
```rust
pub trait TextEmbeddingEngine: Send + Sync + 'static
```
Dokumentation: Text embedding engine trait.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn embed<'a>(&'a self, text: &'a str) -> BoxFuture<'a, crate::Result<Vec<f32>>>` | `crates/contextra-ports/src/embedding.rs:58` | Generates an embedding for the given text. |
| `fn embed_batch<'a>( &'a self, texts: &'a [&'a str], ) -> BoxFuture<'a, crate::Result<Vec<Vec<f32>>>>` | `crates/contextra-ports/src/embedding.rs:62` | Generates embeddings for multiple texts. Default implementation executes sequential calls. |

#### `trait TextGenerator`
- Ort: `crates/contextra-ports/src/embedding.rs:137`
```rust
pub trait TextGenerator: Send + Sync + 'static
```

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn generate_text<'a>(&'a self, prompt: &'a str) -> BoxFuture<'a, crate::Result<String>>` | `crates/contextra-ports/src/embedding.rs:139` | Generates text response for a given prompt. |

### Modul `erasure`

#### `struct Armed`
- Ort: `crates/contextra-ports/src/erasure.rs:151`
```rust
pub struct Armed
```
Dokumentation: Phantom state for an armed scanner verifying deletion residue.

#### `struct LayerId`
- Ort: `crates/contextra-ports/src/erasure.rs:13`
```rust
pub struct LayerId(pub [u8
```
Dokumentation: Layer identifier representing a specific subsystem participating in erasure (e.g. store, text, vector).
- Impl-Block `crates/contextra-ports/src/erasure.rs:15`
- Impl-Block (Trait `fmt::Debug`) `crates/contextra-ports/src/erasure.rs:27`
- Impl-Block (Trait `fmt::Display`) `crates/contextra-ports/src/erasure.rs:33`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn new(raw: [u8` | `crates/contextra-ports/src/erasure.rs:17` | Creates a new `LayerId` from raw bytes. |
| `pub fn as_bytes(&self) -> &[u8` | `crates/contextra-ports/src/erasure.rs:22` | Returns a slice of raw bytes. |

#### `struct LayerScope`
- Ort: `crates/contextra-ports/src/erasure.rs:70`
```rust
pub struct LayerScope
```
Dokumentation: Scope resolved by an erasure layer prior to gating (SPEC §8.2, D-96). Records the concrete file sets, segment IDs, or key intervals that this layer must purge and witness.
- Impl-Block `crates/contextra-ports/src/erasure.rs:77`
- Impl-Block (Trait `fmt::Debug`) `crates/contextra-ports/src/erasure.rs:105`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn empty() -> Self` | `crates/contextra-ports/src/erasure.rs:79` | Creates an empty scope. |
| `pub fn new(scope_id: [u8` | `crates/contextra-ports/src/erasure.rs:87` | Creates a new `LayerScope` with specific units. |
| `pub fn scope_id(&self) -> &[u8` | `crates/contextra-ports/src/erasure.rs:95` | Returns the unique scope ID. |
| `pub fn affected_units(&self) -> &[u64]` | `crates/contextra-ports/src/erasure.rs:100` | Returns affected units count. |

#### `struct PurgeCx`
- Ort: `crates/contextra-ports/src/erasure.rs:118`
```rust
pub struct PurgeCx
```
Dokumentation: Context passed into [`ErasureLayer::purge`] providing operational bounds and budgets.
- Impl-Block `crates/contextra-ports/src/erasure.rs:123`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `deadline_ms` | `u64` | `crates/contextra-ports/src/erasure.rs:120` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn new(deadline_ms: u64) -> Self` | `crates/contextra-ports/src/erasure.rs:125` | Creates a new purge context. |

#### `struct PurgeOutcome`
- Ort: `crates/contextra-ports/src/erasure.rs:132`
```rust
pub struct PurgeOutcome
```
Dokumentation: Outcome of a layer purge execution (SPEC §8.2, WP-3.1).
- Impl-Block `crates/contextra-ports/src/erasure.rs:139`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `purged_count` | `u64` | `crates/contextra-ports/src/erasure.rs:134` |
| `bytes_reclaimed` | `u64` | `crates/contextra-ports/src/erasure.rs:136` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn new(purged_count: u64, bytes_reclaimed: u64) -> Self` | `crates/contextra-ports/src/erasure.rs:141` | Creates a new purge outcome. |

#### `struct Scanner`
- Ort: `crates/contextra-ports/src/erasure.rs:156`
```rust
pub struct Scanner<State>
```
Dokumentation: Residue scanner type passed to [`ErasureLayer::witness`]. Must be purely read-only (PO-05).
- Impl-Block `crates/contextra-ports/src/erasure.rs:160`
- Impl-Block (Trait `Default`) `crates/contextra-ports/src/erasure.rs:169`
- Impl-Block (Trait `fmt::Debug`) `crates/contextra-ports/src/erasure.rs:175`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn new() -> Self` | `crates/contextra-ports/src/erasure.rs:162` | Creates an armed read-only scanner. |

#### `enum EraseTarget`
- Ort: `crates/contextra-ports/src/erasure.rs:44`
```rust
pub enum EraseTarget
```
Dokumentation: Target of an erasure request specifying what needs to be erased.
- Impl-Block (Trait `fmt::Debug`) `crates/contextra-ports/src/erasure.rs:56`

Varianten:

- `KeyRange{` — `crates/contextra-ports/src/erasure.rs:46`
- `ExactKeys(Vec<Vec<u8>>)` — `crates/contextra-ports/src/erasure.rs:53`

#### `enum KeyCoverage`
- Ort: `crates/contextra-ports/src/erasure.rs:189`
```rust
pub enum KeyCoverage
```
Dokumentation: Key coverage capability of a layer (SPEC §8.2, D-69).

Varianten:

- `Full` — `crates/contextra-ports/src/erasure.rs:191`
- `Unlinked` — `crates/contextra-ports/src/erasure.rs:193`
- `None` — `crates/contextra-ports/src/erasure.rs:195`

#### `trait ErasureLayer`
- Ort: `crates/contextra-ports/src/erasure.rs:206`
```rust
pub trait ErasureLayer: Send + Sync
```
Dokumentation: Primary trait for storage and index subsystems participating in the erasure saga. Follows SPEC §8.2 verbatim. # Invariants - **PO-05 (Read-only witness)**: `witness` accepts only `&Scanner<Armed>` and performs no mutations. - **PO-05 (Idempotency)**: `purge` must be idempotent; repeated calls with the same scope must yield safe, repeatable outcomes without double-freeing or state corruption.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn id(&self) -> LayerId` | `crates/contextra-ports/src/erasure.rs:214` | Returns the unique layer identifier. # Fehler Keine; statische Identifikator-Abfrage. # Crash Reiner Statusabruf ohne persistente Seiteneffekte. |
| `fn resolve(&self, t: &EraseTarget) -> BoxFuture<'_, Result<LayerScope, String>>` | `crates/contextra-ports/src/erasure.rs:223` | Resolves the concrete scope affected by the erase target before gating. # Fehler Liefert einen Fehler-String falls das EraseTarget im Layer nicht aufgelöst werd |
| `fn on_gate(&self, s: &LayerScope) -> BoxFuture<'_, Result<(), String>>` | `crates/contextra-ports/src/erasure.rs:232` | Immediate actions prior to gating (e.g. vector surrogate marking). # Fehler Liefert einen Fehler-String falls Vorbereitungsmaßnahmen vor dem Gate fehlschlagen.  |
| `fn purge(&self, s: &LayerScope, cx: &PurgeCx) -> BoxFuture<'_, Result<PurgeOutcome, String>>` | `crates/contextra-ports/src/erasure.rs:243` | Executes the physical purge for the given scope within the operational context. Must be idempotent and repeatable (PO-05). # Fehler Liefert Fehler bei I/O- oder |
| `fn witness( &self, s: &LayerScope, sc: &Scanner<Armed>, ) -> BoxFuture<'_, Result<LayerWitness, String>>` | `crates/contextra-ports/src/erasure.rs:252` | Verifies residue and produces a witness. Read-only (PO-05). # Fehler Liefert Fehler falls Residuen des gelöschten Umfangs gefunden werden oder I/O fehlschlägt.  |
| `fn key_coverage(&self) -> KeyCoverage` | `crates/contextra-ports/src/erasure.rs:265` | Reports the key coverage classification (full \| unlinked \| none). # Fehler Keine; statische Klassifikationsabfrage. # Crash Reiner Statusabruf ohne Mutation. |

#### `trait ErasureTarget`
- Ort: `crates/contextra-ports/src/erasure.rs:272`
```rust
pub trait ErasureTarget: Send + Sync + 'static
```
Dokumentation: Generisches Erasure-Target für Bereinigungsanweisungen.

#### `trait StorageBackend`
- Ort: `crates/contextra-ports/src/erasure.rs:269`
```rust
pub trait StorageBackend: Send + Sync + 'static
```
Dokumentation: Generisches Storage-Backend für modulare Datenzugriffe im Löschkern.

### Modul `fs`

#### `mod prelude`
- Ort: `crates/contextra-ports/src/fs.rs:9`
```rust
pub mod prelude
```
Dokumentation: Prelude for filesystem ports.

### Modul `gate`

#### `struct KeyBuf`
- Ort: `crates/contextra-ports/src/gate.rs:26`
```rust
pub struct KeyBuf(pub [u8
```
Dokumentation: Fixed or bounded key buffer for storage operations (SPEC §8.1).
- Impl-Block `crates/contextra-ports/src/gate.rs:28`
- Impl-Block (Trait `fmt::Debug`) `crates/contextra-ports/src/gate.rs:40`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn new(raw: [u8` | `crates/contextra-ports/src/gate.rs:30` | Creates a new `KeyBuf` from raw bytes. |
| `pub fn as_bytes(&self) -> &[u8` | `crates/contextra-ports/src/gate.rs:35` | Returns raw bytes slice. |

#### `struct KeyRange`
- Ort: `crates/contextra-ports/src/gate.rs:77`
```rust
pub struct KeyRange
```
Dokumentation: Half-open key range `[start, end)` for scan and compaction boundaries.
- Impl-Block `crates/contextra-ports/src/gate.rs:84`
- Impl-Block (Trait `fmt::Debug`) `crates/contextra-ports/src/gate.rs:96`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `start` | `KeyBuf` | `crates/contextra-ports/src/gate.rs:79` |
| `end` | `KeyBuf` | `crates/contextra-ports/src/gate.rs:81` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn new(start: KeyBuf, end: KeyBuf) -> Self` | `crates/contextra-ports/src/gate.rs:86` | Creates a new key range. |
| `pub fn contains(&self, key: &KeyBuf) -> bool` | `crates/contextra-ports/src/gate.rs:91` | Returns `true` if `key` falls within `[start, end)`. |

#### `struct Seq`
- Ort: `crates/contextra-ports/src/gate.rs:10`
```rust
pub struct Seq(pub u64)
```
Dokumentation: Monotonic sequence number for transactions and gate markers.
- Impl-Block `crates/contextra-ports/src/gate.rs:12`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn new(val: u64) -> Self` | `crates/contextra-ports/src/gate.rs:14` | Creates a new sequence number. |
| `pub const fn as_u64(&self) -> u64` | `crates/contextra-ports/src/gate.rs:19` | Returns the underlying numeric sequence. |

#### `enum GateDecision`
- Ort: `crates/contextra-ports/src/gate.rs:104`
```rust
pub enum GateDecision
```
Dokumentation: Gate decision for point queries or file boundaries (SPEC §4.2, §11.3).

Varianten:

- `Open` — `crates/contextra-ports/src/gate.rs:106`
- `Erased` — `crates/contextra-ports/src/gate.rs:108`
- `Restricted` — `crates/contextra-ports/src/gate.rs:110`

#### `enum KeyBound`
- Ort: `crates/contextra-ports/src/gate.rs:49`
```rust
pub enum KeyBound
```
Dokumentation: Boundary key used for compaction or range scanning.
- Impl-Block `crates/contextra-ports/src/gate.rs:58`
- Impl-Block (Trait `fmt::Debug`) `crates/contextra-ports/src/gate.rs:65`

Varianten:

- `Exact(KeyBuf)` — `crates/contextra-ports/src/gate.rs:51`
- `UnboundedLower` — `crates/contextra-ports/src/gate.rs:53`
- `UnboundedUpper` — `crates/contextra-ports/src/gate.rs:55`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn exact(key: KeyBuf) -> Self` | `crates/contextra-ports/src/gate.rs:60` | Creates an exact key boundary. |

#### `enum RangeGate`
- Ort: `crates/contextra-ports/src/gate.rs:115`
```rust
pub enum RangeGate
```
Dokumentation: Decision variant for range operations during compaction (SPEC §4.2, I-18).

Varianten:

- `ContainsErased` — `crates/contextra-ports/src/gate.rs:117`
- `RestrictedNeverDiscard` — `crates/contextra-ports/src/gate.rs:119`
- `EntirelyOpen` — `crates/contextra-ports/src/gate.rs:121`

#### `trait ReadGate`
- Ort: `crates/contextra-ports/src/gate.rs:128`
```rust
pub trait ReadGate: Send + Sync
```
Dokumentation: Synchronous, memory-only read gate port interface (SPEC §4.2, §11.3, PO-07). # Invariants - **PO-07**: `ReadGate` is synchronous, performs zero I/O, and executes in $O(\log n)$ time.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn check(&self, k: &KeyBuf) -> GateDecision` | `crates/contextra-ports/src/gate.rs:136` | Checks point status of an individual key. # Fehler Synchron, keine I/O, panikfrei; liefert GateDecision. # Crash Reiner RAM-Lookup in O(log n); keine Zustandsän |
| `fn check_bound(&self, b: &KeyBound) -> GateDecision` | `crates/contextra-ports/src/gate.rs:145` | Checks boundary status of a file or partition block for compaction or scans. # Fehler Synchron, keine I/O; liefert GateDecision. # Crash Reiner RAM-Lookup; kein |
| `fn variant_for_range(&self, r: &KeyRange) -> RangeGate` | `crates/contextra-ports/src/gate.rs:154` | Evaluates range policy for compaction (I-18: Erased may be dropped, Restricted never). # Fehler Synchron, keine I/O; liefert RangeGate. # Crash Reiner RAM-Looku |
| `fn gated_seq(&self, k: &KeyBuf) -> Option<Seq>` | `crates/contextra-ports/src/gate.rs:163` | Retrieves the sequence number at which the key was gated, if any (Restore-Guard, commit rejection). # Fehler Synchron, liefert `Option<Seq>`. # Crash Reiner RAM |
| `fn gate_epoch(&self) -> u64` | `crates/contextra-ports/src/gate.rs:173` | Returns the monotonic gate epoch counter (SPEC §11.3, D-120, I-21). Incremented before key destruction to allow linearized read verification. # Fehler Keine; sy |
| `fn covers(&self, k: &KeyBuf) -> bool` | `crates/contextra-ports/src/gate.rs:185` | Evaluates whether a 16-byte `KeyBuf` is covered by an active gate (Erased or Restricted) (SPEC I-2). Used by replay filters and commit leader phase. # Fehler Ke |
| `fn check_raw(&self, _raw: &[u8` | `crates/contextra-ports/src/gate.rs:198` | Evaluates point gate status of a canonical 78-byte raw key (SPEC 9.1, D-123). # Fehler Keine; synchroner ram-basierter Lookup. # Crash Reiner RAM-Zugriff ohne M |
| `fn covers_raw(&self, raw: &[u8` | `crates/contextra-ports/src/gate.rs:208` | Evaluates whether a canonical 78-byte raw key is covered by an active gate (SPEC I-2, I-21, D-123). # Fehler Keine; synchroner ram-basierter Lookup. # Crash Rei |
| `fn gated_seq_raw(&self, _raw: &[u8` | `crates/contextra-ports/src/gate.rs:221` | Retrieves the sequence number at which a 78-byte raw key was gated, if any. # Fehler Keine; synchroner ram-basierter Lookup. # Crash Reiner RAM-Zugriff ohne Mut |

### Modul `graph`

#### `struct HyperEdgeId`
- Ort: `crates/contextra-ports/src/graph.rs:63`
```rust
pub struct HyperEdgeId(pub u64)
```
- Impl-Block `crates/contextra-ports/src/graph.rs:65`
- Impl-Block (Trait `From<u64>`) `crates/contextra-ports/src/graph.rs:79`
- Impl-Block (Trait `std::fmt::Display`) `crates/contextra-ports/src/graph.rs:85`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn new(id: u64) -> Self` | `crates/contextra-ports/src/graph.rs:68` | Creates a new `HyperEdgeId`. |
| `pub const fn inner(self) -> u64` | `crates/contextra-ports/src/graph.rs:74` | Returns the underlying `u64` value. |

#### `struct RoleBinding`
- Ort: `crates/contextra-ports/src/graph.rs:44`
```rust
pub struct RoleBinding
```
Dokumentation: Binding of an entity to a specific role within an n-ary hyperedge.
- Impl-Block `crates/contextra-ports/src/graph.rs:51`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `role` | `RoleId` | `crates/contextra-ports/src/graph.rs:46` |
| `entity` | `EntityId` | `crates/contextra-ports/src/graph.rs:48` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(role: RoleId, entity: EntityId) -> Self` | `crates/contextra-ports/src/graph.rs:53` | Creates a new `RoleBinding`. |

#### `struct RoleId`
- Ort: `crates/contextra-ports/src/graph.rs:14`
```rust
pub struct RoleId(pub u32)
```
- Impl-Block `crates/contextra-ports/src/graph.rs:16`
- Impl-Block (Trait `From<u32>`) `crates/contextra-ports/src/graph.rs:30`
- Impl-Block (Trait `std::fmt::Display`) `crates/contextra-ports/src/graph.rs:36`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn new(id: u32) -> Self` | `crates/contextra-ports/src/graph.rs:19` | Creates a new `RoleId`. |
| `pub const fn inner(self) -> u32` | `crates/contextra-ports/src/graph.rs:25` | Returns the underlying `u32` value. |

#### `enum GraphMutationError`
- Ort: `crates/contextra-ports/src/graph.rs:93`
```rust
pub enum GraphMutationError
```
Dokumentation: Error variants for graph mutation operations.

Varianten:

- `LockAcquisitionTimeout(String)` — `crates/contextra-ports/src/graph.rs:96`
- `DuplicateHyperEdgeId(HyperEdgeId)` — `crates/contextra-ports/src/graph.rs:100`
- `RoleBindingInvalid(String)` — `crates/contextra-ports/src/graph.rs:104`
- `EpochReclamationPending(u64)` — `crates/contextra-ports/src/graph.rs:108`
- `InsufficientParticipants{` — `crates/contextra-ports/src/graph.rs:114`
- `HyperedgeNotFound(u64)` — `crates/contextra-ports/src/graph.rs:123`
- `InvalidWeight{` — `crates/contextra-ports/src/graph.rs:127`
- `Internal(String)` — `crates/contextra-ports/src/graph.rs:136`

#### `trait GraphCollectionMutation`
- Ort: `crates/contextra-ports/src/graph.rs:149`
```rust
pub trait GraphCollectionMutation: Send + Sync + 'static
```
Dokumentation: Trait defining n-ary hyperedge collection mutations for graph indexing (Anhang B §B.5.1.3). # Dyn-Kompatibilität Dieser Trait ist durch synchrone Rückgabetypen vtable-kompatibel (dyn-safe).

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn relate_n_ary( &self, predicate_tag: u32, participants: &[RoleBinding], doc_id: DocId, ) -> Result<HyperEdgeId, GraphMutationError>` | `crates/contextra-ports/src/graph.rs:151` | Relates multiple entities into an n-ary hyperedge associated with a document ID. |

### Modul `graph_index`

#### `struct GraphIndexStats`
- Ort: `crates/contextra-ports/src/graph_index.rs:314`
```rust
pub struct GraphIndexStats
```
Dokumentation: Statistics for the GraphIndex layer.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `num_entities` | `usize` | `crates/contextra-ports/src/graph_index.rs:316` |
| `num_edges` | `usize` | `crates/contextra-ports/src/graph_index.rs:318` |
| `memory_usage_bytes` | `usize` | `crates/contextra-ports/src/graph_index.rs:320` |

#### `trait CommunityResolver`
- Ort: `crates/contextra-ports/src/graph_index.rs:325`
```rust
pub trait CommunityResolver: Send + Sync
```
Dokumentation: Contract for resolving graph community assignments for entities.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn get_community<'a>(&'a self, entity_id: EntityId) -> BoxFuture<'a, Result<Option<u64>>>` | `crates/contextra-ports/src/graph_index.rs:327` | Resolves the optional community ID for a given entity. |

#### `trait GraphIndex`
- Ort: `crates/contextra-ports/src/graph_index.rs:58`
```rust
pub trait GraphIndex: Send + Sync + 'static
```
Dokumentation: Graph-Index Trait — CSR-basierte Entity-Relation-Traversal. # Dyn-Kompatibilität Dieser Trait ist durch explizite `BoxFuture`-Rückgabetypen vtable-kompatibel (dyn-safe). # TxId-Origin-Invariant (AGT-GRAPH-001) **Aufrufer MÜSSEN tx entweder aus der Collection-eigenen next_tx-Sequenz oder aus TxId::INTERNAL_BASE-Offset-Bereich beziehen.** **Aufrufer MÜSSEN sicherstellen, dass `tx`-Argumente für [`add_entity`], [`add_edge`] und [`commit`] ausschließlich aus einer der folgenden beiden kanonischen Quellen stammen:** 1. **Collection-eigene Sequenz**: Der `next_tx: Arc<AtomicU64>` Zähler in `contextr

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn traverse<'a>( &'a self, start_node: EntityId, max_hops: usize, ) -> BoxFuture<'a, Result<Vec<(EntityId, f32)>>>` | `crates/contextra-ports/src/graph_index.rs:61` | Traverses the entity graph using BFS up to a maximum number of hops. Distributes traversing decay weights across related entities. |
| `fn neighbors<'a>(&'a self, start_node: EntityId) -> BoxFuture<'a, Result<Vec<EntityId>>>` | `crates/contextra-ports/src/graph_index.rs:68` | Returns direct (1-hop) neighbor EntityIds for the given entity. |
| `fn remove_edge<'a>( &'a self, tx: TxId, from: EntityId, to: EntityId, ) -> BoxFuture<'a, Result<()>>` | `crates/contextra-ports/src/graph_index.rs:76` | Removes an edge between two entities. |
| `fn add_bidirectional<'a>( &'a self, tx: TxId, from: EntityId, to: EntityId, label: &'a str, ) -> BoxFuture<'a, Result<()>>` | `crates/contextra-ports/src/graph_index.rs:89` | Adds a bidirectional edge between two entities. |
| `fn multi_traverse<'a>( &'a self, start_nodes: &'a [EntityId], max_hops: usize, ) -> BoxFuture<'a, Result<Vec<(EntityId, f32)>>>` | `crates/contextra-ports/src/graph_index.rs:105` | Traverses the entity graph starting from multiple anchor entities up to max_hops. Aggregates decay weights (keeping max score per entity) across anchors. |
| `fn multi_traverse_at<'a>( &'a self, start_nodes: &'a [EntityId], max_hops: usize, seq_no: u64, ) -> BoxFuture<'a, Result<Vec<(EntityId, f32)>>>` | `crates/contextra-ports/src/graph_index.rs:129` | Traverses the entity graph starting from multiple anchor entities up to max_hops at a specific sequence number. Aggregates decay weights (keeping max score per  |
| `fn path_rag_at<'a>( &'a self, _start_nodes: &'a [EntityId], _max_hops: usize, _seq_no: u64, ) -> BoxFuture<'a, Result<Vec<(EntityId, f32)>>>` | `crates/contextra-ports/src/graph_index.rs:162` | Executes PathRAG graph traversal starting from multiple anchor entities up to `max_hops` at a specific sequence number (`seq_no`). # Contract - The result is th |
| `fn traverse_at<'a>( &'a self, _start_node: EntityId, _max_hops: usize, _seq_no: u64, ) -> BoxFuture<'a, Result<Vec<(EntityId, f32)>>>` | `crates/contextra-ports/src/graph_index.rs:183` | Traverses the entity graph using BFS up to a maximum number of hops at a specific sequence number. # Errors Returns [`ContextraError::CapabilityUnsupported`][cr |
| `fn personalized_page_rank_at<'a>( &'a self, _seed_nodes: &'a [EntityId], _config: &'a PprConfig, _seq_no: u64, ) -> BoxFuture<'a, Result<Vec<(EntityId, f32)>>>` | `crates/contextra-ports/src/graph_index.rs:203` | Calculates Personalized PageRank (PPR) starting from seed nodes on the graph state visible at a specific sequence number (`seq_no`). # Errors Returns [`Contextr |
| `fn traverse_at_time<'a>( &'a self, _start_node: EntityId, _max_hops: usize, _as_of: TxId, ) -> BoxFuture<'a, Result<Vec<(EntityId, f32)>>>` | `crates/contextra-ports/src/graph_index.rs:223` | Traverses the entity graph using BFS at a specific point in time (bi-temporal edge filtering). # Errors Returns [`ContextraError::CapabilityUnsupported`][crate: |
| `fn traverse_at_bitemporal<'a>( &'a self, _start_node: EntityId, _max_hops: usize, _as_of_tx: TxId, _as_of_business: Option<i64>, ) -> BoxFuture<'a, Result<Vec<(EntityId, f32)>>>` | `crates/contextra-ports/src/graph_index.rs:242` | Traverses the entity graph using BFS with independent system time and business time constraints. # Errors Returns [`ContextraError::CapabilityUnsupported`][crat |
| `fn personalized_page_rank<'a>( &'a self, _seed_nodes: &'a [EntityId], _config: &'a PprConfig, ) -> BoxFuture<'a, Result<Vec<(EntityId, f32)>>>` | `crates/contextra-ports/src/graph_index.rs:268` | Calculates Personalized PageRank (PPR) starting from seed nodes. # Convergence Behavior Power iteration terminates when the L1 norm difference between iteration |
| `fn add_entity<'a>(&'a self, tx: TxId, entity: Entity) -> BoxFuture<'a, Result<()>>` | `crates/contextra-ports/src/graph_index.rs:282` | Inserts or updates a node entity. |
| `fn add_edge<'a>(&'a self, tx: TxId, edge: Edge) -> BoxFuture<'a, Result<()>>` | `crates/contextra-ports/src/graph_index.rs:285` | Inserts or updates an edge between two entities. |
| `fn commit<'a>(&'a self, tx: TxId) -> BoxFuture<'a, Result<()>>` | `crates/contextra-ports/src/graph_index.rs:288` | Commits a transaction. |
| `fn rollback<'a>(&'a self, tx: TxId) -> BoxFuture<'a, Result<()>>` | `crates/contextra-ports/src/graph_index.rs:291` | Rolls back a transaction. |
| `fn rollback_to_tx<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>>` | `crates/contextra-ports/src/graph_index.rs:294` | Rolls back the entire graph state to a specific transaction ID. |
| `fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, Result<TxId>>` | `crates/contextra-ports/src/graph_index.rs:297` | Returns the last transaction ID processed by the index. |
| `fn len<'a>(&'a self) -> BoxFuture<'a, usize>` | `crates/contextra-ports/src/graph_index.rs:300` | Returns the number of entities in the index. |
| `fn is_empty<'a>(&'a self) -> BoxFuture<'a, bool>` | `crates/contextra-ports/src/graph_index.rs:303` | Returns true if the index is empty. |
| `fn stats<'a>(&'a self) -> BoxFuture<'a, Result<GraphIndexStats>>` | `crates/contextra-ports/src/graph_index.rs:308` | Collects statistics for the Graph. |

### Modul `index`

#### `mod prelude`
- Ort: `crates/contextra-ports/src/index.rs:9`
```rust
pub mod prelude
```
Dokumentation: Prelude for indexing ports.

### Modul `inference`

#### `mod prelude`
- Ort: `crates/contextra-ports/src/inference.rs:12`
```rust
pub mod prelude
```
Dokumentation: Prelude for inference ports.

### Modul `key_material`

#### `trait KeyMaterialSource`
- Ort: `crates/contextra-ports/src/key_material.rs:31`
```rust
pub trait KeyMaterialSource: Send + Sync + Debug + 'static
```
Dokumentation: Port-Trait für kryptografisch sichere Schlüssel- und Salt-Generierung. Implementierungen **müssen** fail-closed sein: Bei Entropieerschöpfung oder OS-Fehler wird ein Fehler zurückgegeben, nie ein unsicherer Fallback verwendet. # Trennung von `Rng` Dieser Trait ist bewusst von `RngPort` getrennt, da kryptografische Schlüsselgenerierung stärkere Sicherheitsanforderungen hat als allgemeiner Pseudo-Zufall (P28, ADR-098). # Verwendung - WAL-HMAC-Schlüsselerzeugung (`wal/hmac.rs`) - Database-Salt-Generierung bei `LsmStorage::open` (`lsm/recovery.rs`) - Deterministisches Testen via `StaticKeyMaterial

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn fill_key_bytes(&self, dest: &mut [u8]) -> Result<()>` | `crates/contextra-ports/src/key_material.rs:37` | Füllt `dest` mit kryptografisch sicheren Zufallsbytes. # Errors Gibt `ContextraError::Storage` zurück bei OS-Entropieerschöpfung oder Plattformfehler. Kein Fall |

### Modul `keys`

#### `struct Envelope`
- Ort: `crates/contextra-ports/src/keys.rs:104`
```rust
pub struct Envelope
```
Dokumentation: Sealed record envelope containing encrypted ciphertext and tag.
- Impl-Block `crates/contextra-ports/src/keys.rs:108`
- Impl-Block (Trait `fmt::Debug`) `crates/contextra-ports/src/keys.rs:120`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(payload: Vec<u8>) -> Self` | `crates/contextra-ports/src/keys.rs:110` | Creates a new envelope. |
| `pub fn payload(&self) -> &[u8]` | `crates/contextra-ports/src/keys.rs:115` | Returns payload bytes. |

#### `struct FileId`
- Ort: `crates/contextra-ports/src/keys.rs:12`
```rust
pub struct FileId(pub u64)
```
Dokumentation: Unique identifier for a durable storage file or SSTable.
- Impl-Block `crates/contextra-ports/src/keys.rs:14`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn new(id: u64) -> Self` | `crates/contextra-ports/src/keys.rs:16` | Creates a new `FileId`. |
| `pub const fn as_u64(&self) -> u64` | `crates/contextra-ports/src/keys.rs:21` | Returns the underlying numeric ID. |

#### `struct FileKey`
- Ort: `crates/contextra-ports/src/keys.rs:42`
```rust
pub struct FileKey
```
Dokumentation: Wrapper around a cryptographic file key with zeroizing on drop and masked debug display (PO-08).
- Impl-Block `crates/contextra-ports/src/keys.rs:47`
- Impl-Block (Trait `Drop`) `crates/contextra-ports/src/keys.rs:64`
- Impl-Block (Trait `fmt::Debug`) `crates/contextra-ports/src/keys.rs:71`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(raw: Vec<u8>, file_id: FileId) -> Self` | `crates/contextra-ports/src/keys.rs:49` | Creates a new `FileKey`. |
| `pub fn as_bytes(&self) -> &[u8]` | `crates/contextra-ports/src/keys.rs:54` | Returns a reference to the raw secret bytes. |
| `pub fn file_id(&self) -> FileId` | `crates/contextra-ports/src/keys.rs:59` | Associated file ID. |

#### `struct SealCtx`
- Ort: `crates/contextra-ports/src/keys.rs:84`
```rust
pub struct SealCtx
```
Dokumentation: Sealing context containing metadata and epoch tokens for envelope creation.
- Impl-Block `crates/contextra-ports/src/keys.rs:89`
- Impl-Block (Trait `fmt::Debug`) `crates/contextra-ports/src/keys.rs:96`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `epoch` | `u64` | `crates/contextra-ports/src/keys.rs:86` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn new(epoch: u64) -> Self` | `crates/contextra-ports/src/keys.rs:91` | Creates a new sealing context. |

#### `enum FileKind`
- Ort: `crates/contextra-ports/src/keys.rs:28`
```rust
pub enum FileKind
```
Dokumentation: Kind of file managed by [`FileKeyring`] (SPEC §8.2, D-85).

Varianten:

- `SSTable` — `crates/contextra-ports/src/keys.rs:30`
- `Wal` — `crates/contextra-ports/src/keys.rs:32`
- `TextSegment` — `crates/contextra-ports/src/keys.rs:34`
- `VectorSegment` — `crates/contextra-ports/src/keys.rs:36`
- `Auxiliary` — `crates/contextra-ports/src/keys.rs:38`

#### `trait FileKeyring`
- Ort: `crates/contextra-ports/src/keys.rs:177`
```rust
pub trait FileKeyring: Send + Sync
```
Dokumentation: File keyring interface managing per-file cryptographic keys (SPEC §4.1, §8.2, D-85, PO-06). # Contract (PO-06) `destroy` zeroes the key slot, executes `sync_data`, and verifies the key's non-presence via a read-check; failure leaves the slot in `Pending` requiring a retry upon reopen (D-55).

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn create(&self, kind: FileKind, id: FileId) -> BoxFuture<'_, Result<FileKey, String>>` | `crates/contextra-ports/src/keys.rs:185` | Allocates and records a new file key for the given kind and ID. # Fehler Liefert Fehler falls Slot-Erzeugung oder CSPRNG fehlschlägt. # Crash Persistiert Slot v |
| `fn open(&self, id: FileId) -> BoxFuture<'_, Result<FileKey, String>>` | `crates/contextra-ports/src/keys.rs:194` | Opens and retrieves the file key for the given ID. # Fehler Liefert Fehler falls Dateischlüssel fehlt oder vernichtet wurde. # Crash Reines Laden des Dateischlü |
| `fn destroy(&self, id: FileId) -> BoxFuture<'_, Result<(), String>>` | `crates/contextra-ports/src/keys.rs:203` | Securely destroys the file key slot (PO-06). # Fehler Liefert Fehler falls Slot-Nullung oder Sync fehlschlägt. # Crash Nullt Slot, führt sync_data aus und prüft |

#### `trait RecordSealer`
- Ort: `crates/contextra-ports/src/keys.rs:128`
```rust
pub trait RecordSealer: Send + Sync
```
Dokumentation: Record sealer contract for crypto envelope management (SPEC §4.2, D-53).

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn seal<'a>( &'a self, ctx: &'a SealCtx, key: &'a KeyBuf, plain: &'a [u8], ) -> BoxFuture<'a, Result<Envelope, String>>` | `crates/contextra-ports/src/keys.rs:136` | Encrypts and seals plaintext with slot allocation if missing. # Fehler Liefert Fehler falls Verschlüsselung oder Slot-Allokation fehlschlägt. # Crash Vorübergeh |
| `fn open<'a>(&'a self, key: &'a KeyBuf, env: &'a [u8]) -> BoxFuture<'a, Result<Vec<u8>, String>>` | `crates/contextra-ports/src/keys.rs:150` | Decrypts envelope. If slot was destroyed/missing, returns error indicating erasure. # Fehler Liefert Fehler falls Schlüssel vernichtet wurde oder Entschlüsselun |
| `fn evict(&self, scope: &KeyRange)` | `crates/contextra-ports/src/keys.rs:160` | Evicts key range from DEK cache, zeroizing RAM keys (D-79). # Fehler Synchrone Operation ohne Fehlerzustände. # Crash Entfernt und nullt Schlüsselmaterial flüch |
| `fn make_slots_durable(&self) -> BoxFuture<'_, Result<(), String>>` | `crates/contextra-ports/src/keys.rs:169` | Forces persistence of vault slots (`sync_data`) before WAL frame flush (I-9, L-9). # Fehler Liefert Fehler bei I/O- oder fsync-Fehlern. # Crash Erzwingt Slot-Da |

### Modul `kv`

#### `struct KvBlock`
- Ort: `crates/contextra-ports/src/kv.rs:126`
```rust
pub struct KvBlock
```
Dokumentation: Represents an exported KV block segment.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `block_id` | `u64` | `crates/contextra-ports/src/kv.rs:128` |
| `data` | `Bytes` | `crates/contextra-ports/src/kv.rs:134` |

#### `struct KvLayout`
- Ort: `crates/contextra-ports/src/kv.rs:21`
```rust
pub struct KvLayout
```
Dokumentation: KV cache layer and dimensions layout configuration.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `n_layer` | `u32` | `crates/contextra-ports/src/kv.rs:23` |
| `n_kv_head` | `u32` | `crates/contextra-ports/src/kv.rs:25` |
| `head_dim` | `u32` | `crates/contextra-ports/src/kv.rs:27` |
| `dtype` | `String` | `crates/contextra-ports/src/kv.rs:29` |

#### `struct KvPrefixHit`
- Ort: `crates/contextra-ports/src/kv.rs:139`
```rust
pub struct KvPrefixHit
```
Dokumentation: Represents a prefix match hit in the KV store.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `matched_tokens` | `usize` | `crates/contextra-ports/src/kv.rs:141` |
| `blocks` | `Vec<KvBlock>` | `crates/contextra-ports/src/kv.rs:143` |

#### `struct PrefixKey`
- Ort: `crates/contextra-ports/src/kv.rs:55`
```rust
pub struct PrefixKey
```
Dokumentation: Key identifying a model KV cache prefix structure.
- Impl-Block (Trait `std::hash::Hash`) `crates/contextra-ports/src/kv.rs:66`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `model` | `ModelFingerprint` | `crates/contextra-ports/src/kv.rs:57` |
| `tokenizer_hash` | `[u8; 32]` | `crates/contextra-ports/src/kv.rs:59` |
| `layout` | `KvLayout` | `crates/contextra-ports/src/kv.rs:61` |
| `rope` | `RopeConfig` | `crates/contextra-ports/src/kv.rs:63` |

#### `struct RopeConfig`
- Ort: `crates/contextra-ports/src/kv.rs:34`
```rust
pub struct RopeConfig
```
Dokumentation: Rotary position embedding (RoPE) configuration.
- Impl-Block (Trait `Eq`) `crates/contextra-ports/src/kv.rs:41`
- Impl-Block (Trait `std::hash::Hash`) `crates/contextra-ports/src/kv.rs:43`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `base` | `f32` | `crates/contextra-ports/src/kv.rs:36` |
| `scaling` | `Option<f32>` | `crates/contextra-ports/src/kv.rs:38` |

#### `trait KvPrefixStore`
- Ort: `crates/contextra-ports/src/kv.rs:151`
```rust
pub trait KvPrefixStore: Send + Sync + 'static
```
Dokumentation: Trait defining the contract for KV prefix caching and block reuse (Spec §9.2). # Dyn-Kompatibilität Dieser Trait ist durch synchrone Schnittstellen vtable-kompatibel (dyn-safe).

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn lookup(&self, tenant: TenantId, key: &PrefixKey, tokens: &[u32]) -> Option<KvPrefixHit>` | `crates/contextra-ports/src/kv.rs:158` | Returns the longest exact matching prefix in blocks for the prompt tokens, if any. # Fehler Keine; liefert `None` falls kein übereinstimmendes Präfix gefunden w |
| `fn insert( &self, tenant: TenantId, key: &PrefixKey, tokens: &[u32], blocks: Vec<KvBlock>, ) -> Result<(), ContextraError>` | `crates/contextra-ports/src/kv.rs:161` | Inserts KV blocks for a given token prefix into the store. |
| `fn evict(&self, tenant: TenantId, key: &PrefixKey) -> Result<u64, ContextraError>` | `crates/contextra-ports/src/kv.rs:176` | Evicts KV prefix entries for the given tenant and prefix key. Returns the number of blocks evicted. # Fehler Liefert `ContextraError` bei I/O- oder Freigabefehl |

### Modul `kv_bridge_port`

#### `trait KvBridgeStorage`
- Ort: `crates/contextra-ports/src/kv_bridge_port.rs:14`
```rust
pub trait KvBridgeStorage: Send + Sync
```
Dokumentation: Abstrahiertes Persistence-Interface für KV-Bridge Tier-2 Spill-over operations. Implementierbar von Ring-1 Storage Engines (`contextra-store::LsmStorage`) oder Test-Mocks.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn get<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, Result<Option<Bytes>>>` | `crates/contextra-ports/src/kv_bridge_port.rs:16` | Versucht, einen Wert unter dem angegebenen Schlüssel abzufragen. |
| `fn put<'a>(&'a self, tx_id: TxId, key: &'a [u8], value: &'a [u8]) -> BoxFuture<'a, Result<()>>` | `crates/contextra-ports/src/kv_bridge_port.rs:19` | Schreibt einen Schlüssel-Wert-Paar im Kontext einer Transaktion (`tx_id`). |
| `fn commit<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>>` | `crates/contextra-ports/src/kv_bridge_port.rs:22` | Committet die Transaktion mit der ID `tx_id`. |

### Modul `ledger`

#### `struct LedgerEntry`
- Ort: `crates/contextra-ports/src/ledger.rs:59`
```rust
pub struct LedgerEntry
```
Dokumentation: A tamper-evident entry in the erasure ledger (SPEC §4.2, §11.8, F.9). Contains `(commitment, range_set, gated_seq, completed_at)` along with `chain_hash` and `ledger_mac`.
- Impl-Block `crates/contextra-ports/src/ledger.rs:74`
- Impl-Block (Trait `fmt::Debug`) `crates/contextra-ports/src/ledger.rs:140`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `commitment` | `[u8; 32]` | `crates/contextra-ports/src/ledger.rs:61` |
| `range_set` | `RangeSet` | `crates/contextra-ports/src/ledger.rs:63` |
| `gated_seq` | `Seq` | `crates/contextra-ports/src/ledger.rs:65` |
| `completed_at` | `Timestamp` | `crates/contextra-ports/src/ledger.rs:67` |
| `chain_hash` | `[u8; 32]` | `crates/contextra-ports/src/ledger.rs:69` |
| `ledger_mac` | `[u8; 32]` | `crates/contextra-ports/src/ledger.rs:71` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn new( commitment: [u8` | `crates/contextra-ports/src/ledger.rs:76` | Creates a new ledger entry with explicit fields. |
| `pub fn compute_chained( commitment: [u8` | `crates/contextra-ports/src/ledger.rs:95` | Constructs a ledger entry and computes its chained BLAKE3 hash from the prior entry hash. |
| `pub fn verify_chain_hash(&self, prev_hash: &[u8` | `crates/contextra-ports/src/ledger.rs:125` | Verifies the entry's chain hash against the supplied previous block hash. |

#### `struct RangeSet`
- Ort: `crates/contextra-ports/src/ledger.rs:27`
```rust
pub struct RangeSet
```
Dokumentation: Set of key ranges or tokens included in an erasure ledger entry.
- Impl-Block `crates/contextra-ports/src/ledger.rs:31`
- Impl-Block (Trait `fmt::Debug`) `crates/contextra-ports/src/ledger.rs:48`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn empty() -> Self` | `crates/contextra-ports/src/ledger.rs:33` | Creates an empty range set. |
| `pub fn new(ranges: Vec<(KeyBuf, KeyBuf)>) -> Self` | `crates/contextra-ports/src/ledger.rs:38` | Creates a range set from intervals. |
| `pub fn ranges(&self) -> &[(KeyBuf, KeyBuf)]` | `crates/contextra-ports/src/ledger.rs:43` | Returns the slice of ranges. |

#### `struct Timestamp`
- Ort: `crates/contextra-ports/src/ledger.rs:11`
```rust
pub struct Timestamp(pub u64)
```
Dokumentation: Discrete point in time measured in epoch milliseconds or nanoseconds.
- Impl-Block `crates/contextra-ports/src/ledger.rs:13`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn new(val: u64) -> Self` | `crates/contextra-ports/src/ledger.rs:15` | Creates a new timestamp. |
| `pub const fn as_u64(&self) -> u64` | `crates/contextra-ports/src/ledger.rs:20` | Returns the underlying numeric value. |

#### `trait LedgerSink`
- Ort: `crates/contextra-ports/src/ledger.rs:152`
```rust
pub trait LedgerSink: Send + Sync
```
Dokumentation: Durable audit ledger sink located outside the primary data directory (SPEC §4.2, §11.8).

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn append(&self, e: &LedgerEntry) -> Result<(), String>` | `crates/contextra-ports/src/ledger.rs:160` | Appends a new verified erasure entry to the ledger. # Fehler Liefert Fehler bei Schreib- oder MAC-Validierungsfehlern. # Crash Schreibt dauerhaft in das Ledger- |
| `fn read_all(&self) -> Result<Vec<LedgerEntry>, String>` | `crates/contextra-ports/src/ledger.rs:169` | Reads all entries recorded in the ledger for verification or auditing. # Fehler Liefert Fehler bei I/O- oder Integritätsbrüchen der Hashkette. # Crash Reiner Le |
| `fn prune(&self, older_than: Timestamp) -> Result<usize, String>` | `crates/contextra-ports/src/ledger.rs:178` | Prunes entries older than the specified retention threshold. # Fehler Liefert Fehler bei I/O-Fehlern beim Kürzen. # Crash Kürzt veraltete Ledger-Einträge atomar |

### Modul `license`

#### `struct AuthorizedRing`
- Ort: `crates/contextra-ports/src/license.rs:60`
```rust
pub struct AuthorizedRing
```
- Impl-Block `crates/contextra-ports/src/license.rs:64`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn ring(&self) -> FeatureRing` | `crates/contextra-ports/src/license.rs:71` | Returns the authorized [`FeatureRing`]. |

#### `struct OpenFastGate`
- Ort: `crates/contextra-ports/src/license.rs:131`
```rust
pub struct OpenFastGate
```
- Impl-Block (Trait `LicenseGate`) `crates/contextra-ports/src/license.rs:133`

#### `enum FeatureRing`
- Ort: `crates/contextra-ports/src/license.rs:41`
```rust
pub enum FeatureRing
```

Varianten:

- `Fast` — `crates/contextra-ports/src/license.rs:43`
- `Sovereign` — `crates/contextra-ports/src/license.rs:45`
- `Compliance` — `crates/contextra-ports/src/license.rs:47`

#### `enum LicenseError`
- Ort: `crates/contextra-ports/src/license.rs:83`
```rust
pub enum LicenseError
```

Varianten:

- `NotActivated(FeatureRing)` — `crates/contextra-ports/src/license.rs:86`
- `InvalidSignature` — `crates/contextra-ports/src/license.rs:89`
- `Expired(i64)` — `crates/contextra-ports/src/license.rs:92`

#### `enum VectorDeleteMode`
- Ort: `crates/contextra-ports/src/license.rs:21`
```rust
pub enum VectorDeleteMode
```
- Impl-Block (Trait `Default`) `crates/contextra-ports/src/license.rs:28`

Varianten:

- `SynchronousRepair` — `crates/contextra-ports/src/license.rs:23`
- `BackgroundRepair` — `crates/contextra-ports/src/license.rs:25`

#### `trait LicenseGate`
- Ort: `crates/contextra-ports/src/license.rs:103`
```rust
pub trait LicenseGate: Send + Sync
```

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn check_ring(&self, ring: FeatureRing) -> Result<(), LicenseError>` | `crates/contextra-ports/src/license.rs:110` | Checks if the requested feature ring is activated and accessible. # Fehler Liefert `LicenseError::NotActivated`, `LicenseError::InvalidSignature` oder `LicenseE |
| `fn authorize(&self, requested: FeatureRing) -> Result<AuthorizedRing, LicenseError>` | `crates/contextra-ports/src/license.rs:118` | Authorizes the requested feature ring and yields an [`AuthorizedRing`] token on success. # Fehler Liefert `LicenseError` falls die Autorisierungsprüfung fehlsch |

### Modul `lifecycle`

#### `struct GroundingAssessment`
- Ort: `crates/contextra-ports/src/lifecycle.rs:21`
```rust
pub struct GroundingAssessment
```
Dokumentation: Result of a post-hoc grounding / attribution validation check.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `score` | `f32` | `crates/contextra-ports/src/lifecycle.rs:23` |
| `is_grounded` | `bool` | `crates/contextra-ports/src/lifecycle.rs:25` |
| `reason` | `Option<String>` | `crates/contextra-ports/src/lifecycle.rs:27` |

#### `trait ContextPreparer`
- Ort: `crates/contextra-ports/src/lifecycle.rs:60`
```rust
pub trait ContextPreparer: Send + Sync
```
Dokumentation: Contract for trimming and preparing context windows tailored to token budgets.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn prepare_context( &self, chunks: Vec<ContextChunk>, budget: &TokenBudget, relevance_threshold: f32, ) -> Result<ContextWindow>` | `crates/contextra-ports/src/lifecycle.rs:62` | Prepares and trims context chunks according to the provided token budget and relevance threshold. |

#### `trait GroundingValidator`
- Ort: `crates/contextra-ports/src/lifecycle.rs:32`
```rust
pub trait GroundingValidator: Send + Sync
```
Dokumentation: Abstract contract for post-hoc hallucination / grounding validation.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn validate_grounding<'a>( &'a self, response: &'a str, context_chunks: &'a [ContextChunk], ) -> BoxFuture<'a, Result<GroundingAssessment>>` | `crates/contextra-ports/src/lifecycle.rs:38` | Validates an LLM-generated response against retrieval context chunks. Returns `Ok(GroundingAssessment)` if confidence meets threshold, or `Err(ContextraError)`  |

#### `trait ResponseGroundingValidator`
- Ort: `crates/contextra-ports/src/lifecycle.rs:47`
```rust
pub trait ResponseGroundingValidator: Send + Sync
```
Dokumentation: Abstract contract for grounding validation of LLM-generated responses against raw source strings.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn score_grounding(&self, response: &str, sources: &[&str]) -> Result<f32>` | `crates/contextra-ports/src/lifecycle.rs:55` | Evaluates the grounding score for an LLM-generated response against source text slices. Returns a float score in [0.0, 1.0]. # Fehler Liefert `ContextraError` b |

### Modul `observability`

#### `trait DriftStatusProvider`
- Ort: `crates/contextra-ports/src/observability.rs:10`
```rust
pub trait DriftStatusProvider: Send + Sync
```
Dokumentation: Trait for querying Lyapunov drift status from an attached router engine without creating a cyclic dependency (ADR-080).

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn overall_drift_status(&self) -> String` | `crates/contextra-ports/src/observability.rs:17` | Returns the overall drift status string ("stabil", "warnung", "kritisch", or "unbekannt"). # Fehler Keine; statischer oder synchron berechneter Statusabruf. # C |

### Modul `observer`

#### `struct ErasureEvent`
- Ort: `crates/contextra-ports/src/observer.rs:49`
```rust
pub struct ErasureEvent
```
Dokumentation: Event published to an [`ErasureObserver`] upon reaching an erasure lifecycle phase (SPEC D-118).

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `id` | `ErasureId` | `crates/contextra-ports/src/observer.rs:51` |
| `target_kind` | `TargetKind` | `crates/contextra-ports/src/observer.rs:53` |
| `phase` | `ObservedPhase` | `crates/contextra-ports/src/observer.rs:55` |
| `at` | `Timestamp` | `crates/contextra-ports/src/observer.rs:57` |

#### `enum ObservedPhase`
- Ort: `crates/contextra-ports/src/observer.rs:36`
```rust
pub enum ObservedPhase
```
Dokumentation: Observed lifecycle phase of an erasure operation (SPEC D-118).
- Impl-Block (Trait `fmt::Display`) `crates/contextra-ports/src/observer.rs:117`

Varianten:

- `Gated` — `crates/contextra-ports/src/observer.rs:38`
- `Completed` — `crates/contextra-ports/src/observer.rs:40`
- `Restricted` — `crates/contextra-ports/src/observer.rs:42`
- `Lifted` — `crates/contextra-ports/src/observer.rs:44`

#### `enum ObserverError`
- Ort: `crates/contextra-ports/src/observer.rs:62`
```rust
pub enum ObserverError
```
Dokumentation: Error returned by an [`ErasureObserver`] during notification delivery (SPEC D-118).

Varianten:

- `Transient(String)` — `crates/contextra-ports/src/observer.rs:65`
- `Permanent(String)` — `crates/contextra-ports/src/observer.rs:69`
- `Timeout` — `crates/contextra-ports/src/observer.rs:73`

#### `enum TargetKind`
- Ort: `crates/contextra-ports/src/observer.rs:19`
```rust
pub enum TargetKind
```
Dokumentation: Target kind indicating which hierarchy level was targeted (SPEC D-118, P13).
- Impl-Block (Trait `fmt::Display`) `crates/contextra-ports/src/observer.rs:104`

Varianten:

- `Document` — `crates/contextra-ports/src/observer.rs:21`
- `Subject` — `crates/contextra-ports/src/observer.rs:23`
- `Collection` — `crates/contextra-ports/src/observer.rs:25`
- `Tenant` — `crates/contextra-ports/src/observer.rs:27`
- `DocumentReplace` — `crates/contextra-ports/src/observer.rs:29`
- `Superseded` — `crates/contextra-ports/src/observer.rs:31`

#### `trait ErasureObserver`
- Ort: `crates/contextra-ports/src/observer.rs:79`
```rust
pub trait ErasureObserver: Send + Sync + 'static
```
Dokumentation: Port trait for asynchronous delivery of erasure notifications (SPEC §3.1, §8.2, D-118). Implementations must be object-safe and dyn-compatible as `Arc<dyn ErasureObserver>`.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn on_erased( &self, ev: ErasureEvent, ) -> BoxFuture<'static, std::result::Result<(), ObserverError>>` | `crates/contextra-ports/src/observer.rs:89` | Notifies the observer about an erasure event. # Fehler Liefert [`ObserverError`], wenn die Benachrichtigung scheitert. Die aufrufende Hintergrundaufgabe führt W |

#### `type Timestamp`
- Ort: `crates/contextra-ports/src/observer.rs:15`
```rust
pub type Timestamp = u64
```
Dokumentation: Type representing a millisecond or microsecond timestamp (SPEC D-118).

### Modul `plugin`

#### `struct PluginCapability`
- Ort: `crates/contextra-ports/src/plugin.rs:11`
```rust
pub struct PluginCapability
```
Dokumentation: Capability descriptor for a plugin.
- Impl-Block `crates/contextra-ports/src/plugin.rs:22`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `name` | `&'static str` | `crates/contextra-ports/src/plugin.rs:13` |
| `ring` | `u8` | `crates/contextra-ports/src/plugin.rs:17` |
| `feature_ring_required` | `FeatureRing` | `crates/contextra-ports/src/plugin.rs:19` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new( name: &'static str, version: (u32, u32, u32), ring: u8, feature_ring_required: FeatureRing, ) -> Self` | `crates/contextra-ports/src/plugin.rs:24` | Creates a new `PluginCapability`. |

#### `struct PluginRegistry`
- Ort: `crates/contextra-ports/src/plugin.rs:119`
```rust
pub struct PluginRegistry
```
Dokumentation: Registry managing plugin activations, license gating, and dependency resolution.
- Impl-Block `crates/contextra-ports/src/plugin.rs:125`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(gate: Arc<dyn LicenseGate>) -> Self` | `crates/contextra-ports/src/plugin.rs:127` | Creates a new `PluginRegistry` bounded by the given `LicenseGate`. |
| `pub fn is_active(&self, name: &str) -> bool` | `crates/contextra-ports/src/plugin.rs:136` | Checks whether a plugin with the given name is currently active. |
| `pub fn snapshot(&self) -> Vec<PluginCapability>` | `crates/contextra-ports/src/plugin.rs:141` | Takes a snapshot of all active plugin capabilities. |
| `pub fn current_feature_ring(&self) -> FeatureRing` | `crates/contextra-ports/src/plugin.rs:146` | Returns the highest currently active `FeatureRing` according to the license gate. |
| `pub fn activate_all( &mut self, plugins: Vec<Box<dyn PluginManifest>>, ) -> Result<(), PluginError>` | `crates/contextra-ports/src/plugin.rs:163` | Activates a batch of plugins in an all-or-nothing manner (INV-PLUGIN-DEPENDENCY). Sequence: 1. Conflict checks across requested and active plugins. 2. License g |

#### `enum PluginError`
- Ort: `crates/contextra-ports/src/plugin.rs:98`
```rust
pub enum PluginError
```
Dokumentation: Errors during plugin registration, activation, or dependency resolution.

Varianten:

- `DependencyCycle(Vec<String>)` — `crates/contextra-ports/src/plugin.rs:101`
- `Conflict(String, String)` — `crates/contextra-ports/src/plugin.rs:105`
- `LicenseDenied(String)` — `crates/contextra-ports/src/plugin.rs:109`

#### `trait PluginManifest`
- Ort: `crates/contextra-ports/src/plugin.rs:40`
```rust
pub trait PluginManifest: Send + Sync
```
Dokumentation: Manifest trait implemented by dynamic or static plugins.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn name(&self) -> &str` | `crates/contextra-ports/src/plugin.rs:47` | Returns the name of the plugin. # Fehler Keine; statische Namensabfrage. # Crash Reiner Statusabruf ohne Mutation. |
| `fn requires(&self) -> &[&str]` | `crates/contextra-ports/src/plugin.rs:55` | Returns the list of plugin names required prior to activation. # Fehler Keine; statische Referenzliste. # Crash Reiner Statusabruf ohne Mutation. |
| `fn conflicts(&self) -> &[&str]` | `crates/contextra-ports/src/plugin.rs:65` | Returns the list of plugin names that conflict with this plugin. # Fehler Keine; statische Referenzliste. # Crash Reiner Statusabruf ohne Mutation. |
| `fn feature_ring_required(&self) -> FeatureRing` | `crates/contextra-ports/src/plugin.rs:75` | Returns the required feature ring for this plugin. # Fehler Keine; statischer Feature-Ring-Abruf. # Crash Reiner Statusabruf ohne Mutation. |
| `fn capability(&self) -> PluginCapability` | `crates/contextra-ports/src/plugin.rs:85` | Returns the static capability metadata for this plugin. # Fehler Keine; liefert statischen Deskriptor. # Crash Reiner Statusabruf ohne Mutation. |
| `fn activate(&self) -> Result<(), PluginError>` | `crates/contextra-ports/src/plugin.rs:93` | Activates the plugin instance. # Fehler Liefert `PluginError` bei Initialisierungs-, Berechtigungs- oder Ressourcenfehlern. # Crash Idempotente Aktivierung im R |

### Modul `policy`

#### `mod prelude`
- Ort: `crates/contextra-ports/src/policy.rs:40`
```rust
pub mod prelude
```
Dokumentation: Prelude for policy ports.

#### `enum EgressDecision`
- Ort: `crates/contextra-ports/src/policy.rs:17`
```rust
pub enum EgressDecision
```
Dokumentation: Egress evaluation decision.

Varianten:

- `Allow` — `crates/contextra-ports/src/policy.rs:19`
- `Redact(String)` — `crates/contextra-ports/src/policy.rs:21`
- `Deny(String)` — `crates/contextra-ports/src/policy.rs:23`

#### `trait EgressPolicy`
- Ort: `crates/contextra-ports/src/policy.rs:28`
```rust
pub trait EgressPolicy: Send + Sync
```
Dokumentation: Dynamic port trait for evaluating data egress policies.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn evaluate(&self, ctx: &contextra_types::RequestContext, payload: &str) -> EgressDecision` | `crates/contextra-ports/src/policy.rs:35` | Evaluates if the outgoing content complies with data governance and egress rules. # Fehler Keine; fehlerfrei über `EgressDecision` abgebildet. # Crash Reiner Re |

### Modul `query_rewriter`

#### `trait QueryRewriter`
- Ort: `crates/contextra-ports/src/query_rewriter.rs:7`
```rust
pub trait QueryRewriter: Send + Sync
```
Dokumentation: Trait for query rewriting in multi-step iterative retrieval (LLM-agnostic).

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn rewrite<'a>( &'a self, original_query: &'a str, current_results: &'a [ScoredEntry], ) -> BoxFuture<'a, Result<Vec<String>>>` | `crates/contextra-ports/src/query_rewriter.rs:11` | Generates alternative text sub-queries based on original query and previous search results. For backward compatibility, default implementation calls `rewrite_st |
| `fn rewrite_structured<'a>( &'a self, original_query: &'a str, current_results: &'a [ScoredEntry], ) -> BoxFuture<'a, Result<Vec<QueryRewriteOutput>>>` | `crates/contextra-ports/src/query_rewriter.rs:32` | Generates multi-signal query reformulations (text, semantic, and graph anchors) for multi-step retrieval. Default implementation delegates to `rewrite` and wrap |

### Modul `report_signer`

#### `struct ReportSignature`
- Ort: `crates/contextra-ports/src/report_signer.rs:19`
```rust
pub struct ReportSignature
```
Dokumentation: Digital signature over canonical report bytes (SPEC F.10).
- Impl-Block (Trait `fmt::Display`) `crates/contextra-ports/src/report_signer.rs:83`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `signer_id` | `SignerId` | `crates/contextra-ports/src/report_signer.rs:21` |
| `signature_base64` | `String` | `crates/contextra-ports/src/report_signer.rs:23` |
| `pub_fingerprint` | `String` | `crates/contextra-ports/src/report_signer.rs:25` |

#### `enum SignerError`
- Ort: `crates/contextra-ports/src/report_signer.rs:30`
```rust
pub enum SignerError
```
Dokumentation: Error class for report signing operations (SPEC D-130, 7.5).

Varianten:

- `KeyUnavailable(String)` — `crates/contextra-ports/src/report_signer.rs:33`
- `ExternalSignerFailed(String)` — `crates/contextra-ports/src/report_signer.rs:37`
- `InvalidPayload(String)` — `crates/contextra-ports/src/report_signer.rs:41`

#### `trait ReportSigner`
- Ort: `crates/contextra-ports/src/report_signer.rs:45`
```rust
pub trait ReportSigner: Send + Sync
```
Dokumentation: Abstract port trait for signing audit reports (SPEC §8.2, D-130, T-SIG-1).

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn signer_id(&self) -> SignerId` | `crates/contextra-ports/src/report_signer.rs:52` | Returns the unique numeric signer identifier according to SPEC F.10. # Fehler Keine; statische Signer-ID-Abfrage. # Crash Reiner Statusabruf ohne persistente Mu |
| `fn sign_report<'a>( &'a self, canonical_report_bytes: &'a [u8], ) -> BoxFuture<'a, std::result::Result<ReportSignature, SignerError>>` | `crates/contextra-ports/src/report_signer.rs:64` | Signs the canonical report bytes. # Parameter - `canonical_report_bytes`: canonical JSON representation according to SPEC F.10. # Fehler Liefert [`SignerError`] |

#### `type SignerId`
- Ort: `crates/contextra-ports/src/report_signer.rs:15`
```rust
pub type SignerId = u8
```
Dokumentation: Signer ID per SPEC F.10 (0 = internal/derived K_sign, 1 = operator signer).

### Modul `reranker`

#### `trait Reranker`
- Ort: `crates/contextra-ports/src/reranker.rs:7`
```rust
pub trait Reranker: Send + Sync
```
Dokumentation: Trait for post-retrieval cross-encoder reranking models.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn rerank<'a>( &'a self, query: &'a str, candidates: &'a [String], ) -> BoxFuture<'a, Result<Vec<RerankResult>>>` | `crates/contextra-ports/src/reranker.rs:9` | Reranks candidates for a given query text. |
| `fn record_implicit_feedback(&self, _results: &[RerankResult], _implicit_relevant_k: usize)` | `crates/contextra-ports/src/reranker.rs:21` | Records implicit feedback or outcome labels for calibration. # Fehler Keine; panikfrei und nicht-blockierend. # Crash Reiner flüchtiger RAM-Zustand oder No-Op;  |
| `fn rerank_deadline_ms(&self) -> Option<u64>` | `crates/contextra-ports/src/reranker.rs:29` | Returns the maximum allowed execution time in milliseconds before timing out, if configured. # Fehler Keine; statische Deadline-Abfrage. # Crash Reiner Statusab |

### Modul `scanner`

#### `mod layer`
- Ort: `crates/contextra-ports/src/scanner/mod.rs:16`
```rust
pub mod layer
```
Dokumentation: Layer and technique descriptor types.

#### `mod session`
- Ort: `crates/contextra-ports/src/scanner/mod.rs:18`
```rust
pub mod session
```
Dokumentation: ScanSession state and finish gate.

#### `mod witness`
- Ort: `crates/contextra-ports/src/scanner/mod.rs:20`
```rust
pub mod witness
```
Dokumentation: LayerWitness verified measurement type.

#### `trait LayerScanner`
- Ort: `crates/contextra-ports/src/scanner/mod.rs:60`
```rust
pub trait LayerScanner: Send + Sync
```
Dokumentation: Trait für Storage-Scanner zur Durchführung von Restmengen-Messungen. # Doctests (Compile Fail Checks) (a) `LayerWitness` lässt sich außerhalb des Moduls nicht direkt konstruieren: ```compile_fail use contextra_ports::scanner::{LayerWitness, WitnessLayer, ScanTechnique}; let _w = LayerWitness { layer: WitnessLayer::LsmMemtable, scanned: 0, remaining: 0, technique: ScanTechnique::Structural, digest: [0u8; 32], measured_at: 0, }; ``` (b) `ScanSession::finish` ist von außen nicht aufrufbar: ```compile_fail use contextra_ports::scanner::ScanSession; fn try_finish(session: ScanSession, clock: &dyn c

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn layer(&self) -> WitnessLayer` | `crates/contextra-ports/src/scanner/mod.rs:67` | Gibt die von diesem Scanner bediente Schicht zurück. # Fehler Keine; statische Schichtabfrage. # Crash Reiner Statusabruf ohne Mutation. |
| `fn scan_units(&self, session: &mut ScanSession, ctx: &crate::RequestContext) -> Result<()>` | `crates/contextra-ports/src/scanner/mod.rs:75` | Führt das Scannen von Einheiten auf der übergebenen `ScanSession` aus. # Fehler Liefert `ContextraError` bei I/O- oder Verifizierungsfehlern. # Crash Reiner Les |
| `fn scan( &self, ctx: &contextra_types::RequestContext, target: ScanTarget, technique: ScanTechnique, clock: &dyn Clock, ) -> Result<LayerWitness>` | `crates/contextra-ports/src/scanner/mod.rs:78` | Default-Implementierung von `scan`, die `begin`, `scan_units` und `finish` ausführt. |

### Modul `scanner::layer`

#### `struct PurgeStats`
- Ort: `crates/contextra-ports/src/scanner/layer.rs:107`
```rust
pub struct PurgeStats
```
Dokumentation: Statistiken über bereinigte Einheiten.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `purged_units` | `u64` | `crates/contextra-ports/src/scanner/layer.rs:109` |
| `bytes_freed` | `u64` | `crates/contextra-ports/src/scanner/layer.rs:111` |

#### `struct ScanTarget`
- Ort: `crates/contextra-ports/src/scanner/layer.rs:88`
```rust
pub struct ScanTarget
```
Dokumentation: Zielbeschreibung des Scans inklusive Hash.
- Impl-Block `crates/contextra-ports/src/scanner/layer.rs:95`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `target_id` | `String` | `crates/contextra-ports/src/scanner/layer.rs:90` |
| `target_hash` | `[u8; 32]` | `crates/contextra-ports/src/scanner/layer.rs:92` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(target_id: impl Into<String>, target_hash: [u8` | `crates/contextra-ports/src/scanner/layer.rs:97` | Erzeugt ein neues ScanTarget. |

#### `enum ScanTechnique`
- Ort: `crates/contextra-ports/src/scanner/layer.rs:71`
```rust
pub enum ScanTechnique
```
Dokumentation: Angewandte Methode/Technik während der Verifikation.

Varianten:

- `Structural` — `crates/contextra-ports/src/scanner/layer.rs:73`
- `DecryptedView` — `crates/contextra-ports/src/scanner/layer.rs:75`
- `IndexIntrospection` — `crates/contextra-ports/src/scanner/layer.rs:77`
- `NeedleScan` — `crates/contextra-ports/src/scanner/layer.rs:79`
- `NotMeasured(String)` — `crates/contextra-ports/src/scanner/layer.rs:81`
- `FeatureDisabled` — `crates/contextra-ports/src/scanner/layer.rs:83`

#### `enum WitnessLayer`
- Ort: `crates/contextra-ports/src/scanner/layer.rs:13`
```rust
pub enum WitnessLayer
```
Dokumentation: Identifikator für die physische Storage-Schicht.
- Impl-Block `crates/contextra-ports/src/scanner/layer.rs:32`
- Impl-Block (Trait `From<&str>`) `crates/contextra-ports/src/scanner/layer.rs:48`
- Impl-Block (Trait `From<String>`) `crates/contextra-ports/src/scanner/layer.rs:63`

Varianten:

- `LsmMemtable` — `crates/contextra-ports/src/scanner/layer.rs:15`
- `SsTableAllLevels` — `crates/contextra-ports/src/scanner/layer.rs:17`
- `HnswIndex` — `crates/contextra-ports/src/scanner/layer.rs:19`
- `WalAllSegments` — `crates/contextra-ports/src/scanner/layer.rs:21`
- `CsrGraph` — `crates/contextra-ports/src/scanner/layer.rs:23`
- `KvCacheSegments` — `crates/contextra-ports/src/scanner/layer.rs:25`
- `EmbeddingCache` — `crates/contextra-ports/src/scanner/layer.rs:27`
- `Custom(String)` — `crates/contextra-ports/src/scanner/layer.rs:29`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn as_str(&self) -> &str` | `crates/contextra-ports/src/scanner/layer.rs:34` | Gibt den String-Namen der Schicht zurück. |

#### `trait Purger`
- Ort: `crates/contextra-ports/src/scanner/layer.rs:115`
```rust
pub trait Purger: Send + Sync
```
Dokumentation: Trait für physische Bereinigungsaktionen (Purger) einer Schicht.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn purge(&self, ctx: &crate::RequestContext, target: &ScanTarget) -> Result<PurgeStats>` | `crates/contextra-ports/src/scanner/layer.rs:122` | Führt die physische Bereinigung für das gegebene Ziel aus. # Fehler Liefert `ContextraError` bei I/O- oder Freigabefehlern während der physischen Bereinigung. # |

### Modul `scanner::session`

#### `struct ScanSession`
- Ort: `crates/contextra-ports/src/scanner/session.rs:15`
```rust
pub struct ScanSession
```
Dokumentation: Laufende Scan-Session zur kontrollierten Erzeugung eines `LayerWitness`.
- Impl-Block `crates/contextra-ports/src/scanner/session.rs:25`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn record_unit(&mut self, unit_hash: [u8` | `crates/contextra-ports/src/scanner/session.rs:40` | Registriert eine gescannte Einheit. |
| `pub fn record_control_match(&mut self)` | `crates/contextra-ports/src/scanner/session.rs:49` | Registriert eine bestandene Positivkontrolle (Control Match). |

### Modul `scanner::witness`

#### `struct LayerWitness`
- Ort: `crates/contextra-ports/src/scanner/witness.rs:17`
```rust
pub struct LayerWitness
```
Dokumentation: Verifiziertes Messergebnis eines Storage-Scanners (SPEC §12.7.4). Unter INV-WITNESS-MINT-1 kann ein `LayerWitness` ausschließlich in `ScanSession::finish` entstehen. Ein direkter Konstruktor oder `Default` ist nicht vorhanden.
- Impl-Block `crates/contextra-ports/src/scanner/witness.rs:27`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn layer(&self) -> &str` | `crates/contextra-ports/src/scanner/witness.rs:57` | Gibt die gescannte Storage-Schicht als &str zurück. |
| `pub fn witness_layer(&self) -> &WitnessLayer` | `crates/contextra-ports/src/scanner/witness.rs:62` | Gibt die gescannte Storage-Schicht als &WitnessLayer zurück. |
| `pub fn scanned(&self) -> u64` | `crates/contextra-ports/src/scanner/witness.rs:67` | Gibt die Anzahl der während der Verifikation gescannten Einheiten zurück. |
| `pub fn remaining(&self) -> u64` | `crates/contextra-ports/src/scanner/witness.rs:72` | Gibt die Anzahl der verbleibenden Lebendeinträge zurück. |
| `pub fn technique(&self) -> &ScanTechnique` | `crates/contextra-ports/src/scanner/witness.rs:77` | Gibt die angewandte Scantechnik zurück. |
| `pub fn method(&self) -> &ScanMethod` | `crates/contextra-ports/src/scanner/witness.rs:82` | Gibt die ScanMethod-Entsprechung zurück. |
| `pub fn digest(&self) -> &[u8` | `crates/contextra-ports/src/scanner/witness.rs:87` | Gibt den BLAKE3-Digest der Messung zurück. |
| `pub fn measured_at(&self) -> u64` | `crates/contextra-ports/src/scanner/witness.rs:92` | Gibt den Zeitstempel der Messung (Unix Nanos) zurück. |

#### `fn mint_witness_for_testkit`
- Ort: `crates/contextra-ports/src/scanner/witness.rs:99`
```rust
pub fn mint_witness_for_testkit( layer: WitnessLayer, scanned: u64, remaining: u64, technique: ScanTechnique, digest: [u8
```
Dokumentation: Hilfsfunktion zur Witness-Erzeugung ausschließlich für Testzwecke (`testkit`-Feature).

### Modul `storage`

#### `mod prelude`
- Ort: `crates/contextra-ports/src/storage.rs:289`
```rust
pub mod prelude
```
Dokumentation: Prelude for storage engine ports.

#### `struct StorageStats`
- Ort: `crates/contextra-ports/src/storage.rs:26`
```rust
pub struct StorageStats
```
Dokumentation: Statistics for the storage engine.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `num_segments` | `usize` | `crates/contextra-ports/src/storage.rs:28` |
| `total_size_bytes` | `u64` | `crates/contextra-ports/src/storage.rs:30` |
| `memtable_size_bytes` | `u64` | `crates/contextra-ports/src/storage.rs:32` |

#### `trait StorageEngine`
- Ort: `crates/contextra-ports/src/storage.rs:52`
```rust
pub trait StorageEngine: Send + Sync + 'static
```
Dokumentation: Unified Storage Engine trait — retained for backward compatibility across higher layers. # Dyn-Kompatibilität Dieser Trait ist durch explizite `BoxFuture`-Rückgabetypen vtable-kompatibel (dyn-safe).

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn get<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, Result<Option<Bytes>>>` | `crates/contextra-ports/src/storage.rs:55` | Retrieves a value by key. |
| `fn get_at_seq<'a>(&'a self, key: &'a [u8], seq: u64) -> BoxFuture<'a, Result<Option<Bytes>>>` | `crates/contextra-ports/src/storage.rs:58` | Retrieves a value by key at a specific sequence number (MVCC). |
| `fn get_tracked<'a>( &'a self, _tx_id: TxId, key: &'a [u8], ) -> BoxFuture<'a, Result<Option<Bytes>>>` | `crates/contextra-ports/src/storage.rs:65` | Retrieves a value by key tracked within transaction `tx_id` for SSI read-set validation. # ReadSet Tracking Untracked: Default implementation delegates to [`Sto |
| `fn get_at_seq_tracked<'a>( &'a self, _tx_id: TxId, key: &'a [u8], seq: u64, ) -> BoxFuture<'a, Result<Option<Bytes>>>` | `crates/contextra-ports/src/storage.rs:78` | Retrieves a value by key at `seq` tracked within transaction `tx_id` for SSI read-set validation. # ReadSet Tracking Untracked: Default implementation delegates |
| `fn scan_prefix_tracked<'a>( &'a self, _tx_id: TxId, prefix: &'a [u8], ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>>` | `crates/contextra-ports/src/storage.rs:93` | Scans a range of keys with given prefix tracked within transaction `tx_id` for SSI read-set validation. # ReadSet Tracking Untracked: Default implementation del |
| `fn supports_ssi_tracking(&self) -> bool` | `crates/contextra-ports/src/storage.rs:109` | Returns `true` if this storage engine implementation actively tracks read-sets for SSI validation. Default implementation returns `false`. # Fehler Keine; stati |
| `fn put<'a>(&'a self, tx_id: TxId, key: &'a [u8], value: &'a [u8]) -> BoxFuture<'a, Result<()>>` | `crates/contextra-ports/src/storage.rs:114` | Stores a key-value pair as part of a transaction. |
| `fn put_if_absent<'a>( &'a self, _tx_id: TxId, _key: &'a [u8], _value: &'a [u8], ) -> BoxFuture<'a, Result<bool>>` | `crates/contextra-ports/src/storage.rs:117` | Atomically writes `value` under `key` within transaction ONLY IF no value exists. |
| `fn put_batch<'a>( &'a self, tx_id: TxId, entries: &'a [(Vec<u8>, Vec<u8>)], ) -> BoxFuture<'a, Result<()>>` | `crates/contextra-ports/src/storage.rs:132` | Stores multiple key-value pairs as part of a transaction. |
| `fn delete<'a>(&'a self, tx_id: TxId, key: &'a [u8]) -> BoxFuture<'a, Result<()>>` | `crates/contextra-ports/src/storage.rs:146` | Deletes a key as part of a transaction. |
| `fn delete_many<'a>(&'a self, tx_id: TxId, keys: Vec<Vec<u8>>) -> BoxFuture<'a, Result<u64>>` | `crates/contextra-ports/src/storage.rs:149` | Deletes multiple keys as a single logical batch operation. |
| `fn delete_prefix<'a>(&'a self, tx_id: TxId, prefix: &'a [u8]) -> BoxFuture<'a, Result<u64>>` | `crates/contextra-ports/src/storage.rs:162` |  |
| `fn commit<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>>` | `crates/contextra-ports/src/storage.rs:176` | Commits a transaction. |
| `fn rollback<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>>` | `crates/contextra-ports/src/storage.rs:179` | Rolls back a transaction. |
| `fn rollback_to_tx<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>>` | `crates/contextra-ports/src/storage.rs:182` | Rolls back the entire storage state to a specific transaction ID. |
| `fn flush<'a>(&'a self) -> BoxFuture<'a, Result<()>>` | `crates/contextra-ports/src/storage.rs:185` | Flushes memtable to disk. |
| `fn stats<'a>(&'a self) -> BoxFuture<'a, Result<StorageStats>>` | `crates/contextra-ports/src/storage.rs:188` | Returns storage statistics. |
| `fn last_seq_no<'a>(&'a self) -> BoxFuture<'a, Result<u64>>` | `crates/contextra-ports/src/storage.rs:191` | Returns the last sequence number committed to storage. |
| `fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, Result<TxId>>` | `crates/contextra-ports/src/storage.rs:194` | Returns the last transaction ID committed to storage. |
| `fn pin_checkpoint<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, Result<()>>` | `crates/contextra-ports/src/storage.rs:198` | Pins a checkpoint for sequence number. |
| `fn unpin_checkpoint<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, Result<()>>` | `crates/contextra-ports/src/storage.rs:201` | Unpins a checkpoint for sequence number. |
| `fn scan_prefix<'a>( &'a self, prefix: &'a [u8], ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>>` | `crates/contextra-ports/src/storage.rs:206` | Scans a range of keys with given prefix. |
| `fn scan_prefix_bounded<'a>( &'a self, prefix: &'a [u8], limit: usize, cursor: Option<&'a [u8]>, ) -> BoxFuture<'a, Result<(Vec<(Vec<u8>, Vec<u8>)>, Option<Vec<u8>>)>>` | `crates/contextra-ports/src/storage.rs:213` | Scans prefix bounded. |
| `fn scan_prefix_at<'a>( &'a self, _prefix: &'a [u8], _seq_no: u64, ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>>` | `crates/contextra-ports/src/storage.rs:247` | Scans prefix visible at sequence number. |
| `fn scan<'a>( &'a self, start: std::ops::Bound<&'a [u8]>, end: std::ops::Bound<&'a [u8]>, limit: Option<usize>, ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>>` | `crates/contextra-ports/src/storage.rs:262` | Scans range between start and end bounds. |
| `fn scan_bounded<'a>( &'a self, _start: std::ops::Bound<&'a [u8]>, _end: std::ops::Bound<&'a [u8]>, _limit: usize, _cursor: Option<&'a [u8]>, ) -> BoxFuture<'a, Result<(Vec<(Vec<u8>, Vec<u8>)>, Option<Vec<u8>>)>>` | `crates/contextra-ports/src/storage.rs:271` | Scans range bounded. |

#### `const MAX_SCAN_MERGE_ACCUMULATOR`
- Ort: `crates/contextra-ports/src/storage.rs:45`
```rust
pub const MAX_SCAN_MERGE_ACCUMULATOR: usize = 100_000
```
Dokumentation: Harte Obergrenze für die Anzahl distinkter Keys, die eine StorageEngine- Implementierung während eines einzelnen scan()/scan_prefix_bounded()-Aufrufs intern akkumulieren darf, BEVOR limit/cursor angewendet wird. Verhindert unbegrenztes Speicherwachstum bei sehr breiten Scans, unabhängig vom vom Aufrufer angeforderten `limit`. Muss größer als jedes sinnvolle `limit` sein, um normale paginierte Nutzung nicht zu beeinträchtigen.

### Modul `text_index`

#### `struct TextIndexStats`
- Ort: `crates/contextra-ports/src/text_index.rs:43`
```rust
pub struct TextIndexStats
```
Dokumentation: Statistics for a text index.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `num_documents` | `usize` | `crates/contextra-ports/src/text_index.rs:45` |
| `num_tokens` | `usize` | `crates/contextra-ports/src/text_index.rs:47` |
| `memory_usage_bytes` | `usize` | `crates/contextra-ports/src/text_index.rs:49` |

#### `trait SegmentSynthesizer`
- Ort: `crates/contextra-ports/src/text_index.rs:25`
```rust
pub trait SegmentSynthesizer: Send + Sync
```
Dokumentation: Trait-Abstraktion für LLM-Synthesizer zur Segment-Zusammenfassung (REM-Phase).

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn synthesize_segment<'a>( &'a self, segment_texts: &'a [&'a str], ) -> BoxFuture<'a, Result<String>>` | `crates/contextra-ports/src/text_index.rs:27` | Synthetisiert ein Segment von Texten zu einer abstrakten Zusammenfassung. |
| `fn model_id(&self) -> &str` | `crates/contextra-ports/src/text_index.rs:37` | Gibt die Modell-ID des Synthesizers zurück. # Fehler Keine; statische Identifikator-Abfrage. # Crash Reiner Statusabruf ohne persistente Mutation. |

#### `trait TextIndex`
- Ort: `crates/contextra-ports/src/text_index.rs:57`
```rust
pub trait TextIndex: Send + Sync + 'static
```
Dokumentation: Text-Index Trait — abstrahiert BM25/Inverted-Index-Operationen. # Dyn-Kompatibilität Verwendet native `async fn` (AFIT) für statischen Dispatch.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn search( &self, query: &str, k: usize, ) -> impl Future<Output = Result<Vec<ScoredDocument>>> + Send` | `crates/contextra-ports/src/text_index.rs:60` | Searches for documents matching the query. |
| `fn search_at( &self, query: &str, k: usize, seq_no: u64, ) -> impl Future<Output = Result<Vec<ScoredDocument>>> + Send` | `crates/contextra-ports/src/text_index.rs:72` | Searches for documents matching the query at a specific sequence number. # Errors Returns [`ContextraError::CapabilityUnsupported`][crate::ContextraError::Capab |
| `fn insert(&self, tx: TxId, id: DocId, text: &str) -> impl Future<Output = Result<()>> + Send` | `crates/contextra-ports/src/text_index.rs:93` | Inserts or updates a document in the index. # Fehler Liefert `ContextraError` bei I/O-, Tokenisierungs- oder Speicherfehlern. # Crash Transaktional gepuffert bi |
| `fn delete(&self, tx: TxId, id: DocId) -> impl Future<Output = Result<()>> + Send` | `crates/contextra-ports/src/text_index.rs:101` | Deletes a document from the index. # Fehler Liefert `ContextraError` bei Transaktions- oder I/O-Fehlern. # Crash Transaktional isoliert; bei Crash vor Commit bl |
| `fn commit(&self, tx: TxId) -> impl Future<Output = Result<()>> + Send` | `crates/contextra-ports/src/text_index.rs:109` | Commits a transaction. # Fehler Liefert `ContextraError` bei WAL-, Flush- oder Fsync-Fehlern. # Crash WAL-first; Commit ist nach erfolgreichem Fsync dauerhaft w |
| `fn rollback(&self, tx: TxId) -> impl Future<Output = Result<()>> + Send` | `crates/contextra-ports/src/text_index.rs:117` | Rolls back a transaction. # Fehler Liefert `ContextraError` bei Transaktionsverwaltungsproblemen. # Crash Verwirft uncommittete Schreibpuffer im RAM; Zustand bl |
| `fn rollback_to_tx(&self, tx_id: TxId) -> impl Future<Output = Result<()>> + Send` | `crates/contextra-ports/src/text_index.rs:125` | Rolls back the entire index state to a specific transaction ID. # Fehler Liefert `ContextraError` falls der Zielzustand nicht auffindbar oder korrupt ist. # Cra |
| `fn last_tx_id(&self) -> impl Future<Output = Result<TxId>> + Send` | `crates/contextra-ports/src/text_index.rs:133` | Returns the last transaction ID processed by the index. # Fehler Liefert `ContextraError` bei Lesefehlern des Manifests/Indexheaders. # Crash Reiner Statusabruf |
| `fn len(&self) -> impl Future<Output = usize> + Send` | `crates/contextra-ports/src/text_index.rs:141` | Returns the number of documents in the index. # Fehler Keine; panikfrei. # Crash Reiner Statusabruf aus Header oder RAM-Zustand. |
| `fn is_empty(&self) -> impl Future<Output = bool> + Send` | `crates/contextra-ports/src/text_index.rs:149` | Returns true if the index is empty. # Fehler Keine; delegiert panikfrei an `len`. # Crash Reiner Statusabruf ohne Seiteneffekte. |
| `fn stats(&self) -> impl Future<Output = Result<TextIndexStats>> + Send` | `crates/contextra-ports/src/text_index.rs:159` | Returns index statistics. # Fehler Liefert `ContextraError` falls Statistiken nicht aggregiert werden können. # Crash Reiner Lesevorgang ohne Zustandsänderung. |

### Modul `vector_index`

#### `struct VectorIndexStats`
- Ort: `crates/contextra-ports/src/vector_index.rs:24`
```rust
pub struct VectorIndexStats
```
Dokumentation: Statistics for a vector index.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `num_vectors` | `usize` | `crates/contextra-ports/src/vector_index.rs:26` |
| `memory_usage_bytes` | `usize` | `crates/contextra-ports/src/vector_index.rs:28` |
| `num_layers` | `usize` | `crates/contextra-ports/src/vector_index.rs:30` |
| `deleted_ratio` | `f64` | `crates/contextra-ports/src/vector_index.rs:33` |
| `rebuild_count` | `u64` | `crates/contextra-ports/src/vector_index.rs:36` |

#### `trait HybridSearchProvider`
- Ort: `crates/contextra-ports/src/vector_index.rs:228`
```rust
pub trait HybridSearchProvider: Send + Sync
```
Dokumentation: Contract for executing hybrid (vector + text) queries for profile routing.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn search_hybrid<'a>( &'a self, query_text: &'a str, query_embedding: &'a [f32], top_k: usize, ) -> BoxFuture<'a, Result<Vec<ContextChunk>>>` | `crates/contextra-ports/src/vector_index.rs:230` | Executes a hybrid query returning matched context chunks with relevance scores. |

#### `trait VectorIndex`
- Ort: `crates/contextra-ports/src/vector_index.rs:47`
```rust
pub trait VectorIndex: Send + Sync + 'static
```
Dokumentation: Vector Index Trait — abstrahiert die HNSW-Vektorsuche. # Dyn-Kompatibilität Verwendet native `async fn` (AFIT) für statischen Dispatch.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn insert( &self, tx: TxId, id: DocId, embedding: &[f32], ) -> impl Future<Output = Result<()>> + Send` | `crates/contextra-ports/src/vector_index.rs:50` | Inserts a vector with an associated document ID. |
| `fn all_doc_ids(&self) -> impl Future<Output = Result<Vec<DocId>>> + Send` | `crates/contextra-ports/src/vector_index.rs:63` | Returns all active (non-deleted) document IDs in the index. # Fehler Liefert `ContextraError` bei Lesefehlern der Node-Arena. # Crash Reiner Lesevorgang ohne Mo |
| `fn insert_batch( &self, tx: TxId, vectors: &[(DocId, &[f32])], ) -> impl Future<Output = Result<()>> + Send` | `crates/contextra-ports/src/vector_index.rs:68` | Inserts multiple vectors with associated document IDs. |
| `fn search( &self, query: &[f32], k: usize, ) -> impl Future<Output = Result<Vec<ScoredDocument>>> + Send` | `crates/contextra-ports/src/vector_index.rs:83` | Searches for the k nearest neighbors to a query vector. |
| `fn search_at( &self, query: &[f32], k: usize, seq_no: u64, ) -> impl Future<Output = Result<Vec<ScoredDocument>>> + Send` | `crates/contextra-ports/src/vector_index.rs:95` | Searches for the k nearest neighbors to a query vector at a specific sequence number. # Errors Returns [`ContextraError::CapabilityUnsupported`][crate::Contextr |
| `fn search_filtered( &self, query: &[f32], k: usize, filter: Option<&(dyn Fn(DocId) -> bool + Send + Sync)>, ) -> impl Future<Output = Result<Vec<ScoredDocument>>> + Send` | `crates/contextra-ports/src/vector_index.rs:123` | Searches with an optional filter predicate. # Default Behaviour Returns an error if a filter is provided. Implementors **MUST** override this method if filtered |
| `fn delete(&self, tx: TxId, id: DocId) -> impl Future<Output = Result<()>> + Send` | `crates/contextra-ports/src/vector_index.rs:146` | Deletes a vector by its document ID. # Fehler Liefert `ContextraError` bei Transaktions- oder Arena-Fehlern. # Crash Gemäß VETO-F02 strictly tombstone + slot-ze |
| `fn commit(&self, tx: TxId) -> impl Future<Output = Result<()>> + Send` | `crates/contextra-ports/src/vector_index.rs:154` | Commits a transaction. # Fehler Liefert `ContextraError` bei WAL-, Flush- oder Fsync-Fehlern. # Crash WAL-first; nach erfolgreichem fsync atomar persistent. |
| `fn rollback(&self, tx: TxId) -> impl Future<Output = Result<()>> + Send` | `crates/contextra-ports/src/vector_index.rs:162` | Rolls back a transaction. # Fehler Liefert `ContextraError` bei Transaktionsverwaltungsproblemen. # Crash Verwirft uncommittete Änderungen im RAM; Graph bleibt  |
| `fn rollback_to_tx(&self, tx_id: TxId) -> impl Future<Output = Result<()>> + Send` | `crates/contextra-ports/src/vector_index.rs:170` | Rolls back the entire index state to a specific transaction ID. # Fehler Liefert `ContextraError` falls der Ziel-Transaktionszustand nicht auffindbar ist. # Cra |
| `fn last_tx_id(&self) -> impl Future<Output = Result<TxId>> + Send` | `crates/contextra-ports/src/vector_index.rs:178` | Returns the last transaction ID processed by the index. # Fehler Liefert `ContextraError` bei Lesefehlern der Index-Metadaten. # Crash Reiner Statusabruf ohne M |
| `fn len(&self) -> impl Future<Output = usize> + Send` | `crates/contextra-ports/src/vector_index.rs:186` | Returns the number of vectors in the index. # Fehler Keine; panikfrei. # Crash Reiner Statusabruf aus Header oder RAM-Zustand. |
| `fn is_empty(&self) -> impl Future<Output = bool> + Send` | `crates/contextra-ports/src/vector_index.rs:194` | Returns true if the index is empty. # Fehler Keine; delegiert panikfrei an `len`. # Crash Reiner Statusabruf ohne Seiteneffekte. |
| `fn stats(&self) -> impl Future<Output = Result<VectorIndexStats>> + Send` | `crates/contextra-ports/src/vector_index.rs:204` | Returns index statistics. # Fehler Liefert `ContextraError` falls Statistiken nicht ermittelt werden können. # Crash Reiner Lesevorgang ohne Zustandsänderung. |
| `fn is_rebuild_required(&self) -> bool` | `crates/contextra-ports/src/vector_index.rs:212` | Returns true if the index requires a background rebuild (e.g. due to tombstone accumulation). # Fehler Keine; panikfrei. # Crash Reiner Statusabruf ohne Mutatio |
| `fn trigger_rebuild_async(&self)` | `crates/contextra-ports/src/vector_index.rs:223` | Triggers an asynchronous background rebuild of the index if supported. # Fehler Keine synchronen Fehler; asynchroner Hintergrundtask. # Crash Snapshot-isoliert; |

### Modul `vfs`

#### `struct VfsOpenOptions`
- Ort: `crates/contextra-ports/src/vfs.rs:37`
```rust
pub struct VfsOpenOptions
```
Dokumentation: Options configuring how a file is opened via [`Vfs::open`].
- Impl-Block `crates/contextra-ports/src/vfs.rs:52`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `read` | `bool` | `crates/contextra-ports/src/vfs.rs:39` |
| `write` | `bool` | `crates/contextra-ports/src/vfs.rs:41` |
| `create` | `bool` | `crates/contextra-ports/src/vfs.rs:43` |
| `create_new` | `bool` | `crates/contextra-ports/src/vfs.rs:45` |
| `truncate` | `bool` | `crates/contextra-ports/src/vfs.rs:47` |
| `append` | `bool` | `crates/contextra-ports/src/vfs.rs:49` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn read_only() -> Self` | `crates/contextra-ports/src/vfs.rs:54` | Creates options configured for read-only access. |
| `pub fn write_create() -> Self` | `crates/contextra-ports/src/vfs.rs:62` | Creates options configured for write-only access, creating the file if missing. |

#### `enum VerifiedVia`
- Ort: `crates/contextra-ports/src/vfs.rs:26`
```rust
pub enum VerifiedVia
```
Dokumentation: Method via which medium verification was performed (SPEC D-113).

Varianten:

- `Direct` — `crates/contextra-ports/src/vfs.rs:28`
- `CacheDropped` — `crates/contextra-ports/src/vfs.rs:30`
- `Cached` — `crates/contextra-ports/src/vfs.rs:32`

#### `trait Vfs`
- Ort: `crates/contextra-ports/src/vfs.rs:163`
```rust
pub trait Vfs: Send + Sync + 'static
```
Dokumentation: Abstract object-safe Virtual File System interface. Implements required file operations for LSM storage engine, SSTables, WAL sidecars, vault slot storage, and vector index persistence (SPEC 3.1, D-113, DF-01..DF-03).

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn open(&self, _path: &Path, _opts: &VfsOpenOptions) -> Result<Box<dyn VfsFile>>` | `crates/contextra-ports/src/vfs.rs:174` | Opens a file according to the specified options. # Contract V-VFS Default implementation delegates to `std::io::ErrorKind::Unsupported` unless overridden. # Feh |
| `fn read_at(&self, path: &Path, offset: u64, buf: &mut [u8]) -> Result<usize>` | `crates/contextra-ports/src/vfs.rs:188` | Reads data from byte offset `offset` into `buf` for file at `path`. # Fehler Liefert [`ContextraError::Io`](crate::error::ContextraError::Io) bei I/O-Fehlern. # |
| `fn read_verified( &self, path: &Path, offset: u64, len: usize, ) -> Result<(Vec<u8>, VerifiedVia)>` | `crates/contextra-ports/src/vfs.rs:199` | Reads verified bytes directly from media or cache-dropped layer (SPEC D-113). Returns the verified byte payload and the verification classification `VerifiedVia |
| `fn read(&self, path: &Path) -> Result<Vec<u8>>` | `crates/contextra-ports/src/vfs.rs:218` | Reads the entire contents of a file at `path`. # Fehler Liefert [`ContextraError::Io`](crate::error::ContextraError::Io) oder [`ContextraError::NotFound`](crate |
| `fn write_atomic(&self, path: &Path, data: &[u8]) -> Result<()>` | `crates/contextra-ports/src/vfs.rs:231` | Atomically writes `data` to `path` with crash-consistency guarantees. # Crash-Consistency Contract Writes to a temporary file, calls `fsync` on the file handle, |
| `fn create_dir_all(&self, path: &Path) -> Result<()>` | `crates/contextra-ports/src/vfs.rs:240` | Creates a directory and all parent directories along `path` if missing. # Fehler Liefert [`ContextraError::Io`](crate::error::ContextraError::Io) falls Verzeich |
| `fn remove_file(&self, path: &Path) -> Result<()>` | `crates/contextra-ports/src/vfs.rs:250` | Removes a file at `path`. # Fehler Liefert [`ContextraError::Io`](crate::error::ContextraError::Io) oder [`ContextraError::NotFound`](crate::error::ContextraErr |
| `fn unlink(&self, path: &Path) -> Result<()>` | `crates/contextra-ports/src/vfs.rs:258` | Canonical alias for removing a file (SPEC D-113 / Purge-Primitive). # Fehler Liefert [`ContextraError::Io`](crate::error::ContextraError::Io) oder [`ContextraEr |
| `fn rename(&self, _from: &Path, _to: &Path) -> Result<()>` | `crates/contextra-ports/src/vfs.rs:269` | Atomically renames a file from `from` to `to`. # Fehler Liefert [`ContextraError::Io`](crate::error::ContextraError::Io) falls das Umbenennen fehlschlägt. # Cra |
| `fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>>` | `crates/contextra-ports/src/vfs.rs:283` | Reads all entry paths within directory `path`. # Fehler Liefert [`ContextraError::Io`](crate::error::ContextraError::Io) bei Lesefehlern des Verzeichnisses. # C |
| `fn exists(&self, path: &Path) -> Result<bool>` | `crates/contextra-ports/src/vfs.rs:295` | Returns `true` if a file or directory exists at `path`. # Fehler Liefert [`ContextraError::Io`](crate::error::ContextraError::Io) falls Pfadstatus nicht ermitte |
| `fn len(&self, path: &Path) -> Result<u64>` | `crates/contextra-ports/src/vfs.rs:305` | Returns the length in bytes of the file at `path`. # Fehler Liefert [`ContextraError::Io`](crate::error::ContextraError::Io) oder [`ContextraError::NotFound`](c |
| `fn list_dir(&self, path: &Path) -> Result<Vec<String>>` | `crates/contextra-ports/src/vfs.rs:315` | Lists entry names (file and directory names, excluding `.` and `..`) inside directory `path`. # Fehler Liefert [`ContextraError::Io`](crate::error::ContextraErr |
| `fn list(&self, path: &Path) -> Result<Vec<String>>` | `crates/contextra-ports/src/vfs.rs:323` | Canonical alias for listing entries in directory `path` (SPEC 3.1). # Fehler Liefert [`ContextraError::Io`](crate::error::ContextraError::Io) oder [`ContextraEr |
| `fn sync_dir(&self, path: &Path) -> Result<()>` | `crates/contextra-ports/src/vfs.rs:334` | Flushes directory metadata to persistent storage for durable file additions and renames. # Fehler Liefert [`ContextraError::Io`](crate::error::ContextraError::I |

#### `trait VfsFile`
- Ort: `crates/contextra-ports/src/vfs.rs:73`
```rust
pub trait VfsFile: Send + Sync
```
Dokumentation: Abstract file handle providing positional I/O without modifying internal file offsets.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn read_at(&self, buf: &mut [u8], offset: u64) -> std::io::Result<usize>` | `crates/contextra-ports/src/vfs.rs:81` | Reads data from byte offset `offset` into `buf` without moving the file pointer. # Fehler Liefert `std::io::Error` bei Lesefehlern oder ungültigem Offset. # Cra |
| `fn read_verified(&self, offset: u64, len: usize) -> std::io::Result<(Vec<u8>, VerifiedVia)>` | `crates/contextra-ports/src/vfs.rs:94` | Reads verified data from storage medium bypassing or dropping the page cache (SPEC D-113). In accordance with SPEC D-113: Returns read bytes and classification  |
| `fn write_at(&self, buf: &[u8], offset: u64) -> std::io::Result<usize>` | `crates/contextra-ports/src/vfs.rs:108` | Writes data from `buf` to byte offset `offset` without moving the file pointer. # Fehler Liefert `std::io::Error` bei Schreibfehlern oder Speicherplatzmangel. # |
| `fn set_len(&self, len: u64) -> std::io::Result<()>` | `crates/contextra-ports/src/vfs.rs:117` | Truncates or extends the underlying file length to `len` bytes. # Fehler Liefert `std::io::Error` bei Kürzungs- oder Allokationsfehlern. # Crash Modifiziert die |
| `fn len(&self) -> std::io::Result<u64>` | `crates/contextra-ports/src/vfs.rs:126` | Queries the current physical file length in bytes. # Fehler Liefert `std::io::Error` falls Dateimetadaten nicht abgefragt werden können. # Crash Reines Abfragen |
| `fn is_empty(&self) -> std::io::Result<bool>` | `crates/contextra-ports/src/vfs.rs:135` | Returns `true` if the file length is zero. # Fehler Liefert `std::io::Error` falls die Längenabfrage fehlschlägt. # Crash Reine Längenprüfung ohne Seiteneffekte |
| `fn sync_data(&self) -> std::io::Result<()>` | `crates/contextra-ports/src/vfs.rs:146` | Flushes data blocks to disk (`fdatasync`). # Fehler Liefert `std::io::Error` bei fehlschlagendem Flush. Bei Sync-Fehlern muss das Handle vergiftet werden (DF-03 |
| `fn sync_all(&self) -> std::io::Result<()>` | `crates/contextra-ports/src/vfs.rs:155` | Flushes data blocks and metadata to disk (`fsync`). # Fehler Liefert `std::io::Error` bei Synchronisationsfehlern. Scheitert `sync_all`, ist das Handle dauerhaf |

## 9. Impl-Blöcke ohne Typ im selben Crate

- `impl<T: CommitBarrier + ?Sized> CommitBarrier for Arc<T>` — `crates/contextra-ports/src/barrier.rs:143` (0 pub Methoden)
- `impl<T: PurgeBarrier + ?Sized> PurgeBarrier for Arc<T>` — `crates/contextra-ports/src/barrier.rs:153` (0 pub Methoden)
- `impl<T: EmbeddingProvider> TextEmbeddingEngine for T` — `crates/contextra-ports/src/embedding.rs:179` (0 pub Methoden)
- `impl From<GraphMutationError> for ContextraError` — `crates/contextra-ports/src/graph.rs:139` (0 pub Methoden)
- `impl<T: ErasureObserver + ?Sized> ErasureObserver for Arc<T>` — `crates/contextra-ports/src/observer.rs:95` (0 pub Methoden)
- `impl From<PluginError> for crate::ContextraError` — `crates/contextra-ports/src/plugin.rs:112` (0 pub Methoden)
- `impl<T: ReportSigner + ?Sized> ReportSigner for Arc<T>` — `crates/contextra-ports/src/report_signer.rs:70` (0 pub Methoden)
- `impl<T: Vfs + ?Sized> Vfs for Arc<T>` — `crates/contextra-ports/src/vfs.rs:337` (0 pub Methoden)
