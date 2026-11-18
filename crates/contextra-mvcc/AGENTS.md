# AGENTS.md — contextra-mvcc — Snapshot-Isolation & Concurrency

**Status:** Normativ  
**Crate-Spezifikation:** [`docs/crates/contextra-mvcc.md`](file:///home/freddy/Projekte/contextra/docs/crates/contextra-mvcc.md)  
**Gesamtspezifikation:** [`docs/SPEC.md`](file:///home/freddy/Projekte/contextra/docs/SPEC.md) — Bindung: docs/SPEC.md §2 (Architektur), §5 (Löschkette), §9 (Zeitreise)  
**Workspace-Richtlinie:** [`AGENTS.md`](file:///home/freddy/Projekte/contextra/AGENTS.md) (Wurzel)  
**Ebene:** E1 (Autonomer Domänenbaustein)  
**Kern-Closure:** ids, env (2 Mitglieder)  
**Unsafe-Status:** Verboten (`#![forbid(unsafe_code)]`)  

---

## 1. Zweck & Systemrolle

Multi-Version Concurrency Control (MVCC): Snapshot-Isolation, Pin-Verwaltung und Snapshot-Widerruf.

---

## 2. Kern-Richtlinien für Agenten

- **Snapshot-Isolation:** Verwaltet `SnapshotPin`s für nebenläufige Leser ohne Writer-Blockierung. Bestimmt `min_active_seqno` für Kompaktionen.
- **Rolle in der Löschkette (Snapshot-Widerruf 4a):**
  - Implementiert `SnapshotRevoker::revoke_covering`.
  - Kein aktiver Snapshot darf eine DSGVO-Löschung unbegrenzt blockieren!
  - Pins mit `seq <= gated_seq`, die das Ziel abdecken, werden zwangsweise invalidiert. Nachfolgende Leseversuche scheitern mit `Err(ContextraError::SnapshotRevoked)`.
- **RAII-Garantie:** `SnapshotPin` deregistriert sich im `Drop` zuverlässig aus der Registry.

---

<!-- BEGIN GENERATED: crate_inventory.py -->

## Inventar (generiert)

- **Pfad:** `crates/contextra-mvcc` · **Ring:** 0 · **Version:** 0.1.0
- **Beschreibung:** Multi-Version Concurrency Control (MVCC), sequence log, and transaction buffer for Contextra
- **src:** 10 Dateien, 3305 LOC · **unsafe:** 0 Vorkommen, Policy `forbid`
- **Interne Abhängigkeiten (normal, 4):** `contextra-env`, `contextra-ids`, `contextra-ports`, `contextra-types`
- **Interne Dev/Build-Abhängigkeiten (3):** `contextra-env`, `contextra-ports`, `contextra-types`
- **Externe Abhängigkeiten (normal, 7):** `ahash`, `bytes`, `loom`, `parking_lot`, `serde`, `thiserror`, `tracing`
- **Reverse-Abhängigkeiten (3):** `contextra-db`, `contextra-store`, `contextra-vector`
- **Kopplung an Hub-Crates:** ausgehend `contextra-ports`, `contextra-types`; eingehend `contextra-db`
- **Features (5):** `default`, `docid-128`, `ports-compat`, `types-compat`, `workspace`

### Öffentliche API (grob, aus src/)
- struct (11): `PruningBlockerInfo`, `ReadSet`, `SeqBitTombstone`, `SeqLogEntry`, `SequenceLog`, `SequenceLogSsiValidator`, `SnapshotGuard`, `SnapshotRegistry`, `SnapshotRegistryConfig`, `TxBuffer`, `TxBufferConfig`
- enum (3): `IndexOp`, `MvccError`, `SeqLogChange`
- trait (3): `SsiValidator`, `StagedOpSize`, `TombstoneSemanticsCheck`
- fn (111): `begin`, `begin_at`, `changes_since`, `clear`, `clock`, `compact`, `config`, `deletion_seq`, `diagnose_pruning_blocker`, `diagnose_pruning_blocker_at`, `discard`, `discard_kv`, `doc_id`, `drain`, `drain_kv` … (+96)
- type (1): `Result`
- const (8): `DEFAULT_MAX_OPS_PER_TX`, `DEFAULT_MAX_PIN_DURATION`, `DEFAULT_MAX_READ_SET_KEYS`, `DEFAULT_MAX_TOTAL_STAGED_BYTES`, `DEFAULT_MAX_TRACKED_COMMIT_KEYS`, `DEFAULT_MAX_TX_STAGED_BYTES`, `DEFAULT_SHARD_COUNT`, `STAGING_ENTRY_OVERHEAD_BYTES`
- mod (9): `compat`, `error`, `mvcc_error`, `seq_log`, `snapshot`, `ssi`, `tombstone`, `tx_buffer`, `types`
- pub use: `contextra_env::{Clock, MetricsSink, SystemClock}`, `contextra_ids::{DocId, TenantId, TxId, TOMBSTONE_BIT}`, `contextra_types::error as workspace_error`, `contextra_types::error::ContextraError`, `contextra_types::error::Result`, `crate::compat::{workspace_error as error, ContextraError, Result}`, `crate::mvcc_error::Result`, `crate::mvcc_error::{MvccError, Result as MvccResult}`, `seq_log::{SeqLogChange, SeqLogEntry, SequenceLog}`, `snapshot::{SnapshotGuard, SnapshotRegistry}` … (+3)

Regeneriert mit `python3 tools/crate_inventory.py agents`. Manuelle Notizen stehen außerhalb dieses Blocks.

<!-- END GENERATED: crate_inventory.py -->
