//! SnapshotRegistry for MVCC-safe reads.
//!
//! Manages active read snapshots and computes the minimum active
//! sequence number to prevent premature tombstone GC.

// FILE-CONTEXT
// STAND: 2026-10-15T00:00:00Z
// ZWECK: SnapshotRegistry für MVCC-sichere Reads und minimal aktive Sequenznummern.
// INVARIANTEN: Solange SnapshotLease lebt -> keine Tombstone-GC für seq >= lease.seq_no.
// HOTSPOTS: 40-110
// NICHT-OFFENSICHTLICH: Lock-freie Reads von min_active_seqno() via AtomicU64 (Acquire/Release).

use contextra_ports::Clock;
use contextra_types::ContextraError;

use crate::lease::SnapshotLease;
use crate::sync::atomic::{AtomicU64, Ordering};
use crate::sync::{Arc, Mutex};
use crate::types::TOMBSTONE_BIT;
use std::collections::BTreeMap;
use std::time::Duration;

pub use crate::floor::SnapshotFloor;

/// Default maximum snapshot age policy: 15 minutes in ticks (nanoseconds).
pub const DEFAULT_MAX_SNAPSHOT_AGE_TICKS: u64 = 15 * 60 * 1_000_000_000;

/// Default pin grace policy: 30 seconds in ticks (nanoseconds).
pub const DEFAULT_PIN_GRACE_TICKS: u64 = 30 * 1_000_000_000;

impl SnapshotFloor for SnapshotRegistry {
    fn floor(&self) -> u64 {
        self.min_active_seqno()
    }
}

/// Registry for active read snapshots.
///
/// ### Synchronization & Memory Ordering Strategy
/// - **Single-Writer Exclusivity via Mutex**: All modifications to active snapshot reference
///   counts (`acquire`, `pin`, `release`) acquire the `self.active` `parking_lot::Mutex`.
///   This guarantees serialized state transitions and single-writer updates to `min_active_seqno`.
/// - **Lock-Free Reads with Acquire/Release**: Reads of `min_active_seqno()` perform a lockless
///   atomic load using `Ordering::Acquire`. Updates in `update_min()` store the calculated minimum
///   using `Ordering::Release`.
pub struct SnapshotRegistry {
    active: Mutex<BTreeMap<u64, Vec<u64>>>,
    min_active_seqno: AtomicU64,
    max_snapshot_age_ticks: AtomicU64,
    pin_grace_ticks: AtomicU64,
    clock: Option<Arc<dyn Clock>>,
}

impl std::fmt::Debug for SnapshotRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SnapshotRegistry")
            .field("active", &self.active)
            .field("min_active_seqno", &self.min_active_seqno)
            .field("max_snapshot_age_ticks", &self.max_snapshot_age_ticks)
            .field("pin_grace_ticks", &self.pin_grace_ticks)
            .field("has_clock", &self.clock.is_some())
            .finish()
    }
}

impl Default for SnapshotRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl SnapshotRegistry {
    /// Creates a new empty SnapshotRegistry with default pin-policy settings (15m max_snapshot_age, 30s pin_grace).
    pub fn new() -> Self {
        Self {
            active: Mutex::new(BTreeMap::new()),
            min_active_seqno: AtomicU64::new(u64::MAX),
            max_snapshot_age_ticks: AtomicU64::new(DEFAULT_MAX_SNAPSHOT_AGE_TICKS),
            pin_grace_ticks: AtomicU64::new(DEFAULT_PIN_GRACE_TICKS),
            clock: None,
        }
    }

    /// Sets the injected clock port and returns `self`.
    pub fn with_clock(mut self, clock: Arc<dyn Clock>) -> Self {
        self.clock = Some(clock);
        self
    }

    /// Sets the injected clock port.
    pub fn set_clock(&mut self, clock: Arc<dyn Clock>) {
        self.clock = Some(clock);
    }

    /// Returns the injected clock port, if configured.
    pub fn clock(&self) -> Option<&Arc<dyn Clock>> {
        self.clock.as_ref()
    }

    /// Returns the current monotonic timestamp in ticks (nanoseconds) using the configured clock, or 0.
    pub fn now_ticks(&self) -> u64 {
        if let Some(ref clock) = self.clock {
            clock.monotonic_nanos()
        } else {
            0
        }
    }

    /// Sets the maximum snapshot age in ticks (nanoseconds).
    pub fn set_max_snapshot_age_ticks(&self, ticks: u64) {
        self.max_snapshot_age_ticks.store(ticks, Ordering::Release);
    }

    /// Returns the configured maximum snapshot age in ticks (nanoseconds).
    pub fn max_snapshot_age_ticks(&self) -> u64 {
        self.max_snapshot_age_ticks.load(Ordering::Acquire)
    }

    /// Sets the maximum snapshot age using a [`Duration`].
    pub fn set_max_snapshot_age(&self, duration: Duration) {
        let ticks = u64::try_from(duration.as_nanos()).unwrap_or(u64::MAX);
        self.set_max_snapshot_age_ticks(ticks);
    }

    /// Returns the configured maximum snapshot age as a [`Duration`].
    pub fn max_snapshot_age(&self) -> Duration {
        Duration::from_nanos(self.max_snapshot_age_ticks())
    }

    /// Sets the pin grace period in ticks (nanoseconds).
    pub fn set_pin_grace_ticks(&self, ticks: u64) {
        self.pin_grace_ticks.store(ticks, Ordering::Release);
    }

    /// Returns the configured pin grace period in ticks (nanoseconds).
    pub fn pin_grace_ticks(&self) -> u64 {
        self.pin_grace_ticks.load(Ordering::Acquire)
    }

    /// Sets the pin grace period using a [`Duration`].
    pub fn set_pin_grace(&self, duration: Duration) {
        let ticks = u64::try_from(duration.as_nanos()).unwrap_or(u64::MAX);
        self.set_pin_grace_ticks(ticks);
    }

    /// Returns the configured pin grace period as a [`Duration`].
    pub fn pin_grace(&self) -> Duration {
        Duration::from_nanos(self.pin_grace_ticks())
    }

    /// Erwirbt einen Read-Snapshot, indem `f` *innerhalb* des Registry-Locks ausgewertet wird.
    #[cfg(loom)]
    pub fn acquire(this: &Arc<Self>, f: impl FnOnce() -> u64) -> SnapshotLease {
        let now = this.now_ticks();
        SnapshotRegistry::acquire_at(this, f, now)
    }

    /// Erwirbt einen Read-Snapshot, indem `f` *innerhalb* des Registry-Locks ausgewertet wird.
    #[cfg(not(loom))]
    pub fn acquire(self: &Arc<Self>, f: impl FnOnce() -> u64) -> SnapshotLease {
        let now = self.now_ticks();
        self.acquire_at(f, now)
    }

    /// Wie [`acquire`], erlaubt aber einen expliziten Erstellungszeitstempel `at_ticks`.
    #[cfg(loom)]
    pub fn acquire_at(this: &Arc<Self>, f: impl FnOnce() -> u64, at_ticks: u64) -> SnapshotLease {
        let seq_no = {
            let mut active = match this.active.lock() {
                Ok(guard) => guard,
                Err(poisoned) => poisoned.into_inner(),
            };
            let seq_no = f() & !TOMBSTONE_BIT;
            active.entry(seq_no).or_default().push(at_ticks);
            this.update_min(&active);
            seq_no
        };
        SnapshotLease::new(Arc::clone(this), seq_no, Some(at_ticks))
    }

    /// Wie [`acquire`], erlaubt aber einen expliziten Erstellungszeitstempel `at_ticks`.
    #[cfg(not(loom))]
    pub fn acquire_at(self: &Arc<Self>, f: impl FnOnce() -> u64, at_ticks: u64) -> SnapshotLease {
        let seq_no = {
            let mut active = match self.active.lock() {
                Ok(guard) => guard,
                Err(poisoned) => poisoned.into_inner(),
            };
            let seq_no = f() & !TOMBSTONE_BIT;
            active.entry(seq_no).or_default().push(at_ticks);
            self.update_min(&active);
            seq_no
        };
        SnapshotLease::new(Arc::clone(self), seq_no, Some(at_ticks))
    }

    /// Returns the minimum active sequence number (`u64::MAX` if none).
    #[inline]
    pub fn min_active_seqno(&self) -> u64 {
        self.min_active_seqno.load(Ordering::Acquire)
    }

    /// Persistent pin of a sequence number to prevent GC (SAOS Checkpoint).
    pub fn pin(&self, seq_no: u64) {
        self.pin_at(seq_no, self.now_ticks());
    }

    /// Persistent pin of a sequence number at a specific timestamp `at_ticks`.
    pub fn pin_at(&self, seq_no: u64, at_ticks: u64) {
        let seq_no = seq_no & !TOMBSTONE_BIT;
        let mut active = match self.active.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        active.entry(seq_no).or_default().push(at_ticks);
        self.update_min(&active);
    }

    /// Removes a persistent pin.
    pub fn unpin(&self, seq_no: u64) {
        self.release(seq_no);
    }

    pub(crate) fn release(&self, seq_no: u64) {
        self.release_at(seq_no, None);
    }

    pub(crate) fn release_at(&self, seq_no: u64, created_at_ticks: Option<u64>) {
        let seq_no = seq_no & !TOMBSTONE_BIT;
        let mut active = match self.active.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        if let Some(timestamps) = active.get_mut(&seq_no) {
            if let Some(ts) = created_at_ticks {
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
        }
        self.update_min(&active);
    }

    /// Checks if a given snapshot lease is still valid at `now_ticks`.
    ///
    /// Returns `Err(ContextraError::SnapshotExpired)` if the age of the lease exceeds `max_snapshot_age_ticks`.
    pub fn check_lease_valid(
        &self,
        lease: &SnapshotLease,
        now_ticks: u64,
    ) -> Result<(), ContextraError> {
        if let Some(created_at) = lease.created_at_ticks() {
            let age = now_ticks.saturating_sub(created_at);
            let max_age = self.max_snapshot_age_ticks();
            if age > max_age {
                return Err(ContextraError::snapshot_expired(
                    lease.seq_no(),
                    age,
                    max_age,
                ));
            }
        }
        Ok(())
    }

    /// Forcibly evicts active pins whose duration exceeds `pin_grace_ticks` relative to `now_ticks`.
    /// Returns the list of sequence numbers whose pins were forcibly expired.
    pub fn evict_grace_expired_pins_at(&self, now_ticks: u64) -> Vec<u64> {
        let grace = self.pin_grace_ticks();
        let mut active = match self.active.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        let mut evicted = Vec::new();
        let mut keys_to_remove = Vec::new();

        for (&seq_no, timestamps) in active.iter_mut() {
            timestamps.retain(|&ts| {
                let age = now_ticks.saturating_sub(ts);
                let is_expired = age > grace;
                if is_expired {
                    evicted.push(seq_no);
                }
                !is_expired
            });
            if timestamps.is_empty() {
                keys_to_remove.push(seq_no);
            }
        }

        for seq in keys_to_remove {
            active.remove(&seq);
        }

        self.update_min(&active);
        evicted.sort_unstable();
        evicted.dedup();
        evicted
    }

    /// Returns the minimum active sequence number after forcibly evicting grace-expired pins at `now_ticks`.
    pub fn min_active_seqno_at(&self, now_ticks: u64) -> u64 {
        let _ = self.evict_grace_expired_pins_at(now_ticks);
        self.min_active_seqno()
    }

    /// Identifies which active snapshot sequence number has been held the longest relative to `now_ticks`,
    /// returning `Some((seq_no, duration))` or `None` if no snapshots are currently active.
    pub fn longest_active_pin_at(&self, now_ticks: u64) -> Option<(u64, Duration)> {
        let active = match self.active.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        let mut longest: Option<(u64, u64)> = None;

        for (&seq_no, timestamps) in active.iter() {
            for &ts in timestamps {
                let age = now_ticks.saturating_sub(ts);
                match longest {
                    None => longest = Some((seq_no, age)),
                    Some((_, max_age)) if age > max_age => {
                        longest = Some((seq_no, age));
                    }
                    _ => {}
                }
            }
        }

        longest.map(|(seq, age)| (seq, Duration::from_nanos(age)))
    }

    /// Identifies which active snapshot sequence number has been held the longest,
    /// returning `Some((seq_no, duration))` or `None` if no snapshots are currently active.
    pub fn longest_active_pin(&self) -> Option<(u64, Duration)> {
        self.longest_active_pin_at(self.now_ticks())
    }

    fn update_min(&self, active: &BTreeMap<u64, Vec<u64>>) {
        let min = active.keys().next().copied().unwrap_or(u64::MAX);
        self.min_active_seqno.store(min, Ordering::Release);
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use proptest::prop_assert_eq;

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
        registry.pin_at(50, 100);
        assert_eq!(registry.min_active_seqno(), 50);

        let g = registry.acquire_at(|| 100, 100);
        assert_eq!(registry.min_active_seqno(), 50);

        registry.unpin(50);
        assert_eq!(registry.min_active_seqno(), 100);

        drop(g);
        assert_eq!(registry.min_active_seqno(), u64::MAX);
    }

    #[test]
    fn test_seq_no_tombstone_masking() {
        let registry = Arc::new(SnapshotRegistry::new());
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

        drop(g1);
        assert_eq!(
            registry.min_active_seqno(),
            42,
            "min_active_seqno must remain 42 while second lease is active"
        );

        drop(g2);
        assert_eq!(
            registry.min_active_seqno(),
            u64::MAX,
            "min_active_seqno must return u64::MAX after all leases are dropped"
        );
    }

    #[test]
    fn test_max_snapshot_age_expired_lease() {
        let registry = Arc::new(SnapshotRegistry::new());
        registry.set_max_snapshot_age_ticks(1_000_000); // 1ms

        let lease = registry.acquire_at(|| 50, 100);
        assert!(lease.check_valid_at(100 + 500).is_ok());

        // Age = 1_000_001 > 1_000_000
        let res = lease.check_valid_at(100 + 1_000_001);
        assert!(matches!(
            res,
            Err(ContextraError::SnapshotExpired {
                seq_no: 50,
                age_ticks: 1_000_001,
                max_age_ticks: 1_000_000
            })
        ));
    }

    #[test]
    fn test_pin_grace_eviction() {
        let registry = Arc::new(SnapshotRegistry::new());
        registry.set_pin_grace_ticks(30_000_000_000); // 30s

        registry.pin_at(10, 1000);
        registry.pin_at(20, 1000 + 40_000_000_000);

        assert_eq!(registry.min_active_seqno(), 10);

        // At t = 1000 + 35_000_000_000, pin at seq 10 is 35s old (> 30s pin_grace)
        let evicted = registry.evict_grace_expired_pins_at(1000 + 35_000_000_000);
        assert_eq!(evicted, vec![10]);

        // Remaining active pin is seq 20 (age 5s)
        assert_eq!(registry.min_active_seqno(), 20);
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

            let min_expected = *seqs.iter().min().unwrap();
            prop_assert_eq!(registry.min_active_seqno(), min_expected);

            leases.pop();

            if leases.is_empty() {
                prop_assert_eq!(registry.min_active_seqno(), u64::MAX);
            } else {
                let remaining_min = leases.iter().map(|l| l.seq_no()).min().unwrap_or(u64::MAX);
                prop_assert_eq!(registry.min_active_seqno(), remaining_min);
            }
        }

        #[test]
        fn prop_snapshot_register_unregister_stress(
            seqs in proptest::collection::vec(0..5000u64, 2..80),
            drop_order_seed in proptest::collection::vec(0..1000usize, 2..80),
        ) {
            let registry = Arc::new(SnapshotRegistry::new());
            let mut leases: Vec<Option<SnapshotLease>> = Vec::new();

            for &seq in &seqs {
                leases.push(Some(registry.acquire(|| seq)));
            }

            let mut active_seqs: Vec<u64> = seqs.clone();
            active_seqs.sort_unstable();

            prop_assert_eq!(registry.min_active_seqno(), active_seqs[0]);

            let mut remaining_indices: Vec<usize> = (0..leases.len()).collect();
            for seed_val in &drop_order_seed {
                if remaining_indices.is_empty() {
                    break;
                }
                let idx_in_remaining = seed_val % remaining_indices.len();
                let lease_idx = remaining_indices.remove(idx_in_remaining);

                let seq_val = leases[lease_idx].as_ref().unwrap().seq_no();
                leases[lease_idx] = None;

                if let Some(pos) = active_seqs.iter().position(|&s| s == seq_val) {
                    active_seqs.remove(pos);
                }

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

        #[test]
        fn prop_snapshot_pin_unpin_interleaving(
            pin_seqs in proptest::collection::vec(0..500u64, 1..20),
            lease_seqs in proptest::collection::vec(0..500u64, 1..20),
        ) {
            use std::collections::BTreeMap;
            let registry = Arc::new(SnapshotRegistry::new());
            let mut ref_counts: BTreeMap<u64, usize> = BTreeMap::new();

            for &seq in &pin_seqs {
                registry.pin_at(seq, 0);
                *ref_counts.entry(seq).or_default() += 1;
            }

            let mut leases = Vec::new();
            for &seq in &lease_seqs {
                leases.push(registry.acquire_at(|| seq, 0));
                *ref_counts.entry(seq).or_default() += 1;
            }

            let expected_min = ref_counts.keys().next().copied().unwrap_or(u64::MAX);
            prop_assert_eq!(registry.min_active_seqno(), expected_min);

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

            drop(leases);
            prop_assert_eq!(registry.min_active_seqno(), u64::MAX);
        }
    }
}
