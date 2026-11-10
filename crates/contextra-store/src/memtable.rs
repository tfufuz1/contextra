//! # MemTable - In-Memory Schreibpuffer (Ring 1)
//!
//! Verwaltet aktive Mutationen im Hauptspeicher mit sequenzbasierter
//! Sortierung und MVCC-Unterstuetzung vor dem Schreiben auf Festplatte.

// FILE-CONTEXT
// STAND: 2026-08-30T15:00:19Z (SESSION: 283abf0f)
// ZWECK: In-Memory BTreeMap MemTable-Sharding mit MVCC Snapshot-Isolation.
// INVARIANTEN: Sharding per Range-Sharding über 2-Byte-Key-Präfixe für normale Keys, Hash-Sharding der Suffixe für reservierte Systempräfixe; tombstone via TOMBSTONE_BIT in seq_no.
// NICHT-OFFENSICHTLICH: Rollback(tx_id) entfernt alle Einträge der Transaktion atomar aus allen Shards.
//   iter() nutzt size()-basierte Kapazitätsschätzung (AVG_ENTRY_BYTES=64) um Reallokationen
//   beim Flush zu minimieren. put() pre-allokiert Vec<MemTableEntry> mit capacity=2 für den
//   1-2-Versionen-Normalfall (MVCC-Muster).
// HOTSPOTS: MemTable::put, MemTable::get_at_seq, MemTable::rollback
// SIEHE AUCH: crates/contextra-store/AGENTS.md, DECISIONS.md

//! In-memory sorted MemTable for the LSM-Tree with MVCC support.
//!
//! Entries are sharded across `SHARD_COUNT` independent `BTreeMap` partitions,
//! each protected by its own `parking_lot::RwLock`. This reduces write-lock
//! contention when multiple coroutines insert concurrently (e.g. the 8-way
//! `buffer_unordered` ingestion pipeline).
//!
//! Standard user keys are sharded via deterministic 2-byte prefix range sharding
//! (`(b0 << 8) | b1`), preserving key monotonicity across shards for standard keys.
//! Reserved system prefixes (e.g. `__col:`, `__meta:`, `__rel:`, `__graph:`, etc.)
//! use hash-sharding on the key remainder after the system prefix to distribute
//! system writes evenly across all shards and eliminate write contention.
//!
//! Within each shard, each key maps to a versioned list of values, enabling
//! Snapshot Isolation through point-in-time reads.
//!
//! ## Module Invariants
//! - **INVARIANT 1**: All entries are keyed by (key, seq_no). Higher seq_no shadows lower for same key.
//! - **INVARIANT 2**: Tombstone entries have TOMBSTONE_BIT set in seq_no.
//! - **INVARIANT 3**: flush() serializes entries in ascending key order (required by SSTable format).
//! - **INVARIANT 4**: rollback(tx_id) removes ALL entries from that transaction atomically.
//! - **INVARIANT 5**: Sequence numbers assigned to concurrent writes may arrive out of order at a key's version list. put() maintains version lists strictly sorted by raw sequence number (seq & !TOMBSTONE_BIT) in ascending order so that binary search and MVCC snapshot reads remain valid.

use bytes::Bytes;
use contextra_mvcc::tx_buffer::STAGING_ENTRY_OVERHEAD_BYTES;
use contextra_types::{TxId, TOMBSTONE_BIT};
use parking_lot::RwLock;
use std::collections::BTreeMap;
use std::ops::Bound;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

// Compile-time Invariante: TOMBSTONE_BIT muss exakt Bit 63 sein für atomare Bitmaskierung.
const _: () = assert!(TOMBSTONE_BIT == 1u64 << 63);

/// Helper function to check if a sequence number has TOMBSTONE_BIT set.
#[inline]
pub const fn is_tombstone(seq: u64) -> bool {
    (seq & TOMBSTONE_BIT) != 0
}

/// Validates whether start and end key range bounds define a non-empty, valid interval (`start <= end`).
#[inline]
pub fn is_valid_range_bounds(start: Bound<&[u8]>, end: Bound<&[u8]>) -> bool {
    match (start, end) {
        (Bound::Included(s), Bound::Included(e)) => s <= e,
        (Bound::Included(s), Bound::Excluded(e))
        | (Bound::Excluded(s), Bound::Included(e))
        | (Bound::Excluded(s), Bound::Excluded(e)) => s < e,
        _ => true,
    }
}

type SequenceNumber = u64;
type TransactionId = u64;
type MemTableEntry = (SequenceNumber, Bytes, TransactionId);
type MemTableMap = BTreeMap<Bytes, Vec<MemTableEntry>>;

/// Number of independent shards. 16 provides sufficient parallelism for the
/// 8-concurrent-embed pipeline while keeping memory overhead negligible.
/// Must be > 0 (compile-time const, enforced by type system).
const SHARD_COUNT: usize = 16;

/// List of known reserved system key prefixes.
pub const RESERVED_PREFIXES: &[&[u8]] = &[
    b"__col:",
    b"__col_idx:",
    b"__meta:",
    b"__rel:",
    b"__idx:",
    b"__graph:",
    b"__txt:",
    b"__docid:",
    b"__tx_intent:",
    b"__kv_spill:",
    b"__session_dag:",
    b"__maintenance_intent:",
];

/// Helper to detect if a key matches a reserved system prefix.
#[inline]
fn matching_reserved_prefix(key: &[u8]) -> Option<&'static [u8]> {
    if !key.starts_with(b"__") {
        return None;
    }
    for &prefix in RESERVED_PREFIXES {
        if key.starts_with(prefix) {
            return Some(prefix);
        }
    }
    Some(b"__")
}

/// Helper to compute a deterministic hash for reserved system key remainder.
#[inline]
fn hash_remainder(remainder: &[u8]) -> usize {
    let mut hash = 0xcbf29ce484222325u64;
    for &byte in remainder {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3u64);
    }
    (hash as usize) % SHARD_COUNT
}

/// A single shard of the MemTable, holding a subset of the key space.
#[derive(Debug)]
struct MemTableShard {
    entries: RwLock<MemTableMap>,
}

#[derive(Debug)]
pub struct MemTable {
    /// Sharded storage: each shard holds a disjoint subset of keys.
    /// Shard selection is deterministic via `shard_for()`.
    shards: [MemTableShard; SHARD_COUNT],
    size: AtomicUsize,
    min_tx: AtomicU64,
    max_tx: AtomicU64,
}

impl MemTable {
    /// Creates a new empty MemTable with `SHARD_COUNT` independent shards.
    pub fn new() -> Self {
        Self {
            shards: std::array::from_fn(|_| MemTableShard {
                entries: RwLock::new(BTreeMap::new()),
            }),
            size: AtomicUsize::new(0),
            min_tx: AtomicU64::new(u64::MAX),
            max_tx: AtomicU64::new(0),
        }
    }

    /// Deterministic shard selector mapping keys to independent shards.
    ///
    /// Constructs a 16-bit big-endian value from the first two bytes of the key to maintain
    /// strict monotonicity: `keyA <= keyB => shard_for(keyA) <= shard_for(keyB)`.
    ///
    /// ## Design Rationale & Trade-offs
    /// Range sharding is required so that `shard_range_for_prefix` can restrict prefix scans
    /// (e.g. `scan_prefix_into`) to a single shard (for 2+ byte prefixes) or a small range of shards.
    /// Keys sharing identical 2-byte prefixes (such as system prefixes `__col:`, `__meta:`, `__sys:`
    /// or tenant prefixes `t1:`, `t2:`) map to the same shard by design. Replacing range sharding
    /// with a hash mixer would destroy key monotonicity and force every prefix and range scan
    /// to query all 16 shards.
    ///
    /// Zero-panic and zero-allocation.
    #[inline]
    pub fn shard_for(key: &[u8]) -> usize {
        if let Some(prefix) = matching_reserved_prefix(key) {
            let remainder = &key[prefix.len()..];
            hash_remainder(remainder)
        } else {
            let b0 = key.first().copied().unwrap_or(0) as u16;
            let b1 = key.get(1).copied().unwrap_or(0) as u16;
            let val = (b0 << 8) | b1;
            ((val as usize) * SHARD_COUNT) / 65536
        }
    }

    /// Computes the target shard index range for a given key prefix.
    #[inline]
    fn shard_range_for_prefix(prefix: &[u8]) -> std::ops::RangeInclusive<usize> {
        if prefix.first() == Some(&b'_') {
            0..=SHARD_COUNT - 1
        } else if prefix.len() >= 2 {
            let shard = Self::shard_for(prefix);
            shard..=shard
        } else if prefix.len() == 1 {
            let start = Self::shard_for(&[prefix[0], 0x00]);
            let end = Self::shard_for(&[prefix[0], 0xFF]);
            start..=end
        } else {
            0..=SHARD_COUNT - 1
        }
    }

    /// Computes the target shard index range for start and end key bounds.
    #[inline]
    fn shard_range_for_bounds(
        start: Bound<&[u8]>,
        end: Bound<&[u8]>,
    ) -> std::ops::RangeInclusive<usize> {
        let is_system = match (start, end) {
            (Bound::Included(s), _) | (Bound::Excluded(s), _) if s.first() == Some(&b'_') => true,
            (_, Bound::Included(e)) | (_, Bound::Excluded(e)) if e.first() == Some(&b'_') => true,
            (Bound::Unbounded, _) | (_, Bound::Unbounded) => true,
            (Bound::Included(s), Bound::Included(e))
            | (Bound::Included(s), Bound::Excluded(e))
            | (Bound::Excluded(s), Bound::Included(e))
            | (Bound::Excluded(s), Bound::Excluded(e)) => {
                s <= b"__".as_slice() && e >= b"__".as_slice()
            }
        };

        if is_system {
            0..=SHARD_COUNT - 1
        } else {
            let start_shard = match start {
                Bound::Included(s) | Bound::Excluded(s) => Self::shard_for(s),
                Bound::Unbounded => 0,
            };
            let end_shard = match end {
                Bound::Included(e) | Bound::Excluded(e) => Self::shard_for(e),
                Bound::Unbounded => SHARD_COUNT - 1,
            };
            start_shard..=end_shard
        }
    }

    /// Inserts a key-value pair with a sequence number and transaction ID.
    ///
    /// ## Concurrent Sequence Invariant
    /// Sequence numbers pre-allocated to concurrent transactions may arrive out-of-order
    /// at the shard lock. To ensure MVCC snapshot readers (`get`, `get_at_seq`, `iter_latest`,
    /// `scan_prefix_into`) correctly resolve the latest visible version, `put` preserves
    /// ascending sequence number order for each key's version list. If a lower sequence
    /// number arrives after a higher sequence number, it is inserted into its sorted
    /// position via binary search rather than appended.
    pub fn put(&self, key: Bytes, value: Bytes, seq_no: u64, tx_id: u64) {
        let additional_size = key.len() + value.len() + STAGING_ENTRY_OVERHEAD_BYTES;
        let shard_idx = Self::shard_for(&key);
        let mut entries = self.shards[shard_idx].entries.write();

        let versions = entries.entry(key).or_insert_with(|| Vec::with_capacity(2));
        let raw_seq = seq_no & !TOMBSTONE_BIT;

        if versions
            .last()
            .map_or(true, |(last_s, _, _)| (*last_s & !TOMBSTONE_BIT) <= raw_seq)
        {
            // Fast path: sequence numbers are monotonically non-decreasing
            versions.push((seq_no, value, tx_id));
        } else {
            // Out-of-order insertion: find the correct sorted position for `raw_seq`
            let idx = match versions.binary_search_by_key(&raw_seq, |(s, _, _)| *s & !TOMBSTONE_BIT)
            {
                Ok(i) => {
                    let mut pos = i + 1;
                    while pos < versions.len() && (versions[pos].0 & !TOMBSTONE_BIT) == raw_seq {
                        pos += 1;
                    }
                    pos
                }
                Err(i) => i,
            };
            versions.insert(idx, (seq_no, value, tx_id));
        }

        // Note: Simple size tracking (sums all versions)
        self.size.fetch_add(additional_size, Ordering::Relaxed);

        // Track TX range
        loop {
            let current_min = self.min_tx.load(Ordering::Acquire);
            if tx_id >= current_min
                || self
                    .min_tx
                    .compare_exchange(current_min, tx_id, Ordering::AcqRel, Ordering::Acquire)
                    .is_ok()
            {
                break;
            }
        }
        loop {
            let current_max = self.max_tx.load(Ordering::Acquire);
            if tx_id <= current_max
                || self
                    .max_tx
                    .compare_exchange(current_max, tx_id, Ordering::AcqRel, Ordering::Acquire)
                    .is_ok()
            {
                break;
            }
        }
    }

    /// Returns the transaction ID range covered by this MemTable.
    pub fn tx_range(&self) -> (u64, u64) {
        (
            self.min_tx.load(Ordering::Acquire),
            self.max_tx.load(Ordering::Acquire),
        )
    }

    /// Removes all entries associated with the specified transaction ID from the MemTable.
    pub fn rollback(&self, tx_id: u64) {
        let mut total_freed_size = 0usize;
        let mut new_min_tx = u64::MAX;
        let mut new_max_tx = 0u64;
        let mut has_entries = false;

        for shard in &self.shards {
            let mut entries = shard.entries.write();
            entries.retain(|key, versions| {
                versions.retain(|(_seq, val, tx)| {
                    if *tx == tx_id {
                        total_freed_size += key.len() + val.len() + STAGING_ENTRY_OVERHEAD_BYTES;
                        false
                    } else {
                        new_min_tx = new_min_tx.min(*tx);
                        new_max_tx = new_max_tx.max(*tx);
                        has_entries = true;
                        true
                    }
                });
                !versions.is_empty()
            });
        }

        if total_freed_size > 0 {
            #[cfg(debug_assertions)]
            {
                let before = self.size.load(Ordering::Acquire);
                debug_assert!(
                    before >= total_freed_size,
                    "MemTable size underflow detected: before={before}, freed={total_freed_size}"
                );
            }
            self.saturating_sub_size(total_freed_size);
        }

        if !has_entries {
            self.min_tx.store(u64::MAX, Ordering::Release);
            self.max_tx.store(0, Ordering::Release);
        } else {
            self.min_tx.store(new_min_tx, Ordering::Release);
            self.max_tx.store(new_max_tx, Ordering::Release);
        }
    }

    /// Retrieves the latest value and sequence number by key.
    pub fn get(&self, key: &[u8]) -> Option<(Bytes, u64)> {
        let shard_idx = Self::shard_for(key);
        let entries = self.shards[shard_idx].entries.read();
        entries
            .get(key)
            .and_then(|versions| versions.last().map(|(seq, val, _tx)| (val.clone(), *seq)))
    }

    /// Retrieves a value, sequence number, and transaction ID by key at or below a specific sequence number
    /// and bounded by maximum transaction ID for Snapshot Isolation.
    pub fn get_at_seq(&self, key: &[u8], seq_no: u64, max_tx: u64) -> Option<(Bytes, u64, u64)> {
        let shard_idx = Self::shard_for(key);
        let entries = self.shards[shard_idx].entries.read();
        let versions = entries.get(key)?;

        use contextra_types::{TxId, TOMBSTONE_BIT};

        let target_raw_seq = seq_no & !TOMBSTONE_BIT;

        let idx =
            match versions.binary_search_by_key(&target_raw_seq, |(s, _, _)| *s & !TOMBSTONE_BIT) {
                Ok(i) => {
                    let mut last = i;
                    while last + 1 < versions.len()
                        && (versions[last + 1].0 & !TOMBSTONE_BIT) == target_raw_seq
                    {
                        last += 1;
                    }
                    last
                }
                Err(i) => {
                    if i == 0 {
                        return None;
                    }
                    i - 1
                }
            };

        // Linear search backwards for the latest version satisfying (tx <= max_tx || tx >= INTERNAL_BASE)
        for i in (0..=idx).rev() {
            let (s, v, tx) = &versions[i];
            if *tx <= max_tx || *tx >= TxId::INTERNAL_BASE {
                return Some((v.clone(), *s, *tx));
            }
        }
        None
    }

    /// Atomically subtracts `n` from `self.size` with saturation to zero,
    /// preventing integer underflow wrap-around to `usize::MAX`.
    #[inline]
    fn saturating_sub_size(&self, n: usize) {
        let _ = self
            .size
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |v| {
                Some(v.saturating_sub(n))
            });
    }

    /// Returns the approximate size in bytes.
    pub fn size(&self) -> usize {
        self.size.load(Ordering::Relaxed)
    }

    /// Returns true if the memtable is empty.
    ///
    /// WARNING: This reads all shards sequentially. The result is not an
    /// atomic snapshot, but this method is only called on the flush guard
    /// path which is a rare, single-threaded check.
    pub fn is_empty(&self) -> bool {
        self.shards.iter().all(|s| s.entries.read().is_empty())
    }

    /// Iterates over all entries (all versions) in sorted key order.
    /// Returns (Key, Value, SeqNo, TxId).
    pub fn iter(&self) -> Vec<(Bytes, Bytes, u64, u64)> {
        let estimated_entries = {
            let total_bytes = self.size.load(Ordering::Relaxed);
            const AVG_ENTRY_BYTES: usize = 64;
            std::cmp::max(16, total_bytes / AVG_ENTRY_BYTES)
        };
        let mut results = Vec::with_capacity(estimated_entries);
        for shard in &self.shards {
            let entries = shard.entries.read();
            for (k, versions) in entries.iter() {
                for (seq, val, tx) in versions {
                    results.push((k.clone(), val.clone(), *seq, *tx));
                }
            }
        }
        results.sort_by(|a, b| a.0.cmp(&b.0));
        results
    }

    /// Scans entries matching a key prefix into `target` according to MVCC visibility rules.
    ///
    /// Iterates over each shard's range starting at `prefix` and merges matching visible entries directly
    /// into `target`. Higher `seq_no` versions shadow lower `seq_no` versions.
    pub fn scan_prefix_into(
        &self,
        prefix: &[u8],
        seq_no: u64,
        max_tx: TxId,
        target: &mut BTreeMap<Bytes, (Bytes, u64)>,
    ) {
        self.scan_prefix_into_matching(prefix, seq_no, max_tx, target, |_, _, _| true);
    }

    /// Scans entries matching a key prefix into `target` with an additional matching predicate.
    pub fn scan_prefix_into_matching<F>(
        &self,
        prefix: &[u8],
        seq_no: u64,
        max_tx: TxId,
        target: &mut BTreeMap<Bytes, (Bytes, u64)>,
        predicate: F,
    ) where
        F: Fn(&[u8], u64, u64) -> bool,
    {
        let max_tx_val = max_tx.inner();
        let max_seq = seq_no & !TOMBSTONE_BIT;
        let shard_range = Self::shard_range_for_prefix(prefix);

        for shard_idx in shard_range {
            let entries = self.shards[shard_idx].entries.read();
            for (k, versions) in
                entries.range::<[u8], _>((Bound::Included(prefix), Bound::Unbounded))
            {
                if !k.starts_with(prefix) {
                    break;
                }
                let mut best_version: Option<(&Bytes, u64)> = None;
                for (seq, val, tx) in versions {
                    let raw_seq = seq & !TOMBSTONE_BIT;
                    if raw_seq <= max_seq
                        && (*tx <= max_tx_val || *tx >= TxId::INTERNAL_BASE)
                        && predicate(k.as_ref(), raw_seq, *tx)
                    {
                        match best_version {
                            Some((_, best_seq)) => {
                                if raw_seq > (best_seq & !TOMBSTONE_BIT) {
                                    best_version = Some((val, *seq));
                                }
                            }
                            None => {
                                best_version = Some((val, *seq));
                            }
                        }
                    }
                }
                if let Some((val, seq)) = best_version {
                    let raw_seq = seq & !TOMBSTONE_BIT;
                    let entry = target
                        .entry(k.clone())
                        .or_insert_with(|| (val.clone(), seq));
                    if raw_seq > (entry.1 & !TOMBSTONE_BIT) {
                        *entry = (val.clone(), seq);
                    }
                }
            }
        }
    }

    /// Scans entries matching a key range into `target` according to MVCC visibility rules.
    ///
    /// Iterates over each shard's range defined by `(start, end)` bounds and merges matching visible entries directly
    /// into `target`. Higher `seq_no` versions shadow lower `seq_no` versions.
    pub fn scan_range_into(
        &self,
        start: Bound<&[u8]>,
        end: Bound<&[u8]>,
        seq_no: u64,
        max_tx: TxId,
        target: &mut BTreeMap<Bytes, (Bytes, u64)>,
    ) {
        self.scan_range_into_matching(start, end, seq_no, max_tx, target, |_, _, _| true);
    }

    /// Scans entries matching a key range into `target` with an additional matching predicate.
    pub fn scan_range_into_matching<F>(
        &self,
        start: Bound<&[u8]>,
        end: Bound<&[u8]>,
        seq_no: u64,
        max_tx: TxId,
        target: &mut BTreeMap<Bytes, (Bytes, u64)>,
        predicate: F,
    ) where
        F: Fn(&[u8], u64, u64) -> bool,
    {
        if !is_valid_range_bounds(start, end) {
            return;
        }
        let max_tx_val = max_tx.inner();
        let max_seq = seq_no & !TOMBSTONE_BIT;
        let shard_range = Self::shard_range_for_bounds(start, end);

        for shard_idx in shard_range {
            let entries = self.shards[shard_idx].entries.read();
            for (k, versions) in entries.range::<[u8], _>((start, end)) {
                let mut best_version: Option<(&Bytes, u64)> = None;
                for (seq, val, tx) in versions {
                    let raw_seq = seq & !TOMBSTONE_BIT;
                    if raw_seq <= max_seq
                        && (*tx <= max_tx_val || *tx >= TxId::INTERNAL_BASE)
                        && predicate(k.as_ref(), raw_seq, *tx)
                    {
                        match best_version {
                            Some((_, best_seq)) => {
                                if raw_seq > (best_seq & !TOMBSTONE_BIT) {
                                    best_version = Some((val, *seq));
                                }
                            }
                            None => {
                                best_version = Some((val, *seq));
                            }
                        }
                    }
                }
                if let Some((val, seq)) = best_version {
                    let raw_seq = seq & !TOMBSTONE_BIT;
                    let entry = target
                        .entry(k.clone())
                        .or_insert_with(|| (val.clone(), seq));
                    if raw_seq > (entry.1 & !TOMBSTONE_BIT) {
                        *entry = (val.clone(), seq);
                    }
                }
            }
        }
    }

    /// Iterates over only the latest version of each key in sorted key order.
    /// Returns (Key, Value, SeqNo, TxId).
    pub fn iter_latest(&self) -> Vec<(Bytes, Bytes, u64, u64)> {
        let estimated_entries: usize = self.shards.iter().map(|s| s.entries.read().len()).sum();
        let mut results = Vec::with_capacity(estimated_entries);
        for shard in &self.shards {
            let entries = shard.entries.read();
            for (k, versions) in entries.iter() {
                if let Some((seq, val, tx)) = versions.last() {
                    results.push((k.clone(), val.clone(), *seq, *tx));
                }
            }
        }
        results.sort_by(|a, b| a.0.cmp(&b.0));
        results
    }

    /// Returns entry count for each shard (used for testing shard distribution).
    #[doc(hidden)]
    pub fn shard_entry_counts(&self) -> [usize; SHARD_COUNT] {
        std::array::from_fn(|i| self.shards[i].entries.read().len())
    }
}

impl Default for MemTable {
    fn default() -> Self {
        Self::new()
    }
}


#[cfg(test)]
mod tests;
