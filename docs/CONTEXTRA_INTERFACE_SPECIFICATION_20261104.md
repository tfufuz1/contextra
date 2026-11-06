# Contextra — Globale Mikrofeingranulare Schnittstellenspezifikation

**Repository:** `contextra` (Workspace-Root) · **Generiert am:** 2026-11-04
**Erzeugt durch:** `scripts/generate_interface_spec.py` (automatisierte, reproduzierbare Vollanalyse aller Workspace-Crates)
**Methodik:** Statische Extraktion aller `pub`-Symbole (Traits, Structs, Enums, Funktionen, Typ-Aliase, Konstanten) aus jedem `src/**/*.rs` je Crate, angereichert mit den Original-Doc-Kommentaren (`///`, `//!`), den `Cargo.toml`-Metadaten (Abhängigkeiten, Feature-Flags) und der `lib.rs`-Re-Export-Oberfläche. Test-Module (`#[cfg(test)] mod tests`) sind ausgeschlossen.

> **Hinweis zur Reproduzierbarkeit:** Dieses Dokument wird bei jedem Lauf von `scripts/generate_interface_spec.py` vollständig neu aus dem aktuellen Quellcode-Stand erzeugt. Es spiegelt exakt den Commit wider, gegen den es generiert wurde — bei Architekturänderungen (neue Crates, geänderte Traits/Ports) einfach das Skript erneut ausführen.

## Umfang der Analyse

| Kennzahl | Wert |
|---|---|
| Analysierte Fach-Crates (`crates/*`) | 33 |
| Extrahierte öffentliche Symbole gesamt | **4024** |
| — davon Public Traits (Schnittstellen im engeren Sinn) | 91 |
| — davon Public Structs | 662 |
| — davon Public Enums | 147 |
| — davon Public Funktionen | 2868 |
| — davon Typ-Aliase | 35 |
| — davon Konstanten | 221 |

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
pub mod witness;
pub use error::{ContextraError, HnswDeletionError, Result};
pub use error_dto::ContextraErrorDto;
pub use model_fingerprint::ModelFingerprint;
pub use retrieval_strategy::RetrievalStrategy;
pub use schema::{DocIdWidth, ManifestSchemaVersion};
pub use tenant_scope::{TenantScopeViolation, TenantScoped};
pub use tombstone::{SeqBitTombstone, TombstoneSemanticsCheck};
pub use types::*;
pub use witness::{RequestContext, SagaRecord, ScanMethod};
```

### Feature-Flags

| Flag | Aktiviert |
|---|---|
| `default` | `[]` |
| `docid-128` | `[]` |
| `auto-extraction-opt-out` | `[]` |

### Schnittstellen — Public Traits (1)

**`TombstoneSemanticsCheck`** — ** (`src/tombstone.rs`)

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
| `CollectionId` | `(tuple/unit)` | — | `src/types/domain/ids.rs` |
| `ConfigFingerprint` | `{ /// LLM-Modell-ID (z.B. "llama-3.2-3b-instruct") pub model_id: String, /// Quantisierungsgrad als String: "Q4_K_M", "Q8_0", "F16", "BF16". /// EXPLIZIT Teil de… }` | — | `src/types/domain/misc.rs` |
| `ContextChunk` | `{ /// Document identifier associated with this chunk. pub doc_id: DocId, /// Text content of the chunk. pub content: String, /// Computed relevance score. pub re… }` | — | `src/types/saos.rs` |
| `ContextWindow` | `{ /// List of included context chunks. pub chunks: Vec<ContextChunk>, /// Total aggregated token count. pub total_tokens: usize, /// Indicates whether context ch… }` | — | `src/types/saos.rs` |
| `ContextraErrorDto` | `{ /// Stable string identifier for the error variant (e.g. `"NotFound"`, `"PolicyViolation"`). pub kind: String, /// Human-readable error message. pub message: S… }` | — | `src/error_dto.rs` |
| `DocId` | `(tuple/unit)` | — | `src/types/domain/ids.rs` |
| `DocId` | `(tuple/unit)` | — | `src/types/domain/ids.rs` |
| `Edge` | `{ /// Source entity identifier. pub from: EntityId, /// Destination entity identifier. pub to: EntityId, /// Relationship label. pub label: Arc<str>, /// Numeric… }` | — | `src/types/domain/misc.rs` |
| `Embedding` | `{ /// Raw vector data elements. pub data: Vec<f32>, }` | — | `src/types/domain/document.rs` |
| `Entity` | `{ /// Unique entity ID. pub id: EntityId, /// Canonical human-readable name. pub name: Arc<str>, /// Categorical entity type. pub entity_type: Arc<str>, /// Flex… }` | — | `src/types/domain/misc.rs` |
| `EntityId` | `(tuple/unit)` | — | `src/types/domain/ids.rs` |
| `FusionWeights` | `{ vector: f32, text: f32, graph: f32, /// Reserved for future metadata-signal fusion; currently always 0.0. Do not expose a public constructor parameter for this… }` | — | `src/types/saos.rs` |
| `HybridQuery` | `{ /// Optional full-text query string. pub text_query: Option<String>, /// Optional vector query embedding. pub vector_query: Option<Vec<f32>>, /// Optional star… }` | — | `src/types/saos.rs` |
| `HybridQueryBuilder` | `{ text_query: Option<String>, vector_query: Option<Vec<f32>>, graph_start_node: Option<String>, graph_strategy: Option<GraphTraversalStrategy>, fusion_weights: O… }` | — | `src/types/saos.rs` |
| `ImportanceScore` | `(tuple/unit)` | — | `src/types/importance.rs` |
| `MemoryImportance` | `{ /// Base LLM or heuristic evaluation score. pub base_score: ImportanceScore, /// Recency decay function strategy. pub decay: DecayFunction, /// Transaction ID… }` | — | `src/types/importance.rs` |
| `MemoryLink` | `{ /// The target document ID. pub target: DocId, /// The type of relationship. pub relation: LinkRelation, /// The transaction ID when this link was created. pub… }` | — | `src/types/domain/misc.rs` |
| `ModelFingerprint` | `{ /// SHA-256 hash digest over model weight blob and quantization tier string. pub hash: [u8; 32], /// Model identifier or filename (e.g. "llama-3.2-3b-instruct"… }` | Identifiziert eine Modelldatei und deren Quantisierungsstufe eindeutig. | `src/model_fingerprint.rs` |
| `PprConfig` | `{ /// Damping factor (probability of continuing random walk vs restarting). Default: 0.85. pub damping_factor: f32, /// Maximum power-iteration steps before term… }` | — | `src/types/domain/misc.rs` |
| `QueryRewriteOutput` | `{ /// Textual reformulation intended for BM25 keyword search. pub text_query: Option<String>, /// Semantic reformulation intended for dense vector embedding sear… }` | — | `src/types/saos.rs` |
| `RequestContext` | `{ /// Tenant identifier for authorization and scope isolation. pub tenant_id: crate::TenantId, /// Transaction ID allocated for the operation. pub tx_id: crate::… }` | — | `src/witness.rs` |
| `RerankResult` | `{ /// Rank position after reranking (0-based). pub index: usize, /// Original index in the input candidate array. pub original_index: usize, /// Raw model output… }` | — | `src/types/domain/document.rs` |
| `Reservation` | `{ budget: &'a TokenBudget, amount: usize, settled: AtomicBool, }` | — | `src/types/budget.rs` |
| `ResourceBudget` | `{ /// Maximum allowed memory usage in bytes. pub memory_limit: u64, }` | — | `src/types/budget.rs` |
| `ResourceTracker` | `{ /// The configured budget. budget: ResourceBudget, /// Current memory usage in bytes. memory_used: std::sync::atomic::AtomicU64, }` | — | `src/types/budget.rs` |
| `SagaRecord` | `{ /// BLAKE3 hash of deleted keys. pub deleted_keys_hash: [u8; 32], }` | — | `src/witness.rs` |
| `ScoredDocument` | `{ /// Identifier of the scored document. pub doc_id: DocId, /// Relevance or similarity score. pub score: f32, }` | — | `src/types/domain/document.rs` |
| `ScoredEntry` | `{ /// Unique result identifier. pub id: String, /// Combined final score after signal fusion. pub final_score: f32, /// Associated entry metadata. pub metadata:… }` | — | `src/types/saos.rs` |
| `SeqBitTombstone` | `(tuple/unit)` | — | `src/tombstone.rs` |
| `SignalFusionStrategies` | `{ /// Strategy used for vector search signal fusion. pub vector: FusionStrategy, /// Strategy used for full-text search signal fusion. pub text: FusionStrategy,… }` | — | `src/types/saos.rs` |
| `StepId` | `(tuple/unit)` | — | `src/error.rs` |
| `TenantId` | `(tuple/unit)` | — | `src/types/domain/ids.rs` |
| `TenantScoped` | `{ tenant_id: TenantId, value: T, }` | — | `src/tenant_scope.rs` |
| `TlHfdParams` | `{ /// Regularization parameter $\sigma > 0$ controlling diffusion scale (default: 0.1). #[serde(default = "default_tl_hfd_sigma")] pub sigma: f32, /// Seed injec… }` | — | `src/types/domain/misc.rs` |
| `TokenBudget` | `{ /// Maximum total token limit. pub limit: usize, /// Strategy for calculating usable context window. pub strategy: BudgetStrategy, /// Reserved tokens for syst… }` | — | `src/types/budget.rs` |
| `TxId` | `(tuple/unit)` | — | `src/types/domain/ids.rs` |
| `WorkflowState` | `{ /// Associated transaction. pub tx: TxId, /// Agent memory graph state footprint (BLAKE3 hash digest). #[serde(with = "hex")] pub graph_hash: [u8; 32], }` | — | `src/types/domain/misc.rs` |

### Öffentliche Datentypen — Enums (20)

| Enum | Varianten | Beschreibung | Datei |
|---|---|---|---|
| `AutoExtractionMode` | `/// Automatic entity extraction is enabled. Enabled, /// Automatic entity extraction is disabled. Disabled,` | — | `src/types/domain/misc.rs` |
| `BudgetStrategy` | `/// Konservativ: 80% des Limits als sicherer Puffer (Standard) Conservative, /// Aggressiv: 95% des Limits nutzen Aggressive, /// Exakt: Angabe des verfügbaren Fensters direkt Exact(usize),` | — | `src/types/budget.rs` |
| `CacheDirective` | `/// Segment is explicitly pinned in memory until TTL expiration or explicit release. Pin { /// Optional maximum duration for which the segment remains pinned. `None` indicates indefinite pinning. ttl…` | — | `src/error.rs` |
| `ContextraError` | `// ═══ Core & Logic ═══ /// Internal engine logic error. #[error("Internal error: {0}")] Internal(String), /// Invalid user or API argument input. #[error("Invalid input: {0}")] InvalidInput(String),…` | — | `src/error.rs` |
| `DecayFunction` | `/// No recency decay (factor remains 1.0). #[default] None, /// Exponential decay measured in `TxId` distance (not wall-clock time). Exponential { /// Half-life in transaction count. half_life_tx: u6…` | — | `src/types/importance.rs` |
| `DistanceMetric` | `/// Cosine distance (`1.0 - cos(angle)`). #[default] Cosine, /// Euclidean (L2) distance. Euclidean, /// Dot product distance (negated inner product for f32). DotProduct,` | — | `src/types/domain/document.rs` |
| `DocIdWidth` | `/// 64-bit DocId format (8-byte BLAKE3 truncation). Bit64 = 64, /// 128-bit DocId format (16-byte BLAKE3 truncation, ADR-082). Bit128 = 128,` | Gibt die von einer Schemaversion erwartete oder erzeugte Bitbreite einer `DocId` an. | `src/schema.rs` |
| `FilterExpr` | `/// Exact match: field == value Eq { /// Target metadata field name. field: String, /// Expected field value. value: serde_json::Value, }, /// Not equal: field != value Ne { /// Target metadata field…` | — | `src/types/filter.rs` |
| `FusionStrategy` | `/// Reciprocal Rank Fusion (default strategy per ADR-003). /// Robust against score scale mismatch and partial signal degradation. #[default] Rrf, /// Score-normalized fusion (CombSUM / Z-Score). ///…` | — | `src/types/saos.rs` |
| `GraphTraversalStrategy` | `/// Multi-hop BFS traversal with hop score decay (default max_hops = 3). Hops { /// Maximum traversal hop depth. max_hops: usize, }, /// Personalized PageRank power iteration starting from seed nodes…` | — | `src/types/saos.rs` |
| `HnswDeletionError` | `/// Target node with doc_id not found in graph. #[error("Node for doc_id {0:?} not found in HNSW index")] NodeNotFound(DocId), /// Graph repair resulted in a disconnected graph component. #[error("Gr…` | — | `src/error.rs` |
| `LinkRelation` | `/// Target memory provides more detail or context. Elaborates, /// Target memory contradicts this memory. Contradicts, /// Target memory supersedes this memory, replacing its context. Supersedes, ///…` | — | `src/types/domain/misc.rs` |
| `ManifestSchemaVersion` | `/// Schema Version 1 (v0.x legacy 64-bit DocId layout). #[default] V1 = 1, /// Schema Version 2 (ADR-082 128-bit DocId layout). V2 = 2,` | Schema-Versionskennung für SSTable- und WAL-Manifeste. | `src/schema.rs` |
| `MemoryType` | `/// Episodisches Gedächtnis: Erlebnisse, Unterhaltungen, zeitlich verankerte Ereignisse. /// Retrieval: Zeitliche Nähe und Relevanz zur aktuellen Session. /// Decay: Hohe Recency-Decay-Rate (Informat…` | — | `src/types/domain/misc.rs` |
| `OnSignalFailure` | `/// Abort hybrid search and return an error on signal failure (Default per Spec Teil 6.5 P1). #[default] Fail, /// Degrade search gracefully using remaining active signals and report failed signals i…` | — | `src/types/saos.rs` |
| `PprAlgorithm` | `/// Auto heuristic dispatch based on seed count (ForwardPush for <= 100 seeds, DensePowerIteration otherwise). #[default] Auto, /// Dense power-iteration algorithm (matrix-vector multiplication over…` | — | `src/types/domain/misc.rs` |
| `RetrievalStrategy` | `/// Vektorbasiertes semantisches Signal. Vector, /// Textbasiertes sparse/BM25 Signal. Text, /// Graphbasiertes Beziehungs-Signal. Graph, /// Hybrides Verbund-Signal. Hybrid, /// Nutzt LeanRAG-aggreg…` | Semantische Retrieval-Strategie für Signal-Fusion (§8/§9). | `src/retrieval_strategy.rs` |
| `ScanMethod` | `/// Direct structural scan of storage files, indices, or tables. Structural, /// Layer was not measured during this execution with specified reason. NotMeasured(String), /// Layer measurement is disa…` | — | `src/witness.rs` |
| `Signal` | `/// Dense vector embedding search signal. Vector, /// Lexical full-text BM25 search signal. Text, /// Graph traversal / PageRank signal. Graph,` | — | `src/types/saos.rs` |
| `TenantScopeViolation` | `/// Raised when unpacking a `TenantScoped<T>` value with an expected `TenantId` that differs /// from the value's actual bound `TenantId`. #[error("tenant scope mismatch: expected {expected:?}, got {…` | — | `src/tenant_scope.rs` |

### Typ-Aliase (3)

| Alias | Definition | Beschreibung |
|---|---|---|
| `QueryOptions` | `type QueryOptions= HybridQuery` | Alias for [`HybridQuery`] representing query options for hybrid retrieval operations (Spec Teil 6.5). |
| `QueryOptionsBuilder` | `type QueryOptionsBuilder= HybridQueryBuilder` | Alias for [`HybridQueryBuilder`] representing a query options builder for hybrid retrieval operations (Spec Teil 6.5). |
| `Result` | `type Result<T> = std::result::Result<T, ContextraError>` | — |

### Konstanten (14)

| Konstante | Typ | Beschreibung |
|---|---|---|
| `DEFAULT` | `Self` | Der implizite Default-Mandant für alle bestehenden Single-Tenant-Deployments. |
| `EXPIRY_METADATA_KEY` | `&str` | — |
| `INTERNAL_BASE` | `u64` | Lower bound of the internal system transaction ID range. Exact numeric value: `u64::MAX - 1_000_000` (`18_446_744_073_708_551_615`). Dieser Grenzwert… |
| `INVALID` | `Self` | Invalid tenant identifier sentinel value (`0`). |
| `INVALID` | `Self` | Invalid collection identifier sentinel value (`0`). |
| `INVALID` | `Self` | Invalid or uninitialized transaction identifier sentinel value (`0`). `TxId(0)` is reserved as an uninitialized sentinel or null identifier across th… |
| `MAX` | `Self` | Maximum possible `DocId` value (`u64::MAX`). |
| `MAX` | `Self` | Maximum possible `DocId` value (`u128::MAX`). |
| `MAX_COLLECTION_SEQUENCE` | `u64` | Upper bound of the Collection-sequenced transaction ID range (`10^12`). Transaction allocation sequences managed by `Collection::allocate_tx()` opera… |
| `MAX_SEARCH_K` | `usize` | — |
| `MIN` | `Self` | Minimum possible `DocId` value (`0`). Note: `DocId(0)` is conventionally treated as a sentinel/null value by the MCP layer (`contextra-mcp/src/lib.rs… |
| `MIN` | `Self` | Minimum possible `DocId` value (`0`). |
| `SYSTEM` | `Self` | SYSTEM tenant identifier (`0`). |
| `TOMBSTONE_BIT` | `u64` | — |

### Öffentliche Funktionen (124)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn capability_unsupported( capability: impl Into<String>, reason: impl Into<String>, ) -> Self` | — | `src/error.rs` |
| `pub fn checksum_mismatch(path: impl Into<String>, block_id: u64) -> Self` | — | `src/error.rs` |
| `pub fn cross_device_link( source_path: impl Into<String>, target_path: impl Into<String>, ) -> Self` | — | `src/error.rs` |
| `pub fn durability_config(msg: impl Into<String>) -> Self` | — | `src/error.rs` |
| `pub fn invalid_input(msg: impl Into<String>) -> Self` | — | `src/error.rs` |
| `pub fn is_occ_conflict(&self) -> bool` | — | `src/error.rs` |
| `pub fn kv_quantization(msg: impl Into<String>) -> Self` | — | `src/error.rs` |
| `pub fn limit_exceeded(limit: usize, context: impl Into<String>) -> Self` | — | `src/error.rs` |
| `pub fn model_load(path: impl Into<String>, reason: impl Into<String>) -> Self` | — | `src/error.rs` |
| `pub fn orphaned_vector_reference( doc_id: impl Into<String>, index_id: impl Into<String>, ) -> Self` | — | `src/error.rs` |
| `pub fn pin_budget_exceeded(msg: impl Into<String>) -> Self` | — | `src/error.rs` |
| `pub fn snapshot_unsupported_for_signal(strategy: impl Into<String>) -> Self` | — | `src/error.rs` |
| `pub fn wal_corruption(offset: u64, reason: impl Into<String>) -> Self` | — | `src/error.rs` |
| `pub fn wal_truncation_detected(expected_hmac: [u8; 32], actual_hmac: [u8; 32]) -> Self` | — | `src/error.rs` |
| `pub fn new(kind: impl Into<String>, message: impl Into<String>) -> Self` | — | `src/error_dto.rs` |
| `pub fn with_details( kind: impl Into<String>, message: impl Into<String>, details: serde_json::Value, ) -> Self` | — | `src/error_dto.rs` |
| `pub fn new( hash: [u8; 32], model_id: impl Into<String>, quantization: impl Into<String>, ) -> Self` | Creates a new `ModelFingerprint`. # Ablauf Übernimmt Hash, Modellbezeichner und Quantisierungsstring ohne Nebenwirkungen. | `src/model_fingerprint.rs` |
| `pub fn as_u8(self) -> u8` | Returns the numeric version byte. | `src/schema.rs` |
| `pub fn bytes_len(self) -> usize` | Returns the size in bytes of a single `DocId` value under this width. | `src/schema.rs` |
| `pub fn current() -> Self` | Returns the active compile-time `DocIdWidth`. # Ablauf Prüft zur Compile-Zeit via `cfg!(feature = "docid-128")`, welche Breite gewählt ist. | `src/schema.rs` |
| `pub fn doc_id_width(self) -> DocIdWidth` | Returns the corresponding `DocIdWidth` for this schema version. | `src/schema.rs` |
| `pub fn from_u8(version_byte: u8) -> Result<Self>` | Parses a version from a numeric byte indicator. # Fehler Liefert `ContextraError::Serialization`, falls das übergebene Byte keiner bekannte… | `src/schema.rs` |
| `pub fn is_compatible_with_current_build(self) -> bool` | Returns `true` if this schema version is compatible with the compile-time DocId feature flags. | `src/schema.rs` |
| `pub fn into_inner_checked(self, expected: &TenantId) -> Result<T, TenantScopeViolation>` | Gibt den Wert nur zurück, wenn die aufrufende Seite die erwartete `TenantId` nennt — verhindert versehentliches Cross-Tenant-Leaking durch… | `src/tenant_scope.rs` |
| `pub fn map<U>(self, f: impl FnOnce(T) -> U) -> TenantScoped<U>` | Transforms the inner value `T` into `U` using `f` while preserving the bound `TenantId`. # Ablauf Wendet `f(self.value)` an und erzeugt ein… | `src/tenant_scope.rs` |
| `pub fn new(tenant_id: TenantId, value: T) -> Self` | Creates a new `TenantScoped` value bound to the specified `TenantId`. # Ablauf Kapselt den Wert `value` zusammen mit `tenant_id` ohne Allok… | `src/tenant_scope.rs` |
| `pub fn tenant_id(&self) -> &TenantId` | Returns a reference to the bound `TenantId`. | `src/tenant_scope.rs` |
| `pub fn available(&self) -> usize` | Returns tokens still available after subtracting reserved and consumed tokens. | `src/types/budget.rs` |
| `pub fn budget(&self) -> &ResourceBudget` | Returns a reference to the configured resource budget. | `src/types/budget.rs` |
| `pub fn consume(&self, tokens: usize)` | Records `tokens` as consumed, reducing future availability. | `src/types/budget.rs` |
| `pub fn consume_memory(&self, bytes: u64) -> Result<()>` | Attempts to consume `bytes` of memory from the tracked budget. # Errors Returns `Err(ContextraError::MemoryBudgetExceeded)` if consuming `b… | `src/types/budget.rs` |
| `pub fn consumed(&self) -> usize` | Returns total tokens consumed so far. | `src/types/budget.rs` |
| `pub fn effective_limit(&self) -> usize` | Calculates the effective token limit based on strategy. | `src/types/budget.rs` |
| `pub fn for_model(model: &str) -> Self` | Erstellt ein Budget für gängige Modelle. # Maintenance Note Model name patterns use string matching. Explicit model family patterns (e.g. `… | `src/types/budget.rs` |
| `pub fn has_memory_capacity(&self) -> bool` | Returns true if memory usage is below 95% of the limit. | `src/types/budget.rs` |
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
| `pub fn from_doc_id(doc_id: DocId) -> Self` | Creates an `EntityId` directly from a `DocId`. | `src/types/domain/ids.rs` |
| `pub fn from_key(key: &str) -> Result<Self>` | Derives a `DocId` from a string key using the first 8 bytes (64 bits) of its BLAKE3 hash. # Kollisionssicherheit & Entwurfsentscheidung (AD… | `src/types/domain/ids.rs` |
| `pub fn from_key(key: &str) -> Result<Self>` | Derives a 128-bit `DocId` from a string key using the first 16 bytes (128 bits) of its BLAKE3 hash. # ADR-082 128-Bit BLAKE3 Truncation BLA… | `src/types/domain/ids.rs` |
| `pub fn from_key(key: &str) -> Result<Self>` | Derives an `EntityId` from a string key using the first 8 bytes of its BLAKE3 hash. # Errors Returns `ContextraError::InvalidInput` if `key… | `src/types/domain/ids.rs` |
| `pub fn from_key(key: &str) -> Result<Self>` | Derives an `EntityId` from a string key using the first 8 bytes of its BLAKE3 hash. # Errors Returns `ContextraError::InvalidInput` if `key… | `src/types/domain/ids.rs` |
| `pub fn is_system(self) -> bool` | Returns `true` if this tenant ID is `SYSTEM` (0). | `src/types/domain/ids.rs` |
| `pub fn is_valid_origin(&self) -> bool` | Checks if the transaction ID originates from a valid range per AGT-GRAPH-001. Returns `true` if `self.0 <= MAX_COLLECTION_SEQUENCE` (Collec… | `src/types/domain/ids.rs` |
| `pub fn try_from_internal_offset(offset: u64) -> Result<Self>` | Safely constructs an internal system TxId from an offset relative to `TxId::INTERNAL_BASE`. # Domain Range Separation (ADR-028) Valid syste… | `src/types/domain/ids.rs` |
| `pub fn try_new(id: u64) -> Result<Self>` | Sicherer Konstruktor. Gibt Err wenn id == 0. | `src/types/domain/ids.rs` |
| `pub fn try_new(id: u64) -> Result<Self>` | Creates a new `CollectionId`, ensuring `id != 0`. | `src/types/domain/ids.rs` |
| `pub fn as_metadata_key(&self) -> &'static str` | Gibt den kanonischen Metadaten-Key zurück (für JSON-Serialisierung). | `src/types/domain/misc.rs` |
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
| `pub fn with_tx_validity(mut self, from: Option<TxId>, to: Option<TxId>) -> Self` | Sets transaction validity window (system time / MVCC) on the edge. | `src/types/domain/misc.rs` |
| `pub fn with_validity(self, from: Option<TxId>, to: Option<TxId>) -> Self` | Sets transaction validity window on the edge (alias for `with_tx_validity` for backward compatibility). | `src/types/domain/misc.rs` |
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
| `pub fn graph(&self) -> f32` | Returns the graph signal weight component. | `src/types/saos.rs` |
| `pub fn has_context_prefix(&self) -> bool` | Gibt true zurück wenn contextual_prefix gesetzt und nicht leer. | `src/types/saos.rs` |
| `pub fn is_empty(&self) -> bool` | Returns `true` if all reformulation components are empty or None. | `src/types/saos.rs` |
| `pub fn metadata(&self) -> f32` | Returns the metadata signal weight component. Reserved for future metadata-signal fusion; currently always 0.0. Do not expose a public cons… | `src/types/saos.rs` |
| `pub fn new(vector: f32, text: f32, graph: f32) -> Result<Self>` | Creates 3-signal fusion weights (vector, text, graph) summing to 1.0. | `src/types/saos.rs` |
| `pub fn new( text_query: Option<String>, semantic_query: Option<String>, anchor_entities: Vec<String>, ) -> Self` | Creates a new `QueryRewriteOutput` with text, semantic, and anchor entity reformulations. | `src/types/saos.rs` |
| `pub fn new() -> Self` | Creates a new `HybridQueryBuilder` with empty default options. | `src/types/saos.rs` |
| `pub fn strategy_for_signal(&self, signal_name: &str) -> FusionStrategy` | Returns the fusion strategy configured for a specific signal name ("vector", "text", "graph"). | `src/types/saos.rs` |
| `pub fn text(&self) -> f32` | Returns the text signal weight component. | `src/types/saos.rs` |
| `pub fn text_only(text: impl Into<String>) -> Self` | Creates a simple text-only `QueryRewriteOutput`. | `src/types/saos.rs` |
| `pub fn uniform(strategy: FusionStrategy) -> Self` | Creates a uniform configuration where all signals use the specified strategy. | `src/types/saos.rs` |
| `pub fn vector(&self) -> f32` | Returns the vector signal weight component. | `src/types/saos.rs` |
| `pub fn with_filter(mut self, filter: FilterExpr) -> Self` | Sets a metadata expression filter on the query. | `src/types/saos.rs` |
| `pub fn with_fusion_strategy(mut self, strategy: impl Into<SignalFusionStrategies>) -> Self` | Sets search fusion strategy per signal or globally (`FusionStrategy` or `SignalFusionStrategies`). | `src/types/saos.rs` |
| `pub fn with_fusion_weights(mut self, weights: FusionWeights) -> Self` | Sets custom signal fusion weights. | `src/types/saos.rs` |
| `pub fn with_graph_start_node(mut self, start: impl Into<String>) -> Self` | Sets the graph start node for graph traversal search. | `src/types/saos.rs` |
| `pub fn with_graph_strategy(mut self, strategy: GraphTraversalStrategy) -> Self` | Sets the graph traversal strategy (Hops or PersonalizedPageRank). | `src/types/saos.rs` |
| `pub fn with_include_provenance(mut self, include: bool) -> Self` | Sets whether to calculate and attach ProvenanceRecord to output results. | `src/types/saos.rs` |
| `pub fn with_include_superseded(mut self, include: bool) -> Self` | Sets whether to include superseded memory chunks (ADR-038). | `src/types/saos.rs` |
| `pub fn with_k(mut self, k: usize) -> Self` | Sets the top-K limit for the query. | `src/types/saos.rs` |
| `pub fn with_memory_type_filter(mut self, types: Vec<MemoryType>) -> Self` | Sets the memory_type pre-RRF filter. | `src/types/saos.rs` |
| `pub fn with_on_signal_failure(mut self, policy: OnSignalFailure) -> Self` | Sets the signal failure policy ([`OnSignalFailure`]) for the query (Spec Teil 6.5). | `src/types/saos.rs` |
| `pub fn with_rerank_pool_max(mut self, max: usize) -> Self` | Sets the upper bound cap for pre-reranking candidate pool expansion. | `src/types/saos.rs` |
| `pub fn with_rerank_pool_multiplier(mut self, multiplier: usize) -> Self` | Sets the candidate pool multiplier for pre-reranking retrieval. | `src/types/saos.rs` |
| `pub fn with_same_community_as(mut self, entity_id: EntityId) -> Self` | Sets the community context entity filter/boost. | `src/types/saos.rs` |
| `pub fn with_text_query(mut self, q: impl Into<String>) -> Self` | Sets the full-text search query string. | `src/types/saos.rs` |
| `pub fn with_vector_query(mut self, v: Vec<f32>) -> Self` | Sets the vector query embedding. | `src/types/saos.rs` |
| `pub fn new(tenant_id: crate::TenantId, tx_id: crate::TxId) -> Self` | Creates a new `RequestContext`. # Parameter - `tenant_id`: Mandantenidentifikator. - `tx_id`: Zugeordnete Transaktions-ID. | `src/witness.rs` |
| `pub fn new(deleted_keys_hash: [u8; 32]) -> Self` | Creates a new `SagaRecord`. # Parameter - `deleted_keys_hash`: BLAKE3-Hash der zu löschenden Schlüssel. | `src/witness.rs` |


---

## `contextra-ports`

| Attribut | Wert |
|---|---|
| **Ring** | 0 — Ring 0 — Foundation & Pure Numerics/Types |
| **Status** | Fertig |
| **Zweck** | dyn-kompatible Trait-Definitionen (Ports) für Storage, Indizes, Embedder und System-Uhren. |
| **Unsafe-Code** | Nein (`#![forbid(unsafe_code)]`) |
| **Interne Abhängigkeiten** | `contextra-types` |
| **Externe Abhängigkeiten (Auswahl)** | `ahash`, `blake3`, `bytes`, `futures-util`, `serde`, `serde_json`, `thiserror` |

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
pub mod scanner;
pub mod storage;
pub mod text_index;
pub mod vector_index;
pub mod vfs;
pub mod contract_tests;
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
pub use scanner::{ LayerScanner, LayerWitness, PurgeStats, Purger, ScanSession, ScanTarget, ScanTechnique, WitnessLayer, };
pub use storage::*;
pub use text_index::*;
pub use vector_index::*;
pub use vfs::*;
pub use metrics::{ HistogramSnapshot, InMemoryMetricsSink, MetricKey, MetricSnapshot, NoopMetricsSink, };
```

### Feature-Flags

| Flag | Aktiviert |
|---|---|
| `default` | `[]` |
| `contract-tests` | `[]` |
| `testkit` | `[]` |

### Schnittstellen — Public Traits (33)

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

**`EmbeddingProvider`** — *Abstraction for all embedding providers (e.g. OllamaEmbedder, OnnxEmbedder).* (`src/embedding.rs`)

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

**`LayerScanner`** — *Trait für Storage-Scanner zur Durchführung von Restmengen-Messungen. # Doctests (Compile Fail Checks) (a) `LayerWitness` lässt sich außerhalb des Moduls nicht direkt konstruieren: ```compile_fail use contextra_ports::scanner::{LayerWitness, WitnessLayer, Scan…* (`src/scanner/mod.rs`)

```rust
pub trait LayerScanner : Send + Sync {
/// Gibt die von diesem Scanner bediente Schicht zurück.
    fn layer(&self) -> WitnessLayer;

    /// Führt das Scannen von Einheiten auf der übergebenen `ScanSession` aus.
    fn scan_units(
        &self,
        session: &mut ScanSession,
        ctx: &contextra_types::RequestContext,
    ) -> Result<()>;

    /// Default-Implementierung von `scan`, die `begin`, `scan_units` und `finish` ausführt.
    fn scan(
        &self,
        ctx: &contextra_types::RequestContext,
        target: ScanTarget,
        technique: ScanTechnique,
        clock: &dyn Clock,
    ) -> Result<LayerWitness> {
        let mut session = ScanSession::begin(self.layer(), target, technique);
        self.scan_units(&mut session, ctx)?;
        session.finish(clock)
    }
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

**`Purger`** — *Trait für physische Bereinigungsaktionen (Purger) einer Schicht.* (`src/scanner/layer.rs`)

```rust
pub trait Purger : Send + Sync {
/// Führt die physische Löschung für das gegebene Ziel aus.
    fn purge(
        &self,
        ctx: &contextra_types::RequestContext,
        target: &ScanTarget,
    ) -> Result<PurgeStats>;
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

**`TextGenerator`** — ** (`src/embedding.rs`)

```rust
pub trait TextGenerator : Send + Sync + 'static {
/// Generates text response for a given prompt.
    fn generate_text<'a>(&'a self, prompt: &'a str) -> BoxFuture<'a, crate::Result<String>>;
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

**`Vfs`** — *Abstract object-safe, synchronous Virtual File System interface. Implements minimal required operations for LSM storage engine, SSTables, WAL sidecars, and vector index file persistence.* (`src/vfs.rs`)

```rust
pub trait Vfs : Send + Sync + 'static {
/// Reads the entire contents of a file at `path`.
    ///
    /// # Errors
    /// Returns [`ContextraError::Io`](crate::error::ContextraError::Io) or [`ContextraError::FileNotFound`](crate::error::ContextraError::FileNotFound)
    /// if the file does not exist or cannot be read.
    fn read(&self, path: &Path) -> Result<Vec<u8>>;

    /// Atomically writes `data` to `path` with crash-consistency guarantees.
    ///
    /// # Crash-Consistency Contract
    /// Writes to a temporary file, calls `fsync` on the file handle, renames the temporary
    /// file over `path`, and syncs the parent directory (`sync_dir`).
    ///
    /// # Errors
    /// Returns [`ContextraError::Io`](crate::error::ContextraError::Io) if write, sync, or rename fails.
    fn write_atomic(&self, path: &Path, data: &[u8]) -> Result<()>;

    /// Creates a directory and all parent directories along `path` if missing.
    ///
    /// # Errors
    /// Returns [`ContextraError::Io`](crate::error::ContextraError::Io) if creation fails.
    fn create_dir_all(&self, path: &Path) -> Result<()>;

    /// Removes a file at `path`.
    ///
    /// # Errors
    /// Returns [`ContextraError::Io`](crate::error::ContextraError::Io) or [`ContextraError::FileNotFound`](crate::error::ContextraError::FileNotFound)
    /// if removal fails or file does not exist.
    fn remove_file(&self, path: &Path) -> Result<()>;

    /// Returns `true` if a file or directory exists at `path`.
    ///
    /// # Errors
    /// Returns [`ContextraError::Io`](crate::error::ContextraError::Io) if path state cannot be inspected.
    fn exists(&self, path: &Path) -> Result<bool>;

    /// Returns the length in bytes of the file at `path`.
    ///
    /// # Errors
    /// Returns [`ContextraError::Io`](crate::error::ContextraError::Io) or [`ContextraError::FileNotFound`](crate::error::ContextraError::FileNotFound)
    /// if file metadata cannot be retrieved.
    fn len(&self, path: &Path) -> Result<u64>;

    /// Lists entry names (file and directory names, excluding `.` and `..`) inside directory `path`.
    ///
    /// # Errors
    /// Returns [`ContextraError::Io`](crate::error::ContextraError::Io) or [`ContextraError::FileNotFound`](crate::error::ContextraError::FileNotFound)
    /// if path is not a directory or cannot be read.
    fn list_dir(&self, path: &Path) -> Result<Vec<String>>;

    /// Flushes directory metadata to persistent storage for durable file additions and renames.
    ///
    /// # Errors
    /// Returns [`ContextraError::Io`](crate::error::ContextraError::Io) if directory syncing fails.
    fn sync_dir(&self, path: &Path) -> Result<()>;
}
```

### Öffentliche Datentypen — Structs (32)

| Struct | Felder / Form | Beschreibung | Datei |
|---|---|---|---|
| `AuthorizedRing` | `{ ring: FeatureRing, }` | Token representing an authorized feature ring granting access to engine features (§15). `AuthorizedRing` cannot be directly instantiated outside of t… | `src/license.rs` |
| `ContextSegment` | `{ /// Unique identifier for the chunk. pub chunk_id: u64, /// Text content of the segment. pub text: &'a str, /// Optional model fingerprint reference. pub model… }` | Representation of a context segment passed to context-aware text generation. | `src/embedding.rs` |
| `GraphIndexStats` | `{ /// Number of active nodes (Entities). pub num_entities: usize, /// Number of active edges. pub num_edges: usize, /// Total bytes allocated by CSR representati… }` | Statistics for the GraphIndex layer. | `src/graph_index.rs` |
| `GroundingAssessment` | `{ /// Raw or calibrated confidence score in [0.0, 1.0] indicating attribution quality. pub score: f32, /// Indicates whether the grounding score meets the requir… }` | Result of a post-hoc grounding / attribution validation check. | `src/lifecycle.rs` |
| `HistogramSnapshot` | `{ /// Sum of all observed values. pub sum: f64, /// Total count of observations. pub count: u64, /// Minimum observed value (None if empty). pub min: Option<f64>… }` | Snapshot of a recorded histogram metric. | `src/metrics.rs` |
| `HyperEdgeId` | `(tuple/unit)` | — | `src/graph.rs` |
| `InMemoryMetricsSink` | `{ counters: RwLock<BTreeMap<MetricKey, Arc<AtomicU64>>>, gauges: RwLock<BTreeMap<MetricKey, f64>>, histograms: RwLock<BTreeMap<MetricKey, HistogramState>>, }` | Production-ready local thread-safe implementation of [`MetricsSink`]. Stores metrics strictly in-process with no external network exports (privacy-pr… | `src/metrics.rs` |
| `KvBlock` | `{ /// Block sequence identifier. pub block_id: u64, /// Encrypted or raw block tensor bytes. #[serde( serialize_with = "serialize_bytes", deserialize_with = "des… }` | Represents an exported KV block segment. | `src/kv.rs` |
| `KvLayout` | `{ /// Number of transformer layers. pub n_layer: u32, /// Number of KV heads. pub n_kv_head: u32, /// Dimension per head. pub head_dim: u32, /// Data type repres… }` | KV cache layer and dimensions layout configuration. | `src/kv.rs` |
| `KvPrefixHit` | `{ /// Number of tokens matched by the prefix. pub matched_tokens: usize, /// Extracted KV blocks for the matched prefix. pub blocks: Vec<KvBlock>, }` | Represents a prefix match hit in the KV store. | `src/kv.rs` |
| `LayerWitness` | `{ layer: WitnessLayer, scanned: u64, remaining: u64, technique: ScanTechnique, method: ScanMethod, digest: [u8; 32], measured_at: u64, }` | Verifiziertes Messergebnis eines Storage-Scanners (SPEC §12.7.4). Unter INV-WITNESS-MINT-1 kann ein `LayerWitness` ausschließlich in `ScanSession::fi… | `src/scanner/witness.rs` |
| `MemVfs` | `{ files: RwLock<BTreeMap<PathBuf, Vec<u8>>>, dirs: RwLock<BTreeMap<PathBuf, ()>>, }` | In-memory VFS reference implementation used strictly for contract self-testing. | `src/contract_tests.rs` |
| `MetricKey` | `{ /// Metric identifier (e.g. `lsm_commit_total`). pub name: String, /// Sorted label key-value pairs. pub labels: Vec<(String, String)>, }` | Key identifying a metric by name and sorted key-value labels. | `src/metrics.rs` |
| `MetricSnapshot` | `{ /// Counter metrics (accumulated values). pub counters: BTreeMap<MetricKey, u64>, /// Gauge metrics (last recorded f64 values). pub gauges: BTreeMap<MetricKey,… }` | In-memory snapshot of all metrics collected by [`InMemoryMetricsSink`]. | `src/metrics.rs` |
| `NoopMetricsSink` | `(tuple/unit)` | A no-op implementation of [`MetricsSink`] that silently discards all metrics events. | `src/metrics.rs` |
| `OpenFastGate` | `(tuple/unit)` | Default open activation gate allowing `Fast` ring features without restriction. | `src/license.rs` |
| `PluginCapability` | `{ /// Unique plugin name. pub name: &'static str, /// Plugin version tuple (major, minor, patch). pub version: (u32, u32, u32), /// Architecture ring in which th… }` | Capability descriptor for a plugin. | `src/plugin.rs` |
| `PluginRegistry` | `{ active: BTreeMap<String, PluginCapability>, active_conflicts: BTreeMap<String, Vec<String>>, gate: Arc<dyn LicenseGate>, }` | Registry managing plugin activations, license gating, and dependency resolution. | `src/plugin.rs` |
| `PrefixKey` | `{ /// Target model fingerprint. pub model: ModelFingerprint, /// SHA256/BLAKE3 hash of the tokenizer vocabulary. pub tokenizer_hash: [u8; 32], /// KV tensor memo… }` | Key identifying a model KV cache prefix structure. | `src/kv.rs` |
| `PurgeStats` | `{ /// Anzahl physisch gelöschter Einheiten. pub purged_units: u64, /// Anzahl freigegebener Bytes. pub bytes_freed: u64, }` | Statistiken über bereinigte Einheiten. | `src/scanner/layer.rs` |
| `RequestId` | `(tuple/unit)` | Unique identifier for an inference request within a process. | `src/attention.rs` |
| `RoleBinding` | `{ /// The role assigned to the entity in the hyperedge. pub role: RoleId, /// The bound entity. pub entity: EntityId, }` | Binding of an entity to a specific role within an n-ary hyperedge. | `src/graph.rs` |
| `RoleId` | `(tuple/unit)` | — | `src/graph.rs` |
| `RopeConfig` | `{ /// Base frequency. pub base: f32, /// Optional frequency scaling factor. pub scaling: Option<f32>, }` | Rotary position embedding (RoPE) configuration. | `src/kv.rs` |
| `ScanSession` | `{ layer: WitnessLayer, target: ScanTarget, technique: ScanTechnique, scanned: u64, remaining: u64, control_matches: u64, unit_hashes: Vec<[u8; 32]>, }` | Laufende Scan-Session zur kontrollierten Erzeugung eines `LayerWitness`. | `src/scanner/session.rs` |
| `ScanTarget` | `{ /// Bezeichner des Scan-Ziels (z. B. Tenant, Collection, Path). pub target_id: String, /// BLAKE3 Hash des Scan-Ziels. pub target_hash: [u8; 32], }` | Zielbeschreibung des Scans inklusive Hash. | `src/scanner/layer.rs` |
| `SeededRng` | `{ state: AtomicU64, }` | A fast, deterministic 64-bit pseudorandom number generator using SplitMix64 with atomic state transitions (`AtomicU64`). Lock-free and thread-safe wi… | `src/rng.rs` |
| `SequentialIdGen` | `{ next: AtomicU64, }` | A thread-safe ID generator that produces monotonically increasing 64-bit IDs. Wraps around safely on integer overflow (`u64::MAX` -> `0`) via atomic… | `src/id_gen.rs` |
| `StorageStats` | `{ /// Number of SSTable segments. pub num_segments: usize, /// Total size of all SSTables in bytes. pub total_size_bytes: u64, /// Total size of memtables in byt… }` | Statistics for the storage engine. | `src/storage.rs` |
| `SystemClock` | `{ start_instant: Instant, }` | Standard production implementation of [`Clock`] backed by [`SystemTime`] and [`Instant`]. | `src/clock.rs` |
| `TextIndexStats` | `{ /// Total number of documents indexed. pub num_documents: usize, /// Total number of tokens across all documents. pub num_tokens: usize, /// Estimated memory u… }` | Statistics for a text index. | `src/text_index.rs` |
| `VectorIndexStats` | `{ /// Number of active (non-deleted) vectors. pub num_vectors: usize, /// Estimated memory usage in bytes. pub memory_usage_bytes: usize, /// Number of HNSW laye… }` | Statistics for a vector index. | `src/vector_index.rs` |

### Öffentliche Datentypen — Enums (8)

| Enum | Varianten | Beschreibung | Datei |
|---|---|---|---|
| `EmbeddingError` | `/// Provider is unreachable or unavailable. #[error("Provider unavailable: {0}")] Unavailable(String), /// Computation failed during embedding generation. #[error("Embedding failed: {0}")] Computatio…` | Error types encountered during embedding operations. | `src/embedding.rs` |
| `FeatureRing` | `/// Fast ring (open source core). Fast, /// Sovereign ring (enterprise data sovereignty). Sovereign, /// Compliance ring (strict auditability and regulatory rules). Compliance,` | Feature-Ring gemäß Spec §15. | `src/license.rs` |
| `GraphMutationError` | `/// Lock acquisition timed out during concurrent mutation. #[error("Lock acquisition timed out: {0}")] LockAcquisitionTimeout(String), /// Duplicate hyperedge ID provided. #[error("Duplicate hyperedg…` | Error variants for graph mutation operations. | `src/graph.rs` |
| `LicenseError` | `/// No valid activation found for the requested feature ring. #[error("no valid activation found for feature ring {0:?}")] NotActivated(FeatureRing), /// Activation signature verification failed. #[e…` | Fehlerklasse für Lizenz-/Aktivierungsprüfung — crate-lokal, an der Grenze in `ContextraError` konvertierbar (P6-konform, siehe Spec §17). | `src/license.rs` |
| `PluginError` | `/// Dependency cycle or unresolvable missing dependency detected among plugins. #[error("dependency cycle detected among plugins: {0:?}")] DependencyCycle(Vec<String>), /// Conflicting plugins reques…` | Errors during plugin registration, activation, or dependency resolution. | `src/plugin.rs` |
| `ScanTechnique` | `/// Direkter struktureller Scan. Structural, /// Scan über entschlüsselte Ansicht. DecryptedView, /// Index-Introspektion. IndexIntrospection, /// Nadel-Scan (Needle Scan). NeedleScan, /// Schicht wu…` | Angewandte Methode/Technik während der Verifikation. | `src/scanner/layer.rs` |
| `VectorDeleteMode` | `/// Synchronous graph repair during vector deletion. SynchronousRepair, /// Asynchronous background graph repair during vector deletion. BackgroundRepair,` | Vector deletion mode for vector index maintenance (§15). | `src/license.rs` |
| `WitnessLayer` | `/// LSM Memtable LsmMemtable, /// SsTable Dateien über alle Level SsTableAllLevels, /// HNSW Vektorgraph-Index HnswIndex, /// Write-Ahead-Log Segmente WalAllSegments, /// Compressed Sparse Row Graph…` | Identifikator für die physische Storage-Schicht. | `src/scanner/layer.rs` |

### Typ-Aliase (2)

| Alias | Definition | Beschreibung |
|---|---|---|
| `BoxFuture` | `type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>` | Type alias for a pinned, heap-allocated `Future` that is `Send` and dyn-compatible. |
| `BoxStream` | `type BoxStream<'a, T> = Pin<Box<dyn futures_util::stream::Stream<Item = T> + Send + 'a>>` | Type alias for a pinned, heap-allocated `Stream` that is `Send` and dyn-compatible. |

### Konstanten (1)

| Konstante | Typ | Beschreibung |
|---|---|---|
| `MAX_SCAN_MERGE_ACCUMULATOR` | `usize` | Harte Obergrenze für die Anzahl distinkter Keys, die eine StorageEngine- Implementierung während eines einzelnen scan()/scan_prefix_bounded()-Aufrufs… |

### Öffentliche Funktionen (36)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn new() -> Self` | Creates a new [`SystemClock`] using the current [`Instant`] as the monotonic baseline. | `src/clock.rs` |
| `pub fn new() -> Self` | Creates a new [`MemVfs`]. | `src/contract_tests.rs` |
| `pub fn verify_clock_contract<C: Clock>(clock: &C)` | Contract test helper verifying [`Clock`] trait implementation compliance. Checks monotonic non-decreasing behavior and valid wall-clock epo… | `src/contract_tests.rs` |
| `pub fn verify_metrics_sink_contract<M: MetricsSink>(sink: &M)` | Contract test helper verifying [`MetricsSink`] trait implementation compliance. | `src/contract_tests.rs` |
| `pub fn verify_rng_contract<R: Rng>(rng: &mut R)` | Contract test helper verifying [`Rng`] trait implementation compliance. Checks non-zero outputs, correct buffer filling, and reproducible s… | `src/contract_tests.rs` |
| `pub fn verify_vfs_contract<V: Vfs>(vfs: &V, base_dir: &Path)` | Contract test helper verifying [`Vfs`] trait implementation compliance. Verifies atomic replacement, read roundtrips, directory creation, f… | `src/contract_tests.rs` |
| `pub fn new(chunk_id: u64, text: &'a str) -> Self` | Creates a new `ContextSegment` with default optional metadata set to `None`. | `src/embedding.rs` |
| `pub fn with_fingerprint(mut self, fingerprint: &'a ModelFingerprint) -> Self` | Builder method to set the model fingerprint. | `src/embedding.rs` |
| `pub fn with_rope_offset(mut self, offset: usize) -> Self` | Builder method to set the RoPE offset. | `src/embedding.rs` |
| `pub fn new(role: RoleId, entity: EntityId) -> Self` | Creates a new `RoleBinding`. | `src/graph.rs` |
| `pub fn new(start: u64) -> Self` | Creates a new [`SequentialIdGen`] starting at `start`. | `src/id_gen.rs` |
| `pub fn ring(&self) -> FeatureRing` | Returns the authorized [`FeatureRing`]. | `src/license.rs` |
| `pub fn new(name: impl Into<String>, labels: &[(&str, &str)]) -> Self` | Creates a new normalized [`MetricKey`]. | `src/metrics.rs` |
| `pub fn new() -> Self` | Creates a new empty [`InMemoryMetricsSink`]. | `src/metrics.rs` |
| `pub fn snapshot(&self) -> MetricSnapshot` | Captures a point-in-time [`MetricSnapshot`] of all recorded counters, gauges, and histograms. | `src/metrics.rs` |
| `pub fn activate_all( &mut self, plugins: Vec<Box<dyn PluginManifest>>, ) -> Result<(), PluginError>` | Activates a batch of plugins in an all-or-nothing manner (INV-PLUGIN-DEPENDENCY). Sequence: 1. Conflict checks across requested and active… | `src/plugin.rs` |
| `pub fn current_feature_ring(&self) -> FeatureRing` | Returns the highest currently active `FeatureRing` according to the license gate. | `src/plugin.rs` |
| `pub fn is_active(&self, name: &str) -> bool` | Checks whether a plugin with the given name is currently active. | `src/plugin.rs` |
| `pub fn new( name: &'static str, version: (u32, u32, u32), ring: u8, feature_ring_required: FeatureRing, ) -> Self` | Creates a new `PluginCapability`. | `src/plugin.rs` |
| `pub fn new(gate: Arc<dyn LicenseGate>) -> Self` | Creates a new `PluginRegistry` bounded by the given `LicenseGate`. | `src/plugin.rs` |
| `pub fn snapshot(&self) -> Vec<PluginCapability>` | Takes a snapshot of all active plugin capabilities. | `src/plugin.rs` |
| `pub fn from_entropy() -> Self` | Creates a new [`SeededRng`] seeded from process/runtime entropy. # Non-Cryptographic Usage This entropy source uses [`std::collections::has… | `src/rng.rs` |
| `pub fn new(seed: u64) -> Self` | Creates a new [`SeededRng`] initialized with the given seed. | `src/rng.rs` |
| `pub fn as_str(&self) -> &str` | Gibt den String-Namen der Schicht zurück. | `src/scanner/layer.rs` |
| `pub fn new(target_id: impl Into<String>, target_hash: [u8; 32]) -> Self` | Erzeugt ein neues ScanTarget. | `src/scanner/layer.rs` |
| `pub fn record_control_match(&mut self)` | Registriert eine bestandene Positivkontrolle (Control Match). | `src/scanner/session.rs` |
| `pub fn record_unit(&mut self, unit_hash: [u8; 32], is_live_residual: bool)` | Registriert eine gescannte Einheit. | `src/scanner/session.rs` |
| `pub fn digest(&self) -> &[u8` | Gibt den BLAKE3-Digest der Messung zurück. | `src/scanner/witness.rs` |
| `pub fn layer(&self) -> &str` | Gibt die gescannte Storage-Schicht als &str zurück. | `src/scanner/witness.rs` |
| `pub fn measured_at(&self) -> u64` | Gibt den Zeitstempel der Messung (Unix Nanos) zurück. | `src/scanner/witness.rs` |
| `pub fn method(&self) -> &ScanMethod` | Gibt die ScanMethod-Entsprechung zurück. | `src/scanner/witness.rs` |
| `pub fn mint_witness_for_testkit( layer: WitnessLayer, scanned: u64, remaining: u64, technique: ScanTechnique, digest: [u8; 32], measured_at: u64, ) -> LayerWitness` | Hilfsfunktion zur Witness-Erzeugung ausschließlich für Testzwecke (`testkit`-Feature). | `src/scanner/witness.rs` |
| `pub fn remaining(&self) -> u64` | Gibt die Anzahl der verbleibenden Lebendeinträge zurück. | `src/scanner/witness.rs` |
| `pub fn scanned(&self) -> u64` | Gibt die Anzahl der während der Verifikation gescannten Einheiten zurück. | `src/scanner/witness.rs` |
| `pub fn technique(&self) -> &ScanTechnique` | Gibt die angewandte Scantechnik zurück. | `src/scanner/witness.rs` |
| `pub fn witness_layer(&self) -> &WitnessLayer` | Gibt die gescannte Storage-Schicht als &WitnessLayer zurück. | `src/scanner/witness.rs` |


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

**`SsiValidator`** — *Schnittstelle (Trait) für die Validierung von Transaktionen unter Serializable Snapshot Isolation (SSI). Prüft vor dem Festschreiben (Commit) einer Transaktion, ob nebenläufige Transaktionen Schlüssel oder Präfixe verändert haben, die im [`ReadSet`] dieser Tr…* (`src/ssi.rs`)

```rust
pub trait SsiValidator {
/// Validiert das [`ReadSet`] der Transaktion `tx_id`.
    ///
    /// ### Fehlerfälle:
    /// Gibt `Err(ContextraError::Conflict(...))` zurück, falls:
    /// - Ein Schlüssel im ReadSet nach dem Lese-Snapshot von einer anderen Transaktion committet wurde (Write-Skew).
    /// - Ein neuer Schlüssel committet wurde, der auf einen gelesenen Bereichsscan-Präfix zutrifft (Phantom-Read).
    /// - Der Snapshot-Wasserstand der Transaktion älter ist als die bereits bereinigte Historie (`pruned_through`).
    ///
    /// ### Nebenläufigkeits-Invariante:
    /// Aufrufer koordinieren Validierung und Commit-Registrierung (z. B. via Commit-Gate-Lock),
    /// um atomare, konfliktfreie Zustandsübergänge sicherzustellen.
    fn validate(&self, tx_id: TxId, read_set: &ReadSet) -> Result<()>;
}
```

**`StagedOpSize`** — *Trait zur Schätzung des Speicherverbrauchs in Bytes für gepufferte Nutzdaten. Ermöglicht genaue Buchführung des verbrauchten Hauptspeichers pro Transaktion und global.* (`src/tx_buffer.rs`)

```rust
pub trait StagedOpSize {
/// Liefert den geschätzten Speicherbedarf der Nutzdaten in Bytes.
    fn staged_bytes(&self) -> usize;
}
```

### Öffentliche Datentypen — Structs (9)

| Struct | Felder / Form | Beschreibung | Datei |
|---|---|---|---|
| `PruningBlockerInfo` | `{ /// Kleinste Sequenznummer, die noch unbereinigt im Register liegt. pub min_unpruned_seq: u64, /// Aktueller `pruned_through`-Wasserstand. pub pruned_through_s… }` | Diagnoseinformationen über aktive Snapshots oder Sequenznummern, die das Pruning im Validator blockieren. Wird erzeugt, wenn der Speicher für Commit-… | `src/ssi.rs` |
| `ReadSet` | `{ /// Zuordnung von gelesenen Schlüsseln (Bytefolgen) zur zugehörigen Snapshot-Sequenznummer. keys: AHashMap<Vec<u8>, u64>, /// Zuordnung von Scan-Präfixen zu de… }` | Protokolliert gelesene Einzelschlüssel, Präfixe und deren jeweilige Snapshot-Sequenznummer für SSI. ### Speicher- und Risikoanalyse: Der Speicherbeda… | `src/ssi.rs` |
| `SeqLogEntry` | `{ /// Eindeutige Kennung des Dokuments. pub doc_id: DocId, /// Logische Sequenznummer, bei der dieser Dokumenteintrag eingefügt wurde. pub insert_seq: u64, /// L… }` | Ein versionierter Protokolleintrag für die Snapshot-Isolation (`_at`-Familie). Jeder Eintrag repräsentiert die Lebensspanne einer Dokumentversion: vo… | `src/seq_log.rs` |
| `SequenceLog` | `{ /// Liste aller historischen Dokumentversionen. entries: Vec<SeqLogEntry>, /// Zuordnung von gepinnter Sequenznummer zu Liste von Erstellungs-Zeitstempeln (in… }` | Verwaltet das chronologische Sequenz-Protokoll und Sichtbarkeitsfilter für Indexstrukturen. ### Funktionsweise: `SequenceLog` speichert alle Einfüge-… | `src/seq_log.rs` |
| `SequenceLogSsiValidator` | `{ committed_writes: Arc<RwLock<CommittedWrites>>, sequence_log: Option<Arc<RwLock<SequenceLog>>>, snapshot_registry: Option<Arc<SnapshotRegistry>>, pruned_throug… }` | Referenz-Implementierung von [`SsiValidator`] auf Basis von [`SequenceLog`] und In-Memory-Commit-Tracking. Arbeitet vollständig ohne Festplatten-I/O… | `src/ssi.rs` |
| `SnapshotGuard` | `{ registry: Arc<SnapshotRegistry>, seq_no: u64, created_at: Option<u64>, }` | RAII-Guard (Resource Acquisition Is Initialization) für einen aktiven Lese-Snapshot. Hält einen Snapshot in der [`SnapshotRegistry`] gepinnt, solange… | `src/snapshot.rs` |
| `SnapshotRegistry` | `{ active: Mutex<BTreeMap<u64, Vec<u64>>>, min_active_seqno: AtomicU64, clock: Arc<dyn Clock>, }` | Zentrale Registrierung für aktive Lese-Snapshots im System. ### Synchronisationsstrategie: - `active`: Ein sortierter `BTreeMap`, geschützt durch ein… | `src/snapshot.rs` |
| `TxBuffer` | `{ shards: Vec<RwLock<TxShard<T>>>, key_shards: Vec<RwLock<KeyShard>>, tx_timeout: Duration, config: TxBufferConfig, global_staged_bytes: AtomicUsize, clock: Arc<… }` | Verwaltet gepufferte Transaktionen, deren Staging-Operationen und ReadSets. ### Synchronisations- und Sperrstrategie: Jeder Shard besitzt ein eigenes… | `src/tx_buffer.rs` |
| `TxBufferConfig` | `{ /// Timeout-Dauer, nach der verwaiste (inaktive) Transaktionen automatisch abgeräumt werden. pub tx_timeout: Duration, /// Maximale empfohlene Anzahl gleichzei… }` | Konfigurationsoptionen für den [`TxBuffer`]. | `src/tx_buffer.rs` |

### Öffentliche Datentypen — Enums (2)

| Enum | Varianten | Beschreibung | Datei |
|---|---|---|---|
| `IndexOp` | `/// Einfügen eines Dokuments mit Nutzlast (z. B. Einbettungs-Vektoren oder Schlüssel-Werte). Insert { /// Kennung des Dokuments. doc_id: DocId, /// Assoziierte Nutzdaten. data: T, }, /// Löschen eine…` | Eine im Puffer vorgemerkte Operation auf einem Index. | `src/tx_buffer.rs` |
| `SeqLogChange` | `/// Einfügen eines Dokuments zu einer bestimmten Sequenznummer. Insert { /// Eindeutige Kennung des Dokuments. doc_id: DocId, /// Sequenznummer des Einfügens. seq: u64, }, /// Löschen eines Dokuments…` | Repräsentiert eine historische Änderung (Einfügen oder Löschen) im Sequenz-Log für Delta-Replays. Wird von Abfragen wie [`SequenceLog::changes_since`… | `src/seq_log.rs` |

### Konstanten (8)

| Konstante | Typ | Beschreibung |
|---|---|---|
| `DEFAULT_MAX_OPS_PER_TX` | `usize` | — |
| `DEFAULT_MAX_PIN_DURATION` | `Duration` | Standard-Lebensdauer für Snapshot-Pins (5 Minuten), bevor Diagnose-Warnungen für blockierende oder verwaiste Pins ausgelöst werden. |
| `DEFAULT_MAX_READ_SET_KEYS` | `usize` | Standard-Maximalanzahl an getrackten Lese-Schlüsseln pro Transaktion (100.000 Schlüssel). |
| `DEFAULT_MAX_TOTAL_STAGED_BYTES` | `usize` | Standard-Maximalgröße für den globalen Puffer über alle Transaktionen hinweg (256 MiB). |
| `DEFAULT_MAX_TRACKED_COMMIT_KEYS` | `usize` | Standard-Höchstgrenze für im Speicher gehaltene bestätigte Schreib-Schlüssel (1.000.000 Schlüssel). |
| `DEFAULT_MAX_TX_STAGED_BYTES` | `usize` | Standard-Maximalgröße für gepufferte Rohdaten pro Einzelschreibtransaktion (16 MiB). |
| `DEFAULT_SHARD_COUNT` | `usize` | Standardanzahl an Partitionen (Shards) im Transaktionspuffer (64 Shards). |
| `STAGING_ENTRY_OVERHEAD_BYTES` | `usize` | Fester Speicher-Overhead pro gepufferter Operation (32 Bytes). |

### Öffentliche Funktionen (113)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn changes_since(&self, snapshot_seq: u64) -> Vec<SeqLogChange>` | Liefert alle Änderungen (Einfügungen und Löschungen), die strikt nach `snapshot_seq` stattgefunden haben, aufsteigend sortiert nach Sequenz… | `src/seq_log.rs` |
| `pub fn clock(&self) -> &Arc<dyn Clock>` | Gibt eine Referenz auf den aktiven [`Clock`]-Port zurück. | `src/seq_log.rs` |
| `pub fn compact(&mut self, min_active_seqno: u64)` | Verdichtet (kompaktiert) das Sequenz-Log und entfernt historische Versionen, deren Löschung strikt kleiner als `min_active_seqno` ist. ###… | `src/seq_log.rs` |
| `pub fn deletion_seq(&self, doc_id: DocId) -> Option<u64>` | Liefert die Lösch-Sequenznummer des Dokuments, falls es aktuell als gelöscht verzeichnet ist. | `src/seq_log.rs` |
| `pub fn doc_id(&self) -> DocId` | Liefert die Kennung des betroffenen Dokuments. | `src/seq_log.rs` |
| `pub fn expired_pins(&self) -> Vec<u64>` | Liefert alle abgelaufenen Snapshot-Pins bezogen auf die aktuelle Systemuhr. | `src/seq_log.rs` |
| `pub fn expired_pins_at(&self, now_nanos: u64) -> Vec<u64>` | Findet alle gepinnten Sequenznummern, deren Haltezeit `max_pin_duration` überschreitet, bezogen auf den übergebenen Monotonie-Zeitstempel `… | `src/seq_log.rs` |
| `pub fn is_empty(&self) -> bool` | Gibt `true` zurück, wenn das Sequenz-Log keine Einträge enthält. | `src/seq_log.rs` |
| `pub fn is_visible(&self, as_of: u64) -> bool` | Prüft in O(1), ob dieser Protokolleintrag zu einem bestimmten Snapshot-Stand `as_of` sichtbar ist. ### Ablauf der Sichtbarkeitsprüfung: 1.… | `src/seq_log.rs` |
| `pub fn is_visible(&self, doc_id: DocId, seq_no: u64) -> bool` | Prüft, ob ein Dokument zu einer bestimmten Sequenznummer `seq_no` im Index sichtbar ist. Ein Dokument ist sichtbar, wenn es vor oder bei `s… | `src/seq_log.rs` |
| `pub fn len(&self) -> usize` | Gibt die Anzahl der aktuell im Log vorgehaltenen Einträge zurück. | `src/seq_log.rs` |
| `pub fn max_pin_duration(&self) -> Duration` | Liefert die aktuell konfigurierte Maximaldauer für Pins. | `src/seq_log.rs` |
| `pub fn min_retention_seq(&self) -> Option<u64>` | Berechnet die kleinste aktuell gepinnte Sequenznummer bezogen auf die aktuelle Systemuhr. | `src/seq_log.rs` |
| `pub fn min_retention_seq_at(&self, now_nanos: u64) -> Option<u64>` | Berechnet die kleinste aktuell gepinnte Sequenznummer für Aufbewahrungs- und Pruning-Zwecke. Gibt `None` zurück, falls aktuell kein einzige… | `src/seq_log.rs` |
| `pub fn new() -> Self` | Erstellt ein neues, leeres `SequenceLog` mit der Standard-Systemuhr [`SystemClock`] und einer maximalen Pin-Dauer von 5 Minuten. | `src/seq_log.rs` |
| `pub fn pin_snapshot(&mut self, seq_no: u64)` | Pinnt eine historische Sequenznummer, um zu verhindern, dass die Hintergrund-Bereinigung (Rebuild/Compaction) gelöschte Knoten entfernt, di… | `src/seq_log.rs` |
| `pub fn pin_snapshot_at(&mut self, seq_no: u64, at_nanos: u64)` | Pinnt eine Sequenznummer zu einem expliziten Nanosekunden-Zeitstempel (für deterministische Tests). | `src/seq_log.rs` |
| `pub fn reap_expired_pins(&mut self) -> Vec<u64>` | Bereinigt alle abgelaufenen Snapshot-Pins bezogen auf die aktuelle Uhrzeit. | `src/seq_log.rs` |
| `pub fn reap_expired_pins_at(&mut self, now_nanos: u64) -> Vec<u64>` | Bereinigt (entpinnt) alle Pins, die die Maximaldauer bezüglich `now_nanos` überschritten haben. ### Ablauf: 1. Abgelaufene Pins via [`Seque… | `src/seq_log.rs` |
| `pub fn record_delete(&mut self, doc_id: DocId, seq: u64)` | Zeichnet eine Löschoperation für ein Dokument zu einer Sequenznummer auf. ### Ablauf: 1. Tombstone-Bit der Sequenznummer ausmaskieren. 2. D… | `src/seq_log.rs` |
| `pub fn record_insert(&mut self, doc_id: DocId, seq: u64)` | Zeichnet eine Einfügeoperation für ein Dokument zu einer Sequenznummer auf. ### Ablauf: 1. Tombstone-Bit der Sequenznummer ausmaskieren. 2.… | `src/seq_log.rs` |
| `pub fn seq(&self) -> u64` | Liefert die logische Sequenznummer dieser Änderung. | `src/seq_log.rs` |
| `pub fn set_clock(&mut self, clock: Arc<dyn Clock>)` | Ersetzt die aktive [`Clock`]-Instanz für dieses Log. | `src/seq_log.rs` |
| `pub fn set_max_pin_duration(&mut self, duration: Duration)` | Setzt die maximale Pin-Dauer für Diagnosezwecke. | `src/seq_log.rs` |
| `pub fn unpin_snapshot(&mut self, seq_no: u64)` | Gibt einen gepinnten Snapshot wieder frei. ### Ablauf: 1. Tombstone-Bit der Sequenznummer ausmaskieren. 2. Den letzten Zeitstempel vom Stac… | `src/seq_log.rs` |
| `pub fn with_clock(clock: Arc<dyn Clock>) -> Self` | Erstellt ein neues, leeres `SequenceLog` mit einer benutzerdefinierten Implementierung des [`Clock`]-Ports (z. B. `ManualClock` in determin… | `src/seq_log.rs` |
| `pub fn with_max_pin_duration(mut self, duration: Duration) -> Self` | Konfiguriert die maximale Pin-Dauer als Fluent-Builder und gibt `self` zurück. | `src/seq_log.rs` |
| `pub fn clock(&self) -> &Arc<dyn Clock>` | Liefert eine Referenz auf den aktiven Zeitport ([`Clock`]). | `src/snapshot.rs` |
| `pub fn longest_active_pin(&self) -> Option<(u64, Duration)>` | Findet den am längsten aktiven Snapshot bezogen auf die aktuelle Systemuhr. | `src/snapshot.rs` |
| `pub fn longest_active_pin_at(&self, now_nanos: u64) -> Option<(u64, Duration)>` | Findet den am längsten aktiven Snapshot bezüglich des Nanosekunden-Zeitstempels `now_nanos`. Liefert `Some((seq_no, duration))` oder `None`… | `src/snapshot.rs` |
| `pub fn min_active_seqno(&self) -> u64` | Liefert die kleinste Sequenznummer aller derzeit aktiven Snapshots (`u64::MAX`, falls keine aktiv). ### Nebenläufigkeit: Vollständig sperr-… | `src/snapshot.rs` |
| `pub fn new() -> Self` | Erstellt eine neue, leere Snapshot-Registry mit Standard-[`SystemClock`]. | `src/snapshot.rs` |
| `pub fn pin(&self, seq_no: u64)` | Dauerhaftes Pinnen einer Sequenznummer (z. B. für SAOS Checkpoints oder Backups), ohne einen RAII-Guard zu halten. Muss manuell über [`Snap… | `src/snapshot.rs` |
| `pub fn pin_at(&self, seq_no: u64, at_nanos: u64)` | Pinnt eine Sequenznummer zu einem spezifischen Zeitstempel. | `src/snapshot.rs` |
| `pub fn reap_expired_pins(&self, max_pin_duration: Duration) -> Vec<u64>` | Bereinigt alle überalterten Snapshot-Pins bezogen auf die aktuelle Uhrzeit. | `src/snapshot.rs` |
| `pub fn reap_expired_pins_at(&self, max_pin_duration: Duration, now_nanos: u64) -> Vec<u64>` | Bereinigt und entfernt alle aktiven Snapshot-Pins, die älter als `max_pin_duration` sind. ### Ablauf: 1. Berechne Schwellenwert in Nanoseku… | `src/snapshot.rs` |
| `pub fn register(self: &Arc<Self>, seq_no: u64) -> SnapshotGuard` | Registriert einen Lese-Snapshot und liefert einen RAII-Guard ([`SnapshotGuard`]) zurück. Sobald der Guard den Gültigkeitsbereich verlässt (… | `src/snapshot.rs` |
| `pub fn register_at(self: &Arc<Self>, seq_no: u64, at_nanos: u64) -> SnapshotGuard` | Registriert einen Snapshot mit explizitem Zeitstempel (für deterministische Tests). ### Ablauf: 1. Tombstone-Bit aus der Sequenznummer entf… | `src/snapshot.rs` |
| `pub fn seq_no(&self) -> u64` | Liefert die Sequenznummer, die durch diesen Guard geschützt wird. | `src/snapshot.rs` |
| `pub fn unpin(&self, seq_no: u64)` | Hebt einen dauerhaften Pin auf und bereinigt den minimal aktiven Wasserstand. | `src/snapshot.rs` |
| `pub fn with_clock(clock: Arc<dyn Clock>) -> Self` | Erstellt eine neue Snapshot-Registry mit benutzerdefiniertem [`Clock`]-Port (für Tests). | `src/snapshot.rs` |
| `pub fn clear(&mut self)` | Leert das ReadSet vollständig und setzt die kleinste Sequenznummer zurück. | `src/ssi.rs` |
| `pub fn clock(&self) -> &Arc<dyn Clock>` | Liefert eine Referenz auf den aktiven [`Clock`]-Port. | `src/ssi.rs` |
| `pub fn diagnose_pruning_blocker(&self) -> Option<PruningBlockerInfo>` | Diagnostiziert, welcher unbereinigte Stand oder aktiver Snapshot das Pruning im Validator blockiert. | `src/ssi.rs` |
| `pub fn diagnose_pruning_blocker_at(&self, now_nanos: u64) -> Option<PruningBlockerInfo>` | Diagnostiziert Blockaden bezüglich eines expliziten Zeitstempels `now_nanos`. | `src/ssi.rs` |
| `pub fn forget_from(&self, first_seq: u64) -> usize` | Entfernt alle Commit-Registrierungen mit `commit_seq >= first_seq` (z. B. nach Transaktionsabbruch). | `src/ssi.rs` |
| `pub fn get(&self, key: &[u8]) -> Option<u64>` | Liefert die registrierte Snapshot-Sequenznummer für einen Schlüssel, falls vorhanden. | `src/ssi.rs` |
| `pub fn is_empty(&self) -> bool` | Liefert `true`, falls weder Einzelschlüssel noch Präfixe im ReadSet enthalten sind. | `src/ssi.rs` |
| `pub fn iter(&self) -> impl Iterator<Item = (&Vec<u8>, &u64)>` | Liefert einen Iterator über alle gelesenen Schlüssel und deren Snapshot-Sequenzen. | `src/ssi.rs` |
| `pub fn keys_map(&self) -> &AHashMap<Vec<u8>, u64>` | Liefert die interne Schlüssel-Map als Referenz. | `src/ssi.rs` |
| `pub fn len(&self) -> usize` | Liefert die Gesamtzahl aller getrackten Schlüssel und Präfixe. | `src/ssi.rs` |
| `pub fn max_tracked_keys(&self) -> usize` | Liefert die konfigurierte Obergrenze für Schlüssel. | `src/ssi.rs` |
| `pub fn min_snapshot_seq(&self) -> Option<u64>` | Liefert die kleinste Snapshot-Sequenznummer aller gelesenen Schlüssel und Präfixe. | `src/ssi.rs` |
| `pub fn new() -> Self` | Erstellt ein neues, leeres [`ReadSet`]. | `src/ssi.rs` |
| `pub fn new() -> Self` | Erstellt einen neuen [`SequenceLogSsiValidator`] mit Standard-Obergrenze (1.000.000 Schlüssel) und Standard-[`SystemClock`]. | `src/ssi.rs` |
| `pub fn new_with_bounds(max_tracked_keys: usize) -> Self` | Erstellt einen neuen Validator mit benutzerdefinierter Kapazitätsgrenze für Schlüssel. | `src/ssi.rs` |
| `pub fn new_with_clock_and_bounds(clock: Arc<dyn Clock>, max_tracked_keys: usize) -> Self` | Erstellt einen neuen Validator mit benutzerdefiniertem [`Clock`]-Port und Kapazitätsgrenze. | `src/ssi.rs` |
| `pub fn prefixes_iter(&self) -> impl Iterator<Item = (&Vec<u8>, &u64)>` | Liefert einen Iterator über alle Bereichs-Präfixe und deren Snapshot-Sequenzen. | `src/ssi.rs` |
| `pub fn prefixes_map(&self) -> &AHashMap<Vec<u8>, u64>` | Liefert die interne Präfix-Map als Referenz. | `src/ssi.rs` |
| `pub fn prune_through(&self, bound_seq: u64) -> usize` | Bereinigt (prunt) bestätigte Schreib-Einträge mit `commit_seq <= bound_seq`. Aktualisiert den atomaren `pruned_through`-Wasserstand. ### Fa… | `src/ssi.rs` |
| `pub fn pruned_through(&self) -> u64` | Liefert den aktuellen `pruned_through`-Wasserstand. | `src/ssi.rs` |
| `pub fn record_commit_key(&self, key: &[u8], commit_seq: u64)` | Zeichnet einen committeten Schlüssel zu einer Sequenznummer auf. ### Ablauf: 1. Tombstone-Bit der Commit-Sequenznummer ausmaskieren. 2. Sch… | `src/ssi.rs` |
| `pub fn record_commit_keys<'a>( &self, keys: impl IntoIterator<Item = &'a [u8]>, commit_seq: u64, )` | Zeichnet eine Menge committeter Schlüssel stapelweise (Batch) zu einer Commit-Sequenznummer auf. | `src/ssi.rs` |
| `pub fn record_prefix(&mut self, prefix: impl Into<Vec<u8>>, snapshot_seq: u64)` | Zeichnet einen Bereichsscan-Präfix zu `snapshot_seq` auf, um Phantom-Reads zu erkennen. Wenn nach diesem Lesezugriff eine andere Transaktio… | `src/ssi.rs` |
| `pub fn record_read(&mut self, key: impl Into<Vec<u8>>, snapshot_seq: u64)` | Zeichnet einen Lesezugriff auf `key` zu einer bestimmten Snapshot-Sequenznummer `snapshot_seq` auf. ### Ablauf: 1. Maskiere das Tombstone-B… | `src/ssi.rs` |
| `pub fn register_read(&mut self, key: impl Into<Vec<u8>>, snapshot_seq: u64)` | Alias-Funktion für [`ReadSet::record_read`]. | `src/ssi.rs` |
| `pub fn set_clock(&mut self, clock: Arc<dyn Clock>)` | Ersetzt die aktive [`Clock`]-Instanz. | `src/ssi.rs` |
| `pub fn set_metrics_sink( &mut self, sink: Arc<parking_lot::RwLock<Arc<dyn contextra_ports::MetricsSink>>>, )` | Aktualisiert den Metrik-Empfänger. | `src/ssi.rs` |
| `pub fn set_snapshot_registry(&mut self, registry: Arc<SnapshotRegistry>)` | Setzt die verknüpfte [`SnapshotRegistry`]. | `src/ssi.rs` |
| `pub fn tracked_commit_keys(&self) -> usize` | Liefert die Anzahl aktuell gehaltener Commit-Schlüssel. | `src/ssi.rs` |
| `pub fn validate_and_record<'a>( &self, tx_id: TxId, read_set: &ReadSet, write_keys: impl IntoIterator<Item = &'a [u8]>, commit_seq: u64, ) -> Result<()>` | Führt die Validierung und das Eintragen der Commit-Schlüssel atomar unter demselben Schreib-Lock aus. Verhindert Race Conditions zwischen d… | `src/ssi.rs` |
| `pub fn with_clock(mut self, clock: Arc<dyn Clock>) -> Self` | Bindet eine benutzerdefinierte Implementierung des [`Clock`]-Ports an den Validator. | `src/ssi.rs` |
| `pub fn with_max_pin_duration(mut self, duration: Duration) -> Self` | Konfiguriert die maximale Pin-Dauer für Diagnosealarme. | `src/ssi.rs` |
| `pub fn with_metrics_sink( mut self, sink: Arc<parking_lot::RwLock<Arc<dyn contextra_ports::MetricsSink>>>, ) -> Self` | Bindet einen Metrik-Empfänger ([`contextra_ports::MetricsSink`]) für Monitoring an. | `src/ssi.rs` |
| `pub fn with_sequence_log(seq_log: Arc<RwLock<SequenceLog>>) -> Self` | Erstellt einen Validator, der an ein bestehendes [`SequenceLog`] angebunden ist. | `src/ssi.rs` |
| `pub fn with_snapshot_registry(mut self, registry: Arc<SnapshotRegistry>) -> Self` | Verknüpft eine [`SnapshotRegistry`] zur Überwachung aktiver Snapshot-Pins. | `src/ssi.rs` |
| `pub fn begin(&self, tx: TxId)` | Registriert eine neue Transaktion mit der aktuellen Uhrzeit. | `src/tx_buffer.rs` |
| `pub fn begin_at(&self, tx: TxId, at_nanos: u64)` | Registriert eine neue Transaktion im Puffer zu einem bestimmten Nanosekunden-Zeitstempel. ### Ablauf: 1. Shard-Index für `tx` per Hash bere… | `src/tx_buffer.rs` |
| `pub fn clock(&self) -> &Arc<dyn Clock>` | Liefert eine Referenz auf den aktiven [`Clock`]-Port. | `src/tx_buffer.rs` |
| `pub fn discard(&self, tx: TxId)` | Verwirft alle gepufferten Operationen und das ReadSet einer Transaktion (Rollback). | `src/tx_buffer.rs` |
| `pub fn discard_kv(&self, tx: TxId)` | Verwirft alle KV-Operationen einer Transaktion und bereinigt das Key-Staging. | `src/tx_buffer.rs` |
| `pub fn doc_id(&self) -> DocId` | Liefert die Kennung des Dokuments (`DocId`), das von dieser Operation betroffen ist. | `src/tx_buffer.rs` |
| `pub fn drain(&self, tx: TxId) -> Vec<IndexOp<T>>` | Entnimmt alle gepufferten Operationen der Transaktion `tx` für den anschließenden Commit und gibt den reservierten Speicherplatz frei. | `src/tx_buffer.rs` |
| `pub fn drain_kv(&self, tx: TxId) -> Vec<IndexOp<(Vec<u8>, Vec<u8>)>>` | Entnimmt alle gepufferten KV-Operationen einer Transaktion und bereinigt das Key-Staging. | `src/tx_buffer.rs` |
| `pub fn estimated_bytes(&self) -> usize` | Schätzt den gesamten Speicherbedarf der Operation in Bytes (Overhead + Nutzdaten). | `src/tx_buffer.rs` |
| `pub fn get_ops(&self, tx: TxId) -> Option<Vec<IndexOp<T>>>` | Liefert eine Kopie der gepufferten Operationen einer Transaktion. | `src/tx_buffer.rs` |
| `pub fn get_read_set(&self, tx: TxId) -> Option<ReadSet>` | Liefert eine Kopie des angesammelten [`ReadSet`] für Transaktion `tx`. | `src/tx_buffer.rs` |
| `pub fn has_tx(&self, tx: TxId) -> bool` | Prüft, ob die gegebene Transaktion im Puffer registriert ist. | `src/tx_buffer.rs` |
| `pub fn is_empty(&self) -> bool` | Gibt `true` zurück, wenn in keinem einzigen Shard gepufferte Transaktionen liegen. | `src/tx_buffer.rs` |
| `pub fn is_key_staged_for_tx(&self, tx_id: TxId, key: &[u8]) -> bool` | Prüft, ob ein Schlüssel `key` innerhalb der Transaktion `tx_id` als Einfügung vorgemerkt ist. | `src/tx_buffer.rs` |
| `pub fn len(&self) -> usize` | Liefert die Gesamtzahl aller aktuell gepufferten Transaktionen über alle Shards. | `src/tx_buffer.rs` |
| `pub fn min_read_snapshot(&self) -> Option<u64>` | Findet die kleinste Snapshot-Sequenznummer über alle aktiven ReadSets aller Shards hinweg. Dient der Bestimmung des globalen GC-Schwellenwe… | `src/tx_buffer.rs` |
| `pub fn new() -> Self` | Erstellt einen neuen Puffer mit 64 Shards, 30 Sekunden Timeout und [`SystemClock`]. | `src/tx_buffer.rs` |
| `pub fn new_with_clock_and_config( clock: Arc<dyn Clock>, shard_count: usize, config: TxBufferConfig, ) -> Self` | Erstellt einen Puffer mit benutzerdefinierter [`Clock`], Shard-Anzahl und Konfiguration. | `src/tx_buffer.rs` |
| `pub fn new_with_config(shard_count: usize, tx_timeout: Duration) -> Self` | Erstellt einen neuen Puffer mit konfigurierter Shard-Anzahl und Transaktions-Timeout. | `src/tx_buffer.rs` |
| `pub fn new_with_config_ext( shard_count: usize, _tx_timeout: Duration, config: TxBufferConfig, ) -> Self` | Erstellt einen neuen Puffer mit expliziter Konfiguration ([`TxBufferConfig`]). | `src/tx_buffer.rs` |
| `pub fn read_set(&self, tx: TxId) -> Option<ReadSet>` | Alias-Funktion für [`TxBuffer::get_read_set`]. | `src/tx_buffer.rs` |
| `pub fn reap_orphans(&self) -> Vec<TxId>` | Räumt verwaiste Transaktionen unter Verwendung der aktuellen Uhrzeit ab. | `src/tx_buffer.rs` |
| `pub fn reap_orphans_at(&self, now_nanos: u64) -> Vec<TxId>` | Räumt abgelaufene (verwaiste) Transaktionen bezüglich `now_nanos` über alle Shards ab. | `src/tx_buffer.rs` |
| `pub fn reap_orphans_bounded(&self, max: usize) -> Vec<TxId>` | Räumt bis zu `max` verwaiste Transaktionen bezogen auf die aktuelle Uhrzeit ab. | `src/tx_buffer.rs` |
| `pub fn reap_orphans_bounded_at(&self, max: usize, now_nanos: u64) -> Vec<TxId>` | Räumt maximal `max` verwaiste Transaktionen ab, um Latenzspitzen zu vermeiden. ### Ablauf: 1. Iteriere aufsteigend über die Shards (0 bis $… | `src/tx_buffer.rs` |
| `pub fn record_read(&self, tx: TxId, key: impl Into<Vec<u8>>, snapshot_seq: u64)` | Alias-Funktion für [`TxBuffer::register_read`]. | `src/tx_buffer.rs` |
| `pub fn register_read(&self, tx: TxId, key: impl Into<Vec<u8>>, snapshot_seq: u64)` | Registriert einen Lesezugriff im ReadSet der Transaktion `tx`. | `src/tx_buffer.rs` |
| `pub fn stage(&self, tx: TxId, op: IndexOp<T>) -> Result<()> where T: StagedOpSize,` | Puffert eine Indexoperation für die Transaktion `tx` unter Prüfung aller Speichergrenzen. | `src/tx_buffer.rs` |
| `pub fn stage_bounded(&self, tx: TxId, op: IndexOp<T>) -> Result<()> where T: StagedOpSize,` | Puffert eine Operation und erzwingt Obergrenzen für Operationsanzahl und Byte-Budgets. ### Ablauf: 1. Berechne geschätzten Speicherbedarf d… | `src/tx_buffer.rs` |
| `pub fn stage_kv(&self, tx: TxId, op: IndexOp<(Vec<u8>, Vec<u8>)>) -> Result<()>` | Puffert eine Schlüssel-Wert-Operation und aktualisiert die Key-Staging-Map. Bei Fehlschlag von `stage()` (z. B. Budget-Überlauf) wird das K… | `src/tx_buffer.rs` |
| `pub fn stage_many(&self, tx: TxId, ops: impl IntoIterator<Item = IndexOp<T>>) -> Result<()> where T: StagedOpSize,` | Puffert mehrere Operationen stapelweise mit einer einzigen Sperrung des Shards. | `src/tx_buffer.rs` |
| `pub fn staged_bytes(&self) -> usize` | Liefert den aktuell global über alle Transaktionen verbrauchten Staging-Speicher in Bytes. | `src/tx_buffer.rs` |
| `pub fn staged_status(&self, key: &[u8]) -> Option<bool>` | Ermittelt atomar den Staging-Status eines Schlüssels über alle uncommitted Transaktionen. Liefert `Some(true)` bei Einfügung, `Some(false)`… | `src/tx_buffer.rs` |
| `pub fn try_register_read( &self, tx: TxId, key: impl Into<Vec<u8>>, snapshot_seq: u64, max_keys: usize, ) -> Result<()>` | Registriert einen gelesenen Schlüssel für Transaktion `tx` mit Obergrenzen-Prüfung (`max_keys`). Verhindert unbegrenztes Speicherwachstum b… | `src/tx_buffer.rs` |
| `pub fn tx_timeout(&self) -> Duration` | Liefert die konfigurierte Transaktions-Timeout-Dauer. | `src/tx_buffer.rs` |
| `pub fn validate_pending_ops(&self, tx: TxId) -> Result<()>` | Prüft, ob für eine registrierte Transaktion tatsächlich anhängige Operationen existieren. | `src/tx_buffer.rs` |
| `pub fn with_clock(mut self, clock: Arc<dyn Clock>) -> Self` | Bindet eine benutzerdefinierte [`Clock`]-Instanz an diesen Puffer an. | `src/tx_buffer.rs` |


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

# Crate `contextra-wire`  **Ring & Klasse:** Ring 0 (Unsafe-Insel 3 von 3), Klasse Kern `[K]`.  ## Zweck `contextra-wire` stellt performante Zero-Copy-Serialisierungs- und Deserialisierungs-Primitiven für die Interprozesskommunikation (IPC = Inter-Process Communication) bereit. Es kapselt: 1. Generierten FlatBuffers-Code (`contextra_generated`), der direkt auf Byte-Puffern operiert. 2. Gehärtete Verifikationsfunktionen (`verification`), die Puffer vor dem Zugriff strikt validieren. 3. Zero-Copy-Adapter (`adapter`), wie den referenzgezählten `WireBuffer`. 4. Einen robusten, angriffssicheren JSON-RPC-2.0-Parser (`jsonrpc`) ohne Panics.  ## Einbindung und Abhängigkeiten - **Abhängigkeiten (normal):** `bytes`, `flatbuffers`, `serde`, `serde_json`. - **Wird eingebunden von:** `contextra-core`, `contextra-mcp`, `contextra-py`, `contextra-router`. - **Ring-0-Restriktionen:** Gemäß Regel 3 aus `CLAUDE.md` dürfen Zeit, Zufall und IDs nicht direkt erzeugt werden; I/O erfolgt über Ports oder übergebene Puffer.  ## Sicherheitsinvariante & Zero-Panic-Garantie Die Crate erzwingt `#![deny(clippy::unwrap_used)]`, `#![deny(clippy::expect_used)]` und `#![deny(clippy::panic)]`. Fehler werden konsequent über `Result` gemeldet (Fail-Closed).

### Modul- und Re-Export-Oberfläche (lib.rs)

```rust
pub mod generated;
pub mod adapter;
pub mod jsonrpc;
pub mod verification;
pub use adapter::*;
pub use generated::contextra_generated;
pub use generated::contextra_generated::contextra::ipc::*;
pub use jsonrpc::*;
pub use verification::*;
```

### Öffentliche Datentypen — Structs (30)

| Struct | Felder / Form | Beschreibung | Datei |
|---|---|---|---|
| `Embedding` | `{ pub _tab: flatbuffers::Table<'a>, }` | Öffentliches FlatBuffers-Item `Embedding` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `EmbeddingArgs` | `{ pub data: Option<flatbuffers::WIPOffset<flatbuffers::Vector<'a, f32>>>, pub quantized_u8: Option<flatbuffers::WIPOffset<flatbuffers::Vector<'a, u8>>>, pub metr… }` | Öffentliches FlatBuffers-Item `EmbeddingArgs` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `EmbeddingBuilder` | `{ fbb_: &'b mut flatbuffers::FlatBufferBuilder<'a>, start_: flatbuffers::WIPOffset<flatbuffers::TableUnfinishedWIPOffset>, }` | Öffentliches FlatBuffers-Item `EmbeddingBuilder` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `FlatBufferConfig` | `{ /// Maximum allowed buffer size in bytes (default: 10 MiB). /// Maximal erlaubte Puffergröße in Bytes (Standard: 10 MiB). pub max_message_size: usize, /// Maxi… }` | Configuration parameters for FlatBuffers verification. Definiert Schutzgrenzen gegen Denial-of-Service-Angriffe (DoS) durch übergroße oder tief gesch… | `src/verification.rs` |
| `HyperEdge` | `{ pub _tab: flatbuffers::Table<'a>, }` | Öffentliches FlatBuffers-Item `HyperEdge` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `HyperEdgeArgs` | `{ pub id: u64, pub predicate: u8, pub participants: Option< flatbuffers::WIPOffset< flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<RoleBinding<'a>>>, >, >,… }` | Öffentliches FlatBuffers-Item `HyperEdgeArgs` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `HyperEdgeBuilder` | `{ fbb_: &'b mut flatbuffers::FlatBufferBuilder<'a>, start_: flatbuffers::WIPOffset<flatbuffers::TableUnfinishedWIPOffset>, }` | Öffentliches FlatBuffers-Item `HyperEdgeBuilder` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `HyperEdgeId` | `{ pub _tab: flatbuffers::Table<'a>, }` | Öffentliches FlatBuffers-Item `HyperEdgeId` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `HyperEdgeIdArgs` | `{ pub id: u64, }` | Öffentliches FlatBuffers-Item `HyperEdgeIdArgs` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `HyperEdgeIdBuilder` | `{ fbb_: &'b mut flatbuffers::FlatBufferBuilder<'a>, start_: flatbuffers::WIPOffset<flatbuffers::TableUnfinishedWIPOffset>, }` | Öffentliches FlatBuffers-Item `HyperEdgeIdBuilder` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `JsonRpcConfig` | `{ /// Maximum allowed payload size in bytes (default: 10 MiB). /// Maximal erlaubte Nachrichtengröße in Bytes (Standard: 10 MiB). pub max_message_size: usize, //… }` | Configurable limits and security parameters for JSON-RPC message processing. Konfigurierbare Obergrenzen und Sicherheitsparameter für die JSON-RPC-Ve… | `src/jsonrpc.rs` |
| `JsonRpcError` | `{ /// Numeric error code per JSON-RPC 2.0 specification. /// Numerischer standardisierter Fehlercode gemäß JSON-RPC 2.0 Spezifikation. pub code: i32, /// Brief e… }` | JSON-RPC 2.0 Error object. Standardisiertes JSON-RPC 2.0 Fehlerobjekt. | `src/jsonrpc.rs` |
| `JsonRpcRequest` | `{ /// Always "2.0". /// Protokollversion: Muss strikt `"2.0"` sein. pub jsonrpc: String, /// `None` for notifications (no response expected). /// Anforderungs-ID… }` | Incoming JSON-RPC 2.0 message (Request or Notification). Eingehende JSON-RPC 2.0-Nachricht (Anfrage oder Benachrichtigung). Handelt es sich um eine B… | `src/jsonrpc.rs` |
| `JsonRpcResponse` | `{ /// Always "2.0". /// Protokollversion: Immer `"2.0"`. pub jsonrpc: String, /// Request ID (`None` for parse/batch errors). /// Zugehörige Anforderungs-ID der… }` | Outgoing JSON-RPC 2.0 response. Ausgehende JSON-RPC 2.0-Antwort (Erfolg oder Fehler). | `src/jsonrpc.rs` |
| `RoleBinding` | `{ pub _tab: flatbuffers::Table<'a>, }` | Öffentliches FlatBuffers-Item `RoleBinding` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `RoleBindingArgs` | `{ pub role: u32, pub entity: u64, }` | Öffentliches FlatBuffers-Item `RoleBindingArgs` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `RoleBindingBuilder` | `{ fbb_: &'b mut flatbuffers::FlatBufferBuilder<'a>, start_: flatbuffers::WIPOffset<flatbuffers::TableUnfinishedWIPOffset>, }` | Öffentliches FlatBuffers-Item `RoleBindingBuilder` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `RoleId` | `{ pub _tab: flatbuffers::Table<'a>, }` | Öffentliches FlatBuffers-Item `RoleId` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `RoleIdArgs` | `{ pub id: u32, }` | Öffentliches FlatBuffers-Item `RoleIdArgs` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `RoleIdBuilder` | `{ fbb_: &'b mut flatbuffers::FlatBufferBuilder<'a>, start_: flatbuffers::WIPOffset<flatbuffers::TableUnfinishedWIPOffset>, }` | Öffentliches FlatBuffers-Item `RoleIdBuilder` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `ScoredDocument` | `{ pub _tab: flatbuffers::Table<'a>, }` | Öffentliches FlatBuffers-Item `ScoredDocument` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `ScoredDocumentArgs` | `{ pub id: Option<flatbuffers::WIPOffset<&'a str>>, pub score: f32, pub metadata: Option<flatbuffers::WIPOffset<&'a str>>, pub embedding: Option<flatbuffers::WIPO… }` | Öffentliches FlatBuffers-Item `ScoredDocumentArgs` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `ScoredDocumentBuilder` | `{ fbb_: &'b mut flatbuffers::FlatBufferBuilder<'a>, start_: flatbuffers::WIPOffset<flatbuffers::TableUnfinishedWIPOffset>, }` | Öffentliches FlatBuffers-Item `ScoredDocumentBuilder` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `SearchResponse` | `{ pub _tab: flatbuffers::Table<'a>, }` | Öffentliches FlatBuffers-Item `SearchResponse` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `SearchResponseArgs` | `{ pub results: Option< flatbuffers::WIPOffset< flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ScoredDocument<'a>>>, >, >, pub total_hits: u32, pub processi… }` | Öffentliches FlatBuffers-Item `SearchResponseArgs` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `SearchResponseBuilder` | `{ fbb_: &'b mut flatbuffers::FlatBufferBuilder<'a>, start_: flatbuffers::WIPOffset<flatbuffers::TableUnfinishedWIPOffset>, }` | Öffentliches FlatBuffers-Item `SearchResponseBuilder` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `VectorIndexUpdate` | `{ pub _tab: flatbuffers::Table<'a>, }` | Öffentliches FlatBuffers-Item `VectorIndexUpdate` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `VectorIndexUpdateArgs` | `{ pub id: Option<flatbuffers::WIPOffset<&'a str>>, pub embedding: Option<flatbuffers::WIPOffset<Embedding<'a>>>, pub metadata: Option<flatbuffers::WIPOffset<&'a… }` | Öffentliches FlatBuffers-Item `VectorIndexUpdateArgs` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `VectorIndexUpdateBuilder` | `{ fbb_: &'b mut flatbuffers::FlatBufferBuilder<'a>, start_: flatbuffers::WIPOffset<flatbuffers::TableUnfinishedWIPOffset>, }` | Öffentliches FlatBuffers-Item `VectorIndexUpdateBuilder` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `WireBuffer` | `{ data: Bytes, }` | Zero-Copy Byte-Puffer-Wrapper für FlatBuffers-Nachrichten. Kapselt eine Instanz von `bytes::Bytes`, um ein sicheres, referenzgezähltes Weiterreichen… | `src/adapter.rs` |

### Öffentliche Datentypen — Enums (10)

| Enum | Varianten | Beschreibung | Datei |
|---|---|---|---|
| `EmbeddingOffset` | `` | Öffentliches FlatBuffers-Item `EmbeddingOffset` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `FlatBufferError` | `/// Buffer is empty. /// Der übergebene Byte-Puffer ist leer. EmptyBuffer, /// Buffer exceeds configured maximum message size. /// Der Puffer überschreitet die maximal erlaubte Nachrichtengröße (`max…` | Error type for FlatBuffers verification failures. Signalisiert Verletzungen der Spezifikation oder Sicherheitsgrenzen beim Einlesen von Byte-Puffern. | `src/verification.rs` |
| `HyperEdgeIdOffset` | `` | Öffentliches FlatBuffers-Item `HyperEdgeIdOffset` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `HyperEdgeOffset` | `` | Öffentliches FlatBuffers-Item `HyperEdgeOffset` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `JsonRpcPayload` | `/// Single JSON-RPC request. /// Einzelne JSON-RPC-Anfrage (`JsonRpcRequest`). Single(JsonRpcRequest), /// Batch of JSON-RPC requests. /// Batch-Array mehrerer JSON-RPC-Anfragen (`Vec<JsonRpcRequest>…` | Parsed JSON-RPC payload representing either a single request or a batch of requests. Geparste JSON-RPC-Payload: Repräsentiert entweder eine Einzelanf… | `src/jsonrpc.rs` |
| `RoleBindingOffset` | `` | Öffentliches FlatBuffers-Item `RoleBindingOffset` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `RoleIdOffset` | `` | Öffentliches FlatBuffers-Item `RoleIdOffset` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `ScoredDocumentOffset` | `` | Öffentliches FlatBuffers-Item `ScoredDocumentOffset` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `SearchResponseOffset` | `` | Öffentliches FlatBuffers-Item `SearchResponseOffset` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `VectorIndexUpdateOffset` | `` | Öffentliches FlatBuffers-Item `VectorIndexUpdateOffset` für IPC-Nachrichten. | `src/contextra_generated.rs` |

### Konstanten (27)

| Konstante | Typ | Beschreibung |
|---|---|---|
| `VT_BUSINESS_VALID_FROM` | `flatbuffers::VOffsetT` | Öffentliches FlatBuffers-Item `VT_BUSINESS_VALID_FROM` für IPC-Nachrichten. |
| `VT_BUSINESS_VALID_TO` | `flatbuffers::VOffsetT` | Öffentliches FlatBuffers-Item `VT_BUSINESS_VALID_TO` für IPC-Nachrichten. |
| `VT_CHILD_EDGE_IDS` | `flatbuffers::VOffsetT` | Öffentliches FlatBuffers-Item `VT_CHILD_EDGE_IDS` für IPC-Nachrichten. |
| `VT_DATA` | `flatbuffers::VOffsetT` | Öffentliches FlatBuffers-Item `VT_DATA` für IPC-Nachrichten. |
| `VT_EMBEDDING` | `flatbuffers::VOffsetT` | Öffentliches FlatBuffers-Item `VT_EMBEDDING` für IPC-Nachrichten. |
| `VT_EMBEDDING` | `flatbuffers::VOffsetT` | Öffentliches FlatBuffers-Item `VT_EMBEDDING` für IPC-Nachrichten. |
| `VT_ENTITY` | `flatbuffers::VOffsetT` | Öffentliches FlatBuffers-Item `VT_ENTITY` für IPC-Nachrichten. |
| `VT_ID` | `flatbuffers::VOffsetT` | Öffentliches FlatBuffers-Item `VT_ID` für IPC-Nachrichten. |
| `VT_ID` | `flatbuffers::VOffsetT` | Öffentliches FlatBuffers-Item `VT_ID` für IPC-Nachrichten. |
| `VT_ID` | `flatbuffers::VOffsetT` | Öffentliches FlatBuffers-Item `VT_ID` für IPC-Nachrichten. |
| `VT_ID` | `flatbuffers::VOffsetT` | Öffentliches FlatBuffers-Item `VT_ID` für IPC-Nachrichten. |
| `VT_ID` | `flatbuffers::VOffsetT` | Öffentliches FlatBuffers-Item `VT_ID` für IPC-Nachrichten. |
| `VT_METADATA` | `flatbuffers::VOffsetT` | Öffentliches FlatBuffers-Item `VT_METADATA` für IPC-Nachrichten. |
| `VT_METADATA` | `flatbuffers::VOffsetT` | Öffentliches FlatBuffers-Item `VT_METADATA` für IPC-Nachrichten. |
| `VT_METRIC` | `flatbuffers::VOffsetT` | Öffentliches FlatBuffers-Item `VT_METRIC` für IPC-Nachrichten. |
| `VT_PARTICIPANTS` | `flatbuffers::VOffsetT` | Öffentliches FlatBuffers-Item `VT_PARTICIPANTS` für IPC-Nachrichten. |
| `VT_PREDICATE` | `flatbuffers::VOffsetT` | Öffentliches FlatBuffers-Item `VT_PREDICATE` für IPC-Nachrichten. |
| `VT_PROCESSING_TIME_MS` | `flatbuffers::VOffsetT` | Öffentliches FlatBuffers-Item `VT_PROCESSING_TIME_MS` für IPC-Nachrichten. |
| `VT_QUANTIZED_U8` | `flatbuffers::VOffsetT` | Öffentliches FlatBuffers-Item `VT_QUANTIZED_U8` für IPC-Nachrichten. |
| `VT_RESULTS` | `flatbuffers::VOffsetT` | Öffentliches FlatBuffers-Item `VT_RESULTS` für IPC-Nachrichten. |
| `VT_ROLE` | `flatbuffers::VOffsetT` | Öffentliches FlatBuffers-Item `VT_ROLE` für IPC-Nachrichten. |
| `VT_SCORE` | `flatbuffers::VOffsetT` | Öffentliches FlatBuffers-Item `VT_SCORE` für IPC-Nachrichten. |
| `VT_SOURCE_DOC_ID` | `flatbuffers::VOffsetT` | Öffentliches FlatBuffers-Item `VT_SOURCE_DOC_ID` für IPC-Nachrichten. |
| `VT_TOTAL_HITS` | `flatbuffers::VOffsetT` | Öffentliches FlatBuffers-Item `VT_TOTAL_HITS` für IPC-Nachrichten. |
| `VT_TX_VALID_FROM` | `flatbuffers::VOffsetT` | Öffentliches FlatBuffers-Item `VT_TX_VALID_FROM` für IPC-Nachrichten. |
| `VT_TX_VALID_TO` | `flatbuffers::VOffsetT` | Öffentliches FlatBuffers-Item `VT_TX_VALID_TO` für IPC-Nachrichten. |
| `VT_WEIGHT` | `flatbuffers::VOffsetT` | Öffentliches FlatBuffers-Item `VT_WEIGHT` für IPC-Nachrichten. |

### Öffentliche Funktionen (116)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn as_slice(&self) -> &[u8]` | Gibt eine unveränderliche Slice-Referenz auf die zugrundeliegenden Rohdaten zurück. # Ablauf 1. Zugriff auf den internen Byte-Puffer. 2. Rü… | `src/adapter.rs` |
| `pub fn into_inner(self) -> Bytes` | Konsumiert den Wrapper und gibt das zugrundeliegende `Bytes`-Objekt frei. # Ablauf 1. Entpacken des internen `data`-Felds. 2. Rückgabe des… | `src/adapter.rs` |
| `pub fn new(data: Bytes) -> Self` | Erzeugt einen neuen `WireBuffer`, der das übergebene `Bytes`-Objekt umhüllt. # Ablauf 1. Übernahme des `Bytes`-Objekts (Besitzübertragung o… | `src/adapter.rs` |
| `pub fn add_business_valid_from(&mut self, business_valid_from: i64)` | Öffentliches FlatBuffers-Item `add_business_valid_from` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn add_business_valid_to(&mut self, business_valid_to: i64)` | Öffentliches FlatBuffers-Item `add_business_valid_to` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn add_child_edge_ids( &mut self, child_edge_ids: flatbuffers::WIPOffset<flatbuffers::Vector<'b, u64>>, )` | Öffentliches FlatBuffers-Item `add_child_edge_ids` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn add_data(&mut self, data: flatbuffers::WIPOffset<flatbuffers::Vector<'b, f32>>)` | Öffentliches FlatBuffers-Item `add_data` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn add_embedding(&mut self, embedding: flatbuffers::WIPOffset<Embedding<'b>>)` | Öffentliches FlatBuffers-Item `add_embedding` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn add_embedding(&mut self, embedding: flatbuffers::WIPOffset<Embedding<'b>>)` | Öffentliches FlatBuffers-Item `add_embedding` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn add_entity(&mut self, entity: u64)` | Öffentliches FlatBuffers-Item `add_entity` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn add_id(&mut self, id: flatbuffers::WIPOffset<&'b str>)` | Öffentliches FlatBuffers-Item `add_id` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn add_id(&mut self, id: flatbuffers::WIPOffset<&'b str>)` | Öffentliches FlatBuffers-Item `add_id` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn add_id(&mut self, id: u32)` | Öffentliches FlatBuffers-Item `add_id` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn add_id(&mut self, id: u64)` | Öffentliches FlatBuffers-Item `add_id` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn add_id(&mut self, id: u64)` | Öffentliches FlatBuffers-Item `add_id` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn add_metadata(&mut self, metadata: flatbuffers::WIPOffset<&'b str>)` | Öffentliches FlatBuffers-Item `add_metadata` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn add_metadata(&mut self, metadata: flatbuffers::WIPOffset<&'b str>)` | Öffentliches FlatBuffers-Item `add_metadata` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn add_metric(&mut self, metric: i8)` | Öffentliches FlatBuffers-Item `add_metric` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn add_participants( &mut self, participants: flatbuffers::WIPOffset< flatbuffers::Vector<'b, flatbuffers::ForwardsUOffset<RoleBinding<'b>>>, >, )` | Öffentliches FlatBuffers-Item `add_participants` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn add_predicate(&mut self, predicate: u8)` | Öffentliches FlatBuffers-Item `add_predicate` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn add_processing_time_ms(&mut self, processing_time_ms: f32)` | Öffentliches FlatBuffers-Item `add_processing_time_ms` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn add_quantized_u8( &mut self, quantized_u8: flatbuffers::WIPOffset<flatbuffers::Vector<'b, u8>>, )` | Öffentliches FlatBuffers-Item `add_quantized_u8` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn add_results( &mut self, results: flatbuffers::WIPOffset< flatbuffers::Vector<'b, flatbuffers::ForwardsUOffset<ScoredDocument<'b>>>, >, )` | Öffentliches FlatBuffers-Item `add_results` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn add_role(&mut self, role: u32)` | Öffentliches FlatBuffers-Item `add_role` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn add_score(&mut self, score: f32)` | Öffentliches FlatBuffers-Item `add_score` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn add_source_doc_id(&mut self, source_doc_id: u64)` | Öffentliches FlatBuffers-Item `add_source_doc_id` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn add_total_hits(&mut self, total_hits: u32)` | Öffentliches FlatBuffers-Item `add_total_hits` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn add_tx_valid_from(&mut self, tx_valid_from: u64)` | Öffentliches FlatBuffers-Item `add_tx_valid_from` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn add_tx_valid_to(&mut self, tx_valid_to: u64)` | Öffentliches FlatBuffers-Item `add_tx_valid_to` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn add_weight(&mut self, weight: f32)` | Öffentliches FlatBuffers-Item `add_weight` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn business_valid_from(&self) -> i64` | Öffentliches FlatBuffers-Item `business_valid_from` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn business_valid_to(&self) -> i64` | Öffentliches FlatBuffers-Item `business_valid_to` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn child_edge_ids(&self) -> Option<flatbuffers::Vector<'a, u64>>` | Öffentliches FlatBuffers-Item `child_edge_ids` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>( _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr>, args: &'args EmbeddingArgs<'args>, ) -> flatbuffers::WIPOffset<Embedding<'bldr>>` | Öffentliches FlatBuffers-Item `create` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>( _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr>, args: &'args ScoredDocumentArgs<'args>, ) -> flatbuffers::WIPOffset<ScoredDocument<'bldr>>` | Öffentliches FlatBuffers-Item `create` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>( _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr>, args: &'args SearchResponseArgs<'args>, ) -> flatbuffers::WIPOffset<SearchResponse<'bldr>>` | Öffentliches FlatBuffers-Item `create` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>( _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr>, args: &'args VectorIndexUpdateArgs<'args>, ) -> flatbuffers::WIPOffset<VectorIndexUpdate<'bldr>>` | Öffentliches FlatBuffers-Item `create` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>( _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr>, args: &'args RoleIdArgs, ) -> flatbuffers::WIPOffset<RoleId<'bldr>>` | Öffentliches FlatBuffers-Item `create` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>( _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr>, args: &'args HyperEdgeIdArgs, ) -> flatbuffers::WIPOffset<HyperEdgeId<'bldr>>` | Öffentliches FlatBuffers-Item `create` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>( _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr>, args: &'args RoleBindingArgs, ) -> flatbuffers::WIPOffset<RoleBinding<'bldr>>` | Öffentliches FlatBuffers-Item `create` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>( _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr>, args: &'args HyperEdgeArgs<'args>, ) -> flatbuffers::WIPOffset<HyperEdge<'bldr>>` | Öffentliches FlatBuffers-Item `create` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn data(&self) -> Option<flatbuffers::Vector<'a, f32>>` | Öffentliches FlatBuffers-Item `data` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn embedding(&self) -> Option<Embedding<'a>>` | Öffentliches FlatBuffers-Item `embedding` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn embedding(&self) -> Option<Embedding<'a>>` | Öffentliches FlatBuffers-Item `embedding` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn entity(&self) -> u64` | Öffentliches FlatBuffers-Item `entity` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn finish(self) -> flatbuffers::WIPOffset<Embedding<'a>>` | Öffentliches FlatBuffers-Item `finish` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn finish(self) -> flatbuffers::WIPOffset<ScoredDocument<'a>>` | Öffentliches FlatBuffers-Item `finish` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn finish(self) -> flatbuffers::WIPOffset<SearchResponse<'a>>` | Öffentliches FlatBuffers-Item `finish` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn finish(self) -> flatbuffers::WIPOffset<VectorIndexUpdate<'a>>` | Öffentliches FlatBuffers-Item `finish` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn finish(self) -> flatbuffers::WIPOffset<RoleId<'a>>` | Öffentliches FlatBuffers-Item `finish` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn finish(self) -> flatbuffers::WIPOffset<HyperEdgeId<'a>>` | Öffentliches FlatBuffers-Item `finish` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn finish(self) -> flatbuffers::WIPOffset<RoleBinding<'a>>` | Öffentliches FlatBuffers-Item `finish` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn finish(self) -> flatbuffers::WIPOffset<HyperEdge<'a>>` | Öffentliches FlatBuffers-Item `finish` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn finish_search_response_buffer<'a, 'b>( fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>, root: flatbuffers::WIPOffset<SearchResponse<'a>>, )` | Öffentliches FlatBuffers-Item `finish_search_response_buffer` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn finish_size_prefixed_search_response_buffer<'a, 'b>( fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>, root: flatbuffers::WIPOffset<SearchResponse<'a>>, )` | Öffentliches FlatBuffers-Item `finish_size_prefixed_search_response_buffer` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn id(&self) -> Option<&'a str>` | Öffentliches FlatBuffers-Item `id` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn id(&self) -> Option<&'a str>` | Öffentliches FlatBuffers-Item `id` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn id(&self) -> u32` | Öffentliches FlatBuffers-Item `id` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn id(&self) -> u64` | Öffentliches FlatBuffers-Item `id` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn id(&self) -> u64` | Öffentliches FlatBuffers-Item `id` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub unsafe fn init_from_table(table: flatbuffers::Table<'a>) -> Self` | Öffentliches FlatBuffers-Item `init_from_table` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub unsafe fn init_from_table(table: flatbuffers::Table<'a>) -> Self` | Öffentliches FlatBuffers-Item `init_from_table` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub unsafe fn init_from_table(table: flatbuffers::Table<'a>) -> Self` | Öffentliches FlatBuffers-Item `init_from_table` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub unsafe fn init_from_table(table: flatbuffers::Table<'a>) -> Self` | Öffentliches FlatBuffers-Item `init_from_table` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub unsafe fn init_from_table(table: flatbuffers::Table<'a>) -> Self` | Öffentliches FlatBuffers-Item `init_from_table` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub unsafe fn init_from_table(table: flatbuffers::Table<'a>) -> Self` | Öffentliches FlatBuffers-Item `init_from_table` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub unsafe fn init_from_table(table: flatbuffers::Table<'a>) -> Self` | Öffentliches FlatBuffers-Item `init_from_table` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub unsafe fn init_from_table(table: flatbuffers::Table<'a>) -> Self` | Öffentliches FlatBuffers-Item `init_from_table` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn metadata(&self) -> Option<&'a str>` | Öffentliches FlatBuffers-Item `metadata` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn metadata(&self) -> Option<&'a str>` | Öffentliches FlatBuffers-Item `metadata` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn metric(&self) -> i8` | Öffentliches FlatBuffers-Item `metric` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn new( _fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>, ) -> EmbeddingBuilder<'a, 'b>` | Öffentliches FlatBuffers-Item `new` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn new( _fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>, ) -> ScoredDocumentBuilder<'a, 'b>` | Öffentliches FlatBuffers-Item `new` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn new( _fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>, ) -> SearchResponseBuilder<'a, 'b>` | Öffentliches FlatBuffers-Item `new` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn new( _fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>, ) -> VectorIndexUpdateBuilder<'a, 'b>` | Öffentliches FlatBuffers-Item `new` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn new(_fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>) -> RoleIdBuilder<'a, 'b>` | Öffentliches FlatBuffers-Item `new` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn new( _fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>, ) -> HyperEdgeIdBuilder<'a, 'b>` | Öffentliches FlatBuffers-Item `new` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn new( _fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>, ) -> RoleBindingBuilder<'a, 'b>` | Öffentliches FlatBuffers-Item `new` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn new( _fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>, ) -> HyperEdgeBuilder<'a, 'b>` | Öffentliches FlatBuffers-Item `new` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn participants( &self, ) -> Option<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<RoleBinding<'a>>>>` | Öffentliches FlatBuffers-Item `participants` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn predicate(&self) -> u8` | Öffentliches FlatBuffers-Item `predicate` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn processing_time_ms(&self) -> f32` | Öffentliches FlatBuffers-Item `processing_time_ms` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn quantized_u8(&self) -> Option<flatbuffers::Vector<'a, u8>>` | Öffentliches FlatBuffers-Item `quantized_u8` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn results( &self, ) -> Option<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ScoredDocument<'a>>>>` | Öffentliches FlatBuffers-Item `results` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn role(&self) -> u32` | Öffentliches FlatBuffers-Item `role` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn root_as_search_response( buf: &[u8], ) -> Result<SearchResponse, flatbuffers::InvalidFlatbuffer>` | Öffentliches FlatBuffers-Item `root_as_search_response` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub unsafe fn root_as_search_response_unchecked(buf: &[u8]) -> SearchResponse` | Öffentliches FlatBuffers-Item `root_as_search_response_unchecked` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn root_as_search_response_with_opts<'b, 'o>( opts: &'o flatbuffers::VerifierOptions, buf: &'b [u8], ) -> Result<SearchResponse<'b>, flatbuffers::InvalidFlatbuffer>` | Öffentliches FlatBuffers-Item `root_as_search_response_with_opts` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn score(&self) -> f32` | Öffentliches FlatBuffers-Item `score` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn size_prefixed_root_as_search_response( buf: &[u8], ) -> Result<SearchResponse, flatbuffers::InvalidFlatbuffer>` | Öffentliches FlatBuffers-Item `size_prefixed_root_as_search_response` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub unsafe fn size_prefixed_root_as_search_response_unchecked( buf: &[u8], ) -> SearchResponse` | Öffentliches FlatBuffers-Item `size_prefixed_root_as_search_response_unchecked` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn size_prefixed_root_as_search_response_with_opts<'b, 'o>( opts: &'o flatbuffers::VerifierOptions, buf: &'b [u8], ) -> Result<SearchResponse<'b>, flatbuffers::InvalidFlatbuffer>` | Öffentliches FlatBuffers-Item `size_prefixed_root_as_search_response_with_opts` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn source_doc_id(&self) -> u64` | Öffentliches FlatBuffers-Item `source_doc_id` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn total_hits(&self) -> u32` | Öffentliches FlatBuffers-Item `total_hits` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn tx_valid_from(&self) -> u64` | Öffentliches FlatBuffers-Item `tx_valid_from` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn tx_valid_to(&self) -> u64` | Öffentliches FlatBuffers-Item `tx_valid_to` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn weight(&self) -> f32` | Öffentliches FlatBuffers-Item `weight` für IPC-Nachrichten. | `src/contextra_generated.rs` |
| `pub fn err(id: Option<Value>, code: i32, message: impl Into<String>) -> Self` | Constructs an error JSON-RPC response. Konstruiert eine JSON-RPC 2.0-Fehlerantwort. # Parameter - `id`: ID der Anfrage (oder `None`, falls… | `src/jsonrpc.rs` |
| `pub fn internal_error(message: impl Into<String>) -> Self` | Internal error (-32603): Internal JSON-RPC error. Erzeugt einen Internal error (-32603): Interner JSON-RPC Serverfehler. | `src/jsonrpc.rs` |
| `pub fn invalid_params(message: impl Into<String>) -> Self` | Invalid params (-32602): Invalid method parameter(s). Erzeugt einen Invalid params (-32602): Ungültige Methodenparameter übergeben. | `src/jsonrpc.rs` |
| `pub fn invalid_request(message: impl Into<String>) -> Self` | Invalid Request (-32600): JSON sent is not a valid Request object. Erzeugt einen Invalid Request (-32600): Gesendetes JSON ist keine gültig… | `src/jsonrpc.rs` |
| `pub fn method_not_found(message: impl Into<String>) -> Self` | Method not found (-32601): The method does not exist / is not available. Erzeugt einen Method not found (-32601): Methode existiert nicht o… | `src/jsonrpc.rs` |
| `pub fn ok(id: Option<Value>, result: Value) -> Self` | Constructs a successful JSON-RPC response. Konstruiert eine erfolgreiche JSON-RPC 2.0-Antwort mit Ergebnisdaten. # Parameter - `id`: ID der… | `src/jsonrpc.rs` |
| `pub fn parse_error(message: impl Into<String>) -> Self` | Parse error (-32700): Invalid JSON received by server. Erzeugt einen Parse error (-32700): Ungültiges JSON empfangen. | `src/jsonrpc.rs` |
| `pub fn parse_jsonrpc_payload(raw: &[u8]) -> Result<JsonRpcPayload, JsonRpcError>` | Parses raw bytes into a `JsonRpcPayload` using default configuration limits. Parst rohe Bytes in ein `JsonRpcPayload` unter Verwendung der… | `src/jsonrpc.rs` |
| `pub fn parse_jsonrpc_payload_with_config( raw: &[u8], config: &JsonRpcConfig, ) -> Result<JsonRpcPayload, JsonRpcError>` | Parses raw bytes into a `JsonRpcPayload` with strict security and size validation. Parst rohe Bytes in ein `JsonRpcPayload` mit strikter Si… | `src/jsonrpc.rs` |
| `pub fn validate(&self) -> Result<(), JsonRpcError>` | Validates that the request complies with the JSON-RPC 2.0 specification (fail-closed). # Ablauf 1. Prüfen, ob `jsonrpc` exakt `"2.0"` entsp… | `src/jsonrpc.rs` |
| `pub fn verify_and_parse_embedding<'a>( buf: &'a [u8], config: &FlatBufferConfig, ) -> Result<Embedding<'a>, FlatBufferError>` | Verifies and parses an `Embedding` root table from bytes. # Ablauf von `verify_and_parse_embedding`: 1. Validiert Puffergrenzen und Tabelle… | `src/verification.rs` |
| `pub fn verify_and_parse_hyper_edge<'a>( buf: &'a [u8], config: &FlatBufferConfig, ) -> Result<HyperEdge<'a>, FlatBufferError>` | Verifies and parses a `HyperEdge` root table from bytes. # Ablauf von `verify_and_parse_hyper_edge`: 1. Validiert Puffergrenzen und Tabelle… | `src/verification.rs` |
| `pub fn verify_and_parse_hyper_edge_id<'a>( buf: &'a [u8], config: &FlatBufferConfig, ) -> Result<HyperEdgeId<'a>, FlatBufferError>` | Verifies and parses a `HyperEdgeId` root table from bytes. # Ablauf von `verify_and_parse_hyper_edge_id`: 1. Validiert Puffergrenzen und Ta… | `src/verification.rs` |
| `pub fn verify_and_parse_role_binding<'a>( buf: &'a [u8], config: &FlatBufferConfig, ) -> Result<RoleBinding<'a>, FlatBufferError>` | Verifies and parses a `RoleBinding` root table from bytes. # Ablauf von `verify_and_parse_role_binding`: 1. Validiert Puffergrenzen und Tab… | `src/verification.rs` |
| `pub fn verify_and_parse_role_id<'a>( buf: &'a [u8], config: &FlatBufferConfig, ) -> Result<RoleId<'a>, FlatBufferError>` | Verifies and parses a `RoleId` root table from bytes. # Ablauf von `verify_and_parse_role_id`: 1. Validiert Puffergrenzen und Tabellenstruk… | `src/verification.rs` |
| `pub fn verify_and_parse_scored_document<'a>( buf: &'a [u8], config: &FlatBufferConfig, ) -> Result<ScoredDocument<'a>, FlatBufferError>` | Verifies and parses a `ScoredDocument` root table from bytes. # Ablauf von `verify_and_parse_scored_document`: 1. Validiert Puffergrenzen u… | `src/verification.rs` |
| `pub fn verify_and_parse_search_response<'a>( buf: &'a [u8], config: &FlatBufferConfig, ) -> Result<SearchResponse<'a>, FlatBufferError>` | Verifies and parses a `SearchResponse` root table from bytes. # Ablauf von `verify_and_parse_search_response`: 1. Übergibt Puffer und Konfi… | `src/verification.rs` |
| `pub fn verify_and_parse_vector_index_update<'a>( buf: &'a [u8], config: &FlatBufferConfig, ) -> Result<VectorIndexUpdate<'a>, FlatBufferError>` | Verifies and parses a `VectorIndexUpdate` root table from bytes. # Ablauf von `verify_and_parse_vector_index_update`: 1. Validiert Puffergr… | `src/verification.rs` |
| `pub fn verify_flatbuffer<'a, T>( buf: &'a [u8], config: &FlatBufferConfig, ) -> Result<T::Inner, FlatBufferError> where T: flatbuffers::Follow<'a> + flatbuffers::Verifiable + 'a,` | Generic verified root accessor for any FlatBuffers root table `T`. # Ablauf von `verify_flatbuffer`: 1. Puffer prüfen: Ist der Puffer leer,… | `src/verification.rs` |


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
pub mod durability;
pub mod mlock;
pub mod mmap;
pub mod posix;
pub mod vault;
pub use acl_win32::{set_restrictive_file_acl, verify_file_acl_owner_only};
pub use durability::{durable_flush_dir, durable_flush_file, write_all_durable};
pub use mlock::{mem_lock, mem_lock_with_policy, mem_unlock, MemLockPolicy};
pub use mmap::mmap_readonly;
pub use posix::{ensure_fd_limit, reopen_and_dup2, set_fd_limit};
pub use vault::LockedRegions;
```

### Öffentliche Datentypen — Structs (1)

| Struct | Felder / Form | Beschreibung | Datei |
|---|---|---|---|
| `LockedRegions` | `{ regions: Vec<(usize, usize)>, }` | Kapselt und verfolgt im RAM gesperrte Speicherbereiche (Adressen und Längen) für sensible Schlüsseldaten. Beim Verlassen des Gültigkeitsbereichs ([`D… | `src/vault.rs` |

### Öffentliche Datentypen — Enums (1)

| Enum | Varianten | Beschreibung | Datei |
|---|---|---|---|
| `MemLockPolicy` | `/// Fehlschläge beim Sperren werden als fataler Fehler behandelt und brechen die Operation ab. /// Einsatz z. B. bei hochsensiblen kryptografischen Masterschlüsseln (Fail-Closed). FailClosed, /// Feh…` | Konfigurierbare Richtlinie für das Sperren von Speicherregionen (`mlock` / `VirtualLock`). | `src/mlock.rs` |

### Öffentliche Funktionen (28)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn set_restrictive_file_acl(path: &Path) -> std::io::Result<()>` | Beschränkt die Zugriffsrechte der angegebenen Datei unter Unix auf den Besitzer (`0600` / `-rw-------`). # Ablauf 1. Konvertiert den oktale… | `src/acl_win32.rs` |
| `pub fn set_restrictive_file_acl(path: &Path) -> std::io::Result<()>` | Beschränkt unter Windows die Zugriffsrechte der angegebenen Datei auf den Prozessbesitzer mittels Win32-DACL. # Ablauf 1. Ruft die Besitzer… | `src/acl_win32.rs` |
| `pub fn set_restrictive_file_acl(path: &Path) -> std::io::Result<()>` | Best-Effort-Fallback für Zielarchitekturen, die weder Unix noch Windows sind (No-Op). | `src/acl_win32.rs` |
| `pub fn verify_file_acl_owner_only(path: &Path) -> std::io::Result<()>` | Verifiziert unter Unix, dass eine Datei strikt auf den Besitzer beschränkt ist (`0600`) und dem aktuellen Prozess gehört. # Ablauf 1. Liest… | `src/acl_win32.rs` |
| `pub fn verify_file_acl_owner_only(path: &Path) -> std::io::Result<()>` | Verifiziert unter Windows, dass die Datei über eine geschützte DACL verfügt, die ausschließlich dem aktuellen Prozessbesitzer Zugriff gewäh… | `src/acl_win32.rs` |
| `pub fn verify_file_acl_owner_only(path: &Path) -> std::io::Result<()>` | Best-Effort-Fallback für Zielarchitekturen, die weder Unix noch Windows sind (No-Op). | `src/acl_win32.rs` |
| `pub fn durable_flush_dir(path: &Path) -> Result<()>` | Flasht den Verzeichniseintrag (übergeordnetes Verzeichnis) einer neu erstellten oder umbenannten Datei. # Warum ein Directory Flush notwend… | `src/durability.rs` |
| `pub fn durable_flush_file(file: &File, metadata_changed: bool) -> Result<()>` | Flasht Dateiinhalt und optional Metadaten auf den physischen Datenträger gemäß den Garantien der Zielplattform. # Ablauf 1. Extrahiert den… | `src/durability.rs` |
| `pub fn write_all_durable(file: &mut File, mut buf: &[u8]) -> Result<()>` | Schreibt die gesamten Bytes aus `buf` zuverlässig in `file`, unter robuster Behandlung partieller Schreibvorgänge und Signal-Unterbrechunge… | `src/durability.rs` |
| `pub fn mem_lock(ptr: *const u8, len: usize) -> bool` | Sperrt eine Speicherregion im RAM mit der Standard-Richtlinie [`MemLockPolicy::Tolerant`]. Gibt `true` zurück, falls das Sperren erfolgreic… | `src/mlock.rs` |
| `pub fn mem_lock_with_policy(ptr: *const u8, len: usize, policy: MemLockPolicy) -> Result<bool>` | Sperrt unter Unix eine Speicherregion im RAM, um das Auslagern (Swapping) zu verhindern. # Ablauf 1. Prüft, ob `len == 0` oder `ptr.is_null… | `src/mlock.rs` |
| `pub fn mem_lock_with_policy(ptr: *const u8, len: usize, policy: MemLockPolicy) -> Result<bool>` | Sperrt unter Windows eine Speicherregion im RAM gemäß der übergebenen [`MemLockPolicy`]. # Ablauf 1. Behandelt Nullzeiger oder Nulllänge so… | `src/mlock.rs` |
| `pub fn mem_lock_with_policy(ptr: *const u8, len: usize, policy: MemLockPolicy) -> Result<bool>` | Fallback-Implementierung für Zielarchitekturen, die weder Unix noch Windows unterstützen. # Fehlerfälle Liefert bei [`MemLockPolicy::FailCl… | `src/mlock.rs` |
| `pub fn mem_unlock(addr: usize, len: usize)` | Entsperrt eine zuvor unter Unix gesperrte Speicherregion im RAM. # Vorbedingungen - `addr` und `len` müssen exakt einer zuvor über [`mem_lo… | `src/mlock.rs` |
| `pub fn mem_unlock(addr: usize, len: usize)` | Entsperrt eine zuvor unter Windows gesperrte Speicherregion im RAM über `VirtualUnlock`. | `src/mlock.rs` |
| `pub fn mem_unlock(_addr: usize, _len: usize)` | Fallback No-Op zum Entsperren von Speicher auf nicht unterstützten Plattformen. | `src/mlock.rs` |
| `pub fn mmap_readonly(file: &File) -> io::Result<memmap2::Mmap>` | Erstellt eine unveränderliche Nur-Lese-Speicherabbildung (Read-Only Memory Map) für eine geöffnete Datei. # Ablauf 1. Übergibt die geöffnet… | `src/mmap.rs` |
| `pub fn ensure_fd_limit(min_fds: u64) -> bool` | Stellt sicher, dass das weiche Limit für offene Dateideskriptoren (`RLIMIT_NOFILE`) mindestens `min_fds` beträgt. # Ablauf 1. Liest das akt… | `src/posix.rs` |
| `pub fn reopen_and_dup2(path: &Path, target_fd: i32, flags: i32) -> std::io::Result<()>` | Öffnet den Pfad `path` mit den angegebenen POSIX-Flags (z. B. `O_RDONLY`) und ersetzt atomar `target_fd` via `dup2`. # Ablauf 1. Konvertier… | `src/posix.rs` |
| `pub fn reopen_and_dup2(_path: &Path, _target_fd: i32, _flags: i32) -> std::io::Result<()>` | Fallback für Nicht-Linux-Systeme, auf denen `reopen_and_dup2` nicht unterstützt wird. # Fehlerfälle Liefert stets [`std::io::ErrorKind::Uns… | `src/posix.rs` |
| `pub fn set_fd_limit(limit: u64) -> bool` | Setzt das weiche und harte Dateideskriptorlimit (`RLIMIT_NOFILE`) des aktuellen Prozesses exakt auf `limit`. # Ablauf 1. Initialisiert eine… | `src/posix.rs` |
| `pub fn is_empty(&self) -> bool` | Gibt `true` zurück, wenn aktuell keine gesperrten Speicherregionen verwaltet werden. | `src/vault.rs` |
| `pub fn len(&self) -> usize` | Gibt die Anzahl der aktuell nachverfolgten, gesperrten Regionen zurück. | `src/vault.rs` |
| `pub fn lock_slice(&mut self, slice: &[u8], attempt_mlock: bool) -> bool` | Versucht einen Byte-Slice im RAM zu sperren, falls `attempt_mlock` wahr ist, unter Verwendung der Standardrichtlinie [`MemLockPolicy::Toler… | `src/vault.rs` |
| `pub fn lock_slice_with_policy( &mut self, slice: &[u8], attempt_mlock: bool, policy: MemLockPolicy, ) -> std::io::Result<bool>` | Versucht einen Byte-Slice im RAM gemäß der angegebenen [`MemLockPolicy`] zu sperren, falls `attempt_mlock == true`. # Ablauf 1. Ist `slice.… | `src/vault.rs` |
| `pub fn new() -> Self` | Erstellt eine neue, leere Nachverfolgungsinstanz für gesperrte Speicherregionen. | `src/vault.rs` |
| `pub fn unlock_all(&mut self)` | Entsperrt alle registrierten Speicherregionen und leert die interne Nachverfolgungsliste. | `src/vault.rs` |
| `pub fn zeroize_slice(slice: &mut [u8])` | Überschreibt einen veränderbaren Byte-Slice sicher mit Nullen unter Verwendung von flüchtigen Schreibzugriffen (`write_volatile`). # Warum… | `src/vault.rs` |


---

## `contextra-simd`

| Attribut | Wert |
|---|---|
| **Ring** | 0 — Ring 0 — Foundation & Pure Numerics/Types |
| **Status** | Fertig |
| **Zweck** | AVX2/AVX-512/NEON SIMD-Distanzberechnungskerne mit Laufzeit-Dispatch (Unsafe-Insel). |
| **Unsafe-Code** | Ja (deklarierte Unsafe-Insel) |
| **Interne Abhängigkeiten** | `contextra-types` |
| **Externe Abhängigkeiten (Auswahl)** | `bytemuck`, `thiserror`, `tracing` |

### Architekturbeschreibung

# Crate `contextra-simd`  **Ring-Zugehörigkeit:** Ring 0 (Kern, unsafe-Insel).  **Zweck:** Diese Crate stellt hochoptimierte Vektor-Distanzberechnungen bereit. SIMD (Single Instruction, Multiple Data) nutzt spezielle Prozessorregister (z. B. 128-Bit bei ARM NEON, 256-Bit bei x86 AVX2, 512-Bit bei x86 AVX-512), um mehrere Gleitkomma- oder Ganzzahl-Operationen parallel mit einem einzigen CPU-Befehl auszuführen.  **Verbindung im System:** - **Abhängigkeiten (normal):** `contextra-types` (für `ContextraError` und `DistanceMetric`), `bytemuck`, `thiserror`, `tracing`. - **Wird eingebunden von:** `contextra-vector` (Ring 1) zur Beschleunigung von Ähnlichkeitssuchen in Vektorindizes (z. B. HNSW = Hierarchical Navigable Small World).  **Architektur:** 1. `kernels/`: Enthält isolierte Implementierungen für Skalar (Referenz), AVX2, AVX-512 und ARM NEON. 2. `dispatch.rs`: Ermittelt beim Start dynamisch die CPU-Fähigkeiten und wählt den schnellsten sicheren Pfad. 3. `lib.rs`: Bietet die sichere öffentliche Schnittstelle mit Vorab-Validierung gegen ungültige Werte (NaN/Unendlich).

### Modul- und Re-Export-Oberfläche (lib.rs)

```rust
pub mod dispatch;
pub mod kernels;
pub use dispatch::*;
pub use kernels::scalar::{ cosine_distance_f32_bytes_scalar, cosine_distance_scalar, cosine_similarity_parts_f32_u8, cosine_similarity_parts_u8_scalar, dot_product_f32_bytes_scalar, dot_product_f32_u8, dot_product_scalar, dot_product_u8_scalar, euclidean_distance_f32_bytes_scalar, euclidean_distance_scalar, euclidean_distance_sq_f32_u8, euclidean_distance_sq_u8_scalar, normalize_inplace, CosineSimilarityPartsF32U8, CosineSimilarityPartsU8, };
```

### Öffentliche Datentypen — Structs (2)

| Struct | Felder / Form | Beschreibung | Datei |
|---|---|---|---|
| `CosineSimilarityPartsF32U8` | `{ /// Accumulated dot product $\sum a_i (b_i \text{ as } f32)$. pub dot: f32, /// Accumulated squared L2 norm $\sum a_i^2$. pub norm_a_sq: f32, /// Accumulated s… }` | Struct holding raw float sum components for f32/u8 mixed cosine similarity calculation. | `src/kernels/scalar.rs` |
| `CosineSimilarityPartsU8` | `{ /// Accumulated dot product $\sum a_i b_i$. pub dot: u32, /// Accumulated squared L2 norm $\sum a_i^2$. pub norm_a_sq: u32, /// Accumulated squared L2 norm $\s… }` | Struct holding raw integer sum components for u8 cosine similarity calculation. | `src/kernels/scalar.rs` |

### Öffentliche Datentypen — Enums (1)

| Enum | Varianten | Beschreibung | Datei |
|---|---|---|---|
| `SimdPathOverride` | `/// Force scalar fallback execution. Scalar, /// Force AVX2 SIMD execution path. Avx2, /// Force AVX-512 SIMD execution path. Avx512, /// Force ARM NEON SIMD execution path. Neon,` | SIMD path override options for testing purposes. | `src/dispatch.rs` |

### Öffentliche Funktionen (28)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn cosine_distance(a: &[f32], b: &[f32]) -> Result<f32, ContextraError>` | Computes cosine distance ($1.0 - \text{cosine\_similarity}$) between two `f32` vectors. Dynamically dispatches to AVX-512, AVX2+FMA, or NEO… | `src/dispatch.rs` |
| `pub fn cosine_distance_f32_bytes(a: &[f32], b_bytes: &[u8]) -> Result<f32, ContextraError>` | Computes cosine distance between an `f32` slice and raw little-endian `f32` byte slice. Dynamically dispatches to AVX-512 or AVX2+FMA based… | `src/dispatch.rs` |
| `pub fn cosine_similarity_parts_u8( a: &[u8], b: &[u8], ) -> Result<CosineSimilarityPartsU8, ContextraError>` | Computes cosine similarity component parts (dot product, norm_a_sq, norm_b_sq) for two `u8` byte slices. Dynamically dispatches to AVX-512… | `src/dispatch.rs` |
| `pub fn dot_product_distance(a: &[f32], b: &[f32]) -> Result<f32, ContextraError>` | Computes dot product distance (negated dot product $-a \cdot b$) between two `f32` vectors. Dynamically dispatches to AVX-512, AVX2+FMA, or… | `src/dispatch.rs` |
| `pub fn dot_product_distance_f32_bytes(a: &[f32], b_bytes: &[u8]) -> Result<f32, ContextraError>` | Computes dot product distance between an `f32` slice and raw little-endian `f32` byte slice. Dynamically dispatches to AVX-512 or AVX2+FMA… | `src/dispatch.rs` |
| `pub fn dot_product_u8(a: &[u8], b: &[u8]) -> Result<u32, ContextraError>` | Computes integer dot product between two `u8` byte slices. Dynamically dispatches to AVX-512 VNNI (`dpbusd`) or AVX2 (`madd_epi16`), fallin… | `src/dispatch.rs` |
| `pub fn euclidean_distance(a: &[f32], b: &[f32]) -> Result<f32, ContextraError>` | Computes Euclidean distance ($\|a - b\|_2$) between two `f32` vectors. Dynamically dispatches to AVX-512, AVX2+FMA, or NEON based on runtim… | `src/dispatch.rs` |
| `pub fn euclidean_distance_f32_bytes(a: &[f32], b_bytes: &[u8]) -> Result<f32, ContextraError>` | Computes Euclidean distance between an `f32` slice and raw little-endian `f32` byte slice. Dynamically dispatches to AVX-512 or AVX2+FMA ba… | `src/dispatch.rs` |
| `pub fn euclidean_distance_sq_u8(a: &[u8], b: &[u8]) -> Result<u32, ContextraError>` | Computes squared Euclidean distance ($\|a - b\|_2^2$) between two `u8` byte slices. Dynamically dispatches to AVX-512 or AVX2 SIMD implemen… | `src/dispatch.rs` |
| `pub fn get_dispatch_override() -> Option<SimdPathOverride>` | Retrieves the current thread-local SIMD dispatch path override. | `src/dispatch.rs` |
| `pub fn set_dispatch_override(path: Option<SimdPathOverride>)` | Sets thread-local SIMD dispatch path override for testing. | `src/dispatch.rs` |
| `pub fn cosine_distance_f32_bytes_scalar(a: &[f32], b_bytes: &[u8]) -> f32` | Scalar reference implementation for cosine distance between `f32` slice and raw LE `f32` byte slice. # Semantics Decodes `f32` values from… | `src/kernels/scalar.rs` |
| `pub fn cosine_distance_scalar(a: &[f32], b: &[f32]) -> f32` | Scalar reference implementation for cosine distance on `f32` slices. # Semantics Calculates $1.0 - \text{cosine\_similarity}(a, b)$. Cosine… | `src/kernels/scalar.rs` |
| `pub fn cosine_similarity_parts_f32_u8(a: &[f32], b: &[u8]) -> CosineSimilarityPartsF32U8` | Scalar reference implementation for mixed `f32`/`u8` cosine similarity component parts. # Semantics Returns [`CosineSimilarityPartsF32U8`]… | `src/kernels/scalar.rs` |
| `pub fn cosine_similarity_parts_u8_scalar(a: &[u8], b: &[u8]) -> CosineSimilarityPartsU8` | Scalar reference implementation for `u8` cosine similarity component parts. # Semantics Returns [`CosineSimilarityPartsU8`] containing dot… | `src/kernels/scalar.rs` |
| `pub fn dot_product_f32_bytes_scalar(a: &[f32], b_bytes: &[u8]) -> f32` | Scalar reference implementation for dot product distance between `f32` slice and raw LE `f32` byte slice. # Semantics Decodes `f32` values… | `src/kernels/scalar.rs` |
| `pub fn dot_product_f32_u8(a: &[f32], b: &[u8]) -> f32` | Scalar reference implementation for mixed `f32`/`u8` dot product. # Semantics Calculates $\sum a_i (b_i \text{ as } f32)$. # Complexity $O(… | `src/kernels/scalar.rs` |
| `pub fn dot_product_scalar(a: &[f32], b: &[f32]) -> f32` | Scalar reference implementation for dot product distance on `f32` slices. # Semantics Calculates negated dot product $- \sum a_i b_i$. # Co… | `src/kernels/scalar.rs` |
| `pub fn dot_product_u8_scalar(a: &[u8], b: &[u8]) -> u32` | Scalar reference implementation for unsigned integer dot product between two `u8` byte slices. # Semantics Calculates $\sum a_i b_i$ with 3… | `src/kernels/scalar.rs` |
| `pub fn euclidean_distance_f32_bytes_scalar(a: &[f32], b_bytes: &[u8]) -> f32` | Scalar reference implementation for Euclidean distance between `f32` slice and raw LE `f32` byte slice. # Semantics Decodes `f32` values fr… | `src/kernels/scalar.rs` |
| `pub fn euclidean_distance_scalar(a: &[f32], b: &[f32]) -> f32` | Scalar reference implementation for Euclidean distance on `f32` slices. # Semantics Calculates $\sqrt{\sum (a_i - b_i)^2}$. # Complexity $O… | `src/kernels/scalar.rs` |
| `pub fn euclidean_distance_sq_f32_u8(a: &[f32], b: &[u8], alphas: &[f32], mins: &[f32]) -> f32` | Scalar reference implementation for mixed `f32`/`u8` scalar dequantized squared Euclidean distance. # Semantics Calculates $\sum (a_i - (b_… | `src/kernels/scalar.rs` |
| `pub fn euclidean_distance_sq_u8_scalar(a: &[u8], b: &[u8]) -> u32` | Scalar reference implementation for squared Euclidean distance between two `u8` byte slices. # Semantics Calculates $\sum (a_i - b_i)^2$ wi… | `src/kernels/scalar.rs` |
| `pub fn normalize_inplace(v: &mut [f32])` | Normalizes a vector in-place to unit L2 length. # Semantics Divides each element by $\sqrt{\sum v_i^2}$. Zero-norm vectors remain unmodifie… | `src/kernels/scalar.rs` |
| `pub fn compute_distance( a: &[f32], b: &[f32], metric: DistanceMetric, ) -> contextra_types::Result<f32>` | Berechnet die Distanz zwischen zwei Vektoren unter Verwendung der angegebenen Metrik inklusive Validierung. Diese Funktion ist der primäre… | `src/lib.rs` |
| `pub fn compute_distance_f32_bytes_trusted( a: &[f32], b_bytes: &[u8], metric: DistanceMetric, ) -> contextra_types::Result<f32>` | Berechnet die Distanz zwischen einem `f32`-Vektor und einem rohen Byte-Puffer (`&[u8]`), der `f32`-Werte darstellt. Dies ist besonders nütz… | `src/lib.rs` |
| `pub fn compute_distance_trusted( a: &[f32], b: &[f32], metric: DistanceMetric, ) -> contextra_types::Result<f32>` | Berechnet die Distanz zwischen zwei bereits validierten `f32`-Vektoren ohne erneute NaN-Prüfung. Wird intern verwendet, wenn Vektoren berei… | `src/lib.rs` |
| `pub fn validate_vector(vec: &[f32]) -> contextra_types::Result<()>` | Prüft, ob ein Vektor gültige endliche Zahlen enthält (kein NaN und kein Unendlich). NaN (Not a Number) und Unendlich führen in SIMD-Registe… | `src/lib.rs` |


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
pub use deletion_proof::{ CustodyLevel, DeletionLayer, DeletionProof, DeletionProofKeyPair, DeletionScope, ExcludedScope, LayerCleanupProof, SigningKeyHandle, };
pub use ed25519_proof::{DeletionProofError, SignatureVersion};
pub use error::{CryptoError, Result};
pub use kdf::{derive_key_argon2id, KdfHeader, KdfParams};
pub use kv_cipher::{EncryptedKvLayer, KvCipher, KvSegmentCipher, ModelFingerprint};
pub use kv_segment::{ eviction_worker::{emergency_wipe, EvictionWorker}, segment::KvSegment, store::TenantIsolatedKvStore, CURRENT_KV_KEY_DERIVATION_VERSION, };
pub use kv_shredding::{derive_subkey, KeyRegistry, SubKey, DEFAULT_SHRED_KEY_GROUP_SIZE};
pub use revocation_log::{RevocationEntry, RevocationLog, RevocationTarget};
pub use wal_completeness::verify_wal_chain_completeness;
pub use wal_crypto::CloudEgressGuard;
pub use wal_crypto::{ EncryptedWal, IntegrityVerifier, KmsProvider, LocalKeyManagerKms, WalEntrySnapshot, WalHmac, };
```

### Feature-Flags

| Flag | Aktiviert |
|---|---|
| `test-utils` | `[]` |
| `kv-encryption` | `[]` |
| `cloud-egress-guard` | `[]` |

### Schnittstellen — Public Traits (3)

**`KmsProvider`** — *Key Management Strategy interface for data key derivation, key wrapping/unwrapping, and key rotation.* (`src/wal_crypto.rs`)

```rust
pub trait KmsProvider : Send + Sync {
/// Retrieves the Data Encryption Key (DEK) zeroized on drop.
    fn get_key(&self) -> Result<Zeroizing<Vec<u8>>>;

    /// Returns the active key identifier or version alias.
    fn active_key_id(&self) -> &str;

    /// Derives a raw data key bound to the specified `key_id`.
    fn derive_data_key(&self, key_id: &[u8]) -> Result<Zeroizing<Vec<u8>>>;

    /// Wraps (encrypts) plaintext data or a raw key using the key bound to `key_id`.
    fn wrap_key(&self, key_id: &[u8], plaintext: &[u8]) -> Result<Vec<u8>>;

    /// Unwraps (decrypts) wrapped ciphertext data or key using the key bound to `key_id`.
    fn unwrap_key(&self, key_id: &[u8], wrapped: &[u8]) -> Result<Zeroizing<Vec<u8>>>;

    /// Rotates the active master key or key version and returns the new active key ID.
    fn rotate_key(&mut self) -> Result<String>;
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

**`LayerWitnessExt`** — *Extension trait providing accessors for `LayerWitness` without mutating `contextra-ports`.* (`src/deletion_proof/types.rs`)

```rust
pub trait LayerWitnessExt {
/// Extracts the `wal_seq_after` if layer is `WalAllSegments`, or `None`.
    fn wal_seq_after(&self) -> Option<u64>;
    /// Extracts the positive control check match count.
    fn control_matches(&self) -> u64;
}
```

### Öffentliche Datentypen — Structs (37)

| Struct | Felder / Form | Beschreibung | Datei |
|---|---|---|---|
| `AuditChain` | `{ /// Öffentlicher Zugriff auf die Liste der Ketten-Einträge für Prüfungen/Tests. pub entries: Vec<AuditChainEntry>, head_hash: [u8; 32], }` | Manipulationssichere Audit-Kette mit BLAKE3-Hash-Verkettung. | `src/audit_chain.rs` |
| `AuditChainEntry` | `{ /// Fortlaufender Index in der Audit-Kette (0-basiert). pub index: u64, /// Identifikator des Datenschemas. pub schema_id: String, /// Blake3-Hash des zum Zeit… }` | Einzelner Eintrag in der manipulationssicheren BLAKE3-Audit-Hash-Kette. | `src/audit_chain.rs` |
| `AuditChainHeadSignature` | `{ /// Index des signierten Kettenkopfs. pub chain_index: u64, /// Blake3-Hash des Kettenkopfs. pub head_hash: [u8; 32], /// Ed25519-Signatur (64 Bytes). pub sign… }` | Periodisch erzeugte Ed25519-Signatur über den aktuellen Kettenkopf. | `src/audit_chain.rs` |
| `AuditHeadAnchor` | `{ /// Sequenznummer / Kettenindex des extern verankerten Kettenkopfs. pub seq_no: u64, /// Blake3-Hash des Kettenkopfs an der Position `seq_no`. pub head_hash: [… }` | Externer Kopf-Anker zur Rollback-Erkennung für die Audit-Hash-Kette. | `src/audit_chain.rs` |
| `CloudEgressGuard` | `{ max_egress_bytes: usize, }` | Egress guard for inspecting and sanitizing encrypted payload blocks prior to cloud egress. | `src/wal_crypto.rs` |
| `DeletionProof` | `{ /// Version der Signatur-Payload-Konstruktion. #[serde(default = "default_signature_version")] pub signature_version: u8, /// Target scope of deletion. pub sco… }` | Kryptographisches Löschzertifikat (Deletion Proof) nach Art. 17 DSGVO. | `src/deletion_proof/types.rs` |
| `DeletionProofKeyPair` | `{ /// Privater Signierschlüssel. pub signing_key: ed25519_dalek::SigningKey, /// Öffentlicher Verifikationsschlüssel. #[zeroize(skip)] pub verifying_key: ed25519… }` | Schlüsselpaar für Ed25519-Löschbeweise. Signierschlüssel wird bei Drop automatisch zeroisiert (`ZeroizeOnDrop`). | `src/ed25519_proof.rs` |
| `DeletionProofVerifier` | `(tuple/unit)` | Public facade for standalone third-party verification of deletion proofs and audit chains. Enables external auditors, regulators, or third parties to… | `src/deletion_proof/verifier.rs` |
| `DerivedKey` | `(tuple/unit)` | Wrapper für abgeleitetes 256-Bit Schlüsselmaterial, das bei Drop automatisch aus dem Speicher gelöscht wird. | `src/kdf.rs` |
| `EncryptedCommitmentSalt` | `{ /// Shredding Group ID zur Verwaltung des Sub-Keys. pub group_id: u64, /// Verschlüsselter Salt. pub ciphertext: Vec<u8>, /// AES-GCM-SIV Nonce (12 Bytes). pub… }` | Verschlüsselter Salt-Wert für ein Record-Commitment. | `src/audit_chain.rs` |
| `EncryptedKvLayer` | `{ /// Serialized format version. Must equal [`CURRENT_KV_FORMAT_VERSION`] on decrypt. #[zeroize(skip)] pub format_version: u8, /// AES-256-GCM-SIV ciphertext con… }` | Container for an encrypted KV-cache segment layer. Plaintext data and nonce are zeroized on drop. Public metadata (`tenant_id`, `model_fingerprint`,… | `src/kv_cipher.rs` |
| `EncryptedRecordPayload` | `{ /// Unique identifier for the record. pub record_id: u64, /// Shred group identifier to which this record belongs. pub group_id: u64, /// AES-256-GCM-SIV encry… }` | Container for an encrypted record payload under Envelope Encryption. | `src/kv_shredding.rs` |
| `EncryptedSegmentPayload` | `{ /// Encrypted KV layer container from contextra-crypto. Zeroized on drop. pub layer: EncryptedKvLayer, }` | Encrypted layer representation stored inside a KvSegment when encryption is active. | `src/kv_segment/segment.rs` |
| `EncryptedWal` | `{ kms: Arc<dyn KmsProvider>, }` | Encrypted WAL chunk provider that handles transparent encryption/decryption of WAL payloads via KMS. # Invariants - Delegates encryption and key mana… | `src/wal_crypto.rs` |
| `EvictionWorker` | `{ sender: parking_lot::Mutex<mpsc::Sender<EvictionCommand>>, handle: parking_lot::Mutex<Option<std::thread::JoinHandle<()>>>, }` | Hintergrund-Worker für die Speicherfreigabe nach dem Least-Recently-Used-Prinzip (LRU). | `src/kv_segment/eviction_worker.rs` |
| `GraphRepairAttestation` | `{ /// Document ID whose neighborhood pointers were repaired. pub doc_id: DocId, /// Verified confirmation that no ghost pointers remain. pub verified_no_ghost_po… }` | Attestation confirming synchronous neighborhood graph repair for vector deletions. | `src/deletion_proof/types.rs` |
| `GroupKek` | `(tuple/unit)` | Group Key Encryption Key (KEK) - 256 bits, zeroized on drop. | `src/kv_shredding.rs` |
| `IntegrityVerifier` | `{ last_hmac: [u8; 32], integrity_key: Zeroizing<Vec<u8>>, last_seq_no: Option<u64>, }` | Stateful verifier for WAL HMAC-SHA256 hash chains. # Invariants - Maintains running `last_hmac` state across entries to enforce sequential hash-chain… | `src/wal_crypto.rs` |
| `KdfHeader` | `{ pub version: u8, pub kdf_id: u8, pub params: KdfParams, pub salt: Vec<u8>, }` | Versionierter KDF-Metadaten-Header zur deterministischen Reproduktion von Schlüsseln. Speichert die Argon2id-Kostenparameter (m_cost, t_cost, p_cost)… | `src/kdf.rs` |
| `KdfParams` | `{ pub m_cost_kib: u32, pub t_cost: u32, pub p_cost: u32, }` | Konfiguration der Argon2id KDF-Kostenparameter. | `src/kdf.rs` |
| `KeyManager` | `{ key: VolatileEncryptionKey, cipher_cache: OnceLock<Aes256GcmSiv>, nonce_prefix: [u8; 4], nonce_counter: AtomicU64, }` | Zentraler kryptographischer Schlüsselmanager für symmetrische Verschlüsselung. Hält den Master-Key im RAM, leitet per HKDF-Expand datei- und mandante… | `src/crypto.rs` |
| `KeyRegistry` | `{ groups: RwLock<AHashMap<u64, GroupEntry>>, revoked_groups: RwLock<AHashSet<u64>>, pub revocation_log: Option<std::sync::Arc<RevocationLog>>, }` | In-memory thread-safe registry for envelope KV crypto-shredding keys. | `src/kv_shredding.rs` |
| `KvSegment` | `{ #[zeroize(skip)] // Metadaten, keine sensiblen Tensor-Daten pub tenant_id: TenantId, #[zeroize(skip)] pub segment_id: u64, /// Versionsnummer der HKDF-Schlüsse… }` | Repräsentiert ein einzelnes Datensegment mit automatischer Speicherlöschung. | `src/kv_segment/segment.rs` |
| `KvSegmentCipher` | `{ key_manager: KeyManager, registry: KeyRegistry, }` | Chiffre für segmentweise verschlüsselte KV-Einträge. | `src/kv_cipher.rs` |
| `LayerCleanupProof` | `{ pub(crate) layer: DeletionLayer, pub(crate) _private: (), }` | Beweis, dass ein bestimmter DeletionLayer physisch bereinigt wurde. | `src/deletion_proof/types.rs` |
| `LayerWitnessMeasurement` | `{ /// Layer that was measured. pub layer: DeletionLayer, /// Applied scan technique. #[serde(with = "scan_technique_serde")] pub technique: ScanTechnique, /// Nu… }` | Measurement details for a single witnessed layer in V5 proof. | `src/deletion_proof/types.rs` |
| `LocalKeyManagerKms` | `{ key_manager: KeyManager, key_id: String, rotation_counter: u64, }` | Standard local implementation of [`KmsProvider`] wrapping [`KeyManager`]. Provides thread-safe, zeroized key derivation, wrapping, unwrapping, and ro… | `src/wal_crypto.rs` |
| `RecordDek` | `(tuple/unit)` | Record Data Encryption Key (DEK) - 256 bits, zeroized on drop. | `src/kv_shredding.rs` |
| `RevocationEntry` | `{ /// Monotone Sequenznummer im Log (0, 1, 2, ...). pub sequence_number: u64, /// Das widerrufene Target. pub target: RevocationTarget, /// Widerrufszeitpunkt in… }` | Ein einzelner Eintrag im manipulationssicheren Widerrufslog. | `src/revocation_log.rs` |
| `RevocationLog` | `{ storage: Option<(Arc<dyn Vfs>, PathBuf)>, signing_key: Option<SigningKey>, verifying_key: VerifyingKey, clock: Arc<dyn Clock>, entries: RwLock<Vec<RevocationEn… }` | Persistentes Protokoll über widerrufene Schlüsselgruppen. | `src/revocation_log.rs` |
| `SubKey` | `(tuple/unit)` | Sub-key or KEK bytes (256 bits) zeroized on drop. | `src/kv_shredding.rs` |
| `TenantIsolatedKvStore` | `{ /// 32 cache-line-gepaddete RwLock-Shards. Box<[_]> für flexible Shard-Anzahl. shards: Box<[Shard]>, shard_count: usize, /// Globaler Offset für Shard-Rotation… }` | Tenant-isolierter KV-Segment-Store. INV-TENANT-Analogon für KV-Bridge: Ein Tenant kann niemals Segmente eines anderen Tenants lesen. Strukturell erzw… | `src/kv_segment/store.rs` |
| `UncoveredLayer` | `{ /// Layer that was not covered in measurement. pub layer: DeletionLayer, /// Justification why layer was not covered for the given scope. pub reason: String, }` | Uncovered layer explanation for scope. | `src/deletion_proof/types.rs` |
| `VerificationResult` | `{ /// `true` if all cryptographic checks, signatures, and chain continuities passed. pub is_valid: bool, /// Detailed error reason if `is_valid` is `false`. pub… }` | Structured outcome of third-party deletion proof verification. | `src/deletion_proof/verifier.rs` |
| `VolatileEncryptionKey` | `{ key_bytes: Zeroizing<[u8; 32]>, // AES-256 Key }` | Defines a cryptographic key that is explicitly zeroed out when dropped or when an emergency trigger is activated, protecting against cold-boot attack… | `src/anti_tamper.rs` |
| `WalEntrySnapshot` | `{ pub tx_id: u64, pub seq_no: u64, pub op_type: u8, // 0: Put, 1: Delete, 2: TxEnd pub key: Vec<u8>, pub value: Vec<u8>, pub checksum: [u8; 32], pub prev_hmac: [… }` | Immutable snapshot of a WAL entry used for cryptographic integrity verification. # Invariants - Captures entry fields (`seq_no`, `op_type`, `key`, `v… | `src/wal_crypto.rs` |
| `WalHmac` | `{ mac: Hmac<Sha256>, }` | Stateful wrapper around HMAC-SHA256 initialized with WAL domain separation. # Invariants - Initialized with `b"contextra-wal-v1"` domain separation s… | `src/wal_crypto.rs` |

### Öffentliche Datentypen — Enums (12)

| Enum | Varianten | Beschreibung | Datei |
|---|---|---|---|
| `CryptoError` | `#[error("key derivation failed: {0}")] KeyDerivation(String), #[error("encryption failed: {0}")] Encryption(String), #[error("decryption failed: {0}")] Decryption(String), #[error("invalid nonce or k…` | Standalone error type for all operations within `contextra-crypto`. | `src/error.rs` |
| `CustodyLevel` | `/// Local custody. Local, /// Sovereign custody. Sovereign, /// Escrowed custody. Escrow,` | Custody level for cryptographic key management during deletion proof generation. | `src/deletion_proof/types.rs` |
| `DataClass` | `/// Öffentliche Daten. Public, /// Interne Betriebsdaten. Internal, /// Vertrauliche Geschäftsdaten. Confidential, /// Streng vertrauliche / reglementierte Daten. Restricted, /// Personenbezogene Dat…` | Datenklassifikation für Audit-Einträge (keine Freitext-Strings). | `src/audit_chain.rs` |
| `DeletionLayer` | `/// In-memory memtable records cleared. LsmMemtable, /// Persistent SSTable files purged across all LSM levels. SsTableAllLevels, /// HNSW graph nodes and tombstone references purged. HnswIndex, ///…` | Layer-explizite Coverage-Deklaration. | `src/deletion_proof/types.rs` |
| `DeletionProofError` | `/// Ungültiger Ed25519 VerifyingKey (32 Bytes). #[error("invalid verifying key: {0}")] InvalidVerifyingKey(String), /// Ungültige Ed25519 Signatur (64 Bytes) oder Fehlschlag bei der Verifikation. #[e…` | Fehlerzustände bei der Verifikation von Ed25519-Löschbeweisen. | `src/ed25519_proof.rs` |
| `DeletionScope` | `/// Individual document deletion. Document { /// DocId of target document. doc_id: DocId, /// TenantId of owning tenant. tenant_id: TenantId, }, /// Collection-wide deletion. Collection { /// Collect…` | Target scope for physical deletion. | `src/deletion_proof/types.rs` |
| `ExcludedScope` | `/// Wissen in Zusammenfassungen die als LLM-Fine-Tuning-Input dienten. ConsolidatedAndDistilled, /// LLM-Modellparameter (arXiv:2505.16831 — Unlearning Isn't Deletion). LlmParameterMemory,` | Explizite Nicht-Abdeckung — maschinenlesbar für Audit-Systeme. | `src/deletion_proof/types.rs` |
| `RevocationTarget` | `/// Shredding Group ID (z. B. für `KeyRegistry`) Group(u64), /// Key Encryption Key (KEK) Identifier Kek(String), /// Data Encryption Key (DEK) Identifier Dek(String), /// Specific Record Identifier…` | Das Ziel eines Widerruf-Eintrags im Log (Group ID, KEK ID, DEK ID oder Record ID). | `src/revocation_log.rs` |
| `SignatureVersion` | `/// HMAC-SHA256 legacy scope-only signature (version 1) V1 = 1, /// HMAC-SHA256 scope + layer coverage signature (version 2) V2 = 2, /// Ed25519 asymmetric signature with length-prefixed key hashes (…` | `SignatureVersion`-Enum für typsichere Versionierung von DeletionProof-Signaturen. Referenz: Befund v17 Teil 10.1 & INVARIANTE INV-DELETION-1 (die ph… | `src/ed25519_proof.rs` |
| `SigningKeyHandle` | `/// Symmetric HMAC key slice. Hmac(&'a [u8]), /// Ed25519 signing key reference. Ed25519(&'a ed25519_dalek::SigningKey),` | Handle to the key used for signing deletion proofs. | `src/deletion_proof/types.rs` |
| `VerificationErrorReason` | `/// Digital signature is invalid or tampered with. InvalidSignature(String), /// Unbroken chain invariant violated (hash mismatch or missing predecessor). ChainBroken { /// Position in chain where br…` | Detailed failure reasons for third-party verification of deletion proofs and audit chains. | `src/deletion_proof/verifier.rs` |
| `VerificationKey` | `/// Key for HMAC-SHA256 signature verification (version 1 and 2). Hmac(&'a [u8]), /// Key for Ed25519 signature verification (version 3). Ed25519(&'a ed25519_dalek::VerifyingKey),` | Key parameter for verification (either HMAC-SHA256 byte slice or Ed25519 VerifyingKey). | `src/deletion_proof/types.rs` |

### Typ-Aliase (2)

| Alias | Definition | Beschreibung |
|---|---|---|
| `Result` | `type Result<T> = std::result::Result<T, CryptoError>` | Result type alias for cryptographic operations in `contextra-crypto`. |
| `SpillHandler` | `type SpillHandler= Arc<dyn Fn(TenantId, u64, Vec<u8>) + Send + Sync>` | Optionaler Callback-Hook für Tier-2-LSM-Spill bei Eviction aus dem In-Memory LRU Cache. |

### Konstanten (16)

| Konstante | Typ | Beschreibung |
|---|---|---|
| `CURRENT_KV_FORMAT_VERSION` | `u8` | Current serialized format version for [`EncryptedKvLayer`]. Increment when the serialized layout changes in a breaking way. |
| `CURRENT_KV_KEY_DERIVATION_VERSION` | `u8` | Aktuelle Version der KV-Segment-Schlüsselableitung. Erhöhe diesen Wert, wenn sich der HKDF-Info-String oder der Salt-Aufbau ändert. |
| `DEFAULT_M_COST_KIB` | `u32` | OWASP Standardempfehlungen für Argon2id |
| `DEFAULT_P_COST` | `u32` | Standard-Parallelitätsgrad für Argon2id. |
| `DEFAULT_SEGMENT_CAPACITY_PER_TENANT` | `usize` | — |
| `DEFAULT_SHARD_COUNT` | `usize` | — |
| `DEFAULT_SHRED_KEY_GROUP_SIZE` | `usize` | Default size of a shred key group (number of records sharing one KEK). |
| `DEFAULT_T_COST` | `u32` | Standard-Zeitkosten (Iterationen) für Argon2id. |
| `DELETION_PROOF_KEY_PURPOSE` | `&[u8]` | Domain separation tag for deletion proof key derivation. |
| `KDF_HEADER_MAGIC` | `&[u8` | Magischer Header-Marker "MFKD" (Contextra Key Derivation) |
| `KDF_HEADER_VERSION_1` | `u8` | Aktuelle KDF Header Version |
| `KDF_ID_ARGON2ID` | `u8` | Identifikator für Argon2id |
| `MIN_M_COST_KIB` | `u32` | OWASP Mindestanforderungen für Argon2id |
| `MIN_P_COST` | `u32` | Minimaler Parallelitätsgrad für Argon2id gemäß OWASP. |
| `MIN_SALT_LEN` | `usize` | Min. empfohlener Salt-Umfang (16 Bytes) |
| `MIN_T_COST` | `u32` | Minimale Anzahl an Iterationen (Zeitkosten) für Argon2id gemäß OWASP. |

### Öffentliche Funktionen (166)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn as_bytes(&self) -> &[u8]` | Accessor for cryptographic operations. | `src/anti_tamper.rs` |
| `pub fn emergency_wipe(&mut self)` | Emergency Trigger: Explicitly wipes the key from memory. This is to be hooked into hardware sensors (e.g. crash/voltage). | `src/anti_tamper.rs` |
| `pub fn inspect_key_bytes_for_test(&self) -> &[u8` | Test-only method to inspect key bytes. Used to verify zeroing in integration tests. | `src/anti_tamper.rs` |
| `pub fn new(raw: [u8; 32]) -> Self` | Creates a new volatile key from a raw 32-byte array. | `src/anti_tamper.rs` |
| `pub fn append( &mut self, schema_id: impl Into<String>, rules_hash: [u8; 32], data_class: DataClass, doc_id: DocId, tx_id: TxId, commitment: Option<[u8; 32]>, ) -> Result<&AuditChainEntry>` | Fügt einen neuen Eintrag zur Audit-Kette hinzu und aktualisiert den Kettenkopf-Hash. | `src/audit_chain.rs` |
| `pub fn compute_hash( index: u64, schema_id: &str, rules_hash: &[u8; 32], data_class: DataClass, doc_id: DocId, tx_id: TxId, prev_hash: &[u8; 32], commitment: Option<&[u8; 32]>, ) -> Result<[u8` | Berechnet den Blake3-Hash für die Felder eines Audit-Ketten-Eintrags. | `src/audit_chain.rs` |
| `pub fn compute_record_commitment(salt: &[u8], attr: &[u8]) -> [u8` | Berechnet das Commitment `Blake3(salt || attr)`. | `src/audit_chain.rs` |
| `pub fn decrypt(&self, registry: &KeyRegistry) -> Result<Vec<u8>>` | Entschlüsselt das Salt. Schlägt fehl, wenn der Sub-Key für `group_id` geschreddert/widerrufen wurde. | `src/audit_chain.rs` |
| `pub fn encrypt( registry: &KeyRegistry, master_key: &KeyManager, group_id: u64, salt: &[u8], ) -> Result<Self>` | Verschlüsselt ein Salt-Byte-Array über den `KeyRegistry`-Shredding-Mechanismus. | `src/audit_chain.rs` |
| `pub fn entries(&self) -> &[AuditChainEntry]` | Gibt eine Referenz auf alle Einträge der Kette zurück. | `src/audit_chain.rs` |
| `pub fn head_hash(&self) -> [u8` | Liefert den aktuellen Kettenkopf-Hash. | `src/audit_chain.rs` |
| `pub fn is_empty(&self) -> bool` | Prüft, ob die Kette leer ist. | `src/audit_chain.rs` |
| `pub fn len(&self) -> usize` | Liefert die Anzahl der Einträge in der Kette. | `src/audit_chain.rs` |
| `pub fn new(seq_no: u64, head_hash: [u8; 32]) -> Self` | Erstellt einen neuen nicht signierten `AuditHeadAnchor`. | `src/audit_chain.rs` |
| `pub fn new() -> Self` | Erzeugt eine neue, leere Audit-Kette. | `src/audit_chain.rs` |
| `pub fn sign_head( &self, signing_key: &ed25519_dalek::SigningKey, ) -> Result<AuditChainHeadSignature>` | Periodischer Aufruf: Signiert den aktuellen Kettenkopf mit Ed25519. | `src/audit_chain.rs` |
| `pub fn verify_chain(&self) -> Result<bool>` | Verifiziert die vollständige Kette auf Integrität. # Warnhinweis / Security Warning Diese Methode prüft lediglich die **interne kryptograph… | `src/audit_chain.rs` |
| `pub fn verify_chain_with_anchor( &self, anchor: &AuditHeadAnchor, verifying_key: Option<&ed25519_dalek::VerifyingKey>, ) -> Result<bool>` | Verifiziert die Kette gegen einen externen Kopf-Anker ([`AuditHeadAnchor`]). Schlägt fehl (`Ok(false)`), wenn: 1. Die Kette kürzer ist als… | `src/audit_chain.rs` |
| `pub fn verify_head_signature( head_sig: &AuditChainHeadSignature, verifying_key: &ed25519_dalek::VerifyingKey, ) -> Result<bool>` | Verifiziert eine periodische Kettenkopf-Signatur gegen den angegebenen VerifyingKey. | `src/audit_chain.rs` |
| `pub fn with_signature( seq_no: u64, head_hash: [u8; 32], head_signature: AuditChainHeadSignature, ) -> Self` | Erstellt einen neuen signierten `AuditHeadAnchor`. | `src/audit_chain.rs` |
| `pub fn cipher_for(&self, tenant_id: contextra_types::TenantId) -> Result<Self>` | Derives a tenant-isolated `KeyManager` instance for a specific `TenantId`. | `src/crypto.rs` |
| `pub fn cipher_for_scoped<T>( &self, scoped_key: contextra_types::TenantScoped<T>, expected_tenant_id: &contextra_types::TenantId, ) -> Result<Self>` | Derives a tenant-isolated `KeyManager` instance enforcing compile-time `TenantScoped` verification. Unpacks `scoped_key` only if the bound… | `src/crypto.rs` |
| `pub fn decrypt_auto_nonce(&self, ciphertext: &[u8], nonce_bytes: &[u8; 12]) -> Result<Vec<u8>>` | Decrypts a block of data using a full 12-byte nonce. | `src/crypto.rs` |
| `pub fn derive_deletion_proof_key(&self) -> Result<[u8` | Derives a deletion proof signing key for cryptographic deletion verification. | `src/crypto.rs` |
| `pub fn derive_file_key(&self, file_id: &[u8]) -> Result<Self>` | Derives a sub-key for a specific file or logical stream. This prevents nonce-reuse when multiple files use the same master key. | `src/crypto.rs` |
| `pub fn derive_kv_key( &self, tenant_id: contextra_types::TenantId, model_fingerprint: &crate::kv_cipher::ModelFingerprint, ) -> Result<Self>` | Derives a sub-key for KV-cache segment encryption, bound to `(tenant_id, model_fingerprint)`. # Format Version Uses HKDF info string `"cont… | `src/crypto.rs` |
| `pub fn derive_kv_key_scoped( &self, scoped_fingerprint: contextra_types::TenantScoped<&crate::kv_cipher::ModelFingerprint>, expected_tenant_id: &contextra_types::TenantId, ) -> Result<Self>` | Derives a sub-key for KV-cache segment encryption with strict compile-time tenant scope enforcement. Unpacks `scoped_fingerprint` only if t… | `src/crypto.rs` |
| `pub fn derive_segment_key(&self, info: &str) -> Result<Self>` | Derives a sub-key specifically for a KV-cache segment based on a versioned info string. | `src/crypto.rs` |
| `pub fn emergency_wipe(&mut self)` | Emergency Trigger: Explicitly wipes the key from memory and clears cipher cache. | `src/crypto.rs` |
| `pub fn encrypt_auto_nonce(&self, data: &[u8]) -> Result<(Vec<u8>, [u8` | Encrypts a block of data with a deterministically generated 12-byte nonce. Returns the ciphertext and the full 12-byte nonce used. | `src/crypto.rs` |
| `pub fn export_dek(&self) -> Zeroizing<Vec<u8>>` | Exports the 32-byte raw Data Encryption Key (DEK) wrapped in [`Zeroizing<Vec<u8>>`]. | `src/crypto.rs` |
| `pub fn inspect_key_bytes_for_test(&self) -> &[u8` | Provides access to the key bytes ONLY during testing. | `src/crypto.rs` |
| `pub fn integrity_key(&self) -> Result<[u8` | Derives an integrity key for HMAC-SHA256. | `src/crypto.rs` |
| `pub fn try_new(passphrase: &str, salt: &[u8]) -> Result<Self>` | Creates a new KeyManager by deriving a key from a passphrase using HKDF-SHA256. # WARNHINWEIS Diese Funktion nutzt HKDF-SHA256 und setzt vo… | `src/crypto.rs` |
| `pub fn try_new_random_salt(passphrase: &str) -> Result<(Self, [u8` | Creates a new KeyManager with a cryptographically secure random salt. | `src/crypto.rs` |
| `pub fn try_new_with_kdf(passphrase: &str, header: &KdfHeader) -> Result<Self>` | Creates a new KeyManager using Argon2id password-based key derivation. Nutzt Argon2id mit den im [`KdfHeader`] definierten Kostenparametern… | `src/crypto.rs` |
| `pub fn verify_proof_sequence(proofs: &[DeletionProof]) -> Result<bool>` | Helper functions for verifying deletion proof chains and sequence continuity. | `src/deletion_proof/chain.rs` |
| `pub fn compute_wal_delete_receipt( prev_hmac: &[u8; 32], delete_event_payload: &[u8], integrity_key: &[u8], ) -> Result<[u8` | Erzeugt eine WAL-Löschquittung H(hmac_prev || delete_event) für den Nachweis auf WAL-Ebene gemäß DSGVO Art. 17. | `src/deletion_proof/creation.rs` |
| `pub fn create( scope: DeletionScope, deleted_keys: Vec<Vec<u8>>, deleted_after_tx: TxId, covered_layers: Vec<LayerCleanupProof>, excluded_scopes: Vec<ExcludedScope>, proof_key: &[u8], ) -> Result<Self>` | Erstellt und signiert einen DeletionProof nach Layer-Bereinigung. | `src/deletion_proof/creation.rs` |
| `pub fn create_full_v3( scope: DeletionScope, mut deleted_keys: Vec<Vec<u8>>, deleted_after_tx: TxId, covered_layers: Vec<LayerCleanupProof>, excluded_scopes: Vec<ExcludedScope>, wal_chain_receipt: Option<[u8; 32]>, audit_chain_position: Option<u64>, attested_…` | Erstellt und signiert einen DeletionProof (Version 3, Ed25519) mit allen optionalen Erweiterungen. | `src/deletion_proof/creation.rs` |
| `pub fn create_v3( scope: DeletionScope, deleted_keys: Vec<Vec<u8>>, deleted_after_tx: TxId, covered_layers: Vec<LayerCleanupProof>, excluded_scopes: Vec<ExcludedScope>, attested_at: i64, graph_repair: &[GraphRepairAttestation], signing_key: &ed25519_dalek::Si…` | Erstellt und signiert einen DeletionProof (Version 3) mit Ed25519. | `src/deletion_proof/creation.rs` |
| `pub fn create_v3_with_audit_position( scope: DeletionScope, deleted_keys: Vec<Vec<u8>>, deleted_after_tx: TxId, covered_layers: Vec<LayerCleanupProof>, excluded_scopes: Vec<ExcludedScope>, audit_chain_position: Option<u64>, attested_at: i64, graph_repair: &[G…` | Erstellt und signiert einen DeletionProof (Version 3, Ed25519) inklusive Verweis auf den Audit-Chain-Eintrag. | `src/deletion_proof/creation.rs` |
| `pub fn create_with_wal_receipt( scope: DeletionScope, mut deleted_keys: Vec<Vec<u8>>, deleted_after_tx: TxId, covered_layers: Vec<LayerCleanupProof>, excluded_scopes: Vec<ExcludedScope>, wal_chain_receipt: Option<[u8; 32]>, proof_key: &[u8], ) -> Result<Self>` | Erstellt und signiert einen DeletionProof (Version 2) inklusive optionaler WAL-HMAC-Kettenquittung. | `src/deletion_proof/creation.rs` |
| `pub fn create_with_wal_receipt_v3( scope: DeletionScope, deleted_keys: Vec<Vec<u8>>, deleted_after_tx: TxId, covered_layers: Vec<LayerCleanupProof>, excluded_scopes: Vec<ExcludedScope>, wal_chain_receipt: Option<[u8; 32]>, attested_at: i64, graph_repair: &[Gr…` | Erstellt und signiert einen DeletionProof (Version 3, Ed25519) inklusive optionaler WAL-HMAC-Kettenquittung. | `src/deletion_proof/creation.rs` |
| `pub fn issue<'a>( ctx: &contextra_types::RequestContext, saga: &contextra_types::SagaRecord, witnesses: &[contextra_ports::scanner::LayerWitness], excluded: &[ExcludedScope], custody: CustodyLevel, key: impl Into<SigningKeyHandle<'a>>, clock: &dyn contextra_p…` | Issues a new `DeletionProof` based on verified `LayerWitness` measurements. | `src/deletion_proof/creation.rs` |
| `pub fn derive_deletion_proof_key_for_tenant( master_key: &[u8; 32], tenant_id: TenantId, ) -> Result<[u8` | Derives a 32-byte key specifically bound to a tenant and deletion scope. | `src/deletion_proof/key_derivation.rs` |
| `pub fn deserialize<'de, D>(deserializer: D) -> Result<ScanTechnique, D::Error> where D: Deserializer<'de>,` | — | `src/deletion_proof/types.rs` |
| `pub fn for_test(layer: DeletionLayer) -> Self` | Erzeugt ein `LayerCleanupProof` ausschließlich für Tests. | `src/deletion_proof/types.rs` |
| `pub fn hash_deleted_keys_length_prefixed(deleted_keys: &[Vec<u8>]) -> [u8` | Berechnet den Blake3-Hash einer deterministisch sortierten Liste gelöschter Schlüssel mit Längenpräfix. | `src/deletion_proof/types.rs` |
| `pub fn layer(&self) -> &DeletionLayer` | Gibt den zugrundeliegenden DeletionLayer zurück. | `src/deletion_proof/types.rs` |
| `pub fn map_witness_layer_to_deletion_layer( layer: &WitnessLayer, witness: &LayerWitness, ) -> Result<DeletionLayer>` | Mappt `WitnessLayer` erschöpfend auf `DeletionLayer` ohne `_`-Zweig. | `src/deletion_proof/types.rs` |
| `pub fn serialize<S>(tech: &ScanTechnique, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer,` | — | `src/deletion_proof/types.rs` |
| `pub fn export_for_audit(&self) -> Result<String>` | Formats the deletion proof into a JSON audit string including legal coverage bounds. | `src/deletion_proof/verification.rs` |
| `pub fn signature_version_typed( &self, ) -> std::result::Result<SignatureVersion, DeletionProofError>` | Typed accessor for the signature version. | `src/deletion_proof/verification.rs` |
| `pub fn tenant_id(&self) -> TenantId` | Accessor for the TenantId of this proof. | `src/deletion_proof/verification.rs` |
| `pub fn verify<'a>(&self, key: impl Into<VerificationKey<'a>>) -> Result<bool>` | Verifies the signature of this DeletionProof against a provided key. | `src/deletion_proof/verification.rs` |
| `pub fn verify_external( &self, public_key: &ed25519_dalek::VerifyingKey, ) -> std::result::Result<(), CryptoError>` | Verifies this proof using ONLY the provided Ed25519 public key. | `src/deletion_proof/verification.rs` |
| `pub fn verify_wal_delete_receipt( receipt: &[u8; 32], prev_hmac: &[u8; 32], delete_event_payload: &[u8], integrity_key: &[u8], ) -> Result<bool>` | Verifies a WAL delete receipt HMAC over previous HMAC, delete event, and integrity key. | `src/deletion_proof/verification.rs` |
| `pub fn failure(reason: VerificationErrorReason) -> Self` | Creates a failed verification result with an explicit reason. | `src/deletion_proof/verifier.rs` |
| `pub fn success() -> Self` | Creates a successful verification result. | `src/deletion_proof/verifier.rs` |
| `pub fn verify_audit_chain_and_head( chain: &crate::audit_chain::AuditChain, head_signature: &crate::audit_chain::AuditChainHeadSignature, verifying_key_bytes: &[u8; 32], ) -> VerificationResult` | Verifies an audit chain and checks head signature against expected verifying key. | `src/deletion_proof/verifier.rs` |
| `pub fn verify_standalone( proof: &DeletionProof, verifying_key_bytes: &[u8; 32], ) -> VerificationResult` | Verifies a single standalone `DeletionProof` using a public key. | `src/deletion_proof/verifier.rs` |
| `pub fn verify_standalone_for_tenant( proof: &DeletionProof, expected_tenant: TenantId, verifying_key_bytes: &[u8; 32], ) -> VerificationResult` | Verifies a `DeletionProof` and checks tenant binding. | `src/deletion_proof/verifier.rs` |
| `pub fn as_u8(&self) -> u8` | Gibt den `u8`-Wert der Version zurück. | `src/ed25519_proof.rs` |
| `pub fn from_signing_key(signing_key: ed25519_dalek::SigningKey) -> Self` | Erzeugt ein `DeletionProofKeyPair` aus einem bestehenden Signierschlüssel. | `src/ed25519_proof.rs` |
| `pub fn generate() -> Self` | Generiert ein neues Ed25519-Schlüsselpaar unter Nutzung des System-RNGs (`OsRng`). | `src/ed25519_proof.rs` |
| `pub fn generate_with_rng(rng: &mut impl RngCore) -> Self` | Generiert ein neues Ed25519-Schlüsselpaar unter Nutzung eines injizierten Zufallsgenerators. | `src/ed25519_proof.rs` |
| `pub fn sign_deletion_proof_v3(keypair: &DeletionProofKeyPair, proof_payload: &[u8]) -> [u8` | Signiert den kanonischen Byte-Payload eines Version-3-Löschbeweises mit Ed25519. Gibt eine 64-Byte Ed25519-Signatur zurück. | `src/ed25519_proof.rs` |
| `pub fn signing_key(&self) -> &ed25519_dalek::SigningKey` | Gibt eine Referenz auf den Signierschlüssel zurück. | `src/ed25519_proof.rs` |
| `pub fn verify_deletion_proof_v3( verifying_key_bytes: &[u8; 32], proof_payload: &[u8], signature: &[u8; 64], ) -> Result<(), DeletionProofError>` | Verifiziert eine Ed25519-Signatur eines Version-3-Löschbeweises rein funktional. Diese Funktion benötigt keine Contextra-internen Zustände… | `src/ed25519_proof.rs` |
| `pub fn verifying_key(&self) -> &ed25519_dalek::VerifyingKey` | Gibt eine Referenz auf den Verifikationsschlüssel zurück. | `src/ed25519_proof.rs` |
| `pub fn verifying_key_bytes(&self) -> [u8` | Gibt die 32-Byte-Repräsentation des öffentlichen Verifikationsschlüssels zurück. | `src/ed25519_proof.rs` |
| `pub fn wal_corruption(offset: u64, reason: impl Into<String>) -> Self` | Helper to construct a `WalCorruption` error. | `src/error.rs` |
| `pub fn derive_key_argon2id(passphrase: &str, header: &KdfHeader) -> Result<DerivedKey>` | Leitet einen kryptographisch starken Schlüssel mittels Argon2id aus einer Passphrase ab. # Ablauf der Argon2id-Schlüsselableitung: 1. Passp… | `src/kdf.rs` |
| `pub fn from_bytes(bytes: &[u8]) -> Result<Self>` | Parst einen `KdfHeader` aus Binärdaten. Panic-Freiheit (total function) bei beliebigen Byte-Eingaben. | `src/kdf.rs` |
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
| `pub fn emergency_wipe(store: &TenantIsolatedKvStore)` | SEPARATER Pfad für Notfall-Löschung (z.B. Prozess-Shutdown, expliziter Sicherheits-Trigger). Synchron, blockierend, vor Rückkehr abgeschlos… | `src/kv_segment/eviction_worker.rs` |
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
| `pub fn clear_all(&self)` | Leert alle Segmente aus allen Shards vollständig und löscht deren Speicher via Zeroize. | `src/kv_segment/store.rs` |
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
| `pub fn revoke_group(&self, group_id: u64) -> Result<bool>` | Revokes (destroys) the Group KEK for `group_id` (Group Deletion / Group Crypto-Shredding). # Crash Semantics & Atomic Logging 1. If a `Revo… | `src/kv_shredding.rs` |
| `pub fn revoke_record(&self, group_id: u64, record_id: u64) -> bool` | Revokes (destroys) the DEK wrap for a single record within `group_id` (Single Record Deletion). Leaves neighbor records in the same group i… | `src/kv_shredding.rs` |
| `pub fn revoke_subkey(&self, group_id: u64) -> Result<bool>` | Revokes (destroys) the Group KEK for `group_id` in O(1). Alias for `revoke_group`. | `src/kv_shredding.rs` |
| `pub fn with_revocation_log(mut self, log: std::sync::Arc<RevocationLog>) -> Self` | Attaches an optional `RevocationLog` to check for persisted key/group revocations, syncing any previously logged revocations into `revoked_… | `src/kv_shredding.rs` |
| `pub fn append(&self, target: RevocationTarget) -> Result<RevocationEntry>` | Fügt einen neuen Widerrufs-Eintrag für `target` an den Log an. Nutzt den injizierten Clock-Port zur Generierung des Zeitstempels und den Si… | `src/revocation_log.rs` |
| `pub fn canonical_bytes(&self) -> Vec<u8>` | Gibt eine kanonische Byte-Repräsentation des Targets für das Hashing zurück. | `src/revocation_log.rs` |
| `pub fn compute_entry_hash( sequence_number: u64, target: &RevocationTarget, revoked_at_nanos: u64, prev_hash: &[u8; 32], ) -> [u8` | Berechnet den SHA-256-Hash eines Eintrags aus seinen kanonischen Feldern. | `src/revocation_log.rs` |
| `pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8` | — | `src/revocation_log.rs` |
| `pub fn is_empty(&self) -> bool` | Prüft, ob der Log leer ist. | `src/revocation_log.rs` |
| `pub fn is_revoked(&self, target: &RevocationTarget) -> bool` | Prüft, ob ein gegebenes Target (KEK, DEK, Group oder Record) widerrufen wurde. | `src/revocation_log.rs` |
| `pub fn len(&self) -> usize` | Gibt die Anzahl der Widerrufe im Log zurück. | `src/revocation_log.rs` |
| `pub fn new_in_memory( clock: Arc<dyn Clock>, signing_key: Option<SigningKey>, verifying_key: VerifyingKey, ) -> Self` | Erstellt einen neuen In-Memory-Widerrufslog ohne Dateipersistenz. | `src/revocation_log.rs` |
| `pub fn open_or_create( vfs: Arc<dyn Vfs>, path: impl AsRef<Path>, clock: Arc<dyn Clock>, signing_key: Option<SigningKey>, verifying_key: VerifyingKey, ) -> Result<Self>` | Öffnet einen bestehenden Widerrufslog oder erzeugt einen neuen unter `path` über den Vfs-Port. Beim Öffnen einer bestehenden Datei wird die… | `src/revocation_log.rs` |
| `pub fn revoked_targets(&self) -> AHashSet<RevocationTarget>` | Gibt eine Kopie aller aktuell im Log widerrufenen Targets zurück. | `src/revocation_log.rs` |
| `pub fn serialize<S>(bytes: &[u8; 64], serializer: S) -> Result<S::Ok, S::Error> where S: Serializer,` | — | `src/revocation_log.rs` |
| `pub fn verify_integrity(&self) -> Result<()>` | Verifiziert die Integrität der gesamten Logeinträge-Kette (Sequenz, Hashes, Ed25519-Signaturen). | `src/revocation_log.rs` |
| `pub fn verify_wal_chain_completeness( wal_tail_hmac: &[u8; 32], manifest_high_water_mark: &[u8; 32], ) -> Result<(), CryptoError>` | Validiert die Vollständigkeit einer WAL-HMAC-Kette über ein gesamtes Segment. | `src/wal_completeness.rs` |
| `pub fn decrypt_chunk(&self, data: &[u8]) -> Result<Vec<u8>>` | Decrypts the WAL chunk by extracting the prepended 12-byte nonce from the data. | `src/wal_crypto.rs` |
| `pub fn encrypt_chunk(&self, payload: &[u8]) -> Result<Vec<u8>>` | Wraps the internal WAL chunk in AES-256-GCM-SIV stream. Prepends the 12-byte nonce to the encrypted ciphertext. | `src/wal_crypto.rs` |
| `pub fn finalize(self) -> [u8` | Schließt die HMAC-Berechnung ab und liefert den 32-Byte-Digest. | `src/wal_crypto.rs` |
| `pub fn key_manager(&self) -> &KeyManager` | Accesses the underlying `KeyManager`. | `src/wal_crypto.rs` |
| `pub fn kms(&self) -> &Arc<dyn KmsProvider>` | Returns a reference to the inner KMS provider. | `src/wal_crypto.rs` |
| `pub fn last_hmac_snapshot(&self) -> [u8` | Gibt den aktuellen `last_hmac`-Kettenzustand zurück (für Verifier-Handoff). | `src/wal_crypto.rs` |
| `pub fn last_seq_no_snapshot(&self) -> Option<u64>` | Gibt die aktuell bekannte `last_seq_no` zurück (für Verifier-Handoff). | `src/wal_crypto.rs` |
| `pub fn new(key_manager: KeyManager, key_id: impl Into<String>) -> Self` | Creates a new `LocalKeyManagerKms` wrapping an existing `KeyManager`. | `src/wal_crypto.rs` |
| `pub fn new() -> Self` | Creates a new `CloudEgressGuard` with a default maximum payload limit of 10 MB. | `src/wal_crypto.rs` |
| `pub fn new(key_manager: KeyManager, file_id: &[u8]) -> Result<Self>` | Creates a new EncryptedWal with per-file key derivation via LocalKeyManagerKms to prevent nonce-reuse. `file_id` (e.g., filename) is used t… | `src/wal_crypto.rs` |
| `pub fn new(integrity_key: &[u8]) -> Result<Self>` | — | `src/wal_crypto.rs` |
| `pub fn new(integrity_key: &[u8]) -> Self` | — | `src/wal_crypto.rs` |
| `pub fn set_last_hmac(&mut self, hmac: [u8; 32])` | Übernimmt einen bereits bekannten `last_hmac`-Kettenzustand, z. B. beim Wechsel von einem Normal- auf einen Legacy-Key-Verifier MITTEN in e… | `src/wal_crypto.rs` |
| `pub fn set_last_seq_no(&mut self, seq_no: Option<u64>)` | Übernimmt eine bereits bekannte `last_seq_no` für den Verifier-Handoff. | `src/wal_crypto.rs` |
| `pub fn skip_hmac_verify_legacy(&mut self, entry: &WalEntrySnapshot)` | Updates the chain state for legacy V1 entries without HMAC verification. | `src/wal_crypto.rs` |
| `pub fn update(&mut self, data: &[u8])` | Führt neue Daten in die laufende HMAC-Berechnung ein. | `src/wal_crypto.rs` |
| `pub fn validate_egress_payload(&self, payload: &[u8]) -> Result<()>` | Validates that an egress payload meets size and structural integrity bounds. | `src/wal_crypto.rs` |
| `pub fn verify_and_update(&mut self, entry: &WalEntrySnapshot, offset: u64) -> Result<()>` | Default verification delegating to V3 verification. | `src/wal_crypto.rs` |
| `pub fn verify_and_update_v2(&mut self, entry: &WalEntrySnapshot, offset: u64) -> Result<()>` | Verifiziert und aktualisiert die HMAC-Kette im V2-Format (Rückwärtskompatibilität). | `src/wal_crypto.rs` |
| `pub fn verify_and_update_v3(&mut self, entry: &WalEntrySnapshot, offset: u64) -> Result<()>` | Verifies a V3 entry (with tx_id and length prefixes) and updates the chain state. | `src/wal_crypto.rs` |
| `pub fn with_kms(kms: Arc<dyn KmsProvider>) -> Self` | Creates a new EncryptedWal using an arbitrary KmsProvider. | `src/wal_crypto.rs` |
| `pub fn with_max_bytes(max_egress_bytes: usize) -> Self` | Creates a custom `CloudEgressGuard` with a specified maximum payload size. | `src/wal_crypto.rs` |


---

## `contextra-vector`

| Attribut | Wert |
|---|---|
| **Ring** | 0 — Ring 0 — Foundation & Pure Numerics/Types |
| **Status** | In Migration |
| **Zweck** | HNSW- und DiskANN-Vektorindex-Implementierungen. |
| **Unsafe-Code** | Nein (`#![forbid(unsafe_code)]`) |
| **Interne Abhängigkeiten** | `contextra-crypto`, `contextra-mvcc`, `contextra-ports`, `contextra-simd`, `contextra-sys`, `contextra-types` |
| **Externe Abhängigkeiten (Auswahl)** | `ahash`, `blake3`, `memmap2`, `parking_lot`, `rand`, `roaring`, `serde`, `subtle`, `tracing`, `zeroize` |

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
pub use hnsw::{ hnsw_purger, hnsw_scanner, HnswConfig, HnswIndex, HnswPurger, HnswScanner, RebuildStatus, };
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

### Öffentliche Datentypen — Structs (35)

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
| `HnswPurger` | `{ index: Arc<HnswIndex>, }` | Purger-Implementierung für `WitnessLayer::HnswIndex`. | `src/hnsw/purge.rs` |
| `HnswScanner` | `{ index: Arc<HnswIndex>, }` | LayerScanner-Implementierung für `WitnessLayer::HnswIndex`. | `src/hnsw/scanner.rs` |
| `MmapIndex` | `{ pub mmap: std::sync::Arc<memmap2::Mmap>, pub header: HnswHeader, pub path: Option<std::path::PathBuf>, }` | A reader for memory-mapped HNSW indices. | `src/persistence/mmap.rs` |
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

### Öffentliche Funktionen (195)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn compute_gamma_edge_budget(base_degree: usize, predicate_selectivity: f32) -> usize` | Computes the augmented edge budget factor $\gamma$ for graph expansion in ACORN search. Based on Patel et al. 2024 (ACORN: Performant and P… | `src/acorn/gamma_augmentation.rs` |
| `pub fn from_vectors(vectors: Vec<(DocId, Vec<f32>)>) -> Self` | Creates a `NaiveReferenceIndex` pre-populated with vectors using Euclidean distance. | `src/acorn/naive_reference.rs` |
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
| `pub fn recover_pending_delta( &self, ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<usize>> + Send + '_>>` | Liest uncommittete Pending-Vektoren aus `pending.wal` und fügt sie in den Index ein. | `src/diskann/build.rs` |
| `pub fn recover_pending_delta_sync(&self) -> Result<usize>` | — | `src/diskann/build.rs` |
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
| `pub fn zeroize_and_free_node(&self, ram_idx: usize, max_layer: usize, m: usize)` | Zeroizes connection memory for a node slot across all layers, clears counts, and then marks the node's slot as free in the free_list. | `src/hnsw/arena.rs` |
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
| `pub fn check_connectivity(&self) -> contextra_types::Result<()>` | — | `src/hnsw/core_insert.rs` |
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
| `pub fn rebuild_sync_with_clock(&self, clock: &dyn contextra_ports::Clock) -> Result<()>` | — | `src/hnsw/core_rebuild.rs` |
| `pub async fn rebuild_with_clock(&self, clock: &dyn contextra_ports::Clock) -> Result<()>` | — | `src/hnsw/core_rebuild.rs` |
| `pub fn visited_dead_nodes(&self) -> u64` | — | `src/hnsw/core_rebuild.rs` |
| `pub async fn wait_for_rebuild(&self) -> bool` | — | `src/hnsw/core_rebuild.rs` |
| `pub async fn wait_for_rebuild_with_clock(&self, clock: &dyn contextra_ports::Clock) -> bool` | — | `src/hnsw/core_rebuild.rs` |
| `pub async fn wait_for_rebuild_with_timeout(&self, timeout: std::time::Duration) -> bool` | — | `src/hnsw/core_rebuild.rs` |
| `pub async fn wait_for_rebuild_with_timeout_and_clock( &self, timeout: std::time::Duration, clock: &dyn contextra_ports::Clock, ) -> bool` | — | `src/hnsw/core_rebuild.rs` |
| `pub async fn search_adaptive( &self, query: &[f32], k: usize, policy: &AdaptiveEfPolicy, ) -> Result<Vec<ScoredDocument>>` | Performs an adaptive search (Ada-ef) on the HNSW index, dynamically adjusting search depth `ef`. | `src/hnsw/core_search.rs` |
| `pub async fn search_adaptive_filtered( &self, query: &[f32], k: usize, policy: &AdaptiveEfPolicy, filter: Option<&(dyn Fn(DocId) -> bool + Send + Sync)>, ) -> Result<Vec<ScoredDocument>>` | Performs an adaptive search (Ada-ef) with custom document filter predicate. | `src/hnsw/core_search.rs` |
| `pub async fn search_adaptive_with_stats( &self, query: &[f32], k: usize, policy: &AdaptiveEfPolicy, filter: Option<&(dyn Fn(DocId) -> bool + Send + Sync)>, ) -> Result<(Vec<ScoredDocument>, AdaptiveEfStats)>` | Performs an adaptive search (Ada-ef) returning scored documents along with execution statistics. | `src/hnsw/core_search.rs` |
| `pub fn hnsw_purger(index: Arc<HnswIndex>) -> Arc<dyn Purger>` | Erzeugt einen `Arc<dyn Purger>` für einen HNSW Index. | `src/hnsw/purge.rs` |
| `pub fn new(index: Arc<HnswIndex>) -> Self` | Erzeugt einen neuen `HnswPurger`. | `src/hnsw/purge.rs` |
| `pub fn hnsw_scanner(index: Arc<HnswIndex>) -> Arc<dyn LayerScanner>` | Erzeugt einen `Arc<dyn LayerScanner>` für einen HNSW Index. | `src/hnsw/scanner.rs` |
| `pub fn new(index: Arc<HnswIndex>) -> Self` | Erzeugt einen neuen `HnswScanner`. | `src/hnsw/scanner.rs` |
| `pub fn calibrate( batch: &[&[f32]], quantizer: &crate::quantize::ScalarQuantizer, metric: contextra_types::DistanceMetric, ) -> Self` | Calibrates quantization error statistics across a representative batch of vectors. | `src/hnsw/sq8_bias.rs` |
| `pub fn new(mean_bias: f32, variance_bias: f32, sample_count: u64) -> Self` | Creates a new `Sq8Bias` instance with calibrated parameters. | `src/hnsw/sq8_bias.rs` |
| `pub fn all_doc_ids_and_layers(&self) -> Vec<(DocId, usize)>` | Returns all non-deleted document IDs and their assigned HNSW max layers. | `src/hnsw/types.rs` |
| `pub fn all_doc_ids_from_map(&self) -> Vec<DocId>` | — | `src/hnsw/types.rs` |
| `pub fn check_and_trigger_partial_rebuild(&self)` | — | `src/hnsw/types.rs` |
| `pub fn check_connectivity(&self) -> contextra_types::Result<()>` | — | `src/hnsw/types.rs` |
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
| `pub fn save_sync(&self, path: impl AsRef<std::path::Path>) -> Result<()>` | — | `src/hnsw/types.rs` |
| `pub fn set_entry_point(&self, ep: Option<usize>)` | — | `src/hnsw/types.rs` |
| `pub fn set_fault_injection_insert_target(&self, target: u64)` | Sets the target insertion count for simulating compute_insert fault injection in tests. | `src/hnsw/types.rs` |
| `pub fn set_layer_seed(&self, seed: u64)` | Sets the random seed used for HNSW layer selection during insertions. | `src/hnsw/types.rs` |
| `pub fn set_layer_seed(&self, seed: u64)` | Sets the random seed used for HNSW layer selection during insertions. | `src/hnsw/types.rs` |
| `pub fn set_metrics_sink(&self, sink: std::sync::Arc<dyn contextra_ports::MetricsSink>)` | Sets the metrics sink. | `src/hnsw/types.rs` |
| `pub fn set_metrics_sink(&self, sink: std::sync::Arc<dyn contextra_ports::MetricsSink>)` | Sets the metrics sink. | `src/hnsw/types.rs` |
| `pub fn set_ram_entry_point(&self, ep: Option<usize>)` | — | `src/hnsw/types.rs` |
| `pub fn sq8_bias(&self) -> Sq8Bias` | Returns the calibrated SQ8 quantization bias statistics. | `src/hnsw/types.rs` |
| `pub fn trigger_rebuild_async(&self)` | — | `src/hnsw/types.rs` |
| `pub fn try_new(config: HnswConfig) -> Result<Self>` | Creates a new HNSW index, validating configuration upfront. Uses an internal, non-cryptographic, reproducible default RNG seeded with `DEFA… | `src/hnsw/types.rs` |
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
| `pub fn asymmetric_dist( &self, query: &[f32], quantized: &[u8], metric: DistanceMetric, ) -> contextra_types::Result<f32>` | Computes the asymmetric distance between an exact query and a quantized vector. Optimized for zero allocations via inline dequantization. | `src/quantize/mod.rs` |
| `pub fn calculate_vector_drift(&self, vector: &[f32]) -> f32` | Calculates quantization drift for a single vector as the fraction of dimensions falling outside `[mins[i], maxes[i]]`. This is a **pure cal… | `src/quantize/mod.rs` |
| `pub fn check_drift(&self, vector: &[f32]) -> f32` | Legacy pure drift calculation wrapper for [`calculate_vector_drift`](Self::calculate_vector_drift). # Deprecated Renamed to [`calculate_vec… | `src/quantize/mod.rs` |
| `pub fn dequantize(&self, vector: &[u8]) -> contextra_types::Result<Vec<f32>>` | Dequantizes a `u8` vector back to `f32`. | `src/quantize/mod.rs` |
| `pub fn dimension(&self) -> usize` | Returns the target dimension of the quantizer. | `src/quantize/mod.rs` |
| `pub fn drift_ratio(&self) -> f32` | Returns the fraction of quantized queries that contained values outside the trained min/max range. This is a **stateful metric** reflecting… | `src/quantize/mod.rs` |
| `pub fn inv_scales(&self) -> &[f32]` | Returns a reference to per-dimension inverse scale values. | `src/quantize/mod.rs` |
| `pub fn is_rebuild_required(&self, threshold: f32) -> bool` | Checks if recalibration / index rebuild is required based on cumulative quantization drift ratio. Requires at least 20 queries to avoid fal… | `src/quantize/mod.rs` |
| `pub fn maxes(&self) -> &[f32]` | Returns a reference to per-dimension maximum values. | `src/quantize/mod.rs` |
| `pub fn mins(&self) -> &[f32]` | Returns a reference to per-dimension minimum values. | `src/quantize/mod.rs` |
| `pub fn quantize(&self, vector: &[f32]) -> contextra_types::Result<Vec<u8>>` | Quantizes an `f32` vector to `u8`. | `src/quantize/mod.rs` |
| `pub fn scales(&self) -> &[f32]` | Returns a reference to per-dimension scale values. | `src/quantize/mod.rs` |
| `pub fn symmetric_dist( &self, q1: &[u8], q2: &[u8], metric: DistanceMetric, ) -> contextra_types::Result<f32>` | Computes symmetric (approximate) distance purely in u8. Optimized for zero allocations via inline dequantization. | `src/quantize/mod.rs` |
| `pub fn train(batch: &[&[f32]], dimension: usize) -> Self` | Creates a new ScalarQuantizer trained on a batch of vectors to find per-dimension min/max. For long-lived or growing collections, callers s… | `src/quantize/mod.rs` |
| `pub fn train_with_percentiles( batch: &[&[f32]], dimension: usize, p_low: f32, p_high: f32, ) -> Self` | Creates a new ScalarQuantizer trained on a batch of vectors with custom percentile bounds. | `src/quantize/mod.rs` |
| `pub fn try_train(batch: &[&[f32]], dimension: usize) -> contextra_types::Result<Self>` | Creates a new ScalarQuantizer trained on a batch of vectors to find per-dimension min/max. For long-lived or growing collections, callers s… | `src/quantize/mod.rs` |
| `pub fn try_train_with_percentiles( batch: &[&[f32]], dimension: usize, p_low: f32, p_high: f32, ) -> contextra_types::Result<Self>` | Safely creates a new ScalarQuantizer trained on a batch of vectors using custom percentile clipping bounds. Range `p_low` and `p_high` must… | `src/quantize/mod.rs` |
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
| **Externe Abhängigkeiten (Auswahl)** | `bincode`, `blake3`, `bytes`, `futures`, `itoa`, `parking_lot`, `regex`, `serde`, `serde_json`, `thiserror`, `tracing`, `unicode-segmentation` |

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
pub use inverted::{ text_purger, text_scanner, BM25MorphIndex, InvertedIndex, Language, PurgeReport, TextPurger, TextScanner, };
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

**`DomainVocabulary`** — *Trait für domänenspezifische Vokabularpakete (z. B. Recht und Medizin).* (`src/domain/mod.rs`)

```rust
pub trait DomainVocabulary : Send + Sync {
/// Domänenspezifische Komposita-Bausteine (Ergänzung zum KMU-Basiswörterbuch).
    fn compound_stems(&self) -> &'static [&'static str];
    /// Fachbegriffe, die NICHT als Stopwort behandelt werden dürfen, selbst wenn sie kurz/häufig sind
    /// (z. B. "Klage", "Akte" im juristischen Kontext).
    fn protected_terms(&self) -> &'static [&'static str];
    /// Kurzname für Logging/Konfiguration, z. B. "legal_de", "medical_de".
    fn domain_id(&self) -> &'static str;
}
```

**`MorphologicalTokenizer`** — *Trait für morphologische Zerlegung von Wörtern in ihre Grundbestandteile. # Eingabevertrag Eingegebene Wörter MÜSSEN vor dem Aufruf von `decompose` in Kleinbuchstaben vorliegen und idealerweise mit [`normalize_umlauts`] vorbereitet sein. Bei Großbuchstaben od…* (`src/morphology/compound_splitter.rs`)

```rust
pub trait MorphologicalTokenizer : Send + Sync {
/// Zerlegt ein Token in morphologische Komponenten (Teilwörter).
    fn decompose<'a>(&self, token: &'a str) -> Vec<&'a str>;

    /// Liefert den Sprachcode (z. B. "de").
    fn language(&self) -> &str;
}
```

**`Tokenizer`** — *Trait für Text-Tokenizer mit unterschiedlichen Filter- oder Zerlegungsstrategien.* (`src/tokenizer.rs`)

```rust
pub trait Tokenizer : Send + Sync {
/// Zerlegt den übergebenen Text in kleingeschriebene Wort-Token und filtert Stoppwörter heraus.
    fn tokenize(&self, text: &str) -> Vec<String>;
}
```

### Öffentliche Datentypen — Structs (26)

| Struct | Felder / Form | Beschreibung | Datei |
|---|---|---|---|
| `BM25` | `{ pub k1: f32, pub b: f32, }` | Konfigurationsparameter für das BM25-Scoring-Modell. - `k1`: Steuert die Sättigung der Termhäufigkeit (TF). Standardwert: 1.5. - `b`: Steuert den Ein… | `src/bm25.rs` |
| `BM25F` | `{ pub k1: f32, pub field_weights: Vec<FieldWeight>, cached_weights: Vec<(FieldId, f32, f32)>, }` | Konfiguration für feldgewichtetes BM25F-Scoring (§11.3). | `src/bm25.rs` |
| `BM25MorphIndex` | `{ inner: InvertedIndex<S>, tokenizer: Arc<dyn MorphologicalTokenizer>, }` | Invertierter Index mit morphologischer Zerlegung für deutsche Volltextsuche. | `src/inverted/morph_index.rs` |
| `Bm25CandidateStream` | `{ index: &'a InvertedIndex<S>, query: String, max_seq: Option<u64>, batch_size: usize, buffer: VecDeque<(DocId, f32)>, seen: HashSet<DocId>, fetched_k: usize, yi… }` | Asynchroner Pull-Stream zum bedarfsgerechten Nachladen von BM25-Kandidaten. # Kandidaten-Sortierung & Determinismus Ergebnisse werden strikt determin… | `src/stream.rs` |
| `Bm25Scorer` | `{ index: InvertedIndex<S>, }` | Berechnet Begriffsgewichte und wendet die Standard-BM25-Relevanzbewertung an. `Bm25Scorer` fungiert als primäre Integrationsfassade für den Port [`Te… | `src/lib.rs` |
| `BoundedTopK` | `{ heap: BinaryHeap<Bm25Candidate>, capacity: usize, }` | Begrenzter Min-Heap zur effizienten Verwaltung der K besten Treffer. | `src/wand.rs` |
| `DefaultTokenizer` | `(tuple/unit)` | Standard-Tokenizer basierend auf Unicode-Wortgrenzen und generischen Stoppwörtern. | `src/tokenizer.rs` |
| `FieldWeight` | `{ pub field_id: FieldId, pub weight: f32, pub b: f32, }` | Feldgewichtungskonfiguration für ein bestimmtes Feld in BM25F. | `src/bm25.rs` |
| `GermanCompoundSplitter` | `{ /// Minimum component length for splitting. min_component_len: usize, /// Trie data structure for fast prefix checking and dictionary matching. trie: Trie, }` | Deutscher Komposita-Zerleger basierend auf Dynamischer Programmierung und Fugenanalyse. # Funktionsweise & Algorithmus 1. **Eingebettetes Wörterbuch*… | `src/morphology/compound_splitter.rs` |
| `GermanMorphTokenizer` | `{ splitter: Arc<crate::morphology::GermanCompoundSplitter>, }` | Deutscher Tokenizer mit morphologischer Zerlegung von Wortzusammensetzungen. | `src/tokenizer.rs` |
| `InvertedIndex` | `{ storage: Arc<S>, prefix: Vec<u8>, tokenizer: Arc<dyn Tokenizer>, pub(crate) total_docs: Arc<AtomicU64>, pub(crate) total_tokens: Arc<AtomicU64>, pub(crate) avg… }` | Persistenter invertierter Index über einer Kollektion im Storage-Backend. # Nebenläufigkeit & Sperrhierarchie: 1. `commit_lock` (`parking_lot::Mutex`… | `src/inverted/index_struct.rs` |
| `LegalDomainVocabulary` | `(tuple/unit)` | Deutsches Juristisches Fachvokabular zur Anbindung an [`DomainVocabulary`]. | `src/domain/legal_de.rs` |
| `MedicalDomainVocabulary` | `(tuple/unit)` | Deutsches Medizinisches Fachvokabular zur Anbindung an [`DomainVocabulary`]. | `src/domain/medical_de.rs` |
| `PassthroughTokenizer` | `{ lang: String, }` | Passthrough-Tokenizer für Sprachen, die keine Zerlegung von Zusammensetzungen benötigen. Gibt eingegebene Token unverändert als Einzelkomponente zurü… | `src/morphology/passthrough.rs` |
| `Posting` | `{ /// Dokument-ID des Eintrags. pub doc_id: DocIdRaw, /// Termfrequenz (Häufigkeit des Terms in diesem Dokument). pub tf: u32, /// Gesamtlänge des Dokuments in T… }` | Kompakte Repräsentation eines einzelnen Vorkommens (Posting) im Index. 8-Byte ausgerichtet (`repr(C, align(8))`) für optimale CPU-Cacheline-Ausnutzun… | `src/posting_list.rs` |
| `PostingBlockInfo` | `{ /// Höchste Dokument-ID in diesem Block (ermöglicht schnelles Überspringen). pub max_doc_id: DocIdRaw, /// Maximale Termfrequenz innerhalb dieses Blocks. pub m… }` | Metadaten eines 64er-Posting-Blocks zur Berechnung oberer BM25-Schranken in Block-Max WAND. | `src/posting_list.rs` |
| `PostingList` | `{ postings: Vec<Posting>, blocks: Vec<PostingBlockInfo>, }` | Zusammenhängende, nach DocId sortierte Posting-Liste für einen Begriff. | `src/posting_list.rs` |
| `PurgeReport` | `{ /// Anzahl erfolgreich physisch gelöschter Einheiten. pub units_purged: u64, /// Anzahl fehlgeschlagener Einheiten während der Ausführung. pub units_failed: u6… }` | Detaillierter Ausführungsbericht eines Purge-Laufs mit Einheiten, freigegebenem Speicher und Fehlern. | `src/inverted/purge.rs` |
| `ResidentPostingIndex` | `{ index: RwLock<HashMap<String, Arc<PostingList>>>, }` | Speicherresidenter Lese-Cache für Posting-Listen mit RCU-Verwaltung. | `src/posting_list.rs` |
| `TextIndexMetadata` | `{ /// Gesamtzahl der aktiven Dokumente im Index. pub total_docs: u64, /// Gesamtzahl aller indizierten Token über alle Dokumente. pub total_tokens: u64, /// Fest… }` | Konsolidierte Metadaten des Textindex zur persistenten Speicherung im Schlüssel `meta:stats`. | `src/inverted/types.rs` |
| `TextPurger` | `{ index: Arc<InvertedIndex<S>>, }` | Physischer Purger für den Textindex zur Bereinigung von Storage und Metadaten. | `src/inverted/purge.rs` |
| `TextScanner` | `{ index: Arc<InvertedIndex<S>>, target: Arc<parking_lot::Mutex<Option<ScanTarget>>>, }` | Scanner zur Feststellung verbleibender Restdaten im Textindex. | `src/inverted/scanner.rs` |
| `TokenReductionMetrics` | `{ /// Anzahl ursprünglicher Token vor der morphologischen Zerlegung. pub original_tokens: usize, /// Anzahl resultierender Token nach Zerlegung von Komposita. pu… }` | Metriken zur Erfassung der Token-Expansions- und Zerlegungswirkung. | `src/morphology/metrics.rs` |
| `Trie` | `{ root: TrieNode, }` | Präfixbaum (Trie) für schnelle O(m)-Wörterbuchabfragen (m = Wortlänge). | `src/morphology/trie.rs` |
| `TrieNode` | `{ /// Zeigt an, ob an diesem Knoten ein vollständiges Wort aus dem Wörterbuch endet. pub is_terminal: bool, /// Kindknoten, indiziert nach Folgezeichen. pub chil… }` | Knoten im Präfixbaum zur Repräsentation von Buchstabenfolgen deutscher Wörter. | `src/morphology/trie.rs` |
| `WandSearchResult` | `{ /// Gefundene Trefferdokumente mit Relevanzwert. pub results: Vec<(DocId, f32)>, /// Anzahl tatsächlich voll berechneter Dokumente (Pruning-Effektivitätsmetrik… }` | Ergebnis einer Block-Max-WAND-Suche inklusive Top-K-Treffern und Metadaten über evaluierte Dokumente. | `src/wand.rs` |

### Öffentliche Datentypen — Enums (2)

| Enum | Varianten | Beschreibung | Datei |
|---|---|---|---|
| `Language` | `/// Englischer Standard-Tokenizer (Unicode-Wortgrenzen + Stoppwortfilterung). #[default] English, /// Deutscher morphologischer Tokenizer (Komposita-Zerlegung + Umlaut-Normalisierung). German, /// Be…` | Unterstützte Sprachen und Tokenisierungsstrategien für den invertierten Index. Steuert, welcher Tokenizer für die Dokumentenindizierung und Suchabfra… | `src/inverted/types.rs` |
| `ParsedTextTarget` | `Document(DocId), Collection(String),` | Geparstes Zielobjekt (Dokument-ID oder Kollektion) für den Scanner. | `src/inverted/scanner.rs` |

### Typ-Aliase (3)

| Alias | Definition | Beschreibung |
|---|---|---|
| `DocIdRaw` | `type DocIdRaw= u64` | — |
| `DocIdRaw` | `type DocIdRaw= u128` | — |
| `FieldId` | `type FieldId= u32` | Eindeutige Kennung für Dokumentenfelder im feldgewichteten BM25F-Scoring. |

### Konstanten (7)

| Konstante | Typ | Beschreibung |
|---|---|---|
| `BLOCK_SIZE` | `usize` | Blockgröße für die Block-Max-WAND-Zerlegung (64 Postings pro Block). |
| `BM25_B` | `f32` | Standard-b-Parameter für die Dokumentlängen-Normalisierung in BM25 (0.75). |
| `BM25_K1` | `f32` | Standard-k1-Parameter für die Termfrequenz-Sättigung in BM25 (1.5). |
| `DEFAULT_STREAM_BATCH_SIZE` | `usize` | Standard-Batch-Größe für das Nachladen von Kandidaten (16 Dokumente). |
| `MAX_STAGED_TRANSACTIONS` | `usize` | Maximale Anzahl gleichzeitig ungecommitteter Transaktionen (10.000). |
| `MAX_TEXT_BYTES` | `usize` | Maximale Textgröße pro Dokument (10 MiB) zur Begrenzung von Speicherverbrauch und DoS-Schutz. |
| `POSTING_LIST_V2_MAGIC` | `&[u8` | Magische Bytes für das kompakte V2-Posting-Listen-Format (`PL\x02\x00`). |

### Öffentliche Funktionen (96)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn new(k1: f32, b: f32) -> Result<Self>` | Erzeugt eine neue `BM25`-Instanz mit validierten Hyperparametern. # Fehler Liefert [`ContextraError::InvalidInput`], falls `k1 < 0.0` bzw.… | `src/bm25.rs` |
| `pub fn new(field_id: FieldId, weight: f32, b: f32) -> Result<Self>` | Erzeugt eine neue `FieldWeight`-Konfiguration nach Validierung von `weight` und `b`. | `src/bm25.rs` |
| `pub fn new(k1: f32, field_weights: Vec<FieldWeight>) -> Result<Self>` | Erzeugt ein neues BM25F-Modell mit Feldgewichtungen. | `src/bm25.rs` |
| `pub fn score_term(&self, tf: u32, doc_len: u32, avg_doc_len: f32, df: u32, n: u32) -> f32` | Berechnet den BM25-Score für einen Begriff mit den Parametern dieser Instanz. | `src/bm25.rs` |
| `pub fn score_term(tf: u32, doc_len: u32, avg_doc_len: f32, df: u32, n: u32) -> f32` | Berechnet den BM25-Score für einen Begriff mit den Standardparametern (`k1 = 1.5`, `b = 0.75`). # Parameter * `tf` - Termhäufigkeit (Term F… | `src/bm25.rs` |
| `pub fn score_term( &self, field_term_frequencies: &[(FieldId, u32, u32, f32)], df: u32, n: u32, ) -> f32` | Berechnet den BM25F-Score über mehrere Felder hinweg. | `src/bm25.rs` |
| `pub fn score_term_bm25f( field_term_frequencies: &[(FieldId, u32, u32, f32)], field_weights: &[(FieldId, f32, f32)], k1: f32, df: u32, n: u32, ) -> f32` | Berechnet den feldgewichteten BM25F-Score nach Robertson & Zaragoza (2004). Berechnet zunächst die gewichtete Pseudo-Termfrequenz über alle… | `src/bm25.rs` |
| `pub fn score_term_with_params( tf: u32, doc_len: u32, avg_doc_len: f32, df: u32, n: u32, k1: f32, b: f32, ) -> f32` | Berechnet den BM25-Score mit frei konfigurierbaren Parametern `k1` und `b`. # Ablauf 1. Schnellausstieg mit 0.0, wenn $N = 0$, $df = 0$ ode… | `src/bm25.rs` |
| `pub fn get_legal_compound_stems() -> &'static [&'static str]` | Liefert den statischen Slice aller juristischen Komposita-Wortstämme. | `src/domain/legal_de.rs` |
| `pub fn get_legal_protected_terms() -> &'static [&'static str]` | Liefert den statischen Slice aller juristischen geschützten Begriffe. | `src/domain/legal_de.rs` |
| `pub fn get_medical_compound_stems() -> &'static [&'static str]` | Liefert den statischen Slice aller medizinischen Komposita-Wortstämme. | `src/domain/medical_de.rs` |
| `pub fn get_medical_protected_terms() -> &'static [&'static str]` | Liefert den statischen Slice aller medizinischen geschützten Fachbegriffe. | `src/domain/medical_de.rs` |
| `pub fn commit_lock(&self) -> &parking_lot::Mutex<()>` | Accessor for single-writer commit lock. | `src/inverted/index_struct.rs` |
| `pub async fn delete_document(&self, tx: TxId, doc_id: DocId) -> Result<()>` | Löscht ein Dokument logisch aus dem Index und markiert alle Terme mit Tombstones. | `src/inverted/index_struct.rs` |
| `pub async fn load_stats(&self) -> Result<()>` | Lädt die persistenten Statistiken (`meta:stats`) aus dem Storage in die atomaren In-Memory-Felder. | `src/inverted/index_struct.rs` |
| `pub fn new(storage: Arc<S>, namespace: &str) -> Self` | Erzeugt einen neuen invertierten Index mit englischem Standard-Tokenizer. | `src/inverted/index_struct.rs` |
| `pub fn new_with_language(storage: Arc<S>, namespace: &str, language: Language) -> Self` | Erzeugt einen neuen invertierten Index mit expliziter Sprachkonfiguration. # Language Selection Use `Language::from_iso("de")` for German,… | `src/inverted/index_struct.rs` |
| `pub fn prefix(&self) -> &[u8]` | Accessor for index storage prefix bytes. | `src/inverted/index_struct.rs` |
| `pub async fn resolve_tombstones(&self, tx: TxId) -> Result<u64>` | Löst aufgelaufene Tombstones (`tbs:{doc_id}:{term}`) auf und bereinigt verwaiste Posting-Listen. | `src/inverted/index_struct.rs` |
| `pub async fn search_bm25( &self, query: &str, k: usize, max_seq: Option<u64>, ) -> Result<Vec<(DocId, f32)>>` | Führt eine Volltextsuche mit BM25 aus. | `src/inverted/index_struct.rs` |
| `pub async fn search_bm25_at( &self, query: &str, k: usize, max_seq: Option<u64>, ) -> Result<Vec<(DocId, f32)>>` | Führt eine Volltextsuche mit BM25 zu einer bestimmten MVCC-Snapshot-Sequenznummer aus. | `src/inverted/index_struct.rs` |
| `pub fn storage(&self) -> &Arc<S>` | Accessor for underlying storage engine. | `src/inverted/index_struct.rs` |
| `pub async fn upsert_document(&self, tx: TxId, doc_id: DocId, text: &str) -> Result<()>` | Fügt ein Dokument in den Index ein oder aktualisiert ein bestehendes. # Ablauf 1. Größenprüfung gegen `MAX_TEXT_BYTES`. 2. Tokenisierung un… | `src/inverted/index_struct.rs` |
| `pub fn with_tokenizer(mut self, tokenizer: Arc<dyn Tokenizer>) -> Self` | Konfiguriert einen benutzerdefinierten Tokenizer. | `src/inverted/index_struct.rs` |
| `pub fn new( storage: Arc<S>, namespace: &str, tokenizer: Arc<dyn MorphologicalTokenizer>, ) -> Self` | Erzeugt einen neuen morphologischen Index mit deutschem Standard-Tokenizer. | `src/inverted/morph_index.rs` |
| `pub fn tokenizer(&self) -> &dyn MorphologicalTokenizer` | Liefert eine Referenz auf den konfigurierten morphologischen Tokenizer. | `src/inverted/morph_index.rs` |
| `pub fn is_success(&self) -> bool` | Liefert `true`, wenn die Löschung fehlerfrei und vollständig war. | `src/inverted/purge.rs` |
| `pub fn new() -> Self` | Erzeugt einen leeren Purge-Bericht. | `src/inverted/purge.rs` |
| `pub fn new(index: Arc<InvertedIndex<S>>) -> Self` | Creates a new `TextPurger` for the given index. | `src/inverted/purge.rs` |
| `pub fn purge_collection(&self, tx: TxId, namespace: &str) -> Result<PurgeStats>` | Fast-path purge for an entire collection namespace (`__txt:{collection}:`). | `src/inverted/purge.rs` |
| `pub fn purge_collection_report(&self, tx: TxId, namespace: &str) -> PurgeReport` | Fast-path purge for an entire collection namespace (`__txt:{collection}:`), returning a `PurgeReport`. | `src/inverted/purge.rs` |
| `pub fn purge_document(&self, tx: TxId, doc_id: DocId) -> Result<PurgeStats>` | Purges a single document from the inverted index and recalculates corpus statistics. | `src/inverted/purge.rs` |
| `pub fn purge_document_report(&self, tx: TxId, doc_id: DocId) -> PurgeReport` | Purges a single document from the inverted index and recalculates corpus statistics, returning a detailed `PurgeReport`. | `src/inverted/purge.rs` |
| `pub fn text_purger<S: StorageEngine + 'static>(index: Arc<InvertedIndex<S>>) -> Arc<dyn Purger>` | Erzeugt eine instanziierte Implementierung des [`Purger`]-Ports für den Textindex. | `src/inverted/purge.rs` |
| `pub fn to_purge_stats(&self) -> Result<PurgeStats>` | Konvertiert den Bericht in [`PurgeStats`] oder liefert einen Fehler bei Teilabbrüchen. | `src/inverted/purge.rs` |
| `pub fn new(index: Arc<InvertedIndex<S>>) -> Self` | Erzeugt einen neuen `TextScanner` für den übergebenen invertierten Index. | `src/inverted/scanner.rs` |
| `pub fn new_with_target(index: Arc<InvertedIndex<S>>, target: ScanTarget) -> Self` | Erzeugt einen neuen `TextScanner` mit vorab konfiguriertem Scan-Ziel. | `src/inverted/scanner.rs` |
| `pub fn parse(target_id: &str) -> Self` | Parses a target ID string into document ID or collection namespace. | `src/inverted/scanner.rs` |
| `pub fn set_target(&self, target: ScanTarget)` | Weist dem Scanner ein neues Zielobjekt zu. | `src/inverted/scanner.rs` |
| `pub fn text_scanner<S: StorageEngine + 'static>( index: Arc<InvertedIndex<S>>, ) -> Arc<dyn LayerScanner>` | Erzeugt eine instanziierte Implementierung des [`LayerScanner`]-Ports für den Textindex. | `src/inverted/scanner.rs` |
| `pub fn with_target(self, target: ScanTarget) -> Self` | Konfiguriert das Scan-Ziel im Builder-Muster. | `src/inverted/scanner.rs` |
| `pub fn from_iso(code: &str) -> Self` | Parst einen ISO-639-1 Sprachcode ("de", "de-AT", "en", "en-US"). Bewertet nur das primäre Subtag vor dem ersten Bindestrich. Unbekannte Cod… | `src/inverted/types.rs` |
| `pub fn new(storage: std::sync::Arc<S>, namespace: &str) -> Self` | Erzeugt eine neue `Bm25Scorer`-Instanz über der angegebenen Speicher-Engine. # Ablauf 1. Übernahme der Referenz auf die `StorageEngine`. 2.… | `src/lib.rs` |
| `pub fn extend_vocabulary(&mut self, words: impl IntoIterator<Item = impl AsRef<str>>)` | Erweitert das Wörterbuch des Zerlegers dynamisch um zusätzliche Begriffe. | `src/morphology/compound_splitter.rs` |
| `pub fn min_component_len(&self) -> usize` | Liefert die Mindestlänge für Teilkomponenten. | `src/morphology/compound_splitter.rs` |
| `pub fn new() -> Self` | Erzeugt einen neuen Zerleger mit der Standard-Mindestlänge (3 Zeichen) und Basis-Wortschatz. | `src/morphology/compound_splitter.rs` |
| `pub fn new_with_all_domains() -> Self` | Erzeugt einen Zerleger mit allen eingebauten Fachdomänen (Recht & Medizin). | `src/morphology/compound_splitter.rs` |
| `pub fn with_dictionary(min_len: usize, custom_words: HashSet<String>) -> Self` | Erzeugt einen Zerleger mit benutzerdefiniertem Wörterbuch. | `src/morphology/compound_splitter.rs` |
| `pub fn with_domains(min_len: usize, domains: &[&dyn DomainVocabulary]) -> Self` | Erzeugt einen Zerleger erweitert um spezifische Fachvokabulare (z. B. Recht, Medizin). | `src/morphology/compound_splitter.rs` |
| `pub fn with_min_length(min_len: usize) -> Self` | Erzeugt einen Zerleger mit konfigurierbarer Mindest-Segmentlänge. | `src/morphology/compound_splitter.rs` |
| `pub fn expansion_ratio(&self) -> f32` | Berechnet das Expansionsverhältnis der Token-Menge. Ein Wert > 1,0 bedeutet eine Erhöhung der Tokenanzahl durch Komposita-Splitting. # Ziel… | `src/morphology/metrics.rs` |
| `pub fn new(lang: impl Into<String>) -> Self` | Erzeugt einen neuen Durchleitungs-Tokenizer für den angegebenen ISO-Sprachcode (z. B. "en"). | `src/morphology/passthrough.rs` |
| `pub fn get_german_stopwords() -> &'static HashSet<String>` | Liefert die statische deutsche Stoppwort-Liste. Wird per [`OnceLock`] beim ersten Zugriff lazy initialisiert. Erweitert um hochfrequente de… | `src/morphology/stopwords.rs` |
| `pub fn is_german_stopword(word: &str) -> bool` | Prüft, ob ein kleingeschriebenes, normalisiertes Wort ein deutsches Stoppwort ist. | `src/morphology/stopwords.rs` |
| `pub fn normalize_umlauts(input: &str) -> String` | Normalisiert deutsche Umlaute und Sonderzeichen für robusten Volltextabgleich. # Ersetzungen - `ä` -> `"ae"` - `ö` -> `"oe"` - `ü` -> `"ue"… | `src/morphology/stopwords.rs` |
| `pub fn contains(&self, word: &str) -> bool` | Prüft, ob ein Wort exakt als terminaler Eintrag im Baum enthalten ist. | `src/morphology/trie.rs` |
| `pub fn insert(&mut self, word: &str)` | Fügt ein Wort zeichenweise in den Baum ein und markiert den Endknoten als terminal. | `src/morphology/trie.rs` |
| `pub fn new() -> Self` | Erzeugt einen leeren Präfixbaum. | `src/morphology/trie.rs` |
| `pub fn starts_with(&self, prefix: &str) -> bool` | Prüft, ob im Baum mindestens ein Wort existiert, das mit dem übergebenen Präfix beginnt. | `src/morphology/trie.rs` |
| `pub fn as_slice(&self) -> &[Posting]` | Liefert die Postings als Slice. | `src/posting_list.rs` |
| `pub fn blocks(&self) -> &[PostingBlockInfo]` | Liefert die Block-Max-Metadaten. | `src/posting_list.rs` |
| `pub fn clear(&self)` | Leert den gesamten Cache (z. B. bei vollständigem Index-Neuladen). | `src/posting_list.rs` |
| `pub fn decode_compact(bytes: &[u8]) -> Result<Self, ContextraError>` | Dekodiert eine Posting-Liste aus dem Delta-Varint-Format mit Fallback auf Legacy-Bincode. | `src/posting_list.rs` |
| `pub fn doc_id(&self) -> DocId` | — | `src/posting_list.rs` |
| `pub fn empty() -> Self` | Erzeugt eine leere Posting-Liste. | `src/posting_list.rs` |
| `pub fn encode_compact(&self) -> Result<Vec<u8>, ContextraError>` | Kodiert die Posting-Liste in das kompakte Binärformat mit Delta-Varint-Kompression. | `src/posting_list.rs` |
| `pub fn get(&self, term: &str) -> Option<Arc<PostingList>>` | Liefert eine referenzgezählte Posting-Liste für den Begriff, falls im Cache vorhanden. | `src/posting_list.rs` |
| `pub fn insert_list(&self, term: String, list: PostingList)` | Fügt eine Posting-Liste in den Index ein oder ersetzt sie. | `src/posting_list.rs` |
| `pub fn is_empty(&self) -> bool` | Prüft, ob die Liste leer ist. | `src/posting_list.rs` |
| `pub fn len(&self) -> usize` | Liefert die Anzahl der Einträge in der Liste. | `src/posting_list.rs` |
| `pub fn new(doc_id: DocId, tf: u32, doc_len: u32) -> Self` | — | `src/posting_list.rs` |
| `pub fn new(mut postings: Vec<Posting>) -> Self` | Erzeugt eine sortierte und deduplizierte Posting-Liste und berechnet die Block-Metadaten. | `src/posting_list.rs` |
| `pub fn new() -> Self` | Erzeugt einen leeren residenten Posting-Index. | `src/posting_list.rs` |
| `pub fn remove(&self, doc_id: DocId) -> Self` | Entfernt ein Posting per Copy-On-Write (RCU). | `src/posting_list.rs` |
| `pub fn remove_posting_from_terms(&self, terms: &[String], doc_id: DocId)` | Entfernt eine Dokument-ID aus den Posting-Listen mehrerer Begriffe per RCU. | `src/posting_list.rs` |
| `pub fn remove_terms(&self, terms: &[String])` | Entfernt die angegebenen Begriffe aus dem Cache (z. B. bei Rollback). | `src/posting_list.rs` |
| `pub fn upsert(&self, posting: Posting) -> Self` | Fügt ein Posting per Copy-On-Write (RCU) ein oder aktualisiert einen bestehenden Eintrag. | `src/posting_list.rs` |
| `pub fn upsert_posting(&self, term: &str, posting: Posting)` | Fügt ein einzelnes Posting per RCU (Copy-On-Write) in die Posting-Liste des Begriffs ein. | `src/posting_list.rs` |
| `pub fn is_exhausted(&self) -> bool` | Prüft, ob der Stream vollständig erschöpft ist (keine gepufferten oder nachladbaren Elemente mehr). | `src/stream.rs` |
| `pub fn new( index: &'a InvertedIndex<S>, query: impl Into<String>, max_seq: Option<u64>, ) -> Self` | Erzeugt einen neuen Kandidaten-Stream für die übergebene Abfrage und Index-Snapshot. # Parameter - `index`: Referenz auf den invertierten I… | `src/stream.rs` |
| `pub async fn next_batch(&mut self) -> Result<Vec<(DocId, f32)>>` | Holt den nächsten Batch von Kandidaten-Dokumenten (bis zu `batch_size`). Liefert einen leeren Vektor, sobald der Stream erschöpft ist. # Ab… | `src/stream.rs` |
| `pub async fn next_doc(&mut self) -> Result<Option<DocId>>` | Holt die nächste einzelne Dokument-ID aus dem Stream. Liefert `Ok(None)`, wenn keine weiteren Treffer vorliegen. | `src/stream.rs` |
| `pub fn with_batch_size(mut self, n: usize) -> Self` | Konfiguriert eine benutzerdefinierte Batch-Größe (mindestens 1). | `src/stream.rs` |
| `pub fn yielded(&self) -> usize` | Liefert die Gesamtzahl aller bisher an den Aufrufer herausgegebenen Treffer. | `src/stream.rs` |
| `pub fn compile_protected_regex(pattern: &str) -> Result<Regex>` | Kompiliert ein reguläres Muster für geschützte Token mit Fail-Closed-Fehlerbehandlung. # Fehler Gibt [`ContextraError::Text`] zurück, falls… | `src/tokenizer.rs` |
| `pub fn get_protected_regex() -> Result<&'static Regex>` | Liefert den gecachten regulären Standardausdruck für geschützte URLs und E-Mail-Adressen. | `src/tokenizer.rs` |
| `pub fn new() -> Self` | — | `src/tokenizer.rs` |
| `pub fn new() -> Self` | — | `src/tokenizer.rs` |
| `pub fn tokenize(text: &str) -> Vec<String>` | Hilfsfunktion zur Tokenisierung mit dem [`DefaultTokenizer`]. | `src/tokenizer.rs` |
| `pub fn try_new() -> Result<Self>` | Fallible Konstruktion eines `DefaultTokenizer` mit Validierung des regulären Schutzmusters. | `src/tokenizer.rs` |
| `pub fn try_new() -> Result<Self>` | Erzeugt eine neue Instanz des deutschen morphologischen Tokenizers. | `src/tokenizer.rs` |
| `pub async fn block_max_wand_search<S: StorageEngine>( storage: &S, terms: &[String], resident_index: &crate::posting_list::ResidentPostingIndex, prefix_helper: impl Fn(&str) -> Vec<u8>, tombstone_key_helper: impl Fn(DocId, &str) -> Vec<u8>, doc_len_key_helper…` | — | `src/wand.rs` |
| `pub fn into_sorted_vec(self) -> Vec<(DocId, f32)>` | Konvertiert den Heap in einen nach Relevanz absteigend sortierten Ergebnisvektor. | `src/wand.rs` |
| `pub fn min_threshold(&self) -> f32` | Liefert den aktuellen Minimal-Score (Schwelle), den ein Kandidat übertreffen muss. | `src/wand.rs` |
| `pub fn new(capacity: usize) -> Self` | Erzeugt einen neuen `BoundedTopK`-Puffer mit maximaler Kapazität (`MAX_SEARCH_K`). | `src/wand.rs` |
| `pub fn push(&mut self, doc_id: DocId, score: f32)` | Fügt einen neuen Kandidaten ein, wenn er besser als der schlechteste bisherige Treffer ist. | `src/wand.rs` |


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
pub use cascade::{ cascade_invalidate_edges_for_superseded_doc, cascade_invalidate_hyperedges_for_superseded_doc, CascadeDeletionRecord, CascadeInvalidationReport, CascadeReport, CascadeStatus, CascadeTicket, DeletionProof, GraphGarbageCollection, HyperedgeCascadeReport, MAX_HYPEREDGE_CASCADE_FANOUT, };
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
| `ppr-forward-push` | `[]` |
| `k-path-diffusion` | `[]` |
| `apprh-diffusion` | `[]` |
| `physio-percolation` | `["graph-connectivity-health"]` |

### Schnittstellen — Public Traits (8)

**`ApprhFlipGate`** — ** (`src/apprh/shadow.rs`)

```rust
pub trait ApprhFlipGate {
/// Evaluates whether APPRH meets default flip criteria.
    fn should_flip(&self, report: &ApprhShadowComparison) -> bool;
}
```

**`ContradictionDetector`** — ** (`src/consistency_enforcement.rs`)

```rust
pub trait ContradictionDetector {
/// Prüft, ob zwei Aussagen im Widerspruch zueinander stehen.
    fn conflicts(&self, a: &EdgeAssertion, b: &EdgeAssertion) -> bool;
}
```

**`DefaultFlipGate`** — ** (`src/tl_hfd/shadow.rs`)

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

**`EntityExtractor`** — ** (`src/entity_extraction.rs`)

```rust
pub trait EntityExtractor : Send + Sync {
/// Extrahiert Relationen aus dem gegebenen Freitext.
    fn extract(&self, text: &str) -> Vec<ExtractedRelation>;
}
```

**`GraphGarbageCollection`** — ** (`src/cascade.rs`)

```rust
pub trait GraphGarbageCollection {
/// Sweeps orphaned hyperedges whose participants no longer exist or are fully tombstoned.
    fn sweep_orphans(&self, wal_tx: TxId) -> Result<usize>;
}
```

**`GraphIndexExt`** — ** (`src/lib.rs`)

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

**`KPathDiffusion`** — ** (`src/path_rag/k_path.rs`)

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

**`PathGraph`** — ** (`src/path_rag/mod.rs`)

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

### Öffentliche Datentypen — Structs (67)

| Struct | Felder / Form | Beschreibung | Datei |
|---|---|---|---|
| `AgentStateNode` | `{ pub step_id: NodeIdx, /// LLM prompt of this step. pub prompt: String, /// LLM response of this step. pub response: String, /// Optional MVCC snapshot referenc… }` | — | `src/session_dag.rs` |
| `ApprhGateMonitor` | `{ config: ApprhGateMonitorConfig, gate: Box<dyn ApprhFlipGate + Send + Sync>, state: Mutex<MonitorState>, }` | — | `src/apprh/gate_monitor.rs` |
| `ApprhGateMonitorConfig` | `{ /// Maximum capacity of the sliding window for historical observations. pub window_size: usize, /// Number of consecutive valid observations required for produ… }` | — | `src/apprh/gate_monitor.rs` |
| `ApprhGateMonitorSnapshot` | `{ /// Total number of observations received since creation or last reset. pub total_observations: u64, /// Current count of consecutive passed observations (stre… }` | — | `src/apprh/gate_monitor.rs` |
| `ApprhObservationRecord` | `{ /// Maximum absolute diff from report. pub max_abs_diff: f32, /// Top-k Jaccard overlap ratio from report. pub top_k_overlap: f32, /// Boolean indicator whethe… }` | — | `src/apprh/gate_monitor.rs` |
| `ApprhParams` | `{ /// Base PPR parameters ($\alpha$ teleport, $\epsilon$ convergence threshold). pub ppr: PprParams, /// Hyperedge decay factor ($\beta \in (0.0, 1.0]$, default:… }` | — | `src/apprh/params.rs` |
| `ApprhSelector` | `{ config: ApprhSelectorConfig, monitor: Arc<ApprhGateMonitor>, mode: RwLock<ApprhMode>, call_counter: AtomicU64, }` | — | `src/apprh/selector.rs` |
| `ApprhSelectorConfig` | `{ /// Sampling rate for shadow evaluations: 1 = every call, N = every N-th call. pub sample_interval: u64, /// Maximum number of seed nodes allowed for shadow ev… }` | — | `src/apprh/selector.rs` |
| `ApprhShadowComparison` | `{ /// Maximum absolute score difference across top-k union nodes. pub max_abs_diff: f32, /// Jaccard overlap ratio between top-k entity ID sets [0.0, 1.0]. pub t… }` | — | `src/apprh/shadow.rs` |
| `ArcSlice` | `{ shared: Arc<[T]>, offset: usize, len: usize, }` | — | `src/arc_slice.rs` |
| `CascadeDeletionRecord` | `{ pub doc_id: DocId, pub tombstoned_count: usize, pub wal_seq: u64, }` | — | `src/cascade.rs` |
| `CascadeInvalidationReport` | `{ /// Number of edges tombstoned as a result of the superseded document. pub tombstoned_edge_count: usize, /// Unique node (entity) IDs affected by the tombstone… }` | — | `src/cascade.rs` |
| `CascadeTicket` | `{ pub id: u64, pub doc_id: DocId, pub status: CascadeStatus, pub proof: Option<CascadeDeletionRecord>, }` | — | `src/cascade.rs` |
| `CommunityAssignment` | `{ /// Entity ID. pub entity_id: EntityId, /// Community ID (represented as a 64-bit integer, initialized from the seed EntityId). pub community_id: u64, /// Comm… }` | — | `src/community.rs` |
| `CommunityDetectionConfig` | `{ /// Maximum number of propagation / modularity optimization iterations. pub max_iterations: u32, /// Seed for deterministic node traversal shuffling. pub seed:… }` | — | `src/community.rs` |
| `ConflictPattern` | `{ /// Blake3-Hash über die normalisierte (subject, predicate, object)-Tripel-Signatur der widersprüchlichen Aussage. pub pattern_hash: [u8; 32], /// Anzahl der b… }` | — | `src/consistency_enforcement.rs` |
| `ConsistencyEnforcer` | `{ patterns: HashMap<[u8; 32], ConflictPattern>, suppression_threshold: u32, }` | — | `src/consistency_enforcement.rs` |
| `ConsolidationNodesGuard` | `{ entities: Vec<EntityId>, }` | — | `src/hyperedge.rs` |
| `CooccurrenceSignal` | `{ pub from: EntityId, pub to: EntityId, pub co_activation: f32, }` | — | `src/edge_reinforcement_buffer.rs` |
| `CsrEdge` | `{ pub target: EntityId, pub weight: f32, pub edge_type: EdgeType, #[cfg(feature = "edge-reinforcement-learning")] pub cooccurrence_weight: f32, // w_ij, initiali… }` | — | `src/csr/types.rs` |
| `CsrGraph` | `{ pub config: CsrGraphConfig, pub inner: Arc<ArcSwap<GraphInner>>, pub write_state: Arc<Mutex<GraphInner>>, pub storage: Option<Arc<dyn StorageEngine>>, pub last… }` | — | `src/csr/graph_write/mod.rs` |
| `CsrGraphConfig` | `{ pub rebuild_threshold: usize, pub max_compaction_peak_memory_mb: Option<usize>, pub resource_tracker: Option<Arc<ResourceTracker>>, }` | — | `src/csr/types.rs` |
| `DagEdge` | `{ pub parent: NodeIdx, pub child: NodeIdx, /// Label (e.g. "explore", "reject", "main") pub label: String, }` | — | `src/session_dag.rs` |
| `DefaultApprhFlipGate` | `(tuple/unit)` | — | `src/apprh/shadow.rs` |
| `DeletedView` | `{ deleted_nodes: HashSet<usize>, }` | — | `src/ppr/types.rs` |
| `DocEdgeIndex` | `{ index: RwLock<AHashMap<DocId, AHashSet<EdgeId>>>, }` | — | `src/provenance.rs` |
| `EdgeAssertion` | `{ /// Subjekt-Knoten der Aussage. pub subject: NodeIdx, /// Hash des Prädikats/Relationsnamens. pub predicate_hash: [u8; 32], /// Repräsentation des Objekts/Ziel… }` | — | `src/consistency_enforcement.rs` |
| `EdgeProvenance` | `{ /// Identifikator der Kante (from_entity_id, to_entity_id). pub edge_id: EdgeId, /// Dokumente deren Inhalt zu dieser Kante beigetragen hat. pub source_doc_ids… }` | — | `src/provenance.rs` |
| `EdgeReinforcementBuffer` | `{ cooccurrence: Mutex<Vec<CooccurrenceSignal>>, traversal: Mutex<Vec<TraversalSignal>>, }` | — | `src/edge_reinforcement_buffer.rs` |
| `EdgeReinforcementConfig` | `{ pub eta: f32, pub delta: f32, pub w_max: f32, pub rho: f32, pub q: f32, pub alpha: f32, }` | — | `src/edge_reinforcement.rs` |
| `ExactPredicateConflictDetector` | `(tuple/unit)` | — | `src/consistency_enforcement.rs` |
| `ExtractedRelation` | `{ /// Name / Identifikator der Quell-Entität (Subjekt). pub source: String, /// Name / Identifikator der Ziel-Entität (Objekt). pub target: String, /// Typ / Bez… }` | — | `src/entity_extraction.rs` |
| `GraphInner` | `{ pub(crate) id_map: HashMap<EntityId, InternalIndex>, pub(crate) reverse_map: Vec<EntityId>, pub(crate) entities: Vec<Entity>, pub(crate) communities: HashMap<E… }` | — | `src/csr/inner/mod.rs` |
| `GraphPath` | `{ /// Knoten-Sequenz vom Start zum Ziel. pub nodes: Vec<EntityId>, /// Kantengewichte entlang des Pfades (len = nodes.len() - 1). pub edge_weights: Vec<f32>, ///… }` | — | `src/path_rag/mod.rs` |
| `GraphStats` | `{ /// Number of active/committed nodes \|V\|. pub node_count: usize, /// Number of directed edges \|E\|. pub edge_count: usize, /// Average outgoing degree \|E\| / \|V\|… }` | — | `src/ppr/cost.rs` |
| `HyperEdge` | `{ /// Eindeutige ID der Hyperkante. pub id: HyperEdgeId, /// Prädikat/Typ der Hyperkante. pub predicate: EdgeType, /// Liste der Teilnehmer mit ihren Rollen. ///… }` | — | `src/hyperedge.rs` |
| `HyperEdgeCandidate` | `{ /// Canonical sorted list of participating entity IDs. pub participants: Vec<EntityId>, /// Number of distinct documents in which this exact entity combination… }` | — | `src/hyperedge_suggest.rs` |
| `HyperEdgeView` | `{ pub edge: &'a HyperEdge, pub participants: ArcSlice<RoleBinding>, }` | — | `src/hyperedge.rs` |
| `HyperedgeCascadeReport` | `{ /// Hyperedges tombstoned synchronously in this cascade run (up to MAX_HYPEREDGE_CASCADE_FANOUT). pub invalidated: Vec<crate::hyperedge::HyperEdgeId>, /// Hype… }` | — | `src/cascade.rs` |
| `KPathConfig` | `{ /// Target number of paths to discover. pub k: usize, /// Maximum number of distinct graph nodes visited across forward/backward search. pub max_visited_nodes:… }` | — | `src/path_rag/k_path.rs` |
| `KPathResult` | `{ /// Up to `k` discovered paths between source and target. pub paths: Vec<Vec<EntityId>>, /// Indicates whether the search was halted due to `max_visited_nodes`… }` | — | `src/path_rag/k_path.rs` |
| `MemoryEstimate` | `{ pub shared_payload_bytes: usize, pub private_bytes: usize, }` | — | `src/csr/inner/estimate.rs` |
| `NodesGuard` | `{ tree: &'a SessionBranchTree, _nodes_guard: RwLockReadGuard<'a, HashMap<NodeIdx, AgentStateNode>>, }` | — | `src/session_dag.rs` |
| `NodesWriteGuard` | `{ tree: &'a SessionBranchTree, _nodes_guard: RwLockWriteGuard<'a, HashMap<NodeIdx, AgentStateNode>>, }` | — | `src/session_dag.rs` |
| `PathRAGConfig` | `{ /// Maximale Suchtiefe (Hop-Limit). pub max_hops: usize, /// Sufficiency-Schwelle: Pfade unter dieser Konfidenz werden gefiltert. pub sufficiency_threshold: f6… }` | — | `src/path_rag/mod.rs` |
| `PathRAGEngine` | `{ graph: G, pub config: PathRAGConfig, }` | — | `src/path_rag/mod.rs` |
| `PercolationConfig` | `{ /// Kritischer Perkolationsschwellenwert θ_perc. Default: 0.7. pub critical_threshold: f32, /// Similarity-Schwellenwert für Re-Bonding θ_bond. Default: 0.85.… }` | — | `src/percolation.rs` |
| `PersistedEdgePayload` | `{ pub weight: f32, #[serde(default, alias = "valid_from")] pub tx_valid_from: Option<TxId>, #[serde(default, alias = "valid_to")] pub tx_valid_to: Option<TxId>,… }` | — | `src/csr/types.rs` |
| `PprCandidateStream` | `{ graph: &'a G, seeds: Vec<EntityId>, config: PprConfig, batch_size: usize, exclude_seeds: bool, ranked: Option<VecDeque<(EntityId, f32)>>, yielded: usize, }` | — | `src/ppr_stream.rs` |
| `PprContext` | `{ pub(crate) valid_seeds: Vec<usize>, pub(crate) out_weight_sums: Vec<f32>, pub(crate) ranks: Vec<f32>, pub(crate) next_ranks: Vec<f32>, }` | — | `src/ppr/types.rs` |
| `PprCostCalibrator` | `{ samples: Arc<Mutex<Vec<PprCostSample>>>, }` | — | `src/ppr/cost.rs` |
| `PprCostEstimate` | `{ /// Estimated edge accesses for Andersen-Chung-Lang ForwardPush: $O(1 / (\epsilon \cdot \alpha))$. pub forward_push: f64, /// Estimated edge accesses for Dense… }` | — | `src/ppr/cost.rs` |
| `PprCostSample` | `{ /// Algorithm evaluated. pub algorithm: PprAlgorithm, /// Estimated cost in edge access units. pub estimated_cost: f64, /// Measured edge accesses (counted ope… }` | — | `src/ppr/cost.rs` |
| `PprParams` | `{ /// Teleport probability ($\alpha$, default: 0.15). pub alpha: f32, /// Convergence error tolerance threshold ($\epsilon$, default: 1e-4). pub epsilon: f32, //… }` | — | `src/path_rag/mod.rs` |
| `RoleInterner` | `{ forward: HashMap<String, RoleId>, backward: HashMap<RoleId, String>, next_id: AtomicU32, }` | — | `src/hyperedge.rs` |
| `RuleBasedEntityExtractor` | `{ /// Minimale Konfidenzschwelle. Ergebnisse unterhalb dieser Schwelle werden nicht zurückgegeben. min_confidence: f32, /// Registrierte Extraktionsmuster. patte… }` | — | `src/entity_extraction.rs` |
| `SessionBranchTree` | `{ nodes: RwLock<HashMap<NodeIdx, AgentStateNode>>, edges: RwLock<Vec<DagEdge>>, active_head: RwLock<NodeIdx>, next_id: Arc<std::sync::atomic::AtomicU64>, }` | — | `src/session_dag.rs` |
| `SessionDagStepOptions` | `{ pub prompt: String, pub response: String, pub snapshot_tx_id: Option<TxId>, pub tool_outputs: Vec<String>, pub label: &'a str, }` | — | `src/session_dag.rs` |
| `ShadowComparison` | `{ /// Maximum absolute score difference across top-k union nodes. pub max_abs_diff: f32, /// Jaccard overlap ratio between top-k entity ID sets [0.0, 1.0]. pub t… }` | — | `src/tl_hfd/shadow.rs` |
| `ShadowDiscrepancyReport` | `{ /// Number of evaluated query pairs (ForwardPush vs TL-HFD). pub sample_count: u64, /// Mean Jaccard similarity of top-k entity ID sets between algorithms. pub… }` | — | `src/tl_hfd/shadow.rs` |
| `SnapshotPathGraph` | `{ inner: &'a GraphInner, as_of_tx: TxId, }` | — | `src/path_rag/snapshot.rs` |
| `StarExpansionIterator` | `{ inner: arc_swap::Guard<std::sync::Arc<crate::csr::inner::GraphInner>>, hyperedge_keys: Vec<crate::hyperedge::HyperEdgeId>, current_hyperedge_idx: usize, curren… }` | — | `src/community.rs` |
| `TlHfdFlipGate` | `(tuple/unit)` | — | `src/tl_hfd/shadow.rs` |
| `TlHfdParams` | `{ /// Regularization parameter $\sigma > 0$ controlling diffusion scale (default: 0.1). pub sigma: f32, /// Seed injection multiplier $\delta \ge 2.0$ (default:… }` | — | `src/tl_hfd/params.rs` |
| `TraversalSignal` | `{ pub from: EntityId, pub to: EntityId, pub path_length: usize, }` | — | `src/edge_reinforcement_buffer.rs` |
| `ValidatedHyperEdgeCandidate` | `{ /// The underlying co-occurrence candidate. pub candidate: HyperEdgeCandidate, /// Suggested semantic predicate string. pub predicate: String, /// LLM validati… }` | — | `src/hyperedge_suggest.rs` |
| `VirtualHyperedgeNode` | `{ /// ID der zugrundeliegenden Hyperkante. pub hyperedge_id: crate::hyperedge::HyperEdgeId, }` | — | `src/community.rs` |

### Öffentliche Datentypen — Enums (6)

| Enum | Varianten | Beschreibung | Datei |
|---|---|---|---|
| `ApprhError` | `/// Invalid parameter value provided to APPRH. #[error("Invalid parameter: {0}")] InvalidParameter(String), /// Empty seed list provided to APPRH. #[error("Seed list cannot be empty")] EmptySeeds, //…` | — | `src/apprh/error.rs` |
| `ApprhMode` | `/// APPRH disabled; authoritative Forward-Push PPR is returned. Off, /// APPRH shadow mode (default): returns authoritative Forward-Push PPR result, /// and periodically evaluates shadow comparisons…` | — | `src/apprh/selector.rs` |
| `CascadeStatus` | `/// Invalidation task is queued and pending. Pending, /// Invalidation task is currently processing. InFlight, /// Invalidation completed successfully with a deletion proof. Completed, /// Invalidati…` | — | `src/cascade.rs` |
| `EdgeType` | `#[default] Default,` | — | `src/csr/types.rs` |
| `HyperedgeSuggestError` | `/// Candidate count exceeds the maximum LLM calls allowed per cycle. #[error("LLM validation budget exceeded: {0} candidates exceed limit")] LlmValidationBudgetExceeded(usize),` | — | `src/hyperedge_suggest.rs` |
| `TlHfdError` | `/// Invalid parameter value provided to TL-HFD. #[error("Invalid parameter: {0}")] InvalidParameter(String), /// Non-finite value encountered during diffusion calculation. #[error("Non-finite value e…` | — | `src/tl_hfd/error.rs` |

### Typ-Aliase (4)

| Alias | Definition | Beschreibung |
|---|---|---|
| `CascadeReport` | `type CascadeReport= HyperedgeCascadeReport` | — |
| `DeletionProof` | `type DeletionProof= CascadeDeletionRecord` | — |
| `EdgeId` | `type EdgeId= (EntityId, EntityId)` | — |
| `NodeIdx` | `type NodeIdx= u64` | — |

### Konstanten (17)

| Konstante | Typ | Beschreibung |
|---|---|---|
| `CASCADE_QUEUE_PREFIX` | `&str` | — |
| `DEFAULT_HYPEREDGE_CASCADE_FANOUT_LIMIT` | `usize` | — |
| `DEFAULT_PPR_STREAM_BATCH_SIZE` | `usize` | — |
| `DEFAULT_SUFFICIENCY_THRESHOLD` | `f64` | — |
| `DEFAULT_SUPPRESSION_THRESHOLD` | `u32` | — |
| `HYPEREDGE_BY_ENTITY_PREFIX` | `&str` | — |
| `HYPEREDGE_PREFIX` | `&str` | — |
| `MAX_DAG_STRING_BYTES` | `usize` | — |
| `MAX_DAG_TRAVERSAL_DEPTH` | `usize` | — |
| `MAX_HYPEREDGE_CASCADE_FANOUT` | `usize` | — |
| `MAX_RELATE_PARTICIPANTS` | `usize` | — |
| `MAX_TRAVERSAL_HOPS` | `u8` | — |
| `MAX_VISITED_NODES` | `usize` | — |
| `MIN_AGREEMENT_THRESHOLD` | `f32` | — |
| `MIN_SHADOW_SAMPLES` | `u64` | — |
| `SCORE_DECAY` | `f32` | — |
| `WALLCLOCK_TX_HEURISTIC_MIN` | `u64` | — |

### Öffentliche Funktionen (223)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn run_apprh_push<G: PathGraph>( graph: &G, seeds: &[EntityId], params: &ApprhParams, ) -> Result<AHashMap<EntityId, f32>, ApprhError>` | — | `src/apprh/diffusion.rs` |
| `pub fn config(&self) -> &ApprhGateMonitorConfig` | — | `src/apprh/gate_monitor.rs` |
| `pub fn new(config: ApprhGateMonitorConfig) -> Result<Self, ApprhError>` | — | `src/apprh/gate_monitor.rs` |
| `pub fn observe(&self, report: &ApprhShadowComparison)` | — | `src/apprh/gate_monitor.rs` |
| `pub fn production_ready(&self) -> bool` | — | `src/apprh/gate_monitor.rs` |
| `pub fn reset(&self)` | — | `src/apprh/gate_monitor.rs` |
| `pub fn snapshot(&self) -> ApprhGateMonitorSnapshot` | — | `src/apprh/gate_monitor.rs` |
| `pub fn validate(&self) -> Result<(), ApprhError>` | — | `src/apprh/gate_monitor.rs` |
| `pub fn with_gate<G>(config: ApprhGateMonitorConfig, gate: G) -> Result<Self, ApprhError> where G: ApprhFlipGate + Send + Sync + 'static,` | — | `src/apprh/gate_monitor.rs` |
| `pub fn apprh_diffusion( &self, seeds: &[EntityId], params: &ApprhParams, ) -> Result<AHashMap<EntityId, f32>, ApprhError>` | — | `src/apprh/mod.rs` |
| `pub fn apprh_local<G: PathGraph>( graph: &G, seeds: &[EntityId], params: &ApprhParams, ) -> Result<AHashMap<EntityId, f32>, ApprhError>` | — | `src/apprh/mod.rs` |
| `pub fn apprh_shadow_compare( &self, seeds: &[EntityId], ppr: &PprParams, apprh_params: &ApprhParams, top_k: usize, ) -> Result<ApprhShadowComparison, ApprhError>` | — | `src/apprh/mod.rs` |
| `pub fn forward_push_apprh<G: PathGraph>( graph: &G, seeds: &[EntityId], params: &PprParams, hyperedge_decay_factor: f32, ) -> Result<AHashMap<EntityId, f32>, contextra_types::ContextraError>` | — | `src/apprh/mod.rs` |
| `pub fn validate(&self) -> Result<(), ApprhError>` | — | `src/apprh/params.rs` |
| `pub fn demote_to_shadow(&self)` | — | `src/apprh/selector.rs` |
| `pub fn evaluate_and_run<G: PathGraph>( &self, graph: &G, seeds: &[EntityId], ppr_params: &PprParams, apprh_params: &ApprhParams, ) -> Result<Vec<(EntityId, f32)>, ApprhError>` | — | `src/apprh/selector.rs` |
| `pub fn mode(&self) -> ApprhMode` | — | `src/apprh/selector.rs` |
| `pub fn monitor(&self) -> &ApprhGateMonitor` | — | `src/apprh/selector.rs` |
| `pub fn new(config: ApprhSelectorConfig, monitor: Arc<ApprhGateMonitor>) -> Self` | — | `src/apprh/selector.rs` |
| `pub fn promote_to_production(&self) -> Result<(), ApprhError>` | — | `src/apprh/selector.rs` |
| `pub fn set_mode(&self, new_mode: ApprhMode) -> Result<(), ApprhError>` | — | `src/apprh/selector.rs` |
| `pub fn with_default_monitor(config: ApprhSelectorConfig) -> Result<Self, ApprhError>` | — | `src/apprh/selector.rs` |
| `pub fn shadow_compare_forward_push_vs_apprh<G: PathGraph>( graph: &G, seeds: &[EntityId], ppr: &PprParams, apprh_params: &ApprhParams, top_k: usize, ) -> Result<ApprhShadowComparison, ApprhError>` | — | `src/apprh/shadow.rs` |
| `pub fn as_arc(&self) -> &Arc<[T]>` | — | `src/arc_slice.rs` |
| `pub fn as_slice(&self) -> &[T]` | — | `src/arc_slice.rs` |
| `pub fn from_vec(vec: Vec<T>) -> Self` | — | `src/arc_slice.rs` |
| `pub fn is_empty(&self) -> bool` | — | `src/arc_slice.rs` |
| `pub fn len(&self) -> usize` | — | `src/arc_slice.rs` |
| `pub fn new(shared: Arc<[T]>) -> Self` | — | `src/arc_slice.rs` |
| `pub fn offset(&self) -> usize` | — | `src/arc_slice.rs` |
| `pub fn slice(&self, range: impl RangeBounds<usize>) -> Self` | — | `src/arc_slice.rs` |
| `pub fn try_slice(&self, range: impl RangeBounds<usize>) -> Option<Self>` | — | `src/arc_slice.rs` |
| `pub async fn cascade_invalidate_edges_for_superseded_doc( graph: &CsrGraph, superseded_doc_id: DocId, wal_seq: u64, ) -> Result<CascadeInvalidationReport>` | Tombstoniert alle Kanten, die von `superseded_doc_id` abgeleitet wurden, als Reaktion auf ein Supersedes-Ereignis. Idempotent: mehrfacher A… | `src/cascade.rs` |
| `pub async fn cascade_invalidate_hyperedges_for_superseded_doc( graph: &CsrGraph, superseded_doc_id: DocId, wal_seq: u64, ) -> Result<HyperedgeCascadeReport>` | Cascade invalidates hyperedges derived from `superseded_doc_id` with fan-out protection and recursive super-edge ancestor invalidation. Up… | `src/cascade.rs` |
| `pub fn topological_sort_hyperedge_closure( graph: &CsrGraph, closure_nodes: &[crate::hyperedge::HyperEdgeId], ) -> Vec<crate::hyperedge::HyperEdgeId>` | — | `src/cascade.rs` |
| `pub fn verify_doc_hyperedge_deletion_completeness( graph: &CsrGraph, target_doc_id: DocId, ) -> Result<()>` | — | `src/cascade.rs` |
| `pub async fn detect_communities( graph: &CsrGraph, config: &CommunityDetectionConfig, ) -> Result<Vec<CommunityAssignment>>` | Detects semantic communities in the given CSR graph using the deterministic Leiden algorithm. # Determinism and Reproducibility Guarantees… | `src/community.rs` |
| `pub fn new(hyperedge_id: u64) -> Self` | — | `src/community.rs` |
| `pub fn new(graph: &CsrGraph) -> Self` | — | `src/community.rs` |
| `pub fn next_with_weight(&mut self) -> Option<(EntityId, VirtualHyperedgeNode, f32)>` | — | `src/community.rs` |
| `pub fn active_patterns(&self) -> impl Iterator<Item = &ConflictPattern>` | — | `src/consistency_enforcement.rs` |
| `pub fn check_before_insert(&mut self, assertion: &EdgeAssertion) -> Option<ConflictPattern>` | — | `src/consistency_enforcement.rs` |
| `pub fn detect_contradiction<D: ContradictionDetector>( &self, detector: &D, a: &EdgeAssertion, b: &EdgeAssertion, ) -> bool` | — | `src/consistency_enforcement.rs` |
| `pub fn get_pattern(&self, pattern_hash: &[u8; 32]) -> Option<&ConflictPattern>` | — | `src/consistency_enforcement.rs` |
| `pub fn is_suppressed(&self, pattern_hash: [u8; 32]) -> bool` | — | `src/consistency_enforcement.rs` |
| `pub fn new(suppression_threshold: u32) -> Self` | — | `src/consistency_enforcement.rs` |
| `pub fn pattern_hash(&self) -> [u8` | — | `src/consistency_enforcement.rs` |
| `pub fn record_contradiction( &mut self, pattern_hash: [u8; 32], at_tx: TxId, ) -> &ConflictPattern` | — | `src/consistency_enforcement.rs` |
| `pub fn suggest_tombstone_candidates( &self, csr_edges_matching_pattern: &[EdgeId], ) -> Vec<EdgeId>` | — | `src/consistency_enforcement.rs` |
| `pub fn suppression_threshold(&self) -> u32` | — | `src/consistency_enforcement.rs` |
| `pub async fn delete_edge_persistence<S: StorageEngine + ?Sized>( &self, storage: &S, tx: TxId, from: &EntityId, to: &EntityId, ) -> Result<()>` | Löscht eine einzelne Edge aus dem übergebenen Storage. | `src/csr/graph_persist.rs` |
| `pub async fn load_from_storage<S: StorageEngine + ?Sized>(storage: &S) -> Result<Self>` | Lädt den kompletten Graph-Zustand aus dem Storage (beim Startup). | `src/csr/graph_persist.rs` |
| `pub async fn persist_edge<S: StorageEngine + ?Sized>( &self, storage: &S, tx: TxId, from: &EntityId, to: &EntityId, payload: &PersistedEdgePayload, ) -> Result<()>` | Persistiert eine einzelne Edge in den übergebenen Storage. | `src/csr/graph_persist.rs` |
| `pub async fn persist_entity<S: StorageEngine + ?Sized>( &self, storage: &S, tx: TxId, entity: &Entity, ) -> Result<()>` | Persistiert eine einzelne Entity in den übergebenen Storage. | `src/csr/graph_persist.rs` |
| `pub async fn get_communities_batch( &self, entity_ids: &[EntityId], ) -> Result<HashMap<EntityId, u64>>` | — | `src/csr/graph_read/communities.rs` |
| `pub fn set_communities_batch(&self, assignments: &[crate::CommunityAssignment])` | — | `src/csr/graph_read/communities.rs` |
| `pub fn compact(&self)` | — | `src/csr/graph_read/mod.rs` |
| `pub async fn compact_async(&self) -> Result<()>` | — | `src/csr/graph_read/mod.rs` |
| `pub fn edge_count(&self) -> usize` | — | `src/csr/graph_read/mod.rs` |
| `pub fn entity_count(&self) -> usize` | — | `src/csr/graph_read/mod.rs` |
| `pub fn entity_exists(&self, id: EntityId) -> bool` | — | `src/csr/graph_read/mod.rs` |
| `pub async fn is_entity_deleted(&self, entity: EntityId) -> bool` | — | `src/csr/graph_read/mod.rs` |
| `pub async fn neighbors(&self, start: EntityId) -> Result<Vec<EntityId>>` | — | `src/csr/graph_read/mod.rs` |
| `pub async fn remove_entity(&self, tx: TxId, entity: EntityId) -> Result<()>` | — | `src/csr/graph_read/mod.rs` |
| `pub async fn deleted_view(&self) -> crate::DeletedView` | — | `src/csr/graph_read/pagerank.rs` |
| `pub async fn get_deleted_node_indices(&self) -> HashSet<usize>` | — | `src/csr/graph_read/pagerank.rs` |
| `pub async fn pagerank( &self, damping_factor: f32, max_iterations: usize, tolerance: f32, ) -> HashMap<EntityId, f32>` | — | `src/csr/graph_read/pagerank.rs` |
| `pub async fn personalized_page_rank_with_context_async( &self, seed_nodes: &[EntityId], config: &contextra_types::PprConfig, ctx: &mut crate::PprContext, ) -> Vec<(EntityId, f32)>` | — | `src/csr/graph_read/pagerank.rs` |
| `pub fn path_rag_at<'a>( &'a self, start_nodes: &'a [EntityId], max_hops: usize, seq_no: u64, ) -> contextra_ports::BoxFuture<'a, Result<Vec<(EntityId, f32)>>>` | — | `src/csr/graph_read/path_rag.rs` |
| `pub async fn add_edge( self: &Arc<Self>, from: EntityId, to: EntityId, weight: f32, tx_valid_from: Option<TxId>, tx_valid_to: Option<TxId>, business_valid_from: Option<i64>, business_valid_to: Option<i64>, source_doc_id: Option<DocId>, predicate_hash: Option<…` | — | `src/csr/graph_write/edge_mutation.rs` |
| `pub async fn insert_edge_direct( self: &Arc<Self>, from: EntityId, to: EntityId, weight: f32, ) -> Result<()>` | — | `src/csr/graph_write/edge_mutation.rs` |
| `pub async fn insert_edge_direct_with_bitemporal_validity( self: &Arc<Self>, from: EntityId, to: EntityId, weight: f32, tx_valid_from: Option<TxId>, tx_valid_to: Option<TxId>, business_valid_from: Option<i64>, business_valid_to: Option<i64>, source_doc_id: Opt…` | — | `src/csr/graph_write/edge_mutation.rs` |
| `pub async fn insert_edge_direct_with_validity( self: &Arc<Self>, from: EntityId, to: EntityId, weight: f32, tx_valid_from: Option<TxId>, tx_valid_to: Option<TxId>, ) -> Result<()>` | — | `src/csr/graph_write/edge_mutation.rs` |
| `pub fn load_edge_direct( &self, from: EntityId, to: EntityId, weight: f32, tx_valid_from: Option<TxId>, tx_valid_to: Option<TxId>, business_valid_from: Option<i64>, business_valid_to: Option<i64>, source_doc_id: Option<DocId>, ) -> Result<()>` | — | `src/csr/graph_write/edge_mutation.rs` |
| `pub fn insert_entity_direct(&self, entity: Entity) -> Result<()>` | — | `src/csr/graph_write/entity.rs` |
| `pub fn load_entity_direct(&self, entity: Entity) -> Result<()>` | — | `src/csr/graph_write/entity.rs` |
| `pub fn commit_super_edge_batch( &self, tombstone_ids: &[crate::hyperedge::HyperEdgeId], new_edges: Vec<crate::hyperedge::HyperEdge>, wal_tx: TxId, ) -> std::result::Result<(), GraphMutationError>` | — | `src/csr/graph_write/hyperedge.rs` |
| `pub fn enqueue_cascade_deferred( &self, doc_id: DocId, hyperedge_ids: &[crate::hyperedge::HyperEdgeId], )` | — | `src/csr/graph_write/hyperedge.rs` |
| `pub fn get_hyperedge( &self, id: crate::hyperedge::HyperEdgeId, ) -> Option<Arc<crate::hyperedge::HyperEdge>>` | — | `src/csr/graph_write/hyperedge.rs` |
| `pub fn hyperedges_for_doc(&self, doc_id: DocId) -> Vec<crate::hyperedge::HyperEdgeId>` | — | `src/csr/graph_write/hyperedge.rs` |
| `pub fn hyperedges_for_entity(&self, entity_id: EntityId) -> Vec<crate::hyperedge::HyperEdgeId>` | — | `src/csr/graph_write/hyperedge.rs` |
| `pub fn insert_hyperedge(&self, edge: crate::hyperedge::HyperEdge)` | — | `src/csr/graph_write/hyperedge.rs` |
| `pub fn insert_hyperedge_direct(&self, hyperedge: crate::hyperedge::HyperEdge)` | — | `src/csr/graph_write/hyperedge.rs` |
| `pub fn max_hyperedge_id(&self) -> u64` | — | `src/csr/graph_write/hyperedge.rs` |
| `pub fn parent_hyperedges_of( &self, child: crate::hyperedge::HyperEdgeId, ) -> Vec<crate::hyperedge::HyperEdgeId>` | — | `src/csr/graph_write/hyperedge.rs` |
| `pub fn pending_cascade_queue_len(&self) -> usize` | — | `src/csr/graph_write/hyperedge.rs` |
| `pub async fn persist_hyperedge( &self, tx: TxId, hyperedge: &crate::hyperedge::HyperEdge, ) -> Result<()>` | — | `src/csr/graph_write/hyperedge.rs` |
| `pub async fn process_cascade_queue(&self, batch_size: usize, wal_tx: TxId) -> Result<usize>` | — | `src/csr/graph_write/hyperedge.rs` |
| `pub fn relate_n_ary( &self, id: crate::hyperedge::HyperEdgeId, predicate: crate::csr::EdgeType, participants: Vec<crate::hyperedge::RoleBinding>, weight: f32, doc_id: Option<DocId>, ) -> std::result::Result<crate::hyperedge::HyperEdge, GraphMutationError>` | — | `src/csr/graph_write/hyperedge.rs` |
| `pub fn edges_for_doc(&self, doc_id: DocId) -> Vec<(EntityId, EntityId)>` | — | `src/csr/graph_write/mod.rs` |
| `pub fn estimate_compaction_peak_bytes(&self) -> usize` | — | `src/csr/graph_write/mod.rs` |
| `pub fn estimate_memory(&self, presized: bool) -> MemoryEstimate` | — | `src/csr/graph_write/mod.rs` |
| `pub fn estimate_memory_bytes(&self) -> usize` | — | `src/csr/graph_write/mod.rs` |
| `pub fn get_source_doc_id(&self, index: usize) -> Option<DocId>` | — | `src/csr/graph_write/mod.rs` |
| `pub fn inner_read(&self) -> arc_swap::Guard<Arc<GraphInner>>` | — | `src/csr/graph_write/mod.rs` |
| `pub fn new() -> Self` | — | `src/csr/graph_write/mod.rs` |
| `pub fn set_storage(&mut self, storage: Arc<dyn StorageEngine>)` | — | `src/csr/graph_write/mod.rs` |
| `pub fn source_doc_id_at(&self, from: EntityId, to: EntityId) -> Option<DocId>` | — | `src/csr/graph_write/mod.rs` |
| `pub fn storage(&self) -> Option<Arc<dyn StorageEngine>>` | — | `src/csr/graph_write/mod.rs` |
| `pub fn with_config(config: CsrGraphConfig) -> Self` | — | `src/csr/graph_write/mod.rs` |
| `pub fn with_config_and_storage( config: CsrGraphConfig, storage: Arc<dyn StorageEngine>, ) -> Self` | — | `src/csr/graph_write/mod.rs` |
| `pub fn with_consistency_enforcer(suppression_threshold: u32) -> Self` | — | `src/csr/graph_write/mod.rs` |
| `pub fn with_storage(storage: Arc<dyn StorageEngine>) -> Self` | — | `src/csr/graph_write/mod.rs` |
| `pub async fn tombstone_edge( &self, edge_id: crate::consistency_enforcement::EdgeId, tx: TxId, ) -> Result<()>` | — | `src/csr/graph_write/tombstone.rs` |
| `pub fn tombstone_hyperedge(&self, id: crate::hyperedge::HyperEdgeId, wal_tx: TxId) -> bool` | — | `src/csr/graph_write/tombstone.rs` |
| `pub fn compact(&mut self)` | — | `src/csr/inner/compaction.rs` |
| `pub fn get_or_create_index(&mut self, id: EntityId) -> InternalIndex` | — | `src/csr/inner/compaction.rs` |
| `pub fn total_bytes(&self) -> usize` | — | `src/csr/inner/estimate.rs` |
| `pub fn new() -> Self` | — | `src/csr/inner/mod.rs` |
| `pub fn new(target: EntityId, weight: f32) -> Self` | — | `src/csr/types.rs` |
| `pub fn is_edge_visible( tx_valid_from: Option<TxId>, tx_valid_to: Option<TxId>, as_of: TxId, ) -> bool` | — | `src/csr/visibility.rs` |
| `pub fn is_edge_visible_bitemporal( tx_valid_from: Option<TxId>, tx_valid_to: Option<TxId>, as_of_tx: TxId, business_valid_from: Option<i64>, business_valid_to: Option<i64>, as_of_business: Option<i64>, ) -> bool` | — | `src/csr/visibility.rs` |
| `pub fn is_edge_visible_business( business_valid_from: Option<i64>, business_valid_to: Option<i64>, business_as_of: i64, ) -> bool` | — | `src/csr/visibility.rs` |
| `pub fn is_suspicious_tx_id(tx: TxId) -> bool` | — | `src/csr/visibility.rs` |
| `pub fn cooccurrence_count(&self) -> usize` | — | `src/edge_reinforcement_buffer.rs` |
| `pub fn flush_to_graph(&self, graph: &CsrGraph, config: &EdgeReinforcementConfig)` | — | `src/edge_reinforcement_buffer.rs` |
| `pub fn new() -> Self` | — | `src/edge_reinforcement_buffer.rs` |
| `pub fn push_cooccurrence(&self, from: EntityId, to: EntityId, co_activation: f32)` | — | `src/edge_reinforcement_buffer.rs` |
| `pub fn push_traversal(&self, from: EntityId, to: EntityId, path_length: usize)` | — | `src/edge_reinforcement_buffer.rs` |
| `pub fn traversal_count(&self) -> usize` | — | `src/edge_reinforcement_buffer.rs` |
| `pub fn ingest_extracted_relations( graph: &CsrGraph, relations: &[ExtractedRelation], min_confidence_threshold: f32, doc_id: Option<DocId>, ) -> Result<Vec<HyperEdge>>` | — | `src/entity_extraction.rs` |
| `pub fn new( source: impl Into<String>, target: impl Into<String>, relation_type: impl Into<String>, confidence: f32, ) -> Self` | — | `src/entity_extraction.rs` |
| `pub fn new(min_confidence: f32) -> Self` | — | `src/entity_extraction.rs` |
| `pub fn acquire(entities: &[EntityId]) -> Self` | — | `src/hyperedge.rs` |
| `pub fn contains_id(&self, id: RoleId) -> bool` | — | `src/hyperedge.rs` |
| `pub fn contains_role(&self, name: &str) -> bool` | — | `src/hyperedge.rs` |
| `pub fn deserialize(bytes: &[u8]) -> Result<Self>` | — | `src/hyperedge.rs` |
| `pub fn entities(&self) -> &[EntityId]` | — | `src/hyperedge.rs` |
| `pub fn get_or_intern(&self, name: &str) -> RoleId` | — | `src/hyperedge.rs` |
| `pub fn is_empty(&self) -> bool` | — | `src/hyperedge.rs` |
| `pub fn len(&self) -> usize` | — | `src/hyperedge.rs` |
| `pub fn new( id: HyperEdgeId, predicate: EdgeType, participants: impl Into<Arc<[RoleBinding]>>, weight: f32, ) -> Self` | — | `src/hyperedge.rs` |
| `pub fn new(edge: &'a HyperEdge) -> Self` | — | `src/hyperedge.rs` |
| `pub fn new() -> Self` | — | `src/hyperedge.rs` |
| `pub fn resolve<F, R>(&self, id: RoleId, f: F) -> Option<R> where F: FnOnce(&str) -> R,` | — | `src/hyperedge.rs` |
| `pub fn resolve_string(&self, id: RoleId) -> Option<String>` | — | `src/hyperedge.rs` |
| `pub fn serialize(&self) -> Result<Vec<u8>>` | — | `src/hyperedge.rs` |
| `pub fn slice_participants( &self, range: impl std::ops::RangeBounds<usize>, ) -> ArcSlice<RoleBinding>` | — | `src/hyperedge.rs` |
| `pub fn sort_dedup_entities(entities: &[EntityId]) -> Vec<EntityId>` | — | `src/hyperedge.rs` |
| `pub fn star_weight(weight: f32, cardinality: usize) -> f32` | — | `src/hyperedge.rs` |
| `pub fn try_acquire(entities: &[EntityId]) -> std::result::Result<Self, GraphMutationError>` | — | `src/hyperedge.rs` |
| `pub fn validate(&self) -> std::result::Result<(), GraphMutationError>` | — | `src/hyperedge.rs` |
| `pub fn view(&self) -> HyperEdgeView<'_>` | — | `src/hyperedge.rs` |
| `pub fn with_business_validity( mut self, valid_from: Option<i64>, valid_to: Option<i64>, ) -> Self` | — | `src/hyperedge.rs` |
| `pub fn with_child_edge_ids(mut self, ids: impl Into<Arc<[HyperEdgeId]>>) -> Self` | — | `src/hyperedge.rs` |
| `pub fn with_entities<F, R>(&self, f: F) -> R where F: FnOnce(&[EntityId]) -> R,` | — | `src/hyperedge.rs` |
| `pub fn with_source_doc_id(mut self, source_doc_id: Option<DocId>) -> Self` | — | `src/hyperedge.rs` |
| `pub fn with_tx_validity(mut self, valid_from: Option<TxId>, valid_to: Option<TxId>) -> Self` | — | `src/hyperedge.rs` |
| `pub fn compute_co_occurrence_candidates( entities_per_document: &[(DocId, Vec<EntityId>)], min_co_occurrence: usize, ) -> Vec<HyperEdgeCandidate>` | — | `src/hyperedge_suggest.rs` |
| `pub async fn validate_candidates_with_llm( candidates: &[HyperEdgeCandidate], entity_labels: &dyn Fn(EntityId) -> Option<String>, generator: &dyn LlmTextGenerator, max_llm_calls_per_cycle: usize, ) -> Result<Vec<ValidatedHyperEdgeCandidate>, GraphMutationErro…` | Validates hyperedge candidates using an [`LlmTextGenerator`]. For each candidate (up to `max_llm_calls_per_cycle`), queries the LLM to conf… | `src/hyperedge_suggest.rs` |
| `pub fn find_all_paths(&self, source: EntityId) -> Vec<GraphPath>` | — | `src/path_rag/mod.rs` |
| `pub fn find_path(&self, source: EntityId, target: EntityId) -> Option<GraphPath>` | — | `src/path_rag/mod.rs` |
| `pub fn forward_push_ppr<G: PathGraph>( graph: &G, seeds: &[EntityId], params: &PprParams, ) -> AHashMap<EntityId, f32>` | — | `src/path_rag/mod.rs` |
| `pub fn max_hops(&self) -> usize` | — | `src/path_rag/mod.rs` |
| `pub fn new(graph: G, max_hops: usize, sufficiency_threshold: f64) -> Self` | — | `src/path_rag/mod.rs` |
| `pub fn sufficiency_check(&self, path: &GraphPath) -> bool` | — | `src/path_rag/mod.rs` |
| `pub fn sufficiency_threshold(&self) -> f64` | — | `src/path_rag/mod.rs` |
| `pub fn to_rrf_signal(&self, paths: &[GraphPath]) -> Vec<(DocId, f32)>` | — | `src/path_rag/mod.rs` |
| `pub fn with_config(graph: G, config: PathRAGConfig) -> Self` | — | `src/path_rag/mod.rs` |
| `pub fn with_defaults(graph: G) -> Self` | — | `src/path_rag/mod.rs` |
| `pub fn new(inner: &'a GraphInner, as_of_tx: TxId) -> Self` | — | `src/path_rag/snapshot.rs` |
| `pub fn compute_percolation_health( active_node_count: usize, active_edge_count: usize, // nach Tombstone-Abzug ) -> Option<f32>` | — | `src/percolation.rs` |
| `pub async fn find_rebonding_candidates<G: contextra_ports::GraphIndex>( graph: &G, embeddings: &HashMap<EntityId, Vec<f32>>, config: &PercolationConfig, ) -> Vec<(EntityId, EntityId, f32)>` | Findet Kandidaten-Paare für Re-Bonding. Gibt `(EntityId, EntityId, similarity)`-Tupel zurück, sortiert absteigend nach Similarity. | `src/percolation.rs` |
| `pub fn should_trigger_rebonding(health: f32, config: &PercolationConfig) -> bool` | — | `src/percolation.rs` |
| `pub fn cheapest_strategy(&self) -> PprAlgorithm` | — | `src/ppr/cost.rs` |
| `pub fn clear(&self)` | — | `src/ppr/cost.rs` |
| `pub fn estimate_ppr_cost( stats: Option<&GraphStats>, seed_count: usize, config: &PprConfig, ) -> (PprCostEstimate, PprAlgorithm)` | — | `src/ppr/cost.rs` |
| `pub fn new(node_count: usize, edge_count: usize) -> Self` | — | `src/ppr/cost.rs` |
| `pub fn new() -> Self` | — | `src/ppr/cost.rs` |
| `pub fn recommended_multiplier(&self) -> Option<f64>` | — | `src/ppr/cost.rs` |
| `pub fn record(&self, sample: PprCostSample)` | — | `src/ppr/cost.rs` |
| `pub fn sample_count(&self) -> usize` | — | `src/ppr/cost.rs` |
| `pub fn compute_ppr_with_apprh_selector( inner: &GraphInner, seed_nodes: &[EntityId], config: &PprConfig, deleted_nodes: &DeletedView, ctx: &mut PprContext, selector: &crate::apprh::ApprhSelector, apprh_params: &crate::apprh::ApprhParams, ) -> Vec<(EntityId, f…` | — | `src/ppr/mod.rs` |
| `pub fn log_tl_hfd_vs_baseline_discrepancy( baseline: &AHashMap<EntityId, f32>, candidate: &AHashMap<EntityId, f32>, context_id: u64, sink: &dyn ShadowSink<f64>, )` | — | `src/ppr/shadow_hook.rs` |
| `pub fn contains(&self, idx: usize) -> bool` | — | `src/ppr/types.rs` |
| `pub fn is_empty(&self) -> bool` | — | `src/ppr/types.rs` |
| `pub fn len(&self) -> usize` | — | `src/ppr/types.rs` |
| `pub fn new() -> Self` | — | `src/ppr/types.rs` |
| `pub fn is_exhausted(&self) -> bool` | — | `src/ppr_stream.rs` |
| `pub fn new(graph: &'a G, seeds: impl Into<Vec<EntityId>>, config: PprConfig) -> Self` | — | `src/ppr_stream.rs` |
| `pub async fn next_batch(&mut self) -> Result<Vec<(EntityId, f32)>>` | Fetches the next batch of candidate entity-score pairs (up to `batch_size`). Returns an empty `Vec` when exhausted. | `src/ppr_stream.rs` |
| `pub async fn next_batch_docs<F>(&mut self, mut resolve: F) -> Result<Vec<DocId>> where F: FnMut(EntityId) -> Option<DocId>,` | Fetches the next batch of resolved [`DocId`]s (up to `batch_size`). Entities for which `resolve` returns `None` are skipped. | `src/ppr_stream.rs` |
| `pub async fn next_entity(&mut self) -> Result<Option<EntityId>>` | Fetches the next single candidate entity ID. Returns `Ok(None)` when exhausted. | `src/ppr_stream.rs` |
| `pub fn with_batch_size(mut self, n: usize) -> Self` | — | `src/ppr_stream.rs` |
| `pub fn with_exclude_seeds(mut self, exclude: bool) -> Self` | — | `src/ppr_stream.rs` |
| `pub fn yielded(&self) -> usize` | — | `src/ppr_stream.rs` |
| `pub fn edges_for_doc(&self, doc_id: DocId) -> Vec<EdgeId>` | — | `src/provenance.rs` |
| `pub fn new(edge_id: EdgeId, source_doc_ids: Vec<DocId>, created_at_tx: TxId) -> Self` | — | `src/provenance.rs` |
| `pub fn new() -> Self` | — | `src/provenance.rs` |
| `pub fn record(&self, doc_id: DocId, edge_id: EdgeId)` | — | `src/provenance.rs` |
| `pub fn record_provenance(&self, provenance: &EdgeProvenance)` | — | `src/provenance.rs` |
| `pub fn remove_doc(&self, doc_id: DocId)` | — | `src/provenance.rs` |
| `pub fn active_head(&self) -> RwLockReadGuard<'_, NodeIdx>` | — | `src/session_dag.rs` |
| `pub fn active_head(&self) -> RwLockReadGuard<'_, NodeIdx>` | — | `src/session_dag.rs` |
| `pub fn active_head(&self) -> NodeIdx` | — | `src/session_dag.rs` |
| `pub fn active_head_write(&self) -> RwLockWriteGuard<'_, NodeIdx>` | — | `src/session_dag.rs` |
| `pub fn active_head_write(&self) -> RwLockWriteGuard<'_, NodeIdx>` | — | `src/session_dag.rs` |
| `pub fn append_step( &self, prompt: String, response: String, snapshot_tx_id: Option<TxId>, tool_outputs: Vec<String>, label: &str, ) -> Result<NodeIdx>` | — | `src/session_dag.rs` |
| `pub fn branch_from( &self, parent_node: NodeIdx, prompt: String, response: String, snapshot_tx_id: Option<TxId>, tool_outputs: Vec<String>, label: &str, ) -> Result<NodeIdx>` | — | `src/session_dag.rs` |
| `pub fn children_of(&self, node_idx: NodeIdx) -> Vec<NodeIdx>` | — | `src/session_dag.rs` |
| `pub fn edges(&self) -> RwLockReadGuard<'_, Vec<DagEdge>>` | — | `src/session_dag.rs` |
| `pub fn edges(&self) -> RwLockReadGuard<'_, Vec<DagEdge>>` | — | `src/session_dag.rs` |
| `pub fn edges_write(&self) -> RwLockWriteGuard<'_, Vec<DagEdge>>` | — | `src/session_dag.rs` |
| `pub fn edges_write(&self) -> RwLockWriteGuard<'_, Vec<DagEdge>>` | — | `src/session_dag.rs` |
| `pub fn get_node(&self, node_idx: NodeIdx) -> Option<AgentStateNode>` | — | `src/session_dag.rs` |
| `pub async fn load<S: StorageEngine + ?Sized>(storage: &S, namespace: &str) -> Result<Self>` | Loads and reconstructs a `SessionBranchTree` from storage for the specified namespace. | `src/session_dag.rs` |
| `pub fn lock_nodes(&self) -> NodesGuard<'_>` | — | `src/session_dag.rs` |
| `pub fn lock_nodes_write(&self) -> NodesWriteGuard<'_>` | — | `src/session_dag.rs` |
| `pub fn new(root_prompt: String, root_response: String) -> Self` | — | `src/session_dag.rs` |
| `pub fn node_count(&self) -> usize` | — | `src/session_dag.rs` |
| `pub fn nodes(&self) -> &HashMap<NodeIdx, AgentStateNode>` | — | `src/session_dag.rs` |
| `pub fn nodes(&self) -> &HashMap<NodeIdx, AgentStateNode>` | — | `src/session_dag.rs` |
| `pub fn nodes_mut(&mut self) -> &mut HashMap<NodeIdx, AgentStateNode>` | — | `src/session_dag.rs` |
| `pub fn path_to_head(&self) -> Vec<AgentStateNode>` | — | `src/session_dag.rs` |
| `pub async fn save<S: StorageEngine + ?Sized>( &self, storage: &S, namespace: &str, tx: TxId, ) -> Result<()>` | Persists all nodes, edges, active head, and next_id of the Session-DAG to the storage engine under the given namespace prefix. | `src/session_dag.rs` |
| `pub fn set_active_head(&self, node_idx: NodeIdx) -> Result<()>` | — | `src/session_dag.rs` |
| `pub fn run_diffusion<G: PathGraph>( graph: &G, seeds: &[EntityId], params: &TlHfdParams, ) -> Result<AHashMap<EntityId, f32>, TlHfdError>` | — | `src/tl_hfd/diffusion.rs` |
| `pub fn compute_lovasz_extension( participants: &[EntityId], get_x: impl Fn(EntityId) -> f32, ) -> (f32, Option<EntityId>, Option<EntityId>)` | — | `src/tl_hfd/lovasz.rs` |
| `pub fn truncate_participants( participants: &[EntityId], get_x: impl Fn(EntityId) -> f32, max_sort_size: usize, ) -> Vec<EntityId>` | — | `src/tl_hfd/lovasz.rs` |
| `pub fn thresholded_local_hfd( &self, seeds: &[EntityId], params: &TlHfdParams, ) -> Result<AHashMap<EntityId, f32>, TlHfdError>` | — | `src/tl_hfd/mod.rs` |
| `pub fn tl_hfd_local<G: PathGraph>( graph: &G, seeds: &[EntityId], params: &TlHfdParams, ) -> Result<AHashMap<EntityId, f32>, TlHfdError>` | — | `src/tl_hfd/mod.rs` |
| `pub fn validate(&self) -> Result<(), TlHfdError>` | — | `src/tl_hfd/params.rs` |
| `pub fn shadow_compare_forward_push_vs_tl_hfd<G: PathGraph>( graph: &G, seeds: &[EntityId], ppr: &PprParams, tl: &TlHfdParams, top_k: usize, ) -> Result<ShadowComparison, TlHfdError>` | — | `src/tl_hfd/shadow.rs` |


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

# Contextra Rank (`contextra-rank`)  **Ring:** Ring 0 (Ranking & Kalibrierung / Kernlogik) **Klasse:** Rand `[R]` **Reifegrad:** Experimentell / Aktiv (Kern v1)  ## Zweck des Crates Dieses Crate implementiert Methoden zur Fusion, Kalibrierung und Erklärung von Retrieval-Ergebnissen im Contextra-System. Dazu gehören: - **4-Signal-Fusion:** Verknüpfung von Vektor-, Text- (BM25), Graph- und Rerank-Scores mittels reziproker Rangfusion (RRF = Reciprocal Rank Fusion) und score-basierter Normalisierung. - **Kalibrierung:** Umwandlung von Roh-Scores in Wahrscheinlichkeiten via Platt-Scaling (logistische Regression) und Isotonische Regression (PAVA = Pool-Adjacent Violators Algorithm). - **Konforme Prädiktion:** Finite-Sample-Abdeckungsgarantien und adaptive Schwellenwertanpassung bei Verteilungsverschiebung (Covariate Shift). - **Drift-Erkennung:** Überwachung von Score-Verteilungen zur Laufzeit. - **Erklärbarkeit (Explainability):** Umwandlung maschinenlesbarer Herkunftsnachweise (`ProvenanceRecord`) in strukturierte und deutschsprachige Erklärungen. - **DiBud (Discrete Budgeted):** Jederzeit abbrechbare Streaming-Fusion unter Zeit- und Kanalbudgets.  ## Einbindung und Abhängigkeiten - **Konsumiert (Ring 0):** `contextra-types` (Domain-Typen), `contextra-ports` (Hardware-Abstraktionen). - **Wird eingebunden von:** `contextra`, `contextra-db`, `contextra-engine`, `contextra-infer-candle`, `contextra-infer-ollama`, `contextra-infer-onnx`, `contextra-mcp`, `contextra-py`.

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

**`ConformalCalibrator`** — *Trait für deterministische konforme Schwellenwertschätzung.* (`src/calibration/conformal.rs`)

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
    // ANCHOR[contextra-rank:CONF-004]
    // Ablauf des Online-Pinball-Updates:
    // 1. Validierung: Prüfen, ob propensity_weight > 0.0 und non_conformity_score endlich sind.
    // 2. Schrittweite berechnen:
    //    - Bei Überschreitung (S_t > threshold): Delta = +gamma * w_t * (1 - alpha)
    //    - Bei Unterschreitung (S_t <= threshold): Delta = -gamma * w_t * alpha
    // 3. Klemmen: Neuer Schwellenwert wird auf [min_threshold, max_threshold] beschränkt.
    // 4. Inkrementieren: Zähler der Beobachtungen erhöhen und Änderungsstatus zurückgeben.
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
| `AdaptiveConformalCalibrator` | `{ alpha: f32, threshold: f32, learning_rate: f32, min_threshold: f32, max_threshold: f32, observation_count: u64, fingerprint: Option<ConfigFingerprint>, }` | Adaptiver konformer Kalibrierer für Online-Schwellenwerte unter Covariate Shift. | `src/calibration/conformal.rs` |
| `BoundedTopK` | `{ heap: std::collections::BinaryHeap<T>, capacity: usize, }` | Größenbeschränkter Min-Heap für die Top-K-Kandidatenselektion während der Rangfusion. | `src/fusion/topk.rs` |
| `DiBudFusionState` | `{ channel_depths: [usize; 3], exhausted: [bool; 3], accesses: usize, observed_docs: AHashMap<DocId, ObservedDocState>, er_cache: AHashMap<DocId, f32>, provenance… }` | Zustandsautomat für die budgetierte Streaming-Rangfusion. | `src/dibud/state.rs` |
| `DiBudOutcome` | `{ /// Ranked candidate document IDs sorted descending by lower bound $F^-(x)$ (tie-breaking by ascending `DocId`). pub ranked: Vec<DocId>, /// Number of prefix e… }` | Endergebnis der DiBud-Fusion inklusive zertifiziertem Präfix. | `src/dibud/types.rs` |
| `DriftDetector` | `{ window_size: usize, threshold: f32, baseline_mean: Option<f32>, recent_scores: VecDeque<f32>, }` | Detektor zur Überwachung von Score-Verschiebungen über ein gleitendes Fenster. | `src/drift.rs` |
| `ExplanationEntry` | `{ /// Identified search signal type. pub signal: SignalKind, /// Raw unnormalized score or distance. pub raw_score: f32, /// 1-based rank in the signal's result… }` | Einzelner Beitrags-Eintrag für eine spezifische Signalquelle innerhalb der Erklärung. | `src/explain.rs` |
| `FusionBudget` | `{ /// Maximum total channel access budget (reads). pub max_total_accesses: usize, /// Minimum required certified prefix length before stopping. pub min_certified… }` | Ressourcen- und Abbruchbudget für die Streaming-Rangfusion. | `src/dibud/types.rs` |
| `GlobalFusionConfig` | `{ /// Maximum number of community nodes to include in the final result set. pub max_community_nodes: Option<usize>, /// Minimum size (member count) required for… }` | Konfigurationsparameter für die globale Community-Fusionsstrategie. | `src/fusion/global.rs` |
| `GlobalFusionStrategy` | `{ config: GlobalFusionConfig, }` | Globale Fusionsstrategie für hierarchische und korpusweite Retrieval-Szenarien. | `src/fusion/global.rs` |
| `IsotonicCalibrator` | `{ observations: VecDeque<(f32, bool)>, warmup_required: u32, max_observations: usize, cached_model: Option<Vec<(f32, f32)>>, // (max_score_in_block, calibrated_p… }` | Nicht-parametrischer Kalibrierer auf Basis von PAVA. | `src/calibration/isotonic.rs` |
| `PlattScaler` | `{ a: f32, b: f32, fingerprint: Option<ConfigFingerprint>, }` | Parametrischer Kalibrierer nach Platt (1999). | `src/calibration/platt.rs` |
| `ProvenanceBuilder` | `{ vector_distance: Option<f32>, vector_rank: Option<u32>, vector_weight: Option<f32>, bm25_score: Option<f32>, bm25_rank: Option<u32>, text_weight: Option<f32>,… }` | Fluent-Builder zum deterministischen Erstellen von `ProvenanceRecord`-Strukturen. | `src/fusion/provenance.rs` |
| `ProvenanceRecord` | `{ /// Distance score from vector search signal. #[serde(default, skip_serializing_if = "Option::is_none")] pub vector_distance: Option<f32>, /// BM25 score from… }` | Auditierbarer Herkunftsnachweis (Provenance) über alle beteiligten Retrieval-Kanäle. | `src/fusion/types.rs` |
| `ResonanceConfig` | `{ /// Exponent β for coherence bonus. Default: 0.5. pub beta: f32, /// Boost strength γ. Default: 0.3. pub gamma: f32, }` | Konfigurationsparameter für die Berechnung des Resonanz-Kohärenz-Bonus. | `src/fusion/resonance.rs` |
| `RetrievalExplanation` | `{ /// Individual signal contribution entries, sorted descending by `contribution_share`. pub entries: Vec<ExplanationEntry>, /// Dominant search signal contribut… }` | Strukturierte Gesamterklärung zur Nachvollziehbarkeit eines Retrieval-Ergebnisses. | `src/explain.rs` |
| `SearchReport` | `{ /// List of signals that failed during query execution when using Degrade policy. pub degraded_signals: Vec<Signal>, /// Warnings captured during query process… }` | Detaillierter Audit-Report über den Fusions- und Suchablauf. | `src/fusion/types.rs` |
| `SearchResult` | `{ /// Document or memory entry identifier. pub id: String, /// Fused score value. pub score: f32, /// Document metadata payload. pub metadata: Option<serde_json:… }` | Standard-Suchergebnis eines einzelnen Retrieval-Signals oder einer Fusionsstufe. | `src/fusion/types.rs` |
| `SignalCalibrationContext` | `{ /// Signal identifier matching the result set signal name (e.g., "vector", "text", "graph"). pub signal_name: String, /// Indicates whether the calibrator for… }` | Kontext zu Modellkalibrierung und Driftzustand eines Signals für die mRRF-Gewichtsmodulation. | `src/fusion/signal.rs` |
| `SignalContribution` | `{ /// Raw score before normalization/fusion. pub raw_score: f32, /// 1-based rank in the signal's result list. pub rank: u32, /// Calculated RRF or normalized co… }` | Beitrag eines einzelnen Signals zum aggregierten RRF-Score. | `src/fusion/types.rs` |

### Öffentliche Datentypen — Enums (7)

| Enum | Varianten | Beschreibung | Datei |
|---|---|---|---|
| `BudgetedChannel` | `/// Vector (semantic) search channel (index 0). Vector = 0, /// Text (BM25) search channel (index 1). Text = 1, /// Graph search channel (index 2). Graph = 2,` | Identifiziert die durch DiBud budgetierten Retrieval-Kanäle. | `src/dibud/types.rs` |
| `ConformalError` | `/// Target miscoverage level `alpha` is not finite or outside (0.0, 1.0). #[error("Invalid target miscoverage alpha: {0} (must be finite and in (0.0, 1.0))")] Alpha(f32), /// Propensity weight is not…` | Fehlertypen für die konforme Kalibrierung. | `src/calibration/conformal.rs` |
| `DiBudStep` | `/// Request reading the next item from the specified channel. Poll(BudgetedChannel), /// Fusion is complete; outcome contains certified prefix and ranked documents. Done(DiBudOutcome),` | Nächste durchzuführende Aktion im DiBud-Zustandsautomaten. | `src/dibud/state.rs` |
| `DriftStatus` | `/// Distribution is stable. Stable { /// Mean score shift relative to baseline. mean_shift: f32, }, /// Drift detected exceeding configured tolerance threshold. DriftDetected { /// Mean score shift r…` | Zusammenfassung des aktuellen Drift-Status einer Signalverteilung. | `src/drift.rs` |
| `MetadataMergePriority` | `/// Vector metadata is processed first (default behavior). Order: Vector, Text, Graph. #[default] VectorFirst, /// Text metadata is processed first. Order: Text, Vector, Graph. TextFirst, /// Graph m…` | Prioritätsstrategie bei der Zusammenführung kollidierender Metadatenschlüssel. | `src/fusion/signal.rs` |
| `Signal` | `/// Dense vector similarity signal. Vector, /// Full-text BM25 keyword signal. Text, /// Graph entity relationship signal. Graph,` | Kennzeichnung der Signalherkunft. | `src/fusion/types.rs` |
| `SignalKind` | `/// Vector (semantic k-NN) search signal. Vector, /// Text (BM25 keyword) search signal. Text, /// Graph (traversal / PageRank) search signal. Graph, /// Edge-reinforcement weight signal / Bandit rec…` | Klassifikation der unterschiedlichen Retrieval-Signalarten im Contextra-System. | `src/fusion/signal.rs` |

### Typ-Aliase (1)

| Alias | Definition | Beschreibung |
|---|---|---|
| `FusedScore` | `type FusedScore= SearchResult` | Aggregierter Fusionswert eines Dokuments (Typ-Alias auf SearchResult). |

### Konstanten (4)

| Konstante | Typ | Beschreibung |
|---|---|---|
| `DEFAULT_INITIAL_THRESHOLD` | `f32` | Default initial threshold $\hat{q}_0$. |
| `DEFAULT_LEARNING_RATE` | `f32` | Default learning rate $\gamma$ for online quantile updates. |
| `DEFAULT_MAX_THRESHOLD` | `f32` | Default maximum threshold bound. |
| `DEFAULT_MIN_THRESHOLD` | `f32` | Default minimum threshold bound. |

### Öffentliche Funktionen (89)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn alpha(&self) -> f32` | Returns the target error rate $\alpha$. | `src/calibration/conformal.rs` |
| `pub fn invalidate_on_config_change(&mut self, new_fingerprint: ConfigFingerprint)` | Resets state if ConfigFingerprint changes (P8 Compliance). | `src/calibration/conformal.rs` |
| `pub fn learning_rate(&self) -> f32` | Returns the learning rate $\gamma$. | `src/calibration/conformal.rs` |
| `pub fn new(alpha: f32) -> Result<Self, ConformalError>` | Creates a new `AdaptiveConformalCalibrator` with target error rate `alpha` in `(0.0, 1.0)`. Uses default initial threshold (0.5) and defaul… | `src/calibration/conformal.rs` |
| `pub fn observation_count(&self) -> u64` | Returns the total number of observations processed. | `src/calibration/conformal.rs` |
| `pub fn with_bounds( mut self, min_threshold: f32, max_threshold: f32, ) -> Result<Self, ConformalError>` | Sets bounding constraints `[min_threshold, max_threshold]` for the estimated threshold. | `src/calibration/conformal.rs` |
| `pub fn with_params( alpha: f32, initial_threshold: f32, learning_rate: f32, ) -> Result<Self, ConformalError>` | Creates an `AdaptiveConformalCalibrator` with custom parameters. | `src/calibration/conformal.rs` |
| `pub fn cached_ece(&self) -> Option<f32>` | Returns cached Expected Calibration Error (ECE). | `src/calibration/isotonic.rs` |
| `pub fn calibrated_probability(&mut self, raw_score: f32) -> Option<f32>` | Liefert die kalibrierte Wahrscheinlichkeit für einen Roh-Score. | `src/calibration/isotonic.rs` |
| `pub fn expected_calibration_error(&mut self) -> Option<f32>` | Calculates Expected Calibration Error over 10 bins. | `src/calibration/isotonic.rs` |
| `pub fn force_rebuild(&mut self)` | Forces immediate PAVA model rebuild. | `src/calibration/isotonic.rs` |
| `pub fn invalidate_on_config_change(&mut self, new_fingerprint: ConfigFingerprint)` | Resets observations if ConfigFingerprint changes (P8 Compliance). | `src/calibration/isotonic.rs` |
| `pub fn is_calibrated(&self) -> bool` | Checks if warmup observations threshold has been reached. | `src/calibration/isotonic.rs` |
| `pub fn last_calibration_at(&self) -> Option<u64>` | Returns timestamp (UNIX epoch in seconds) of last model rebuild. | `src/calibration/isotonic.rs` |
| `pub fn new(warmup_required: u32, max_observations: usize) -> Self` | Creates a new `IsotonicCalibrator`. | `src/calibration/isotonic.rs` |
| `pub fn observation_count(&self) -> usize` | Returns current observation count. | `src/calibration/isotonic.rs` |
| `pub fn record_outcome(&mut self, raw_score: f32, outcome: bool)` | Nimmt eine neue Beobachtung `(raw_score, outcome)` auf. | `src/calibration/isotonic.rs` |
| `pub fn with_clock(mut self, clock: Arc<dyn Clock>) -> Self` | Sets a custom clock port for deterministic time access. | `src/calibration/isotonic.rs` |
| `pub fn with_defaults() -> Self` | Creates an `IsotonicCalibrator` with defaults (Warmup: 50, Max Obs: 2000). | `src/calibration/isotonic.rs` |
| `pub fn apply(&self, logit: f32) -> f32` | Alias for `transform(logit)`. | `src/calibration/platt.rs` |
| `pub fn fit(observations: &[(f32, bool)]) -> Self` | Trainiert die Parameter A und B anhand von Beobachtungen mittels Gradientenabstieg. | `src/calibration/platt.rs` |
| `pub fn identity() -> Self` | Uncalibrated default fallback ($A=1.0, B=0.0$). | `src/calibration/platt.rs` |
| `pub fn invalidate_on_config_change(&mut self, new_fingerprint: ConfigFingerprint)` | Resets scaler to identity if ConfigFingerprint changes (P8 Compliance). | `src/calibration/platt.rs` |
| `pub fn is_fitted(&self) -> bool` | Checks if model has been fitted. | `src/calibration/platt.rs` |
| `pub fn is_identity(&self) -> bool` | Checks if this scaler is identical to uncalibrated default ($A=1.0, B=0.0$). | `src/calibration/platt.rs` |
| `pub fn new(a: f32, b: f32) -> Self` | Creates a new `PlattScaler` with specified parameters `a` and `b`. Falls back to identity if non-finite. | `src/calibration/platt.rs` |
| `pub fn params(&self) -> (f32, f32)` | Returns current parameters `(a, b)`. | `src/calibration/platt.rs` |
| `pub fn predict(&self, raw_score: f32) -> f32` | Calculates calibrated probability for raw score. | `src/calibration/platt.rs` |
| `pub fn transform(&self, logit: f32) -> f32` | Führt die logistische Transformation `sigmoid(A * logit + B)` aus. | `src/calibration/platt.rs` |
| `pub fn fuse_exact_prefix<I1, I2, I3>( mut state: DiBudFusionState, vector: &mut I1, text: &mut I2, graph: &mut I3, edge_reinforcement: impl Fn(DocId) -> f32, budget: &FusionBudget, ) -> Result<DiBudOutcome, ContextraError> where I1: Iterator<Item = DocId>, I2…` | Führt die DiBud-Streaming-Fusion synchron über Iterator-Kanäle aus. | `src/dibud/driver.rs` |
| `pub async fn fuse_exact_prefix_async<P, Fut>( mut state: DiBudFusionState, mut poll: P, edge_reinforcement: impl Fn(DocId) -> f32, budget: &FusionBudget, ) -> Result<DiBudOutcome, ContextraError> where P: FnMut(BudgetedChannel) -> Fut, Fut: Future<Output = Re…` | Führt die DiBud-Streaming-Fusion asynchron aus. | `src/dibud/driver.rs` |
| `pub fn accesses(&self) -> usize` | Returns total read accesses performed. | `src/dibud/state.rs` |
| `pub fn channel_depths(&self) -> [usize` | Returns current channel depths `[Vector, Text, Graph]`. | `src/dibud/state.rs` |
| `pub fn exhausted(&self) -> [bool` | Returns exhausted status for each channel `[Vector, Text, Graph]`. | `src/dibud/state.rs` |
| `pub fn feed( &mut self, ch: BudgetedChannel, item: Option<DocId>, er: impl Fn(DocId) -> f32, budget: &FusionBudget, ) -> Result<(), ContextraError>` | Feeds the result of reading a channel into the state machine. Raw scores are unknown in pure `DocId` streams and are set to 0.0 in `Provena… | `src/dibud/state.rs` |
| `pub fn new(capacity: usize) -> Self` | Creates a new `DiBudFusionState` with pre-allocated map capacities. | `src/dibud/state.rs` |
| `pub fn next_request(&self, budget: &FusionBudget) -> DiBudStep` | Bestimmt die nächste Abfrageaktion oder schließt die Fusion ab. | `src/dibud/state.rs` |
| `pub fn provenance_of(&self, doc_id: &DocId) -> Option<&ProvenanceRecord>` | Returns reference to provenance record for a given document if present. | `src/dibud/state.rs` |
| `pub fn validate(&self) -> Result<(), ContextraError>` | Validates budget parameter constraints. Ensures all weights are finite and non-negative, and minimum accesses/certified lengths are at leas… | `src/dibud/types.rs` |
| `pub fn analyze(&self) -> DriftStatus` | Analyzes current observations against baseline. | `src/drift.rs` |
| `pub fn as_str(&self) -> &'static str` | Returns human-readable summary string ("stabil", "warnung", "kritisch", or "unbekannt"). | `src/drift.rs` |
| `pub fn new(window_size: usize, threshold: f32) -> Self` | Creates a new `DriftDetector` with window size and sensitivity threshold. | `src/drift.rs` |
| `pub fn observe(&mut self, score: f32) -> DriftStatus` | Records a new score observation and calculates updated drift status. | `src/drift.rs` |
| `pub fn overall_drift_status(&self) -> &'static str` | Returns human-readable overall drift status string. | `src/drift.rs` |
| `pub fn set_baseline(&mut self, baseline: f32)` | Sets explicit baseline mean score. | `src/drift.rs` |
| `pub fn with_defaults() -> Self` | Creates a `DriftDetector` with default parameters (window size 50, threshold 0.15). | `src/drift.rs` |
| `pub fn explain(record: &ProvenanceRecord) -> RetrievalExplanation` | Erzeugt aus einem maschinenlesbaren `ProvenanceRecord` eine strukturierte `RetrievalExplanation`. | `src/explain.rs` |
| `pub fn to_human_readable_de(&self) -> String` | Erzeugt eine verständliche deutsche Textzusammenfassung für Audit-Logs oder UIs. | `src/explain.rs` |
| `pub fn config(&self) -> &GlobalFusionConfig` | Returns a reference to the configuration. | `src/fusion/global.rs` |
| `pub fn fuse( &self, result_sets: Vec<(String, Vec<SearchResult>, f32)>, max_results: usize, ) -> Result<Vec<SearchResult>, ContextraError>` | Fuses multi-signal search results using community-aware RRF. Candidates are grouped by community ID (extracted from candidate metadata `"co… | `src/fusion/global.rs` |
| `pub fn new(config: GlobalFusionConfig) -> Self` | Creates a new `GlobalFusionStrategy` with the given configuration. | `src/fusion/global.rs` |
| `pub fn score_normalized_fusion_with_options( mut result_sets: Vec<(String, Vec<SearchResult>, f32)>, max_results: usize, priority: MetadataMergePriority, include_provenance: bool, resonance_config: Option<&ResonanceConfig>, ) -> Vec<SearchResult>` | Führt eine score-normalisierte lineare Fusion mehrerer Signal-Ergebnismengen durch. | `src/fusion/normalized.rs` |
| `pub fn weights_to_signal_factors( weights: Option<&contextra_types::FusionWeights>, ) -> (f32, f32, f32)` | Konvertiert relative Signalgewichte in Normalisierungsfaktoren. | `src/fusion/normalized.rs` |
| `pub fn bm25(mut self, score: f32, rank: u32, weight: impl Into<Option<f32>>) -> Self` | Attaches BM25 text signal score, rank, and optional weight. | `src/fusion/provenance.rs` |
| `pub fn build(self) -> ProvenanceRecord` | Schließt die Konstruktion ab und berechnet die RRF-Beiträge je Signal. | `src/fusion/provenance.rs` |
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
| `pub fn apply_resonance_bonus( results: Vec<SearchResult>, valid_signal_count: usize, config: &ResonanceConfig, ) -> Vec<SearchResult>` | Wendet den Resonanz-Kohärenz-Bonus auf bereits fusionierte Suchergebnisse an. | `src/fusion/resonance.rs` |
| `pub fn fuse_signals( result_sets: Vec<(String, Vec<SearchResult>, f32)>, max_results: usize, ) -> Vec<FusedScore>` | Komfortfunktion zur Zusammenführung mehrerer Signal-Ergebnismengen mittels gewichteter RRF. | `src/fusion/resonance.rs` |
| `pub fn fuse_search_results_with_signal_strategies( mut result_sets: Vec<(String, Vec<SearchResult>, f32)>, max_results: usize, priority: MetadataMergePriority, include_provenance: bool, resonance_config: Option<&ResonanceConfig>, strategies: contextra_types::…` | Führt Teilergebnisse mit individuellen Strategien je Signalart zusammen. | `src/fusion/rrf.rs` |
| `pub fn fuse_search_results_with_strategy( result_sets: Vec<(String, Vec<SearchResult>, f32)>, max_results: usize, priority: MetadataMergePriority, include_provenance: bool, resonance_config: Option<&ResonanceConfig>, strategy: FusionStrategy, ) -> Vec<SearchR…` | Universeller Einstiegspunkt zur Signal-Fusion basierend auf einer globalen `FusionStrategy`. | `src/fusion/rrf.rs` |
| `pub fn g(x: f32) -> f32` | Glatte, beschränkte Einflussfunktion $g(x) = \frac{x}{1 + |x|}$. | `src/fusion/rrf.rs` |
| `pub fn modulate_and_renormalize_weights( result_sets: &[(String, Vec<SearchResult>, f32)], calibration_contexts: Option<&[SignalCalibrationContext]>, beta: f32, ) -> Vec<f32>` | Moduliert Signalgewichte anhand der Margin-Differenz (mRRF) und renormalisiert sie. | `src/fusion/rrf.rs` |
| `pub fn reciprocal_rank_fusion( result_sets: Vec<Vec<SearchResult>>, max_results: usize, ) -> Vec<SearchResult>` | Führt mehrere ungewichtete Signal-Ergebnislisten mittels Standard-RRF ($k=60$) zusammen. | `src/fusion/rrf.rs` |
| `pub fn weighted_reciprocal_rank_fusion( result_sets: Vec<(String, Vec<SearchResult>, f32)>, max_results: usize, ) -> Vec<SearchResult>` | Gewichtete reziproke Rangfusion (Weighted RRF) mit Standard-Metadatenpriorität. | `src/fusion/rrf.rs` |
| `pub fn weighted_reciprocal_rank_fusion_mrrf( mut result_sets: Vec<(String, Vec<SearchResult>, f32)>, max_results: usize, priority: MetadataMergePriority, include_provenance: bool, resonance_config: Option<&ResonanceConfig>, calibration_contexts: Option<&[Sign…` | Gewichtete Rangfusion mit margin-basierter mRRF-Gewichtsmodulation. | `src/fusion/rrf.rs` |
| `pub fn weighted_reciprocal_rank_fusion_with_options( result_sets: Vec<(String, Vec<SearchResult>, f32)>, max_results: usize, priority: MetadataMergePriority, include_provenance: bool, resonance_config: Option<&ResonanceConfig>, ) -> Vec<SearchResult>` | Vollständig konfigurierbare gewichtete Rangfusion mit Resonanz- und Provenance-Optionen. | `src/fusion/rrf.rs` |
| `pub fn weighted_reciprocal_rank_fusion_with_priority( result_sets: Vec<(String, Vec<SearchResult>, f32)>, max_results: usize, priority: MetadataMergePriority, ) -> Vec<SearchResult>` | Gewichtete reziproke Rangfusion mit expliziter Metadaten-Zusammenführungspriorität. | `src/fusion/rrf.rs` |
| `pub fn as_str(&self) -> &'static str` | Converts `SignalKind` to its canonical string representation. | `src/fusion/signal.rs` |
| `pub fn from_name(name: &str) -> Option<Self>` | Identifies `SignalKind` from a signal name string. | `src/fusion/signal.rs` |
| `pub fn name_de(&self) -> &'static str` | Returns the German display name for the signal. | `src/fusion/signal.rs` |
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

**`BanditDefaultFlipGate`** — ** (`src/flow_thompson.rs`)

```rust
pub trait BanditDefaultFlipGate {
/// `true` ⇔ alle drei Bedingungen erfüllt:
    /// 1. `sample_count >= MIN_BANDIT_SHADOW_SAMPLES` (10_000)
    /// 2. `cumulative_regret <= MAX_CUMULATIVE_REGRET_THRESHOLD` (0.0)
    /// 3. `p99_latency_delta_us <= 0.0`
    fn should_flip(&self, report: &BanditShadowReport) -> bool;
}
```

**`BanditPolicy`** — ** (`src/bandit.rs`)

```rust
pub trait BanditPolicy : Send + Sync {
/// Wendet Drift-Penalty / Covariance Discounting bei erkannter concept drift an.
    fn apply_drift_penalty(&mut self, k_drift: f32, alpha_max: f32, gamma: f32);
}
```

**`DriftDetector`** — *Generic interface for sequential O(1) drift detectors. # Abgrenzung zu `contextra-rank::DriftDetector` `contextra-adapt::DriftDetector` ist ein O(1) sequentielles Online-Signal-Trait für synchrone Regler und Bandit-Policies in Ring 0. Es arbeitet direkt auf S…* (`src/drift.rs`)

```rust
pub trait DriftDetector {
/// Observes a reward or non-conformity score and returns the updated drift signal.
    fn observe(&mut self, reward: f32) -> DriftSignal;
    /// Returns `true` if drift is currently detected.
    fn is_drifting(&self) -> bool;
}
```

**`FcTsRng`** — ** (`src/flow_thompson.rs`)

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

**`OffPolicyCompatibility`** — ** (`src/flow_thompson.rs`)

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
| `AlignedF32Vec` | `(tuple/unit)` | — | `src/bandit.rs` |
| `BanditProfileState` | `{ /// Gewichtsvektor θ_p (Dimension = Embedding-Dim). pub theta: Vec<f32>, /// Diagonale Kovarianz-Terme σ²_p (Diagonal-Approximation). pub sigma_sq: Vec<f32>, /… }` | — | `src/bandit.rs` |
| `BanditShadowReport` | `{ /// Anzahl ausgewerteter Shadow-Mode-Samples. pub sample_count: u64, /// Kumulatives Regret des FC-TS-Kandidaten im Vergleich zur Baseline. pub cumulative_regr… }` | — | `src/flow_thompson.rs` |
| `CatoniDriftDetector` | `{ /// Robust mean estimate $\mu$. pub catoni_mu: f64, /// CUSUM sequential change detection accumulator. pub cusum_sum: f64, /// Threshold for declaring drift de… }` | Catoni-M-estimator based robust drift detector (arXiv:2505.20051 & arXiv:2501.10974). Uses influence function $\psi(x) = \frac{x}{1 + |x|}$ for $O(1)… | `src/drift.rs` |
| `DecayControllerConfig` | `{ /// Verstärkungsfaktor κ. Default: 2.0. pub kappa: f32, /// Basis-Half-Life in TxId-Einheiten. Default: 10_000. pub base_half_life_tx: u64, /// Eviction-Schwel… }` | Konfiguration für Adaptive Decay Controller (Cache-Eviction mit Time-Weighted Decay) (via MaintenanceConfig). | `src/decay_controller.rs` |
| `DecaySignalInputs` | `{ /// HNSW-Tombstone-Ratio ∈ `[0, 1]`. Quelle: HnswIndex::tombstone_ratio(). pub tombstone_ratio: f32, /// Normalisierte inverse Query-Rate ∈ `[0, 1]`. /// Berec… }` | Inputs for Adaptive Decay Controller (Cache-Eviction mit Time-Weighted Decay) aus bestehenden System-Metriken. | `src/decay_controller.rs` |
| `DefaultBanditFlipGate` | `(tuple/unit)` | — | `src/flow_thompson.rs` |
| `DiagonalApproximationBandit` | `{ /// Propensities P(a \| x) pro RetrievalStrategy. pub propensities: std::collections::HashMap<contextra_types::RetrievalStrategy, f32>, }` | — | `src/flow_thompson.rs` |
| `DriftPolicyBridge` | `{ /// Der zugrundeliegende Lyapunov-Drift-Wächter. pub watcher: LyapunovDriftWatcher, /// Multiplikator k_drift für die Alpha-Erhöhung bei Drift. pub k_drift: f3… }` | Adapter zur Kopplung des `LyapunovDriftWatcher` mit einer `BanditPolicy`. | `src/drift.rs` |
| `DriftReason` | `{ /// Aktuelle Kullback-Leibler-Divergenz D_t = KL(N_t \|\| N_baseline). pub kl_divergence: f32, /// Diskreter Lyapunov-Exponent λ_t über das gleitende Fenster. pu… }` | Grund für erkannte Verteilungsverschiebung (Distributional Drift). | `src/lyapunov.rs` |
| `EnsembleDriftWatcher` | `{ /// Primary Lyapunov drift watcher instance. pub lyapunov: LyapunovDriftWatcher, /// Secondary Catoni robust drift detector instance. pub catoni: CatoniDriftDe… }` | Ensemble combination of [`LyapunovDriftWatcher`] and [`CatoniDriftDetector`]. Triggers drift response ONLY if BOTH detectors independently detect dri… | `src/drift.rs` |
| `FcTsArmSet` | `{ /// Liste der Bandit-Arme. pub arms: Vec<FlowCorrectedThompsonBandit>, }` | — | `src/flow_thompson.rs` |
| `FcTsConfig` | `{ /// Feature-Dimension $d$. pub dim: usize, /// Ridge-Prior Parameter $\lambda > 0$. pub lambda: f32, /// Rauschvarianz $\sigma^2 > 0$. pub noise_var: f32, ///… }` | — | `src/flow_thompson.rs` |
| `FcTsSamplingDistribution` | `{ /// Mapping von RetrievalStrategy zu FC-TS-Sampling-Wahrscheinlichkeit P_FCTS(a \| x). pub probabilities: std::collections::HashMap<contextra_types::RetrievalSt… }` | — | `src/flow_thompson.rs` |
| `FlowCorrectedThompsonBandit` | `{ config: FcTsConfig, /// Inverse Kovarianzmatrix $A^{-1}$ ($d \times d$, row-major). inv_a: Vec<f32>, /// Akkumulator-Vektor $\eta$ ($d$ Elemente). eta: Vec<f32… }` | — | `src/flow_thompson.rs` |
| `LatencyBudgetGuard` | `{ start_nanos: u64, budget_ms: f64, clock: Arc<C>, }` | Simple, panic-free latency budget guard for iterative multi-step search loops. | `src/pid_latency_controller.rs` |
| `LyapunovDriftWatcher` | `{ /// Fenstergröße w für die Lyapunov-Exponenten-Berechnung (Default: 20). pub window_size: usize, /// Historie der berechneten KL-Divergenzen D_t. pub divergenc… }` | Proaktiver Drift-Wächter auf Basis diskreter Lyapunov-Exponenten über KL-Divergenzen. | `src/lyapunov.rs` |
| `OffPolicyEvaluator` | `{ cumulative_ips: f64, samples: u64, discarded: u64, }` | Evaluator für Inverse Propensity Scoring (IPS) zur kontrafaktischen Off-Policy-Schätzung. | `src/offpolicy.rs` |
| `OffPolicyStats` | `{ /// Anzahl verworfener Samples aufgrund ungültiger Propensity-Werte (NaN, < 0.0, > 1.0). pub discarded: u64, }` | Statistiken über verworrene/ungültige Samples der Off-Policy-Evaluation. | `src/offpolicy.rs` |
| `PidController` | `{ /// Proportionaler Gewichtsbeiwert $K_p$. pub kp: f32, /// Integraler Gewichtsbeiwert $K_i$. pub ki: f32, /// Differentieller Gewichtsbeiwert $K_d$. pub kd: f3… }` | PID-Regler zur dynamischen Steuerung der Reranking-Kandidatenpool-Größe basierend auf Latenzmessungen. # Anti-Windup Strategien - Standard-Methode [`… | `src/pid.rs` |
| `PidLatencyController` | `{ /// Proportional gain coefficient. pub kp: f64, /// Integral gain coefficient. pub ki: f64, /// Derivative gain coefficient. pub kd: f64, /// Target latency bu… }` | Classical PID controller for dynamically scaling multi-step retrieval parameters (`k_pool`, `max_hops`). HINWEIS ZUR ANTI-WINDUP STRATEGIE: Dieser Co… | `src/pid_latency_controller.rs` |
| `RandomizedLoggingPolicy` | `{ /// Exploration-Wahrscheinlichkeit epsilon (ε in [0.0, 1.0]). pub epsilon: f32, /// Anzahl verfügbarer Aktionen K (K >= 1). pub num_actions: u32, }` | Randomisierte Logging-Policy für unverzerrte Datensammlung zur Off-Policy-Evaluation (§B.5.2.4 & §8.5). | `src/offpolicy.rs` |
| `RerankDeadline` | `{ /// Maximum time budget allowed for reranking operations. pub budget: Duration, clock: std::sync::Arc<C>, }` | Hard deadline manager for candidate retrieval and Cross-Encoder reranking phases (P11 requirement). # Intended Integration Point Evaluated before or… | `src/homeostat.rs` |
| `RerankPidController` | `{ inner: crate::PidController, }` | — | `src/homeostat.rs` |
| `RieGreedyProfile` | `{ /// Inverse Präzisionsmatrix $\Lambda^{-1}$ ($d \times d$, row-major). pub precision_inv: Vec<f32>, /// Informationsvektor $\eta$ ($d$ Elemente). pub info_vect… }` | Laufzeitzustand eines RIE-Greedy-Profils (§10.4.1). Implementiert die deterministisch-gierige Personalisierung mit zeitlich diskontierter Präzisionsm… | `src/rie_greedy.rs` |
| `Ring3Token` | `{ _private: (), }` | — | `src/flow_thompson.rs` |
| `ShadowDiscrepancy` | `{ /// Das Ergebnis des etablierten Baseline-Algorithmus. pub baseline: T, /// Das Ergebnis des neuen Kandidaten-Algorithmus. pub candidate: T, /// Eindeutige Kon… }` | Diskrepanz-Eintrag: Baseline-Ergebnis vs. Kandidaten-Ergebnis, ohne Einfluss auf das tatsächlich zurückgegebene Nutzerergebnis (§12 Integrationsregel… | `src/shadow_mode.rs` |
| `SketchMatrix` | `{ /// Flacher Speicher der Matrix (Row-Major, k Zeilen x d Spalten). pub data: AlignedF32Vec, /// Projektionsdimension k. pub projected_dim: usize, /// Ursprüngl… }` | — | `src/bandit.rs` |
| `SplitMix64` | `{ state: u64, }` | — | `src/flow_thompson.rs` |
| `TracingShadowSink` | `(tuple/unit)` | Referenzimplementierung über `tracing::debug!` — kein zusätzlicher I/O-Pfad, keine Rng/Clock-Portverletzung (P28), da rein diagnostisch. | `src/shadow_mode.rs` |

### Öffentliche Datentypen — Enums (7)

| Enum | Varianten | Beschreibung | Datei |
|---|---|---|---|
| `BanditError` | `/// Dimension des Eingabevektors stimmt nicht mit dem Bandit-Zustand überein. #[error("Embedding dimension mismatch: expected {expected}, actual {actual}")] DimensionMismatch { /// Erwartete Dimensio…` | — | `src/bandit.rs` |
| `BanditImplementation` | `/// Sherman-Morrison O(d²): Produktions-Default. #[default] ShermanMorrison, /// Diagonal-Approximation O(d): Opt-in-Variante. DiagonalApproximation, /// Flow-Corrected Thompson Sampling (§21.3). ///…` | — | `src/bandit.rs` |
| `DriftSignal` | `/// Signal distribution is stable. Stable, /// Distributional drift detected. Detected,` | Drift detection signal returned by a [`DriftDetector`]. | `src/drift.rs` |
| `FcTsError` | `/// Dimension des Eingabevektors stimmt nicht mit der konfigurierten Arm-Dimension überein. #[error("Embedding dimension mismatch: expected {expected}, actual {actual}")] DimensionMismatch { /// Erwa…` | — | `src/flow_thompson.rs` |
| `LyapunovResult` | `/// Die Score-Verteilung ist stabil (λ_t <= 0.0). Stable { /// Diskreter Lyapunov-Exponent λ_t. lyapunov_exponent: f32, }, /// Proaktive Drift erkannt (λ_t > 0.0 über das Fenster). DriftDetected { //…` | Ergebnis der Lyapunov-Drift-Analyse. | `src/lyapunov.rs` |
| `OffPolicyError` | `/// Ein Arm hat Propensity 0 unter der Referenzpolicy, aber eine positive FC-TS-Sampling-Wahrscheinlichkeit. #[error("arm {0:?} has zero propensity under reference policy but positive FC-TS sampling…` | — | `src/flow_thompson.rs` |
| `RieGreedyError` | `/// Dimension des Kontext-Vektors stimmt nicht mit der konfigurierten Profil-Dimension überein. #[error("Embedding dimension mismatch: expected {expected}, actual {actual}")] DimensionMismatch { ///…` | Fehlerzustände für den RIE-Greedy-Personalismus-Mechanismus (§10.4.1). | `src/rie_greedy.rs` |

### Konstanten (9)

| Konstante | Typ | Beschreibung |
|---|---|---|
| `DEFAULT_TARGET_LATENCY_MS` | `f64` | Konservativer Default für das P95-Latenzziel in Millisekunden (100ms). HINWEIS: implementation_plan-db2.md identifiziert den genauen P95-Zielwert als… |
| `MAX_CUMULATIVE_REGRET_THRESHOLD` | `f64` | Maximales zulässiges kumulatives Regret für den Bandit-Default-Flip (≤ 0.0). |
| `MAX_INTEGRAL` | `f64` | Max-Grenze für den I-Anteil (Anti-Windup Clamping). |
| `MAX_SCALING_FACTOR` | `f64` | Oberer Clamp für den K-Pool-Skalierungsfaktor (100% des konfigurierten K-Pools). |
| `MIN_BANDIT_SHADOW_SAMPLES` | `u64` | — |
| `MIN_SCALING_FACTOR` | `f64` | Unterer Clamp für den K-Pool-Skalierungsfaktor (30% des konfigurierten K-Pools), um APM-RECALL-COLLAPSE in Standardlastszenarien zu verhindern. |
| `PID_MAX_POOL_SIZE_DEFAULT` | `usize` | Maximale Kandidaten-Pool-Größe. |
| `PID_MIN_POOL_SIZE_DEFAULT` | `usize` | Minimale Kandidaten-Pool-Größe (Hard Floor = 50, Quality Knee per arXiv:2604.01733). |
| `SHERMAN_MORRISON_REFACTORIZATION_INTERVAL` | `u64` | — |

### Öffentliche Funktionen (99)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn cold_start(d: usize, alpha_base: f32) -> Self` | — | `src/bandit.rs` |
| `pub fn derive_seed(tenant_id: &contextra_types::TenantId, base_seed: u64) -> u64` | — | `src/bandit.rs` |
| `pub fn ensure_sketched_state(&mut self, projected_dim: usize) -> Result<(), BanditError>` | — | `src/bandit.rs` |
| `pub fn expected_dim(&self) -> usize` | Gibt die erwartete Dimension der Feature-Vektoren zurück. | `src/bandit.rs` |
| `pub fn from_seed( seed: u64, original_dim: usize, projected_dim: usize, ) -> Result<Self, BanditError>` | — | `src/bandit.rs` |
| `pub fn new(vec: Vec<f32>) -> Self` | Erstellt einen neuen `AlignedF32Vec` aus einem `Vec<f32>`. | `src/bandit.rs` |
| `pub fn on_drift_detected(&mut self, k_drift: f32)` | — | `src/bandit.rs` |
| `pub fn project(&self, x: &[f32]) -> Vec<f32>` | — | `src/bandit.rs` |
| `pub fn score( &self, x: &[f32], cost: f32, is_cloud_transport: bool, ) -> Result<f32, BanditError>` | — | `src/bandit.rs` |
| `pub fn update( &mut self, x: &[f32], r_outcome: f32, cost: f32, is_cloud_transport: bool, ) -> Result<(), BanditError>` | — | `src/bandit.rs` |
| `pub fn zeros(len: usize) -> Self` | Erstellt einen neuen `AlignedF32Vec` mit `len` Nullen. | `src/bandit.rs` |
| `pub fn config(&self) -> &DecayControllerConfig` | Liefer eine Referenz auf die Konfiguration zurück. | `src/decay_controller.rs` |
| `pub fn effective_half_life(&self, temperature: f32) -> f32` | Effektive Half-Life unter aktueller Temperatur. | `src/decay_controller.rs` |
| `pub fn effective_score( &self, base_score: f32, elapsed_tx: u64, inputs: &DecaySignalInputs, ) -> f32` | Temperatur-adjustierter Score für einen Chunk. `elapsed_tx`: TxId(now) - TxId(chunk_created). ADR-016: kein SystemTime. | `src/decay_controller.rs` |
| `pub fn new(config: DecayControllerConfig) -> Self` | Erstellt einen neuen `AdaptiveDecayController` mit der angegebenen Konfiguration. | `src/decay_controller.rs` |
| `pub fn should_evict( &self, base_score: f32, elapsed_tx: u64, inputs: &DecaySignalInputs, ) -> bool` | Gibt true wenn Chunk unter Eviction-Schwelle. Wird von Background Workers (background_workers.rs) für Sweep verwendet. | `src/decay_controller.rs` |
| `pub fn system_temperature(&self, inputs: &DecaySignalInputs) -> f32` | Berechnet aktuelle Systemtemperatur T(t) ∈ `[0, 1]`. | `src/decay_controller.rs` |
| `pub fn with_defaults() -> Self` | Erstellt einen neuen `AdaptiveDecayController` mit Standardwerten. | `src/decay_controller.rs` |
| `pub fn new(baseline_reward: f64, threshold: f64, alpha: f64) -> Self` | Creates a new `CatoniDriftDetector` with specified baseline, threshold, and learning rate. | `src/drift.rs` |
| `pub fn new(lyapunov: LyapunovDriftWatcher, catoni: CatoniDriftDetector) -> Self` | Creates a new `EnsembleDriftWatcher`. | `src/drift.rs` |
| `pub fn new(window_size: usize, k_drift: f32, alpha_max: f32, gamma: f32) -> Self` | Erstellt eine neue `DriftPolicyBridge` mit benutzerdefinierten Parametern. | `src/drift.rs` |
| `pub fn observe_and_decide(&mut self, reward: f32) -> bool` | Observes a new reward / score and decides if drift is detected by BOTH detectors. | `src/drift.rs` |
| `pub fn observe_and_react<P: crate::bandit::BanditPolicy + ?Sized>( &mut self, current_scores: &[f32], policy: &mut P, ) -> LyapunovResult` | Nimmt aktuelle Non-Conformity-Scores auf, wertet den Drift-Status aus und wendet bei erkannter Drift eine Strafe auf die `BanditPolicy` an. | `src/drift.rs` |
| `pub fn reset(&mut self)` | Resets the CUSUM accumulator and restores `catoni_mu` to baseline. | `src/drift.rs` |
| `pub fn set_baseline(&mut self, baseline_scores: &[f32])` | Sets the baseline distribution for the Lyapunov watcher and baseline mean for the Catoni detector. | `src/drift.rs` |
| `pub fn baseline_score(&self, context: &[f32]) -> Result<f32, FcTsError>` | Berechnet den Baseline-Mittelwert-Score (Sherman-Morrison $\langle \mu, x \rangle$) für den Kontext $x$. | `src/flow_thompson.rs` |
| `pub fn config(&self) -> &FcTsConfig` | Gibt eine Referenz auf die Konfiguration zurück. | `src/flow_thompson.rs` |
| `pub fn drift_rate(&self) -> &[f32]` | Gibt die aktuelle Drift-Rate $\delta$ zurück. | `src/flow_thompson.rs` |
| `pub fn from_pairs( pairs: impl IntoIterator<Item = (contextra_types::RetrievalStrategy, f32)>, ) -> Self` | Erstellt eine `DiagonalApproximationBandit`-Referenzpolitik aus Strategie-Propensitäts-Paaren. ``` use contextra_adapt::flow_thompson::Diag… | `src/flow_thompson.rs` |
| `pub fn from_pairs( pairs: impl IntoIterator<Item = (contextra_types::RetrievalStrategy, f32)>, ) -> Self` | Erstellt eine `FcTsSamplingDistribution`-Instanz aus Strategie-Wahrscheinlichkeits-Paaren. ``` use contextra_adapt::flow_thompson::FcTsSamp… | `src/flow_thompson.rs` |
| `pub fn inv_a(&self) -> &[f32]` | Gibt die inverse Kovarianzmatrix $A^{-1}$ ($d \times d$, row-major) zurück. | `src/flow_thompson.rs` |
| `pub fn mu(&self) -> &[f32]` | Gibt den aktuellen Mittelwert-Vektor $\mu$ zurück. | `src/flow_thompson.rs` |
| `pub fn new(seed: u64) -> Self` | Erstellt eine neue `SplitMix64`-Instanz mit dem gegebenen Seed. | `src/flow_thompson.rs` |
| `pub fn new(config: FcTsConfig) -> Result<Self, FcTsError>` | Erstellt eine neue FC-TS-Bandit-Instanz mit der gegebenen Konfiguration. | `src/flow_thompson.rs` |
| `pub fn new( propensities: std::collections::HashMap<contextra_types::RetrievalStrategy, f32>, ) -> Self` | Erstellt eine neue `DiagonalApproximationBandit`-Referenzpolitik mit der gegebenen Propensitätsmap. | `src/flow_thompson.rs` |
| `pub fn new( probabilities: std::collections::HashMap<contextra_types::RetrievalStrategy, f32>, ) -> Self` | Erstellt eine neue `FcTsSamplingDistribution`-Instanz mit der gegebenen Wahrscheinlichkeitsmap. | `src/flow_thompson.rs` |
| `pub fn probability(&self, strategy: contextra_types::RetrievalStrategy) -> f32` | Liefert die FC-TS-Sampling-Wahrscheinlichkeit für eine gegebene RetrievalStrategy. | `src/flow_thompson.rs` |
| `pub fn propensity(&self, strategy: contextra_types::RetrievalStrategy) -> f32` | Liefert die Propensity P(a | x) für eine gegebene RetrievalStrategy. | `src/flow_thompson.rs` |
| `pub fn recompute_drift_rate_from_window(&mut self, _token: &Ring3Token)` | Berechnet die Drift-Rate $\delta$ aus dem aktuellen Fenster neu via Ridge-Regression (AK-18). Nutzt Conjugate Gradient zur Lösung von $(Z^T… | `src/flow_thompson.rs` |
| `pub fn ring3_background_task_token() -> Ring3Token` | — | `src/flow_thompson.rs` |
| `pub fn sample_score(&self, context: &[f32], rng: &mut dyn FcTsRng) -> Result<f32, FcTsError>` | Berechnet einen zufälligen Thompson-Sample Score für den Kontext $x$. $w_i = \mu_i + v \cdot \sqrt{\max(A^{-1}_{i,i}, 0)} \cdot z_i$, $\tex… | `src/flow_thompson.rs` |
| `pub fn select_arm(&self, context: &[f32], rng: &mut dyn FcTsRng) -> Result<u32, FcTsError>` | Wählt die Aktion für diese Bandit-Instanz und protokolliert Diskrepanzen bei konfiguriertem Shadow-Sink. | `src/flow_thompson.rs` |
| `pub fn select_arm(&self, context: &[f32], rng: &mut dyn FcTsRng) -> Result<u32, FcTsError>` | Wählt den Arm mit dem höchsten gesampelten Score aus (Argmax). Bei Gleichstand wird der kleinste Arm-Index gewählt. Bei konfiguriertem Shad… | `src/flow_thompson.rs` |
| `pub fn shadow_sink(&self) -> Option<&std::sync::Arc<dyn crate::shadow_mode::ShadowSink<u32>>>` | Gibt eine Referenz auf die optional konfigurierte Diskrepanz-Senke zurück. | `src/flow_thompson.rs` |
| `pub fn update_with_flow( &mut self, context: &[f32], reward: f32, observation_time: u64, confidence_weight: f32, ) -> Result<(), FcTsError>` | Akkumuliert ein neues Beobachtungstupel mit konfidenzgewichtetem Transport (§21.3). # Parameter - `context`: Kontext-Vektor $x \in \mathbb{… | `src/flow_thompson.rs` |
| `pub fn validate(&self) -> Result<(), FcTsError>` | Validiert die Konfigurationsparameter. | `src/flow_thompson.rs` |
| `pub fn with_shadow_sink( mut self, sink: std::sync::Arc<dyn crate::shadow_mode::ShadowSink<u32>>, ) -> Self` | Konfiguriert eine Diskrepanz-Senke für den Shadow-Mode-Vergleich (§12). | `src/flow_thompson.rs` |
| `pub fn with_shadow_sink( mut self, sink: std::sync::Arc<dyn crate::shadow_mode::ShadowSink<u32>>, ) -> Self` | Konfiguriert eine Diskrepanz-Senke für alle Arme in der Menge (§12). | `src/flow_thompson.rs` |
| `pub fn deadline_exceeded(&self, started_at: std::time::Instant) -> bool` | Legacy helper using monotonic_nanos delta based on Instant baseline. | `src/homeostat.rs` |
| `pub fn deadline_exceeded_nanos(&self, started_at_nanos: u64) -> bool` | Checks if elapsed nanoseconds since `started_at_nanos` equals or exceeds deadline budget. | `src/homeostat.rs` |
| `pub fn k_max(&self) -> usize` | Accessor for `k_max`. | `src/homeostat.rs` |
| `pub fn k_min(&self) -> usize` | Accessor for `k_min`. | `src/homeostat.rs` |
| `pub fn k_pool(&self) -> usize` | Accessor for current `k_pool`. | `src/homeostat.rs` |
| `pub fn kd(&self) -> f32` | Accessor for `kd`. | `src/homeostat.rs` |
| `pub fn ki(&self) -> f32` | Accessor for `ki`. | `src/homeostat.rs` |
| `pub fn kp(&self) -> f32` | Accessor for `kp`. | `src/homeostat.rs` |
| `pub fn new( target_p95_latency_ms: f32, k_min: usize, k_max: usize, initial_pool: usize, ) -> Self` | Creates a new `RerankPidController` with default PID coefficients and configured parameters. | `src/homeostat.rs` |
| `pub fn new(budget: Duration) -> Self` | Creates a new `RerankDeadline` with the specified duration budget. | `src/homeostat.rs` |
| `pub fn new_with_clock(budget: Duration, clock: std::sync::Arc<C>) -> Self` | Creates a new `RerankDeadline` with explicit `Clock` port injection. | `src/homeostat.rs` |
| `pub fn start_nanos(&self) -> u64` | Returns current monotonic timestamp in nanoseconds from injected clock. | `src/homeostat.rs` |
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
| `pub fn new(budget_ms: f64) -> Self` | Creates a new `LatencyBudgetGuard` with the given budget in milliseconds using `SystemClock`. | `src/pid_latency_controller.rs` |
| `pub fn new_with_clock(budget_ms: f64, clock: Arc<C>) -> Self` | Creates a new `LatencyBudgetGuard` with an injected [`Clock`] port for deterministic time tracking. | `src/pid_latency_controller.rs` |
| `pub fn reset(&mut self)` | Resets the controller integral state and error history. | `src/pid_latency_controller.rs` |
| `pub fn with_params(kp: f64, ki: f64, kd: f64, target_latency_ms: f64) -> Self` | Custom constructor specifying gains and target latency. | `src/pid_latency_controller.rs` |
| `pub fn mu_hat(&self) -> Result<Vec<f32>, RieGreedyError>` | Berechnet die Parameter-Schätzung $\hat{\mu}_t = \Lambda_t^{-1} \eta_t$. | `src/rie_greedy.rs` |
| `pub fn new(dim: usize, lambda: f32, gamma: f32) -> Self` | Erstellt ein neues RIE-Greedy-Profil mit initialer Präzisionsmatrix $\Lambda_0 = \lambda I_d$. | `src/rie_greedy.rs` |
| `pub fn predict(&self, context: &[f32]) -> Result<f32, RieGreedyError>` | Berechnet die deterministisch-gierige Erwartungswert-Vorhersage $\hat{\mu}_t \cdot x$. | `src/rie_greedy.rs` |
| `pub fn trace_precision_inv(&self) -> f32` | Berechnet die Spur der inversen Präzisionsmatrix $\text{Tr}(\Lambda^{-1})$. | `src/rie_greedy.rs` |
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

`Contextra` Core — Typen, Traits und Fehlerbehandlung (Strangler Facade).  # Architektur-Rolle (Ring 0 / Legacy-Schicht 0 -> Ring 0-4 Migration)  Im Zuge der Migration auf das Ring-Modell (Phase 1b / SPEC §5.2) wurde die monolithische Funktionalität der früheren `contextra-core`-Crate in eigenständige, modulare Ring-0-Crates zerlegt: - [`contextra_types`]: Kanonische Domänenmodelle, Bezeichner (`DocId`, `TxId`), Ressourcenbudgets, Filter und das zentrale Fehlersystem (`ContextraError`). - [`contextra_ports`]: Abstrakte, `dyn`-kompatible Subsystem-Schnittstellen (Ports) gemäß der Hexagonalen Architektur (z. B. `StorageEngine`, `Clock`, `Rng`). - [`contextra_mvcc`]: Multi-Version Concurrency Control (Mehrversions-Nebenläufigkeitssteuerung), Sequenz-Log (`SequenceLog`) und Transaktionspuffer (`TxBuffer`). - [`contextra_wire`]: Zero-Copy-IPC (Inter-Process Communication) via FlatBuffers und Wire-Nachrichtenadapter.  Diese Crate fungiert als abwärtskompatible **Strangler Facade** (Strangler-Muster nach Martin Fowler). Sie leitet alle historischen Typen und Module per Re-Export (`pub use`) weiter, damit bestehende Aufrufer schrittweise ohne Breaking Changes migriert werden können.  ## Status und Ring-Zugehörigkeit - **Ring:** Ring 0 (Basis-Typen und Abstraktionen ohne zyklische Abhängigkeiten). - **Status:** Veraltet (`deprecated`). Die Crate soll nach vollständiger Migration aller Konsumenten auf die spezialisierten Crates entfernt werden (SPEC §5.2 / WP 2.8).  ## Konsumenten (Wer nutzt diese Crate?) - [`contextra`]: Haupt-Fassadencrate. - [`contextra-engine`]: Core-Engine zur Ausführung von Speicher-, Such- und Löschoperationen.  ## Nebenläufigkeits- und Sperrkonzept In `contextra-core` selbst existieren keine internen Mutexe oder Sperren (`#![forbid(unsafe_code)]`). Die nebenläufige Synchronisation erfolgt vollständig in den re-exportierten Modulen: - `TxBuffer` nutzt partitioniertes/sharded Lock-Striping (über `DashMap`) für sperrenfreie oder feingranulare Nebenläufigkeit beim Staging von Index-Operationen. - `SnapshotRegistry` verwaltet atomare Sequenznummern (`AtomicU64`) für lock-free Leseschnappschüsse.  ## Persistenz und I/O Re-exportiert Schnittstellen (`traits`) zur Interaktion mit WAL (Write-Ahead-Log) und VFS (Virtual File System). E/A-Operationen und `fsync`-Garantien werden von den implementierenden Speichercrates (wie `contextra-store`) bereitgestellt. Fehler werden strikt über den kanonischen Fehlertyp [`ContextraError`] nach dem Fail-Closed-Prinzip propagiert.  ## Determinismus und Ports (Regel 3 aus CLAUDE.md) Zeit (`Clock`), Zufall (`Rng`/`EntropyRng`) und ID-Generierung (`IdGenerator`) dürfen in produktiven Pfaden niemals über undeterministische Systemaufrufe (`SystemTime::now`, `thread_rng`) erfolgen, sondern müssen über die in `traits` re-exportierten Ports injiziert werden.

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

`contextra-audit-export` — GDPR Article 30 Processing Register export generator.  # Architecture Role (Ring 0) This crate provides an external interface for generating automated processing registers (Verzeichnis von Verarbeitungstätigkeiten nach Art. 30 DSGVO) based on egress gateway logs and cryptographic deletion proofs.

### Modul- und Re-Export-Oberfläche (lib.rs)

```rust
pub mod bsi_mapping;
pub mod error;
pub mod markdown_template;
pub mod testkit;
pub use bsi_mapping::{bsi_mapping_table, render_bsi_mapping_markdown, BsiMappingEntry};
pub use error::AuditExportError;
pub use markdown_template::render_register_markdown;
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

### Öffentliche Datentypen — Structs (8)

| Struct | Felder / Form | Beschreibung | Datei |
|---|---|---|---|
| `BsiMappingEntry` | `{ /// Name of the cryptographic primitive (e.g. "Ed25519"). pub primitive_name: String, /// Code location path within the workspace (e.g. "crates/contextra-crypt… }` | A mapping entry linking a cryptographic primitive in the codebase to a BSI Technical Guideline & IT-Grundschutz Baustein. | `src/bsi_mapping.rs` |
| `ControllerContact` | `{ /// Name or company name of the data controller (Verantwortlicher). pub controller_name: String, /// Contact details of the data controller (e.g., address, ema… }` | Contact details for the data controller, representative, and data protection officer (Art. 30 Abs. 1 lit. a DSGVO). | `src/lib.rs` |
| `DeletionProofSummary` | `{ /// Unique identifier of the deletion proof. pub proof_id: String, /// Target resource scope or deleted item identifier. pub scope: String, /// Timestamp in Un… }` | Summary of a cryptographic deletion proof (GDPR Art. 17 / Art. 30 compliance). | `src/lib.rs` |
| `EgressEventSummary` | `{ /// Unique identifier of the egress event. pub event_id: String, /// Target destination or external endpoint. pub destination: String, /// Timestamp in Unix na… }` | Summary of an egress gateway event recorded for audit verification. | `src/lib.rs` |
| `InMemoryProcessingRegisterSource` | `{ entries: Vec<ProcessingRegisterEntry>, }` | In-memory implementation of [`ProcessingRegisterSource`] for testing purposes. | `src/testkit.rs` |
| `ProcessingRegisterEntry` | `{ /// Associated tenant identifier (`contextra_types::TenantId`). pub tenant_id: TenantId, /// Contact details of data controller, representative, and DSB (Art.… }` | A record entry in the GDPR Article 30 processing register. | `src/lib.rs` |
| `RetentionPeriod` | `{ /// Category of personal data associated with this retention policy. pub data_category: String, /// Time limit or rule for erasure / retention period (Löschfri… }` | Retention period definition per category of personal data (Art. 30 Abs. 1 lit. f DSGVO). | `src/lib.rs` |
| `ThirdCountryTransfer` | `{ /// Destination third country or international organization. pub destination_country: String, /// Legal safeguards / guarantees (e.g., EU Standard Contractual… }` | Third country transfer record including legal safeguard guarantees (Art. 30 Abs. 1 lit. e DSGVO). | `src/lib.rs` |

### Öffentliche Datentypen — Enums (1)

| Enum | Varianten | Beschreibung | Datei |
|---|---|---|---|
| `AuditExportError` | `/// Failure during JSON serialization or formatting. #[error("Serialization error: {0}")] Serialization(#[from] serde_json::Error), /// Missing mandatory field required by GDPR Article 30. #[error("M…` | Error type returned by operations in `contextra-audit-export`. | `src/error.rs` |

### Öffentliche Funktionen (14)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn bsi_mapping_table() -> Vec<BsiMappingEntry>` | Returns the static, code-maintained cryptographic mapping table for Contextra. Serves as the Single Source of Truth for audit exports and c… | `src/bsi_mapping.rs` |
| `pub fn render_bsi_mapping_markdown(entries: &[BsiMappingEntry]) -> String` | Renders the BSI Grundschutz / TR-02102 cryptographic mapping table as Markdown. Consistent with the rendering style in [`crate::markdown_te… | `src/bsi_mapping.rs` |
| `pub fn render_register_json( entries: &[ProcessingRegisterEntry], ) -> Result<String, AuditExportError>` | Renders a list of processing register entries as a formatted JSON string. Entries and internal collections are deterministically sorted pri… | `src/lib.rs` |
| `pub fn sort_internal_collections(&mut self)` | Sorts internal lists deterministically in place. | `src/lib.rs` |
| `pub fn validate(&self) -> Result<(), AuditExportError>` | Validates mandatory fields in controller contact details. | `src/lib.rs` |
| `pub fn validate(&self) -> Result<(), AuditExportError>` | Validates mandatory fields in third country transfer details. | `src/lib.rs` |
| `pub fn validate(&self) -> Result<(), AuditExportError>` | Validates mandatory fields in retention period rules. | `src/lib.rs` |
| `pub fn validate(&self) -> Result<(), AuditExportError>` | Validates all mandatory fields required by GDPR Article 30 Abs. 1. Returns [`AuditExportError::MissingField`] if any statutory field is mis… | `src/lib.rs` |
| `pub fn render_register_markdown( entries: &[ProcessingRegisterEntry], export_date: &str, ) -> Result<String, AuditExportError>` | Renders a list of [`ProcessingRegisterEntry`] records as a formatted Markdown document containing a GDPR Article 30 processing register tab… | `src/markdown_template.rs` |
| `pub fn sanitize_markdown_cell(input: &str) -> String` | Sanitizes input strings against Markdown injection and table breakage. Escapes ampersands (`&`), pipes (`|`), backticks (`` ` ``), newlines… | `src/markdown_template.rs` |
| `pub fn add_entry(&mut self, entry: ProcessingRegisterEntry)` | Appends a custom [`ProcessingRegisterEntry`] record. | `src/testkit.rs` |
| `pub fn new() -> Self` | Creates a new empty [`InMemoryProcessingRegisterSource`]. | `src/testkit.rs` |
| `pub fn sample_controller_contact() -> ControllerContact` | Creates a default [`ControllerContact`] instance for sample data. | `src/testkit.rs` |
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
pub mod assistance;
pub mod audit;
pub mod confidentiality;
pub mod data_subject_rights;
pub mod error;
pub mod instructions;
pub mod template;
pub mod termination;
pub use error::{AvvError, AvvGeneratorError};
```

### Öffentliche Datentypen — Structs (2)

| Struct | Felder / Form | Beschreibung | Datei |
|---|---|---|---|
| `AvvContext` | `{ /// Name oder Bezeichnung des Verantwortlichen (Auftraggebers, z. B. Kanzlei, Praxis, Unternehmen). pub controller_name: String, /// Name oder Bezeichnung des… }` | Kontext- und Konfigurationsparameter zur Generierung einer AVV-Vorlage gemäß Art. 28 DSGVO. Enthält alle variablen Vertragsbestandteile, wie Namen de… | `src/lib.rs` |
| `TechnicalMeasure` | `{ /// Bezeichnung der Sicherheitsmaßnahme (z. B. "Kryptografischer Löschbeweis (Ed25519)"). pub name: String, /// Detaillierte technische Funktionsbeschreibung d… }` | Beschreibung einer technischen und organisatorischen Maßnahme (TOM) gemäß Art. 32 DSGVO. TOMs beschreiben die Sicherheitsmechanismen, die zum Schutz… | `src/lib.rs` |

### Öffentliche Datentypen — Enums (1)

| Enum | Varianten | Beschreibung | Datei |
|---|---|---|---|
| `AvvError` | `/// Name des Verantwortlichen (Auftraggebers) ist leer oder enthält nur Whitespace. /// /// Gesetzliche Grundlage: Nach Art. 28 Abs. 3 DSGVO müssen die Vertragsparteien /// eindeutig identifiziert se…` | Benannte Fehlerzustände während der AVV-Vorlagenvalidierung oder des Markdown-Renderings. Alle Varianten implementieren [`thiserror::Error`] und lief… | `src/error.rs` |

### Typ-Aliase (1)

| Alias | Definition | Beschreibung |
|---|---|---|
| `AvvGeneratorError` | `type AvvGeneratorError= AvvError` | Typ-Alias für Rückwärtskompatibilität mit älteren Schnittstellen ([`AvvGeneratorError`]). |

### Öffentliche Funktionen (16)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn render_section(out: &mut String) -> Result<(), AvvError>` | Rendert Abschnitt 10: „Unterstützung bei Einhaltung der Pflichten gemäß Art. 32–36 DSGVO“. Schreibt die detaillierten Unterstützungspflicht… | `src/assistance.rs` |
| `pub fn render_section(out: &mut String) -> Result<(), AvvError>` | Rendert Abschnitt 11: „Nachweis- und Kontrollrechte“. Schreibt die Klauseln zu Auskunftsrechten und Vor-Ort- bzw. Fern-Audits in den Ausgab… | `src/audit.rs` |
| `pub fn render_section(out: &mut String) -> Result<(), AvvError>` | Rendert Abschnitt 5: „Vertraulichkeitsverpflichtung“. Schreibt den Textblock zur Wahrung des Datengeheimnisses in den Markdown-Ausgabepuffe… | `src/confidentiality.rs` |
| `pub fn render_section(out: &mut String) -> Result<(), AvvError>` | Rendert Abschnitt 9: „Unterstützung bei Betroffenenrechten“. Schreibt die Unterstützungszusagen für Rechte betroffener Personen nach Kapite… | `src/data_subject_rights.rs` |
| `pub fn render_section(out: &mut String) -> Result<(), AvvError>` | Rendert Abschnitt 4: „Pflichten und Rechte des Verantwortlichen & Weisungsbindung“. Schreibt den formatierten Markdown-Abschnitt in den übe… | `src/instructions.rs` |
| `pub fn default_technical_measures() -> Vec<TechnicalMeasure>` | Liefert die standardmäßig implementierten technischen Schutzmaßnahmen von Contextra zurück. Diese Liste spiegelt die Kern-Sicherheitsfeatur… | `src/lib.rs` |
| `pub fn new( controller_name: impl Into<String>, processor_name: impl Into<String>, tenant_id: TenantId, technical_measures: Vec<TechnicalMeasure>, subprocessors: Vec<String>, deletion_sla_days: u32, ) -> Self` | Erzeugt einen neuen [`AvvContext`] mit den Pflichtparametern und Standard-Optionen. # Parameter - `controller_name`: Name des Auftraggebers… | `src/lib.rs` |
| `pub fn render_avv_markdown(ctx: &AvvContext) -> Result<String, AvvError>` | Rendert das vollständige AVV-Dokument im Markdown-Format anhand des übergebenen [`AvvContext`]. # Ablauf 1. Validiert den Kontext über [`Av… | `src/lib.rs` |
| `pub fn render_avv_markdown_with_date(ctx: &AvvContext, date: &str) -> Result<String, AvvError>` | Rendert das AVV-Dokument im Markdown-Format mit einem explizit vorgegebenen Vertragsdatum. # Ablauf 1. Validiert den Kontext über [`AvvCont… | `src/lib.rs` |
| `pub fn validate(&self) -> Result<(), AvvError>` | Validiert alle Kontextfelder gegen rechtliche und logische Mindestanforderungen. # Prüfschritte 1. `controller_name`: Darf nach Trimmen nic… | `src/lib.rs` |
| `pub fn with_contract_date(mut self, contract_date: impl Into<String>) -> Self` | Setzt ein explizites Vertragsdatum für die AVV-Vorlage (Builder-Muster). | `src/lib.rs` |
| `pub fn with_subprocessors_allowed(mut self, allowed: bool) -> Self` | Konfiguriert die grundsätzliche Zulässigkeit von Unterauftragsverarbeitern (Builder-Muster). | `src/lib.rs` |
| `pub fn escape_markdown(input: &str) -> String` | Maskiert Markdown-Steuerzeichen in Benutzereingaben zur Verhinderung von Formatierungs-Injections. Ersetzt Zeichen mit spezieller Markdown-… | `src/template.rs` |
| `pub fn render(ctx: &AvvContext) -> Result<String, AvvError>` | Rendert das AVV-Markdown-Dokument basierend auf den Parametern in [`AvvContext`]. Falls kein explizites Vertragsdatum in `ctx.contract_date… | `src/template.rs` |
| `pub fn render_with_date(ctx: &AvvContext, date: &str) -> Result<String, AvvError>` | Rendert das AVV-Markdown-Dokument mit einer expliziten Datumsangabe. # Ablauf 1. Allokation eines Ergebnis-Puffers mit einer Startkapazität… | `src/template.rs` |
| `pub fn render_section( out: &mut String, ctx: &AvvContext, escaped_tenant_id: &str, ) -> Result<(), AvvError>` | Rendert Abschnitt 8: „Löschung von Daten und SLA“. Schreibt die Löschklausel unter Einbindung der Mandanten-ID ([`contextra_types::TenantId… | `src/termination.rs` |


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
pub use signed_gate::{ constant_time_eq_32, derive_local_installation_id_hash, verify_step_1_fast_ring, verify_step_2_presence, verify_step_3_installation, verify_step_4_signature, verify_step_5_expiration, verify_step_6_ring_permission, verify_step_7_success, LicensePayload, SignedActivation, SignedLicenseGate, SignedLicenseOptions, StepVerificationError, MAX_PAYLOAD_BYTES, };
```

### Öffentliche Datentypen — Structs (4)

| Struct | Felder / Form | Beschreibung | Datei |
|---|---|---|---|
| `LicensePayload` | `{ /// Mandantenidentität, an die diese Lizenz gebunden ist. pub tenant_id: TenantId, /// Liste aller durch diese Lizenz freigeschalteten Feature-Ringe. pub allow… }` | Strukturierter Lizenznutzlast-Datensatz (Payload). Enthält Mandantenbindung ([`TenantId`]), freigeschaltete Feature-Ringe, optionales Ablaufdatum, be… | `src/signed_gate.rs` |
| `SignedActivation` | `{ /// Der freizuschaltende Feature-Ring (z. B. `FeatureRing::Sovereign`). pub ring: FeatureRing, /// 32-Byte-BLAKE3-Hash der Ziel-Installationskennung. pub insta… }` | Signierter Aktivierungsdatensatz zur Bindung eines Feature-Rings an eine Maschinenidentität (§10.2, §15). Verbindet kryptografisch den freizuschalten… | `src/signed_gate.rs` |
| `SignedLicenseGate` | `{ verifying_key: Option<VerifyingKey>, activation: Option<SignedActivation>, license_payload: Option<LicensePayload>, local_installation_id: Option<[u8; 32]>, cl… }` | Kryptografisches Lizenz-Gate zur Durchsetzung von Feature-Ringen. Verifiziert Ed25519-Signaturen über Aktivierungsdatensätze ([`SignedActivation`]) o… | `src/signed_gate.rs` |
| `SignedLicenseOptions` | `{ /// Gibt an, ob der Fallback auf das Legacy-Schema (ohne Installations-Hash) erlaubt ist. pub allow_legacy: bool, /// Obergrenze für die Größe des serialisiert… }` | Konfigurationsoptionen für die Prüfung signierter Payloads und Rückwärtskompatibilität. Ermöglicht es, die maximale Nutzlastgröße anzupassen oder alt… | `src/signed_gate.rs` |

### Öffentliche Datentypen — Enums (1)

| Enum | Varianten | Beschreibung | Datei |
|---|---|---|---|
| `StepVerificationError` | `/// Schritt 2: Weder ein Aktivierungsdatensatz noch ein Lizenz-Payload ist vorhanden. #[error("activation or license payload missing for ring {0:?}")] MissingActivation(FeatureRing), /// Schritt 3: D…` | Detaillierte Fehlerursache während der 7-stufigen Lizenz- und Aktivierungsprüfung. **Zweck:** Ermöglicht intern eine präzise Diagnose, welcher Schrit… | `src/signed_gate.rs` |

### Konstanten (1)

| Konstante | Typ | Beschreibung |
|---|---|---|
| `MAX_PAYLOAD_BYTES` | `usize` | Maximale zulässige Payload-Größe in Bytes (64 KiB) für die Lizenzverifikation. **Zweck:** Schutz vor Speicher- und CPU-Erschöpfungsangriffen (Denial… |

### Öffentliche Funktionen (33)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn activation(&self) -> Option<&SignedActivation>` | Gibt eine Referenz auf den geprüften [`SignedActivation`]-Datensatz zurück, falls vorhanden. | `src/signed_gate.rs` |
| `pub fn check_step_1_fast_ring(&self, ring: FeatureRing) -> bool` | Führt Schritt 1 (Fast Ring Bypass) auf dieser Instanz aus. | `src/signed_gate.rs` |
| `pub fn check_step_2_presence(&self, ring: FeatureRing) -> Result<(), StepVerificationError>` | Führt Schritt 2 (Vorhandenseinsprüfung) auf dieser Instanz aus. | `src/signed_gate.rs` |
| `pub fn check_step_3_installation( &self, ring: FeatureRing, ) -> Result<(), StepVerificationError>` | Führt Schritt 3 (Installationsbindung) auf dieser Instanz aus. | `src/signed_gate.rs` |
| `pub fn check_step_4_signature(&self) -> Result<(), StepVerificationError>` | Führt Schritt 4 (Signaturprüfung) auf dieser Instanz aus. | `src/signed_gate.rs` |
| `pub fn check_step_5_expiration(&self) -> Result<(), StepVerificationError>` | Führt Schritt 5 (Ablaufprüfung via Zeit-Port) auf dieser Instanz aus. | `src/signed_gate.rs` |
| `pub fn check_step_6_ring_permission( &self, ring: FeatureRing, ) -> Result<(), StepVerificationError>` | Führt Schritt 6 (Ring-Berechtigung) auf dieser Instanz aus. | `src/signed_gate.rs` |
| `pub fn check_step_7_success(&self) -> Result<(), StepVerificationError>` | Führt Schritt 7 (Erfolgsbestätigung) auf dieser Instanz aus. | `src/signed_gate.rs` |
| `pub fn constant_time_eq_32(a: &[u8; 32], b: &[u8; 32]) -> bool` | Vergleicht zwei 32-Byte-Arrays in konstanter Laufzeit (Constant-Time). **Zweck:** Verhindert Seitenkanal-Angriffe (Timing Attacks) beim Prü… | `src/signed_gate.rs` |
| `pub fn create_signed( ring: FeatureRing, installation_id_hash: [u8; 32], expires_at_unix: i64, signing_key: &ed25519_dalek::SigningKey, ) -> Self` | Erstellt und signiert einen neuen Aktivierungsdatensatz mit dem privaten Ed25519-Schlüssel. **Ablauf:** 1. Berechnet die Bytefolge via [`Se… | `src/signed_gate.rs` |
| `pub fn create_test_signed_payload( rings: Vec<FeatureRing>, expires_at: Option<i64>, ) -> (Vec<u8>, [u8` | Erzeugt Testdaten (Payload-Bytes, Signatur und Verifying-Key-Bytes) für Unit- und Integrationstests. | `src/signed_gate.rs` |
| `pub fn derive_local_installation_id_hash(storage_path: Option<&std::path::Path>) -> [u8` | Ermittelt oder erzeugt deterministisch den 32-Byte-BLAKE3-Hash der lokalen Installation. # Status **VORLÄUFIG, BIS ZUR SCHRIFTLICHEN ENTSCH… | `src/signed_gate.rs` |
| `pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8` | Deserialisiert eine Bytefolge und prüft strikt auf exakt 64 Bytes Länge. | `src/signed_gate.rs` |
| `pub fn from_activation(activation: SignedActivation, verifying_key: VerifyingKey) -> Self` | Konstruiert ein Gate aus einem [`SignedActivation`]-Datensatz und dem öffentlichen Ed25519-Schlüssel. Nutzt standardmäßig die [`SystemClock… | `src/signed_gate.rs` |
| `pub fn from_activation_with_clock( activation: SignedActivation, verifying_key: VerifyingKey, clock: Arc<dyn Clock>, ) -> Self` | Konstruiert ein Gate aus einem [`SignedActivation`]-Datensatz mit injiziertem [`Clock`]-Port. | `src/signed_gate.rs` |
| `pub fn from_signed_payload( payload_bytes: &[u8], signature: &[u8; 64], verifying_key: VerifyingKey, ) -> Result<Self, LicenseError>` | Konstruiert ein Gate durch Signaturprüfung eines Roh-Payloads mit Standardeinstellungen. # Fehler Gibt [`LicenseError::InvalidSignature`] z… | `src/signed_gate.rs` |
| `pub fn from_signed_payload_with_clock( payload_bytes: &[u8], signature: &[u8; 64], verifying_key: VerifyingKey, clock: Arc<dyn Clock>, ) -> Result<Self, LicenseError>` | Konstruiert ein Gate aus signierten Payload-Bytes mit injiziertem [`Clock`]-Port. | `src/signed_gate.rs` |
| `pub fn from_signed_payload_with_clock_and_opts( payload_bytes: &[u8], signature: &[u8; 64], verifying_key: VerifyingKey, clock: Arc<dyn Clock>, options: SignedLicenseOptions, ) -> Result<Self, LicenseError>` | Konstruiert ein Gate aus signierten Payload-Bytes mit Optionen und Zeit-Port. **Ablauf:** 1. Prüft, ob `payload_bytes` die Obergrenze `opti… | `src/signed_gate.rs` |
| `pub fn into_license_error(self) -> LicenseError` | Überführt den internen Schrittfehler in den kanonischen [`LicenseError`]. **Garantie gegen Informationslecks:** Hardware-Hashes, Maschinend… | `src/signed_gate.rs` |
| `pub fn license_payload(&self) -> Option<&LicensePayload>` | Gibt eine Referenz auf das geprüfte [`LicensePayload`] zurück, falls vorhanden. | `src/signed_gate.rs` |
| `pub fn no_activation() -> Self` | Erzeugt ein leeres Gate ohne aktive Lizenz (nur der `Fast`-Ring ist freigeschaltet). | `src/signed_gate.rs` |
| `pub fn payload_bytes( ring: FeatureRing, installation_id_hash: [u8; 32], expires_at_unix: i64, ) -> Vec<u8>` | Berechnet die binäre Repräsentation der Aktivierungsfelder zur Signaturerstellung/-prüfung. **Ablauf:** Serialisiert das Tripel `(ring, ins… | `src/signed_gate.rs` |
| `pub fn serialize<S>(bytes: &[u8; 64], serializer: S) -> Result<S::Ok, S::Error> where S: Serializer,` | Serialisiert ein 64-Byte-Array als rohe Bytefolge. | `src/signed_gate.rs` |
| `pub fn verify_signature(&self, verifying_key: &VerifyingKey) -> bool` | Verifiziert die Ed25519-Signatur dieses Datensatzes anhand des öffentlichen Schlüssels. **Rückgabe:** `true` genau dann, wenn die Signatur… | `src/signed_gate.rs` |
| `pub fn verify_step_1_fast_ring(ring: FeatureRing) -> bool` | Schritt 1: Prüfung auf Fast-Ring-Bypass (Invariante INV-LICENSE-2). **Garantie:** Der freie Standard-Ring `FeatureRing::Fast` benötigt niem… | `src/signed_gate.rs` |
| `pub fn verify_step_2_presence( activation: Option<&SignedActivation>, payload: Option<&LicensePayload>, ring: FeatureRing, ) -> Result<(), StepVerificationError>` | Schritt 2: Vorhandenseinsprüfung von Aktivierungsdatensatz oder Lizenzpayload. **Vorbedingung:** Schritt 1 war `false` (Ring ist nicht `Fas… | `src/signed_gate.rs` |
| `pub fn verify_step_3_installation( activation: Option<&SignedActivation>, payload: Option<&LicensePayload>, local_installation_id: Option<&[u8; 32]>, ring: FeatureRing, ) -> Result<(), StepVerificationError>` | Schritt 3: Prüfung auf Übereinstimmung der lokalen Installations-ID. Vergleicht den Hash der aktuellen Maschinenumgebung in konstanter Zeit… | `src/signed_gate.rs` |
| `pub fn verify_step_4_signature( activation: Option<&SignedActivation>, payload: Option<&LicensePayload>, verifying_key: Option<&VerifyingKey>, ) -> Result<(), StepVerificationError>` | Schritt 4: Kryptografische Ed25519-Signaturprüfung. Prüft, ob der Aktivierungsdatensatz mit dem angegebenen öffentlichen Schlüssel (`verify… | `src/signed_gate.rs` |
| `pub fn verify_step_5_expiration( activation: Option<&SignedActivation>, payload: Option<&LicensePayload>, clock: &dyn Clock, ) -> Result<(), StepVerificationError>` | Schritt 5: Ablaufdatumsprüfung über den Zeit-Port ([`Clock`]). Liest die aktuelle Zeit über `clock.now_unix_nanos()` (Regel 3: kein direkte… | `src/signed_gate.rs` |
| `pub fn verify_step_6_ring_permission( activation: Option<&SignedActivation>, payload: Option<&LicensePayload>, requested_ring: FeatureRing, ) -> Result<(), StepVerificationError>` | Schritt 6: Prüfung der Ring-Berechtigung. Prüft, ob der konkret angeforderte Feature-Ring in der Lizenz enthalten ist. Es gibt keine automa… | `src/signed_gate.rs` |
| `pub fn verify_step_7_success() -> Result<(), StepVerificationError>` | Schritt 7: Bestätigung des erfolgreichen Verifikationsabschlusses. Dient als finaler Schlusspunkt der 7-stufigen Verifikationskette. | `src/signed_gate.rs` |
| `pub fn verifying_key(&self) -> Option<&VerifyingKey>` | Gibt eine Referenz auf den öffentlichen Ed25519-Schlüssel zurück, falls hinterlegt. | `src/signed_gate.rs` |
| `pub fn with_local_installation_id(mut self, id_hash: [u8; 32]) -> Self` | Bindet einen lokalen 32-Byte-Installations-Hash an diese Gate-Instanz. | `src/signed_gate.rs` |


---

## `contextra-store`

| Attribut | Wert |
|---|---|
| **Ring** | 1 — Ring 1 — Persistence & Cache |
| **Status** | Fertig |
| **Zweck** | LSM-Tree Storage Engine mit WAL-Group-Commit und HMAC-Integritätsprüfung. |
| **Unsafe-Code** | Nein (`#![forbid(unsafe_code)]`) |
| **Interne Abhängigkeiten** | `contextra-crypto`, `contextra-mvcc`, `contextra-ports`, `contextra-sys`, `contextra-types` |
| **Externe Abhängigkeiten (Auswahl)** | `ahash`, `blake3`, `bytes`, `crc32fast`, `lru`, `memmap2`, `parking_lot`, `quick_cache`, `rand`, `serde`, `thiserror`, `tokio`, `tokio-util`, `tracing`, `uuid` |

### Architekturbeschreibung

# contextra-store (Ring 1)  Die Crate `contextra-store` implementiert das persistente LSM-Tree-Speichertriebwerk (Log-Structured Merge-tree) fuer die Contextra-Plattform. Sie stellt die primaere Datenspeicherebene bereit und garantiert atomare Schreibvorgaenge, Konsistenz ueber Snapshots sowie Ausfallsicherheit durch Write-Ahead-Logging (WAL).  ## Architektur & Datenpfad - **Client / Engine:** Sendet atomare Schreiboperationen (Put, Delete) ueber Transaktionspuffer. - **WAL (Write-Ahead-Log):** Persistiert alle Mutationen sequentiell auf Festplatte inklusive CRC32- und HMAC-Integritaetspruefung, bevor sie im RAM bestaetigt werden. - **MemTable:** Haltebereich im Arbeitsspeicher (SkipList / Sharded BTreeMap), der schnelle Suchen und Einfuegungen bei geordneten Sequenznummern ermoeglicht. - **Flush & SSTable:** Bei Erreichen von Groessenschwellen wird die aktive MemTable eingefroren und asynchron als unveraenderliche SSTable (Sorted String Table) auf Platte geschrieben. - **Compaction:** Hintergrundprozess, der SSTables hierarchisch (Leveled / Tiered) zusammenfuehrt, Tombstones bereinigt und Lese- sowie Speicherverstaerkung minimiert.  ## Sicherheitsinvariante Die gesamte Crate erzwingt `#![forbid(unsafe_code)]` in Produktion. Es existieren keinerlei `unsafe`-Bloecke im Produktionscode.  ## Ring-Hierarchie & Abhaengigkeiten - **Ring 1:** Kernkomponente des Speichersystems. - **Eingehende Abhaengigkeiten:** Wird genutzt von `contextra-engine`, `contextra-db`, `contextra-cognition`, `contextra-graph`, `contextra-checkpoint`, `contextra-agent`, `contextra-py` und der uebergeordneten Facade `contextra`. - **Ausgehende Abhaengigkeiten:** Nutzt `contextra-types`, `contextra-ports`, `contextra-crypto`, `contextra-sys`, `contextra-mvcc`.

### Modul- und Re-Export-Oberfläche (lib.rs)

```rust
pub mod compaction;
pub mod engine;
pub mod fsync;
pub mod kv;
pub mod lsm;
pub mod manifest;
pub mod memtable;
pub mod scanner;
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
| `default` | `[]` |
| `test-hooks` | `[]` |
| `memory-only-storage` | `[]` |
| `fault-injection` | `[]` |
| `block-cache-v2` | `["dep:quick_cache"]` |
| `sieve-cache` | `[]` |
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

### Öffentliche Datentypen — Structs (43)

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
| `CompactionEngine` | `{ config: CompactionConfig, pub(super) snapshot_registry: Arc<SnapshotRegistry>, block_cache: Arc<BlockCache>, key_manager: Option<Arc<KeyManager>>, budget: Arc<… }` | Orchestriert die periodische Verdichtung und Bereinigung von SSTables. | `src/compaction/engine.rs` |
| `CostBasedAdaptivePlanner` | `{ read_ratio_threshold: f64, min_sstables_per_tier: usize, size_ratio: f64, }` | Reference implementation of cost-based adaptive compaction (EcoTune / ArceKV inspired). Adjusts compaction aggressiveness based on read-vs-write work… | `src/compaction/adaptive.rs` |
| `InMemoryStorageEngine` | `{ data: Arc<RwLock<StoreDataMap>>, staged: Arc<RwLock<StagedTxMap>>, seq: Arc<AtomicU64>, last_tx: Arc<RwLock<TxId>>, }` | An in-memory implementation of [`StorageEngine`] guaranteed to perform no disk I/O. | `src/engine/memory_only.rs` |
| `KvSegmentConfig` | `{ /// Delete mode specifying whether segments are tombstone-only or crypto-shredded. pub delete_mode: KvDeleteMode, }` | Konfiguration fuer segmentierten KV-Speicher. | `src/kv/segment.rs` |
| `KvSegmentManager` | `{ config: KvSegmentConfig, registry: Arc<KeyRegistry>, master_key: Option<Arc<KeyManager>>, }` | Manager for KV segment encryption, decryption, and crypto-shredding. | `src/kv/segment.rs` |
| `KvSegmentPayload` | `{ /// Group ID corresponding to the sub-key in `KeyRegistry`. pub group_id: u64, /// Physical payload stored on disk (ciphertext or raw plaintext). pub payload:… }` | Representation of a written KV segment (encrypted or raw payload with group_id and nonce). | `src/kv/segment.rs` |
| `LruBlockCacheBackend` | `{ state: RwLock<LruBlockCacheState>, }` | Standard strict-LRU block cache backend using `parking_lot::RwLock<LruCache>` with byte-based capacity. | `src/sstable/block_cache.rs` |
| `LsmConfig` | `{ /// Path to the data directory. pub path: PathBuf, /// Maximum size of the memtable before flushing to disk. pub memtable_size_limit: usize, /// Maximum RAM us… }` | Configuration for the LSM storage engine. | `src/lsm/config.rs` |
| `LsmPurger` | `{ storage: Arc<LsmStorage>, }` | Purger für LSM Storage Engine. | `src/scanner/purge.rs` |
| `LsmState` | `{ pub(super) memtable: Arc<MemTable>, pub(super) immutable_memtables: Vec<Arc<MemTable>>, }` | — | `src/lsm/guard.rs` |
| `LsmStorage` | `{ pub(super) config: LsmConfig, pub(super) key_manager: Option<Arc<KeyManager>>, pub(super) state: RwLock<LsmState>, /// SSTables stored separately for shared ac… }` | Primaere Schnittstelle fuer den LSM-Baum mit get, put, delete und scan. | `src/lsm/engine.rs` |
| `Manifest` | `{ pub(super) path: PathBuf, pub(super) file: tokio::sync::Mutex<tokio::fs::File>, pub(super) _dir_lock: Option<DirLock>, }` | Verwaltet die VersionSet-Manifestdatei zur dauerhaften Erfassung aller SSTables. | `src/manifest/core.rs` |
| `MemTable` | `{ /// Sharded storage: each shard holds a disjoint subset of keys. /// Shard selection is deterministic via `shard_for()`. shards: [MemTableShard; SHARD_COUNT],… }` | — | `src/memtable.rs` |
| `MemtableScanner` | `{ storage: Arc<LsmStorage>, }` | Scanner für die `LsmMemtable`-Schicht. | `src/scanner/memtable.rs` |
| `NoopReplayProgressSink` | `(tuple/unit)` | — | `src/wal/replay.rs` |
| `ObserverRegistry` | `{ observers: parking_lot::RwLock<Vec<Arc<BoundedObserverWorker>>>, historical_drops: Arc<AtomicU64>, max_observer_latency: parking_lot::RwLock<Duration>, clock:… }` | Configuration and handle for managing WAL observers with bounded delivery and circuit breaker. | `src/lsm/observer.rs` |
| `PreparedBatch` | `(tuple/unit)` | A batch of prepared WAL entries bound to a specific HMAC chain. `PreparedBatch` has a private inner field and cannot be constructed outside `wal.rs`.… | `src/wal/encode.rs` |
| `QuickCacheBlockCacheBackend` | `{ cache: quick_cache::sync::Cache<(u64, u64), Bytes, BlockWeighter>, }` | Lock-optimized block cache backend using `quick_cache::sync::Cache` (S3-FIFO / Clock-based eviction) with byte-based capacity. | `src/sstable/block_cache.rs` |
| `SieveCacheBackend` | `{ state: RwLock<SieveCacheState>, }` | Trait abstraction for block cache backend implementations. ## Approximative LRU & Lock-Optimized Read Path SSTable block caches store immutable 4KB d… | `src/sstable/block_cache.rs` |
| `SstableBuilder` | `{ path: PathBuf, file: File, block_builder: BlockBuilder, index: Vec<(Bytes, u64)>, // (last_key, offset) first_key: Option<Bytes>, last_key: Option<Bytes>, prev… }` | Erzeugt deterministisch geordnete SSTable-Dateien mit Datenbloecken und Index. | `src/sstable/builder.rs` |
| `SstableMetadata` | `{ pub first_key: Bytes, pub last_key: Bytes, pub file_size: u64, pub min_tx_id: u64, pub max_tx_id: u64, pub min_seq: u64, pub max_seq: u64, }` | Metadata for an SSTable. | `src/sstable/builder.rs` |
| `SstableReader` | `{ pub(super) file: Arc<std::fs::File>, pub(super) index: Vec<(Bytes, u64)>, pub(super) metadata: SstableMetadata, /// Byte offset where the index data begins (=… }` | Liest SSTable-Dateien mit Block-Cache und Bloom-Filter-Unterstuetzung. | `src/sstable/reader.rs` |
| `SstableScanner` | `{ storage: Arc<LsmStorage>, }` | Scanner für die `SsTableAllLevels`-Schicht. | `src/scanner/sstable.rs` |
| `SstableStream` | `{ pub(super) reader: Arc<SstableReader>, pub(super) block_idx: usize, pub(super) entry_idx: usize, pub(super) current_block: Option<Bytes>, pub(super) num_offset… }` | — | `src/sstable/stream.rs` |
| `SystemPressure` | `{ pub wal_queue_depth: usize, pub scheduler_queue_depth: f32, // 0.0–1.0 (Tokio global scheduler queue depth ratio) pub blocking_util: f32, // 0.0–1.0 (Blocking… }` | — | `src/system_pressure.rs` |
| `SystemPressureMonitor` | `{ pressure_tx: tokio::sync::watch::Sender<SystemPressure>, pub pressure_rx: tokio::sync::watch::Receiver<SystemPressure>, sampling_interval: Duration, }` | — | `src/system_pressure.rs` |
| `TenantKeyCodec` | `{ tenant_id: TenantId, /// Vorberechnetes Scan-Präfix ohne Allokation im Hot-Path. scan_prefix_cache: Vec<u8>, }` | Encodes LSM storage keys with tenant isolation prefixes. | `src/tenant_codec.rs` |
| `TenantScopedStorage` | `{ inner: S, codec: TenantKeyCodec, }` | Tenant-aware `StorageEngine` wrapper implementing strict key-level isolation (INV-TENANT-2). | `src/tenant_codec.rs` |
| `WalEntry` | `{ /// The operation performed. pub op: WalOp, /// Sequence number assigned to the operation. pub seq_no: u64, /// HMAC of the current entry (includes previous HM… }` | A single entry in the Write-Ahead Log. | `src/wal/encode.rs` |
| `WalEntryRef` | `{ pub entry: &'a WalEntry, }` | Lightweight view reference into a `WalEntry`. | `src/lsm/observer.rs` |
| `WalFlusherConfig` | `{ /// Maximum time to wait for additional messages after receiving the first, /// before issuing `sync_all()`. Set to 0 to disable (immediate flush). /// /// Typ… }` | Configuration for the WAL background flusher actor. Controls how aggressively the flusher coalesces concurrent writes into a single `sync_all()` call… | `src/wal/flusher.rs` |
| `WalScanner` | `{ storage: Arc<LsmStorage>, }` | Scanner für die `WalAllSegments`-Schicht. | `src/scanner/wal.rs` |
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

### Öffentliche Funktionen (241)

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
| `pub fn new( config: CompactionConfig, snapshot_registry: Arc<SnapshotRegistry>, block_cache: Arc<BlockCache>, key_manager: Option<Arc<KeyManager>>, budget: Arc<contextra_ports::ResourceTracker>, manifest: Option<Arc<crate::manifest::Manifest>>, ) -> Self` | Creates a new compaction engine. | `src/compaction/engine.rs` |
| `pub fn record_read_op(&self)` | Records a read operation for workload tracking. | `src/compaction/engine.rs` |
| `pub fn record_write_op(&self, seq_no: u64)` | Records a write operation for workload tracking. | `src/compaction/engine.rs` |
| `pub async fn run_loop( self: Arc<Self>, sstables: Arc<RwLock<Vec<Arc<SstableReader>>>>, data_path: PathBuf, shutdown: tokio_util::sync::CancellationToken, )` | Runs the background compaction loop. Periodically checks if compaction is needed and performs it. Designed to be spawned via `tokio::spawn`. | `src/compaction/engine.rs` |
| `pub fn select_compaction_candidates( &self, ssts: &[Arc<SstableReader>], ) -> Option<Vec<Arc<SstableReader>>>` | Selects SSTables to compact using Size-Tiered strategy. Groups by size class and returns the group that meets the threshold, prioritizing k… | `src/compaction/engine.rs` |
| `pub fn with_adaptive_planner(mut self, planner: Arc<dyn AdaptiveCompactionPlanner>) -> Self` | Attaches a custom adaptive compaction planner. | `src/compaction/engine.rs` |
| `pub fn with_clock(mut self, clock: Arc<dyn contextra_ports::Clock>) -> Self` | Attaches a custom clock port for deterministic execution timing (P28). | `src/compaction/engine.rs` |
| `pub fn with_merge_operator(mut self, merge_operator: Arc<dyn MergeOperator>) -> Self` | Attaches a custom merge operator for compaction value merging (§4.12). | `src/compaction/engine.rs` |
| `pub fn with_pressure_rx( mut self, rx: tokio::sync::watch::Receiver<crate::system_pressure::SystemPressure>, ) -> Self` | Attaches a system pressure watch receiver to enable pressure-aware compaction backpressure. | `src/compaction/engine.rs` |
| `pub fn retain_key_versions<T, F>(entries: Vec<T>, get_seq: F, floor_seq: u64) -> Vec<T> where F: Fn(&T) -> u64,` | Filters a sequence of entries for a SINGLE key (sorted by raw sequence DESCENDING) according to MVCC retention rules given a `floor` sequen… | `src/compaction/retention.rs` |
| `pub fn new() -> Self` | Creates a new, empty in-memory storage engine. | `src/engine/memory_only.rs` |
| `pub async fn fsync_parent_dir(path: &Path) -> Result<()>` | Alias for `sync_parent_dir` for backward compatibility across storage modules. | `src/fsync.rs` |
| `pub async fn sync_file(path: &Path) -> Result<()>` | Performs fsync on a single file at `path`. # Errors Returns `ContextraError::Storage` on Unix if opening or syncing fails. Returns `Context… | `src/fsync.rs` |
| `pub async fn sync_parent_dir(path: &Path) -> Result<()>` | Performs fsync on the parent directory of `path`. Directory fsync is required on POSIX filesystems to guarantee that newly created files or… | `src/fsync.rs` |
| `pub fn delete_mode(&self) -> KvDeleteMode` | Returns the current delete mode. | `src/kv/segment.rs` |
| `pub fn delete_segment(&self, group_id: u64) -> bool` | Deletes a KV segment. Under `CryptoShred`, revokes (destroys) the subkey for `group_id` in `KeyRegistry` in O(1) time. The physical ciphert… | `src/kv/segment.rs` |
| `pub fn generate_deletion_proof(&self, group_id: u64) -> Result<bool, ContextraError>` | Generates or validates a KV deletion proof. INVARIANTE `INV-KV-DELETE-1`: Löschbeweis auf Key-Value-Seite ist ausschließlich für `CryptoShr… | `src/kv/segment.rs` |
| `pub fn new( config: KvSegmentConfig, registry: Arc<KeyRegistry>, master_key: Option<Arc<KeyManager>>, ) -> Self` | Creates a new `KvSegmentManager` with specified config, key registry, and master key. | `src/kv/segment.rs` |
| `pub fn read_segment(&self, segment: &KvSegmentPayload) -> Result<Vec<u8>, ContextraError>` | Reads/decrypts a KV segment payload. Under `CryptoShred`, decrypts using the subkey in `KeyRegistry`. If the subkey was revoked (deleted),… | `src/kv/segment.rs` |
| `pub fn write_segment( &self, group_id: u64, plaintext: &[u8], ) -> Result<KvSegmentPayload, ContextraError>` | Writes a KV segment payload according to the configured `delete_mode`. Under `KvDeleteMode::CryptoShred`, encrypts `plaintext` using a subk… | `src/kv/segment.rs` |
| `pub fn cleanup_intent_locks_where<F>(&self, mut predicate: F) where F: FnMut(TxId) -> bool,` | Clears intent locks matching a predicate, gracefully recovering from poisoned locks. | `src/lsm/commit.rs` |
| `pub fn clear_intent_locks_above_tx(&self, target_tx: TxId)` | Removes all intent lock registrations associated with transactions strictly newer than target_tx. | `src/lsm/commit.rs` |
| `pub fn clear_intent_locks_for_tx(&self, tx_id: TxId)` | — | `src/lsm/commit.rs` |
| `pub fn validate_against_features( self, deletion_proof_active: bool, feature_ring: contextra_ports::license::FeatureRing, ) -> Result<(), DurabilityConfigError>` | — | `src/lsm/config.rs` |
| `pub fn block_cache_for_test(&self) -> Arc<BlockCache>` | — | `src/lsm/engine.rs` |
| `pub fn budget_for_test(&self) -> Arc<ResourceTracker>` | — | `src/lsm/engine.rs` |
| `pub fn budget_tracking_drift_bytes(&self) -> u64` | Returns the accumulated total memory budget tracking drift in bytes caused by unbudgeted memtable puts during commit when memory limit was… | `src/lsm/engine.rs` |
| `pub fn clock(&self) -> Arc<dyn contextra_ports::Clock>` | Returns the currently configured clock port. | `src/lsm/engine.rs` |
| `pub async fn close(&self) -> Result<()>` | Gracefully closes the storage engine, stopping background tasks and flushing active memtable to disk. | `src/lsm/engine.rs` |
| `pub fn data_path(&self) -> &std::path::Path` | Returns the data directory path. | `src/lsm/engine.rs` |
| `pub fn deregister_observer(&self, observer: &Arc<dyn WalObserver>)` | Deregisters an observer matching the provided `Arc` reference via pointer equality. | `src/lsm/engine.rs` |
| `pub async fn get_at_seq_tracked( &self, tx_id: TxId, key: &[u8], snapshot_seq: u64, ) -> Result<Option<Bytes>>` | Performs a tracked read for transaction `tx_id` at an explicit `snapshot_seq`. Registers the read key in `TxBuffer` at `snapshot_seq` and r… | `src/lsm/engine.rs` |
| `pub async fn get_tracked(&self, tx_id: TxId, key: &[u8]) -> Result<Option<Bytes>>` | Performs a tracked read for transaction `tx_id`. Evaluates `snapshot_seq = next_seq_no - 1` ONCE (K2), registers the read key in `TxBuffer`… | `src/lsm/engine.rs` |
| `pub fn has_observers(&self) -> bool` | Returns true if at least one observer is registered. | `src/lsm/engine.rs` |
| `pub async fn has_pending_legacy_wal_migration(&self) -> Result<bool>` | Checks if the current storage instance's data directory contains any pending legacy WAL files. Dies ist der Migrationsmechanismus vor der e… | `src/lsm/engine.rs` |
| `pub async fn has_pending_legacy_wal_migration_path(path: &std::path::Path) -> Result<bool>` | Checks if a database directory contains any legacy WAL files requiring explicit migration. Dies ist der Migrationsmechanismus vor der endgü… | `src/lsm/engine.rs` |
| `pub fn health(&self) -> StorageHealth` | Returns the current storage health status. | `src/lsm/engine.rs` |
| `pub fn last_applied_seq(&self) -> u64` | Returns the highest sequence number applied to the MemTable and published. | `src/lsm/engine.rs` |
| `pub fn manifest_for_test(&self) -> Arc<crate::manifest::Manifest>` | — | `src/lsm/engine.rs` |
| `pub fn max_observer_latency(&self) -> std::time::Duration` | Returns the current maximum latency threshold for observer callbacks. | `src/lsm/engine.rs` |
| `pub async fn maybe_compact_for_test(&self) -> Result<bool>` | — | `src/lsm/engine.rs` |
| `pub fn metrics_sink(&self) -> Arc<dyn contextra_ports::MetricsSink>` | Returns the currently configured [`MetricsSink`]. | `src/lsm/engine.rs` |
| `pub async fn migrate_legacy_wal_keys( config: &LsmConfig, clock: Arc<dyn contextra_ports::Clock>, ) -> Result<usize>` | Explicitly migrates all legacy WAL files in the specified directory using the provided clock port. Returns the number of migrated WAL files… | `src/lsm/engine.rs` |
| `pub fn next_seq_no_for_test(&self) -> u64` | Returns the current `next_seq_no` value for testing verification. | `src/lsm/engine.rs` |
| `pub async fn open(config: LsmConfig) -> Result<Self>` | Opens or recovers an LSM storage instance with the given configuration. | `src/lsm/engine.rs` |
| `pub async fn open_with_merge_operator( config: LsmConfig, merge_operator: Arc<dyn MergeOperator>, ) -> Result<Self>` | Opens or recovers an LSM storage instance with the given configuration and a `MergeOperator`. | `src/lsm/engine.rs` |
| `pub fn pressure_receiver( &self, ) -> tokio::sync::watch::Receiver<crate::system_pressure::SystemPressure>` | Returns a watch receiver for monitoring system pressure levels. | `src/lsm/engine.rs` |
| `pub fn register_observer(&self, observer: Arc<dyn WalObserver>)` | Registers a new WAL observer. | `src/lsm/engine.rs` |
| `pub fn request_flush(&self)` | Requests an asynchronous background flush without blocking the caller or holding commit locks. | `src/lsm/engine.rs` |
| `pub async fn restore_wal_file_handle_for_test(&self)` | — | `src/lsm/engine.rs` |
| `pub async fn scan_prefix_tracked( &self, tx_id: TxId, prefix: &[u8], ) -> Result<Vec<(Vec<u8>, Vec<u8>)>>` | Performs a tracked prefix scan for transaction `tx_id`. Evaluates `snapshot_seq = last_applied_seq` ONCE, scans all matching entries, and r… | `src/lsm/engine.rs` |
| `pub fn set_clock(&self, clock: Arc<dyn contextra_ports::Clock>)` | Sets the injected clock port for deterministic execution timing (P28). | `src/lsm/engine.rs` |
| `pub fn set_max_observer_latency(&self, latency: std::time::Duration)` | Sets the maximum latency threshold for observer callbacks. | `src/lsm/engine.rs` |
| `pub fn set_metrics_sink(&self, sink: Arc<dyn contextra_ports::MetricsSink>)` | Sets the [`MetricsSink`] for recording operational metrics. | `src/lsm/engine.rs` |
| `pub fn set_ssi_validator( &mut self, validator: Arc<dyn contextra_mvcc::SsiValidator + Send + Sync>, )` | Sets or replaces the [`SsiValidator`] port for transaction conflict detection. | `src/lsm/engine.rs` |
| `pub fn shutdown(&self)` | Signals the background compaction engine to stop. | `src/lsm/engine.rs` |
| `pub async fn simulate_wal_append_failure_for_test(&self)` | — | `src/lsm/engine.rs` |
| `pub async fn simulate_wal_append_failure_for_tx_for_test(&self, _tx_id: u64)` | — | `src/lsm/engine.rs` |
| `pub fn spawn_tracked<F>(&self, future: F) where F: std::future::Future<Output = ()> + Send + 'static,` | Spawns a background task tracked by this storage instance. | `src/lsm/engine.rs` |
| `pub fn sstables_for_test(&self) -> Arc<RwLock<Vec<Arc<SstableReader>>>>` | — | `src/lsm/engine.rs` |
| `pub async fn wait_shutdown(&self)` | Waits for all spawned tasks to shut down fully. | `src/lsm/engine.rs` |
| `pub fn with_clock(self, clock: Arc<dyn contextra_ports::Clock>) -> Self` | Configures the clock port fluently on `self` and returns `self`. | `src/lsm/engine.rs` |
| `pub fn with_metrics_sink(self, sink: Arc<dyn contextra_ports::MetricsSink>) -> Self` | Configures the [`MetricsSink`] fluently on `self` and returns `self`. | `src/lsm/engine.rs` |
| `pub fn with_ssi_validator( mut self, validator: Arc<dyn contextra_mvcc::SsiValidator + Send + Sync>, ) -> Self` | Configures the [`SsiValidator`] fluently on `self` and returns `self`. | `src/lsm/engine.rs` |
| `pub async fn force_flush(&self) -> Result<()>` | — | `src/lsm/flush.rs` |
| `pub async fn maybe_compact(&self) -> Result<bool>` | Evaluates whether compaction should run and performs it if needed. Uses the persistent CompactionEngine instance to maintain a stable count… | `src/lsm/flush.rs` |
| `pub fn checksum(&self) -> &'a [u8` | Returns a reference to the entry's checksum. | `src/lsm/observer.rs` |
| `pub fn clear_circuit_breaker(&self, observer: &Arc<dyn WalObserver>)` | Resets the circuit breaker state for a specific observer. | `src/lsm/observer.rs` |
| `pub fn deregister_observer(&self, observer: &Arc<dyn WalObserver>)` | Deregisters an observer matching the provided `Arc` reference via pointer equality. | `src/lsm/observer.rs` |
| `pub fn dropped_count(&self) -> u64` | Returns the total number of dropped events across all observers (including deregistered ones). | `src/lsm/observer.rs` |
| `pub fn dropped_count_for(&self, observer: &Arc<dyn WalObserver>) -> u64` | Returns the number of dropped events for a specific registered observer. | `src/lsm/observer.rs` |
| `pub fn is_any_circuit_breaker_open(&self) -> bool` | Returns whether any registered observer currently has an open circuit breaker. | `src/lsm/observer.rs` |
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
| `pub async fn point_lookup_metrics(&self, key: &[u8]) -> (usize, usize, usize, usize, bool)` | Evaluates detailed traversal metrics (evaluated_sstables, bloom_passes, range_passes, block_reads, found) for point lookups. | `src/lsm/scan.rs` |
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
| `pub fn put(&self, key: Bytes, value: Bytes, seq_no: u64, tx_id: u64)` | Inserts a key-value pair with a sequence number and transaction ID. ## Concurrent Sequence Invariant Sequence numbers pre-allocated to conc… | `src/memtable.rs` |
| `pub fn rollback(&self, tx_id: u64)` | Removes all entries associated with the specified transaction ID from the MemTable. | `src/memtable.rs` |
| `pub fn scan_prefix_into( &self, prefix: &[u8], seq_no: u64, max_tx: TxId, target: &mut BTreeMap<Bytes, (Bytes, u64)>, )` | Scans entries matching a key prefix into `target` according to MVCC visibility rules. Iterates over each shard's range starting at `prefix`… | `src/memtable.rs` |
| `pub fn scan_prefix_into_matching<F>( &self, prefix: &[u8], seq_no: u64, max_tx: TxId, target: &mut BTreeMap<Bytes, (Bytes, u64)>, predicate: F, ) where F: Fn(&[u8], u64, u64) -> bool,` | Scans entries matching a key prefix into `target` with an additional matching predicate. | `src/memtable.rs` |
| `pub fn scan_range_into( &self, start: Bound<&[u8]>, end: Bound<&[u8]>, seq_no: u64, max_tx: TxId, target: &mut BTreeMap<Bytes, (Bytes, u64)>, )` | Scans entries matching a key range into `target` according to MVCC visibility rules. Iterates over each shard's range defined by `(start, e… | `src/memtable.rs` |
| `pub fn scan_range_into_matching<F>( &self, start: Bound<&[u8]>, end: Bound<&[u8]>, seq_no: u64, max_tx: TxId, target: &mut BTreeMap<Bytes, (Bytes, u64)>, predicate: F, ) where F: Fn(&[u8], u64, u64) -> bool,` | Scans entries matching a key range into `target` with an additional matching predicate. | `src/memtable.rs` |
| `pub fn shard_entry_counts(&self) -> [usize` | Returns entry count for each shard (used for testing shard distribution). | `src/memtable.rs` |
| `pub fn shard_for(key: &[u8]) -> usize` | Deterministic shard selector mapping keys to independent shards. Constructs a 16-bit big-endian value from the first two bytes of the key t… | `src/memtable.rs` |
| `pub fn size(&self) -> usize` | Returns the approximate size in bytes. | `src/memtable.rs` |
| `pub fn tx_range(&self) -> (u64, u64)` | Returns the transaction ID range covered by this MemTable. | `src/memtable.rs` |
| `pub fn new(storage: Arc<LsmStorage>) -> Self` | Erzeugt einen neuen `MemtableScanner`. | `src/scanner/memtable.rs` |
| `pub fn new(storage: Arc<LsmStorage>) -> Self` | Erzeugt einen neuen `LsmPurger`. | `src/scanner/purge.rs` |
| `pub fn new(storage: Arc<LsmStorage>) -> Self` | Erzeugt einen neuen `SstableScanner`. | `src/scanner/sstable.rs` |
| `pub fn new(storage: Arc<LsmStorage>) -> Self` | Erzeugt einen neuen `WalScanner`. | `src/scanner/wal.rs` |
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
| `pub fn current_size(&self) -> usize` | — | `src/sstable/builder.rs` |
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
| `pub fn collection_prefix(&self, collection: &CollectionId) -> Vec<u8>` | Collection-spezifisches Scan-Präfix: `t:{tenant}:{col}:` | `src/tenant_codec.rs` |
| `pub fn decode_tenant_id(key: &[u8]) -> Option<TenantId>` | Dekodiert TenantId aus encoded Key. None bei ungültigem Format. Kein Unwrap — robuste Nutzung im Recovery-Pfad. | `src/tenant_codec.rs` |
| `pub fn encode_chunk_key(&self, collection: &CollectionId, doc_id: DocId) -> Vec<u8>` | Encodes a document chunk key: `t:{tenant}:{collection}:chunk:{doc}`. | `src/tenant_codec.rs` |
| `pub fn encode_graph_key(&self, collection: &CollectionId, entity_id: u64) -> Vec<u8>` | Encodes a graph entity key: `t:{tenant}:{collection}:graph:{entity}`. | `src/tenant_codec.rs` |
| `pub fn new(tenant_id: TenantId) -> Self` | Creates a new `TenantKeyCodec` for the given `TenantId`. | `src/tenant_codec.rs` |
| `pub fn new(inner: S, tenant_id: TenantId) -> Self` | Creates a new `TenantScopedStorage` wrapping an underlying `StorageEngine` or shared pointer. | `src/tenant_codec.rs` |
| `pub fn scan_prefix(&self) -> &[u8]` | Globales Scan-Präfix für diesen Tenant (alle Collections): `t:{tenant}:` Cached — keine Allokation im Hot-Path. | `src/tenant_codec.rs` |
| `pub fn tenant_id(&self) -> TenantId` | Returns the immutable `TenantId` bound to this storage wrapper. | `src/tenant_codec.rs` |
| `pub fn compute_checksum( op: &WalOp, seq_no: u64, integrity_key: &[u8], prev_hmac: [u8; 32], ) -> Result<[u8` | — | `src/wal/encode.rs` |
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
| `pub fn legacy_integrity_key_for_test() -> [u8` | — | `src/wal/hmac.rs` |
| `pub fn migrate_legacy_key(legacy: &[u8], new_key_material: &[u8]) -> Result<[u8` | Migrates a legacy integrity key to a new 32-byte key material state. # Chain Compatibility When migrating a legacy WAL segment to a new int… | `src/wal/hmac.rs` |
| `pub fn migrate_legacy_key(_legacy: &[u8], _new_key_material: &[u8]) -> Result<[u8` | — | `src/wal/hmac.rs` |
| `pub async fn prepare_batch(&self, ops: Vec<(WalOp, u64)>) -> Result<(PreparedBatch, [u8` | Prepares a batch of WAL operations with sequential sequence numbers and HMAC hash-chaining. | `src/wal/hmac.rs` |
| `pub async fn rollback_hmac(&self, prev_hmac: [u8; 32])` | Rolls back the in-memory HMAC state to `prev_hmac` asynchronously. | `src/wal/hmac.rs` |
| `pub fn rollback_hmac_sync(&self, prev_hmac: [u8; 32]) -> bool` | Synchronously rolls back the in-memory HMAC state to `prev_hmac` using non-blocking `try_lock`. Suitable for use within `Drop` implementati… | `src/wal/hmac.rs` |
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
| `pub async fn replay_mmap_with_sink<S: ReplayProgressSink>( &self, sink: &S, ) -> Result<(Vec<(u64, WalEntry, u64)>, WalVersion)>` | Replays the WAL using zero-copy memory mapping (`mmap`) with progress callback. | `src/wal/replay.rs` |
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
pub mod adapters;
pub mod attention_score;
pub mod eviction_worker;
pub mod prefix_store;
pub mod quantize_kivi;
pub mod radix;
pub mod scratchpad_invalidation;
pub mod segment;
pub mod service;
pub mod store;
pub use adapters::{ EmbeddingProviderSemanticEmbedder, KvCachePluginConfig, KvCachePluginManifest, StorageEngineKvBridge, };
pub use attention_score::{ rank_for_eviction, rank_for_eviction_weighted, AttentionScoreSource, NullAttentionScoreSource, };
pub use eviction_worker::{emergency_wipe, EvictionWorker};
pub use prefix_store::TenantPrefixKvStore;
pub use quantize_kivi::{ compress_bytes, decompress_bytes, kivi_dequantize, kivi_quantize, pack_kivi_block, unpack_kivi_block, KiviBlockMeta, KiviQuantizeConfig, KiviQuantizedBlock, KvTensorView, };
pub use radix::{ ContentAddressedKvStore, KvLookupResult, KvSegmentRef, SemanticCacheConfig, SemanticEmbedder, };
pub use radix::{KvBlockGuard, KvReusePolicy, PrefixMatch, PrefixRadixTree};
pub use scratchpad_invalidation::{ PrefixStoreScratchpadInvalidator, ScratchpadCacheScope, ScratchpadInvalidator, };
pub use segment::{ KvSegment, KvSegmentContent, ShreddableSegmentKey, Tier2EncryptedSegment, CURRENT_KV_KEY_DERIVATION_VERSION, };
pub use service::{KvCacheConfig, KvCacheService, KvCacheServiceBuilder, KvCacheStats};
pub use store::{SpillHandler, TenantIsolatedKvStore};
```

### Feature-Flags

| Flag | Aktiviert |
|---|---|
| `default` | `[]` |
| `content-addressed-kv-cache` | `[]` |
| `kv-encryption` | `["contextra-crypto/kv-encryption"]` |
| `kvcache-attention-eviction` | `[]` |
| `kvcache-kivi-quant` | `[]` |
| `kivi-quantization` | `[]` |

### Schnittstellen — Public Traits (2)

**`AttentionScoreSource`** — *Dyn-kompatible Trait-Schnittstelle zur Bereitstellung von Attention-Wichtigkeits-Scores. Wird von Upstream-Inferenz-Backends (z. B. Candle/Ollama) oder Test-Mocks implementiert, um Aufmerksamkeitsscores bereitzustellen, ohne direkte Abhängigkeiten zu Tensor-F…* (`src/attention_score.rs`)

```rust
pub trait AttentionScoreSource : Send + Sync {
/// Liefert den Wichtigkeits-Score für ein gegebenes `segment_id`.
    ///
    /// - Höhere Werte bedeuten höhere Relevanz (bleibt länger im Cache).
    /// - Niedrigere Werte bedeuten geringe Relevanz (wird bevorzugt verdrängt).
    /// - Liefert `None`, falls für dieses Segment kein Score bekannt ist.
    fn importance_score(&self, segment_id: u64) -> Option<f32>;

    /// Liefert den Wichtigkeits-Score für einen Mandanten und ein `segment_id`.
    ///
    /// Standardmäßig wird an [`Self::importance_score`] delegiert.
    fn importance_score_for_tenant(&self, _tenant_id: TenantId, segment_id: u64) -> Option<f32> {
        self.importance_score(segment_id)
    }

    /// Registriert eine Zuordnung von `(tenant_id, segment_id)` zu `request_id`.
    fn register_segment(&self, _tenant_id: TenantId, _segment_id: u64, _request_id: RequestId) {}

    /// Entfernt eine Zuordnung, wenn das Segment verdrängt oder freigegeben wurde.
    fn unregister_segment(&self, _tenant_id: TenantId, _segment_id: u64) {}
}
```

**`ScratchpadInvalidator`** — *Schnittstelle zur gezielten Invalidation von Scratchpad-Cache-Scopes.* (`src/scratchpad_invalidation.rs`)

```rust
pub trait ScratchpadInvalidator {
/// Invalidiert alle Gruppen/Blöcke im Radix-Baum, die zum angegebenen [`ScratchpadCacheScope`] gehören.
    ///
    /// # Fehler
    /// Gibt ein `Result<usize>` zurück, wobei `usize` die Anzahl invalidierter Segmente/Blöcke darstellt.
    fn invalidate_scope(&self, scope: &ScratchpadCacheScope) -> Result<usize>;
}
```

### Öffentliche Datentypen — Structs (25)

| Struct | Felder / Form | Beschreibung | Datei |
|---|---|---|---|
| `EmbeddingProviderSemanticEmbedder` | `{ provider: Arc<dyn EmbeddingProvider>, }` | Adaptiert einen generischen [`EmbeddingProvider`] für den `SemanticEmbedder`-Trait im KV-Cache. | `src/adapters.rs` |
| `EncryptedSegmentPayload` | `{ /// Verschlüsselter KV-Layer-Container aus `contextra-crypto`. Wird beim Drop gezeroized. pub layer: EncryptedKvLayer, }` | Verschlüsselte Layer-Repräsentation innerhalb eines `KvSegment`, wenn Feature `kv-encryption` aktiv ist. | `src/segment.rs` |
| `EvictionWorker` | `{ attention_source: Arc<RwLock<Arc<dyn AttentionScoreSource>>>, sender: parking_lot::Mutex<mpsc::Sender<EvictionCommand>>, handle: parking_lot::Mutex<Option<std:… }` | Asynchroner Hintergrund-Worker für die KV-Cache-Verdrängung. Läuft auf einem eigenständigen Betriebssystem-Thread, um die Tokio-Worker-Threads nicht… | `src/eviction_worker.rs` |
| `KiviBlockMeta` | `{ /// Anzahl Bits pro Skalarwert (für KIVI strikt 2). pub bits_per_value: u8, /// Skalierungsfaktor (Scale) für lineare Min/Max-Quantisierung. pub scale: f32, //… }` | Metadaten zur Dequantisierung eines 2-Bit KIVI-Blocks. | `src/quantize_kivi.rs` |
| `KiviQuantizeConfig` | `{ /// Gruppengröße für die per-channel Key-Quantisierung (Standard: 16). pub key_group_size: usize, /// Gibt an, ob Values ebenfalls quantisiert werden sollen (S… }` | Konfiguration für die KIVI-Quantisierungsparameter. | `src/quantize_kivi.rs` |
| `KiviQuantizedBlock` | `{ /// Dequantisierungs-Metadaten. pub meta: KiviBlockMeta, /// Gepackte 2-Bit-Skalarwerte (4 Werte pro Byte gepackt, LSB-first). pub packed: Vec<u8>, }` | 2-Bit-quantisierter Byte-Block mit zugehörigen Dequantisierungs-Metadaten. | `src/quantize_kivi.rs` |
| `KvBlockGuard` | `{ /// Eindeutige ID des geschützten Blocks. pub block_id: u64, /// Mandanten-ID des Eigentümers. pub tenant_id: TenantId, /// Atomarer Referenzzähler, geteilt mi… }` | RAII-Guard für ausgeliehene/gematchte KV-Blöcke. Verhindert, dass ein Block während einer laufenden Modell-Inferenz vom `EvictionWorker` verdrängt wi… | `src/radix.rs` |
| `KvCacheConfig` | `{ /// Maximales RAM-Speicherbudget in Bytes (Standard: 512 MiB). pub memory_budget_bytes: usize, /// Präfix-Wiederverwendungs-Policy (Standard: `CostBased { min_… }` | Konfiguration für die Fassade [`KvCacheService`]. | `src/service.rs` |
| `KvCachePluginConfig` | `{ /// Plugin Versions-Tupel (Major, Minor, Patch). pub version: (u32, u32, u32), /// Maximales Speicherlimit im RAM in Bytes für KV-Cache-Allokationen. pub max_m… }` | Validierte Konfiguration für das KV-Cache-Plugin-Manifest. | `src/adapters.rs` |
| `KvCachePluginManifest` | `{ config: KvCachePluginConfig, }` | Konkrete, validierte [`PluginManifest`]-Implementierung für `contextra-kvcache`. | `src/adapters.rs` |
| `KvCacheService` | `{ config: KvCacheConfig, store: Arc<TenantIsolatedKvStore>, eviction_worker: Arc<EvictionWorker>, clock: Arc<dyn Clock>, spill_storage: Option<Arc<dyn KvBridgeSt… }` | Öffentlicher mandantenisolierter KV-Cache-Dienst. Zentrale Facade für Inferenz-Laufzeiten: verbindet Radix-Präfixsuche, Speicher-LRU, Hintergrundverd… | `src/service.rs` |
| `KvCacheServiceBuilder` | `{ config: KvCacheConfig, clock: Option<Arc<dyn Clock>>, spill_storage: Option<Arc<dyn KvBridgeStorage>>, cipher: Option<Arc<KvSegmentCipher>>, attention_source:… }` | Builder zur Erstellung von [`KvCacheService`]. | `src/service.rs` |
| `KvCacheStats` | `{ /// Gesamtanzahl aktiver Segmente im RAM für diesen Mandanten. pub segment_count: usize, /// Gesamtanzahl von Einträgen im Prefix-Radix-Baum dieses Mandanten.… }` | Statistiken über die KV-Cache-Nutzung eines Mandanten. | `src/service.rs` |
| `KvSegment` | `{ #[zeroize(skip)] // Metadaten, keine sensiblen Tensor-Daten pub tenant_id: TenantId, #[zeroize(skip)] pub segment_id: u64, /// Versionsnummer der HKDF-Schlüsse… }` | Ein KV-Cache-Segment im Hauptspeicher. Erfüllt die P9-Sicherheitsanforderung: Zeroize-on-Drop für vertrauliche Daten. Darf niemals unverschlüsselt au… | `src/segment.rs` |
| `KvTensorView` | `{ /// Key-Tensor als Fließkommazahlen in `[num_tokens, num_channels]` Row-Major-Ordnung. pub keys: Vec<f32>, /// Value-Tensor als Fließkommazahlen in `[num_token… }` | Sicht (View) auf unquantisierte Roh-Tensoren für Keys und Values. | `src/quantize_kivi.rs` |
| `NullAttentionScoreSource` | `(tuple/unit)` | Standard-Implementierung, die keine Attention-Scores liefert (reines LRU-Verhalten). # Fallback-Verhalten Wenn kein Attention-Signal verfügbar ist, e… | `src/attention_score.rs` |
| `PrefixMatch` | `{ /// Übereinstimmende Token-Sequenz. pub matched_tokens: Vec<u32>, /// Anzahl der übereinstimmenden Tokens. pub matched_len: usize, /// Assoziierte KV-Block-ID.… }` | Ergebnis eines Präfix-Matches im Radix-Baum. | `src/radix.rs` |
| `PrefixRadixTree` | `{ tenant_id: TenantId, children: Vec<RadixNode>, total_entries: usize, }` | Komprimierter Prefix-Radix-Baum für Token-Sequenzen. Bietet schnelle Präfixsuche und automatische Knoten-Aufspaltung (Node Splitting), wenn sich Sequ… | `src/radix.rs` |
| `PrefixStoreScratchpadInvalidator` | `{ /// Referenz auf den unterliegenden [`TenantPrefixKvStore`]. pub store: &'a TenantPrefixKvStore, /// Liste von [`PrefixKey`]s, die im Store auf Scratchpad-Präf… }` | Implementierung von [`ScratchpadInvalidator`] für [`TenantPrefixKvStore`]. | `src/scratchpad_invalidation.rs` |
| `ScratchpadCacheScope` | `{ /// Eindeutige Mandanten-ID (TenantId) des ausführenden Agenten. pub tenant_id: TenantId, /// Token-Präfix, das den editierbaren Scratchpad-Bereich im Radix-Ba… }` | Definiert den Bereich (Scope) eines Agenten-Scratchpads für die Cache-Invalidierung. | `src/scratchpad_invalidation.rs` |
| `ShreddableSegmentKey` | `{ /// Interner kryptographischer Schlüsselmanager in einem Thread-sicheren RwLock. key_manager: Arc<RwLock<Option<CryptoKey>>>, }` | Shred-fähiger Schlüssel für Tier-2 Segmentdateien (Crypto-Shredding). Durch Aufruf von `shred()` wird der Schlüssel im Speicher mit Nullen überschrie… | `src/segment.rs` |
| `StorageEngineKvBridge` | `{ engine: Arc<dyn StorageEngine>, }` | Produktiver Tier-2 Spill-over Adapter über das generische [`StorageEngine`]-Interface. Bietet mandantengetrennte Persistierung von KV-Segmenten im St… | `src/adapters.rs` |
| `TenantIsolatedKvStore` | `{ shards: Box<[Shard]>, shard_count: usize, global_shard_offset: AtomicUsize, eviction_round_offsets: Box<[AtomicUsize]>, segment_capacity: NonZeroUsize, spill_h… }` | Mandantenisolierter KV-Segment-Store im Hauptspeicher. Bildet den zentralen RAM-Zwischenspeicher für KV-Caches mit Sharding, LRU-Verdrängung, Aufmerk… | `src/store.rs` |
| `TenantPrefixKvStore` | `{ partitions: RwLock<AHashMap<(TenantId, PrefixKey), Partition>>, byte_budget_per_tenant: usize, reuse_policy: KvReusePolicy, next_tick: AtomicU64, next_group_id… }` | Mandantenisolierter KV-Prefix-Store. Bietet strukturelle Isolation via `(TenantId, PrefixKey)`-Key in den Partitionen. Unterstützt konfigurierbares B… | `src/prefix_store.rs` |
| `Tier2EncryptedSegment` | `{ /// Mandanten-Identifikator zur Durchsetzung strenger Mandantentrennung. pub tenant_id: TenantId, /// Eindeutige Segment-ID innerhalb des Mandanten. pub segmen… }` | Verschlüsselte Tier-2 Segmentdatei (ausgelagerter KV-Block auf persistentem Speicher). Hält den Ciphertext und die Nonce zusammen mit dem zugehörigen… | `src/segment.rs` |

### Öffentliche Datentypen — Enums (2)

| Enum | Varianten | Beschreibung | Datei |
|---|---|---|---|
| `KvReusePolicy` | `/// Wiederverwendung aller Treffer mit `matched_len > 0`. Always, /// Kostenbasiert: Treffer erst ab Mindestpräfixlänge `min_prefix_len`. CostBased { min_prefix_len: usize }, /// Keinerlei Prefix-Reu…` | Policy zur Steuerung von Prefix-Reuse (§9.2). Definiert, ab wann ein gefundener Präfix-Treffer lohnend genug ist, um einen Cache-Treffer zu melden (d… | `src/radix.rs` |
| `KvSegmentContent` | `/// Rohe, unkomprimierte Tensor-Bytes im RAM. Raw(bytes::Bytes), /// 2-Bit-quantisierter KIVI-Block (spart bis zu 75% Speicher). #[cfg(any(feature = "kivi-quantization", feature = "kvcache-kivi-quant…` | Repräsentation des Inhalts eines KV-Segments. Kann entweder als rohe Bytes vorliegen oder in einem KIVI-quantisierten Format. | `src/segment.rs` |

### Typ-Aliase (1)

| Alias | Definition | Beschreibung |
|---|---|---|
| `SpillHandler` | `type SpillHandler= Arc<dyn Fn(TenantId, u64, Vec<u8>) + Send + Sync>` | Optionaler Callback-Hook für Tier-2-LSM-Spill bei Eviction aus dem In-Memory LRU Cache. Wird aufgerufen, wenn ein Segment aus dem RAM verdrängt wird,… |

### Konstanten (5)

| Konstante | Typ | Beschreibung |
|---|---|---|
| `CURRENT_KV_KEY_DERIVATION_VERSION` | `u8` | Aktuelle Version der KV-Segment-Schlüsselableitung (HKDF = HMAC-based Extract-and-Expand Key Derivation Function). Erhöhe diesen Wert, wenn sich der… |
| `DEFAULT_BYTE_BUDGET_PER_TENANT` | `usize` | Standard-Byte-Budget pro Mandant (256 MiB). |
| `DEFAULT_SEGMENT_CAPACITY_PER_TENANT` | `usize` | Standardkapazität an Segmenten pro Mandant (256). |
| `DEFAULT_SHARD_COUNT` | `usize` | Standardanzahl paralleler Shards (32). |
| `MAX_PINNED_BYTES_PER_TENANT` | `u64` | Maximal erlaubte Gesamtgröße gepinnter Bytes pro Mandant für ungepinnte Segmente ohne Ablaufdatum (`Pin { ttl: None }`). Verhindert Denial-of-Service… |

### Öffentliche Funktionen (113)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn new(engine: Arc<dyn StorageEngine>) -> Self` | Erstellt eine neue `StorageEngineKvBridge` um eine gegebene [`StorageEngine`]. | `src/adapters.rs` |
| `pub fn new(provider: Arc<dyn EmbeddingProvider>) -> Self` | Erstellt einen neuen `EmbeddingProviderSemanticEmbedder`. | `src/adapters.rs` |
| `pub fn new(config: KvCachePluginConfig) -> ContextraResult<Self>` | Erstellt ein neues, validiertes `KvCachePluginManifest`. | `src/adapters.rs` |
| `pub fn provider(&self) -> &Arc<dyn EmbeddingProvider>` | Liefert eine Referenz auf den zugrundeliegenden [`EmbeddingProvider`]. | `src/adapters.rs` |
| `pub fn validate(&self) -> ContextraResult<()>` | Validiert die Konfigurationsparameter (mindestens 1 MiB RAM erforderlich). | `src/adapters.rs` |
| `pub fn rank_for_eviction( candidates: &[(u64, Instant)], scores: &dyn AttentionScoreSource, ) -> Vec<u64>` | Bewertet Verdrängungskandidaten mit Standard-Gewichtung (50% LRU-Alter, 50% Attention-Score). Gibt eine sortierte Liste von Segment-IDs zur… | `src/attention_score.rs` |
| `pub fn rank_for_eviction_weighted( candidates: &[(u64, Instant)], scores: &dyn AttentionScoreSource, attention_weight: f32, ) -> Vec<u64>` | Bewertet Verdrängungskandidaten unter Kombination von normalisiertem LRU-Alter und Attention-Score. # Parameter - `candidates`: Liste aus T… | `src/attention_score.rs` |
| `pub fn rank_for_eviction_weighted_for_tenant( tenant_id: TenantId, candidates: &[(u64, Instant)], scores: &dyn AttentionScoreSource, attention_weight: f32, ) -> Vec<u64>` | Bewertet Verdrängungskandidaten mandantenspezifisch. # Ablauf 1. Schnellausstieg bei 0 oder 1 Kandidaten. 2. Zeitstempel normalisieren: Zei… | `src/attention_score.rs` |
| `pub fn attention_source(&self) -> Arc<dyn AttentionScoreSource>` | Liefert die aktuell konfigurierte [`AttentionScoreSource`]. | `src/eviction_worker.rs` |
| `pub fn emergency_wipe(store: &TenantIsolatedKvStore)` | Synchroner Notfall-Wipe für sofortige Speicherlöschung. Löscht blockierend alle Segmente im Store synchron aus dem RAM und stellt sicher, d… | `src/eviction_worker.rs` |
| `pub fn notify_block_released(&self, block_id: u64)` | Benachrichtigt den EvictionWorker, dass eine Block-Referenz freigegeben wurde. | `src/eviction_worker.rs` |
| `pub fn register_segment_request( &self, tenant_id: TenantId, segment_id: u64, request_id: RequestId, )` | Registriert eine Zuordnung von `(tenant_id, segment_id)` zu einer `request_id`. | `src/eviction_worker.rs` |
| `pub fn shutdown(&self)` | Beendet den Worker-Thread geordnet und wartet auf dessen Abschluss (Join). | `src/eviction_worker.rs` |
| `pub fn spawn(store: Arc<TenantIsolatedKvStore>) -> Self` | Spawnt den Eviction-Worker auf einem dedizierten OS-Thread. # Ablauf 1. Initialisiert Standard-[`NullAttentionScoreSource`]. 2. Erzeugt ein… | `src/eviction_worker.rs` |
| `pub fn trigger_eviction(&self, target_free_bytes: usize)` | Löst nicht-blockierend eine Verdrängung im Hintergrund aus. Sendet den Auftrag über den MPSC-Kanal an den Worker-Thread und kehrt sofort zu… | `src/eviction_worker.rs` |
| `pub fn unregister_segment_request(&self, tenant_id: TenantId, segment_id: u64)` | Entfernt die Zuordnung für `(tenant_id, segment_id)`. | `src/eviction_worker.rs` |
| `pub fn with_attention_exporter(self, exporter: Arc<dyn AttentionExporter>) -> Self` | Builder-Methode zur Injektion eines echten Attention-Backends via [`AttentionExporter`]. | `src/eviction_worker.rs` |
| `pub fn evict(&self, tenant: TenantId, key: &PrefixKey) -> Result<u64, ContextraError>` | Evictiert KV-Präfix-Einträge für den angegebenen Mandanten und PrefixKey. Gibt die Anzahl der entfernten Blöcke zurück. | `src/prefix_store.rs` |
| `pub fn insert( &self, tenant: TenantId, key: &PrefixKey, tokens: &[u32], blocks: Vec<KvBlock>, ) -> Result<(), ContextraError>` | Fügt KV-Blöcke für ein Token-Präfix in den Store ein. # Ablauf 1. Eingabe prüfen (weder Tokens noch Blöcke dürfen leer sein). 2. Prüfen, ob… | `src/prefix_store.rs` |
| `pub fn invalidate_prefix_scope_keys( &self, tenant: TenantId, scratchpad_key_prefix: &[u32], keys: &[PrefixKey], ) -> Result<usize, ContextraError>` | Gezielte Invalidation eines Scratchpad-Scopes für den angegebenen Mandanten über alle oder spezifische PrefixKeys. Entfernt ausschließlich… | `src/prefix_store.rs` |
| `pub fn lookup(&self, tenant: TenantId, key: &PrefixKey, tokens: &[u32]) -> Option<KvPrefixHit>` | Sucht nach dem längsten übereinstimmenden Präfix in Blöcken für die angegebenen Tokens. # Parameter - `tenant`: Eigentümer-Mandant. - `key`… | `src/prefix_store.rs` |
| `pub fn new() -> Self` | Erstellt einen neuen `TenantPrefixKvStore` mit Standard-Konfiguration (256 MiB Budget). | `src/prefix_store.rs` |
| `pub fn with_byte_budget_per_tenant(mut self, budget: usize) -> Self` | Konfiguriert das maximale Byte-Budget pro Mandant über alle seine Präfix-Partitionen. | `src/prefix_store.rs` |
| `pub fn with_reuse_policy(mut self, policy: KvReusePolicy) -> Self` | Konfiguriert die Wiederverwendungs-Policy ([`KvReusePolicy`]). | `src/prefix_store.rs` |
| `pub fn compress_bytes(input: &[u8]) -> Vec<u8>` | Komprimiert ein Byte-Slice verlustfrei für Tier-2 Auslagerungen (LSM-Spill). Nutzt ein einfaches Run-Length-Encoding (RLE) mit Header `0x43… | `src/quantize_kivi.rs` |
| `pub fn decompress_bytes(input: &[u8]) -> Result<Vec<u8>>` | Dekomprimiert ein zuvor mit [`compress_bytes`] komprimiertes Byte-Slice. # Fehler Liefert [`ContextraError::kv_quantization`], falls die Ei… | `src/quantize_kivi.rs` |
| `pub fn from_bytes(bytes: &[u8]) -> Result<Self>` | Deserialisiert eine Tensor-Sicht aus Bytes via Bincode. | `src/quantize_kivi.rs` |
| `pub fn kivi_dequantize(bytes: &[u8], meta: &KiviBlockMeta) -> Result<KvTensorView>` | Dequantisiert gepackte 2-Bit-Bytes zurück in einen [`KvTensorView`]. # Ablauf 1. Metadaten validieren (`bits_per_value == 2`, Skalen und Nu… | `src/quantize_kivi.rs` |
| `pub fn kivi_quantize(raw: &KvTensorView, config: KiviQuantizeConfig) -> Result<KiviQuantizedBlock>` | Quantisiert einen unkomprimierten [`KvTensorView`] in einen 2-Bit [`KiviQuantizedBlock`]. # Ablauf 1. Dimensionen prüfen und Gruppengröße f… | `src/quantize_kivi.rs` |
| `pub fn new( keys: Vec<f32>, values: Vec<f32>, num_tokens: usize, num_channels: usize, ) -> Result<Self>` | Erstellt einen neuen `KvTensorView` und prüft die Dimensionen gegen die Elementanzahl. # Fehler Liefert [`ContextraError::InvalidInput`], f… | `src/quantize_kivi.rs` |
| `pub fn pack_kivi_block(values: &[f32]) -> KiviQuantizedBlock` | Referenz-Implementierung für lineare 2-Bit Min/Max-Quantisierung eines Fließkomma-Slices. | `src/quantize_kivi.rs` |
| `pub fn to_bytes(&self) -> Result<Vec<u8>>` | Serialisiert die Tensor-Sicht in Bytes via Bincode. | `src/quantize_kivi.rs` |
| `pub fn unpack_kivi_block(block: &KiviQuantizedBlock) -> Result<Vec<f32>>` | Entpackt einen einzelnen linearen 2-Bit-Block zurück in ein `Vec<f32>`. | `src/quantize_kivi.rs` |
| `pub fn active_refs(&self) -> usize` | Gibt die aktuelle Anzahl aktiver Referenzen zurück. | `src/radix.rs` |
| `pub fn block_id(&self) -> u64` | Gibt die Block-ID zurück. | `src/radix.rs` |
| `pub fn clear(&mut self)` | Leert den gesamten Baum vollständig. | `src/radix.rs` |
| `pub fn find_longest_prefix( &self, tokens: &[u32], policy: KvReusePolicy, ) -> Option<PrefixMatch>` | Sucht das längste übereinstimmende Präfix unter Anwendung der [`KvReusePolicy`]. # Ablauf 1. Schnellausstieg bei leeren Tokens oder Policy… | `src/radix.rs` |
| `pub fn insert(&mut self, tokens: &[u32], block_id: u64) -> Result<(), ContextraError>` | Fügt eine Token-Sequenz mit assoziierter KV-Block-ID ein. # Ablauf 1. Prüfen, ob die Sequenz nicht leer ist (leere Sequenzen erzeugen [`Con… | `src/radix.rs` |
| `pub fn is_empty(&self) -> bool` | Prüft, ob der Baum leer ist. | `src/radix.rs` |
| `pub fn len(&self) -> usize` | Gibt die Anzahl gespeicherter Präfixe zurück. | `src/radix.rs` |
| `pub fn new( block_id: u64, tenant_id: TenantId, ref_counter: Arc<AtomicUsize>, worker: Option<Arc<EvictionWorker>>, ) -> Self` | Erstellt einen neuen Guard und inkrementiert den Referenzzähler atomar (`SeqCst`). | `src/radix.rs` |
| `pub fn new(tenant_id: TenantId) -> Self` | Erstellt einen neuen Prefix-Radix-Baum für den angegebenen Mandanten. | `src/radix.rs` |
| `pub fn remove(&mut self, tokens: &[u32]) -> Option<u64>` | Entfernt eine Token-Sequenz aus dem Baum und gibt die bisherige Block-ID zurück. | `src/radix.rs` |
| `pub fn should_reuse(&self, matched_len: usize) -> bool` | Prüft, ob ein Treffer mit der Länge `matched_len` wiederverwendet werden soll. # Parameter - `matched_len`: Anzahl übereinstimmender Tokens… | `src/radix.rs` |
| `pub fn tenant_id(&self) -> TenantId` | Gibt die Mandanten-ID zurück. | `src/radix.rs` |
| `pub fn tenant_id(&self) -> TenantId` | Gibt die Mandanten-ID des Radix-Baums zurück. | `src/radix.rs` |
| `pub fn new(tenant_id: TenantId, scratchpad_key_prefix: Vec<u32>) -> Self` | Erstellt einen neuen [`ScratchpadCacheScope`] für den angegebenen Mandanten und das Token-Präfix. | `src/scratchpad_invalidation.rs` |
| `pub fn new(store: &'a TenantPrefixKvStore) -> Self` | Erstellt einen neuen [`PrefixStoreScratchpadInvalidator`] für den angegebenen Store. | `src/scratchpad_invalidation.rs` |
| `pub fn with_keys(mut self, keys: Vec<PrefixKey>) -> Self` | Konfiguriert die zu prüfenden [`PrefixKey`]s für die Invalidation. | `src/scratchpad_invalidation.rs` |
| `pub fn acquire_guard(&self, worker: Option<Arc<EvictionWorker>>) -> KvBlockGuard` | Erstellt einen [`KvBlockGuard`] für diesen Block und inkrementiert den Referenzzähler atomar. | `src/segment.rs` |
| `pub fn active_refs(&self) -> usize` | Gibt die aktuelle Anzahl aktiver Referenzen (`KvBlockGuard`) zurück. | `src/segment.rs` |
| `pub fn as_bytes(&self) -> &[u8]` | Read-Only-Zugriff auf die Rohdaten. Kein unverschlüsselter Export nach außen ohne explizite Autorisierung. | `src/segment.rs` |
| `pub fn decrypt(&self, ciphertext: &[u8], nonce: &[u8; 12]) -> Result<Vec<u8>, ContextraError>` | Entschlüsselt Daten mit AES-256-GCM-SIV unter Verwendung der 12-Byte-Nonce. # Parameter - `ciphertext`: Verschlüsselte Daten. - `nonce`: 12… | `src/segment.rs` |
| `pub fn decrypt_data(&self, cipher: &KvSegmentCipher) -> Result<Vec<u8>, CryptoError>` | Entschlüsselt die Daten des Segments, falls es verschlüsselt ist. | `src/segment.rs` |
| `pub fn encrypt(&self, plaintext: &[u8]) -> Result<(Vec<u8>, [u8` | Verschlüsselt Daten mit AES-256-GCM-SIV und automatisch generiertem Nonce (Number used once). # Ablauf 1. Lesesperre auf `key_manager` anfo… | `src/segment.rs` |
| `pub fn is_empty(&self) -> bool` | Prüft, ob das Segment leer ist (0 Bytes). | `src/segment.rs` |
| `pub fn is_shredded(&self) -> bool` | Prüft, ob der Schlüssel bereits ge-shredded (vernichtet) wurde. | `src/segment.rs` |
| `pub fn len(&self) -> usize` | Länge der Tensor-Bytes in Bytes. | `src/segment.rs` |
| `pub fn new( tenant_id: TenantId, segment_id: u64, plaintext: &[u8], key: ShreddableSegmentKey, ) -> Result<Self, ContextraError>` | Verschlüsselt Klartext-Tensor-Bytes und erstellt ein neues Tier-2 Segment. # Ablauf 1. Klartext mit `key.encrypt()` verschlüsseln. 2. Struk… | `src/segment.rs` |
| `pub fn new(tenant_id: TenantId, segment_id: u64, data: Vec<u8>) -> Self` | Erstellt ein neues Klartext-KV-Cache-Segment (Default / Zero-Config, P12-konform). # Parameter - `tenant_id`: Zugehöriger Mandant. - `segme… | `src/segment.rs` |
| `pub fn new_encrypted( cipher: &KvSegmentCipher, tenant_id: TenantId, segment_id: u64, model_fingerprint: ModelFingerprint, rope_offset: Option<usize>, plaintext: &[u8], ) -> Result<Self, CryptoError>` | Erstellt ein verschlüsseltes KV-Cache-Segment aus einem Klartext-Tensor. | `src/segment.rs` |
| `pub fn new_with_metadata( tenant_id: TenantId, segment_id: u64, data: Vec<u8>, #[cfg(feature = "kv-encryption")] model_fingerprint: Option<ModelFingerprint>, rope_offset: Option<usize>, ) -> Self` | Erstellt ein neues Klartext-KV-Cache-Segment mit optionalen Metadaten. Ermöglicht die Übergabe eines RoPE-Offsets und eines Modell-Fingerpr… | `src/segment.rs` |
| `pub fn read_and_decrypt(&self) -> Result<Vec<u8>, ContextraError>` | Liest und entschlüsselt das Segment. Scheitert sofort, wenn der Schlüssel ge-shredded wurde. | `src/segment.rs` |
| `pub fn read_dequantized( &self, cipher: &dyn contextra_crypto::KvCipher, ) -> Result<KvTensorView, ContextraError>` | Liest und dequantisiert Segmentdaten unter Beachtung von `INV-KIVI-AEAD-ORDER`. # Ablauf 1. **AEAD-Entschlüsselung ZUERST**: Öffnen und Ver… | `src/segment.rs` |
| `pub fn shred(&self)` | Crypto-Shredding: Vernichtet den Schlüssel unwiderruflich im Speicher. # Ablauf 1. Schreibsperre anfordern. 2. `Option::take()` entnimmt di… | `src/segment.rs` |
| `pub fn to_spill_bytes(&self) -> Vec<u8>` | Serialisiert das Segment bzw. dessen verschlüsselte Layer für Tier-2-LSM-Spill (Auslagerung). | `src/segment.rs` |
| `pub fn try_new(passphrase: &str, salt: &[u8]) -> Result<Self, ContextraError>` | Erstellt einen neuen shred-fähigen Schlüssel aus Passphrase und Salt. # Parameter - `passphrase`: Geheime Passphrase des Mandanten. - `salt… | `src/segment.rs` |
| `pub fn try_new_random(passphrase: &str) -> Result<Self, ContextraError>` | Erstellt einen neuen shred-fähigen Schlüssel mit zufällig erzeugtem Salt. # Parameter - `passphrase`: Geheime Passphrase des Mandanten. # F… | `src/segment.rs` |
| `pub fn write_quantized( &mut self, raw_kv: &KvTensorView, config: KiviQuantizeConfig, cipher: &dyn contextra_crypto::KvCipher, ) -> Result<(), ContextraError>` | Schreibt quantisierte KV-Tensor-Daten in das Segment unter Beachtung von `INV-KIVI-AEAD-ORDER`. # Ablauf 1. **Quantisierung ZUERST**: Reduk… | `src/segment.rs` |
| `pub fn attention_exporter(mut self, exporter: Arc<dyn AttentionExporter>) -> Self` | Konfiguriert einen [`AttentionExporter`] für den [`EvictionWorker`]. | `src/service.rs` |
| `pub fn attention_source(mut self, source: Arc<dyn AttentionScoreSource>) -> Self` | Konfiguriert eine benutzerdefinierte [`AttentionScoreSource`]. | `src/service.rs` |
| `pub fn build(self) -> KvCacheService` | Baut die [`KvCacheService`]-Instanz. # Ablauf 1. Clock und Store initialisieren. 2. EvictionWorker spawnen und Attention-Quellen injizieren… | `src/service.rs` |
| `pub fn cipher(mut self, cipher: Arc<KvSegmentCipher>) -> Self` | Konfiguriert den Segment-Cipher für verschlüsselten Tier-2-Spill. | `src/service.rs` |
| `pub fn clock(mut self, clock: Arc<dyn Clock>) -> Self` | Injiziert die Clock-Port-Implementierung zur Einhaltung deterministischer Zeitvorgaben (P28 / Regel 3). | `src/service.rs` |
| `pub fn clock(&self) -> &Arc<dyn Clock>` | Liefert eine Referenz auf den injizierten Clock-Port. | `src/service.rs` |
| `pub fn delete_tenant(&self, tenant_id: TenantId)` | Entfernt alle RAM-Segmente und Radix-Knoten für einen Mandanten (DSGVO Art. 17 Mandantenlöschung). # Ablauf 1. Alle Segment-IDs des Mandant… | `src/service.rs` |
| `pub fn enable_kivi_quantization(mut self, config: Option<KiviQuantizeConfig>) -> Self` | Aktiviert oder deaktiviert die KIVI 2-Bit Quantisierung. | `src/service.rs` |
| `pub fn get_or_insert_prefix( &self, tenant_id: TenantId, tokens: &[u32], data: &[u8], ) -> Result<Option<(PrefixMatch, KvBlockGuard)>>` | Sucht nach einem übereinstimmenden Prefix oder fügt ein neues KV-Segment ein. Fail-closed: `tenant_id` ist Pflichtparameter jeder Operation… | `src/service.rs` |
| `pub fn invalidate_scratchpad( &self, tenant_id: TenantId, scope: ScratchpadCacheScope, ) -> Result<usize>` | Invalidiert Scratchpad-Tokens, die zum angegebenen Scope gehören. | `src/service.rs` |
| `pub fn memory_budget_bytes(mut self, bytes: usize) -> Self` | Konfiguriert das Speicherbudget in Bytes (mindestens 1 MiB). | `src/service.rs` |
| `pub fn new() -> Self` | Erstellt einen neuen Builder mit Standardkonfiguration. | `src/service.rs` |
| `pub async fn read_segment( &self, tenant_id: TenantId, segment_id: u64, ) -> Result<Option<Vec<u8>>>` | Liest ein Segment-Payload aus RAM oder Tier-2 Spill-Storage. # Ablauf 1. Versuch, das Segment direkt aus dem RAM-Store zu lesen und zu deko… | `src/service.rs` |
| `pub fn reuse_policy(mut self, policy: KvReusePolicy) -> Self` | Konfiguriert die Prefix-Wiederverwendungs-Policy ([`KvReusePolicy`]). | `src/service.rs` |
| `pub fn spill_enabled(mut self, enabled: bool) -> Self` | Aktiviert oder deaktiviert den Tier-2 Spill auf Sekundärspeicher. | `src/service.rs` |
| `pub async fn spill_segment( &self, tx_id: TxId, tenant_id: TenantId, segment_id: u64, payload: &[u8], ) -> Result<()>` | Persistiert ein Segment explizit verschlüsselt auf Tier-2 Sekundärspeicher. | `src/service.rs` |
| `pub fn spill_storage(mut self, storage: Arc<dyn KvBridgeStorage>) -> Self` | Konfiguriert den Tier-2 Spill-Storage Adapter ([`KvBridgeStorage`]). | `src/service.rs` |
| `pub fn stats(&self, tenant_id: TenantId) -> KvCacheStats` | Liefert Nutzungsstatistiken für den angegebenen Mandanten. | `src/service.rs` |
| `pub fn trigger_eviction(&self, target_free_bytes: usize)` | Löst eine nicht-blockierende Verdrängung im Hintergrund aus. | `src/service.rs` |
| `pub fn acquire_block_guard( &self, tenant: TenantId, block_id: u64, worker: Option<Arc<EvictionWorker>>, ) -> Option<KvBlockGuard>` | Erstellt einen [`KvBlockGuard`] für einen bestimmten Block. | `src/store.rs` |
| `pub fn attention_source(&self) -> Arc<dyn AttentionScoreSource>` | Liefert die konfigurierte Standard-[`AttentionScoreSource`] für diesen Store. | `src/store.rs` |
| `pub fn clear_all(&self)` | Löscht alle Segmente synchron aus dem Speicher und stellt Zeroization sicher (Emergency Wipe). | `src/store.rs` |
| `pub fn evict_fair(&self, target_free_bytes: usize, scores: &dyn AttentionScoreSource) -> usize` | Evictiert KV-Segmente unter Erhaltung von Tenant-Fairness und Attention-Scores (Default 50/50 Gewichtung). | `src/store.rs` |
| `pub fn evict_lru_fair(&self, target_free_bytes: usize) -> usize` | Evictiert KV-Segmente unter Erhaltung von Tenant-Fairness und Guard-Schutz (`active_refs == 0`). # Ablauf 1. Ruft [`Self::attention_source`… | `src/store.rs` |
| `pub fn evict_lru_fair_with_hook<F>(&self, target_free_bytes: usize, mut hook: F) -> usize where F: FnMut(),` | — | `src/store.rs` |
| `pub fn evict_weighted_fair( &self, target_free_bytes: usize, scores: &dyn AttentionScoreSource, attention_weight: f32, ) -> usize` | Evictiert KV-Segmente unter Erhaltung von Tenant-Fairness und gewichteten Attention-Scores. | `src/store.rs` |
| `pub fn find_prefix_match( &self, tenant: TenantId, tokens: &[u32], policy: KvReusePolicy, worker: Option<Arc<EvictionWorker>>, ) -> Option<(PrefixMatch, KvBlockGuard)>` | Sucht ein Präfix-Match für die angegebenen Tokens unter der gewünschten [`KvReusePolicy`]. | `src/store.rs` |
| `pub fn get_decrypted_segment( &self, cipher: &contextra_crypto::KvSegmentCipher, tenant: TenantId, segment_id: u64, ) -> Result<Option<Vec<u8>>, contextra_crypto::CryptoError>` | Liest ein Segment und entschlüsselt es falls nötig. | `src/store.rs` |
| `pub fn get_segment_bytes(&self, tenant: TenantId, segment_id: u64) -> Option<Vec<u8>>` | Liefert unverschlüsselte Segment-Bytes für einen Mandanten. | `src/store.rs` |
| `pub fn get_segments(&self, tenant: TenantId) -> Vec<u64>` | Liefert alle gespeicherten Segment-IDs eines Mandanten. | `src/store.rs` |
| `pub fn get_tenant_segment_len(&self, tenant: TenantId) -> usize` | Gibt die Anzahl der gespeicherten Segmente für einen Mandanten zurück. | `src/store.rs` |
| `pub fn insert_encrypted_segment( &self, cipher: &contextra_crypto::KvSegmentCipher, tenant: TenantId, segment_id: u64, model_fingerprint: contextra_crypto::ModelFingerprint, rope_offset: Option<usize>, plaintext: &[u8], ) -> Result<(), contextra_crypto::Crypt…` | Verschlüsselt einen Klartext-Tensor und fügt ein verschlüsseltes Segment ein. | `src/store.rs` |
| `pub fn insert_segment(&self, tenant: TenantId, segment: KvSegment)` | Fügt ein Segment für einen bestimmten Mandanten mit Standard-Direktive (`Auto`) ein. | `src/store.rs` |
| `pub fn insert_segment_with_directive( &self, tenant: TenantId, segment: KvSegment, directive: CacheDirective, ) -> Result<(), ContextraError>` | Fügt ein Segment mit einer deklarativen [`CacheDirective`] ein. # Ablauf 1. Bei `CacheDirective::NeverCache` sofort abbrechen (No-Op). 2. S… | `src/store.rs` |
| `pub fn insert_token_sequence( &self, tenant: TenantId, tokens: &[u32], block_id: u64, ) -> Result<(), ContextraError>` | Fügt eine Token-Sequenz und deren assoziierte Block-ID in den Prefix-Radix-Baum ein. | `src/store.rs` |
| `pub fn new() -> Self` | Erstellt einen neuen tenant-isolierten KV-Store mit Standard-Sharding. | `src/store.rs` |
| `pub fn on_rollback(&self, tenant: TenantId, chunk_ids: &[u64])` | Bereinigt assoziierte KV-Segmente bei einem transaktionalen Rollback (z. B. nach Abbruch einer Inferenz). | `src/store.rs` |
| `pub fn release_step(&self, tenant: TenantId, step_id: StepId)` | Gibt transiente Segmente frei, die mit einem bestimmten Ausführungsschritt verknüpft sind. | `src/store.rs` |
| `pub fn remove_segment(&self, tenant: TenantId, segment_id: u64)` | Entfernt ein einzelnes Segment. | `src/store.rs` |
| `pub fn remove_segments_for_rollback(&self, tenant: TenantId, segment_ids: &[u64])` | Entfernt Segmente mit den angegebenen IDs für einen Mandanten. | `src/store.rs` |
| `pub fn set_attention_source(&self, source: Arc<dyn AttentionScoreSource>)` | Setzt die konfigurierte Standard-[`AttentionScoreSource`] für diesen Store. | `src/store.rs` |
| `pub fn set_spill_handler(&self, handler: SpillHandler)` | Registriert einen Spill-Handler für evictierte Segmente (Tier-2 Auslagerung). | `src/store.rs` |
| `pub fn with_capacity(segment_capacity_per_tenant: usize) -> Self` | Erstellt einen Store mit konfigurierter Segment-Kapazität pro Mandant. | `src/store.rs` |
| `pub fn with_shard_count(n: usize) -> Self` | Erstellt einen Store mit benutzerdefinierter Shard-Anzahl (wird auf Zweierpotenz aufgerundet). | `src/store.rs` |


---

## `contextra-checkpoint`

| Attribut | Wert |
|---|---|
| **Ring** | 1 — Ring 1 — Persistence & Cache |
| **Status** | Fertig |
| **Zweck** | Time-Travel-Registry und Checkpoint-Verwaltung ohne globalen Zustand. |
| **Unsafe-Code** | Nein (`#![forbid(unsafe_code)]`) |
| **Interne Abhängigkeiten** | `contextra-mvcc`, `contextra-ports`, `contextra-types` |
| **Externe Abhängigkeiten (Auswahl)** | `blake3`, `bytes`, `dirs`, `futures-executor`, `parking_lot`, `serde`, `serde_json`, `tokio`, `tracing` |

### Modul- und Re-Export-Oberfläche (lib.rs)

```rust
pub use guard::{CheckpointGuard, PinGuard};
pub use hardlink_cloner::{CheckpointHardlinkCloner, DefaultHardlinkCloner, HardlinkCloneResult};
pub use manifest::CheckpointManifest;
pub use meta::{CheckpointMeta, StateCheckpoint};
pub use orphan::{ await_pending_rollbacks, clear_all_orphaned_checkpoints, clear_orphaned_checkpoint, get_orphaned_checkpoints, get_orphaned_checkpoints_for_namespace, global_orphan_registry, monotonic_timestamp_ms, monotonic_timestamp_ms_with, orphaned_checkpoint_count, pending_rollback_count, register_orphaned_checkpoint, register_pinned_seq_no_orphan, InstanceOrphanRegistry, OrphanRegistry, OrphanState, PinId, PinnedSeqNoOrphan, RecoveryReport, };
pub use store::{CheckpointRegistry, PersistentCheckpointStore};
```

### Schnittstellen — Public Traits (2)

**`CheckpointHardlinkCloner`** — *Trait für die Schnittstelle zum physischen SSTable-Klonen via Hardlinks. # Zweck Abstrahiert das Dateisystem-Klonen für Tests und verschiedene Plattformen. # Windows NTFS Limit Unter Windows ist `CreateHardLink` NTFS-exklusiv und auf maximal **1023 Links** pr…* (`src/hardlink_cloner.rs`)

```rust
pub trait CheckpointHardlinkCloner : Send + Sync {
/// Klont alle für `seq_no` sichtbaren SSTable-Dateien im Quellverzeichnis ins Zielverzeichnis.
    ///
    /// Hält während der Operation einen aktiven `SnapshotRegistry`-Pin auf `seq_no`.
    ///
    /// # Parameter
    /// - `seq_no`: Sequenznummer des MVCC-Snapshots.
    /// - `source_dir`: Quellverzeichnis mit bestehenden `.sst`-Dateien.
    /// - `target_dir`: Zielverzeichnis für die Klone.
    /// - `snapshot_registry`: Referenz auf die MVCC-Snapshot-Registry zum Pinning.
    ///
    /// # Fehler
    /// Gibt einen Fehler zurück bei Dateisystemproblemen oder unauflösbaren Link-Konflikten.
    fn clone_sstable_hardlinks<'a>(
        &'a self,
        seq_no: u64,
        source_dir: &'a Path,
        target_dir: &'a Path,
        snapshot_registry: &'a Arc<SnapshotRegistry>,
    ) -> contextra_ports::BoxFuture<'a, Result<HardlinkCloneResult>>;
}
```

**`CheckpointRegistry`** — *Trait für die erweiterte Checkpoint-Verwaltung inklusive Orphan-Recovery. Definiert die Schnittstelle zum Speichern, Laden und Bereinigen von Checkpoints.* (`src/store.rs`)

```rust
pub trait CheckpointRegistry : contextra_ports::Checkpoint + Send + Sync {
/// Speichert die Metadaten eines Checkpoints asynchron ab.
    fn save_checkpoint<'a>(&'a self, meta: CheckpointMeta) -> BoxFuture<'a, Result<()>>;
    /// Lädt die Metadaten eines Checkpoints anhand der Sequenznummer.
    fn load_checkpoint<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, Result<Option<CheckpointMeta>>>;
    /// Listet alle im Store registrierten Checkpoints auf.
    fn list_checkpoints<'a>(&'a self) -> BoxFuture<'a, Result<Vec<CheckpointMeta>>>;

    /// Liefert eine Referenz auf die zugeordnete Orphan-Registry, falls vorhanden.
    fn orphan_registry(&self) -> Option<&Arc<InstanceOrphanRegistry>> {
        None
    }

    /// Stellt verwaiste Pins wieder her und unpinnt sie in der Speicherengine.
    fn recover_orphaned_pins<'a>(&'a self) -> BoxFuture<'a, Result<Vec<PinId>>> {
        Box::pin(async move { Ok(Vec::new()) })
    }

    /// Führt für alle verwaisten Checkpoints ein kontrolliertes Rollback aus.
    fn recover_orphaned_checkpoints<'a>(&'a self) -> BoxFuture<'a, Result<Vec<TxId>>> {
        Box::pin(async move { Ok(Vec::new()) })
    }

    /// Schreibt ungespeicherte Orphan-Einträge auf die Festplatte.
    fn flush_orphan_registry<'a>(&'a self) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }
}
```

### Öffentliche Datentypen — Structs (16)

| Struct | Felder / Form | Beschreibung | Datei |
|---|---|---|---|
| `CheckpointGuard` | `{ pub(crate) checkpoint: Option<StateCheckpoint>, pub(crate) storage: Arc<S>, pub(crate) namespace: String, pub(crate) orphan_registry: Arc<InstanceOrphanRegistr… }` | RAII Guard, der explizit über [`commit`](Self::commit) oder [`rollback`](Self::rollback) finalisiert werden MUSS. # RAII-Kontrakt und Betriebshinweis… | `src/guard.rs` |
| `CheckpointManifest` | `{ pub meta: CheckpointMeta, pub components: Vec<String>, pub checksum: String, }` | Kryptografisch gesichertes Manifest eines gespeicherten Checkpoints. # Zweck Bindet Metadaten (`CheckpointMeta`) und zugehörige Komponenten-Dateien (… | `src/manifest.rs` |
| `CheckpointMeta` | `{ pub name: String, pub collection_id: String, pub seq_no: u64, pub tx_id: TxId, pub metadata: serde_json::Value, pub created_at: u64, }` | Metadaten für einen persistenten, benannten Checkpoint. # Zweck Speichert alle Identifikations-, Versions- und Audit-Informationen zu einem Snapshot… | `src/meta.rs` |
| `DefaultHardlinkCloner` | `(tuple/unit)` | Produktive Implementierung von [`CheckpointHardlinkCloner`] basierend auf `tokio::fs`. Nutzt asynchrone Dateisystem-Operationen ohne den Haupt-Thread… | `src/hardlink_cloner.rs` |
| `HardlinkCloneResult` | `{ /// Die gepinnte und geklonte Quell-Sequenznummer. pub source_seq_no: u64, /// Liste der Zielpfade, die als Hardlinks oder Kopien erzeugt wurden. pub linked_fi… }` | Ergebnisstruktur einer durchgeführten Hardlink-Klon-Operation für SSTables. # Zweck Liefert dem Aufrufer die Sequenznummer sowie die Liste aller erfo… | `src/hardlink_cloner.rs` |
| `InstanceOrphanRegistry` | `{ pins: Mutex<Vec<PinnedSeqNoOrphan>>, checkpoints: Mutex<Vec<StateCheckpoint>>, persist_path: std::path::PathBuf, is_dirty: AtomicBool, clock: Arc<dyn contextra… }` | Instanz-gebundene Orphan-Registry für Checkpoints und gepinnte Sequenznummern (ADR-053). # Thread-Sicherheit & Nebenläufigkeit Verwendet [`parking_lo… | `src/orphan.rs` |
| `LoomDirEntry` | `(tuple/unit)` | — | `src/hardlink_cloner.rs` |
| `LoomMetadata` | `(tuple/unit)` | — | `src/hardlink_cloner.rs` |
| `LoomReadDir` | `(tuple/unit)` | — | `src/hardlink_cloner.rs` |
| `OrphanRegistry` | `{ inner: InstanceOrphanRegistry, }` | — | `src/orphan.rs` |
| `OrphanState` | `{ pub checkpoints: Vec<StateCheckpoint>, pub pinned_seq_nos: Vec<PinnedSeqNoOrphan>, pub persist_path: std::path::PathBuf, }` | Serialisierbarer Gesamtzustand aller verwaisten Checkpoints und Pins einer Instanz. # Felder - `checkpoints`: Liste unvollendeter Transaktions-Checkp… | `src/orphan.rs` |
| `PersistentCheckpointStore` | `{ storage: Arc<S>, /// Registrierter Checkpoint-Index im Arbeitsspeicher — geschützt durch ein konsolidiertes RwLock (P-23) index: RwLock<CheckpointIndex>, /// N… }` | Registry für gespeicherte Checkpoints mit Thread-sicherem Zustand. # Zweck Verwaltet Checkpoint-Metadaten, koordiniert MVCC-Sequenznummern und stellt… | `src/store.rs` |
| `PinGuard` | `{ storage: Arc<S>, seq_no: Option<u64>, orphan_registry: Arc<InstanceOrphanRegistry>, clock: Arc<dyn contextra_ports::Clock>, }` | RAII-Guard für gepinnte Checkpoint-Sequenznummern (gemäß ADR-015). # Zweck Garantiert, dass eine gepinnte Sequenznummer bei einem Fehler oder Panic w… | `src/guard.rs` |
| `PinnedSeqNoOrphan` | `{ pub seq_no: u64, pub timestamp_ms: u64, }` | Verwaiste gepinnte Sequenznummer — wird beim Recovery verarbeitet. # Felder - `seq_no`: Die Sequenznummer, die entpinnt werden muss. - `timestamp_ms`… | `src/orphan.rs` |
| `RecoveryReport` | `{ /// Liste aller erfolgreich entpinnten Sequenznummern. pub recovered_pins: Vec<PinId>, /// Liste aller bereinigten Transaktions-IDs. pub recovered_checkpoints:… }` | Bericht über wiederhergestellte und aufgeräumte verwaiste Pins und Checkpoints. # Felder - `recovered_pins`: Liste aller erfolgreich entpinnten Seque… | `src/orphan.rs` |
| `StateCheckpoint` | `{ pub tx_id: TxId, pub timestamp_ms: u64, #[serde(default)] pub namespace: Option<String>, }` | Point-in-Time Checkpoint für einen Agenten-Schritt oder eine Transaktionsgrenze. # Zweck Repräsentiert einen leichten Transaktions-Zwischenstand zur… | `src/meta.rs` |

### Typ-Aliase (1)

| Alias | Definition | Beschreibung |
|---|---|---|
| `PinId` | `type PinId= u64` | Typ-Alias für Sequenznummern, die als gepinnte Checkpoint-Identifikatoren verwaltet werden. |

### Öffentliche Funktionen (103)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn checkpoint(&self) -> Result<&StateCheckpoint>` | Liefert eine Referenz auf den enthaltenen Checkpoint, sofern noch nicht konsumiert. | `src/guard.rs` |
| `pub fn clock(&self) -> &Arc<dyn contextra_ports::Clock>` | Liefert eine Referenz auf den injizierten Clock-Port. | `src/guard.rs` |
| `pub fn clock(&self) -> &Arc<dyn contextra_ports::Clock>` | Liefert den verwendeten Clock-Port. | `src/guard.rs` |
| `pub fn commit(mut self) -> Result<StateCheckpoint>` | Überführt den Guard in einen dauerhaften Zustand (Commit) und konsumiert ihn. # Ablauf von `commit`: 1. Checkpoint aus Option entnehmen (be… | `src/guard.rs` |
| `pub fn defuse(mut self)` | Entschärft den Guard nach erfolgreicher Persistierung, sodass die Sequenznummer gepinnt bleibt. | `src/guard.rs` |
| `pub async fn for_agent_step(storage: Arc<S>, tx: TxId) -> Result<Self>` | Erstellt einen neuen CheckpointGuard für einen Agenten-Schritt mit SystemClock. | `src/guard.rs` |
| `pub async fn for_agent_step_with_clock( storage: Arc<S>, tx: TxId, clock: Arc<dyn contextra_ports::Clock>, ) -> Result<Self>` | Erstellt einen CheckpointGuard für einen Agenten-Schritt mit speziellem Clock-Port. | `src/guard.rs` |
| `pub async fn for_agent_step_with_registry( storage: Arc<S>, tx: TxId, orphan_registry: Arc<InstanceOrphanRegistry>, ) -> Result<Self>` | Erstellt einen CheckpointGuard für einen Agenten-Schritt mit spezifischer Orphan-Registry. | `src/guard.rs` |
| `pub fn new(checkpoint: StateCheckpoint, storage: Arc<S>, namespace: impl Into<String>) -> Self` | Erzeugt einen neuen `CheckpointGuard` mit Standard-SystemClock. | `src/guard.rs` |
| `pub async fn pin( storage: Arc<S>, seq_no: u64, orphan_registry: Arc<InstanceOrphanRegistry>, ) -> Result<Self>` | Pinnt eine Sequenznummer in der Storage-Engine unter Verwendung der Clock aus der Orphan-Registry. # Parameter - `storage`: Die zugrundelie… | `src/guard.rs` |
| `pub async fn pin_with_clock( storage: Arc<S>, seq_no: u64, orphan_registry: Arc<InstanceOrphanRegistry>, clock: Arc<dyn contextra_ports::Clock>, ) -> Result<Self>` | Pinnt eine Sequenznummer mit explizit übergebenem Clock-Port. # Ablauf von `pin_with_clock`: 1. `storage.pin_checkpoint(seq_no)` asynchron… | `src/guard.rs` |
| `pub async fn rollback(mut self) -> Result<()>` | Führt ein manuelles, asynchrones Rollback des Checkpoints aus. Nach Aufruf von `rollback()` ist der Guard konsumiert, sodass beim Drop kein… | `src/guard.rs` |
| `pub fn rollback_blocking(mut self) -> Result<()>` | Führt ein synchrones, blockierendes Rollback des Checkpoints aus. # Einschränkung Diese Methode darf **nicht** aus einem laufenden asynchro… | `src/guard.rs` |
| `pub async fn unpin(mut self) -> Result<()>` | Entpinnt die Sequenznummer explizit und asynchron bei einem abgefangenen Fehler. # Ablauf von `unpin`: 1. `seq_no` entnehmen. 2. `storage.u… | `src/guard.rs` |
| `pub fn with_clock( checkpoint: StateCheckpoint, storage: Arc<S>, namespace: impl Into<String>, clock: Arc<dyn contextra_ports::Clock>, ) -> Self` | Erzeugt einen neuen `CheckpointGuard` mit injiziertem Clock-Port. | `src/guard.rs` |
| `pub fn with_metrics_sink(mut self, sink: Arc<dyn contextra_ports::MetricsSink>) -> Self` | Ergänzt den Guard um eine Metrik-Senke zur Erfassung von Ausführungsdauern. | `src/guard.rs` |
| `pub fn with_metrics_sink_opt( mut self, sink: Option<Arc<dyn contextra_ports::MetricsSink>>, ) -> Self` | Ergänzt den Guard optional um eine Metrik-Senke. | `src/guard.rs` |
| `pub fn with_registry( checkpoint: StateCheckpoint, storage: Arc<S>, namespace: impl Into<String>, orphan_registry: Arc<InstanceOrphanRegistry>, ) -> Self` | Erzeugt einen neuen `CheckpointGuard` mit vorhandener Orphan-Registry. | `src/guard.rs` |
| `pub fn with_registry_and_counter( checkpoint: StateCheckpoint, storage: Arc<S>, namespace: impl Into<String>, orphan_registry: Arc<InstanceOrphanRegistry>, skipped_rollbacks: Arc<AtomicU64>, ) -> Self` | Erzeugt einen neuen `CheckpointGuard` mit vorhandener Orphan-Registry und Zähler für übersprungene Rollbacks. | `src/guard.rs` |
| `pub fn with_registry_counter_and_clock( checkpoint: StateCheckpoint, storage: Arc<S>, namespace: impl Into<String>, orphan_registry: Arc<InstanceOrphanRegistry>, skipped_rollbacks: Arc<AtomicU64>, clock: Arc<dyn contextra_ports::Clock>, ) -> Self` | Vollständiger interner Konstruktor mit allen Parametern. | `src/guard.rs` |
| `pub async fn copy<P: AsRef<Path>, Q: AsRef<Path>>(_from: P, _to: Q) -> std::io::Result<u64>` | — | `src/hardlink_cloner.rs` |
| `pub async fn create_dir_all<P: AsRef<Path>>(_path: P) -> std::io::Result<()>` | — | `src/hardlink_cloner.rs` |
| `pub async fn hard_link<P: AsRef<Path>, Q: AsRef<Path>>( _from: P, _to: Q, ) -> std::io::Result<()>` | — | `src/hardlink_cloner.rs` |
| `pub async fn metadata<P: AsRef<Path>>(_path: P) -> std::io::Result<LoomMetadata>` | — | `src/hardlink_cloner.rs` |
| `pub fn new() -> Self` | Erzeugt eine neue Instanz von [`DefaultHardlinkCloner`]. | `src/hardlink_cloner.rs` |
| `pub async fn next_entry(&mut self) -> std::io::Result<Option<LoomDirEntry>>` | — | `src/hardlink_cloner.rs` |
| `pub fn path(&self) -> std::path::PathBuf` | — | `src/hardlink_cloner.rs` |
| `pub async fn read_dir<P: AsRef<Path>>(_path: P) -> std::io::Result<LoomReadDir>` | — | `src/hardlink_cloner.rs` |
| `pub async fn remove_file<P: AsRef<Path>>(_path: P) -> std::io::Result<()>` | — | `src/hardlink_cloner.rs` |
| `pub fn new(meta: CheckpointMeta, components: Vec<String>) -> Result<Self>` | Erzeugt ein neues [`CheckpointManifest`] und berechnet die BLAKE3-Prüfsumme. # Vorbedingungen - `meta.name` und `meta.collection_id` müssen… | `src/manifest.rs` |
| `pub fn verify(&self) -> Result<()>` | Verifiziert die Integrität des Manifests anhand der gespeicherten Prüfsumme. # Ablauf von `verify`: 1. Tupel `(&self.meta, &self.components… | `src/manifest.rs` |
| `pub fn into_workflow_state(&self) -> WorkflowState` | Konvertiert die Metadaten in einen [`WorkflowState`]. # Kompatibilität Behält `&self` für Rückwärtskompatibilität mit externen Aufrufern be… | `src/meta.rs` |
| `pub fn to_workflow_state(&self) -> WorkflowState` | Konvertiert die Metadaten in einen [`WorkflowState`]. # Ablauf: 1. Transaktions-ID aus `tx_id` übernehmen. 2. Deterministic Hash aus der Se… | `src/meta.rs` |
| `pub async fn await_pending_rollbacks()` | Rückwärtskompatibilität: Wartet auf ausstehende Rollbacks (No-op, da Hintergrund-Tasks entfernt wurden). | `src/orphan.rs` |
| `pub fn clear_all(&self)` | Löscht alle Einträge im Speicher und auf der Platte synchron. | `src/orphan.rs` |
| `pub fn clear_all(&self)` | Löscht alle Einträge (Pins und Checkpoints) und leert die Statusdatei. | `src/orphan.rs` |
| `pub fn clear_all_orphaned_checkpoints()` | — | `src/orphan.rs` |
| `pub fn clear_orphan_pin(&self, seq_no: u64)` | Entfernt eine einzelne gepinnte Sequenznummer aus dem Speicher und persistiert. | `src/orphan.rs` |
| `pub fn clear_orphaned_checkpoint(&self, tx_id: TxId)` | Entfernt einen einzelnen verwaisten Checkpoint anhand seiner Transaktions-ID und persistiert. | `src/orphan.rs` |
| `pub fn clear_orphaned_checkpoint(_tx_id: TxId)` | — | `src/orphan.rs` |
| `pub fn clock(&self) -> &Arc<dyn contextra_ports::Clock>` | Liefert eine Referenz auf den injizierten Clock-Port. | `src/orphan.rs` |
| `pub fn drain_orphan_pins(&self) -> Vec<PinnedSeqNoOrphan>` | Entnimmt alle verwaisten Pins aus dem Speicher und aktualisiert die Statusdatei. | `src/orphan.rs` |
| `pub fn drain_orphaned_checkpoints(&self) -> Vec<StateCheckpoint>` | Entnimmt alle verwaisten Checkpoints aus dem Speicher und aktualisiert die Statusdatei. | `src/orphan.rs` |
| `pub async fn flush_orphan_registry(&self) -> std::io::Result<()>` | Flusht die In-Memory-Einträge asynchron auf Festplatte, sofern Änderungen vorliegen (`is_dirty`). # Ablauf von `flush_orphan_registry`: 1.… | `src/orphan.rs` |
| `pub fn get_orphan_pins(&self) -> Vec<PinnedSeqNoOrphan>` | Liefert eine Momentaufnahme aller verwaisten Pins im Speicher. | `src/orphan.rs` |
| `pub fn get_orphaned_checkpoints(&self) -> Vec<StateCheckpoint>` | Liefert eine Momentaufnahme aller verwaisten Checkpoints im Speicher. | `src/orphan.rs` |
| `pub fn get_orphaned_checkpoints() -> Vec<StateCheckpoint>` | — | `src/orphan.rs` |
| `pub fn get_orphaned_checkpoints_for_namespace(ns: &str) -> Vec<StateCheckpoint>` | Liefert verwaiste Checkpoints eines bestimmten Namespaces (veraltet). | `src/orphan.rs` |
| `pub fn get_orphans(&self) -> Vec<PinId>` | Liefert alle aktuell registrierten verwaisten Pin-Sequenznummern. | `src/orphan.rs` |
| `pub fn global_orphan_registry() -> &'static OrphanRegistry` | — | `src/orphan.rs` |
| `pub fn is_empty(&self) -> bool` | Liefert `true`, wenn keine verwaisten Einträge vorhanden waren oder bereinigt wurden. | `src/orphan.rs` |
| `pub fn load_sync(path: &std::path::Path) -> Self` | Lädt den Zustand synchron von der Festplatte oder liefert einen leeren Zustand bei Fehlern. | `src/orphan.rs` |
| `pub fn load_sync(path: &std::path::Path) -> Self` | Lädt den Zustand synchron von der Festplatte. | `src/orphan.rs` |
| `pub fn monotonic_timestamp_ms() -> u64` | Liest den Millisekunden-Zeitstempel über die standardmäßige SystemClock. | `src/orphan.rs` |
| `pub fn monotonic_timestamp_ms_with(clock: &dyn contextra_ports::Clock) -> u64` | Liest einen Millisekunden-Zeitstempel deterministisch über die übergebene Clock-Port-Instanz. # Parameter - `clock`: Referenz auf eine Impl… | `src/orphan.rs` |
| `pub fn new(file_path: impl Into<std::path::PathBuf>) -> Self` | Erzeugt eine neue Instanz von [`OrphanRegistry`]. | `src/orphan.rs` |
| `pub fn new(persist_path: impl Into<std::path::PathBuf>) -> Self` | Erzeugt eine neue instanzgebundene Registry für den angegebenen Dateipfad. | `src/orphan.rs` |
| `pub fn orphaned_checkpoint_count() -> usize` | Liefert die Anzahl der aktuell registrierten verwaisten ("orphaned") Checkpoints. | `src/orphan.rs` |
| `pub fn pending_rollback_count() -> usize` | Rückwärtskompatibilität: Liefert die Anzahl ausstehender Rollback-Hintergrundaufgaben (immer 0). | `src/orphan.rs` |
| `pub fn persist_sync(&self) -> std::io::Result<()>` | Schreibt den Zustand synchron als formatierte JSON-Datei auf die Festplatte. | `src/orphan.rs` |
| `pub fn persist_sync(&self) -> std::io::Result<()>` | Persistiert den aktuellen Zustand synchron auf die Festplatte. | `src/orphan.rs` |
| `pub async fn recover_and_clean<S: contextra_ports::StorageEngine>( &self, storage: &S, ) -> Result<Vec<PinId>>` | Bereinigt alle verwaisten Pins durch Entpinnen in der Storage-Engine. # Ablauf von `recover_and_clean`: 1. Liste aller verwaisten Pins abfr… | `src/orphan.rs` |
| `pub fn register_checkpoint_sync(&self, cp: StateCheckpoint)` | Registriert einen verwaisten Checkpoint rein im Arbeitsspeicher ohne blockierendes Disk-I/O. # Crash-Safety & RAII Drop Semantik Wird synch… | `src/orphan.rs` |
| `pub fn register_orphan(&self, pin_id: PinId) -> std::io::Result<()>` | Registriert synchron einen verwaisten Pin. | `src/orphan.rs` |
| `pub fn register_orphan_sync(&self, orphan: PinnedSeqNoOrphan)` | Registriert einen verwaisten Sequenz-Pin rein im Arbeitsspeicher ohne blockierendes Disk-I/O. # Crash-Safety & RAII Drop Semantik Wird sync… | `src/orphan.rs` |
| `pub fn register_orphaned_checkpoint(cp: StateCheckpoint)` | — | `src/orphan.rs` |
| `pub fn register_pinned_seq_no_orphan(orphan: PinnedSeqNoOrphan)` | — | `src/orphan.rs` |
| `pub fn set_clock(&mut self, clock: Arc<dyn contextra_ports::Clock>)` | Setzt die Clock-Implementierung nachträglich. | `src/orphan.rs` |
| `pub fn total_recovered(&self) -> usize` | Liefert die Gesamtzahl aller bereinigten Pins und Checkpoints. | `src/orphan.rs` |
| `pub fn with_clock(mut self, clock: Arc<dyn contextra_ports::Clock>) -> Self` | Konfiguriert die Registry mit einer spezifischen Clock-Implementierung. | `src/orphan.rs` |
| `pub async fn allocate_tx(&self) -> Result<TxId>` | Reserviert eine neue Transaktions-ID aus dem reservierten internen Bereich (`INTERNAL_BASE`). # Ablauf von `allocate_tx`: 1. Atomaren Zähle… | `src/store.rs` |
| `pub fn checkpoint_counter(&self) -> u64` | Liefert den Zählerstand für Checkpoint-Zeitstempel. | `src/store.rs` |
| `pub fn checkpoint_guard_skipped_rollback_count(&self) -> u64` | Alias für `skipped_rollback_count`. | `src/store.rs` |
| `pub fn clear_all_orphaned_checkpoints(&self)` | Löscht alle verwaisten Checkpoints. | `src/store.rs` |
| `pub fn clear_orphaned_checkpoint(&self, tx_id: TxId)` | Löscht einen verwaisten Checkpoint anhand seiner Transaktions-ID. | `src/store.rs` |
| `pub fn clock(&self) -> &Arc<dyn contextra_ports::Clock>` | Liefert den aktuellen Clock-Port. | `src/store.rs` |
| `pub async fn close(&self) -> Result<()>` | Alias für `shutdown()`. Führt einen kontrollierten Flush beim Schließen aus. | `src/store.rs` |
| `pub async fn create_checkpoint( &self, name: &str, collection_id: &str, seq_no: u64, tx_id: TxId, metadata: serde_json::Value, ) -> Result<CheckpointMeta>` | Creates a new persistent checkpoint. # Ablauf von `create_checkpoint`: 1. Bezeichner `name` und `collection_id` validieren. 2. `write_lock`… | `src/store.rs` |
| `pub fn create_guard(&self, tx_id: TxId) -> Result<CheckpointGuard<S>>` | Creates an ephemeral transactional checkpoint RAII guard. If the returned guard is dropped without calling `.commit()`, the underlying stor… | `src/store.rs` |
| `pub async fn create_hardlink_clone( &self, seq_no: u64, source_dir: &Path, target_dir: &Path, snapshot_registry: &Arc<SnapshotRegistry>, ) -> Result<HardlinkCloneResult>` | Creates a physical hardlink clone of all SSTable files visible for `seq_no` into `target_dir`. Delegates to [`DefaultHardlinkCloner`] while… | `src/store.rs` |
| `pub async fn drop_checkpoint(&self, name: &str) -> Result<()>` | Deletes a persistent checkpoint by name. # Ablauf von `drop_checkpoint`: 1. Bezeichner validieren und `write_lock` holen. 2. Checkpoint lad… | `src/store.rs` |
| `pub async fn flush_orphan_registry(&self) -> Result<()>` | Flusht die Orphan-Registry asynchron auf Platte. | `src/store.rs` |
| `pub async fn get_checkpoint(&self, name: &str) -> Result<Option<CheckpointMeta>>` | Ruft einen Checkpoint nach Namen ab. | `src/store.rs` |
| `pub fn get_orphaned_checkpoints(&self) -> Vec<StateCheckpoint>` | Liefert alle verwaisten Checkpoints. | `src/store.rs` |
| `pub async fn list_checkpoints(&self) -> Result<Vec<CheckpointMeta>>` | Listet alle persistent gespeicherten Checkpoints im konfigurierten Namespace auf. # Ablauf von `list_checkpoints`: 1. Präfix-Scan auf der S… | `src/store.rs` |
| `pub fn monotonic_timestamp_ms(&self) -> u64` | Liefert einen monoton steigenden Zeitstempel in Millisekunden. | `src/store.rs` |
| `pub fn new(storage: Arc<S>, namespace: impl Into<String>) -> Result<Self>` | Synchroner Konstruktor für Umgebungen ohne asynchronen Tokio-Einstiegspunkt. | `src/store.rs` |
| `pub fn new_with_orphan_registry( storage: Arc<S>, namespace: impl Into<String>, orphan_registry: Arc<InstanceOrphanRegistry>, ) -> Result<Self>` | Synchroner Konstruktor mit injizierter Orphan Registry. | `src/store.rs` |
| `pub async fn open(storage: Arc<S>, namespace: impl Into<String>) -> Result<Self>` | Öffnet einen PersistentCheckpointStore asynchron mit Rekonstruktion und Monotonie-Garantie. | `src/store.rs` |
| `pub async fn open_and_recover( storage: Arc<S>, namespace: impl Into<String>, ) -> Result<(Self, RecoveryReport)>` | Öffnet einen PersistentCheckpointStore und führt eine vollständige Orphan-Recovery für Pins und Checkpoints durch. # Ablauf von `open_and_r… | `src/store.rs` |
| `pub async fn open_and_recover_with_orphan_registry( storage: Arc<S>, namespace: impl Into<String>, orphan_registry: Arc<InstanceOrphanRegistry>, ) -> Result<(Self, RecoveryReport)>` | Öffnet einen PersistentCheckpointStore mit einer spezifischen instanzgebundenen Orphan Registry und führt Orphan-Recovery durch. | `src/store.rs` |
| `pub async fn open_with_orphan_registry( storage: Arc<S>, namespace: impl Into<String>, orphan_registry: Arc<InstanceOrphanRegistry>, ) -> Result<Self>` | Öffnet einen PersistentCheckpointStore mit einer spezifischen instanzgebundenen Orphan Registry. | `src/store.rs` |
| `pub fn orphan_registry(&self) -> &Arc<InstanceOrphanRegistry>` | Liefert eine Referenz auf die Orphan-Registry. | `src/store.rs` |
| `pub async fn recover_orphaned_checkpoints(&self) -> Result<Vec<TxId>>` | Recovers all registered/persisted orphaned checkpoints during controlled startup or recovery. Checks the serialization barrier (`last_tx <=… | `src/store.rs` |
| `pub async fn recover_orphaned_pins(&self) -> Result<Vec<PinId>>` | Recovers all registered/persisted orphaned sequence pins (ADR-052). # Ablauf von `recover_orphaned_pins`: 1. Liste verwaister Pins aus der… | `src/store.rs` |
| `pub fn register_orphaned_checkpoint(&self, cp: StateCheckpoint)` | Registriert einen verwaisten Checkpoint. | `src/store.rs` |
| `pub fn register_pinned_seq_no_orphan(&self, orphan: PinnedSeqNoOrphan)` | Registriert einen verwaisten Sequenz-Pin. | `src/store.rs` |
| `pub async fn restore_checkpoint(&self, name: &str) -> Result<CheckpointMeta>` | Restores the system to a specific checkpoint by name. This will rollback the underlying storage to the transaction ID of the checkpoint. #… | `src/store.rs` |
| `pub fn set_metrics_sink(&mut self, sink: Arc<dyn contextra_ports::MetricsSink>)` | Setzt die Metrik-Senke. | `src/store.rs` |
| `pub async fn shutdown(&self) -> Result<()>` | Führt einen expliziten Flush der Orphan-Registry beim Herunterfahren durch. | `src/store.rs` |
| `pub fn skipped_rollback_count(&self) -> u64` | Liefert die Anzahl übersprungener Rollbacks. | `src/store.rs` |
| `pub fn with_clock(mut self, clock: Arc<dyn contextra_ports::Clock>) -> Self` | Setzt den Zeitgeber für den `PersistentCheckpointStore`. # Composition Root Der Composition Root setzt per `with_clock` eine injizierte Uhr… | `src/store.rs` |
| `pub fn with_metrics_sink(mut self, sink: Arc<dyn contextra_ports::MetricsSink>) -> Self` | Konfiguriert den Store mit einer Metrik-Senke. | `src/store.rs` |


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
| `test-utils` | `[]` |

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

### Öffentliche Datentypen — Structs (22)

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
| `GenerationConfig` | `{ /// Random number generator seed (default 299792458). pub seed: u64, /// Sampling temperature (0.0 = greedy deterministic bit-identical sampling). pub temperat… }` | Sampling and resource bounds configuration for Candle LLM text generation. | `src/inference/config.rs` |
| `GgufMetadata` | `{ /// Identified architecture (e.g., "llama", "mistral"). pub architecture: String, /// Number of tensors present in the container. pub tensor_count: usize, ///… }` | Metadata extracted from a GGUF model container. | `src/gguf_loader.rs` |
| `KvBridgeAdapter` | `{ pub store: Arc<TenantIsolatedKvStore>, pub lsm_store: Option<Arc<dyn KvBridgeStorage>>, pub cipher: Arc<KvSegmentCipher>, pub consultations: Arc<std::sync::ato… }` | Verbindet Retrieval-Chunks mit dem mandantenisolierten, verschlüsselten KV-Cache. # Fail-Safe-Garantie Jede Methode fällt bei Fehler oder Cache-Miss… | `src/kv_bridge.rs` |
| `KvCacheKey` | `{ pub chunk_id: u64, pub fingerprint: ModelFingerprint, pub rope_offset: Option<usize>, }` | Cache-Lookup-Schlüssel: eindeutige Kombination aus Chunk-ID, Modell und optionalem RoPE-Offset. | `src/kv_bridge.rs` |
| `KvPrefixContext` | `{ /// Reference to the underlying KV prefix store port implementation. pub store: Arc<dyn KvPrefixStore>, }` | Wrapper context holding a thread-safe KV prefix store reference. | `src/inference/prefix_reuse.rs` |
| `KvState` | `{ /// Layer-wise KV cache tensor pairs. pub layers: Vec<LayerKv>, /// Active sequence length (token count) represented in the cache. pub pos: usize, }` | Explicit KV-Cache state holding layer-wise tensors and current sequence length position. | `src/kv_state.rs` |
| `LayerKv` | `{ /// Key tensor of shape `(batch, seq_len, n_kv_head, head_dim)` or `(batch, n_kv_head, seq_len, head_dim)`. pub k: Tensor, /// Value tensor of shape `(batch, s… }` | Represents the key (`k`) and value (`v`) tensors for a single transformer layer. | `src/kv_state.rs` |
| `LayerWeights` | `{ pub attention_wq: QuantizedMatMul, pub attention_wk: QuantizedMatMul, pub attention_wv: QuantizedMatMul, pub attention_wo: QuantizedMatMul, pub attention_norm:… }` | — | `src/model/quantized_llama.rs` |
| `ModelWeights` | `{ /// Token-Einbettungsmatrix. pub tok_embeddings: Embedding, /// Decoder-Schichten des Transformer-Netzwerks. pub layers: Vec<LayerWeights>, /// Finale RMS-Norm… }` | — | `src/model/quantized_llama.rs` |
| `PrefixRun` | `{ /// Generated output string. pub output: String, /// Tokenized prompt sequence. pub prompt_tokens: Vec<u32>, /// Number of prompt tokens reused from the KV sto… }` | Execution result of a prefix-assisted generation run. | `src/inference/prefix_reuse.rs` |
| `PrefixSeed` | `{ /// Number of prompt tokens covered by the imported KV state. pub matched_len: usize, /// Imported and truncated KV state ready for remaining prompt prefill. p… }` | Seed state for executing prompt generation with an imported KV cache prefix. | `src/inference/prefix_reuse.rs` |
| `QuantizedLlamaModel` | `{ weights: ModelWeights, sample_len: usize, }` | Real quantized Llama / GGUF model execution wrapper. | `src/inference/core.rs` |
| `QuantizedLlamaModel` | `{ weights: ModelWeights, config: GenerationConfig, }` | Real quantized Llama / GGUF model execution wrapper. | `src/inference/quantized_llama.rs` |
| `QuantizedMatMul` | `{ inner: QMatMul, span: tracing::Span, }` | — | `src/model/quantized_llama.rs` |

### Öffentliche Datentypen — Enums (1)

| Enum | Varianten | Beschreibung | Datei |
|---|---|---|---|
| `CandleQuantization` | `/// 4-bit medium K-quantization Q4KM, /// 8-bit quantization Q8_0, /// 16-bit floating point F16,` | Supported Candle quantization formats. | `src/model_registry.rs` |

### Konstanten (10)

| Konstante | Typ | Beschreibung |
|---|---|---|
| `DEFAULT_GROUNDING_THRESHOLD` | `f32` | Standard-Schwellenwert für Grounding-Konfidenz (Default: 0.70). |
| `DEFAULT_MAX_CONCURRENT_EMBEDDINGS` | `usize` | Default maximum concurrent embedding operations for Candle vector embedding. |
| `DEFAULT_MAX_CONCURRENT_INFERENCES` | `usize` | Default maximum concurrent inference operations for Candle LLM text generation. |
| `MAX_CANDLE_EMBED_BATCH_SIZE` | `usize` | Maximum batch size for Candle GGUF embeddings. Set to 256 (lower than ONNX's 512) because GGUF tensor activations and intermediate FP32/FP16 buffers… |
| `MAX_GGUF_METADATA_DEPTH` | `usize` | Maximum permitted depth for nested GGUF metadata array parsing. |
| `MAX_GGUF_METADATA_KVS` | `u64` | Maximum number of metadata key-value pairs permitted in a valid GGUF header. |
| `MAX_GGUF_STRING_LEN` | `u64` | Maximum length of a string in bytes permitted in a GGUF metadata string field. |
| `MAX_GGUF_TENSORS` | `u64` | Maximum number of tensors permitted in a valid GGUF header. |
| `MAX_SEQ_LEN` | `usize` | — |
| `MAX_TRACKED_REQUESTS` | `usize` | Maximum number of tracked requests in the attention history ring buffer. |

### Öffentliche Funktionen (70)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn clear(&self)` | Clears all tracked request attention histories. | `src/attention_exporter.rs` |
| `pub fn new() -> Self` | Creates a new `CandleAttentionExporter`. | `src/attention_exporter.rs` |
| `pub fn record_attention_weights(&self, request_id: RequestId, weights: Vec<f32>)` | Records pre-summed attention weights for a request ID. Enforces [`MAX_TRACKED_REQUESTS`] ring-buffer capacity limit. | `src/attention_exporter.rs` |
| `pub fn record_layer_head_scores(&self, request_id: RequestId, scores: &[Vec<f32>])` | Combines and records attention weights from layer/head score matrices/vectors. Sums weights across heads/layers per token position in $O(\t… | `src/attention_exporter.rs` |
| `pub fn tracked_request_count(&self) -> usize` | Returns the current number of tracked requests in memory. | `src/attention_exporter.rs` |
| `pub fn fingerprint(&self) -> &ModelFingerprint` | Returns a reference to the model's fingerprint. | `src/embedding.rs` |
| `pub fn from_dir( model_dir: &std::path::Path, quantization: crate::model_registry::CandleQuantization, ) -> Result<Self>` | Loads a `CandleEmbedClient` from a model directory. | `src/embedding.rs` |
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
| `pub fn parse_gguf_metadata(model_path: &Path) -> Result<GgufMetadata, ContextraError>` | Inspects a GGUF model file and extracts its architecture and metadata header keys after performing strict header validation and optional SH… | `src/gguf_loader.rs` |
| `pub fn parse_gguf_metadata_with_checksum( model_path: &Path, expected_sha256: Option<&[u8; 32]>, ) -> Result<GgufMetadata, ContextraError>` | Inspects a GGUF model file with optional SHA-256 verification. | `src/gguf_loader.rs` |
| `pub fn deterministic(seed: u64) -> Self` | Creates a deterministic generation configuration (temperature = 0.0). | `src/inference/config.rs` |
| `pub fn validate_prompt_len(&self, prompt_token_count: usize) -> Result<(), ContextraError>` | Validates input prompt token length against `max_context_length`. | `src/inference/config.rs` |
| `pub fn validate_token_budget(&self, requested_tokens: usize) -> Result<(), ContextraError>` | Validates target sample length against `max_token_budget`. | `src/inference/config.rs` |
| `pub fn with_max_context_length(mut self, limit: usize) -> Self` | Sets the maximum context length in tokens. | `src/inference/config.rs` |
| `pub fn with_max_token_budget(mut self, budget: usize) -> Self` | Sets the maximum token budget for generation. | `src/inference/config.rs` |
| `pub fn fingerprint(&self) -> &ModelFingerprint` | Returns a reference to the model's fingerprint. | `src/inference/core.rs` |
| `pub fn from_dir( model_dir: &std::path::Path, quantization: crate::model_registry::CandleQuantization, ) -> Result<Self>` | Loads a `CandleLlmClient` from a model directory. | `src/inference/core.rs` |
| `pub fn generate_with_prefix_store<'a>( &'a self, tenant: TenantId, segments: &'a [ContextSegment<'a>], ) -> BoxFuture<'a, Result<String>>` | Generates completion with stage B KV prefix reuse and tenant isolation. | `src/inference/core.rs` |
| `pub fn load(model_path: &Path, device: &Device) -> Result<Self>` | Loads GGUF quantized model weights from the specified file path. | `src/inference/core.rs` |
| `pub fn new( device: Device, model: Box<dyn CandleModelInner + Send>, fingerprint: ModelFingerprint, tokenizer: tokenizers::Tokenizer, ) -> Self` | Creates a new `CandleLlmClient` with default concurrency limits. | `src/inference/core.rs` |
| `pub fn prefill_count(&self) -> u64` | Returns the total count of segment full prefill calculations performed. | `src/inference/core.rs` |
| `pub fn prefill_skip_count(&self) -> u64` | Returns the total count of segment prefill skips via KV cache hits. | `src/inference/core.rs` |
| `pub fn prefill_skipped_tokens(&self) -> u64` | Returns the total count of prompt tokens skipped via KV prefix reuse. | `src/inference/core.rs` |
| `pub fn swap_model( &mut self, new_model: Box<dyn CandleModelInner + Send>, new_fingerprint: ModelFingerprint, validator: Option<&mut GaspValidator>, )` | Tauscht das zugrundeliegende Modell und dessen Fingerprint zur Laufzeit aus. Falls ein optionaler `GaspValidator` übergeben wird, wird dess… | `src/inference/core.rs` |
| `pub fn with_kv_bridge(mut self, adapter: crate::KvBridgeAdapter) -> Self` | Configures optional `KvBridgeAdapter` for KV cache segment retrieval and injection. | `src/inference/core.rs` |
| `pub fn with_max_concurrent_inferences(mut self, limit: usize) -> Self` | Configures maximum concurrent inference operations for backpressure control. | `src/inference/core.rs` |
| `pub fn with_prefix_store(mut self, store: Arc<dyn contextra_ports::kv::KvPrefixStore>) -> Self` | Configures optional `KvPrefixStore` context for stage B prefix reuse. | `src/inference/core.rs` |
| `pub fn build_prefix_key( fingerprint: &ModelFingerprint, tokenizer: &tokenizers::Tokenizer, layout: KvLayout, rope: RopeConfig, ) -> Result<PrefixKey>` | Constructs a `PrefixKey` for the given model fingerprint, tokenizer, layout, and RoPE config. | `src/inference/prefix_reuse.rs` |
| `pub fn seed_from_hit(hit: &KvPrefixHit, tokens_len: usize) -> Result<PrefixSeed>` | Reconstructs a `PrefixSeed` from a `KvPrefixHit` and total prompt token count. Ensures at least one prompt token remains to be evaluated fo… | `src/inference/prefix_reuse.rs` |
| `pub fn clear_kv_cache(&mut self)` | Clears the underlying layer-wise KV cache tensors completely. | `src/inference/quantized_llama.rs` |
| `pub fn load(model_path: &Path, device: &Device) -> Result<Self>` | Loads GGUF quantized model weights from the specified file path with default generation config. | `src/inference/quantized_llama.rs` |
| `pub fn load_with_config( model_path: &Path, device: &Device, config: GenerationConfig, ) -> Result<Self>` | Loads GGUF quantized model weights from file path with custom `GenerationConfig`. | `src/inference/quantized_llama.rs` |
| `pub fn truncate_kv_cache(&mut self, pos: usize) -> Result<()>` | Truncates the underlying layer-wise KV cache tensors to `pos` tokens. Used when context window overflow occurs or prefix reuse requires res… | `src/inference/quantized_llama.rs` |
| `pub fn with_generation_config(mut self, config: GenerationConfig) -> Self` | Configures generation config for sampling parameters and context bounds. | `src/inference/quantized_llama.rs` |
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
| `pub fn clear_kv_cache(&mut self)` | — | `src/model/quantized_llama.rs` |
| `pub fn export_kv_state(&self) -> Result<KvState, ContextraError>` | — | `src/model/quantized_llama.rs` |
| `pub fn forward(&mut self, x: &Tensor, index_pos: usize) -> Result<Tensor, ContextraError>` | — | `src/model/quantized_llama.rs` |
| `pub fn from_gguf<R: std::io::Seek + std::io::Read>( ct: gguf_file::Content, reader: &mut R, device: &Device, ) -> Result<Self, ContextraError>` | — | `src/model/quantized_llama.rs` |
| `pub fn import_kv_state(&mut self, state: &KvState) -> Result<(), ContextraError>` | — | `src/model/quantized_llama.rs` |
| `pub fn truncate_kv_cache(&mut self, pos: usize) -> Result<(), ContextraError>` | — | `src/model/quantized_llama.rs` |
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
pub mod query_rewriter;
pub use client::{ build_rag_prompt, read_response_bytes, read_response_text, xml_escape, CircuitBreaker, EgressAuditSink, EndpointPolicy, NoopEgressAuditSink, OllamaClient, OllamaConfig, DEFAULT_BASE_URL, DEFAULT_EMBED_MODEL, };
pub use context_prefixer::{ContextPrefixConfig, ContextPrefixEngine, ContextPrefixer};
pub use embedding::OllamaEmbedder;
pub use importance::{ parse_importance_score_response, score_importance, score_importance_batch, score_importance_with_calibrator, Confidence, ImportanceAssessment, };
pub use model_info::ModelInfo;
pub use query_rewriter::{OllamaQueryRewriter, OllamaQueryRewriterConfig};
```

### Schnittstellen — Public Traits (1)

**`EgressAuditSink`** — *Trait for auditing remote egress access attempts.* (`src/client/policy.rs`)

```rust
pub trait EgressAuditSink : Send + Sync + std::fmt::Debug {
/// Called when an egress check is performed for a remote (non-loopback) endpoint.
    fn audit_egress(&self, endpoint: &str, allowed: bool, reason: &str);

    /// Returns `true` if this audit sink is a no-op sink (default).
    fn is_noop(&self) -> bool {
        false
    }
}
```

### Öffentliche Datentypen — Structs (12)

| Struct | Felder / Form | Beschreibung | Datei |
|---|---|---|---|
| `CircuitBreaker` | `{ failure_threshold: u32, reset_timeout_nanos: u64, state: Mutex<BreakerState>, }` | Thread-safe circuit breaker backed by the `Clock` port. | `src/client/circuit_breaker.rs` |
| `ContextPrefixConfig` | `{ /// Ollama-Modell für Prefix-Generierung (sollte schnell & klein sein). /// Empfehlung: "llama3.2:3b" oder "gemma2:2b" — nicht das Haupt-Chat-Modell. pub model… }` | Konfiguration für Kontext-Präfix-Generierung. | `src/context_prefixer.rs` |
| `ContextPrefixEngine` | `{ client: OllamaClient, config: ContextPrefixConfig, }` | Generiert LLM-basierte Kontext-Präfixe für Dokument-Chunks. # Verwendung ```no_run # async fn example() -> Result<(), contextra_types::ContextraError… | `src/context_prefixer.rs` |
| `EndpointPolicy` | `{ /// Whether non-loopback remote endpoints are permitted. Default: `false`. pub allow_remote: bool, /// Explicit allowlist of allowed remote hosts or host:port… }` | Endpoint policy governing allowed egress destinations for Ollama requests. | `src/client/policy.rs` |
| `ImportanceAssessment` | `{ /// The normalized importance score (0.0 to 1.0). pub score: ImportanceScore, /// Confidence indicator describing whether the score was parsed or defaulted. pu… }` | Result of an LLM importance evaluation containing the score, parse confidence status, model provenance, and optional calibrated confidence. | `src/importance.rs` |
| `ModelInfo` | `{ pub modelfile: Option<String>, pub parameter_size: Option<String>, pub quantization_level: Option<String>, }` | — | `src/model_info.rs` |
| `NoopEgressAuditSink` | `(tuple/unit)` | Default no-op implementation of `EgressAuditSink`. | `src/client/policy.rs` |
| `OllamaClient` | `{ pub(crate) config: OllamaConfig, pub(crate) client: reqwest::Client, pub(crate) client_build_error: Option<String>, pub(crate) rng: Arc<dyn Rng>, pub(crate) cl… }` | HTTP client for interacting with a local or configured Ollama instance. | `src/client/core.rs` |
| `OllamaConfig` | `{ /// Base URL for the Ollama instance (default: `http://localhost:11434`) pub base_url: String, /// Model name for embedding generation (default: `nomic-embed-t… }` | Configuration options for the Ollama HTTP client. | `src/client/config.rs` |
| `OllamaEmbedder` | `{ client: OllamaClient, model: String, concurrency: usize, expected_dimension: Option<usize>, observed_dimension: Arc<Mutex<Option<usize>>>, }` | Implementation of `TextEmbeddingEngine` using Ollama's HTTP API. | `src/embedding.rs` |
| `OllamaQueryRewriter` | `{ client: OllamaClient, config: OllamaQueryRewriterConfig, }` | LLM-basierter Multi-Signal Query Rewriter über Ollama. | `src/query_rewriter.rs` |
| `OllamaQueryRewriterConfig` | `{ /// Ollama-Modell für Rewriting (z. B. "llama3.2" oder "gemma2"). pub model: String, /// Maximale Anzahl von vorherigen Suchergebnissen als Kontext im Prompt.… }` | Konfiguration für den `OllamaQueryRewriter`. | `src/query_rewriter.rs` |

### Öffentliche Datentypen — Enums (1)

| Enum | Varianten | Beschreibung | Datei |
|---|---|---|---|
| `Confidence` | `/// Score was successfully parsed from the LLM output. Parsed, /// LLM output was unparseable; score defaulted to fallback value (0.5). Unparseable,` | Confidence or parse status of an LLM importance rating. | `src/importance.rs` |

### Typ-Aliase (1)

| Alias | Definition | Beschreibung |
|---|---|---|
| `ContextPrefixer` | `type ContextPrefixer= ContextPrefixEngine` | Compatibility alias for ContextPrefixEngine. |

### Konstanten (6)

| Konstante | Typ | Beschreibung |
|---|---|---|
| `DEFAULT_BASE_URL` | `&str` | Standard Ollama base URL |
| `DEFAULT_EMBED_MODEL` | `&str` | Standard embedding model for SME usage (multilingual support for German) |
| `DEFAULT_MAX_RESPONSE_BYTES` | `usize` | Default maximum response size limit in bytes (10 MB) |
| `MAX_BATCH_SIZE` | `usize` | Maximum allowed batch size for batch operations to prevent request timeouts and memory exhaustion. |
| `MAX_RETRIES` | `u32` | Default maximum retry attempts for transient errors |
| `MAX_TEXT_BYTES` | `usize` | Maximum allowed text length in bytes (10 MB) to prevent OOM or DoS |

### Öffentliche Funktionen (66)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn check(&self, clock: &dyn Clock) -> Result<(), String>` | Checks if the circuit breaker allows a request. Returns `Ok(())` if closed/half-open, or `Err(msg)` if open. | `src/client/circuit_breaker.rs` |
| `pub fn new(failure_threshold: u32, reset_timeout: Duration) -> Self` | — | `src/client/circuit_breaker.rs` |
| `pub fn record_failure(&self, clock: &dyn Clock)` | Records a failed request (transient/network/5xx). | `src/client/circuit_breaker.rs` |
| `pub fn record_success(&self)` | Records a successful request, resetting consecutive failure counts. | `src/client/circuit_breaker.rs` |
| `pub fn base_url(&self) -> &str` | — | `src/client/core.rs` |
| `pub async fn chat_with_rag_streaming( &self, model: &str, user_query: &str, context: &str, mut on_token: impl FnMut(String) + Send, ) -> Result<String>` | Streams RAG chat response token by token via POST /api/chat | `src/client/core.rs` |
| `pub async fn check_availability(&self) -> Result<()>` | Health check verifying Ollama availability via GET /api/tags | `src/client/core.rs` |
| `pub fn config(&self) -> &OllamaConfig` | — | `src/client/core.rs` |
| `pub async fn embed(&self, model: &str, text: &str) -> Result<Vec<f32>>` | Generates vector embedding with retry logic for transient failures. | `src/client/core.rs` |
| `pub async fn embed_batch(&self, model: &str, texts: &[&str]) -> Result<Vec<Vec<f32>>>` | Generates vector embeddings for multiple texts. Idempotent retry operation. | `src/client/core.rs` |
| `pub async fn ensure_model_available(&self, model: &str) -> Result<()>` | Ensures that the specified model exists; returns `ContextraError::NotFound` with helpful instruction if missing. | `src/client/core.rs` |
| `pub async fn generate(&self, model: &str, prompt: &str) -> Result<String>` | Generates non-streaming text completion via POST /api/generate with retry logic. | `src/client/core.rs` |
| `pub async fn generate_text(&self, model: &str, prompt: &str) -> Result<String>` | Idempotent text generation with retries for transient failures. | `src/client/core.rs` |
| `pub async fn generate_text_stream<F, Fut>( &self, model: &str, prompt: &str, mut on_token: F, ) -> Result<String> where F: FnMut(String) -> Fut + Send, Fut: std::future::Future<Output = bool> + Send,` | Streams text completion token by token via POST /api/chat. Non-idempotent (no automatic retries). | `src/client/core.rs` |
| `pub async fn is_available(&self) -> bool` | Health check verifying Ollama availability via GET /api/tags | `src/client/core.rs` |
| `pub async fn is_model_available(&self, model: &str) -> bool` | Verifies if a specific model is available in the Ollama instance. | `src/client/core.rs` |
| `pub async fn list_models(&self) -> Result<Vec<String>>` | List available models in Ollama via GET /api/tags | `src/client/core.rs` |
| `pub fn new(base_url: impl Into<String>) -> Self` | Creates a new `OllamaClient` with the specified base URL and default timeout config. Note: Intended for local development and testing. For… | `src/client/core.rs` |
| `pub async fn read_response_bytes(response: reqwest::Response, max_bytes: usize) -> Result<Vec<u8>>` | Helper to read HTTP response bytes while strictly enforcing `max_bytes` limit. | `src/client/core.rs` |
| `pub async fn read_response_text(response: reqwest::Response, max_bytes: usize) -> Result<String>` | Helper to read HTTP response text while strictly enforcing `max_bytes` limit. | `src/client/core.rs` |
| `pub async fn try_embed(&self, model: &str, text: &str) -> Result<Vec<f32>>` | Single embed attempt via POST /api/embeddings (no retry). | `src/client/core.rs` |
| `pub async fn try_embed_batch(&self, model: &str, texts: &[&str]) -> Result<Vec<Vec<f32>>>` | — | `src/client/core.rs` |
| `pub async fn try_generate(&self, model: &str, prompt: &str) -> Result<String>` | Single generate attempt via POST /api/generate (no retry). | `src/client/core.rs` |
| `pub async fn try_generate_text(&self, model: &str, prompt: &str) -> Result<String>` | Single generate_text attempt via POST /api/chat. | `src/client/core.rs` |
| `pub fn try_with_config( config: OllamaConfig, audit_sink: Arc<dyn EgressAuditSink>, ) -> Result<Self>` | Creates a fallible `OllamaClient` with custom configuration parameters and a mandatory audit sink. Returns an error immediately if building… | `src/client/core.rs` |
| `pub fn with_audit_sink(mut self, audit_sink: Arc<dyn EgressAuditSink>) -> Self` | Sets a custom egress audit sink. | `src/client/core.rs` |
| `pub fn with_clock(mut self, clock: Arc<dyn Clock>) -> Self` | Sets a custom clock port. | `src/client/core.rs` |
| `pub fn with_config(config: OllamaConfig) -> Self` | Creates a new `OllamaClient` with custom configuration parameters. Note: Intended for local development and testing. If building the underl… | `src/client/core.rs` |
| `pub fn with_defaults() -> Self` | — | `src/client/core.rs` |
| `pub fn with_rng(mut self, rng: Arc<dyn Rng>) -> Self` | Sets a custom random number generator for retry jitter computation. | `src/client/core.rs` |
| `pub fn classify_reqwest_error(e: reqwest::Error, base_url: &str, context: &str) -> ContextraError` | Classifies a `reqwest::Error` into structured `ContextraError` variants (`Io` for connection refused or timeouts, `Storage` otherwise). | `src/client/errors.rs` |
| `pub fn is_transient_error(e: &ContextraError) -> bool` | Helper to classify transient network errors for retry. Returns true only for transient network failures (I/O error, connection reset, timeo… | `src/client/errors.rs` |
| `pub fn is_transient_network_error(err: &reqwest::Error) -> bool` | Helper to check if a reqwest error is a transient network error (timeout or connection error). | `src/client/errors.rs` |
| `pub fn check_endpoint(&self, endpoint: &str, audit_sink: &dyn EgressAuditSink) -> Result<()>` | Checks whether the given endpoint URL is permitted under this policy. Loopback endpoints (`127.0.0.0/8`, `::1`, `localhost`) and permitted… | `src/client/policy.rs` |
| `pub fn loopback_only() -> Self` | Creates a default loopback-only policy. | `src/client/policy.rs` |
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
| **Interne Abhängigkeiten** | `contextra-ports`, `contextra-rank`, `contextra-types` |
| **Externe Abhängigkeiten (Auswahl)** | `parking_lot`, `serde`, `sha2`, `tokio`, `tracing` |

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
| `onnx` | `["dep:ort", "dep:tokenizers", "dep:ndarray", "dep:reqwest", "dep:sha2"]` |

### Öffentliche Datentypen — Structs (7)

| Struct | Felder / Form | Beschreibung | Datei |
|---|---|---|---|
| `CrossEncoderReranker` | `{ /// Aktive Konfiguration des Rerankers. pub(super) _config: RerankConfig, /// Internes Backend (Passthrough oder ONNX). backend: RerankerBackend, /// Online-Ka… }` | Einheitlicher, öffentlicher `CrossEncoderReranker`-Typ, unabhängig von aktiven Feature-Flags. Bietet eine thread-sichere Schnittstelle für die semant… | `src/reranker/cross_encoder.rs` |
| `ModelDownloadOptions` | `{ /// Allow downloading missing model files over network. Default: false. pub allow_download: bool, /// List of allowed hostnames for downloading models (e.g., `… }` | Sicherheitsrichtlinien und Konfigurationsparameter für das Herunterladen von ONNX-Modellartefakten. Gemäß Offline-First-Doktrin standardmäßig deaktiv… | `src/lib.rs` |
| `OnnxConfig` | `{ /// Die eingebettete Reranker-Konfiguration. pub inner: RerankConfig, }` | Eigenständige Produktionskonfiguration für den ONNX-Cross-Encoder-Reranker. Kapselt eine [`RerankConfig`]. | `src/reranker/config.rs` |
| `OnnxRerankerBuilder` | `{ config: RerankConfig, }` | Builder zur schrittweisen Konstruktion von `CrossEncoderReranker`-Instanzen. Erzwingt Offline-First-Betrieb und validiert vor dem Laden der ONNX-Sess… | `src/reranker/config.rs` |
| `RerankConfig` | `{ /// Dateipfad zur ONNX-Modelldatei (z. B. `models/bge-reranker-base.onnx`). pub model_path: PathBuf, /// Dateipfad zur Tokenizer-Konfiguration (z. B. `models/t… }` | Vollständige Konfiguration für das Cross-Encoder Reranking. Definiert Modellpfade, Thread-Einstellungen für die ONNX Runtime, Batchgrößen und Timeout… | `src/reranker/config.rs` |
| `TextEmbedder` | `{ /// Shared ONNX session (thread-safe via `Arc<Mutex>`). session: Arc<parking_lot::Mutex<ort::session::Session>>, /// Shared tokenizer instance (thread-safe via… }` | Führt Text-Tokenisierung und ONNX-Modell-Inferenz im Prozess aus. | `src/lib.rs` |
| `TextEmbedderConfig` | `{ /// Maximum number of tokens per text sequence (default: 512). pub max_sequence_length: usize, /// Maximum parallel ONNX inference threads (default: 1). /// Va… }` | Konfigurationsparameter für den [`TextEmbedder`]. | `src/lib.rs` |

### Typ-Aliase (1)

| Alias | Definition | Beschreibung |
|---|---|---|
| `OnnxEmbedder` | `type OnnxEmbedder= TextEmbedder` | Typalias für [`TextEmbedder`] zur Wahrung von Kompatibilität mit `contextra-mcp`. |

### Konstanten (2)

| Konstante | Typ | Beschreibung |
|---|---|---|
| `MAX_CANDIDATES` | `usize` | Maximale Anzahl von Kandidaten pro Reranking-Aufruf (10.000). Dient als Obergrenze zur Vermeidung unbegrenzter Speicherallokationen und CPU-Blockaden… |
| `MAX_EMBED_BATCH_SIZE` | `usize` | Standard-Obergrenze für Batch-Embedding-Anfragen (512 Texte). |

### Öffentliche Funktionen (39)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn default_model_cache_dir(model_name: &str) -> PathBuf` | Liefert das standardmäßige lokale Cache-Verzeichnis für ONNX-Embedding-Modelle. Standardpfad: `~/.contextra/models/<model_name>` # Ablauf 1… | `src/lib.rs` |
| `pub async fn embed_async(&self, text: &str) -> Result<Vec<f32>>` | Berechnet die Vektor-Einbettung (Embedding) für einen Text asynchron unter Einhaltung von Backpressure. | `src/lib.rs` |
| `pub async fn ensure_onnx_model_download( model_name: &str, cache_dir: Option<&Path>, ) -> Result<PathBuf>` | Stellt sicher, dass die ONNX-Modelldateien (`model.onnx` und `tokenizer.json`) lokal existieren. Nutzt die Standardoptionen von [`ModelDown… | `src/lib.rs` |
| `pub async fn ensure_onnx_model_download( _model_name: &str, _cache_dir: Option<&Path>, ) -> contextra_types::Result<PathBuf>` | Stellt sicher, dass die ONNX-Modelldateien (`model.onnx` und `tokenizer.json`) lokal existieren. Nutzt die Standardoptionen von [`ModelDown… | `src/lib.rs` |
| `pub async fn ensure_onnx_model_download_with_options( model_name: &str, cache_dir: Option<&Path>, options: ModelDownloadOptions, ) -> Result<PathBuf>` | Stellt sicher, dass die ONNX-Modelldateien lokal existieren, unter Verwendung benutzerdefinierter [`ModelDownloadOptions`]. # Ablauf 1. Zie… | `src/lib.rs` |
| `pub fn from_path(model_path: impl AsRef<Path>) -> Result<Self>` | Erstellt einen Embedder aus einer Modelldatei oder einem Modellverzeichnis. | `src/lib.rs` |
| `pub fn load(model_dir: impl AsRef<Path>) -> Result<Self>` | Lädt Modell und Tokenizer aus dem angegebenen Verzeichnis mit Standardkonfiguration. | `src/lib.rs` |
| `pub fn load_with_config( model_dir: impl AsRef<Path>, config: TextEmbedderConfig, ) -> Result<Self>` | Lädt Modell und Tokenizer mit einer benutzerdefinierten Konfiguration. | `src/lib.rs` |
| `pub fn new(model_dir: impl AsRef<Path>) -> Result<Self>` | Erstellt einen neuen Embedder aus einem Modellverzeichnis (Alias für [`TextEmbedder::load`]). | `src/lib.rs` |
| `pub fn session_load_count(&self) -> usize` | Gibt zurück, wie oft eine ONNX-Session geladen wurde. | `src/lib.rs` |
| `pub fn validate(&self) -> Result<()>` | Validiert die Konfigurationswerte auf Konsistenz und Grenzwerte. | `src/lib.rs` |
| `pub fn with_expected_dimension(mut self, dim: usize) -> Self` | Konfiguriert die erwartete Dimension des Ausgabeverktors zur Validierung nach Inferenz. | `src/lib.rs` |
| `pub fn batch_size(mut self, batch_size: usize) -> Self` | Konfiguriert die Batch-Größe für parallele Inferenz (Standard: 8). | `src/reranker/config.rs` |
| `pub fn build(self) -> Result<super::cross_encoder::CrossEncoderReranker, ContextraError>` | Erzeugt die finale [`super::cross_encoder::CrossEncoderReranker`]-Instanz nach Validierung. # Ablauf 1. `config.validate()` ausführen. 2. B… | `src/reranker/config.rs` |
| `pub fn calibration(mut self, calibration: PlattScaler) -> Self` | Übergibt ein kalibriertes `PlattScaler`-Modell für Wahrscheinlichkeits-Normalisierung. | `src/reranker/config.rs` |
| `pub fn calibration_warmup(mut self, warmup: usize) -> Self` | Konfiguriert die Warmup-Schwelle (Mindestanzahl Beobachtungen vor erstem Online-Fit). | `src/reranker/config.rs` |
| `pub fn inter_op_threads(mut self, threads: usize) -> Self` | Konfiguriert die Anzahl der Inter-Op-Threads in der ONNX Runtime. | `src/reranker/config.rs` |
| `pub fn intra_op_threads(mut self, threads: usize) -> Self` | Konfiguriert die Anzahl der Intra-Op-Threads in der ONNX Runtime. | `src/reranker/config.rs` |
| `pub fn max_sequence_length(mut self, max_length: usize) -> Self` | Konfiguriert die maximale Token-Länge für `(Query, Candidate)`-Paare (Standard: 512). | `src/reranker/config.rs` |
| `pub fn new(model_path: impl AsRef<Path>, tokenizer_path: impl AsRef<Path>) -> Self` | Erstellt eine `OnnxConfig` mit den erforderlichen Modell- und Tokenizerpfaden. Alle anderen Optionen werden mit Standardwerten initialisier… | `src/reranker/config.rs` |
| `pub fn new(model_path: impl AsRef<Path>, tokenizer_path: impl AsRef<Path>) -> Self` | Erstellt einen neuen Builder mit den erforderlichen Dateipfaden zu Modell und Tokenizer. | `src/reranker/config.rs` |
| `pub fn validate(&self) -> Result<(), ContextraError>` | Validiert die Konfigurationsparameter und prüft die Dateiexistenz vor der Initialisierung. # Validierungsprüfungen 1. `model_path` muss auf… | `src/reranker/config.rs` |
| `pub fn validate(&self) -> Result<(), ContextraError>` | Validiert die Konfigurationsparameter und prüft die Dateiexistenz (delegiert an `inner.validate()`). | `src/reranker/config.rs` |
| `pub fn with_calibration(mut self, calibration: PlattScaler) -> Self` | Konfiguriert ein vorab gefittetes [`PlattScaler`]-Modell für die Logit-Kalibrierung. | `src/reranker/config.rs` |
| `pub fn builder( model_path: impl AsRef<Path>, tokenizer_path: impl AsRef<Path>, ) -> OnnxRerankerBuilder` | Erstellt einen neuen Builder [`OnnxRerankerBuilder`]. | `src/reranker/cross_encoder.rs` |
| `pub fn calibrate(&self, raw_logit: f32) -> f32` | Appliziert die aktuell gefittete Kalibrierung auf einen rohen Logit-Score. | `src/reranker/cross_encoder.rs` |
| `pub fn calibration_observation_count(&self) -> usize` | Gibt die Gesamtanzahl der bisher gesammelten Kalibrierungs-Beobachtungen zurück. | `src/reranker/cross_encoder.rs` |
| `pub fn config(&self) -> &RerankConfig` | Liefert eine Referenz auf die aktive [`RerankConfig`]. | `src/reranker/cross_encoder.rs` |
| `pub fn fitted_calibration(&self) -> PlattScaler` | Gibt einen Klon des aktuell aktiven Kalibrierungsmodells zurück. | `src/reranker/cross_encoder.rs` |
| `pub fn from_config(config: OnnxConfig) -> Result<Self, ContextraError>` | Erstellt einen `CrossEncoderReranker` aus einer [`OnnxConfig`]. Führt vorab eine Validierung der Parameter und Dateipfade durch. | `src/reranker/cross_encoder.rs` |
| `pub fn invalidate_calibration(&self, new_fingerprint: ConfigFingerprint)` | Invalidiert die Kalibrierung bei einem Modellwechsel gemäß Invariante INV-CAL-2. Setzt den `PlattScaler` auf Identität zurück, leert den Pu… | `src/reranker/cross_encoder.rs` |
| `pub fn is_calibrated(&self) -> bool` | Gibt `true` zurück, wenn der `PlattScaler` gefittet wurde (Warmup abgeschlossen). | `src/reranker/cross_encoder.rs` |
| `pub fn new(config: RerankConfig) -> Result<Self, ContextraError>` | Erstellt einen neuen `CrossEncoderReranker`. Falls `feature = "onnx"` aktiv ist, wird das ONNX-Backend initialisiert. Andernfalls wird der… | `src/reranker/cross_encoder.rs` |
| `pub fn passthrough() -> Self` | Erstellt einen Passthrough-CrossEncoderReranker mit Standardkonfiguration (für Tests und Benchmarks). | `src/reranker/cross_encoder.rs` |
| `pub fn passthrough_with_config(config: RerankConfig) -> Self` | Erstellt einen Passthrough-CrossEncoderReranker mit einer benutzerdefinierten Konfiguration. | `src/reranker/cross_encoder.rs` |
| `pub fn record_implicit_feedback(&self, results: &[RerankResult], implicit_relevant_k: usize)` | Zeichnet implizites Benutzer-Feedback auf: Die Top `implicit_relevant_k` Dokumente werden als relevant (`true`) markiert, die restlichen Do… | `src/reranker/cross_encoder.rs` |
| `pub fn record_outcome(&self, raw_logit: f32, is_relevant: bool)` | Nimmt ein Feedback-Signal auf (Roh-Logit + Relevanz-Label). Triggert automatisch einen Re-Fit des PlattScalers, sobald die Warmup-Schwelle… | `src/reranker/cross_encoder.rs` |
| `pub async fn rerank( &self, _query: &str, candidates: &[String], ) -> Result<Vec<RerankResult>, ContextraError>` | Führt das Reranking einer Kandidatenliste für eine Suchabfrage aus. # Ablauf 1. Prüfen, ob `candidates.len() <= MAX_CANDIDATES` ist (DoS-Sc… | `src/reranker/cross_encoder.rs` |
| `pub fn with_calibration(mut self, calibration: PlattScaler) -> Self` | Setzt ein initial gefittetes `PlattScaler`-Modell für den Reranker. | `src/reranker/cross_encoder.rs` |


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

### Öffentliche Datentypen — Structs (10)

| Struct | Felder / Form | Beschreibung | Datei |
|---|---|---|---|
| `ApprovalRequest` | `{ /// Unique identifier for the approval request. pub request_id: String, /// Classification risk level. pub risk: ApprovalRisk, /// Human-readable summary of ca… }` | Human approval request state machine for a WASM execution run. | `src/approval.rs` |
| `CapabilityViolationError` | `{ pub capability: &'static str, }` | Custom error type returned by host functions on capability violation (INV-SBX-3). | `src/executor/state.rs` |
| `HostCallLimitExceededError` | `{ pub limit: u32, }` | Custom error type returned when host calls exceed maximum allowed limit. | `src/executor/state.rs` |
| `MergeOperatorCapabilities` | `(tuple/unit)` | Dedicated pure capabilities preset for I/O-free WASM Merge Operators (§4.18). | `src/capabilities.rs` |
| `OutputLimitExceededError` | `{ pub stream: &'static str, pub limit: usize, }` | Error thrown when stdout or stderr exceeds `max_output_bytes`. | `src/wasi/mod.rs` |
| `ProcessExitError` | `{ pub code: i32, }` | Error thrown on WASI `proc_exit`. | `src/wasi/mod.rs` |
| `WasmCapabilities` | `{ /// Stdout-Ausgabe erlaubt. Default: true. pub allow_stdout: bool, /// Stderr-Ausgabe erlaubt. Default: false (kein Logging-Leak). pub allow_stderr: bool, ///… }` | Whitelist für WASM-Guest-Capabilities. # Sicherheitsmodell Kein Dateisystem- und Netzwerkzugriff per Default. Monotone Uhr ist erlaubt (deterministis… | `src/capabilities.rs` |
| `WasmExecutor` | `{ engine: Engine, }` | WASM-Ausführungsgrenze für die `CodeExecution`-Permission. # Designprinzip (§4.18) - Frische `Store` + `Instance` pro execute()-Aufruf: kein Zustands… | `src/executor/mod.rs` |
| `WasmMergeFunction` | `{ executor: WasmExecutor, wasm_bytes: Arc<[u8]>, caps: WasmCapabilities, }` | Standalone, pure, deterministic WASM merge operator executor. Encapsulates a pre-validated WASM module and executes pure merge logic over binary inpu… | `src/merge.rs` |
| `WasmOutput` | `{ /// Stdout des WASM-Guests — sensitiv, ZeroizeOnDrop. #[zeroize(skip)] // Zeroizing<Vec<u8>> implements Drop zeroization itself pub stdout: Zeroizing<Vec<u8>>,… }` | Ausgabe einer WASM-Execution. # Sicherheits-Invariante (P9) `stdout` ist `ZeroizeOnDrop` — der Speicher wird beim Drop überschrieben. Sensitive WASM-… | `src/output.rs` |

### Öffentliche Datentypen — Enums (4)

| Enum | Varianten | Beschreibung | Datei |
|---|---|---|---|
| `ApprovalRisk` | `/// Low risk: Execution within standard safe defaults (no filesystem/network/cloud egress). Low, /// Elevated risk: File system access requested without network or egress. Elevated, /// High risk: Ne…` | Risk level classification for a requested WASM execution configuration. | `src/approval.rs` |
| `ApprovalStatus` | `/// Pending decision. Pending, /// Approved by an authorized entity. Approved { approved_by: String, approved_at_unix_ms: u64, }, /// Rejected with an explicit reason. Rejected { rejected_by: String,…` | Status of a human approval decision. | `src/approval.rs` |
| `ApprovalTransitionError` | `/// Transition rejected because the approval request has expired. #[error( "Approval request expired (created_at: {created_at_unix_ms}ms, ttl: {ttl_ms}ms, evaluated_at: {now_unix_ms}ms)" )] Expired {…` | Errors occurring during approval state transitions. | `src/approval.rs` |
| `SandboxError` | `#[error("WASM-Execution timed out after {timeout_ms}ms")] Timeout { timeout_ms: u64 }, #[error("WASM module exceeded memory limit ({pages} pages, max {max_pages})")] MemoryExceeded { pages: u32, max_…` | Fehlertypen der WASM-Ausführungsgrenze. | `src/error.rs` |

### Öffentliche Funktionen (19)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn approve( mut self, approved_by: String, now_unix_ms: u64, ) -> Result<Self, ApprovalTransitionError>` | Transitions request from `Pending` to `Approved`. Fails with `ApprovalTransitionError` if not in `Pending` state or if expired at `now_unix… | `src/approval.rs` |
| `pub fn check_and_expire(self, now_unix_ms: u64) -> Result<Self, ApprovalTransitionError>` | Checks if request is expired and transitions to `Expired` if pending. | `src/approval.rs` |
| `pub fn classify_risk(caps: &WasmCapabilities) -> ApprovalRisk` | Derives the execution risk level deterministically from `WasmCapabilities`. # Rule Matrix | Capability Condition | Derived Risk Level | Rat… | `src/approval.rs` |
| `pub fn expire(mut self, now_unix_ms: u64) -> Result<Self, ApprovalTransitionError>` | Explicitly transitions request from `Pending` to `Expired`. | `src/approval.rs` |
| `pub fn is_expired(&self, now_unix_ms: u64) -> bool` | Checks if the request is expired relative to `now_unix_ms`. Uses `saturating_add` to prevent overflow panic on boundary values. | `src/approval.rs` |
| `pub fn new( request_id: String, caps: &WasmCapabilities, created_at_unix_ms: u64, ttl_ms: u64, ) -> Self` | Creates a new approval request in `Pending` status. Computes `risk` via `classify_risk` and generates `requested_capabilities_summary` from… | `src/approval.rs` |
| `pub fn reject( mut self, rejected_by: String, reason: String, now_unix_ms: u64, ) -> Result<Self, ApprovalTransitionError>` | Transitions request from `Pending` to `Rejected`. Fails with `ApprovalTransitionError` if not in `Pending` state or if expired at `now_unix… | `src/approval.rs` |
| `pub fn requires_approval(&self) -> bool` | Determines whether this execution requires human approval before proceeding. # Product Rationale Low-risk executions (default sandboxed cap… | `src/approval.rs` |
| `pub fn pure() -> WasmCapabilities` | Returns a strict `WasmCapabilities` preset for pure merge operators. # Security Profile - `allow_stdout`: `false` - `allow_stderr`: `false`… | `src/capabilities.rs` |
| `pub fn pure_merge_operator() -> Self` | Returns a strict `WasmCapabilities` preset for pure merge operators. | `src/capabilities.rs` |
| `pub fn pure_with_result_channel() -> WasmCapabilities` | Returns a strict `WasmCapabilities` preset for merge operators with stdout enabled for returning output bytes. Differs from `pure()` strict… | `src/capabilities.rs` |
| `pub async fn execute( &self, wasm_bytes: &[u8], input: &[u8], capabilities: &WasmCapabilities, timeout: Duration, ) -> Result<WasmOutput, SandboxError>` | Führt ein WASM-Binary mit Capability-Einschränkungen aus. # Garantien - Fuel-Budget: CPU-Limit (deterministisch), verhindert Endlosschleife… | `src/executor/mod.rs` |
| `pub fn new() -> Result<Self, SandboxError>` | Erstellt einen neuen WasmExecutor. # Errors Wenn wasmtime::Engine nicht initialisiert werden kann. | `src/executor/mod.rs` |
| `pub fn validate_module( &self, wasm_bytes: &[u8], max_module_size_bytes: usize, ) -> Result<(), SandboxError>` | Validates a WASM binary without executing it. Checks binary size limits and attempts module compilation. | `src/executor/mod.rs` |
| `pub async fn merge(&self, existing: &[u8], new: &[u8]) -> Result<Vec<u8>, SandboxError>` | Merges two binary values (`existing` and `new`) deterministically using the WASM module. # Input Encoding Binary stream passed to guest std… | `src/merge.rs` |
| `pub fn new(wasm_bytes: impl Into<Arc<[u8]>>) -> Result<Self, SandboxError>` | Creates a new `WasmMergeFunction` and validates the WASM binary once upon creation. Uses default pure merge capabilities (`MergeOperatorCap… | `src/merge.rs` |
| `pub fn new_pure(wasm_bytes: impl Into<Arc<[u8]>>) -> Result<Self, SandboxError>` | Creates a new `WasmMergeFunction` with strict `pure()` capabilities (no stdout/stderr/FS/net/clock/PRNG). | `src/merge.rs` |
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
| **Interne Abhängigkeiten** | `contextra-adapt`, `contextra-checkpoint`, `contextra-crypto`, `contextra-graph`, `contextra-infer-onnx`, `contextra-mvcc`, `contextra-ports`, `contextra-rank`, `contextra-sandbox`, `contextra-store`, `contextra-sys`, `contextra-text`, `contextra-types`, `contextra-vector` |
| **Externe Abhängigkeiten (Auswahl)** | `ahash`, `bincode`, `blake3`, `bytes`, `hmac`, `parking_lot`, `serde`, `serde_json`, `sha2`, `thiserror`, `tokio`, `tokio-util`, `tracing`, `zeroize` |

### Architekturbeschreibung

# Modul: lib.rs  **Zweck:** Zentrale Facade und Orchestrierung der Contextra-Engine (Ring 2).  **Architektur:** Gehört zur Crate `contextra-engine` (Ring 2). Eingehaltene Invarianten: Fail-Closed bei Fehlern, kein ungeschützter globaler Zustand.

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
pub mod metrics_names;
pub mod temporal_filter;
pub mod transaction;
pub mod merge_operator_adapter;
pub use contextra_crypto::deletion_proof::{ DeletionLayer, DeletionProof, DeletionScope, ExcludedScope, };
pub use contextra_ports::TextEmbeddingEngine;
pub use contextra_ports::VectorDeleteMode;
pub use contextra_store::lsm::DurabilityMode;
pub use merge_operator_adapter::WasmMergeOperator;
pub use decay_controller::{AdaptiveDecayController, DecayControllerConfig, DecaySignalInputs};
pub use export::{ ExportCollectionV1, ExportDocumentV1, ExportMemoryV1, ExportRelationV1, SCHEMA_VERSION_V1, };
pub use import::ImportSummary;
pub use background_workers::start_decay_cleanup_worker;
pub use background_workers::{start_expiry_cleanup_worker, start_orphan_cleanup_worker};
pub use collection::crud::MAX_SCAN_RESULTS;
pub use collection::maintenance::PercolationConfig;
pub use collection::maintenance::PercolationResult;
pub use collection::query_builder::{HybridQueryBuilder, SearchStrategy, SignalWeights};
pub use collection::{Collection, CollectionConfig};
pub use contextra_checkpoint;
pub use contextra_text::Language;
pub use filter::MetadataFilter;
pub use fusion::{ProvenanceRecord, SearchResult, SignalContribution};
pub use contextra_ports::DriftStatusProvider;
pub use contextra_impl::erasure::{ summary_from_witness, ErasureFailure, ErasureReport, LayerSummary, ProofKeyProvider, UNMEASURED_ERASURE_LAYERS, };
pub use contextra_types::DistanceMetric;
pub use serde_json::json;
```

### Feature-Flags

| Flag | Aktiviert |
|---|---|
| `default` | `[]` |
| `onnx` | `["dep:contextra-infer-onnx", "contextra-infer-onnx/onnx"]` |
| `reranking` | `[]` |
| `sandbox` | `["dep:contextra-sandbox"]` |
| `experimental-diskann` | `["contextra-vector/experimental-diskann"]` |
| `background-maintenance` | `[]` |
| `graph-connectivity-health` | `["contextra-graph/graph-connectivity-health"]` |
| `edge-reinforcement-learning` | `[]` |
| `adaptive-candidate-pool-sizing` | `["dep:contextra-adapt"]` |
| `entity-extraction` | `[]` |
| `auto-extraction-opt-out` | `[]` |

### Schnittstellen — Public Traits (5)

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

**`ProofKeyProvider`** — *Injected provider interface for deletion proof signing keys.* (`src/contextra_impl/erasure.rs`)

```rust
pub trait ProofKeyProvider : Send + Sync {
/// Resolves the deletion proof key for the specified tenant.
    fn resolve_proof_key(&self, tenant_id: TenantId) -> Result<Vec<u8>>;
}
```

**`SandboxBridge`** — *Brücke zur sicheren Ausführung von sandboxed Operatoren (z.B. Wasm).* (`src/lib.rs`)

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

### Öffentliche Datentypen — Structs (42)

| Struct | Felder / Form | Beschreibung | Datei |
|---|---|---|---|
| `AdaptiveDecayController` | `{ config: DecayControllerConfig, }` | Adaptive Decay Controller für time-weighted Cache-Eviction. | `src/decay_controller.rs` |
| `AutoExtractionConfig` | `{ /// Laufzeit-Modus fuer die automatische Extraktion (Default: Enabled, bzw. Disabled wenn Feature `auto-extraction-opt-out` aktiv ist). #[serde(default)] pub m… }` | Konfiguration fuer die automatische Entitaetsextraktion beim Einfuegen von Dokumenten. | `src/collection/crud/auto_extraction.rs` |
| `CheckpointPinGuard` | `{ storage: &'a S, seq: u64, unpinned: bool, }` | RAII Guard for pinning snapshot checkpoints during search/scan operations. # Note on async Drop Rust does not support native `async Drop`. Therefore,… | `src/collection/search/checkpoint.rs` |
| `ChunkerConfig` | `{ /// Maximum tokens per chunk (soft limit, hard limit is this * 1.2). /// Default: 512 tokens (aligned with standard dense embedding models). pub max_tokens: us… }` | Configuration for the Markdown chunker. | `src/chunker/config.rs` |
| `Collection` | `{ pub(super) name: String, pub(super) prefix: Vec<u8>, pub(super) index: Arc<V>, pub(super) text_index: InvertedIndex<S>, pub(super) graph_index: Arc<CsrGraph>,… }` | # Concurrency & Lock Hierarchy Lock acquisition within `Collection` follows strict ordering to prevent deadlocks: 1. `Collection::kv_locks` (`KvKeyLo… | `src/collection/mod.rs` |
| `CollectionConfig` | `{ /// Configurable backpressure delay in milliseconds when system pressure is Critical. /// Default: Some(50) pub backpressure_delay_ms: Option<u64>, }` | Configuration for a collection. | `src/collection/mod.rs` |
| `CommitLedger` | `{ actions: Vec<Box<dyn CompensatingAction>>, }` | A ledger of compensating actions collected during 2PC, executed in reverse order (LIFO) on failure. | `src/transaction/compensating_actions.rs` |
| `CommunityDetectionConfig` | `{ pub auto_trigger_threshold: u64, }` | Konfiguration für Community-Erkennungs- und Graph-Clustering-Algorithmen. | `src/lib.rs` |
| `CompensateGraphAction` | `{ pub(super) collection: Collection<S, V>, pub(super) doc_ids: Arc<Vec<DocId>>, }` | Dokumentation für `CompensateGraphAction` im Rahmen der Transaktions- und Facade-Orchestrierung. | `src/transaction/compensating_actions.rs` |
| `CompensateHnswAction` | `{ pub(super) collection: Collection<S, V>, pub(super) tenant_id: TenantId, pub(super) doc_ids: Arc<Vec<DocId>>, }` | Dokumentation für `CompensateHnswAction` im Rahmen der Transaktions- und Facade-Orchestrierung. | `src/transaction/compensating_actions.rs` |
| `CompensateLsmAction` | `{ pub(super) collection: Collection<S, V>, pub(super) intent_key: Vec<u8>, pub(super) doc_ids: Arc<Vec<DocId>>, pub(super) f_keys: Vec<StagedKeyOp>, pub(super) r… }` | Dokumentation für `CompensateLsmAction` im Rahmen der Transaktions- und Facade-Orchestrierung. | `src/transaction/compensating_actions.rs` |
| `Contextra` | `{ storage: Arc<LsmStorage>, next_tx: Arc<AtomicU64>, dimension: usize, config: ContextraConfig, license_gate: Arc<dyn LicenseGate>, expiry_reaper_interval: std::… }` | Zentraler Orchestrator und Einstiegspunkt der Contextra-Engine. | `src/lib.rs` |
| `ContextraConfig` | `{ pub dimension: usize, pub max_elements: usize, pub distance_metric: contextra_types::DistanceMetric, pub encryption_passphrase: Option<String>, /// Maximum RAM… }` | Globale Konfiguration für die Contextra-Engine-Initialisierung. | `src/lib.rs` |
| `ContextraStats` | `{ pub drift_status: String, #[serde(default, skip_serializing_if = "Option::is_none")] pub calibration_ece: Option<f32>, #[serde(default, skip_serializing_if = "… }` | Aggregierte Betriebs- und Belegungsstatistiken der Engine. | `src/lib.rs` |
| `DbTransaction` | `{ pub tx_id: TxId, pub(super) collection: Collection<S, V>, pub(super) staged_forward_keys: Mutex<Vec<StagedKeyOp>>, pub(super) staged_reverse_keys: Mutex<Vec<St… }` | A transaction wrapper that ensures atomic multi-index commits across LSM-Store, HNSW-Index, Text-Index, and Graph-Index. | `src/transaction/db_transaction.rs` |
| `DecayControllerConfig` | `{ pub kappa: f32, pub base_half_life_tx: u64, pub eviction_threshold: f32, }` | Konfiguration für den Adaptive Decay Controller. | `src/decay_controller.rs` |
| `DecaySignalInputs` | `{ pub tombstone_ratio: f32, pub query_load_inverse: f32, pub w_tombstone: f32, pub w_query: f32, }` | Inputs für den Adaptive Decay Controller. | `src/decay_controller.rs` |
| `DeferredHyperedgeQueue` | `{ inner: Mutex<VecDeque<HyperEdgeId>>, }` | Thread-safe FIFO queue for storing hyperedges whose cascade invalidation was deferred to the background worker due to fan-out limits (H5 / AK-6). | `src/background_workers/hyperedge_worker.rs` |
| `Document` | `{ pub id: String, pub metadata: Option<Value>, }` | Repräsentiert ein Dokument mit eindeutiger ID, Text, Vektoreinbettung und Metadaten. | `src/lib.rs` |
| `EntityExtractionConfig` | `{ /// Ob Entitätsextraktion aktiviert ist pub enabled: bool, /// Maximale Anzahl von LLM-Aufrufen pro Zyklus / Extraktionsdurchlauf pub max_llm_calls_per_cycle:… }` | Konfiguration für die automatische Entitätsextraktion. | `src/extraction/types.rs` |
| `EntityExtractionConfig` | `{ pub enabled: bool, pub max_llm_calls_per_cycle: usize, pub min_confidence: f32, }` | Dokumentation für `EntityExtractionConfig` im Rahmen der Collection- und Suchabstraktion. | `src/collection/crud/auto_extraction.rs` |
| `ErasureFailure` | `{ /// Layer where verification or purging failed. pub layer: String, /// Detailed failure explanation. pub reason: String, }` | Detailed failure reason when erasure verification fails. | `src/contextra_impl/erasure.rs` |
| `ErasureReport` | `{ /// Gibt an, ob alle tatsächlich gemessenen Schichten verifizierbar ohne Restbestände bereinigt wurden. /// /// **Hinweis zur Verifikation (SPEC §9.4, §9.10):*… }` | Comprehensive erasure verification report. | `src/contextra_impl/erasure.rs` |
| `ExportCollectionV1` | `{ /// Name der Collection pub name: String, /// Liste aller Memory-Einträge pub memories: Vec<ExportMemoryV1>, /// Liste aller direkten Graph-Relationen pub rela… }` | Export-Repräsentation einer einzelnen Collection. | `src/export.rs` |
| `ExportDocumentV1` | `{ /// Pflichtfeld für Schema-Migrationen (MUSS "1.0" sein) pub schema_version: String, /// UTC-Zeitstempel der Export-Erstellung (ISO-8601 String) #[serde(defaul… }` | Wurzel-Dokument für den Export aller DB-Collections. | `src/export.rs` |
| `ExportMemoryV1` | `{ /// Eindeutiger Memory-Schlüssel / ID pub id: String, /// Kognitiver Gedächtnistyp (episodic, semantic, procedural, working) #[serde(rename = "type")] pub memo… }` | Export-Repräsentation eines einzelnen Memory-Eintrags. | `src/export.rs` |
| `ExportRelationV1` | `{ /// Quell-Memory ID pub from: String, /// Ziel-Memory ID pub to: String, /// Beziehungs-Label (z.B. "references", "depends_on") pub label: String, }` | Export-Repräsentation einer Graph-Beziehung. | `src/export.rs` |
| `ExtractedTriple` | `{ /// Subjekt der Relation / Entität pub subject: String, /// Prädikat / Relation pub predicate: String, /// Objekt der Relation / Entität pub object: String, //… }` | Extrahiertes Wissens-Tripel (Subjekt, Prädikat, Objekt). | `src/extraction/types.rs` |
| `HybridQueryBuilder` | `{ pub(super) collection: &'a Collection<S, V>, pub(super) text: Option<String>, pub(super) vector: Option<Vec<f32>>, pub(super) k: Option<usize>, pub(super) weig… }` | Fluent query builder for unifying vector, text, graph, and hybrid search operations. | `src/collection/query_builder/builder.rs` |
| `ImportSummary` | `{ /// Anzahl erfolgreich importierter oder aktualisierter Memory-Einträge pub imported_memories: usize, /// Anzahl übersprungener Memory-Einträge pub skipped_mem… }` | Zusammenfassung eines Import-Vorgangs. | `src/import.rs` |
| `LayerSummary` | `{ /// Layer name. pub layer: String, /// Applied scanning technique. pub technique: String, /// Number of units scanned. pub scanned: u64, /// Number of remainin… }` | Summary of a single storage layer's scan result. | `src/contextra_impl/erasure.rs` |
| `MarkdownChunker` | `{ config: ChunkerConfig, }` | A deterministic chunker for markdown files respecting structural blocks (headings, code blocks, tables, paragraphs) and UTF-8 safety. | `src/chunker/mod.rs` |
| `OpenIeExtractor` | `(tuple/unit)` | Implementierung des OpenIE-Extraktors hinter `EntityExtractor`. | `src/extraction/mod.rs` |
| `OrphanCleanupBackoffConfig` | `{ /// Initial cooldown duration after the first failed rebuild attempt. pub base_delay: Duration, /// Maximum cooldown cap between rebuild attempts. pub max_dela… }` | Configuration parameters for HNSW index rebuild backoff and failure escalation. | `src/background_workers/config.rs` |
| `PercolationResult` | `{ pub health: Option<f32>, pub rebonding_triggered: bool, pub new_edges_added: usize, }` | — | `src/collection/maintenance.rs` |
| `RuleBasedExtractor` | `(tuple/unit)` | Regelbasierter Entitäts- und Relationsextraktor. | `src/extraction/rule_based.rs` |
| `ScopeConstraint` | `{ /// Explicit set of allowed document IDs (BTreeSet guarantees deterministic ordering). pub allowed_doc_ids: BTreeSet<DocId>, /// Gamma augmentation factor for… }` | Scoping constraint defining an explicit hard-boundary set of allowed document IDs for predicate-agnostic vector search (ACORN-Hard-Boundary, Spec §5.… | `src/collection/query_builder/scope.rs` |
| `SignalWeights` | `{ /// Weight for vector similarity signal. pub vector: f32, /// Weight for BM25 text match signal. pub text: f32, /// Weight for graph traversal signal. pub grap… }` | Custom weights for vector, text, and graph signals in hybrid search. | `src/collection/query_builder/weights.rs` |
| `StoredDocument` | `{ pub id: String, pub embedding: Vec<f32>, pub metadata: Option<serde_json::Value>, }` | Dokumentation für `StoredDocument` im Rahmen der Collection- und Suchabstraktion. | `src/collection/mod.rs` |
| `StoredDocumentMeta` | `{ pub id: String, pub metadata: Option<serde_json::Value>, }` | Leichtgewichtige Metadaten (für doc_key, key_type=1) — KEIN Embedding. Wird für DocId-basierte Hydration nach HNSW/BM25-Suche verwendet. | `src/collection/mod.rs` |
| `ValidityWindow` | `{ /// Start of transaction validity (system time / MVCC). pub tx_valid_from: Option<TxId>, /// End of transaction validity (system time / MVCC, exclusive upper b… }` | Extracted bi-temporal validity window representation for a document or search candidate. | `src/temporal_filter.rs` |
| `WasmMergeOperator` | `{ tx: Option<Sender<MergeJob>>, worker_handle: Option<JoinHandle<()>>, recv_timeout: Duration, }` | Synchronous wrapper around [`WasmMergeFunction`] implementing [`MergeOperator`]. | `src/merge_operator_adapter.rs` |

### Öffentliche Datentypen — Enums (9)

| Enum | Varianten | Beschreibung | Datei |
|---|---|---|---|
| `CollectionStatus` | `/// Normal aktiv. #[default] Active, /// Zur Löschung Vormerkung/Fehlgeschlagen. PendingPurge,` | Status der Collection im System. | `src/contextra_impl/erasure.rs` |
| `CommitIntent` | `/// Transaction is in the "Prepared" state. Stored DocIds assist recovery. /// `stages_completed` tracks progress: /// 0 = Intent written, /// 1 = Storage (LSM) committed, /// 2 = Indices (Vector, Te…` | Status of a multi-index transaction during the 2-phase commit. | `src/transaction/intent.rs` |
| `EmbeddingBackend` | `Onnx { model_name: String, cache_dir: Option<std::path::PathBuf>, }, None,` | Wählbares Backend zur Berechnung von Text-Embeddings. | `src/lib.rs` |
| `ExtractorMode` | `#[default] OpenIe, RuleBased,` | Modus der Entitätsextraktion (LLM-basiertes OpenIE oder regelbasiert). | `src/extraction/types.rs` |
| `ExtractorMode` | `OpenIe, RuleBased,` | Dokumentation für `ExtractorMode` im Rahmen der Collection- und Suchabstraktion. | `src/collection/crud/auto_extraction.rs` |
| `FilterOp` | `/// Equal to Eq, /// Not equal to Ne, /// Greater than Gt, /// Greater than or equal to Gte, /// Less than Lt, /// Less than or equal to Lte, /// In a set of values In, /// Not in a set of values Not…` | Vergleichsoperationen für Metadaten-Prädikatfilter (z. B. Gleichheit, Größer, Kleiner, Inklusion). | `src/filter.rs` |
| `MetadataFilter` | `/// A single condition on a metadata field. Condition { field: String, #[allow(deprecated)] op: FilterOp, value: Value, }, /// Logical AND of multiple filters. And(Vec<MetadataFilter>), /// Logical O…` | — | `src/filter.rs` |
| `SearchStrategy` | `/// Reciprocal Rank Fusion (default fusion strategy per ADR-003). Rrf, /// Score-normalized fusion (CombSUM / Z-Score). ScoreNormalized, /// Multi-hop BFS graph traversal strategy. Hops { /// Maximum…` | Strategy for hybrid search and graph signal traversal. | `src/collection/query_builder/strategy.rs` |
| `TenantPolicy` | `/// Non-tenant collection access (`collection(...)`) is allowed (Default / Admin / Single-tenant path). #[default] Optional, /// Non-tenant collection access is forbidden; callers MUST use tenant-sco…` | Tenant isolation enforcement policy. | `src/lib.rs` |

### Typ-Aliase (4)

| Alias | Definition | Beschreibung |
|---|---|---|
| `ConsolidationLauncher` | `type ConsolidationLauncher= Arc< dyn Fn( Arc<Collection<LsmStorage>>, std::time::Duration, usize, tokio_util::sync::CancellationToken, ) -> tokio::task::JoinHandle<()> + Send + Sync, >` | Funktions-Typdefinition zum Starten von Wissenskonsolidierungs-Workern. |
| `DbStats` | `type DbStats= ContextraStats` | Alias für `ContextraStats` zur Abwärtskompatibilität. |
| `FusionResult` | `type FusionResult= crate::SearchResult` | Type alias for search & fusion results in post-retrieval pipelines. |
| `TenantCollectionMap` | `type TenantCollectionMap= ahash::AHashMap< (TenantId, String), Arc<Collection<contextra_store::tenant_codec::TenantScopedStorage<Arc<LsmStorage>>>>, >` | Typdefinition für die mandantenisolierte Collection-Verwaltung. |

### Konstanten (28)

| Konstante | Typ | Beschreibung |
|---|---|---|
| `BACKPRESSURE_LEVEL` | `&str` | Metric name for system backpressure level gauge readings (0.0 = Normal, 1.0 = Elevated, 2.0 = Critical). |
| `CHECKPOINT_DURATION` | `&str` | Metric name for checkpoint operation duration histogram observations. |
| `COMMIT_LATENCY` | `&str` | Metric name for engine commit latency histogram observations. |
| `DECAY_DELETION_THRESHOLD` | `f32` | werden als "vergessen" markiert und gelöscht. |
| `DEFAULT_COMMUNITY_BOOST` | `f32` | Standard community boost factor applied to post-fusion scores for matching community members. |
| `DEFAULT_EXCLUDED_SCOPES` | `&[ExcludedScope]` | Default excluded scopes for Contextra deletion proofs (§20.4). |
| `DEFAULT_MIN_RERANK_CANDIDATES` | `usize` | Performs hybrid search combining BM25, vector search, and graph traversal, followed by optional Cross-Encoder reranking. Mindest-Kandidatenpool für C… |
| `DEFAULT_RERANK_POOL_MAX` | `usize` | Default upper cap for pre-reranking candidate pool expansion (200 candidates). Prevents retrieval cost explosion for queries with large `k`. |
| `DEFAULT_RERANK_POOL_MULTIPLIER` | `usize` | Default candidate pool expansion multiplier for Cross-Encoder reranking (10x requested k). Grounded in empirical evaluation from T2-RAGBench (arXiv:2… |
| `DEFAULT_SCAN_LIMIT` | `usize` | Harte Obergrenze für scan()/scan_prefix()-Ergebnisse, falls kein explizites `limit` übergeben wird. Verhindert unbeabsichtigten Vollscan bei generisc… |
| `HARD_SCAN_CEILING` | `usize` | Harte Obergrenze für den maximal erlaubten `limit`-Parameter in scan() und scan_prefix(). Verhindert unbegrenzte Materialisierung im Speicher selbst… |
| `INSERT_LATENCY` | `&str` | Metric name for document insert operation duration histogram observations. |
| `MAX_DEFERRED_HYPEREDGES_PER_TICK` | `usize` | Maximum number of deferred hyperedges processed in a single worker tick. |
| `MAX_EXPIRED_PER_TICK` | `usize` | Maximum number of expired documents processed in a single expiry cleanup tick. |
| `MAX_EXTRACTED_TRIPLES` | `usize` | Maximale Anzahl extrahierter Tripel pro Durchlauf. |
| `MAX_EXTRACTION_TEXT_LENGTH` | `usize` | Maximale zulässige Eingabelänge in Zeichen für die regelbasierte Extraktion. |
| `MAX_ORPHANS_PER_TICK` | `usize` | Maximum number of orphan transactions processed in a single worker tick to avoid starving foreground operations. |
| `MAX_OVERFETCH_SCAN` | `usize` | Upper bound scan limit for adaptive candidate overfetching. |
| `MAX_SCAN_RESULTS` | `usize` | Alias for backwards compatibility with earlier MAX_SCAN_RESULTS references. |
| `OVERFETCH_FACTOR` | `usize` | Standard overfetch factor applied to candidate limits before fusion to balance OOM protection and recall. |
| `OVERFETCH_STAGE_MULTIPLIERS` | `[usize` | Named overfetch stage multipliers for adaptive candidate overfetching (Spec v17 §6.5). |
| `REAPER_DOCS_REAPED` | `&str` | Metric name for expired or decayed document reaper count observations. |
| `REBUILD_DURATION` | `&str` | Metric name for index rebuild duration histogram observations. |
| `SCHEMA_VERSION_V1` | `&str` | Aktuelle Schema-Version für das JSON Export-Format. |
| `SEARCH_LATENCY_GRAPH` | `&str` | Reserved metric name for graph signal search latency observations. |
| `SEARCH_LATENCY_TEXT` | `&str` | Reserved metric name for text signal search latency observations. |
| `SEARCH_LATENCY_VECTOR` | `&str` | Reserved metric name for vector signal search latency observations. |
| `UNMEASURED_ERASURE_LAYERS` | `&[&str]` | Schichten, die in `erase_document` und `erase_collection` derzeit NICHT gemessen werden (SPEC §9.4, §9.10). |

### Öffentliche Funktionen (296)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn calculate_rebuild_cooldown( consecutive_failures: u32, base_delay: Duration, max_delay: Duration, ) -> Duration` | Helper calculating exponential backoff cooldown given consecutive failures. | `src/background_workers/config.rs` |
| `pub fn start_decay_cleanup_worker<S: StorageEngine, V: VectorIndex>( collection: Arc<Collection<S, V>>, decay_config: DecayControllerConfig, interval: Duration, cancel_token: tokio_util::sync::CancellationToken, ) -> tokio::task::JoinHandle<()>` | Starts a background task for decay-controller-driven importance-score eviction. Nur aktiv wenn `background-maintenance` Feature-Flag gesetz… | `src/background_workers/expiry_workers.rs` |
| `pub fn start_expiry_cleanup_worker<S: StorageEngine>( collection: Arc<Collection<S>>, interval: Duration, cancel_token: tokio_util::sync::CancellationToken, ) -> tokio::task::JoinHandle<()>` | Starts a background task to periodically clean up expired documents with TTL. | `src/background_workers/expiry_workers.rs` |
| `pub fn start_expiry_reaper<S: StorageEngine>( collection: Arc<Collection<S>>, interval: Duration, cancel_token: tokio_util::sync::CancellationToken, ) -> tokio::task::JoinHandle<()>` | Deprecated legacy alias for `start_expiry_cleanup_worker`. | `src/background_workers/expiry_workers.rs` |
| `pub fn start_thermostat_reaper<S: StorageEngine, V: VectorIndex>( collection: Arc<Collection<S, V>>, decay_config: DecayControllerConfig, interval: Duration, cancel_token: tokio_util::sync::CancellationToken, ) -> tokio::task::JoinHandle<()>` | Deprecated legacy alias for `start_decay_cleanup_worker`. | `src/background_workers/expiry_workers.rs` |
| `pub async fn drain_up_to(&self, max: usize) -> Vec<HyperEdgeId>` | Drains up to `max` hyperedge IDs from the front of the queue (FIFO). | `src/background_workers/hyperedge_worker.rs` |
| `pub async fn enqueue(&self, ids: impl IntoIterator<Item = HyperEdgeId>)` | Enqueues hyperedge IDs for deferred background tombstoning. | `src/background_workers/hyperedge_worker.rs` |
| `pub fn new() -> Self` | Creates a new empty `DeferredHyperedgeQueue`. | `src/background_workers/hyperedge_worker.rs` |
| `pub fn start_hyperedge_cascade_deferred_worker<S: StorageEngine>( collection: Arc<Collection<S>>, queue: Arc<DeferredHyperedgeQueue>, interval: Duration, cancel_token: tokio_util::sync::CancellationToken, ) -> tokio::task::JoinHandle<()>` | Starts a background task to process deferred hyperedge tombstones. | `src/background_workers/hyperedge_worker.rs` |
| `pub fn start_orphan_cleanup_worker<T: Clone + Send + Sync + 'static>( buffer: Arc<TxBuffer<T>>, hnsw_index: Arc<contextra_vector::hnsw::HnswIndex>, interval: Duration, cancel_token: tokio_util::sync::CancellationToken, ) -> tokio::task::JoinHandle<()>` | Starts a background task to periodically clean up orphan transactions. This worker handles the cleanup of transactions that have exceeded t… | `src/background_workers/orphan_workers.rs` |
| `pub fn start_orphan_cleanup_worker_with_config< T: Clone + Send + Sync + 'static, I: OrphanCleanupIndex + 'static, >( buffer: Arc<TxBuffer<T>>, hnsw_index: Arc<I>, interval: Duration, backoff_config: OrphanCleanupBackoffConfig, cancel_token: tokio_util::sync:…` | Starts a background task to periodically clean up orphan transactions and manage HNSW rebuilds with exponential backoff. | `src/background_workers/orphan_workers.rs` |
| `pub fn start_orphan_reaper<T: Clone + Send + Sync + 'static>( buffer: Arc<TxBuffer<T>>, hnsw_index: Arc<contextra_vector::hnsw::HnswIndex>, interval: Duration, cancel_token: tokio_util::sync::CancellationToken, ) -> tokio::task::JoinHandle<()>` | Deprecated legacy alias for `start_orphan_cleanup_worker`. | `src/background_workers/orphan_workers.rs` |
| `pub fn estimate_tokens(text: &str) -> usize` | Schätzt die Token-Anzahl eines Textes mit heuristischen Regeln. | `src/chunker/estimate.rs` |
| `pub fn chunk(&self, doc_id: DocId, markdown: &str) -> Vec<ContextChunk>` | Chunks a Markdown document into semantically coherent pieces. | `src/chunker/mod.rs` |
| `pub fn new(config: ChunkerConfig) -> Self` | Creates a new MarkdownChunker with the given configuration. | `src/chunker/mod.rs` |
| `pub fn with_defaults() -> Self` | Creates a new MarkdownChunker with default configuration. | `src/chunker/mod.rs` |
| `pub fn chunk_text(text: &str, chunk_size: usize) -> Vec<&str>` | Segmentiert Text in Abschnitte bis zur Maximallänge ohne Überlappung. | `src/chunker/windows.rs` |
| `pub fn chunk_text_with_overlap<'a>( text: &'a str, window_chars: usize, overlap_chars: usize, ) -> Vec<&'a str>` | Erzeugt überlappende Text-Chunks mit definierter Fenstergröße und Überlappung. | `src/chunker/windows.rs` |
| `pub fn auto_extraction_config( &self, ) -> Option<(Arc<dyn LlmTextGenerator>, AutoExtractionConfig)>` | Returns the configured auto extraction generator and config, if present. | `src/collection/crud/auto_extraction.rs` |
| `pub fn auto_extraction_config_with_extractor( &self, ) -> Option<( Arc<dyn LlmTextGenerator>, AutoExtractionConfig, ExtractorMode, )>` | Returns the configured auto extraction generator, config, and extractor mode. | `src/collection/crud/auto_extraction.rs` |
| `pub fn is_enabled(&self) -> bool` | Prueft, ob die automatische Extraktion unter Beruecksichtigung aller Abschaltwege aktiv ist. Precedence / Abschaltkette: 1. Compile-Zeit fe… | `src/collection/crud/auto_extraction.rs` |
| `pub fn new(mode: AutoExtractionMode) -> Self` | Erstellt eine neue `AutoExtractionConfig` mit dem angegebenen Modus. | `src/collection/crud/auto_extraction.rs` |
| `pub fn recommended_mode_for_regulated(is_regulated: bool) -> AutoExtractionMode` | Liefert den empfohlenen `AutoExtractionMode` fuer das gegebene "regulated"-Flag. Gemäss v17 Teil 10.5 soll in EnterpriseRegulated / Regulie… | `src/collection/crud/auto_extraction.rs` |
| `pub fn set_auto_extraction( &self, generator: Arc<dyn LlmTextGenerator>, config: AutoExtractionConfig, )` | Sets or updates the auto extraction generator and config for this collection. | `src/collection/crud/auto_extraction.rs` |
| `pub fn set_auto_extraction_config(&self, config: AutoExtractionConfig)` | Sets or updates auto extraction without an external LLM generator (suitable for RuleBased mode). | `src/collection/crud/auto_extraction.rs` |
| `pub fn set_auto_extraction_with_mode( &self, generator: Arc<dyn LlmTextGenerator>, config: AutoExtractionConfig, extractor_mode: ExtractorMode, )` | Sets or updates auto extraction with an explicit extractor mode. | `src/collection/crud/auto_extraction.rs` |
| `pub fn with_auto_extraction( self, generator: Arc<dyn LlmTextGenerator>, config: AutoExtractionConfig, ) -> Self` | Configures auto extraction with an LLM generator and config (builder pattern). | `src/collection/crud/auto_extraction.rs` |
| `pub fn with_mode(mut self, mode: AutoExtractionMode) -> Self` | Setzt den Laufzeit-Modus (Builder Pattern). | `src/collection/crud/auto_extraction.rs` |
| `pub async fn delete(&self, id: &str) -> Result<()>` | Deletes a document from the collection by its ID. | `src/collection/crud/delete.rs` |
| `pub async fn delete_op( &self, db_tx: &mut crate::transaction::DbTransaction<S, V>, id: &str, ) -> Result<()>` | Dokumentation für `delete_op` im Rahmen der Collection- und Suchabstraktion. | `src/collection/crud/delete.rs` |
| `pub async fn insert( &self, id: &str, embedding: &[f32], metadata: Option<serde_json::Value>, ) -> Result<()>` | Inserts a document with an embedding and optional metadata. | `src/collection/crud/insert.rs` |
| `pub async fn insert_many( &self, docs: &[(String, Vec<f32>, Option<serde_json::Value>)], ) -> Result<()>` | Inserts multiple documents in a single atomic transaction under key-granular locks. | `src/collection/crud/insert.rs` |
| `pub async fn insert_op( &self, db_tx: &crate::transaction::DbTransaction<S, V>, id: &str, embedding: &[f32], metadata: Option<serde_json::Value>, ) -> Result<()>` | Dokumentation für `insert_op` im Rahmen der Collection- und Suchabstraktion. | `src/collection/crud/insert.rs` |
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
| `pub async fn update_op( &self, db_tx: &crate::transaction::DbTransaction<S, V>, id: &str, embedding: &[f32], metadata: Option<serde_json::Value>, ) -> Result<()>` | Dokumentation für `update_op` im Rahmen der Collection- und Suchabstraktion. | `src/collection/crud/update.rs` |
| `pub async fn lock_for<'a>(&'a self, key: &str) -> KeyGuard<'a>` | Dokumentation für `lock_for` im Rahmen der Collection- und Suchabstraktion. | `src/collection/kv_lock.rs` |
| `pub async fn lock_many_sorted<'a>( &'a self, keys: impl IntoIterator<Item = &'a str>, ) -> Vec<KeyGuard<'a>>` | Locks multiple shards in ascending shard index order to prevent deadlocks across operations. Deduplicates by lock shard index to prevent se… | `src/collection/kv_lock.rs` |
| `pub async fn lock_shard<'a>(&'a self, idx: usize) -> KeyGuard<'a>` | Dokumentation für `lock_shard` im Rahmen der Collection- und Suchabstraktion. | `src/collection/kv_lock.rs` |
| `pub fn new() -> Self` | Dokumentation für `new` im Rahmen der Collection- und Suchabstraktion. | `src/collection/kv_lock.rs` |
| `pub fn shard_idx(&self, key: &str) -> usize` | Dokumentation für `shard_idx` im Rahmen der Collection- und Suchabstraktion. | `src/collection/kv_lock.rs` |
| `pub fn sorted_unique_shards<'a>( keys: impl IntoIterator<Item = &'a str>, shard_count: usize, hasher: &ahash::RandomState, ) -> Vec<usize>` | Pure, synchronous function to map string keys to unique shard indices in ascending order. Deduplicates shard indices to prevent self-deadlo… | `src/collection/kv_lock.rs` |
| `pub fn try_lock_for<'a>(&'a self, key: &str) -> Option<KeyGuard<'a>>` | Dokumentation für `try_lock_for` im Rahmen der Collection- und Suchabstraktion. | `src/collection/kv_lock.rs` |
| `pub fn try_lock_shard<'a>(&'a self, idx: usize) -> Option<KeyGuard<'a>>` | Dokumentation für `try_lock_shard` im Rahmen der Collection- und Suchabstraktion. | `src/collection/kv_lock.rs` |
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
| `pub fn dimension(&self) -> usize` | Returns the vector dimension for this collection. | `src/collection/mod.rs` |
| `pub fn ensure_importance_metadata( metadata: &mut Option<serde_json::Value>, tx: TxId, text_opt: Option<&str>, )` | Ensures document metadata contains a valid `MemoryImportance` JSON payload. | `src/collection/mod.rs` |
| `pub fn extract_effective_importance(metadata: &Option<serde_json::Value>, now_tx: TxId) -> f32` | Extracts the effective importance score of a document at a given transaction ID. | `src/collection/mod.rs` |
| `pub fn graph_index(&self) -> Arc<CsrGraph>` | Returns the CSR graph index for this collection. | `src/collection/mod.rs` |
| `pub async fn is_empty(&self) -> bool` | Returns true if the collection is empty. | `src/collection/mod.rs` |
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
| `pub fn set_clock(&self, clock: Arc<dyn Clock>)` | Sets or overrides the clock implementation. | `src/collection/mod.rs` |
| `pub fn set_community_detection_trigger_threshold(&self, threshold: u64)` | Sets the mutation threshold for auto-triggered community detection (0 = disabled). | `src/collection/mod.rs` |
| `pub fn set_config(&self, config: CollectionConfig)` | Sets or updates the collection configuration. | `src/collection/mod.rs` |
| `pub async fn set_embedder(&self, embedder: Arc<dyn TextEmbeddingEngine>) -> Result<()>` | Configures the text embedder for this collection. | `src/collection/mod.rs` |
| `pub fn set_metrics_sink(&self, metrics: Arc<dyn MetricsSink>)` | Sets or overrides the metrics sink implementation. | `src/collection/mod.rs` |
| `pub fn set_pressure_receiver( &self, pressure_rx: tokio::sync::watch::Receiver<contextra_store::SystemPressure>, )` | Sets or overrides the pressure receiver channel. | `src/collection/mod.rs` |
| `pub fn storage(&self) -> &Arc<S>` | Returns a reference to the underlying storage engine. | `src/collection/mod.rs` |
| `pub fn user_key_prefix(&self) -> Vec<u8>` | Returns the namespaced prefix for user document keys in this collection. | `src/collection/mod.rs` |
| `pub fn vector_index(&self) -> &Arc<V>` | Returns a reference to the vector index of this collection. | `src/collection/mod.rs` |
| `pub fn with_clock(self, clock: Arc<dyn Clock>) -> Self` | Sets or overrides the clock implementation (builder version). | `src/collection/mod.rs` |
| `pub fn with_community_detection_trigger_threshold(self, threshold: u64) -> Self` | Builder method to set the mutation threshold for auto-triggered community detection (0 = disabled). | `src/collection/mod.rs` |
| `pub fn with_config(self, config: CollectionConfig) -> Self` | Sets or updates the collection configuration. | `src/collection/mod.rs` |
| `pub fn with_embedder(self, embedder: Arc<dyn TextEmbeddingEngine>) -> Self` | Sets the text embedder for this collection (consuming version). | `src/collection/mod.rs` |
| `pub fn with_hnsw( name: String, storage: Arc<S>, index: Arc<HnswIndex>, graph_index: Arc<CsrGraph>, next_tx: Arc<AtomicU64>, dimension: usize, language: Language, ) -> Self` | Convenience constructor for creating a `Collection` with `HnswIndex`. | `src/collection/mod.rs` |
| `pub fn with_metrics_sink(self, metrics: Arc<dyn MetricsSink>) -> Self` | Sets or overrides the metrics sink implementation (builder version). | `src/collection/mod.rs` |
| `pub fn with_pressure_receiver( self, pressure_rx: tokio::sync::watch::Receiver<contextra_store::SystemPressure>, ) -> Self` | Sets or overrides the pressure receiver channel. | `src/collection/mod.rs` |
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
| `pub async fn hybrid_search_reranked( &self, text: &str, vector: &[f32], k: usize, reranker: Option<&dyn Reranker>, anchor_entities: Option<&[contextra_types::EntityId]>, ) -> Result<Vec<crate::SearchResult>>` | Dokumentation für `hybrid_search_reranked` im Rahmen der Collection- und Suchabstraktion. | `src/collection/search/hybrid/mod.rs` |
| `pub async fn hybrid_search_with_strategy( &self, text: &str, vector: &[f32], k: usize, anchor_entities: Option<&[contextra_types::EntityId]>, weights: Option<&contextra_types::FusionWeights>, strategy: Option<&contextra_types::GraphTraversalStrategy>, same_co…` | Performs hybrid search with custom signal fusion weights and graph traversal strategy. | `src/collection/search/hybrid/mod.rs` |
| `pub async fn hybrid_search_with_weights( &self, text: &str, vector: &[f32], k: usize, anchor_entities: Option<&[contextra_types::EntityId]>, weights: Option<&contextra_types::FusionWeights>, ) -> Result<Vec<crate::SearchResult>>` | Performs hybrid search with custom fusion weights for vector, text, and graph signals, and optional community filtering/boosting. | `src/collection/search/hybrid/mod.rs` |
| `pub async fn traverse_links( &self, start: DocId, max_depth: usize, ) -> Result<Vec<(DocId, usize)>>` | Traverses Zettelkasten memory links starting from a given document up to a maximum hop depth (ADR-038). Performs iterative BFS with cycle d… | `src/collection/search/hybrid/mod.rs` |
| `pub async fn hybrid_search_with_query( &self, query: &contextra_types::HybridQuery, ) -> Result<Vec<crate::SearchResult>>` | Performs hybrid search combining BM25, vector, and graph signals configured via `HybridQuery`. Applies `memory_type_filter` and metadata `F… | `src/collection/search/hybrid/query.rs` |
| `pub async fn hybrid_search_with_query_and_report_at( &self, query: &contextra_types::HybridQuery, seq: u64, _on_signal_failure: SignalFailurePolicy, metrics: Option<&dyn MetricsSink>, ) -> Result<(Vec<crate::SearchResult>, SearchReport)>` | Performs hybrid search returning both search results and an execution `SearchReport`. | `src/collection/search/hybrid/query.rs` |
| `pub async fn hybrid_search_with_query_at( &self, query: &contextra_types::HybridQuery, seq: u64, ) -> Result<Vec<crate::SearchResult>>` | Performs hybrid search combining BM25, vector, and graph signals configured via `HybridQuery` at a specific snapshot sequence. | `src/collection/search/hybrid/query.rs` |
| `pub fn allocate_tx(&self) -> Result<TxId>` | Allokiert eine eindeutige, atomar inkrementierte Transaction-ID. Externe Crates verwenden diese Methode statt eigener TxId-Generierung. Ver… | `src/collection/tx.rs` |
| `pub fn begin_transaction(&self) -> Result<crate::transaction::DbTransaction<S, V>>` | Begins a new atomic transaction for this collection. | `src/collection/tx.rs` |
| `pub fn next_tx(&self) -> Result<TxId>` | — | `src/collection/tx.rs` |
| `pub fn allocate_tx(&self) -> Result<TxId>` | Dokumentation für `allocate_tx` im Rahmen der Transaktions- und Facade-Orchestrierung. | `src/contextra_impl/collections.rs` |
| `pub async fn collection(&self, name: &str) -> Result<Arc<Collection<LsmStorage>>>` | Retrieves or creates an un-tenanted [`Collection`] handle (Admin / Single-Tenant path). # Errors Returns [`ContextraError::InvalidInput`] i… | `src/contextra_impl/collections.rs` |
| `pub async fn collection_for_tenant( &self, name: &str, tenant_id: TenantId, ) -> Result<Arc<Collection<contextra_store::tenant_codec::TenantScopedStorage<Arc<LsmStorage>>>>>` | Liefert oder initialisiert eine Collection für einen Mandanten. | `src/contextra_impl/collections.rs` |
| `pub async fn collection_with_language( &self, name: &str, language: Language, ) -> Result<Arc<Collection<LsmStorage>>>` | Retrieves or creates an un-tenanted [`Collection`] handle with specified language (Admin / Single-Tenant path). # Errors Returns [`Contextr… | `src/contextra_impl/collections.rs` |
| `pub async fn collection_with_language_and_tenant( &self, name: &str, language: Language, tenant_id: TenantId, ) -> Result<Arc<Collection<contextra_store::tenant_codec::TenantScopedStorage<Arc<LsmStorage>>>>>` | Öffnet oder erstellt eine Collection mit Sprachkonfiguration für einen Mandanten. | `src/contextra_impl/collections.rs` |
| `pub async fn drop_collection( &self, name: &str, tenant_id: TenantId, proof_key: &[u8], ) -> Result<DeletionProof>` | Dokumentation für `drop_collection` im Rahmen der Transaktions- und Facade-Orchestrierung. | `src/contextra_impl/collections.rs` |
| `pub async fn list_collections(&self) -> Result<Vec<String>>` | Dokumentation für `list_collections` im Rahmen der Transaktions- und Facade-Orchestrierung. | `src/contextra_impl/collections.rs` |
| `pub async fn list_collections_for_tenant(&self, tenant_id: TenantId) -> Result<Vec<String>>` | Dokumentation für `list_collections_for_tenant` im Rahmen der Transaktions- und Facade-Orchestrierung. | `src/contextra_impl/collections.rs` |
| `pub async fn create_snapshot(&self) -> Result<u64>` | Dokumentation für `create_snapshot` im Rahmen der Transaktions- und Facade-Orchestrierung. | `src/contextra_impl/crud.rs` |
| `pub async fn delete(&self, id: &str) -> Result<()>` | Dokumentation für `delete` im Rahmen der Transaktions- und Facade-Orchestrierung. | `src/contextra_impl/crud.rs` |
| `pub async fn get(&self, id: &str) -> Result<Option<Document>>` | Dokumentation für `get` im Rahmen der Transaktions- und Facade-Orchestrierung. | `src/contextra_impl/crud.rs` |
| `pub async fn get_at_snapshot(&self, id: &str, seq_no: u64) -> Result<Option<Document>>` | Dokumentation für `get_at_snapshot` im Rahmen der Transaktions- und Facade-Orchestrierung. | `src/contextra_impl/crud.rs` |
| `pub async fn get_kv(&self, id: &str) -> Result<Option<Value>>` | Dokumentation für `get_kv` im Rahmen der Transaktions- und Facade-Orchestrierung. | `src/contextra_impl/crud.rs` |
| `pub async fn hybrid_search( &self, text: &str, vector: &[f32], k: usize, anchor_entities: Option<&[contextra_types::EntityId]>, ) -> Result<Vec<SearchResult>>` | Dokumentation für `hybrid_search` im Rahmen der Transaktions- und Facade-Orchestrierung. | `src/contextra_impl/crud.rs` |
| `pub async fn hybrid_search_reranked( &self, text: &str, vector: &[f32], k: usize, reranker: Option<&dyn contextra_ports::Reranker>, anchor_entities: Option<&[contextra_types::EntityId]>, ) -> Result<Vec<SearchResult>>` | Dokumentation für `hybrid_search_reranked` im Rahmen der Transaktions- und Facade-Orchestrierung. | `src/contextra_impl/crud.rs` |
| `pub async fn hybrid_search_with_query( &self, query: &contextra_types::HybridQuery, ) -> Result<Vec<SearchResult>>` | Dokumentation für `hybrid_search_with_query` im Rahmen der Transaktions- und Facade-Orchestrierung. | `src/contextra_impl/crud.rs` |
| `pub async fn hybrid_search_with_strategy( &self, text: &str, vector: &[f32], k: usize, anchor_entities: Option<&[contextra_types::EntityId]>, weights: Option<&contextra_types::FusionWeights>, strategy: Option<&contextra_types::GraphTraversalStrategy>, ) -> Re…` | Dokumentation für `hybrid_search_with_strategy` im Rahmen der Transaktions- und Facade-Orchestrierung. | `src/contextra_impl/crud.rs` |
| `pub async fn hybrid_search_with_weights( &self, text: &str, vector: &[f32], k: usize, anchor_entities: Option<&[contextra_types::EntityId]>, weights: Option<&contextra_types::FusionWeights>, ) -> Result<Vec<SearchResult>>` | Dokumentation für `hybrid_search_with_weights` im Rahmen der Transaktions- und Facade-Orchestrierung. | `src/contextra_impl/crud.rs` |
| `pub async fn insert(&self, id: &str, embedding: &[f32], metadata: Option<Value>) -> Result<()>` | Dokumentation für `insert` im Rahmen der Transaktions- und Facade-Orchestrierung. | `src/contextra_impl/crud.rs` |
| `pub async fn insert_many(&self, docs: &[(String, Vec<f32>, Option<Value>)]) -> Result<()>` | Dokumentation für `insert_many` im Rahmen der Transaktions- und Facade-Orchestrierung. | `src/contextra_impl/crud.rs` |
| `pub async fn insert_text_only( &self, id: &str, text: &str, metadata: Option<Value>, ) -> Result<()>` | Dokumentation für `insert_text_only` im Rahmen der Transaktions- und Facade-Orchestrierung. | `src/contextra_impl/crud.rs` |
| `pub async fn insert_typed( &self, collection_name: &str, id: &str, embedding: &[f32], memory_type: contextra_types::MemoryType, metadata: Option<Value>, ) -> Result<()>` | Dokumentation für `insert_typed` im Rahmen der Transaktions- und Facade-Orchestrierung. | `src/contextra_impl/crud.rs` |
| `pub async fn is_empty(&self) -> Result<bool>` | Dokumentation für `is_empty` im Rahmen der Transaktions- und Facade-Orchestrierung. | `src/contextra_impl/crud.rs` |
| `pub async fn last_committed_seq(&self) -> Result<u64>` | Dokumentation für `last_committed_seq` im Rahmen der Transaktions- und Facade-Orchestrierung. | `src/contextra_impl/crud.rs` |
| `pub async fn len(&self) -> Result<usize>` | Dokumentation für `len` im Rahmen der Transaktions- und Facade-Orchestrierung. | `src/contextra_impl/crud.rs` |
| `pub async fn put_kv(&self, id: &str, value: &Value) -> Result<()>` | Dokumentation für `put_kv` im Rahmen der Transaktions- und Facade-Orchestrierung. | `src/contextra_impl/crud.rs` |
| `pub async fn relate(&self, from: &str, to: &str, label: &str) -> Result<()>` | Dokumentation für `relate` im Rahmen der Transaktions- und Facade-Orchestrierung. | `src/contextra_impl/crud.rs` |
| `pub async fn relate_bidirectional(&self, from: &str, to: &str, label: &str) -> Result<()>` | Dokumentation für `relate_bidirectional` im Rahmen der Transaktions- und Facade-Orchestrierung. | `src/contextra_impl/crud.rs` |
| `pub async fn scan( &self, start: std::ops::Bound<&[u8]>, end: std::ops::Bound<&[u8]>, limit: Option<usize>, ) -> Result<Vec<(String, Value)>>` | Dokumentation für `scan` im Rahmen der Transaktions- und Facade-Orchestrierung. | `src/contextra_impl/crud.rs` |
| `pub async fn scan_prefix( &self, prefix: &str, limit: Option<usize>, ) -> Result<Vec<(String, Value)>>` | Dokumentation für `scan_prefix` im Rahmen der Transaktions- und Facade-Orchestrierung. | `src/contextra_impl/crud.rs` |
| `pub async fn search(&self, query: &[f32], k: usize) -> Result<Vec<SearchResult>>` | Dokumentation für `search` im Rahmen der Transaktions- und Facade-Orchestrierung. | `src/contextra_impl/crud.rs` |
| `pub async fn search_filtered( &self, query: &[f32], k: usize, filter: Option<&(dyn Fn(DocId) -> bool + Send + Sync)>, ) -> Result<Vec<SearchResult>>` | Dokumentation für `search_filtered` im Rahmen der Transaktions- und Facade-Orchestrierung. | `src/contextra_impl/crud.rs` |
| `pub async fn search_text(&self, text: &str, k: usize) -> Result<Vec<SearchResult>>` | Dokumentation für `search_text` im Rahmen der Transaktions- und Facade-Orchestrierung. | `src/contextra_impl/crud.rs` |
| `pub async fn search_with_filter( &self, query: &[f32], k: usize, filter: Option<MetadataFilter>, ) -> Result<Vec<SearchResult>>` | Dokumentation für `search_with_filter` im Rahmen der Transaktions- und Facade-Orchestrierung. | `src/contextra_impl/crud.rs` |
| `pub async fn search_with_filter_expr( &self, query: &[f32], k: usize, filter: Option<FilterExpr>, ) -> Result<Vec<SearchResult>>` | Dokumentation für `search_with_filter_expr` im Rahmen der Transaktions- und Facade-Orchestrierung. | `src/contextra_impl/crud.rs` |
| `pub async fn update(&self, id: &str, embedding: &[f32], metadata: Option<Value>) -> Result<()>` | Dokumentation für `update` im Rahmen der Transaktions- und Facade-Orchestrierung. | `src/contextra_impl/crud.rs` |
| `pub async fn upsert(&self, id: &str, embedding: &[f32], metadata: Option<Value>) -> Result<()>` | Dokumentation für `upsert` im Rahmen der Transaktions- und Facade-Orchestrierung. | `src/contextra_impl/crud.rs` |
| `pub async fn upsert_many(&self, docs: &[(String, Vec<f32>, Option<Value>)]) -> Result<()>` | Dokumentation für `upsert_many` im Rahmen der Transaktions- und Facade-Orchestrierung. | `src/contextra_impl/crud.rs` |
| `pub async fn upsert_text_only( &self, id: &str, text: &str, metadata: Option<Value>, ) -> Result<()>` | Dokumentation für `upsert_text_only` im Rahmen der Transaktions- und Facade-Orchestrierung. | `src/contextra_impl/crud.rs` |
| `pub async fn flush(&self) -> Result<()>` | Dokumentation für `flush` im Rahmen der Transaktions- und Facade-Orchestrierung. | `src/contextra_impl/diagnostics.rs` |
| `pub fn orphan_registry(&self) -> &Arc<contextra_checkpoint::InstanceOrphanRegistry>` | Liefert Zugriff auf die Registry verwaister Knoten. | `src/contextra_impl/diagnostics.rs` |
| `pub async fn stats(&self) -> Result<ContextraStats>` | Dokumentation für `stats` im Rahmen der Transaktions- und Facade-Orchestrierung. | `src/contextra_impl/diagnostics.rs` |
| `pub fn compute_collection_id(name: &str) -> CollectionId` | Erzeugt eine deterministische, kollisionsfreie [`CollectionId`] aus einem Namen. | `src/contextra_impl/erasure.rs` |
| `pub async fn erase_collection(&self, tenant_id: TenantId, name: &str) -> Result<ErasureReport>` | Löscht eine gesamte Collection fail-closed aus allen Schichten und erzeugt einen [`ErasureReport`]. | `src/contextra_impl/erasure.rs` |
| `pub async fn erase_document( &self, tenant_id: TenantId, collection: &str, id: &str, ) -> Result<ErasureReport>` | Löscht ein Dokument fail-closed aus allen Schichten und erzeugt einen [`ErasureReport`]. | `src/contextra_impl/erasure.rs` |
| `pub fn set_proof_key_provider(&self, provider: Arc<dyn ProofKeyProvider>)` | Sets the [`ProofKeyProvider`] for resolving deletion proof signing keys. | `src/contextra_impl/erasure.rs` |
| `pub fn summary_from_witness(layer_name: String, witness: &LayerWitness) -> LayerSummary` | Erstellt ein [`LayerSummary`] aus einem [`LayerWitness`] unter Verwendung der tatsächlichen Messwerte. | `src/contextra_impl/erasure.rs` |
| `pub fn with_proof_key_provider(self, provider: Arc<dyn ProofKeyProvider>) -> Self` | Ergänzt die Instanz um einen [`ProofKeyProvider`]. | `src/contextra_impl/erasure.rs` |
| `pub async fn close(self) -> Result<()>` | Dokumentation für `close` im Rahmen der Transaktions- und Facade-Orchestrierung. | `src/contextra_impl/lifecycle.rs` |
| `pub async fn open(path: impl AsRef<Path>) -> Result<Self>` | Opens or creates a `Contextra` instance at `path` enforcing `deletion_proof_active = false` and using default [`contextra_ports::license::L… | `src/contextra_impl/lifecycle.rs` |
| `pub async fn open_authorized( path: impl AsRef<Path>, mut config: ContextraConfig, authorized_ring: &AuthorizedRing, gate: Arc<dyn LicenseGate>, ) -> Result<Self>` | Opens or creates a `Contextra` instance with an explicitly authorized [`AuthorizedRing`] token and attached [`LicenseGate`]. | `src/contextra_impl/lifecycle.rs` |
| `pub async fn open_with_config( path: impl AsRef<Path>, mut config: ContextraConfig, ) -> Result<Self>` | Opens or creates a `Contextra` instance at `path` with explicit `config`, forcing `deletion_proof_active = false` (as no authorized token w… | `src/contextra_impl/lifecycle.rs` |
| `pub async fn repair_on_open(&self) -> Result<()>` | Dokumentation für `repair_on_open` im Rahmen der Transaktions- und Facade-Orchestrierung. | `src/contextra_impl/lifecycle.rs` |
| `pub async fn set_embedder(&self, embedder: Arc<dyn TextEmbeddingEngine>) -> Result<()>` | Dokumentation für `set_embedder` im Rahmen der Transaktions- und Facade-Orchestrierung. | `src/contextra_impl/lifecycle.rs` |
| `pub fn shutdown(&self)` | Signalisiert den Shutdown aller Hintergrunddienste. | `src/contextra_impl/lifecycle.rs` |
| `pub async fn wait_shutdown(&self)` | Wartet asynchron auf den geordneten Stopp aller Worker. | `src/contextra_impl/lifecycle.rs` |
| `pub async fn with_embedder(self, embedder: Arc<dyn TextEmbeddingEngine>) -> Self` | Dokumentation für `with_embedder` im Rahmen der Transaktions- und Facade-Orchestrierung. | `src/contextra_impl/lifecycle.rs` |
| `pub fn metrics_sink(&self) -> Arc<dyn contextra_ports::MetricsSink>` | Returns the active [`MetricsSink`]. | `src/contextra_impl/wiring.rs` |
| `pub fn set_calibrator( &self, calibrator: std::sync::Weak<parking_lot::Mutex<contextra_rank::IsotonicCalibrator>>, )` | Registriert die Score-Kalibrator-Komponente. | `src/contextra_impl/wiring.rs` |
| `pub fn set_metrics_sink(&self, sink: Arc<dyn contextra_ports::MetricsSink>)` | Configures or replaces the active [`MetricsSink`]. | `src/contextra_impl/wiring.rs` |
| `pub fn set_router(&self, router: std::sync::Weak<dyn DriftStatusProvider>)` | Registriert die Router-Komponente. | `src/contextra_impl/wiring.rs` |
| `pub fn effective_half_life(&self, temperature: f32) -> f32` | Berechnet die temperaturangepasste effektive Halbwertszeit in Transaktionszyklen. | `src/decay_controller.rs` |
| `pub fn effective_score( &self, base_score: f32, elapsed_tx: u64, inputs: &DecaySignalInputs, ) -> f32` | Berechnet den aktuellen abgewerteten Score eines Objekts unter Berücksichtigung von Alter und Systemtemperatur. | `src/decay_controller.rs` |
| `pub fn new(config: DecayControllerConfig) -> Self` | Erstellt eine neue Instanz des `AdaptiveDecayController` mit der gegebenen Konfiguration. | `src/decay_controller.rs` |
| `pub fn should_evict( &self, base_score: f32, elapsed_tx: u64, inputs: &DecaySignalInputs, ) -> bool` | Prüft, ob ein Eintrag aufgrund seines verfallenen Scores unter den Eviktions-Schwellenwert gefallen ist. | `src/decay_controller.rs` |
| `pub fn system_temperature(&self, inputs: &DecaySignalInputs) -> f32` | Berechnet die aktuelle Systemtemperatur basierend auf Tombstone-Verhältnis und Abfragelast. | `src/decay_controller.rs` |
| `pub fn with_defaults() -> Self` | Initialisiert den Controller mit den Standardwerten der Spezifikation. | `src/decay_controller.rs` |
| `pub async fn export_memories(&self) -> Result<ExportCollectionV1>` | Exportiert alle Memories und Graph-Relationen der Collection in das ExportCollectionV1 Schema. | `src/export.rs` |
| `pub fn create_extractor(mode: ExtractorMode) -> Box<dyn EntityExtractor>` | Erstellt eine Instanz des konfigurierten Extraktors. | `src/extraction/mod.rs` |
| `pub async fn extract_triples_for_config( text: &str, generator: Option<&dyn LlmTextGenerator>, config: &EntityExtractionConfig, ) -> Result<Vec<ExtractedTriple>>` | Hilfsfunktion zur Ausführung der Extraktion basierend auf der Standard-Konfiguration (OpenIE). | `src/extraction/mod.rs` |
| `pub async fn extract_triples_for_mode( mode: ExtractorMode, text: &str, generator: Option<&dyn LlmTextGenerator>, config: &EntityExtractionConfig, ) -> Result<Vec<ExtractedTriple>>` | Hilfsfunktion zur Ausführung der Extraktion für einen expliziten Extraktor-Modus. | `src/extraction/mod.rs` |
| `pub async fn extract_triples( text: &str, generator: &dyn LlmTextGenerator, config: &EntityExtractionConfig, ) -> Result<Vec<ExtractedTriple>>` | Extrahiert Wissens-Tripel (Subjekt, Prädikat, Objekt) aus dem gegebenen Text mittels LLM. # Fehlerverhalten - Wenn `config.enabled == false… | `src/extraction/open_ie.rs` |
| `pub fn extract( &self, text: &str, config: &EntityExtractionConfig, ) -> Result<Vec<ExtractedTriple>>` | Extrahieren von Wissens-Tripeln rein regelbasiert und deterministisch. | `src/extraction/rule_based.rs` |
| `pub fn new() -> Self` | Erstellt eine neue Instanz von `RuleBasedExtractor`. | `src/extraction/rule_based.rs` |
| `pub fn extract_memory_type(metadata: &Option<Value>) -> MemoryType` | Extrahiert den MemoryType aus Dokument-Metadaten (Rückwärtskompatibel). | `src/filter.rs` |
| `pub fn matches(&self, metadata: &Value) -> bool` | Prüft, ob das gegebene JSON-Metadatenobjekt die Filterbedingungen erfüllt. | `src/filter.rs` |
| `pub fn fuse_search_results_with_signal_strategies( result_sets: Vec<(String, Vec<SearchResult>, f32)>, max_results: usize, priority: MetadataMergePriority, include_provenance: bool, resonance_config: Option<&ResonanceConfig>, strategies: contextra_types::Sign…` | Fuses search result sets using per-signal fusion strategies (`SignalFusionStrategies`). | `src/fusion.rs` |
| `pub fn fuse_search_results_with_strategy( result_sets: Vec<(String, Vec<SearchResult>, f32)>, max_results: usize, priority: MetadataMergePriority, include_provenance: bool, resonance_config: Option<&ResonanceConfig>, strategy: contextra_types::FusionStrategy,…` | Fuses search result sets using the specified FusionStrategy. | `src/fusion.rs` |
| `pub fn fuse_signals( result_sets: Vec<(String, Vec<SearchResult>, f32)>, max_results: usize, ) -> Vec<SearchResult>` | Convenience function for fusing multi-signal search results. | `src/fusion.rs` |
| `pub async fn import_memories(&self, col_export: ExportCollectionV1) -> Result<ImportSummary>` | Importiert alle Memories und Graph-Relationen einer ExportCollectionV1 idempotent (Upsert-Semantik). | `src/import.rs` |
| `pub async fn import_memories(&self, doc: ExportDocumentV1) -> Result<ImportSummary>` | Importiert ein ExportDocumentV1 in die Contextra-Datenbank mit Schema-Versionsprüfung und Idempotenz. | `src/import.rs` |
| `pub fn config(&self) -> &ContextraConfig` | Returns a reference to the active [`ContextraConfig`]. | `src/lib.rs` |
| `pub fn inner_storage(&self) -> Arc<LsmStorage>` | Gibt den zugrundeliegenden LSM-Speicher zurück (intern/verborgen). | `src/lib.rs` |
| `pub fn license_gate(&self) -> Arc<dyn LicenseGate>` | Returns a reference to the attached [`LicenseGate`]. | `src/lib.rs` |
| `pub fn register_consolidation_launcher<F>(_f: F) where F: Fn( Arc<Collection<LsmStorage>>, std::time::Duration, usize, tokio_util::sync::CancellationToken, ) -> tokio::task::JoinHandle<()> + Send + Sync + 'static,` | — | `src/lib.rs` |
| `pub fn with_consolidation_launcher(mut self, launcher: ConsolidationLauncher) -> Self` | Konfiguriert den optionalen Konsolidierungs-Launcher auf dem Builder. | `src/lib.rs` |
| `pub fn with_metrics_sink(mut self, sink: Arc<dyn contextra_ports::MetricsSink>) -> Self` | Konfiguriert die Metrik-Senke zur Erfassung von Systemmetriken. | `src/lib.rs` |
| `pub fn with_tenant_policy(mut self, policy: TenantPolicy) -> Self` | Konfiguriert die Mandantenisolations- und Zugriffspolicy. | `src/lib.rs` |
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
| `pub async fn recover_pending_intents<S: StorageEngine>( storage: &S, next_tx: &std::sync::atomic::AtomicU64, ) -> Result<usize>` | Recovers or reconciles any pending transaction intents found in storage upon engine startup. For each pending intent: - If storage records… | `src/transaction/cleanup.rs` |
| `pub async fn execute_rollback(&mut self)` | Dokumentation für `execute_rollback` im Rahmen der Transaktions- und Facade-Orchestrierung. | `src/transaction/compensating_actions.rs` |
| `pub fn new() -> Self` | Initialisiert eine leere Aktionsliste. | `src/transaction/compensating_actions.rs` |
| `pub fn new( collection: Collection<S, V>, intent_key: Vec<u8>, doc_ids: Arc<Vec<DocId>>, f_keys: Vec<StagedKeyOp>, r_keys: Vec<StagedKeyOp>, ) -> Self` | Dokumentation für `new` im Rahmen der Transaktions- und Facade-Orchestrierung. | `src/transaction/compensating_actions.rs` |
| `pub fn new(collection: Collection<S, V>, doc_ids: Arc<Vec<DocId>>) -> Self` | Dokumentation für `new` im Rahmen der Transaktions- und Facade-Orchestrierung. | `src/transaction/compensating_actions.rs` |
| `pub fn new( collection: Collection<S, V>, tenant_id: TenantId, doc_ids: Arc<Vec<DocId>>, ) -> Self` | Dokumentation für `new` im Rahmen der Transaktions- und Facade-Orchestrierung. | `src/transaction/compensating_actions.rs` |
| `pub fn push<A: CompensatingAction + 'static>(&mut self, action: A)` | Dokumentation für `push` im Rahmen der Transaktions- und Facade-Orchestrierung. | `src/transaction/compensating_actions.rs` |
| `pub fn new(collection: Collection<S, V>, tx_id: TxId) -> Self` | Dokumentation für `new` im Rahmen der Transaktions- und Facade-Orchestrierung. | `src/transaction/db_transaction.rs` |
| `pub fn record_keys(&self, forward: Vec<u8>, reverse: Vec<u8>, doc_id: DocId)` | Records keys and IDs that have been operated on for potential compensating rollback. | `src/transaction/db_transaction.rs` |
| `pub fn record_keys_with_old_values( &self, forward: Vec<u8>, old_forward: Option<Bytes>, reverse: Vec<u8>, old_reverse: Option<Bytes>, doc_id: DocId, )` | Records keys and IDs along with pre-write values for potential compensating rollback. | `src/transaction/db_transaction.rs` |
| `pub fn stage_graph_edge(&self, edge: Edge)` | Dokumentation für `stage_graph_edge` im Rahmen der Transaktions- und Facade-Orchestrierung. | `src/transaction/db_transaction.rs` |
| `pub fn stage_graph_edge_delete(&self, from: EntityId, to: EntityId)` | Dokumentation für `stage_graph_edge_delete` im Rahmen der Transaktions- und Facade-Orchestrierung. | `src/transaction/db_transaction.rs` |
| `pub fn stage_graph_entity(&self, entity: Entity)` | Dokumentation für `stage_graph_entity` im Rahmen der Transaktions- und Facade-Orchestrierung. | `src/transaction/db_transaction.rs` |
| `pub fn stage_graph_entity_delete(&self, entity_id: EntityId)` | Dokumentation für `stage_graph_entity_delete` im Rahmen der Transaktions- und Facade-Orchestrierung. | `src/transaction/db_transaction.rs` |
| `pub fn stage_hyperedge(&self, hyperedge: contextra_graph::hyperedge::HyperEdge)` | Dokumentation für `stage_hyperedge` im Rahmen der Transaktions- und Facade-Orchestrierung. | `src/transaction/db_transaction.rs` |
| `pub fn stage_text_delete(&self, doc_id: DocId)` | Dokumentation für `stage_text_delete` im Rahmen der Transaktions- und Facade-Orchestrierung. | `src/transaction/db_transaction.rs` |
| `pub fn stage_text_insert(&self, doc_id: DocId, text: String)` | Dokumentation für `stage_text_insert` im Rahmen der Transaktions- und Facade-Orchestrierung. | `src/transaction/db_transaction.rs` |
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

### Architekturbeschreibung

# Kognitions-Engine (`contextra-cognition`)  Die Crate `contextra-cognition` bildet die Kognitions- und Konsolidierungsschicht im Contextra-Workspace (historisch Ring 3, adaptiert nach Spezifikation §4/§5.2). Sie stellt Hintergrundprozesse für Gedächtniskonsolidierung (Memory Consolidation), Kontext-Kompaktierung (Context Compaction), semantische Aggregation (LeanRAG / Hyperedge Consolidation) und Wartungsplanung (Maintenance Scheduling) bereit.  ## Ring-Zugehörigkeit und Abhängigkeiten - **Abhängigkeiten (eingehend):** Wird von höheren Orchestrierungsschichten wie `contextra-db` eingebunden. - **Abhängigkeiten (ausgehend):** Nutzt Basis- und Speicher-Crates: - `contextra-types`: Domänentypen, Fehlermodelle, Mandanten-IDs. - `contextra-ports`: Abstrakte Port-Schnittstellen (Storage, Clock, Rng, Metrics). - `contextra-engine`: Ausführungs- und Storage-Subsysteme. - `contextra-store`: Schlüssel-Wert- und LSM-Speicher (Key-Value-Store). - `contextra-vector`: Vektorindizes und Embeddings. - `contextra-graph`: CSR-Graphdarstellung und Hyperedges.  ## Kernkomponenten - [`ContextManager`]: Verwaltet Kontexte, Raumgrenzen (SpatialFence) und Token-Budgets. - [`ContextCompactor`]: Führt Transaktions- und Dialogkompaktierung durch. - [`ConsolidationEngine`]: Hintergrund-Worker für Schlafzyklen (Sleep Cycles) und semantische Verschmelzung. - [`MaintenanceScheduler`]: Steuert periodische Wartungsläufe und Speicherbereinigungen.  ## Invarianten - **Kein Unsafe-Code:** `#![forbid(unsafe_code)]` wird strikt auf Crate-Ebene erzwungen. - **Mandantentrennung:** Strikte Isolierung nach `tenant_id` (VETO-F10). - **Deterministische Sortierung:** Alle aggregierten und vorverdichteten Strukturen werden deterministisch sortiert.

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

**`SuperEdgeSink`** — *Senkentrait für Superkanten und Tombstones.* (`src/aggregation_phase.rs`)

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

### Öffentliche Datentypen — Structs (32)

| Struct | Felder / Form | Beschreibung | Datei |
|---|---|---|---|
| `AggregationConfig` | `{ /// Maximal zulässiger Speicherverbrauch für Compaction/Aggregation in MB (Default: 512). pub max_compaction_peak_memory_mb: usize, /// Schwelle λ für die Supe… }` | Konfiguration für die Aggregationsphase (LeanRAG Stage 3). | `src/aggregation_phase.rs` |
| `AggregationEdge` | `{ /// Eindeutige ID der Hyperkante. pub id: HyperEdgeId, /// Prädikatstyp der Kante. pub predicate_type: u32, /// Beteiligte Entitäten der Kante. pub participant… }` | Eingabekante für die Aggregationsphase. | `src/aggregation_phase.rs` |
| `AggregationNode` | `{ /// Eindeutige EntityId des Knotens. pub entity: EntityId, /// Merkmalsvektor/Embedding. pub embedding: Vec<f32>, /// Typ-ID des Entitätstyps. pub type_id: u32, }` | Eingabeknoten für die Aggregationsphase. | `src/aggregation_phase.rs` |
| `AggregationPhaseResult` | `{ /// Anzahl tombstonierter Kanten wegen Inkompatibilität. pub raw_edges_tombstoned: usize, /// Anzahl erzeugter abstrakter Super-Hyperkanten. pub abstract_hyper… }` | Ergebnis der Aggregationsphase. | `src/aggregation_phase.rs` |
| `AlphaNode` | `{ /// Cluster-Index. pub cluster_id: usize, /// Mitglieds-Entitäten des Clusters. pub members: Vec<EntityId>, /// LLM-generierte Zusammenfassung/Synthese. pub su… }` | Synthetisierter abstrakter Alpha-Knoten. | `src/aggregation_phase.rs` |
| `CommunityStabilityTracker` | `{ history: AHashMap<u64, u32>, }` | Verfolgt die Stabilität von Graph-Communities über Zyklen hinweg. | `src/memory_consolidation.rs` |
| `CompactedContext` | `{ /// Beibehaltene Chunks (innerhalb Budget). pub retained_chunks: Vec<ContextChunk>, /// Status-Token für kompaktierte Chunks. pub status_tokens: Vec<StatusToke… }` | Ergebnis eines Kompaktierungslaufs. | `src/context_compaction/types.rs` |
| `ConsolidationConfig` | `{ /// Mindestanzahl von Turns pro Segment (Default: 3). pub min_turns_per_segment: usize, /// Maximale Anzahl von Turns pro Segment (Default: 20). pub max_turns_… }` | Konfiguration für den Structural Consolidation Pass. | `src/memory_consolidation.rs` |
| `ConsolidationEngine` | `{ collection: Arc<Collection<S, V>>, llm: Option<Arc<dyn LlmTextGenerator>>, validator: Option<Arc<dyn ResponseGroundingValidator>>, rich_validator: Option<Arc<d… }` | Engine für periodische Hintergrund-Konsolidierung und Wissenssynthese. | `src/consolidation_executor.rs` |
| `ConsolidationLockGuard` | `{ flag: Arc<AtomicBool>, }` | RAII-Guard zur Absicherung gegen parallele Konsolidierungsläufe. | `src/consolidation_executor.rs` |
| `ConsolidationNodesGuard` | `{ collection: &'a Collection<S, V>, _guard: MutexGuard<'a, ()>, }` | Guard zur Erzwingung der Lock-Reihenfolge und kanonischen Node-Sortierung während der Konsolidierung. | `src/consolidation_locks.rs` |
| `ConsolidationPhaseResult` | `{ /// Anzahl der erzeugten Segmente. pub segments_created: usize, /// Liste aller DocIds, die als Duplikate markiert/tombstoned wurden. pub duplicates_tombstoned… }` | Ergebnis des Structural Consolidation Pass. | `src/memory_consolidation.rs` |
| `ConsolidationPipelineResult` | `{ /// Ergebnis des Structural Pass. pub structural: ConsolidationPhaseResult, /// Ergebnis des Synthesis Pass. pub synthesis: crate::memory_consolidation::Synthe… }` | Gesamtergebnis der dreistufigen Konsolidierungspipeline. | `src/aggregation_phase.rs` |
| `ConsolidationSession` | `{ /// Reference to the active collection. pub collection: &'a Collection<S, V>, /// Source document IDs and their transaction IDs captured at read snapshot time.… }` | Transaktionale OCC-Konsolidierungssitzung. | `src/context_compaction/session.rs` |
| `ContextCompactor` | `{ budget: TokenBudget, strategy: CompactionStrategy, rng: Arc<dyn Rng>, }` | Zentrale Kompaktierungs-Engine. | `src/context_compaction/compactor.rs` |
| `ContextManager` | `{ /// Token budget configuration. budget: TokenBudget, /// Adaptive relevance threshold. relevance_threshold: f32, }` | Verwaltet autonome Kontextvorbereitung und Token-Budgetierung für Sprachmodelle. | `src/context.rs` |
| `CsrGraphSuperEdgeSink` | `{ graph: &'a CsrGraph, wal_tx: TxId, pending_tombstones: Vec<HyperEdgeId>, pending_edges: Vec<HyperEdge>, }` | Gepufferte Senke zum Schreiben synthetischer Hyperkanten und Tombstone-Anfragen in den CsrGraph. | `src/graph_sink.rs` |
| `GroundingValidatorAdapter` | `(tuple/unit)` | Adapter für GroundingValidator auf ResponseGroundingValidator. | `src/consolidation_executor.rs` |
| `LeanRagInputs` | `{ /// Liste der zugrundeliegenden Entitätsknoten. pub nodes: Vec<AggregationNode>, /// Liste der zugrundeliegenden Hyperkanten zwischen den Knoten. pub edges: Ve… }` | Eingabepaket für die LeanRAG Stage-3-Aggregation. | `src/leanrag_input.rs` |
| `MaintenanceConfig` | `{ /// Intervall zwischen Ticks in Sekunden. Default: 60s. pub tick_interval_secs: u64, // --- F-01 Adaptive Decay Controller --- /// Ob Decay-Eviction aktiviert… }` | Zentrale Konfiguration für den MaintenanceScheduler (§10.2). | `src/maintenance_config.rs` |
| `MaintenanceScheduler` | `{ config: MaintenanceConfig, collection: Arc<Collection<S, V>>, consolidation_config: ConsolidationConfig, #[cfg(feature = "edge-reinforcement-learning")] edge_r… }` | Zentraler Scheduler für koordiniertes Ausführen von Background-Maintenance-Prozessen (ADR-079). | `src/maintenance_scheduler.rs` |
| `MetaChunk` | `{ /// Der synthetisierte, abstrakte Inhalt (MUSS mit `[SYNTHESIZED FROM {n} SOURCES] ` beginnen). pub content: String, /// DocIds aller Quell-Chunks, aus denen s… }` | Generativ synthetisierter Wissens-Chunk mit Provenienz. | `src/memory_consolidation.rs` |
| `PercolationConfig` | `{ pub critical_threshold: f32, pub rebonding_similarity: f32, pub max_new_edges_per_pass: usize, }` | Fallback-Konfiguration für Perkolationsprüfungen. | `src/maintenance_config.rs` |
| `PreCondensedInput` | `{ pub tenant_id: contextra_types::TenantId, pub source_label: String, pub text_digest: String, pub token_count: usize, pub provenance: Vec<DocId>, }` | Struktur für vorverdichtete externe Eingabeknoten. | `src/memory_consolidation.rs` |
| `SegmentSynthesisResult` | `{ /// Neu synthetisierte Chunks (einer pro konsolidiertem Segment). pub synthesized_chunks: Vec<SynthesizedChunk>, /// Anzahl der Segmente, für die keine Synthes… }` | Ergebnis des Generative Synthesis Pass auf Segment-Ebene. | `src/synthesis_phase.rs` |
| `SpatialFence` | `{ /// The region identifier to filter by. pub region: String, /// The metadata field name containing the region. pub field_name: String, }` | Geographische Beschränkung (Spatial Fencing) für regionsspezifische Chunks. | `src/context.rs` |
| `StatusToken` | `{ /// Kompakter Beschreibungstext (z. B. "Tool-Output: DB-Abfrage lieferte 42 Ergebnisse"). pub summary: String, /// Anzahl der ersetzten originalen Tokens. pub… }` | Status-Token als kompakter Stellvertreter für komprimierte Chunks. | `src/context_compaction/types.rs` |
| `SuperEdgeDraft` | `{ /// Indizes der beiden beteiligten Cluster `(j, k)`. pub cluster_pair: (usize, usize), /// Alle beteiligten Entitäten aus den untergeordneten Kanten. pub parti… }` | Entwurf einer abstrakten Super-Hyperkante. | `src/aggregation_phase.rs` |
| `SynthesisConfig` | `{ /// Mindestanzahl von Chunks/Dokumenten in einer Community (Default: 4). pub min_community_size: usize, /// Anzahl aufeinanderfolgender Beobachtungen, bis eine… }` | Konfiguration für den Generative Synthesis Pass. | `src/memory_consolidation.rs` |
| `SynthesisPhaseResult` | `{ /// Neu generierte MetaChunks. pub synthesized: Vec<MetaChunk>, /// Hashes von qualifizierten Communities, deren Synthese wegen `max_llm_calls_per_cycle` versc… }` | Ergebnis des Generative Synthesis Pass. | `src/memory_consolidation.rs` |
| `SynthesizedChunk` | `{ /// Der neue, abstrakte Inhalt. pub content: String, /// DocIds der Quell-Turns aus dem ursprünglichen Segment. pub source_turn_ids: Vec<DocId>, /// Modell, da… }` | Ein generativ synthetisierter Wissens-Chunk mit Provenienz. | `src/synthesis_phase.rs` |
| `TurnSegment` | `{ /// Liste aller DocIds der Turns in diesem Segment. pub turn_ids: Vec<DocId>, /// Repräsentatives Embedding des Segments. /// Berechnet als Zentroid (Mittelwer… }` | Semantisch zusammenhängendes Segment aufeinanderfolgender Turns. | `src/memory_consolidation.rs` |

### Öffentliche Datentypen — Enums (1)

| Enum | Varianten | Beschreibung | Datei |
|---|---|---|---|
| `CompactionStrategy` | `/// Einfachste Strategie: Chunks über Token-Limit werden weggelassen. Truncate, /// Summarisierung via LLM (erfordert externen Summarizer-Trait). Summarize, /// Ersetze Tool-Outputs durch kompakte St…` | Strategie für die Kontext-Kompaktierung. | `src/context_compaction/types.rs` |

### Typ-Aliase (1)

| Alias | Definition | Beschreibung |
|---|---|---|
| `SynthesisPhaseResult` | `type SynthesisPhaseResult= SegmentSynthesisResult` | Veralteter Typalias für SegmentSynthesisResult. |

### Konstanten (4)

| Konstante | Typ | Beschreibung |
|---|---|---|
| `DEFAULT_MAX_LEANRAG_NODES` | `usize` | Standard-Obergrenze für LeanRAG-Knoten. |
| `MAX_PRE_CONDENSED_DIGEST_LEN` | `usize` | Maximale Länge für text_digest in PreCondensedInput. |
| `MAX_PRE_CONDENSED_LABEL_LEN` | `usize` | Maximale Länge für source_label in PreCondensedInput. |
| `MAX_PRE_CONDENSED_PROVENANCE_COUNT` | `usize` | Maximale Anzahl an Provenienz-IDs in PreCondensedInput. |

### Öffentliche Funktionen (71)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn check_compaction_budget(peak_bytes: usize, cfg: &AggregationConfig) -> Result<()>` | Prüft den geschätzten Speicherbedarf gegen das konfigurierte Budget. | `src/aggregation_phase.rs` |
| `pub fn compute_entity_community_hash(members: &[EntityId]) -> u64` | Berechnet einen deterministischen Hash über eine Liste von EntityIds. | `src/aggregation_phase.rs` |
| `pub async fn run_aggregation_pass( nodes: &[AggregationNode], edges: &[AggregationEdge], cfg: &AggregationConfig, llm: &dyn LlmTextGenerator, tracker: &mut CommunityStabilityTracker, sink: &mut dyn SuperEdgeSink, ) -> Result<(AggregationPhaseResult, Vec<Alpha…` | Führt den vollständigen Aggregationspass aus. | `src/aggregation_phase.rs` |
| `pub fn validate(&self) -> Result<()>` | Validiert die Konfigurationseinstellungen. | `src/aggregation_phase.rs` |
| `pub async fn execute_background_consolidation<S: StorageEngine, V: VectorIndex>( collection: &Collection<S, V>, turns: &[(DocId, Vec<f32>)], consolidation_config: &ConsolidationConfig, synthesis_config: Option<&SynthesisConfig>, llm: Option<&dyn LlmTextGenera…` | Führt die vollständige Hintergrund-Konsolidierung aus. | `src/consolidation_executor.rs` |
| `pub async fn execute_background_consolidation_with_rich_validator< S: StorageEngine, V: VectorIndex, >( collection: &Collection<S, V>, turns: &[(DocId, Vec<f32>)], consolidation_config: &ConsolidationConfig, synthesis_config: Option<&SynthesisConfig>, llm: Op…` | Führt Hintergrund-Konsolidierung mit Unterstützung für Rich-Grounding-Validatoren aus. | `src/consolidation_executor.rs` |
| `pub async fn execute_consolidation_pass<S: StorageEngine, V: VectorIndex>( collection: &Collection<S, V>, turns: &[(DocId, Vec<f32>)], config: &ConsolidationConfig, ) -> Result<ConsolidationPhaseResult>` | Führt den Structural Pass aus und wendet Änderungen auf die Collection an. | `src/consolidation_executor.rs` |
| `pub async fn execute_leanrag_aggregation_stage<S: StorageEngine, V: VectorIndex>( collection: &Collection<S, V>, nodes: &[AggregationNode], edges: &[AggregationEdge], cfg: &AggregationConfig, llm: &dyn LlmTextGenerator, tracker: &mut CommunityStabilityTracker…` | Führt die LeanRAG Stage 3 Aggregation aus. | `src/consolidation_executor.rs` |
| `pub async fn execute_sleep_cycle<S: StorageEngine, V: VectorIndex>( collection: &Collection<S, V>, turns: &[(DocId, Vec<f32>)], consolidation_config: &ConsolidationConfig, synthesis_config: Option<&SynthesisConfig>, llm: Option<&dyn LlmTextGenerator>, stabili…` | Veralteter Wrapper für execute_background_consolidation. | `src/consolidation_executor.rs` |
| `pub fn new( collection: Arc<Collection<S, V>>, consolidation_config: ConsolidationConfig, synthesis_config: SynthesisConfig, interval: std::time::Duration, cancel_token: tokio_util::sync::CancellationToken, ) -> Self` | Erstellt eine neue `ConsolidationEngine`-Instanz. | `src/consolidation_executor.rs` |
| `pub async fn run(&self)` | Hauptschleife der `ConsolidationEngine`. | `src/consolidation_executor.rs` |
| `pub async fn run_cycle( &self, ) -> Result<(ConsolidationPhaseResult, Option<SynthesisPhaseResult>)>` | Führt einen einzelnen Konsolidierungszyklus aus. | `src/consolidation_executor.rs` |
| `pub fn start(self: Arc<Self>) -> tokio::task::JoinHandle<()>` | Startet die Engine als Tokio-Background-Task. | `src/consolidation_executor.rs` |
| `pub fn start_consolidation_reaper<S: StorageEngine + 'static, V: VectorIndex + 'static>( collection: Arc<Collection<S, V>>, consolidation_config: ConsolidationConfig, synthesis_config: SynthesisConfig, interval: std::time::Duration, cancel_token: tokio_util::…` | Veralteter Alias für start_consolidation_worker. | `src/consolidation_executor.rs` |
| `pub fn start_consolidation_worker<S: StorageEngine + 'static, V: VectorIndex + 'static>( collection: Arc<Collection<S, V>>, consolidation_config: ConsolidationConfig, synthesis_config: SynthesisConfig, interval: std::time::Duration, cancel_token: tokio_util::…` | Startet einen Hintergrund-Worker für periodische Konsolidierung. | `src/consolidation_executor.rs` |
| `pub fn start_worker( collection: Arc<Collection<S, V>>, consolidation_config: ConsolidationConfig, synthesis_config: SynthesisConfig, interval: std::time::Duration, cancel_token: tokio_util::sync::CancellationToken, ) -> tokio::task::JoinHandle<()>` | Starts a background task for periodic consolidation. | `src/consolidation_executor.rs` |
| `pub fn try_acquire(flag: &Arc<AtomicBool>) -> Option<Self>` | Versucht, das Konsolidierungs-Lock atomic zu erwerben (`compare_exchange`). Gibt `Some(ConsolidationLockGuard)` zurück, wenn das Lock erfol… | `src/consolidation_executor.rs` |
| `pub fn with_leanrag(mut self, cfg: AggregationConfig) -> Self` | Fügt eine optionale AggregationConfig für LeanRAG Stage 3 (Opt-in) hinzu. | `src/consolidation_executor.rs` |
| `pub fn with_llm(mut self, llm: Arc<dyn LlmTextGenerator>) -> Self` | Fügt ein optionales LLM für die generative Wissenssynthese hinzu. | `src/consolidation_executor.rs` |
| `pub fn with_rich_validator(mut self, validator: Arc<dyn GroundingValidator>) -> Self` | Fügt einen optionalen reichhaltigen Grounding-Validator (`GroundingValidator`) mit Kalibrierung & Policy-Violation-Semantik hinzu. | `src/consolidation_executor.rs` |
| `pub fn with_validator(mut self, validator: Arc<dyn ResponseGroundingValidator>) -> Self` | Fügt einen optionalen Grounding-Validator für den Generative Synthesis Pass hinzu. | `src/consolidation_executor.rs` |
| `pub async fn cascade_invalidate_edges_ordered( &self, superseded_doc_ids: &[DocId], wal_seq: u64, ) -> (Vec<(DocId, Result<CascadeInvalidationReport>)>, Vec<String>)` | Führt kaskadierende Graph-Edge-Invalidierungen in kanonischer Reihenfolge aus. | `src/consolidation_locks.rs` |
| `pub fn collection(&self) -> &'a Collection<S, V>` | Liefert eine Referenz auf die geschützte Collection. | `src/consolidation_locks.rs` |
| `pub fn try_acquire(collection: &'a Collection<S, V>) -> Option<Self>` | Versucht, das Konsolidierungs-Lock atomar zu erwerben. | `src/consolidation_locks.rs` |
| `pub fn estimate_tokens(text: &str) -> usize` | Schätzt die Token-Anzahl eines Textes heuristisch nach BPE-Regeln. | `src/context.rs` |
| `pub fn matches(&self, chunk: &ContextChunk) -> bool` | Prüft, ob ein Chunk den regionalen Kriterien entspricht. | `src/context.rs` |
| `pub fn new(budget: TokenBudget) -> Self` | Erstellt einen neuen ContextManager mit dem angegebenen TokenBudget. | `src/context.rs` |
| `pub fn new(region: impl Into<String>) -> Self` | Erstellt eine neue SpatialFence für die angegebene Region. | `src/context.rs` |
| `pub fn prepare_context(&self, mut chunks: Vec<ContextChunk>) -> Result<ContextWindow>` | Bereitet das finale ContextWindow aus übergebenen Chunks vor. | `src/context.rs` |
| `pub fn relevance_threshold(&self) -> f32` | Liefert die aktuelle Mindestrelevanzschwelle. | `src/context.rs` |
| `pub fn set_relevance_threshold(&mut self, threshold: f32)` | Setzt die Mindestrelevanzschwelle für einzubindende Chunks. | `src/context.rs` |
| `pub fn with_defaults() -> Self` | Erstellt einen ContextManager mit Standardeinstellungen. | `src/context.rs` |
| `pub async fn cleanup_orphaned_consolidation_intents<S: StorageEngine>( storage: &S, next_tx: &std::sync::atomic::AtomicU64, ) -> Result<usize>` | Bereinigt nach Abstürzen verwaiste ConsolidationIntents im Storage. | `src/context_compaction/cleanup.rs` |
| `pub fn compact(&self, chunks: Vec<ContextChunk>) -> CompactedContext` | Kompaktiert Chunks auf das konfigurierte Token-Budget. | `src/context_compaction/compactor.rs` |
| `pub async fn consolidate_via_llm( &self, tenant: TenantId, chunks: &[ContextChunk], generator: &(impl LlmTextGenerator + ?Sized), _model: &str, ) -> Result<CompactedContext>` | Konsolidiert Chunks per LLM zu einem einzigen Zusammenfassungs-Chunk. | `src/context_compaction/compactor.rs` |
| `pub async fn consolidate_with_retry<S, V, G>( collection: &Collection<S, V>, source_doc_ids: &[DocId], target_doc_id: DocId, generator: &G, max_retries: usize, ) -> Result<()> where S: StorageEngine, V: VectorIndex, G: LlmTextGenerator + ?Sized,` | Führt Konsolidierung mit automatischem Retry bei OCC-Konflikten aus (veraltet). | `src/context_compaction/compactor.rs` |
| `pub async fn consolidate_with_retry_with_rng<S, V, G>( collection: &Collection<S, V>, source_doc_ids: &[DocId], target_doc_id: DocId, generator: &G, max_retries: usize, rng: &dyn Rng, ) -> Result<()> where S: StorageEngine, V: VectorIndex, G: LlmTextGenerator…` | Führt Konsolidierung mit automatischem Retry und injiziertem RNG aus. | `src/context_compaction/compactor.rs` |
| `pub fn new(budget: TokenBudget, strategy: CompactionStrategy) -> Self` | Erstellt einen ContextCompactor mit Standard-RNG (veraltet). | `src/context_compaction/compactor.rs` |
| `pub fn new_with_rng( budget: TokenBudget, strategy: CompactionStrategy, rng: Arc<dyn Rng>, ) -> Self` | Erstellt einen ContextCompactor mit explizit injiziertem RNG. | `src/context_compaction/compactor.rs` |
| `pub fn with_rng(mut self, rng: Arc<dyn Rng>) -> Self` | Setzt den Zufallszahlengenerator für Backoff-Jitter. | `src/context_compaction/compactor.rs` |
| `pub async fn abort(self) -> Result<()>` | Bricht die Session ab und entfernt den Intent-Schlüssel. | `src/context_compaction/session.rs` |
| `pub async fn commit( self, target_string_id: &str, embedding: &[f32], summary_content: &str, metadata: Option<serde_json::Value>, ) -> Result<()>` | Committet das konsolidierte Dokument und löscht Quelldokumente. | `src/context_compaction/session.rs` |
| `pub async fn commit_ref( &self, target_string_id: &str, embedding: &[f32], summary_content: &str, metadata: Option<serde_json::Value>, ) -> Result<()>` | Committet das konsolidierte Dokument unter Beibehaltung der Session-Referenz. | `src/context_compaction/session.rs` |
| `pub async fn execute<G>(&self, generator: &G) -> Result<()> where G: LlmTextGenerator + ?Sized,` | Führt den gesamten Konsolidierungsablauf aus. | `src/context_compaction/session.rs` |
| `pub async fn refresh(&mut self) -> Result<Vec<DocId>>` | Aktualisiert Versionen nach einem OCC-Konflikt. | `src/context_compaction/session.rs` |
| `pub async fn start( collection: &'a Collection<S, V>, source_doc_ids: &[DocId], target_id: DocId, ) -> Result<Self>` | Startet eine neue Konsolidierungs-Session und sichert Versionen der Quelldokumente. | `src/context_compaction/session.rs` |
| `pub async fn validate_occ(&self) -> Result<()>` | Validiert die optimistische Nebenläufigkeitskontrolle. | `src/context_compaction/session.rs` |
| `pub fn new(graph: &'a CsrGraph, wal_tx: TxId) -> Self` | Erstellt eine neue CsrGraphSuperEdgeSink gebunden an den Graphen und die WAL-Transaktion. | `src/graph_sink.rs` |
| `pub fn build_leanrag_inputs<S: StorageEngine, V: VectorIndex>( collection: &Collection<S, V>, turns: &[(DocId, Vec<f32>)], max_nodes: usize, ) -> LeanRagInputs` | Baut LeanRagInputs aus Turns und Graphendaten deterministisch auf. | `src/leanrag_input.rs` |
| `pub fn active_agent_sessions(&self) -> usize` | Liefert die Anzahl aktuell aktiver Agenten-Sessions. | `src/maintenance_scheduler.rs` |
| `pub fn decrement_active_sessions(&self) -> usize` | Verringert die Anzahl aktiver Agenten-Sessions. | `src/maintenance_scheduler.rs` |
| `pub fn increment_active_sessions(&self) -> usize` | Erhöht die Anzahl aktiver Agenten-Sessions. | `src/maintenance_scheduler.rs` |
| `pub fn new( config: MaintenanceConfig, collection: Arc<Collection<S, V>>, consolidation_config: ConsolidationConfig, ) -> Self` | Erstellt eine neue Instanz des MaintenanceScheduler. | `src/maintenance_scheduler.rs` |
| `pub async fn run_tick(&self)` | Führt einen einzelnen sequenziellen Wartungs-Tick aus. | `src/maintenance_scheduler.rs` |
| `pub fn start( self: Arc<Self>, cancel_token: tokio_util::sync::CancellationToken, ) -> tokio::task::JoinHandle<()>` | Startet den Scheduler in einem eigenen Tokio-Hintergrund-Task. | `src/maintenance_scheduler.rs` |
| `pub fn with_clock(mut self, clock: Arc<dyn Clock>) -> Self` | Setzt die Uhr für den Scheduler (Clock-Port Injektion nach Regel 3). | `src/maintenance_scheduler.rs` |
| `pub fn with_edge_reinforcement_buffer( mut self, buffer: Arc<contextra_graph::EdgeReinforcementBuffer>, ) -> Self` | Setzt den optionalen Puffer für Kantenverstärkung. | `src/maintenance_scheduler.rs` |
| `pub fn compact_segment_via_context_compactor( segment: &TurnSegment, compactor: &ContextCompactor, chunks: &[ContextChunk], ) -> CompactedContext` | Kompaktiert ein einzelnes Segment via ContextCompactor. | `src/memory_consolidation.rs` |
| `pub fn compute_community_hash(member_doc_ids: &[DocId]) -> u64` | Berechnet einen deterministischen 64-Bit-Hash für Member-DocIds. | `src/memory_consolidation.rs` |
| `pub fn detect_near_duplicates(turns: &[(DocId, Vec<f32>)], threshold: f32) -> Vec<(DocId, DocId)>` | Führt segmentlokale Duplikatserkennung via Cosine-Similarity durch. | `src/memory_consolidation.rs` |
| `pub fn group_turns_into_segments( turns: &[(DocId, Vec<f32>)], config: &ConsolidationConfig, ) -> Vec<TurnSegment>` | Gruppiert benachbarte Turns via Sliding-Window-Clustering. | `src/memory_consolidation.rs` |
| `pub fn intake_pre_condensed_inputs( target_tenant_id: &contextra_types::TenantId, inputs: &[PreCondensedInput], validator: Option<&dyn ResponseGroundingValidator>, min_grounding_score: Option<f32>, ) -> contextra_types::Result<Vec<TurnSegment>>` | Nimmt vorverdichtete Eingabeknoten mandantengeschützt in die Pipeline auf. | `src/memory_consolidation.rs` |
| `pub fn new() -> Self` | — | `src/memory_consolidation.rs` |
| `pub fn observe(&mut self, community_members_hash: u64) -> u32` | Registriert die Beobachtung einer Community und liefert die aktualisierte Anzahl aufeinanderfolgender Zyklen. | `src/memory_consolidation.rs` |
| `pub fn reset_if_absent(&mut self, currently_observed: &AHashSet<u64>)` | Setzt nicht mehr beobachtete Communities zurück/entfernt sie aus der Historie. | `src/memory_consolidation.rs` |
| `pub fn run_consolidation_pass( turns: &[(DocId, Vec<f32>)], config: &ConsolidationConfig, ) -> ConsolidationPhaseResult` | Orchestriert den deterministischen Structural Consolidation Pass. | `src/memory_consolidation.rs` |
| `pub async fn run_structural_synthesis_pass( stable_communities: &[(u64, Vec<DocId>)], source_texts: &AHashMap<DocId, String>, llm: &dyn LlmTextGenerator, config: &SynthesisConfig, validator: Option<&dyn ResponseGroundingValidator>, ) -> contextra_types::Resul…` | Führt den Syntheselauf über stabile Graph-Communities aus. | `src/memory_consolidation.rs` |
| `pub async fn consolidate_semantic_hyperedges<S: StorageEngine, V: VectorIndex>( collection: &Collection<S, V>, embedder: &dyn TextEmbeddingEngine, llm: &dyn LlmTextGenerator, config: &AggregationConfig, ) -> Result<AggregationPhaseResult>` | Freie Funktion zur Ausführung der semantischen Hyperkanten-Konsolidierung (LeanRAG Stage 3). | `src/semantic_aggregation_facade.rs` |
| `pub async fn run_synthesis_pass( segments: &[TurnSegment], segment_texts: &[Vec<String>], // Texte der Turns pro Segment synthesizer: &dyn SegmentSynthesizer, min_turns_for_synthesis: usize, // Default: 3 — kurze Segmente überspringen ) -> SegmentSynthesisRes…` | Führt den generativen Syntheselauf über Turn-Segmente aus. | `src/synthesis_phase.rs` |
| `pub fn filter_candidates_with_transitivity_veto( turns: &[(DocId, Vec<f32>)], threshold: f32, agg_cfg: &AggregationConfig, ) -> Vec<(DocId, DocId)>` | Orchestriert die Transitivitätsprüfung unter Einhaltung der P24-Lokalität. | `src/transitivity_veto.rs` |
| `pub fn validate_transitivity_veto( node_a: &AggregationNode, node_b: &AggregationNode, node_c: &AggregationNode, threshold: f32, ) -> bool` | Reine Funktion zur Prüfung von Transitivitätsverletzungen bei Dreieckskonstellationen. | `src/transitivity_veto.rs` |


---

## `contextra-privacy`

| Attribut | Wert |
|---|---|
| **Ring** | 3 — Ring 3 — Orchestration & Cognition |
| **Status** | In Migration |
| **Zweck** | Cloud-Egress Privacy Gateway, PII-Vault, Surrogat-Tokenisierung und DLP. |
| **Unsafe-Code** | Nein (`#![forbid(unsafe_code)]`) |
| **Interne Abhängigkeiten** | `contextra-avv-generator`, `contextra-ports`, `contextra-types` |
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
pub mod entity_recognizer;
pub mod error;
pub mod guarded_payload;
pub mod processing_registry;
pub use audit_trace::{compute_audit_trace, extract_rule_id, EgressClassifierTrace};
pub use avv_generator::{generate_avv_draft, AvvContext};
pub use bulk_exfiltration_detector::{ BulkExfiltrationDetector, BulkExfiltrationOutcome, SessionId, };
pub use context_edit_audit::{ build_context_edit_audit_record, compute_record_hash, render_audit_line, verify_audit_chain, ContextEditAuditError, ContextEditAuditRecord, ContextEditKind, };
pub use egress_gateway::{ handle_cloud_query, handle_cloud_query_with_bulk_detector, handle_cloud_query_with_guard, process_cloud_response, CloudQueryRequest, CloudQueryResponse, CloudResponseRehydrator, DefaultEgressClassifier, EgressGateway, EgressGatewayBuilder, EgressGuardCheck, InjectionDetector, MAX_SEARCH_QUERY_BYTES, };
pub use egress_guard::{ EgressGuard, TextSearchEngine, TextSearchResult, DEFAULT_EGRESS_GUARD_MIN_BYTES, DEFAULT_EGRESS_GUARD_THRESHOLD, DEFAULT_EGRESS_GUARD_TIMEOUT, };
pub use egress_vault::NoOpRecognizer;
pub use egress_vault::{ BlockReason, BoxFuture, CompiledPattern, EgressClassification, EgressClassifier, EgressVault, EgressVaultError, EntityRecognizer, PolicyCategory, SurrogateVault, MAX_CLASSIFY_PAYLOAD_BYTES, };
pub use entity_recognizer::{ Gazetteer, PatternRecognizer, RuleBasedEntityRecognizer, DEFAULT_MAX_INPUT_LENGTH, };
pub use error::EgressError;
pub use guarded_payload::{GuardedPayload, Sanitized, Unsanitized};
pub use processing_registry::{ generate_registry, render_markdown, ProcessingActivityRecord, ProcessingRegistry, ProcessingRegistryExport, ProcessorRole, };
```

### Schnittstellen — Public Traits (6)

**`EgressClassifier`** — *Schnittstelle für Downstream-Crates (`contextra-mcp`, `contextra-router`).* (`src/egress_vault.rs`)

```rust
pub trait EgressClassifier : Send + Sync {
/// Klassifiziert einen Text-Payload für den Egress-Export.
    fn classify<'a>(&'a self, payload: &'a str) -> BoxFuture<'a, EgressClassification>;
}
```

**`EgressClassifierTrace`** — *Erweiterter Trait für Egress-Klassifikatoren mit kryptografischem Audit-Trace. Kombiniert die reguläre Klassifikation mit der deterministischen Trace-Berechnung über einen Zeitstempel-Port.* (`src/audit_trace.rs`)

```rust
pub trait EgressClassifierTrace : EgressClassifier {
/// Klassifiziert einen Payload und liefert sowohl das Klassifikationsergebnis
    /// als auch einen 32-Byte-BLAKE3-Hash-Trace.
    ///
    /// # Ablauf
    /// 1. Liest den aktuellen Zeitstempel in Nanosekunden aus `clock.now_unix_nanos()`.
    /// 2. Führt die asynchrone Klassifikation über `self.classify(payload)` aus.
    /// 3. Extrahiert die Regel-Kennung via [`extract_rule_id`].
    /// 4. Berechnet den Audit-Trace via [`compute_audit_trace`].
    /// 5. Gibt das Tupel `(Klassifikation, Trace)` zurück.
    ///
    /// # Nebenwirkungen
    /// Liest Zeit über den injizierten `Clock`-Port (Port-Entkopplung gemäß Regel 3).
    fn classify_with_trace<'a>(
        &'a self,
        payload: &'a str,
        clock: &'a dyn Clock,
    ) -> BoxFuture<'a, (EgressClassification, [u8; 32])>;
}
```

**`EgressGuardCheck`** — *Trait-Vertrag für Layer-4 k-NN Vektor-Ähnlichkeitsprüfungen gegen Bulk-Exfiltration.* (`src/egress_gateway.rs`)

```rust
pub trait EgressGuardCheck : Send + Sync {
/// Evaluates the outbound payload against vector index / similarity models.
    fn check<'a>(&'a self, payload: &'a str) -> BoxFuture<'a, EgressClassification>;
}
```

**`EntityRecognizer`** — *Trait zur Erkennung benannter Entitäten (NER) im Text ohne Abhängigkeit von Machine-Learning-Crates.* (`src/egress_vault.rs`)

```rust
pub trait EntityRecognizer : Send + Sync {
/// Recognizes entities in the text and returns byte ranges and category labels.
    fn recognize(&self, text: &str) -> Vec<(std::ops::Range<usize>, String)>;
}
```

**`InjectionDetector`** — *Trait zur Prüfung auf Prompt-Injection in eingehenden Cloud-Antworten.* (`src/egress_gateway.rs`)

```rust
pub trait InjectionDetector : Send + Sync {
fn detect(&self, text: &str) -> Option<String>;
}
```

**`TextSearchEngine`** — *Abstraktions-Trait für die Textähnlichkeitssuche zur Entkopplung von konkreten Datenbank-Engines.* (`src/egress_guard.rs`)

```rust
pub trait TextSearchEngine : Send + Sync {
fn search_text<'a>(
        &'a self,
        text: &'a str,
        limit: usize,
    ) -> BoxFuture<'a, Result<Vec<TextSearchResult>, String>>;
}
```

### Öffentliche Datentypen — Structs (24)

| Struct | Felder / Form | Beschreibung | Datei |
|---|---|---|---|
| `BulkExfiltrationDetector` | `{ /// Maximal zulässige Bytes pro Zeitfenster. pub max_bytes_per_window: usize, /// Dauer des gleitenden Zeitfensters. pub window: Duration, clock: Arc<dyn Clock… }` | Volumen- und Zeitfenster-basierter Rate-Limiter für Cloud-Egress-Anfragen (Layer 4). | `src/bulk_exfiltration_detector.rs` |
| `CloudQueryRequest` | `{ pub query: String, pub collection: Option<String>, pub max_results: Option<usize>, }` | Anfrage-Struktur für ausgehende Cloud-Queries. | `src/egress_gateway.rs` |
| `CloudQueryResponse` | `{ pub status: String, pub query: String, pub abstracted: bool, pub results: Vec<Value>, pub abstraction_notice: Option<String>, }` | Antwort-Struktur nach erfolgreicher Bereinigung und Durchlauf der Egress-Pipeline. | `src/egress_gateway.rs` |
| `CloudResponseRehydrator` | `{ vault_map: BTreeMap<String, String>, }` | Rehydrierer zur sicheren Rückübersetzung von Surrogat-Tokens in Cloud-Antworten. | `src/egress_gateway.rs` |
| `CompiledPattern` | `{ /// Bezeichner oder Originalmuster. pub name: String, /// Kompiliertes Regex-Objekt. pub regex: regex::Regex, }` | Ein vorkompiliertes Regex-Muster für die Egress-Klassifikation. | `src/egress_vault.rs` |
| `ContextEditAuditRecord` | `{ /// BLAKE3-Hash des vorherigen Audit-Records in der Kette (32 Bytes). pub previous_hash: [u8; 32], /// Eigenadresse: BLAKE3-Hash dieses Datensatzes über alle F… }` | Nachvollziehbarer, kryptografisch verketteter Audit-Datensatz für eine Änderung am temporären Agenten-Kontext. **Sicherheitsgarantie:** Enthält keine… | `src/context_edit_audit.rs` |
| `ContextEditInput` | `{ /// Eindeutige Kennung des ausführenden Agenten-Tasks. pub task_id: String, /// Mandanten-Identifikator (Sicherheitstrennung). pub tenant_id: TenantId, /// Art… }` | Eingabe-Parameter zur Erstellung eines Audit-Datensatzes für eine Kontext-Änderung. | `src/context_edit_audit.rs` |
| `DefaultEgressGuard` | `(tuple/unit)` | Standardmäßiger Pass-Through Guard, falls kein separater L4-Vektor-Guard konfiguriert ist. | `src/egress_gateway.rs` |
| `EgressGateway` | `{ classifier: Arc<dyn EgressClassifier>, egress_guard: Arc<dyn EgressGuardCheck>, recognizer: Arc<dyn EntityRecognizer>, egress_vault: Arc<EgressVault>, bulk_det… }` | Einheitliches Egress-Gateway zur Durchsetzung der 5-stufigen Schutzpipeline: 1. Policy/Guard check 2. Entity recognition 3. Surrogate replacement 4.… | `src/egress_gateway.rs` |
| `EgressGatewayBuilder` | `{ classifier: Option<Arc<dyn EgressClassifier>>, egress_guard: Option<Arc<dyn EgressGuardCheck>>, recognizer: Option<Arc<dyn EntityRecognizer>>, egress_vault: Op… }` | Builder zur typsicheren Konstruktion einer [`EgressGateway`]-Instanz. | `src/egress_gateway.rs` |
| `EgressGuard` | `{ search_engine: Arc<dyn TextSearchEngine>, threshold: f32, min_bytes: usize, timeout: Duration, }` | Layer-4 EgressGuard zur Erkennung und Blockierung von Bulk-Exfiltrationen. Baut auf einem Text-Sucher (`TextSearchEngine`) auf. Outbound-Payloads mit… | `src/egress_guard.rs` |
| `EgressVault` | `{ patterns: Arc<Vec<CompiledPattern>>, regex_set: Arc<regex::RegexSet>, timeout: Duration, surrogate_vault: Arc<SurrogateVault>, }` | Cloud Egress Vault zur Abhandlung und Bündelung von Layer-1-Klassifikationen. | `src/egress_vault.rs` |
| `Gazetteer` | `{ terms: BTreeSet<String>, }` | Erweiterungspunkt für benutzerdefinierte Begriff-, Namens- oder Unternehmenslisten (Wörterbuch-Abgleich). | `src/entity_recognizer.rs` |
| `GuardedPayload` | `{ pub(crate) inner: String, pub(crate) session_id: String, _state: PhantomData<S>, }` | Type-State-Wrapper für Cloud-Egress-Payloads. Ein `GuardedPayload<Sanitized>` kann NUR durch den vollständigen Durchlauf aller 5 Egress-Guard-Layer k… | `src/guarded_payload.rs` |
| `NoOpRecognizer` | `(tuple/unit)` | No-Op-Implementierung von [`EntityRecognizer`], die keine Entitäten erkennt (Null-Objekt-Muster). | `src/egress_vault.rs` |
| `ProcessingActivityRecord` | `{ /// Bezeichnung der Verarbeitungstätigkeit. pub activity_name: String, /// Rolle (Verantwortlicher oder Auftragsverarbeiter). pub controller_or_processor_role:… }` | Ein Datensatz einer Verarbeitungstätigkeit gemäß Art. 30 Abs. 1 DSGVO. | `src/processing_registry.rs` |
| `ProcessingRegistry` | `{ records: Vec<ProcessingActivityRecord>, }` | Unveränderliches, append-only Verzeichnis von Verarbeitungstätigkeiten. | `src/processing_registry.rs` |
| `ProcessingRegistryExport` | `{ /// Liste der erfassten Verarbeitungstätigkeiten. pub records: Vec<ProcessingActivityRecord>, }` | Strukturierte Export-Repräsentation des Verzeichnisses von Verarbeitungstätigkeiten. | `src/processing_registry.rs` |
| `RuleBasedEntityRecognizer` | `{ gazetteer: Gazetteer, max_input_length: usize, }` | Regelbasierter PII-Erkenner ohne externe Regex-Laufzeitabhängigkeit. Erkennt Kategorien personenbezogener Daten: EMAIL, IBAN, BIC, PHONE, CREDIT_CARD… | `src/entity_recognizer.rs` |
| `Sanitized` | `(tuple/unit)` | Marker-Typ: Payload hat alle 5 Egress-Guard-Layer erfolgreich durchlaufen und ist bereinigt. | `src/guarded_payload.rs` |
| `SessionId` | `(tuple/unit)` | Eindeutige Sitzungs-Kennung (SessionId) zur Gruppierung von Egress-Rate-Limiting-Buckets. | `src/bulk_exfiltration_detector.rs` |
| `SurrogateVault` | `{ session_salt: [u8; 16], map: Mutex<BTreeMap<String, String>>, }` | Sitzungsgebundener Tresor zur bidirektionalen Abbildung zwischen Klartext-PII und Surrogat-Tokens. Alle kryptografischen Schlüssel, Salts und Zuordnu… | `src/egress_vault.rs` |
| `TextSearchResult` | `{ pub id: String, pub score: f32, }` | Vereinfachtes Kandidaten-Ergebnis einer Text- bzw. Vektorähnlichkeitssuche. | `src/egress_guard.rs` |
| `Unsanitized` | `(tuple/unit)` | Marker-Typ: Payload hat noch nicht alle Egress-Guard-Layer durchlaufen und gilt als unbereinigt. | `src/guarded_payload.rs` |

### Öffentliche Datentypen — Enums (9)

| Enum | Varianten | Beschreibung | Datei |
|---|---|---|---|
| `BlockReason` | `/// Ein sensibles Muster (PII, Secret, API-Key etc.) wurde erkannt. SensitivePattern(String), /// Richtlinie untersagt die Übertragung (lokal oder allgemein). PolicyDenied(String), /// Spezifische Cl…` | Grund für das Blockieren einer Egress-Anfrage. | `src/egress_vault.rs` |
| `BulkExfiltrationOutcome` | `/// Das kumulierte Datenvolumen liegt innerhalb der zulässigen Grenzen des aktiven Zeitfensters. Allow, /// Das kumulierte Datenvolumen überschreitet das Limit im aktiven Zeitfenster; Abfrage wird bl…` | Ergebnis der Auswertung des ausgehenden Datenvolumens gegen die konfigurierten Sitzungslimits. | `src/bulk_exfiltration_detector.rs` |
| `ContextEditAuditError` | `/// Manipulationserkennung: Die Audit-Kette ist ab dem angegebenen Index fehlerhaft oder manipuliert. #[error("Audit chain tampered or invalid at index {index}")] ChainTampered { index: usize },` | Lokaler Fehlertyp für Audit-Ketten-Verifizierungsfehler. | `src/context_edit_audit.rs` |
| `ContextEditKind` | `/// Anfügen von neuem Inhalt an den Kontext. Append, /// Ersetzen von bestehendem Inhalt im Kontext. Replace, /// Entfernen von Inhalt aus dem Kontext. Remove, /// Zurücksetzen des Kontexts auf einen…` | Art der Kontext-Änderung am temporären Agenten-Scratchpad. | `src/context_edit_audit.rs` |
| `EgressClassification` | `/// Übertragung zulässig. Allow, /// Übertragung blockiert mit Grund. Block(BlockReason),` | Klassifikationsergebnis der Egress-Prüfung. | `src/egress_vault.rs` |
| `EgressError` | `/// Ungültige Eingabeparameter oder Konfiguration. #[error("Invalid parameter: {0}")] InvalidParams(String), /// Richtlinienverstoß (Blockierung durch Richtlinie). #[error("Policy violation: {0}")] P…` | Fehlertypen des Egress-Gateways und der DLP-Prüfung. Signalisiert spezifische Fehlerzustände bei der Analyse und Weiterleitung von Daten an externe C… | `src/error.rs` |
| `EgressVaultError` | `#[error("Invalid pattern '{pattern}': {reason}")] InvalidPattern { pattern: String, reason: String }, #[error("Payload size exceeds limit: {size} bytes > {limit} limit")] PayloadTooLarge { size: usiz…` | Fehler beim Erstellen oder Laden des EgressVaults. | `src/egress_vault.rs` |
| `PolicyCategory` | `/// Rein lokaler Lese- oder Speicherzugriff innerhalb der Vertrauensgrenze. LocalAccess, /// Externe Cloud-Egress-Übertragung (Daten verlassen die lokale Vertrauensgrenze). CloudEgress,` | Policy-Kategorie für Audit-Logging und Zugriffskontrolle. | `src/egress_vault.rs` |
| `ProcessorRole` | `/// Verantwortlicher (Art. 4 Nr. 7 DSGVO). Controller, /// Auftragsverarbeiter (Art. 4 Nr. 8 DSGVO). Processor,` | Rolle bezüglich der Verarbeitungstätigkeit gemäß DSGVO. | `src/processing_registry.rs` |

### Typ-Aliase (3)

| Alias | Definition | Beschreibung |
|---|---|---|
| `BoxFuture` | `type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>` | Typalias für gepinnte, sende-fähige dynamische Zukunfts-Futures. |
| `DefaultEgressClassifier` | `type DefaultEgressClassifier= EgressVault` | Standardmäßiger Klassifikator auf Basis von [`EgressVault`]. |
| `PatternRecognizer` | `type PatternRecognizer= RuleBasedEntityRecognizer` | Alias für [`RuleBasedEntityRecognizer`] als standardmäßiger Mustererkenner. |

### Konstanten (7)

| Konstante | Typ | Beschreibung |
|---|---|---|
| `DEFAULT_EGRESS_GUARD_MIN_BYTES` | `usize` | Minimale Byte-Länge des Payloads, ab der die Ähnlichkeitsprüfung durchgeführt wird (128 Bytes). |
| `DEFAULT_EGRESS_GUARD_THRESHOLD` | `f32` | Standard-Schwellwert für Cosine-Similarity-Übereinstimmung (0.85 = 85 % Ähnlichkeit). |
| `DEFAULT_EGRESS_GUARD_TIMEOUT` | `Duration` | Standard-Timeout für EgressGuard Vector-Search (200 ms). |
| `DEFAULT_MAX_INPUT_LENGTH` | `usize` | Standardmäßige maximale Eingabelänge in Bytes für die Entitätserkennung (65.536 Bytes = 64 KiB). |
| `DEFAULT_TIMEOUT` | `Duration` | Standard-Timeout für Klassifikationsprüfungen (100 ms). |
| `MAX_CLASSIFY_PAYLOAD_BYTES` | `usize` | Maximale zulässige Payload-Länge in Bytes für Layer-1-Klassifikation (64 KiB = 65.536 Bytes). |
| `MAX_SEARCH_QUERY_BYTES` | `usize` | Maximale Suchanfragengröße in Bytes (64 KiB = 65.536 Bytes). |

### Öffentliche Funktionen (91)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn compute_audit_trace(payload: &str, rule_id: &str, nanos: u64) -> [u8` | Berechnet einen deterministischen 32-Byte-BLAKE3-Hash-Trace für eine Klassifikationsentscheidung. Segmente werden in fester Byte-Reihenfolg… | `src/audit_trace.rs` |
| `pub fn extract_rule_id(classification: &EgressClassification) -> String` | Extrahiert eine stabile, nicht-leere Regel-Kennung aus einer [`EgressClassification`]. # Ablauf 1. Bei `EgressClassification::Allow` wird d… | `src/audit_trace.rs` |
| `pub fn generate_avv_draft(ctx: &contextra_avv_generator::AvvContext) -> String` | Generiert einen AVV-Entwurf im Markdown-Format anhand des übergebenen Kontexts. # Ablauf 1. Übergibt die Kontextstruktur an `contextra_avv_… | `src/avv_generator.rs` |
| `pub fn new(max_bytes_per_window: usize, window: Duration) -> Self` | Erstellt einen neuen `BulkExfiltrationDetector` mit Standard-Systemzeit-Clock. # Parameter * `max_bytes_per_window`: Maximale Byteanzahl im… | `src/bulk_exfiltration_detector.rs` |
| `pub fn record_and_check( &self, session: SessionId, payload_len: usize, ) -> BulkExfiltrationOutcome` | Registriert `payload_len` Bytes für `session` und bewertet, ob das Limit im gleitenden Fenster überschritten wurde. Thread-sicher für neben… | `src/bulk_exfiltration_detector.rs` |
| `pub fn record_and_check_at_nanos( &self, session: SessionId, payload_len: usize, now_nanos: u64, ) -> BulkExfiltrationOutcome` | Interner Hilfsablauf zur Auswertung an einem expliziten Nanosekunden-Zeitstempel. # Ablauf 1. Sucht den Sitzungs-Eintrag in der `scc::HashM… | `src/bulk_exfiltration_detector.rs` |
| `pub fn record_and_check_with_clock( &self, session: SessionId, payload_len: usize, clock: &dyn Clock, ) -> BulkExfiltrationOutcome` | Registriert Bytes für eine Sitzung unter Verwendung einer explizit übergebenen [`Clock`]. | `src/bulk_exfiltration_detector.rs` |
| `pub fn with_clock( max_bytes_per_window: usize, window: Duration, clock: Arc<dyn Clock>, ) -> Self` | Erstellt einen neuen `BulkExfiltrationDetector` mit einem spezifischen [`Clock`]-Port. # Parameter * `max_bytes_per_window`: Maximale Bytea… | `src/bulk_exfiltration_detector.rs` |
| `pub fn build_context_edit_audit_record( previous_hash: [u8; 32], input: ContextEditInput, clock: &dyn Clock, ) -> ContextEditAuditRecord` | Konstruiert deterministisch einen neuen [`ContextEditAuditRecord`] mit Hash-Verkettung. Der Zeitstempel wird ausschließlich über den injizi… | `src/context_edit_audit.rs` |
| `pub fn compute_record_hash( previous_hash: [u8; 32], input: &ContextEditInput, recorded_at_tx: u64, ) -> [u8` | Berechnet den deterministischen BLAKE3-Hash über alle Datenfelder eines Audit-Records. # Ablauf 1. Hash-Initialisierung mit `blake3::Hasher… | `src/context_edit_audit.rs` |
| `pub fn render_audit_line(record: &ContextEditAuditRecord) -> String` | Formatiert einen Audit-Datensatz in eine einzeilige, menschenlesbare Log-Zeile. ### Format `"task={task_id} tenant={tenant_id} kind={kind:?… | `src/context_edit_audit.rs` |
| `pub fn verify_audit_chain(chain: &[ContextEditAuditRecord]) -> Result<(), ContextEditAuditError>` | Verifiziert die Integrität und Hash-Verkettung einer Reihe von Audit-Records. # Ablauf 1. Initialisiert den erwarteten Vorgängerhash mit Nu… | `src/context_edit_audit.rs` |
| `pub fn build(self) -> Result<EgressGateway, EgressError>` | Builds the `EgressGateway`, strictly enforcing that mandatory components (including `egress_guard`) are present. | `src/egress_gateway.rs` |
| `pub fn builder() -> EgressGatewayBuilder` | — | `src/egress_gateway.rs` |
| `pub fn check_bulk_exfiltration( detector: &BulkExfiltrationDetector, session: SessionId, payload: &str, ) -> Result<(), EgressError>` | Überprüft eine Anfrage gegen den Bulk-Exfiltrations-Detektor. | `src/egress_gateway.rs` |
| `pub async fn handle_cloud_query( request: CloudQueryRequest, classifier: &dyn EgressClassifier, ) -> Result<CloudQueryResponse, EgressError>` | Verarbeitet eine ausgehende Cloud-Query über die Standard-Schutz-Pipeline. | `src/egress_gateway.rs` |
| `pub async fn handle_cloud_query_scoped( request: TenantScoped<CloudQueryRequest>, expected_tenant_id: &TenantId, classifier: &dyn EgressClassifier, ) -> Result<CloudQueryResponse, EgressError>` | Verarbeitet eine mandantengebundene Cloud-Query ([`TenantScoped`]). | `src/egress_gateway.rs` |
| `pub async fn handle_cloud_query_scoped_with_bulk_detector( request: TenantScoped<CloudQueryRequest>, expected_tenant_id: &TenantId, classifier: &dyn EgressClassifier, egress_guard: Option<&dyn EgressGuardCheck>, bulk_detector: Option<(&BulkExfiltrationDetecto…` | Vollständige Prüfung inklusive Mandantenisolation, L4-Guard und Bulk-Exfiltrations-Detektor. | `src/egress_gateway.rs` |
| `pub async fn handle_cloud_query_scoped_with_guard( request: TenantScoped<CloudQueryRequest>, expected_tenant_id: &TenantId, classifier: &dyn EgressClassifier, egress_guard: Option<&dyn EgressGuardCheck>, ) -> Result<CloudQueryResponse, EgressError>` | Verarbeitet eine mandantengebundene Cloud-Query mit zusätzlichem Layer-4 Vektor-Guard. | `src/egress_gateway.rs` |
| `pub async fn handle_cloud_query_with_bulk_detector( request: CloudQueryRequest, classifier: &dyn EgressClassifier, egress_guard: Option<&dyn EgressGuardCheck>, bulk_detector: Option<(&BulkExfiltrationDetector, SessionId)>, ) -> Result<CloudQueryResponse, Egre…` | Führt die Cloud-Query-Prüfung mit L4-Guard und Bulk-Exfiltrations-Detektor durch. | `src/egress_gateway.rs` |
| `pub async fn handle_cloud_query_with_guard( request: CloudQueryRequest, classifier: &dyn EgressClassifier, egress_guard: Option<&dyn EgressGuardCheck>, ) -> Result<CloudQueryResponse, EgressError>` | Verarbeitet eine Cloud-Query mit zusätzlichem Layer-4 Vektor-Guard. | `src/egress_gateway.rs` |
| `pub fn new() -> Self` | — | `src/egress_gateway.rs` |
| `pub fn new(vault_map: impl IntoIterator<Item = (String, String)>) -> Self` | Creates a new `CloudResponseRehydrator` with the provided surrogate-to-entity map. | `src/egress_gateway.rs` |
| `pub fn pii_vault_forces_crypto_shred(is_pii_match: bool, is_memory_only: bool) -> bool` | Entscheidet anhand von PII-Match und Speichermodus, ob kryptografisches Schreddern erzwungen wird. | `src/egress_gateway.rs` |
| `pub async fn process( &self, request: TenantScoped<CloudQueryRequest>, expected_tenant_id: &TenantId, ) -> Result<CloudQueryResponse, EgressError>` | Executes the standard, mandatory 5-stage egress pipeline for a tenant-scoped request. | `src/egress_gateway.rs` |
| `pub async fn process_cloud_response( raw_response: &str, injection_guard: &dyn InjectionDetector, rehydrator: &CloudResponseRehydrator, ) -> Result<String, EgressError>` | Verarbeitet die Antwort einer Cloud-Query (Rehydrierung und Prompt-Injection-Erkennung). | `src/egress_gateway.rs` |
| `pub async fn process_with_components( &self, request: TenantScoped<CloudQueryRequest>, expected_tenant_id: &TenantId, classifier: &dyn EgressClassifier, egress_guard: Option<&dyn EgressGuardCheck>, bulk_detector: Option<(&BulkExfiltrationDetector, SessionId)>…` | Executes the standard 5-stage egress pipeline with optional custom classifier, guard, or bulk detector overrides: 1. Policy/Guard check ->… | `src/egress_gateway.rs` |
| `pub fn rehydrate(&self, cloud_response_text: &str) -> String` | Rehydrates surrogate tokens matching `[USER_ENTITY_[0-9a-fA-F]{4}]` back to original entity values. Unknown or missing surrogate tokens rem… | `src/egress_gateway.rs` |
| `pub fn resolve_effective_kv_delete_mode<D, K>( is_pii_match: bool, durability_mode: D, preset_kv_delete_mode: K, crypto_shred_mode: K, ) -> K where D: std::fmt::Debug, K: Copy,` | Bestimmt den effektiven Löschmodus (Shredding vs. Tombstone) unter Berücksichtigung von PII-Vorgaben. | `src/egress_gateway.rs` |
| `pub fn with_bulk_detector( mut self, detector: Arc<BulkExfiltrationDetector>, session_id: SessionId, ) -> Self` | — | `src/egress_gateway.rs` |
| `pub fn with_classifier(mut self, classifier: Arc<dyn EgressClassifier>) -> Self` | — | `src/egress_gateway.rs` |
| `pub fn with_clock(mut self, clock: Arc<dyn Clock>) -> Self` | — | `src/egress_gateway.rs` |
| `pub fn with_egress_guard(mut self, guard: Arc<dyn EgressGuardCheck>) -> Self` | — | `src/egress_gateway.rs` |
| `pub fn with_egress_vault(mut self, vault: Arc<EgressVault>) -> Self` | — | `src/egress_gateway.rs` |
| `pub fn with_recognizer(mut self, recognizer: Arc<dyn EntityRecognizer>) -> Self` | — | `src/egress_gateway.rs` |
| `pub fn with_surrogate_vault(mut self, surrogate_vault: Arc<SurrogateVault>) -> Self` | — | `src/egress_gateway.rs` |
| `pub async fn check(&self, payload: &str) -> EgressClassification` | Prüft einen Egress-Payload auf mögliche Bulk-Exfiltration gegen die Collection. Gemäß **v17 Teil 10.3** (Priorität P1, "Fail-Closed-Randfal… | `src/egress_guard.rs` |
| `pub async fn check_scoped( &self, payload: contextra_types::TenantScoped<&str>, expected_tenant_id: &contextra_types::TenantId, ) -> EgressClassification` | Prüft einen an einen Mandanten gebundenen Payload ([`contextra_types::TenantScoped`]) mit Mandantenisolationsprüfung. Liefert `Block(BlockR… | `src/egress_guard.rs` |
| `pub fn min_bytes(&self) -> usize` | Gibt die minimale Payload-Länge in Bytes zurück. | `src/egress_guard.rs` |
| `pub fn new(search_engine: Arc<dyn TextSearchEngine>, threshold: f32, min_bytes: usize) -> Self` | Erstellt eine neue `EgressGuard`-Instanz mit Standard-Timeout (200 ms). | `src/egress_guard.rs` |
| `pub fn threshold(&self) -> f32` | Gibt den gesetzten Schwellwert für Ähnlichkeitsprüfungen zurück. | `src/egress_guard.rs` |
| `pub fn timeout(&self) -> Duration` | Gibt das konfigurierte Abfrage-Timeout zurück. | `src/egress_guard.rs` |
| `pub fn with_timeout(mut self, timeout: Duration) -> Self` | Setzt ein benutzerdefiniertes Timeout für die Index-Abfrage. | `src/egress_guard.rs` |
| `pub fn category(&self) -> PolicyCategory` | Gibt die zugehörige Policy-Kategorie des Blockierungsgrunds zurück. | `src/egress_vault.rs` |
| `pub async fn classify_layer1( payload: &str, patterns: &[CompiledPattern], timeout: Duration, ) -> EgressClassification` | Führt eine Layer-1-Regex-Klassifikation auf dem Payload mit neuem Task und hartem Timeout aus. Invariante: **Fail-Closed**. Bei Timeout, Pa… | `src/egress_vault.rs` |
| `pub async fn classify_layer1_arc( payload: &str, patterns: Arc<Vec<CompiledPattern>>, regex_set: Arc<regex::RegexSet>, timeout: Duration, ) -> EgressClassification` | Optimierter Ausführungspfad für `classify_layer1` unter Wiederverwendung von vorkompilierten `Arc`-Mengen. | `src/egress_vault.rs` |
| `pub async fn classify_scoped( &self, payload: contextra_types::TenantScoped<&str>, expected_tenant_id: &contextra_types::TenantId, ) -> EgressClassification` | Classifies a `TenantScoped` payload for egress export only if the bound `TenantId` matches `expected_tenant_id`. If a tenant scope violatio… | `src/egress_vault.rs` |
| `pub fn default_patterns() -> Vec<String>` | Standard-DLP-Muster für Egress-Klassifikationen (Secrets, API-Keys, PII, E-Mail). | `src/egress_vault.rs` |
| `pub fn generate_surrogate(&self, entity_text: &str) -> Result<String, EgressVaultError>` | Generates a session-stable surrogate identifier for `entity_text` and stores the surrogate -> entity mapping. # Backward Compatibility & To… | `src/egress_vault.rs` |
| `pub fn get_entity(&self, surrogate: &str) -> Option<String>` | Retrieves the original entity text for a given surrogate key if present. | `src/egress_vault.rs` |
| `pub fn is_empty(&self) -> bool` | Returns true if the vault contains no surrogate mappings. | `src/egress_vault.rs` |
| `pub fn len(&self) -> usize` | Returns the current count of stored surrogate mappings. | `src/egress_vault.rs` |
| `pub fn new(session_salt: [u8; 16]) -> Self` | Erstellt einen neuen `SurrogateVault` mit dem übergebenen 16-Byte Sitzungssalz. | `src/egress_vault.rs` |
| `pub fn new(name: impl Into<String>, pattern: &str) -> Result<Self, EgressVaultError>` | Erstellt ein neues `CompiledPattern`. | `src/egress_vault.rs` |
| `pub fn new(patterns: Vec<String>) -> Result<Self, EgressVaultError>` | Erstellt eine neue `EgressVault`-Instanz aus einer Liste von Regex-Patterns. Jedem Pattern wird eine opake Regel-ID ("R-001", "R-002", ...)… | `src/egress_vault.rs` |
| `pub fn patterns(&self) -> &[CompiledPattern]` | Gibt die kompilierten Muster zurück. | `src/egress_vault.rs` |
| `pub fn policy_category(&self) -> PolicyCategory` | Gibt die Policy-Kategorie dieser Vault-Instanz für Cloud-Egress-Klassifizierungen zurück. | `src/egress_vault.rs` |
| `pub fn random_salt() -> Self` | Erstellt einen neuen `SurrogateVault` mit einem kryptografisch sicheren Zufallssalz. | `src/egress_vault.rs` |
| `pub fn sanitize_and_vault( &self, payload: &str, recognizer: &dyn EntityRecognizer, ) -> Result<(String, usize), EgressVaultError>` | Tokenisiert und ersetzt erkannte strukturierte Regex-Muster sowie unstrukturierte NER-Entitäten durch sitzungsstabile Surrogate und speiche… | `src/egress_vault.rs` |
| `pub fn sanitize_and_vault_scoped( &self, payload: contextra_types::TenantScoped<&str>, expected_tenant_id: &contextra_types::TenantId, recognizer: &dyn EntityRecognizer, ) -> Result<(contextra_types::TenantScoped<String>, usize), EgressVaultError>` | Tokenizes and replaces recognized entities in a `TenantScoped` payload only if the bound `TenantId` matches `expected_tenant_id`. Re-wraps… | `src/egress_vault.rs` |
| `pub fn surrogate_vault(&self) -> &Arc<SurrogateVault>` | Gibt eine Referenz auf den verknüpften `SurrogateVault` zurück. | `src/egress_vault.rs` |
| `pub fn timeout(&self) -> Duration` | Gibt das gesetzte Timeout zurück. | `src/egress_vault.rs` |
| `pub fn try_default() -> Result<Self, EgressVaultError>` | Versucht, eine `EgressVault`-Instanz mit den Standard-DLP-Mustern zu erstellen. | `src/egress_vault.rs` |
| `pub fn with_surrogate_vault(mut self, surrogate_vault: Arc<SurrogateVault>) -> Self` | Setzt einen benutzerdefinierten `SurrogateVault`. | `src/egress_vault.rs` |
| `pub fn with_timeout(mut self, timeout: Duration) -> Self` | Setzt ein benutzerdefiniertes Timeout für Klassifikationsabfragen. | `src/egress_vault.rs` |
| `pub fn add_gazetteer_term(mut self, term: impl Into<String>) -> Self` | Ergänzt einen Begriff für das Verzeichnis über ein Builder-Muster. | `src/entity_recognizer.rs` |
| `pub fn add_term(&mut self, term: impl Into<String>)` | Fügt dem Verzeichnis einen Begriff hinzu (wird für case-insensitiven Abgleich in Kleinbuchstaben normalisiert). | `src/entity_recognizer.rs` |
| `pub fn is_empty(&self) -> bool` | Prüft, ob das Begriffsverzeichnis leer ist. | `src/entity_recognizer.rs` |
| `pub fn max_input_length(&self) -> usize` | Liefert die Obergrenze für Eingabetexte in Bytes. | `src/entity_recognizer.rs` |
| `pub fn new() -> Self` | Erstellt eine neue leere [`Gazetteer`]-Instanz. | `src/entity_recognizer.rs` |
| `pub fn new() -> Self` | Erstellt eine neue Instanz von [`RuleBasedEntityRecognizer`] mit Standard-Längenlimit und leerem Gazetteer. | `src/entity_recognizer.rs` |
| `pub fn recognize(&self, text: &str) -> Vec<(Range<usize>, String)>` | Führt die Erkennung auf dem übergebenen Text aus und liefert Byte-Bereiche und Kategorielabels. | `src/entity_recognizer.rs` |
| `pub fn recognize_bounded(&self, text: &str) -> Result<Vec<(Range<usize>, String)>, EgressError>` | Führt die Erkennung mit strikter Längenprüfung aus und liefert `Err(EgressError::InvalidParams)` bei Überschreitung. | `src/entity_recognizer.rs` |
| `pub fn with_gazetteer(gazetteer: Gazetteer) -> Self` | Erstellt einen Erkenner mit einem vorkonfigurierten Begriffsverzeichnis ([`Gazetteer`]). | `src/entity_recognizer.rs` |
| `pub fn with_max_input_length(mut self, limit: usize) -> Self` | Konfiguriert das maximale Längenlimit in Bytes zur Begrenzung von DoS-Risiken. | `src/entity_recognizer.rs` |
| `pub fn internal_error(msg: impl Into<String>) -> Self` | Konstruiert einen `EgressError::InternalError`. # Parameter * `msg`: Interne System-Fehlermeldung. | `src/error.rs` |
| `pub fn invalid_params(msg: impl Into<String>) -> Self` | Konstruiert einen `EgressError::InvalidParams`. # Parameter * `msg`: Fehlermeldung oder Beschreibung der ungültigen Parameter. | `src/error.rs` |
| `pub fn policy_violation(msg: impl Into<String>) -> Self` | Konstruiert einen `EgressError::PolicyViolation`. # Parameter * `msg`: Beschreibung des verletzten Richtlinien-Kriteriums. | `src/error.rs` |
| `pub fn from_sanitized(sanitized: String, session_id: String) -> Self` | Konstruiert einen bereinigten Payload nach erfolgreicher Bereinigung. # Ablauf Nur von `egress_gateway.rs` nach erfolgreichem Durchlauf all… | `src/guarded_payload.rs` |
| `pub fn into_inner(self) -> String` | Extrahiert den bereinigten String aus dem Schutzbehälter für den Versand. # Ablauf Konsumiert die Instanz und gibt das interne `String`-Obj… | `src/guarded_payload.rs` |
| `pub fn new(raw: String, session_id: String) -> Self` | Erstellt einen neuen unsanitizierten Payload (Eingang aus MCP-Client oder interner Engine). # Ablauf 1. Übernimmt den rohen Eingabestring `… | `src/guarded_payload.rs` |
| `pub fn session_id(&self) -> &str` | Gibt eine Referenz auf die zugeordnete Sitzungs-ID (`session_id`) zurück. | `src/guarded_payload.rs` |
| `pub fn append_record(&mut self, mut record: ProcessingActivityRecord, clock: &dyn contextra_ports::Clock)` | Fügt einen Datensatz dem Verzeichnis hinzu (append-only) und belegt den Erstellungs-Zeitstempel über die injizierte Uhr. | `src/processing_registry.rs` |
| `pub fn as_str(&self) -> &'static str` | Liefert die deutschsprachige Bezeichnung der Rolle. | `src/processing_registry.rs` |
| `pub fn export(&self) -> ProcessingRegistryExport` | Exportiert das Verzeichnis in eine `ProcessingRegistryExport`-Struktur. | `src/processing_registry.rs` |
| `pub fn filter_by_tenant(&self, tenant_id: TenantId) -> Self` | Filtert alle Verarbeitungstätigkeiten nach der angegebenen Mandanten-ID (Art. 15 DSGVO Auskunftsersuchen). Garantiert strikte Mandantentren… | `src/processing_registry.rs` |
| `pub fn generate_registry(records: &[ProcessingActivityRecord]) -> ProcessingRegistryExport` | Erzeugt aus einer Reihe von Datensätzen eine strukturierte Export-Repräsentation. | `src/processing_registry.rs` |
| `pub fn new() -> Self` | Erstellt ein neues leeres `ProcessingRegistry`. | `src/processing_registry.rs` |
| `pub fn new(records: Vec<ProcessingActivityRecord>) -> Self` | Erstellt eine neue Export-Instanz mit den angegebenen Datensätzen. | `src/processing_registry.rs` |
| `pub fn records(&self) -> &[ProcessingActivityRecord]` | Liefert die Datensätze des Verzeichnisses. | `src/processing_registry.rs` |
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

### Architekturbeschreibung

# contextra-router  **Ring:** Ring 3 (Applikations- und Koordinationsschicht / Peripherie). **Zweck:** Intelligentes Routing von Anfragen an geeignete Small Language Models (SLM). Bewertet Modell-Profile (z. B. Token-Limits, Domänen-Affinitäten), kalibriert Konfidenzwerte über konforme Vorhersagen (Conformal Prediction) und steuert Multi-Armed-Bandit-Verfahren (LinUCB / Thompson Sampling).  **Abhängigkeiten:** - Upstream: `contextra-types`, `contextra-ports`, `contextra-wire`, `contextra-adapt`, `contextra-privacy`. - Downstream: Wird eingebunden von `contextra-agent` und `contextra-mcp`.  **Hauptkomponenten:** - [`RouterEngine`]: Primärer Routing-Endpunkt zur Entscheidung für das beste Modell. - [`SlmProfile`]: Spezifikation der Fähigkeiten und Begrenzungen eines Modells. - [`ConformalCalibrator`]: Statistische Fehlerüberwachung und Kalibrierung. - [`ArmRegistry`]: Zuordnung von Modellprofilen zu Bandit-Armen. - [`dispatch_to_slm`]: Ausführung des eigentlichen SLM-Aufrufs.

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
pub use dispatch::dispatch_to_slm;
pub use outcome::{DecisionId, DecisionIdGenerator, RoutingOutcome};
pub use profile::{ConformalCalibrator, ProfileCalibrator, SlmProfile};
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
| `bandit-routing` | `["egress-sherman-morrison"]` |
| `cloud-egress-guard` | `[]` |
| `egress-sherman-morrison` | `[]` |
| `flow-corrected-thompson` | `["contextra-adapt/flow-corrected-thompson"]` |

### Öffentliche Datentypen — Structs (6)

| Struct | Felder / Form | Beschreibung | Datei |
|---|---|---|---|
| `ArmRegistry` | `{ arms: [RetrievalStrategy; 5], }` | Verlustfreie, deterministische Abbildung Arm-Index <-> RetrievalStrategy. Lebt in `contextra-router` (wo Arme registriert und ausgewertet werden), NI… | `src/arm_registry.rs` |
| `DecisionId` | `(tuple/unit)` | Eindeutige ID einer Routing-Entscheidung. Wird von `RouterEngine::route()` erzeugt und vergeben und später von `RouterEngine::record_outcome()` konsu… | `src/outcome.rs` |
| `DecisionIdGenerator` | `{ next: AtomicU64, }` | Instanzgebundener Generator für monoton steigende [`DecisionId`]s (Spec §3 P29). Garantiert Thread-Sicherheit über atomare Inkrement-Operationen ohne… | `src/outcome.rs` |
| `ProfileCalibrationState` | `{ /// Gesamtzahl der Routing-Entscheidungen für dieses Profil. pub times_selected: u64, /// Kumulierte Routing-Konfidenz (best_score / second_best_score). /// Hö… }` | Laufzeit-Zustand der Kalibrierung eines Profils inklusive Auswahlhäufigkeit und Konfidenz. | `src/profile.rs` |
| `ProfileCalibrator` | `{ /// Target error rate (e.g., 0.05 for 95% coverage). pub alpha: f32, /// Adaptation rate for online quantile updates. pub gamma: f32, /// Current calibrated co… }` | Konformaler Kalibrator nach Gibbs & Candès zur dynamischen Schwellwertanpassung. | `src/profile.rs` |
| `SlmProfile` | `{ /// Identifier name of the SLM (e.g. "coding-slm", "docs-slm"). pub name: String, /// MCP endpoint address or URI for client communication. pub mcp_endpoint: S… }` | Repräsentiert das Profil eines Small Language Models (SLM) mit Token-Budget und Domänen-Zuständigkeiten. | `src/profile.rs` |

### Öffentliche Datentypen — Enums (6)

| Enum | Varianten | Beschreibung | Datei |
|---|---|---|---|
| `ArmRegistryError` | `/// Der angegebene Arm-Index liegt außerhalb des gültigen Bereichs (0..=4). #[error("arm index {0} out of range, expected 0..=4")] OutOfRange(u32),` | Fehlerklasse für ungültige Arm-Indizes — crate-lokal, P6-konform an Systemgrenzen konvertierbar. | `src/arm_registry.rs` |
| `FcTsDispatchError` | `/// Das FC-TS-Arm-Set ist leer oder ungültig konfiguriert. #[error("FC-TS arm set is empty or misconfigured: {0}")] ArmSet(#[from] FcTsError), /// Anzahl der übergebenen Profilnamen stimmt nicht mit…` | Fehler der FC-TS-Profilauswahl für contextra-router. | `src/fc_ts_dispatch.rs` |
| `QuantizationLevel` | `F16, Q8_0, Q4_K_M, #[default] Unknown,` | Quantisierungsstufe des Modells (z. B. F16, Q8_0, Q4_K_M). | `src/profile.rs` |
| `RoutingOutcome` | `/// SLM hat die Anfrage vollständig und korrekt beantwortet. Success, /// SLM-Antwort war unzureichend — Eskalation zu einem größeren Modell war nötig. /// SLM-Antwort war unzureichend — Eskalation z…` | Tatsächliches Ergebnis einer getroffenen Routing-Entscheidung. Wird vom Agent-Orchestrator oder Evaluator NACH dem SLM-Aufruf an den Router zurückgem… | `src/outcome.rs` |
| `RoutingStrategy` | `/// Deterministischer Kaskaden-Klassifikator (produktiv, kalibriert). /// Bewertet Token-Budgets, Domänen-Tags und empirische Konfidenz. #[default] Cascade, /// LinUCB Contextual Bandit (Feature `ban…` | Auswahl-Strategie für SLM-Profil-Routing. Steuert, wie die `RouterEngine` unter den verfügbaren Profilen das am besten geeignete Modell bestimmt. # I… | `src/routing_strategy.rs` |
| `Transport` | `/// Standard: stdio JSON-RPC 2.0 zu lokalem SLM-Prozess (ADR-010). StdioMcp, /// Cloud-HTTP: Nur mit Feature `cloud-egress-guard` + `GuardedPayload<Sanitized>`. #[cfg(feature = "cloud-egress-guard")]…` | Transport-Kanal für SLM-Routing-Entscheidungen. Definiert, über welches Medium das Sprachmodell (SLM = Small Language Model) angesprochen wird. # Inv… | `src/transport.rs` |

### Typ-Aliase (1)

| Alias | Definition | Beschreibung |
|---|---|---|
| `ConformalCalibrator` | `type ConformalCalibrator= ProfileCalibrator` | Typ-Alias für `ProfileCalibrator` gemäß Spec-Namenskonvention. |

### Öffentliche Funktionen (47)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn arm_for(&self, strategy: RetrievalStrategy) -> u32` | — | `src/arm_registry.rs` |
| `pub fn strategy_for(&self, arm: u32) -> Result<RetrievalStrategy, ArmRegistryError>` | Liefert die [`RetrievalStrategy`] für den gegebenen Arm-Index (0..=4). # Ablauf 1. Konvertierung des `u32`-Index nach `usize`. 2. Abruf des… | `src/arm_registry.rs` |
| `pub async fn dispatch_to_slm(decision: &RoutingDecision) -> Result<String>` | Führt den MCP-Aufruf an das in der Routing-Entscheidung ausgewählte SLM aus. # Ablauf 1. Überprüfung des Transport-Kanals (aktuell Stdio JS… | `src/dispatch.rs` |
| `pub fn deterministic_fc_ts_rng(seed: u64) -> impl FcTsRng` | — | `src/fc_ts_dispatch.rs` |
| `pub fn select_profile_fc_ts( profile_names: &[String], arm_set: &FcTsArmSet, context: &[f32], rng: &mut dyn FcTsRng, ) -> Result<(usize, String), FcTsDispatchError>` | — | `src/fc_ts_dispatch.rs` |
| `pub fn new(start: u64) -> Self` | Erstellt einen neuen Generator mit dem angegebenen Startwert. # Parameter - `start`: Anfangswert für die Sequenz (meist 0). | `src/outcome.rs` |
| `pub fn next(&self) -> DecisionId` | — | `src/outcome.rs` |
| `pub fn non_conformity_score(&self) -> f32` | Berechnet den Non-Conformity-Score: 0.0 = perfekt konform, 1.0 = maximal non-konform / fehlerhaft. # Werte - [`RoutingOutcome::Success`] =>… | `src/outcome.rs` |
| `pub fn average_confidence(&self) -> f64` | Liefert die durchschnittliche Konfidenz über alle Auswahlen. | `src/profile.rs` |
| `pub fn check_and_invalidate_fingerprint(&mut self, active_fp: Option<&ConfigFingerprint>)` | Prüft, ob der aktive Konfigurations-Fingerprint abweicht, und setzt bei Diskrepanz die Kalibrierung zurück (INV-P8-1). | `src/profile.rs` |
| `pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<BTreeSet<u64>, D::Error>` | — | `src/profile.rs` |
| `pub fn empirical_error_rate(&self) -> f32` | Berechnet die empirische Fehlerrate im aktuellen Beobachtungsfenster. | `src/profile.rs` |
| `pub fn estimated_cost(&self) -> f32` | Liefert die geschätzten Kosten des Profils (Fallback auf `token_budget.limit`). | `src/profile.rs` |
| `pub fn is_calibrated(&self, active_fp: Option<&ConfigFingerprint>) -> bool` | Prüft, ob das Profil kalibriert und der Fingerprint noch valide ist. | `src/profile.rs` |
| `pub fn new( name: impl Into<String>, mcp_endpoint: impl Into<String>, domain_communities: impl IntoIterator<Item = u64>, token_budget: TokenBudget, min_relevance_score: f32, ) -> Self` | Erzeugt ein neues `SlmProfile` mit Standardwerten. # Ablauf 1. Belegung aller Pflichtfelder (Name, Endpunkt, Communities, Budget, Mindestsc… | `src/profile.rs` |
| `pub fn new(alpha: f32, gamma: f32, initial_threshold: f32) -> Self` | Erstellt einen neuen `ProfileCalibrator` mit Fehlertoleranz `alpha`, Schrittweite `gamma` und Startschwelle. | `src/profile.rs` |
| `pub fn new(original_min_score: f32) -> Self` | Erzeugt einen neuen Initialzustand für die Kalibrierung eines Profils. | `src/profile.rs` |
| `pub fn recalibrate_conformal(&mut self, non_conformity_score: f32) -> bool` | Führt eine Neukalibrierung anhand des beobachteten Non-Conformity-Scores durch. | `src/profile.rs` |
| `pub fn reset(&mut self)` | Setzt alle Kalibrierungsstatistiken auf die Ausgangswerte zurück. | `src/profile.rs` |
| `pub fn reset_window(&mut self)` | Setzt die Fensterzähler für Fehler und Gesamtzahl zurück. | `src/profile.rs` |
| `pub fn serialize<S: Serializer>(set: &BTreeSet<u64>, s: S) -> Result<S::Ok, S::Error>` | — | `src/profile.rs` |
| `pub fn try_new( name: impl Into<String>, mcp_endpoint: impl Into<String>, domain_communities: impl IntoIterator<Item = u64>, token_budget: TokenBudget, min_relevance_score: f32, ) -> Result<Self>` | Erzeugt ein neues `SlmProfile` und validiert es sofort strikt. | `src/profile.rs` |
| `pub fn update(&mut self, non_conformity_score: f32) -> bool` | Aktualisiert den Quantils-Schwellwert basierend auf einem beobachteten Non-Conformity-Score. # Ablauf 1. Fehlerprüfung: War der Score größe… | `src/profile.rs` |
| `pub fn validate(&self) -> Result<()>` | Validiert alle Invarianten des Profils. # Invarianten - Name darf nicht leer sein. - MCP-Endpunkt darf nicht leer sein. - `min_relevance_sc… | `src/profile.rs` |
| `pub fn with_fingerprint(mut self, fingerprint: ConfigFingerprint) -> Self` | Weist dem Profil einen Konfigurations-Fingerprint zu (P8-Pflicht). | `src/profile.rs` |
| `pub fn with_resource_cost_estimate(mut self, resource_cost_estimate: f32) -> Self` | Weist dem Profil geschätzte Ressourcenkosten zu. | `src/profile.rs` |
| `pub async fn route( &self, query_embedding: &[f32], query_text: &str, ) -> Result<RoutingDecision>` | Führt eine vollständige Routing-Entscheidung für die übergebene Anfrage aus. # Ablauf 1. Suchergebnisse via `search_provider` abrufen und m… | `src/router/dispatch_core.rs` |
| `pub fn new( search_provider: Arc<dyn HybridSearchProvider>, community_resolver: Arc<dyn CommunityResolver>, context_preparer: Arc<dyn ContextPreparer>, profiles: Vec<SlmProfile>, calibration_store_path: Option<std::path::PathBuf>, ) -> Self` | Erzeugt eine neue `RouterEngine` mit Standardeinstellungen. | `src/router/lifecycle.rs` |
| `pub fn profiles(&self) -> Vec<SlmProfile>` | Liefert eine Momentaufnahme der aktuell aktiven Profile. | `src/router/lifecycle.rs` |
| `pub fn try_new( search_provider: Arc<dyn HybridSearchProvider>, community_resolver: Arc<dyn CommunityResolver>, context_preparer: Arc<dyn ContextPreparer>, profiles: Vec<SlmProfile>, calibration_store_path: Option<std::path::PathBuf>, ) -> Result<Self>` | Erzeugt eine neue `RouterEngine` und validiert alle übergebenen Profile. | `src/router/lifecycle.rs` |
| `pub fn try_update_profiles(&self, new_profiles: Vec<SlmProfile>) -> Result<()>` | Aktualisiert Profile atomar nach vorheriger strikter Validierung. | `src/router/lifecycle.rs` |
| `pub fn update_profiles(&self, new_profiles: Vec<SlmProfile>)` | Aktualisiert die Profile im laufenden Betrieb atomar (Hot-Reload via ArcSwap). | `src/router/lifecycle.rs` |
| `pub fn with_clock(mut self, clock: Arc<dyn Clock>) -> Self` | Injiziert eine benutzerdefinierte System-Uhr (Port-Entwurf für Tests). | `src/router/lifecycle.rs` |
| `pub fn with_initial_decision_id(mut self, start: u64) -> Self` | Setzt den Startwert für monotone Entscheidungs-IDs. | `src/router/lifecycle.rs` |
| `pub fn with_routing_strategy( mut self, strategy: RoutingStrategy, epsilon: f32, seed: u64, ) -> Self` | Konfiguriert die Routing-Strategie (Cascade oder Bandit). | `src/router/lifecycle.rs` |
| `pub fn bandit_decision_propensity(&self, id: DecisionId) -> Option<f32>` | Liefert die Aktionswahrscheinlichkeit (Propensity) einer offenen Bandit-Entscheidung. | `src/router/outcomes.rs` |
| `pub fn calibration_stats(&self) -> BTreeMap<String, ProfileCalibrationState>` | Liefert eine Momentaufnahme aller aktuellen Profil-Kalibrierungen. | `src/router/outcomes.rs` |
| `pub fn drift_status(&self, profile_name: &str) -> Option<LyapunovResult>` | Fragt den Ljapunow-Drift-Status für ein bestimmtes Profil ab. | `src/router/outcomes.rs` |
| `pub fn overall_drift_status(&self) -> String` | Liefert eine textuelle Gesamtübersicht des System-Drifts. | `src/router/outcomes.rs` |
| `pub fn pending_decision_count(&self) -> usize` | Liefert die Anzahl offener Entscheidungen, die auf Feedback warten. | `src/router/outcomes.rs` |
| `pub fn record_outcome(&self, decision_id: DecisionId, outcome: RoutingOutcome) -> bool` | Protokolliert das Ergebnis einer Routing-Entscheidung und aktualisiert Kalibrierung und Bandit-Statistiken. | `src/router/outcomes.rs` |
| `pub fn reset_all_calibration(&self)` | Setzt alle Kalibrierungen systemweit auf ihre Initialwerte zurück. | `src/router/outcomes.rs` |
| `pub fn reset_calibration(&self, profile_name: &str)` | Setzt die Kalibrierungsstatistik für ein einzelnes Profil zurück. | `src/router/outcomes.rs` |
| `pub fn set_lyapunov_baseline(&self, profile_name: &str, baseline: &[f32]) -> bool` | Setzt die Referenz-Baseline für die Drift-Erkennung eines Profils. | `src/router/outcomes.rs` |
| `pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<BTreeSet<u64>, D::Error>` | Deserialisiert eine Sequenz von `u64`-Werten in ein geordnetes `BTreeSet<u64>`. # Ablauf 1. Sequenz aus dem Deserializer als `Vec<u64>` ein… | `src/serde_helpers.rs` |
| `pub fn serialize<S: Serializer>(set: &BTreeSet<u64>, s: S) -> Result<S::Ok, S::Error>` | — | `src/serde_helpers.rs` |
| `pub fn is_cloud(&self) -> bool` | Prüft, ob dieser Transport einen externen Cloud-Endpunkt repräsentiert. # Ablauf 1. Pattern-Matching auf die Transport-Variante. 2. `false`… | `src/transport.rs` |


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
pub mod tools;
pub use budget::{BudgetStrategy, Reservation, TokenBudget};
pub use clm_scratchpad::{ ClmScratchpad, ContextEditAuditSink, CountingContextEditAuditSink, CountingScratchpadCacheInvalidator, NoopContextEditAuditSink, NoopScratchpadCacheInvalidator, PinnedRegionId, ScratchpadCacheInvalidator, ScratchpadCheckpoint, ScratchpadEditOp, };
pub use context::{AgentContext, AgentStatus};
pub use dlq::DeadLetterQueue;
pub use engine::{EventLoopExitReason, OrchestratorEngine, MAX_WORKFLOW_STEPS};
pub use event_source::{BackgroundEvent, EventSource, PollingDocumentEventSource, VecEventSource};
pub use goal_condition::{parse_legacy_condition_string, GoalCondition};
pub use graph::{AgentNode, NodeType, StateGraph, WorkflowEdge};
pub use step::{AgentTool, DeadLetterReason, StepDeadLetter, StepResult};
pub use tools::{RetrieveTool, ToolManifest, ToolRegistry};
```

### Feature-Flags

| Flag | Aktiviert |
|---|---|
| `default` | `[]` |
| `test-utils` | `[]` |

### Schnittstellen — Public Traits (4)

**`AgentTool`** — *Core interface implemented by all executable tools participating in agent workflows.* (`src/step.rs`)

```rust
pub trait AgentTool : Send + Sync {
/// Returns the unique name identifier of this tool.
    fn name(&self) -> &str;

    /// Returns the estimated token cost of executing this tool with the given input.
    ///
    /// Used for strict pre-execution budget validation to prevent side-effects when budget is exhausted.
    fn estimated_cost(&self, _input: &serde_json::Value) -> usize {
        0
    }

    /// Executes the tool asynchronously within the given [`AgentContext`].
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
/// Records a scratchpad edit operation audit entry.
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
/// Invalidates cache entries associated with a scratchpad subgoal scope.
    fn invalidate_scratchpad_scope(&self, tenant_id: TenantId, task_id: &str, subgoal_index: u64);
}
```

### Öffentliche Datentypen — Structs (26)

| Struct | Felder / Form | Beschreibung | Datei |
|---|---|---|---|
| `AgentContext` | `{ /// Unique task identifier for this agent workflow run. pub task_id: String, /// Active graph node identifier currently being evaluated. pub current_node: Stri… }` | Operational context spanning the entire workflow execution. | `src/context.rs` |
| `AgentEngine` | `{ /// Active cache directive set during step execution. pub current_directive: CacheDirective, }` | Agent execution engine helper managing per-step cache directives during workflow execution. | `src/context.rs` |
| `AgentNode` | `{ /// Unique identifier of the node. pub id: NodeId, /// Text description of the node's task or role. pub description: String, /// Functional type category of th… }` | Represents a single specialized node in the workflow. | `src/graph.rs` |
| `AuditEntry` | `{ /// Task identifier associated with this audit entry. pub task_id: String, /// Step sequence index within the task workflow. pub step_count: u64, /// Node iden… }` | Single immutable record of an agent step execution. | `src/audit.rs` |
| `AuditLog` | `{ collection: Arc<Collection<S>>, }` | Append-only audit log backed by a Contextra collection. | `src/audit.rs` |
| `BackgroundEvent` | `{ /// Payload associated with the background telemetry event. pub payload: serde_json::Value, /// Origin identifier of the event producer. pub source: String, //… }` | Background telemetry/trigger event delivered to the agent context. | `src/event_source.rs` |
| `ClmScratchpad` | `{ task_id: String, tenant_id: TenantId, current_subgoal_index: u64, created_at_tx: u64, config: VaultConfig, vault: VolatileContextVault, chunks: Vec<VaultChunk>… }` | Temporärer, aufgabengebundener Arbeitskontext-Scratchpad für Agentenschleifen. Kapselt eine [`VolatileContextVault`]-Instanz für die sichere RAM-nati… | `src/clm_scratchpad.rs` |
| `ContextEditAuditRecord` | `{ /// Tenant identifier executing the scratchpad edit. pub tenant_id: TenantId, /// Task identifier associated with the scratchpad edit. pub task_id: String, ///… }` | Audit-Datensatz für Scratchpad-Kontextänderungen und Resets. | `src/clm_scratchpad.rs` |
| `CountingContextEditAuditSink` | `{ count: AtomicU64, }` | Test-Implementierung von [`ContextEditAuditSink`], die Aufrufe zählt. | `src/clm_scratchpad.rs` |
| `CountingScratchpadCacheInvalidator` | `{ count: AtomicU64, }` | Test-Implementierung von [`ScratchpadCacheInvalidator`], die Aufrufe zählt. | `src/clm_scratchpad.rs` |
| `DeadLetterQueue` | `{ storage: Arc<dyn StorageEngine>, next_tx: OnceCell<AtomicU64>, metrics_sink: Option<Arc<dyn contextra_ports::MetricsSink>>, }` | Persistente Dead-Letter-Queue für fehlgeschlagene Agent-Schritte. Verwendet denselben Storage wie der Agent (LSM) mit einem fixen Key-Prefix. | `src/dlq.rs` |
| `MigrationStats` | `{ /// Number of legacy entries successfully migrated. pub migrated: usize, /// Number of legacy entries that failed migration. pub failed: usize, }` | Summary statistics for legacy audit entry migration. | `src/audit.rs` |
| `NoopContextEditAuditSink` | `(tuple/unit)` | No-op-Implementierung von [`ContextEditAuditSink`]. | `src/clm_scratchpad.rs` |
| `NoopScratchpadCacheInvalidator` | `(tuple/unit)` | No-op-Implementierung von [`ScratchpadCacheInvalidator`]. | `src/clm_scratchpad.rs` |
| `OrchestratorEngine` | `{ /// Registry managing tool manifests, availability, and authorization checks. pub registry: crate::tools::ToolRegistry, /// Store managing workflow state check… }` | Async executor engine applying nodes in sequence. | `src/engine.rs` |
| `PinnedRegionId` | `(tuple/unit)` | Marker für unveränderliche Kontextteile (Systemprompt, Gesamtziel, Safety-Regeln). Wird ausschließlich gelesen und NIEMALS durch diesen Typ selbst ve… | `src/clm_scratchpad.rs` |
| `PollingDocumentEventSource` | `{ collection: Arc<Collection<S>>, last_seen_seq: u64, poll_interval: Duration, pending_events: VecDeque<BackgroundEvent>, }` | Concrete `EventSource` that periodically polls `Collection` storage sequence numbers, using `scan_prefix_at` for snapshot delta calculation to emit d… | `src/event_source.rs` |
| `RetrieveTool` | `{ name: String, }` | Read-only tenant context retrieval tool implementing [`AgentTool`]. Searches documents within the tenant's collection context without mutating state. | `src/tools/retrieve.rs` |
| `ScratchpadCheckpoint` | `{ /// Eindeutige Aufgaben-ID. pub task_id: String, /// Index des erfolgreich abgeschlossenen Teilziels. pub completed_subgoal_index: u64, /// Löschquittung nach… }` | Zusammenfassung eines Subgoal-Checkpoints vor dem Zurücksetzen des Scratchpads. Dieser Typ dient als Übergabeobjekt für nachfolgende Stufen (z. B. Sp… | `src/clm_scratchpad.rs` |
| `StateGraph` | `{ /// Map of node identifiers to workflow nodes. pub nodes: BTreeMap<NodeId, AgentNode>, /// List of directional conditional transition edges. pub edges: Vec<Wor… }` | Core declarative structure routing autonomous agent steps. | `src/graph.rs` |
| `StepDeadLetter` | `{ /// Session-ID des Agenten pub session_id: String, /// Node-ID des fehlgeschlagenen Schritts pub node_id: String, /// Schrittnummer im Workflow (0-indexed) #[s… }` | Ein fehlgeschlagener Agent-Schritt der für spätere Analyse und Idempotenz-Prüfung persistiert wird. | `src/step.rs` |
| `StepResult` | `{ /// Unique identifier of the node that produced this step result. pub node_id: String, /// JSON output payload produced by the step execution. pub output: serd… }` | The explicit result of an agent step execution. | `src/step.rs` |
| `ToolManifest` | `{ /// Unique identifier of the tool. pub name: String, /// Human-readable description of the tool's purpose. pub description: String, /// Required capability ide… }` | Declarative manifest specifying operational properties, capability requirements, and retry policies for a tool. | `src/tools/manifest.rs` |
| `ToolRegistry` | `{ entries: BTreeMap<String, (ToolManifest, Box<dyn AgentTool>)>, }` | Registry managing validated tool manifests and execution dispatch for agent steps. | `src/tools/registry.rs` |
| `VecEventSource` | `{ events: VecDeque<BackgroundEvent>, }` | Trivial mock/static `EventSource` that reads from a fixed list of `BackgroundEvent`s. | `src/event_source.rs` |
| `WorkflowEdge` | `{ /// Origin node identifier of the transition. pub from: NodeId, /// Target node identifier of the transition. pub to: NodeId, /// Legacy text-based condition s… }` | Represents a conditional transition between two graph nodes. | `src/graph.rs` |

### Öffentliche Datentypen — Enums (6)

| Enum | Varianten | Beschreibung | Datei |
|---|---|---|---|
| `AgentStatus` | `/// Agent workflow has been initialized but is not actively running. Idle, /// Agent workflow is actively executing graph step transitions. Running, /// Agent workflow reached a terminal `End` node s…` | Operational status of an agent workflow. | `src/context.rs` |
| `DeadLetterReason` | `/// Tool hat nicht innerhalb des Timeouts geantwortet Timeout { /// Execution timeout limit in milliseconds. timeout_ms: u64, }, /// Budget erschöpft bevor der Schritt starten konnte BudgetExhausted…` | Specific failure reason category recorded when an agent step is moved to the Dead-Letter Queue. | `src/step.rs` |
| `EventLoopExitReason` | `/// Workflow event loop was terminated due to a cancellation request. Shutdown, /// Workflow event loop exited because the underlying event source was exhausted. SourceExhausted,` | Reason for exiting `OrchestratorEngine::run_event_loop`. | `src/engine.rs` |
| `GoalCondition` | `/// Prüft, ob ein JSON-Feld unter `field_path` im `StepResult::output` mit `expected` übereinstimmt. OutputFieldEquals { /// Punkt-getrennter JSON-Pfad (z. B. "status.done"). field_path: String, ///…` | Repräsentiert eine strukturierte Bedingung zur Auswertung, ob ein Teilziel eines Agentenschritts erfüllt ist. | `src/goal_condition.rs` |
| `NodeType` | `/// Initial workflow entry point node. Start, /// Operational task step executing a registered tool handler. Task, /// Conditional branching node evaluating context conditions. Decision, /// Terminal…` | Type of node within the declarative agent state graph. | `src/graph.rs` |
| `ScratchpadEditOp` | `/// Hängt einen neuen Kontext-Chunk an das Scratchpad an. Append { /// Byte-Inhalt des neuen Chunks. content: Vec<u8>, /// Optionaler Label-Bezeichner für spätere Referenzierung (Replace/Remove). lab…` | Operationen zur Bearbeitung des Scratchpads. | `src/clm_scratchpad.rs` |

### Typ-Aliase (1)

| Alias | Definition | Beschreibung |
|---|---|---|
| `NodeId` | `type NodeId= String` | Type alias for node identifiers in the state graph. |

### Konstanten (8)

| Konstante | Typ | Beschreibung |
|---|---|---|
| `MAX_EVENT_SOURCE_CAPACITY` | `usize` | Maximum allowed events in pending event buffers to prevent memory exhaustion. |
| `MAX_ID_LEN` | `usize` | Maximum allowed length in bytes for task IDs and node IDs. |
| `MAX_MANIFEST_DESC_LEN` | `usize` | Maximum text length for manifest description. |
| `MAX_PENDING_EVENTS_CAPACITY` | `usize` | Maximum capacity for pending background telemetry events queue before dropping or rejecting. |
| `MAX_TELEMETRY_EVENTS` | `usize` | Maximum allowed telemetry events stored in memory history. |
| `MAX_TEXT_LEN` | `usize` | Maximum allowed length in bytes for descriptions and conditions. |
| `MAX_WORKFLOW_STEPS` | `u64` | Maximum allowed steps in a single workflow execution to prevent unbounded loops. |
| `PREFIX` | `&'static [u8]` | LSM key prefix used to isolate DLQ records. |

### Öffentliche Funktionen (99)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub async fn append(&self, entry: &AuditEntry) -> Result<()>` | Appends an immutable audit entry directly via LSM storage without HNSW vector index participation (AC-3). Enforces append-only immutability… | `src/audit.rs` |
| `pub async fn append_to(collection: &Collection<S>, entry: &AuditEntry) -> Result<()>` | Appends an immutable audit entry directly to a collection without needing an [`AuditLog`] wrapper. | `src/audit.rs` |
| `pub async fn migrate_legacy_audit_entries<S: StorageEngine, V: contextra_ports::VectorIndex>( collection: &Collection<S, V>, ) -> Result<MigrationStats>` | Migrates legacy zero-vector audit entries from the HNSW vector index and doc_key mappings into the direct LSM KV store format (`put_kv`). R… | `src/audit.rs` |
| `pub fn new(collection: Arc<Collection<S>>) -> Self` | Constructs a new [`AuditLog`] wrapping a collection reference. | `src/audit.rs` |
| `pub async fn replay_task(&self, task_id: &str) -> Result<Vec<AuditEntry>>` | Replays all audit entries for a given task via scan_prefix. | `src/audit.rs` |
| `pub fn validate_audit_payload_and_error( payload: &serde_json::Value, error: Option<&str>, ) -> Result<()>` | Validates payload and optional error string of an audit entry for non-emptiness and absence of null bytes. | `src/audit.rs` |
| `pub fn reserve_tokens<'a>( budget: &'a TokenBudget, amount: usize, ) -> contextra_types::Result<Reservation<'a>>` | Helper to reserve `amount` tokens from `budget`. Returns an RAII [`Reservation`] guard on success. If execution drops the guard without cal… | `src/budget.rs` |
| `pub fn apply_edit(&mut self, op: ScratchpadEditOp, captured_tx: TxId) -> Result<()>` | Wendet eine Bearbeitungs-Operation ([`ScratchpadEditOp`]) auf das Scratchpad an. # Fehler Liefert `ContextraError::InvalidInput` falls: - V… | `src/clm_scratchpad.rs` |
| `pub fn as_str(&self) -> &str` | Gibt den inneren String-Bezeichner zurück. | `src/clm_scratchpad.rs` |
| `pub fn checkpoint_and_reset(&mut self) -> Result<ScratchpadCheckpoint>` | Erzeugt eine [`ScratchpadCheckpoint`]-Zusammenfassung, leert den Vault intern via `purge()` (inkl. Zeroize aller RAM-Inhalte), meldet Audit… | `src/clm_scratchpad.rs` |
| `pub fn count(&self) -> u64` | Returns the accumulated count of recorded edit operations. | `src/clm_scratchpad.rs` |
| `pub fn count(&self) -> u64` | Returns the accumulated count of invalidation calls. | `src/clm_scratchpad.rs` |
| `pub fn created_at_tx(&self) -> u64` | Gibt die Transaktions-ID der Erzeugung zurück. | `src/clm_scratchpad.rs` |
| `pub fn current_subgoal_index(&self) -> u64` | Gibt den aktuellen Subgoal-Index zurück. | `src/clm_scratchpad.rs` |
| `pub fn new() -> Self` | Constructs a new `CountingContextEditAuditSink`. | `src/clm_scratchpad.rs` |
| `pub fn new() -> Self` | Constructs a new `CountingScratchpadCacheInvalidator`. | `src/clm_scratchpad.rs` |
| `pub fn new(id: impl Into<String>) -> Self` | Erzeugt eine neue `PinnedRegionId`. | `src/clm_scratchpad.rs` |
| `pub fn new(task_id: String, config: VaultConfig) -> Self` | Erzeugt einen neuen `ClmScratchpad` für eine Task mit gegebener Vault-Konfiguration. | `src/clm_scratchpad.rs` |
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
| `pub async fn allocate_tx(&self) -> Result<TxId>` | Allocates a unique sequence transaction ID for DLQ storage mutations. | `src/dlq.rs` |
| `pub async fn drain(&self) -> Result<Vec<StepDeadLetter>>` | Reads and removes all dead-letter records atomically from storage. | `src/dlq.rs` |
| `pub async fn is_already_committed(&self, letter: &StepDeadLetter) -> Result<bool>` | Prüft vor dem Replay eines DLQ-Eintrags, ob die im Eintrag assoziierte `TxId` bereits im WAL/Storage für denselben `(Session, Node, Step)`-… | `src/dlq.rs` |
| `pub async fn list(&self) -> Result<Vec<StepDeadLetter>>` | Lists all current dead-letter records without removing them. | `src/dlq.rs` |
| `pub fn new(storage: Arc<dyn StorageEngine>) -> Self` | Constructs a new [`DeadLetterQueue`] backed by the provided storage engine. | `src/dlq.rs` |
| `pub fn new_with_metrics( storage: Arc<dyn StorageEngine>, sink: Arc<dyn contextra_ports::MetricsSink>, ) -> Self` | Helper constructor creating a [`DeadLetterQueue`] with an explicit metrics sink. | `src/dlq.rs` |
| `pub async fn push(&self, letter: &StepDeadLetter) -> Result<()>` | Pushes a failed step record to the persistent Dead-Letter Queue. | `src/dlq.rs` |
| `pub async fn remove(&self, letter: &StepDeadLetter) -> Result<bool>` | Entfernt einen spezifischen DLQ-Eintrag anhand seines `(session_id, node_id, step_index)` Schlüssels. | `src/dlq.rs` |
| `pub fn with_metrics_sink(mut self, sink: Arc<dyn contextra_ports::MetricsSink>) -> Self` | Attaches a metrics sink for DLQ gauge telemetry (Builder pattern). | `src/dlq.rs` |
| `pub async fn checkpoint(&self, ctx: &AgentContext) -> Result<()>` | Persists a workflow execution state checkpoint for the active context step. | `src/engine.rs` |
| `pub fn from_db(db: &contextra_db::Contextra) -> Self` | — | `src/engine.rs` |
| `pub fn new(storage: Arc<LsmStorage>) -> Self` | Constructs a new [`OrchestratorEngine`] from LSM storage (legacy constructor). | `src/engine.rs` |
| `pub async fn recover_orphans(&self) -> Result<()>` | Recovers all registered/persisted orphaned sequence pins and checkpoints. | `src/engine.rs` |
| `pub async fn replay_from(&self, ctx: &mut AgentContext, identifier: &str) -> Result<()>` | Setzt den AgentContext auf einen früheren Checkpoint zurück (AC-2). # Adressierung von Checkpoints Das `identifier`-Argument unterstützt fo… | `src/engine.rs` |
| `pub async fn run(&self, ctx: &mut AgentContext, graph: &StateGraph) -> Result<()>` | Runs the persistent workflow execution loop for a given [`AgentContext`] and [`StateGraph`]. | `src/engine.rs` |
| `pub async fn run_event_loop( &self, ctx: &mut AgentContext, graph: &StateGraph, source: &mut dyn crate::event_source::EventSource, shutdown: tokio_util::sync::CancellationToken, ) -> Result<EventLoopExitReason>` | Continuous event loop reading telemetry events from an `EventSource`, attaching each event to `AgentContext`, executing `run()`, and checkp… | `src/engine.rs` |
| `pub fn try_from_db(db: &contextra_db::Contextra) -> Result<Self>` | Attempts to construct an [`OrchestratorEngine`] directly from a Contextra DB handle. | `src/engine.rs` |
| `pub fn try_new(storage: Arc<LsmStorage>) -> Result<Self>` | Attempts to construct a new [`OrchestratorEngine`], returning an error if checkpoint store initialization fails. | `src/engine.rs` |
| `pub fn try_register_tool(&mut self, tool: Box<dyn AgentTool>) -> Result<()>` | Attempts to register an agent tool with boundary validation on the tool name. | `src/engine.rs` |
| `pub fn try_register_tool_with_manifest( &mut self, manifest: crate::tools::ToolManifest, tool: Box<dyn AgentTool>, ) -> Result<()>` | Registers an agent tool with an explicit declarative [`ToolManifest`](crate::tools::ToolManifest). | `src/engine.rs` |
| `pub fn with_clock(mut self, clock: Arc<dyn Clock>) -> Self` | Sets a custom [`Clock`] implementation for deterministic time management. | `src/engine.rs` |
| `pub fn evaluate_condition_expr(expr: &str, ctx: &AgentContext) -> bool` | Evaluates a declarative condition expression against the provided [`AgentContext`]. # Supported Grammar: - `<key> exists`: Returns `true` i… | `src/engine/condition.rs` |
| `pub fn new( payload: serde_json::Value, source: impl Into<String>, observed_at_seq: u64, ) -> Self` | Legacy constructor for `BackgroundEvent` (deprecated). | `src/event_source.rs` |
| `pub fn new(collection: Arc<Collection<S>>, poll_interval: Duration) -> Self` | Creates a new `PollingDocumentEventSource` watching the specified collection. | `src/event_source.rs` |
| `pub fn poll_interval(&self) -> Duration` | Returns the configured polling interval duration. | `src/event_source.rs` |
| `pub fn try_new( payload: serde_json::Value, source: impl Into<String>, observed_at_seq: u64, ) -> Result<Self>` | — | `src/event_source.rs` |
| `pub fn try_new(events: Vec<BackgroundEvent>) -> Result<Self>` | Attempts to construct a `VecEventSource` with a capacity check. | `src/event_source.rs` |
| `pub fn with_capacity( collection: Arc<Collection<S>>, poll_interval: Duration, _max_pending_capacity: usize, ) -> Self` | — | `src/event_source.rs` |
| `pub fn with_last_seen_seq(mut self, last_seen_seq: u64) -> Self` | Configures the initial last seen sequence number (Builder pattern). | `src/event_source.rs` |
| `pub fn describe(&self) -> String` | Liefert eine menschenlesbare Beschreibung der Bedingung (z. B. für Audit-Logging). | `src/goal_condition.rs` |
| `pub fn evaluate(&self, result: &StepResult) -> bool` | Wertet die Bedingung deterministisch gegen ein gegebenes [`StepResult`] aus. Liefert bei fehlendem oder ungültigem JSON-Pfad stets `false`.… | `src/goal_condition.rs` |
| `pub fn parse_legacy_condition_string(raw: &str) -> Option<GoalCondition>` | Bildet einen freien Bedingungs-String aus Altbestand auf eine strukturierte [`GoalCondition`] ab. Unterstützt Gleichheitsprüfungen ("field… | `src/goal_condition.rs` |
| `pub fn add_edge(&mut self, from: &str, to: &str, condition: Option<&str>, priority: u8)` | Adds an edge to the graph, panicking if validation fails. | `src/graph.rs` |
| `pub fn add_node( &mut self, id: &str, description: &str, node_type: NodeType, handler: Option<&str>, )` | Adds a node to the graph, panicking if validation fails. | `src/graph.rs` |
| `pub fn evaluate_condition(&self, result: &StepResult) -> bool` | Evaluates whether this edge's condition/goal is satisfied against a [`StepResult`]. # Evaluation Order & Fail-Closed Semantics: 1. If `goal… | `src/graph.rs` |
| `pub fn get_node(&self, id: &str) -> Option<&AgentNode>` | Looks up a node by its identifier. | `src/graph.rs` |
| `pub fn has_condition(&self) -> bool` | Returns whether this edge has an explicit condition or goal attached. | `src/graph.rs` |
| `pub fn new() -> Self` | Creates a new empty [`StateGraph`]. | `src/graph.rs` |
| `pub fn try_add_edge( &mut self, from: &str, to: &str, condition: Option<&str>, priority: u8, ) -> Result<()>` | — | `src/graph.rs` |
| `pub fn try_add_edge_with_goal( &mut self, from: &str, to: &str, goal: GoalCondition, priority: u8, ) -> Result<()>` | Tries to insert a new edge with an explicit [`GoalCondition`]. | `src/graph.rs` |
| `pub fn try_add_node( &mut self, id: &str, description: &str, node_type: NodeType, handler: Option<&str>, ) -> Result<()>` | — | `src/graph.rs` |
| `pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self` | Creates a new `ToolManifest` with the given name and description. | `src/tools/manifest.rs` |
| `pub fn validate(&self) -> Result<()>` | Validates the tool manifest fields against boundary rules. | `src/tools/manifest.rs` |
| `pub fn with_capability(mut self, capability: impl Into<String>) -> Self` | Sets the required capability for execution. | `src/tools/manifest.rs` |
| `pub fn with_retry(mut self, is_retriable: bool, max_retries: u32) -> Self` | Sets retry behavior. | `src/tools/manifest.rs` |
| `pub fn with_timeout_ms(mut self, timeout_ms: u64) -> Self` | Sets the timeout in milliseconds. | `src/tools/manifest.rs` |
| `pub fn authorize(&self, ctx: &AgentContext, tool_name: &str) -> Result<&ToolManifest>` | Checks pre-execution permissions for a tool (fail-closed). Returns `Ok(&ToolManifest)` if the context is authorized, or `Err(ContextraError… | `src/tools/registry.rs` |
| `pub fn contains(&self, name: &str) -> bool` | Returns whether a tool with the given name is registered. | `src/tools/registry.rs` |
| `pub fn get_manifest(&self, name: &str) -> Option<&ToolManifest>` | Retrieves a reference to a registered tool's manifest by name. | `src/tools/registry.rs` |
| `pub fn get_tool(&self, name: &str) -> Option<&dyn AgentTool>` | Retrieves a reference to a registered tool by name. | `src/tools/registry.rs` |
| `pub fn is_empty(&self) -> bool` | Returns `true` if no tools are registered. | `src/tools/registry.rs` |
| `pub fn len(&self) -> usize` | Returns the number of registered tools. | `src/tools/registry.rs` |
| `pub fn new() -> Self` | Creates a new empty `ToolRegistry`. | `src/tools/registry.rs` |
| `pub fn register(&mut self, manifest: ToolManifest, tool: Box<dyn AgentTool>) -> Result<()>` | Registers an `AgentTool` with its validated `ToolManifest`. | `src/tools/registry.rs` |
| `pub fn register_tool(&mut self, tool: Box<dyn AgentTool>) -> Result<()>` | Convenience helper to register a tool using default manifest settings derived from `AgentTool`. | `src/tools/registry.rs` |
| `pub fn new() -> Self` | Creates a new `RetrieveTool` instance. | `src/tools/retrieve.rs` |
| `pub fn with_name(name: impl Into<String>) -> Self` | Creates a new `RetrieveTool` with a custom handler name. | `src/tools/retrieve.rs` |


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
pub mod prelude { .. }
pub mod advanced { .. }
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
pub use contextra_engine::collection::{self, Collection, CollectionConfig};
pub use contextra_engine::{ Contextra, ContextraConfig, ContextraStats, DbStats, Document, EmbeddingBackend, HybridQueryBuilder, Language, ProvenanceRecord, SearchResult, SearchStrategy, SignalContribution, SignalWeights, };
pub use contextra_types::{DistanceMetric, DocId, RequestContext, TenantId, TxId};
pub use crate::multistep::{MultiStepConfig, MultiStepEngine, MultiStepResult, QueryRewriter};
pub use crate::volatile_vault::{ CommitReceipt, PurgeReceipt, SignalModality, VaultChunk, VaultChunkMetadata, VaultConfig, VaultError, VolatileContextVault, };
pub use crate::decay_controller::{ AdaptiveDecayController, DecayControllerConfig, DecaySignalInputs, };
pub use crate::homeostat::RerankDeadline;
pub use crate::homeostat::RerankPidController;
pub use crate::pid_latency_controller::{ LatencyBudgetGuard, PidLatencyController, DEFAULT_TARGET_LATENCY_MS, MAX_SCALING_FACTOR, MIN_SCALING_FACTOR, };
pub use contextra_cognition as cognition;
pub use contextra_cognition::{ cleanup_orphaned_consolidation_intents, compact_segment_via_context_compactor, compute_community_hash, detect_near_duplicates, execute_background_consolidation, execute_consolidation_pass, group_turns_into_segments, run_consolidation_pass, run_synthesis_pass, CommunityStabilityTracker, CompactedContext, CompactionStrategy, ConsolidationConfig, ConsolidationEngine, ConsolidationNodesGuard, ConsolidationPhaseResult, ConsolidationSession, ContextCompactor, ContextManager, MaintenanceConfig, MaintenanceScheduler, MetaChunk, SpatialFence, StatusToken, SynthesisConfig, SynthesisPhaseResult, TurnSegment, };
pub use contextra_engine as engine;
pub use contextra_engine::{ background_workers, chunker, export, filter, import, temporal_filter, transaction, CommunityDetectionConfig, ExportCollectionV1, ExportDocumentV1, ExportMemoryV1, ExportRelationV1, ImportSummary, MAX_SCAN_RESULTS, SCHEMA_VERSION_V1, };
pub use contextra_engine::collection::maintenance::PercolationResult;
pub use contextra_engine::SandboxBridge;
pub use advanced::*;
pub use prelude::*;
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
| `onnx` | `["contextra-engine/onnx"]` |
| `experimental-diskann` | `["contextra-vector/experimental-diskann"]` |
| `background-maintenance` | `["contextra-engine/background-maintenance"]` |
| `graph-connectivity-health` | `["contextra-engine/graph-connectivity-health", "contextra-graph/physio-percolation"]` |
| `coherence-bonus-fusion` | `[]` |
| `adaptive-candidate-pool-sizing` | `["contextra-engine/adaptive-candidate-pool-sizing"]` |
| `volatile-vault` | `[]` |
| `edge-reinforcement-learning` | `["contextra-graph/edge-reinforcement-learning"]` |

### Öffentliche Datentypen — Structs (9)

| Struct | Felder / Form | Beschreibung | Datei |
|---|---|---|---|
| `CommitReceipt` | `{ /// Anzahl geschriebener Chunks. pub chunks_committed: usize, /// Gesamtgröße der übergebenen Nutzdaten in Bytes. pub bytes_committed: usize, /// Mandantenkenn… }` | Quittung nach Übergabe der Chunks an die persistente Datenbank via [`VolatileContextVault::commit()`]. | `src/volatile_vault.rs` |
| `MultiStepConfig` | `{ /// Maximale Anzahl an Iterationsrunden (Standard: 3). pub max_rounds: usize, /// Mindest-Score-Schwellenwert (0.0 bis 1.0): Liegen die Scores darunter, /// wi… }` | Konfiguration für das mehrstufige iterative Multi-Step Retrieval. Steuert die Abbruchbedingungen, Qualitätsanforderungen und Zeitbudgets. | `src/multistep.rs` |
| `MultiStepEngine` | `{ collection: Arc<Collection<S>>, config: MultiStepConfig, pid_controller: parking_lot::Mutex<PidController>, clock: Arc<dyn Clock>, }` | Multi-Step Retrieval Engine für iterative Abfragen. Bietet adaptive, mehrstufige Retrieval-Zyklen auf einer zugrundeliegenden [`Collection`]. Nutzt e… | `src/multistep.rs` |
| `MultiStepResult` | `{ /// Die fusionierten Gesamtsuchergebnisse nach RRF (Reciprocal Rank Fusion). pub results: Vec<SearchResult>, /// Anzahl der tatsächlich durchlaufenen Iteration… }` | Ergebnis einer mehrstufigen Suche inklusive vollständiger Audit- und Nachvollziehbarkeitsdaten. | `src/multistep.rs` |
| `PurgeReceipt` | `{ /// Anzahl physisch gelöschter und überschriebener Chunks. pub chunks_purged: usize, /// Exakte Anzahl an Bytes, die im RAM mit Nullen überschrieben wurden. pu… }` | Quittung nach erfolgreichem [`VolatileContextVault::purge()`]. | `src/volatile_vault.rs` |
| `VaultChunk` | `{ /// Dokumenten-ID (DocId) des Chunks. #[zeroize(skip)] pub id: DocId, // DocId-Wert, ohne komplexe Drop-Interaktion /// Sensitiver Inhalt im Speicher — wird be… }` | Ein einzelner Datenblock (Chunk) im `VolatileContextVault`. Sämtliche Inhalte werden beim Drop via [`ZeroizeOnDrop`] kryptographisch überschrieben. | `src/volatile_vault.rs` |
| `VaultChunkMetadata` | `{ /// Eindeutige ID des Dokuments. pub id: DocId, /// Modalität (Text, Audio etc.). pub modality: SignalModality, /// Größe der Nutzdaten in Bytes. pub size_byte… }` | Metadaten-Ansicht eines Chunks ohne sensiblen Nutzdateninhalt. | `src/volatile_vault.rs` |
| `VaultConfig` | `{ /// Maximale Speicherkapazität aller Chunks in Bytes (Standard: 512 MB). /// Bei Überschreitung schlägt `ingest()` mit [`VaultError::CapacityExceeded`] fehl (F… }` | Konfiguration für den `VolatileContextVault`. | `src/volatile_vault.rs` |
| `VolatileContextVault` | `{ chunks: Vec<VaultChunk>, config: VaultConfig, current_bytes: usize, /// Gekapselte mlock'd Speicherbereiche. /// Wird in munlock() aufgelöst, wenn der Vault dr… }` | Ephemerer RAM-Puffer für sensitive Kontexte im Safe-Modus. # INVARIANTEN - **INV-VAULT-1:** `purge()` zeroized alle Chunk-Inhalte via `ZeroizeOnDrop`… | `src/volatile_vault.rs` |

### Öffentliche Datentypen — Enums (2)

| Enum | Varianten | Beschreibung | Datei |
|---|---|---|---|
| `SignalModality` | `/// Audiospur oder Sprach-Transkript. AudioTranscript, /// Visuelle Erfassung, Screenshot oder Bilddaten. VisualCapture, /// Reguläre Texteingabe oder Prompt. TextInput, /// Strukturierte JSON- oder…` | Modalität eines Vault-Chunks — bestimmt den Verarbeitungskontext beim späteren Commit. | `src/volatile_vault.rs` |
| `VaultError` | `/// Kapazitätsgrenze im RAM wurde überschritten (Fail-Closed, kein Disk-Spillover). #[error("Vault-Kapazität überschritten: {current_bytes} + {new_bytes} > {max_bytes}")] CapacityExceeded { current_b…` | Fehlerzustände des flüchtigen Speicher-Puffers. | `src/volatile_vault.rs` |

### Konstanten (2)

| Konstante | Typ | Beschreibung |
|---|---|---|
| `DEFAULT_MAX_SUB_QUERIES` | `usize` | Standardmäßige Obergrenze für generierte Teilfragen pro Iterationsrunde (N=3). Verhindert kombinatorische Explosion und hält Latenzschranken ein. |
| `DEFAULT_RERANK_POOL_MULTIPLIER` | `usize` | Standardmäßiger Multiplikator zur Vergrößerung des Kandidatenpools vor dem Re-Ranking. Verdoppelt die Trefferanzahl (2 * k), um genügend Recall vor F… |

### Öffentliche Funktionen (16)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn new(collection: Arc<Collection<S>>, config: MultiStepConfig) -> Self` | Erzeugt eine neue `MultiStepEngine` für eine gegebene [`Collection`]. # Parameter - `collection`: Die Contextra-Kollektion, auf der gesucht… | `src/multistep.rs` |
| `pub async fn search( &self, original_query: &str, vector: &[f32], k: usize, rewriter: Option<&dyn QueryRewriter>, ) -> Result<MultiStepResult>` | Führt eine iterative Hybrid-Suche ohne separaten Vektor-Embedder durch. # Ablauf 1. Runde 1: Standard-Hybrid-Suche mit Originalanfrage und… | `src/multistep.rs` |
| `pub async fn search_with_embedder( &self, original_query: &str, vector: &[f32], k: usize, rewriter: Option<&dyn QueryRewriter>, embedder: Option<&dyn TextEmbeddingEngine>, ) -> Result<MultiStepResult>` | Führt eine iterative multi-signale Hybrid-Suche mit Vektor-Embedder durch. # Ablauf im Detail: 1. Eingabeprüfung & Schranke: `k` wird gegen… | `src/multistep.rs` |
| `pub fn with_clock(mut self, clock: Arc<dyn Clock>) -> Self` | Überschreibt den Zeit-Port mit einem benutzerdefinierten [`Clock`]-Port. # Zweck Ermöglicht deterministische Tests ohne Abhängigkeit von re… | `src/multistep.rs` |
| `pub fn commit( mut self, tenant_id: TenantId, tx_id: TxId, deletion_proof_reference: Option<impl Into<String>>, ) -> Result<(Vec<VaultChunk>, CommitReceipt), VaultError>` | Transferiert alle Chunks zur dauerhaften Speicherung und stellt ein [`CommitReceipt`] aus. # Ablauf von `commit`: 1. Status prüfen: Falls b… | `src/volatile_vault.rs` |
| `pub fn current_size_bytes(&self) -> usize` | Gibt die Summe der aktuell im flüchtigen RAM liegenden Nutzdaten in Bytes zurück. | `src/volatile_vault.rs` |
| `pub fn ingest(&mut self, chunk: VaultChunk) -> Result<(), VaultError>` | Nimmt einen sensitiven Chunk in den RAM-Puffer auf. # Ablauf von `ingest`: 1. Status prüfen: Wurde der Vault bereits konsumiert? Falls ja:… | `src/volatile_vault.rs` |
| `pub fn is_empty(&self) -> bool` | Prüft, ob der Vault leer ist. | `src/volatile_vault.rs` |
| `pub fn is_memory_locked(&self) -> bool` | Gibt an, ob alle Chunk-Inhalte im RAM gegen Swap gesichert sind (`mlock`). Liefert `false`, falls `attempt_mlock` deaktiviert war oder das… | `src/volatile_vault.rs` |
| `pub fn len(&self) -> usize` | Gibt die Anzahl der aktuell gepufferten Chunks zurück. | `src/volatile_vault.rs` |
| `pub fn new(id: DocId, content: Vec<u8>, modality: SignalModality, captured_tx: TxId) -> Self` | Erzeugt einen neuen sensitiven Chunk. # Parameter - `id`: Eindeutige Dokumenten-ID. - `content`: Rohe Nutzdaten (z. B. Text- oder Binärdate… | `src/volatile_vault.rs` |
| `pub fn open(config: VaultConfig) -> Self` | Öffnet einen neuen leeren Vault im RAM. Führt keinerlei Datei-I/O oder Festplattenzugriffe aus. | `src/volatile_vault.rs` |
| `pub fn preview_metadata(&self) -> Vec<VaultChunkMetadata>` | Gibt eine Vorschau der Metadaten zurück, ohne sensible Inhalte zu exponieren. Verhindert versehentliches Ausleiten oder Protokollieren von… | `src/volatile_vault.rs` |
| `pub fn purge(mut self) -> PurgeReceipt` | Verwirft alle Chunks sofort und überschreibt deren Heap-Inhalte kryptographisch mit Nullen. # Ablauf von `purge`: 1. Alle Chunk-Inhalte im… | `src/volatile_vault.rs` |
| `pub fn size_bytes(&self) -> usize` | Gibt die Größe des Chunk-Inhalts in Bytes zurück. | `src/volatile_vault.rs` |
| `pub fn with_label(mut self, label: impl Into<String>) -> Self` | Weist dem Chunk eine beschreibende Bezeichnung (Label) zu. | `src/volatile_vault.rs` |


---

## `contextra`

| Attribut | Wert |
|---|---|
| **Ring** | 4 — Ring 4 — Boundary & Composition Roots |
| **Status** | Fertig |
| **Zweck** | Öffentliche Haupt-Fassade und Composition Root für Rust-Anwendungen. |
| **Unsafe-Code** | Nein (`#![forbid(unsafe_code)]`) |
| **Interne Abhängigkeiten** | `contextra-audit-export`, `contextra-avv-generator`, `contextra-crypto`, `contextra-db`, `contextra-infer-candle`, `contextra-license`, `contextra-ports`, `contextra-privacy`, `contextra-rank`, `contextra-store`, `contextra-types` |
| **Externe Abhängigkeiten (Auswahl)** | `serde`, `serde_json`, `thiserror`, `tokio`, `tracing`, `uuid`, `zeroize` |

### Architekturbeschreibung

# Modul: Contextra Primary Facade & Composition Root (Ring 4)  Dieses Modul bildet die oberste Einstiegsebene (Fassade bzw. Composition Root) des Gesamtsystems **Contextra** innerhalb der Architektur (Ring 4).  ## Zweck & Architektur Die Fassade dient als zentrale Abstraktions- und Konfigurationsschicht für anwendende Systeme (wie z. B. `contextra-mcp` oder externe SDKs/Applikationen). Sie verbirgt die interne Komplexität der tieferen Schichten (Ring 0 bis 3) und stellt eine einheitliche, ergonomische Programmierschnittstelle (API = Application Programming Interface) bereit.  ## Ring-Zugehörigkeit & Abhängigkeiten - **Ring 4 (Fassade)**: Greift als Composition Root auf Teilsysteme wie `contextra-db` (Ring 3), `contextra-types`, `contextra-ports`, `contextra-license`, `contextra-store` und `contextra-rank` zu. - **Keine zyklischen Abhängigkeiten**: Höhere Ringe hängen von niederen Ringen ab, nie umgekehrt. - **Eingebunden von**: `contextra-mcp` (Model Context Protocol Adapter für LLM-Agenten).  ## Kernkonzepte & wichtigste Typen - [`ContextraBuilder`]: Konfiguriert Speicherpfade, Vektordimensionen, Verschlüsselung, Lizenzprüfung und Profile vor dem Start der Datenbankinstanz. - [`Contextra`]: Die eigentliche Datenbankinstanz zur Verwaltung von Collections (Dokumentensammlungen), Vektorindizes und Volltextindizes. - [`AgentMemory`]: Eine agentenzentrierte High-Level-Abstraktion mit `remember()`, `recall()`, `forget()` und Wissensgraphen-Relationen (`relate()`). - [`LlmQueryRewriter`]: LLM-gestützte Umformulierung von Suchanfragen für mehrstufiges Retrieval.  ## Sicherheits- und Entwurfs-Invarianten - **Kein Unsafe**: Striktes `#![forbid(unsafe_code)]` garantiert Speichersicherheit zur Compile-Zeit. - **Kein Panik-Zustand**: Fehler werden strikt über den Ergebnistyp [`Result`] bzw. [`ContextraError`] nach oben gereicht (Fail-Closed-Prinzip). - **Entkopplung**: Zeit-, Zufalls- und ID-Generierungsoperationen erfolgen deterministisch über Ports.

### Modul- und Re-Export-Oberfläche (lib.rs)

```rust
pub mod agent_memory;
pub mod builder;
pub mod collection_profile;
pub mod performance_profile;
pub mod query_rewriter;
pub use agent_memory::{AgentMemory, Memory, MemoryId};
pub use builder::{ContextraBuilder, ContextraConfigExt, EncryptionMode};
pub use contextra_db::{ chunker, execute_background_consolidation, memory_consolidation, Collection, CollectionConfig, Contextra, ContextraConfig, ContextraStats, DriftStatusProvider, EmbeddingBackend, SearchResult, TextEmbeddingEngine, };
pub use contextra_types::error::ContextraError;
pub use contextra_types::types::domain::{DocId, ScoredDocument};
pub use contextra_types::DistanceMetric;
pub use query_rewriter::LlmQueryRewriter;
pub use contextra_rank as rank;
pub use contextra_infer_candle as candle;
```

### Feature-Flags

| Flag | Aktiviert |
|---|---|
| `default` | `["embedded"]` |
| `embedded` | `[]` |
| `compliance` | `["audit-export", "avv-generator"]` |
| `audit-export` | `["dep:contextra-audit-export"]` |
| `avv-generator` | `["dep:contextra-avv-generator"]` |
| `candle` | `["contextra-infer-candle"]` |

### Schnittstellen — Public Traits (1)

**`ContextraConfigExt`** — *Erweiterungstraits für [`ContextraConfig`] zur Abfrage des Verschlüsselungsstatus.* (`src/builder.rs`)

```rust
pub trait ContextraConfigExt {
/// Returns `true` if storage encryption is configured with a non-empty passphrase.
    fn is_encrypted(&self) -> bool;
}
```

### Öffentliche Datentypen — Structs (9)

| Struct | Felder / Form | Beschreibung | Datei |
|---|---|---|---|
| `AgentMemory` | `{ engine: Arc<Contextra>, id_gen: Option<Arc<dyn IdGen>>, }` | High-Level-Fassade für die Verwaltung des Agentengedächtnisses. Bietet die Kernoperationen: - `remember`: Speichern von Texten und Metadaten. - `reca… | `src/agent_memory.rs` |
| `CollectionProfile` | `{ pub performance: PerformanceProfile, pub kv_delete_mode: KvDeleteMode, pub lsm_tuning: LsmTuning, pub auto_extraction: AutoExtractionMode, }` | Vollständiges Profil zur Konfiguration einer Collection. Bündelt Performance-, Persistenz- und Datenschutzaspekte für eine Sammlung. | `src/collection_profile.rs` |
| `ContextraBuilder` | `{ storage_path: PathBuf, config: ContextraConfig, encryption_mode: Option<EncryptionMode>, encryption_error: Option<String>, explicit_user_config: bool, embedder… }` | Ein Builder zur flexiblen Konfiguration und Instanziierung der Datenbank [`Contextra`]. `ContextraBuilder` fungiert als primärer Einstiegspunkt (Comp… | `src/builder.rs` |
| `LlmQueryRewriter` | `{ generator: Arc<dyn LlmTextGenerator>, max_subqueries: usize, max_context_results: usize, max_snippet_chars: usize, }` | LLM-unterstützter Abfrage-Umformulierer für iteratives oder mehrstufiges Retrieval (Multi-Step Retrieval). Sendet den aktuellen Suchkontext an ein LL… | `src/query_rewriter.rs` |
| `LsmTuning` | `{ pub memtable_size_limit: u64, pub max_ram_mb: u32, pub group_commit_window_micros: u32, pub block_cache_shards: u16, }` | Feineinstellungsparameter für die LSM-Speicher-Engine (Log-Structured Merge-Tree). Regelt Speicherbudgets und Flush-Frequenzen, um Systemressourcen o… | `src/collection_profile.rs` |
| `Memory` | `{ /// Document identifier string. pub id: String, /// Relevance / similarity score. pub score: f32, /// Optional metadata associated with the memory entry. pub m… }` | Repräsentation einer aus dem Speicher abgerufenen Erinnerung (`recall`). | `src/agent_memory.rs` |
| `MemoryId` | `(tuple/unit)` | Eindeutiger, typisierter Identifikator für eine in [`AgentMemory`] abgelegte Erinnerung. Kapselt den internen Dokument-ID-String der Datenbank-Engine. | `src/agent_memory.rs` |
| `RelationId` | `(tuple/unit)` | Eindeutiger Identifikator für eine n-äre Beziehung (Hyperedge) im Wissensgraphen. Kapselt den 64-Bit-Graphenschlüssel der Wissensgraphen-Engine. | `src/agent_memory.rs` |
| `ResolvedProfileConfig` | `{ pub durability_mode: DurabilityMode, pub vector_delete_mode: VectorDeleteMode, pub deletion_proof_active: bool, pub feature_ring: FeatureRing, }` | Vollständig aufgelöste Konfigurationsparameter eines Performance-Profils. | `src/performance_profile.rs` |

### Öffentliche Datentypen — Enums (5)

| Enum | Varianten | Beschreibung | Datei |
|---|---|---|---|
| `CollectionProfileError` | `#[error("performance profile invalid: {0}")] Performance(#[from] PerformanceProfileError), #[error("deletion-proof active but KvDeleteMode::TombstoneOnly cannot satisfy a KV-side DeletionProof")] KvD…` | Mögliche Fehler bei der Validierung eines `CollectionProfile`. | `src/collection_profile.rs` |
| `DeploymentTier` | `EdgeMinimal, PowerUserLocal, EnterpriseShared, EnterpriseRegulated,` | Vordefinierte Betriebsstufen (Deployment Tiers) für typische Anwendungsszenarien. | `src/collection_profile.rs` |
| `EncryptionMode` | `/// Storage is encrypted using a key derived from a passphrase. Passphrase(Zeroizing<String>), /// Storage is explicitly unencrypted. Unencrypted,` | Konfiguration der Speicherverschlüsselung (Encryption at Rest) für persistente Datenbankschichten. Bestimmt, ob Schreib-Lese-Operationen auf Festplat… | `src/builder.rs` |
| `PerformanceProfile` | `Compliance, Balanced, BareMetal,` | Vordefinierte Leistungsprofile für Contextra-Instanzen. | `src/performance_profile.rs` |
| `PerformanceProfileError` | `#[error("durability configuration invalid: {0}")] Durability(#[from] DurabilityConfigError), #[error("DurabilityMode::Full combined with VectorDeleteMode::BackgroundRepair while deletion-proof is act…` | Fehler bei der Validierung oder Lizenzierung eines Performance-Profils. | `src/performance_profile.rs` |

### Konstanten (2)

| Konstante | Typ | Beschreibung |
|---|---|---|
| `DEFAULT_AUTO_EXTRACTION_MODE` | `AutoExtractionMode` | Standardmodus für die automatische LLM-gestützte Metadaten-/Entitäten-Extraktion bei Standard-Tiers. Standardmäßig aktiviert (`AutoExtractionMode::En… |
| `ENTERPRISE_REGULATED_AUTO_EXTRACTION_DEFAULT` | `AutoExtractionMode` | Standardmodus für `EnterpriseRegulated`. In EnterpriseRegulated wird kein Dokumenttext automatisch an ein Extraktions-LLM gegeben, solange der Betrei… |

### Öffentliche Funktionen (40)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn as_str(&self) -> &str` | Returns a string slice reference to the underlying memory ID. | `src/agent_memory.rs` |
| `pub fn engine(&self) -> &Contextra` | Liefert eine Referenz auf die zugrundeliegende [`Contextra`]-Engine. | `src/agent_memory.rs` |
| `pub async fn forget(&self, id: &MemoryId) -> Result<(), ContextraError>` | Entfernt eine Erinnerung anhand ihrer [`MemoryId`]. Delegiert direkt an `Contextra::delete`. | `src/agent_memory.rs` |
| `pub fn new(id: impl Into<String>) -> Self` | Creates a new `MemoryId` from a string identifier. | `src/agent_memory.rs` |
| `pub fn new(engine: Contextra) -> Self` | Erstellt eine neue `AgentMemory`-Instanz, indem die übergebene `Contextra`-Engine in ein `Arc` gehüllt wird. | `src/agent_memory.rs` |
| `pub fn new_arc(engine: Arc<Contextra>) -> Self` | Erstellt eine neue `AgentMemory`-Instanz um ein bereits geteiltes `Arc<Contextra>`. | `src/agent_memory.rs` |
| `pub fn new_with_id_gen(engine: Contextra, id_gen: Arc<dyn IdGen>) -> Self` | Erstellt eine Instanz mit einem explizit injizierten ID-Generator-Port. | `src/agent_memory.rs` |
| `pub async fn recall(&self, query: &str, k: usize) -> Result<Vec<Memory>, ContextraError>` | Ruft die Top-k relevantesten Erinnerungen zu einer Suchanfrage ab. Delegiert direkt an `Contextra::search_text` und transformiert Ergebniss… | `src/agent_memory.rs` |
| `pub async fn relate( &self, from: &MemoryId, to: &MemoryId, label: &str, ) -> Result<(), ContextraError>` | Erstellt eine gerichtete Beziehung zwischen zwei Erinnerungen im Wissensgraphen. # Parameter - `from`: Quell-Erinnerung. - `to`: Ziel-Erinn… | `src/agent_memory.rs` |
| `pub async fn relate_n_ary( &self, predicate: &str, participants: &[(&MemoryId, &str)], source: Option<&MemoryId>, ) -> Result<RelationId, ContextraError>` | Erstellt eine n-äre Beziehung (Hyperedge) zwischen zwei oder mehr Erinnerungen. # Parameter - `predicate`: Semantische Bezeichnung der Bezi… | `src/agent_memory.rs` |
| `pub async fn remember( &self, text: &str, metadata: Option<Value>, ) -> Result<MemoryId, ContextraError>` | Speichert eine Text-Erinnerung mit optionalen JSON-Metadaten. # Ablauf von `remember`: 1. ID ermitteln: Über den injizierten `IdGen`-Port o… | `src/agent_memory.rs` |
| `pub fn with_id_gen(mut self, id_gen: Arc<dyn IdGen>) -> Self` | Injiziert oder aktualisiert den ID-Generator nachträglich (Builder-Muster). | `src/agent_memory.rs` |
| `pub async fn build(self) -> Result<Contextra, ContextraError>` | Baut und initialisiert die [`Contextra`]-Datenbankinstanz asynchron. # Ablauf von `build`: 1. **Vorabprüfung**: Prüfe, ob frühere Fehler (z… | `src/builder.rs` |
| `pub fn is_encrypted(&self) -> bool` | Prüft, ob der Builder aktuell auf verschlüsselten Modus mit Passphrase eingestellt ist. | `src/builder.rs` |
| `pub fn new(dimension: usize) -> Self` | Erzeugt einen neuen `ContextraBuilder` mit der angegebenen Vektordimension. # Parameter - `dimension`: Anzahl der Dimensionen für Vektoren… | `src/builder.rs` |
| `pub fn with_config(mut self, mut config: ContextraConfig) -> Self` | Überschreibt die Konfiguration vollständig mit einer bestehenden [`ContextraConfig`]. Überträgt auch eventuell in `config` enthaltene Passp… | `src/builder.rs` |
| `pub fn with_consolidation(mut self, enabled: bool, interval: Duration) -> Self` | Konfiguriert die automatische Hintergrund-Konsolidierung (Memory Consolidation) von Erinnerungen. # Parameter - `enabled`: Schaltet den per… | `src/builder.rs` |
| `pub fn with_distance_metric(mut self, metric: DistanceMetric) -> Self` | Legt die Distanzmetrik für die Ähnlichkeitssuche fest. # Parameter - `metric`: Metrik wie Kosinus-Ähnlichkeit (`Cosine`), Euklidischer Abst… | `src/builder.rs` |
| `pub fn with_embedder(mut self, embedder: Arc<dyn TextEmbeddingEngine>) -> Self` | Bindet eine benutzerdefinierte Implementierung des [`TextEmbeddingEngine`]-Ports ein. | `src/builder.rs` |
| `pub fn with_embedding_backend(mut self, backend: EmbeddingBackend) -> Self` | Wählt das Backend für Text-Embeddings (z. B. intern via Candle oder externer Dienst). | `src/builder.rs` |
| `pub fn with_encryption(mut self, mode: EncryptionMode) -> Self` | Konfiguriert den Verschlüsselungsmodus explizit. Validiert bei Passphrasen sofort, dass diese nicht leer sind oder nur aus Leerzeichen best… | `src/builder.rs` |
| `pub fn with_encryption_passphrase(self, passphrase: impl Into<String>) -> Self` | Bequeme Methode zum Setzen einer Passphrase für die Speicherverschlüsselung. Kapselt die Passphrase direkt in einem [`Zeroizing<String>`],… | `src/builder.rs` |
| `pub fn with_license_gate(mut self, gate: Arc<dyn LicenseGate>) -> Self` | Injiziert ein beliebiges Lizenzgate ([`LicenseGate`]) zur Ring-Autorisierung. | `src/builder.rs` |
| `pub fn with_max_elements(mut self, max_elements: usize) -> Self` | Begrenzt die maximale Anzahl an Vektorelementen im Index. # Parameter - `max_elements`: Maximalwert (z. B. 100_000). | `src/builder.rs` |
| `pub fn with_performance_profile( mut self, profile: crate::performance_profile::PerformanceProfile, ) -> Self` | Setzt ein vordefiniertes Performance-Profil ([`PerformanceProfile`]). Profile (wie `Compliance`, `Balanced`, `BareMetal`) steuern standardi… | `src/builder.rs` |
| `pub fn with_signed_license( mut self, payload_bytes: &[u8], signature: &[u8; 64], verifying_key_bytes: &[u8; 32], ) -> Self` | Konfiguriert ein kryptografisch signiertes Lizenzgate (Ed25519-Signaturprüfung). Überprüft die Signatur des Lizenz-Payloads mit dem angegeb… | `src/builder.rs` |
| `pub fn with_storage_path(mut self, path: impl Into<PathBuf>) -> Self` | Setzt das Speicherverzeichnis für die Datenbankdateien (LSM-SSTables, WAL, Indizes). # Parameter - `path`: Pfad im Dateisystem (z. B. `/var… | `src/builder.rs` |
| `pub fn resolve(self) -> CollectionProfile` | Löst die Betriebsstufe in ein konkretes, vorkonfiguriertes [`CollectionProfile`] auf. Garantiert stets ein konsistentes Profil gemäß Invari… | `src/collection_profile.rs` |
| `pub fn validate(&self) -> Result<(), CollectionProfileError>` | Validiert die interne Konsistenz des Profils. # Ablauf von `validate`: 1. Performance-Profil auflösen und dessen Konfiguration prüfen (`res… | `src/collection_profile.rs` |
| `pub fn validate_with_license( &self, gate: &dyn LicenseGate, ) -> Result<(), CollectionProfileError>` | Validiert das Profil zusätzlich gegen ein Lizenzgate. Stellt sicher, dass das Profil intern konsistent ist und der geforderte Feature-Ring… | `src/collection_profile.rs` |
| `pub fn builder(dimension: usize) -> ContextraBuilder` | Erzeugt einen neuen [`ContextraBuilder`] zur Konfiguration und Instanziierung der Datenbank [`Contextra`]. # Parameter - `dimension`: Die A… | `src/lib.rs` |
| `pub async fn open(path: impl AsRef<std::path::Path>) -> Result<Contextra, ContextraError>` | Öffnet oder erstellt eine [`Contextra`]-Datenbankinstanz am angegebenen Dateisystempfad mit Standardkonfiguration. Verwendet standardmäßig… | `src/lib.rs` |
| `pub async fn open_with_config( path: impl AsRef<std::path::Path>, config: ContextraConfig, ) -> Result<Contextra, ContextraError>` | Öffnet oder erstellt eine [`Contextra`]-Datenbankinstanz mit einer explizit übergebenen Konfiguration. # Parameter - `path`: Dateipfad zum… | `src/lib.rs` |
| `pub fn enforce_license(&self, gate: &dyn LicenseGate) -> Result<(), PerformanceProfileError>` | Prüft den geforderten Feature-Ring gegen das übergebene [`LicenseGate`]. Der Basisring `FeatureRing::Fast` ist stets lizenzfrei erlaubt. Hö… | `src/performance_profile.rs` |
| `pub fn resolve(self) -> ResolvedProfileConfig` | Löst das Profil in konkrete Konfigurationsparameter auf. # Ablauf von `resolve`: - `Compliance`: Full Durability, SynchronousRepair, Deleti… | `src/performance_profile.rs` |
| `pub fn validate(&self) -> Result<(), PerformanceProfileError>` | Validiert die gegenseitige Kompatibilität der Profilparameter. Stellt sicher, dass Durability und Vektor-Löschung nicht in einen ungültigen… | `src/performance_profile.rs` |
| `pub fn new(generator: Arc<dyn LlmTextGenerator>) -> Self` | Erstellt einen neuen `LlmQueryRewriter` mit Standardgrenzwerten: - `max_subqueries`: 3 alternative Abfragen - `max_context_results`: 3 Doku… | `src/query_rewriter.rs` |
| `pub fn with_max_context_results(mut self, n: usize) -> Self` | Konfiguriert die maximale Anzahl an bestehenden Suchergebnissen, die als Kontext in den Prompt fließen. | `src/query_rewriter.rs` |
| `pub fn with_max_snippet_chars(mut self, n: usize) -> Self` | Konfiguriert die maximale Textlänge (in Zeichen) für Textausschnitte (Snippets) im Prompt. | `src/query_rewriter.rs` |
| `pub fn with_max_subqueries(mut self, n: usize) -> Self` | Konfiguriert die maximale Anzahl an generierten Teilabfragen. | `src/query_rewriter.rs` |


---

## `contextra-mcp`

| Attribut | Wert |
|---|---|
| **Ring** | 4 — Ring 4 — Boundary & Composition Roots |
| **Status** | In Migration |
| **Zweck** | Stdio-JSON-RPC MCP-Server-Protokoll-Adapter. |
| **Unsafe-Code** | Nein (`#![forbid(unsafe_code)]`) |
| **Interne Abhängigkeiten** | `contextra`, `contextra-crypto`, `contextra-infer-candle`, `contextra-infer-ollama`, `contextra-infer-onnx`, `contextra-license`, `contextra-ports`, `contextra-privacy`, `contextra-rank`, `contextra-router`, `contextra-types`, `contextra-wire` |
| **Externe Abhängigkeiten (Auswahl)** | `futures-util`, `parking_lot`, `rand`, `scc`, `serde`, `serde_json`, `thiserror`, `tokio`, `tokio-util`, `tracing`, `tracing-subscriber`, `unicode-normalization`, `zeroize` |

### Modul- und Re-Export-Oberfläche (lib.rs)

```rust
pub mod bulk_exfiltration_detector;
pub mod config;
pub mod egress_gateway;
pub mod egress_guard;
pub mod explain;
pub mod io;
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
pub use proof_key::deletion_proof_key_from_env;
pub use io::{read_line_bounded, MAX_RPC_BYTES, MAX_SEARCH_QUERY_BYTES};
pub use prompt_injection::{ PromptInjectionConfig, PromptInjectionGuard, QuarantinePolicy, SecurityAuditLogger, SecurityAuditRecord, DEFAULT_REDACTION_PLACEHOLDER, };
pub use routing::{setup_routing, RoutingHandle};
pub use server::McpServer;
pub use validation::{is_write_allowed_by_env, validate_collection_name};
```

### Feature-Flags

| Flag | Aktiviert |
|---|---|
| `default` | `[]` |
| `candle` | `["dep:contextra-infer-candle"]` |
| `ollama` | `["dep:contextra-infer-ollama"]` |
| `onnx` | `["dep:contextra-infer-onnx"]` |
| `test-utils` | `[]` |

### Öffentliche Datentypen — Structs (20)

| Struct | Felder / Form | Beschreibung | Datei |
|---|---|---|---|
| `CollectionSearchEngine` | `(tuple/unit)` | Adapter implementing `TextSearchEngine` for `contextra::Collection`. | `src/egress_guard.rs` |
| `EmbeddingConfig` | `{ /// Provider type ("ollama", "onnx", "candle", or "mock"). Unconfigured when empty. pub provider: String, /// Base URL for Ollama HTTP API. pub ollama_url: Str… }` | Embedding provider configuration settings. | `src/config.rs` |
| `ExplainRequest` | `{ /// Target document identifier. pub id: String, /// Target collection name (defaults to "default"). #[serde(default = "default_collection")] pub collection: St… }` | Request payload for `contextra_explain`. | `src/explain.rs` |
| `ExplainResponse` | `{ /// Document identifier. pub id: String, /// Collection name. pub collection: String, /// Detailed provenance record if available. #[serde(default, skip_serial… }` | Response payload for `contextra_explain`. | `src/explain.rs` |
| `LlmConfig` | `{ /// Provider type ("ollama", "candle", or "mock"). Unconfigured when empty. pub provider: String, /// Base URL for Ollama HTTP API. pub ollama_url: String, ///… }` | LLM provider configuration settings. | `src/config.rs` |
| `McpSandbox` | `{ policy: SandboxPolicy, /// Session-lokale volatile Ergebnisse (werden bei Session-Ende gedropt). volatile_results: Mutex<HashMap<String, VolatileToolResult>>,… }` | MCP Sandbox: validiert Tool-Calls und verwaltet volatile Ergebnisse. | `src/sandbox.rs` |
| `McpServer` | `{ pub db: Arc<Contextra>, pub embedder: Arc<dyn EmbeddingProvider>, pub sandbox: Arc<McpSandbox>, pub injection_guard: Arc<PromptInjectionGuard>, pub egress_clas… }` | — | `src/server.rs` |
| `MockEmbedder` | `{ dimension: usize, all_zero: bool, }` | Fallback Mock Embedder (Test & dev utilities only). | `src/config.rs` |
| `MockLlmGenerator` | `(tuple/unit)` | Fallback Mock LLM Text Generator (Test & dev utilities only). | `src/config.rs` |
| `NormalizedPattern` | `{ pub original: String, pub collapsed: String, pub no_ws: String, }` | Vornormalisiertes Injection-Pattern zur Performance-Optimierung. | `src/prompt_injection/guard.rs` |
| `PromptInjectionConfig` | `{ #[serde(default)] pub policy: QuarantinePolicy, #[serde(default = "default_redaction_placeholder")] pub redaction_placeholder: String, #[serde(default)] pub au… }` | Konfiguration zur Initialisierung aus einer externen Konfigurationsdatei. | `src/prompt_injection/policy.rs` |
| `PromptInjectionGuard` | `{ policy: QuarantinePolicy, pub redaction_placeholder: String, patterns: Vec<String>, normalized_patterns: Vec<NormalizedPattern>, audit_logger: SecurityAuditLog… }` | Signatur- und phrasenbasierter Prompt-Injection-Erkennungsfilter. Erkennt bekannte Angriffsmuster (direkte Phrasen, whitespace-verschleierte Variante… | `src/prompt_injection/guard.rs` |
| `ProofKeyResolution` | `{ pub key: Option<Zeroizing<String>>, pub source: Option<ProofKeySource>, pub has_conflict: bool, pub used_legacy: bool, }` | Ergebnis der Schlüssel-Auflösung. | `src/proof_key_env.rs` |
| `RouterConfig` | `{ /// List of SLM profiles. If non-empty, routing is enabled. pub profiles: Vec<contextra_router::SlmProfile>, /// Optional path to routing profiles JSON file. p… }` | Router configuration settings. | `src/config.rs` |
| `RoutingHandle` | `{ pub router: Arc<contextra_router::DefaultRouterEngine>, pub calibrator: Arc<parking_lot::Mutex<contextra_rank::IsotonicCalibrator>>, pub _drift_adapter: Arc<dy… }` | Holds strong Arc references to routing and calibration components to maintain live Weak references in `Contextra`. | `src/routing.rs` |
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

### Öffentliche Funktionen (74)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn available_compiled_providers() -> Vec<&'static str>` | Returns the list of currently compiled provider identifiers. | `src/config.rs` |
| `pub fn build_generator(&self) -> Result<Arc<dyn LlmTextGenerator>, ContextraError>` | Instantiates the configured `LlmTextGenerator` as an `Arc<dyn LlmTextGenerator>`. | `src/config.rs` |
| `pub fn build_provider(&self) -> Result<Arc<dyn EmbeddingProvider>, ContextraError>` | Instantiates the configured `EmbeddingProvider` as an `Arc<dyn EmbeddingProvider>` and runs startup probe. | `src/config.rs` |
| `pub fn create_embedding_provider( provider_type: Option<&str>, ollama_url: &str, embed_model: &str, onnx_model_path: Option<&Path>, candle_model_dir: Option<&Path>, ) -> Result<Arc<dyn EmbeddingProvider>, ContextraError>` | Dynamically constructs an `EmbeddingProvider` implementation based on provider identifier. | `src/config.rs` |
| `pub fn create_llm_text_generator( provider_type: Option<&str>, ollama_url: &str, llm_model: &str, candle_model_dir: Option<&Path>, ) -> Result<Arc<dyn LlmTextGenerator>, ContextraError>` | Dynamically constructs an `LlmTextGenerator` implementation based on provider identifier. | `src/config.rs` |
| `pub fn from_env() -> Self` | Loads configuration from environment variables with fallbacks. | `src/config.rs` |
| `pub fn from_env() -> Self` | Loads configuration from environment variables with fallbacks. | `src/config.rs` |
| `pub fn from_env() -> Self` | Loads configuration from environment variables with fallbacks. | `src/config.rs` |
| `pub fn new(dimension: usize) -> Self` | — | `src/config.rs` |
| `pub fn provider_opt(&self) -> Option<&str>` | — | `src/config.rs` |
| `pub fn provider_opt(&self) -> Option<&str>` | — | `src/config.rs` |
| `pub fn verify_embedding_provider( provider: &dyn EmbeddingProvider, expected_dim: Option<usize>, ) -> Result<(), ContextraError>` | Startup embedding self-test (probe) verifying provider functionality. | `src/config.rs` |
| `pub fn with_expected_dim(mut self, dim: usize) -> Self` | — | `src/config.rs` |
| `pub fn with_zero_vectors(dimension: usize) -> Self` | — | `src/config.rs` |
| `pub async fn read_line_bounded<R: tokio::io::AsyncBufRead + Unpin>( reader: &mut R, buf: &mut String, max_bytes: usize, ) -> std::io::Result<usize>` | Reads a single line from an async reader into `buf` up to `max_bytes`. If the line exceeds `max_bytes`, consumes and discards the remainder… | `src/io.rs` |
| `pub fn get_recorded_events(&self) -> Vec<SecurityAuditRecord>` | — | `src/prompt_injection/audit.rs` |
| `pub fn log_event(&self, record: SecurityAuditRecord)` | — | `src/prompt_injection/audit.rs` |
| `pub fn new(file_path: Option<PathBuf>) -> Self` | — | `src/prompt_injection/audit.rs` |
| `pub fn audit_logger(&self) -> &SecurityAuditLogger` | — | `src/prompt_injection/guard.rs` |
| `pub fn clock(&self) -> &Arc<dyn Clock>` | — | `src/prompt_injection/guard.rs` |
| `pub fn collapse_whitespace(s: &str) -> String` | Collapsiert aufeinanderfolgende Whitespaces zu einem einzelnen Leerzeichen. | `src/prompt_injection/guard.rs` |
| `pub fn decode_base64(input: &str) -> Option<Vec<u8>>` | Versucht, einen Base64-String (Standard oder URL-Safe) ohne Panics zu dekodieren. | `src/prompt_injection/guard.rs` |
| `pub fn default_patterns() -> Vec<String>` | Gibt die Standard-Erkennungsmuster zurück. Enthält: - 16 englische Kern-Patterns (Token-Marker, direkte Befehle) - 9 deutsche Patterns (Pro… | `src/prompt_injection/guard.rs` |
| `pub fn detect(&self, text: &str) -> Option<String>` | Prüft den Eingabetext auf bekannte Prompt-Injection-Muster unter Verwendung von Normalisierung, Whitespace-Analysen und rekursiver Base64-D… | `src/prompt_injection/guard.rs` |
| `pub fn detect_recursive(&self, text: &str, depth: usize) -> Option<String>` | Rekursive Injektions-Erkennung mit harter Tiefenbegrenzung (max. Tiefe 2 für Base64-Dekodierung). | `src/prompt_injection/guard.rs` |
| `pub fn extract_base64_candidates(text: &str) -> Vec<&str>` | Extrahiert kandidate Base64-Substrings (Länge >= 16) aus einem Eingabetext. | `src/prompt_injection/guard.rs` |
| `pub fn from_env() -> Self` | Erstellt eine Instanz basierend auf Umgebungsvariablen. | `src/prompt_injection/guard.rs` |
| `pub fn is_zero_width(c: char) -> bool` | Prüft ob ein Zeichen ein Zero-Width- oder unsichtbares Steuer-Zeichen ist. | `src/prompt_injection/guard.rs` |
| `pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, String>` | Lädt die Konfiguration aus einer externen JSON- oder TOML-Datei. | `src/prompt_injection/guard.rs` |
| `pub fn new( policy: QuarantinePolicy, redaction_placeholder: String, patterns: Vec<String>, audit_logger: SecurityAuditLogger, ) -> Self` | — | `src/prompt_injection/guard.rs` |
| `pub fn normalize_text(text: &str) -> String` | Normalisiert den Eingabetext (Zero-Width-Stripping, NFKC Normalisierung, Skeletonisierung, Lowercasing). | `src/prompt_injection/guard.rs` |
| `pub fn patterns(&self) -> &[String]` | — | `src/prompt_injection/guard.rs` |
| `pub fn policy(&self) -> QuarantinePolicy` | — | `src/prompt_injection/guard.rs` |
| `pub fn process_result( &self, doc_id: &str, collection: &str, obj: &mut serde_json::Map<String, serde_json::Value>, ) -> bool` | Prüft und verarbeitet das JSON-Ergebnisobjekt eines Such- oder Get-Aufrufs. Wendet die konfigurierte `QuarantinePolicy` an und liefert `tru… | `src/prompt_injection/guard.rs` |
| `pub fn skeletonize_char(c: char) -> char` | Mappt gängige kyrillische und griechische Homoglyphen (Confusables) auf ihre lateinischen Äquivalente. | `src/prompt_injection/guard.rs` |
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
| `pub fn new( db: Arc<Contextra>, embedder: Arc<dyn EmbeddingProvider>, ) -> Result<Self, ContextraError>` | — | `src/server.rs` |
| `pub async fn run_stdio(self: Arc<Self>) -> Result<(), Box<dyn std::error::Error>>` | Startet den MCP stdio-Loop. Liest zeilenweise von stdin, dispatcht JSON-RPC-Requests und schreibt Antworten als einzelne JSON-Zeile nach st… | `src/server.rs` |
| `pub fn with_egress_classifier(mut self, classifier: Arc<dyn EgressClassifier>) -> Self` | — | `src/server.rs` |
| `pub fn with_injection_guard(mut self, injection_guard: Arc<PromptInjectionGuard>) -> Self` | — | `src/server.rs` |
| `pub fn with_plugin_registry( mut self, registry: Option<Arc<contextra_ports::plugin::PluginRegistry>>, ) -> Self` | — | `src/server.rs` |
| `pub fn with_routing(mut self, routing: Option<Arc<RoutingHandle>>) -> Self` | — | `src/server.rs` |
| `pub fn with_sandbox( db: Arc<Contextra>, embedder: Arc<dyn EmbeddingProvider>, sandbox: Arc<McpSandbox>, ) -> Self` | — | `src/server.rs` |
| `pub fn with_tenant_id(mut self, tenant_id: Option<TenantId>) -> Self` | — | `src/server.rs` |
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
| **Interne Abhängigkeiten** | `contextra-adapt`, `contextra-db`, `contextra-rank`, `contextra-store`, `contextra-types`, `contextra-wire` |
| **Externe Abhängigkeiten (Auswahl)** | `flatbuffers`, `numpy`, `parking_lot`, `pyo3`, `pythonize`, `serde_json`, `tokio` |

### Architekturbeschreibung

# Contextra Python Bindings (`contextra-py`)  Dieses Crate stellt die offizielle Python-Brücke (FFI = Foreign Function Interface) für die eingebettete 4-Signal Hybrid-Search Vektordatenbank **Contextra** bereit. Es verbindet das Python-Ökosystem via PyO3 mit dem Rust-Kern (`contextra-db`, `contextra-store`).  ## Ring-Zugehörigkeit & Architektur  - **Ring / Klasse:** Ring 4 (Ränder / FFI-Bindings). Laut SPEC §5.2 / CLAUDE.md ist `contextra-py` zwar formell archiviert bzw. an der Peripherie angesiedelt, wird jedoch weiterhin als vollwertiges Workspace-Mitglied geführt. - **Abhängigkeiten (eingehend):** Keine Workspace-Crates hängen von `contextra-py` ab. Die Nutzung erfolgt ausschließlich durch Python-Clients / User-Space. - **Abhängigkeiten (ausgehend):** `contextra-types`, `contextra-store`, `contextra-db`, `contextra-rank`, `contextra-adapt`, `contextra-wire` sowie Third-Party Crates wie `pyo3`, `numpy`, `tokio`, `parking_lot`, `flatbuffers`, `serde_json`, `pythonize`.  ## Wichtige Invarianten für Entwickler  1. **Keine Panics über FFI-Grenzen (ADR-N04):** Rust-Panics dürfen unter keinen Umständen in den Python-Interpreter durchschlagen (dies würde den gesamten Prozess crashen). Alle Aufrufe werden über `run_blocking_ffi` abgesichert und in strukturierte Python-Exceptions konvertiert. 2. **GIL-Release (Global Interpreter Lock):** Während I/O-lastiger oder rechenintensiver Rust-Aufrufe (`block_on`) wird der Python GIL via `py.allow_threads` freigegeben, damit andere Python-Threads nicht blockiert werden. 3. **Sub-Interpreter-Isolation (PEP 684):** Die Tokio-Runtime (`PyRuntimeState`) wird pro Python-Modulinstanz verwaltet, um globale Zustände zu vermeiden. 4. **Zero-Copy Vektor-Umschlag:** Input-Vektoren werden über `numpy::PyReadonlyArray1<f32>` als `&[f32]` ohne Kopie direkt an die Speicher-/Indexschichten übergeben.

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
| `test-hooks` | `[]` |

### Öffentliche Datentypen — Structs (8)

| Struct | Felder / Form | Beschreibung | Datei |
|---|---|---|---|
| `PyCollection` | `{ pub(crate) inner: Arc<ContextraCollection>, pub(crate) runtime: Arc<Runtime>, pub(crate) poisoned: Arc<AtomicBool>, }` | Kapselt eine benannte Contextra-Dokumentensammlung für Python. # Felder - `inner`: Thread-sicherer Zeiger auf die interne `ContextraCollection`. - `r… | `src/bindings/collection.rs` |
| `PyContextra` | `{ pub(crate) inner: Arc<Contextra>, pub(crate) runtime: Arc<Runtime>, pub(crate) worker_threads: usize, pub(crate) poisoned: Arc<AtomicBool>, }` | Zentrales Python-Handle für eine geöffnete Contextra-Datenbankinstanz (`Db`). # Felder - `inner`: Thread-sichere Referenz auf den Rust-Datenbankkern… | `src/bindings/db.rs` |
| `PyDbStats` | `{ /// Lyapunov drift status ("stabil", "warnung", "kritisch", "unbekannt"). pub drift_status: String, /// Expected Calibration Error (ECE) from IsotonicCalibrato… }` | Ganzheitliche Datenbank- und Systembeobachtungs-Statistiken (`DbStats`). # Felder - `drift_status`: Zustand der Vektor-Driftanalyse ("stabil", "warnu… | `src/bindings/db_stats.rs` |
| `PyDocument` | `{ /// Die eindeutige Dokumenten-ID. pub id: String, /// Optionale Metadaten, die mit dem Dokument verknüpft sind. pub metadata: Option<PyObject>, }` | Ein aus Contextra abgerufener Datensatz (Dokument). # Felder - `id`: Der Primärschlüssel des Dokuments als String. - `metadata`: Assoziierte Metadate… | `src/bindings/document.rs` |
| `PyRuntimeState` | `{ pub runtime: Arc<Runtime>, pub worker_threads: usize, }` | Kapselt die Tokio-Laufzeitumgebung und die Thread-Konfiguration pro Python-Modul. # Felder - `runtime`: Referenzgezählter Handle (`Arc<Runtime>`) auf… | `src/bindings/runtime_state.rs` |
| `PySearchResult` | `{ /// Die Dokumenten-ID. pub id: String, /// Der Ähnlichkeits-Score (höher = relevanter). pub score: f32, /// Optionale Metadaten des Dokuments. pub metadata: Op… }` | Ein einzelnes Suchergebnis aus einer Vektor- oder Hybrid-Suche. # Felder - `id`: Der Primärschlüssel des gefundenen Dokuments. - `score`: Der berechn… | `src/bindings/search_result.rs` |
| `PyStorageStats` | `{ /// Number of SSTable segments. pub num_segments: usize, /// Total size of all SSTables in bytes. pub total_size_bytes: u64, /// Total size of memtables in byt… }` | Statistiken der persistenten LSM-Speicherengine (`StorageStats`). # Felder - `num_segments`: Anzahl der auf der Festplatte persistierten SSTable-Segm… | `src/bindings/storage_stats.rs` |
| `PyVectorIndexStats` | `{ /// Number of active (non-deleted) vectors. pub num_vectors: usize, /// Estimated memory usage in bytes. pub memory_usage_bytes: usize, /// Number of HNSW laye… }` | Statistiken über den HNSW-Vektorindex (`VectorIndexStats`). # Felder - `num_vectors`: Anzahl der aktuell aktiven (nicht als gelöscht markierten) Vekt… | `src/bindings/vector_index_stats.rs` |

### Konstanten (6)

| Konstante | Typ | Beschreibung |
|---|---|---|
| `MAX_BATCH_SIZE` | `usize` | Maximale Batch-Größe für Massenoperationen (10.000 Einträge). |
| `MAX_COLLECTION_NAME_LENGTH` | `usize` | Maximale Länge für Collection-Namen (64 Zeichen). |
| `MAX_ID_LENGTH` | `usize` | Maximale Länge für Dokumenten-IDs (1024 Zeichen bzw. Bytes). |
| `MAX_LABEL_LENGTH` | `usize` | Maximale Länge für Relations-Labels (256 Zeichen). |
| `MAX_WORKER_THREADS` | `usize` | Maximale Anzahl an Worker-Threads für die Tokio-Laufzeitumgebung. |
| `MIN_WORKER_THREADS` | `usize` | Minimale Anzahl an Worker-Threads für die Tokio-Laufzeitumgebung. |

### Öffentliche Funktionen (60)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn _trigger_panic_for_test(&self, py: Python<'_>, message: Option<String>) -> PyResult<()>` | Interner Testhook zur Verifikation von FFI-Panic-Isolation und Engine-Poisoning. # Panics Bewusstes Auslösen eines Panics zum Testen der Ba… | `src/bindings/collection.rs` |
| `pub fn is_empty(&self, py: Python<'_>) -> PyResult<bool>` | Prüft, ob die Collection leer ist (`True` falls keine Dokumente enthalten sind). | `src/bindings/collection.rs` |
| `pub fn is_poisoned(&self) -> bool` | Gibt `True` zurück, wenn die Collection aufgrund eines vorherigen Rust-Panics vergiftet wurde. # Invariante (Fail-Closed) Sobald `is_poison… | `src/bindings/collection.rs` |
| `pub fn len(&self, py: Python<'_>) -> PyResult<usize>` | Gibt die Gesamtzahl der Dokumente in dieser Collection zurück. | `src/bindings/collection.rs` |
| `pub fn new( inner: Arc<ContextraCollection>, runtime: Arc<Runtime>, poisoned: Arc<AtomicBool>, ) -> Self` | Erzeugt eine neue `PyCollection`-Instanz. | `src/bindings/collection.rs` |
| `pub fn stats(&self, py: Python<'_>) -> PyResult<PyVectorIndexStats>` | Liefert detaillierte Statistiken über den HNSW-Vektorindex dieser Collection. # Ablauf 1. Asynchron `inner.stats()` unter GIL-Release ausfü… | `src/bindings/collection.rs` |
| `pub fn check_subinterpreter_guard(py: Python<'_>) -> PyResult<()>` | Prüft explizit, ob das Crate innerhalb eines CPython Sub-Interpreters geladen wird. # Zweck Verhindert fehlerhafte Ausführung in isolierten… | `src/bindings/common.rs` |
| `pub fn contextra_err(e: contextra_types::ContextraError) -> PyErr` | Übersetzt ein typisiertes Rust `ContextraError` in eine strukturierte Python-Exception (`PyErr`). # Ablauf 1. Wandelt `ContextraError` in e… | `src/bindings/common.rs` |
| `pub fn dict_to_json(d: &pyo3::Bound<'_, pyo3::types::PyDict>) -> PyResult<serde_json::Value>` | Konvertiert ein Python-Dictionary (`PyDict`) in einen `serde_json::Value`. # Zweck Wandelt dynamische Metadaten aus Python in das interne J… | `src/bindings/common.rs` |
| `pub fn doc_to_py(py: Python<'_>, d: contextra_db::Document) -> PyResult<PyDocument>` | Konvertiert ein `contextra_db::Document` in ein `PyDocument`. # Ablauf 1. Metadaten-JSON (falls vorhanden) in ein natives Python-Dictionary… | `src/bindings/common.rs` |
| `pub fn json_to_py(py: Python<'_>, val: &serde_json::Value) -> PyResult<PyObject>` | Konvertiert einen `serde_json::Value` zurück in ein natives Python-Objekt (`PyObject`). # Fehlerfälle Liefert `PyRuntimeError`, falls die K… | `src/bindings/common.rs` |
| `pub fn opt_dict_to_json( metadata: Option<&pyo3::Bound<'_, pyo3::types::PyDict>>, ) -> PyResult<Option<serde_json::Value>>` | Konvertiert ein optionales Python-Dictionary in ein optionales `serde_json::Value`. # Rückgabe - `Ok(Some(Value))` wenn ein Dictionary über… | `src/bindings/common.rs` |
| `pub fn results_to_py( py: Python<'_>, results: Vec<contextra_db::SearchResult>, ) -> PyResult<Vec<PySearchResult>>` | Konvertiert einen Vektor von internen `contextra_db::SearchResult` in Python-taugliche `PySearchResult`. # Ablauf 1. Allokiert einen Vektor… | `src/bindings/common.rs` |
| `pub fn run_blocking_ffi<F, R>(py: Python<'_>, poisoned: &AtomicBool, f: F) -> PyResult<R> where F: FnOnce() -> PyResult<R> + Send, R: Send,` | Führt einen blockierenden Aufruf sicher über die FFI-Grenze hinweg aus (mit Panic-Catch und Poisoning). # Zweck & Invariante (ADR-N04 / APM… | `src/bindings/common.rs` |
| `pub fn validate_batch_size(size: usize) -> PyResult<()>` | Prüft, dass eine Batch-Größe größer 0 ist und das Limit von 10.000 Elementen nicht überschreitet. # Fehlerfälle Liefert `ContextraValueErro… | `src/bindings/common.rs` |
| `pub fn validate_collection_name(name: &str) -> PyResult<()>` | Validiert, dass ein Collection-Name nicht leer ist, keine Null-Bytes enthält und maximal 64 Zeichen lang ist. # Fehlerfälle Liefert `Contex… | `src/bindings/common.rs` |
| `pub fn validate_db_path(path: &str) -> PyResult<()>` | Validiert, dass ein Datenbank-Dateipfad nicht leer ist und keine Null-Bytes enthält. # Fehlerfälle Liefert `ContextraValueError` bei fehler… | `src/bindings/common.rs` |
| `pub fn validate_id(id: &str) -> PyResult<()>` | Validiert, dass eine Dokumenten-ID nicht leer ist, keine Null-Bytes enthält und maximal 1024 Zeichen lang ist. # Fehlerfälle Liefert `Conte… | `src/bindings/common.rs` |
| `pub fn validate_id_obj(id_obj: &pyo3::Bound<'_, pyo3::types::PyAny>) -> PyResult<String>` | Validiert und extrahiert ein Dokument-ID-Objekt aus Python (erlaubt String oder nicht-negativen Integer). # Ablauf 1. Prüft, ob das Objekt… | `src/bindings/common.rs` |
| `pub fn validate_label(label: &str) -> PyResult<()>` | Validiert, dass ein Beziehungs-Label nicht leer ist, keine Null-Bytes enthält und die Maximallänge nicht überschreitet. # Fehlerfälle Liefe… | `src/bindings/common.rs` |
| `pub fn validate_query_text(text: &str) -> PyResult<()>` | Validiert, dass ein Volltext-Suchquery nicht leer ist und keine Null-Bytes enthält. # Fehlerfälle Liefert `ContextraValueError` bei ungülti… | `src/bindings/common.rs` |
| `pub fn validate_vector(vector: &[f32]) -> PyResult<()>` | Validiert, dass ein Vektor nicht leer ist und weder `NaN` noch unendliche Werte (`+/-Inf`) enthält. # Fehlerfälle Liefert `ContextraValueEr… | `src/bindings/common.rs` |
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
| `pub fn _trigger_panic_for_test(&self, py: Python<'_>, message: Option<String>) -> PyResult<()>` | Interner Testhook zur Verifikation der FFI-Panik-Isolation. # Panics Test-Panic wird ausgelöst und durch `run_blocking_ffi` kontrolliert in… | `src/bindings/db.rs` |
| `pub fn collection(&self, name: &str, py: Python<'_>) -> PyResult<PyCollection>` | Öffnet oder erstellt eine spezifische Collection (Namespace) innerhalb der Datenbank. # Parameter - `name`: Name der Collection (maximal 64… | `src/bindings/db.rs` |
| `pub fn drop_collection(&self, name: &str, py: Python<'_>) -> PyResult<()>` | Löscht eine Collection und entfernt alle darin enthaltenen Daten unwiderruflich aus dem Storage. # Parameter - `name`: Name der zu entferne… | `src/bindings/db.rs` |
| `pub fn flush(&self, py: Python<'_>) -> PyResult<()>` | Schreibt alle im Speicher gepufferten Daten (MemTable/WAL) persistent auf die Festplatte (`fsync`). | `src/bindings/db.rs` |
| `pub fn is_empty(&self, py: Python<'_>) -> PyResult<bool>` | Prüft, ob die Standard-Collection leer ist. | `src/bindings/db.rs` |
| `pub fn is_poisoned(&self) -> bool` | Gibt `True` zurück, wenn die Datenbankinstanz nach einem vorangegangenen Rust-Panic vergiftet ist. | `src/bindings/db.rs` |
| `pub fn len(&self, py: Python<'_>) -> PyResult<usize>` | Gibt die Anzahl der Dokumente in der Standard-Collection zurück. | `src/bindings/db.rs` |
| `pub fn list_collections(&self, py: Python<'_>) -> PyResult<Vec<String>>` | Liefert eine Liste der Namen aller vorhandenen Collections in der Datenbank. | `src/bindings/db.rs` |
| `pub fn new( inner: Arc<Contextra>, runtime: Arc<Runtime>, worker_threads: usize, poisoned: Arc<AtomicBool>, ) -> Self` | Konstruiert ein neues `PyContextra`-Handle. | `src/bindings/db.rs` |
| `pub fn stats(&self, py: Python<'_>) -> PyResult<PyDbStats>` | Ruft kombinierte Statistiken für Vektorindex, LSM-Tree, Kalibrierung und Drift ab. | `src/bindings/db.rs` |
| `pub fn worker_threads(&self) -> usize` | Gibt die konfigurierte Anzahl an Tokio-Worker-Threads für diese Instanz zurück. | `src/bindings/db.rs` |
| `pub fn _trigger_panic_for_test(py: Python<'_>, message: Option<String>) -> PyResult<()>` | Python-Funktion zum Testen der Panik-Isolation (nur bei Feature `test-hooks`). # Zweck Wird von der Python-Testsuite aufgerufen, um zu veri… | `src/bindings/functions.rs` |
| `pub fn open( py: Python<'_>, path: &str, dimension: usize, max_elements: Option<usize>, encryption_passphrase: Option<String>, distance_metric: Option<String>, ) -> PyResult<PyContextra>` | Öffnet oder erstellt eine Contextra-Datenbank am angegebenen Dateipfad. # Parameter - `path`: Dateipfad zum Datenbankverzeichnis (wird vali… | `src/bindings/functions.rs` |
| `pub fn relate_n_ary<'py>( &self, py: Python<'py>, predicate: &str, participants: Vec<(Bound<'py, PyAny>, String)>, source_doc_id: Option<Bound<'py, PyAny>>, ) -> PyResult<u64>` | Legt eine n-äre Hyperkante an. # Parameter - `predicate`: Der Name des Prädikats als String. - `participants`: Liste von Tupeln `[(doc_id,… | `src/bindings/hyperedge.rs` |
| `pub fn validate_hyperedge_args( predicate: &str, participants: &[(&str, &str)], source_doc_id: Option<&str>, ) -> PyResult<()>` | Validiert die Eingabeparameter für das Erstellen einer n-ären Hyperkante. # Parameter - `predicate`: Der Beziehungsname (z. B. "verhandelt"… | `src/bindings/hyperedge.rs` |
| `pub fn get_runtime(py: Python<'_>) -> PyResult<Arc<Runtime>>` | Ruft die Tokio-Runtime für den aktuellen Python-Interpreter ab oder initialisiert sie bei Bedarf. # Zweck Stellt sicher, dass asynchrone Ru… | `src/bindings/runtime_state.rs` |
| `pub fn parse_worker_threads_env() -> usize` | Liest und begrenzt die Umgebungsvariable `CONTEXTRA_WORKER_THREADS` auf den Bereich `[MIN_WORKER_THREADS, MAX_WORKER_THREADS]`. # Zweck Bes… | `src/bindings/runtime_state.rs` |
| `pub fn get_kv<'py>( &self, py: Python<'py>, id: &Bound<'py, PyAny>, ) -> PyResult<Option<PyObject>>` | Liest ein Key-Value-Paar direkt anhand des Schlüssels aus dem Speicher. # Rückgabe - `Some(dict)` falls der Eintrag existiert (als Python-D… | `src/kv_links.rs` |
| `pub fn get_links<'py>( &self, py: Python<'py>, id: &Bound<'py, PyAny>, ) -> PyResult<Vec<PyObject>>` | Ruft alle ausgehenden Memory-Links für ein bestimmtes Dokument ab. # Rückgabe Eine Python-Liste von Dictionaries mit den Schlüsseln `target… | `src/kv_links.rs` |
| `pub fn link_memories<'py>( &self, py: Python<'py>, from: &Bound<'py, PyAny>, to: &Bound<'py, PyAny>, relation: &str, ) -> PyResult<()>` | Verknüpft zwei Memory-Knoten mit einer typisierten Zettelkasten-Relation (A-MEM). # Parameter - `from`: ID des Ausgangsdokuments. - `to`: I… | `src/kv_links.rs` |
| `pub fn parse_link_relation(s: &str) -> PyResult<LinkRelation>` | Parst einen String ohne Allokation und unabhängig von Groß-/Kleinschreibung in eine `LinkRelation`. # Zweck Konvertiert nutzerdefinierte Re… | `src/kv_links.rs` |
| `pub fn put_kv<'py>( &self, py: Python<'py>, id: &Bound<'py, PyAny>, value: &Bound<'py, PyDict>, ) -> PyResult<()>` | Speichert ein reines Key-Value-Paar direkt im Speicher ab, ohne Vektor- oder Volltextindizes zu verändern. # Parameter - `py`: Das Python G… | `src/kv_links.rs` |
| `pub fn put_kv_if_absent<'py>( &self, py: Python<'py>, id: &Bound<'py, PyAny>, value: &Bound<'py, PyDict>, ) -> PyResult<()>` | Speichert ein Key-Value-Paar nur dann, wenn der Schlüssel noch nicht im Storage existiert. # Zweck Verhindert versehentliches Überschreiben… | `src/kv_links.rs` |
| `pub fn relate_bidirectional<'py>( &self, py: Python<'py>, from: &Bound<'py, PyAny>, to: &Bound<'py, PyAny>, label: &str, ) -> PyResult<()>` | Erstellt atomar eine ungerichtete bzw. beidseitige Relation zwischen zwei Dokumenten. # Parameter - `from`: ID des Ursprungsdokuments. - `t… | `src/kv_links.rs` |


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

### Architekturbeschreibung

# Crate `contextra-testkit`  Deterministische Test-Utilities, Mocks, In-Memory-Referenzimplementierungen und Konformitätsprüfsuiten für das Contextra-System.  ## Ring & Architektur-Einordnung - **Ring:** Werkzeug / Tooling (außerhalb der numerischen Ringe 0 bis 4). - **Klasse:** Werkzeug (`—`). - **Sicherheits-Invariante (INV-EMBED-1):** `contextra-testkit` darf ausschließlich als `[dev-dependencies]` in Produktions-Crates eingebunden werden und niemals in Release-Binaries oder produktiven Abhängigkeiten landen. - **Unsafe-Code:** Ausdrücklich verboten über `#![forbid(unsafe_code)]`.  ## Bereitgestellte Hauptkomponenten - [`ManualClock`]: Deterministische, manuell steuerbare Uhr (implementiert `Clock`-Port). - [`FaultVfs`]: Virtuelles Dateisystem mit kontrollierter Fehlerinjektion für I/O- und fsync-Fehler. - [`InMemoryStorageEngine`]: Transaktionale In-Memory-Speicher-Engine mit Rollback-Unterstützung. - [`InMemoryVectorIndex`]: In-Memory-Vektorindex mit Cosine-Distanzberechnung. - [`InMemoryTextIndex`]: In-Memory-Volltextindex (BM25-ähnlich). - [`InMemoryGraphIndex`]: In-Memory-Graphindex für Kanten- und Knotentraversierung. - [`ReferenceModel`]: Referenzmodell zur Validierung von MVCC-Isolationsstufen (Multi-Version Concurrency Control). - [`mocks`]: Mocks für Embedder und Metriken (`MockEmbedder`, `TestMetricsSink`). - [`conformance`]: Standardisierte Konformitätstests für Ports (Clock, RNG, Storage, Vector, etc.).  ## Abhängigkeiten - **Normal:** `contextra-ports`, `contextra-types`, `bytes`, `parking_lot`, `tracing`. - **Wird eingebunden von:** `contextra`, `contextra-agent`, `contextra-checkpoint`, `contextra-cognition`, `contextra-crypto`, `contextra-db`, `contextra-engine`, `contextra-graph`, `contextra-mvcc`, `contextra-privacy`, `contextra-store`.

### Modul- und Re-Export-Oberfläche (lib.rs)

```rust
pub mod conformance;
pub mod fault_vfs;
pub mod in_memory_graph;
pub mod in_memory_store;
pub mod in_memory_text;
pub mod in_memory_vector;
pub mod manual_clock;
pub mod mocks;
pub mod reference_model;
pub use fault_vfs::{FaultConfig, FaultVfs};
pub use in_memory_graph::InMemoryGraphIndex;
pub use in_memory_store::InMemoryStorageEngine;
pub use in_memory_text::InMemoryTextIndex;
pub use in_memory_vector::InMemoryVectorIndex;
pub use manual_clock::ManualClock;
pub use mocks::*;
pub use reference_model::{RefOp, ReferenceModel};
```

### Feature-Flags

| Flag | Aktiviert |
|---|---|
| `default` | `["testkit"]` |
| `testkit` | `[]` |

### Öffentliche Datentypen — Structs (11)

| Struct | Felder / Form | Beschreibung | Datei |
|---|---|---|---|
| `ConformanceFailure` | `{ /// Name der ausgeführten Suite (z. B. "StorageEngine", "VectorIndex", "Clock"). pub suite_name: &'static str, /// Name des spezifischen fehlgeschlagenen Testf… }` | Repräsentiert einen aufgetretenen Fehler während einer Konformitätsprüfung. **Zweck:** Enthält den Suitenamen, den konkreten Testfall und eine mensch… | `src/conformance/failure.rs` |
| `FaultConfig` | `{ /// Schreiboperationen nach dieser Anzahl erfolgreicher Schreibvorgänge fehlschlagen lassen. pub fail_writes_after: Option<usize>, /// Sync- bzw. Flush-Operati… }` | Konfiguration für deterministische Fehlerinjektion in [`FaultVfs`]. **Zweck:** Ermöglicht Test-Cases die exakte Vorgabe von Fehlerbedingungen, beispi… | `src/fault_vfs.rs` |
| `FaultVfs` | `{ /// In-Memory Dateispeicher: Pfad -> Dateibytes. files: Arc<RwLock<BTreeMap<PathBuf, Vec<u8>>>>, /// Anzahl der versuchten Leseoperationen. read_count: Arc<Ato… }` | Simuliertes In-Memory-Dateisystem mit deterministischer Fehlerinjektion. **Zweck:** Dient als I/O-Testunterlage für Speichermodule (z. B. WAL und LSM… | `src/fault_vfs.rs` |
| `InMemoryGraphIndex` | `{ entities: Arc<RwLock<EntityMap>>, edges: Arc<RwLock<EdgeMap>>, staged: Arc<RwLock<StagedGraphTxMap>>, last_tx: Arc<RwLock<TxId>>, }` | Thread-sichere In-Memory-Referenzimplementierung von [`GraphIndex`] für Tests. **Zweck:** Ermöglicht Wissensgraph- und Beziehungstests ohne externe G… | `src/in_memory_graph.rs` |
| `InMemoryStorageEngine` | `{ /// Bestätigter globaler Datenbestand. data: Arc<RwLock<StoreDataMap>>, /// Unbestätigte Staging-Puffer nach Transaktions-ID geordnet. staged: Arc<RwLock<Stage… }` | Thread-sichere In-Memory-Implementierung von `StorageEngine` für Tests. **Zweck:** Ermöglicht Unit- und Integrationstests von Engines und Abfrageschi… | `src/in_memory_store.rs` |
| `InMemoryTextIndex` | `{ /// Bestätigter Dokumentendatenbestand. data: Arc<RwLock<TextMap>>, /// Unbestätigte Transaktionsänderungen. staged: Arc<RwLock<StagedTextTxMap>>, /// ID der l… }` | Thread-sichere In-Memory-Referenzimplementierung von [`TextIndex`] für Tests. **Zweck:** Ermöglicht Volltext- und Stichwortsuchen in Tests ohne exter… | `src/in_memory_text.rs` |
| `InMemoryVectorIndex` | `{ /// Erwartete Dimension der Vektoren. dim: usize, /// Bestätigter Vektordatenbestand. data: Arc<RwLock<VectorMap>>, /// Unbestätigte Staging-Änderungen nach Tr… }` | Thread-sichere Referenzimplementierung von [`VectorIndex`] für Tests. **Zweck:** Ermöglicht semantische Vektorsuchen in Unit- und Integrationstests,… | `src/in_memory_vector.rs` |
| `ManualClock` | `{ /// Nanosekunden seit UNIX_EPOCH (1970-01-01 00:00:00 UTC). nanos: AtomicU64, }` | Thread-sichere, manuell vorstellbare Uhr für reproduzierbare Tests. **Zweck:** Ermöglicht deterministische zeitabhängige Tests ohne Abhängigkeit von… | `src/manual_clock.rs` |
| `MockEmbedder` | `{ /// Ziel-Dimensionalität der generierten Embeddings. pub dim: usize, /// Fester Ausgabevektor für reproduzierbare Assertions in Tests. pub fixed_output: Option… }` | Deterministischer Mock-Embedding-Provider für Testläufe. **Zweck:** Ermöglicht semantische Vektortests ohne teure oder nicht-deterministische externe… | `src/mocks.rs` |
| `ReferenceModel` | `{ /// Versionshistorie: Schlüssel -> Liste von (Sequenznummer, Optionaler Wert). #[allow(clippy::type_complexity)] history: BTreeMap<Vec<u8>, Vec<(u64, Option<Ve… }` | Deterministic, std-only MVCC reference model for Contextra store verification. **Zweck:** Bietet eine überprüfbare "Ground Truth" für Transaktionsiso… | `src/reference_model.rs` |
| `TestMetricsSink` | `{ /// Mutex-geschützte Liste aller eingegangenen Ereignisse. events: std::sync::Arc<std::sync::Mutex<Vec<MetricEvent>>>, }` | In-Memory-Implementierung von [`contextra_ports::MetricsSink`] für Tests. **Zweck:** Fängt alle Metrik-Aufrufe ab und speichert sie chronologisch in… | `src/mocks.rs` |

### Öffentliche Datentypen — Enums (2)

| Enum | Varianten | Beschreibung | Datei |
|---|---|---|---|
| `MetricEvent` | `/// Inkrement-Ereignis eines monotonen Zählers (Counter). Counter { /// Name der Metrik. name: String, /// Um wie viel der Zähler erhöht wurde. value: u64, /// Schlüssel-Wert-Labels (Tags) zur Dimens…` | Ein einzelnes aufgezeichnetes Metrik-Ereignis aus [`TestMetricsSink`]. **Zweck:** Dient in Test-Assertions der genauen Überprüfung, ob Instrumentieru… | `src/mocks.rs` |
| `RefOp` | `/// Inserts or overwrites a key-value pair. Put { key: Vec<u8>, value: Vec<u8> }, /// Marks a key as deleted (tombstone). Delete { key: Vec<u8> },` | Operation for batch execution on the reference model. | `src/reference_model.rs` |

### Typ-Aliase (1)

| Alias | Definition | Beschreibung |
|---|---|---|
| `ConformanceResult` | `type ConformanceResult= Result<(), ConformanceFailure>` | Spezialisierter Result-Typ für Konformitätsläufe: `Ok(())` oder `Err(ConformanceFailure)`. |

### Öffentliche Funktionen (47)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn run_clock_suite<C: Clock>(clock: &C) -> ConformanceResult` | Führt die vollständige Konformitätsprüfung für eine [`Clock`]-Implementierung durch. **Ablauf:** 1. Ruft zweimal hintereinander `monotonic_… | `src/conformance/clock.rs` |
| `pub fn new( suite_name: &'static str, test_case: &'static str, details: impl Into<String>, ) -> Self` | Erstellt eine neue [`ConformanceFailure`]-Instanz. | `src/conformance/failure.rs` |
| `pub async fn run_graph_index_suite<F, Fut, G>(factory: F) -> ConformanceResult where F: Fn() -> Fut, Fut: Future<Output = G>, G: GraphIndex,` | Führt die vollständige Konformitätsprüfung für eine [`GraphIndex`]-Implementierung durch. **Parameter:** - `factory`: Asynchrone Closure, d… | `src/conformance/graph.rs` |
| `pub fn run_metrics_sink_suite<M: MetricsSink>(sink: &M) -> ConformanceResult` | Führt die vollständige Konformitätsprüfung für eine [`MetricsSink`]-Implementierung durch. **Ablauf:** 1. Ruft in einem geschützten Block (… | `src/conformance/metrics.rs` |
| `pub fn run_rng_suite<R: Rng>(rng: &R) -> ConformanceResult` | Führt die vollständige Konformitätsprüfung für einen [`Rng`]-Port durch. **Ablauf:** 1. Zieht 3x `next_u64()` und verifiziert Varianz. 2. F… | `src/conformance/rng.rs` |
| `pub async fn run_storage_engine_suite<F, Fut, S>(factory: F) -> ConformanceResult where F: Fn() -> Fut, Fut: Future<Output = S>, S: StorageEngine,` | Führt die vollständige Konformitätsprüfung für eine [`StorageEngine`]-Implementierung durch. **Parameter:** - `factory`: Eine Closure, die… | `src/conformance/storage.rs` |
| `pub async fn run_text_index_suite<F, Fut, T>(factory: F) -> ConformanceResult where F: Fn() -> Fut, Fut: Future<Output = T>, T: TextIndex,` | Führt die vollständige Konformitätsprüfung für eine [`TextIndex`]-Implementierung durch. **Parameter:** - `factory`: Asynchrone Closure, di… | `src/conformance/text.rs` |
| `pub async fn run_vector_index_suite<F, Fut, V>(factory: F) -> ConformanceResult where F: Fn() -> Fut, Fut: Future<Output = V>, V: VectorIndex,` | Führt die vollständige Konformitätsprüfung für eine [`VectorIndex`]-Implementierung durch. **Parameter:** - `factory`: Asynchrone Closure,… | `src/conformance/vector.rs` |
| `pub fn new() -> Self` | Erstellt eine neue, leere [`FaultVfs`]-Instanz ohne aktive Fehlerregeln. | `src/fault_vfs.rs` |
| `pub fn read_count(&self) -> usize` | Liefert die Anzahl der bisher versuchten Leseoperationen. | `src/fault_vfs.rs` |
| `pub fn read_file(&self, path: &Path) -> io::Result<Vec<u8>>` | Liest Bytes vom angegebenen Pfad unter Berücksichtigung der Fehlerrichtlinien. **Fehlerbehandlung:** Liefert einen Lesefehler bei injiziert… | `src/fault_vfs.rs` |
| `pub fn reset_counters(&self)` | Setzt die Operationszähler und den Crash-Status zurück. Die bestehenden Dateiinhalte in `files` bleiben dabei unverändert erhalten. | `src/fault_vfs.rs` |
| `pub fn set_config(&self, config: FaultConfig)` | Aktualisiert die aktive Konfiguration für Fehlerinjektion. **Parameter:** - `config`: Die neue [`FaultConfig`]. | `src/fault_vfs.rs` |
| `pub fn sync(&self) -> io::Result<()>` | Simuliert einen `fsync`-Aufruf zur dauerhaften Bestätigung auf dem Speichermedium. **Ablauf:** 1. Crash-Zustand prüfen. 2. `fail_all` prüfe… | `src/fault_vfs.rs` |
| `pub fn sync_count(&self) -> usize` | Liefert die Anzahl der bisher versuchten Sync-Operationen. | `src/fault_vfs.rs` |
| `pub fn trigger_crash(&self)` | Simuliert einen abrupten Systemabsturz (Power-Cut). Nach diesem Aufruf schlagen alle folgenden Schreib-, Lese- und Sync-Operationen mit `Er… | `src/fault_vfs.rs` |
| `pub fn write_count(&self) -> usize` | Liefert die Anzahl der bisher versuchten Schreiboperationen. | `src/fault_vfs.rs` |
| `pub fn write_file(&self, path: &Path, data: &[u8]) -> io::Result<()>` | Schreibt Bytes an den angegebenen Pfad unter Berücksichtigung der Fehlerrichtlinien. **Fehlerbehandlung:** Schlägt fehl, falls ein Crash ak… | `src/fault_vfs.rs` |
| `pub fn new() -> Self` | Erstellt einen neuen, leeren `InMemoryGraphIndex`. | `src/in_memory_graph.rs` |
| `pub fn new() -> Self` | Erstellt eine neue, leere In-Memory-Speicher-Engine. | `src/in_memory_store.rs` |
| `pub fn new() -> Self` | Erstellt einen neuen, leeren `InMemoryTextIndex`. | `src/in_memory_text.rs` |
| `pub fn new(dim: usize) -> Self` | Erstellt einen neuen `InMemoryVectorIndex` mit der erwarteten Vektordimension `dim`. | `src/in_memory_vector.rs` |
| `pub fn mint_witness_for_tests( layer: impl Into<contextra_ports::scanner::WitnessLayer>, scanned: u64, remaining: u64, technique: contextra_ports::scanner::ScanTechnique, ) -> contextra_ports::scanner::LayerWitness` | Hilfsfunktion zum Erzeugen eines synthetischen [`contextra_ports::scanner::LayerWitness`]. **Zweck:** Ermöglicht Testsuiten das Instanziier… | `src/lib.rs` |
| `pub fn advance(&self, duration: Duration)` | Stellt die Uhr um eine bestimmte Dauer vor. **Ablauf:** 1. Konvertiert die [`Duration`] in Nanosekunden (`u128` zu `u64`). 2. Bei Überlauf… | `src/manual_clock.rs` |
| `pub fn new(initial_nanos: u64) -> Self` | Erstellt eine neue `ManualClock` mit einem gegebenen Nanosekundenwert. **Parameter:** - `initial_nanos`: Startzeit in Nanosekunden seit UNI… | `src/manual_clock.rs` |
| `pub fn now_nanos(&self) -> u64` | Liest den aktuellen Zeitstempel in Nanosekunden seit UNIX_EPOCH aus. | `src/manual_clock.rs` |
| `pub fn now_secs_f64(&self) -> f64` | Gibt den aktuellen Zeitstempel als Fließkommazahl in Sekunden zurück. | `src/manual_clock.rs` |
| `pub fn now_system_time(&self) -> SystemTime` | Konvertiert den aktuellen Zustand in einen Standard-Rust-[`SystemTime`]-Zeitpunkt. | `src/manual_clock.rs` |
| `pub fn set_nanos(&self, nanos: u64)` | Setzt die Uhr absolut auf einen bestimmten Nanosekunden-Zeitstempel. **Hinweis:** Kann die Zeit auch rückwärts setzen; sollte mit Bedacht g… | `src/manual_clock.rs` |
| `pub fn clear(&self)` | Leert die Liste aller aufgezeichneten Metrik-Ereignisse. | `src/mocks.rs` |
| `pub fn events(&self) -> Vec<MetricEvent>` | Erstellt einen Schnappschuss (Snapshot) aller bisher aufgezeichneten Ereignisse. Behandelt vergiftete Mutexe (`PoisonError`) defensiv via `… | `src/mocks.rs` |
| `pub fn new(dim: usize) -> Self` | Erzeugt einen `MockEmbedder` mit der vorgegebenen Dimension. Standardmäßig wird ein Vektor aus `0.1f32` der Länge `dim` erzeugt. | `src/mocks.rs` |
| `pub fn new() -> Self` | Erstellt eine leere [`TestMetricsSink`]. | `src/mocks.rs` |
| `pub fn with_fixed_output(fixed_output: Vec<f32>) -> Self` | Erzeugt einen `MockEmbedder` mit einem festen Vektor. Die Dimension wird automatisch aus der Länge von `fixed_output` abgeleitet. | `src/mocks.rs` |
| `pub fn commit(&mut self) -> u64` | Commits all pending operations under a new sequence number. Returns the sequence number assigned to this commit batch. **Ablauf:** 1. Ist d… | `src/reference_model.rs` |
| `pub fn commit_batch(&mut self, ops: Vec<RefOp>) -> u64` | Appends `ops` to pending buffer and commits them under a new sequence number. | `src/reference_model.rs` |
| `pub fn delete(&mut self, key: &[u8])` | Stages a `Delete` operation in uncommitted pending buffer. | `src/reference_model.rs` |
| `pub fn get_at(&self, key: &[u8], seq: u64) -> Option<Vec<u8>>` | Retrieves the value of `key` as visible at `seq`. Returns `Some(value)` if key exists and was active (not deleted) at `seq`, `None` otherwi… | `src/reference_model.rs` |
| `pub fn get_latest(&self, key: &[u8]) -> Option<Vec<u8>>` | Gets the latest committed value for `key`. | `src/reference_model.rs` |
| `pub fn new() -> Self` | Creates a new empty reference model initialized at sequence number 0. | `src/reference_model.rs` |
| `pub fn put(&mut self, key: &[u8], value: &[u8])` | Stages a `Put` operation in uncommitted pending buffer. | `src/reference_model.rs` |
| `pub fn rollback(&mut self)` | Clears any uncommitted pending writes without advancing sequence number. | `src/reference_model.rs` |
| `pub fn scan_prefix_at(&self, prefix: &[u8], seq: u64) -> Vec<(Vec<u8>, Vec<u8>)>` | Scans keys starting with `prefix` and returns key-value pairs visible at `seq`. **Ablauf:** Nutzt `BTreeMap::range` ab dem Präfix und filte… | `src/reference_model.rs` |
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
