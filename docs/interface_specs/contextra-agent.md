# Schnittstellenspezifikation: `contextra-agent`

> Erzeugt durch `tools/extract_interfaces.py` aus dem Quellcode. Keine Handpflege. Jede Angabe trägt Datei:Zeile. Maßgeblich bleibt der Quellcode.

## 1. Kopf

| Feld | Wert |
|---|---|
| Name | `contextra-agent` |
| Version | workspace |
| Pfad | `crates/contextra-agent` |
| Rolle | Agenten-Laufzeit |
| Bibliothekspfad | src/lib.rs (Standard) |

**Crate-Dokumentation (//!):**

> Contextra Agent — Persistent workflow engine for multi-step agent execution.
>
> Implements the deterministic `checkpoint → execute → commit → audit` loop.
> Sovereign alternative to LangGraph/AutoGen: pure Rust, zero external dependencies.
>
> # Architecture Role (Layer 3)
> Built upon `contextra-db` (Collections), `contextra-checkpoint` (Snapshots & RAII CheckpointGuard),
> `contextra-graph` (Declarative StateGraph), and `contextra-store` (LSM Storage). Manages workflow
> state, token budget enforcement, and immutable audit logging over LSM-persisted keys.
>
> # State Machine Diagram & Invariants
> ```text

## 2. Abhängigkeiten

| Name | Art | Quelle | optional | verwendet von (normal/dev) |
|---|---|---|---|---|
| `bytes` | normal | workspace | nein |  |
| `contextra-checkpoint` | normal | workspace | nein | contextra-agent, contextra-db, contextra-engine |
| `contextra-db` | normal | workspace | nein | contextra, contextra-agent, contextra-bench, contextra-cognition, contextra-db-fuzz, contextra-engine |
| `contextra-graph` | dev | workspace | nein | contextra-agent, contextra-cognition, contextra-db, contextra-engine, contextra-graph, contextra-graph-fuzz |
| `contextra-ports` | normal | workspace | nein | contextra, contextra-adapt, contextra-agent, contextra-checkpoint, contextra-cognition, contextra-crypto, contextra-crypto-fuzz, contextra-db, contextra-engine, contextra-graph, contextra-graph-fuzz, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-kvcache, contextra-mcp, contextra-mvcc, contextra-rank, contextra-router, contextra-store, contextra-store-fuzz, contextra-testkit, contextra-text, contextra-vector, contextra-vector-fuzz |
| `contextra-router` | normal | workspace | nein | contextra-agent, contextra-mcp |
| `contextra-store` | dev | workspace | nein | contextra-agent, contextra-bench, contextra-db, contextra-engine, contextra-store-fuzz |
| `contextra-testkit` | dev | workspace | nein | contextra-agent, contextra-checkpoint, contextra-cognition, contextra-crypto-fuzz, contextra-db, contextra-engine, contextra-store |
| `contextra-text` | dev | workspace | nein | contextra-agent, contextra-bench, contextra-cognition, contextra-db, contextra-engine, contextra-text-fuzz |
| `contextra-types` | normal | workspace | nein | contextra, contextra-adapt, contextra-agent, contextra-bench, contextra-checkpoint, contextra-cognition, contextra-crypto, contextra-crypto-fuzz, contextra-db, contextra-engine, contextra-graph, contextra-graph-fuzz, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-ingest, contextra-kvcache, contextra-mcp, contextra-mvcc, contextra-ports, contextra-py, contextra-rank, contextra-router, contextra-simd, contextra-store, contextra-store-fuzz, contextra-testkit, contextra-text, contextra-vault, contextra-vector, contextra-vector-fuzz |
| `contextra-vector` | dev | workspace | nein | contextra-agent, contextra-bench, contextra-db, contextra-engine, contextra-vector-fuzz |
| `serde` | normal | workspace | nein |  |
| `serde_json` | normal | workspace | nein |  |
| `tempfile` | dev | workspace | nein |  |
| `thiserror` | normal | workspace | nein |  |
| `tokio` | normal | workspace | nein |  |
| `tokio-util` | normal | registry | nein |  |
| `tracing` | normal | workspace | nein |  |
| `tracing-subscriber` | dev | workspace | nein |  |

**Verwendet von (Crate-Ebene):** –

## 3. Features

| Feature | Aktiviert |
|---|---|
| `default` | – |
| `test-utils` | – |

## 4. Metriken (Quellcode)

| Kennzahl | Wert |
|---|---|
| Quelldateien | 22 |
| Quellzeilen | 5937 |
| Öffentliche Items (Modul-Ebene) | 77 |
| davon `fn` | 7 |
| davon `struct` | 29 |
| davon `enum` | 7 |
| davon `trait` | 8 |
| davon `type` | 1 |
| davon `const` | 7 |
| davon `mod` | 18 |
| Öffentliche Makros (`#[macro_export]`) | 0 |
| Öffentliche Re-Exports (`pub use`) | 17 |
| `unsafe`-Vorkommen (Produktionscode) | 0 |
| `.unwrap()` (Produktionscode) | 2 |
| `.expect(` (Produktionscode) | 24 |
| `panic!` (Produktionscode) | 0 |
| `todo!/unimplemented!/unreachable!` | 1 |
| Indexierungen `x[…]` (grob, Produktionscode) | 3 |
| `#[deprecated]` | 6 |
| `#[cfg(feature…)]`-Gates | 0 |
| Parser-Abdeckung (geparste `pub`-Deklarationen / Grep-Treffer) | 173 / 173 = 100 % |

## 5. Modulbaum und Dateien

| Modul | Datei | Zeilen |
|---|---|---|
| `audit` | `crates/contextra-agent/src/audit.rs` | 521 |
| `budget` | `crates/contextra-agent/src/budget.rs` | 99 |
| `clm_scratchpad` | `crates/contextra-agent/src/clm_scratchpad.rs` | 792 |
| `compat` | `crates/contextra-agent/src/compat/mod.rs` | 12 |
| `compat::store_adapter` | `crates/contextra-agent/src/compat/store_adapter.rs` | 59 |
| `context` | `crates/contextra-agent/src/context.rs` | 580 |
| `dlq` | `crates/contextra-agent/src/dlq.rs` | 418 |
| `engine::condition` | `crates/contextra-agent/src/engine/condition.rs` | 157 |
| `engine::tests` | `crates/contextra-agent/src/engine/tests.rs` | 277 |
| `engine` | `crates/contextra-agent/src/engine.rs` | 879 |
| `error` | `crates/contextra-agent/src/error.rs` | 72 |
| `event_source` | `crates/contextra-agent/src/event_source.rs` | 304 |
| `goal_condition` | `crates/contextra-agent/src/goal_condition.rs` | 504 |
| `graph` | `crates/contextra-agent/src/graph.rs` | 332 |
| `crate` | `crates/contextra-agent/src/lib.rs` | 114 |
| `ports` | `crates/contextra-agent/src/ports.rs` | 74 |
| `step` | `crates/contextra-agent/src/step.rs` | 123 |
| `tools::manifest` | `crates/contextra-agent/src/tools/manifest.rs` | 143 |
| `tools` | `crates/contextra-agent/src/tools/mod.rs` | 25 |
| `tools::registry` | `crates/contextra-agent/src/tools/registry.rs` | 115 |
| `tools::retrieve` | `crates/contextra-agent/src/tools/retrieve.rs` | 159 |
| `tools::tests` | `crates/contextra-agent/src/tools/tests.rs` | 178 |

## 6. Re-Exports (`pub use`)

- `pub use contextra_types::types::budget::` — `crates/contextra-agent/src/budget.rs:16` (Modul `budget`)
- `pub use store_adapter::*` — `crates/contextra-agent/src/compat/mod.rs:10` (Modul `compat`)
- `pub use condition::evaluate_condition_expr` — `crates/contextra-agent/src/engine.rs:816` (Modul `engine`)
- `pub use error::AgentError` — `crates/contextra-agent/src/lib.rs:85` (Modul `crate`)
- `pub use budget::` — `crates/contextra-agent/src/lib.rs:99` (Modul `crate`)
- `pub use clm_scratchpad::` — `crates/contextra-agent/src/lib.rs:100` (Modul `crate`)
- `pub use context::` — `crates/contextra-agent/src/lib.rs:105` (Modul `crate`)
- `pub use dlq::DeadLetterQueue` — `crates/contextra-agent/src/lib.rs:106` (Modul `crate`)
- `pub use engine::` — `crates/contextra-agent/src/lib.rs:107` (Modul `crate`)
- `pub use event_source::` — `crates/contextra-agent/src/lib.rs:108` (Modul `crate`)
- `pub use goal_condition::` — `crates/contextra-agent/src/lib.rs:109` (Modul `crate`)
- `pub use graph::` — `crates/contextra-agent/src/lib.rs:110` (Modul `crate`)
- `pub use step::` — `crates/contextra-agent/src/lib.rs:111` (Modul `crate`)
- `pub use tools::` — `crates/contextra-agent/src/lib.rs:112` (Modul `crate`)
- `pub use manifest::ToolManifest` — `crates/contextra-agent/src/tools/mod.rs:18` (Modul `tools`)
- `pub use registry::ToolRegistry` — `crates/contextra-agent/src/tools/mod.rs:19` (Modul `tools`)
- `pub use retrieve::` — `crates/contextra-agent/src/tools/mod.rs:20` (Modul `tools`)

## 8. Öffentliche Schnittstelle je Item

### Modul `audit`

#### `struct AuditEntry`
- Ort: `crates/contextra-agent/src/audit.rs:29`
```rust
pub struct AuditEntry
```
Dokumentation: Single immutable record of an agent step execution.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `task_id` | `String` | `crates/contextra-agent/src/audit.rs:31` |
| `step_count` | `u64` | `crates/contextra-agent/src/audit.rs:33` |
| `node_id` | `String` | `crates/contextra-agent/src/audit.rs:35` |
| `tokens_consumed` | `usize` | `crates/contextra-agent/src/audit.rs:37` |
| `payload` | `serde_json::Value` | `crates/contextra-agent/src/audit.rs:39` |
| `error` | `Option<String>` | `crates/contextra-agent/src/audit.rs:42` |
| `tx_id` | `Option<contextra_types::TxId>` | `crates/contextra-agent/src/audit.rs:45` |

#### `struct AuditLog`
- Ort: `crates/contextra-agent/src/audit.rs:58`
```rust
pub struct AuditLog<S: StorageEngine>
```
Dokumentation: Append-only audit log backed by a Contextra collection.
- Impl-Block `crates/contextra-agent/src/audit.rs:98`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(collection: Arc<Collection<S>>) -> Self` | `crates/contextra-agent/src/audit.rs:100` | Constructs a new [`AuditLog`] wrapping a collection reference. |
| `pub async fn append_to(collection: &Collection<S>, entry: &AuditEntry) -> Result<()>` | `crates/contextra-agent/src/audit.rs:105` | Appends an immutable audit entry directly to a collection without needing an [`AuditLog`] wrapper. |
| `pub async fn append(&self, entry: &AuditEntry) -> Result<()>` | `crates/contextra-agent/src/audit.rs:121` | Appends an immutable audit entry directly via LSM storage without HNSW vector index participation (AC-3). Enforces append-only immutability: returns `ContextraE |
| `pub async fn replay_task(&self, task_id: &str) -> Result<Vec<AuditEntry>>` | `crates/contextra-agent/src/audit.rs:126` | Replays all audit entries for a given task via scan_prefix. |

#### `struct MigrationStats`
- Ort: `crates/contextra-agent/src/audit.rs:50`
```rust
pub struct MigrationStats
```
Dokumentation: Summary statistics for legacy audit entry migration.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `migrated` | `usize` | `crates/contextra-agent/src/audit.rs:52` |
| `failed` | `usize` | `crates/contextra-agent/src/audit.rs:54` |

#### `fn migrate_legacy_audit_entries`
- Ort: `crates/contextra-agent/src/audit.rs:209`
```rust
pub async fn migrate_legacy_audit_entries<S: StorageEngine, V: contextra_ports::VectorIndex>( collection: &Collection<S, V>, ) -> Result<MigrationStats>
```
Dokumentation: Migrates legacy zero-vector audit entries from the HNSW vector index and doc_key mappings into the direct LSM KV store format (`put_kv`). Returns [`MigrationStats`] detailing successfully migrated and failed entries.

#### `fn validate_audit_payload_and_error`
- Ort: `crates/contextra-agent/src/audit.rs:63`
```rust
pub fn validate_audit_payload_and_error( payload: &serde_json::Value, error: Option<&str>, ) -> Result<()>
```
Dokumentation: Validates payload and optional error string of an audit entry for non-emptiness and absence of null bytes.

### Modul `budget`

#### `fn reserve_tokens`
- Ort: `crates/contextra-agent/src/budget.rs:24`
```rust
pub fn reserve_tokens<'a>( budget: &'a TokenBudget, amount: usize, ) -> contextra_types::Result<Reservation<'a>>
```
Dokumentation: Helper to reserve `amount` tokens from `budget`. Returns an RAII [`Reservation`] guard on success. If execution drops the guard without calling `reservation.settle()`, the reserved tokens are automatically refunded.

### Modul `clm_scratchpad`

#### `struct ClmScratchpad`
- Ort: `crates/contextra-agent/src/clm_scratchpad.rs:213`
```rust
pub struct ClmScratchpad
```
Dokumentation: Temporärer, aufgabengebundener Arbeitskontext-Scratchpad für Agentenschleifen. Kapselt eine [`VolatileContextVault`]-Instanz für die sichere RAM-native Speicherung (mlock, Zeroize bei Drop) ohne persistenten Disk-Schreibpfad.
- Impl-Block `crates/contextra-agent/src/clm_scratchpad.rs:242`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(task_id: String, config: VaultConfig) -> Self` | `crates/contextra-agent/src/clm_scratchpad.rs:244` | Erzeugt einen neuen `ClmScratchpad` für eine Task mit gegebener Vault-Konfiguration. |
| `pub fn with_tenant_id(mut self, tenant_id: TenantId) -> Self` | `crates/contextra-agent/src/clm_scratchpad.rs:263` | Setzt die Mandanten-ID (Builder-Muster). |
| `pub fn with_clock(mut self, clock: Arc<dyn Clock>) -> Self` | `crates/contextra-agent/src/clm_scratchpad.rs:269` | Setzt die Clock für deterministisches Zeitmanagement (Builder-Muster). |
| `pub fn with_audit_sink(mut self, audit_sink: Arc<dyn ContextEditAuditSink>) -> Self` | `crates/contextra-agent/src/clm_scratchpad.rs:275` | Setzt die Audit-Sink (Builder-Muster). |
| `pub fn with_cache_invalidator( mut self, cache_invalidator: Arc<dyn ScratchpadCacheInvalidator>, ) -> Self` | `crates/contextra-agent/src/clm_scratchpad.rs:281` | Setzt den Cache-Invalidator (Builder-Muster). |
| `pub fn with_pinned_region(mut self, region_id: PinnedRegionId) -> Self` | `crates/contextra-agent/src/clm_scratchpad.rs:290` | Registriert eine gepinnte Region (unveränderlicher Kontextteil). |
| `pub fn with_created_at_tx(mut self, created_at_tx: TxId) -> Self` | `crates/contextra-agent/src/clm_scratchpad.rs:298` | Setzt die Transaktions-ID der Erstellung (Builder-Muster). |
| `pub fn task_id(&self) -> &str` | `crates/contextra-agent/src/clm_scratchpad.rs:304` | Gibt die Task-ID zurück. |
| `pub fn tenant_id(&self) -> TenantId` | `crates/contextra-agent/src/clm_scratchpad.rs:309` | Gibt die Mandanten-ID zurück. |
| `pub fn current_subgoal_index(&self) -> u64` | `crates/contextra-agent/src/clm_scratchpad.rs:314` | Gibt den aktuellen Subgoal-Index zurück. |
| `pub fn created_at_tx(&self) -> u64` | `crates/contextra-agent/src/clm_scratchpad.rs:319` | Gibt die Transaktions-ID der Erzeugung zurück. |
| `pub fn pinned_regions(&self) -> &[PinnedRegionId]` | `crates/contextra-agent/src/clm_scratchpad.rs:324` | Gibt eine immutable Referenz auf die registrierten gepinnten Regionen zurück. |
| `pub fn apply_edit(&mut self, op: ScratchpadEditOp, captured_tx: TxId) -> Result<()>` | `crates/contextra-agent/src/clm_scratchpad.rs:469` | Wendet eine Bearbeitungs-Operation ([`ScratchpadEditOp`]) auf das Scratchpad an. # Fehler Liefert `ContextraError::InvalidInput` falls: - Versucht wird, eine ge |
| `pub fn checkpoint_and_reset(&mut self) -> Result<ScratchpadCheckpoint>` | `crates/contextra-agent/src/clm_scratchpad.rs:501` | Erzeugt eine [`ScratchpadCheckpoint`]-Zusammenfassung, leert den Vault intern via `purge()` (inkl. Zeroize aller RAM-Inhalte), meldet Audit und Cache-Invalidier |
| `pub fn total_bytes(&self) -> usize` | `crates/contextra-agent/src/clm_scratchpad.rs:542` | Gibt die Gesamtzahl der belegten Bytes im aktuellen Scratchpad-Vault zurück. |

#### `struct ContextEditAuditRecord`
- Ort: `crates/contextra-agent/src/clm_scratchpad.rs:37`
```rust
pub struct ContextEditAuditRecord
```
Dokumentation: Audit-Datensatz für Scratchpad-Kontextänderungen und Resets.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `tenant_id` | `TenantId` | `crates/contextra-agent/src/clm_scratchpad.rs:39` |
| `task_id` | `String` | `crates/contextra-agent/src/clm_scratchpad.rs:41` |
| `operation` | `String` | `crates/contextra-agent/src/clm_scratchpad.rs:43` |
| `subgoal_index` | `u64` | `crates/contextra-agent/src/clm_scratchpad.rs:45` |
| `timestamp_nanos` | `u64` | `crates/contextra-agent/src/clm_scratchpad.rs:47` |

#### `struct CountingContextEditAuditSink`
- Ort: `crates/contextra-agent/src/clm_scratchpad.rs:66`
```rust
pub struct CountingContextEditAuditSink
```
Dokumentation: Test-Implementierung von [`ContextEditAuditSink`], die Aufrufe zählt.
- Impl-Block `crates/contextra-agent/src/clm_scratchpad.rs:70`
- Impl-Block (Trait `ContextEditAuditSink`) `crates/contextra-agent/src/clm_scratchpad.rs:84`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-agent/src/clm_scratchpad.rs:72` | Constructs a new `CountingContextEditAuditSink`. |
| `pub fn count(&self) -> u64` | `crates/contextra-agent/src/clm_scratchpad.rs:79` | Returns the accumulated count of recorded edit operations. |

#### `struct CountingScratchpadCacheInvalidator`
- Ort: `crates/contextra-agent/src/clm_scratchpad.rs:112`
```rust
pub struct CountingScratchpadCacheInvalidator
```
Dokumentation: Test-Implementierung von [`ScratchpadCacheInvalidator`], die Aufrufe zählt.
- Impl-Block `crates/contextra-agent/src/clm_scratchpad.rs:116`
- Impl-Block (Trait `ScratchpadCacheInvalidator`) `crates/contextra-agent/src/clm_scratchpad.rs:130`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-agent/src/clm_scratchpad.rs:118` | Constructs a new `CountingScratchpadCacheInvalidator`. |
| `pub fn count(&self) -> u64` | `crates/contextra-agent/src/clm_scratchpad.rs:125` | Returns the accumulated count of invalidation calls. |

#### `struct NoopContextEditAuditSink`
- Ort: `crates/contextra-agent/src/clm_scratchpad.rs:58`
```rust
pub struct NoopContextEditAuditSink
```
Dokumentation: No-op-Implementierung von [`ContextEditAuditSink`].
- Impl-Block (Trait `ContextEditAuditSink`) `crates/contextra-agent/src/clm_scratchpad.rs:60`

#### `struct NoopScratchpadCacheInvalidator`
- Ort: `crates/contextra-agent/src/clm_scratchpad.rs:98`
```rust
pub struct NoopScratchpadCacheInvalidator
```
Dokumentation: No-op-Implementierung von [`ScratchpadCacheInvalidator`].
- Impl-Block (Trait `ScratchpadCacheInvalidator`) `crates/contextra-agent/src/clm_scratchpad.rs:100`

#### `struct PinnedRegionId`
- Ort: `crates/contextra-agent/src/clm_scratchpad.rs:145`
```rust
pub struct PinnedRegionId(pub String)
```
Dokumentation: Marker für unveränderliche Kontextteile (Systemprompt, Gesamtziel, Safety-Regeln). Wird ausschließlich gelesen und NIEMALS durch diesen Typ selbst verändert.
- Impl-Block `crates/contextra-agent/src/clm_scratchpad.rs:147`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(id: impl Into<String>) -> Self` | `crates/contextra-agent/src/clm_scratchpad.rs:149` | Erzeugt eine neue `PinnedRegionId`. |
| `pub fn as_str(&self) -> &str` | `crates/contextra-agent/src/clm_scratchpad.rs:154` | Gibt den inneren String-Bezeichner zurück. |

#### `struct ScratchpadCheckpoint`
- Ort: `crates/contextra-agent/src/clm_scratchpad.rs:198`
```rust
pub struct ScratchpadCheckpoint
```
Dokumentation: Zusammenfassung eines Subgoal-Checkpoints vor dem Zurücksetzen des Scratchpads. Dieser Typ dient als Übergabeobjekt für nachfolgende Stufen (z. B. Speicher-Konsolidierung).

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `task_id` | `String` | `crates/contextra-agent/src/clm_scratchpad.rs:200` |
| `completed_subgoal_index` | `u64` | `crates/contextra-agent/src/clm_scratchpad.rs:202` |
| `reset_receipt` | `VaultResetReceipt` | `crates/contextra-agent/src/clm_scratchpad.rs:204` |
| `chunk_count_at_checkpoint` | `usize` | `crates/contextra-agent/src/clm_scratchpad.rs:206` |

#### `struct VaultResetReceipt`
- Ort: `crates/contextra-agent/src/clm_scratchpad.rs:185`
```rust
pub struct VaultResetReceipt
```
Dokumentation: Quittung über das Zurücksetzen des flüchtigen Kontext-Speichers.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `chunks_purged` | `usize` | `crates/contextra-agent/src/clm_scratchpad.rs:187` |
| `bytes_zeroed` | `usize` | `crates/contextra-agent/src/clm_scratchpad.rs:189` |
| `reset_at` | `std::time::Instant` | `crates/contextra-agent/src/clm_scratchpad.rs:191` |

#### `enum ScratchpadEditOp`
- Ort: `crates/contextra-agent/src/clm_scratchpad.rs:161`
```rust
pub enum ScratchpadEditOp
```
Dokumentation: Operationen zur Bearbeitung des Scratchpads.

Varianten:

- `Append{` — `crates/contextra-agent/src/clm_scratchpad.rs:163`
- `Replace{` — `crates/contextra-agent/src/clm_scratchpad.rs:170`
- `Remove{` — `crates/contextra-agent/src/clm_scratchpad.rs:177`

#### `trait ContextEditAuditSink`
- Ort: `crates/contextra-agent/src/clm_scratchpad.rs:51`
```rust
pub trait ContextEditAuditSink: Send + Sync
```
Dokumentation: Lokaler Trait für Audit-Benachrichtigungen bei Scratchpad-Änderungen.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn record_edit(&self, record: ContextEditAuditRecord)` | `crates/contextra-agent/src/clm_scratchpad.rs:53` | Records a scratchpad edit operation audit entry. |

#### `trait ScratchpadCacheInvalidator`
- Ort: `crates/contextra-agent/src/clm_scratchpad.rs:91`
```rust
pub trait ScratchpadCacheInvalidator: Send + Sync
```
Dokumentation: Lokaler Trait zur Cache-Invalidierung des Scratchpad-Scopes bei Resets.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn invalidate_scratchpad_scope(&self, tenant_id: TenantId, task_id: &str, subgoal_index: u64)` | `crates/contextra-agent/src/clm_scratchpad.rs:93` | Invalidates cache entries associated with a scratchpad subgoal scope. |

### Modul `compat`

#### `mod store_adapter`
- Ort: `crates/contextra-agent/src/compat/mod.rs:9`
```rust
pub mod store_adapter
```

### Modul `compat::store_adapter`

#### `struct DbCollectionStore`
- Ort: `crates/contextra-agent/src/compat/store_adapter.rs:18`
```rust
pub struct DbCollectionStore<'a>
```
Dokumentation: Adapter für eine persistente contextra-db Collection.
- Impl-Block `crates/contextra-agent/src/compat/store_adapter.rs:22`
- Impl-Block (Trait `ContextStore`) `crates/contextra-agent/src/compat/store_adapter.rs:29`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(collection: &'a contextra_db::Collection) -> Self` | `crates/contextra-agent/src/compat/store_adapter.rs:24` | Erstellt eine neue Adapter-Instanz. |

### Modul `context`

#### `struct AgentContext`
- Ort: `crates/contextra-agent/src/context.rs:89`
```rust
pub struct AgentContext
```
Dokumentation: Operational context spanning the entire workflow execution.
- Impl-Block `crates/contextra-agent/src/context.rs:123`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `task_id` | `String` | `crates/contextra-agent/src/context.rs:91` |
| `current_node` | `String` | `crates/contextra-agent/src/context.rs:93` |
| `step_count` | `u64` | `crates/contextra-agent/src/context.rs:95` |
| `db` | `Arc<Contextra>` | `crates/contextra-agent/src/context.rs:97` |
| `state_collection` | `Arc<Collection>` | `crates/contextra-agent/src/context.rs:99` |
| `budget` | `TokenBudget` | `crates/contextra-agent/src/context.rs:101` |
| `status` | `AgentStatus` | `crates/contextra-agent/src/context.rs:103` |
| `current_cache_directive` | `CacheDirective` | `crates/contextra-agent/src/context.rs:105` |
| `events` | `VecDeque<crate::event_source::BackgroundEvent>` | `crates/contextra-agent/src/context.rs:111` |
| `pending_routing_decision` | `Option<(` | `crates/contextra-agent/src/context.rs:113` |
| `clm_scratchpad` | `Option<crate::clm_scratchpad::ClmScratchpad>` | `crates/contextra-agent/src/context.rs:118` |
| `latest_scratchpad_checkpoint` | `Option<crate::clm_scratchpad::ScratchpadCheckpoint>` | `crates/contextra-agent/src/context.rs:120` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn try_new( task_id: impl Into<String>, start_node: impl Into<String>, db: Arc<Contextra>, state_collection: Arc<Collection>, budget: TokenBudget, ) -> contextra_types::Result<Self>` | `crates/contextra-agent/src/context.rs:126` |  |
| `pub fn with_clm_scratchpad(mut self, scratchpad: crate::clm_scratchpad::ClmScratchpad) -> Self` | `crates/contextra-agent/src/context.rs:167` | Attaches an optional [`ClmScratchpad`](crate::clm_scratchpad::ClmScratchpad) to the agent context (Builder pattern). |
| `pub fn set_cache_directive(&mut self, directive: CacheDirective)` | `crates/contextra-agent/src/context.rs:173` | Sets the active cache directive for the context. |
| `pub fn get_cache_directive(&self) -> &CacheDirective` | `crates/contextra-agent/src/context.rs:178` | Returns the active cache directive for the context. |
| `pub fn set_step_cache_directive(&mut self, step_id: StepId, directive: CacheDirective)` | `crates/contextra-agent/src/context.rs:183` | Records a cache directive for a specific step ID. |
| `pub fn directive_for_node(&self, node_id: &str, step_id: StepId) -> CacheDirective` | `crates/contextra-agent/src/context.rs:189` | Returns the appropriate `CacheDirective` for a workflow node / prompt role. |
| `pub fn attach_event(&mut self, event: crate::event_source::BackgroundEvent)` | `crates/contextra-agent/src/context.rs:203` | Integrates a background telemetry event into the agent context memory and history. Maintains event history capacity bounded at `MAX_TELEMETRY_EVENTS` to prevent |
| `pub fn next_retry_step_count(&self) -> u64` | `crates/contextra-agent/src/context.rs:214` | Returns the step count to be used for a recovery retry after a partial step failure. |
| `pub fn try_attach_event(&mut self, event: crate::event_source::BackgroundEvent) -> Result<()>` | `crates/contextra-agent/src/context.rs:219` | Integrates a background telemetry event with an explicit capacity check. |

#### `struct AgentEngine`
- Ort: `crates/contextra-agent/src/context.rs:235`
```rust
pub struct AgentEngine
```
Dokumentation: Agent execution engine helper managing per-step cache directives during workflow execution.
- Impl-Block `crates/contextra-agent/src/context.rs:240`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `current_directive` | `CacheDirective` | `crates/contextra-agent/src/context.rs:237` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-agent/src/context.rs:242` | Constructs a new `AgentEngine`. |
| `pub fn directive_for_step(&self, node_id: &str, step_id: StepId) -> CacheDirective` | `crates/contextra-agent/src/context.rs:253` | Determines the appropriate `CacheDirective` for a node ID / prompt role. - System prompt nodes -> `CacheDirective::Pin { ttl: None }` - Transient reasoning step |
| `pub fn run( &mut self, ctx: &mut AgentContext, step_id: StepId, node_id: &str, ) -> CacheDirective` | `crates/contextra-agent/src/context.rs:265` | Executes step configuration, updating context with per-step cache directives. |

#### `enum AgentStatus`
- Ort: `crates/contextra-agent/src/context.rs:77`
```rust
pub enum AgentStatus
```
Dokumentation: Operational status of an agent workflow.

Varianten:

- `Idle` — `crates/contextra-agent/src/context.rs:79`
- `Running` — `crates/contextra-agent/src/context.rs:81`
- `Completed` — `crates/contextra-agent/src/context.rs:83`
- `Failed` — `crates/contextra-agent/src/context.rs:85`

#### `fn validate_node_id`
- Ort: `crates/contextra-agent/src/context.rs:54`
```rust
pub fn validate_node_id(node_id: &str) -> Result<()>
```
Dokumentation: Validates a node identifier to ensure it is non-empty, <= 256 bytes, and contains no null bytes.

#### `fn validate_task_id`
- Ort: `crates/contextra-agent/src/context.rs:32`
```rust
pub fn validate_task_id(task_id: &str) -> Result<()>
```
Dokumentation: Validates a task identifier to ensure it is non-empty, <= 256 bytes, and contains no null bytes.

#### `const MAX_ID_LEN`
- Ort: `crates/contextra-agent/src/context.rs:26`
```rust
pub const MAX_ID_LEN: usize = 256
```
Dokumentation: Maximum allowed length in bytes for task IDs and node IDs.

#### `const MAX_TELEMETRY_EVENTS`
- Ort: `crates/contextra-agent/src/context.rs:29`
```rust
pub const MAX_TELEMETRY_EVENTS: usize = 10_000
```
Dokumentation: Maximum allowed telemetry events stored in memory history.

### Modul `crate`

#### `mod audit`
- Ort: `crates/contextra-agent/src/lib.rs:87`
```rust
pub mod audit
```

#### `mod budget`
- Ort: `crates/contextra-agent/src/lib.rs:88`
```rust
pub mod budget
```

#### `mod clm_scratchpad`
- Ort: `crates/contextra-agent/src/lib.rs:89`
```rust
pub mod clm_scratchpad
```

#### `mod compat`
- Ort: `crates/contextra-agent/src/lib.rs:82`
```rust
pub mod compat
```

#### `mod context`
- Ort: `crates/contextra-agent/src/lib.rs:90`
```rust
pub mod context
```

#### `mod dlq`
- Ort: `crates/contextra-agent/src/lib.rs:91`
```rust
pub mod dlq
```

#### `mod engine`
- Ort: `crates/contextra-agent/src/lib.rs:92`
```rust
pub mod engine
```

#### `mod error`
- Ort: `crates/contextra-agent/src/lib.rs:83`
```rust
pub mod error
```

#### `mod event_source`
- Ort: `crates/contextra-agent/src/lib.rs:93`
```rust
pub mod event_source
```

#### `mod goal_condition`
- Ort: `crates/contextra-agent/src/lib.rs:94`
```rust
pub mod goal_condition
```

#### `mod graph`
- Ort: `crates/contextra-agent/src/lib.rs:95`
```rust
pub mod graph
```

#### `mod ports`
- Ort: `crates/contextra-agent/src/lib.rs:84`
```rust
pub mod ports
```

#### `mod step`
- Ort: `crates/contextra-agent/src/lib.rs:96`
```rust
pub mod step
```

#### `mod tools`
- Ort: `crates/contextra-agent/src/lib.rs:97`
```rust
pub mod tools
```

### Modul `dlq`

#### `struct DeadLetterQueue`
- Ort: `crates/contextra-agent/src/dlq.rs:49`
```rust
pub struct DeadLetterQueue
```
Dokumentation: Persistente Dead-Letter-Queue für fehlgeschlagene Agent-Schritte. Verwendet denselben Storage wie der Agent (LSM) mit einem fixen Key-Prefix.
- Impl-Block `crates/contextra-agent/src/dlq.rs:55`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(storage: Arc<dyn StorageEngine>) -> Self` | `crates/contextra-agent/src/dlq.rs:60` | Constructs a new [`DeadLetterQueue`] backed by the provided storage engine. |
| `pub fn with_metrics_sink(mut self, sink: Arc<dyn contextra_ports::MetricsSink>) -> Self` | `crates/contextra-agent/src/dlq.rs:69` | Attaches a metrics sink for DLQ gauge telemetry (Builder pattern). |
| `pub fn new_with_metrics( storage: Arc<dyn StorageEngine>, sink: Arc<dyn contextra_ports::MetricsSink>, ) -> Self` | `crates/contextra-agent/src/dlq.rs:75` | Helper constructor creating a [`DeadLetterQueue`] with an explicit metrics sink. |
| `pub async fn push(&self, letter: &StepDeadLetter) -> Result<()>` | `crates/contextra-agent/src/dlq.rs:92` | Pushes a failed step record to the persistent Dead-Letter Queue. |
| `pub async fn drain(&self) -> Result<Vec<StepDeadLetter>>` | `crates/contextra-agent/src/dlq.rs:118` | Reads and removes all dead-letter records atomically from storage. |
| `pub async fn list(&self) -> Result<Vec<StepDeadLetter>>` | `crates/contextra-agent/src/dlq.rs:154` | Lists all current dead-letter records without removing them. |
| `pub async fn remove(&self, letter: &StepDeadLetter) -> Result<bool>` | `crates/contextra-agent/src/dlq.rs:172` | Entfernt einen spezifischen DLQ-Eintrag anhand seines `(session_id, node_id, step_index)` Schlüssels. |
| `pub async fn is_already_committed(&self, letter: &StepDeadLetter) -> Result<bool>` | `crates/contextra-agent/src/dlq.rs:199` | Prüft vor dem Replay eines DLQ-Eintrags, ob die im Eintrag assoziierte `TxId` bereits im WAL/Storage für denselben `(Session, Node, Step)`-Schlüssel committet i |
| `pub async fn allocate_tx(&self) -> Result<TxId>` | `crates/contextra-agent/src/dlq.rs:236` | Allocates a unique sequence transaction ID for DLQ storage mutations. |

Assoziierte Konstanten/Typen:

- `pub const PREFIX: &'static [u8] = b""` — `crates/contextra-agent/src/dlq.rs:57`

### Modul `engine`

#### `struct OrchestratorEngine`
- Ort: `crates/contextra-agent/src/engine.rs:52`
```rust
pub struct OrchestratorEngine
```
- Impl-Block `crates/contextra-agent/src/engine.rs:63`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `registry` | `crate::tools::ToolRegistry` | `crates/contextra-agent/src/engine.rs:54` |
| `checkpoint_store` | `Arc<dyn AgentCheckpointStore>` | `crates/contextra-agent/src/engine.rs:56` |
| `dead_letter_queue` | `Option<DeadLetterQueue>` | `crates/contextra-agent/src/engine.rs:58` |
| `clock` | `Arc<dyn Clock>` | `crates/contextra-agent/src/engine.rs:60` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn try_new<S: StorageEngine>(storage: Arc<S>) -> Result<Self>` | `crates/contextra-agent/src/engine.rs:65` | Attempts to construct a new [`OrchestratorEngine`], returning an error if checkpoint store initialization fails. |
| `pub fn with_clock(mut self, clock: Arc<dyn Clock>) -> Self` | `crates/contextra-agent/src/engine.rs:76` | Sets a custom [`Clock`] implementation for deterministic time management. |
| `pub fn try_from_db(db: &contextra_db::Contextra) -> Result<Self>` | `crates/contextra-agent/src/engine.rs:90` | Attempts to construct an [`OrchestratorEngine`] directly from a Contextra DB handle. |
| `pub fn new<S: StorageEngine>(storage: Arc<S>) -> Self` | `crates/contextra-agent/src/engine.rs:96` | Constructs a new [`OrchestratorEngine`] from LSM storage (legacy constructor). |
| `pub fn from_db(db: &contextra_db::Contextra) -> Self` | `crates/contextra-agent/src/engine.rs:113` |  |
| `pub fn try_register_tool(&mut self, tool: Box<dyn AgentTool>) -> Result<()>` | `crates/contextra-agent/src/engine.rs:118` | Attempts to register an agent tool with boundary validation on the tool name. |
| `pub fn try_register_tool_with_manifest( &mut self, manifest: crate::tools::ToolManifest, tool: Box<dyn AgentTool>, ) -> Result<()>` | `crates/contextra-agent/src/engine.rs:123` | Registers an agent tool with an explicit declarative [`ToolManifest`](crate::tools::ToolManifest). |
| `pub async fn recover_orphans(&self) -> Result<()>` | `crates/contextra-agent/src/engine.rs:132` | Recovers all registered/persisted orphaned sequence pins and checkpoints. |
| `pub async fn run(&self, ctx: &mut AgentContext, graph: &StateGraph) -> Result<()>` | `crates/contextra-agent/src/engine.rs:139` | Runs the persistent workflow execution loop for a given [`AgentContext`] and [`StateGraph`]. |
| `pub async fn replay_from(&self, ctx: &mut AgentContext, identifier: &str) -> Result<()>` | `crates/contextra-agent/src/engine.rs:581` | Setzt den AgentContext auf einen früheren Checkpoint zurück (AC-2). # Adressierung von Checkpoints Das `identifier`-Argument unterstützt folgende Adressierungsf |
| `pub async fn checkpoint(&self, ctx: &AgentContext) -> Result<()>` | `crates/contextra-agent/src/engine.rs:596` | Persists a workflow execution state checkpoint for the active context step. |
| `pub async fn run_event_loop( &self, ctx: &mut AgentContext, graph: &StateGraph, source: &mut dyn crate::event_source::EventSource, shutdown: tokio_util::sync::CancellationToken, ) -> Result<EventLoopExitReason>` | `crates/contextra-agent/src/engine.rs:630` | Continuous event loop reading telemetry events from an `EventSource`, attaching each event to `AgentContext`, executing `run()`, and checkpointing state after e |

#### `enum EventLoopExitReason`
- Ort: `crates/contextra-agent/src/engine.rs:33`
```rust
pub enum EventLoopExitReason
```
Dokumentation: Reason for exiting `OrchestratorEngine::run_event_loop`.

Varianten:

- `Shutdown` — `crates/contextra-agent/src/engine.rs:35`
- `SourceExhausted` — `crates/contextra-agent/src/engine.rs:37`

#### `trait AgentCheckpointStore`
- Ort: `crates/contextra-agent/src/engine.rs:48`
```rust
pub trait AgentCheckpointStore: CheckpointRegistry + Checkpoint + Send + Sync
```
Dokumentation: Async executor engine applying nodes in sequence. Checkpoint-Naht des Agenten: Registry-Operationen plus Zustands-Restore. Der Blanket-Impl macht jede Implementierung beider Traits verwendbar. So kann `dyn AgentCheckpointStore` beide Schnittstellen aufrufen.

#### `const MAX_WORKFLOW_STEPS`
- Ort: `crates/contextra-agent/src/engine.rs:41`
```rust
pub const MAX_WORKFLOW_STEPS: u64 = 10_000
```
Dokumentation: Maximum allowed steps in a single workflow execution to prevent unbounded loops.

### Modul `engine::condition`

#### `fn evaluate_condition_expr`
- Ort: `crates/contextra-agent/src/engine/condition.rs:104`
```rust
pub fn evaluate_condition_expr(expr: &str, ctx: &AgentContext) -> bool
```
Dokumentation: Evaluates a declarative condition expression against the provided [`AgentContext`]. # Supported Grammar: - `<key> exists`: Returns `true` if `<key>` is present in context memory or context properties and is not `null`. - `<key> == <value>`: Returns `true` if the value at `<key>` matches `<value>`. - `<key> != <value>`: Returns `true` if the value at `<key>` does not match `<value>` (or if `<key>` does not exist). Key resolution supports direct keys in `ctx.memory` (e.g. `"result"`), nested dot-notation paths (e.g. `"output.status"`), and built-in context fields (`"task_id"`, `"current_node"`, 

### Modul `error`

#### `enum AgentError`
- Ort: `crates/contextra-agent/src/error.rs:20`
```rust
pub enum AgentError
```
Dokumentation: Lokaler Fehlertyp für Workflow-, Zustands- und Tool-Operationen.

Varianten:

- `NodeNotFound(String)` — `crates/contextra-agent/src/error.rs:23`
- `InvalidIdentifier(String)` — `crates/contextra-agent/src/error.rs:27`
- `BudgetExhausted(String)` — `crates/contextra-agent/src/error.rs:31`
- `ToolExecution{` — `crates/contextra-agent/src/error.rs:35`
- `Internal(String)` — `crates/contextra-agent/src/error.rs:44`

### Modul `event_source`

#### `struct BackgroundEvent`
- Ort: `crates/contextra-agent/src/event_source.rs:32`
```rust
pub struct BackgroundEvent
```
Dokumentation: Background telemetry/trigger event delivered to the agent context.
- Impl-Block `crates/contextra-agent/src/event_source.rs:41`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `payload` | `serde_json::Value` | `crates/contextra-agent/src/event_source.rs:34` |
| `source` | `String` | `crates/contextra-agent/src/event_source.rs:36` |
| `observed_at_seq` | `u64` | `crates/contextra-agent/src/event_source.rs:38` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn try_new( payload: serde_json::Value, source: impl Into<String>, observed_at_seq: u64, ) -> Result<Self>` | `crates/contextra-agent/src/event_source.rs:44` |  |
| `pub fn new( payload: serde_json::Value, source: impl Into<String>, observed_at_seq: u64, ) -> Self` | `crates/contextra-agent/src/event_source.rs:76` | Legacy constructor for `BackgroundEvent` (deprecated). |

#### `struct PollingDocumentEventSource`
- Ort: `crates/contextra-agent/src/event_source.rs:126`
```rust
pub struct PollingDocumentEventSource<S: StorageEngine>
```
Dokumentation: Concrete `EventSource` that periodically polls `Collection` storage sequence numbers, using `scan_prefix_at` for snapshot delta calculation to emit document changes.
- Impl-Block `crates/contextra-agent/src/event_source.rs:133`
- Impl-Block (Trait `EventSource`) `crates/contextra-agent/src/event_source.rs:166`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(collection: Arc<Collection<S>>, poll_interval: Duration) -> Self` | `crates/contextra-agent/src/event_source.rs:135` | Creates a new `PollingDocumentEventSource` watching the specified collection. |
| `pub fn with_capacity( collection: Arc<Collection<S>>, poll_interval: Duration, _max_pending_capacity: usize, ) -> Self` | `crates/contextra-agent/src/event_source.rs:141` |  |
| `pub fn with_last_seen_seq(mut self, last_seen_seq: u64) -> Self` | `crates/contextra-agent/src/event_source.rs:155` | Configures the initial last seen sequence number (Builder pattern). |
| `pub fn poll_interval(&self) -> Duration` | `crates/contextra-agent/src/event_source.rs:161` | Returns the configured polling interval duration. |

#### `struct VecEventSource`
- Ort: `crates/contextra-agent/src/event_source.rs:233`
```rust
pub struct VecEventSource
```
Dokumentation: Trivial mock/static `EventSource` that reads from a fixed list of `BackgroundEvent`s.
- Impl-Block `crates/contextra-agent/src/event_source.rs:237`
- Impl-Block (Trait `EventSource`) `crates/contextra-agent/src/event_source.rs:253`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn try_new(events: Vec<BackgroundEvent>) -> Result<Self>` | `crates/contextra-agent/src/event_source.rs:239` | Attempts to construct a `VecEventSource` with a capacity check. |

#### `trait EventSource`
- Ort: `crates/contextra-agent/src/event_source.rs:99`
```rust
pub trait EventSource: Send + Sync
```
Dokumentation: Dynamic event source delivering continuous background events.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn next_event<'a>(&'a mut self) -> BoxFuture<'a, Result<Option<BackgroundEvent>>>` | `crates/contextra-agent/src/event_source.rs:102` | Fetches the next available background event. Returns `Ok(Some(event))` when an event is ready, `Ok(None)` if no event is currently pending. |
| `fn is_exhausted(&self) -> bool` | `crates/contextra-agent/src/event_source.rs:106` | Indicates whether the event source is permanently exhausted. Always returns `false` for continuous/always-on producers. |
| `fn wait_for_event<'a>(&'a self) -> BoxFuture<'a, ()>` | `crates/contextra-agent/src/event_source.rs:114` | Waits until a new event is likely available or the timeout expires. Default implementation: short sleep fallback for polling-based sources. Implementations with |

#### `const MAX_EVENT_SOURCE_CAPACITY`
- Ort: `crates/contextra-agent/src/event_source.rs:28`
```rust
pub const MAX_EVENT_SOURCE_CAPACITY: usize = 10_000
```
Dokumentation: Maximum allowed events in pending event buffers to prevent memory exhaustion.

#### `const MAX_PENDING_EVENTS_CAPACITY`
- Ort: `crates/contextra-agent/src/event_source.rs:122`
```rust
pub const MAX_PENDING_EVENTS_CAPACITY: usize = 10_000
```
Dokumentation: Maximum capacity for pending background telemetry events queue before dropping or rejecting.

### Modul `goal_condition`

#### `enum GoalCondition`
- Ort: `crates/contextra-agent/src/goal_condition.rs:27`
```rust
pub enum GoalCondition
```
Dokumentation: Repräsentiert eine strukturierte Bedingung zur Auswertung, ob ein Teilziel eines Agentenschritts erfüllt ist.
- Impl-Block `crates/contextra-agent/src/goal_condition.rs:64`

Varianten:

- `OutputFieldEquals{` — `crates/contextra-agent/src/goal_condition.rs:29`
- `OutputFieldContains{` — `crates/contextra-agent/src/goal_condition.rs:36`
- `TokensConsumedAtLeast{` — `crates/contextra-agent/src/goal_condition.rs:43`
- `NextEdgeEquals{` — `crates/contextra-agent/src/goal_condition.rs:48`
- `All(Vec<GoalCondition>)` — `crates/contextra-agent/src/goal_condition.rs:55`
- `Any(Vec<GoalCondition>)` — `crates/contextra-agent/src/goal_condition.rs:59`
- `Not(Box<GoalCondition>)` — `crates/contextra-agent/src/goal_condition.rs:61`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn evaluate(&self, result: &StepResult) -> bool` | `crates/contextra-agent/src/goal_condition.rs:68` | Wertet die Bedingung deterministisch gegen ein gegebenes [`StepResult`] aus. Liefert bei fehlendem oder ungültigem JSON-Pfad stets `false`. Panict niemals. |
| `pub fn describe(&self) -> String` | `crates/contextra-agent/src/goal_condition.rs:107` | Liefert eine menschenlesbare Beschreibung der Bedingung (z. B. für Audit-Logging). |

#### `fn parse_legacy_condition_string`
- Ort: `crates/contextra-agent/src/goal_condition.rs:173`
```rust
pub fn parse_legacy_condition_string(raw: &str) -> Option<GoalCondition>
```
Dokumentation: Bildet einen freien Bedingungs-String aus Altbestand auf eine strukturierte [`GoalCondition`] ab. Unterstützt Gleichheitsprüfungen ("field == value", "next_edge == edge_name"), Substring-Suchen ("field contains substring") und Token-Schwellenwerte ("tokens_consumed >= N"). Bei nicht parsbarer Syntax wird `None` zurückgegeben.

### Modul `graph`

#### `struct AgentNode`
- Ort: `crates/contextra-agent/src/graph.rs:44`
```rust
pub struct AgentNode
```
Dokumentation: Represents a single specialized node in the workflow.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `id` | `NodeId` | `crates/contextra-agent/src/graph.rs:46` |
| `description` | `String` | `crates/contextra-agent/src/graph.rs:48` |
| `node_type` | `NodeType` | `crates/contextra-agent/src/graph.rs:50` |
| `handler` | `Option<String>` | `crates/contextra-agent/src/graph.rs:52` |

#### `struct StateGraph`
- Ort: `crates/contextra-agent/src/graph.rs:119`
```rust
pub struct StateGraph
```
Dokumentation: Core declarative structure routing autonomous agent steps.
- Impl-Block `crates/contextra-agent/src/graph.rs:126`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `edges` | `Vec<WorkflowEdge>` | `crates/contextra-agent/src/graph.rs:123` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-agent/src/graph.rs:128` | Creates a new empty [`StateGraph`]. |
| `pub fn try_add_node( &mut self, id: &str, description: &str, node_type: NodeType, handler: Option<&str>, ) -> Result<()>` | `crates/contextra-agent/src/graph.rs:134` |  |
| `pub fn add_node( &mut self, id: &str, description: &str, node_type: NodeType, handler: Option<&str>, )` | `crates/contextra-agent/src/graph.rs:195` | Adds a node to the graph, panicking if validation fails. |
| `pub fn try_add_edge( &mut self, from: &str, to: &str, condition: Option<&str>, priority: u8, ) -> Result<()>` | `crates/contextra-agent/src/graph.rs:209` |  |
| `pub fn try_add_edge_with_goal( &mut self, from: &str, to: &str, goal: GoalCondition, priority: u8, ) -> Result<()>` | `crates/contextra-agent/src/graph.rs:247` | Tries to insert a new edge with an explicit [`GoalCondition`]. |
| `pub fn add_edge(&mut self, from: &str, to: &str, condition: Option<&str>, priority: u8)` | `crates/contextra-agent/src/graph.rs:269` | Adds an edge to the graph, panicking if validation fails. |
| `pub fn get_node(&self, id: &str) -> Option<&AgentNode>` | `crates/contextra-agent/src/graph.rs:276` | Looks up a node by its identifier. |

#### `struct WorkflowEdge`
- Ort: `crates/contextra-agent/src/graph.rs:57`
```rust
pub struct WorkflowEdge
```
Dokumentation: Represents a conditional transition between two graph nodes.
- Impl-Block `crates/contextra-agent/src/graph.rs:71`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `from` | `NodeId` | `crates/contextra-agent/src/graph.rs:59` |
| `to` | `NodeId` | `crates/contextra-agent/src/graph.rs:61` |
| `condition` | `Option<String>` | `crates/contextra-agent/src/graph.rs:63` |
| `priority` | `u8` | `crates/contextra-agent/src/graph.rs:65` |
| `goal` | `Option<GoalCondition>` | `crates/contextra-agent/src/graph.rs:68` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn has_condition(&self) -> bool` | `crates/contextra-agent/src/graph.rs:73` | Returns whether this edge has an explicit condition or goal attached. |
| `pub fn evaluate_condition(&self, result: &StepResult) -> bool` | `crates/contextra-agent/src/graph.rs:91` | Evaluates whether this edge's condition/goal is satisfied against a [`StepResult`]. # Evaluation Order & Fail-Closed Semantics: 1. If `goal` is present, it is e |

#### `enum NodeType`
- Ort: `crates/contextra-agent/src/graph.rs:31`
```rust
pub enum NodeType
```
Dokumentation: Type of node within the declarative agent state graph.

Varianten:

- `Start` — `crates/contextra-agent/src/graph.rs:33`
- `Task` — `crates/contextra-agent/src/graph.rs:35`
- `Decision` — `crates/contextra-agent/src/graph.rs:37`
- `End` — `crates/contextra-agent/src/graph.rs:39`

#### `type NodeId`
- Ort: `crates/contextra-agent/src/graph.rs:24`
```rust
pub type NodeId = String
```
Dokumentation: Type alias for node identifiers in the state graph.

#### `const MAX_TEXT_LEN`
- Ort: `crates/contextra-agent/src/graph.rs:27`
```rust
pub const MAX_TEXT_LEN: usize = 65_536
```
Dokumentation: Maximum allowed length in bytes for descriptions and conditions.

### Modul `ports`

#### `struct InMemoryScratchpadStore`
- Ort: `crates/contextra-agent/src/ports.rs:48`
```rust
pub struct InMemoryScratchpadStore
```
Dokumentation: In-Memory-Referenzimplementierung von [`ScratchpadStore`] für Tests und entkoppelte Ausführung.
- Impl-Block `crates/contextra-agent/src/ports.rs:52`
- Impl-Block (Trait `ScratchpadStore`) `crates/contextra-agent/src/ports.rs:59`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-agent/src/ports.rs:54` | Erzeugt einen neuen leeren `InMemoryScratchpadStore`. |

#### `trait ContextStore`
- Ort: `crates/contextra-agent/src/ports.rs:19`
```rust
pub trait ContextStore: Send + Sync
```
Dokumentation: Abstraktion über persistente Agenten-Zustände und Retrieval-Quellen.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn put_state<'a>(&'a self, key: &'a str, value: &'a [u8]) -> BoxFuture<'a, Result<()>>` | `crates/contextra-agent/src/ports.rs:21` | Speichert einen Key-Value-Zustand. |
| `fn get_state<'a>(&'a self, key: &'a str) -> BoxFuture<'a, Result<Option<Vec<u8>>>>` | `crates/contextra-agent/src/ports.rs:23` | Liest einen Key-Value-Zustand. |
| `fn delete_state<'a>(&'a self, key: &'a str) -> BoxFuture<'a, Result<bool>>` | `crates/contextra-agent/src/ports.rs:25` | Löscht einen Key-Value-Zustand. |

#### `trait ScratchpadStore`
- Ort: `crates/contextra-agent/src/ports.rs:36`
```rust
pub trait ScratchpadStore: Send + Sync
```
Dokumentation: Abstraktion für ephemere Scratchpad-Puffer im RAM.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn append_chunk(&mut self, id: DocId, content: Vec<u8>) -> Result<()>` | `crates/contextra-agent/src/ports.rs:38` | Fügt dem flüchtigen Speicher einen Chunk hinzu. |
| `fn purge(&mut self)` | `crates/contextra-agent/src/ports.rs:40` | Leert den flüchtigen Speicher und überschreibt ihn via Zeroize. |
| `fn total_bytes(&self) -> usize` | `crates/contextra-agent/src/ports.rs:42` | Liefert die aktuell belegten Bytes im RAM. |

### Modul `step`

#### `struct StepDeadLetter`
- Ort: `crates/contextra-agent/src/step.rs:26`
```rust
pub struct StepDeadLetter
```
Dokumentation: Ein fehlgeschlagener Agent-Schritt der für spätere Analyse und Idempotenz-Prüfung persistiert wird.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `session_id` | `String` | `crates/contextra-agent/src/step.rs:28` |
| `node_id` | `String` | `crates/contextra-agent/src/step.rs:30` |
| `step_index` | `u64` | `crates/contextra-agent/src/step.rs:33` |
| `tx_id` | `Option<TxId>` | `crates/contextra-agent/src/step.rs:36` |
| `failure_reason` | `DeadLetterReason` | `crates/contextra-agent/src/step.rs:38` |
| `input` | `serde_json::Value` | `crates/contextra-agent/src/step.rs:40` |
| `attempt` | `u32` | `crates/contextra-agent/src/step.rs:42` |
| `failed_at_secs` | `u64` | `crates/contextra-agent/src/step.rs:44` |

#### `struct StepResult`
- Ort: `crates/contextra-agent/src/step.rs:76`
```rust
pub struct StepResult
```
Dokumentation: The explicit result of an agent step execution.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `node_id` | `String` | `crates/contextra-agent/src/step.rs:78` |
| `output` | `serde_json::Value` | `crates/contextra-agent/src/step.rs:80` |
| `tokens_consumed` | `usize` | `crates/contextra-agent/src/step.rs:82` |
| `next_edge` | `Option<String>` | `crates/contextra-agent/src/step.rs:84` |

#### `enum DeadLetterReason`
- Ort: `crates/contextra-agent/src/step.rs:49`
```rust
pub enum DeadLetterReason
```
Dokumentation: Specific failure reason category recorded when an agent step is moved to the Dead-Letter Queue.

Varianten:

- `Timeout{` — `crates/contextra-agent/src/step.rs:51`
- `BudgetExhausted{` — `crates/contextra-agent/src/step.rs:56`
- `ToolError{` — `crates/contextra-agent/src/step.rs:63`
- `MaxRetriesExceeded{` — `crates/contextra-agent/src/step.rs:68`

#### `trait AgentTool`
- Ort: `crates/contextra-agent/src/step.rs:88`
```rust
pub trait AgentTool: Send + Sync
```
Dokumentation: Core interface implemented by all executable tools participating in agent workflows.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn name(&self) -> &str` | `crates/contextra-agent/src/step.rs:90` | Returns the unique name identifier of this tool. |
| `fn estimated_cost(&self, _input: &serde_json::Value) -> usize` | `crates/contextra-agent/src/step.rs:95` | Returns the estimated token cost of executing this tool with the given input. Used for strict pre-execution budget validation to prevent side-effects when budge |
| `fn execute<'a>( &'a self, ctx: &'a AgentContext, input: serde_json::Value, ) -> BoxFuture<'a, Result<StepResult>>` | `crates/contextra-agent/src/step.rs:100` | Executes the tool asynchronously within the given [`AgentContext`]. |
| `fn timeout_ms(&self) -> u64` | `crates/contextra-agent/src/step.rs:108` | Maximale Ausführungszeit dieses Tools in Millisekunden. Standard: 30 Sekunden. Überschreibe für langläufige Tools. |
| `fn is_retriable(&self) -> bool` | `crates/contextra-agent/src/step.rs:113` | Ob dieser Schritt bei Timeout/transientem Fehler wiederholbar ist. |
| `fn max_retries(&self) -> u32` | `crates/contextra-agent/src/step.rs:118` | Maximale Anzahl Retries. Standard: 2 (d.h. insgesamt 3 Versuche). |

### Modul `tools`

#### `mod manifest`
- Ort: `crates/contextra-agent/src/tools/mod.rs:14`
```rust
pub mod manifest
```

#### `mod registry`
- Ort: `crates/contextra-agent/src/tools/mod.rs:15`
```rust
pub mod registry
```

#### `mod retrieve`
- Ort: `crates/contextra-agent/src/tools/mod.rs:16`
```rust
pub mod retrieve
```

### Modul `tools::manifest`

#### `struct ToolManifest`
- Ort: `crates/contextra-agent/src/tools/manifest.rs:35`
```rust
pub struct ToolManifest
```
Dokumentation: Declarative manifest specifying operational properties, capability requirements, and retry policies for a tool.
- Impl-Block `crates/contextra-agent/src/tools/manifest.rs:54`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `name` | `String` | `crates/contextra-agent/src/tools/manifest.rs:37` |
| `description` | `String` | `crates/contextra-agent/src/tools/manifest.rs:39` |
| `required_capability` | `Option<String>` | `crates/contextra-agent/src/tools/manifest.rs:42` |
| `timeout_ms` | `u64` | `crates/contextra-agent/src/tools/manifest.rs:45` |
| `is_retriable` | `bool` | `crates/contextra-agent/src/tools/manifest.rs:48` |
| `max_retries` | `u32` | `crates/contextra-agent/src/tools/manifest.rs:51` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self` | `crates/contextra-agent/src/tools/manifest.rs:56` | Creates a new `ToolManifest` with the given name and description. |
| `pub fn with_capability(mut self, capability: impl Into<String>) -> Self` | `crates/contextra-agent/src/tools/manifest.rs:68` | Sets the required capability for execution. |
| `pub fn with_timeout_ms(mut self, timeout_ms: u64) -> Self` | `crates/contextra-agent/src/tools/manifest.rs:74` | Sets the timeout in milliseconds. |
| `pub fn with_retry(mut self, is_retriable: bool, max_retries: u32) -> Self` | `crates/contextra-agent/src/tools/manifest.rs:80` | Sets retry behavior. |
| `pub fn validate(&self) -> Result<()>` | `crates/contextra-agent/src/tools/manifest.rs:87` | Validates the tool manifest fields against boundary rules. |

#### `const MAX_MANIFEST_DESC_LEN`
- Ort: `crates/contextra-agent/src/tools/manifest.rs:19`
```rust
pub const MAX_MANIFEST_DESC_LEN: usize = 4096
```
Dokumentation: Maximum text length for manifest description.

### Modul `tools::registry`

#### `struct ToolRegistry`
- Ort: `crates/contextra-agent/src/tools/registry.rs:22`
```rust
pub struct ToolRegistry
```
Dokumentation: Registry managing validated tool manifests and execution dispatch for agent steps.
- Impl-Block `crates/contextra-agent/src/tools/registry.rs:26`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-agent/src/tools/registry.rs:28` | Creates a new empty `ToolRegistry`. |
| `pub fn register(&mut self, manifest: ToolManifest, tool: Box<dyn AgentTool>) -> Result<()>` | `crates/contextra-agent/src/tools/registry.rs:33` | Registers an `AgentTool` with its validated `ToolManifest`. |
| `pub fn register_tool(&mut self, tool: Box<dyn AgentTool>) -> Result<()>` | `crates/contextra-agent/src/tools/registry.rs:47` | Convenience helper to register a tool using default manifest settings derived from `AgentTool`. |
| `pub fn get_tool(&self, name: &str) -> Option<&dyn AgentTool>` | `crates/contextra-agent/src/tools/registry.rs:56` | Retrieves a reference to a registered tool by name. |
| `pub fn get_manifest(&self, name: &str) -> Option<&ToolManifest>` | `crates/contextra-agent/src/tools/registry.rs:61` | Retrieves a reference to a registered tool's manifest by name. |
| `pub fn authorize(&self, ctx: &AgentContext, tool_name: &str) -> Result<&ToolManifest>` | `crates/contextra-agent/src/tools/registry.rs:68` | Checks pre-execution permissions for a tool (fail-closed). Returns `Ok(&ToolManifest)` if the context is authorized, or `Err(ContextraError::PermissionDenied)`  |
| `pub fn contains(&self, name: &str) -> bool` | `crates/contextra-agent/src/tools/registry.rs:100` | Returns whether a tool with the given name is registered. |
| `pub fn len(&self) -> usize` | `crates/contextra-agent/src/tools/registry.rs:105` | Returns the number of registered tools. |
| `pub fn is_empty(&self) -> bool` | `crates/contextra-agent/src/tools/registry.rs:110` | Returns `true` if no tools are registered. |

### Modul `tools::retrieve`

#### `struct RetrieveTool`
- Ort: `crates/contextra-agent/src/tools/retrieve.rs:35`
```rust
pub struct RetrieveTool
```
Dokumentation: Read-only tenant context retrieval tool implementing [`AgentTool`]. Searches documents within the tenant's collection context without mutating state.
- Impl-Block (Trait `std::fmt::Debug`) `crates/contextra-agent/src/tools/retrieve.rs:40`
- Impl-Block (Trait `Default`) `crates/contextra-agent/src/tools/retrieve.rs:49`
- Impl-Block `crates/contextra-agent/src/tools/retrieve.rs:58`
- Impl-Block (Trait `AgentTool`) `crates/contextra-agent/src/tools/retrieve.rs:81`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-agent/src/tools/retrieve.rs:60` | Creates a new `RetrieveTool` instance. |
| `pub fn with_name(name: impl Into<String>) -> Self` | `crates/contextra-agent/src/tools/retrieve.rs:65` | Creates a new `RetrieveTool` with a custom handler name. |
| `pub fn with_retriever(retriever: Arc<dyn ContextRetriever>) -> Self` | `crates/contextra-agent/src/tools/retrieve.rs:73` | Create a `RetrieveTool` with a custom retriever for decoupled execution. |

#### `trait ContextRetriever`
- Ort: `crates/contextra-agent/src/tools/retrieve.rs:22`
```rust
pub trait ContextRetriever: Send + Sync
```
Dokumentation: Abstrakter ContextRetriever-Port, um Retrieval unabhängig von konkreten DB-Instanzen auszuführen.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn scan_prefix<'a>( &'a self, prefix: &'a str, limit: Option<usize>, ) -> BoxFuture<'a, Result<Vec<(String, serde_json::Value)>>>` | `crates/contextra-agent/src/tools/retrieve.rs:24` | Sucht Dokumente/Kontexte asynchron anhand eines Präfixes und Limits. |

## 9. Impl-Blöcke ohne Typ im selben Crate

- `impl<T: CheckpointRegistry + Checkpoint + Send + Sync> AgentCheckpointStore for T` — `crates/contextra-agent/src/engine.rs:50` (0 pub Methoden)
- `impl From<AgentError> for contextra_types::ContextraError` — `crates/contextra-agent/src/error.rs:54` (0 pub Methoden)
