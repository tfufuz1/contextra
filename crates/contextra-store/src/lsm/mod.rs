//! LSM-Tree (Log-Structured Merge-Tree) storage engine.
// FILE-CONTEXT
// STAND: 2026-11-11
// ZWECK: LSM-Tree-Implementierung (MemTable + SSTable + Compaction + Lesepfad + Recovery)
// INVARIANTEN: Compaction darf keine Daten verlieren; WAL-Replay vor MemTable-Aufbau.
// SPERRENHIERARCHIE: Siehe lock_order.rs (INV-LOCK-ORDER-1, SPEC §3.5):
//   1. Document-Lock / Intent (RANK 100)
//   2. commit_mutex (RANK 200)
//   3. flush_mutex (RANK 300)
//   4. state-RwLock (RANK 400)
//   5. wal.truncate_lock (RANK 500)
//   6. sstables-RwLock (RANK 600)
//   7. MemTable-Shards (RANK 700)
//   8. Manifest-Append (RANK 800)
// KEIN synchroner Guard (std / parking_lot) ueber ein .await halten!
//!
//! The `LsmStorage` engine provides a high-performance, persistent key-value store
//! implementing the `StorageEngine` trait.
//!
//! ## Architecture
//! - **MemTable**: An in-memory sorted buffer (`BTreeMap`) that absorbs all writes.
//!   Once it reaches a size threshold, it is frozen (becoming an immutable MemTable)
//!   and eventually flushed to disk as an SSTable.
//! - **WAL (Write-Ahead Log)**: Ensures durability by logging all operations before
//!   they are applied to the MemTable.
//! - **SSTables (Sorted String Tables)**: Persistent, immutable files on disk.
//!   They are organized into tiers by the Compaction Engine.
//! - **Compaction**: A background process that merges multiple SSTables into one,
//!   deduplicating keys and garbage-collecting tombstones.
//! - **MVCC (Multi-Version Concurrency Control)**: Supports snapshots and transactional
//!   isolation via sequence numbers and the `SnapshotRegistry`.
//!
//! ## Read Path
//! 1. Check the active MemTable.
//! 2. Check immutable MemTables (from newest to oldest).
//! 3. Check SSTables (from newest to oldest).
//!    Newer sequence numbers shadow older ones for the same key.
//!
//! ## Write Path
//! 1. Operations are staged in the `TxBuffer`.
//! 2. On `commit()`, operations acquire `commit_mutex` to serialize sequence assignment,
//!    are written to the WAL (with fsync durability), and applied to the active MemTable.
//! 3. When the MemTable exceeds `memtable_size_limit`, it rotates to an immutable MemTable
//!    and is flushed asynchronously to a new SSTable file on disk.
//!
//! ## Compaction
//! Compaction runs as a background task. When the number of SSTables in a tier exceeds
//! configured thresholds, compaction merges multiple SSTables into a single new SSTable,
//! deduplicating key versions and garbage-collecting tombstones not pinned by active snapshots.
//!
//! ## Lock Hierarchy & Concurrency Control
//! Formal acquisition order strictly follows `lock_order.rs` (`INV-LOCK-ORDER-1`):
//! 1. Document intent locks (`intent_locks`)
//! 2. Commit mutex (`commit_mutex`)
//! 3. Flush mutex (`flush_mutex`)
//! 4. State read/write lock (`state`)
//! 5. WAL truncation lock (`wal.truncate_lock`)
//! 6. SSTables read/write lock (`sstables`)
//! 7. MemTable shard locks (`MemTable::shards`)
//!
//! Note: synchronous locks must never be held across an `.await` boundary.

pub(super) use crate::memtable::MemTable;
pub(super) use crate::sstable::{SstableBuilder, SstableReader};
pub(super) use crate::wal::{Wal, WalOp};
pub(super) use bytes::Bytes;
use crate::error::{StoreError, StoreResult as Result};
use contextra_ids::TxId;
pub(super) use std::path::PathBuf;
pub(super) use std::sync::atomic::Ordering;

pub mod config;
pub(crate) mod engine;
mod guard;
pub mod observer;
mod validate;

pub mod commit;
pub mod fence;
pub mod flush;
pub mod group_commit;
pub mod lock_order;
pub mod recovery;
pub mod scan;

#[cfg(test)]
mod tests;

pub mod ops;

pub use config::{DurabilityConfigError, DurabilityMode, LsmConfig};
pub use engine::{LsmStorage, StorageHealth};

pub(super) use guard::{CommitGuard, LsmState};
pub use observer::{
    AsyncObserverAdapter, CommitContext, CommittedBatch, ObserverRegistry, WalEntryRef,
    WalObserver, WriteOrigin, DEFAULT_MAX_OBSERVER_LATENCY,
};
pub(super) use validate::validate_key;

/// Maximum key size allowed for LSM operations (65,535 bytes).
pub const MAX_KEY_SIZE: usize = 65_535;

/// Maximum value size allowed for LSM operations (128MB).
pub const MAX_VALUE_SIZE: usize = 134_217_728;

/// Maximum batch size for `delete_many` operations (10,000 items).
pub const MAX_BATCH_SIZE: usize = 10_000;

/// Maximum factor for internal merge set size relative to limit in bounded scans.
pub const MAX_INTERNAL_MERGE_ENTRIES_FACTOR: usize = 8;

/// Maximum batch size for group commits (1,000 transactions).
pub const MAX_GROUP_COMMIT_BATCH_SIZE: usize = 1_000;

/// Minimum surviving entry threshold required to rebuild a new SSTable during rollback.
/// Below this threshold (1..7 entries), surviving entries from a spanning SSTable are inserted
/// directly into the MemTable instead of allocating a full new SSTable/manifest pipeline.
pub const MIN_ENTRIES_FOR_SSTABLE_REBUILD: usize = 8;
