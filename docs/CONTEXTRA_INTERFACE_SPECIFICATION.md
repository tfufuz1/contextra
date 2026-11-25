# Contextra — Globale Mikrofeingranulare Schnittstellenspezifikation

**Repository:** `contextra` (Workspace-Root) · **Generiert am:** 2026-11-05
**Erzeugt durch:** `scripts/generate_interface_spec.py` (automatisierte, reproduzierbare Vollanalyse aller Workspace-Crates)
**Methodik:** Statische Extraktion aller `pub`-Symbole (Traits, Structs, Enums, Funktionen, Typ-Aliase, Konstanten) aus jedem `src/**/*.rs` je Crate, angereichert mit den Original-Doc-Kommentaren (`///`, `//!`), den `Cargo.toml`-Metadaten (Abhängigkeiten, Feature-Flags) und der `lib.rs`-Re-Export-Oberfläche. Test-Module (`#[cfg(test)] mod tests`) sind ausgeschlossen.

> **Hinweis zur Reproduzierbarkeit:** Dieses Dokument wird bei jedem Lauf von `scripts/generate_interface_spec.py` vollständig neu aus dem aktuellen Quellcode-Stand erzeugt. Es spiegelt exakt den Commit wider, gegen den es generiert wurde — bei Architekturänderungen (neue Crates, geänderte Traits/Ports) einfach das Skript erneut ausführen.

## Umfang der Analyse

| Kennzahl | Wert |
|---|---|
| Analysierte Fach-Crates (`crates/*`) | 33 |
| Extrahierte öffentliche Symbole gesamt | **3884** |
| — davon Public Traits (Schnittstellen im engeren Sinn) | 85 |
| — davon Public Structs | 610 |
| — davon Public Enums | 138 |
| — davon Public Funktionen | 2809 |
| — davon Typ-Aliase | 32 |
| — davon Konstanten | 210 |

---

## 1. Ring-Architektur (Layering-Modell, DAG-Invariante)

Contextra erzwingt ein striktes 5-Ring-Schichtenmodell. Abhängigkeiten dürfen **ausschließlich von höheren auf niedrigere Ringe** verlaufen (keine Aufwärtskanten, P5-Invariante).

| Ring | Name | Verantwortung | Merkmal |
|---|---|---|---|
| **0** | Foundation & Pure Numerics/Types | Domänentypen, Traits/Ports, Kryptographie, Mathematik, Vektor-/Text-/Graph-Primitive | **Kein `tokio`** — Sync-Kern, async-Schale (P26) |
| **1** | Persistence & Cache | LSM-Tree-Storage, WAL, Block-Cache, KV-Cache-Segmente, Time-Travel-Checkpoints | Async-I/O nur an den Rändern |
| **2** | External Adapters & Sandboxing | Candle/GGUF-, Ollama-, ONNX-Inferenz, WASM-Sandbox | Blätter im Abhängigkeitsgraphen |
| **3** | Orchestration & Cognition | Transaktions-Engine, Collection-Engine, Kompaktierung, Privacy-Gateway, Routing, Agent-Workflows | Geschäfts-/Orchestrierungslogik |
| **4** | Boundary & Composition Roots | Öffentliche Fassade (`contextra`), MCP-Server, Python-Bindings | Einzige Composition Root |

**Unsafe-Inseln:** `contextra-sys`, `contextra-simd`, `contextra-wire` (einzige Crates ohne `#![forbid(unsafe_code)]`).

**Locking-Disziplin:** `collections (RwLock) → kv_locks (KvKeyLocks) → embedder (RwLock)`, Shard-Indizes strikt aufsteigend akquirieren, kein `.await` unter gehaltenem Sync-Lock.

---

## 2. Crate-Inventar nach Ring (Navigationsübersicht)

**Ring 0 — Foundation:** `contextra-types`, `contextra-ports`, `contextra-mvcc`, `contextra-wire`, `contextra-sys`, `contextra-simd`, `contextra-crypto`, `contextra-vector`, `contextra-text`, `contextra-graph`, `contextra-rank`, `contextra-adapt`, `contextra-core`, `contextra-audit-export`, `contextra-avv-generator`, `contextra-license`

**Ring 1 — Persistence & Cache:** `contextra-store`, `contextra-kvcache`, `contextra-checkpoint`

**Ring 2 — External Adapters & Sandboxing:** `contextra-infer-candle`, `contextra-infer-ollama`, `contextra-infer-onnx`, `contextra-sandbox`

**Ring 3 — Orchestration & Cognition:** `contextra-engine`, `contextra-cognition`, `contextra-privacy`, `contextra-router`, `contextra-agent`, `contextra-db`

**Ring 4 — Boundary & Composition Roots:** `contextra`, `contextra-mcp`, `contextra-py`

**Tooling:** `contextra-testkit`

---

## 3. Legende der Crate-Abschnitte

1. **Metadatentabelle** — Ring, Migrationsstatus, Zweck, Unsafe-Deklaration, Abhängigkeiten.
2. **Architekturbeschreibung** — Original-`//!`-Kopfkommentar aus `lib.rs`.
3. **Modul- und Re-Export-Oberfläche** — exakte `pub mod`/`pub use`-Deklarationen; definiert die von außen sichtbare API-Grenze.
4. **Feature-Flags** — Cargo-Features und Aktivierungsketten.
5. **Schnittstellen (Public Traits)** — vollständige Methodensignaturen.
6. **Öffentliche Datentypen** — Structs (Felder) und Enums (Varianten).
7. **Typ-Aliase / Konstanten**.
8. **Öffentliche Funktionen** — vollständige Signaturen mit Kurzbeschreibung.

---

## `contextra-types`

| Attribut | Wert |
|---|---|
| **Ring** | 0 — Ring 0 — Foundation & Pure Numerics/Types |
| **Status** | Fertig |
| **Zweck** | Kanonische Domänenmodelle, IDs, Budgets, Filter-AST und Fehlertypen. |
| **Unsafe-Code** | Nein (`#![forbid(unsafe_code)]`) |
| **Interne Abhängigkeiten** | — |
| **Externe Abhängigkeiten (Auswahl)** | `ahash`, `bincode`, `blake3`, `bytes`, `serde`, `serde_json`, `sha2`, `thiserror` |

### Modul- und Re-Export-Oberfläche (lib.rs)

```rust
pub mod error;
pub mod error_dto;
pub mod model_fingerprint;
pub mod retrieval_strategy;
pub mod schema;
pub mod tenant_scope;
pub mod tombstone;
pub mod types;
pub use types::math;
pub use error::{ContextraError, Result};
pub use error_dto::ContextraErrorDto;
pub use model_fingerprint::ModelFingerprint;
pub use retrieval_strategy::RetrievalStrategy;
pub use schema::{DocIdWidth, ManifestSchemaVersion};
pub use tenant_scope::{TenantScopeViolation, TenantScoped};
pub use tombstone::{SeqBitTombstone, TombstoneSemanticsCheck};
pub use types::*;
```

### Feature-Flags

| Flag | Aktiviert |
|---|---|
| `default` | `[]` |
| `docid-128` | `[]` |
| `auto-extraction-opt-out` | `[]` |

### Schnittstellen — Public Traits (1)

**`TombstoneSemanticsCheck`** — *Trait defining tombstone semantics for index sequence numbers and key-value records. # Bit-Mask Convention (IP-07 / ADR-041) By convention in Contextra, sequence numbers (`u64`) reserve the most significant bit (bit 63) as the `TOMBSTONE_BIT` (`1 << 63`). Whe…* (`src/tombstone.rs`)

```rust
pub trait TombstoneSemanticsCheck {
/// Returns `true` if the sequence number or flags indicate a soft-deleted tombstone.
    fn is_tombstone(seq: u64, flags: u8) -> bool;

    /// Encodes a sequence number as an 8-byte little-endian tombstone payload with `TOMBSTONE_BIT` set.
    fn make_tombstone(seq: u64) -> Vec<u8>;
}
```

### Öffentliche Datentypen — Structs (37)

| Struct | Felder / Form | Beschreibung | Datei |
|---|---|---|---|
| `CollectionId` | `(tuple/unit)` | Internal collection identifier. | `src/types/domain/ids.rs` |
| `ConfigFingerprint` | `{ /// LLM-Modell-ID (z.B. "llama-3.2-3b-instruct") pub model_id: String, /// Quantisierungsgrad als String: "Q4_K_M", "Q8_0", "F16", "BF16". /// EXPLIZIT Teil de… }` | Konfigurations-Fingerabdruck für P8-Kalibrierungs-Integrität. INVARIANTE INV-P8-1: Jede Änderung an einem der Felder MUSS `IsotonicCalibrator::invali… | `src/types/domain/misc.rs` |
| `ContextChunk` | `{ /// Document identifier associated with this chunk. pub doc_id: DocId, /// Text content of the chunk. pub content: String, /// Computed relevance score. pub re… }` | A chunk of context for LLM budget allocation. | `src/types/saos.rs` |
| `ContextWindow` | `{ /// List of included context chunks. pub chunks: Vec<ContextChunk>, /// Total aggregated token count. pub total_tokens: usize, /// Indicates whether context ch… }` | An aggregated context window constrained by a token budget. | `src/types/saos.rs` |
| `ContextraErrorDto` | `{ /// Stable string identifier for the error variant (e.g. `"NotFound"`, `"PolicyViolation"`). pub kind: String, /// Human-readable error message. pub message: S… }` | Serializable data transfer object representing a [`ContextraError`]. Used across IPC and API boundaries (e.g. Tauri frontend IPC) to preserve structu… | `src/error_dto.rs` |
| `DocId` | `(tuple/unit)` | Internal document identifier. Under the default configuration, `DocId` wraps a 64-bit primitive (`u64`). When feature `docid-128` is enabled (ADR-082… | `src/types/domain/ids.rs` |
| `DocId` | `(tuple/unit)` | Internal document identifier (128-bit variant). | `src/types/domain/ids.rs` |
| `Edge` | `{ /// Source entity identifier. pub from: EntityId, /// Destination entity identifier. pub to: EntityId, /// Relationship label. pub label: Arc<str>, /// Numeric… }` | Graph directed edge representation with explicit bitemporal axis separation. | `src/types/domain/misc.rs` |
| `Embedding` | `{ /// Raw vector data elements. pub data: Vec<f32>, }` | Vector embedding representation. | `src/types/domain/document.rs` |
| `Entity` | `{ /// Unique entity ID. pub id: EntityId, /// Canonical human-readable name. pub name: Arc<str>, /// Categorical entity type. pub entity_type: Arc<str>, /// Flex… }` | Represents a canonical node in the knowledge graph. | `src/types/domain/misc.rs` |
| `EntityId` | `(tuple/unit)` | Internal entity identifier for graph nodes. | `src/types/domain/ids.rs` |
| `FusionWeights` | `{ vector: f32, text: f32, graph: f32, /// Reserved for future metadata-signal fusion; currently always 0.0. Do not expose a public constructor parameter for this… }` | Normalized fusion weights for hybrid search. | `src/types/saos.rs` |
| `HybridQuery` | `{ /// Optional full-text query string. pub text_query: Option<String>, /// Optional vector query embedding. pub vector_query: Option<Vec<f32>>, /// Optional star… }` | A unified query traversing multiple index signals. | `src/types/saos.rs` |
| `HybridQueryBuilder` | `{ text_query: Option<String>, vector_query: Option<Vec<f32>>, graph_start_node: Option<String>, graph_strategy: Option<GraphTraversalStrategy>, fusion_weights: O… }` | Builder for HybridQuery to improve DX. | `src/types/saos.rs` |
| `ImportanceScore` | `(tuple/unit)` | Scores the importance of a memory item on a normalized scale of `0.0` to `1.0`. | `src/types/importance.rs` |
| `MemoryImportance` | `{ /// Base LLM or heuristic evaluation score. pub base_score: ImportanceScore, /// Recency decay function strategy. pub decay: DecayFunction, /// Transaction ID… }` | Tracks the importance score and decay parameters of a document. | `src/types/importance.rs` |
| `MemoryLink` | `{ /// The target document ID. pub target: DocId, /// The type of relationship. pub relation: LinkRelation, /// The transaction ID when this link was created. pub… }` | A directional link to another memory chunk in the Zettelkasten. | `src/types/domain/misc.rs` |
| `ModelFingerprint` | `{ /// SHA-256 hash digest over model weight blob and quantization tier string. pub hash: [u8; 32], /// Model identifier or filename (e.g. "llama-3.2-3b-instruct"… }` | Uniquely identifies a model weight file and its quantization tier. | `src/model_fingerprint.rs` |
| `PprConfig` | `{ /// Damping factor (probability of continuing random walk vs restarting). Default: 0.85. pub damping_factor: f32, /// Maximum power-iteration steps before term… }` | Configuration parameters for Personalized PageRank (PPR). | `src/types/domain/misc.rs` |
| `QueryRewriteOutput` | `{ /// Textual reformulation intended for BM25 keyword search. pub text_query: Option<String>, /// Semantic reformulation intended for dense vector embedding sear… }` | Output of a query rewriter containing text, semantic, and graph anchor reformulations. | `src/types/saos.rs` |
| `RerankResult` | `{ /// Rank position after reranking (0-based). pub index: usize, /// Original index in the input candidate array. pub original_index: usize, /// Raw model output… }` | Result of cross-encoder reranking operation for a candidate item. | `src/types/domain/document.rs` |
| `Reservation` | `{ budget: &'a TokenBudget, amount: usize, settled: AtomicBool, }` | RAII reservation structure for guaranteed atomicity & drop-refund against double-spend. | `src/types/budget.rs` |
| `ResourceBudget` | `{ /// Maximum allowed memory usage in bytes. pub memory_limit: u64, }` | Resource budget for memory management. | `src/types/budget.rs` |
| `ResourceTracker` | `{ /// The configured budget. budget: ResourceBudget, /// Current memory usage in bytes. memory_used: std::sync::atomic::AtomicU64, }` | Tracks resource usage against a budget. | `src/types/budget.rs` |
| `SaturatingU16` | `(tuple/unit)` | A saturating wrapper around `u16` designed to safely convert `usize` values into `u16` without silent integer-truncation wraparound. This type exists… | `src/types/saturating.rs` |
| `SaturatingU8` | `(tuple/unit)` | A branch-free newtype for counters that saturate at `u8::MAX` rather than wrapping around (prevents silent truncation). | `src/types/math.rs` |
| `ScoredDocument` | `{ /// Identifier of the scored document. pub doc_id: DocId, /// Relevance or similarity score. pub score: f32, }` | A scored search result. | `src/types/domain/document.rs` |
| `ScoredEntry` | `{ /// Unique result identifier. pub id: String, /// Combined final score after signal fusion. pub final_score: f32, /// Associated entry metadata. pub metadata:… }` | Evaluated result for hybrid/4-signal search. | `src/types/saos.rs` |
| `SeqBitTombstone` | `(tuple/unit)` | Default reference implementation of [`TombstoneSemanticsCheck`]. Uses bit 63 (`TOMBSTONE_BIT = 1 << 63`) in `seq: u64` or bit 0 in `flags: u8` (`flag… | `src/tombstone.rs` |
| `SignalFusionStrategies` | `{ /// Strategy used for vector search signal fusion. pub vector: FusionStrategy, /// Strategy used for full-text search signal fusion. pub text: FusionStrategy,… }` | Strategy configuration mapping per retrieval signal (vector, text, graph). | `src/types/saos.rs` |
| `StepId` | `(tuple/unit)` | — | `src/error.rs` |
| `TenantId` | `(tuple/unit)` | LAYER-BEGRÜNDUNG: TenantId ist Abhängigkeit für KV-Cache-Isolation (Layer 4). In Layer 1+ definiert → zyklische Crate-Abhängigkeit unvermeidbar. INVA… | `src/types/domain/ids.rs` |
| `TenantScoped` | `{ tenant_id: TenantId, value: T, }` | Ein Wrapper-Typ, der einen Wert untrennbar an eine `TenantId` bindet. Macht es auf Typ-Ebene unmöglich, einen Wert ohne explizite Tenant-Zuordnung au… | `src/tenant_scope.rs` |
| `TlHfdParams` | `{ /// Regularization parameter $\sigma > 0$ controlling diffusion scale (default: 0.1). #[serde(default = "default_tl_hfd_sigma")] pub sigma: f32, /// Seed injec… }` | Configuration parameters for Thresholded Local Hyper-Flow Diffusion (TL-HFD, Spec §21.1). # Ring Architecture Defined in `contextra-types` (Ring 0) t… | `src/types/domain/misc.rs` |
| `TokenBudget` | `{ /// Maximum total token limit. pub limit: usize, /// Strategy for calculating usable context window. pub strategy: BudgetStrategy, /// Reserved tokens for syst… }` | Token budget configuration for LLM context management. | `src/types/budget.rs` |
| `TxId` | `(tuple/unit)` | Transaction identifier. | `src/types/domain/ids.rs` |
| `WorkflowState` | `{ /// Associated transaction. pub tx: TxId, /// Agent memory graph state footprint (BLAKE3 hash digest). #[serde(with = "hex")] pub graph_hash: [u8; 32], }` | Defines a frozen workflow state acting as a savepoint. | `src/types/domain/misc.rs` |

### Öffentliche Datentypen — Enums (19)

| Enum | Varianten | Beschreibung | Datei |
|---|---|---|---|
| `AutoExtractionMode` | `/// Automatic entity extraction is enabled. Enabled, /// Automatic entity extraction is disabled. Disabled,` | Mode for automatic OpenIE entity extraction during document ingestion. | `src/types/domain/misc.rs` |
| `BudgetStrategy` | `/// Konservativ: 80% des Limits als sicherer Puffer (Standard) Conservative, /// Aggressiv: 95% des Limits nutzen Aggressive, /// Exakt: Angabe des verfügbaren Fensters direkt Exact(usize),` | Strategie für Token-Budget-Management. | `src/types/budget.rs` |
| `CacheDirective` | `/// Segment is explicitly pinned in memory until TTL expiration or explicit release. Pin { /// Optional maximum duration for which the segment remains pinned. `None` indicates indefinite pinning. ttl…` | Declarative agent directive controlling segment caching, pinning, and eviction lifecycle. | `src/error.rs` |
| `ContextraError` | `// ═══ Core & Logic ═══ /// Internal engine logic error. #[error("Internal error: {0}")] Internal(String), /// Invalid user or API argument input. #[error("Invalid input: {0}")] InvalidInput(String),…` | Unified error type for all `Contextra` operations across the entire workspace. # Non-Exhaustive Variant Guarantee This enum is marked `#[non_exhausti… | `src/error.rs` |
| `DecayFunction` | `/// No recency decay (factor remains 1.0). #[default] None, /// Exponential decay measured in `TxId` distance (not wall-clock time). Exponential { /// Half-life in transaction count. half_life_tx: u6…` | Recency-decay mathematical function for episodic relevance. | `src/types/importance.rs` |
| `DistanceMetric` | `/// Cosine distance (`1.0 - cos(angle)`). #[default] Cosine, /// Euclidean (L2) distance. Euclidean, /// Dot product distance (negated inner product for f32). DotProduct,` | Distance metric for vector comparison. | `src/types/domain/document.rs` |
| `DocIdWidth` | `/// 64-bit DocId format (8-byte BLAKE3 truncation). Bit64 = 64, /// 128-bit DocId format (16-byte BLAKE3 truncation, ADR-082). Bit128 = 128,` | Indicates the DocId bit-width expected or produced by a schema version. | `src/schema.rs` |
| `FilterExpr` | `/// Exact match: field == value Eq { /// Target metadata field name. field: String, /// Expected field value. value: serde_json::Value, }, /// Not equal: field != value Ne { /// Target metadata field…` | Metadata filter expressions for pre/post filtering. | `src/types/filter.rs` |
| `FusionStrategy` | `/// Reciprocal Rank Fusion (default strategy per ADR-003). /// Robust against score scale mismatch and partial signal degradation. #[default] Rrf, /// Score-normalized fusion (CombSUM / Z-Score). ///…` | Strategy used for search result signal fusion. | `src/types/saos.rs` |
| `GraphTraversalStrategy` | `/// Multi-hop BFS traversal with hop score decay (default max_hops = 3). Hops { /// Maximum traversal hop depth. max_hops: usize, }, /// Personalized PageRank power iteration starting from seed nodes…` | Strategy used for graph retrieval in hybrid search queries. | `src/types/saos.rs` |
| `HnswDeletionError` | `/// Target node with doc_id not found in graph. #[error("Node for doc_id {0:?} not found in HNSW index")] NodeNotFound(DocId), /// Graph repair resulted in a disconnected graph component. #[error("Gr…` | Errors that can occur during HNSW graph deletion and repair operations. | `src/error.rs` |
| `LinkRelation` | `/// Target memory provides more detail or context. Elaborates, /// Target memory contradicts this memory. Contradicts, /// Target memory supersedes this memory, replacing its context. Supersedes, ///…` | Relationship types between Zettelkasten memory chunks (A-MEM). | `src/types/domain/misc.rs` |
| `ManifestSchemaVersion` | `/// Schema Version 1 (v0.x legacy 64-bit DocId layout). #[default] V1 = 1, /// Schema Version 2 (ADR-082 128-bit DocId layout). V2 = 2,` | — | `src/schema.rs` |
| `MemoryType` | `/// Episodisches Gedächtnis: Erlebnisse, Unterhaltungen, zeitlich verankerte Ereignisse. /// Retrieval: Zeitliche Nähe und Relevanz zur aktuellen Session. /// Decay: Hohe Recency-Decay-Rate (Informat…` | Klassifiziert den kognitiven Gedächtnistyp einer gespeicherten Einheit. # Persistence Wird als Teil der Dokument-Metadaten serialisiert (JSON-Feld "m… | `src/types/domain/misc.rs` |
| `OnSignalFailure` | `/// Abort hybrid search and return an error on signal failure (Default per Spec Teil 6.5 P1). #[default] Fail, /// Degrade search gracefully using remaining active signals and report failed signals i…` | Strategy for handling actual signal failures during hybrid search orchestration (Spec Teil 6.5). **Semantik**: - `Fail` (Default): If an individual s… | `src/types/saos.rs` |
| `PprAlgorithm` | `/// Auto heuristic dispatch based on seed count (ForwardPush for <= 100 seeds, DensePowerIteration otherwise). #[default] Auto, /// Dense power-iteration algorithm (matrix-vector multiplication over…` | Selection of algorithm strategy for Personalized PageRank (PPR). | `src/types/domain/misc.rs` |
| `RetrievalStrategy` | `/// Vektorbasiertes semantisches Signal. Vector, /// Textbasiertes sparse/BM25 Signal. Text, /// Graphbasiertes Beziehungs-Signal. Graph, /// Hybrides Verbund-Signal. Hybrid, /// Nutzt LeanRAG-aggreg…` | Semantische Retrieval-Strategie für Signal-Fusion (§8/§9). | `src/retrieval_strategy.rs` |
| `Signal` | `/// Dense vector embedding search signal. Vector, /// Lexical full-text BM25 search signal. Text, /// Graph traversal / PageRank signal. Graph,` | Individual retrieval signal (vector, text, graph) in multi-signal hybrid search. | `src/types/saos.rs` |
| `TenantScopeViolation` | `/// Raised when unpacking a `TenantScoped<T>` value with an expected `TenantId` that differs /// from the value's actual bound `TenantId`. #[error("tenant scope mismatch: expected {expected:?}, got {…` | Errors raised when attempting an illegal or mismatched tenant scope unpacking operation. | `src/tenant_scope.rs` |

### Typ-Aliase (3)

| Alias | Definition | Beschreibung |
|---|---|---|
| `QueryOptions` | `type QueryOptions= HybridQuery` | Alias for [`HybridQuery`] representing query options for hybrid retrieval operations (Spec Teil 6.5). |
| `QueryOptionsBuilder` | `type QueryOptionsBuilder= HybridQueryBuilder` | Alias for [`HybridQueryBuilder`] representing a query options builder for hybrid retrieval operations (Spec Teil 6.5). |
| `Result` | `type Result<T> = std::result::Result<T, ContextraError>` | Convenience alias for `Result<T, ContextraError>`. |

### Konstanten (14)

| Konstante | Typ | Beschreibung |
|---|---|---|
| `DEFAULT` | `Self` | Der implizite Default-Mandant für alle bestehenden Single-Tenant-Deployments. |
| `EXPIRY_METADATA_KEY` | `&str` | Reserved metadata key for sequence-based document TTL expiration. |
| `INTERNAL_BASE` | `u64` | Lower bound of the internal system transaction ID range. Exact numeric value: `u64::MAX - 1_000_000` (`18_446_744_073_708_551_615`). Dieser Grenzwert… |
| `INVALID` | `Self` | Invalid tenant identifier sentinel value (`0`). |
| `INVALID` | `Self` | Invalid collection identifier sentinel value (`0`). |
| `INVALID` | `Self` | Invalid or uninitialized transaction identifier sentinel value (`0`). `TxId(0)` is reserved as an uninitialized sentinel or null identifier across th… |
| `MAX` | `Self` | Maximum possible `DocId` value (`u64::MAX`). |
| `MAX` | `Self` | Maximum possible `DocId` value (`u128::MAX`). |
| `MAX_COLLECTION_SEQUENCE` | `u64` | Upper bound of the Collection-sequenced transaction ID range (`10^12`). Transaction allocation sequences managed by `Collection::allocate_tx()` opera… |
| `MAX_SEARCH_K` | `usize` | Maximum number of search results that any hybrid/vector/text search may return. Callers in contextra-mcp and contextra-db both enforce this limit bef… |
| `MIN` | `Self` | Minimum possible `DocId` value (`0`). Note: `DocId(0)` is conventionally treated as a sentinel/null value by the MCP layer (`contextra-mcp/src/lib.rs… |
| `MIN` | `Self` | Minimum possible `DocId` value (`0`). |
| `SYSTEM` | `Self` | SYSTEM tenant identifier (`0`). |
| `TOMBSTONE_BIT` | `u64` | Bit mask for identifying tombstones in sequence numbers. |

### Öffentliche Funktionen (140)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn capability_unsupported( capability: impl Into<String>, reason: impl Into<String>, ) -> Self` | Creates a `CapabilityUnsupported` error. | `src/error.rs` |
| `pub fn check_durability(condition: bool, msg: impl Into<String>) -> Result<()>` | Helper to validate durability configuration, producing `DurabilityConfig` on failure. | `src/error.rs` |
| `pub fn check_vector_reference( found: bool, doc_id: impl Into<String>, index_id: impl Into<String>, ) -> Result<()>` | Helper to validate vector reference existence, producing `OrphanedVectorReference` on failure. | `src/error.rs` |
| `pub fn checksum_mismatch(path: impl Into<String>, block_id: u64) -> Self` | Creates a `ChecksumMismatch` error. | `src/error.rs` |
| `pub fn cross_device_link( source_path: impl Into<String>, target_path: impl Into<String>, ) -> Self` | Creates a `CrossDeviceLink` error. | `src/error.rs` |
| `pub fn durability_config(msg: impl Into<String>) -> Self` | Creates a `DurabilityConfig` error. | `src/error.rs` |
| `pub fn from_kind_and_message(kind: &str, msg: impl Into<String>) -> Self` | Constructs a `ContextraError` from an error kind identifier and detail message. | `src/error.rs` |
| `pub fn invalid_input(msg: impl Into<String>) -> Self` | Creates an `InvalidInput` error from any displayable value. | `src/error.rs` |
| `pub fn is_occ_conflict(&self) -> bool` | Returns `true` if this error represents an optimistic concurrency control (OCC) conflict or stale read. | `src/error.rs` |
| `pub fn kv_quantization(msg: impl Into<String>) -> Self` | Creates a `KvQuantization` error. | `src/error.rs` |
| `pub fn limit_exceeded(limit: usize, context: impl Into<String>) -> Self` | Creates a `LimitExceeded` error. | `src/error.rs` |
| `pub fn model_load(path: impl Into<String>, reason: impl Into<String>) -> Self` | Creates a `ModelLoad` error. | `src/error.rs` |
| `pub fn orphaned_vector_reference( doc_id: impl Into<String>, index_id: impl Into<String>, ) -> Self` | Creates an `OrphanedVectorReference` error. | `src/error.rs` |
| `pub fn pin_budget_exceeded(msg: impl Into<String>) -> Self` | Creates a `PinBudgetExceeded` error. | `src/error.rs` |
| `pub fn snapshot_unsupported_for_signal(strategy: impl Into<String>) -> Self` | Creates a `SnapshotUnsupportedForSignal` error. | `src/error.rs` |
| `pub fn wal_corruption(offset: u64, reason: impl Into<String>) -> Self` | Creates a `WalCorruption` error. | `src/error.rs` |
| `pub fn wal_truncation_detected(expected_hmac: [u8; 32], actual_hmac: [u8; 32]) -> Self` | Creates a `WalTruncationDetected` error. | `src/error.rs` |
| `pub fn new(kind: impl Into<String>, message: impl Into<String>) -> Self` | Creates a new `ContextraErrorDto` with custom kind and message. | `src/error_dto.rs` |
| `pub fn with_details( kind: impl Into<String>, message: impl Into<String>, details: serde_json::Value, ) -> Self` | Creates a new `ContextraErrorDto` with custom kind, message, and details payload. | `src/error_dto.rs` |
| `pub fn new( hash: [u8; 32], model_id: impl Into<String>, quantization: impl Into<String>, ) -> Self` | Creates a new `ModelFingerprint`. | `src/model_fingerprint.rs` |
| `pub fn as_u8(self) -> u8` | Returns the numeric version byte. | `src/schema.rs` |
| `pub fn bytes_len(self) -> usize` | Returns the size in bytes of a single `DocId` value under this width. | `src/schema.rs` |
| `pub fn current() -> Self` | Returns the active compile-time `DocIdWidth`. | `src/schema.rs` |
| `pub fn current_bytes_len() -> usize` | Returns the size in bytes of the active compile-time `DocIdWidth`. | `src/schema.rs` |
| `pub fn doc_id_bytes_len(self) -> usize` | Returns the size in bytes of a single `DocId` value under this schema version's DocId width. | `src/schema.rs` |
| `pub fn doc_id_width(self) -> DocIdWidth` | Returns the corresponding `DocIdWidth` for this schema version. | `src/schema.rs` |
| `pub fn from_u8(version_byte: u8) -> Result<Self>` | Parses a version from a numeric byte indicator. | `src/schema.rs` |
| `pub fn from_u8_checked(version_byte: u8) -> Result<Self>` | Parses and validates build compatibility of a schema version byte. | `src/schema.rs` |
| `pub fn is_compatible_with_current_build(self) -> bool` | Returns `true` if this schema version is compatible with the compile-time DocId feature flags. | `src/schema.rs` |
| `pub fn validate_build_compatibility(self) -> Result<Self>` | Validates that this schema version is compatible with the current compile-time build flags. # Errors Returns `ContextraError::Serialization… | `src/schema.rs` |
| `pub fn into_inner_checked(self, expected: &TenantId) -> Result<T, TenantScopeViolation>` | Gibt den Wert nur zurück, wenn die aufrufende Seite die erwartete `TenantId` nennt — verhindert versehentliches Cross-Tenant-Leaking durch… | `src/tenant_scope.rs` |
| `pub fn map<U>(self, f: impl FnOnce(T) -> U) -> TenantScoped<U>` | Transforms the inner value `T` into `U` using `f` while preserving the bound `TenantId`. | `src/tenant_scope.rs` |
| `pub fn new(tenant_id: TenantId, value: T) -> Self` | Creates a new `TenantScoped` value bound to the specified `TenantId`. | `src/tenant_scope.rs` |
| `pub fn tenant_id(&self) -> &TenantId` | Returns a reference to the bound `TenantId`. | `src/tenant_scope.rs` |
| `pub fn available(&self) -> usize` | Returns tokens still available after subtracting reserved and consumed tokens. | `src/types/budget.rs` |
| `pub fn budget(&self) -> &ResourceBudget` | Returns a reference to the configured resource budget. | `src/types/budget.rs` |
| `pub fn consume(&self, tokens: usize)` | Records `tokens` as consumed, reducing future availability. | `src/types/budget.rs` |
| `pub fn consume_memory(&self, bytes: u64) -> Result<()>` | Attempts to consume `bytes` of memory from the tracked budget. # Errors Returns `Err(ContextraError::MemoryBudgetExceeded)` if consuming `b… | `src/types/budget.rs` |
| `pub fn consumed(&self) -> usize` | Returns total tokens consumed so far. | `src/types/budget.rs` |
| `pub fn effective_limit(&self) -> usize` | — | `src/types/budget.rs` |
| `pub fn for_model(model: &str) -> Self` | Erstellt ein Budget für gängige Modelle. # Maintenance Note Model name patterns use string matching. Explicit model family patterns (e.g. `… | `src/types/budget.rs` |
| `pub fn has_memory_capacity(&self) -> bool` | — | `src/types/budget.rs` |
| `pub fn memory_used(&self) -> u64` | Returns current memory usage in bytes. | `src/types/budget.rs` |
| `pub fn new(max_tokens: usize, reserve_tokens: usize) -> Self` | Creates a new token budget with an exact max limit and reserve tokens. | `src/types/budget.rs` |
| `pub fn new(budget: ResourceBudget) -> Self` | Creates a new ResourceTracker with the given budget. | `src/types/budget.rs` |
| `pub fn refund(&self, tokens: usize)` | Refunds `tokens` back to the available budget if execution fails or consumes less than estimated. | `src/types/budget.rs` |
| `pub fn release_memory(&self, bytes: u64)` | Releases memory from the budget. Uses saturating subtraction to prevent atomic underflow (wrapping to `u64::MAX`) which would permanently b… | `src/types/budget.rs` |
| `pub fn reserve(&self, amount: usize) -> Result<Reservation<'_>>` | Reserviert `amount` Tokens atomar. Schlägt fehl, wenn das verfügbare Budget (limit - reserved - consumed) unterschritten würde. | `src/types/budget.rs` |
| `pub fn settle(self)` | Bestätigt den Verbrauch endgültig. Nach `settle()` erfolgt bei Drop KEINE Rückerstattung mehr. # Compile-time Duplicate Call Prevention `se… | `src/types/budget.rs` |
| `pub fn try_reserve(&self, tokens: usize) -> Result<()>` | Tries to reserve `tokens` from the available budget before step execution. Returns `Err(ContextraError::Internal(...))` if available budget… | `src/types/budget.rs` |
| `pub fn with_reserved(mut self, system_tokens: usize, answer_tokens: usize) -> Self` | Reserviert Token für System-Prompt und Antwort. | `src/types/budget.rs` |
| `pub fn with_strategy(mut self, strategy: BudgetStrategy) -> Self` | Sets the budget strategy. | `src/types/budget.rs` |
| `pub fn as_slice(&self) -> &[f32]` | Returns the vector data as an `f32` slice. | `src/types/domain/document.rs` |
| `pub fn compute(&self, a: &[f32], b: &[f32]) -> Result<f32>` | Computes the distance between two f32 vectors using this metric. | `src/types/domain/document.rs` |
| `pub fn compute_u8(&self, a: &[u8], b: &[u8]) -> Result<u32>` | — | `src/types/domain/document.rs` |
| `pub fn dim(&self) -> usize` | Returns the dimension (length) of the embedding vector. | `src/types/domain/document.rs` |
| `pub fn l2_norm(&self) -> f32` | Computes the Euclidean L2 norm of the vector. | `src/types/domain/document.rs` |
| `pub fn new(data: Vec<f32>) -> Self` | Creates a new vector embedding from `f32` slice or vector. | `src/types/domain/document.rs` |
| `pub fn new(doc_id: DocId, score: f32) -> Self` | Creates a new `ScoredDocument` with document ID and score. | `src/types/domain/document.rs` |
| `pub fn normalize(&self) -> Self` | Returns an L2-unit-normalized clone of this embedding. Guarded against zero and subnormal/near-zero vector norms (`norm < 1e-12`). Vectors… | `src/types/domain/document.rs` |
| `pub fn as_bytes(&self) -> Vec<u8>` | Converts the entity identifier into its string byte representation. | `src/types/domain/ids.rs` |
| `pub fn from_doc_id(doc_id: DocId) -> Self` | Creates an `EntityId` directly from a `DocId`. | `src/types/domain/ids.rs` |
| `pub fn from_key(key: &str) -> Result<Self>` | Derives a `DocId` from a string key using the first 8 bytes (64 bits) of its BLAKE3 hash. # Kollisionssicherheit & Entwurfsentscheidung (AD… | `src/types/domain/ids.rs` |
| `pub fn from_key(key: &str) -> Result<Self>` | Derives a 128-bit `DocId` from a string key using the first 16 bytes (128 bits) of its BLAKE3 hash. # ADR-082 128-Bit BLAKE3 Truncation BLA… | `src/types/domain/ids.rs` |
| `pub fn from_key(key: &str) -> Result<Self>` | Derives an `EntityId` from a string key using the first 8 bytes of its BLAKE3 hash. # Errors Returns `ContextraError::InvalidInput` if `key… | `src/types/domain/ids.rs` |
| `pub fn internal_with_offset(offset: u64) -> Result<Self>` | Returns a new internal/system transaction ID with the given offset from INTERNAL_BASE. | `src/types/domain/ids.rs` |
| `pub fn is_system(self) -> bool` | Returns `true` if this tenant ID is `SYSTEM` (0). | `src/types/domain/ids.rs` |
| `pub fn is_valid_origin(&self) -> bool` | Checks if the transaction ID originates from a valid range per AGT-GRAPH-001. Returns `true` if `self.0 <= MAX_COLLECTION_SEQUENCE` (Collec… | `src/types/domain/ids.rs` |
| `pub fn try_from_internal_offset(offset: u64) -> Result<Self>` | Safely constructs an internal system TxId from an offset relative to `TxId::INTERNAL_BASE`. # Domain Range Separation (ADR-028) Valid syste… | `src/types/domain/ids.rs` |
| `pub fn try_new(id: u64) -> Result<Self>` | Sicherer Konstruktor. Gibt Err wenn id == 0. | `src/types/domain/ids.rs` |
| `pub fn try_new(id: u64) -> Result<Self>` | Creates a new `CollectionId`, ensuring `id != 0`. | `src/types/domain/ids.rs` |
| `pub fn as_metadata_key(&self) -> &'static str` | Gibt den kanonischen Metadaten-Key zurück (für JSON-Serialisierung). | `src/types/domain/misc.rs` |
| `pub fn as_str(&self) -> &'static str` | Returns the canonical metadata key string. | `src/types/domain/misc.rs` |
| `pub fn default_decay(&self) -> crate::types::importance::DecayFunction` | Gibt den Standard-Decay-Typ für diesen Gedächtnistyp zurück. | `src/types/domain/misc.rs` |
| `pub fn default_ttl_tx(&self) -> Option<u64>` | Gibt die empfohlene TTL (in Transaktionen) für Session-scoped Working Memory zurück. None bedeutet kein automatisches Ablaufen. | `src/types/domain/misc.rs` |
| `pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8` | — | `src/types/domain/misc.rs` |
| `pub fn is_enabled(self) -> bool` | Returns `true` if auto extraction mode is enabled. | `src/types/domain/misc.rs` |
| `pub fn new(id: EntityId, name: impl Into<String>, entity_type: impl Into<String>) -> Self` | Creates a new `Entity` with id, name, and type. | `src/types/domain/misc.rs` |
| `pub fn new(from: EntityId, to: EntityId, label: impl Into<String>) -> Self` | Creates a new `Edge` between source and target entities with a label. | `src/types/domain/misc.rs` |
| `pub fn new( model_id: impl Into<String>, quantization: impl Into<String>, prompt_template: &str, temperature: f32, ) -> Self` | Erstellt einen neuen `ConfigFingerprint`. | `src/types/domain/misc.rs` |
| `pub fn serialize<S>(bytes: &[u8; 32], serializer: S) -> Result<S::Ok, S::Error> where S: Serializer,` | — | `src/types/domain/misc.rs` |
| `pub fn temperature(&self) -> f32` | Extrahiert Temperatur als f32 (verlustfrei da via to_bits gespeichert). | `src/types/domain/misc.rs` |
| `pub fn threshold(&self) -> f32` | Extrahiert Schwellenwert/Threshold als f32 (verlustfrei da via to_bits gespeichert). | `src/types/domain/misc.rs` |
| `pub fn try_new( id: EntityId, name: impl Into<String>, entity_type: impl Into<String>, ) -> Result<Self>` | Creates a new `Entity`, validating that `name` and `entity_type` are non-empty. # Errors Returns `ContextraError::InvalidInput` if `name` o… | `src/types/domain/misc.rs` |
| `pub fn try_new( from: EntityId, to: EntityId, label: impl Into<String>, weight: f32, ) -> Result<Self>` | Creates a new `Edge`, validating non-empty label and finite non-negative weight. # Errors Returns `ContextraError::InvalidInput` if `label`… | `src/types/domain/misc.rs` |
| `pub fn with_business_validity(mut self, from: Option<i64>, to: Option<i64>) -> Self` | Sets business validity window (business time in Unix ms) on the edge. | `src/types/domain/misc.rs` |
| `pub fn with_source_doc_id(mut self, doc_id: DocId) -> Self` | Sets the optional source document ID from which this edge was derived for provenance tracking. | `src/types/domain/misc.rs` |
| `pub fn with_threshold(mut self, threshold: f32) -> Self` | Setzt die Schwellenwert-Komponente (Threshold) für diesen Fingerabdruck (Builder Pattern). | `src/types/domain/misc.rs` |
| `pub fn with_tx_validity(self, from: Option<TxId>, to: Option<TxId>) -> Self` | Sets transaction validity window (system time / MVCC) on the edge. | `src/types/domain/misc.rs` |
| `pub fn with_validity(mut self, from: Option<TxId>, to: Option<TxId>) -> Self` | Sets transaction validity window on the edge (alias for `with_tx_validity` for backward compatibility). | `src/types/domain/misc.rs` |
| `pub fn with_weight(mut self, weight: f32) -> Self` | Sets a custom weight on the edge. | `src/types/domain/misc.rs` |
| `pub fn evaluate(&self, metadata: &serde_json::Value) -> bool` | Evaluates the filter expression against a JSON metadata object. | `src/types/filter.rs` |
| `pub fn decay_factor(&self, created_at_tx: TxId, now_tx: TxId) -> f32` | Pure non-failing decay factor calculation. Returns a factor in `[0.0, 1.0]`. # Invarianten & Edge Cases - If `now_tx < created_at_tx` (e.g.… | `src/types/importance.rs` |
| `pub fn decay_factor_wallclock( &self, created_at_epoch_secs: u64, now_epoch_secs: u64, created_at_tx: TxId, now_tx: TxId, ) -> f32` | Wall-clock based decay factor calculation. Returns a factor in `[0.0, 1.0]` based on elapsed wall-clock seconds. For non-WallClockExponenti… | `src/types/importance.rs` |
| `pub fn effective_score(&self, now_tx: TxId) -> f32` | Computes the effective importance score: `base_score * decay_factor`. # Zero-Panic Guarantee Always returns a valid `f32` in `[0.0, 1.0]`. | `src/types/importance.rs` |
| `pub fn new(raw: f32) -> Self` | Creates a new `ImportanceScore` clamped strictly within `[0.0, 1.0]`. # Zero-Panic Guarantee NaN inputs return `ImportanceScore(0.0)`. Posi… | `src/types/importance.rs` |
| `pub fn new(base_score: ImportanceScore, decay: DecayFunction, created_at_tx: TxId) -> Self` | Creates a new `MemoryImportance` configuration. | `src/types/importance.rs` |
| `pub fn value(&self) -> f32` | Returns the raw `f32` importance score value. | `src/types/importance.rs` |
| `pub fn build(self) -> Result<HybridQuery>` | Builds the `HybridQuery` instance. # Errors Returns `Err` if query parameters or fusion weights fail validation. | `src/types/saos.rs` |
| `pub fn builder() -> HybridQueryBuilder` | Creates a new `HybridQueryBuilder`. | `src/types/saos.rs` |
| `pub fn combined_text_owned(&self) -> String` | Kombinierten Text für BM25-Indizierung und Embedding. Entspricht dem etablierten Contextual-BM25-Schema: prefix + "\n\n" + content. Allokie… | `src/types/saos.rs` |
| `pub fn combined_token_count(&self) -> usize` | Kombinierter Token-Count (Heuristik: chars/4 ≈ Tokens). Verwendet für Budget-Management in `contextra-db/src/context.rs`. | `src/types/saos.rs` |
| `pub fn community_boosted(entity_id: EntityId) -> Result<Self>` | Creates a `HybridQuery` with community context boosting. | `src/types/saos.rs` |
| `pub fn filtered(filter: FilterExpr) -> Result<Self>` | Creates a `HybridQuery` with a metadata filter expression. | `src/types/saos.rs` |
| `pub fn graph(&self) -> f32` | Returns the graph signal weight component. | `src/types/saos.rs` |
| `pub fn graph_search( start_node: impl Into<String>, strategy: GraphTraversalStrategy, ) -> Result<Self>` | Creates a `HybridQuery` configured for graph traversal search. | `src/types/saos.rs` |
| `pub fn has_context_prefix(&self) -> bool` | Gibt true zurück wenn contextual_prefix gesetzt und nicht leer. | `src/types/saos.rs` |
| `pub fn into_query_builder(self) -> HybridQueryBuilder` | Converts this query rewrite output into a [`HybridQueryBuilder`]. | `src/types/saos.rs` |
| `pub fn is_empty(&self) -> bool` | Returns `true` if all reformulation components are empty or None. | `src/types/saos.rs` |
| `pub fn memory_filtered(types: Vec<MemoryType>) -> Result<Self>` | Creates a `HybridQuery` filtered by cognitive memory types. | `src/types/saos.rs` |
| `pub fn metadata(&self) -> f32` | Returns the metadata signal weight component. Reserved for future metadata-signal fusion; currently always 0.0. Do not expose a public cons… | `src/types/saos.rs` |
| `pub fn new(vector: f32, text: f32, graph: f32) -> Result<Self>` | Creates 3-signal fusion weights (vector, text, graph) summing to 1.0. | `src/types/saos.rs` |
| `pub fn new( text_query: Option<String>, semantic_query: Option<String>, anchor_entities: Vec<String>, ) -> Self` | Creates a new `QueryRewriteOutput` with text, semantic, and anchor entity reformulations. | `src/types/saos.rs` |
| `pub fn new() -> Self` | Creates a new `HybridQueryBuilder` with empty default options. | `src/types/saos.rs` |
| `pub fn rerank_pool_config(multiplier: usize, max: usize) -> Result<Self>` | Creates a `HybridQuery` with pre-reranking pool expansion settings. | `src/types/saos.rs` |
| `pub fn strategy_for_signal(&self, signal_name: &str) -> FusionStrategy` | Returns the fusion strategy configured for a specific signal name ("vector", "text", "graph"). | `src/types/saos.rs` |
| `pub fn text(&self) -> f32` | Returns the text signal weight component. | `src/types/saos.rs` |
| `pub fn text_only(text: impl Into<String>) -> Self` | Creates a simple text-only `QueryRewriteOutput`. | `src/types/saos.rs` |
| `pub fn text_search(text: impl Into<String>) -> Result<Self>` | Creates a `HybridQuery` with a text search query string. | `src/types/saos.rs` |
| `pub fn uniform(strategy: FusionStrategy) -> Self` | Creates a uniform configuration where all signals use the specified strategy. | `src/types/saos.rs` |
| `pub fn vector(&self) -> f32` | Returns the vector signal weight component. | `src/types/saos.rs` |
| `pub fn vector_search(vector: Vec<f32>) -> Result<Self>` | Creates a `HybridQuery` with a vector query embedding. | `src/types/saos.rs` |
| `pub fn with_failure_policy(policy: OnSignalFailure) -> Result<Self>` | Creates a `HybridQuery` with a specified signal failure policy. | `src/types/saos.rs` |
| `pub fn with_filter(mut self, filter: FilterExpr) -> Self` | Sets a metadata expression filter on the query. | `src/types/saos.rs` |
| `pub fn with_fusion( weights: FusionWeights, strategy: impl Into<SignalFusionStrategies>, ) -> Result<Self>` | Creates a `HybridQuery` with custom fusion weights and signal fusion strategy. | `src/types/saos.rs` |
| `pub fn with_fusion_strategy(mut self, strategy: impl Into<SignalFusionStrategies>) -> Self` | Sets search fusion strategy per signal or globally (`FusionStrategy` or `SignalFusionStrategies`). | `src/types/saos.rs` |
| `pub fn with_fusion_weights(mut self, weights: FusionWeights) -> Self` | Sets custom signal fusion weights. | `src/types/saos.rs` |
| `pub fn with_graph_start_node(mut self, start: impl Into<String>) -> Self` | Sets the graph start node for graph traversal search. | `src/types/saos.rs` |
| `pub fn with_graph_strategy(mut self, strategy: GraphTraversalStrategy) -> Self` | Sets the graph traversal strategy (Hops or PersonalizedPageRank). | `src/types/saos.rs` |
| `pub fn with_include_provenance(mut self, include: bool) -> Self` | Sets whether to calculate and attach ProvenanceRecord to output results. | `src/types/saos.rs` |
| `pub fn with_include_superseded(mut self, include: bool) -> Self` | Sets whether to include superseded memory chunks (ADR-038). | `src/types/saos.rs` |
| `pub fn with_k(mut self, k: usize) -> Self` | Sets the top-K limit for the query. | `src/types/saos.rs` |
| `pub fn with_memory_type_filter(mut self, types: Vec<MemoryType>) -> Self` | Sets the memory_type pre-RRF filter. | `src/types/saos.rs` |
| `pub fn with_on_signal_failure(mut self, policy: OnSignalFailure) -> Self` | Sets the signal failure policy ([`OnSignalFailure`]) for the query (Spec Teil 6.5). | `src/types/saos.rs` |
| `pub fn with_provenance_and_superseded( include_provenance: bool, include_superseded: bool, ) -> Result<Self>` | Creates a `HybridQuery` with options for superseded items and provenance. | `src/types/saos.rs` |
| `pub fn with_rerank_pool_max(mut self, max: usize) -> Self` | Sets the upper bound cap for pre-reranking candidate pool expansion. | `src/types/saos.rs` |
| `pub fn with_rerank_pool_multiplier(mut self, multiplier: usize) -> Self` | Sets the candidate pool multiplier for pre-reranking retrieval. | `src/types/saos.rs` |
| `pub fn with_same_community_as(mut self, entity_id: EntityId) -> Self` | Sets the community context entity filter/boost. | `src/types/saos.rs` |
| `pub fn with_text_query(mut self, q: impl Into<String>) -> Self` | Sets the full-text search query string. | `src/types/saos.rs` |
| `pub fn with_vector_query(mut self, v: Vec<f32>) -> Self` | Sets the vector query embedding. | `src/types/saos.rs` |


---

## `contextra-ports`

| Attribut | Wert |
|---|---|
| **Ring** | 0 — Ring 0 — Foundation & Pure Numerics/Types |
| **Status** | Fertig |
| **Zweck** | dyn-kompatible Trait-Definitionen (Ports) für Storage, Indizes, Embedder und System-Uhren. |
| **Unsafe-Code** | Nein (`#![forbid(unsafe_code)]`) |
| **Interne Abhängigkeiten** | `contextra-types` |
| **Externe Abhängigkeiten (Auswahl)** | `ahash`, `bytes`, `futures-util`, `serde`, `serde_json`, `thiserror` |

### Architekturbeschreibung

Core trait definitions for Contextra subsystems.  These traits define the abstract interfaces that concrete implementations must fulfill, enabling modularity and testability.

### Modul- und Re-Export-Oberfläche (lib.rs)

```rust
pub mod attention;
pub mod checkpoint;
pub mod clock;
pub mod embedding;
pub mod graph;
pub mod graph_index;
pub mod id_gen;
pub mod kv;
pub mod kv_bridge_port;
pub mod license;
pub mod lifecycle;
pub mod metrics;
pub mod observability;
pub mod plugin;
pub mod query_rewriter;
pub mod reranker;
pub mod rng;
pub mod storage;
pub mod text_index;
pub mod vector_index;
pub use contextra_types::*;
pub use contextra_types::{error, error_dto, model_fingerprint, schema, tombstone, types};
pub use attention::*;
pub use checkpoint::*;
pub use clock::*;
pub use embedding::*;
pub use graph::*;
pub use graph_index::*;
pub use id_gen::*;
pub use kv::*;
pub use kv_bridge_port::*;
pub use license::*;
pub use lifecycle::*;
pub use metrics::*;
pub use observability::*;
pub use plugin::*;
pub use query_rewriter::*;
pub use reranker::*;
pub use rng::*;
pub use storage::*;
pub use text_index::*;
pub use vector_index::*;
```

### Schnittstellen — Public Traits (30)

**`AttentionExporter`** — *Port implemented by inference backends (Ring 2) to export attention weights from the latest prefill step to the KV-cache eviction worker (Ring 1).* (`src/attention.rs`)

```rust
pub trait AttentionExporter : Send + Sync {
/// Returns accumulated attention weights per token position for the given request ID.
    ///
    /// Returns `None` if no attention weights are tracked or available for the request.
    fn export_attention_weights(&self, request_id: RequestId) -> Option<Vec<f32>>;
}
```

**`Checkpoint`** — *Abstract contract for generating consistent checkpoints.* (`src/checkpoint.rs`)

```rust
pub trait Checkpoint : Send + Sync + 'static {
/// Takes a deterministic snapshot of the current state.
    fn take_snapshot<'a>(&'a self, tx: TxId) -> BoxFuture<'a, Result<WorkflowState>>;

    /// Rolls the state back to the specified checkpoint.
    fn restore<'a>(&'a self, state: &'a WorkflowState) -> BoxFuture<'a, Result<()>>;
}
```

**`CheckpointCoordinator`** — *Unified Checkpoint Coordinator Trait combining named, TxId+seq_no-scoped, persistent checkpoints. # Dyn-Safety Note This trait is intentionally NOT dyn-compatible without specifying the associated type `Meta` (e.g. `Arc<dyn CheckpointCoordinator<Meta = ...>>`…* (`src/checkpoint.rs`)

```rust
pub trait CheckpointCoordinator : Send + Sync + 'static {
/// Type representing checkpoint metadata.
    type Meta: Send + Sync;

    /// Creates and persists a new named checkpoint.
    fn create_named_checkpoint(
        &self,
        name: &str,
        collection_id: &str,
        seq_no: u64,
        tx_id: TxId,
        metadata: serde_json::Value,
    ) -> impl Future<Output = Result<Self::Meta>> + Send;

    /// Restores database state to a named checkpoint.
    fn restore_named_checkpoint(
        &self,
        name: &str,
    ) -> impl Future<Output = Result<Self::Meta>> + Send;

    /// Deletes a checkpoint by name.
    fn drop_named_checkpoint(&self, name: &str) -> impl Future<Output = Result<()>> + Send;

    /// Lists all active checkpoints.
    fn list_named_checkpoints(&self) -> impl Future<Output = Result<Vec<Self::Meta>>> + Send;
}
```

**`Clock`** — *Port trait for time operations, enabling deterministic testing by decoupling system wall-clock and monotonic time sources.* (`src/clock.rs`)

```rust
pub trait Clock : Send + Sync + 'static {
/// Returns the current wall-clock time as nanoseconds since the Unix epoch.
    ///
    /// Returns `0` if system time is before the epoch instead of panicking.
    fn now_unix_nanos(&self) -> u64;

    /// Returns nanoseconds elapsed since a monotonic baseline reference point.
    fn monotonic_nanos(&self) -> u64;
}
```

**`CommunityResolver`** — *Contract for resolving graph community assignments for entities.* (`src/graph_index.rs`)

```rust
pub trait CommunityResolver : Send + Sync {
/// Resolves the optional community ID for a given entity.
    fn get_community<'a>(&'a self, entity_id: EntityId) -> BoxFuture<'a, Result<Option<u64>>>;
}
```

**`ContextPreparer`** — *Contract for trimming and preparing context windows tailored to token budgets.* (`src/lifecycle.rs`)

```rust
pub trait ContextPreparer : Send + Sync {
/// Prepares and trims context chunks according to the provided token budget and relevance threshold.
    fn prepare_context(
        &self,
        chunks: Vec<ContextChunk>,
        budget: &TokenBudget,
        relevance_threshold: f32,
    ) -> Result<ContextWindow>;
}
```

**`DriftStatusProvider`** — *Trait for querying Lyapunov drift status from an attached router engine without creating a cyclic dependency (ADR-080).* (`src/observability.rs`)

```rust
pub trait DriftStatusProvider : Send + Sync {
/// Returns the overall drift status string ("stabil", "warnung", "kritisch", or "unbekannt").
    fn overall_drift_status(&self) -> String;
}
```

**`EmbeddingProvider`** — *Abstraction for all embedding providers (OllamaEmbedder, OnnxEmbedder, MockEmbedder).* (`src/embedding.rs`)

```rust
pub trait EmbeddingProvider : Send + Sync + 'static {
/// Returns the provider identifier/name (e.g. for logging/debugging).
    fn provider_name(&self) -> &str;

    /// Generates an embedding vector for the given text.
    fn embed<'a>(&'a self, text: &'a str) -> BoxFuture<'a, Result<Vec<f32>, EmbeddingError>>;

    /// Returns the dimensionality of embeddings generated by this provider.
    fn embedding_dim(&self) -> usize;

    /// Generates embeddings for multiple texts.
    fn embed_batch<'a>(
        &'a self,
        texts: &'a [&'a str],
    ) -> BoxFuture<'a, Result<Vec<Vec<f32>>, EmbeddingError>> {
        Box::pin(async move {
            let mut results = Vec::with_capacity(texts.len());
            for text in texts {
                results.push(self.embed(text).await?);
            }
            Ok(results)
        })
    }
}
```

**`GraphCollectionMutation`** — *Trait defining n-ary hyperedge collection mutations for graph indexing (Anhang B §B.5.1.3). # Dyn-Kompatibilität Dieser Trait ist durch synchrone Rückgabetypen vtable-kompatibel (dyn-safe).* (`src/graph.rs`)

```rust
pub trait GraphCollectionMutation : Send + Sync + 'static {
/// Relates multiple entities into an n-ary hyperedge associated with a document ID.
    fn relate_n_ary(
        &self,
        predicate_tag: u32,
        participants: &[RoleBinding],
        doc_id: DocId,
    ) -> Result<HyperEdgeId, GraphMutationError>;
}
```

**`GraphIndex`** — *Graph-Index Trait — CSR-basierte Entity-Relation-Traversal. # Dyn-Kompatibilität Dieser Trait ist durch explizite `BoxFuture`-Rückgabetypen vtable-kompatibel (dyn-safe). # TxId-Origin-Invariant (AGT-GRAPH-001) **Aufrufer MÜSSEN tx entweder aus der Collection-…* (`src/graph_index.rs`)

```rust
pub trait GraphIndex : Send + Sync + 'static {
/// Traverses the entity graph using BFS up to a maximum number of hops.
    /// Distributes traversing decay weights across related entities.
    fn traverse<'a>(
        &'a self,
        start_node: EntityId,
        max_hops: usize,
    ) -> BoxFuture<'a, Result<Vec<(EntityId, f32)>>>;

    /// Returns direct (1-hop) neighbor EntityIds for the given entity.
    fn neighbors<'a>(&'a self, start_node: EntityId) -> BoxFuture<'a, Result<Vec<EntityId>>> {
        Box::pin(async move {
            let results = self.traverse(start_node, 1).await?;
            Ok(results.into_iter().map(|(id, _)| id).collect())
        })
    }

    /// Removes an edge between two entities.
    fn remove_edge<'a>(
        &'a self,
        tx: TxId,
        from: EntityId,
        to: EntityId,
    ) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            let _ = (tx, from, to);
            Ok(())
        })
    }

    /// Adds a bidirectional edge between two entities.
    fn add_bidirectional<'a>(
        &'a self,
        tx: TxId,
        from: EntityId,
        to: EntityId,
        label: &'a str,
    ) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            self.add_edge(tx, Edge::new(from, to, label)).await?;
            self.add_edge(tx, Edge::new(to, from, label)).await?;
            Ok(())
        })
    }

    /// Traverses the entity graph starting from multiple anchor entities up to max_hops.
    /// Aggregates decay weights (keeping max score per entity) across anchors.
    fn multi_traverse<'a>(
        &'a self,
        start_nodes: &'a [EntityId],
        max_hops: usize,
    ) -> BoxFuture<'a, Result<Vec<(EntityId, f32)>>> {
        Box::pin(async move {
            let mut combined: AHashMap<EntityId, f32> = AHashMap::default();
            for &start in start_nodes {
                let results = self.traverse(start, max_hops).await?;
                for (entity_id, score) in results {
                    combined
                        .entry(entity_id)
                        .and_modify(|s| *s = s.max(score))
                        .or_insert(score);
                }
            }
            let mut results: Vec<(EntityId, f32)> = combined.into_iter().collect();
            results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            Ok(results)
        })
    }

    /// Traverses the entity graph starting from multiple anchor entities up to max_hops at a specific sequence number.
    /// Aggregates decay weights (keeping max score per entity) across anchors.
    fn multi_traverse_at<'a>(
        &'a self,
        start_nodes: &'a [EntityId],
        max_hops: usize,
        seq_no: u64,
    ) -> BoxFuture<'a, Result<Vec<(EntityId, f32)>>> {
        Box::pin(async move {
            let mut combined: AHashMap<EntityId, f32> = AHashMap::default();
            for &start in start_nodes {
                let results = self.traverse_at(start, max_hops, seq_no).await?;
                for (entity_id, score) in results {
                    combined
                        .entry(entity_id)
                        .and_modify(|s| *s = s.max(score))
                        .or_insert(score);
                }
            }
            let mut results: Vec<(EntityId, f32)> = combined.into_iter().collect();
            results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            Ok(results)
        })
    }

    /// Executes PathRAG graph traversal starting from multiple anchor entities up to `max_hops` at a specific sequence number (`seq_no`).
    ///
    /// # Contract
    /// - The result is the set of entities along with their relevance scores visible at sequence number `seq_no`.
    /// - Results are sorted by score descending, with ties broken by EntityId ascending.
    /// - A backend implementation without snapshot support returns the default [`ContextraError::SnapshotUnsupportedForSignal`][crate::error::ContextraError::SnapshotUnsupportedForSignal] error.
    ///
    /// # Errors
    /// Returns [`ContextraError::SnapshotUnsupportedForSignal`][crate::error::ContextraError::SnapshotUnsupportedForSignal]
    /// if snapshot-isolated PathRAG traversal is not implemented by the backend.
    fn path_rag_at<'a>(
        &'a self,
        _start_nodes: &'a [EntityId],
        _max_hops: usize,
        _seq_no: u64,
    ) -> BoxFuture<'a, Result<Vec<(EntityId, f32)>>> {
        Box::pin(async move {
            Err(
                crate::error::ContextraError::snapshot_unsupported_for_signal(
                    "PathRag strategy does not support snapshot-isolated retrieval",
                ),
            )
        })
    }

    /// Traverses the entity graph using BFS up to a maximum number of hops at a specific sequence number.
    ///
    /// # Errors
    /// Returns [`ContextraError::CapabilityUnsupported`][crate::ContextraError::CapabilityUnsupported]
    /// with capability `"graph_traverse_at"` if snapshot-isolated graph traversal is not implemented.
    /// Tested via `capability_coverage` test module.
    fn traverse_at<'a>(
        &'a self,
        _start_node: EntityId,
        _max_hops: usize,
        _seq_no: u64,
    ) -> BoxFuture<'a, Result<Vec<(EntityId, f32)>>> {
        Box::pin(async move {
            Err(crate::error::ContextraError::capability_unsupported(
                "graph_traverse_at",
                "Graph traversal snapshot isolation (traverse_at) is not supported by default — tracked in ADR-024",
            ))
        })
    }

    /// Calculates Personalized PageRank (PPR) starting from seed nodes on the graph state visible at a specific sequence number (`seq_no`).
    ///
    /// # Errors
    /// Returns [`ContextraError::CapabilityUnsupported`][crate::ContextraError::CapabilityUnsupported]
    /// with capability `"graph_ppr_snapshot"` if snapshot-isolated Personalized PageRank is not implemented.
    /// Tested via `capability_coverage` test module.
    fn personalized_page_rank_at<'a>(
        &'a self,
        _seed_nodes: &'a [EntityId],
        _config: &'a PprConfig,
        _seq_no: u64,
    ) -> BoxFuture<'a, Result<Vec<(EntityId, f32)>>> {
        Box::pin(async move {
            Err(crate::error::ContextraError::capability_unsupported(
                "graph_ppr_snapshot",
                "Personalized PageRank snapshot isolation (personalized_page_rank_at) is not supported by default — tracked in ADR-024",
            ))
        })
    }

    /// Traverses the entity graph using BFS at a specific point in time (bi-temporal edge filtering).
    ///
    /// # Errors
    /// Returns [`ContextraError::CapabilityUnsupported`][crate::ContextraError::CapabilityUnsupported]
    /// with capability `"graph_traverse_at_time"` if bi-temporal graph traversal is not implemented.
    /// Tested via `capability_coverage` test module.
    fn traverse_at_time<'a>(
        &'a self,
        _start_node: EntityId,
        _max_hops: usize,
        _as_of: TxId,
    ) -> BoxFuture<'a, Result<Vec<(EntityId, f32)>>> {
        Box::pin(async move {
            Err(crate::error::ContextraError::capability_unsupported(
                "graph_traverse_at_time",
                "Bi-temporal graph traversal (traverse_at_time) is not supported by default",
            ))
        })
    }

    /// Traverses the entity graph using BFS with independent system time and business time constraints.
    ///
    /// # Errors
    /// Returns [`ContextraError::CapabilityUnsupported`][crate::ContextraError::CapabilityUnsupported]
    /// with capability `"graph_traverse_at_bitemporal"` if bitemporal graph traversal is not implemented.
    fn traverse_at_bitemporal<'a>(
        &'a self,
        _start_node: EntityId,
        _max_hops: usize,
        _as_of_tx: TxId,
        _as_of_business: Option<i64>,
    ) -> BoxFuture<'a, Result<Vec<(EntityId, f32)>>> {
        Box::pin(async move {
            Err(crate::error::ContextraError::capability_unsupported(
                "graph_traverse_at_bitemporal",
                "Bi-temporal graph traversal (traverse_at_bitemporal) is not supported by default",
            ))
        })
    }

    /// Calculates Personalized PageRank (PPR) starting from seed nodes.
    ///
    /// # Convergence Behavior
    /// Power iteration terminates when the L1 norm difference between iterations drops below `config.convergence_epsilon`,
    /// or when `config.max_iterations` is reached. If `config.max_iterations` is reached without full convergence,
    /// the function returns the best-effort intermediate ranking state (no `Err`) and emits a `tracing::warn!` log entry.
    ///
    /// # Errors
    /// Returns [`ContextraError::CapabilityUnsupported`][crate::ContextraError::CapabilityUnsupported]
    /// with capability `"graph_ppr"` if Personalized PageRank is not supported by this implementation.
    /// Tested via `capability_coverage` test module.
    fn personalized_page_rank<'a>(
        &'a self,
        _seed_nodes: &'a [EntityId],
        _config: &'a PprConfig,
    ) -> BoxFuture<'a, Result<Vec<(EntityId, f32)>>> {
        Box::pin(async move {
            Err(crate::error::ContextraError::capability_unsupported(
                "graph_ppr",
                "Personalized PageRank (personalized_page_rank) is not supported by default for this GraphIndex implementation",
            ))
        })
    }

    /// Inserts or updates a node entity.
    fn add_entity<'a>(&'a self, tx: TxId, entity: Entity) -> BoxFuture<'a, Result<()>>;

    /// Inserts or updates an edge between two entities.
    fn add_edge<'a>(&'a self, tx: TxId, edge: Edge) -> BoxFuture<'a, Result<()>>;

    /// Commits a transaction.
    fn commit<'a>(&'a self, tx: TxId) -> BoxFuture<'a, Result<()>>;

    /// Rolls back a transaction.
    fn rollback<'a>(&'a self, tx: TxId) -> BoxFuture<'a, Result<()>>;

    /// Rolls back the entire graph state to a specific transaction ID.
    fn rollback_to_tx<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>>;

    /// Returns the last transaction ID processed by the index.
    fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, Result<TxId>>;

    /// Returns the number of entities in the index.
    fn len<'a>(&'a self) -> BoxFuture<'a, usize>;

    /// Returns true if the index is empty.
    fn is_empty<'a>(&'a self) -> BoxFuture<'a, bool> {
        Box::pin(async move { self.len().await == 0 })
    }

    /// Collects statistics for the Graph.
    fn stats<'a>(&'a self) -> BoxFuture<'a, Result<GraphIndexStats>>;
}
```

**`GroundingValidator`** — *Abstract contract for post-hoc hallucination / grounding validation.* (`src/lifecycle.rs`)

```rust
pub trait GroundingValidator : Send + Sync {
/// Validates an LLM-generated response against retrieval context chunks.
    ///
    /// Returns `Ok(GroundingAssessment)` if confidence meets threshold,
    /// or `Err(ContextraError)` (e.g. `ContextraError::PolicyViolation` or low-confidence signal)
    /// on grounding failure or abstention.
    fn validate_grounding<'a>(
        &'a self,
        response: &'a str,
        context_chunks: &'a [ContextChunk],
    ) -> BoxFuture<'a, Result<GroundingAssessment>>;
}
```

**`HybridSearchProvider`** — *Contract for executing hybrid (vector + text) queries for profile routing.* (`src/vector_index.rs`)

```rust
pub trait HybridSearchProvider : Send + Sync {
/// Executes a hybrid query returning matched context chunks with relevance scores.
    fn search_hybrid<'a>(
        &'a self,
        query_text: &'a str,
        query_embedding: &'a [f32],
        top_k: usize,
    ) -> BoxFuture<'a, Result<Vec<ContextChunk>>>;
}
```

**`IdGen`** — *Port trait for unique ID generation.* (`src/id_gen.rs`)

```rust
pub trait IdGen : Send + Sync + 'static {
/// Generates and returns the next unique 64-bit unsigned identifier.
    fn next_id(&self) -> u64;
}
```

**`KvBridgeStorage`** — *Abstrahiertes Persistence-Interface für KV-Bridge Tier-2 Spill-over operations. Implementierbar von Ring-1 Storage Engines (`contextra-store::LsmStorage`) oder Test-Mocks.* (`src/kv_bridge_port.rs`)

```rust
pub trait KvBridgeStorage : Send + Sync {
/// Versucht, einen Wert unter dem angegebenen Schlüssel abzufragen.
    fn get<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, Result<Option<Bytes>>>;

    /// Schreibt einen Schlüssel-Wert-Paar im Kontext einer Transaktion (`tx_id`).
    fn put<'a>(&'a self, tx_id: TxId, key: &'a [u8], value: &'a [u8]) -> BoxFuture<'a, Result<()>>;

    /// Committet die Transaktion mit der ID `tx_id`.
    fn commit<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>>;
}
```

**`KvLifecycleHooks`** — *Trait defining lifecycle hooks for KV cache stores (rollback, document removal, tenant purge). Synchronous and dyn-compatible interface without async runtime dependencies (P26). Guarantees strict tenant isolation across all lifecycle operations (INV-TENANT).* (`src/kv.rs`)

```rust
pub trait KvLifecycleHooks : Send + Sync + 'static {
/// Cleans up KV cache segments associated with a rolled-back transaction.
    fn on_rollback(&self, tenant: TenantId, chunk_ids: &[u64]);

    /// Removes KV cache segments associated with a specific document.
    fn remove_doc_segments(&self, tenant: TenantId, doc: contextra_types::DocId);

    /// Purges all cached KV segments and radix state for the given tenant.
    fn purge_tenant(&self, tenant: TenantId);
}
```

**`KvPrefixStore`** — *Trait defining the contract for KV prefix caching and block reuse (Spec §9.2). # Dyn-Kompatibilität Dieser Trait ist durch synchrone Schnittstellen vtable-kompatibel (dyn-safe).* (`src/kv.rs`)

```rust
pub trait KvPrefixStore : Send + Sync + 'static {
/// Returns the longest exact matching prefix in blocks for the prompt tokens, if any.
    fn lookup(&self, tenant: TenantId, key: &PrefixKey, tokens: &[u32]) -> Option<KvPrefixHit>;

    /// Inserts KV blocks for a given token prefix into the store.
    fn insert(
        &self,
        tenant: TenantId,
        key: &PrefixKey,
        tokens: &[u32],
        blocks: Vec<KvBlock>,
    ) -> Result<(), ContextraError>;

    /// Evicts KV prefix entries for the given tenant and prefix key.
    /// Returns the number of blocks evicted.
    fn evict(&self, tenant: TenantId, key: &PrefixKey) -> Result<u64, ContextraError>;
}
```

**`LicenseGate`** — *Dyn-kompatibler Port (P27) für Lizenzprüfung — Implementierung (Lizenzserver-Anbindung, Ed25519-Signaturprüfung analog zum Löschbeweis-Pfad §14.1) ist NICHT Teil dieses Tickets und folgt in einem eigenen, separat zu beauftragenden Schritt.* (`src/license.rs`)

```rust
pub trait LicenseGate : Send + Sync {
/// Checks if the requested feature ring is activated and accessible.
    fn check_ring(&self, ring: FeatureRing) -> Result<(), LicenseError>;

    /// Authorizes the requested feature ring and yields an [`AuthorizedRing`] token on success.
    fn authorize(&self, requested: FeatureRing) -> Result<AuthorizedRing, LicenseError> {
        self.check_ring(requested)?;
        Ok(AuthorizedRing::new(requested))
    }
}
```

**`LlmTextGenerator`** — *Abstract contract for LLM text generation (summarization, importance evaluation, query expansion).* (`src/embedding.rs`)

```rust
pub trait LlmTextGenerator : Send + Sync + 'static {
/// Generates text for a given prompt using an LLM.
    fn generate<'a>(&'a self, prompt: &'a str) -> BoxFuture<'a, crate::Result<String>>;

    /// Generates text for a given sequence of context segments using an LLM.
    ///
    /// Default implementation naively concatenates segment texts with `"\n\n"` and invokes `self.generate(&concatenated)`.
    fn generate_with_context<'a>(
        &'a self,
        tenant: TenantId,
        segments: &'a [ContextSegment<'a>],
    ) -> BoxFuture<'a, crate::Result<String>> {
        Box::pin(async move {
            let _ = tenant;
            let concatenated = segments
                .iter()
                .map(|s| s.text)
                .collect::<Vec<_>>()
                .join("\n\n");
            self.generate(&concatenated).await
        })
    }
}
```

**`LlmTextGeneratorStreaming`** — *Abstract contract for streaming LLM text generation (token by token / chunk by chunk).* (`src/embedding.rs`)

```rust
pub trait LlmTextGeneratorStreaming : LlmTextGenerator {
/// Generates text stream for a given prompt using an LLM configuration.
    fn generate_stream<'a>(
        &'a self,
        prompt: &'a str,
        config: &'a ConfigFingerprint,
    ) -> super::BoxStream<'a, crate::Result<String>>;
}
```

**`MetricsSink`** — *Metrics sink port trait for recording system counters, gauges, and histograms. Implemented by observability backends to collect operational metrics from domain components without creating direct dependencies on specific metrics libraries.* (`src/metrics.rs`)

```rust
pub trait MetricsSink : Send + Sync + 'static {
/// Records a counter metric increment.
    fn record_counter(&self, name: &str, value: u64, labels: &[(&str, &str)]);

    /// Records a gauge metric absolute value.
    fn record_gauge(&self, name: &str, value: f64, labels: &[(&str, &str)]);

    /// Records a histogram observation value.
    fn record_histogram(&self, name: &str, value: f64, labels: &[(&str, &str)]);
}
```

**`PluginManifest`** — *Manifest trait implemented by dynamic or static plugins.* (`src/plugin.rs`)

```rust
pub trait PluginManifest : Send + Sync {
/// Returns the name of the plugin.
    fn name(&self) -> &str;

    /// Returns the list of plugin names required prior to activation.
    fn requires(&self) -> &[&str] {
        &[]
    }

    /// Returns the list of plugin names that conflict with this plugin.
    fn conflicts(&self) -> &[&str] {
        &[]
    }

    /// Returns the required feature ring for this plugin.
    fn feature_ring_required(&self) -> FeatureRing {
        FeatureRing::Fast
    }

    /// Returns the static capability metadata for this plugin.
    fn capability(&self) -> PluginCapability;

    /// Activates the plugin instance.
    fn activate(&self) -> Result<(), PluginError>;
}
```

**`QueryRewriter`** — *Trait for query rewriting in multi-step iterative retrieval (LLM-agnostic).* (`src/query_rewriter.rs`)

```rust
pub trait QueryRewriter : Send + Sync {
/// Generates alternative text sub-queries based on original query and previous search results.
    ///
    /// For backward compatibility, default implementation calls `rewrite_structured` and collects non-empty text queries.
    fn rewrite<'a>(
        &'a self,
        original_query: &'a str,
        current_results: &'a [ScoredEntry],
    ) -> BoxFuture<'a, Result<Vec<String>>> {
        Box::pin(async move {
            let outputs = self
                .rewrite_structured(original_query, current_results)
                .await?;
            let text_queries = outputs
                .into_iter()
                .filter_map(|out| out.text_query)
                .filter(|q| !q.trim().is_empty())
                .collect();
            Ok(text_queries)
        })
    }

    /// Generates multi-signal query reformulations (text, semantic, and graph anchors) for multi-step retrieval.
    ///
    /// Default implementation delegates to `rewrite` and wraps the resulting string queries as `QueryRewriteOutput::text_only`.
    fn rewrite_structured<'a>(
        &'a self,
        original_query: &'a str,
        current_results: &'a [ScoredEntry],
    ) -> BoxFuture<'a, Result<Vec<QueryRewriteOutput>>> {
        Box::pin(async move {
            let queries = self.rewrite(original_query, current_results).await?;
            let outputs = queries
                .into_iter()
                .map(QueryRewriteOutput::text_only)
                .collect();
            Ok(outputs)
        })
    }
}
```

**`Reranker`** — *Trait for post-retrieval cross-encoder reranking models.* (`src/reranker.rs`)

```rust
pub trait Reranker : Send + Sync {
/// Reranks candidates for a given query text.
    fn rerank<'a>(
        &'a self,
        query: &'a str,
        candidates: &'a [String],
    ) -> BoxFuture<'a, Result<Vec<RerankResult>>>;

    /// Records implicit feedback or outcome labels for calibration.
    fn record_implicit_feedback(&self, _results: &[RerankResult], _implicit_relevant_k: usize) {}

    /// Returns the maximum allowed execution time in milliseconds before timing out, if configured.
    fn rerank_deadline_ms(&self) -> Option<u64> {
        Some(500)
    }
}
```

**`ResponseGroundingValidator`** — *Abstract contract for grounding validation of LLM-generated responses against raw source strings.* (`src/lifecycle.rs`)

```rust
pub trait ResponseGroundingValidator : Send + Sync {
/// Evaluates the grounding score for an LLM-generated response against source text slices.
    /// Returns a float score in [0.0, 1.0].
    fn score_grounding(&self, response: &str, sources: &[&str]) -> Result<f32>;
}
```

**`Rng`** — *Port trait for random number generators, enabling deterministic testing by decoupling non-deterministic entropy sources.* (`src/rng.rs`)

```rust
pub trait Rng : Send + Sync + 'static {
/// Generates the next pseudorandom 64-bit unsigned integer.
    fn next_u64(&self) -> u64;

    /// Fills `dest` with pseudorandom bytes generated by this RNG.
    fn fill_bytes(&self, dest: &mut [u8]);

    /// Returns a pseudorandom `f64` value in the half-open interval `[0.0, 1.0)`
    /// with 53 bits of precision.
    fn next_unit_f64(&self) -> f64;
}
```

**`SegmentSynthesizer`** — *Trait-Abstraktion für LLM-Synthesizer zur Segment-Zusammenfassung (REM-Phase).* (`src/text_index.rs`)

```rust
pub trait SegmentSynthesizer : Send + Sync {
/// Synthetisiert ein Segment von Texten zu einer abstrakten Zusammenfassung.
    fn synthesize_segment<'a>(
        &'a self,
        segment_texts: &'a [&'a str],
    ) -> BoxFuture<'a, Result<String>>;
    /// Gibt die Modell-ID des Synthesizers zurück.
    fn model_id(&self) -> &str;
}
```

**`StorageEngine`** — *Unified Storage Engine trait — retained for backward compatibility across higher layers. # Dyn-Kompatibilität Dieser Trait ist durch explizite `BoxFuture`-Rückgabetypen vtable-kompatibel (dyn-safe).* (`src/storage.rs`)

```rust
pub trait StorageEngine : Send + Sync + 'static {
// TOCTOU-ACCEPTED: StorageEngine provides default batching/tracking delegate methods.
    /// Retrieves a value by key.
    fn get<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, Result<Option<Bytes>>>;

    /// Retrieves a value by key at a specific sequence number (MVCC).
    fn get_at_seq<'a>(&'a self, key: &'a [u8], seq: u64) -> BoxFuture<'a, Result<Option<Bytes>>>;

    /// Retrieves a value by key tracked within transaction `tx_id` for SSI read-set validation.
    ///
    /// # ReadSet Tracking
    /// Untracked: Default implementation delegates to [`StorageEngine::get`] and registriert KEINE Lesezugriffe im ReadSet.
    /// Concrete SSI-enabled engines (e.g. `LsmStorage`) override this method to record keys in the transaction's `ReadSet`.
    fn get_tracked<'a>(
        &'a self,
        _tx_id: TxId,
        key: &'a [u8],
    ) -> BoxFuture<'a, Result<Option<Bytes>>> {
        self.get(key)
    }

    /// Retrieves a value by key at `seq` tracked within transaction `tx_id` for SSI read-set validation.
    ///
    /// # ReadSet Tracking
    /// Untracked: Default implementation delegates to [`StorageEngine::get_at_seq`] and registriert KEINE Lesezugriffe im ReadSet.
    /// Concrete SSI-enabled engines (e.g. `LsmStorage`) override this method to record keys in the transaction's `ReadSet`.
    fn get_at_seq_tracked<'a>(
        &'a self,
        _tx_id: TxId,
        key: &'a [u8],
        seq: u64,
    ) -> BoxFuture<'a, Result<Option<Bytes>>> {
        self.get_at_seq(key, seq)
    }

    /// Scans a range of keys with given prefix tracked within transaction `tx_id` for SSI read-set validation.
    ///
    /// # ReadSet Tracking
    /// Untracked: Default implementation delegates to [`StorageEngine::scan_prefix`] and registriert KEINE Lesezugriffe im ReadSet.
    /// Concrete SSI-enabled engines (e.g. `LsmStorage`) override this method to record keys/prefixes in the transaction's `ReadSet`.
    #[allow(clippy::type_complexity)]
    fn scan_prefix_tracked<'a>(
        &'a self,
        _tx_id: TxId,
        prefix: &'a [u8],
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        self.scan_prefix(prefix)
    }

    /// Returns `true` if this storage engine implementation actively tracks read-sets for SSI validation.
    ///
    /// Default implementation returns `false`.
    fn supports_ssi_tracking(&self) -> bool {
        false
    }

    /// Stores a key-value pair as part of a transaction.
    fn put<'a>(&'a self, tx_id: TxId, key: &'a [u8], value: &'a [u8]) -> BoxFuture<'a, Result<()>>;

    /// Atomically writes `value` under `key` within transaction ONLY IF no value exists.
    fn put_if_absent<'a>(
        &'a self,
        _tx_id: TxId,
        _key: &'a [u8],
        _value: &'a [u8],
    ) -> BoxFuture<'a, Result<bool>> {
        Box::pin(async move {
            Err(crate::error::ContextraError::capability_unsupported(
                "put_if_absent",
                "Atomic put_if_absent is not supported by default — concrete storage engines must implement it atomically.",
            ))
        })
    }

    /// Stores multiple key-value pairs as part of a transaction.
    fn put_batch<'a>(
        &'a self,
        tx_id: TxId,
        entries: &'a [(Vec<u8>, Vec<u8>)],
    ) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            for (key, value) in entries {
                self.put(tx_id, key, value).await?;
            }
            Ok(())
        })
    }

    /// Deletes a key as part of a transaction.
    fn delete<'a>(&'a self, tx_id: TxId, key: &'a [u8]) -> BoxFuture<'a, Result<()>>;

    /// Deletes multiple keys as a single logical batch operation.
    fn delete_many<'a>(&'a self, tx_id: TxId, keys: Vec<Vec<u8>>) -> BoxFuture<'a, Result<u64>> {
        Box::pin(async move {
            let mut deleted = 0u64;
            for key in keys {
                self.delete(tx_id, &key).await?;
                deleted += 1;
            }
            Ok(deleted)
        })
    }

    /// Deletes all key-value pairs starting with `prefix` as part of a transaction.
    // TOCTOU-ACCEPTED
    fn delete_prefix<'a>(&'a self, tx_id: TxId, prefix: &'a [u8]) -> BoxFuture<'a, Result<u64>> {
        Box::pin(async move {
            let matching_keys: Vec<Vec<u8>> = self
                .scan_prefix(prefix)
                .await?
                .into_iter()
                .map(|(key, _)| key)
                .collect();
            self.delete_many(tx_id, matching_keys).await
        })
    }

    /// Commits a transaction.
    fn commit<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>>;

    /// Rolls back a transaction.
    fn rollback<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>>;

    /// Rolls back the entire storage state to a specific transaction ID.
    fn rollback_to_tx<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>>;

    /// Flushes memtable to disk.
    fn flush<'a>(&'a self) -> BoxFuture<'a, Result<()>>;

    /// Returns storage statistics.
    fn stats<'a>(&'a self) -> BoxFuture<'a, Result<StorageStats>>;

    /// Returns the last sequence number committed to storage.
    fn last_seq_no<'a>(&'a self) -> BoxFuture<'a, Result<u64>>;

    /// Returns the last transaction ID committed to storage.
    fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, Result<TxId>>;

    /// Pins a checkpoint for sequence number.
    fn pin_checkpoint<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, Result<()>>;

    /// Unpins a checkpoint for sequence number.
    fn unpin_checkpoint<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, Result<()>>;

    /// Scans a range of keys with given prefix.
    #[allow(clippy::type_complexity)]
    fn scan_prefix<'a>(
        &'a self,
        prefix: &'a [u8],
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>>;

    /// Scans prefix bounded.
    #[allow(clippy::type_complexity)]
    fn scan_prefix_bounded<'a>(
        &'a self,
        prefix: &'a [u8],
        limit: usize,
        cursor: Option<&'a [u8]>,
    ) -> BoxFuture<'a, Result<(Vec<(Vec<u8>, Vec<u8>)>, Option<Vec<u8>>)>> {
        Box::pin(async move {
            let all = self.scan_prefix(prefix).await?;
            let mut results = Vec::new();

            for (k, v) in all {
                if let Some(cur_bytes) = cursor {
                    if k.as_slice() <= cur_bytes {
                        continue;
                    }
                }
                results.push((k, v));
                if results.len() == limit {
                    break;
                }
            }

            let next_cursor = if results.len() == limit {
                results.last().map(|(k, _)| k.clone())
            } else {
                None
            };

            Ok((results, next_cursor))
        })
    }

    /// Scans prefix visible at sequence number.
    #[allow(clippy::type_complexity)]
    fn scan_prefix_at<'a>(
        &'a self,
        _prefix: &'a [u8],
        _seq_no: u64,
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        Box::pin(async move {
            Err(crate::error::ContextraError::capability_unsupported(
                "snapshot_read_at",
                "Storage-level snapshot-isolated prefix scan (scan_prefix_at) is not supported by default — implementors must override this method to guarantee MVCC snapshot isolation.",
            ))
        })
    }

    /// Scans range between start and end bounds.
    #[allow(clippy::type_complexity)]
    fn scan<'a>(
        &'a self,
        start: std::ops::Bound<&'a [u8]>,
        end: std::ops::Bound<&'a [u8]>,
        limit: Option<usize>,
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>>;

    /// Scans range bounded.
    #[allow(clippy::type_complexity)]
    fn scan_bounded<'a>(
        &'a self,
        _start: std::ops::Bound<&'a [u8]>,
        _end: std::ops::Bound<&'a [u8]>,
        _limit: usize,
        _cursor: Option<&'a [u8]>,
    ) -> BoxFuture<'a, Result<(Vec<(Vec<u8>, Vec<u8>)>, Option<Vec<u8>>)>> {
        Box::pin(async move {
            Err(crate::error::ContextraError::capability_unsupported(
                "scan_bounded",
                "Bounded range scan (scan_bounded) is not supported by default",
            ))
        })
    }
}
```

**`TextEmbeddingEngine`** — *Text embedding engine trait.* (`src/embedding.rs`)

```rust
pub trait TextEmbeddingEngine : Send + Sync + 'static {
/// Generates an embedding for the given text.
    fn embed<'a>(&'a self, text: &'a str) -> BoxFuture<'a, crate::Result<Vec<f32>>>;

    /// Generates embeddings for multiple texts.
    /// Default implementation executes sequential calls.
    fn embed_batch<'a>(
        &'a self,
        texts: &'a [&'a str],
    ) -> BoxFuture<'a, crate::Result<Vec<Vec<f32>>>> {
        Box::pin(async move {
            let mut results = Vec::with_capacity(texts.len());
            for text in texts {
                results.push(self.embed(text).await?);
            }
            Ok(results)
        })
    }
}
```

**`TextIndex`** — *Text-Index Trait — abstrahiert BM25/Inverted-Index-Operationen. # Dyn-Kompatibilität Verwendet native `async fn` (AFIT) für statischen Dispatch.* (`src/text_index.rs`)

```rust
pub trait TextIndex : Send + Sync + 'static {
/// Searches for documents matching the query.
    fn search(
        &self,
        query: &str,
        k: usize,
    ) -> impl Future<Output = Result<Vec<ScoredDocument>>> + Send;

    /// Searches for documents matching the query at a specific sequence number.
    ///
    /// # Errors
    /// Returns [`ContextraError::CapabilityUnsupported`][crate::ContextraError::CapabilityUnsupported]
    /// with capability `"snapshot_read_at"` if snapshot-isolated text search is not implemented.
    /// Tested via `capability_coverage` test module.
    fn search_at(
        &self,
        query: &str,
        k: usize,
        seq_no: u64,
    ) -> impl Future<Output = Result<Vec<ScoredDocument>>> + Send {
        async move {
            let _ = (query, k, seq_no);
            Err(crate::error::ContextraError::capability_unsupported(
                "snapshot_read_at",
                "Text search snapshot isolation (search_at) is not supported by default — tracked in ADR-024",
            ))
        }
    }

    /// Inserts or updates a document in the index.
    fn insert(&self, tx: TxId, id: DocId, text: &str) -> impl Future<Output = Result<()>> + Send;

    /// Deletes a document from the index.
    fn delete(&self, tx: TxId, id: DocId) -> impl Future<Output = Result<()>> + Send;

    /// Commits a transaction.
    fn commit(&self, tx: TxId) -> impl Future<Output = Result<()>> + Send;

    /// Rolls back a transaction.
    fn rollback(&self, tx: TxId) -> impl Future<Output = Result<()>> + Send;

    /// Rolls back the entire index state to a specific transaction ID.
    fn rollback_to_tx(&self, tx_id: TxId) -> impl Future<Output = Result<()>> + Send;

    /// Returns the last transaction ID processed by the index.
    fn last_tx_id(&self) -> impl Future<Output = Result<TxId>> + Send;

    /// Returns the number of documents in the index.
    fn len(&self) -> impl Future<Output = usize> + Send;

    /// Returns true if the index is empty.
    fn is_empty(&self) -> impl Future<Output = bool> + Send {
        async { self.len().await == 0 }
    }

    /// Returns index statistics.
    fn stats(&self) -> impl Future<Output = Result<TextIndexStats>> + Send;
}
```

**`VectorIndex`** — *Vector Index Trait — abstrahiert die HNSW-Vektorsuche. # Dyn-Kompatibilität Verwendet native `async fn` (AFIT) für statischen Dispatch.* (`src/vector_index.rs`)

```rust
pub trait VectorIndex : Send + Sync + 'static {
/// Inserts a vector with an associated document ID.
    fn insert(
        &self,
        tx: TxId,
        id: DocId,
        embedding: &[f32],
    ) -> impl Future<Output = Result<()>> + Send;

    /// Returns all active (non-deleted) document IDs in the index.
    fn all_doc_ids(&self) -> impl Future<Output = Result<Vec<DocId>>> + Send {
        async { Ok(Vec::new()) }
    }

    /// Inserts multiple vectors with associated document IDs.
    fn insert_batch(
        &self,
        tx: TxId,
        vectors: &[(DocId, &[f32])],
    ) -> impl Future<Output = Result<()>> + Send {
        async move {
            for (id, embedding) in vectors {
                self.insert(tx, *id, embedding).await?;
            }
            Ok(())
        }
    }

    /// Searches for the k nearest neighbors to a query vector.
    fn search(
        &self,
        query: &[f32],
        k: usize,
    ) -> impl Future<Output = Result<Vec<ScoredDocument>>> + Send;

    /// Searches for the k nearest neighbors to a query vector at a specific sequence number.
    ///
    /// # Errors
    /// Returns [`ContextraError::CapabilityUnsupported`][crate::ContextraError::CapabilityUnsupported]
    /// with capability `"snapshot_read_at"` if snapshot-isolated vector search is not implemented.
    /// Tested via `capability_coverage` test module.
    fn search_at(
        &self,
        query: &[f32],
        k: usize,
        seq_no: u64,
    ) -> impl Future<Output = Result<Vec<ScoredDocument>>> + Send {
        async move {
            let _ = (query, k, seq_no);
            Err(crate::error::ContextraError::capability_unsupported(
                "snapshot_read_at",
                "Vector search snapshot isolation (search_at) is not supported by default — tracked in ADR-024",
            ))
        }
    }

    /// Searches with an optional filter predicate.
    ///
    /// # Default Behaviour
    /// Returns an error if a filter is provided. Implementors **MUST** override
    /// this method if filtered search is supported by their vector engine.
    ///
    /// # Errors
    /// Returns [`ContextraError::CapabilityUnsupported`][crate::ContextraError::CapabilityUnsupported]
    /// with capability `"vector_filtered_search"` if a filter predicate is passed to an engine without filter support.
    /// Tested via `capability_coverage` test module.
    ///
    /// # Note
    /// This default exists solely for backward compatibility.
    fn search_filtered(
        &self,
        query: &[f32],
        k: usize,
        filter: Option<&(dyn Fn(DocId) -> bool + Send + Sync)>,
    ) -> impl Future<Output = Result<Vec<ScoredDocument>>> + Send {
        async move {
            if filter.is_some() {
                return Err(crate::error::ContextraError::capability_unsupported(
                    "vector_filtered_search",
                    "Filtered vector search is not supported by default for this vector engine",
                ));
            }
            self.search(query, k).await
        }
    }

    /// Deletes a vector by its document ID.
    fn delete(&self, tx: TxId, id: DocId) -> impl Future<Output = Result<()>> + Send;

    /// Commits a transaction.
    fn commit(&self, tx: TxId) -> impl Future<Output = Result<()>> + Send;

    /// Rolls back a transaction.
    fn rollback(&self, tx: TxId) -> impl Future<Output = Result<()>> + Send;

    /// Rolls back the entire index state to a specific transaction ID.
    fn rollback_to_tx(&self, tx_id: TxId) -> impl Future<Output = Result<()>> + Send;

    /// Returns the last transaction ID processed by the index.
    fn last_tx_id(&self) -> impl Future<Output = Result<TxId>> + Send;

    /// Returns the number of vectors in the index.
    fn len(&self) -> impl Future<Output = usize> + Send;

    /// Returns true if the index is empty.
    fn is_empty(&self) -> impl Future<Output = bool> + Send {
        async { self.len().await == 0 }
    }

    /// Returns index statistics.
    fn stats(&self) -> impl Future<Output = Result<VectorIndexStats>> + Send;

    /// Returns true if the index requires a background rebuild (e.g. due to tombstone accumulation).
    fn is_rebuild_required(&self) -> bool {
        false
    }

    /// Triggers an asynchronous background rebuild of the index if supported.
    fn trigger_rebuild_async(&self) {}
}
```

### Öffentliche Datentypen — Structs (25)

| Struct | Felder / Form | Beschreibung | Datei |
|---|---|---|---|
| `AuthorizedRing` | `{ ring: FeatureRing, }` | Token representing an authorized feature ring granting access to engine features (§15). `AuthorizedRing` cannot be directly instantiated outside of t… | `src/license.rs` |
| `ContextSegment` | `{ /// Unique identifier for the chunk. pub chunk_id: u64, /// Text content of the segment. pub text: &'a str, /// Optional model fingerprint reference. pub model… }` | Representation of a context segment passed to context-aware text generation. | `src/embedding.rs` |
| `GraphIndexStats` | `{ /// Number of active nodes (Entities). pub num_entities: usize, /// Number of active edges. pub num_edges: usize, /// Total bytes allocated by CSR representati… }` | Statistics for the GraphIndex layer. | `src/graph_index.rs` |
| `GroundingAssessment` | `{ /// Raw or calibrated confidence score in [0.0, 1.0] indicating attribution quality. pub score: f32, /// Indicates whether the grounding score meets the requir… }` | Result of a post-hoc grounding / attribution validation check. | `src/lifecycle.rs` |
| `HyperEdgeId` | `(tuple/unit)` | — | `src/graph.rs` |
| `KvBlock` | `{ /// Block sequence identifier. pub block_id: u64, /// Encrypted or raw block tensor bytes. #[serde( serialize_with = "serialize_bytes", deserialize_with = "des… }` | Represents an exported KV block segment. | `src/kv.rs` |
| `KvLayout` | `{ /// Number of transformer layers. pub n_layer: u32, /// Number of KV heads. pub n_kv_head: u32, /// Dimension per head. pub head_dim: u32, /// Data type repres… }` | KV cache layer and dimensions layout configuration. | `src/kv.rs` |
| `KvPrefixHit` | `{ /// Number of tokens matched by the prefix. pub matched_tokens: usize, /// Extracted KV blocks for the matched prefix. pub blocks: Vec<KvBlock>, }` | Represents a prefix match hit in the KV store. | `src/kv.rs` |
| `MockEmbedder` | `{ /// Target embedding dimensionality. pub dim: usize, /// Fixed output vector for deterministic testing. pub fixed_output: Option<Vec<f32>>, }` | Deterministic mock embedding provider for tests. | `src/embedding.rs` |
| `NoopMetricsSink` | `(tuple/unit)` | A no-op implementation of [`MetricsSink`] that silently discards all metrics events. | `src/metrics.rs` |
| `OpenFastGate` | `(tuple/unit)` | Default open activation gate allowing `Fast` ring features without restriction. | `src/license.rs` |
| `PluginCapability` | `{ /// Unique plugin name. pub name: &'static str, /// Plugin version tuple (major, minor, patch). pub version: (u32, u32, u32), /// Architecture ring in which th… }` | Capability descriptor for a plugin. | `src/plugin.rs` |
| `PluginRegistry` | `{ active: BTreeMap<String, PluginCapability>, active_conflicts: BTreeMap<String, Vec<String>>, gate: Arc<dyn LicenseGate>, }` | Registry managing plugin activations, license gating, and dependency resolution. | `src/plugin.rs` |
| `PrefixKey` | `{ /// Target model fingerprint. pub model: ModelFingerprint, /// SHA256/BLAKE3 hash of the tokenizer vocabulary. pub tokenizer_hash: [u8; 32], /// KV tensor memo… }` | Key identifying a model KV cache prefix structure. | `src/kv.rs` |
| `RequestId` | `(tuple/unit)` | Unique identifier for an inference request within a process. | `src/attention.rs` |
| `RoleBinding` | `{ /// The role assigned to the entity in the hyperedge. pub role: RoleId, /// The bound entity. pub entity: EntityId, }` | Binding of an entity to a specific role within an n-ary hyperedge. | `src/graph.rs` |
| `RoleId` | `(tuple/unit)` | — | `src/graph.rs` |
| `RopeConfig` | `{ /// Base frequency. pub base: f32, /// Optional frequency scaling factor. pub scaling: Option<f32>, }` | Rotary position embedding (RoPE) configuration. | `src/kv.rs` |
| `SeededRng` | `{ state: AtomicU64, }` | A fast, deterministic 64-bit pseudorandom number generator using SplitMix64 with atomic state transitions (`AtomicU64`). Lock-free and thread-safe wi… | `src/rng.rs` |
| `SequentialIdGen` | `{ next: AtomicU64, }` | A thread-safe ID generator that produces monotonically increasing 64-bit IDs. Wraps around safely on integer overflow (`u64::MAX` -> `0`) via atomic… | `src/id_gen.rs` |
| `StorageStats` | `{ /// Number of SSTable segments. pub num_segments: usize, /// Total size of all SSTables in bytes. pub total_size_bytes: u64, /// Total size of memtables in byt… }` | Statistics for the storage engine. | `src/storage.rs` |
| `SystemClock` | `{ start_instant: Instant, }` | Standard production implementation of [`Clock`] backed by [`SystemTime`] and [`Instant`]. | `src/clock.rs` |
| `TestMetricsSink` | `{ events: std::sync::Arc<std::sync::Mutex<Vec<MetricEvent>>>, }` | An in-memory [`MetricsSink`] implementation for testing that records all emitted metric events. | `src/metrics.rs` |
| `TextIndexStats` | `{ /// Total number of documents indexed. pub num_documents: usize, /// Total number of tokens across all documents. pub num_tokens: usize, /// Estimated memory u… }` | Statistics for a text index. | `src/text_index.rs` |
| `VectorIndexStats` | `{ /// Number of active (non-deleted) vectors. pub num_vectors: usize, /// Estimated memory usage in bytes. pub memory_usage_bytes: usize, /// Number of HNSW laye… }` | Statistics for a vector index. | `src/vector_index.rs` |

### Öffentliche Datentypen — Enums (7)

| Enum | Varianten | Beschreibung | Datei |
|---|---|---|---|
| `EmbeddingError` | `/// Provider is unreachable or unavailable. #[error("Provider unavailable: {0}")] Unavailable(String), /// Computation failed during embedding generation. #[error("Embedding failed: {0}")] Computatio…` | Error types encountered during embedding operations. | `src/embedding.rs` |
| `FeatureRing` | `/// Fast ring (open source core). Fast, /// Sovereign ring (enterprise data sovereignty). Sovereign, /// Compliance ring (strict auditability and regulatory rules). Compliance,` | Feature-Ring gemäß Spec §15. | `src/license.rs` |
| `GraphMutationError` | `/// Lock acquisition timed out during concurrent mutation. #[error("Lock acquisition timed out: {0}")] LockAcquisitionTimeout(String), /// Provided role binding is invalid. #[error("Invalid role bind…` | Error variants for graph mutation operations. | `src/graph.rs` |
| `LicenseError` | `/// No valid activation found for the requested feature ring. #[error("no valid activation found for feature ring {0:?}")] NotActivated(FeatureRing), /// Activation signature verification failed. #[e…` | Fehlerklasse für Lizenz-/Aktivierungsprüfung — crate-lokal, an der Grenze in `ContextraError` konvertierbar (P6-konform, siehe Spec §17). | `src/license.rs` |
| `MetricEvent` | `/// Counter increment event. Counter { /// Metric name. name: String, /// Incremented value. value: u64, /// Metric key-value labels. labels: Vec<(String, String)>, }, /// Gauge setting event. Gauge…` | A recorded metric event captured by [`TestMetricsSink`]. | `src/metrics.rs` |
| `PluginError` | `/// Dependency cycle or unresolvable missing dependency detected among plugins. #[error("dependency cycle detected among plugins: {0:?}")] DependencyCycle(Vec<String>), /// Conflicting plugins reques…` | Errors during plugin registration, activation, or dependency resolution. | `src/plugin.rs` |
| `VectorDeleteMode` | `/// Synchronous graph repair during vector deletion. SynchronousRepair, /// Asynchronous background graph repair during vector deletion. BackgroundRepair,` | Vector deletion mode for vector index maintenance (§15). | `src/license.rs` |

### Typ-Aliase (2)

| Alias | Definition | Beschreibung |
|---|---|---|
| `BoxFuture` | `type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>` | Type alias for a pinned, heap-allocated `Future` that is `Send` and dyn-compatible. |
| `BoxStream` | `type BoxStream<'a, T> = Pin<Box<dyn futures_util::stream::Stream<Item = T> + Send + 'a>>` | Type alias for a pinned, heap-allocated `Stream` that is `Send` and dyn-compatible. |

### Konstanten (1)

| Konstante | Typ | Beschreibung |
|---|---|---|
| `MAX_SCAN_MERGE_ACCUMULATOR` | `usize` | Harte Obergrenze für die Anzahl distinkter Keys, die eine StorageEngine- Implementierung während eines einzelnen scan()/scan_prefix_bounded()-Aufrufs… |

### Öffentliche Funktionen (23)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn new() -> Self` | Creates a new [`SystemClock`] using the current [`Instant`] as the monotonic baseline. | `src/clock.rs` |
| `pub fn fixed_unit(dim: usize) -> Self` | Creates a mock embedder with standard fixed unit output. | `src/embedding.rs` |
| `pub fn new(chunk_id: u64, text: &'a str) -> Self` | Creates a new `ContextSegment` with default optional metadata set to `None`. | `src/embedding.rs` |
| `pub fn new(dim: usize) -> Self` | Creates a new `MockEmbedder` with the specified dimension. | `src/embedding.rs` |
| `pub fn with_fingerprint(mut self, fingerprint: &'a ModelFingerprint) -> Self` | Builder method to set the model fingerprint. | `src/embedding.rs` |
| `pub fn with_fixed_output(fixed_output: Vec<f32>) -> Self` | Creates a new `MockEmbedder` with a fixed output vector. | `src/embedding.rs` |
| `pub fn with_offset(chunk_id: u64, text: &'a str, offset: usize) -> Self` | Constructs a `ContextSegment` initialized with a RoPE positional offset. | `src/embedding.rs` |
| `pub fn with_rope_offset(mut self, offset: usize) -> Self` | Builder method to set the RoPE offset. | `src/embedding.rs` |
| `pub fn new(role: RoleId, entity: EntityId) -> Self` | Creates a new `RoleBinding`. | `src/graph.rs` |
| `pub fn new(start: u64) -> Self` | Creates a new [`SequentialIdGen`] starting at `start`. | `src/id_gen.rs` |
| `pub fn ring(&self) -> FeatureRing` | Returns the authorized [`FeatureRing`]. | `src/license.rs` |
| `pub fn clear(&self)` | Clears all recorded metric events. | `src/metrics.rs` |
| `pub fn events(&self) -> Vec<MetricEvent>` | Returns a snapshot copy of all recorded metric events. | `src/metrics.rs` |
| `pub fn new() -> Self` | Creates a new empty [`TestMetricsSink`]. | `src/metrics.rs` |
| `pub fn activate(&mut self, plugin: Box<dyn PluginManifest>) -> Result<(), PluginError>` | Activates a single plugin instance by delegating to [`activate_all`](Self::activate_all). | `src/plugin.rs` |
| `pub fn activate_all( &mut self, plugins: Vec<Box<dyn PluginManifest>>, ) -> Result<(), PluginError>` | Activates a batch of plugins in an all-or-nothing manner (INV-PLUGIN-DEPENDENCY). Sequence: 1. Conflict checks across requested and active… | `src/plugin.rs` |
| `pub fn current_feature_ring(&self) -> FeatureRing` | Returns the highest currently active `FeatureRing` according to the license gate. | `src/plugin.rs` |
| `pub fn is_active(&self, name: &str) -> bool` | Checks whether a plugin with the given name is currently active. | `src/plugin.rs` |
| `pub fn new( name: &'static str, version: (u32, u32, u32), ring: u8, feature_ring_required: FeatureRing, ) -> Self` | Creates a new `PluginCapability`. | `src/plugin.rs` |
| `pub fn new(gate: Arc<dyn LicenseGate>) -> Self` | Creates a new `PluginRegistry` bounded by the given `LicenseGate`. | `src/plugin.rs` |
| `pub fn snapshot(&self) -> Vec<PluginCapability>` | Takes a snapshot of all active plugin capabilities. | `src/plugin.rs` |
| `pub fn from_entropy() -> Self` | Creates a new [`SeededRng`] seeded from process/runtime entropy. # Non-Cryptographic Usage This entropy source uses [`std::collections::has… | `src/rng.rs` |
| `pub fn new(seed: u64) -> Self` | Creates a new [`SeededRng`] initialized with the given seed. | `src/rng.rs` |


---

## `contextra-mvcc`

| Attribut | Wert |
|---|---|
| **Ring** | 0 — Ring 0 — Foundation & Pure Numerics/Types |
| **Status** | Fertig |
| **Zweck** | SeqLog, SnapshotRegistry und transaktionaler TxBuffer (MVCC-Isolation). |
| **Unsafe-Code** | Nein (`#![forbid(unsafe_code)]`) |
| **Interne Abhängigkeiten** | `contextra-ports`, `contextra-types` |
| **Externe Abhängigkeiten (Auswahl)** | `ahash`, `bytes`, `parking_lot`, `serde`, `thiserror`, `tracing` |

### Modul- und Re-Export-Oberfläche (lib.rs)

```rust
pub mod seq_log;
pub mod snapshot;
pub mod ssi;
pub mod tx_buffer;
pub use contextra_types::*;
pub use seq_log::{SeqLogChange, SeqLogEntry, SequenceLog};
pub use snapshot::{SnapshotGuard, SnapshotRegistry};
pub use ssi::{PruningBlockerInfo, ReadSet, SequenceLogSsiValidator, SsiValidator};
pub use tx_buffer::{IndexOp, TxBuffer};
```

### Schnittstellen — Public Traits (2)

**`SsiValidator`** — *Trait for Serializable Snapshot Isolation (SSI) validation. Implementations evaluate whether concurrent committed write transactions have mutated any keys in the transaction's [`ReadSet`] after the snapshot sequence numbers recorded when those keys were read.* (`src/ssi.rs`)

```rust
pub trait SsiValidator {
/// Validates the [`ReadSet`] for transaction `tx_id`.
    ///
    /// # Errors
    /// Returns `Err(ContextraError::Conflict(...))` if a key in `read_set` was committed by a
    /// concurrent transaction at a sequence number strictly greater than the key's snapshot sequence number.
    ///
    /// # INVARIANT (contextra-store Integration)
    /// `contextra-store` assumes serializable commit validation ordering (e.g. via `commit_mutex` or engine pipeline)
    /// during `validate` and `record_commit_key` calls to guarantee atomic, write-skew-free commits.
    fn validate(&self, tx_id: TxId, read_set: &ReadSet) -> Result<()>;
}
```

**`StagedOpSize`** — *Trait for estimating the byte footprint of a staged payload.* (`src/tx_buffer.rs`)

```rust
pub trait StagedOpSize {
/// Estimates memory footprint in bytes for staging entry overhead accounting.
    fn staged_bytes(&self) -> usize;
}
```

### Öffentliche Datentypen — Structs (10)

| Struct | Felder / Form | Beschreibung | Datei |
|---|---|---|---|
| `PruningBlockerInfo` | `{ /// The lowest sequence number currently unpruned in the validator register. pub min_unpruned_seq: u64, /// The current `pruned_through` watermark sequence num… }` | Diagnostic information identifying an unreleased snapshot or sequence that blocks pruning in [`SequenceLogSsiValidator`]. | `src/ssi.rs` |
| `ReadSet` | `{ keys: AHashMap<Vec<u8>, u64>, prefixes: AHashMap<Vec<u8>, u64>, min_snapshot_seq: Option<u64>, }` | Tracked read keys, range prefixes, and their snapshot sequence numbers for Serializable Snapshot Isolation (SSI). # Memory Overhead & Risk Analysis M… | `src/ssi.rs` |
| `SeqLogEntry` | `{ /// Document ID. pub doc_id: DocId, /// Sequence number at which this document entry was inserted. pub insert_seq: u64, /// Sequence number at which this docum… }` | Versioned sequence log entry for snapshot isolation (`_at` family). **Memory Overhead**: Each entry is 24 bytes (8 bytes `DocId` + 8 bytes `insert_se… | `src/seq_log.rs` |
| `SequenceLog` | `{ entries: Vec<SeqLogEntry>, pinned_snapshots: ahash::AHashMap<u64, Vec<Instant>>, compacted_below: Option<u64>, deletions: ahash::AHashMap<DocId, u64>, max_pin_… }` | Helper structure managing sequence log tracking and visibility filtering for index implementations. | `src/seq_log.rs` |
| `SequenceLogSsiValidator` | `{ committed_writes: Arc<RwLock<CommittedWrites>>, sequence_log: Option<Arc<RwLock<SequenceLog>>>, snapshot_registry: Option<Arc<SnapshotRegistry>>, pruned_throug… }` | Reference implementation of [`SsiValidator`] backed by MVCC [`SequenceLog`] and committed key tracking. Operates strictly in-memory without disk I/O… | `src/ssi.rs` |
| `SequenceLogSsiValidatorBuilder` | `{ max_tracked_keys: Option<usize>, sequence_log: Option<Arc<RwLock<SequenceLog>>>, snapshot_registry: Option<Arc<SnapshotRegistry>>, max_pin_duration: Option<Dur… }` | Builder for constructing a [`SequenceLogSsiValidator`]. // TODO(wiring): Facade-Anbindung in contextra/src/builder.rs folgt in separatem Task | `src/ssi.rs` |
| `SnapshotGuard` | `{ registry: Arc<SnapshotRegistry>, seq_no: u64, created_at: Option<Instant>, }` | RAII Guard for an active snapshot. | `src/snapshot.rs` |
| `SnapshotRegistry` | `{ active: Mutex<BTreeMap<u64, Vec<Instant>>>, min_active_seqno: AtomicU64, }` | Registry for active read snapshots. ### Synchronization & Memory Ordering Strategy - **Single-Writer Exclusivity via Mutex**: All modifications to ac… | `src/snapshot.rs` |
| `TxBuffer` | `{ shards: Vec<RwLock<TxShard<T>>>, key_shards: Vec<RwLock<KeyShard>>, tx_timeout: Duration, config: TxBufferConfig, global_staged_bytes: AtomicUsize, }` | Buffers index operations until commit or rollback. Sharded into sub-buffers to reduce lock contention. # INVARIANT Each shard is protected by an inde… | `src/tx_buffer.rs` |
| `TxBufferConfig` | `{ /// Transaction timeout duration. pub tx_timeout: Duration, /// Maximum recommended active transactions. pub max_active_tx: usize, /// Maximale Anzahl von Stag… }` | Configuration options for `TxBuffer`. | `src/tx_buffer.rs` |

### Öffentliche Datentypen — Enums (2)

| Enum | Varianten | Beschreibung | Datei |
|---|---|---|---|
| `IndexOp` | `/// Insert a document with associated data. Insert { /// Document identifier to insert. doc_id: DocId, /// Associated payload or index data. data: T, }, /// Delete a document. Delete { /// Document i…` | Operation to be executed in an index. | `src/tx_buffer.rs` |
| `SeqLogChange` | `/// A document insertion at a specific sequence number. Insert { /// Document ID. doc_id: DocId, /// Sequence number of the insert. seq: u64, }, /// A document deletion at a specific sequence number.…` | Represents a historical sequence log change (insert or delete) for delta replaying. | `src/seq_log.rs` |

### Konstanten (8)

| Konstante | Typ | Beschreibung |
|---|---|---|
| `DEFAULT_MAX_OPS_PER_TX` | `usize` | — |
| `DEFAULT_MAX_PIN_DURATION` | `Duration` | Default maximum pin duration (5 minutes) before diagnostic warnings are issued for expired pins. |
| `DEFAULT_MAX_READ_SET_KEYS` | `usize` | Default maximum number of read keys tracked per transaction's [`ReadSet`]. Prevents unbounded memory growth during large read scans (~100,000 keys *… |
| `DEFAULT_MAX_TOTAL_STAGED_BYTES` | `usize` | Default maximum total staged bytes across all active transactions (256 MiB). Guards the process memory budget against multi-transaction staging memor… |
| `DEFAULT_MAX_TRACKED_COMMIT_KEYS` | `usize` | Default maximum tracked committed write keys in [`SequenceLogSsiValidator`] (1,000,000 keys). |
| `DEFAULT_MAX_TX_STAGED_BYTES` | `usize` | Default maximum staged bytes per transaction (16 MiB). Bounds staging memory per individual transaction while enabling high-throughput bulk operation… |
| `DEFAULT_SHARD_COUNT` | `usize` | Default number of shards for the transaction buffer. |
| `STAGING_ENTRY_OVERHEAD_BYTES` | `usize` | Single canonical constant for overhead calculation per staged entry (32 bytes). |

### Öffentliche Funktionen (109)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn changes_since(&self, snapshot_seq: u64) -> Vec<SeqLogChange>` | Returns all sequence log changes that occurred strictly after `snapshot_seq`, ordered chronologically by sequence number. | `src/seq_log.rs` |
| `pub fn compact(&mut self, min_active_seqno: u64)` | Compacts log entries where deletion sequence number is strictly less than `min_active_seqno`. | `src/seq_log.rs` |
| `pub fn deletion_seq(&self, doc_id: DocId) -> Option<u64>` | Returns the deletion sequence number of the given document, if currently soft-deleted. | `src/seq_log.rs` |
| `pub fn doc_id(&self) -> DocId` | Returns the document ID associated with this change. | `src/seq_log.rs` |
| `pub fn expired_pins(&self) -> Vec<u64>` | Returns all pinned sequence numbers whose active pin duration exceeds `max_pin_duration`. | `src/seq_log.rs` |
| `pub fn expired_pins_at(&self, now: Instant) -> Vec<u64>` | Returns all pinned sequence numbers whose active pin duration exceeds `max_pin_duration` relative to the provided timestamp `now`. | `src/seq_log.rs` |
| `pub fn is_empty(&self) -> bool` | Returns true if the sequence log is empty. | `src/seq_log.rs` |
| `pub fn is_visible(&self, as_of: u64) -> bool` | Returns `true` if this log entry is visible at snapshot sequence number `as_of`. # Visibility Rule An entry is visible if `insert_seq <= as… | `src/seq_log.rs` |
| `pub fn is_visible(&self, doc_id: DocId, seq_no: u64) -> bool` | Checks if `doc_id` was inserted at or before `seq_no` and not deleted at or before `seq_no`. | `src/seq_log.rs` |
| `pub fn len(&self) -> usize` | Returns the number of entries in the sequence log. | `src/seq_log.rs` |
| `pub fn max_pin_duration(&self) -> Duration` | Returns the configured maximum pin duration. | `src/seq_log.rs` |
| `pub fn min_retention_seq(&self) -> Option<u64>` | Calculates the minimum sequence number currently pinned for snapshot retention. Returns `None` if no snapshot sequence numbers are pinned.… | `src/seq_log.rs` |
| `pub fn min_retention_seq_at(&self, now: Instant) -> Option<u64>` | Calculates the minimum sequence number currently pinned for snapshot retention at timestamp `now`. Returns `None` if no snapshot sequence n… | `src/seq_log.rs` |
| `pub fn new() -> Self` | Creates a new empty sequence log with default max pin duration (5 minutes). | `src/seq_log.rs` |
| `pub fn pin_snapshot(&mut self, seq_no: u64)` | Pins a historical sequence number to prevent rebuild from purging soft-deleted nodes active at this snapshot. | `src/seq_log.rs` |
| `pub fn pin_snapshot_at(&mut self, seq_no: u64, at: Instant)` | Pins a historical sequence number at a specific timestamp for deterministic testing and lease tracking. | `src/seq_log.rs` |
| `pub fn record_delete(&mut self, doc_id: DocId, seq: u64)` | Records a delete operation at the given sequence number `seq`. | `src/seq_log.rs` |
| `pub fn record_insert(&mut self, doc_id: DocId, seq: u64)` | Records an insert operation at the given sequence number `seq`. | `src/seq_log.rs` |
| `pub fn seq(&self) -> u64` | Returns the sequence number associated with this change. | `src/seq_log.rs` |
| `pub fn set_max_pin_duration(&mut self, duration: Duration)` | Sets a custom maximum pin duration for diagnostics. | `src/seq_log.rs` |
| `pub fn unpin_snapshot(&mut self, seq_no: u64)` | Unpins a historical sequence number. | `src/seq_log.rs` |
| `pub fn with_max_pin_duration(mut self, duration: Duration) -> Self` | Sets a custom maximum pin duration for diagnostics and returns `self`. | `src/seq_log.rs` |
| `pub fn longest_active_pin(&self) -> Option<(u64, Duration)>` | Identifies which active snapshot sequence number has been held the longest, returning `Some((seq_no, duration))` or `None` if no snapshots… | `src/snapshot.rs` |
| `pub fn longest_active_pin_at(&self, now: Instant) -> Option<(u64, Duration)>` | Identifies which active snapshot sequence number has been held the longest relative to `now`, returning `Some((seq_no, duration))` or `None… | `src/snapshot.rs` |
| `pub fn min_active_seqno(&self) -> u64` | Returns the minimum active sequence number (`u64::MAX` if none). Uses `Ordering::Acquire` to synchronize with `Ordering::Release` writes in… | `src/snapshot.rs` |
| `pub fn new() -> Self` | Creates a new empty SnapshotRegistry. | `src/snapshot.rs` |
| `pub fn pin(&self, seq_no: u64)` | Persistent pin of a sequence number to prevent GC (SAOS Checkpoint). | `src/snapshot.rs` |
| `pub fn pin_at(&self, seq_no: u64, at: Instant)` | Persistent pin of a sequence number at a specific timestamp `at`. | `src/snapshot.rs` |
| `pub fn register(self: &Arc<Self>, seq_no: u64) -> SnapshotGuard` | Registers a read snapshot. Returns an RAII guard that automatically deregisters on drop. | `src/snapshot.rs` |
| `pub fn register_at(self: &Arc<Self>, seq_no: u64, at: Instant) -> SnapshotGuard` | Registers a read snapshot at a specific creation timestamp `at`. | `src/snapshot.rs` |
| `pub fn seq_no(&self) -> u64` | Returns the sequence number pinned by this snapshot guard. | `src/snapshot.rs` |
| `pub fn unpin(&self, seq_no: u64)` | Removes a persistent pin. | `src/snapshot.rs` |
| `pub fn build(self) -> SequenceLogSsiValidator` | Builds the configured [`SequenceLogSsiValidator`]. | `src/ssi.rs` |
| `pub fn builder() -> SequenceLogSsiValidatorBuilder` | Creates a builder to configure and instantiate a [`SequenceLogSsiValidator`]. | `src/ssi.rs` |
| `pub fn clear(&mut self)` | Clears all tracked read keys, range prefixes, and resets the cached minimum snapshot sequence. | `src/ssi.rs` |
| `pub fn diagnose_pruning_blocker(&self) -> Option<PruningBlockerInfo>` | Diagnoses which unpruned sequence or active snapshot is currently blocking pruning in this validator. | `src/ssi.rs` |
| `pub fn diagnose_pruning_blocker_at(&self, now: Instant) -> Option<PruningBlockerInfo>` | Diagnoses which unpruned sequence or active snapshot is currently blocking pruning relative to timestamp `now`. | `src/ssi.rs` |
| `pub fn forget_from(&self, first_seq: u64) -> usize` | Removes all committed key registrations with `commit_seq >= first_seq`. Used when WAL append or commit execution fails after keys were regi… | `src/ssi.rs` |
| `pub fn get(&self, key: &[u8]) -> Option<u64>` | Returns the snapshot sequence number recorded for `key`, if present. | `src/ssi.rs` |
| `pub fn is_empty(&self) -> bool` | Returns `true` if no read keys or range prefixes are tracked in this set. | `src/ssi.rs` |
| `pub fn iter(&self) -> impl Iterator<Item = (&Vec<u8>, &u64)>` | Returns an iterator over tracked read keys and their snapshot sequence numbers. | `src/ssi.rs` |
| `pub fn keys_map(&self) -> &AHashMap<Vec<u8>, u64>` | Returns the inner map of read keys to snapshot sequence numbers. | `src/ssi.rs` |
| `pub fn len(&self) -> usize` | Returns the total number of read keys and range prefixes tracked in this set. | `src/ssi.rs` |
| `pub fn max_tracked_keys(&self) -> usize` | Returns the configured maximum commit key tracking limit. | `src/ssi.rs` |
| `pub fn min_snapshot_seq(&self) -> Option<u64>` | Returns the lowest snapshot sequence number recorded across all read keys and prefixes in this set, or `None` if the set is empty. | `src/ssi.rs` |
| `pub fn new() -> Self` | Creates a new empty [`ReadSet`]. | `src/ssi.rs` |
| `pub fn new() -> Self` | Creates a new builder instance. | `src/ssi.rs` |
| `pub fn new() -> Self` | Creates a new [`SequenceLogSsiValidator`] with default maximum commit key capacity (1,000,000 keys). | `src/ssi.rs` |
| `pub fn new_with_bounds(max_tracked_keys: usize) -> Self` | Creates a new [`SequenceLogSsiValidator`] with specified maximum commit key bounds `max_tracked_keys`. | `src/ssi.rs` |
| `pub fn prefixes_iter(&self) -> impl Iterator<Item = (&Vec<u8>, &u64)>` | Returns an iterator over tracked range prefixes and their snapshot sequence numbers. | `src/ssi.rs` |
| `pub fn prefixes_map(&self) -> &AHashMap<Vec<u8>, u64>` | Returns the inner map of range prefixes to snapshot sequence numbers. | `src/ssi.rs` |
| `pub fn prune_through(&self, bound_seq: u64) -> usize` | Prunes tracked committed writes with `commit_seq <= bound_seq`. Atomically updates `pruned_through` watermark monotonically under the `comm… | `src/ssi.rs` |
| `pub fn pruned_through(&self) -> u64` | Returns the current `pruned_through` watermark sequence number. | `src/ssi.rs` |
| `pub fn record_commit_key(&self, key: &[u8], commit_seq: u64)` | Records a committed write for `key` at sequence number `commit_seq`. # INVARIANT (contextra-store Integration) `contextra-store` assumes st… | `src/ssi.rs` |
| `pub fn record_commit_keys<'a>( &self, keys: impl IntoIterator<Item = &'a [u8]>, commit_seq: u64, )` | Records committed writes for multiple keys at sequence number `commit_seq`. | `src/ssi.rs` |
| `pub fn record_prefix(&mut self, prefix: impl Into<Vec<u8>>, snapshot_seq: u64)` | Records a range prefix read operation at `snapshot_seq` for phantom protection. SSI protects point reads directly; range scans are protecte… | `src/ssi.rs` |
| `pub fn record_read(&mut self, key: impl Into<Vec<u8>>, snapshot_seq: u64)` | Records a read operation for `key` at snapshot sequence `snapshot_seq`. If the key was previously recorded in the same transaction, the low… | `src/ssi.rs` |
| `pub fn register_read(&mut self, key: impl Into<Vec<u8>>, snapshot_seq: u64)` | Alias for [`ReadSet::record_read`]. | `src/ssi.rs` |
| `pub fn set_metrics_sink( &mut self, sink: Arc<parking_lot::RwLock<Arc<dyn contextra_ports::MetricsSink>>>, )` | Sets or updates the metrics sink container attached to this validator. | `src/ssi.rs` |
| `pub fn set_snapshot_registry(&mut self, registry: Arc<SnapshotRegistry>)` | Attaches an active [`SnapshotRegistry`] to this validator for active snapshot pin duration tracking. | `src/ssi.rs` |
| `pub fn tracked_commit_keys(&self) -> usize` | Returns the number of committed keys currently tracked. | `src/ssi.rs` |
| `pub fn validate_and_record<'a>( &self, tx_id: TxId, read_set: &ReadSet, write_keys: impl IntoIterator<Item = &'a [u8]>, commit_seq: u64, ) -> Result<()>` | Atomically validates `read_set` and records `write_keys` at `commit_seq` under a single write lock. | `src/ssi.rs` |
| `pub fn with_max_pin_duration(mut self, duration: Duration) -> Self` | Sets a custom maximum pin duration. | `src/ssi.rs` |
| `pub fn with_max_pin_duration(mut self, duration: Duration) -> Self` | Sets a custom maximum pin duration before diagnostic alarms are triggered for unreleased snapshots. | `src/ssi.rs` |
| `pub fn with_max_tracked_keys(mut self, limit: usize) -> Self` | Configures the maximum tracked keys bound. | `src/ssi.rs` |
| `pub fn with_metrics_sink( mut self, sink: Arc<parking_lot::RwLock<Arc<dyn contextra_ports::MetricsSink>>>, ) -> Self` | Attaches a metrics sink container. | `src/ssi.rs` |
| `pub fn with_metrics_sink( mut self, sink: Arc<parking_lot::RwLock<Arc<dyn contextra_ports::MetricsSink>>>, ) -> Self` | Attaches a metrics sink container to this validator for observability reporting. # Architecture Decision & Invariant (v17 Teil 2.2 & Teil 4… | `src/ssi.rs` |
| `pub fn with_sequence_log(mut self, seq_log: Arc<RwLock<SequenceLog>>) -> Self` | Attaches an existing [`SequenceLog`]. | `src/ssi.rs` |
| `pub fn with_sequence_log(seq_log: Arc<RwLock<SequenceLog>>) -> Self` | Creates a new [`SequenceLogSsiValidator`] attached to an existing [`SequenceLog`]. | `src/ssi.rs` |
| `pub fn with_snapshot_registry(mut self, registry: Arc<SnapshotRegistry>) -> Self` | Attaches an active [`SnapshotRegistry`]. | `src/ssi.rs` |
| `pub fn with_snapshot_registry(mut self, registry: Arc<SnapshotRegistry>) -> Self` | Attaches an active [`SnapshotRegistry`] to this validator for active snapshot pin duration tracking. | `src/ssi.rs` |
| `pub fn begin(&self, tx: TxId)` | Registers a new transaction in the buffer. | `src/tx_buffer.rs` |
| `pub fn begin_at(&self, tx: TxId, at: Instant)` | Registers a new transaction in the buffer at timestamp `at`. | `src/tx_buffer.rs` |
| `pub fn discard(&self, tx: TxId)` | Discards all buffered operations and tracked read-sets for a transaction. | `src/tx_buffer.rs` |
| `pub fn discard_kv(&self, tx: TxId)` | Discards all buffered operations for a key-value transaction, cleaning up key staging. | `src/tx_buffer.rs` |
| `pub fn doc_id(&self) -> DocId` | Returns the document ID for this operation. | `src/tx_buffer.rs` |
| `pub fn drain(&self, tx: TxId) -> Vec<IndexOp<T>>` | Drains and returns all buffered operations for a transaction. Returns an empty vector if the transaction does not exist or has no operation… | `src/tx_buffer.rs` |
| `pub fn drain_kv(&self, tx: TxId) -> Vec<IndexOp<(Vec<u8>, Vec<u8>)>>` | Drains and returns all buffered operations for a key-value transaction, cleaning up key staging. # INVARIANT (contextra-store Integration)… | `src/tx_buffer.rs` |
| `pub fn estimated_bytes(&self) -> usize` | Estimates total memory footprint in bytes including fixed overhead and payload size. | `src/tx_buffer.rs` |
| `pub fn get_ops(&self, tx: TxId) -> Option<Vec<IndexOp<T>>>` | Returns a clone of the pending operations for a transaction. | `src/tx_buffer.rs` |
| `pub fn get_read_set(&self, tx: TxId) -> Option<ReadSet>` | Returns a clone of the accumulated [`ReadSet`] for transaction `tx`, if present. | `src/tx_buffer.rs` |
| `pub fn has_tx(&self, tx: TxId) -> bool` | Checks if the given transaction exists in the buffer. | `src/tx_buffer.rs` |
| `pub fn is_empty(&self) -> bool` | Returns true if all shards are empty. WARNING: Like `len()`, this is not an atomic operation across all shards. | `src/tx_buffer.rs` |
| `pub fn is_key_staged_for_tx(&self, tx_id: TxId, key: &[u8]) -> bool` | Checks if `key` is staged for insertion in the specified transaction `tx_id`. This method is atomic because all operations for a given `tx_… | `src/tx_buffer.rs` |
| `pub fn len(&self) -> usize` | Returns the total number of pending transactions. WARNING: This iterates over all shards with individual read locks. The result is NOT an a… | `src/tx_buffer.rs` |
| `pub fn min_read_snapshot(&self) -> Option<u64>` | Returns the lowest snapshot sequence number across all read sets of all active transactions in all shards. # Concurrency & Atomicity Note I… | `src/tx_buffer.rs` |
| `pub fn new() -> Self` | Creates a new buffer with default 64 shards and 30s timeout. | `src/tx_buffer.rs` |
| `pub fn new_with_config(shard_count: usize, tx_timeout: Duration) -> Self` | Creates a new buffer with custom settings. If `shard_count` is 0, it defaults to 1 to prevent division-by-zero in `shard_idx()` (§2 Zero-Pa… | `src/tx_buffer.rs` |
| `pub fn new_with_config_ext( shard_count: usize, _tx_timeout: Duration, config: TxBufferConfig, ) -> Self` | Creates a new buffer with explicit `TxBufferConfig`. | `src/tx_buffer.rs` |
| `pub fn read_set(&self, tx: TxId) -> Option<ReadSet>` | Alias for [`TxBuffer::get_read_set`]. | `src/tx_buffer.rs` |
| `pub fn reap_orphans(&self) -> Vec<TxId>` | Cleans up expired transactions. Reaps expired orphan transactions. # INVARIANT Acquires shard locks sequentially in ascending index order (… | `src/tx_buffer.rs` |
| `pub fn reap_orphans_at(&self, now: Instant) -> Vec<TxId>` | Cleans up expired transactions relative to timestamp `now`. Reaps expired orphan transactions. # INVARIANT Acquires shard locks sequentiall… | `src/tx_buffer.rs` |
| `pub fn reap_orphans_bounded(&self, max: usize) -> Vec<TxId>` | Reaps up to `max` expired orphan transactions across shards. # INVARIANT Acquires shard locks sequentially in ascending index order (0 to N… | `src/tx_buffer.rs` |
| `pub fn reap_orphans_bounded_at(&self, max: usize, now: Instant) -> Vec<TxId>` | Reaps up to `max` expired orphan transactions across shards relative to timestamp `now`. # INVARIANT Acquires shard locks sequentially in a… | `src/tx_buffer.rs` |
| `pub fn record_read(&self, tx: TxId, key: impl Into<Vec<u8>>, snapshot_seq: u64)` | Alias for [`TxBuffer::register_read`]. | `src/tx_buffer.rs` |
| `pub fn register_prefix_read(&self, tx: TxId, prefix: impl Into<Vec<u8>>, snapshot_seq: u64)` | Registers a range prefix read operation and its snapshot sequence number for transaction `tx`. | `src/tx_buffer.rs` |
| `pub fn register_read(&self, tx: TxId, key: impl Into<Vec<u8>>, snapshot_seq: u64)` | Registers a read key and its snapshot sequence number for transaction `tx`. | `src/tx_buffer.rs` |
| `pub fn stage(&self, tx: TxId, op: IndexOp<T>) -> Result<()> where T: StagedOpSize,` | Stages an operation for the given transaction, checking bounded capacity and byte budget. | `src/tx_buffer.rs` |
| `pub fn stage_bounded(&self, tx: TxId, op: IndexOp<T>) -> Result<()> where T: StagedOpSize,` | Stages an operation for the given transaction, enforcing `max_ops_per_tx` and byte limits. | `src/tx_buffer.rs` |
| `pub fn stage_kv(&self, tx: TxId, op: IndexOp<(Vec<u8>, Vec<u8>)>) -> Result<()>` | Stages a key-value operation and updates atomic key staging map. | `src/tx_buffer.rs` |
| `pub fn stage_kv_if_absent( &self, tx: TxId, doc_id: DocId, key: Vec<u8>, val: Vec<u8>, ) -> Result<()>` | Stages a key-value insert operation for transaction `tx` if the key is not already staged for insert within the same transaction scope (`IN… | `src/tx_buffer.rs` |
| `pub fn stage_many(&self, tx: TxId, ops: impl IntoIterator<Item = IndexOp<T>>) -> Result<()> where T: StagedOpSize,` | Stages multiple operations for a transaction in a single shard lock acquisition. | `src/tx_buffer.rs` |
| `pub fn staged_bytes(&self) -> usize` | Returns the current total staged bytes across all active transactions. | `src/tx_buffer.rs` |
| `pub fn staged_status(&self, key: &[u8]) -> Option<bool>` | Atomisch ermittelt den Staging-Status eines Keys über alle uncommitted Transaktionen. Gibt `Some(true)` für Insert, `Some(false)` für Delet… | `src/tx_buffer.rs` |
| `pub fn try_drain(&self, tx: TxId) -> Result<Vec<IndexOp<T>>>` | Validates that `tx` has pending operations before draining and returning them. # Errors Returns `Err(ContextraError::Transaction(...))` if… | `src/tx_buffer.rs` |
| `pub fn try_drain_kv(&self, tx: TxId) -> Result<Vec<IndexOp<(Vec<u8>, Vec<u8>)>>>` | Validates that `tx` has pending operations before draining key-value operations. | `src/tx_buffer.rs` |
| `pub fn try_register_read( &self, tx: TxId, key: impl Into<Vec<u8>>, snapshot_seq: u64, max_keys: usize, ) -> Result<()>` | Registers a read key and its snapshot sequence number for transaction `tx`, enforcing a maximum key count limit `max_keys` on new key inser… | `src/tx_buffer.rs` |
| `pub fn tx_timeout(&self) -> Duration` | Returns the transaction timeout. | `src/tx_buffer.rs` |
| `pub fn validate_pending_ops(&self, tx: TxId) -> Result<()>` | Validates that the transaction has pending operations. | `src/tx_buffer.rs` |


---

## `contextra-wire`

| Attribut | Wert |
|---|---|
| **Ring** | 0 — Ring 0 — Foundation & Pure Numerics/Types |
| **Status** | Fertig |
| **Zweck** | FlatBuffers-IPC-Generat und Zero-Copy-Adapter (Unsafe-Insel). |
| **Unsafe-Code** | Ja (deklarierte Unsafe-Insel) |
| **Interne Abhängigkeiten** | — |
| **Externe Abhängigkeiten (Auswahl)** | `bytes`, `flatbuffers`, `serde`, `serde_json` |

### Architekturbeschreibung

Ring 0 Unsafe Island: Auto-generated FlatBuffers IPC bindings and zero-copy adapters for Contextra.

### Modul- und Re-Export-Oberfläche (lib.rs)

```rust
pub mod contextra_generated;
pub mod adapter;
pub mod jsonrpc;
pub use contextra_generated::contextra::ipc::*;
pub use jsonrpc::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
```

### Öffentliche Datentypen — Structs (28)

| Struct | Felder / Form | Beschreibung | Datei |
|---|---|---|---|
| `Embedding` | `{ pub _tab: flatbuffers::Table<'a>, }` | — | `src/contextra_generated.rs` |
| `EmbeddingArgs` | `{ pub data: Option<flatbuffers::WIPOffset<flatbuffers::Vector<'a, f32>>>, pub quantized_u8: Option<flatbuffers::WIPOffset<flatbuffers::Vector<'a, u8>>>, pub metr… }` | — | `src/contextra_generated.rs` |
| `EmbeddingBuilder` | `{ fbb_: &'b mut flatbuffers::FlatBufferBuilder<'a>, start_: flatbuffers::WIPOffset<flatbuffers::TableUnfinishedWIPOffset>, }` | — | `src/contextra_generated.rs` |
| `HyperEdge` | `{ pub _tab: flatbuffers::Table<'a>, }` | — | `src/contextra_generated.rs` |
| `HyperEdgeArgs` | `{ pub id: u64, pub predicate: u8, pub participants: Option<flatbuffers::WIPOffset<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<RoleBinding<'a>>>>>, pub w… }` | — | `src/contextra_generated.rs` |
| `HyperEdgeBuilder` | `{ fbb_: &'b mut flatbuffers::FlatBufferBuilder<'a>, start_: flatbuffers::WIPOffset<flatbuffers::TableUnfinishedWIPOffset>, }` | — | `src/contextra_generated.rs` |
| `HyperEdgeId` | `{ pub _tab: flatbuffers::Table<'a>, }` | — | `src/contextra_generated.rs` |
| `HyperEdgeIdArgs` | `{ pub id: u64, }` | — | `src/contextra_generated.rs` |
| `HyperEdgeIdBuilder` | `{ fbb_: &'b mut flatbuffers::FlatBufferBuilder<'a>, start_: flatbuffers::WIPOffset<flatbuffers::TableUnfinishedWIPOffset>, }` | — | `src/contextra_generated.rs` |
| `JsonRpcError` | `{ /// Numerischer Fehlercode gemäß JSON-RPC 2.0 Spezifikation. pub code: i32, /// Kurze Fehlerbeschreibung. pub message: String, /// Ggfs. zusätzliche Details/Ko… }` | JSON-RPC 2.0 Fehlerobjekt. | `src/jsonrpc.rs` |
| `JsonRpcRequest` | `{ /// Immer "2.0". pub jsonrpc: String, /// `None` bei Notifications (keine Antwort erwartet). pub id: Option<Value>, /// Name der aufzurufenden RPC-Methode. pub… }` | Eingehende JSON-RPC 2.0 Nachricht (Request oder Notification). | `src/jsonrpc.rs` |
| `JsonRpcResponse` | `{ /// Immer "2.0". pub jsonrpc: String, /// Request ID (`None` bei Parse/Batch Errors). pub id: Option<Value>, /// Erfolgs-Ergebnis der RPC-Methode (falls vorhan… }` | Ausgehende JSON-RPC 2.0 Nachricht. | `src/jsonrpc.rs` |
| `RoleBinding` | `{ pub _tab: flatbuffers::Table<'a>, }` | — | `src/contextra_generated.rs` |
| `RoleBindingArgs` | `{ pub role: u32, pub entity: u64, }` | — | `src/contextra_generated.rs` |
| `RoleBindingBuilder` | `{ fbb_: &'b mut flatbuffers::FlatBufferBuilder<'a>, start_: flatbuffers::WIPOffset<flatbuffers::TableUnfinishedWIPOffset>, }` | — | `src/contextra_generated.rs` |
| `RoleId` | `{ pub _tab: flatbuffers::Table<'a>, }` | — | `src/contextra_generated.rs` |
| `RoleIdArgs` | `{ pub id: u32, }` | — | `src/contextra_generated.rs` |
| `RoleIdBuilder` | `{ fbb_: &'b mut flatbuffers::FlatBufferBuilder<'a>, start_: flatbuffers::WIPOffset<flatbuffers::TableUnfinishedWIPOffset>, }` | — | `src/contextra_generated.rs` |
| `ScoredDocument` | `{ pub _tab: flatbuffers::Table<'a>, }` | — | `src/contextra_generated.rs` |
| `ScoredDocumentArgs` | `{ pub id: Option<flatbuffers::WIPOffset<&'a str>>, pub score: f32, pub metadata: Option<flatbuffers::WIPOffset<&'a str>>, pub embedding: Option<flatbuffers::WIPO… }` | — | `src/contextra_generated.rs` |
| `ScoredDocumentBuilder` | `{ fbb_: &'b mut flatbuffers::FlatBufferBuilder<'a>, start_: flatbuffers::WIPOffset<flatbuffers::TableUnfinishedWIPOffset>, }` | — | `src/contextra_generated.rs` |
| `SearchResponse` | `{ pub _tab: flatbuffers::Table<'a>, }` | — | `src/contextra_generated.rs` |
| `SearchResponseArgs` | `{ pub results: Option<flatbuffers::WIPOffset<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ScoredDocument<'a>>>>>, pub total_hits: u32, pub processing_tim… }` | — | `src/contextra_generated.rs` |
| `SearchResponseBuilder` | `{ fbb_: &'b mut flatbuffers::FlatBufferBuilder<'a>, start_: flatbuffers::WIPOffset<flatbuffers::TableUnfinishedWIPOffset>, }` | — | `src/contextra_generated.rs` |
| `VectorIndexUpdate` | `{ pub _tab: flatbuffers::Table<'a>, }` | — | `src/contextra_generated.rs` |
| `VectorIndexUpdateArgs` | `{ pub id: Option<flatbuffers::WIPOffset<&'a str>>, pub embedding: Option<flatbuffers::WIPOffset<Embedding<'a>>>, pub metadata: Option<flatbuffers::WIPOffset<&'a… }` | — | `src/contextra_generated.rs` |
| `VectorIndexUpdateBuilder` | `{ fbb_: &'b mut flatbuffers::FlatBufferBuilder<'a>, start_: flatbuffers::WIPOffset<flatbuffers::TableUnfinishedWIPOffset>, }` | — | `src/contextra_generated.rs` |
| `WireBuffer` | `{ data: Bytes, }` | Zero-copy byte buffer slice wrapper for FlatBuffers messages. | `src/adapter.rs` |

### Öffentliche Datentypen — Enums (8)

| Enum | Varianten | Beschreibung | Datei |
|---|---|---|---|
| `EmbeddingOffset` | `` | — | `src/contextra_generated.rs` |
| `HyperEdgeIdOffset` | `` | — | `src/contextra_generated.rs` |
| `HyperEdgeOffset` | `` | — | `src/contextra_generated.rs` |
| `RoleBindingOffset` | `` | — | `src/contextra_generated.rs` |
| `RoleIdOffset` | `` | — | `src/contextra_generated.rs` |
| `ScoredDocumentOffset` | `` | — | `src/contextra_generated.rs` |
| `SearchResponseOffset` | `` | — | `src/contextra_generated.rs` |
| `VectorIndexUpdateOffset` | `` | — | `src/contextra_generated.rs` |

### Konstanten (27)

| Konstante | Typ | Beschreibung |
|---|---|---|
| `VT_BUSINESS_VALID_FROM` | `flatbuffers::VOffsetT` | — |
| `VT_BUSINESS_VALID_TO` | `flatbuffers::VOffsetT` | — |
| `VT_CHILD_EDGE_IDS` | `flatbuffers::VOffsetT` | — |
| `VT_DATA` | `flatbuffers::VOffsetT` | — |
| `VT_EMBEDDING` | `flatbuffers::VOffsetT` | — |
| `VT_EMBEDDING` | `flatbuffers::VOffsetT` | — |
| `VT_ENTITY` | `flatbuffers::VOffsetT` | — |
| `VT_ID` | `flatbuffers::VOffsetT` | — |
| `VT_ID` | `flatbuffers::VOffsetT` | — |
| `VT_ID` | `flatbuffers::VOffsetT` | — |
| `VT_ID` | `flatbuffers::VOffsetT` | — |
| `VT_ID` | `flatbuffers::VOffsetT` | — |
| `VT_METADATA` | `flatbuffers::VOffsetT` | — |
| `VT_METADATA` | `flatbuffers::VOffsetT` | — |
| `VT_METRIC` | `flatbuffers::VOffsetT` | — |
| `VT_PARTICIPANTS` | `flatbuffers::VOffsetT` | — |
| `VT_PREDICATE` | `flatbuffers::VOffsetT` | — |
| `VT_PROCESSING_TIME_MS` | `flatbuffers::VOffsetT` | — |
| `VT_QUANTIZED_U8` | `flatbuffers::VOffsetT` | — |
| `VT_RESULTS` | `flatbuffers::VOffsetT` | — |
| `VT_ROLE` | `flatbuffers::VOffsetT` | — |
| `VT_SCORE` | `flatbuffers::VOffsetT` | — |
| `VT_SOURCE_DOC_ID` | `flatbuffers::VOffsetT` | — |
| `VT_TOTAL_HITS` | `flatbuffers::VOffsetT` | — |
| `VT_TX_VALID_FROM` | `flatbuffers::VOffsetT` | — |
| `VT_TX_VALID_TO` | `flatbuffers::VOffsetT` | — |
| `VT_WEIGHT` | `flatbuffers::VOffsetT` | — |

### Öffentliche Funktionen (102)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn as_slice(&self) -> &[u8]` | Returns a slice reference to the underlying byte payload. | `src/adapter.rs` |
| `pub fn into_inner(self) -> Bytes` | Consumes the wrapper and returns the inner `Bytes`. | `src/adapter.rs` |
| `pub fn new(data: Bytes) -> Self` | Creates a new `WireBuffer` wrapping the provided `Bytes`. | `src/adapter.rs` |
| `pub fn add_business_valid_from(&mut self, business_valid_from: i64)` | — | `src/contextra_generated.rs` |
| `pub fn add_business_valid_to(&mut self, business_valid_to: i64)` | — | `src/contextra_generated.rs` |
| `pub fn add_child_edge_ids(&mut self, child_edge_ids: flatbuffers::WIPOffset<flatbuffers::Vector<'b , u64>>)` | — | `src/contextra_generated.rs` |
| `pub fn add_data(&mut self, data: flatbuffers::WIPOffset<flatbuffers::Vector<'b , f32>>)` | — | `src/contextra_generated.rs` |
| `pub fn add_embedding(&mut self, embedding: flatbuffers::WIPOffset<Embedding<'b >>)` | — | `src/contextra_generated.rs` |
| `pub fn add_embedding(&mut self, embedding: flatbuffers::WIPOffset<Embedding<'b >>)` | — | `src/contextra_generated.rs` |
| `pub fn add_entity(&mut self, entity: u64)` | — | `src/contextra_generated.rs` |
| `pub fn add_id(&mut self, id: flatbuffers::WIPOffset<&'b str>)` | — | `src/contextra_generated.rs` |
| `pub fn add_id(&mut self, id: flatbuffers::WIPOffset<&'b str>)` | — | `src/contextra_generated.rs` |
| `pub fn add_id(&mut self, id: u32)` | — | `src/contextra_generated.rs` |
| `pub fn add_id(&mut self, id: u64)` | — | `src/contextra_generated.rs` |
| `pub fn add_id(&mut self, id: u64)` | — | `src/contextra_generated.rs` |
| `pub fn add_metadata(&mut self, metadata: flatbuffers::WIPOffset<&'b str>)` | — | `src/contextra_generated.rs` |
| `pub fn add_metadata(&mut self, metadata: flatbuffers::WIPOffset<&'b str>)` | — | `src/contextra_generated.rs` |
| `pub fn add_metric(&mut self, metric: i8)` | — | `src/contextra_generated.rs` |
| `pub fn add_participants(&mut self, participants: flatbuffers::WIPOffset<flatbuffers::Vector<'b , flatbuffers::ForwardsUOffset<RoleBinding<'b >>>>)` | — | `src/contextra_generated.rs` |
| `pub fn add_predicate(&mut self, predicate: u8)` | — | `src/contextra_generated.rs` |
| `pub fn add_processing_time_ms(&mut self, processing_time_ms: f32)` | — | `src/contextra_generated.rs` |
| `pub fn add_quantized_u8(&mut self, quantized_u8: flatbuffers::WIPOffset<flatbuffers::Vector<'b , u8>>)` | — | `src/contextra_generated.rs` |
| `pub fn add_results(&mut self, results: flatbuffers::WIPOffset<flatbuffers::Vector<'b , flatbuffers::ForwardsUOffset<ScoredDocument<'b >>>>)` | — | `src/contextra_generated.rs` |
| `pub fn add_role(&mut self, role: u32)` | — | `src/contextra_generated.rs` |
| `pub fn add_score(&mut self, score: f32)` | — | `src/contextra_generated.rs` |
| `pub fn add_source_doc_id(&mut self, source_doc_id: u64)` | — | `src/contextra_generated.rs` |
| `pub fn add_total_hits(&mut self, total_hits: u32)` | — | `src/contextra_generated.rs` |
| `pub fn add_tx_valid_from(&mut self, tx_valid_from: u64)` | — | `src/contextra_generated.rs` |
| `pub fn add_tx_valid_to(&mut self, tx_valid_to: u64)` | — | `src/contextra_generated.rs` |
| `pub fn add_weight(&mut self, weight: f32)` | — | `src/contextra_generated.rs` |
| `pub fn business_valid_from(&self) -> i64` | — | `src/contextra_generated.rs` |
| `pub fn business_valid_to(&self) -> i64` | — | `src/contextra_generated.rs` |
| `pub fn child_edge_ids(&self) -> Option<flatbuffers::Vector<'a, u64>>` | — | `src/contextra_generated.rs` |
| `pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>( _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr>, args: &'args EmbeddingArgs<'args> ) -> flatbuffers::WIPOffset<Embedding<'bldr>>` | — | `src/contextra_generated.rs` |
| `pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>( _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr>, args: &'args ScoredDocumentArgs<'args> ) -> flatbuffers::WIPOffset<ScoredDocument<'bldr>>` | — | `src/contextra_generated.rs` |
| `pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>( _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr>, args: &'args SearchResponseArgs<'args> ) -> flatbuffers::WIPOffset<SearchResponse<'bldr>>` | — | `src/contextra_generated.rs` |
| `pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>( _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr>, args: &'args VectorIndexUpdateArgs<'args> ) -> flatbuffers::WIPOffset<VectorIndexUpdate<'bldr>>` | — | `src/contextra_generated.rs` |
| `pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>( _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr>, args: &'args RoleIdArgs ) -> flatbuffers::WIPOffset<RoleId<'bldr>>` | — | `src/contextra_generated.rs` |
| `pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>( _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr>, args: &'args HyperEdgeIdArgs ) -> flatbuffers::WIPOffset<HyperEdgeId<'bldr>>` | — | `src/contextra_generated.rs` |
| `pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>( _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr>, args: &'args RoleBindingArgs ) -> flatbuffers::WIPOffset<RoleBinding<'bldr>>` | — | `src/contextra_generated.rs` |
| `pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>( _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr>, args: &'args HyperEdgeArgs<'args> ) -> flatbuffers::WIPOffset<HyperEdge<'bldr>>` | — | `src/contextra_generated.rs` |
| `pub fn data(&self) -> Option<flatbuffers::Vector<'a, f32>>` | — | `src/contextra_generated.rs` |
| `pub fn embedding(&self) -> Option<Embedding<'a>>` | — | `src/contextra_generated.rs` |
| `pub fn embedding(&self) -> Option<Embedding<'a>>` | — | `src/contextra_generated.rs` |
| `pub fn entity(&self) -> u64` | — | `src/contextra_generated.rs` |
| `pub fn finish(self) -> flatbuffers::WIPOffset<Embedding<'a>>` | — | `src/contextra_generated.rs` |
| `pub fn finish(self) -> flatbuffers::WIPOffset<ScoredDocument<'a>>` | — | `src/contextra_generated.rs` |
| `pub fn finish(self) -> flatbuffers::WIPOffset<SearchResponse<'a>>` | — | `src/contextra_generated.rs` |
| `pub fn finish(self) -> flatbuffers::WIPOffset<VectorIndexUpdate<'a>>` | — | `src/contextra_generated.rs` |
| `pub fn finish(self) -> flatbuffers::WIPOffset<RoleId<'a>>` | — | `src/contextra_generated.rs` |
| `pub fn finish(self) -> flatbuffers::WIPOffset<HyperEdgeId<'a>>` | — | `src/contextra_generated.rs` |
| `pub fn finish(self) -> flatbuffers::WIPOffset<RoleBinding<'a>>` | — | `src/contextra_generated.rs` |
| `pub fn finish(self) -> flatbuffers::WIPOffset<HyperEdge<'a>>` | — | `src/contextra_generated.rs` |
| `pub fn finish_search_response_buffer<'a, 'b>( fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>, root: flatbuffers::WIPOffset<SearchResponse<'a>>)` | — | `src/contextra_generated.rs` |
| `pub fn finish_size_prefixed_search_response_buffer<'a, 'b>(fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>, root: flatbuffers::WIPOffset<SearchResponse<'a>>)` | — | `src/contextra_generated.rs` |
| `pub fn id(&self) -> Option<&'a str>` | — | `src/contextra_generated.rs` |
| `pub fn id(&self) -> Option<&'a str>` | — | `src/contextra_generated.rs` |
| `pub fn id(&self) -> u32` | — | `src/contextra_generated.rs` |
| `pub fn id(&self) -> u64` | — | `src/contextra_generated.rs` |
| `pub fn id(&self) -> u64` | — | `src/contextra_generated.rs` |
| `pub unsafe fn init_from_table(table: flatbuffers::Table<'a>) -> Self` | — | `src/contextra_generated.rs` |
| `pub unsafe fn init_from_table(table: flatbuffers::Table<'a>) -> Self` | — | `src/contextra_generated.rs` |
| `pub unsafe fn init_from_table(table: flatbuffers::Table<'a>) -> Self` | — | `src/contextra_generated.rs` |
| `pub unsafe fn init_from_table(table: flatbuffers::Table<'a>) -> Self` | — | `src/contextra_generated.rs` |
| `pub unsafe fn init_from_table(table: flatbuffers::Table<'a>) -> Self` | — | `src/contextra_generated.rs` |
| `pub unsafe fn init_from_table(table: flatbuffers::Table<'a>) -> Self` | — | `src/contextra_generated.rs` |
| `pub unsafe fn init_from_table(table: flatbuffers::Table<'a>) -> Self` | — | `src/contextra_generated.rs` |
| `pub unsafe fn init_from_table(table: flatbuffers::Table<'a>) -> Self` | — | `src/contextra_generated.rs` |
| `pub fn metadata(&self) -> Option<&'a str>` | — | `src/contextra_generated.rs` |
| `pub fn metadata(&self) -> Option<&'a str>` | — | `src/contextra_generated.rs` |
| `pub fn metric(&self) -> i8` | — | `src/contextra_generated.rs` |
| `pub fn new(_fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>) -> EmbeddingBuilder<'a, 'b>` | — | `src/contextra_generated.rs` |
| `pub fn new(_fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>) -> ScoredDocumentBuilder<'a, 'b>` | — | `src/contextra_generated.rs` |
| `pub fn new(_fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>) -> SearchResponseBuilder<'a, 'b>` | — | `src/contextra_generated.rs` |
| `pub fn new(_fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>) -> VectorIndexUpdateBuilder<'a, 'b>` | — | `src/contextra_generated.rs` |
| `pub fn new(_fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>) -> RoleIdBuilder<'a, 'b>` | — | `src/contextra_generated.rs` |
| `pub fn new(_fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>) -> HyperEdgeIdBuilder<'a, 'b>` | — | `src/contextra_generated.rs` |
| `pub fn new(_fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>) -> RoleBindingBuilder<'a, 'b>` | — | `src/contextra_generated.rs` |
| `pub fn new(_fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>) -> HyperEdgeBuilder<'a, 'b>` | — | `src/contextra_generated.rs` |
| `pub fn participants(&self) -> Option<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<RoleBinding<'a>>>>` | — | `src/contextra_generated.rs` |
| `pub fn predicate(&self) -> u8` | — | `src/contextra_generated.rs` |
| `pub fn processing_time_ms(&self) -> f32` | — | `src/contextra_generated.rs` |
| `pub fn quantized_u8(&self) -> Option<flatbuffers::Vector<'a, u8>>` | — | `src/contextra_generated.rs` |
| `pub fn results(&self) -> Option<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ScoredDocument<'a>>>>` | — | `src/contextra_generated.rs` |
| `pub fn role(&self) -> u32` | — | `src/contextra_generated.rs` |
| `pub fn root_as_search_response(buf: &[u8]) -> Result<SearchResponse, flatbuffers::InvalidFlatbuffer>` | Verifies that a buffer of bytes contains a `SearchResponse` and returns it. Note that verification is still experimental and may not catch… | `src/contextra_generated.rs` |
| `pub unsafe fn root_as_search_response_unchecked(buf: &[u8]) -> SearchResponse` | Assumes, without verification, that a buffer of bytes contains a SearchResponse and returns it. # Safety Callers must trust the given bytes… | `src/contextra_generated.rs` |
| `pub fn root_as_search_response_with_opts<'b, 'o>( opts: &'o flatbuffers::VerifierOptions, buf: &'b [u8], ) -> Result<SearchResponse<'b>, flatbuffers::InvalidFlatbuffer>` | Verifies, with the given options, that a buffer of bytes contains a `SearchResponse` and returns it. Note that verification is still experi… | `src/contextra_generated.rs` |
| `pub fn score(&self) -> f32` | — | `src/contextra_generated.rs` |
| `pub fn size_prefixed_root_as_search_response(buf: &[u8]) -> Result<SearchResponse, flatbuffers::InvalidFlatbuffer>` | Verifies that a buffer of bytes contains a size prefixed `SearchResponse` and returns it. Note that verification is still experimental and… | `src/contextra_generated.rs` |
| `pub unsafe fn size_prefixed_root_as_search_response_unchecked(buf: &[u8]) -> SearchResponse` | Assumes, without verification, that a buffer of bytes contains a size prefixed SearchResponse and returns it. # Safety Callers must trust t… | `src/contextra_generated.rs` |
| `pub fn size_prefixed_root_as_search_response_with_opts<'b, 'o>( opts: &'o flatbuffers::VerifierOptions, buf: &'b [u8], ) -> Result<SearchResponse<'b>, flatbuffers::InvalidFlatbuffer>` | Verifies, with the given verifier options, that a buffer of bytes contains a size prefixed `SearchResponse` and returns it. Note that verif… | `src/contextra_generated.rs` |
| `pub fn source_doc_id(&self) -> u64` | — | `src/contextra_generated.rs` |
| `pub fn total_hits(&self) -> u32` | — | `src/contextra_generated.rs` |
| `pub fn tx_valid_from(&self) -> u64` | — | `src/contextra_generated.rs` |
| `pub fn tx_valid_to(&self) -> u64` | — | `src/contextra_generated.rs` |
| `pub fn weight(&self) -> f32` | — | `src/contextra_generated.rs` |
| `pub fn err(id: Option<Value>, code: i32, message: impl Into<String>) -> Self` | Fehlerantwort gemäß JSON-RPC 2.0. | `src/jsonrpc.rs` |
| `pub fn ok(id: Option<Value>, result: Value) -> Self` | Erfolgreiche Antwort. | `src/jsonrpc.rs` |
| `pub fn finish_buffer<'bldr>( fbb: &mut flatbuffers::FlatBufferBuilder<'bldr>, root: flatbuffers::WIPOffset<SearchResponse<'bldr>>, )` | Finishes building the standard FlatBuffer with `SearchResponse` as the root table. | `src/lib.rs` |
| `pub fn finish_size_prefixed_buffer<'bldr>( fbb: &mut flatbuffers::FlatBufferBuilder<'bldr>, root: flatbuffers::WIPOffset<SearchResponse<'bldr>>, )` | Finishes building the size-prefixed FlatBuffer with `SearchResponse` as the root table. | `src/lib.rs` |
| `pub fn size_prefixed_root_with_opts<'buf>( opts: &flatbuffers::VerifierOptions, buf: &'buf [u8], ) -> Result<SearchResponse<'buf>, flatbuffers::InvalidFlatbuffer>` | Verifies and parses a size-prefixed buffer with options into a `SearchResponse`. | `src/lib.rs` |


---

## `contextra-sys`

| Attribut | Wert |
|---|---|
| **Ring** | 0 — Ring 0 — Foundation & Pure Numerics/Types |
| **Status** | Fertig |
| **Zweck** | System-Abstraktionen für mmap, mlock und Win32-ACLs (Unsafe-Insel). |
| **Unsafe-Code** | Ja (deklarierte Unsafe-Insel) |
| **Interne Abhängigkeiten** | — |
| **Externe Abhängigkeiten (Auswahl)** | `memmap2`, `tracing` |

### Modul- und Re-Export-Oberfläche (lib.rs)

```rust
pub mod acl_win32;
pub mod mlock;
pub mod mmap;
pub mod posix;
pub mod vault;
pub use acl_win32::{set_restrictive_file_acl, verify_file_acl_owner_only};
pub use mlock::{mem_lock, mem_unlock};
pub use mmap::mmap_readonly;
pub use posix::reopen_and_dup2;
pub use vault::LockedRegions;
```

### Öffentliche Datentypen — Structs (1)

| Struct | Felder / Form | Beschreibung | Datei |
|---|---|---|---|
| `LockedRegions` | `{ regions: Vec<(usize, usize)>, }` | Encapsulates locked memory regions (addresses and lengths) for sensitive vault buffers. Automatically releases (unlocks) locked regions when [`unlock… | `src/vault.rs` |

### Öffentliche Funktionen (18)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn set_restrictive_file_acl(path: &Path) -> std::io::Result<()>` | — | `src/acl_win32.rs` |
| `pub fn set_restrictive_file_acl(path: &Path) -> std::io::Result<()>` | — | `src/acl_win32.rs` |
| `pub fn verify_file_acl_owner_only(path: &Path) -> std::io::Result<()>` | — | `src/acl_win32.rs` |
| `pub fn verify_file_acl_owner_only(path: &Path) -> std::io::Result<()>` | — | `src/acl_win32.rs` |
| `pub fn mem_lock(ptr: *const u8, len: usize) -> bool` | Locks a memory region in RAM to prevent swapping (best-effort). Returns `true` if locking succeeded, or `false` with a logged warning if it… | `src/mlock.rs` |
| `pub fn mem_lock(ptr: *const u8, len: usize) -> bool` | Locks a memory region in RAM on Windows to prevent swapping (best-effort). Returns `true` if locking succeeded, or `false` with a logged wa… | `src/mlock.rs` |
| `pub fn mem_lock(ptr: *const u8, len: usize) -> bool` | Fallback implementation for non-Unix/non-Windows platforms. Returns `false` when `len > 0` and `ptr` is non-null to truthfully indicate mem… | `src/mlock.rs` |
| `pub fn mem_unlock(addr: usize, len: usize)` | Unlocks a previously locked memory region. | `src/mlock.rs` |
| `pub fn mem_unlock(addr: usize, len: usize)` | Unlocks a previously locked memory region on Windows. | `src/mlock.rs` |
| `pub fn mem_unlock(_addr: usize, _len: usize)` | Fallback no-op memory unlock for unsupported platforms. | `src/mlock.rs` |
| `pub fn mmap_readonly(file: &File) -> io::Result<memmap2::Mmap>` | Map a file into memory read-only using `memmap2`. # Safety / Invariants The caller guarantees that `file` is an open read-only file handle.… | `src/mmap.rs` |
| `pub fn reopen_and_dup2(path: &Path, target_fd: i32, flags: i32) -> std::io::Result<()>` | Opens `path` with `flags` (e.g. O_RDONLY or O_RDWR|O_APPEND), then replaces `target_fd` with the new file descriptor via `dup2`. Returns `O… | `src/posix.rs` |
| `pub fn reopen_and_dup2(_path: &Path, _target_fd: i32, _flags: i32) -> std::io::Result<()>` | — | `src/posix.rs` |
| `pub fn is_empty(&self) -> bool` | Returns `true` if no locked regions are tracked. | `src/vault.rs` |
| `pub fn len(&self) -> usize` | Returns the number of currently tracked locked regions. | `src/vault.rs` |
| `pub fn lock_slice(&mut self, slice: &[u8], attempt_mlock: bool) -> bool` | Attempts to lock a byte slice in RAM (best-effort) if `attempt_mlock` is `true`. Returns `true` if the slice is successfully memory-locked… | `src/vault.rs` |
| `pub fn new() -> Self` | Creates a new empty tracker for locked memory regions. | `src/vault.rs` |
| `pub fn unlock_all(&mut self)` | Unlocks all tracked memory regions and clears the tracked list. | `src/vault.rs` |


---

## `contextra-simd`

| Attribut | Wert |
|---|---|
| **Ring** | 0 — Ring 0 — Foundation & Pure Numerics/Types |
| **Status** | Fertig |
| **Zweck** | AVX2/AVX-512/NEON SIMD-Distanzberechnungskerne mit Laufzeit-Dispatch (Unsafe-Insel). |
| **Unsafe-Code** | Ja (deklarierte Unsafe-Insel) |
| **Interne Abhängigkeiten** | `contextra-core` |
| **Externe Abhängigkeiten (Auswahl)** | `bytemuck`, `thiserror`, `tracing` |

### Architekturbeschreibung

Contextra SIMD — Ring 0 SIMD distance kernels and hardware runtime dispatch.

### Modul- und Re-Export-Oberfläche (lib.rs)

```rust
pub mod dispatch;
pub mod kernels;
pub use dispatch::*;
pub use kernels::neon;
pub use kernels::scalar::{ cosine_distance_f32_bytes_scalar, cosine_distance_scalar, cosine_similarity_parts_f32_u8, cosine_similarity_parts_u8_scalar, dot_product_f32_bytes_scalar, dot_product_f32_u8, dot_product_scalar, dot_product_u8_scalar, euclidean_distance_f32_bytes_scalar, euclidean_distance_scalar, euclidean_distance_sq_f32_u8, euclidean_distance_sq_u8_scalar, normalize_inplace, CosineSimilarityPartsF32U8, CosineSimilarityPartsU8, };
pub use kernels::{avx2, avx512};
```

### Öffentliche Datentypen — Structs (2)

| Struct | Felder / Form | Beschreibung | Datei |
|---|---|---|---|
| `CosineSimilarityPartsF32U8` | `{ pub dot: f32, pub norm_a_sq: f32, pub norm_b_sq: f32, }` | — | `src/kernels/scalar.rs` |
| `CosineSimilarityPartsU8` | `{ pub dot: u32, pub norm_a_sq: u32, pub norm_b_sq: u32, }` | — | `src/kernels/scalar.rs` |

### Öffentliche Funktionen (51)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn cosine_distance(a: &[f32], b: &[f32]) -> Result<f32, ContextraError>` | — | `src/dispatch.rs` |
| `pub fn cosine_distance_f32_bytes(a: &[f32], b_bytes: &[u8]) -> Result<f32, ContextraError>` | — | `src/dispatch.rs` |
| `pub fn cosine_similarity_parts_u8( a: &[u8], b: &[u8], ) -> Result<CosineSimilarityPartsU8, ContextraError>` | — | `src/dispatch.rs` |
| `pub fn dot_product_distance(a: &[f32], b: &[f32]) -> Result<f32, ContextraError>` | — | `src/dispatch.rs` |
| `pub fn dot_product_distance_f32_bytes(a: &[f32], b_bytes: &[u8]) -> Result<f32, ContextraError>` | — | `src/dispatch.rs` |
| `pub fn dot_product_u8(a: &[u8], b: &[u8]) -> Result<u32, ContextraError>` | — | `src/dispatch.rs` |
| `pub fn euclidean_distance(a: &[f32], b: &[f32]) -> Result<f32, ContextraError>` | — | `src/dispatch.rs` |
| `pub fn euclidean_distance_f32_bytes(a: &[f32], b_bytes: &[u8]) -> Result<f32, ContextraError>` | — | `src/dispatch.rs` |
| `pub fn euclidean_distance_sq_u8(a: &[u8], b: &[u8]) -> Result<u32, ContextraError>` | — | `src/dispatch.rs` |
| `pub unsafe fn cosine_distance_avx2(a: &[f32], b: &[f32]) -> f32` | — | `src/kernels/avx2.rs` |
| `pub unsafe fn cosine_distance_f32_bytes_avx2(a: &[f32], b_bytes: &[u8]) -> f32` | — | `src/kernels/avx2.rs` |
| `pub unsafe fn cosine_similarity_parts_u8_avx2(a: &[u8], b: &[u8]) -> CosineSimilarityPartsU8` | — | `src/kernels/avx2.rs` |
| `pub unsafe fn dot_product_avx2(a: &[f32], b: &[f32]) -> f32` | — | `src/kernels/avx2.rs` |
| `pub unsafe fn dot_product_f32_bytes_avx2(a: &[f32], b_bytes: &[u8]) -> f32` | — | `src/kernels/avx2.rs` |
| `pub unsafe fn dot_product_u8_avx2(a: &[u8], b: &[u8]) -> u32` | — | `src/kernels/avx2.rs` |
| `pub unsafe fn euclidean_distance_avx2(a: &[f32], b: &[f32]) -> f32` | — | `src/kernels/avx2.rs` |
| `pub unsafe fn euclidean_distance_f32_bytes_avx2(a: &[f32], b_bytes: &[u8]) -> f32` | — | `src/kernels/avx2.rs` |
| `pub unsafe fn euclidean_distance_sq_u8_avx2(a: &[u8], b: &[u8]) -> u32` | — | `src/kernels/avx2.rs` |
| `pub unsafe fn hsum256_epi32_avx2(v: __m256i) -> i32` | — | `src/kernels/avx2.rs` |
| `pub unsafe fn hsum256_ps_avx(v: __m256) -> f32` | — | `src/kernels/avx2.rs` |
| `pub unsafe fn cosine_distance_avx512(a: &[f32], b: &[f32]) -> f32` | — | `src/kernels/avx512.rs` |
| `pub unsafe fn cosine_distance_f32_bytes_avx512(a: &[f32], b_bytes: &[u8]) -> f32` | — | `src/kernels/avx512.rs` |
| `pub unsafe fn cosine_similarity_parts_u8_avx512(a: &[u8], b: &[u8]) -> CosineSimilarityPartsU8` | — | `src/kernels/avx512.rs` |
| `pub unsafe fn dot_product_avx512(a: &[f32], b: &[f32]) -> f32` | — | `src/kernels/avx512.rs` |
| `pub unsafe fn dot_product_f32_bytes_avx512(a: &[f32], b_bytes: &[u8]) -> f32` | — | `src/kernels/avx512.rs` |
| `pub unsafe fn dot_product_u8_avx512vnni(a: &[u8], b: &[u8]) -> u32` | — | `src/kernels/avx512.rs` |
| `pub unsafe fn euclidean_distance_avx512(a: &[f32], b: &[f32]) -> f32` | — | `src/kernels/avx512.rs` |
| `pub unsafe fn euclidean_distance_f32_bytes_avx512(a: &[f32], b_bytes: &[u8]) -> f32` | — | `src/kernels/avx512.rs` |
| `pub unsafe fn euclidean_distance_sq_u8_avx512(a: &[u8], b: &[u8]) -> u32` | — | `src/kernels/avx512.rs` |
| `pub unsafe fn hsum512_epi32_avx512(v: __m512i) -> i32` | — | `src/kernels/avx512.rs` |
| `pub unsafe fn hsum512_ps_avx(v: __m512) -> f32` | — | `src/kernels/avx512.rs` |
| `pub unsafe fn cosine_distance_neon(a: &[f32], b: &[f32]) -> f32` | — | `src/kernels/neon.rs` |
| `pub unsafe fn dot_product_neon(a: &[f32], b: &[f32]) -> f32` | — | `src/kernels/neon.rs` |
| `pub unsafe fn euclidean_distance_neon(a: &[f32], b: &[f32]) -> f32` | — | `src/kernels/neon.rs` |
| `pub fn cosine_distance_f32_bytes_scalar(a: &[f32], b_bytes: &[u8]) -> f32` | — | `src/kernels/scalar.rs` |
| `pub fn cosine_distance_scalar(a: &[f32], b: &[f32]) -> f32` | — | `src/kernels/scalar.rs` |
| `pub fn cosine_similarity_parts_f32_u8(a: &[f32], b: &[u8]) -> CosineSimilarityPartsF32U8` | — | `src/kernels/scalar.rs` |
| `pub fn cosine_similarity_parts_u8_scalar(a: &[u8], b: &[u8]) -> CosineSimilarityPartsU8` | — | `src/kernels/scalar.rs` |
| `pub fn dot_product_f32_bytes_scalar(a: &[f32], b_bytes: &[u8]) -> f32` | — | `src/kernels/scalar.rs` |
| `pub fn dot_product_f32_u8(a: &[f32], b: &[u8]) -> f32` | — | `src/kernels/scalar.rs` |
| `pub fn dot_product_scalar(a: &[f32], b: &[f32]) -> f32` | — | `src/kernels/scalar.rs` |
| `pub fn dot_product_u8_scalar(a: &[u8], b: &[u8]) -> u32` | — | `src/kernels/scalar.rs` |
| `pub fn euclidean_distance_f32_bytes_scalar(a: &[f32], b_bytes: &[u8]) -> f32` | — | `src/kernels/scalar.rs` |
| `pub fn euclidean_distance_scalar(a: &[f32], b: &[f32]) -> f32` | — | `src/kernels/scalar.rs` |
| `pub fn euclidean_distance_sq_f32_u8(a: &[f32], b: &[u8], alphas: &[f32], mins: &[f32]) -> f32` | — | `src/kernels/scalar.rs` |
| `pub fn euclidean_distance_sq_u8_scalar(a: &[u8], b: &[u8]) -> u32` | — | `src/kernels/scalar.rs` |
| `pub fn normalize_inplace(v: &mut [f32])` | — | `src/kernels/scalar.rs` |
| `pub fn compute_distance( a: &[f32], b: &[f32], metric: DistanceMetric, ) -> contextra_core::Result<f32>` | — | `src/lib.rs` |
| `pub fn compute_distance_f32_bytes_trusted( a: &[f32], b_bytes: &[u8], metric: DistanceMetric, ) -> contextra_core::Result<f32>` | — | `src/lib.rs` |
| `pub fn compute_distance_trusted( a: &[f32], b: &[f32], metric: DistanceMetric, ) -> contextra_core::Result<f32>` | — | `src/lib.rs` |
| `pub fn validate_vector(vec: &[f32]) -> contextra_core::Result<()>` | Validates that a vector contains no NaN or Infinite values. | `src/lib.rs` |


---

## `contextra-crypto`

| Attribut | Wert |
|---|---|
| **Ring** | 0 — Ring 0 — Foundation & Pure Numerics/Types |
| **Status** | In Migration |
| **Zweck** | AES-256-GCM-SIV, WAL-HMAC-Integritätsketten, Zeroize und Deletion-Proofs. |
| **Unsafe-Code** | Nein (`#![forbid(unsafe_code)]`) |
| **Interne Abhängigkeiten** | `contextra-ports`, `contextra-types` |
| **Externe Abhängigkeiten (Auswahl)** | `aes-gcm-siv`, `ahash`, `argon2`, `bincode`, `blake3`, `ed25519-dalek`, `hkdf`, `hmac`, `lru`, `parking_lot`, `rand`, `regex`, `serde`, `serde_json`, `sha2`, `subtle`, `thiserror`, `tracing`, `zeroize` |

### Modul- und Re-Export-Oberfläche (lib.rs)

```rust
pub mod anti_tamper;
pub mod audit_chain;
pub mod crypto;
pub mod deletion_proof;
pub mod deletion_proof_typestate;
pub mod ed25519_proof;
pub mod error;
pub mod kdf;
pub mod kv_cipher;
pub mod kv_segment;
pub mod kv_shredding;
pub mod revocation_log;
pub mod wal_completeness;
pub mod wal_crypto;
pub use audit_chain::{ compute_record_commitment, AuditChain, AuditChainEntry, AuditChainHeadSignature, DataClass, EncryptedCommitmentSalt, };
pub use crypto::KeyManager as CryptoKey;
pub use crypto::KeyManager;
pub use deletion_proof::{ DeletionLayer, DeletionProof, DeletionProofKeyPair, DeletionScope, ExcludedScope, LayerCleanupProof, };
pub use ed25519_proof::{DeletionProofError, SignatureVersion};
pub use error::{CryptoError, Result};
pub use kdf::{derive_key_argon2id, KdfHeader, KdfParams};
pub use kv_cipher::{EncryptedKvLayer, KvCipher, KvSegmentCipher, ModelFingerprint};
pub use kv_segment::{ eviction_worker::{emergency_wipe, EvictionWorker}, segment::KvSegment, store::TenantIsolatedKvStore, CURRENT_KV_KEY_DERIVATION_VERSION, };
pub use kv_shredding::{derive_subkey, KeyRegistry, SubKey, DEFAULT_SHRED_KEY_GROUP_SIZE};
pub use revocation_log::{RevocationEntry, RevocationLog, RevocationTarget};
pub use wal_completeness::verify_wal_chain_completeness;
```

### Feature-Flags

| Flag | Aktiviert |
|---|---|
| `test-utils` | `[]` |
| `kv-encryption` | `[]` |
| `cloud-egress-guard` | `[]` |

### Schnittstellen — Public Traits (3)

**`GroupRevocationListener`** — *Listener trait for group revocation events.* (`src/kv_shredding.rs`)

```rust
pub trait GroupRevocationListener : Send + Sync {
fn on_group_revoked(&self, group_id: u64);
}
```

**`KmsProvider`** — *Provides Key Management Strategy hooks.* (`src/wal_crypto.rs`)

```rust
pub trait KmsProvider {
/// Retrieves the Data Encryption Key (DEK).
    fn get_key(&self) -> Result<Vec<u8>>;
}
```

**`KvCipher`** — *Abstract cipher trait for KV block encryption and seal/open operations.* (`src/kv_cipher.rs`)

```rust
pub trait KvCipher : Send + Sync {
/// Encrypts (seals) plaintext byte payload into authenticated ciphertext bytes.
    fn seal(&self, plaintext: &[u8]) -> contextra_types::Result<Vec<u8>>;
    /// Decrypts (opens) authenticated ciphertext bytes back to plaintext byte payload.
    fn open(&self, ciphertext: &[u8]) -> contextra_types::Result<Vec<u8>>;
}
```

### Öffentliche Datentypen — Structs (35)

| Struct | Felder / Form | Beschreibung | Datei |
|---|---|---|---|
| `AuditChain` | `{ /// Öffentlicher Zugriff auf die Liste der Ketten-Einträge für Prüfungen/Tests. pub entries: Vec<AuditChainEntry>, head_hash: [u8; 32], }` | Manipulationssichere Blake3-Hash-Kette für Audit-Einträge. | `src/audit_chain.rs` |
| `AuditChainAnchor` | `{ /// Index des geankerten Eintrags. pub index: u64, /// Blake3-Hash des geankerten Eintrags. pub entry_hash: [u8; 32], }` | Anker für die Verifizierung einer Audit-Kette gegen extern gespeicherte Zustände. | `src/audit_chain.rs` |
| `AuditChainEntry` | `{ /// Fortlaufender Index in der Audit-Kette (0-basiert). pub index: u64, /// Identifikator des Datenschemas. pub schema_id: String, /// Blake3-Hash des zum Zeit… }` | Einzelner Eintrag in der Audit-Hash-Kette. | `src/audit_chain.rs` |
| `AuditChainHeadSignature` | `{ /// Index des signierten Kettenkopfs. pub chain_index: u64, /// Blake3-Hash des Kettenkopfs. pub head_hash: [u8; 32], /// Ed25519-Signatur (64 Bytes). pub sign… }` | Periodisch erzeugte Ed25519-Signatur über den aktuellen Kettenkopf. | `src/audit_chain.rs` |
| `Cleaned` | `(tuple/unit)` | Marker type indicating a storage layer has been attested as cleaned. | `src/deletion_proof_typestate.rs` |
| `DeletionProof` | `{ /// Version der Signatur-Payload-Konstruktion. /// /// Version 1 (Legacy): Signiert NUR `scope`, `deleted_keys_hash` und `deleted_after_tx`. /// WARNUNG: Versi… }` | — | `src/deletion_proof.rs` |
| `DeletionProofBuilder` | `{ scope: DeletionScope, deleted_keys: Vec<Vec<u8>>, deleted_after_tx: TxId, excluded_scopes: Vec<ExcludedScope>, wal_chain_receipt: Option<[u8; 32]>, proof_key:… }` | Type-state builder enforcing complete storage layer coverage at compile time for [`DeletionProof`]. # Example ```rust use contextra_crypto::deletion_… | `src/deletion_proof_typestate.rs` |
| `DeletionProofKeyPair` | `{ /// Privater Signierschlüssel. pub signing_key: ed25519_dalek::SigningKey, /// Öffentlicher Verifikationsschlüssel. pub verifying_key: ed25519_dalek::Verifying… }` | Schlüsselpaar für Ed25519-Löschbeweise. | `src/ed25519_proof.rs` |
| `DeletionProofKeyPair` | `{ signing_key: ed25519_dalek::SigningKey, pub verifying_key: ed25519_dalek::VerifyingKey, }` | KeyPair for Ed25519 signing and verification of DeletionProofs (version 3). | `src/deletion_proof.rs` |
| `DerivedKey` | `(tuple/unit)` | Wrapper für abgeleitetes 256-Bit Schlüsselmaterial, das bei Drop automatisch aus dem Speicher gelöscht wird. | `src/kdf.rs` |
| `EncryptedCommitmentSalt` | `{ /// Shredding Group ID zur Verwaltung des Sub-Keys. pub group_id: u64, /// Verschlüsselter Salt. pub ciphertext: Vec<u8>, /// AES-GCM-SIV Nonce (12 Bytes). pub… }` | Verschlüsselter Salt-Wert für ein Record-Commitment. | `src/audit_chain.rs` |
| `EncryptedKvLayer` | `{ /// Serialized format version. Must equal [`CURRENT_KV_FORMAT_VERSION`] on decrypt. #[zeroize(skip)] pub format_version: u8, /// AES-256-GCM-SIV ciphertext con… }` | Container for an encrypted KV-cache segment layer. Plaintext data and nonce are zeroized on drop. Public metadata (`tenant_id`, `model_fingerprint`,… | `src/kv_cipher.rs` |
| `EncryptedRecordPayload` | `{ /// Unique identifier for the record. pub record_id: u64, /// Shred group identifier to which this record belongs. pub group_id: u64, /// AES-256-GCM-SIV encry… }` | Container for an encrypted record payload under Envelope Encryption. | `src/kv_shredding.rs` |
| `EncryptedSegmentPayload` | `{ /// Encrypted KV layer container from contextra-crypto. Zeroized on drop. pub layer: EncryptedKvLayer, }` | Encrypted layer representation stored inside a KvSegment when encryption is active. | `src/kv_segment/segment.rs` |
| `EncryptedWal` | `{ key_manager: KeyManager, }` | Encrypted WAL chunk provider that handles transparent encryption/decryption of WAL payloads. # Invariants - Uses per-file sub-key derivation via `Key… | `src/wal_crypto.rs` |
| `EvictionWorker` | `{ sender: parking_lot::Mutex<mpsc::Sender<EvictionCommand>>, handle: parking_lot::Mutex<Option<std::thread::JoinHandle<()>>>, }` | — | `src/kv_segment/eviction_worker.rs` |
| `GraphRepairAttestation` | `{ /// Document ID whose neighborhood pointers were repaired. pub doc_id: DocId, /// Verified confirmation that no ghost pointers remain. pub verified_no_ghost_po… }` | Attestation confirming synchronous neighborhood graph repair for vector deletions. | `src/deletion_proof.rs` |
| `GroupKek` | `(tuple/unit)` | Group Key Encryption Key (KEK) - 256 bits, zeroized on drop. | `src/kv_shredding.rs` |
| `IntegrityVerifier` | `{ last_hmac: [u8; 32], integrity_key: Zeroizing<Vec<u8>>, last_seq_no: Option<u64>, }` | Stateful verifier for WAL HMAC-SHA256 hash chains. # Invariants - Maintains running `last_hmac` state across entries to enforce sequential hash-chain… | `src/wal_crypto.rs` |
| `KdfHeader` | `{ pub version: u8, pub kdf_id: u8, pub params: KdfParams, pub salt: Vec<u8>, }` | Versionierter KDF-Header, der alle Parameter und den Salt für die Argon2id-Ableitung speichert. | `src/kdf.rs` |
| `KdfParams` | `{ pub m_cost_kib: u32, pub t_cost: u32, pub p_cost: u32, }` | Konfiguration der Argon2id KDF-Kostenparameter. | `src/kdf.rs` |
| `KeyManager` | `{ key: VolatileEncryptionKey, cipher_cache: OnceLock<Aes256GcmSiv>, nonce_prefix: [u8; 4], nonce_counter: AtomicU64, }` | Manager for encryption keys and block encryption. | `src/crypto.rs` |
| `KeyRegistry` | `{ groups: RwLock<HashMap<u64, GroupEntry>>, revoked_groups: RwLock<HashSet<u64>>, ever_registered_groups: RwLock<HashSet<u64>>, pub revocation_log: Option<Arc<Re… }` | In-memory thread-safe registry for envelope KV crypto-shredding keys. | `src/kv_shredding.rs` |
| `KvSegment` | `{ #[zeroize(skip)] // Metadaten, keine sensiblen Tensor-Daten pub tenant_id: TenantId, #[zeroize(skip)] pub segment_id: u64, /// Versionsnummer der HKDF-Schlüsse… }` | Ein KV-Cache-Segment. P9-Pflicht: Zeroize-on-Drop, nie unverschlüsselt auf persistentem/auslagerbarem Speicher. Gemäß Gesamtspezifikation v7.0 §7.3 e… | `src/kv_segment/segment.rs` |
| `KvSegmentCipher` | `{ key_manager: KeyManager, registry: KeyRegistry, }` | High-level cipher engine for KV-cache segment encryption and decryption. Consolidates onto `KeyRegistry` envelope crypto-shredding as the single shre… | `src/kv_cipher.rs` |
| `LayerCleanupProof` | `{ layer: DeletionLayer, _private: (), }` | Beweis, dass ein bestimmter DeletionLayer physisch bereinigt wurde. Kann NUR von den jeweiligen Bereinigungsfunktionen der Storage-Layer erzeugt werd… | `src/deletion_proof.rs` |
| `Missing` | `(tuple/unit)` | Marker type indicating a storage layer has not yet been attested as cleaned. | `src/deletion_proof_typestate.rs` |
| `RecordDek` | `(tuple/unit)` | Record Data Encryption Key (DEK) - 256 bits, zeroized on drop. | `src/kv_shredding.rs` |
| `RevocationEntry` | `{ /// Monotone Sequenznummer im Log (0, 1, 2, ...). pub sequence_number: u64, /// Das widerrufene Target. pub target: RevocationTarget, /// Widerrufszeitpunkt in… }` | Ein einzelner Eintrag im manipulationssicheren Widerrufslog. | `src/revocation_log.rs` |
| `RevocationLog` | `{ file_path: Option<PathBuf>, signing_key: Option<SigningKey>, verifying_key: VerifyingKey, clock: Arc<dyn Clock>, entries: RwLock<Vec<RevocationEntry>>, revoked… }` | Thread-sicherer, persistierter, Ed25519-signierter Widerrufslog. | `src/revocation_log.rs` |
| `SubKey` | `(tuple/unit)` | Sub-key or KEK bytes (256 bits) zeroized on drop. | `src/kv_shredding.rs` |
| `TenantIsolatedKvStore` | `{ /// 32 cache-line-gepaddete RwLock-Shards. Box<[_]> für flexible Shard-Anzahl. shards: Box<[Shard]>, shard_count: usize, /// Globaler Offset für Shard-Rotation… }` | Tenant-isolierter KV-Segment-Store. INV-TENANT-Analogon für KV-Bridge: Ein Tenant kann niemals Segmente eines anderen Tenants lesen. Strukturell erzw… | `src/kv_segment/store.rs` |
| `VolatileEncryptionKey` | `{ key_bytes: Zeroizing<[u8; 32]>, // AES-256 Key }` | Defines a cryptographic key that is explicitly zeroed out when dropped or when an emergency trigger is activated, protecting against cold-boot attack… | `src/anti_tamper.rs` |
| `WalEntrySnapshot` | `{ pub tx_id: u64, pub seq_no: u64, pub op_type: u8, // 0: Put, 1: Delete, 2: TxEnd pub key: Vec<u8>, pub value: Vec<u8>, pub checksum: [u8; 32], pub prev_hmac: [… }` | Immutable snapshot of a WAL entry used for cryptographic integrity verification. # Invariants - Captures entry fields (`seq_no`, `op_type`, `key`, `v… | `src/wal_crypto.rs` |
| `WalHmac` | `{ mac: Hmac<Sha256>, }` | Stateful wrapper around HMAC-SHA256 initialized with WAL domain separation. # Invariants - Initialized with `b"contextra-wal-v1"` domain separation s… | `src/wal_crypto.rs` |

### Öffentliche Datentypen — Enums (9)

| Enum | Varianten | Beschreibung | Datei |
|---|---|---|---|
| `CryptoError` | `#[error("key derivation failed: {0}")] KeyDerivation(String), #[error("encryption failed: {0}")] Encryption(String), #[error("decryption failed: {0}")] Decryption(String), #[error("invalid nonce or k…` | Standalone error type for all operations within `contextra-crypto`. | `src/error.rs` |
| `DataClass` | `/// Öffentliche Daten. Public, /// Interne Betriebsdaten. Internal, /// Vertrauliche Geschäftsdaten. Confidential, /// Streng vertrauliche / reglementierte Daten. Restricted, /// Personenbezogene Dat…` | Datenklassifikation für Audit-Einträge (keine Freitext-Strings). | `src/audit_chain.rs` |
| `DeletionLayer` | `/// In-memory memtable records cleared. LsmMemtable, /// Persistent SSTable files purged across all LSM levels. SsTableAllLevels, /// HNSW graph nodes and tombstone references purged. HnswIndex, ///…` | Layer-explizite Coverage-Deklaration. Jeder Layer MUSS physisch bereinigt sein bevor er hier deklariert wird. | `src/deletion_proof.rs` |
| `DeletionProofError` | `/// Ungültiger Ed25519 VerifyingKey (32 Bytes). #[error("invalid verifying key: {0}")] InvalidVerifyingKey(String), /// Ungültige Ed25519 Signatur (64 Bytes) oder Fehlschlag bei der Verifikation. #[e…` | Fehlerzustände bei der Verifikation von Ed25519-Löschbeweisen. | `src/ed25519_proof.rs` |
| `DeletionScope` | `/// Individual document deletion. Document { /// DocId of target document. doc_id: DocId, /// TenantId of owning tenant. tenant_id: TenantId, }, /// Collection-wide deletion. Collection { /// Collect…` | Target scope for physical deletion. | `src/deletion_proof.rs` |
| `ExcludedScope` | `/// Wissen in Zusammenfassungen die als LLM-Fine-Tuning-Input dienten. ConsolidatedAndDistilled, /// LLM-Modellparameter (arXiv:2505.16831 — Unlearning Isn't Deletion). LlmParameterMemory,` | Explizite Nicht-Abdeckung — maschinenlesbar für Audit-Systeme. | `src/deletion_proof.rs` |
| `RevocationTarget` | `/// Shredding Group ID (z. B. für `KeyRegistry`) Group(u64), /// Key Encryption Key (KEK) Identifier Kek(String), /// Data Encryption Key (DEK) Identifier Dek(String), /// Specific Record Identifier…` | Das Ziel eines Widerruf-Eintrags im Log (Group ID, KEK ID, DEK ID oder Record ID). | `src/revocation_log.rs` |
| `SignatureVersion` | `/// HMAC-SHA256 legacy scope-only signature (version 1) V1 = 1, /// HMAC-SHA256 scope + layer coverage signature (version 2) V2 = 2, /// Ed25519 asymmetric signature with length-prefixed key hashes (…` | `SignatureVersion`-Enum für typsichere Versionierung von DeletionProof-Signaturen. Referenz: Befund v17 Teil 10.1 & INVARIANTE INV-DELETION-1 (die ph… | `src/ed25519_proof.rs` |
| `VerificationKey` | `/// Key for HMAC-SHA256 signature verification (version 1 and 2). Hmac(&'a [u8]), /// Key for Ed25519 signature verification (version 3). Ed25519(&'a ed25519_dalek::VerifyingKey),` | Key parameter for verification (either HMAC-SHA256 byte slice or Ed25519 VerifyingKey). | `src/deletion_proof.rs` |

### Typ-Aliase (2)

| Alias | Definition | Beschreibung |
|---|---|---|
| `Result` | `type Result<T> = std::result::Result<T, CryptoError>` | Result type alias for cryptographic operations in `contextra-crypto`. |
| `SpillHandler` | `type SpillHandler= Arc<dyn Fn(TenantId, u64, Vec<u8>) + Send + Sync>` | Optionaler Callback-Hook für Tier-2-LSM-Spill bei Eviction aus dem In-Memory LRU Cache. |

### Konstanten (15)

| Konstante | Typ | Beschreibung |
|---|---|---|
| `CURRENT_KV_FORMAT_VERSION` | `u8` | Current serialized format version for [`EncryptedKvLayer`]. Increment when the serialized layout changes in a breaking way. |
| `CURRENT_KV_KEY_DERIVATION_VERSION` | `u8` | Aktuelle Version der KV-Segment-Schlüsselableitung. Erhöhe diesen Wert, wenn sich der HKDF-Info-String oder der Salt-Aufbau ändert. |
| `DEFAULT_M_COST_KIB` | `u32` | OWASP Standardempfehlungen für Argon2id |
| `DEFAULT_P_COST` | `u32` | — |
| `DEFAULT_SEGMENT_CAPACITY_PER_TENANT` | `usize` | — |
| `DEFAULT_SHARD_COUNT` | `usize` | — |
| `DEFAULT_SHRED_KEY_GROUP_SIZE` | `usize` | Default size of a shred key group (number of records sharing one KEK). |
| `DEFAULT_T_COST` | `u32` | — |
| `KDF_HEADER_MAGIC` | `&[u8` | Magischer Header-Marker "MFKD" (Contextra Key Derivation) |
| `KDF_HEADER_VERSION_1` | `u8` | Aktuelle KDF Header Version |
| `KDF_ID_ARGON2ID` | `u8` | Identifikator für Argon2id |
| `MIN_M_COST_KIB` | `u32` | OWASP Mindestanforderungen für Argon2id |
| `MIN_P_COST` | `u32` | — |
| `MIN_SALT_LEN` | `usize` | Min. empfohlener Salt-Umfang (16 Bytes) |
| `MIN_T_COST` | `u32` | — |

### Öffentliche Funktionen (169)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn as_bytes(&self) -> &[u8]` | Accessor for cryptographic operations. | `src/anti_tamper.rs` |
| `pub fn emergency_wipe(&mut self)` | Emergency Trigger: Explicitly wipes the key from memory. This is to be hooked into hardware sensors (e.g. crash/voltage). | `src/anti_tamper.rs` |
| `pub fn inspect_key_bytes_for_test(&self) -> &[u8` | Test-only method to inspect key bytes. Used to verify zeroing in integration tests. | `src/anti_tamper.rs` |
| `pub fn new(raw: [u8; 32]) -> Self` | Creates a new volatile key from a raw 32-byte array. | `src/anti_tamper.rs` |
| `pub fn anchor(&self) -> Option<AuditChainAnchor>` | Liefert den Anker des letzten Eintrags der Kette, oder `None` bei leerer Kette. | `src/audit_chain.rs` |
| `pub fn append( &mut self, schema_id: impl Into<String>, rules_hash: [u8; 32], data_class: DataClass, doc_id: DocId, tx_id: TxId, commitment: Option<[u8; 32]>, ) -> Result<&AuditChainEntry>` | Fügt einen neuen Eintrag zur Audit-Kette hinzu und aktualisiert den Kettenkopf-Hash. | `src/audit_chain.rs` |
| `pub fn compute_hash( index: u64, schema_id: &str, rules_hash: &[u8; 32], data_class: DataClass, doc_id: DocId, tx_id: TxId, prev_hash: &[u8; 32], commitment: Option<&[u8; 32]>, ) -> Result<[u8` | Berechnet den Blake3-Hash für die Felder eines Audit-Ketten-Eintrags. | `src/audit_chain.rs` |
| `pub fn compute_record_commitment(salt: &[u8], attr: &[u8]) -> [u8` | Berechnet das Commitment `Blake3(salt || attr)`. | `src/audit_chain.rs` |
| `pub fn create_signed_anchor( &self, signing_key: &ed25519_dalek::SigningKey, ) -> Result<(AuditChainAnchor, AuditChainHeadSignature)>` | Erzeugt einen Anker des aktuellen Kettenkopfs zusammen mit einer Ed25519-Kettenkopf-Signatur. | `src/audit_chain.rs` |
| `pub fn decrypt(&self, registry: &KeyRegistry) -> Result<Vec<u8>>` | Entschlüsselt das Salt. Schlägt fehl, wenn der Sub-Key für `group_id` geschreddert/widerrufen wurde. | `src/audit_chain.rs` |
| `pub fn encrypt( registry: &KeyRegistry, master_key: &KeyManager, group_id: u64, salt: &[u8], ) -> Result<Self>` | Verschlüsselt ein Salt-Byte-Array über den `KeyRegistry`-Shredding-Mechanismus. | `src/audit_chain.rs` |
| `pub fn entries(&self) -> &[AuditChainEntry]` | Gibt eine Referenz auf alle Einträge der Kette zurück. | `src/audit_chain.rs` |
| `pub fn head_hash(&self) -> [u8` | Liefert den aktuellen Kettenkopf-Hash. | `src/audit_chain.rs` |
| `pub fn is_empty(&self) -> bool` | Prüft, ob die Kette leer ist. | `src/audit_chain.rs` |
| `pub fn len(&self) -> usize` | Liefert die Anzahl der Einträge in der Kette. | `src/audit_chain.rs` |
| `pub fn new() -> Self` | Erzeugt eine neue, leere Audit-Kette. | `src/audit_chain.rs` |
| `pub fn sign_head( &self, signing_key: &ed25519_dalek::SigningKey, ) -> Result<AuditChainHeadSignature>` | Periodischer Aufruf: Signiert den aktuellen Kettenkopf mit Ed25519. | `src/audit_chain.rs` |
| `pub fn verify_chain(&self) -> Result<bool>` | Verifiziert die vollständige Kette auf Integrität. Prüft für jeden Eintrag, ob `prev_hash` dem `entry_hash` des Vorgängers entspricht und o… | `src/audit_chain.rs` |
| `pub fn verify_chain_against_anchor(&self, anchor: &AuditChainAnchor) -> Result<bool>` | Verifiziert die Kette gegen einen externen Anker. Ruft zuerst [`Self::verify_chain`] zur Prüfung der internen Integrität auf. Anschließend… | `src/audit_chain.rs` |
| `pub fn verify_head_signature( head_sig: &AuditChainHeadSignature, verifying_key: &ed25519_dalek::VerifyingKey, ) -> Result<bool>` | Verifiziert eine periodische Kettenkopf-Signatur gegen den angegebenen VerifyingKey. | `src/audit_chain.rs` |
| `pub fn verify_signed_anchor( &self, anchor: &AuditChainAnchor, head_sig: &AuditChainHeadSignature, verifying_key: &ed25519_dalek::VerifyingKey, ) -> Result<bool>` | Verifiziert die Kette gegen einen Anker und prüft zusätzlich die Kettenkopf-Signatur. | `src/audit_chain.rs` |
| `pub fn cipher_for(&self, tenant_id: contextra_types::TenantId) -> Result<Self>` | Derives a tenant-isolated `KeyManager` instance for a specific `TenantId`. | `src/crypto.rs` |
| `pub fn cipher_for_scoped<T>( &self, scoped_key: contextra_types::TenantScoped<T>, expected_tenant_id: &contextra_types::TenantId, ) -> Result<Self>` | Derives a tenant-isolated `KeyManager` instance enforcing compile-time `TenantScoped` verification. Unpacks `scoped_key` only if the bound… | `src/crypto.rs` |
| `pub fn create_deletion_proof( &self, scope: crate::deletion_proof::DeletionScope, deleted_keys: Vec<Vec<u8>>, deleted_after_tx: contextra_types::TxId, covered_layers: Vec<crate::deletion_proof::LayerCleanupProof>, excluded_scopes: Vec<crate::deletion_proof::E…` | Creates and signs a v2 [`DeletionProof`][crate::deletion_proof::DeletionProof] using a sub-key derived specifically for cryptographic delet… | `src/crypto.rs` |
| `pub fn decrypt_auto_nonce(&self, ciphertext: &[u8], nonce_bytes: &[u8; 12]) -> Result<Vec<u8>>` | Decrypts a block of data using a full 12-byte nonce. | `src/crypto.rs` |
| `pub fn derive_deletion_proof_key(&self) -> Result<[u8` | Derives a deletion proof signing key for cryptographic deletion verification. | `src/crypto.rs` |
| `pub fn derive_file_key(&self, file_id: &[u8]) -> Result<Self>` | Derives a sub-key for a specific file or logical stream. This prevents nonce-reuse when multiple files use the same master key. | `src/crypto.rs` |
| `pub fn derive_kv_key( &self, tenant_id: contextra_types::TenantId, model_fingerprint: &crate::kv_cipher::ModelFingerprint, ) -> Result<Self>` | Derives a sub-key for KV-cache segment encryption, bound to `(tenant_id, model_fingerprint)`. # Format Version Uses HKDF info string `"cont… | `src/crypto.rs` |
| `pub fn derive_kv_key_scoped( &self, scoped_fingerprint: contextra_types::TenantScoped<&crate::kv_cipher::ModelFingerprint>, expected_tenant_id: &contextra_types::TenantId, ) -> Result<Self>` | Derives a sub-key for KV-cache segment encryption with strict compile-time tenant scope enforcement. Unpacks `scoped_fingerprint` only if t… | `src/crypto.rs` |
| `pub fn derive_segment_key(&self, info: &str) -> Result<Self>` | Derives a sub-key specifically for a KV-cache segment based on a versioned info string. | `src/crypto.rs` |
| `pub fn derive_segment_key_bytes(&self, info_bytes: &[u8]) -> Result<Self>` | Derives a sub-key specifically for a KV-cache segment based on raw info bytes. | `src/crypto.rs` |
| `pub fn derive_segment_key_for_id( &self, tenant_id: contextra_types::TenantId, segment_id: u64, ) -> Result<Self>` | Derives a sub-key specifically for a KV-cache segment identified by tenant ID and segment ID. Formats the versioned segment descriptor stri… | `src/crypto.rs` |
| `pub fn emergency_wipe(&mut self)` | Emergency Trigger: Explicitly wipes the key from memory and clears cipher cache. | `src/crypto.rs` |
| `pub fn encrypt_auto_nonce(&self, data: &[u8]) -> Result<(Vec<u8>, [u8` | Encrypts a block of data with a deterministically generated 12-byte nonce. Returns the ciphertext and the full 12-byte nonce used. | `src/crypto.rs` |
| `pub fn inspect_key_bytes_for_test(&self) -> &[u8` | Provides access to the key bytes ONLY during testing. | `src/crypto.rs` |
| `pub fn integrity_key(&self) -> Result<[u8` | Derives an integrity key for HMAC-SHA256. | `src/crypto.rs` |
| `pub fn try_new(passphrase: &str, salt: &[u8]) -> Result<Self>` | Creates a new KeyManager by deriving a key from a passphrase using HKDF-SHA256. # WARNHINWEIS Diese Funktion nutzt HKDF-SHA256 und setzt vo… | `src/crypto.rs` |
| `pub fn try_new_argon2id(passphrase: &str) -> Result<(Self, KdfHeader)>` | Creates a new [`KeyManager`] using Argon2id password-based key derivation with default OWASP parameters. Generates a default [`KdfHeader`]… | `src/crypto.rs` |
| `pub fn try_new_random_salt(passphrase: &str) -> Result<(Self, [u8` | Creates a new KeyManager with a cryptographically secure random salt. | `src/crypto.rs` |
| `pub fn try_new_with_kdf(passphrase: &str, header: &KdfHeader) -> Result<Self>` | Creates a new KeyManager using Argon2id password-based key derivation. Nutzt Argon2id mit den im [`KdfHeader`] definierten Kostenparametern… | `src/crypto.rs` |
| `pub fn compute_wal_delete_receipt( prev_hmac: &[u8; 32], delete_event_payload: &[u8], integrity_key: &[u8], ) -> Result<[u8` | Erzeugt eine WAL-Löschquittung H(hmac_prev || delete_event) für den Nachweis auf WAL-Ebene gemäß DSGVO Art. 17. | `src/deletion_proof.rs` |
| `pub fn create( scope: DeletionScope, deleted_keys: Vec<Vec<u8>>, deleted_after_tx: TxId, covered_layers: Vec<LayerCleanupProof>, excluded_scopes: Vec<ExcludedScope>, proof_key: &[u8], ) -> Result<Self>` | Erstellt und signiert einen DeletionProof nach Layer-Bereinigung. AUFRUFREIHENFOLGE (INV-DELETION-1): 1. Alle covered_layers physisch berei… | `src/deletion_proof.rs` |
| `pub fn create_full_v3( scope: DeletionScope, mut deleted_keys: Vec<Vec<u8>>, deleted_after_tx: TxId, covered_layers: Vec<LayerCleanupProof>, excluded_scopes: Vec<ExcludedScope>, wal_chain_receipt: Option<[u8; 32]>, audit_chain_position: Option<u64>, attested_…` | Erstellt und signiert einen DeletionProof (Version 3, Ed25519) mit allen optionalen Erweiterungen. | `src/deletion_proof.rs` |
| `pub fn create_v3( scope: DeletionScope, deleted_keys: Vec<Vec<u8>>, deleted_after_tx: TxId, covered_layers: Vec<LayerCleanupProof>, excluded_scopes: Vec<ExcludedScope>, attested_at: i64, graph_repair: &[GraphRepairAttestation], signing_key: &ed25519_dalek::Si…` | Erstellt und signiert einen DeletionProof (Version 3) mit Ed25519. | `src/deletion_proof.rs` |
| `pub fn create_v3_with_audit_position( scope: DeletionScope, deleted_keys: Vec<Vec<u8>>, deleted_after_tx: TxId, covered_layers: Vec<LayerCleanupProof>, excluded_scopes: Vec<ExcludedScope>, audit_chain_position: Option<u64>, attested_at: i64, graph_repair: &[G…` | Erstellt und signiert einen DeletionProof (Version 3, Ed25519) inklusive Verweis auf den Audit-Chain-Eintrag. | `src/deletion_proof.rs` |
| `pub fn create_with_wal_receipt( scope: DeletionScope, mut deleted_keys: Vec<Vec<u8>>, deleted_after_tx: TxId, covered_layers: Vec<LayerCleanupProof>, excluded_scopes: Vec<ExcludedScope>, wal_chain_receipt: Option<[u8; 32]>, proof_key: &[u8], ) -> Result<Self>` | Erstellt und signiert einen DeletionProof (Version 2) inklusive optionaler WAL-HMAC-Kettenquittung. | `src/deletion_proof.rs` |
| `pub fn create_with_wal_receipt_v3( scope: DeletionScope, deleted_keys: Vec<Vec<u8>>, deleted_after_tx: TxId, covered_layers: Vec<LayerCleanupProof>, excluded_scopes: Vec<ExcludedScope>, wal_chain_receipt: Option<[u8; 32]>, attested_at: i64, graph_repair: &[Gr…` | Erstellt und signiert einen DeletionProof (Version 3, Ed25519) inklusive optionaler WAL-HMAC-Kettenquittung. | `src/deletion_proof.rs` |
| `pub fn export_for_audit(&self) -> Result<String>` | Exportiert Proof als JSON für Compliance-Dokumentation. ExcludedScope-Liste ist maschinenlesbar enthalten. | `src/deletion_proof.rs` |
| `pub fn generate() -> Self` | Generates a new random Ed25519 keypair using OsRng. | `src/deletion_proof.rs` |
| `pub fn hash_deleted_keys_length_prefixed(deleted_keys: &[Vec<u8>]) -> [u8` | Berechnet den Blake3-Hash einer deterministisch sortierten Liste gelöschter Schlüssel mit Längenpräfix. DOKUMENTATION ZUM KOLLISIONSRISIKO:… | `src/deletion_proof.rs` |
| `pub fn layer(&self) -> &DeletionLayer` | Gibt den zugrundeliegenden DeletionLayer zurück. | `src/deletion_proof.rs` |
| `pub fn new_after_verified_empty( layer: DeletionLayer, remaining_live_entries: usize, ) -> Result<Self>` | Erzeugt einen Proof NUR, wenn `remaining_live_entries == 0` — also nur dann, wenn der Aufrufer nachweislich (durch einen Re-Scan des betrof… | `src/deletion_proof.rs` |
| `pub fn signature_version_typed( &self, ) -> std::result::Result<SignatureVersion, DeletionProofError>` | Parst und liefert die typsichere Signaturversion des Beweises. # Errors Gibt [`DeletionProofError::UnsupportedVersion`] zurück, wenn `signa… | `src/deletion_proof.rs` |
| `pub fn signing_key(&self) -> &ed25519_dalek::SigningKey` | Returns a reference to the signing key. | `src/deletion_proof.rs` |
| `pub fn tenant_id(&self) -> TenantId` | Gibt die Tenant-ID aus dem Scope zurück. | `src/deletion_proof.rs` |
| `pub fn verify<'a>(&self, key: impl Into<VerificationKey<'a>>) -> Result<bool>` | Verifiziert Signatur. Referenz: Befund v17 Teil 10.1 & INVARIANTE INV-DELETION-1. NICHT-GARANTIE: Prüft nur Signatur, nicht ob Storage tats… | `src/deletion_proof.rs` |
| `pub fn verify_and_create<F>(layer: DeletionLayer, verification: F) -> Result<Self> where F: FnOnce() -> Result<bool>,` | Erzeugt einen Proof für `layer` NUR, wenn `verification` bestätigt, dass die physische Bereinigung tatsächlich abgeschlossen ist. `verifica… | `src/deletion_proof.rs` |
| `pub fn verify_external( &self, public_key: &ed25519_dalek::VerifyingKey, ) -> std::result::Result<(), CryptoError>` | Verifies this proof using ONLY the provided Ed25519 public key. Does NOT require access to any `KeyManager` state — this is the verificatio… | `src/deletion_proof.rs` |
| `pub fn verify_wal_delete_receipt( receipt: &[u8; 32], prev_hmac: &[u8; 32], delete_event_payload: &[u8], integrity_key: &[u8], ) -> Result<bool>` | Verifiziert eine WAL-Löschquittung in O(1) konstanter Zeit ohne Klartextzugang. | `src/deletion_proof.rs` |
| `pub fn verify_wal_receipt( &self, prev_hmac: &[u8; 32], delete_event_payload: &[u8], integrity_key: &[u8], ) -> Result<bool>` | Verifiziert die in diesem Beweis enthaltene WAL-Löschquittung (falls vorhanden) gegen das WAL-Event. | `src/deletion_proof.rs` |
| `pub fn verifying_key_bytes(&self) -> [u8` | Returns the 32-byte representation of the verifying public key. | `src/deletion_proof.rs` |
| `pub fn finish(self) -> Result<DeletionProof>` | Finalizes the builder and creates a fully attested [`DeletionProof`]. This method is only available when all 7 storage layers have been mar… | `src/deletion_proof_typestate.rs` |
| `pub fn new( scope: DeletionScope, deleted_keys: Vec<Vec<u8>>, deleted_after_tx: TxId, excluded_scopes: Vec<ExcludedScope>, proof_key: Vec<u8>, ) -> Self` | Constructs a new type-state builder initialized with all storage layers as [`Missing`]. | `src/deletion_proof_typestate.rs` |
| `pub fn with_csr_graph_cleanup( mut self, proof: LayerCleanupProof, ) -> DeletionProofBuilder<Lsm, Sst, Hnsw, Wal, Cleaned, Kv, Emb>` | Attests physical cleanup of the CSR knowledge graph. | `src/deletion_proof_typestate.rs` |
| `pub fn with_embedding_cache_cleanup( mut self, proof: LayerCleanupProof, ) -> DeletionProofBuilder<Lsm, Sst, Hnsw, Wal, Csr, Kv, Cleaned>` | Attests physical cleanup of the embedding cache. | `src/deletion_proof_typestate.rs` |
| `pub fn with_hnsw_index_cleanup( mut self, proof: LayerCleanupProof, ) -> DeletionProofBuilder<Lsm, Sst, Cleaned, Wal, Csr, Kv, Emb>` | Attests physical cleanup of the HNSW vector index. | `src/deletion_proof_typestate.rs` |
| `pub fn with_kv_cache_segments_cleanup( mut self, proof: LayerCleanupProof, ) -> DeletionProofBuilder<Lsm, Sst, Hnsw, Wal, Csr, Cleaned, Emb>` | Attests physical cleanup of key-value cache segments. | `src/deletion_proof_typestate.rs` |
| `pub fn with_lsm_memtable_cleanup( mut self, proof: LayerCleanupProof, ) -> DeletionProofBuilder<Cleaned, Sst, Hnsw, Wal, Csr, Kv, Emb>` | Attests physical cleanup of the LSM memtable layer. | `src/deletion_proof_typestate.rs` |
| `pub fn with_sstable_all_levels_cleanup( mut self, proof: LayerCleanupProof, ) -> DeletionProofBuilder<Lsm, Cleaned, Hnsw, Wal, Csr, Kv, Emb>` | Attests physical cleanup of all SSTable levels. | `src/deletion_proof_typestate.rs` |
| `pub fn with_wal_all_segments_cleanup( mut self, proof: LayerCleanupProof, ) -> DeletionProofBuilder<Lsm, Sst, Hnsw, Cleaned, Csr, Kv, Emb>` | Attests physical cleanup of all WAL log segments. | `src/deletion_proof_typestate.rs` |
| `pub fn with_wal_chain_receipt(mut self, receipt: Option<[u8; 32]>) -> Self` | Sets an optional WAL HMAC chain receipt. | `src/deletion_proof_typestate.rs` |
| `pub fn as_u8(&self) -> u8` | Gibt den `u8`-Wert der Version zurück. | `src/ed25519_proof.rs` |
| `pub fn generate(rng: &mut impl RngCore) -> Self` | Generiert ein neues Ed25519-Schlüsselpaar unter Nutzung eines injizierten Zufallsgenerators. | `src/ed25519_proof.rs` |
| `pub fn sign_deletion_proof_v3(keypair: &DeletionProofKeyPair, proof_payload: &[u8]) -> [u8` | Signiert den kanonischen Byte-Payload eines Version-3-Löschbeweises mit Ed25519. Gibt eine 64-Byte Ed25519-Signatur zurück. | `src/ed25519_proof.rs` |
| `pub fn signing_key(&self) -> &ed25519_dalek::SigningKey` | Gibt eine Referenz auf den Signierschlüssel zurück. | `src/ed25519_proof.rs` |
| `pub fn verify_deletion_proof_v3( verifying_key_bytes: &[u8; 32], proof_payload: &[u8], signature: &[u8; 64], ) -> Result<(), DeletionProofError>` | Verifiziert eine Ed25519-Signatur eines Version-3-Löschbeweises rein funktional. Diese Funktion benötigt keine Contextra-internen Zustände… | `src/ed25519_proof.rs` |
| `pub fn verifying_key(&self) -> &ed25519_dalek::VerifyingKey` | Gibt eine Referenz auf den Verifikationsschlüssel zurück. | `src/ed25519_proof.rs` |
| `pub fn verifying_key_bytes(&self) -> [u8` | Gibt die 32-Byte-Repräsentation des öffentlichen Verifikationsschlüssels zurück. | `src/ed25519_proof.rs` |
| `pub fn wal_corruption(offset: u64, reason: impl Into<String>) -> Self` | Helper to construct a `WalCorruption` error. | `src/error.rs` |
| `pub fn derive_key_argon2id(passphrase: &str, header: &KdfHeader) -> Result<DerivedKey>` | Leitet einen 256-Bit (32 Byte) Schlüssel aus einer Passphrase und einem `KdfHeader` via Argon2id ab. | `src/kdf.rs` |
| `pub fn from_bytes(bytes: &[u8]) -> Result<Self>` | Parst einen `KdfHeader` aus Binärdaten. Garantiert Panic-Freiheit (total function) bei beliebigen Byte-Eingaben. | `src/kdf.rs` |
| `pub fn generate_default() -> Result<Self>` | Erstellt einen neuen `KdfHeader` v1 mit zufälligem 32-Byte-Salt und OWASP-Standardparametern. | `src/kdf.rs` |
| `pub fn new(m_cost_kib: u32, t_cost: u32, p_cost: u32) -> Result<Self>` | Erstellt neue `KdfParams` mit Validierung der Untergrenzen. | `src/kdf.rs` |
| `pub fn new(params: KdfParams, salt: Vec<u8>) -> Result<Self>` | Erstellt einen `KdfHeader` v1 mit vorgegebenen Parametern und Salt. | `src/kdf.rs` |
| `pub fn new_for_test(m_cost_kib: u32, t_cost: u32, p_cost: u32) -> Self` | Erstellt `KdfParams` mit kleineren Kostenparametern ausschließlich für Testzwecke. | `src/kdf.rs` |
| `pub fn to_bytes(&self) -> Vec<u8>` | Serialisiert den `KdfHeader` in Binärform. Layout: `magic (4B "MFKD") | version (1B = 1) | kdf_id (1B = 1 Argon2id) | m_cost_kib (4B BE) |… | `src/kdf.rs` |
| `pub fn decrypt(&self, encrypted: &EncryptedKvLayer) -> Result<Vec<u8>>` | Decrypts an `EncryptedKvLayer` back to its original plaintext. Checks that `encrypted.format_version == CURRENT_KV_FORMAT_VERSION`. Derives… | `src/kv_cipher.rs` |
| `pub fn decrypt_with_version( &self, encrypted: &EncryptedKvLayer, segment_id: u64, _version: u8, ) -> Result<Vec<u8>>` | Decrypts an `EncryptedKvLayer` using consolidated `KeyRegistry` envelope crypto-shredding. | `src/kv_cipher.rs` |
| `pub fn encrypt( &self, tenant_id: TenantId, model_fingerprint: ModelFingerprint, plaintext: &[u8], ) -> Result<EncryptedKvLayer>` | Encrypts KV-cache plaintext payload for a given `(tenant_id, model_fingerprint)` pair. Derives an isolated sub-key via HKDF-SHA256 through… | `src/kv_cipher.rs` |
| `pub fn encrypt_with_version( &self, tenant_id: TenantId, segment_id: u64, _version: u8, model_fingerprint: ModelFingerprint, plaintext: &[u8], ) -> Result<EncryptedKvLayer>` | Encrypts KV-cache plaintext payload using consolidated `KeyRegistry` envelope crypto-shredding. | `src/kv_cipher.rs` |
| `pub fn new(key_manager: KeyManager) -> Self` | Creates a new `KvSegmentCipher` wrapping the workspace master `KeyManager`. | `src/kv_cipher.rs` |
| `pub fn registry(&self) -> &KeyRegistry` | Returns a reference to the consolidated `KeyRegistry` for crypto-shredding key management. | `src/kv_cipher.rs` |
| `pub fn with_revocation_log(mut self, log: Arc<RevocationLog>) -> Self` | Attaches a `RevocationLog` to the underlying `KeyRegistry`. | `src/kv_cipher.rs` |
| `pub fn emergency_wipe(store: &TenantIsolatedKvStore)` | SEPARATER Pfad für Notfall-Löschung (z.B. Prozess-Shutdown, expliziter Sicherheits-Trigger). Synchron, blockierend, garantiert vor Rückkehr… | `src/kv_segment/eviction_worker.rs` |
| `pub fn shutdown(&self)` | Beendet den Worker-Thread geordnet. | `src/kv_segment/eviction_worker.rs` |
| `pub fn spawn(store: Arc<TenantIsolatedKvStore>) -> Self` | Spawnt den Eviction-Worker auf einem dedizierten OS-Thread. | `src/kv_segment/eviction_worker.rs` |
| `pub fn trigger_eviction(&self, target_free_bytes: usize)` | Nicht-blockierender Trigger vom Hot-Path aus. | `src/kv_segment/eviction_worker.rs` |
| `pub fn as_bytes(&self) -> &[u8]` | Read-Only-Zugriff. Kein Klartext-Export nach außen ohne expliziten Call. | `src/kv_segment/segment.rs` |
| `pub fn decrypt_data(&self, cipher: &KvSegmentCipher) -> Result<Vec<u8>, crate::CryptoError>` | Entschlüsselt die Daten des Segments, falls es verschlüsselt ist. Gibt bei Klartext-Segmenten direkt einen Klon der `data`-Bytes zurück. | `src/kv_segment/segment.rs` |
| `pub fn is_empty(&self) -> bool` | Prüft ob das Segment leer ist. | `src/kv_segment/segment.rs` |
| `pub fn len(&self) -> usize` | Länge der Tensor-Bytes in Bytes. | `src/kv_segment/segment.rs` |
| `pub fn new(tenant_id: TenantId, segment_id: u64, data: Vec<u8>) -> Self` | Erstellt ein neues Klartext-KV-Cache-Segment (Default / Zero-Config, P12-konform). | `src/kv_segment/segment.rs` |
| `pub fn new_encrypted( cipher: &KvSegmentCipher, tenant_id: TenantId, segment_id: u64, model_fingerprint: ModelFingerprint, rope_offset: Option<usize>, plaintext: &[u8], ) -> Result<Self, crate::CryptoError>` | Erstellt ein verschlüsseltes KV-Cache-Segment aus einem Klartext-Tensor. | `src/kv_segment/segment.rs` |
| `pub fn new_with_metadata( tenant_id: TenantId, segment_id: u64, data: Vec<u8>, #[cfg(feature = "kv-encryption")] model_fingerprint: Option<ModelFingerprint>, rope_offset: Option<usize>, ) -> Self` | Erstellt ein neues Klartext-KV-Cache-Segment mit optionalen Metadaten. | `src/kv_segment/segment.rs` |
| `pub fn to_spill_bytes(&self) -> Vec<u8>` | Serialisiert das Segment bzw. dessen verschlüsselte Layer für Tier-2-LSM-Spill. | `src/kv_segment/segment.rs` |
| `pub fn clear_all(&self)` | — | `src/kv_segment/store.rs` |
| `pub fn evict_lru_fair(&self, target_free_bytes: usize) -> usize` | Evictiert KV-Segmente unter Erhaltung von Tenant-Fairness via Round-Robin über alle aktiven Tenants. Im Gegensatz zu `evict_lru_global()` v… | `src/kv_segment/store.rs` |
| `pub fn evict_lru_fair_with_hook<F>(&self, target_free_bytes: usize, mut hook: F) -> usize where F: FnMut(),` | — | `src/kv_segment/store.rs` |
| `pub fn get_decrypted_segment( &self, cipher: &crate::KvSegmentCipher, tenant: TenantId, segment_id: u64, ) -> Result<Option<Vec<u8>>, crate::CryptoError>` | Liest ein bestimmtes Segment eines Mandanten und entschlüsselt es falls nötig. | `src/kv_segment/store.rs` |
| `pub fn get_segment_bytes(&self, tenant: TenantId, segment_id: u64) -> Option<Vec<u8>>` | Liefert unverschlüsselte Segment-Bytes für einen Tenant (Klartext-Modus). | `src/kv_segment/store.rs` |
| `pub fn get_segments(&self, tenant: TenantId) -> Vec<u64>` | INV-TENANT-Analogon für KV-Bridge: Ein Tenant kann niemals Segmente eines anderen Tenants lesen. Strukturell erzwungen durch getrennte Maps. | `src/kv_segment/store.rs` |
| `pub fn get_tenant_segment_len(&self, tenant: TenantId) -> usize` | Gibt die Anzahl der gespeicherten Segmente für einen bestimmten Tenant zurück. | `src/kv_segment/store.rs` |
| `pub fn insert_encrypted_segment( &self, cipher: &crate::KvSegmentCipher, tenant: TenantId, segment_id: u64, model_fingerprint: crate::ModelFingerprint, rope_offset: Option<usize>, plaintext: &[u8], ) -> Result<(), crate::CryptoError>` | Verschlüsselt einen Klartext-Tensor und fügt ein verschlüsseltes Segment ein. | `src/kv_segment/store.rs` |
| `pub fn insert_segment(&self, tenant: TenantId, segment: KvSegment)` | Fügt ein Segment für einen bestimmten Tenant ein. | `src/kv_segment/store.rs` |
| `pub fn new() -> Self` | Erstellt einen neuen tenant-isolierten KV-Store mit Standard-Shard-Anzahl (32). | `src/kv_segment/store.rs` |
| `pub fn on_rollback(&self, tenant: TenantId, chunk_ids: &[u64])` | Bereinigt assoziierte KV-Segmente bei einem transaktionalen Rollback. Delegiert direkt an `remove_segments_for_rollback`, um verwaiste KV-S… | `src/kv_segment/store.rs` |
| `pub fn remove_segment(&self, tenant: TenantId, segment_id: u64)` | Entfernt ein einzelnes Segment. Convenience-Wrapper um `remove_segments_for_rollback`. | `src/kv_segment/store.rs` |
| `pub fn remove_segments_for_rollback(&self, tenant: TenantId, segment_ids: &[u64])` | Entfernt alle Segmente mit den angegebenen `segment_ids` für den gegebenen Tenant. Muss bei transaktionalem Rollback aufgerufen werden, wen… | `src/kv_segment/store.rs` |
| `pub fn set_spill_handler(&self, handler: SpillHandler)` | Registriert einen Spill-Handler für evictierte Segmente. | `src/kv_segment/store.rs` |
| `pub fn with_capacity(segment_capacity_per_tenant: usize) -> Self` | Erstellt einen Store mit konfigurierter Segment-Kapazität pro Tenant. | `src/kv_segment/store.rs` |
| `pub fn with_shard_count(n: usize) -> Self` | Erstellt einen Store mit angegebener Shard-Anzahl. Die Shard-Anzahl wird automatisch auf die nächste Zweierpotenz aufgerundet. | `src/kv_segment/store.rs` |
| `pub fn decrypt_record( &self, master_key: &KeyManager, payload: &EncryptedRecordPayload, ) -> Result<Vec<u8>>` | Decrypts an `EncryptedRecordPayload` using Envelope Encryption. Fails with `CryptoError::Crypto` if group or record has been revoked. | `src/kv_shredding.rs` |
| `pub fn decrypt_with_group( &self, group_id: u64, ciphertext: &[u8], nonce_bytes: &[u8; 12], ) -> Result<Vec<u8>>` | Decrypts ciphertext using the sub-key (KEK) for `group_id`. Returns `Err(CryptoError::Crypto(...))` if key was revoked or decryption fails. | `src/kv_shredding.rs` |
| `pub fn derive_subkey( registry: &KeyRegistry, master_key: &KeyManager, group_id: u64, ) -> Result<SubKey>` | Derives or retrieves a random sub-key (Group KEK) for a shred group via `KeyRegistry`. | `src/kv_shredding.rs` |
| `pub fn encrypt_record( &self, master_key: &KeyManager, group_id: u64, record_id: u64, plaintext: &[u8], ) -> Result<EncryptedRecordPayload>` | Encrypts a record using Envelope Encryption (Random KEK per group, Random DEK per record). | `src/kv_shredding.rs` |
| `pub fn encrypt_with_group( &self, master_key: &KeyManager, group_id: u64, plaintext: &[u8], ) -> Result<(Vec<u8>, [u8` | Encrypts plaintext using the sub-key (KEK) for `group_id`. | `src/kv_shredding.rs` |
| `pub fn get_or_derive(&self, master_key: &KeyManager, group_id: u64) -> Result<SubKey>` | Retrieves or generates a random Group KEK for `group_id`. Returns `Err(CryptoError::KeyRevoked(...))` if `group_id` has been revoked/shredd… | `src/kv_shredding.rs` |
| `pub fn get_wrapped_dek(&self, group_id: u64, record_id: u64) -> Option<(Vec<u8>, [u8` | Retrieves the wrapped DEK and nonce for a record, if available and active. | `src/kv_shredding.rs` |
| `pub fn get_wrapped_kek(&self, group_id: u64) -> Option<(Vec<u8>, [u8` | Retrieves the wrapped KEK and nonce for a group, if available and active. | `src/kv_shredding.rs` |
| `pub fn is_group_active(&self, group_id: u64) -> bool` | Returns `true` if `group_id` has an active KEK and is not revoked. | `src/kv_shredding.rs` |
| `pub fn is_group_revoked(&self, group_id: u64) -> bool` | Checks if a group is revoked. | `src/kv_shredding.rs` |
| `pub fn is_key_active(&self, group_id: u64) -> bool` | Returns `true` if `group_id` has an active sub-key in the registry. | `src/kv_shredding.rs` |
| `pub fn is_record_active(&self, group_id: u64, record_id: u64) -> bool` | Returns `true` if `record_id` in `group_id` is active and not revoked. | `src/kv_shredding.rs` |
| `pub fn new() -> Self` | Creates a new empty `KeyRegistry`. | `src/kv_shredding.rs` |
| `pub fn new_in_memory( clock: Arc<dyn contextra_ports::Clock>, signing_key: Option<ed25519_dalek::SigningKey>, verifying_key: ed25519_dalek::VerifyingKey, ) -> Self` | Creates a new `KeyRegistry` backed by an in-memory `RevocationLog`. | `src/kv_shredding.rs` |
| `pub fn revoke_group(&self, group_id: u64) -> Result<bool>` | Revokes (destroys) the Group KEK for `group_id` (Group Deletion / Group Crypto-Shredding). Returns `true` if the group was active and is no… | `src/kv_shredding.rs` |
| `pub fn revoke_record(&self, group_id: u64, record_id: u64) -> Result<bool>` | Revokes (destroys) the DEK wrap for a single record within `group_id` (Single Record Deletion). Leaves neighbor records in the same group i… | `src/kv_shredding.rs` |
| `pub fn revoke_subkey(&self, group_id: u64) -> Result<bool>` | Revokes (destroys) the Group KEK for `group_id` in O(1). Alias for `revoke_group`. | `src/kv_shredding.rs` |
| `pub fn was_group_ever_registered(&self, group_id: u64) -> bool` | Checks if a group was ever registered in this registry. | `src/kv_shredding.rs` |
| `pub fn with_revocation_listener(mut self, listener: Arc<dyn GroupRevocationListener>) -> Self` | Attaches a `GroupRevocationListener` to be notified on group revocation. | `src/kv_shredding.rs` |
| `pub fn with_revocation_log(mut self, log: Arc<RevocationLog>) -> Self` | Attaches an optional `RevocationLog` to check for persisted key/group revocations. | `src/kv_shredding.rs` |
| `pub fn append(&self, target: RevocationTarget) -> Result<RevocationEntry>` | Fügt einen neuen Widerrufs-Eintrag für `target` an den Log an. Nutzt den injizierten Clock-Port zur Generierung des Zeitstempels und den Si… | `src/revocation_log.rs` |
| `pub fn canonical_bytes(&self) -> Vec<u8>` | Gibt eine kanonische Byte-Repräsentation des Targets für das Hashing zurück. | `src/revocation_log.rs` |
| `pub fn compute_entry_hash( sequence_number: u64, target: &RevocationTarget, revoked_at_nanos: u64, prev_hash: &[u8; 32], ) -> [u8` | Berechnet den SHA-256-Hash eines Eintrags aus seinen kanonischen Feldern. | `src/revocation_log.rs` |
| `pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8` | — | `src/revocation_log.rs` |
| `pub fn is_empty(&self) -> bool` | Prüft, ob der Log leer ist. | `src/revocation_log.rs` |
| `pub fn is_revoked(&self, target: &RevocationTarget) -> bool` | Prüft, ob ein gegebenes Target (KEK, DEK, Group oder Record) widerrufen wurde. | `src/revocation_log.rs` |
| `pub fn len(&self) -> usize` | Gibt die Anzahl der Widerrufe im Log zurück. | `src/revocation_log.rs` |
| `pub fn new_in_memory( clock: Arc<dyn Clock>, signing_key: Option<SigningKey>, verifying_key: VerifyingKey, ) -> Self` | Erstellt einen neuen In-Memory-Widerrufslog ohne Dateipersistenz. | `src/revocation_log.rs` |
| `pub fn open_or_create( path: impl AsRef<Path>, clock: Arc<dyn Clock>, signing_key: Option<SigningKey>, verifying_key: VerifyingKey, ) -> Result<Self>` | Öffnet einen bestehenden Widerrufslog oder erzeugt einen neuen unter `path`. Beim Öffnen einer bestehenden Datei wird die komplette Hash-Ke… | `src/revocation_log.rs` |
| `pub fn serialize<S>(bytes: &[u8; 64], serializer: S) -> Result<S::Ok, S::Error> where S: Serializer,` | — | `src/revocation_log.rs` |
| `pub fn verify_integrity(&self) -> Result<()>` | Verifiziert die Integrität der gesamten Logeinträge-Kette (Sequenz, Hashes, Ed25519-Signaturen). | `src/revocation_log.rs` |
| `pub fn verify_wal_chain_completeness( wal_tail_hmac: &[u8; 32], manifest_high_water_mark: &[u8; 32], ) -> Result<(), CryptoError>` | Verifiziert, dass der WAL-Tail nicht abgeschnitten wurde, indem der letzte bekannte WAL-HMAC mit der im Checkpoint-Manifest persistierten H… | `src/wal_completeness.rs` |
| `pub fn decrypt_chunk(&self, data: &[u8]) -> Result<Vec<u8>>` | Decrypts the WAL chunk by extracting the prepended 12-byte nonce from the data. | `src/wal_crypto.rs` |
| `pub fn encrypt_chunk(&self, payload: &[u8]) -> Result<Vec<u8>>` | Wraps the internal WAL chunk in AES-256-GCM-SIV stream. Prepends the 12-byte nonce to the encrypted ciphertext. | `src/wal_crypto.rs` |
| `pub fn finalize(self) -> [u8` | — | `src/wal_crypto.rs` |
| `pub fn handoff_from(&mut self, source: &Self)` | Transferiert sowohl `last_hmac` als auch `last_seq_no` von einer `source`-Instanz für einen vollständigen Verifier-Handoff. | `src/wal_crypto.rs` |
| `pub fn last_hmac_snapshot(&self) -> [u8` | Gibt den aktuellen `last_hmac`-Kettenzustand zurück (für Verifier-Handoff). | `src/wal_crypto.rs` |
| `pub fn last_seq_no_snapshot(&self) -> Option<u64>` | Gibt die aktuell bekannte `last_seq_no` zurück (für Verifier-Handoff). | `src/wal_crypto.rs` |
| `pub fn new(key_manager: KeyManager, file_id: &[u8]) -> Result<Self>` | Creates a new EncryptedWal with per-file key derivation to prevent nonce-reuse. `file_id` (e.g., filename) is used to derive a unique sub-k… | `src/wal_crypto.rs` |
| `pub fn new(integrity_key: &[u8]) -> Result<Self>` | — | `src/wal_crypto.rs` |
| `pub fn new(integrity_key: &[u8]) -> Self` | — | `src/wal_crypto.rs` |
| `pub fn set_last_hmac(&mut self, hmac: [u8; 32])` | Übernimmt einen bereits bekannten `last_hmac`-Kettenzustand, z. B. beim Wechsel von einem Normal- auf einen Legacy-Key-Verifier MITTEN in e… | `src/wal_crypto.rs` |
| `pub fn set_last_seq_no(&mut self, seq_no: Option<u64>)` | Übernimmt eine bereits bekannte `last_seq_no` für den Verifier-Handoff. | `src/wal_crypto.rs` |
| `pub fn skip_hmac_verify_legacy(&mut self, entry: &WalEntrySnapshot)` | Updates the chain state for legacy V1 entries without HMAC verification. | `src/wal_crypto.rs` |
| `pub fn update(&mut self, data: &[u8])` | — | `src/wal_crypto.rs` |
| `pub fn verify_and_update(&mut self, entry: &WalEntrySnapshot, offset: u64) -> Result<()>` | Default verification delegating to V3 verification. | `src/wal_crypto.rs` |
| `pub fn verify_and_update_v2(&mut self, entry: &WalEntrySnapshot, offset: u64) -> Result<()>` | — | `src/wal_crypto.rs` |
| `pub fn verify_and_update_v3(&mut self, entry: &WalEntrySnapshot, offset: u64) -> Result<()>` | Verifies a V3 entry (with tx_id and length prefixes) and updates the chain state. | `src/wal_crypto.rs` |


---

## `contextra-vector`

| Attribut | Wert |
|---|---|
| **Ring** | 0 — Ring 0 — Foundation & Pure Numerics/Types |
| **Status** | In Migration |
| **Zweck** | HNSW- und DiskANN-Vektorindex-Implementierungen. |
| **Unsafe-Code** | Nein (`#![forbid(unsafe_code)]`) |
| **Interne Abhängigkeiten** | `contextra-core`, `contextra-crypto`, `contextra-ports`, `contextra-simd`, `contextra-sys`, `contextra-types` |
| **Externe Abhängigkeiten (Auswahl)** | `ahash`, `memmap2`, `parking_lot`, `rand`, `roaring`, `serde`, `subtle`, `tracing` |

### Architekturbeschreibung

Contextra Index — HNSW vector index with SIMD distance computation.

### Modul- und Re-Export-Oberfläche (lib.rs)

```rust
pub mod diskann;
pub mod distance;
pub mod hnsw;
pub mod persistence;
pub mod quantize;
pub mod acorn;
pub mod candidate_stream;
pub mod compute_pool;
pub mod partial_rebuild;
pub mod quantize_rabitq;
pub use candidate_stream::{VectorCandidateStream, DEFAULT_VECTOR_STREAM_BATCH_SIZE};
pub use compute_pool::ComputePool;
pub use diskann::{DiskAnnConfig, DiskAnnFallbackPolicy, DiskAnnIndex};
pub use hnsw::{HnswConfig, HnswIndex, RebuildStatus};
pub use persistence::{HnswHeader, MmapIndex};
pub use quantize::ScalarQuantizer;
pub use quantize_rabitq::RaBitQQuantizer;
```

### Feature-Flags

| Flag | Aktiviert |
|---|---|
| `default` | `[]` |
| `docid-128` | `["contextra-types/docid-128"]` |
| `experimental-diskann` | `[]` |
| `experimental-rabitq` | `[]` |
| `experimental-predicate-augmented-search` | `[]` |
| `graph` | `[]` |
| `partial-index-rebuild` | `[]` |

### Schnittstellen — Public Traits (2)

**`FilteredIndex`** — *Prädikatsagnostischer Suchpfad (ACORN-Muster, Patel et al. 2024). Im Gegensatz zu Post-Filtering wird das Prädikat WÄHREND der Graph-Traversal ausgewertet, mit erhöhter Kantenzahl (γ-Augmentierung) zur Konnektivitätssicherung bei hoher Selektivität.* (`src/acorn/mod.rs`)

```rust
pub trait FilteredIndex {
type Predicate: ?Sized;

    fn search_knn_acorn(
        &self,
        query: &[f32],
        k: usize,
        predicate: &Self::Predicate,
        gamma: u32,
    ) -> Result<Vec<(DocId, f32)>, AcornError>;
}
```

**`GhostFreeVectorIndex`** — *Trait providing ghost-free vector index deletion with synchronous neighborhood graph repair.* (`src/hnsw/deletion.rs`)

```rust
pub trait GhostFreeVectorIndex {
/// Deletes `doc_id` and synchronously repairs all neighborhood pointers across all layers
    /// before returning.
    ///
    /// # Invariant
    /// `INV-DELETION-2`: After `Ok(stats)` returns, no neighborhood pointer in the entire index
    /// references `doc_id`.
    fn remove_with_graph_repair(&mut self, doc_id: DocId) -> Result<DeletionStats>;
}
```

### Öffentliche Datentypen — Structs (33)

| Struct | Felder / Form | Beschreibung | Datei |
|---|---|---|---|
| `AdaptiveEfPolicy` | `{ /// Initial search depth ef for layer 0. pub min_ef: usize, /// Maximum search depth ef limit. pub max_ef: usize, /// Growth factor for increasing ef in each e… }` | Configuration policy for adaptive ef_search (Ada-ef). | `src/hnsw/adaptive_ef.rs` |
| `AdaptiveEfStateMachine` | `{ policy: AdaptiveEfPolicy, k: usize, current_ef: usize, previous_top_k: Vec<DocId>, stable_rounds: usize, round_count: usize, is_terminated: bool, converged: bo… }` | Deterministic state machine managing Ada-ef expansion rounds. | `src/hnsw/adaptive_ef.rs` |
| `AdaptiveEfStats` | `{ /// Final ef depth reached at termination. pub final_ef: usize, /// Total expansion rounds executed. pub rounds: usize, /// Whether search terminated due to ca… }` | Execution statistics for an adaptive search run. | `src/hnsw/adaptive_ef.rs` |
| `BacklinkTable` | `{ pub map: AHashMap<BacklinkKey, Vec<u32>>, }` | Thread-safe $O(1)$ backlink connection table using `AHashMap`. | `src/hnsw/arena.rs` |
| `BatchContext` | `{ pub running_max_layer: usize, pub has_entry_point: bool, pub has_ram_entry_point: bool, pub backlink_map: BacklinkTable, }` | Batch tracking context for running state across multi-operation transaction commits. | `src/hnsw/batch.rs` |
| `Candidate` | `{ pub index: usize, pub distance: f32, }` | Search candidate. | `src/hnsw/types.rs` |
| `ComputePool` | `{ inner: Arc<PoolState>, }` | Bounded compute thread pool for background index rebuilds and heavy CPU tasks. Enforces a hard ceiling on concurrent worker threads to prevent thread… | `src/compute_pool.rs` |
| `DeletionStats` | `{ /// Document ID that was deleted. pub doc_id: DocId, /// Number of neighborhood edges repaired (backlinks removed/replaced). pub repaired_edges: usize, /// Num… }` | Statistics returned after a ghost-free node deletion and neighborhood repair. | `src/hnsw/deletion.rs` |
| `DiskAnnConfig` | `{ /// Path to the on-disk index file. pub index_path: PathBuf, /// Vector dimension. pub dimension: usize, /// Maximum graph degree (R). pub max_degree: usize, /… }` | Configuration for DiskANN index. | `src/diskann/config.rs` |
| `DiskAnnIndex` | `{ pub(crate) inner: Arc<DiskAnnIndexInner>, }` | DiskANN out-of-core vector index. This index uses `memmap2` to offload vector data and graph edges to disk. Note that mmap operations and page faults… | `src/diskann/types.rs` |
| `HnswArena` | `{ pub arena: RwLock<Vec<u32>>, pub offsets: RwLock<Vec<usize>>, pub capacities: RwLock<Vec<usize>>, pub count_offsets: RwLock<Vec<usize>>, pub counts: RwLock<Vec… }` | Contiguous 64-byte aligned arena allocator for HNSW node connections with free-list slot reuse. | `src/hnsw/arena.rs` |
| `HnswColdCore` | `{ pub config: HnswConfig, pub validation_error: Option<String>, pub tx_buffer: TxBuffer<Vec<f32>>, pub quantizer: RwLock<Option<crate::quantize::ScalarQuantizer>… }` | — | `src/hnsw/types.rs` |
| `HnswConfig` | `{ /// Vector dimensionality. pub dimension: usize, /// Maximum number of elements. pub max_elements: usize, /// Number of connections per element (M parameter).… }` | — | `src/hnsw/config.rs` |
| `HnswConfigBuilder` | `{ config: HnswConfig, }` | Builder for HnswConfig with resource limit enforcements to prevent OOM. | `src/hnsw/config.rs` |
| `HnswHeader` | `{ magic: u32, version: u16, dimension: u32, m: u32, metric: u8, quantized: u8, q_min: f32, // Legacy v1 field q_max: f32, // Legacy v1 field node_count: u64, ent… }` | The header of an HNSW persistent file. | `src/persistence/header.rs` |
| `HnswHotCore` | `{ pub nodes: RwLock<Vec<HnswNode>>, #[cfg(not(feature = "docid-128"))] pub doc_to_node: RwLock<AHashMap<u64, usize>>, #[cfg(feature = "docid-128")] pub doc_to_no… }` | — | `src/hnsw/types.rs` |
| `HnswIndex` | `{ pub(super) inner: std::sync::Arc<HnswIndexCore>, }` | The HNSW (Hierarchical Navigable Small World) vector index. | `src/hnsw/types.rs` |
| `HnswIndexCore` | `{ pub hot: HnswHotCore, pub cold: HnswColdCore, }` | The core implementation of the HNSW index. | `src/hnsw/types.rs` |
| `HnswNode` | `{ pub(super) doc_id: DocId, pub(super) vector: VectorData, pub(super) max_layer: usize, pub(super) committed_tx: u64, }` | A node in the HNSW graph. | `src/hnsw/types.rs` |
| `MmapIndex` | `{ pub mmap: std::sync::Arc<memmap2::Mmap>, pub header: HnswHeader, }` | A reader for memory-mapped HNSW indices. | `src/persistence/mmap.rs` |
| `NaiveReferenceIndex` | `{ vectors: Vec<(DocId, Vec<f32>)>, metric: DistanceMetric, }` | Naive reference index performing brute-force search over all stored points. **NICHT für Produktion — O(n) Referenzimplementierung** Serves exclusivel… | `src/acorn/naive_reference.rs` |
| `NeighborBacklink` | `{ pub neighbor_ram_idx: usize, pub layer: usize, pub updated_connections: Vec<u32>, }` | Back-link connection update for a neighbor node at a specific layer. | `src/hnsw/batch.rs` |
| `NodeRecord` | `{ #[cfg(not(feature = "docid-128"))] pub doc_id: u64, #[cfg(feature = "docid-128")] pub doc_id: u128, pub max_layer: u8, pub vector_offset: u64, pub connections_… }` | Represents a node's metadata in the flat file. | `src/persistence/node.rs` |
| `PartialRebuildConfig` | `{ /// Kritische Übersättigungs-Schwelle θ_c. Default: 3.0 (3× globale Dichte). pub critical_ratio: f32, /// Ringpuffer-Größe N für Traversal-Statistik. Default:… }` | Konfiguration für den Partial-Rebuild-Trigger. | `src/partial_rebuild.rs` |
| `PreparedInsert` | `{ pub doc_id: DocId, pub vector_data: VectorData, pub new_layer: usize, pub new_idx: usize, pub final_connections: Vec<Vec<u32>>, pub neighbor_backlinks: Vec<Nei… }` | — | `src/hnsw/batch.rs` |
| `RaBitQQuantizer` | `{ dimension: usize, /// Row-major orthogonal rotation matrix R of size dimension x dimension. rotation_matrix: Vec<f32>, }` | RaBitQ Quantizer with random orthogonal rotation and scalar metadata. | `src/quantize_rabitq.rs` |
| `RebuildGuard` | `(tuple/unit)` | — | `src/hnsw/types.rs` |
| `ScalarQuantizer` | `{ pub(crate) mins: Vec<f32>, pub(crate) maxes: Vec<f32>, pub(crate) scales: Vec<f32>, pub(crate) inv_scales: Vec<f32>, pub(crate) dimension: usize, #[serde(skip,… }` | An 8-bit Scalar Quantizer (SQ8) with per-dimension scaling. Quantization reduces the memory footprint of vector storage by 4x. Per-dimension scaling… | `src/quantize/mod.rs` |
| `SnapshotPinGuard` | `(tuple/unit)` | — | `src/hnsw/types.rs` |
| `Sq8Bias` | `{ /// Mean quantization error across distance computations. pub mean_bias: f32, /// Variance of quantization error across distance computations. pub variance_bia… }` | Measured quantization bias and variance statistics for SQ8 distance calculations. | `src/hnsw/sq8_bias.rs` |
| `TaskHandle` | `{ rx: Receiver<T>, }` | Handle to await the outcome of a task submitted to [`ComputePool`]. | `src/compute_pool.rs` |
| `TraversalTracker` | `{ config: PartialRebuildConfig, visited_ring: VecDeque<Vec<u64>>, // Ein Vec<u64> pro Traversierungsaufruf }` | Ringpuffer für besuchte HNSW-Knoten-IDs während ef_search-Traversierungen. | `src/partial_rebuild.rs` |
| `VectorCandidateStream` | `{ index: &'a V, query: Vec<f32>, seq_no: Option<u64>, batch_size: usize, buffer: VecDeque<ScoredDocument>, seen: HashSet<DocId>, fetched_k: usize, yielded: usize… }` | On-demand loading async pull-stream for vector search candidates. # Candidate Ordering & Determinism Vector search scores represent relevance (higher… | `src/candidate_stream.rs` |

### Öffentliche Datentypen — Enums (4)

| Enum | Varianten | Beschreibung | Datei |
|---|---|---|---|
| `AcornError` | `/// Dimension mismatch between query and vector database points. DimensionMismatch { /// Expected dimension. expected: usize, /// Provided query dimension. got: usize, }, /// Invalid search parameter…` | Errors occurring during ACORN search or graph traversal operations. | `src/acorn/mod.rs` |
| `DiskAnnFallbackPolicy` | `/// Transparently fall back to in-memory HNSW index on load failure or detected corruption. #[default] UseHnswOnFailure, /// Immediately return a typed error on load failure or index corruption. Fail…` | Fallback behavior policy when DiskANN loading, integrity checks, or reads fail. | `src/diskann/config.rs` |
| `RebuildStatus` | `/// Kein Rebuild läuft oder geplant. Idle, /// Rebuild läuft gerade im Hintergrund. Running, /// Rebuild wurde getriggert, startet bald. Pending,` | — | `src/hnsw/types.rs` |
| `VectorData` | `/// Standard 32-bit floating point vectors. F32(Vec<f32>), /// 8-bit quantized vectors (SQ8). U8(Vec<u8>),` | — | `src/hnsw/types.rs` |

### Typ-Aliase (1)

| Alias | Definition | Beschreibung |
|---|---|---|
| `BacklinkKey` | `type BacklinkKey= (usize, usize)` | Composite key (neighbor_ram_idx, layer) for backlink lookups. |

### Konstanten (13)

| Konstante | Typ | Beschreibung |
|---|---|---|
| `ARENA_ALIGNMENT_BYTES` | `usize` | Slot alignment in bytes (64-byte alignment = 16 u32 elements for SIMD cache-line compatibility). |
| `ARENA_ALIGNMENT_U32` | `usize` | Alignment in u32 elements (64 bytes / 4 bytes = 16 u32 elements). |
| `DEFAULT_LAYER_SEED` | `u64` | Default-Seed für die deterministische HNSW-Ebenenzuweisung (wie in hnswlib üblich). |
| `DEFAULT_P_HIGH` | `f32` | Default high percentile for quantization scaling (99.5%). |
| `DEFAULT_P_LOW` | `f32` | Default low percentile for quantization scaling (0.5%). |
| `DEFAULT_VECTOR_STREAM_BATCH_SIZE` | `usize` | Default batch size for vector candidate retrieval. |
| `HNSW_MAGIC` | `u32` | Magic number for HNSW files (0x484E5357 = "HNSW"). |
| `HNSW_REBUILD_DELETION_RATIO` | `f64` | Standard-Löschanteil (0.10 = 10 % gelöschte Knoten), ab dem ein Rebuild getriggert wird. |
| `HNSW_VERSION` | `u16` | Current file format version. |
| `SENTINEL_NO_ENTRY_POINT` | `u32` | Sentinel-Wert für ungültigen / nicht gesetzten Entry-Point in HNSW. |
| `SIZE` | `usize` | — |
| `SIZE` | `usize` | — |
| `SIZE` | `usize` | — |

### Öffentliche Funktionen (184)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn compute_gamma_edge_budget(base_degree: usize, predicate_selectivity: f32) -> usize` | Computes the augmented edge budget factor $\gamma$ for graph expansion in ACORN search. Based on Patel et al. 2024 (ACORN: Performant and P… | `src/acorn/gamma_augmentation.rs` |
| `pub fn from_vectors(vectors: Vec<(DocId, Vec<f32>)>) -> Self` | Creates a `NaiveReferenceIndex` pre-populated with vectors using Euclidean distance. **Test & Benchmark Only**: Used to construct baseline… | `src/acorn/naive_reference.rs` |
| `pub fn insert(&mut self, id: DocId, vector: Vec<f32>)` | Inserts a document vector into the index. | `src/acorn/naive_reference.rs` |
| `pub fn new(metric: DistanceMetric) -> Self` | Creates a new empty `NaiveReferenceIndex` with the specified distance metric. | `src/acorn/naive_reference.rs` |
| `pub fn fetched_depth(&self) -> usize` | Returns the last $k$ search depth requested from the vector index. | `src/candidate_stream.rs` |
| `pub fn is_exhausted(&self) -> bool` | Returns whether the candidate stream is exhausted. | `src/candidate_stream.rs` |
| `pub fn new(index: &'a V, query: impl Into<Vec<f32>>, seq_no: Option<u64>) -> Self` | Creates a new vector candidate stream for a query and index snapshot. | `src/candidate_stream.rs` |
| `pub async fn next_batch(&mut self) -> Result<Vec<ScoredDocument>>` | Fetches the next batch of candidate documents (up to `batch_size`). Returns an empty `Vec` when exhausted. | `src/candidate_stream.rs` |
| `pub async fn next_doc(&mut self) -> Result<Option<DocId>>` | Fetches the next single candidate document ID. Returns `Ok(None)` when exhausted. | `src/candidate_stream.rs` |
| `pub fn with_batch_size(mut self, n: usize) -> Self` | Sets a custom batch size (minimum 1). | `src/candidate_stream.rs` |
| `pub fn yielded(&self) -> usize` | Returns the total number of items yielded so far. | `src/candidate_stream.rs` |
| `pub fn active_jobs(&self) -> usize` | Returns the number of jobs currently executing in worker threads. | `src/compute_pool.rs` |
| `pub fn execute<F>(&self, f: F) where F: FnOnce() + Send + 'static,` | Submits a fire-and-forget closure for execution on the pool. | `src/compute_pool.rs` |
| `pub fn join(self) -> Result<T, std::sync::mpsc::RecvError>` | Waits for the task to complete and returns its result, or an error if the task failed or was dropped. | `src/compute_pool.rs` |
| `pub fn max_workers(&self) -> usize` | Returns the maximum worker capacity of this pool. | `src/compute_pool.rs` |
| `pub fn new(max_workers: usize) -> Self` | Creates a new [`ComputePool`] with a fixed worker thread ceiling. | `src/compute_pool.rs` |
| `pub fn spawn<F, T>(&self, f: F) -> TaskHandle<T> where F: FnOnce() -> T + Send + 'static, T: Send + 'static,` | Submits a function and returns a [`TaskHandle`] to wait for its return value. | `src/compute_pool.rs` |
| `pub async fn build(&self, vectors: &[Vec<f32>], ids: &[DocId]) -> Result<()>` | Builds the index from a set of vectors. | `src/diskann/build.rs` |
| `pub fn build_sync(&self, vectors: &[Vec<f32>], ids: &[DocId]) -> Result<()>` | — | `src/diskann/build.rs` |
| `pub fn build_to_path_sync( &self, target_path: &std::path::Path, vectors: &[Vec<f32>], ids: &[DocId], ) -> Result<()>` | — | `src/diskann/build.rs` |
| `pub fn recover_pending_delta_sync(&self) -> Result<usize>` | Liest uncommittete Pending-Vektoren aus `pending.wal` und fügt sie in den Index ein. | `src/diskann/build.rs` |
| `pub async fn load(&self) -> Result<()>` | Loads the index from the configured path. | `src/diskann/persistence.rs` |
| `pub fn load_sync(&self) -> Result<()>` | — | `src/diskann/persistence.rs` |
| `pub async fn persist_delta(&self) -> Result<()>` | — | `src/diskann/persistence.rs` |
| `pub fn persist_delta_sync(&self) -> Result<()>` | — | `src/diskann/persistence.rs` |
| `pub fn trigger_background_persist_delta(&self)` | — | `src/diskann/persistence.rs` |
| `pub async fn search_predicate_augmented( &self, query: &[f32], k: usize, filter: &(dyn Fn(DocId) -> bool + Send + Sync), gamma: usize, ) -> Result<Vec<ScoredDocument>>` | Executes a predicate-augmented vector search using ACORN 2-hop neighbor expansion during graph traversal. # Parameters - `query`: Query vec… | `src/diskann/predicate_augmented.rs` |
| `pub async fn search_internal(&self, query: &[f32], k: usize) -> Result<Vec<ScoredDocument>>` | — | `src/diskann/search.rs` |
| `pub fn current_ef(&self) -> usize` | Returns the active `ef_search` for the current expansion round. | `src/hnsw/adaptive_ef.rs` |
| `pub fn is_terminated(&self) -> bool` | Returns whether the state machine has reached a termination condition. | `src/hnsw/adaptive_ef.rs` |
| `pub fn new( min_ef: usize, max_ef: usize, growth_factor: f32, convergence_window: usize, stability_threshold: f32, ) -> Result<Self>` | Validating constructor for `AdaptiveEfPolicy`. | `src/hnsw/adaptive_ef.rs` |
| `pub fn new(policy: AdaptiveEfPolicy, k: usize) -> Self` | Initializes a new state machine for adaptive search with target top-k. | `src/hnsw/adaptive_ef.rs` |
| `pub fn round_count(&self) -> usize` | Returns the number of expansion rounds completed. | `src/hnsw/adaptive_ef.rs` |
| `pub fn stats(&self) -> AdaptiveEfStats` | Returns the current execution statistics. | `src/hnsw/adaptive_ef.rs` |
| `pub fn step(&mut self, current_top_k: &[DocId]) -> bool` | Evaluates current top-k candidate set, updating state and determining if search should terminate. Returns `true` if search should terminate… | `src/hnsw/adaptive_ef.rs` |
| `pub fn validate(&self) -> Result<()>` | Validates policy invariants. | `src/hnsw/adaptive_ef.rs` |
| `pub fn align_capacity(raw_capacity: usize) -> usize` | Rounds up an allocation size in u32s to the nearest 64-byte (16 x u32) boundary. | `src/hnsw/arena.rs` |
| `pub fn allocate_node( &self, max_layer: usize, m: usize, final_connections: &[Vec<u32>], ) -> Result<usize>` | Allocates or reuses a slot for a node's neighbor lists across all its layers. Ensures 64-byte alignment and validates capacity on slot reus… | `src/hnsw/arena.rs` |
| `pub fn clear(&mut self)` | Clears all stored backlinks. | `src/hnsw/arena.rs` |
| `pub fn free_node(&self, ram_idx: usize)` | Marks a node's slot as free for future slot reuse. | `src/hnsw/arena.rs` |
| `pub fn get(&self, neighbor_ram_idx: usize, layer: usize) -> Option<&Vec<u32>>` | Gets connection list for (neighbor_ram_idx, layer) in O(1) time. | `src/hnsw/arena.rs` |
| `pub fn get_ram_node_connections(&self, ram_idx: usize, layer: usize, m: usize) -> Vec<u32>` | Gets connection list for a node at a specific layer without panicking. | `src/hnsw/arena.rs` |
| `pub fn insert(&mut self, neighbor_ram_idx: usize, layer: usize, connections: Vec<u32>)` | Inserts or updates connection list for (neighbor_ram_idx, layer) in O(1) time. | `src/hnsw/arena.rs` |
| `pub fn layer_offset(node_offset: usize, layer: usize, m: usize) -> usize` | Calculates layer offset within a node's allocated arena block. | `src/hnsw/arena.rs` |
| `pub fn new() -> Self` | Creates a new `BacklinkTable`. | `src/hnsw/arena.rs` |
| `pub fn new() -> Self` | Creates a new `HnswArena`. | `src/hnsw/arena.rs` |
| `pub fn try_relink_pruned_neighbors( &self, ram_idx: usize, layer: usize, m: usize, kept_neighbors: &[u32], ) -> bool` | Non-blocking relinking for lazy neighbor pruning during search traversal. | `src/hnsw/arena.rs` |
| `pub fn update_backlink( &self, ram_idx: usize, layer: usize, m: usize, updated_connections: &[u32], ) -> Result<()>` | Updates connection list for a backlink update safely. | `src/hnsw/arena.rs` |
| `pub fn new(core: &HnswIndexCore) -> Self` | — | `src/hnsw/batch.rs` |
| `pub fn backoff_alert_threshold(mut self, threshold: u32) -> Self` | Sets the threshold of consecutive rapid rebuilds that trigger warning alerts. | `src/hnsw/config.rs` |
| `pub fn backoff_base_delay(mut self, delay: std::time::Duration) -> Self` | Sets the base delay for rebuild exponential backoff cooldown. | `src/hnsw/config.rs` |
| `pub fn backoff_max_delay(mut self, delay: std::time::Duration) -> Self` | Sets the maximum cap for rebuild exponential backoff cooldown. | `src/hnsw/config.rs` |
| `pub fn build(self) -> Result<HnswConfig>` | Build the configuration after validating bounds. | `src/hnsw/config.rs` |
| `pub fn distance_metric(mut self, metric: DistanceMetric) -> Self` | Use a specific distance metric. | `src/hnsw/config.rs` |
| `pub fn ef_construction(mut self, ef: usize) -> Self` | Set dynamic candidate list size for construction. | `src/hnsw/config.rs` |
| `pub fn ef_search(mut self, ef: usize) -> Self` | Set dynamic candidate list size for search. | `src/hnsw/config.rs` |
| `pub fn m(mut self, m: usize) -> Self` | Set the number of connections per element (M). | `src/hnsw/config.rs` |
| `pub fn max_elements(mut self, max: usize) -> Self` | Set max elements with a hardcap limit to avoid OOM. | `src/hnsw/config.rs` |
| `pub fn new(dimension: usize) -> Self` | Creates a new builder with the chosen dimensionality. | `src/hnsw/config.rs` |
| `pub fn partial_rebuild_config( mut self, config: crate::partial_rebuild::PartialRebuildConfig, ) -> Self` | Sets the partial rebuild configuration for local hot-path rebuilds (F-02). | `src/hnsw/config.rs` |
| `pub fn quantize(mut self, quantize: bool) -> Self` | Enable or disable scalar quantization (SQ8) to reduce footprint. | `src/hnsw/config.rs` |
| `pub fn quantizer_drift_threshold(mut self, threshold: f32) -> Self` | Sets the drift ratio threshold for ScalarQuantizer recalibration and rebuilds. | `src/hnsw/config.rs` |
| `pub fn quantizer_recalibration_sample_size(mut self, size: usize) -> Self` | Sets the sample size used for ScalarQuantizer recalibration during rebuilds. | `src/hnsw/config.rs` |
| `pub fn rebuild_threshold(mut self, threshold: f64) -> Self` | Sets the rebuild threshold. | `src/hnsw/config.rs` |
| `pub fn validate(&self) -> Result<()>` | Validates that the configuration parameters are within acceptable bounds. | `src/hnsw/config.rs` |
| `pub fn apply_insert(&self, prepared: PreparedInsert, tx_id: u64)` | — | `src/hnsw/core_insert.rs` |
| `pub fn check_connectivity(&self) -> contextra_core::Result<()>` | — | `src/hnsw/core_insert.rs` |
| `pub fn compute_insert(&self, id: DocId, vector: &[f32]) -> Result<PreparedInsert>` | — | `src/hnsw/core_insert.rs` |
| `pub fn compute_insert_with_context( &self, id: DocId, vector: &[f32], offset: usize, prior_prepared: &mut [PreparedInsert], batch_ctx: &mut BatchContext, ) -> Result<PreparedInsert>` | — | `src/hnsw/core_insert.rs` |
| `pub fn connectivity_score(&self) -> f64` | — | `src/hnsw/core_insert.rs` |
| `pub fn deleted_ratio(&self) -> f64` | — | `src/hnsw/core_rebuild.rs` |
| `pub fn is_rebuild_required(&self) -> bool` | — | `src/hnsw/core_rebuild.rs` |
| `pub async fn rebuild(&self) -> Result<()>` | — | `src/hnsw/core_rebuild.rs` |
| `pub fn rebuild_count(&self) -> u64` | — | `src/hnsw/core_rebuild.rs` |
| `pub async fn rebuild_region(&self, region_node_ids: Vec<u64>) -> Result<()>` | — | `src/hnsw/core_rebuild.rs` |
| `pub fn rebuild_region_sync(&self, region_node_ids: Vec<u64>) -> Result<()>` | — | `src/hnsw/core_rebuild.rs` |
| `pub fn rebuild_status(&self) -> RebuildStatus` | — | `src/hnsw/core_rebuild.rs` |
| `pub fn rebuild_sync(&self) -> Result<()>` | — | `src/hnsw/core_rebuild.rs` |
| `pub fn visited_dead_nodes(&self) -> u64` | — | `src/hnsw/core_rebuild.rs` |
| `pub async fn wait_for_rebuild(&self) -> bool` | — | `src/hnsw/core_rebuild.rs` |
| `pub async fn wait_for_rebuild_with_timeout(&self, timeout: std::time::Duration) -> bool` | — | `src/hnsw/core_rebuild.rs` |
| `pub async fn search_adaptive( &self, query: &[f32], k: usize, policy: &AdaptiveEfPolicy, ) -> Result<Vec<ScoredDocument>>` | Performs an adaptive search (Ada-ef) on the HNSW index, dynamically adjusting search depth `ef`. | `src/hnsw/core_search.rs` |
| `pub async fn search_adaptive_filtered( &self, query: &[f32], k: usize, policy: &AdaptiveEfPolicy, filter: Option<&(dyn Fn(DocId) -> bool + Send + Sync)>, ) -> Result<Vec<ScoredDocument>>` | Performs an adaptive search (Ada-ef) with custom document filter predicate. | `src/hnsw/core_search.rs` |
| `pub async fn search_adaptive_with_stats( &self, query: &[f32], k: usize, policy: &AdaptiveEfPolicy, filter: Option<&(dyn Fn(DocId) -> bool + Send + Sync)>, ) -> Result<(Vec<ScoredDocument>, AdaptiveEfStats)>` | Performs an adaptive search (Ada-ef) returning scored documents along with execution statistics. | `src/hnsw/core_search.rs` |
| `pub fn calibrate( batch: &[&[f32]], quantizer: &crate::quantize::ScalarQuantizer, metric: contextra_core::DistanceMetric, ) -> Self` | Calibrates quantization error statistics across a representative batch of vectors. | `src/hnsw/sq8_bias.rs` |
| `pub fn new(mean_bias: f32, variance_bias: f32, sample_count: u64) -> Self` | Creates a new `Sq8Bias` instance with calibrated parameters. | `src/hnsw/sq8_bias.rs` |
| `pub fn all_doc_ids_and_layers(&self) -> Vec<(DocId, usize)>` | Returns all non-deleted document IDs and their assigned HNSW max layers. | `src/hnsw/types.rs` |
| `pub fn all_doc_ids_from_map(&self) -> Vec<DocId>` | — | `src/hnsw/types.rs` |
| `pub fn check_and_trigger_partial_rebuild(&self)` | — | `src/hnsw/types.rs` |
| `pub fn check_connectivity(&self) -> contextra_core::Result<()>` | — | `src/hnsw/types.rs` |
| `pub fn compact_seq_log(&self, min_active_seqno: u64)` | — | `src/hnsw/types.rs` |
| `pub fn connectivity_score(&self) -> f64` | — | `src/hnsw/types.rs` |
| `pub fn deleted_ratio(&self) -> f64` | — | `src/hnsw/types.rs` |
| `pub fn get_entry_point(&self) -> Option<usize>` | — | `src/hnsw/types.rs` |
| `pub fn get_ram_entry_point(&self) -> Option<usize>` | — | `src/hnsw/types.rs` |
| `pub fn get_ram_node_connections(&self, ram_idx: usize, layer: usize, m: usize) -> Vec<u32>` | — | `src/hnsw/types.rs` |
| `pub fn is_rebuild_required(&self) -> bool` | — | `src/hnsw/types.rs` |
| `pub async fn load_mmap(&self, path: impl AsRef<std::path::Path> + Send) -> Result<()>` | — | `src/hnsw/types.rs` |
| `pub fn metrics_sink(&self) -> std::sync::Arc<dyn contextra_ports::MetricsSink>` | Returns the metrics sink. | `src/hnsw/types.rs` |
| `pub fn metrics_sink(&self) -> std::sync::Arc<dyn contextra_ports::MetricsSink>` | Returns the metrics sink. | `src/hnsw/types.rs` |
| `pub fn new(config: HnswConfig) -> Self` | — | `src/hnsw/types.rs` |
| `pub fn pin_snapshot(&self, seq_no: u64)` | — | `src/hnsw/types.rs` |
| `pub fn quantizer(&self) -> Option<crate::quantize::ScalarQuantizer>` | Returns a snapshot clone of the current quantizer if trained. | `src/hnsw/types.rs` |
| `pub async fn rebuild(&self) -> Result<()>` | — | `src/hnsw/types.rs` |
| `pub fn rebuild_count(&self) -> u64` | — | `src/hnsw/types.rs` |
| `pub async fn rebuild_region(&self, region_node_ids: Vec<u64>) -> Result<()>` | — | `src/hnsw/types.rs` |
| `pub fn rebuild_status(&self) -> RebuildStatus` | — | `src/hnsw/types.rs` |
| `pub async fn save(&self, path: impl AsRef<std::path::Path>) -> Result<()>` | — | `src/hnsw/types.rs` |
| `pub fn set_entry_point(&self, ep: Option<usize>)` | — | `src/hnsw/types.rs` |
| `pub fn set_fault_injection_insert_target(&self, target: u64)` | Sets the target insertion count for simulating compute_insert fault injection in tests. | `src/hnsw/types.rs` |
| `pub fn set_layer_seed(&self, seed: u64)` | Sets the random seed used for HNSW layer selection during insertions. | `src/hnsw/types.rs` |
| `pub fn set_layer_seed(&self, seed: u64)` | Sets the random seed used for HNSW layer selection during insertions. | `src/hnsw/types.rs` |
| `pub fn set_metrics_sink(&self, sink: std::sync::Arc<dyn contextra_ports::MetricsSink>)` | Sets the metrics sink. | `src/hnsw/types.rs` |
| `pub fn set_metrics_sink(&self, sink: std::sync::Arc<dyn contextra_ports::MetricsSink>)` | Sets the metrics sink. | `src/hnsw/types.rs` |
| `pub fn set_ram_entry_point(&self, ep: Option<usize>)` | — | `src/hnsw/types.rs` |
| `pub fn sq8_bias(&self) -> Sq8Bias` | Returns the calibrated SQ8 quantization bias statistics. | `src/hnsw/types.rs` |
| `pub fn trigger_rebuild_async(&self)` | — | `src/hnsw/types.rs` |
| `pub fn try_new(config: HnswConfig) -> Result<Self>` | Creates a new HNSW index, validating configuration upfront. Uses an internal, non-cryptographic, reproducible default RNG seeded with [`DEF… | `src/hnsw/types.rs` |
| `pub fn try_new_with_rng( config: HnswConfig, rng: std::sync::Arc<dyn contextra_ports::Rng>, ) -> Result<Self>` | Creates a new HNSW index with an explicit random number generator for deterministic testing or custom seeding. | `src/hnsw/types.rs` |
| `pub fn unpin_snapshot(&self, seq_no: u64)` | — | `src/hnsw/types.rs` |
| `pub fn visited_dead_nodes(&self) -> u64` | — | `src/hnsw/types.rs` |
| `pub async fn wait_for_rebuild(&self) -> bool` | — | `src/hnsw/types.rs` |
| `pub async fn wait_for_rebuild_with_timeout(&self, timeout: std::time::Duration) -> bool` | — | `src/hnsw/types.rs` |
| `pub fn with_metrics_sink(self, sink: std::sync::Arc<dyn contextra_ports::MetricsSink>) -> Self` | Builder method to set the metrics sink. | `src/hnsw/types.rs` |
| `pub fn find_oversaturated_regions( &self, tombstone_map: &HashMap<u64, bool>, // node_id → is_tombstoned ) -> Vec<u64>` | Berechnet S_local für alle Hot-Path-Regionen. Gibt Knoten-IDs zurück, für die S_local > θ_c gilt. | `src/partial_rebuild.rs` |
| `pub fn hot_path_nodes(&self) -> impl Iterator<Item = u64> + '_` | Gibt die N zuletzt besuchten eindeutigen Knoten-IDs zurück. | `src/partial_rebuild.rs` |
| `pub fn new(config: PartialRebuildConfig) -> Self` | Erstellt einen neuen TraversalTracker mit der gegebenen Konfiguration. | `src/partial_rebuild.rs` |
| `pub fn record_traversal(&mut self, visited_node_ids: Vec<u64>)` | Trägt die in einem ef_search-Lauf besuchten Node-IDs ein. | `src/partial_rebuild.rs` |
| `pub fn should_trigger_partial_rebuild( tracker: &TraversalTracker, tombstone_map: &HashMap<u64, bool>, global_tombstone_ratio: f32, config: &PartialRebuildConfig, ) -> Option<Vec<u64>>` | Entscheidet ob ein lokaler Partial-Rebuild ausgelöst werden soll. Returns `Some(hot_node_ids)` wenn Partial-Rebuild ausgelöst werden soll,… | `src/partial_rebuild.rs` |
| `pub fn connections_offset(&self) -> u64` | — | `src/persistence/header.rs` |
| `pub fn dimension(&self) -> u32` | — | `src/persistence/header.rs` |
| `pub fn entry_point(&self) -> i64` | — | `src/persistence/header.rs` |
| `pub fn is_quantized(&self) -> bool` | — | `src/persistence/header.rs` |
| `pub fn last_tx_id(&self) -> u64` | — | `src/persistence/header.rs` |
| `pub fn m(&self) -> u32` | — | `src/persistence/header.rs` |
| `pub fn magic(&self) -> u32` | — | `src/persistence/header.rs` |
| `pub fn metric(&self) -> u8` | — | `src/persistence/header.rs` |
| `pub fn new( dimension: u32, m: u32, metric: u8, quantized: u8, q_min: f32, q_max: f32, node_count: u64, entry_point: i64, nodes_offset: u64, connections_offset: u64, last_tx_id: u64, ) -> Self` | — | `src/persistence/header.rs` |
| `pub fn new_v2( dimension: u32, m: u32, metric: u8, quantized: u8, q_min: f32, q_max: f32, node_count: u64, entry_point: i64, nodes_offset: u64, connections_offset: u64, last_tx_id: u64, quant_calibration_offset: u64, quant_calibration_len: u32, ) -> Self` | — | `src/persistence/header.rs` |
| `pub fn new_v2_with_bias( dimension: u32, m: u32, metric: u8, quantized: u8, q_min: f32, q_max: f32, node_count: u64, entry_point: i64, nodes_offset: u64, connections_offset: u64, last_tx_id: u64, quant_calibration_offset: u64, quant_calibration_len: u32, sq8_…` | — | `src/persistence/header.rs` |
| `pub fn node_count(&self) -> u64` | — | `src/persistence/header.rs` |
| `pub fn nodes_offset(&self) -> u64` | — | `src/persistence/header.rs` |
| `pub fn q_max(&self) -> f32` | — | `src/persistence/header.rs` |
| `pub fn q_min(&self) -> f32` | — | `src/persistence/header.rs` |
| `pub fn q_range(&self) -> (f32, f32)` | — | `src/persistence/header.rs` |
| `pub fn quant_calibration_len(&self) -> u32` | — | `src/persistence/header.rs` |
| `pub fn quant_calibration_offset(&self) -> u64` | — | `src/persistence/header.rs` |
| `pub fn quantized(&self) -> u8` | — | `src/persistence/header.rs` |
| `pub fn set_connections_offset(&mut self, offset: u64)` | — | `src/persistence/header.rs` |
| `pub fn sq8_bias(&self) -> Sq8Bias` | — | `src/persistence/header.rs` |
| `pub fn sq8_bias_mean(&self) -> f32` | — | `src/persistence/header.rs` |
| `pub fn sq8_bias_variance(&self) -> f32` | — | `src/persistence/header.rs` |
| `pub fn to_bytes(&self) -> [u8` | — | `src/persistence/header.rs` |
| `pub fn try_from_bytes(bytes: &[u8]) -> Result<Self>` | — | `src/persistence/header.rs` |
| `pub fn version(&self) -> u16` | — | `src/persistence/header.rs` |
| `pub fn get_connections(&self, record: &NodeRecord, layer: usize) -> Result<Vec<u32>>` | — | `src/persistence/mmap.rs` |
| `pub fn get_node_record(&self, index: usize) -> Result<NodeRecord>` | — | `src/persistence/mmap.rs` |
| `pub fn get_vector(&self, record: &NodeRecord) -> Result<&[u8]>` | — | `src/persistence/mmap.rs` |
| `pub fn open(path: impl AsRef<std::path::Path>) -> Result<Self>` | — | `src/persistence/mmap.rs` |
| `pub async fn open_async(path: impl AsRef<std::path::Path> + Send) -> Result<Self>` | Opens an HNSW file without requiring external runtime blocking task spawns. | `src/persistence/mmap.rs` |
| `pub fn from_bytes(bytes: &[u8]) -> Result<Self>` | — | `src/persistence/node.rs` |
| `pub fn to_bytes(&self) -> Vec<u8>` | — | `src/persistence/node.rs` |
| `pub fn asymmetric_dist( &self, query: &[f32], quantized: &[u8], metric: DistanceMetric, ) -> contextra_core::Result<f32>` | Computes the asymmetric distance between an exact query and a quantized vector. Optimized for zero allocations via inline dequantization. | `src/quantize/mod.rs` |
| `pub fn calculate_vector_drift(&self, vector: &[f32]) -> f32` | Calculates quantization drift for a single vector as the fraction of dimensions falling outside `[mins[i], maxes[i]]`. This is a **pure cal… | `src/quantize/mod.rs` |
| `pub fn check_drift(&self, vector: &[f32]) -> f32` | Legacy pure drift calculation wrapper for [`calculate_vector_drift`](Self::calculate_vector_drift). # Deprecated Renamed to [`calculate_vec… | `src/quantize/mod.rs` |
| `pub fn dequantize(&self, vector: &[u8]) -> contextra_core::Result<Vec<f32>>` | Dequantizes a `u8` vector back to `f32`. | `src/quantize/mod.rs` |
| `pub fn dimension(&self) -> usize` | Returns the target dimension of the quantizer. | `src/quantize/mod.rs` |
| `pub fn drift_ratio(&self) -> f32` | Returns the fraction of quantized queries that contained values outside the trained min/max range. This is a **stateful metric** reflecting… | `src/quantize/mod.rs` |
| `pub fn inv_scales(&self) -> &[f32]` | Returns a reference to per-dimension inverse scale values. | `src/quantize/mod.rs` |
| `pub fn is_rebuild_required(&self, threshold: f32) -> bool` | Checks if recalibration / index rebuild is required based on cumulative quantization drift ratio. Requires at least 20 queries to avoid fal… | `src/quantize/mod.rs` |
| `pub fn maxes(&self) -> &[f32]` | Returns a reference to per-dimension maximum values. | `src/quantize/mod.rs` |
| `pub fn mins(&self) -> &[f32]` | Returns a reference to per-dimension minimum values. | `src/quantize/mod.rs` |
| `pub fn quantize(&self, vector: &[f32]) -> contextra_core::Result<Vec<u8>>` | Quantizes an `f32` vector to `u8`. | `src/quantize/mod.rs` |
| `pub fn scales(&self) -> &[f32]` | Returns a reference to per-dimension scale values. | `src/quantize/mod.rs` |
| `pub fn symmetric_dist( &self, q1: &[u8], q2: &[u8], metric: DistanceMetric, ) -> contextra_core::Result<f32>` | Computes symmetric (approximate) distance purely in u8. Optimized for zero allocations via inline dequantization. | `src/quantize/mod.rs` |
| `pub fn train(batch: &[&[f32]], dimension: usize) -> Self` | Creates a new ScalarQuantizer trained on a batch of vectors to find per-dimension min/max. For long-lived or growing collections, callers s… | `src/quantize/mod.rs` |
| `pub fn train_with_percentiles( batch: &[&[f32]], dimension: usize, p_low: f32, p_high: f32, ) -> Self` | Creates a new ScalarQuantizer trained on a batch of vectors with custom percentile bounds. | `src/quantize/mod.rs` |
| `pub fn try_train(batch: &[&[f32]], dimension: usize) -> contextra_core::Result<Self>` | Creates a new ScalarQuantizer trained on a batch of vectors to find per-dimension min/max. For long-lived or growing collections, callers s… | `src/quantize/mod.rs` |
| `pub fn try_train_with_percentiles( batch: &[&[f32]], dimension: usize, p_low: f32, p_high: f32, ) -> contextra_core::Result<Self>` | Safely creates a new ScalarQuantizer trained on a batch of vectors using custom percentile clipping bounds. Range `p_low` and `p_high` must… | `src/quantize/mod.rs` |
| `pub fn asymmetric_distance(&self, query: &[f32], code: &[u8]) -> Result<f32>` | Computes the asymmetric Euclidean distance between an unquantized query vector and a quantized bitcode. # Mathematical Derivation & Bias Co… | `src/quantize_rabitq.rs` |
| `pub fn code_length(&self) -> usize` | Returns the expected byte length of quantized codes (`dimension.div_ceil(8) + 8` bytes). | `src/quantize_rabitq.rs` |
| `pub fn dimension(&self) -> usize` | Returns the vector dimension supported by this quantizer. | `src/quantize_rabitq.rs` |
| `pub fn quantize(&self, vector: &[f32]) -> Result<Vec<u8>>` | Quantizes a single $f32$ vector into a RaBitQ packed bitcode buffer. The returned code consists of: - $\lceil D / 8 \rceil$ bytes of packed… | `src/quantize_rabitq.rs` |
| `pub fn try_train(vectors: &[Vec<f32>], dimension: usize) -> Result<Self>` | Safely trains a `RaBitQQuantizer` for vectors of the specified `dimension`. Generates a deterministic orthogonal matrix $R \in \mathbb{R}^{… | `src/quantize_rabitq.rs` |


---

## `contextra-text`

| Attribut | Wert |
|---|---|
| **Ring** | 0 — Ring 0 — Foundation & Pure Numerics/Types |
| **Status** | Fertig |
| **Zweck** | BM25/BM25F-Volltextindexierung und deutsche Kompositazerlegung. |
| **Unsafe-Code** | Nein (`#![forbid(unsafe_code)]`) |
| **Interne Abhängigkeiten** | `contextra-ports`, `contextra-types` |
| **Externe Abhängigkeiten (Auswahl)** | `bincode`, `bytes`, `itoa`, `parking_lot`, `regex`, `serde`, `serde_json`, `thiserror`, `tracing`, `unicode-segmentation` |

### Modul- und Re-Export-Oberfläche (lib.rs)

```rust
pub mod bm25;
pub mod domain;
pub mod inverted;
pub mod morphology;
pub mod posting_list;
pub mod stream;
pub mod tokenizer;
pub mod wand;
pub use bm25::{score_term_bm25f, FieldId, FieldWeight, BM25, BM25F};
pub use domain::{DomainVocabulary, LegalDomainVocabulary, MedicalDomainVocabulary};
pub use inverted::{BM25MorphIndex, InvertedIndex, Language};
pub use morphology::{normalize_umlauts, GermanCompoundSplitter, MorphologicalTokenizer};
pub use posting_list::{Posting, PostingList, ResidentPostingIndex};
pub use stream::{Bm25CandidateStream, DEFAULT_STREAM_BATCH_SIZE};
pub use tokenizer::{DefaultTokenizer, GermanMorphTokenizer, Tokenizer};
```

### Feature-Flags

| Flag | Aktiviert |
|---|---|
| `default` | `[]` |
| `bm25f` | `[]` |
| `docid-128` | `["contextra-types/docid-128"]` |

### Schnittstellen — Public Traits (3)

**`DomainVocabulary`** — *Trait für domänenspezifische Vokabularpakete.* (`src/domain/mod.rs`)

```rust
pub trait DomainVocabulary : Send + Sync {
/// Domänenspezifische Komposita-Bausteine (Ergänzung, kein Ersatz für das KMU-Basiswörterbuch).
    fn compound_stems(&self) -> &'static [&'static str];
    /// Fachbegriffe, die NICHT als Stopwort behandelt werden dürfen, selbst wenn sie kurz/häufig sind
    /// (z. B. "Klage", "Akte" im juristischen Kontext).
    fn protected_terms(&self) -> &'static [&'static str];
    /// Kurzname für Logging/Konfiguration, z. B. "legal_de", "medical_de".
    fn domain_id(&self) -> &'static str;
}
```

**`MorphologicalTokenizer`** — *Trait for morphological tokenization. Decomposes compound words into constituent morphemes. # Input Contract Input tokens MUST be lowercased before passing to `decompose`. Passing mixed-case or uppercase tokens causes silent dictionary misses and returns the…* (`src/morphology/compound_splitter.rs`)

```rust
pub trait MorphologicalTokenizer : Send + Sync {
/// Decomposes a token into its morphological components.
    fn decompose<'a>(&self, token: &'a str) -> Vec<&'a str>;

    /// Returns the language code of this tokenizer (e.g. "de", "en").
    fn language(&self) -> &str;
}
```

**`Tokenizer`** — *Tokenizer trait for different language-specific or morphological strategies.* (`src/tokenizer.rs`)

```rust
pub trait Tokenizer : Send + Sync {
/// Tokenizes text into lowercase words and filters stopwords.
    fn tokenize(&self, text: &str) -> Vec<String>;
}
```

### Öffentliche Datentypen — Structs (23)

| Struct | Felder / Form | Beschreibung | Datei |
|---|---|---|---|
| `BM25` | `{ pub k1: f32, pub b: f32, }` | BM25 scoring model parameters. Recommendations (Robertson-Walker defaults): - `k1`: Term frequency saturation control parameter. Recommended default… | `src/bm25.rs` |
| `BM25F` | `{ pub k1: f32, pub field_weights: Vec<FieldWeight>, cached_weights: Vec<(FieldId, f32, f32)>, }` | BM25F field-weighted scoring model configuration (§11.3). **Opt-in Feature Note**: BM25F field-weighted scoring requires the explicit Cargo feature `… | `src/bm25.rs` |
| `BM25MorphIndex` | `{ inner: InvertedIndex<S>, tokenizer: Arc<dyn MorphologicalTokenizer>, }` | An inverted index with morphological optimization. | `src/inverted/morph_index.rs` |
| `Bm25CandidateStream` | `{ index: &'a InvertedIndex<S>, query: String, max_seq: Option<u64>, batch_size: usize, buffer: VecDeque<(DocId, f32)>, seen: HashSet<DocId>, fetched_k: usize, yi… }` | On-demand loading async pull-stream for BM25 search candidates. # Candidate Ordering & Determinism Results are delivered in deterministic order: scor… | `src/stream.rs` |
| `Bm25Scorer` | `{ index: InvertedIndex<S>, }` | Evaluates keyword weights and applies standard BM25 logic. | `src/lib.rs` |
| `BoundedTopK` | `{ heap: BinaryHeap<Bm25Candidate>, capacity: usize, }` | — | `src/wand.rs` |
| `DefaultTokenizer` | `(tuple/unit)` | Default tokenizer using Unicode word boundaries and generic stopwords. | `src/tokenizer.rs` |
| `FieldWeight` | `{ pub field_id: FieldId, pub weight: f32, pub b: f32, }` | Field weighting configuration for BM25F scoring. | `src/bm25.rs` |
| `GermanCompoundSplitter` | `{ /// Minimum component length for splitting. min_component_len: usize, /// Trie data structure for fast prefix checking and dictionary matching. trie: Trie, }` | German compound word splitter (*Komposita-Zerleger*). Uses dictionary-based recursive segmentation powered by Dynamic Programming (DP). # Architectur… | `src/morphology/compound_splitter.rs` |
| `GermanMorphTokenizer` | `{ splitter: Arc<crate::morphology::GermanCompoundSplitter>, }` | German tokenizer with morphological compound splitting. | `src/tokenizer.rs` |
| `InvertedIndex` | `{ storage: Arc<S>, prefix: Vec<u8>, tokenizer: Arc<dyn Tokenizer>, pub(crate) total_docs: Arc<AtomicU64>, pub(crate) total_tokens: Arc<AtomicU64>, pub(crate) avg… }` | An inverted index tied to a specific collection namespace. # Concurrency & Lock Hierarchy: 1. `commit_lock` (`parking_lot::Mutex`): Ensures single-wr… | `src/inverted/index_struct.rs` |
| `LegalDomainVocabulary` | `(tuple/unit)` | Deutsches Juristisches Fachvokabular. | `src/domain/legal_de.rs` |
| `MedicalDomainVocabulary` | `(tuple/unit)` | Deutsches Medizinisches Fachvokabular. | `src/domain/medical_de.rs` |
| `PassthroughTokenizer` | `{ lang: String, }` | Passthrough tokenizer for languages without compound words. | `src/morphology/passthrough.rs` |
| `Posting` | `{ pub doc_id: DocIdRaw, pub tf: u32, pub doc_len: u32, }` | Compact representation of a single posting in a posting list. Packed with 8-byte alignment (16 bytes total) for optimal CPU cache alignment. | `src/posting_list.rs` |
| `PostingBlockInfo` | `{ /// Upper bound doc_id in this block (for fast skip). pub max_doc_id: DocIdRaw, /// Maximum term frequency in this block. pub max_tf: u32, /// Minimum document… }` | Block-Max metadata for a chunk of postings (typically 64 postings). | `src/posting_list.rs` |
| `PostingList` | `{ postings: Vec<Posting>, blocks: Vec<PostingBlockInfo>, }` | A contiguous, sorted sequence of postings for a specific term. | `src/posting_list.rs` |
| `ResidentPostingIndex` | `{ index: RwLock<HashMap<String, Arc<PostingList>>>, }` | In-Memory resident index mapping terms to their posting lists. Uses RCU (Read-Copy-Update) via `Arc<PostingList>` for lock-free read access under `Rw… | `src/posting_list.rs` |
| `TextIndexMetadata` | `{ pub total_docs: u64, pub total_tokens: u64, pub avg_doc_len_x1000: u64, // Fixed-point for BM25 caching (FIND-TXT-004) }` | Consolidated metadata for the text index. | `src/inverted/types.rs` |
| `TokenReductionMetrics` | `{ /// Number of original tokens before morphological decomposition. pub original_tokens: usize, /// Number of tokens after decomposition. pub decomposed_tokens:… }` | Metrics for measuring token reduction effectiveness. | `src/morphology/metrics.rs` |
| `Trie` | `{ root: TrieNode, }` | A prefix trie data structure for dictionary lookup. | `src/morphology/trie.rs` |
| `TrieNode` | `{ /// Indicates if a word ends at this node. pub is_terminal: bool, /// Child nodes keyed by character. pub children: std::collections::HashMap<char, TrieNode>, }` | A prefix trie node for fast lookup and prefix matching of German dictionary words. | `src/morphology/trie.rs` |
| `WandSearchResult` | `{ pub results: Vec<(DocId, f32)>, pub evaluated_docs_count: usize, }` | Result from Block-Max WAND search containing top-k items and execution metrics. | `src/wand.rs` |

### Öffentliche Datentypen — Enums (1)

| Enum | Varianten | Beschreibung | Datei |
|---|---|---|---|
| `Language` | `/// English tokenizer (default, whitespace-based + stopword filtering). #[default] English, /// German morphology tokenizer (compound splitting + umlaut normalization). German, /// Custom tokenizer —…` | Supported tokenizer languages for BM25 text indexing. Controls which tokenizer is used for document indexing and query processing. Use `Language::fro… | `src/inverted/types.rs` |

### Typ-Aliase (3)

| Alias | Definition | Beschreibung |
|---|---|---|
| `DocIdRaw` | `type DocIdRaw= u64` | — |
| `DocIdRaw` | `type DocIdRaw= u128` | — |
| `FieldId` | `type FieldId= u32` | Field identifier for field-weighted BM25F scoring. |

### Konstanten (7)

| Konstante | Typ | Beschreibung |
|---|---|---|
| `BLOCK_SIZE` | `usize` | Block size for Block-Max WAND decomposition. |
| `BM25_B` | `f32` | Default b parameter for BM25 document length normalization penalty tuning. |
| `BM25_K1` | `f32` | Default k1 parameter for BM25 term frequency saturation scaling. |
| `DEFAULT_STREAM_BATCH_SIZE` | `usize` | Default batch size for candidate retrieval. |
| `MAX_STAGED_TRANSACTIONS` | `usize` | Maximum allowed staged uncommitted transaction statistics changes (10,000). |
| `MAX_TEXT_BYTES` | `usize` | Maximum allowed document text size in bytes (10 MB). |
| `POSTING_LIST_V2_MAGIC` | `&[u8` | — |

### Öffentliche Funktionen (73)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn new(k1: f32, b: f32) -> Result<Self>` | Creates a new `BM25` configuration after validating `k1` and `b`. # Errors Returns `ContextraError::InvalidInput` if `k1 < 0.0` or `k1` is… | `src/bm25.rs` |
| `pub fn new(field_id: FieldId, weight: f32, b: f32) -> Result<Self>` | Creates a new `FieldWeight` validation configuration. # Errors Returns `ContextraError::InvalidInput` if `weight < 0.0` or `weight` is NaN,… | `src/bm25.rs` |
| `pub fn new(k1: f32, field_weights: Vec<FieldWeight>) -> Result<Self>` | Creates a new `BM25F` scoring configuration. # Errors Returns `ContextraError::InvalidInput` if `k1 < 0.0` or `k1` is NaN. | `src/bm25.rs` |
| `pub fn score_term(&self, tf: u32, doc_len: u32, avg_doc_len: f32, df: u32, n: u32) -> f32` | Calculates the BM25 score for a single term using this instance's `k1` and `b`. | `src/bm25.rs` |
| `pub fn score_term(tf: u32, doc_len: u32, avg_doc_len: f32, df: u32, n: u32) -> f32` | Calculates the BM25 score for a single term in a document using default `BM25_K1` and `BM25_B`. # Arguments * `tf` - Term frequency in the… | `src/bm25.rs` |
| `pub fn score_term( &self, field_term_frequencies: &[(FieldId, u32, u32, f32)], df: u32, n: u32, ) -> f32` | Calculates the BM25F score for a single term across multiple fields. | `src/bm25.rs` |
| `pub fn score_term_bm25f( field_term_frequencies: &[(FieldId, u32, u32, f32)], field_weights: &[(FieldId, f32, f32)], k1: f32, df: u32, n: u32, ) -> f32` | Calculates the field-weighted BM25F score for a single term across document fields. Implements the Robertson & Zaragoza (2004) field-weight… | `src/bm25.rs` |
| `pub fn score_term_with_params( tf: u32, doc_len: u32, avg_doc_len: f32, df: u32, n: u32, k1: f32, b: f32, ) -> f32` | Calculates the BM25 score for a single term in a document using custom `k1` and `b` parameters. | `src/bm25.rs` |
| `pub fn get_legal_compound_stems() -> &'static [&'static str]` | Exportierte Hilfsfunktionen für direkten Slice-Zugriff. | `src/domain/legal_de.rs` |
| `pub fn get_legal_protected_terms() -> &'static [&'static str]` | — | `src/domain/legal_de.rs` |
| `pub fn get_medical_compound_stems() -> &'static [&'static str]` | Exportierte Hilfsfunktionen für direkten Slice-Zugriff. | `src/domain/medical_de.rs` |
| `pub fn get_medical_protected_terms() -> &'static [&'static str]` | — | `src/domain/medical_de.rs` |
| `pub async fn delete_document(&self, tx: TxId, doc_id: DocId) -> Result<()>` | Deletes a document from the index. | `src/inverted/index_struct.rs` |
| `pub async fn load_stats(&self) -> Result<()>` | Loads index statistics from storage into the cache. | `src/inverted/index_struct.rs` |
| `pub fn new(storage: Arc<S>, namespace: &str) -> Self` | Creates a new InvertedIndex with English tokenizer (default). | `src/inverted/index_struct.rs` |
| `pub fn new_with_language(storage: Arc<S>, namespace: &str, language: Language) -> Self` | Creates a new InvertedIndex with explicit language configuration. # Language Selection Use `Language::from_iso("de")` for German, `Language… | `src/inverted/index_struct.rs` |
| `pub async fn resolve_tombstones(&self, tx: TxId) -> Result<u64>` | For every document with a `tbs:{doc_id}` marker this method reads the *current* forward index and removes posting-list entries for terms th… | `src/inverted/index_struct.rs` |
| `pub async fn search_bm25( &self, query: &str, k: usize, max_seq: Option<u64>, ) -> Result<Vec<(DocId, f32)>>` | Searches the inverted index using BM25. | `src/inverted/index_struct.rs` |
| `pub async fn search_bm25_at( &self, query: &str, k: usize, max_seq: Option<u64>, ) -> Result<Vec<(DocId, f32)>>` | Searches the inverted index using BM25 at a specific snapshot version. | `src/inverted/index_struct.rs` |
| `pub async fn upsert_document(&self, tx: TxId, doc_id: DocId, text: &str) -> Result<()>` | — | `src/inverted/index_struct.rs` |
| `pub fn with_tokenizer(mut self, tokenizer: Arc<dyn Tokenizer>) -> Self` | Sets a custom tokenizer for the inverted index. | `src/inverted/index_struct.rs` |
| `pub fn new( storage: Arc<S>, namespace: &str, tokenizer: Arc<dyn MorphologicalTokenizer>, ) -> Self` | — | `src/inverted/morph_index.rs` |
| `pub fn tokenizer(&self) -> &dyn MorphologicalTokenizer` | — | `src/inverted/morph_index.rs` |
| `pub fn from_iso(code: &str) -> Self` | Parses an ISO-639-1 language code ("de", "de-AT", "en", "en-US"). Only the primary subtag (before the first '-') is evaluated. Unknown code… | `src/inverted/types.rs` |
| `pub fn new(storage: std::sync::Arc<S>, namespace: &str) -> Self` | — | `src/lib.rs` |
| `pub fn extend_vocabulary(&mut self, words: impl IntoIterator<Item = impl AsRef<str>>)` | Extends the splitter's dictionary with additional terms. | `src/morphology/compound_splitter.rs` |
| `pub fn min_component_len(&self) -> usize` | Returns the minimum component length. | `src/morphology/compound_splitter.rs` |
| `pub fn new() -> Self` | Creates a new German compound splitter loaded with the embedded German vocabulary. | `src/morphology/compound_splitter.rs` |
| `pub fn new_with_all_domains() -> Self` | Creates a splitter loaded with default German vocabulary and all built-in domain vocabularies (Legal & Medical). | `src/morphology/compound_splitter.rs` |
| `pub fn with_dictionary(min_len: usize, custom_words: HashSet<String>) -> Self` | Creates a splitter with custom minimum component length and custom dictionary set. | `src/morphology/compound_splitter.rs` |
| `pub fn with_domains(min_len: usize, domains: &[&dyn DomainVocabulary]) -> Self` | Creates a splitter with the default vocabulary plus compound stems and protected terms from specific domain packages. | `src/morphology/compound_splitter.rs` |
| `pub fn with_min_length(min_len: usize) -> Self` | Creates a splitter with custom minimum component length and default embedded vocabulary. | `src/morphology/compound_splitter.rs` |
| `pub fn expansion_ratio(&self) -> f32` | Computes the token reduction ratio. A ratio > 1.0 means decomposition increased tokens (expected for compounds). Target: > 20% token-count… | `src/morphology/metrics.rs` |
| `pub fn new(lang: impl Into<String>) -> Self` | Creates a passthrough tokenizer for the given language. | `src/morphology/passthrough.rs` |
| `pub fn get_german_stopwords() -> &'static HashSet<String>` | Returns the static German stopword list. Erweitert um hochfrequente deutsche Funktionswörter gemäß DACH-KMU-Domänenanalyse, 2026-09-01. | `src/morphology/stopwords.rs` |
| `pub fn is_german_stopword(word: &str) -> bool` | Checks if a normalized lowercased word is a German stopword. | `src/morphology/stopwords.rs` |
| `pub fn normalize_umlauts(input: &str) -> String` | Normalisiert deutsche Umlaute für robusten Suchabgleich. # Example ``` use contextra_text::morphology::{normalize_umlauts, GermanCompoundSp… | `src/morphology/stopwords.rs` |
| `pub fn contains(&self, word: &str) -> bool` | Checks if a word exists in the Trie. | `src/morphology/trie.rs` |
| `pub fn insert(&mut self, word: &str)` | Inserts a word into the Trie. | `src/morphology/trie.rs` |
| `pub fn new() -> Self` | Creates a new empty Trie. | `src/morphology/trie.rs` |
| `pub fn starts_with(&self, prefix: &str) -> bool` | Checks if any word in the Trie starts with the given prefix. | `src/morphology/trie.rs` |
| `pub fn as_slice(&self) -> &[Posting]` | Returns a slice of postings. | `src/posting_list.rs` |
| `pub fn blocks(&self) -> &[PostingBlockInfo]` | Returns the block-max metadata blocks. | `src/posting_list.rs` |
| `pub fn clear(&self)` | Clears the in-memory cache (e.g., on full re-sync). | `src/posting_list.rs` |
| `pub fn decode_compact(bytes: &[u8]) -> Result<Self, ContextraError>` | Decodes a posting list from compact Delta-Varint binary format, with legacy bincode V1 fallback. | `src/posting_list.rs` |
| `pub fn doc_id(&self) -> DocId` | — | `src/posting_list.rs` |
| `pub fn empty() -> Self` | Creates an empty `PostingList`. | `src/posting_list.rs` |
| `pub fn encode_compact(&self) -> Result<Vec<u8>, ContextraError>` | Encodes the posting list into a compact binary format with Delta-Varint encoding. | `src/posting_list.rs` |
| `pub fn get(&self, term: &str) -> Option<Arc<PostingList>>` | Retrieves an Arc to the PostingList for a given term if cached. | `src/posting_list.rs` |
| `pub fn insert_list(&self, term: String, list: PostingList)` | Inserts or replaces a PostingList for a given term. | `src/posting_list.rs` |
| `pub fn is_empty(&self) -> bool` | Returns true if the posting list is empty. | `src/posting_list.rs` |
| `pub fn len(&self) -> usize` | Returns the number of postings. | `src/posting_list.rs` |
| `pub fn new(doc_id: DocId, tf: u32, doc_len: u32) -> Self` | — | `src/posting_list.rs` |
| `pub fn new(mut postings: Vec<Posting>) -> Self` | Creates a new `PostingList` from a vector of postings, ensuring sorted order by DocId. | `src/posting_list.rs` |
| `pub fn new() -> Self` | — | `src/posting_list.rs` |
| `pub fn remove(&self, doc_id: DocId) -> Self` | Removes a posting for a doc_id using RCU/copy-on-write semantics. | `src/posting_list.rs` |
| `pub fn remove_posting_from_terms(&self, terms: &[String], doc_id: DocId)` | Removes a posting for a doc_id across a list of terms using RCU. | `src/posting_list.rs` |
| `pub fn remove_terms(&self, doc_id: DocId, terms: &[String])` | Removes postings for a given doc_id across specified terms from the in-memory cache (e.g., on transaction rollback). | `src/posting_list.rs` |
| `pub fn upsert(&self, posting: Posting) -> Self` | Inserts or updates a posting for a doc_id using RCU/copy-on-write semantics. | `src/posting_list.rs` |
| `pub fn upsert_posting(&self, term: &str, posting: Posting)` | Upserts a single posting for a term using RCU (Copy-On-Write). | `src/posting_list.rs` |
| `pub fn is_exhausted(&self) -> bool` | Returns whether the candidate stream is exhausted. | `src/stream.rs` |
| `pub fn new( index: &'a InvertedIndex<S>, query: impl Into<String>, max_seq: Option<u64>, ) -> Self` | Creates a new candidate stream for a query and index snapshot. | `src/stream.rs` |
| `pub async fn next_batch(&mut self) -> Result<Vec<(DocId, f32)>>` | Fetches the next batch of candidate documents (up to `batch_size`). Returns an empty `Vec` when exhausted. | `src/stream.rs` |
| `pub async fn next_doc(&mut self) -> Result<Option<DocId>>` | Fetches the next single candidate document ID. Returns `Ok(None)` when exhausted. | `src/stream.rs` |
| `pub fn with_batch_size(mut self, n: usize) -> Self` | Sets a custom batch size (minimum 1). | `src/stream.rs` |
| `pub fn yielded(&self) -> usize` | Returns the total number of items yielded so far. | `src/stream.rs` |
| `pub fn new() -> Self` | — | `src/tokenizer.rs` |
| `pub fn tokenize(text: &str) -> Vec<String>` | Tokenizes text into lowercase words using Unicode word boundaries and filters stopwords. Deprecated: Use `DefaultTokenizer.tokenize()` inst… | `src/tokenizer.rs` |
| `pub async fn block_max_wand_search<S: StorageEngine>( storage: &S, terms: &[String], resident_index: &crate::posting_list::ResidentPostingIndex, prefix_helper: impl Fn(&str) -> Vec<u8>, tombstone_key_helper: impl Fn(DocId, &str) -> Vec<u8>, doc_len_key_helper…` | Executes Block-Max WAND traversal over resident posting lists. | `src/wand.rs` |
| `pub fn into_sorted_vec(self) -> Vec<(DocId, f32)>` | — | `src/wand.rs` |
| `pub fn min_threshold(&self) -> f32` | — | `src/wand.rs` |
| `pub fn new(capacity: usize) -> Self` | — | `src/wand.rs` |
| `pub fn push(&mut self, doc_id: DocId, score: f32)` | — | `src/wand.rs` |


---

## `contextra-graph`

| Attribut | Wert |
|---|---|
| **Ring** | 0 — Ring 0 — Foundation & Pure Numerics/Types |
| **Status** | Fertig |
| **Zweck** | Compressed Sparse Row (CSR) Graph, Forward-Push PPR, Leiden und Hyperkanten. |
| **Unsafe-Code** | Nein (`#![forbid(unsafe_code)]`) |
| **Interne Abhängigkeiten** | `contextra-adapt`, `contextra-ports`, `contextra-types` |
| **Externe Abhängigkeiten (Auswahl)** | `ahash`, `arc-swap`, `bincode`, `blake3`, `parking_lot`, `scc`, `serde`, `serde_json`, `thiserror`, `tracing` |

### Modul- und Re-Export-Oberfläche (lib.rs)

```rust
pub mod apprh;
pub mod arc_slice;
pub mod cascade;
pub mod community;
pub mod consistency_enforcement;
pub mod csr;
pub mod edge_reinforcement;
pub mod edge_reinforcement_buffer;
pub mod entity_extraction;
pub mod error;
pub mod hyperedge;
pub mod hyperedge_suggest;
pub mod path_rag;
pub mod percolation;
pub mod ppr;
pub mod ppr_stream;
pub mod provenance;
pub mod session_dag;
pub mod tl_hfd;
pub use apprh::{ apprh_local, forward_push_apprh, shadow_compare_forward_push_vs_apprh, ApprhError, ApprhFlipGate, ApprhMode, ApprhParams, ApprhSelector, ApprhSelectorConfig, ApprhShadowComparison, DefaultApprhFlipGate, };
pub use arc_slice::ArcSlice;
pub use cascade::{ cascade_invalidate_edges_for_superseded_doc, cascade_invalidate_hyperedges_for_superseded_doc, CascadeInvalidationReport, CascadeReport, CascadeStatus, CascadeTicket, DeletionProof, GraphGarbageCollection, HyperedgeCascadeReport, MAX_HYPEREDGE_CASCADE_FANOUT, };
pub use community::{ detect_communities, CommunityAssignment, CommunityDetectionConfig, StarExpansionIterator, VirtualHyperedgeNode, };
pub use consistency_enforcement::{ ConflictPattern, ConsistencyEnforcer, ContradictionDetector, EdgeAssertion, EdgeId, ExactPredicateConflictDetector, };
pub use csr::CsrGraph;
pub use edge_reinforcement::{ apply_cooccurrence_reinforcement, apply_traversal_reinforcement, apply_weight_normalization, compute_edge_weight, EdgeReinforcementConfig, };
pub use edge_reinforcement_buffer::{CooccurrenceSignal, EdgeReinforcementBuffer, TraversalSignal};
pub use entity_extraction::{ ingest_extracted_relations, EntityExtractor, ExtractedRelation, RuleBasedEntityExtractor, };
pub use error::GraphMutationError;
pub use hyperedge::{ sort_dedup_entities, star_weight, ConsolidationNodesGuard, HyperEdge, HyperEdgeId, HyperEdgeView, RoleBinding, RoleId, RoleInterner, HYPEREDGE_BY_ENTITY_PREFIX, HYPEREDGE_PREFIX, };
pub use hyperedge_suggest::{ compute_co_occurrence_candidates, validate_candidates_with_llm, HyperEdgeCandidate, HyperedgeSuggestError, ValidatedHyperEdgeCandidate, MAX_RELATE_PARTICIPANTS, };
pub use path_rag::{EntityId, GraphPath, PathGraph, PathRAGConfig, PathRAGEngine};
pub use path_rag::{KPathConfig, KPathDiffusion, KPathResult};
pub use percolation::{ compute_percolation_health, find_rebonding_candidates, should_trigger_rebonding, PercolationConfig, };
pub use ppr::compute_ppr_with_apprh_selector;
pub use ppr::{DeletedView, PprContext};
pub use ppr_stream::{PprCandidateStream, DEFAULT_PPR_STREAM_BATCH_SIZE};
pub use provenance::{DocEdgeIndex, EdgeProvenance};
pub use session_dag::{ AgentStateNode, DagEdge, NodeIdx, NodesGuard, NodesWriteGuard, SessionBranchTree, };
pub use tl_hfd::{ shadow_compare_forward_push_vs_tl_hfd, tl_hfd_local, DefaultFlipGate, ShadowComparison, ShadowDiscrepancyReport, TlHfdError, TlHfdFlipGate, TlHfdParams, MIN_AGREEMENT_THRESHOLD, MIN_SHADOW_SAMPLES, };
```

### Feature-Flags

| Flag | Aktiviert |
|---|---|
| `default` | `[]` |
| `docid-128` | `["contextra-types/docid-128"]` |
| `graph-connectivity-health` | `[]` |
| `edge-reinforcement-learning` | `[]` |
| `k-path-diffusion` | `[]` |
| `apprh-diffusion` | `[]` |
| `physio-percolation` | `["graph-connectivity-health"]` |
| `physio-synaptic-edges` | `["edge-reinforcement-learning"]` |

### Schnittstellen — Public Traits (8)

**`ApprhFlipGate`** — *Formal default-flip gate trait evaluating whether APPRH can replace ForwardPush as default.* (`src/apprh/shadow.rs`)

```rust
pub trait ApprhFlipGate {
/// Evaluates whether APPRH meets default flip criteria.
    fn should_flip(&self, report: &ApprhShadowComparison) -> bool;
}
```

**`ContradictionDetector`** — *Trait für modulare Widerspruchserkennungs-Strategien.* (`src/consistency_enforcement.rs`)

```rust
pub trait ContradictionDetector {
/// Prüft, ob zwei Aussagen im Widerspruch zueinander stehen.
    fn conflicts(&self, a: &EdgeAssertion, b: &EdgeAssertion) -> bool;
}
```

**`DefaultFlipGate`** — *Formal default-flip gate trait evaluating whether TL-HFD can replace ForwardPush as default.* (`src/tl_hfd/shadow.rs`)

```rust
pub trait DefaultFlipGate {
/// Returns `true` iff:
    /// 1. `sample_count >= MIN_SHADOW_SAMPLES`
    /// 2. `mean_topk_jaccard >= MIN_AGREEMENT_THRESHOLD`
    /// 3. `p99_latency_delta_us <= 0.0`
    fn should_flip(&self, report: &ShadowDiscrepancyReport) -> bool {
        report.sample_count >= MIN_SHADOW_SAMPLES
            && report.mean_topk_jaccard >= MIN_AGREEMENT_THRESHOLD
            && report.p99_latency_delta_us <= 0.0
    }
}
```

**`EntityExtractor`** — *Trait für Entitäts- und Relations-Extraktionsstufen.* (`src/entity_extraction.rs`)

```rust
pub trait EntityExtractor : Send + Sync {
/// Extrahiert Relationen aus dem gegebenen Freitext.
    fn extract(&self, text: &str) -> Vec<ExtractedRelation>;
}
```

**`GraphGarbageCollection`** — *Garbage collection trait for sweeping orphaned hyperedge references (Anhang B §B.5.1.7).* (`src/cascade.rs`)

```rust
pub trait GraphGarbageCollection {
/// Sweeps orphaned hyperedges whose participants no longer exist or are fully tombstoned.
    fn sweep_orphans(&self, wal_tx: TxId) -> Result<usize>;
}
```

**`GraphIndexExt`** — *Extension trait for [`contextra_ports::GraphIndex`] providing entity removal functionality.* (`src/lib.rs`)

```rust
pub trait GraphIndexExt : contextra_ports::GraphIndex {
/// Removes an entity node and all its incident (outgoing and incoming) edges from the graph.
    fn remove_entity<'a>(
        &'a self,
        tx: contextra_types::TxId,
        entity: EntityId,
    ) -> contextra_ports::BoxFuture<'a, contextra_types::Result<()>>;
}
```

**`KPathDiffusion`** — *Trait for budgeted k-path diffusion search over entity graphs.* (`src/path_rag/k_path.rs`)

```rust
pub trait KPathDiffusion : Send + Sync {
/// Discovers up to `k` paths between `source` and `target` bounded by `config`.
    ///
    /// Budget exhaustion (`budget_exhausted: true`) is returned as a regular `Ok(KPathResult)`.
    fn find_k_paths(
        &self,
        source: EntityId,
        target: EntityId,
        config: &KPathConfig,
    ) -> contextra_types::Result<KPathResult>;
}
```

**`PathGraph`** — *Trait für Graphen die PathRAG konsumieren kann. Ermöglicht Testbarkeit ohne echten CSR-Graphen.* (`src/path_rag/mod.rs`)

```rust
pub trait PathGraph : Send + Sync {
fn neighbors_with_weights(&self, node: EntityId) -> Vec<(EntityId, f32)>;
    fn predecessors_with_weights(&self, node: EntityId) -> Vec<(EntityId, f32)>;

    /// Returns hyperedge IDs associated with the given entity (default: empty).
    fn hyperedges_for_entity(&self, _node: EntityId) -> Vec<HyperEdgeId> {
        Vec::new()
    }

    /// Resolves a HyperEdgeId to its full HyperEdge details (default: None).
    fn get_hyperedge(&self, _id: HyperEdgeId) -> Option<Arc<HyperEdge>> {
        None
    }

    /// Returns participant role bindings for a given hyperedge ID (default: resolves via get_hyperedge).
    fn hyperedge_participants(&self, id: HyperEdgeId) -> Arc<[RoleBinding]> {
        match self.get_hyperedge(id) {
            Some(he) => he.participants.clone(),
            None => Arc::from([]),
        }
    }
}
```

### Öffentliche Datentypen — Structs (70)

| Struct | Felder / Form | Beschreibung | Datei |
|---|---|---|---|
| `AgentStateNode` | `{ pub step_id: NodeIdx, /// LLM prompt of this step. pub prompt: String, /// LLM response of this step. pub response: String, /// Optional MVCC snapshot referenc… }` | State of an agent step in the conversation DAG. | `src/session_dag.rs` |
| `ApprhGateMonitor` | `{ config: ApprhGateMonitorConfig, gate: Box<dyn ApprhFlipGate + Send + Sync>, state: Mutex<MonitorState>, }` | Independent, thread-safe monitor evaluating a series of APPRH shadow mode comparisons for production readiness (P1.3). # Invariants - Ring 0 (pure sy… | `src/apprh/gate_monitor.rs` |
| `ApprhGateMonitorConfig` | `{ /// Maximum capacity of the sliding window for historical observations. pub window_size: usize, /// Number of consecutive valid observations required for produ… }` | Configuration for [`ApprhGateMonitor`]. | `src/apprh/gate_monitor.rs` |
| `ApprhGateMonitorSnapshot` | `{ /// Total number of observations received since creation or last reset. pub total_observations: u64, /// Current count of consecutive passed observations (stre… }` | Telemetry snapshot from [`ApprhGateMonitor`]. | `src/apprh/gate_monitor.rs` |
| `ApprhObservationRecord` | `{ /// Maximum absolute diff from report. pub max_abs_diff: f32, /// Top-k Jaccard overlap ratio from report. pub top_k_overlap: f32, /// Boolean indicator whethe… }` | Historical observation record stored within the sliding window of [`ApprhGateMonitor`]. | `src/apprh/gate_monitor.rs` |
| `ApprhParams` | `{ /// Base PPR parameters ($\alpha$ teleport, $\epsilon$ convergence threshold). pub ppr: PprParams, /// Hyperedge decay factor ($\beta \in (0.0, 1.0]$, default:… }` | Configuration parameters for Averaging-based Personalized PageRank for Hypergraphs (APPRH). # Complexity Time complexity of parameter validation is $… | `src/apprh/params.rs` |
| `ApprhSelector` | `{ config: ApprhSelectorConfig, monitor: Arc<ApprhGateMonitor>, mode: RwLock<ApprhMode>, call_counter: AtomicU64, }` | Instance-level selector managing APPRH shadow-mode evaluation and production switching (P1.3). # Invariants - Instance object without global static s… | `src/apprh/selector.rs` |
| `ApprhSelectorConfig` | `{ /// Sampling rate for shadow evaluations: 1 = every call, N = every N-th call. pub sample_interval: u64, /// Maximum number of seed nodes allowed for shadow ev… }` | Configuration parameters for [`ApprhSelector`]. | `src/apprh/selector.rs` |
| `ApprhShadowComparison` | `{ /// Maximum absolute score difference across top-k union nodes. pub max_abs_diff: f32, /// Jaccard overlap ratio between top-k entity ID sets [0.0, 1.0]. pub t… }` | Result structure of shadow mode comparison between Forward-Push PPR and APPRH. | `src/apprh/shadow.rs` |
| `ArcSlice` | `{ shared: Arc<[T]>, offset: usize, len: usize, }` | A zero-copy slice view into an underlying `Arc<[T]>`. | `src/arc_slice.rs` |
| `CascadeInvalidationReport` | `{ /// Number of edges tombstoned as a result of the superseded document. pub tombstoned_edge_count: usize, /// Unique node (entity) IDs affected by the tombstone… }` | Report summarizing the cascade invalidation of graph edges derived from a superseded document. | `src/cascade.rs` |
| `CascadeTicket` | `{ pub id: u64, pub doc_id: DocId, pub status: CascadeStatus, pub proof: Option<DeletionProof>, }` | Ticket tracking persistent cascade invalidation status (§6.8). | `src/cascade.rs` |
| `CommunityAssignment` | `{ /// Entity ID. pub entity_id: EntityId, /// Community ID (represented as a 64-bit integer, initialized from the seed EntityId). pub community_id: u64, /// Comm… }` | Represents the assignment of an entity to a detected community. | `src/community.rs` |
| `CommunityDetectionConfig` | `{ /// Maximum number of propagation / modularity optimization iterations. pub max_iterations: u32, /// Seed for deterministic node traversal shuffling. pub seed:… }` | Configuration for Leiden Community Detection. | `src/community.rs` |
| `ConflictPattern` | `{ /// Blake3-Hash über die normalisierte (subject, predicate, object)-Tripel-Signatur der widersprüchlichen Aussage. pub pattern_hash: [u8; 32], /// Anzahl der b… }` | Eintrag für ein gelerntes Konfliktmuster (Constraint-Violation-Signatur). | `src/consistency_enforcement.rs` |
| `ConsistencyEnforcer` | `{ patterns: HashMap<[u8; 32], ConflictPattern>, suppression_threshold: u32, }` | Consistency-Enforcement-Register zur Verwaltung registrierter Konfliktmuster. | `src/consistency_enforcement.rs` |
| `ConsolidationNodesGuard` | `{ entities: Vec<EntityId>, }` | Guard zur Erzwingung kanonischer Lock-Reihenfolge über mehrere Entitäten (H2 / §5.1.2). Stellt sicher, dass Entitäts-IDs vor dem Erwerb mehrerer Schl… | `src/hyperedge.rs` |
| `CooccurrenceSignal` | `{ pub from: EntityId, pub to: EntityId, pub co_activation: f32, }` | Ein gepufferter Co-Occurrence-Update: zwei ko-aktivierte Entitäten. | `src/edge_reinforcement_buffer.rs` |
| `CsrGraph` | `{ pub config: CsrGraphConfig, pub inner: Arc<ArcSwap<GraphInner>>, pub write_state: Arc<Mutex<GraphInner>>, /// Optionaler Persistenz-Handle. None = reiner In-Me… }` | Compressed Sparse Row graph for entity-relation traversal. Implements `GraphIndex` trait as Signal 3 in the 4-Signal Fusion architecture. | `src/csr/graph_write.rs` |
| `CsrGraphConfig` | `{ /// Rebuild threshold: max number of uncompacted pending edges in delta buffer before triggering an automatic full CSR rebuild. pub rebuild_threshold: usize, /… }` | — | `src/csr/types.rs` |
| `DagEdge` | `{ pub parent: NodeIdx, pub child: NodeIdx, /// Label (e.g. "explore", "reject", "main") pub label: String, }` | Directed edge in the DAG: parent -> child. | `src/session_dag.rs` |
| `DefaultApprhFlipGate` | `(tuple/unit)` | Standard default flip gate evaluator for APPRH. | `src/apprh/shadow.rs` |
| `DeletedView` | `{ deleted_nodes: HashSet<usize>, }` | Information view over graph tombstones/deleted node indices. MUST NOT implement `Default`. Constructible only via explicit paths that load tombstone… | `src/ppr.rs` |
| `DeletionProof` | `{ pub doc_id: DocId, pub tombstoned_count: usize, pub wal_seq: u64, }` | Proof of edge / hyperedge deletion during cascade invalidation (§6.8). | `src/cascade.rs` |
| `DocEdgeIndex` | `{ doc_to_edges: RwLock<AHashMap<DocId, AHashSet<EdgeId>>>, edge_provenance: RwLock<AHashMap<EdgeId, EdgeProvenance>>, }` | Rückverfolgung DocId -> betroffene Kanten, für Cascading-Invalidation (INV-GRAPH-PROV-1). Speichert sowohl die Zuordnung DocId -> Kanten als auch Edg… | `src/provenance.rs` |
| `Edge` | `{ pub target: EntityId, pub weight: f32, pub edge_type: EdgeType, #[cfg(feature = "edge-reinforcement-learning")] pub cooccurrence_weight: f32, // w_ij, initiali… }` | Edge structure in CSR graph representation. | `src/csr/types.rs` |
| `EdgeAssertion` | `{ /// Subjekt-Knoten der Aussage. pub subject: NodeIdx, /// Hash des Prädikats/Relationsnamens. pub predicate_hash: [u8; 32], /// Repräsentation des Objekts/Ziel… }` | Abstrakte Aussage über eine Kante für die semantische Widerspruchsprävention. | `src/consistency_enforcement.rs` |
| `EdgeProvenance` | `{ /// Identifikator der Kante (from_entity_id, to_entity_id). pub edge_id: EdgeId, /// Dokumente deren Inhalt zu dieser Kante beigetragen hat. pub source_doc_ids… }` | Herkunftsnachweis für eine Graph-Kante. INV-GRAPH-PROV-1: Jede aktive CSR-Kante trägt einen EdgeProvenance-Eintrag, der ihre Quell-Dokumente zurückve… | `src/provenance.rs` |
| `EdgeReinforcementBuffer` | `{ cooccurrence: Mutex<Vec<CooccurrenceSignal>>, traversal: Mutex<Vec<TraversalSignal>>, }` | Puffert pending Edge-Reinforcement-Updates bis zum nächsten Scheduler-Flush. Aufrufer (z.B. Collection::search nach erfolgreichem Retrieval) akkumuli… | `src/edge_reinforcement_buffer.rs` |
| `EdgeReinforcementConfig` | `{ /// Co-occurrence reinforcement rate η. Default: 0.01. pub eta: f32, /// Passiver Kantenzerfall δ. Default: 0.001. pub delta: f32, /// Maximale Gesamtgewicht p… }` | — | `src/edge_reinforcement.rs` |
| `ExactPredicateConflictDetector` | `(tuple/unit)` | Referenzimplementierung für exakten Prädikats-Konflikt: Ein Widerspruch liegt vor, wenn Subjekt und Prädikats-Hash identisch sind, die Objekt-Repräse… | `src/consistency_enforcement.rs` |
| `ExtractedRelation` | `{ /// Name / Identifikator der Quell-Entität (Subjekt). pub source: String, /// Name / Identifikator der Ziel-Entität (Objekt). pub target: String, /// Typ / Bez… }` | Repräsentiert eine aus Freitext extrahierte Binärbeziehung zwischen zwei Entitäten. | `src/entity_extraction.rs` |
| `GraphInner` | `{ /// Mapping from public EntityId to internal contiguous index. pub(crate) id_map: HashMap<EntityId, InternalIndex>, /// Mapping from internal index back to Ent… }` | Inner state of the CsrGraph to manage contiguous storage. | `src/csr/inner.rs` |
| `GraphPath` | `{ /// Knoten-Sequenz vom Start zum Ziel. pub nodes: Vec<EntityId>, /// Kantengewichte entlang des Pfades (len = nodes.len() - 1). pub edge_weights: Vec<f32>, ///… }` | Ein gefundener Pfad zwischen zwei Knoten. | `src/path_rag/mod.rs` |
| `GraphStats` | `{ /// Number of active/committed nodes \|V\|. pub node_count: usize, /// Number of directed edges \|E\|. pub edge_count: usize, /// Average outgoing degree \|E\| / \|V\|… }` | CSR Graph structural statistics used for PPR cost estimation. | `src/ppr/cost.rs` |
| `HyperEdge` | `{ /// Eindeutige ID der Hyperkante. pub id: HyperEdgeId, /// Prädikat/Typ der Hyperkante. pub predicate: EdgeType, /// Liste der Teilnehmer mit ihren Rollen. ///… }` | Eine n-äre Hyperkante zur Verbindung von 2 oder mehr Entitäten mit spezifischen Rollen. | `src/hyperedge.rs` |
| `HyperEdgeCandidate` | `{ /// Canonical sorted list of participating entity IDs. pub participants: Vec<EntityId>, /// Number of distinct documents in which this exact entity combination… }` | A candidate hyperedge derived from document co-occurrence. | `src/hyperedge_suggest.rs` |
| `HyperEdgeId` | `(tuple/unit)` | — | `src/hyperedge.rs` |
| `HyperEdgeView` | `{ pub edge: &'a HyperEdge, pub participants: ArcSlice<RoleBinding>, }` | Zero-Copy read-only view of a [`HyperEdge`] backed by [`ArcSlice<RoleBinding>`]. | `src/hyperedge.rs` |
| `HyperedgeCascadeReport` | `{ /// Hyperedges tombstoned synchronously in this cascade run (up to MAX_HYPEREDGE_CASCADE_FANOUT). pub invalidated: Vec<crate::hyperedge::HyperEdgeId>, /// Hype… }` | Report summarizing the cascade invalidation of hyperedges derived from a superseded document. | `src/cascade.rs` |
| `KPathConfig` | `{ /// Target number of paths to discover. pub k: usize, /// Maximum number of distinct graph nodes visited across forward/backward search. pub max_visited_nodes:… }` | Configuration parameters for [`KPathDiffusion`]. | `src/path_rag/k_path.rs` |
| `KPathResult` | `{ /// Up to `k` discovered paths between source and target. pub paths: Vec<Vec<EntityId>>, /// Indicates whether the search was halted due to `max_visited_nodes`… }` | Result returned by [`KPathDiffusion::find_k_paths`]. | `src/path_rag/k_path.rs` |
| `MemoryEstimate` | `{ /// Memory used by shared payloads (e.g. `Arc<HyperEdge>` / `Arc<[RoleBinding]>`). pub shared_payload_bytes: usize, /// Memory used by private internal graph s… }` | Detailed memory estimate separating shared payloads from private structural allocations (§6.3). | `src/csr/inner.rs` |
| `NodesGuard` | `{ tree: &'a SessionBranchTree, _nodes_guard: RwLockReadGuard<'a, HashMap<NodeIdx, AgentStateNode>>, }` | Guard, der NUR erhalten werden kann, nachdem der `nodes`-Lock bereits gehalten wird. Erzwingt die Lock-Reihenfolge nodes -> edges/active_head zur Com… | `src/session_dag.rs` |
| `NodesWriteGuard` | `{ tree: &'a SessionBranchTree, _nodes_guard: RwLockWriteGuard<'a, HashMap<NodeIdx, AgentStateNode>>, }` | Exklusiver Schreib-Guard, der NUR erhalten werden kann, nachdem der `nodes`-Schreiblock bereits gehalten wird. Erzwingt die Lock-Reihenfolge nodes ->… | `src/session_dag.rs` |
| `PathRAGConfig` | `{ /// Maximale Suchtiefe (Hop-Limit). pub max_hops: usize, /// Sufficiency-Schwelle: Pfade unter dieser Konfidenz werden gefiltert. pub sufficiency_threshold: f6… }` | Configuration options for [`PathRAGEngine`]. | `src/path_rag/mod.rs` |
| `PathRAGEngine` | `{ graph: G, pub config: PathRAGConfig, }` | — | `src/path_rag/mod.rs` |
| `PercolationConfig` | `{ /// Kritischer Perkolationsschwellenwert θ_perc. Default: 0.7. pub critical_threshold: f32, /// Similarity-Schwellenwert für Re-Bonding θ_bond. Default: 0.85.… }` | Konfiguration für den Perkolations-Gesundheitsmonitor. | `src/percolation.rs` |
| `PersistedEdgePayload` | `{ pub weight: f32, #[serde(default, alias = "valid_from")] pub tx_valid_from: Option<TxId>, #[serde(default, alias = "valid_to")] pub tx_valid_to: Option<TxId>,… }` | Persisted edge payload format for storage. | `src/csr/types.rs` |
| `PprCandidateStream` | `{ graph: &'a G, seeds: Vec<EntityId>, config: PprConfig, batch_size: usize, exclude_seeds: bool, ranked: Option<VecDeque<(EntityId, f32)>>, yielded: usize, }` | On-demand loading async candidate stream for Personalized PageRank search. | `src/ppr_stream.rs` |
| `PprContext` | `{ valid_seeds: Vec<usize>, out_weight_sums: Vec<f32>, ranks: Vec<f32>, next_ranks: Vec<f32>, }` | Reusable scratch buffers for Personalized PageRank (PPR) power iteration. Pre-allocating `PprContext` eliminates all intermediate heap allocations du… | `src/ppr.rs` |
| `PprCostCalibrator` | `{ samples: Arc<Mutex<Vec<PprCostSample>>>, }` | ShadowMode calibrator that collects `PprCostSample` observations and derives a recommended cost threshold multiplier without automatically applying i… | `src/ppr/cost.rs` |
| `PprCostEstimate` | `{ /// Estimated edge accesses for Andersen-Chung-Lang ForwardPush: $O(1 / (\epsilon \cdot \alpha))$. pub forward_push: f64, /// Estimated edge accesses for Dense… }` | Cost estimate for PPR evaluation algorithms in estimated edge access units. | `src/ppr/cost.rs` |
| `PprCostSample` | `{ /// Algorithm evaluated. pub algorithm: PprAlgorithm, /// Estimated cost in edge access units. pub estimated_cost: f64, /// Measured edge accesses (counted ope… }` | A sample recording estimated cost vs measured edge access counters (operations count, no wall-clock time). | `src/ppr/cost.rs` |
| `PprParams` | `{ /// Teleport probability ($\alpha$, default: 0.15). pub alpha: f32, /// Convergence error tolerance threshold ($\epsilon$, default: 1e-4). pub epsilon: f32, //… }` | Configuration parameters for Forward-Push Personalized PageRank (PPR). | `src/path_rag/mod.rs` |
| `RoleBinding` | `{ /// Die Rolle der Entity in der Hyperkante. pub role: RoleId, /// Die gebundene Entity. pub entity: EntityId, }` | Bindung einer Entity an eine spezifische Rolle innerhalb einer Hyperkante. | `src/hyperedge.rs` |
| `RoleId` | `(tuple/unit)` | — | `src/hyperedge.rs` |
| `RoleInterner` | `{ forward: HashMap<String, RoleId>, backward: HashMap<RoleId, String>, next_id: AtomicU32, }` | Concurrent string interner for hyperedge participant roles ([`RoleId`]). Maps role names (`&str`) to numeric [`RoleId`]s and vice versa. Uses lock-fr… | `src/hyperedge.rs` |
| `RuleBasedEntityExtractor` | `{ /// Minimale Konfidenzschwelle. Ergebnisse unterhalb dieser Schwelle werden nicht zurückgegeben. min_confidence: f32, /// Registrierte Extraktionsmuster. patte… }` | Regelbasierte Referenzimplementierung zur Extraktion von Entitäten und Relationen. Nutzt Phrasen-Matching in Kombination mit Erkennung von Großbuchst… | `src/entity_extraction.rs` |
| `SessionBranchTree` | `{ nodes: RwLock<HashMap<NodeIdx, AgentStateNode>>, edges: RwLock<Vec<DagEdge>>, active_head: RwLock<NodeIdx>, next_id: Arc<std::sync::atomic::AtomicU64>, }` | Session-DAG for a single agent conversation tree. # Invariants - Directed Acyclic Graph: parent must exist before branch creation - `active_head` alw… | `src/session_dag.rs` |
| `SessionDagStepOptions` | `{ pub prompt: String, pub response: String, pub snapshot_tx_id: Option<TxId>, pub tool_outputs: Vec<String>, pub label: &'a str, }` | Options for creating or branching a new step in the Session-DAG. | `src/session_dag.rs` |
| `ShadowComparison` | `{ /// Maximum absolute score difference across top-k union nodes. pub max_abs_diff: f32, /// Jaccard overlap ratio between top-k entity ID sets [0.0, 1.0]. pub t… }` | Result structure of shadow mode comparison between Forward-Push PPR and TL-HFD. | `src/tl_hfd/shadow.rs` |
| `ShadowDiscrepancyReport` | `{ /// Number of evaluated query pairs (ForwardPush vs TL-HFD). pub sample_count: u64, /// Mean Jaccard similarity of top-k entity ID sets between algorithms. pub… }` | Aggregated discrepancy report over a shadow mode evaluation window. | `src/tl_hfd/shadow.rs` |
| `SnapshotPathGraph` | `{ inner: &'a GraphInner, as_of_tx: TxId, }` | Snapshot-isolated wrapper around `GraphInner` at a specific transaction sequence number (`as_of_tx`). | `src/path_rag/snapshot.rs` |
| `StarExpansionIterator` | `{ inner: arc_swap::Guard<std::sync::Arc<crate::csr::inner::GraphInner>>, hyperedge_keys: Vec<crate::hyperedge::HyperEdgeId>, current_hyperedge_idx: usize, curren… }` | Stern-Expansion: Jede Hyperkante wird als künstlicher bipartiter Knoten (`VirtualHyperedgeNode`) repräsentiert. Der Iterator gaukelt dem Leiden-Solve… | `src/community.rs` |
| `TlHfdFlipGate` | `(tuple/unit)` | Standard default flip gate evaluator for TL-HFD. | `src/tl_hfd/shadow.rs` |
| `TlHfdParams` | `{ /// Regularization parameter $\sigma > 0$ controlling diffusion scale (default: 0.1). pub sigma: f32, /// Seed injection multiplier $\delta \ge 2.0$ (default:… }` | Configuration parameters for Thresholded Local Hyper-Flow Diffusion (TL-HFD). # Complexity Time complexity of parameter validation is $O(1)$. | `src/tl_hfd/params.rs` |
| `TraversalSignal` | `{ pub from: EntityId, pub to: EntityId, pub path_length: usize, }` | Ein gepufferter Traversal-Update: eine traversierte Kante mit Pfadlänge. | `src/edge_reinforcement_buffer.rs` |
| `ValidatedHyperEdgeCandidate` | `{ /// The underlying co-occurrence candidate. pub candidate: HyperEdgeCandidate, /// Suggested semantic predicate string. pub predicate: String, /// LLM validati… }` | A candidate hyperedge after LLM semantic validation. | `src/hyperedge_suggest.rs` |
| `VirtualHyperedgeNode` | `{ /// ID der zugrundeliegenden Hyperkante. pub hyperedge_id: crate::hyperedge::HyperEdgeId, }` | Künstlicher bipartiter Knoten für eine Hyperkante in der Stern-Expansion. | `src/community.rs` |

### Öffentliche Datentypen — Enums (7)

| Enum | Varianten | Beschreibung | Datei |
|---|---|---|---|
| `ApprhError` | `/// Invalid parameter value provided to APPRH. #[error("Invalid parameter: {0}")] InvalidParameter(String), /// Empty seed list provided to APPRH. #[error("Seed list cannot be empty")] EmptySeeds, //…` | Error type for APPRH operations and parameter validations. | `src/apprh/error.rs` |
| `ApprhMode` | `/// APPRH disabled; authoritative Forward-Push PPR is returned. Off, /// APPRH shadow mode (default): returns authoritative Forward-Push PPR result, /// and periodically evaluates shadow comparisons…` | Operating mode for [`ApprhSelector`]. | `src/apprh/selector.rs` |
| `CascadeStatus` | `/// Invalidation task is queued and pending. Pending, /// Invalidation task is currently processing. InFlight, /// Invalidation completed successfully with a deletion proof. Completed, /// Invalidati…` | Status tracking for a cascade invalidation ticket (§6.8). | `src/cascade.rs` |
| `EdgeType` | `#[default] Default,` | Edge type representation for CSR edges. | `src/csr/types.rs` |
| `GraphMutationError` | `/// Lock acquisition timed out during concurrent mutation. #[error("Lock acquisition timed out: {0}")] LockAcquisitionTimeout(String), /// Provided role binding is invalid. #[error("Invalid role bind…` | Error variants for graph mutation operations. | `src/error.rs` |
| `HyperedgeSuggestError` | `/// Candidate count exceeds the maximum LLM calls allowed per cycle. #[error("LLM validation budget exceeded: {0} candidates exceed limit")] LlmValidationBudgetExceeded(usize),` | Errors specific to hyperedge suggestion and validation. | `src/hyperedge_suggest.rs` |
| `TlHfdError` | `/// Invalid parameter value provided to TL-HFD. #[error("Invalid parameter: {0}")] InvalidParameter(String), /// Non-finite value encountered during diffusion calculation. #[error("Non-finite value e…` | Error type for TL-HFD operations and parameter validations. | `src/tl_hfd/error.rs` |

### Typ-Aliase (4)

| Alias | Definition | Beschreibung |
|---|---|---|
| `CascadeReport` | `type CascadeReport= HyperedgeCascadeReport` | Alias for [`HyperedgeCascadeReport`] (§6.8 specification alignment). |
| `EdgeId` | `type EdgeId= (EntityId, EntityId)` | Identifikator für eine Kante im CSR-Graphen. |
| `EdgePayload` | `type EdgePayload= super::types::EdgePayload` | — |
| `NodeIdx` | `type NodeIdx= u64` | Index of a node in the Session-DAG. |

### Konstanten (17)

| Konstante | Typ | Beschreibung |
|---|---|---|
| `CASCADE_QUEUE_PREFIX` | `&str` | LSM-Key-Präfix für die persistent cascade queue (§6.6 / §6.8). |
| `DEFAULT_HYPEREDGE_CASCADE_FANOUT_LIMIT` | `usize` | Default fan-out limit for hyperedge cascade invalidation per document (§6.8). |
| `DEFAULT_PPR_STREAM_BATCH_SIZE` | `usize` | Default batch size for candidate retrieval (Spec §21.2). |
| `DEFAULT_SUFFICIENCY_THRESHOLD` | `f64` | Normativer Default-Schwellenwert für das Sufficiency-Gate (ADR-067). Pfade mit Konfidenz < 0.10 werden verworfen, um Precision-Kollaps durch Rauschen… |
| `DEFAULT_SUPPRESSION_THRESHOLD` | `u32` | Standard-Schwellenwert für die Unterdrückung (3 gegenseitige Detektionen). |
| `HYPEREDGE_BY_ENTITY_PREFIX` | `&str` | LSM-Key-Präfix für Hyperkanten-Index nach Entity. |
| `HYPEREDGE_PREFIX` | `&str` | LSM-Key-Präfix für Hyperkanten. |
| `MAX_DAG_STRING_BYTES` | `usize` | Maximum allowed byte size for prompt/response strings (10 MB). |
| `MAX_DAG_TRAVERSAL_DEPTH` | `usize` | Maximale Pfadtiefe in einem SessionDag (Schutz vor Zyklen und Tiefenexplosion). |
| `MAX_HYPEREDGE_CASCADE_FANOUT` | `usize` | Maximum number of hyperedges processed synchronously per cascade invalidation run to bound latency spikes. Any remaining hyperedges are returned in `… |
| `MAX_RELATE_PARTICIPANTS` | `usize` | Maximum allowed participants in a hyperedge candidate (Combinatorial explosion guard, P5). |
| `MAX_TRAVERSAL_HOPS` | `u8` | Maximum traversal depth. |
| `MAX_VISITED_NODES` | `usize` | Maximum visited nodes limit during BFS traversal to prevent intermediate hub-node memory explosion. |
| `MIN_AGREEMENT_THRESHOLD` | `f32` | Minimum required mean top-k Jaccard agreement threshold for shadow mode default flip evaluation. |
| `MIN_SHADOW_SAMPLES` | `u64` | Minimum required sample count for shadow mode default flip evaluation. |
| `SCORE_DECAY` | `f32` | Score decay factor per hop (0.7^hop). |
| `WALLCLOCK_TX_HEURISTIC_MIN` | `u64` | Untere Schranke für Wall-Clock-abgeleitete TxId-Heuristik. Unix-Nanosekunden seit Epoch lagen am 01-01-2014 bei ca. 1.39×10¹⁸. TxIds, die in diesen B… |

### Öffentliche Funktionen (248)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn run_apprh_push<G: PathGraph>( graph: &G, seeds: &[EntityId], params: &ApprhParams, ) -> Result<AHashMap<EntityId, f32>, ApprhError>` | Computes Averaging-based Personalized PageRank for Hypergraphs (APPRH). # Determinism Pushes and neighbor iterations are strictly ordered b… | `src/apprh/diffusion.rs` |
| `pub fn config(&self) -> &ApprhGateMonitorConfig` | Returns a reference to the monitor's configuration. | `src/apprh/gate_monitor.rs` |
| `pub fn new(config: ApprhGateMonitorConfig) -> Result<Self, ApprhError>` | Creates a new [`ApprhGateMonitor`] with the provided configuration and standard [`DefaultApprhFlipGate`]. # Errors Returns [`ApprhError::In… | `src/apprh/gate_monitor.rs` |
| `pub fn observe(&self, report: &ApprhShadowComparison)` | Observes a single [`ApprhShadowComparison`] report, evaluating it against the injected gate. An observation is considered a pass if the gat… | `src/apprh/gate_monitor.rs` |
| `pub fn production_ready(&self) -> bool` | Returns `true` if the monitor currently deems APPRH ready for production replacement. Production readiness is declared when the streak of c… | `src/apprh/gate_monitor.rs` |
| `pub fn reset(&self)` | Resets the monitor state, clearing the sliding window, total observations counter, and pass streak. | `src/apprh/gate_monitor.rs` |
| `pub fn snapshot(&self) -> ApprhGateMonitorSnapshot` | Captures a telemetry snapshot of the current monitor state and sliding window. | `src/apprh/gate_monitor.rs` |
| `pub fn validate(&self) -> Result<(), ApprhError>` | Validates the monitor configuration parameters. # Errors Returns [`ApprhError::InvalidParameter`] if any parameter is invalid. | `src/apprh/gate_monitor.rs` |
| `pub fn with_gate<G>(config: ApprhGateMonitorConfig, gate: G) -> Result<Self, ApprhError> where G: ApprhFlipGate + Send + Sync + 'static,` | Creates a new [`ApprhGateMonitor`] with the provided configuration and a custom [`ApprhFlipGate`]. # Errors Returns [`ApprhError::InvalidPa… | `src/apprh/gate_monitor.rs` |
| `pub fn apprh_diffusion( &self, seeds: &[EntityId], params: &ApprhParams, ) -> Result<AHashMap<EntityId, f32>, ApprhError>` | Computes Averaging-based Personalized PageRank for Hypergraphs (APPRH) over the graph. # Invariants Runs exclusively in shadow mode unless… | `src/apprh/mod.rs` |
| `pub fn apprh_local<G: PathGraph>( graph: &G, seeds: &[EntityId], params: &ApprhParams, ) -> Result<AHashMap<EntityId, f32>, ApprhError>` | Computes Averaging-based Personalized PageRank for Hypergraphs (APPRH) over a generic [`PathGraph`]. # Invariants Runs exclusively in shado… | `src/apprh/mod.rs` |
| `pub fn apprh_shadow_compare( &self, seeds: &[EntityId], ppr: &PprParams, apprh_params: &ApprhParams, top_k: usize, ) -> Result<ApprhShadowComparison, ApprhError>` | Computes shadow comparison between Forward-Push PPR and APPRH on CsrGraph. | `src/apprh/mod.rs` |
| `pub fn evaluate_apprh_diffusion( &self, seeds: &[EntityId], params: &ApprhParams, ) -> Result<AHashMap<EntityId, f32>, ApprhError>` | Evaluates APPRH diffusion over CsrGraph. | `src/apprh/mod.rs` |
| `pub fn evaluate_apprh_shadow( &self, seeds: &[EntityId], ppr: &PprParams, apprh_params: &ApprhParams, top_k: usize, ) -> Result<ApprhShadowComparison, ApprhError>` | Evaluates APPRH shadow comparison on CsrGraph. | `src/apprh/mod.rs` |
| `pub fn forward_push_apprh<G: PathGraph>( graph: &G, seeds: &[EntityId], params: &PprParams, hyperedge_decay_factor: f32, ) -> Result<AHashMap<EntityId, f32>, contextra_types::ContextraError>` | Spec-compliant additive APPRH operator (Spec Y.1.2). Computes Averaging-based Personalized PageRank for Hypergraphs (APPRH) over a generic… | `src/apprh/mod.rs` |
| `pub fn validate(&self) -> Result<(), ApprhError>` | Validates configuration parameters against domain constraints. # Errors Returns [`ApprhError::InvalidParameter`] if any parameter violates… | `src/apprh/params.rs` |
| `pub fn demote_to_shadow(&self)` | Explicitly demotes the selector to `Shadow` mode. | `src/apprh/selector.rs` |
| `pub fn evaluate_and_run<G: PathGraph>( &self, graph: &G, seeds: &[EntityId], ppr_params: &PprParams, apprh_params: &ApprhParams, ) -> Result<Vec<(EntityId, f32)>, ApprhError>` | Evaluates and executes Personalized PageRank over the graph according to current operating mode. # Errors Returns [`ApprhError`] if graph e… | `src/apprh/selector.rs` |
| `pub fn mode(&self) -> ApprhMode` | Returns the current operating mode of the selector. | `src/apprh/selector.rs` |
| `pub fn monitor(&self) -> &ApprhGateMonitor` | Returns a reference to the internal [`ApprhGateMonitor`]. | `src/apprh/selector.rs` |
| `pub fn new(config: ApprhSelectorConfig, monitor: Arc<ApprhGateMonitor>) -> Self` | Creates a new [`ApprhSelector`] with the specified configuration and gate monitor. | `src/apprh/selector.rs` |
| `pub fn promote_to_production(&self) -> Result<(), ApprhError>` | Explicitly promotes the selector to `Production` mode. # Errors Returns [`ApprhError::InvalidParameter`] if gate monitor is not production… | `src/apprh/selector.rs` |
| `pub fn set_mode(&self, new_mode: ApprhMode) -> Result<(), ApprhError>` | Explicitly sets the operating mode of the selector. # Errors Returns [`ApprhError::InvalidParameter`] if attempting to promote to `Producti… | `src/apprh/selector.rs` |
| `pub fn with_default_monitor(config: ApprhSelectorConfig) -> Result<Self, ApprhError>` | Creates a selector with default configuration and standard gate monitor. # Errors Returns [`ApprhError`] if monitor configuration fails val… | `src/apprh/selector.rs` |
| `pub fn shadow_compare_forward_push_vs_apprh<G: PathGraph>( graph: &G, seeds: &[EntityId], ppr: &PprParams, apprh_params: &ApprhParams, top_k: usize, ) -> Result<ApprhShadowComparison, ApprhError>` | Computes shadow comparison between Forward-Push PPR and APPRH. # Invariants - Authoritative result is Forward-Push PPR. - Emits `tracing::w… | `src/apprh/shadow.rs` |
| `pub fn as_arc(&self) -> &Arc<[T]>` | Returns a reference to the underlying `Arc<[T]>` slice. | `src/arc_slice.rs` |
| `pub fn as_slice(&self) -> &[T]` | Returns a reference to the slice of elements. | `src/arc_slice.rs` |
| `pub fn from_vec(vec: Vec<T>) -> Self` | Creates an `ArcSlice` from a `Vec<T>`. | `src/arc_slice.rs` |
| `pub fn is_empty(&self) -> bool` | Returns `true` if the slice contains no elements. | `src/arc_slice.rs` |
| `pub fn len(&self) -> usize` | Returns the number of elements in the sub-slice. | `src/arc_slice.rs` |
| `pub fn new(shared: Arc<[T]>) -> Self` | Creates a new `ArcSlice` wrapping the entire `Arc<[T]>`. | `src/arc_slice.rs` |
| `pub fn offset(&self) -> usize` | Returns the offset within the backing `Arc<[T]>`. | `src/arc_slice.rs` |
| `pub fn slice(&self, range: impl RangeBounds<usize>) -> Self` | Creates a zero-copy sub-slice within the given index range bounds, clamping to valid bounds. | `src/arc_slice.rs` |
| `pub fn try_slice(&self, range: impl RangeBounds<usize>) -> Option<Self>` | Creates a zero-copy sub-slice within the given index range bounds. Returns `None` if the range bounds are invalid or out of bounds. | `src/arc_slice.rs` |
| `pub async fn cascade_invalidate_edges_for_superseded_doc( graph: &CsrGraph, superseded_doc_id: DocId, wal_seq: u64, ) -> Result<CascadeInvalidationReport>` | Tombstoniert alle Kanten, die von `superseded_doc_id` abgeleitet wurden, als Reaktion auf ein Supersedes-Ereignis. Idempotent: mehrfacher A… | `src/cascade.rs` |
| `pub async fn cascade_invalidate_hyperedges_for_superseded_doc( graph: &CsrGraph, superseded_doc_id: DocId, wal_seq: u64, ) -> Result<HyperedgeCascadeReport>` | Cascade invalidates hyperedges derived from `superseded_doc_id` with fan-out protection and recursive super-edge ancestor invalidation. Up… | `src/cascade.rs` |
| `pub fn topological_sort_hyperedge_closure( graph: &CsrGraph, closure_nodes: &[crate::hyperedge::HyperEdgeId], ) -> Vec<crate::hyperedge::HyperEdgeId>` | Cascade invalidates hyperedges derived from `superseded_doc_id` with fan-out protection. Up to `MAX_HYPEREDGE_CASCADE_FANOUT` hyperedges ar… | `src/cascade.rs` |
| `pub fn verify_doc_hyperedge_deletion_completeness( graph: &CsrGraph, target_doc_id: DocId, ) -> Result<()>` | Verifies byte-accurately that no active (non-tombstoned) hyperedge in `graph` directly or transitively (via `child_edge_ids`) references `t… | `src/cascade.rs` |
| `pub async fn detect_communities( graph: &CsrGraph, config: &CommunityDetectionConfig, ) -> Result<Vec<CommunityAssignment>>` | Detects semantic communities in the given CSR graph using the deterministic Leiden algorithm. # Determinism and Reproducibility Guarantees… | `src/community.rs` |
| `pub fn new(hyperedge_id: u64) -> Self` | Creates a new `VirtualHyperedgeNode`. | `src/community.rs` |
| `pub fn new(graph: &CsrGraph) -> Self` | Erstellt einen neuen `StarExpansionIterator` für den gegebenen CsrGraph. | `src/community.rs` |
| `pub fn next_with_weight(&mut self) -> Option<(EntityId, VirtualHyperedgeNode, f32)>` | Liest das nächste Element inkl. Kantengewicht (Konvention K: `2w/(|e|-1)`). | `src/community.rs` |
| `pub fn active_patterns(&self) -> impl Iterator<Item = &ConflictPattern>` | Gibt einen Iterator über alle aktuell aktiven (unterdrückenden) Konfliktmuster zurück. | `src/consistency_enforcement.rs` |
| `pub fn check_before_insert(&mut self, assertion: &EdgeAssertion) -> Option<ConflictPattern>` | Prüft eine Kanten-Aussage vor dem Einfügen, registriert das Widerspruchsmuster und gibt den Konfliktmuster-Eintrag zurück. | `src/consistency_enforcement.rs` |
| `pub fn detect_contradiction<D: ContradictionDetector>( &self, detector: &D, a: &EdgeAssertion, b: &EdgeAssertion, ) -> bool` | Bewertet mit Hilfe des übergebenen [`ContradictionDetector`]s, ob zwei Kanten-Aussagen einen semantischen Widerspruch darstellen. | `src/consistency_enforcement.rs` |
| `pub fn get_pattern(&self, pattern_hash: &[u8; 32]) -> Option<&ConflictPattern>` | Gibt ein registriertes Konfliktmuster zu einem Muster-Hash zurück, falls vorhanden. | `src/consistency_enforcement.rs` |
| `pub fn is_suppressed(&self, pattern_hash: [u8; 32]) -> bool` | Prüft, ob ein gegebenes Muster unterdrückt wird (`suppressed == true`). | `src/consistency_enforcement.rs` |
| `pub fn new(suppression_threshold: u32) -> Self` | Erstellt einen neuen ConsistencyEnforcer mit konfigurierbarem Schwellenwert. | `src/consistency_enforcement.rs` |
| `pub fn pattern_hash(&self) -> [u8` | Berechnet den Blake3-Pattern-Hash über (subject, predicate_hash, object_repr). | `src/consistency_enforcement.rs` |
| `pub fn record_contradiction( &mut self, pattern_hash: [u8; 32], at_tx: TxId, ) -> &ConflictPattern` | Registriert oder aktualisiert einen Widerspruch für den gegebenen Muster-Hash (`pattern_hash`). Erhöht `contradiction_count` und setzt `sup… | `src/consistency_enforcement.rs` |
| `pub fn suggest_tombstone_candidates( &self, csr_edges_matching_pattern: &[EdgeId], ) -> Vec<EdgeId>` | Integration mit der Kanten-Provenienz-Cascade (§4.5 der Spezifikation). Nimmt eine Liste von Kanten-IDs entgegen, die einem unterdrückten M… | `src/consistency_enforcement.rs` |
| `pub fn suppression_threshold(&self) -> u32` | Gibt die konfigurierte Unterdrückungsschwelle zurück. | `src/consistency_enforcement.rs` |
| `pub async fn delete_edge_persistence<S: StorageEngine + ?Sized>( &self, storage: &S, tx: TxId, from: &EntityId, to: &EntityId, ) -> Result<()>` | Löscht eine einzelne Edge aus dem übergebenen Storage. | `src/csr/graph_persist.rs` |
| `pub async fn load_from_storage<S: StorageEngine + ?Sized>(storage: &S) -> Result<Self>` | Lädt den kompletten Graph-Zustand aus dem Storage (beim Startup). | `src/csr/graph_persist.rs` |
| `pub async fn persist_edge<S: StorageEngine + ?Sized>( &self, storage: &S, tx: TxId, from: &EntityId, to: &EntityId, payload: &PersistedEdgePayload, ) -> Result<()>` | Persistiert eine einzelne Edge in den übergebenen Storage. | `src/csr/graph_persist.rs` |
| `pub async fn persist_entity<S: StorageEngine + ?Sized>( &self, storage: &S, tx: TxId, entity: &Entity, ) -> Result<()>` | Persistiert eine einzelne Entity in den übergebenen Storage. | `src/csr/graph_persist.rs` |
| `pub fn compact(&self)` | Force compacts the graph delta buffer into the main CSR arrays to optimize traversal layout. | `src/csr/graph_read.rs` |
| `pub async fn compact_async(&self) -> Result<()>` | Asynchronously compacts the graph delta buffer. | `src/csr/graph_read.rs` |
| `pub async fn deleted_view(&self) -> crate::DeletedView` | Loads tombstone status of all graph nodes and constructs an authoritative [`crate::DeletedView`]. | `src/csr/graph_read.rs` |
| `pub fn edge_count(&self) -> usize` | Returns the number of edges in the graph. | `src/csr/graph_read.rs` |
| `pub fn entity_count(&self) -> usize` | Returns the number of committed entities in the graph. | `src/csr/graph_read.rs` |
| `pub fn entity_exists(&self, id: EntityId) -> bool` | Checks if a committed entity exists in the graph. | `src/csr/graph_read.rs` |
| `pub async fn get_communities_batch( &self, entity_ids: &[EntityId], ) -> Result<HashMap<EntityId, u64>>` | Retrieves community assignments for a batch of entity IDs in a single operation. | `src/csr/graph_read.rs` |
| `pub async fn get_deleted_node_indices(&self) -> HashSet<usize>` | Collects internal node indices of entities that are marked as deleted in storage. | `src/csr/graph_read.rs` |
| `pub async fn is_entity_deleted(&self, entity: EntityId) -> bool` | Prüft ob eine Entity per Tombstone als gelöscht markiert wurde. Nutzt den Key "graph:entity:deleted:{entity_id}" im LSM-Storage (wenn vorha… | `src/csr/graph_read.rs` |
| `pub async fn neighbors(&self, start: EntityId) -> Result<Vec<EntityId>>` | Returns direct 1-hop outgoing neighbors of `start`. | `src/csr/graph_read.rs` |
| `pub async fn pagerank( &self, damping_factor: f32, max_iterations: usize, tolerance: f32, ) -> HashMap<EntityId, f32>` | Calculates PageRank for all entities in the graph using the CSR layout. | `src/csr/graph_read.rs` |
| `pub fn path_rag_at<'a>( &'a self, start_nodes: &'a [EntityId], max_hops: usize, seq_no: u64, ) -> contextra_ports::BoxFuture<'a, Result<Vec<(EntityId, f32)>>>` | Runs PathRAG graph retrieval starting from multiple anchor entities up to `max_hops` at a specific sequence number (`seq_no`). Ref: v17 Tei… | `src/csr/graph_read.rs` |
| `pub async fn personalized_page_rank_with_context_async( &self, seed_nodes: &[EntityId], config: &contextra_types::PprConfig, ctx: &mut crate::PprContext, ) -> Vec<(EntityId, f32)>` | Calculates Personalized PageRank (PPR) using a reusable [`crate::PprContext`] buffer to avoid allocations. | `src/csr/graph_read.rs` |
| `pub async fn remove_entity(&self, tx: TxId, entity: EntityId) -> Result<()>` | Removes an entity node and all its incident (outgoing and incoming) edges from the graph. | `src/csr/graph_read.rs` |
| `pub fn set_communities_batch(&self, assignments: &[crate::CommunityAssignment])` | Sets community assignments in batch for in-memory graph index lookups. | `src/csr/graph_read.rs` |
| `pub fn active_conflict_patterns(&self) -> Vec<crate::consistency_enforcement::ConflictPattern>` | Exposes active conflict patterns from the optional ConsistencyEnforcer. | `src/csr/graph_write.rs` |
| `pub async fn add_edge( self: &Arc<Self>, from: EntityId, to: EntityId, weight: f32, tx_valid_from: Option<TxId>, tx_valid_to: Option<TxId>, business_valid_from: Option<i64>, business_valid_to: Option<i64>, source_doc_id: Option<DocId>, predicate_hash: Option<…` | Inserts an edge directly into the CSR graph with bi-temporal validity, offloading compaction asynchronously if needed. | `src/csr/graph_write.rs` |
| `pub fn commit_super_edge_batch( &self, tombstone_ids: &[crate::hyperedge::HyperEdgeId], new_edges: Vec<crate::hyperedge::HyperEdge>, wal_tx: TxId, ) -> std::result::Result<(), GraphMutationError>` | Batched, atomic commit of hyperedge tombstoning and new superedge insertion. Ein Aufruf = ein `ArcSwap::store` = ein atomarer RCU-Publish f… | `src/csr/graph_write.rs` |
| `pub async fn commit_super_edge_batch_async( &self, tombstone_ids: &[crate::hyperedge::HyperEdgeId], new_edges: Vec<crate::hyperedge::HyperEdge>, wal_tx: TxId, ) -> std::result::Result<(), GraphMutationError>` | Batched, atomic commit of hyperedges with async storage persistence. | `src/csr/graph_write.rs` |
| `pub fn contains_role(&self, name: &str) -> bool` | Checks if a role name exists in the role interner. | `src/csr/graph_write.rs` |
| `pub fn contains_role_id(&self, id: crate::hyperedge::RoleId) -> bool` | Checks if a RoleId exists in the role interner. | `src/csr/graph_write.rs` |
| `pub fn detect_contradiction<D: crate::consistency_enforcement::ContradictionDetector>( &self, detector: &D, a: &EdgeAssertion, b: &EdgeAssertion, ) -> bool` | Detects whether two edge assertions conflict using the provided ContradictionDetector. | `src/csr/graph_write.rs` |
| `pub fn edges_for_doc(&self, doc_id: DocId) -> Vec<(EntityId, EntityId)>` | Returns all edge IDs derived from the given source `DocId`. | `src/csr/graph_write.rs` |
| `pub fn enqueue_cascade_deferred( &self, doc_id: DocId, hyperedge_ids: &[crate::hyperedge::HyperEdgeId], )` | Enqueues deferred hyperedges for background cascade invalidation. | `src/csr/graph_write.rs` |
| `pub fn estimate_compaction_peak_bytes(&self) -> usize` | Returns the estimated peak memory usage during compaction (§6.3), combining residence total bytes with the private structural bytes of a pr… | `src/csr/graph_write.rs` |
| `pub fn estimate_memory(&self, presized: bool) -> MemoryEstimate` | Returns a detailed memory estimate of the graph (§6.3). | `src/csr/graph_write.rs` |
| `pub fn estimate_memory_bytes(&self) -> usize` | Returns the estimated total memory usage of the graph in bytes based on allocation capacities (§6.3). | `src/csr/graph_write.rs` |
| `pub fn flush_reinforcement_buffer( &self, config: &crate::edge_reinforcement::EdgeReinforcementConfig, )` | Flushes buffered reinforcement signals to the CSR graph (F-03). | `src/csr/graph_write.rs` |
| `pub fn get_conflict_pattern( &self, pattern_hash: &[u8; 32], ) -> Option<crate::consistency_enforcement::ConflictPattern>` | Retrieves a conflict pattern by hash if tracked by ConsistencyEnforcer. | `src/csr/graph_write.rs` |
| `pub fn get_hyperedge( &self, id: crate::hyperedge::HyperEdgeId, ) -> Option<Arc<crate::hyperedge::HyperEdge>>` | Retrieves a non-tombstoned hyperedge by its ID if present. | `src/csr/graph_write.rs` |
| `pub fn get_source_doc_id(&self, index: usize) -> Option<DocId>` | Returns the optional source document ID stored at the given edge index in `source_doc_ids`. | `src/csr/graph_write.rs` |
| `pub fn hyperedges_for_doc(&self, doc_id: DocId) -> Vec<crate::hyperedge::HyperEdgeId>` | Returns all non-tombstoned hyperedge IDs derived from `doc_id`. | `src/csr/graph_write.rs` |
| `pub fn hyperedges_for_entity(&self, entity_id: EntityId) -> Vec<crate::hyperedge::HyperEdgeId>` | Returns all non-tombstoned hyperedge IDs associated with `entity_id`. | `src/csr/graph_write.rs` |
| `pub fn inner_read(&self) -> arc_swap::Guard<Arc<GraphInner>>` | Read path contract for RCU snapshot isolation (IP-08): Returns a lock-free reference `arc_swap::Guard<Arc<GraphInner>>` to the current `Gra… | `src/csr/graph_write.rs` |
| `pub async fn insert_edge_auto( self: &Arc<Self>, from: EntityId, to: EntityId, weight: f32, tx_valid_from: Option<TxId>, tx_valid_to: Option<TxId>, business_valid_from: Option<i64>, business_valid_to: Option<i64>, source_doc_id: Option<DocId>, ) -> Result<()>` | Auto-routes direct edge insertion to the appropriate validity-specialized helper. | `src/csr/graph_write.rs` |
| `pub async fn insert_edge_direct( self: &Arc<Self>, from: EntityId, to: EntityId, weight: f32, ) -> Result<()>` | Directly inserts an edge into the CSR graph without staging. | `src/csr/graph_write.rs` |
| `pub async fn insert_edge_direct_with_bitemporal_validity( self: &Arc<Self>, from: EntityId, to: EntityId, weight: f32, tx_valid_from: Option<TxId>, tx_valid_to: Option<TxId>, business_valid_from: Option<i64>, business_valid_to: Option<i64>, source_doc_id: Opt…` | Directly inserts an edge with full bi-temporal validity into the CSR graph without staging. | `src/csr/graph_write.rs` |
| `pub async fn insert_edge_direct_with_validity( self: &Arc<Self>, from: EntityId, to: EntityId, weight: f32, tx_valid_from: Option<TxId>, tx_valid_to: Option<TxId>, ) -> Result<()>` | Directly inserts an edge with validity into the CSR graph without staging. # Weight Validation & Policy Edge weights represent relationship… | `src/csr/graph_write.rs` |
| `pub fn insert_entity_direct(&self, entity: Entity) -> Result<()>` | Directly inserts an entity into the CSR graph without staging. | `src/csr/graph_write.rs` |
| `pub fn insert_hyperedge(&self, edge: crate::hyperedge::HyperEdge)` | Atomically inserts a hyperedge into the graph and updates the entity secondary index. | `src/csr/graph_write.rs` |
| `pub fn insert_hyperedge_direct(&self, hyperedge: crate::hyperedge::HyperEdge)` | — | `src/csr/graph_write.rs` |
| `pub fn intern_role(&self, name: &str) -> crate::hyperedge::RoleId` | Interns a role name and returns its assigned RoleId. | `src/csr/graph_write.rs` |
| `pub fn is_conflict_suppressed(&self, pattern_hash: [u8; 32]) -> bool` | Checks if a pattern hash is suppressed by ConsistencyEnforcer. | `src/csr/graph_write.rs` |
| `pub fn load_edge_direct( &self, from: EntityId, to: EntityId, weight: f32, tx_valid_from: Option<TxId>, tx_valid_to: Option<TxId>, business_valid_from: Option<i64>, business_valid_to: Option<i64>, source_doc_id: Option<DocId>, ) -> Result<()>` | Fügt eine Edge direkt in committed_staged / pending_edges ein (für load_from_storage). Umgeht das TX-Staging, da beim Laden alle Daten bere… | `src/csr/graph_write.rs` |
| `pub fn load_entity_direct(&self, entity: Entity) -> Result<()>` | Fügt eine Entity direkt ein (für load_from_storage). | `src/csr/graph_write.rs` |
| `pub fn max_hyperedge_id(&self) -> u64` | Returns the maximum `HyperEdgeId` present in the current graph snapshot, or 0 if empty. | `src/csr/graph_write.rs` |
| `pub fn new() -> Self` | Creates a new, empty CSR graph with default configuration. | `src/csr/graph_write.rs` |
| `pub fn parent_hyperedges_of( &self, child: crate::hyperedge::HyperEdgeId, ) -> Vec<crate::hyperedge::HyperEdgeId>` | Returns all non-tombstoned parent hyperedge IDs that contain `child` in `child_edge_ids`. The returned list is sorted in ascending order by… | `src/csr/graph_write.rs` |
| `pub fn pending_cascade_queue_len(&self) -> usize` | Returns the number of hyperedges pending in the in-memory cascade queue. | `src/csr/graph_write.rs` |
| `pub async fn persist_hyperedge( &self, tx: TxId, hyperedge: &crate::hyperedge::HyperEdge, ) -> Result<()>` | Persists a hyperedge to storage under `__graph:hyperedge:` and secondary index `__graph:hyperedge_by_entity:`. | `src/csr/graph_write.rs` |
| `pub async fn process_cascade_queue(&self, batch_size: usize, wal_tx: TxId) -> Result<usize>` | Drains and processes up to `batch_size` deferred hyperedges from the cascade queue. Tombstones each hyperedge and all its non-tombstoned an… | `src/csr/graph_write.rs` |
| `pub fn push_cooccurrence_signal(&self, from: EntityId, to: EntityId, co_activation: f32)` | Appends a co-occurrence signal to the internal edge reinforcement buffer (F-03). | `src/csr/graph_write.rs` |
| `pub fn push_traversal_signal(&self, from: EntityId, to: EntityId, path_length: usize)` | Appends a traversal signal to the internal edge reinforcement buffer (F-03). | `src/csr/graph_write.rs` |
| `pub fn reinforcement_signal_counts(&self) -> (usize, usize)` | Returns the counts of pending co-occurrence and traversal signals (F-03). | `src/csr/graph_write.rs` |
| `pub fn relate_n_ary( &self, id: crate::hyperedge::HyperEdgeId, predicate: crate::csr::EdgeType, participants: Vec<crate::hyperedge::RoleBinding>, weight: f32, doc_id: Option<DocId>, ) -> std::result::Result<crate::hyperedge::HyperEdge, GraphMutationError>` | Directly inserts a hyperedge into memory. Erstellt eine n-äre Hyperkante mit H2-Multi-Key-Locking unter Verwending von [`ConsolidationNodes… | `src/csr/graph_write.rs` |
| `pub async fn relate_n_ary_and_persist( &self, tx: TxId, id: crate::hyperedge::HyperEdgeId, predicate: crate::csr::EdgeType, participants: Vec<crate::hyperedge::RoleBinding>, weight: f32, doc_id: Option<DocId>, ) -> std::result::Result<crate::hyperedge::HyperE…` | Creates and persists an n-ary hyperedge with storage persistence. | `src/csr/graph_write.rs` |
| `pub async fn remove_doc( &self, doc_id: DocId, wal_tx: TxId, ) -> Result<Vec<crate::consistency_enforcement::EdgeId>>` | Entfernt ein Dokument aus dem Graph und tombstoniert alle Kanten, die ausschließlich von diesem Dokument belegt wurden (DSGVO-Kaskaden-Lösc… | `src/csr/graph_write.rs` |
| `pub fn resolve_role_string(&self, id: crate::hyperedge::RoleId) -> Option<String>` | Resolves a RoleId to an owned String. | `src/csr/graph_write.rs` |
| `pub fn set_storage(&mut self, storage: Arc<dyn StorageEngine>)` | Sets or replaces the persistent storage handle. | `src/csr/graph_write.rs` |
| `pub fn source_doc_id_at(&self, from: EntityId, to: EntityId) -> Option<DocId>` | Returns the optional source document ID from which the edge (from, to) was derived. | `src/csr/graph_write.rs` |
| `pub fn storage(&self) -> Option<Arc<dyn StorageEngine>>` | Returns a reference to the optional persistent storage handle. | `src/csr/graph_write.rs` |
| `pub fn suggest_tombstone_candidates_for_pattern( &self, matching_edges: &[crate::consistency_enforcement::EdgeId], ) -> Vec<crate::consistency_enforcement::EdgeId>` | Suggests tombstone candidate edges for a conflict pattern using ConsistencyEnforcer. | `src/csr/graph_write.rs` |
| `pub async fn tombstone_edge( &self, edge_id: crate::consistency_enforcement::EdgeId, tx: TxId, ) -> Result<()>` | Tombstoniert eine Kante direkt für eine Transaktions-ID via Cascading-Invalidation (INV-GRAPH-PROV-1). | `src/csr/graph_write.rs` |
| `pub fn tombstone_hyperedge(&self, id: crate::hyperedge::HyperEdgeId, wal_tx: TxId) -> bool` | Atomically tombstones a hyperedge by setting `tx_valid_to = Some(wal_tx)` and removing it from `hyperedge_index` and `doc_to_hyperedges` in… | `src/csr/graph_write.rs` |
| `pub fn with_config(config: CsrGraphConfig) -> Self` | Creates a new, empty CSR graph with specified configuration. | `src/csr/graph_write.rs` |
| `pub fn with_config_and_storage( config: CsrGraphConfig, storage: Arc<dyn StorageEngine>, ) -> Self` | Creates a new CSR graph with configuration and persistent storage. | `src/csr/graph_write.rs` |
| `pub fn with_consistency_enforcer(suppression_threshold: u32) -> Self` | Erstellt CsrGraph mit aktiviertem ConsistencyEnforcer für Widerspruchsprävention (F-04/ADR-073). | `src/csr/graph_write.rs` |
| `pub fn with_storage(storage: Arc<dyn StorageEngine>) -> Self` | Creates a new CSR graph with persistent storage and default config. | `src/csr/graph_write.rs` |
| `pub fn compact(&mut self)` | Compacts pending edges in the delta buffer into the main CSR arrays. | `src/csr/inner.rs` |
| `pub fn get_or_create_index(&mut self, id: EntityId) -> InternalIndex` | — | `src/csr/inner.rs` |
| `pub fn new() -> Self` | — | `src/csr/inner.rs` |
| `pub fn total_bytes(&self) -> usize` | Returns the total memory in bytes (`shared_payload_bytes + private_bytes`). | `src/csr/inner.rs` |
| `pub fn new(target: EntityId, weight: f32) -> Self` | — | `src/csr/types.rs` |
| `pub fn is_edge_visible( tx_valid_from: Option<TxId>, tx_valid_to: Option<TxId>, as_of: TxId, ) -> bool` | Prüft ob eine Kante bezogen auf die Transaktionszeit (MVCC / Systemzeit) zum Zeitpunkt `as_of` sichtbar ist. | `src/csr/visibility.rs` |
| `pub fn is_edge_visible_bitemporal( tx_valid_from: Option<TxId>, tx_valid_to: Option<TxId>, as_of_tx: TxId, business_valid_from: Option<i64>, business_valid_to: Option<i64>, as_of_business: Option<i64>, ) -> bool` | Prüft bi-temporale Sichtbarkeit einer Kante (unabhängige Auswertung von System- und Businesszeit). Business-Zeit wird nur ausgewertet, wenn… | `src/csr/visibility.rs` |
| `pub fn is_edge_visible_business( business_valid_from: Option<i64>, business_valid_to: Option<i64>, business_as_of: i64, ) -> bool` | Prüft ob eine Kante bezogen auf die Businesszeit zum Zeitpunkt `business_as_of` gültig ist. | `src/csr/visibility.rs` |
| `pub fn is_suspicious_tx_id(tx: TxId) -> bool` | Prüft, ob `tx` aus einem verdächtigen (wall-clock-ähnlichen) Bereich stammt. Gibt `true` zurück, wenn `tx` zwischen [`WALLCLOCK_TX_HEURISTI… | `src/csr/visibility.rs` |
| `pub fn cooccurrence_count(&self) -> usize` | Gibt die Anzahl der gepufferten Co-Occurrence-Signale zurück (für Monitoring). | `src/edge_reinforcement_buffer.rs` |
| `pub fn flush_to_graph(&self, graph: &CsrGraph, config: &EdgeReinforcementConfig)` | Wendet alle gepufferten Signale auf den CSR-Graphen an und leert den Buffer. Wird ausschließlich vom `MaintenanceScheduler::run_tick()` auf… | `src/edge_reinforcement_buffer.rs` |
| `pub fn new() -> Self` | — | `src/edge_reinforcement_buffer.rs` |
| `pub fn push_cooccurrence(&self, from: EntityId, to: EntityId, co_activation: f32)` | Fügt ein Co-Occurrence-Signal hinzu (ko-aktivierte Entitäten in derselben Retrieval-Session). Kann von mehreren Threads gleichzeitig aufger… | `src/edge_reinforcement_buffer.rs` |
| `pub fn push_traversal(&self, from: EntityId, to: EntityId, path_length: usize)` | Fügt ein Traversal-Signal hinzu (traversierte Kante mit Pfadlänge). | `src/edge_reinforcement_buffer.rs` |
| `pub fn traversal_count(&self) -> usize` | Gibt die Anzahl der gepufferten Traversal-Signale zurück (für Monitoring). | `src/edge_reinforcement_buffer.rs` |
| `pub fn ingest_extracted_relations( graph: &CsrGraph, relations: &[ExtractedRelation], min_confidence_threshold: f32, doc_id: Option<DocId>, ) -> Result<Vec<HyperEdge>>` | Wandelt extrahierte Relationen oberhalb der Konfidenzschwelle `min_confidence` in Hyperkanten auf dem `CsrGraph` um (`graph.relate_n_ary`).… | `src/entity_extraction.rs` |
| `pub fn new( source: impl Into<String>, target: impl Into<String>, relation_type: impl Into<String>, confidence: f32, ) -> Self` | Erstellt eine neue `ExtractedRelation`. | `src/entity_extraction.rs` |
| `pub fn new(min_confidence: f32) -> Self` | Erstellt einen neuen `RuleBasedEntityExtractor` mit der angegebenen Mindest-Konfidenz-Schwelle. | `src/entity_extraction.rs` |
| `pub fn acquire(entities: &[EntityId]) -> Self` | Erstellt einen neuen `ConsolidationNodesGuard` durch kanonische Sortierung und Deduplizierung der übergebenen `EntityId`s. | `src/hyperedge.rs` |
| `pub fn contains_id(&self, id: RoleId) -> bool` | Prüft, ob eine [`RoleId`] im Interner vorhanden ist. | `src/hyperedge.rs` |
| `pub fn contains_role(&self, name: &str) -> bool` | Prüft, ob ein Rollenname im Interner vorhanden ist. | `src/hyperedge.rs` |
| `pub fn deserialize(bytes: &[u8]) -> Result<Self>` | Deserialisiert eine Hyperkante aus einem `bincode`-Byte-Slice. | `src/hyperedge.rs` |
| `pub fn entities(&self) -> &[EntityId]` | Gibt die kanonisch sortierten und deduplizierten Entitäts-IDs zurück. | `src/hyperedge.rs` |
| `pub fn get_or_intern(&self, name: &str) -> RoleId` | Interniert einen Rollennamen und gibt die zugehörige [`RoleId`] zurück. Falls der Rollenname bereits interniert wurde, wird die bestehende… | `src/hyperedge.rs` |
| `pub fn is_empty(&self) -> bool` | Gibt `true` zurück, falls der Interner leer ist. | `src/hyperedge.rs` |
| `pub fn len(&self) -> usize` | Gibt die Anzahl der internierten Rollen zurück. | `src/hyperedge.rs` |
| `pub fn new(role: RoleId, entity: EntityId) -> Self` | Erstellt eine neue `RoleBinding`. | `src/hyperedge.rs` |
| `pub fn new( id: HyperEdgeId, predicate: EdgeType, participants: impl Into<Arc<[RoleBinding]>>, weight: f32, ) -> Self` | Erstellt eine neue `HyperEdge`. | `src/hyperedge.rs` |
| `pub fn new(edge: &'a HyperEdge) -> Self` | Creates a new `HyperEdgeView` for the given hyperedge. | `src/hyperedge.rs` |
| `pub fn new() -> Self` | Erstellt einen neuen, leeren [`RoleInterner`]. | `src/hyperedge.rs` |
| `pub fn primary_participants(&self, count: usize) -> ArcSlice<RoleBinding>` | Returns a zero-copy sub-view containing the primary participants (first `count` bindings). | `src/hyperedge.rs` |
| `pub fn resolve<F, R>(&self, id: RoleId, f: F) -> Option<R> where F: FnOnce(&str) -> R,` | Löst eine [`RoleId`] im Lesepfad zero-copy über eine Closure auf. Es entsteht keine neue String-Allokation pro Aufruf im Lesepfad. | `src/hyperedge.rs` |
| `pub fn resolve_string(&self, id: RoleId) -> Option<String>` | Löst eine [`RoleId`] in ein owned `String` auf. | `src/hyperedge.rs` |
| `pub fn serialize(&self) -> Result<Vec<u8>>` | Serialisiert die Hyperkante via `bincode`. | `src/hyperedge.rs` |
| `pub fn slice_participants( &self, range: impl std::ops::RangeBounds<usize>, ) -> ArcSlice<RoleBinding>` | Creates a sub-view of participants without copying. | `src/hyperedge.rs` |
| `pub fn sort_dedup_entities(entities: &[EntityId]) -> Vec<EntityId>` | Sortiert und dedupliziert eine Menge von [`EntityId`]s in kanonischer (aufsteigender) Reihenfolge. Garantiert eine totale Ordnung zur Verme… | `src/hyperedge.rs` |
| `pub fn star_weight(weight: f32, cardinality: usize) -> f32` | Berechnet das Kantengewicht in der Stern-Expansion nach Konvention K (§6.6 H6). Formula: `w_star = 2 * w / (|e| - 1)` for hyperedge weight… | `src/hyperedge.rs` |
| `pub fn try_acquire(entities: &[EntityId]) -> std::result::Result<Self, GraphMutationError>` | Versucht kanonischen Erwerb der Entitäten-Lock-Reihenfolge (`try_lock`-Muster). Gibt `Ok(ConsolidationNodesGuard)` zurück mit kanonisch sor… | `src/hyperedge.rs` |
| `pub fn validate(&self) -> std::result::Result<(), GraphMutationError>` | Validiert die Invarianten der Hyperkante. # Invarianten - Mindestens 2 Teilnehmer (`participants.len() >= 2`), sonst degeneriert zur binäre… | `src/hyperedge.rs` |
| `pub fn view(&self) -> HyperEdgeView<'_>` | Erstellt eine Zero-Copy-Teilsicht (`HyperEdgeView`) auf diese Hyperkante. | `src/hyperedge.rs` |
| `pub fn with_business_validity( mut self, valid_from: Option<i64>, valid_to: Option<i64>, ) -> Self` | Setzt die fachliche Gültigkeit der Hyperkante. | `src/hyperedge.rs` |
| `pub fn with_child_edge_ids(mut self, ids: impl Into<Arc<[HyperEdgeId]>>) -> Self` | Setzt die untergeordneten Hyperkanten-IDs (LeanRAG Super-Kanten). Sortiert und dedupliziert die IDs deterministisch und entfernt eine etwai… | `src/hyperedge.rs` |
| `pub fn with_entities<F, R>(&self, f: F) -> R where F: FnOnce(&[EntityId]) -> R,` | Führt eine Closure über die kanonisch sortierten Entitäts-IDs aus. | `src/hyperedge.rs` |
| `pub fn with_source_doc_id(mut self, source_doc_id: Option<DocId>) -> Self` | Setzt die Quell-Dokument-ID der Hyperkante. | `src/hyperedge.rs` |
| `pub fn with_tx_validity(mut self, valid_from: Option<TxId>, valid_to: Option<TxId>) -> Self` | Setzt die Transaktionsgültigkeit der Hyperkante. | `src/hyperedge.rs` |
| `pub fn compute_co_occurrence_candidates( entities_per_document: &[(DocId, Vec<EntityId>)], min_co_occurrence: usize, ) -> Vec<HyperEdgeCandidate>` | Computes hyperedge candidates from document entity co-occurrences. Purely deterministic function (no `Rng`, P28). For each document, dedupl… | `src/hyperedge_suggest.rs` |
| `pub async fn validate_candidates_with_llm( candidates: &[HyperEdgeCandidate], entity_labels: &dyn Fn(EntityId) -> Option<String>, generator: &dyn LlmTextGenerator, max_llm_calls_per_cycle: usize, ) -> Result<Vec<ValidatedHyperEdgeCandidate>, GraphMutationErro…` | Validates hyperedge candidates using an [`LlmTextGenerator`]. For each candidate (up to `max_llm_calls_per_cycle`), queries the LLM to conf… | `src/hyperedge_suggest.rs` |
| `pub fn find_all_paths(&self, source: EntityId) -> Vec<GraphPath>` | Findet Pfade von einem Ankerknoten zu allen erreichbaren Knoten innerhalb von max_hops. | `src/path_rag/mod.rs` |
| `pub fn find_path(&self, source: EntityId, target: EntityId) -> Option<GraphPath>` | Findet den optimalen Pfad zwischen source und target via bidirektionalem Dijkstra. Gibt None zurück wenn kein Pfad innerhalb max_hops exist… | `src/path_rag/mod.rs` |
| `pub fn forward_push_ppr<G: PathGraph>( graph: &G, seeds: &[EntityId], params: &PprParams, ) -> AHashMap<EntityId, f32>` | Andersen-Chung-Lang Forward-Push PPR implementation with Hyperedge Expansion (§6.6, H3). Runtime is $O(1/(\alpha \cdot \epsilon))$, indepen… | `src/path_rag/mod.rs` |
| `pub fn max_hops(&self) -> usize` | — | `src/path_rag/mod.rs` |
| `pub fn new(graph: G, max_hops: usize, sufficiency_threshold: f64) -> Self` | — | `src/path_rag/mod.rs` |
| `pub fn sufficiency_check(&self, path: &GraphPath) -> bool` | Sufficiency-Gate: Filtert Pfade unter Konfidenz-Schwelle. Kritisch für Precision (arXiv:2506.00610). | `src/path_rag/mod.rs` |
| `pub fn sufficiency_threshold(&self) -> f64` | — | `src/path_rag/mod.rs` |
| `pub fn to_rrf_signal(&self, paths: &[GraphPath]) -> Vec<(DocId, f32)>` | Konvertiert gefilterte Pfade in RRF-kompatibles Signal. Nur Knoten aus Pfaden die sufficiency_check() passiert haben. NOTE (Type-Punning As… | `src/path_rag/mod.rs` |
| `pub fn with_config(graph: G, config: PathRAGConfig) -> Self` | — | `src/path_rag/mod.rs` |
| `pub fn with_defaults(graph: G) -> Self` | — | `src/path_rag/mod.rs` |
| `pub fn new(inner: &'a GraphInner, as_of_tx: TxId) -> Self` | Creates a new `SnapshotPathGraph` borrowing `inner` at transaction `as_of_tx`. | `src/path_rag/snapshot.rs` |
| `pub fn compute_percolation_health( active_node_count: usize, active_edge_count: usize, // nach Tombstone-Abzug ) -> Option<f32>` | Berechnet aktuelle Perkolations-Gesundheit φ(t). Gibt `None` zurück wenn zu wenige Knoten vorhanden (< 10) — Perkolation nicht sinnvoll. | `src/percolation.rs` |
| `pub async fn find_rebonding_candidates<G: contextra_ports::GraphIndex>( graph: &G, embeddings: &HashMap<EntityId, Vec<f32>>, config: &PercolationConfig, ) -> Vec<(EntityId, EntityId, f32)>` | Findet Kandidaten-Paare für Re-Bonding. Gibt `(EntityId, EntityId, similarity)`-Tupel zurück, sortiert absteigend nach Similarity. | `src/percolation.rs` |
| `pub fn should_trigger_rebonding(health: f32, config: &PercolationConfig) -> bool` | Prüft ob Re-Bonding-Pass ausgelöst werden soll. | `src/percolation.rs` |
| `pub fn compute_ppr_with_apprh_selector( inner: &GraphInner, seed_nodes: &[EntityId], config: &PprConfig, deleted_nodes: &DeletedView, ctx: &mut PprContext, selector: &crate::apprh::ApprhSelector, apprh_params: &crate::apprh::ApprhParams, ) -> Vec<(EntityId, f…` | Calculates Personalized PageRank using APPRH selector routing and hysteresis gate evaluation (P1.3). | `src/ppr.rs` |
| `pub fn contains(&self, idx: usize) -> bool` | Checks if internal node index `idx` is marked as deleted. | `src/ppr.rs` |
| `pub fn is_empty(&self) -> bool` | Returns `true` if this view contains no deleted nodes. | `src/ppr.rs` |
| `pub fn len(&self) -> usize` | Returns the number of deleted node indices contained in this view. | `src/ppr.rs` |
| `pub fn new() -> Self` | Creates an empty PPR context buffer. | `src/ppr.rs` |
| `pub fn cheapest_strategy(&self) -> PprAlgorithm` | Selects the cheapest algorithm based on estimated cost. Tie-break rule: If `forward_push == power_iteration`, `ForwardPush` is deterministi… | `src/ppr/cost.rs` |
| `pub fn clear(&self)` | Clears all recorded samples. | `src/ppr/cost.rs` |
| `pub fn estimate_ppr_cost( stats: Option<&GraphStats>, seed_count: usize, config: &PprConfig, ) -> (PprCostEstimate, PprAlgorithm)` | Estimates computational costs (in estimated edge accesses) for PPR algorithms. # Fallback If `stats` is `None` or `stats.node_count == 0`,… | `src/ppr/cost.rs` |
| `pub fn evaluate_recommended_multiplier(&self) -> Option<f64>` | Evaluates the recommended cost multiplier across recorded samples. | `src/ppr/cost.rs` |
| `pub fn new(node_count: usize, edge_count: usize) -> Self` | Creates a new `GraphStats` from explicit counts. | `src/ppr/cost.rs` |
| `pub fn new() -> Self` | Creates a new empty `PprCostCalibrator`. | `src/ppr/cost.rs` |
| `pub fn recommended_multiplier(&self) -> Option<f64>` | Derives a recommended cost multiplier (ratio of measured edge accesses to estimated cost). Returns `None` if no samples have been recorded.… | `src/ppr/cost.rs` |
| `pub fn record(&self, sample: PprCostSample)` | Records a new calibration sample comparing estimated cost vs measured edge accesses. | `src/ppr/cost.rs` |
| `pub fn sample_count(&self) -> usize` | Returns the total number of recorded calibration samples. | `src/ppr/cost.rs` |
| `pub fn log_tl_hfd_discrepancy_tracing( baseline: &AHashMap<EntityId, f32>, candidate: &AHashMap<EntityId, f32>, context_id: u64, )` | Helper to log TL-HFD vs baseline discrepancy using tracing shadow sink. | `src/ppr/shadow_hook.rs` |
| `pub fn log_tl_hfd_vs_baseline_discrepancy( baseline: &AHashMap<EntityId, f32>, candidate: &AHashMap<EntityId, f32>, context_id: u64, sink: &dyn ShadowSink<f64>, )` | Protokolliert die mittlere absolute Differenz (MAE) zwischen den Baseline- und TL-HFD-Kandidaten-PPR-Score-Maps über die gegebene Diskrepan… | `src/ppr/shadow_hook.rs` |
| `pub fn is_exhausted(&self) -> bool` | Returns whether the candidate stream is exhausted. | `src/ppr_stream.rs` |
| `pub fn new(graph: &'a G, seeds: impl Into<Vec<EntityId>>, config: PprConfig) -> Self` | Creates a new candidate stream for Personalized PageRank starting from `seeds`. | `src/ppr_stream.rs` |
| `pub async fn next_batch(&mut self) -> Result<Vec<(EntityId, f32)>>` | Fetches the next batch of candidate entity-score pairs (up to `batch_size`). Returns an empty `Vec` when exhausted. | `src/ppr_stream.rs` |
| `pub async fn next_batch_docs<F>(&mut self, mut resolve: F) -> Result<Vec<DocId>> where F: FnMut(EntityId) -> Option<DocId>,` | Fetches the next batch of resolved [`DocId`]s (up to `batch_size`). Entities for which `resolve` returns `None` are skipped. | `src/ppr_stream.rs` |
| `pub async fn next_entity(&mut self) -> Result<Option<EntityId>>` | Fetches the next single candidate entity ID. Returns `Ok(None)` when exhausted. | `src/ppr_stream.rs` |
| `pub fn with_batch_size(mut self, n: usize) -> Self` | Sets a custom batch size (minimum 1). | `src/ppr_stream.rs` |
| `pub fn with_exclude_seeds(mut self, exclude: bool) -> Self` | Specifies whether seed entities themselves should be excluded from the yielded candidate stream. | `src/ppr_stream.rs` |
| `pub fn yielded(&self) -> usize` | Returns the total number of items yielded so far. | `src/ppr_stream.rs` |
| `pub fn edges_for_doc(&self, doc_id: DocId) -> Vec<EdgeId>` | Gibt alle Kanten zurück, die von diesem Dokument abhängen. | `src/provenance.rs` |
| `pub fn new(edge_id: EdgeId, source_doc_ids: Vec<DocId>, created_at_tx: TxId) -> Self` | — | `src/provenance.rs` |
| `pub fn new() -> Self` | — | `src/provenance.rs` |
| `pub fn provenance_for_edge(&self, edge_id: EdgeId) -> Option<EdgeProvenance>` | Gibt das `EdgeProvenance` einer Kante zurück, falls vorhanden. | `src/provenance.rs` |
| `pub fn record(&self, doc_id: DocId, edge_id: EdgeId)` | Registriert die Abhängigkeit einer Kante von einem Dokument. | `src/provenance.rs` |
| `pub fn record_provenance(&self, provenance: &EdgeProvenance)` | Registriert einen EdgeProvenance-Eintrag und indiziert alle darin enthaltenen source_doc_ids. | `src/provenance.rs` |
| `pub fn remove_doc(&self, doc_id: DocId) -> Vec<EdgeId>` | Entfernt ein Dokument aus dem Herkunftsindex (DSGVO-Kaskaden-Löschung). # Semantik (INV-GRAPH-PROV-1) Beim Löschen von Dokument D: (a) D wi… | `src/provenance.rs` |
| `pub fn active_head(&self) -> RwLockReadGuard<'_, NodeIdx>` | — | `src/session_dag.rs` |
| `pub fn active_head(&self) -> RwLockReadGuard<'_, NodeIdx>` | — | `src/session_dag.rs` |
| `pub fn active_head(&self) -> NodeIdx` | Returns the active head node index. | `src/session_dag.rs` |
| `pub fn active_head_write(&self) -> RwLockWriteGuard<'_, NodeIdx>` | — | `src/session_dag.rs` |
| `pub fn active_head_write(&self) -> RwLockWriteGuard<'_, NodeIdx>` | — | `src/session_dag.rs` |
| `pub fn append_step( &self, prompt: String, response: String, snapshot_tx_id: Option<TxId>, tool_outputs: Vec<String>, label: &str, ) -> Result<NodeIdx>` | Appends a step to the current active head node atomically under a single write lock. | `src/session_dag.rs` |
| `pub fn branch_from( &self, parent_node: NodeIdx, prompt: String, response: String, snapshot_tx_id: Option<TxId>, tool_outputs: Vec<String>, label: &str, ) -> Result<NodeIdx>` | Branches from an arbitrary prior node (Branch-Erstellung ab beliebigem Vorgänger-Knoten). # Errors Returns `Err(ContextraError::InvalidInpu… | `src/session_dag.rs` |
| `pub fn children_of(&self, node_idx: NodeIdx) -> Vec<NodeIdx>` | Returns all direct children of a given node. | `src/session_dag.rs` |
| `pub fn edges(&self) -> RwLockReadGuard<'_, Vec<DagEdge>>` | Erwirbt den edges-Lock. Kann NUR aufgerufen werden, wenn bereits ein NodesGuard existiert -> Lock-Reihenfolge ist durch die Typsignatur erz… | `src/session_dag.rs` |
| `pub fn edges(&self) -> RwLockReadGuard<'_, Vec<DagEdge>>` | — | `src/session_dag.rs` |
| `pub fn edges_write(&self) -> RwLockWriteGuard<'_, Vec<DagEdge>>` | — | `src/session_dag.rs` |
| `pub fn edges_write(&self) -> RwLockWriteGuard<'_, Vec<DagEdge>>` | — | `src/session_dag.rs` |
| `pub fn get_node(&self, node_idx: NodeIdx) -> Option<AgentStateNode>` | Retrieves a reference to a specific node by index if it exists. | `src/session_dag.rs` |
| `pub async fn load<S: StorageEngine + ?Sized>(storage: &S, namespace: &str) -> Result<Self>` | Loads and reconstructs a `SessionBranchTree` from storage for the specified namespace. | `src/session_dag.rs` |
| `pub fn lock_nodes(&self) -> NodesGuard<'_>` | Einziger öffentlicher Einstiegspunkt, um nodes zu lesen/sperren — ersetzt jeden direkten Zugriff auf `self.nodes` von außerhalb des Moduls. | `src/session_dag.rs` |
| `pub fn lock_nodes_write(&self) -> NodesWriteGuard<'_>` | Einziger öffentlicher Einstiegspunkt für exklusiven Schreibzugriff auf nodes. | `src/session_dag.rs` |
| `pub fn new(root_prompt: String, root_response: String) -> Self` | Creates a new DAG with a root node (step_id 0). | `src/session_dag.rs` |
| `pub fn node_count(&self) -> usize` | Returns the total number of nodes in the DAG. | `src/session_dag.rs` |
| `pub fn nodes(&self) -> &HashMap<NodeIdx, AgentStateNode>` | — | `src/session_dag.rs` |
| `pub fn nodes(&self) -> &HashMap<NodeIdx, AgentStateNode>` | — | `src/session_dag.rs` |
| `pub fn nodes_mut(&mut self) -> &mut HashMap<NodeIdx, AgentStateNode>` | — | `src/session_dag.rs` |
| `pub fn path_to_head(&self) -> Vec<AgentStateNode>` | Reconstructs the linear conversation path from root to current active head. | `src/session_dag.rs` |
| `pub async fn save<S: StorageEngine + ?Sized>( &self, storage: &S, namespace: &str, tx: TxId, ) -> Result<()>` | Persists all nodes, edges, active head, and next_id of the Session-DAG to the storage engine under the given namespace prefix. | `src/session_dag.rs` |
| `pub fn set_active_head(&self, node_idx: NodeIdx) -> Result<()>` | Sets the active head to an existing node. | `src/session_dag.rs` |
| `pub fn run_diffusion<G: PathGraph>( graph: &G, seeds: &[EntityId], params: &TlHfdParams, ) -> Result<AHashMap<EntityId, f32>, TlHfdError>` | Runs Thresholded Local Hyper-Flow Diffusion over a generic [`PathGraph`]. # Invariants - Deterministic execution: Floating-point operations… | `src/tl_hfd/diffusion.rs` |
| `pub fn compute_lovasz_extension( participants: &[EntityId], get_x: impl Fn(EntityId) -> f32, ) -> (f32, Option<EntityId>, Option<EntityId>)` | Evaluates Lovász extension $f_e(x) = \max_{u \in e} x_u - \min_{u \in e} x_u$ and subgradient $s_{e, u}(x)$ for participants. Returns `(f_e… | `src/tl_hfd/lovasz.rs` |
| `pub fn truncate_participants( participants: &[EntityId], get_x: impl Fn(EntityId) -> f32, max_sort_size: usize, ) -> Vec<EntityId>` | Truncates hyperedge participants to at most `max_sort_size` top x-value participants plus the min x-value participant. # Determinism Determ… | `src/tl_hfd/lovasz.rs` |
| `pub fn evaluate_thresholded_local_hfd( &self, seeds: &[EntityId], params: &TlHfdParams, ) -> Result<AHashMap<EntityId, f32>, TlHfdError>` | Evaluates TL-HFD over CsrGraph. | `src/tl_hfd/mod.rs` |
| `pub fn thresholded_local_hfd( &self, seeds: &[EntityId], params: &TlHfdParams, ) -> Result<AHashMap<EntityId, f32>, TlHfdError>` | Computes Thresholded Local Hyper-Flow Diffusion (TL-HFD, Spec §21.1) over the graph. # Invariants Runs exclusively in the shadow path and n… | `src/tl_hfd/mod.rs` |
| `pub fn tl_hfd_local<G: PathGraph>( graph: &G, seeds: &[EntityId], params: &TlHfdParams, ) -> Result<AHashMap<EntityId, f32>, TlHfdError>` | Computes Thresholded Local Hyper-Flow Diffusion (TL-HFD) over a generic [`PathGraph`]. # Invariants Runs exclusively in the shadow path and… | `src/tl_hfd/mod.rs` |
| `pub fn validate(&self) -> Result<(), TlHfdError>` | Validates configuration parameters against spec constraints. # Errors Returns [`TlHfdError::InvalidParameter`] if any parameter violates it… | `src/tl_hfd/params.rs` |
| `pub fn shadow_compare_forward_push_vs_tl_hfd<G: PathGraph>( graph: &G, seeds: &[EntityId], ppr: &PprParams, tl: &TlHfdParams, top_k: usize, ) -> Result<ShadowComparison, TlHfdError>` | Computes shadow comparison between Forward-Push PPR and TL-HFD. # Invariants - Authoritative result is Forward-Push PPR. - Emits `tracing::… | `src/tl_hfd/shadow.rs` |


---

## `contextra-rank`

| Attribut | Wert |
|---|---|
| **Ring** | 0 — Ring 0 — Foundation & Pure Numerics/Types |
| **Status** | In Migration |
| **Zweck** | 4-Signal-Retrieval-Fusion, Score-Kalibrierung und Drift-Messung. |
| **Unsafe-Code** | Nein (`#![forbid(unsafe_code)]`) |
| **Interne Abhängigkeiten** | — |
| **Externe Abhängigkeiten (Auswahl)** | — |

### Architekturbeschreibung

`contextra-rank`: 4-Signal Fusion, Isotonic & Platt Calibration, and Drift Detection.

### Modul- und Re-Export-Oberfläche (lib.rs)

```rust
pub mod calibration;
pub mod drift;
pub mod explain;
pub mod fusion;
pub mod dibud;
pub use calibration::{ AdaptiveConformalCalibrator, ConformalCalibrator, ConformalError, IsotonicCalibrator, PlattScaler, };
pub use dibud::{ fuse_exact_prefix, fuse_exact_prefix_async, BudgetedChannel, DiBudFusionState, DiBudOutcome, DiBudStep, FusionBudget, };
pub use drift::{DriftDetector, DriftStatus};
pub use explain::{explain, ExplanationEntry, RetrievalExplanation};
pub use fusion::{ apply_resonance_bonus, fuse_search_results_with_strategy, g, modulate_and_renormalize_weights, reciprocal_rank_fusion, score_normalized_fusion_with_options, weighted_reciprocal_rank_fusion, weighted_reciprocal_rank_fusion_mrrf, weighted_reciprocal_rank_fusion_with_options, weighted_reciprocal_rank_fusion_with_priority, weights_to_signal_factors, BoundedTopK, GlobalFusionConfig, GlobalFusionStrategy, MetadataMergePriority, ProvenanceBuilder, ProvenanceRecord, ResonanceConfig, SearchResult, SignalCalibrationContext, SignalContribution, SignalKind, };
```

### Feature-Flags

| Flag | Aktiviert |
|---|---|
| `dibud` | `[]` |

### Schnittstellen — Public Traits (1)

**`ConformalCalibrator`** — *Trait for deterministic, online conformal calibration and threshold estimation.* (`src/calibration/conformal.rs`)

```rust
pub trait ConformalCalibrator {
/// Deterministically updates the threshold given a non-conformity score and propensity weight.
    ///
    /// # Parameters
    /// - `non_conformity_score`: The non-conformity score $S_t$ of the current sample.
    /// - `propensity_weight`: The propensity weight $w_t > 0$ correcting for covariate shift.
    ///
    /// # Returns
    /// - `Ok(true)` if the update altered the threshold value.
    /// - `Ok(false)` if the threshold value remained unchanged.
    /// - `Err(ConformalError)` if any input parameter is non-finite or non-positive.
    fn update(
        &mut self,
        non_conformity_score: f32,
        propensity_weight: f32,
    ) -> Result<bool, ConformalError>;

    /// Returns the currently valid threshold $\hat{q}$.
    fn threshold(&self) -> f32;
}
```

### Öffentliche Datentypen — Structs (19)

| Struct | Felder / Form | Beschreibung | Datei |
|---|---|---|---|
| `AdaptiveConformalCalibrator` | `{ alpha: f32, threshold: f32, learning_rate: f32, min_threshold: f32, max_threshold: f32, observation_count: u64, fingerprint: Option<ConfigFingerprint>, }` | Adaptive Conformal Calibrator for quantile-based online threshold estimation under covariate shift. Implements online stochastic gradient updates on… | `src/calibration/conformal.rs` |
| `BoundedTopK` | `{ heap: std::collections::BinaryHeap<T>, capacity: usize, }` | Size-bounded min-heap for Top-K candidate selection during Reciprocal Rank Fusion. | `src/fusion/topk.rs` |
| `DiBudFusionState` | `{ channel_depths: [usize; 3], exhausted: [bool; 3], accesses: usize, observed_docs: AHashMap<DocId, ObservedDocState>, er_cache: AHashMap<DocId, f32>, provenance… }` | Core state machine tracking channel depths, observed document scores, and prefix certification. | `src/dibud/state.rs` |
| `DiBudOutcome` | `{ /// Ranked candidate document IDs sorted descending by lower bound $F^-(x)$ (tie-breaking by ascending `DocId`). pub ranked: Vec<DocId>, /// Number of prefix e… }` | Final outcome of a DiBud fusion execution. | `src/dibud/types.rs` |
| `DriftDetector` | `{ window_size: usize, threshold: f32, baseline_mean: Option<f32>, recent_scores: VecDeque<f32>, }` | Drift detector monitoring score shifts over sliding windows. | `src/drift.rs` |
| `ExplanationEntry` | `{ /// Identified search signal type. pub signal: SignalKind, /// Raw unnormalized score or distance. pub raw_score: f32, /// 1-based rank in the signal's result… }` | Breakdown entry representing the contribution of a single search signal. | `src/explain.rs` |
| `FusionBudget` | `{ /// Maximum total channel access budget (reads). pub max_total_accesses: usize, /// Minimum required certified prefix length before stopping. pub min_certified… }` | Dynamic budget configuration for DiBud fusion. | `src/dibud/types.rs` |
| `GlobalFusionConfig` | `{ /// Maximum number of community nodes to include in the final result set. pub max_community_nodes: Option<usize>, /// Minimum size (member count) required for… }` | Configuration for `GlobalFusionStrategy`. | `src/fusion/global.rs` |
| `GlobalFusionStrategy` | `{ config: GlobalFusionConfig, }` | Community-aware fusion strategy for corpus-wide topic retrieval (LeanRAG-backed). | `src/fusion/global.rs` |
| `IsotonicCalibrator` | `{ observations: VecDeque<(f32, bool)>, warmup_required: u32, max_observations: usize, cached_model: Option<Vec<(f32, f32)>>, // (max_score_in_block, calibrated_p… }` | Isotonic Calibrator using Pool-Adjacent Violators Algorithm. | `src/calibration/isotonic.rs` |
| `PlattScaler` | `{ a: f32, b: f32, fingerprint: Option<ConfigFingerprint>, }` | Platt Scaler for parametric logit/score calibration. | `src/calibration/platt.rs` |
| `ProvenanceBuilder` | `{ vector_distance: Option<f32>, vector_rank: Option<u32>, vector_weight: Option<f32>, bm25_score: Option<f32>, bm25_rank: Option<u32>, text_weight: Option<f32>,… }` | Fluent builder for constructing `ProvenanceRecord` instances. | `src/fusion/provenance.rs` |
| `ProvenanceRecord` | `{ /// Distance score from vector search signal. #[serde(default, skip_serializing_if = "Option::is_none")] pub vector_distance: Option<f32>, /// BM25 score from… }` | Provenance record for fused search result. | `src/fusion/types.rs` |
| `ResonanceConfig` | `{ /// Exponent β for coherence bonus. Default: 0.5. pub beta: f32, /// Boost strength γ. Default: 0.3. pub gamma: f32, }` | Configuration for Resonance Coherence Bonus. | `src/fusion/resonance.rs` |
| `RetrievalExplanation` | `{ /// Individual signal contribution entries, sorted descending by `contribution_share`. pub entries: Vec<ExplanationEntry>, /// Dominant search signal contribut… }` | Structured explanation detailing why a document was retrieved with its given rank. | `src/explain.rs` |
| `SearchReport` | `{ /// List of signals that failed during query execution when using Degrade policy. pub degraded_signals: Vec<Signal>, /// Warnings captured during query process… }` | Execution report for search query execution. | `src/fusion/types.rs` |
| `SearchResult` | `{ /// Document or memory entry identifier. pub id: String, /// Fused score value. pub score: f32, /// Document metadata payload. pub metadata: Option<serde_json:… }` | Individual search result candidate for fusion. | `src/fusion/types.rs` |
| `SignalCalibrationContext` | `{ /// Signal identifier matching the result set signal name (e.g., "vector", "text", "graph"). pub signal_name: String, /// Indicates whether the calibrator for… }` | Calibration and drift context for a search signal during mRRF weight modulation. | `src/fusion/signal.rs` |
| `SignalContribution` | `{ /// Raw score before normalization/fusion. pub raw_score: f32, /// 1-based rank in the signal's result list. pub rank: u32, /// Calculated RRF or normalized co… }` | Signal contribution details. | `src/fusion/types.rs` |

### Öffentliche Datentypen — Enums (8)

| Enum | Varianten | Beschreibung | Datei |
|---|---|---|---|
| `BudgetedChannel` | `/// Vector (semantic) search channel (index 0). Vector = 0, /// Text (BM25) search channel (index 1). Text = 1, /// Graph search channel (index 2). Graph = 2,` | Identifies the channels managed by DiBud fusion budget. Order of variants is defined such that `Vector < Text < Graph`, matching the tie-breaking pri… | `src/dibud/types.rs` |
| `ConformalError` | `/// Target miscoverage level `alpha` is not finite or outside (0.0, 1.0). #[error("Invalid target miscoverage alpha: {0} (must be finite and in (0.0, 1.0))")] InvalidAlpha(f32), /// Propensity weight…` | Error type for conformal calibration and threshold updates. | `src/calibration/conformal.rs` |
| `DiBudStep` | `/// Request reading the next item from the specified channel. Poll(BudgetedChannel), /// Fusion is complete; outcome contains certified prefix and ranked documents. Done(DiBudOutcome),` | Represents the step action requested by `DiBudFusionState::next_request`. | `src/dibud/state.rs` |
| `DriftStatus` | `/// Distribution is stable. Stable { /// Mean score shift relative to baseline. mean_shift: f32, }, /// Drift detected exceeding configured tolerance threshold. DriftDetected { /// Mean score shift r…` | Drift status summary representation. | `src/drift.rs` |
| `MetadataMergePriority` | `/// Vector metadata is processed first (default behavior). Order: Vector, Text, Graph. #[default] VectorFirst, /// Text metadata is processed first. Order: Text, Vector, Graph. TextFirst, /// Graph m…` | Configures signal priority order for metadata merging during Reciprocal Rank Fusion. | `src/fusion/signal.rs` |
| `Signal` | `/// Dense vector similarity signal. Vector, /// Full-text BM25 keyword signal. Text, /// Graph entity relationship signal. Graph,` | Query retrieval signal identifier. | `src/fusion/types.rs` |
| `SignalKind` | `/// Vector (semantic k-NN) search signal. Vector, /// Text (BM25 keyword) search signal. Bm25, /// Graph (traversal / PageRank) search signal. Graph, /// Cross-encoder reranking signal. Rerank,` | Search signal kind used during multi-signal fusion. | `src/explain.rs` |
| `SignalKind` | `/// Vector (semantic k-NN) search signal. Vector, /// Text (BM25 keyword) search signal. Text, /// Graph (traversal / PageRank) search signal. Graph, /// Edge-reinforcement weight signal / Bandit rec…` | Identifies the kind of search signal used during fusion. | `src/fusion/signal.rs` |

### Typ-Aliase (1)

| Alias | Definition | Beschreibung |
|---|---|---|
| `FusedScore` | `type FusedScore= SearchResult` | Type alias for fused score output item. |

### Konstanten (5)

| Konstante | Typ | Beschreibung |
|---|---|---|
| `DEFAULT_ECE_REBUILD_THRESHOLD` | `f32` | Default Expected Calibration Error (ECE) threshold for triggering a rebuild. |
| `DEFAULT_INITIAL_THRESHOLD` | `f32` | Default initial threshold $\hat{q}_0$. |
| `DEFAULT_LEARNING_RATE` | `f32` | Default learning rate $\gamma$ for online quantile updates. |
| `DEFAULT_MAX_THRESHOLD` | `f32` | Default maximum threshold bound. |
| `DEFAULT_MIN_THRESHOLD` | `f32` | Default minimum threshold bound. |

### Öffentliche Funktionen (94)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn alpha(&self) -> f32` | Returns the target error rate $\alpha$. | `src/calibration/conformal.rs` |
| `pub fn bounded( alpha: f32, min_threshold: f32, max_threshold: f32, ) -> Result<Self, ConformalError>` | Creates a bounded `AdaptiveConformalCalibrator` with custom miscoverage level `alpha` and explicit threshold bounds. | `src/calibration/conformal.rs` |
| `pub fn invalidate_on_config_change(&mut self, new_fingerprint: ConfigFingerprint)` | Resets state if ConfigFingerprint changes (P8 Compliance). | `src/calibration/conformal.rs` |
| `pub fn learning_rate(&self) -> f32` | Returns the learning rate $\gamma$. | `src/calibration/conformal.rs` |
| `pub fn new(alpha: f32) -> Result<Self, ConformalError>` | Creates a new `AdaptiveConformalCalibrator` with target error rate `alpha` in `(0.0, 1.0)`. Uses default initial threshold (0.5) and defaul… | `src/calibration/conformal.rs` |
| `pub fn observation_count(&self) -> u64` | Returns the total number of observations processed. | `src/calibration/conformal.rs` |
| `pub fn with_bounds( mut self, min_threshold: f32, max_threshold: f32, ) -> Result<Self, ConformalError>` | Sets bounding constraints `[min_threshold, max_threshold]` for the estimated threshold. | `src/calibration/conformal.rs` |
| `pub fn with_params( alpha: f32, initial_threshold: f32, learning_rate: f32, ) -> Result<Self, ConformalError>` | Creates an `AdaptiveConformalCalibrator` with custom parameters. | `src/calibration/conformal.rs` |
| `pub fn cached_ece(&self) -> Option<f32>` | Returns cached Expected Calibration Error (ECE). | `src/calibration/isotonic.rs` |
| `pub fn calibrated_probability(&mut self, raw_score: f32) -> Option<f32>` | Returns calibrated probability if warm, or `None` if below warmup requirement. | `src/calibration/isotonic.rs` |
| `pub fn expected_calibration_error(&mut self) -> Option<f32>` | Calculates Expected Calibration Error over 10 bins. | `src/calibration/isotonic.rs` |
| `pub fn force_rebuild(&mut self)` | Forces immediate PAVA model rebuild. | `src/calibration/isotonic.rs` |
| `pub fn invalidate_on_config_change(&mut self, new_fingerprint: ConfigFingerprint)` | Resets observations if ConfigFingerprint changes (P8 Compliance). | `src/calibration/isotonic.rs` |
| `pub fn is_calibrated(&self) -> bool` | Checks if warmup observations threshold has been reached. | `src/calibration/isotonic.rs` |
| `pub fn last_calibration_at(&self) -> Option<u64>` | Returns timestamp (UNIX epoch in seconds) of last model rebuild. | `src/calibration/isotonic.rs` |
| `pub fn maybe_rebuild_on_ece(&mut self, threshold: f32) -> bool` | Checks the Expected Calibration Error (ECE) against `threshold`. Calls `expected_calibration_error()`. If the result is `Some(ece)` with `e… | `src/calibration/isotonic.rs` |
| `pub fn new(warmup_required: u32, max_observations: usize) -> Self` | Creates a new `IsotonicCalibrator`. | `src/calibration/isotonic.rs` |
| `pub fn observation_count(&self) -> usize` | Returns current observation count. | `src/calibration/isotonic.rs` |
| `pub fn record_outcome(&mut self, raw_score: f32, outcome: bool)` | Records a new observation `(raw_score, outcome)`. | `src/calibration/isotonic.rs` |
| `pub fn with_clock(mut self, clock: Arc<dyn Clock>) -> Self` | Sets a custom clock port for deterministic time access. | `src/calibration/isotonic.rs` |
| `pub fn with_defaults() -> Self` | Creates an `IsotonicCalibrator` with defaults (Warmup: 50, Max Obs: 2000). | `src/calibration/isotonic.rs` |
| `pub fn apply(&self, logit: f32) -> f32` | Alias for `transform(logit)`. | `src/calibration/platt.rs` |
| `pub fn calibration_context( &self, signal_name: impl Into<String>, drift_status: crate::drift::DriftStatus, ) -> crate::fusion::SignalCalibrationContext` | Constructs a `SignalCalibrationContext` for this scaler. | `src/calibration/platt.rs` |
| `pub fn fit(observations: &[(f32, bool)]) -> Self` | Fits parameters `A` and `B` via Negative Log-Likelihood minimization with L2 regularization and target smoothing (Platt, 1999). | `src/calibration/platt.rs` |
| `pub fn identity() -> Self` | Uncalibrated default fallback ($A=1.0, B=0.0$). | `src/calibration/platt.rs` |
| `pub fn invalidate_on_config_change(&mut self, new_fingerprint: ConfigFingerprint)` | Resets scaler to identity if ConfigFingerprint changes (P8 Compliance). | `src/calibration/platt.rs` |
| `pub fn is_fitted(&self) -> bool` | Checks if model has been fitted. | `src/calibration/platt.rs` |
| `pub fn is_identity(&self) -> bool` | Checks if this scaler is identical to uncalibrated default ($A=1.0, B=0.0$). | `src/calibration/platt.rs` |
| `pub fn new(a: f32, b: f32) -> Self` | Creates a new `PlattScaler` with specified parameters `a` and `b`. | `src/calibration/platt.rs` |
| `pub fn params(&self) -> (f32, f32)` | Returns current parameters `(a, b)`. | `src/calibration/platt.rs` |
| `pub fn predict(&self, raw_score: f32) -> f32` | Calculates calibrated probability for raw score. | `src/calibration/platt.rs` |
| `pub fn transform(&self, logit: f32) -> f32` | Applies Platt Scaling `sigmoid(A * logit + B)`. | `src/calibration/platt.rs` |
| `pub fn fuse_exact_prefix<I1, I2, I3>( mut state: DiBudFusionState, vector: &mut I1, text: &mut I2, graph: &mut I3, edge_reinforcement: impl Fn(DocId) -> f32, budget: &FusionBudget, ) -> Result<DiBudOutcome, ContextraError> where I1: Iterator<Item = DocId>, I2…` | Synchronously executes DiBud fusion using three channel iterators. | `src/dibud/driver.rs` |
| `pub async fn fuse_exact_prefix_async<P, Fut>( mut state: DiBudFusionState, mut poll: P, edge_reinforcement: impl Fn(DocId) -> f32, budget: &FusionBudget, ) -> Result<DiBudOutcome, ContextraError> where P: FnMut(BudgetedChannel) -> Fut, Fut: Future<Output = Re…` | Asynchronously executes DiBud fusion using a channel poll closure returning a std `Future`. This function relies purely on `core::future::F… | `src/dibud/driver.rs` |
| `pub fn accesses(&self) -> usize` | Returns total read accesses performed. | `src/dibud/state.rs` |
| `pub fn channel_depths(&self) -> [usize` | Returns current channel depths `[Vector, Text, Graph]`. | `src/dibud/state.rs` |
| `pub fn exhausted(&self) -> [bool` | Returns exhausted status for each channel `[Vector, Text, Graph]`. | `src/dibud/state.rs` |
| `pub fn explain_doc(&self, doc_id: &DocId) -> Option<crate::explain::RetrievalExplanation>` | Generates a structured retrieval explanation for a given document if present in provenance. | `src/dibud/state.rs` |
| `pub fn feed( &mut self, ch: BudgetedChannel, item: Option<DocId>, er: impl Fn(DocId) -> f32, budget: &FusionBudget, ) -> Result<(), ContextraError>` | Feeds the result of reading a channel into the state machine. Raw scores are unknown in pure `DocId` streams and are set to 0.0 in `Provena… | `src/dibud/state.rs` |
| `pub fn new(capacity: usize) -> Self` | Creates a new `DiBudFusionState` with pre-allocated map capacities. | `src/dibud/state.rs` |
| `pub fn next_request(&self, budget: &FusionBudget) -> DiBudStep` | Determines whether fusion is done or returns the next channel to poll. | `src/dibud/state.rs` |
| `pub fn provenance_of(&self, doc_id: &DocId) -> Option<&ProvenanceRecord>` | Returns reference to provenance record for a given document if present. | `src/dibud/state.rs` |
| `pub fn all() -> impl Iterator<Item = Self>` | Iterates through all valid channels in index order. | `src/dibud/types.rs` |
| `pub fn validate(&self) -> Result<(), ContextraError>` | Validates budget parameter constraints. Ensures all weights are finite and non-negative, and minimum accesses/certified lengths are at leas… | `src/dibud/types.rs` |
| `pub fn analyze(&self) -> DriftStatus` | Analyzes current observations against baseline. | `src/drift.rs` |
| `pub fn as_str(&self) -> &'static str` | Returns human-readable summary string ("stabil", "warnung", "kritisch", or "unbekannt"). | `src/drift.rs` |
| `pub fn new(window_size: usize, threshold: f32) -> Self` | Creates a new `DriftDetector` with window size and sensitivity threshold. | `src/drift.rs` |
| `pub fn observe(&mut self, score: f32) -> DriftStatus` | Records a new score observation and calculates updated drift status. | `src/drift.rs` |
| `pub fn overall_drift_status(&self) -> &'static str` | Returns human-readable overall drift status string. | `src/drift.rs` |
| `pub fn set_baseline(&mut self, baseline: f32)` | Sets explicit baseline mean score. | `src/drift.rs` |
| `pub fn with_defaults() -> Self` | Creates a `DriftDetector` with default parameters (window size 50, threshold 0.15). | `src/drift.rs` |
| `pub fn explain(record: &ProvenanceRecord) -> RetrievalExplanation` | Generates a structured `RetrievalExplanation` from a `ProvenanceRecord`. Computes per-signal `contribution_share` normalized such that all… | `src/explain.rs` |
| `pub fn name_de(&self) -> &'static str` | Returns the German display name for the signal. | `src/explain.rs` |
| `pub fn to_human_readable_de(&self) -> String` | Generates a human-readable German explanation string for audit logging or CLI output. Formats all signal contributions using exact ranks an… | `src/explain.rs` |
| `pub fn config(&self) -> &GlobalFusionConfig` | Returns a reference to the configuration. | `src/fusion/global.rs` |
| `pub fn fuse( &self, result_sets: Vec<(String, Vec<SearchResult>, f32)>, max_results: usize, ) -> Result<Vec<SearchResult>, ContextraError>` | Fuses multi-signal search results using community-aware RRF. Candidates are grouped by community ID (extracted from candidate metadata `"co… | `src/fusion/global.rs` |
| `pub fn new(config: GlobalFusionConfig) -> Self` | Creates a new `GlobalFusionStrategy` with the given configuration. | `src/fusion/global.rs` |
| `pub fn score_normalized_fusion_with_options( mut result_sets: Vec<(String, Vec<SearchResult>, f32)>, max_results: usize, priority: MetadataMergePriority, include_provenance: bool, resonance_config: Option<&ResonanceConfig>, ) -> Vec<SearchResult>` | Score-normalized fusion (CombSUM with MinMax normalization). | `src/fusion/normalized.rs` |
| `pub fn weights_to_signal_factors( weights: Option<&contextra_types::FusionWeights>, ) -> (f32, f32, f32)` | Converts optional `FusionWeights` into (vector, text, graph) weight tuple. | `src/fusion/normalized.rs` |
| `pub fn bm25(mut self, score: f32, rank: u32, weight: impl Into<Option<f32>>) -> Self` | Attaches BM25 text signal score, rank, and optional weight. | `src/fusion/provenance.rs` |
| `pub fn build(self) -> ProvenanceRecord` | Builds the resulting `ProvenanceRecord`. | `src/fusion/provenance.rs` |
| `pub fn expected_total(mut self, total: f32) -> Self` | Sets expected ground truth RRF score for debug invariant checking. | `src/fusion/provenance.rs` |
| `pub fn explain_human_readable(&self) -> String` | Generates a deterministic human-readable explanation string from the built provenance record. | `src/fusion/provenance.rs` |
| `pub fn explain_human_readable(&self) -> String` | Generates a concise, deterministic human-readable explanation from retrieval signal contributions. | `src/fusion/provenance.rs` |
| `pub fn explain_human_readable_with_params( &self, fusion_mode: Option<&str>, calibrated_threshold: Option<f32>, ) -> String` | Generates a concise, deterministic human-readable explanation with optional fusion mode and calibrated threshold. | `src/fusion/provenance.rs` |
| `pub fn graph(mut self, score: f32, rank: u32, weight: impl Into<Option<f32>>) -> Self` | Attaches graph signal score, rank, and optional weight. | `src/fusion/provenance.rs` |
| `pub fn index_type(mut self, index_type: impl Into<String>) -> Self` | Attaches index type string. | `src/fusion/provenance.rs` |
| `pub fn new(rrf_k: f32) -> Self` | Creates a new `ProvenanceBuilder` with the specified RRF parameter `k`. | `src/fusion/provenance.rs` |
| `pub fn rerank_score(mut self, score: f32) -> Self` | Attaches optional cross-encoder rerank score. | `src/fusion/provenance.rs` |
| `pub fn source_collection(mut self, collection: impl Into<String>) -> Self` | Attaches source collection name. | `src/fusion/provenance.rs` |
| `pub fn vector(mut self, dist: f32, rank: u32, weight: impl Into<Option<f32>>) -> Self` | Attaches vector signal distance, rank, and optional weight. | `src/fusion/provenance.rs` |
| `pub fn apply_resonance_bonus( results: Vec<SearchResult>, valid_signal_count: usize, config: &ResonanceConfig, ) -> Vec<SearchResult>` | Applies Resonance Coherence Bonus to fused search results. | `src/fusion/resonance.rs` |
| `pub fn fuse_signals( result_sets: Vec<(String, Vec<SearchResult>, f32)>, max_results: usize, ) -> Vec<FusedScore>` | Convenience API to fuse multi-signal search results. | `src/fusion/resonance.rs` |
| `pub fn fuse_search_results_with_signal_strategies( mut result_sets: Vec<(String, Vec<SearchResult>, f32)>, max_results: usize, priority: MetadataMergePriority, include_provenance: bool, resonance_config: Option<&ResonanceConfig>, strategies: contextra_types::…` | Fuses search result sets using per-signal fusion strategies (`SignalFusionStrategies`). | `src/fusion/rrf.rs` |
| `pub fn fuse_search_results_with_strategy( result_sets: Vec<(String, Vec<SearchResult>, f32)>, max_results: usize, priority: MetadataMergePriority, include_provenance: bool, resonance_config: Option<&ResonanceConfig>, strategy: FusionStrategy, ) -> Vec<SearchR…` | Fuses search result sets using the specified `FusionStrategy`. | `src/fusion/rrf.rs` |
| `pub fn g(x: f32) -> f32` | Bounded smooth influence function $g(x) = \frac{x}{1 + |x|}$. Returns $x / (1 + |x|)$, bounded in $(-1.0, 1.0)$ for all finite $x$. Returns… | `src/fusion/rrf.rs` |
| `pub fn modulate_and_renormalize_weights( result_sets: &[(String, Vec<SearchResult>, f32)], calibration_contexts: Option<&[SignalCalibrationContext]>, beta: f32, ) -> Vec<f32>` | Modulates signal weights using mRRF margin formula: $m_s = S_1 - S_2$ (when signal is calibrated AND `drift_status` is `Stable` and $\ge 2$… | `src/fusion/rrf.rs` |
| `pub fn reciprocal_rank_fusion( result_sets: Vec<Vec<SearchResult>>, max_results: usize, ) -> Vec<SearchResult>` | Fuses multiple sets of ranked search results into a single ranked list using Reciprocal Rank Fusion (RRF). | `src/fusion/rrf.rs` |
| `pub fn weighted_reciprocal_rank_fusion( result_sets: Vec<(String, Vec<SearchResult>, f32)>, max_results: usize, ) -> Vec<SearchResult>` | Weighted Reciprocal Rank Fusion with default signal metadata priority (`VectorFirst`). | `src/fusion/rrf.rs` |
| `pub fn weighted_reciprocal_rank_fusion_mrrf( mut result_sets: Vec<(String, Vec<SearchResult>, f32)>, max_results: usize, priority: MetadataMergePriority, include_provenance: bool, resonance_config: Option<&ResonanceConfig>, calibration_contexts: Option<&[Sign…` | Weighted Reciprocal Rank Fusion with mRRF margin-based calibration weight modulation. | `src/fusion/rrf.rs` |
| `pub fn weighted_reciprocal_rank_fusion_with_options( result_sets: Vec<(String, Vec<SearchResult>, f32)>, max_results: usize, priority: MetadataMergePriority, include_provenance: bool, resonance_config: Option<&ResonanceConfig>, ) -> Vec<SearchResult>` | Weighted Reciprocal Rank Fusion with options. | `src/fusion/rrf.rs` |
| `pub fn weighted_reciprocal_rank_fusion_with_priority( result_sets: Vec<(String, Vec<SearchResult>, f32)>, max_results: usize, priority: MetadataMergePriority, ) -> Vec<SearchResult>` | Weighted Reciprocal Rank Fusion with explicit metadata merge priority. | `src/fusion/rrf.rs` |
| `pub fn as_str(&self) -> &'static str` | Converts `SignalKind` to its canonical string representation. | `src/fusion/signal.rs` |
| `pub fn from_name(name: &str) -> Option<Self>` | Identifies `SignalKind` from a signal name string. | `src/fusion/signal.rs` |
| `pub fn new( signal_name: impl Into<String>, is_calibrated: bool, drift_status: DriftStatus, ) -> Self` | Creates a new `SignalCalibrationContext`. | `src/fusion/signal.rs` |
| `pub fn should_use_calibrated_scores(&self) -> bool` | Evaluates whether calibrated scores should be used for margin calculation. Returns `true` if and only if `is_calibrated` is true AND `drift… | `src/fusion/signal.rs` |
| `pub fn signal_rank(&self, signal_name: &str) -> usize` | Returns the precedence rank for a given signal name. | `src/fusion/signal.rs` |
| `pub fn capacity(&self) -> usize` | Returns maximum allowed capacity `k`. | `src/fusion/topk.rs` |
| `pub fn into_sorted_vec(self) -> Vec<T>` | Consumes the container and returns items sorted from best to worst. | `src/fusion/topk.rs` |
| `pub fn is_empty(&self) -> bool` | Returns `true` if the container holds zero items. | `src/fusion/topk.rs` |
| `pub fn len(&self) -> usize` | Returns current number of items held in the container. | `src/fusion/topk.rs` |
| `pub fn new(capacity: usize) -> Self` | Creates a new `BoundedTopK` container with maximum capacity `k`. | `src/fusion/topk.rs` |
| `pub fn push(&mut self, item: T)` | Pushes an item into top-K container, evicting the worst candidate if size exceeds capacity `k`. | `src/fusion/topk.rs` |
| `pub fn synthesized_from(source_doc_ids: &[contextra_types::DocId]) -> Self` | Constructs a `ProvenanceRecord` synthesized from source document IDs. | `src/fusion/types.rs` |


---

## `contextra-adapt`

| Attribut | Wert |
|---|---|
| **Ring** | 0 — Ring 0 — Foundation & Pure Numerics/Types |
| **Status** | Fertig |
| **Zweck** | LinUCB-Bandit-Mathematik, Lyapunov-Drift-Wächter und PID-Regler. |
| **Unsafe-Code** | Nein (`#![forbid(unsafe_code)]`) |
| **Interne Abhängigkeiten** | `contextra-ports`, `contextra-types` |
| **Externe Abhängigkeiten (Auswahl)** | `blake3`, `serde`, `serde_json`, `thiserror`, `tracing` |

### Architekturbeschreibung

Adaptive controllers, PID regulators, bandits, and Lyapunov drift watchers for Contextra.

### Modul- und Re-Export-Oberfläche (lib.rs)

```rust
pub mod bandit;
pub mod decay_controller;
pub mod drift;
pub mod flow_thompson;
pub mod homeostat;
pub mod lyapunov;
pub mod off_policy;
pub mod offpolicy;
pub mod pid;
pub mod pid_latency_controller;
pub mod rie_greedy;
pub mod shadow_mode;
pub use bandit::*;
pub use decay_controller::*;
pub use drift::*;
pub use flow_thompson::*;
pub use homeostat::*;
pub use lyapunov::*;
pub use off_policy::*;
pub use pid::*;
pub use pid_latency_controller::*;
pub use rie_greedy::{RieGreedyError, RieGreedyProfile};
pub use shadow_mode::*;
```

### Feature-Flags

| Flag | Aktiviert |
|---|---|
| `default` | `["bandit-routing", "flow-corrected-thompson"]` |
| `bandit-routing` | `[]` |
| `flow-corrected-thompson` | `["bandit-routing"]` |
| `sketched-bandit` | `["bandit-routing"]` |
| `rie-greedy-personalization` | `[]` |

### Schnittstellen — Public Traits (7)

**`AntiWindupController`** — *Erweitert PID-Regler um konditionelles Anti-Windup bei Aktuatorsättigung. Unterschied zu einfachem Integral-Clamping: Der Integrator akkumuliert NUR, wenn die Stellgröße nicht gesättigt ist — verhindert träges Abklingen nach I/O-Spitzen (Back-Calculation Anti…* (`src/pid.rs`)

```rust
pub trait AntiWindupController : Send + Sync {
/// Aktualisiert den Regler mit konditionellem Integrator-Update.
    ///
    /// `dt`: Zeitdelta seit letztem Update (aus injiziertem Clock-Port).
    /// `measured_latency_ms`: gemessene Latenz in Millisekunden.
    /// `is_actuator_saturated`: true, wenn die Stellgröße am konfigurierten
    ///   Limit liegt (Pool-Größe = max oder = min).
    ///
    /// Gibt die neue Pool-Größe zurück.
    fn update_with_anti_windup(
        &mut self,
        dt: Duration,
        measured_latency_ms: f32,
        is_actuator_saturated: bool,
    ) -> usize;
}
```

**`BanditDefaultFlipGate`** — *Formales Bandit-Default-Flip-Gate nach analogem Muster wie `DefaultFlipGate` (AP-P15-01).* (`src/flow_thompson.rs`)

```rust
pub trait BanditDefaultFlipGate {
/// `true` ⇔ alle drei Bedingungen erfüllt:
    /// 1. `sample_count >= MIN_BANDIT_SHADOW_SAMPLES` (10_000)
    /// 2. `cumulative_regret <= MAX_CUMULATIVE_REGRET_THRESHOLD` (0.0)
    /// 3. `p99_latency_delta_us <= 0.0`
    fn should_flip(&self, report: &BanditShadowReport) -> bool;
}
```

**`BanditPolicy`** — *Trait für Bandit-Routing Policies mit Konzeptdrift-Anpassung (§5.2.3).* (`src/bandit.rs`)

```rust
pub trait BanditPolicy : Send + Sync {
/// Wendet Drift-Penalty / Covariance Discounting bei erkannter concept drift an.
    fn apply_drift_penalty(&mut self, k_drift: f32, alpha_max: f32, gamma: f32);
}
```

**`DriftDetector`** — *Generic interface for sequential O(1) drift detectors.* (`src/drift.rs`)

```rust
pub trait DriftDetector {
/// Observes a reward or non-conformity score and returns the updated drift signal.
    fn observe(&mut self, reward: f32) -> DriftSignal;
    /// Returns `true` if drift is currently detected.
    fn is_drifting(&self) -> bool;
}
```

**`FcTsRng`** — *Trait für deterministischen Zufallszahlengenerator im Thompson-Sampling.* (`src/flow_thompson.rs`)

```rust
pub trait FcTsRng {
/// Generiert die nächste 64-Bit vorzeichenlose Ganzzahl.
    fn next_u64(&mut self) -> u64;

    /// Generiert eine gleichverteilte Fließkommazahl im halboffenen Intervall `[0, 1)`.
    fn next_f32(&mut self) -> f32 {
        let v = (self.next_u64() >> 40) as f32;
        v / 16777216.0
    }

    /// Generiert eine standardnormalverteilte Fließkommazahl $\mathcal{N}(0, 1)$ mittels Box-Muller-Transformation.
    fn next_standard_normal(&mut self) -> f32 {
        let mut u1 = self.next_f32();
        while u1 <= 1e-7 {
            u1 = self.next_f32();
        }
        let u2 = self.next_f32();
        (-2.0 * u1.ln()).sqrt() * (2.0 * std::f32::consts::TAU * u2).cos()
    }
}
```

**`OffPolicyCompatibility`** — *Trait zur Überprüfung der Off-Policy-Kompatibilität zwischen FC-TS und einer Referenzpolitik (§B.3.2).* (`src/flow_thompson.rs`)

```rust
pub trait OffPolicyCompatibility {
/// Gibt `Err` zurück, wenn für irgendeinen Arm eine Propensity von 0
    /// unter der Referenz-Policy gemessen wird, während FC-TS diesem Arm
    /// eine positive Sampling-Wahrscheinlichkeit zuweist (Verstoß gegen
    /// die Positivitätsannahme — IPS-Gewichte würden divergieren).
    fn verify_positivity(
        &self,
        reference_policy: &DiagonalApproximationBandit,
    ) -> Result<(), OffPolicyError>;
}
```

**`ShadowSink`** — *Dyn-kompatibler Port (P27) für Diskrepanz-Senken (In-Memory für Tests, Tracing/Log für Produktion).* (`src/shadow_mode.rs`)

```rust
pub trait ShadowSink<T> : Send + Sync {
/// Zeichnet einen Diskrepanz-Eintrag auf.
    fn record(&self, discrepancy: ShadowDiscrepancy<T>);
}
```

### Öffentliche Datentypen — Structs (31)

| Struct | Felder / Form | Beschreibung | Datei |
|---|---|---|---|
| `AdaptiveDecayController` | `{ config: DecayControllerConfig, }` | Adaptive Decay Controller für time-weighted Cache-Eviction. | `src/decay_controller.rs` |
| `AlignedF32Vec` | `(tuple/unit)` | 64-Byte cache-aligned Wrapper um `Vec<f32>` zur Vermeidung von False Sharing. Hinweis: Der `Vec<f32>`-Heap-Buffer selbst folgt zwar nicht zwingend de… | `src/bandit.rs` |
| `BanditProfileState` | `{ /// Gewichtsvektor θ_p (Dimension = Embedding-Dim). pub theta: Vec<f32>, /// Diagonale Kovarianz-Terme σ²_p (Diagonal-Approximation). pub sigma_sq: Vec<f32>, /… }` | Laufzeitzustand eines LinUCB-Bandits pro Profil. # Formeln (§13.2) Score: r̂_p(x) = θ_pᵀ x + α_p √(Σ_p(x)) - λ·c_p - μ·1[transport=HttpCloud] Reward:… | `src/bandit.rs` |
| `BanditShadowReport` | `{ /// Anzahl ausgewerteter Shadow-Mode-Samples. pub sample_count: u64, /// Kumulatives Regret des FC-TS-Kandidaten im Vergleich zur Baseline. pub cumulative_regr… }` | Aggregierter Shadow-Mode-Bericht für den Bandit-Default-Flip. | `src/flow_thompson.rs` |
| `CatoniDriftDetector` | `{ /// Robust mean estimate $\mu$. pub catoni_mu: f64, /// CUSUM sequential change detection accumulator. pub cusum_sum: f64, /// Threshold for declaring drift de… }` | Catoni-M-estimator based robust drift detector (arXiv:2505.20051 & arXiv:2501.10974). Uses influence function $\psi(x) = \frac{x}{1 + |x|}$ for $O(1)… | `src/drift.rs` |
| `DecayControllerConfig` | `{ /// Verstärkungsfaktor κ. Default: 2.0. pub kappa: f32, /// Basis-Half-Life in TxId-Einheiten. Default: 10_000. pub base_half_life_tx: u64, /// Eviction-Schwel… }` | Konfiguration für Adaptive Decay Controller (Cache-Eviction mit Time-Weighted Decay) (via MaintenanceConfig). | `src/decay_controller.rs` |
| `DecaySignalInputs` | `{ /// HNSW-Tombstone-Ratio ∈ [0,1]. Quelle: HnswIndex::tombstone_ratio(). pub tombstone_ratio: f32, /// Normalisierte inverse Query-Rate ∈ [0,1]. /// Berechnung:… }` | Inputs for Adaptive Decay Controller (Cache-Eviction mit Time-Weighted Decay) aus bestehenden System-Metriken. | `src/decay_controller.rs` |
| `DefaultBanditFlipGate` | `(tuple/unit)` | Standardimplementierung des [`BanditDefaultFlipGate`]. | `src/flow_thompson.rs` |
| `DiagonalApproximationBandit` | `{ /// Propensities P(a \| x) pro RetrievalStrategy. pub propensities: std::collections::HashMap<contextra_types::RetrievalStrategy, f32>, }` | Referenzpolitik basierend auf Diagonal-Approximation für Off-Policy-Evaluation (§B.3.2). | `src/flow_thompson.rs` |
| `DriftPolicyBridge` | `{ /// Der zugrundeliegende Lyapunov-Drift-Wächter. pub watcher: LyapunovDriftWatcher, /// Multiplikator k_drift für die Alpha-Erhöhung bei Drift. pub k_drift: f3… }` | Adapter zur Kopplung des `LyapunovDriftWatcher` mit einer `BanditPolicy`. | `src/drift.rs` |
| `DriftReason` | `{ /// Aktuelle Kullback-Leibler-Divergenz D_t = KL(N_t \|\| N_baseline). pub kl_divergence: f32, /// Diskreter Lyapunov-Exponent λ_t über das gleitende Fenster. pu… }` | Grund für erkannte Verteilungsverschiebung (Distributional Drift). | `src/lyapunov.rs` |
| `EnsembleDriftWatcher` | `{ /// Primary Lyapunov drift watcher instance. pub lyapunov: LyapunovDriftWatcher, /// Secondary Catoni robust drift detector instance. pub catoni: CatoniDriftDe… }` | Ensemble combination of [`LyapunovDriftWatcher`] and [`CatoniDriftDetector`]. Triggers drift response ONLY if BOTH detectors independently detect dri… | `src/drift.rs` |
| `FcTsArmSet` | `{ /// Liste der Bandit-Arme. pub arms: Vec<FlowCorrectedThompsonBandit>, }` | Menge von FC-TS-Armen zur Auswahl der optimalen Aktion. | `src/flow_thompson.rs` |
| `FcTsConfig` | `{ /// Feature-Dimension $d$. pub dim: usize, /// Ridge-Prior Parameter $\lambda > 0$. pub lambda: f32, /// Rauschvarianz $\sigma^2 > 0$. pub noise_var: f32, ///… }` | Konfigurationsparameter für Flow-Corrected Thompson Sampling (§21.3). | `src/flow_thompson.rs` |
| `FcTsSamplingDistribution` | `{ /// Mapping von RetrievalStrategy zu FC-TS-Sampling-Wahrscheinlichkeit P_FCTS(a \| x). pub probabilities: std::collections::HashMap<contextra_types::RetrievalSt… }` | Sampling-Wahrscheinlichkeitsverteilung über FC-TS-Arme/Strategien (§B.3.2). | `src/flow_thompson.rs` |
| `FlowCorrectedThompsonBandit` | `{ config: FcTsConfig, /// Inverse Kovarianzmatrix $A^{-1}$ ($d \times d$, row-major). inv_a: Vec<f32>, /// Akkumulator-Vektor $\eta$ ($d$ Elemente). eta: Vec<f32… }` | Laufzeitzustand eines Arms im Flow-Corrected Thompson Sampling (§21.3). | `src/flow_thompson.rs` |
| `LatencyBudgetGuard` | `{ start: Instant, budget_ms: f64, }` | Simple, panic-free latency budget guard for iterative multi-step search loops. | `src/pid_latency_controller.rs` |
| `LyapunovDriftWatcher` | `{ /// Fenstergröße w für die Lyapunov-Exponenten-Berechnung (Default: 20). pub window_size: usize, /// Historie der berechneten KL-Divergenzen D_t. pub divergenc… }` | Proaktiver Drift-Wächter auf Basis diskreter Lyapunov-Exponenten über KL-Divergenzen. | `src/lyapunov.rs` |
| `OffPolicyEvaluator` | `{ cumulative_ips: f64, samples: u64, discarded: u64, }` | Evaluator für Inverse Propensity Scoring (IPS) zur kontrafaktischen Off-Policy-Schätzung. | `src/offpolicy.rs` |
| `OffPolicyStats` | `{ /// Anzahl verworfener Samples aufgrund ungültiger Propensity-Werte (NaN, < 0.0, > 1.0). pub discarded: u64, }` | Statistiken über verworrene/ungültige Samples der Off-Policy-Evaluation. | `src/offpolicy.rs` |
| `PidController` | `{ /// Proportionaler Gewichtsbeiwert $K_p$. pub kp: f32, /// Integraler Gewichtsbeiwert $K_i$. pub ki: f32, /// Differentieller Gewichtsbeiwert $K_d$. pub kd: f3… }` | PID-Regler zur dynamischen Steuerung der Reranking-Kandidatenpool-Größe basierend auf Latenzmessungen. # Anti-Windup Strategien - Standard-Methode [`… | `src/pid.rs` |
| `PidLatencyController` | `{ /// Proportional gain coefficient. pub kp: f64, /// Integral gain coefficient. pub ki: f64, /// Derivative gain coefficient. pub kd: f64, /// Target latency bu… }` | Classical PID controller for dynamically scaling multi-step retrieval parameters (`k_pool`, `max_hops`). HINWEIS ZUR ANTI-WINDUP STRATEGIE: Dieser Co… | `src/pid_latency_controller.rs` |
| `RandomizedLoggingPolicy` | `{ /// Exploration-Wahrscheinlichkeit epsilon (ε in [0.0, 1.0]). pub epsilon: f32, /// Anzahl verfügbarer Aktionen K (K >= 1). pub num_actions: u32, }` | Randomisierte Logging-Policy für unverzerrte Datensammlung zur Off-Policy-Evaluation (§B.5.2.4 & §8.5). | `src/offpolicy.rs` |
| `RerankDeadline` | `{ /// Maximum time budget allowed for reranking operations. pub budget: Duration, }` | Hard deadline manager for candidate retrieval and Cross-Encoder reranking phases (P11 requirement). # Intended Integration Point Evaluated before or… | `src/homeostat.rs` |
| `RerankPidController` | `{ inner: crate::PidController, }` | — | `src/homeostat.rs` |
| `RieGreedyProfile` | `{ /// Inverse Präzisionsmatrix $\Lambda^{-1}$ ($d \times d$, row-major). pub precision_inv: Vec<f32>, /// Informationsvektor $\eta$ ($d$ Elemente). pub info_vect… }` | Laufzeitzustand eines RIE-Greedy-Profils (§10.4.1). Implementiert die deterministisch-gierige Personalisierung mit zeitlich diskontierter Präzisionsm… | `src/rie_greedy.rs` |
| `Ring3Token` | `{ _private: (), }` | Privater Zero-Sized Token zur Erzwingung, dass Drift-Update-Aufrufe exklusiv aus Ring-3-Hintergrundtasks stammen (AK-18). | `src/flow_thompson.rs` |
| `ShadowDiscrepancy` | `{ /// Das Ergebnis des etablierten Baseline-Algorithmus. pub baseline: T, /// Das Ergebnis des neuen Kandidaten-Algorithmus. pub candidate: T, /// Eindeutige Kon… }` | Diskrepanz-Eintrag: Baseline-Ergebnis vs. Kandidaten-Ergebnis, ohne Einfluss auf das tatsächlich zurückgegebene Nutzerergebnis (§12 Integrationsregel… | `src/shadow_mode.rs` |
| `SketchMatrix` | `{ /// Flacher Speicher der Matrix (Row-Major, k Zeilen x d Spalten). pub data: AlignedF32Vec, /// Projektionsdimension k. pub projected_dim: usize, /// Ursprüngl… }` | Projektionsmatrix $R \in \mathbb{R}^{k \times d}$ für Sketched Projection. | `src/bandit.rs` |
| `SplitMix64` | `{ state: u64, }` | Deterministische `SplitMix64`-Referenzimplementierung des [`FcTsRng`]-Traits. | `src/flow_thompson.rs` |
| `TracingShadowSink` | `(tuple/unit)` | Referenzimplementierung über `tracing::debug!` — kein zusätzlicher I/O-Pfad, keine Rng/Clock-Portverletzung (P28), da rein diagnostisch. | `src/shadow_mode.rs` |

### Öffentliche Datentypen — Enums (7)

| Enum | Varianten | Beschreibung | Datei |
|---|---|---|---|
| `BanditError` | `/// Dimension des Eingabevektors stimmt nicht mit dem Bandit-Zustand überein. #[error("Embedding dimension mismatch: expected {expected}, actual {actual}")] DimensionMismatch { /// Erwartete Dimensio…` | Fehlerzustände des Contextual Bandits. | `src/bandit.rs` |
| `BanditImplementation` | `/// Sherman-Morrison O(d²): Produktions-Default. #[default] ShermanMorrison, /// Diagonal-Approximation O(d): Opt-in-Variante. DiagonalApproximation, /// Flow-Corrected Thompson Sampling (§21.3). ///…` | LinUCB-Implementierungsvariante (§13.2). | `src/bandit.rs` |
| `DriftSignal` | `/// Signal distribution is stable. Stable, /// Distributional drift detected. Detected,` | Drift detection signal returned by a [`DriftDetector`]. | `src/drift.rs` |
| `FcTsError` | `/// Dimension des Eingabevektors stimmt nicht mit der konfigurierten Arm-Dimension überein. #[error("Embedding dimension mismatch: expected {expected}, actual {actual}")] DimensionMismatch { /// Erwa…` | Fehlerzustände für Flow-Corrected Thompson Sampling (FC-TS). | `src/flow_thompson.rs` |
| `LyapunovResult` | `/// Die Score-Verteilung ist stabil (λ_t <= 0.0). Stable { /// Diskreter Lyapunov-Exponent λ_t. lyapunov_exponent: f32, }, /// Proaktive Drift erkannt (λ_t > 0.0 über das Fenster). DriftDetected { //…` | Ergebnis der Lyapunov-Drift-Analyse. | `src/lyapunov.rs` |
| `OffPolicyError` | `/// Ein Arm hat Propensity 0 unter der Referenzpolicy, aber eine positive FC-TS-Sampling-Wahrscheinlichkeit. #[error("arm {0:?} has zero propensity under reference policy but positive FC-TS sampling…` | Fehlerzustände bei der Off-Policy-Kompatibilitätsprüfung. | `src/flow_thompson.rs` |
| `RieGreedyError` | `/// Dimension des Kontext-Vektors stimmt nicht mit der konfigurierten Profil-Dimension überein. #[error("Embedding dimension mismatch: expected {expected}, actual {actual}")] DimensionMismatch { ///…` | Fehlerzustände für den RIE-Greedy-Personalismus-Mechanismus (§10.4.1). | `src/rie_greedy.rs` |

### Konstanten (9)

| Konstante | Typ | Beschreibung |
|---|---|---|
| `DEFAULT_TARGET_LATENCY_MS` | `f64` | Konservativer Default für das P95-Latenzziel in Millisekunden (100ms). HINWEIS: implementation_plan-db2.md identifiziert den genauen P95-Zielwert als… |
| `MAX_CUMULATIVE_REGRET_THRESHOLD` | `f64` | Maximales zulässiges kumulatives Regret für den Bandit-Default-Flip (≤ 0.0). |
| `MAX_INTEGRAL` | `f64` | Max-Grenze für den I-Anteil (Anti-Windup Clamping). |
| `MAX_SCALING_FACTOR` | `f64` | Oberer Clamp für den K-Pool-Skalierungsfaktor (100% des konfigurierten K-Pools). |
| `MIN_BANDIT_SHADOW_SAMPLES` | `u64` | Minimale Anzahl an Shadow-Samples für den Bandit-Default-Flip. |
| `MIN_SCALING_FACTOR` | `f64` | Unterer Clamp für den K-Pool-Skalierungsfaktor (30% des konfigurierten K-Pools), um APM-RECALL-COLLAPSE in Standardlastszenarien zu verhindern. |
| `PID_MAX_POOL_SIZE_DEFAULT` | `usize` | Maximale Kandidaten-Pool-Größe. |
| `PID_MIN_POOL_SIZE_DEFAULT` | `usize` | Minimale Kandidaten-Pool-Größe (Hard Floor = 50, Quality Knee per arXiv:2604.01733). |
| `SHERMAN_MORRISON_REFACTORIZATION_INTERVAL` | `u64` | — |

### Öffentliche Funktionen (106)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn cold_start(d: usize, alpha_base: f32) -> Self` | Erstellt einen neuen Cold-Start-Zustand für Dimension `d`. | `src/bandit.rs` |
| `pub fn derive_seed(tenant_id: &contextra_types::TenantId, base_seed: u64) -> u64` | Leitet einen mandantenisolierten PRNG-Seed via BLAKE3-Hash aus `TenantId` und `base_seed` ab. | `src/bandit.rs` |
| `pub fn ensure_sketched_state(&mut self, projected_dim: usize) -> Result<(), BanditError>` | Stellt sicher, dass der Sketched-State für `projected_dim` k ordnungsgemäß initialisiert ist. | `src/bandit.rs` |
| `pub fn expected_dim(&self) -> usize` | Gibt die erwartete Dimension der Feature-Vektoren zurück. | `src/bandit.rs` |
| `pub fn from_seed( seed: u64, original_dim: usize, projected_dim: usize, ) -> Result<Self, BanditError>` | SplitMix64-PRNG aus `seed`, Rademacher-Einträge ±1/√k. | `src/bandit.rs` |
| `pub fn from_tenant_seed( tenant_id: &contextra_types::TenantId, base_seed: u64, original_dim: usize, projected_dim: usize, ) -> Result<Self, BanditError>` | Erstellt eine `SketchMatrix` mit mandantenisoliertem Seed aus `TenantId` und `base_seed`. | `src/bandit.rs` |
| `pub fn init_tenant_sketch( &mut self, tenant_id: &contextra_types::TenantId, base_seed: u64, projected_dim: usize, ) -> Result<(), BanditError>` | Initialisiert die Sketched Projection mit einem mandantenisolierten PRNG-Seed. | `src/bandit.rs` |
| `pub fn new(vec: Vec<f32>) -> Self` | Erstellt einen neuen `AlignedF32Vec` aus einem `Vec<f32>`. | `src/bandit.rs` |
| `pub fn observe_and_adapt_drift( &mut self, bridge: &mut crate::drift::DriftPolicyBridge, current_scores: &[f32], ) -> crate::lyapunov::LyapunovResult` | Observes current non-conformity scores via a `DriftPolicyBridge` and applies drift adaptation if detected. | `src/bandit.rs` |
| `pub fn on_drift_detected(&mut self, k_drift: f32)` | Drift-Kopplung: Erhöhe α temporär und aktiviere beschleunigten Decay bei erkannter Drift (§13.2, §5.2.3). α_p ← min(α_p · k_drift, α_base ·… | `src/bandit.rs` |
| `pub fn project(&self, x: &[f32]) -> Vec<f32>` | Projiziert Vektor x aus dem Ursprungsraum (Dim d) in den projizierten Raum (Dim k) via Rx. | `src/bandit.rs` |
| `pub fn score( &self, x: &[f32], cost: f32, is_cloud_transport: bool, ) -> Result<f32, BanditError>` | Berechnet UCB-Score für Kontext-Embedding `x` und Profilkosten `cost`. r̂_p(x) = θᵀx + α√(Σ(x)) - λ·cost - μ·is_cloud | `src/bandit.rs` |
| `pub fn update( &mut self, x: &[f32], r_outcome: f32, cost: f32, is_cloud_transport: bool, ) -> Result<(), BanditError>` | Aktualisiert θ und σ² (sowie A⁻¹ bei Sherman-Morrison) nach beobachtetem Outcome unter Berücksichtigung von Discounting. r_adj = r_outcome… | `src/bandit.rs` |
| `pub fn zeros(len: usize) -> Self` | Erstellt einen neuen `AlignedF32Vec` mit `len` Nullen. | `src/bandit.rs` |
| `pub fn effective_half_life(&self, temperature: f32) -> f32` | Effektive Half-Life unter aktueller Temperatur. | `src/decay_controller.rs` |
| `pub fn effective_score( &self, base_score: f32, elapsed_tx: u64, inputs: &DecaySignalInputs, ) -> f32` | Temperatur-adjustierter Score für einen Chunk. `elapsed_tx`: TxId(now) - TxId(chunk_created). ADR-016: kein SystemTime. | `src/decay_controller.rs` |
| `pub fn new(config: DecayControllerConfig) -> Self` | Erstellt einen neuen `AdaptiveDecayController` mit der angegebenen Konfiguration. | `src/decay_controller.rs` |
| `pub fn should_evict( &self, base_score: f32, elapsed_tx: u64, inputs: &DecaySignalInputs, ) -> bool` | Gibt true wenn Chunk unter Eviction-Schwelle. Wird von Background Workers (background_workers.rs) für Sweep verwendet. | `src/decay_controller.rs` |
| `pub fn system_temperature(&self, inputs: &DecaySignalInputs) -> f32` | Berechnet aktuelle Systemtemperatur T(t) ∈ [0,1]. | `src/decay_controller.rs` |
| `pub fn with_defaults() -> Self` | Erstellt einen neuen `AdaptiveDecayController` mit Standardwerten. | `src/decay_controller.rs` |
| `pub fn new(baseline_reward: f64, threshold: f64, alpha: f64) -> Self` | Creates a new `CatoniDriftDetector` with specified baseline, threshold, and learning rate. | `src/drift.rs` |
| `pub fn new(lyapunov: LyapunovDriftWatcher, catoni: CatoniDriftDetector) -> Self` | Creates a new `EnsembleDriftWatcher`. | `src/drift.rs` |
| `pub fn new(window_size: usize, k_drift: f32, alpha_max: f32, gamma: f32) -> Self` | Erstellt eine neue `DriftPolicyBridge` mit benutzerdefinierten Parametern. | `src/drift.rs` |
| `pub fn observe_and_decide(&mut self, reward: f32) -> bool` | Observes a new reward / score and decides if drift is detected by BOTH detectors. | `src/drift.rs` |
| `pub fn observe_and_react<P: crate::bandit::BanditPolicy + ?Sized>( &mut self, current_scores: &[f32], policy: &mut P, ) -> LyapunovResult` | Nimmt aktuelle Non-Conformity-Scores auf, wertet den Drift-Status aus und wendet bei erkannter Drift eine Strafe auf die `BanditPolicy` an. | `src/drift.rs` |
| `pub fn observe_and_react_policy<P: crate::bandit::BanditPolicy + ?Sized>( &mut self, reward: f32, policy: &mut P, k_drift: f32, alpha_max: f32, gamma: f32, ) -> bool` | Observes a new reward / score, evaluates ensemble drift detection, and applies a penalty to the target policy when drift occurs. | `src/drift.rs` |
| `pub fn reset(&mut self)` | Resets the CUSUM accumulator and restores `catoni_mu` to baseline. | `src/drift.rs` |
| `pub fn set_baseline(&mut self, baseline_scores: &[f32])` | Sets the baseline distribution for the Lyapunov watcher and baseline mean for the Catoni detector. | `src/drift.rs` |
| `pub fn baseline_score(&self, context: &[f32]) -> Result<f32, FcTsError>` | Berechnet den Baseline-Mittelwert-Score (Sherman-Morrison $\langle \mu, x \rangle$) für den Kontext $x$. | `src/flow_thompson.rs` |
| `pub fn config(&self) -> &FcTsConfig` | Gibt eine Referenz auf die Konfiguration zurück. | `src/flow_thompson.rs` |
| `pub fn drift_rate(&self) -> &[f32]` | Gibt die aktuelle Drift-Rate $\delta$ zurück. | `src/flow_thompson.rs` |
| `pub fn from_pairs( pairs: impl IntoIterator<Item = (contextra_types::RetrievalStrategy, f32)>, ) -> Self` | Erstellt eine `DiagonalApproximationBandit`-Referenzpolitik aus Strategie-Propensitäts-Paaren. | `src/flow_thompson.rs` |
| `pub fn from_pairs( pairs: impl IntoIterator<Item = (contextra_types::RetrievalStrategy, f32)>, ) -> Self` | Erstellt eine `FcTsSamplingDistribution`-Instanz aus Strategie-Wahrscheinlichkeits-Paaren. | `src/flow_thompson.rs` |
| `pub fn inv_a(&self) -> &[f32]` | Gibt die inverse Kovarianzmatrix $A^{-1}$ ($d \times d$, row-major) zurück. | `src/flow_thompson.rs` |
| `pub fn mu(&self) -> &[f32]` | Gibt den aktuellen Mittelwert-Vektor $\mu$ zurück. | `src/flow_thompson.rs` |
| `pub fn new(seed: u64) -> Self` | Erstellt eine neue `SplitMix64`-Instanz mit dem gegebenen Seed. | `src/flow_thompson.rs` |
| `pub fn new(config: FcTsConfig) -> Result<Self, FcTsError>` | Erstellt eine neue FC-TS-Bandit-Instanz mit der gegebenen Konfiguration. | `src/flow_thompson.rs` |
| `pub fn new( propensities: std::collections::HashMap<contextra_types::RetrievalStrategy, f32>, ) -> Self` | Erstellt eine neue `DiagonalApproximationBandit`-Referenzpolitik mit der gegebenen Propensitätsmap. | `src/flow_thompson.rs` |
| `pub fn new( probabilities: std::collections::HashMap<contextra_types::RetrievalStrategy, f32>, ) -> Self` | Erstellt eine neue `FcTsSamplingDistribution`-Instanz mit der gegebenen Wahrscheinlichkeitsmap. | `src/flow_thompson.rs` |
| `pub fn probability(&self, strategy: contextra_types::RetrievalStrategy) -> f32` | Liefert die FC-TS-Sampling-Wahrscheinlichkeit für eine gegebene RetrievalStrategy. | `src/flow_thompson.rs` |
| `pub fn propensity(&self, strategy: contextra_types::RetrievalStrategy) -> f32` | Liefert die Propensity P(a | x) für eine gegebene RetrievalStrategy. | `src/flow_thompson.rs` |
| `pub fn recompute_drift_rate_from_window(&mut self, _token: &Ring3Token)` | Berechnet die Drift-Rate $\delta$ aus dem aktuellen Fenster neu via Ridge-Regression (AK-18). Nutzt Conjugate Gradient zur Lösung von $(Z^T… | `src/flow_thompson.rs` |
| `pub fn recompute_drift_rate_in_background(&mut self)` | Recomputes the drift rate from the window in a background task by creating a `Ring3Token` internally. | `src/flow_thompson.rs` |
| `pub fn recompute_drift_rates_in_background(&mut self)` | Triggers background drift rate recomputation for all arms in the set. | `src/flow_thompson.rs` |
| `pub fn ring3_background_task_token() -> Ring3Token` | Erzeugt einen `Ring3Token` für den Aufruf in asynchronen Ring-3-Hintergrundtasks. # Architektur-Hinweis (AK-18) Darf ausschließlich im Ring… | `src/flow_thompson.rs` |
| `pub fn sample_score(&self, context: &[f32], rng: &mut dyn FcTsRng) -> Result<f32, FcTsError>` | Berechnet einen zufälligen Thompson-Sample Score für den Kontext $x$. $w_i = \mu_i + v \cdot \sqrt{\max(A^{-1}_{i,i}, 0)} \cdot z_i$, $\tex… | `src/flow_thompson.rs` |
| `pub fn select_arm(&self, context: &[f32], rng: &mut dyn FcTsRng) -> Result<u32, FcTsError>` | Wählt die Aktion für diese Bandit-Instanz und protokolliert Diskrepanzen bei konfiguriertem Shadow-Sink. | `src/flow_thompson.rs` |
| `pub fn select_arm(&self, context: &[f32], rng: &mut dyn FcTsRng) -> Result<u32, FcTsError>` | Wählt den Arm mit dem höchsten gesampelten Score aus (Argmax). Bei Gleichstand wird der kleinste Arm-Index gewählt. Bei konfiguriertem Shad… | `src/flow_thompson.rs` |
| `pub fn shadow_sink(&self) -> Option<&std::sync::Arc<dyn crate::shadow_mode::ShadowSink<u32>>>` | Gibt eine Referenz auf die optional konfigurierte Diskrepanz-Senke zurück. | `src/flow_thompson.rs` |
| `pub fn update_arm( &mut self, arm_idx: usize, context: &[f32], reward: f32, observation_time: u64, confidence_weight: f32, ) -> Result<(), FcTsError>` | Updates an arm in the set with a new observation flow. | `src/flow_thompson.rs` |
| `pub fn update_with_flow( &mut self, context: &[f32], reward: f32, observation_time: u64, confidence_weight: f32, ) -> Result<(), FcTsError>` | Akkumuliert ein neues Beobachtungstupel mit konfidenzgewichtetem Transport (§21.3). # Parameter - `context`: Kontext-Vektor $x \in \mathbb{… | `src/flow_thompson.rs` |
| `pub fn validate(&self) -> Result<(), FcTsError>` | Validiert die Konfigurationsparameter. | `src/flow_thompson.rs` |
| `pub fn with_shadow_sink( mut self, sink: std::sync::Arc<dyn crate::shadow_mode::ShadowSink<u32>>, ) -> Self` | Konfiguriert eine Diskrepanz-Senke für den Shadow-Mode-Vergleich (§12). | `src/flow_thompson.rs` |
| `pub fn with_shadow_sink( mut self, sink: std::sync::Arc<dyn crate::shadow_mode::ShadowSink<u32>>, ) -> Self` | Konfiguriert eine Diskrepanz-Senke für alle Arme in der Menge (§12). | `src/flow_thompson.rs` |
| `pub fn deadline_exceeded(&self, started_at: Instant) -> bool` | Checks if the elapsed time since `started_at` equals or exceeds the deadline budget. | `src/homeostat.rs` |
| `pub fn k_max(&self) -> usize` | Accessor for `k_max`. | `src/homeostat.rs` |
| `pub fn k_min(&self) -> usize` | Accessor for `k_min`. | `src/homeostat.rs` |
| `pub fn k_pool(&self) -> usize` | Accessor for current `k_pool`. | `src/homeostat.rs` |
| `pub fn kd(&self) -> f32` | Accessor for `kd`. | `src/homeostat.rs` |
| `pub fn ki(&self) -> f32` | Accessor for `ki`. | `src/homeostat.rs` |
| `pub fn kp(&self) -> f32` | Accessor for `kp`. | `src/homeostat.rs` |
| `pub fn new( target_p95_latency_ms: f32, k_min: usize, k_max: usize, initial_pool: usize, ) -> Self` | Creates a new `RerankPidController` with default PID coefficients and configured parameters. | `src/homeostat.rs` |
| `pub fn new(budget: Duration) -> Self` | Creates a new `RerankDeadline` with the specified duration budget. | `src/homeostat.rs` |
| `pub fn pid_regulated_candidate_pool( controller: &mut crate::PidController, observed_p95_latency_ms: f32, ) -> usize` | Dynamic PID regulation for candidate pool size based on observed p95 latency. # Time Measurement & P28 Determinism P95 latency measurement… | `src/homeostat.rs` |
| `pub fn target_p95_latency_ms(&self) -> f32` | Accessor for target latency. | `src/homeostat.rs` |
| `pub fn update(&mut self, observed_p95_latency_ms: f32) -> usize` | Updates the controller state with observed latency and returns new pool size. | `src/homeostat.rs` |
| `pub fn analyze(&self) -> LyapunovResult` | Gibt das neueste Analyseergebnis der Lyapunov-Drift-Berechnung zurück. | `src/lyapunov.rs` |
| `pub fn apply_drift_if_detected<P: crate::bandit::BanditPolicy + ?Sized>( &self, policy: &mut P, k_drift: f32, alpha_max: f32, gamma: f32, ) -> bool` | Wendet `apply_drift_penalty` auf die gegebene `BanditPolicy` an, falls `DriftDetected` vorliegt. Gibt `true` zurück, falls Drift erkannt un… | `src/lyapunov.rs` |
| `pub fn is_drift_detected(&self) -> bool` | Returns `true` if this result represents a detected distributional drift. | `src/lyapunov.rs` |
| `pub fn latest_result(&self) -> Option<&LyapunovResult>` | Gibt eine Referenz auf das neueste Analyseergebnis zurück. | `src/lyapunov.rs` |
| `pub fn new(window_size: usize) -> Self` | Erstellt einen neuen `LyapunovDriftWatcher` mit der angegebenen Fenstergröße. | `src/lyapunov.rs` |
| `pub fn observe_score(&mut self, score: f32) -> LyapunovResult` | Fügt einen einzelnen Non-Conformity-Score zur Historie hinzu und aktualisiert den Drift-Status. | `src/lyapunov.rs` |
| `pub fn set_baseline(&mut self, baseline_scores: &[f32])` | Setzt die Baseline-Verteilung der Non-Conformity-Scores aus dem Kalibrierungs-Warmup. | `src/lyapunov.rs` |
| `pub fn status_str(&self) -> &'static str` | Gibt den aktuellen Drift-Status als menschenlesbare Zeichenkette ("stabil", "warnung", "kritisch", "unbekannt") zurück. | `src/lyapunov.rs` |
| `pub fn update(&mut self, current_scores: &[f32]) -> LyapunovResult` | Nimmt aktuelle Non-Conformity-Scores auf und berechnet den Drift-Status. | `src/lyapunov.rs` |
| `pub fn compute_propensities(&self, greedy_action: u32) -> Vec<f32>` | Berechnet die Propensity P(a | x) für jede Aktion 0..K-1 bei gegebener greedy/target Aktion. | `src/offpolicy.rs` |
| `pub fn cumulative_ips(&self) -> f64` | Liefert die kumulative IPS-Summe Σ (r_i / max(p_i, 0.01)). | `src/offpolicy.rs` |
| `pub fn discarded(&self) -> u64` | Liefert die Anzahl verworfener Ungültig-Propensity-Samples. | `src/offpolicy.rs` |
| `pub fn estimate(&self) -> f64` | Liefert den aktuellen IPS-Schätzwert V_IPS = cumulative_ips / samples. Falls noch keine Samples beobachtet wurden (`samples == 0`), wird 0.… | `src/offpolicy.rs` |
| `pub fn new(epsilon: f32, num_actions: u32) -> Self` | Erstellt eine neue `RandomizedLoggingPolicy`. | `src/offpolicy.rs` |
| `pub fn new() -> Self` | Erstellt einen neuen `OffPolicyEvaluator` mit leeren Akkumulatoren. *Hinweis zur Spezifikation:* Von der Spezifikation normativ nicht expli… | `src/offpolicy.rs` |
| `pub fn observe( &mut self, target_action: u32, logged_action: u32, propensity: f32, reward: f32, )` | Beobachtet ein geloggtes Event und akkumuliert den IPS-Wert. - `target_action`: Die Aktion, die von der zu evaluierenden Ziel-Policy gewähl… | `src/offpolicy.rs` |
| `pub fn samples(&self) -> u64` | Liefert die Gesamtanzahl beobachteter Samples (inklusive verworfener). | `src/offpolicy.rs` |
| `pub fn select_action(&self, greedy_action: u32, random_sample: f32) -> (u32, f32)` | Wählt eine Aktion aus und gibt den (Aktionsindex, Propensity) Vektor/Tupel zurück. `random_sample` muss im Intervall [0.0, 1.0) liegen. | `src/offpolicy.rs` |
| `pub fn stats(&self) -> OffPolicyStats` | Liefert die Off-Policy-Evaluator-Statistiken. | `src/offpolicy.rs` |
| `pub fn current_pool_size(&self) -> Option<usize>` | Gibt die aktuell empfohlene Pool-Größe zurück. | `src/pid.rs` |
| `pub fn new( target_latency_ms: f32, min_pool_size: usize, max_pool_size: usize, initial_pool: Option<usize>, ) -> Self` | Erstellt einen neuen `PidController` mit expliziten Parametern. `min_pool_size` wird strikt auf mindestens 50 erzwungen (`min_pool_size.max… | `src/pid.rs` |
| `pub fn reset(&mut self)` | Setzt Integral und Derivative auf 0 zurück (z.B. bei Konfigurationsänderung). | `src/pid.rs` |
| `pub fn update(&mut self, dt: Duration, measured_latency_ms: f32) -> usize` | Verarbeitet eine neue Latenz-Messung und gibt die neue Pool-Größe zurück. ANTI-WINDUP: Führt Integral-Update nur durch, wenn Stellwert nich… | `src/pid.rs` |
| `pub fn budget_ms(&self) -> f64` | Returns configured budget in milliseconds. | `src/pid_latency_controller.rs` |
| `pub fn compute_adjustment(&mut self, observed_latency_ms: f64) -> f64` | Calculates scaling adjustment factor in range `[0.3, 1.0]` based on observed latency. - Observed latency > target_latency => scaling factor… | `src/pid_latency_controller.rs` |
| `pub fn elapsed_ms(&self) -> f64` | Returns elapsed time in milliseconds since guard instantiation. | `src/pid_latency_controller.rs` |
| `pub fn is_exceeded(&self) -> bool` | Returns `true` if the elapsed time has exceeded the configured budget. | `src/pid_latency_controller.rs` |
| `pub fn new(target_latency_ms: f64) -> Self` | Creates a new `PidLatencyController` with specified target latency in milliseconds. | `src/pid_latency_controller.rs` |
| `pub fn new(budget_ms: f64) -> Self` | Creates a new `LatencyBudgetGuard` with the given budget in milliseconds. | `src/pid_latency_controller.rs` |
| `pub fn observe_and_adjust(&mut self, guard: &LatencyBudgetGuard) -> f64` | Observes elapsed latency from a `LatencyBudgetGuard` and calculates scaling adjustment. | `src/pid_latency_controller.rs` |
| `pub fn reset(&mut self)` | Resets the controller integral state and error history. | `src/pid_latency_controller.rs` |
| `pub fn scale_pool_size(&mut self, observed_latency_ms: f64, base_pool_size: usize) -> usize` | Scales a base candidate pool size using the computed PID scaling adjustment factor. | `src/pid_latency_controller.rs` |
| `pub fn with_params(kp: f64, ki: f64, kd: f64, target_latency_ms: f64) -> Self` | Custom constructor specifying gains and target latency. | `src/pid_latency_controller.rs` |
| `pub fn confidence_score(&self) -> f32` | Berechnet den Konfidenz-Score im Intervall (0, 1] basierend auf der Spur der inversen Präzisionsmatrix. | `src/rie_greedy.rs` |
| `pub fn mu_hat(&self) -> Result<Vec<f32>, RieGreedyError>` | Berechnet die Parameter-Schätzung $\hat{\mu}_t = \Lambda_t^{-1} \eta_t$. | `src/rie_greedy.rs` |
| `pub fn new(dim: usize, lambda: f32, gamma: f32) -> Self` | Erstellt ein neues RIE-Greedy-Profil mit initialer Präzisionsmatrix $\Lambda_0 = \lambda I_d$. | `src/rie_greedy.rs` |
| `pub fn predict(&self, context: &[f32]) -> Result<f32, RieGreedyError>` | Berechnet die deterministisch-gierige Erwartungswert-Vorhersage $\hat{\mu}_t \cdot x$. | `src/rie_greedy.rs` |
| `pub fn trace_precision_inv(&self) -> f32` | Berechnet die Spur der inversen Präzisionsmatrix $\text{Tr}(\Lambda^{-1})$. | `src/rie_greedy.rs` |
| `pub fn uncertainty(&self) -> f32` | Liefert die mittlere Parameter-Unsicherheit des Profils ($\text{Tr}(\Lambda^{-1}) / d$). | `src/rie_greedy.rs` |
| `pub fn update(&mut self, context: &[f32], reward: f32) -> Result<(), RieGreedyError>` | Führt die Diskontierung VOR dem Sherman-Morrison Rang-1 Update durch (§10.4.1, Spec §9.2). 1. Diskontierung: $\Lambda^{-1} \leftarrow \gamm… | `src/rie_greedy.rs` |


---

## `contextra-core`

| Attribut | Wert |
|---|---|
| **Ring** | 0 — Ring 0 — Foundation & Pure Numerics/Types |
| **Status** | Fertig (Strangler Facade) |
| **Zweck** | Re-Export-Fassade über types, ports, mvcc und wire zur Abwärtskompatibilität. |
| **Unsafe-Code** | Nein (`#![forbid(unsafe_code)]`) |
| **Interne Abhängigkeiten** | `contextra-mvcc`, `contextra-ports`, `contextra-types`, `contextra-wire` |
| **Externe Abhängigkeiten (Auswahl)** | — |

### Architekturbeschreibung

`Contextra` Core — Types, traits, and error handling (Strangler Facade).  # Architecture Role (Legacy Triebwerk — Layer 0 -> Ring 0-4 Migration)  As part of the Ring-Modell Phase 1b migration, the functionality of `contextra-core` has been decomposed into modular Ring-0 crates: - [`contextra_types`]: Canonical domain models, IDs, budgets, filters, and error types. - [`contextra_ports`]: Abstract `dyn`-compatible subsystem interfaces and traits. - [`contextra_mvcc`]: Multi-version concurrency control, sequence log, and tx staging. - [`contextra_wire`]: Zero-copy FlatBuffers IPC and wire message adapters.  This crate acts as a backward-compatible Strangler Facade, re-exporting all types with deprecation markers to avoid breaking existing downstream callers.

### Modul- und Re-Export-Oberfläche (lib.rs)

```rust
pub mod traits { .. }
pub mod ipc { .. }
pub use contextra_types::error;
pub use contextra_types::error_dto;
pub use contextra_types::model_fingerprint;
pub use contextra_types::schema;
pub use contextra_types::tombstone;
pub use contextra_types::types;
pub use contextra_ports::*;
pub use contextra_mvcc::seq_log;
pub use contextra_mvcc::snapshot;
pub use contextra_mvcc::tx_buffer;
pub use contextra_wire::*;
pub use contextra_mvcc::seq_log::{SeqLogChange, SeqLogEntry, SequenceLog};
pub use contextra_mvcc::snapshot::{SnapshotGuard, SnapshotRegistry};
pub use contextra_mvcc::tx_buffer::{IndexOp, TxBuffer};
pub use contextra_ports::*;
pub use contextra_types::error::{ContextraError, Result};
pub use contextra_types::error_dto::ContextraErrorDto;
pub use contextra_types::model_fingerprint::ModelFingerprint;
pub use contextra_types::schema::{DocIdWidth, ManifestSchemaVersion};
pub use contextra_types::tombstone::{SeqBitTombstone, TombstoneSemanticsCheck};
```

### Feature-Flags

| Flag | Aktiviert |
|---|---|
| `test-utils` | `[]` |


---

## `contextra-audit-export`

| Attribut | Wert |
|---|---|
| **Ring** | 0 — Ring 0 — Foundation & Pure Numerics/Types |
| **Status** | Fertig |
| **Zweck** | Generator für Verarbeitungsverzeichnisse gemäß DSGVO Art. 30. |
| **Unsafe-Code** | Nein (`#![forbid(unsafe_code)]`) |
| **Interne Abhängigkeiten** | `contextra-ports`, `contextra-types` |
| **Externe Abhängigkeiten (Auswahl)** | `serde`, `serde_json`, `thiserror` |

### Architekturbeschreibung

`contextra-audit-export` — GDPR Article 30 Processing Register export generator.  # Architecture Role (Ring 4) This crate provides an external interface for generating automated processing registers (Verzeichnis von Verarbeitungstätigkeiten nach Art. 30 DSGVO) based on egress gateway logs and cryptographic deletion proofs.

### Modul- und Re-Export-Oberfläche (lib.rs)

```rust
pub mod bsi_mapping;
pub mod error;
pub mod markdown_template;
pub mod testkit;
pub use bsi_mapping::{bsi_mapping_table, render_bsi_mapping_markdown, BsiMappingEntry};
pub use error::AuditExportError;
pub use markdown_template::{render_art30_register_markdown, render_register_markdown};
```

### Schnittstellen — Public Traits (1)

**`ProcessingRegisterSource`** — *Abstract data source for retrieving processing register records.* (`src/lib.rs`)

```rust
pub trait ProcessingRegisterSource {
/// Collects processing register entries for the specified tenant.
    fn collect_entries(
        &self,
        tenant_id: TenantId,
    ) -> Result<Vec<ProcessingRegisterEntry>, AuditExportError>;
}
```

### Öffentliche Datentypen — Structs (7)

| Struct | Felder / Form | Beschreibung | Datei |
|---|---|---|---|
| `Art30ProcessingRegisterEntry` | `{ /// Associated tenant identifier (`contextra_types::TenantId`). pub tenant_id: TenantId, /// Name and contact details of controller/representative/DPO (Art. 30… }` | Complete processing register record according to GDPR Article 30 (Art. 30 Abs. 1 lit. a–g DSGVO). | `src/lib.rs` |
| `AuditRegisterExporter` | `{ source: S, }` | Exporter for generating GDPR Article 30 processing register exports from a [`ProcessingRegisterSource`]. | `src/lib.rs` |
| `BsiMappingEntry` | `{ /// Name of the cryptographic primitive (e.g. "Ed25519"). pub primitive_name: String, /// Code location path within the workspace (e.g. "crates/contextra-crypt… }` | A mapping entry linking a cryptographic primitive in the codebase to a BSI Technical Guideline (Spec §2.4 Punkt 2, §16.3). | `src/bsi_mapping.rs` |
| `DeletionProofSummary` | `{ /// Unique identifier of the deletion proof. pub proof_id: String, /// Target resource scope or deleted item identifier. pub scope: String, /// Timestamp in Un… }` | Summary of a cryptographic deletion proof (GDPR Art. 17 / Art. 30 compliance). | `src/lib.rs` |
| `EgressEventSummary` | `{ /// Unique identifier of the egress event. pub event_id: String, /// Target destination or external endpoint. pub destination: String, /// Timestamp in Unix na… }` | Summary of an egress gateway event recorded for audit verification. | `src/lib.rs` |
| `InMemoryProcessingRegisterSource` | `{ entries: Vec<ProcessingRegisterEntry>, }` | In-memory implementation of [`ProcessingRegisterSource`] for testing purposes. | `src/testkit.rs` |
| `ProcessingRegisterEntry` | `{ /// Associated tenant identifier (`contextra_types::TenantId`). pub tenant_id: TenantId, /// Specific purpose of processing (Verarbeitungszweck). pub processin… }` | A record entry in the GDPR Article 30 processing register. | `src/lib.rs` |

### Öffentliche Datentypen — Enums (1)

| Enum | Varianten | Beschreibung | Datei |
|---|---|---|---|
| `AuditExportError` | `/// Failure during JSON serialization or formatting. #[error("Serialization error: {0}")] Serialization(#[from] serde_json::Error), /// Failure during register data collection. #[error("Source collec…` | Error type returned by operations in `contextra-audit-export`. | `src/error.rs` |

### Öffentliche Funktionen (15)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn bsi_mapping_table() -> Vec<BsiMappingEntry>` | Returns the static, code-maintained cryptographic mapping table for Contextra. Serves as the Single Source of Truth for audit exports and c… | `src/bsi_mapping.rs` |
| `pub fn render_bsi_mapping_markdown(entries: &[BsiMappingEntry]) -> String` | Renders the BSI Grundschutz / TR-02102 cryptographic mapping table as Markdown. Consistent with the rendering style in [`crate::markdown_te… | `src/bsi_mapping.rs` |
| `pub fn add_entry(&mut self, entry: ProcessingRegisterEntry)` | Appends a processing register entry to the underlying in-memory register source. | `src/lib.rs` |
| `pub fn export_json(&self, tenant_id: TenantId) -> Result<String, AuditExportError>` | Exports the processing register entries for `tenant_id` as a formatted JSON string. | `src/lib.rs` |
| `pub fn export_markdown(&self, tenant_id: TenantId) -> Result<String, AuditExportError>` | Exports the processing register entries for `tenant_id` as a Markdown document. | `src/lib.rs` |
| `pub fn new(source: S) -> Self` | Creates a new [`AuditRegisterExporter`] backed by the specified source. | `src/lib.rs` |
| `pub fn render_register_json( entries: &[ProcessingRegisterEntry], ) -> Result<String, AuditExportError>` | Renders a list of processing register entries as a formatted JSON string. | `src/lib.rs` |
| `pub fn source(&self) -> &S` | Returns a reference to the underlying processing register source. | `src/lib.rs` |
| `pub fn source_mut(&mut self) -> &mut S` | Returns a mutable reference to the underlying processing register source. | `src/lib.rs` |
| `pub fn with_sample_data_for(tenant_id: TenantId) -> Self` | Creates an exporter pre-populated with synthetic sample processing register entries for `tenant_id`. | `src/lib.rs` |
| `pub fn render_art30_register_markdown( entries: &[Art30ProcessingRegisterEntry], ) -> Result<String, AuditExportError>` | Renders a list of [`Art30ProcessingRegisterEntry`] records as a formatted Markdown document containing all mandatory Article 30 GDPR attrib… | `src/markdown_template.rs` |
| `pub fn render_register_markdown( entries: &[ProcessingRegisterEntry], ) -> Result<String, AuditExportError>` | Renders a list of [`ProcessingRegisterEntry`] records as a formatted Markdown document containing a GDPR Article 30 processing register tab… | `src/markdown_template.rs` |
| `pub fn add_entry(&mut self, entry: ProcessingRegisterEntry)` | Appends a custom [`ProcessingRegisterEntry`] record. | `src/testkit.rs` |
| `pub fn new() -> Self` | Creates a new empty [`InMemoryProcessingRegisterSource`]. | `src/testkit.rs` |
| `pub fn with_sample_data_for(tenant_id: TenantId) -> Self` | Creates an [`InMemoryProcessingRegisterSource`] pre-populated with synthetic sample data for `tenant_id`. | `src/testkit.rs` |


---

## `contextra-avv-generator`

| Attribut | Wert |
|---|---|
| **Ring** | 0 — Ring 0 — Foundation & Pure Numerics/Types |
| **Status** | Fertig |
| **Zweck** | Template-Generator für Auftragsverarbeitungsverträge (AVV). |
| **Unsafe-Code** | Nein (`#![forbid(unsafe_code)]`) |
| **Interne Abhängigkeiten** | `contextra-types` |
| **Externe Abhängigkeiten (Auswahl)** | `serde`, `thiserror` |

### Modul- und Re-Export-Oberfläche (lib.rs)

```rust
pub mod error;
pub mod template;
pub use error::AvvGeneratorError;
```

### Öffentliche Datentypen — Structs (3)

| Struct | Felder / Form | Beschreibung | Datei |
|---|---|---|---|
| `AvvContext` | `{ /// Controller / Responsible Party Name (e.g. Kanzlei / Praxis) pub controller_name: String, /// Processor / Operator Name (Contextra Operator) pub processor_n… }` | Context parameter container for generating an AVV document. | `src/lib.rs` |
| `AvvContextBuilder` | `{ ctx: AvvContext, }` | Builder for constructing and customizing an [`AvvContext`]. | `src/lib.rs` |
| `TechnicalMeasure` | `{ /// Name of measure (e.g. "Kryptografischer Löschbeweis (Ed25519)") pub name: String, /// Detailed technical description of measure pub description: String, //… }` | Description of a Technical and Organizational Measure (TOM). | `src/lib.rs` |

### Öffentliche Datentypen — Enums (1)

| Enum | Varianten | Beschreibung | Datei |
|---|---|---|---|
| `AvvGeneratorError` | `/// A required field in the context was empty or invalid. #[error("Ungültiger Kontext: {0}")] InvalidContext(String), /// Error during template rendering or formatting. #[error("Render-Fehler: {0}")]…` | Errors that can occur during AVV template rendering. | `src/error.rs` |

### Öffentliche Funktionen (16)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn add_subprocessor(mut self, subprocessor: impl Into<String>) -> Self` | Appends a subprocessor to the list. | `src/lib.rs` |
| `pub fn add_technical_measure(mut self, measure: TechnicalMeasure) -> Self` | Appends a technical measure to the list. | `src/lib.rs` |
| `pub fn build(self) -> AvvContext` | Returns the built [`AvvContext`]. | `src/lib.rs` |
| `pub fn builder( controller_name: impl Into<String>, processor_name: impl Into<String>, tenant_id: TenantId, ) -> AvvContextBuilder` | — | `src/lib.rs` |
| `pub fn default_technical_measures() -> Vec<TechnicalMeasure>` | Associated helper returning Contextra's default list of technical measures. | `src/lib.rs` |
| `pub fn default_technical_measures() -> Vec<TechnicalMeasure>` | Returns the default list of technical measures derived from Contextra's product guarantees. | `src/lib.rs` |
| `pub fn new( controller_name: impl Into<String>, processor_name: impl Into<String>, tenant_id: TenantId, ) -> Self` | Creates a new [`AvvContext`] initialized with Contextra's default technical measures and SLA (14 days). | `src/lib.rs` |
| `pub fn new( controller_name: impl Into<String>, processor_name: impl Into<String>, tenant_id: TenantId, ) -> Self` | Creates a new [`AvvContextBuilder`] initialized with default technical measures. | `src/lib.rs` |
| `pub fn render(&self) -> Result<String, AvvGeneratorError>` | Renders this AVV context into a Markdown document using [`render_avv_markdown`]. | `src/lib.rs` |
| `pub fn render(self) -> Result<String, AvvGeneratorError>` | Directly renders the built [`AvvContext`] into a Markdown document using [`render_avv_markdown`]. | `src/lib.rs` |
| `pub fn render_avv_markdown(ctx: &AvvContext) -> Result<String, AvvGeneratorError>` | Renders the AVV document in Markdown format using the provided [`AvvContext`]. | `src/lib.rs` |
| `pub fn with_default_technical_measures(mut self) -> Self` | Resets technical measures to Contextra's default set. | `src/lib.rs` |
| `pub fn with_deletion_sla_days(mut self, days: u32) -> Self` | Sets the SLA for deletion in days. | `src/lib.rs` |
| `pub fn with_subprocessors(mut self, subprocessors: Vec<String>) -> Self` | Overrides the list of subprocessors. | `src/lib.rs` |
| `pub fn with_technical_measures(mut self, measures: Vec<TechnicalMeasure>) -> Self` | Overrides the list of technical measures. | `src/lib.rs` |
| `pub fn render(ctx: &AvvContext) -> Result<String, AvvGeneratorError>` | Renders the AVV Markdown document from the provided [`AvvContext`]. | `src/template.rs` |


---

## `contextra-license`

| Attribut | Wert |
|---|---|
| **Ring** | 0 — Ring 0 — Foundation & Pure Numerics/Types |
| **Status** | Fertig |
| **Zweck** | Lizenzierungs- und Aktivierungsprüfung für Feature-Ringe. |
| **Unsafe-Code** | Nein (`#![forbid(unsafe_code)]`) |
| **Interne Abhängigkeiten** | `contextra-ports`, `contextra-types` |
| **Externe Abhängigkeiten (Auswahl)** | `bincode`, `blake3`, `ed25519-dalek`, `serde`, `thiserror`, `tracing` |

### Modul- und Re-Export-Oberfläche (lib.rs)

```rust
pub mod signed_gate;
pub use contextra_ports::license::{FeatureRing, LicenseError, LicenseGate, OpenFastGate};
pub use signed_gate::{ derive_local_installation_id_hash, LicensePayload, SignedActivation, SignedLicenseGate, };
```

### Öffentliche Datentypen — Structs (3)

| Struct | Felder / Form | Beschreibung | Datei |
|---|---|---|---|
| `LicensePayload` | `{ /// Tenant identity bound to this license. pub tenant_id: TenantId, /// List of feature rings activated by this license. pub allowed_rings: Vec<FeatureRing>, /… }` | License payload containing tenant binding, granted feature rings, optional expiration timestamp, arbitrary feature flags, and optional installation I… | `src/signed_gate.rs` |
| `SignedActivation` | `{ /// Activated feature ring. pub ring: FeatureRing, /// 32-byte hash of the target local installation identity. pub installation_id_hash: [u8; 32], /// Expirati… }` | Signed activation record binding a feature ring activation to a specific machine/deployment installation hash (§10.2, §15). | `src/signed_gate.rs` |
| `SignedLicenseGate` | `{ verifying_key: Option<VerifyingKey>, activation: Option<SignedActivation>, license_payload: Option<LicensePayload>, local_installation_id: Option<[u8; 32]>, cl… }` | Cryptographic license gate verifying Ed25519 signatures over activation records ([`SignedActivation`]) or legacy [`LicensePayload`]. | `src/signed_gate.rs` |

### Öffentliche Funktionen (20)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn activation(&self) -> Option<&SignedActivation>` | Returns a reference to the verified [`SignedActivation`], if present. | `src/signed_gate.rs` |
| `pub fn create_signed( ring: FeatureRing, installation_id_hash: [u8; 32], expires_at_unix: i64, signing_key: &ed25519_dalek::SigningKey, ) -> Self` | Creates a signed activation record given data fields and an Ed25519 signing key. | `src/signed_gate.rs` |
| `pub fn create_test_signed_payload( rings: Vec<FeatureRing>, expires_at: Option<i64>, ) -> (Vec<u8>, [u8` | Generates a valid signed payload, signature, and verifying key bytes for testing purposes. | `src/signed_gate.rs` |
| `pub fn derive_local_installation_id_hash(storage_path: Option<&std::path::Path>) -> [u8` | Determines or generates a deterministic local installation ID hash (`[u8; 32]`). # Status **VORLÄUFIG, BIS ZUR SCHRIFTLICHEN ENTSCHEIDUNG**… | `src/signed_gate.rs` |
| `pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8` | — | `src/signed_gate.rs` |
| `pub fn for_testing(rings: Vec<FeatureRing>, expires_at: Option<i64>) -> Self` | Constructs a [`SignedLicenseGate`] configured with a test signed payload for development or testing. | `src/signed_gate.rs` |
| `pub fn from_activation(activation: SignedActivation, verifying_key: VerifyingKey) -> Self` | Constructs a [`SignedLicenseGate`] from a [`SignedActivation`]. | `src/signed_gate.rs` |
| `pub fn from_activation_params( ring: FeatureRing, installation_id_hash: [u8; 32], expires_at_unix: i64, signing_key: &ed25519_dalek::SigningKey, verifying_key: VerifyingKey, ) -> Self` | Constructs a [`SignedLicenseGate`] directly from activation parameters by creating a [`SignedActivation`]. | `src/signed_gate.rs` |
| `pub fn from_activation_with_clock( activation: SignedActivation, verifying_key: VerifyingKey, clock: Arc<dyn Clock>, ) -> Self` | Constructs a [`SignedLicenseGate`] from a [`SignedActivation`] with an injected [`Clock`]. | `src/signed_gate.rs` |
| `pub fn from_signed_payload( payload_bytes: &[u8], signature: &[u8; 64], verifying_key: VerifyingKey, ) -> Result<Self, LicenseError>` | Constructs a [`SignedLicenseGate`] by verifying an Ed25519 signature over `payload_bytes` using standard system wall-clock. # Errors Return… | `src/signed_gate.rs` |
| `pub fn from_signed_payload_with_clock( payload_bytes: &[u8], signature: &[u8; 64], verifying_key: VerifyingKey, clock: Arc<dyn Clock>, ) -> Result<Self, LicenseError>` | Constructs a [`SignedLicenseGate`] with an injected [`Clock`] instance for deterministic testing (P28). # Errors Returns [`LicenseError::In… | `src/signed_gate.rs` |
| `pub fn license_payload(&self) -> Option<&LicensePayload>` | Returns a reference to the verified [`LicensePayload`], if present. | `src/signed_gate.rs` |
| `pub fn no_activation() -> Self` | Constructs a [`SignedLicenseGate`] without any active activation record. | `src/signed_gate.rs` |
| `pub fn payload_bytes( ring: FeatureRing, installation_id_hash: [u8; 32], expires_at_unix: i64, ) -> Vec<u8>` | Computes the binary byte representation of the activation data fields `(ring, installation_id_hash, expires_at_unix)` used for Ed25519 sign… | `src/signed_gate.rs` |
| `pub fn serialize<S>(bytes: &[u8; 64], serializer: S) -> Result<S::Ok, S::Error> where S: Serializer,` | — | `src/signed_gate.rs` |
| `pub fn unactivated_local(storage_path: Option<&std::path::Path>) -> Self` | Constructs an unactivated [`SignedLicenseGate`] bound to the auto-derived local installation ID. | `src/signed_gate.rs` |
| `pub fn verify_signature(&self, verifying_key: &VerifyingKey) -> bool` | Verifies whether the Ed25519 signature on this activation record is valid for the given verifying key. | `src/signed_gate.rs` |
| `pub fn verifying_key(&self) -> Option<&VerifyingKey>` | Returns a reference to the public Ed25519 verifying key, if present. | `src/signed_gate.rs` |
| `pub fn with_auto_installation_id(self, storage_path: Option<&std::path::Path>) -> Self` | Binds the auto-derived local installation ID hash (using optional persistent storage path) to this gate instance. | `src/signed_gate.rs` |
| `pub fn with_local_installation_id(mut self, id_hash: [u8; 32]) -> Self` | Binds a local installation ID hash (e.g. BLAKE3 hash of machine ID or MAC address) to this gate instance. | `src/signed_gate.rs` |


---

## `contextra-store`

| Attribut | Wert |
|---|---|
| **Ring** | 1 — Ring 1 — Persistence & Cache |
| **Status** | Fertig |
| **Zweck** | LSM-Tree Storage Engine mit WAL-Group-Commit und HMAC-Integritätsprüfung. |
| **Unsafe-Code** | Nein (`#![forbid(unsafe_code)]`) |
| **Interne Abhängigkeiten** | `contextra-core`, `contextra-crypto`, `contextra-mvcc`, `contextra-ports`, `contextra-sys`, `contextra-types` |
| **Externe Abhängigkeiten (Auswahl)** | `ahash`, `blake3`, `bytes`, `crc32fast`, `lru`, `memmap2`, `parking_lot`, `quick_cache`, `rand`, `serde`, `thiserror`, `tokio`, `tokio-util`, `tracing`, `uuid` |

### Modul- und Re-Export-Oberfläche (lib.rs)

```rust
pub mod compaction;
pub mod engine;
pub mod kv;
pub mod lsm;
pub mod manifest;
pub mod memtable;
pub mod sstable;
pub mod system_pressure;
pub mod tenant_codec;
pub mod wal;
pub use compaction::{ adaptive::{ AdaptiveCompactionPlan, AdaptiveCompactionPlanner, CompactionStrategy, CostBasedAdaptivePlanner, WorkloadMetrics, WorkloadMetricsSnapshot, }, CompactionConfig, CompactionEngine, MergeOperator, TtlMetadata, };
pub use lsm::{ CommittedBatch, LsmConfig, LsmStorage, ObserverRegistry, WalEntryRef, WalObserver, WriteOrigin, DEFAULT_MAX_OBSERVER_LATENCY, };
pub use manifest::{Manifest, ManifestEntry};
pub use system_pressure::{PressureLevel, SystemPressure, SystemPressureMonitor};
pub use tenant_codec::{TenantKeyCodec, TenantScopedStorage};
```

### Feature-Flags

| Flag | Aktiviert |
|---|---|
| `default` | `["wal-integrity"]` |
| `encryption-at-rest` | `["wal-integrity"]` |
| `wal-integrity` | `[]` |
| `deletion-proof` | `["wal-integrity"]` |
| `memory-only-storage` | `[]` |
| `fault-injection` | `[]` |
| `block-cache-v2` | `["dep:quick_cache"]` |
| `docid-128` | `["contextra-types/docid-128"]` |
| `legacy-wal-key` | `[]` |

### Schnittstellen — Public Traits (6)

**`AdaptiveCompactionPlanner`** — *Trait defining adaptive compaction strategy selection.* (`src/compaction/adaptive.rs`)

```rust
pub trait AdaptiveCompactionPlanner : Send + Sync {
/// Derives a compaction decision based on storage stats and operational workload metrics.
    fn plan_compaction(
        &self,
        stats: &StorageStats,
        metrics: &WorkloadMetricsSnapshot,
        sstables: &[Arc<SstableReader>],
        min_active_seqno: u64,
    ) -> Result<Option<AdaptiveCompactionPlan>>;
}
```

**`BlockCacheBackend`** — ** (`src/sstable/block_cache.rs`)

```rust
pub trait BlockCacheBackend : Send + Sync {
/// Creates a new cache backend instance with the specified capacity.
    fn new(capacity: usize) -> Self
    where
        Self: Sized;

    /// Retrieves a cached block by `(file_id, offset)` key.
    fn get(&self, key: &(u64, u64)) -> Option<Bytes>;

    /// Inserts a block into the cache.
    fn insert(&self, key: (u64, u64), value: Bytes);

    /// Returns the number of cached blocks in this backend instance.
    fn len(&self) -> usize;

    /// Returns `true` if the backend contains no cached blocks.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Checks if a key is present in the cache.
    fn contains(&self, key: &(u64, u64)) -> bool {
        self.get(key).is_some()
    }
}
```

**`MergeOperator`** — *Trait for merging two raw byte values (`existing_val` from older tier, `new_val` from newer tier) for the same key across different SSTable tiers.* (`src/compaction/merge_operator.rs`)

```rust
pub trait MergeOperator : Send + Sync {
/// Merges two raw byte values for the same key.
    ///
    /// # Fail-Safe Requirement
    /// Implementations running untrusted guest code (e.g. WASM via `contextra-sandbox`) MUST
    /// implement fail-safe handling: if execution fails (e.g. fuel exhaustion or trap),
    /// the caller or operator must swallow the error for that single entry, log a `tracing::warn!`,
    /// and retain both original versions in the compacted SSTable without aborting the overall compaction.
    fn merge(&self, existing_val: &[u8], new_val: &[u8]) -> Result<Vec<u8>>;
}
```

**`ReplayProgressSink`** — ** (`src/wal/replay.rs`)

```rust
pub trait ReplayProgressSink : Send + Sync {
fn on_entry_replayed(&self, seq: WalSeq, entries_total_estimate: Option<u64>);
}
```

**`StorageEngineHandle`** — *Internal trait abstraction enabling `TenantScopedStorage` to wrap both `S: StorageEngine` and `Arc<S>`.* (`src/tenant_codec.rs`)

```rust
pub trait StorageEngineHandle : Send + Sync + 'static {
fn get_handle<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, Result<Option<Bytes>>>;
    fn get_at_seq_handle<'a>(
        &'a self,
        key: &'a [u8],
        seq_no: u64,
    ) -> BoxFuture<'a, Result<Option<Bytes>>>;
    fn get_tracked_handle<'a>(
        &'a self,
        tx_id: TxId,
        key: &'a [u8],
    ) -> BoxFuture<'a, Result<Option<Bytes>>>;
    fn get_at_seq_tracked_handle<'a>(
        &'a self,
        tx_id: TxId,
        key: &'a [u8],
        seq: u64,
    ) -> BoxFuture<'a, Result<Option<Bytes>>>;
    #[allow(clippy::type_complexity)]
    fn scan_prefix_tracked_handle<'a>(
        &'a self,
        tx_id: TxId,
        prefix: &'a [u8],
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>>;
    fn supports_ssi_tracking_handle(&self) -> bool;
    fn put_handle<'a>(
        &'a self,
        tx_id: TxId,
        key: &'a [u8],
        value: &'a [u8],
    ) -> BoxFuture<'a, Result<()>>;
    fn put_if_absent_handle<'a>(
        &'a self,
        tx_id: TxId,
        key: &'a [u8],
        value: &'a [u8],
    ) -> BoxFuture<'a, Result<bool>>;
    fn put_batch_handle<'a>(
        &'a self,
        tx_id: TxId,
        entries: &'a [(Vec<u8>, Vec<u8>)],
    ) -> BoxFuture<'a, Result<()>>;
    fn delete_handle<'a>(&'a self, tx_id: TxId, key: &'a [u8]) -> BoxFuture<'a, Result<()>>;
    fn delete_many_handle<'a>(
        &'a self,
        tx_id: TxId,
        keys: Vec<Vec<u8>>,
    ) -> BoxFuture<'a, Result<u64>>;
    fn delete_prefix_handle<'a>(
        &'a self,
        tx_id: TxId,
        prefix: &'a [u8],
    ) -> BoxFuture<'a, Result<u64>>;
    fn commit_handle<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>>;
    fn rollback_handle<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>>;
    fn rollback_to_tx_handle<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>>;
    fn flush_handle<'a>(&'a self) -> BoxFuture<'a, Result<()>>;
    fn stats_handle<'a>(&'a self) -> BoxFuture<'a, Result<StorageStats>>;
    fn last_seq_no_handle<'a>(&'a self) -> BoxFuture<'a, Result<u64>>;
    fn last_tx_id_handle<'a>(&'a self) -> BoxFuture<'a, Result<TxId>>;
    fn pin_checkpoint_handle<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, Result<()>>;
    fn unpin_checkpoint_handle<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, Result<()>>;
    #[allow(clippy::type_complexity)]
    fn scan_prefix_handle<'a>(
        &'a self,
        prefix: &'a [u8],
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>>;
    #[allow(clippy::type_complexity)]
    fn scan_prefix_bounded_handle<'a>(
        &'a self,
        prefix: &'a [u8],
        limit: usize,
        cursor: Option<&'a [u8]>,
    ) -> BoxFuture<'a, Result<(Vec<(Vec<u8>, Vec<u8>)>, Option<Vec<u8>>)>>;
    #[allow(clippy::type_complexity)]
    fn scan_prefix_at_handle<'a>(
        &'a self,
        prefix: &'a [u8],
        seq_no: u64,
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>>;
    #[allow(clippy::type_complexity)]
    fn scan_handle<'a>(
        &'a self,
        start: Bound<&'a [u8]>,
        end: Bound<&'a [u8]>,
        limit: Option<usize>,
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>>;
    #[allow(clippy::type_complexity)]
    fn scan_bounded_handle<'a>(
        &'a self,
        start: Bound<&'a [u8]>,
        end: Bound<&'a [u8]>,
        limit: usize,
        cursor: Option<&'a [u8]>,
    ) -> BoxFuture<'a, Result<(Vec<(Vec<u8>, Vec<u8>)>, Option<Vec<u8>>)>>;
}
```

**`WalObserver`** — *Synchronous, deterministic observer interface for committed WAL batches.* (`src/lsm/observer.rs`)

```rust
pub trait WalObserver : Send + Sync {
/// Indicates whether this observer requires durable WAL commits.
    ///
    /// Returns `false` by default. Observers setting this to `false` will also receive notifications
    /// for `DurabilityMode::MemoryOnly` commits.
    fn requires_durability(&self) -> bool {
        false
    }

    /// Notification callback executed upon batch commit.
    ///
    /// Invoked via a bounded, non-blocking dispatcher mechanism. The commit path enforces a
    /// configurable maximum latency deadline (`max_observer_latency`). Observers taking longer
    /// than this deadline will time out on the commit path, increment drop counters, and trip
    /// the circuit breaker to prevent blocking subsequent transaction commits.
    fn on_commit(&self, batch: &CommittedBatch<'_>, seq_no: u64, tx_id: TxId);
}
```

### Öffentliche Datentypen — Structs (39)

| Struct | Felder / Form | Beschreibung | Datei |
|---|---|---|---|
| `AdaptiveCompactionPlan` | `{ pub strategy: CompactionStrategy, pub candidates: Vec<Arc<SstableReader>>, pub is_full_compaction: bool, }` | Resulting plan proposed by adaptive compaction planner. | `src/compaction/adaptive.rs` |
| `AsyncObserverAdapter` | `{ inner: Arc<dyn WalObserver>, sender: tokio::sync::mpsc::Sender<(u64, TxId, WriteOrigin)>, }` | Non-blocking, bounded async adapter wrapper for `WalObserver` implementations. | `src/lsm/observer.rs` |
| `BlockBuilder` | `{ data: BytesMut, offsets: Vec<u32>, pub(super) block_size: usize, key_hashes: Vec<(u64, u64)>, v3_bloom: [u64; 4], format_version: u16, }` | A builder for SSTable data blocks (Format v4 default, supports v3). | `src/sstable/builder.rs` |
| `BlockCache` | `{ shards: Vec<BlockCacheShard>, hash_builder: ahash::RandomState, }` | — | `src/sstable/block_cache.rs` |
| `BloomFilter` | `{ pub(super) bits: Vec<u64>, pub(super) num_hashes: usize, pub(super) num_bits: usize, }` | SPECCED: Speichereffizienter Bloom-Filter für SSTable-Pre-Checks. Ziel: False-Positive-Rate p ≤ 0.01. Formel: m = -n * ln(p) / (ln(2)^2) ≈ 9.6 * n Ha… | `src/sstable/bloom.rs` |
| `CommitContext` | `{ pub tx_id: TxId, pub origin: WriteOrigin, pub durable: bool, }` | Context describing a committed transaction passed to observers. | `src/lsm/observer.rs` |
| `CommitGuard` | `{ pub(super) _lock: &'a tokio::sync::MutexGuard<'a, ()>, }` | Proof that `commit_mutex` is currently held by the calling task. Can only be constructed while holding the mutex guard. | `src/lsm/guard.rs` |
| `CommittedBatch` | `{ pub entries: Vec<WalEntryRef<'a>>, pub origin: WriteOrigin, }` | Batch of committed WAL entries passed to observers. | `src/lsm/observer.rs` |
| `CompactionConfig` | `{ /// Minimum number of SSTables in a size tier to trigger compaction. pub min_sstables_per_tier: usize, /// Size ratio between adjacent tiers (e.g., 4.0 means e… }` | Configuration for the compaction engine. | `src/compaction/config.rs` |
| `CompactionEngine` | `{ config: CompactionConfig, pub(super) snapshot_registry: Arc<SnapshotRegistry>, block_cache: Arc<BlockCache>, key_manager: Option<Arc<KeyManager>>, budget: Arc<… }` | — | `src/compaction/engine.rs` |
| `CostBasedAdaptivePlanner` | `{ read_ratio_threshold: f64, min_sstables_per_tier: usize, size_ratio: f64, }` | Reference implementation of cost-based adaptive compaction (EcoTune / ArceKV inspired). Adjusts compaction aggressiveness based on read-vs-write work… | `src/compaction/adaptive.rs` |
| `InMemoryStorageEngine` | `{ data: Arc<RwLock<StoreDataMap>>, staged: Arc<RwLock<StagedTxMap>>, seq: Arc<AtomicU64>, last_tx: Arc<RwLock<TxId>>, }` | An in-memory implementation of [`StorageEngine`] guaranteed to perform no disk I/O. | `src/engine/memory_only.rs` |
| `KvSegmentConfig` | `{ /// Delete mode specifying whether segments are tombstone-only or crypto-shredded. pub delete_mode: KvDeleteMode, }` | Configuration for KV segment storage and shredding behavior. | `src/kv/segment.rs` |
| `KvSegmentManager` | `{ config: KvSegmentConfig, registry: Arc<KeyRegistry>, master_key: Option<Arc<KeyManager>>, }` | Manager for KV segment encryption, decryption, and crypto-shredding. | `src/kv/segment.rs` |
| `KvSegmentPayload` | `{ /// Group ID corresponding to the sub-key in `KeyRegistry`. pub group_id: u64, /// Physical payload stored on disk (ciphertext or raw plaintext). pub payload:… }` | Representation of a written KV segment (encrypted or raw payload with group_id and nonce). | `src/kv/segment.rs` |
| `LruBlockCacheBackend` | `{ state: RwLock<LruBlockCacheState>, }` | Standard strict-LRU block cache backend using `parking_lot::RwLock<LruCache>` with byte-based capacity. | `src/sstable/block_cache.rs` |
| `LsmConfig` | `{ /// Path to the data directory. pub path: PathBuf, /// Maximum size of the memtable before flushing to disk. pub memtable_size_limit: usize, /// Maximum RAM us… }` | Configuration for the LSM storage engine. | `src/lsm/config.rs` |
| `LsmState` | `{ pub(super) memtable: Arc<MemTable>, pub(super) immutable_memtables: Vec<Arc<MemTable>>, }` | — | `src/lsm/guard.rs` |
| `LsmStorage` | `{ pub(super) config: LsmConfig, pub(super) key_manager: Option<Arc<KeyManager>>, pub(super) state: RwLock<LsmState>, /// SSTables stored separately for shared ac… }` | LSM-Tree based storage engine. | `src/lsm/engine.rs` |
| `Manifest` | `{ pub(super) path: PathBuf, pub(super) file: tokio::sync::Mutex<tokio::fs::File>, pub(super) _dir_lock: Option<DirLock>, }` | Append-only Manifest file handle for recording active SSTable set transitions. | `src/manifest/core.rs` |
| `MemTable` | `{ /// Sharded storage: each shard holds a disjoint subset of keys. /// Shard selection is deterministic via `shard_for()`. shards: [MemTableShard; SHARD_COUNT],… }` | — | `src/memtable.rs` |
| `NoopReplayProgressSink` | `(tuple/unit)` | — | `src/wal/replay.rs` |
| `ObserverRegistry` | `{ observers: parking_lot::RwLock<Vec<Arc<BoundedObserverWorker>>>, historical_drops: Arc<AtomicU64>, max_observer_latency: parking_lot::RwLock<Duration>, clock:… }` | Configuration and handle for managing WAL observers with bounded delivery and circuit breaker. | `src/lsm/observer.rs` |
| `PreparedBatch` | `(tuple/unit)` | A batch of prepared WAL entries bound to a specific HMAC chain. `PreparedBatch` has a private inner field and cannot be constructed outside `wal.rs`.… | `src/wal/encode.rs` |
| `QuickCacheBlockCacheBackend` | `{ cache: quick_cache::sync::Cache<(u64, u64), Bytes, BlockWeighter>, }` | Lock-optimized block cache backend using `quick_cache::sync::Cache` (S3-FIFO / Clock-based eviction) with byte-based capacity. | `src/sstable/block_cache.rs` |
| `SieveCacheBackend` | `{ state: RwLock<SieveCacheState>, }` | Trait abstraction for block cache backend implementations. ## Approximative LRU & Lock-Optimized Read Path SSTable block caches store immutable 4KB d… | `src/sstable/block_cache.rs` |
| `SstableBuilder` | `{ path: PathBuf, file: File, block_builder: BlockBuilder, index: Vec<(Bytes, u64)>, // (last_key, offset) first_key: Option<Bytes>, last_key: Option<Bytes>, prev… }` | A builder for creating new SSTables. Note: Uses a whole-SSTable Bloom filter with a default FPR. The Bloom filter FPR should be treated as a tunable… | `src/sstable/builder.rs` |
| `SstableMetadata` | `{ pub first_key: Bytes, pub last_key: Bytes, pub file_size: u64, pub min_tx_id: u64, pub max_tx_id: u64, pub min_seq: u64, pub max_seq: u64, }` | Metadata for an SSTable. | `src/sstable/builder.rs` |
| `SstableReader` | `{ pub(super) file: Arc<std::fs::File>, pub(super) index: Vec<(Bytes, u64)>, pub(super) metadata: SstableMetadata, /// Byte offset where the index data begins (=… }` | A reader for existing SSTables. | `src/sstable/reader.rs` |
| `SstableStream` | `{ pub(super) reader: Arc<SstableReader>, pub(super) block_idx: usize, pub(super) entry_idx: usize, pub(super) current_block: Option<Bytes>, pub(super) num_offset… }` | — | `src/sstable/stream.rs` |
| `SystemPressure` | `{ pub wal_queue_depth: usize, pub scheduler_queue_depth: f32, // 0.0–1.0 (Tokio global scheduler queue depth ratio) pub blocking_util: f32, // 0.0–1.0 (Blocking… }` | — | `src/system_pressure.rs` |
| `SystemPressureMonitor` | `{ pressure_tx: tokio::sync::watch::Sender<SystemPressure>, pub pressure_rx: tokio::sync::watch::Receiver<SystemPressure>, sampling_interval: Duration, }` | — | `src/system_pressure.rs` |
| `TenantKeyCodec` | `{ tenant_id: TenantId, /// Vorberechnetes Scan-Präfix ohne Allokation im Hot-Path. scan_prefix_cache: Vec<u8>, }` | Encodes LSM storage keys with tenant isolation prefixes. | `src/tenant_codec.rs` |
| `TenantScopedStorage` | `{ inner: S, codec: TenantKeyCodec, }` | Tenant-aware `StorageEngine` wrapper implementing strict key-level isolation (INV-TENANT-2). | `src/tenant_codec.rs` |
| `WalEntry` | `{ /// The operation performed. pub op: WalOp, /// Sequence number assigned to the operation. pub seq_no: u64, /// HMAC of the current entry (includes previous HM… }` | A single entry in the Write-Ahead Log. | `src/wal/encode.rs` |
| `WalEntryRef` | `{ pub entry: &'a WalEntry, }` | Lightweight view reference into a `WalEntry`. | `src/lsm/observer.rs` |
| `WalFlusherConfig` | `{ /// Maximum time to wait for additional messages after receiving the first, /// before issuing `sync_all()`. Set to 0 to disable (immediate flush). /// /// Typ… }` | Configuration for the WAL background flusher actor. Controls how aggressively the flusher coalesces concurrent writes into a single `sync_all()` call… | `src/wal/flusher.rs` |
| `WorkloadMetrics` | `{ read_count: AtomicU64, write_count: AtomicU64, last_seq_no: AtomicU64, }` | Operational workload metrics captured deterministically without wall-clock time (P28 compliance). All trigger decisions are strictly derived from tra… | `src/compaction/adaptive.rs` |
| `WorkloadMetricsSnapshot` | `{ pub read_count: u64, pub write_count: u64, pub last_seq_no: u64, }` | Point-in-time snapshot of operational workload metrics. | `src/compaction/adaptive.rs` |

### Öffentliche Datentypen — Enums (12)

| Enum | Varianten | Beschreibung | Datei |
|---|---|---|---|
| `CompactionStrategy` | `/// Standard Size-Tiered Compaction Strategy optimized for write throughput. WriteOptimizedSTCS, /// Aggressive compaction merging more tiers / full LSM to reduce read amplification. ReadOptimizedAgg…` | Compaction strategy variants selected by adaptive planner. | `src/compaction/adaptive.rs` |
| `DurabilityConfigError` | `#[error("DurabilityMode::{mode:?} is incompatible with feature '{feature}': {reason}")] IncompatibleCombination { mode: DurabilityMode, feature: &'static str, reason: &'static str, },` | — | `src/lsm/config.rs` |
| `DurabilityMode` | `Full, WalNoHmac, MemoryOnly,` | — | `src/lsm/config.rs` |
| `KvDeleteMode` | `TombstoneOnly, CryptoShred,` | Mode specifying KV segment deletion behavior. INVARIANTE `INV-KV-DELETE-1`: Löschbeweis auf Key-Value-Seite ist ausschließlich für `CryptoShred`-Segm… | `src/kv/delete_mode.rs` |
| `LegacyKeyStatus` | `/// A standard, secure integrity key (e.g. from `.wal_integrity_key` or `KeyManager`) is in use. Standard, /// The legacy obfuscated key is active and requires migration. LegacyActive,` | Status indicating whether a WAL segment or key configuration uses the obfuscated legacy key. | `src/wal/hmac.rs` |
| `ManifestEntry` | `/// A new SSTable file was created and fully fsynced. Add { path: PathBuf, max_tx: u64 }, /// An SSTable file was removed or superseded. Remove { path: PathBuf }, /// A transaction rollback operation…` | An entry in the SSTable manifest log. | `src/manifest/entry.rs` |
| `PressureLevel` | `Normal, Elevated, // Warnung: System unter Last Critical, // Backpressure aktiv: neue Inserts verlangsamen` | — | `src/system_pressure.rs` |
| `StorageHealth` | `Healthy, FlushFailing,` | Health status of the storage engine. | `src/lsm/engine.rs` |
| `WalOp` | `/// Inserts or updates a key-value pair. Put { tx_id: TxId, key: Vec<u8>, value: Vec<u8>, }, /// Deletes a key. Delete { tx_id: TxId, key: Vec<u8> }, /// Transaction outcome marker (committed / abort…` | — | `src/wal/encode.rs` |
| `WalParseError` | `#[error("WAL entry too short for CRC header")] HeaderTooShort, #[error("CRC mismatch: stored={stored:#010x}, computed={computed:#010x}")] CrcMismatch { stored: u32, computed: u32 }, #[error("WAL payl…` | Typed parsing errors encountered when decoding WAL entries from raw bytes. | `src/wal/encode.rs` |
| `WalVersion` | `V1, // Legacy: kein HMAC V2, // Current: HMAC ohne tx_id V3, // New: HMAC mit tx_id` | — | `src/wal/encode.rs` |
| `WriteOrigin` | `/// Regular user/application write transaction. UserWrite,` | Origin tag indicating where a committed batch originated from. | `src/lsm/observer.rs` |

### Typ-Aliase (3)

| Alias | Definition | Beschreibung |
|---|---|---|
| `BlockCacheShard` | `type BlockCacheShard= LruBlockCacheBackend` | — |
| `BlockCacheShard` | `type BlockCacheShard= QuickCacheBlockCacheBackend` | — |
| `WalSeq` | `type WalSeq= u64` | — |

### Konstanten (16)

| Konstante | Typ | Beschreibung |
|---|---|---|
| `BLOCKING_UTIL_CRITICAL` | `f32` | — |
| `BLOCKING_UTIL_ELEVATED` | `f32` | — |
| `BLOCK_CACHE_SHARDS` | `usize` | Sharded block cache: 64 independent shards to reduce contention. Shard selection uses ahash tuple hashing on (file_id, offset). |
| `CURRENT_MANIFEST_VERSION` | `u8` | — |
| `DEFAULT_MAX_OBSERVER_LATENCY` | `Duration` | Default maximum latency allowed for WAL observer callback execution (1 millisecond). |
| `MANIFEST_HEADER_MAGIC` | `&[u8` | — |
| `RESERVED_PREFIXES` | `&[&[u8]]` | List of known reserved system key prefixes. |
| `ROLLBACK_INLINE_THRESHOLD_ENTRIES` | `usize` | Maximum threshold for surviving entries during rollback below which entries are retained in memtable instead of creating a new SSTable (M-4 optimizat… |
| `SCHEDULER_QUEUE_CRITICAL` | `f32` | — |
| `SCHEDULER_QUEUE_ELEVATED` | `f32` | — |
| `SSTABLE_MAGIC_LEGACY` | `u32` | — |
| `SSTABLE_MAGIC_MFSX` | `u32` | Binary search for a key index inside a SSTable data block. Returns `Ok(Ok(index))` if the exact key is found, or `Ok(Err(index))` indicating the inse… |
| `WAL_QUEUE_CRITICAL_THRESHOLD` | `usize` | — |
| `WAL_QUEUE_ELEVATED_THRESHOLD` | `usize` | — |
| `WAL_V2_HEADER` | `[u8` | Magic header for V2 batch-encrypted WAL files (`b"MFW2"`). |
| `WAL_V3_HEADER` | `[u8` | Magic header for V3 WAL files (`b"MFW3"`). |

### Öffentliche Funktionen (232)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn new() -> Self` | Creates a new WorkloadMetrics counter instance initialized to zero. | `src/compaction/adaptive.rs` |
| `pub fn new(read_ratio_threshold: f64, min_sstables_per_tier: usize, size_ratio: f64) -> Self` | Creates a new CostBasedAdaptivePlanner with configurable read ratio threshold. | `src/compaction/adaptive.rs` |
| `pub fn record_read(&self)` | Records a read operation atomically. | `src/compaction/adaptive.rs` |
| `pub fn record_write(&self, seq_no: u64)` | Records a write operation and updates the latest sequence number atomically. | `src/compaction/adaptive.rs` |
| `pub fn snapshot(&self) -> WorkloadMetricsSnapshot` | Takes a deterministic snapshot of current metrics for compaction strategy evaluation. | `src/compaction/adaptive.rs` |
| `pub fn validate_tombstone_safety( candidates: &[Arc<SstableReader>], min_active_seqno: u64, is_full_compaction: bool, ) -> bool` | Hard invariant verification enforcing INV-COMPACTION-ADAPTIVE-1. Validates that no candidate tombstone with `seq >= min_active_seqno` can e… | `src/compaction/adaptive.rs` |
| `pub fn generate_sst_path(&self, data_path: &std::path::Path) -> Result<PathBuf>` | Generates a unique SSTable file path using a monotonic counter. | `src/compaction/engine.rs` |
| `pub async fn maybe_compact( &self, sstables: &RwLock<Vec<Arc<SstableReader>>>, data_path: &std::path::Path, ) -> Result<bool>` | — | `src/compaction/engine.rs` |
| `pub async fn maybe_compact_with_cancel( &self, sstables: &RwLock<Vec<Arc<SstableReader>>>, data_path: &std::path::Path, cancel_token: Option<&tokio_util::sync::CancellationToken>, ) -> Result<bool>` | Evaluates whether compaction should run and performs it with an optional cancellation token. | `src/compaction/engine.rs` |
| `pub async fn merge_sstables( &self, inputs: &[Arc<SstableReader>], output_path: &std::path::Path, min_snapshot_seq: u64, is_full_compaction: bool, ) -> Result<()>` | Performs a multi-way merge of input SSTables into a single output SSTable. During merge: - Duplicate keys: newest sequence number wins - To… | `src/compaction/engine.rs` |
| `pub async fn merge_sstables_with_cancel( &self, inputs: &[Arc<SstableReader>], output_path: &std::path::Path, min_snapshot_seq: u64, is_full_compaction: bool, cancel_token: Option<&tokio_util::sync::CancellationToken>, ) -> Result<()>` | Performs a multi-way merge with an optional cancellation token. | `src/compaction/engine.rs` |
| `pub fn new( config: CompactionConfig, snapshot_registry: Arc<SnapshotRegistry>, block_cache: Arc<BlockCache>, key_manager: Option<Arc<KeyManager>>, budget: Arc<contextra_core::ResourceTracker>, manifest: Option<Arc<crate::manifest::Manifest>>, ) -> Self` | Creates a new compaction engine. | `src/compaction/engine.rs` |
| `pub fn record_read_op(&self)` | Records a read operation for workload tracking. | `src/compaction/engine.rs` |
| `pub fn record_write_op(&self, seq_no: u64)` | Records a write operation for workload tracking. | `src/compaction/engine.rs` |
| `pub async fn run_loop( self: Arc<Self>, sstables: Arc<RwLock<Vec<Arc<SstableReader>>>>, data_path: PathBuf, shutdown: tokio_util::sync::CancellationToken, )` | Runs the background compaction loop. Periodically checks if compaction is needed and performs it. Designed to be spawned via `tokio::spawn`. | `src/compaction/engine.rs` |
| `pub fn select_compaction_candidates( &self, ssts: &[Arc<SstableReader>], ) -> Option<Vec<Arc<SstableReader>>>` | Selects SSTables to compact using Size-Tiered strategy. Groups by size class and returns the group that meets the threshold, prioritizing k… | `src/compaction/engine.rs` |
| `pub fn with_adaptive_planner(mut self, planner: Arc<dyn AdaptiveCompactionPlanner>) -> Self` | — | `src/compaction/engine.rs` |
| `pub fn with_clock(mut self, clock: Arc<dyn contextra_ports::Clock>) -> Self` | Attaches a custom clock port for deterministic execution timing (P28). | `src/compaction/engine.rs` |
| `pub fn with_merge_operator(mut self, merge_operator: Arc<dyn MergeOperator>) -> Self` | Attaches a custom merge operator for compaction value merging (§4.12). | `src/compaction/engine.rs` |
| `pub fn with_pressure_rx( mut self, rx: tokio::sync::watch::Receiver<crate::system_pressure::SystemPressure>, ) -> Self` | Attaches a system pressure watch receiver to enable pressure-aware compaction backpressure. | `src/compaction/engine.rs` |
| `pub fn retain_key_versions<T, F>(entries: Vec<T>, get_seq: F, floor_seq: u64) -> Vec<T> where F: Fn(&T) -> u64,` | Filters a sequence of entries for a SINGLE key (sorted by raw sequence DESCENDING) according to MVCC retention rules given a `floor` sequen… | `src/compaction/retention.rs` |
| `pub fn new() -> Self` | Creates a new, empty in-memory storage engine. | `src/engine/memory_only.rs` |
| `pub fn delete_mode(&self) -> KvDeleteMode` | Returns the current delete mode. | `src/kv/segment.rs` |
| `pub fn delete_segment(&self, group_id: u64) -> bool` | — | `src/kv/segment.rs` |
| `pub fn generate_deletion_proof(&self, group_id: u64) -> Result<bool, ContextraError>` | — | `src/kv/segment.rs` |
| `pub fn new( config: KvSegmentConfig, registry: Arc<KeyRegistry>, master_key: Option<Arc<KeyManager>>, ) -> Self` | Creates a new `KvSegmentManager` with specified config, key registry, and master key. | `src/kv/segment.rs` |
| `pub fn read_segment(&self, segment: &KvSegmentPayload) -> Result<Vec<u8>, ContextraError>` | — | `src/kv/segment.rs` |
| `pub fn write_segment( &self, group_id: u64, plaintext: &[u8], ) -> Result<KvSegmentPayload, ContextraError>` | — | `src/kv/segment.rs` |
| `pub fn cleanup_intent_locks_where<F>(&self, mut predicate: F) where F: FnMut(TxId) -> bool,` | Clears intent locks matching a predicate, gracefully recovering from poisoned locks. | `src/lsm/commit.rs` |
| `pub fn clear_intent_locks_above_tx(&self, target_tx: TxId)` | Removes all intent lock registrations associated with transactions strictly newer than target_tx. | `src/lsm/commit.rs` |
| `pub fn clear_intent_locks_for_tx(&self, tx_id: TxId)` | — | `src/lsm/commit.rs` |
| `pub fn validate_against_features( self, deletion_proof_active: bool, feature_ring: contextra_ports::license::FeatureRing, ) -> Result<(), DurabilityConfigError>` | — | `src/lsm/config.rs` |
| `pub fn block_cache_for_test(&self) -> Arc<BlockCache>` | — | `src/lsm/engine.rs` |
| `pub fn budget_for_test(&self) -> Arc<ResourceTracker>` | — | `src/lsm/engine.rs` |
| `pub fn budget_tracking_drift_bytes(&self) -> u64` | Returns the accumulated total memory budget tracking drift in bytes caused by unbudgeted memtable puts during commit when memory limit was… | `src/lsm/engine.rs` |
| `pub fn clear_circuit_breaker(&self, observer: &Arc<dyn WalObserver>)` | Resets the circuit breaker state for a specific observer. | `src/lsm/engine.rs` |
| `pub async fn close(&self) -> Result<()>` | Gracefully closes the storage engine, stopping background tasks and flushing active memtable to disk. | `src/lsm/engine.rs` |
| `pub fn deregister_observer(&self, observer: &Arc<dyn WalObserver>)` | Deregisters an observer matching the provided `Arc` reference via pointer equality. | `src/lsm/engine.rs` |
| `pub fn dropped_count_for(&self, observer: &Arc<dyn WalObserver>) -> u64` | Returns the number of dropped events for a specific registered observer. | `src/lsm/engine.rs` |
| `pub async fn get_at_seq_tracked( &self, tx_id: TxId, key: &[u8], snapshot_seq: u64, ) -> Result<Option<Bytes>>` | Performs a tracked read for transaction `tx_id` at an explicit `snapshot_seq`. Registers the read key in `TxBuffer` at `snapshot_seq` and r… | `src/lsm/engine.rs` |
| `pub async fn get_tracked(&self, tx_id: TxId, key: &[u8]) -> Result<Option<Bytes>>` | Performs a tracked read for transaction `tx_id`. Evaluates `snapshot_seq = next_seq_no - 1` ONCE (K2), registers the read key in `TxBuffer`… | `src/lsm/engine.rs` |
| `pub fn has_observers(&self) -> bool` | Returns true if at least one observer is registered. | `src/lsm/engine.rs` |
| `pub async fn has_pending_legacy_wal_migration(&self) -> Result<bool>` | — | `src/lsm/engine.rs` |
| `pub async fn has_pending_legacy_wal_migration_path(path: &std::path::Path) -> Result<bool>` | Checks if a database directory contains any legacy WAL files requiring explicit migration. Dies ist der Migrationsmechanismus vor der endgü… | `src/lsm/engine.rs` |
| `pub fn health(&self) -> StorageHealth` | Returns the current storage health status. | `src/lsm/engine.rs` |
| `pub fn last_applied_seq(&self) -> u64` | Returns the highest sequence number applied to the MemTable and published. | `src/lsm/engine.rs` |
| `pub fn manifest_for_test(&self) -> Arc<crate::manifest::Manifest>` | — | `src/lsm/engine.rs` |
| `pub fn max_observer_latency(&self) -> std::time::Duration` | Returns the current maximum latency threshold for observer callbacks. | `src/lsm/engine.rs` |
| `pub async fn maybe_compact_for_test(&self) -> Result<bool>` | — | `src/lsm/engine.rs` |
| `pub fn metrics_sink(&self) -> Arc<dyn contextra_ports::MetricsSink>` | Returns the currently configured [`MetricsSink`]. | `src/lsm/engine.rs` |
| `pub async fn migrate_legacy_wal_keys( config: &LsmConfig, clock: Arc<dyn contextra_ports::Clock>, ) -> Result<usize>` | — | `src/lsm/engine.rs` |
| `pub fn next_seq_no_for_test(&self) -> u64` | Returns the current `next_seq_no` value for testing verification. | `src/lsm/engine.rs` |
| `pub async fn open(config: LsmConfig) -> Result<Self>` | Opens or recovers an LSM storage instance with the given configuration. | `src/lsm/engine.rs` |
| `pub async fn open_with_merge_operator( config: LsmConfig, merge_operator: Arc<dyn MergeOperator>, ) -> Result<Self>` | — | `src/lsm/engine.rs` |
| `pub fn pressure_receiver( &self, ) -> tokio::sync::watch::Receiver<crate::system_pressure::SystemPressure>` | — | `src/lsm/engine.rs` |
| `pub fn register_observer(&self, observer: Arc<dyn WalObserver>)` | Registers a new WAL observer. | `src/lsm/engine.rs` |
| `pub fn request_flush(&self)` | Requests an asynchronous background flush without blocking the caller or holding commit locks. | `src/lsm/engine.rs` |
| `pub async fn restore_wal_file_handle_for_test(&self)` | — | `src/lsm/engine.rs` |
| `pub async fn scan_prefix_tracked( &self, tx_id: TxId, prefix: &[u8], ) -> Result<Vec<(Vec<u8>, Vec<u8>)>>` | Performs a tracked prefix scan for transaction `tx_id`. Evaluates `snapshot_seq = last_applied_seq` ONCE, scans all matching entries, and r… | `src/lsm/engine.rs` |
| `pub fn set_clock(&self, clock: Arc<dyn contextra_ports::Clock>)` | Sets the injected clock port for deterministic observer latency evaluation (P28). | `src/lsm/engine.rs` |
| `pub fn set_max_observer_latency(&self, latency: std::time::Duration)` | Sets the maximum latency threshold for observer callbacks. | `src/lsm/engine.rs` |
| `pub fn set_metrics_sink(&self, sink: Arc<dyn contextra_ports::MetricsSink>)` | Sets the [`MetricsSink`] for recording operational metrics. | `src/lsm/engine.rs` |
| `pub fn shutdown(&self)` | Signals the background compaction engine to stop. | `src/lsm/engine.rs` |
| `pub async fn simulate_wal_append_failure_for_test(&self)` | — | `src/lsm/engine.rs` |
| `pub async fn simulate_wal_append_failure_for_tx_for_test(&self, _tx_id: u64)` | — | `src/lsm/engine.rs` |
| `pub fn spawn_tracked<F>(&self, future: F) where F: std::future::Future<Output = ()> + Send + 'static,` | — | `src/lsm/engine.rs` |
| `pub fn sstables_for_test(&self) -> Arc<RwLock<Vec<Arc<SstableReader>>>>` | — | `src/lsm/engine.rs` |
| `pub async fn wait_shutdown(&self)` | Waits for all spawned tasks to shut down fully. | `src/lsm/engine.rs` |
| `pub fn with_metrics_sink(self, sink: Arc<dyn contextra_ports::MetricsSink>) -> Self` | Configures the [`MetricsSink`] fluently on `self` and returns `self`. | `src/lsm/engine.rs` |
| `pub async fn force_flush(&self) -> Result<()>` | — | `src/lsm/flush.rs` |
| `pub async fn maybe_compact(&self) -> Result<bool>` | Evaluates whether compaction should run and performs it if needed. Uses the persistent CompactionEngine instance to maintain a stable count… | `src/lsm/flush.rs` |
| `pub fn checksum(&self) -> &'a [u8` | Returns a reference to the entry's checksum. | `src/lsm/observer.rs` |
| `pub fn clear_circuit_breaker(&self, observer: &Arc<dyn WalObserver>)` | — | `src/lsm/observer.rs` |
| `pub fn deregister_observer(&self, observer: &Arc<dyn WalObserver>)` | Deregisters an observer matching the provided `Arc` reference via pointer equality. | `src/lsm/observer.rs` |
| `pub fn dropped_count(&self) -> u64` | Returns the total number of dropped events across all observers (including deregistered ones). | `src/lsm/observer.rs` |
| `pub fn dropped_count_for(&self, observer: &Arc<dyn WalObserver>) -> u64` | — | `src/lsm/observer.rs` |
| `pub fn is_any_circuit_breaker_open(&self) -> bool` | — | `src/lsm/observer.rs` |
| `pub fn is_circuit_breaker_open(&self, observer: &Arc<dyn WalObserver>) -> bool` | Returns whether the circuit breaker is currently open for a specific observer. | `src/lsm/observer.rs` |
| `pub fn is_empty(&self) -> bool` | Returns true if no observers are registered. | `src/lsm/observer.rs` |
| `pub fn max_observer_latency(&self) -> Duration` | Returns the current maximum latency threshold. | `src/lsm/observer.rs` |
| `pub fn new(entry: &'a WalEntry) -> Self` | Creates a new `WalEntryRef` from a `WalEntry`. | `src/lsm/observer.rs` |
| `pub fn new( inner: Arc<dyn WalObserver>, capacity: usize, ) -> (Self, tokio::task::JoinHandle<()>)` | — | `src/lsm/observer.rs` |
| `pub fn new() -> Self` | Creates a new, empty `ObserverRegistry`. | `src/lsm/observer.rs` |
| `pub fn notify(&self, entries: &[WalEntry], seq_no: u64, tx_id: TxId, origin: WriteOrigin)` | Synchronously notifies all registered observers of a committed batch. | `src/lsm/observer.rs` |
| `pub fn notify_with_context(&self, entries: &[WalEntry], seq_no: u64, ctx: CommitContext)` | Synchronously notifies all registered observers of a committed batch with explicit CommitContext. When invoked within a multi-threaded Toki… | `src/lsm/observer.rs` |
| `pub fn op(&self) -> &'a WalOp` | Returns a reference to the operation inside the entry. | `src/lsm/observer.rs` |
| `pub fn prev_hmac(&self) -> &'a [u8` | Returns a reference to the entry's previous HMAC checksum. | `src/lsm/observer.rs` |
| `pub fn register_observer(&self, observer: Arc<dyn WalObserver>)` | Registers a new WAL observer. | `src/lsm/observer.rs` |
| `pub fn seq_no(&self) -> u64` | Returns the sequence number of the entry. | `src/lsm/observer.rs` |
| `pub fn set_clock(&self, clock: Arc<dyn Clock>)` | Sets the injected clock port for deterministic execution timing (P28). | `src/lsm/observer.rs` |
| `pub fn set_max_observer_latency(&self, latency: Duration)` | Sets the maximum latency threshold for observer callbacks. | `src/lsm/observer.rs` |
| `pub fn tx_id(&self) -> TxId` | Returns the transaction ID associated with this entry. | `src/lsm/observer.rs` |
| `pub async fn new(config: LsmConfig) -> Result<Self>` | — | `src/lsm/recovery.rs` |
| `pub async fn new_with_merge_operator( config: LsmConfig, merge_operator: Option<Arc<dyn MergeOperator>>, ) -> Result<Self>` | — | `src/lsm/recovery.rs` |
| `pub async fn rollback_to_tx(&self, target_tx: TxId) -> Result<()>` | — | `src/lsm/recovery.rs` |
| `pub async fn point_lookup_metrics(&self, key: &[u8]) -> (usize, usize, usize, usize, bool)` | — | `src/lsm/scan.rs` |
| `pub fn derive_doc_id(key: &[u8]) -> DocId` | — | `src/lsm/validate.rs` |
| `pub fn derive_doc_id(key: &[u8]) -> DocId` | — | `src/lsm/validate.rs` |
| `pub fn validate_key(key: &[u8]) -> Result<()>` | — | `src/lsm/validate.rs` |
| `pub fn validate_value(value: &[u8]) -> Result<()>` | — | `src/lsm/validate.rs` |
| `pub async fn append(&self, entry: &ManifestEntry) -> Result<()>` | Appends a new entry to the manifest and performs flush + fsync. | `src/manifest/core.rs` |
| `pub async fn append_batch(&self, entries: &[ManifestEntry]) -> Result<()>` | Appends multiple entries to the manifest in a single atomic batch with flush + fsync. | `src/manifest/core.rs` |
| `pub fn extract_high_water_mark(entries: &[ManifestEntry]) -> Option<[u8` | Extracts the most recent WAL high-water-mark HMAC from a list of manifest entries. | `src/manifest/core.rs` |
| `pub async fn load(path: &Path) -> Result<Vec<ManifestEntry>>` | Loads all valid manifest entries from `path`. If the file does not exist, returns an empty vector. Tail-truncation (an incomplete frame at… | `src/manifest/core.rs` |
| `pub async fn open(path: impl AsRef<Path>) -> Result<Self>` | Opens or creates an append-only Manifest file at `path`. | `src/manifest/core.rs` |
| `pub fn path(&self) -> &Path` | — | `src/manifest/core.rs` |
| `pub fn reconstruct_dead_sstables( entries: &[ManifestEntry], ) -> std::collections::HashSet<PathBuf>` | Reconstructs the set of SSTable file names (or path components) that are proven dead (explicitly removed via `Remove` or `Replace.removed`)… | `src/manifest/core.rs` |
| `pub fn reconstruct_valid_sstables(entries: &[ManifestEntry]) -> Vec<(PathBuf, u64)>` | Reconstructs the ordered list of currently valid SSTable file names (or path components) and their rank (shadowing order: smaller rank = ol… | `src/manifest/core.rs` |
| `pub fn from_bytes(data: &[u8]) -> Result<Self>` | Deserializes a manifest entry from a slice containing `[crc32: u32 LE] [payload...]`. | `src/manifest/entry.rs` |
| `pub fn to_bytes(&self) -> Result<Vec<u8>>` | Serializes the entry into binary format with a 4-byte total size prefix and a 4-byte CRC32. | `src/manifest/entry.rs` |
| `pub async fn maybe_rollover( &self, live_entries: &[ManifestEntry], threshold_bytes: u64, ) -> Result<bool>` | Evaluates whether the MANIFEST file size exceeds `threshold_bytes` and performs an atomic rollover if needed. Rollover process: (a) Write c… | `src/manifest/rollover.rs` |
| `pub async fn rollover(&self, live_entries: &[ManifestEntry]) -> Result<()>` | Evaluates whether the MANIFEST file size exceeds `threshold_bytes` and performs an atomic rollover if needed. Rollover process: (a) Write c… | `src/manifest/rollover.rs` |
| `pub fn get(&self, key: &[u8]) -> Option<(Bytes, u64)>` | Retrieves the latest value and sequence number by key. | `src/memtable.rs` |
| `pub fn get_at_seq(&self, key: &[u8], seq_no: u64, max_tx: u64) -> Option<(Bytes, u64, u64)>` | Retrieves a value, sequence number, and transaction ID by key at or below a specific sequence number and bounded by maximum transaction ID… | `src/memtable.rs` |
| `pub fn is_empty(&self) -> bool` | Returns true if the memtable is empty. WARNING: This reads all shards sequentially. The result is not an atomic snapshot, but this method i… | `src/memtable.rs` |
| `pub fn is_valid_range_bounds(start: Bound<&[u8]>, end: Bound<&[u8]>) -> bool` | Validates whether start and end key range bounds define a non-empty, valid interval (`start <= end`). | `src/memtable.rs` |
| `pub fn iter(&self) -> Vec<(Bytes, Bytes, u64, u64)>` | Iterates over all entries (all versions) in sorted key order. Returns (Key, Value, SeqNo, TxId). | `src/memtable.rs` |
| `pub fn iter_latest(&self) -> Vec<(Bytes, Bytes, u64, u64)>` | Iterates over only the latest version of each key in sorted key order. Returns (Key, Value, SeqNo, TxId). | `src/memtable.rs` |
| `pub fn new() -> Self` | Creates a new empty MemTable with `SHARD_COUNT` independent shards. | `src/memtable.rs` |
| `pub fn put(&self, key: Bytes, value: Bytes, seq_no: u64, tx_id: u64)` | Inserts a key-value pair with a sequence number and transaction ID. | `src/memtable.rs` |
| `pub fn rollback(&self, tx_id: u64)` | Removes all entries associated with the specified transaction ID from the MemTable. | `src/memtable.rs` |
| `pub fn scan_prefix_into( &self, prefix: &[u8], seq_no: u64, max_tx: TxId, target: &mut BTreeMap<Bytes, (Bytes, u64)>, )` | Scans entries matching a key prefix into `target` according to MVCC visibility rules. Iterates over each shard's range starting at `prefix`… | `src/memtable.rs` |
| `pub fn scan_prefix_into_matching<F>( &self, prefix: &[u8], seq_no: u64, max_tx: TxId, target: &mut BTreeMap<Bytes, (Bytes, u64)>, predicate: F, ) where F: Fn(&[u8], u64, u64) -> bool,` | Scans entries matching a key prefix into `target` with an additional matching predicate. | `src/memtable.rs` |
| `pub fn scan_range_into( &self, start: Bound<&[u8]>, end: Bound<&[u8]>, seq_no: u64, max_tx: TxId, target: &mut BTreeMap<Bytes, (Bytes, u64)>, )` | — | `src/memtable.rs` |
| `pub fn scan_range_into_matching<F>( &self, start: Bound<&[u8]>, end: Bound<&[u8]>, seq_no: u64, max_tx: TxId, target: &mut BTreeMap<Bytes, (Bytes, u64)>, predicate: F, ) where F: Fn(&[u8], u64, u64) -> bool,` | Scans entries matching a key range into `target` with an additional matching predicate. | `src/memtable.rs` |
| `pub fn shard_entry_counts(&self) -> [usize` | — | `src/memtable.rs` |
| `pub fn shard_for(key: &[u8]) -> usize` | Deterministic shard selector mapping keys to independent shards. Constructs a 16-bit big-endian value from the first two bytes of the key t… | `src/memtable.rs` |
| `pub fn size(&self) -> usize` | Returns the approximate size in bytes. | `src/memtable.rs` |
| `pub fn tx_range(&self) -> (u64, u64)` | — | `src/memtable.rs` |
| `pub fn contains(&self, file_id: u64, offset: u64) -> bool` | — | `src/sstable/block_cache.rs` |
| `pub fn create_block_cache(capacity_mb: usize) -> Arc<BlockCache>` | Creates a new block cache instance with default shard count. Capacity is in MB (assuming 4KB blocks). | `src/sstable/block_cache.rs` |
| `pub fn create_block_cache_with_shards(capacity_mb: usize, num_shards: usize) -> Arc<BlockCache>` | Creates a new block cache instance with configurable shard count. Capacity is in MB. | `src/sstable/block_cache.rs` |
| `pub fn get(&self, file_id: u64, offset: u64) -> Option<Bytes>` | — | `src/sstable/block_cache.rs` |
| `pub fn insert(&self, file_id: u64, offset: u64, data: Bytes)` | — | `src/sstable/block_cache.rs` |
| `pub fn is_empty(&self) -> bool` | — | `src/sstable/block_cache.rs` |
| `pub fn len(&self) -> usize` | — | `src/sstable/block_cache.rs` |
| `pub fn new(capacity_per_shard: usize) -> Self` | — | `src/sstable/block_cache.rs` |
| `pub fn new_with_shards(capacity_per_shard: usize, num_shards: usize) -> Self` | — | `src/sstable/block_cache.rs` |
| `pub fn shard_idx(&self, file_id: u64, offset: u64) -> usize` | — | `src/sstable/block_cache.rs` |
| `pub fn binary_search_entry_in_block( block_data: &[u8], offsets_start: usize, num_offsets: usize, key: &[u8], is_v3: bool, ) -> Result<Option<(usize, usize)>>` | — | `src/sstable/block_search.rs` |
| `pub fn binary_search_first_index_in_block( block_data: &[u8], offsets_start: usize, num_offsets: usize, key: &[u8], is_v3: bool, ) -> Result<Option<usize>>` | — | `src/sstable/block_search.rs` |
| `pub fn binary_search_index_in_block( block_data: &[u8], offsets_start: usize, num_offsets: usize, key: &[u8], is_v3: bool, ) -> Result<std::result::Result<usize, usize>>` | — | `src/sstable/block_search.rs` |
| `pub fn block_binary_search( block_data: &[u8], offsets_start: usize, num_offsets: usize, key: &[u8], is_v3: bool, ) -> Result<Option<usize>>` | — | `src/sstable/block_search.rs` |
| `pub fn get_entry_at_index( block_data: &[u8], offsets_start: usize, idx: usize, is_v3: bool, ) -> Result<(usize, usize)>` | Entry offset accessor supporting both 16-bit (legacy v0..v2) and 32-bit (v3+) block formats. | `src/sstable/block_search.rs` |
| `pub fn get_entry_key_at_index( block_data: &[u8], offsets_start: usize, idx: usize, is_v3: bool, ) -> Result<&[u8]>` | — | `src/sstable/block_search.rs` |
| `pub fn from_bytes(data: &[u8]) -> Result<Self>` | — | `src/sstable/bloom.rs` |
| `pub fn hash_pair(key: &[u8]) -> (u64, u64)` | Erzeugt zwei unabhängige 64-bit Hashes aus dem Blake3-Digest. h1 = erste 8 Bytes, h2 = Bytes 8-15 (oder fallback zu h1 ^ const) | `src/sstable/bloom.rs` |
| `pub fn insert(&mut self, key: &[u8])` | Inserts a key into the filter. | `src/sstable/bloom.rs` |
| `pub fn may_contain(&self, key: &[u8]) -> bool` | Checks if a key might be in the filter. | `src/sstable/bloom.rs` |
| `pub fn new(expected_elements: usize, fpr: f64) -> Self` | Creates a new Bloom filter for the expected number of elements and target false positive rate (fpr). ## FPR Trade-off - `fpr = 0.01` (1%) u… | `src/sstable/bloom.rs` |
| `pub fn to_bytes(&self) -> Vec<u8>` | — | `src/sstable/bloom.rs` |
| `pub fn add(&mut self, key: &[u8], value: &[u8], seq_no: u64, tx_id: u64) -> bool` | — | `src/sstable/builder.rs` |
| `pub async fn add(&mut self, key: &[u8], value: &[u8], seq_no: u64, tx_id: u64) -> Result<()>` | Adds a key-value pair to the SSTable being built. | `src/sstable/builder.rs` |
| `pub fn build(mut self) -> Bytes` | Finalizes the block and returns the bytes. | `src/sstable/builder.rs` |
| `pub async fn create(path: impl AsRef<Path>) -> Result<Self>` | Creates a new SstableBuilder that writes to the given file path. | `src/sstable/builder.rs` |
| `pub async fn create_with_key_manager( path: impl AsRef<Path>, key_manager: Option<Arc<KeyManager>>, ) -> Result<Self>` | — | `src/sstable/builder.rs` |
| `pub async fn create_with_version(path: impl AsRef<Path>, version: u16) -> Result<Self>` | Creates a new SstableBuilder with an explicit format version. | `src/sstable/builder.rs` |
| `pub fn current_size(&self) -> usize` | — | `src/sstable/builder.rs` |
| `pub fn current_size(&self) -> u64` | Returns the estimated current total SSTable size written so far. | `src/sstable/builder.rs` |
| `pub async fn finish(mut self) -> Result<SstableMetadata>` | Finalizes the SSTable and returns metadata. | `src/sstable/builder.rs` |
| `pub fn is_empty(&self) -> bool` | — | `src/sstable/builder.rs` |
| `pub fn new(block_size: usize) -> Self` | — | `src/sstable/builder.rs` |
| `pub fn new_with_version(block_size: usize, format_version: u16) -> Self` | — | `src/sstable/builder.rs` |
| `pub fn set_format_version(&mut self, version: u16)` | Explicitly configures the SSTable format version to write (default: 4). | `src/sstable/builder.rs` |
| `pub fn file_path(&self) -> &Path` | — | `src/sstable/reader.rs` |
| `pub fn first_key(&self) -> &Bytes` | — | `src/sstable/reader.rs` |
| `pub async fn get(&self, key: &[u8]) -> Result<Option<(Bytes, u64, u64)>>` | Retrieves a value from the SSTable by key. | `src/sstable/reader.rs` |
| `pub async fn get_at( &self, key: &[u8], max_seq: u64, max_tx: u64, ) -> Result<Option<(Bytes, u64, u64)>>` | Retrieves a value, sequence number, and transaction ID from the SSTable by key bounded by maximum sequence number (`max_seq`) and maximum t… | `src/sstable/reader.rs` |
| `pub async fn iter(&self) -> Result<Vec<(Bytes, Bytes, u64)>>` | Iterates over all entries in sorted key order (allocates memory for all entries). | `src/sstable/reader.rs` |
| `pub fn last_key(&self) -> &Bytes` | — | `src/sstable/reader.rs` |
| `pub async fn lookup_metrics(&self, key: &[u8]) -> (bool, bool, bool, bool)` | Metrics result for point lookup instrumentation. | `src/sstable/reader.rs` |
| `pub fn max_tx_id(&self) -> u64` | — | `src/sstable/reader.rs` |
| `pub fn metadata(&self) -> &SstableMetadata` | — | `src/sstable/reader.rs` |
| `pub fn min_tx_id(&self) -> u64` | — | `src/sstable/reader.rs` |
| `pub async fn open(path: impl AsRef<Path>, block_cache: Arc<BlockCache>) -> Result<Self>` | Opens an existing SSTable file for reading. | `src/sstable/reader.rs` |
| `pub async fn open_with_key_manager( path: impl AsRef<Path>, block_cache: Arc<BlockCache>, key_manager: Option<Arc<KeyManager>>, ) -> Result<Self>` | — | `src/sstable/reader.rs` |
| `pub async fn stream(self: &Arc<Self>) -> Result<SstableStream>` | — | `src/sstable/reader.rs` |
| `pub async fn scan_prefix(&self, prefix: &[u8]) -> Result<Vec<(Bytes, Bytes, u64, u64)>>` | Scans the SSTable for keys starting with the given prefix. | `src/sstable/reader_ext.rs` |
| `pub async fn scan_range( &self, start: std::ops::Bound<&[u8]>, end: std::ops::Bound<&[u8]>, ) -> Result<Vec<(Bytes, Bytes, u64, u64)>>` | Scans all entries within a key range. | `src/sstable/reader_ext.rs` |
| `pub async fn next(&mut self) -> Result<Option<(Bytes, Bytes, u64, u64)>>` | — | `src/sstable/stream.rs` |
| `pub async fn next_entry(&mut self) -> Result<Option<(Bytes, Bytes, u64, u64)>>` | Optimized for compaction: avoids copying if possible. | `src/sstable/stream.rs` |
| `pub fn compute_pressure( &self, wal_depth: usize, scheduler_queue_depth: f32, blocking_util: f32, embedding_queue: usize, max_permits: usize, ) -> SystemPressure` | — | `src/system_pressure.rs` |
| `pub fn new(sampling_interval: Duration) -> Self` | — | `src/system_pressure.rs` |
| `pub async fn run( self, cancellation: CancellationToken, wal_queue_depth_fn: impl Fn() -> usize + Send + 'static, embedding_permits_fn: impl Fn() -> usize + Send + 'static, max_embedding_permits: usize, )` | — | `src/system_pressure.rs` |
| `pub async fn run_with_metrics_sink( self, cancellation: CancellationToken, wal_queue_depth_fn: impl Fn() -> usize + Send + 'static, embedding_permits_fn: impl Fn() -> usize + Send + 'static, max_embedding_permits: usize, metrics_sink: Option<Arc<parking_lot::…` | — | `src/system_pressure.rs` |
| `pub fn collection_prefix(&self, collection: &CollectionId) -> Vec<u8>` | — | `src/tenant_codec.rs` |
| `pub fn decode_tenant_id(key: &[u8]) -> Option<TenantId>` | — | `src/tenant_codec.rs` |
| `pub fn encode_chunk_key(&self, collection: &CollectionId, doc_id: DocId) -> Vec<u8>` | — | `src/tenant_codec.rs` |
| `pub fn encode_graph_key(&self, collection: &CollectionId, entity_id: u64) -> Vec<u8>` | — | `src/tenant_codec.rs` |
| `pub fn new(tenant_id: TenantId) -> Self` | Creates a new `TenantKeyCodec` for the given `TenantId`. | `src/tenant_codec.rs` |
| `pub fn new(inner: S, tenant_id: TenantId) -> Self` | Creates a new `TenantScopedStorage` wrapping an underlying `StorageEngine` or shared pointer. | `src/tenant_codec.rs` |
| `pub fn scan_prefix(&self) -> &[u8]` | Globales Scan-Präfix für diesen Tenant (alle Collections): `t:{tenant}:` Cached — keine Allokation im Hot-Path. | `src/tenant_codec.rs` |
| `pub fn tenant_id(&self) -> TenantId` | Returns the immutable `TenantId` bound to this storage wrapper. | `src/tenant_codec.rs` |
| `pub fn compute_checksum( op: &WalOp, seq_no: u64, integrity_key: &[u8], prev_hmac: [u8; 32], ) -> Result<[u8` | Computes default checksum (delegates to V3). | `src/wal/encode.rs` |
| `pub fn compute_checksum_v2( op: &WalOp, seq_no: u64, integrity_key: &[u8], prev_hmac: [u8; 32], ) -> Result<[u8` | Legacy V2 checksum calculation (without tx_id and length-prefixes in HMAC). | `src/wal/encode.rs` |
| `pub fn compute_checksum_v3( op: &WalOp, seq_no: u64, integrity_key: &[u8], prev_hmac: [u8; 32], ) -> Result<[u8` | Computes V3 checksum (includes tx_id and length prefixes for key/value). | `src/wal/encode.rs` |
| `pub fn entries(&self) -> &[WalEntry]` | Returns a slice of the inner WAL entries. | `src/wal/encode.rs` |
| `pub fn extend(&mut self, other: PreparedBatch)` | Extends this prepared batch with entries from another prepared batch. | `src/wal/encode.rs` |
| `pub fn from_bytes(data: &[u8]) -> Result<Self>` | Deserializes a WAL entry from bytes, verifying CRC32. | `src/wal/encode.rs` |
| `pub fn from_bytes_classified(data: &[u8]) -> std::result::Result<Self, WalParseError>` | Deserializes a WAL entry from bytes, returning a classified `WalParseError` on failure. | `src/wal/encode.rs` |
| `pub fn into_inner(self) -> Vec<WalEntry>` | Consumes the `PreparedBatch` and returns the inner vector of entries. | `src/wal/encode.rs` |
| `pub fn is_empty(&self) -> bool` | Returns `true` if the batch contains no entries. | `src/wal/encode.rs` |
| `pub fn len(&self) -> usize` | Returns the number of entries in the batch. | `src/wal/encode.rs` |
| `pub fn to_bytes(&self) -> Result<Vec<u8>>` | Serializes the entry to bytes. | `src/wal/encode.rs` |
| `pub fn try_new( op: WalOp, seq_no: u64, integrity_key: &[u8], prev_hmac: [u8; 32], ) -> Result<Self>` | Creates a new WAL entry with HMAC-SHA256 checksum and chaining. | `src/wal/encode.rs` |
| `pub fn tx_id(&self) -> TxId` | — | `src/wal/encode.rs` |
| `pub fn tx_id(&self) -> TxId` | — | `src/wal/encode.rs` |
| `pub fn integrity_key_for_test(&self) -> Result<[u8` | Exposes the HMAC integrity key for testing. | `src/wal/hmac.rs` |
| `pub fn legacy_integrity_key_for_test() -> [u8` | Returns the static legacy HMAC integrity key for testing. | `src/wal/hmac.rs` |
| `pub fn legacy_key_status(&self) -> LegacyKeyStatus` | Returns the legacy integrity key status for this WAL segment. | `src/wal/hmac.rs` |
| `pub fn migrate_legacy_key(legacy: &[u8], new_key_material: &[u8]) -> Result<[u8` | Migrates a legacy integrity key to a new 32-byte key material state. # Chain Compatibility When migrating a legacy WAL segment to a new int… | `src/wal/hmac.rs` |
| `pub fn migrate_legacy_key(_legacy: &[u8], _new_key_material: &[u8]) -> Result<[u8` | — | `src/wal/hmac.rs` |
| `pub async fn prepare_batch(&self, ops: Vec<(WalOp, u64)>) -> Result<(PreparedBatch, [u8` | Prepares a batch of WAL operations with sequential sequence numbers and HMAC hash-chaining. | `src/wal/hmac.rs` |
| `pub async fn append_batch(&self, batch: PreparedBatch) -> Result<()>` | — | `src/wal/io.rs` |
| `pub async fn append_batch_locked( &self, batch: PreparedBatch, guard: &tokio::sync::MutexGuard<'_, ()>, ) -> Result<()>` | — | `src/wal/io.rs` |
| `pub fn arm_truncate_failure_for_test(&self)` | Aktiviert eine einmalige simulierte Truncate-I/O-Failure für Tests. Der Flusher gibt beim nächsten `WalCommand::Truncate` einen Fehler zurü… | `src/wal/io.rs` |
| `pub async fn enqueue_append_batch_locked( &self, batch: PreparedBatch, _guard: &tokio::sync::MutexGuard<'_, ()>, ) -> Result<tokio::sync::oneshot::Receiver<Result<()>>>` | — | `src/wal/io.rs` |
| `pub async fn find_tx_offset(&self, target_tx_id: TxId) -> Result<(u64, [u8` | — | `src/wal/io.rs` |
| `pub async fn last_hmac_snapshot(&self) -> [u8` | Returns a snapshot of the last HMAC written to the log. | `src/wal/io.rs` |
| `pub async fn rotate_and_seal(&self) -> Result<PathBuf>` | — | `src/wal/io.rs` |
| `pub async fn scan_entries_with_callback<F>( &self, file_size: u64, mut callback: F, ) -> Result<WalVersion> where F: FnMut(u64, WalEntry, u64) -> bool,` | Scans the WAL entry by entry, executing full HMAC chain validation, CRC checks, and key manager decryption. Invokes `callback(seq_no, entry… | `src/wal/io.rs` |
| `pub async fn truncate(&self, offset: u64, new_last_hmac: [u8; 32]) -> Result<()>` | — | `src/wal/io.rs` |
| `pub async fn try_append_batch(&self, batch: PreparedBatch) -> Result<()>` | Non-blocking attempt to append a prepared batch. Returns `Err(ContextraError::Storage("WAL queue full (backpressure)"))` if the flusher cha… | `src/wal/io.rs` |
| `pub async fn try_append_batch_locked( &self, batch: PreparedBatch, guard: &tokio::sync::MutexGuard<'_, ()>, ) -> Result<()>` | — | `src/wal/io.rs` |
| `pub async fn try_enqueue_append_batch_locked( &self, batch: PreparedBatch, _guard: &tokio::sync::MutexGuard<'_, ()>, ) -> Result<tokio::sync::oneshot::Receiver<Result<()>>>` | — | `src/wal/io.rs` |
| `pub async fn replay(&self) -> Result<Vec<(u64, WalEntry, u64)>>` | Replays the WAL using memory mapping (`memmap2`) for zero-copy entry decoding. Falls back to stream replay if mmap or parsing fails. | `src/wal/replay.rs` |
| `pub async fn replay_from(&self, since_seq_no: u64) -> Result<Vec<WalEntry>>` | Replays the WAL and returns all entries with seq_no > since_seq_no. | `src/wal/replay.rs` |
| `pub async fn replay_mmap(&self) -> Result<(Vec<(u64, WalEntry, u64)>, WalVersion)>` | Replays the WAL using zero-copy memory mapping (`mmap`). Returns all valid entries with sequence numbers, entries, end offsets, and detecte… | `src/wal/replay.rs` |
| `pub async fn replay_mmap_with_sink<S: ReplayProgressSink>( &self, sink: &S, ) -> Result<(Vec<(u64, WalEntry, u64)>, WalVersion)>` | Replays the WAL using zero-copy memory mapping (`mmap`) with progress callback. Falls back to stream replay if mmap or parsing fails. | `src/wal/replay.rs` |
| `pub async fn replay_stream(&self) -> Result<Vec<(u64, WalEntry, u64)>>` | Replays the WAL using the stream reader (`BufReader`). | `src/wal/replay.rs` |
| `pub async fn replay_with_sink<S: ReplayProgressSink>( &self, sink: &S, ) -> Result<Vec<(u64, WalEntry, u64)>>` | Replays the WAL while notifying the provided `ReplayProgressSink` of progress. | `src/wal/replay.rs` |
| `pub async fn scan_entries_mmap<F>(&self, file_size: u64, mut callback: F) -> Result<WalVersion> where F: FnMut(u64, WalEntry, u64) -> bool,` | Scans the WAL using read-only memory mapping (`memmap2`) for zero-copy entry parsing. If mmap mapping or slice scanning encounters any erro… | `src/wal/replay.rs` |


---

## `contextra-kvcache`

| Attribut | Wert |
|---|---|
| **Ring** | 1 — Ring 1 — Persistence & Cache |
| **Status** | Fertig |
| **Zweck** | Verschlüsselter Prefix-Radix-Baum und KV-Cache-Segment-Verwaltung. |
| **Unsafe-Code** | Nein (`#![forbid(unsafe_code)]`) |
| **Interne Abhängigkeiten** | `contextra-crypto`, `contextra-ports`, `contextra-types` |
| **Externe Abhängigkeiten (Auswahl)** | `ahash`, `bincode`, `blake3`, `bytes`, `lru`, `parking_lot`, `serde`, `serde_json`, `tracing`, `zeroize` |

### Modul- und Re-Export-Oberfläche (lib.rs)

```rust
pub mod attention_score;
pub mod eviction_worker;
pub mod prefix_store;
pub mod quantize_kivi;
pub mod radix;
pub mod scratchpad_invalidation;
pub mod segment;
pub mod store;
pub use attention_score::{ rank_for_eviction, rank_for_eviction_weighted, AttentionScoreSource, NullAttentionScoreSource, };
pub use eviction_worker::{emergency_wipe, EvictionWorker};
pub use prefix_store::TenantPrefixKvStore;
pub use quantize_kivi::{ compress_bytes, decompress_bytes, kivi_dequantize, kivi_quantize, pack_kivi_block, unpack_kivi_block, KiviBlockMeta, KiviQuantizeConfig, KiviQuantizedBlock, KvTensorView, };
pub use radix::{ ContentAddressedKvStore, KvLookupResult, KvSegmentRef, SemanticCacheConfig, SemanticEmbedder, };
pub use radix::{KvBlockGuard, KvReusePolicy, PrefixMatch, PrefixRadixTree};
pub use scratchpad_invalidation::{ PrefixStoreScratchpadInvalidator, ScratchpadCacheScope, ScratchpadInvalidator, };
pub use segment::{ KvSegment, KvSegmentContent, ShreddableSegmentKey, Tier2EncryptedSegment, CURRENT_KV_KEY_DERIVATION_VERSION, };
pub use store::{SpillHandler, TenantIsolatedKvStore};
```

### Feature-Flags

| Flag | Aktiviert |
|---|---|
| `default` | `[]` |
| `content-addressed-kv-cache` | `[]` |
| `kv-encryption` | `["contextra-crypto/kv-encryption"]` |
| `kvcache-kivi-quant` | `[]` |
| `kivi-quantization` | `[]` |

### Schnittstellen — Public Traits (2)

**`AttentionScoreSource`** — *Dyn-compatible source for segment attention importance scores. Implemented by upstream inference backends or test mocks to supply attention weights without introducing dependencies on specific ML tensor frameworks.* (`src/attention_score.rs`)

```rust
pub trait AttentionScoreSource : Send + Sync {
/// Returns the importance score for a given `segment_id`.
    ///
    /// Higher values indicate higher retention importance (less evictable).
    /// Lower values indicate lower retention importance (more evictable).
    /// Returns `None` if no attention score is available for the segment.
    fn importance_score(&self, segment_id: u64) -> Option<f32>;

    /// Returns the importance score for a given tenant and `segment_id`.
    ///
    /// Default implementation delegates to [`importance_score`](Self::importance_score).
    fn importance_score_for_tenant(&self, _tenant_id: TenantId, segment_id: u64) -> Option<f32> {
        self.importance_score(segment_id)
    }

    /// Registers a segment mapping from `(tenant_id, segment_id)` to `request_id`.
    fn register_segment(&self, _tenant_id: TenantId, _segment_id: u64, _request_id: RequestId) {}

    /// Unregisters a segment mapping for `(tenant_id, segment_id)` when evicted or released.
    fn unregister_segment(&self, _tenant_id: TenantId, _segment_id: u64) {}
}
```

**`ScratchpadInvalidator`** — *Schnittstelle zur gezielten Invalidation von Scratchpad-Cache-Scopes.* (`src/scratchpad_invalidation.rs`)

```rust
pub trait ScratchpadInvalidator {
/// Invalidiert alle Gruppen/Blöcke im Radix-Baum, die zum angegebenen `ScratchpadCacheScope` gehören.
    ///
    /// # Fehler
    /// Gibt einen `Result<usize>` zurück, wobei `usize` die Anzahl invalidierter Segmente/Blöcke darstellt.
    fn invalidate_scope(&self, scope: &ScratchpadCacheScope) -> Result<usize>;
}
```

### Öffentliche Datentypen — Structs (17)

| Struct | Felder / Form | Beschreibung | Datei |
|---|---|---|---|
| `EncryptedSegmentPayload` | `{ /// Encrypted KV layer container from contextra-crypto. Zeroized on drop. pub layer: EncryptedKvLayer, #[zeroize(skip)] pub key_derivation_version: u8, }` | Encrypted layer representation stored inside a KvSegment when encryption is active. | `src/segment.rs` |
| `EvictionWorker` | `{ attention_source: Arc<RwLock<Arc<dyn AttentionScoreSource>>>, sender: parking_lot::Mutex<mpsc::Sender<EvictionCommand>>, handle: parking_lot::Mutex<Option<std:… }` | Regelmäßiger Eviction-Pfad (VRAM > 80%-Trigger). NICHT im Async-Executor, da regelmäßige Zeroize-Operationen den Tokio-Scheduler blockieren würden. | `src/eviction_worker.rs` |
| `KiviBlockMeta` | `{ /// Number of bits allocated per scalar value (must be 2 for KIVI). pub bits_per_value: u8, /// Scaling multiplier for linear quantization. pub scale: f32, ///… }` | Metadata required to dequantize a 2-bit KIVI block. | `src/quantize_kivi.rs` |
| `KiviQuantizeConfig` | `{ /// Group size for key channel quantization (default: 16). pub key_group_size: usize, /// Whether value quantization is enabled (default: true). pub quantize_v… }` | Configuration for KIVI quantization parameters. | `src/quantize_kivi.rs` |
| `KiviQuantizedBlock` | `{ /// Quantization metadata. pub meta: KiviBlockMeta, /// Packed 2-bit scalar values (4 values packed per byte, LSB-first). pub packed: Vec<u8>, }` | A 2-bit quantized byte block with accompanying metadata. | `src/quantize_kivi.rs` |
| `KvBlockGuard` | `{ pub block_id: u64, pub tenant_id: TenantId, ref_counter: Arc<AtomicUsize>, worker: Option<Arc<EvictionWorker>>, }` | RAII-Guard für ausgeliehene/gematchte KV-Blöcke. Dekrementiert bei `Drop` atomar die Referenzzählung des Blocks und benachrichtigt ggf. den `Eviction… | `src/radix.rs` |
| `KvSegment` | `{ #[zeroize(skip)] // Metadaten, keine sensiblen Tensor-Daten pub tenant_id: TenantId, #[zeroize(skip)] pub segment_id: u64, /// Versionsnummer der HKDF-Schlüsse… }` | Ein KV-Cache-Segment. P9-Pflicht: Zeroize-on-Drop, nie unverschlüsselt auf persistentem/auslagerbarem Speicher. | `src/segment.rs` |
| `KvTensorView` | `{ /// Key tensor floats in [num_tokens, num_channels] row-major order. pub keys: Vec<f32>, /// Value tensor floats in [num_tokens, num_channels] row-major order.… }` | A view over unquantized raw key and value tensors. | `src/quantize_kivi.rs` |
| `NullAttentionScoreSource` | `(tuple/unit)` | Default implementation that provides no attention scores (pure LRU behavior). | `src/attention_score.rs` |
| `PrefixMatch` | `{ /// Übereinstimmende Token-Sequenz. pub matched_tokens: Vec<u32>, /// Anzahl der übereinstimmenden Tokens. pub matched_len: usize, /// Assoziierte KV-Block-ID.… }` | Ergebnis eines Präfix-Matches im Radix-Baum. | `src/radix.rs` |
| `PrefixRadixTree` | `{ tenant_id: TenantId, children: Vec<RadixNode>, total_entries: usize, }` | Komprimierter Prefix-Radix-Baum für Token-Sequenzen. | `src/radix.rs` |
| `PrefixStoreScratchpadInvalidator` | `{ /// Referenz auf den unterliegenden `TenantPrefixKvStore`. pub store: &'a TenantPrefixKvStore, /// Liste von `PrefixKey`s, die im Store auf Scratchpad-Präfixe… }` | Implementierung von `ScratchpadInvalidator` für `TenantPrefixKvStore`. | `src/scratchpad_invalidation.rs` |
| `ScratchpadCacheScope` | `{ /// Eindeutige Mandanten-ID (TenantId) des ausführenden Agenten. pub tenant_id: TenantId, /// Token-Präfix, das den editierbaren Scratchpad-Bereich im Radix-Ba… }` | Definiert den Bereich (Scope) eines Agenten-Scratchpads für die Cache-Invalidierung. | `src/scratchpad_invalidation.rs` |
| `ShreddableSegmentKey` | `{ key_manager: Arc<RwLock<Option<CryptoKey>>>, }` | Shred-fähiger Schlüssel für Tier-2 Segmentdateien (Crypto-Shredding). Durch Aufruf von `shred()` wird der Schlüssel im Speicher gezeroized und gelösc… | `src/segment.rs` |
| `TenantIsolatedKvStore` | `{ shards: Box<[Shard]>, shard_count: usize, global_shard_offset: AtomicUsize, eviction_round_offsets: Box<[AtomicUsize]>, segment_capacity: NonZeroUsize, spill_h… }` | Tenant-isolierter KV-Segment-Store. | `src/store.rs` |
| `TenantPrefixKvStore` | `{ partitions: RwLock<AHashMap<(TenantId, PrefixKey), Partition>>, byte_budget_per_tenant: usize, reuse_policy: KvReusePolicy, next_tick: AtomicU64, next_group_id… }` | Mandantenisolierter KV-Prefix-Store. Bietet strukturelle Isolation via `(TenantId, PrefixKey)`-Key in den Partitionen. Unterstützt konfigurierbares B… | `src/prefix_store.rs` |
| `Tier2EncryptedSegment` | `{ pub tenant_id: TenantId, pub segment_id: u64, pub ciphertext: Vec<u8>, pub nonce: [u8; 12], pub key: ShreddableSegmentKey, }` | Verschlüsselte Tier-2 Segmentdatei (ausgelagerter KV-Block auf Disk). | `src/segment.rs` |

### Öffentliche Datentypen — Enums (2)

| Enum | Varianten | Beschreibung | Datei |
|---|---|---|---|
| `KvReusePolicy` | `/// Wiederverwendung aller Treffer mit `matched_len > 0`. Always, /// Kostenbasiert: Treffer erst ab Mindestpräfixlänge `min_prefix_len`. CostBased { min_prefix_len: usize }, /// Keinerlei Prefix-Reu…` | Policy zur Steuerung von Prefix-Reuse (§9.2). | `src/radix.rs` |
| `KvSegmentContent` | `Raw(bytes::Bytes), #[cfg(any(feature = "kivi-quantization", feature = "kvcache-kivi-quant"))] KiviQuantized(KiviQuantizedBlock),` | Content representation state for a KV segment. | `src/segment.rs` |

### Typ-Aliase (1)

| Alias | Definition | Beschreibung |
|---|---|---|
| `SpillHandler` | `type SpillHandler= Arc<dyn Fn(TenantId, u64, Vec<u8>) + Send + Sync>` | Optionaler Callback-Hook für Tier-2-LSM-Spill bei Eviction aus dem In-Memory LRU Cache. |

### Konstanten (5)

| Konstante | Typ | Beschreibung |
|---|---|---|
| `CURRENT_KV_KEY_DERIVATION_VERSION` | `u8` | Aktuelle Version der KV-Segment-Schlüsselableitung. Erhöhe diesen Wert, wenn sich der HKDF-Info-String oder der Salt-Aufbau ändert. |
| `DEFAULT_BYTE_BUDGET_PER_TENANT` | `usize` | Standard-Byte-Budget pro Tenant (256 MiB). |
| `DEFAULT_SEGMENT_CAPACITY_PER_TENANT` | `usize` | — |
| `DEFAULT_SHARD_COUNT` | `usize` | — |
| `MAX_PINNED_BYTES_PER_TENANT` | `u64` | Maximum allowed total pinned bytes per tenant for un-expiring pinned segments (`Pin { ttl: None }`). |

### Öffentliche Funktionen (101)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn rank_for_eviction( candidates: &[(u64, Instant)], scores: &dyn AttentionScoreSource, ) -> Vec<u64>` | Ranks eviction candidates using a default balanced weighting (50% LRU age, 50% attention score). Returns a vector of segment IDs ordered by… | `src/attention_score.rs` |
| `pub fn rank_for_eviction_weighted( candidates: &[(u64, Instant)], scores: &dyn AttentionScoreSource, attention_weight: f32, ) -> Vec<u64>` | Ranks eviction candidates combining normalized LRU access age and attention scores. `attention_weight` must be in `[0.0, 1.0]`, controlling… | `src/attention_score.rs` |
| `pub fn rank_for_eviction_weighted_for_tenant( tenant_id: TenantId, candidates: &[(u64, Instant)], scores: &dyn AttentionScoreSource, attention_weight: f32, ) -> Vec<u64>` | Ranks eviction candidates for a specific tenant combining normalized LRU access age and attention scores. | `src/attention_score.rs` |
| `pub fn attention_source(&self) -> Arc<dyn AttentionScoreSource>` | Liefert die aktuell konfigurierte AttentionScoreSource. | `src/eviction_worker.rs` |
| `pub fn emergency_wipe(store: &TenantIsolatedKvStore)` | SEPARATER Pfad für Notfall-Löschung (z.B. Prozess-Shutdown, expliziter Sicherheits-Trigger). Synchron, blockierend, garantiert vor Rückkehr… | `src/eviction_worker.rs` |
| `pub fn notify_block_released(&self, block_id: u64)` | Benachrichtigt den EvictionWorker, dass eine Block-Referenz freigegeben wurde. | `src/eviction_worker.rs` |
| `pub fn register_segment_request( &self, tenant_id: TenantId, segment_id: u64, request_id: RequestId, )` | Registriert eine Zuordnung von `(tenant_id, segment_id)` zu einer `request_id`. | `src/eviction_worker.rs` |
| `pub fn shutdown(&self)` | Beendet den Worker-Thread geordnet. | `src/eviction_worker.rs` |
| `pub fn spawn(store: Arc<TenantIsolatedKvStore>) -> Self` | Spawnt den Eviction-Worker auf einem dedizierten OS-Thread. | `src/eviction_worker.rs` |
| `pub fn trigger_eviction(&self, target_free_bytes: usize)` | Nicht-blockierender Trigger vom Hot-Path aus. | `src/eviction_worker.rs` |
| `pub fn unregister_segment_request(&self, tenant_id: TenantId, segment_id: u64)` | Entfernt die Zuordnung für `(tenant_id, segment_id)`. | `src/eviction_worker.rs` |
| `pub fn with_attention_exporter(self, exporter: Arc<dyn AttentionExporter>) -> Self` | Builder-Methode zur Injektion eines echten Attention-Backends. | `src/eviction_worker.rs` |
| `pub fn evict(&self, tenant: TenantId, key: &PrefixKey) -> Result<u64, ContextraError>` | Evictiert KV-Präfix-Einträge für den angegebenen Mandanten und PrefixKey. Gibt die Anzahl der entfernten Blöcke zurück. | `src/prefix_store.rs` |
| `pub fn insert( &self, tenant: TenantId, key: &PrefixKey, tokens: &[u32], blocks: Vec<KvBlock>, ) -> Result<(), ContextraError>` | Fügt KV-Blöcke für ein Token-Präfix in den Store ein. | `src/prefix_store.rs` |
| `pub fn invalidate_prefix_scope_keys( &self, tenant: TenantId, scratchpad_key_prefix: &[u32], keys: &[PrefixKey], ) -> Result<usize, ContextraError>` | Gezielte Invalidation eines Scratchpad-Scopes für den angegebenen Mandanten über alle oder spezifische PrefixKeys. Entfernt ausschließlich… | `src/prefix_store.rs` |
| `pub fn lookup(&self, tenant: TenantId, key: &PrefixKey, tokens: &[u32]) -> Option<KvPrefixHit>` | Sucht nach dem längsten übereinstimmenden Präfix in Blöcken für die angegebenen Tokens. | `src/prefix_store.rs` |
| `pub fn new() -> Self` | Erstellt einen neuen `TenantPrefixKvStore` mit Standard-Konfiguration (256 MiB Budget). | `src/prefix_store.rs` |
| `pub fn with_byte_budget_per_tenant(mut self, budget: usize) -> Self` | Konfiguriert das maximale Byte-Budget pro Mandant über alle seine Präfix-Partitionen. | `src/prefix_store.rs` |
| `pub fn with_capacity_and_policy(budget: usize, policy: KvReusePolicy) -> Self` | Erstellt einen `TenantPrefixKvStore` mit spezifischem Byte-Budget und Reuse-Policy. | `src/prefix_store.rs` |
| `pub fn with_reuse_policy(mut self, policy: KvReusePolicy) -> Self` | Konfiguriert die Wiederverwendungs-Policy (`KvReusePolicy`). | `src/prefix_store.rs` |
| `pub fn compress_bytes(input: &[u8]) -> Vec<u8>` | Losslessly compresses byte slice for LSM spill (Quantization -> Compression -> AEAD). | `src/quantize_kivi.rs` |
| `pub fn decompress_bytes(input: &[u8]) -> Result<Vec<u8>>` | Losslessly decompresses byte slice produced by `compress_bytes`. | `src/quantize_kivi.rs` |
| `pub fn from_bytes(bytes: &[u8]) -> Result<Self>` | Deserializes tensor view from bytes using bincode. | `src/quantize_kivi.rs` |
| `pub fn kivi_dequantize(bytes: &[u8], meta: &KiviBlockMeta) -> Result<KvTensorView>` | Dequantizes packed 2-bit bytes back into `KvTensorView`. | `src/quantize_kivi.rs` |
| `pub fn kivi_quantize(raw: &KvTensorView, config: KiviQuantizeConfig) -> Result<KiviQuantizedBlock>` | Quantizes raw `KvTensorView` into 2-bit `KiviQuantizedBlock`. Implements asymmetric 2-bit quantization (per-channel for keys, per-token for… | `src/quantize_kivi.rs` |
| `pub fn new( keys: Vec<f32>, values: Vec<f32>, num_tokens: usize, num_channels: usize, ) -> Result<Self>` | Creates a new `KvTensorView` validating element count dimensions. | `src/quantize_kivi.rs` |
| `pub fn pack_kivi_block(values: &[f32]) -> KiviQuantizedBlock` | Reference implementation for 2-bit linear min/max quantization. | `src/quantize_kivi.rs` |
| `pub fn to_bytes(&self) -> Result<Vec<u8>>` | Serializes tensor view to bytes using bincode. | `src/quantize_kivi.rs` |
| `pub fn unpack_kivi_block(block: &KiviQuantizedBlock) -> Result<Vec<f32>>` | Dequantizes a 2-bit packed block back into floating point `Vec<f32>`. | `src/quantize_kivi.rs` |
| `pub fn active_refs(&self) -> usize` | Gibt die aktuelle Anzahl aktiver Referenzen zurück. | `src/radix.rs` |
| `pub fn block_id(&self) -> u64` | Gibt die Block-ID zurück. | `src/radix.rs` |
| `pub fn clear(&mut self)` | Leert den gesamten Baum. | `src/radix.rs` |
| `pub fn find_longest_prefix( &self, tokens: &[u32], policy: KvReusePolicy, ) -> Option<PrefixMatch>` | Sucht das längste übereinstimmende Präfix unter Anwendung der `KvReusePolicy`. | `src/radix.rs` |
| `pub fn insert(&mut self, tokens: &[u32], block_id: u64) -> Result<(), ContextraError>` | Fügt eine Token-Sequenz mit assoziierter KV-Block-ID ein. | `src/radix.rs` |
| `pub fn is_empty(&self) -> bool` | Prüft, ob der Baum leer ist. | `src/radix.rs` |
| `pub fn len(&self) -> usize` | Gibt die Anzahl gespeicherter Präfixe zurück. | `src/radix.rs` |
| `pub fn new( block_id: u64, tenant_id: TenantId, ref_counter: Arc<AtomicUsize>, worker: Option<Arc<EvictionWorker>>, ) -> Self` | Erstellt einen neuen Guard und inkrementiert den Referenzzähler. | `src/radix.rs` |
| `pub fn new(tenant_id: TenantId) -> Self` | Erstellt einen neuen Prefix-Radix-Baum für den angegebenen Tenant. | `src/radix.rs` |
| `pub fn remove(&mut self, tokens: &[u32]) -> Option<u64>` | Entfernt eine Token-Sequenz aus dem Baum. | `src/radix.rs` |
| `pub fn should_reuse(&self, matched_len: usize) -> bool` | Prüft, ob ein Treffer mit der Länge `matched_len` wiederverwendet werden soll. | `src/radix.rs` |
| `pub fn tenant_id(&self) -> TenantId` | Gibt die Mandanten-ID zurück. | `src/radix.rs` |
| `pub fn tenant_id(&self) -> TenantId` | Gibt die Tenant-ID des Radix-Baums zurück. | `src/radix.rs` |
| `pub fn new(tenant_id: TenantId, scratchpad_key_prefix: Vec<u32>) -> Self` | Erstellt einen neuen `ScratchpadCacheScope` für den angegebenen Mandanten und das Token-Präfix. | `src/scratchpad_invalidation.rs` |
| `pub fn new(store: &'a TenantPrefixKvStore) -> Self` | Erstellt einen neuen `PrefixStoreScratchpadInvalidator` für den angegebenen Store. | `src/scratchpad_invalidation.rs` |
| `pub fn with_keys(mut self, keys: Vec<PrefixKey>) -> Self` | Konfiguriert die zu prüfenden `PrefixKey`s für die Invalidation. | `src/scratchpad_invalidation.rs` |
| `pub fn acquire_guard(&self, worker: Option<Arc<EvictionWorker>>) -> KvBlockGuard` | Erstellt einen `KvBlockGuard` für diesen Block und inkrementiert den Referenzzähler. | `src/segment.rs` |
| `pub fn active_refs(&self) -> usize` | Gibt die aktuelle Anzahl aktiver Referenzen zurück. | `src/segment.rs` |
| `pub fn as_bytes(&self) -> &[u8]` | Read-Only-Zugriff. Kein Klartext-Export nach außen ohne expliziten Call. | `src/segment.rs` |
| `pub fn decrypt(&self, ciphertext: &[u8], nonce: &[u8; 12]) -> Result<Vec<u8>, ContextraError>` | Entschlüsselt Daten mit AES-256-GCM-SIV. | `src/segment.rs` |
| `pub fn decrypt_data(&self, cipher: &KvSegmentCipher) -> Result<Vec<u8>, CryptoError>` | Entschlüsselt die Daten des Segments, falls es verschlüsselt ist. | `src/segment.rs` |
| `pub fn encrypt(&self, plaintext: &[u8]) -> Result<(Vec<u8>, [u8` | Verschlüsselt Daten mit AES-256-GCM-SIV und automatischem Nonce. | `src/segment.rs` |
| `pub fn is_empty(&self) -> bool` | Prüft ob das Segment leer ist. | `src/segment.rs` |
| `pub fn is_shredded(&self) -> bool` | Prüft, ob der Schlüssel ge-shredded wurde. | `src/segment.rs` |
| `pub fn len(&self) -> usize` | Länge der Tensor-Bytes in Bytes. | `src/segment.rs` |
| `pub fn new( tenant_id: TenantId, segment_id: u64, plaintext: &[u8], key: ShreddableSegmentKey, ) -> Result<Self, ContextraError>` | Verschlüsselt Klartext-Tensor-Bytes und erstellt ein neues Tier-2 Segment. | `src/segment.rs` |
| `pub fn new(tenant_id: TenantId, segment_id: u64, data: Vec<u8>) -> Self` | Erstellt ein neues Klartext-KV-Cache-Segment (Default / Zero-Config, P12-konform). | `src/segment.rs` |
| `pub fn new_encrypted( cipher: &KvSegmentCipher, tenant_id: TenantId, segment_id: u64, model_fingerprint: ModelFingerprint, rope_offset: Option<usize>, plaintext: &[u8], ) -> Result<Self, CryptoError>` | Erstellt ein verschlüsseltes KV-Cache-Segment aus einem Klartext-Tensor. | `src/segment.rs` |
| `pub fn new_quantized( tenant_id: TenantId, segment_id: u64, raw_kv: &KvTensorView, config: KiviQuantizeConfig, cipher: &dyn contextra_crypto::KvCipher, ) -> Result<Self, ContextraError>` | Erstellt ein neues quantisiertes KV-Cache-Segment. | `src/segment.rs` |
| `pub fn new_with_metadata( tenant_id: TenantId, segment_id: u64, data: Vec<u8>, #[cfg(feature = "kv-encryption")] model_fingerprint: Option<ModelFingerprint>, rope_offset: Option<usize>, ) -> Self` | Erstellt ein neues Klartext-KV-Cache-Segment mit optionalen Metadaten. | `src/segment.rs` |
| `pub fn new_with_random_key( tenant_id: TenantId, segment_id: u64, plaintext: &[u8], passphrase: &str, ) -> Result<Self, ContextraError>` | Erstellt ein neues Tier-2 Segment mit einem zufälligen Schlüssel aus der angegebenen Passphrase. | `src/segment.rs` |
| `pub fn read_and_decrypt(&self) -> Result<Vec<u8>, ContextraError>` | Liest und entschlüsselt das Segment. Scheitert sofort, wenn der Key ge-shredded wurde. | `src/segment.rs` |
| `pub fn read_dequantized( &self, cipher: &dyn contextra_crypto::KvCipher, ) -> Result<KvTensorView, ContextraError>` | Reads and dequantizes segment data, enforcing `INV-KIVI-AEAD-ORDER`: AEAD Decryption -> Decompression -> Dequantization. | `src/segment.rs` |
| `pub fn shred(&self)` | Crypto-Shredding: Vernichtet den Schlüssel im Speicher. Alle mit diesem Schlüssel verschlüsselten Daten werden dauerhaft unlesbar. | `src/segment.rs` |
| `pub fn to_kv_segment(&self) -> Result<KvSegment, ContextraError>` | Entschlüsselt das Tier-2 Segment und stellt ein In-Memory `KvSegment` wieder her. | `src/segment.rs` |
| `pub fn to_spill_bytes(&self) -> Vec<u8>` | Serialisiert das Segment bzw. dessen verschlüsselte Layer für Tier-2-LSM-Spill. | `src/segment.rs` |
| `pub fn try_new(passphrase: &str, salt: &[u8]) -> Result<Self, ContextraError>` | Erstellt einen neuen shred-fähigen Schlüssel aus Passphrase und Salt. | `src/segment.rs` |
| `pub fn try_new_random(passphrase: &str) -> Result<Self, ContextraError>` | Erstellt einen neuen shred-fähigen Schlüssel mit zufälligem Salt. | `src/segment.rs` |
| `pub fn write_quantized( &mut self, raw_kv: &KvTensorView, config: KiviQuantizeConfig, cipher: &dyn contextra_crypto::KvCipher, ) -> Result<(), ContextraError>` | Writes quantized raw KV tensor into segment, enforcing `INV-KIVI-AEAD-ORDER`: Quantization -> Compression -> AEAD Encryption. | `src/segment.rs` |
| `pub fn acquire_block_guard( &self, tenant: TenantId, block_id: u64, worker: Option<Arc<EvictionWorker>>, ) -> Option<KvBlockGuard>` | Erstellt einen `KvBlockGuard` für einen bestimmten Block. | `src/store.rs` |
| `pub fn attention_source(&self) -> Arc<dyn AttentionScoreSource>` | Liefert die konfigurierte Standard-AttentionScoreSource für diesen Store. | `src/store.rs` |
| `pub fn clear_all(&self)` | — | `src/store.rs` |
| `pub fn evict_fair(&self, target_free_bytes: usize, scores: &dyn AttentionScoreSource) -> usize` | Evictiert KV-Segmente unter Erhaltung von Tenant-Fairness und Attention-Scores (Default 50/50 Gewichtung). | `src/store.rs` |
| `pub fn evict_lru_fair(&self, target_free_bytes: usize) -> usize` | Evictiert KV-Segmente unter Erhaltung von Tenant-Fairness und Guard-Schutz (`active_refs == 0`). | `src/store.rs` |
| `pub fn evict_lru_fair_with_hook<F>(&self, target_free_bytes: usize, mut hook: F) -> usize where F: FnMut(),` | — | `src/store.rs` |
| `pub fn evict_weighted_fair( &self, target_free_bytes: usize, scores: &dyn AttentionScoreSource, attention_weight: f32, ) -> usize` | Evictiert KV-Segmente unter Erhaltung von Tenant-Fairness und gewichteten Attention-Scores. | `src/store.rs` |
| `pub fn find_prefix_match( &self, tenant: TenantId, tokens: &[u32], policy: KvReusePolicy, worker: Option<Arc<EvictionWorker>>, ) -> Option<(PrefixMatch, KvBlockGuard)>` | Sucht ein Präfix-Match für die angegebenen Tokens unter der gewünschten `KvReusePolicy`. | `src/store.rs` |
| `pub fn get_decrypted_segment( &self, cipher: &contextra_crypto::KvSegmentCipher, tenant: TenantId, segment_id: u64, ) -> Result<Option<Vec<u8>>, contextra_crypto::CryptoError>` | Liest ein Segment und entschlüsselt es falls nötig. | `src/store.rs` |
| `pub fn get_dequantized_segment( &self, tenant: TenantId, segment_id: u64, cipher: &dyn contextra_crypto::KvCipher, ) -> Result<Option<super::quantize_kivi::KvTensorView>, ContextraError>` | Liest und dequantisiert ein gespeichertes Segment. | `src/store.rs` |
| `pub fn get_segment_bytes(&self, tenant: TenantId, segment_id: u64) -> Option<Vec<u8>>` | Liefert unverschlüsselte Segment-Bytes für einen Tenant. | `src/store.rs` |
| `pub fn get_segments(&self, tenant: TenantId) -> Vec<u64>` | Liefert alle gespeicherten Segment-IDs eines Tenants. | `src/store.rs` |
| `pub fn get_tenant_segment_len(&self, tenant: TenantId) -> usize` | Gibt die Anzahl der gespeicherten Segmente für einen Mandanten zurück. | `src/store.rs` |
| `pub fn insert_encrypted_segment( &self, cipher: &contextra_crypto::KvSegmentCipher, tenant: TenantId, segment_id: u64, model_fingerprint: contextra_crypto::ModelFingerprint, rope_offset: Option<usize>, plaintext: &[u8], ) -> Result<(), contextra_crypto::Crypt…` | Verschlüsselt einen Klartext-Tensor und fügt ein verschlüsseltes Segment ein. | `src/store.rs` |
| `pub fn insert_quantized_segment( &self, tenant: TenantId, segment_id: u64, raw_kv: &super::quantize_kivi::KvTensorView, config: super::quantize_kivi::KiviQuantizeConfig, cipher: &dyn contextra_crypto::KvCipher, ) -> Result<(), ContextraError>` | Fügt ein quantisiertes Segment in den Store ein. | `src/store.rs` |
| `pub fn insert_segment(&self, tenant: TenantId, segment: KvSegment)` | Fügt ein Segment für einen bestimmten Tenant ein. | `src/store.rs` |
| `pub fn insert_segment_for_request( &self, tenant: TenantId, segment: KvSegment, request_id: contextra_ports::RequestId, )` | Fügt ein Segment zusammen mit einer `RequestId` ein und registriert die Request-Zuordnung. | `src/store.rs` |
| `pub fn insert_segment_with_directive( &self, tenant: TenantId, segment: KvSegment, directive: CacheDirective, ) -> Result<(), ContextraError>` | Inserts a segment with an explicit declarative cache directive. | `src/store.rs` |
| `pub fn insert_token_sequence( &self, tenant: TenantId, tokens: &[u32], block_id: u64, ) -> Result<(), ContextraError>` | Fügt eine Token-Sequenz und deren assoziierte Block-ID in den Prefix-Radix-Baum ein. | `src/store.rs` |
| `pub fn new() -> Self` | Erstellt einen neuen tenant-isolierten KV-Store. | `src/store.rs` |
| `pub fn on_rollback(&self, tenant: TenantId, chunk_ids: &[u64])` | Bereinigt assoziierte KV-Segmente bei einem transaktionalen Rollback. | `src/store.rs` |
| `pub fn pin_block(&self, tenant: TenantId, block_id: u64) -> Option<KvBlockGuard>` | Pinned einen Block im Store mittels Guard gegen vorzeitige LRU-Eviction. | `src/store.rs` |
| `pub fn purge_tenant(&self, tenant: TenantId)` | Purges all cached KV segments and radix state for the given tenant. | `src/store.rs` |
| `pub fn register_segment_request( &self, tenant_id: TenantId, segment_id: u64, request_id: contextra_ports::RequestId, )` | Registriert eine Zuordnung von `(tenant_id, segment_id)` zu einer `request_id` für Attention-Score-basierte Eviction. | `src/store.rs` |
| `pub fn release_step(&self, tenant: TenantId, step_id: StepId)` | Releases transient cache segments created during a specific step. | `src/store.rs` |
| `pub fn remove_doc_segments(&self, tenant: TenantId, doc: contextra_types::DocId)` | Removes KV cache segments associated with a document ID. | `src/store.rs` |
| `pub fn remove_segment(&self, tenant: TenantId, segment_id: u64)` | Entfernt ein einzelnes Segment. | `src/store.rs` |
| `pub fn remove_segments_for_rollback(&self, tenant: TenantId, segment_ids: &[u64])` | Entfernt Segmente mit den angegebenen `segment_ids` für einen Tenant. | `src/store.rs` |
| `pub fn set_attention_source(&self, source: Arc<dyn AttentionScoreSource>)` | Setzt die konfigurierte Standard-AttentionScoreSource für diesen Store. | `src/store.rs` |
| `pub fn set_spill_handler(&self, handler: SpillHandler)` | Registriert einen Spill-Handler für evictierte Segmente. | `src/store.rs` |
| `pub fn unregister_segment_request(&self, tenant_id: TenantId, segment_id: u64)` | Entfernt die Zuordnung für `(tenant_id, segment_id)`. | `src/store.rs` |
| `pub fn with_capacity(segment_capacity_per_tenant: usize) -> Self` | Erstellt einen Store mit konfigurierter Segment-Kapazität pro Tenant. | `src/store.rs` |
| `pub fn with_shard_count(n: usize) -> Self` | Erstellt einen Store mit angegebener Shard-Anzahl. | `src/store.rs` |


---

## `contextra-checkpoint`

| Attribut | Wert |
|---|---|
| **Ring** | 1 — Ring 1 — Persistence & Cache |
| **Status** | Fertig |
| **Zweck** | Time-Travel-Registry und Checkpoint-Verwaltung ohne globalen Zustand. |
| **Unsafe-Code** | Nein (`#![forbid(unsafe_code)]`) |
| **Interne Abhängigkeiten** | `contextra-core`, `contextra-ports`, `contextra-types` |
| **Externe Abhängigkeiten (Auswahl)** | `blake3`, `bytes`, `dirs`, `futures-executor`, `parking_lot`, `serde`, `serde_json`, `tokio`, `tracing` |

### Modul- und Re-Export-Oberfläche (lib.rs)

```rust
pub use guard::{CheckpointGuard, PinGuard};
pub use hardlink_cloner::{CheckpointHardlinkCloner, DefaultHardlinkCloner, HardlinkCloneResult};
pub use manifest::CheckpointManifest;
pub use meta::{CheckpointMeta, StateCheckpoint};
pub use orphan::{ await_pending_rollbacks, clear_all_orphaned_checkpoints, clear_orphaned_checkpoint, get_orphaned_checkpoints, get_orphaned_checkpoints_for_namespace, global_orphan_registry, monotonic_timestamp_ms, monotonic_timestamp_ms_with, orphaned_checkpoint_count, pending_rollback_count, register_orphaned_checkpoint, register_pinned_seq_no_orphan, InstanceOrphanRegistry, OrphanRegistry, OrphanState, PinId, PinnedSeqNoOrphan, };
pub use store::{CheckpointRegistry, PersistentCheckpointStore};
```

### Schnittstellen — Public Traits (2)

**`CheckpointHardlinkCloner`** — *Trait defining the physical hardlink cloning interface for SSTables. # Windows NTFS Limit On Windows systems, `CreateHardLink` is NTFS-exclusive and capped at **1023 links** per volume file. Exceeding this limit or linking across different filesystems/mount p…* (`src/hardlink_cloner.rs`)

```rust
pub trait CheckpointHardlinkCloner : Send + Sync {
/// Clones all SSTable files visible for `seq_no` in `source_dir` into `target_dir` via physical hardlinks.
    ///
    /// Maintains an active `SnapshotRegistry` pin for `seq_no` during the operation.
    fn clone_sstable_hardlinks<'a>(
        &'a self,
        seq_no: u64,
        source_dir: &'a Path,
        target_dir: &'a Path,
        snapshot_registry: &'a Arc<SnapshotRegistry>,
    ) -> contextra_ports::BoxFuture<'a, Result<HardlinkCloneResult>>;
}
```

**`CheckpointRegistry`** — *Trait für die Checkpoint-Verwaltung.* (`src/store.rs`)

```rust
pub trait CheckpointRegistry : contextra_ports::Checkpoint + Send + Sync {
fn save_checkpoint<'a>(&'a self, meta: CheckpointMeta) -> BoxFuture<'a, Result<()>>;
    fn load_checkpoint<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, Result<Option<CheckpointMeta>>>;
    fn list_checkpoints<'a>(&'a self) -> BoxFuture<'a, Result<Vec<CheckpointMeta>>>;

    fn orphan_registry(&self) -> Option<&Arc<InstanceOrphanRegistry>> {
        None
    }

    fn register_orphan(&self, pin_id: PinId) -> std::io::Result<()> {
        if let Some(registry) = self.orphan_registry() {
            registry.register_orphan(pin_id)
        } else {
            Ok(())
        }
    }

    fn create_hardlink_clone<'a>(
        &'a self,
        seq_no: u64,
        source_dir: &'a Path,
        target_dir: &'a Path,
        snapshot_registry: &'a Arc<SnapshotRegistry>,
    ) -> BoxFuture<'a, Result<HardlinkCloneResult>> {
        Box::pin(async move {
            let _ = (seq_no, source_dir, target_dir, snapshot_registry);
            Err(ContextraError::Internal(
                "Hardlink clone not supported for this registry".into(),
            ))
        })
    }

    fn recover_orphaned_pins<'a>(&'a self) -> BoxFuture<'a, Result<Vec<PinId>>> {
        Box::pin(async move { Ok(Vec::new()) })
    }

    fn recover_orphaned_checkpoints<'a>(&'a self) -> BoxFuture<'a, Result<Vec<TxId>>> {
        Box::pin(async move { Ok(Vec::new()) })
    }

    fn flush_orphan_registry<'a>(&'a self) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }
}
```

### Öffentliche Datentypen — Structs (15)

| Struct | Felder / Form | Beschreibung | Datei |
|---|---|---|---|
| `CheckpointGuard` | `{ pub(crate) checkpoint: Option<StateCheckpoint>, pub(crate) storage: Arc<S>, pub(crate) namespace: String, pub(crate) orphan_registry: Arc<InstanceOrphanRegistr… }` | RAII Guard, der explizit über [`commit`](Self::commit) oder [`rollback`](Self::rollback) finalisiert werden MUSS. # RAII-Kontrakt und Betriebshinweis… | `src/guard.rs` |
| `CheckpointManifest` | `{ pub meta: CheckpointMeta, pub components: Vec<String>, pub checksum: String, }` | AI-TAG\[PANIC-SAFETY\]\[CRITICAL\] RESOLVED: AGT-CKPT-f3a1b2c4 (TS:2026-08-29T08:06:29Z) (SESSION:14348074) Fault-Injection-Tests in tests/manifest_f… | `src/manifest.rs` |
| `CheckpointMeta` | `{ pub name: String, pub collection_id: String, pub seq_no: u64, pub tx_id: TxId, pub metadata: serde_json::Value, pub created_at: u64, }` | Metadata for a persistent checkpoint. | `src/meta.rs` |
| `DefaultHardlinkCloner` | `(tuple/unit)` | Productive implementation of [`CheckpointHardlinkCloner`] using `tokio::fs`. | `src/hardlink_cloner.rs` |
| `HardlinkCloneResult` | `{ /// The source sequence number pinned and cloned. pub source_seq_no: u64, /// List of target paths created as hardlinks. pub linked_files: Vec<PathBuf>, /// WA… }` | Result structure returned upon completing an SSTable hardlink clone operation. | `src/hardlink_cloner.rs` |
| `InstanceOrphanRegistry` | `{ pins: Mutex<Vec<PinnedSeqNoOrphan>>, checkpoints: Mutex<Vec<StateCheckpoint>>, persist_path: std::path::PathBuf, is_dirty: AtomicBool, clock: Arc<dyn contextra… }` | Instance-scoped orphan registry for checkpoints and pinned sequence numbers (ADR-053). | `src/orphan.rs` |
| `LoomDirEntry` | `(tuple/unit)` | — | `src/hardlink_cloner.rs` |
| `LoomMetadata` | `(tuple/unit)` | — | `src/hardlink_cloner.rs` |
| `LoomReadDir` | `(tuple/unit)` | — | `src/hardlink_cloner.rs` |
| `OrphanRegistry` | `{ inner: InstanceOrphanRegistry, }` | — | `src/orphan.rs` |
| `OrphanState` | `{ pub checkpoints: Vec<StateCheckpoint>, pub pinned_seq_nos: Vec<PinnedSeqNoOrphan>, pub persist_path: std::path::PathBuf, }` | Instance-scoped orphan state for checkpoints and pinned sequence numbers. | `src/orphan.rs` |
| `PersistentCheckpointStore` | `{ storage: Arc<S>, /// Registrierter Checkpoint-Index im Arbeitsspeicher — geschützt durch ein konsolidiertes RwLock (P-23) index: RwLock<CheckpointIndex>, /// N… }` | Registry für gespeicherte Checkpoints mit Thread-sicherem Zustand. # Invarianten - Alle Methoden sind durch `RwLock` thread-sicher - `StorageEngine`-… | `src/store.rs` |
| `PinGuard` | `{ storage: Arc<S>, seq_no: Option<u64>, orphan_registry: Arc<InstanceOrphanRegistry>, clock: Arc<dyn contextra_ports::Clock>, }` | RAII-Guard für gepinnte Checkpoint-Sequenznummern (gemäß ADR-015). Garantiert, dass eine gepinnte Sequenznummer bei einem Fehler oder Panic während d… | `src/guard.rs` |
| `PinnedSeqNoOrphan` | `{ pub seq_no: u64, pub timestamp_ms: u64, }` | Orphaned gepinnte Sequenznummer — wird beim Recovery verarbeitet. | `src/orphan.rs` |
| `StateCheckpoint` | `{ pub tx_id: TxId, pub timestamp_ms: u64, #[serde(default)] pub namespace: Option<String>, }` | Point-in-Time Checkpoint representing an agent step or transaction boundary. | `src/meta.rs` |

### Typ-Aliase (1)

| Alias | Definition | Beschreibung |
|---|---|---|
| `PinId` | `type PinId= u64` | Type alias for sequence numbers managed as pinned checkpoint identifiers. |

### Öffentliche Funktionen (101)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn checkpoint(&self) -> Result<&StateCheckpoint>` | — | `src/guard.rs` |
| `pub fn clock(&self) -> &Arc<dyn contextra_ports::Clock>` | — | `src/guard.rs` |
| `pub fn clock(&self) -> &Arc<dyn contextra_ports::Clock>` | — | `src/guard.rs` |
| `pub fn commit(mut self) -> Result<StateCheckpoint>` | — | `src/guard.rs` |
| `pub fn defuse(mut self)` | Entschärft den Guard nach erfolgreicher Persistierung, sodass die Sequenznummer gepinnt bleibt. | `src/guard.rs` |
| `pub async fn for_agent_step(storage: Arc<S>, tx: TxId) -> Result<Self>` | Erstellt einen neuen CheckpointGuard für einen Agenten-Schritt. | `src/guard.rs` |
| `pub async fn for_agent_step_with_clock( storage: Arc<S>, tx: TxId, clock: Arc<dyn contextra_ports::Clock>, ) -> Result<Self>` | — | `src/guard.rs` |
| `pub async fn for_agent_step_with_registry( storage: Arc<S>, tx: TxId, orphan_registry: Arc<InstanceOrphanRegistry>, ) -> Result<Self>` | — | `src/guard.rs` |
| `pub fn new(checkpoint: StateCheckpoint, storage: Arc<S>, namespace: impl Into<String>) -> Self` | — | `src/guard.rs` |
| `pub async fn pin( storage: Arc<S>, seq_no: u64, orphan_registry: Arc<InstanceOrphanRegistry>, ) -> Result<Self>` | — | `src/guard.rs` |
| `pub async fn pin_with_clock( storage: Arc<S>, seq_no: u64, orphan_registry: Arc<InstanceOrphanRegistry>, clock: Arc<dyn contextra_ports::Clock>, ) -> Result<Self>` | — | `src/guard.rs` |
| `pub async fn rollback(mut self) -> Result<()>` | Führt ein manuelles, asynchrones Rollback des Checkpoints aus. Nach Aufruf von `rollback()` ist der Guard konsumiert, sodass beim Drop kein… | `src/guard.rs` |
| `pub fn rollback_blocking(mut self) -> Result<()>` | Führt ein synchrones, blockierendes Rollback des Checkpoints aus. # Einschränkung Diese Methode darf **nicht** aus einem laufenden asynchro… | `src/guard.rs` |
| `pub async fn unpin(mut self) -> Result<()>` | Entpinnt die Sequenznummer explizit und asynchron bei einem abgefangenen Fehler. | `src/guard.rs` |
| `pub fn with_clock( checkpoint: StateCheckpoint, storage: Arc<S>, namespace: impl Into<String>, clock: Arc<dyn contextra_ports::Clock>, ) -> Self` | — | `src/guard.rs` |
| `pub fn with_metrics_sink(mut self, sink: Arc<dyn contextra_ports::MetricsSink>) -> Self` | — | `src/guard.rs` |
| `pub fn with_metrics_sink_opt( mut self, sink: Option<Arc<dyn contextra_ports::MetricsSink>>, ) -> Self` | — | `src/guard.rs` |
| `pub fn with_registry( checkpoint: StateCheckpoint, storage: Arc<S>, namespace: impl Into<String>, orphan_registry: Arc<InstanceOrphanRegistry>, ) -> Self` | — | `src/guard.rs` |
| `pub fn with_registry_and_counter( checkpoint: StateCheckpoint, storage: Arc<S>, namespace: impl Into<String>, orphan_registry: Arc<InstanceOrphanRegistry>, skipped_rollbacks: Arc<AtomicU64>, ) -> Self` | — | `src/guard.rs` |
| `pub fn with_registry_counter_and_clock( checkpoint: StateCheckpoint, storage: Arc<S>, namespace: impl Into<String>, orphan_registry: Arc<InstanceOrphanRegistry>, skipped_rollbacks: Arc<AtomicU64>, clock: Arc<dyn contextra_ports::Clock>, ) -> Self` | — | `src/guard.rs` |
| `pub async fn create_dir_all<P: AsRef<Path>>(_path: P) -> std::io::Result<()>` | — | `src/hardlink_cloner.rs` |
| `pub async fn hard_link<P: AsRef<Path>, Q: AsRef<Path>>( _from: P, _to: Q, ) -> std::io::Result<()>` | — | `src/hardlink_cloner.rs` |
| `pub async fn metadata<P: AsRef<Path>>(_path: P) -> std::io::Result<LoomMetadata>` | — | `src/hardlink_cloner.rs` |
| `pub fn new() -> Self` | Creates a new [`DefaultHardlinkCloner`]. | `src/hardlink_cloner.rs` |
| `pub async fn next_entry(&mut self) -> std::io::Result<Option<LoomDirEntry>>` | — | `src/hardlink_cloner.rs` |
| `pub fn path(&self) -> std::path::PathBuf` | — | `src/hardlink_cloner.rs` |
| `pub async fn read_dir<P: AsRef<Path>>(_path: P) -> std::io::Result<LoomReadDir>` | — | `src/hardlink_cloner.rs` |
| `pub async fn remove_file<P: AsRef<Path>>(_path: P) -> std::io::Result<()>` | — | `src/hardlink_cloner.rs` |
| `pub fn new(meta: CheckpointMeta, components: Vec<String>) -> Result<Self>` | — | `src/manifest.rs` |
| `pub fn verify(&self) -> Result<()>` | — | `src/manifest.rs` |
| `pub fn into_workflow_state(&self) -> WorkflowState` | — | `src/meta.rs` |
| `pub async fn await_pending_rollbacks()` | Retained for backward compatibility. No background tasks are spawned during drop. | `src/orphan.rs` |
| `pub fn clear_all(&self)` | Synchronously clears memory and disk records. | `src/orphan.rs` |
| `pub fn clear_all(&self)` | — | `src/orphan.rs` |
| `pub fn clear_all_orphaned_checkpoints()` | — | `src/orphan.rs` |
| `pub fn clear_orphan_pin(&self, seq_no: u64)` | — | `src/orphan.rs` |
| `pub fn clear_orphaned_checkpoint(&self, tx_id: TxId)` | — | `src/orphan.rs` |
| `pub fn clear_orphaned_checkpoint(_tx_id: TxId)` | — | `src/orphan.rs` |
| `pub fn clock(&self) -> &Arc<dyn contextra_ports::Clock>` | — | `src/orphan.rs` |
| `pub fn drain_orphan_pins(&self) -> Vec<PinnedSeqNoOrphan>` | — | `src/orphan.rs` |
| `pub fn drain_orphaned_checkpoints(&self) -> Vec<StateCheckpoint>` | — | `src/orphan.rs` |
| `pub async fn flush_orphan_registry(&self) -> std::io::Result<()>` | Asynchronously flushes the in-memory orphan registry to disk if dirty. | `src/orphan.rs` |
| `pub fn get_orphan_pins(&self) -> Vec<PinnedSeqNoOrphan>` | — | `src/orphan.rs` |
| `pub fn get_orphaned_checkpoints(&self) -> Vec<StateCheckpoint>` | — | `src/orphan.rs` |
| `pub fn get_orphaned_checkpoints() -> Vec<StateCheckpoint>` | — | `src/orphan.rs` |
| `pub fn get_orphaned_checkpoints_for_namespace(ns: &str) -> Vec<StateCheckpoint>` | Retrieves orphaned checkpoints registered for a specific namespace. | `src/orphan.rs` |
| `pub fn get_orphans(&self) -> Vec<PinId>` | Retrieves all currently registered orphaned pin sequence numbers. | `src/orphan.rs` |
| `pub fn global_orphan_registry() -> &'static OrphanRegistry` | — | `src/orphan.rs` |
| `pub fn load_sync(path: &std::path::Path) -> Self` | — | `src/orphan.rs` |
| `pub fn load_sync(path: &std::path::Path) -> Self` | — | `src/orphan.rs` |
| `pub fn monotonic_timestamp_ms() -> u64` | Reads millisecond timestamp using the default SystemClock. | `src/orphan.rs` |
| `pub fn monotonic_timestamp_ms_with(clock: &dyn contextra_ports::Clock) -> u64` | Reads millisecond timestamp from the provided clock implementation. | `src/orphan.rs` |
| `pub fn new(file_path: impl Into<std::path::PathBuf>) -> Self` | — | `src/orphan.rs` |
| `pub fn new(persist_path: impl Into<std::path::PathBuf>) -> Self` | — | `src/orphan.rs` |
| `pub fn orphaned_checkpoint_count() -> usize` | Liefert die Anzahl der aktuell registrierten verwaisten ("orphaned") Checkpoints. | `src/orphan.rs` |
| `pub fn pending_rollback_count() -> usize` | Retained for backward compatibility. Always returns 0 as background task spawning is removed. | `src/orphan.rs` |
| `pub fn persist_sync(&self) -> std::io::Result<()>` | — | `src/orphan.rs` |
| `pub fn persist_sync(&self) -> std::io::Result<()>` | — | `src/orphan.rs` |
| `pub async fn recover_and_clean<S: contextra_ports::StorageEngine>( &self, storage: &S, ) -> Result<Vec<PinId>>` | Recovers all registered orphaned pins by unpinning them in storage and cleaning the registry. | `src/orphan.rs` |
| `pub async fn recover_and_clean<S: contextra_ports::StorageEngine>( &self, storage: &S, ) -> Result<Vec<PinId>>` | Recovers all registered orphaned pins in this instance registry by unpinning them in storage and cleaning the registry. | `src/orphan.rs` |
| `pub fn register_checkpoint_sync(&self, cp: StateCheckpoint)` | Registers an orphaned checkpoint in-memory without blocking disk I/O. # Crash-Safety & RAII Drop Semantics This method is called synchronou… | `src/orphan.rs` |
| `pub fn register_orphan(&self, pin_id: PinId) -> std::io::Result<()>` | Synchronously registers an orphaned sequence pin and persists to disk. | `src/orphan.rs` |
| `pub fn register_orphan(&self, pin_id: PinId) -> std::io::Result<()>` | Registers a pinned sequence number orphan in-memory without blocking disk I/O. # Crash-Safety & RAII Drop Semantics This method is called s… | `src/orphan.rs` |
| `pub fn register_orphan_sync(&self, orphan: PinnedSeqNoOrphan)` | Registers a pinned sequence number orphan in-memory without blocking disk I/O. # Crash-Safety & RAII Drop Semantics This method is called s… | `src/orphan.rs` |
| `pub fn register_orphaned_checkpoint(cp: StateCheckpoint)` | — | `src/orphan.rs` |
| `pub fn register_pinned_seq_no_orphan(orphan: PinnedSeqNoOrphan)` | — | `src/orphan.rs` |
| `pub fn set_clock(&mut self, clock: Arc<dyn contextra_ports::Clock>)` | — | `src/orphan.rs` |
| `pub fn with_clock(mut self, clock: Arc<dyn contextra_ports::Clock>) -> Self` | — | `src/orphan.rs` |
| `pub async fn allocate_tx(&self) -> Result<TxId>` | — | `src/store.rs` |
| `pub fn checkpoint_counter(&self) -> u64` | — | `src/store.rs` |
| `pub fn checkpoint_guard_skipped_rollback_count(&self) -> u64` | — | `src/store.rs` |
| `pub fn clear_all_orphaned_checkpoints(&self)` | — | `src/store.rs` |
| `pub fn clear_orphaned_checkpoint(&self, tx_id: TxId)` | — | `src/store.rs` |
| `pub fn clock(&self) -> &Arc<dyn contextra_ports::Clock>` | — | `src/store.rs` |
| `pub async fn close(&self) -> Result<()>` | Alias for `shutdown()`. Performs an explicit graceful close flush. | `src/store.rs` |
| `pub async fn create_checkpoint( &self, name: &str, collection_id: &str, seq_no: u64, tx_id: TxId, metadata: serde_json::Value, ) -> Result<CheckpointMeta>` | Creates a new persistent checkpoint. | `src/store.rs` |
| `pub fn create_guard(&self, tx_id: TxId) -> Result<CheckpointGuard<S>>` | Creates an ephemeral transactional checkpoint RAII guard. If the returned guard is dropped without calling `.commit()`, the underlying stor… | `src/store.rs` |
| `pub async fn create_hardlink_clone( &self, seq_no: u64, source_dir: &Path, target_dir: &Path, snapshot_registry: &Arc<SnapshotRegistry>, ) -> Result<HardlinkCloneResult>` | — | `src/store.rs` |
| `pub fn drain_orphan_pins(&self) -> Vec<PinnedSeqNoOrphan>` | Atomically drains all orphaned sequence pins from the underlying registry. | `src/store.rs` |
| `pub fn drain_orphaned_checkpoints(&self) -> Vec<StateCheckpoint>` | Atomically drains all orphaned checkpoints from the underlying registry. | `src/store.rs` |
| `pub async fn drop_checkpoint(&self, name: &str) -> Result<()>` | Deletes a persistent checkpoint by name. | `src/store.rs` |
| `pub async fn flush_orphan_registry(&self) -> Result<()>` | — | `src/store.rs` |
| `pub async fn get_checkpoint(&self, name: &str) -> Result<Option<CheckpointMeta>>` | — | `src/store.rs` |
| `pub fn get_orphaned_checkpoints(&self) -> Vec<StateCheckpoint>` | — | `src/store.rs` |
| `pub async fn list_checkpoints(&self) -> Result<Vec<CheckpointMeta>>` | Public inherent methods for compatibility | `src/store.rs` |
| `pub fn monotonic_timestamp_ms(&self) -> u64` | — | `src/store.rs` |
| `pub fn new(storage: Arc<S>, namespace: impl Into<String>) -> Result<Self>` | — | `src/store.rs` |
| `pub fn new_with_orphan_registry( storage: Arc<S>, namespace: impl Into<String>, orphan_registry: Arc<InstanceOrphanRegistry>, ) -> Result<Self>` | — | `src/store.rs` |
| `pub async fn open(storage: Arc<S>, namespace: impl Into<String>) -> Result<Self>` | Öffnet einen PersistentCheckpointStore asynchron mit Rekonstruktion und Monotonie-Garantie. | `src/store.rs` |
| `pub async fn open_with_orphan_registry( storage: Arc<S>, namespace: impl Into<String>, orphan_registry: Arc<InstanceOrphanRegistry>, ) -> Result<Self>` | Öffnet einen PersistentCheckpointStore mit einer spezifischen instanzgebundenen Orphan Registry. | `src/store.rs` |
| `pub fn orphan_registry(&self) -> &Arc<InstanceOrphanRegistry>` | — | `src/store.rs` |
| `pub async fn recover_orphaned_checkpoints(&self) -> Result<Vec<TxId>>` | Recovers all registered/persisted orphaned checkpoints during controlled startup or recovery. Checks the serialization barrier (`last_tx <=… | `src/store.rs` |
| `pub async fn recover_orphaned_pins(&self) -> Result<Vec<PinId>>` | — | `src/store.rs` |
| `pub fn register_orphaned_checkpoint(&self, cp: StateCheckpoint)` | — | `src/store.rs` |
| `pub fn register_pinned_seq_no_orphan(&self, orphan: PinnedSeqNoOrphan)` | — | `src/store.rs` |
| `pub async fn restore_checkpoint(&self, name: &str) -> Result<CheckpointMeta>` | Restores the system to a specific checkpoint by name. This will rollback the underlying storage to the transaction ID of the checkpoint. #… | `src/store.rs` |
| `pub fn set_metrics_sink(&mut self, sink: Arc<dyn contextra_ports::MetricsSink>)` | — | `src/store.rs` |
| `pub async fn shutdown(&self) -> Result<()>` | Performs an explicit shutdown flush of the orphan registry to disk. | `src/store.rs` |
| `pub fn skipped_rollback_count(&self) -> u64` | — | `src/store.rs` |
| `pub fn with_clock(mut self, clock: Arc<dyn contextra_ports::Clock>) -> Self` | Setzt den Zeitgeber für den `PersistentCheckpointStore`. # Composition Root Der Composition Root setzt per `with_clock` eine injizierte Uhr… | `src/store.rs` |
| `pub fn with_metrics_sink(mut self, sink: Arc<dyn contextra_ports::MetricsSink>) -> Self` | — | `src/store.rs` |


---

## `contextra-infer-candle`

| Attribut | Wert |
|---|---|
| **Ring** | 2 — Ring 2 — External Adapters & Sandboxing (Leaf Engines) |
| **Status** | In Migration |
| **Zweck** | Native GGUF-Modell-Inferenz via Candle. |
| **Unsafe-Code** | Nein (`#![forbid(unsafe_code)]`) |
| **Interne Abhängigkeiten** | `contextra-crypto`, `contextra-ports`, `contextra-rank`, `contextra-types` |
| **Externe Abhängigkeiten (Auswahl)** | `bincode`, `bytes`, `candle-core`, `candle-nn`, `candle-transformers`, `futures-util`, `half`, `parking_lot`, `serde`, `sha2`, `tokenizers`, `tokio`, `tracing` |

### Modul- und Re-Export-Oberfläche (lib.rs)

```rust
pub mod attention_exporter;
pub mod embedding;
pub mod embedding_provider;
pub mod gguf_loader;
pub mod inference;
pub mod model_registry;
pub mod gasp;
pub mod kv_bridge;
pub mod kv_state;
pub mod model;
pub use attention_exporter::*;
pub use embedding::CandleEmbedClient;
pub use embedding_provider::MAX_CANDLE_EMBED_BATCH_SIZE;
pub use gasp::{GaspConfig, GaspValidator, DEFAULT_GROUNDING_THRESHOLD};
pub use inference::QuantizedLlamaModel;
pub use inference::*;
pub use kv_bridge::KvBridgeAdapter;
pub use kv_state::{KvState, LayerKv};
pub use model::quantized_llama;
pub use model_registry::{compute_fingerprint, CandleQuantization, ModelFingerprint};
```

### Feature-Flags

| Flag | Aktiviert |
|---|---|
| `default` | `[]` |
| `candle` | `[]` |
| `kv-bridge` | `["dep:contextra-crypto", "contextra-crypto/kv-encryption", "dep:bincode"]` |
| `kv-stage-b` | `["dep:bincode"]` |
| `cuda` | `["candle-core/cuda", "candle-nn/cuda", "candle-transformers/cuda"]` |

### Schnittstellen — Public Traits (2)

**`CandleEmbedInner`** — *Inner trait abstracting low-level Candle forward execution for vector embeddings. Enables mock-based unit testing without loading full ONNX or GGUF files in CI.* (`src/embedding.rs`)

```rust
pub trait CandleEmbedInner : Send {
/// Generates an embedding vector for the provided input text.
    fn embed(
        &mut self,
        text: &str,
        tokenizer: &tokenizers::Tokenizer,
        device: &Device,
    ) -> Result<Vec<f32>>;

    /// Returns the vector dimension produced by this embedding model.
    fn dim(&self) -> usize;
}
```

**`CandleModelInner`** — *Inner trait abstracting low-level Candle forward/text-generation execution. This trait allows dependency injection for unit testing with mock models without requiring full GGUF binary weights in CI environments.* (`src/inference/core.rs`)

```rust
pub trait CandleModelInner : Send {
/// Generates text completion for a given prompt using tokenizer and device settings.
    fn generate(
        &mut self,
        prompt: &str,
        tokenizer: &tokenizers::Tokenizer,
        device: &Device,
    ) -> Result<String>;

    /// Generates text stream for a given prompt, invoking `on_token` for each generated token or chunk.
    /// Returns early if `on_token` returns `false`.
    fn generate_stream(
        &mut self,
        prompt: &str,
        tokenizer: &tokenizers::Tokenizer,
        device: &Device,
        on_token: &mut dyn FnMut(String) -> bool,
    ) -> Result<String> {
        let text = self.generate(prompt, tokenizer, device)?;
        on_token(text.clone());
        Ok(text)
    }

    /// Returns the last computed layer/head attention scores for the request if supported by the model implementation.
    fn last_attention_scores(&self) -> Option<Vec<Vec<f32>>> {
        None
    }

    /// Returns the KV layout and RoPE configuration of the underlying model, if supported.
    #[cfg(feature = "kv-stage-b")]
    fn kv_layout(
        &self,
    ) -> Option<(
        contextra_ports::kv::KvLayout,
        contextra_ports::kv::RopeConfig,
    )> {
        None
    }

    /// Generates text stream using an optional KV prefix seed state.
    #[cfg(feature = "kv-stage-b")]
    fn generate_stream_with_prefix(
        &mut self,
        prompt: &str,
        tokenizer: &tokenizers::Tokenizer,
        device: &Device,
        seed: Option<crate::inference::PrefixSeed>,
        on_token: &mut dyn FnMut(String) -> bool,
    ) -> Result<crate::inference::PrefixRun> {
        let _ = seed;
        let tokens = tokenizer
            .encode(prompt, true)
            .map_err(|e| ContextraError::InvalidInput(format!("Failed to tokenize prompt: {e}")))?;
        let prompt_tokens = tokens.get_ids().to_vec();
        let output = self.generate_stream(prompt, tokenizer, device, on_token)?;
        Ok(crate::inference::PrefixRun {
            output,
            prompt_tokens,
            reused_tokens: 0,
            exported_prefix: None,
        })
    }
}
```

### Öffentliche Datentypen — Structs (20)

| Struct | Felder / Form | Beschreibung | Datei |
|---|---|---|---|
| `BertEmbedModel` | `{ model: BertModel, dim: usize, }` | Real BERT transformer text embedding model wrapper. | `src/embedding.rs` |
| `CandleAttentionExporter` | `{ state: Mutex<ExporterState>, }` | Sums and exports attention weights across heads and layers of prefill steps. Maintains a bounded history (`MAX_TRACKED_REQUESTS = 256`) per active re… | `src/attention_exporter.rs` |
| `CandleEmbedClient` | `{ /// Hardware device (CPU, CUDA, Metal). pub device: Device, /// Thread-safe mutex wrapping the inner embedding model. pub model: Arc<tokio::sync::Mutex<Box<dyn… }` | Text embedding client powered by Candle ML backend. | `src/embedding.rs` |
| `CandleLlmClient` | `{ /// Target hardware device (CPU, CUDA, Metal). pub device: Device, /// Thread-safe mutex wrapping model execution state. pub model: Arc<tokio::sync::Mutex<Box<… }` | LLM Text Generator implementation powered by Candle inference engine. | `src/inference/core.rs` |
| `DefaultCandleEmbedModel` | `{ /// Vector dimension. pub dim: usize, }` | Default inner Candle embedding mock model for unit testing when weight files are missing. | `src/embedding.rs` |
| `DefaultCandleLlmModel` | `(tuple/unit)` | Default inner Candle LLM mock model for unit tests when binary weights are absent. | `src/inference/core.rs` |
| `GaspConfig` | `{ /// Schwellenwert für Grounding-Konfidenz (default: 0.70). pub threshold: f32, /// Benötigte Beobachtungen für Isotonic Warmup (default: 10). pub warmup_requir… }` | Konfiguration für den `GaspValidator`. | `src/gasp.rs` |
| `GaspValidator` | `{ config: GaspConfig, calibrator: Mutex<IsotonicCalibrator>, }` | Post-Hoc-Halluzinations-Validator (GASP / TPA-Pattern). | `src/gasp.rs` |
| `GgufMetadata` | `{ /// Identified architecture (e.g., "llama", "mistral"). pub architecture: String, /// Number of tensors present in the container. pub tensor_count: usize, ///… }` | Metadata extracted from a GGUF model container. | `src/gguf_loader.rs` |
| `KvBridgeAdapter` | `{ pub store: Arc<TenantIsolatedKvStore>, pub lsm_store: Option<Arc<dyn KvBridgeStorage>>, pub cipher: Arc<KvSegmentCipher>, pub consultations: Arc<std::sync::ato… }` | Verbindet Retrieval-Chunks mit dem mandantenisolierten, verschlüsselten KV-Cache. # Fail-Safe-Garantie Jede Methode fällt bei Fehler oder Cache-Miss… | `src/kv_bridge.rs` |
| `KvCacheKey` | `{ pub chunk_id: u64, pub fingerprint: ModelFingerprint, pub rope_offset: Option<usize>, }` | Cache-Lookup-Schlüssel: eindeutige Kombination aus Chunk-ID, Modell und optionalem RoPE-Offset. | `src/kv_bridge.rs` |
| `KvPrefixContext` | `{ /// Reference to the underlying KV prefix store port implementation. pub store: Arc<dyn KvPrefixStore>, }` | Wrapper context holding a thread-safe KV prefix store reference. | `src/inference/prefix_reuse.rs` |
| `KvState` | `{ /// Layer-wise KV cache tensor pairs. pub layers: Vec<LayerKv>, /// Active sequence length (token count) represented in the cache. pub pos: usize, }` | Explicit KV-Cache state holding layer-wise tensors and current sequence length position. | `src/kv_state.rs` |
| `LayerKv` | `{ /// Key tensor of shape `(batch, seq_len, n_kv_head, head_dim)` or `(batch, n_kv_head, seq_len, head_dim)`. pub k: Tensor, /// Value tensor of shape `(batch, s… }` | Represents the key (`k`) and value (`v`) tensors for a single transformer layer. | `src/kv_state.rs` |
| `LayerWeights` | `{ pub attention_wq: QuantizedMatMul, pub attention_wk: QuantizedMatMul, pub attention_wv: QuantizedMatMul, pub attention_wo: QuantizedMatMul, pub attention_norm:… }` | Transformer layer weights wrapping QMatMul attention projections, RMS norms, and KV cache. | `src/model/quantized_llama.rs` |
| `ModelWeights` | `{ pub tok_embeddings: Embedding, pub layers: Vec<LayerWeights>, pub norm: RmsNorm, pub output: QuantizedMatMul, pub masks: HashMap<(usize, usize), Tensor>, span:… }` | Quantized Llama model weights and layer structures with explicit KvState management. | `src/model/quantized_llama.rs` |
| `PrefixRun` | `{ /// Generated output string. pub output: String, /// Tokenized prompt sequence. pub prompt_tokens: Vec<u32>, /// Number of prompt tokens reused from the KV sto… }` | Execution result of a prefix-assisted generation run. | `src/inference/prefix_reuse.rs` |
| `PrefixSeed` | `{ /// Number of prompt tokens covered by the imported KV state. pub matched_len: usize, /// Imported and truncated KV state ready for remaining prompt prefill. p… }` | Seed state for executing prompt generation with an imported KV cache prefix. | `src/inference/prefix_reuse.rs` |
| `QuantizedLlamaModel` | `{ weights: ModelWeights, sample_len: usize, }` | Real quantized Llama / GGUF model execution wrapper. | `src/inference/core.rs` |
| `QuantizedMatMul` | `{ inner: QMatMul, span: tracing::Span, }` | — | `src/model/quantized_llama.rs` |

### Öffentliche Datentypen — Enums (1)

| Enum | Varianten | Beschreibung | Datei |
|---|---|---|---|
| `CandleQuantization` | `/// 4-bit medium K-quantization Q4KM, /// 8-bit quantization Q8_0, /// 16-bit floating point F16,` | Supported Candle quantization formats. | `src/model_registry.rs` |

### Konstanten (6)

| Konstante | Typ | Beschreibung |
|---|---|---|
| `DEFAULT_GROUNDING_THRESHOLD` | `f32` | Standard-Schwellenwert für Grounding-Konfidenz (Default: 0.70). |
| `DEFAULT_MAX_CONCURRENT_EMBEDDINGS` | `usize` | Default maximum concurrent embedding operations for Candle vector embedding. |
| `DEFAULT_MAX_CONCURRENT_INFERENCES` | `usize` | Default maximum concurrent inference operations for Candle LLM text generation. |
| `MAX_CANDLE_EMBED_BATCH_SIZE` | `usize` | Maximum batch size for Candle GGUF embeddings. Set to 256 (lower than ONNX's 512) because GGUF tensor activations and intermediate FP32/FP16 buffers… |
| `MAX_SEQ_LEN` | `usize` | — |
| `MAX_TRACKED_REQUESTS` | `usize` | Maximum number of tracked requests in the attention history ring buffer. |

### Öffentliche Funktionen (65)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn clear(&self)` | Clears all tracked request attention histories. | `src/attention_exporter.rs` |
| `pub fn new() -> Self` | Creates a new `CandleAttentionExporter`. | `src/attention_exporter.rs` |
| `pub fn record_attention_weights(&self, request_id: RequestId, weights: Vec<f32>)` | Records pre-summed attention weights for a request ID. Enforces [`MAX_TRACKED_REQUESTS`] ring-buffer capacity limit. | `src/attention_exporter.rs` |
| `pub fn record_layer_head_scores(&self, request_id: RequestId, scores: &[Vec<f32>])` | Combines and records attention weights from layer/head score matrices/vectors. Sums weights across heads/layers per token position in $O(\t… | `src/attention_exporter.rs` |
| `pub fn remove_request(&self, request_id: RequestId)` | Removes attention weights for a specific request ID from tracked history. | `src/attention_exporter.rs` |
| `pub fn tracked_request_count(&self) -> usize` | Returns the current number of tracked requests in memory. | `src/attention_exporter.rs` |
| `pub fn fingerprint(&self) -> &ModelFingerprint` | Returns a reference to the model's fingerprint. | `src/embedding.rs` |
| `pub fn from_dir( model_dir: &std::path::Path, quantization: crate::model_registry::CandleQuantization, ) -> Result<Self>` | Loads a `CandleEmbedClient` from a model directory. | `src/embedding.rs` |
| `pub fn from_dir_with_concurrency( model_dir: &std::path::Path, quantization: crate::model_registry::CandleQuantization, max_concurrent: usize, ) -> Result<Self>` | Loads a `CandleEmbedClient` from a model directory with custom maximum concurrent operations limit. | `src/embedding.rs` |
| `pub fn load(weights_path: &Path, _config_path: &Path, device: &Device) -> Result<Self>` | Loads BERT model weights from a `.safetensors` file and configuration from `config.json`. | `src/embedding.rs` |
| `pub fn new( device: Device, model: Box<dyn CandleEmbedInner + Send>, fingerprint: ModelFingerprint, tokenizer: tokenizers::Tokenizer, ) -> Self` | Creates a new `CandleEmbedClient`. | `src/embedding.rs` |
| `pub fn with_max_concurrent_embeddings(mut self, limit: usize) -> Self` | Configures maximum concurrent embedding operations for backpressure control. | `src/embedding.rs` |
| `pub fn compute_raw_grounding_score( &self, response: &str, context_chunks: &[ContextChunk], ) -> Result<f32>` | Führt die tatsächliche Attributions- und Grounding-Analyse durch. Extrahierte Fakten / Zahlen / Entitäten in `response` werden mit den bere… | `src/gasp.rs` |
| `pub fn config(&self) -> &GaspConfig` | Gibt eine Referenz auf die aktuelle `GaspConfig` zurück. | `src/gasp.rs` |
| `pub fn new() -> Self` | Erstellt einen neuen `GaspValidator` mit Standard-Konfiguration. | `src/gasp.rs` |
| `pub fn observation_count(&self) -> usize` | Gibt die Anzahl der aktuell gesammelten Kalibrierungsbeobachtungen zurück. | `src/gasp.rs` |
| `pub fn record_external_feedback(&self, raw_score: f32, ground_truth_grounded: bool)` | Zeichnet ein externes Ground-Truth-Signal (z. B. aus Nutzer-Feedback oder manueller Überprüfung) zur Kalibrierung des GaspValidators auf. I… | `src/gasp.rs` |
| `pub fn refresh_config(&mut self, config: GaspConfig)` | Aktualisiert die Konfiguration des `GaspValidator` und invalidiert die Kalibrierung, falls sich der `ConfigFingerprint` geändert hat (INV-C… | `src/gasp.rs` |
| `pub fn set_threshold(&mut self, threshold: f32)` | Setzt den Schwellenwert für Abstention und aktualisiert den Fingerabdruck (INV-CAL-2). | `src/gasp.rs` |
| `pub fn threshold(&self) -> f32` | Gibt den aktuellen Schwellenwert zurück. | `src/gasp.rs` |
| `pub fn with_config(config: GaspConfig) -> Self` | Erstellt einen `GaspValidator` mit angegebener Konfiguration. | `src/gasp.rs` |
| `pub fn parse_gguf_metadata(model_path: &Path) -> Result<GgufMetadata, ContextraError>` | Inspects a GGUF model file and extracts its architecture and metadata header keys. Uses `candle_core::quantized::gguf_file` to parse contai… | `src/gguf_loader.rs` |
| `pub fn fingerprint(&self) -> &ModelFingerprint` | Returns a reference to the model's fingerprint. | `src/inference/core.rs` |
| `pub fn from_dir( model_dir: &std::path::Path, quantization: crate::model_registry::CandleQuantization, ) -> Result<Self>` | Loads a `CandleLlmClient` from a model directory. | `src/inference/core.rs` |
| `pub fn from_dir_with_prefix_store( model_dir: &std::path::Path, quantization: crate::model_registry::CandleQuantization, store: Arc<dyn contextra_ports::kv::KvPrefixStore>, ) -> Result<Self>` | Loads a `CandleLlmClient` from a model directory and attaches a `KvPrefixStore`. | `src/inference/core.rs` |
| `pub fn generate_with_prefix_store<'a>( &'a self, tenant: TenantId, segments: &'a [ContextSegment<'a>], ) -> BoxFuture<'a, Result<String>>` | Generates completion with stage B KV prefix reuse and tenant isolation. | `src/inference/core.rs` |
| `pub fn load(model_path: &Path, device: &Device) -> Result<Self>` | Loads GGUF quantized model weights from the specified file path. | `src/inference/core.rs` |
| `pub fn new( device: Device, model: Box<dyn CandleModelInner + Send>, fingerprint: ModelFingerprint, tokenizer: tokenizers::Tokenizer, ) -> Self` | Creates a new `CandleLlmClient` with default concurrency limits. | `src/inference/core.rs` |
| `pub fn prefill_count(&self) -> u64` | Returns the total count of segment full prefill calculations performed. | `src/inference/core.rs` |
| `pub fn prefill_skip_count(&self) -> u64` | Returns the total count of segment prefill skips via KV cache hits. | `src/inference/core.rs` |
| `pub fn prefill_skipped_tokens(&self) -> u64` | Returns the total count of prompt tokens skipped via KV prefix reuse. | `src/inference/core.rs` |
| `pub fn swap_model( &mut self, new_model: Box<dyn CandleModelInner + Send>, new_fingerprint: ModelFingerprint, validator: Option<&mut GaspValidator>, )` | Tauscht das zugrundeliegende Modell und dessen Fingerprint zur Laufzeit aus. Falls ein optionaler `GaspValidator` übergeben wird, wird dess… | `src/inference/core.rs` |
| `pub fn tracked_request_count(&self) -> usize` | Returns the current number of tracked requests in memory if an attention exporter is attached. | `src/inference/core.rs` |
| `pub fn with_attention_exporter( mut self, exporter: Arc<crate::attention_exporter::CandleAttentionExporter>, ) -> Self` | Configures optional `CandleAttentionExporter` for exporting attention weights during inference. Note: Accepts `Arc<CandleAttentionExporter>… | `src/inference/core.rs` |
| `pub fn with_id_gen(mut self, id_gen: Arc<dyn contextra_ports::IdGen>) -> Self` | Configures custom `IdGen` port for request ID allocation. | `src/inference/core.rs` |
| `pub fn with_kv_bridge(mut self, adapter: crate::KvBridgeAdapter) -> Self` | Configures optional `KvBridgeAdapter` for KV cache segment retrieval and injection. | `src/inference/core.rs` |
| `pub fn with_max_concurrent_inferences(mut self, limit: usize) -> Self` | Configures maximum concurrent inference operations for backpressure control. | `src/inference/core.rs` |
| `pub fn with_prefix_store(mut self, store: Arc<dyn contextra_ports::kv::KvPrefixStore>) -> Self` | — | `src/inference/core.rs` |
| `pub fn build_prefix_key( fingerprint: &ModelFingerprint, tokenizer: &tokenizers::Tokenizer, layout: KvLayout, rope: RopeConfig, ) -> Result<PrefixKey>` | Constructs a `PrefixKey` for the given model fingerprint, tokenizer, layout, and RoPE config. | `src/inference/prefix_reuse.rs` |
| `pub fn seed_from_hit(hit: &KvPrefixHit, tokens_len: usize) -> Result<PrefixSeed>` | Reconstructs a `PrefixSeed` from a `KvPrefixHit` and total prompt token count. Ensures at least one prompt token remains to be evaluated fo… | `src/inference/prefix_reuse.rs` |
| `pub fn consult_segment<'a>(&self, segment: &ContextSegment<'a>)` | Consults segment metadata and KV cache bridge state for a context segment. | `src/kv_bridge.rs` |
| `pub fn consultation_count(&self) -> u64` | Returns the number of segment consultations recorded. | `src/kv_bridge.rs` |
| `pub fn new(chunk_id: u64, fingerprint: ModelFingerprint, rope_offset: Option<usize>) -> Self` | — | `src/kv_bridge.rs` |
| `pub fn new(store: Arc<TenantIsolatedKvStore>, cipher: Arc<KvSegmentCipher>) -> Self` | Erstellt einen neuen Adapter mit gegebenem Store und Cipher. | `src/kv_bridge.rs` |
| `pub fn store_segment(&self, tenant: TenantId, key: KvCacheKey, plaintext_kv_bytes: Vec<u8>)` | Speichert ein KV-Segment im Cache. Fehler werden geloggt, nie propagiert. | `src/kv_bridge.rs` |
| `pub fn try_get_cached_segment(&self, tenant: TenantId, key: &KvCacheKey) -> Option<Vec<u8>>` | Versucht, ein gecachetes KV-Segment zu laden. Gibt `None` zurück bei Cache-Miss, Decrypt-Fehler, Fingerprint-Mismatch oder jedem anderen Fe… | `src/kv_bridge.rs` |
| `pub async fn try_get_cached_segment_async( &self, tenant: TenantId, key: &KvCacheKey, ) -> Option<Vec<u8>>` | Versucht asynchron, ein gecachetes KV-Segment erst im RAM (Tier 1) und bei Miss im LSM-Store (Tier 2) zu laden. | `src/kv_bridge.rs` |
| `pub async fn try_get_cached_segment_async_bytes( &self, tenant: TenantId, key: &KvCacheKey, ) -> Option<Bytes>` | Versucht asynchron, ein gecachetes KV-Segment zu laden und als `Bytes` Slice (Zero-Copy) zurückzugeben. | `src/kv_bridge.rs` |
| `pub fn with_lsm_fallback( store: Arc<TenantIsolatedKvStore>, lsm_store: Arc<dyn KvBridgeStorage>, cipher: Arc<KvSegmentCipher>, ) -> Self` | Erstellt einen neuen Adapter mit Zwei-Tier-Hierarchie (RAM Fast-Path + LSM-Spill Slow-Path). | `src/kv_bridge.rs` |
| `pub fn export_block(&self, range: Range<usize>) -> Result<KvBlock, ContextraError>` | Exports a sub-range of KV cache tensors across all layers into a `KvBlock`. | `src/kv_state.rs` |
| `pub fn import_block(&mut self, block: &KvBlock) -> Result<(), ContextraError>` | Imports a `KvBlock` and concatenates its layer tensors onto the current state. | `src/kv_state.rs` |
| `pub fn import_block_at(&mut self, block: &KvBlock, at: usize) -> Result<(), ContextraError>` | Imports a `KvBlock` at a specific position or appends it if `at == self.pos`. | `src/kv_state.rs` |
| `pub fn is_empty(&self) -> bool` | Returns `true` if there are no layer KV tensors or position is zero. | `src/kv_state.rs` |
| `pub fn layer_count(&self) -> usize` | Returns the number of transformer layers in this state. | `src/kv_state.rs` |
| `pub fn new(k: Tensor, v: Tensor) -> Self` | Creates a new `LayerKv` pair. | `src/kv_state.rs` |
| `pub fn new(layers: Vec<LayerKv>, pos: usize) -> Self` | Creates a new `KvState` wrapping layer-wise KV tensors and position tracking. | `src/kv_state.rs` |
| `pub fn pos(&self) -> usize` | Returns the active sequence length / position. | `src/kv_state.rs` |
| `pub fn truncate(&mut self, len: usize) -> Result<(), ContextraError>` | Truncates stored KV tensors to at most `len` tokens along the sequence dimension. | `src/kv_state.rs` |
| `pub fn clear_kv_cache(&mut self)` | Clears the KV cache across all layers. | `src/model/quantized_llama.rs` |
| `pub fn export_kv_state(&self) -> Result<KvState, ContextraError>` | Exports the current KV cache across all model layers into a `KvState`. | `src/model/quantized_llama.rs` |
| `pub fn forward(&mut self, x: &Tensor, index_pos: usize) -> Result<Tensor, ContextraError>` | Performs forward pass through token embeddings, attention layers, norms, and output projection. | `src/model/quantized_llama.rs` |
| `pub fn from_gguf<R: std::io::Seek + std::io::Read>( ct: gguf_file::Content, reader: &mut R, device: &Device, ) -> Result<Self, ContextraError>` | — | `src/model/quantized_llama.rs` |
| `pub fn import_kv_state(&mut self, state: &KvState) -> Result<(), ContextraError>` | Imports a `KvState` into all model layers. | `src/model/quantized_llama.rs` |
| `pub fn truncate_kv_cache(&mut self, pos: usize) -> Result<(), ContextraError>` | Truncates the cached KV tensors across all layers to `pos` tokens. | `src/model/quantized_llama.rs` |
| `pub fn compute_fingerprint( model_path: &Path, quantization: &CandleQuantization, ) -> Result<ModelFingerprint, ContextraError>` | Computes a unique fingerprint for a model file and quantization variant. Calculates SHA-256 over the weight blob concatenated with the quan… | `src/model_registry.rs` |


---

## `contextra-infer-ollama`

| Attribut | Wert |
|---|---|
| **Ring** | 2 — Ring 2 — External Adapters & Sandboxing (Leaf Engines) |
| **Status** | In Migration |
| **Zweck** | HTTP-Inferenz-Client für Ollama mit Contextual-Chunk-Prefixing. |
| **Unsafe-Code** | Nein (`#![forbid(unsafe_code)]`) |
| **Interne Abhängigkeiten** | `contextra-ports`, `contextra-rank`, `contextra-types` |
| **Externe Abhängigkeiten (Auswahl)** | `futures`, `futures-util`, `parking_lot`, `quick-xml`, `rand`, `regex`, `reqwest`, `serde`, `serde_json`, `tokio`, `tracing` |

### Modul- und Re-Export-Oberfläche (lib.rs)

```rust
pub mod context_prefixer;
pub mod importance;
pub mod model_info;
pub use client::{ build_rag_prompt, xml_escape, OllamaClient, OllamaConfig, DEFAULT_BASE_URL, DEFAULT_EMBED_MODEL, };
pub use context_prefixer::{ContextPrefixConfig, ContextPrefixEngine, ContextPrefixer};
pub use embedding::OllamaEmbedder;
pub use importance::{ parse_importance_score_response, score_importance, score_importance_batch, score_importance_with_calibrator, Confidence, ImportanceAssessment, };
pub use model_info::ModelInfo;
```

### Öffentliche Datentypen — Structs (9)

| Struct | Felder / Form | Beschreibung | Datei |
|---|---|---|---|
| `ContextPrefixConfig` | `{ /// Ollama-Modell für Prefix-Generierung (sollte schnell & klein sein). /// Empfehlung: "llama3.2:3b" oder "gemma2:2b" — nicht das Haupt-Chat-Modell. pub model… }` | Konfiguration für Kontext-Präfix-Generierung. | `src/context_prefixer.rs` |
| `ContextPrefixEngine` | `{ client: OllamaClient, config: ContextPrefixConfig, }` | Generiert LLM-basierte Kontext-Präfixe für Dokument-Chunks. # Verwendung ```no_run # async fn example() -> Result<(), contextra_types::ContextraError… | `src/context_prefixer.rs` |
| `ImportanceAssessment` | `{ /// The normalized importance score (0.0 to 1.0). pub score: ImportanceScore, /// Confidence indicator describing whether the score was parsed or defaulted. pu… }` | Result of an LLM importance evaluation containing the score, parse confidence status, model provenance, and optional calibrated confidence. | `src/importance.rs` |
| `ModelInfo` | `{ pub modelfile: Option<String>, pub parameter_size: Option<String>, pub quantization_level: Option<String>, }` | — | `src/model_info.rs` |
| `OllamaClient` | `{ pub(crate) config: OllamaConfig, pub(crate) client: reqwest::Client, pub(crate) rng: Arc<dyn Rng>, }` | HTTP client for interacting with a local Ollama instance. | `src/client/core.rs` |
| `OllamaConfig` | `{ /// Base URL for the Ollama instance (default: `http://localhost:11434`) pub base_url: String, /// Model name for embedding generation (default: `nomic-embed-t… }` | Configuration options for the Ollama HTTP client. | `src/client/config.rs` |
| `OllamaEmbedder` | `{ client: OllamaClient, model: String, concurrency: usize, expected_dimension: Option<usize>, }` | Implementation of `TextEmbeddingEngine` using Ollama's HTTP API. | `src/embedding.rs` |
| `OllamaQueryRewriter` | `{ client: OllamaClient, config: OllamaQueryRewriterConfig, }` | LLM-basierter Multi-Signal Query Rewriter über Ollama. | `src/query_rewriter.rs` |
| `OllamaQueryRewriterConfig` | `{ /// Ollama-Modell für Rewriting (z. B. "llama3.2:3b" oder "gemma2:2b"). pub model: String, /// Maximale Anzahl von vorherigen Suchergebnissen als Kontext im Pr… }` | Konfiguration für den `OllamaQueryRewriter`. | `src/query_rewriter.rs` |

### Öffentliche Datentypen — Enums (1)

| Enum | Varianten | Beschreibung | Datei |
|---|---|---|---|
| `Confidence` | `/// Score was successfully parsed from the LLM output. Parsed, /// LLM output was unparseable; score defaulted to fallback value (0.5). Unparseable,` | Confidence or parse status of an LLM importance rating. | `src/importance.rs` |

### Typ-Aliase (1)

| Alias | Definition | Beschreibung |
|---|---|---|
| `ContextPrefixer` | `type ContextPrefixer= ContextPrefixEngine` | Compatibility alias for ContextPrefixEngine. |

### Konstanten (5)

| Konstante | Typ | Beschreibung |
|---|---|---|
| `DEFAULT_BASE_URL` | `&str` | Standard Ollama base URL |
| `DEFAULT_EMBED_MODEL` | `&str` | Standard embedding model for SME usage (multilingual support for German) |
| `MAX_BATCH_SIZE` | `usize` | Maximum allowed batch size for batch operations to prevent request timeouts and memory exhaustion. Bounded to 512 to match `contextra_infer_onnx::MAX… |
| `MAX_RETRIES` | `u32` | Default maximum retry attempts for transient errors |
| `MAX_TEXT_BYTES` | `usize` | Maximum allowed text length in bytes (10 MB) to prevent OOM or DoS |

### Öffentliche Funktionen (55)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn base_url(&self) -> &str` | — | `src/client/core.rs` |
| `pub async fn chat_with_rag_streaming( &self, model: &str, user_query: &str, context: &str, mut on_token: impl FnMut(String) + Send, ) -> Result<String>` | Streams RAG chat response token by token via POST /api/chat | `src/client/core.rs` |
| `pub async fn check_availability(&self) -> Result<()>` | Health check verifying Ollama availability via GET /api/tags returning structured error diagnostics if offline/unreachable. | `src/client/core.rs` |
| `pub fn config(&self) -> &OllamaConfig` | — | `src/client/core.rs` |
| `pub async fn embed(&self, model: &str, text: &str) -> Result<Vec<f32>>` | Generates vector embedding with retry logic for transient failures. Retries up to `max_retries` (default: 3) times with exponential backoff… | `src/client/core.rs` |
| `pub async fn embed_batch(&self, model: &str, texts: &[&str]) -> Result<Vec<Vec<f32>>>` | Generiert Embeddings für mehrere Texte. Teilt Eingaben, die `MAX_BATCH_SIZE` überschreiten, automatisch in Sub-Batches auf, um Timeouts und… | `src/client/core.rs` |
| `pub async fn ensure_model_available(&self, model: &str) -> Result<()>` | Ensures that the specified model exists; returns `ContextraError::NotFound` with helpful instruction if missing. | `src/client/core.rs` |
| `pub async fn generate(&self, model: &str, prompt: &str) -> Result<String>` | Generates non-streaming text completion via POST /api/generate. | `src/client/core.rs` |
| `pub async fn generate_text(&self, model: &str, prompt: &str) -> Result<String>` | Sendet eine einfache Chat-Anfrage (non-streaming) und gibt die vollständige Antwort zurück. Verwendet für Kontextpräfix-Generierung in der… | `src/client/core.rs` |
| `pub async fn generate_text_stream<F, Fut>( &self, model: &str, prompt: &str, mut on_token: F, ) -> Result<String> where F: FnMut(String) -> Fut + Send, Fut: std::future::Future<Output = bool> + Send,` | Streams text completion token by token via POST /api/chat. | `src/client/core.rs` |
| `pub async fn is_available(&self) -> bool` | Health check verifying Ollama availability via GET /api/tags | `src/client/core.rs` |
| `pub async fn is_model_available(&self, model: &str) -> bool` | Verifies if a specific model is available in the Ollama instance. | `src/client/core.rs` |
| `pub async fn list_models(&self) -> Result<Vec<String>>` | List available models in Ollama via GET /api/tags | `src/client/core.rs` |
| `pub fn new(base_url: impl Into<String>) -> Self` | Creates a new `OllamaClient` with the specified base URL and default timeout config. | `src/client/core.rs` |
| `pub async fn try_embed(&self, model: &str, text: &str) -> Result<Vec<f32>>` | Single embed attempt via POST /api/embeddings (no retry). | `src/client/core.rs` |
| `pub async fn try_embed_batch(&self, model: &str, texts: &[&str]) -> Result<Vec<Vec<f32>>>` | — | `src/client/core.rs` |
| `pub async fn try_generate(&self, model: &str, prompt: &str) -> Result<String>` | Single generate attempt via POST /api/generate (no retry). | `src/client/core.rs` |
| `pub async fn try_generate_text(&self, model: &str, prompt: &str) -> Result<String>` | Single generate_text attempt via POST /api/chat. | `src/client/core.rs` |
| `pub fn with_config(config: OllamaConfig) -> Self` | Creates a new `OllamaClient` with custom configuration parameters. | `src/client/core.rs` |
| `pub fn with_defaults() -> Self` | — | `src/client/core.rs` |
| `pub fn with_rng(mut self, rng: Arc<dyn Rng>) -> Self` | Sets a custom random number generator for retry jitter computation. | `src/client/core.rs` |
| `pub fn classify_reqwest_error(e: reqwest::Error, base_url: &str, context: &str) -> ContextraError` | Classifies a `reqwest::Error` into structured `ContextraError` variants (`Io` for connection refused or timeouts, `Storage` otherwise). | `src/client/errors.rs` |
| `pub fn is_transient_error(e: &ContextraError) -> bool` | Helper to classify transient network errors for retry. Returns true only for transient network failures (I/O error, connection reset, timeo… | `src/client/errors.rs` |
| `pub fn is_transient_network_error(err: &reqwest::Error) -> bool` | Helper to check if a reqwest error is a transient network error (timeout or connection error). | `src/client/errors.rs` |
| `pub fn build_rag_prompt(system_context: &str, rag_context: &str, user_query: &str) -> String` | Halluzinationsprävention bei lokalen 7B-Modellen ohne Guard: Lokale Sprachmodelle neigen ohne explizite Verhaltensanweisungen dazu, bei feh… | `src/client/validation.rs` |
| `pub fn validate_batch_size(count: usize) -> Result<()>` | Validates that batch size does not exceed `MAX_BATCH_SIZE`. | `src/client/validation.rs` |
| `pub fn validate_model_name(name: &str) -> Result<()>` | Validates that model name does not contain invalid path traversal or whitespace control characters. | `src/client/validation.rs` |
| `pub fn validate_text_length(text: &str, field_name: &str) -> Result<()>` | Validates that input text length does not exceed `MAX_TEXT_BYTES`. | `src/client/validation.rs` |
| `pub fn xml_escape(input: &str) -> String` | Escapes XML special characters in string inputs to prevent tag injection. | `src/client/validation.rs` |
| `pub async fn generate_prefix( &self, full_document: &str, chunk_content: &str, ) -> Result<String, ContextraError>` | Generiert ein Kontext-Präfix für einen einzelnen Chunk. Der Prompt folgt dem Contextra-Contextual-Chunk-Prefixing-Schema: "Describe this ch… | `src/context_prefixer.rs` |
| `pub async fn generate_prefix_batch( &self, full_document: &str, chunks: &[&str], ) -> Vec<Result<String, ContextraError>>` | Generiert Präfixe für einen Batch von Chunks desselben Dokuments. Nutzt sequentielle Verarbeitung (Ollama ist single-threaded per default).… | `src/context_prefixer.rs` |
| `pub fn new(client: OllamaClient, config: ContextPrefixConfig) -> Self` | — | `src/context_prefixer.rs` |
| `pub fn truncate_chars(s: &str, max_chars: usize) -> String` | — | `src/context_prefixer.rs` |
| `pub fn truncate_prefix(s: &str, max_tokens: usize, max_chars: usize) -> String` | Truncates context prefix text while preserving word boundaries up to `max_tokens` (words) and hard character limit `max_chars`. | `src/context_prefixer.rs` |
| `pub fn config(&self) -> &OllamaConfig` | — | `src/embedding.rs` |
| `pub fn model(&self) -> &str` | — | `src/embedding.rs` |
| `pub fn new(base_url: impl Into<String>, model: impl Into<String>) -> Self` | — | `src/embedding.rs` |
| `pub fn with_concurrency(mut self, concurrency: usize) -> Self` | — | `src/embedding.rs` |
| `pub fn with_config(config: OllamaConfig) -> Self` | — | `src/embedding.rs` |
| `pub fn with_defaults() -> Self` | — | `src/embedding.rs` |
| `pub fn with_expected_dimension(mut self, dim: usize) -> Self` | — | `src/embedding.rs` |
| `pub fn is_parsed(&self) -> bool` | Returns `true` if the score was parsed successfully from the LLM output. | `src/importance.rs` |
| `pub fn new(score: ImportanceScore, confidence: Confidence) -> Self` | Creates a new `ImportanceAssessment` with default empty model ID and no calibration. | `src/importance.rs` |
| `pub fn parse_importance_score_response(raw_response: &str) -> ImportanceAssessment` | Parses an `ImportanceAssessment` from raw LLM output. Returns `Confidence::Parsed` on successful float extraction, or `Confidence::Unparsea… | `src/importance.rs` |
| `pub fn record_importance_outcome( calibrator: &Arc<Mutex<IsotonicCalibrator>>, raw_score: f32, was_useful: bool, )` | Records outcome feedback for an importance score into an `IsotonicCalibrator`. Intended for downstream retrieval feedback loops evaluating… | `src/importance.rs` |
| `pub async fn score_importance( client: &OllamaClient, chunk_text: &str, ) -> Result<ImportanceAssessment>` | — | `src/importance.rs` |
| `pub async fn score_importance_batch( client: &Arc<OllamaClient>, chunks: &[&str], calibrator: Option<&Arc<Mutex<IsotonicCalibrator>>>, max_concurrent: usize, ) -> Vec<ImportanceAssessment>` | Evaluates importance for multiple text chunks in parallel using tokio::spawn. Returns a `Vec<ImportanceAssessment>` in the same order as `c… | `src/importance.rs` |
| `pub async fn score_importance_with_calibrator( client: &OllamaClient, chunk_text: &str, calibrator: Option<&Arc<Mutex<IsotonicCalibrator>>>, ) -> Result<ImportanceAssessment>` | Evaluates importance of a text chunk with an optional shared `IsotonicCalibrator`. Populates `model_id` provenance from `client.config().mo… | `src/importance.rs` |
| `pub fn value(&self) -> f32` | Helper returning the raw `f32` importance score value. | `src/importance.rs` |
| `pub fn with_provenance( score: ImportanceScore, confidence: Confidence, model_id: impl Into<String>, calibrated_confidence: Option<f32>, ) -> Self` | Creates a new `ImportanceAssessment` with model provenance and optional calibrated confidence. | `src/importance.rs` |
| `pub fn known_dimension(model: &str) -> Option<usize>` | Gibt die Embedding-Dimension für bekannte Modelle zurück. Gibt `None` zurück wenn das Modell unbekannt ist. | `src/model_info.rs` |
| `pub async fn show_model(&self, model: &str) -> Result<ModelInfo>` | Retrieves model info via POST /api/show | `src/model_info.rs` |
| `pub async fn validate_model_available(&self, model: &str) -> Result<()>` | Validates that the model exists in Ollama via GET /api/tags before invoking embeddings/chat. | `src/model_info.rs` |
| `pub fn config(&self) -> &OllamaQueryRewriterConfig` | Gibt eine Referenz auf die Konfiguration zurück. | `src/query_rewriter.rs` |
| `pub fn new(client: OllamaClient, config: OllamaQueryRewriterConfig) -> Self` | Erstellt einen neuen `OllamaQueryRewriter` mit dem angegebenen Client und der Konfiguration. | `src/query_rewriter.rs` |


---

## `contextra-infer-onnx`

| Attribut | Wert |
|---|---|
| **Ring** | 2 — Ring 2 — External Adapters & Sandboxing (Leaf Engines) |
| **Status** | In Migration |
| **Zweck** | ONNX-Embeddings und Reranking via ort. |
| **Unsafe-Code** | Nein (`#![forbid(unsafe_code)]`) |
| **Interne Abhängigkeiten** | `contextra-infer-candle`, `contextra-ports`, `contextra-rank`, `contextra-types` |
| **Externe Abhängigkeiten (Auswahl)** | `parking_lot`, `serde`, `tokio`, `tracing` |

### Modul- und Re-Export-Oberfläche (lib.rs)

```rust
pub mod reranker;
pub use reranker::{CrossEncoderReranker, PlattScaledSigmoid, RerankConfig, RerankResult};
```

### Feature-Flags

| Flag | Aktiviert |
|---|---|
| `default` | `[]` |
| `reranking` | `["dep:ort"]` |
| `onnx` | `["dep:ort", "dep:tokenizers", "dep:ndarray", "dep:reqwest"]` |
| `candle-backend` | `["dep:contextra-infer-candle"]` |

### Öffentliche Datentypen — Structs (4)

| Struct | Felder / Form | Beschreibung | Datei |
|---|---|---|---|
| `CrossEncoderReranker` | `{ pub(super) _config: RerankConfig, backend: RerankerBackend, /// Online-Kalibrierungshistorie für Platt-Fitting. /// Tuple: (raw_logit, is_relevant: bool) pub(s… }` | Unified public `CrossEncoderReranker` struct independent of active feature flags. | `src/reranker/cross_encoder.rs` |
| `RerankConfig` | `{ /// Pfad zur ONNX-Modelldatei (bge-reranker-base.onnx) pub model_path: std::path::PathBuf, /// Pfad zur Tokenizer-Konfigurationsdatei (tokenizer.json) pub toke… }` | Konfiguration für Cross-Encoder Reranking. | `src/reranker/config.rs` |
| `TextEmbedder` | `{ /// Shared ONNX session (thread-safe via `Arc<Mutex>`). session: Arc<parking_lot::Mutex<ort::session::Session>>, /// Shared tokenizer instance (thread-safe via… }` | Handles text tokenization and ONNX model inference. Uses `tokio::task::spawn_blocking` to offload ONNX inference to a blocking thread pool, preventin… | `src/lib.rs` |
| `TextEmbedderConfig` | `{ /// Maximum number of tokens per text sequence (default: 512). pub max_sequence_length: usize, /// Maximum parallel ONNX inference threads (default: 1). /// Va… }` | Configuration settings for the text embedder. | `src/lib.rs` |

### Typ-Aliase (1)

| Alias | Definition | Beschreibung |
|---|---|---|
| `OnnxEmbedder` | `type OnnxEmbedder= TextEmbedder` | — |

### Konstanten (2)

| Konstante | Typ | Beschreibung |
|---|---|---|
| `MAX_CANDIDATES` | `usize` | Maximale Anzahl von Kandidaten pro Reranking-Aufruf zur Vermeidung unbegrenzter Allokationen. |
| `MAX_EMBED_BATCH_SIZE` | `usize` | Conservative default. Override via `TextEmbedderConfig::max_batch_size`. At 1536D × 512 × f32 = ~3 MB input tensor; safe within 128 MB memory budgets. |

### Öffentliche Funktionen (25)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn default_model_cache_dir(model_name: &str) -> PathBuf` | Returns the default local cache directory for ONNX embedding models (`~/.contextra/models/<model_name>`). | `src/lib.rs` |
| `pub async fn embed_async(&self, text: &str) -> Result<Vec<f32>>` | Generates an embedding for the given text using `spawn_blocking`. Acquires a semaphore permit to limit concurrent inference operations, the… | `src/lib.rs` |
| `pub async fn ensure_onnx_model_download( model_name: &str, cache_dir: Option<&Path>, ) -> Result<PathBuf>` | Ensures the ONNX embedding model files (`model.onnx` and `tokenizer.json`) exist locally in `cache_dir` (or `~/.contextra/models/<model_nam… | `src/lib.rs` |
| `pub async fn ensure_onnx_model_download( _model_name: &str, _cache_dir: Option<&Path>, ) -> contextra_types::Result<PathBuf>` | — | `src/lib.rs` |
| `pub fn from_path(model_path: impl AsRef<Path>) -> Result<Self>` | Creates a new embedder from a model file or model directory path. | `src/lib.rs` |
| `pub fn load(model_dir: impl AsRef<Path>) -> Result<Self>` | Creates a new embedder by loading a tokenizer and ONNX model path from the specified directory. The directory should contain `model.onnx` a… | `src/lib.rs` |
| `pub fn load_with_config( model_dir: impl AsRef<Path>, config: TextEmbedderConfig, ) -> Result<Self>` | Creates a new embedder with a custom configuration. | `src/lib.rs` |
| `pub fn new(model_dir: impl AsRef<Path>) -> Result<Self>` | Creates a new embedder from a model directory. Alias for `load`. | `src/lib.rs` |
| `pub fn session_load_count(&self) -> usize` | Returns the number of times an ONNX session has been loaded by this embedder instance. | `src/lib.rs` |
| `pub fn validate(&self) -> Result<()>` | Validates configuration parameters. | `src/lib.rs` |
| `pub fn with_expected_dimension(mut self, dim: usize) -> Self` | Sets the expected embedding output dimension for post-inference validation. | `src/lib.rs` |
| `pub fn with_calibration(mut self, calibration: PlattScaler) -> Self` | Setzt ein gefittetes `PlattScaler` Modell für die Logit-Kalibrierung. | `src/reranker/config.rs` |
| `pub fn calibrate(&self, raw_logit: f32) -> f32` | Appliziert aktuell gefittete Kalibrierung auf rohen Logit-Score. | `src/reranker/cross_encoder.rs` |
| `pub fn calibration_observation_count(&self) -> usize` | Returns number of observations recorded. | `src/reranker/cross_encoder.rs` |
| `pub fn config(&self) -> &RerankConfig` | Returns a reference to the active `RerankConfig`. | `src/reranker/cross_encoder.rs` |
| `pub fn fitted_calibration(&self) -> PlattScaler` | Gibt die aktuell aktiv gefittete Kalibrierung zurück. | `src/reranker/cross_encoder.rs` |
| `pub fn invalidate_calibration(&self, new_fingerprint: ConfigFingerprint)` | Invalidiert Kalibrierung bei Modellwechsel (INV-CAL-2). | `src/reranker/cross_encoder.rs` |
| `pub fn is_calibrated(&self) -> bool` | Returns true if PlattScaler has been fitted (warmup completed). | `src/reranker/cross_encoder.rs` |
| `pub fn new(config: RerankConfig) -> Result<Self, ContextraError>` | Erstellt einen neuen CrossEncoderReranker. | `src/reranker/cross_encoder.rs` |
| `pub fn passthrough() -> Self` | Erstellt einen Passthrough-CrossEncoderReranker (für Benchmarks/Tests ohne ONNX-Modelldatei). | `src/reranker/cross_encoder.rs` |
| `pub fn passthrough_with_config(config: RerankConfig) -> Self` | Erstellt einen Passthrough-CrossEncoderReranker mit einer benutzerdefinierten Konfiguration. | `src/reranker/cross_encoder.rs` |
| `pub fn record_implicit_feedback(&self, results: &[RerankResult], implicit_relevant_k: usize)` | Records implicit feedback: top `implicit_relevant_k` results are marked as relevant, the rest as not relevant. Call this AFTER every succes… | `src/reranker/cross_encoder.rs` |
| `pub fn record_outcome(&self, raw_logit: f32, is_relevant: bool)` | Nimmt Feedback-Signal auf (raw logit + Relevanz-Label). Triggert Re-Fit wenn Warmup erreicht. | `src/reranker/cross_encoder.rs` |
| `pub async fn rerank( &self, _query: &str, candidates: &[String], ) -> Result<Vec<RerankResult>, ContextraError>` | Rerankt Kandidaten für eine Abfrage. | `src/reranker/cross_encoder.rs` |
| `pub fn with_calibration(mut self, calibration: PlattScaler) -> Self` | Setzt ein gefittetes `PlattScaler` Modell für den CrossEncoderReranker. | `src/reranker/cross_encoder.rs` |


---

## `contextra-sandbox`

| Attribut | Wert |
|---|---|
| **Ring** | 2 — Ring 2 — External Adapters & Sandboxing (Leaf Engines) |
| **Status** | Fertig |
| **Zweck** | WASM-Ausführungs-Isolation mit Fuel- und Wall-Clock-Limits via Wasmtime. |
| **Unsafe-Code** | Nein (`#![forbid(unsafe_code)]`) |
| **Interne Abhängigkeiten** | — |
| **Externe Abhängigkeiten (Auswahl)** | `anyhow`, `thiserror`, `tokio`, `tracing`, `wasmtime`, `zeroize` |

### Modul- und Re-Export-Oberfläche (lib.rs)

```rust
pub mod approval;
pub mod capabilities;
pub mod error;
pub mod executor;
pub mod merge;
pub mod output;
pub mod wasi;
pub use approval::{ classify_risk, ApprovalRequest, ApprovalRisk, ApprovalStatus, ApprovalTransitionError, };
pub use capabilities::{MergeOperatorCapabilities, WasmCapabilities};
pub use error::SandboxError;
pub use executor::WasmExecutor;
pub use merge::WasmMergeFunction;
pub use output::WasmOutput;
```

### Öffentliche Datentypen — Structs (9)

| Struct | Felder / Form | Beschreibung | Datei |
|---|---|---|---|
| `ApprovalRequest` | `{ /// Unique identifier for the approval request. pub request_id: String, /// Classification risk level. pub risk: ApprovalRisk, /// Human-readable summary of ca… }` | Human approval request state machine for a WASM execution run. | `src/approval.rs` |
| `CapabilityViolationError` | `{ pub capability: &'static str, }` | Custom error type returned by host functions on capability violation (INV-SBX-3). | `src/executor.rs` |
| `MergeOperatorCapabilities` | `(tuple/unit)` | Dedicated pure capabilities preset for I/O-free WASM Merge Operators (§4.18). | `src/capabilities.rs` |
| `OutputLimitExceededError` | `{ pub stream: &'static str, pub limit: usize, }` | Error thrown when stdout or stderr exceeds `max_output_bytes`. | `src/wasi.rs` |
| `ProcessExitError` | `{ pub code: i32, }` | Error thrown on WASI `proc_exit`. | `src/wasi.rs` |
| `WasmCapabilities` | `{ /// Stdout-Ausgabe erlaubt. Default: true. pub allow_stdout: bool, /// Stderr-Ausgabe erlaubt. Default: false (kein Logging-Leak). pub allow_stderr: bool, ///… }` | Whitelist für WASM-Guest-Capabilities. # Sicherheitsmodell Kein Dateisystem- und Netzwerkzugriff per Default. Monotone Uhr ist erlaubt (deterministis… | `src/capabilities.rs` |
| `WasmExecutor` | `{ engine: Engine, }` | WASM-Ausführungsgrenze für die `CodeExecution`-Permission. # Designprinzip (§4.18) - Frische `Store` + `Instance` pro execute()-Aufruf: kein Zustands… | `src/executor.rs` |
| `WasmMergeFunction` | `{ executor: WasmExecutor, wasm_bytes: Arc<[u8]>, caps: WasmCapabilities, }` | Standalone, pure, deterministic WASM merge operator executor. Encapsulates a pre-validated WASM module and executes pure merge logic over binary inpu… | `src/merge.rs` |
| `WasmOutput` | `{ /// Stdout des WASM-Guests — sensitiv, ZeroizeOnDrop. #[zeroize(skip)] // Zeroizing<Vec<u8>> implements Drop zeroization itself pub stdout: Zeroizing<Vec<u8>>,… }` | Ausgabe einer WASM-Execution. # Sicherheits-Invariante (P9) `stdout` ist `ZeroizeOnDrop` — der Speicher wird beim Drop überschrieben. Sensitive WASM-… | `src/output.rs` |

### Öffentliche Datentypen — Enums (4)

| Enum | Varianten | Beschreibung | Datei |
|---|---|---|---|
| `ApprovalRisk` | `/// Low risk: Execution within standard safe defaults (no filesystem/network/cloud egress). Low, /// Elevated risk: File system access requested without network or egress. Elevated, /// High risk: Ne…` | Risk level classification for a requested WASM execution configuration. | `src/approval.rs` |
| `ApprovalStatus` | `/// Pending decision. Pending, /// Approved by an authorized entity. Approved { approved_by: String, approved_at_unix_ms: u64, }, /// Rejected with an explicit reason. Rejected { rejected_by: String,…` | Status of a human approval decision. | `src/approval.rs` |
| `ApprovalTransitionError` | `/// Transition rejected because the approval request has expired. #[error( "Approval request expired (created_at: {created_at_unix_ms}ms, ttl: {ttl_ms}ms, evaluated_at: {now_unix_ms}ms)" )] Expired {…` | Errors occurring during approval state transitions. | `src/approval.rs` |
| `SandboxError` | `#[error("WASM-Execution timed out after {timeout_ms}ms")] Timeout { timeout_ms: u64 }, #[error("WASM module exceeded memory limit ({pages} pages, max {max_pages})")] MemoryExceeded { pages: u32, max_…` | Fehlertypen der WASM-Ausführungsgrenze. | `src/error.rs` |

### Öffentliche Funktionen (18)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn approve( mut self, approved_by: String, now_unix_ms: u64, ) -> Result<Self, ApprovalTransitionError>` | Transitions request from `Pending` to `Approved`. Fails with `ApprovalTransitionError` if not in `Pending` state or if expired at `now_unix… | `src/approval.rs` |
| `pub fn auto_approve_if_low_risk( self, system_identity: String, now_unix_ms: u64, ) -> Result<Self, ApprovalTransitionError>` | Automatically approves the request if it is low-risk (`!self.requires_approval()`). If human approval is required (`self.requires_approval(… | `src/approval.rs` |
| `pub fn classify_risk(caps: &WasmCapabilities) -> ApprovalRisk` | Derives the execution risk level deterministically from `WasmCapabilities`. # Rule Matrix | Capability Condition | Derived Risk Level | Rat… | `src/approval.rs` |
| `pub fn expire(mut self, now_unix_ms: u64) -> Result<Self, ApprovalTransitionError>` | Explicitly transitions request from `Pending` to `Expired`. | `src/approval.rs` |
| `pub fn is_expired(&self, now_unix_ms: u64) -> bool` | Checks if the request is expired relative to `now_unix_ms`. Uses `saturating_add` to prevent overflow panic on boundary values. | `src/approval.rs` |
| `pub fn new( request_id: String, caps: &WasmCapabilities, created_at_unix_ms: u64, ttl_ms: u64, ) -> Self` | Creates a new approval request in `Pending` status. Computes `risk` via `classify_risk` and generates `requested_capabilities_summary` from… | `src/approval.rs` |
| `pub fn reject( mut self, rejected_by: String, reason: String, now_unix_ms: u64, ) -> Result<Self, ApprovalTransitionError>` | Transitions request from `Pending` to `Rejected`. Fails with `ApprovalTransitionError` if not in `Pending` state or if expired at `now_unix… | `src/approval.rs` |
| `pub fn requires_approval(&self) -> bool` | Determines whether this execution requires human approval before proceeding. # Product Rationale Low-risk executions (default sandboxed cap… | `src/approval.rs` |
| `pub fn pure() -> WasmCapabilities` | Returns a strict `WasmCapabilities` preset for pure merge operators. | `src/capabilities.rs` |
| `pub fn pure_merge_operator() -> Self` | Returns a strict `WasmCapabilities` preset for pure merge operators. # Security Profile - `allow_stdout`: `false` - `allow_stderr`: `false`… | `src/capabilities.rs` |
| `pub fn pure_with_result_channel() -> WasmCapabilities` | Returns a strict `WasmCapabilities` preset for merge operators with stdout enabled for returning output bytes. Differs from `pure()` strict… | `src/capabilities.rs` |
| `pub async fn execute( &self, wasm_bytes: &[u8], input: &[u8], capabilities: &WasmCapabilities, timeout: Duration, ) -> Result<WasmOutput, SandboxError>` | Führt ein WASM-Binary mit Capability-Einschränkungen aus. # Garantien - Fuel-Budget: CPU-Limit (deterministisch), verhindert Endlosschleife… | `src/executor.rs` |
| `pub fn new() -> Result<Self, SandboxError>` | Erstellt einen neuen WasmExecutor. # Errors Wenn wasmtime::Engine nicht initialisiert werden kann. | `src/executor.rs` |
| `pub fn validate_module( &self, wasm_bytes: &[u8], max_module_size_bytes: usize, ) -> Result<(), SandboxError>` | Validates a WASM binary without executing it. Checks binary size limits and attempts module compilation. | `src/executor.rs` |
| `pub async fn merge(&self, existing: &[u8], new: &[u8]) -> Result<Vec<u8>, SandboxError>` | Merges two binary values (`existing` and `new`) deterministically using the WASM module. # Input Encoding Binary stream passed to guest std… | `src/merge.rs` |
| `pub fn new(wasm_bytes: impl Into<Arc<[u8]>>) -> Result<Self, SandboxError>` | Creates a new `WasmMergeFunction` and validates the WASM binary once upon creation. Uses default pure merge capabilities (`MergeOperatorCap… | `src/merge.rs` |
| `pub fn new_with_capabilities( wasm_bytes: impl Into<Arc<[u8]>>, caps: WasmCapabilities, ) -> Result<Self, SandboxError>` | Creates a new `WasmMergeFunction` with custom capability limits (e.g. `max_fuel` or `max_wall_clock_ms`). | `src/merge.rs` |
| `pub fn new(stdout: Vec<u8>, stderr: Vec<u8>, fuel_consumed: u64) -> Self` | — | `src/output.rs` |


---

## `contextra-engine`

| Attribut | Wert |
|---|---|
| **Ring** | 3 — Ring 3 — Orchestration & Cognition |
| **Status** | In Migration |
| **Zweck** | Collection-LSM-Anbindung, Multi-Index-Pläne und Schreibtransaktionen. |
| **Unsafe-Code** | Nein (`#![forbid(unsafe_code)]`) |
| **Interne Abhängigkeiten** | `contextra-adapt`, `contextra-checkpoint`, `contextra-crypto`, `contextra-graph`, `contextra-infer-candle`, `contextra-kvcache`, `contextra-mvcc`, `contextra-ports`, `contextra-rank`, `contextra-sandbox`, `contextra-store`, `contextra-sys`, `contextra-text`, `contextra-types`, `contextra-vector` |
| **Externe Abhängigkeiten (Auswahl)** | `ahash`, `bincode`, `blake3`, `bytes`, `hmac`, `parking_lot`, `serde`, `serde_json`, `sha2`, `thiserror`, `tokio`, `tokio-util`, `tracing`, `zeroize` |

### Modul- und Re-Export-Oberfläche (lib.rs)

```rust
pub mod background_workers;
pub mod chunker;
pub mod collection;
pub mod decay_controller;
pub mod export;
pub mod extraction;
pub mod filter;
pub mod fusion;
pub mod import;
pub mod kv_cache_integration;
pub mod metrics_names;
pub mod temporal_filter;
pub mod transaction;
pub mod merge_operator_adapter;
pub use contextra_crypto::deletion_proof::{ DeletionLayer, DeletionProof, DeletionScope, LayerCleanupProof, };
pub use no_crypto_stubs::*;
pub use contextra_ports::TextEmbeddingEngine;
pub use contextra_ports::VectorDeleteMode;
pub use contextra_store::lsm::DurabilityMode;
pub use kv_cache_integration::build_eviction_worker;
pub use merge_operator_adapter::WasmMergeOperator;
pub use decay_controller::{AdaptiveDecayController, DecayControllerConfig, DecaySignalInputs};
pub use export::{ ExportCollectionV1, ExportDocumentV1, ExportMemoryV1, ExportRelationV1, SCHEMA_VERSION_V1, };
pub use import::ImportSummary;
pub use background_workers::start_decay_cleanup_worker;
pub use background_workers::{start_expiry_cleanup_worker, start_orphan_cleanup_worker};
pub use collection::crud::MAX_SCAN_RESULTS;
pub use collection::maintenance::PercolationResult;
pub use collection::query_builder::{HybridQueryBuilder, SearchStrategy, SignalWeights};
pub use collection::{Collection, CollectionConfig};
pub use contextra_checkpoint;
pub use contextra_graph::percolation::PercolationConfig;
pub use contextra_text::Language;
pub use filter::MetadataFilter;
pub use fusion::{ProvenanceRecord, SearchResult, SignalContribution};
pub use contextra_ports::DriftStatusProvider;
pub use contextra_types::DistanceMetric;
pub use serde_json::json;
```

### Feature-Flags

| Flag | Aktiviert |
|---|---|
| `default` | `[]` |
| `onnx` | `[]` |
| `reranking` | `[]` |
| `encryption-at-rest` | `["dep:contextra-crypto", "contextra-store/encryption-at-rest"]` |
| `bench` | `[]` |
| `sandbox` | `["dep:contextra-sandbox"]` |
| `experimental-diskann` | `["contextra-vector/experimental-diskann"]` |
| `background-maintenance` | `[]` |
| `graph-connectivity-health` | `["contextra-graph/physio-percolation"]` |
| `edge-reinforcement-learning` | `["contextra-graph/edge-reinforcement-learning"]` |
| `coherence-bonus-fusion` | `[]` |
| `adaptive-candidate-pool-sizing` | `[]` |
| `entity-extraction` | `[]` |
| `auto-extraction-opt-out` | `[]` |

### Schnittstellen — Public Traits (4)

**`CompensatingAction`** — *Trait representing an undo operation to be executed during transaction rollback/compensation.* (`src/transaction/compensating_actions.rs`)

```rust
pub trait CompensatingAction : Send + Sync {
fn execute<'a>(&'a self) -> BoxFuture<'a, Result<()>>;
}
```

**`EntityExtractor`** — *Gemeinsamer Trait für alle Entitätsextraktoren in Contextra-Engine.* (`src/extraction/mod.rs`)

```rust
pub trait EntityExtractor : Send + Sync {
/// Extrahiert Wissens-Tripel aus dem übergebenen Freitext.
    fn extract_triples<'a>(
        &'a self,
        text: &'a str,
        generator: Option<&'a dyn LlmTextGenerator>,
        config: &'a EntityExtractionConfig,
    ) -> BoxFuture<'a, Result<Vec<ExtractedTriple>>>;
}
```

**`OrphanCleanupIndex`** — *Abstraction trait over vector indexes capable of connectivity check and rebuild.* (`src/background_workers/config.rs`)

```rust
pub trait OrphanCleanupIndex : Send + Sync {
fn check_connectivity(&self) -> contextra_types::Result<()>;
    fn rebuild(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = contextra_types::Result<()>> + Send + '_>>;
}
```

**`SandboxBridge`** — ** (`src/lib.rs`)

```rust
pub trait SandboxBridge : Send + Sync {
fn db_search<'a>(
        &'a self,
        query: &'a [u8],
        k: usize,
    ) -> BoxFuture<'a, contextra_types::Result<Vec<u8>>>;
    fn db_insert<'a>(
        &'a self,
        key: &'a [u8],
        value: &'a [u8],
    ) -> BoxFuture<'a, contextra_types::Result<()>>;
    fn db_get<'a>(
        &'a self,
        key: &'a [u8],
    ) -> BoxFuture<'a, contextra_types::Result<Option<Vec<u8>>>>;
}
```

### Öffentliche Datentypen — Structs (38)

| Struct | Felder / Form | Beschreibung | Datei |
|---|---|---|---|
| `AccessCounterAttentionExporter` | `{ access_counts: RwLock<HashMap<RequestId, u64>>, }` | A minimal fallback adapter tracking request access counts. Honestly marked as initial iteration fallback providing real access counters rather than c… | `src/kv_cache_integration/mod.rs` |
| `AutoExtractionConfig` | `{ /// Laufzeit-Modus fuer die automatische Extraktion (Default: Enabled, bzw. Disabled wenn Feature `auto-extraction-opt-out` aktiv ist). #[serde(default)] pub m… }` | Konfiguration fuer die automatische Entitaetsextraktion beim Einfuegen von Dokumenten. | `src/collection/crud/auto_extraction.rs` |
| `CheckpointPinGuard` | `{ storage: &'a S, seq: u64, unpinned: bool, }` | RAII Guard for pinning snapshot checkpoints during search/scan operations. # Note on async Drop Rust does not support native `async Drop`. Therefore,… | `src/collection/search/checkpoint.rs` |
| `ChunkerConfig` | `{ /// Maximum tokens per chunk (soft limit, hard limit is this * 1.2). /// Default: 512 tokens (aligned with standard dense embedding models). pub max_tokens: us… }` | Configuration for the Markdown chunker. | `src/chunker.rs` |
| `Collection` | `{ pub(super) name: String, pub(super) prefix: Vec<u8>, pub(super) index: Arc<V>, pub(super) text_index: InvertedIndex<S>, pub(super) graph_index: Arc<CsrGraph>,… }` | — | `src/collection/mod.rs` |
| `CollectionConfig` | `{ /// Configurable backpressure delay in milliseconds when system pressure is Critical. /// Default: Some(50) pub backpressure_delay_ms: Option<u64>, }` | Configuration for a collection. | `src/collection/mod.rs` |
| `CommitLedger` | `{ actions: Vec<Box<dyn CompensatingAction>>, }` | A ledger of compensating actions collected during 2PC, executed in reverse order (LIFO) on failure. | `src/transaction/compensating_actions.rs` |
| `CommunityDetectionConfig` | `{ pub auto_trigger_threshold: u64, }` | — | `src/lib.rs` |
| `CompensateLsmAction` | `{ pub(super) collection: Collection<S, V>, pub(super) intent_key: Vec<u8>, pub(super) doc_ids: Arc<Vec<DocId>>, pub(super) f_keys: Vec<StagedKeyOp>, pub(super) r… }` | — | `src/transaction/compensating_actions.rs` |
| `Contextra` | `{ storage: Arc<LsmStorage>, next_tx: Arc<AtomicU64>, dimension: usize, config: ContextraConfig, license_gate: Arc<dyn LicenseGate>, expiry_reaper_interval: std::… }` | — | `src/lib.rs` |
| `ContextraConfig` | `{ pub dimension: usize, pub max_elements: usize, pub distance_metric: contextra_types::DistanceMetric, pub encryption_passphrase: Option<String>, /// Maximum RAM… }` | — | `src/lib.rs` |
| `ContextraStats` | `{ pub drift_status: String, #[serde(default, skip_serializing_if = "Option::is_none")] pub calibration_ece: Option<f32>, #[serde(default, skip_serializing_if = "… }` | — | `src/lib.rs` |
| `DbTransaction` | `{ pub tx_id: TxId, pub(super) collection: Collection<S, V>, pub(super) staged_forward_keys: Mutex<Vec<StagedKeyOp>>, pub(super) staged_reverse_keys: Mutex<Vec<St… }` | A transaction wrapper that ensures atomic multi-index commits across LSM-Store, HNSW-Index, Text-Index, and Graph-Index. | `src/transaction/db_transaction.rs` |
| `DeferredHyperedgeQueue` | `{ inner: Mutex<VecDeque<HyperEdgeId>>, }` | Thread-safe FIFO queue for storing hyperedges whose cascade invalidation was deferred to the background worker due to fan-out limits (H5 / AK-6). | `src/background_workers/hyperedge_worker.rs` |
| `DeletionProof` | `{ #[serde(default = "default_signature_version")] pub signature_version: u8, pub scope: DeletionScope, #[serde(default)] pub deleted_keys_hash: [u8; 32], #[serde… }` | — | `src/lib.rs` |
| `Document` | `{ pub id: String, pub metadata: Option<Value>, }` | — | `src/lib.rs` |
| `EntityExtractionConfig` | `{ /// Ob Entitätsextraktion aktiviert ist pub enabled: bool, /// Maximale Anzahl von LLM-Aufrufen pro Zyklus / Extraktionsdurchlauf pub max_llm_calls_per_cycle:… }` | Konfiguration für die automatische Entitätsextraktion. | `src/extraction/types.rs` |
| `EntityExtractionConfig` | `{ pub enabled: bool, pub max_llm_calls_per_cycle: usize, pub min_confidence: f32, }` | — | `src/collection/crud/auto_extraction.rs` |
| `ExportCollectionV1` | `{ /// Name der Collection pub name: String, /// Liste aller Memory-Einträge pub memories: Vec<ExportMemoryV1>, /// Liste aller direkten Graph-Relationen pub rela… }` | Export-Repräsentation einer einzelnen Collection. | `src/export.rs` |
| `ExportDocumentV1` | `{ /// Pflichtfeld für Schema-Migrationen (MUSS "1.0" sein) pub schema_version: String, /// UTC-Zeitstempel der Export-Erstellung (ISO-8601 String) #[serde(defaul… }` | Wurzel-Dokument für den Export aller DB-Collections. | `src/export.rs` |
| `ExportMemoryV1` | `{ /// Eindeutiger Memory-Schlüssel / ID pub id: String, /// Kognitiver Gedächtnistyp (episodic, semantic, procedural, working) #[serde(rename = "type")] pub memo… }` | Export-Repräsentation eines einzelnen Memory-Eintrags. | `src/export.rs` |
| `ExportRelationV1` | `{ /// Quell-Memory ID pub from: String, /// Ziel-Memory ID pub to: String, /// Beziehungs-Label (z.B. "references", "depends_on") pub label: String, }` | Export-Repräsentation einer Graph-Beziehung. | `src/export.rs` |
| `ExtractedTriple` | `{ /// Subjekt der Relation / Entität pub subject: String, /// Prädikat / Relation pub predicate: String, /// Objekt der Relation / Entität pub object: String, //… }` | Ein aus dem Text extrahiertes Wissens-Tripel (Subject, Predicate, Object). | `src/extraction/types.rs` |
| `HybridQueryBuilder` | `{ pub(super) collection: &'a Collection<S, V>, pub(super) text: Option<String>, pub(super) vector: Option<Vec<f32>>, pub(super) k: Option<usize>, pub(super) weig… }` | Fluent query builder for unifying vector, text, graph, and hybrid search operations. | `src/collection/query_builder/builder.rs` |
| `ImportSummary` | `{ /// Anzahl erfolgreich importierter oder aktualisierter Memory-Einträge pub imported_memories: usize, /// Anzahl übersprungener Memory-Einträge pub skipped_mem… }` | Zusammenfassung eines Import-Vorgangs. | `src/import.rs` |
| `LayerCleanupProof` | `{ pub layer: DeletionLayer, pub remaining_count: usize, }` | — | `src/lib.rs` |
| `MarkdownChunker` | `{ config: ChunkerConfig, }` | A deterministic chunker for markdown files respecting structural blocks (headings, code blocks, tables, paragraphs) and UTF-8 safety. | `src/chunker.rs` |
| `OpenIeExtractor` | `(tuple/unit)` | Implementierung des OpenIE-Extraktors hinter `EntityExtractor`. | `src/extraction/mod.rs` |
| `OrphanCleanupBackoffConfig` | `{ /// Initial cooldown duration after the first failed rebuild attempt. pub base_delay: Duration, /// Maximum cooldown cap between rebuild attempts. pub max_dela… }` | Configuration parameters for HNSW index rebuild backoff and failure escalation. | `src/background_workers/config.rs` |
| `PercolationResult` | `{ pub health: Option<f32>, pub rebonding_triggered: bool, pub new_edges_added: usize, }` | — | `src/collection/maintenance.rs` |
| `RuleBasedExtractor` | `(tuple/unit)` | Regelbasierter Entitäts- und Relationsextraktor. | `src/extraction/rule_based.rs` |
| `ScopeConstraint` | `{ /// Explicit set of allowed document IDs (BTreeSet guarantees deterministic ordering). pub allowed_doc_ids: BTreeSet<DocId>, /// Gamma augmentation factor for… }` | Scoping constraint defining an explicit hard-boundary set of allowed document IDs for predicate-agnostic vector search (ACORN-Hard-Boundary, Spec §5.… | `src/collection/query_builder/scope.rs` |
| `SignalWeights` | `{ /// Weight for vector similarity signal. pub vector: f32, /// Weight for BM25 text match signal. pub text: f32, /// Weight for graph traversal signal. pub grap… }` | Custom weights for vector, text, and graph signals in hybrid search. | `src/collection/query_builder/weights.rs` |
| `StoredDocument` | `{ pub id: String, pub embedding: Vec<f32>, pub metadata: Option<serde_json::Value>, }` | — | `src/collection/mod.rs` |
| `StoredDocumentMeta` | `{ pub id: String, pub metadata: Option<serde_json::Value>, }` | Leichtgewichtige Metadaten (für doc_key, key_type=1) — KEIN Embedding. Wird für DocId-basierte Hydration nach HNSW/BM25-Suche verwendet. | `src/collection/mod.rs` |
| `TenantIsolatedKvStore` | `(tuple/unit)` | — | `src/lib.rs` |
| `ValidityWindow` | `{ /// Start of transaction validity (system time / MVCC). pub tx_valid_from: Option<TxId>, /// End of transaction validity (system time / MVCC, exclusive upper b… }` | Extracted bi-temporal validity window representation for a document or search candidate. | `src/temporal_filter.rs` |
| `WasmMergeOperator` | `{ tx: Option<Sender<MergeJob>>, worker_handle: Option<JoinHandle<()>>, recv_timeout: Duration, }` | Synchronous wrapper around [`WasmMergeFunction`] implementing [`MergeOperator`]. | `src/merge_operator_adapter.rs` |

### Öffentliche Datentypen — Enums (10)

| Enum | Varianten | Beschreibung | Datei |
|---|---|---|---|
| `CommitIntent` | `/// Transaction is in the "Prepared" state. Stored DocIds assist recovery. /// `stages_completed` tracks progress: /// 0 = Intent written, /// 1 = Storage (LSM) committed, /// 2 = Indices (Vector, Te…` | Status of a multi-index transaction during the 2-phase commit. | `src/transaction/intent.rs` |
| `DeletionLayer` | `LsmMemtable, SsTableAllLevels, HnswIndex,` | — | `src/lib.rs` |
| `DeletionScope` | `Document { doc_id: contextra_types::DocId, tenant_id: TenantId, }, Collection { collection_id: CollectionId, tenant_id: TenantId, }, Tenant { tenant_id: TenantId, },` | — | `src/lib.rs` |
| `EmbeddingBackend` | `Onnx { model_name: String, cache_dir: Option<std::path::PathBuf>, }, None,` | — | `src/lib.rs` |
| `ExcludedScope` | `ConsolidatedAndDistilled, LlmParameterMemory,` | — | `src/lib.rs` |
| `ExtractorMode` | `OpenIe, RuleBased,` | — | `src/collection/crud/auto_extraction.rs` |
| `FilterOp` | `/// Equal to Eq, /// Not equal to Ne, /// Greater than Gt, /// Greater than or equal to Gte, /// Less than Lt, /// Less than or equal to Lte, /// In a set of values In, /// Not in a set of values Not…` | — | `src/filter.rs` |
| `MetadataFilter` | `/// A single condition on a metadata field. Condition { field: String, #[allow(deprecated)] op: FilterOp, value: Value, }, /// Logical AND of multiple filters. And(Vec<MetadataFilter>), /// Logical O…` | — | `src/filter.rs` |
| `SearchStrategy` | `/// Reciprocal Rank Fusion (default fusion strategy per ADR-003). Rrf, /// Score-normalized fusion (CombSUM / Z-Score). ScoreNormalized, /// Multi-hop BFS graph traversal strategy. Hops { /// Maximum…` | Strategy for hybrid search and graph signal traversal. | `src/collection/query_builder/strategy.rs` |
| `TenantPolicy` | `/// Non-tenant collection access (`collection(...)`) is allowed (Default / Admin / Single-tenant path). #[default] Optional, /// Non-tenant collection access is forbidden; callers MUST use tenant-sco…` | Tenant isolation enforcement policy. | `src/lib.rs` |

### Typ-Aliase (4)

| Alias | Definition | Beschreibung |
|---|---|---|
| `ConsolidationLauncher` | `type ConsolidationLauncher= Arc< dyn Fn( Arc<Collection<LsmStorage>>, std::time::Duration, usize, tokio_util::sync::CancellationToken, ) -> tokio::task::JoinHandle<()> + Send + Sync, >` | — |
| `DbStats` | `type DbStats= ContextraStats` | — |
| `FusionResult` | `type FusionResult= crate::SearchResult` | Type alias for search & fusion results in post-retrieval pipelines. |
| `TenantCollectionMap` | `type TenantCollectionMap= ahash::AHashMap< (TenantId, String), Arc<Collection<contextra_store::tenant_codec::TenantScopedStorage<Arc<LsmStorage>>>>, >` | — |

### Konstanten (24)

| Konstante | Typ | Beschreibung |
|---|---|---|
| `BACKPRESSURE_LEVEL` | `&str` | Metric name for system backpressure level gauge readings (0.0 = Normal, 1.0 = Elevated, 2.0 = Critical). |
| `CHECKPOINT_DURATION` | `&str` | Metric name for checkpoint operation duration histogram observations. |
| `COMMIT_LATENCY` | `&str` | Metric name for engine commit latency histogram observations. |
| `DECAY_DELETION_THRESHOLD` | `f32` | werden als "vergessen" markiert und gelöscht. |
| `DEFAULT_COMMUNITY_BOOST` | `f32` | Standard community boost factor applied to post-fusion scores for matching community members. |
| `DEFAULT_MIN_RERANK_CANDIDATES` | `usize` | Performs hybrid search combining BM25, vector search, and graph traversal, followed by optional Cross-Encoder reranking. Mindest-Kandidatenpool für C… |
| `DEFAULT_RERANK_POOL_MAX` | `usize` | Default upper cap for pre-reranking candidate pool expansion (200 candidates). Prevents retrieval cost explosion for queries with large `k`. |
| `DEFAULT_RERANK_POOL_MULTIPLIER` | `usize` | Default candidate pool expansion multiplier for Cross-Encoder reranking (10x requested k). Grounded in empirical evaluation from T2-RAGBench (arXiv:2… |
| `DEFAULT_SCAN_LIMIT` | `usize` | Harte Obergrenze für scan()/scan_prefix()-Ergebnisse, falls kein explizites `limit` übergeben wird. Verhindert unbeabsichtigten Vollscan bei generisc… |
| `HARD_SCAN_CEILING` | `usize` | Harte Obergrenze für den maximal erlaubten `limit`-Parameter in scan() und scan_prefix(). Verhindert unbegrenzte Materialisierung im Speicher selbst… |
| `MAX_DEFERRED_HYPEREDGES_PER_TICK` | `usize` | Maximum number of deferred hyperedges processed in a single worker tick. |
| `MAX_EXPIRED_PER_TICK` | `usize` | Maximum number of expired documents processed in a single expiry cleanup tick. |
| `MAX_EXTRACTED_TRIPLES` | `usize` | Maximale Anzahl extrahierter Tripel pro Durchlauf. |
| `MAX_EXTRACTION_TEXT_LENGTH` | `usize` | Maximale zulässige Eingabelänge in Zeichen für die regelbasierte Extraktion. |
| `MAX_ORPHANS_PER_TICK` | `usize` | Maximum number of orphan transactions processed in a single worker tick to avoid starving foreground operations. |
| `MAX_OVERFETCH_SCAN` | `usize` | Upper bound scan limit for adaptive candidate overfetching. |
| `MAX_SCAN_RESULTS` | `usize` | Alias for backwards compatibility with earlier MAX_SCAN_RESULTS references. |
| `OVERFETCH_FACTOR` | `usize` | Standard overfetch factor applied to candidate limits before fusion to balance OOM protection and recall. |
| `OVERFETCH_STAGE_MULTIPLIERS` | `[usize` | Named overfetch stage multipliers for adaptive candidate overfetching (Spec v17 §6.5). |
| `REBUILD_DURATION` | `&str` | Metric name for index rebuild duration histogram observations. |
| `SCHEMA_VERSION_V1` | `&str` | Aktuelle Schema-Version für das JSON Export-Format. |
| `SEARCH_LATENCY_GRAPH` | `&str` | Reserved metric name for graph signal search latency observations. |
| `SEARCH_LATENCY_TEXT` | `&str` | Reserved metric name for text signal search latency observations. |
| `SEARCH_LATENCY_VECTOR` | `&str` | Reserved metric name for vector signal search latency observations. |

### Öffentliche Funktionen (319)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn calculate_rebuild_cooldown( consecutive_failures: u32, base_delay: Duration, max_delay: Duration, ) -> Duration` | Helper calculating exponential backoff cooldown given consecutive failures. | `src/background_workers/config.rs` |
| `pub fn start_decay_cleanup_worker<S: StorageEngine, V: VectorIndex>( collection: Arc<Collection<S, V>>, decay_config: DecayControllerConfig, interval: Duration, cancel_token: tokio_util::sync::CancellationToken, ) -> tokio::task::JoinHandle<()>` | Starts a background task for decay-controller-driven importance-score eviction. Nur aktiv wenn `background-maintenance` Feature-Flag gesetz… | `src/background_workers/expiry_workers.rs` |
| `pub fn start_expiry_cleanup_worker<S: StorageEngine>( collection: Arc<Collection<S>>, interval: Duration, cancel_token: tokio_util::sync::CancellationToken, ) -> tokio::task::JoinHandle<()>` | Starts a background task to periodically clean up expired documents with TTL. | `src/background_workers/expiry_workers.rs` |
| `pub async fn drain_up_to(&self, max: usize) -> Vec<HyperEdgeId>` | Drains up to `max` hyperedge IDs from the front of the queue (FIFO). | `src/background_workers/hyperedge_worker.rs` |
| `pub async fn enqueue(&self, ids: impl IntoIterator<Item = HyperEdgeId>)` | Enqueues hyperedge IDs for deferred background tombstoning. | `src/background_workers/hyperedge_worker.rs` |
| `pub fn new() -> Self` | Creates a new empty `DeferredHyperedgeQueue`. | `src/background_workers/hyperedge_worker.rs` |
| `pub fn start_hyperedge_cascade_deferred_worker<S: StorageEngine>( collection: Arc<Collection<S>>, queue: Arc<DeferredHyperedgeQueue>, interval: Duration, cancel_token: tokio_util::sync::CancellationToken, ) -> tokio::task::JoinHandle<()>` | Starts a background task to process deferred hyperedge tombstones. | `src/background_workers/hyperedge_worker.rs` |
| `pub fn start_orphan_cleanup_worker<T: Clone + Send + Sync + 'static>( buffer: Arc<TxBuffer<T>>, hnsw_index: Arc<contextra_vector::hnsw::HnswIndex>, interval: Duration, cancel_token: tokio_util::sync::CancellationToken, ) -> tokio::task::JoinHandle<()>` | Starts a background task to periodically clean up orphan transactions. This worker handles the cleanup of transactions that have exceeded t… | `src/background_workers/orphan_workers.rs` |
| `pub fn start_orphan_cleanup_worker_with_config< T: Clone + Send + Sync + 'static, I: OrphanCleanupIndex + 'static, >( buffer: Arc<TxBuffer<T>>, hnsw_index: Arc<I>, interval: Duration, backoff_config: OrphanCleanupBackoffConfig, cancel_token: tokio_util::sync:…` | Starts a background task to periodically clean up orphan transactions and manage HNSW rebuilds with exponential backoff. | `src/background_workers/orphan_workers.rs` |
| `pub fn chunk(&self, doc_id: DocId, markdown: &str) -> Vec<ContextChunk>` | Chunks a Markdown document into semantically coherent pieces. | `src/chunker.rs` |
| `pub fn chunk_text(text: &str, chunk_size: usize) -> Vec<&str>` | — | `src/chunker.rs` |
| `pub fn chunk_text_with_overlap(text: &str, window_chars: usize, overlap_chars: usize) -> Vec<&str>` | — | `src/chunker.rs` |
| `pub fn estimate_tokens(text: &str) -> usize` | Schätzt die Token-Anzahl eines Textes mit heuristischen Regeln. | `src/chunker.rs` |
| `pub fn new(config: ChunkerConfig) -> Self` | Creates a new MarkdownChunker with the given configuration. | `src/chunker.rs` |
| `pub fn with_defaults() -> Self` | Creates a new MarkdownChunker with default configuration. | `src/chunker.rs` |
| `pub fn auto_extraction_config( &self, ) -> Option<(Arc<dyn LlmTextGenerator>, AutoExtractionConfig)>` | Returns the configured auto extraction generator and config, if present. | `src/collection/crud/auto_extraction.rs` |
| `pub fn auto_extraction_config_with_extractor( &self, ) -> Option<( Arc<dyn LlmTextGenerator>, AutoExtractionConfig, ExtractorMode, )>` | Returns the configured auto extraction generator, config, and extractor mode. | `src/collection/crud/auto_extraction.rs` |
| `pub fn for_regulated(is_regulated: bool) -> Self` | Erstellt eine neue `AutoExtractionConfig` unter Verwendung der Empfehlung fuer Regulierungs-Kontexte. | `src/collection/crud/auto_extraction.rs` |
| `pub fn is_enabled(&self) -> bool` | Prueft, ob die automatische Extraktion unter Beruecksichtigung aller Abschaltwege aktiv ist. Precedence / Abschaltkette: 1. Compile-Zeit fe… | `src/collection/crud/auto_extraction.rs` |
| `pub fn new(mode: AutoExtractionMode) -> Self` | Erstellt eine neue `AutoExtractionConfig` mit dem angegebenen Modus. | `src/collection/crud/auto_extraction.rs` |
| `pub fn recommended_mode_for_regulated(is_regulated: bool) -> AutoExtractionMode` | Liefert den empfohlenen `AutoExtractionMode` fuer das gegebene "regulated"-Flag. Gemäss v17 Teil 10.5 soll in EnterpriseRegulated / Regulie… | `src/collection/crud/auto_extraction.rs` |
| `pub fn set_auto_extraction( &self, generator: Arc<dyn LlmTextGenerator>, config: AutoExtractionConfig, )` | Sets or updates the auto extraction generator and config for this collection. | `src/collection/crud/auto_extraction.rs` |
| `pub fn set_auto_extraction_config(&self, config: AutoExtractionConfig)` | Sets or updates auto extraction without an external LLM generator (suitable for RuleBased mode). | `src/collection/crud/auto_extraction.rs` |
| `pub fn set_auto_extraction_with_mode( &self, generator: Arc<dyn LlmTextGenerator>, config: AutoExtractionConfig, extractor_mode: ExtractorMode, )` | Sets or updates auto extraction with an explicit extractor mode. | `src/collection/crud/auto_extraction.rs` |
| `pub fn with_auto_extraction( self, generator: Arc<dyn LlmTextGenerator>, config: AutoExtractionConfig, ) -> Self` | Configures auto extraction with an LLM generator and config (builder pattern). | `src/collection/crud/auto_extraction.rs` |
| `pub fn with_mode(mut self, mode: AutoExtractionMode) -> Self` | Setzt den Laufzeit-Modus (Builder Pattern). | `src/collection/crud/auto_extraction.rs` |
| `pub async fn delete(&self, id: &str) -> Result<()>` | Deletes a document from the collection by its ID. | `src/collection/crud/delete.rs` |
| `pub async fn delete_op( &self, db_tx: &mut crate::transaction::DbTransaction<S, V>, id: &str, ) -> Result<()>` | — | `src/collection/crud/delete.rs` |
| `pub async fn insert( &self, id: &str, embedding: &[f32], metadata: Option<serde_json::Value>, ) -> Result<()>` | Inserts a document with an embedding and optional metadata. | `src/collection/crud/insert.rs` |
| `pub async fn insert_many( &self, docs: &[(String, Vec<f32>, Option<serde_json::Value>)], ) -> Result<()>` | Inserts multiple documents in a single atomic transaction under key-granular locks. | `src/collection/crud/insert.rs` |
| `pub async fn insert_op( &self, db_tx: &crate::transaction::DbTransaction<S, V>, id: &str, embedding: &[f32], metadata: Option<serde_json::Value>, ) -> Result<()>` | — | `src/collection/crud/insert.rs` |
| `pub async fn insert_text_only( &self, id: &str, text: &str, mut metadata: Option<serde_json::Value>, ) -> Result<()>` | Inserts a text document, automatically generating its embedding. | `src/collection/crud/insert.rs` |
| `pub async fn insert_typed( &self, id: &str, embedding: &[f32], memory_type: contextra_types::MemoryType, metadata: Option<serde_json::Value>, ) -> Result<()>` | Speichert ein Dokument mit expliziter kognitiver Gedächtnisklassifikation. | `src/collection/crud/insert.rs` |
| `pub async fn insert_with_ttl( &self, id: &str, embedding: &[f32], metadata: Option<serde_json::Value>, ttl_committed_ops: u64, ) -> Result<()>` | Inserts a document with a Sequence-based Time-To-Live (TTL in committed ops). | `src/collection/crud/insert.rs` |
| `pub async fn upsert( &self, id: &str, embedding: &[f32], metadata: Option<serde_json::Value>, ) -> Result<()>` | Upserts a document (inserts if missing, updates if exists) atomically. | `src/collection/crud/insert.rs` |
| `pub async fn upsert_many( &self, docs: &[(String, Vec<f32>, Option<serde_json::Value>)], ) -> Result<()>` | Upserts multiple documents in a single transaction. | `src/collection/crud/insert.rs` |
| `pub async fn upsert_text_only( &self, id: &str, text: &str, mut metadata: Option<serde_json::Value>, ) -> Result<()>` | Upserts a text document, automatically generating its embedding. | `src/collection/crud/insert.rs` |
| `pub async fn get_kv(&self, id: &str) -> Result<Option<serde_json::Value>>` | Retrieves a key-value entry directly from LSM storage. | `src/collection/crud/kv.rs` |
| `pub async fn put_kv(&self, id: &str, value: &serde_json::Value) -> Result<()>` | Stores a non-vector key-value entry directly in LSM storage without touching vector, text, or graph indices. | `src/collection/crud/kv.rs` |
| `pub async fn put_kv_if_absent(&self, id: &str, value: &serde_json::Value) -> Result<()>` | Stores a non-vector key-value entry directly in LSM storage only if the key does not already exist. Returns `ContextraError::Conflict` if t… | `src/collection/crud/kv.rs` |
| `pub async fn get_links( &self, doc_id: DocId, ) -> Result<Vec<contextra_types::domain::MemoryLink>>` | Retrieves all links for a given document. | `src/collection/crud/links.rs` |
| `pub async fn link_memories( &self, from: DocId, to: DocId, relation: contextra_types::domain::LinkRelation, ) -> Result<()>` | Links two memories together with a specific relation (Zettelkasten A-MEM). | `src/collection/crud/links.rs` |
| `pub async fn get(&self, id: &str) -> Result<Option<crate::Document>>` | Retrieves a document by its user-provided string ID. | `src/collection/crud/read.rs` |
| `pub async fn get_at_seq_tracked( &self, tx: TxId, id: &str, max_seq: u64, ) -> Result<Option<StoredDocument>>` | Fetches a document snapshot bounded by `seq` and tracks the read in `tx`'s `ReadSet`. | `src/collection/crud/read.rs` |
| `pub async fn get_at_snapshot(&self, id: &str, seq_no: u64) -> Result<Option<crate::Document>>` | Retrieves a document at a specific snapshot point. | `src/collection/crud/read.rs` |
| `pub async fn get_doc_tx(&self, doc_id: DocId) -> Result<Option<TxId>>` | Returns the latest transaction ID for a document (via `updated_at_tx` or `created_at_tx`). | `src/collection/crud/read.rs` |
| `pub async fn get_tracked(&self, tx: TxId, id: &str) -> Result<Option<StoredDocument>>` | Fetches a document by user ID and tracks the read in the active transaction's `ReadSet`. | `src/collection/crud/read.rs` |
| `pub async fn scan( &self, start: std::ops::Bound<&[u8]>, end: std::ops::Bound<&[u8]>, limit: Option<usize>, ) -> Result<Vec<(String, serde_json::Value)>>` | Performs a range scan of documents in the collection. | `src/collection/crud/read.rs` |
| `pub async fn scan_prefix( &self, prefix: &str, limit: Option<usize>, ) -> Result<Vec<(String, serde_json::Value)>>` | Scans documents in the collection that match a given key prefix. Caps maximum results to `MAX_SCAN_RESULTS_DEFAULT` to protect against unbo… | `src/collection/crud/read.rs` |
| `pub async fn snapshot_seq(&self) -> Result<u64>` | — | `src/collection/crud/read.rs` |
| `pub async fn update( &self, id: &str, embedding: &[f32], metadata: Option<serde_json::Value>, ) -> Result<()>` | Updates an existing document in the collection. | `src/collection/crud/update.rs` |
| `pub async fn update_op( &self, db_tx: &crate::transaction::DbTransaction<S, V>, id: &str, embedding: &[f32], metadata: Option<serde_json::Value>, ) -> Result<()>` | — | `src/collection/crud/update.rs` |
| `pub async fn lock_for<'a>(&'a self, key: &str) -> KeyGuard<'a>` | — | `src/collection/kv_lock.rs` |
| `pub async fn lock_many_sorted<'a>( &'a self, keys: impl IntoIterator<Item = &'a str>, ) -> Vec<KeyGuard<'a>>` | Locks multiple shards in ascending shard index order to prevent deadlocks across operations. Deduplicates by lock shard index to prevent se… | `src/collection/kv_lock.rs` |
| `pub async fn lock_shard<'a>(&'a self, idx: usize) -> KeyGuard<'a>` | — | `src/collection/kv_lock.rs` |
| `pub fn new() -> Self` | — | `src/collection/kv_lock.rs` |
| `pub fn shard_idx(&self, key: &str) -> usize` | — | `src/collection/kv_lock.rs` |
| `pub fn sorted_unique_shards<'a>( keys: impl IntoIterator<Item = &'a str>, shard_count: usize, hasher: &ahash::RandomState, ) -> Vec<usize>` | Pure, synchronous function to map string keys to unique shard indices in ascending order. Deduplicates shard indices to prevent self-deadlo… | `src/collection/kv_lock.rs` |
| `pub fn try_lock_for<'a>(&'a self, key: &str) -> Option<KeyGuard<'a>>` | — | `src/collection/kv_lock.rs` |
| `pub fn try_lock_shard<'a>(&'a self, idx: usize) -> Option<KeyGuard<'a>>` | — | `src/collection/kv_lock.rs` |
| `pub async fn drop_collection(&self) -> Result<()>` | Removes all data belonging to this collection from storage. | `src/collection/maintenance.rs` |
| `pub async fn evict_decayed_chunks( &self, decay_controller: &AdaptiveDecayController, max_per_tick: usize, ) -> Result<usize>` | Führt einen Importance-Score-Sweep via Adaptive Decay Controller durch. Evictet Chunks deren effective_score unter eviction_threshold liegt… | `src/collection/maintenance.rs` |
| `pub async fn get_communities_batch( &self, entity_ids: &[EntityId], ) -> Result<std::collections::HashMap<EntityId, u64>>` | Retrieves community assignments for a batch of entity IDs in a single operation. | `src/collection/maintenance.rs` |
| `pub async fn get_community(&self, entity_id: EntityId) -> Result<Option<u64>>` | Retrieves the persisted community ID for a given entity. | `src/collection/maintenance.rs` |
| `pub async fn reap_expired_documents(&self, max_expired: usize) -> Result<usize>` | Rebuilds the HNSW index from storage. Scans the collection for documents with expired sequence-based TTLs and deletes them in batches. | `src/collection/maintenance.rs` |
| `pub async fn repair(&self) -> Result<()>` | Repairs the index by re-syncing with the storage. Scans the storage for any documents that are missing from the index and reconciles them.… | `src/collection/maintenance.rs` |
| `pub async fn run_community_detection(&self) -> Result<Vec<CommunityAssignment>>` | Runs Leiden Community Detection on the collection's graph index and persists the resulting assignments in storage using TxId allocation. | `src/collection/maintenance.rs` |
| `pub async fn run_community_detection_with_config( &self, config: &CommunityDetectionConfig, ) -> Result<Vec<CommunityAssignment>>` | Runs Leiden Community Detection with custom configuration and persists the resulting assignments in storage using TxId allocation. | `src/collection/maintenance.rs` |
| `pub async fn run_percolation_check( &self, config: &contextra_graph::percolation::PercolationConfig, ) -> Result<PercolationResult>` | Überprüft die Perkolations-Gesundheit des Wissensgraphen und löst bei Unterschreitung des Schwellenwerts automatisch einen Re-Bonding-Pass… | `src/collection/maintenance.rs` |
| `pub async fn stats(&self) -> Result<contextra_ports::VectorIndexStats>` | Returns statistics for the collection's vector index. | `src/collection/maintenance.rs` |
| `pub async fn trigger_expiry_cleanup(&self) -> Result<usize>` | Scans the collection for documents with expired TTLs or decayed importance scores and deletes them. Reads `created_at_ms` (or `timestamp_ms… | `src/collection/maintenance.rs` |
| `pub async fn update_document_importance( &self, doc_id: &str, importance_score: f32, model_id: &str, ) -> Result<()>` | Aktualisiert den Importance-Score sowie die Provenance (Modell-ID) in den Metadaten. Diese Funktion entkoppelt die LLM-Bewertung (die z.B.… | `src/collection/maintenance.rs` |
| `pub fn clock(&self) -> Arc<dyn Clock>` | Returns a cloned reference to the injected clock. | `src/collection/mod.rs` |
| `pub fn community_detection_trigger_threshold(&self) -> u64` | Returns the configured mutation threshold for auto-triggered community detection. | `src/collection/mod.rs` |
| `pub fn compute_default_importance(text_opt: Option<&str>) -> contextra_types::ImportanceScore` | Computes a non-LLM heuristic baseline importance score based on text length and character entropy. | `src/collection/mod.rs` |
| `pub fn config(&self) -> CollectionConfig` | Returns a copy of the current collection configuration. | `src/collection/mod.rs` |
| `pub fn consolidation_in_progress(&self) -> Arc<AtomicBool>` | Returns a reference handle to the collection's consolidation lock flag. | `src/collection/mod.rs` |
| `pub fn deferred_hyperedge_queue(&self) -> Arc<crate::background_workers::DeferredHyperedgeQueue>` | Returns a cloned handle to the deferred hyperedge queue. | `src/collection/mod.rs` |
| `pub fn delete_kv_segment( &self, manager: &contextra_store::kv::KvSegmentManager, group_id: u64, ) -> bool` | Delegates KV segment key revocation/deletion to `contextra_store::kv::KvSegmentManager`. | `src/collection/mod.rs` |
| `pub fn dimension(&self) -> usize` | Returns the vector dimension for this collection. | `src/collection/mod.rs` |
| `pub fn ensure_importance_metadata( metadata: &mut Option<serde_json::Value>, tx: TxId, text_opt: Option<&str>, )` | Ensures document metadata contains a valid `MemoryImportance` JSON payload. | `src/collection/mod.rs` |
| `pub fn extract_effective_importance(metadata: &Option<serde_json::Value>, now_tx: TxId) -> f32` | Extracts the effective importance score of a document at a given transaction ID. | `src/collection/mod.rs` |
| `pub fn generate_kv_deletion_proof( &self, manager: &contextra_store::kv::KvSegmentManager, group_id: u64, ) -> std::result::Result<bool, contextra_types::ContextraError>` | Delegates KV deletion proof generation to `contextra_store::kv::KvSegmentManager`. | `src/collection/mod.rs` |
| `pub fn graph_index(&self) -> Arc<CsrGraph>` | Returns the CSR graph index for this collection. | `src/collection/mod.rs` |
| `pub async fn is_empty(&self) -> bool` | Returns true if the collection is empty. | `src/collection/mod.rs` |
| `pub fn kv_hooks(&self) -> Option<Arc<dyn KvLifecycleHooks>>` | Returns a cloned reference to the attached `KvLifecycleHooks`, if configured. | `src/collection/mod.rs` |
| `pub fn kv_store(&self) -> Option<&Arc<dyn KvLifecycleHooks>>` | Returns a reference to the attached `TenantIsolatedKvStore`, if configured. | `src/collection/mod.rs` |
| `pub fn kv_store(&self) -> Option<&Arc<dyn KvLifecycleHooks>>` | Returns a reference to the attached `TenantIsolatedKvStore`, if configured. | `src/collection/mod.rs` |
| `pub async fn len(&self) -> usize` | Returns the number of documents in the collection. | `src/collection/mod.rs` |
| `pub async fn load_index(&self) -> Result<()>` | Performs a range scan of documents in the collection. | `src/collection/mod.rs` |
| `pub async fn load_text_stats(&self) -> Result<()>` | Loads text index statistics from storage. | `src/collection/mod.rs` |
| `pub fn metrics_sink(&self) -> Arc<dyn MetricsSink>` | Returns a cloned reference to the active metrics sink. | `src/collection/mod.rs` |
| `pub async fn migrate_doc_keys_v1(&self) -> Result<u64>` | Migrates old doc_keys (with Embedding) to new doc_keys (only Metadata). Safe to call multiple times (idempotent). | `src/collection/mod.rs` |
| `pub fn mutations_since_community_detection(&self) -> u64` | Returns the number of graph mutations since the last community detection execution. | `src/collection/mod.rs` |
| `pub fn name(&self) -> &str` | Returns the name of the collection. | `src/collection/mod.rs` |
| `pub fn namespaced_key(&self, key: &[u8], key_type: u8) -> Vec<u8>` | Internal helper to generate namespaced keys. key_type: 0 = user key, 1 = docid mapping, 2 = relationship, 3 = tx intent, 4 = system/communi… | `src/collection/mod.rs` |
| `pub fn new( name: String, storage: Arc<S>, index: Arc<V>, graph_index: Arc<CsrGraph>, next_tx: Arc<AtomicU64>, dimension: usize, language: Language, ) -> Self` | Creates a new `Collection` instance with explicit language configuration. The `language` parameter controls the BM25 tokenizer. Use `Langua… | `src/collection/mod.rs` |
| `pub fn parse_importance_score(response: &str) -> f32` | Parses an LLM response string into an f32 importance score in `[0.0, 1.0]`. | `src/collection/mod.rs` |
| `pub async fn point_lookup_metrics( &self, storage: &contextra_store::LsmStorage, key: &[u8], ) -> (usize, usize, usize, usize, bool)` | Evaluates point lookup traversal metrics on the underlying LSM storage engine. | `src/collection/mod.rs` |
| `pub fn read_kv_segment( &self, manager: &contextra_store::kv::KvSegmentManager, segment: &contextra_store::kv::KvSegmentPayload, ) -> std::result::Result<Vec<u8>, contextra_types::ContextraError>` | Delegates KV segment reading/decryption to `contextra_store::kv::KvSegmentManager`. | `src/collection/mod.rs` |
| `pub fn set_clock(&self, clock: Arc<dyn Clock>)` | Sets or overrides the clock implementation. | `src/collection/mod.rs` |
| `pub fn set_community_detection_trigger_threshold(&self, threshold: u64)` | Sets the mutation threshold for auto-triggered community detection (0 = disabled). | `src/collection/mod.rs` |
| `pub fn set_config(&self, config: CollectionConfig)` | Sets or updates the collection configuration. | `src/collection/mod.rs` |
| `pub async fn set_embedder(&self, embedder: Arc<dyn TextEmbeddingEngine>) -> Result<()>` | Configures the text embedder for this collection. | `src/collection/mod.rs` |
| `pub fn set_kv_hooks(&self, hooks: Arc<dyn KvLifecycleHooks>)` | Sets KV lifecycle hooks port implementation on the collection. | `src/collection/mod.rs` |
| `pub fn set_kv_store(&mut self, kv_store: Arc<contextra_crypto::TenantIsolatedKvStore>)` | Attaches a `TenantIsolatedKvStore` to the collection for KV cache rollback management. | `src/collection/mod.rs` |
| `pub fn set_kv_store(&mut self, kv_store: Arc<contextra_crypto::TenantIsolatedKvStore>)` | — | `src/collection/mod.rs` |
| `pub fn set_metrics_sink(&self, metrics: Arc<dyn MetricsSink>)` | Sets or overrides the metrics sink implementation. | `src/collection/mod.rs` |
| `pub fn set_pressure_receiver( &self, pressure_rx: tokio::sync::watch::Receiver<contextra_store::SystemPressure>, )` | Sets or overrides the pressure receiver channel. | `src/collection/mod.rs` |
| `pub fn storage(&self) -> &Arc<S>` | Returns a reference to the underlying storage engine. | `src/collection/mod.rs` |
| `pub fn tenant_collection_prefix( &self, codec: &contextra_store::tenant_codec::TenantKeyCodec, collection: &contextra_types::CollectionId, ) -> Vec<u8>` | Generates a collection prefix using TenantKeyCodec. | `src/collection/mod.rs` |
| `pub fn tenant_encode_graph_key( &self, codec: &contextra_store::tenant_codec::TenantKeyCodec, collection: &contextra_types::CollectionId, entity_id: u64, ) -> Vec<u8>` | Encodes a graph key using TenantKeyCodec. | `src/collection/mod.rs` |
| `pub fn tenant_id(&self) -> contextra_types::TenantId` | Returns the tenant ID for this collection (extracted from `TenantScopedStorage` if present, otherwise defaulting to `TenantId::default()`). | `src/collection/mod.rs` |
| `pub fn user_key_prefix(&self) -> Vec<u8>` | Returns the namespaced prefix for user document keys in this collection. | `src/collection/mod.rs` |
| `pub fn vector_index(&self) -> &Arc<V>` | Returns a reference to the vector index of this collection. | `src/collection/mod.rs` |
| `pub fn with_clock(self, clock: Arc<dyn Clock>) -> Self` | Sets or overrides the clock implementation (builder version). | `src/collection/mod.rs` |
| `pub fn with_community_detection_trigger_threshold(self, threshold: u64) -> Self` | Builder method to set the mutation threshold for auto-triggered community detection (0 = disabled). | `src/collection/mod.rs` |
| `pub fn with_config(self, config: CollectionConfig) -> Self` | Sets or updates the collection configuration. | `src/collection/mod.rs` |
| `pub fn with_embedder(self, embedder: Arc<dyn TextEmbeddingEngine>) -> Self` | Sets the text embedder for this collection (consuming version). | `src/collection/mod.rs` |
| `pub fn with_hnsw( name: String, storage: Arc<S>, index: Arc<HnswIndex>, graph_index: Arc<CsrGraph>, next_tx: Arc<AtomicU64>, dimension: usize, language: Language, ) -> Self` | Convenience constructor for creating a `Collection` with `HnswIndex`. | `src/collection/mod.rs` |
| `pub fn with_kv_hooks(self, hooks: Arc<dyn KvLifecycleHooks>) -> Self` | Attaches KV lifecycle hooks port implementation to the collection. | `src/collection/mod.rs` |
| `pub fn with_kv_store(mut self, kv_store: Arc<contextra_crypto::TenantIsolatedKvStore>) -> Self` | Attaches a `TenantIsolatedKvStore` to the collection for KV cache rollback management. | `src/collection/mod.rs` |
| `pub fn with_kv_store(self, kv_store: Arc<contextra_crypto::TenantIsolatedKvStore>) -> Self` | — | `src/collection/mod.rs` |
| `pub fn with_metrics_sink(self, metrics: Arc<dyn MetricsSink>) -> Self` | Sets or overrides the metrics sink implementation (builder version). | `src/collection/mod.rs` |
| `pub fn with_pressure_receiver( self, pressure_rx: tokio::sync::watch::Receiver<contextra_store::SystemPressure>, ) -> Self` | Sets or overrides the pressure receiver channel. | `src/collection/mod.rs` |
| `pub fn write_kv_segment( &self, manager: &contextra_store::kv::KvSegmentManager, group_id: u64, plaintext: &[u8], ) -> std::result::Result<contextra_store::kv::KvSegmentPayload, contextra_types::ContextraError>` | Delegates KV segment payload writing to `contextra_store::kv::KvSegmentManager`. | `src/collection/mod.rs` |
| `pub fn anchor_entities(mut self, anchors: impl IntoIterator<Item = EntityId>) -> Self` | Sets anchor seed entities for graph traversal signal. | `src/collection/query_builder/builder.rs` |
| `pub fn anchors(self, anchors: impl IntoIterator<Item = EntityId>) -> Self` | Alias for `.anchor_entities()`. | `src/collection/query_builder/builder.rs` |
| `pub fn as_of(mut self, timestamp: u64) -> Self` | Sets historical point-in-time "as-of" timestamp for temporal validity filtering (Post-RRF, Pre-Reranking). | `src/collection/query_builder/builder.rs` |
| `pub fn current_tx(mut self, tx: contextra_types::TxId) -> Self` | Sets current system transaction ID for temporal validity filtering (Post-RRF, Pre-Reranking). | `src/collection/query_builder/builder.rs` |
| `pub fn embedding(mut self, embedding: impl AsRef<[f32]>) -> Self` | Sets dense vector query embedding. | `src/collection/query_builder/builder.rs` |
| `pub fn filter(mut self, filter: FilterExpr) -> Self` | Sets metadata expression filter (`FilterExpr`). | `src/collection/query_builder/builder.rs` |
| `pub fn filter_fn(mut self, f: impl Fn(DocId) -> bool + Send + Sync + 'static) -> Self` | Sets custom filter predicate function for vector search candidates. | `src/collection/query_builder/builder.rs` |
| `pub fn fusion_strategy(mut self, strategy: impl Into<SignalFusionStrategies>) -> Self` | Sets search fusion strategy per signal or globally (`FusionStrategy` or `SignalFusionStrategies`). | `src/collection/query_builder/builder.rs` |
| `pub fn fusion_weights(mut self, weights: FusionWeights) -> Self` | Sets custom signal fusion weights directly using `FusionWeights`. | `src/collection/query_builder/builder.rs` |
| `pub fn include_provenance(mut self, include: bool) -> Self` | Sets whether to calculate and attach ProvenanceRecord to output results. | `src/collection/query_builder/builder.rs` |
| `pub fn include_superseded(mut self, include: bool) -> Self` | Sets whether to include superseded documents (Post-RRF Supersedes Displacement, ADR-038). | `src/collection/query_builder/builder.rs` |
| `pub fn k(mut self, k: usize) -> Self` | Sets top-k maximum result count. | `src/collection/query_builder/builder.rs` |
| `pub fn memory_type_filter(mut self, types: impl IntoIterator<Item = MemoryType>) -> Self` | Sets cognitive memory type filter (Pre-RRF filter). | `src/collection/query_builder/builder.rs` |
| `pub fn memory_types(self, types: impl IntoIterator<Item = MemoryType>) -> Self` | Alias for `.memory_type_filter()`. | `src/collection/query_builder/builder.rs` |
| `pub fn metadata_filter(mut self, filter: MetadataFilter) -> Self` | Sets legacy metadata filter (`MetadataFilter`). | `src/collection/query_builder/builder.rs` |
| `pub fn new(collection: &'a Collection<S, V>) -> Self` | Creates a new `HybridQueryBuilder` for a given `Collection`. | `src/collection/query_builder/builder.rs` |
| `pub fn on_signal_failure(mut self, policy: SignalFailurePolicy) -> Self` | Sets signal failure policy (`Fail` or `Degrade`). Spec v17 §6.5: `Fail` (default) fails the entire query if any signal fails. `Degrade` ski… | `src/collection/query_builder/builder.rs` |
| `pub fn pid_controller( mut self, pid: Arc<parking_lot::Mutex<contextra_adapt::PidController>>, ) -> Self` | Sets optional PID controller for adaptive reranking candidate pool sizing. | `src/collection/query_builder/builder.rs` |
| `pub fn query(&self) -> HybridQueryBuilder<'_, S, V>` | Recommended unified entrypoint for search queries using fluent `HybridQueryBuilder`. | `src/collection/query_builder/builder.rs` |
| `pub fn query_config(mut self, query: &contextra_types::HybridQuery) -> Self` | Configures builder options from an existing `HybridQuery` struct. | `src/collection/query_builder/builder.rs` |
| `pub fn query_timestamp(mut self, timestamp: u64) -> Self` | Sets query business timestamp for temporal validity filtering (Post-RRF, Pre-Reranking). | `src/collection/query_builder/builder.rs` |
| `pub fn rerank_pool_max(mut self, max: usize) -> Self` | Sets the upper cap for pre-reranking candidate pool expansion. Default: 200 (`DEFAULT_RERANK_POOL_MAX`), capping pre-retrieval pool size fo… | `src/collection/query_builder/builder.rs` |
| `pub fn rerank_pool_multiplier(mut self, multiplier: usize) -> Self` | Sets the candidate pool multiplier for pre-reranking candidate expansion. Default: 10 (`DEFAULT_RERANK_POOL_MULTIPLIER`), yielding ~100 can… | `src/collection/query_builder/builder.rs` |
| `pub fn reranker(mut self, reranker: &'a (dyn Reranker + 'a)) -> Self` | Sets optional CrossEncoder reranker for post-retrieval ranking. | `src/collection/query_builder/builder.rs` |
| `pub fn same_community_as(mut self, entity_id: EntityId) -> Self` | Sets target entity for community context boosting/filtering. | `src/collection/query_builder/builder.rs` |
| `pub fn seq(mut self, seq_no: u64) -> Self` | Sets snapshot sequence number for MVCC snapshot-isolated queries. | `src/collection/query_builder/builder.rs` |
| `pub fn strategy(mut self, strategy: impl Into<SearchStrategy>) -> Self` | Sets hybrid search / graph traversal strategy. | `src/collection/query_builder/builder.rs` |
| `pub fn text(mut self, text: impl Into<String>) -> Self` | Sets full-text search query string. | `src/collection/query_builder/builder.rs` |
| `pub fn vector(self, vector: impl AsRef<[f32]>) -> Self` | Alias for `.embedding()` to specify dense vector query embedding. | `src/collection/query_builder/builder.rs` |
| `pub fn weights<W>(mut self, weights: W) -> Self where W: TryInto<FusionWeights, Error = contextra_types::ContextraError>,` | Sets custom signal fusion weights using `SignalWeights` or `FusionWeights`. | `src/collection/query_builder/builder.rs` |
| `pub fn with_metrics_sink(mut self, sink: Arc<dyn MetricsSink>) -> Self` | Configures an optional metrics sink for search signal latency instrumentation. | `src/collection/query_builder/builder.rs` |
| `pub async fn execute(self) -> Result<Vec<crate::SearchResult>>` | Executes search query, delegating core signal retrieval to `execute_with_report()`. Under `SignalFailurePolicy::Fail` (default), returns an… | `src/collection/query_builder/builder_exec.rs` |
| `pub async fn execute_with_report( self, ) -> Result<(Vec<crate::SearchResult>, crate::fusion::SearchReport)>` | Executes search query and returns both search results and an execution `SearchReport`. The `SearchReport` captures degraded signals (when u… | `src/collection/query_builder/builder_exec.rs` |
| `pub async fn execute_with_scope(self) -> Result<Vec<crate::SearchResult>>` | Executes search query with ACORN hard-boundary scoping (`.scope(ScopeConstraint)`). Evaluates vector similarity strictly within `allowed_do… | `src/collection/query_builder/builder_exec.rs` |
| `pub fn from_allowed_ids(allowed_doc_ids: BTreeSet<DocId>) -> Self` | Creates a new `ScopeConstraint` with default `gamma = 2` (Spec §8.5). | `src/collection/query_builder/scope.rs` |
| `pub fn new(allowed_doc_ids: BTreeSet<DocId>, gamma: u32) -> Self` | Creates a new `ScopeConstraint` with a custom `gamma` factor. | `src/collection/query_builder/scope.rs` |
| `pub fn scope(mut self, constraint: ScopeConstraint) -> Self` | Sets an explicit ACORN hard-boundary scope constraint for vector retrieval. Spec §5.3: `.scope(ScopeConstraint)` enforces strict document b… | `src/collection/query_builder/scope.rs` |
| `pub fn scope_ids(self, allowed_doc_ids: BTreeSet<DocId>) -> Self` | Convenience builder method setting a hard boundary scope from allowed document IDs. | `src/collection/query_builder/scope.rs` |
| `pub fn to_fusion_strategy(&self) -> FusionStrategy` | Converts `SearchStrategy` into core `FusionStrategy`. | `src/collection/query_builder/strategy.rs` |
| `pub fn to_graph_strategy(&self) -> GraphTraversalStrategy` | Converts `SearchStrategy` into core `GraphTraversalStrategy`. | `src/collection/query_builder/strategy.rs` |
| `pub fn to_signal_fusion_strategies(&self) -> SignalFusionStrategies` | Converts `SearchStrategy` into core `SignalFusionStrategies`. | `src/collection/query_builder/strategy.rs` |
| `pub fn new(vector: f32, text: f32, graph: f32) -> Result<Self>` | Creates a new `SignalWeights` instance, ensuring weights sum to 1.0. | `src/collection/query_builder/weights.rs` |
| `pub async fn relate(&self, from: &str, to: &str, label: &str) -> Result<()>` | Creates a directional relationship between two documents in the collection. | `src/collection/relate.rs` |
| `pub async fn relate_bidirectional(&self, from: &str, to: &str, label: &str) -> Result<()>` | Creates a bidirectional relationship atomically. | `src/collection/relate.rs` |
| `pub async fn relate_n_ary( &self, predicate: &str, participants: &[(&str, &str)], source_doc_id: Option<&str>, ) -> Result<contextra_graph::hyperedge::HyperEdgeId>` | Creates an n-ary hyperedge relationship connecting two or more participants. # Deadlock Prevention & Canonical Lock Ordering `participants`… | `src/collection/relate.rs` |
| `pub async fn relate_with_provenance( &self, from: &str, to: &str, label: &str, source_doc_id: Option<&str>, ) -> Result<()>` | Creates a directional relationship between two documents in the collection with optional source provenance document ID. | `src/collection/relate.rs` |
| `pub async fn search( &self, query_embedding: &[f32], k: usize, ) -> Result<Vec<crate::SearchResult>>` | Performs semantic k-NN search over stored embeddings. | `src/collection/search/basic.rs` |
| `pub async fn search_text( &self, query_text: &str, k: usize, ) -> Result<Vec<crate::SearchResult>>` | Performs semantic search using a raw text query (automatically embedded). | `src/collection/search/basic.rs` |
| `pub async fn search_with_filter( &self, query: &[f32], k: usize, filter: Option<MetadataFilter>, ) -> Result<Vec<crate::SearchResult>>` | Performs semantic search with an advanced metadata filter. | `src/collection/search/basic.rs` |
| `pub async fn search_with_filter_expr( &self, query: &[f32], k: usize, filter: Option<FilterExpr>, ) -> Result<Vec<crate::SearchResult>>` | Performs semantic search with an advanced metadata filter expression (`FilterExpr`). | `src/collection/search/basic.rs` |
| `pub async fn new(storage: &'a S, seq: u64) -> Result<Self>` | Creates a new guard and pins the checkpoint at sequence `seq`. | `src/collection/search/checkpoint.rs` |
| `pub async fn new_at_latest(storage: &'a S) -> Result<(Self, u64)>` | Pin-first, read-after: the seq is read under the protection of the pin, eliminating the TOCTOU window between snapshot_seq() and pin activa… | `src/collection/search/checkpoint.rs` |
| `pub async fn release(mut self) -> Result<()>` | Explicitly unpins the checkpoint and consumes the guard, preventing the fallback `Drop` warning. | `src/collection/search/checkpoint.rs` |
| `pub async fn with_pinned_checkpoint<S, F, Fut, T>(storage: &S, seq: u64, f: F) -> Result<T> where S: StorageEngine + ?Sized, F: FnOnce() -> Fut, Fut: std::future::Future<Output = Result<T>>,` | Higher-order function wrapping an async block with a pinned checkpoint guard, ensuring `release()` is ALWAYS called even on error paths. | `src/collection/search/checkpoint.rs` |
| `pub async fn with_pinned_checkpoint_at_latest<S, F, Fut, T>(storage: &S, f: F) -> Result<T> where S: StorageEngine + ?Sized, F: FnOnce(u64) -> Fut, Fut: std::future::Future<Output = Result<T>>,` | Higher-order function providing pin-first, read-after semantics with automatic release on completion/error. Pin-first, read-after: the seq… | `src/collection/search/checkpoint.rs` |
| `pub async fn search_filtered( &self, query: &[f32], k: usize, filter: Option<&(dyn Fn(DocId) -> bool + Send + Sync)>, ) -> Result<Vec<crate::SearchResult>>` | — | `src/collection/search/filtered.rs` |
| `pub async fn search_filtered_at( &self, query: &[f32], k: usize, filter: Option<&(dyn Fn(DocId) -> bool + Send + Sync)>, seq: u64, ) -> Result<Vec<crate::SearchResult>>` | Performs filtered semantic vector search at a specific MVCC sequence number. | `src/collection/search/filtered.rs` |
| `pub fn filter_by_importance( results: Vec<crate::SearchResult>, min_threshold: f32, now_tx: TxId, ) -> Vec<crate::SearchResult>` | Filters a candidate list of search results by effective importance score threshold. Candidate results with `effective_score(now_tx) < min_t… | `src/collection/search/hybrid/mod.rs` |
| `pub async fn hybrid_search( &self, text: &str, vector: &[f32], k: usize, anchor_entities: Option<&[contextra_types::EntityId]>, ) -> Result<Vec<crate::SearchResult>>` | Performs hybrid search combining BM25, vector search, and graph traversal results via RRF. | `src/collection/search/hybrid/mod.rs` |
| `pub async fn hybrid_search_reranked( &self, text: &str, vector: &[f32], k: usize, reranker: Option<&dyn Reranker>, anchor_entities: Option<&[contextra_types::EntityId]>, ) -> Result<Vec<crate::SearchResult>>` | — | `src/collection/search/hybrid/mod.rs` |
| `pub async fn hybrid_search_with_strategy( &self, text: &str, vector: &[f32], k: usize, anchor_entities: Option<&[contextra_types::EntityId]>, weights: Option<&contextra_types::FusionWeights>, strategy: Option<&contextra_types::GraphTraversalStrategy>, same_co…` | Performs hybrid search with custom signal fusion weights and graph traversal strategy. | `src/collection/search/hybrid/mod.rs` |
| `pub async fn hybrid_search_with_weights( &self, text: &str, vector: &[f32], k: usize, anchor_entities: Option<&[contextra_types::EntityId]>, weights: Option<&contextra_types::FusionWeights>, ) -> Result<Vec<crate::SearchResult>>` | Performs hybrid search with custom fusion weights for vector, text, and graph signals, and optional community filtering/boosting. | `src/collection/search/hybrid/mod.rs` |
| `pub async fn traverse_links( &self, start: DocId, max_depth: usize, ) -> Result<Vec<(DocId, usize)>>` | Traverses Zettelkasten memory links starting from a given document up to a maximum hop depth (ADR-038). Performs iterative BFS with cycle d… | `src/collection/search/hybrid/mod.rs` |
| `pub async fn hybrid_search_with_query( &self, query: &contextra_types::HybridQuery, ) -> Result<Vec<crate::SearchResult>>` | Performs hybrid search combining BM25, vector, and graph signals configured via `HybridQuery`. Applies `memory_type_filter` and metadata `F… | `src/collection/search/hybrid/query.rs` |
| `pub async fn hybrid_search_with_query_and_report_at( &self, query: &contextra_types::HybridQuery, seq: u64, on_signal_failure: SignalFailurePolicy, metrics: Option<&dyn MetricsSink>, ) -> Result<(Vec<crate::SearchResult>, SearchReport)>` | Performs hybrid search returning both search results and an execution `SearchReport`. | `src/collection/search/hybrid/query.rs` |
| `pub async fn hybrid_search_with_query_at( &self, query: &contextra_types::HybridQuery, seq: u64, ) -> Result<Vec<crate::SearchResult>>` | Performs hybrid search combining BM25, vector, and graph signals configured via `HybridQuery` at a specific snapshot sequence. | `src/collection/search/hybrid/query.rs` |
| `pub fn allocate_tx(&self) -> Result<TxId>` | Allokiert eine eindeutige, atomar inkrementierte Transaction-ID. Externe Crates verwenden diese Methode statt eigener TxId-Generierung. Ver… | `src/collection/tx.rs` |
| `pub fn begin_transaction(&self) -> Result<crate::transaction::DbTransaction<S, V>>` | Begins a new atomic transaction for this collection. | `src/collection/tx.rs` |
| `pub fn next_tx(&self) -> Result<TxId>` | — | `src/collection/tx.rs` |
| `pub fn allocate_tx(&self) -> Result<TxId>` | — | `src/contextra_impl/collections.rs` |
| `pub async fn collection(&self, name: &str) -> Result<Arc<Collection<LsmStorage>>>` | Retrieves or creates an un-tenanted [`Collection`] handle (Admin / Single-Tenant path). # Errors Returns [`ContextraError::InvalidInput`] i… | `src/contextra_impl/collections.rs` |
| `pub async fn collection_for_tenant( &self, name: &str, tenant_id: TenantId, ) -> Result<Arc<Collection<contextra_store::tenant_codec::TenantScopedStorage<Arc<LsmStorage>>>>>` | — | `src/contextra_impl/collections.rs` |
| `pub async fn collection_with_language( &self, name: &str, language: Language, ) -> Result<Arc<Collection<LsmStorage>>>` | Retrieves or creates an un-tenanted [`Collection`] handle with specified language (Admin / Single-Tenant path). # Errors Returns [`Contextr… | `src/contextra_impl/collections.rs` |
| `pub async fn collection_with_language_and_tenant( &self, name: &str, language: Language, tenant_id: TenantId, ) -> Result<Arc<Collection<contextra_store::tenant_codec::TenantScopedStorage<Arc<LsmStorage>>>>>` | — | `src/contextra_impl/collections.rs` |
| `pub async fn drop_collection( &self, name: &str, tenant_id: TenantId, proof_key: &[u8], ) -> Result<DeletionProof>` | Drops a collection and issues a [`DeletionProof`] attesting to logical LSM key space deletion. # Limitations / Non-Guarantees - Does NOT gu… | `src/contextra_impl/collections.rs` |
| `pub async fn list_collections(&self) -> Result<Vec<String>>` | — | `src/contextra_impl/collections.rs` |
| `pub async fn list_collections_for_tenant(&self, tenant_id: TenantId) -> Result<Vec<String>>` | — | `src/contextra_impl/collections.rs` |
| `pub async fn purge_tenant(&self, tenant_id: TenantId) -> Result<()>` | Purges all cached KV segments and stored collection data for the given tenant across the engine. # Limitations / Non-Guarantees (SCHRITT 4)… | `src/contextra_impl/collections.rs` |
| `pub async fn create_snapshot(&self) -> Result<u64>` | — | `src/contextra_impl/crud.rs` |
| `pub async fn delete(&self, id: &str) -> Result<()>` | — | `src/contextra_impl/crud.rs` |
| `pub async fn get(&self, id: &str) -> Result<Option<Document>>` | — | `src/contextra_impl/crud.rs` |
| `pub async fn get_at_snapshot(&self, id: &str, seq_no: u64) -> Result<Option<Document>>` | — | `src/contextra_impl/crud.rs` |
| `pub async fn get_kv(&self, id: &str) -> Result<Option<Value>>` | — | `src/contextra_impl/crud.rs` |
| `pub async fn hybrid_search( &self, text: &str, vector: &[f32], k: usize, anchor_entities: Option<&[contextra_types::EntityId]>, ) -> Result<Vec<SearchResult>>` | — | `src/contextra_impl/crud.rs` |
| `pub async fn hybrid_search_reranked( &self, text: &str, vector: &[f32], k: usize, reranker: Option<&dyn contextra_ports::Reranker>, anchor_entities: Option<&[contextra_types::EntityId]>, ) -> Result<Vec<SearchResult>>` | — | `src/contextra_impl/crud.rs` |
| `pub async fn hybrid_search_with_query( &self, query: &contextra_types::HybridQuery, ) -> Result<Vec<SearchResult>>` | — | `src/contextra_impl/crud.rs` |
| `pub async fn hybrid_search_with_strategy( &self, text: &str, vector: &[f32], k: usize, anchor_entities: Option<&[contextra_types::EntityId]>, weights: Option<&contextra_types::FusionWeights>, strategy: Option<&contextra_types::GraphTraversalStrategy>, ) -> Re…` | — | `src/contextra_impl/crud.rs` |
| `pub async fn hybrid_search_with_weights( &self, text: &str, vector: &[f32], k: usize, anchor_entities: Option<&[contextra_types::EntityId]>, weights: Option<&contextra_types::FusionWeights>, ) -> Result<Vec<SearchResult>>` | — | `src/contextra_impl/crud.rs` |
| `pub async fn insert(&self, id: &str, embedding: &[f32], metadata: Option<Value>) -> Result<()>` | — | `src/contextra_impl/crud.rs` |
| `pub async fn insert_many(&self, docs: &[(String, Vec<f32>, Option<Value>)]) -> Result<()>` | — | `src/contextra_impl/crud.rs` |
| `pub async fn insert_text_only( &self, id: &str, text: &str, metadata: Option<Value>, ) -> Result<()>` | — | `src/contextra_impl/crud.rs` |
| `pub async fn insert_typed( &self, collection_name: &str, id: &str, embedding: &[f32], memory_type: contextra_types::MemoryType, metadata: Option<Value>, ) -> Result<()>` | — | `src/contextra_impl/crud.rs` |
| `pub async fn is_empty(&self) -> Result<bool>` | — | `src/contextra_impl/crud.rs` |
| `pub async fn last_committed_seq(&self) -> Result<u64>` | — | `src/contextra_impl/crud.rs` |
| `pub async fn len(&self) -> Result<usize>` | — | `src/contextra_impl/crud.rs` |
| `pub async fn put_kv(&self, id: &str, value: &Value) -> Result<()>` | — | `src/contextra_impl/crud.rs` |
| `pub async fn relate(&self, from: &str, to: &str, label: &str) -> Result<()>` | — | `src/contextra_impl/crud.rs` |
| `pub async fn relate_bidirectional(&self, from: &str, to: &str, label: &str) -> Result<()>` | — | `src/contextra_impl/crud.rs` |
| `pub async fn scan( &self, start: std::ops::Bound<&[u8]>, end: std::ops::Bound<&[u8]>, limit: Option<usize>, ) -> Result<Vec<(String, Value)>>` | — | `src/contextra_impl/crud.rs` |
| `pub async fn scan_prefix( &self, prefix: &str, limit: Option<usize>, ) -> Result<Vec<(String, Value)>>` | — | `src/contextra_impl/crud.rs` |
| `pub async fn search(&self, query: &[f32], k: usize) -> Result<Vec<SearchResult>>` | — | `src/contextra_impl/crud.rs` |
| `pub async fn search_filtered( &self, query: &[f32], k: usize, filter: Option<&(dyn Fn(DocId) -> bool + Send + Sync)>, ) -> Result<Vec<SearchResult>>` | — | `src/contextra_impl/crud.rs` |
| `pub async fn search_text(&self, text: &str, k: usize) -> Result<Vec<SearchResult>>` | — | `src/contextra_impl/crud.rs` |
| `pub async fn search_with_filter( &self, query: &[f32], k: usize, filter: Option<MetadataFilter>, ) -> Result<Vec<SearchResult>>` | — | `src/contextra_impl/crud.rs` |
| `pub async fn search_with_filter_expr( &self, query: &[f32], k: usize, filter: Option<FilterExpr>, ) -> Result<Vec<SearchResult>>` | — | `src/contextra_impl/crud.rs` |
| `pub async fn update(&self, id: &str, embedding: &[f32], metadata: Option<Value>) -> Result<()>` | — | `src/contextra_impl/crud.rs` |
| `pub async fn upsert(&self, id: &str, embedding: &[f32], metadata: Option<Value>) -> Result<()>` | — | `src/contextra_impl/crud.rs` |
| `pub async fn upsert_many(&self, docs: &[(String, Vec<f32>, Option<Value>)]) -> Result<()>` | — | `src/contextra_impl/crud.rs` |
| `pub async fn upsert_text_only( &self, id: &str, text: &str, metadata: Option<Value>, ) -> Result<()>` | — | `src/contextra_impl/crud.rs` |
| `pub async fn flush(&self) -> Result<()>` | — | `src/contextra_impl/diagnostics.rs` |
| `pub fn orphan_registry(&self) -> &Arc<contextra_checkpoint::InstanceOrphanRegistry>` | — | `src/contextra_impl/diagnostics.rs` |
| `pub async fn stats(&self) -> Result<ContextraStats>` | — | `src/contextra_impl/diagnostics.rs` |
| `pub async fn close(self) -> Result<()>` | — | `src/contextra_impl/lifecycle.rs` |
| `pub async fn open(path: impl AsRef<Path>) -> Result<Self>` | Opens or creates a `Contextra` instance at `path` enforcing `deletion_proof_active = false` and using default [`contextra_ports::license::L… | `src/contextra_impl/lifecycle.rs` |
| `pub async fn open_authorized( path: impl AsRef<Path>, mut config: ContextraConfig, authorized_ring: &AuthorizedRing, gate: Arc<dyn LicenseGate>, ) -> Result<Self>` | Opens or creates a `Contextra` instance with an explicitly authorized [`AuthorizedRing`] token and attached [`LicenseGate`]. | `src/contextra_impl/lifecycle.rs` |
| `pub async fn open_with_config( path: impl AsRef<Path>, mut config: ContextraConfig, ) -> Result<Self>` | Opens or creates a `Contextra` instance at `path` with explicit `config`, forcing `deletion_proof_active = false` (as no authorized token w… | `src/contextra_impl/lifecycle.rs` |
| `pub async fn repair_on_open(&self) -> Result<()>` | — | `src/contextra_impl/lifecycle.rs` |
| `pub async fn set_embedder(&self, embedder: Arc<dyn TextEmbeddingEngine>) -> Result<()>` | — | `src/contextra_impl/lifecycle.rs` |
| `pub fn shutdown(&self)` | — | `src/contextra_impl/lifecycle.rs` |
| `pub async fn wait_shutdown(&self)` | — | `src/contextra_impl/lifecycle.rs` |
| `pub async fn with_embedder(self, embedder: Arc<dyn TextEmbeddingEngine>) -> Self` | — | `src/contextra_impl/lifecycle.rs` |
| `pub fn metrics_sink(&self) -> Arc<dyn contextra_ports::MetricsSink>` | Returns the active [`MetricsSink`]. | `src/contextra_impl/wiring.rs` |
| `pub fn set_calibrator( &self, calibrator: std::sync::Weak<parking_lot::Mutex<contextra_rank::IsotonicCalibrator>>, )` | — | `src/contextra_impl/wiring.rs` |
| `pub fn set_metrics_sink(&self, sink: Arc<dyn contextra_ports::MetricsSink>)` | Configures or replaces the active [`MetricsSink`]. | `src/contextra_impl/wiring.rs` |
| `pub fn set_pid_controller( &self, pid_controller: std::sync::Weak<parking_lot::Mutex<contextra_adapt::PidController>>, )` | — | `src/contextra_impl/wiring.rs` |
| `pub fn set_router(&self, router: std::sync::Weak<dyn DriftStatusProvider>)` | — | `src/contextra_impl/wiring.rs` |
| `pub async fn export_memories(&self) -> Result<ExportCollectionV1>` | Exportiert alle Memories und Graph-Relationen der Collection in das ExportCollectionV1 Schema. | `src/export.rs` |
| `pub fn create_extractor(mode: ExtractorMode) -> Box<dyn EntityExtractor>` | Erstellt eine Instanz des konfigurierten Extraktors. | `src/extraction/mod.rs` |
| `pub async fn extract_triples_for_config( text: &str, generator: Option<&dyn LlmTextGenerator>, config: &EntityExtractionConfig, ) -> Result<Vec<ExtractedTriple>>` | Hilfsfunktion zur Ausführung der Extraktion basierend auf der Standard-Konfiguration (OpenIE). | `src/extraction/mod.rs` |
| `pub async fn extract_triples_for_mode( mode: ExtractorMode, text: &str, generator: Option<&dyn LlmTextGenerator>, config: &EntityExtractionConfig, ) -> Result<Vec<ExtractedTriple>>` | Hilfsfunktion zur Ausführung der Extraktion für einen expliziten Extraktor-Modus. | `src/extraction/mod.rs` |
| `pub async fn extract_triples( text: &str, generator: &dyn LlmTextGenerator, config: &EntityExtractionConfig, ) -> Result<Vec<ExtractedTriple>>` | Extrahiert Wissens-Tripel (Subjekt, Prädikat, Objekt) aus dem gegebenen Text mittels LLM. # Fehlerverhalten - Wenn `config.enabled == false… | `src/extraction/open_ie.rs` |
| `pub fn extract( &self, text: &str, config: &EntityExtractionConfig, ) -> Result<Vec<ExtractedTriple>>` | Extrahieren von Wissens-Tripeln rein regelbasiert und deterministisch. | `src/extraction/rule_based.rs` |
| `pub fn new() -> Self` | Erstellt eine neue Instanz von `RuleBasedExtractor`. | `src/extraction/rule_based.rs` |
| `pub fn extract_memory_type(metadata: &Option<Value>) -> MemoryType` | Extrahiert den MemoryType aus Dokument-Metadaten (Rückwärtskompatibel). | `src/filter.rs` |
| `pub fn matches(&self, metadata: &Value) -> bool` | — | `src/filter.rs` |
| `pub fn fuse_search_results_with_signal_strategies( result_sets: Vec<(String, Vec<SearchResult>, f32)>, max_results: usize, priority: MetadataMergePriority, include_provenance: bool, resonance_config: Option<&ResonanceConfig>, strategies: contextra_types::Sign…` | Fuses search result sets using per-signal fusion strategies (`SignalFusionStrategies`). | `src/fusion.rs` |
| `pub fn fuse_search_results_with_strategy( result_sets: Vec<(String, Vec<SearchResult>, f32)>, max_results: usize, priority: MetadataMergePriority, include_provenance: bool, resonance_config: Option<&ResonanceConfig>, strategy: contextra_types::FusionStrategy,…` | Fuses search result sets using the specified FusionStrategy. | `src/fusion.rs` |
| `pub fn fuse_signals( result_sets: Vec<(String, Vec<SearchResult>, f32)>, max_results: usize, ) -> Vec<SearchResult>` | Convenience function for fusing multi-signal search results. | `src/fusion.rs` |
| `pub async fn import_memories(&self, col_export: ExportCollectionV1) -> Result<ImportSummary>` | Importiert alle Memories und Graph-Relationen einer ExportCollectionV1 idempotent (Upsert-Semantik). | `src/import.rs` |
| `pub async fn import_memories(&self, doc: ExportDocumentV1) -> Result<ImportSummary>` | Importiert ein ExportDocumentV1 in die Contextra-Datenbank mit Schema-Versionsprüfung und Idempotenz. | `src/import.rs` |
| `pub fn build_eviction_worker(store: Arc<TenantIsolatedKvStore>) -> EvictionWorker` | Primary composition root function for building an [`EvictionWorker`]. Spawns the [`EvictionWorker`] for the provided [`TenantIsolatedKvStor… | `src/kv_cache_integration/mod.rs` |
| `pub fn build_eviction_worker_with_exporter( store: Arc<TenantIsolatedKvStore>, exporter: Arc<dyn AttentionExporter>, ) -> EvictionWorker` | Composition root function allowing injection of a custom [`AttentionExporter`]. Spawns the [`EvictionWorker`] and connects it with the supp… | `src/kv_cache_integration/mod.rs` |
| `pub fn get_access_count(&self, request_id: RequestId) -> u64` | Returns the current access count for the given request ID. | `src/kv_cache_integration/mod.rs` |
| `pub fn new() -> Self` | Creates a new `AccessCounterAttentionExporter`. | `src/kv_cache_integration/mod.rs` |
| `pub fn record_access(&self, request_id: RequestId)` | Records an access for the given request ID, incrementing its frequency counter. | `src/kv_cache_integration/mod.rs` |
| `pub fn start_kv_cache_eviction(store: Arc<TenantIsolatedKvStore>) -> EvictionWorker` | Engine lifecycle integration entry point for KV cache background eviction. Delegates to [`build_eviction_worker_with_exporter`] to instanti… | `src/kv_cache_integration/mod.rs` |
| `pub fn config(&self) -> &ContextraConfig` | Returns a reference to the active [`ContextraConfig`]. | `src/lib.rs` |
| `pub fn create( scope: DeletionScope, mut deleted_keys: Vec<Vec<u8>>, tx_id: TxId, layer_proofs: Vec<LayerCleanupProof>, _cas_proofs: Vec<Vec<u8>>, proof_key: &[u8], ) -> contextra_types::Result<Self>` | — | `src/lib.rs` |
| `pub fn export_for_audit(&self) -> contextra_types::Result<String>` | — | `src/lib.rs` |
| `pub fn inner_storage(&self) -> Arc<LsmStorage>` | — | `src/lib.rs` |
| `pub fn kv_eviction_attention_source( &self, ) -> Option<Arc<dyn contextra_kvcache::AttentionScoreSource>>` | Public observability API: exposes the active KV eviction worker's attention score source for external monitoring integrations. Intentionall… | `src/lib.rs` |
| `pub fn kv_hooks(&self) -> Option<Arc<dyn contextra_ports::KvLifecycleHooks>>` | Returns a cloned reference to the active KV lifecycle hooks, if configured. | `src/lib.rs` |
| `pub fn license_gate(&self) -> Arc<dyn LicenseGate>` | Returns a reference to the attached [`LicenseGate`]. | `src/lib.rs` |
| `pub fn new() -> Self` | — | `src/lib.rs` |
| `pub fn new_after_verified_empty( layer: DeletionLayer, remaining_count: usize, ) -> contextra_types::Result<Self>` | — | `src/lib.rs` |
| `pub fn on_rollback(&self, tenant: TenantId, _chunk_ids: &[u64])` | — | `src/lib.rs` |
| `pub fn purge_tenant_segments(&self, tenant: TenantId)` | — | `src/lib.rs` |
| `pub fn register_consolidation_launcher<F>(_f: F) where F: Fn( Arc<Collection<LsmStorage>>, std::time::Duration, usize, tokio_util::sync::CancellationToken, ) -> tokio::task::JoinHandle<()> + Send + Sync + 'static,` | — | `src/lib.rs` |
| `pub fn remove_tenant_segment(&self, tenant: TenantId, doc_id: contextra_types::DocId)` | — | `src/lib.rs` |
| `pub fn set_kv_hooks(&self, hooks: Arc<dyn contextra_ports::KvLifecycleHooks>)` | Sets or overrides the KV lifecycle hooks implementation on Contextra. | `src/lib.rs` |
| `pub fn tenant_id(&self) -> TenantId` | — | `src/lib.rs` |
| `pub fn verify(&self, key: &[u8]) -> contextra_types::Result<bool>` | — | `src/lib.rs` |
| `pub fn verify_and_create<F>( layer: DeletionLayer, verifier: F, ) -> contextra_types::Result<Self> where F: FnOnce() -> contextra_types::Result<bool>,` | — | `src/lib.rs` |
| `pub fn with_consolidation_launcher(mut self, launcher: ConsolidationLauncher) -> Self` | — | `src/lib.rs` |
| `pub fn with_metrics_sink(mut self, sink: Arc<dyn contextra_ports::MetricsSink>) -> Self` | — | `src/lib.rs` |
| `pub fn with_tenant_policy(mut self, policy: TenantPolicy) -> Self` | — | `src/lib.rs` |
| `pub fn new(wasm_bytes: impl Into<Arc<[u8]>>) -> Result<Self>` | Creates a new `WasmMergeOperator` with default capability limits (10,000,000 fuel, 5s timeout). | `src/merge_operator_adapter.rs` |
| `pub fn new_with_config( wasm_bytes: impl Into<Arc<[u8]>>, max_fuel: u64, wall_clock_timeout: Duration, ) -> Result<Self>` | Creates a new `WasmMergeOperator` with custom `max_fuel` and `wall_clock_timeout` configuration. # Errors Returns `ContextraError::Sandbox`… | `src/merge_operator_adapter.rs` |
| `pub fn record_counter_safe( sink: &dyn contextra_ports::MetricsSink, name: &str, value: u64, labels: &[(&str, &str)], )` | Helper to record a counter metric best-effort without allowing panics to propagate. | `src/metrics_names.rs` |
| `pub fn record_gauge_safe( sink: &dyn contextra_ports::MetricsSink, name: &str, value: f64, labels: &[(&str, &str)], )` | Helper to record a gauge metric best-effort without allowing panics to propagate. | `src/metrics_names.rs` |
| `pub fn record_histogram_safe( sink: &dyn contextra_ports::MetricsSink, name: &str, value: f64, labels: &[(&str, &str)], )` | Helper to record a histogram metric best-effort without allowing panics to propagate. | `src/metrics_names.rs` |
| `pub fn apply_temporal_validity_filter( results: Vec<crate::SearchResult>, current_tx: TxId, query_timestamp: Option<u64>, ) -> Vec<crate::SearchResult>` | Applies bi-temporal validity filtering to candidates post-RRF fusion. Filters out candidate results that are invalid relative to the curren… | `src/temporal_filter.rs` |
| `pub fn apply_temporal_validity_filter_at( results: Vec<crate::SearchResult>, as_of_timestamp: u64, ) -> Vec<crate::SearchResult>` | Applies historical "as-of" bi-temporal validity filtering to candidates post-RRF fusion. Evaluates candidate validity at historical point-i… | `src/temporal_filter.rs` |
| `pub fn extract_validity_window(metadata: Option<&Value>) -> ValidityWindow` | Extracts bi-temporal validity window metadata from candidate document metadata. | `src/temporal_filter.rs` |
| `pub fn is_empty(&self) -> bool` | Returns `true` if no validity window metadata constraints are set. Candidates without validity window metadata (legacy data) are treated as… | `src/temporal_filter.rs` |
| `pub fn is_valid_bitemporal(&self, as_of_tx: TxId, as_of_business: Option<u64>) -> bool` | Evaluates bi-temporal validity using strict AND logic across system transaction time and business time. Intervall-Semantik: `[valid_from, v… | `src/temporal_filter.rs` |
| `pub async fn cleanup_orphaned_consolidation_intents<S: StorageEngine>( storage: &S, next_tx: &std::sync::atomic::AtomicU64, ) -> Result<usize>` | Aufgerufen beim Öffnen einer Collection / DB. Findet und entfernt verwaiste ConsolidationIntents aus dem WAL/Storage. | `src/transaction/cleanup.rs` |
| `pub async fn execute_rollback(&mut self)` | — | `src/transaction/compensating_actions.rs` |
| `pub fn new() -> Self` | — | `src/transaction/compensating_actions.rs` |
| `pub fn new( collection: Collection<S, V>, intent_key: Vec<u8>, doc_ids: Arc<Vec<DocId>>, f_keys: Vec<StagedKeyOp>, r_keys: Vec<StagedKeyOp>, ) -> Self` | — | `src/transaction/compensating_actions.rs` |
| `pub fn push<A: CompensatingAction + 'static>(&mut self, action: A)` | — | `src/transaction/compensating_actions.rs` |
| `pub fn new(collection: Collection<S, V>, tx_id: TxId) -> Self` | — | `src/transaction/db_transaction.rs` |
| `pub fn record_keys(&self, forward: Vec<u8>, reverse: Vec<u8>, doc_id: DocId)` | Records keys and IDs that have been operated on for potential compensating rollback. | `src/transaction/db_transaction.rs` |
| `pub fn record_keys_with_old_values( &self, forward: Vec<u8>, old_forward: Option<Bytes>, reverse: Vec<u8>, old_reverse: Option<Bytes>, doc_id: DocId, )` | Records keys and IDs along with pre-write values for potential compensating rollback. | `src/transaction/db_transaction.rs` |
| `pub fn stage_graph_edge(&self, edge: Edge)` | — | `src/transaction/db_transaction.rs` |
| `pub fn stage_graph_edge_delete(&self, from: EntityId, to: EntityId)` | — | `src/transaction/db_transaction.rs` |
| `pub fn stage_graph_entity(&self, entity: Entity)` | — | `src/transaction/db_transaction.rs` |
| `pub fn stage_graph_entity_delete(&self, entity_id: EntityId)` | — | `src/transaction/db_transaction.rs` |
| `pub fn stage_hyperedge(&self, hyperedge: contextra_graph::hyperedge::HyperEdge)` | — | `src/transaction/db_transaction.rs` |
| `pub fn stage_text_delete(&self, doc_id: DocId)` | — | `src/transaction/db_transaction.rs` |
| `pub fn stage_text_insert(&self, doc_id: DocId, text: String)` | — | `src/transaction/db_transaction.rs` |
| `pub async fn commit(self) -> Result<()>` | Commits the transaction atomically across all 4 indices (LSM, HNSW, BM25, CSR). | `src/transaction/db_transaction_commit.rs` |
| `pub async fn rollback(self) -> Result<()>` | Rolls back any uncommitted changes applied to all 4 sub-systems in reverse commit order. | `src/transaction/db_transaction_rollback.rs` |


---

## `contextra-cognition`

| Attribut | Wert |
|---|---|
| **Ring** | 3 — Ring 3 — Orchestration & Cognition |
| **Status** | In Migration |
| **Zweck** | Hintergrund-Kompaktierung, Synthese und Konsolidierungs-Scheduler. |
| **Unsafe-Code** | Nein (`#![forbid(unsafe_code)]`) |
| **Interne Abhängigkeiten** | `contextra-engine`, `contextra-graph`, `contextra-ports`, `contextra-store`, `contextra-types`, `contextra-vector` |
| **Externe Abhängigkeiten (Auswahl)** | `ahash`, `blake3`, `bytes`, `parking_lot`, `serde`, `serde_json`, `thiserror`, `tokio`, `tokio-util`, `tracing` |

### Modul- und Re-Export-Oberfläche (lib.rs)

```rust
pub mod aggregation_phase;
pub mod consolidation_executor;
pub mod consolidation_locks;
pub mod context;
pub mod context_compaction;
pub mod graph_sink;
pub mod leanrag_input;
pub mod maintenance_config;
pub mod maintenance_scheduler;
pub mod memory_consolidation;
pub mod semantic_aggregation_facade;
pub mod synthesis_phase;
pub mod transitivity_veto;
pub use aggregation_phase::{ check_compaction_budget, compute_entity_community_hash, run_aggregation_pass, AggregationConfig, AggregationEdge, AggregationNode, AggregationPhaseResult, AlphaNode, ConsolidationPipelineResult, SuperEdgeDraft, SuperEdgeSink, };
pub use consolidation_executor::{ execute_background_consolidation, execute_consolidation_pass, execute_leanrag_aggregation_stage, execute_sleep_cycle, start_consolidation_reaper, start_consolidation_worker, ConsolidationEngine, };
pub use consolidation_locks::ConsolidationNodesGuard;
pub use context::{ContextManager, SpatialFence};
pub use context_compaction::{ cleanup_orphaned_consolidation_intents, CompactedContext, CompactionStrategy, ConsolidationSession, ContextCompactor, StatusToken, };
pub use graph_sink::CsrGraphSuperEdgeSink;
pub use leanrag_input::{build_leanrag_inputs, LeanRagInputs, DEFAULT_MAX_LEANRAG_NODES};
pub use maintenance_config::MaintenanceConfig;
pub use maintenance_scheduler::MaintenanceScheduler;
pub use memory_consolidation::{ compact_segment_via_context_compactor, compute_community_hash, detect_near_duplicates, group_turns_into_segments, intake_pre_condensed_inputs, run_consolidation_pass, CommunityStabilityTracker, ConsolidationConfig, ConsolidationPhaseResult, MetaChunk, PreCondensedInput, SynthesisConfig, SynthesisPhaseResult, TurnSegment, MAX_PRE_CONDENSED_DIGEST_LEN, MAX_PRE_CONDENSED_LABEL_LEN, MAX_PRE_CONDENSED_PROVENANCE_COUNT, };
pub use semantic_aggregation_facade::consolidate_semantic_hyperedges;
pub use synthesis_phase::run_synthesis_pass;
pub use transitivity_veto::{filter_candidates_with_transitivity_veto, validate_transitivity_veto};
```

### Feature-Flags

| Flag | Aktiviert |
|---|---|
| `default` | `[]` |
| `graph-connectivity-health` | `["contextra-graph/physio-percolation", "contextra-engine/graph-connectivity-health"]` |
| `edge-reinforcement-learning` | `["contextra-graph/edge-reinforcement-learning", "contextra-engine/edge-reinforcement-learning"]` |

### Schnittstellen — Public Traits (1)

**`SuperEdgeSink`** — *Trait zur Abstraktion des Speicherziels für Superkanten und Tombstones.* (`src/aggregation_phase.rs`)

```rust
pub trait SuperEdgeSink {
/// Markiert eine Kante als tombstoned.
    fn tombstone_edge(&mut self, id: HyperEdgeId) -> Result<()>;
    /// Schreibt eine neue Super-Hyperkante und gibt deren zugewiesene ID zurück.
    fn write_super_edge(&mut self, draft: SuperEdgeDraft) -> Result<HyperEdgeId>;
    /// Atomarer Commit aller Änderungen.
    fn commit(&mut self) -> Result<()>;
}
```

### Öffentliche Datentypen — Structs (33)

| Struct | Felder / Form | Beschreibung | Datei |
|---|---|---|---|
| `ActiveSessionGuard` | `{ scheduler: &'a MaintenanceScheduler<S, V>, }` | RAII-Guard for active agent sessions. Increments session count on creation and decrements on drop. | `src/maintenance_scheduler.rs` |
| `AggregationConfig` | `{ /// Maximal zulässiger Speicherverbrauch für Compaction/Aggregation in MB (Default: 512). pub max_compaction_peak_memory_mb: usize, /// Schwelle λ für die Supe… }` | Konfiguration für die Aggregation-Phase. | `src/aggregation_phase.rs` |
| `AggregationEdge` | `{ /// Eindeutige ID der Hyperkante. pub id: HyperEdgeId, /// Prädikatstyp der Kante. pub predicate_type: u32, /// Beteiligte Entitäten der Kante. pub participant… }` | Eingabe-Kante für die Aggregationsphase. | `src/aggregation_phase.rs` |
| `AggregationNode` | `{ /// Eindeutige EntityId des Knotens. pub entity: EntityId, /// Merkmalsvektor/Embedding. pub embedding: Vec<f32>, /// Typ-ID des Entitätstyps. pub type_id: u32, }` | Eingabe-Knoten für die Aggregationsphase. | `src/aggregation_phase.rs` |
| `AggregationPhaseResult` | `{ /// Anzahl tombstonierter Kanten wegen Inkompatibilität. pub raw_edges_tombstoned: usize, /// Anzahl erzeugter abstrakter Super-Hyperkanten. pub abstract_hyper… }` | Ergebnis der Aggregation-Phase. | `src/aggregation_phase.rs` |
| `AlphaNode` | `{ /// Cluster-Index. pub cluster_id: usize, /// Mitglieds-Entitäten des Clusters. pub members: Vec<EntityId>, /// LLM-generierte Zusammenfassung/Synthese. pub su… }` | Ein synthetisierter Alpha-Knoten (abstraktes Community-Konzept). | `src/aggregation_phase.rs` |
| `CommunityStabilityTracker` | `{ history: std::collections::HashMap<u64, u32>, }` | Verfolgt die Stabilität von Graph-Communities über aufeinanderfolgende Zyklen hinweg. | `src/memory_consolidation.rs` |
| `CompactedContext` | `{ /// Beibehaltene Chunks (innerhalb Budget). pub retained_chunks: Vec<ContextChunk>, /// Status-Token für kompaktierte Chunks. pub status_tokens: Vec<StatusToke… }` | Kompaktierter Kontext für LLM-Übergabe. | `src/context_compaction/types.rs` |
| `ConsolidationConfig` | `{ /// Mindestanzahl von Turns pro Segment (Default: 3). pub min_turns_per_segment: usize, /// Maximale Anzahl von Turns pro Segment (Default: 20). pub max_turns_… }` | Konfiguration für den Structural Consolidation Pass. | `src/memory_consolidation.rs` |
| `ConsolidationEngine` | `{ collection: Arc<Collection<S, V>>, llm: Option<Arc<dyn LlmTextGenerator>>, validator: Option<Arc<dyn ResponseGroundingValidator>>, rich_validator: Option<Arc<d… }` | Tokio-Background-Task Engine für periodische Speicher-Konsolidierung und Wissenssynthese. | `src/consolidation_executor.rs` |
| `ConsolidationLockGuard` | `{ flag: Arc<AtomicBool>, }` | RAII Guard zur Koordination des Konsolidierungslaufs. Stellt sicher, dass das `consolidation_in_progress`-Flag bei Beendigung oder Panic stets panic-… | `src/consolidation_executor.rs` |
| `ConsolidationNodesGuard` | `{ collection: &'a Collection<S, V>, _guard: MutexGuard<'a, ()>, }` | Guard zur Erzwingung der Lock-Reihenfolge und kanonischen Node-Sortierung waehrend der Konsolidierung. Garantiert, dass: 1. `collection.consolidation… | `src/consolidation_locks.rs` |
| `ConsolidationPhaseResult` | `{ /// Anzahl der erzeugten Segmente. pub segments_created: usize, /// Liste aller DocIds, die als Duplikate markiert/tombstoned wurden. pub duplicates_tombstoned… }` | Ergebnis des Structural Consolidation Pass. | `src/memory_consolidation.rs` |
| `ConsolidationPipelineResult` | `{ /// Ergebnis des Structural Pass. pub structural: ConsolidationPhaseResult, /// Ergebnis des Synthesis Pass. pub synthesis: crate::memory_consolidation::Synthe… }` | Gesamtergebnis der Konsolidierungspipeline (Spec §21.4). | `src/aggregation_phase.rs` |
| `ConsolidationSession` | `{ /// Reference to the active collection. pub collection: &'a Collection<S, V>, /// Source document IDs and their transaction IDs captured at read snapshot time.… }` | Optimistic Concurrency Control (OCC) Consolidation Session for Sleep-Cycle Memory Compaction. Prevents lost updates / phantom erasures by verifying t… | `src/context_compaction/session.rs` |
| `ContextCompactor` | `{ budget: TokenBudget, strategy: CompactionStrategy, rng: Arc<dyn Rng>, }` | Context Compaction Engine. | `src/context_compaction/compactor.rs` |
| `ContextManager` | `{ /// Token budget configuration. budget: TokenBudget, /// Adaptive relevance threshold. relevance_threshold: f32, }` | Manages autonomous context preparation for LLM consumption. Implements Small-to-Big retrieval: 1. Find small, precise chunks (Small Retrieval) 2. Loa… | `src/context.rs` |
| `CsrGraphSuperEdgeSink` | `{ graph: &'a CsrGraph, wal_tx: TxId, pending_tombstones: Vec<HyperEdgeId>, pending_edges: Vec<HyperEdge>, }` | Buffered sink implementation writing synthetic superedges and tombstone requests to [`CsrGraph`]. # Invariant Assumes a single active writer per `Csr… | `src/graph_sink.rs` |
| `GroundingValidatorAdapter` | `(tuple/unit)` | Führt die vollständige Hintergrund-Konsolidierung (Structural Consolidation Pass und optional Generative Synthesis Pass) aus. 1. Structural Consolida… | `src/consolidation_executor.rs` |
| `LeanRagInputs` | `{ /// Liste der zugrundeliegenden Entitätsknoten. pub nodes: Vec<AggregationNode>, /// Liste der zugrundeliegenden Hyperkanten zwischen den Knoten. pub edges: Ve… }` | Extrahiertes Eingabepaket für die LeanRAG Stage-3-Aggregation. | `src/leanrag_input.rs` |
| `MaintenanceConfig` | `{ /// Intervall zwischen Ticks in Sekunden. Default: 60s. pub tick_interval_secs: u64, // --- F-01 Adaptive Decay Controller --- /// Ob Decay-Eviction aktiviert… }` | Zentrale Konfiguration für den `MaintenanceScheduler` (§10.2). | `src/maintenance_config.rs` |
| `MaintenanceScheduler` | `{ config: MaintenanceConfig, collection: Arc<Collection<S, V>>, consolidation_config: ConsolidationConfig, #[cfg(feature = "edge-reinforcement-learning")] edge_r… }` | Zentraler Scheduler für die Ausführung der Background-Maintenance-Prozesse (ADR-079). | `src/maintenance_scheduler.rs` |
| `MetaChunk` | `{ /// Der synthetisierte, abstrakte Inhalt (MUSS mit `[SYNTHESIZED FROM {n} SOURCES] ` beginnen). pub content: String, /// DocIds aller Quell-Chunks, aus denen s… }` | Ein generativ synthetisierter Wissens-Chunk (MetaChunk) aus dem Generative Synthesis Pass. | `src/memory_consolidation.rs` |
| `PercolationConfig` | `{ pub critical_threshold: f32, pub rebonding_similarity: f32, pub max_new_edges_per_pass: usize, }` | — | `src/maintenance_config.rs` |
| `PreCondensedInput` | `{ pub tenant_id: contextra_types::TenantId, pub source_label: String, pub text_digest: String, pub token_count: usize, pub provenance: Vec<DocId>, }` | Agent-independent pre-condensed input structure (e.g. from agent scratchpad checkpoint or external pipeline). | `src/memory_consolidation.rs` |
| `SegmentSynthesisResult` | `{ /// Neu synthetisierte Chunks (einer pro konsolidiertem Segment). pub synthesized_chunks: Vec<SynthesizedChunk>, /// Anzahl der Segmente, für die keine Synthes… }` | Ergebnis des Generative Synthesis Pass auf Segment-Ebene. | `src/synthesis_phase.rs` |
| `SpatialFence` | `{ /// The region identifier to filter by. pub region: String, /// The metadata field name containing the region. pub field_name: String, }` | Spatial fencing for geographically constrained context (optional). Filters context by geographic region metadata field. | `src/context.rs` |
| `StatusToken` | `{ /// Kompakter Beschreibungstext (z. B. "Tool-Output: DB-Abfrage lieferte 42 Ergebnisse"). pub summary: String, /// Anzahl der ersetzten originalen Tokens. pub… }` | Kompakter Stellvertreter für einen oder mehrere kompaktierte Chunks. | `src/context_compaction/types.rs` |
| `SuperEdgeDraft` | `{ /// Indizes der beiden beteiligten Cluster `(j, k)`. pub cluster_pair: (usize, usize), /// Alle beteiligten Entitäten aus den untergeordneten Kanten. pub parti… }` | Entwurf einer abstrakten Super-Hyperkante. | `src/aggregation_phase.rs` |
| `SynthesisConfig` | `{ /// Mindestanzahl von Chunks/Dokumenten in einer Community (Default: 4). pub min_community_size: usize, /// Anzahl aufeinanderfolgender Beobachtungen, bis eine… }` | Konfiguration für den Generative Synthesis Pass. | `src/memory_consolidation.rs` |
| `SynthesisPhaseResult` | `{ /// Neu generierte MetaChunks. pub synthesized: Vec<MetaChunk>, /// Hashes von qualifizierten Communities, deren Synthese wegen `max_llm_calls_per_cycle` versc… }` | Ergebnis des Generative Synthesis Pass. | `src/memory_consolidation.rs` |
| `SynthesizedChunk` | `{ /// Der neue, abstrakte Inhalt. pub content: String, /// DocIds der Quell-Turns aus dem ursprünglichen Segment. pub source_turn_ids: Vec<DocId>, /// Modell, da… }` | Ein generativ synthetisierter Wissens-Chunk mit Provenienz. | `src/synthesis_phase.rs` |
| `TurnSegment` | `{ /// Liste aller DocIds der Turns in diesem Segment. pub turn_ids: Vec<DocId>, /// Repräsentatives Embedding des Segments. /// Berechnet als Zentroid (Mittelwer… }` | Repräsentiert ein semantisch zusammenhängendes Segment aus aufeinanderfolgenden Turns. | `src/memory_consolidation.rs` |

### Öffentliche Datentypen — Enums (1)

| Enum | Varianten | Beschreibung | Datei |
|---|---|---|---|
| `CompactionStrategy` | `/// Einfachste Strategie: Chunks über Token-Limit werden weggelassen. Truncate, /// Summarisierung via LLM (erfordert externen Summarizer-Trait). Summarize, /// Ersetze Tool-Outputs durch kompakte St…` | Strategie für Context Compaction. | `src/context_compaction/types.rs` |

### Typ-Aliase (1)

| Alias | Definition | Beschreibung |
|---|---|---|
| `SynthesisPhaseResult` | `type SynthesisPhaseResult= SegmentSynthesisResult` | — |

### Konstanten (4)

| Konstante | Typ | Beschreibung |
|---|---|---|
| `DEFAULT_MAX_LEANRAG_NODES` | `usize` | Standardmäßige Obergrenze für die Anzahl extrahierter Knoten im LeanRAG-Input. |
| `MAX_PRE_CONDENSED_DIGEST_LEN` | `usize` | Maximum allowed length in bytes for text_digest in pre-condensed input. |
| `MAX_PRE_CONDENSED_LABEL_LEN` | `usize` | Maximum allowed length in bytes for source_label in pre-condensed input. |
| `MAX_PRE_CONDENSED_PROVENANCE_COUNT` | `usize` | Maximum allowed provenance count in pre-condensed input. |

### Öffentliche Funktionen (76)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn check_compaction_budget(peak_bytes: usize, cfg: &AggregationConfig) -> Result<()>` | Prüft, ob der geschätzte Compaction-Speicherbedarf das konfigurierte Budget überschreitet. | `src/aggregation_phase.rs` |
| `pub fn compute_entity_community_hash(members: &[EntityId]) -> u64` | Berechnet einen deterministischen Hash über eine Liste von `EntityId`s. Im Gegensatz zu `compute_community_hash(&[DocId])` verwendet diese… | `src/aggregation_phase.rs` |
| `pub async fn run_aggregation_pass( nodes: &[AggregationNode], edges: &[AggregationEdge], cfg: &AggregationConfig, llm: &dyn LlmTextGenerator, tracker: &mut CommunityStabilityTracker, sink: &mut dyn SuperEdgeSink, ) -> Result<(AggregationPhaseResult, Vec<Alpha…` | Führt die Aggregation (Consolidation Stage 3) aus. | `src/aggregation_phase.rs` |
| `pub fn validate(&self) -> Result<()>` | Validiert die Konfigurationseinstellungen. | `src/aggregation_phase.rs` |
| `pub async fn execute_background_consolidation<S: StorageEngine, V: VectorIndex>( collection: &Collection<S, V>, turns: &[(DocId, Vec<f32>)], consolidation_config: &ConsolidationConfig, synthesis_config: Option<&SynthesisConfig>, llm: Option<&dyn LlmTextGenera…` | Führt die vollständige Hintergrund-Konsolidierung aus. Abwärtskompatibler Wrapper um [`execute_background_consolidation_with_rich_validator… | `src/consolidation_executor.rs` |
| `pub async fn execute_background_consolidation_with_rich_validator< S: StorageEngine, V: VectorIndex, >( collection: &Collection<S, V>, turns: &[(DocId, Vec<f32>)], consolidation_config: &ConsolidationConfig, synthesis_config: Option<&SynthesisConfig>, llm: Op…` | Führt die vollständige Hintergrund-Konsolidierung mit Unterstützung für `rich_validator` (`GroundingValidator`) aus. 1. Structural Consolid… | `src/consolidation_executor.rs` |
| `pub async fn execute_consolidation_pass<S: StorageEngine, V: VectorIndex>( collection: &Collection<S, V>, turns: &[(DocId, Vec<f32>)], config: &ConsolidationConfig, ) -> Result<ConsolidationPhaseResult>` | Führt den Structural Consolidation Pass aus UND wendet die Ergebnisse an (Tombstones, Graph-Cascade). Gibt das `ConsolidationPhaseResult` z… | `src/consolidation_executor.rs` |
| `pub async fn execute_leanrag_aggregation_stage<S: StorageEngine, V: VectorIndex>( collection: &Collection<S, V>, nodes: &[AggregationNode], edges: &[AggregationEdge], cfg: &AggregationConfig, llm: &dyn LlmTextGenerator, tracker: &mut CommunityStabilityTracker…` | Führt die Stage-3-Aggregation (LeanRAG) aus und wendet Kanten-Tombstones und Super-Hyperkanten atomar über [`CsrGraphSuperEdgeSink`] an. | `src/consolidation_executor.rs` |
| `pub async fn execute_sleep_cycle<S: StorageEngine, V: VectorIndex>( collection: &Collection<S, V>, turns: &[(DocId, Vec<f32>)], consolidation_config: &ConsolidationConfig, synthesis_config: Option<&SynthesisConfig>, llm: Option<&dyn LlmTextGenerator>, stabili…` | Deprecated legacy wrapper for `execute_background_consolidation`. | `src/consolidation_executor.rs` |
| `pub fn new( collection: Arc<Collection<S, V>>, consolidation_config: ConsolidationConfig, synthesis_config: SynthesisConfig, interval: std::time::Duration, cancel_token: tokio_util::sync::CancellationToken, ) -> Self` | Erstellt eine neue `ConsolidationEngine`-Instanz. | `src/consolidation_executor.rs` |
| `pub async fn run(&self)` | Hauptschleife der `ConsolidationEngine`. | `src/consolidation_executor.rs` |
| `pub async fn run_cycle( &self, ) -> Result<(ConsolidationPhaseResult, Option<SynthesisPhaseResult>)>` | Führt einen einzelnen Konsolidierungszyklus aus. | `src/consolidation_executor.rs` |
| `pub fn start(self: Arc<Self>) -> tokio::task::JoinHandle<()>` | Startet die Engine als Tokio-Background-Task. | `src/consolidation_executor.rs` |
| `pub fn start_consolidation_reaper<S: StorageEngine + 'static, V: VectorIndex + 'static>( collection: Arc<Collection<S, V>>, consolidation_config: ConsolidationConfig, synthesis_config: SynthesisConfig, interval: std::time::Duration, cancel_token: tokio_util::…` | Deprecated legacy alias for `start_consolidation_worker`. | `src/consolidation_executor.rs` |
| `pub fn start_consolidation_worker<S: StorageEngine + 'static, V: VectorIndex + 'static>( collection: Arc<Collection<S, V>>, consolidation_config: ConsolidationConfig, synthesis_config: SynthesisConfig, interval: std::time::Duration, cancel_token: tokio_util::…` | Starts a background task for periodic consolidation. | `src/consolidation_executor.rs` |
| `pub fn start_consolidation_worker_full<S: StorageEngine + 'static, V: VectorIndex + 'static>( collection: Arc<Collection<S, V>>, consolidation_config: ConsolidationConfig, synthesis_config: SynthesisConfig, interval: std::time::Duration, cancel_token: tokio_u…` | Starts a fully-configured background task for periodic consolidation with LLM, rich validator, and LeanRAG stage 3. | `src/consolidation_executor.rs` |
| `pub fn start_worker( collection: Arc<Collection<S, V>>, consolidation_config: ConsolidationConfig, synthesis_config: SynthesisConfig, interval: std::time::Duration, cancel_token: tokio_util::sync::CancellationToken, ) -> tokio::task::JoinHandle<()>` | Starts a background task for periodic consolidation. | `src/consolidation_executor.rs` |
| `pub fn start_worker_full( collection: Arc<Collection<S, V>>, consolidation_config: ConsolidationConfig, synthesis_config: SynthesisConfig, interval: std::time::Duration, cancel_token: tokio_util::sync::CancellationToken, llm: Option<Arc<dyn LlmTextGenerator>>…` | Starts a fully-configured background task for periodic consolidation. | `src/consolidation_executor.rs` |
| `pub fn try_acquire(flag: &Arc<AtomicBool>) -> Option<Self>` | Versucht, das Konsolidierungs-Lock atomic zu erwerben (`compare_exchange`). Gibt `Some(ConsolidationLockGuard)` zurück, wenn das Lock erfol… | `src/consolidation_executor.rs` |
| `pub fn with_leanrag(mut self, cfg: AggregationConfig) -> Self` | Fügt eine optionale AggregationConfig für LeanRAG Stage 3 (Opt-in) hinzu. | `src/consolidation_executor.rs` |
| `pub fn with_llm(mut self, llm: Arc<dyn LlmTextGenerator>) -> Self` | Fügt ein optionales LLM für die generative Wissenssynthese hinzu. | `src/consolidation_executor.rs` |
| `pub fn with_rich_validator(mut self, validator: Arc<dyn GroundingValidator>) -> Self` | Fügt einen optionalen reichhaltigen Grounding-Validator (`GroundingValidator`) mit Kalibrierung & Policy-Violation-Semantik hinzu. | `src/consolidation_executor.rs` |
| `pub fn with_validator(mut self, validator: Arc<dyn ResponseGroundingValidator>) -> Self` | Fügt einen optionalen Grounding-Validator für den Generative Synthesis Pass hinzu. | `src/consolidation_executor.rs` |
| `pub async fn cascade_invalidate_edges_ordered( &self, superseded_doc_ids: &[DocId], wal_seq: u64, ) -> (Vec<(DocId, Result<CascadeInvalidationReport>)>, Vec<String>)` | Führt kaskadierende Graph-Edge-Invalidierungen für eine Liste supersedeter Dokumente aus. Sortiert die Eingabe-`DocId`s vor der Verarbeitun… | `src/consolidation_locks.rs` |
| `pub fn collection(&self) -> &'a Collection<S, V>` | Referenz auf die geschützte Collection. | `src/consolidation_locks.rs` |
| `pub fn try_acquire(collection: &'a Collection<S, V>) -> Option<Self>` | Versucht, den `consolidation_guard` der Collection zu erwerben. Gibt `Some(ConsolidationNodesGuard)` zurueck, wenn der Lock frei war, sonst… | `src/consolidation_locks.rs` |
| `pub fn estimate_tokens(text: &str) -> usize` | Schätzt die Token-Anzahl eines Textes mit heuristischen Regeln. Algorithmus (BPE-Approximation, kalibriert auf cl100k_base / GPT-4): - Jede… | `src/context.rs` |
| `pub fn matches(&self, chunk: &ContextChunk) -> bool` | Checks if a chunk's metadata matches this spatial fence. | `src/context.rs` |
| `pub fn new(budget: TokenBudget) -> Self` | Creates a new ContextManager with the given token budget. | `src/context.rs` |
| `pub fn new(region: impl Into<String>) -> Self` | Creates a new spatial fence for the given region. | `src/context.rs` |
| `pub fn prepare_context(&self, mut chunks: Vec<ContextChunk>) -> Result<ContextWindow>` | Prepares a context window from retrieved chunks. Filters by relevance threshold, sorts by score, and truncates to budget. | `src/context.rs` |
| `pub fn relevance_threshold(&self) -> f32` | Returns the current relevance threshold. | `src/context.rs` |
| `pub fn set_relevance_threshold(&mut self, threshold: f32)` | Sets the minimum relevance score for context inclusion. | `src/context.rs` |
| `pub fn with_defaults() -> Self` | Creates a ContextManager with default settings. | `src/context.rs` |
| `pub fn with_relevance_threshold(mut self, threshold: f32) -> Self` | Builder pattern helper to set the minimum relevance score for context inclusion. | `src/context.rs` |
| `pub fn with_threshold(mut self, threshold: f32) -> Self` | Convenient builder pattern alias to set the minimum relevance threshold. | `src/context.rs` |
| `pub async fn cleanup_orphaned_consolidation_intents<S: StorageEngine>( storage: &S, next_tx: &std::sync::atomic::AtomicU64, ) -> Result<usize>` | Aufgerufen beim Öffnen einer Collection / DB. Findet und entfernt verwaiste ConsolidationIntents aus dem WAL/Storage. Verwaiste Intents ent… | `src/context_compaction/cleanup.rs` |
| `pub fn compact(&self, chunks: Vec<ContextChunk>) -> CompactedContext` | Kompaktiert eine Liste von Chunks auf das Token-Budget. Priorisiert nach Relevanz-Score. Tool-Output-Chunks (erkennbar an Metadata-Key "too… | `src/context_compaction/compactor.rs` |
| `pub async fn consolidate_via_llm( &self, tenant: TenantId, chunks: &[ContextChunk], generator: &(impl LlmTextGenerator + ?Sized), _model: &str, ) -> Result<CompactedContext>` | Consolidates multiple context chunks into a single summarized chunk using an external LLM via Ollama. Preserves strict provenance tracking… | `src/context_compaction/compactor.rs` |
| `pub async fn consolidate_with_retry<S, V, G>( collection: &Collection<S, V>, source_doc_ids: &[DocId], target_doc_id: DocId, generator: &G, max_retries: usize, ) -> Result<()> where S: StorageEngine, V: VectorIndex, G: LlmTextGenerator + ?Sized,` | Führt eine Konsolidierung durch und wiederholt bei OCC-Konflikten automatisch. # Parameter - `collection`: die Ziel-Collection - `source_do… | `src/context_compaction/compactor.rs` |
| `pub async fn consolidate_with_retry_with_rng<S, V, G>( collection: &Collection<S, V>, source_doc_ids: &[DocId], target_doc_id: DocId, generator: &G, max_retries: usize, rng: &dyn Rng, ) -> Result<()> where S: StorageEngine, V: VectorIndex, G: LlmTextGenerator…` | Führt eine Konsolidierung durch und wiederholt bei OCC-Konflikten automatisch mit einem konfigurierbaren RNG. | `src/context_compaction/compactor.rs` |
| `pub fn new(budget: TokenBudget, strategy: CompactionStrategy) -> Self` | Creates a new `ContextCompactor` with given `TokenBudget` and `CompactionStrategy`. | `src/context_compaction/compactor.rs` |
| `pub fn with_rng(mut self, rng: Arc<dyn Rng>) -> Self` | Setzt den Zufallszahlengenerator für Backoff-Jitter. | `src/context_compaction/compactor.rs` |
| `pub async fn abort(self) -> Result<()>` | Cancels / aborts the consolidation session, removing the intent key. | `src/context_compaction/session.rs` |
| `pub async fn commit( self, target_string_id: &str, embedding: &[f32], summary_content: &str, metadata: Option<serde_json::Value>, ) -> Result<()>` | Commits the consolidated document and removes the source documents. | `src/context_compaction/session.rs` |
| `pub async fn commit_ref( &self, target_string_id: &str, embedding: &[f32], summary_content: &str, metadata: Option<serde_json::Value>, ) -> Result<()>` | Commits the consolidated document and removes the source documents (takes `&self`). | `src/context_compaction/session.rs` |
| `pub async fn execute<G>(&self, generator: &G) -> Result<()> where G: LlmTextGenerator + ?Sized,` | Executes the consolidation process using the provided LLM text generator: validates OCC, reads source document contents, generates summary,… | `src/context_compaction/session.rs` |
| `pub async fn refresh(&mut self) -> Result<Vec<DocId>>` | Refreshes the consolidation session by reading the current transaction IDs of all source documents. This is used to retry consolidation aft… | `src/context_compaction/session.rs` |
| `pub async fn start( collection: &'a Collection<S, V>, source_doc_ids: &[DocId], target_id: DocId, ) -> Result<Self>` | Starts a consolidation session, snapshotting the version of each source document and writing a `CommitIntent::Consolidation` intent to stor… | `src/context_compaction/session.rs` |
| `pub async fn validate_occ(&self) -> Result<()>` | Validates optimistic concurrency control: checks that every source document has NOT been mutated since the consolidation session started. | `src/context_compaction/session.rs` |
| `pub fn new(graph: &'a CsrGraph, wal_tx: TxId) -> Self` | Creates a new `CsrGraphSuperEdgeSink` instance attached to the given graph and WAL transaction. | `src/graph_sink.rs` |
| `pub fn build_leanrag_inputs<S: StorageEngine, V: VectorIndex>( collection: &Collection<S, V>, turns: &[(DocId, Vec<f32>)], max_nodes: usize, ) -> LeanRagInputs` | Baut [`LeanRagInputs`] aus den Turns einer Collection und den assoziierten Graphendaten. # Konvertierungsregeln - **Knoten**: Für jeden Tur… | `src/leanrag_input.rs` |
| `pub fn acquire_session_guard(&self) -> ActiveSessionGuard<'_, S, V>` | Acquires an RAII session guard that increments active agent session count during its lifetime. | `src/maintenance_scheduler.rs` |
| `pub fn active_agent_sessions(&self) -> usize` | Liefert die Anzahl aktuell aktiver Agenten-Sessions. | `src/maintenance_scheduler.rs` |
| `pub fn decrement_active_sessions(&self) -> usize` | Verringert die Anzahl aktiver Agenten-Sessions. | `src/maintenance_scheduler.rs` |
| `pub fn increment_active_sessions(&self) -> usize` | Erhöht die Anzahl aktiver Agenten-Sessions. | `src/maintenance_scheduler.rs` |
| `pub fn new( config: MaintenanceConfig, collection: Arc<Collection<S, V>>, consolidation_config: ConsolidationConfig, ) -> Self` | Erstellt eine neue Instanz des `MaintenanceScheduler`. | `src/maintenance_scheduler.rs` |
| `pub fn new_with_edge_reinforcement( config: MaintenanceConfig, collection: Arc<Collection<S, V>>, consolidation_config: ConsolidationConfig, buffer: Arc<contextra_graph::EdgeReinforcementBuffer>, ) -> Self` | Erstellt eine neue Instanz des `MaintenanceScheduler` mit `EdgeReinforcementBuffer`. | `src/maintenance_scheduler.rs` |
| `pub async fn run_tick(&self)` | Führt einen einzelnen sequenziellen Maintenance-Tick durch (ADR-079). | `src/maintenance_scheduler.rs` |
| `pub fn start( self: Arc<Self>, cancel_token: tokio_util::sync::CancellationToken, ) -> tokio::task::JoinHandle<()>` | Startet den MaintenanceScheduler in einem eigenen Tokio-Task. | `src/maintenance_scheduler.rs` |
| `pub fn with_clock(mut self, clock: Arc<dyn Clock>) -> Self` | Setzt die Uhr für den `MaintenanceScheduler` (Standard: `SystemClock::new()`). | `src/maintenance_scheduler.rs` |
| `pub fn with_edge_reinforcement_buffer( mut self, buffer: Arc<contextra_graph::EdgeReinforcementBuffer>, ) -> Self` | Setzt den optionalen `EdgeReinforcementBuffer` für F-03. | `src/maintenance_scheduler.rs` |
| `pub fn compact_segment_via_context_compactor( segment: &TurnSegment, compactor: &ContextCompactor, chunks: &[ContextChunk], ) -> CompactedContext` | Adapterfunktion zur Kompaktierung eines Segments via des bereits vorhandenen `ContextCompactor`. Wählt Chunks aus `chunks`, die zu `segment… | `src/memory_consolidation.rs` |
| `pub fn compute_community_hash(member_doc_ids: &[DocId]) -> u64` | Berechnet einen deterministischen 64-Bit-Hash für eine Liste von Member-DocIds. | `src/memory_consolidation.rs` |
| `pub fn detect_near_duplicates(turns: &[(DocId, Vec<f32>)], threshold: f32) -> Vec<(DocId, DocId)>` | Führt einen paarweisen Cosine-Similarity-Vergleich INNERHALB eines Segments durch (O(n²) segmentlokal). Bei `similarity > threshold` wird d… | `src/memory_consolidation.rs` |
| `pub fn group_turns_into_segments( turns: &[(DocId, Vec<f32>)], config: &ConsolidationConfig, ) -> Vec<TurnSegment>` | Gruppiert semantisch zusammenhängende, zeitlich benachbarte Turns via sequenziellem Sliding-Window-Clustering. Ein neuer Turn gehört zum ak… | `src/memory_consolidation.rs` |
| `pub fn intake_pre_condensed_inputs( target_tenant_id: &contextra_types::TenantId, inputs: &[PreCondensedInput], validator: Option<&dyn ResponseGroundingValidator>, min_grounding_score: Option<f32>, ) -> contextra_types::Result<Vec<TurnSegment>>` | Ingests one or more pre-condensed inputs as nodes into the memory consolidation pipeline. # Requirements & Invariants - **Strict Tenant Bou… | `src/memory_consolidation.rs` |
| `pub fn new() -> Self` | — | `src/memory_consolidation.rs` |
| `pub fn observe(&mut self, community_members_hash: u64) -> u32` | Registriert die Beobachtung einer Community und liefert die aktualisierte Anzahl aufeinanderfolgender Zyklen. | `src/memory_consolidation.rs` |
| `pub fn reset_if_absent(&mut self, currently_observed: &std::collections::HashSet<u64>)` | Setzt nicht mehr beobachtete Communities zurück/entfernt sie aus der Historie. | `src/memory_consolidation.rs` |
| `pub fn run_consolidation_pass( turns: &[(DocId, Vec<f32>)], config: &ConsolidationConfig, ) -> ConsolidationPhaseResult` | Orchestriert den Structural Consolidation Pass (Segmentierung & Near-Duplicate-Detection). **Vorbedingung / Invariante:** Das übergebene `t… | `src/memory_consolidation.rs` |
| `pub async fn run_structural_synthesis_pass( stable_communities: &[(u64, Vec<DocId>)], source_texts: &std::collections::HashMap<DocId, String>, llm: &dyn LlmTextGenerator, config: &SynthesisConfig, validator: Option<&dyn ResponseGroundingValidator>, ) -> conte…` | Dies ist der DETERMINISTISCHE, LLM-FREIE Structural Consolidation Pass. Für den LLM-basierten Generative Synthesis Pass siehe `synthesis_ph… | `src/memory_consolidation.rs` |
| `pub async fn consolidate_semantic_hyperedges<S: StorageEngine, V: VectorIndex>( collection: &Collection<S, V>, embedder: &dyn TextEmbeddingEngine, llm: &dyn LlmTextGenerator, config: &AggregationConfig, ) -> Result<AggregationPhaseResult>` | Ausführung der semantischen Hyperkanten-Konsolidierung (LeanRAG Stage 3) auf einer Collection. # Parameter - `collection`: Referenz auf die… | `src/semantic_aggregation_facade.rs` |
| `pub async fn run_synthesis_pass( segments: &[TurnSegment], segment_texts: &[Vec<String>], // Texte der Turns pro Segment synthesizer: &dyn SegmentSynthesizer, min_turns_for_synthesis: usize, // Default: 3 — kurze Segmente überspringen ) -> SegmentSynthesisRes…` | Dies ist der LLM-BASIERTE Generative Synthesis Pass (erzeugt `SynthesizedChunk`s via `SegmentSynthesizer`-Trait). Für den deterministischen… | `src/synthesis_phase.rs` |
| `pub fn filter_candidates_with_transitivity_veto( turns: &[(DocId, Vec<f32>)], threshold: f32, agg_cfg: &AggregationConfig, ) -> Vec<(DocId, DocId)>` | Orchestrierungsfunktion für die Transitivitätsprüfung unter Einhaltung der P24-Lokalität. **Ablauf**: (a) Nutzt zuerst `detect_near_duplica… | `src/transitivity_veto.rs` |
| `pub fn validate_transitivity_veto( node_a: &AggregationNode, node_b: &AggregationNode, node_c: &AggregationNode, threshold: f32, ) -> bool` | Reine, synchrone Funktion, die für drei `AggregationNode`-Kandidaten (A, B, C) und einen Ähnlichkeits-Schwellenwert prüft, ob eine transiti… | `src/transitivity_veto.rs` |


---

## `contextra-privacy`

| Attribut | Wert |
|---|---|
| **Ring** | 3 — Ring 3 — Orchestration & Cognition |
| **Status** | In Migration |
| **Zweck** | Cloud-Egress Privacy Gateway, PII-Vault, Surrogat-Tokenisierung und DLP. |
| **Unsafe-Code** | Nein (`#![forbid(unsafe_code)]`) |
| **Interne Abhängigkeiten** | `contextra-ports`, `contextra-types` |
| **Externe Abhängigkeiten (Auswahl)** | `blake3`, `parking_lot`, `rand`, `regex`, `scc`, `serde`, `serde_json`, `thiserror`, `tokio`, `tracing`, `zeroize` |

### Modul- und Re-Export-Oberfläche (lib.rs)

```rust
pub mod audit_trace;
pub mod avv_generator;
pub mod bulk_exfiltration_detector;
pub mod context_edit_audit;
pub mod egress_gateway;
pub mod egress_guard;
pub mod egress_vault;
pub mod error;
pub mod guarded_payload;
pub mod processing_registry;
pub use audit_trace::{compute_audit_trace, extract_rule_id, EgressClassifierTrace};
pub use avv_generator::{generate_avv_draft, AvvContext};
pub use bulk_exfiltration_detector::{ BulkExfiltrationDetector, BulkExfiltrationOutcome, SessionId, };
pub use context_edit_audit::{ build_context_edit_audit_record, compute_record_hash, render_audit_line, verify_audit_chain, ContextEditAuditError, ContextEditAuditRecord, ContextEditKind, };
pub use egress_gateway::{ handle_cloud_query, handle_cloud_query_with_bulk_detector, handle_cloud_query_with_guard, pii_vault_forces_crypto_shred, process_cloud_response, resolve_effective_kv_delete_mode, CloudQueryRequest, CloudQueryResponse, CloudResponseRehydrator, DefaultEgressClassifier, EgressGuardCheck, InjectionDetector, MAX_SEARCH_QUERY_BYTES, };
pub use egress_guard::{ EgressGuard, TextSearchEngine, TextSearchResult, DEFAULT_EGRESS_GUARD_MIN_BYTES, DEFAULT_EGRESS_GUARD_THRESHOLD, DEFAULT_EGRESS_GUARD_TIMEOUT, };
pub use egress_vault::{ BlockReason, BoxFuture, CompiledPattern, EgressClassification, EgressClassifier, EgressVault, EgressVaultError, EntityRecognizer, NoOpRecognizer, PolicyCategory, SurrogateVault, MAX_CLASSIFY_PAYLOAD_BYTES, };
pub use error::EgressError;
pub use guarded_payload::{GuardedPayload, Sanitized, Unsanitized};
pub use processing_registry::{ generate_registry, render_markdown, ProcessingActivityRecord, ProcessingRegistryExport, ProcessorRole, TenantId, };
```

### Schnittstellen — Public Traits (6)

**`EgressClassifier`** — *Schnittstelle für Downstream-Crates (`contextra-mcp`, `contextra-router`).* (`src/egress_vault.rs`)

```rust
pub trait EgressClassifier : Send + Sync {
/// Klassifiziert einen Text-Payload für den Egress-Export.
    fn classify<'a>(&'a self, payload: &'a str) -> BoxFuture<'a, EgressClassification>;

    /// Classifies a `TenantScoped` payload for egress export.
    fn classify_scoped<'a>(
        &'a self,
        payload: contextra_types::TenantScoped<&'a str>,
        expected_tenant_id: &'a contextra_types::TenantId,
    ) -> BoxFuture<'a, EgressClassification> {
        Box::pin(async move {
            match payload.into_inner_checked(expected_tenant_id) {
                Ok(unpacked) => self.classify(unpacked).await,
                Err(err) => EgressClassification::Block(BlockReason::PolicyDenied(err.to_string())),
            }
        })
    }
}
```

**`EgressClassifierTrace`** — *Erweiterter Trait für Egress-Klassifikatoren mit kryptografischem Audit-Trace.* (`src/audit_trace.rs`)

```rust
pub trait EgressClassifierTrace : EgressClassifier {
/// Klassifiziert einen Payload und liefert sowohl das Klassifikationsergebnis
    /// als auch einen 32-Byte-BLAKE3-Hash-Trace.
    fn classify_with_trace<'a>(
        &'a self,
        payload: &'a str,
        clock: &'a dyn Clock,
    ) -> BoxFuture<'a, (EgressClassification, [u8; 32])>;
}
```

**`EgressGuardCheck`** — *Trait contract for Layer 4 bulk-exfiltration checks via `EgressGuard`.* (`src/egress_gateway.rs`)

```rust
pub trait EgressGuardCheck : Send + Sync {
/// Evaluates the outbound payload against vector index / similarity models.
    fn check<'a>(&'a self, payload: &'a str) -> BoxFuture<'a, EgressClassification>;

    /// Evaluates a `TenantScoped` outbound payload for bulk exfiltration.
    fn check_scoped<'a>(
        &'a self,
        payload: contextra_types::TenantScoped<&'a str>,
        expected_tenant_id: &'a contextra_types::TenantId,
    ) -> BoxFuture<'a, EgressClassification> {
        Box::pin(async move {
            match payload.into_inner_checked(expected_tenant_id) {
                Ok(unpacked) => self.check(unpacked).await,
                Err(err) => EgressClassification::Block(BlockReason::PolicyDenied(err.to_string())),
            }
        })
    }
}
```

**`EntityRecognizer`** — *Trait for recognizing entities (NER) in text without creating a direct dependency on heavy embedding or ML crates (DAG-neutral interface).* (`src/egress_vault.rs`)

```rust
pub trait EntityRecognizer : Send + Sync {
/// Recognizes entities in the text and returns byte ranges and category labels.
    fn recognize(&self, text: &str) -> Vec<(std::ops::Range<usize>, String)>;
}
```

**`InjectionDetector`** — *Trait contract for checking prompt injection in cloud responses without depending on Ring 4 crates.* (`src/egress_gateway.rs`)

```rust
pub trait InjectionDetector : Send + Sync {
fn detect(&self, text: &str) -> Option<String>;
}
```

**`TextSearchEngine`** — *Trait for text similarity search without depending on `contextra-db::Collection` (Ring 3 DB).* (`src/egress_guard.rs`)

```rust
pub trait TextSearchEngine : Send + Sync {
fn search_text<'a>(
        &'a self,
        text: &'a str,
        limit: usize,
    ) -> BoxFuture<'a, Result<Vec<TextSearchResult>, String>>;
}
```

### Öffentliche Datentypen — Structs (19)

| Struct | Felder / Form | Beschreibung | Datei |
|---|---|---|---|
| `AvvContext` | `{ /// Name oder Bezeichnung des Verantwortlichen (Auftraggeber). pub controller_name: String, /// Name oder Bezeichnung des Auftragsverarbeiters (Auftragnehmer).… }` | Kontextdaten für die Erstellung eines Auftragsverarbeitungsvertrags (AVV). | `src/avv_generator.rs` |
| `BulkExfiltrationDetector` | `{ pub max_bytes_per_window: usize, pub window: Duration, sessions: HashMap<SessionId, Arc<Mutex<SlidingWindowCounter>>>, }` | Volume and time-window based rate limiter for cloud egress requests (Layer 4). | `src/bulk_exfiltration_detector.rs` |
| `CloudQueryRequest` | `{ pub query: String, pub collection: Option<String>, pub max_results: Option<usize>, }` | — | `src/egress_gateway.rs` |
| `CloudQueryResponse` | `{ pub status: String, pub query: String, pub abstracted: bool, pub results: Vec<Value>, pub abstraction_notice: Option<String>, }` | — | `src/egress_gateway.rs` |
| `CloudResponseRehydrator` | `{ vault_map: HashMap<String, String>, allowed_tokens: Option<HashSet<String>>, }` | Inbound Layer 5 Re-Hydrator replacing surrogate tokens with original vault entities. | `src/egress_gateway.rs` |
| `CompiledPattern` | `{ /// Bezeichner oder Originalmuster. pub name: String, /// Kompiliertes Regex-Objekt. pub regex: regex::Regex, /// Optionale Nachprüfung (z. B. Prüfsumme) nach… }` | Ein vorkompiliertes Regex-Muster für die Egress-Klassifikation. | `src/egress_vault.rs` |
| `ContextEditAuditRecord` | `{ /// BLAKE3-Hash des vorherigen Audit-Records in der Kette (32 Bytes). pub previous_hash: [u8; 32], /// Eigenadresse: BLAKE3-Hash dieses Datensatzes über alle F… }` | Nachvollziehbarer, kryptografisch verketteter Audit-Datensatz für eine Änderung am temporären Agenten-Kontext. **Sicherheitsgarantie:** Enthält keine… | `src/context_edit_audit.rs` |
| `ContextEditInput` | `{ /// Eindeutige Kennung des ausführenden Agenten-Tasks. pub task_id: String, /// Mandanten-Identifikator (Sicherheitstrennung). pub tenant_id: TenantId, /// Art… }` | Eingabe-Parameter zur Erstellung eines Audit-Datensatzes für eine Kontext-Änderung. | `src/context_edit_audit.rs` |
| `EgressGuard` | `{ search_engine: Arc<dyn TextSearchEngine>, threshold: f32, min_bytes: usize, timeout: Duration, }` | Layer-4 EgressGuard zur Erkennung und Blockierung von Bulk-Exfiltrationen. Baut auf einem Text-Sucher (`TextSearchEngine`) auf. Outbound-Payloads mit… | `src/egress_guard.rs` |
| `EgressVault` | `{ patterns: Arc<Vec<CompiledPattern>>, regex_set: Arc<regex::RegexSet>, timeout: Duration, surrogate_vault: Arc<SurrogateVault>, }` | Cloud Egress Vault zur Abhandlung und Bündelung von Layer-1-Klassifikationen. | `src/egress_vault.rs` |
| `GuardedPayload` | `{ pub(crate) inner: String, pub(crate) session_id: String, _state: PhantomData<S>, }` | Type-State-Wrapper für Cloud-Egress-Payloads. Ein `GuardedPayload<Sanitized>` kann NUR durch den vollständigen Durchlauf aller 5 Egress-Guard-Layer k… | `src/guarded_payload.rs` |
| `NoOpRecognizer` | `(tuple/unit)` | A default no-op entity recognizer that detects no entities. | `src/egress_vault.rs` |
| `ProcessingActivityRecord` | `{ /// Bezeichnung der Verarbeitungstätigkeit. pub activity_name: String, /// Rolle (Verantwortlicher oder Auftragsverarbeiter). pub controller_or_processor_role:… }` | Ein Datensatz einer Verarbeitungstätigkeit gemäß Art. 30 Abs. 1 DSGVO. | `src/processing_registry.rs` |
| `ProcessingRegistryExport` | `{ /// Liste der erfassten Verarbeitungstätigkeiten. pub records: Vec<ProcessingActivityRecord>, }` | Strukturierte Export-Repräsentation des Verzeichnisses von Verarbeitungstätigkeiten. | `src/processing_registry.rs` |
| `Sanitized` | `(tuple/unit)` | Marker-Typ: Payload hat alle 5 Egress-Guard-Layer erfolgreich durchlaufen. | `src/guarded_payload.rs` |
| `SessionId` | `(tuple/unit)` | Session identifier for grouping egress rate-limiting buckets. | `src/bulk_exfiltration_detector.rs` |
| `SurrogateVault` | `{ session_salt: [u8; 16], map: Mutex<HashMap<String, String>>, }` | Session-bound vault storing bidirectional mappings between original PII/entities and generated surrogates. Material and mappings are zeroized upon dr… | `src/egress_vault.rs` |
| `TextSearchResult` | `{ pub id: String, pub score: f32, }` | Simplified candidate result from text similarity search. | `src/egress_guard.rs` |
| `Unsanitized` | `(tuple/unit)` | Marker-Typ: Payload noch nicht durch alle Egress-Guard-Layer gelaufen. | `src/guarded_payload.rs` |

### Öffentliche Datentypen — Enums (10)

| Enum | Varianten | Beschreibung | Datei |
|---|---|---|---|
| `BlockReason` | `/// Ein sensibles Muster (PII, Secret, API-Key etc.) wurde erkannt. SensitivePattern(String), /// Richtlinie untersagt die Übertragung (lokal oder allgemein). PolicyDenied(String), /// Spezifische Cl…` | Grund für das Blockieren einer Egress-Anfrage. | `src/egress_vault.rs` |
| `BulkExfiltrationOutcome` | `/// Request payload byte volume is within allowed limits for the active time window. Allow, /// Request cumulative payload byte volume exceeds maximum allowed limit in active window. Block { /// Tota…` | Outcome of evaluating outbound bulk payload volume against session limits. | `src/bulk_exfiltration_detector.rs` |
| `ContextEditAuditError` | `/// Manipulationserkennung: Die Audit-Kette ist ab dem angegebenen Index fehlerhaft oder manipuliert. #[error("Audit chain tampered or invalid at index {index}")] ChainTampered { index: usize },` | Lokaler Fehlertyp für Audit-Ketten-Verifizierungsfehler. | `src/context_edit_audit.rs` |
| `ContextEditKind` | `/// Anfügen von neuem Inhalt an den Kontext. Append, /// Ersetzen von bestehendem Inhalt im Kontext. Replace, /// Entfernen von Inhalt aus dem Kontext. Remove, /// Zurücksetzen des Kontexts auf einen…` | Art der Kontext-Änderung am temporären Agenten-Scratchpad. | `src/context_edit_audit.rs` |
| `EgressClassification` | `/// Übertragung zulässig. Allow, /// Übertragung blockiert mit Grund. Block(BlockReason),` | Klassifikationsergebnis der Egress-Prüfung. | `src/egress_vault.rs` |
| `EgressError` | `#[error("Invalid parameter: {0}")] InvalidParams(String), #[error("Policy violation: {0}")] PolicyViolation(String), #[error("Internal egress error: {0}")] InternalError(String), #[error("Surrogate t…` | — | `src/error.rs` |
| `EgressVaultError` | `#[error("Invalid pattern '{pattern}': {reason}")] InvalidPattern { pattern: String, reason: String }, #[error("Payload size exceeds limit: {size} bytes > {limit} limit")] PayloadTooLarge { size: usiz…` | Fehler beim Erstellen oder Laden des EgressVaults. | `src/egress_vault.rs` |
| `PatternValidator` | `/// Luhn checksum validation (credit card numbers). Luhn, /// ISO 7064 MOD 97-10 checksum validation (IBAN). IbanMod97,` | Optional validator applied post-regex match to confirm checksum-backed patterns. | `src/egress_vault.rs` |
| `PolicyCategory` | `/// Rein lokaler Lese- oder Speicherzugriff innerhalb der Vertrauensgrenze. LocalAccess, /// Externe Cloud-Egress-Übertragung (Daten verlassen die lokale Vertrauensgrenze). CloudEgress,` | Policy-Kategorie für Audit-Logging und Zugriffskontrolle. | `src/egress_vault.rs` |
| `ProcessorRole` | `/// Verantwortlicher (Art. 4 Nr. 7 DSGVO). Controller, /// Auftragsverarbeiter (Art. 4 Nr. 8 DSGVO). Processor,` | Rolle bezüglich der Verarbeitungstätigkeit gemäß DSGVO. | `src/processing_registry.rs` |

### Typ-Aliase (3)

| Alias | Definition | Beschreibung |
|---|---|---|
| `BoxFuture` | `type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>` | Type alias for boxed dyn futures in `EgressClassifier` trait to ensure dyn compatibility. |
| `DefaultEgressClassifier` | `type DefaultEgressClassifier= EgressVault` | — |
| `TenantId` | `type TenantId= String` | Identifikator eines Mandanten im System. |

### Konstanten (7)

| Konstante | Typ | Beschreibung |
|---|---|---|
| `DEFAULT_EGRESS_GUARD_MIN_BYTES` | `usize` | Minimum Byte-Länge des Payloads, ab der die HNSW-Prüfung durchgeführt wird (128 Bytes). |
| `DEFAULT_EGRESS_GUARD_THRESHOLD` | `f32` | Standard-Schwellwert für HNSW Cosine Similarity Match (0.85). |
| `DEFAULT_EGRESS_GUARD_TIMEOUT` | `Duration` | Standard-Timeout für EgressGuard Vector-Search (200 ms). |
| `DEFAULT_TIMEOUT` | `Duration` | Standard-Timeout für Klassifikationsprüfungen (100 ms). |
| `MAX_CLASSIFY_PAYLOAD_BYTES` | `usize` | Maximale zulässige Payload-Länge in Bytes für Layer-1-Klassifikation. |
| `MAX_SEARCH_QUERY_BYTES` | `usize` | Maximum allowed search query length in bytes (64 KB). |
| `SURROGATE_PREFIX` | `&str` | Common prefix for session-bound surrogate tokens. |

### Öffentliche Funktionen (74)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn compute_audit_trace(payload: &str, rule_id: &str, nanos: u64) -> [u8` | Berechnet einen deterministischen 32-Byte-BLAKE3-Hash-Trace für eine Klassifikationsentscheidung. Segmente werden in fester Byte-Reihenfolg… | `src/audit_trace.rs` |
| `pub fn extract_rule_id(classification: &EgressClassification) -> String` | Extrahiert eine stabile, nicht-leere Regel-Kennung aus einer `EgressClassification`. | `src/audit_trace.rs` |
| `pub fn generate_avv_draft(ctx: &AvvContext) -> String` | Generiert einen vollständigen AVV-Entwurf im Markdown-Format. Der Entwurf enthält alle nach Art. 28 DSGVO geforderten Abschnitte und refere… | `src/avv_generator.rs` |
| `pub fn new(max_bytes_per_window: usize, window: Duration) -> Self` | Creates a new `BulkExfiltrationDetector` configured with `max_bytes_per_window` and `window` duration. | `src/bulk_exfiltration_detector.rs` |
| `pub fn record_and_check( &self, session: SessionId, payload_len: usize, ) -> BulkExfiltrationOutcome` | Registers `payload_len` bytes for `session` and evaluates whether the volume limit in the current sliding time window was exceeded. Thread-… | `src/bulk_exfiltration_detector.rs` |
| `pub fn build_context_edit_audit_record( previous_hash: [u8; 32], input: ContextEditInput, clock: &dyn Clock, ) -> ContextEditAuditRecord` | Konstruiert deterministisch einen neuen [`ContextEditAuditRecord`] mit Hash-Verkettung. Der Zeitstempel wird ausschließlich über den injizi… | `src/context_edit_audit.rs` |
| `pub fn compute_record_hash( previous_hash: [u8; 32], input: &ContextEditInput, recorded_at_tx: u64, ) -> [u8` | Berechnet den deterministischen BLAKE3-Hash über alle Datenfelder eines Audit-Records. | `src/context_edit_audit.rs` |
| `pub fn render_audit_line(record: &ContextEditAuditRecord) -> String` | Formatiert einen Audit-Datensatz in eine einzeilige, menschenlesbare Log-Zeile. ### Format `"task={task_id} tenant={tenant_id} kind={kind:?… | `src/context_edit_audit.rs` |
| `pub fn verify_audit_chain(chain: &[ContextEditAuditRecord]) -> Result<(), ContextEditAuditError>` | Verifiziert die Integrität und Hash-Verkettung einer Reihe von Audit-Records. Gibt `Ok(())` zurück, wenn die Kette intakt ist, oder `Err(Co… | `src/context_edit_audit.rs` |
| `pub fn check_bulk_exfiltration( detector: &BulkExfiltrationDetector, session: SessionId, payload: &str, ) -> Result<(), EgressError>` | Evaluates a query payload against the `BulkExfiltrationDetector` for a specific `SessionId`. | `src/egress_gateway.rs` |
| `pub fn extract_surrogate_tokens(sanitized_request: &str) -> HashSet<String>` | Extracts all surrogate tokens matching `[USER_ENTITY_[0-9a-fA-F]{4}]` from the given text. Guarantees UTF-8 safety and zero panics. | `src/egress_gateway.rs` |
| `pub fn from_vault(vault: &EgressVault, surrogates: impl IntoIterator<Item = String>) -> Self` | Creates a new `CloudResponseRehydrator` by fetching surrogate mappings from an `EgressVault`. | `src/egress_gateway.rs` |
| `pub async fn guard_and_sanitize_payload( payload: GuardedPayload<Unsanitized>, classifier: &dyn EgressClassifier, egress_guard: Option<&dyn EgressGuardCheck>, ) -> Result<GuardedPayload<Sanitized>, EgressError>` | Evaluates an unsanitized payload against egress security classifiers and returns a `GuardedPayload<Sanitized>` upon successful validation a… | `src/egress_gateway.rs` |
| `pub async fn guard_and_sanitize_payload_scoped( payload: contextra_types::TenantScoped<GuardedPayload<Unsanitized>>, expected_tenant_id: &contextra_types::TenantId, vault: &EgressVault, recognizer: &dyn crate::egress_vault::EntityRecognizer, ) -> Result<conte…` | Evaluates an unsanitized tenant-scoped payload against egress security classifiers and returns a `TenantScoped<GuardedPayload<Sanitized>>`… | `src/egress_gateway.rs` |
| `pub async fn handle_cloud_query( request: CloudQueryRequest, classifier: &dyn EgressClassifier, ) -> Result<CloudQueryResponse, EgressError>` | — | `src/egress_gateway.rs` |
| `pub async fn handle_cloud_query_scoped( request: contextra_types::TenantScoped<CloudQueryRequest>, expected_tenant_id: &contextra_types::TenantId, classifier: &dyn EgressClassifier, ) -> Result<CloudQueryResponse, EgressError>` | Handles a cloud query request bound to a tenant scope, enforcing matching `TenantId` before execution. | `src/egress_gateway.rs` |
| `pub async fn handle_cloud_query_scoped_with_bulk_detector( request: contextra_types::TenantScoped<CloudQueryRequest>, expected_tenant_id: &contextra_types::TenantId, classifier: &dyn EgressClassifier, egress_guard: Option<&dyn EgressGuardCheck>, bulk_detector…` | Extended cloud query handler enforcing tenant scope matching, DLP classifier, Layer 4 guard, and bulk volume detector. | `src/egress_gateway.rs` |
| `pub async fn handle_cloud_query_scoped_with_guard( request: contextra_types::TenantScoped<CloudQueryRequest>, expected_tenant_id: &contextra_types::TenantId, classifier: &dyn EgressClassifier, egress_guard: Option<&dyn EgressGuardCheck>, ) -> Result<CloudQuer…` | Extended cloud query handler enforcing tenant scope matching alongside DLP classifier and Layer 4 guard. | `src/egress_gateway.rs` |
| `pub async fn handle_cloud_query_with_bulk_detector( request: CloudQueryRequest, classifier: &dyn EgressClassifier, egress_guard: Option<&dyn EgressGuardCheck>, bulk_detector: Option<(&BulkExfiltrationDetector, SessionId)>, ) -> Result<CloudQueryResponse, Egre…` | Extended cloud query handler including additive Layer 4 bulk volume check along with DLP classifier and similarity guard. | `src/egress_gateway.rs` |
| `pub async fn handle_cloud_query_with_guard( request: CloudQueryRequest, classifier: &dyn EgressClassifier, egress_guard: Option<&dyn EgressGuardCheck>, ) -> Result<CloudQueryResponse, EgressError>` | — | `src/egress_gateway.rs` |
| `pub fn new(vault_map: HashMap<String, String>) -> Self` | Creates a new `CloudResponseRehydrator` with the provided surrogate-to-entity map. | `src/egress_gateway.rs` |
| `pub fn pii_vault_forces_crypto_shred(is_pii_match: bool, is_memory_only: bool) -> bool` | Evaluates whether PII vault data forces CryptoShred mode for an affected document (INV-COLLECTION-PROFILE-3). | `src/egress_gateway.rs` |
| `pub async fn process_cloud_response( raw_response: &str, injection_guard: &dyn InjectionDetector, rehydrator: &CloudResponseRehydrator, ) -> Result<String, EgressError>` | Processes inbound responses from cloud LLMs by checking for prompt injections and rehydrating surrogate tokens back to original values. | `src/egress_gateway.rs` |
| `pub fn rehydrate(&self, cloud_response_text: &str) -> String` | Rehydrates surrogate tokens matching `[USER_ENTITY_[0-9a-fA-F]{4}]` back to original entity values. Unknown or missing surrogate tokens rem… | `src/egress_gateway.rs` |
| `pub fn resolve_effective_kv_delete_mode<D, K>( is_pii_match: bool, durability_mode: D, preset_kv_delete_mode: K, crypto_shred_mode: K, ) -> K where D: std::fmt::Debug, K: Copy,` | Read-only check for PII Vault coupling (Spec B.1.8 / INV-COLLECTION-PROFILE-3): If a document contains a PII match AND `durability_mode` is… | `src/egress_gateway.rs` |
| `pub fn scoped_to_request(mut self, allowed_tokens: impl IntoIterator<Item = String>) -> Self` | Restricts rehydration to only tokens present in `allowed_tokens`. | `src/egress_gateway.rs` |
| `pub async fn check(&self, payload: &str) -> EgressClassification` | Prüft einen Egress-Payload auf mögliche Bulk-Exfiltration gegen die Collection. Gemäß **v17 Teil 10.3** (Priorität P1, "Fail-Closed-Randfal… | `src/egress_guard.rs` |
| `pub async fn check_scoped( &self, payload: contextra_types::TenantScoped<&str>, expected_tenant_id: &contextra_types::TenantId, ) -> EgressClassification` | — | `src/egress_guard.rs` |
| `pub fn min_bytes(&self) -> usize` | Gibt die minimale Payload-Länge in Bytes zurück. | `src/egress_guard.rs` |
| `pub fn new(search_engine: Arc<dyn TextSearchEngine>, threshold: f32, min_bytes: usize) -> Self` | Erstellt eine neue `EgressGuard`-Instanz mit Standard-Timeout (200 ms). | `src/egress_guard.rs` |
| `pub fn threshold(&self) -> f32` | Gibt den gesetzten Schwellwert zurück. | `src/egress_guard.rs` |
| `pub fn timeout(&self) -> Duration` | Gibt das gesetzte Timeout zurück. | `src/egress_guard.rs` |
| `pub fn with_timeout(mut self, timeout: Duration) -> Self` | Setzt ein benutzerdefiniertes Timeout für die Index-Abfrage. | `src/egress_guard.rs` |
| `pub fn category(&self) -> PolicyCategory` | Gibt die zugehörige Policy-Kategorie des Blockierungsgrunds zurück. | `src/egress_vault.rs` |
| `pub async fn classify_layer1( payload: &str, patterns: &[CompiledPattern], timeout: Duration, ) -> EgressClassification` | Führt eine Layer-1-Regex-Klassifikation auf dem Payload mit neuem Task und hartem Timeout aus. Invariante: **Fail-Closed**. Bei Timeout, Pa… | `src/egress_vault.rs` |
| `pub async fn classify_layer1_arc( payload: &str, patterns: Arc<Vec<CompiledPattern>>, _regex_set: Arc<regex::RegexSet>, timeout: Duration, ) -> EgressClassification` | Optimierter Ausführungspfad für `classify_layer1` unter Wiederverwendung von vorkompilierten `Arc`-Mengen. | `src/egress_vault.rs` |
| `pub async fn classify_scoped( &self, payload: contextra_types::TenantScoped<&str>, expected_tenant_id: &contextra_types::TenantId, ) -> EgressClassification` | Classifies a `TenantScoped` payload for egress export only if the bound `TenantId` matches `expected_tenant_id`. If a tenant scope violatio… | `src/egress_vault.rs` |
| `pub fn default_patterns() -> Vec<String>` | Standard-DLP-Muster für Egress-Klassifikationen (Secrets, API-Keys, PII, E-Mail, IBAN, JWT, PEM, Credit Card, Phone, IPv4). | `src/egress_vault.rs` |
| `pub fn generate_surrogate(&self, entity_text: &str) -> Result<String, EgressVaultError>` | Generates a session-stable surrogate identifier for `entity_text` and stores the surrogate -> entity mapping. # Backward Compatibility & To… | `src/egress_vault.rs` |
| `pub fn get_entity(&self, surrogate: &str) -> Option<String>` | Retrieves the original entity text for a given surrogate key if present. | `src/egress_vault.rs` |
| `pub fn get_entity(&self, surrogate: &str) -> Option<String>` | Retrieves the original entity text for a given surrogate key if present in the vault. | `src/egress_vault.rs` |
| `pub fn is_empty(&self) -> bool` | Returns true if the vault contains no surrogate mappings. | `src/egress_vault.rs` |
| `pub fn is_iban_valid(iban_str: &str) -> bool` | Validates an IBAN candidate using the ISO 7064 MOD 97-10 algorithm. | `src/egress_vault.rs` |
| `pub fn is_luhn_valid(number_str: &str) -> bool` | Validates a credit card number candidate using the Luhn algorithm (ISO/IEC 7812-1 MOD 10). | `src/egress_vault.rs` |
| `pub fn is_valid_match(&self, matched_text: &str) -> bool` | Prüft, ob ein übereinstimmender Substring das verknüpfte Prüfsummenverfahren erfüllt. | `src/egress_vault.rs` |
| `pub fn len(&self) -> usize` | Returns the current count of stored surrogate mappings. | `src/egress_vault.rs` |
| `pub fn new(session_salt: [u8; 16]) -> Self` | Creates a new `SurrogateVault` with the specified session salt. | `src/egress_vault.rs` |
| `pub fn new(name: impl Into<String>, pattern: &str) -> Result<Self, EgressVaultError>` | Erstellt ein neues `CompiledPattern`. | `src/egress_vault.rs` |
| `pub fn new(patterns: Vec<String>) -> Result<Self, EgressVaultError>` | Erstellt eine neue `EgressVault`-Instanz aus einer Liste von Regex-Patterns. Jedem Pattern wird eine opake Regel-ID ("R-001", "R-002", ...)… | `src/egress_vault.rs` |
| `pub fn normalize_payload(text: &str) -> String` | Normalizes a payload string prior to Layer-1 classification by stripping whitespace and hyphen delimiters bridging alphanumeric characters… | `src/egress_vault.rs` |
| `pub fn normalize_payload_for_dlp(payload: &str) -> String` | Normalizes a payload for DLP pattern matching by removing common delimiters (spaces, hyphens) occurring between digits or alphanumeric char… | `src/egress_vault.rs` |
| `pub fn patterns(&self) -> &[CompiledPattern]` | Gibt die kompilierten Muster zurück. | `src/egress_vault.rs` |
| `pub fn policy_category(&self) -> PolicyCategory` | Gibt die Policy-Kategorie dieser Vault-Instanz für Cloud-Egress-Klassifizierungen zurück. | `src/egress_vault.rs` |
| `pub fn random_salt() -> Self` | Creates a new `SurrogateVault` with a cryptographically secure random session salt. | `src/egress_vault.rs` |
| `pub fn sanitize_and_vault( &self, payload: &str, recognizer: &dyn EntityRecognizer, ) -> Result<(String, usize), EgressVaultError>` | Tokenisiert und ersetzt erkannte strukturierte Regex-Muster sowie unstrukturierte NER-Entitäten durch sitzungsstabile Surrogate und speiche… | `src/egress_vault.rs` |
| `pub fn sanitize_and_vault_scoped( &self, payload: contextra_types::TenantScoped<&str>, expected_tenant_id: &contextra_types::TenantId, recognizer: &dyn EntityRecognizer, ) -> Result<(contextra_types::TenantScoped<String>, usize), EgressVaultError>` | Tokenizes and replaces recognized entities in a `TenantScoped` payload only if the bound `TenantId` matches `expected_tenant_id`. Re-wraps… | `src/egress_vault.rs` |
| `pub fn surrogate_vault(&self) -> &Arc<SurrogateVault>` | Gibt eine Referenz auf den verknüpften `SurrogateVault` zurück. | `src/egress_vault.rs` |
| `pub fn timeout(&self) -> Duration` | Gibt das gesetzte Timeout zurück. | `src/egress_vault.rs` |
| `pub fn try_default() -> Result<Self, EgressVaultError>` | Versucht, eine `EgressVault`-Instanz mit den Standard-DLP-Mustern zu erstellen. | `src/egress_vault.rs` |
| `pub fn validate_iban_mod97(iban: &str) -> bool` | Validates an IBAN candidate string using ISO 7064 MOD 97-10 checksum algorithm. | `src/egress_vault.rs` |
| `pub fn validate_luhn(number: &str) -> bool` | Validates a credit card candidate number using the Luhn checksum algorithm (Modulus 10). | `src/egress_vault.rs` |
| `pub fn with_surrogate_vault(mut self, surrogate_vault: Arc<SurrogateVault>) -> Self` | Setzt einen benutzerdefinierten `SurrogateVault`. | `src/egress_vault.rs` |
| `pub fn with_timeout(mut self, timeout: Duration) -> Self` | Setzt ein benutzerdefiniertes Timeout für Klassifikationsabfragen. | `src/egress_vault.rs` |
| `pub fn with_validator(mut self, validator: PatternValidator) -> Self` | Setzt ein spezifisches `PatternValidator`-Prüfverfahren. | `src/egress_vault.rs` |
| `pub fn internal_error(msg: impl Into<String>) -> Self` | — | `src/error.rs` |
| `pub fn invalid_params(msg: impl Into<String>) -> Self` | — | `src/error.rs` |
| `pub fn policy_violation(msg: impl Into<String>) -> Self` | — | `src/error.rs` |
| `pub fn from_sanitized(sanitized: String, session_id: String) -> Self` | Nur von `egress_gateway.rs` aufzurufen nach Durchlauf aller 5 Layer. | `src/guarded_payload.rs` |
| `pub fn into_inner(self) -> String` | Gibt den sanitizierten Payload-String zurück. | `src/guarded_payload.rs` |
| `pub fn new(raw: String, session_id: String) -> Self` | Erstellt einen neuen unsanitizierten Payload (Eingang aus MCP-Client). | `src/guarded_payload.rs` |
| `pub fn session_id(&self) -> &str` | Gibt die Session-ID zurück. | `src/guarded_payload.rs` |
| `pub fn as_str(&self) -> &'static str` | Liefert die deutschsprachige Bezeichnung der Rolle. | `src/processing_registry.rs` |
| `pub fn generate_registry(records: &[ProcessingActivityRecord]) -> ProcessingRegistryExport` | Erzeugt aus einer Reihe von Datensätzen eine strukturierte Export-Repräsentation. | `src/processing_registry.rs` |
| `pub fn render_markdown(export: &ProcessingRegistryExport) -> String` | Erzeugt eine druckfertige, deutschsprachige Markdown-Tabelle aus dem Export. Die Tabelle enthält eine Zeile pro `ProcessingActivityRecord`… | `src/processing_registry.rs` |


---

## `contextra-router`

| Attribut | Wert |
|---|---|
| **Ring** | 3 — Ring 3 — Orchestration & Cognition |
| **Status** | In Migration |
| **Zweck** | SLM-Profil-Routing und MCP-Dispatch. |
| **Unsafe-Code** | Nein (`#![forbid(unsafe_code)]`) |
| **Interne Abhängigkeiten** | `contextra-adapt`, `contextra-ports`, `contextra-privacy`, `contextra-types`, `contextra-wire` |
| **Externe Abhängigkeiten (Auswahl)** | `arc-swap`, `parking_lot`, `serde`, `serde_json`, `shlex`, `thiserror`, `tokio`, `tracing` |

### Modul- und Re-Export-Oberfläche (lib.rs)

```rust
pub mod arm_registry;
pub mod dispatch;
pub mod outcome;
pub mod ports_local;
pub mod profile;
pub mod router;
pub mod serde_helpers;
pub mod fc_ts_dispatch;
pub mod routing_strategy;
pub mod transport;
pub use contextra_adapt::bandit;
pub use contextra_adapt::lyapunov;
pub use contextra_adapt::offpolicy;
pub use contextra_privacy::guarded_payload;
pub use arm_registry::{ArmRegistry, ArmRegistryError};
pub use contextra_adapt::{DriftReason, LyapunovDriftWatcher, LyapunovResult};
pub use contextra_adapt::{OffPolicyEvaluator, OffPolicyStats};
pub use dispatch::{dispatch_to_slm, route_and_dispatch_to_slm};
pub use outcome::{DecisionId, DecisionIdGenerator, RoutingOutcome};
pub use profile::SlmProfile;
pub use router::{DefaultRouterEngine, RouterEngine, RoutingDecision};
pub use contextra_adapt::{BanditImplementation, BanditPolicy, BanditProfileState};
pub use contextra_privacy::guarded_payload::{GuardedPayload, Sanitized, Unsanitized};
pub use fc_ts_dispatch::{deterministic_fc_ts_rng, select_profile_fc_ts, FcTsDispatchError};
pub use routing_strategy::RoutingStrategy;
pub use transport::Transport;
```

### Feature-Flags

| Flag | Aktiviert |
|---|---|
| `default` | `[]` |
| `bandit-routing` | `["contextra-adapt/bandit-routing"]` |
| `cloud-egress-guard` | `[]` |
| `flow-corrected-thompson` | `["contextra-adapt/flow-corrected-thompson"]` |

### Öffentliche Datentypen — Structs (6)

| Struct | Felder / Form | Beschreibung | Datei |
|---|---|---|---|
| `ArmRegistry` | `{ arms: [RetrievalStrategy; 5], }` | Verlustfreie, deterministische Abbildung Arm-Index <-> RetrievalStrategy. Lebt in contextra-router (wo Arme registriert werden), NICHT im BanditPolic… | `src/arm_registry.rs` |
| `ConformalCalibrator` | `{ /// Target error rate (e.g., 0.05 for 95% coverage). pub alpha: f32, /// Adaptation rate for online quantile updates. pub gamma: f32, /// Current calibrated co… }` | Conformal-inspirierte Kalibrierung (Coverage-Garantie erst mit record_outcome()). Basierend auf quantile-basierter Schwellenwert-Adaption (Gibbs & Ca… | `src/profile.rs` |
| `DecisionId` | `(tuple/unit)` | Eindeutige ID einer Routing-Entscheidung. Wird von route() zurückgegeben und von record_outcome() konsumiert. | `src/outcome.rs` |
| `DecisionIdGenerator` | `{ next: AtomicU64, }` | Instanzgebundener Generator für monoton steigende `DecisionId`s (Spec §3 P29). | `src/outcome.rs` |
| `ProfileCalibrationState` | `{ /// Gesamtzahl der Routing-Entscheidungen für dieses Profil. pub times_selected: u64, /// Kumulierte Routing-Konfidenz (best_score / second_best_score). /// Hö… }` | Laufzeit-Kalibrierungsstatistik für ein SLM-Profil. Wird von RouterEngine verwaltet und nicht vom Aufrufer gesetzt. | `src/profile.rs` |
| `SlmProfile` | `{ /// Identifier name of the SLM (e.g. "coding-slm", "docs-slm"). pub name: String, /// MCP endpoint address or URI for client communication. pub mcp_endpoint: S… }` | Represents a Small Language Model (SLM) target and its domain expertise parameters. | `src/profile.rs` |

### Öffentliche Datentypen — Enums (6)

| Enum | Varianten | Beschreibung | Datei |
|---|---|---|---|
| `ArmRegistryError` | `/// Der angegebene Arm-Index liegt außerhalb des gültigen Bereichs (0..=4). #[error("arm index {0} out of range, expected 0..=4")] OutOfRange(u32),` | Fehlerklasse für ungültige Arm-Indizes — crate-lokal, P6-konform an Grenze konvertierbar. | `src/arm_registry.rs` |
| `FcTsDispatchError` | `#[error("FC-TS arm set is empty or misconfigured: {0}")] ArmSet(#[from] FcTsError), #[error("profile/arm count mismatch: {profiles} profiles vs {arms} arms")] ProfileArmMismatch { profiles: usize, ar…` | Fehler der FC-TS-Profilauswahl für contextra-router. | `src/fc_ts_dispatch.rs` |
| `QuantizationLevel` | `F16, Q8_0, Q4_K_M, #[default] Unknown,` | Quantization level of the underlying model execution path. | `src/profile.rs` |
| `RoutingOutcome` | `/// SLM hat die Anfrage vollständig und korrekt beantwortet. Success, /// SLM-Antwort war unzureichend — Eskalation zu größerem Modell war nötig. Escalated { escalated_to: String }, /// Nachgelagerte…` | Tatsächliches Ergebnis einer getroffenen Routing-Entscheidung. Wird vom Agent-Orchestrator NACH dem SLM-Aufruf geliefert. | `src/outcome.rs` |
| `RoutingStrategy` | `/// Deterministischer Kaskaden-Klassifikator (produktiv, kalibriert). #[default] Cascade, /// LinUCB Contextual Bandit (Feature `bandit-routing`, Default: off). #[cfg(feature = "bandit-routing")] Con…` | Auswahl-Strategie für SLM-Profil-Routing. # Invariante (§13.3) `ContextualBandit` bleibt Default-off bis `test_bandit_vs_cascade_regret_comparison` m… | `src/routing_strategy.rs` |
| `Transport` | `/// Standard: stdio JSON-RPC 2.0 zu lokalem SLM-Prozess (ADR-010). StdioMcp, /// Cloud-HTTP: Nur mit Feature `cloud-egress-guard` + `GuardedPayload<Sanitized>`. #[cfg(feature = "cloud-egress-guard")]…` | Transport-Kanal für SLM-Routing-Entscheidungen. # Invariante `HttpCloud` darf nur mit aktivem Feature `cloud-egress-guard` und einem `GuardedPayload<… | `src/transport.rs` |

### Öffentliche Funktionen (55)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn active_strategies(&self) -> Vec<(u32, RetrievalStrategy)>` | Liefert alle im Registry registrierten Arm-Indizes samt zugehörigen Retrieval-Strategien. | `src/arm_registry.rs` |
| `pub fn arm_for(&self, strategy: RetrievalStrategy) -> u32` | Liefert den Arm-Index (0..=4) für die gegebene `RetrievalStrategy`. Die Implementierung verwendet ein `match` über alle bekannten Varianten… | `src/arm_registry.rs` |
| `pub fn contains_strategy(&self, strategy: RetrievalStrategy) -> bool` | Prüft, ob eine `RetrievalStrategy` im Registry registriert und gültig auflösbar ist. | `src/arm_registry.rs` |
| `pub fn create_fc_ts_arm_set(&self, count: usize, dim: usize) -> Result<FcTsArmSet, FcTsError>` | Erzeugt ein `FcTsArmSet` mit `count` Armen der Dimension `dim`. | `src/arm_registry.rs` |
| `pub fn resolve_strategy( &self, strategy: RetrievalStrategy, ) -> Option<(u32, RetrievalStrategy)>` | Löse eine `RetrievalStrategy` deterministisch in ihr Arm-Index/Strategie-Paar auf. Nutzt intern `arm_for` und `strategy_for` zur Verifizier… | `src/arm_registry.rs` |
| `pub fn strategy_for(&self, arm: u32) -> Result<RetrievalStrategy, ArmRegistryError>` | Liefert die `RetrievalStrategy` für den gegebenen Arm-Index (0..=4). | `src/arm_registry.rs` |
| `pub async fn dispatch_to_slm(decision: &RoutingDecision) -> Result<String>` | Dispatches the prepared context from a [`RoutingDecision`] to the target SLM's MCP endpoint over stdio JSON-RPC 2.0 (ADR-010 compliant). Se… | `src/dispatch.rs` |
| `pub async fn route_and_dispatch_to_slm( router: &crate::router::RouterEngine, query_embedding: &[f32], query_text: &str, ) -> Result<String>` | Convenience-Hilfsfunktion, die über die [`crate::router::RouterEngine`] eine Routing-Entscheidung für die Query ermittelt und den maßgeschn… | `src/dispatch.rs` |
| `pub fn deterministic_fc_ts_rng(seed: u64) -> impl FcTsRng` | Deterministischer Default-RNG-Konstruktor für reproduzierbare FC-TS-Auswahl bei gegebenem Seed (z. B. abgeleitet aus einer Request-ID) — ka… | `src/fc_ts_dispatch.rs` |
| `pub fn select_profile_fc_ts( profile_names: &[String], arm_set: &FcTsArmSet, context: &[f32], rng: &mut dyn FcTsRng, ) -> Result<(usize, String), FcTsDispatchError>` | Wählt einen Profil-Index über FC-TS-Argmax-Sampling aus `arm_set` für den gegebenen Kontextvektor. `profile_names[i]` MUSS `arm_set.arms[i]… | `src/fc_ts_dispatch.rs` |
| `pub fn new(start: u64) -> Self` | Erstellt einen neuen Generator mit dem angegebenen Startwert. | `src/outcome.rs` |
| `pub fn next(&self) -> DecisionId` | Erzeugt die nächste monotone `DecisionId`. | `src/outcome.rs` |
| `pub fn non_conformity_score(&self) -> f32` | Non-Conformity-Score: 0.0 = perfekt, 1.0 = komplett falsch. Diese Werte sind Expertenschätzungen und sollten durch A/B-Tests kalibriert wer… | `src/outcome.rs` |
| `pub fn average_confidence(&self) -> f64` | Durchschnittliche Konfidenz über alle bisherigen Entscheidungen. | `src/profile.rs` |
| `pub fn check_and_invalidate_fingerprint(&mut self, active_fp: Option<&ConfigFingerprint>)` | Checks active fingerprint against recorded calibration fingerprint. Invalidates calibration stats immediately if active fingerprint is None… | `src/profile.rs` |
| `pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<HashSet<u64>, D::Error>` | — | `src/profile.rs` |
| `pub fn empirical_error_rate(&self) -> f32` | Returns the current empirical error rate. | `src/profile.rs` |
| `pub fn empirical_error_rate(&self) -> f32` | Liefert die empirische Fehlerrate des untergeordneten ConformalCalibrators. | `src/profile.rs` |
| `pub fn estimated_cost(&self) -> f32` | Returns the effective estimated cost for resource-aware fallback routing. If `resource_cost_estimate` is > 0.0, returns it; otherwise falls… | `src/profile.rs` |
| `pub fn is_calibrated(&self, active_fp: Option<&ConfigFingerprint>) -> bool` | Returns whether calibration is valid and active for the given active fingerprint. | `src/profile.rs` |
| `pub fn is_healthy(&self) -> bool` | Prüft, ob der Kalibrierungszustand gesunde Metriken aufweist. | `src/profile.rs` |
| `pub fn new( name: impl Into<String>, mcp_endpoint: impl Into<String>, domain_communities: impl IntoIterator<Item = u64>, token_budget: TokenBudget, min_relevance_score: f32, ) -> Self` | Creates a new `SlmProfile`. | `src/profile.rs` |
| `pub fn new(alpha: f32, gamma: f32, initial_threshold: f32) -> Self` | Creates a new `ConformalCalibrator` with specified error tolerance and adaptation rate. | `src/profile.rs` |
| `pub fn new(original_min_score: f32) -> Self` | — | `src/profile.rs` |
| `pub fn recalibrate_conformal(&mut self, non_conformity_score: f32) -> bool` | Recalibrates using the conformal calibrator. The non_conformity_score represents how poorly the SLM handled the last query. Returns true if… | `src/profile.rs` |
| `pub fn reset(&mut self)` | Setzt Kalibrierungsstate vollständig zurück. | `src/profile.rs` |
| `pub fn reset_window(&mut self)` | Resets statistics while preserving the current calibrated threshold. | `src/profile.rs` |
| `pub fn reset_window(&mut self)` | Setzt das Beobachtungsfenster des ConformalCalibrators zurück, behält aber den kalibrierten Schwellenwert bei. | `src/profile.rs` |
| `pub fn serialize<S: Serializer>(set: &HashSet<u64>, s: S) -> Result<S::Ok, S::Error>` | — | `src/profile.rs` |
| `pub fn try_new( name: impl Into<String>, mcp_endpoint: impl Into<String>, domain_communities: impl IntoIterator<Item = u64>, token_budget: TokenBudget, min_relevance_score: f32, ) -> Result<Self>` | Creates and validates a new `SlmProfile`. | `src/profile.rs` |
| `pub fn try_new_with_cost( name: impl Into<String>, mcp_endpoint: impl Into<String>, domain_communities: impl IntoIterator<Item = u64>, token_budget: TokenBudget, min_relevance_score: f32, resource_cost_estimate: f32, ) -> Result<Self>` | Creates and validates a new `SlmProfile` with an explicit resource cost estimate. | `src/profile.rs` |
| `pub fn update(&mut self, non_conformity_score: f32) -> bool` | Updates the conformal threshold based on a non-conformity score. The non-conformity score `s_t` measures how "surprising" the routing outco… | `src/profile.rs` |
| `pub fn validate(&self) -> Result<()>` | Validates `SlmProfile` parameters. | `src/profile.rs` |
| `pub fn with_fingerprint(mut self, fingerprint: ConfigFingerprint) -> Self` | Builder method to attach a specific `ConfigFingerprint` to this profile. | `src/profile.rs` |
| `pub fn with_resource_cost_estimate(mut self, resource_cost_estimate: f32) -> Self` | Builder method to attach an explicit resource cost estimate to this profile. | `src/profile.rs` |
| `pub async fn route( &self, query_embedding: &[f32], query_text: &str, ) -> Result<RoutingDecision>` | Routes a query with embedding and text to the best matching SLM profile. | `src/router/dispatch_core.rs` |
| `pub fn new( search_provider: Arc<dyn HybridSearchProvider>, community_resolver: Arc<dyn CommunityResolver>, context_preparer: Arc<dyn ContextPreparer>, profiles: Vec<SlmProfile>, calibration_store_path: Option<std::path::PathBuf>, ) -> Self` | Creates a new `RouterEngine` instance from decoupled ports. | `src/router/lifecycle.rs` |
| `pub fn profiles(&self) -> Vec<SlmProfile>` | Returns a copy of the active SLM profiles. | `src/router/lifecycle.rs` |
| `pub fn try_new( search_provider: Arc<dyn HybridSearchProvider>, community_resolver: Arc<dyn CommunityResolver>, context_preparer: Arc<dyn ContextPreparer>, profiles: Vec<SlmProfile>, calibration_store_path: Option<std::path::PathBuf>, ) -> Result<Self>` | Validates all profiles and creates a new `RouterEngine` instance. | `src/router/lifecycle.rs` |
| `pub fn try_update_profiles(&self, new_profiles: Vec<SlmProfile>) -> Result<()>` | Validates all profiles and updates configured SLM profiles at runtime (Hot-Reload). | `src/router/lifecycle.rs` |
| `pub fn update_profiles(&self, new_profiles: Vec<SlmProfile>)` | Dynamically updates configured SLM profiles at runtime (Hot-Reload). | `src/router/lifecycle.rs` |
| `pub fn with_initial_decision_id(mut self, start: u64) -> Self` | Konfiguriert den Startwert des instanzgebundenen DecisionIdGenerators. | `src/router/lifecycle.rs` |
| `pub fn with_routing_strategy( mut self, strategy: RoutingStrategy, epsilon: f32, seed: u64, ) -> Self` | Builder-Methode zur Konfiguration der Routing-Strategie und Bandit-Exploration. | `src/router/lifecycle.rs` |
| `pub fn bandit_decision_propensity(&self, id: DecisionId) -> Option<f32>` | Liefert die aufgezeichnete Logging-Propensity für eine ausstehende Bandit-Entscheidung. | `src/router/outcomes.rs` |
| `pub fn calibration_stats(&self) -> HashMap<String, ProfileCalibrationState>` | Gibt aktuelle Kalibrierungsstatistik für alle Profile zurück. | `src/router/outcomes.rs` |
| `pub fn drift_status(&self, profile_name: &str) -> Option<LyapunovResult>` | Gibt den aktuellen Lyapunov-Drift-Status für ein Profil zurück. | `src/router/outcomes.rs` |
| `pub fn overall_drift_status(&self) -> String` | Gives a human-readable summary string of Lyapunov drift status across active profile watchers. Priority order: "kritisch" > "warnung" > "st… | `src/router/outcomes.rs` |
| `pub fn pending_decision_count(&self) -> usize` | Anzahl offener (noch nicht mit record_outcome() abgeschlossener) Decisions. Sollte in normaler Laufzeit nahe 0 bleiben. | `src/router/outcomes.rs` |
| `pub fn record_outcome(&self, decision_id: DecisionId, outcome: RoutingOutcome) -> bool` | Muss vom Aufrufer (Agent-Loop) nach Abschluss des SLM-Aufrufs aufgerufen werden. Liefert das tatsächliche Ergebnis zurück und trainiert die… | `src/router/outcomes.rs` |
| `pub fn reset_all_calibration(&self)` | Setzt Kalibrierungsstatistik für alle Profile zurück. | `src/router/outcomes.rs` |
| `pub fn reset_calibration(&self, profile_name: &str)` | Setzt Kalibrierungsstatistik für ein bestimmtes Profil zurück. | `src/router/outcomes.rs` |
| `pub fn set_lyapunov_baseline(&self, profile_name: &str, baseline: &[f32]) -> bool` | Setzt die Baseline für den Lyapunov-Drift-Wächter eines bestimmten Profils. | `src/router/outcomes.rs` |
| `pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<HashSet<u64>, D::Error>` | — | `src/serde_helpers.rs` |
| `pub fn serialize<S: Serializer>(set: &HashSet<u64>, s: S) -> Result<S::Ok, S::Error>` | — | `src/serde_helpers.rs` |
| `pub fn is_cloud(&self) -> bool` | Returns whether this transport represents a cloud endpoint. | `src/transport.rs` |


---

## `contextra-agent`

| Attribut | Wert |
|---|---|
| **Ring** | 3 — Ring 3 — Orchestration & Cognition |
| **Status** | Fertig |
| **Zweck** | Agenten-Workflow-Engine mit auditierbarer State-Machine und DLQ. |
| **Unsafe-Code** | Nein (`#![forbid(unsafe_code)]`) |
| **Interne Abhängigkeiten** | `contextra-checkpoint`, `contextra-db`, `contextra-graph`, `contextra-ports`, `contextra-router`, `contextra-store`, `contextra-types` |
| **Externe Abhängigkeiten (Auswahl)** | `bytes`, `serde`, `serde_json`, `tokio`, `tokio-util`, `tracing` |

### Architekturbeschreibung

Contextra Agent — Persistent workflow engine for multi-step agent execution.  Implements the deterministic `checkpoint → execute → commit → audit` loop. Sovereign alternative to LangGraph/AutoGen: pure Rust, zero external dependencies.  # Architecture Role (Layer 3) Built upon `contextra-db` (Collections), `contextra-checkpoint` (Snapshots & RAII CheckpointGuard), `contextra-graph` (Declarative StateGraph), and `contextra-store` (LSM Storage). Manages workflow state, token budget enforcement, and immutable audit logging over LSM-persisted keys.  # State Machine Diagram & Invariants ```text +--------+ |  Idle  | +---+----+ | run() v +-----+------+ |  Running   | <---+ (Loop: Checkpoint -> Execute -> Commit -> Audit) +--+------+--+     | |      |        | (NodeEnd)    (Error/  ---+ |     Panic) v      v +-----+--+ +-+------+ |Completed| | Failed | +--------+ +--------+ ```  ### Enforced State Transitions: - **Idle -> Running**: Initiated at entry of `OrchestratorEngine::run()`. - **Running -> Running**: Step execution loop enforced via `CheckpointGuard::for_agent_step` before node handler execution. Execution cannot proceed without completing the preceding checkpoint. - **Running -> Completed**: Triggered on `NodeType::End` node arrival after flushing LSM storage. - **Running -> Failed**: Triggered on any step error (tool execution error, budget exhaustion, unresolved edge, missing handler).  ### Crash Recovery Behavior during `execute()`: If a crash or panic occurs after a checkpoint is written but before step execution finishes: 1. The active `CheckpointGuard` is dropped, triggering automatic transaction rollback via `rollback_to_tx`. 2. On application restart, `OrchestratorEngine::replay_from()` restores `AgentContext` state to the last valid checkpoint step.  ### Cross-Layer Coupling Analysis: - `contextra-agent` depends on `contextra-db` for `Collection` and high-level operations. - Direct calls to `contextra-store` (`inner_storage()`) are strictly isolated to `PersistentCheckpointStore` and `CheckpointGuard` for transaction ID allocation and checkpoint snapshot manipulation, which `contextra-db` intentionally does not expose. - Production code (`src/`) contains **zero** direct calls to `contextra-graph`; graph state operations pass exclusively through `StateGraph` struct and `contextra-db`.  ### Audit Trail Integrity: - Immutable append-only audit entries are stored under `audit:{task_id}:step:{n}` via `Collection::insert`. - Entries pass directly through LSM storage and are protected by the same WAL-HMAC integrity chain (`contextra-crypto`) as all standard storage operations. - Failed executions log an `AuditEntry` with `error` details before transitioning context to `AgentStatus::Failed`.  # Invariants 1. **Auto-checkpoint before step**: Creates a checkpoint before executing each node handler, backed by RAII `CheckpointGuard` for automatic transaction rollback upon failure. 2. **Deterministic replay & rollback**: Restores `AgentContext` to any prior checkpoint step. 3. **Immutable audit trail**: Appends step records to state collection under `audit:{task_id}:step:{n}`. 4. **Token budget limit**: Enforces token budget consumption on each step.  ADR-042: Re-integration from archived `contextra-saos-agent` (Commit ddc4c77).

### Modul- und Re-Export-Oberfläche (lib.rs)

```rust
pub mod audit;
pub mod budget;
pub mod clm_scratchpad;
pub mod context;
pub mod dlq;
pub mod engine;
pub mod event_source;
pub mod goal_condition;
pub mod graph;
pub mod step;
pub use budget::{BudgetStrategy, Reservation, TokenBudget};
pub use clm_scratchpad::{ ClmScratchpad, ContextEditAuditSink, CountingContextEditAuditSink, CountingScratchpadCacheInvalidator, NoopContextEditAuditSink, NoopScratchpadCacheInvalidator, PinnedRegionId, ScratchpadCacheInvalidator, ScratchpadCheckpoint, ScratchpadEditOp, };
pub use context::{AgentContext, AgentStatus};
pub use dlq::DeadLetterQueue;
pub use engine::{EventLoopExitReason, OrchestratorEngine, MAX_WORKFLOW_STEPS};
pub use event_source::{BackgroundEvent, EventSource, PollingDocumentEventSource, VecEventSource};
pub use goal_condition::{parse_legacy_condition_string, GoalCondition};
pub use graph::{AgentNode, NodeType, StateGraph, WorkflowEdge};
pub use step::{AgentTool, DeadLetterReason, StepDeadLetter, StepResult};
```

### Feature-Flags

| Flag | Aktiviert |
|---|---|
| `default` | `[]` |
| `test-utils` | `[]` |

### Schnittstellen — Public Traits (4)

**`AgentTool`** — ** (`src/step.rs`)

```rust
pub trait AgentTool : Send + Sync {
fn name(&self) -> &str;

    /// Returns the estimated token cost of executing this tool with the given input.
    ///
    /// Used for strict pre-execution budget validation to prevent side-effects when budget is exhausted.
    fn estimated_cost(&self, _input: &serde_json::Value) -> usize {
        0
    }

    fn execute<'a>(
        &'a self,
        ctx: &'a AgentContext,
        input: serde_json::Value,
    ) -> BoxFuture<'a, Result<StepResult>>;

    /// Maximale Ausführungszeit dieses Tools in Millisekunden.
    /// Standard: 30 Sekunden. Überschreibe für langläufige Tools.
    fn timeout_ms(&self) -> u64 {
        30_000
    }

    /// Ob dieser Schritt bei Timeout/transientem Fehler wiederholbar ist.
    fn is_retriable(&self) -> bool {
        true
    }

    /// Maximale Anzahl Retries. Standard: 2 (d.h. insgesamt 3 Versuche).
    fn max_retries(&self) -> u32 {
        2
    }
}
```

**`ContextEditAuditSink`** — *Lokaler Trait für Audit-Benachrichtigungen bei Scratchpad-Änderungen.* (`src/clm_scratchpad.rs`)

```rust
pub trait ContextEditAuditSink : Send + Sync {
fn record_edit(&self, record: ContextEditAuditRecord);
}
```

**`EventSource`** — *Dynamic event source delivering continuous background events.* (`src/event_source.rs`)

```rust
pub trait EventSource : Send + Sync {
/// Fetches the next available background event.
    /// Returns `Ok(Some(event))` when an event is ready, `Ok(None)` if no event is currently pending.
    fn next_event<'a>(&'a mut self) -> BoxFuture<'a, Result<Option<BackgroundEvent>>>;

    /// Indicates whether the event source is permanently exhausted.
    /// Always returns `false` for continuous/always-on producers.
    fn is_exhausted(&self) -> bool {
        false
    }

    /// Waits until a new event is likely available or the timeout expires.
    /// Default implementation: short sleep fallback for polling-based sources.
    /// Implementations with push notifications (e.g. `tokio::sync::Notify`) should override
    /// this method to wait on signals instead of polling.
    fn wait_for_event<'a>(&'a self) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
        })
    }
}
```

**`ScratchpadCacheInvalidator`** — *Lokaler Trait zur Cache-Invalidierung des Scratchpad-Scopes bei Resets.* (`src/clm_scratchpad.rs`)

```rust
pub trait ScratchpadCacheInvalidator : Send + Sync {
fn invalidate_scratchpad_scope(&self, tenant_id: TenantId, task_id: &str, subgoal_index: u64);
}
```

### Öffentliche Datentypen — Structs (23)

| Struct | Felder / Form | Beschreibung | Datei |
|---|---|---|---|
| `AgentContext` | `{ pub task_id: String, pub current_node: String, pub step_count: u64, pub db: Arc<Contextra>, pub state_collection: Arc<Collection<LsmStorage>>, pub budget: Toke… }` | Operational context spanning the entire workflow execution. | `src/context.rs` |
| `AgentEngine` | `{ /// Active cache directive set during step execution. pub current_directive: CacheDirective, }` | Agent execution engine helper managing per-step cache directives during workflow execution. | `src/context.rs` |
| `AgentNode` | `{ pub id: NodeId, pub description: String, pub node_type: NodeType, /// Registered name of the tool/function to execute, if applicable. pub handler: Option<Strin… }` | Represents a single specialized node in the workflow. | `src/graph.rs` |
| `AuditEntry` | `{ pub task_id: String, pub step_count: u64, pub node_id: String, pub tokens_consumed: usize, pub payload: serde_json::Value, #[serde(default, skip_serializing_if… }` | Single immutable record of an agent step execution. | `src/audit.rs` |
| `AuditLog` | `{ collection: Arc<Collection<S>>, }` | Append-only audit log backed by a Contextra collection. | `src/audit.rs` |
| `BackgroundEvent` | `{ pub payload: serde_json::Value, pub source: String, pub observed_at_seq: u64, }` | Background telemetry/trigger event delivered to the agent context. | `src/event_source.rs` |
| `ClmScratchpad` | `{ task_id: String, tenant_id: TenantId, current_subgoal_index: u64, created_at_tx: u64, config: VaultConfig, vault: VolatileContextVault, chunks: Vec<VaultChunk>… }` | Temporärer, aufgabengebundener Arbeitskontext-Scratchpad für Agentenschleifen. Kapselt eine [`VolatileContextVault`]-Instanz für die sichere RAM-nati… | `src/clm_scratchpad.rs` |
| `ContextEditAuditRecord` | `{ pub tenant_id: TenantId, pub task_id: String, pub operation: String, pub subgoal_index: u64, pub timestamp_nanos: u64, }` | Audit-Datensatz für Scratchpad-Kontextänderungen und Resets. | `src/clm_scratchpad.rs` |
| `CountingContextEditAuditSink` | `{ count: AtomicU64, }` | Test-Implementierung von [`ContextEditAuditSink`], die Aufrufe zählt. | `src/clm_scratchpad.rs` |
| `CountingScratchpadCacheInvalidator` | `{ count: AtomicU64, }` | Test-Implementierung von [`ScratchpadCacheInvalidator`], die Aufrufe zählt. | `src/clm_scratchpad.rs` |
| `DeadLetterQueue` | `{ storage: Arc<dyn StorageEngine>, next_tx: OnceCell<AtomicU64>, metrics_sink: Option<Arc<dyn contextra_ports::MetricsSink>>, }` | Persistente Dead-Letter-Queue für fehlgeschlagene Agent-Schritte. Verwendet denselben Storage wie der Agent (LSM) mit einem fixen Key-Prefix. | `src/dlq.rs` |
| `MigrationStats` | `{ pub migrated: usize, pub failed: usize, }` | Summary statistics for legacy audit entry migration. | `src/audit.rs` |
| `NoopContextEditAuditSink` | `(tuple/unit)` | No-op-Implementierung von [`ContextEditAuditSink`]. | `src/clm_scratchpad.rs` |
| `NoopScratchpadCacheInvalidator` | `(tuple/unit)` | No-op-Implementierung von [`ScratchpadCacheInvalidator`]. | `src/clm_scratchpad.rs` |
| `OrchestratorEngine` | `{ pub tools: HashMap<String, Box<dyn AgentTool>>, pub checkpoint_store: Arc<dyn CheckpointRegistry>, pub dead_letter_queue: Option<DeadLetterQueue>, pub clock: A… }` | Async executor engine applying nodes in Sequence. | `src/engine.rs` |
| `PinnedRegionId` | `(tuple/unit)` | Marker für unveränderliche Kontextteile (Systemprompt, Gesamtziel, Safety-Regeln). Wird ausschließlich gelesen und NIEMALS durch diesen Typ selbst ve… | `src/clm_scratchpad.rs` |
| `PollingDocumentEventSource` | `{ collection: Arc<Collection<S>>, last_seen_seq: u64, poll_interval: Duration, pending_events: VecDeque<BackgroundEvent>, }` | Concrete `EventSource` that periodically polls `Collection` storage sequence numbers, using `scan_prefix_at` for snapshot delta calculation to emit d… | `src/event_source.rs` |
| `ScratchpadCheckpoint` | `{ /// Eindeutige Aufgaben-ID. pub task_id: String, /// Index des erfolgreich abgeschlossenen Teilziels. pub completed_subgoal_index: u64, /// Löschquittung nach… }` | Zusammenfassung eines Subgoal-Checkpoints vor dem Zurücksetzen des Scratchpads. Dieser Typ dient als Übergabeobjekt für nachfolgende Stufen (z. B. Sp… | `src/clm_scratchpad.rs` |
| `StateGraph` | `{ pub nodes: HashMap<NodeId, AgentNode>, pub edges: Vec<WorkflowEdge>, }` | Core declarative structure routing autonomous agent steps. | `src/graph.rs` |
| `StepDeadLetter` | `{ /// Session-ID des Agenten pub session_id: String, /// Node-ID des fehlgeschlagenen Schritts pub node_id: String, /// Schrittnummer im Workflow (0-indexed) #[s… }` | Ein fehlgeschlagener Agent-Schritt der für spätere Analyse und Idempotenz-Prüfung persistiert wird. | `src/step.rs` |
| `StepResult` | `{ pub node_id: String, pub output: serde_json::Value, pub tokens_consumed: usize, /// Identifier condition of the next edge transition if dictated dynamically. p… }` | The explicit result of an agent step execution. | `src/step.rs` |
| `VecEventSource` | `{ events: VecDeque<BackgroundEvent>, }` | Trivial mock/static `EventSource` that reads from a fixed list of `BackgroundEvent`s. | `src/event_source.rs` |
| `WorkflowEdge` | `{ pub from: NodeId, pub to: NodeId, /// Legacy text-based condition string (maintained for 100% backward compatibility). pub condition: Option<String>, pub prior… }` | Represents a conditional transition between two graph nodes. | `src/graph.rs` |

### Öffentliche Datentypen — Enums (6)

| Enum | Varianten | Beschreibung | Datei |
|---|---|---|---|
| `AgentStatus` | `Idle, Running, Completed, Failed,` | Operational status of an agent workflow. | `src/context.rs` |
| `DeadLetterReason` | `/// Tool hat nicht innerhalb des Timeouts geantwortet Timeout { timeout_ms: u64 }, /// Budget erschöpft bevor der Schritt starten konnte BudgetExhausted { available: usize, required: usize }, /// Too…` | — | `src/step.rs` |
| `EventLoopExitReason` | `Shutdown, SourceExhausted,` | Reason for exiting `OrchestratorEngine::run_event_loop`. | `src/engine.rs` |
| `GoalCondition` | `/// Prüft, ob ein JSON-Feld unter `field_path` im `StepResult::output` mit `expected` übereinstimmt. OutputFieldEquals { /// Punkt-getrennter JSON-Pfad (z. B. "status.done"). field_path: String, ///…` | Repräsentiert eine strukturierte Bedingung zur Auswertung, ob ein Teilziel eines Agentenschritts erfüllt ist. | `src/goal_condition.rs` |
| `NodeType` | `Start, Task, Decision, End,` | Type of node within the declarative agent state graph. | `src/graph.rs` |
| `ScratchpadEditOp` | `/// Hängt einen neuen Kontext-Chunk an das Scratchpad an. Append { /// Byte-Inhalt des neuen Chunks. content: Vec<u8>, /// Optionaler Label-Bezeichner für spätere Referenzierung (Replace/Remove). lab…` | Operationen zur Bearbeitung des Scratchpads. | `src/clm_scratchpad.rs` |

### Typ-Aliase (1)

| Alias | Definition | Beschreibung |
|---|---|---|
| `NodeId` | `type NodeId= String` | — |

### Konstanten (7)

| Konstante | Typ | Beschreibung |
|---|---|---|
| `MAX_EVENT_SOURCE_CAPACITY` | `usize` | Maximum allowed events in pending event buffers to prevent memory exhaustion. |
| `MAX_ID_LEN` | `usize` | Maximum allowed length in bytes for task IDs and node IDs. |
| `MAX_PENDING_EVENTS_CAPACITY` | `usize` | Maximum capacity for pending background telemetry events queue before dropping or rejecting. |
| `MAX_TELEMETRY_EVENTS` | `usize` | Maximum allowed telemetry events stored in memory history. |
| `MAX_TEXT_LEN` | `usize` | Maximum allowed length in bytes for descriptions and conditions. |
| `MAX_WORKFLOW_STEPS` | `u64` | Maximum allowed steps in a single workflow execution to prevent unbounded loops. |
| `PREFIX` | `&'static [u8]` | — |

### Öffentliche Funktionen (87)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub async fn append(&self, entry: &AuditEntry) -> Result<()>` | Appends an immutable audit entry directly via LSM storage without HNSW vector index participation (AC-3). Enforces append-only immutability… | `src/audit.rs` |
| `pub async fn append_to(collection: &Collection<S>, entry: &AuditEntry) -> Result<()>` | Appends an immutable audit entry directly to a collection without needing an [`AuditLog`] wrapper. | `src/audit.rs` |
| `pub async fn migrate_legacy_audit_entries<S: StorageEngine, V: contextra_ports::VectorIndex>( collection: &Collection<S, V>, ) -> Result<MigrationStats>` | Migrates legacy zero-vector audit entries from the HNSW vector index and doc_key mappings into the direct LSM KV store format (`put_kv`). R… | `src/audit.rs` |
| `pub async fn migrate_legacy_entries(&self) -> Result<MigrationStats>` | Migrates legacy zero-vector audit entries from the HNSW vector index into direct LSM KV store format. | `src/audit.rs` |
| `pub fn new(collection: Arc<Collection<S>>) -> Self` | — | `src/audit.rs` |
| `pub async fn replay_task(&self, task_id: &str) -> Result<Vec<AuditEntry>>` | Replays all audit entries for a given task via scan_prefix. | `src/audit.rs` |
| `pub fn validate_audit_payload_and_error( payload: &serde_json::Value, error: Option<&str>, ) -> Result<()>` | Validates payload and optional error string of an audit entry for non-emptiness and absence of null bytes. | `src/audit.rs` |
| `pub fn reserve_tokens<'a>( budget: &'a TokenBudget, amount: usize, ) -> contextra_types::Result<Reservation<'a>>` | Helper to reserve `amount` tokens from `budget`. Returns an RAII [`Reservation`] guard on success. If execution drops the guard without cal… | `src/budget.rs` |
| `pub fn apply_edit(&mut self, op: ScratchpadEditOp, captured_tx: TxId) -> Result<()>` | Wendet eine Bearbeitungs-Operation ([`ScratchpadEditOp`]) auf das Scratchpad an. # Fehler Liefert `ContextraError::InvalidInput` falls: - V… | `src/clm_scratchpad.rs` |
| `pub fn as_str(&self) -> &str` | Gibt den inneren String-Bezeichner zurück. | `src/clm_scratchpad.rs` |
| `pub fn checkpoint_and_reset(&mut self) -> Result<ScratchpadCheckpoint>` | Erzeugt eine [`ScratchpadCheckpoint`]-Zusammenfassung, leert den Vault intern via `purge()` (inkl. Zeroize aller RAM-Inhalte), meldet Audit… | `src/clm_scratchpad.rs` |
| `pub fn count(&self) -> u64` | — | `src/clm_scratchpad.rs` |
| `pub fn count(&self) -> u64` | — | `src/clm_scratchpad.rs` |
| `pub fn created_at_tx(&self) -> u64` | Gibt die Transaktions-ID der Erzeugung zurück. | `src/clm_scratchpad.rs` |
| `pub fn current_subgoal_index(&self) -> u64` | Gibt den aktuellen Subgoal-Index zurück. | `src/clm_scratchpad.rs` |
| `pub fn new() -> Self` | — | `src/clm_scratchpad.rs` |
| `pub fn new() -> Self` | — | `src/clm_scratchpad.rs` |
| `pub fn new(id: impl Into<String>) -> Self` | Erzeugt eine neue `PinnedRegionId`. | `src/clm_scratchpad.rs` |
| `pub fn new(task_id: String, config: VaultConfig) -> Self` | Erzeugt einen neuen `ClmScratchpad` für eine Task mit gegebener Vault-Konfiguration. | `src/clm_scratchpad.rs` |
| `pub fn new_scoped( task_id: String, config: VaultConfig, tenant_id: TenantId, created_at_tx: TxId, audit_sink: Arc<dyn ContextEditAuditSink>, cache_invalidator: Arc<dyn ScratchpadCacheInvalidator>, pinned_regions: Vec<PinnedRegionId>, ) -> Self` | Erzeugt einen neuen `ClmScratchpad` für eine Task mit voll spezifizierten Mandanten- und Auditeigenschaften. | `src/clm_scratchpad.rs` |
| `pub fn pinned_regions(&self) -> &[PinnedRegionId]` | Gibt eine immutable Referenz auf die registrierten gepinnten Regionen zurück. | `src/clm_scratchpad.rs` |
| `pub fn task_id(&self) -> &str` | Gibt die Task-ID zurück. | `src/clm_scratchpad.rs` |
| `pub fn tenant_id(&self) -> TenantId` | Gibt die Mandanten-ID zurück. | `src/clm_scratchpad.rs` |
| `pub fn total_bytes(&self) -> usize` | Gibt die Gesamtzahl der belegten Bytes im aktuellen Scratchpad-Vault zurück. | `src/clm_scratchpad.rs` |
| `pub fn with_audit_sink(mut self, audit_sink: Arc<dyn ContextEditAuditSink>) -> Self` | Setzt die Audit-Sink (Builder-Muster). | `src/clm_scratchpad.rs` |
| `pub fn with_cache_invalidator( mut self, cache_invalidator: Arc<dyn ScratchpadCacheInvalidator>, ) -> Self` | Setzt den Cache-Invalidator (Builder-Muster). | `src/clm_scratchpad.rs` |
| `pub fn with_clock(mut self, clock: Arc<dyn Clock>) -> Self` | Setzt die Clock für deterministisches Zeitmanagement (Builder-Muster). | `src/clm_scratchpad.rs` |
| `pub fn with_created_at_tx(mut self, created_at_tx: TxId) -> Self` | Setzt die Transaktions-ID der Erstellung (Builder-Muster). | `src/clm_scratchpad.rs` |
| `pub fn with_pinned_region(mut self, region_id: PinnedRegionId) -> Self` | Registriert eine gepinnte Region (unveränderlicher Kontextteil). | `src/clm_scratchpad.rs` |
| `pub fn with_tenant_id(mut self, tenant_id: TenantId) -> Self` | Setzt die Mandanten-ID (Builder-Muster). | `src/clm_scratchpad.rs` |
| `pub fn attach_event(&mut self, event: crate::event_source::BackgroundEvent)` | Integrates a background telemetry event into the agent context memory and history. Maintains event history capacity bounded at `MAX_TELEMET… | `src/context.rs` |
| `pub fn directive_for_node(&self, node_id: &str, step_id: StepId) -> CacheDirective` | Returns the appropriate `CacheDirective` for a workflow node / prompt role. | `src/context.rs` |
| `pub fn directive_for_step(&self, node_id: &str, step_id: StepId) -> CacheDirective` | Determines the appropriate `CacheDirective` for a node ID / prompt role. - System prompt nodes -> `CacheDirective::Pin { ttl: None }` - Tra… | `src/context.rs` |
| `pub fn get_cache_directive(&self) -> &CacheDirective` | Returns the active cache directive for the context. | `src/context.rs` |
| `pub fn new() -> Self` | Constructs a new `AgentEngine`. | `src/context.rs` |
| `pub fn next_retry_step_count(&self) -> u64` | Returns the step count to be used for a recovery retry after a partial step failure. | `src/context.rs` |
| `pub fn run( &mut self, ctx: &mut AgentContext, step_id: StepId, node_id: &str, ) -> CacheDirective` | Executes step configuration, updating context with per-step cache directives. | `src/context.rs` |
| `pub fn set_cache_directive(&mut self, directive: CacheDirective)` | Sets the active cache directive for the context. | `src/context.rs` |
| `pub fn set_step_cache_directive(&mut self, step_id: StepId, directive: CacheDirective)` | Records a cache directive for a specific step ID. | `src/context.rs` |
| `pub fn try_attach_event(&mut self, event: crate::event_source::BackgroundEvent) -> Result<()>` | Integrates a background telemetry event with an explicit capacity check. | `src/context.rs` |
| `pub fn try_new( task_id: impl Into<String>, start_node: impl Into<String>, db: Arc<Contextra>, state_collection: Arc<Collection<LsmStorage>>, budget: TokenBudget, ) -> contextra_types::Result<Self>` | — | `src/context.rs` |
| `pub fn validate_node_id(node_id: &str) -> Result<()>` | Validates a node identifier to ensure it is non-empty, <= 256 bytes, and contains no null bytes. | `src/context.rs` |
| `pub fn validate_task_id(task_id: &str) -> Result<()>` | Validates a task identifier to ensure it is non-empty, <= 256 bytes, and contains no null bytes. | `src/context.rs` |
| `pub fn with_clm_scratchpad(mut self, scratchpad: crate::clm_scratchpad::ClmScratchpad) -> Self` | Attaches an optional [`ClmScratchpad`](crate::clm_scratchpad::ClmScratchpad) to the agent context (Builder pattern). | `src/context.rs` |
| `pub async fn allocate_tx(&self) -> Result<TxId>` | — | `src/dlq.rs` |
| `pub async fn drain(&self) -> Result<Vec<StepDeadLetter>>` | — | `src/dlq.rs` |
| `pub async fn drain_uncommitted(&self) -> Result<Vec<StepDeadLetter>>` | Drains uncommitted dead letter entries from the queue, discarding entries whose transaction was already committed in WAL/storage according… | `src/dlq.rs` |
| `pub async fn is_already_committed(&self, letter: &StepDeadLetter) -> Result<bool>` | Prüft vor dem Replay eines DLQ-Eintrags, ob die im Eintrag assoziierte `TxId` bereits im WAL/Storage für denselben `(Session, Node, Step)`-… | `src/dlq.rs` |
| `pub async fn list(&self) -> Result<Vec<StepDeadLetter>>` | — | `src/dlq.rs` |
| `pub fn new(storage: Arc<dyn StorageEngine>) -> Self` | — | `src/dlq.rs` |
| `pub fn new_with_metrics( storage: Arc<dyn StorageEngine>, sink: Arc<dyn contextra_ports::MetricsSink>, ) -> Self` | — | `src/dlq.rs` |
| `pub async fn push(&self, letter: &StepDeadLetter) -> Result<()>` | — | `src/dlq.rs` |
| `pub async fn remove(&self, letter: &StepDeadLetter) -> Result<bool>` | Entfernt einen spezifischen DLQ-Eintrag anhand seines `(session_id, node_id, step_index)` Schlüssels. | `src/dlq.rs` |
| `pub fn with_metrics_sink(mut self, sink: Arc<dyn contextra_ports::MetricsSink>) -> Self` | — | `src/dlq.rs` |
| `pub async fn checkpoint(&self, ctx: &AgentContext) -> Result<()>` | — | `src/engine.rs` |
| `pub fn from_db(db: &contextra_db::Contextra) -> Result<Self>` | Constructs an [`OrchestratorEngine`] from a Contextra DB handle. Alias for [`try_from_db`]. | `src/engine.rs` |
| `pub fn new(storage: Arc<LsmStorage>) -> Self` | — | `src/engine.rs` |
| `pub async fn recover_orphans(&self) -> Result<()>` | Recovers all registered/persisted orphaned sequence pins and checkpoints. | `src/engine.rs` |
| `pub async fn replay_from(&self, ctx: &mut AgentContext, identifier: &str) -> Result<()>` | Setzt den AgentContext auf einen früheren Checkpoint zurück (AC-2). # Adressierung von Checkpoints Das `identifier`-Argument unterstützt fo… | `src/engine.rs` |
| `pub async fn run(&self, ctx: &mut AgentContext, graph: &StateGraph) -> Result<()>` | — | `src/engine.rs` |
| `pub async fn run_event_loop( &self, ctx: &mut AgentContext, graph: &StateGraph, source: &mut dyn crate::event_source::EventSource, shutdown: tokio_util::sync::CancellationToken, ) -> Result<EventLoopExitReason>` | Continuous event loop reading telemetry events from an `EventSource`, attaching each event to `AgentContext`, executing `run()`, and checkp… | `src/engine.rs` |
| `pub fn try_from_db(db: &contextra_db::Contextra) -> Result<Self>` | Attempts to construct an [`OrchestratorEngine`] directly from a Contextra DB handle. | `src/engine.rs` |
| `pub fn try_new(storage: Arc<LsmStorage>) -> Result<Self>` | Attempts to construct a new [`OrchestratorEngine`], returning an error if checkpoint store initialization fails. | `src/engine.rs` |
| `pub fn try_register_tool(&mut self, tool: Box<dyn AgentTool>) -> Result<()>` | Attempts to register an agent tool with boundary validation on the tool name. | `src/engine.rs` |
| `pub fn with_clock(mut self, clock: Arc<dyn Clock>) -> Self` | Sets a custom [`Clock`] implementation for deterministic time management. | `src/engine.rs` |
| `pub fn with_metrics_sink(mut self, sink: Arc<dyn contextra_ports::MetricsSink>) -> Self` | Attaches a [`MetricsSink`] to the internal dead-letter queue, if configured. | `src/engine.rs` |
| `pub fn evaluate_condition_expr(expr: &str, ctx: &AgentContext) -> bool` | Evaluates a declarative condition expression against the provided [`AgentContext`]. # Supported Grammar: - `<key> exists`: Returns `true` i… | `src/engine/condition.rs` |
| `pub fn new( payload: serde_json::Value, source: impl Into<String>, observed_at_seq: u64, ) -> Self` | — | `src/event_source.rs` |
| `pub fn new(collection: Arc<Collection<S>>, poll_interval: Duration) -> Self` | — | `src/event_source.rs` |
| `pub fn new_from_seq( collection: Arc<Collection<S>>, poll_interval: Duration, last_seen_seq: u64, ) -> Self` | Creates a new `PollingDocumentEventSource` initialized from a specific sequence number checkpoint. | `src/event_source.rs` |
| `pub fn poll_interval(&self) -> Duration` | — | `src/event_source.rs` |
| `pub fn try_new( payload: serde_json::Value, source: impl Into<String>, observed_at_seq: u64, ) -> Result<Self>` | — | `src/event_source.rs` |
| `pub fn try_new(events: Vec<BackgroundEvent>) -> Result<Self>` | Attempts to construct a `VecEventSource` with a capacity check. | `src/event_source.rs` |
| `pub fn with_capacity( collection: Arc<Collection<S>>, poll_interval: Duration, _max_pending_capacity: usize, ) -> Self` | — | `src/event_source.rs` |
| `pub fn with_last_seen_seq(mut self, last_seen_seq: u64) -> Self` | — | `src/event_source.rs` |
| `pub fn describe(&self) -> String` | Liefert eine menschenlesbare Beschreibung der Bedingung (z. B. für Audit-Logging). | `src/goal_condition.rs` |
| `pub fn evaluate(&self, result: &StepResult) -> bool` | Wertet die Bedingung deterministisch gegen ein gegebenes [`StepResult`] aus. Liefert bei fehlendem oder ungültigem JSON-Pfad stets `false`.… | `src/goal_condition.rs` |
| `pub fn parse_legacy_condition_string(raw: &str) -> Option<GoalCondition>` | Bildet einen freien Bedingungs-String aus Altbestand auf eine strukturierte [`GoalCondition`] ab. Unterstützt Gleichheitsprüfungen ("field… | `src/goal_condition.rs` |
| `pub fn add_edge(&mut self, from: &str, to: &str, condition: Option<&str>, priority: u8)` | Adds an edge to the graph, panicking if validation fails. | `src/graph.rs` |
| `pub fn add_node( &mut self, id: &str, description: &str, node_type: NodeType, handler: Option<&str>, )` | Adds a node to the graph, panicking if validation fails. | `src/graph.rs` |
| `pub fn evaluate_condition(&self, result: &StepResult) -> bool` | Evaluates whether this edge's condition/goal is satisfied against a [`StepResult`]. # Evaluation Order & Fail-Closed Semantics: 1. If `goal… | `src/graph.rs` |
| `pub fn get_node(&self, id: &str) -> Option<&AgentNode>` | — | `src/graph.rs` |
| `pub fn has_condition(&self) -> bool` | Returns whether this edge has an explicit condition or goal attached. | `src/graph.rs` |
| `pub fn new() -> Self` | — | `src/graph.rs` |
| `pub fn try_add_edge( &mut self, from: &str, to: &str, condition: Option<&str>, priority: u8, ) -> Result<()>` | — | `src/graph.rs` |
| `pub fn try_add_edge_with_goal( &mut self, from: &str, to: &str, goal: GoalCondition, priority: u8, ) -> Result<()>` | Tries to insert a new edge with an explicit [`GoalCondition`]. | `src/graph.rs` |
| `pub fn try_add_node( &mut self, id: &str, description: &str, node_type: NodeType, handler: Option<&str>, ) -> Result<()>` | — | `src/graph.rs` |


---

## `contextra-db`

| Attribut | Wert |
|---|---|
| **Ring** | 3 — Ring 3 — Orchestration & Cognition |
| **Status** | In Migration (Legacy-Fassade) |
| **Zweck** | Monolithische Übergangsfassade für Speicher-, Such- und Indexzugriffe. |
| **Unsafe-Code** | Nein (`#![forbid(unsafe_code)]`) |
| **Interne Abhängigkeiten** | `contextra-adapt`, `contextra-checkpoint`, `contextra-cognition`, `contextra-crypto`, `contextra-engine`, `contextra-graph`, `contextra-ports`, `contextra-rank`, `contextra-store`, `contextra-sys`, `contextra-text`, `contextra-types`, `contextra-vector` |
| **Externe Abhängigkeiten (Auswahl)** | `ahash`, `blake3`, `bytes`, `parking_lot`, `serde`, `serde_json`, `thiserror`, `tokio`, `tokio-util`, `tracing`, `zeroize` |

### Modul- und Re-Export-Oberfläche (lib.rs)

```rust
pub mod homeostat;
pub mod multistep;
pub mod volatile_vault;
pub mod context { .. }
pub mod context_compaction { .. }
pub mod memory_consolidation { .. }
pub mod synthesis_phase { .. }
pub mod consolidation_executor { .. }
pub mod consolidation_locks { .. }
pub mod maintenance_scheduler { .. }
pub mod maintenance_config { .. }
pub mod reaper { .. }
pub mod decay_controller { .. }
pub mod pid_latency_controller { .. }
pub use contextra_cognition as cognition;
pub use contextra_engine as engine;
pub use contextra_engine::MetadataFilter;
pub use contextra_engine::{ background_workers, chunker, collection, export, filter, import, temporal_filter, transaction, };
pub use contextra_engine::{ CommunityDetectionConfig, Contextra, ContextraConfig, ContextraStats, DbStats, Document, EmbeddingBackend, ExportCollectionV1, ExportDocumentV1, ExportMemoryV1, ExportRelationV1, HybridQueryBuilder, ImportSummary, Language, ProvenanceRecord, SearchResult, SearchStrategy, SignalContribution, SignalWeights, MAX_SCAN_RESULTS, SCHEMA_VERSION_V1, };
pub use contextra_engine::collection::maintenance::PercolationResult;
pub use contextra_engine::SandboxBridge;
pub use contextra_cognition::execute_sleep_cycle;
pub use contextra_cognition::{ cleanup_orphaned_consolidation_intents, compact_segment_via_context_compactor, compute_community_hash, detect_near_duplicates, execute_background_consolidation, execute_consolidation_pass, group_turns_into_segments, run_consolidation_pass, run_synthesis_pass, CommunityStabilityTracker, CompactedContext, CompactionStrategy, ConsolidationConfig, ConsolidationEngine, ConsolidationNodesGuard, ConsolidationPhaseResult, ConsolidationSession, ContextCompactor, ContextManager, MaintenanceConfig, MaintenanceScheduler, MetaChunk, SpatialFence, StatusToken, SynthesisConfig, SynthesisPhaseResult, TurnSegment, };
pub use contextra_cognition::context::*;
pub use contextra_cognition::context_compaction::*;
pub use contextra_cognition::memory_consolidation::*;
pub use contextra_cognition::synthesis_phase::*;
pub use contextra_cognition::consolidation_executor::*;
pub use contextra_cognition::consolidation_locks::*;
pub use contextra_cognition::maintenance_scheduler::*;
pub use contextra_cognition::maintenance_config::*;
pub use contextra_engine::background_workers::*;
pub use contextra_cognition::start_consolidation_reaper;
pub use contextra_adapt::decay_controller::*;
pub use contextra_engine::fusion;
pub use contextra_adapt::pid_latency_controller::*;
pub use volatile_vault::{ CommitReceipt, PurgeReceipt, SignalModality, VaultChunk, VaultChunkMetadata, VaultConfig, VaultError, VolatileContextVault, };
pub use decay_controller::{AdaptiveDecayController, DecayControllerConfig, DecaySignalInputs};
pub use homeostat::RerankDeadline;
pub use homeostat::RerankPidController;
pub use multistep::{MultiStepConfig, MultiStepEngine, MultiStepResult, QueryRewriter};
pub use pid_latency_controller::{ LatencyBudgetGuard, PidLatencyController, DEFAULT_TARGET_LATENCY_MS, MAX_SCALING_FACTOR, MIN_SCALING_FACTOR, };
pub use collection::{Collection, CollectionConfig};
pub use contextra_checkpoint;
pub use contextra_graph::percolation::PercolationConfig;
pub use contextra_ports::{DriftStatusProvider, SegmentSynthesizer, TextEmbeddingEngine};
pub use contextra_types::DistanceMetric;
pub use serde_json::json;
```

### Feature-Flags

| Flag | Aktiviert |
|---|---|
| `default` | `[]` |
| `bench` | `[]` |
| `sandbox` | `["contextra-engine/sandbox"]` |
| `reranking` | `[]` |
| `onnx` | `[]` |
| `experimental-diskann` | `["contextra-vector/experimental-diskann"]` |
| `background-maintenance` | `["contextra-engine/background-maintenance"]` |
| `graph-connectivity-health` | `["contextra-graph/physio-percolation", "contextra-engine/graph-connectivity-health"]` |
| `coherence-bonus-fusion` | `[]` |
| `adaptive-candidate-pool-sizing` | `[]` |
| `volatile-vault` | `[]` |
| `edge-reinforcement-learning` | `["contextra-graph/edge-reinforcement-learning"]` |

### Öffentliche Datentypen — Structs (9)

| Struct | Felder / Form | Beschreibung | Datei |
|---|---|---|---|
| `CommitReceipt` | `{ pub chunks_committed: usize, pub bytes_committed: usize, }` | Quittung nach erfolgreichem commit_to_storage(). | `src/volatile_vault.rs` |
| `MultiStepConfig` | `{ /// Maximale Iterationsrunden (Standard: 3). pub max_rounds: usize, /// Mindest-Score-Schwellenwert: unter diesem Wert gilt Runde als unzureichend. pub quality… }` | Konfiguration für Multi-Step Retrieval. | `src/multistep.rs` |
| `MultiStepEngine` | `{ collection: Arc<Collection<S>>, config: MultiStepConfig, pid_controller: parking_lot::Mutex<PidController>, clock: Arc<dyn Clock>, }` | Multi-Step Retrieval Engine. Implementiert iteratives Query-Rewriting für komplexe Agenten-Abfragen. Erfordert ein `QueryRewriter`-Trait für LLM-basi… | `src/multistep.rs` |
| `MultiStepResult` | `{ pub results: Vec<SearchResult>, /// Anzahl der tatsächlich durchgeführten Runden. pub rounds_executed: usize, /// Queries die in den Folgerunden verwendet wurd… }` | Ergebnis einer Multi-Step-Suche mit Audit-Informationen. | `src/multistep.rs` |
| `PurgeReceipt` | `{ /// Anzahl gelöschter Chunks. pub chunks_purged: usize, /// Bytes, die gezeroized wurden. pub bytes_zeroed: usize, /// Zeitpunkt des Purge. pub purged_at: Inst… }` | Quittung nach erfolgreichem purge(). | `src/volatile_vault.rs` |
| `VaultChunk` | `{ #[zeroize(skip)] pub id: DocId, // DocId-Wert, ohne komplexe Drop-Interaktion /// Sensitiver Inhalt — wird bei Drop gezeroized. pub content: Vec<u8>, #[zeroize… }` | Ein einzelner Chunk im VolatileContextVault. Inhalte werden bei Drop via Zeroize überschrieben. | `src/volatile_vault.rs` |
| `VaultChunkMetadata` | `{ pub id: DocId, pub modality: SignalModality, pub size_bytes: usize, pub label: Option<String>, }` | Metadaten-Ansicht eines VaultChunks ohne sensitiven Inhalt. | `src/volatile_vault.rs` |
| `VaultConfig` | `{ /// Maximale Größe aller Chunks (Bytes). Überschreitung → Err statt Disk-Fallback. /// Default: 512 MB pub max_capacity_bytes: usize, /// Ob mlock() versucht w… }` | Konfiguration für den VolatileContextVault. | `src/volatile_vault.rs` |
| `VolatileContextVault` | `{ chunks: Vec<VaultChunk>, config: VaultConfig, current_bytes: usize, /// Gekapselte mlock'd Speicherbereiche. /// Wird in munlock() aufgelöst, wenn der Vault dr… }` | Ephemerer RAM-Puffer für sensitive Kontexte. # INVARIANTEN - INV-VAULT-1: `purge()` zeroized alle Chunk-Inhalte via `ZeroizeOnDrop` vor der Deallokat… | `src/volatile_vault.rs` |

### Öffentliche Datentypen — Enums (2)

| Enum | Varianten | Beschreibung | Datei |
|---|---|---|---|
| `SignalModality` | `AudioTranscript, VisualCapture, TextInput, StructuredData,` | Modalität eines Vault-Chunks — bestimmt Verarbeitungskontext beim Commit. | `src/volatile_vault.rs` |
| `VaultError` | `#[error("Vault-Kapazität überschritten: {current_bytes} + {new_bytes} > {max_bytes}")] CapacityExceeded { current_bytes: usize, new_bytes: usize, max_bytes: usize, }, #[error("Vault bereits konsumier…` | Fehler des VolatileContextVault. | `src/volatile_vault.rs` |

### Konstanten (2)

| Konstante | Typ | Beschreibung |
|---|---|---|
| `DEFAULT_MAX_SUB_QUERIES` | `usize` | Default maximum number of sub-queries generated per round (N=3). |
| `DEFAULT_RERANK_POOL_MULTIPLIER` | `usize` | Default candidate pool expansion multiplier for pre-reranking retrieval. |

### Öffentliche Funktionen (16)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn new(collection: Arc<Collection<S>>, config: MultiStepConfig) -> Self` | — | `src/multistep.rs` |
| `pub async fn search( &self, original_query: &str, vector: &[f32], k: usize, rewriter: Option<&dyn QueryRewriter>, ) -> Result<MultiStepResult>` | Führt iterative Hybrid-Suche durch. Runde 1: Standard-Hybrid-Suche mit `original_query`. Runde 2–N: Falls Qualität unzureichend, QueryRewri… | `src/multistep.rs` |
| `pub async fn search_with_embedder( &self, original_query: &str, vector: &[f32], k: usize, rewriter: Option<&dyn QueryRewriter>, embedder: Option<&dyn TextEmbeddingEngine>, ) -> Result<MultiStepResult>` | Führt iterative multi-signale Hybrid-Suche mit Embedder für semantische Sub-Queries durch. | `src/multistep.rs` |
| `pub fn with_clock(mut self, clock: Arc<dyn Clock>) -> Self` | Setzt einen benutzerdefinierten `Clock`-Port für deterministische Tests. | `src/multistep.rs` |
| `pub fn current_size_bytes(&self) -> usize` | Wie viele Bytes liegen aktuell im Vault (RAM)? | `src/volatile_vault.rs` |
| `pub fn drain_for_commit(&mut self) -> (Vec<VaultChunkMetadata>, Vec<VaultChunk>)` | Entnimmt alle Chunks aus dem Vault für die Speicherung im Storage Engine. # INVARIANTE INV-VAULT-2 Dies ist der einzige Schreibpfad auf dau… | `src/volatile_vault.rs` |
| `pub fn ingest(&mut self, chunk: VaultChunk) -> Result<(), VaultError>` | Chunk einpflegen. # INVARIANTE INV-VAULT-2 Diese Funktion schreibt NIE auf dauerhaften Speicher. Jeder Codepfad, der Storage-Puts oder Tran… | `src/volatile_vault.rs` |
| `pub fn is_empty(&self) -> bool` | — | `src/volatile_vault.rs` |
| `pub fn is_memory_locked(&self) -> bool` | Gibt zurück, ob alle sensitiven Chunk-Inhalte im RAM fixiert (memory-locked) sind. Liefert `false`, wenn `attempt_mlock` in der [`VaultConf… | `src/volatile_vault.rs` |
| `pub fn len(&self) -> usize` | Anzahl der Chunks im Vault. | `src/volatile_vault.rs` |
| `pub fn new(id: DocId, content: Vec<u8>, modality: SignalModality, captured_tx: TxId) -> Self` | — | `src/volatile_vault.rs` |
| `pub fn open(config: VaultConfig) -> Self` | Neuen leeren Vault öffnen. Kein I/O. | `src/volatile_vault.rs` |
| `pub fn preview_metadata(&self) -> Vec<VaultChunkMetadata>` | Lesezugriff auf Chunks (für User-Preview vor Commit/Purge-Entscheidung). Gibt nur Metadaten zurück, keine Inhalte — verhindert versehentlic… | `src/volatile_vault.rs` |
| `pub fn purge(mut self) -> PurgeReceipt` | Alle Chunks verwerfen und Heap-Inhalt nullsetzen. Nach diesem Aufruf enthält der Vault keine Daten mehr. Der Vault selbst wird konsumiert (… | `src/volatile_vault.rs` |
| `pub fn size_bytes(&self) -> usize` | Größe des Chunk-Inhalts in Bytes. | `src/volatile_vault.rs` |
| `pub fn with_label(mut self, label: impl Into<String>) -> Self` | — | `src/volatile_vault.rs` |


---

## `contextra`

| Attribut | Wert |
|---|---|
| **Ring** | 4 — Ring 4 — Boundary & Composition Roots |
| **Status** | Fertig |
| **Zweck** | Öffentliche Haupt-Fassade und Composition Root für Rust-Anwendungen. |
| **Unsafe-Code** | Nein (`#![forbid(unsafe_code)]`) |
| **Interne Abhängigkeiten** | `contextra-audit-export`, `contextra-avv-generator`, `contextra-core`, `contextra-crypto`, `contextra-db`, `contextra-infer-candle`, `contextra-infer-ollama`, `contextra-infer-onnx`, `contextra-license`, `contextra-ports`, `contextra-privacy`, `contextra-rank`, `contextra-router`, `contextra-store`, `contextra-types` |
| **Externe Abhängigkeiten (Auswahl)** | `serde`, `serde_json`, `thiserror`, `tokio`, `uuid` |

### Modul- und Re-Export-Oberfläche (lib.rs)

```rust
pub mod agent_memory;
pub mod builder;
pub mod collection_profile;
pub mod performance_profile;
pub mod query_rewriter;
pub use agent_memory::{AgentMemory, Memory, MemoryId};
pub use builder::ContextraBuilder;
pub use contextra_core::error::ContextraError;
pub use contextra_core::types::domain::{DocId, ScoredDocument};
pub use contextra_core::DistanceMetric;
pub use contextra_db::{ chunker, execute_background_consolidation, memory_consolidation, Collection, CollectionConfig, Contextra, ContextraConfig, ContextraStats, DriftStatusProvider, EmbeddingBackend, SearchResult, TextEmbeddingEngine, };
pub use query_rewriter::LlmQueryRewriter;
pub use contextra_router as router;
pub use contextra_rank as rank;
pub use contextra_infer_ollama as ollama;
pub use contextra_infer_candle as candle;
pub use contextra_infer_onnx as onnx;
```

### Feature-Flags

| Flag | Aktiviert |
|---|---|
| `default` | `["fast", "candle"]` |
| `fast` | `[]` |
| `docid-128` | `["contextra-types/docid-128"]` |
| `sovereign` | `["contextra-crypto", "contextra-privacy"]` |
| `compliance` | `["sovereign", "audit-export", "avv-generator"]` |
| `audit-export` | `["dep:contextra-audit-export"]` |
| `avv-generator` | `["dep:contextra-avv-generator"]` |
| `onnx` | `["contextra-infer-onnx/onnx", "contextra-db/onnx"]` |
| `candle` | `["contextra-infer-candle"]` |
| `ollama` | `["contextra-infer-ollama"]` |
| `router` | `["contextra-router", "contextra-rank"]` |

### Öffentliche Datentypen — Structs (9)

| Struct | Felder / Form | Beschreibung | Datei |
|---|---|---|---|
| `AgentMemory` | `{ engine: Arc<Contextra>, id_gen: Option<Arc<dyn IdGen>>, }` | High-level facade for agent-oriented memory management (`remember`, `recall`, `forget`, `relate`). `AgentMemory` wraps an underlying `Contextra` engi… | `src/agent_memory.rs` |
| `CollectionProfile` | `{ pub performance: PerformanceProfile, pub kv_delete_mode: KvDeleteMode, pub lsm_tuning: LsmTuning, pub auto_extraction: AutoExtractionMode, }` | — | `src/collection_profile.rs` |
| `ContextraBuilder` | `{ storage_path: PathBuf, config: ContextraConfig, explicit_user_config: bool, embedder: Option<Arc<dyn TextEmbeddingEngine>>, license_gate: Arc<dyn LicenseGate>,… }` | A builder for configuring and instantiating `Contextra`. `ContextraBuilder` acts as the primary builder pattern in the `contextra` facade (Ring 4). I… | `src/builder.rs` |
| `LlmQueryRewriter` | `{ generator: Arc<dyn LlmTextGenerator>, max_subqueries: usize, max_context_results: usize, max_snippet_chars: usize, }` | LLM-supported query rewriter for multi-step retrieval. | `src/query_rewriter.rs` |
| `LsmTuning` | `{ pub memtable_size_limit: u64, pub max_ram_mb: u32, pub group_commit_window_micros: u32, pub block_cache_shards: u16, }` | — | `src/collection_profile.rs` |
| `Memory` | `{ /// Document identifier string. pub id: String, /// Relevance / similarity score. pub score: f32, /// Optional metadata associated with the memory entry. pub m… }` | Representation of a recalled memory record. Derived from `SearchResult` provided by `Contextra::search_text` or hybrid queries. | `src/agent_memory.rs` |
| `MemoryId` | `(tuple/unit)` | Opaque identifier for a memory stored via `AgentMemory`. Wraps the underlying document ID string used by `Contextra`. | `src/agent_memory.rs` |
| `RelationId` | `(tuple/unit)` | Opaque identifier for an n-ary relation (hyperedge) stored via `AgentMemory::relate_n_ary`. Wraps the underlying 64-bit hyperedge ID generated by the… | `src/agent_memory.rs` |
| `ResolvedProfileConfig` | `{ pub durability_mode: DurabilityMode, pub vector_delete_mode: VectorDeleteMode, pub deletion_proof_active: bool, pub feature_ring: FeatureRing, }` | — | `src/performance_profile.rs` |

### Öffentliche Datentypen — Enums (4)

| Enum | Varianten | Beschreibung | Datei |
|---|---|---|---|
| `CollectionProfileError` | `#[error("performance profile invalid: {0}")] Performance(#[from] PerformanceProfileError), #[error("deletion-proof active but KvDeleteMode::TombstoneOnly cannot satisfy a KV-side DeletionProof")] KvD…` | — | `src/collection_profile.rs` |
| `DeploymentTier` | `EdgeMinimal, PowerUserLocal, EnterpriseShared, EnterpriseRegulated,` | — | `src/collection_profile.rs` |
| `PerformanceProfile` | `Compliance, Balanced, BareMetal,` | — | `src/performance_profile.rs` |
| `PerformanceProfileError` | `#[error("durability configuration invalid: {0}")] Durability(#[from] DurabilityConfigError), #[error("DurabilityMode::Full combined with VectorDeleteMode::BackgroundRepair while deletion-proof is act…` | — | `src/performance_profile.rs` |

### Konstanten (2)

| Konstante | Typ | Beschreibung |
|---|---|---|
| `DEFAULT_AUTO_EXTRACTION_MODE` | `AutoExtractionMode` | Default auto extraction mode for standard deployment tiers. |
| `ENTERPRISE_REGULATED_AUTO_EXTRACTION_DEFAULT` | `AutoExtractionMode` | In EnterpriseRegulated wird kein Dokumenttext automatisch an ein Extraktions-LLM gegeben, solange der Betreiber es nicht ausdrücklich aktiviert (Date… |

### Öffentliche Funktionen (51)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn as_str(&self) -> &str` | Returns a string slice reference to the underlying memory ID. | `src/agent_memory.rs` |
| `pub fn engine(&self) -> &Contextra` | Returns a reference to the underlying `Contextra` engine instance. | `src/agent_memory.rs` |
| `pub async fn forget(&self, id: &MemoryId) -> Result<(), ContextraError>` | Deletes a memory entry by its `MemoryId`. Delegates directly to `Contextra::delete`. | `src/agent_memory.rs` |
| `pub fn new(id: impl Into<String>) -> Self` | Creates a new `MemoryId` from a string identifier. | `src/agent_memory.rs` |
| `pub fn new(engine: Contextra) -> Self` | Constructs a new `AgentMemory` wrapper around a `Contextra` engine instance. | `src/agent_memory.rs` |
| `pub fn new_arc(engine: Arc<Contextra>) -> Self` | Constructs a new `AgentMemory` wrapper around an `Arc<Contextra>` engine instance. | `src/agent_memory.rs` |
| `pub fn new_with_id_gen(engine: Contextra, id_gen: Arc<dyn IdGen>) -> Self` | Constructs a new `AgentMemory` wrapper around a `Contextra` engine instance with an explicit ID generator. | `src/agent_memory.rs` |
| `pub async fn recall(&self, query: &str, k: usize) -> Result<Vec<Memory>, ContextraError>` | Retrieves top-k matching memories for a natural language query text string. Delegates directly to `Contextra::search_text` and transforms e… | `src/agent_memory.rs` |
| `pub async fn relate( &self, from: &MemoryId, to: &MemoryId, label: &str, ) -> Result<(), ContextraError>` | Establishes a directed knowledge-graph relationship between two memories with a given label. Delegates directly to `Contextra::relate`. | `src/agent_memory.rs` |
| `pub async fn relate_n_ary( &self, predicate: &str, participants: &[(&MemoryId, &str)], source: Option<&MemoryId>, ) -> Result<RelationId, ContextraError>` | Establishes an n-ary hyperedge relationship connecting two or more memories, each bound to a specific role. # Role Semantics The underlying… | `src/agent_memory.rs` |
| `pub async fn remember( &self, text: &str, metadata: Option<Value>, ) -> Result<MemoryId, ContextraError>` | Stores a text memory entry with optional JSON metadata. Generates a unique memory identifier via the injected `IdGen` if available, or fall… | `src/agent_memory.rs` |
| `pub fn with_id_gen(mut self, id_gen: Arc<dyn IdGen>) -> Self` | Builder method to inject or update the ID generator on an `AgentMemory` instance. | `src/agent_memory.rs` |
| `pub async fn build(mut self) -> Result<Contextra, ContextraError>` | Builds and initializes the `Contextra` engine instance. | `src/builder.rs` |
| `pub fn config(&self) -> &ContextraConfig` | Returns a reference to the inner [`ContextraConfig`]. | `src/builder.rs` |
| `pub fn from_config(config: ContextraConfig) -> Self` | Creates a `ContextraBuilder` pre-populated from a `ContextraConfig`. | `src/builder.rs` |
| `pub fn from_profile(profile: crate::collection_profile::CollectionProfile) -> Self` | Creates a `ContextraBuilder` configured from a `CollectionProfile`. | `src/builder.rs` |
| `pub fn from_signed_license( payload_bytes: &[u8], signature: &[u8; 64], verifying_key_bytes: &[u8; 32], ) -> Self` | Creates a `ContextraBuilder` initialized with a cryptographic signed license gate (§14.6, §15). | `src/builder.rs` |
| `pub fn from_tier(tier: crate::collection_profile::DeploymentTier) -> Self` | Creates a `ContextraBuilder` configured for a specific `DeploymentTier`. | `src/builder.rs` |
| `pub fn license_gate(&self) -> &Arc<dyn LicenseGate>` | Returns a reference to the configured license gate. | `src/builder.rs` |
| `pub fn new(dimension: usize) -> Self` | Creates a new `ContextraBuilder` with specified vector dimension. | `src/builder.rs` |
| `pub fn performance_profile(&self) -> Option<crate::performance_profile::PerformanceProfile>` | Returns the configured performance profile preset, if any. | `src/builder.rs` |
| `pub fn storage_path(&self) -> &std::path::Path` | Returns the configured storage path. | `src/builder.rs` |
| `pub fn with_collection_profile( self, profile: crate::collection_profile::CollectionProfile, ) -> Self` | Attaches a `CollectionProfile` and pre-configures performance profile settings. | `src/builder.rs` |
| `pub fn with_config(self, config: ContextraConfig) -> Self` | Sets the complete `ContextraConfig` directly. | `src/builder.rs` |
| `pub fn with_consolidation(mut self, enabled: bool, interval: Duration) -> Self` | Configures memory consolidation behavior. | `src/builder.rs` |
| `pub fn with_distance_metric(mut self, metric: DistanceMetric) -> Self` | Sets the distance metric for vector similarity search. | `src/builder.rs` |
| `pub fn with_embedder(mut self, embedder: Arc<dyn TextEmbeddingEngine>) -> Self` | Attaches a custom text embedding engine implementation. | `src/builder.rs` |
| `pub fn with_embedding_backend(mut self, backend: EmbeddingBackend) -> Self` | Sets the embedding backend configuration. | `src/builder.rs` |
| `pub fn with_encryption_passphrase(mut self, passphrase: impl Into<String>) -> Self` | Sets the encryption passphrase for persistent storage. | `src/builder.rs` |
| `pub fn with_license_gate(mut self, gate: Arc<dyn LicenseGate>) -> Self` | Attaches a license gate for feature ring authorization. | `src/builder.rs` |
| `pub fn with_max_elements(mut self, max_elements: usize) -> Self` | Sets the maximum number of vector elements stored. | `src/builder.rs` |
| `pub fn with_performance_profile( mut self, profile: crate::performance_profile::PerformanceProfile, ) -> Self` | Sets the performance profile preset for the contextra instance. | `src/builder.rs` |
| `pub fn with_signed_license( mut self, payload_bytes: &[u8], signature: &[u8; 64], verifying_key_bytes: &[u8; 32], ) -> Self` | Attaches a cryptographic signed license gate (§14.6, §15). | `src/builder.rs` |
| `pub fn with_storage_path(mut self, path: impl Into<PathBuf>) -> Self` | Sets the storage directory path for the database engine. | `src/builder.rs` |
| `pub fn resolve(self) -> CollectionProfile` | — | `src/collection_profile.rs` |
| `pub fn validate(&self) -> Result<(), CollectionProfileError>` | — | `src/collection_profile.rs` |
| `pub fn validate_with_license( &self, gate: &dyn LicenseGate, ) -> Result<(), CollectionProfileError>` | — | `src/collection_profile.rs` |
| `pub fn builder(dimension: usize) -> ContextraBuilder` | Creates a new `ContextraBuilder` for configuring and instantiating `Contextra`. | `src/lib.rs` |
| `pub async fn open(path: impl AsRef<std::path::Path>) -> Result<Contextra, ContextraError>` | Opens or creates a `Contextra` instance at the given storage path using default configuration and `OpenFastGate`. | `src/lib.rs` |
| `pub async fn open_with_config( path: impl AsRef<std::path::Path>, config: ContextraConfig, ) -> Result<Contextra, ContextraError>` | Opens or creates a `Contextra` instance at the given storage path with an explicit configuration and `OpenFastGate`. | `src/lib.rs` |
| `pub fn enforce_license(&self, gate: &dyn LicenseGate) -> Result<(), PerformanceProfileError>` | — | `src/performance_profile.rs` |
| `pub fn resolve(self) -> ResolvedProfileConfig` | — | `src/performance_profile.rs` |
| `pub fn validate(&self) -> Result<(), PerformanceProfileError>` | — | `src/performance_profile.rs` |
| `pub fn max_context_results(&self) -> usize` | Returns the maximum number of previous context search results to include in the prompt. | `src/query_rewriter.rs` |
| `pub fn max_snippet_chars(&self) -> usize` | Returns the maximum number of characters per result snippet in the prompt. | `src/query_rewriter.rs` |
| `pub fn max_subqueries(&self) -> usize` | Returns the maximum number of alternative sub-queries to generate. | `src/query_rewriter.rs` |
| `pub fn new(generator: Arc<dyn LlmTextGenerator>) -> Self` | Creates a new `LlmQueryRewriter` with default limits: - `max_subqueries`: 3 - `max_context_results`: 3 - `max_snippet_chars`: 300 | `src/query_rewriter.rs` |
| `pub fn new_with_limits( generator: Arc<dyn LlmTextGenerator>, max_subqueries: usize, max_context_results: usize, max_snippet_chars: usize, ) -> Self` | Creates a new `LlmQueryRewriter` with explicit limits. | `src/query_rewriter.rs` |
| `pub fn with_max_context_results(mut self, n: usize) -> Self` | Sets the maximum number of previous context search results to include in the prompt. | `src/query_rewriter.rs` |
| `pub fn with_max_snippet_chars(mut self, n: usize) -> Self` | Sets the maximum number of characters per result snippet in the prompt. | `src/query_rewriter.rs` |
| `pub fn with_max_subqueries(mut self, n: usize) -> Self` | Sets the maximum number of alternative sub-queries to generate. | `src/query_rewriter.rs` |


---

## `contextra-mcp`

| Attribut | Wert |
|---|---|
| **Ring** | 4 — Ring 4 — Boundary & Composition Roots |
| **Status** | In Migration |
| **Zweck** | Stdio-JSON-RPC MCP-Server-Protokoll-Adapter. |
| **Unsafe-Code** | Nein (`#![forbid(unsafe_code)]`) |
| **Interne Abhängigkeiten** | `contextra`, `contextra-adapt`, `contextra-agent`, `contextra-crypto`, `contextra-infer-candle`, `contextra-infer-ollama`, `contextra-infer-onnx`, `contextra-license`, `contextra-ports`, `contextra-privacy`, `contextra-rank`, `contextra-types`, `contextra-wire` |
| **Externe Abhängigkeiten (Auswahl)** | `futures-util`, `parking_lot`, `rand`, `scc`, `serde`, `serde_json`, `thiserror`, `tokio`, `tokio-util`, `tracing`, `tracing-subscriber`, `unicode-normalization`, `zeroize` |

### Modul- und Re-Export-Oberfläche (lib.rs)

```rust
pub mod bulk_exfiltration_detector;
pub mod config;
pub mod egress_gateway;
pub mod egress_guard;
pub mod explain;
pub mod io;
pub mod plugin_status;
pub mod prompt_injection;
pub mod proof_key;
pub mod proof_key_env;
pub mod protocol;
pub mod routing;
pub mod sandbox;
pub mod server;
pub mod server_dispatch;
pub mod server_tools;
pub mod tools_crud;
pub mod validation;
pub use bulk_exfiltration_detector::{ BulkExfiltrationDetector, BulkExfiltrationOutcome, SessionId, };
pub use config::*;
pub use egress_gateway::{ CloudQueryRequest, CloudQueryResponse, DefaultEgressClassifier, EgressClassification, EgressClassifier, };
pub use explain::{ExplainRequest, ExplainResponse};
pub use plugin_status::{handle_plugin_status, PluginStatusEntry, PluginStatusResponse};
pub use proof_key::deletion_proof_key_from_env;
pub use io::{read_line_bounded, MAX_RPC_BYTES, MAX_SEARCH_QUERY_BYTES};
pub use prompt_injection::{ PromptInjectionConfig, PromptInjectionGuard, QuarantinePolicy, SecurityAuditLogger, SecurityAuditRecord, DEFAULT_REDACTION_PLACEHOLDER, };
pub use routing::setup_kv_bridge;
pub use routing::{setup_routing, RoutingHandle};
pub use server::McpServer;
pub use validation::{is_write_allowed_by_env, validate_collection_name};
```

### Feature-Flags

| Flag | Aktiviert |
|---|---|
| `default` | `[]` |
| `agent-workflows` | `["contextra-agent"]` |
| `onnx` | `["contextra-infer-onnx", "contextra-infer-onnx/onnx"]` |
| `candle` | `["dep:contextra-infer-candle"]` |
| `ollama` | `["dep:contextra-infer-ollama", "contextra/ollama"]` |
| `kv-bridge` | `["contextra-infer-candle/kv-bridge", "dep:contextra-infer-candle"]` |
| `test-utils` | `[]` |

### Öffentliche Datentypen — Structs (20)

| Struct | Felder / Form | Beschreibung | Datei |
|---|---|---|---|
| `EmbeddingConfig` | `{ /// Provider type ("ollama", "onnx", "candle", or "mock"). pub provider: String, /// Base URL for Ollama HTTP API. pub ollama_url: String, /// Model identifier… }` | Embedding provider configuration settings. | `src/config.rs` |
| `ExplainRequest` | `{ /// Target document identifier. pub id: String, /// Target collection name (defaults to "default"). #[serde(default = "default_collection")] pub collection: St… }` | Request payload for `contextra_explain`. | `src/explain.rs` |
| `ExplainResponse` | `{ /// Document identifier. pub id: String, /// Collection name. pub collection: String, /// Detailed provenance record if available. #[serde(default, skip_serial… }` | Response payload for `contextra_explain`. | `src/explain.rs` |
| `LlmConfig` | `{ /// Provider type ("ollama", "candle", or "mock"). pub provider: String, /// Base URL for Ollama HTTP API. pub ollama_url: String, /// Model identifier for Oll… }` | LLM provider configuration settings. | `src/config.rs` |
| `McpSandbox` | `{ policy: SandboxPolicy, /// Session-lokale volatile Ergebnisse (werden bei Session-Ende gedropt). volatile_results: Mutex<HashMap<String, VolatileToolResult>>,… }` | MCP Sandbox: validiert Tool-Calls und verwaltet volatile Ergebnisse. | `src/sandbox.rs` |
| `McpServer` | `{ pub db: Arc<Contextra>, pub embedder: Arc<dyn EmbeddingProvider>, pub sandbox: Arc<McpSandbox>, pub injection_guard: Arc<PromptInjectionGuard>, pub egress_clas… }` | — | `src/server.rs` |
| `MockLlmGenerator` | `(tuple/unit)` | Fallback Mock LLM Text Generator. | `src/config.rs` |
| `NormalizedPattern` | `{ pub original: String, pub collapsed: String, pub no_ws: String, }` | Vornormalisiertes Injection-Pattern zur Performance-Optimierung. | `src/prompt_injection/guard.rs` |
| `PluginStatusEntry` | `{ pub name: String, pub version: String, pub ring: u8, pub feature_ring_required: contextra_ports::license::FeatureRing, }` | Status entry describing an active plugin. | `src/plugin_status.rs` |
| `PluginStatusResponse` | `{ pub plugins: Vec<PluginStatusEntry>, pub feature_ring_active: contextra_ports::license::FeatureRing, }` | Response payload for the `contextra_plugin_status` MCP tool. SECURITY BOUNDARY NOTICE: This response strictly contains high-level feature ring indica… | `src/plugin_status.rs` |
| `PromptInjectionConfig` | `{ #[serde(default)] pub policy: QuarantinePolicy, #[serde(default = "default_redaction_placeholder")] pub redaction_placeholder: String, #[serde(default)] pub au… }` | Konfiguration zur Initialisierung aus einer externen Konfigurationsdatei. | `src/prompt_injection/policy.rs` |
| `PromptInjectionGuard` | `{ policy: QuarantinePolicy, pub redaction_placeholder: String, patterns: Vec<String>, normalized_patterns: Vec<NormalizedPattern>, audit_logger: SecurityAuditLog… }` | Signatur- und phrasenbasierter Prompt-Injection-Erkennungsfilter. Erkennt bekannte Angriffsmuster (direkte Phrasen, whitespace-verschleierte Variante… | `src/prompt_injection/guard.rs` |
| `ProofKeyResolution` | `{ pub key: Option<Zeroizing<String>>, pub source: Option<ProofKeySource>, pub has_conflict: bool, pub used_legacy: bool, }` | Ergebnis der Schlüssel-Auflösung. | `src/proof_key_env.rs` |
| `RouterConfig` | `{ /// List of SLM profiles. If non-empty, routing is enabled. pub profiles: Vec<contextra::router::SlmProfile>, /// Optional path to routing profiles JSON file.… }` | Router configuration settings. | `src/config.rs` |
| `RoutingHandle` | `{ pub router: Arc<contextra::router::DefaultRouterEngine>, pub calibrator: Arc<parking_lot::Mutex<contextra_rank::IsotonicCalibrator>>, pub pid_controller: Arc<p… }` | Holds strong Arc references to routing and calibration components to maintain live Weak references in `Contextra`. | `src/routing.rs` |
| `SandboxPolicy` | `{ pub allow_db_reads: bool, pub allow_db_writes: bool, pub allow_code_execution: bool, /// Cloud-Egress erlaubt? Standardmäßig `false` (additiv zu Layer 1/4 Egre… }` | Konfiguration der Sandbox-Policy. | `src/sandbox.rs` |
| `SecurityAuditLogger` | `{ file_path: Option<PathBuf>, in_memory_records: Arc<Mutex<Vec<SecurityAuditRecord>>>, }` | Separater Audit-Logger für Sicherheitsvorfälle (vollständig isoliert vom Vektor-Index). | `src/prompt_injection/audit.rs` |
| `SecurityAuditRecord` | `{ pub timestamp: String, pub event_type: String, pub doc_id: String, pub collection: String, pub pattern_matched: String, pub action_taken: String, }` | Struktur für Sicherheits-Audit-Logeinträge bei Auslösung des `Escalate`-Modus. | `src/prompt_injection/audit.rs` |
| `ToolDefinition` | `{ pub name: &'static str, pub category: ToolCategory, pub description: &'static str, pub input_schema: fn() -> Value, }` | Definition eines MCP-Tools mit Name, Kategorie, Beschreibung und JSON-Schema. | `src/sandbox.rs` |
| `VolatileToolResult` | `{ /// Verschlüsselte Ausgabe (AES-256-GCM-SIV via contextra-security). encrypted: zeroize::Zeroizing<Vec<u8>>, /// Klartext-Nonce (nicht sensitiv). nonce: Vec<u8… }` | Volatiler Tool-Output-Speicher mit Zeroize-Garantie bei Drop. Tool-Ergebnisse werden verschlüsselt im Arbeitsspeicher gehalten. Beim Drop wird der Sp… | `src/sandbox.rs` |

### Öffentliche Datentypen — Enums (4)

| Enum | Varianten | Beschreibung | Datei |
|---|---|---|---|
| `McpError` | `#[error("{0}")] ParseError(String), #[error("{0}")] InvalidRequest(String), #[error("{0}")] MethodNotFound(String), #[error("{0}")] InvalidParams(String), #[error("{0}")] InternalError(String),` | MCP Error representation matching standard JSON-RPC 2.0 error codes. | `src/protocol.rs` |
| `ProofKeySource` | `Primary, Legacy, BothSame, BothDifferentConflict,` | Quelle des aufgelösten Schlüssels. | `src/proof_key_env.rs` |
| `QuarantinePolicy` | `/// Verdächtige Texte werden durch einen neutralen Platzhalter ersetzt (Default). #[default] Strict, /// **WARNUNG (UNSICHER)**: Text wird unverändert durchgereicht, lediglich das Flag /// `suspiciou…` | Konfigurierbare Quarantäne-Policy für die Prompt-Injection-Behandlung. **SICHERHEITSHINWEIS & HEURISTIK-LIMITATION**: Pattern-Matching und Normalisie… | `src/prompt_injection/policy.rs` |
| `ToolCategory` | `/// Read-only Datenbankoperationen (immer erlaubt). DatabaseRead, /// Schreibende Datenbankoperationen (erfordern explizite Freigabe). DatabaseWrite, /// Externe Code-Ausführung (höchste Risikostufe,…` | Erlaubte MCP-Tool-Kategorien (Whitelist-Prinzip). | `src/sandbox.rs` |

### Konstanten (8)

| Konstante | Typ | Beschreibung |
|---|---|---|
| `DEFAULT_REDACTION_PLACEHOLDER` | `&str` | Neutraler Standard-Platzhalter für als verdächtig erkannten Text im Strict/Escalate-Modus. |
| `MAX_RECURSION_DEPTH` | `usize` | — |
| `MAX_RPC_BYTES` | `usize` | Maximum allowed single JSON-RPC message size via stdio (4 MB). |
| `MAX_SEARCH_QUERY_BYTES` | `usize` | Maximum allowed search query length in bytes (64 KB). |
| `MAX_VOLATILE_KEY_BYTES` | `usize` | Maximale Länge eines Volatile-Result-Schlüssels in Bytes. |
| `MAX_VOLATILE_OUTPUT_BYTES` | `usize` | Maximale Größe einer volatilen Tool-Ausgabe in Bytes (16 MB). |
| `MAX_VOLATILE_RESULTS` | `usize` | Maximale Anzahl von volatilen Ergebnissen pro Sandbox-Session. |
| `TOOL_REGISTRY` | `&[ToolDefinition]` | Single Source of Truth für MCP-Tool-Registrierung, Discovery (`tools/list`) und -Klassifizierung. ## Tool-Kategorie-Tabelle | Werkzeug | Kategorie |… |

### Öffentliche Funktionen (76)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn build_generator(&self) -> Result<Arc<dyn LlmTextGenerator>, ContextraError>` | Instantiates the configured `LlmTextGenerator` as an `Arc<dyn LlmTextGenerator>`. | `src/config.rs` |
| `pub fn build_provider(&self) -> Result<Arc<dyn EmbeddingProvider>, ContextraError>` | Instantiates the configured `EmbeddingProvider` as an `Arc<dyn EmbeddingProvider>`. | `src/config.rs` |
| `pub fn create_embedding_provider( provider_type: &str, ollama_url: &str, embed_model: &str, onnx_model_path: Option<&Path>, candle_model_dir: Option<&Path>, ) -> Result<Arc<dyn EmbeddingProvider>, ContextraError>` | Dynamically constructs an `EmbeddingProvider` implementation based on provider identifier. | `src/config.rs` |
| `pub fn create_llm_text_generator( provider_type: &str, ollama_url: &str, llm_model: &str, candle_model_dir: Option<&Path>, ) -> Result<Arc<dyn LlmTextGenerator>, ContextraError>` | Dynamically constructs an `LlmTextGenerator` implementation based on provider identifier. | `src/config.rs` |
| `pub fn from_env() -> Self` | Loads configuration from environment variables with fallbacks. | `src/config.rs` |
| `pub fn from_env() -> Self` | Loads configuration from environment variables with fallbacks. | `src/config.rs` |
| `pub fn from_env() -> Self` | Loads configuration from environment variables with fallbacks. | `src/config.rs` |
| `pub async fn read_line_bounded<R: tokio::io::AsyncBufRead + Unpin>( reader: &mut R, buf: &mut String, max_bytes: usize, ) -> std::io::Result<usize>` | Reads a single line from an async reader into `buf` up to `max_bytes`. If the line exceeds `max_bytes`, consumes and discards the remainder… | `src/io.rs` |
| `pub async fn handle_plugin_status( registry: &contextra_ports::plugin::PluginRegistry, ) -> Result<PluginStatusResponse, McpError>` | Handler for the `contextra_plugin_status` MCP tool call. | `src/plugin_status.rs` |
| `pub fn get_recorded_events(&self) -> Vec<SecurityAuditRecord>` | — | `src/prompt_injection/audit.rs` |
| `pub fn log_event(&self, record: SecurityAuditRecord)` | — | `src/prompt_injection/audit.rs` |
| `pub fn new(file_path: Option<PathBuf>) -> Self` | — | `src/prompt_injection/audit.rs` |
| `pub fn audit_logger(&self) -> &SecurityAuditLogger` | — | `src/prompt_injection/guard.rs` |
| `pub fn clock(&self) -> &Arc<dyn Clock>` | — | `src/prompt_injection/guard.rs` |
| `pub fn collapse_whitespace(s: &str) -> String` | Collapsiert aufeinanderfolgende Whitespaces zu einem einzelnen Leerzeichen. | `src/prompt_injection/guard.rs` |
| `pub fn decode_base64(input: &str) -> Option<Vec<u8>>` | Versucht, einen Base64-String (Standard oder URL-Safe) ohne Panics zu dekodieren. | `src/prompt_injection/guard.rs` |
| `pub fn decode_html_entities_view(text: &str) -> String` | Minimale HTML-Entity-Dekodierung (`&#DEC;`, `&#xHEX;` und gängige benannte Entities). | `src/prompt_injection/guard.rs` |
| `pub fn decode_leetspeak_view(text: &str) -> String` | Generische Leetspeak-Rückübersetzung (Ziffern und Sonderzeichen zu Buchstaben). | `src/prompt_injection/guard.rs` |
| `pub fn default_patterns() -> Vec<String>` | Gibt die Standard-Erkennungsmuster zurück. Enthält: - 16 englische Kern-Patterns (Token-Marker, direkte Befehle) - 9 deutsche Patterns (Pro… | `src/prompt_injection/guard.rs` |
| `pub fn detect(&self, text: &str) -> Option<String>` | Prüft den Eingabetext auf bekannte Prompt-Injection-Muster unter Verwendung von Normalisierung, Leetspeak-Rückübersetzung, HTML-Entity-Deko… | `src/prompt_injection/guard.rs` |
| `pub fn detect_recursive(&self, text: &str, depth: usize) -> Option<String>` | Rekursive Injektions-Erkennung mit harter Tiefenbegrenzung (max. Tiefe 2 für Base64-Dekodierung). | `src/prompt_injection/guard.rs` |
| `pub fn extract_base64_candidates(text: &str) -> Vec<&str>` | Extrahiert kandidate Base64-Substrings (Länge >= 16) aus einem Eingabetext. | `src/prompt_injection/guard.rs` |
| `pub fn from_env() -> Self` | Erstellt eine Instanz basierend auf Umgebungsvariablen. | `src/prompt_injection/guard.rs` |
| `pub fn is_markdown_formatting_char(c: char) -> bool` | Prüft ob ein Zeichen ein Markdown-Inline-Formatierungszeichen ist. | `src/prompt_injection/guard.rs` |
| `pub fn is_zero_width(c: char) -> bool` | Prüft ob ein Zeichen ein Zero-Width- oder unsichtbares Steuer-Zeichen ist. | `src/prompt_injection/guard.rs` |
| `pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, String>` | Lädt die Konfiguration aus einer externen JSON- oder TOML-Datei. | `src/prompt_injection/guard.rs` |
| `pub fn new( policy: QuarantinePolicy, redaction_placeholder: String, patterns: Vec<String>, audit_logger: SecurityAuditLogger, ) -> Self` | — | `src/prompt_injection/guard.rs` |
| `pub fn normalize_text(text: &str) -> String` | Normalisiert den Eingabetext (Zero-Width-Stripping, NFKC Normalisierung, Skeletonisierung, Lowercasing). | `src/prompt_injection/guard.rs` |
| `pub fn patterns(&self) -> &[String]` | — | `src/prompt_injection/guard.rs` |
| `pub fn policy(&self) -> QuarantinePolicy` | — | `src/prompt_injection/guard.rs` |
| `pub fn process_result( &self, doc_id: &str, collection: &str, obj: &mut serde_json::Map<String, serde_json::Value>, ) -> bool` | Prüft und verarbeitet das JSON-Ergebnisobjekt eines Such- oder Get-Aufrufs. Wendet die konfigurierte `QuarantinePolicy` an und liefert `tru… | `src/prompt_injection/guard.rs` |
| `pub fn skeletonize_char(c: char) -> char` | Mappt gängige kyrillische und griechische Homoglyphen (Confusables) auf ihre lateinischen Äquivalente. | `src/prompt_injection/guard.rs` |
| `pub fn strip_markdown_inline_formatting_view(text: &str) -> String` | Entfernt Inline-Markdown-Formatierungszeichen (`*`, `_`, `` ` ``, `~`) innerhalb oder an Wörtern. | `src/prompt_injection/guard.rs` |
| `pub fn strip_whitespace(s: &str) -> String` | Entfernt sämtliche Whitespaces vollständig (zur Erkennung von Zeichen-Einstreuung). | `src/prompt_injection/guard.rs` |
| `pub fn with_clock(mut self, clock: Arc<dyn Clock>) -> Self` | — | `src/prompt_injection/guard.rs` |
| `pub fn default_redaction_placeholder() -> String` | — | `src/prompt_injection/policy.rs` |
| `pub fn from_env() -> Self` | — | `src/prompt_injection/policy.rs` |
| `pub fn deletion_proof_key_from_env() -> Result<Zeroizing<String>, McpError>` | Liest den DeletionProof-Schlüssel aus den Prozess-Umgebungsvariablen (`CONTEXTRA_DELETION_PROOF_KEY` und `CONTEXTRA_PROOF_KEY`), ruft die r… | `src/proof_key.rs` |
| `pub fn resolve_proof_key(primary: Option<String>, legacy: Option<String>) -> ProofKeyResolution` | Reine Hilfsfunktion zur Auflösung des DeletionProof-Schlüssels ohne Zugriff auf die Prozess-Umgebung. Leerstrings oder rein aus Whitespace… | `src/proof_key_env.rs` |
| `pub fn resolve_proof_key_from_env() -> ProofKeyResolution` | Liest beide Umgebungsvariablen (`CONTEXTRA_DELETION_PROOF_KEY` und `CONTEXTRA_PROOF_KEY`), löst den Schlüssel per `resolve_proof_key` auf u… | `src/proof_key_env.rs` |
| `pub fn code(&self) -> i32` | — | `src/protocol.rs` |
| `pub fn internal_error(msg: impl Into<String>) -> Self` | — | `src/protocol.rs` |
| `pub fn invalid_params(msg: impl Into<String>) -> Self` | — | `src/protocol.rs` |
| `pub fn invalid_request(msg: impl Into<String>) -> Self` | — | `src/protocol.rs` |
| `pub fn method_not_found(msg: impl Into<String>) -> Self` | — | `src/protocol.rs` |
| `pub fn parse_error(msg: impl Into<String>) -> Self` | — | `src/protocol.rs` |
| `pub fn response_from_error(id: Option<Value>, err: McpError) -> JsonRpcResponse` | Helper function to convert an `McpError` directly into a `JsonRpcResponse`. | `src/protocol.rs` |
| `pub fn setup_kv_bridge( db: &Arc<Contextra>, ) -> Option<Arc<contextra_infer_candle::KvBridgeAdapter>>` | Conditionally sets up `KvBridgeAdapter` when feature `kv-bridge` is enabled. | `src/routing.rs` |
| `pub async fn setup_routing( db: &Arc<Contextra>, config: &RouterConfig, ) -> Result<Option<Arc<RoutingHandle>>, ContextraError>` | — | `src/routing.rs` |
| `pub fn classify_method(method: &str) -> ToolCategory` | — | `src/sandbox.rs` |
| `pub fn decrypt( &self, key: &contextra_crypto::CryptoKey, ) -> Result<zeroize::Zeroizing<Vec<u8>>>` | Entschlüsselt und gibt den Klartext zeroized zurück. | `src/sandbox.rs` |
| `pub fn encode(bytes: [u8; 32]) -> String` | — | `src/sandbox.rs` |
| `pub fn encrypt(plaintext: &[u8], key: &contextra_crypto::CryptoKey) -> Result<Self>` | Speichert einen Tool-Output verschlüsselt. | `src/sandbox.rs` |
| `pub async fn execute_with_timeout<F, T, E>( &self, tool_name: &str, fut: F, ) -> std::result::Result<T, E> where F: std::future::Future<Output = std::result::Result<T, E>>, E: From<ContextraError>,` | Führt eine Asynchrone Tool-Future mit Timeout gemäß SandboxPolicy aus. | `src/sandbox.rs` |
| `pub fn get_volatile(&self, key: &str) -> Result<Option<zeroize::Zeroizing<Vec<u8>>>>` | Ruft einen verschlüsselten Tool-Output ab und entschlüsselt ihn. | `src/sandbox.rs` |
| `pub fn new(policy: SandboxPolicy) -> Result<Self>` | Erstellt eine neue Sandbox-Instanz mit frischem Sitzungsschlüssel. | `src/sandbox.rs` |
| `pub fn policy(&self) -> &SandboxPolicy` | Gibt die aktuell konfigurierte SandboxPolicy zurück. | `src/sandbox.rs` |
| `pub fn store_volatile(&self, key: &str, output: &[u8]) -> Result<()>` | Speichert einen Tool-Output verschlüsselt in der Session. | `src/sandbox.rs` |
| `pub fn try_classify_method(method: &str) -> Result<ToolCategory>` | Klassifiziert den MCP-Tool-Namen strikt anhand von `TOOL_REGISTRY`. Gibt `Err(ContextraError::InvalidInput("unknown tool"))` zurück, falls… | `src/sandbox.rs` |
| `pub fn validate_tool_call(&self, method: &str, _params: &Value) -> Result<()>` | Validiert ob ein Tool-Call erlaubt ist. HINWEIS: Unbekannte Tool-Namen (die nicht in `TOOL_REGISTRY` gelistet sind) werden sofort per `try_… | `src/sandbox.rs` |
| `pub fn from_env( db: Arc<Contextra>, embedder: Arc<dyn EmbeddingProvider>, ) -> Result<Self, ContextraError>` | Instantiates an `McpServer` fully configured from environment variables and default runtime components. | `src/server.rs` |
| `pub fn get_volatile_output( &self, key: &str, ) -> Result<Option<zeroize::Zeroizing<Vec<u8>>>, ContextraError>` | Retrieves and decrypts a volatile tool result stored in the MCP sandbox. | `src/server.rs` |
| `pub fn new( db: Arc<Contextra>, embedder: Arc<dyn EmbeddingProvider>, ) -> Result<Self, ContextraError>` | — | `src/server.rs` |
| `pub async fn run_stdio(self: Arc<Self>) -> Result<(), Box<dyn std::error::Error>>` | Startet den MCP stdio-Loop. Liest zeilenweise von stdin, dispatcht JSON-RPC-Requests und schreibt Antworten als einzelne JSON-Zeile nach st… | `src/server.rs` |
| `pub fn with_egress_classifier(mut self, classifier: Arc<dyn EgressClassifier>) -> Self` | — | `src/server.rs` |
| `pub fn with_egress_guard(mut self, guard: Arc<dyn EgressGuardCheck>) -> Self` | — | `src/server.rs` |
| `pub fn with_injection_guard(mut self, injection_guard: Arc<PromptInjectionGuard>) -> Self` | — | `src/server.rs` |
| `pub fn with_kv_bridge( mut self, kv_bridge: Option<Arc<contextra_infer_candle::KvBridgeAdapter>>, ) -> Self` | — | `src/server.rs` |
| `pub fn with_plugin_registry( mut self, registry: Option<Arc<contextra_ports::plugin::PluginRegistry>>, ) -> Self` | — | `src/server.rs` |
| `pub fn with_routing(mut self, routing: Option<Arc<RoutingHandle>>) -> Self` | — | `src/server.rs` |
| `pub fn with_sandbox( db: Arc<Contextra>, embedder: Arc<dyn EmbeddingProvider>, sandbox: Arc<McpSandbox>, ) -> Self` | — | `src/server.rs` |
| `pub fn with_write_permission( db: Arc<Contextra>, embedder: Arc<dyn EmbeddingProvider>, allow_db_writes: bool, ) -> Result<Self, ContextraError>` | — | `src/server.rs` |
| `pub async fn handle(&self, req: JsonRpcRequest) -> JsonRpcResponse` | — | `src/server_dispatch.rs` |
| `pub async fn handle_value(&self, val: Value) -> Option<Value>` | Verarbeitet einen Parsed JSON-RPC 2.0 Request (Einzelanfrage, Notification oder Batch Array). Gibt `Some(Value)` zurück, falls eine Antwort… | `src/server_dispatch.rs` |
| `pub fn is_write_allowed_by_env() -> bool` | Checks whether database write access is explicitly enabled via environment variable `CONTEXTRA_MCP_ALLOW_WRITE`. | `src/validation.rs` |
| `pub fn validate_collection_name(name: &str) -> Result<(), McpError>` | MCP-Server mit stdio-Transport (JSON-RPC 2.0). stdout ist dem Protokoll vorbehalten — Logs gehen ausschließlich nach stderr. | `src/validation.rs` |


---

## `contextra-py`

| Attribut | Wert |
|---|---|
| **Ring** | 4 — Ring 4 — Boundary & Composition Roots |
| **Status** | In Migration |
| **Zweck** | PyO3 Python-FFI-Bindings. |
| **Unsafe-Code** | Nein (`#![forbid(unsafe_code)]`) |
| **Interne Abhängigkeiten** | `contextra-adapt`, `contextra-core`, `contextra-db`, `contextra-rank`, `contextra-router`, `contextra-store`, `contextra-types` |
| **Externe Abhängigkeiten (Auswahl)** | `flatbuffers`, `numpy`, `parking_lot`, `pyo3`, `pythonize`, `serde_json`, `tokio` |

### Architekturbeschreibung

# Contextra Python Bindings  This crate provides the Python bridge for the Contextra embedded hybrid-search database. It utilizes PyO3 for the bridge and NumPy for efficient vector operations.  ## Architecture Role  - **Python Bridge (Layer 3)**: Exposes the core functionality of Contextra to Python. - **Async Orchestration**: Manages a shared Tokio runtime for executing async Rust code from synchronous Python calls. - **Minimal Copying**: Zero-copy borrowing of input vector data from NumPy arrays into Rust; FlatBuffer responses returned as PyBytes.

### Modul- und Re-Export-Oberfläche (lib.rs)

```rust
pub mod bindings;
pub mod kv_links;
pub use bindings::*;
```

### Feature-Flags

| Flag | Aktiviert |
|---|---|
| `default` | `[]` |
| `docid-128` | `["contextra-types/docid-128"]` |

### Öffentliche Datentypen — Structs (8)

| Struct | Felder / Form | Beschreibung | Datei |
|---|---|---|---|
| `PyCollection` | `{ pub(crate) inner: Arc<ContextraCollection>, pub(crate) runtime: Arc<Runtime>, pub(crate) poisoned: Arc<AtomicBool>, }` | — | `src/bindings/collection.rs` |
| `PyContextra` | `{ pub(crate) inner: Arc<Contextra>, pub(crate) runtime: Arc<Runtime>, pub(crate) worker_threads: usize, pub(crate) poisoned: Arc<AtomicBool>, }` | — | `src/bindings/db.rs` |
| `PyDbStats` | `{ /// Lyapunov drift status ("stabil", "warnung", "kritisch", "unbekannt"). pub drift_status: String, /// Expected Calibration Error (ECE) from IsotonicCalibrato… }` | Overall database statistics and system observability. | `src/bindings/db_stats.rs` |
| `PyDocument` | `{ /// The document ID. pub id: String, /// Metadata associated with the document. pub metadata: Option<PyObject>, }` | A document retrieved from Contextra. | `src/bindings/document.rs` |
| `PyRuntimeState` | `{ pub runtime: Arc<Runtime>, pub worker_threads: usize, }` | Holds the per-interpreter/per-module Tokio runtime state and worker thread configuration. | `src/bindings/runtime_state.rs` |
| `PySearchResult` | `{ /// The document ID. pub id: String, /// Similarity score (higher = more similar). pub score: f32, /// Metadata associated with the document. pub metadata: Opt… }` | A single search result from Contextra. | `src/bindings/search_result.rs` |
| `PyStorageStats` | `{ /// Number of SSTable segments. pub num_segments: usize, /// Total size of all SSTables in bytes. pub total_size_bytes: u64, /// Total size of memtables in byt… }` | Statistics for the storage engine. | `src/bindings/storage_stats.rs` |
| `PyVectorIndexStats` | `{ /// Number of active (non-deleted) vectors. pub num_vectors: usize, /// Estimated memory usage in bytes. pub memory_usage_bytes: usize, /// Number of HNSW laye… }` | Statistics for a vector index. | `src/bindings/vector_index_stats.rs` |

### Konstanten (6)

| Konstante | Typ | Beschreibung |
|---|---|---|
| `MAX_BATCH_SIZE` | `usize` | Maximum batch size for batch insertion/upsertion (10,000 items). |
| `MAX_COLLECTION_NAME_LENGTH` | `usize` | Maximum allowed length for collection names (64 characters). |
| `MAX_ID_LENGTH` | `usize` | Maximum allowed length for document string IDs (1024 characters). |
| `MAX_LABEL_LENGTH` | `usize` | Maximum allowed length for relationship labels (256 characters). |
| `MAX_WORKER_THREADS` | `usize` | Maximum allowed worker threads for Tokio multi-thread runtime. |
| `MIN_WORKER_THREADS` | `usize` | Minimum allowed worker threads for Tokio multi-thread runtime. |

### Öffentliche Funktionen (60)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn _trigger_panic_for_test(&self, py: Python<'_>, message: Option<String>) -> PyResult<()>` | Internal helper method for testing FFI panic isolation and engine poisoning. # Panics bewusst; nur Testhook für FFI-Panic-Isolation; Panic… | `src/bindings/collection.rs` |
| `pub fn is_empty(&self, py: Python<'_>) -> PyResult<bool>` | Returns true if the collection is empty. | `src/bindings/collection.rs` |
| `pub fn is_poisoned(&self) -> bool` | Returns true if the collection/database engine was poisoned by a previous caught Rust panic. | `src/bindings/collection.rs` |
| `pub fn len(&self, py: Python<'_>) -> PyResult<usize>` | Returns the number of documents. | `src/bindings/collection.rs` |
| `pub fn new( inner: Arc<ContextraCollection>, runtime: Arc<Runtime>, poisoned: Arc<AtomicBool>, ) -> Self` | — | `src/bindings/collection.rs` |
| `pub fn stats(&self, py: Python<'_>) -> PyResult<PyVectorIndexStats>` | Returns statistics for the collection's vector index. | `src/bindings/collection.rs` |
| `pub fn check_subinterpreter_guard(py: Python<'_>) -> PyResult<()>` | Explicitly checks whether the module is being imported inside a CPython sub-interpreter. If running inside a sub-interpreter (interpreter I… | `src/bindings/common.rs` |
| `pub fn contextra_err(e: contextra_core::ContextraError) -> PyErr` | Maps a ContextraError into a structured Python PyErr with `kind`, `message`, and `details` attributes. | `src/bindings/common.rs` |
| `pub fn dict_to_json(d: &pyo3::Bound<'_, pyo3::types::PyDict>) -> PyResult<serde_json::Value>` | Converts a Python dict to a serde_json::Value. | `src/bindings/common.rs` |
| `pub fn doc_to_py(py: Python<'_>, d: contextra_db::Document) -> PyResult<PyDocument>` | Converts a contextra_db::Document to a PyDocument. | `src/bindings/common.rs` |
| `pub fn json_to_py(py: Python<'_>, val: &serde_json::Value) -> PyResult<PyObject>` | Converts a serde_json::Value to a Python object. | `src/bindings/common.rs` |
| `pub fn opt_dict_to_json( metadata: Option<&pyo3::Bound<'_, pyo3::types::PyDict>>, ) -> PyResult<Option<serde_json::Value>>` | Converts an optional Python dict to an optional serde_json::Value. | `src/bindings/common.rs` |
| `pub fn results_to_py( py: Python<'_>, results: Vec<contextra_db::SearchResult>, ) -> PyResult<Vec<PySearchResult>>` | Converts a Vec of SearchResult to Vec of PySearchResult. | `src/bindings/common.rs` |
| `pub fn run_blocking_ffi<F, R>(py: Python<'_>, poisoned: &AtomicBool, f: F) -> PyResult<R> where F: FnOnce() -> PyResult<R> + Send, R: Send,` | Safely executes a blocking closure across FFI boundaries with thread state release and panic containment to guarantee no Rust panic propaga… | `src/bindings/common.rs` |
| `pub fn validate_batch_size(size: usize) -> PyResult<()>` | Validates batch size against maximum resource allocation limits. | `src/bindings/common.rs` |
| `pub fn validate_collection_name(name: &str) -> PyResult<()>` | Validates that a collection name is non-empty, contains no null bytes, and does not exceed maximum length. | `src/bindings/common.rs` |
| `pub fn validate_db_path(path: &str) -> PyResult<()>` | Validates that a database storage path is non-empty and contains no null bytes. | `src/bindings/common.rs` |
| `pub fn validate_id(id: &str) -> PyResult<()>` | Validates that a string ID is non-empty, contains no null bytes, and does not exceed maximum length. | `src/bindings/common.rs` |
| `pub fn validate_id_obj(id_obj: &pyo3::Bound<'_, pyo3::types::PyAny>) -> PyResult<String>` | Validates a document ID provided as a string or numeric value. | `src/bindings/common.rs` |
| `pub fn validate_label(label: &str) -> PyResult<()>` | Validates that a relationship label is non-empty, contains no null bytes, and does not exceed maximum length. | `src/bindings/common.rs` |
| `pub fn validate_query_text(text: &str) -> PyResult<()>` | Validates that a search query text is non-empty, contains no null bytes, and does not exceed maximum length. | `src/bindings/common.rs` |
| `pub fn validate_vector(vector: &[f32]) -> PyResult<()>` | Validates that a vector slice is non-empty and contains no NaN or infinite values. | `src/bindings/common.rs` |
| `pub fn delete<'py>( &self, py: Python<'py>, id: &pyo3::Bound<'py, pyo3::types::PyAny>, ) -> PyResult<()>` | Deletes a document by its ID. | `src/bindings/crud_macros.rs` |
| `pub fn get<'py>( &self, py: Python<'py>, id: &pyo3::Bound<'py, pyo3::types::PyAny>, ) -> PyResult<Option<PyDocument>>` | Retrieves a document by its user-provided string or numeric ID. | `src/bindings/crud_macros.rs` |
| `pub fn hybrid_search<'py>( &self, py: Python<'py>, text: &str, vector: PyReadonlyArray1<'py, f32>, k: usize, vector_weight: Option<f32>, text_weight: Option<f32>, graph_weight: Option<f32>, ) -> PyResult<Vec<PySearchResult>>` | Performs hybrid search combining BM25, vector search, and graph traversal results. | `src/bindings/crud_macros.rs` |
| `pub fn hybrid_search_fb<'py>( &self, py: Python<'py>, text: &str, vector: PyReadonlyArray1<'py, f32>, k: usize, vector_weight: Option<f32>, text_weight: Option<f32>, graph_weight: Option<f32>, ) -> PyResult<Bound<'py, PyBytes>>` | Performs hybrid search and returns results as FlatBuffer-encoded bytes (PyBytes). Returns FlatBuffer binary IPC payload copied into Python… | `src/bindings/crud_macros.rs` |
| `pub fn insert<'py>( &self, py: Python<'py>, id: &pyo3::Bound<'py, pyo3::types::PyAny>, vector: PyReadonlyArray1<'py, f32>, metadata: Option<pyo3::Bound<'py, pyo3::types::PyDict>>, ) -> PyResult<()>` | Inserts a document with an embedding and optional metadata. | `src/bindings/crud_macros.rs` |
| `pub fn insert_many<'py>( &self, py: Python<'py>, docs: Vec<( pyo3::Bound<'py, pyo3::types::PyAny>, PyReadonlyArray1<'py, f32>, Option<pyo3::Bound<'py, pyo3::types::PyDict>>, )>, ) -> PyResult<()>` | Inserts multiple documents in a single transaction. Each doc is a tuple of `(id: str, vector: np.ndarray, metadata: dict | None)`. | `src/bindings/crud_macros.rs` |
| `pub fn relate<'py>( &self, py: Python<'py>, from: &pyo3::Bound<'py, pyo3::types::PyAny>, to: &pyo3::Bound<'py, pyo3::types::PyAny>, label: &str, ) -> PyResult<()>` | Creates a bidirectional relationship between two documents. | `src/bindings/crud_macros.rs` |
| `pub fn scan( &self, py: Python<'_>, start: Option<&str>, end: Option<&str>, limit: Option<usize>, ) -> PyResult<Vec<(String, PyObject)>>` | Performs a range scan of documents. Accepts optional string keys for start and end bounds (inclusive). Pass `None` for unbounded. | `src/bindings/crud_macros.rs` |
| `pub fn scan_prefix( &self, py: Python<'_>, prefix: &str, limit: Option<usize>, ) -> PyResult<Vec<(String, PyObject)>>` | Scans documents matching a given key prefix. | `src/bindings/crud_macros.rs` |
| `pub fn search<'py>( &self, py: Python<'py>, vector: PyReadonlyArray1<'py, f32>, k: usize, ) -> PyResult<Vec<PySearchResult>>` | Performs semantic k-NN search over the embeddings. | `src/bindings/crud_macros.rs` |
| `pub fn search_fb<'py>( &self, py: Python<'py>, vector: PyReadonlyArray1<'py, f32>, k: usize, ) -> PyResult<Bound<'py, PyBytes>>` | Performs semantic search and returns results as FlatBuffer-encoded bytes (PyBytes). Returns FlatBuffer binary IPC payload copied into Pytho… | `src/bindings/crud_macros.rs` |
| `pub fn update<'py>( &self, py: Python<'py>, id: &pyo3::Bound<'py, pyo3::types::PyAny>, vector: PyReadonlyArray1<'py, f32>, metadata: Option<pyo3::Bound<'py, pyo3::types::PyDict>>, ) -> PyResult<()>` | Updates an existing document. | `src/bindings/crud_macros.rs` |
| `pub fn upsert<'py>( &self, py: Python<'py>, id: &pyo3::Bound<'py, pyo3::types::PyAny>, vector: PyReadonlyArray1<'py, f32>, metadata: Option<pyo3::Bound<'py, pyo3::types::PyDict>>, ) -> PyResult<()>` | Upserts a document (inserts if missing, updates if exists). | `src/bindings/crud_macros.rs` |
| `pub fn upsert_many<'py>( &self, py: Python<'py>, docs: Vec<( pyo3::Bound<'py, pyo3::types::PyAny>, PyReadonlyArray1<'py, f32>, Option<pyo3::Bound<'py, pyo3::types::PyDict>>, )>, ) -> PyResult<()>` | Upserts multiple documents in a single transaction. Each doc is a tuple of `(id: str, vector: np.ndarray, metadata: dict | None)`. | `src/bindings/crud_macros.rs` |
| `pub fn _trigger_panic_for_test(&self, py: Python<'_>, message: Option<String>) -> PyResult<()>` | Internal helper method for testing FFI panic isolation and engine poisoning. # Panics bewusst; nur Testhook für FFI-Panic-Isolation; Panic… | `src/bindings/db.rs` |
| `pub fn collection(&self, name: &str, py: Python<'_>) -> PyResult<PyCollection>` | Returns a specific collection (namespace). Creates the collection if it does not already exist. | `src/bindings/db.rs` |
| `pub fn drop_collection( &self, name: &str, proof_key: Option<&Bound<'_, PyBytes>>, py: Python<'_>, ) -> PyResult<String>` | Drops a collection, removing all its data from storage and returning a collection-scoped DeletionProof as JSON. # Key Resolution Drops requ… | `src/bindings/db.rs` |
| `pub fn flush(&self, py: Python<'_>) -> PyResult<()>` | Flushes all pending writes to disk. | `src/bindings/db.rs` |
| `pub fn is_empty(&self, py: Python<'_>) -> PyResult<bool>` | Returns true if the collection/database is empty. | `src/bindings/db.rs` |
| `pub fn is_poisoned(&self) -> bool` | Returns true if the database engine was poisoned by a previous caught Rust panic. | `src/bindings/db.rs` |
| `pub fn len(&self, py: Python<'_>) -> PyResult<usize>` | Returns the number of documents. | `src/bindings/db.rs` |
| `pub fn list_collections(&self, py: Python<'_>) -> PyResult<Vec<String>>` | Lists all existing collection names. | `src/bindings/db.rs` |
| `pub fn new( inner: Arc<Contextra>, runtime: Arc<Runtime>, worker_threads: usize, poisoned: Arc<AtomicBool>, ) -> Self` | — | `src/bindings/db.rs` |
| `pub fn stats(&self, py: Python<'_>) -> PyResult<PyDbStats>` | Returns combined statistics for the vector index and storage engine. | `src/bindings/db.rs` |
| `pub fn worker_threads(&self) -> usize` | Returns the number of worker threads configured in this database's Tokio runtime. | `src/bindings/db.rs` |
| `pub fn _trigger_panic_for_test(py: Python<'_>, message: Option<String>) -> PyResult<()>` | Internal Python binding function for testing FFI panic isolation. # Intention Exposed to Python test suite (`tests/test_panic_isolation.py`… | `src/bindings/functions.rs` |
| `pub fn open( py: Python<'_>, path: &str, dimension: usize, max_elements: Option<usize>, encryption_passphrase: Option<String>, distance_metric: Option<String>, ) -> PyResult<PyContextra>` | Opens or creates a Contextra database at the given path. Default `dimension` is 768 to align with `ContextraConfig::default().dimension` in… | `src/bindings/functions.rs` |
| `pub fn relate_n_ary<'py>( &self, py: Python<'py>, predicate: &str, participants: Vec<(Bound<'py, PyAny>, String)>, source_doc_id: Option<Bound<'py, PyAny>>, ) -> PyResult<u64>` | Legt eine n-äre Hyperkante an. `participants` = [(doc_id, role), ...], mindestens 2. | `src/bindings/hyperedge.rs` |
| `pub fn validate_hyperedge_args( predicate: &str, participants: &[(&str, &str)], source_doc_id: Option<&str>, ) -> PyResult<()>` | Validates arguments for creating an n-ary hyperedge. | `src/bindings/hyperedge.rs` |
| `pub fn get_runtime(py: Python<'_>) -> PyResult<Arc<Runtime>>` | Retrieves or initializes the per-interpreter Tokio runtime attached to the `_contextra` module state. Reads `CONTEXTRA_WORKER_THREADS` on i… | `src/bindings/runtime_state.rs` |
| `pub fn parse_worker_threads_env() -> usize` | Parses and clamps `CONTEXTRA_WORKER_THREADS` environment variable to `[MIN_WORKER_THREADS, MAX_WORKER_THREADS]`. If `CONTEXTRA_WORKER_THREA… | `src/bindings/runtime_state.rs` |
| `pub fn get_kv<'py>( &self, py: Python<'py>, id: &Bound<'py, PyAny>, ) -> PyResult<Option<PyObject>>` | Retrieves a non-vector key-value entry directly from storage. | `src/kv_links.rs` |
| `pub fn get_links<'py>( &self, py: Python<'py>, id: &Bound<'py, PyAny>, ) -> PyResult<Vec<PyObject>>` | Retrieves all memory links for a given document. | `src/kv_links.rs` |
| `pub fn link_memories<'py>( &self, py: Python<'py>, from: &Bound<'py, PyAny>, to: &Bound<'py, PyAny>, relation: &str, ) -> PyResult<()>` | Links two memories together with a specific relation (Zettelkasten A-MEM). | `src/kv_links.rs` |
| `pub fn parse_link_relation(s: &str) -> PyResult<LinkRelation>` | Parses a string into a `LinkRelation` (case-insensitive, allocation-free). | `src/kv_links.rs` |
| `pub fn put_kv<'py>( &self, py: Python<'py>, id: &Bound<'py, PyAny>, value: &Bound<'py, PyDict>, ) -> PyResult<()>` | Stores a non-vector key-value entry directly in storage without modifying indices. | `src/kv_links.rs` |
| `pub fn put_kv_if_absent<'py>( &self, py: Python<'py>, id: &Bound<'py, PyAny>, value: &Bound<'py, PyDict>, ) -> PyResult<()>` | Stores a non-vector key-value entry directly in storage only if the key does not already exist. | `src/kv_links.rs` |
| `pub fn relate_bidirectional<'py>( &self, py: Python<'py>, from: &Bound<'py, PyAny>, to: &Bound<'py, PyAny>, label: &str, ) -> PyResult<()>` | Creates a bidirectional relationship between two documents atomically. | `src/kv_links.rs` |


---

## `contextra-testkit`

| Attribut | Wert |
|---|---|
| **Ring** | 99 — Tooling (Cross-Ring) |
| **Status** | Fertig (Tooling) |
| **Zweck** | Deterministische Test-Utilities (ManualClock, InMemoryStorageEngine, FaultVfs). |
| **Unsafe-Code** | Nein (`#![forbid(unsafe_code)]`) |
| **Interne Abhängigkeiten** | `contextra-ports`, `contextra-types` |
| **Externe Abhängigkeiten (Auswahl)** | `bytes`, `parking_lot`, `tracing` |

### Modul- und Re-Export-Oberfläche (lib.rs)

```rust
pub mod fault_vfs;
pub mod in_memory_store;
pub mod manual_clock;
pub mod reference_model;
pub use fault_vfs::{FaultConfig, FaultVfs};
pub use in_memory_store::InMemoryStorageEngine;
pub use manual_clock::ManualClock;
pub use reference_model::{RefOp, ReferenceModel};
```

### Öffentliche Datentypen — Structs (5)

| Struct | Felder / Form | Beschreibung | Datei |
|---|---|---|---|
| `FaultConfig` | `{ /// Fail write operations after this many successful writes. pub fail_writes_after: Option<usize>, /// Fail sync / flush operations after this many successful… }` | Configuration for fault injection. | `src/fault_vfs.rs` |
| `FaultVfs` | `{ files: Arc<RwLock<HashMap<PathBuf, Vec<u8>>>>, read_count: Arc<AtomicUsize>, write_count: Arc<AtomicUsize>, sync_count: Arc<AtomicUsize>, config: Arc<RwLock<Fa… }` | Simulated in-memory file system with deterministic fault injection capabilities. | `src/fault_vfs.rs` |
| `InMemoryStorageEngine` | `{ data: Arc<RwLock<StoreDataMap>>, staged: Arc<RwLock<StagedTxMap>>, seq: Arc<AtomicU64>, last_tx: Arc<RwLock<TxId>>, }` | An in-memory, thread-safe implementation of `StorageEngine` for tests. | `src/in_memory_store.rs` |
| `ManualClock` | `{ nanos: AtomicU64, }` | A thread-safe, manually advanceable clock for deterministic tests. | `src/manual_clock.rs` |
| `ReferenceModel` | `{ #[allow(clippy::type_complexity)] history: BTreeMap<Vec<u8>, Vec<(u64, Option<Vec<u8>>)>>, pending_writes: Vec<RefOp>, current_seq: u64, }` | Deterministic, std-only MVCC reference model for Contextra store verification. | `src/reference_model.rs` |

### Öffentliche Datentypen — Enums (1)

| Enum | Varianten | Beschreibung | Datei |
|---|---|---|---|
| `RefOp` | `/// Inserts or overwrites a key-value pair. Put { key: Vec<u8>, value: Vec<u8> }, /// Marks a key as deleted (tombstone). Delete { key: Vec<u8> },` | Operation for batch execution on the reference model. | `src/reference_model.rs` |

### Öffentliche Funktionen (30)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn new() -> Self` | Creates a new, clean `FaultVfs`. | `src/fault_vfs.rs` |
| `pub fn read_count(&self) -> usize` | Returns the number of read operations attempted. | `src/fault_vfs.rs` |
| `pub fn read_file(&self, path: &Path) -> io::Result<Vec<u8>>` | Reads data from a path, respecting fault policies. | `src/fault_vfs.rs` |
| `pub fn reset_counters(&self)` | Resets crash state and counters. | `src/fault_vfs.rs` |
| `pub fn set_config(&self, config: FaultConfig)` | Sets the active fault configuration and resets operation counters. | `src/fault_vfs.rs` |
| `pub fn sync(&self) -> io::Result<()>` | Syncs / flushes a path or the entire VFS, respecting fault policies. | `src/fault_vfs.rs` |
| `pub fn sync_count(&self) -> usize` | Returns the number of sync operations attempted. | `src/fault_vfs.rs` |
| `pub fn trigger_crash(&self)` | Simulates a system crash (disables further operations until reset). | `src/fault_vfs.rs` |
| `pub fn write_count(&self) -> usize` | Returns the number of write operations attempted. | `src/fault_vfs.rs` |
| `pub fn write_file(&self, path: &Path, data: &[u8]) -> io::Result<()>` | Writes data to a path, respecting fault policies. | `src/fault_vfs.rs` |
| `pub fn new() -> Self` | Creates a new, empty in-memory storage engine. | `src/in_memory_store.rs` |
| `pub fn advance(&self, duration: Duration)` | Advances the clock by the given duration. | `src/manual_clock.rs` |
| `pub fn new(initial_nanos: u64) -> Self` | Creates a new `ManualClock` initialized to `initial_nanos` since UNIX_EPOCH. | `src/manual_clock.rs` |
| `pub fn now_nanos(&self) -> u64` | Returns the current timestamp in nanoseconds since UNIX_EPOCH. | `src/manual_clock.rs` |
| `pub fn now_secs_f64(&self) -> f64` | Returns the current timestamp as seconds (floating point). | `src/manual_clock.rs` |
| `pub fn now_system_time(&self) -> SystemTime` | Returns the current timestamp as a `SystemTime`. | `src/manual_clock.rs` |
| `pub fn set_nanos(&self, nanos: u64)` | Sets the clock to an absolute timestamp in nanoseconds since UNIX_EPOCH. | `src/manual_clock.rs` |
| `pub fn commit(&mut self) -> u64` | Commits all pending operations under a new sequence number. Returns the sequence number assigned to this commit batch. | `src/reference_model.rs` |
| `pub fn commit_batch(&mut self, ops: Vec<RefOp>) -> u64` | Appends `ops` to pending buffer and commits them under a new sequence number. | `src/reference_model.rs` |
| `pub fn delete(&mut self, key: &[u8])` | Stages a `Delete` operation in uncommitted pending buffer. | `src/reference_model.rs` |
| `pub fn get_at(&self, key: &[u8], seq: u64) -> Option<Vec<u8>>` | Retrieves the value of `key` as visible at `seq`. Returns `Some(value)` if key exists and was active (not deleted) at `seq`, `None` otherwi… | `src/reference_model.rs` |
| `pub fn get_latest(&self, key: &[u8]) -> Option<Vec<u8>>` | Gets the latest committed value for `key`. | `src/reference_model.rs` |
| `pub fn new() -> Self` | Creates a new empty reference model initialized at sequence number 0. | `src/reference_model.rs` |
| `pub fn put(&mut self, key: &[u8], value: &[u8])` | Stages a `Put` operation in uncommitted pending buffer. | `src/reference_model.rs` |
| `pub fn rollback(&mut self)` | Clears any uncommitted pending writes without advancing sequence number. | `src/reference_model.rs` |
| `pub fn scan_prefix_at(&self, prefix: &[u8], seq: u64) -> Vec<(Vec<u8>, Vec<u8>)>` | Scans keys starting with `prefix` and returns key-value pairs visible at `seq`. | `src/reference_model.rs` |
| `pub fn scan_prefix_latest(&self, prefix: &[u8]) -> Vec<(Vec<u8>, Vec<u8>)>` | Scans all entries starting with `prefix` at latest sequence number. | `src/reference_model.rs` |
| `pub fn scan_range_at( &self, start: Bound<&[u8]>, end: Bound<&[u8]>, seq: u64, ) -> Vec<(Vec<u8>, Vec<u8>)>` | Scans key range `[start, end]` (using `Bound`) and returns visible pairs at `seq`. | `src/reference_model.rs` |
| `pub fn snapshot_map_at(&self, seq: u64) -> BTreeMap<Vec<u8>, Vec<u8>>` | Returns a full snapshot map of latest visible key-values at `seq`. | `src/reference_model.rs` |
| `pub fn snapshot_seq(&self) -> u64` | Returns the current high water mark sequence number. | `src/reference_model.rs` |


---

## Anhang A — Querschnittliche Invarianten (gelten crateübergreifend)

* **Zero-Panic-Doktrin:** `unwrap_used`, `expect_used`, `panic`, `todo`, `unimplemented`, `unreachable` sind workspace-weit als Clippy-Lint verboten.
* **`unsafe_code = "deny"`** ist Workspace-Default; nur `contextra-sys`, `contextra-simd`, `contextra-wire` heben dies explizit auf.
* **Sync-Kern, Async-Schale (P26):** Ring-0-Crates verwenden kein `tokio`.
* **DAG-Integrität (P5):** Aufwärtskanten sind verboten; einzige dokumentierte Restausnahme: `contextra-router → contextra-db` (Migrationsartefakt).
* **`#[non_exhaustive]`-Disziplin** bei zentralen Fehler-Enums (u. a. `ContextraError`).
* **Deletion-Proof-Kette:** kryptographisch extern verifizierbare Löschbeweise (`contextra-crypto`).
* **Feature-Ring-Trennung:** `fast` / `sovereign` / `compliance` (Crate `contextra`) trennen Lizenzstufen auf Kompilierungsebene.

## Anhang B — Nicht tiefenanalysierte Hilfs-Targets

| Target | Pfad | Rolle |
|---|---|---|
| `contextra-bench` | `benchmarks/contextra-bench` | Criterion-Benchmark-Harness, kein Teil der Produktions-API. |
| `xtask` | `xtask/` | Cargo-`xtask`-Automatisierungsbinary, von `[workspace] exclude` ausgenommen. |

## Anhang C — Referenzierte Primärquellen im Repository

* `README.md`, `ARCHITECTURE.md`, `CONSTITUTION.md`, `VETOES.md`, `AGENTS.md`
* `capabilities.toml`, `deny.toml`, `supply-chain/`
* `docs/spec/CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md`
* `docs/refactor/router-db-edge-audit.md`, `docs/refactor/mcp-upward-edges-audit.md`

---

*Ende der globalen Schnittstellenspezifikation — automatisch generiert. Für Implementierungsdetails unterhalb der öffentlichen API-Grenze (private Module, `pub(crate)`-Symbole) den Quellcode direkt konsultieren.*
