# AGENTS.md — contextra-store — LSM-Storage Engine

**Status:** Normativ  
**Crate-Spezifikation:** [`docs/crates/contextra-store.md`](file:///home/freddy/Projekte/contextra/docs/crates/contextra-store.md)  
**Gesamtspezifikation:** [`docs/SPEC.md`](file:///home/freddy/Projekte/contextra/docs/SPEC.md) — Bindung: docs/SPEC.md §2 (Architektur), §3 (Datenklassifikation), §4 (Schlüsselhierarchie), §5 (Löschkette), §6 (ReadGate), §7 (ErasureLayer)  
**Workspace-Richtlinie:** [`AGENTS.md`](file:///home/freddy/Projekte/contextra/AGENTS.md) (Wurzel)  
**Ebene:** E1 (Autonomer Domänenbaustein)  
**Kern-Closure:** ids, env, sys, durable-fs, crypto (5 Mitglieder)  
**Unsafe-Status:** Verboten (`#![forbid(unsafe_code)]`)  

---

## 1. Zweck & Systemrolle

ACID-konforme LSM-Tree Persistenz-Engine: Aktive/Immutable MemTables, SSTables (L0..LN), WAL.

---

## 2. Kern-Richtlinien für Agenten

- **Verschlüsselung auf Datenträger (T-DEL-1):** Niemals Klartext auf Platte. Payloads werden vor dem Einreihen in MemTable/WAL über `RecordSealer` mit AES-256-GCM-SIV versiegelt. AAD bindet `tenant_id || collection_id || doc_id || version`.
- **ReadGate-Integration:** In `get()`, `scan()`, `scan_prefix()` MUSS vor jedem Plattenzugriff und vor der Entschlüsselung synchron `read_gate.check(key)` aufgerufen werden. Bei `Erased` -> `Ok(None)` (Fail-Closed).
- **ErasureLayer-Implementierung:**
  - `resolve`: Identifiziert betroffene SSTables und WAL-Segmente.
  - `on_gate`: Registriert Schlüssel im In-Memory WriteFence.
  - `purge`: Führt Sub-Schritt 4c (WAL-Drain) und 4d (SSTable-Rewrite ohne gelöschte Keys) unter `PurgeBarrier` aus; überschreibt alte SSTables physisch mit Nullen (`shred_file`).
  - `witness`: Scannt verbliebene SSTables auf Residuen (`residue_matches == 0`).

---

<!-- BEGIN GENERATED: crate_inventory.py -->

## Inventar (generiert)

- **Pfad:** `crates/contextra-store` · **Ring:** 1 · **Version:** 0.1.0
- **Beschreibung:** LSM-Tree storage engine for Contextra
- **src:** 161 Dateien, 25624 LOC · **unsafe:** 0 Vorkommen, Policy `forbid`
- **Interne Abhängigkeiten (normal, 7):** `contextra-crypto`, `contextra-env`, `contextra-ids`, `contextra-mvcc`, `contextra-ports`, `contextra-sys`, `contextra-types`
- **Interne Dev/Build-Abhängigkeiten (2):** `contextra-env`, `contextra-testkit`
- **Externe Abhängigkeiten (normal, 15):** `ahash`, `blake3`, `bytes`, `crc32fast`, `lru`, `memmap2`, `parking_lot`, `quick_cache`, `rand`, `serde`, `thiserror`, `tokio`, `tokio-util`, `tracing`, `uuid`
- **Reverse-Abhängigkeiten (2):** `contextra-bench`, `contextra-db`
- **Kopplung an Hub-Crates:** ausgehend `contextra-ports`, `contextra-types`; eingehend `contextra-db`
- **Features (10):** `block-cache-v2`, `default`, `docid-128`, `fault-injection`, `inspect`, `legacy-wal-key`, `memory-only-storage`, `sieve-cache`, `test-hooks`, `workspace`

### Öffentliche API (grob, aus src/)
- struct (62): `AdaptiveCompactionPlan`, `AsyncObserverAdapter`, `BlockBuilder`, `BlockCache`, `BloomFilter`, `CommitContext`, `CommitGuard`, `CommittedBatch`, `CompactionConfig`, `CompactionEngine`, `CostBasedAdaptivePlanner`, `FlushPurgeFence`, `InMemoryStorageEngine`, `LoomFile`, `LoomMetadata` … (+47)
- enum (14): `CompactionStrategy`, `DurabilityConfigError`, `DurabilityMode`, `LegacyKeyStatus`, `ManifestEntry`, `PoisonCause`, `PressureLevel`, `StorageHealth`, `StoreError`, `VerifiedVia`, `WalOp`, `WalParseError`, `WalVersion`, `WriteOrigin`
- trait (14): `AdaptiveCompactionPlanner`, `BlockCacheBackend`, `KeyMaterialSource`, `MergeOperator`, `RecordAuthenticator`, `RecordSealer`, `ReplayProgressSink`, `StorageEngineHandle`, `StoreFile`, `StoreFs`, `SystemPressureSignal`, `Vfs`, `VfsFile`, `WalObserver`
- fn (320): `active_memtable`, `add`, `all_memtables`, `allow_legacy_fallback_for_test`, `append`, `append_batch`, `append_batch_locked`, `arm_truncate_failure_for_test`, `binary_search_entry_in_block`, `binary_search_first_index_in_block`, `binary_search_index_in_block`, `block_binary_search`, `block_cache`, `block_cache_for_test`, `block_on` … (+305)
- type (8): `BlockCacheShard`, `File`, `MemTableEntry`, `MemTableMap`, `SequenceNumber`, `StoreResult`, `TransactionId`, `WalSeq`
- const (47): `BLOCKING_UTIL_CRITICAL`, `BLOCKING_UTIL_ELEVATED`, `BLOCK_CACHE_SHARDS`, `BLOCK_SIZE`, `CURRENT_MANIFEST_VERSION`, `CURRENT_SSTABLE_FORMAT_VERSION`, `DEFAULT_MAX_OBSERVER_LATENCY`, `DEFAULT_ROLLOVER_THRESHOLD_BYTES`, `DEFAULT_WAL_QUEUE_CAPACITY`, `INTENT_MAGIC`, `LEGACY_TRAILER_LEN`, `MANIFEST_HEADER_MAGIC`, `MANIFEST_VERSION_V1`, `MANIFEST_VERSION_V2`, `MAX_BATCH_SIZE` … (+32)
- static (7): `DELAY_APPEND_FOR_TX`, `DELAY_APPEND_MS`, `FAIL_APPEND_AFTER_PARTIAL_BYTES`, `FAIL_APPEND_FOR_TX`, `FAIL_APPEND_PARTIAL_ONCE`, `FAIL_SYNC_ONCE`, `FAIL_TRUNCATE_ONCE`
- mod (74): `acl`, `adaptive`, `append`, `block_cache`, `bounded`, `budget`, `calculation`, `commit`, `compaction`, `compat`, `config`, `core`, `cost_based`, `encode`, `engine` … (+59)
- pub use: `adapter::AsyncObserverAdapter`, `adaptive::*`, `block::BlockBuilder`, `block_cache::*`, `block_search::*`, `bloom::*`, `builder::*`, `calculation::*`, `compaction::{ adaptive::{ AdaptiveCompactionPlan, AdaptiveCompactionPlanner, CompactionStrategy, CostBasedAdaptivePlanner, WorkloadMetrics, WorkloadMetricsSnapshot, }, CompactionConfig, CompactionEngine, MergeOperator, TtlMetadata, }`, `config::*` … (+67)

Regeneriert mit `python3 tools/crate_inventory.py agents`. Manuelle Notizen stehen außerhalb dieses Blocks.

<!-- END GENERATED: crate_inventory.py -->
