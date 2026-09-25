//! LSM-Tree (Log-Structured Merge-Tree) storage engine.
// FILE-CONTEXT
// STAND: 2026-08-30T21:49:55Z (SESSION: 283abf0f)
// ZWECK: LSM-Tree-Implementierung (MemTable + SSTable + Compaction)
// INVARIANTEN: Compaction darf keine Daten verlieren; WAL-Replay vor MemTable-Aufbau; LOCK-REIHENFOLGE: commit_mutex → state.write/read → MemTable-RwLock
//              Single-Commit: state.write (Flush-Schutz). Group-Commit-Leader: state.read (commit_mutex hält Isolation).
// NICHT-OFFENSICHTLICH: Compaction-Lock muss VOR MemTable-Lock genommen werden (Deadlock-Gefahr)
// SIEHE AUCH: wal.rs, sstable.rs, DECISIONS.md ADR-003

// INVARIANT: Zentraler Storage-Engine-Orchestrator des Triebwerks.
// IMPLEMENTS: StorageEngine Trait (contextra-core/src/traits.rs)
// READ-PATH:  get() → Active MemTable → Immutable MemTables → SSTables (newest first)
// WRITE-PATH: put()/delete() → TxBuffer → commit() → WAL + MemTable
// FLUSH:      MemTable > size_limit → rotate → SSTable schreiben → cleanup
// BACKGROUND: CompactionEngine läuft als tokio::spawn loop
// INVARIANTE: WAL Replay bei Neustart stellt MemTable deterministisch wieder her.
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
//! ## `commit_mutex` Role
//! `commit_mutex` serializes sequence allocation and WAL batch preparation during commits, preventing
//! snapshot inversion. In the group commit leader path, `commit_mutex` is released prior to executing physical
//! disk I/O (`wal.append_batch`) and re-acquired afterwards for MemTable updates / visibility advancement (and on error for WAL rollback).
//!
//! ## Lock Hierarchy & Concurrency Control
//! To prevent deadlocks, locks across the LSM storage engine must be acquired in the following order:
//! 1. `commit_mutex` (`tokio::sync::Mutex<()>`) - Acquired during sequence/batch preparation, rollback_to_tx, and state mutations. Released before disk I/O in group commit leader happy path, and re-acquired for MemTable update and visibility advancement.
//! 2. `state` write lock (`tokio::sync::RwLock<LsmState>`) - Protects active/immutable memtable pointers & WAL.
//! 3. `sstables` write lock (`tokio::sync::RwLock<Vec<Arc<SstableReader>>>`) - Protects SSTable set.
//!    Read locks on `state` and `sstables` may be acquired concurrently without holding `commit_mutex`.

pub(super) use crate::memtable::MemTable;
pub(super) use crate::sstable::{SstableBuilder, SstableReader};
pub(super) use crate::wal::{Wal, WalOp};
pub(super) use bytes::Bytes;
pub(super) use contextra_core::{
    BoxFuture, ContextraError, Result, StorageEngine, TxId,
};
pub(super) use std::path::PathBuf;
pub(super) use std::sync::atomic::Ordering;

mod config;
mod engine;
mod guard;
mod validate;

pub mod commit;
pub mod flush;
pub mod group_commit;
pub mod recovery;
pub mod scan;

#[cfg(test)]
mod tests;

pub mod ops;

pub use config::LsmConfig;
pub use engine::LsmStorage;
pub(super) use guard::{CommitGuard, LsmState};
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
