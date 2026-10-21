//! LSM-Tree (Log-Structured Merge-Tree) storage engine.
// FILE-CONTEXT
// STAND: 2026-10-21
// ZWECK: LSM-Tree-Implementierung (MemTable + SSTable + Compaction)
// INVARIANTEN: Compaction darf keine Daten verlieren; WAL-Replay vor MemTable-Aufbau; LOCK-REIHENFOLGE: commit_mutex → state.write/read → MemTable-RwLock
//              Single-Commit: state.write (Flush-Schutz). Group-Commit-Leader: state.read (commit_mutex hält Isolation).
// NICHT-OFFENSICHTLICH: Compaction-Lock muss VOR MemTable-Lock genommen werden (Deadlock-Gefahr)
// SIEHE AUCH: wal.rs, sstable.rs, DECISIONS.md ADR-003

pub(super) use crate::memtable::MemTable;
pub(super) use crate::sstable::{SstableBuilder, SstableReader};
pub(super) use crate::wal::{Wal, WalOp};
pub(super) use bytes::Bytes;
pub(super) use contextra_ports::{BoxFuture, StorageEngine};
pub(super) use contextra_types::{ContextraError, Result, TxId};
pub(super) use std::path::PathBuf;
pub(super) use std::sync::atomic::Ordering;

pub mod config;
mod engine;
pub mod erasure_backend;
pub mod gc_floor;
mod guard;
pub mod observer;
pub mod probe;
mod validate;
pub mod witness;

pub mod commit;
pub mod fence;
pub mod flush;
pub mod group_commit;
pub mod purge;
pub mod recovery;
pub mod scan;

#[cfg(test)]
mod tests;

pub mod ops;

pub use config::{DurabilityConfigError, DurabilityMode, LsmConfig};
pub use engine::{LsmStorage, StorageHealth};
pub use fence::{
    fenced_keys, ErasureFence, ErasureGate, FenDocLockGuard, FenFenceGuard, FenceError,
    FenceHandle, FenceScopeState, FenceTarget, FencedKey, InflightGuard, PrefixFence, WriteTicket,
};
pub use gc_floor::GcFloor;
pub use probe::ZeuStoreProbe;

pub(super) use guard::{CommitGuard, LsmState};
pub use observer::{
    AsyncObserverAdapter, CommitContext, CommittedBatch, ObserverRegistry, WalEntryRef,
    WalObserver, WriteOrigin, DEFAULT_MAX_OBSERVER_LATENCY,
};
pub(super) use validate::validate_key;

/// Maximum key size allowed for LSM operations (65,535 bytes).
pub const MAX_KEY_SIZE: usize = 65_535;

/// Maximum value size allowed for LSM operations.
/// Must fit within `MAX_WAL_ENTRY_SIZE` (64 MiB) minus `MAX_KEY_SIZE` and framing/encryption overhead.
pub const MAX_VALUE_SIZE: usize = (crate::wal::MAX_WAL_ENTRY_SIZE as usize) - MAX_KEY_SIZE - 4096;

const _: () = assert!(
    MAX_VALUE_SIZE + MAX_KEY_SIZE + 4096 <= crate::wal::MAX_WAL_ENTRY_SIZE as usize,
    "MAX_VALUE_SIZE plus MAX_KEY_SIZE and framing overhead must fit within MAX_WAL_ENTRY_SIZE"
);

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
