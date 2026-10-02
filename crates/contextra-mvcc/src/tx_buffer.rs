//! Transactional buffer for staging index operations.
//!
//! Sharded into sub-buffers to reduce lock contention.
//! Each shard is independently locked, allowing concurrent writers
//! to different transactions.
//!
//! # INVARIANT
//! To avoid deadlocks, callers must never acquire more than one shard lock simultaneously.
//! In multi-shard sweeps, `reap_orphans()` acquires all shards sequentially in ascending index
//! order (index 0 to N-1), acquiring and releasing each shard lock one at a time via `try_write()`.

// FILE-CONTEXT
// STAND: 2026-09-28T00:00:00Z (SESSION: e459bd5f)
// ZWECK: Shard-basierter Transaktionsbuffer für das Staging von 2-Phase-Commit Index-Operationen, ReadSet-Limits & Watermark-Tracking.
// INVARIANTEN: Shard-Isolation per TxId; Niemals zwei Shards gleichzeitig sperren (Deadlock-Prävention); Bounded ReadSet via try_register_read.
// HOTSPOTS: 110-380
// NICHT-OFFENSICHTLICH: Orphan Reaper & min_read_snapshot führen getrennte Locks pro Shard 0..N-1 durch; keinesfalls verschachtelt.
// SIEHE AUCH: crates/contextra-mvcc/src/ssi.rs

// INVARIANT: Sharded Transaction Buffer für lock-freie Concurrency.

use crate::error::{ContextraError, Result};
use crate::ssi::ReadSet;
use crate::types::{DocId, TxId};
use ahash::AHashMap;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

/// Single canonical constant for overhead calculation per staged entry (32 bytes).
pub const STAGING_ENTRY_OVERHEAD_BYTES: usize = 32;

/// Default number of shards for the transaction buffer.
pub const DEFAULT_SHARD_COUNT: usize = 64;

/// Recommended maximum operations per single transaction to guard against memory exhaustion DoS.
// AI-TAG[SMELL][MINOR] RESOLVED: AGT-CORE-001 — Bounded staging capacity enforced (TS:2026-08-29T12:00:00Z) (SESSION: a3f29c1d)
pub const DEFAULT_MAX_OPS_PER_TX: usize = 10_000;

/// Default maximum number of read keys tracked per transaction's [`ReadSet`].
///
/// Prevents unbounded memory growth during large read scans (~100,000 keys * ~100B per key = ~10MB per active transaction).
pub const DEFAULT_MAX_READ_SET_KEYS: usize = 100_000;

/// Default maximum staged bytes per transaction (16 MiB).
///
/// Bounds staging memory per individual transaction while enabling high-throughput bulk operations.
pub const DEFAULT_MAX_TX_STAGED_BYTES: usize = 16 * 1024 * 1024;

/// Default maximum total staged bytes across all active transactions (256 MiB).
///
/// Guards the process memory budget against multi-transaction staging memory exhaustion.
pub const DEFAULT_MAX_TOTAL_STAGED_BYTES: usize = 256 * 1024 * 1024;

/// Configuration options for `TxBuffer`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxBufferConfig {
    /// Transaction timeout duration.
    pub tx_timeout: Duration,
    /// Maximum recommended active transactions.
    pub max_active_tx: usize,
    /// Maximale Anzahl von Staging-Operationen pro Transaktion.
    /// Verhindert OOM durch einzelne unbegrenzte Transaktionen.
    /// Default: 10_000 (großzügig, aber bounded).
    pub max_ops_per_tx: usize,
    /// Maximale Byte-Kapazität von Staging-Einträgen pro einzelnen Transaktion.
    /// Default: 16 MiB.
    pub max_tx_staged_bytes: usize,
    /// Maximale Gesamtkapazität von Staging-Einträgen über alle Transaktionen hinweg.
    /// Default: 256 MiB.
    pub max_total_staged_bytes: usize,
}

impl Default for TxBufferConfig {
    fn default() -> Self {
        Self {
            tx_timeout: Duration::from_secs(30),
            max_active_tx: 64,
            max_ops_per_tx: DEFAULT_MAX_OPS_PER_TX,
            max_tx_staged_bytes: DEFAULT_MAX_TX_STAGED_BYTES,
            max_total_staged_bytes: DEFAULT_MAX_TOTAL_STAGED_BYTES,
        }
    }
}

/// Operation to be executed in an index.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum IndexOp<T: Clone> {
    /// Insert a document with associated data.
    Insert {
        /// Document identifier to insert.
        doc_id: DocId,
        /// Associated payload or index data.
        data: T,
    },
    /// Delete a document.
    Delete {
        /// Document identifier to delete.
        doc_id: DocId,
        /// Optional payload associated with deletion.
        data: Option<T>,
    },
}

impl<T: Clone> IndexOp<T> {
    /// Returns the document ID for this operation.
    pub fn doc_id(&self) -> DocId {
        match self {
            IndexOp::Insert { doc_id, .. } => *doc_id,
            IndexOp::Delete { doc_id, .. } => *doc_id,
        }
    }
}

/// Trait for estimating the byte footprint of a staged payload.
pub trait StagedOpSize {
    /// Estimates memory footprint in bytes for staging entry overhead accounting.
    fn staged_bytes(&self) -> usize;
}

impl<T> StagedOpSize for Vec<T> {
    fn staged_bytes(&self) -> usize {
        self.len() * std::mem::size_of::<T>()
    }
}

impl StagedOpSize for (Vec<u8>, Vec<u8>) {
    fn staged_bytes(&self) -> usize {
        self.0.len() + self.1.len()
    }
}

impl StagedOpSize for String {
    fn staged_bytes(&self) -> usize {
        self.len()
    }
}

impl StagedOpSize for usize {
    fn staged_bytes(&self) -> usize {
        std::mem::size_of::<usize>()
    }
}

impl StagedOpSize for u64 {
    fn staged_bytes(&self) -> usize {
        std::mem::size_of::<u64>()
    }
}

impl StagedOpSize for u8 {
    fn staged_bytes(&self) -> usize {
        std::mem::size_of::<u8>()
    }
}

impl<T: Clone + StagedOpSize> IndexOp<T> {
    /// Estimates total memory footprint in bytes including fixed overhead and payload size.
    pub fn estimated_bytes(&self) -> usize {
        STAGING_ENTRY_OVERHEAD_BYTES
            + match self {
                IndexOp::Insert { data, .. } => data.staged_bytes(),
                IndexOp::Delete { data: Some(d), .. } => d.staged_bytes(),
                IndexOp::Delete { data: None, .. } => 0,
            }
    }
}

#[derive(Debug)]
struct TxShard<T: Clone> {
    ops: AHashMap<TxId, (Vec<IndexOp<T>>, Instant)>,
    read_sets: AHashMap<TxId, ReadSet>,
    staged_bytes: AHashMap<TxId, usize>,
}

impl<T: Clone> TxShard<T> {
    fn new() -> Self {
        Self {
            ops: AHashMap::new(),
            read_sets: AHashMap::new(),
            staged_bytes: AHashMap::new(),
        }
    }
}

#[derive(Debug)]
struct KeyShard {
    // Map: key -> Map<tx_id, is_insert>
    staged: AHashMap<Vec<u8>, AHashMap<TxId, bool>>,
}

impl KeyShard {
    fn new() -> Self {
        Self {
            staged: AHashMap::new(),
        }
    }
}

/// Buffers index operations until commit or rollback.
///
/// Sharded into sub-buffers to reduce lock contention.
///
/// # INVARIANT
/// Each shard is protected by an independent `parking_lot::RwLock`.
/// Standard acquisition order: Read-lock for queries, Write-lock for mutations.
/// To avoid deadlocks, cross-shard operations must never acquire more than
/// one shard lock simultaneously. Operations that process all shards (like `reap_orphans()` or `min_read_snapshot()`)
/// MUST iterate over shards sequentially in ascending index order (from shard 0 to N-1)
/// and release each lock before acquiring the next.
#[derive(Debug)]
pub struct TxBuffer<T: Clone> {
    shards: Vec<RwLock<TxShard<T>>>,
    key_shards: Vec<RwLock<KeyShard>>,
    tx_timeout: Duration,
    config: TxBufferConfig,
    global_staged_bytes: AtomicUsize,
}

impl<T: Clone> TxBuffer<T> {
    /// Creates a new buffer with default 64 shards and 30s timeout.
    pub fn new() -> Self {
        Self::new_with_config(DEFAULT_SHARD_COUNT, Duration::from_secs(30))
    }

    /// Creates a new buffer with custom settings.
    ///
    /// If `shard_count` is 0, it defaults to 1 to prevent division-by-zero
    /// in `shard_idx()` (§2 Zero-Panic-Gesetz).
    pub fn new_with_config(shard_count: usize, tx_timeout: Duration) -> Self {
        let config = TxBufferConfig {
            tx_timeout,
            ..Default::default()
        };
        Self::new_with_config_ext(shard_count, tx_timeout, config)
    }

    /// Creates a new buffer with explicit `TxBufferConfig`.
    pub fn new_with_config_ext(
        shard_count: usize,
        _tx_timeout: Duration,
        config: TxBufferConfig,
    ) -> Self {
        let shard_count = if shard_count == 0 { 1 } else { shard_count };
        let mut shards = Vec::with_capacity(shard_count);
        let mut key_shards = Vec::with_capacity(shard_count);
        for _ in 0..shard_count {
            shards.push(RwLock::new(TxShard::new()));
            key_shards.push(RwLock::new(KeyShard::new()));
        }
        Self {
            shards,
            key_shards,
            tx_timeout: config.tx_timeout,
            config,
            global_staged_bytes: AtomicUsize::new(0),
        }
    }

    /// Returns the current total staged bytes across all active transactions.
    pub fn staged_bytes(&self) -> usize {
        self.global_staged_bytes.load(Ordering::Relaxed)
    }

    #[inline]
    fn shard_idx(&self, tx: TxId) -> usize {
        // SAFETY: Modulo-Cast u64→usize (sicher wegen %-Operator)
        (tx.inner() % self.shards.len() as u64) as usize
    }

    #[inline]
    fn key_shard_idx(&self, key: &[u8]) -> usize {
        use std::hash::Hasher;
        let mut hasher = ahash::AHasher::default();
        hasher.write(key);
        (hasher.finish() % self.key_shards.len() as u64) as usize
    }

    /// Checks if the given transaction exists in the buffer.
    pub fn has_tx(&self, tx: TxId) -> bool {
        // SAFETY: Slice-Indexing — sicher weil shard_idx = modulo len()
        let shard = &self.shards[self.shard_idx(tx)];
        shard.read().ops.contains_key(&tx)
    }

    /// Registers a new transaction in the buffer at timestamp `at`.
    pub fn begin_at(&self, tx: TxId, at: Instant) {
        let shard_idx = self.shard_idx(tx);
        let mut shard = self.shards[shard_idx].write();
        shard
            .ops
            .entry(tx)
            .or_insert_with(|| (Vec::with_capacity(16), at));
        shard.read_sets.entry(tx).or_default();
        shard.staged_bytes.entry(tx).or_insert(0);
    }

    /// Registers a new transaction in the buffer.
    pub fn begin(&self, tx: TxId) {
        self.begin_at(tx, Instant::now());
    }

    /// Registers a read key and its snapshot sequence number for transaction `tx`.
    pub fn register_read(&self, tx: TxId, key: impl Into<Vec<u8>>, snapshot_seq: u64) {
        let shard_idx = self.shard_idx(tx);
        let mut shard = self.shards[shard_idx].write();
        shard
            .read_sets
            .entry(tx)
            .or_default()
            .record_read(key, snapshot_seq);
    }

    /// Registers a read key and its snapshot sequence number for transaction `tx`,
    /// enforcing a maximum key count limit `max_keys` on new key insertions.
    ///
    /// Updating an existing key's snapshot sequence is always allowed even if `max_keys` is reached.
    ///
    /// # Errors
    /// Returns `Err(ContextraError::LimitExceeded)` if a new key is added and `read_set.len() >= max_keys`.
    pub fn try_register_read(
        &self,
        tx: TxId,
        key: impl Into<Vec<u8>>,
        snapshot_seq: u64,
        max_keys: usize,
    ) -> Result<()> {
        let shard_idx = self.shard_idx(tx);
        let mut shard = self.shards[shard_idx].write();
        let rs = shard.read_sets.entry(tx).or_default();

        let k = key.into();
        if rs.get(&k).is_none() && rs.len() >= max_keys {
            return Err(ContextraError::limit_exceeded(
                max_keys,
                format!("Read set size limit exceeded for transaction {tx}"),
            ));
        }

        rs.record_read(k, snapshot_seq);
        Ok(())
    }

    /// Alias for [`TxBuffer::register_read`].
    pub fn record_read(&self, tx: TxId, key: impl Into<Vec<u8>>, snapshot_seq: u64) {
        self.register_read(tx, key, snapshot_seq);
    }

    /// Returns the lowest snapshot sequence number across all read sets of all active transactions in all shards.
    ///
    /// # Concurrency & Atomicity Note
    /// Iterates through shards sequentially in ascending order (0 to N-1), acquiring and releasing a single shard read lock at a time.
    /// The returned minimum snapshot sequence is NOT an atomic snapshot across all shards.
    /// Callers determining a safe pruning watermark MUST compute:
    /// `min(min_read_snapshot(), last_allocated_seq_before_call)`.
    /// Returns the lowest snapshot sequence number across all read sets of all active transactions in all shards.
    ///
    /// # INVARIANT (contextra-store Integration)
    /// `contextra-store` uses `min_read_snapshot()` combined with `SnapshotRegistry::min_active_seqno()` to
    /// prevent pruning SSI write keys active transactions still depend on.
    /// Locks each shard sequentially in ascending index order (0..N-1) without holding multiple shard locks simultaneously.
    pub fn min_read_snapshot(&self) -> Option<u64> {
        let mut overall_min: Option<u64> = None;
        for shard_lock in &self.shards {
            let shard = shard_lock.read();
            for rs in shard.read_sets.values() {
                if let Some(min_seq) = rs.min_snapshot_seq() {
                    overall_min = Some(overall_min.map_or(min_seq, |curr| curr.min(min_seq)));
                }
            }
        }
        overall_min
    }

    /// Returns a clone of the accumulated [`ReadSet`] for transaction `tx`, if present.
    pub fn get_read_set(&self, tx: TxId) -> Option<ReadSet> {
        let shard_idx = self.shard_idx(tx);
        let shard = self.shards[shard_idx].read();
        shard.read_sets.get(&tx).cloned()
    }

    /// Alias for [`TxBuffer::get_read_set`].
    pub fn read_set(&self, tx: TxId) -> Option<ReadSet> {
        self.get_read_set(tx)
    }

    /// Stages an operation for the given transaction, checking bounded capacity and byte budget.
    pub fn stage(&self, tx: TxId, op: IndexOp<T>) -> Result<()>
    where
        T: StagedOpSize,
    {
        self.stage_bounded(tx, op)
    }

    /// Stages an operation for the given transaction, enforcing `max_ops_per_tx` and byte limits.
    pub fn stage_bounded(&self, tx: TxId, op: IndexOp<T>) -> Result<()>
    where
        T: StagedOpSize,
    {
        let op_bytes = op.estimated_bytes();
        let shard_idx = self.shard_idx(tx);
        let mut shard = self.shards[shard_idx].write();

        let curr_tx_bytes = *shard.staged_bytes.entry(tx).or_insert(0);
        if curr_tx_bytes + op_bytes > self.config.max_tx_staged_bytes {
            return Err(ContextraError::Storage(format!(
                "Transaction {tx} staging budget exceeded: {} + {op_bytes} > {}",
                curr_tx_bytes, self.config.max_tx_staged_bytes
            )));
        }

        let curr_global = self.global_staged_bytes.load(Ordering::Relaxed);
        if curr_global + op_bytes > self.config.max_total_staged_bytes {
            return Err(ContextraError::Storage(format!(
                "Global staging budget exceeded: {curr_global} + {op_bytes} > {}",
                self.config.max_total_staged_bytes
            )));
        }

        let max_ops = self.config.max_ops_per_tx;
        let entry = shard
            .ops
            .entry(tx)
            .or_insert_with(|| (Vec::with_capacity(16), Instant::now()));

        if entry.0.len() >= max_ops {
            return Err(ContextraError::Transaction(format!(
                "Transaction {} exceeded max staging capacity ({} ops)",
                tx, max_ops
            )));
        }

        entry.0.push(op);
        shard.staged_bytes.entry(tx).and_modify(|b| *b += op_bytes);
        self.global_staged_bytes
            .fetch_add(op_bytes, Ordering::Relaxed);
        Ok(())
    }

    /// Stages multiple operations for a transaction in a single shard lock acquisition.
    pub fn stage_many(&self, tx: TxId, ops: impl IntoIterator<Item = IndexOp<T>>) -> Result<()>
    where
        T: StagedOpSize,
    {
        let ops_vec: Vec<_> = ops.into_iter().collect();
        let total_op_bytes: usize = ops_vec.iter().map(|op| op.estimated_bytes()).sum();

        let shard_idx = self.shard_idx(tx);
        let mut shard = self.shards[shard_idx].write();

        let curr_tx_bytes = *shard.staged_bytes.entry(tx).or_insert(0);
        if curr_tx_bytes + total_op_bytes > self.config.max_tx_staged_bytes {
            return Err(ContextraError::Storage(format!(
                "Transaction {tx} staging budget exceeded: {} + {total_op_bytes} > {}",
                curr_tx_bytes, self.config.max_tx_staged_bytes
            )));
        }

        let curr_global = self.global_staged_bytes.load(Ordering::Relaxed);
        if curr_global + total_op_bytes > self.config.max_total_staged_bytes {
            return Err(ContextraError::Storage(format!(
                "Global staging budget exceeded: {curr_global} + {total_op_bytes} > {}",
                self.config.max_total_staged_bytes
            )));
        }

        let max_ops = self.config.max_ops_per_tx;
        let entry = shard
            .ops
            .entry(tx)
            .or_insert_with(|| (Vec::new(), Instant::now()));

        if entry.0.len() + ops_vec.len() > max_ops {
            return Err(ContextraError::Transaction(format!(
                "Transaction {} exceeded max staging capacity ({} ops)",
                tx, max_ops
            )));
        }

        entry.0.extend(ops_vec);
        shard
            .staged_bytes
            .entry(tx)
            .and_modify(|b| *b += total_op_bytes);
        self.global_staged_bytes
            .fetch_add(total_op_bytes, Ordering::Relaxed);
        Ok(())
    }

    /// Validates that the transaction has pending operations.
    pub fn validate_pending_ops(&self, tx: TxId) -> Result<()> {
        let shard_idx = self.shard_idx(tx);
        let shard = self.shards[shard_idx].read();

        if let Some((ops, _)) = shard.ops.get(&tx) {
            if ops.is_empty() {
                return Err(ContextraError::Transaction(format!(
                    "Transaction {} was registered but has no pending operations",
                    tx
                )));
            }
        }
        Ok(())
    }

    /// Drains and returns all buffered operations for a transaction.
    ///
    /// Returns an empty vector if the transaction does not exist or has no operations.
    /// This operation is atomic per shard and cleans up tracked [`ReadSet`] and byte budgets.
    pub fn drain(&self, tx: TxId) -> Vec<IndexOp<T>> {
        let shard_idx = self.shard_idx(tx);
        let mut shard = self.shards[shard_idx].write();
        shard.read_sets.remove(&tx);
        let tx_bytes = shard.staged_bytes.remove(&tx).unwrap_or(0);
        if tx_bytes > 0 {
            self.global_staged_bytes
                .fetch_sub(tx_bytes, Ordering::Relaxed);
        }
        shard
            .ops
            .remove(&tx)
            .map(|(ops, _)| ops)
            .unwrap_or_default()
    }

    /// Discards all buffered operations and tracked read-sets for a transaction.
    pub fn discard(&self, tx: TxId) {
        let shard_idx = self.shard_idx(tx);
        let mut shard = self.shards[shard_idx].write();
        shard.read_sets.remove(&tx);
        let tx_bytes = shard.staged_bytes.remove(&tx).unwrap_or(0);
        if tx_bytes > 0 {
            self.global_staged_bytes
                .fetch_sub(tx_bytes, Ordering::Relaxed);
        }
        shard.ops.remove(&tx);
    }

    /// Returns the total number of pending transactions.
    ///
    /// WARNING: This iterates over all shards with individual read locks.
    /// The result is NOT an atomic snapshot of the buffer's size.
    /// Do not use this as a strict control metric for backpressure.
    pub fn len(&self) -> usize {
        self.shards.iter().map(|s| s.read().ops.len()).sum()
    }

    /// Returns true if all shards are empty.
    ///
    /// WARNING: Like `len()`, this is not an atomic operation across all shards.
    pub fn is_empty(&self) -> bool {
        self.shards.iter().all(|s| s.read().ops.is_empty())
    }

    /// Cleans up expired transactions relative to timestamp `now`.
    /// Reaps expired orphan transactions.
    ///
    /// # INVARIANT
    /// Acquires shard locks sequentially in ascending index order (0 to N-1),
    /// dropping each lock before attempting the next to guarantee deadlock-free execution.
    pub fn reap_orphans_at(&self, now: Instant) -> Vec<TxId> {
        self.reap_orphans_bounded_at(usize::MAX, now)
    }

    /// Cleans up expired transactions.
    /// Reaps expired orphan transactions.
    ///
    /// # INVARIANT
    /// Acquires shard locks sequentially in ascending index order (0 to N-1),
    /// dropping each lock before attempting the next to guarantee deadlock-free execution.
    pub fn reap_orphans(&self) -> Vec<TxId> {
        self.reap_orphans_at(Instant::now())
    }

    /// Reaps up to `max` expired orphan transactions across shards relative to timestamp `now`.
    ///
    /// # INVARIANT
    /// Acquires shard locks sequentially in ascending index order (0 to N-1),
    /// dropping each lock before attempting the next to guarantee deadlock-free execution.
    pub fn reap_orphans_bounded_at(&self, max: usize, now: Instant) -> Vec<TxId> {
        let mut expired = Vec::new();
        for shard_lock in &self.shards {
            if expired.len() >= max {
                break;
            }
            if let Some(mut shard) = shard_lock.try_write() {
                let tx_timeout = self.tx_timeout;
                let mut shard_expired = Vec::new();
                shard.ops.retain(|tx, (_, created)| {
                    if expired.len() + shard_expired.len() < max
                        && now.saturating_duration_since(*created) > tx_timeout
                    {
                        shard_expired.push(*tx);
                        false
                    } else {
                        true
                    }
                });
                for tx in &shard_expired {
                    shard.read_sets.remove(tx);
                    let tx_bytes = shard.staged_bytes.remove(tx).unwrap_or(0);
                    if tx_bytes > 0 {
                        self.global_staged_bytes
                            .fetch_sub(tx_bytes, Ordering::Relaxed);
                    }
                }
                expired.extend(shard_expired);
            }
            std::thread::yield_now();
        }
        expired
    }

    /// Reaps up to `max` expired orphan transactions across shards.
    ///
    /// # INVARIANT
    /// Acquires shard locks sequentially in ascending index order (0 to N-1),
    /// dropping each lock before attempting the next to guarantee deadlock-free execution.
    pub fn reap_orphans_bounded(&self, max: usize) -> Vec<TxId> {
        self.reap_orphans_bounded_at(max, Instant::now())
    }

    /// Returns a clone of the pending operations for a transaction.
    pub fn get_ops(&self, tx: TxId) -> Option<Vec<IndexOp<T>>> {
        let shard_idx = self.shard_idx(tx);
        let shard = self.shards[shard_idx].read();
        shard.ops.get(&tx).map(|(ops, _)| ops.clone())
    }

    /// Returns the transaction timeout.
    pub fn tx_timeout(&self) -> Duration {
        self.tx_timeout
    }
}

impl TxBuffer<(Vec<u8>, Vec<u8>)> {
    fn track_key_stage(&self, tx: TxId, key: &[u8], is_insert: bool) {
        let idx = self.key_shard_idx(key);
        let mut shard = self.key_shards[idx].write();
        shard
            .staged
            .entry(key.to_vec())
            .or_default()
            .insert(tx, is_insert);
    }

    fn untrack_key_ops_kv(&self, tx: TxId, ops: &[IndexOp<(Vec<u8>, Vec<u8>)>]) {
        for op in ops {
            let key = match op {
                IndexOp::Insert { data, .. } => &data.0,
                IndexOp::Delete { data: Some(d), .. } => &d.0,
                IndexOp::Delete { data: None, .. } => continue,
            };
            let idx = self.key_shard_idx(key);
            let mut shard = self.key_shards[idx].write();
            if let std::collections::hash_map::Entry::Occupied(mut entry) =
                shard.staged.entry(key.to_vec())
            {
                entry.get_mut().remove(&tx);
                if entry.get().is_empty() {
                    entry.remove();
                }
            }
        }
    }

    /// Stages a key-value operation and updates atomic key staging map.
    pub fn stage_kv(&self, tx: TxId, op: IndexOp<(Vec<u8>, Vec<u8>)>) -> Result<()> {
        let (key, is_insert) = match &op {
            IndexOp::Insert { data, .. } => (data.0.as_slice(), true),
            IndexOp::Delete { data: Some(d), .. } => (d.0.as_slice(), false),
            IndexOp::Delete { data: None, .. } => {
                return self.stage(tx, op);
            }
        };

        self.track_key_stage(tx, key, is_insert);
        let key_vec = key.to_vec();
        if let Err(e) = self.stage(tx, op) {
            // Roll back key tracking if staging fails capacity check
            let idx = self.key_shard_idx(&key_vec);
            let mut shard = self.key_shards[idx].write();
            if let std::collections::hash_map::Entry::Occupied(mut entry) =
                shard.staged.entry(key_vec)
            {
                entry.get_mut().remove(&tx);
                if entry.get().is_empty() {
                    entry.remove();
                }
            }
            return Err(e);
        }
        Ok(())
    }

    /// Drains and returns all buffered operations for a key-value transaction, cleaning up key staging.
    ///
    /// # INVARIANT (contextra-store Integration)
    /// Draining atomizes staging removal and clears the transaction's `ReadSet` and staging byte footprint.
    pub fn drain_kv(&self, tx: TxId) -> Vec<IndexOp<(Vec<u8>, Vec<u8>)>> {
        let ops = self.drain(tx);
        if !ops.is_empty() {
            self.untrack_key_ops_kv(tx, &ops);
        }
        ops
    }

    /// Discards all buffered operations for a key-value transaction, cleaning up key staging.
    pub fn discard_kv(&self, tx: TxId) {
        if let Some(ops) = self.get_ops(tx) {
            self.untrack_key_ops_kv(tx, &ops);
        }
        self.discard(tx);
    }

    /// Checks if `key` is staged for insertion in the specified transaction `tx_id`.
    ///
    /// This method is atomic because all operations for a given `tx_id` land in the same shard
    /// (since `shard_idx` is a pure function of `tx.inner()`), requiring only a single shard read-lock acquisition.
    /// Use this for Read-Your-Writes isolation decisions within a specific transaction scope.
    pub fn is_key_staged_for_tx(&self, tx_id: TxId, key: &[u8]) -> bool {
        let shard_idx = self.shard_idx(tx_id);
        let shard = self.shards[shard_idx].read();
        if let Some((ops, _)) = shard.ops.get(&tx_id) {
            for op in ops {
                if let IndexOp::Insert { data, .. } = op {
                    if data.0 == key {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Atomisch ermittelt den Staging-Status eines Keys über alle uncommitted Transaktionen.
    ///
    /// Gibt `Some(true)` für Insert, `Some(false)` für Delete, oder `None` zurück.
    pub fn staged_status(&self, key: &[u8]) -> Option<bool> {
        let idx = self.key_shard_idx(key);
        let shard = self.key_shards[idx].read();
        if let Some(map) = shard.staged.get(key) {
            let mut max_tx: Option<TxId> = None;
            let mut status: Option<bool> = None;
            for (&tx_id, &is_insert) in map {
                match max_tx {
                    None => {
                        max_tx = Some(tx_id);
                        status = Some(is_insert);
                    }
                    Some(m) if tx_id > m => {
                        max_tx = Some(tx_id);
                        status = Some(is_insert);
                    }
                    _ => {}
                }
            }
            return status;
        }
        None
    }
}

impl<T: Clone> Default for TxBuffer<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use proptest::{prop_assert, prop_assert_eq};
    use std::sync::Arc;

    #[test]
    fn test_tx_buffer_read_set_tracking() {
        let buffer = TxBuffer::<String>::new();
        let tx = TxId::new(10);
        buffer.begin(tx);

        let initial_rs = buffer.get_read_set(tx);
        assert!(initial_rs.as_ref().is_some_and(|rs| rs.is_empty()));

        buffer.register_read(tx, b"key_a".to_vec(), 100);
        buffer.record_read(tx, b"key_b".to_vec(), 105);

        let rs = buffer.read_set(tx).expect("read set present");
        assert_eq!(rs.len(), 2);
        assert_eq!(rs.get(b"key_a"), Some(100));
        assert_eq!(rs.get(b"key_b"), Some(105));

        // Drain cleans up read_set
        buffer.drain(tx);
        assert!(buffer.get_read_set(tx).is_none());
    }

    #[test]
    fn test_txbuffer_respects_max_ops_limit() {
        let config = TxBufferConfig {
            max_ops_per_tx: 3,
            ..Default::default()
        };
        let buffer = TxBuffer::<String>::new_with_config_ext(64, Duration::from_secs(5), config);
        let tx = TxId::new(1);
        buffer.begin(tx);
        // 3 operations succeed
        for i in 0..3 {
            let res = buffer.stage_bounded(
                tx,
                IndexOp::Insert {
                    doc_id: DocId::from(i as u64),
                    data: format!("data_{i}"),
                },
            );
            assert!(res.is_ok());
        }
        // 4th operation must fail
        let result = buffer.stage_bounded(
            tx,
            IndexOp::Insert {
                doc_id: DocId::from(99u64),
                data: "overflow".to_string(),
            },
        );
        assert!(matches!(result, Err(ContextraError::Transaction(_))));
    }

    #[test]
    fn test_txbuffer_default_config_allows_normal_workload() {
        let buffer = TxBuffer::<String>::new();
        let tx = TxId::new(42);
        buffer.begin(tx);
        for i in 0..500 {
            let res = buffer.stage(
                tx,
                IndexOp::Insert {
                    doc_id: DocId::from(i as u64),
                    data: format!("data_{i}"),
                },
            );
            assert!(res.is_ok());
        }
        assert_eq!(buffer.get_ops(tx).map(|v| v.len()), Some(500));
    }

    #[test]
    fn test_tx_buffer_stage_many_and_max_ops_constant() {
        let buffer = TxBuffer::<String>::new();
        let tx = TxId::new(42);
        assert_eq!(DEFAULT_MAX_OPS_PER_TX, 10_000);

        let ops = vec![
            IndexOp::Insert {
                doc_id: DocId::from(1u64),
                data: "op1".to_string(),
            },
            IndexOp::Insert {
                doc_id: DocId::from(2u64),
                data: "op2".to_string(),
            },
        ];

        assert!(buffer.stage_many(tx, ops).is_ok());
        assert!(buffer.has_tx(tx));
        let drained = buffer.drain(tx);
        assert_eq!(drained.len(), 2);
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_tx_buffer_stage_many_exceeds_capacity() {
        let config = TxBufferConfig {
            max_ops_per_tx: 2,
            ..Default::default()
        };
        let buffer = TxBuffer::<String>::new_with_config_ext(1, Duration::from_secs(5), config);
        let tx = TxId::new(10);

        let ops = vec![
            IndexOp::Insert {
                doc_id: DocId::from(1u64),
                data: "op1".to_string(),
            },
            IndexOp::Insert {
                doc_id: DocId::from(2u64),
                data: "op2".to_string(),
            },
            IndexOp::Insert {
                doc_id: DocId::from(3u64),
                data: "op3".to_string(),
            },
        ];

        let res = buffer.stage_many(tx, ops);
        assert!(matches!(res, Err(ContextraError::Transaction(_))));
    }

    #[test]
    fn test_tx_buffer_reap_orphans_bounded_capping() {
        let buffer = TxBuffer::<String>::new_with_config(1, Duration::from_millis(5));

        buffer.begin(TxId::new(1));
        buffer.begin(TxId::new(2));
        buffer.begin(TxId::new(3));

        std::thread::sleep(Duration::from_millis(20));

        let reaped = buffer.reap_orphans_bounded(2);
        assert_eq!(reaped.len(), 2);
        assert_eq!(buffer.len(), 1);
    }

    #[test]
    fn test_tx_buffer_discard() {
        let buffer = TxBuffer::<String>::new_with_config(64, Duration::from_secs(30));
        let tx = TxId::new(1);

        let _ = buffer.stage(
            tx,
            IndexOp::Insert {
                doc_id: DocId::from(1u64),
                data: "data1".to_string(),
            },
        );
        buffer.discard(tx);
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_concurrent_stage_no_data_loss() {
        let buffer = Arc::new(TxBuffer::<usize>::new_with_config(
            64,
            Duration::from_secs(30),
        ));
        let num_tx = 100;
        let ops_per_tx = 100;

        let mut handles = Vec::new();
        for t in 0..num_tx {
            let buffer = buffer.clone();
            handles.push(std::thread::spawn(move || {
                let tx = TxId::new(t as u64);
                buffer.begin(tx);
                for i in 0..ops_per_tx {
                    let _ = buffer.stage(
                        tx,
                        IndexOp::Insert {
                            doc_id: DocId::from(i as u64),
                            data: i,
                        },
                    );
                }
            }));
        }

        for h in handles {
            // INTENT: intentional expect in tests
            h.join().expect("task panicked"); // #[cfg(test)] // expect
        }

        assert_eq!(buffer.len(), num_tx);
        for t in 0..num_tx {
            let tx = TxId::new(t as u64);
            let ops = buffer.drain(tx);
            assert_eq!(ops.len(), ops_per_tx);
        }
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_tx_buffer_reap_orphans() {
        let buffer = TxBuffer::<String>::new_with_config(1, Duration::from_millis(10));
        let tx = TxId::new(1);
        buffer.begin(tx);

        // Wait for timeout
        std::thread::sleep(Duration::from_millis(20));

        let expired = buffer.reap_orphans();
        assert_eq!(expired, vec![tx]);
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_tx_buffer_reap_orphans_deterministic_time_injection() {
        let buffer = TxBuffer::<String>::new_with_config(1, Duration::from_secs(10));
        let tx = TxId::new(100);
        let now = Instant::now();
        let past = now.checked_sub(Duration::from_secs(15)).unwrap_or(now);

        // Begin transaction at 15 seconds in the past
        buffer.begin_at(tx, past);

        // Reaping at `now` (elapsed 15s > timeout 10s) must reap the orphan without real-time delay
        let expired = buffer.reap_orphans_at(now);
        assert_eq!(expired, vec![tx]);
        assert!(buffer.is_empty());
    }

    #[tokio::test]
    async fn test_has_tx_concurrent() {
        let buffer = Arc::new(TxBuffer::<String>::new());
        let num_tasks = 8;
        let mut handles = Vec::new();

        for i in 0..num_tasks {
            let buffer = buffer.clone();
            let tx = TxId::new(i as u64 + 100);
            handles.push(tokio::spawn(async move {
                buffer.begin(tx);
                let _ = buffer.stage(
                    tx,
                    IndexOp::Insert {
                        doc_id: DocId::from(i as u64),
                        data: format!("data_{i}"),
                    },
                );
                // Return tx ID for caller verification
                tx
            }));
        }

        let mut txs = Vec::new();
        for h in handles {
            let tx = h.await.expect("task failed"); // expect
            txs.push(tx);
        }

        // Verify has_tx returns true for all concurrently active transactions
        for &tx in &txs {
            assert!(buffer.has_tx(tx));
        }

        // Commit / drain all transactions and verify has_tx returns false
        for &tx in &txs {
            buffer.drain(tx);
            assert!(!buffer.has_tx(tx));
        }

        assert!(buffer.is_empty());
    }

    #[test]
    fn test_tx_buffer_validate_pending_ops() {
        let buffer = TxBuffer::<String>::new();
        let tx = TxId::new(1);

        // No tx yet
        assert!(buffer.validate_pending_ops(tx).is_ok());

        // Registered but empty
        buffer.begin(tx);
        assert!(buffer.validate_pending_ops(tx).is_err());

        // With ops
        let _ = buffer.stage(
            tx,
            IndexOp::Insert {
                doc_id: DocId::from(1u64),
                data: "s".to_string(),
            },
        );
        assert!(buffer.validate_pending_ops(tx).is_ok());
    }

    #[test]
    fn test_is_key_staged_for_tx_semantics() {
        let buffer = TxBuffer::<(Vec<u8>, Vec<u8>)>::new();
        let tx1 = TxId::new(1);
        let tx2 = TxId::new(2);

        buffer.begin(tx1);
        buffer.begin(tx2);

        let key_a = b"key_a".to_vec();
        let val_a = b"val_a".to_vec();

        buffer
            .stage_kv(
                tx1,
                IndexOp::Insert {
                    doc_id: DocId::from(100u64),
                    data: (key_a.clone(), val_a.clone()),
                },
            )
            .expect("stage tx1");

        // tx1 has key_a staged
        assert!(buffer.is_key_staged_for_tx(tx1, &key_a));
        // tx2 does NOT have key_a staged in its transaction scope
        assert!(!buffer.is_key_staged_for_tx(tx2, &key_a));
    }

    #[test]
    fn test_index_op_helpers() {
        let op = IndexOp::Insert {
            doc_id: DocId::from(1u64),
            data: "d",
        };
        assert_eq!(op.doc_id(), DocId::from(1u64));

        let op2 = IndexOp::Delete::<String> {
            doc_id: DocId::from(2u64),
            data: None,
        };
        assert_eq!(op2.doc_id(), DocId::from(2u64));
    }

    #[test]
    fn test_tx_buffer_config() {
        let timeout = Duration::from_secs(100);
        let buffer = TxBuffer::<u8>::new_with_config(4, timeout);
        assert_eq!(buffer.tx_timeout(), timeout);
        // Ensure we can use all shards
        for i in 0..10 {
            buffer.begin(TxId::new(i));
        }
    }

    #[test]
    fn test_tx_buffer_zero_shards_defaults_to_one() {
        // FIND-COR-001: shard_count=0 must not panic (§2 Zero-Panic)
        let buffer = TxBuffer::<u8>::new_with_config(0, Duration::from_secs(1));
        let tx = TxId::new(42);
        buffer.begin(tx);
        let _ = buffer.stage(
            tx,
            IndexOp::Insert {
                doc_id: DocId::from(1u64),
                data: 0,
            },
        );
        let ops = buffer.drain(tx);
        assert_eq!(ops.len(), 1);
        assert!(buffer.is_empty());
    }

    proptest::proptest! {
        #[test]
        fn prop_tx_buffer_isolation(
            tx_ids in proptest::collection::vec(0..u64::MAX, 1..50),
            shard_count in 1..256usize
        ) {
            let buffer = TxBuffer::<u64>::new_with_config(shard_count, Duration::from_secs(60));

            // 1. Stage values for all unique TXs
            for &id in &tx_ids {
                let tx = TxId::new(id);
                let _ = buffer.stage(tx, IndexOp::Insert { doc_id: DocId::from(id), data: id });
            }

            // 2. Verify each TX only has its own data
            for &id in &tx_ids {
                let tx = TxId::new(id);
                let ops = buffer.get_ops(tx).unwrap(); // unwrap
                for op in ops {
                    match op {
                        IndexOp::Insert { doc_id, data } => {
                            prop_assert_eq!(doc_id, DocId::from(id));
                            prop_assert_eq!(data, id);
                        },
                        _ => panic!("Unexpected op"),
                    }
                }
            }

            // 3. Drain and verify emptiness
            for &id in &tx_ids {
                let tx = TxId::new(id);
                buffer.drain(tx);
            }
            prop_assert!(buffer.is_empty());
        }

        #[test]
        fn prop_tx_buffer_reap_is_complete(
            tx_count in 1..100usize,
            timeout_ms in 1..100u64
        ) {
            let buffer = TxBuffer::<u8>::new_with_config(16, Duration::from_millis(timeout_ms));

            for i in 0..tx_count {
                buffer.begin(TxId::new(i as u64));
            }

            // Wait double the timeout
            std::thread::sleep(Duration::from_millis(timeout_ms * 2));

            let reaped = buffer.reap_orphans();
            prop_assert_eq!(reaped.len(), tx_count);
            prop_assert!(buffer.is_empty());
        }

        #[test]
        fn prop_tx_buffer_stage_drain_stage_lifecycle(
            tx_id_raw in 0..u64::MAX,
            first_ops in proptest::collection::vec(0..100u64, 1..20),
            second_ops in proptest::collection::vec(0..100u64, 1..20)
        ) {
            let buffer = TxBuffer::<u64>::new();
            let tx = TxId::new(tx_id_raw);

            // 1. Stage first batch
            for &val in &first_ops {
                let _ = buffer.stage(tx, IndexOp::Insert { doc_id: DocId::from(val), data: val });
            }

            // 2. Drain and verify matching first batch
            let drained1 = buffer.drain(tx);
            prop_assert_eq!(drained1.len(), first_ops.len());
            for (idx, op) in drained1.into_iter().enumerate() {
                match op {
                    IndexOp::Insert { doc_id, data } => {
                        prop_assert_eq!(doc_id, DocId::from(first_ops[idx]));
                        prop_assert_eq!(data, first_ops[idx]);
                    }
                    _ => panic!("Expected Insert"),
                }
            }

            // 3. Verify buffer is clean for this tx
            prop_assert!(buffer.get_ops(tx).is_none());
            prop_assert!(!buffer.has_tx(tx));

            // 4. Stage second batch
            for &val in &second_ops {
                let _ = buffer.stage(tx, IndexOp::Insert { doc_id: DocId::from(val), data: val });
            }

            // 5. Drain and verify matching second batch exactly (no ghost leakage)
            let drained2 = buffer.drain(tx);
            prop_assert_eq!(drained2.len(), second_ops.len());
            for (idx, op) in drained2.into_iter().enumerate() {
                match op {
                    IndexOp::Insert { doc_id, data } => {
                        prop_assert_eq!(doc_id, DocId::from(second_ops[idx]));
                        prop_assert_eq!(data, second_ops[idx]);
                    }
                    _ => panic!("Expected Insert"),
                }
            }
            prop_assert!(buffer.is_empty());
        }

        #[test]
        fn prop_tx_buffer_partial_discard_isolation(
            tx_ids in proptest::collection::vec(0..u64::MAX, 2..40),
            discard_indices in proptest::collection::vec(0..100usize, 1..20)
        ) {
            let buffer = TxBuffer::<u64>::new();

            // Setup unique tx ids and stage one op each
            let mut unique_txs = tx_ids;
            unique_txs.sort_unstable();
            unique_txs.dedup();
            if unique_txs.len() < 2 {
                return Ok(()); // Skip trivial cases
            }

            for &id in &unique_txs {
                let tx = TxId::new(id);
                let _ = buffer.stage(tx, IndexOp::Insert { doc_id: DocId::from(id), data: id });
            }

            // Determine which to discard
            let mut to_discard = std::collections::HashSet::new();
            for seed in discard_indices {
                let idx = seed % unique_txs.len();
                to_discard.insert(unique_txs[idx]);
            }

            // Discard selected
            for &id in &to_discard {
                buffer.discard(TxId::new(id));
            }

            // Verify isolated status: discarded are gone, others remain intact
            for &id in &unique_txs {
                let tx = TxId::new(id);
                if to_discard.contains(&id) {
                    prop_assert!(!buffer.has_tx(tx));
                    prop_assert!(buffer.get_ops(tx).is_none());
                } else {
                    prop_assert!(buffer.has_tx(tx));
                    let ops = buffer.get_ops(tx).unwrap(); // unwrap
                    prop_assert_eq!(ops.len(), 1);
                    match &ops[0] {
                        IndexOp::Insert { doc_id, data } => {
                            prop_assert_eq!(*doc_id, DocId::from(id));
                            prop_assert_eq!(*data, id);
                        }
                        _ => panic!("Expected Insert"),
                    }
                }
            }
        }
    }
}
