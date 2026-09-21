# Audit Report: `hybrid_search_*`-Konsolidierung zu `search(SearchRequest)` & Facade-Entwurf

* **Datum:** 2026-09-20
* **Autor:** Principal Rust Systems Engineer (Jules)
* **Status:** AUDIT (Read-Only Analyse & Architektur-Entwurf)
* **Ziel-Crates:** `crates/memfuse-engine`, `crates/memfuse`

---

## 1. Blocker-Check & Ist-Zustand Verification

### 1.1 Blocker-Check (`crates/memfuse-engine/src/collection/search.rs`)
Die Verifizierung der sechs legacy `hybrid_search_*`-Funktionssignaturen in `crates/memfuse-engine/src/collection/search.rs` wurde am aktuellen Stand des Repositories durchgeführt:

| Methode | Exakte Zeile | Signatur / Merkmale |
| :--- | :---: | :--- |
| `hybrid_search` | **Zeile 591** | `pub async fn hybrid_search(&self, text: &str, vector: &[f32], k: usize, anchor_entities: Option<&[memfuse_core::EntityId]>) -> Result<Vec<crate::SearchResult>>` |
| `hybrid_search_reranked` | **Zeile 612** | `pub async fn hybrid_search_reranked(&self, text: &str, vector: &[f32], k: usize, reranker: Option<&memfuse_embed::CrossEncoderReranker>, anchor_entities: Option<&[memfuse_core::EntityId]>) -> Result<Vec<crate::SearchResult>>` |
| `hybrid_search_with_weights` | **Zeile 635** | `pub async fn hybrid_search_with_weights(&self, text: &str, vector: &[f32], k: usize, anchor_entities: Option<&[memfuse_core::EntityId]>, weights: Option<&memfuse_core::FusionWeights>) -> Result<Vec<crate::SearchResult>>` |
| `hybrid_search_with_strategy` | **Zeile 652** | `pub async fn hybrid_search_with_strategy(&self, text: &str, vector: &[f32], k: usize, anchor_entities: Option<&[memfuse_core::EntityId]>, weights: Option<&memfuse_core::FusionWeights>, strategy: Option<&memfuse_core::GraphTraversalStrategy>, same_community_as: Option<EntityId>) -> Result<Vec<crate::SearchResult>>` |
| `hybrid_search_with_query` | **Zeile 825** | `pub async fn hybrid_search_with_query(&self, query: &memfuse_core::HybridQuery) -> Result<Vec<crate::SearchResult>>` |
| `hybrid_search_with_query_at` | **Zeile 839** | `pub async fn hybrid_search_with_query_at(&self, query: &memfuse_core::HybridQuery, seq: u64) -> Result<Vec<crate::SearchResult>>` |

### 1.2 Status der Facade (`crates/memfuse/src/lib.rs`)
* `crates/memfuse/src/lib.rs` ist derzeit ein reines Builder-Skelett (`MemFuseBuilder`) und reexportiert Core/DB/Router-Typen.
* Aktuell besitzt `crates/memfuse/src/lib.rs` genau **2** `pub fn`-Methoden:
  1. `pub fn new(dimension: usize) -> Self` (Zeile 31)
  2. `pub fn with_storage_path(...) -> Self` (Zeile 41)

### 1.3 Audit der `pub fn`-Anzahl in `crates/memfuse-db`
In `crates/memfuse-db` wurden per Code-Analyse (**keine ungeprüften README-Schätzungen**) alle Öffentlichen Funktionen ermittelt:
* Command: `grep -rn "pub fn" crates/memfuse-db/src/ | wc -l`
* Ergebnis: **38 `pub fn`** (bzw. **39 `pub (async) fn`** inklusive `pub async fn`).
* Dies bestätigt den Refactoring-Bedarf, eine auf **$\le 20$ `pub fn`** reduzierte High-Level-Facade in `crates/memfuse` bereitzustellen.

---

## 2. Analyse & Vergleichs-Matrix der 6 `hybrid_search_*`-Varianten

### 2.1 Unterscheidungsmerkmale der 6 Methoden

| Methode | Eingabe-Format | Signal-Gewichte | Graph-Strategie / Anchors | Reranking | MVCC Snapshot (`seq`) | Pre-RRF Filter / Community |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `hybrid_search` | `(text, vector, k)` | Default (Equal / `FusionWeights::default()`) | `anchor_entities: Option<&[EntityId]>` | Nein | Automatisch latest | Nein |
| `hybrid_search_reranked` | `(text, vector, k)` | Default | `anchor_entities: Option<&[EntityId]>` | `reranker: Option<&CrossEncoderReranker>` | Automatisch latest | Nein |
| `hybrid_search_with_weights` | `(text, vector, k)` | Custom `FusionWeights` | `anchor_entities: Option<&[EntityId]>` | Nein | Automatisch latest | Nein |
| `hybrid_search_with_strategy` | `(text, vector, k)` | Custom `FusionWeights` | `GraphTraversalStrategy` + `anchor_entities` | Nein | Automatisch latest | `same_community_as: Option<EntityId>` |
| `hybrid_search_with_query` | `HybridQuery` Struct | In `HybridQuery` (`fusion_weights`) | In `HybridQuery` (`graph_strategy`, `graph_start_node`) | Via `has_reranker` Flag | Automatisch latest | In `HybridQuery` (`filter`, `memory_type_filter`, `same_community_as`) |
| `hybrid_search_with_query_at` | `HybridQuery` Struct | In `HybridQuery` | In `HybridQuery` | Via `has_reranker` Flag | **Expliziter `seq: u64` Parameter** | In `HybridQuery` |

### 2.2 Kern-Befund der Analyse
1. **Redundanz:** Die Methoden `hybrid_search`, `hybrid_search_reranked` und `hybrid_search_with_weights` sind lediglich dünne Adapter/Wrapper um `hybrid_search_with_strategy`.
2. **`HybridQuery` als unvollständiges Modell:** `hybrid_search_with_strategy` unterstützt Parameter wie `anchor_entities` als Slice, während `HybridQuery` `graph_start_node` als `Option<String>` führt.
3. **Snapshot Isolation:** Einzig `hybrid_search_with_query_at` nimmt einen zeitstempelisolierten Sequence Number `seq` entgegen (`with_pinned_checkpoint_at_latest`).

---

## 3. Entwurf von `SearchRequest` (Rust Code-Skizze)

Um alle 6 Varianten verlustfrei, typsicher und ohne `panic!` / `unwrap` abzubilden, wird die Konsolidierung zu `SearchRequest` vorgeschlagen.

### 3.1 Code-Skizze: `SearchRequest` & `SearchRequestBuilder`

```rust
use std::sync::Arc;
use memfuse_core::{
    EntityId, FilterExpr, FusionStrategy, FusionWeights, GraphTraversalStrategy,
    MemoryType, MemFuseError, Result,
};

/// Unified search request covering vector, text, graph, filtering, and reranking parameters.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SearchRequest {
    /// Full-text query string (BM25 signal).
    pub text: Option<String>,
    /// Dense vector embedding query (HNSW signal).
    pub vector: Option<Vec<f32>>,
    /// Desired number of top results to return.
    pub k: usize,
    /// Explicit graph anchor entities for relational traversal.
    pub anchor_entities: Option<Vec<EntityId>>,
    /// Custom signal fusion weights (vector, text, graph).
    pub weights: Option<FusionWeights>,
    /// Fusion strategy (RRF, Linear, etc.).
    pub fusion_strategy: FusionStrategy,
    /// Graph traversal strategy (Hops, PPR, PathRag).
    pub graph_strategy: GraphTraversalStrategy,
    /// Optional entity ID to filter/boost results within the same community.
    pub same_community_as: Option<EntityId>,
    /// Pre-RRF metadata expression filter.
    pub filter: Option<FilterExpr>,
    /// Memory type pre-filter.
    pub memory_type_filter: Option<Vec<MemoryType>>,
    /// Explicit snapshot sequence number for MVCC point-in-time isolation.
    pub snapshot_seq: Option<u64>,
    /// Cross-encoder reranking configuration option.
    pub enable_reranking: bool,
    /// Include superseded memory chunks (ADR-038).
    pub include_superseded: bool,
    /// Attach provenance records to results.
    pub include_provenance: bool,
}

impl SearchRequest {
    /// Creates a builder for `SearchRequest`.
    pub fn builder() -> SearchRequestBuilder {
        SearchRequestBuilder::default()
    }

    /// Validates the request invariants under the Zero-Panic Doctrine.
    pub fn validate(&self, expected_dim: Option<usize>) -> Result<()> {
        if self.k == 0 {
            return Err(MemFuseError::invalid_input("k must be greater than 0"));
        }
        if let Some(vec) = &self.vector {
            if let Some(dim) = expected_dim {
                if !vec.is_empty() && vec.len() != dim {
                    return Err(MemFuseError::invalid_input(format!(
                        "Vector dimension mismatch: expected {}, got {}",
                        dim,
                        vec.len()
                    )));
                }
            }
            for (idx, &val) in vec.iter().enumerate() {
                if !val.is_finite() {
                    return Err(MemFuseError::invalid_input(format!(
                        "Non-finite float in vector query at index {idx}: {val}"
                    )));
                }
            }
        }
        if self.text.as_ref().map_or(true, |t| t.trim().is_empty())
            && self.vector.as_ref().map_or(true, |v| v.is_empty())
            && self.anchor_entities.as_ref().map_or(true, |a| a.is_empty())
        {
            return Err(MemFuseError::invalid_input(
                "SearchRequest must specify at least one search signal (text, vector, or anchors)",
            ));
        }
        Ok(())
    }
}
```

### 3.2 Invarianten & Zero-Panic-Doctrine Compliance
* Ungültige Dimensionen oder NaN/Infinity-Werte im Vektor lösen ein `Err(MemFuseError::InvalidInput(...))` aus.
* Es gibt **keine** `unwrap()`, `panic!()` oder `expect()` Aufrufe.
* Ungültige Parameter-Kombinationen werden bereits in `.validate()` abgefangen.

---

## 4. workspace-weite Aufrufer-Analyse & Abwertungsstrategie

Ein `grep -rn "\.hybrid_search"` ergab Aufrufe in folgenden Komponenten:

1. **`memfuse-py` (Python Bindings):**
   - `crates/memfuse-py/src/lib.rs` ruft `hybrid_search_with_weights` auf.
   - `crates/memfuse-py/tests/` ruft `.hybrid_search(...)` und `.hybrid_search_fb(...)` auf.
2. **Engine / DB Tests:**
   - `crates/memfuse-engine/src/collection/tests.rs` (30+ Aufrufe).
   - `crates/memfuse-db/tests/` (z. B. `graph_hybrid_search.rs`, `fault_injection_2pc.rs`, `compound_split_recall_impact_test.rs`).
3. **Benchmarks:**
   - `benches/competitive_bench.rs`, `benches/scale_bench.rs`, `benches/migration_benchmarks.rs`.
4. **Examples:**
   - `examples/hybrid_search.rs`, `crates/memfuse-db/examples/hybrid_search.rs`.

### 4.1 Abwertungs- und Übergangsstrategie
* **Phase 1 (Welle 5 IMPL):** Einführen von `Collection::search(&self, request: SearchRequest) -> Result<Vec<SearchResult>>` (bzw. Weiterführung von `Collection::query()`).
* **Phase 2 (Deprecation Wrappers):** Markieren aller 6 legacy `hybrid_search_*`-Methoden mit `#[deprecated(since = "0.2.0", note = "use Collection::search(req) or Collection::query() instead")]`.
* **Phase 3 (Internes Refactoring):** Umstellung der internen Aufrufe in `memfuse-py`, `memfuse-db/tests`, `benches` und `examples` auf die neue API.

---

## 5. DAG-Topologie & Ring-Modell Platzierung

### 5.1 Zuordnung von `SearchRequest` im Ring-Modell
Gemäß `WORKING_STATE.md` und der DAG-Topologie:
* `memfuse-types` befindet sich in **Layer 0 / Ring 0**.
* `memfuse-core` befindet sich in **Layer 2 / Ring 1**.
* `memfuse-engine` befindet sich in **Layer 5 / Ring 3**.
* `memfuse-db` befindet sich in **Layer 7 / Ring 3**.
* `memfuse-router` befindet sich in **Layer 8 / Ring 3** (hängt von `memfuse-core` und `memfuse-db` ab).
* `memfuse` Facade befindet sich in **Layer 7/8 / Ring 3**.

**Begründung für die Platzierung:**
* `SearchRequest` muss in `memfuse-types` oder `memfuse-core` definiert werden.
* Eine Platzierung in `memfuse-types` oder `memfuse-core` stellt sicher, dass tiefere Ringe (`memfuse-engine`) und parallele Module (`memfuse-router`, `memfuse-py`, `memfuse-mcp`) den Typ importieren können, **ohne** Aufwärtskanten oder zyklenbehaftete DAG-Verletzungen zu erzeugen.

---

## 6. Facade-Entwurf für `crates/memfuse/src/lib.rs` (≤ 20 `pub fn`)

Die Facade in `crates/memfuse` soll als schlanker High-Level Entry Point dienen, der Aufrufe direkt an `memfuse-db` / `memfuse-engine` delegiert.

### 6.1 Priorisierte Liste der 15 Kern-Methoden für die Facade

Aus den **38 `pub fn`** in `memfuse-db` werden folgende 15 Methoden für die `memfuse` Facade ausgewählt, um das Ziel **$\le 20$ `pub fn`** garantiert einzuhalten:

1. **Lifecycle & Management (4):**
   - `open(path, config) -> Result<MemFuse>`
   - `close(&self) -> Result<()>`
   - `stats(&self) -> Result<MemFuseStats>`
   - `flush(&self) -> Result<()>`
2. **Document Ingestion & CRUD (4):**
   - `insert(&self, doc: Document) -> Result<DocId>`
   - `get(&self, id: DocId) -> Result<Option<Document>>`
   - `delete(&self, id: DocId) -> Result<bool>`
   - `update(&self, doc: Document) -> Result<()>`
3. **Consolidated Search Entrypoint (2):**
   - `search(&self, request: SearchRequest) -> Result<Vec<SearchResult>>`
   - `query(&self) -> QueryBuilder`
4. **Graph & Relations (2):**
   - `relate(&self, source: EntityId, target: EntityId, relation: &str) -> Result<()>`
   - `traverse(&self, anchor: EntityId, max_hops: usize) -> Result<Vec<EntityResult>>`
5. **Consolidation & Maintenance (3):**
   - `consolidate(&self) -> Result<ConsolidationStats>`
   - `create_checkpoint(&self) -> Result<CheckpointId>`
   - `repair(&self) -> Result<RepairReport>`

### 6.2 Facade Code-Skizze (`crates/memfuse/src/lib.rs`)

```rust
//! MemFuse — High-Level Embedded Hybrid-Search Facade

pub use memfuse_core::{DocId, MemFuseError, Result, SearchResult};
pub use memfuse_types::SearchRequest;

pub struct MemFuse {
    inner: memfuse_db::MemFuse,
}

impl MemFuse {
    /// Opens a MemFuse instance at the given path with configuration.
    pub async fn open(path: impl Into<std::path::PathBuf>, config: memfuse_db::MemFuseConfig) -> Result<Self> {
        let inner = memfuse_db::MemFuse::open_with_config(path, config).await?;
        Ok(Self { inner })
    }

    /// Unified search entrypoint accepting a `SearchRequest`.
    pub async fn search(&self, request: SearchRequest) -> Result<Vec<SearchResult>> {
        self.inner.search(request).await
    }

    /// Flushes pending writes to disk.
    pub async fn flush(&self) -> Result<()> {
        self.inner.flush().await
    }

    /// Closes the database instance gracefully.
    pub async fn close(self) -> Result<()> {
        self.inner.close().await
    }
}
```

---

## 7. Umsetzungsplan für IMPL-Welle (Welle 5)

### 7.1 Schritt-für-Schritt Umsetzungsreihenfolge
1. **Typ-Definition (Phase 1):**
   - Hinzufügen von `SearchRequest` & `SearchRequestBuilder` in `crates/memfuse-types/src/types/saos.rs` / `memfuse-core`.
   - Implementierung der Validation `.validate(dimension)`.
2. **Engine Integration (Phase 2):**
   - Implementieren von `Collection::search(&self, request: SearchRequest) -> Result<Vec<SearchResult>>` in `crates/memfuse-engine/src/collection/search.rs`.
   - Umleiten der bestehenden `hybrid_search_*`-Signaturen als `#[deprecated]` Wrapper auf `Collection::search`.
3. **Facade Delegation (Phase 3):**
   - Anbinden von `MemFuse::search` in `crates/memfuse/src/lib.rs`.
4. **Verifikation & Test-Migration (Phase 4):**
   - Anpassen der Benchmarks und Tests in `memfuse-db` und `memfuse-py`.
   - Ausführen aller Quality Gates (`just check`, `cargo xtask jules-preflight`).

### 7.2 Warnhinweis & Kollisions-Schutz hinsichtlich `memfuse-rank`
* **Parallelitäts-Warnung:** Falls Welle 5 parallel zu einer eventuellen Neuanlage oder Umstrukturierung von `memfuse-rank` ausgeführt wird, muss sichergestellt werden, dass nicht gleichzeitig dieselben Zeilen in `crates/memfuse-engine/src/collection/search.rs` (Zeilen 580–850) bearbeitet werden.
* **Prüfung:** `memfuse-rank` greift primär in Signal-Fusions-Funktionen (`fusion.rs`) und Reranking-Kernel ein. Dennoch muss vor Beginn von Wave 5 IMPL per `cargo xtask claim` geprüft werden, ob Ansprüche auf `memfuse-engine` oder `memfuse-rank` bestehen.

---

## 8. Fazit & Audit-Bestätigung

* **Scope-Einhaltung:** Es wurden **keine** Quellcodedateien oder `Cargo.toml` modifiziert.
* **Belegbarkeit:** Alle Funktionszahlen (38 `pub fn` in `memfuse-db`, 2 `pub fn` in `memfuse`) und Zeilennummern wurden live aus dem Workspace erhoben.
* **Vollständigkeit:** Der Report liefert eine vollständige Basis für die Welle 5 IMPL-Umsetzung.
