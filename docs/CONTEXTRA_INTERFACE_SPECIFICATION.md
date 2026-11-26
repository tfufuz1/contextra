# Contextra — Globale Mikrofeingranulare Schnittstellenspezifikation

**Repository:** `contextra` (Workspace-Root) · **Generiert am:** 2026-11-27
**Erzeugt durch:** `scripts/generate_interface_spec.py` (automatisierte, reproduzierbare Vollanalyse aller Workspace-Crates)
**Methodik:** Statische Extraktion aller `pub`-Symbole (Traits, Structs, Enums, Funktionen, Typ-Aliase, Konstanten) aus jedem `src/**/*.rs` je Crate, angereichert mit den Original-Doc-Kommentaren (`///`, `//!`), den `Cargo.toml`-Metadaten (Abhängigkeiten, Feature-Flags) und der `lib.rs`-Re-Export-Oberfläche. Test-Module (`#[cfg(test)] mod tests`) sind ausgeschlossen.

> **Hinweis zur Reproduzierbarkeit:** Dieses Dokument wird bei jedem Lauf von `scripts/generate_interface_spec.py` vollständig neu aus dem aktuellen Quellcode-Stand erzeugt. Es spiegelt exakt den Commit wider, gegen den es generiert wurde — bei Architekturänderungen (neue Crates, geänderte Traits/Ports) einfach das Skript erneut ausführen.

## Umfang der Analyse

| Kennzahl | Wert |
|---|---|
| Analysierte Fach-Crates (`crates/*`) | 17 |
| Extrahierte öffentliche Symbole gesamt | **3325** |
| — davon Public Traits (Schnittstellen im engeren Sinn) | 110 |
| — davon Public Structs | 543 |
| — davon Public Enums | 145 |
| — davon Public Funktionen | 2313 |
| — davon Typ-Aliase | 45 |
| — davon Konstanten | 169 |

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

**Unsafe-Inseln:** `contextra-sys`, `contextra-simd` (einzige Crates ohne `#![forbid(unsafe_code)]`).

**Locking-Disziplin:** `collections (RwLock) → kv_locks (KvKeyLocks) → embedder (RwLock)`, Shard-Indizes strikt aufsteigend akquirieren, kein `.await` unter gehaltenem Sync-Lock.

---

## 2. Crate-Inventar nach Ring (Navigationsübersicht)

**Ring 0 — Foundation:** `contextra-types`, `contextra-ports`, `contextra-mvcc`, `contextra-sys`, `contextra-simd`, `contextra-crypto`, `contextra-vector`, `contextra-text`, `contextra-graph`, `contextra-rank`, `contextra-core`

**Ring 1 — Persistence & Cache:** `contextra-store`, `contextra-checkpoint`

**Ring 2 — External Adapters & Sandboxing:** 

**Ring 3 — Orchestration & Cognition:** `contextra-engine`, `contextra-db`

**Ring 4 — Boundary & Composition Roots:** `contextra`, `contextra-mcp`

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
pub mod erasure;
pub mod ids;
pub mod refs;
pub mod witness;
pub use erasure::*;
pub use ids::{ErasureId, FileId, GateId, Seq, StoreId, Timestamp};
pub use refs::{CollectionRef, DocKey, OperatorRef, SubjectRef, TenantRef, MAX_REF_LEN};
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

### Öffentliche Datentypen — Structs (54)

| Struct | Felder / Form | Beschreibung | Datei |
|---|---|---|---|
| `Armed` | `(tuple/unit)` | — | `src/erasure.rs` |
| `CollectionId` | `(tuple/unit)` | — | `src/types/domain/ids.rs` |
| `ConfigFingerprint` | `{ /// LLM-Modell-ID (z.B. "llama-3.2-3b-instruct") pub model_id: String, /// Quantisierungsgrad als String: "Q4_K_M", "Q8_0", "F16", "BF16". /// EXPLIZIT Teil de… }` | — | `src/types/domain/misc.rs` |
| `ContextChunk` | `{ /// Document identifier associated with this chunk. pub doc_id: DocId, /// Text content of the chunk. pub content: String, /// Computed relevance score. pub re… }` | — | `src/types/saos.rs` |
| `ContextWindow` | `{ /// List of included context chunks. pub chunks: Vec<ContextChunk>, /// Total aggregated token count. pub total_tokens: usize, /// Indicates whether context ch… }` | — | `src/types/saos.rs` |
| `ContextraErrorDto` | `{ /// Stable string identifier for the error variant (e.g. `"NotFound"`, `"PolicyViolation"`). pub kind: String, /// Human-readable error message. pub message: S… }` | — | `src/error_dto.rs` |
| `ControlEvidence` | `(tuple/unit)` | — | `src/erasure.rs` |
| `DocId` | `(tuple/unit)` | — | `src/types/domain/ids.rs` |
| `DocId` | `(tuple/unit)` | — | `src/types/domain/ids.rs` |
| `DocRef` | `{ /// Optionaler Betroffener (Subject). pub subject: Option<SubjectRef>, /// Eindeutiger Dokumentenschlüssel innerhalb des Clusters. pub key: DocKey, }` | Eindeutige Dokumentenadresse: Betroffener ist unveränderlicher Teil der Identität (SPEC 7.2, D-70). | `src/ids.rs` |
| `Edge` | `{ /// Source entity identifier. pub from: EntityId, /// Destination entity identifier. pub to: EntityId, /// Relationship label. pub label: Arc<str>, /// Numeric… }` | — | `src/types/domain/misc.rs` |
| `Embedding` | `{ /// Raw vector data elements. pub data: Vec<f32>, }` | — | `src/types/domain/document.rs` |
| `Entity` | `{ /// Unique entity ID. pub id: EntityId, /// Canonical human-readable name. pub name: Arc<str>, /// Categorical entity type. pub entity_type: Arc<str>, /// Flex… }` | — | `src/types/domain/misc.rs` |
| `EntityId` | `(tuple/unit)` | — | `src/types/domain/ids.rs` |
| `EraseReceipt` | `{ /// Eindeutige Kennung des Löschvorgangs. pub id: ErasureId, /// Status des Vorgangs. pub status: ErasureStatus, /// Zeitstempel der Sperrung im Gate. pub gate… }` | — | `src/erasure.rs` |
| `EraseRequest` | `{ /// Löschziel. pub target: EraseTarget, /// Timing (sofort oder gebündelt). pub timing: Timing, /// Aufbewahrungs-Attestierung. pub retention: RetentionAttesta… }` | — | `src/erasure.rs` |
| `ErasureId` | `(tuple/unit)` | — | `src/ids.rs` |
| `FileId` | `(tuple/unit)` | 16-Byte Dateikennung (SPEC F.1). | `src/ids.rs` |
| `FusionWeights` | `{ vector: f32, text: f32, graph: f32, /// Reserved for future metadata-signal fusion; currently always 0.0. Do not expose a public constructor parameter for this… }` | — | `src/types/saos.rs` |
| `GateId` | `(tuple/unit)` | — | `src/ids.rs` |
| `HybridQuery` | `{ /// Optional full-text query string. pub text_query: Option<String>, /// Optional vector query embedding. pub vector_query: Option<Vec<f32>>, /// Optional star… }` | — | `src/types/saos.rs` |
| `HybridQueryBuilder` | `{ text_query: Option<String>, vector_query: Option<Vec<f32>>, graph_start_node: Option<String>, graph_strategy: Option<GraphTraversalStrategy>, fusion_weights: O… }` | — | `src/types/saos.rs` |
| `ImportanceScore` | `(tuple/unit)` | — | `src/types/importance.rs` |
| `LayerId` | `(tuple/unit)` | — | `src/erasure.rs` |
| `LayerWitness` | `{ layer: LayerId, class: MeasureClass, method_id: u16, files: u64, bytes: u64, duration: Duration, hash: [u8; 32], }` | — | `src/erasure.rs` |
| `MemoryImportance` | `{ /// Base LLM or heuristic evaluation score. pub base_score: ImportanceScore, /// Recency decay function strategy. pub decay: DecayFunction, /// Transaction ID… }` | — | `src/types/importance.rs` |
| `MemoryLink` | `{ /// The target document ID. pub target: DocId, /// The type of relationship. pub relation: LinkRelation, /// The transaction ID when this link was created. pub… }` | — | `src/types/domain/misc.rs` |
| `ModelFingerprint` | `{ /// SHA-256 hash digest over model weight blob and quantization tier string. pub hash: [u8; 32], /// Model identifier or filename (e.g. "llama-3.2-3b-instruct"… }` | Identifiziert eine Modelldatei und deren Quantisierungsstufe eindeutig. | `src/model_fingerprint.rs` |
| `PprConfig` | `{ /// Damping factor (probability of continuing random walk vs restarting). Default: 0.85. pub damping_factor: f32, /// Maximum power-iteration steps before term… }` | — | `src/types/domain/misc.rs` |
| `QueryRewriteOutput` | `{ /// Textual reformulation intended for lexical keyword search. pub text_query: Option<String>, /// Semantic reformulation intended for dense vector embedding s… }` | — | `src/types/saos.rs` |
| `RequestContext` | `{ /// Tenant identifier for authorization and scope isolation. pub tenant_id: crate::TenantId, /// Transaction ID allocated for the operation. pub tx_id: crate::… }` | — | `src/witness.rs` |
| `RerankResult` | `{ /// Rank position after reranking (0-based). pub index: usize, /// Original index in the input candidate array. pub original_index: usize, /// Raw model output… }` | — | `src/types/domain/document.rs` |
| `Reservation` | `{ budget: &'a TokenBudget, amount: usize, settled: AtomicBool, }` | — | `src/types/budget.rs` |
| `Residue` | `{ /// Anzahl gefundener Treffer / Rückstände. pub matches: u64, }` | — | `src/erasure.rs` |
| `ResourceBudget` | `{ /// Maximum allowed memory usage in bytes. pub memory_limit: u64, }` | — | `src/types/budget.rs` |
| `ResourceTracker` | `{ /// The configured budget. budget: ResourceBudget, /// Current memory usage in bytes. memory_used: std::sync::atomic::AtomicU64, }` | — | `src/types/budget.rs` |
| `RetentionAttestation` | `{ /// Aufbewahrungscode. pub code: RetentionCode, /// Opake Betreiberkennung (nicht im Klartext im Bericht). pub attested_by: OperatorRef, /// Zeitstempel der At… }` | — | `src/erasure.rs` |
| `SagaRecord` | `{ /// BLAKE3 hash of deleted keys. pub deleted_keys_hash: [u8; 32], }` | — | `src/witness.rs` |
| `Scanner` | `{ layer: LayerId, class: MeasureClass, method_id: u16, files: u64, bytes: u64, _state: S, }` | — | `src/erasure.rs` |
| `ScoredDocument` | `{ /// Identifier of the scored document. pub doc_id: DocId, /// Relevance or similarity score. pub score: f32, }` | — | `src/types/domain/document.rs` |
| `ScoredEntry` | `{ /// Unique result identifier. pub id: String, /// Combined final score after signal fusion. pub final_score: f32, /// Associated entry metadata. pub metadata:… }` | — | `src/types/saos.rs` |
| `Seq` | `(tuple/unit)` | Streng monoton steigende Sequenznummer (SPEC 8.7, I-12, TY-08). | `src/ids.rs` |
| `SeqBitTombstone` | `(tuple/unit)` | — | `src/tombstone.rs` |
| `SignalFusionStrategies` | `{ /// Strategy used for vector search signal fusion. pub vector: FusionStrategy, /// Strategy used for full-text search signal fusion. pub text: FusionStrategy,… }` | — | `src/types/saos.rs` |
| `StepId` | `(tuple/unit)` | — | `src/error.rs` |
| `StoreId` | `(tuple/unit)` | 16-Byte Speicherinstanz-Kennung (SPEC F.1). | `src/ids.rs` |
| `TenantId` | `(tuple/unit)` | — | `src/types/domain/ids.rs` |
| `TenantScoped` | `{ tenant_id: TenantId, value: T, }` | — | `src/tenant_scope.rs` |
| `Timestamp` | `(tuple/unit)` | Mikrosekunden-Zeitstempel in UTC (SPEC 7.4). | `src/ids.rs` |
| `TlHfdParams` | `{ /// Regularization parameter $\sigma > 0$ controlling diffusion scale (default: 0.1). #[serde(default = "default_tl_hfd_sigma")] pub sigma: f32, /// Seed injec… }` | — | `src/types/domain/misc.rs` |
| `TokenBudget` | `{ /// Maximum total token limit. pub limit: usize, /// Strategy for calculating usable context window. pub strategy: BudgetStrategy, /// Reserved tokens for syst… }` | — | `src/types/budget.rs` |
| `TxId` | `(tuple/unit)` | — | `src/types/domain/ids.rs` |
| `Unarmed` | `(tuple/unit)` | — | `src/erasure.rs` |
| `WorkflowState` | `{ /// Associated transaction. pub tx: TxId, /// Agent memory graph state footprint (BLAKE3 hash digest). #[serde(with = "hex")] pub graph_hash: [u8; 32], }` | — | `src/types/domain/misc.rs` |

### Öffentliche Datentypen — Enums (34)

| Enum | Varianten | Beschreibung | Datei |
|---|---|---|---|
| `AutoExtractionMode` | `/// Automatic entity extraction is enabled. Enabled, /// Automatic entity extraction is disabled. Disabled,` | — | `src/types/domain/misc.rs` |
| `BudgetStrategy` | `/// Konservativ: 80% des Limits als sicherer Puffer (Standard) Conservative, /// Aggressiv: 95% des Limits nutzen Aggressive, /// Exakt: Angabe des verfügbaren Fensters direkt Exact(usize),` | — | `src/types/budget.rs` |
| `CacheDirective` | `/// Segment is explicitly pinned in memory until TTL expiration or explicit release. Pin { /// Optional maximum duration for which the segment remains pinned. `None` indicates indefinite pinning. ttl…` | — | `src/error.rs` |
| `ContextraError` | `// ═══ Core & Logic ═══ /// Internal engine logic error. #[error("Internal error: {0}")] Internal(String), /// Invalid user or API argument input. #[error("Invalid input: {0}")] InvalidInput(String),…` | — | `src/error.rs` |
| `DecayFunction` | `/// No recency decay (factor remains 1.0). #[default] None, /// Exponential decay measured in `TxId` distance (not wall-clock time). Exponential { /// Half-life in transaction count. half_life_tx: u6…` | — | `src/types/importance.rs` |
| `DistanceMetric` | `/// Cosine distance (`1.0 - cos(angle)`). #[default] Cosine, /// Euclidean (L2) distance. Euclidean, /// Dot product distance (negated inner product for f32). DotProduct,` | — | `src/types/domain/document.rs` |
| `DocIdWidth` | `/// 64-bit DocId format (8-byte BLAKE3 truncation). Bit64 = 64, /// 128-bit DocId format (16-byte BLAKE3 truncation, ADR-082). Bit128 = 128,` | Gibt die von einer Schemaversion erwartete oder erzeugte Bitbreite einer `DocId` an. | `src/schema.rs` |
| `EraseMode` | `/// Vollständige physische Bereinigung und Schlüsselvernichtung. Erase, /// Einschränkung der Verarbeitung. Restrict,` | — | `src/erasure.rs` |
| `EraseTarget` | `/// Einzelnes Dokument. Document { /// Mandanten-Referenz. tenant: TenantRef, /// Collection-Referenz. collection: CollectionRef, /// Dokument-Referenz (inklusive Subject). doc: DocRef, }, /// Betrof…` | — | `src/erasure.rs` |
| `ErasureStatus` | `/// Auftrag empfangen und validiert. Requested, /// Im Gate persistiert, Lesepfade gesperrt. Gated, /// Schlüssel in Keyslots dauerhaft vernichtet. KeyDestroyed, /// Physisches Umschreiben und Berein…` | — | `src/erasure.rs` |
| `FilterExpr` | `/// Exact match: field == value Eq { /// Target metadata field name. field: String, /// Expected field value. value: serde_json::Value, }, /// Not equal: field != value Ne { /// Target metadata field…` | — | `src/types/filter.rs` |
| `FusionStrategy` | `/// Reciprocal Rank Fusion (default strategy per ADR-003). /// Robust against score scale mismatch and partial signal degradation. #[default] Rrf, /// Score-normalized fusion (CombSUM / Z-Score). ///…` | — | `src/types/saos.rs` |
| `GateVariant` | `/// Klasse P: Physisch gelöscht und gesperrt. Erased, /// Einschränkung der Verarbeitung aktiv. Restricted,` | — | `src/erasure.rs` |
| `GraphTraversalStrategy` | `/// Multi-hop BFS traversal with hop score decay (default max_hops = 3). Hops { /// Maximum traversal hop depth. max_hops: usize, }, /// Personalized PageRank power iteration starting from seed nodes…` | — | `src/types/saos.rs` |
| `HnswDeletionError` | `/// Target node with doc_id not found in graph. #[error("Node for doc_id {0:?} not found in HNSW index")] NodeNotFound(DocId), /// Graph repair resulted in a disconnected graph component. #[error("Gr…` | — | `src/error.rs` |
| `KeyCoverage` | `/// Vollständige kryptografische Unlesbarkeit aller Records/Payloads. Full, /// Struktur sichtbar, aber Verknüpfung unlesbar gemacht. Unlinked, /// Keine Schlüsselvernichtung auf dieser Schichtebene.…` | — | `src/erasure.rs` |
| `LinkRelation` | `/// Target memory provides more detail or context. Elaborates, /// Target memory contradicts this memory. Contradicts, /// Target memory supersedes this memory, replacing its context. Supersedes, ///…` | — | `src/types/domain/misc.rs` |
| `ManifestSchemaVersion` | `/// Schema Version 1 (v0.x legacy 64-bit DocId layout). #[default] V1 = 1, /// Schema Version 2 (ADR-082 128-bit DocId layout). V2 = 2,` | Schema-Versionskennung für SSTable- und WAL-Manifeste. | `src/schema.rs` |
| `MeasureClass` | `/// MK1: Logische/funktionale Prüfung. Mk1, /// MK2: Speicherebenen-Rohdatenprüfung. Mk2, /// MK3: Physische Medieninspektion. Mk3,` | — | `src/erasure.rs` |
| `MemoryType` | `/// Episodisches Gedächtnis: Erlebnisse, Unterhaltungen, zeitlich verankerte Ereignisse. /// Retrieval: Zeitliche Nähe und Relevanz zur aktuellen Session. /// Decay: Hohe Recency-Decay-Rate (Informat…` | — | `src/types/domain/misc.rs` |
| `OnSignalFailure` | `/// Abort hybrid search and return an error on signal failure (Default per Spec Teil 6.5 P1). #[default] Fail, /// Degrade search gracefully using remaining active signals and report failed signals i…` | — | `src/types/saos.rs` |
| `PendingReason` | `/// I/O-Fehler bei der Schichtbearbeitung. Io, /// SLA-Zeitfenster überschritten. SlaExceeded, /// Eine Schicht ist fehlgeschlagen. LayerFailed(LayerId), /// Nachweis/Zeuge fehlt noch. WitnessMissing…` | — | `src/erasure.rs` |
| `Phase` | `/// Intent-Registrierung. Intent, /// Gate-Setzung. Gate, /// Schlüsselvernichtung. KeyDestroy, /// Physisches Bereinigen. Purge, /// Unabhängige Nachweismessung. Verify, /// Dokument-Ersetzung (D-11…` | — | `src/erasure.rs` |
| `PprAlgorithm` | `/// Auto heuristic dispatch based on seed count (ForwardPush for <= 100 seeds, DensePowerIteration otherwise). #[default] Auto, /// Dense power-iteration algorithm (matrix-vector multiplication over…` | — | `src/types/domain/misc.rs` |
| `RefusalReason` | `/// Gesetzliche Aufbewahrungspflicht steht der Bereinigung entgegen. RetentionObligation, /// Unzulässige Parameterkombination. InvalidParameters,` | — | `src/erasure.rs` |
| `ResidualRisk` | `/// Flash-Remanenz auf FTL-Ebene. FlashRemanence, /// CoW-Dateisysteme oder Dateisystemjournale. FsJournalOrCow, /// Reste verpackter Schlüsselkopien. WrappedKeyRemanence, /// Reste in freigegebenen…` | — | `src/erasure.rs` |
| `RetentionCode` | `/// Es besteht keine Aufbewahrungspflicht. NoObligation, /// Aufbewahrungspflicht bestand und ist abgelaufen. ObligationExpired, /// Aufbewahrungspflicht besteht aktiv. ObligationActive, /// Rechtsan…` | — | `src/erasure.rs` |
| `RetrievalStrategy` | `/// Vektorbasiertes semantisches Signal. Vector, /// Textbasiertes lexikalisches/sparse Signal. Text, /// Graphbasiertes Beziehungs-Signal. Graph, /// Hybrides Verbund-Signal. Hybrid, /// Nutzt LeanR…` | Semantische Retrieval-Strategie für Signal-Fusion (§8/§9). | `src/retrieval_strategy.rs` |
| `RiskCondition` | `/// Bedingung trifft immer zu. Always, /// Trifft zu bei CoW-Dateisystemen oder Journaling. CowOrJournaling, /// Trifft zu bis zum nächsten Kompaktions- und Purge-Durchlauf. UntilPurgeComplete,` | — | `src/erasure.rs` |
| `ScanMethod` | `/// Direct structural scan of storage files, indices, or tables. Structural, /// Layer was not measured during this execution with specified reason. NotMeasured(String), /// Layer measurement is disa…` | — | `src/witness.rs` |
| `ScanMethod` | `/// MK1: Logische Scanmethode (z. B. Iteration über Indexeinträge). Mk1, /// MK2: Physische Scanmethode auf Block-/Dateiebene. Mk2, /// MK3: Rohmedien-Inspektion. Mk3,` | — | `src/erasure.rs` |
| `Signal` | `/// Dense vector embedding search signal. Vector, /// Lexical full-text search signal. Text, /// Graph traversal / PageRank signal. Graph,` | — | `src/types/saos.rs` |
| `TenantScopeViolation` | `/// Raised when unpacking a `TenantScoped<T>` value with an expected `TenantId` that differs /// from the value's actual bound `TenantId`. #[error("tenant scope mismatch: expected {expected:?}, got {…` | — | `src/tenant_scope.rs` |
| `Timing` | `/// Sofortige Ausführung im Vordergrund. Now, /// Gebündelte Ausführung im Batch-Fenster. Batched,` | — | `src/erasure.rs` |

### Typ-Aliase (3)

| Alias | Definition | Beschreibung |
|---|---|---|
| `QueryOptions` | `type QueryOptions= HybridQuery` | Alias for [`HybridQuery`] representing query options for hybrid retrieval operations (Spec Teil 6.5). |
| `QueryOptionsBuilder` | `type QueryOptionsBuilder= HybridQueryBuilder` | Alias for [`HybridQueryBuilder`] representing a query options builder for hybrid retrieval operations (Spec Teil 6.5). |
| `Result` | `type Result<T> = std::result::Result<T, ContextraError>` | — |

### Konstanten (15)

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
| `MAX_REF_LEN` | `usize` | Maximale Bytelänge für Referenz-Strings / Byte-Identifier. |
| `MAX_SEARCH_K` | `usize` | — |
| `MIN` | `Self` | Minimum possible `DocId` value (`0`). Note: `DocId(0)` is conventionally treated as a sentinel/null value by the MCP layer (`contextra-mcp/src/lib.rs… |
| `MIN` | `Self` | Minimum possible `DocId` value (`0`). |
| `SYSTEM` | `Self` | SYSTEM tenant identifier (`0`). |
| `TOMBSTONE_BIT` | `u64` | — |

### Öffentliche Funktionen (140)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn arm(self, _ev: ControlEvidence) -> Scanner<Armed>` | Einzige Brücke nach Armed; erfordert Positivkontrollbeleg (TY-10). | `src/erasure.rs` |
| `pub fn bytes(&self) -> u64` | Liefert die Anzahl geprüfter Bytes. | `src/erasure.rs` |
| `pub fn class(&self) -> MeasureClass` | Liefert die Messklasse (MK1-MK3). | `src/erasure.rs` |
| `pub fn condition(&self) -> RiskCondition` | Statische Bedingung für Audit- und Verifikationswerkzeuge (AR-4). | `src/erasure.rs` |
| `pub fn duration(&self) -> Duration` | Liefert die Scandauer. | `src/erasure.rs` |
| `pub fn files(&self) -> u64` | Liefert die Anzahl geprüfter Dateien. | `src/erasure.rs` |
| `pub fn finish( self, matches: u64, duration: Duration, hash: [u8; 32], ) -> Result<LayerWitness, Residue>` | Schließt die Messung ab. matches != 0 => Err(Residue). LayerWitness wird ausschließlich bei matches == 0 ausgegeben (TY-10, I-19, D-72). | `src/erasure.rs` |
| `pub fn hash(&self) -> &[u8` | Liefert den kryptografischen Hash des Prüflaufs. | `src/erasure.rs` |
| `pub fn id(&self) -> &'static str` | Feste Identifikator-Zeichenkette nach SPEC Anhang D. | `src/erasure.rs` |
| `pub fn layer(&self) -> LayerId` | Liefert die Schicht-ID. | `src/erasure.rs` |
| `pub fn method_id(&self) -> u16` | Liefert die Methoden-ID. | `src/erasure.rs` |
| `pub fn new(layer: LayerId, class: MeasureClass, method_id: u16) -> Self` | Erzeugt einen neuen unbewaffneten Scanner. | `src/erasure.rs` |
| `pub fn record(&mut self, files: u64, bytes: u64)` | Zeichnet geprüfte Dateien und Bytes auf. | `src/erasure.rs` |
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
| `pub fn as_bytes(&self) -> &[u8]` | Liefert die Bytes als Slice. | `src/ids.rs` |
| `pub fn into_inner(self) -> Vec<u8>` | Verbraucht den Wrapper und liefert den Vektor. | `src/ids.rs` |
| `pub fn try_new(bytes: Vec<u8>) -> Result<Self>` | Erzeugt eine geprüfte Instanz. Verstoß führt zu `ContextraError::InvalidInput`. | `src/ids.rs` |
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
| `pub fn combined_text_owned(&self) -> String` | Kombinierten Text für Text-Indizierung und Embedding. Entspricht dem etablierten Contextual-Schema: prefix + "\n\n" + content. Allokiert nu… | `src/types/saos.rs` |
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
pub mod conformance;
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
pub mod env;
pub mod erasure_legacy;
pub mod fs;
pub mod index;
pub mod inference;
pub mod policy;
pub mod contract_tests;
pub mod barrier;
pub mod erasure;
pub mod gate;
pub mod keys;
pub mod ledger;
pub mod observer;
pub mod report_signer;
pub use contextra_types::{ error, ids, model_fingerprint, refs, tenant_scope, types, witness, CollectionRef, ConfigFingerprint, ContextChunk, ContextWindow, ContextraError, DocId, DocKey, EntityId, ErasureId, FileId, GateId, ModelFingerprint, OperatorRef, QueryRewriteOutput, RequestContext, RerankResult, ResourceBudget, ResourceTracker, Result, RetrievalStrategy, SagaRecord, ScanMethod, ScoredEntry, Seq, StoreId, SubjectRef, TenantId, TenantRef, TenantScopeViolation, TenantScoped, Timestamp, TokenBudget, TxId, WorkflowState, MAX_REF_LEN, };
pub use attention::*;
pub use checkpoint::*;
pub use clock::*;
pub use conformance::*;
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
pub use barrier::{ BarrierExclusive, BarrierShared, CommitBarrier, CompactionPermit, PurgeBarrier, PurgePermit, };
pub use erasure::{ EraseTarget as PortEraseTarget, ErasureLayer, ErasureTarget, LayerScope, PurgeCx, PurgeOutcome, StorageBackend, };
pub use gate::{GateDecision, KeyBound, KeyRange, RangeGate, ReadGate};
pub use keys::{Envelope, FileKey, FileKeyring, FileKind, RecordSealer, SealCtx};
pub use ledger::{LedgerEntry, LedgerSink, RangeSet};
pub use observer::{ErasureEvent, ErasureObserver, ObservedPhase, TargetKind};
pub use report_signer::ReportSigner;
```

### Feature-Flags

| Flag | Aktiviert |
|---|---|
| `default` | `[]` |
| `conformance` | `["contract-tests"]` |
| `contract-tests` | `[]` |
| `testkit` | `[]` |

### Schnittstellen — Public Traits (46)

**`AttentionExporter`** — *Port implemented by inference backends (Ring 2) to export attention weights from the latest prefill step to the KV-cache eviction worker (Ring 1).* (`src/attention.rs`)

```rust
pub trait AttentionExporter : Send + Sync {
/// Returns accumulated attention weights per token position for the given request ID.
    ///
    /// Returns `None` if no attention weights are tracked or available for the request.
    ///
    /// # Fehler
    /// Keine; liefert `None` falls keine Aufmerksamkeitsgewichte verfügbar sind.
    /// # Crash
    /// Reiner flüchtiger RAM-Statusabruf ohne I/O oder persistente Mutation.
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
    ///
    /// # Fehler
    /// Liefert `ContextraError::NotFound` falls der Checkpoint nicht existiert, oder I/O-Fehler.
    /// # Crash
    /// Atomare Manifest-/VFS-Operation; bei Crash vor Abschluss bleibt der Checkpoint erhalten.
    fn drop_named_checkpoint(&self, name: &str) -> impl Future<Output = Result<()>> + Send;

    /// Lists all active checkpoints.
    ///
    /// # Fehler
    /// Liefert I/O- oder Deserialisierungsfehler beim Lesen der Checkpoint-Metadaten.
    /// # Crash
    /// Reiner Lesevorgang aus Manifest oder Verzeichnis; keine Seiteneffekte.
    fn list_named_checkpoints(&self) -> impl Future<Output = Result<Vec<Self::Meta>>> + Send;
}
```

**`Clock`** — *Port trait for time operations, enabling deterministic testing by decoupling system wall-clock and monotonic time sources.* (`src/clock.rs`)

```rust
pub trait Clock : Send + Sync + 'static {
/// Returns the current wall-clock time as nanoseconds since the Unix epoch.
    ///
    /// Returns `0` if system time is before the epoch instead of panicking.
    ///
    /// # Fehler
    /// Keine; garantiert panikfrei, liefert bei Zeit vor Unix-Epoche `0`.
    /// # Crash
    /// Reiner Statusabruf ohne persistente Mutation.
    fn now_unix_nanos(&self) -> u64;

    /// Returns nanoseconds elapsed since a monotonic baseline reference point.
    ///
    /// # Fehler
    /// Keine; garantiert panikfrei und monoton nicht fallend.
    /// # Crash
    /// Reiner Statusabruf ohne persistente Mutation.
    fn monotonic_nanos(&self) -> u64;
}
```

**`CommitBarrier`** — *Barrier trait managing commit phases and exclusive gate operations (SPEC §11.3, D-57, D-78, L-7).* (`src/barrier.rs`)

```rust
pub trait CommitBarrier : Send + Sync {
/// Acquires a shared permit for standard group commit operations.
    ///
    /// # Fehler
    /// Unfehlbar; wartet asynchron auf Commit-Freigabe.
    ///
    /// # Crash
    /// Reines Synchronisations-Token im RAM; cancel-safe.
    fn shared(&self) -> BoxFuture<'_, BarrierShared>;

    /// Acquires an exclusive permit for gate group transitions.
    ///
    /// # Fehler
    /// Unfehlbar; blockiert nachfolgende Commits exklusiv.
    ///
    /// # Crash
    /// Exklusiver Token-Inhaber; cancel-safe.
    fn exclusive(&self) -> BoxFuture<'_, BarrierExclusive>;
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
    ///
    /// # Fehler
    /// Keine; statischer oder synchron berechneter Statusabruf.
    /// # Crash
    /// Reiner RAM-Abruf ohne persistente Mutation.
    fn overall_drift_status(&self) -> String;
}
```

**`EgressPolicy`** — *Dynamic port trait for evaluating data egress policies.* (`src/policy.rs`)

```rust
pub trait EgressPolicy : Send + Sync {
/// Evaluates if the outgoing content complies with data governance and egress rules.
    ///
    /// # Fehler
    /// Keine; garantiert fehlerfrei über `EgressDecision` abgebildet.
    /// # Crash
    /// Reiner Rechenvorgang im Arbeitsspeicher; keine persistente Mutation.
    fn evaluate(&self, ctx: &contextra_types::RequestContext, payload: &str) -> EgressDecision;
}
```

**`EmbeddingProvider`** — *Abstraction for all embedding providers (e.g. OllamaEmbedder, OnnxEmbedder).* (`src/embedding.rs`)

```rust
pub trait EmbeddingProvider : Send + Sync + 'static {
/// Returns the provider identifier/name (e.g. for logging/debugging).
    ///
    /// # Fehler
    /// Keine; statische Identifikator-Abfrage.
    /// # Crash
    /// Reiner Statusabruf ohne Mutation oder Seiteneffekte.
    fn provider_name(&self) -> &str;

    /// Generates an embedding vector for the given text.
    fn embed<'a>(&'a self, text: &'a str) -> BoxFuture<'a, Result<Vec<f32>, EmbeddingError>>;

    /// Returns the dimensionality of embeddings generated by this provider.
    ///
    /// # Fehler
    /// Keine; statische Dimensionsabfrage.
    /// # Crash
    /// Reiner Statusabruf ohne Mutation oder Seiteneffekte.
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

**`ErasureLayer`** — *Primary trait for storage and index subsystems participating in the erasure saga. Follows SPEC §8.2 verbatim. # Invariants - **PO-05 (Read-only witness)**: `witness` accepts only `&Scanner<Armed>` and performs no mutations. - **PO-05 (Idempotency)**: `purge`…* (`src/erasure.rs`)

```rust
pub trait ErasureLayer : Send + Sync {
/// Returns the unique layer identifier.
    ///
    /// # Fehler
    /// Keine; statische Identifikator-Abfrage.
    ///
    /// # Crash
    /// Reiner Statusabruf ohne persistente Seiteneffekte.
    fn id(&self) -> LayerId;

    /// Resolves the concrete scope affected by the erase target before gating.
    ///
    /// # Fehler
    /// Liefert einen Fehler-String falls das EraseTarget im Layer nicht aufgelöst werden kann.
    ///
    /// # Crash
    /// Reiner Berechnungsschritt vor dem Gating; modifiziert keine Daten auf Festplatte.
    fn resolve(&self, t: &EraseTarget) -> BoxFuture<'_, Result<LayerScope, String>>;

    /// Immediate actions prior to gating (e.g. vector surrogate marking).
    ///
    /// # Fehler
    /// Liefert einen Fehler-String falls Vorbereitungsmaßnahmen vor dem Gate fehlschlagen.
    ///
    /// # Crash
    /// Bereitet RAM-Zustand vor; persistente Markierung erfolgt im Gruppen-Commit.
    fn on_gate(&self, s: &LayerScope) -> BoxFuture<'_, Result<(), String>>;

    /// Executes the physical purge for the given scope within the operational context.
    ///
    /// Must be idempotent and repeatable (PO-05).
    ///
    /// # Fehler
    /// Liefert Fehler bei I/O- oder Kompaktionsfehlern während des physischen Löschens.
    ///
    /// # Crash
    /// Idempotent (PO-05). Jeder Teilschritt ist wiederholbar. fsync-Fehler führen zu Poisoning.
    fn purge(&self, s: &LayerScope, cx: &PurgeCx) -> BoxFuture<'_, Result<PurgeOutcome, String>>;

    /// Verifies residue and produces a witness. Read-only (PO-05).
    ///
    /// # Fehler
    /// Liefert Fehler falls Residuen des gelöschten Umfangs gefunden werden oder I/O fehlschlägt.
    ///
    /// # Crash
    /// Reiner Lesezugriff (PO-05); modifiziert niemals persistenten Zustand.
    fn witness(
        &self,
        s: &LayerScope,
        sc: &Scanner<Armed>,
    ) -> BoxFuture<'_, Result<LayerWitness, String>>;

    /// Reports the key coverage classification (full | unlinked | none).
    ///
    /// # Fehler
    /// Keine; statische Klassifikationsabfrage.
    ///
    /// # Crash
    /// Reiner Statusabruf ohne Mutation.
    fn key_coverage(&self) -> KeyCoverage;
}
```

**`ErasureObserver`** — *Port trait for asynchronous delivery of erasure notifications (SPEC §3.1, §8.2, D-118). Implementations must be object-safe and dyn-compatible as `Arc<dyn ErasureObserver>`.* (`src/observer.rs`)

```rust
pub trait ErasureObserver : Send + Sync + 'static {
/// Notifies the observer about an erasure event.
    ///
    /// # Fehler
    /// Liefert [`ObserverError`], wenn die Benachrichtigung scheitert. Die aufrufende
    /// Hintergrundaufgabe führt Wiederholungen mit exponentiellem Backoff durch (D-118).
    ///
    /// # Crash
    /// Quittungen (`ObserverAck`) werden erst nach erfolgreicher Rückgabe im Journal
    /// persistiert; nicht quittierte Ereignisse werden nach Neustart erneut zugestellt.
    fn on_erased(
        &self,
        ev: ErasureEvent,
    ) -> BoxFuture<'static, std::result::Result<(), ObserverError>>;
}
```

**`ErasureTarget`** — *Generisches Erasure-Target für Bereinigungsanweisungen.* (`src/erasure.rs`)

```rust
pub trait ErasureTarget : Send + Sync + 'static {

}
```

**`FileKeyring`** — *File keyring interface managing per-file cryptographic keys (SPEC §4.1, §8.2, D-85, PO-06). # Contract (PO-06) `destroy` zeroes the key slot, executes `sync_data`, and verifies the key's non-presence via a read-check; failure leaves the slot in `Pending` requ…* (`src/keys.rs`)

```rust
pub trait FileKeyring : Send + Sync {
/// Allocates and records a new file key for the given kind and ID.
    ///
    /// # Fehler
    /// Liefert Fehler falls Slot-Erzeugung oder CSPRNG fehlschlägt.
    ///
    /// # Crash
    /// Persistiert Slot vor Dateierzeugung (I-9b).
    fn create(&self, kind: FileKind, id: FileId) -> BoxFuture<'_, Result<FileKey, String>>;

    /// Opens and retrieves the file key for the given ID.
    ///
    /// # Fehler
    /// Liefert Fehler falls Dateischlüssel fehlt oder vernichtet wurde.
    ///
    /// # Crash
    /// Reines Laden des Dateischlüssels aus dem Tresor.
    fn open(&self, id: FileId) -> BoxFuture<'_, Result<FileKey, String>>;

    /// Securely destroys the file key slot (PO-06).
    ///
    /// # Fehler
    /// Liefert Fehler falls Slot-Nullung oder Sync fehlschlägt.
    ///
    /// # Crash
    /// Nullt Slot, führt sync_data aus und prüft Fehlen; bei fsync-Fehler verbleibt Slot in Pending (PO-06, D-55).
    fn destroy(&self, id: FileId) -> BoxFuture<'_, Result<(), String>>;
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
    ///
    /// # Fehler
    /// Keine; garantiert panikfrei durch atomare Operationen.
    /// # Crash
    /// Reiner flüchtiger RAM-Zustand; bei Crash wird der Generator neu initialisiert.
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
    ///
    /// # Fehler
    /// Keine; liefert `None` falls kein übereinstimmendes Präfix gefunden wurde.
    /// # Crash
    /// Reiner Lesevorgang aus dem Speicher oder Cache ohne persistente Mutation.
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
    ///
    /// # Fehler
    /// Liefert `ContextraError` bei I/O- oder Freigabefehlern.
    /// # Crash
    /// Idempotente Eviction; Teilausfall hinterlässt entweder alten oder eviktierten Zustand.
    fn evict(&self, tenant: TenantId, key: &PrefixKey) -> Result<u64, ContextraError>;
}
```

**`LayerScanner`** — *Trait für Storage-Scanner zur Durchführung von Restmengen-Messungen. # Doctests (Compile Fail Checks) (a) `LayerWitness` lässt sich außerhalb des Moduls nicht direkt konstruieren: ```compile_fail use contextra_ports::scanner::{LayerWitness, WitnessLayer, Scan…* (`src/scanner/mod.rs`)

```rust
pub trait LayerScanner : Send + Sync {
/// Gibt die von diesem Scanner bediente Schicht zurück.
    ///
    /// # Fehler
    /// Keine; statische Schichtabfrage.
    /// # Crash
    /// Reiner Statusabruf ohne Mutation.
    fn layer(&self) -> WitnessLayer;

    /// Führt das Scannen von Einheiten auf der übergebenen `ScanSession` aus.
    ///
    /// # Fehler
    /// Liefert `ContextraError` bei I/O- oder Verifizierungsfehlern.
    /// # Crash
    /// Reiner Lese- und Messzugriff auf Speicherschichten; verändert niemals persistente Daten.
    fn scan_units(&self, session: &mut ScanSession, ctx: &crate::RequestContext) -> Result<()>;

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

**`LedgerSink`** — *Durable audit ledger sink located outside the primary data directory (SPEC §4.2, §11.8).* (`src/ledger.rs`)

```rust
pub trait LedgerSink : Send + Sync {
/// Appends a new verified erasure entry to the ledger.
    ///
    /// # Fehler
    /// Liefert Fehler bei Schreib- oder MAC-Validierungsfehlern.
    ///
    /// # Crash
    /// Schreibt dauerhaft in das Ledger-Log; fsync stellt Dauerhaftigkeit sicher.
    fn append(&self, e: &LedgerEntry) -> Result<(), String>;

    /// Reads all entries recorded in the ledger for verification or auditing.
    ///
    /// # Fehler
    /// Liefert Fehler bei I/O- oder Integritätsbrüchen der Hashkette.
    ///
    /// # Crash
    /// Reiner Lesezugriff zur Audit-Prüfung.
    fn read_all(&self) -> Result<Vec<LedgerEntry>, String>;

    /// Prunes entries older than the specified retention threshold.
    ///
    /// # Fehler
    /// Liefert Fehler bei I/O-Fehlern beim Kürzen.
    ///
    /// # Crash
    /// Kürzt veraltete Ledger-Einträge atomar.
    fn prune(&self, older_than: Timestamp) -> Result<usize, String>;
}
```

**`LicenseGate`** — *Dyn-kompatibler Port (P27) für Lizenzprüfung — Implementierung (Lizenzserver-Anbindung, Ed25519-Signaturprüfung analog zum Signaturprüfpfad §14.1) ist NICHT Teil dieses Tickets und folgt in einem eigenen, separat zu beauftragenden Schritt.* (`src/license.rs`)

```rust
pub trait LicenseGate : Send + Sync {
/// Checks if the requested feature ring is activated and accessible.
    ///
    /// # Fehler
    /// Liefert `LicenseError::NotActivated`, `LicenseError::InvalidSignature` oder `LicenseError::Expired`.
    /// # Crash
    /// Reiner Lesevorgang von Lizenz-Zertifikaten oder RAM-Zustand; keine Seiteneffekte.
    fn check_ring(&self, ring: FeatureRing) -> Result<(), LicenseError>;

    /// Authorizes the requested feature ring and yields an [`AuthorizedRing`] token on success.
    ///
    /// # Fehler
    /// Liefert `LicenseError` falls die Autorisierungsprüfung fehlschlägt.
    /// # Crash
    /// Reiner Statusabruf und Token-Ausstellung ohne persistente Mutation.
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
    ///
    /// # Fehler
    /// Keine; garantiert panikfrei und nicht-blockierend.
    /// # Crash
    /// Reiner flüchtiger RAM-Zähler; keine I/O oder persistente Mutation.
    fn record_counter(&self, name: &str, value: u64, labels: &[(&str, &str)]);

    /// Records a gauge metric absolute value.
    ///
    /// # Fehler
    /// Keine; garantiert panikfrei und nicht-blockierend.
    /// # Crash
    /// Reiner flüchtiger RAM-Messwert; keine I/O oder persistente Mutation.
    fn record_gauge(&self, name: &str, value: f64, labels: &[(&str, &str)]);

    /// Records a histogram observation value.
    ///
    /// # Fehler
    /// Keine; garantiert panikfrei und nicht-blockierend.
    /// # Crash
    /// Reiner flüchtiger RAM-Histogrammwert; keine I/O oder persistente Mutation.
    fn record_histogram(&self, name: &str, value: f64, labels: &[(&str, &str)]);
}
```

**`PluginManifest`** — *Manifest trait implemented by dynamic or static plugins.* (`src/plugin.rs`)

```rust
pub trait PluginManifest : Send + Sync {
/// Returns the name of the plugin.
    ///
    /// # Fehler
    /// Keine; statische Namensabfrage.
    /// # Crash
    /// Reiner Statusabruf ohne Mutation.
    fn name(&self) -> &str;

    /// Returns the list of plugin names required prior to activation.
    ///
    /// # Fehler
    /// Keine; statische Referenzliste.
    /// # Crash
    /// Reiner Statusabruf ohne Mutation.
    fn requires(&self) -> &[&str] {
        &[]
    }

    /// Returns the list of plugin names that conflict with this plugin.
    ///
    /// # Fehler
    /// Keine; statische Referenzliste.
    /// # Crash
    /// Reiner Statusabruf ohne Mutation.
    fn conflicts(&self) -> &[&str] {
        &[]
    }

    /// Returns the required feature ring for this plugin.
    ///
    /// # Fehler
    /// Keine; statischer Feature-Ring-Abruf.
    /// # Crash
    /// Reiner Statusabruf ohne Mutation.
    fn feature_ring_required(&self) -> FeatureRing {
        FeatureRing::Fast
    }

    /// Returns the static capability metadata for this plugin.
    ///
    /// # Fehler
    /// Keine; liefert statischen Deskriptor.
    /// # Crash
    /// Reiner Statusabruf ohne Mutation.
    fn capability(&self) -> PluginCapability;

    /// Activates the plugin instance.
    ///
    /// # Fehler
    /// Liefert `PluginError` bei Initialisierungs-, Berechtigungs- oder Ressourcenfehlern.
    /// # Crash
    /// Idempotente Aktivierung im RAM; Fehler machen Zustandsänderungen rückgängig.
    fn activate(&self) -> Result<(), PluginError>;
}
```

**`PurgeBarrier`** — *Barrier trait managing LSM compaction and physical purge mutual exclusion (SPEC §11.3, §11.4, I-16, L-7).* (`src/barrier.rs`)

```rust
pub trait PurgeBarrier : Send + Sync {
/// Acquires a shared permit for compaction.
    ///
    /// # Fehler
    /// Unfehlbar; wartet auf Purge-Freigabe.
    ///
    /// # Crash
    /// Kooperative Kompaktions-Sperre im RAM.
    fn compaction(&self) -> BoxFuture<'_, CompactionPermit>;

    /// Acquires an exclusive permit for physical purging.
    ///
    /// # Fehler
    /// Unfehlbar; exklusiver Purge-Zugriff.
    ///
    /// # Crash
    /// Erzwingt Abbruch kooperativer Kompaktionen; cancel-safe.
    fn purge(&self) -> BoxFuture<'_, PurgePermit>;

    /// Queries whether an exclusive purge operation is waiting for access.
    ///
    /// Background compactions inspect this periodically to cooperatively yield or abort.
    ///
    /// # Fehler
    /// Keine; nicht-blockierende atomare Abfrage.
    ///
    /// # Crash
    /// Reiner RAM-Zustandsprüfer.
    fn purge_waiting(&self) -> bool;
}
```

**`Purger`** — *Trait für physische Bereinigungsaktionen (Purger) einer Schicht.* (`src/scanner/layer.rs`)

```rust
pub trait Purger : Send + Sync {
/// Führt die physische Bereinigung für das gegebene Ziel aus.
    ///
    /// # Fehler
    /// Liefert `ContextraError` bei I/O- oder Freigabefehlern während der physischen Bereinigung.
    /// # Crash
    /// Idempotent; Teilausfälle hinterlassen konsistente Zwischenzustände ohne Datenverlust lebender Einheiten.
    fn purge(&self, ctx: &crate::RequestContext, target: &ScanTarget) -> Result<PurgeStats>;
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

**`ReadGate`** — *Synchronous, memory-only read gate port interface (SPEC §4.2, §11.3, PO-07). # Invariants - **PO-07**: `ReadGate` is synchronous, performs zero I/O, and executes in $O(\log n)$ time.* (`src/gate.rs`)

```rust
pub trait ReadGate : Send + Sync {
/// Checks point status of an individual key.
    ///
    /// # Fehler
    /// Synchron, keine I/O, panikfrei; liefert GateDecision.
    ///
    /// # Crash
    /// Reiner RAM-Lookup in O(log n); keine Zustandsänderung.
    fn check(&self, k: &KeyBuf) -> GateDecision;

    /// Checks boundary status of a file or partition block for compaction or scans.
    ///
    /// # Fehler
    /// Synchron, keine I/O; liefert GateDecision.
    ///
    /// # Crash
    /// Reiner RAM-Lookup; keine Seiteneffekte.
    fn check_bound(&self, b: &KeyBound) -> GateDecision;

    /// Evaluates range policy for compaction (I-18: Erased may be dropped, Restricted never).
    ///
    /// # Fehler
    /// Synchron, keine I/O; liefert RangeGate.
    ///
    /// # Crash
    /// Reiner RAM-Lookup für Kompaktions-Entscheidungen.
    fn variant_for_range(&self, r: &KeyRange) -> RangeGate;

    /// Retrieves the sequence number at which the key was gated, if any (Restore-Guard, commit rejection).
    ///
    /// # Fehler
    /// Synchron, liefert `Option<Seq>`.
    ///
    /// # Crash
    /// Reiner RAM-Lookup; keine Mutation.
    fn gated_seq(&self, k: &KeyBuf) -> Option<Seq>;

    /// Returns the monotonic gate epoch counter (SPEC §11.3, D-120, I-21).
    ///
    /// Incremented before key destruction to allow linearized read verification.
    ///
    /// # Fehler
    /// Keine; synchroner atomarer Abruf.
    /// # Crash
    /// Reiner RAM-Zugriff ohne Mutation.
    fn gate_epoch(&self) -> u64 {
        0
    }

    /// Evaluates whether a 16-byte `KeyBuf` is covered by an active gate (Erased or Restricted) (SPEC I-2).
    ///
    /// Used by replay filters and commit leader phase.
    ///
    /// # Fehler
    /// Keine; synchroner ram-basierter Lookup.
    /// # Crash
    /// Reiner RAM-Zugriff ohne Mutation.
    fn covers(&self, k: &KeyBuf) -> bool {
        match self.check(k) {
            GateDecision::Erased | GateDecision::Restricted => true,
            GateDecision::Open => false,
        }
    }

    /// Evaluates point gate status of a canonical 78-byte raw key (SPEC 9.1, D-123).
    ///
    /// # Fehler
    /// Keine; synchroner ram-basierter Lookup.
    /// # Crash
    /// Reiner RAM-Zugriff ohne Mutation.
    fn check_raw(&self, _raw: &[u8; 78]) -> GateDecision {
        GateDecision::Open
    }

    /// Evaluates whether a canonical 78-byte raw key is covered by an active gate (SPEC I-2, I-21, D-123).
    ///
    /// # Fehler
    /// Keine; synchroner ram-basierter Lookup.
    /// # Crash
    /// Reiner RAM-Zugriff ohne Mutation.
    fn covers_raw(&self, raw: &[u8; 78]) -> bool {
        match self.check_raw(raw) {
            GateDecision::Erased | GateDecision::Restricted => true,
            GateDecision::Open => false,
        }
    }

    /// Retrieves the sequence number at which a 78-byte raw key was gated, if any.
    ///
    /// # Fehler
    /// Keine; synchroner ram-basierter Lookup.
    /// # Crash
    /// Reiner RAM-Zugriff ohne Mutation.
    fn gated_seq_raw(&self, _raw: &[u8; 78]) -> Option<Seq> {
        None
    }
}
```

**`RecordSealer`** — *Record sealer contract for crypto envelope management (SPEC §4.2, D-53).* (`src/keys.rs`)

```rust
pub trait RecordSealer : Send + Sync {
/// Encrypts and seals plaintext with slot allocation if missing.
    ///
    /// # Fehler
    /// Liefert Fehler falls Verschlüsselung oder Slot-Allokation fehlschlägt.
    ///
    /// # Crash
    /// Vorübergehende Daten im RAM; Persistierung der Slots erfolgt vor dem Commit.
    fn seal<'a>(
        &'a self,
        ctx: &'a SealCtx,
        key: &'a KeyBuf,
        plain: &'a [u8],
    ) -> BoxFuture<'a, Result<Envelope, String>>;

    /// Decrypts envelope. If slot was destroyed/missing, returns error indicating erasure.
    ///
    /// # Fehler
    /// Liefert Fehler falls Schlüssel vernichtet wurde oder Entschlüsselung fehlschlägt.
    ///
    /// # Crash
    /// Reiner kryptografischer Entschlüsselungszugriff ohne Seiteneffekte.
    fn open<'a>(&'a self, key: &'a KeyBuf, env: &'a [u8])
        -> BoxFuture<'a, Result<Vec<u8>, String>>;

    /// Evicts key range from DEK cache, zeroizing RAM keys (D-79).
    ///
    /// # Fehler
    /// Synchrone Operation ohne Fehlerzustände.
    ///
    /// # Crash
    /// Entfernt und nullt Schlüsselmaterial flüchtig im RAM.
    fn evict(&self, scope: &KeyRange);

    /// Forces persistence of vault slots (`sync_data`) before WAL frame flush (I-9, L-9).
    ///
    /// # Fehler
    /// Liefert Fehler bei I/O- oder fsync-Fehlern.
    ///
    /// # Crash
    /// Erzwingt Slot-Dauerhaftigkeit via sync_data; bei fsync-Fehler tritt Poisoning ein (I-9, L-9).
    fn make_slots_durable(&self) -> BoxFuture<'_, Result<(), String>>;
}
```

**`ReportSigner`** — *Abstract port trait for signing audit reports (SPEC §8.2, D-130, T-SIG-1).* (`src/report_signer.rs`)

```rust
pub trait ReportSigner : Send + Sync {
/// Returns the unique numeric signer identifier according to SPEC F.10.
    ///
    /// # Fehler
    /// Keine; statische Signer-ID-Abfrage.
    /// # Crash
    /// Reiner Statusabruf ohne persistente Mutation.
    fn signer_id(&self) -> SignerId;

    /// Signs the canonical report bytes.
    ///
    /// # Parameter
    /// - `canonical_report_bytes`: canonical JSON representation according to SPEC F.10.
    ///
    /// # Fehler
    /// Liefert [`SignerError`], wenn das kryptografische Signieren scheitert.
    ///
    /// # Crash
    /// Reines Berechnungs- oder HSM-Protokoll ohne ungesicherten Seiteneffekt.
    fn sign_report<'a>(
        &'a self,
        canonical_report_bytes: &'a [u8],
    ) -> BoxFuture<'a, std::result::Result<ReportSignature, SignerError>>;
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
    ///
    /// # Fehler
    /// Keine; garantiert panikfrei und nicht-blockierend.
    /// # Crash
    /// Reiner flüchtiger RAM-Zustand oder No-Op; keine Seiteneffekte.
    fn record_implicit_feedback(&self, _results: &[RerankResult], _implicit_relevant_k: usize) {}

    /// Returns the maximum allowed execution time in milliseconds before timing out, if configured.
    ///
    /// # Fehler
    /// Keine; statische Deadline-Abfrage.
    /// # Crash
    /// Reiner Statusabruf ohne persistente Mutation.
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
    ///
    /// # Fehler
    /// Liefert `ContextraError` bei Evaluierungsfehlern oder ungültigen Eingaben.
    /// # Crash
    /// Reiner Rechenvorgang im Arbeitsspeicher; keine Seiteneffekte oder I/O.
    fn score_grounding(&self, response: &str, sources: &[&str]) -> Result<f32>;
}
```

**`Rng`** — *Port trait for random number generators, enabling deterministic testing by decoupling non-deterministic entropy sources.* (`src/rng.rs`)

```rust
pub trait Rng : Send + Sync + 'static {
/// Generates the next pseudorandom 64-bit unsigned integer.
    ///
    /// # Fehler
    /// Keine; garantiert panikfrei und nicht-blockierend.
    /// # Crash
    /// Reiner flüchtiger RAM-Zustand; bei Crash wird der Generator neu initialisiert.
    fn next_u64(&self) -> u64;

    /// Fills `dest` with pseudorandom bytes generated by this RNG.
    ///
    /// # Fehler
    /// Keine; garantiert panikfrei für alle Puffergrößen.
    /// # Crash
    /// Reiner flüchtiger RAM-Zustand; keine persistente Mutation.
    fn fill_bytes(&self, dest: &mut [u8]);

    /// Returns a pseudorandom `f64` value in the half-open interval `[0.0, 1.0)`
    /// with 53 bits of precision.
    ///
    /// # Fehler
    /// Keine; garantiert panikfrei.
    /// # Crash
    /// Reiner Statusabruf ohne persistente Mutation.
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
    ///
    /// # Fehler
    /// Keine; statische Identifikator-Abfrage.
    /// # Crash
    /// Reiner Statusabruf ohne persistente Mutation.
    fn model_id(&self) -> &str;
}
```

**`StorageBackend`** — *Generisches Storage-Backend für modulare Datenzugriffe im Löschkern.* (`src/erasure.rs`)

```rust
pub trait StorageBackend : Send + Sync + 'static {

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
    ///
    /// # Fehler
    /// Keine; statische Fähigkeitsabfrage.
    /// # Crash
    /// Reiner Statusabruf ohne persistente Mutation.
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
    ///
    /// # Fehler
    /// Liefert `ContextraError` bei I/O-, Tokenisierungs- oder Speicherfehlern.
    /// # Crash
    /// Transaktional gepuffert bis zum Commit; vor Commit kein persistenter Schaden.
    fn insert(&self, tx: TxId, id: DocId, text: &str) -> impl Future<Output = Result<()>> + Send;

    /// Deletes a document from the index.
    ///
    /// # Fehler
    /// Liefert `ContextraError` bei Transaktions- oder I/O-Fehlern.
    /// # Crash
    /// Transaktional isoliert; bei Crash vor Commit bleibt der Vorzustand erhalten.
    fn delete(&self, tx: TxId, id: DocId) -> impl Future<Output = Result<()>> + Send;

    /// Commits a transaction.
    ///
    /// # Fehler
    /// Liefert `ContextraError` bei WAL-, Flush- oder Fsync-Fehlern.
    /// # Crash
    /// WAL-first; Commit ist nach erfolgreichem Fsync dauerhaft wiederherstellbar.
    fn commit(&self, tx: TxId) -> impl Future<Output = Result<()>> + Send;

    /// Rolls back a transaction.
    ///
    /// # Fehler
    /// Liefert `ContextraError` bei Transaktionsverwaltungsproblemen.
    /// # Crash
    /// Verwirft uncommittete Schreibpuffer im RAM; Zustand bleibt konsistent.
    fn rollback(&self, tx: TxId) -> impl Future<Output = Result<()>> + Send;

    /// Rolls back the entire index state to a specific transaction ID.
    ///
    /// # Fehler
    /// Liefert `ContextraError` falls der Zielzustand nicht auffindbar oder korrupt ist.
    /// # Crash
    /// Atomarer Rollback; bei Unterbrechung stellt Recovery den sauberen Zustand her.
    fn rollback_to_tx(&self, tx_id: TxId) -> impl Future<Output = Result<()>> + Send;

    /// Returns the last transaction ID processed by the index.
    ///
    /// # Fehler
    /// Liefert `ContextraError` bei Lesefehlern des Manifests/Indexheaders.
    /// # Crash
    /// Reiner Statusabruf ohne Mutation.
    fn last_tx_id(&self) -> impl Future<Output = Result<TxId>> + Send;

    /// Returns the number of documents in the index.
    ///
    /// # Fehler
    /// Keine; garantiert panikfrei.
    /// # Crash
    /// Reiner Statusabruf aus Header oder RAM-Zustand.
    fn len(&self) -> impl Future<Output = usize> + Send;

    /// Returns true if the index is empty.
    ///
    /// # Fehler
    /// Keine; delegiert panikfrei an `len`.
    /// # Crash
    /// Reiner Statusabruf ohne Seiteneffekte.
    fn is_empty(&self) -> impl Future<Output = bool> + Send {
        async { self.len().await == 0 }
    }

    /// Returns index statistics.
    ///
    /// # Fehler
    /// Liefert `ContextraError` falls Statistiken nicht aggregiert werden können.
    /// # Crash
    /// Reiner Lesevorgang ohne Zustandsänderung.
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
    ///
    /// # Fehler
    /// Liefert `ContextraError` bei Lesefehlern der Node-Arena.
    /// # Crash
    /// Reiner Lesevorgang ohne Modifikation.
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
    ///
    /// # Fehler
    /// Liefert `ContextraError` bei Transaktions- oder Arena-Fehlern.
    /// # Crash
    /// Gemäß VETO-F02 strictly tombstone + slot-zeroization; Teilausfälle vor Commit verwerfen Puffer.
    fn delete(&self, tx: TxId, id: DocId) -> impl Future<Output = Result<()>> + Send;

    /// Commits a transaction.
    ///
    /// # Fehler
    /// Liefert `ContextraError` bei WAL-, Flush- oder Fsync-Fehlern.
    /// # Crash
    /// WAL-first; nach erfolgreichem fsync atomar persistent.
    fn commit(&self, tx: TxId) -> impl Future<Output = Result<()>> + Send;

    /// Rolls back a transaction.
    ///
    /// # Fehler
    /// Liefert `ContextraError` bei Transaktionsverwaltungsproblemen.
    /// # Crash
    /// Verwirft uncommittete Änderungen im RAM; Graph bleibt unverändert konsistent.
    fn rollback(&self, tx: TxId) -> impl Future<Output = Result<()>> + Send;

    /// Rolls back the entire index state to a specific transaction ID.
    ///
    /// # Fehler
    /// Liefert `ContextraError` falls der Ziel-Transaktionszustand nicht auffindbar ist.
    /// # Crash
    /// Atomarer Rollback; Recovery stellt bei Unterbrechung den sauberen Zustand her.
    fn rollback_to_tx(&self, tx_id: TxId) -> impl Future<Output = Result<()>> + Send;

    /// Returns the last transaction ID processed by the index.
    ///
    /// # Fehler
    /// Liefert `ContextraError` bei Lesefehlern der Index-Metadaten.
    /// # Crash
    /// Reiner Statusabruf ohne Mutation.
    fn last_tx_id(&self) -> impl Future<Output = Result<TxId>> + Send;

    /// Returns the number of vectors in the index.
    ///
    /// # Fehler
    /// Keine; garantiert panikfrei.
    /// # Crash
    /// Reiner Statusabruf aus Header oder RAM-Zustand.
    fn len(&self) -> impl Future<Output = usize> + Send;

    /// Returns true if the index is empty.
    ///
    /// # Fehler
    /// Keine; delegiert panikfrei an `len`.
    /// # Crash
    /// Reiner Statusabruf ohne Seiteneffekte.
    fn is_empty(&self) -> impl Future<Output = bool> + Send {
        async { self.len().await == 0 }
    }

    /// Returns index statistics.
    ///
    /// # Fehler
    /// Liefert `ContextraError` falls Statistiken nicht ermittelt werden können.
    /// # Crash
    /// Reiner Lesevorgang ohne Zustandsänderung.
    fn stats(&self) -> impl Future<Output = Result<VectorIndexStats>> + Send;

    /// Returns true if the index requires a background rebuild (e.g. due to tombstone accumulation).
    ///
    /// # Fehler
    /// Keine; garantiert panikfrei.
    /// # Crash
    /// Reiner Statusabruf ohne Mutation.
    fn is_rebuild_required(&self) -> bool {
        false
    }

    /// Triggers an asynchronous background rebuild of the index if supported.
    ///
    /// # Fehler
    /// Keine synchronen Fehler; asynchroner Hintergrundtask.
    /// # Crash
    /// Snapshot-isoliert; parallele Leser sehen bit-identische Ergebnisse während und nach Rebuild.
    fn trigger_rebuild_async(&self) {}
}
```

**`Vfs`** — *Abstract object-safe Virtual File System interface. Implements required file operations for LSM storage engine, SSTables, WAL sidecars, vault slot storage, and vector index persistence (SPEC 3.1, D-113, DF-01..DF-03).* (`src/vfs.rs`)

```rust
pub trait Vfs : Send + Sync + 'static {
/// Opens a file according to the specified options.
    ///
    /// # Contract V-VFS
    /// Default implementation delegates to `std::io::ErrorKind::Unsupported` unless overridden.
    ///
    /// # Fehler
    /// Liefert [`ContextraError::Io`](crate::error::ContextraError::Io) bei Berechtigungs-, Nicht-Gefunden- oder I/O-Fehlern.
    ///
    /// # Crash
    /// Öffnet Dateideskriptor; erzeugt ggf. eine leere Datei bei `create_new`/`create`. Keine implizite Datenpersistenz.
    fn open(&self, _path: &Path, _opts: &VfsOpenOptions) -> Result<Box<dyn VfsFile>> {
        Err(crate::ContextraError::Io(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "positional VfsFile open is not supported by default in this Vfs implementation",
        )))
    }

    /// Reads data from byte offset `offset` into `buf` for file at `path`.
    ///
    /// # Fehler
    /// Liefert [`ContextraError::Io`](crate::error::ContextraError::Io) bei I/O-Fehlern.
    ///
    /// # Crash
    /// Reiner Lesezugriff ohne persistente Mutation.
    fn read_at(&self, path: &Path, offset: u64, buf: &mut [u8]) -> Result<usize> {
        let file = self.open(path, &VfsOpenOptions::read_only())?;
        file.read_at(buf, offset).map_err(crate::ContextraError::Io)
    }

    /// Reads verified bytes directly from media or cache-dropped layer (SPEC D-113).
    ///
    /// Returns the verified byte payload and the verification classification `VerifiedVia`.
    ///
    /// # Fehler
    /// Liefert [`ContextraError::Io`](crate::error::ContextraError::Io) bei Lesefehlern.
    fn read_verified(
        &self,
        path: &Path,
        offset: u64,
        len: usize,
    ) -> Result<(Vec<u8>, VerifiedVia)> {
        let file = self.open(path, &VfsOpenOptions::read_only())?;
        file.read_verified(offset, len)
            .map_err(crate::ContextraError::Io)
    }

    /// Reads the entire contents of a file at `path`.
    ///
    /// # Fehler
    /// Liefert [`ContextraError::Io`](crate::error::ContextraError::Io) oder [`ContextraError::NotFound`](crate::error::ContextraError::NotFound)
    /// falls die Datei fehlt oder nicht gelesen werden kann.
    ///
    /// # Crash
    /// Reiner Lesezugriff; keine Zustandsänderung auf dem Speichermedium.
    fn read(&self, path: &Path) -> Result<Vec<u8>>;

    /// Atomically writes `data` to `path` with crash-consistency guarantees.
    ///
    /// # Crash-Consistency Contract
    /// Writes to a temporary file, calls `fsync` on the file handle, renames the temporary
    /// file over `path`, and syncs the parent directory (`sync_dir`).
    ///
    /// # Fehler
    /// Liefert [`ContextraError::Io`](crate::error::ContextraError::Io) falls Schreiben, Synchronisieren oder Umbenennen fehlschlägt.
    ///
    /// # Crash
    /// Crash-konsistente All-or-Nothing-Garantie: Bricht der Vorgang vor atomarem Rename ab, bleibt die Originaldatei unberührt.
    fn write_atomic(&self, path: &Path, data: &[u8]) -> Result<()>;

    /// Creates a directory and all parent directories along `path` if missing.
    ///
    /// # Fehler
    /// Liefert [`ContextraError::Io`](crate::error::ContextraError::Io) falls Verzeichnisanlage fehlschlägt.
    ///
    /// # Crash
    /// Idempotente Verzeichnisanlage; Durability der Verzeichnisstruktur wird über `sync_dir` sichergestellt.
    fn create_dir_all(&self, path: &Path) -> Result<()>;

    /// Removes a file at `path`.
    ///
    /// # Fehler
    /// Liefert [`ContextraError::Io`](crate::error::ContextraError::Io) oder [`ContextraError::NotFound`](crate::error::ContextraError::NotFound)
    /// falls das Entfernen fehlschlägt oder die Datei fehlt.
    ///
    /// # Crash
    /// Entfernt Verzeichniseintrag; Persistenz erfordert `sync_dir` des übergeordneten Verzeichnisses.
    fn remove_file(&self, path: &Path) -> Result<()>;

    /// Canonical alias for removing a file (SPEC D-113 / Purge-Primitive).
    ///
    /// # Fehler
    /// Liefert [`ContextraError::Io`](crate::error::ContextraError::Io) oder [`ContextraError::NotFound`](crate::error::ContextraError::NotFound).
    /// # Crash
    /// Entfernt Verzeichniseintrag; Persistenz erfordert `sync_dir`.
    fn unlink(&self, path: &Path) -> Result<()> {
        self.remove_file(path)
    }

    /// Atomically renames a file from `from` to `to`.
    ///
    /// # Fehler
    /// Liefert [`ContextraError::Io`](crate::error::ContextraError::Io) falls das Umbenennen fehlschlägt.
    ///
    /// # Crash
    /// Atomarer Dateisystem-Rename; bei Systemabsturz existiert entweder der Quell- oder der Zielpfad.
    fn rename(&self, _from: &Path, _to: &Path) -> Result<()> {
        Err(crate::ContextraError::Io(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "atomic rename is not supported by default in this Vfs implementation",
        )))
    }

    /// Reads all entry paths within directory `path`.
    ///
    /// # Fehler
    /// Liefert [`ContextraError::Io`](crate::error::ContextraError::Io) bei Lesefehlern des Verzeichnisses.
    ///
    /// # Crash
    /// Reiner Lesezugriff; keine Verzeichnisänderung.
    fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>> {
        let entries = self.list_dir(path)?;
        Ok(entries.into_iter().map(|name| path.join(name)).collect())
    }

    /// Returns `true` if a file or directory exists at `path`.
    ///
    /// # Fehler
    /// Liefert [`ContextraError::Io`](crate::error::ContextraError::Io) falls Pfadstatus nicht ermittelt werden kann.
    ///
    /// # Crash
    /// Reiner Statusabruf ohne Mutation.
    fn exists(&self, path: &Path) -> Result<bool>;

    /// Returns the length in bytes of the file at `path`.
    ///
    /// # Fehler
    /// Liefert [`ContextraError::Io`](crate::error::ContextraError::Io) oder [`ContextraError::NotFound`](crate::error::ContextraError::NotFound)
    /// falls Metadaten nicht abgefragt werden können.
    ///
    /// # Crash
    /// Reines Abfragen der Dateilänge ohne Seiteneffekte.
    fn len(&self, path: &Path) -> Result<u64>;

    /// Lists entry names (file and directory names, excluding `.` and `..`) inside directory `path`.
    ///
    /// # Fehler
    /// Liefert [`ContextraError::Io`](crate::error::ContextraError::Io) oder [`ContextraError::NotFound`](crate::error::ContextraError::NotFound)
    /// falls Pfad kein Verzeichnis ist oder nicht gelesen werden kann.
    ///
    /// # Crash
    /// Reiner Lesezugriff auf Verzeichniseinträge.
    fn list_dir(&self, path: &Path) -> Result<Vec<String>>;

    /// Canonical alias for listing entries in directory `path` (SPEC 3.1).
    ///
    /// # Fehler
    /// Liefert [`ContextraError::Io`](crate::error::ContextraError::Io) oder [`ContextraError::NotFound`](crate::error::ContextraError::NotFound).
    /// # Crash
    /// Reiner Lesezugriff auf Verzeichniseinträge.
    fn list(&self, path: &Path) -> Result<Vec<String>> {
        self.list_dir(path)
    }

    /// Flushes directory metadata to persistent storage for durable file additions and renames.
    ///
    /// # Fehler
    /// Liefert [`ContextraError::Io`](crate::error::ContextraError::Io) falls Directory-Sync fehlschlägt.
    ///
    /// # Crash
    /// Schreibt Verzeichnis-Metadaten dauerhaft auf Festplatte; schützt vor Datenverlust bei Stromausfall.
    fn sync_dir(&self, path: &Path) -> Result<()>;
}
```

**`VfsFile`** — *Abstract file handle providing positional I/O without modifying internal file offsets.* (`src/vfs.rs`)

```rust
pub trait VfsFile : Send + Sync {
/// Reads data from byte offset `offset` into `buf` without moving the file pointer.
    ///
    /// # Fehler
    /// Liefert `std::io::Error` bei Lesefehlern oder ungültigem Offset.
    ///
    /// # Crash
    /// Reiner positioneller Lesezugriff; verändert keine Dateimetadaten oder persistente Zustände.
    fn read_at(&self, buf: &mut [u8], offset: u64) -> std::io::Result<usize>;

    /// Reads verified data from storage medium bypassing or dropping the page cache (SPEC D-113).
    ///
    /// In accordance with SPEC D-113:
    /// Returns read bytes and classification `VerifiedVia ∈ {Direct, CacheDropped, Cached}`.
    /// Default implementation reads positionally via `read_at` classified as `VerifiedVia::Cached`.
    ///
    /// # Fehler
    /// Liefert `std::io::Error` bei I/O-Fehlern oder Leseproblemen.
    ///
    /// # Crash
    /// Reiner Lesezugriff zur Verifikation von Slots oder Medienblöcken.
    fn read_verified(&self, offset: u64, len: usize) -> std::io::Result<(Vec<u8>, VerifiedVia)> {
        let mut buf = vec![0u8; len];
        let n = self.read_at(&mut buf, offset)?;
        buf.truncate(n);
        Ok((buf, VerifiedVia::Cached))
    }

    /// Writes data from `buf` to byte offset `offset` without moving the file pointer.
    ///
    /// # Fehler
    /// Liefert `std::io::Error` bei Schreibfehlern oder Speicherplatzmangel.
    ///
    /// # Crash
    /// Schreibt Datenblöcke; Persistenz erfordert nachfolgendes `sync_data` oder `sync_all`.
    fn write_at(&self, buf: &[u8], offset: u64) -> std::io::Result<usize>;

    /// Truncates or extends the underlying file length to `len` bytes.
    ///
    /// # Fehler
    /// Liefert `std::io::Error` bei Kürzungs- oder Allokationsfehlern.
    ///
    /// # Crash
    /// Modifiziert die Dateigröße; wird erst durch `sync_all` crash-resistent festgeschrieben.
    fn set_len(&self, len: u64) -> std::io::Result<()>;

    /// Queries the current physical file length in bytes.
    ///
    /// # Fehler
    /// Liefert `std::io::Error` falls Dateimetadaten nicht abgefragt werden können.
    ///
    /// # Crash
    /// Reines Abfragen von Dateimetadaten ohne Seiteneffekte.
    fn len(&self) -> std::io::Result<u64>;

    /// Returns `true` if the file length is zero.
    ///
    /// # Fehler
    /// Liefert `std::io::Error` falls die Längenabfrage fehlschlägt.
    ///
    /// # Crash
    /// Reine Längenprüfung ohne Seiteneffekte.
    fn is_empty(&self) -> std::io::Result<bool> {
        Ok(self.len()? == 0)
    }

    /// Flushes data blocks to disk (`fdatasync`).
    ///
    /// # Fehler
    /// Liefert `std::io::Error` bei fehlschlagendem Flush. Bei Sync-Fehlern muss das Handle vergiftet werden (DF-03, D-55, T-FSYNC-1).
    ///
    /// # Crash
    /// Garantiert die Haltbarkeit der geschriebenen Datenblöcke bis zum Aufrufzeitpunkt.
    fn sync_data(&self) -> std::io::Result<()>;

    /// Flushes data blocks and metadata to disk (`fsync`).
    ///
    /// # Fehler
    /// Liefert `std::io::Error` bei Synchronisationsfehlern. Scheitert `sync_all`, ist das Handle dauerhaft vergiftet und darf nicht weiterverwendet werden (D-55, I-10).
    ///
    /// # Crash
    /// Bewirkt Haltbarkeit aller Datenblöcke und Inode-Metadaten (z. B. Dateilänge).
    fn sync_all(&self) -> std::io::Result<()>;
}
```

### Öffentliche Datentypen — Structs (55)

| Struct | Felder / Form | Beschreibung | Datei |
|---|---|---|---|
| `Armed` | `(tuple/unit)` | Phantom state for an armed scanner verifying deletion residue. | `src/erasure.rs` |
| `AuthorizedRing` | `{ ring: FeatureRing, }` | Token representing an authorized feature ring granting access to engine features (§15). `AuthorizedRing` cannot be directly instantiated outside of t… | `src/license.rs` |
| `BarrierExclusive` | `{ _token: Option<Arc<()>>, }` | RAII token guard representing an exclusive gate phase permit. | `src/barrier.rs` |
| `BarrierShared` | `{ _token: Option<Arc<()>>, }` | RAII token guard representing a shared commit phase permit. | `src/barrier.rs` |
| `CompactionPermit` | `{ _token: Option<Arc<()>>, }` | RAII token guard representing a compaction permit. | `src/barrier.rs` |
| `ContextSegment` | `{ /// Unique identifier for the chunk. pub chunk_id: u64, /// Text content of the segment. pub text: &'a str, /// Optional model fingerprint reference. pub model… }` | Representation of a context segment passed to context-aware text generation. | `src/embedding.rs` |
| `Envelope` | `{ payload: Vec<u8>, }` | Sealed record envelope containing encrypted ciphertext and tag. | `src/keys.rs` |
| `ErasureEvent` | `{ /// Unique identifier of the erasure operation. pub id: ErasureId, /// Kind of target affected. pub target_kind: TargetKind, /// Current phase reached. pub pha… }` | Event published to an [`ErasureObserver`] upon reaching an erasure lifecycle phase (SPEC D-118). | `src/observer.rs` |
| `FileId` | `(tuple/unit)` | Unique identifier for a durable storage file or SSTable. | `src/keys.rs` |
| `FileKey` | `{ raw: Vec<u8>, file_id: FileId, }` | Wrapper around a cryptographic file key with zeroizing on drop and masked debug display (PO-08). | `src/keys.rs` |
| `GraphIndexStats` | `{ /// Number of active nodes (Entities). pub num_entities: usize, /// Number of active edges. pub num_edges: usize, /// Total bytes allocated by CSR representati… }` | Statistics for the GraphIndex layer. | `src/graph_index.rs` |
| `GroundingAssessment` | `{ /// Raw or calibrated confidence score in [0.0, 1.0] indicating attribution quality. pub score: f32, /// Indicates whether the grounding score meets the requir… }` | Result of a post-hoc grounding / attribution validation check. | `src/lifecycle.rs` |
| `HistogramSnapshot` | `{ /// Sum of all observed values. pub sum: f64, /// Total count of observations. pub count: u64, /// Minimum observed value (None if empty). pub min: Option<f64>… }` | Snapshot of a recorded histogram metric. | `src/metrics.rs` |
| `HyperEdgeId` | `(tuple/unit)` | — | `src/graph.rs` |
| `InMemoryMetricsSink` | `{ counters: RwLock<BTreeMap<MetricKey, Arc<AtomicU64>>>, gauges: RwLock<BTreeMap<MetricKey, f64>>, histograms: RwLock<BTreeMap<MetricKey, HistogramState>>, }` | Production-ready local thread-safe implementation of [`MetricsSink`]. Stores metrics strictly in-process with no external network exports (privacy-pr… | `src/metrics.rs` |
| `KeyBuf` | `(tuple/unit)` | Fixed or bounded key buffer for storage operations (SPEC §8.1). | `src/gate.rs` |
| `KeyRange` | `{ /// Lower boundary inclusive. pub start: KeyBuf, /// Upper boundary exclusive. pub end: KeyBuf, }` | Half-open key range `[start, end)` for scan and compaction boundaries. | `src/gate.rs` |
| `KvBlock` | `{ /// Block sequence identifier. pub block_id: u64, /// Encrypted or raw block tensor bytes. #[serde( serialize_with = "serialize_bytes", deserialize_with = "des… }` | Represents an exported KV block segment. | `src/kv.rs` |
| `KvLayout` | `{ /// Number of transformer layers. pub n_layer: u32, /// Number of KV heads. pub n_kv_head: u32, /// Dimension per head. pub head_dim: u32, /// Data type repres… }` | KV cache layer and dimensions layout configuration. | `src/kv.rs` |
| `KvPrefixHit` | `{ /// Number of tokens matched by the prefix. pub matched_tokens: usize, /// Extracted KV blocks for the matched prefix. pub blocks: Vec<KvBlock>, }` | Represents a prefix match hit in the KV store. | `src/kv.rs` |
| `LayerId` | `(tuple/unit)` | Layer identifier representing a specific subsystem participating in erasure (e.g. store, text, vector). | `src/erasure.rs` |
| `LayerScope` | `{ /// Opaque scope identifier or token. scope_id: [u8; 16], /// Segment or file identifiers affected. affected_units: Vec<u64>, }` | Scope resolved by an erasure layer prior to gating (SPEC §8.2, D-96). Records the concrete file sets, segment IDs, or key intervals that this layer m… | `src/erasure.rs` |
| `LayerWitness` | `{ layer: WitnessLayer, scanned: u64, remaining: u64, technique: ScanTechnique, method: ScanMethod, digest: [u8; 32], measured_at: u64, }` | Verifiziertes Messergebnis eines Storage-Scanners (SPEC §12.7.4). Unter INV-WITNESS-MINT-1 kann ein `LayerWitness` ausschließlich in `ScanSession::fi… | `src/scanner/witness.rs` |
| `LedgerEntry` | `{ /// Commitment hash over the target erasure definition. pub commitment: [u8; 32], /// Key ranges covered by this erasure. pub range_set: RangeSet, /// Sequence… }` | A tamper-evident entry in the erasure ledger (SPEC §4.2, §11.8, F.9). Contains `(commitment, range_set, gated_seq, completed_at)` along with `chain_h… | `src/ledger.rs` |
| `MemVfs` | `{ files: RwLock<BTreeMap<PathBuf, Vec<u8>>>, dirs: RwLock<BTreeMap<PathBuf, ()>>, }` | In-memory VFS reference implementation used strictly for contract self-testing. | `src/contract_tests.rs` |
| `MetricKey` | `{ /// Metric identifier (e.g. `lsm_commit_total`). pub name: String, /// Sorted label key-value pairs. pub labels: Vec<(String, String)>, }` | Key identifying a metric by name and sorted key-value labels. | `src/metrics.rs` |
| `MetricSnapshot` | `{ /// Counter metrics (accumulated values). pub counters: BTreeMap<MetricKey, u64>, /// Gauge metrics (last recorded f64 values). pub gauges: BTreeMap<MetricKey,… }` | In-memory snapshot of all metrics collected by [`InMemoryMetricsSink`]. | `src/metrics.rs` |
| `NoopMetricsSink` | `(tuple/unit)` | A no-op implementation of [`MetricsSink`] that silently discards all metrics events. | `src/metrics.rs` |
| `OpenFastGate` | `(tuple/unit)` | Default open activation gate allowing `Fast` ring features without restriction. | `src/license.rs` |
| `PluginCapability` | `{ /// Unique plugin name. pub name: &'static str, /// Plugin version tuple (major, minor, patch). pub version: (u32, u32, u32), /// Architecture ring in which th… }` | Capability descriptor for a plugin. | `src/plugin.rs` |
| `PluginRegistry` | `{ active: BTreeMap<String, PluginCapability>, active_conflicts: BTreeMap<String, Vec<String>>, gate: Arc<dyn LicenseGate>, }` | Registry managing plugin activations, license gating, and dependency resolution. | `src/plugin.rs` |
| `PrefixKey` | `{ /// Target model fingerprint. pub model: ModelFingerprint, /// SHA256/BLAKE3 hash of the tokenizer vocabulary. pub tokenizer_hash: [u8; 32], /// KV tensor memo… }` | Key identifying a model KV cache prefix structure. | `src/kv.rs` |
| `PurgeCx` | `{ /// Budget limit or deadline in milliseconds. pub deadline_ms: u64, }` | Context passed into [`ErasureLayer::purge`] providing operational bounds and budgets. | `src/erasure.rs` |
| `PurgeOutcome` | `{ /// Number of records or items successfully removed. pub purged_count: u64, /// Number of bytes freed or compacted. pub bytes_reclaimed: u64, }` | Outcome of a layer purge execution (SPEC §8.2, WP-3.1). | `src/erasure.rs` |
| `PurgePermit` | `{ _token: Option<Arc<()>>, }` | RAII token guard representing an exclusive purge permit. | `src/barrier.rs` |
| `PurgeStats` | `{ /// Anzahl physisch bereinigter Einheiten. pub purged_units: u64, /// Anzahl freigegebener Bytes. pub bytes_freed: u64, }` | Statistiken über bereinigte Einheiten. | `src/scanner/layer.rs` |
| `RangeSet` | `{ ranges: Vec<(KeyBuf, KeyBuf)>, }` | Set of key ranges or tokens included in an erasure ledger entry. | `src/ledger.rs` |
| `ReportSignature` | `{ /// Signer identifier (u8: 0 = internal, 1 = operator / HSM). pub signer_id: SignerId, /// Base64url-encoded signature string without padding (RFC 4648 §5, SPE… }` | Digital signature over canonical report bytes (SPEC F.10). | `src/report_signer.rs` |
| `RequestId` | `(tuple/unit)` | Unique identifier for an inference request within a process. | `src/attention.rs` |
| `RoleBinding` | `{ /// The role assigned to the entity in the hyperedge. pub role: RoleId, /// The bound entity. pub entity: EntityId, }` | Binding of an entity to a specific role within an n-ary hyperedge. | `src/graph.rs` |
| `RoleId` | `(tuple/unit)` | — | `src/graph.rs` |
| `RopeConfig` | `{ /// Base frequency. pub base: f32, /// Optional frequency scaling factor. pub scaling: Option<f32>, }` | Rotary position embedding (RoPE) configuration. | `src/kv.rs` |
| `ScanSession` | `{ layer: WitnessLayer, target: ScanTarget, technique: ScanTechnique, scanned: u64, remaining: u64, control_matches: u64, unit_hashes: Vec<[u8; 32]>, }` | Laufende Scan-Session zur kontrollierten Erzeugung eines `LayerWitness`. | `src/scanner/session.rs` |
| `ScanTarget` | `{ /// Bezeichner des Scan-Ziels (z. B. Tenant, Collection, Path). pub target_id: String, /// BLAKE3 Hash des Scan-Ziels. pub target_hash: [u8; 32], }` | Zielbeschreibung des Scans inklusive Hash. | `src/scanner/layer.rs` |
| `Scanner` | `{ _state: PhantomData<State>, }` | Residue scanner type passed to [`ErasureLayer::witness`]. Must be purely read-only (PO-05). | `src/erasure.rs` |
| `SealCtx` | `{ /// Epoch sequence or slot identifier. pub epoch: u64, }` | Sealing context containing metadata and epoch tokens for envelope creation. | `src/keys.rs` |
| `SeededRng` | `{ state: AtomicU64, }` | A fast, deterministic 64-bit pseudorandom number generator using SplitMix64 with atomic state transitions (`AtomicU64`). Lock-free and thread-safe wi… | `src/rng.rs` |
| `Seq` | `(tuple/unit)` | Monotonic sequence number for transactions and gate markers. | `src/gate.rs` |
| `SequentialIdGen` | `{ next: AtomicU64, }` | A thread-safe ID generator that produces monotonically increasing 64-bit IDs. Wraps around safely on integer overflow (`u64::MAX` -> `0`) via atomic… | `src/id_gen.rs` |
| `StorageStats` | `{ /// Number of SSTable segments. pub num_segments: usize, /// Total size of all SSTables in bytes. pub total_size_bytes: u64, /// Total size of memtables in byt… }` | Statistics for the storage engine. | `src/storage.rs` |
| `SystemClock` | `{ start_instant: Instant, }` | Standard production implementation of [`Clock`] backed by [`SystemTime`] and [`Instant`]. | `src/clock.rs` |
| `TextIndexStats` | `{ /// Total number of documents indexed. pub num_documents: usize, /// Total number of tokens across all documents. pub num_tokens: usize, /// Estimated memory u… }` | Statistics for a text index. | `src/text_index.rs` |
| `Timestamp` | `(tuple/unit)` | Discrete point in time measured in epoch milliseconds or nanoseconds. | `src/ledger.rs` |
| `VectorIndexStats` | `{ /// Number of active (non-deleted) vectors. pub num_vectors: usize, /// Estimated memory usage in bytes. pub memory_usage_bytes: usize, /// Number of HNSW laye… }` | Statistics for a vector index. | `src/vector_index.rs` |
| `VfsOpenOptions` | `{ /// Opens the file with read access. pub read: bool, /// Opens the file with write access. pub write: bool, /// Creates a new file if it does not exist. pub cr… }` | Options configuring how a file is opened via [`Vfs::open`]. | `src/vfs.rs` |

### Öffentliche Datentypen — Enums (20)

| Enum | Varianten | Beschreibung | Datei |
|---|---|---|---|
| `EgressDecision` | `/// Request is permitted to leave without modification. Allow, /// Request is permitted but payload must be redacted or sanitized. Redact(String), /// Request is denied. Deny(String),` | Egress evaluation decision. | `src/policy.rs` |
| `EmbeddingError` | `/// Provider is unreachable or unavailable. #[error("Provider unavailable: {0}")] Unavailable(String), /// Computation failed during embedding generation. #[error("Embedding failed: {0}")] Computatio…` | Error types encountered during embedding operations. | `src/embedding.rs` |
| `EraseTarget` | `/// Target a specific key or document identifier prefix. KeyRange { /// Lower key bound inclusive. start: Vec<u8>, /// Upper key bound exclusive. end: Vec<u8>, }, /// Target a discrete set of documen…` | Target of an erasure request specifying what needs to be erased. | `src/erasure.rs` |
| `FeatureRing` | `/// Fast ring (open source core). Fast, /// Sovereign ring (enterprise data sovereignty). Sovereign, /// Compliance ring (strict auditability and regulatory rules). Compliance,` | Feature-Ring gemäß Spec §15. | `src/license.rs` |
| `FileKind` | `/// SSTable data file. SSTable, /// Write-Ahead-Log (WAL) file. Wal, /// Inverted index segment. TextSegment, /// Vector HNSW segment. VectorSegment, /// General auxiliary file. Auxiliary,` | Kind of file managed by [`FileKeyring`] (SPEC §8.2, D-85). | `src/keys.rs` |
| `GateDecision` | `/// Key or boundary is open for reading. Open, /// Key or boundary has been erased. Access must be blocked immediately. Erased, /// Key or boundary is temporarily restricted (e.g. pending gate sequen…` | Gate decision for point queries or file boundaries (SPEC §4.2, §11.3). | `src/gate.rs` |
| `GraphMutationError` | `/// Lock acquisition timed out during concurrent mutation. #[error("Lock acquisition timed out: {0}")] LockAcquisitionTimeout(String), /// Duplicate hyperedge ID provided. #[error("Duplicate hyperedg…` | Error variants for graph mutation operations. | `src/graph.rs` |
| `KeyBound` | `/// Exact key match boundary. Exact(KeyBuf), /// Unbounded lower boundary. UnboundedLower, /// Unbounded upper boundary. UnboundedUpper,` | Boundary key used for compaction or range scanning. | `src/gate.rs` |
| `KeyCoverage` | `/// Layer maintains full key separation and slot-zeroing capability. Full, /// Layer holds keys unlinked from the main vault hierarchy. Unlinked, /// Layer maintains no direct key slots (relies entir…` | Key coverage capability of a layer (SPEC §8.2, D-69). | `src/erasure.rs` |
| `LicenseError` | `/// No valid activation found for the requested feature ring. #[error("no valid activation found for feature ring {0:?}")] NotActivated(FeatureRing), /// Activation signature verification failed. #[e…` | Fehlerklasse für Lizenz-/Aktivierungsprüfung — crate-lokal, an der Grenze in `ContextraError` konvertierbar (P6-konform, siehe Spec §17). | `src/license.rs` |
| `ObservedPhase` | `/// Read-gate is established; all future reads will see target as erased. Gated, /// Physical purge and verification complete. Completed, /// Access restriction policy is activated (D-116). Restricte…` | Observed lifecycle phase of an erasure operation (SPEC D-118). | `src/observer.rs` |
| `ObserverError` | `/// Delivery failed due to transient network or backend error. #[error("transient delivery failure: {0}")] Transient(String), /// Delivery rejected permanently by the receiver. #[error("permanent del…` | Error returned by an [`ErasureObserver`] during notification delivery (SPEC D-118). | `src/observer.rs` |
| `PluginError` | `/// Dependency cycle or unresolvable missing dependency detected among plugins. #[error("dependency cycle detected among plugins: {0:?}")] DependencyCycle(Vec<String>), /// Conflicting plugins reques…` | Errors during plugin registration, activation, or dependency resolution. | `src/plugin.rs` |
| `RangeGate` | `/// Range contains erased data: records may be safely discarded. ContainsErased, /// Range is restricted: records must never be discarded until release. RestrictedNeverDiscard, /// Entire range is co…` | Decision variant for range operations during compaction (SPEC §4.2, I-18). | `src/gate.rs` |
| `ScanTechnique` | `/// Direkter struktureller Scan. Structural, /// Scan über entschlüsselte Ansicht. DecryptedView, /// Index-Introspektion. IndexIntrospection, /// Nadel-Scan (Needle Scan). NeedleScan, /// Schicht wu…` | Angewandte Methode/Technik während der Verifikation. | `src/scanner/layer.rs` |
| `SignerError` | `/// Signing key unavailable or corrupted. #[error("signing key unavailable: {0}")] KeyUnavailable(String), /// Hardware security module or external signature provider error. #[error("external signer…` | Error class for report signing operations (SPEC D-130, 7.5). | `src/report_signer.rs` |
| `TargetKind` | `/// Single document erasure. Document, /// Subject-level erasure (all documents belonging to a subject). Subject, /// Collection-level erasure. Collection, /// Tenant-level erasure. Tenant, /// Docum…` | Target kind indicating which hierarchy level was targeted (SPEC D-118, P13). | `src/observer.rs` |
| `VectorDeleteMode` | `/// Synchronous graph repair during vector deletion. SynchronousRepair, /// Asynchronous background graph repair during vector deletion. BackgroundRepair,` | Vector deletion mode for vector index maintenance (§15). | `src/license.rs` |
| `VerifiedVia` | `/// Read bypassed page cache via Direct I/O (`O_DIRECT` or `F_NOCACHE`). Direct, /// Read performed after page cache eviction hint (`posix_fadvise(POSIX_FADV_DONTNEED)`). CacheDropped, /// Read perfo…` | Method via which medium verification was performed (SPEC D-113). | `src/vfs.rs` |
| `WitnessLayer` | `/// LSM Memtable LsmMemtable, /// SsTable Dateien über alle Level SsTableAllLevels, /// HNSW Vektorgraph-Index HnswIndex, /// Write-Ahead-Log Segmente WalAllSegments, /// Compressed Sparse Row Graph…` | Identifikator für die physische Storage-Schicht. | `src/scanner/layer.rs` |

### Typ-Aliase (4)

| Alias | Definition | Beschreibung |
|---|---|---|
| `BoxFuture` | `type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>` | Type alias for a pinned, heap-allocated `Future` that is `Send` and dyn-compatible. |
| `BoxStream` | `type BoxStream<'a, T> = Pin<Box<dyn futures_util::stream::Stream<Item = T> + Send + 'a>>` | Type alias for a pinned, heap-allocated `Stream` that is `Send` and dyn-compatible. |
| `SignerId` | `type SignerId= u8` | Signer ID per SPEC F.10 (0 = internal/derived K_sign, 1 = operator signer). |
| `Timestamp` | `type Timestamp= u64` | Type representing a millisecond or microsecond timestamp (SPEC D-118). |

### Konstanten (1)

| Konstante | Typ | Beschreibung |
|---|---|---|
| `MAX_SCAN_MERGE_ACCUMULATOR` | `usize` | Harte Obergrenze für die Anzahl distinkter Keys, die eine StorageEngine- Implementierung während eines einzelnen scan()/scan_prefix_bounded()-Aufrufs… |

### Öffentliche Funktionen (82)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn new(token: Option<Arc<()>>) -> Self` | Creates a new `BarrierShared` token. | `src/barrier.rs` |
| `pub fn new(token: Option<Arc<()>>) -> Self` | Creates a new `BarrierExclusive` token. | `src/barrier.rs` |
| `pub fn new(token: Option<Arc<()>>) -> Self` | Creates a new `CompactionPermit` token. | `src/barrier.rs` |
| `pub fn new(token: Option<Arc<()>>) -> Self` | Creates a new `PurgePermit` token. | `src/barrier.rs` |
| `pub fn new() -> Self` | Creates a new [`SystemClock`] using the current [`Instant`] as the monotonic baseline. | `src/clock.rs` |
| `pub fn assert_attention_exporter_contract<T: AttentionExporter>(t: &T, req: RequestId)` | Verifies that [`AttentionExporter`] methods can be invoked. | `src/conformance.rs` |
| `pub fn assert_checkpoint_contract<T: Checkpoint>(t: &T, tx: TxId, state: &WorkflowState)` | Verifies that [`Checkpoint`] methods can be invoked in generic contexts. | `src/conformance.rs` |
| `pub async fn assert_checkpoint_coordinator_contract<T: CheckpointCoordinator>( t: &T, tx: TxId, metadata: serde_json::Value, )` | Verifies that [`CheckpointCoordinator`] methods can be invoked in generic contexts. | `src/conformance.rs` |
| `pub fn assert_egress_policy_contract<T: crate::policy::EgressPolicy>( t: &T, ctx: &RequestContext, payload: &str, )` | Verifies dynamic object-safety and evaluation conformance of [`crate::policy::EgressPolicy`]. | `src/conformance.rs` |
| `pub fn assert_embedding_contract< T: EmbeddingProvider + LlmTextGeneratorStreaming + TextEmbeddingEngine + LlmTextGenerator + TextGenerator, >( t: &T, tenant: TenantId, seg: &[ContextSegment], )` | Verifies that [`EmbeddingProvider`], [`LlmTextGeneratorStreaming`], [`LlmTextGenerator`], and [`TextGenerator`] can be invoked. | `src/conformance.rs` |
| `pub fn assert_erasure_layer_contract<E: crate::erasure::ErasureLayer>( e: &E, target: &crate::erasure::EraseTarget, scope: &crate::erasure::LayerScope, )` | Verifies that [`crate::erasure::ErasureLayer`] methods can be invoked. | `src/conformance.rs` |
| `pub fn assert_graph_index_contract<T: GraphIndex + CommunityResolver>( t: &T, tx: TxId, e1: EntityId, e2: EntityId, )` | Verifies that [`GraphIndex`] and [`CommunityResolver`] methods can be invoked. | `src/conformance.rs` |
| `pub fn assert_keys_contract<S: crate::keys::RecordSealer, K: crate::keys::FileKeyring>( s: &S, k: &K, file_id: &crate::keys::FileId, ctx: &crate::keys::SealCtx, key: &crate::gate::KeyBuf, )` | Verifies that [`crate::keys::RecordSealer`] and [`crate::keys::FileKeyring`] methods can be invoked. | `src/conformance.rs` |
| `pub fn assert_ledger_contract<L: crate::ledger::LedgerSink>(l: &L)` | Verifies that [`crate::ledger::LedgerSink`] methods can be invoked. | `src/conformance.rs` |
| `pub fn assert_license_contract<T: LicenseGate>(t: &T, ring: FeatureRing)` | Verifies that [`LicenseGate`] methods can be invoked. | `src/conformance.rs` |
| `pub fn assert_lifecycle_contract< T: GroundingValidator + ResponseGroundingValidator + ContextPreparer, >( t: &T, chunks: &[ContextChunk], assessment: &GroundingAssessment, budget: &TokenBudget, )` | Verifies that [`GroundingValidator`], [`ResponseGroundingValidator`], and [`ContextPreparer`] can be invoked. | `src/conformance.rs` |
| `pub fn assert_observability_contract<T: DriftStatusProvider>(t: &T)` | Verifies that [`DriftStatusProvider`] methods can be invoked. | `src/conformance.rs` |
| `pub fn assert_purger_contract<T: Purger>(t: &T, ctx: &RequestContext, target: &ScanTarget)` | Verifies that [`Purger`] methods can be invoked. | `src/conformance.rs` |
| `pub fn assert_read_gate_contract<G: crate::gate::ReadGate>( g: &G, b: &crate::gate::KeyBound, r: &crate::gate::KeyRange, k: &crate::gate::KeyBuf, )` | Verifies that [`crate::gate::ReadGate`] methods can be invoked in generic contexts. | `src/conformance.rs` |
| `pub fn assert_reranker_contract<T: Reranker>(t: &T, results: &[RerankResult])` | Verifies that [`Reranker`] methods can be invoked. | `src/conformance.rs` |
| `pub fn assert_storage_engine_contract<T: StorageEngine>( t: &T, key: &[u8], val: &[u8], tx: TxId, )` | Verifies that [`StorageEngine`] methods can be invoked. | `src/conformance.rs` |
| `pub fn assert_text_index_contract<T: TextIndex + SegmentSynthesizer>(t: &T)` | Verifies that [`SegmentSynthesizer`] methods can be invoked. | `src/conformance.rs` |
| `pub async fn assert_vector_index_contract<T: VectorIndex + HybridSearchProvider>( t: &T, doc: DocId, )` | Verifies that [`VectorIndex`] and [`HybridSearchProvider`] methods can be invoked. | `src/conformance.rs` |
| `pub fn assert_vfs_file_contract<F: crate::vfs::VfsFile>(f: &F, buf: &mut [u8])` | Verifies that [`crate::vfs::VfsFile`] methods can be invoked in generic contexts. | `src/conformance.rs` |
| `pub fn assert_visitor_methods()` | Internal visitor conformance verification for KV deserialization. | `src/conformance.rs` |
| `pub fn new() -> Self` | Creates a new [`MemVfs`]. | `src/contract_tests.rs` |
| `pub fn verify_clock_contract<C: Clock>(clock: &C)` | Contract test helper verifying [`Clock`] trait implementation compliance. Checks monotonic non-decreasing behavior and valid wall-clock epo… | `src/contract_tests.rs` |
| `pub fn verify_layer_scanner_contract<L: LayerScanner>(scanner: &L, clock: &dyn Clock)` | Contract test helper verifying [`LayerScanner`] trait implementation compliance. Verifies positive control, scanned unit registration, and… | `src/contract_tests.rs` |
| `pub fn verify_metrics_sink_contract<M: MetricsSink>(sink: &M)` | Contract test helper verifying [`MetricsSink`] trait implementation compliance. | `src/contract_tests.rs` |
| `pub fn verify_purger_contract<P: Purger>(purger: &P)` | Contract test helper verifying [`Purger`] trait implementation compliance. Verifies idempotency and safe fail-closed execution. | `src/contract_tests.rs` |
| `pub fn verify_rng_contract<R: Rng>(rng: &mut R)` | Contract test helper verifying [`Rng`] trait implementation compliance. Checks non-zero outputs, correct buffer filling, and reproducible s… | `src/contract_tests.rs` |
| `pub async fn verify_storage_engine_contract<S: StorageEngine>(storage: &S)` | Contract test helper verifying [`StorageEngine`] trait implementation compliance. Verifies put/get, isolation, prefix scanning, delete, and… | `src/contract_tests.rs` |
| `pub async fn verify_text_index_contract<T: TextIndex>(index: &T)` | Contract test helper verifying [`TextIndex`] trait implementation compliance. Verifies text insertion, search, delete, len, and commit. | `src/contract_tests.rs` |
| `pub async fn verify_vector_index_contract<V: VectorIndex>(index: &V)` | Contract test helper verifying [`VectorIndex`] trait implementation compliance. Verifies insertion, search, delete, len, and transaction li… | `src/contract_tests.rs` |
| `pub fn verify_vfs_contract<V: Vfs>(vfs: &V, base_dir: &Path)` | Contract test helper verifying [`Vfs`] trait implementation compliance. Verifies atomic replacement, read roundtrips, directory creation, f… | `src/contract_tests.rs` |
| `pub fn new(chunk_id: u64, text: &'a str) -> Self` | Creates a new `ContextSegment` with default optional metadata set to `None`. | `src/embedding.rs` |
| `pub fn with_fingerprint(mut self, fingerprint: &'a ModelFingerprint) -> Self` | Builder method to set the model fingerprint. | `src/embedding.rs` |
| `pub fn with_rope_offset(mut self, offset: usize) -> Self` | Builder method to set the RoPE offset. | `src/embedding.rs` |
| `pub fn affected_units(&self) -> &[u64]` | Returns affected units count. | `src/erasure.rs` |
| `pub fn as_bytes(&self) -> &[u8` | Returns a slice of raw bytes. | `src/erasure.rs` |
| `pub fn new(scope_id: [u8; 16], affected_units: Vec<u64>) -> Self` | Creates a new `LayerScope` with specific units. | `src/erasure.rs` |
| `pub fn scope_id(&self) -> &[u8` | Returns the unique scope ID. | `src/erasure.rs` |
| `pub fn as_bytes(&self) -> &[u8` | Returns raw bytes slice. | `src/gate.rs` |
| `pub fn contains(&self, key: &KeyBuf) -> bool` | Returns `true` if `key` falls within `[start, end)`. | `src/gate.rs` |
| `pub fn new(role: RoleId, entity: EntityId) -> Self` | Creates a new `RoleBinding`. | `src/graph.rs` |
| `pub fn new(start: u64) -> Self` | Creates a new [`SequentialIdGen`] starting at `start`. | `src/id_gen.rs` |
| `pub fn as_bytes(&self) -> &[u8]` | Returns a reference to the raw secret bytes. | `src/keys.rs` |
| `pub fn file_id(&self) -> FileId` | Associated file ID. | `src/keys.rs` |
| `pub fn new(raw: Vec<u8>, file_id: FileId) -> Self` | Creates a new `FileKey`. | `src/keys.rs` |
| `pub fn new(payload: Vec<u8>) -> Self` | Creates a new envelope. | `src/keys.rs` |
| `pub fn payload(&self) -> &[u8]` | Returns payload bytes. | `src/keys.rs` |
| `pub fn compute_chained( commitment: [u8; 32], range_set: RangeSet, gated_seq: Seq, completed_at: Timestamp, prev_hash: [u8; 32], ledger_mac: [u8; 32], ) -> Self` | Constructs a ledger entry and computes its chained BLAKE3 hash from the prior entry hash. | `src/ledger.rs` |
| `pub fn new(ranges: Vec<(KeyBuf, KeyBuf)>) -> Self` | Creates a range set from intervals. | `src/ledger.rs` |
| `pub fn ranges(&self) -> &[(KeyBuf, KeyBuf)]` | Returns the slice of ranges. | `src/ledger.rs` |
| `pub fn verify_chain_hash(&self, prev_hash: &[u8; 32]) -> bool` | Verifies the entry's chain hash against the supplied previous block hash. | `src/ledger.rs` |
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
| `pub fn read_only() -> Self` | Creates options configured for read-only access. | `src/vfs.rs` |
| `pub fn write_create() -> Self` | Creates options configured for write-only access, creating the file if missing. | `src/vfs.rs` |


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
pub use contextra_types::{ error::{self, ContextraError, Result}, tombstone::{SeqBitTombstone, TombstoneSemanticsCheck}, types::{self, DocId, TenantId, TxId, TOMBSTONE_BIT}, };
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

### Öffentliche Datentypen — Structs (10)

| Struct | Felder / Form | Beschreibung | Datei |
|---|---|---|---|
| `PruningBlockerInfo` | `{ /// Kleinste Sequenznummer, die noch unbereinigt im Register liegt. pub min_unpruned_seq: u64, /// Aktueller `pruned_through`-Wasserstand. pub pruned_through_s… }` | Diagnoseinformationen über aktive Snapshots oder Sequenznummern, die das Pruning im Validator blockieren. Wird erzeugt, wenn der Speicher für Commit-… | `src/ssi.rs` |
| `ReadSet` | `{ /// Zuordnung von gelesenen Schlüsseln (Bytefolgen) zur zugehörigen Snapshot-Sequenznummer. keys: AHashMap<Vec<u8>, u64>, /// Zuordnung von Scan-Präfixen zu de… }` | Protokolliert gelesene Einzelschlüssel, Präfixe und deren jeweilige Snapshot-Sequenznummer für SSI. ### Speicher- und Risikoanalyse: Der Speicherbeda… | `src/ssi.rs` |
| `SeqLogEntry` | `{ /// Eindeutige Kennung des Dokuments. pub doc_id: DocId, /// Logische Sequenznummer, bei der dieser Dokumenteintrag eingefügt wurde. pub insert_seq: u64, /// L… }` | Ein versionierter Protokolleintrag für die Snapshot-Isolation (`_at`-Familie). Jeder Eintrag repräsentiert die Lebensspanne einer Dokumentversion: vo… | `src/seq_log.rs` |
| `SequenceLog` | `{ /// Liste aller historischen Dokumentversionen. entries: Vec<SeqLogEntry>, /// Zuordnung von gepinnter Sequenznummer zu Liste von Erstellungs-Zeitstempeln (in… }` | Verwaltet das chronologische Sequenz-Protokoll und Sichtbarkeitsfilter für Indexstrukturen. ### Funktionsweise: `SequenceLog` speichert alle Einfüge-… | `src/seq_log.rs` |
| `SequenceLogSsiValidator` | `{ committed_writes: Arc<RwLock<CommittedWrites>>, sequence_log: Option<Arc<RwLock<SequenceLog>>>, snapshot_registry: Option<Arc<SnapshotRegistry>>, pruned_throug… }` | Referenz-Implementierung von [`SsiValidator`] auf Basis von [`SequenceLog`] und In-Memory-Commit-Tracking. Arbeitet vollständig ohne Festplatten-I/O… | `src/ssi.rs` |
| `SnapshotGuard` | `{ registry: Arc<SnapshotRegistry>, seq_no: u64, created_at: Option<u64>, tenant: Option<TenantId>, }` | RAII-Guard (Resource Acquisition Is Initialization) für einen aktiven Lese-Snapshot. Hält einen Snapshot in der [`SnapshotRegistry`] gepinnt, solange… | `src/snapshot.rs` |
| `SnapshotRegistry` | `{ active: Mutex<BTreeMap<u64, Vec<SnapshotMeta>>>, min_active_seqno: AtomicU64, clock: Arc<dyn Clock>, config: SnapshotRegistryConfig, }` | Zentrale Registrierung für aktive Lese-Snapshots im System. | `src/snapshot.rs` |
| `SnapshotRegistryConfig` | `{ /// Maximale Anzahl gleichzeitig aktiver Snapshots (Default: 100.000). pub max_active_snapshots: usize, /// Maximales Alter eines Snapshots (Default: 300 s / 5… }` | Konfigurationsoptionen für die [`SnapshotRegistry`]. | `src/snapshot.rs` |
| `TxBuffer` | `{ shards: Vec<RwLock<TxShard<T>>>, key_shards: Vec<RwLock<KeyShard>>, tx_timeout: Duration, config: TxBufferConfig, global_staged_bytes: AtomicUsize, clock: Arc<… }` | Verwaltet gepufferte Transaktionen, deren Staging-Operationen und ReadSets. ### Synchronisations- und Sperrstrategie: Jeder Shard besitzt ein eigenes… | `src/tx_buffer.rs` |
| `TxBufferConfig` | `{ /// Timeout-Dauer, nach der verwaiste (inaktive) Transaktionen automatisch abgeräumt werden. pub tx_timeout: Duration, /// Maximale empfohlene Anzahl gleichzei… }` | Konfigurationsoptionen für den [`TxBuffer`]. | `src/tx_buffer.rs` |

### Öffentliche Datentypen — Enums (2)

| Enum | Varianten | Beschreibung | Datei |
|---|---|---|---|
| `IndexOp` | `/// Einfügen eines Dokuments mit Nutzlast (z. B. Einbettungs-Vektoren oder Schlüssel-Werte). Insert { /// Kennung des Dokuments. doc_id: DocId, /// Assoziierte Nutzdaten. data: T, }, /// Entfernen ei…` | Eine im Puffer vorgemerkte Operation auf einem Index. | `src/tx_buffer.rs` |
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

### Öffentliche Funktionen (136)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn changes_since(&self, snapshot_seq: u64) -> Vec<SeqLogChange>` | Liefert alle Änderungen (Einfügungen und Entfernungen), die strikt nach `snapshot_seq` stattgefunden haben, aufsteigend sortiert nach Seque… | `src/seq_log.rs` |
| `pub fn clock(&self) -> &Arc<dyn Clock>` | Gibt eine Referenz auf den aktiven [`Clock`]-Port zurück. | `src/seq_log.rs` |
| `pub fn compact(&mut self, min_active_seqno: u64)` | Verdichtet (kompaktiert) das Sequenz-Log und entfernt historische Versionen, deren Entfernung strikt kleiner als `min_active_seqno` ist. ##… | `src/seq_log.rs` |
| `pub fn deletion_seq(&self, doc_id: DocId) -> Option<u64>` | Liefert die Lösch-Sequenznummer des Dokuments, falls es aktuell als gelöscht verzeichnet ist. | `src/seq_log.rs` |
| `pub fn doc_id(&self) -> DocId` | Liefert die Kennung des betroffenen Dokuments. | `src/seq_log.rs` |
| `pub fn expired_pins(&self) -> Vec<u64>` | Liefert alle abgelaufenen Snapshot-Pins bezogen auf die aktuelle Systemuhr. | `src/seq_log.rs` |
| `pub fn expired_pins_at(&self, now_nanos: u64) -> Vec<u64>` | Findet alle gepinnten Sequenznummern, deren Haltezeit `max_pin_duration` überschreitet, bezogen auf den übergebenen Monotonie-Zeitstempel `… | `src/seq_log.rs` |
| `pub fn is_empty(&self) -> bool` | Gibt `true` zurück, wenn das Sequenz-Log keine Einträge enthält. | `src/seq_log.rs` |
| `pub fn is_visible(&self, as_of: u64) -> bool` | Prüft in O(1), ob dieser Protokolleintrag zu einem bestimmten Snapshot-Stand `as_of` sichtbar ist. ### Formale Sichtbarkeitsinvariante (INV… | `src/seq_log.rs` |
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
| `pub fn with_time_fn<F>(time_fn: F) -> Self where F: Fn() -> u64 + Send + Sync + 'static,` | Erstellt ein `SequenceLog` mit einer anpassbaren Zeitstempel-Closure (State-Passing per `Fn() -> u64`). | `src/seq_log.rs` |
| `pub fn clock(&self) -> &Arc<dyn Clock>` | Liefert eine Referenz auf den aktiven Zeitport ([`Clock`]). | `src/snapshot.rs` |
| `pub fn config(&self) -> SnapshotRegistryConfig` | Liefert die aktuelle Konfiguration der Registry. | `src/snapshot.rs` |
| `pub fn longest_active_pin(&self) -> Option<(u64, Duration)>` | Findet den am längsten aktiven Snapshot bezogen auf die aktuelle Systemuhr. | `src/snapshot.rs` |
| `pub fn longest_active_pin_at(&self, now_nanos: u64) -> Option<(u64, Duration)>` | Findet den am längsten aktiven Snapshot bezüglich des Nanosekunden-Zeitstempels `now_nanos`. Liefert `Some((seq_no, duration))` oder `None`… | `src/snapshot.rs` |
| `pub fn min_active_seqno(&self) -> u64` | Liefert die kleinste Sequenznummer aller derzeit aktiven Snapshots (`u64::MAX`, falls keine aktiv). | `src/snapshot.rs` |
| `pub fn min_for_compaction(&self, current_seq: u64) -> u64` | Berechnet die minimale Sequenznummer für die Kompaktierung (GC-Grenze) ohne Sentinel. | `src/snapshot.rs` |
| `pub fn min_read_snapshot(&self) -> Option<u64>` | Liefert die kleinste Snapshot-Sequenznummer als `Option<u64>` (ohne Sentinel). | `src/snapshot.rs` |
| `pub fn new() -> Self` | Erstellt eine neue, leere Snapshot-Registry mit Standard-[`SystemClock`] und Standardkonfiguration. | `src/snapshot.rs` |
| `pub fn oldest_active_snapshot(&self) -> Option<u64>` | Liefert die Untergrenze für Garbage Collection (älteste aktive Snapshot-Sequenz). Entspricht dem Vertrag für Store-Kompaktierung und ist äq… | `src/snapshot.rs` |
| `pub fn pin(&self, seq_no: u64)` | Dauerhaftes Pinnen einer Sequenznummer (z. B. für SAOS Checkpoints oder Backups), ohne einen RAII-Guard zu halten. Muss manuell über [`Snap… | `src/snapshot.rs` |
| `pub fn pin_at(&self, seq_no: u64, at_nanos: u64)` | Pinnt eine Sequenznummer zu einem expliziten Zeitstempel. | `src/snapshot.rs` |
| `pub fn pin_for_tenant(&self, tenant: TenantId, seq_no: u64)` | Pinnt eine Sequenznummer zu einem spezifischen Zeitstempel. Pinnt eine Sequenznummer für einen Mandanten bezogen auf die aktuelle Systemuhr. | `src/snapshot.rs` |
| `pub fn pin_for_tenant_at(&self, tenant: TenantId, seq_no: u64, at_nanos: u64)` | Pinnt eine Sequenznummer für einen Mandanten zu einem expliziten Zeitstempel. | `src/snapshot.rs` |
| `pub fn reap_expired_pins(&self, max_pin_duration: Duration) -> Vec<u64>` | Bereinigt alle überalterten Snapshot-Pins bezogen auf die aktuelle Uhrzeit. | `src/snapshot.rs` |
| `pub fn reap_expired_pins_at(&self, max_pin_duration: Duration, now_nanos: u64) -> Vec<u64>` | Bereinigt und entfernt alle aktiven Snapshot-Pins, die älter als `max_pin_duration` sind. ### Ablauf: 1. Berechne Schwellenwert in Nanoseku… | `src/snapshot.rs` |
| `pub fn register(self: &Arc<Self>, seq_no: u64) -> SnapshotGuard` | Registriert einen Lese-Snapshot und liefert einen RAII-Guard ([`SnapshotGuard`]) zurück. Sobald der Guard den Gültigkeitsbereich verlässt (… | `src/snapshot.rs` |
| `pub fn register_at(self: &Arc<Self>, seq_no: u64, at_nanos: u64) -> SnapshotGuard` | Registriert einen Lese-Snapshot mit explizitem Zeitstempel ohne Mandantenzuordnung. | `src/snapshot.rs` |
| `pub fn register_for_tenant(self: &Arc<Self>, tenant: TenantId, seq_no: u64) -> SnapshotGuard` | Registriert einen Lese-Snapshot für einen bestimmten Mandanten. | `src/snapshot.rs` |
| `pub fn register_for_tenant_at( self: &Arc<Self>, tenant: TenantId, seq_no: u64, at_nanos: u64, ) -> SnapshotGuard` | Registriert einen Lese-Snapshot für einen bestimmten Mandanten mit explizitem Zeitstempel. | `src/snapshot.rs` |
| `pub fn revoke_covering( &self, tenant: TenantId, range: std::ops::Range<u64>, gated_seq: u64, ) -> usize` | Widerruft Snapshots im Bereich für einen Mandanten (Tenant-Isolation). | `src/snapshot.rs` |
| `pub fn seq_no(&self) -> u64` | Liefert die Sequenznummer, die durch diesen Guard geschützt wird. | `src/snapshot.rs` |
| `pub fn set_config(&mut self, config: SnapshotRegistryConfig)` | Setzt eine neue Konfiguration für die Registry. | `src/snapshot.rs` |
| `pub fn tenant(&self) -> Option<TenantId>` | Liefert den zugeordneten Mandanten, falls angegeben. | `src/snapshot.rs` |
| `pub fn try_register(self: &Arc<Self>, seq_no: u64) -> Result<SnapshotGuard>` | Versucht, einen Lese-Snapshot zu registrieren. Im Gegensatz zu `register` schlägt diese Methode fail-closed mit `ContextraError::LimitExcee… | `src/snapshot.rs` |
| `pub fn try_register_at(self: &Arc<Self>, seq_no: u64, at_nanos: u64) -> Result<SnapshotGuard>` | Versucht, einen Lese-Snapshot zu einem expliziten Nanosekunden-Zeitstempel zu registrieren. | `src/snapshot.rs` |
| `pub fn try_register_for_tenant( self: &Arc<Self>, tenant: TenantId, seq_no: u64, ) -> Result<SnapshotGuard>` | Versucht, einen Lese-Snapshot für einen bestimmten Mandanten zu registrieren. | `src/snapshot.rs` |
| `pub fn try_register_for_tenant_at( self: &Arc<Self>, tenant: Option<TenantId>, seq_no: u64, at_nanos: u64, ) -> Result<SnapshotGuard>` | Versucht, einen Lese-Snapshot für einen Mandanten zu einem expliziten Zeitstempel zu registrieren. | `src/snapshot.rs` |
| `pub fn unpin(&self, seq_no: u64)` | Hebt einen dauerhaften Pin auf und bereinigt den minimal aktiven Wasserstand. | `src/snapshot.rs` |
| `pub fn with_clock(clock: Arc<dyn Clock>) -> Self` | Erstellt eine neue Snapshot-Registry mit benutzerdefiniertem [`Clock`]-Port (für Tests). | `src/snapshot.rs` |
| `pub fn with_clock_and_config(clock: Arc<dyn Clock>, config: SnapshotRegistryConfig) -> Self` | Erstellt eine neue Snapshot-Registry mit benutzerdefiniertem [`Clock`]-Port und Konfiguration. | `src/snapshot.rs` |
| `pub fn with_time_fn<F>(time_fn: F) -> Self where F: Fn() -> u64 + Send + Sync + 'static,` | Erstellt eine neue Snapshot-Registry mit einer Zeitstempel-Closure (State-Passing per `Fn() -> u64`). | `src/snapshot.rs` |
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
| `pub fn new_with_time_fn_and_bounds<F>(time_fn: F, max_tracked_keys: usize) -> Self where F: Fn() -> u64 + Send + Sync + 'static,` | Erstellt einen neuen Validator mit einer Zeitstempel-Closure (State-Passing per `Fn() -> u64`) und Kapazitätsgrenze. | `src/ssi.rs` |
| `pub fn prefixes_iter(&self) -> impl Iterator<Item = (&Vec<u8>, &u64)>` | Liefert einen Iterator über alle Bereichs-Präfixe und deren Snapshot-Sequenzen. | `src/ssi.rs` |
| `pub fn prefixes_map(&self) -> &AHashMap<Vec<u8>, u64>` | Liefert die interne Präfix-Map als Referenz. | `src/ssi.rs` |
| `pub fn prune_through(&self, bound_seq: u64) -> usize` | Bereinigt (prunt) bestätigte Schreib-Einträge mit `commit_seq <= bound_seq`. Aktualisiert den atomaren `pruned_through`-Wasserstand. ### Fa… | `src/ssi.rs` |
| `pub fn prune_through_checked(&self, bound: Option<u64>) -> Result<usize>` | Bereinigt bestätigte Schreib-Einträge bis zu einem optionalen Grenzwert mit expliziter Fehlerbehandlung. ### Fehlerfälle: Liefert `Err(Cont… | `src/ssi.rs` |
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
| `pub fn validate_and_record<'a>( &self, tx_id: TxId, read_set: &ReadSet, write_keys: impl IntoIterator<Item = &'a [u8]>, commit_seq: u64, ) -> Result<()>` | Atomically validates `read_set` and records `write_keys` at `commit_seq` under a single write lock. Führt die Validierung und das Eintragen… | `src/ssi.rs` |
| `pub fn with_clock(mut self, clock: Arc<dyn Clock>) -> Self` | Bindet eine benutzerdefinierte Implementierung des [`Clock`]-Ports an den Validator. | `src/ssi.rs` |
| `pub fn with_max_pin_duration(mut self, duration: Duration) -> Self` | Konfiguriert die maximale Pin-Dauer für Diagnosealarme. | `src/ssi.rs` |
| `pub fn with_metrics_sink( mut self, sink: Arc<parking_lot::RwLock<Arc<dyn contextra_ports::MetricsSink>>>, ) -> Self` | Bindet einen Metrik-Empfänger ([`contextra_ports::MetricsSink`]) für Monitoring an. | `src/ssi.rs` |
| `pub fn with_sequence_log(seq_log: Arc<RwLock<SequenceLog>>) -> Self` | Erstellt einen Validator, der an ein bestehendes [`SequenceLog`] angebunden ist. | `src/ssi.rs` |
| `pub fn with_snapshot_registry(mut self, registry: Arc<SnapshotRegistry>) -> Self` | Verknüpft eine [`SnapshotRegistry`] zur Überwachung aktiver Snapshot-Pins. | `src/ssi.rs` |
| `pub fn with_time_fn<F>(self, time_fn: F) -> Self where F: Fn() -> u64 + Send + Sync + 'static,` | Bindet eine Zeitstempel-Closure (State-Passing per `Fn() -> u64`) an den Validator. | `src/ssi.rs` |
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
| `pub fn new_with_time_fn_and_config<F>( time_fn: F, shard_count: usize, config: TxBufferConfig, ) -> Self where F: Fn() -> u64 + Send + Sync + 'static,` | Erstellt einen Puffer mit einer Zeitstempel-Closure (State-Passing per `Fn() -> u64`), Shard-Anzahl und Konfiguration. | `src/tx_buffer.rs` |
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
| `pub fn with_time_fn<F>(self, time_fn: F) -> Self where F: Fn() -> u64 + Send + Sync + 'static,` | Bindet eine Zeitstempel-Closure (State-Passing per `Fn() -> u64`) an diesen Puffer an. | `src/tx_buffer.rs` |


---

## `contextra-sys`

| Attribut | Wert |
|---|---|
| **Ring** | 0 — Ring 0 — Foundation & Pure Numerics/Types |
| **Status** | Fertig |
| **Zweck** | System-Abstraktionen für mmap, mlock und Win32-ACLs (Unsafe-Insel). |
| **Unsafe-Code** | Ja (deklarierte Unsafe-Insel) |
| **Interne Abhängigkeiten** | — |
| **Externe Abhängigkeiten (Auswahl)** | `memmap2`, `thiserror`, `tracing` |

### Modul- und Re-Export-Oberfläche (lib.rs)

```rust
pub mod acl_win32;
pub mod durability;
pub mod error;
pub mod mlock;
pub mod mmap;
pub mod posix;
pub mod vault;
pub mod mem { .. }
pub mod fsmeta { .. }
pub mod map { .. }
pub use crate::mlock::{ apply_dont_dump, mem_lock, mem_lock_typed, mem_lock_with_policy, mem_unlock, MemLockPolicy, };
pub use crate::vault::{LockedRegions, VaultBuffer};
pub use crate::acl_win32::{set_restrictive_file_acl, verify_file_acl_owner_only};
pub use crate::mmap::{mmap_readonly, SealedFile, SealedMap, SealedReadOnly, Unsealed};
pub use acl_win32::{set_restrictive_file_acl, verify_file_acl_owner_only};
pub use durability::{durable_flush_dir, durable_flush_file, write_all_durable, DurableFile};
pub use error::{Error, Result};
pub use mlock::{ apply_dont_dump, mem_lock, mem_lock_typed, mem_lock_with_policy, mem_unlock, MemLockPolicy, };
pub use mmap::{mmap_readonly, SealedFile, SealedMap, SealedReadOnly, Unsealed};
pub use posix::{ensure_fd_limit, reopen_and_dup2, set_fd_limit};
pub use vault::{LockedRegions, VaultBuffer};
```

### Öffentliche Datentypen — Structs (7)

| Struct | Felder / Form | Beschreibung | Datei |
|---|---|---|---|
| `DurableFile` | `{ file: File, poisoned: bool, }` | Ein sicherer RAII-Wrapper um [`std::fs::File`], der die Invariante **INV-SYS-FSYNC-1** erzwingt: Sobald ein `fsync`/`fdatasync` oder Flush-Aufruf feh… | `src/durability.rs` |
| `LockedRegions` | `{ regions: Vec<(usize, usize)>, }` | Kapselt und verfolgt im RAM gesperrte Speicherbereiche (Adressen und Längen) für sensible Schlüsseldaten. | `src/vault.rs` |
| `SealedFile` | `{ file: File, expected_len: u64, _marker: std::marker::PhantomData<State>, }` | Ein typisierter Dateiwächter, der den Versiegelungszustand einer Datei zur Compilezeit und Laufzeit abbildet. | `src/mmap.rs` |
| `SealedMap` | `{ inner: memmap2::Mmap, }` | Sichere Nur-Lese-Speicherabbildung für versiegelte, unveränderliche Dateien (INV-IO-1). | `src/mmap.rs` |
| `SealedReadOnly` | `(tuple/unit)` | Typzustands-Marker: Zeigt an, dass eine Datei vollständig versiegelt, unveränderlich und prüfsummenverifiziert ist (gemäß INV-IO-1 und INV-ENCRYPT-RE… | `src/mmap.rs` |
| `Unsealed` | `(tuple/unit)` | Typzustands-Marker: Zeigt an, dass eine Datei noch unversiegelt oder potenziell veränderlich ist. | `src/mmap.rs` |
| `VaultBuffer` | `{ data: Vec<u8>, is_locked: bool, }` | Ein im RAM verankerter, geschützter Speicherpuffer für hochsensible kryptografische Schlüsseldaten (INV-SYS-MEMLOCK-1). | `src/vault.rs` |

### Öffentliche Datentypen — Enums (2)

| Enum | Varianten | Beschreibung | Datei |
|---|---|---|---|
| `Error` | `/// Ein Speicherpuffer konnte nicht im physischen RAM verankert werden (`mlock`/`VirtualLock`). #[error("memory locking failed (errno {errno}): {reason}")] MemoryLockFailed { /// Betriebssystemspezif…` | Typisierter Systemfehler für Low-Level-OS-Operationen. | `src/error.rs` |
| `MemLockPolicy` | `/// Fehlschläge beim Sperren werden als fataler Fehler behandelt und brechen die Operation ab. /// Einsatz z. B. bei hochsensiblen kryptografischen Masterschlüsseln (Fail-Closed). FailClosed, /// Feh…` | Konfigurierbare Richtlinie für das Sperren von Speicherregionen (`mlock` / `VirtualLock`). | `src/mlock.rs` |

### Typ-Aliase (1)

| Alias | Definition | Beschreibung |
|---|---|---|
| `Result` | `type Result<T> = std::result::Result<T, Error>` | Spezifischer Result-Typ für `contextra-sys`. |

### Öffentliche Funktionen (53)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn set_restrictive_file_acl(path: &Path) -> std::io::Result<()>` | Beschränkt die Zugriffsrechte der angegebenen Datei unter Unix auf den Besitzer (`0600` / `-rw-------`). # Ablauf 1. Konvertiert den oktale… | `src/acl_win32.rs` |
| `pub fn set_restrictive_file_acl(path: &Path) -> std::io::Result<()>` | Beschränkt unter Windows die Zugriffsrechte der angegebenen Datei auf den Prozessbesitzer mittels Win32-DACL. # Ablauf 1. Ruft die Besitzer… | `src/acl_win32.rs` |
| `pub fn set_restrictive_file_acl(path: &Path) -> std::io::Result<()>` | Best-Effort-Fallback für Zielarchitekturen, die weder Unix noch Windows sind (No-Op). | `src/acl_win32.rs` |
| `pub fn verify_file_acl_owner_only(path: &Path) -> std::io::Result<()>` | Verifiziert unter Unix, dass eine Datei strikt auf den Besitzer beschränkt ist (`0600`) und dem aktuellen Prozess gehört. # Ablauf 1. Liest… | `src/acl_win32.rs` |
| `pub fn verify_file_acl_owner_only(path: &Path) -> std::io::Result<()>` | Verifiziert unter Windows, dass die Datei über eine geschützte DACL verfügt, die ausschließlich dem aktuellen Prozessbesitzer Zugriff gewäh… | `src/acl_win32.rs` |
| `pub fn verify_file_acl_owner_only(path: &Path) -> std::io::Result<()>` | Best-Effort-Fallback für Zielarchitekturen, die weder Unix noch Windows sind (No-Op). | `src/acl_win32.rs` |
| `pub fn durable_flush_dir(path: &Path) -> IoResult<()>` | Flasht den Verzeichniseintrag (übergeordnetes Verzeichnis) einer neu erstellten oder umbenannten Datei. | `src/durability.rs` |
| `pub fn durable_flush_file(file: &File, metadata_changed: bool) -> IoResult<()>` | Flasht Dateiinhalt und optional Metadaten auf den physischen Datenträger gemäß den Garantien der Zielplattform. | `src/durability.rs` |
| `pub fn get_ref(&self) -> Result<&File>` | Gibt eine Referenz auf die innere [`File`] zurück, sofern das Handle nicht vergiftet ist. | `src/durability.rs` |
| `pub fn is_poisoned(&self) -> bool` | Gibt `true` zurück, wenn das Datei-Handle infolge eines früheren Flush-Fehlers vergiftet ist. | `src/durability.rs` |
| `pub fn new(file: File) -> Self` | Kapselt eine geöffnete [`File`] in einem gesunden, unvergifteten Zustand. | `src/durability.rs` |
| `pub fn set_fail_fsync(fail: bool, errno: i32)` | — | `src/durability.rs` |
| `pub fn should_fail_fsync() -> Option<i32>` | — | `src/durability.rs` |
| `pub fn sync_all(&mut self) -> Result<()>` | Flasht Dateiinhalt und Metadaten auf das physische Speichermedium. | `src/durability.rs` |
| `pub fn sync_data(&mut self) -> Result<()>` | Flasht den Dateiinhalt (ohne Metadaten, z. B. via `fdatasync` unter Linux). | `src/durability.rs` |
| `pub fn write_all(&mut self, buf: &[u8]) -> Result<()>` | Schreibt Daten dauerhaft in die Datei. Schlägt sofort fehl, wenn das Handle vergiftet ist. | `src/durability.rs` |
| `pub fn write_all_durable(file: &mut File, mut buf: &[u8]) -> IoResult<()>` | Schreibt die gesamten Bytes aus `buf` zuverlässig in `file`, unter robuster Behandlung partieller Schreibvorgänge und Signal-Unterbrechunge… | `src/durability.rs` |
| `pub fn apply_dont_dump(ptr: *const u8, len: usize) -> Result<()>` | Schließt eine Speicherregion per `madvise(MADV_DONTDUMP)` aus Core-Dumps aus (Unix). Auf Nicht-Unix-Systemen ist dies ein no-op. | `src/mlock.rs` |
| `pub fn mem_lock(ptr: *const u8, len: usize) -> bool` | Sperrt eine Speicherregion im RAM mit der Standard-Richtlinie [`MemLockPolicy::Tolerant`]. | `src/mlock.rs` |
| `pub fn mem_lock_typed(ptr: *const u8, len: usize, policy: MemLockPolicy) -> Result<bool>` | Sperrt eine Speicherregion im RAM gemäß Richtlinie und liefert ein typisiertes [`Result<bool>`]. | `src/mlock.rs` |
| `pub fn mem_lock_with_policy(ptr: *const u8, len: usize, policy: MemLockPolicy) -> IoResult<bool>` | Sperrt unter POSIX/Windows eine Speicherregion im RAM, um das Auslagern (Swapping) zu verhindern. | `src/mlock.rs` |
| `pub fn mem_unlock(addr: usize, len: usize)` | Entsperrt eine zuvor gesperrte Speicherregion im RAM. | `src/mlock.rs` |
| `pub fn set_fail_mlock(fail: bool, errno: i32)` | — | `src/mlock.rs` |
| `pub fn should_fail_mlock() -> Option<i32>` | — | `src/mlock.rs` |
| `pub fn as_file(&self) -> &File` | Gibt eine Referenz auf die zugrundeliegende [`File`] zurück. | `src/mmap.rs` |
| `pub fn as_slice(&self) -> &[u8]` | Gibt den gesamten Speicherinhalt als Byte-Slice zurück. | `src/mmap.rs` |
| `pub fn get_subslice(&self, offset: usize, len: usize) -> Result<&[u8]>` | Liest einen Teilbereich der Speicherabbildung mit sicherer Grenzenprüfung. | `src/mmap.rs` |
| `pub fn is_empty(&self) -> bool` | Prüft, ob die Speicherabbildung leer ist (0 Bytes). | `src/mmap.rs` |
| `pub fn len(&self) -> usize` | Gibt die Länge der Speicherabbildung in Bytes zurück. | `src/mmap.rs` |
| `pub fn map_readonly(&self) -> io::Result<SealedMap>` | Mappt die versiegelte Datei über die typgesicherte [`SealedMap`]-Abstraktion. | `src/mmap.rs` |
| `pub fn map_sealed(file: &SealedFile<SealedReadOnly>) -> io::Result<Self>` | Mappt eine nachweislich versiegelte Datei in den virtuellen Adressraum. | `src/mmap.rs` |
| `pub fn mmap_readonly(file: &File) -> io::Result<memmap2::Mmap>` | Erstellt eine unveränderliche Nur-Lese-Speicherabbildung (Read-Only Memory Map) für eine geöffnete Datei. | `src/mmap.rs` |
| `pub fn new_verified(file: File, expected_len: u64) -> io::Result<Self>` | Erstellt eine versiegelte Dateireferenz nach erfolgreicher Integritäts- und Prüfsummenverifikation. | `src/mmap.rs` |
| `pub fn verified_len(&self) -> u64` | Gibt die verifizierte Dateilänge in Bytes zurück. | `src/mmap.rs` |
| `pub fn ensure_fd_limit(min_fds: u64) -> bool` | Stellt sicher, dass das weiche Limit für offene Dateideskriptoren (`RLIMIT_NOFILE`) mindestens `min_fds` beträgt. | `src/posix.rs` |
| `pub fn reopen_and_dup2(path: &Path, target_fd: i32, flags: i32) -> std::io::Result<()>` | Öffnet den Pfad `path` mit den angegebenen POSIX-Flags (z. B. `O_RDONLY`) und ersetzt atomar `target_fd` via `dup2`. | `src/posix.rs` |
| `pub fn reopen_and_dup2(_path: &Path, _target_fd: i32, _flags: i32) -> std::io::Result<()>` | Fallback für Nicht-Linux-Systeme, auf denen `reopen_and_dup2` nicht unterstützt wird. | `src/posix.rs` |
| `pub fn set_fd_limit(limit: u64) -> bool` | Setzt das weiche und harte Dateideskriptorlimit (`RLIMIT_NOFILE`) des aktuellen Prozesses exakt auf `limit`. | `src/posix.rs` |
| `pub fn allocate(size: usize) -> Result<Self>` | Allokiert einen neuen Puffer der Größe `size` mit Nullen, verankert ihn im RAM (`MemLockPolicy::FailClosed`) und schließt ihn von Core-Dump… | `src/vault.rs` |
| `pub fn allocate_with_policy(size: usize, policy: MemLockPolicy) -> Result<Self>` | Allokiert einen neuen Puffer der Größe `size` gemäß der übergebenen Sperr-Richtlinie. | `src/vault.rs` |
| `pub fn as_mut_slice(&mut self) -> &mut [u8]` | Liefert die veränderbare Sicht auf den Pufferinhalt. | `src/vault.rs` |
| `pub fn as_slice(&self) -> &[u8]` | Liefert die unveränderliche Sicht auf den Pufferinhalt. | `src/vault.rs` |
| `pub fn from_slice(source: &[u8]) -> Result<Self>` | Allokiert einen Puffer und initialisiert ihn sofort mit einer Kopie der übergebenen sensiblen Daten. | `src/vault.rs` |
| `pub fn is_empty(&self) -> bool` | Gibt `true` zurück, wenn aktuell keine gesperrten Speicherregionen verwaltet werden. | `src/vault.rs` |
| `pub fn is_empty(&self) -> bool` | Prüft, ob der Puffer 0 Bytes lang ist. | `src/vault.rs` |
| `pub fn is_locked(&self) -> bool` | Gibt `true` zurück, wenn der Speicherbereich aktiv im physischen RAM verankert ist. | `src/vault.rs` |
| `pub fn len(&self) -> usize` | Gibt die Anzahl der aktuell nachverfolgten, gesperrten Regionen zurück. | `src/vault.rs` |
| `pub fn len(&self) -> usize` | Gibt die Kapazität/Länge des Puffers in Bytes zurück. | `src/vault.rs` |
| `pub fn lock_slice(&mut self, slice: &[u8], attempt_mlock: bool) -> bool` | Versucht einen Byte-Slice im RAM zu sperren, falls `attempt_mlock` wahr ist, unter Verwendung der Standardrichtlinie [`MemLockPolicy::Toler… | `src/vault.rs` |
| `pub fn lock_slice_with_policy( &mut self, slice: &[u8], attempt_mlock: bool, policy: MemLockPolicy, ) -> std::io::Result<bool>` | Versucht einen Byte-Slice im RAM gemäß der angegebenen [`MemLockPolicy`] zu sperren, falls `attempt_mlock == true`. | `src/vault.rs` |
| `pub fn new() -> Self` | Erstellt eine neue, leere Nachverfolgungsinstanz für gesperrte Speicherregionen. | `src/vault.rs` |
| `pub fn unlock_all(&mut self)` | Entsperrt alle registrierten Speicherregionen und leert die interne Nachverfolgungsliste. | `src/vault.rs` |
| `pub fn zeroize_slice(slice: &mut [u8])` | Überschreibt einen veränderbaren Byte-Slice sicher mit Nullen unter Verwendung von flüchtigen Schreibzugriffen (`write_volatile`) und einer… | `src/vault.rs` |


---

## `contextra-simd`

| Attribut | Wert |
|---|---|
| **Ring** | 0 — Ring 0 — Foundation & Pure Numerics/Types |
| **Status** | Fertig |
| **Zweck** | AVX2/AVX-512/NEON SIMD-Distanzberechnungskerne mit Laufzeit-Dispatch (Unsafe-Insel). |
| **Unsafe-Code** | Ja (deklarierte Unsafe-Insel) |
| **Interne Abhängigkeiten** | `contextra-types` |
| **Externe Abhängigkeiten (Auswahl)** | `bytemuck`, `serde`, `thiserror`, `tracing` |

### Architekturbeschreibung

# Crate `contextra-simd`  **Ring-Zugehörigkeit:** Ring 0 (Kern, unsafe-Insel).  **Zweck:** Diese Crate stellt hochoptimierte Vektor-Distanzberechnungen bereit. SIMD (Single Instruction, Multiple Data) nutzt spezielle Prozessorregister (z. B. 128-Bit bei ARM NEON, 256-Bit bei x86 AVX2, 512-Bit bei x86 AVX-512), um mehrere Gleitkomma- oder Ganzzahl-Operationen parallel mit einem einzigen CPU-Befehl auszuführen.  **Verbindung im System:** - **Abhängigkeiten (normal):** `bytemuck`, `thiserror`. Optional: `contextra-types` (nur transitional via Feature `types-compat`), `serde`, `tracing`. - **Wird eingebunden von:** `contextra-vector` (Ring 1) zur Beschleunigung von Ähnlichkeitssuchen in Vektorindizes (z. B. HNSW = Hierarchical Navigable Small World).  **Architektur:** 1. `kernels/`: Enthält isolierte Implementierungen für Skalar (Referenz), AVX2, AVX-512 und ARM NEON. 2. `dispatch.rs`: Ermittelt beim Start dynamisch die CPU-Fähigkeiten und wählt den schnellsten sicheren Pfad. 3. `lib.rs`: Bietet die sichere öffentliche Schnittstelle mit Vorab-Validierung gegen ungültige Werte (NaN/Unendlich).

### Modul- und Re-Export-Oberfläche (lib.rs)

```rust
pub mod dispatch;
pub mod error;
pub mod kernels;
pub mod metric;
pub use dispatch::{ cosine_distance, cosine_distance_f32_bytes, cosine_similarity_parts_u8, dot_product_distance, dot_product_distance_f32_bytes, dot_product_u8, euclidean_distance, euclidean_distance_f32_bytes, euclidean_distance_sq_u8, Dispatcher, SimdTarget, };
pub use dispatch::{get_dispatch_override, set_dispatch_override, SimdPathOverride};
pub use error::{Error, Result, SimdError};
pub use kernels::scalar::{ cosine_distance_f32_bytes_scalar, cosine_distance_scalar, cosine_similarity_parts_f32_u8, cosine_similarity_parts_u8_scalar, dot_product_f32_bytes_scalar, dot_product_f32_u8, dot_product_scalar, dot_product_u8_scalar, euclidean_distance_f32_bytes_scalar, euclidean_distance_scalar, euclidean_distance_sq_f32_u8, euclidean_distance_sq_u8_scalar, normalize_inplace, CosineSimilarityPartsF32U8, CosineSimilarityPartsU8, };
pub use metric::DistanceMetric;
```

### Feature-Flags

| Flag | Aktiviert |
|---|---|
| `default` | `["std"]` |
| `std` | `[]` |
| `serde` | `["dep:serde"]` |
| `types-compat` | `["dep:contextra-types"]` |
| `simd-avx512` | `[]` |

### Öffentliche Datentypen — Structs (3)

| Struct | Felder / Form | Beschreibung | Datei |
|---|---|---|---|
| `CosineSimilarityPartsF32U8` | `{ /// Accumulated dot product $\sum a_i (b_i \text{ as } f32)$. pub dot: f32, /// Accumulated squared L2 norm $\sum a_i^2$. pub norm_a_sq: f32, /// Accumulated s… }` | Struct holding raw float sum components for f32/u8 mixed cosine similarity calculation. | `src/kernels/scalar.rs` |
| `CosineSimilarityPartsU8` | `{ /// Accumulated dot product $\sum a_i b_i$. pub dot: u32, /// Accumulated squared L2 norm $\sum a_i^2$. pub norm_a_sq: u32, /// Accumulated squared L2 norm $\s… }` | Struct holding raw integer sum components for u8 cosine similarity calculation. | `src/kernels/scalar.rs` |
| `Dispatcher` | `{ target: SimdTarget, }` | Injectable and thread-safe SIMD dispatcher (Design by Contract). Encapsulates SIMD hardware target selection and execution without relying on mutable… | `src/dispatch.rs` |

### Öffentliche Datentypen — Enums (3)

| Enum | Varianten | Beschreibung | Datei |
|---|---|---|---|
| `DistanceMetric` | `Cosine, Euclidean, DotProduct,` | — | `src/metric.rs` |
| `SimdError` | `#[error("invalid input: {0}")] InvalidInput(&'static str), #[error("dimension mismatch: left={left}, right={right}")] DimensionMismatch { left: usize, right: usize }, #[error("unsupported metric")] U…` | — | `src/error.rs` |
| `SimdTarget` | `/// Automatically detects CPU features at runtime (default). #[default] Auto, /// Forces scalar fallback execution without unsafe SIMD intrinsics. Scalar, /// Forces AVX2 + FMA SIMD execution path on…` | Explicit hardware target configuration for the SIMD dispatcher. | `src/dispatch.rs` |

### Typ-Aliase (3)

| Alias | Definition | Beschreibung |
|---|---|---|
| `Error` | `type Error= SimdError` | Typ-Alias `Error` für B4-Konformität (genau ein primärer Fehlertyp `Error`). |
| `Result` | `type Result<T> = core::result::Result<T, SimdError>` | — |
| `SimdPathOverride` | `type SimdPathOverride= SimdTarget` | SIMD path override options for testing purposes. |

### Öffentliche Funktionen (39)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn auto() -> Self` | Creates a new `Dispatcher` with automatic CPU feature detection. | `src/dispatch.rs` |
| `pub fn cosine_distance(&self, a: &[f32], b: &[f32]) -> Result<f32>` | Computes cosine distance ($1.0 - \text{cosine\_similarity}$) between two `f32` vectors. | `src/dispatch.rs` |
| `pub fn cosine_distance(a: &[f32], b: &[f32]) -> Result<f32>` | Computes cosine distance ($1.0 - \text{cosine\_similarity}$) between two `f32` vectors. Dynamically dispatches to AVX-512, AVX2+FMA, or NEO… | `src/dispatch.rs` |
| `pub fn cosine_distance_f32_bytes(&self, a: &[f32], b_bytes: &[u8]) -> Result<f32>` | Computes cosine distance between an `f32` slice and raw little-endian `f32` byte slice. | `src/dispatch.rs` |
| `pub fn cosine_distance_f32_bytes(a: &[f32], b_bytes: &[u8]) -> Result<f32>` | Computes cosine distance between an `f32` slice and raw little-endian `f32` byte slice. Dynamically dispatches to AVX-512 or AVX2+FMA based… | `src/dispatch.rs` |
| `pub fn cosine_similarity_parts_u8( &self, a: &[u8], b: &[u8], ) -> Result<CosineSimilarityPartsU8>` | Computes cosine similarity component parts (dot product, norm_a_sq, norm_b_sq) for two `u8` byte slices. | `src/dispatch.rs` |
| `pub fn cosine_similarity_parts_u8(a: &[u8], b: &[u8]) -> Result<CosineSimilarityPartsU8>` | Computes cosine similarity component parts (dot product, norm_a_sq, norm_b_sq) for two `u8` byte slices. Dynamically dispatches to AVX-512… | `src/dispatch.rs` |
| `pub fn detect_best_target() -> SimdTarget` | Detects the best supported SIMD target on the host CPU, with support for caching and env override. | `src/dispatch.rs` |
| `pub fn dot_product_distance(&self, a: &[f32], b: &[f32]) -> Result<f32>` | Computes dot product distance (negated dot product $-a \cdot b$) between two `f32` vectors. | `src/dispatch.rs` |
| `pub fn dot_product_distance(a: &[f32], b: &[f32]) -> Result<f32>` | Computes dot product distance (negated dot product $-a \cdot b$) between two `f32` vectors. Dynamically dispatches to AVX-512, AVX2+FMA, or… | `src/dispatch.rs` |
| `pub fn dot_product_distance_f32_bytes(&self, a: &[f32], b_bytes: &[u8]) -> Result<f32>` | Computes dot product distance between an `f32` slice and raw little-endian `f32` byte slice. | `src/dispatch.rs` |
| `pub fn dot_product_distance_f32_bytes(a: &[f32], b_bytes: &[u8]) -> Result<f32>` | Computes dot product distance between an `f32` slice and raw little-endian `f32` byte slice. Dynamically dispatches to AVX-512 or AVX2+FMA… | `src/dispatch.rs` |
| `pub fn dot_product_u8(&self, a: &[u8], b: &[u8]) -> Result<u32>` | Computes integer dot product between two `u8` byte slices. | `src/dispatch.rs` |
| `pub fn dot_product_u8(a: &[u8], b: &[u8]) -> Result<u32>` | Computes integer dot product between two `u8` byte slices. Dynamically dispatches to AVX-512 VNNI (`dpbusd`) or AVX2 (`madd_epi16`), fallin… | `src/dispatch.rs` |
| `pub fn euclidean_distance(&self, a: &[f32], b: &[f32]) -> Result<f32>` | Computes Euclidean distance ($\|a - b\|_2$) between two `f32` vectors. | `src/dispatch.rs` |
| `pub fn euclidean_distance(a: &[f32], b: &[f32]) -> Result<f32>` | Computes Euclidean distance ($\|a - b\|_2$) between two `f32` vectors. Dynamically dispatches to AVX-512, AVX2+FMA, or NEON based on runtim… | `src/dispatch.rs` |
| `pub fn euclidean_distance_f32_bytes(&self, a: &[f32], b_bytes: &[u8]) -> Result<f32>` | Computes Euclidean distance between an `f32` slice and raw little-endian `f32` byte slice. | `src/dispatch.rs` |
| `pub fn euclidean_distance_f32_bytes(a: &[f32], b_bytes: &[u8]) -> Result<f32>` | Computes Euclidean distance between an `f32` slice and raw little-endian `f32` byte slice. Dynamically dispatches to AVX-512 or AVX2+FMA ba… | `src/dispatch.rs` |
| `pub fn euclidean_distance_sq_u8(&self, a: &[u8], b: &[u8]) -> Result<u32>` | Computes squared Euclidean distance ($\|a - b\|_2^2$) between two `u8` byte slices. | `src/dispatch.rs` |
| `pub fn euclidean_distance_sq_u8(a: &[u8], b: &[u8]) -> Result<u32>` | Computes squared Euclidean distance ($\|a - b\|_2^2$) between two `u8` byte slices. Dynamically dispatches to AVX-512 or AVX2 SIMD implemen… | `src/dispatch.rs` |
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
| `pub fn compute_distance(a: &[f32], b: &[f32], metric: DistanceMetric) -> Result<f32>` | Berechnet die Distanz zwischen zwei Vektoren unter Verwendung der angegebenen Metrik inklusive Validierung. Diese Funktion ist der primäre… | `src/lib.rs` |
| `pub fn compute_distance_f32_bytes_trusted( a: &[f32], b_bytes: &[u8], metric: DistanceMetric, ) -> Result<f32>` | Berechnet die Distanz zwischen einem `f32`-Vektor und einem rohen Byte-Puffer (`&[u8]`), der `f32`-Werte darstellt. Dies ist besonders nütz… | `src/lib.rs` |
| `pub fn compute_distance_trusted(a: &[f32], b: &[f32], metric: DistanceMetric) -> Result<f32>` | Berechnet die Distanz zwischen zwei bereits validierten `f32`-Vektoren ohne erneute NaN-Prüfung. Wird intern verwendet, wenn Vektoren berei… | `src/lib.rs` |
| `pub fn validate_vector(vec: &[f32]) -> Result<()>` | Prüft, ob ein Vektor gültige endliche Zahlen enthält (kein NaN und kein Unendlich). NaN (Not a Number) und Unendlich führen in SIMD-Registe… | `src/lib.rs` |


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
pub mod aead;
pub mod anti_tamper;
pub mod audit_chain;
pub mod crypto;
pub mod dek;
pub mod deletion_proof;
pub mod ed25519_proof;
pub mod error;
pub mod kdf;
pub mod kv_cipher;
pub mod kv_segment;
pub mod kv_shredding;
pub mod revocation_log;
pub mod secret;
pub mod wal_completeness;
pub mod wal_crypto;
pub use aead::*;
pub use dek::{derive_dek, AncestorSalts, Dek, UnitKey};
pub use kdf::hkdf_expand;
pub use secret::SecretKey;
pub use audit_chain::{ compute_record_commitment, AuditChain, AuditChainEntry, AuditChainHeadSignature, DataClass, EncryptedCommitmentSalt, };
pub use crypto::KeyManager as CryptoKey;
pub use crypto::KeyManager;
pub use deletion_proof::{ CustodyLevel, DeletionLayer, DeletionProof, DeletionProofKeyPair, DeletionScope, ExcludedScope, LayerCleanupProof, SigningKeyHandle, };
pub use ed25519_proof::{DeletionProofError, SignatureVersion};
pub use error::{CryptoError, Result};
pub use kdf::{derive_key_argon2id, KdfHeader, KdfParams};
pub use kv_cipher::KvSegmentCipher;
pub use kv_cipher::{EncryptedKvLayer, KvCipher, ModelFingerprint};
pub use kv_segment::{ delete_mode::KvDeleteMode, eviction_worker::{emergency_wipe, EvictionWorker}, manager::{KvSegmentConfig, KvSegmentManager, KvSegmentPayload}, segment::KvSegment, store::TenantIsolatedKvStore, CURRENT_KV_KEY_DERIVATION_VERSION, };
pub use kv_shredding::{derive_subkey, KeyRegistry, SubKey, DEFAULT_SHRED_KEY_GROUP_SIZE};
pub use revocation_log::{RevocationEntry, RevocationLog, RevocationTarget};
pub use wal_completeness::verify_wal_chain_completeness;
pub use wal_crypto::CloudEgressGuard;
pub use wal_crypto::{ EncryptedWal, IntegrityVerifier, KmsProvider, LocalKeyManagerKms, WalEntrySnapshot, WalHmac, };
```

### Feature-Flags

| Flag | Aktiviert |
|---|---|
| `default` | `["legacy-erasure", "kv-encryption", "test-utils"]` |
| `legacy-erasure` | `[]` |
| `test-utils` | `[]` |
| `kv-encryption` | `["legacy-erasure"]` |
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

### Öffentliche Datentypen — Structs (46)

| Struct | Felder / Form | Beschreibung | Datei |
|---|---|---|---|
| `AncestorSalts` | `{ pub t: Salt, pub c: Salt, pub s: Option<Salt>, pub d: Salt, }` | — | `src/dek.rs` |
| `AuditChain` | `{ /// Öffentlicher Zugriff auf die Liste der Ketten-Einträge für Prüfungen/Tests. pub entries: Vec<AuditChainEntry>, head_hash: [u8; 32], }` | Manipulationssichere Audit-Kette mit BLAKE3-Hash-Verkettung. | `src/audit_chain.rs` |
| `AuditChainEntry` | `{ /// Fortlaufender Index in der Audit-Kette (0-basiert). pub index: u64, /// Identifikator des Datenschemas. pub schema_id: String, /// Blake3-Hash des zum Zeit… }` | Einzelner Eintrag in der manipulationssicheren BLAKE3-Audit-Hash-Kette. | `src/audit_chain.rs` |
| `AuditChainHeadSignature` | `{ /// Index des signierten Kettenkopfs. pub chain_index: u64, /// Blake3-Hash des Kettenkopfs. pub head_hash: [u8; 32], /// Ed25519-Signatur (64 Bytes). pub sign… }` | Periodisch erzeugte Ed25519-Signatur über den aktuellen Kettenkopf. | `src/audit_chain.rs` |
| `AuditHeadAnchor` | `{ /// Sequenznummer / Kettenindex des extern verankerten Kettenkopfs. pub seq_no: u64, /// Blake3-Hash des Kettenkopfs an der Position `seq_no`. pub head_hash: [… }` | Externer Kopf-Anker zur Rollback-Erkennung für die Audit-Hash-Kette. | `src/audit_chain.rs` |
| `Blake3Chain` | `{ current_hash: [u8; 32], step_count: u64, }` | BLAKE3-basierte Integritätskette für sequentielle Log- oder Datenblöcke. | `src/anti_tamper.rs` |
| `CloudEgressGuard` | `{ max_egress_bytes: usize, }` | Egress guard for inspecting and sanitizing encrypted payload blocks prior to cloud egress. | `src/wal_crypto.rs` |
| `Dek` | `(tuple/unit)` | Data Encryption Key for per-record / per-block encryption. | `src/dek.rs` |
| `DeletionProof` | `{ /// Version der Signatur-Payload-Konstruktion. #[serde(default = "default_signature_version")] pub signature_version: u8, /// Target scope of deletion. pub sco… }` | Kryptographisches Attestationszertifikat (Deletion Proof) und Attestationsnachweis. | `src/deletion_proof/types.rs` |
| `DeletionProofKeyPair` | `{ /// Privater Signierschlüssel. pub signing_key: ed25519_dalek::SigningKey, /// Öffentlicher Verifikationsschlüssel. #[zeroize(skip)] pub verifying_key: ed25519… }` | Schlüsselpaar für Ed25519-Attestationsbeweise. Signierschlüssel wird bei Drop automatisch zeroisiert (`ZeroizeOnDrop`). | `src/ed25519_proof.rs` |
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
| `KvSegment` | `{ #[zeroize(skip)] // Metadaten, keine sensiblen Tensor-Daten pub tenant_id: TenantId, #[zeroize(skip)] pub segment_id: u64, /// Versionsnummer der HKDF-Schlüsse… }` | — | `src/kv_segment/segment.rs` |
| `KvSegmentCipher` | `{ key_manager: KeyManager, registry: KeyRegistry, }` | Chiffre für segmentweise verschlüsselte KV-Einträge. | `src/kv_cipher.rs` |
| `KvSegmentConfig` | `{ /// Delete mode specifying whether segments are tombstone-only or crypto-shredded. pub delete_mode: KvDeleteMode, }` | Configuration for KV segment storage and shredding behavior. | `src/kv_segment/manager.rs` |
| `KvSegmentManager` | `{ config: KvSegmentConfig, registry: Arc<KeyRegistry>, master_key: Option<Arc<KeyManager>>, }` | — | `src/kv_segment/manager.rs` |
| `KvSegmentPayload` | `{ /// Group ID corresponding to the sub-key in . pub group_id: u64, /// Physical payload stored on disk (ciphertext or raw plaintext). pub payload: Vec<u8>, ///… }` | Representation of a written KV segment (encrypted or raw payload with group_id and nonce). | `src/kv_segment/manager.rs` |
| `LayerCleanupProof` | `{ pub(crate) layer: DeletionLayer, pub(crate) _private: (), }` | Beweis, dass ein bestimmter DeletionLayer physisch bereinigt wurde. | `src/deletion_proof/types.rs` |
| `LayerWitnessMeasurement` | `{ /// Layer that was measured. pub layer: DeletionLayer, /// Applied scan technique. #[serde(with = "scan_technique_serde")] pub technique: ScanTechnique, /// Nu… }` | Measurement details for a single witnessed layer in V5 proof. | `src/deletion_proof/types.rs` |
| `LocalKeyManagerKms` | `{ key_manager: KeyManager, key_id: String, rotation_counter: u64, }` | Standard local implementation of [`KmsProvider`] wrapping [`KeyManager`]. Provides thread-safe, zeroized key derivation, wrapping, unwrapping, and ro… | `src/wal_crypto.rs` |
| `RecordDek` | `(tuple/unit)` | Record Data Encryption Key (DEK) - 256 bits, zeroized on drop. | `src/kv_shredding.rs` |
| `RevocationEntry` | `{ /// Monotone Sequenznummer im Log (0, 1, 2, ...). pub sequence_number: u64, /// Das widerrufene Target. pub target: RevocationTarget, /// Widerrufszeitpunkt in… }` | Ein einzelner Eintrag im manipulationssicheren Widerrufslog. | `src/revocation_log.rs` |
| `RevocationLog` | `{ storage: Option<(Arc<dyn Vfs>, PathBuf)>, signing_key: Option<SigningKey>, verifying_key: VerifyingKey, clock: Arc<dyn Clock>, entries: RwLock<Vec<RevocationEn… }` | Persistentes Protokoll über widerrufene Schlüsselgruppen. | `src/revocation_log.rs` |
| `Secret` | `{ inner: T, }` | Dynamically-sized secret container that is wiped on drop and never printed. | `src/secret.rs` |
| `SecretKey` | `{ bytes: [u8; N], }` | Fixed-size secret key that is wiped on drop and never printed. | `src/secret.rs` |
| `SubKey` | `(tuple/unit)` | Sub-key or KEK bytes (256 bits) zeroized on drop. | `src/kv_shredding.rs` |
| `TenantIsolatedKvStore` | `{ /// 32 cache-line-gepaddete RwLock-Shards. Box<[_]> für flexible Shard-Anzahl. shards: Box<[Shard]>, shard_count: usize, /// Globaler Offset für Shard-Rotation… }` | — | `src/kv_segment/store.rs` |
| `UncoveredLayer` | `{ /// Layer that was not covered in measurement. pub layer: DeletionLayer, /// Justification why layer was not covered for the given scope. pub reason: String, }` | Uncovered layer explanation for scope. | `src/deletion_proof/types.rs` |
| `UnitKey` | `(tuple/unit)` | Key representing a storage or tenant unit. | `src/dek.rs` |
| `VerificationResult` | `{ /// `true` if all cryptographic checks, signatures, and chain continuities passed. pub is_valid: bool, /// Detailed error reason if `is_valid` is `false`. pub… }` | Structured outcome of third-party deletion proof verification. | `src/deletion_proof/verifier.rs` |
| `VolatileEncryptionKey` | `{ key_bytes: Zeroizing<[u8; 32]>, // AES-256 Key }` | Defines a cryptographic key that is explicitly zeroed out when dropped or when an emergency trigger is activated, protecting against cold-boot attack… | `src/anti_tamper.rs` |
| `WalEntrySnapshot` | `{ pub tx_id: u64, pub seq_no: u64, pub op_type: u8, // 0: Put, 1: Delete, 2: TxEnd pub key: Vec<u8>, pub value: Vec<u8>, pub checksum: [u8; 32], pub prev_hmac: [… }` | Immutable snapshot of a WAL entry used for cryptographic integrity verification. # Invariants - Captures entry fields (`seq_no`, `op_type`, `key`, `v… | `src/wal_crypto.rs` |
| `WalHmac` | `{ mac: Hmac<Sha256>, }` | Stateful wrapper around HMAC-SHA256 initialized with WAL domain separation. # Invariants - Initialized with `b"contextra-wal-v1"` domain separation s… | `src/wal_crypto.rs` |

### Öffentliche Datentypen — Enums (13)

| Enum | Varianten | Beschreibung | Datei |
|---|---|---|---|
| `CryptoError` | `#[error("key derivation failed: {0}")] KeyDerivation(String), #[error("encryption failed: {0}")] Encryption(String), #[error("decryption failed: {0}")] Decryption(String), #[error("AEAD operation fai…` | Standalone error type for all operations within `contextra-crypto`. | `src/error.rs` |
| `CustodyLevel` | `/// Local custody. Local, /// Sovereign custody. Sovereign, /// Escrowed custody. Escrow,` | Custody level for cryptographic key management during deletion proof generation. | `src/deletion_proof/types.rs` |
| `DataClass` | `/// Öffentliche Daten. Public, /// Interne Betriebsdaten. Internal, /// Vertrauliche Geschäftsdaten. Confidential, /// Streng vertrauliche / reglementierte Daten. Restricted, /// Personenbezogene Dat…` | Datenklassifikation für Audit-Einträge (keine Freitext-Strings). | `src/audit_chain.rs` |
| `DeletionLayer` | `/// In-memory memtable records cleared. LsmMemtable, /// Persistent SSTable files purged across all LSM levels. SsTableAllLevels, /// HNSW graph nodes and tombstone references purged. HnswIndex, ///…` | Layer-explizite Coverage-Deklaration. | `src/deletion_proof/types.rs` |
| `DeletionProofError` | `/// Ungültiger Ed25519 VerifyingKey (32 Bytes). #[error("invalid verifying key: {0}")] InvalidVerifyingKey(String), /// Ungültige Ed25519 Signatur (64 Bytes) oder Fehlschlag bei der Verifikation. #[e…` | Fehlerzustände bei der Verifikation von Ed25519-Attestationsbeweisen. | `src/ed25519_proof.rs` |
| `DeletionScope` | `/// Individual document deletion. Document { /// DocId of target document. doc_id: DocId, /// TenantId of owning tenant. tenant_id: TenantId, }, /// Collection-wide deletion. Collection { /// Collect…` | Target scope for physical deletion. | `src/deletion_proof/types.rs` |
| `ExcludedScope` | `/// Wissen in Zusammenfassungen die als LLM-Fine-Tuning-Input dienten. ConsolidatedAndDistilled, /// LLM-Modellparameter (arXiv:2505.16831 — Unlearning Isn't Deletion). LlmParameterMemory,` | Explizite Nicht-Abdeckung — maschinenlesbar für Audit-Systeme. | `src/deletion_proof/types.rs` |
| `KvDeleteMode` | `TombstoneOnly, CryptoShred,` | — | `src/kv_segment/delete_mode.rs` |
| `RevocationTarget` | `/// Shredding Group ID (z. B. für `KeyRegistry`) Group(u64), /// Key Encryption Key (KEK) Identifier Kek(String), /// Data Encryption Key (DEK) Identifier Dek(String), /// Specific Record Identifier…` | Das Ziel eines Widerruf-Eintrags im Log (Group ID, KEK ID, DEK ID oder Record ID). | `src/revocation_log.rs` |
| `SignatureVersion` | `/// HMAC-SHA256 legacy scope-only signature (version 1) V1 = 1, /// HMAC-SHA256 scope + layer coverage signature (version 2) V2 = 2, /// Ed25519 asymmetric signature with length-prefixed key hashes (…` | `SignatureVersion`-Enum für typsichere Versionierung von DeletionProof-Signaturen. Referenz: Befund v17 Teil 10.1 & INVARIANTE INV-DELETION-1 (die ph… | `src/ed25519_proof.rs` |
| `SigningKeyHandle` | `/// Symmetric HMAC key slice. Hmac(&'a [u8]), /// Ed25519 signing key reference. Ed25519(&'a ed25519_dalek::SigningKey),` | Handle to the key used for signing deletion proofs. | `src/deletion_proof/types.rs` |
| `VerificationErrorReason` | `/// Digital signature is invalid or tampered with. InvalidSignature(String), /// Unbroken chain invariant violated (hash mismatch or missing predecessor). ChainBroken { /// Position in chain where br…` | Detailed failure reasons for third-party verification of deletion proofs and audit chains. | `src/deletion_proof/verifier.rs` |
| `VerificationKey` | `/// Key for HMAC-SHA256 signature verification (version 1 and 2). Hmac(&'a [u8]), /// Key for Ed25519 signature verification (version 3). Ed25519(&'a ed25519_dalek::VerifyingKey),` | Key parameter for verification (either HMAC-SHA256 byte slice or Ed25519 VerifyingKey). | `src/deletion_proof/types.rs` |

### Typ-Aliase (4)

| Alias | Definition | Beschreibung |
|---|---|---|
| `DocTag` | `type DocTag= [u8` | — |
| `Result` | `type Result<T> = std::result::Result<T, CryptoError>` | Result type alias for cryptographic operations in `contextra-crypto`. |
| `Salt` | `type Salt= [u8` | — |
| `SpillHandler` | `type SpillHandler= Arc<dyn Fn(TenantId, u64, Vec<u8>) + Send + Sync>` | Optionaler Callback-Hook für Tier-2-LSM-Spill bei Eviction aus dem In-Memory LRU Cache. |

### Konstanten (20)

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
| `DEK_LABEL` | `&[u8]` | — |
| `DEK_SALT_LABEL` | `&[u8]` | — |
| `DELETION_PROOF_KEY_PURPOSE` | `&[u8]` | Domain separation tag for deletion proof key derivation. |
| `KDF_HEADER_MAGIC` | `&[u8` | Magischer Header-Marker "MFKD" (Contextra Key Derivation) |
| `KDF_HEADER_VERSION_1` | `u8` | Aktuelle KDF Header Version |
| `KDF_ID_ARGON2ID` | `u8` | Identifikator für Argon2id |
| `MIN_M_COST_KIB` | `u32` | OWASP Mindestanforderungen für Argon2id |
| `MIN_P_COST` | `u32` | Minimaler Parallelitätsgrad für Argon2id gemäß OWASP. |
| `MIN_SALT_LEN` | `usize` | Min. empfohlener Salt-Umfang (16 Bytes) |
| `MIN_T_COST` | `u32` | Minimale Anzahl an Iterationen (Zeitkosten) für Argon2id gemäß OWASP. |
| `REC_AAD` | `&[u8]` | — |
| `WRAP_AAD` | `&[u8]` | — |

### Öffentliche Funktionen (197)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn open_record(dek: &Dek, key_bytes: &[u8], env: &[u8]) -> Result<Zeroizing<Vec<u8>>>` | Opens an encrypted record envelope, verifying the associated data and authentication tag. Fails with a uniform `CryptoError::Aead` on any a… | `src/aead.rs` |
| `pub fn seal_record(dek: &Dek, doc: [u8; 16], key_bytes: &[u8], plain: &[u8]) -> Vec<u8>` | Seals a record using OsRng (convenience wrapper). | `src/aead.rs` |
| `pub fn seal_record_with_nonce( dek: &Dek, doc: [u8; 16], key_bytes: &[u8], plain: &[u8], nonce: [u8; 12], ) -> Result<Vec<u8>>` | Seals a plaintext record into an envelope using an explicit, caller-provided 12-byte nonce. Binds `doc` and `key_bytes` into the Associated… | `src/aead.rs` |
| `pub fn seal_record_with_rng<R: RngCore>( dek: &Dek, doc: [u8; 16], key_bytes: &[u8], plain: &[u8], rng: &mut R, ) -> Result<Vec<u8>>` | Seals a plaintext record into an envelope using an injected RNG for nonce generation. | `src/aead.rs` |
| `pub fn as_bytes(&self) -> &[u8]` | Accessor for cryptographic operations. | `src/anti_tamper.rs` |
| `pub fn constant_time_compare(a: &[u8], b: &[u8]) -> bool` | Konstantzeit-Verifikation zweier Byte-Slices (`INV-CRYPTO-CONSTANT-TIME-1`). Verhindert Timing-Orakel bei MAC-, Tag- und Prüfsummen-Validie… | `src/anti_tamper.rs` |
| `pub fn current_hash(&self) -> &[u8` | Gibt den aktuellen Ketten-Hash zurück. | `src/anti_tamper.rs` |
| `pub fn emergency_wipe(&mut self)` | Emergency Trigger: Explicitly wipes the key from memory. This is to be hooked into hardware sensors (e.g. crash/voltage). | `src/anti_tamper.rs` |
| `pub fn extend(&mut self, context: &[u8], data: &[u8]) -> [u8` | Aktualisiert den Zustand der Kette mit einem neuen Block und Kontext in AAD-Disziplin. | `src/anti_tamper.rs` |
| `pub fn inspect_key_bytes_for_test(&self) -> &[u8` | Test-only method to inspect key bytes. | `src/anti_tamper.rs` |
| `pub fn new(raw: [u8; 32]) -> Self` | Creates a new volatile key from a raw 32-byte array. | `src/anti_tamper.rs` |
| `pub fn new(genesis: [u8; 32]) -> Self` | Initialisiert eine neue BLAKE3-Kette mit einem Seed/Genesis-Hash. | `src/anti_tamper.rs` |
| `pub fn step_count(&self) -> u64` | Gibt die bisherige Anzahl an Kettengliedern zurück. | `src/anti_tamper.rs` |
| `pub fn verify_current_hash(&self, expected: &[u8; 32]) -> Result<()>` | Verifiziert in konstanter Zeit, ob der aktuelle Hash mit einem erwarteten Wert übereinstimmt. | `src/anti_tamper.rs` |
| `pub fn verify_tag_constant_time(expected: &[u8], actual: &[u8]) -> Result<()>` | Verifiziert ein Authentifizierungs-Tag in konstanter Zeit und liefert `Result<()>`. | `src/anti_tamper.rs` |
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
| `pub fn derive_dek(unit: &UnitKey, a: &AncestorSalts, doc: DocTag) -> Dek` | Derives a DEK using AncestorSalts and DocTag. | `src/dek.rs` |
| `pub fn expose_secret(&self) -> &[u8` | — | `src/dek.rs` |
| `pub fn expose_secret(&self) -> &[u8` | — | `src/dek.rs` |
| `pub fn new(bytes: [u8; 32]) -> Self` | — | `src/dek.rs` |
| `pub fn new(bytes: [u8; 32]) -> Self` | — | `src/dek.rs` |
| `pub fn unwrap_dek(kek: &UnitKey, wrapped: &[u8]) -> Result<Dek>` | Unwraps a wrapped DEK using a Key Encryption Key (KEK / UnitKey). | `src/dek.rs` |
| `pub fn wrap_dek(kek: &UnitKey, dek: &Dek, nonce: [u8; 12]) -> Result<Vec<u8>>` | Wraps a DEK using a Key Encryption Key (KEK / UnitKey) and AES-256-GCM-SIV. | `src/dek.rs` |
| `pub fn verify_proof_sequence(proofs: &[DeletionProof]) -> Result<bool>` | Helper functions for verifying deletion proof chains and sequence continuity. | `src/deletion_proof/chain.rs` |
| `pub fn compute_wal_delete_receipt( prev_hmac: &[u8; 32], delete_event_payload: &[u8], integrity_key: &[u8], ) -> Result<[u8` | Erzeugt eine WAL-Auditquittung H(hmac_prev || delete_event) für den Nachweis auf WAL-Ebene für kryptografische Konsistenz. | `src/deletion_proof/creation.rs` |
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
| `pub fn sign_deletion_proof_v3(keypair: &DeletionProofKeyPair, proof_payload: &[u8]) -> [u8` | Signiert den kanonischen Byte-Payload eines Version-3-Attestationsbeweises mit Ed25519. Gibt eine 64-Byte Ed25519-Signatur zurück. | `src/ed25519_proof.rs` |
| `pub fn signing_key(&self) -> &ed25519_dalek::SigningKey` | Gibt eine Referenz auf den Signierschlüssel zurück. | `src/ed25519_proof.rs` |
| `pub fn verify_deletion_proof_v3( verifying_key_bytes: &[u8; 32], proof_payload: &[u8], signature: &[u8; 64], ) -> Result<(), DeletionProofError>` | Verifiziert eine Ed25519-Signatur eines Version-3-Attestationsbeweises rein funktional. Diese Funktion benötigt keine Contextra-internen Zu… | `src/ed25519_proof.rs` |
| `pub fn verifying_key(&self) -> &ed25519_dalek::VerifyingKey` | Gibt eine Referenz auf den Verifikationsschlüssel zurück. | `src/ed25519_proof.rs` |
| `pub fn verifying_key_bytes(&self) -> [u8` | Gibt die 32-Byte-Repräsentation des öffentlichen Verifikationsschlüssels zurück. | `src/ed25519_proof.rs` |
| `pub fn wal_corruption(offset: u64, reason: impl Into<String>) -> Self` | Helper to construct a `WalCorruption` error. | `src/error.rs` |
| `pub fn derive_key_argon2id(passphrase: &str, header: &KdfHeader) -> Result<DerivedKey>` | Leitet einen kryptographisch starken Schlüssel mittels Argon2id aus einer Passphrase ab. # Ablauf der Argon2id-Schlüsselableitung: 1. Passp… | `src/kdf.rs` |
| `pub fn from_bytes(bytes: &[u8]) -> Result<Self>` | Parst einen `KdfHeader` aus Binärdaten. Panic-Freiheit (total function) bei beliebigen Byte-Eingaben. | `src/kdf.rs` |
| `pub fn generate_default() -> Result<Self>` | Erstellt einen neuen `KdfHeader` v1 mit zufälligem 32-Byte-Salt und OWASP-Standardparametern. | `src/kdf.rs` |
| `pub fn hkdf_expand(prk: &[u8], info: &[u8], out: &mut [u8]) -> Result<()>` | HKDF-SHA256 expand. | `src/kdf.rs` |
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
| `pub fn emergency_wipe(store: &TenantIsolatedKvStore)` | SEPARATER Pfad für Notfall-Bereinigung (z.B. Prozess-Shutdown, expliziter Sicherheits-Trigger). Synchron, blockierend, vor Rückkehr abgesch… | `src/kv_segment/eviction_worker.rs` |
| `pub fn shutdown(&self)` | Beendet den Worker-Thread geordnet. | `src/kv_segment/eviction_worker.rs` |
| `pub fn spawn(store: Arc<TenantIsolatedKvStore>) -> Self` | Spawnt den Eviction-Worker auf einem dedizierten OS-Thread. | `src/kv_segment/eviction_worker.rs` |
| `pub fn trigger_eviction(&self, target_free_bytes: usize)` | Nicht-blockierender Trigger vom Hot-Path aus. | `src/kv_segment/eviction_worker.rs` |
| `pub fn delete_mode(&self) -> KvDeleteMode` | Returns the current delete mode. | `src/kv_segment/manager.rs` |
| `pub fn delete_segment(&self, group_id: u64) -> Result<bool, CryptoError>` | Deletes a KV segment. | `src/kv_segment/manager.rs` |
| `pub fn generate_deletion_proof(&self, group_id: u64) -> Result<bool, CryptoError>` | Generates or validates a KV deletion proof. | `src/kv_segment/manager.rs` |
| `pub fn new( config: KvSegmentConfig, registry: Arc<KeyRegistry>, master_key: Option<Arc<KeyManager>>, ) -> Self` | Creates a new with specified config, key registry, and master key. | `src/kv_segment/manager.rs` |
| `pub fn read_segment(&self, segment: &KvSegmentPayload) -> Result<Vec<u8>, CryptoError>` | Reads/decrypts a KV segment payload. | `src/kv_segment/manager.rs` |
| `pub fn write_segment( &self, group_id: u64, plaintext: &[u8], ) -> Result<KvSegmentPayload, CryptoError>` | Writes a KV segment payload according to the configured . | `src/kv_segment/manager.rs` |
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
| `pub fn as_bytes(&self) -> &[u8` | Exposes a reference to the secret bytes (compatibility alias). | `src/secret.rs` |
| `pub fn expose_secret(&self) -> &[u8` | Exposes a reference to the secret bytes. | `src/secret.rs` |
| `pub fn expose_secret(&self) -> &T` | Exposes a reference to the inner secret. | `src/secret.rs` |
| `pub fn expose_secret_mut(&mut self) -> &mut T` | Exposes a mutable reference to the inner secret. | `src/secret.rs` |
| `pub fn new(bytes: [u8; N]) -> Self` | Creates a new secret key wrapping raw bytes. | `src/secret.rs` |
| `pub fn new(inner: T) -> Self` | Creates a new `Secret` wrapping sensitive data. | `src/secret.rs` |
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
| **Interne Abhängigkeiten** | `contextra-mvcc`, `contextra-ports`, `contextra-sys`, `contextra-types` |
| **Externe Abhängigkeiten (Auswahl)** | `ahash`, `blake3`, `memmap2`, `parking_lot`, `rand`, `roaring`, `serde`, `subtle`, `thiserror`, `tracing`, `zeroize` |

### Architekturbeschreibung

Contextra Index — HNSW vector index with SIMD distance computation.

### Modul- und Re-Export-Oberfläche (lib.rs)

```rust
pub mod casting;
pub mod contract;
pub mod error;
pub mod types;
pub mod diskann;
pub mod distance;
pub mod hnsw;
pub mod persistence;
pub mod quantize;
pub mod acorn;
pub mod candidate_stream;
pub mod compute_pool;
pub mod fixed_hasher;
pub mod partial_rebuild;
pub mod quantize_rabitq;
pub use candidate_stream::{VectorCandidateStream, DEFAULT_VECTOR_STREAM_BATCH_SIZE};
pub use compute_pool::ComputePool;
pub use contract::{ BlockTransform, Clock as ContractClock, DistanceFn, Error as ContractError, ExternalId, Index as ContractIndex, Maintain as ContractMaintain, Metric as ContractMetric, Result as ContractResult, Rng as ContractRng, ScoredPoint, };
pub use diskann::{DiskAnnConfig, DiskAnnFallbackPolicy, DiskAnnIndex};
pub use hnsw::{ hnsw_purger, hnsw_scanner, HnswConfig, HnswIndex, HnswPurger, HnswScanner, RebuildStatus, };
pub use persistence::{HnswHeader, MmapIndex};
pub use quantize::ScalarQuantizer;
pub use quantize_rabitq::RaBitQQuantizer;
pub use types::{Clock, DistanceMetric, DocId, Maintain, MetricsSink, Rng, ScoredDocument, TxId};
```

### Feature-Flags

| Flag | Aktiviert |
|---|---|
| `default` | `["std", "ports-compat"]` |
| `std` | `[]` |
| `serde` | `[]` |
| `tracing` | `[]` |
| `inspect` | `[]` |
| `experimental` | `["experimental-diskann", "experimental-rabitq", "experimental-predicate-augmented-search"]` |
| `mmap` | `[]` |
| `diskann` | `["experimental-diskann"]` |
| `quantize-rabitq` | `["experimental-rabitq"]` |
| `docid-128` | `[]` |
| `experimental-diskann` | `[]` |
| `experimental-rabitq` | `[]` |
| `experimental-predicate-augmented-search` | `[]` |
| `graph` | `[]` |
| `partial-index-rebuild` | `[]` |

### Schnittstellen — Public Traits (15)

**`BlockTransform`** — *Generischer Transformations-Hook (z. B. für optionale Verschlüsselung/Transformation).* (`src/contract.rs`)

```rust
pub trait BlockTransform : Send + Sync {
/// Transformiert einen Datenblock vor der Persistierung.
    fn transform(&self, block: &mut [u8]) -> Result<()>;
    /// Rekonstruiert einen transformierten Datenblock beim Lesen.
    fn inverse_transform(&self, block: &mut [u8]) -> Result<()>;
}
```

**`Clock`** — *Trait zur Zeitabfrage.* (`src/contract.rs`)

```rust
pub trait Clock : Send + Sync {
/// Liefert die aktuelle Zeit in Unix-Millisekunden.
    fn now_unix_ms(&self) -> u64;
}
```

**`Clock`** — *Trait für Zeitabfragen ohne Abhängigkeit von `contextra-ports`.* (`src/types.rs`)

```rust
pub trait Clock : Send + Sync {
/// Liefert die aktuelle Zeit in Unix-Nanosekunden.
    fn now_unix_nanos(&self) -> u64;
    /// Liefert einen monoton steigenden Nanosekunden-Zähler.
    fn monotonic_nanos(&self) -> u64;
}
```

**`DistanceFn`** — *Distanzfunktions-Schnittstelle zur externen Injektion von Berechnungen (z.B. SIMD).* (`src/contract.rs`)

```rust
pub trait DistanceFn : Send + Sync {
/// Berechnet die Distanz zwischen zwei gleich langen Vektoren.
    fn distance(&self, a: &[f32], b: &[f32]) -> f32;
}
```

**`DistanceKernel`** — *Distance kernel interface defining vector metrics and quantized distance operations.* (`src/distance.rs`)

```rust
pub trait DistanceKernel : Send + Sync + 'static {
/// Computes distance between two slices after validating finiteness.
    fn distance(&self, metric: DistanceMetric, a: &[f32], b: &[f32]) -> Result<f32>;

    /// Computes distance between an f32 query and little-endian f32 byte slice after validating finiteness and alignment.
    fn distance_f32_bytes(&self, metric: DistanceMetric, a: &[f32], b_bytes: &[u8]) -> Result<f32>;

    /// Computes distance between pre-validated f32 slices.
    fn distance_trusted(&self, metric: DistanceMetric, a: &[f32], b: &[f32]) -> Result<f32>;

    /// Computes distance between pre-validated f32 slice and LE f32 byte slice.
    fn distance_f32_bytes_trusted(
        &self,
        metric: DistanceMetric,
        a: &[f32],
        b_bytes: &[u8],
    ) -> Result<f32>;

    /// Computes cosine distance between two f32 slices.
    fn cosine_distance(&self, a: &[f32], b: &[f32]) -> Result<f32>;

    /// Computes dot product distance (negative dot product) between two f32 slices.
    fn dot_product_distance(&self, a: &[f32], b: &[f32]) -> Result<f32>;

    /// Computes Euclidean distance between two f32 slices.
    fn euclidean_distance(&self, a: &[f32], b: &[f32]) -> Result<f32>;

    /// Computes squared Euclidean distance between two u8 slices.
    fn euclidean_distance_sq_u8(&self, a: &[u8], b: &[u8]) -> u32;

    /// Computes mixed f32/u8 squared Euclidean distance with dequantization.
    fn euclidean_distance_sq_f32_u8(
        &self,
        a: &[f32],
        b: &[u8],
        alphas: &[f32],
        mins: &[f32],
    ) -> f32;

    /// Computes unsigned integer dot product between two u8 byte slices.
    fn dot_product_u8(&self, a: &[u8], b: &[u8]) -> u32;

    /// Computes mixed f32/u8 dot product.
    fn dot_product_f32_u8(&self, a: &[f32], b: &[u8]) -> f32;

    /// Computes raw component parts for u8 cosine similarity.
    fn cosine_similarity_parts_u8(&self, a: &[u8], b: &[u8]) -> CosineSimilarityPartsU8;

    /// Normalizes a vector in-place to unit length.
    fn normalize_inplace(&self, v: &mut [f32]);
}
```

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

**`Index`** — *Standardisierter Index-Trait für Klasse A Bausteine (SPEC §4.3).* (`src/contract.rs`)

```rust
pub trait Index<Id, Query> {
/// Typ eines einzelnen Suchtreffers.
    type Hit;

    /// Fügt einen Vektordatenpunkt ein.
    fn insert(&mut self, id: Id, payload: &[u8]) -> Result<()>;

    /// Entfernt einen Vektordatenpunkt anhand seiner ID.
    fn remove(&mut self, id: &Id) -> Result<bool>;

    /// Sucht die nächsten Nachbarn für eine gegebene Abfrage.
    fn search(&self, query: &Query, limit: usize) -> Result<Vec<Self::Hit>>;
}
```

**`Index`** — *Normative Index-Schnittstelle gemäß SPEC §4.3 (Klasse A).* (`src/hnsw/vector_index_impl.rs`)

```rust
pub trait Index<Id, Query: ?Sized> {
/// Treffertyp der Suchoperation.
    type Hit;

    /// Fügt ein Dokument mit gegebener ID und Byte-Nutzlast in den Index ein.
    fn insert(&mut self, id: Id, payload: &[u8]) -> ContractResult<()>;

    /// Entfernt ein Dokument anhand seiner ID aus dem Index.
    fn remove(&mut self, id: &Id) -> ContractResult<bool>;

    /// Führt eine Suche anhand der Query aus und liefert bis zu `limit` Treffer zurück.
    fn search(&self, query: &Query, limit: usize) -> ContractResult<Vec<Self::Hit>>;
}
```

**`Maintain`** — *Neutraler Wartungs-Trait für Klasse A Bausteine (SPEC §4.3).* (`src/contract.rs`)

```rust
pub trait Maintain {
/// Typ der Wartungsstatistiken.
    type Stats;

    /// Kompaktiert den Index und gibt freigegebene Ressourcen zurück.
    fn compact(&mut self) -> Result<Self::Stats>;

    /// Führt physische Bereinigungen durch.
    fn vacuum(&mut self) -> Result<()>;
}
```

**`Maintain`** — *Neutraler Wartungs-Trait für Speicherfreigaben und Kompaktierung (SPEC §6.1).* (`src/types.rs`)

```rust
pub trait Maintain : Send + Sync {
/// Führt physische Bereinigung und Slot-Freigaben durch.
    fn compact(&self) -> crate::error::Result<()>;
    /// Prüft den Rebuild- bzw. Reorganisationsbedarf.
    fn is_rebuild_required(&self) -> bool;
}
```

**`Maintain`** — *Normative Wartungsschnittstelle gemäß SPEC §4.3 (Klasse A).* (`src/hnsw/vector_index_impl.rs`)

```rust
pub trait Maintain {
/// Statistiktyp der Kompaktierungsoperation.
    type Stats;

    /// Führt eine Kompaktierung durch und gibt gelöschte Einträge tatsächlich frei.
    fn compact(&mut self) -> ContractResult<Self::Stats>;

    /// Bereinigt fragmentierte interne Strukturen.
    fn vacuum(&mut self) -> ContractResult<()>;
}
```

**`MetricsSink`** — *Trait zur Metrik-Erfassung.* (`src/types.rs`)

```rust
pub trait MetricsSink : Send + Sync {
/// Zählt ein Ereignis hoch.
    fn record_counter(
        &self,
        name: &'static str,
        value: u64,
        labels: &[(&'static str, &'static str)],
    );
    /// Zeichnet eine Histogramm-Messung auf.
    fn record_histogram(
        &self,
        name: &'static str,
        value: f64,
        labels: &[(&'static str, &'static str)],
    );
}
```

**`Rng`** — *Trait für deterministischen Zufall.* (`src/contract.rs`)

```rust
pub trait Rng : Send {
/// Liefert das nächste Pseudozufalls-`u64`.
    fn next_u64(&mut self) -> u64;
}
```

**`Rng`** — *Trait für deterministische Zufallszahlen im Vektorindex.* (`src/types.rs`)

```rust
pub trait Rng : Send + Sync {
/// Liefert ein zufälliges `u64`.
    fn next_u64(&self) -> u64;
    /// Liefert ein gleichverteiltes `f64` im Intervall `[0.0, 1.0)`.
    fn next_f64(&self) -> f64;
}
```

### Öffentliche Datentypen — Structs (53)

| Struct | Felder / Form | Beschreibung | Datei |
|---|---|---|---|
| `AdaptiveEfPolicy` | `{ /// Initial search depth ef for layer 0. pub min_ef: usize, /// Maximum search depth ef limit. pub max_ef: usize, /// Growth factor for increasing ef in each e… }` | Configuration policy for adaptive ef_search (Ada-ef). | `src/hnsw/adaptive_ef.rs` |
| `AdaptiveEfStateMachine` | `{ policy: AdaptiveEfPolicy, k: usize, current_ef: usize, previous_top_k: Vec<DocId>, stable_rounds: usize, round_count: usize, is_terminated: bool, converged: bo… }` | Deterministic state machine managing Ada-ef expansion rounds. | `src/hnsw/adaptive_ef.rs` |
| `AdaptiveEfStats` | `{ /// Final ef depth reached at termination. pub final_ef: usize, /// Total expansion rounds executed. pub rounds: usize, /// Whether search terminated due to ca… }` | Execution statistics for an adaptive search run. | `src/hnsw/adaptive_ef.rs` |
| `BacklinkTable` | `{ pub map: BTreeMap<BacklinkKey, Vec<u32>>, }` | Deterministic backlink connection table using `BTreeMap`. | `src/hnsw/arena.rs` |
| `BatchContext` | `{ pub running_max_layer: usize, pub has_entry_point: bool, pub has_ram_entry_point: bool, pub backlink_map: BacklinkTable, }` | Batch tracking context for running state across multi-operation transaction commits. | `src/hnsw/batch.rs` |
| `Candidate` | `{ pub index: usize, pub distance: f32, }` | Search candidate. | `src/hnsw/types.rs` |
| `ComputePool` | `{ inner: Arc<PoolState>, }` | Bounded compute thread pool for background index rebuilds and heavy CPU tasks. Enforces a hard ceiling on concurrent worker threads to prevent thread… | `src/compute_pool.rs` |
| `CosineSimilarityPartsU8` | `{ /// Accumulated dot product $\sum a_i b_i$. pub dot: u32, /// Accumulated squared L2 norm $\sum a_i^2$. pub norm_a_sq: u32, /// Accumulated squared L2 norm $\s… }` | Raw integer sum components for u8 cosine similarity calculation. | `src/distance.rs` |
| `DeletionStats` | `{ /// Document ID that was deleted. pub doc_id: DocId, /// Number of neighborhood edges repaired (backlinks removed/replaced). pub repaired_edges: usize, /// Num… }` | Statistics returned after a ghost-free node deletion and neighborhood repair. | `src/hnsw/deletion.rs` |
| `DiskAnnConfig` | `{ /// Path to the on-disk index file. pub index_path: PathBuf, /// Vector dimension. pub dimension: usize, /// Maximum graph degree (R). pub max_degree: usize, /… }` | Configuration for DiskANN index. | `src/diskann/config.rs` |
| `DiskAnnHmac` | `{ hasher: blake3::Hasher, }` | Standalone keyed BLAKE3 hasher for DiskANN file integrity. | `src/diskann/integrity.rs` |
| `DiskAnnIndex` | `{ pub(crate) inner: Arc<DiskAnnIndexInner>, }` | DiskANN out-of-core vector index. This index uses `memmap2` to offload vector data and graph edges to disk. Note that mmap operations and page faults… | `src/diskann/types.rs` |
| `DocId` | `(tuple/unit)` | — | `src/types.rs` |
| `DocId` | `(tuple/unit)` | — | `src/types.rs` |
| `ExternalId` | `(tuple/unit)` | Externe Identifikationsnummer (128-Bit für breite Kompatibilität). | `src/contract.rs` |
| `GenericHnswIndex` | `{ pub(super) inner: std::sync::Arc<HnswIndexCore>, pub(super) _kernel: std::marker::PhantomData<K>, }` | Generic HNSW (Hierarchical Navigable Small World) vector index parameterized by distance kernel. | `src/hnsw/types.rs` |
| `HnswArena` | `{ pub arena: RwLock<Vec<u32>>, pub offsets: RwLock<Vec<usize>>, pub capacities: RwLock<Vec<usize>>, pub count_offsets: RwLock<Vec<usize>>, pub counts: RwLock<Vec… }` | Contiguous 64-byte aligned arena allocator for HNSW node connections with free-list slot reuse. | `src/hnsw/arena.rs` |
| `HnswColdCore` | `{ pub config: HnswConfig, pub validation_error: Option<String>, pub tx_buffer: super::staging::VectorTxBuffer, pub quantizer: RwLock<Option<crate::quantize::Scal… }` | — | `src/hnsw/types.rs` |
| `HnswCompactStats` | `{ /// Anzahl physisch bereinigter Tombstones. pub purged_docs: usize, /// Anzahl verbliebener aktiver Dokumente. pub active_docs: usize, }` | Statistiken über den Kompaktierungsvorgang des HNSW-Vektorindex. | `src/hnsw/vector_index_impl.rs` |
| `HnswConfig` | `{ /// Vector dimensionality. pub dimension: usize, /// Maximum number of elements. pub max_elements: usize, /// Number of connections per element (M parameter).… }` | — | `src/hnsw/config.rs` |
| `HnswConfigBuilder` | `{ config: HnswConfig, }` | Builder for HnswConfig with resource limit enforcements to prevent OOM. | `src/hnsw/config.rs` |
| `HnswHeader` | `{ magic: u32, version: u16, dimension: u32, m: u32, metric: u8, quantized: u8, q_min: f32, // Legacy v1 field q_max: f32, // Legacy v1 field node_count: u64, ent… }` | The header of an HNSW persistent file. | `src/persistence/header.rs` |
| `HnswHotCore` | `{ pub nodes: RwLock<Vec<HnswNode>>, #[cfg(not(feature = "docid-128"))] pub doc_to_node: RwLock<BTreeMap<u64, usize>>, #[cfg(feature = "docid-128")] pub doc_to_no… }` | — | `src/hnsw/types.rs` |
| `HnswIndexCore` | `{ pub hot: HnswHotCore, pub cold: HnswColdCore, }` | The core implementation of the HNSW index. | `src/hnsw/types.rs` |
| `HnswNode` | `{ pub(super) doc_id: DocId, pub(super) vector: VectorData, pub(super) max_layer: usize, pub(super) committed_tx: u64, }` | A node in the HNSW graph. | `src/hnsw/types.rs` |
| `HnswPurger` | `{ index: Arc<HnswIndex>, }` | — | `src/hnsw/purge.rs` |
| `HnswScanner` | `{ index: Arc<HnswIndex>, }` | — | `src/hnsw/scanner.rs` |
| `InjectedDistanceKernel` | `{ pub distance_fn: D, }` | Dynamic adapter enabling any injected [`DistanceFn`] to be used as a [`DistanceKernel`]. | `src/distance.rs` |
| `MmapIndex` | `{ pub mmap: std::sync::Arc<memmap2::Mmap>, pub header: HnswHeader, pub path: Option<std::path::PathBuf>, }` | A reader for memory-mapped HNSW indices. | `src/persistence/mmap.rs` |
| `MonotonicCounterClock` | `{ nanos: AtomicU64, }` | Deterministic monotonic counter clock for HNSW vector index. Implements [`Clock`] without any reference to system wall-clock or non-deterministic OS… | `src/hnsw/clock.rs` |
| `NaiveReferenceIndex` | `{ vectors: Vec<(DocId, Vec<f32>)>, metric: DistanceMetric, }` | Naive reference index performing brute-force search over all stored points. **NICHT für Produktion — O(n) Referenzimplementierung** Serves exclusivel… | `src/acorn/naive_reference.rs` |
| `NeighborBacklink` | `{ pub neighbor_ram_idx: usize, pub layer: usize, pub updated_connections: Vec<u32>, }` | Back-link connection update for a neighbor node at a specific layer. | `src/hnsw/batch.rs` |
| `NodeRecord` | `{ #[cfg(not(feature = "docid-128"))] pub doc_id: u64, #[cfg(feature = "docid-128")] pub doc_id: u128, pub max_layer: u8, pub vector_offset: u64, pub connections_… }` | Represents a node's metadata in the flat file. | `src/persistence/node.rs` |
| `NoopMetricsSink` | `(tuple/unit)` | No-Op Metriksink für Standardbetrieb ohne Telemetrie. | `src/types.rs` |
| `PartialRebuildConfig` | `{ /// Kritische Übersättigungs-Schwelle θ_c. Default: 3.0 (3× globale Dichte). pub critical_ratio: f32, /// Ringpuffer-Größe N für Traversal-Statistik. Default:… }` | Konfiguration für den Partial-Rebuild-Trigger. | `src/partial_rebuild.rs` |
| `PreparedInsert` | `{ pub doc_id: DocId, pub vector_data: VectorData, pub new_layer: usize, pub new_idx: usize, pub final_connections: Vec<Vec<u32>>, pub neighbor_backlinks: Vec<Nei… }` | — | `src/hnsw/batch.rs` |
| `RaBitQQuantizer` | `{ dimension: usize, /// Row-major orthogonal rotation matrix R of size dimension x dimension. rotation_matrix: Vec<f32>, }` | RaBitQ Quantizer with random orthogonal rotation and scalar metadata. | `src/quantize_rabitq.rs` |
| `RebuildGuard` | `(tuple/unit)` | — | `src/hnsw/types.rs` |
| `ResidualReport` | `{ /// Ob die ID noch in der ID-zu-Knoten-Zuordnung auffindbar ist. pub in_id_map: bool, /// Ob im zugehörigen Slot noch Vektordaten ungleich Null liegen. pub non… }` | Struktur für die lesende Inspektionssicht (`inspect`-Sicht). Ermittelt, ob Spuren einer Dokument-ID bzw. eines Vektors noch in ID-Mappings, Slots (Ni… | `src/hnsw/scanner.rs` |
| `ScalarDistance` | `{ pub metric: ContractMetric, }` | Crate-local scalar distance function implementing [`DistanceFn`]. | `src/distance.rs` |
| `ScalarKernel` | `(tuple/unit)` | Complete, safe, scalar reference implementation using safe iterators and chunking. | `src/distance.rs` |
| `ScalarQuantizer` | `{ pub(crate) mins: Vec<f32>, pub(crate) maxes: Vec<f32>, pub(crate) scales: Vec<f32>, pub(crate) inv_scales: Vec<f32>, pub(crate) dimension: usize, #[serde(skip,… }` | An 8-bit Scalar Quantizer (SQ8) with per-dimension scaling. Quantization reduces the memory footprint of vector storage by 4x. Per-dimension scaling… | `src/quantize/mod.rs` |
| `ScoredDocument` | `{ /// Dokument-ID des Treffers. pub doc_id: DocId, /// Ähnlichkeitswert (Score, höher ist ähnlicher). pub score: f32, /// Distanzwert (niedriger ist näher). pub… }` | Suchergebnis mit Dokument-ID und Relevanz-Score / Distanz. | `src/types.rs` |
| `ScoredPoint` | `{ /// Externe Identifikationsnummer des Punkts. pub id: ExternalId, /// Ähnlichkeitswert (höher = besser, total geordnet). pub score: f32, }` | Trefferpunkt mit Distanz-/Ähnlichkeitsbewertung. | `src/contract.rs` |
| `SeededRng` | `{ state: std::sync::atomic::AtomicU64, }` | Einfacher deterministischer SplitMix64 RNG. | `src/types.rs` |
| `SnapshotPinGuard` | `(tuple/unit)` | — | `src/hnsw/types.rs` |
| `Sq8Bias` | `{ /// Mean quantization error across distance computations. pub mean_bias: f32, /// Variance of quantization error across distance computations. pub variance_bia… }` | Measured quantization bias and variance statistics for SQ8 distance calculations. | `src/hnsw/sq8_bias.rs` |
| `SystemClock` | `(tuple/unit)` | Standardmäßige Implementierung von [`Clock`] auf Basis von `std::time::SystemTime` / `Instant`. | `src/types.rs` |
| `TaskHandle` | `{ rx: Receiver<T>, }` | Handle to await the outcome of a task submitted to [`ComputePool`]. | `src/compute_pool.rs` |
| `TraversalTracker` | `{ config: PartialRebuildConfig, visited_ring: VecDeque<Vec<u64>>, // Ein Vec<u64> pro Traversierungsaufruf }` | Ringpuffer für besuchte HNSW-Knoten-IDs während ef_search-Traversierungen. | `src/partial_rebuild.rs` |
| `TxId` | `(tuple/unit)` | — | `src/types.rs` |
| `VectorCandidateStream` | `{ index: &'a V, query: Vec<f32>, seq_no: Option<u64>, batch_size: usize, buffer: VecDeque<ScoredDocument>, seen: BTreeSet<DocId>, fetched_k: usize, yielded: usiz… }` | On-demand loading async pull-stream for vector search candidates. # Candidate Ordering & Determinism Vector search scores represent relevance (higher… | `src/candidate_stream.rs` |
| `VectorTxBuffer` | `{ inner: contextra_mvcc::TxBuffer<Vec<f32>>, }` | Transaction staging buffer for vector operations. Encapsulates transactional staging locally, isolating from direct coupling to storage transaction b… | `src/hnsw/staging.rs` |

### Öffentliche Datentypen — Enums (9)

| Enum | Varianten | Beschreibung | Datei |
|---|---|---|---|
| `AcornError` | `/// Dimension mismatch between query and vector database points. DimensionMismatch { /// Expected dimension. expected: usize, /// Provided query dimension. got: usize, }, /// Invalid search parameter…` | Errors occurring during ACORN search or graph traversal operations. | `src/acorn/mod.rs` |
| `DiskAnnFallbackPolicy` | `/// Transparently fall back to in-memory HNSW index on load failure or detected corruption. #[default] UseHnswOnFailure, /// Immediately return a typed error on load failure or index corruption. Fail…` | Fallback behavior policy when DiskANN loading, integrity checks, or reads fail. | `src/diskann/config.rs` |
| `DistanceMetric` | `/// Kosinus-Distanz ($1 - \text{sim}$). #[default] Cosine, /// Euklidische Distanz ($L_2$). Euclidean, /// Skalarprodukt (Dot-Produkt, invertiert für Minimierung). DotProduct,` | — | `src/types.rs` |
| `Error` | `/// Ungültige Eingabedaten. #[error("ungültige Eingabedaten: {0}")] InvalidInput(&'static str), /// Kapazitätsgrenze überschritten. #[error("Kapazitätsgrenze überschritten: gefordert {requested}, max…` | — | `src/contract.rs` |
| `Metric` | `/// Kosinus-Distanz. Cosine, /// Euklidische Distanz. Euclidean, /// Skalarprodukt. DotProduct,` | Unterstützte Distanzmetriken. | `src/contract.rs` |
| `RebuildStatus` | `/// Kein Rebuild läuft oder geplant. Idle, /// Rebuild läuft gerade im Hintergrund. Running, /// Rebuild wurde getriggert, startet bald. Pending,` | — | `src/hnsw/types.rs` |
| `VectorData` | `/// Standard 32-bit floating point vectors. F32(Vec<f32>), /// 8-bit quantized vectors (SQ8). U8(Vec<u8>),` | — | `src/hnsw/types.rs` |
| `VectorError` | `/// Invalid input or configuration parameter. #[error("invalid input: {0}")] InvalidInput(String), /// Index structure corruption or inconsistency. #[error("index error: {0}")] Index(String), /// Dim…` | Error states of `contextra-vector`. | `src/error.rs` |
| `VectorStagingOp` | `/// Insertion or update of a vector for a document. Insert { doc_id: DocId, embedding: Vec<f32> }, /// Deletion of a document vector. Delete { doc_id: DocId },` | Operation within a transactional staging buffer. | `src/hnsw/staging.rs` |

### Typ-Aliase (6)

| Alias | Definition | Beschreibung |
|---|---|---|
| `BacklinkKey` | `type BacklinkKey= (usize, usize)` | Composite key (neighbor_ram_idx, layer) for backlink lookups. |
| `FixedHashMap` | `type FixedHashMap<K, V> = HashMap<K, V, RandomState>` | A deterministic `HashMap` using constant seeds. |
| `FixedHashSet` | `type FixedHashSet<T> = HashSet<T, RandomState>` | A deterministic `HashSet` using constant seeds. |
| `HnswIndex` | `type HnswIndex= GenericHnswIndex<ScalarKernel>` | The default HNSW vector index using the scalar reference distance kernel. |
| `Result` | `type Result<T> = core::result::Result<T, Error>` | Standard-Result-Typ für Vertragsoperationen. |
| `Result` | `type Result<T> = std::result::Result<T, VectorError>` | Standard Result type for `contextra-vector`. |

### Konstanten (18)

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
| `MAX_SEARCH_K` | `usize` | Maximal zulässiges `k` für Nachbarschaftssuchen. |
| `MIN` | `Self` | Kleinstmöglicher Bezeichner. |
| `SENTINEL_NO_ENTRY_POINT` | `u32` | Sentinel-Wert für ungültigen / nicht gesetzten Entry-Point in HNSW. |
| `SIZE` | `usize` | — |
| `SIZE` | `usize` | — |
| `SIZE` | `usize` | — |
| `SQ8_MAX_ERROR_FACTOR_PER_DIM` | `f32` | Theoretical maximum quantization error bound factor per dimension: $1 / (2 \cdot 255) = 1 / 510$. |
| `SYSTEM` | `Self` | Transaktion 0 (System/Baseline). |
| `TOMBSTONE_BIT` | `u64` | Bitmaske für Tombstone-Markierung in Sequenznummern. |

### Öffentliche Funktionen (261)

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
| `pub fn f32_to_usize(val: f32) -> Result<usize>` | Converts `f32` to `usize` defensively with bounds validation. | `src/casting.rs` |
| `pub fn f64_to_usize(val: f64) -> Result<usize>` | Converts `f64` to `usize` defensively with bounds validation. | `src/casting.rs` |
| `pub fn to_u32(val: usize) -> Result<u32>` | Converts `usize` to `u32`, returning `ContextraError::invalid_input` if out of bounds. | `src/casting.rs` |
| `pub fn to_u64(val: usize) -> u64` | Converts `usize` to `u64`. Infallible on 32-bit and 64-bit platforms. | `src/casting.rs` |
| `pub fn to_u8(val: usize) -> Result<u8>` | Converts `usize` to `u8`, returning `ContextraError::invalid_input` if out of bounds. | `src/casting.rs` |
| `pub fn u32_to_usize(val: u32) -> usize` | Converts `u32` to `usize`. On supported architectures (32-bit and 64-bit), this is infallible. | `src/casting.rs` |
| `pub fn u64_to_u32(val: u64) -> Result<u32>` | Converts `u64` to `u32`, returning `ContextraError::invalid_input` if out of bounds. | `src/casting.rs` |
| `pub fn u64_to_usize(val: u64) -> Result<usize>` | Converts `u64` to `usize`, returning `ContextraError::invalid_input` if out of bounds. | `src/casting.rs` |
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
| `pub fn finalize(self) -> [u8` | Finalizes the calculation and yields the 32-byte digest. | `src/diskann/integrity.rs` |
| `pub fn new(integrity_key: &[u8]) -> Result<Self>` | Creates a new `DiskAnnHmac` using the provided integrity key. | `src/diskann/integrity.rs` |
| `pub fn update(&mut self, data: &[u8])` | Feeds new data into the integrity calculation. | `src/diskann/integrity.rs` |
| `pub async fn load(&self) -> Result<()>` | Loads the index from the configured path. | `src/diskann/persistence.rs` |
| `pub fn load_sync(&self) -> Result<()>` | — | `src/diskann/persistence.rs` |
| `pub async fn persist_delta(&self) -> Result<()>` | — | `src/diskann/persistence.rs` |
| `pub fn persist_delta_sync(&self) -> Result<()>` | — | `src/diskann/persistence.rs` |
| `pub fn trigger_background_persist_delta(&self)` | — | `src/diskann/persistence.rs` |
| `pub async fn search_predicate_augmented( &self, query: &[f32], k: usize, filter: &(dyn Fn(DocId) -> bool + Send + Sync), gamma: usize, ) -> Result<Vec<ScoredDocument>>` | Executes a predicate-augmented vector search using ACORN 2-hop neighbor expansion during graph traversal. # Parameters - `query`: Query vec… | `src/diskann/predicate_augmented.rs` |
| `pub async fn search_internal(&self, query: &[f32], k: usize) -> Result<Vec<ScoredDocument>>` | — | `src/diskann/search.rs` |
| `pub fn compute(&self, a: &[f32], b: &[f32]) -> core::result::Result<f32, ContractError>` | Computes distance with strict validation of finiteness and dimension match. | `src/distance.rs` |
| `pub fn compute_distance(a: &[f32], b: &[f32], metric: DistanceMetric) -> Result<f32>` | — | `src/distance.rs` |
| `pub fn compute_distance_f32_bytes_trusted( a: &[f32], b_bytes: &[u8], metric: DistanceMetric, ) -> Result<f32>` | — | `src/distance.rs` |
| `pub fn compute_distance_trusted(a: &[f32], b: &[f32], metric: DistanceMetric) -> Result<f32>` | — | `src/distance.rs` |
| `pub fn cosine_distance(a: &[f32], b: &[f32]) -> Result<f32>` | — | `src/distance.rs` |
| `pub fn cosine_distance_scalar(a: &[f32], b: &[f32]) -> f32` | — | `src/distance.rs` |
| `pub fn cosine_similarity_parts_u8(a: &[u8], b: &[u8]) -> CosineSimilarityPartsU8` | — | `src/distance.rs` |
| `pub fn dot_product_distance(a: &[f32], b: &[f32]) -> Result<f32>` | — | `src/distance.rs` |
| `pub fn dot_product_f32_u8(a: &[f32], b: &[u8]) -> f32` | — | `src/distance.rs` |
| `pub fn dot_product_scalar(a: &[f32], b: &[f32]) -> f32` | — | `src/distance.rs` |
| `pub fn dot_product_u8(a: &[u8], b: &[u8]) -> u32` | — | `src/distance.rs` |
| `pub fn euclidean_distance(a: &[f32], b: &[f32]) -> Result<f32>` | — | `src/distance.rs` |
| `pub fn euclidean_distance_scalar(a: &[f32], b: &[f32]) -> f32` | — | `src/distance.rs` |
| `pub fn euclidean_distance_sq_f32_u8(a: &[f32], b: &[u8], alphas: &[f32], mins: &[f32]) -> f32` | — | `src/distance.rs` |
| `pub fn euclidean_distance_sq_u8(a: &[u8], b: &[u8]) -> u32` | — | `src/distance.rs` |
| `pub fn new(distance_fn: D) -> Self` | — | `src/distance.rs` |
| `pub fn normalize_inplace(v: &mut [f32])` | — | `src/distance.rs` |
| `pub fn validate_vector(vec: &[f32]) -> Result<()>` | Validates that a vector does not contain NaN or infinite values (INV-SIMD-FINITE-1). | `src/distance.rs` |
| `pub fn fixed_hasher() -> RandomState` | Returns a deterministic `RandomState` build hasher seeded with constant values. | `src/fixed_hasher.rs` |
| `pub fn new_fixed_hashmap<K, V>(capacity: usize) -> FixedHashMap<K, V>` | Creates a new deterministic `FixedHashMap` with the given capacity. | `src/fixed_hasher.rs` |
| `pub fn new_fixed_hashset<T>(capacity: usize) -> FixedHashSet<T>` | Creates a new deterministic `FixedHashSet` with the given capacity. | `src/fixed_hasher.rs` |
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
| `pub fn zeroize_node_connections(&self, ram_idx: usize, max_layer: usize, m: usize)` | Zeroizes connection memory for a node slot across all layers and clears counts, without releasing to `free_list` (`INV-VEC-ZEROIZE-1`, `INV… | `src/hnsw/arena.rs` |
| `pub fn new(core: &HnswIndexCore) -> Self` | — | `src/hnsw/batch.rs` |
| `pub fn advance_nanos(&self, delta_nanos: u64)` | Advances the clock by `delta_nanos`. | `src/hnsw/clock.rs` |
| `pub fn new(initial_nanos: u64) -> Self` | Creates a new [`MonotonicCounterClock`] starting at the given nanosecond timestamp. | `src/hnsw/clock.rs` |
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
| `pub fn check_connectivity(&self) -> contextra_types::Result<()>` | Prüft die Graph-Konnektivität (`INV-VEC-CONNECTIVITY-1`): Verifiziert mittels BFS auf Layer 0, dass alle aktiven (nicht gelöschten) Knoten… | `src/hnsw/core_insert.rs` |
| `pub fn compute_insert(&self, id: DocId, vector: &[f32]) -> Result<PreparedInsert>` | — | `src/hnsw/core_insert.rs` |
| `pub fn compute_insert_with_context( &self, id: DocId, vector: &[f32], offset: usize, prior_prepared: &mut [PreparedInsert], batch_ctx: &mut BatchContext, ) -> Result<PreparedInsert>` | — | `src/hnsw/core_insert.rs` |
| `pub fn connectivity_score(&self) -> f64` | — | `src/hnsw/core_insert.rs` |
| `pub fn deleted_ratio(&self) -> f64` | — | `src/hnsw/core_rebuild.rs` |
| `pub fn is_rebuild_required(&self) -> bool` | — | `src/hnsw/core_rebuild.rs` |
| `pub fn rebuild(&self) -> Result<()>` | — | `src/hnsw/core_rebuild.rs` |
| `pub fn rebuild_count(&self) -> u64` | — | `src/hnsw/core_rebuild.rs` |
| `pub fn rebuild_region(&self, region_node_ids: Vec<u64>) -> Result<()>` | — | `src/hnsw/core_rebuild.rs` |
| `pub fn rebuild_region_sync(&self, region_node_ids: Vec<u64>) -> Result<()>` | — | `src/hnsw/core_rebuild.rs` |
| `pub fn rebuild_status(&self) -> RebuildStatus` | — | `src/hnsw/core_rebuild.rs` |
| `pub fn rebuild_sync(&self) -> Result<()>` | — | `src/hnsw/core_rebuild.rs` |
| `pub fn rebuild_sync_with_clock(&self, clock: &dyn contextra_ports::Clock) -> Result<()>` | — | `src/hnsw/core_rebuild.rs` |
| `pub fn rebuild_with_clock(&self, clock: &dyn contextra_ports::Clock) -> Result<()>` | — | `src/hnsw/core_rebuild.rs` |
| `pub fn verify_full_connectivity(&self) -> bool` | Führt Graphtraversierung durch zur Überprüfung von `INV-VEC-CONNECTIVITY-1`. Verifiziert, dass jeder lebende Knoten im Index vom Einstiegsp… | `src/hnsw/core_rebuild.rs` |
| `pub fn visited_dead_nodes(&self) -> u64` | — | `src/hnsw/core_rebuild.rs` |
| `pub fn wait_for_rebuild(&self) -> bool` | — | `src/hnsw/core_rebuild.rs` |
| `pub fn wait_for_rebuild_blocking( &self, timeout: std::time::Duration, _clock: &dyn contextra_ports::Clock, ) -> bool` | — | `src/hnsw/core_rebuild.rs` |
| `pub fn wait_for_rebuild_with_clock(&self, clock: &dyn contextra_ports::Clock) -> bool` | — | `src/hnsw/core_rebuild.rs` |
| `pub fn wait_for_rebuild_with_timeout(&self, timeout: std::time::Duration) -> bool` | — | `src/hnsw/core_rebuild.rs` |
| `pub fn wait_for_rebuild_with_timeout_and_clock( &self, timeout: std::time::Duration, clock: &dyn contextra_ports::Clock, ) -> bool` | — | `src/hnsw/core_rebuild.rs` |
| `pub async fn search_adaptive( &self, query: &[f32], k: usize, policy: &AdaptiveEfPolicy, ) -> Result<Vec<ScoredDocument>>` | Performs an adaptive search (Ada-ef) on the HNSW index, dynamically adjusting search depth `ef`. | `src/hnsw/core_search.rs` |
| `pub async fn search_adaptive_filtered( &self, query: &[f32], k: usize, policy: &AdaptiveEfPolicy, filter: Option<&(dyn Fn(DocId) -> bool + Send + Sync)>, ) -> Result<Vec<ScoredDocument>>` | Performs an adaptive search (Ada-ef) with custom document filter predicate. | `src/hnsw/core_search.rs` |
| `pub async fn search_adaptive_with_stats( &self, query: &[f32], k: usize, policy: &AdaptiveEfPolicy, filter: Option<&(dyn Fn(DocId) -> bool + Send + Sync)>, ) -> Result<(Vec<ScoredDocument>, AdaptiveEfStats)>` | Performs an adaptive search (Ada-ef) returning scored documents along with execution statistics. | `src/hnsw/core_search.rs` |
| `pub fn remove(&self, doc_id: DocId) -> Result<bool>` | O(1) Tombstone-Löschung gemäß VETO-F02 und INV-VEC-ZEROIZE-1. Markiert den Knoten deterministisch als gelöscht, nullt den Vektorspeicher im… | `src/hnsw/deletion.rs` |
| `pub fn remove_with_repair(&mut self, id: &u64) -> Result<bool>` | Removes a document from the vector index by its 64-bit ID. Executes tombstone insertion, immediate slot zeroization (`INV-VEC-ZEROIZE-1`),… | `src/hnsw/deletion.rs` |
| `pub fn hnsw_purger(index: Arc<HnswIndex>) -> Arc<dyn Purger>` | — | `src/hnsw/purge.rs` |
| `pub fn new(index: Arc<HnswIndex>) -> Self` | Erzeugt einen neuen `HnswPurger`. | `src/hnsw/purge.rs` |
| `pub fn has_residual(&self) -> bool` | Gibt an, ob noch irgendwelche Restspuren (ID, Vektorbytes, Kanten, Staging) existieren. | `src/hnsw/scanner.rs` |
| `pub fn hnsw_scanner(index: Arc<HnswIndex>) -> Arc<dyn LayerScanner>` | — | `src/hnsw/scanner.rs` |
| `pub fn inspect_residual(&self, doc_id: DocId) -> ResidualReport` | Lesende Inspektionssicht: Prüft, ob `doc_id` noch in Slots, Kanten oder Puffern vorkommt. | `src/hnsw/scanner.rs` |
| `pub fn new(index: Arc<HnswIndex>) -> Self` | Erzeugt einen neuen `HnswScanner`. | `src/hnsw/scanner.rs` |
| `pub fn scan_units(&self, session: &mut ScanSession, _ctx: &RequestContext) -> Result<()>` | Führt den Schicht-Scan für den HNSW-Index aus. | `src/hnsw/scanner.rs` |
| `pub fn calibrate( batch: &[&[f32]], quantizer: &crate::quantize::ScalarQuantizer, metric: contextra_types::DistanceMetric, ) -> Self` | Calibrates quantization error statistics across a representative batch of vectors. | `src/hnsw/sq8_bias.rs` |
| `pub fn new(mean_bias: f32, variance_bias: f32, sample_count: u64) -> Self` | Creates a new `Sq8Bias` instance with calibrated parameters. | `src/hnsw/sq8_bias.rs` |
| `pub fn discard(&self, tx: TxId)` | Discards staged operations on transaction rollback. | `src/hnsw/staging.rs` |
| `pub fn drain(&self, tx: TxId) -> Vec<VectorStagingOp>` | Drains and returns all staged operations for a given transaction. | `src/hnsw/staging.rs` |
| `pub fn new(capacity: usize, timeout: std::time::Duration) -> Self` | Creates a new vector staging buffer with the specified concurrency ceiling and timeout. | `src/hnsw/staging.rs` |
| `pub fn stage(&self, tx: TxId, op: VectorStagingOp) -> contextra_types::Result<()>` | Stages an operation into the transaction buffer. | `src/hnsw/staging.rs` |
| `pub fn all_doc_ids_and_layers(&self) -> Vec<(DocId, usize)>` | Returns all non-deleted document IDs and their assigned HNSW max layers. | `src/hnsw/types.rs` |
| `pub fn all_doc_ids_from_map(&self) -> Vec<DocId>` | — | `src/hnsw/types.rs` |
| `pub fn check_and_trigger_partial_rebuild(&self)` | — | `src/hnsw/types.rs` |
| `pub fn check_connectivity(&self) -> contextra_types::Result<()>` | — | `src/hnsw/types.rs` |
| `pub fn clock(&self) -> std::sync::Arc<dyn contextra_ports::Clock>` | Returns the clock instance used by this index. | `src/hnsw/types.rs` |
| `pub fn clock(&self) -> std::sync::Arc<dyn contextra_ports::Clock>` | Returns the clock instance used by this index. | `src/hnsw/types.rs` |
| `pub fn compact_seq_log(&self, min_active_seqno: u64)` | — | `src/hnsw/types.rs` |
| `pub fn connectivity_score(&self) -> f64` | — | `src/hnsw/types.rs` |
| `pub fn deleted_ratio(&self) -> f64` | — | `src/hnsw/types.rs` |
| `pub fn entry_point(&self) -> Option<usize>` | Gibt den aktuellen globalen Entry-Point-Knotenindex zurück (falls vorhanden). | `src/hnsw/types.rs` |
| `pub fn get_entry_point(&self) -> Option<usize>` | — | `src/hnsw/types.rs` |
| `pub fn get_ram_entry_point(&self) -> Option<usize>` | — | `src/hnsw/types.rs` |
| `pub fn get_ram_node_connections(&self, ram_idx: usize, layer: usize, m: usize) -> Vec<u32>` | — | `src/hnsw/types.rs` |
| `pub fn is_rebuild_required(&self) -> bool` | — | `src/hnsw/types.rs` |
| `pub fn layer0_connections(&self, node_idx: usize) -> Vec<u32>` | Gibt die Layer-0-Verbindungen für einen Knotenindex zurück. | `src/hnsw/types.rs` |
| `pub fn load_mmap(&self, path: impl AsRef<std::path::Path>) -> Result<()>` | — | `src/hnsw/types.rs` |
| `pub fn load_mmap_sync(&self, path: impl AsRef<std::path::Path>) -> Result<()>` | — | `src/hnsw/types.rs` |
| `pub fn metrics_sink(&self) -> std::sync::Arc<dyn contextra_ports::MetricsSink>` | Returns the metrics sink. | `src/hnsw/types.rs` |
| `pub fn metrics_sink(&self) -> std::sync::Arc<dyn contextra_ports::MetricsSink>` | Returns the metrics sink. | `src/hnsw/types.rs` |
| `pub fn new(config: HnswConfig) -> Self` | — | `src/hnsw/types.rs` |
| `pub fn node_idx_for_doc(&self, doc_id: DocId) -> Option<usize>` | Gibt den internen Knotenindex für eine `DocId` zurück (falls vorhanden und nicht gelöscht). | `src/hnsw/types.rs` |
| `pub fn pin_snapshot(&self, seq_no: u64)` | — | `src/hnsw/types.rs` |
| `pub fn quantizer(&self) -> Option<crate::quantize::ScalarQuantizer>` | Returns a snapshot clone of the current quantizer if trained. | `src/hnsw/types.rs` |
| `pub fn rebuild(&self) -> Result<()>` | — | `src/hnsw/types.rs` |
| `pub fn rebuild_count(&self) -> u64` | — | `src/hnsw/types.rs` |
| `pub fn rebuild_region(&self, region_node_ids: Vec<u64>) -> Result<()>` | — | `src/hnsw/types.rs` |
| `pub fn rebuild_region_sync(&self, region_node_ids: Vec<u64>) -> Result<()>` | — | `src/hnsw/types.rs` |
| `pub fn rebuild_status(&self) -> RebuildStatus` | — | `src/hnsw/types.rs` |
| `pub fn rebuild_sync(&self) -> Result<()>` | — | `src/hnsw/types.rs` |
| `pub fn save(&self, path: impl AsRef<std::path::Path>) -> Result<()>` | — | `src/hnsw/types.rs` |
| `pub fn save_sync(&self, path: impl AsRef<std::path::Path>) -> Result<()>` | — | `src/hnsw/types.rs` |
| `pub fn set_clock(&self, clock: std::sync::Arc<dyn contextra_ports::Clock>)` | Sets the clock instance. | `src/hnsw/types.rs` |
| `pub fn set_clock(&self, clock: std::sync::Arc<dyn contextra_ports::Clock>)` | Sets the clock instance. | `src/hnsw/types.rs` |
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
| `pub fn try_new_with_rng( config: HnswConfig, rng: std::sync::Arc<dyn contextra_ports::Rng>, ) -> Result<Self>` | Creates a new HNSW index with an explicit random number generator for deterministic testing or custom seeding. Uses [`MonotonicCounterClock… | `src/hnsw/types.rs` |
| `pub fn try_new_with_rng_and_clock( config: HnswConfig, rng: std::sync::Arc<dyn contextra_ports::Rng>, clock: std::sync::Arc<dyn contextra_ports::Clock>, ) -> Result<Self>` | Creates a new HNSW index with explicit random number generator and clock injection for full determinism (B11). | `src/hnsw/types.rs` |
| `pub fn unpin_snapshot(&self, seq_no: u64)` | — | `src/hnsw/types.rs` |
| `pub fn visited_dead_nodes(&self) -> u64` | — | `src/hnsw/types.rs` |
| `pub fn wait_for_rebuild(&self) -> bool` | — | `src/hnsw/types.rs` |
| `pub fn wait_for_rebuild_blocking( &self, timeout: std::time::Duration, clock: &dyn contextra_ports::Clock, ) -> bool` | — | `src/hnsw/types.rs` |
| `pub fn wait_for_rebuild_with_timeout(&self, timeout: std::time::Duration) -> bool` | — | `src/hnsw/types.rs` |
| `pub fn with_clock(self, clock: std::sync::Arc<dyn contextra_ports::Clock>) -> Self` | Builder method to set the clock port. | `src/hnsw/types.rs` |
| `pub fn with_metrics_sink(self, sink: std::sync::Arc<dyn contextra_ports::MetricsSink>) -> Self` | Builder method to set the metrics sink. | `src/hnsw/types.rs` |
| `pub fn find_oversaturated_regions( &self, tombstone_map: &BTreeMap<u64, bool>, // node_id → is_tombstoned ) -> Vec<u64>` | Berechnet S_local für alle Hot-Path-Regionen. Gibt Knoten-IDs zurück, für die S_local > θ_c gilt. | `src/partial_rebuild.rs` |
| `pub fn hot_path_nodes(&self) -> impl Iterator<Item = u64> + '_` | Gibt die N zuletzt besuchten eindeutigen Knoten-IDs zurück. | `src/partial_rebuild.rs` |
| `pub fn new(config: PartialRebuildConfig) -> Self` | Erstellt einen neuen TraversalTracker mit der gegebenen Konfiguration. | `src/partial_rebuild.rs` |
| `pub fn record_traversal(&mut self, visited_node_ids: Vec<u64>)` | Trägt die in einem ef_search-Lauf besuchten Node-IDs ein. | `src/partial_rebuild.rs` |
| `pub fn should_trigger_partial_rebuild( tracker: &TraversalTracker, tombstone_map: &BTreeMap<u64, bool>, global_tombstone_ratio: f32, config: &PartialRebuildConfig, ) -> Option<Vec<u64>>` | Entscheidet ob ein lokaler Partial-Rebuild ausgelöst werden soll. Returns `Some(hot_node_ids)` wenn Partial-Rebuild ausgelöst werden soll,… | `src/partial_rebuild.rs` |
| `pub fn write_atomic(destination: &Path, data: &[u8]) -> io::Result<()>` | Writes data to `destination` atomically via a temporary file in the same parent directory, ensuring both file data and parent directory ent… | `src/persistence/atomic.rs` |
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
| `pub fn max_reconstruction_error_per_dim(&self, dim: usize) -> Option<f32>` | Returns the theoretical maximum quantization reconstruction error bound for dimension `dim`. According to INV-SQ8-ERROR-BOUND-1: $$e_i \le… | `src/quantize/mod.rs` |
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
| `pub fn try_train(vectors: &[Vec<f32>], dimension: usize) -> Result<Self>` | Safely trains a `RaBitQQuantizer` for vectors of the specified `dimension` with a default seed. | `src/quantize_rabitq.rs` |
| `pub fn try_train_with_seed(vectors: &[Vec<f32>], dimension: usize, seed: u64) -> Result<Self>` | Safely trains a `RaBitQQuantizer` for vectors of the specified `dimension` with an explicit injected `seed`. Generates a deterministic orth… | `src/quantize_rabitq.rs` |
| `pub fn from_key(key: &str) -> Result<Self, crate::error::VectorError>` | Erzeugt eine DocId aus einem Schlüssel-String via Hashing. | `src/types.rs` |
| `pub fn new(doc_id: DocId, score: f32, distance: f32) -> Self` | Erzeugt ein neues ScoredDocument. | `src/types.rs` |
| `pub fn new() -> Self` | Erstellt eine neue [`SystemClock`]. | `src/types.rs` |
| `pub fn new(seed: u64) -> Self` | Erstellt einen neuen Generator mit Seed. | `src/types.rs` |


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
pub mod contract;
pub mod error;
pub mod memory_index;
pub mod traits;
pub mod types;
pub mod bm25;
pub mod domain;
pub mod inverted;
pub mod morphology;
pub mod posting_list;
pub mod stream;
pub mod tokenizer;
pub mod wand;
pub use contract::{Clock, DocKey, Epoch, Error as ContractError, Result as ContractResult, Rng};
pub use error::{Result as LocalResult, TextError};
pub use memory_index::{CompactStats, MemoryTextIndex};
pub use traits::{Index, Maintain};
pub use types::{DocId as LocalDocId, TextHit, MAX_SEARCH_K as LOCAL_MAX_SEARCH_K};
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
| `default` | `["std", "types-compat"]` |
| `std` | `[]` |
| `serde` | `[]` |
| `tracing` | `[]` |
| `inspect` | `[]` |
| `experimental` | `[]` |
| `types-compat` | `["dep:contextra-types", "dep:contextra-ports"]` |
| `ports-compat` | `["types-compat"]` |
| `bm25f` | `[]` |
| `docid-128` | `[]` |

### Schnittstellen — Public Traits (9)

**`Analyzer`** — *Trait für vollständige Textanalyse-Pipelines (`Tokenizer -> Filter* -> Stemmer`). Garantiert den identischen Ergebnisvertrag für Dokumenten-Indexierung und Abfrage.* (`src/tokenizer.rs`)

```rust
pub trait Analyzer : Send + Sync {
/// Analysiert den gegebenen Text und liefert eine strukturierte Token-Sequenz.
    fn analyze(&self, text: &str) -> Result<Vec<Token>>;

    /// Liefert die stabile, kryptographisch eindeutige Fingerprint-Zeichenkette
    /// bestehend aus Name, Version und Konfigurations-Hash.
    fn fingerprint(&self) -> String;
}
```

**`Clock`** — ** (`src/contract.rs`)

```rust
pub trait Clock : Send + Sync {
fn now_unix_ms(&self) -> u64;
}
```

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

**`Index`** — *Kern-Index-Schnittstelle (Klasse A / SPEC §4.3).* (`src/traits.rs`)

```rust
pub trait Index<Id, Query: ?Sized> {
/// Treffertyp der Suchoperation.
    type Hit;

    /// Fügt ein Dokument mit gegebener ID und Byte-Nutzlast in den Index ein.
    fn insert(&mut self, id: Id, payload: &[u8]) -> Result<()>;

    /// Entfernt ein Dokument anhand seiner ID aus dem Index.
    ///
    /// Gibt `true` zurück, wenn das Dokument vorhanden war und gelöscht wurde.
    fn remove(&mut self, id: &Id) -> Result<bool>;

    /// Führt eine Suche anhand der Query aus und liefert bis zu `limit` Treffer zurück.
    fn search(&self, query: &Query, limit: usize) -> Result<Vec<Self::Hit>>;
}
```

**`Maintain`** — *Neutrale Wartungsschnittstelle (Klasse A / SPEC §4.3).* (`src/traits.rs`)

```rust
pub trait Maintain {
/// Statistiktyp der Kompaktierungsoperation.
    type Stats;

    /// Führt eine Kompaktierung durch und gibt Segmente bzw. gelöschte Einträge tatsächlich frei.
    fn compact(&mut self) -> Result<Self::Stats>;

    /// Bereinigt fragmentierte interne Strukturen.
    fn vacuum(&mut self) -> Result<()>;
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

**`Rng`** — ** (`src/contract.rs`)

```rust
pub trait Rng : Send {
fn next_u64(&mut self) -> u64;
}
```

**`TokenFilter`** — *Trait für Token-Filter in einer Analyse-Pipeline.* (`src/tokenizer.rs`)

```rust
pub trait TokenFilter : Send + Sync {
/// Wendet den Filter auf eine Sequenz von Tokens an.
    fn filter(&self, tokens: Vec<Token>) -> Vec<Token>;
    /// Eindeutiger Bezeichner des Filters für die Fingerprint-Generierung.
    fn filter_id(&self) -> &str;
}
```

**`Tokenizer`** — *Trait für Text-Tokenizer mit unterschiedlichen Filter- oder Zerlegungsstrategien.* (`src/tokenizer.rs`)

```rust
pub trait Tokenizer : Send + Sync {
/// Zerlegt den übergebenen Text in kleingeschriebene Wort-Token und filtert Stoppwörter heraus.
    fn tokenize(&self, text: &str) -> Vec<String>;
}
```

### Öffentliche Datentypen — Structs (38)

| Struct | Felder / Form | Beschreibung | Datei |
|---|---|---|---|
| `AnalyzerPipeline` | `{ name: String, version: u32, limits: PipelineLimits, normalization: NormalizationPolicy, stopwords: Option<HashSet<String>>, filters: Vec<Arc<dyn TokenFilter>>,… }` | Modulare, konfigurierbare Textanalyse-Pipeline. | `src/tokenizer.rs` |
| `BM25` | `{ pub k1: f32, pub b: f32, }` | Konfigurationsparameter für das BM25-Scoring-Modell. - `k1`: Steuert die Sättigung der Termhäufigkeit (TF). Standardwert: 1.5. - `b`: Steuert den Ein… | `src/bm25.rs` |
| `BM25F` | `{ pub k1: f32, pub field_weights: Vec<FieldWeight>, cached_weights: Vec<(FieldId, f32, f32)>, }` | Konfiguration für feldgewichtetes BM25F-Scoring (§11.3). | `src/bm25.rs` |
| `BM25MorphIndex` | `{ inner: InvertedIndex<S>, tokenizer: Arc<dyn MorphologicalTokenizer>, }` | Invertierter Index mit morphologischer Zerlegung für deutsche Volltextsuche. | `src/inverted/morph_index.rs` |
| `Bm25CandidateStream` | `{ index: &'a InvertedIndex<S>, query: String, max_seq: Option<u64>, batch_size: usize, buffer: VecDeque<(DocId, f32)>, seen: BTreeSet<DocId>, fetched_k: usize, y… }` | Asynchroner Pull-Stream zum bedarfsgerechten Nachladen von BM25-Kandidaten. # Kandidaten-Sortierung & Determinismus Ergebnisse werden strikt determin… | `src/stream.rs` |
| `Bm25Scorer` | `{ index: InvertedIndex<S>, }` | — | `src/lib.rs` |
| `BoundedTopK` | `{ heap: BinaryHeap<Bm25Candidate>, capacity: usize, }` | Begrenzter Min-Heap zur effizienten Verwaltung der K besten Treffer. | `src/wand.rs` |
| `CompactStats` | `{ /// Anzahl physisch bereinigter Tombstones. pub purged_docs: usize, /// Anzahl verbliebener aktiver Dokumente. pub active_docs: usize, }` | Statistiken über den Kompaktierungsvorgang. | `src/memory_index.rs` |
| `DefaultTokenizer` | `(tuple/unit)` | Standard-Tokenizer basierend auf Unicode-Wortgrenzen und generischen Stoppwörtern. | `src/tokenizer.rs` |
| `DocId` | `(tuple/unit)` | Standard 64-Bit Dokument-Identifikator. | `src/types.rs` |
| `DocId` | `(tuple/unit)` | 128-Bit Dokument-Identifikator bei aktiviertem `docid-128` Feature. | `src/types.rs` |
| `DocKey` | `(tuple/unit)` | — | `src/contract.rs` |
| `Epoch` | `(tuple/unit)` | — | `src/contract.rs` |
| `FieldWeight` | `{ pub field_id: FieldId, pub weight: f32, pub b: f32, }` | Feldgewichtungskonfiguration für ein bestimmtes Feld in BM25F. | `src/bm25.rs` |
| `GermanCompoundSplitter` | `{ /// Minimum component length for splitting. min_component_len: usize, /// Maximum decomposition depth (number of segments). max_depth: usize, /// Trie data str… }` | Deutscher Komposita-Zerleger basierend auf Dynamischer Programmierung und Fugenanalyse. # Funktionsweise & Algorithmus 1. **Eingebettetes Wörterbuch*… | `src/morphology/compound_splitter.rs` |
| `GermanMorphTokenizer` | `{ splitter: Arc<crate::morphology::GermanCompoundSplitter>, }` | Deutscher Tokenizer mit morphologischer Zerlegung von Wortzusammensetzungen. | `src/tokenizer.rs` |
| `IndexSnapshot` | `{ /// Opake, monotone Epochenmarke dieses Snapshots. pub epoch: Epoch, /// Invertiertes Wörterbuch mit sortierten Posting-Listen. pub postings: BTreeMap<String,… }` | Unveränderlicher Snapshot des Index für lock-freie, konsistente Lesezugriffe. | `src/memory_index.rs` |
| `InvertedIndex` | `{ storage: Arc<S>, prefix: Vec<u8>, tokenizer: Arc<dyn Tokenizer>, pub(crate) total_docs: Arc<AtomicU64>, pub(crate) total_tokens: Arc<AtomicU64>, pub(crate) avg… }` | Persistenter invertierter Index über einer Kollektion im Storage-Backend. # Nebenläufigkeit & Sperrhierarchie: 1. `commit_lock` (`parking_lot::Mutex`… | `src/inverted/index_struct.rs` |
| `KeyInspection` | `{ /// Ob der Schlüssel in den aktiven Dokumentlängen vorkommt. pub in_doc_lengths: bool, /// Ob der Schlüssel aktuell als Tombstone markiert ist. pub is_tombston… }` | Prüfergebnis einer Schlüssel-Inspektion (Feature `inspect`). | `src/memory_index.rs` |
| `LegalDomainVocabulary` | `(tuple/unit)` | Deutsches Juristisches Fachvokabular zur Anbindung an [`DomainVocabulary`]. | `src/domain/legal_de.rs` |
| `MedicalDomainVocabulary` | `(tuple/unit)` | Deutsches Medizinisches Fachvokabular zur Anbindung an [`DomainVocabulary`]. | `src/domain/medical_de.rs` |
| `MemoryTextIndex` | `{ tokenizer: Arc<dyn Tokenizer>, scorer: BM25, snapshot: RwLock<Arc<IndexSnapshot>>, }` | Autarker, thread-sicherer In-Memory Volltextindex (Klasse A). Bietet die standardisierte Index-Schnittstelle gemäß SPEC §4.3 (`insert`, `remove`, `se… | `src/memory_index.rs` |
| `PassthroughTokenizer` | `{ lang: String, }` | Passthrough-Tokenizer für Sprachen, die keine Zerlegung von Zusammensetzungen benötigen. Gibt eingegebene Token unverändert als Einzelkomponente zurü… | `src/morphology/passthrough.rs` |
| `PipelineLimits` | `{ /// Maximale Eingabelänge in Bytes (Default: 1 MiB). pub max_input_length: usize, /// Maximale Tokenlänge in Bytes (Default: 256 Bytes). pub max_token_length:… }` | Konfigurierbare Ressourcen- und Sicherheitsgrenzen der Analyse-Pipeline. | `src/tokenizer.rs` |
| `Posting` | `{ /// Dokument-ID des Eintrags. pub doc_id: DocIdRaw, /// Termfrequenz (Häufigkeit des Terms in diesem Dokument). pub tf: u32, /// Gesamtlänge des Dokuments in T… }` | Kompakte Repräsentation eines einzelnen Vorkommens (Posting) im Index. 8-Byte ausgerichtet (`repr(C, align(8))`) für optimale CPU-Cacheline-Ausnutzun… | `src/posting_list.rs` |
| `PostingBlockInfo` | `{ /// Höchste Dokument-ID in diesem Block (ermöglicht schnelles Überspringen). pub max_doc_id: DocIdRaw, /// Maximale Termfrequenz innerhalb dieses Blocks. pub m… }` | Metadaten eines 64er-Posting-Blocks zur Berechnung oberer BM25-Schranken in Block-Max WAND. | `src/posting_list.rs` |
| `PostingList` | `{ postings: Vec<Posting>, blocks: Vec<PostingBlockInfo>, }` | Zusammenhängende, nach DocId sortierte Posting-Liste für einen Begriff. | `src/posting_list.rs` |
| `PurgeReport` | `{ /// Anzahl erfolgreich physisch gelöschter Einheiten. pub units_purged: u64, /// Anzahl fehlgeschlagener Einheiten während der Ausführung. pub units_failed: u6… }` | — | `src/inverted/purge.rs` |
| `ResidentPostingIndex` | `{ index: RwLock<BTreeMap<String, Arc<PostingList>>>, }` | Speicherresidenter Lese-Cache für Posting-Listen mit RCU-Verwaltung. | `src/posting_list.rs` |
| `TextHit` | `{ /// Eindeutige ID des Dokuments. pub doc_id: DocId, /// Berechneter BM25-Relevanz-Score. pub score: f32, }` | Ergebnisdokument mit Relevanz-Score für BM25-Suchen. | `src/types.rs` |
| `TextIndexMetadata` | `{ /// Gesamtzahl der aktiven Dokumente im Index. pub total_docs: u64, /// Gesamtzahl aller indizierten Token über alle Dokumente. pub total_tokens: u64, /// Fest… }` | Konsolidierte Metadaten des Textindex zur persistenten Speicherung im Schlüssel `meta:stats`. | `src/inverted/types.rs` |
| `TextPurger` | `{ index: Arc<InvertedIndex<S>>, }` | — | `src/inverted/purge.rs` |
| `TextScanner` | `{ index: Arc<InvertedIndex<S>>, target: Arc<parking_lot::Mutex<Option<ScanTarget>>>, }` | — | `src/inverted/scanner.rs` |
| `Token` | `{ /// Normalisierter Termwert. pub term: String, /// Start-Byte-Offset im Originaltext (auf gültiger Zeichengrenze). pub start_offset: usize, /// End-Byte-Offset… }` | Repräsentiert ein strukturiertes Token aus der Textanalyse-Pipeline. Ein Token umfasst den extrahierten Term, Byte-Offsets in den Originaltext (stets… | `src/tokenizer.rs` |
| `TokenReductionMetrics` | `{ /// Anzahl ursprünglicher Token vor der morphologischen Zerlegung. pub original_tokens: usize, /// Anzahl resultierender Token nach Zerlegung von Komposita. pu… }` | Metriken zur Erfassung der Token-Expansions- und Zerlegungswirkung. | `src/morphology/metrics.rs` |
| `Trie` | `{ root: TrieNode, }` | Präfixbaum (Trie) für schnelle O(m)-Wörterbuchabfragen (m = Wortlänge). | `src/morphology/trie.rs` |
| `TrieNode` | `{ /// Zeigt an, ob an diesem Knoten ein vollständiges Wort aus dem Wörterbuch endet. pub is_terminal: bool, /// Kindknoten, indiziert nach Folgezeichen. pub chil… }` | Knoten im Präfixbaum zur Repräsentation von Buchstabenfolgen deutscher Wörter. | `src/morphology/trie.rs` |
| `WandSearchResult` | `{ /// Gefundene Trefferdokumente mit Relevanzwert. pub results: Vec<(DocId, f32)>, /// Anzahl tatsächlich voll berechneter Dokumente (Pruning-Effektivitätsmetrik… }` | Ergebnis einer Block-Max-WAND-Suche inklusive Top-K-Treffern und Metadaten über evaluierte Dokumente. | `src/wand.rs` |

### Öffentliche Datentypen — Enums (6)

| Enum | Varianten | Beschreibung | Datei |
|---|---|---|---|
| `Error` | `#[error("ungültige Eingabedaten: {0}")] InvalidInput(&'static str), #[error("Kapazitätsgrenze überschritten: gefordert {requested}, maximal {maximum}")] CapacityExceeded { requested: usize, maximum:…` | — | `src/contract.rs` |
| `Language` | `/// Englischer Standard-Tokenizer (Unicode-Wortgrenzen + Stoppwortfilterung). #[default] English, /// Deutscher morphologischer Tokenizer (Komposita-Zerlegung + Umlaut-Normalisierung). German, /// Be…` | Unterstützte Sprachen und Tokenisierungsstrategien für den invertierten Index. Steuert, welcher Tokenizer für die Dokumentenindizierung und Suchabfra… | `src/inverted/types.rs` |
| `NormalizationPolicy` | `/// Nur Unicode Lowercasing. #[default] Lowercase, /// NFKC-Normalisierung (Ligaturen auflösen, ZWJ/Formatzeichen entfernen, Fullwidth normalisieren) + Lowercasing. Nfkc, /// Spezifische deutsche Uml…` | Konfigurierbare Normalisierungsrichtlinie für Textanalyse. | `src/tokenizer.rs` |
| `OverflowPolicy` | `/// Überschüssige Eingaben oder Token werden deterministisch abgeschnitten. #[default] Truncate, /// Eine Überschreitung führt zu einem Fail-Closed Fehler. Error,` | Strategie bei Überschreitung konfigurierter Kapazitätsgrenzen. | `src/tokenizer.rs` |
| `ParsedTextTarget` | `Document(DocId), Collection(String),` | — | `src/inverted/scanner.rs` |
| `TextError` | `/// Ungültige Eingabedaten (z. B. leere/ungültige Abfrage, ungültige Scoring-Parameter). #[error("ungültige Eingabedaten: {0}")] InvalidInput(String), /// Fehler bei Textanalyse, Regex oder Tokenisie…` | Typisierter Fehler für Volltextindexierung, Tokenisierung und Scoring. | `src/error.rs` |

### Typ-Aliase (5)

| Alias | Definition | Beschreibung |
|---|---|---|
| `DocIdRaw` | `type DocIdRaw= u64` | — |
| `DocIdRaw` | `type DocIdRaw= u128` | — |
| `FieldId` | `type FieldId= u32` | Eindeutige Kennung für Dokumentenfelder im feldgewichteten BM25F-Scoring. |
| `Result` | `type Result<T> = core::result::Result<T, Error>` | — |
| `Result` | `type Result<T> = core::result::Result<T, TextError>` | Einheitlicher Result-Typ für `contextra-text`. |

### Konstanten (11)

| Konstante | Typ | Beschreibung |
|---|---|---|
| `BLOCK_SIZE` | `usize` | Blockgröße für die Block-Max-WAND-Zerlegung (64 Postings pro Block). |
| `BM25_B` | `f32` | Standard-b-Parameter für die Dokumentlängen-Normalisierung in BM25 (0.75). |
| `BM25_K1` | `f32` | Standard-k1-Parameter für die Termfrequenz-Sättigung in BM25 (1.5). |
| `DEFAULT_STREAM_BATCH_SIZE` | `usize` | Standard-Batch-Größe für das Nachladen von Kandidaten (16 Dokumente). |
| `MAX_DOC_BYTES` | `usize` | Maximale Dokumentgröße in Bytes (fail-closed, 10 MB). |
| `MAX_DOC_TERMS` | `usize` | Maximale Anzahl von Begriffen (Tokens) pro Dokument (fail-closed). |
| `MAX_POSTINGS_PER_TERM` | `usize` | Maximale Anzahl von Postings pro Term (fail-closed). |
| `MAX_SEARCH_K` | `usize` | Maximale Anzahl von Suchtreffern pro Anfrage (SPEC §11.3). |
| `MAX_STAGED_TRANSACTIONS` | `usize` | Maximale Anzahl gleichzeitig ungecommitteter Transaktionen (10.000). |
| `MAX_TEXT_BYTES` | `usize` | Maximale Textgröße pro Dokument (10 MiB) zur Begrenzung von Speicherverbrauch und DoS-Schutz. |
| `POSTING_LIST_V2_MAGIC` | `&[u8` | Magische Bytes für das kompakte V2-Posting-Listen-Format (`PL\x02\x00`). |

### Öffentliche Funktionen (120)

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
| `pub async fn compact_terms(&self, tx: TxId) -> Result<u64>` | Kompaktiert Terme und bereinigt verwaiste Metadaten im Inverted Index (Maintenance / Compaction API). Führt eine vollständige Bereinigung v… | `src/inverted/index_struct.rs` |
| `pub async fn delete_document(&self, tx: TxId, doc_id: DocId) -> Result<()>` | Löscht ein Dokument logisch aus dem Index und markiert alle Terme mit Tombstones. | `src/inverted/index_struct.rs` |
| `pub async fn load_stats(&self) -> Result<()>` | Lädt die persistenten Statistiken (`meta:stats`) aus dem Storage in die atomaren In-Memory-Felder. | `src/inverted/index_struct.rs` |
| `pub fn new(storage: Arc<S>, namespace: &str) -> Self` | Erzeugt einen neuen invertierten Index mit englischem Standard-Tokenizer. | `src/inverted/index_struct.rs` |
| `pub fn new_with_language(storage: Arc<S>, namespace: &str, language: Language) -> Self` | Erzeugt einen neuen invertierten Index mit expliziter Sprachkonfiguration. # Language Selection Use `Language::from_iso("de")` for German,… | `src/inverted/index_struct.rs` |
| `pub fn prefix(&self) -> &[u8]` | Accessor for index storage prefix bytes. | `src/inverted/index_struct.rs` |
| `pub async fn resolve_tombstones(&self, tx: TxId) -> Result<u64>` | — | `src/inverted/index_struct.rs` |
| `pub async fn rewrite_posting_lists(&self, tx: TxId, terms: &[&str]) -> Result<usize>` | Schreibt Postinglisten für gegebene Terme neu (Maintenance / Rewrite API). Konsolidiert unkomprimierte und komprimierte Batches, bereinigt… | `src/inverted/index_struct.rs` |
| `pub async fn search_bm25( &self, query: &str, k: usize, max_seq: Option<u64>, ) -> Result<Vec<(DocId, f32)>>` | Führt eine Volltextsuche mit BM25 aus. | `src/inverted/index_struct.rs` |
| `pub async fn search_bm25_at( &self, query: &str, k: usize, max_seq: Option<u64>, ) -> Result<Vec<(DocId, f32)>>` | Führt eine Volltextsuche mit BM25 zu einer bestimmten MVCC-Snapshot-Sequenznummer aus. | `src/inverted/index_struct.rs` |
| `pub fn storage(&self) -> &Arc<S>` | Accessor for underlying storage engine. | `src/inverted/index_struct.rs` |
| `pub async fn upsert_document(&self, tx: TxId, doc_id: DocId, text: &str) -> Result<()>` | Fügt ein Dokument in den Index ein oder aktualisiert ein bestehendes. # Ablauf 1. Größenprüfung gegen `MAX_TEXT_BYTES`. 2. Tokenisierung un… | `src/inverted/index_struct.rs` |
| `pub fn with_tokenizer(mut self, tokenizer: Arc<dyn Tokenizer>) -> Self` | Konfiguriert einen benutzerdefinierten Tokenizer. | `src/inverted/index_struct.rs` |
| `pub fn new( storage: Arc<S>, namespace: &str, tokenizer: Arc<dyn MorphologicalTokenizer>, ) -> Self` | Erzeugt einen neuen morphologischen Index mit deutschem Standard-Tokenizer. | `src/inverted/morph_index.rs` |
| `pub fn tokenizer(&self) -> &dyn MorphologicalTokenizer` | Liefert eine Referenz auf den konfigurierten morphologischen Tokenizer. | `src/inverted/morph_index.rs` |
| `pub fn is_success(&self) -> bool` | Liefert `true`, wenn die Bereinigung fehlerfrei und vollständig war. | `src/inverted/purge.rs` |
| `pub fn new() -> Self` | Erzeugt einen leeren Purge-Bericht. | `src/inverted/purge.rs` |
| `pub fn new(index: Arc<InvertedIndex<S>>) -> Self` | Creates a new `TextPurger` for the given index. | `src/inverted/purge.rs` |
| `pub fn purge_collection(&self, tx: TxId, namespace: &str) -> Result<PurgeStats>` | Fast-path purge for an entire collection namespace (`__txt:{collection}:`). | `src/inverted/purge.rs` |
| `pub fn purge_collection_report(&self, tx: TxId, namespace: &str) -> PurgeReport` | Fast-path purge for an entire collection namespace (`__txt:{collection}:`), returning a `PurgeReport`. | `src/inverted/purge.rs` |
| `pub fn purge_document(&self, tx: TxId, doc_id: DocId) -> Result<PurgeStats>` | Purges a single document from the inverted index and recalculates corpus statistics. | `src/inverted/purge.rs` |
| `pub fn purge_document_report(&self, tx: TxId, doc_id: DocId) -> PurgeReport` | Purges a single document from the inverted index and recalculates corpus statistics, returning a detailed `PurgeReport`. | `src/inverted/purge.rs` |
| `pub fn text_purger<S: StorageEngine + 'static>(index: Arc<InvertedIndex<S>>) -> Arc<dyn Purger>` | — | `src/inverted/purge.rs` |
| `pub fn to_purge_stats(&self) -> Result<PurgeStats>` | Konvertiert den Bericht in [`PurgeStats`] oder liefert einen Fehler bei Teilabbrüchen. | `src/inverted/purge.rs` |
| `pub fn new(index: Arc<InvertedIndex<S>>) -> Self` | Erzeugt einen neuen `TextScanner` für den übergebenen invertierten Index. | `src/inverted/scanner.rs` |
| `pub fn new_with_target(index: Arc<InvertedIndex<S>>, target: ScanTarget) -> Self` | Erzeugt einen neuen `TextScanner` mit vorab konfiguriertem Scan-Ziel. | `src/inverted/scanner.rs` |
| `pub fn parse(target_id: &str) -> Self` | Parses a target ID string into document ID or collection namespace. | `src/inverted/scanner.rs` |
| `pub fn set_target(&self, target: ScanTarget)` | Weist dem Scanner ein neues Zielobjekt zu. | `src/inverted/scanner.rs` |
| `pub fn text_scanner<S: StorageEngine + 'static>( index: Arc<InvertedIndex<S>>, ) -> Arc<dyn LayerScanner>` | — | `src/inverted/scanner.rs` |
| `pub fn with_target(self, target: ScanTarget) -> Self` | Konfiguriert das Scan-Ziel im Builder-Muster. | `src/inverted/scanner.rs` |
| `pub fn from_iso(code: &str) -> Self` | Parst einen ISO-639-1 Sprachcode ("de", "de-AT", "en", "en-US"). Bewertet nur das primäre Subtag vor dem ersten Bindestrich. Unbekannte Cod… | `src/inverted/types.rs` |
| `pub fn new(storage: std::sync::Arc<S>, namespace: &str) -> Self` | Erzeugt eine neue `Bm25Scorer`-Instanz über der angegebenen Speicher-Engine. # Ablauf 1. Übernahme der Referenz auf die `StorageEngine`. 2.… | `src/lib.rs` |
| `pub fn current_epoch(&self) -> Epoch` | Liefert die aktuelle Epoche des Index. | `src/memory_index.rs` |
| `pub fn inspect_doc_lengths(&self) -> BTreeMap<DocKey, u32>` | Rohsicht-Inspektion der Dokumentlängen unter dem Feature `inspect`. | `src/memory_index.rs` |
| `pub fn inspect_key(&self, key: &DocKey) -> KeyInspection` | Rohsicht-Inspektion unter dem Feature `inspect`. | `src/memory_index.rs` |
| `pub fn inspect_postings(&self) -> BTreeMap<String, Vec<Posting>>` | Rohsicht-Inspektion aller Postings unter dem Feature `inspect`. | `src/memory_index.rs` |
| `pub fn is_empty(&self) -> bool` | Gibt an, ob der Index keine aktiven Dokumente enthält. | `src/memory_index.rs` |
| `pub fn len(&self) -> usize` | Liefert die Anzahl aktiver Dokumente im Index. | `src/memory_index.rs` |
| `pub fn new() -> Self` | Erzeugt einen neuen `MemoryTextIndex` mit Standard-Tokenizer und BM25-Scorer. | `src/memory_index.rs` |
| `pub fn search(&self, query_tokens: &[String], scorer: &BM25, limit: usize) -> Vec<TextHit>` | Sucht im unveränderlichen Snapshot anhand von BM25. | `src/memory_index.rs` |
| `pub fn snapshot(&self) -> Arc<IndexSnapshot>` | Liefert einen unveränderlichen Snapshot des Index (O(1) Lesezugriff). | `src/memory_index.rs` |
| `pub fn with_tokenizer(mut self, tokenizer: Arc<dyn Tokenizer>) -> Self` | Konfiguriert einen spezifischen Tokenizer. | `src/memory_index.rs` |
| `pub fn extend_vocabulary(&mut self, words: impl IntoIterator<Item = impl AsRef<str>>)` | Erweitert das Wörterbuch des Zerlegers dynamisch um zusätzliche Begriffe. | `src/morphology/compound_splitter.rs` |
| `pub fn max_depth(&self) -> usize` | Liefert die maximale Segmenttiefe. | `src/morphology/compound_splitter.rs` |
| `pub fn min_component_len(&self) -> usize` | Liefert die Mindestlänge für Teilkomponenten. | `src/morphology/compound_splitter.rs` |
| `pub fn new() -> Self` | Erzeugt einen neuen Zerleger mit der Standard-Mindestlänge (3 Zeichen) und Basis-Wortschatz. | `src/morphology/compound_splitter.rs` |
| `pub fn new_with_all_domains() -> Self` | Erzeugt einen Zerleger mit allen eingebauten Fachdomänen (Recht & Medizin). | `src/morphology/compound_splitter.rs` |
| `pub fn with_dictionary(min_len: usize, custom_words: HashSet<String>) -> Self` | Erzeugt einen Zerleger mit benutzerdefiniertem Wörterbuch. | `src/morphology/compound_splitter.rs` |
| `pub fn with_domains(min_len: usize, domains: &[&dyn DomainVocabulary]) -> Self` | Erzeugt einen Zerleger erweitert um spezifische Fachvokabulare (z. B. Recht, Medizin). | `src/morphology/compound_splitter.rs` |
| `pub fn with_max_depth(mut self, max_depth: usize) -> Self` | Setzt die maximale Rekursions-/Segmenttiefe für Zerlegungen. | `src/morphology/compound_splitter.rs` |
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
| `pub fn decode_compact(bytes: &[u8]) -> Result<Self, ContextraError>` | Dekodiert eine Posting-Liste aus dem Delta-Varint-Format mit Fallback auf Legacy-Bincode. # Defensives Verhalten Weist unvollständige, abge… | `src/posting_list.rs` |
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
| `pub fn default_english() -> Self` | Erzeugt die Standard-Pipeline für englischen und generischen Text. | `src/tokenizer.rs` |
| `pub fn default_german() -> Self` | Erzeugt die Standard-Pipeline für deutsche Texte inklusive Komposita-Zerlegung und Umlauten. | `src/tokenizer.rs` |
| `pub fn get_protected_regex() -> Result<&'static Regex>` | Liefert den gecachten regulären Standardausdruck für geschützte URLs und E-Mail-Adressen. | `src/tokenizer.rs` |
| `pub fn new() -> Self` | — | `src/tokenizer.rs` |
| `pub fn new() -> Self` | — | `src/tokenizer.rs` |
| `pub fn new( term: impl Into<String>, start_offset: usize, end_offset: usize, position_increment: usize, ) -> Self` | Erzeugt ein neues Token mit Term, Offsets und Positionsinkrement. | `src/tokenizer.rs` |
| `pub fn new(name: impl Into<String>, version: u32) -> Self` | Erzeugt eine neue leere Analyse-Pipeline mit Standard-Grenzen. | `src/tokenizer.rs` |
| `pub fn normalize(&self, text: &str) -> String` | Führt die Normalisierung deterministisch und idempotent auf dem String aus. | `src/tokenizer.rs` |
| `pub fn tokenize(text: &str) -> Vec<String>` | Hilfsfunktion zur Tokenisierung mit dem [`DefaultTokenizer`]. | `src/tokenizer.rs` |
| `pub fn try_new() -> Result<Self>` | Fallible Konstruktion eines `DefaultTokenizer` mit Validierung des regulären Schutzmusters. | `src/tokenizer.rs` |
| `pub fn try_new() -> Result<Self>` | Erzeugt eine neue Instanz des deutschen morphologischen Tokenizers. | `src/tokenizer.rs` |
| `pub fn with_filter(mut self, filter: Arc<dyn TokenFilter>) -> Self` | Fügt einen Token-Filter am Ende der Filterkette hinzu. | `src/tokenizer.rs` |
| `pub fn with_german_morphology( mut self, splitter: Arc<crate::morphology::GermanCompoundSplitter>, ) -> Self` | Aktiviert die deutsche Komposita-Zerlegung. | `src/tokenizer.rs` |
| `pub fn with_limits(mut self, limits: PipelineLimits) -> Self` | Konfiguriert die Ressourcen- und Sicherheitsgrenzen. | `src/tokenizer.rs` |
| `pub fn with_normalization(mut self, norm: NormalizationPolicy) -> Self` | Konfiguriert die Normalisierungsrichtlinie. | `src/tokenizer.rs` |
| `pub fn with_stopwords(mut self, stopwords: HashSet<String>) -> Self` | Konfiguriert die Stoppwortliste. | `src/tokenizer.rs` |
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
| **Interne Abhängigkeiten** | `contextra-ports`, `contextra-types` |
| **Externe Abhängigkeiten (Auswahl)** | `ahash`, `arc-swap`, `bincode`, `blake3`, `bytes`, `parking_lot`, `scc`, `serde`, `serde_json`, `thiserror`, `tracing` |

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
pub mod provider;
pub mod purger;
pub mod session_dag;
pub mod test_doubles;
pub mod tl_hfd;
pub use apprh::{ apprh_local, forward_push_apprh, shadow_compare_forward_push_vs_apprh, ApprhError, ApprhFlipGate, ApprhMode, ApprhParams, ApprhSelector, ApprhSelectorConfig, ApprhShadowComparison, DefaultApprhFlipGate, };
pub use arc_slice::ArcSlice;
pub use cascade::{ cascade_invalidate_edges_for_superseded_doc, cascade_invalidate_hyperedges_for_superseded_doc, CascadeDeletionRecord, CascadeInvalidationReport, CascadeReport, CascadeStatus, CascadeTicket, GraphGarbageCollection, HyperedgeCascadeReport, MAX_HYPEREDGE_CASCADE_FANOUT, };
pub use community::{ detect_communities, CommunityAssignment, CommunityDetectionConfig, StarExpansionIterator, VirtualHyperedgeNode, };
pub use consistency_enforcement::{ ConflictPattern, ConsistencyEnforcer, ContradictionDetector, EdgeAssertion, EdgeId, ExactPredicateConflictDetector, };
pub use csr::CsrGraph;
pub use edge_reinforcement::{ apply_cooccurrence_reinforcement, apply_traversal_reinforcement, apply_weight_normalization, compute_edge_weight, EdgeReinforcementConfig, };
pub use edge_reinforcement_buffer::{CooccurrenceSignal, EdgeReinforcementBuffer, TraversalSignal};
pub use entity_extraction::{ ingest_extracted_relations, EntityExtractor, ExtractedRelation, RuleBasedEntityExtractor, };
pub use error::{GraphError, GraphMutationError, Result};
pub use error::GraphError as Error;
pub use hyperedge::{ sort_dedup_entities, star_weight, ConsolidationNodesGuard, HyperEdge, HyperEdgeId, HyperEdgeView, RoleBinding, RoleId, RoleInterner, HYPEREDGE_BY_ENTITY_PREFIX, HYPEREDGE_PREFIX, };
pub use hyperedge_suggest::{ compute_co_occurrence_candidates, validate_candidates_with_llm, HyperEdgeCandidate, HyperedgeSuggestError, ValidatedHyperEdgeCandidate, MAX_RELATE_PARTICIPANTS, };
pub use path_rag::{EntityId, GraphPath, PathGraph, PathRAGConfig, PathRAGEngine};
pub use path_rag::{KPathConfig, KPathDiffusion, KPathResult};
pub use percolation::{ compute_percolation_health, find_rebonding_candidates, should_trigger_rebonding, PercolationConfig, };
pub use ppr::compute_ppr_with_apprh_selector;
pub use ppr::{DeletedView, PprContext};
pub use ppr_stream::{PprCandidateStream, DEFAULT_PPR_STREAM_BATCH_SIZE};
pub use provenance::{DocEdgeIndex, EdgeProvenance};
pub use purger::{graph_purger, GraphPurger};
pub use provider::{ compute_pagerank, detect_communities_on_provider, GraphProvider, InMemoryGraphProvider, };
pub use session_dag::{ AgentStateNode, DagEdge, NodeIdx, NodesGuard, NodesWriteGuard, SessionBranchTree, };
pub use tl_hfd::{ shadow_compare_forward_push_vs_tl_hfd, tl_hfd_local, DefaultFlipGate, ShadowComparison, ShadowDiscrepancyReport, TlHfdError, TlHfdFlipGate, TlHfdParams, MIN_AGREEMENT_THRESHOLD, MIN_SHADOW_SAMPLES, };
```

### Feature-Flags

| Flag | Aktiviert |
|---|---|
| `default` | `["types-compat", "ports-compat", "experimental-graph"]` |
| `experimental-graph` | `[]` |
| `types-compat` | `["dep:contextra-types", "dep:contextra-ports"]` |
| `ports-compat` | `["types-compat"]` |
| `docid-128` | `[]` |
| `graph-connectivity-health` | `[]` |
| `edge-reinforcement-learning` | `[]` |
| `ppr-forward-push` | `[]` |
| `k-path-diffusion` | `[]` |
| `apprh-diffusion` | `[]` |
| `physio-percolation` | `["graph-connectivity-health"]` |

### Schnittstellen — Public Traits (13)

**`ApprhFlipGate`** — ** (`src/apprh/shadow.rs`)

```rust
pub trait ApprhFlipGate {
/// Evaluates whether APPRH meets default flip criteria.
    fn should_flip(&self, report: &ApprhShadowComparison) -> bool;
}
```

**`Clock`** — ** (`src/contract.rs`)

```rust
pub trait Clock : Send + Sync {
fn now_unix_ms(&self) -> u64;
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

**`GraphProvider`** — *Generic Graph Provider abstraction.* (`src/provider.rs`)

```rust
pub trait GraphProvider : Send + Sync {
/// Returns all active node IDs in the graph.
    fn node_ids(&self) -> Vec<EntityId>;
    /// Returns the number of active nodes.
    fn node_count(&self) -> usize {
        self.node_ids().len()
    }
    /// Checks if a node exists.
    fn contains_node(&self, node: EntityId) -> bool;
    /// Returns outgoing neighbors of `node` with their edge weights.
    fn outgoing_edges(&self, node: EntityId) -> Vec<(EntityId, f32)>;
    /// Returns incoming predecessors of `node` with their edge weights.
    fn incoming_edges(&self, node: EntityId) -> Vec<(EntityId, f32)>;
    /// Returns out-degree.
    fn out_degree(&self, node: EntityId) -> usize {
        self.outgoing_edges(node).len()
    }
    /// Returns in-degree.
    fn in_degree(&self, node: EntityId) -> usize {
        self.incoming_edges(node).len()
    }
    /// Returns hyperedge IDs for entity.
    fn hyperedges_for_entity(&self, _node: EntityId) -> Vec<HyperEdgeId> {
        Vec::new()
    }
    /// Resolves HyperEdgeId.
    fn get_hyperedge(&self, _id: HyperEdgeId) -> Option<Arc<HyperEdge>> {
        None
    }
    /// Returns role bindings.
    fn hyperedge_participants(&self, id: HyperEdgeId) -> Arc<[RoleBinding]> {
        self.get_hyperedge(id)
            .map(|he| he.participants.clone())
            .unwrap_or_else(|| Arc::from([]))
    }
}
```

**`GraphStorage`** — *Standalone key-value storage interface for persisting graph data.* (`src/storage.rs`)

```rust
pub trait GraphStorage : Send + Sync {
/// Inserts or updates a key-value pair.
    fn put<'a>(&'a self, key: &'a [u8], value: &'a [u8]) -> StorageFuture<'a, Result<(), String>>;

    /// Retrieves the value associated with a key.
    fn get<'a>(&'a self, key: &'a [u8]) -> StorageFuture<'a, Result<Option<Vec<u8>>, String>>;

    /// Deletes a key-value pair.
    fn delete<'a>(&'a self, key: &'a [u8]) -> StorageFuture<'a, Result<(), String>>;

    /// Scans key-value pairs matching a given prefix.
    fn scan_prefix<'a>(
        &'a self,
        prefix: &'a [u8],
    ) -> StorageFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>, String>>;
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

**`Rng`** — ** (`src/contract.rs`)

```rust
pub trait Rng : Send {
fn next_u64(&mut self) -> u64;
}
```

**`ShadowSink`** — *Dyn-kompatibler Port für Diskrepanz-Senken.* (`src/ppr/shadow_hook.rs`)

```rust
pub trait ShadowSink<T> : Send + Sync {
/// Zeichnet einen Diskrepanz-Eintrag auf.
    fn record(&self, discrepancy: ShadowDiscrepancy<T>);
}
```

### Öffentliche Datentypen — Structs (86)

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
| `DocId` | `(tuple/unit)` | Document identifier. | `src/types.rs` |
| `Edge` | `{ /// Source entity ID. pub from: EntityId, /// Target entity ID. pub to: EntityId, /// Edge relation label. pub label: Arc<str>, /// Edge weight. pub weight: f3… }` | Directed edge. | `src/types.rs` |
| `EdgeAssertion` | `{ /// Subjekt-Knoten der Aussage. pub subject: NodeIdx, /// Hash des Prädikats/Relationsnamens. pub predicate_hash: [u8; 32], /// Repräsentation des Objekts/Ziel… }` | — | `src/consistency_enforcement.rs` |
| `EdgeProvenance` | `{ /// Identifikator der Kante (from_entity_id, to_entity_id). pub edge_id: EdgeId, /// Dokumente deren Inhalt zu dieser Kante beigetragen hat. pub source_doc_ids… }` | — | `src/provenance.rs` |
| `EdgeReinforcementBuffer` | `{ cooccurrence: Mutex<Vec<CooccurrenceSignal>>, traversal: Mutex<Vec<TraversalSignal>>, }` | — | `src/edge_reinforcement_buffer.rs` |
| `EdgeReinforcementConfig` | `{ pub eta: f32, pub delta: f32, pub w_max: f32, pub rho: f32, pub q: f32, pub alpha: f32, }` | — | `src/edge_reinforcement.rs` |
| `Entity` | `{ /// Unique entity ID. pub id: EntityId, /// Canonical entity name. pub name: Arc<str>, /// Entity type / category. pub entity_type: Arc<str>, }` | Entity node. | `src/types.rs` |
| `EntityId` | `(tuple/unit)` | Identifier of an entity node in the graph. | `src/types.rs` |
| `ExactPredicateConflictDetector` | `(tuple/unit)` | — | `src/consistency_enforcement.rs` |
| `ExtractedRelation` | `{ /// Name / Identifikator der Quell-Entität (Subjekt). pub source: String, /// Name / Identifikator der Ziel-Entität (Objekt). pub target: String, /// Typ / Bez… }` | — | `src/entity_extraction.rs` |
| `FaultConfig` | `{ /// If true, all file operations fail. pub fail_all: bool, }` | Fault configuration for crash simulation. | `src/test_doubles.rs` |
| `FaultVfs` | `{ files: Arc<RwLock<BTreeMap<std::path::PathBuf, Vec<u8>>>>, failing: Arc<parking_lot::Mutex<bool>>, }` | Simple in-memory fault-injecting VFS double for crash simulation tests. | `src/test_doubles.rs` |
| `GraphInner` | `{ pub(crate) id_map: HashMap<EntityId, InternalIndex>, pub(crate) reverse_map: Vec<EntityId>, pub(crate) entities: Vec<Entity>, pub(crate) communities: HashMap<E… }` | — | `src/csr/inner/mod.rs` |
| `GraphPath` | `{ /// Knoten-Sequenz vom Start zum Ziel. pub nodes: Vec<EntityId>, /// Kantengewichte entlang des Pfades (len = nodes.len() - 1). pub edge_weights: Vec<f32>, ///… }` | — | `src/path_rag/mod.rs` |
| `GraphPurger` | `{ graph: Arc<CsrGraph>, }` | — | `src/purger.rs` |
| `GraphStats` | `{ /// Number of active/committed nodes \|V\|. pub node_count: usize, /// Number of directed edges \|E\|. pub edge_count: usize, /// Average outgoing degree \|E\| / \|V\|… }` | — | `src/ppr/cost.rs` |
| `HyperEdge` | `{ /// Eindeutige ID der Hyperkante. pub id: HyperEdgeId, /// Prädikat/Typ der Hyperkante. pub predicate: EdgeType, /// Liste der Teilnehmer mit ihren Rollen. ///… }` | — | `src/hyperedge.rs` |
| `HyperEdgeCandidate` | `{ /// Canonical sorted list of participating entity IDs. pub participants: Vec<EntityId>, /// Number of distinct documents in which this exact entity combination… }` | — | `src/hyperedge_suggest.rs` |
| `HyperEdgeId` | `(tuple/unit)` | Hyperedge identifier. | `src/types.rs` |
| `HyperEdgeView` | `{ pub edge: &'a HyperEdge, pub participants: ArcSlice<RoleBinding>, }` | — | `src/hyperedge.rs` |
| `HyperedgeCascadeReport` | `{ /// Hyperedges tombstoned synchronously in this cascade run (up to MAX_HYPEREDGE_CASCADE_FANOUT). pub invalidated: Vec<crate::hyperedge::HyperEdgeId>, /// Hype… }` | — | `src/cascade.rs` |
| `InMemoryGraphProvider` | `{ nodes: BTreeSet<EntityId>, outgoing: BTreeMap<EntityId, Vec<(EntityId, f32)>>, incoming: BTreeMap<EntityId, Vec<(EntityId, f32)>>, hyperedges: BTreeMap<HyperEd… }` | Standalone in-memory reference implementation of `GraphProvider`. | `src/provider.rs` |
| `InMemoryGraphStorage` | `{ map: std::sync::Arc<parking_lot::RwLock<std::collections::BTreeMap<Vec<u8>, Vec<u8>>>>, }` | Standalone in-memory storage implementation for standalone testing and benchmarks. | `src/storage.rs` |
| `InMemoryStorageEngine` | `{ data: Arc<RwLock<BTreeMap<Vec<u8>, Vec<u8>>>>, last_tx: Arc<AtomicU64>, }` | In-memory storage engine double implementing `contextra_ports::StorageEngine`. | `src/test_doubles.rs` |
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
| `PprConfig` | `{ /// Damping factor. pub damping_factor: f32, /// Maximum convergence iterations. pub max_iterations: u32, /// Convergence tolerance delta. pub convergence_epsi… }` | Configuration parameters for Personalized PageRank execution. | `src/types.rs` |
| `PprContext` | `{ pub(crate) valid_seeds: Vec<usize>, pub(crate) out_weight_sums: Vec<f32>, pub(crate) ranks: Vec<f32>, pub(crate) next_ranks: Vec<f32>, }` | — | `src/ppr/types.rs` |
| `PprCostCalibrator` | `{ samples: Arc<Mutex<Vec<PprCostSample>>>, }` | — | `src/ppr/cost.rs` |
| `PprCostEstimate` | `{ /// Estimated edge accesses for Andersen-Chung-Lang ForwardPush: $O(1 / (\epsilon \cdot \alpha))$. pub forward_push: f64, /// Estimated edge accesses for Dense… }` | — | `src/ppr/cost.rs` |
| `PprCostSample` | `{ /// Algorithm evaluated. pub algorithm: PprAlgorithm, /// Estimated cost in edge access units. pub estimated_cost: f64, /// Measured edge accesses (counted ope… }` | — | `src/ppr/cost.rs` |
| `PprParams` | `{ /// Teleport probability ($\alpha$, default: 0.15). pub alpha: f32, /// Convergence error tolerance threshold ($\epsilon$, default: 1e-4). pub epsilon: f32, //… }` | — | `src/path_rag/mod.rs` |
| `RequestContext` | `{ /// Optional tenant identifier. pub tenant_id: Option<String>, }` | Request context placeholder. | `src/types.rs` |
| `ResourceTracker` | `{ consumed_bytes: usize, max_bytes: usize, }` | Resource tracker. | `src/types.rs` |
| `RoleBinding` | `{ /// Role identifier. pub role: RoleId, /// Participant entity ID. pub entity: EntityId, }` | Role binding. | `src/types.rs` |
| `RoleId` | `(tuple/unit)` | Role identifier. | `src/types.rs` |
| `RoleInterner` | `{ forward: HashMap<String, RoleId>, backward: HashMap<RoleId, String>, next_id: AtomicU32, }` | — | `src/hyperedge.rs` |
| `RuleBasedEntityExtractor` | `{ /// Minimale Konfidenzschwelle. Ergebnisse unterhalb dieser Schwelle werden nicht zurückgegeben. min_confidence: f32, /// Registrierte Extraktionsmuster. patte… }` | — | `src/entity_extraction.rs` |
| `SessionBranchTree` | `{ nodes: RwLock<HashMap<NodeIdx, AgentStateNode>>, edges: RwLock<Vec<DagEdge>>, active_head: RwLock<NodeIdx>, next_id: Arc<std::sync::atomic::AtomicU64>, }` | — | `src/session_dag.rs` |
| `SessionDagStepOptions` | `{ pub prompt: String, pub response: String, pub snapshot_tx_id: Option<TxId>, pub tool_outputs: Vec<String>, pub label: &'a str, }` | — | `src/session_dag.rs` |
| `ShadowComparison` | `{ /// Maximum absolute score difference across top-k union nodes. pub max_abs_diff: f32, /// Jaccard overlap ratio between top-k entity ID sets [0.0, 1.0]. pub t… }` | — | `src/tl_hfd/shadow.rs` |
| `ShadowDiscrepancy` | `{ /// Das Ergebnis des etablierten Baseline-Algorithmus. pub baseline: T, /// Das Ergebnis des neuen Kandidaten-Algorithmus. pub candidate: T, /// Eindeutige Kon… }` | Diskrepanz-Eintrag: Baseline-Ergebnis vs. Kandidaten-Ergebnis (§12). | `src/ppr/shadow_hook.rs` |
| `ShadowDiscrepancyReport` | `{ /// Number of evaluated query pairs (ForwardPush vs TL-HFD). pub sample_count: u64, /// Mean Jaccard similarity of top-k entity ID sets between algorithms. pub… }` | — | `src/tl_hfd/shadow.rs` |
| `SnapshotPathGraph` | `{ inner: &'a GraphInner, as_of_tx: TxId, }` | — | `src/path_rag/snapshot.rs` |
| `StarExpansionIterator` | `{ inner: arc_swap::Guard<std::sync::Arc<crate::csr::inner::GraphInner>>, hyperedge_keys: Vec<crate::hyperedge::HyperEdgeId>, current_hyperedge_idx: usize, curren… }` | — | `src/community.rs` |
| `TlHfdFlipGate` | `(tuple/unit)` | — | `src/tl_hfd/shadow.rs` |
| `TlHfdParams` | `{ /// Regularization scale. pub sigma: f32, /// Seed injection multiplier. pub delta: f32, /// Boundary conductance exponent. pub gamma: f32, /// Maximum number… }` | Thresholded local high-frequency diffusion parameters. | `src/types.rs` |
| `TlHfdParams` | `{ /// Regularization parameter $\sigma > 0$ controlling diffusion scale (default: 0.1). pub sigma: f32, /// Seed injection multiplier $\delta \ge 2.0$ (default:… }` | — | `src/tl_hfd/params.rs` |
| `TraversalSignal` | `{ pub from: EntityId, pub to: EntityId, pub path_length: usize, }` | — | `src/edge_reinforcement_buffer.rs` |
| `TxId` | `(tuple/unit)` | Transaction or sequence identifier. | `src/types.rs` |
| `ValidatedHyperEdgeCandidate` | `{ /// The underlying co-occurrence candidate. pub candidate: HyperEdgeCandidate, /// Suggested semantic predicate string. pub predicate: String, /// LLM validati… }` | — | `src/hyperedge_suggest.rs` |
| `VirtualHyperedgeNode` | `{ /// ID der zugrundeliegenden Hyperkante. pub hyperedge_id: crate::hyperedge::HyperEdgeId, }` | — | `src/community.rs` |

### Öffentliche Datentypen — Enums (10)

| Enum | Varianten | Beschreibung | Datei |
|---|---|---|---|
| `ApprhError` | `/// Invalid parameter value provided to APPRH. #[error("Invalid parameter: {0}")] InvalidParameter(String), /// Empty seed list provided to APPRH. #[error("Seed list cannot be empty")] EmptySeeds, //…` | — | `src/apprh/error.rs` |
| `ApprhMode` | `/// APPRH disabled; authoritative Forward-Push PPR is returned. Off, /// APPRH shadow mode (default): returns authoritative Forward-Push PPR result, /// and periodically evaluates shadow comparisons…` | — | `src/apprh/selector.rs` |
| `CascadeStatus` | `/// Invalidation task is queued and pending. Pending, /// Invalidation task is currently processing. InFlight, /// Invalidation completed successfully with a deletion proof. Completed, /// Invalidati…` | — | `src/cascade.rs` |
| `EdgeType` | `#[default] Default,` | — | `src/csr/types.rs` |
| `Error` | `#[error("ungültige Eingabedaten: {0}")] InvalidInput(&'static str), #[error("Kapazitätsgrenze überschritten: gefordert {requested}, maximal {maximum}")] CapacityExceeded { requested: usize, maximum:…` | — | `src/contract.rs` |
| `GraphError` | `/// Invalid input parameter or argument. #[error("Invalid input: {0}")] InvalidInput(String), /// Requested node, edge, or hyperedge not found. #[error("Not found: {0}")] NotFound(String), /// Graph…` | Canonical error taxonomy for `contextra-graph`. | `src/error.rs` |
| `GraphMutationError` | `/// Lock acquisition timed out. #[error("Lock acquisition timed out: {0}")] LockAcquisitionTimeout(String), /// Invalid role binding for hyperedge. #[error("Invalid role binding: {0}")] RoleBindingIn…` | Detailed errors occurring during graph mutations. | `src/error.rs` |
| `HyperedgeSuggestError` | `/// Candidate count exceeds the maximum LLM calls allowed per cycle. #[error("LLM validation budget exceeded: {0} candidates exceed limit")] LlmValidationBudgetExceeded(usize),` | — | `src/hyperedge_suggest.rs` |
| `PprAlgorithm` | `/// Auto heuristic dispatch. #[default] Auto, /// Dense power-iteration algorithm. DensePowerIteration, /// Andersen-Chung-Lang forward push approximation. ForwardPush, /// Shadow mode comparing Forw…` | Execution algorithm variant for Personalized PageRank. | `src/types.rs` |
| `TlHfdError` | `/// Invalid parameter value provided to TL-HFD. #[error("Invalid parameter: {0}")] InvalidParameter(String), /// Non-finite value encountered during diffusion calculation. #[error("Non-finite value e…` | — | `src/tl_hfd/error.rs` |

### Typ-Aliase (8)

| Alias | Definition | Beschreibung |
|---|---|---|
| `CascadeReport` | `type CascadeReport= HyperedgeCascadeReport` | — |
| `EdgeId` | `type EdgeId= (EntityId, EntityId)` | — |
| `NodeId` | `type NodeId= u64` | — |
| `NodeIdx` | `type NodeIdx= u64` | — |
| `Result` | `type Result<T> = core::result::Result<T, Error>` | — |
| `Result` | `type Result<T, E = GraphError> = std::result::Result<T, E>` | Crate-level Result type. |
| `StorageFuture` | `type StorageFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>` | Boxed future for async storage methods. |
| `Weight` | `type Weight= f32` | — |

### Konstanten (18)

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
| `MAX_SEARCH_K` | `usize` | Maximum candidates in search results. |
| `MAX_TRAVERSAL_HOPS` | `u8` | — |
| `MAX_VISITED_NODES` | `usize` | — |
| `MIN_AGREEMENT_THRESHOLD` | `f32` | — |
| `MIN_SHADOW_SAMPLES` | `u64` | — |
| `SCORE_DECAY` | `f32` | — |
| `WALLCLOCK_TX_HEURISTIC_MIN` | `u64` | — |

### Öffentliche Funktionen (254)

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
| `pub fn personalized_page_rank_at<'a>( &'a self, seed_nodes: &'a [contextra_types::EntityId], config: &'a contextra_types::PprConfig, seq_no: u64, ) -> contextra_ports::BoxFuture< 'a, contextra_types::Result<Vec<(contextra_types::EntityId, f32)>>, >` | Delegiert an [`GraphIndex::personalized_page_rank_at`]. | `src/csr/graph_write/mod.rs` |
| `pub fn rollback_to_tx<'a>( &'a self, tx_id: contextra_types::TxId, ) -> contextra_ports::BoxFuture<'a, contextra_types::Result<()>>` | Delegiert an [`GraphIndex::rollback_to_tx`]. | `src/csr/graph_write/mod.rs` |
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
| `pub fn add_edge(&mut self, source: EntityId, target: EntityId, weight: f32)` | — | `src/provider.rs` |
| `pub fn add_hyperedge(&mut self, hyperedge: HyperEdge)` | — | `src/provider.rs` |
| `pub fn add_node(&mut self, node: EntityId)` | — | `src/provider.rs` |
| `pub fn compute_pagerank<G: GraphProvider>( graph: &G, damping: f32, max_iter: usize, tol: f32, ) -> HashMap<EntityId, f32>` | Generic PageRank computation executing against any `GraphProvider`. | `src/provider.rs` |
| `pub fn detect_communities_on_provider<G: GraphProvider>( graph: &G, config: &crate::community::CommunityDetectionConfig, ) -> Result<Vec<crate::community::CommunityAssignment>>` | Generic Community Detection on any `GraphProvider`. | `src/provider.rs` |
| `pub fn new() -> Self` | — | `src/provider.rs` |
| `pub fn graph_purger(graph: Arc<CsrGraph>) -> Arc<dyn Purger>` | — | `src/purger.rs` |
| `pub fn new(graph: Arc<CsrGraph>) -> Self` | Erzeugt einen neuen [`GraphPurger`]. | `src/purger.rs` |
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
| `pub fn new() -> Self` | Creates a new empty `InMemoryGraphStorage`. | `src/storage.rs` |
| `pub fn new() -> Self` | Creates a new `FaultVfs`. | `src/test_doubles.rs` |
| `pub fn new() -> Self` | Creates a new empty `InMemoryStorageEngine`. | `src/test_doubles.rs` |
| `pub fn read_file(&self, path: &std::path::Path) -> std::result::Result<Vec<u8>, String>` | Reads data from a virtual path. | `src/test_doubles.rs` |
| `pub fn reset_counters(&self)` | Resets fault counters and failure state. | `src/test_doubles.rs` |
| `pub fn set_config(&self, config: FaultConfig)` | Sets the fault injection config. | `src/test_doubles.rs` |
| `pub fn sync(&self) -> std::result::Result<(), String>` | Flushes virtual files to disk. | `src/test_doubles.rs` |
| `pub fn trigger_crash(&self)` | Triggers an immediate crash state. | `src/test_doubles.rs` |
| `pub fn write_file( &self, path: &std::path::Path, data: &[u8], ) -> std::result::Result<(), String>` | Writes data to a virtual path. | `src/test_doubles.rs` |
| `pub fn run_diffusion<G: PathGraph>( graph: &G, seeds: &[EntityId], params: &TlHfdParams, ) -> Result<AHashMap<EntityId, f32>, TlHfdError>` | — | `src/tl_hfd/diffusion.rs` |
| `pub fn compute_lovasz_extension( participants: &[EntityId], get_x: impl Fn(EntityId) -> f32, ) -> (f32, Option<EntityId>, Option<EntityId>)` | — | `src/tl_hfd/lovasz.rs` |
| `pub fn truncate_participants( participants: &[EntityId], get_x: impl Fn(EntityId) -> f32, max_sort_size: usize, ) -> Vec<EntityId>` | — | `src/tl_hfd/lovasz.rs` |
| `pub fn thresholded_local_hfd( &self, seeds: &[EntityId], params: &TlHfdParams, ) -> Result<AHashMap<EntityId, f32>, TlHfdError>` | — | `src/tl_hfd/mod.rs` |
| `pub fn tl_hfd_local<G: PathGraph>( graph: &G, seeds: &[EntityId], params: &TlHfdParams, ) -> Result<AHashMap<EntityId, f32>, TlHfdError>` | — | `src/tl_hfd/mod.rs` |
| `pub fn validate(&self) -> Result<(), TlHfdError>` | — | `src/tl_hfd/params.rs` |
| `pub fn shadow_compare_forward_push_vs_tl_hfd<G: PathGraph>( graph: &G, seeds: &[EntityId], ppr: &PprParams, tl: &TlHfdParams, top_k: usize, ) -> Result<ShadowComparison, TlHfdError>` | — | `src/tl_hfd/shadow.rs` |
| `pub fn from_key(key: impl AsRef<[u8]>) -> Result<Self, std::num::ParseIntError>` | Creates an entity id from a string slice deterministically. | `src/types.rs` |
| `pub fn from_key(key: impl AsRef<[u8]>) -> Result<Self, std::num::ParseIntError>` | Derives a `DocId` deterministically from a key slice using BLAKE3. | `src/types.rs` |
| `pub fn is_exhausted(&self) -> bool` | Checks if budget is exhausted. | `src/types.rs` |
| `pub fn new(id: EntityId, name: impl Into<String>, entity_type: impl Into<String>) -> Self` | Creates a new `Entity`. | `src/types.rs` |
| `pub fn new(from: EntityId, to: EntityId, label: impl Into<String>) -> Self` | Creates a new `Edge`. | `src/types.rs` |
| `pub fn new() -> Self` | Creates a new default request context. | `src/types.rs` |
| `pub fn record_allocation(&mut self, bytes: usize)` | Records consumed bytes. | `src/types.rs` |
| `pub fn unlimited() -> Self` | Unlimited tracker. | `src/types.rs` |
| `pub fn with_limit(max_bytes: usize) -> Self` | Tracker with byte limit. | `src/types.rs` |
| `pub fn with_source_doc_id(mut self, doc_id: DocId) -> Self` | Sets source doc id. | `src/types.rs` |
| `pub fn with_tenant(tenant: impl Into<String>) -> Self` | Creates a request context with tenant identifier. | `src/types.rs` |
| `pub fn with_weight(mut self, weight: f32) -> Self` | Sets edge weight. | `src/types.rs` |


---

## `contextra-rank`

| Attribut | Wert |
|---|---|
| **Ring** | 0 — Ring 0 — Foundation & Pure Numerics/Types |
| **Status** | In Migration |
| **Zweck** | 4-Signal-Retrieval-Fusion, Score-Kalibrierung und Drift-Messung. |
| **Unsafe-Code** | Nein (`#![forbid(unsafe_code)]`) |
| **Interne Abhängigkeiten** | `contextra-ports`, `contextra-types` |
| **Externe Abhängigkeiten (Auswahl)** | `ahash`, `serde`, `serde_json`, `thiserror`, `tracing` |

### Architekturbeschreibung

# Contextra Rank (`contextra-rank`)  **Ring:** Ring 0 (Ranking & Kalibrierung / Kernlogik) **Klasse:** Rand `[R]` **Reifegrad:** Experimentell / Aktiv (Kern v1)  ## Zweck des Crates Dieses Crate implementiert Methoden zur Fusion, Kalibrierung und Erklärung von Retrieval-Ergebnissen im Contextra-System. Dazu gehören: - **4-Signal-Fusion:** Verknüpfung von Vektor-, Text- (BM25), Graph- und Rerank-Scores mittels reziproker Rangfusion (RRF = Reciprocal Rank Fusion) und score-basierter Normalisierung. - **Kalibrierung:** Umwandlung von Roh-Scores in Wahrscheinlichkeiten via Platt-Scaling (logistische Regression) und Isotonische Regression (PAVA = Pool-Adjacent Violators Algorithm). - **Konforme Prädiktion:** Finite-Sample-Abdeckungsgarantien und adaptive Schwellenwertanpassung bei Verteilungsverschiebung (Covariate Shift). - **Drift-Erkennung:** Überwachung von Score-Verteilungen zur Laufzeit. - **Erklärbarkeit (Explainability):** Umwandlung maschinenlesbarer Herkunftsnachweise (`ProvenanceRecord`) in strukturierte und deutschsprachige Erklärungen. - **DiBud (Discrete Budgeted):** Jederzeit abbrechbare Streaming-Fusion unter Zeit- und Kanalbudgets.  ## Einbindung und Abhängigkeiten - **Konsumiert (Ring 0):** `contextra-types` (Domain-Typen), `contextra-ports` (Hardware-Abstraktionen). - **Wird eingebunden von:** `contextra`, `contextra-db`, `contextra-engine`, `contextra-infer-candle`, `contextra-infer-ollama`, `contextra-infer-onnx`, `contextra-mcp`, `contextra-py`.

### Modul- und Re-Export-Oberfläche (lib.rs)

```rust
pub mod calibration;
pub mod drift;
pub mod error;
pub mod explain;
pub mod fusion;
pub mod local_types;
pub mod metrics;
pub mod dibud;
pub use error::{RankError, Result};
pub use local_types::{ ConfigFingerprint, DocId, FusionStrategy, FusionWeights, SignalFailurePolicy, SignalFusionStrategies, MAX_SEARCH_K, };
pub use calibration::{ calibrate_score, AdaptiveConformalCalibrator, Calibrator, Clock, ConformalCalibrator, ConformalError, IsotonicCalibrator, PlattScaler, SystemClock, };
pub use dibud::{ fuse_exact_prefix, fuse_exact_prefix_async, BudgetedChannel, DiBudFusionState, DiBudOutcome, DiBudStep, FusionBudget, };
pub use drift::{DriftDetector, DriftStatus};
pub use explain::{explain, ExplanationEntry, RetrievalExplanation};
pub use fusion::{ apply_resonance_bonus, fuse_search_results_with_strategy, g, modulate_and_renormalize_weights, reciprocal_rank_fusion, rrf_fuse, score_normalized_fusion_with_options, weighted_reciprocal_rank_fusion, weighted_reciprocal_rank_fusion_mrrf, weighted_reciprocal_rank_fusion_with_options, weighted_reciprocal_rank_fusion_with_priority, weights_to_signal_factors, BoundedTopK, GlobalFusionConfig, GlobalFusionStrategy, MetadataMergePriority, ProvenanceBuilder, ProvenanceRecord, ResonanceConfig, ScoredItem, SearchResult, SignalCalibrationContext, SignalContribution, SignalKind, };
pub use metrics::{mean_reciprocal_rank, ndcg_at_k, precision_at_k, recall_at_k, reciprocal_rank};
```

### Feature-Flags

| Flag | Aktiviert |
|---|---|
| `default` | `["std", "types-compat"]` |
| `std` | `[]` |
| `types-compat` | `["dep:contextra-types", "dep:contextra-ports"]` |
| `dibud` | `[]` |

### Schnittstellen — Public Traits (2)

**`Clock`** — *Lokaler Clock-Port für deterministische Zeitstempel (B11-Konformität).* (`src/calibration/isotonic.rs`)

```rust
pub trait Clock : Send + Sync {
/// Gibt den aktuellen UNIX-Zeitstempel in Nanosekunden zurück.
    fn now_unix_nanos(&self) -> u64;

    /// Gibt eine monotone Zeit in Nanosekunden zurück.
    fn monotonic_nanos(&self) -> u64 {
        self.now_unix_nanos()
    }
}
```

**`ConformalCalibrator`** — *Trait for deterministic, online conformal calibration and threshold estimation. Trait für deterministische konforme Schwellenwertschätzung.* (`src/calibration/conformal.rs`)

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

### Öffentliche Datentypen — Structs (25)

| Struct | Felder / Form | Beschreibung | Datei |
|---|---|---|---|
| `AdaptiveConformalCalibrator` | `{ alpha: f32, threshold: f32, learning_rate: f32, min_threshold: f32, max_threshold: f32, observation_count: u64, fingerprint: Option<ConfigFingerprint>, }` | Adaptive Conformal Calibrator for quantile-based online threshold estimation under covariate shift. Implements online stochastic gradient updates on… | `src/calibration/conformal.rs` |
| `BoundedTopK` | `{ heap: std::collections::BinaryHeap<T>, capacity: usize, }` | Size-bounded min-heap for Top-K candidate selection during Reciprocal Rank Fusion. # Abgrenzung Dieser Typ (`contextra_rank::fusion::BoundedTopK`) di… | `src/fusion/topk.rs` |
| `ConfigFingerprint` | `{ /// Identifikator oder Hash der Modell-/Promptkonfiguration. pub hash: [u8; 32], }` | Konfigurations-Fingerprint zur Validierung und Cache-Invalidierung. | `src/local_types.rs` |
| `DiBudFusionState` | `{ channel_depths: [usize; 3], exhausted: [bool; 3], accesses: usize, observed_docs: AHashMap<DocId, ObservedDocState>, er_cache: AHashMap<DocId, f32>, provenance… }` | Core state machine tracking channel depths, observed document scores, and prefix certification. Zustandsautomat für die budgetierte Streaming-Rangfus… | `src/dibud/state.rs` |
| `DiBudOutcome` | `{ /// Ranked candidate document IDs sorted descending by lower bound $F^-(x)$ (tie-breaking by ascending `DocId`). pub ranked: Vec<DocId>, /// Number of prefix e… }` | Final outcome of a DiBud fusion execution. Endergebnis der DiBud-Fusion inklusive zertifiziertem Präfix. | `src/dibud/types.rs` |
| `DocId` | `(tuple/unit)` | Opake Dokumentidentität für `contextra-rank`. | `src/local_types.rs` |
| `DriftDetector` | `{ window_size: usize, threshold: f32, baseline_mean: Option<f32>, recent_scores: VecDeque<f32>, }` | Drift detector monitoring score shifts over sliding windows. Detektor zur Überwachung von Score-Verschiebungen über ein gleitendes Fenster. | `src/drift.rs` |
| `ExplanationEntry` | `{ /// Identified search signal type. pub signal: SignalKind, /// Raw unnormalized score or distance. pub raw_score: f32, /// 1-based rank in the signal's result… }` | Breakdown entry representing the contribution of a single search signal. Einzelner Beitrags-Eintrag für eine spezifische Signalquelle innerhalb der E… | `src/explain.rs` |
| `FusionBudget` | `{ /// Maximum total channel access budget (reads). pub max_total_accesses: usize, /// Minimum required certified prefix length before stopping. pub min_certified… }` | Dynamic budget configuration for DiBud fusion. Ressourcen- und Abbruchbudget für die Streaming-Rangfusion. | `src/dibud/types.rs` |
| `FusionWeights` | `{ /// Vektor-Gewicht. pub vector: f32, /// Text-Gewicht. pub text: f32, /// Graph-Gewicht. pub graph: f32, /// Zusätzliche Gewichte. pub metadata: f32, }` | Fusionsgewichte für die Normalisierung. | `src/local_types.rs` |
| `GlobalFusionConfig` | `{ /// Maximum number of community nodes to include in the final result set. pub max_community_nodes: Option<usize>, /// Minimum size (member count) required for… }` | Configuration for `GlobalFusionStrategy`. Konfigurationsparameter für die globale Community-Fusionsstrategie. | `src/fusion/global.rs` |
| `GlobalFusionStrategy` | `{ config: GlobalFusionConfig, }` | Community-aware fusion strategy for corpus-wide topic retrieval (LeanRAG-backed). Globale Fusionsstrategie für hierarchische und korpusweite Retrieva… | `src/fusion/global.rs` |
| `IsotonicCalibrator` | `{ observations: VecDeque<(f32, bool)>, warmup_required: u32, max_observations: usize, cached_model: Option<Vec<(f32, f32)>>, // (max_score_in_block, calibrated_p… }` | Isotonic Calibrator using Pool-Adjacent Violators Algorithm. Nicht-parametrischer Kalibrierer auf Basis von PAVA. | `src/calibration/isotonic.rs` |
| `PlattScaler` | `{ a: f32, b: f32, fingerprint: Option<ConfigFingerprint>, }` | Platt Scaler for parametric logit/score calibration. Parametrischer Kalibrierer nach Platt (1999). | `src/calibration/platt.rs` |
| `ProvenanceBuilder` | `{ vector_distance: Option<f32>, vector_rank: Option<u32>, vector_weight: Option<f32>, bm25_score: Option<f32>, bm25_rank: Option<u32>, text_weight: Option<f32>,… }` | Fluent builder for constructing `ProvenanceRecord` instances. Fluent-Builder zum deterministischen Erstellen von `ProvenanceRecord`-Strukturen. | `src/fusion/provenance.rs` |
| `ProvenanceRecord` | `{ /// Distance score from vector search signal. #[serde(default, skip_serializing_if = "Option::is_none")] pub vector_distance: Option<f32>, /// BM25 score from… }` | Provenance record for fused search result. Auditierbarer Herkunftsnachweis (Provenance) über alle beteiligten Retrieval-Kanäle. | `src/fusion/types.rs` |
| `ResonanceConfig` | `{ /// Exponent β for coherence bonus. Default: 0.5. pub beta: f32, /// Boost strength γ. Default: 0.3. pub gamma: f32, }` | Configuration for Resonance Coherence Bonus. Konfigurationsparameter für die Berechnung des Resonanz-Kohärenz-Bonus. | `src/fusion/resonance.rs` |
| `RetrievalExplanation` | `{ /// Individual signal contribution entries, sorted descending by `contribution_share`. pub entries: Vec<ExplanationEntry>, /// Dominant search signal contribut… }` | Structured explanation detailing why a document was retrieved with its given rank. Strukturierte Gesamterklärung zur Nachvollziehbarkeit eines Retrie… | `src/explain.rs` |
| `ScoredItem` | `{ /// Eindeutiger Dokument- oder Elementbezeichner. pub id: String, /// Punktwert / Ähnlichkeitswert / Fusionswert. pub score: f32, /// Optionaler strukturierter… }` | Crate-lokaler Typ für gerankte/fusionierte Elemente. | `src/fusion/types.rs` |
| `SearchReport` | `{ /// List of signals that failed during query execution when using Degrade policy. pub degraded_signals: Vec<Signal>, /// Warnings captured during query process… }` | Execution report for search query execution. Detaillierter Audit-Report über den Fusions- und Suchablauf. | `src/fusion/types.rs` |
| `SearchResult` | `{ /// Document or memory entry identifier. pub id: String, /// Fused score value. pub score: f32, /// Document metadata payload. pub metadata: Option<serde_json:… }` | Individual search result candidate for fusion. Standard-Suchergebnis eines einzelnen Retrieval-Signals oder einer Fusionsstufe. | `src/fusion/types.rs` |
| `SignalCalibrationContext` | `{ /// Signal identifier matching the result set signal name (e.g., "vector", "text", "graph"). pub signal_name: String, /// Indicates whether the calibrator for… }` | Calibration and drift context for a search signal during mRRF weight modulation. Kontext zu Modellkalibrierung und Driftzustand eines Signals für die… | `src/fusion/signal.rs` |
| `SignalContribution` | `{ /// Raw score before normalization/fusion. pub raw_score: f32, /// 1-based rank in the signal's result list. pub rank: u32, /// Calculated RRF or normalized co… }` | Signal contribution details. Beitrag eines einzelnen Signals zum aggregierten RRF-Score. | `src/fusion/types.rs` |
| `SignalFusionStrategies` | `{ /// Strategie für Vektor. pub vector: FusionStrategy, /// Strategie für Text. pub text: FusionStrategy, /// Strategie für Graph. pub graph: FusionStrategy, }` | Signal-Fusionsstrategien pro Kanal. | `src/local_types.rs` |
| `SystemClock` | `(tuple/unit)` | Standard SystemClock basierend auf std::time::SystemTime. | `src/calibration/isotonic.rs` |

### Öffentliche Datentypen — Enums (11)

| Enum | Varianten | Beschreibung | Datei |
|---|---|---|---|
| `BudgetedChannel` | `/// Vector (semantic) search channel (index 0). Vector = 0, /// Text (BM25) search channel (index 1). Text = 1, /// Graph search channel (index 2). Graph = 2,` | Identifies the channels managed by DiBud fusion budget. Order of variants is defined such that `Vector < Text < Graph`, matching the tie-breaking pri… | `src/dibud/types.rs` |
| `Calibrator` | `/// Platt-Scaling (logistische Regression) mit Format-Version 1. #[serde(rename = "platt_v1")] Platt { /// Format-Versionsfeld für defensive Dekodierung. format_version: u8, /// Platt-Scaler-Instanz.…` | Versionierter Container für Kalibrierungsmodelle (Klassenvertrag RANK-1). | `src/calibration/mod.rs` |
| `ConformalError` | `/// Target miscoverage level `alpha` is not finite or outside (0.0, 1.0). #[error("Invalid target miscoverage alpha: {0} (must be finite and in (0.0, 1.0))")] Alpha(f32), /// Propensity weight is not…` | Error type for conformal calibration and threshold updates. Fehlertypen für die konforme Kalibrierung. | `src/calibration/conformal.rs` |
| `DiBudStep` | `/// Request reading the next item from the specified channel. Poll(BudgetedChannel), /// Fusion is complete; outcome contains certified prefix and ranked documents. Done(DiBudOutcome),` | Represents the step action requested by `DiBudFusionState::next_request`. Nächste durchzuführende Aktion im DiBud-Zustandsautomaten. | `src/dibud/state.rs` |
| `DriftStatus` | `/// Distribution is stable. Stable { /// Mean score shift relative to baseline. mean_shift: f32, }, /// Drift detected exceeding configured tolerance threshold. DriftDetected { /// Mean score shift r…` | Drift status summary representation. Zusammenfassung des aktuellen Drift-Status einer Signalverteilung. | `src/drift.rs` |
| `FusionStrategy` | `/// Standard Reciprocal Rank Fusion mit k=60. #[default] Rrf, /// Score-normalisierte gewichtete lineare Kombination. ScoreNormalized,` | Fusionsstrategie für hybride Suchen. | `src/local_types.rs` |
| `MetadataMergePriority` | `/// Vector metadata is processed first (default behavior). Order: Vector, Text, Graph. #[default] VectorFirst, /// Text metadata is processed first. Order: Text, Vector, Graph. TextFirst, /// Graph m…` | Configures signal priority order for metadata merging during Reciprocal Rank Fusion. Prioritätsstrategie bei der Zusammenführung kollidierender Metad… | `src/fusion/signal.rs` |
| `RankError` | `/// Ungültige Eingabedaten oder Konfiguration. #[error("invalid input: {0}")] InvalidInput(String), /// Fehler bei der Kalibrierung oder Wahrscheinlichkeitsberechnung. #[error("calibration error: {0}…` | Fehlerzustände von `contextra-rank`. | `src/error.rs` |
| `Signal` | `/// Dense vector similarity signal. Vector, /// Full-text BM25 keyword signal. Text, /// Graph entity relationship signal. Graph,` | Query retrieval signal identifier. # Abgrenzung Dieser Typ (`contextra_rank::fusion::Signal`) bezeichnet die spezifische Suchsignal-Art (z. B. `Vecto… | `src/fusion/types.rs` |
| `SignalFailurePolicy` | `/// Bricht die Suche ab. #[default] Fail, /// Setzt die Suche mit reduzierten Signalen fort. Degrade,` | Richtlinie bei Signal-Fehlern. | `src/local_types.rs` |
| `SignalKind` | `/// Vector (semantic k-NN) search signal. Vector, /// Text (BM25 keyword) search signal. Text, /// Graph (traversal / PageRank) search signal. Graph, /// Edge-reinforcement weight signal / Bandit rec…` | Identifies the kind of search signal used during fusion. Klassifikation der unterschiedlichen Retrieval-Signalarten im Contextra-System. | `src/fusion/signal.rs` |

### Typ-Aliase (2)

| Alias | Definition | Beschreibung |
|---|---|---|
| `FusedScore` | `type FusedScore= SearchResult` | Type alias for fused score output item. Aggregierter Fusionswert eines Dokuments (Typ-Alias auf SearchResult). |
| `Result` | `type Result<T> = core::result::Result<T, RankError>` | Standard-Result-Typ für `contextra-rank`. |

### Konstanten (6)

| Konstante | Typ | Beschreibung |
|---|---|---|
| `DEFAULT_INITIAL_THRESHOLD` | `f32` | Default initial threshold $\hat{q}_0$. |
| `DEFAULT_LEARNING_RATE` | `f32` | Default learning rate $\gamma$ for online quantile updates. |
| `DEFAULT_MAX_THRESHOLD` | `f32` | Default maximum threshold bound. |
| `DEFAULT_MIN_THRESHOLD` | `f32` | Default minimum threshold bound. |
| `MAX_SEARCH_K` | `usize` | Maximale Anzahl an Suchergebnissen für Top-K-Buffer. |
| `ZERO` | `Self` | Null-Fingerprint. |

### Öffentliche Funktionen (110)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn alpha(&self) -> f32` | Returns the target error rate $\alpha$. | `src/calibration/conformal.rs` |
| `pub fn invalidate_on_config_change(&mut self, new_fingerprint: impl Into<ConfigFingerprint>)` | Resets state if ConfigFingerprint changes (P8 Compliance). | `src/calibration/conformal.rs` |
| `pub fn learning_rate(&self) -> f32` | Returns the learning rate $\gamma$. | `src/calibration/conformal.rs` |
| `pub fn new(alpha: f32) -> Result<Self, ConformalError>` | Creates a new `AdaptiveConformalCalibrator` with target error rate `alpha` in `(0.0, 1.0)`. Uses default initial threshold (0.5) and defaul… | `src/calibration/conformal.rs` |
| `pub fn observation_count(&self) -> u64` | Returns the total number of observations processed. | `src/calibration/conformal.rs` |
| `pub fn with_bounds( mut self, min_threshold: f32, max_threshold: f32, ) -> Result<Self, ConformalError>` | Sets bounding constraints `[min_threshold, max_threshold]` for the estimated threshold. | `src/calibration/conformal.rs` |
| `pub fn with_params( alpha: f32, initial_threshold: f32, learning_rate: f32, ) -> Result<Self, ConformalError>` | Creates an `AdaptiveConformalCalibrator` with custom parameters. | `src/calibration/conformal.rs` |
| `pub fn cached_ece(&self) -> Option<f32>` | Returns cached Expected Calibration Error (ECE). | `src/calibration/isotonic.rs` |
| `pub fn calibrated_probability(&mut self, raw_score: f32) -> Option<f32>` | Returns calibrated probability if warm, or `None` if below warmup requirement. Liefert die kalibrierte Wahrscheinlichkeit für einen Roh-Sco… | `src/calibration/isotonic.rs` |
| `pub fn expected_calibration_error(&mut self) -> Option<f32>` | Calculates Expected Calibration Error over 10 bins. | `src/calibration/isotonic.rs` |
| `pub fn force_rebuild(&mut self)` | Forces immediate PAVA model rebuild. | `src/calibration/isotonic.rs` |
| `pub fn invalidate_on_config_change(&mut self, new_fingerprint: impl Into<ConfigFingerprint>)` | Resets observations if ConfigFingerprint changes (P8 Compliance). | `src/calibration/isotonic.rs` |
| `pub fn is_calibrated(&self) -> bool` | Checks if warmup observations threshold has been reached. | `src/calibration/isotonic.rs` |
| `pub fn last_calibration_at(&self) -> Option<u64>` | Returns timestamp (UNIX epoch in seconds) of last model rebuild. | `src/calibration/isotonic.rs` |
| `pub fn new() -> Self` | Erzeugt eine neue `SystemClock`. | `src/calibration/isotonic.rs` |
| `pub fn new(warmup_required: u32, max_observations: usize) -> Self` | Creates a new `IsotonicCalibrator`. | `src/calibration/isotonic.rs` |
| `pub fn observation_count(&self) -> usize` | Returns current observation count. | `src/calibration/isotonic.rs` |
| `pub fn record_outcome(&mut self, raw_score: f32, outcome: bool)` | Records a new observation `(raw_score, outcome)`. Non-finite `raw_score` values are safely ignored. Nimmt eine neue Beobachtung `(raw_score… | `src/calibration/isotonic.rs` |
| `pub fn with_clock(mut self, clock: Arc<dyn Clock>) -> Self` | Sets a custom clock port for deterministic time access. | `src/calibration/isotonic.rs` |
| `pub fn with_defaults() -> Self` | Creates an `IsotonicCalibrator` with defaults (Warmup: 50, Max Obs: 2000). | `src/calibration/isotonic.rs` |
| `pub fn with_ports_clock(mut self, clock: Arc<dyn contextra_ports::Clock>) -> Self` | Sets a custom clock port from `contextra-ports` (transitional compat). | `src/calibration/isotonic.rs` |
| `pub fn calibrate_score(raw: f32, calibrator: &Calibrator) -> Result<f32>` | Kalibriert einen Roh-Score anhand eines übergebenen Kalibrierers (Klassenvertrag RANK-1). # Fehler Gibt `RankError::InvalidInput` zurück, w… | `src/calibration/mod.rs` |
| `pub fn conformal(calibrator: AdaptiveConformalCalibrator) -> Self` | Erzeugt einen neuen `Calibrator` basierend auf einem `AdaptiveConformalCalibrator`. | `src/calibration/mod.rs` |
| `pub fn isotonic(calibrator: IsotonicCalibrator) -> Self` | Erzeugt einen neuen `Calibrator` basierend auf einem `IsotonicCalibrator`. | `src/calibration/mod.rs` |
| `pub fn platt(scaler: PlattScaler) -> Self` | Erzeugt einen neuen `Calibrator` basierend auf einem `PlattScaler`. | `src/calibration/mod.rs` |
| `pub fn apply(&self, logit: f32) -> f32` | Alias for `transform(logit)`. | `src/calibration/platt.rs` |
| `pub fn fit(observations: &[(f32, bool)]) -> Self` | Fits parameters `A` and `B` via Negative Log-Likelihood minimization with L2 regularization and target smoothing (Platt, 1999). Trainiert d… | `src/calibration/platt.rs` |
| `pub fn identity() -> Self` | Uncalibrated default fallback ($A=1.0, B=0.0$). | `src/calibration/platt.rs` |
| `pub fn invalidate_on_config_change(&mut self, new_fingerprint: impl Into<ConfigFingerprint>)` | Resets scaler to identity if ConfigFingerprint changes (P8 Compliance). | `src/calibration/platt.rs` |
| `pub fn is_fitted(&self) -> bool` | Checks if model has been fitted. | `src/calibration/platt.rs` |
| `pub fn is_identity(&self) -> bool` | Checks if this scaler is identical to uncalibrated default ($A=1.0, B=0.0$). | `src/calibration/platt.rs` |
| `pub fn new(a: f32, b: f32) -> Self` | Creates a new `PlattScaler` with specified parameters `a` and `b`. Falls back to identity if non-finite. | `src/calibration/platt.rs` |
| `pub fn params(&self) -> (f32, f32)` | Returns current parameters `(a, b)`. | `src/calibration/platt.rs` |
| `pub fn predict(&self, raw_score: f32) -> f32` | Calculates calibrated probability for raw score. | `src/calibration/platt.rs` |
| `pub fn transform(&self, logit: f32) -> f32` | Applies Platt Scaling `sigmoid(A * logit + B)`. Führt die logistische Transformation `sigmoid(A * logit + B)` aus. | `src/calibration/platt.rs` |
| `pub fn fuse_exact_prefix<I1, I2, I3>( mut state: DiBudFusionState, vector: &mut I1, text: &mut I2, graph: &mut I3, edge_reinforcement: impl Fn(DocId) -> f32, budget: &FusionBudget, ) -> Result<DiBudOutcome, RankError> where I1: Iterator<Item = DocId>, I2: Ite…` | Synchronously executes DiBud fusion using three channel iterators. Führt die DiBud-Streaming-Fusion synchron über Iterator-Kanäle aus. | `src/dibud/driver.rs` |
| `pub async fn fuse_exact_prefix_async<P, Fut>( mut state: DiBudFusionState, mut poll: P, edge_reinforcement: impl Fn(DocId) -> f32, budget: &FusionBudget, ) -> Result<DiBudOutcome, RankError> where P: FnMut(BudgetedChannel) -> Fut, Fut: Future<Output = Result<…` | Asynchronously executes DiBud fusion using a channel poll closure returning a std `Future`. This function relies purely on `core::future::F… | `src/dibud/driver.rs` |
| `pub fn accesses(&self) -> usize` | Returns total read accesses performed. | `src/dibud/state.rs` |
| `pub fn channel_depths(&self) -> [usize` | Returns current channel depths `[Vector, Text, Graph]`. | `src/dibud/state.rs` |
| `pub fn exhausted(&self) -> [bool` | Returns exhausted status for each channel `[Vector, Text, Graph]`. | `src/dibud/state.rs` |
| `pub fn feed( &mut self, ch: BudgetedChannel, item: Option<DocId>, er: impl Fn(DocId) -> f32, budget: &FusionBudget, ) -> Result<(), RankError>` | Feeds the result of reading a channel into the state machine. Raw scores are unknown in pure `DocId` streams and are set to 0.0 in `Provena… | `src/dibud/state.rs` |
| `pub fn new(capacity: usize) -> Self` | Creates a new `DiBudFusionState` with pre-allocated map capacities. | `src/dibud/state.rs` |
| `pub fn next_request(&self, budget: &FusionBudget) -> DiBudStep` | Determines whether fusion is done or returns the next channel to poll. Bestimmt die nächste Abfrageaktion oder schließt die Fusion ab. | `src/dibud/state.rs` |
| `pub fn provenance_of(&self, doc_id: &DocId) -> Option<&ProvenanceRecord>` | Returns reference to provenance record for a given document if present. | `src/dibud/state.rs` |
| `pub fn validate(&self) -> Result<(), RankError>` | Validates budget parameter constraints. Ensures all weights are finite and non-negative, and minimum accesses/certified lengths are at leas… | `src/dibud/types.rs` |
| `pub fn analyze(&self) -> DriftStatus` | Analyzes current observations against baseline. | `src/drift.rs` |
| `pub fn as_str(&self) -> &'static str` | Returns human-readable summary string ("stabil", "warnung", "kritisch", or "unbekannt"). | `src/drift.rs` |
| `pub fn new(window_size: usize, threshold: f32) -> Self` | Creates a new `DriftDetector` with window size and sensitivity threshold. | `src/drift.rs` |
| `pub fn observe(&mut self, score: f32) -> DriftStatus` | Records a new score observation and calculates updated drift status. | `src/drift.rs` |
| `pub fn overall_drift_status(&self) -> &'static str` | Returns human-readable overall drift status string. | `src/drift.rs` |
| `pub fn set_baseline(&mut self, baseline: f32)` | Sets explicit baseline mean score. | `src/drift.rs` |
| `pub fn with_defaults() -> Self` | Creates a `DriftDetector` with default parameters (window size 50, threshold 0.15). | `src/drift.rs` |
| `pub fn explain(record: &ProvenanceRecord) -> RetrievalExplanation` | Generates a structured `RetrievalExplanation` from a `ProvenanceRecord`. Computes per-signal `contribution_share` normalized such that all… | `src/explain.rs` |
| `pub fn to_human_readable_de(&self) -> String` | Generates a human-readable German explanation string for audit logging or CLI output. Formats all signal contributions using exact ranks an… | `src/explain.rs` |
| `pub fn config(&self) -> &GlobalFusionConfig` | Returns a reference to the configuration. | `src/fusion/global.rs` |
| `pub fn fuse( &self, result_sets: Vec<(String, Vec<SearchResult>, f32)>, max_results: usize, ) -> Result<Vec<SearchResult>, RankError>` | Fuses multi-signal search results using community-aware RRF. Candidates are grouped by community ID (extracted from candidate metadata `"co… | `src/fusion/global.rs` |
| `pub fn new(config: GlobalFusionConfig) -> Self` | Creates a new `GlobalFusionStrategy` with the given configuration. | `src/fusion/global.rs` |
| `pub fn score_normalized_fusion_with_options( mut result_sets: Vec<(String, Vec<SearchResult>, f32)>, max_results: usize, priority: MetadataMergePriority, include_provenance: bool, resonance_config: Option<&ResonanceConfig>, ) -> Vec<SearchResult>` | Score-normalized fusion (CombSUM with MinMax normalization). Führt eine score-normalisierte lineare Fusion mehrerer Signal-Ergebnismengen d… | `src/fusion/normalized.rs` |
| `pub fn weights_to_signal_factors( weights: Option<&crate::local_types::FusionWeights>, ) -> (f32, f32, f32)` | Converts optional `FusionWeights` into (vector, text, graph) weight tuple. Konvertiert relative Signalgewichte in Normalisierungsfaktoren. | `src/fusion/normalized.rs` |
| `pub fn weights_to_signal_factors_types_compat( weights: Option<&contextra_types::FusionWeights>, ) -> (f32, f32, f32)` | Converts optional contextra_types::FusionWeights. | `src/fusion/normalized.rs` |
| `pub fn bm25(mut self, score: f32, rank: u32, weight: impl Into<Option<f32>>) -> Self` | Attaches BM25 text signal score, rank, and optional weight. | `src/fusion/provenance.rs` |
| `pub fn build(self) -> ProvenanceRecord` | Builds the resulting `ProvenanceRecord`. Schließt die Konstruktion ab und berechnet die RRF-Beiträge je Signal. | `src/fusion/provenance.rs` |
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
| `pub fn apply_resonance_bonus( results: Vec<SearchResult>, valid_signal_count: usize, config: &ResonanceConfig, ) -> Vec<SearchResult>` | Applies Resonance Coherence Bonus to fused search results. Wendet den Resonanz-Kohärenz-Bonus auf bereits fusionierte Suchergebnisse an. | `src/fusion/resonance.rs` |
| `pub fn fuse_signals( result_sets: Vec<(String, Vec<SearchResult>, f32)>, max_results: usize, ) -> Vec<FusedScore>` | Convenience API to fuse multi-signal search results. Komfortfunktion zur Zusammenführung mehrerer Signal-Ergebnismengen mittels gewichteter… | `src/fusion/resonance.rs` |
| `pub fn fuse_search_results_with_signal_strategies( mut result_sets: Vec<(String, Vec<SearchResult>, f32)>, max_results: usize, priority: MetadataMergePriority, include_provenance: bool, resonance_config: Option<&ResonanceConfig>, strategies: crate::local_type…` | Fuses search result sets using per-signal fusion strategies (`SignalFusionStrategies`). Führt Teilergebnisse mit individuellen Strategien j… | `src/fusion/rrf.rs` |
| `pub fn fuse_search_results_with_strategy( result_sets: Vec<(String, Vec<SearchResult>, f32)>, max_results: usize, priority: MetadataMergePriority, include_provenance: bool, resonance_config: Option<&ResonanceConfig>, strategy: FusionStrategy, ) -> Vec<SearchR…` | Fuses search result sets using the specified `FusionStrategy`. Universeller Einstiegspunkt zur Signal-Fusion basierend auf einer globalen `… | `src/fusion/rrf.rs` |
| `pub fn g(x: f32) -> f32` | Bounded smooth influence function $g(x) = \frac{x}{1 + |x|}$. Returns $x / (1 + |x|)$, bounded in $(-1.0, 1.0)$ for all finite $x$. Returns… | `src/fusion/rrf.rs` |
| `pub fn modulate_and_renormalize_weights( result_sets: &[(String, Vec<SearchResult>, f32)], calibration_contexts: Option<&[SignalCalibrationContext]>, beta: f32, ) -> Vec<f32>` | Modulates signal weights using mRRF margin formula: $m_s = S_1 - S_2$ (when signal is calibrated AND `drift_status` is `Stable` and $\ge 2$… | `src/fusion/rrf.rs` |
| `pub fn reciprocal_rank_fusion( result_sets: Vec<Vec<SearchResult>>, max_results: usize, ) -> Vec<SearchResult>` | Fuses multiple sets of ranked search results into a single ranked list using Reciprocal Rank Fusion (RRF). Führt mehrere ungewichtete Signa… | `src/fusion/rrf.rs` |
| `pub fn rrf_fuse(ranked_lists: &[Vec<ScoredItem>], k: usize) -> Result<Vec<ScoredItem>>` | Standard RRF-Fusion gemäß Klassenvertrag RANK-1: `rrf_fuse(ranked_lists: &[Vec<ScoredItem>], k: usize) -> Result<Vec<ScoredItem>>`. Berechn… | `src/fusion/rrf.rs` |
| `pub fn weighted_reciprocal_rank_fusion( result_sets: Vec<(String, Vec<SearchResult>, f32)>, max_results: usize, ) -> Vec<SearchResult>` | Weighted Reciprocal Rank Fusion with default signal metadata priority (`VectorFirst`). Gewichtete reziproke Rangfusion (Weighted RRF) mit S… | `src/fusion/rrf.rs` |
| `pub fn weighted_reciprocal_rank_fusion_mrrf( mut result_sets: Vec<(String, Vec<SearchResult>, f32)>, max_results: usize, priority: MetadataMergePriority, include_provenance: bool, resonance_config: Option<&ResonanceConfig>, calibration_contexts: Option<&[Sign…` | Weighted Reciprocal Rank Fusion with mRRF margin-based calibration weight modulation. Gewichtete Rangfusion mit margin-basierter mRRF-Gewic… | `src/fusion/rrf.rs` |
| `pub fn weighted_reciprocal_rank_fusion_with_options( result_sets: Vec<(String, Vec<SearchResult>, f32)>, max_results: usize, priority: MetadataMergePriority, include_provenance: bool, resonance_config: Option<&ResonanceConfig>, ) -> Vec<SearchResult>` | Weighted Reciprocal Rank Fusion with options. Vollständig konfigurierbare gewichtete Rangfusion mit Resonanz- und Provenance-Optionen. | `src/fusion/rrf.rs` |
| `pub fn weighted_reciprocal_rank_fusion_with_priority( result_sets: Vec<(String, Vec<SearchResult>, f32)>, max_results: usize, priority: MetadataMergePriority, ) -> Vec<SearchResult>` | Weighted Reciprocal Rank Fusion with explicit metadata merge priority. Gewichtete reziproke Rangfusion mit expliziter Metadaten-Zusammenfüh… | `src/fusion/rrf.rs` |
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
| `pub fn map_is_empty<K, V, S>(m: &ahash::AHashMap<K, V, S>) -> bool` | — | `src/fusion/types.rs` |
| `pub fn new(id: impl Into<String>, score: f32) -> Self` | Erzeugt ein neues `ScoredItem` ohne Provenienz. | `src/fusion/types.rs` |
| `pub fn synthesized_from(source_doc_ids: &[DocId]) -> Self` | Constructs a `ProvenanceRecord` synthesized from source document IDs. | `src/fusion/types.rs` |
| `pub fn synthesized_from_types(source_doc_ids: &[contextra_types::DocId]) -> Self` | Synthesized from contextra_types::DocId. | `src/fusion/types.rs` |
| `pub fn with_provenance(mut self, provenance: ProvenanceRecord) -> Self` | Fügt dem Element Provenienzinformationen hinzu. | `src/fusion/types.rs` |
| `pub fn as_bytes(&self) -> &[u8` | Gibt die rohen Bytes des Fingerprints zurück. | `src/local_types.rs` |
| `pub fn from_key(key: &str) -> Result<Self, String>` | Erzeugt eine `DocId` aus einer hexadezimalen oder numerischen Zeichenkette. | `src/local_types.rs` |
| `pub fn new( model_id: impl Into<String>, quantization: impl Into<String>, prompt_template: &str, temperature: f32, ) -> Self` | Erzeugt einen Fingerprint aus Modell, Quantisierung, Template und Temperatur. | `src/local_types.rs` |
| `pub fn strategy_for_signal(&self, signal_name: &str) -> FusionStrategy` | Gibt die Fusionsstrategie für ein bestimmtes Signal ("vector", "text", "graph") zurück. | `src/local_types.rs` |
| `pub fn mean_reciprocal_rank<'a, I>(queries: &[(&[I], &AHashSet<&str>)]) -> f32 where I: AsRef<str> + 'a,` | Berechnet den Mean Reciprocal Rank (MRR) über mehrere Abfragen hinweg. Liefert das arithmetische Mittel der Reciprocal Ranks aller Abfragen… | `src/metrics.rs` |
| `pub fn ndcg_at_k<'a, I>(ranked_ids: &[I], relevance_scores: &AHashMap<&str, f32>, k: usize) -> f32 where I: AsRef<str> + 'a,` | Berechnet den Normalized Discounted Cumulative Gain (nDCG@k). Setzt den tatsächlichen DCG@k ins Verhältnis zum idealen DCG@k (IDCG@k) bei p… | `src/metrics.rs` |
| `pub fn precision_at_k<'a, I>(ranked_ids: &[I], relevant_ids: &AHashSet<&str>, k: usize) -> f32 where I: AsRef<str> + 'a,` | Berechnet Precision@k (Genauigkeit der Top-k-Ergebnisse). Anteil der relevanten Dokumente unter den ersten `k` Treffern: `|relevant ∩ top_k… | `src/metrics.rs` |
| `pub fn recall_at_k<'a, I>(ranked_ids: &[I], relevant_ids: &AHashSet<&str>, k: usize) -> f32 where I: AsRef<str> + 'a,` | Berechnet Recall@k (Trefferquote bezogen auf alle bekannten relevanten Dokumente). Anteil der gefundenen relevanten Dokumente unter den ers… | `src/metrics.rs` |
| `pub fn reciprocal_rank<'a, I>(ranked_ids: &[I], relevant_ids: &AHashSet<&str>) -> f32 where I: AsRef<str> + 'a,` | Berechnet den Reciprocal Rank (RR) für eine geordnete Liste von Treffer-IDs. Findet die 1-basierte Rangposition des ersten relevanten Dokum… | `src/metrics.rs` |


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

# contextra-store (Ring 1)  Die Crate `contextra-store` implementiert das persistente LSM-Tree-Speichertriebwerk (Log-Structured Merge-tree) für die Contextra-Plattform. Sie stellt die primäre Datenspeicherebene bereit und bietet atomare Schreibvorgänge, Konsistenz über Snapshots sowie Ausfallsicherheit durch Write-Ahead-Logging (WAL).  ## Architektur & Datenpfad - **Client / Transaktion:** Sendet atomare Schreiboperationen (Put, Delete) über Transaktionspuffer. - **WAL (Write-Ahead-Log):** Persistiert alle Mutationen sequentiell auf Festplatte inklusive CRC32C- und HMAC-SHA256-Integritätsprüfung, bevor sie im RAM bestätigt werden. - **MemTable:** Haltebereich im Arbeitsspeicher (geschardete BTreeMap), der schnelle Suchen und Einfügungen bei geordneten Sequenznummern ermöglicht. - **Flush & SSTable:** Bei Erreichen von Größenschwellen wird die aktive MemTable eingefroren und asynchron als unveränderliche SSTable (Sorted String Table, Format V4 Standard) auf Platte geschrieben. - **Compaction:** Hintergrundprozess, der SSTables hierarchisch (Leveled / Adaptive) zusammenführt, Tombstones bereinigt und Lese- sowie Speicherverstärkung minimiert.  ## Sicherheitsinvariante Die gesamte Crate erzwingt `#![forbid(unsafe_code)]` in Produktion. Es existieren keinerlei `unsafe`-Blöcke im Produktionscode.  ## Ring-Hierarchie & Abhängigkeiten - **Ring 1 / Klasse B:** Kernkomponente des Storage-Bundles (`types`, `ports`, `mvcc`, `crypto`, `sys`, `store`). - **Eingehende Abhängigkeiten:** Wird genutzt von `contextra-engine`, `contextra-db`, `contextra-checkpoint`, `contextra-agent` und der übergeordneten Fassade `contextra`. - **Ausgehende Abhängigkeiten:** Nutzt `contextra-types`, `contextra-ports`, `contextra-crypto`, `contextra-sys`, `contextra-mvcc`.

### Modul- und Re-Export-Oberfläche (lib.rs)

```rust
pub mod compaction;
pub mod engine;
pub mod fsync;
pub mod key_source;
pub mod lsm;
pub mod manifest;
pub mod memtable;
pub mod scanner;
pub mod sstable;
pub mod system_pressure;
pub mod tenant_codec;
pub mod util;
pub mod wal;
pub mod io_seam;
pub use util::{RecordAuthenticator, RecordSealer, SealedEnvelope};
pub use compaction::{ adaptive::{ AdaptiveCompactionPlan, AdaptiveCompactionPlanner, CompactionStrategy, CostBasedAdaptivePlanner, WorkloadMetrics, WorkloadMetricsSnapshot, }, CompactionConfig, CompactionEngine, MergeOperator, TtlMetadata, };
pub use key_source::{KeyMaterialSource, OsKeyMaterialSource, StaticKeyMaterialSource};
pub use lsm::{ CommittedBatch, LsmConfig, LsmStorage, ObserverRegistry, WalEntryRef, WalObserver, WriteOrigin, DEFAULT_MAX_OBSERVER_LATENCY, };
pub use manifest::{Manifest, ManifestEntry};
pub use system_pressure::{PressureLevel, SystemPressure, SystemPressureMonitor};
pub use tenant_codec::{TenantKeyCodec, TenantScopedStorage};
pub use io_seam::{RealFs, RealFsFile, StoreIo};
```

### Feature-Flags

| Flag | Aktiviert |
|---|---|
| `default` | `[]` |
| `test-hooks` | `[]` |
| `inspect` | `[]` |
| `memory-only-storage` | `[]` |
| `fault-injection` | `[]` |
| `block-cache-v2` | `["dep:quick_cache"]` |
| `docid-128` | `["contextra-types/docid-128"]` |
| `legacy-wal-key` | `[]` |

### Schnittstellen — Public Traits (10)

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

    /// Returns the current memory usage in bytes.
    fn current_bytes(&self) -> usize;

    /// Returns the capacity limit in bytes.
    fn capacity_bytes(&self) -> usize;
}
```

**`KeyMaterialSource`** — *Trait for acquiring cryptographic key material and salts (fail-closed, strictly separated from Rng port).* (`src/key_source.rs`)

```rust
pub trait KeyMaterialSource : Send + Sync + Debug + 'static {
/// Fills the destination buffer with cryptographically secure random bytes.
    fn fill_key_bytes(&self, dest: &mut [u8]) -> Result<()>;
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

**`RecordAuthenticator`** — *Lokaler Trait zur Authentisierung von Datensätzen und Blöcken.* (`src/util.rs`)

```rust
pub trait RecordAuthenticator : Send + Sync {
/// Berechnet ein Authentisierungs-Tag über gegebene Daten.
    ///
    /// # Errors
    /// Liefert einen Fehler zurück, falls das Tag nicht erzeugt werden kann.
    fn authenticate(&self, key: &[u8], data: &[u8]) -> Result<[u8; 32]>;

    /// Verifiziert ein Authentisierungs-Tag gegen gegebene Daten.
    ///
    /// # Errors
    /// Liefert einen Fehler zurück, falls die Verifikation fehlschlägt.
    fn verify(&self, key: &[u8], data: &[u8], expected_tag: &[u8; 32]) -> Result<bool>;
}
```

**`RecordSealer`** — *Lokaler Trait zur Versiegelung von Datensätzen (Ersatz für externe Krypto-Kopplungen).* (`src/util.rs`)

```rust
pub trait RecordSealer : Send + Sync {
/// Versiegelt Klartextdaten deterministisch oder mit Umschlag.
    ///
    /// # Errors
    /// Liefert einen Fehler zurück, falls die Versiegelung fehlschlägt.
    fn seal(&self, key: &[u8], plaintext: &[u8]) -> Result<SealedEnvelope>;

    /// Öffnet einen versiegelten Datensatz.
    ///
    /// # Errors
    /// Liefert einen Fehler zurück, falls die Entsiegelung fehlschlägt.
    fn open(&self, key: &[u8], envelope: &[u8]) -> Result<Vec<u8>>;
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

**`SystemPressureSignal`** — *Injektions-Trait für externe Speicherdruck-Signale (ohne globale Zustände).* (`src/system_pressure.rs`)

```rust
pub trait SystemPressureSignal : Send + Sync {
/// Liefert den aktuellen Druckpegel.
    fn current_pressure(&self) -> PressureLevel;
    /// Informiert über eine empfohlene Speicherreduktion (z. B. Cache-Verkleinerung).
    fn advise_reduction_ratio(&self) -> f32;
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

### Öffentliche Datentypen — Structs (50)

| Struct | Felder / Form | Beschreibung | Datei |
|---|---|---|---|
| `AdaptiveCompactionPlan` | `{ pub strategy: CompactionStrategy, pub candidates: Vec<Arc<SstableReader>>, pub is_full_compaction: bool, }` | Resulting plan proposed by adaptive compaction planner. | `src/compaction/adaptive.rs` |
| `AsyncObserverAdapter` | `{ inner: Arc<dyn WalObserver>, sender: tokio::sync::mpsc::Sender<(u64, TxId, WriteOrigin)>, }` | Non-blocking, bounded async adapter wrapper for `WalObserver` implementations. | `src/lsm/observer.rs` |
| `BlockBuilder` | `{ data: BytesMut, offsets: Vec<u32>, pub(super) block_size: usize, key_hashes: Vec<(u64, u64)>, v3_bloom: [u64; 4], format_version: u16, }` | A builder for SSTable data blocks (Format v4 default, supports v3). | `src/sstable/builder.rs` |
| `BlockCache` | `{ shards: Vec<BlockCacheShard>, hash_builder: ahash::RandomState, hits: std::sync::atomic::AtomicU64, misses: std::sync::atomic::AtomicU64, }` | — | `src/sstable/block_cache.rs` |
| `BloomFilter` | `{ pub(super) bits: Vec<u64>, pub(super) num_hashes: usize, pub(super) num_bits: usize, }` | SPECCED: Speichereffizienter Bloom-Filter für SSTable-Pre-Checks. Ziel: False-Positive-Rate p ≤ 0.01. Formel: m = -n * ln(p) / (ln(2)^2) ≈ 9.6 * n Ha… | `src/sstable/bloom.rs` |
| `CommitContext` | `{ pub tx_id: TxId, pub origin: WriteOrigin, pub durable: bool, }` | Context describing a committed transaction passed to observers. | `src/lsm/observer.rs` |
| `CommitGuard` | `{ pub(super) _lock: &'a tokio::sync::MutexGuard<'a, ()>, }` | Proof that `commit_mutex` is currently held by the calling task. Can only be constructed while holding the mutex guard. | `src/lsm/guard.rs` |
| `CommittedBatch` | `{ pub entries: Vec<WalEntryRef<'a>>, pub origin: WriteOrigin, }` | Batch of committed WAL entries passed to observers. | `src/lsm/observer.rs` |
| `CompactionConfig` | `{ /// Minimum number of SSTables in a size tier to trigger compaction. pub min_sstables_per_tier: usize, /// Number of SSTables in L0 that trigger compaction. pu… }` | Configuration for the compaction engine. | `src/compaction/config.rs` |
| `CompactionEngine` | `{ config: CompactionConfig, pub(super) snapshot_registry: Arc<SnapshotRegistry>, block_cache: Arc<BlockCache>, key_manager: Option<Arc<KeyManager>>, budget: Arc<… }` | Orchestriert die periodische Verdichtung und Bereinigung von SSTables. | `src/compaction/engine.rs` |
| `CostBasedAdaptivePlanner` | `{ read_ratio_threshold: f64, min_sstables_per_tier: usize, size_ratio: f64, }` | Reference implementation of cost-based adaptive compaction (EcoTune / ArceKV inspired). Adjusts compaction aggressiveness based on read-vs-write work… | `src/compaction/adaptive.rs` |
| `FlushPurgeFence` | `{ flush_active: AtomicBool, purge_active: AtomicBool, }` | Koordiniert das gegenseitige Fencing zwischen MemTable-Flush und Purge-Kompaktion. Atomares Fencing: Garantierter wechselseitiger Ausschluss über Seq… | `src/lsm/fence.rs` |
| `InMemoryStorageEngine` | `{ data: Arc<RwLock<StoreDataMap>>, staged: Arc<RwLock<StagedTxMap>>, seq: Arc<AtomicU64>, last_tx: Arc<RwLock<TxId>>, }` | An in-memory implementation of [`StorageEngine`] guaranteed to perform no disk I/O. | `src/engine/memory_only.rs` |
| `KvSegment` | `{ /// Eindeutige Segment-ID. pub id: u64, /// Nutzdaten des Segments. pub data: Vec<u8>, }` | Neutrales Segment eines KV-Speichers. | `src/kv/mod.rs` |
| `KvSegmentStore` | `{ segments: std::collections::BTreeMap<u64, KvSegment>, }` | Speicherbereich für KV-Segmente. | `src/kv/mod.rs` |
| `LruBlockCacheBackend` | `{ state: RwLock<LruBlockCacheState>, }` | Standard strict-LRU block cache backend using `parking_lot::RwLock<LruCache>` with byte-based capacity. | `src/sstable/block_cache.rs` |
| `LsmConfig` | `{ /// Path to the data directory. pub path: PathBuf, /// Maximum size of the memtable before flushing to disk. pub memtable_size_limit: usize, /// Maximum RAM us… }` | Configuration for the LSM storage engine. | `src/lsm/config.rs` |
| `LsmPurger` | `{ storage: Arc<LsmStorage>, runtime: ScannerRuntime, }` | Purger für LSM Storage Engine. | `src/scanner/purge.rs` |
| `LsmState` | `{ pub(super) memtable: Arc<MemTable>, pub(super) immutable_memtables: Vec<Arc<MemTable>>, }` | — | `src/lsm/guard.rs` |
| `LsmStorage` | `{ pub(super) config: LsmConfig, pub(super) key_manager: Option<Arc<KeyManager>>, pub(super) state: RwLock<LsmState>, /// SSTables stored separately for shared ac… }` | Primaere Schnittstelle fuer den LSM-Baum mit get, put, delete und scan. | `src/lsm/engine.rs` |
| `Manifest` | `{ pub(super) path: PathBuf, pub(super) file: tokio::sync::Mutex<tokio::fs::File>, pub(super) _dir_lock: Option<DirLock>, pub(super) is_poisoned: AtomicBool, }` | — | `src/manifest/core.rs` |
| `MemTable` | `{ /// Sharded storage: each shard holds a disjoint subset of keys. /// Shard selection is deterministic via `shard_for()`. shards: [MemTableShard; SHARD_COUNT],… }` | — | `src/memtable.rs` |
| `MemtableScanner` | `{ storage: Arc<LsmStorage>, runtime: ScannerRuntime, }` | Scanner für die `LsmMemtable`-Schicht. | `src/scanner/memtable.rs` |
| `NoopReplayProgressSink` | `(tuple/unit)` | — | `src/wal/replay.rs` |
| `ObserverRegistry` | `{ observers: parking_lot::RwLock<Vec<Arc<BoundedObserverWorker>>>, historical_drops: Arc<AtomicU64>, max_observer_latency: parking_lot::RwLock<Duration>, clock:… }` | Configuration and handle for managing WAL observers with bounded delivery and circuit breaker. | `src/lsm/observer.rs` |
| `OsKeyMaterialSource` | `(tuple/unit)` | Default production implementation using OS CSPRNG (`rand::rngs::OsRng`). Fail-closed: returns error on entropy exhaustion or failure without fallback. | `src/key_source.rs` |
| `PreparedBatch` | `(tuple/unit)` | A batch of prepared WAL entries bound to a specific HMAC chain. `PreparedBatch` has a private inner field and cannot be constructed outside `wal.rs`.… | `src/wal/encode.rs` |
| `QuickCacheBlockCacheBackend` | `{ cache: quick_cache::sync::Cache<(u64, u64), Bytes, BlockWeighter>, }` | Lock-optimized block cache backend using `quick_cache::sync::Cache` (S3-FIFO / Clock-based eviction) with byte-based capacity. | `src/sstable/block_cache.rs` |
| `RealFs` | `(tuple/unit)` | Default POSIX/OS filesystem implementation of [`Vfs`] for contextra-store. | `src/io_seam.rs` |
| `RealFsFile` | `{ file: std::fs::File, }` | Standard [`VfsFile`] handle based on [`std::fs::File`]. | `src/io_seam.rs` |
| `ScannerRuntime` | `{ handle: Option<Handle>, }` | Laufzeit-Ausführungsstrategie für synchrone Scanner- und Purger-Ports. SPEC §3.4 (Sync-Kern / Async-Schale) & SPEC E-07 (Zero-Panic-Doktrin): Synchro… | `src/scanner/runtime.rs` |
| `SealedEnvelope` | `{ payload: Vec<u8>, }` | Kapselung eines versiegelten Datensatzes. | `src/util.rs` |
| `SieveCacheBackend` | `{ state: RwLock<SieveCacheState>, }` | Trait abstraction for block cache backend implementations. ## Approximative LRU & Lock-Optimized Read Path SSTable block caches store immutable 4KB d… | `src/sstable/block_cache.rs` |
| `SstableBuilder` | `{ path: PathBuf, tmp_path: PathBuf, file: Option<Box<dyn VfsFile>>, vfs: Arc<dyn Vfs>, block_builder: BlockBuilder, block_size: usize, index: Vec<(Bytes, u64)>,… }` | Erzeugt deterministisch geordnete SSTable-Dateien mit Datenbloecken und Index. | `src/sstable/builder.rs` |
| `SstableMetadata` | `{ pub first_key: Bytes, pub last_key: Bytes, pub file_size: u64, pub min_tx_id: u64, pub max_tx_id: u64, pub min_seq: u64, pub max_seq: u64, }` | Metadata for an SSTable. | `src/sstable/builder.rs` |
| `SstableReader` | `{ pub(super) file: Arc<dyn VfsFile>, pub(super) index: Vec<(Bytes, u64)>, pub(super) metadata: SstableMetadata, /// Byte offset where the index data begins (= en… }` | Liest SSTable-Dateien mit Block-Cache und Bloom-Filter-Unterstuetzung. | `src/sstable/reader.rs` |
| `SstableScanner` | `{ storage: Arc<LsmStorage>, runtime: ScannerRuntime, }` | Scanner für die `SsTableAllLevels`-Schicht. | `src/scanner/sstable.rs` |
| `SstableStream` | `{ pub(super) reader: Arc<SstableReader>, pub(super) block_idx: usize, pub(super) entry_idx: usize, pub(super) current_block: Option<Bytes>, pub(super) num_offset… }` | — | `src/sstable/stream.rs` |
| `StaticKeyMaterialSource` | `{ key_stream: std::sync::Arc<parking_lot::Mutex<Vec<u8>>>, }` | Fixed deterministic key material source for reproducible testing and fault simulation. | `src/key_source.rs` |
| `StoreIo` | `{ vfs: Arc<dyn Vfs>, }` | Central Store I/O abstraction (`StoreIo`) wrapping `Arc<dyn Vfs>`. | `src/io_seam.rs` |
| `SystemPressure` | `{ pub wal_queue_depth: usize, pub scheduler_queue_depth: f32, // 0.0–1.0 (Tokio global scheduler queue depth ratio) pub blocking_util: f32, // 0.0–1.0 (Blocking… }` | — | `src/system_pressure.rs` |
| `SystemPressureMonitor` | `{ pressure_tx: tokio::sync::watch::Sender<SystemPressure>, pub pressure_rx: tokio::sync::watch::Receiver<SystemPressure>, sampling_interval: Duration, pub(crate)… }` | — | `src/system_pressure.rs` |
| `TenantKeyCodec` | `{ tenant_id: TenantId, /// Vorberechnetes Scan-Präfix ohne Allokation im Hot-Path. scan_prefix_cache: Vec<u8>, }` | Encodes LSM storage keys with tenant isolation prefixes. | `src/tenant_codec.rs` |
| `TenantScopedStorage` | `{ inner: S, codec: TenantKeyCodec, }` | Tenant-aware `StorageEngine` wrapper implementing strict key-level isolation (INV-TENANT-2). | `src/tenant_codec.rs` |
| `WalEntry` | `{ /// The operation performed. pub op: WalOp, /// Sequence number assigned to the operation. pub seq_no: u64, /// HMAC of the current entry (includes previous HM… }` | A single entry in the Write-Ahead Log. | `src/wal/encode.rs` |
| `WalEntryRef` | `{ pub entry: &'a WalEntry, }` | Lightweight view reference into a `WalEntry`. | `src/lsm/observer.rs` |
| `WalFlusherConfig` | `{ /// Maximum time to wait for additional messages after receiving the first, /// before issuing `sync_all()`. Set to 0 to disable (immediate flush). /// /// Typ… }` | Configuration for the WAL background flusher actor. Controls how aggressively the flusher coalesces concurrent writes into a single `sync_all()` call… | `src/wal/flusher.rs` |
| `WalScanner` | `{ storage: Arc<LsmStorage>, runtime: ScannerRuntime, }` | Scanner für die `WalAllSegments`-Schicht. | `src/scanner/wal.rs` |
| `WorkloadMetrics` | `{ read_count: AtomicU64, write_count: AtomicU64, last_seq_no: AtomicU64, }` | Operational workload metrics captured deterministically without wall-clock time (P28 compliance). All trigger decisions are strictly derived from tra… | `src/compaction/adaptive.rs` |
| `WorkloadMetricsSnapshot` | `{ pub read_count: u64, pub write_count: u64, pub last_seq_no: u64, }` | Point-in-time snapshot of operational workload metrics. | `src/compaction/adaptive.rs` |

### Öffentliche Datentypen — Enums (11)

| Enum | Varianten | Beschreibung | Datei |
|---|---|---|---|
| `CompactionStrategy` | `/// Standard Size-Tiered Compaction Strategy optimized for write throughput. WriteOptimizedSTCS, /// Aggressive compaction merging more tiers / full LSM to reduce read amplification. ReadOptimizedAgg…` | Compaction strategy variants selected by adaptive planner. | `src/compaction/adaptive.rs` |
| `DurabilityConfigError` | `#[error("DurabilityMode::{mode:?} is incompatible with feature '{feature}': {reason}")] IncompatibleCombination { mode: DurabilityMode, feature: &'static str, reason: &'static str, },` | — | `src/lsm/config.rs` |
| `DurabilityMode` | `Full, WalNoHmac, MemoryOnly,` | — | `src/lsm/config.rs` |
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

### Konstanten (26)

| Konstante | Typ | Beschreibung |
|---|---|---|
| `BLOCKING_UTIL_CRITICAL` | `f32` | — |
| `BLOCKING_UTIL_ELEVATED` | `f32` | — |
| `BLOCK_CACHE_SHARDS` | `usize` | Sharded block cache: 64 independent shards to reduce contention. Shard selection uses ahash tuple hashing on (file_id, offset). |
| `CURRENT_MANIFEST_VERSION` | `u8` | — |
| `DEFAULT_MAX_OBSERVER_LATENCY` | `Duration` | Default maximum latency allowed for WAL observer callback execution (1 millisecond). |
| `MANIFEST_HEADER_MAGIC` | `&[u8` | — |
| `RANK_BLOCK_CACHE` | `u32` | Rang 950: SSTable-Block-Cache-Sperre (`SieveCacheBackend::state`). |
| `RANK_COMMIT_MUTEX` | `u32` | Rang 200: Globaler Commit-Mutex des Stores (`commit_mutex`), entspricht Commit-Barriere. |
| `RANK_DOCUMENT_INTENT` | `u32` | Rang 100: Transaktionale Absichtssperren auf Dokument-/Schlüsselebene (`intent_locks`). |
| `RANK_FLUSH_MUTEX` | `u32` | Rang 300: Flush-Mutex des Stores (`flush_mutex`), serialisiert Flush-Vorgänge. |
| `RANK_MANIFEST_MUTEX` | `u32` | Rang 800: Manifest-Append-Sperre (`Manifest::file`). |
| `RANK_MEMTABLE_SHARDS` | `u32` | Rang 700: MemTable-Shards (`MemTable::shards`), atomare RAM-Modifikationen. |
| `RANK_OBSERVER_REGISTRY` | `u32` | Rang 900: Observer-Registry-Sperre (`ObserverRegistry::observers`). |
| `RANK_SSTABLES_RWLOCK` | `u32` | Rang 600: SSTable-Katalog (`sstables`), schützt die Liste aktiver L0..Ln SSTables. |
| `RANK_STATE_RWLOCK` | `u32` | Rang 400: LSM-Zustand (`state`), schützt MemTable-Zeiger und unveränderliche MemTables. |
| `RANK_WAL_TRUNCATE` | `u32` | Rang 500: WAL-Truncate-Sperre (`wal.truncate_lock`), serialisiert Append/Truncate im WAL. |
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

### Öffentliche Funktionen (299)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn new() -> Self` | Creates a new WorkloadMetrics counter instance initialized to zero. | `src/compaction/adaptive.rs` |
| `pub fn new(read_ratio_threshold: f64, min_sstables_per_tier: usize, size_ratio: f64) -> Self` | Creates a new CostBasedAdaptivePlanner with configurable read ratio threshold. | `src/compaction/adaptive.rs` |
| `pub fn record_read(&self)` | Records a read operation atomically. | `src/compaction/adaptive.rs` |
| `pub fn record_write(&self, seq_no: u64)` | Records a write operation and updates the latest sequence number atomically. | `src/compaction/adaptive.rs` |
| `pub fn snapshot(&self) -> WorkloadMetricsSnapshot` | Takes a deterministic snapshot of current metrics for compaction strategy evaluation. | `src/compaction/adaptive.rs` |
| `pub fn validate_tombstone_safety( candidates: &[Arc<SstableReader>], min_active_seqno: u64, is_full_compaction: bool, ) -> bool` | Hard invariant verification enforcing INV-COMPACTION-ADAPTIVE-1. Validates that no candidate tombstone with `seq >= min_active_seqno` can e… | `src/compaction/adaptive.rs` |
| `pub async fn compact_all( &self, sstables: &RwLock<Vec<Arc<SstableReader>>>, data_path: &std::path::Path, ) -> Result<bool>` | Forces a full compaction over all existing SSTables. This maintenance API merges all currently registered SSTables into a single consolidat… | `src/compaction/engine.rs` |
| `pub async fn compact_range( &self, range: std::ops::RangeInclusive<&[u8]>, sstables: &RwLock<Vec<Arc<SstableReader>>>, data_path: &std::path::Path, ) -> Result<bool>` | Compacts all SSTables that overlap with the specified key range `[start, end]`. Neutral maintenance API without domain-specific erasure/shr… | `src/compaction/engine.rs` |
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
| `pub fn with_vfs(mut self, vfs: Arc<dyn Vfs>) -> Self` | Configures a custom VFS implementation for compaction file operations. | `src/compaction/engine.rs` |
| `pub fn retain_key_versions<T, F>(entries: Vec<T>, get_seq: F, floor_seq: u64) -> Vec<T> where F: Fn(&T) -> u64,` | Filters a sequence of entries for a SINGLE key (sorted by raw sequence DESCENDING) according to MVCC retention rules given a `floor` sequen… | `src/compaction/retention.rs` |
| `pub fn new() -> Self` | Creates a new, empty in-memory storage engine. | `src/engine/memory_only.rs` |
| `pub async fn fsync_parent_dir(path: &Path) -> Result<()>` | Alias for `sync_parent_dir` for backward compatibility across storage modules. | `src/fsync.rs` |
| `pub async fn sync_file(path: &Path) -> Result<()>` | Performs fsync on a single file at `path`. # Errors Returns `ContextraError::Storage` on Unix if opening or syncing fails. Returns `Context… | `src/fsync.rs` |
| `pub async fn sync_parent_dir(path: &Path) -> Result<()>` | Performs fsync on the parent directory of `path`. Directory fsync is required on POSIX filesystems to guarantee that newly created files or… | `src/fsync.rs` |
| `pub async fn create_dir_all(&self, path: &Path) -> Result<()>` | Creates directories along `path`. | `src/io_seam.rs` |
| `pub async fn exists(&self, path: &Path) -> Result<bool>` | Checks if a file exists. | `src/io_seam.rs` |
| `pub fn file(&self) -> &std::fs::File` | Returns a reference to the inner file. | `src/io_seam.rs` |
| `pub async fn len(&self, path: &Path) -> Result<u64>` | Returns the length in bytes of the file at `path`. | `src/io_seam.rs` |
| `pub fn new(file: std::fs::File) -> Self` | Creates a new `RealFsFile` from an open file handle. | `src/io_seam.rs` |
| `pub fn new(vfs: Arc<dyn Vfs>) -> Self` | Creates a new `StoreIo` with the specified `Vfs`. | `src/io_seam.rs` |
| `pub fn open(&self, path: &Path, opts: &VfsOpenOptions) -> Result<Box<dyn VfsFile>>` | Opens a file using positional VfsFile options. | `src/io_seam.rs` |
| `pub async fn read(&self, path: &Path) -> Result<Vec<u8>>` | Reads an entire file to a vector of bytes. | `src/io_seam.rs` |
| `pub async fn read_at(&self, path: &Path, offset: u64, len: usize) -> Result<Vec<u8>>` | Reads data positionally from `offset` into `buf`. | `src/io_seam.rs` |
| `pub async fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>>` | Reads directory entries as PathBufs. | `src/io_seam.rs` |
| `pub async fn remove_file(&self, path: &Path) -> Result<()>` | Removes a file at `path`. | `src/io_seam.rs` |
| `pub async fn rename(&self, from: &Path, to: &Path) -> Result<()>` | Atomically renames a file. | `src/io_seam.rs` |
| `pub async fn sync_dir(&self, path: &Path) -> Result<()>` | Syncs directory metadata. | `src/io_seam.rs` |
| `pub fn vfs(&self) -> &Arc<dyn Vfs>` | Returns a reference to the inner `Vfs`. | `src/io_seam.rs` |
| `pub async fn write_atomic(&self, path: &Path, data: Vec<u8>) -> Result<()>` | Atomically writes data to `path` with crash-consistency. | `src/io_seam.rs` |
| `pub fn new(bytes: Vec<u8>) -> Self` | Creates a static key material source repeating or providing fixed byte slices. | `src/key_source.rs` |
| `pub fn get(&self, id: u64) -> Option<&KvSegment>` | Liest ein Segment anhand seiner ID. | `src/kv/mod.rs` |
| `pub fn new(id: u64, data: Vec<u8>) -> Self` | Erzeugt ein neues KV-Segment. | `src/kv/mod.rs` |
| `pub fn new() -> Self` | Erzeugt einen neuen leeren Segmentspeicher. | `src/kv/mod.rs` |
| `pub fn put(&mut self, segment: KvSegment) -> Result<()>` | Speichert ein Segment. | `src/kv/mod.rs` |
| `pub fn remove(&mut self, id: u64) -> Option<KvSegment>` | Entfernt ein Segment anhand seiner ID (reine neutrale Speicheroperation). | `src/kv/mod.rs` |
| `pub fn cleanup_intent_locks_where<F>(&self, mut predicate: F) where F: FnMut(TxId) -> bool,` | Clears intent locks matching a predicate, gracefully recovering from poisoned locks. | `src/lsm/commit.rs` |
| `pub fn clear_intent_locks_above_tx(&self, target_tx: TxId)` | Removes all intent lock registrations associated with transactions strictly newer than target_tx. | `src/lsm/commit.rs` |
| `pub fn clear_intent_locks_for_tx(&self, tx_id: TxId)` | — | `src/lsm/commit.rs` |
| `pub fn validate_against_features( self, deletion_proof_active: bool, feature_ring: contextra_ports::license::FeatureRing, ) -> Result<(), DurabilityConfigError>` | — | `src/lsm/config.rs` |
| `pub fn validate_durability( self, integrity_chain_required: bool, require_persistence: bool, ) -> Result<(), DurabilityConfigError>` | Validates the durability mode against required integrity guarantees and persistence constraints. | `src/lsm/config.rs` |
| `pub fn block_cache_for_test(&self) -> Arc<BlockCache>` | — | `src/lsm/engine.rs` |
| `pub fn budget_for_test(&self) -> Arc<ResourceTracker>` | — | `src/lsm/engine.rs` |
| `pub fn budget_tracking_drift_bytes(&self) -> u64` | Returns the accumulated total memory budget tracking drift in bytes caused by unbudgeted memtable puts during commit when memory limit was… | `src/lsm/engine.rs` |
| `pub fn clock(&self) -> Arc<dyn contextra_ports::Clock>` | Returns the currently configured clock port. | `src/lsm/engine.rs` |
| `pub async fn close(&self) -> Result<()>` | Gracefully closes the storage engine, stopping background tasks and flushing active memtable to disk. # Errors Gibt `ContextraError::Storag… | `src/lsm/engine.rs` |
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
| `pub fn metrics_sink(&self) -> Arc<dyn contextra_ports::MetricsSink>` | Returns the currently configured [`contextra_ports::MetricsSink`]. | `src/lsm/engine.rs` |
| `pub async fn migrate_legacy_wal_keys( config: &LsmConfig, clock: Arc<dyn contextra_ports::Clock>, ) -> Result<usize>` | Explicitly migrates all legacy WAL files in the specified directory using the provided clock port. Returns the number of migrated WAL files… | `src/lsm/engine.rs` |
| `pub fn next_seq_no_for_test(&self) -> u64` | Returns the current `next_seq_no` value for testing verification. | `src/lsm/engine.rs` |
| `pub async fn open(config: LsmConfig) -> Result<Self>` | Opens or recovers an LSM storage instance with the given configuration. # Errors Gibt `ContextraError::Storage` zurück, falls I/O-Fehler, V… | `src/lsm/engine.rs` |
| `pub async fn open_with_merge_operator( config: LsmConfig, merge_operator: Arc<dyn MergeOperator>, ) -> Result<Self>` | Opens or recovers an LSM storage instance with the given configuration and a `MergeOperator`. # Errors Gibt `ContextraError::Storage` zurüc… | `src/lsm/engine.rs` |
| `pub fn pressure_receiver( &self, ) -> tokio::sync::watch::Receiver<crate::system_pressure::SystemPressure>` | Returns a watch receiver for monitoring system pressure levels. | `src/lsm/engine.rs` |
| `pub fn register_observer(&self, observer: Arc<dyn WalObserver>)` | Registers a new WAL observer. | `src/lsm/engine.rs` |
| `pub fn request_flush(&self)` | Requests an asynchronous background flush without blocking the caller or holding commit locks. | `src/lsm/engine.rs` |
| `pub async fn restore_wal_file_handle_for_test(&self)` | — | `src/lsm/engine.rs` |
| `pub async fn scan_prefix_tracked( &self, tx_id: TxId, prefix: &[u8], ) -> Result<Vec<(Vec<u8>, Vec<u8>)>>` | Performs a tracked prefix scan for transaction `tx_id`. Evaluates `snapshot_seq = last_applied_seq` ONCE, scans all matching entries, and r… | `src/lsm/engine.rs` |
| `pub fn set_clock(&self, clock: Arc<dyn contextra_ports::Clock>)` | Sets the injected clock port for deterministic execution timing (P28). | `src/lsm/engine.rs` |
| `pub fn set_max_observer_latency(&self, latency: std::time::Duration)` | Sets the maximum latency threshold for observer callbacks. | `src/lsm/engine.rs` |
| `pub fn set_metrics_sink(&self, sink: Arc<dyn contextra_ports::MetricsSink>)` | Sets the [`contextra_ports::MetricsSink`] for recording operational metrics. | `src/lsm/engine.rs` |
| `pub fn set_ssi_validator( &mut self, validator: Arc<dyn contextra_mvcc::SsiValidator + Send + Sync>, )` | Sets or replaces the [`contextra_mvcc::SsiValidator`] port for transaction conflict detection. | `src/lsm/engine.rs` |
| `pub fn shutdown(&self)` | Signals the background compaction engine to stop. | `src/lsm/engine.rs` |
| `pub async fn simulate_wal_append_failure_for_test(&self)` | — | `src/lsm/engine.rs` |
| `pub async fn simulate_wal_append_failure_for_tx_for_test(&self, _tx_id: u64)` | — | `src/lsm/engine.rs` |
| `pub fn spawn_tracked<F>(&self, future: F) where F: std::future::Future<Output = ()> + Send + 'static,` | Spawns a background task tracked by this storage instance. | `src/lsm/engine.rs` |
| `pub fn sstables_for_test(&self) -> Arc<RwLock<Vec<Arc<SstableReader>>>>` | — | `src/lsm/engine.rs` |
| `pub fn vfs(&self) -> &std::sync::Arc<dyn contextra_ports::Vfs>` | — | `src/lsm/engine.rs` |
| `pub async fn wait_shutdown(&self)` | Waits for all spawned tasks to shut down fully. | `src/lsm/engine.rs` |
| `pub fn with_clock(self, clock: Arc<dyn contextra_ports::Clock>) -> Self` | Configures the clock port fluently on `self` and returns `self`. | `src/lsm/engine.rs` |
| `pub fn with_metrics_sink(self, sink: Arc<dyn contextra_ports::MetricsSink>) -> Self` | Configures the [`contextra_ports::MetricsSink`] fluently on `self` and returns `self`. | `src/lsm/engine.rs` |
| `pub fn with_ssi_validator( mut self, validator: Arc<dyn contextra_mvcc::SsiValidator + Send + Sync>, ) -> Self` | Configures the [`contextra_mvcc::SsiValidator`] fluently on `self` and returns `self`. | `src/lsm/engine.rs` |
| `pub fn enter_flush(&self) -> bool` | Markiert den Beginn eines Flush-Vorgangs. Gibt `true` zurück, wenn kein Purge aktiv ist und Flush erworben wurde. | `src/lsm/fence.rs` |
| `pub fn enter_purge(&self) -> bool` | Markiert den Beginn einer Purge-Kompaktion (Priorität). Gibt `true` zurück, wenn Purge exklusiven Zugriff erlangt hat. | `src/lsm/fence.rs` |
| `pub fn is_flush_active(&self) -> bool` | Prüft nicht-blockierend, ob ein Flush aktiv ist. | `src/lsm/fence.rs` |
| `pub fn is_purge_active(&self) -> bool` | Prüft nicht-blockierend, ob ein Purge ansteht oder aktiv ist. | `src/lsm/fence.rs` |
| `pub fn leave_flush(&self)` | Beendet den Flush-Abschnitt. | `src/lsm/fence.rs` |
| `pub fn leave_purge(&self)` | Beendet den Purge-Abschnitt. | `src/lsm/fence.rs` |
| `pub fn load_snapshot_tx(target: &AtomicU64) -> u64` | Lädt den aktuellen Sichtbarkeitshorizont für einen konsistenten Snapshot mit `Acquire`-Semantik. | `src/lsm/fence.rs` |
| `pub fn publish_visibility(target: &AtomicU64, new_tx: u64)` | Publiziert Sichtbarkeitsfortschritte (`last_committed_tx`) mit `Release`-Semantik. | `src/lsm/fence.rs` |
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
| `pub fn is_poisoned(&self) -> bool` | Checks if the manifest handle has been poisoned by a preceding fsync failure (INV-FSYNC-1). | `src/manifest/core.rs` |
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
| `pub fn new(storage: Arc<LsmStorage>) -> Self` | Erzeugt einen neuen `MemtableScanner` mit implizit erfasster Laufzeit. | `src/scanner/memtable.rs` |
| `pub fn with_runtime(storage: Arc<LsmStorage>, runtime: ScannerRuntime) -> Self` | Erzeugt einen neuen `MemtableScanner` mit expliziter `ScannerRuntime`. | `src/scanner/memtable.rs` |
| `pub fn new(storage: Arc<LsmStorage>) -> Self` | Erzeugt einen neuen `LsmPurger` mit implizit erfasster Laufzeit. | `src/scanner/purge.rs` |
| `pub fn with_runtime(storage: Arc<LsmStorage>, runtime: ScannerRuntime) -> Self` | Erzeugt einen neuen `LsmPurger` mit expliziter `ScannerRuntime`. | `src/scanner/purge.rs` |
| `pub fn block_on<F, T>(&self, future: F) -> Result<T> where F: Future<Output = Result<T>> + Send, T: Send,` | Führt ein Future synchron aus. Strategie: 1. Kein Handle vorhanden: Sofortiger Fehler `ContextraError::Storage`. 2. Multi-Thread Runtime: A… | `src/scanner/runtime.rs` |
| `pub fn capture() -> Self` | Erfasst das aktuelle Tokio-Runtime-Handle aus dem Aufrufkontext. | `src/scanner/runtime.rs` |
| `pub fn empty() -> Self` | Erstellt eine leere `ScannerRuntime` (führt bei Port-Aufruf zu einem typisierten Fehler). | `src/scanner/runtime.rs` |
| `pub fn handle(&self) -> Option<&Handle>` | Liefert eine Referenz auf das Handle, falls vorhanden. | `src/scanner/runtime.rs` |
| `pub fn with_handle(handle: Handle) -> Self` | Erstellt eine `ScannerRuntime` mit explizit bereitgestelltem Handle. | `src/scanner/runtime.rs` |
| `pub fn new(storage: Arc<LsmStorage>) -> Self` | Erzeugt einen neuen `SstableScanner` mit implizit erfasster Laufzeit. | `src/scanner/sstable.rs` |
| `pub fn with_runtime(storage: Arc<LsmStorage>, runtime: ScannerRuntime) -> Self` | Erzeugt einen neuen `SstableScanner` mit expliziter `ScannerRuntime`. | `src/scanner/sstable.rs` |
| `pub fn new(storage: Arc<LsmStorage>) -> Self` | Erzeugt einen neuen `WalScanner` mit implizit erfasster Laufzeit. | `src/scanner/wal.rs` |
| `pub fn with_runtime(storage: Arc<LsmStorage>, runtime: ScannerRuntime) -> Self` | Erzeugt einen neuen `WalScanner` mit expliziter `ScannerRuntime`. | `src/scanner/wal.rs` |
| `pub fn contains(&self, file_id: u64, offset: u64) -> bool` | — | `src/sstable/block_cache.rs` |
| `pub fn create_block_cache(capacity_mb: usize) -> Arc<BlockCache>` | Creates a new block cache instance with default shard count. Capacity is in MB (assuming 4KB blocks). | `src/sstable/block_cache.rs` |
| `pub fn create_block_cache_with_shards(capacity_mb: usize, num_shards: usize) -> Arc<BlockCache>` | Creates a new block cache instance with configurable shard count. Capacity is in MB. | `src/sstable/block_cache.rs` |
| `pub fn current_bytes(&self) -> usize` | Returns current memory usage in bytes across all shards. | `src/sstable/block_cache.rs` |
| `pub fn get(&self, file_id: u64, offset: u64) -> Option<Bytes>` | — | `src/sstable/block_cache.rs` |
| `pub fn hit_ratio(&self) -> f64` | Returns the cache hit ratio (0.0 .. 1.0). | `src/sstable/block_cache.rs` |
| `pub fn hits(&self) -> u64` | Returns the total number of cache hits recorded. | `src/sstable/block_cache.rs` |
| `pub fn insert(&self, file_id: u64, offset: u64, data: Bytes)` | — | `src/sstable/block_cache.rs` |
| `pub fn is_empty(&self) -> bool` | — | `src/sstable/block_cache.rs` |
| `pub fn len(&self) -> usize` | — | `src/sstable/block_cache.rs` |
| `pub fn misses(&self) -> u64` | Returns the total number of cache misses recorded. | `src/sstable/block_cache.rs` |
| `pub fn new(capacity_per_shard: usize) -> Self` | — | `src/sstable/block_cache.rs` |
| `pub fn new_with_shards(capacity_per_shard: usize, num_shards: usize) -> Self` | — | `src/sstable/block_cache.rs` |
| `pub fn reset_metrics(&self)` | Resets hit and miss metrics counters. | `src/sstable/block_cache.rs` |
| `pub fn shard_idx(&self, file_id: u64, offset: u64) -> usize` | — | `src/sstable/block_cache.rs` |
| `pub fn total_capacity_bytes(&self) -> usize` | Returns total configured byte capacity across all shards. | `src/sstable/block_cache.rs` |
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
| `pub async fn create_with_vfs( path: impl AsRef<Path>, key_manager: Option<Arc<KeyManager>>, vfs: Arc<dyn Vfs>, ) -> Result<Self>` | — | `src/sstable/builder.rs` |
| `pub fn current_size(&self) -> usize` | — | `src/sstable/builder.rs` |
| `pub async fn finish(mut self) -> Result<SstableMetadata>` | Finalizes the SSTable and returns metadata. | `src/sstable/builder.rs` |
| `pub fn is_empty(&self) -> bool` | — | `src/sstable/builder.rs` |
| `pub fn new(block_size: usize) -> Self` | — | `src/sstable/builder.rs` |
| `pub fn new_with_version(block_size: usize, format_version: u16) -> Self` | — | `src/sstable/builder.rs` |
| `pub fn set_block_size(&mut self, size: usize)` | Configures the target block size for data blocks. | `src/sstable/builder.rs` |
| `pub fn set_bloom_filter(&mut self, expected_keys: usize, fpr: f64)` | Configures the whole-SSTable bloom filter parameters. | `src/sstable/builder.rs` |
| `pub fn set_format_version(&mut self, version: u16)` | Explicitly configures the SSTable format version to write (default: 4). | `src/sstable/builder.rs` |
| `pub fn with_block_size(mut self, size: usize) -> Self` | Sets the target block size for data blocks. | `src/sstable/builder.rs` |
| `pub fn with_bloom_fpr(mut self, expected_keys: usize, fpr: f64) -> Self` | Sets the whole-SSTable bloom filter parameters. | `src/sstable/builder.rs` |
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
| `pub async fn open_with_vfs( path: impl AsRef<Path>, block_cache: Arc<BlockCache>, key_manager: Option<Arc<KeyManager>>, vfs: Arc<dyn Vfs>, ) -> Result<Self>` | — | `src/sstable/reader.rs` |
| `pub fn parse_block_trailer( block_data: &[u8], format_version: u16, ) -> Result<(usize, usize, &[u8])>` | Helper function to parse block trailer across format versions v0..v4. Returns `(num_offsets, offsets_start, bloom_bytes)`. | `src/sstable/reader.rs` |
| `pub async fn stream(self: &Arc<Self>) -> Result<SstableStream>` | — | `src/sstable/reader.rs` |
| `pub async fn scan_prefix(&self, prefix: &[u8]) -> Result<Vec<(Bytes, Bytes, u64, u64)>>` | Scans the SSTable for keys starting with the given prefix. | `src/sstable/reader_ext.rs` |
| `pub async fn scan_range( &self, start: std::ops::Bound<&[u8]>, end: std::ops::Bound<&[u8]>, ) -> Result<Vec<(Bytes, Bytes, u64, u64)>>` | Scans all entries within a key range. | `src/sstable/reader_ext.rs` |
| `pub async fn next(&mut self) -> Result<Option<(Bytes, Bytes, u64, u64)>>` | — | `src/sstable/stream.rs` |
| `pub async fn next_entry(&mut self) -> Result<Option<(Bytes, Bytes, u64, u64)>>` | Optimized for compaction: avoids copying if possible. | `src/sstable/stream.rs` |
| `pub fn compute_pressure( &self, wal_depth: usize, scheduler_queue_depth: f32, blocking_util: f32, embedding_queue: usize, max_permits: usize, ) -> SystemPressure` | — | `src/system_pressure.rs` |
| `pub fn new(sampling_interval: Duration) -> Self` | — | `src/system_pressure.rs` |
| `pub fn new_with_clock( sampling_interval: Duration, clock: Arc<dyn contextra_ports::Clock>, ) -> Self` | — | `src/system_pressure.rs` |
| `pub async fn run( self, cancellation: CancellationToken, wal_queue_depth_fn: impl Fn() -> usize + Send + 'static, embedding_permits_fn: impl Fn() -> usize + Send + 'static, max_embedding_permits: usize, )` | — | `src/system_pressure.rs` |
| `pub async fn run_with_metrics_sink( self, cancellation: CancellationToken, wal_queue_depth_fn: impl Fn() -> usize + Send + 'static, embedding_permits_fn: impl Fn() -> usize + Send + 'static, max_embedding_permits: usize, metrics_sink: Option<Arc<parking_lot::…` | — | `src/system_pressure.rs` |
| `pub fn with_clock(mut self, clock: Arc<dyn contextra_ports::Clock>) -> Self` | — | `src/system_pressure.rs` |
| `pub fn collection_prefix(&self, collection: &CollectionId) -> Vec<u8>` | Collection-spezifisches Scan-Präfix: `t:{tenant}:{col}:` | `src/tenant_codec.rs` |
| `pub fn decode_chunk_key(key: &[u8]) -> Option<(TenantId, CollectionId, DocId)>` | Dekodiert einen Chunk-Key vollständig in (TenantId, CollectionId, DocId). Invariante: decode(encode(x)) == x (Injektivität). | `src/tenant_codec.rs` |
| `pub fn decode_tenant_id(key: &[u8]) -> Option<TenantId>` | Dekodiert TenantId aus encoded Key. None bei ungültigem Format. Kein Unwrap — robuste Nutzung im Recovery-Pfad. | `src/tenant_codec.rs` |
| `pub fn encode_chunk_key(&self, collection: &CollectionId, doc_id: DocId) -> Vec<u8>` | Encodes a document chunk key: `t:{tenant}:{collection}:chunk:{doc}`. | `src/tenant_codec.rs` |
| `pub fn encode_graph_key(&self, collection: &CollectionId, entity_id: u64) -> Vec<u8>` | Encodes a graph entity key: `t:{tenant}:{collection}:graph:{entity}`. | `src/tenant_codec.rs` |
| `pub fn new(tenant_id: TenantId) -> Self` | Creates a new `TenantKeyCodec` for the given `TenantId`. | `src/tenant_codec.rs` |
| `pub fn new(inner: S, tenant_id: TenantId) -> Self` | Creates a new `TenantScopedStorage` wrapping an underlying `StorageEngine` or shared pointer. | `src/tenant_codec.rs` |
| `pub fn scan_prefix(&self) -> &[u8]` | Globales Scan-Präfix für diesen Tenant (alle Collections): `t:{tenant}:` Cached — keine Allokation im Hot-Path. | `src/tenant_codec.rs` |
| `pub fn tenant_id(&self) -> TenantId` | Returns the immutable `TenantId` bound to this storage wrapper. | `src/tenant_codec.rs` |
| `pub fn new(payload: Vec<u8>) -> Self` | Erzeugt einen neuen [`SealedEnvelope`]. | `src/util.rs` |
| `pub fn payload(&self) -> &[u8]` | Liefert die Rohdaten des Umschlags. | `src/util.rs` |
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
| `pub fn last_hmac_lock_for_test(&self) -> std::sync::Arc<tokio::sync::Mutex<[u8` | Exposes internal `last_hmac` lock for concurrency testing. | `src/wal/hmac.rs` |
| `pub fn legacy_integrity_key_for_test() -> Result<[u8` | — | `src/wal/hmac.rs` |
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
| `pub async fn rewrite_as_v3_with_integrity_key( &self, replayed_entries: &[(u64, WalEntry, u64)], integrity_key: [u8; 32], ) -> Result<()>` | Rewrites legacy V1 or V2 WAL files as V3 using an explicit integrity key. | `src/wal/io.rs` |
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

## `contextra-checkpoint`

| Attribut | Wert |
|---|---|
| **Ring** | 1 — Ring 1 — Persistence & Cache |
| **Status** | Fertig |
| **Zweck** | Time-Travel-Registry und Checkpoint-Verwaltung ohne globalen Zustand. |
| **Unsafe-Code** | Nein (`#![forbid(unsafe_code)]`) |
| **Interne Abhängigkeiten** | `contextra-ports`, `contextra-types` |
| **Externe Abhängigkeiten (Auswahl)** | `blake3`, `bytes`, `parking_lot`, `serde`, `serde_json`, `tokio`, `tracing` |

### Modul- und Re-Export-Oberfläche (lib.rs)

```rust
pub use guard::{CheckpointGuard, PinGuard};
pub use hardlink_cloner::{ CheckpointHardlinkCloner, DefaultHardlinkCloner, HardlinkCloneResult, SnapshotPinner, };
pub use manifest::CheckpointManifest;
pub use meta::{CheckpointMeta, StateCheckpoint};
pub use orphan::{ await_pending_rollbacks, clear_all_orphaned_checkpoints, clear_orphaned_checkpoint, get_orphaned_checkpoints, get_orphaned_checkpoints_for_namespace, global_orphan_registry, monotonic_timestamp_ms, monotonic_timestamp_ms_with, orphaned_checkpoint_count, pending_rollback_count, register_orphaned_checkpoint, register_pinned_seq_no_orphan, InstanceOrphanRegistry, OrphanRegistry, OrphanState, PinId, PinnedSeqNoOrphan, RecoveryReport, };
pub use store::{CheckpointRegistry, PersistentCheckpointStore};
```

### Schnittstellen — Public Traits (3)

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
        snapshot_pinner: &'a (dyn SnapshotPinner + 'a),
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

**`SnapshotPinner`** — *Trait zur Abstraktion von Snapshot-Pinning-Mechanismen während des SSTable-Klonens.* (`src/hardlink_cloner.rs`)

```rust
pub trait SnapshotPinner : Send + Sync {
/// Pinnt eine Sequenznummer und liefert einen RAII-Guard zurück.
    fn pin_seq_no(&self, seq_no: u64) -> Box<dyn std::any::Any + Send + Sync>;
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

### Öffentliche Funktionen (108)

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
| `pub fn load_vfs(vfs: &dyn contextra_ports::Vfs, path: &std::path::Path) -> Self` | Lädt den Zustand über die Vfs-Schnittstelle. | `src/orphan.rs` |
| `pub fn monotonic_timestamp_ms() -> u64` | Liest den Millisekunden-Zeitstempel über die standardmäßige SystemClock. | `src/orphan.rs` |
| `pub fn monotonic_timestamp_ms_with(clock: &dyn contextra_ports::Clock) -> u64` | Liest einen Millisekunden-Zeitstempel deterministisch über die übergebene Clock-Port-Instanz. # Parameter - `clock`: Referenz auf eine Impl… | `src/orphan.rs` |
| `pub fn new(file_path: impl Into<std::path::PathBuf>) -> Self` | Erzeugt eine neue Instanz von [`OrphanRegistry`]. | `src/orphan.rs` |
| `pub fn new(persist_path: impl Into<std::path::PathBuf>) -> Self` | Erzeugt eine neue instanzgebundene Registry für den angegebenen Dateipfad. | `src/orphan.rs` |
| `pub fn new_with_vfs( persist_path: impl Into<std::path::PathBuf>, vfs: Arc<dyn contextra_ports::Vfs>, ) -> Self` | Erzeugt eine neue instanzgebundene Registry über den VFS-Port. | `src/orphan.rs` |
| `pub fn orphaned_checkpoint_count() -> usize` | Liefert die Anzahl der aktuell registrierten verwaisten ("orphaned") Checkpoints. | `src/orphan.rs` |
| `pub fn pending_rollback_count() -> usize` | Rückwärtskompatibilität: Liefert die Anzahl ausstehender Rollback-Hintergrundaufgaben (immer 0). | `src/orphan.rs` |
| `pub fn persist_sync(&self) -> std::io::Result<()>` | Schreibt den Zustand synchron als formatierte JSON-Datei auf die Festplatte (ggf. über Vfs). | `src/orphan.rs` |
| `pub fn persist_sync(&self) -> std::io::Result<()>` | Persistiert den aktuellen Zustand synchron auf die Festplatte (bevorzugt über Vfs). | `src/orphan.rs` |
| `pub fn persist_vfs(&self, vfs: &dyn contextra_ports::Vfs) -> std::io::Result<()>` | Schreibt den Zustand über die Vfs-Schnittstelle crash-konsistent (`write_atomic`). | `src/orphan.rs` |
| `pub async fn recover_and_clean<S: contextra_ports::StorageEngine>( &self, storage: &S, ) -> Result<Vec<PinId>>` | Bereinigt alle verwaisten Pins durch Entpinnen in der Storage-Engine. # Ablauf von `recover_and_clean`: 1. Liste aller verwaisten Pins abfr… | `src/orphan.rs` |
| `pub fn register_checkpoint_sync(&self, cp: StateCheckpoint)` | Registriert einen verwaisten Checkpoint rein im Arbeitsspeicher ohne blockierendes Disk-I/O. # Crash-Safety & RAII Drop Semantik Wird synch… | `src/orphan.rs` |
| `pub fn register_orphan(&self, pin_id: PinId) -> std::io::Result<()>` | Registriert synchron einen verwaisten Pin. | `src/orphan.rs` |
| `pub fn register_orphan_sync(&self, orphan: PinnedSeqNoOrphan)` | Registriert einen verwaisten Sequenz-Pin rein im Arbeitsspeicher ohne blockierendes Disk-I/O. # Crash-Safety & RAII Drop Semantik Wird sync… | `src/orphan.rs` |
| `pub fn register_orphaned_checkpoint(cp: StateCheckpoint)` | — | `src/orphan.rs` |
| `pub fn register_pinned_seq_no_orphan(orphan: PinnedSeqNoOrphan)` | — | `src/orphan.rs` |
| `pub fn set_clock(&mut self, clock: Arc<dyn contextra_ports::Clock>)` | Setzt die Clock-Implementierung nachträglich. | `src/orphan.rs` |
| `pub fn total_recovered(&self) -> usize` | Liefert die Gesamtzahl aller bereinigten Pins und Checkpoints. | `src/orphan.rs` |
| `pub fn vfs(&self) -> Option<&Arc<dyn contextra_ports::Vfs>>` | Liefert eine Referenz auf das injizierte VFS (falls vorhanden). | `src/orphan.rs` |
| `pub fn with_clock(mut self, clock: Arc<dyn contextra_ports::Clock>) -> Self` | Konfiguriert die Registry mit einer spezifischen Clock-Implementierung. | `src/orphan.rs` |
| `pub fn with_vfs(mut self, vfs: Arc<dyn contextra_ports::Vfs>) -> Self` | Konfiguriert die Registry mit einer VFS-Implementierung. | `src/orphan.rs` |
| `pub async fn allocate_tx(&self) -> Result<TxId>` | Reserviert eine neue Transaktions-ID aus dem reservierten internen Bereich (`INTERNAL_BASE`). # Ablauf von `allocate_tx`: 1. Atomaren Zähle… | `src/store.rs` |
| `pub fn checkpoint_counter(&self) -> u64` | Liefert den Zählerstand für Checkpoint-Zeitstempel. | `src/store.rs` |
| `pub fn checkpoint_guard_skipped_rollback_count(&self) -> u64` | Alias für `skipped_rollback_count`. | `src/store.rs` |
| `pub fn clear_all_orphaned_checkpoints(&self)` | Löscht alle verwaisten Checkpoints. | `src/store.rs` |
| `pub fn clear_orphaned_checkpoint(&self, tx_id: TxId)` | Löscht einen verwaisten Checkpoint anhand seiner Transaktions-ID. | `src/store.rs` |
| `pub fn clock(&self) -> &Arc<dyn contextra_ports::Clock>` | Liefert den aktuellen Clock-Port. | `src/store.rs` |
| `pub async fn close(&self) -> Result<()>` | Alias für `shutdown()`. Führt einen kontrollierten Flush beim Schließen aus. | `src/store.rs` |
| `pub async fn create_checkpoint( &self, name: &str, collection_id: &str, seq_no: u64, tx_id: TxId, metadata: serde_json::Value, ) -> Result<CheckpointMeta>` | Creates a new persistent checkpoint. # Ablauf von `create_checkpoint`: 1. Bezeichner `name` und `collection_id` validieren. 2. `write_lock`… | `src/store.rs` |
| `pub fn create_guard(&self, tx_id: TxId) -> Result<CheckpointGuard<S>>` | Creates an ephemeral transactional checkpoint RAII guard. If the returned guard is dropped without calling `.commit()`, the underlying stor… | `src/store.rs` |
| `pub async fn create_hardlink_clone( &self, seq_no: u64, source_dir: &Path, target_dir: &Path, snapshot_pinner: &(dyn SnapshotPinner + '_), ) -> Result<HardlinkCloneResult>` | Creates a physical hardlink clone of all SSTable files visible for `seq_no` into `target_dir`. Delegates to [`DefaultHardlinkCloner`] while… | `src/store.rs` |
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

## `contextra-engine`

| Attribut | Wert |
|---|---|
| **Ring** | 3 — Ring 3 — Orchestration & Cognition |
| **Status** | In Migration |
| **Zweck** | Collection-LSM-Anbindung, Multi-Index-Pläne und Schreibtransaktionen. |
| **Unsafe-Code** | Nein (`#![forbid(unsafe_code)]`) |
| **Interne Abhängigkeiten** | `contextra-checkpoint`, `contextra-graph`, `contextra-mvcc`, `contextra-ports`, `contextra-rank`, `contextra-store`, `contextra-text`, `contextra-types`, `contextra-vector` |
| **Externe Abhängigkeiten (Auswahl)** | `ahash`, `bincode`, `blake3`, `bytes`, `hmac`, `parking_lot`, `serde`, `serde_json`, `sha2`, `thiserror`, `tokio`, `tokio-util`, `tracing`, `zeroize` |

### Architekturbeschreibung

# Modul: lib.rs  **Zweck:** Zentrale Facade und Orchestrierung der Contextra-Engine (Ring 2).  **Architektur:** Gehört zur Crate `contextra-engine` (Ring 2). Eingehaltene Invarianten: Fail-Closed bei Fehlern, kein ungeschützter globaler Zustand.

### Modul- und Re-Export-Oberfläche (lib.rs)

```rust
pub mod orchestration;
pub mod adapters;
pub mod backends;
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
pub mod contextra_impl;
pub use contextra_ports::TextEmbeddingEngine;
pub use contextra_ports::VectorDeleteMode;
pub use contextra_store::lsm::DurabilityMode;
pub use decay_controller::{AdaptiveDecayController, DecayControllerConfig, DecaySignalInputs};
pub use export::{ ExportCollectionV1, ExportDocumentV1, ExportMemoryV1, ExportRelationV1, SCHEMA_VERSION_V1, };
pub use import::ImportSummary;
pub use background_workers::start_decay_cleanup_worker;
pub use background_workers::{start_expiry_cleanup_worker, start_orphan_cleanup_worker};
pub use backends::Backends;
pub use backends::DefaultBackends;
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
pub use contextra_types::DistanceMetric;
pub use serde_json::json;
```

### Feature-Flags

| Flag | Aktiviert |
|---|---|
| `default` | `[]` |
| `reranking` | `[]` |
| `experimental-diskann` | `["contextra-vector/experimental-diskann"]` |
| `background-maintenance` | `[]` |
| `graph-connectivity-health` | `["contextra-graph/graph-connectivity-health"]` |
| `edge-reinforcement-learning` | `[]` |
| `entity-extraction` | `[]` |
| `auto-extraction-opt-out` | `[]` |

### Schnittstellen — Public Traits (5)

**`Backends`** — *Baukasten-Vertrag (M3, WP-ENG-1): Assoziiertes Trait-Bündel zur Entkopplung der Engine von konkreten Backend-Implementierungen (`store`, `vector`, `text`).* (`src/backends.rs`)

```rust
pub trait Backends : Send + Sync + 'static {
/// Konkreter Speicher-Backend-Typ (implementiert `StorageEngine`).
    type Storage: StorageEngine;
    /// Konkreter Vektor-Index-Typ (implementiert `VectorIndex`).
    type Vector: VectorIndex;
}
```

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

**`WorkflowStep`** — *Trait für zustandslose oder isolierte Workflow-Schritte (Dyn-kompatibel via Pin<Box<dyn Future>>).* (`src/orchestration/mod.rs`)

```rust
pub trait WorkflowStep : Send + Sync {
/// Name oder Bezeichner des Schritts.
    fn name(&self) -> &str;

    /// Führt den Schritt asynchron auf einem Kontext-Objekt aus.
    fn run<'a>(
        &'a self,
        context: &'a mut WorkflowContext,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>>;
}
```

### Öffentliche Datentypen — Structs (48)

| Struct | Felder / Form | Beschreibung | Datei |
|---|---|---|---|
| `AdaptiveDecayController` | `{ config: DecayControllerConfig, clock: Option<Arc<dyn Clock>>, }` | Adaptive Decay Controller für time-weighted Cache-Eviction mit injizierter Clock. | `src/decay_controller.rs` |
| `AutoExtractionConfig` | `{ /// Laufzeit-Modus fuer die automatische Extraktion (Default: Enabled, bzw. Disabled wenn Feature `auto-extraction-opt-out` aktiv ist). #[serde(default)] pub m… }` | Konfiguration fuer die automatische Entitaetsextraktion beim Einfuegen von Dokumenten. | `src/collection/crud/auto_extraction.rs` |
| `Bm25Scorer` | `{ index: InvertedIndex<S>, }` | Adapter implementing [`TextIndex`] on top of [`InvertedIndex`]. | `src/adapters/text.rs` |
| `CheckpointPinGuard` | `{ storage: &'a S, seq: u64, unpinned: bool, }` | RAII Guard for pinning snapshot checkpoints during search/scan operations. # Note on async Drop Rust does not support native `async Drop`. Therefore,… | `src/collection/search/checkpoint.rs` |
| `ChunkerConfig` | `{ /// Maximum tokens per chunk (soft limit, hard limit is this * 1.2). /// Default: 512 tokens (aligned with standard dense embedding models). pub max_tokens: us… }` | Configuration for the Markdown chunker. | `src/chunker/config.rs` |
| `Collection` | `{ pub(super) name: String, pub(super) prefix: Vec<u8>, pub(super) index: Arc<V>, pub(super) text_index: InvertedIndex<S>, pub(super) graph_index: Arc<CsrGraph>,… }` | # Concurrency & Lock Hierarchy Lock acquisition within `Collection` follows strict ordering to prevent deadlocks: 1. `Collection::kv_locks` (`KvKeyLo… | `src/collection/mod.rs` |
| `CollectionConfig` | `{ /// Configurable backpressure delay in milliseconds when system pressure is Critical. /// Default: Some(50) pub backpressure_delay_ms: Option<u64>, }` | Configuration for a collection. | `src/collection/mod.rs` |
| `CommitLedger` | `{ actions: Vec<Box<dyn CompensatingAction>>, }` | A ledger of compensating actions collected during 2PC, executed in reverse order (LIFO) on failure. | `src/transaction/compensating_actions.rs` |
| `CommunityDetectionConfig` | `{ pub auto_trigger_threshold: u64, }` | Konfiguration für Community-Erkennungs- und Graph-Clustering-Algorithmen. | `src/lib.rs` |
| `CompensateGraphAction` | `{ pub(super) collection: Collection<S, V>, pub(super) doc_ids: Arc<Vec<DocId>>, }` | Dokumentation für `CompensateGraphAction` im Rahmen der Transaktions- und Facade-Orchestrierung. | `src/transaction/compensating_actions.rs` |
| `CompensateHnswAction` | `{ pub(super) collection: Collection<S, V>, pub(super) tenant_id: TenantId, pub(super) doc_ids: Arc<Vec<DocId>>, }` | Dokumentation für `CompensateHnswAction` im Rahmen der Transaktions- und Facade-Orchestrierung. | `src/transaction/compensating_actions.rs` |
| `CompensateLsmAction` | `{ pub(super) collection: Collection<S, V>, pub(super) intent_key: Vec<u8>, pub(super) doc_ids: Arc<Vec<DocId>>, pub(super) f_keys: Vec<StagedKeyOp>, pub(super) r… }` | Dokumentation für `CompensateLsmAction` im Rahmen der Transaktions- und Facade-Orchestrierung. | `src/transaction/compensating_actions.rs` |
| `Completed` | `(tuple/unit)` | — | `src/orchestration/mod.rs` |
| `Contextra` | `{ storage: Arc<LsmStorage>, next_tx: Arc<AtomicU64>, dimension: usize, config: ContextraConfig, license_gate: Arc<dyn LicenseGate>, expiry_reaper_interval: std::… }` | Zentraler Orchestrator und Einstiegspunkt der Contextra-Engine. | `src/lib.rs` |
| `ContextraConfig` | `{ pub dimension: usize, pub max_elements: usize, pub distance_metric: contextra_types::DistanceMetric, pub encryption_passphrase: Option<String>, /// Maximum RAM… }` | Globale Konfiguration für die Contextra-Engine-Initialisierung. | `src/lib.rs` |
| `ContextraStats` | `{ pub drift_status: String, #[serde(default, skip_serializing_if = "Option::is_none")] pub calibration_ece: Option<f32>, #[serde(default, skip_serializing_if = "… }` | Aggregierte Betriebs- und Belegungsstatistiken der Engine. | `src/lib.rs` |
| `DbTransaction` | `{ pub tx_id: TxId, pub(super) collection: Collection<S, V>, pub(super) staged_forward_keys: Mutex<Vec<StagedKeyOp>>, pub(super) staged_reverse_keys: Mutex<Vec<St… }` | A transaction wrapper that ensures atomic multi-index commits across LSM-Store, HNSW-Index, Text-Index, and Graph-Index. | `src/transaction/db_transaction.rs` |
| `DecayControllerConfig` | `{ /// Sensitivitätskoeffizient für Temperatur. pub kappa: f32, /// Basis-Halbwertszeit in Transaktionszyklen. pub base_half_life_tx: u64, /// Basis-Halbwertszeit… }` | Konfiguration für den Adaptive Decay Controller. | `src/decay_controller.rs` |
| `DecaySignalInputs` | `{ /// Anteil von Tombstones im Index (0.0..1.0). pub tombstone_ratio: f32, /// Inverse Abfragelast (1.0 = idle, 0.0 = überlastet). pub query_load_inverse: f32, /… }` | Inputs für den Adaptive Decay Controller. | `src/decay_controller.rs` |
| `DefaultBackends` | `(tuple/unit)` | Standard-Backends unter Verwendung von LSM-Storage und HNSW-Index. | `src/backends.rs` |
| `DeferredHyperedgeQueue` | `{ inner: Mutex<VecDeque<HyperEdgeId>>, }` | Thread-safe FIFO queue for storing hyperedges whose cascade invalidation was deferred to the background worker due to fan-out limits (H5 / AK-6). | `src/background_workers/hyperedge_worker.rs` |
| `Document` | `{ pub id: String, pub metadata: Option<Value>, }` | Repräsentiert ein Dokument mit eindeutiger ID, Text, Vektoreinbettung und Metadaten. | `src/lib.rs` |
| `Draft` | `(tuple/unit)` | — | `src/orchestration/mod.rs` |
| `EngineServiceContainer` | `{ storage: Arc<S>, vector_index: Arc<V>, text_index: Arc<T>, }` | Service-Container für Dependency Injection in State-Machines und Orchestratoren. | `src/orchestration/mod.rs` |
| `EntityExtractionConfig` | `{ /// Ob Entitätsextraktion aktiviert ist pub enabled: bool, /// Maximale Anzahl von LLM-Aufrufen pro Zyklus / Extraktionsdurchlauf pub max_llm_calls_per_cycle:… }` | Konfiguration für die automatische Entitätsextraktion. | `src/extraction/types.rs` |
| `EntityExtractionConfig` | `{ pub enabled: bool, pub max_llm_calls_per_cycle: usize, pub min_confidence: f32, }` | Dokumentation für `EntityExtractionConfig` im Rahmen der Collection- und Suchabstraktion. | `src/collection/crud/auto_extraction.rs` |
| `Executing` | `(tuple/unit)` | — | `src/orchestration/mod.rs` |
| `ExportCollectionV1` | `{ /// Name der Collection pub name: String, /// Liste aller Memory-Einträge pub memories: Vec<ExportMemoryV1>, /// Liste aller direkten Graph-Relationen pub rela… }` | Export-Repräsentation einer einzelnen Collection. | `src/export.rs` |
| `ExportDocumentV1` | `{ /// Pflichtfeld für Schema-Migrationen (MUSS "1.0" sein) pub schema_version: String, /// UTC-Zeitstempel der Export-Erstellung (ISO-8601 String) #[serde(defaul… }` | Wurzel-Dokument für den Export aller DB-Collections. | `src/export.rs` |
| `ExportMemoryV1` | `{ /// Eindeutiger Memory-Schlüssel / ID pub id: String, /// Kognitiver Gedächtnistyp (episodic, semantic, procedural, working) #[serde(rename = "type")] pub memo… }` | Export-Repräsentation eines einzelnen Memory-Eintrags. | `src/export.rs` |
| `ExportRelationV1` | `{ /// Quell-Memory ID pub from: String, /// Ziel-Memory ID pub to: String, /// Beziehungs-Label (z.B. "references", "depends_on") pub label: String, }` | Export-Repräsentation einer Graph-Beziehung. | `src/export.rs` |
| `ExtractedTriple` | `{ /// Subjekt der Relation / Entität pub subject: String, /// Prädikat / Relation pub predicate: String, /// Objekt der Relation / Entität pub object: String, //… }` | Extrahiertes Wissens-Tripel (Subjekt, Prädikat, Objekt). | `src/extraction/types.rs` |
| `Failed` | `(tuple/unit)` | — | `src/orchestration/mod.rs` |
| `HybridQueryBuilder` | `{ pub(super) collection: &'a Collection<S, V>, pub(super) text: Option<String>, pub(super) vector: Option<Vec<f32>>, pub(super) k: Option<usize>, pub(super) weig… }` | Fluent query builder for unifying vector, text, graph, and hybrid search operations. | `src/collection/query_builder/builder.rs` |
| `ImportSummary` | `{ /// Anzahl erfolgreich importierter oder aktualisierter Memory-Einträge pub imported_memories: usize, /// Anzahl übersprungener Memory-Einträge pub skipped_mem… }` | Zusammenfassung eines Import-Vorgangs. | `src/import.rs` |
| `MarkdownChunker` | `{ config: ChunkerConfig, }` | A deterministic chunker for markdown files respecting structural blocks (headings, code blocks, tables, paragraphs) and UTF-8 safety. | `src/chunker/mod.rs` |
| `OpenIeExtractor` | `(tuple/unit)` | Implementierung des OpenIE-Extraktors hinter `EntityExtractor`. | `src/extraction/mod.rs` |
| `OrphanCleanupBackoffConfig` | `{ /// Initial cooldown duration after the first failed rebuild attempt. pub base_delay: Duration, /// Maximum cooldown cap between rebuild attempts. pub max_dela… }` | Configuration parameters for HNSW index rebuild backoff and failure escalation. | `src/background_workers/config.rs` |
| `PercolationResult` | `{ pub health: Option<f32>, pub rebonding_triggered: bool, pub new_edges_added: usize, }` | — | `src/collection/maintenance.rs` |
| `Prepared` | `(tuple/unit)` | — | `src/orchestration/mod.rs` |
| `RuleBasedExtractor` | `(tuple/unit)` | Regelbasierter Entitäts- und Relationsextraktor. | `src/extraction/rule_based.rs` |
| `ScopeConstraint` | `{ /// Explicit set of allowed document IDs (BTreeSet guarantees deterministic ordering). pub allowed_doc_ids: BTreeSet<DocId>, /// Gamma augmentation factor for… }` | Scoping constraint defining an explicit hard-boundary set of allowed document IDs for predicate-agnostic vector search (ACORN-Hard-Boundary, Spec §5.… | `src/collection/query_builder/scope.rs` |
| `SignalWeights` | `{ /// Weight for vector similarity signal. pub vector: f32, /// Weight for BM25 text match signal. pub text: f32, /// Weight for graph traversal signal. pub grap… }` | Custom weights for vector, text, and graph signals in hybrid search. | `src/collection/query_builder/weights.rs` |
| `StoredDocument` | `{ pub id: String, pub embedding: Vec<f32>, pub metadata: Option<serde_json::Value>, }` | Dokumentation für `StoredDocument` im Rahmen der Collection- und Suchabstraktion. | `src/collection/mod.rs` |
| `StoredDocumentMeta` | `{ pub id: String, pub metadata: Option<serde_json::Value>, }` | Leichtgewichtige Metadaten (für doc_key, key_type=1) — KEIN Embedding. Wird für DocId-basierte Hydration nach HNSW/BM25-Suche verwendet. | `src/collection/mod.rs` |
| `ValidityWindow` | `{ /// Start of transaction validity (system time / MVCC). pub tx_valid_from: Option<TxId>, /// End of transaction validity (system time / MVCC, exclusive upper b… }` | Extracted bi-temporal validity window representation for a document or search candidate. | `src/temporal_filter.rs` |
| `WorkflowContext` | `{ data: std::collections::HashMap<String, String>, }` | Dynamischer Workflow-Kontext zur Übergabe von Zwischenergebnissen zwischen Schritten. | `src/orchestration/mod.rs` |
| `WorkflowState` | `{ workflow_id: String, steps: Vec<Box<dyn WorkflowStep>>, context: WorkflowContext, completed_steps: usize, error: Option<ContextraError>, _state: PhantomData<St… }` | Typestate-basierte State Machine für Workflows. | `src/orchestration/mod.rs` |

### Öffentliche Datentypen — Enums (8)

| Enum | Varianten | Beschreibung | Datei |
|---|---|---|---|
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
| `FusionResult` | `type FusionResult= contextra_rank::SearchResult` | Type alias for search & fusion results in post-retrieval pipelines. |
| `TenantCollectionMap` | `type TenantCollectionMap= ahash::AHashMap< (TenantId, String), Arc<Collection<contextra_store::tenant_codec::TenantScopedStorage<Arc<LsmStorage>>>>, >` | Typdefinition für die mandantenisolierte Collection-Verwaltung. |

### Konstanten (29)

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
| `INSERT_LATENCY` | `&str` | Metric name for document insert operation duration histogram observations. |
| `MAX_DEFERRED_HYPEREDGES_PER_TICK` | `usize` | Maximum number of deferred hyperedges processed in a single worker tick. |
| `MAX_EXPIRED_PER_TICK` | `usize` | Maximum number of expired documents processed in a single expiry cleanup tick. |
| `MAX_EXTRACTED_TRIPLES` | `usize` | Maximale Anzahl extrahierter Tripel pro Durchlauf. |
| `MAX_EXTRACTION_TEXT_LENGTH` | `usize` | Maximale zulässige Eingabelänge in Zeichen für die regelbasierte Extraktion. |
| `MAX_FILTER_DEPTH` | `usize` | Maximale Rekursionstiefe für verschachtelte Filterausdrücke (Schutz vor Stack Overflow). |
| `MAX_FILTER_NODES` | `usize` | Maximale Anzahl von Filterknoten (Schutz vor DoS / unbeschränkter Allokation). |
| `MAX_IMPORT_COLLECTIONS` | `usize` | Maximum allowable collections in an import document (bounded resource principle). |
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

### Öffentliche Funktionen (319)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn new(storage: Arc<S>, namespace: &str) -> Self` | Creates a new `Bm25Scorer` with the given storage and namespace. | `src/adapters/text.rs` |
| `pub fn calculate_rebuild_cooldown( consecutive_failures: u32, base_delay: Duration, max_delay: Duration, ) -> Duration` | Helper calculating exponential backoff cooldown given consecutive failures. | `src/background_workers/config.rs` |
| `pub fn start_decay_cleanup_worker<S: StorageEngine, V: VectorIndex>( collection: Arc<Collection<S, V>>, decay_config: DecayControllerConfig, interval: Duration, cancel_token: tokio_util::sync::CancellationToken, ) -> tokio::task::JoinHandle<()>` | Starts a background task for decay-controller-driven importance-score eviction. Nur aktiv wenn `background-maintenance` Feature-Flag gesetz… | `src/background_workers/expiry_workers.rs` |
| `pub fn start_expiry_cleanup_worker<S: StorageEngine>( collection: Arc<Collection<S>>, interval: Duration, cancel_token: tokio_util::sync::CancellationToken, ) -> tokio::task::JoinHandle<()>` | Starts a background task to periodically clean up expired documents with TTL. | `src/background_workers/expiry_workers.rs` |
| `pub fn start_expiry_reaper<S: StorageEngine>( collection: Arc<Collection<S>>, interval: Duration, cancel_token: tokio_util::sync::CancellationToken, ) -> tokio::task::JoinHandle<()>` | Deprecated legacy alias for `start_expiry_cleanup_worker`. | `src/background_workers/expiry_workers.rs` |
| `pub fn start_thermostat_reaper<S: StorageEngine, V: VectorIndex>( collection: Arc<Collection<S, V>>, decay_config: DecayControllerConfig, interval: Duration, cancel_token: tokio_util::sync::CancellationToken, ) -> tokio::task::JoinHandle<()>` | Deprecated legacy alias for `start_decay_cleanup_worker`. | `src/background_workers/expiry_workers.rs` |
| `pub async fn drain_up_to(&self, max: usize) -> Vec<HyperEdgeId>` | Drains up to `max` hyperedge IDs from the front of the queue (FIFO). | `src/background_workers/hyperedge_worker.rs` |
| `pub async fn enqueue(&self, ids: impl IntoIterator<Item = HyperEdgeId>)` | Enqueues hyperedge IDs for deferred background tombstoning. | `src/background_workers/hyperedge_worker.rs` |
| `pub fn new() -> Self` | Creates a new empty `DeferredHyperedgeQueue`. | `src/background_workers/hyperedge_worker.rs` |
| `pub fn start_hyperedge_cascade_deferred_worker<S: StorageEngine>( collection: Arc<Collection<S>>, queue: Arc<DeferredHyperedgeQueue>, interval: Duration, cancel_token: tokio_util::sync::CancellationToken, ) -> tokio::task::JoinHandle<()>` | Starts a background task to process deferred hyperedge tombstones. | `src/background_workers/hyperedge_worker.rs` |
| `pub fn reap_orphans_step<T: Clone + Send + Sync + 'static>( buffer: &TxBuffer<T>, limit: usize, ) -> Vec<contextra_types::TxId>` | Performs a single synchronous step of orphan transaction reaping. | `src/background_workers/orphan_workers.rs` |
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
| `pub fn embedder(&self) -> Option<Arc<dyn TextEmbeddingEngine>>` | Returns a clone of the configured embedder engine, if set. | `src/collection/mod.rs` |
| `pub fn ensure_importance_metadata( metadata: &mut Option<serde_json::Value>, tx: TxId, text_opt: Option<&str>, )` | Ensures document metadata contains a valid `MemoryImportance` JSON payload. | `src/collection/mod.rs` |
| `pub fn extract_effective_importance(metadata: &Option<serde_json::Value>, now_tx: TxId) -> f32` | Extracts the effective importance score of a document at a given transaction ID. | `src/collection/mod.rs` |
| `pub fn graph_index(&self) -> Arc<CsrGraph>` | Returns the CSR graph index for this collection. | `src/collection/mod.rs` |
| `pub fn graph_port(&self) -> Arc<dyn contextra_ports::GraphIndex>` | Returns the abstract graph index port for this collection (M3 Trait-Decoupling). | `src/collection/mod.rs` |
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
| `pub async fn drop_collection( &self, name: &str, tenant_id: TenantId, ) -> Result<()>` | Dokumentation für `drop_collection` im Rahmen der Transaktions- und Facade-Orchestrierung. Führt die rein mechanische Löschung einer Collec… | `src/contextra_impl/collections.rs` |
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
| `pub fn effective_half_life_ms(&self, temperature: f32) -> f32` | Berechnet die temperaturangepasste effektive Halbwertszeit in Millisekunden. | `src/decay_controller.rs` |
| `pub fn effective_score( &self, base_score: f32, elapsed_tx: u64, inputs: &DecaySignalInputs, ) -> f32` | Berechnet den aktuellen abgewerteten Score eines Objekts unter Berücksichtigung von Alter und Systemtemperatur. | `src/decay_controller.rs` |
| `pub fn effective_score_wall_clock( &self, base_score: f32, created_at_ms: u64, inputs: &DecaySignalInputs, ) -> f32` | Berechnet den abgewerteten Score basierend auf der Zeitdifferenz zur injizierten Clock. | `src/decay_controller.rs` |
| `pub fn new(config: DecayControllerConfig) -> Self` | Erstellt eine neue Instanz des `AdaptiveDecayController` mit der gegebenen Konfiguration. | `src/decay_controller.rs` |
| `pub fn set_clock(&mut self, clock: Arc<dyn Clock>)` | Setzt oder aktualisiert die injizierte Clock. | `src/decay_controller.rs` |
| `pub fn should_evict( &self, base_score: f32, elapsed_tx: u64, inputs: &DecaySignalInputs, ) -> bool` | Prüft, ob ein Eintrag aufgrund seines verfallenen Scores unter den Eviktions-Schwellenwert gefallen ist. | `src/decay_controller.rs` |
| `pub fn should_evict_wall_clock( &self, base_score: f32, created_at_ms: u64, inputs: &DecaySignalInputs, ) -> bool` | Prüft, ob ein Eintrag unter Verwendung der injizierten Clock eviktiert werden soll. | `src/decay_controller.rs` |
| `pub fn system_temperature(&self, inputs: &DecaySignalInputs) -> f32` | Berechnet die aktuelle Systemtemperatur basierend auf Tombstone-Verhältnis und Abfragelast. | `src/decay_controller.rs` |
| `pub fn with_clock(config: DecayControllerConfig, clock: Arc<dyn Clock>) -> Self` | Erstellt eine neue Instanz des `AdaptiveDecayController` mit injizierter Clock (B11). | `src/decay_controller.rs` |
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
| `pub fn validate_bounds(&self) -> Result<()>` | Validiert, dass der `MetadataFilter` die Tiefen- und Knotengrenzen einhält. | `src/filter.rs` |
| `pub fn validate_filter_expr_bounds(expr: &FilterExpr) -> Result<()>` | Validiert, dass ein `FilterExpr` die Tiefen- und Knotengrenzen einhält. | `src/filter.rs` |
| `pub fn fuse_search_results_with_signal_strategies( result_sets: Vec<(String, Vec<SearchResult>, f32)>, max_results: usize, priority: MetadataMergePriority, include_provenance: bool, resonance_config: Option<&ResonanceConfig>, strategies: contextra_types::Sign…` | Fuses search result sets using per-signal fusion strategies (`SignalFusionStrategies`). | `src/fusion.rs` |
| `pub fn fuse_search_results_with_strategy( result_sets: Vec<(String, Vec<SearchResult>, f32)>, max_results: usize, priority: MetadataMergePriority, include_provenance: bool, resonance_config: Option<&ResonanceConfig>, strategy: contextra_types::FusionStrategy,…` | Fuses search result sets using the specified FusionStrategy. | `src/fusion.rs` |
| `pub fn fuse_signals( result_sets: Vec<(String, Vec<SearchResult>, f32)>, max_results: usize, ) -> Vec<SearchResult>` | Convenience function for fusing multi-signal search results. | `src/fusion.rs` |
| `pub fn weights_to_signal_factors( weights: Option<&contextra_types::FusionWeights>, ) -> (f32, f32, f32)` | Converts optional [`contextra_types::FusionWeights`] into `(vector, text, graph)` factors. | `src/fusion.rs` |
| `pub async fn import_memories(&self, col_export: ExportCollectionV1) -> Result<ImportSummary>` | Importiert alle Memories und Graph-Relationen einer ExportCollectionV1 idempotent (Upsert-Semantik). | `src/import.rs` |
| `pub async fn import_memories(&self, doc: ExportDocumentV1) -> Result<ImportSummary>` | Importiert ein ExportDocumentV1 in die Contextra-Datenbank mit Schema-Versionsprüfung und Idempotenz. | `src/import.rs` |
| `pub fn config(&self) -> &ContextraConfig` | Returns a reference to the active [`ContextraConfig`]. | `src/lib.rs` |
| `pub fn inner_storage(&self) -> Arc<LsmStorage>` | Gibt den zugrundeliegenden LSM-Speicher zurück (intern/verborgen). | `src/lib.rs` |
| `pub fn license_gate(&self) -> Arc<dyn LicenseGate>` | Returns a reference to the attached [`LicenseGate`]. | `src/lib.rs` |
| `pub fn register_consolidation_launcher<F>(_f: F) where F: Fn( Arc<Collection<LsmStorage>>, std::time::Duration, usize, tokio_util::sync::CancellationToken, ) -> tokio::task::JoinHandle<()> + Send + Sync + 'static,` | — | `src/lib.rs` |
| `pub fn with_consolidation_launcher(mut self, launcher: ConsolidationLauncher) -> Self` | Konfiguriert den optionalen Konsolidierungs-Launcher auf dem Builder. | `src/lib.rs` |
| `pub fn with_metrics_sink(mut self, sink: Arc<dyn contextra_ports::MetricsSink>) -> Self` | Konfiguriert die Metrik-Senke zur Erfassung von Systemmetriken. | `src/lib.rs` |
| `pub fn with_tenant_policy(mut self, policy: TenantPolicy) -> Self` | Konfiguriert die Mandantenisolations- und Zugriffspolicy. | `src/lib.rs` |
| `pub fn record_counter_safe( sink: &dyn contextra_ports::MetricsSink, name: &str, value: u64, labels: &[(&str, &str)], )` | Helper to record a counter metric best-effort without allowing panics to propagate. | `src/metrics_names.rs` |
| `pub fn record_gauge_safe( sink: &dyn contextra_ports::MetricsSink, name: &str, value: f64, labels: &[(&str, &str)], )` | Helper to record a gauge metric best-effort without allowing panics to propagate. | `src/metrics_names.rs` |
| `pub fn record_histogram_safe( sink: &dyn contextra_ports::MetricsSink, name: &str, value: f64, labels: &[(&str, &str)], )` | Helper to record a histogram metric best-effort without allowing panics to propagate. | `src/metrics_names.rs` |
| `pub fn add_step(mut self, step: Box<dyn WorkflowStep>) -> Self` | Fügt dem Entwurf einen Schritt hinzu. | `src/orchestration/mod.rs` |
| `pub fn completed_steps(&self) -> usize` | Liefert die Anzahl erfolgreich ausgeführter Schritte. | `src/orchestration/mod.rs` |
| `pub fn completed_steps(&self) -> usize` | Liefert die Anzahl bis zum Abbruch ausgeführter Schritte. | `src/orchestration/mod.rs` |
| `pub fn context(&self) -> &WorkflowContext` | Gibt eine Referenz auf den resultierenden Kontext zurück. | `src/orchestration/mod.rs` |
| `pub fn error(&self) -> Option<&ContextraError>` | Liefert den aufgetretenen Fehler. | `src/orchestration/mod.rs` |
| `pub async fn execute_all( mut self, ) -> std::result::Result<WorkflowState<Completed>, WorkflowState<Failed>>` | Führt alle Schritte sequentiell aus und wechselt je nach Ergebnis in `Completed` oder `Failed`. | `src/orchestration/mod.rs` |
| `pub fn get(&self, key: &str) -> Option<&str>` | Liest einen Wert aus. | `src/orchestration/mod.rs` |
| `pub fn new(storage: Arc<S>, vector_index: Arc<V>, text_index: Arc<T>) -> Self` | Erzeugt einen neuen typisierten Service-Container. | `src/orchestration/mod.rs` |
| `pub fn new() -> Self` | Erstellt einen leeren Kontext. | `src/orchestration/mod.rs` |
| `pub fn new(workflow_id: impl Into<String>) -> Self` | Startet die Erstellung eines neuen Workflows im Zustand `Draft`. | `src/orchestration/mod.rs` |
| `pub fn prepare(self) -> Result<WorkflowState<Prepared>>` | Validiert und überführt den Workflow in den Zustand `Prepared`. | `src/orchestration/mod.rs` |
| `pub fn set(&mut self, key: impl Into<String>, val: impl Into<String>)` | Setzt einen Schlüssel-Wert-Eintrag. | `src/orchestration/mod.rs` |
| `pub fn start_execution(self) -> WorkflowState<Executing>` | Beginnt die Ausführung und wechselt in den Typestate `Executing`. | `src/orchestration/mod.rs` |
| `pub fn storage(&self) -> &Arc<S>` | Liefert Referenz auf die StorageEngine. | `src/orchestration/mod.rs` |
| `pub fn text_index(&self) -> &Arc<T>` | Liefert Referenz auf den TextIndex. | `src/orchestration/mod.rs` |
| `pub fn vector_index(&self) -> &Arc<V>` | Liefert Referenz auf den VectorIndex. | `src/orchestration/mod.rs` |
| `pub fn workflow_id(&self) -> &str` | Liefert die ID des abgeschlossenen Workflows. | `src/orchestration/mod.rs` |
| `pub fn workflow_id(&self) -> &str` | Liefert die ID des fehlgeschlagenen Workflows. | `src/orchestration/mod.rs` |
| `pub fn apply_temporal_validity_filter( results: Vec<contextra_rank::SearchResult>, current_tx: TxId, query_timestamp: Option<u64>, ) -> Vec<contextra_rank::SearchResult>` | Applies bi-temporal validity filtering to candidates post-RRF fusion. Filters out candidate results that are invalid relative to the curren… | `src/temporal_filter.rs` |
| `pub fn apply_temporal_validity_filter_at( results: Vec<contextra_rank::SearchResult>, as_of_timestamp: u64, ) -> Vec<contextra_rank::SearchResult>` | Applies historical "as-of" bi-temporal validity filtering to candidates post-RRF fusion. Evaluates candidate validity at historical point-i… | `src/temporal_filter.rs` |
| `pub fn apply_temporal_validity_filter_with_clock( results: Vec<contextra_rank::SearchResult>, current_tx: TxId, clock: &dyn contextra_ports::Clock, ) -> Vec<contextra_rank::SearchResult>` | Applies bi-temporal validity filtering using an injected `Clock` port trait (B11). | `src/temporal_filter.rs` |
| `pub fn extract_validity_window(metadata: Option<&Value>) -> ValidityWindow` | Extracts bi-temporal validity window metadata from candidate document metadata. | `src/temporal_filter.rs` |
| `pub fn is_empty(&self) -> bool` | Returns `true` if no validity window metadata constraints are set. Candidates without validity window metadata (legacy data) are treated as… | `src/temporal_filter.rs` |
| `pub fn is_valid_bitemporal(&self, as_of_tx: TxId, as_of_business: Option<u64>) -> bool` | Evaluates bi-temporal validity using strict AND logic across system transaction time and business time. Intervall-Semantik: `[valid_from, v… | `src/temporal_filter.rs` |
| `pub fn sort_candidates_deterministic(results: &mut [contextra_rank::SearchResult])` | Sorts candidates deterministically by score descending, breaking ties by document ID ascending (B11). | `src/temporal_filter.rs` |
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

## `contextra-db`

| Attribut | Wert |
|---|---|
| **Ring** | 3 — Ring 3 — Orchestration & Cognition |
| **Status** | In Migration (Legacy-Fassade) |
| **Zweck** | Monolithische Übergangsfassade für Speicher-, Such- und Indexzugriffe. |
| **Unsafe-Code** | Nein (`#![forbid(unsafe_code)]`) |
| **Interne Abhängigkeiten** | `contextra-engine`, `contextra-ports`, `contextra-types` |
| **Externe Abhängigkeiten (Auswahl)** | `ahash`, `blake3`, `bytes`, `parking_lot`, `serde`, `serde_json`, `thiserror`, `tokio`, `tokio-util`, `tracing`, `zeroize` |

### Modul- und Re-Export-Oberfläche (lib.rs)

```rust
pub mod homeostat;
pub mod multistep;
pub mod pid_latency_controller;
pub mod volatile_vault;
pub mod prelude { .. }
pub mod advanced { .. }
pub mod decay_controller { .. }
pub use contextra_engine::collection::{self, Collection, CollectionConfig};
pub use contextra_engine::{ Contextra, ContextraConfig, ContextraStats, DbStats, Document, EmbeddingBackend, HybridQueryBuilder, Language, ProvenanceRecord, SearchResult, SearchStrategy, SignalContribution, SignalWeights, };
pub use contextra_types::{DistanceMetric, DocId, RequestContext, TenantId, TxId};
pub use crate::multistep::{MultiStepConfig, MultiStepEngine, MultiStepResult, QueryRewriter};
pub use crate::volatile_vault::*;
pub use crate::decay_controller::{ AdaptiveDecayController, DecayControllerConfig, DecaySignalInputs, };
pub use crate::homeostat::RerankDeadline;
pub use crate::homeostat::RerankPidController;
pub use crate::pid_latency_controller::{ LatencyBudgetGuard, PidLatencyController, DEFAULT_TARGET_LATENCY_MS, MAX_SCALING_FACTOR, MIN_SCALING_FACTOR, };
pub use contextra_engine as engine;
pub use contextra_engine::DurabilityMode;
pub use contextra_engine::{ background_workers, chunker, export, filter, import, temporal_filter, transaction, CommunityDetectionConfig, ExportCollectionV1, ExportDocumentV1, ExportMemoryV1, ExportRelationV1, ImportSummary, MAX_SCAN_RESULTS, SCHEMA_VERSION_V1, };
pub use advanced::*;
pub use prelude::*;
pub use contextra_engine::decay_controller::*;
pub use contextra_engine::fusion;
pub use volatile_vault::*;
pub use decay_controller::{AdaptiveDecayController, DecayControllerConfig, DecaySignalInputs};
pub use homeostat::RerankDeadline;
pub use homeostat::RerankPidController;
pub use multistep::{MultiStepConfig, MultiStepEngine, MultiStepResult, QueryRewriter};
pub use pid_latency_controller::{ LatencyBudgetGuard, PidLatencyController, DEFAULT_TARGET_LATENCY_MS, MAX_SCALING_FACTOR, MIN_SCALING_FACTOR, };
pub use collection::{Collection, CollectionConfig};
pub use contextra_ports::{DriftStatusProvider, SegmentSynthesizer, TextEmbeddingEngine};
pub use contextra_types::DistanceMetric;
pub use serde_json::json;
```

### Feature-Flags

| Flag | Aktiviert |
|---|---|
| `default` | `[]` |
| `bench` | `[]` |
| `reranking` | `[]` |
| `experimental-diskann` | `["contextra-vector/experimental-diskann"]` |
| `background-maintenance` | `["contextra-engine/background-maintenance"]` |
| `graph-connectivity-health` | `["contextra-engine/graph-connectivity-health"]` |
| `coherence-bonus-fusion` | `[]` |
| `volatile-vault` | `[]` |
| `edge-reinforcement-learning` | `[]` |

### Öffentliche Datentypen — Structs (13)

| Struct | Felder / Form | Beschreibung | Datei |
|---|---|---|---|
| `CommitReceipt` | `{ /// Anzahl geschriebener Chunks. pub chunks_committed: usize, /// Gesamtgröße der übergebenen Nutzdaten in Bytes. pub bytes_committed: usize, /// Mandantenkenn… }` | Quittung nach Übergabe der Chunks an die persistente Datenbank via [`VolatileContextVault::commit()`]. | `src/volatile_vault.rs` |
| `LatencyBudgetGuard` | `{ start_time_nanos: u64, budget_ms: f64, clock: Arc<dyn Clock>, }` | — | `src/pid_latency_controller.rs` |
| `MultiStepConfig` | `{ /// Maximale Anzahl an Iterationsrunden (Standard: 3). pub max_rounds: usize, /// Mindest-Score-Schwellenwert (0.0 bis 1.0): Liegen die Scores darunter, /// wi… }` | Konfiguration für das mehrstufige iterative Multi-Step Retrieval. Steuert die Abbruchbedingungen, Qualitätsanforderungen und Zeitbudgets. | `src/multistep.rs` |
| `MultiStepEngine` | `{ collection: Arc<Collection<S>>, config: MultiStepConfig, pid_controller: parking_lot::Mutex<PidController>, clock: Arc<dyn Clock>, }` | Multi-Step Retrieval Engine für iterative Abfragen. Bietet adaptive, mehrstufige Retrieval-Zyklen auf einer zugrundeliegenden [`Collection`]. Nutzt e… | `src/multistep.rs` |
| `MultiStepResult` | `{ /// Die fusionierten Gesamtsuchergebnisse nach RRF (Reciprocal Rank Fusion). pub results: Vec<SearchResult>, /// Anzahl der tatsächlich durchlaufenen Iteration… }` | Ergebnis einer mehrstufigen Suche inklusive vollständiger Audit- und Nachvollziehbarkeitsdaten. | `src/multistep.rs` |
| `PidLatencyController` | `{ kp: f64, ki: f64, kd: f64, integral: f64, last_error: f64, target_latency_ms: f64, scaling_factor: f64, }` | — | `src/pid_latency_controller.rs` |
| `RerankDeadline` | `{ deadline: Instant, }` | — | `src/homeostat.rs` |
| `RerankPidController` | `{ target_ms: f32, pool_size: usize, }` | — | `src/homeostat.rs` |
| `VaultChunk` | `{ /// Dokumenten-ID (DocId) des Chunks. #[zeroize(skip)] pub id: DocId, // DocId-Wert, ohne komplexe Drop-Interaktion /// Sensitiver Inhalt im Speicher — wird be… }` | Ein einzelner Datenblock (Chunk) im `VolatileContextVault`. Sämtliche Inhalte werden beim Drop via [`ZeroizeOnDrop`] kryptographisch überschrieben. | `src/volatile_vault.rs` |
| `VaultChunkMetadata` | `{ /// Eindeutige ID des Dokuments. pub id: DocId, /// Modalität (Text, Audio etc.). pub modality: SignalModality, /// Größe der Nutzdaten in Bytes. pub size_byte… }` | Metadaten-Ansicht eines Chunks ohne sensiblen Nutzdateninhalt. | `src/volatile_vault.rs` |
| `VaultConfig` | `{ /// Maximale Speicherkapazität aller Chunks in Bytes (Standard: 512 MB). /// Bei Überschreitung schlägt `ingest()` mit [`VaultError::CapacityExceeded`] fehl (F… }` | Konfiguration für den `VolatileContextVault`. | `src/volatile_vault.rs` |
| `VolatileContextVault` | `{ chunks: Vec<VaultChunk>, config: VaultConfig, current_bytes: usize, /// Gekapselte mlock'd Speicherbereiche. /// Wird in munlock() aufgelöst, wenn der Vault dr… }` | Ephemerer RAM-Puffer für sensitive Kontexte im Safe-Modus. # INVARIANTEN - **INV-VAULT-1:** `purge()` zeroized alle Chunk-Inhalte via `ZeroizeOnDrop`… | `src/volatile_vault.rs` |
| `VolatileDrainReceipt` | `{ /// Anzahl überschriebener und verworfener Chunks. pub chunks_purged: usize, /// Exakte Anzahl an Bytes, die im RAM mit Nullen überschrieben wurden. pub bytes_… }` | Quittung nach erfolgreicher flüchtiger Speicherbereinigung via [`VolatileContextVault::purge_memory()`]. | `src/volatile_vault.rs` |

### Öffentliche Datentypen — Enums (2)

| Enum | Varianten | Beschreibung | Datei |
|---|---|---|---|
| `SignalModality` | `/// Audiospur oder Sprach-Transkript. AudioTranscript, /// Visuelle Erfassung, Screenshot oder Bilddaten. VisualCapture, /// Reguläre Texteingabe oder Prompt. TextInput, /// Strukturierte JSON- oder…` | Modalität eines Vault-Chunks — bestimmt den Verarbeitungskontext beim späteren Commit. | `src/volatile_vault.rs` |
| `VaultError` | `/// Kapazitätsgrenze im RAM wurde überschritten (Fail-Closed, kein Disk-Spillover). #[error("Vault-Kapazität überschritten: {current_bytes} + {new_bytes} > {max_bytes}")] CapacityExceeded { current_b…` | Fehlerzustände des flüchtigen Speicher-Puffers. | `src/volatile_vault.rs` |

### Konstanten (5)

| Konstante | Typ | Beschreibung |
|---|---|---|
| `DEFAULT_MAX_SUB_QUERIES` | `usize` | Standardmäßige Obergrenze für generierte Teilfragen pro Iterationsrunde (N=3). Verhindert kombinatorische Explosion und hält Latenzschranken ein. |
| `DEFAULT_RERANK_POOL_MULTIPLIER` | `usize` | Standardmäßiger Multiplikator zur Vergrößerung des Kandidatenpools vor dem Re-Ranking. Verdoppelt die Trefferanzahl (2 * k), um genügend Recall vor F… |
| `DEFAULT_TARGET_LATENCY_MS` | `f64` | — |
| `MAX_SCALING_FACTOR` | `f64` | — |
| `MIN_SCALING_FACTOR` | `f64` | — |

### Öffentliche Funktionen (34)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn is_expired(&self) -> bool` | — | `src/homeostat.rs` |
| `pub fn new(timeout: Duration) -> Self` | — | `src/homeostat.rs` |
| `pub fn new(target_ms: f32, initial_pool: usize) -> Self` | — | `src/homeostat.rs` |
| `pub fn pool_size(&self) -> usize` | — | `src/homeostat.rs` |
| `pub fn remaining(&self) -> Duration` | — | `src/homeostat.rs` |
| `pub fn update(&mut self, elapsed_ms: f32) -> usize` | — | `src/homeostat.rs` |
| `pub fn new(collection: Arc<Collection<S>>, config: MultiStepConfig) -> Self` | Erzeugt eine neue `MultiStepEngine` für eine gegebene [`Collection`]. # Parameter - `collection`: Die Contextra-Kollektion, auf der gesucht… | `src/multistep.rs` |
| `pub async fn search( &self, original_query: &str, vector: &[f32], k: usize, rewriter: Option<&dyn QueryRewriter>, ) -> Result<MultiStepResult>` | Führt eine iterative Hybrid-Suche ohne separaten Vektor-Embedder durch. # Ablauf 1. Runde 1: Standard-Hybrid-Suche mit Originalanfrage und… | `src/multistep.rs` |
| `pub async fn search_with_embedder( &self, original_query: &str, vector: &[f32], k: usize, rewriter: Option<&dyn QueryRewriter>, embedder: Option<&dyn TextEmbeddingEngine>, ) -> Result<MultiStepResult>` | Führt eine iterative multi-signale Hybrid-Suche mit Vektor-Embedder durch. # Ablauf im Detail: 1. Eingabeprüfung & Schranke: `k` wird gegen… | `src/multistep.rs` |
| `pub fn with_clock(mut self, clock: Arc<dyn Clock>) -> Self` | Überschreibt den Zeit-Port mit einem benutzerdefinierten [`Clock`]-Port. # Zweck Ermöglicht deterministische Tests ohne Abhängigkeit von re… | `src/multistep.rs` |
| `pub fn budget_ms(&self) -> f64` | — | `src/pid_latency_controller.rs` |
| `pub fn current_scaling_factor(&self) -> f64` | — | `src/pid_latency_controller.rs` |
| `pub fn elapsed_ms(&self) -> f64` | — | `src/pid_latency_controller.rs` |
| `pub fn is_exceeded(&self) -> bool` | — | `src/pid_latency_controller.rs` |
| `pub fn new(target_latency_ms: f64) -> Self` | — | `src/pid_latency_controller.rs` |
| `pub fn new(budget_ms: f64) -> Self` | — | `src/pid_latency_controller.rs` |
| `pub fn remaining_ms(&self) -> f64` | — | `src/pid_latency_controller.rs` |
| `pub fn reset(&mut self)` | — | `src/pid_latency_controller.rs` |
| `pub fn target_latency_ms(&self) -> f64` | — | `src/pid_latency_controller.rs` |
| `pub fn update(&mut self, elapsed_ms: f64, dt_seconds: f64) -> f64` | — | `src/pid_latency_controller.rs` |
| `pub fn with_clock(budget_ms: f64, clock: Arc<dyn Clock>) -> Self` | — | `src/pid_latency_controller.rs` |
| `pub fn commit( mut self, tenant_id: TenantId, tx_id: TxId, metadata_reference: Option<impl Into<String>>, ) -> Result<(Vec<VaultChunk>, CommitReceipt), VaultError>` | Transferiert alle Chunks zur dauerhaften Speicherung und stellt ein [`CommitReceipt`] aus. # Ablauf von `commit`: 1. Status prüfen: Falls b… | `src/volatile_vault.rs` |
| `pub fn current_size_bytes(&self) -> usize` | Gibt die Summe der aktuell im flüchtigen RAM liegenden Nutzdaten in Bytes zurück. | `src/volatile_vault.rs` |
| `pub fn ingest(&mut self, chunk: VaultChunk) -> Result<(), VaultError>` | Nimmt einen sensitiven Chunk in den RAM-Puffer auf. # Ablauf von `ingest`: 1. Status prüfen: Wurde der Vault bereits konsumiert? Falls ja:… | `src/volatile_vault.rs` |
| `pub fn is_empty(&self) -> bool` | Prüft, ob der Vault leer ist. | `src/volatile_vault.rs` |
| `pub fn is_memory_locked(&self) -> bool` | Gibt an, ob alle Chunk-Inhalte im RAM gegen Swap gesichert sind (`mlock`). Liefert `false`, falls `attempt_mlock` deaktiviert war oder das… | `src/volatile_vault.rs` |
| `pub fn len(&self) -> usize` | Gibt die Anzahl der aktuell gepufferten Chunks zurück. | `src/volatile_vault.rs` |
| `pub fn new(id: DocId, content: Vec<u8>, modality: SignalModality, captured_tx: TxId) -> Self` | Erzeugt einen neuen sensitiven Chunk. # Parameter - `id`: Eindeutige Dokumenten-ID. - `content`: Rohe Nutzdaten (z. B. Text- oder Binärdate… | `src/volatile_vault.rs` |
| `pub fn open(config: VaultConfig) -> Self` | Öffnet einen neuen leeren Vault im RAM. Führt keinerlei Datei-I/O oder Festplattenzugriffe aus. | `src/volatile_vault.rs` |
| `pub fn preview_metadata(&self) -> Vec<VaultChunkMetadata>` | Gibt eine Vorschau der Metadaten zurück, ohne sensible Inhalte zu exponieren. Verhindert versehentliches Ausleiten oder Protokollieren von… | `src/volatile_vault.rs` |
| `pub fn purge(self) -> VolatileDrainReceipt` | Neutralisierte Speicherleeren-Methode (Alias für [`Self::purge_memory`]). | `src/volatile_vault.rs` |
| `pub fn purge_memory(mut self) -> VolatileDrainReceipt` | Verwirft alle Chunks sofort und überschreibt deren Heap-Inhalte kryptographisch mit Nullen. # Ablauf von `purge`: 1. Alle Chunk-Inhalte im… | `src/volatile_vault.rs` |
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
| **Interne Abhängigkeiten** | `contextra-db`, `contextra-ports`, `contextra-types` |
| **Externe Abhängigkeiten (Auswahl)** | `serde`, `serde_json`, `thiserror`, `tokio`, `tracing`, `uuid`, `zeroize` |

### Architekturbeschreibung

# Modul: Contextra Primary Facade & Composition Root (Ring 4)  Dieses Modul bildet die oberste Einstiegsebene (Fassade bzw. Composition Root) des Gesamtsystems **Contextra** innerhalb der Architektur (Ring 4).  ## Zweck & Architektur Die Fassade dient als zentrale Abstraktions- und Konfigurationsschicht für anwendende Systeme (wie z. B. `contextra-mcp` oder externe SDKs/Applikationen). Sie verbirgt die interne Komplexität der tieferen Schichten (Ring 0 bis 3) und stellt eine einheitliche, ergonomische Programmierschnittstelle (API = Application Programming Interface) bereit.  ## Ring-Zugehörigkeit & Abhängigkeiten - **Ring 4 (Fassade)**: Greift als Composition Root auf Teilsysteme wie `contextra-db` (Ring 3), `contextra-types` und `contextra-ports` zu. - **Keine zyklischen Abhängigkeiten**: Höhere Ringe hängen von niederen Ringen ab, nie umgekehrt. - **Eingebunden von**: `contextra-mcp` (Model Context Protocol Adapter für LLM-Agenten).  ## Kernkonzepte & wichtigste Typen - [`ContextraBuilder`]: Konfiguriert Speicherpfade, Vektordimensionen, Verschlüsselung, Lizenzprüfung und Profile vor dem Start der Datenbankinstanz. - [`Contextra`]: Die eigentliche Datenbankinstanz zur Verwaltung von Collections (Dokumentensammlungen), Vektorindizes und Volltextindizes. - [`AgentMemory`]: Eine agentenzentrierte High-Level-Abstraktion mit `remember()`, `recall()`, `forget()` und Wissensgraphen-Relationen (`relate()`). - [`LlmQueryRewriter`]: LLM-gestützte Umformulierung von Suchanfragen für mehrstufiges Retrieval.  ## Sicherheits- und Entwurfs-Invarianten - **Kein Unsafe**: Striktes `#![forbid(unsafe_code)]` garantiert Speichersicherheit zur Compile-Zeit. - **Kein Panik-Zustand**: Fehler werden strikt über den Ergebnistyp [`Result`] bzw. [`ContextraError`] nach oben gereicht (Fail-Closed-Prinzip). - **Entkopplung**: Zeit-, Zufalls- und ID-Generierungsoperationen erfolgen deterministisch über Ports.

### Modul- und Re-Export-Oberfläche (lib.rs)

```rust
pub mod agent_memory;
pub mod boot;
pub mod builder;
pub mod collection_profile;
pub mod performance_profile;
pub mod query_rewriter;
pub use agent_memory::{AgentMemory, Memory, MemoryId};
pub use builder::{ContextraBuilder, ContextraConfigExt, EncryptionMode};
pub use collection_profile::{CollectionProfile, DeploymentTier, KvDeleteMode, LsmTuning};
pub use contextra_db::{ chunker, Collection, CollectionConfig, Contextra, ContextraConfig, ContextraStats, Document, DriftStatusProvider, EmbeddingBackend, SearchResult, TextEmbeddingEngine, };
pub use contextra_types::error::ContextraError;
pub use contextra_types::types::domain::{DocId, ScoredDocument};
pub use contextra_types::DistanceMetric;
pub use performance_profile::{DurabilityConfigError, DurabilityMode, PerformanceProfile};
pub use query_rewriter::LlmQueryRewriter;
```

### Feature-Flags

| Flag | Aktiviert |
|---|---|
| `default` | `["embedded"]` |
| `embedded` | `[]` |

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
| `ResolvedProfileConfig` | `{ pub durability_mode: DurabilityMode, pub vector_delete_mode: VectorDeleteMode, pub integrity_proof_active: bool, pub feature_ring: FeatureRing, }` | Vollständig aufgelöste Konfigurationsparameter eines Performance-Profils. | `src/performance_profile.rs` |

### Öffentliche Datentypen — Enums (9)

| Enum | Varianten | Beschreibung | Datei |
|---|---|---|---|
| `BootPhase` | `/// B0: Konfiguration, Tier und Profil prüfen B0ConfigValidation, /// B1: Datenverzeichnis und Dateisystem-Voraussetzungen B1StoragePath, /// B2: Schlüssel und KDF-Parameter B2KeyProvider, /// B3: Si…` | Repräsentiert die normativen Startphasen B0–B9 gemäß SPEC §13.5. | `src/boot.rs` |
| `CollectionProfileError` | `#[error("performance profile invalid: {0}")] Performance(#[from] PerformanceProfileError), #[error("integrity-proof active but KvDeleteMode::TombstoneOnly cannot satisfy a KV-side IntegrityProof")] K…` | Mögliche Fehler bei der Validierung eines `CollectionProfile`. | `src/collection_profile.rs` |
| `DeploymentTier` | `EdgeMinimal, PowerUserLocal, EnterpriseShared, EnterpriseRegulated,` | Vordefinierte Betriebsstufen (Deployment Tiers) für typische Anwendungsszenarien. | `src/collection_profile.rs` |
| `DurabilityConfigError` | `#[error("DurabilityMode::{mode:?} is incompatible with feature '{feature}': {reason}")] IncompatibleCombination { mode: DurabilityMode, feature: &'static str, reason: &'static str, },` | Fehler bei der Validierung des DurabilityMode gegen Ring- und Integritätsfeatures. | `src/performance_profile.rs` |
| `DurabilityMode` | `Full, WalNoHmac, MemoryOnly,` | Durability mode for WAL persistence and integrity checks. | `src/performance_profile.rs` |
| `EncryptionMode` | `/// Storage is encrypted using a key derived from a passphrase. Passphrase(Zeroizing<String>), /// Storage is explicitly unencrypted. Unencrypted,` | Konfiguration der Speicherverschlüsselung (Encryption at Rest) für persistente Datenbankschichten. Bestimmt, ob Schreib-Lese-Operationen auf Festplat… | `src/builder.rs` |
| `KvDeleteMode` | `TombstoneOnly, CryptoShred,` | Mode specifying KV segment deletion behavior. | `src/collection_profile.rs` |
| `PerformanceProfile` | `Compliance, Balanced, BareMetal,` | Vordefinierte Leistungsprofile für Contextra-Instanzen. | `src/performance_profile.rs` |
| `PerformanceProfileError` | `#[error("durability configuration invalid: {0}")] Durability(#[from] DurabilityConfigError), #[error("DurabilityMode::Full combined with VectorDeleteMode::BackgroundRepair while integrity-proof is ac…` | Fehler bei der Validierung oder Lizenzierung eines Performance-Profils. | `src/performance_profile.rs` |

### Konstanten (2)

| Konstante | Typ | Beschreibung |
|---|---|---|
| `DEFAULT_AUTO_EXTRACTION_MODE` | `AutoExtractionMode` | Standardmodus für die automatische LLM-gestützte Metadaten-/Entitäten-Extraktion bei Standard-Tiers. Standardmäßig aktiviert (`AutoExtractionMode::En… |
| `ENTERPRISE_REGULATED_AUTO_EXTRACTION_DEFAULT` | `AutoExtractionMode` | Standardmodus für `EnterpriseRegulated`. In EnterpriseRegulated wird kein Dokumenttext automatisch an ein Extraktions-LLM gegeben, solange der Betrei… |

### Öffentliche Funktionen (42)

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
| `pub fn execute_boot_sequence( path: &Path, dimension: usize, passphrase_configured: bool, empty_passphrase: bool, ) -> Result<(), ContextraError>` | Führt die normativen Startphasen B0 bis B9 in fester Reihenfolge aus. | `src/boot.rs` |
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
| `pub fn with_license_gate(self, _gate: Arc<dyn LicenseGate>) -> Self` | Injiziert ein beliebiges Lizenzgate ([`LicenseGate`]) zur Ring-Autorisierung. # Veraltet Die Lizenzüberprüfung wurde entfernt. Diese Method… | `src/builder.rs` |
| `pub fn with_max_elements(mut self, max_elements: usize) -> Self` | Begrenzt die maximale Anzahl an Vektorelementen im Index. # Parameter - `max_elements`: Maximalwert (z. B. 100_000). | `src/builder.rs` |
| `pub fn with_performance_profile( mut self, profile: crate::performance_profile::PerformanceProfile, ) -> Self` | Setzt ein vordefiniertes Performance-Profil ([`PerformanceProfile`]). Profile (wie `Compliance`, `Balanced`, `BareMetal`) steuern standardi… | `src/builder.rs` |
| `pub fn with_storage_path(mut self, path: impl Into<PathBuf>) -> Self` | Setzt das Speicherverzeichnis für die Datenbankdateien (LSM-SSTables, WAL, Indizes). # Parameter - `path`: Pfad im Dateisystem (z. B. `/var… | `src/builder.rs` |
| `pub fn resolve(self) -> CollectionProfile` | Löst die Betriebsstufe in ein konkretes, vorkonfiguriertes [`CollectionProfile`] auf. Garantiert stets ein konsistentes Profil gemäß Invari… | `src/collection_profile.rs` |
| `pub fn validate(&self) -> Result<(), CollectionProfileError>` | Validiert die interne Konsistenz des Profils. # Ablauf von `validate`: 1. Performance-Profil auflösen und dessen Konfiguration prüfen (`res… | `src/collection_profile.rs` |
| `pub fn validate_with_license( &self, _gate: &dyn LicenseGate, ) -> Result<(), CollectionProfileError>` | Validiert das Profil zusätzlich gegen ein Lizenzgate. # Veraltet Die Lizenzprüfung wurde entfernt. Diese Methode ist ein No-op bezüglich de… | `src/collection_profile.rs` |
| `pub fn builder(dimension: usize) -> ContextraBuilder` | Erzeugt einen neuen [`ContextraBuilder`] zur Konfiguration und Instanziierung der Datenbank [`Contextra`]. # Parameter - `dimension`: Die A… | `src/lib.rs` |
| `pub async fn open(path: impl AsRef<std::path::Path>) -> Result<Contextra, ContextraError>` | Öffnet oder erstellt eine [`Contextra`]-Datenbankinstanz am angegebenen Dateisystempfad mit Standardkonfiguration. Verwendet standardmäßig… | `src/lib.rs` |
| `pub async fn open_with_config( path: impl AsRef<std::path::Path>, config: ContextraConfig, ) -> Result<Contextra, ContextraError>` | Öffnet oder erstellt eine [`Contextra`]-Datenbankinstanz mit einer explizit übergebenen Konfiguration. # Parameter - `path`: Dateipfad zum… | `src/lib.rs` |
| `pub fn enforce_license(&self, _gate: &dyn LicenseGate) -> Result<(), PerformanceProfileError>` | Prüft den geforderten Feature-Ring gegen das übergebene [`LicenseGate`]. # Veraltet Lizenzprüfung wurde entfernt; diese Methode ist ein No-… | `src/performance_profile.rs` |
| `pub fn resolve(self) -> ResolvedProfileConfig` | Löst das Profil in konkrete Konfigurationsparameter auf. # Ablauf von `resolve`: - `Compliance`: Full Durability, SynchronousRepair, Integr… | `src/performance_profile.rs` |
| `pub fn validate(&self) -> Result<(), PerformanceProfileError>` | Validiert die gegenseitige Kompatibilität der Profilparameter. Stellt sicher, dass Durability und Vektor-Bereinigung nicht in einen inkonsi… | `src/performance_profile.rs` |
| `pub fn validate_against_features( self, integrity_proof_active: bool, feature_ring: FeatureRing, ) -> Result<(), DurabilityConfigError>` | Validiert die Kombination aus DurabilityMode, Integritätsnachweisen und Feature-Ring. | `src/performance_profile.rs` |
| `pub fn new(generator: Arc<dyn LlmTextGenerator>) -> Self` | Erstellt einen neuen `LlmQueryRewriter` mit Standardgrenzwerten: - `max_subqueries`: 3 alternative Abfragen - `max_context_results`: 3 Doku… | `src/query_rewriter.rs` |
| `pub async fn rewrite_query( &self, original_query: &str, current_results: &[ScoredEntry], ) -> Result<Vec<String>>` | Öffentlicher Einstiegspunkt zur asynchronen Umformulierung einer Suchanfrage. | `src/query_rewriter.rs` |
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
| **Interne Abhängigkeiten** | `contextra`, `contextra-crypto`, `contextra-ports`, `contextra-rank`, `contextra-types` |
| **Externe Abhängigkeiten (Auswahl)** | `futures-util`, `parking_lot`, `rand`, `scc`, `serde`, `serde_json`, `thiserror`, `tokio`, `tokio-util`, `tracing`, `tracing-subscriber`, `unicode-normalization`, `zeroize` |

### Modul- und Re-Export-Oberfläche (lib.rs)

```rust
pub mod config;
pub mod explain;
pub mod io;
pub mod prompt_injection;
pub mod protocol;
pub mod sandbox;
pub mod server;
pub mod server_dispatch;
pub mod server_tools;
pub mod tools_crud;
pub mod validation;
pub use config::*;
pub use explain::{ExplainRequest, ExplainResponse};
pub use io::{read_line_bounded, MAX_RPC_BYTES, MAX_SEARCH_QUERY_BYTES};
pub use prompt_injection::{ PromptInjectionConfig, PromptInjectionGuard, QuarantinePolicy, SecurityAuditLogger, SecurityAuditRecord, DEFAULT_REDACTION_PLACEHOLDER, };
pub use server::McpServer;
pub use validation::{is_write_allowed_by_env, validate_collection_name};
```

### Feature-Flags

| Flag | Aktiviert |
|---|---|
| `default` | `[]` |
| `test-utils` | `[]` |

### Öffentliche Datentypen — Structs (19)

| Struct | Felder / Form | Beschreibung | Datei |
|---|---|---|---|
| `EmbeddingConfig` | `{ /// Provider type ("ollama", "onnx", "candle", or "mock"). Unconfigured when empty. pub provider: String, /// Base URL for Ollama HTTP API. pub ollama_url: Str… }` | Embedding provider configuration settings. | `src/config.rs` |
| `ExplainRequest` | `{ /// Target document identifier. pub id: String, /// Target collection name (defaults to "default"). #[serde(default = "default_collection")] pub collection: St… }` | Request payload for `contextra_explain`. | `src/explain.rs` |
| `ExplainResponse` | `{ /// Document identifier. pub id: String, /// Collection name. pub collection: String, /// Detailed provenance record if available. #[serde(default, skip_serial… }` | Response payload for `contextra_explain`. | `src/explain.rs` |
| `JsonRpcError` | `{ pub code: i32, pub message: String, #[serde(skip_serializing_if = "Option::is_none")] pub data: Option<Value>, }` | — | `src/protocol.rs` |
| `JsonRpcRequest` | `{ pub jsonrpc: String, #[serde(default)] pub id: Option<Value>, pub method: String, #[serde(default)] pub params: Value, }` | — | `src/protocol.rs` |
| `JsonRpcResponse` | `{ pub jsonrpc: String, pub id: Option<Value>, #[serde(skip_serializing_if = "Option::is_none")] pub result: Option<Value>, #[serde(skip_serializing_if = "Option:… }` | — | `src/protocol.rs` |
| `LlmConfig` | `{ /// Provider type ("ollama", "candle", or "mock"). Unconfigured when empty. pub provider: String, /// Base URL for Ollama HTTP API. pub ollama_url: String, ///… }` | LLM provider configuration settings. | `src/config.rs` |
| `McpSandbox` | `{ policy: SandboxPolicy, /// Session-lokale volatile Ergebnisse (werden bei Session-Ende gedropt). volatile_results: Mutex<HashMap<String, VolatileToolResult>>,… }` | MCP Sandbox: validiert Tool-Calls und verwaltet volatile Ergebnisse. | `src/sandbox.rs` |
| `McpServer` | `{ pub db: Arc<Contextra>, pub embedder: Arc<dyn EmbeddingProvider>, pub sandbox: Arc<McpSandbox>, pub injection_guard: Arc<PromptInjectionGuard>, pub plugin_regi… }` | — | `src/server.rs` |
| `MockEmbedder` | `{ dimension: usize, all_zero: bool, }` | Fallback Mock Embedder (Test & dev utilities only). | `src/config.rs` |
| `MockLlmGenerator` | `(tuple/unit)` | Fallback Mock LLM Text Generator (Test & dev utilities only). | `src/config.rs` |
| `NormalizedPattern` | `{ pub original: String, pub collapsed: String, pub no_ws: String, }` | Vornormalisiertes Injection-Pattern zur Performance-Optimierung. | `src/prompt_injection/guard.rs` |
| `PromptInjectionConfig` | `{ #[serde(default)] pub policy: QuarantinePolicy, #[serde(default = "default_redaction_placeholder")] pub redaction_placeholder: String, #[serde(default)] pub au… }` | Konfiguration zur Initialisierung aus einer externen Konfigurationsdatei. | `src/prompt_injection/policy.rs` |
| `PromptInjectionGuard` | `{ policy: QuarantinePolicy, pub redaction_placeholder: String, patterns: Vec<String>, normalized_patterns: Vec<NormalizedPattern>, audit_logger: SecurityAuditLog… }` | Signatur- und phrasenbasierter Prompt-Injection-Erkennungsfilter. Erkennt bekannte Angriffsmuster (direkte Phrasen, whitespace-verschleierte Variante… | `src/prompt_injection/guard.rs` |
| `SandboxPolicy` | `{ pub allow_db_reads: bool, pub allow_db_writes: bool, pub allow_code_execution: bool, /// Cloud-Egress erlaubt? Standardmäßig `false` (additiv zu Layer 1/4 Egre… }` | Konfiguration der Sandbox-Policy. | `src/sandbox.rs` |
| `SecurityAuditLogger` | `{ file_path: Option<PathBuf>, in_memory_records: Arc<Mutex<Vec<SecurityAuditRecord>>>, }` | Separater Audit-Logger für Sicherheitsvorfälle (vollständig isoliert vom Vektor-Index). | `src/prompt_injection/audit.rs` |
| `SecurityAuditRecord` | `{ pub timestamp: String, pub event_type: String, pub doc_id: String, pub collection: String, pub pattern_matched: String, pub action_taken: String, }` | Struktur für Sicherheits-Audit-Logeinträge bei Auslösung des `Escalate`-Modus. | `src/prompt_injection/audit.rs` |
| `ToolDefinition` | `{ pub name: &'static str, pub category: ToolCategory, pub description: &'static str, pub input_schema: fn() -> Value, }` | Definition eines MCP-Tools mit Name, Kategorie, Beschreibung und JSON-Schema. | `src/sandbox.rs` |
| `VolatileToolResult` | `{ /// Verschlüsselte Ausgabe (AES-256-GCM-SIV via contextra-security). encrypted: zeroize::Zeroizing<Vec<u8>>, /// Klartext-Nonce (nicht sensitiv). nonce: Vec<u8… }` | Volatiler Tool-Output-Speicher mit Zeroize-Garantie bei Drop. Tool-Ergebnisse werden verschlüsselt im Arbeitsspeicher gehalten. Beim Drop wird der Sp… | `src/sandbox.rs` |

### Öffentliche Datentypen — Enums (3)

| Enum | Varianten | Beschreibung | Datei |
|---|---|---|---|
| `McpError` | `#[error("{0}")] ParseError(String), #[error("{0}")] InvalidRequest(String), #[error("{0}")] MethodNotFound(String), #[error("{0}")] InvalidParams(String), #[error("{0}")] InternalError(String),` | MCP Error representation matching standard JSON-RPC 2.0 error codes. | `src/protocol.rs` |
| `QuarantinePolicy` | `/// Verdächtige Texte werden durch einen neutralen Platzhalter ersetzt (Default). #[default] Strict, /// **WARNUNG (UNSICHER)**: Text wird unverändert durchgereicht, lediglich das Flag /// `suspiciou…` | Konfigurierbare Quarantäne-Policy für die Prompt-Injection-Behandlung. **SICHERHEITSHINWEIS & HEURISTIK-LIMITATION**: Pattern-Matching und Normalisie… | `src/prompt_injection/policy.rs` |
| `ToolCategory` | `/// Read-only Datenbankoperationen (immer erlaubt). DatabaseRead, /// Schreibende Datenbankoperationen (erfordern explizite Freigabe). DatabaseWrite, /// Externe Code-Ausführung (höchste Risikostufe,…` | Erlaubte MCP-Tool-Kategorien (Whitelist-Prinzip). | `src/sandbox.rs` |

### Konstanten (10)

| Konstante | Typ | Beschreibung |
|---|---|---|
| `DEFAULT_REDACTION_PLACEHOLDER` | `&str` | Neutraler Standard-Platzhalter für als verdächtig erkannten Text im Strict/Escalate-Modus. |
| `MAX_BATCH_SIZE` | `usize` | Maximale Anzahl von Elementen in einem JSON-RPC Batch-Request (Schutz gegen Batch-DoS). |
| `MAX_JSON_DEPTH` | `usize` | Maximale Rekursionstiefe für eingehende JSON-RPC Payloads (Schutz gegen Stack-Overflow). |
| `MAX_RECURSION_DEPTH` | `usize` | — |
| `MAX_RPC_BYTES` | `usize` | Maximum allowed single JSON-RPC message size via stdio (4 MB). |
| `MAX_SEARCH_QUERY_BYTES` | `usize` | Maximum allowed search query length in bytes (64 KB). |
| `MAX_VOLATILE_KEY_BYTES` | `usize` | Maximale Länge eines Volatile-Result-Schlüssels in Bytes. |
| `MAX_VOLATILE_OUTPUT_BYTES` | `usize` | Maximale Größe einer volatilen Tool-Ausgabe in Bytes (16 MB). |
| `MAX_VOLATILE_RESULTS` | `usize` | Maximale Anzahl von volatilen Ergebnissen pro Sandbox-Session. |
| `TOOL_REGISTRY` | `&[ToolDefinition]` | Single Source of Truth für MCP-Tool-Registrierung, Discovery (`tools/list`) und -Klassifizierung. ## Tool-Kategorie-Tabelle | Werkzeug | Kategorie |… |

### Öffentliche Funktionen (71)

| Signatur | Beschreibung | Datei |
|---|---|---|
| `pub fn available_compiled_providers() -> Vec<&'static str>` | Returns the list of currently compiled provider identifiers. | `src/config.rs` |
| `pub fn build_generator(&self) -> Result<Arc<dyn LlmTextGenerator>, ContextraError>` | Instantiates the configured `LlmTextGenerator` as an `Arc<dyn LlmTextGenerator>`. | `src/config.rs` |
| `pub fn build_provider(&self) -> Result<Arc<dyn EmbeddingProvider>, ContextraError>` | Instantiates the configured `EmbeddingProvider` as an `Arc<dyn EmbeddingProvider>` and runs startup probe. | `src/config.rs` |
| `pub fn create_embedding_provider( provider_type: Option<&str>, ollama_url: &str, embed_model: &str, onnx_model_path: Option<&Path>, candle_model_dir: Option<&Path>, ) -> Result<Arc<dyn EmbeddingProvider>, ContextraError>` | Dynamically constructs an `EmbeddingProvider` implementation based on provider identifier. | `src/config.rs` |
| `pub fn create_llm_text_generator( provider_type: Option<&str>, ollama_url: &str, llm_model: &str, candle_model_dir: Option<&Path>, ) -> Result<Arc<dyn LlmTextGenerator>, ContextraError>` | Dynamically constructs an `LlmTextGenerator` implementation based on provider identifier. | `src/config.rs` |
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
| `pub fn code(&self) -> i32` | — | `src/protocol.rs` |
| `pub fn err(id: Option<Value>, code: i32, message: impl Into<String>) -> Self` | — | `src/protocol.rs` |
| `pub fn internal_error(msg: impl Into<String>) -> Self` | — | `src/protocol.rs` |
| `pub fn invalid_params(msg: impl Into<String>) -> Self` | — | `src/protocol.rs` |
| `pub fn invalid_request(msg: impl Into<String>) -> Self` | — | `src/protocol.rs` |
| `pub fn measure_json_depth(val: &Value) -> usize` | Ermittelt die Schachtelungstiefe eines JSON-Value. | `src/protocol.rs` |
| `pub fn method_not_found(msg: impl Into<String>) -> Self` | — | `src/protocol.rs` |
| `pub fn ok(id: Option<Value>, result: Value) -> Self` | — | `src/protocol.rs` |
| `pub fn parse_error(msg: impl Into<String>) -> Self` | — | `src/protocol.rs` |
| `pub fn response_from_error(id: Option<Value>, err: McpError) -> JsonRpcResponse` | Helper function to convert an `McpError` directly into a `JsonRpcResponse`. | `src/protocol.rs` |
| `pub fn validate_json_depth(val: &Value, max_depth: usize) -> Result<(), McpError>` | Validiert, dass ein JSON-Value die maximale Tiefe nicht überschreitet. | `src/protocol.rs` |
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
| `pub fn with_injection_guard(mut self, injection_guard: Arc<PromptInjectionGuard>) -> Self` | — | `src/server.rs` |
| `pub fn with_plugin_registry( mut self, registry: Option<Arc<contextra_ports::plugin::PluginRegistry>>, ) -> Self` | — | `src/server.rs` |
| `pub fn with_sandbox( db: Arc<Contextra>, embedder: Arc<dyn EmbeddingProvider>, sandbox: Arc<McpSandbox>, ) -> Self` | — | `src/server.rs` |
| `pub fn with_tenant_id(mut self, tenant_id: Option<TenantId>) -> Self` | — | `src/server.rs` |
| `pub fn with_write_permission( db: Arc<Contextra>, embedder: Arc<dyn EmbeddingProvider>, allow_db_writes: bool, ) -> Result<Self, ContextraError>` | — | `src/server.rs` |
| `pub async fn handle(&self, req: JsonRpcRequest) -> JsonRpcResponse` | — | `src/server_dispatch.rs` |
| `pub async fn handle_value(&self, val: Value) -> Option<Value>` | Verarbeitet einen Parsed JSON-RPC 2.0 Request (Einzelanfrage, Notification oder Batch Array). Gibt `Some(Value)` zurück, falls eine Antwort… | `src/server_dispatch.rs` |
| `pub fn is_write_allowed_by_env() -> bool` | Checks whether database write access is explicitly enabled via environment variable `CONTEXTRA_MCP_ALLOW_WRITE`. | `src/validation.rs` |
| `pub fn validate_collection_name(name: &str) -> Result<(), McpError>` | MCP-Server mit stdio-Transport (JSON-RPC 2.0). stdout ist dem Protokoll vorbehalten — Logs gehen ausschließlich nach stderr. | `src/validation.rs` |


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
pub mod doubles { .. }
pub mod fault { .. }
pub mod model { .. }
pub use crate::in_memory_graph::InMemoryGraphIndex;
pub use crate::in_memory_store::InMemoryStorageEngine;
pub use crate::in_memory_text::InMemoryTextIndex;
pub use crate::in_memory_vector::InMemoryVectorIndex;
pub use crate::manual_clock::ManualClock;
pub use crate::mocks::*;
pub use crate::fault_vfs::{FaultConfig, FaultVfs};
pub use crate::reference_model::{RefOp, ReferenceModel};
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
| `default` | `["testkit", "erasure-conformance"]` |
| `testkit` | `[]` |
| `erasure-conformance` | `[]` |

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

### Öffentliche Funktionen (48)

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
| `pub fn run_vfs_suite<V: Vfs>(vfs: &V) -> ConformanceResult` | Führt die umfassende Konformitätsprüfsuite gegen eine gegebene [`Vfs`]-Instanz aus. **Geprüfte Verträge:** 1. `non_existent_read_fails`: Ni… | `src/conformance/vfs.rs` |
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
* **`unsafe_code = "deny"`** ist Workspace-Default; nur `contextra-sys`, `contextra-simd` heben dies explizit auf.
* **Sync-Kern, Async-Schale (P26):** Ring-0-Crates verwenden kein `tokio`.
* **DAG-Integrität (P5):** Aufwärtskanten sind verboten.
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
