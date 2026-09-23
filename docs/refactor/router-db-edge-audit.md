# Audit & Refactoring-Entwurf: Router→DB Aufwärtskante & Port-Schnittstellen (Welle 1)

**Datum:** 2026-09-19
**Autor:** Google-Jules (Principal Rust Systems Engineer)
**Status:** AUDIT (Analyse & Port-Entwurf)
**Ziel-Crates:** `crates/contextra-router` (Read-Only Target), `crates/contextra-db` (Read-Only Target), `crates/contextra-ports` (Port Spec Target)
**Scope:** `docs/refactor/router-db-edge-audit.md`

---

## 1. Executive Summary & Ring-Modell Verletzungsanalyse

Gemäß Contextra Architektur-Spezifikation (README.md §A2, §4, §20) ist das Repository in ein striktes Ring-Modell (0–4) gegliedert:
* **Ring 0 (Foundation):** `contextra-types`, `contextra-ports`, `contextra-wire`
* **Ring 1 (Storage & Index):** `contextra-core`, `contextra-store`, `contextra-index`, `contextra-graph`, `contextra-text`
* **Ring 2 (Core OS & DB):** `contextra-db`, `contextra-checkpoint`, `contextra-mvcc`
* **Ring 3 (Cognitive OS & Router):** `contextra-router`, `contextra-adapt`, `contextra-calibration`, `contextra-embed`
* **Ring 4 (Egress & Interfaces):** `contextra-mcp`, `contextra-agent`, `contextra-py`

### Das Problem (Aufwärtskante Ring 3 → Ring 2)
Aktuell importiert `contextra-router` (Ring 3) direkt den Beton-Typ `Collection<S>` sowie `ContextManager` aus `contextra-db` (Ring 2). Darüber hinaus implemeniert `RouterEngine<S>` den Trait `contextra_db::DriftStatusProvider` direkt über die `contextra_db`-Re-Export-Schiene.

Dies verletzt die verbotene Aufwärtskanten-Regel ("Ring 3 darf nicht von Ring 2 abhängen"). Dadurch entsteht eine unzulässige Kopplung, welche modulare Tests erschwert und die Unabhängigkeit des Routers von der spezifischen DB-Implementierung bricht.

---

## 2. Ist-Zustand Inventar: Alle `contextra_db`-Verwendungen im Router

Ein vollständiger Scan über `crates/contextra-router` liefert folgende konkrete Fundstellen:

| Datei | Zeile | Symbol / Import | Zweck & Kontext |
| :--- | :--- | :--- | :--- |
| `crates/contextra-router/Cargo.toml` | 21 | `contextra-db = { workspace = true }` | Crate-Abhängigkeit im Manifest |
| `crates/contextra-router/AGENTS.md` | 93 | `- Erlaubte Imports: contextra-core (L0), contextra-db (L2)` | Veraltete Dokumentations-Spezifikation |
| `crates/contextra-router/src/router.rs` | 15 | `use contextra_db::{collection::Collection, context::ContextManager};` | Import von Betontypen für Search & Context Preparation |
| `crates/contextra-router/src/router.rs` | 900 | `impl<S: StorageEngine> contextra_db::DriftStatusProvider for RouterEngine<S>` | Implementierung des DriftStatusProvider Traits über DB-Reexport |
| `crates/contextra-router/src/tests.rs` | 16 | `use contextra_db::{Contextra, ContextraConfig};` | Test-Setup für Integrations-Tests |
| `crates/contextra-router/src/tests.rs` | 127 | `let collection = Arc::new(contextra_db::Collection::new(...));` | Test-Collection Erstellung |
| `crates/contextra-router/src/tests.rs` | 3218, 3222 | `let config = contextra_db::ContextraConfig { ... }; let db = contextra_db::Contextra::open_with_config(...);` | Test-DB Initialisierung |
| `crates/contextra-router/benches/router_bench.rs` | 3 | `use contextra_db::{Contextra, ContextraConfig};` | Benchmark Setup |

### Funktions- & Methodenaufruf-Analyse in `router.rs`:
In `RouterEngine<S>` werden konkret folgende Methoden von `contextra_db::Collection` und `ContextManager` aufgerufen:
1. `self.collection.query().text(query_text).embedding(query_embedding).k(10).execute().await?`
   * *Verwendung:* Hybrid-Suche zur Ermittlung top-k relevanter Chunks.
2. `self.collection.get_community(eid).await.ok().flatten()`
   * *Verwendung:* Lookup der Graph-Community-ID für ein `EntityId`.
3. `let mut context_mgr = ContextManager::new(selected_profile.token_budget.clone());`
   * *Verwendung:* Zusammenstellung und Beschneidung des Kontextfensters für ein bestimmtes Token-Budget.

---

## 3. Soll-Zustand: Port-Schnittstellen Entwurf (`contextra-ports`)

Um `contextra-router` vollständig von `contextra-db` zu entkoppeln, werden die Betontypen durch abstrahierte Port-Traits (`Arc<dyn Trait>`) aus `contextra-ports` (Ring 0) bzw. `contextra-core::traits` ersetzt.

### 3.1 Trait 1: `CommunityResolver`
Dient dem Abfragen der Community-Zuordnung von Entity-IDs.

```rust
// In crates/contextra-ports/src/graph_index.rs (oder vector_index.rs)
use contextra_types::{EntityId, Result};
use contextra_core::BoxFuture;

/// Contract for resolving graph community assignments for entities.
pub trait CommunityResolver: Send + Sync {
    /// Resolves the optional community ID for a given entity.
    fn get_community<'a>(&'a self, entity_id: EntityId) -> BoxFuture<'a, Result<Option<u64>>>;
}
```

### 3.2 Trait 2: `HybridSearchProvider`
Abstrahiert die Hybrid-Suche zur Auswahl passender Chunks.

```rust
// In crates/contextra-ports/src/vector_index.rs
use contextra_types::{ContextChunk, Result};
use contextra_core::BoxFuture;

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
// In crates/contextra-ports/src/lifecycle.rs
use contextra_types::{ContextChunk, ContextWindow, TokenBudget, Result};

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
Der Trait `DriftStatusProvider` existiert **bereits** in `contextra-ports::observability` (`crates/contextra-ports/src/observability.rs`):
```rust
pub trait DriftStatusProvider: Send + Sync {
    fn overall_drift_status(&self) -> String;
}
```
In `router.rs` wird die Implementierung in Welle 2 einfach von `impl contextra_db::DriftStatusProvider` auf `impl contextra_ports::DriftStatusProvider` umgestellt.

---

## 4. Welle 2 Umsetzungsplan (Strangler-Muster & Dateiliste)

Die Migration erfolgt stufenweise ohne Binnendifferenzierung und ohne API-Brechung (Strangler-Prinzip). Legacy-Reexporte in `contextra-db` stellen Abwärtskompatibilität sicher.

### 4.1 Datei-Änderungsliste für Welle 2

1. **`crates/contextra-ports/src/` (Ring 0)**
   * `graph_index.rs`: Hinzufügen des Traits `CommunityResolver`.
   * `vector_index.rs`: Hinzufügen des Traits `HybridSearchProvider`.
   * `lifecycle.rs`: Hinzufügen des Traits `ContextPreparer`.
   * `lib.rs`: Re-Export aller neuen Traits.

2. **`crates/contextra-db/src/` (Ring 2)**
   * `collection.rs` / `lib.rs`: Implementieren von `CommunityResolver` und `HybridSearchProvider` für `Collection<S>`.
   * `context.rs`: Implementieren von `ContextPreparer` für `ContextManager`.
   * `lib.rs`: Ergänzen von Re-Exporten (`pub use contextra_ports::{CommunityResolver, HybridSearchProvider, ContextPreparer, DriftStatusProvider};`).

3. **`crates/contextra-router/Cargo.toml` & `AGENTS.md` (Ring 3)**
   * `Cargo.toml`: Entfernen von `contextra-db` aus `[dependencies]`. (Verschieben nach `[dev-dependencies]` falls für Integrationstests zwingend benötigt, oder Test-Mocks via `contextra-ports` nutzen).
   * `Cargo.toml`: Sicherstellen, dass `contextra-ports` als Dependency eingetragen ist.
   * `AGENTS.md`: Aktualisieren der erlaubten Imports (Ersetze `contextra-db` durch `contextra-ports`).

4. **`crates/contextra-router/src/` (Ring 3)**
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
     * Ersetze `impl<S: StorageEngine> contextra_db::DriftStatusProvider` durch `impl contextra_ports::DriftStatusProvider for RouterEngine`.
   * `tests.rs`: Umstellen auf Mocks or Test-Adapter basierend auf `contextra-ports`.

---

## 5. Zusatz-Audit: `contextra-core-ipc-gen` vs. `contextra-wire`

Es wurde eine vollständige Durchsuchung aller Crates, Tooling-Skripte und Dokumentationen bezüglich des veralteten Crate-Namens `contextra-core-ipc-gen` durchgeführt:

### 5.1 Ist-Zustand im Quellcode
* **Code-Imports (Crates):** Alle Rust-Quellcodes in `crates/` wurden bereits auf `contextra-wire` migriert. In `crates/contextra-core/src/ipc/mod.rs` gilt:
  `pub use contextra_wire::*;`
* Es existiert **keine** aktiver `use contextra_core_ipc_gen`-Import mehr in Rust-Produktionsdateien.

### 5.2 Verbleibende Referenzen im Tooling (`xtask`)
1. `xtask/src/check_layering.rs` (Zeile 37):
   ```rust
   "contextra-core" | "contextra-core-ipc-gen" => Some(Ring::Ring0),
   ```
   * *Aktion für Welle 2:* Ergänze `"contextra-wire"` bzw. ersetze `"contextra-core-ipc-gen"` vollständig durch `"contextra-wire"`.
2. `xtask/src/check_flatbuffers_drift.rs` (Zeile 152):
   ```rust
   /// Regenerates `crates/contextra-core-ipc-gen/src/contextra_generated.rs` directly from `schemas/contextra.fbs`.
   ```
   * *Aktion für Welle 2:* Doku-Kommentar auf `crates/contextra-wire/src/contextra_generated.rs` anpassen.

### 5.3 Verbleibende Referenzen in Dokumenten
In `README.md`, `docs/ARCHITECTURE.md` und `docs/audits/` existieren noch historische Bezeichnungen (`contextra-core-ipc-gen`). Gemäß Entwurfsentscheidungen wurde `contextra-core-ipc-gen` formal durch `contextra-wire` abgelöst.

---

## 6. Verification & Strict Invariants Assessment

| Invariante / Policy | Bewertung & Einhaltung im Port-Entwurf |
| :--- | :--- |
| **Zero-Panic-Doctrine** | Alle neuen Trait-Methoden liefern `Result<T, ContextraError>` und propagieren Fehler via `?`. KEIN `.unwrap()`/`.expect()`. |
| **Zero-Copy & Alignment** | Dynamic Trait-Dispatch nutzt `Arc<dyn Trait>` oder Slices `&[f32]`, pass-by-reference ohne Klonen großer Payloads. |
| **DAG-Layering Integrität** | Nach Umsetzung in Welle 2 zeigt `just dag-check` 0 Verstöße; `contextra-router` (Ring 3) hängt nur noch von `contextra-ports` (Ring 0) und `contextra-core` (Ring 1) ab. |
| **Unsafe-Inseln** | Weder `contextra-router` noch `contextra-ports` enthalten `unsafe` Code (`#![forbid(unsafe_code)]`). |
| **Strangler-Prinzip / Backward-Compatibility** | Re-Export des `DriftStatusProvider` in `contextra-db` bleibt erhalten. Public API von Router und DB bleibt unterbrechungsfrei kompatibel. |

---
*Ende des Audit-Berichts `docs/refactor/router-db-edge-audit.md`*
