//! # MemTable - In-Memory Schreibpuffer (Ring 1)
//!
//! Verwaltet aktive Mutationen im Hauptspeicher mit sequenzbasierter
//! Sortierung und MVCC-Unterstuetzung vor dem Schreiben auf Festplatte.

// FILE-CONTEXT
// STAND: 2026-08-30T15:00:19Z (SESSION: 283abf0f)
// ZWECK: In-Memory BTreeMap MemTable-Sharding mit MVCC Snapshot-Isolation.
// INVARIANTEN: Sharding per Range-Sharding über 2-Byte-Key-Präfixe für normale Keys, Hash-Sharding der Suffixe für reservierte Systempräfixe; tombstone via TOMBSTONE_BIT in seq_no.

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

pub mod iter;
pub mod mvcc;
pub mod shard;
pub mod tombstone;

#[cfg(test)]
mod tests;

pub use shard::{
    hash_remainder, matching_reserved_prefix, shard_for, shard_range_for_bounds,
    shard_range_for_prefix, MemTableEntry, MemTableMap, MemTableShard, RESERVED_PREFIXES,
    SHARD_COUNT,
};
pub use tombstone::{is_tombstone, is_valid_range_bounds, TOMBSTONE_BIT};

#[derive(Debug)]
pub struct MemTable {
    /// Sharded storage: each shard holds a disjoint subset of keys.
    /// Shard selection is deterministic via `shard_for()`.
    pub(crate) shards: [MemTableShard; SHARD_COUNT],
    pub(crate) size: AtomicUsize,
    pub(crate) min_tx: AtomicU64,
    pub(crate) max_tx: AtomicU64,
}

impl MemTable {
    /// Creates a new, empty sharded MemTable.
    pub fn new() -> Self {
        Self {
            shards: std::array::from_fn(|_| MemTableShard::new()),
            size: AtomicUsize::new(0),
            min_tx: AtomicU64::new(u64::MAX),
            max_tx: AtomicU64::new(0),
        }
    }

    /// Atomically subtracts `n` from `self.size` with saturation to zero,
    /// preventing integer underflow wrap-around to `usize::MAX`.
    #[inline]
    pub(crate) fn saturating_sub_size(&self, n: usize) {
        let _ = self
            .size
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |v| {
                Some(v.saturating_sub(n))
            });
    }

    /// Returns the approximate size in bytes.
    #[inline]
    pub fn size(&self) -> usize {
        self.size.load(Ordering::Relaxed)
    }

    /// Returns true if the memtable is empty.
    pub fn is_empty(&self) -> bool {
        self.shards.iter().all(|s| s.entries.read().is_empty())
    }

    /// Returns entry count for each shard (used for testing shard distribution).
    #[doc(hidden)]
    pub fn shard_entry_counts(&self) -> [usize; SHARD_COUNT] {
        std::array::from_fn(|i| self.shards[i].entries.read().len())
    }

    /// Exposes shard calculation for testing key distribution.
    #[inline]
    pub fn shard_for(key: &[u8]) -> usize {
        shard_for(key)
    }

    /// Exposes shard range for prefix for testing and internal scanning.
    #[inline]
    pub fn shard_range_for_prefix(prefix: &[u8]) -> std::ops::RangeInclusive<usize> {
        shard_range_for_prefix(prefix)
    }

    /// Exposes shard range for bounds for testing and internal scanning.
    #[inline]
    pub fn shard_range_for_bounds(
        start: std::ops::Bound<&[u8]>,
        end: std::ops::Bound<&[u8]>,
    ) -> std::ops::RangeInclusive<usize> {
        shard_range_for_bounds(start, end)
    }
}

impl Default for MemTable {
    fn default() -> Self {
        Self::new()
    }
}
