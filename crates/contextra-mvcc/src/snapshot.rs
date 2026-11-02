// ANCHOR[contextra-mvcc:SNAPSHOT-REGISTRY-ROOT]
//! # Snapshot-Registry (`SnapshotRegistry`) für MVCC-sichere Leseoperationen
//!
//! Die `SnapshotRegistry` verwaltet alle aktuell aktiven Lese-Snapshots nach Mandanten (`TenantTag`)
//! und ermittelt die minimal aktive Sequenznummer (`min_read_snapshot` / `min_for_compaction`).
//!
//! ## Warum ist das essenziell? (GC-Schutz)
//! In einem MVCC-System erzeugen Löschungen sogenannte Tombstones.
//! Der Garbage-Collector (LSM Compaction) darf Tombstones und ältere Datenversionen erst dann
//! physisch entfernen, wenn kein aktiver Leser mehr existiert, dessen Snapshot-Sequenznummer
//! vor oder bei dieser Änderung liegt.
//!
//! ## Synchronisations- und Speicherarchitektur
//! - **Mandantentrennung**: Snapshots werden in einer Map `TenantTag -> BTreeMap<Seq, usize>` geführt.
//!   Das Limit beträgt maximal 1024 Snapshots je Mandant (D-87).
//! - **Kein Sentinel**: Ohne offene Snapshots liefert `min_read_snapshot()` atomar `None`.
//!   Die Kompaktierung nutzt `min_for_compaction(current_seq)`, was bei leerer Registry `current_seq` liefert.

use crate::error::MvccError;
use crate::types::TOMBSTONE_BIT;
use contextra_ports::{Clock, SystemClock};
use parking_lot::Mutex;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Fester 16-Byte Mandanten-Tag gemäß D-61.
pub type TenantTag = [u8; 16];

/// Sequenznummer.
pub type Seq = u64;

/// Anzahl widerrufener Snapshots.
pub type RevokedCount = usize;

/// Platzhalter für Matcher-Bereiche (WP-5.1: Fail-Closed Mandantenwiderruf bis `gated_seq`).
pub type Matcher = ();

/// Standard-Mandantentag für Abwärtskompatibilität.
pub const DEFAULT_TENANT_TAG: TenantTag = [0u8; 16];

/// Obergrenze aktiver Snapshots je Mandant (D-87).
pub const MAX_SNAPSHOTS_PER_TENANT: usize = 1024;

#[derive(Default)]
struct TenantState {
    active: BTreeMap<Seq, usize>,
    timestamps: BTreeMap<Seq, Vec<u64>>,
    total_active: usize,
}

// ANCHOR[contextra-mvcc:SNAPSHOT-REGISTRY-STRUCT]
/// Zentrale Registrierung für aktive Lese-Snapshots im System.
pub struct SnapshotRegistry {
    tenants: Mutex<BTreeMap<TenantTag, TenantState>>,
    revoked_tags: Mutex<BTreeSet<(TenantTag, Seq)>>,
    active_count: AtomicUsize,
    min_active_seqno: AtomicU64,
    clock: Arc<dyn Clock>,
}

impl std::fmt::Debug for SnapshotRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SnapshotRegistry")
            .field("active_count", &self.active_count.load(Ordering::Acquire))
            .field(
                "min_active_seqno",
                &self.min_active_seqno.load(Ordering::Acquire),
            )
            .finish()
    }
}

impl Default for SnapshotRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl SnapshotRegistry {
    // ANCHOR[contextra-mvcc:SNAPSHOT-REGISTRY-NEW]
    /// Erstellt eine neue, leere Snapshot-Registry mit Standard-[`SystemClock`].
    pub fn new() -> Self {
        Self::with_clock(Arc::new(SystemClock::new()))
    }

    /// Erstellt eine neue Snapshot-Registry mit benutzerdefiniertem [`Clock`]-Port.
    pub fn with_clock(clock: Arc<dyn Clock>) -> Self {
        Self {
            tenants: Mutex::new(BTreeMap::new()),
            revoked_tags: Mutex::new(BTreeSet::new()),
            active_count: AtomicUsize::new(0),
            min_active_seqno: AtomicU64::new(0),
            clock,
        }
    }

    /// Liefert eine Referenz auf den aktiven Zeitport ([`Clock`]).
    pub fn clock(&self) -> &Arc<dyn Clock> {
        &self.clock
    }

    /// Öffnet einen neuen Snapshot für den Mandanten `tenant` bei Sequenz `at`.
    ///
    /// Garantiert das Limit von maximal 1024 aktiven Snapshots je Mandant (D-87).
    pub fn open(self: &Arc<Self>, tenant: TenantTag, at: Seq) -> Result<SnapshotHandle, MvccError> {
        let clean_seq = at & !TOMBSTONE_BIT;
        let now_nanos = self.clock.monotonic_nanos();
        let mut tenants = self.tenants.lock();
        let state = tenants.entry(tenant).or_default();

        if state.total_active >= MAX_SNAPSHOTS_PER_TENANT {
            return Err(MvccError::LimitSnapshots);
        }

        let ref_count = state.active.entry(clean_seq).or_insert(0);
        *ref_count = ref_count.saturating_add(1);
        state
            .timestamps
            .entry(clean_seq)
            .or_default()
            .push(now_nanos);
        state.total_active = state.total_active.saturating_add(1);

        self.active_count.fetch_add(1, Ordering::SeqCst);
        self.update_min_locked(&tenants);

        let revoked_flag = Arc::new(AtomicBool::new(false));

        Ok(SnapshotHandle {
            registry: self.clone(),
            tenant,
            seq: clean_seq,
            created_at: Some(now_nanos),
            revoked: revoked_flag,
        })
    }

    /// Widerruft alle Snapshots des Mandanten mit `seq <= gated_seq` (fail-closed, D-61).
    pub fn revoke_covering(
        &self,
        tenant: TenantTag,
        _range: &Matcher,
        gated_seq: Seq,
    ) -> RevokedCount {
        let clean_gated = gated_seq & !TOMBSTONE_BIT;
        let mut tenants = self.tenants.lock();
        let mut count: RevokedCount = 0;

        if let Some(state) = tenants.get_mut(&tenant) {
            let mut revoked = self.revoked_tags.lock();
            for &seq in state.active.keys() {
                if seq <= clean_gated && revoked.insert((tenant, seq)) {
                    count = count.saturating_add(1);
                }
            }
        }

        count
    }

    /// Minimal aktive Snapshot-Sequenznummer über alle Mandanten.
    ///
    /// Liefert `None`, wenn keine Snapshots aktiv sind (MV-01).
    pub fn min_read_snapshot(&self) -> Option<Seq> {
        if self.active_count.load(Ordering::Acquire) == 0 {
            None
        } else {
            Some(self.min_active_seqno.load(Ordering::Acquire))
        }
    }

    /// Minimal erforderliche Sequenz für die Kompaktierung (MV-02).
    ///
    /// Liefert `min(min_read_snapshot, current)`, bei leerer Registry stets `current`.
    pub fn min_for_compaction(&self, current: Seq) -> Seq {
        match self.min_read_snapshot() {
            Some(min_seq) => min_seq.min(current),
            None => current,
        }
    }

    /// Abwärtskompatible Hilfsmethode für bestehende Aufrufer.
    ///
    /// Gibt die kleinste Sequenznummer zurück, oder 0, falls keine Snapshots aktiv sind.
    /// Keinesfalls wird ein Sentinel-Wert wie `u64::MAX` zurückgegeben (MV-01, I-12).
    /// Konsumenten müssen auf `min_read_snapshot()` (`Option<Seq>`) oder `min_for_compaction()` umsteigen.
    pub fn min_active_seqno(&self) -> u64 {
        if self.active_count.load(Ordering::Acquire) == 0 {
            0
        } else {
            self.min_active_seqno.load(Ordering::Acquire)
        }
    }

    /// Prüft, ob ein Snapshot für den Standard-Mandanten widerrufen wurde.
    pub fn is_revoked(&self, seq_no: u64) -> bool {
        self.is_revoked_tenant(DEFAULT_TENANT_TAG, seq_no)
    }

    /// Prüft, ob ein Snapshot eines bestimmten Mandanten widerrufen wurde.
    pub fn is_revoked_tenant(&self, tenant: TenantTag, seq_no: u64) -> bool {
        let clean_seq = seq_no & !TOMBSTONE_BIT;
        self.revoked_tags.lock().contains(&(tenant, clean_seq))
    }

    /// Abwärtskompatible Registrierung für Default-Mandant.
    pub fn register(self: &Arc<Self>, seq_no: u64) -> SnapshotGuard {
        let now_nanos = self.clock.monotonic_nanos();
        self.register_at(seq_no, now_nanos)
    }

    /// Abwärtskompatible Registrierung mit explizitem Zeitstempel.
    pub fn register_at(self: &Arc<Self>, seq_no: u64, at_nanos: u64) -> SnapshotGuard {
        let seq_no = seq_no & !TOMBSTONE_BIT;
        let mut tenants = self.tenants.lock();
        let state = tenants.entry(DEFAULT_TENANT_TAG).or_default();
        let ref_count = state.active.entry(seq_no).or_insert(0);
        *ref_count = ref_count.saturating_add(1);
        state.timestamps.entry(seq_no).or_default().push(at_nanos);
        state.total_active = state.total_active.saturating_add(1);

        self.active_count.fetch_add(1, Ordering::SeqCst);
        self.update_min_locked(&tenants);

        SnapshotGuard {
            registry: self.clone(),
            seq_no,
            created_at: Some(at_nanos),
        }
    }

    /// Pinnt eine Sequenznummer dauerhaft.
    pub fn pin(&self, seq_no: u64) {
        let now_nanos = self.clock.monotonic_nanos();
        self.pin_at(seq_no, now_nanos);
    }

    /// Pinnt eine Sequenznummer zu einem spezifischen Zeitstempel.
    pub fn pin_at(&self, seq_no: u64, at_nanos: u64) {
        let seq_no = seq_no & !TOMBSTONE_BIT;
        let mut tenants = self.tenants.lock();
        let state = tenants.entry(DEFAULT_TENANT_TAG).or_default();
        let ref_count = state.active.entry(seq_no).or_insert(0);
        *ref_count = ref_count.saturating_add(1);
        state.timestamps.entry(seq_no).or_default().push(at_nanos);
        state.total_active = state.total_active.saturating_add(1);

        self.active_count.fetch_add(1, Ordering::SeqCst);
        self.update_min_locked(&tenants);
    }

    /// Hebt einen dauerhaften Pin auf.
    pub fn unpin(&self, seq_no: u64) {
        self.release(seq_no);
    }

    pub(crate) fn release(&self, seq_no: u64) {
        self.release_at(seq_no, None);
    }

    pub(crate) fn release_at(&self, seq_no: u64, created_at: Option<u64>) {
        self.release_tenant_at(DEFAULT_TENANT_TAG, seq_no, created_at);
    }

    pub(crate) fn release_tenant_at(
        &self,
        tenant: TenantTag,
        seq_no: u64,
        created_at: Option<u64>,
    ) {
        let seq_no = seq_no & !TOMBSTONE_BIT;
        let mut tenants = self.tenants.lock();
        if let Some(state) = tenants.get_mut(&tenant) {
            let mut removed_one = false;
            if let Some(count) = state.active.get_mut(&seq_no) {
                if *count > 1 {
                    *count = count.saturating_sub(1);
                    removed_one = true;
                } else {
                    state.active.remove(&seq_no);
                    self.revoked_tags.lock().remove(&(tenant, seq_no));
                    removed_one = true;
                }
            }

            if removed_one {
                state.total_active = state.total_active.saturating_sub(1);
                if let Some(timestamps) = state.timestamps.get_mut(&seq_no) {
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
                        state.timestamps.remove(&seq_no);
                    }
                }
                self.active_count.fetch_sub(1, Ordering::SeqCst);
            }

            if state.total_active == 0 && state.active.is_empty() {
                tenants.remove(&tenant);
            }
        }

        self.update_min_locked(&tenants);
    }

    /// Widerruft alle Snapshots mit `seq <= erased_through_seq` (D-11).
    pub fn revoke_through(&self, erased_through_seq: u64) -> usize {
        let clean_seq = erased_through_seq & !TOMBSTONE_BIT;
        let tenants = self.tenants.lock();
        let mut revoked = self.revoked_tags.lock();
        let mut count: usize = 0;

        for (&tenant, state) in tenants.iter() {
            for &seq in state.active.keys() {
                if seq <= clean_seq && revoked.insert((tenant, seq)) {
                    count = count.saturating_add(1);
                }
            }
        }
        count
    }

    /// Markiert Snapshots, die das Prädikat erfüllen, als widerrufen.
    pub fn revoke_where<F>(&self, predicate: F) -> usize
    where
        F: Fn(u64) -> bool,
    {
        let tenants = self.tenants.lock();
        let mut revoked = self.revoked_tags.lock();
        let mut count: usize = 0;

        for (&tenant, state) in tenants.iter() {
            for &seq in state.active.keys() {
                if predicate(seq) && revoked.insert((tenant, seq)) {
                    count = count.saturating_add(1);
                }
            }
        }
        count
    }

    /// Findet den am längsten aktiven Snapshot bezüglich des Nanosekunden-Zeitstempels `now_nanos`.
    pub fn longest_active_pin_at(&self, now_nanos: u64) -> Option<(u64, Duration)> {
        let tenants = self.tenants.lock();
        let mut longest: Option<(u64, Duration)> = None;

        for state in tenants.values() {
            for (&seq_no, timestamps) in state.timestamps.iter() {
                for &ts in timestamps {
                    let duration = Duration::from_nanos(now_nanos.saturating_sub(ts));
                    match longest {
                        None => longest = Some((seq_no, duration)),
                        Some((_, max_duration)) if duration > max_duration => {
                            longest = Some((seq_no, duration));
                        }
                        _ => {}
                    }
                }
            }
        }

        longest
    }

    /// Findet den am längsten aktiven Snapshot bezogen auf die aktuelle Systemuhr.
    pub fn longest_active_pin(&self) -> Option<(u64, Duration)> {
        self.longest_active_pin_at(self.clock.monotonic_nanos())
    }

    /// Bereinigt und entfernt alle aktiven Snapshot-Pins, die älter als `max_pin_duration` sind.
    pub fn reap_expired_pins_at(&self, max_pin_duration: Duration, now_nanos: u64) -> Vec<u64> {
        let mut expired_seqs = Vec::new();
        let mut tenants = self.tenants.lock();
        let mut total_reaped: usize = 0;

        for state in tenants.values_mut() {
            let mut seqs_to_remove = Vec::new();
            for (&seq_no, timestamps) in state.timestamps.iter_mut() {
                let initial_len = timestamps.len();
                timestamps.retain(|&ts| {
                    let elapsed = Duration::from_nanos(now_nanos.saturating_sub(ts));
                    if elapsed > max_pin_duration {
                        expired_seqs.push(seq_no);
                        false
                    } else {
                        true
                    }
                });
                let removed_from_this_seq = initial_len.saturating_sub(timestamps.len());
                if removed_from_this_seq > 0 {
                    if let Some(count) = state.active.get_mut(&seq_no) {
                        *count = count.saturating_sub(removed_from_this_seq);
                        if *count == 0 {
                            seqs_to_remove.push(seq_no);
                        }
                    }
                    state.total_active = state.total_active.saturating_sub(removed_from_this_seq);
                    total_reaped = total_reaped.saturating_add(removed_from_this_seq);
                }
            }

            for seq in seqs_to_remove {
                state.active.remove(&seq);
                state.timestamps.remove(&seq);
            }
        }

        tenants.retain(|_, state| state.total_active > 0);

        self.active_count.fetch_sub(total_reaped, Ordering::SeqCst);
        self.update_min_locked(&tenants);

        expired_seqs.sort_unstable();
        expired_seqs.dedup();
        expired_seqs
    }

    /// Bereinigt alle überalterten Snapshot-Pins bezogen auf die aktuelle Uhrzeit.
    pub fn reap_expired_pins(&self, max_pin_duration: Duration) -> Vec<u64> {
        let now_nanos = self.clock.monotonic_nanos();
        self.reap_expired_pins_at(max_pin_duration, now_nanos)
    }

    fn update_min_locked(&self, tenants: &BTreeMap<TenantTag, TenantState>) {
        let mut min_val: Option<Seq> = None;
        for state in tenants.values() {
            if let Some(&first_seq) = state.active.keys().next() {
                min_val = match min_val {
                    None => Some(first_seq),
                    Some(curr_min) => Some(curr_min.min(first_seq)),
                };
            }
        }

        if let Some(min) = min_val {
            self.min_active_seqno.store(min, Ordering::Release);
        } else {
            self.min_active_seqno.store(0, Ordering::Release);
        }
    }
}

// ANCHOR[contextra-mvcc:SNAPSHOT-HANDLE]
/// RAII-Handle für einen aktiven Lese-Snapshot eines Mandanten.
#[derive(Debug)]
pub struct SnapshotHandle {
    registry: Arc<SnapshotRegistry>,
    tenant: TenantTag,
    seq: Seq,
    created_at: Option<u64>,
    revoked: Arc<AtomicBool>,
}

impl SnapshotHandle {
    /// Liefert die Sequenznummer des Snapshots oder `Err(MvccError::SnapshotRevoked)`,
    /// falls der Snapshot widerrufen wurde (MV-05).
    pub fn seq(&self) -> Result<Seq, MvccError> {
        if self.revoked.load(Ordering::Acquire)
            || self.registry.is_revoked_tenant(self.tenant, self.seq)
        {
            self.revoked.store(true, Ordering::Release);
            Err(MvccError::SnapshotRevoked(self.seq))
        } else {
            Ok(self.seq)
        }
    }

    /// Mandanten-Tag dieses Handles.
    pub fn tenant(&self) -> TenantTag {
        self.tenant
    }
}

impl Drop for SnapshotHandle {
    fn drop(&mut self) {
        self.registry
            .release_tenant_at(self.tenant, self.seq, self.created_at);
    }
}

// ANCHOR[contextra-mvcc:SNAPSHOT-GUARD]
/// Veralteter RAII-Guard für Abwärtskompatibilität.
pub struct SnapshotGuard {
    registry: Arc<SnapshotRegistry>,
    seq_no: u64,
    created_at: Option<u64>,
}

impl SnapshotGuard {
    /// Liefert die Sequenznummer.
    pub fn seq_no(&self) -> u64 {
        self.seq_no
    }

    /// Prüft, ob dieser Snapshot widerrufen wurde.
    pub fn is_revoked(&self) -> bool {
        self.registry.is_revoked(self.seq_no)
    }

    /// Validiert den Snapshot.
    pub fn validate(&self) -> Result<(), MvccError> {
        if self.is_revoked() {
            Err(MvccError::SnapshotRevoked(self.seq_no))
        } else {
            Ok(())
        }
    }
}

impl Drop for SnapshotGuard {
    fn drop(&mut self) {
        self.registry.release_at(self.seq_no, self.created_at);
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use contextra_testkit::ManualClock;
    use proptest::prop_assert_eq;

    #[test]
    fn test_snapshot_registry_basic() {
        let registry = Arc::new(SnapshotRegistry::new());
        assert_eq!(registry.min_read_snapshot(), None);

        let guard = registry.register(100);
        assert_eq!(guard.seq_no(), 100);
        assert_eq!(registry.min_read_snapshot(), Some(100));

        drop(guard);
        assert_eq!(registry.min_read_snapshot(), None);
    }

    #[test]
    fn test_multiple_snapshots_min_calc() {
        let registry = Arc::new(SnapshotRegistry::new());
        let _g1 = registry.register(200);
        let g2 = registry.register(100);
        let _g3 = registry.register(300);

        assert_eq!(registry.min_read_snapshot(), Some(100));

        drop(g2);
        assert_eq!(registry.min_read_snapshot(), Some(200));
    }

    #[test]
    fn test_pin_unpin() {
        let registry = Arc::new(SnapshotRegistry::new());
        registry.pin(50);
        assert_eq!(registry.min_read_snapshot(), Some(50));

        let g = registry.register(100);
        assert_eq!(registry.min_read_snapshot(), Some(50));

        registry.unpin(50);
        assert_eq!(registry.min_read_snapshot(), Some(100));

        drop(g);
        assert_eq!(registry.min_read_snapshot(), None);
    }

    #[test]
    fn test_seq_no_tombstone_masking() {
        let registry = Arc::new(SnapshotRegistry::new());
        let seq = 100 | crate::types::TOMBSTONE_BIT;
        let guard = registry.register(seq);

        assert_eq!(guard.seq_no(), 100);
        assert_eq!(registry.min_read_snapshot(), Some(100));
    }

    #[test]
    fn test_ref_counting() {
        let registry = Arc::new(SnapshotRegistry::new());
        let g1 = registry.register(100);
        let g2 = registry.register(100);

        assert_eq!(registry.min_read_snapshot(), Some(100));

        drop(g1);
        assert_eq!(registry.min_read_snapshot(), Some(100));

        drop(g2);
        assert_eq!(registry.min_read_snapshot(), None);
    }

    #[test]
    fn test_unpin_unknown_seq_no_does_not_panic() {
        let registry = Arc::new(SnapshotRegistry::new());
        registry.unpin(999);
        assert_eq!(registry.min_read_snapshot(), None);
    }

    #[test]
    fn test_snapshot_panic_free_safety() {
        let registry = Arc::new(SnapshotRegistry::new());
        assert_eq!(registry.min_read_snapshot(), None);
        registry.release(12345);
        registry.unpin(12345);
        assert_eq!(registry.min_read_snapshot(), None);
    }

    #[test]
    fn test_snapshot_revocation() {
        let registry = Arc::new(SnapshotRegistry::new());
        let g1 = registry.register(50);
        let g2 = registry.register(100);

        assert!(!g1.is_revoked());
        assert!(!g2.is_revoked());
        assert!(g1.validate().is_ok());

        let count = registry.revoke_through(50);
        assert_eq!(count, 1);

        assert!(g1.is_revoked());
        assert!(!g2.is_revoked());
        assert_eq!(
            g1.validate(),
            Err(crate::error::MvccError::SnapshotRevoked(50))
        );
        assert!(g2.validate().is_ok());

        drop(g1);
        assert!(!registry.is_revoked(50));
    }

    #[test]
    fn test_double_registration_and_sequential_drop() {
        let registry = Arc::new(SnapshotRegistry::new());
        let g1 = registry.register(42);
        let g2 = registry.register(42);

        assert_eq!(registry.min_read_snapshot(), Some(42));

        drop(g1);
        assert_eq!(registry.min_read_snapshot(), Some(42));

        drop(g2);
        assert_eq!(registry.min_read_snapshot(), None);
    }

    #[test]
    fn test_snapshot_registry_reap_expired_pins() {
        let clock = Arc::new(ManualClock::new(10_000_000_000));
        let registry = Arc::new(SnapshotRegistry::with_clock(clock.clone()));

        registry.pin(100);
        assert_eq!(registry.min_read_snapshot(), Some(100));

        clock.advance(Duration::from_secs(10));
        registry.pin(200);
        assert_eq!(registry.min_read_snapshot(), Some(100));

        clock.advance(Duration::from_secs(295));
        let reaped = registry.reap_expired_pins(Duration::from_secs(300));
        assert_eq!(reaped, vec![100]);
        assert_eq!(registry.min_read_snapshot(), Some(200));
    }

    proptest::proptest! {
        #[test]
        fn prop_snapshot_registry_min_active(
            seqs in proptest::collection::vec(0..1000u64, 1..50)
        ) {
            let registry = Arc::new(SnapshotRegistry::new());
            let mut guards = Vec::new();

            for &seq in &seqs {
                guards.push(registry.register(seq));
            }

            let min_expected = match seqs.iter().min() {
                Some(&m) => m,
                None => 0,
            };
            prop_assert_eq!(registry.min_read_snapshot(), Some(min_expected));

            guards.pop();

            if guards.is_empty() {
                prop_assert_eq!(registry.min_read_snapshot(), None);
            } else {
                let remaining_min = guards.iter().map(|g| g.seq_no()).min();
                prop_assert_eq!(registry.min_read_snapshot(), remaining_min);
            }
        }

        #[test]
        fn prop_snapshot_register_unregister_stress(
            seqs in proptest::collection::vec(0..5000u64, 2..80),
            drop_order_seed in proptest::collection::vec(0..1000usize, 2..80),
        ) {
            let registry = Arc::new(SnapshotRegistry::new());
            let mut guards: Vec<Option<SnapshotGuard>> = Vec::new();

            for &seq in &seqs {
                guards.push(Some(registry.register(seq)));
            }

            let mut active_seqs: Vec<u64> = seqs.clone();
            active_seqs.sort_unstable();

            if let Some(&first_seq) = active_seqs.first() {
                prop_assert_eq!(registry.min_read_snapshot(), Some(first_seq));
            }

            let mut remaining_indices: Vec<usize> = (0..guards.len()).collect();
            for seed_val in &drop_order_seed {
                if remaining_indices.is_empty() {
                    break;
                }
                let idx_in_remaining = seed_val % remaining_indices.len();
                let guard_idx = remaining_indices.remove(idx_in_remaining);

                let seq_val = if let Some(g) = guards.get(guard_idx).and_then(|opt| opt.as_ref()) {
                    g.seq_no()
                } else {
                    continue;
                };
                if let Some(guard_slot) = guards.get_mut(guard_idx) {
                    *guard_slot = None;
                }

                if let Some(pos) = active_seqs.iter().position(|&s| s == seq_val) {
                    active_seqs.remove(pos);
                }

                let expected_min = active_seqs.first().copied();
                prop_assert_eq!(registry.min_read_snapshot(), expected_min);
            }
        }

        #[test]
        fn prop_snapshot_pin_unpin_interleaving(
            pin_seqs in proptest::collection::vec(0..500u64, 1..20),
            guard_seqs in proptest::collection::vec(0..500u64, 1..20),
        ) {
            use std::collections::BTreeMap;
            let registry = Arc::new(SnapshotRegistry::new());
            let mut ref_counts: BTreeMap<u64, usize> = BTreeMap::new();

            for &seq in &pin_seqs {
                registry.pin(seq);
                *ref_counts.entry(seq).or_default() += 1;
            }

            let mut guards = Vec::new();
            for &seq in &guard_seqs {
                guards.push(registry.register(seq));
                *ref_counts.entry(seq).or_default() += 1;
            }

            let expected_min = ref_counts.keys().next().copied();
            prop_assert_eq!(registry.min_read_snapshot(), expected_min);

            for &seq in &pin_seqs {
                registry.unpin(seq);
                if let Some(count) = ref_counts.get_mut(&seq) {
                    *count -= 1;
                    if *count == 0 {
                        ref_counts.remove(&seq);
                    }
                }
            }

            let expected_min2 = ref_counts.keys().next().copied();
            prop_assert_eq!(registry.min_read_snapshot(), expected_min2);

            drop(guards);
            prop_assert_eq!(registry.min_read_snapshot(), None);
        }
    }
}
