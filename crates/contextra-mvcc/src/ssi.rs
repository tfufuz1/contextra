//! Serializable Snapshot Isolation (SSI) read-set tracking and conflict validation.
//!
//! # Architecture Role (Ring 0)
//!
//! Provides in-memory SSI conflict detection for transactions:
//! - [`ReadSet`]: Tracks read keys, range prefixes, and their snapshot sequence numbers per transaction.
//! - [`SsiValidator`]: Trait for checking concurrent write conflicts against a transaction's `ReadSet`.
//! - [`SequenceLogSsiValidator`]: Reference validator implementation using MVCC sequence tracking.

// FILE-CONTEXT
// STAND: 2026-10-15T00:00:00Z
// ZWECK: In-memory SSI Read-Set Tracking, Write-Skew-Konfliktvalidierung, Fail-Closed Pruning und atomare Commit-Registrierung.
// INVARIANTEN: INV-MVCC-SSI-1: Konfliktauflösung ist rein datengetrieben (Sequenznummernvergleich) und deterministisch.
// HOTSPOTS: 35-220
// NICHT-OFFENSICHTLICH: Pruning unter `committed_writes` Schreib-Lock garantiert atomaren Wasserstand. Fail-closed Protection verweigert Reads mit snapshot_seq < pruned_through.

use crate::error::{ContextraError, Result};
use crate::seq_log::{SequenceLog, DEFAULT_MAX_PIN_DURATION_TICKS};
use crate::snapshot::SnapshotRegistry;
use crate::sync::atomic::{AtomicU64, Ordering};
use crate::sync::Arc;
use crate::types::{DocId, TenantId, TxId};
use ahash::AHashMap;
use parking_lot::RwLock;
use std::collections::BTreeMap;
use std::hash::Hasher;
use std::time::Duration;

/// Default maximum tracked committed write keys in [`SequenceLogSsiValidator`] (1,000,000 keys).
pub const DEFAULT_MAX_TRACKED_COMMIT_KEYS: usize = 1_000_000;

/// Tracked read keys, range prefixes, and their snapshot sequence numbers for Serializable Snapshot Isolation (SSI).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReadSet {
    keys: AHashMap<Vec<u8>, u64>,
    prefixes: AHashMap<Vec<u8>, u64>,
    min_snapshot_seq: Option<u64>,
}

impl ReadSet {
    /// Creates a new empty [`ReadSet`].
    pub fn new() -> Self {
        Self {
            keys: AHashMap::default(),
            prefixes: AHashMap::default(),
            min_snapshot_seq: None,
        }
    }

    /// Records a read operation for `key` at snapshot sequence `snapshot_seq`.
    pub fn record_read(&mut self, key: impl Into<Vec<u8>>, snapshot_seq: u64) {
        let snapshot_seq = snapshot_seq & !crate::types::TOMBSTONE_BIT;
        let k = key.into();
        self.keys
            .entry(k)
            .and_modify(|existing_seq| {
                if snapshot_seq < *existing_seq {
                    *existing_seq = snapshot_seq;
                }
            })
            .or_insert(snapshot_seq);

        self.min_snapshot_seq = Some(
            self.min_snapshot_seq
                .map_or(snapshot_seq, |curr| curr.min(snapshot_seq)),
        );
    }

    /// Records a range prefix read operation at `snapshot_seq` for phantom protection.
    pub fn record_prefix(&mut self, prefix: impl Into<Vec<u8>>, snapshot_seq: u64) {
        let snapshot_seq = snapshot_seq & !crate::types::TOMBSTONE_BIT;
        let p = prefix.into();
        self.prefixes
            .entry(p)
            .and_modify(|existing_seq| {
                if snapshot_seq < *existing_seq {
                    *existing_seq = snapshot_seq;
                }
            })
            .or_insert(snapshot_seq);

        self.min_snapshot_seq = Some(
            self.min_snapshot_seq
                .map_or(snapshot_seq, |curr| curr.min(snapshot_seq)),
        );
    }

    /// Alias for [`ReadSet::record_read`].
    pub fn register_read(&mut self, key: impl Into<Vec<u8>>, snapshot_seq: u64) {
        self.record_read(key, snapshot_seq);
    }

    /// Returns the snapshot sequence number recorded for `key`, if present.
    pub fn get(&self, key: &[u8]) -> Option<u64> {
        self.keys.get(key).copied()
    }

    /// Returns `true` if no read keys or range prefixes are tracked in this set.
    pub fn is_empty(&self) -> bool {
        self.keys_map().is_empty() && self.prefixes_map().is_empty()
    }

    /// Returns the total number of read keys and range prefixes tracked in this set.
    pub fn len(&self) -> usize {
        self.keys_map().len() + self.prefixes_map().len()
    }

    /// Clears all tracked read keys, range prefixes, and resets the cached minimum snapshot sequence.
    pub fn clear(&mut self) {
        self.keys.clear();
        self.prefixes.clear();
        self.min_snapshot_seq = None;
    }

    /// Returns the lowest snapshot sequence number recorded across all read keys and prefixes in this set,
    /// or `None` if the set is empty.
    pub fn min_snapshot_seq(&self) -> Option<u64> {
        self.min_snapshot_seq
    }

    /// Returns an iterator over tracked read keys and their snapshot sequence numbers.
    pub fn iter(&self) -> impl Iterator<Item = (&Vec<u8>, &u64)> {
        self.keys_map().iter()
    }

    /// Returns an iterator over tracked range prefixes and their snapshot sequence numbers.
    pub fn prefixes_iter(&self) -> impl Iterator<Item = (&Vec<u8>, &u64)> {
        self.prefixes_map().iter()
    }

    /// Returns the inner map of read keys to snapshot sequence numbers.
    pub fn keys_map(&self) -> &AHashMap<Vec<u8>, u64> {
        &self.keys
    }

    /// Returns the inner map of range prefixes to snapshot sequence numbers.
    pub fn prefixes_map(&self) -> &AHashMap<Vec<u8>, u64> {
        &self.prefixes
    }
}

/// Trait for Serializable Snapshot Isolation (SSI) validation.
pub trait SsiValidator {
    /// Validates the [`ReadSet`] for transaction `tx_id`.
    fn validate(&self, tx_id: TxId, read_set: &ReadSet) -> Result<()>;
}

/// Represents a sequence bucket storing either exact committed write keys or coarsened prefix summaries.
#[derive(Debug, Clone, PartialEq, Eq)]
enum SeqBucket {
    Exact(Vec<Vec<u8>>),
    Coarsened {
        prefixes: Vec<Vec<u8>>,
        original_key_count: usize,
    },
}

/// Diagnostic information identifying an unreleased snapshot or sequence that blocks pruning in [`SequenceLogSsiValidator`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PruningBlockerInfo {
    /// The lowest sequence number currently unpruned in the validator register.
    pub min_unpruned_seq: u64,
    /// The current `pruned_through` watermark sequence number.
    pub pruned_through_seq: u64,
    /// Total tracked committed keys currently retained.
    pub tracked_commit_keys: usize,
    /// Total sequence buckets currently retained.
    pub total_seq_buckets: usize,
    /// Number of coarsened sequence buckets.
    pub coarsened_seq_buckets: usize,
    /// Configured maximum tracked keys capacity limit.
    pub max_tracked_keys: usize,
    /// Sequence number of the longest active snapshot pinning sequence, if any.
    pub longest_active_snapshot_seq: Option<u64>,
    /// Pin duration of the longest active snapshot, if any.
    pub longest_pin_duration: Option<Duration>,
    /// True if the longest active snapshot pin duration exceeds `max_pin_duration` (e.g. 300s).
    pub is_pin_expired: bool,
}

fn coarsen_keys_to_prefixes(keys: &[Vec<u8>], max_prefixes: usize) -> Vec<Vec<u8>> {
    if keys.is_empty() {
        return Vec::new();
    }
    if keys.len() <= max_prefixes {
        return keys.to_vec();
    }

    let mut sorted_keys = keys.to_vec();
    sorted_keys.sort_unstable();

    let chunk_size = sorted_keys.len().div_ceil(max_prefixes);
    let mut prefixes = Vec::with_capacity(max_prefixes);

    for chunk in sorted_keys.chunks(chunk_size) {
        if chunk.is_empty() {
            continue;
        }
        let mut lcp = chunk[0].clone();
        for k in &chunk[1..] {
            let common_len = lcp.iter().zip(k.iter()).take_while(|(a, b)| a == b).count();
            lcp.truncate(common_len);
            if lcp.is_empty() {
                break;
            }
        }
        if !prefixes.contains(&lcp) {
            prefixes.push(lcp);
        }
    }

    prefixes
}

#[derive(Debug, Default)]
struct CommittedWrites {
    keys: AHashMap<Vec<u8>, u64>,
    seq_index: BTreeMap<u64, SeqBucket>,
}

impl CommittedWrites {
    fn tracked_count(&self) -> usize {
        self.keys.len() + self.coarsened_prefix_count()
    }

    fn coarsened_prefix_count(&self) -> usize {
        let mut count = 0;
        for bucket in self.seq_index.values() {
            if let SeqBucket::Coarsened { prefixes, .. } = bucket {
                count += prefixes.len();
            }
        }
        count
    }

    fn insert(&mut self, key: Vec<u8>, commit_seq: u64) {
        let commit_seq = commit_seq & !crate::types::TOMBSTONE_BIT;
        self.keys.insert(key.clone(), commit_seq);

        match self.seq_index.entry(commit_seq) {
            std::collections::btree_map::Entry::Vacant(e) => {
                e.insert(SeqBucket::Exact(vec![key]));
            }
            std::collections::btree_map::Entry::Occupied(mut e) => match e.get_mut() {
                SeqBucket::Exact(keys) => {
                    if !keys.contains(&key) {
                        keys.push(key);
                    }
                }
                SeqBucket::Coarsened {
                    original_key_count, ..
                } => {
                    *original_key_count += 1;
                }
            },
        }
    }

    fn coarsen_oldest_buckets(&mut self) -> usize {
        let mut coarsened_count = 0;
        let seqs: Vec<u64> = self.seq_index.keys().copied().collect();

        for seq in seqs {
            if let Some(SeqBucket::Exact(keys)) = self.seq_index.get(&seq) {
                let original_keys = keys.clone();
                let prefixes = coarsen_keys_to_prefixes(&original_keys, 2);

                for k in &original_keys {
                    if self.keys.get(k) == Some(&seq) {
                        self.keys.remove(k);
                    }
                }

                self.seq_index.insert(
                    seq,
                    SeqBucket::Coarsened {
                        prefixes,
                        original_key_count: original_keys.len(),
                    },
                );
                coarsened_count += 1;
            }
        }

        coarsened_count
    }

    fn merge_adjacent_buckets(&mut self) -> usize {
        if self.seq_index.len() < 2 {
            return 0;
        }

        let seqs: Vec<u64> = self.seq_index.keys().copied().collect();
        let mut merged_count = 0;

        for chunk in seqs.chunks(2) {
            if chunk.len() == 2 {
                let s1 = chunk[0];
                let s2 = chunk[1];

                if let (Some(b1), Some(b2)) =
                    (self.seq_index.remove(&s1), self.seq_index.remove(&s2))
                {
                    let mut prefixes = Vec::new();
                    let mut total_keys = 0;

                    for (b, seq) in [(b1, s1), (b2, s2)] {
                        match b {
                            SeqBucket::Exact(keys) => {
                                for k in &keys {
                                    if self.keys.get(k) == Some(&seq) {
                                        self.keys.remove(k);
                                    }
                                }
                                total_keys += keys.len();
                                prefixes.extend(coarsen_keys_to_prefixes(&keys, 2));
                            }
                            SeqBucket::Coarsened {
                                prefixes: p,
                                original_key_count,
                            } => {
                                total_keys += original_key_count;
                                prefixes.extend(p);
                            }
                        }
                    }

                    let coarsened_prefixes = coarsen_keys_to_prefixes(&prefixes, 2);
                    self.seq_index.insert(
                        s2,
                        SeqBucket::Coarsened {
                            prefixes: coarsened_prefixes,
                            original_key_count: total_keys,
                        },
                    );
                    merged_count += 1;
                }
            }
        }

        merged_count
    }

    fn remove_from_seq(&mut self, first_seq: u64) -> usize {
        let first_seq = first_seq & !crate::types::TOMBSTONE_BIT;
        let mut removed = 0;
        let mut seqs_to_remove = Vec::new();

        for (&seq, bucket) in self.seq_index.range(first_seq..) {
            seqs_to_remove.push(seq);
            match bucket {
                SeqBucket::Exact(keys) => {
                    for key in keys {
                        if self.keys.get(key) == Some(&seq) {
                            let mut highest_old_seq = None;
                            for (&old_seq, old_bucket) in self.seq_index.range(..first_seq).rev() {
                                if let SeqBucket::Exact(old_keys) = old_bucket {
                                    if old_keys.contains(key) {
                                        highest_old_seq = Some(old_seq);
                                        break;
                                    }
                                }
                            }
                            if let Some(old_seq) = highest_old_seq {
                                self.keys.insert(key.clone(), old_seq);
                            } else {
                                self.keys.remove(key);
                            }
                            removed += 1;
                        }
                    }
                }
                SeqBucket::Coarsened {
                    original_key_count, ..
                } => {
                    removed += *original_key_count;
                }
            }
        }

        for seq in seqs_to_remove {
            self.seq_index.remove(&seq);
        }

        removed
    }

    fn prune_through(&mut self, bound_seq: u64) -> usize {
        let bound_seq = bound_seq & !crate::types::TOMBSTONE_BIT;
        let mut removed = 0;
        let mut seqs_to_remove = Vec::new();

        for (&seq, bucket) in self.seq_index.range(..=bound_seq) {
            seqs_to_remove.push(seq);
            match bucket {
                SeqBucket::Exact(keys) => {
                    for key in keys {
                        if self.keys.get(key) == Some(&seq) && self.keys.remove(key).is_some() {
                            removed += 1;
                        }
                    }
                }
                SeqBucket::Coarsened {
                    original_key_count, ..
                } => {
                    removed += *original_key_count;
                }
            }
        }

        for seq in seqs_to_remove {
            self.seq_index.remove(&seq);
        }

        removed
    }

    fn len(&self) -> usize {
        self.tracked_count()
    }

    fn get(&self, key: &[u8]) -> Option<u64> {
        self.keys.get(key).copied()
    }
}

/// Reference implementation of [`SsiValidator`] backed by MVCC [`SequenceLog`] and committed key tracking.
#[derive(Clone)]
pub struct SequenceLogSsiValidator {
    committed_writes: Arc<RwLock<CommittedWrites>>,
    sequence_log: Option<Arc<RwLock<SequenceLog>>>,
    snapshot_registry: Option<Arc<SnapshotRegistry>>,
    pruned_through: Arc<AtomicU64>,
    max_tracked_keys: usize,
    max_pin_duration: Duration,
    metrics_sink: Option<Arc<parking_lot::RwLock<Arc<dyn contextra_ports::MetricsSink>>>>,
}

impl std::fmt::Debug for SequenceLogSsiValidator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SequenceLogSsiValidator")
            .field("committed_writes", &self.committed_writes)
            .field("sequence_log", &self.sequence_log)
            .field("snapshot_registry", &self.snapshot_registry)
            .field("pruned_through", &self.pruned_through)
            .field("max_tracked_keys", &self.max_tracked_keys)
            .field("max_pin_duration", &self.max_pin_duration)
            .field("has_metrics_sink", &self.metrics_sink.is_some())
            .finish()
    }
}

impl Default for SequenceLogSsiValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for constructing a [`SequenceLogSsiValidator`].
#[derive(Default)]
pub struct SequenceLogSsiValidatorBuilder {
    max_tracked_keys: Option<usize>,
    sequence_log: Option<Arc<RwLock<SequenceLog>>>,
    snapshot_registry: Option<Arc<SnapshotRegistry>>,
    max_pin_duration: Option<Duration>,
    metrics_sink: Option<Arc<parking_lot::RwLock<Arc<dyn contextra_ports::MetricsSink>>>>,
}

impl std::fmt::Debug for SequenceLogSsiValidatorBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SequenceLogSsiValidatorBuilder")
            .field("max_tracked_keys", &self.max_tracked_keys)
            .field("sequence_log", &self.sequence_log)
            .field("snapshot_registry", &self.snapshot_registry)
            .field("max_pin_duration", &self.max_pin_duration)
            .field("has_metrics_sink", &self.metrics_sink.is_some())
            .finish()
    }
}

impl SequenceLogSsiValidatorBuilder {
    /// Creates a new builder instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Configures the maximum tracked keys bound.
    pub fn with_max_tracked_keys(mut self, limit: usize) -> Self {
        self.max_tracked_keys = Some(limit);
        self
    }

    /// Attaches an existing [`SequenceLog`].
    pub fn with_sequence_log(mut self, seq_log: Arc<RwLock<SequenceLog>>) -> Self {
        self.sequence_log = Some(seq_log);
        self
    }

    /// Attaches an active [`SnapshotRegistry`].
    pub fn with_snapshot_registry(mut self, registry: Arc<SnapshotRegistry>) -> Self {
        self.snapshot_registry = Some(registry);
        self
    }

    /// Sets a custom maximum pin duration.
    pub fn with_max_pin_duration(mut self, duration: Duration) -> Self {
        self.max_pin_duration = Some(duration);
        self
    }

    /// Attaches a metrics sink container.
    pub fn with_metrics_sink(
        mut self,
        sink: Arc<parking_lot::RwLock<Arc<dyn contextra_ports::MetricsSink>>>,
    ) -> Self {
        self.metrics_sink = Some(sink);
        self
    }

    /// Builds the configured [`SequenceLogSsiValidator`].
    pub fn build(self) -> SequenceLogSsiValidator {
        let max_keys = self
            .max_tracked_keys
            .unwrap_or(DEFAULT_MAX_TRACKED_COMMIT_KEYS);
        let mut validator = if let Some(seq_log) = self.sequence_log {
            SequenceLogSsiValidator::with_sequence_log(seq_log)
        } else {
            SequenceLogSsiValidator::new_with_bounds(max_keys)
        };
        validator.max_tracked_keys = max_keys;
        if let Some(registry) = self.snapshot_registry {
            validator = validator.with_snapshot_registry(registry);
        }
        if let Some(duration) = self.max_pin_duration {
            validator = validator.with_max_pin_duration(duration);
        }
        if let Some(sink) = self.metrics_sink {
            validator = validator.with_metrics_sink(sink);
        }
        validator
    }
}

impl SequenceLogSsiValidator {
    /// Creates a builder to configure and instantiate a [`SequenceLogSsiValidator`].
    pub fn builder() -> SequenceLogSsiValidatorBuilder {
        SequenceLogSsiValidatorBuilder::new()
    }

    /// Creates a new [`SequenceLogSsiValidator`] with default maximum commit key capacity (1,000,000 keys).
    pub fn new() -> Self {
        Self::new_with_bounds(DEFAULT_MAX_TRACKED_COMMIT_KEYS)
    }

    /// Creates a new [`SequenceLogSsiValidator`] with specified maximum commit key bounds `max_tracked_keys`.
    pub fn new_with_bounds(max_tracked_keys: usize) -> Self {
        Self {
            committed_writes: Arc::new(RwLock::new(CommittedWrites::default())),
            sequence_log: None,
            snapshot_registry: None,
            pruned_through: Arc::new(AtomicU64::new(0)),
            max_tracked_keys,
            max_pin_duration: Duration::from_nanos(DEFAULT_MAX_PIN_DURATION_TICKS),
            metrics_sink: None,
        }
    }

    /// Creates a new [`SequenceLogSsiValidator`] attached to an existing [`SequenceLog`].
    pub fn with_sequence_log(seq_log: Arc<RwLock<SequenceLog>>) -> Self {
        Self {
            committed_writes: Arc::new(RwLock::new(CommittedWrites::default())),
            sequence_log: Some(seq_log),
            snapshot_registry: None,
            pruned_through: Arc::new(AtomicU64::new(0)),
            max_tracked_keys: DEFAULT_MAX_TRACKED_COMMIT_KEYS,
            max_pin_duration: Duration::from_nanos(DEFAULT_MAX_PIN_DURATION_TICKS),
            metrics_sink: None,
        }
    }

    /// Attaches an active [`SnapshotRegistry`] to this validator for active snapshot pin duration tracking.
    pub fn with_snapshot_registry(mut self, registry: Arc<SnapshotRegistry>) -> Self {
        self.set_snapshot_registry(registry);
        self
    }

    /// Attaches an active [`SnapshotRegistry`] to this validator for active snapshot pin duration tracking.
    pub fn set_snapshot_registry(&mut self, registry: Arc<SnapshotRegistry>) {
        self.snapshot_registry = Some(registry);
    }

    /// Attaches a metrics sink container to this validator for observability reporting.
    pub fn with_metrics_sink(
        mut self,
        sink: Arc<parking_lot::RwLock<Arc<dyn contextra_ports::MetricsSink>>>,
    ) -> Self {
        self.metrics_sink = Some(sink);
        self
    }

    /// Sets or updates the metrics sink container attached to this validator.
    pub fn set_metrics_sink(
        &mut self,
        sink: Arc<parking_lot::RwLock<Arc<dyn contextra_ports::MetricsSink>>>,
    ) {
        self.metrics_sink = Some(sink);
    }

    /// Sets a custom maximum pin duration before diagnostic alarms are triggered for unreleased snapshots.
    pub fn with_max_pin_duration(mut self, duration: Duration) -> Self {
        self.max_pin_duration = duration;
        if let Some(ref seq_log) = self.sequence_log {
            seq_log.write().set_max_pin_duration(duration);
        }
        self
    }

    /// Prunes tracked committed writes with `commit_seq <= bound_seq`.
    pub fn prune_through(&self, bound_seq: u64) -> usize {
        let bound_seq = bound_seq & !crate::types::TOMBSTONE_BIT;
        let mut writes = self.committed_writes.write();
        let removed = writes.prune_through(bound_seq);
        self.pruned_through.fetch_max(bound_seq, Ordering::SeqCst);
        removed
    }

    /// Removes all committed key registrations with `commit_seq >= first_seq`.
    pub fn forget_from(&self, first_seq: u64) -> usize {
        let first_seq = first_seq & !crate::types::TOMBSTONE_BIT;
        let mut writes = self.committed_writes.write();
        writes.remove_from_seq(first_seq)
    }

    /// Returns the current `pruned_through` watermark sequence number.
    pub fn pruned_through(&self) -> u64 {
        self.pruned_through.load(Ordering::SeqCst)
    }

    /// Returns the number of committed keys currently tracked.
    pub fn tracked_commit_keys(&self) -> usize {
        self.committed_writes.read().len()
    }

    /// Returns the configured maximum commit key tracking limit.
    pub fn max_tracked_keys(&self) -> usize {
        self.max_tracked_keys
    }

    /// Checks register capacity utilization and triggers coarsening or diagnostic warnings if threshold is reached.
    fn check_and_coarsen_if_needed(&self, writes: &mut CommittedWrites) {
        let threshold = (self.max_tracked_keys * 80) / 100;
        let initial_len = writes.len();
        let reached_threshold = initial_len >= threshold;

        let mut rounds = 0;
        while writes.len() >= threshold && rounds < 10 {
            let coarsened = writes.coarsen_oldest_buckets();
            let merged = writes.merge_adjacent_buckets();
            rounds += 1;
            if coarsened == 0 && merged == 0 {
                break;
            }
        }

        let now_ticks = self
            .snapshot_registry
            .as_ref()
            .map_or(0, |reg| reg.now_ticks());

        if let Some(ref sink_lock) = self.metrics_sink {
            let sink = sink_lock.read().clone();
            let ratio = initial_len as f64 / self.max_tracked_keys as f64;
            sink.record_gauge("mvcc_ssi_commit_register_utilization_ratio", ratio, &[]);

            if reached_threshold {
                if let Some(ref blocker) = self.diagnose_pruning_blocker_internal(writes, now_ticks)
                {
                    sink.record_gauge(
                        "mvcc_ssi_coarsened_buckets_total",
                        blocker.coarsened_seq_buckets as f64,
                        &[],
                    );
                    if let Some(dur) = blocker.longest_pin_duration {
                        sink.record_gauge(
                            "mvcc_ssi_longest_pin_duration_seconds",
                            dur.as_secs_f64(),
                            &[],
                        );
                    }
                }
            }
        }

        if writes.len() >= threshold {
            if let Some(blocker) = self.diagnose_pruning_blocker_internal(writes, now_ticks) {
                tracing::warn!(
                    min_unpruned_seq = blocker.min_unpruned_seq,
                    pruned_through_seq = blocker.pruned_through_seq,
                    tracked_commit_keys = blocker.tracked_commit_keys,
                    total_seq_buckets = blocker.total_seq_buckets,
                    coarsened_seq_buckets = blocker.coarsened_seq_buckets,
                    max_tracked_keys = blocker.max_tracked_keys,
                    longest_active_snapshot_seq = ?blocker.longest_active_snapshot_seq,
                    longest_pin_duration = ?blocker.longest_pin_duration,
                    is_pin_expired = blocker.is_pin_expired,
                    "SequenceLogSsiValidator capacity utilization reached threshold (>= 80%). Coarsening active. Pruning may be blocked by active snapshot or unpruned sequence."
                );
            }
        }
    }

    /// Diagnoses which unpruned sequence or active snapshot is currently blocking pruning in this validator.
    pub fn diagnose_pruning_blocker(&self) -> Option<PruningBlockerInfo> {
        let now_ticks = self
            .snapshot_registry
            .as_ref()
            .map_or(0, |reg| reg.now_ticks());
        self.diagnose_pruning_blocker_at(now_ticks)
    }

    /// Diagnoses which unpruned sequence or active snapshot is currently blocking pruning relative to timestamp `now_ticks`.
    pub fn diagnose_pruning_blocker_at(&self, now_ticks: u64) -> Option<PruningBlockerInfo> {
        let writes = self.committed_writes.read();
        self.diagnose_pruning_blocker_internal(&writes, now_ticks)
    }

    fn diagnose_pruning_blocker_internal(
        &self,
        writes: &CommittedWrites,
        now_ticks: u64,
    ) -> Option<PruningBlockerInfo> {
        let min_unpruned_seq = writes.seq_index.keys().copied().next()?;
        let total_seq_buckets = writes.seq_index.len();
        let coarsened_seq_buckets = writes
            .seq_index
            .values()
            .filter(|b| matches!(b, SeqBucket::Coarsened { .. }))
            .count();

        let longest_pin = self
            .snapshot_registry
            .as_ref()
            .and_then(|reg| reg.longest_active_pin_at(now_ticks));

        let longest_active_snapshot_seq = longest_pin.map(|(seq, _)| seq);
        let longest_pin_duration = longest_pin.map(|(_, dur)| dur);
        let is_pin_expired = longest_pin.is_some_and(|(_, dur)| dur > self.max_pin_duration);

        Some(PruningBlockerInfo {
            min_unpruned_seq,
            pruned_through_seq: self.pruned_through(),
            tracked_commit_keys: writes.tracked_count(),
            total_seq_buckets,
            coarsened_seq_buckets,
            max_tracked_keys: self.max_tracked_keys,
            longest_active_snapshot_seq,
            longest_pin_duration,
            is_pin_expired,
        })
    }

    /// Records a committed write for `key` at sequence number `commit_seq`.
    pub fn record_commit_key(&self, key: &[u8], commit_seq: u64) {
        self.record_commit_keys(Some(key), commit_seq);
    }

    /// Records committed writes for multiple keys at sequence number `commit_seq`.
    pub fn record_commit_keys<'a>(
        &self,
        keys: impl IntoIterator<Item = &'a [u8]>,
        commit_seq: u64,
    ) {
        let commit_seq = commit_seq & !crate::types::TOMBSTONE_BIT;
        let mut writes = self.committed_writes.write();
        for key in keys {
            writes.insert(key.to_vec(), commit_seq);
        }
        self.check_and_coarsen_if_needed(&mut writes);
    }

    /// Validates `read_set` against `committed_writes` map and `sequence_log`.
    fn validate_internal(
        &self,
        committed_writes: &CommittedWrites,
        tx_id: TxId,
        read_set: &ReadSet,
    ) -> Result<()> {
        if read_set.is_empty() {
            return Ok(());
        }

        if committed_writes.len() >= self.max_tracked_keys {
            let now_ticks = self
                .snapshot_registry
                .as_ref()
                .map_or(0, |reg| reg.now_ticks());
            let blocker_msg = if let Some(blocker) =
                self.diagnose_pruning_blocker_internal(committed_writes, now_ticks)
            {
                format!(
                    "; pruning blocker details: min_unpruned_seq={}, longest_pin_duration={:?}, is_pin_expired={}",
                    blocker.min_unpruned_seq,
                    blocker.longest_pin_duration,
                    blocker.is_pin_expired
                )
            } else {
                String::new()
            };
            return Err(ContextraError::Conflict(format!(
                "Serializable isolation validation failed for TxId({}): commit key capacity limit reached ({} >= {}){}",
                tx_id.inner(),
                committed_writes.len(),
                self.max_tracked_keys,
                blocker_msg
            )));
        }

        let pruned_through = self.pruned_through.load(Ordering::SeqCst);

        let mut entries: Vec<(&Vec<u8>, &u64)> = read_set.iter().collect();
        entries.sort_unstable_by(|(k1, _), (k2, _)| k1.cmp(k2));

        for (key, &snapshot_seq) in entries {
            if snapshot_seq < pruned_through {
                return Err(ContextraError::Conflict(format!(
                    "Serializable isolation violation for TxId({}): snapshot too old (snapshot_seq {} < pruned_through {}) for key '{:?}'",
                    tx_id.inner(),
                    snapshot_seq,
                    pruned_through,
                    String::from_utf8_lossy(key)
                )));
            }

            if let Some(commit_seq) = committed_writes.get(key) {
                if commit_seq > snapshot_seq {
                    return Err(ContextraError::Conflict(format!(
                        "Serializable isolation violation for TxId({}): key '{:?}' modified at commit_seq {} > snapshot_seq {}",
                        tx_id.inner(),
                        String::from_utf8_lossy(key),
                        commit_seq,
                        snapshot_seq
                    )));
                }
            }

            for (&commit_seq, bucket) in &committed_writes.seq_index {
                if commit_seq > snapshot_seq {
                    if let SeqBucket::Coarsened { prefixes, .. } = bucket {
                        for p in prefixes {
                            if key.starts_with(p) {
                                return Err(ContextraError::Conflict(format!(
                                    "Serializable isolation violation for TxId({}): key '{:?}' matches coarsened commit prefix '{:?}' at commit_seq {} > snapshot_seq {}",
                                    tx_id.inner(),
                                    String::from_utf8_lossy(key),
                                    String::from_utf8_lossy(p),
                                    commit_seq,
                                    snapshot_seq
                                )));
                            }
                        }
                    }
                }
            }

            if let Some(ref seq_log_lock) = self.sequence_log {
                let seq_log = seq_log_lock.read();
                let doc_id = derive_doc_id_for_key(key);

                let changes = seq_log.changes_since(snapshot_seq);
                for change in changes {
                    if change.doc_id() == doc_id && change.seq() > snapshot_seq {
                        return Err(ContextraError::Conflict(format!(
                            "Serializable isolation violation for TxId({}): doc_id {} modified at seq {} > snapshot_seq {}",
                            tx_id.inner(),
                            doc_id,
                            change.seq(),
                            snapshot_seq
                        )));
                    }
                }
            }
        }

        let mut prefixes: Vec<(&Vec<u8>, &u64)> = read_set.prefixes_iter().collect();
        prefixes.sort_unstable_by(|(p1, _), (p2, _)| p1.cmp(p2));

        for (prefix, &snapshot_seq) in prefixes {
            if snapshot_seq < pruned_through {
                return Err(ContextraError::Conflict(format!(
                    "Serializable isolation violation for TxId({}): snapshot too old (snapshot_seq {} < pruned_through {}) for range prefix '{:?}'",
                    tx_id.inner(),
                    snapshot_seq,
                    pruned_through,
                    String::from_utf8_lossy(prefix)
                )));
            }

            for (committed_key, &commit_seq) in &committed_writes.keys {
                if committed_key.starts_with(prefix) && commit_seq > snapshot_seq {
                    return Err(ContextraError::Conflict(format!(
                        "Serializable isolation phantom violation for TxId({}): key '{:?}' matching range prefix '{:?}' committed at commit_seq {} > snapshot_seq {}",
                        tx_id.inner(),
                        String::from_utf8_lossy(committed_key),
                        String::from_utf8_lossy(prefix),
                        commit_seq,
                        snapshot_seq
                    )));
                }
            }

            for (&commit_seq, bucket) in &committed_writes.seq_index {
                if commit_seq > snapshot_seq {
                    if let SeqBucket::Coarsened {
                        prefixes: c_prefixes,
                        ..
                    } = bucket
                    {
                        for cp in c_prefixes {
                            if cp.starts_with(prefix) || prefix.starts_with(cp) {
                                return Err(ContextraError::Conflict(format!(
                                    "Serializable isolation phantom violation for TxId({}): coarsened prefix '{:?}' overlapping range prefix '{:?}' committed at commit_seq {} > snapshot_seq {}",
                                    tx_id.inner(),
                                    String::from_utf8_lossy(cp),
                                    String::from_utf8_lossy(prefix),
                                    commit_seq,
                                    snapshot_seq
                                )));
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Atomically validates `read_set` and records `write_keys` at `commit_seq` under a single write lock.
    pub fn validate_and_record<'a>(
        &self,
        tx_id: TxId,
        read_set: &ReadSet,
        write_keys: impl IntoIterator<Item = &'a [u8]>,
        commit_seq: u64,
    ) -> Result<()> {
        let commit_seq = commit_seq & !crate::types::TOMBSTONE_BIT;
        let mut writes = self.committed_writes.write();
        self.validate_internal(&writes, tx_id, read_set)?;
        for key in write_keys {
            writes.insert(key.to_vec(), commit_seq);
        }
        self.check_and_coarsen_if_needed(&mut writes);

        Ok(())
    }
}

impl SsiValidator for SequenceLogSsiValidator {
    fn validate(&self, tx_id: TxId, read_set: &ReadSet) -> Result<()> {
        self.validate_and_record(tx_id, read_set, std::iter::empty::<&[u8]>(), 0)
    }
}

fn derive_doc_id_for_key(key: &[u8]) -> DocId {
    if let Ok(s) = std::str::from_utf8(key) {
        if let Ok(doc_id) = DocId::from_key(s) {
            return doc_id;
        }
    }
    let mut hasher = ahash::AHasher::default();
    hasher.write(key);
    DocId::from(hasher.finish())
}

/// Status of a transaction for SSI validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TxStateStatus {
    /// Active transaction ready for validation and commit.
    Active,
    /// Transaction has been aborted or rolled back.
    Aborted,
    /// Unknown or uninitialized transaction state.
    Unknown,
}

/// Transaction state container passed to [`SsiCommitGate`] for SSI validation.
#[derive(Debug, Clone)]
pub struct TxSsiState {
    tx_id: TxId,
    tenant_id: TenantId,
    commit_seq: u64,
    read_set: ReadSet,
    write_keys: Vec<Vec<u8>>,
    has_read_tracking: bool,
    status: TxStateStatus,
}

impl TxSsiState {
    /// Creates a new `TxSsiState` with required transaction identifiers and target commit sequence number.
    pub fn new(tx_id: TxId, tenant_id: TenantId, commit_seq: u64) -> Self {
        Self {
            tx_id,
            tenant_id,
            commit_seq,
            read_set: ReadSet::new(),
            write_keys: Vec::new(),
            has_read_tracking: true,
            status: TxStateStatus::Active,
        }
    }

    /// Attaches the transaction's [`ReadSet`].
    pub fn with_read_set(mut self, read_set: ReadSet) -> Self {
        self.read_set = read_set;
        self
    }

    /// Sets written keys for commit registration.
    pub fn with_write_keys(mut self, write_keys: Vec<Vec<u8>>) -> Self {
        self.write_keys = write_keys;
        self
    }

    /// Sets read tracking status flag (fail-closed if `false`).
    pub fn with_read_tracking(mut self, enabled: bool) -> Self {
        self.has_read_tracking = enabled;
        self
    }

    /// Sets transaction lifecycle status.
    pub fn with_status(mut self, status: TxStateStatus) -> Self {
        self.status = status;
        self
    }

    /// Returns the transaction ID.
    pub fn tx_id(&self) -> TxId {
        self.tx_id
    }

    /// Returns the tenant ID.
    pub fn tenant_id(&self) -> TenantId {
        self.tenant_id
    }

    /// Returns the target commit sequence number.
    pub fn commit_seq(&self) -> u64 {
        self.commit_seq
    }

    /// Returns a reference to the read set.
    pub fn read_set(&self) -> &ReadSet {
        &self.read_set
    }

    /// Returns a slice of write keys.
    pub fn write_keys(&self) -> &[Vec<u8>] {
        &self.write_keys
    }

    /// Returns whether read tracking was active.
    pub fn has_read_tracking(&self) -> bool {
        self.has_read_tracking
    }

    /// Returns transaction lifecycle status.
    pub fn status(&self) -> TxStateStatus {
        self.status
    }
}

/// Represents the decision produced by [`SsiCommitGate`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommitDecision {
    /// Transaction validation passed and commit registration succeeded.
    Accepted {
        /// Sequence number at which transaction writes were registered.
        commit_seq: u64,
    },
    /// Transaction validation failed and must be aborted due to a conflict or rule violation.
    Aborted {
        /// Human-readable explanation for the commit rejection.
        reason: String,
    },
}

/// Error conditions arising during SSI commit gate validation.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SsiError {
    /// Conflict detected (e.g., write-skew or concurrent modification).
    #[error("SSI conflict detected for transaction {tx_id}: {details}")]
    Conflict {
        /// Transaction ID that experienced conflict.
        tx_id: TxId,
        /// Details of conflict.
        details: String,
    },

    /// Transaction read tracking was disabled or missing (fail-closed).
    #[error("Read tracking missing or uninitialized for transaction {tx_id}")]
    MissingReadTracking {
        /// Transaction ID missing read set.
        tx_id: TxId,
    },

    /// Unknown, invalid, or aborted transaction state.
    #[error("Transaction {tx_id} is in invalid or unknown state: {status:?}")]
    UnknownTxState {
        /// Transaction ID in invalid state.
        tx_id: TxId,
        /// Transaction status.
        status: TxStateStatus,
    },

    /// Tenant mismatch during validation.
    #[error("Tenant mismatch during SSI commit: expected {expected:?}, found {found:?}")]
    TenantMismatch {
        /// Expected tenant ID.
        expected: TenantId,
        /// Found tenant ID.
        found: TenantId,
    },

    /// Sequence log mismatch or divergence.
    #[error("Sequence log divergence: {message}")]
    SequenceLogDivergence {
        /// Explanation of divergence.
        message: String,
    },

    /// Generic internal validation error.
    #[error("Internal SSI validation error: {message}")]
    Internal {
        /// Error message.
        message: String,
    },
}

impl From<SsiError> for ContextraError {
    fn from(err: SsiError) -> Self {
        match err {
            SsiError::Conflict { details, .. } => ContextraError::Conflict(details),
            SsiError::MissingReadTracking { tx_id } => {
                ContextraError::InvalidInput(format!("Missing read tracking for tx {tx_id}"))
            }
            SsiError::UnknownTxState { tx_id, status } => {
                ContextraError::InvalidInput(format!("Invalid state {:?} for tx {tx_id}", status))
            }
            SsiError::TenantMismatch { expected, found } => ContextraError::PolicyViolation(
                format!("Tenant mismatch: expected {expected:?}, found {found:?}"),
            ),
            SsiError::SequenceLogDivergence { message } | SsiError::Internal { message } => {
                ContextraError::Internal(message)
            }
        }
    }
}

/// Commit gate performing atomic SSI validation and committed write registration.
#[derive(Debug)]
pub struct SsiCommitGate {
    validator: Arc<SequenceLogSsiValidator>,
}

impl SsiCommitGate {
    /// Constructs an `SsiCommitGate` backed by the given validator.
    pub fn new(validator: Arc<SequenceLogSsiValidator>) -> Self {
        Self { validator }
    }

    /// Returns a reference to the underlying [`SequenceLogSsiValidator`].
    pub fn validator(&self) -> &Arc<SequenceLogSsiValidator> {
        &self.validator
    }

    /// Atomically validates `tx` state and read set, and registers `tx` write keys if valid.
    pub fn validate_and_register(
        &self,
        tx: &TxSsiState,
        _seq_log: &SequenceLog,
    ) -> std::result::Result<CommitDecision, SsiError> {
        if tx.status != TxStateStatus::Active {
            return Err(SsiError::UnknownTxState {
                tx_id: tx.tx_id,
                status: tx.status,
            });
        }

        if !tx.has_read_tracking {
            return Err(SsiError::MissingReadTracking { tx_id: tx.tx_id });
        }

        let write_key_refs: Vec<&[u8]> = tx.write_keys.iter().map(|k| k.as_slice()).collect();

        match self.validator.validate_and_record(
            tx.tx_id,
            &tx.read_set,
            write_key_refs,
            tx.commit_seq,
        ) {
            Ok(()) => Ok(CommitDecision::Accepted {
                commit_seq: tx.commit_seq,
            }),
            Err(ContextraError::Conflict(details)) => Ok(CommitDecision::Aborted {
                reason: format!("SSI conflict for tx {}: {}", tx.tx_id, details),
            }),
            Err(err) => Err(SsiError::Internal {
                message: err.to_string(),
            }),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn test_read_set_basic() {
        let mut rs = ReadSet::new();
        assert!(rs.is_empty());
        assert_eq!(rs.len(), 0);
        assert_eq!(rs.min_snapshot_seq(), None);

        rs.record_read(b"key1".to_vec(), 10);
        rs.record_read(b"key2".to_vec(), 15);

        assert!(!rs.is_empty());
        assert_eq!(rs.len(), 2);
        assert_eq!(rs.get(b"key1"), Some(10));
        assert_eq!(rs.get(b"key2"), Some(15));
        assert_eq!(rs.get(b"key3"), None);
        assert_eq!(rs.min_snapshot_seq(), Some(10));

        rs.record_read(b"key1".to_vec(), 5);
        assert_eq!(rs.get(b"key1"), Some(5));
        assert_eq!(rs.min_snapshot_seq(), Some(5));

        rs.record_read(b"key1".to_vec(), 20);
        assert_eq!(rs.get(b"key1"), Some(5));
        assert_eq!(rs.min_snapshot_seq(), Some(5));

        rs.clear();
        assert!(rs.is_empty());
        assert_eq!(rs.min_snapshot_seq(), None);
    }

    #[test]
    fn test_validator_no_conflict() {
        let validator = SequenceLogSsiValidator::new();
        let tx = TxId::new(1);

        let mut rs = ReadSet::new();
        rs.record_read(b"key1".to_vec(), 10);

        validator.record_commit_key(b"key1", 10);

        assert!(validator.validate(tx, &rs).is_ok());
    }

    #[test]
    fn test_validator_conflict_detected() {
        let validator = SequenceLogSsiValidator::new();
        let tx = TxId::new(2);

        let mut rs = ReadSet::new();
        rs.record_read(b"key1".to_vec(), 10);

        validator.record_commit_key(b"key1", 11);

        let res = validator.validate(tx, &rs);
        assert!(res.is_err());
        assert!(matches!(res, Err(ContextraError::Conflict(_))));
    }

    #[test]
    fn test_validator_with_sequence_log() {
        let seq_log = Arc::new(RwLock::new(SequenceLog::new()));
        let validator = SequenceLogSsiValidator::with_sequence_log(seq_log.clone());
        let tx = TxId::new(3);

        let doc_key = "doc_alpha";
        let doc_id = DocId::from_key(doc_key).expect("valid doc_id");

        let mut rs = ReadSet::new();
        rs.record_read(doc_key.as_bytes(), 5);

        assert!(validator.validate(tx, &rs).is_ok());

        seq_log.write().record_insert(doc_id, 8);

        let res = validator.validate(tx, &rs);
        assert!(res.is_err());
        assert!(matches!(res, Err(ContextraError::Conflict(_))));
    }
}
