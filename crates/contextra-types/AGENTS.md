# AGENTS.md — contextra-types — Gemeinsame Domänentypen

**Status:** Normativ  
**Crate-Spezifikation:** [`docs/crates/contextra-types.md`](file:///home/freddy/Projekte/contextra/docs/crates/contextra-types.md)  
**Gesamtspezifikation:** [`docs/SPEC.md`](file:///home/freddy/Projekte/contextra/docs/SPEC.md) — Bindung: docs/SPEC.md §2 (Architektur), §3 (Datenklassifikation), §5 (Löschkette), §11 (Fehlerverhalten)  
**Workspace-Richtlinie:** [`AGENTS.md`](file:///home/freddy/Projekte/contextra/AGENTS.md) (Wurzel)  
**Ebene:** E2 (Integration)  
**Kern-Closure:** Darf E0, E1, E2 nutzen, nie E3  
**Unsafe-Status:** Verboten (`#![forbid(unsafe_code)]`)  

---

## 1. Zweck & Systemrolle

Gemeinsame kanonische Datentypen, Enums und Fehlertypen (`ContextraError`).

---

## 2. Kern-Richtlinien für Agenten

- **Kanonischer Fehlertyp (R4):** `ContextraError` ist der zentrale Systemfehler. Spezifische Varianten für `Erased`, `Restricted`, `RestrictedReadAuditRequired`, `ClassViolation`, `Crypto`, `SnapshotRevoked`.
- **Typen der Löschkette:** Definiert `EraseTarget`, `SagaState`, `RetentionAttestation`, `RetentionCode`, `OperatorRef`.
- **Reines Datenmodell:** Keine Persistenz- oder Krypto-Logik; strikte Serialisierbarkeit via `serde`.

---

<!-- BEGIN GENERATED: crate_inventory.py -->

## Inventar (generiert)

- **Pfad:** `crates/contextra-types` · **Ring:** 0 · **Version:** 0.1.0
- **Beschreibung:** Canonical domain types, IDs, budgets, filters, and error types for Contextra
- **src:** 25 Dateien, 5281 LOC · **unsafe:** 0 Vorkommen, Policy `forbid`
- **Interne Abhängigkeiten (normal, 1):** `contextra-ids`
- **Interne Dev/Build-Abhängigkeiten (0):** –
- **Externe Abhängigkeiten (normal, 8):** `ahash`, `bincode`, `blake3`, `bytes`, `serde`, `serde_json`, `sha2`, `thiserror`
- **Reverse-Abhängigkeiten (16):** `contextra`, `contextra-bench`, `contextra-checkpoint`, `contextra-crypto`, `contextra-db`, `contextra-graph`, `contextra-mcp`, `contextra-mvcc`, `contextra-ports`, `contextra-rank`, `contextra-simd`, `contextra-store`, `contextra-testkit`, `contextra-text`, `contextra-vault`, `contextra-vector`
- **Kopplung an Hub-Crates:** ausgehend –; eingehend `contextra`, `contextra-db`, `contextra-ports`
- **Features (3):** `auto-extraction-opt-out`, `default`, `docid-128`

### Öffentliche API (grob, aus src/)
- struct (42): `Armed`, `ConfigFingerprint`, `ContextChunk`, `ContextWindow`, `ContextraErrorDto`, `ControlEvidence`, `Edge`, `Embedding`, `Entity`, `EraseReceipt`, `EraseRequest`, `ErasureId`, `FusionWeights`, `GateId`, `HybridQuery` … (+27)
- enum (32): `AutoExtractionMode`, `BudgetStrategy`, `CacheDirective`, `ContextraError`, `DecayFunction`, `DistanceMetric`, `DocIdWidth`, `EraseMode`, `EraseTarget`, `ErasureStatus`, `FilterExpr`, `FusionStrategy`, `GateVariant`, `GraphTraversalStrategy`, `HnswDeletionError` … (+17)
- trait (1): `TombstoneSemanticsCheck`
- fn (108): `arm`, `as_bytes`, `as_metadata_key`, `as_slice`, `as_u8`, `available`, `budget`, `build`, `builder`, `bytes`, `bytes_len`, `capability_unsupported`, `checksum_mismatch`, `class`, `combined_text_owned` … (+93)
- type (3): `QueryOptions`, `QueryOptionsBuilder`, `Result`
- const (2): `EXPIRY_METADATA_KEY`, `MAX_SEARCH_K`
- mod (17): `budget`, `domain`, `erasure`, `error`, `error_dto`, `filter`, `ids`, `importance`, `model_fingerprint`, `refs`, `retrieval_strategy`, `saos`, `schema`, `tenant_scope`, `tombstone` … (+2)
- pub use: `budget::*`, `contextra_ids::*`, `contextra_ids::TOMBSTONE_BIT`, `contextra_ids::{ CollectionRef, DocKey, DocRef, OperatorRef, SubjectRef, TenantRef, TenantScoped, MAX_REF_LEN, }`, `contextra_ids::{CollectionId, DocId, EntityId, IdError, TenantId, TxId, ID_WIDTH_BITS}`, `contextra_ids::{TenantScopeViolation, TenantScoped}`, `document::*`, `domain::*`, `erasure::*`, `error::{ContextraError, HnswDeletionError, Result}` … (+15)

Regeneriert mit `python3 tools/crate_inventory.py agents`. Manuelle Notizen stehen außerhalb dieses Blocks.

<!-- END GENERATED: crate_inventory.py -->
