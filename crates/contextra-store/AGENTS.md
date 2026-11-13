# AGENTS.md — `contextra-store`

> Werkzeug/Crate-Kontext für Agenten. Der obere Teil ist generiert; manuelle Notizen unterhalb ergänzen.

<!-- BEGIN GENERATED: crate_inventory.py -->

## Inventar (generiert)

- **Pfad:** `crates/contextra-store` · **Ring:** 1 · **Version:** 0.1.0
- **Beschreibung:** LSM-Tree storage engine for Contextra
- **src:** 161 Dateien, 25508 LOC · **unsafe:** 0 Vorkommen, Policy `forbid`
- **Interne Abhängigkeiten (normal, 7):** `contextra-crypto`, `contextra-env`, `contextra-ids`, `contextra-mvcc`, `contextra-ports`, `contextra-sys`, `contextra-types`
- **Interne Dev/Build-Abhängigkeiten (1):** `contextra-testkit`
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
