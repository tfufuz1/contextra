# Audit & Refactoring-Entwurf: Router→DB Aufwärtskante & Port-Schnittstellen (Welle 1)

**Datum:** 2026-09-19
**Autor:** Google-Jules (Principal Rust Systems Engineer)
**Status:** AUDIT (Analyse & Port-Entwurf)
**Ziel-Crates:** `crates/memfuse-router` (Read-Only Target), `crates/memfuse-db` (Read-Only Target), `crates/memfuse-ports` (Port Spec Target)
**Scope:** `docs/refactor/router-db-edge-audit.md`

---

## 1. Executive Summary & Ring-Modell Verletzungsanalyse

Gemäß MemFuse Architektur-Spezifikation (README.md §A2, §4, §20) ist das Repository in ein striktes Ring-Modell (0–4) gegliedert:
* **Ring 0 (Foundation):** `memfuse-types`, `memfuse-ports`, `memfuse-wire`
* **Ring 1 (Storage & Index):** `memfuse-core`, `memfuse-store`, `memfuse-index`, `memfuse-graph`, `memfuse-text`
* **Ring 2 (Core OS & DB):** `memfuse-db`, `memfuse-checkpoint`, `memfuse-mvcc`
* **Ring 3 (Cognitive OS & Router):** `memfuse-router`, `memfuse-adapt`, `memfuse-calibration`, `memfuse-embed`
* **Ring 4 (Egress & Interfaces):** `memfuse-mcp`, `memfuse-agent`, `memfuse-py`

### Das Problem (Aufwärtskante Ring 3 → Ring 2)
Aktuell importiert `memfuse-router` (Ring 3) direkt den Beton-Typ `Collection<S>` sowie `ContextManager` aus `memfuse-db` (Ring 2). Darüber hinaus implemeniert `RouterEngine<S>` den Trait `memfuse_db::DriftStatusProvider` direkt über die `memfuse_db`-Re-Export-Schiene.

Dies verletzt die verbotene Aufwärtskanten-Regel ("Ring 3 darf nicht von Ring 2 abhängen"). Dadurch entsteht eine unzulässige Kopplung, welche modulare Tests erschwert und die Unabhängigkeit des Routers von der spezifischen DB-Implementierung bricht.

---

## 2. Ist-Zustand Inventar: Alle `memfuse_db`-Verwendungen im Router

Ein vollständiger Scan über `crates/memfuse-router` liefert folgende konkrete Fundstellen:

| Datei | Zeile | Symbol / Import | Zweck & Kontext |
| :--- | :--- | :--- | :--- |
| `crates/memfuse-router/Cargo.toml` | 21 | `memfuse-db = { workspace = true }` | Crate-Abhängigkeit im Manifest |
| `crates/memfuse-router/AGENTS.md` | 93 | `- Erlaubte Imports: memfuse-core (L0), memfuse-db (L2)` | Veraltete Dokumentations-Spezifikation |
| `crates/memfuse-router/src/router.rs` | 15 | `use memfuse_db::{collection::Collection, context::ContextManager};` | Import von Betontypen für Search & Context Preparation |
| `crates/memfuse-router/src/router.rs` | 900 | `impl<S: StorageEngine> memfuse_db::DriftStatusProvider for RouterEngine<S>` | Implementierung des DriftStatusProvider Traits über DB-Reexport |
| `crates/memfuse-router/src/tests.rs` | 16 | `use memfuse_db::{MemFuse, MemFuseConfig};` | Test-Setup für Integrations-Tests |
| `crates/memfuse-router/src/tests.rs` | 127 | `let collection = Arc::new(memfuse_db::Collection::new(...));` | Test-Collection Erstellung |
| `crates/memfuse-router/src/tests.rs` | 3218, 3222 | `let config = memfuse_db::MemFuseConfig { ... }; let db = memfuse_db::MemFuse::open_with_config(...);` | Test-DB Initialisierung |
| `crates/memfuse-router/benches/router_bench.rs` | 3 | `use memfuse_db::{MemFuse, MemFuseConfig};` | Benchmark Setup |

### Funktions- & Methodenaufruf-Analyse in `router.rs`:
In `RouterEngine<S>` werden konkret folgende Methoden von `memfuse_db::Collection` und `ContextManager` aufgerufen:
1. `self.collection.query().text(query_text).embedding(query_embedding).k(10).execute().await?`
   * *Verwendung:* Hybrid-Suche zur Ermittlung top-k relevanter Chunks.
2. `self.collection.get_community(eid).await.ok().flatten()`
   * *Verwendung:* Lookup der Graph-Community-ID für ein `EntityId`.
3. `let mut context_mgr = ContextManager::new(selected_profile.token_budget.clone());`
   * *Verwendung:* Zusammenstellung und Beschneidung des Kontextfensters für ein bestimmtes Token-Budget.

---

## 3. Soll-Zustand: Port-Schnittstellen Entwurf (`memfuse-ports`)

Um `memfuse-router` vollständig von `memfuse-db` zu entkoppeln, werden die Betontypen durch abstrahierte Port-Traits (`Arc<dyn Trait>`) aus `memfuse-ports` (Ring 0) bzw. `memfuse-core::traits` ersetzt.

### 3.1 Trait 1: `CommunityResolver`
Dient dem Abfragen der Community-Zuordnung von Entity-IDs.

```rust
// In crates/memfuse-ports/src/graph_index.rs (oder vector_index.rs)
use memfuse_types::{EntityId, Result};
use memfuse_core::BoxFuture;

/// Contract for resolving graph community assignments for entities.
pub trait CommunityResolver: Send + Sync {
    /// Resolves the optional community ID for a given entity.
    fn get_community<'a>(&'a self, entity_id: EntityId) -> BoxFuture<'a, Result<Option<u64>>>;
}
```

### 3.2 Trait 2: `HybridSearchProvider`
Abstrahiert die Hybrid-Suche zur Auswahl passender Chunks.

```rust
// In crates/memfuse-ports/src/vector_index.rs
use memfuse_types::{ContextChunk, Result};
use memfuse_core::BoxFuture;

/// Contract for executing hybrid (vector + text) queries for profile routing.
pub trait HybridSearchProvider: Send + Sync {
    /// Executes a hybrid query returning matched context chunks with relevance scores.
    fn search_hybrid<'a>(
        &'a self,
        query_text: &'a str,
        query_embedding: &'a [f32],
        top_k: usize,
    ) -> BoxFuture<'a, Result<Vec<ContextChunk>>>;
}
```

### 3.3 Trait 3: `ContextPreparer`
Abstrahiert das Token-Budget-basierte Context-Fitting.

```rust
// In crates/memfuse-ports/src/lifecycle.rs
use memfuse_types::{ContextChunk, ContextWindow, TokenBudget, Result};

/// Contract for trimming and preparing context windows tailored to token budgets.
pub trait ContextPreparer: Send + Sync {
    /// Prepares and trims context chunks according to the provided token budget and relevance threshold.
    fn prepare_context(
        &self,
        chunks: Vec<ContextChunk>,
        budget: &TokenBudget,
        relevance_threshold: f32,
    ) -> Result<ContextWindow>;
}
```

### 3.4 Umstellung `DriftStatusProvider`
Der Trait `DriftStatusProvider` existiert **bereits** in `memfuse-ports::observability` (`crates/memfuse-ports/src/observability.rs`):
```rust
pub trait DriftStatusProvider: Send + Sync {
    fn overall_drift_status(&self) -> String;
}
```
In `router.rs` wird die Implementierung in Welle 2 einfach von `impl memfuse_db::DriftStatusProvider` auf `impl memfuse_ports::DriftStatusProvider` umgestellt.

---

## 4. Welle 2 Umsetzungsplan (Strangler-Muster & Dateiliste)

Die Migration erfolgt stufenweise ohne Binnendifferenzierung und ohne API-Brechung (Strangler-Prinzip). Legacy-Reexporte in `memfuse-db` stellen Abwärtskompatibilität sicher.

### 4.1 Datei-Änderungsliste für Welle 2

1. **`crates/memfuse-ports/src/` (Ring 0)**
   * `graph_index.rs`: Hinzufügen des Traits `CommunityResolver`.
   * `vector_index.rs`: Hinzufügen des Traits `HybridSearchProvider`.
   * `lifecycle.rs`: Hinzufügen des Traits `ContextPreparer`.
   * `lib.rs`: Re-Export aller neuen Traits.

2. **`crates/memfuse-db/src/` (Ring 2)**
   * `collection.rs` / `lib.rs`: Implementieren von `CommunityResolver` und `HybridSearchProvider` für `Collection<S>`.
   * `context.rs`: Implementieren von `ContextPreparer` für `ContextManager`.
   * `lib.rs`: Ergänzen von Re-Exporten (`pub use memfuse_ports::{CommunityResolver, HybridSearchProvider, ContextPreparer, DriftStatusProvider};`).

3. **`crates/memfuse-router/Cargo.toml` & `AGENTS.md` (Ring 3)**
   * `Cargo.toml`: Entfernen von `memfuse-db` aus `[dependencies]`. (Verschieben nach `[dev-dependencies]` falls für Integrationstests zwingend benötigt, oder Test-Mocks via `memfuse-ports` nutzen).
   * `Cargo.toml`: Sicherstellen, dass `memfuse-ports` als Dependency eingetragen ist.
   * `AGENTS.md`: Aktualisieren der erlaubten Imports (Ersetze `memfuse-db` durch `memfuse-ports`).

4. **`crates/memfuse-router/src/` (Ring 3)**
   * `router.rs`:
     * Ersetze `collection: Arc<Collection<S>>` in `RouterEngine` durch ein entkoppeltes Struct/Field-Set, z.B.:
       ```rust
       pub struct RouterEngine {
           search_provider: Arc<dyn HybridSearchProvider>,
           community_resolver: Arc<dyn CommunityResolver>,
           context_preparer: Arc<dyn ContextPreparer>,
           ...
       }
       ```
     * Ersetze `impl<S: StorageEngine> memfuse_db::DriftStatusProvider` durch `impl memfuse_ports::DriftStatusProvider for RouterEngine`.
   * `tests.rs`: Umstellen auf Mocks or Test-Adapter basierend auf `memfuse-ports`.

---

## 5. Zusatz-Audit: `memfuse-core-ipc-gen` vs. `memfuse-wire`

Es wurde eine vollständige Durchsuchung aller Crates, Tooling-Skripte und Dokumentationen bezüglich des veralteten Crate-Namens `memfuse-core-ipc-gen` durchgeführt:

### 5.1 Ist-Zustand im Quellcode
* **Code-Imports (Crates):** Alle Rust-Quellcodes in `crates/` wurden bereits auf `memfuse-wire` migriert. In `crates/memfuse-core/src/ipc/mod.rs` gilt:
  `pub use memfuse_wire::*;`
* Es existiert **keine** aktiver `use memfuse_core_ipc_gen`-Import mehr in Rust-Produktionsdateien.

### 5.2 Verbleibende Referenzen im Tooling (`xtask`)
1. `xtask/src/check_layering.rs` (Zeile 37):
   ```rust
   "memfuse-core" | "memfuse-core-ipc-gen" => Some(Ring::Ring0),
   ```
   * *Aktion für Welle 2:* Ergänze `"memfuse-wire"` bzw. ersetze `"memfuse-core-ipc-gen"` vollständig durch `"memfuse-wire"`.
2. `xtask/src/check_flatbuffers_drift.rs` (Zeile 152):
   ```rust
   /// Regenerates `crates/memfuse-core-ipc-gen/src/memfuse_generated.rs` directly from `schemas/memfuse.fbs`.
   ```
   * *Aktion für Welle 2:* Doku-Kommentar auf `crates/memfuse-wire/src/memfuse_generated.rs` anpassen.

### 5.3 Verbleibende Referenzen in Dokumenten
In `README.md`, `docs/ARCHITECTURE.md` und `docs/audits/` existieren noch historische Bezeichnungen (`memfuse-core-ipc-gen`). Gemäß Entwurfsentscheidungen wurde `memfuse-core-ipc-gen` formal durch `memfuse-wire` abgelöst.

---

## 6. Verification & Strict Invariants Assessment

| Invariante / Policy | Bewertung & Einhaltung im Port-Entwurf |
| :--- | :--- |
| **Zero-Panic-Doctrine** | Alle neuen Trait-Methoden liefern `Result<T, MemFuseError>` und propagieren Fehler via `?`. KEIN `.unwrap()`/`.expect()`. |
| **Zero-Copy & Alignment** | Dynamic Trait-Dispatch nutzt `Arc<dyn Trait>` oder Slices `&[f32]`, pass-by-reference ohne Klonen großer Payloads. |
| **DAG-Layering Integrität** | Nach Umsetzung in Welle 2 zeigt `just dag-check` 0 Verstöße; `memfuse-router` (Ring 3) hängt nur noch von `memfuse-ports` (Ring 0) und `memfuse-core` (Ring 1) ab. |
| **Unsafe-Inseln** | Weder `memfuse-router` noch `memfuse-ports` enthalten `unsafe` Code (`#![forbid(unsafe_code)]`). |
| **Strangler-Prinzip / Backward-Compatibility** | Re-Export des `DriftStatusProvider` in `memfuse-db` bleibt erhalten. Public API von Router und DB bleibt unterbrechungsfrei kompatibel. |

---
*Ende des Audit-Berichts `docs/refactor/router-db-edge-audit.md`*
