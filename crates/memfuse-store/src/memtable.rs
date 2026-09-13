// FILE-CONTEXT
// STAND: 2026-08-30T15:00:19Z (SESSION: 283abf0f)
// ZWECK: In-Memory BTreeMap MemTable-Sharding mit MVCC Snapshot-Isolation.
// INVARIANTEN: Sharding per 64-Bit-Avalanche-Hash-Mixer modulo SHARD_COUNT; tombstone via TOMBSTONE_BIT in seq_no.
// NICHT-OFFENSICHTLICH: Rollback(tx_id) entfernt alle Einträge der Transaktion atomar aus allen Shards.
//   iter() nutzt size()-basierte Kapazitätsschätzung (AVG_ENTRY_BYTES=64) um Reallokationen
//   beim Flush zu minimieren. put() pre-allokiert Vec<MemTableEntry> mit capacity=2 für den
//   1-2-Versionen-Normalfall (MVCC-Muster).
// HOTSPOTS: MemTable::put, MemTable::get_at_seq, MemTable::rollback
// SIEHE AUCH: crates/memfuse-store/AGENTS.md, DECISIONS.md

//! In-memory sorted MemTable for the LSM-Tree with MVCC support.
//!
//! Entries are sharded across `SHARD_COUNT` independent `BTreeMap` partitions,
//! each protected by its own `parking_lot::RwLock`. This reduces write-lock
//! contention when multiple coroutines insert concurrently (e.g. the 8-way
//! `buffer_unordered` ingestion pipeline).
//!
//! The shard for a given key is selected deterministically via a fast
//! (<5ns) 64-bit avalanche hash mixer over the full key, taken modulo
//! `SHARD_COUNT` to prevent lock contention on shared key
//! prefixes (e.g. `__col:`, `__docid:`).
//!
//! Within each shard, each key maps to a versioned list of values, enabling
//! Snapshot Isolation through point-in-time reads.
//!
//! ## Module Invariants
//! - **INVARIANT 1**: All entries are keyed by (key, seq_no). Higher seq_no shadows lower for same key.
//! - **INVARIANT 2**: Tombstone entries have TOMBSTONE_BIT set in seq_no.
//! - **INVARIANT 3**: flush() serializes entries in ascending key order (required by SSTable format).
//! - **INVARIANT 4**: rollback(tx_id) removes ALL entries from that transaction atomically.

// INVARIANT: In-Memory Sortierter Puffer (hot writes), sharded for concurrency.
// AI-NOTE: Sharding pattern mirrors memfuse-core::TxBuffer<T> (ADR-implicit).
//          Key difference: TxBuffer shards by TxId, MemTable shards by key bytes.

use bytes::Bytes;
use memfuse_core::{TxId, TOMBSTONE_BIT};
use parking_lot::RwLock;
use std::collections::BTreeMap;
use std::ops::Bound;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

const _: () = assert!(TOMBSTONE_BIT == 1u64 << 63);

/// Helper function to check if a sequence number has TOMBSTONE_BIT set.
#[inline]
pub const fn is_tombstone(seq: u64) -> bool {
    (seq & TOMBSTONE_BIT) != 0
}

type SequenceNumber = u64;
type TransactionId = u64;
type MemTableEntry = (SequenceNumber, Bytes, TransactionId);
type MemTableMap = BTreeMap<Bytes, Vec<MemTableEntry>>;

/// Number of independent shards. 16 provides sufficient parallelism for the
/// 8-concurrent-embed pipeline while keeping memory overhead negligible.
/// Must be > 0 (compile-time const, enforced by type system).
const SHARD_COUNT: usize = 16;

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

    /// Deterministic shard selector based on a fast non-cryptographic 64-bit avalanche hash mixer.
    #[inline]
    fn shard_for(key: &[u8]) -> usize {
        // Hash the FULL key, not just the first byte. Namespaced keys share
        // long common prefixes (e.g. "__col:hr:\0", "__docid:"), so any shard
        // selector must mix all input bytes to avoid skew — a single-byte or
        // prefix-only discriminator is provably insufficient given MemFuse's
        // key layout (see Collection::namespaced_key() in
        // crates/memfuse-db/src/collection.rs).
        // Uses a fast 64-bit avalanche mixer (< 5ns) processing 8-byte chunks with finalizer fold.
        // Zero-panic: modulo a compile-time const > 0.
        let mut hash: u64 = 0xa076_1d64_78bd_642f;
        let mut chunks = key.chunks_exact(8);
        for chunk in chunks.by_ref() {
            let val = u64::from_le_bytes(chunk.try_into().unwrap_or([0; 8]));
            hash = hash.wrapping_add(val).wrapping_mul(0x9e37_79b9_7f4a_7c15);
            hash = (hash ^ (hash >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        }
        let remainder = chunks.remainder();
        if !remainder.is_empty() {
            let mut buf = [0u8; 8];
            buf[..remainder.len()].copy_from_slice(remainder);
            let val = u64::from_le_bytes(buf);
            hash = hash.wrapping_add(val).wrapping_mul(0x9e37_79b9_7f4a_7c15);
        }
        hash = (hash ^ (hash >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        hash ^= hash >> 31;
        (hash as usize) % SHARD_COUNT
    }

    /// Inserts a key-value pair with a sequence number and transaction ID.
    pub fn put(&self, key: Bytes, value: Bytes, seq_no: u64, tx_id: u64) {
        let additional_size = key.len() + value.len() + 16; // Added tx_id
        let shard_idx = Self::shard_for(&key);
        let mut entries = self.shards[shard_idx].entries.write();

        let versions = entries.entry(key).or_insert_with(|| Vec::with_capacity(2));
        versions.push((seq_no, value, tx_id));

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
                        total_freed_size += key.len() + val.len() + 16;
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

        use memfuse_core::{TxId, TOMBSTONE_BIT};

        let idx = match versions.binary_search_by_key(&seq_no, |(s, _, _)| *s & !TOMBSTONE_BIT) {
            Ok(i) => i,
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
    ///
    /// Collects from all shards and sorts by key to maintain the global
    /// sorted-order invariant. Called only during flush (low-frequency).
    pub fn iter(&self) -> Vec<(Bytes, Bytes, u64, u64)> {
        let estimated_entries = {
            let total_bytes = self.size.load(Ordering::Relaxed);
            const AVG_ENTRY_BYTES: usize = 64; // konservative Schätzung: 32B Key + 24B Val + 8B Seq
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
        // Restore global sorted order: primary by key, secondary by seq_no
        // within versions of the same key (already sorted per-shard via BTreeMap
        // + push order, but cross-shard merge requires a sort).
        results.sort_by(|a, b| a.0.cmp(&b.0).then(a.2.cmp(&b.2)));
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

        for shard in &self.shards {
            let entries = shard.entries.read();
            for (k, versions) in entries.range::<[u8], _>((Bound::Included(prefix), Bound::Unbounded)) {
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
                    let entry = target.entry(k.clone()).or_insert_with(|| (val.clone(), seq));
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
        let max_tx_val = max_tx.inner();
        let max_seq = seq_no & !TOMBSTONE_BIT;

        for shard in &self.shards {
            let entries = shard.entries.read();
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
                    let entry = target.entry(k.clone()).or_insert_with(|| (val.clone(), seq));
                    if raw_seq > (entry.1 & !TOMBSTONE_BIT) {
                        *entry = (val.clone(), seq);
                    }
                }
            }
        }
    }

    /// Iterates over only the latest version of each key in sorted key order.
    /// Returns (Key, Value, SeqNo, TxId).
    ///
    /// Collects from all shards and sorts by key. Called only during flush.
    pub fn iter_latest(&self) -> Vec<(Bytes, Bytes, u64, u64)> {
        let mut results = Vec::new();
        for shard in &self.shards {
            let entries = shard.entries.read();
            for (k, versions) in entries.iter() {
                if let Some((seq, val, tx)) = versions.last() {
                    results.push((k.clone(), val.clone(), *seq, *tx));
                }
            }
        }
        // Restore global sorted order by key.
        results.sort_by(|a, b| a.0.cmp(&b.0));
        results
    }
}

impl Default for MemTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_put_get() {
        let mt = MemTable::new();
        mt.put(Bytes::from("key1"), Bytes::from("val1"), 1, 1);
        mt.put(Bytes::from("key2"), Bytes::from("val2"), 2, 2);

        let (val, seq) = mt.get(b"key1").expect("key1 should exist"); // expect
        assert_eq!(val.as_ref(), b"val1");
        assert_eq!(seq, 1);

        let (val, seq) = mt.get(b"key2").expect("key2 should exist"); // expect
        assert_eq!(val.as_ref(), b"val2");
        assert_eq!(seq, 2);

        assert!(mt.get(b"key3").is_none());
    }

    #[test]
    fn test_mvcc_get_at_seq() {
        let mt = MemTable::new();
        mt.put(Bytes::from("key1"), Bytes::from("v1"), 10, 1);
        mt.put(Bytes::from("key1"), Bytes::from("v2"), 20, 2);
        mt.put(Bytes::from("key1"), Bytes::from("v3"), 30, 3);

        // Before any version
        assert!(mt.get_at_seq(b"key1", 5, u64::MAX).is_none());

        // Exact match
        let (val, seq, tx) = mt.get_at_seq(b"key1", 20, u64::MAX).unwrap(); // unwrap
        assert_eq!(val.as_ref(), b"v2");
        assert_eq!(seq, 20);
        assert_eq!(tx, 2);

        // Between versions
        let (val, seq, tx) = mt.get_at_seq(b"key1", 25, u64::MAX).unwrap(); // unwrap
        assert_eq!(val.as_ref(), b"v2");
        assert_eq!(seq, 20);
        assert_eq!(tx, 2);

        // Filtered by max_tx: seq 20 has tx=2, max_tx=1 should fallback to seq 10 tx 1
        let (val, seq, tx) = mt.get_at_seq(b"key1", 25, 1).unwrap(); // unwrap
        assert_eq!(val.as_ref(), b"v1");
        assert_eq!(seq, 10);
        assert_eq!(tx, 1);

        // Latest version
        let (val, seq, tx) = mt.get_at_seq(b"key1", 100, u64::MAX).unwrap(); // unwrap
        assert_eq!(val.as_ref(), b"v3");
        assert_eq!(seq, 30);
        assert_eq!(tx, 3);
    }

    #[test]
    fn test_iter_all_versions() {
        let mt = MemTable::new();
        mt.put(Bytes::from("a"), Bytes::from("v1"), 1, 1);
        mt.put(Bytes::from("a"), Bytes::from("v2"), 2, 2);
        mt.put(Bytes::from("b"), Bytes::from("v3"), 3, 3);

        let entries = mt.iter();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].2, 1);
        assert_eq!(entries[1].2, 2);
        assert_eq!(entries[2].2, 3);
    }

    #[test]
    fn test_mvcc_tombstone_binary_search() {
        use memfuse_core::TOMBSTONE_BIT;
        let mt = MemTable::new();
        let key = Bytes::from("key1");

        // Insert a value then a tombstone at a higher sequence number
        mt.put(key.clone(), Bytes::from("val1"), 10, 1);
        mt.put(key.clone(), Bytes::new(), 20 | TOMBSTONE_BIT, 2);

        // Read at seq 15 -> should get val1
        let (val, seq, tx) = mt.get_at_seq(&key, 15, u64::MAX).expect("Should find v1"); // expect
        assert_eq!(val.as_ref(), b"val1");
        assert_eq!(seq, 10);
        assert_eq!(tx, 1);

        // Read at seq 25 -> should get tombstone
        let (val, seq, tx) = mt
            .get_at_seq(&key, 25, u64::MAX)
            .expect("Should find tombstone"); // expect
        assert_eq!(val.len(), 0);
        assert_eq!(seq, 20 | TOMBSTONE_BIT);
        assert_eq!(tx, 2);
    }

    #[test]
    fn test_iter_latest() {
        let mt = MemTable::new();
        mt.put(Bytes::from("a"), Bytes::from("v1"), 1, 1);
        mt.put(Bytes::from("a"), Bytes::from("v2"), 2, 2);
        mt.put(Bytes::from("b"), Bytes::from("v3"), 3, 3);

        let entries = mt.iter_latest();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].0.as_ref(), b"a");
        assert_eq!(entries[0].2, 2);
        assert_eq!(entries[1].0.as_ref(), b"b");
        assert_eq!(entries[1].2, 3);
    }

    #[test]
    fn test_iter_ordering_across_shards() {
        let mt = MemTable::new();
        // Insert keys that map to various shards in reverse order
        let keys = ["z", "m", "a", "k", "b", "c", "x", "p"];
        for (i, &k) in keys.iter().enumerate() {
            mt.put(
                Bytes::from(k),
                Bytes::from(format!("val_{}", k)),
                (i + 1) as u64,
                (i + 1) as u64,
            );
        }

        let latest = mt.iter_latest();
        let fetched_keys: Vec<&[u8]> = latest.iter().map(|(k, _, _, _)| k.as_ref()).collect();
        assert_eq!(
            fetched_keys,
            vec![b"a", b"b", b"c", b"k", b"m", b"p", b"x", b"z"]
        );

        let all = mt.iter();
        let fetched_all_keys: Vec<&[u8]> = all.iter().map(|(k, _, _, _)| k.as_ref()).collect();
        assert_eq!(
            fetched_all_keys,
            vec![b"a", b"b", b"c", b"k", b"m", b"p", b"x", b"z"]
        );
    }

    #[test]
    fn test_size_estimation_accuracy() {
        let mt = MemTable::new();
        assert_eq!(mt.size(), 0);

        // Put key (len 4), val (len 4), overhead 16 => size = 24
        let key1 = Bytes::from("key1");
        let val1 = Bytes::from("val1");
        mt.put(key1, val1, 1, 1);
        assert_eq!(mt.size(), 24);

        // Put delete tombstone: key (len 4), val (len 0), overhead 16 => size += 20
        let key1_del = Bytes::from("key1");
        mt.put(key1_del, Bytes::new(), 2 | TOMBSTONE_BIT, 1);
        assert_eq!(mt.size(), 44);
    }

    #[test]
    fn test_transaction_rollback() {
        let mt = MemTable::new();

        // Begin tx A (tx_id = 100), put 3 keys
        mt.put(Bytes::from("keyA1"), Bytes::from("valA1"), 1, 100);
        mt.put(Bytes::from("keyA2"), Bytes::from("valA2"), 2, 100);
        mt.put(Bytes::from("keyA3"), Bytes::from("valA3"), 3, 100);

        // Begin tx B (tx_id = 200), put 2 keys
        mt.put(Bytes::from("keyB1"), Bytes::from("valB1"), 4, 200);
        mt.put(Bytes::from("keyB2"), Bytes::from("valB2"), 5, 200);

        assert_eq!(mt.iter_latest().len(), 5);

        // Rollback tx A
        mt.rollback(100);

        // Scan all keys — only B's keys should appear
        let latest = mt.iter_latest();
        assert_eq!(latest.len(), 2);
        assert_eq!(latest[0].0.as_ref(), b"keyB1");
        assert_eq!(latest[0].3, 200);
        assert_eq!(latest[1].0.as_ref(), b"keyB2");
        assert_eq!(latest[1].3, 200);

        assert!(mt.get(b"keyA1").is_none());
        assert!(mt.get(b"keyA2").is_none());
        assert!(mt.get(b"keyA3").is_none());
        assert!(mt.get(b"keyB1").is_some());
        assert!(mt.get(b"keyB2").is_some());

        // Verify size tracking updated after rollback
        // Originally: tx A = 3 * (5 + 5 + 16) = 78; tx B = 2 * (5 + 5 + 16) = 52. Total = 130.
        // After rollback tx A: remaining size should be 52.
        assert_eq!(mt.size(), 52);
    }

    #[test]
    fn test_shard_distribution_for_realistic_collection_keys() {
        // Simulates the actual key shapes produced during ingestion into ONE
        // named collection ("hr") — the primary workload this sharding
        // targets. Mirrors Collection::namespaced_key() for key_type == 0.
        use std::collections::HashSet;

        let mut shards_hit = HashSet::new();
        for i in 0..1000u32 {
            let mut key = b"__col:hr:\x00".to_vec();
            key.push(0u8); // key_type = 0 (user key)
            key.extend_from_slice(format!("doc-{i}").as_bytes());
            shards_hit.insert(MemTable::shard_for(&key));
        }
        assert!(
            shards_hit.len() >= SHARD_COUNT / 2,
            "Realistic collection keys must spread across at least half the \
             shards; got only {} distinct shards — sharding provides no \
             contention relief for this key shape.",
            shards_hit.len()
        );
    }

    #[test]
    fn test_shard_distribution_for_docid_mapping_keys() {
        // Mirrors the __docid: keys written on every insert() in the default
        // collection (Collection::insert_op, key_type == 1).
        use std::collections::HashSet;

        let mut shards_hit = HashSet::new();
        for i in 0u64..1000 {
            let mut key = b"__docid:".to_vec();
            key.extend_from_slice(&i.to_le_bytes());
            shards_hit.insert(MemTable::shard_for(&key));
        }
        assert!(
            shards_hit.len() >= SHARD_COUNT / 2,
            "Docid-mapping keys (written on every insert) must spread across \
             at least half the shards; got only {} — every insert() call \
             would otherwise serialize on a handful of shards.",
            shards_hit.len()
        );
    }

    #[test]
    fn test_concurrent_put_no_data_loss() {
        // Write 1000 keys from 8 threads, verify all are readable.
        use std::sync::Arc;
        let mt = Arc::new(MemTable::new());
        let handles: Vec<_> = (0..8u64)
            .map(|t| {
                let mt = Arc::clone(&mt);
                std::thread::spawn(move || {
                    for i in 0..125u64 {
                        let key = Bytes::from(format!("key-{}-{}", t, i));
                        mt.put(key, Bytes::from("val"), t * 125 + i + 1, t);
                    }
                })
            })
            .collect();
        for h in handles {
            h.join().expect("thread panicked"); // #[cfg(test)] // expect
        }
        assert_eq!(mt.iter_latest().len(), 1000);
    }

    #[test]
    fn test_empty_memtable_edge_cases() {
        let mt = MemTable::new();

        // 1. is_empty on new memtable
        assert!(mt.is_empty());
        assert_eq!(mt.size(), 0);

        // 2. tx_range on empty memtable -> (u64::MAX, 0)
        assert_eq!(mt.tx_range(), (u64::MAX, 0));

        // 3. Rollback non-existent transaction on empty or populated memtable
        mt.rollback(999);
        assert!(mt.is_empty());
        assert_eq!(mt.tx_range(), (u64::MAX, 0));

        // Insert and then rollback non-existent tx
        mt.put(Bytes::from("k"), Bytes::from("v"), 1, 10);
        assert!(!mt.is_empty());
        assert_eq!(mt.tx_range(), (10, 10));

        mt.rollback(999); // should do nothing
        assert!(!mt.is_empty());
        assert_eq!(mt.get(b"k").expect("should exist").0.as_ref(), b"v"); // expect
        assert_eq!(mt.tx_range(), (10, 10));

        // Rollback existing tx
        mt.rollback(10);
        assert!(mt.is_empty());
        assert_eq!(mt.size(), 0);
        assert_eq!(mt.tx_range(), (u64::MAX, 0));
    }

    #[test]
    fn test_saturating_sub_size_underflow_saturates_to_zero() {
        let mt = MemTable::new();
        mt.put(Bytes::from("key1"), Bytes::from("val1"), 1, 1);
        assert!(mt.size() > 0);

        // Directly invoke saturating_sub_size with an amount far larger than current size
        let current_size = mt.size();
        mt.saturating_sub_size(current_size + 1000);

        // Verify size saturates at 0 and does not wrap around to usize::MAX
        assert_eq!(mt.size(), 0);
    }

    #[test]
    fn test_scan_prefix_into_and_scan_range_into() {
        let mt = MemTable::new();

        // Insert versions for keys across prefix and range
        mt.put(Bytes::from("prefix:a"), Bytes::from("val_a_v1"), 10, 1);
        mt.put(Bytes::from("prefix:a"), Bytes::from("val_a_v2"), 20, 2);
        mt.put(Bytes::from("prefix:b"), Bytes::from("val_b_v1"), 15, 1);
        mt.put(Bytes::from("other:c"), Bytes::from("val_c_v1"), 30, 3);

        // Test scan_prefix_into at max_seq=25, max_tx=TxId(2)
        let mut target = BTreeMap::new();
        mt.scan_prefix_into(b"prefix:", 25, TxId(2), &mut target);
        assert_eq!(target.len(), 2);
        assert_eq!(target.get(b"prefix:a".as_slice()), Some(&(Bytes::from("val_a_v2"), 20)));
        assert_eq!(target.get(b"prefix:b".as_slice()), Some(&(Bytes::from("val_b_v1"), 15)));

        // Test scan_prefix_into at max_seq=12, max_tx=TxId(1)
        let mut target2 = BTreeMap::new();
        mt.scan_prefix_into(b"prefix:", 12, TxId(1), &mut target2);
        assert_eq!(target2.len(), 1);
        assert_eq!(target2.get(b"prefix:a".as_slice()), Some(&(Bytes::from("val_a_v1"), 10)));

        // Test scan_range_into bounded range [prefix:a, prefix:z]
        let mut target3 = BTreeMap::new();
        mt.scan_range_into(
            Bound::Included(b"prefix:a".as_slice()),
            Bound::Included(b"prefix:z".as_slice()),
            100,
            TxId(10),
            &mut target3,
        );
        assert_eq!(target3.len(), 2);
        assert_eq!(target3.get(b"prefix:a".as_slice()), Some(&(Bytes::from("val_a_v2"), 20)));
        assert_eq!(target3.get(b"prefix:b".as_slice()), Some(&(Bytes::from("val_b_v1"), 15)));
    }

    #[test]
    fn test_scan_prefix_into_bit_identity_with_iter() {
        let mt = MemTable::new();
        let prefix = b"user:";

        for i in 0..100u64 {
            let k = Bytes::from(format!("user:{:04}", i));
            let v = Bytes::from(format!("val_{}", i));
            mt.put(k.clone(), v.clone(), i + 1, (i % 5) + 1);
            // Put a second version for even keys
            if i % 2 == 0 {
                mt.put(k, Bytes::from(format!("val_v2_{}", i)), i + 101, (i % 5) + 1);
            }
        }

        let max_seq = 150u64;
        let max_tx = TxId(4);

        // 1. Target via scan_prefix_into
        let mut fast_target = BTreeMap::new();
        mt.scan_prefix_into(prefix, max_seq, max_tx, &mut fast_target);

        // 2. Target via legacy mt.iter() filter logic
        let mut legacy_target: BTreeMap<Bytes, (Bytes, u64)> = BTreeMap::new();
        for (k, v, seq, tx) in mt.iter() {
            if k.starts_with(prefix) {
                let raw_seq = seq & !TOMBSTONE_BIT;
                if raw_seq <= (max_seq & !TOMBSTONE_BIT)
                    && (tx <= max_tx.inner() || tx >= TxId::INTERNAL_BASE)
                {
                    let entry = legacy_target.entry(k).or_insert_with(|| (v.clone(), seq));
                    if (seq & !TOMBSTONE_BIT) > (entry.1 & !TOMBSTONE_BIT) {
                        *entry = (v, seq);
                    }
                }
            }
        }

        assert_eq!(fast_target, legacy_target, "Fast range/prefix scan results must be bit-identical to legacy iter-based filtering");
    }

    #[test]
    fn test_concurrent_writer_and_range_scanner() {
        use std::sync::Arc;
        let mt = Arc::new(MemTable::new());

        // Pre-populate 50 keys
        for i in 0..50u64 {
            mt.put(
                Bytes::from(format!("doc:{:03}", i)),
                Bytes::from(format!("val_{}", i)),
                i + 1,
                1,
            );
        }

        let mt_writer = Arc::clone(&mt);
        let mt_scanner = Arc::clone(&mt);

        let writer_handle = std::thread::spawn(move || {
            for i in 50..150u64 {
                mt_writer.put(
                    Bytes::from(format!("doc:{:03}", i)),
                    Bytes::from(format!("val_{}", i)),
                    i + 1,
                    2,
                );
            }
        });

        let scanner_handle = std::thread::spawn(move || {
            for _ in 0..100 {
                let mut target = BTreeMap::new();
                mt_scanner.scan_prefix_into(b"doc:", 200, TxId(2), &mut target);
                assert!(target.len() >= 50 && target.len() <= 150);
            }
        });

        writer_handle.join().expect("writer finished");
        scanner_handle.join().expect("scanner finished");
    }
}
