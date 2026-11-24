# Schnittstellenspezifikation: `contextra-cognition`

> Erzeugt durch `tools/extract_interfaces.py` aus dem Quellcode. Keine Handpflege. Jede Angabe trägt Datei:Zeile. Maßgeblich bleibt der Quellcode.

## 1. Kopf

| Feld | Wert |
|---|---|
| Name | `contextra-cognition` |
| Version | workspace |
| Pfad | `crates/contextra-cognition` |
| Rolle | Konsolidierung und Gedächtnis |
| Bibliothekspfad | src/lib.rs (Standard) |

**Crate-Dokumentation (//!):**

> # Kognitions-Engine (`contextra-cognition`)
>
> Die Crate `contextra-cognition` bildet die Kognitions- und Konsolidierungsschicht
> im Contextra-Workspace (Ebene E2 nach Spezifikation docs/SPEC.md v3).
> Sie stellt Hintergrundprozesse für Gedächtniskonsolidierung (Memory Consolidation),
> Kontext-Kompaktierung (Context Compaction), semantische Aggregation (LeanRAG / Hyperedge Consolidation)
> und Wartungsplanung (Maintenance Scheduling) bereit.
>
> ## Ring-Zugehörigkeit und Abhängigkeiten
> - **Abhängigkeiten (eingehend):** Wird von höheren Orchestrierungsschichten wie `contextra-db` eingebunden.
> - **Abhängigkeiten (ausgehend):** Nutzt Basis- und Speicher-Crates:
> - `contextra-types`: Domänentypen, Fehlermodelle, Mandanten-IDs.

## 2. Abhängigkeiten

| Name | Art | Quelle | optional | verwendet von (normal/dev) |
|---|---|---|---|---|
| `ahash` | normal | workspace | nein |  |
| `async-trait` | normal | registry | nein |  |
| `blake3` | normal | workspace | nein |  |
| `bytes` | normal | workspace | nein |  |
| `contextra-db` | dev | workspace | nein | contextra, contextra-agent, contextra-bench, contextra-cognition, contextra-db-fuzz, contextra-engine |
| `contextra-graph` | normal | workspace | nein | contextra-agent, contextra-cognition, contextra-db, contextra-engine, contextra-graph, contextra-graph-fuzz |
| `contextra-ports` | normal | workspace | nein | contextra, contextra-adapt, contextra-agent, contextra-checkpoint, contextra-cognition, contextra-crypto, contextra-crypto-fuzz, contextra-db, contextra-engine, contextra-graph, contextra-graph-fuzz, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-kvcache, contextra-mcp, contextra-mvcc, contextra-rank, contextra-router, contextra-store, contextra-store-fuzz, contextra-testkit, contextra-text, contextra-vector, contextra-vector-fuzz |
| `contextra-rank` | normal | workspace | nein | contextra-bench, contextra-cognition, contextra-db, contextra-engine, contextra-infer-candle, contextra-mcp |
| `contextra-testkit` | dev | workspace | nein | contextra-agent, contextra-checkpoint, contextra-cognition, contextra-crypto-fuzz, contextra-db, contextra-engine, contextra-store |
| `contextra-text` | normal | workspace | nein | contextra-agent, contextra-bench, contextra-cognition, contextra-db, contextra-engine, contextra-text-fuzz |
| `contextra-types` | normal | workspace | nein | contextra, contextra-adapt, contextra-agent, contextra-bench, contextra-checkpoint, contextra-cognition, contextra-crypto, contextra-crypto-fuzz, contextra-db, contextra-engine, contextra-graph, contextra-graph-fuzz, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-ingest, contextra-kvcache, contextra-mcp, contextra-mvcc, contextra-ports, contextra-py, contextra-rank, contextra-router, contextra-simd, contextra-store, contextra-store-fuzz, contextra-testkit, contextra-text, contextra-vault, contextra-vector, contextra-vector-fuzz |
| `parking_lot` | normal | workspace | nein |  |
| `serde` | normal | workspace | nein |  |
| `serde_json` | normal | workspace | nein |  |
| `tempfile` | dev | workspace | nein |  |
| `thiserror` | normal | workspace | nein |  |
| `tokio` | dev | workspace | nein |  |
| `tokio-util` | normal | registry | nein |  |
| `tracing` | normal | workspace | nein |  |

**Verwendet von (Crate-Ebene):** `contextra-db`

## 3. Features

| Feature | Aktiviert |
|---|---|
| `default` | – |
| `edge-reinforcement-learning` | `contextra-graph/edge-reinforcement-learning` |
| `graph-connectivity-health` | `contextra-graph/physio-percolation` |

## 4. Metriken (Quellcode)

| Kennzahl | Wert |
|---|---|
| Quelldateien | 29 |
| Quellzeilen | 8478 |
| Öffentliche Items (Modul-Ebene) | 103 |
| davon `fn` | 23 |
| davon `struct` | 42 |
| davon `enum` | 3 |
| davon `trait` | 8 |
| davon `type` | 1 |
| davon `const` | 4 |
| davon `mod` | 22 |
| Öffentliche Makros (`#[macro_export]`) | 0 |
| Öffentliche Re-Exports (`pub use`) | 41 |
| `unsafe`-Vorkommen (Produktionscode) | 0 |
| `.unwrap()` (Produktionscode) | 16 |
| `.expect(` (Produktionscode) | 31 |
| `panic!` (Produktionscode) | 0 |
| `todo!/unimplemented!/unreachable!` | 1 |
| Indexierungen `x[…]` (grob, Produktionscode) | 48 |
| `#[deprecated]` | 2 |
| `#[cfg(feature…)]`-Gates | 8 |
| Parser-Abdeckung (geparste `pub`-Deklarationen / Grep-Treffer) | 194 / 194 = 100 % |

## 5. Modulbaum und Dateien

| Modul | Datei | Zeilen |
|---|---|---|
| `aggregation_phase` | `crates/contextra-cognition/src/aggregation_phase.rs` | 930 |
| `channel` | `crates/contextra-cognition/src/channel.rs` | 149 |
| `collection` | `crates/contextra-cognition/src/collection.rs` | 395 |
| `collection_types` | `crates/contextra-cognition/src/collection_types.rs` | 38 |
| `consolidation_executor` | `crates/contextra-cognition/src/consolidation_executor.rs` | 1161 |
| `consolidation_locks` | `crates/contextra-cognition/src/consolidation_locks.rs` | 131 |
| `context` | `crates/contextra-cognition/src/context.rs` | 660 |
| `context_compaction::cleanup` | `crates/contextra-cognition/src/context_compaction/cleanup.rs` | 62 |
| `context_compaction::compactor` | `crates/contextra-cognition/src/context_compaction/compactor.rs` | 392 |
| `context_compaction` | `crates/contextra-cognition/src/context_compaction/mod.rs` | 42 |
| `context_compaction::session` | `crates/contextra-cognition/src/context_compaction/session.rs` | 364 |
| `context_compaction::tests` | `crates/contextra-cognition/src/context_compaction/tests.rs` | 657 |
| `context_compaction::types` | `crates/contextra-cognition/src/context_compaction/types.rs` | 67 |
| `decay` | `crates/contextra-cognition/src/decay.rs` | 190 |
| `engine_local` | `crates/contextra-cognition/src/engine_local.rs` | 25 |
| `error` | `crates/contextra-cognition/src/error.rs` | 91 |
| `graph_local` | `crates/contextra-cognition/src/graph_local.rs` | 19 |
| `graph_sink` | `crates/contextra-cognition/src/graph_sink.rs` | 114 |
| `leanrag_input` | `crates/contextra-cognition/src/leanrag_input.rs` | 140 |
| `crate` | `crates/contextra-cognition/src/lib.rs` | 124 |
| `maintenance_config` | `crates/contextra-cognition/src/maintenance_config.rs` | 118 |
| `maintenance_scheduler` | `crates/contextra-cognition/src/maintenance_scheduler.rs` | 568 |
| `memory_consolidation` | `crates/contextra-cognition/src/memory_consolidation.rs` | 1315 |
| `ports` | `crates/contextra-cognition/src/ports.rs` | 136 |
| `ports_local` | `crates/contextra-cognition/src/ports_local.rs` | 17 |
| `semantic_aggregation_facade` | `crates/contextra-cognition/src/semantic_aggregation_facade.rs` | 130 |
| `synthesis_phase` | `crates/contextra-cognition/src/synthesis_phase.rs` | 250 |
| `transitivity_veto` | `crates/contextra-cognition/src/transitivity_veto.rs` | 178 |
| `types_local` | `crates/contextra-cognition/src/types_local.rs` | 15 |

## 6. Re-Exports (`pub use`)

- `pub use cleanup::cleanup_orphaned_consolidation_intents` — `crates/contextra-cognition/src/context_compaction/mod.rs:38` (Modul `context_compaction`)
- `pub use compactor::ContextCompactor` — `crates/contextra-cognition/src/context_compaction/mod.rs:39` (Modul `context_compaction`)
- `pub use session::ConsolidationSession` — `crates/contextra-cognition/src/context_compaction/mod.rs:40` (Modul `context_compaction`)
- `pub use types::` — `crates/contextra-cognition/src/context_compaction/mod.rs:41` (Modul `context_compaction`)
- `pub use crate::collection::` — `crates/contextra-cognition/src/engine_local.rs:7` (Modul `engine_local`)
- `pub use crate::collection_types::` — `crates/contextra-cognition/src/engine_local.rs:8` (Modul `engine_local`)
- `pub use crate::decay::` — `crates/contextra-cognition/src/engine_local.rs:9` (Modul `engine_local`)
- `pub use crate::ports::` — `crates/contextra-cognition/src/engine_local.rs:10` (Modul `engine_local`)
- `pub use contextra_rank::fusion::` — `crates/contextra-cognition/src/engine_local.rs:11` (Modul `engine_local`)
- `pub use contextra_text::Language` — `crates/contextra-cognition/src/engine_local.rs:12` (Modul `engine_local`)
- `pub use crate::collection::` — `crates/contextra-cognition/src/engine_local.rs:15` (Modul `engine_local::collection`)
- `pub use crate::collection_types::` — `crates/contextra-cognition/src/engine_local.rs:16` (Modul `engine_local::collection`)
- `pub use crate::decay::` — `crates/contextra-cognition/src/engine_local.rs:20` (Modul `engine_local::decay_controller`)
- `pub use crate::ports::CommitIntent` — `crates/contextra-cognition/src/engine_local.rs:24` (Modul `engine_local::transaction`)
- `pub use contextra_graph as graph` — `crates/contextra-cognition/src/graph_local.rs:10` (Modul `graph_local`)
- `pub use graph::cascade::` — `crates/contextra-cognition/src/graph_local.rs:11` (Modul `graph_local`)
- `pub use graph::cascade_invalidate_edges_for_superseded_doc` — `crates/contextra-cognition/src/graph_local.rs:12` (Modul `graph_local`)
- `pub use graph::csr::` — `crates/contextra-cognition/src/graph_local.rs:13` (Modul `graph_local`)
- `pub use graph::hyperedge::` — `crates/contextra-cognition/src/graph_local.rs:14` (Modul `graph_local`)
- `pub use graph::` — `crates/contextra-cognition/src/graph_local.rs:15` (Modul `graph_local`)
- `pub use graph::percolation::PercolationConfig` — `crates/contextra-cognition/src/graph_local.rs:18` (Modul `graph_local`)
- `pub use error::CognitionError` — `crates/contextra-cognition/src/lib.rs:45` (Modul `crate`)
- `pub use aggregation_phase::` — `crates/contextra-cognition/src/lib.rs:89` (Modul `crate`)
- `pub use channel::` — `crates/contextra-cognition/src/lib.rs:94` (Modul `crate`)
- `pub use consolidation_executor::` — `crates/contextra-cognition/src/lib.rs:99` (Modul `crate`)
- `pub use consolidation_locks::ConsolidationNodesGuard` — `crates/contextra-cognition/src/lib.rs:104` (Modul `crate`)
- `pub use context::` — `crates/contextra-cognition/src/lib.rs:105` (Modul `crate`)
- `pub use context_compaction::` — `crates/contextra-cognition/src/lib.rs:106` (Modul `crate`)
- `pub use graph_sink::CsrGraphSuperEdgeSink` — `crates/contextra-cognition/src/lib.rs:110` (Modul `crate`)
- `pub use leanrag_input::` — `crates/contextra-cognition/src/lib.rs:111` (Modul `crate`)
- `pub use maintenance_config::MaintenanceConfig` — `crates/contextra-cognition/src/lib.rs:112` (Modul `crate`)
- `pub use maintenance_scheduler::MaintenanceScheduler` — `crates/contextra-cognition/src/lib.rs:113` (Modul `crate`)
- `pub use memory_consolidation::` — `crates/contextra-cognition/src/lib.rs:114` (Modul `crate`)
- `pub use semantic_aggregation_facade::consolidate_semantic_hyperedges` — `crates/contextra-cognition/src/lib.rs:121` (Modul `crate`)
- `pub use synthesis_phase::run_synthesis_pass` — `crates/contextra-cognition/src/lib.rs:122` (Modul `crate`)
- `pub use transitivity_veto::` — `crates/contextra-cognition/src/lib.rs:123` (Modul `crate`)
- `pub use contextra_ports as ports` — `crates/contextra-cognition/src/ports_local.rs:10` (Modul `ports_local`)
- `pub use ports::` — `crates/contextra-cognition/src/ports_local.rs:11` (Modul `ports_local`)
- `pub use crate::ports_local::SegmentSynthesizer as TraitSegmentSynthesizer` — `crates/contextra-cognition/src/synthesis_phase.rs:57` (Modul `synthesis_phase`)
- `pub use contextra_types as types` — `crates/contextra-cognition/src/types_local.rs:10` (Modul `types_local`)
- `pub use types::` — `crates/contextra-cognition/src/types_local.rs:11` (Modul `types_local`)

## 8. Öffentliche Schnittstelle je Item

### Modul `aggregation_phase`

#### `struct AggregationConfig`
- Ort: `crates/contextra-cognition/src/aggregation_phase.rs:43`
```rust
pub struct AggregationConfig
```
- Impl-Block (Trait `Default`) `crates/contextra-cognition/src/aggregation_phase.rs:66`
- Impl-Block `crates/contextra-cognition/src/aggregation_phase.rs:83`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `max_compaction_peak_memory_mb` | `usize` | `crates/contextra-cognition/src/aggregation_phase.rs:45` |
| `clustering_tau_threshold` | `f32` | `crates/contextra-cognition/src/aggregation_phase.rs:47` |
| `gmm_deterministic_seed` | `u64` | `crates/contextra-cognition/src/aggregation_phase.rs:49` |
| `max_llm_calls_per_cycle` | `usize` | `crates/contextra-cognition/src/aggregation_phase.rs:51` |
| `max_clusters` | `usize` | `crates/contextra-cognition/src/aggregation_phase.rs:53` |
| `min_cluster_size` | `usize` | `crates/contextra-cognition/src/aggregation_phase.rs:55` |
| `em_max_iterations` | `u32` | `crates/contextra-cognition/src/aggregation_phase.rs:57` |
| `em_tolerance` | `f32` | `crates/contextra-cognition/src/aggregation_phase.rs:59` |
| `min_type_compat_score` | `f32` | `crates/contextra-cognition/src/aggregation_phase.rs:61` |
| `stability_cycles_required` | `u32` | `crates/contextra-cognition/src/aggregation_phase.rs:63` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn validate(&self) -> Result<()>` | `crates/contextra-cognition/src/aggregation_phase.rs:85` | Validiert die Konfigurationseinstellungen. |

#### `struct AggregationEdge`
- Ort: `crates/contextra-cognition/src/aggregation_phase.rs:176`
```rust
pub struct AggregationEdge
```
Dokumentation: Eingabekante für die Aggregationsphase.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `id` | `HyperEdgeId` | `crates/contextra-cognition/src/aggregation_phase.rs:178` |
| `predicate_type` | `u32` | `crates/contextra-cognition/src/aggregation_phase.rs:180` |
| `participants` | `Vec<EntityId>` | `crates/contextra-cognition/src/aggregation_phase.rs:182` |

#### `struct AggregationNode`
- Ort: `crates/contextra-cognition/src/aggregation_phase.rs:163`
```rust
pub struct AggregationNode
```
Dokumentation: Eingabeknoten für die Aggregationsphase.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `entity` | `EntityId` | `crates/contextra-cognition/src/aggregation_phase.rs:165` |
| `embedding` | `Vec<f32>` | `crates/contextra-cognition/src/aggregation_phase.rs:167` |
| `type_id` | `u32` | `crates/contextra-cognition/src/aggregation_phase.rs:169` |

#### `struct AggregationPhaseResult`
- Ort: `crates/contextra-cognition/src/aggregation_phase.rs:135`
```rust
pub struct AggregationPhaseResult
```
Dokumentation: Ergebnis der Aggregationsphase.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `raw_edges_invalidated` | `usize` | `crates/contextra-cognition/src/aggregation_phase.rs:137` |
| `abstract_hyperedges_created` | `usize` | `crates/contextra-cognition/src/aggregation_phase.rs:139` |
| `peak_memory_used_mb` | `usize` | `crates/contextra-cognition/src/aggregation_phase.rs:141` |
| `child_edge_ids_written` | `usize` | `crates/contextra-cognition/src/aggregation_phase.rs:143` |

#### `struct AlphaNode`
- Ort: `crates/contextra-cognition/src/aggregation_phase.rs:204`
```rust
pub struct AlphaNode
```
Dokumentation: Synthetisierter abstrakter Alpha-Knoten.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `cluster_id` | `usize` | `crates/contextra-cognition/src/aggregation_phase.rs:206` |
| `members` | `Vec<EntityId>` | `crates/contextra-cognition/src/aggregation_phase.rs:208` |
| `summary` | `String` | `crates/contextra-cognition/src/aggregation_phase.rs:210` |

#### `struct ConsolidationPipelineResult`
- Ort: `crates/contextra-cognition/src/aggregation_phase.rs:150`
```rust
pub struct ConsolidationPipelineResult
```
Dokumentation: Gesamtergebnis der dreistufigen Konsolidierungspipeline.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `structural` | `ConsolidationPhaseResult` | `crates/contextra-cognition/src/aggregation_phase.rs:152` |
| `synthesis` | `crate::memory_consolidation::SynthesisPhaseResult` | `crates/contextra-cognition/src/aggregation_phase.rs:154` |
| `aggregation` | `Option<AggregationPhaseResult>` | `crates/contextra-cognition/src/aggregation_phase.rs:156` |

#### `struct SuperEdgeDraft`
- Ort: `crates/contextra-cognition/src/aggregation_phase.rs:189`
```rust
pub struct SuperEdgeDraft
```
Dokumentation: Entwurf einer abstrakten Super-Hyperkante.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `participants` | `Vec<EntityId>` | `crates/contextra-cognition/src/aggregation_phase.rs:193` |
| `child_edge_ids` | `Vec<HyperEdgeId>` | `crates/contextra-cognition/src/aggregation_phase.rs:195` |
| `weight` | `f32` | `crates/contextra-cognition/src/aggregation_phase.rs:197` |

#### `trait SuperEdgeSink`
- Ort: `crates/contextra-cognition/src/aggregation_phase.rs:216`
```rust
pub trait SuperEdgeSink
```
Dokumentation: Senkentrait für Superkanten und Tombstones.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn invalidate_edge(&mut self, id: HyperEdgeId) -> Result<()>` | `crates/contextra-cognition/src/aggregation_phase.rs:218` | Markiert eine Kante als invalidiert (superseded). |
| `fn write_super_edge(&mut self, draft: SuperEdgeDraft) -> Result<HyperEdgeId>` | `crates/contextra-cognition/src/aggregation_phase.rs:220` | Schreibt eine neue Super-Hyperkante und gibt deren zugewiesene ID zurück. |
| `fn commit(&mut self) -> Result<()>` | `crates/contextra-cognition/src/aggregation_phase.rs:222` | Atomarer Commit aller Änderungen. |

#### `fn check_compaction_budget`
- Ort: `crates/contextra-cognition/src/aggregation_phase.rs:228`
```rust
pub fn check_compaction_budget(peak_bytes: usize, cfg: &AggregationConfig) -> Result<()>
```
Dokumentation: Prüft den geschätzten Speicherbedarf gegen das konfigurierte Budget.

#### `fn compute_entity_community_hash`
- Ort: `crates/contextra-cognition/src/aggregation_phase.rs:274`
```rust
pub fn compute_entity_community_hash(members: &[EntityId]) -> u64
```

#### `fn run_aggregation_pass`
- Ort: `crates/contextra-cognition/src/aggregation_phase.rs:584`
```rust
pub async fn run_aggregation_pass( nodes: &[AggregationNode], edges: &[AggregationEdge], cfg: &AggregationConfig, llm: &dyn LlmTextGenerator, tracker: &mut CommunityStabilityTracker, sink: &mut dyn SuperEdgeSink, ) -> Result<(AggregationPhaseResult, Vec<AlphaNode>)>
```

### Modul `channel`

#### `struct DirectCognitiveChannel`
- Ort: `crates/contextra-cognition/src/channel.rs:86`
```rust
pub struct DirectCognitiveChannel
```
Dokumentation: Standalone synchronous/direct intake adapter implementing `CognitiveInputChannel`.
- Impl-Block (Trait `CognitiveInputChannel`) `crates/contextra-cognition/src/channel.rs:88`

#### `struct InMemoryCognitiveChannel`
- Ort: `crates/contextra-cognition/src/channel.rs:48`
```rust
pub struct InMemoryCognitiveChannel
```
Dokumentation: In-Memory asynchronous MPSC implementation of `CognitiveInputChannel`.
- Impl-Block `crates/contextra-cognition/src/channel.rs:52`
- Impl-Block (Trait `CognitiveInputChannel`) `crates/contextra-cognition/src/channel.rs:60`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(capacity: usize) -> (Self, mpsc::Receiver<(TenantId, Vec<PreCondensedInput>)>)` | `crates/contextra-cognition/src/channel.rs:54` | Creates a new `InMemoryCognitiveChannel` with given buffer capacity. |

#### `struct InMemoryOutputSink`
- Ort: `crates/contextra-cognition/src/channel.rs:107`
```rust
pub struct InMemoryOutputSink<T>
```
Dokumentation: In-memory queue sink collecting emitted items in an `Arc<parking_lot::Mutex<Vec<T>>>`.
- Impl-Block `crates/contextra-cognition/src/channel.rs:111`
- Impl-Block (Trait `Default`) `crates/contextra-cognition/src/channel.rs:135`
- Impl-Block (Trait `CognitiveOutputSink<T>`) `crates/contextra-cognition/src/channel.rs:141`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-cognition/src/channel.rs:113` | Creates a new empty `InMemoryOutputSink`. |
| `pub fn collected(&self) -> Vec<T>` | `crates/contextra-cognition/src/channel.rs:120` | Returns a copy of all collected items. |
| `pub fn len(&self) -> usize` | `crates/contextra-cognition/src/channel.rs:125` | Returns the number of collected items. |
| `pub fn is_empty(&self) -> bool` | `crates/contextra-cognition/src/channel.rs:130` | Checks if the sink is empty. |

#### `trait CognitiveInputChannel`
- Ort: `crates/contextra-cognition/src/channel.rs:25`
```rust
pub trait CognitiveInputChannel: Send + Sync
```
Dokumentation: Abstract asynchronous input channel for feeding cognitive inputs.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn send_pre_condensed<'a>( &'a self, target_tenant_id: &'a TenantId, inputs: &'a [PreCondensedInput], ) -> BoxFuture<'a, Result<Vec<TurnSegment>>>` | `crates/contextra-cognition/src/channel.rs:27` | Ingests a pre-condensed input batch into the cognition engine. |

#### `trait CognitiveOutputSink`
- Ort: `crates/contextra-cognition/src/channel.rs:36`
```rust
pub trait CognitiveOutputSink<T>: Send + Sync
```
Dokumentation: Abstract asynchronous output sink for dispatching consolidated or synthesized chunks.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn emit<'a>(&'a self, item: T) -> BoxFuture<'a, Result<()>>` | `crates/contextra-cognition/src/channel.rs:38` | Emits a consolidated item to the destination sink. |

### Modul `collection`

#### `struct Collection`
- Ort: `crates/contextra-cognition/src/collection.rs:68`
```rust
pub struct Collection<S: StorageEngine, V: VectorIndex>
```
Dokumentation: Repräsentiert eine namensraumisolierte Collection innerhalb der Kognitions-Engine.
- Impl-Block (Trait `Clone`) `crates/contextra-cognition/src/collection.rs:81`
- Impl-Block `crates/contextra-cognition/src/collection.rs:98`
- Impl-Block (Trait `CollectionOps`) `crates/contextra-cognition/src/collection.rs:351`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `consolidation_guard` | `Arc<tokio::sync::Mutex<()>>` | `crates/contextra-cognition/src/collection.rs:77` |
| `consolidation_in_progress` | `Arc<AtomicBool>` | `crates/contextra-cognition/src/collection.rs:78` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new( name: String, storage: Arc<S>, index: Arc<V>, graph_index: Arc<CsrGraph>, next_tx: Arc<AtomicU64>, dimension: usize, language: Language, ) -> Self` | `crates/contextra-cognition/src/collection.rs:100` | Erstellt eine neue Collection. |
| `pub fn name(&self) -> &str` | `crates/contextra-cognition/src/collection.rs:130` | Gibt den Namen der Collection zurück. |
| `pub fn dimension(&self) -> usize` | `crates/contextra-cognition/src/collection.rs:135` | Gibt die Vektordimension zurück. |
| `pub fn storage(&self) -> &S` | `crates/contextra-cognition/src/collection.rs:140` | Gibt eine Referenz auf den physischen Speicher zurück. |
| `pub fn index(&self) -> &V` | `crates/contextra-cognition/src/collection.rs:145` | Gibt den Vektorindex zurück. |
| `pub fn graph_index(&self) -> Arc<CsrGraph>` | `crates/contextra-cognition/src/collection.rs:150` | Gibt den Wissensgraphen zurück. |
| `pub fn namespaced_key(&self, raw: &[u8], key_type: u8) -> Vec<u8>` | `crates/contextra-cognition/src/collection.rs:155` | Erzeugt einen namensraumisolierten Schlüssel. |
| `pub fn user_key_prefix(&self) -> Vec<u8>` | `crates/contextra-cognition/src/collection.rs:164` | Gibt das Schlüsselpräfix für reguläre Benutzerdaten zurück. |
| `pub fn allocate_tx(&self) -> Result<TxId>` | `crates/contextra-cognition/src/collection.rs:171` | Allokiert eine Transaktions-ID. |
| `pub fn begin_transaction(&self) -> Result<DbTransaction<S, V>>` | `crates/contextra-cognition/src/collection.rs:177` | Beginnt eine neue Transaktion. |
| `pub async fn get_doc_tx(&self, _doc_id: DocId) -> Result<Option<TxId>>` | `crates/contextra-cognition/src/collection.rs:183` | Ruft den Transaktionszeitstempel eines Dokuments ab. |
| `pub fn consolidation_in_progress(&self) -> Arc<AtomicBool>` | `crates/contextra-cognition/src/collection.rs:188` | Prüft, ob eine Konsolidierung aktiv ist. |
| `pub async fn insert_op( &self, db_tx: &DbTransaction<S, V>, id: &str, embedding: &[f32], metadata: Option<serde_json::Value>, ) -> Result<()>` | `crates/contextra-cognition/src/collection.rs:193` | Bereitet das Einfügen eines Dokuments in einer Transaktion vor. |
| `pub async fn delete_op(&self, db_tx: &mut DbTransaction<S, V>, id: &str) -> Result<()>` | `crates/contextra-cognition/src/collection.rs:215` | Bereitet das Löschen eines Dokuments in einer Transaktion vor. |
| `pub async fn get(&self, id: &str) -> Result<Option<StoredDocument>>` | `crates/contextra-cognition/src/collection.rs:224` | Ruft ein Dokument ab. |
| `pub async fn delete(&self, id: &str) -> Result<()>` | `crates/contextra-cognition/src/collection.rs:242` | Löscht ein Dokument direkt. |
| `pub async fn insert_text_only( &self, id: &str, text: &str, mut metadata: Option<serde_json::Value>, ) -> Result<()>` | `crates/contextra-cognition/src/collection.rs:254` | Inserts a text document. |
| `pub async fn put_kv(&self, key: &str, value: &serde_json::Value) -> Result<()>` | `crates/contextra-cognition/src/collection.rs:273` | Schreibt einen beliebigen Schlüssel-Wert-Eintrag. |
| `pub async fn evict_decayed_chunks( &self, _controller: &AdaptiveDecayController, _limit: usize, ) -> Result<usize>` | `crates/contextra-cognition/src/collection.rs:284` | Bereinigt abgewertete Chunks. |
| `pub async fn insert( &self, id: &str, embedding: &[f32], metadata: Option<serde_json::Value>, ) -> Result<()>` | `crates/contextra-cognition/src/collection.rs:296` | Fügt ein Dokument mit Embedding und optionalen Metadaten ein. Schlanke Insert-Methode, die `insert_op` intern nutzt. Keine Backpressure oder komplexes Locking — |
| `pub async fn insert_typed( &self, id: &str, embedding: &[f32], _memory_type: contextra_types::MemoryType, metadata: Option<serde_json::Value>, ) -> Result<()>` | `crates/contextra-cognition/src/collection.rs:311` | Fügt ein Dokument mit typisiertem `MemoryType` ein (Wrapper um `insert`). |
| `pub async fn update( &self, id: &str, embedding: &[f32], metadata: Option<serde_json::Value>, ) -> Result<()>` | `crates/contextra-cognition/src/collection.rs:322` | Aktualisiert ein Dokument (upsert-Semantik, Alias für `insert`). |
| `pub async fn relate( &self, from: &str, to: &str, label: &str, ) -> Result<()>` | `crates/contextra-cognition/src/collection.rs:332` | Erstellt eine gerichtete Relation zwischen zwei Dokumenten im Graph-Index. |

#### `struct DbTransaction`
- Ort: `crates/contextra-cognition/src/collection.rs:22`
```rust
pub struct DbTransaction<S: StorageEngine, V: VectorIndex>
```
Dokumentation: Lokale Transaktionsstruktur für 2-Phase Commits in der Konsolidierungssitzung.
- Impl-Block `crates/contextra-cognition/src/collection.rs:29`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `tx_id` | `TxId` | `crates/contextra-cognition/src/collection.rs:23` |
| `staged_deletes` | `Vec<String>` | `crates/contextra-cognition/src/collection.rs:26` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(collection: Collection<S, V>, tx_id: TxId) -> Self` | `crates/contextra-cognition/src/collection.rs:31` | Erzeugt eine neue Transaktion. |
| `pub async fn commit(self) -> Result<()>` | `crates/contextra-cognition/src/collection.rs:41` | Führt den Transaktions-Commit aus. |

### Modul `collection_types`

#### `struct StoredDocument`
- Ort: `crates/contextra-cognition/src/collection_types.rs:13`
```rust
pub struct StoredDocument
```
Dokumentation: Dokumentenstruktur für gespeicherte Entitäten und Chunks.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `id` | `String` | `crates/contextra-cognition/src/collection_types.rs:15` |
| `embedding` | `Vec<f32>` | `crates/contextra-cognition/src/collection_types.rs:17` |
| `metadata` | `Option<serde_json::Value>` | `crates/contextra-cognition/src/collection_types.rs:19` |

#### `struct StoredDocumentMeta`
- Ort: `crates/contextra-cognition/src/collection_types.rs:24`
```rust
pub struct StoredDocumentMeta
```
Dokumentation: Leichtgewichtige Metadaten für Dokumente (ohne Embedding-Vektor).
- Impl-Block (Trait `From<&StoredDocument>`) `crates/contextra-cognition/src/collection_types.rs:31`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `id` | `String` | `crates/contextra-cognition/src/collection_types.rs:26` |
| `metadata` | `Option<serde_json::Value>` | `crates/contextra-cognition/src/collection_types.rs:28` |

### Modul `consolidation_executor`

#### `struct ConsolidationEngine`
- Ort: `crates/contextra-cognition/src/consolidation_executor.rs:530`
```rust
pub struct ConsolidationEngine<S: StorageEngine, V: VectorIndex>
```
- Impl-Block `crates/contextra-cognition/src/consolidation_executor.rs:543`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new( collection: Arc<Collection<S, V>>, consolidation_config: ConsolidationConfig, synthesis_config: SynthesisConfig, interval: std::time::Duration, cancel_token: tokio_util::sync::CancellationToken, ) -> Self` | `crates/contextra-cognition/src/consolidation_executor.rs:545` | Erstellt eine neue `ConsolidationEngine`-Instanz. |
| `pub fn with_llm(mut self, llm: Arc<dyn LlmTextGenerator>) -> Self` | `crates/contextra-cognition/src/consolidation_executor.rs:567` | Fügt ein optionales LLM für die generative Wissenssynthese hinzu. |
| `pub fn with_leanrag(mut self, cfg: AggregationConfig) -> Self` | `crates/contextra-cognition/src/consolidation_executor.rs:573` | Fügt eine optionale AggregationConfig für LeanRAG Stage 3 (Opt-in) hinzu. |
| `pub fn with_validator(mut self, validator: Arc<dyn ResponseGroundingValidator>) -> Self` | `crates/contextra-cognition/src/consolidation_executor.rs:579` | Fügt einen optionalen Grounding-Validator für den Generative Synthesis Pass hinzu. |
| `pub fn with_rich_validator(mut self, validator: Arc<dyn GroundingValidator>) -> Self` | `crates/contextra-cognition/src/consolidation_executor.rs:585` | Fügt einen optionalen reichhaltigen Grounding-Validator (`GroundingValidator`) mit Kalibrierung & Policy-Violation-Semantik hinzu. |
| `pub fn start(self: Arc<Self>) -> tokio::task::JoinHandle<()>` | `crates/contextra-cognition/src/consolidation_executor.rs:591` | Startet die Engine als Tokio-Background-Task. |
| `pub fn start_worker( collection: Arc<Collection<S, V>>, consolidation_config: ConsolidationConfig, synthesis_config: SynthesisConfig, interval: std::time::Duration, cancel_token: tokio_util::sync::CancellationToken, ) -> tokio::task::JoinHandle<()>` | `crates/contextra-cognition/src/consolidation_executor.rs:598` | Starts a background task for periodic consolidation. |
| `pub async fn run(&self)` | `crates/contextra-cognition/src/consolidation_executor.rs:616` | Hauptschleife der `ConsolidationEngine`. |
| `pub async fn run_cycle( &self, ) -> Result<(ConsolidationPhaseResult, Option<SynthesisPhaseResult>)>` | `crates/contextra-cognition/src/consolidation_executor.rs:754` | Führt einen einzelnen Konsolidierungszyklus aus. |

#### `struct ConsolidationLockGuard`
- Ort: `crates/contextra-cognition/src/consolidation_executor.rs:60`
```rust
pub struct ConsolidationLockGuard
```
Dokumentation: RAII-Guard zur Absicherung gegen parallele Konsolidierungsläufe.
- Impl-Block `crates/contextra-cognition/src/consolidation_executor.rs:64`
- Impl-Block (Trait `Drop`) `crates/contextra-cognition/src/consolidation_executor.rs:80`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn try_acquire(flag: &Arc<AtomicBool>) -> Option<Self>` | `crates/contextra-cognition/src/consolidation_executor.rs:68` | Versucht, das Konsolidierungs-Lock atomic zu erwerben (`compare_exchange`). Gibt `Some(ConsolidationLockGuard)` zurück, wenn das Lock erfolgreich reserviert wur |

#### `struct GroundingValidatorAdapter`
- Ort: `crates/contextra-cognition/src/consolidation_executor.rs:192`
```rust
pub struct GroundingValidatorAdapter<'a>(pub &'a dyn GroundingValidator)
```
Dokumentation: Adapter für GroundingValidator auf ResponseGroundingValidator.
- Impl-Block (Trait `ResponseGroundingValidator`) `crates/contextra-cognition/src/consolidation_executor.rs:194`

#### `fn execute_background_consolidation`
- Ort: `crates/contextra-cognition/src/consolidation_executor.rs:246`
```rust
pub async fn execute_background_consolidation<S: StorageEngine, V: VectorIndex>( collection: &Collection<S, V>, turns: &[(DocId, Vec<f32>)], consolidation_config: &ConsolidationConfig, synthesis_config: Option<&SynthesisConfig>, llm: Option<&dyn LlmTextGenerator>, validator: Option<&dyn ResponseGroundingValidator>, stability_tracker: Option<&mut CommunityStabilityTracker>, ) -> Result<(ConsolidationPhaseResult, Option<SynthesisPhaseResult>)>
```
Dokumentation: Führt die vollständige Hintergrund-Konsolidierung aus.

#### `fn execute_background_consolidation_with_rich_validator`
- Ort: `crates/contextra-cognition/src/consolidation_executor.rs:373`
```rust
pub async fn execute_background_consolidation_with_rich_validator< S: StorageEngine, V: VectorIndex, >( collection: &Collection<S, V>, turns: &[(DocId, Vec<f32>)], consolidation_config: &ConsolidationConfig, synthesis_config: Option<&SynthesisConfig>, llm: Option<&dyn LlmTextGenerator>, validator: Option<&dyn ResponseGroundingValidator>, rich_validator: Option<&dyn GroundingValidator>, stability_tracker: Option<&mut CommunityStabilityTracker>, ) -> Result<(ConsolidationPhaseResult, Option<SynthesisPhaseResult>)>
```
Dokumentation: Führt Hintergrund-Konsolidierung mit Unterstützung für Rich-Grounding-Validatoren aus.

#### `fn execute_consolidation_pass`
- Ort: `crates/contextra-cognition/src/consolidation_executor.rs:126`
```rust
pub async fn execute_consolidation_pass<S: StorageEngine, V: VectorIndex>( collection: &Collection<S, V>, turns: &[(DocId, Vec<f32>)], config: &ConsolidationConfig, ) -> Result<ConsolidationPhaseResult>
```
Dokumentation: Führt den Structural Pass aus und wendet Änderungen auf die Collection an.

#### `fn execute_leanrag_aggregation_stage`
- Ort: `crates/contextra-cognition/src/consolidation_executor.rs:440`
```rust
pub async fn execute_leanrag_aggregation_stage<S: StorageEngine, V: VectorIndex>( collection: &Collection<S, V>, nodes: &[AggregationNode], edges: &[AggregationEdge], cfg: &AggregationConfig, llm: &dyn LlmTextGenerator, tracker: &mut CommunityStabilityTracker, wal_tx: contextra_types::TxId, ) -> Result<(AggregationPhaseResult, Vec<AlphaNode>)>
```

#### `fn execute_sleep_cycle`
- Ort: `crates/contextra-cognition/src/consolidation_executor.rs:463`
```rust
pub async fn execute_sleep_cycle<S: StorageEngine, V: VectorIndex>( collection: &Collection<S, V>, turns: &[(DocId, Vec<f32>)], consolidation_config: &ConsolidationConfig, synthesis_config: Option<&SynthesisConfig>, llm: Option<&dyn LlmTextGenerator>, stability_tracker: Option<&mut CommunityStabilityTracker>, ) -> Result<(ConsolidationPhaseResult, Option<SynthesisPhaseResult>)>
```
Dokumentation: Veralteter Wrapper für execute_background_consolidation.

#### `fn start_consolidation_reaper`
- Ort: `crates/contextra-cognition/src/consolidation_executor.rs:508`
```rust
pub fn start_consolidation_reaper<S: StorageEngine + 'static, V: VectorIndex + 'static>( collection: Arc<Collection<S, V>>, consolidation_config: ConsolidationConfig, synthesis_config: SynthesisConfig, interval: std::time::Duration, cancel_token: tokio_util::sync::CancellationToken, ) -> tokio::task::JoinHandle<()>
```
Dokumentation: Veralteter Alias für start_consolidation_worker.

#### `fn start_consolidation_worker`
- Ort: `crates/contextra-cognition/src/consolidation_executor.rs:486`
```rust
pub fn start_consolidation_worker<S: StorageEngine + 'static, V: VectorIndex + 'static>( collection: Arc<Collection<S, V>>, consolidation_config: ConsolidationConfig, synthesis_config: SynthesisConfig, interval: std::time::Duration, cancel_token: tokio_util::sync::CancellationToken, ) -> tokio::task::JoinHandle<()>
```
Dokumentation: Startet einen Hintergrund-Worker für periodische Konsolidierung.

### Modul `consolidation_locks`

#### `struct ConsolidationNodesGuard`
- Ort: `crates/contextra-cognition/src/consolidation_locks.rs:48`
```rust
pub struct ConsolidationNodesGuard<'a, S: StorageEngine, V: VectorIndex>
```
- Impl-Block `crates/contextra-cognition/src/consolidation_locks.rs:53`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn try_acquire(collection: &'a Collection<S, V>) -> Option<Self>` | `crates/contextra-cognition/src/consolidation_locks.rs:58` | Versucht, das Konsolidierungs-Lock atomar zu erwerben. |
| `pub fn collection(&self) -> &'a Collection<S, V>` | `crates/contextra-cognition/src/consolidation_locks.rs:72` | Liefert eine Referenz auf die geschützte Collection. |
| `pub async fn cascade_invalidate_edges_ordered( &self, superseded_doc_ids: &[DocId], wal_seq: u64, ) -> (Vec<(DocId, Result<CascadeInvalidationReport>)>, Vec<String>)` | `crates/contextra-cognition/src/consolidation_locks.rs:83` | Führt kaskadierende Graph-Edge-Invalidierungen in kanonischer Reihenfolge aus. |

### Modul `context`

#### `struct ContextManager`
- Ort: `crates/contextra-cognition/src/context.rs:53`
```rust
pub struct ContextManager
```
- Impl-Block (Trait `Default`) `crates/contextra-cognition/src/context.rs:60`
- Impl-Block `crates/contextra-cognition/src/context.rs:66`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(budget: TokenBudget) -> Self` | `crates/contextra-cognition/src/context.rs:70` | Erstellt einen neuen ContextManager mit dem angegebenen TokenBudget. |
| `pub fn with_defaults() -> Self` | `crates/contextra-cognition/src/context.rs:80` | Erstellt einen ContextManager mit Standardeinstellungen. |
| `pub fn set_relevance_threshold(&mut self, threshold: f32)` | `crates/contextra-cognition/src/context.rs:87` | Setzt die Mindestrelevanzschwelle für einzubindende Chunks. |
| `pub fn relevance_threshold(&self) -> f32` | `crates/contextra-cognition/src/context.rs:94` | Liefert die aktuelle Mindestrelevanzschwelle. |
| `pub fn prepare_context(&self, mut chunks: Vec<ContextChunk>) -> Result<ContextWindow>` | `crates/contextra-cognition/src/context.rs:104` | Bereitet das finale ContextWindow aus übergebenen Chunks vor. |
| `pub fn estimate_tokens(text: &str) -> usize` | `crates/contextra-cognition/src/context.rs:180` | Schätzt die Token-Anzahl eines Textes heuristisch nach BPE-Regeln. |

#### `struct SpatialFence`
- Ort: `crates/contextra-cognition/src/context.rs:262`
```rust
pub struct SpatialFence
```
- Impl-Block `crates/contextra-cognition/src/context.rs:269`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `region` | `String` | `crates/contextra-cognition/src/context.rs:264` |
| `field_name` | `String` | `crates/contextra-cognition/src/context.rs:266` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(region: impl Into<String>) -> Self` | `crates/contextra-cognition/src/context.rs:273` | Erstellt eine neue SpatialFence für die angegebene Region. |
| `pub fn matches(&self, chunk: &ContextChunk) -> bool` | `crates/contextra-cognition/src/context.rs:283` | Prüft, ob ein Chunk den regionalen Kriterien entspricht. |

### Modul `context_compaction::cleanup`

#### `fn cleanup_orphaned_consolidation_intents`
- Ort: `crates/contextra-cognition/src/context_compaction/cleanup.rs:31`
```rust
pub async fn cleanup_orphaned_consolidation_intents<S: StorageEngine>( storage: &S, next_tx: &std::sync::atomic::AtomicU64, ) -> Result<usize>
```
Dokumentation: Aufgerufen beim Öffnen einer Collection / DB. Findet und entfernt verwaiste ConsolidationIntents aus dem WAL/Storage. Verwaiste Intents entstehen wenn consolidate_with_retry() nach commit(intent_key) aber vor dem eigentlichen Dokument-Commit crashed.

### Modul `context_compaction::compactor`

#### `struct ContextCompactor`
- Ort: `crates/contextra-cognition/src/context_compaction/compactor.rs:40`
```rust
pub struct ContextCompactor
```
Dokumentation: Zentrale Kompaktierungs-Engine.
- Impl-Block (Trait `std::fmt::Debug`) `crates/contextra-cognition/src/context_compaction/compactor.rs:46`
- Impl-Block `crates/contextra-cognition/src/context_compaction/compactor.rs:109`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new_with_rng( budget: TokenBudget, strategy: CompactionStrategy, rng: Arc<dyn Rng>, ) -> Self` | `crates/contextra-cognition/src/context_compaction/compactor.rs:118` | Erstellt einen ContextCompactor mit explizit injiziertem RNG. |
| `pub fn new(budget: TokenBudget, strategy: CompactionStrategy) -> Self` | `crates/contextra-cognition/src/context_compaction/compactor.rs:138` | Erstellt einen ContextCompactor mit Standard-RNG (veraltet). |
| `pub fn with_rng(mut self, rng: Arc<dyn Rng>) -> Self` | `crates/contextra-cognition/src/context_compaction/compactor.rs:148` | Setzt den Zufallszahlengenerator für Backoff-Jitter. |
| `pub fn compact(&self, chunks: Vec<ContextChunk>) -> CompactedContext` | `crates/contextra-cognition/src/context_compaction/compactor.rs:164` | Kompaktiert Chunks auf das konfigurierte Token-Budget. |
| `pub async fn consolidate_via_llm( &self, tenant: TenantId, chunks: &[ContextChunk], generator: &(impl LlmTextGenerator + ?Sized), _model: &str, ) -> Result<CompactedContext>` | `crates/contextra-cognition/src/context_compaction/compactor.rs:239` | Konsolidiert Chunks per LLM zu einem einzigen Zusammenfassungs-Chunk. |
| `pub async fn consolidate_with_retry<S, V, G>( collection: &Collection<S, V>, source_doc_ids: &[DocId], target_doc_id: DocId, generator: &G, max_retries: usize, ) -> Result<()> where S: StorageEngine, V: VectorIndex, G: LlmTextGenerator + ?Sized,` | `crates/contextra-cognition/src/context_compaction/compactor.rs:319` | Führt Konsolidierung mit automatischem Retry bei OCC-Konflikten aus (veraltet). |
| `pub async fn consolidate_with_retry_with_rng<S, V, G>( collection: &Collection<S, V>, source_doc_ids: &[DocId], target_doc_id: DocId, generator: &G, max_retries: usize, rng: &dyn Rng, ) -> Result<()> where S: StorageEngine, V: VectorIndex, G: LlmTextGenerator + ?Sized,` | `crates/contextra-cognition/src/context_compaction/compactor.rs:350` | Führt Konsolidierung mit automatischem Retry und injiziertem RNG aus. |

### Modul `context_compaction::session`

#### `struct ConsolidationSession`
- Ort: `crates/contextra-cognition/src/context_compaction/session.rs:41`
```rust
pub struct ConsolidationSession<'a, S: StorageEngine, V: VectorIndex>
```
- Impl-Block `crates/contextra-cognition/src/context_compaction/session.rs:54`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `intent_key` | `Vec<u8>` | `crates/contextra-cognition/src/context_compaction/session.rs:47` |
| `target_id` | `DocId` | `crates/contextra-cognition/src/context_compaction/session.rs:49` |
| `base_tx` | `TxId` | `crates/contextra-cognition/src/context_compaction/session.rs:51` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub async fn start( collection: &'a Collection<S, V>, source_doc_ids: &[DocId], target_id: DocId, ) -> Result<Self>` | `crates/contextra-cognition/src/context_compaction/session.rs:59` | Startet eine neue Konsolidierungs-Session und sichert Versionen der Quelldokumente. |
| `pub async fn validate_occ(&self) -> Result<()>` | `crates/contextra-cognition/src/context_compaction/session.rs:97` | Validiert die optimistische Nebenläufigkeitskontrolle. |
| `pub async fn refresh(&mut self) -> Result<Vec<DocId>>` | `crates/contextra-cognition/src/context_compaction/session.rs:131` |  |
| `pub async fn abort(self) -> Result<()>` | `crates/contextra-cognition/src/context_compaction/session.rs:175` | Bricht die Session ab und entfernt den Intent-Schlüssel. |
| `pub async fn execute<G>(&self, generator: &G) -> Result<()> where G: LlmTextGenerator + ?Sized,` | `crates/contextra-cognition/src/context_compaction/session.rs:194` |  |
| `pub async fn commit( self, target_string_id: &str, embedding: &[f32], summary_content: &str, metadata: Option<serde_json::Value>, ) -> Result<()>` | `crates/contextra-cognition/src/context_compaction/session.rs:269` | Committet das konsolidierte Dokument und löscht Quelldokumente. |
| `pub async fn commit_ref( &self, target_string_id: &str, embedding: &[f32], summary_content: &str, metadata: Option<serde_json::Value>, ) -> Result<()>` | `crates/contextra-cognition/src/context_compaction/session.rs:283` | Committet das konsolidierte Dokument unter Beibehaltung der Session-Referenz. |

### Modul `context_compaction::types`

#### `struct CompactedContext`
- Ort: `crates/contextra-cognition/src/context_compaction/types.rs:45`
```rust
pub struct CompactedContext
```
Dokumentation: Kompaktierter Kontext für LLM-Übergabe.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `retained_chunks` | `Vec<ContextChunk>` | `crates/contextra-cognition/src/context_compaction/types.rs:47` |
| `status_tokens` | `Vec<StatusToken>` | `crates/contextra-cognition/src/context_compaction/types.rs:49` |
| `tokens_used` | `usize` | `crates/contextra-cognition/src/context_compaction/types.rs:51` |
| `source_doc_ids` | `Vec<DocId>` | `crates/contextra-cognition/src/context_compaction/types.rs:53` |

#### `struct StatusToken`
- Ort: `crates/contextra-cognition/src/context_compaction/types.rs:59`
```rust
pub struct StatusToken
```
Dokumentation: Kompakter Stellvertreter für einen oder mehrere kompaktierte Chunks.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `summary` | `String` | `crates/contextra-cognition/src/context_compaction/types.rs:61` |
| `replaced_tokens` | `usize` | `crates/contextra-cognition/src/context_compaction/types.rs:63` |
| `replaced_doc_ids` | `Vec<DocId>` | `crates/contextra-cognition/src/context_compaction/types.rs:65` |

#### `enum CompactionStrategy`
- Ort: `crates/contextra-cognition/src/context_compaction/types.rs:23`
```rust
pub enum CompactionStrategy
```
Dokumentation: Strategie für Context Compaction.

Varianten:

- `Truncate` — `crates/contextra-cognition/src/context_compaction/types.rs:25`
- `Summarize` — `crates/contextra-cognition/src/context_compaction/types.rs:27`
- `StatusToken` — `crates/contextra-cognition/src/context_compaction/types.rs:29`
- `LlmSummarize{` — `crates/contextra-cognition/src/context_compaction/types.rs:31`

### Modul `crate`

#### `mod aggregation_phase`
- Ort: `crates/contextra-cognition/src/lib.rs:49`
```rust
pub mod aggregation_phase
```
Dokumentation: Modul für Aggregationsphase (LeanRAG Stage 3).

#### `mod channel`
- Ort: `crates/contextra-cognition/src/lib.rs:51`
```rust
pub mod channel
```
Dokumentation: Modul für entkoppelte Eingabe- und Ausgabe-Channels (Baukasten-Integration).

#### `mod collection`
- Ort: `crates/contextra-cognition/src/lib.rs:60`
```rust
pub mod collection
```

#### `mod collection_types`
- Ort: `crates/contextra-cognition/src/lib.rs:61`
```rust
pub mod collection_types
```

#### `mod consolidation_executor`
- Ort: `crates/contextra-cognition/src/lib.rs:53`
```rust
pub mod consolidation_executor
```
Dokumentation: Modul für die Ausführung der Konsolidierung.

#### `mod consolidation_locks`
- Ort: `crates/contextra-cognition/src/lib.rs:55`
```rust
pub mod consolidation_locks
```
Dokumentation: Modul für Konsolidierungs-Locks und Kaskaden-Invalidierung.

#### `mod context`
- Ort: `crates/contextra-cognition/src/lib.rs:57`
```rust
pub mod context
```
Dokumentation: Modul für autonome Kontextverwaltung und Token-Budgetierung.

#### `mod context_compaction`
- Ort: `crates/contextra-cognition/src/lib.rs:59`
```rust
pub mod context_compaction
```
Dokumentation: Modul für Kontext-Kompaktierung.

#### `mod decay`
- Ort: `crates/contextra-cognition/src/lib.rs:62`
```rust
pub mod decay
```

#### `mod error`
- Ort: `crates/contextra-cognition/src/lib.rs:43`
```rust
pub mod error
```

#### `mod graph_sink`
- Ort: `crates/contextra-cognition/src/lib.rs:66`
```rust
pub mod graph_sink
```
Dokumentation: Modul für Graph-Senken (Super-Hyperkanten).

#### `mod leanrag_input`
- Ort: `crates/contextra-cognition/src/lib.rs:68`
```rust
pub mod leanrag_input
```
Dokumentation: Modul zur Vorbereitung von LeanRAG-Eingabedaten.

#### `mod maintenance_config`
- Ort: `crates/contextra-cognition/src/lib.rs:70`
```rust
pub mod maintenance_config
```
Dokumentation: Modul für Wartungskonfigurationen.

#### `mod maintenance_scheduler`
- Ort: `crates/contextra-cognition/src/lib.rs:72`
```rust
pub mod maintenance_scheduler
```
Dokumentation: Modul für periodische Wartungsplanung.

#### `mod memory_consolidation`
- Ort: `crates/contextra-cognition/src/lib.rs:74`
```rust
pub mod memory_consolidation
```
Dokumentation: Modul für Gedächtniskonsolidierung (Structural Pass).

#### `mod ports`
- Ort: `crates/contextra-cognition/src/lib.rs:44`
```rust
pub mod ports
```

#### `mod semantic_aggregation_facade`
- Ort: `crates/contextra-cognition/src/lib.rs:77`
```rust
pub mod semantic_aggregation_facade
```
Dokumentation: Modul für die Fassadenschnittstelle zur semantischen Aggregation.

#### `mod synthesis_phase`
- Ort: `crates/contextra-cognition/src/lib.rs:79`
```rust
pub mod synthesis_phase
```
Dokumentation: Modul für generative Wissenssynthese (LLM-Pass).

#### `mod transitivity_veto`
- Ort: `crates/contextra-cognition/src/lib.rs:81`
```rust
pub mod transitivity_veto
```
Dokumentation: Modul für Transitivitäts-Veto-Prüfungen.

### Modul `decay`

#### `struct AdaptiveDecayController`
- Ort: `crates/contextra-cognition/src/decay.rs:61`
```rust
pub struct AdaptiveDecayController
```
Dokumentation: Adaptive Decay Controller für time-weighted Cache-Eviction mit injizierter Clock.
- Impl-Block (Trait `std::fmt::Debug`) `crates/contextra-cognition/src/decay.rs:67`
- Impl-Block `crates/contextra-cognition/src/decay.rs:76`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(config: DecayControllerConfig) -> Self` | `crates/contextra-cognition/src/decay.rs:78` | Erstellt eine neue Instanz des `AdaptiveDecayController` mit der gegebenen Konfiguration. |
| `pub fn with_clock(config: DecayControllerConfig, clock: Arc<dyn Clock>) -> Self` | `crates/contextra-cognition/src/decay.rs:87` | Erstellt eine neue Instanz des `AdaptiveDecayController` mit injizierter Clock. |
| `pub fn with_base_half_life_ms(mut self, base_ms: u64) -> Self` | `crates/contextra-cognition/src/decay.rs:96` | Setzt die Basis-Halbwertszeit in Millisekunden für Wall-Clock-Decay. |
| `pub fn with_defaults() -> Self` | `crates/contextra-cognition/src/decay.rs:102` | Initialisiert den Controller mit den Standardwerten der Spezifikation. |
| `pub fn set_clock(&mut self, clock: Arc<dyn Clock>)` | `crates/contextra-cognition/src/decay.rs:107` | Setzt oder aktualisiert die injizierte Clock. |
| `pub fn config(&self) -> &DecayControllerConfig` | `crates/contextra-cognition/src/decay.rs:112` | Konfiguration des Controllers abrufen. |
| `pub fn system_temperature(&self, inputs: &DecaySignalInputs) -> f32` | `crates/contextra-cognition/src/decay.rs:118` | Berechnet die aktuelle Systemtemperatur basierend auf Tombstone-Verhältnis und Abfragelast. |
| `pub fn effective_half_life(&self, temperature: f32) -> f32` | `crates/contextra-cognition/src/decay.rs:125` | Berechnet die temperaturangepasste effektive Halbwertszeit in Transaktionszyklen. |
| `pub fn effective_half_life_ms(&self, temperature: f32) -> f32` | `crates/contextra-cognition/src/decay.rs:131` | Berechnet die temperaturangepasste effektive Halbwertszeit in Millisekunden. |
| `pub fn effective_score( &self, base_score: f32, elapsed_tx: u64, inputs: &DecaySignalInputs, ) -> f32` | `crates/contextra-cognition/src/decay.rs:137` | Berechnet den aktuellen abgewerteten Score eines Objekts unter Berücksichtigung von Alter und Systemtemperatur. |
| `pub fn effective_score_wall_clock( &self, base_score: f32, created_at_ms: u64, inputs: &DecaySignalInputs, ) -> f32` | `crates/contextra-cognition/src/decay.rs:150` | Berechnet den abgewerteten Score basierend auf der Zeitdifferenz zur injizierten Clock. |
| `pub fn should_evict( &self, base_score: f32, elapsed_tx: u64, inputs: &DecaySignalInputs, ) -> bool` | `crates/contextra-cognition/src/decay.rs:170` | Prüft, ob ein Eintrag aufgrund seines verfallenen Scores unter den Eviktions-Schwellenwert gefallen ist. |
| `pub fn should_evict_wall_clock( &self, base_score: f32, created_at_ms: u64, inputs: &DecaySignalInputs, ) -> bool` | `crates/contextra-cognition/src/decay.rs:181` | Prüft, ob ein Eintrag unter Verwendung der injizierten Clock eviktiert werden soll. |

#### `struct DecayControllerConfig`
- Ort: `crates/contextra-cognition/src/decay.rs:40`
```rust
pub struct DecayControllerConfig
```
Dokumentation: Konfiguration für den Adaptive Decay Controller.
- Impl-Block (Trait `Default`) `crates/contextra-cognition/src/decay.rs:49`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `kappa` | `f32` | `crates/contextra-cognition/src/decay.rs:42` |
| `base_half_life_tx` | `u64` | `crates/contextra-cognition/src/decay.rs:44` |
| `eviction_threshold` | `f32` | `crates/contextra-cognition/src/decay.rs:46` |

#### `struct DecaySignalInputs`
- Ort: `crates/contextra-cognition/src/decay.rs:16`
```rust
pub struct DecaySignalInputs
```
Dokumentation: Inputs für den Adaptive Decay Controller.
- Impl-Block (Trait `Default`) `crates/contextra-cognition/src/decay.rs:27`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `tombstone_ratio` | `f32` | `crates/contextra-cognition/src/decay.rs:18` |
| `query_load_inverse` | `f32` | `crates/contextra-cognition/src/decay.rs:20` |
| `w_tombstone` | `f32` | `crates/contextra-cognition/src/decay.rs:22` |
| `w_query` | `f32` | `crates/contextra-cognition/src/decay.rs:24` |

### Modul `engine_local`

#### `mod collection`
- Ort: `crates/contextra-cognition/src/engine_local.rs:14`
```rust
pub mod collection
```

#### `mod decay_controller`
- Ort: `crates/contextra-cognition/src/engine_local.rs:19`
```rust
pub mod decay_controller
```

#### `mod transaction`
- Ort: `crates/contextra-cognition/src/engine_local.rs:23`
```rust
pub mod transaction
```

### Modul `error`

#### `enum CognitionError`
- Ort: `crates/contextra-cognition/src/error.rs:21`
```rust
pub enum CognitionError
```
Dokumentation: Lokaler Fehlertyp für Kognitions-, Konsolidierungs- und Kompaktierungsoperationen. Garantiert strukturierte Fehlerbehandlung innerhalb des Bausteins ohne direkte Abhängigkeit von Workspace-Fehlern im Kern.

Varianten:

- `InvalidInput(String)` — `crates/contextra-cognition/src/error.rs:24`
- `PolicyViolation(String)` — `crates/contextra-cognition/src/error.rs:28`
- `Internal(String)` — `crates/contextra-cognition/src/error.rs:32`
- `BudgetExceeded{` — `crates/contextra-cognition/src/error.rs:36`
- `StaleRead(String)` — `crates/contextra-cognition/src/error.rs:45`
- `NotFound(String)` — `crates/contextra-cognition/src/error.rs:49`
- `Serialization(String)` — `crates/contextra-cognition/src/error.rs:53`
- `Transaction(String)` — `crates/contextra-cognition/src/error.rs:57`

### Modul `graph_sink`

#### `struct CsrGraphSuperEdgeSink`
- Ort: `crates/contextra-cognition/src/graph_sink.rs:39`
```rust
pub struct CsrGraphSuperEdgeSink<'a>
```
Dokumentation: Gepufferte Senke zum Schreiben synthetischer Hyperkanten und Tombstone-Anfragen in den CsrGraph.
- Impl-Block `crates/contextra-cognition/src/graph_sink.rs:46`
- Impl-Block (Trait `SuperEdgeSink`) `crates/contextra-cognition/src/graph_sink.rs:60`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(graph: &'a CsrGraph, wal_tx: TxId) -> Self` | `crates/contextra-cognition/src/graph_sink.rs:50` | Erstellt eine neue CsrGraphSuperEdgeSink gebunden an den Graphen und die WAL-Transaktion. |

### Modul `leanrag_input`

#### `struct LeanRagInputs`
- Ort: `crates/contextra-cognition/src/leanrag_input.rs:44`
```rust
pub struct LeanRagInputs
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `nodes` | `Vec<AggregationNode>` | `crates/contextra-cognition/src/leanrag_input.rs:46` |
| `edges` | `Vec<AggregationEdge>` | `crates/contextra-cognition/src/leanrag_input.rs:48` |

#### `fn build_leanrag_inputs`
- Ort: `crates/contextra-cognition/src/leanrag_input.rs:69`
```rust
pub fn build_leanrag_inputs<S: StorageEngine, V: VectorIndex>( collection: &Collection<S, V>, turns: &[(DocId, Vec<f32>)], max_nodes: usize, ) -> LeanRagInputs
```
Dokumentation: Baut LeanRagInputs aus Turns und Graphendaten deterministisch auf.

#### `const DEFAULT_MAX_LEANRAG_NODES`
- Ort: `crates/contextra-cognition/src/leanrag_input.rs:35`
```rust
pub const DEFAULT_MAX_LEANRAG_NODES: usize = 10_000
```
Dokumentation: Standard-Obergrenze für LeanRAG-Knoten.

### Modul `maintenance_config`

#### `struct MaintenanceConfig`
- Ort: `crates/contextra-cognition/src/maintenance_config.rs:52`
```rust
pub struct MaintenanceConfig
```
- Impl-Block (Trait `Default`) `crates/contextra-cognition/src/maintenance_config.rs:100`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `tick_interval_secs` | `u64` | `crates/contextra-cognition/src/maintenance_config.rs:54` |
| `decay_enabled` | `bool` | `crates/contextra-cognition/src/maintenance_config.rs:59` |
| `decay_config` | `DecayControllerConfig` | `crates/contextra-cognition/src/maintenance_config.rs:62` |
| `edge_reinforcement` | `crate::graph_local::EdgeReinforcementConfig` | `crates/contextra-cognition/src/maintenance_config.rs:67` |
| `percolation_enabled` | `bool` | `crates/contextra-cognition/src/maintenance_config.rs:71` |
| `percolation` | `PercolationConfig` | `crates/contextra-cognition/src/maintenance_config.rs:74` |
| `replicator_enabled` | `bool` | `crates/contextra-cognition/src/maintenance_config.rs:78` |
| `replicator_lr` | `f32` | `crates/contextra-cognition/src/maintenance_config.rs:80` |
| `coherence_bonus_beta` | `f32` | `crates/contextra-cognition/src/maintenance_config.rs:84` |
| `background_consolidation_enabled` | `bool` | `crates/contextra-cognition/src/maintenance_config.rs:89` |
| `background_consolidation_episode_threshold` | `usize` | `crates/contextra-cognition/src/maintenance_config.rs:92` |

#### `struct PercolationConfig`
- Ort: `crates/contextra-cognition/src/maintenance_config.rs:28`
```rust
pub struct PercolationConfig
```
Dokumentation: Fallback-Konfiguration für Perkolationsprüfungen.
- Impl-Block (Trait `Default`) `crates/contextra-cognition/src/maintenance_config.rs:35`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `critical_threshold` | `f32` | `crates/contextra-cognition/src/maintenance_config.rs:29` |
| `rebonding_similarity` | `f32` | `crates/contextra-cognition/src/maintenance_config.rs:30` |
| `max_new_edges_per_pass` | `usize` | `crates/contextra-cognition/src/maintenance_config.rs:31` |

### Modul `maintenance_scheduler`

#### `struct MaintenanceScheduler`
- Ort: `crates/contextra-cognition/src/maintenance_scheduler.rs:48`
```rust
pub struct MaintenanceScheduler<S: StorageEngine, V: VectorIndex>
```
- Impl-Block `crates/contextra-cognition/src/maintenance_scheduler.rs:58`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new( config: MaintenanceConfig, collection: Arc<Collection<S, V>>, consolidation_config: ConsolidationConfig, ) -> Self` | `crates/contextra-cognition/src/maintenance_scheduler.rs:62` | Erstellt eine neue Instanz des MaintenanceScheduler. |
| `pub fn with_clock(mut self, clock: Arc<dyn Clock>) -> Self` | `crates/contextra-cognition/src/maintenance_scheduler.rs:81` | Setzt die Uhr für den Scheduler (Clock-Port Injektion nach Regel 3). |
| `pub fn with_edge_reinforcement_buffer( mut self, buffer: Arc<crate::graph_local::EdgeReinforcementBuffer>, ) -> Self` | `crates/contextra-cognition/src/maintenance_scheduler.rs:90` | Setzt den optionalen Puffer für Kantenverstärkung. |
| `pub fn active_agent_sessions(&self) -> usize` | `crates/contextra-cognition/src/maintenance_scheduler.rs:101` | Liefert die Anzahl aktuell aktiver Agenten-Sessions. |
| `pub fn increment_active_sessions(&self) -> usize` | `crates/contextra-cognition/src/maintenance_scheduler.rs:108` | Erhöht die Anzahl aktiver Agenten-Sessions. |
| `pub fn decrement_active_sessions(&self) -> usize` | `crates/contextra-cognition/src/maintenance_scheduler.rs:115` | Verringert die Anzahl aktiver Agenten-Sessions. |
| `pub fn start( self: Arc<Self>, cancel_token: tokio_util::sync::CancellationToken, ) -> tokio::task::JoinHandle<()>` | `crates/contextra-cognition/src/maintenance_scheduler.rs:124` | Startet den Scheduler in einem eigenen Tokio-Hintergrund-Task. |
| `pub async fn run_tick(&self)` | `crates/contextra-cognition/src/maintenance_scheduler.rs:302` | Führt einen einzelnen sequenziellen Wartungs-Tick aus. |

### Modul `memory_consolidation`

#### `struct CommunityStabilityTracker`
- Ort: `crates/contextra-cognition/src/memory_consolidation.rs:401`
```rust
pub struct CommunityStabilityTracker
```
Dokumentation: Verfolgt die Stabilität von Graph-Communities über Zyklen hinweg.
- Impl-Block `crates/contextra-cognition/src/memory_consolidation.rs:405`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-cognition/src/memory_consolidation.rs:406` |  |
| `pub fn observe(&mut self, community_members_hash: u64) -> u32` | `crates/contextra-cognition/src/memory_consolidation.rs:413` | Registriert die Beobachtung einer Community und liefert die aktualisierte Anzahl aufeinanderfolgender Zyklen. |
| `pub fn reset_if_absent(&mut self, currently_observed: &AHashSet<u64>)` | `crates/contextra-cognition/src/memory_consolidation.rs:420` | Setzt nicht mehr beobachtete Communities zurück/entfernt sie aus der Historie. |

#### `struct ConsolidationConfig`
- Ort: `crates/contextra-cognition/src/memory_consolidation.rs:55`
```rust
pub struct ConsolidationConfig
```
Dokumentation: Konfiguration für den Structural Consolidation Pass.
- Impl-Block (Trait `Default`) `crates/contextra-cognition/src/memory_consolidation.rs:72`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `min_turns_per_segment` | `usize` | `crates/contextra-cognition/src/memory_consolidation.rs:57` |
| `max_turns_per_segment` | `usize` | `crates/contextra-cognition/src/memory_consolidation.rs:59` |
| `segment_cohesion_threshold` | `f32` | `crates/contextra-cognition/src/memory_consolidation.rs:63` |
| `near_duplicate_cosine_threshold` | `f32` | `crates/contextra-cognition/src/memory_consolidation.rs:67` |
| `aggregation_config` | `crate::aggregation_phase::AggregationConfig` | `crates/contextra-cognition/src/memory_consolidation.rs:69` |

#### `struct ConsolidationPhaseResult`
- Ort: `crates/contextra-cognition/src/memory_consolidation.rs:101`
```rust
pub struct ConsolidationPhaseResult
```
Dokumentation: Ergebnis des Structural Consolidation Pass.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `segments_created` | `usize` | `crates/contextra-cognition/src/memory_consolidation.rs:103` |
| `duplicates_superseded` | `Vec<DocId>` | `crates/contextra-cognition/src/memory_consolidation.rs:105` |
| `cascade_edge_invalidations_needed` | `Vec<DocId>` | `crates/contextra-cognition/src/memory_consolidation.rs:109` |
| `cascade_errors` | `Vec<String>` | `crates/contextra-cognition/src/memory_consolidation.rs:111` |

#### `struct MetaChunk`
- Ort: `crates/contextra-cognition/src/memory_consolidation.rs:430`
```rust
pub struct MetaChunk
```
Dokumentation: Generativ synthetisierter Wissens-Chunk mit Provenienz.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `content` | `String` | `crates/contextra-cognition/src/memory_consolidation.rs:432` |
| `abstracts_from` | `Vec<DocId>` | `crates/contextra-cognition/src/memory_consolidation.rs:434` |
| `source_community_hash` | `u64` | `crates/contextra-cognition/src/memory_consolidation.rs:436` |
| `created_at_tx` | `TxId` | `crates/contextra-cognition/src/memory_consolidation.rs:438` |
| `llm_model_id` | `String` | `crates/contextra-cognition/src/memory_consolidation.rs:440` |

#### `struct PreCondensedInput`
- Ort: `crates/contextra-cognition/src/memory_consolidation.rs:668`
```rust
pub struct PreCondensedInput
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `tenant_id` | `contextra_types::TenantId` | `crates/contextra-cognition/src/memory_consolidation.rs:669` |
| `source_label` | `String` | `crates/contextra-cognition/src/memory_consolidation.rs:670` |
| `text_digest` | `String` | `crates/contextra-cognition/src/memory_consolidation.rs:671` |
| `token_count` | `usize` | `crates/contextra-cognition/src/memory_consolidation.rs:672` |
| `provenance` | `Vec<DocId>` | `crates/contextra-cognition/src/memory_consolidation.rs:673` |

#### `struct SynthesisConfig`
- Ort: `crates/contextra-cognition/src/memory_consolidation.rs:375`
```rust
pub struct SynthesisConfig
```
Dokumentation: Konfiguration für den Generative Synthesis Pass.
- Impl-Block (Trait `Default`) `crates/contextra-cognition/src/memory_consolidation.rs:386`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `min_community_size` | `usize` | `crates/contextra-cognition/src/memory_consolidation.rs:377` |
| `stability_cycles_required` | `u32` | `crates/contextra-cognition/src/memory_consolidation.rs:379` |
| `max_llm_calls_per_cycle` | `u32` | `crates/contextra-cognition/src/memory_consolidation.rs:381` |
| `min_grounding_score` | `Option<f32>` | `crates/contextra-cognition/src/memory_consolidation.rs:383` |

#### `struct SynthesisPhaseResult`
- Ort: `crates/contextra-cognition/src/memory_consolidation.rs:447`
```rust
pub struct SynthesisPhaseResult
```
Dokumentation: Ergebnis des Generative Synthesis Pass.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `synthesized` | `Vec<MetaChunk>` | `crates/contextra-cognition/src/memory_consolidation.rs:449` |
| `deferred_community_hashes` | `Vec<u64>` | `crates/contextra-cognition/src/memory_consolidation.rs:451` |

#### `struct TurnSegment`
- Ort: `crates/contextra-cognition/src/memory_consolidation.rs:88`
```rust
pub struct TurnSegment
```
Dokumentation: Semantisch zusammenhängendes Segment aufeinanderfolgender Turns.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `turn_ids` | `Vec<DocId>` | `crates/contextra-cognition/src/memory_consolidation.rs:90` |
| `representative_embedding` | `Vec<f32>` | `crates/contextra-cognition/src/memory_consolidation.rs:94` |

#### `fn compact_segment_via_context_compactor`
- Ort: `crates/contextra-cognition/src/memory_consolidation.rs:847`
```rust
pub fn compact_segment_via_context_compactor( segment: &TurnSegment, compactor: &ContextCompactor, chunks: &[ContextChunk], ) -> CompactedContext
```
Dokumentation: Kompaktiert ein einzelnes Segment via ContextCompactor.

#### `fn compute_community_hash`
- Ort: `crates/contextra-cognition/src/memory_consolidation.rs:457`
```rust
pub fn compute_community_hash(member_doc_ids: &[DocId]) -> u64
```
Dokumentation: Berechnet einen deterministischen 64-Bit-Hash für Member-DocIds.

#### `fn detect_near_duplicates`
- Ort: `crates/contextra-cognition/src/memory_consolidation.rs:281`
```rust
pub fn detect_near_duplicates(turns: &[(DocId, Vec<f32>)], threshold: f32) -> Vec<(DocId, DocId)>
```

#### `fn group_turns_into_segments`
- Ort: `crates/contextra-cognition/src/memory_consolidation.rs:177`
```rust
pub fn group_turns_into_segments( turns: &[(DocId, Vec<f32>)], config: &ConsolidationConfig, ) -> Vec<TurnSegment>
```
Dokumentation: Gruppiert benachbarte Turns via Sliding-Window-Clustering.

#### `fn intake_pre_condensed_inputs`
- Ort: `crates/contextra-cognition/src/memory_consolidation.rs:808`
```rust
pub fn intake_pre_condensed_inputs( target_tenant_id: &contextra_types::TenantId, inputs: &[PreCondensedInput], validator: Option<&dyn ResponseGroundingValidator>, min_grounding_score: Option<f32>, ) -> contextra_types::Result<Vec<TurnSegment>>
```

#### `fn run_consolidation_pass`
- Ort: `crates/contextra-cognition/src/memory_consolidation.rs:322`
```rust
pub fn run_consolidation_pass( turns: &[(DocId, Vec<f32>)], config: &ConsolidationConfig, ) -> ConsolidationPhaseResult
```
Dokumentation: Orchestriert den deterministischen Structural Consolidation Pass.

#### `fn run_structural_synthesis_pass`
- Ort: `crates/contextra-cognition/src/memory_consolidation.rs:598`
```rust
pub async fn run_structural_synthesis_pass( stable_communities: &[(u64, Vec<DocId>)], source_texts: &AHashMap<DocId, String>, llm: &dyn LlmTextGenerator, config: &SynthesisConfig, validator: Option<&dyn ResponseGroundingValidator>, ) -> contextra_types::Result<SynthesisPhaseResult>
```
Dokumentation: Führt den Syntheselauf über stabile Graph-Communities aus.

#### `const MAX_PRE_CONDENSED_DIGEST_LEN`
- Ort: `crates/contextra-cognition/src/memory_consolidation.rs:649`
```rust
pub const MAX_PRE_CONDENSED_DIGEST_LEN: usize = 8192
```
Dokumentation: Maximale Länge für text_digest in PreCondensedInput.

#### `const MAX_PRE_CONDENSED_LABEL_LEN`
- Ort: `crates/contextra-cognition/src/memory_consolidation.rs:654`
```rust
pub const MAX_PRE_CONDENSED_LABEL_LEN: usize = 512
```
Dokumentation: Maximale Länge für source_label in PreCondensedInput.

#### `const MAX_PRE_CONDENSED_PROVENANCE_COUNT`
- Ort: `crates/contextra-cognition/src/memory_consolidation.rs:659`
```rust
pub const MAX_PRE_CONDENSED_PROVENANCE_COUNT: usize = 256
```
Dokumentation: Maximale Anzahl an Provenienz-IDs in PreCondensedInput.

### Modul `ports`

#### `enum CommitIntent`
- Ort: `crates/contextra-cognition/src/ports.rs:21`
```rust
pub enum CommitIntent
```
Dokumentation: Lokaler Ersatztyp für `CommitIntent`, falls die direkte Engine-Bindung entfällt. Hält die für atomare Transaktionsabschlüsse notwendigen Metadaten isoliert von der physischen Speicher-Engine.

Varianten:

- `Pending{` — `crates/contextra-cognition/src/ports.rs:23`
- `Committed` — `crates/contextra-cognition/src/ports.rs:33`
- `Aborted` — `crates/contextra-cognition/src/ports.rs:35`
- `Consolidation{` — `crates/contextra-cognition/src/ports.rs:37`
- `Failed{ reason: String }` — `crates/contextra-cognition/src/ports.rs:43`

#### `trait CollectionOps`
- Ort: `crates/contextra-cognition/src/ports.rs:101`
```rust
pub trait CollectionOps: Send + Sync
```
Dokumentation: Abstrakte Port-Schnittstelle für Collection-Operationen (SPEC S-63). Entkoppelt contextra-cognition von konkreten Storage- und Index-Engines.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn name(&self) -> &str` | `crates/contextra-cognition/src/ports.rs:103` | Gibt den Namen der Collection zurück. |
| `fn namespaced_key(&self, raw: &[u8], key_type: u8) -> Vec<u8>` | `crates/contextra-cognition/src/ports.rs:106` | Erzeugt einen namensraumisolierten Schlüssel für den physischen Speicher. |
| `fn user_key_prefix(&self) -> Vec<u8>` | `crates/contextra-cognition/src/ports.rs:109` | Gibt das Schlüsselpräfix für reguläre Benutzerdaten zurück. |
| `fn consolidation_in_progress(&self) -> bool` | `crates/contextra-cognition/src/ports.rs:112` | Prüft, ob aktuell eine Konsolidierung läuft. |
| `fn dimension(&self) -> usize` | `crates/contextra-cognition/src/ports.rs:115` | Ermittelt die Vektordimension der Collection. |
| `fn allocate_tx(&self) -> Result<TxId>` | `crates/contextra-cognition/src/ports.rs:118` | Allokiert eine monoton steigende Transaktions-ID. |
| `async fn get(&self, id: &str) -> Result<Option<crate::collection_types::StoredDocument>>` | `crates/contextra-cognition/src/ports.rs:121` | Ruft ein Dokument anhand seines Bezeichners ab. |
| `async fn delete(&self, id: &str) -> Result<()>` | `crates/contextra-cognition/src/ports.rs:124` | Löscht ein Dokument anhand seines Bezeichners. |
| `async fn put_kv(&self, key: &str, value: &serde_json::Value) -> Result<()>` | `crates/contextra-cognition/src/ports.rs:127` | Schreibt einen Schlüssel-Wert-Eintrag. |
| `async fn evict_decayed_chunks( &self, controller: &crate::decay::AdaptiveDecayController, limit: usize, ) -> Result<usize>` | `crates/contextra-cognition/src/ports.rs:130` | Führt die zeitliche Abwertung und Bereinigung verfallener Chunks durch. |

#### `trait ConsolidationSink`
- Ort: `crates/contextra-cognition/src/ports.rs:68`
```rust
pub trait ConsolidationSink: Send + Sync
```
Dokumentation: Abstrakte Senke für Konsolidierungsergebnisse. Garantiert, dass konsolidierte Chunks atomar in das Zielsubsystem eingepflegt werden können.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `async fn apply_intent(&self, intent: CommitIntent) -> Result<()>` | `crates/contextra-cognition/src/ports.rs:70` | Wendet einen Commit-Intent atomar an. |

#### `trait DecayPolicy`
- Ort: `crates/contextra-cognition/src/ports.rs:91`
```rust
pub trait DecayPolicy: Send + Sync
```
Dokumentation: Richtlinie zur adaptiven Abwertung zeitlicher Relevanz (Decay Control). Erlaubt anpassbare Abklingfunktionen für Langzeitgedächtnisse.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn compute_decay(&self, elapsed_secs: u64, access_count: u64) -> f32` | `crates/contextra-cognition/src/ports.rs:93` | Berechnet den aktuellen Dämpfungsfaktor für ein Dokument. |

#### `trait DocumentSource`
- Ort: `crates/contextra-cognition/src/ports.rs:57`
```rust
pub trait DocumentSource: Send + Sync
```
Dokumentation: Abstrakte Quelle für Dokumente zur Konsolidierung. Entkoppelt das Laden von Chunks oder Gesprächs-Turns von der konkreten Datenbank- oder Vektordatenbank-Implementierung.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `async fn fetch_turns(&self) -> Result<Vec<(DocId, Vec<f32>)>>` | `crates/contextra-cognition/src/ports.rs:59` | Lädt Turns / Dokumente für die angegebene Kollektion oder den Kontext. |

#### `trait HyperedgeSink`
- Ort: `crates/contextra-cognition/src/ports.rs:78`
```rust
pub trait HyperedgeSink: Send + Sync
```
Dokumentation: Senke für synthetische Hyperkanten und Kaskaden-Invalidierungen. Wird in der semantischen Aggregationsphase (LeanRAG Stage 3) verwendet, um verdichtete Superkanten im Wissensgraphen anzulegen.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn invalidate_edge(&mut self, edge_id: u64)` | `crates/contextra-cognition/src/ports.rs:80` | Invalidiert eine bestehende Hyperkante anhand ihrer ID. |
| `fn write_super_edge(&mut self, edge_data: Vec<u8>)` | `crates/contextra-cognition/src/ports.rs:82` | Schreibt eine neu synthetisierte Hyperkante. |
| `fn commit(&mut self) -> Result<()>` | `crates/contextra-cognition/src/ports.rs:84` | Veröffentlicht alle gesammelten Änderungen atomar. |

### Modul `semantic_aggregation_facade`

#### `fn consolidate_semantic_hyperedges`
- Ort: `crates/contextra-cognition/src/semantic_aggregation_facade.rs:61`
```rust
pub async fn consolidate_semantic_hyperedges<S: StorageEngine, V: VectorIndex>( collection: &Collection<S, V>, embedder: &dyn TextEmbeddingEngine, llm: &dyn LlmTextGenerator, config: &AggregationConfig, ) -> Result<AggregationPhaseResult>
```

### Modul `synthesis_phase`

#### `struct SegmentSynthesisResult`
- Ort: `crates/contextra-cognition/src/synthesis_phase.rs:31`
```rust
pub struct SegmentSynthesisResult
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `synthesized_chunks` | `Vec<SynthesizedChunk>` | `crates/contextra-cognition/src/synthesis_phase.rs:33` |
| `skipped_segments` | `usize` | `crates/contextra-cognition/src/synthesis_phase.rs:35` |

#### `struct SynthesizedChunk`
- Ort: `crates/contextra-cognition/src/synthesis_phase.rs:47`
```rust
pub struct SynthesizedChunk
```
Dokumentation: Ein generativ synthetisierter Wissens-Chunk mit Provenienz.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `content` | `String` | `crates/contextra-cognition/src/synthesis_phase.rs:49` |
| `source_turn_ids` | `Vec<DocId>` | `crates/contextra-cognition/src/synthesis_phase.rs:51` |
| `model_id` | `String` | `crates/contextra-cognition/src/synthesis_phase.rs:53` |

#### `fn run_synthesis_pass`
- Ort: `crates/contextra-cognition/src/synthesis_phase.rs:73`
```rust
pub async fn run_synthesis_pass( segments: &[TurnSegment], segment_texts: &[Vec<String>], synthesizer: &dyn SegmentSynthesizer, min_turns_for_synthesis: usize, ) -> SegmentSynthesisResult
```
Dokumentation: Führt den generativen Syntheselauf über Turn-Segmente aus.

#### `type SynthesisPhaseResult`
- Ort: `crates/contextra-cognition/src/synthesis_phase.rs:41`
```rust
pub type SynthesisPhaseResult = SegmentSynthesisResult
```
Dokumentation: Veralteter Typalias für SegmentSynthesisResult.

### Modul `transitivity_veto`

#### `fn filter_candidates_with_transitivity_veto`
- Ort: `crates/contextra-cognition/src/transitivity_veto.rs:135`
```rust
pub fn filter_candidates_with_transitivity_veto( turns: &[(DocId, Vec<f32>)], threshold: f32, agg_cfg: &AggregationConfig, ) -> Vec<(DocId, DocId)>
```

#### `fn validate_transitivity_veto`
- Ort: `crates/contextra-cognition/src/transitivity_veto.rs:47`
```rust
pub fn validate_transitivity_veto( node_a: &AggregationNode, node_b: &AggregationNode, node_c: &AggregationNode, threshold: f32, ) -> bool
```

## 9. Impl-Blöcke ohne Typ im selben Crate

- `impl From<CognitionError> for contextra_types::ContextraError` — `crates/contextra-cognition/src/error.rs:67` (0 pub Methoden)
