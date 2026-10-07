//! SnapshotRegistry for MVCC-safe reads.
//!
//! Manages active read snapshots and computes the minimum active
//! sequence number to prevent premature tombstone GC.

// FILE-CONTEXT
// STAND: 2026-08-30T18:51:56Z (SESSION: e459bd5f)
// ZWECK: SnapshotRegistry für MVCC-sichere Reads und minimal aktive Sequenznummern.
// INVARIANTEN: Solange SnapshotGuard lebt -> keine Tombstone-GC für seq >= guard.seq_no.
// HOTSPOTS: 40-110
// NICHT-OFFENSICHTLICH: Lock-freie Reads von min_active_seqno() via AtomicU64 (Acquire/Release).
// SIEHE AUCH: rules/tag_taxonomy.md, DECISIONS.md (ADR-024)

// INVARIANT: Snapshot-Registry schützt Reads vor Compaction-GC.
// INVARIANTE: Solange SnapshotGuard lebt → keine Tombstone-GC für seq >= guard.seq_no.
// RAII-PATTERN: Drop deregistriert automatisch. unwrap_or(u64::MAX) ist KORREKT.

use crate::types::TOMBSTONE_BIT;
use parking_lot::Mutex;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Trait for querying the minimum sequence number floor for active MVCC snapshots and transaction readers.
pub trait SnapshotFloor: Send + Sync {
    /// Returns the active GC sequence floor. Sequence numbers strictly below this floor may be safely garbage collected.
    fn floor(&self) -> u64;
}

impl<T: SnapshotFloor + ?Sized> SnapshotFloor for Arc<T> {
    fn floor(&self) -> u64 {
        (**self).floor()
    }
}

impl SnapshotFloor for SnapshotRegistry {
    fn floor(&self) -> u64 {
        self.min_active_seqno()
    }
}

/// An RAII lease for a read snapshot sequence number.
pub struct SnapshotLease {
    guard: SnapshotGuard,
}

impl SnapshotLease {
    /// Returns the sequence number pinned by this snapshot lease.
    pub fn seq_no(&self) -> u64 {
        self.guard.seq_no()
    }
}

/// Registry for active read snapshots.
///
/// ### Synchronization & Memory Ordering Strategy
/// - **Single-Writer Exclusivity via Mutex**: All modifications to active snapshot reference
///   counts (`register`, `pin`, `release`) acquire the `self.active` `parking_lot::Mutex`.
///   This guarantees serialized state transitions and single-writer updates to `min_active_seqno`.
/// - **Lock-Free Reads with Acquire/Release**: Reads of `min_active_seqno()` perform a lockless
///   atomic load using `Ordering::Acquire`. Updates in `update_min()` store the calculated minimum
///   using `Ordering::Release`.
/// - **Sufficiency**: `Ordering::Release` on writes paired with `Ordering::Acquire` on reads is
///   the minimum safe memory ordering for single-writer, many-reader scenarios without holding
///   a lock. It establishes a release-acquire ordering guarantee between registration/deregistration
///   state changes and reader threads (such as LSM compaction workers), preventing instruction
///   reordering across the atomic synchronization boundary.
#[derive(Debug)]
pub struct SnapshotRegistry {
    active: Mutex<BTreeMap<u64, Vec<Instant>>>,
    min_active_seqno: AtomicU64,
}

impl Default for SnapshotRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl SnapshotRegistry {
    /// Creates a new empty SnapshotRegistry.
    pub fn new() -> Self {
        Self {
            active: Mutex::new(BTreeMap::new()),
            min_active_seqno: AtomicU64::new(u64::MAX),
        }
    }

    /// Acquires a read snapshot lease at `seq_no`. Returns an RAII lease that
    /// automatically releases on drop.
    pub fn acquire(self: &Arc<Self>, seq_no: u64) -> SnapshotLease {
        SnapshotLease {
            guard: self.register(seq_no),
        }
    }

    /// Registers a read snapshot. Returns an RAII guard that
    /// automatically deregisters on drop.
    #[deprecated(
        note = "Use SnapshotRegistry::acquire instead to eliminate the read-then-register race window"
    )]
    #[allow(deprecated)]
    pub fn register(self: &Arc<Self>, seq_no: u64) -> SnapshotGuard {
        self.register_at(seq_no, Instant::now())
    }

    /// Registers a read snapshot at a specific creation timestamp `at`.
    #[deprecated(
        note = "Use SnapshotRegistry::acquire_at instead to eliminate the read-then-register race window"
    )]
    #[allow(deprecated)]
    pub fn register_at(self: &Arc<Self>, seq_no: u64, at: Instant) -> SnapshotGuard {
        let lease = self.acquire_at(|| seq_no, at);
        SnapshotGuard { lease }
    }

    /// Returns the minimum active sequence number (`u64::MAX` if none).
    ///
    /// Uses `Ordering::Acquire` to synchronize with `Ordering::Release` writes in `update_min`.
    /// This allows reader threads (e.g., compaction processes) to safely query the minimum
    /// active snapshot sequence without acquiring the `active` mutex lock.
    /// Returns the minimum active sequence number (`u64::MAX` if none).
    ///
    /// # INVARIANT (contextra-store Integration)
    /// `contextra-store` compaction workers query `min_active_seqno()` using lock-free Acquire ordering
    /// to determine tombstone GC floors. The minimum sequence number will never exceed the creation sequence
    /// of any live `SnapshotGuard` or active pin.
    #[inline]
    pub fn min_active_seqno(&self) -> u64 {
        self.min_active_seqno.load(Ordering::Acquire)
    }

    /// Persistent pin of a sequence number to prevent GC (SAOS Checkpoint).
    pub fn pin(&self, seq_no: u64) {
        self.pin_at(seq_no, Instant::now());
    }

    /// Persistent pin of a sequence number at a specific timestamp `at`.
    pub fn pin_at(&self, seq_no: u64, at: Instant) {
        let seq_no = seq_no & !TOMBSTONE_BIT;
        let mut active = self.active.lock();
        active.entry(seq_no).or_default().push(at);
        self.update_min(&active);
    }

    /// Removes a persistent pin.
    pub fn unpin(&self, seq_no: u64) {
        self.release(seq_no);
    }

    pub(crate) fn release(&self, seq_no: u64) {
        self.release_at(seq_no, None);
    }

    pub(crate) fn release_at(&self, seq_no: u64, created_at: Option<Instant>) {
        let seq_no = seq_no & !TOMBSTONE_BIT;
        let mut active = self.active.lock();
        if let Some(timestamps) = active.get_mut(&seq_no) {
            if let Some(ts) = created_at {
                if let Some(pos) = timestamps.iter().position(|&t| t == ts) {
                    timestamps.swap_remove(pos);
                } else {
                    timestamps.pop();
                }
            } else {
                timestamps.pop();
            }
            if timestamps.is_empty() {
                active.remove(&seq_no);
            }
        } else {
            // Unpinning or releasing an un-tracked sequence number is a no-op (§2 Zero-Panic).
        }
        self.update_min(&active);
    }

    /// Identifies which active snapshot sequence number has been held the longest relative to `now`,
    /// returning `Some((seq_no, duration))` or `None` if no snapshots are currently active.
    pub fn longest_active_pin_at(&self, now: Instant) -> Option<(u64, Duration)> {
        let active = self.active.lock();
        let mut longest: Option<(u64, Duration)> = None;

        for (&seq_no, timestamps) in active.iter() {
            for &ts in timestamps {
                let duration = now.saturating_duration_since(ts);
                match longest {
                    None => longest = Some((seq_no, duration)),
                    Some((_, max_duration)) if duration > max_duration => {
                        longest = Some((seq_no, duration));
                    }
                    _ => {}
                }
            }
        }

        longest
    }

    /// Identifies which active snapshot sequence number has been held the longest,
    /// returning `Some((seq_no, duration))` or `None` if no snapshots are currently active.
    pub fn longest_active_pin(&self) -> Option<(u64, Duration)> {
        self.longest_active_pin_at(Instant::now())
    }

    fn update_min(&self, active: &BTreeMap<u64, Vec<Instant>>) {
        // SAFETY: u64::MAX is the correct default when no snapshots are active.
        // It allows the LSM compaction to garbage collect ALL tombstones, as
        // all existing records will have seq_no < u64::MAX.
        //
        // Memory Ordering: Ordering::Release is required to publish all snapshot state updates
        // made under the `active` mutex lock to concurrent readers calling `min_active_seqno()`
        // with Ordering::Acquire.
        let min = active.keys().next().copied().unwrap_or(u64::MAX);
        self.min_active_seqno.store(min, Ordering::Release);
    }
}

/// RAII Guard for an active snapshot.
#[deprecated(note = "Use SnapshotLease returned by SnapshotRegistry::acquire instead")]
pub struct SnapshotGuard {
    pub(crate) lease: SnapshotLease,
}

#[allow(deprecated)]
impl SnapshotGuard {
    /// Returns the sequence number pinned by this snapshot guard.
    pub fn seq_no(&self) -> u64 {
        self.lease.seq_no()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use proptest::prop_assert_eq;

    // INTENT: Snapshot-Registry Lifecycle verified by 5 unit tests.
    #[test]
    fn test_snapshot_registry_basic() {
        let registry = Arc::new(SnapshotRegistry::new());
        assert_eq!(registry.min_active_seqno(), u64::MAX);

        let lease = registry.acquire(|| 100);
        assert_eq!(lease.seq_no(), 100);
        assert_eq!(registry.min_active_seqno(), 100);

        drop(lease);
        assert_eq!(registry.min_active_seqno(), u64::MAX);
    }

    #[test]
    fn test_multiple_snapshots_min_calc() {
        let registry = Arc::new(SnapshotRegistry::new());
        let _g1 = registry.acquire(|| 200);
        let g2 = registry.acquire(|| 100);
        let _g3 = registry.acquire(|| 300);

        assert_eq!(registry.min_active_seqno(), 100);

        drop(g2);
        assert_eq!(registry.min_active_seqno(), 200);
    }

    #[test]
    fn test_pin_unpin() {
        let registry = Arc::new(SnapshotRegistry::new());
        registry.pin(50);
        assert_eq!(registry.min_active_seqno(), 50);

        let g = registry.acquire(|| 100);
        assert_eq!(registry.min_active_seqno(), 50);

        registry.unpin(50);
        assert_eq!(registry.min_active_seqno(), 100);

        drop(g);
        assert_eq!(registry.min_active_seqno(), u64::MAX);
    }

    #[test]
    fn test_seq_no_tombstone_masking() {
        let registry = Arc::new(SnapshotRegistry::new());
        // seq_no with tombstone bit set
        let seq = 100 | crate::types::TOMBSTONE_BIT;
        let lease = registry.acquire(|| seq);

        assert_eq!(lease.seq_no(), 100);
        assert_eq!(registry.min_active_seqno(), 100);
    }

    #[test]
    fn test_ref_counting() {
        let registry = Arc::new(SnapshotRegistry::new());
        let g1 = registry.acquire(|| 100);
        let g2 = registry.acquire(|| 100);

        assert_eq!(registry.min_active_seqno(), 100);

        drop(g1);
        assert_eq!(registry.min_active_seqno(), 100);

        drop(g2);
        assert_eq!(registry.min_active_seqno(), u64::MAX);
    }

    #[test]
    fn test_unpin_unknown_seq_no_does_not_panic() {
        let registry = Arc::new(SnapshotRegistry::new());
        // Unpinning a sequence number that was never pinned or registered must not panic
        registry.unpin(999);
        assert_eq!(registry.min_active_seqno(), u64::MAX);
    }

    #[test]
    fn test_snapshot_panic_free_safety() {
        let registry = Arc::new(SnapshotRegistry::new());
        assert_eq!(registry.min_active_seqno(), u64::MAX);
        registry.release(12345);
        registry.unpin(12345);
        assert_eq!(registry.min_active_seqno(), u64::MAX);
    }

    #[test]
    fn test_double_registration_and_sequential_drop() {
        let registry = Arc::new(SnapshotRegistry::new());
        let g1 = registry.acquire(|| 42);
        let g2 = registry.acquire(|| 42);

        assert_eq!(registry.min_active_seqno(), 42);

        // Dropping one guard decrements ref-count to 1; seq_no 42 is still active.
        drop(g1);
        assert_eq!(
            registry.min_active_seqno(),
            42,
            "min_active_seqno must remain 42 while second guard is active"
        );

        // Dropping second guard decrements ref-count to 0 and removes entry.
        drop(g2);
        assert_eq!(
            registry.min_active_seqno(),
            u64::MAX,
            "min_active_seqno must return u64::MAX after all guards are dropped"
        );
    }

    proptest::proptest! {
        #[test]
        fn prop_snapshot_registry_min_active(
            seqs in proptest::collection::vec(0..1000u64, 1..50)
        ) {
            let registry = Arc::new(SnapshotRegistry::new());
            let mut leases = Vec::new();

            for &seq in &seqs {
                leases.push(registry.acquire(|| seq));
            }

            let min_expected = *seqs.iter().min().unwrap(); // #[cfg(test)]
            prop_assert_eq!(registry.min_active_seqno(), min_expected);

            leases.pop(); // Drop last element

            // If all elements dropped, min_active is MAX, else it's min of remaining
            if leases.is_empty() {
                prop_assert_eq!(registry.min_active_seqno(), u64::MAX);
            } else {
                let remaining_min = leases.iter().map(|l| l.seq_no()).min().unwrap_or(u64::MAX);
                prop_assert_eq!(registry.min_active_seqno(), remaining_min);
            }
        }

        /// Proptest: Proves that dropping leases one-by-one in arbitrary order
        /// always maintains the correct min_active_seqno invariant.
        ///
        /// # Anti-Mirroring
        /// Expected min is computed by maintaining an independent sorted list
        /// of remaining sequence numbers, not by calling SnapshotRegistry methods.
        #[test]
        fn prop_snapshot_register_unregister_stress(
            seqs in proptest::collection::vec(0..5000u64, 2..80),
            // Indices into the leases vec to determine drop order
            drop_order_seed in proptest::collection::vec(0..1000usize, 2..80),
        ) {
            let registry = Arc::new(SnapshotRegistry::new());
            let mut leases: Vec<Option<SnapshotLease>> = Vec::new();

            // Acquire all
            for &seq in &seqs {
                leases.push(Some(registry.acquire(|| seq)));
            }

            // Build independent reference: sorted multiset of active seqs
            let mut active_seqs: Vec<u64> = seqs.clone();
            active_seqs.sort_unstable();

            // Verify initial state
            prop_assert_eq!(registry.min_active_seqno(), active_seqs[0]);

            // Drop leases one-by-one using the seed to pick which to drop
            let mut remaining_indices: Vec<usize> = (0..leases.len()).collect();
            for seed_val in &drop_order_seed {
                if remaining_indices.is_empty() {
                    break;
                }
                let idx_in_remaining = seed_val % remaining_indices.len();
                let lease_idx = remaining_indices.remove(idx_in_remaining);

                // Drop the lease
                let seq_val = leases[lease_idx].as_ref().unwrap().seq_no(); // #[cfg(test)]
                leases[lease_idx] = None;

                // Remove from reference (one occurrence only)
                if let Some(pos) = active_seqs.iter().position(|&s| s == seq_val) {
                    active_seqs.remove(pos);
                }

                // Verify invariant
                let expected_min = active_seqs.first().copied().unwrap_or(u64::MAX);
                prop_assert_eq!(
                    registry.min_active_seqno(),
                    expected_min,
                    "After dropping lease for seq={}, min should be {}",
                    seq_val,
                    expected_min
                );
            }
        }

        /// Proptest: Pin/Unpin combined with acquire/drop.
        /// Proves that persistent pins and RAII leases coexist correctly.
        ///
        /// # Anti-Mirroring
        /// Reference min is maintained in an independent BTreeMap<u64, usize>.
        #[test]
        fn prop_snapshot_pin_unpin_interleaving(
            pin_seqs in proptest::collection::vec(0..500u64, 1..20),
            lease_seqs in proptest::collection::vec(0..500u64, 1..20),
        ) {
            use std::collections::BTreeMap;
            let registry = Arc::new(SnapshotRegistry::new());
            let mut ref_counts: BTreeMap<u64, usize> = BTreeMap::new();

            // Pin all
            for &seq in &pin_seqs {
                registry.pin(seq);
                *ref_counts.entry(seq).or_default() += 1;
            }

            // Acquire leases
            let mut leases = Vec::new();
            for &seq in &lease_seqs {
                leases.push(registry.acquire(|| seq));
                *ref_counts.entry(seq).or_default() += 1;
            }

            // Verify combined min
            let expected_min = ref_counts.keys().next().copied().unwrap_or(u64::MAX);
            prop_assert_eq!(registry.min_active_seqno(), expected_min);

            // Unpin all pins
            for &seq in &pin_seqs {
                registry.unpin(seq);
                if let Some(count) = ref_counts.get_mut(&seq) {
                    *count -= 1;
                    if *count == 0 {
                        ref_counts.remove(&seq);
                    }
                }
            }

            let expected_min2 = ref_counts.keys().next().copied().unwrap_or(u64::MAX);
            prop_assert_eq!(registry.min_active_seqno(), expected_min2);

            // Drop all leases
            drop(leases);
            prop_assert_eq!(registry.min_active_seqno(), u64::MAX);
        }
    }
}
