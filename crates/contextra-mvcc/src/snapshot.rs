//! # SnapshotRegistry — Verwaltung aktiver MVCC-Lesesnapshots
//!
//! **Ring:** Ring 0 (Transaktionsisolation & Speicher-Kern)
//! **Klasse:** Kern `[K]`
//!
//! ## Zweck & Architektur-Rolle
//!
//! Die [`SnapshotRegistry`] verwaltet alle aktuell aktiven Lese-Transaktionen und berechnet
//! atomar die minimal aktive Sequenznummer (`min_active_seqno`).
//!
//! In einem LSM-Tree (Log-Structured Merge-Tree wie in `contextra-store`) dürfen gelöschte Einträge
//! (Tombstones) erst dann physisch bereinigt werden (Garbage Collection / Kompaktierung), wenn kein
//! aktiver Leseschnappschuss mehr existiert, der diese Daten noch im Zustand vor der Löschung sehen können muss.
//!
//! ### Garantien & Invarianten
//! - **REQ-MVCC-002 / RAII-Garantie:** Das Erzeugen eines Lesers liefert einen [`SnapshotGuard`].
//!   Wird dieser verworfen (`Drop`), deregistriert er sich automatisch in der Registry.
//! - **Tombstone-Schutz:** Solange ein `SnapshotGuard` oder persistenter Pin für eine Sequenznummer $S$
//!   existiert, gilt `min_active_seqno <= S`.
//! - **Lock-freie Leser:** Hintergrundthreads (z. B. LSM-Kompaktierungs-Worker) können `min_active_seqno()`
//!   völlig lock-frei über atomare Operationen mit `Acquire`-Semantik abfragen, ohne Schreiber zu blockieren.

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
use contextra_ports::{Clock, SystemClock};
use parking_lot::Mutex;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

// ANCHOR[contextra-mvcc:SNAP-001]
/// Zentrale Registrierungskomponente für aktive MVCC-Lesesnapshots.
///
/// ### Synchronisations- und Speicherordnungs-Strategie:
/// - **Single-Writer Exklusivität via Mutex**: Alle Mutationen an den aktiven Snapshots
///   (`register`, `pin`, `release`) halten den `active`-Mutex von `parking_lot`.
///   Dies garantiert strikte Serialisierung der Zustandsübergänge.
/// - **Lock-freie Leser via Acquire/Release**: `min_active_seqno()` liest atomar per `Ordering::Acquire`.
///   Schreiber aktualisieren `min_active_seqno` per `Ordering::Release`. Dadurch sehen Leser stets
///   einen konsistenten Stand aller vorangegangenen Speicheroperationen.
pub struct SnapshotRegistry {
    /// Sortierte Tabelle aktiver Sequenznummern auf ihre Erstellungs-Nanosekunden.
    /// Mehrere Leser können dieselbe Sequenznummer halten (daher `Vec<u64>`).
    active: Mutex<BTreeMap<u64, Vec<u64>>>,
    /// Zwischengespeicherte minimal aktive Sequenznummer für lock-freien Zugriff.
    min_active_seqno: AtomicU64,
    /// Deterministischer Zeit-Port.
    clock: Arc<dyn Clock>,
}

impl std::fmt::Debug for SnapshotRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SnapshotRegistry")
            .field("active", &self.active)
            .field("min_active_seqno", &self.min_active_seqno)
            .finish()
    }
}

impl Default for SnapshotRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl SnapshotRegistry {
    // ANCHOR[contextra-mvcc:SNAP-002]
    /// Erstellt eine neue, leere SnapshotRegistry mit [`SystemClock`].
    pub fn new() -> Self {
        Self::with_clock(Arc::new(SystemClock::new()))
    }

    // ANCHOR[contextra-mvcc:SNAP-003]
    /// Erstellt eine neue SnapshotRegistry unter Verwendung der übergebenen [`Clock`]-Instanz.
    ///
    /// Dient der Unterstützung deterministischer Zeittests ohne Warten auf reale Wanduhrzeit.
    pub fn with_clock(clock: Arc<dyn Clock>) -> Self {
        Self {
            active: Mutex::new(BTreeMap::new()),
            min_active_seqno: AtomicU64::new(u64::MAX),
            clock,
        }
    }

    // ANCHOR[contextra-mvcc:SNAP-004]
    /// Liefert den aktiven [`Clock`]-Port dieser Registry.
    pub fn clock(&self) -> &Arc<dyn Clock> {
        &self.clock
    }

    // ANCHOR[contextra-mvcc:SNAP-005]
    /// Registriert einen neuen Lese-Snapshot für die gegebene Sequenznummer `seq_no`.
    ///
    /// # Ablauf
    /// 1. Aktuelle Monotoniezeit über `Clock`-Port abrufen.
    /// 2. Aufruf von `register_at` mit diesem Zeitstempel.
    /// 3. Liefert einen RAII-Guard ([`SnapshotGuard`]) zurück.
    pub fn register(self: &Arc<Self>, seq_no: u64) -> SnapshotGuard {
        let now_nanos = self.clock.monotonic_nanos();
        self.register_at(seq_no, now_nanos)
    }

    // ANCHOR[contextra-mvcc:SNAP-006]
    /// Registriert einen Lese-Snapshot zu einem expliziten Nanosekunden-Zeitstempel `at_nanos`.
    ///
    /// # Ablauf
    /// 1. Maskiere das `TOMBSTONE_BIT` aus `seq_no` heraus.
    /// 2. Sperre den internen `active`-Mutex exklusiv.
    /// 3. Füge den Zeitstempel in die Liste der aktiven Snapshots für `seq_no` ein.
    /// 4. Aktualisiere atomar `min_active_seqno` via `update_min`.
    /// 5. Gebe den `SnapshotGuard` zurück.
    pub fn register_at(self: &Arc<Self>, seq_no: u64, at_nanos: u64) -> SnapshotGuard {
        let seq_no = seq_no & !TOMBSTONE_BIT;
        let mut active = self.active.lock();
        active.entry(seq_no).or_default().push(at_nanos);
        self.update_min(&active);
        SnapshotGuard {
            registry: self.clone(),
            seq_no,
            created_at: Some(at_nanos),
        }
    }

    // ANCHOR[contextra-mvcc:SNAP-007]
    /// Liefert die minimal aktive Sequenznummer (`u64::MAX`, falls keine Snapshots aktiv sind).
    ///
    /// Nutzt atomares Laden mit `Ordering::Acquire`, um mit `Ordering::Release`-Speichervorgängen
    /// in `update_min` zu synchronisieren. Dies garantiert, dass Hintergrund-Worker (z. B. Compaction
    /// in `contextra-store`) den GC-Wasserstand abfragen können, ohne den Mutex zu sperren.
    #[inline]
    pub fn min_active_seqno(&self) -> u64 {
        self.min_active_seqno.load(Ordering::Acquire)
    }

    // ANCHOR[contextra-mvcc:SNAP-008]
    /// Erzeugt einen persistenten Pin für eine Sequenznummer (z. B. für SAOS Checkpoints oder Backups).
    ///
    /// Im Gegensatz zu `register()` liefert `pin()` keinen RAII-Guard; der Pin bleibt aktiv,
    /// bis er explizit via `unpin()` aufgehoben wird.
    pub fn pin(&self, seq_no: u64) {
        let now_nanos = self.clock.monotonic_nanos();
        self.pin_at(seq_no, now_nanos);
    }

    // ANCHOR[contextra-mvcc:SNAP-009]
    /// Erzeugt einen persistenten Pin zu einem expliziten Nanosekunden-Zeitstempel `at_nanos`.
    pub fn pin_at(&self, seq_no: u64, at_nanos: u64) {
        let seq_no = seq_no & !TOMBSTONE_BIT;
        let mut active = self.active.lock();
        active.entry(seq_no).or_default().push(at_nanos);
        self.update_min(&active);
    }

    // ANCHOR[contextra-mvcc:SNAP-010]
    /// Entfernt einen zuvor gesetzten persistenten Pin.
    pub fn unpin(&self, seq_no: u64) {
        self.release(seq_no);
    }

    // ANCHOR[contextra-mvcc:SNAP-011]
    /// Interne Freigabe einer Sequenznummer ohne expliziten Erstellungszeitstempel.
    pub(crate) fn release(&self, seq_no: u64) {
        self.release_at(seq_no, None);
    }

    // ANCHOR[contextra-mvcc:SNAP-012]
    /// Gibt eine Sequenznummer frei, optional mit passendem Erstellungs-Zeitstempel.
    ///
    /// # Ablauf
    /// 1. Maskiere `TOMBSTONE_BIT` aus `seq_no`.
    /// 2. Sperre den `active`-Mutex exklusiv.
    /// 3. Entferne den passenden Eintrag aus der Zeitstempelliste (Swap-Remove bei Treffer, sonst Pop).
    /// 4. Ist die Liste für diese Sequenznummer leer, entferne den Key aus dem `BTreeMap`.
    /// 5. Aktualisiere atomar `min_active_seqno`.
    ///
    /// # Fehler-Robustheit
    /// Das Freigeben einer nicht registrierten Sequenznummer ist ein No-Op (Regel 2: Zero-Panic).
    pub(crate) fn release_at(&self, seq_no: u64, created_at: Option<u64>) {
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
            // NOTE: Freigabe einer unbekannten Sequenznummer wird ignoriert, um Panics zu verhindern.
        }
        self.update_min(&active);
    }

    // ANCHOR[contextra-mvcc:SNAP-013]
    /// Identifiziert den ältesten aktiven Snapshot bezogen auf `now_nanos`.
    ///
    /// Gibt `Some((seq_no, duration))` oder `None` zurück, falls keine Snapshots aktiv sind.
    pub fn longest_active_pin_at(&self, now_nanos: u64) -> Option<(u64, Duration)> {
        let active = self.active.lock();
        let mut longest: Option<(u64, Duration)> = None;

        for (&seq_no, timestamps) in active.iter() {
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

        longest
    }

    // ANCHOR[contextra-mvcc:SNAP-014]
    /// Identifiziert den am längsten gehaltenen aktiven Snapshot zur aktuellen Monotoniezeit.
    pub fn longest_active_pin(&self) -> Option<(u64, Duration)> {
        self.longest_active_pin_at(self.clock.monotonic_nanos())
    }

    // ANCHOR[contextra-mvcc:SNAP-015]
    /// Bereinigt und deregistriert alle Snapshot-Pins, die älter als `max_pin_duration` sind, bezogen auf `now_nanos`.
    ///
    /// Liefert die Liste aller abgelaufenen, bereinigten Sequenznummern (sortiert und dedupliziert).
    pub fn reap_expired_pins_at(&self, max_pin_duration: Duration, now_nanos: u64) -> Vec<u64> {
        let max_nanos = u64::try_from(max_pin_duration.as_nanos()).unwrap_or(u64::MAX);
        let mut expired_seqs = Vec::new();
        let mut active = self.active.lock();

        let mut seqs_to_remove = Vec::new();
        for (&seq_no, timestamps) in active.iter_mut() {
            timestamps.retain(|&ts| {
                if now_nanos.saturating_sub(ts) > max_nanos {
                    expired_seqs.push(seq_no);
                    false
                } else {
                    true
                }
            });
            if timestamps.is_empty() {
                seqs_to_remove.push(seq_no);
            }
        }

        for seq in seqs_to_remove {
            active.remove(&seq);
        }

        self.update_min(&active);
        expired_seqs.sort_unstable();
        expired_seqs.dedup();
        expired_seqs
    }

    // ANCHOR[contextra-mvcc:SNAP-016]
    /// Bereinigt alle Snapshot-Pins, die älter als `max_pin_duration` sind, basierend auf der aktuellen Uhrzeit.
    pub fn reap_expired_pins(&self, max_pin_duration: Duration) -> Vec<u64> {
        let now_nanos = self.clock.monotonic_nanos();
        self.reap_expired_pins_at(max_pin_duration, now_nanos)
    }

    // ANCHOR[contextra-mvcc:SNAP-017]
    /// Interne Hilfsfunktion zur Neuberechnung von `min_active_seqno`.
    ///
    /// Wird stets unter exklusivem Halten des `active`-Mutex aufgerufen.
    fn update_min(&self, active: &BTreeMap<u64, Vec<u64>>) {
        // NOTE: u64::MAX ist der Standard, wenn keine Snapshots aktiv sind.
        // Dadurch darf die LSM-Kompaktierung alle Tombstones bereinigen.
        //
        // Memory Ordering: Ordering::Release publiziert die Aktualisierung für Leser mit Ordering::Acquire.
        let min = active.keys().next().copied().unwrap_or(u64::MAX);
        self.min_active_seqno.store(min, Ordering::Release);
    }
}

// ANCHOR[contextra-mvcc:SNAP-018]
/// RAII-Guard für einen aktiven Lese-Snapshot.
///
/// Hält die Snapshot-Registrierung aufrecht, solange dieser Guard im Scope ist.
/// Beim Verwerfen (`Drop`) wird der Snapshot automatisch deregistriert.
pub struct SnapshotGuard {
    registry: Arc<SnapshotRegistry>,
    seq_no: u64,
    created_at: Option<u64>,
}

impl SnapshotGuard {
    // ANCHOR[contextra-mvcc:SNAP-019]
    /// Gibt die durch diesen Guard geschützte Sequenznummer zurück.
    pub fn seq_no(&self) -> u64 {
        self.seq_no
    }
}

impl Drop for SnapshotGuard {
    // ANCHOR[contextra-mvcc:SNAP-020]
    /// Automatische Deregistrierung beim Verlassen des Gültigkeitsbereichs.
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

    // INTENT: Snapshot-Registry Lifecycle verified by 5 unit tests.
    #[test]
    fn test_snapshot_registry_basic() {
        let registry = Arc::new(SnapshotRegistry::new());
        assert_eq!(registry.min_active_seqno(), u64::MAX);

        let guard = registry.register(100);
        assert_eq!(guard.seq_no(), 100);
        assert_eq!(registry.min_active_seqno(), 100);

        drop(guard);
        assert_eq!(registry.min_active_seqno(), u64::MAX);
    }

    #[test]
    fn test_multiple_snapshots_min_calc() {
        let registry = Arc::new(SnapshotRegistry::new());
        let _g1 = registry.register(200);
        let g2 = registry.register(100);
        let _g3 = registry.register(300);

        assert_eq!(registry.min_active_seqno(), 100);

        drop(g2);
        assert_eq!(registry.min_active_seqno(), 200);
    }

    #[test]
    fn test_pin_unpin() {
        let registry = Arc::new(SnapshotRegistry::new());
        registry.pin(50);
        assert_eq!(registry.min_active_seqno(), 50);

        let g = registry.register(100);
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
        let guard = registry.register(seq);

        assert_eq!(guard.seq_no(), 100);
        assert_eq!(registry.min_active_seqno(), 100);
    }

    #[test]
    fn test_ref_counting() {
        let registry = Arc::new(SnapshotRegistry::new());
        let g1 = registry.register(100);
        let g2 = registry.register(100);

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
        let g1 = registry.register(42);
        let g2 = registry.register(42);

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

    #[test]
    fn test_snapshot_registry_reap_expired_pins() {
        let clock = Arc::new(ManualClock::new(10_000_000_000));
        let registry = Arc::new(SnapshotRegistry::with_clock(clock.clone()));

        registry.pin(100);
        assert_eq!(registry.min_active_seqno(), 100);

        // Advance clock by 10 seconds
        clock.advance(Duration::from_secs(10));

        // Pin 200 at t = 10s
        registry.pin(200);
        assert_eq!(registry.min_active_seqno(), 100);

        // Advance clock by 295s (total elapsed for pin 100: 305s > 300s max_pin_duration)
        clock.advance(Duration::from_secs(295));

        // Reap expired pins with 300s limit
        let reaped = registry.reap_expired_pins(Duration::from_secs(300));
        assert_eq!(reaped, vec![100]);
        assert_eq!(registry.min_active_seqno(), 200);
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

            let min_expected = *seqs.iter().min().unwrap(); // #[cfg(test)]
            prop_assert_eq!(registry.min_active_seqno(), min_expected);

            guards.pop(); // Drop last element

            // If all elements dropped, min_active is MAX, else it's min of remaining
            if guards.is_empty() {
                prop_assert_eq!(registry.min_active_seqno(), u64::MAX);
            } else {
                let remaining_min = guards.iter().map(|g| g.seq_no()).min().unwrap_or(u64::MAX);
                prop_assert_eq!(registry.min_active_seqno(), remaining_min);
            }
        }

        /// Proptest: Proves that dropping guards one-by-one in arbitrary order
        /// always maintains the correct min_active_seqno invariant.
        ///
        /// # Anti-Mirroring
        /// Expected min is computed by maintaining an independent sorted list
        /// of remaining sequence numbers, not by calling SnapshotRegistry methods.
        #[test]
        fn prop_snapshot_register_unregister_stress(
            seqs in proptest::collection::vec(0..5000u64, 2..80),
            // Indices into the guards vec to determine drop order
            drop_order_seed in proptest::collection::vec(0..1000usize, 2..80),
        ) {
            let registry = Arc::new(SnapshotRegistry::new());
            let mut guards: Vec<Option<SnapshotGuard>> = Vec::new();

            // Register all
            for &seq in &seqs {
                guards.push(Some(registry.register(seq)));
            }

            // Build independent reference: sorted multiset of active seqs
            let mut active_seqs: Vec<u64> = seqs.clone();
            active_seqs.sort_unstable();

            // Verify initial state
            if let Some(&first_seq) = active_seqs.first() {
                prop_assert_eq!(registry.min_active_seqno(), first_seq);
            }

            // Drop guards one-by-one using the seed to pick which to drop
            let mut remaining_indices: Vec<usize> = (0..guards.len()).collect();
            for seed_val in &drop_order_seed {
                if remaining_indices.is_empty() {
                    break;
                }
                let idx_in_remaining = seed_val % remaining_indices.len();
                let guard_idx = remaining_indices.remove(idx_in_remaining);

                // Drop the guard
                let seq_val = if let Some(g) = guards.get(guard_idx).and_then(|opt| opt.as_ref()) {
                    g.seq_no()
                } else {
                    continue;
                };
                if let Some(guard_slot) = guards.get_mut(guard_idx) {
                    *guard_slot = None;
                }

                // Remove from reference (one occurrence only)
                if let Some(pos) = active_seqs.iter().position(|&s| s == seq_val) {
                    active_seqs.remove(pos);
                }

                // Verify invariant
                let expected_min = active_seqs.first().copied().unwrap_or(u64::MAX);
                prop_assert_eq!(
                    registry.min_active_seqno(),
                    expected_min,
                    "After dropping guard for seq={}, min should be {}",
                    seq_val,
                    expected_min
                );
            }
        }

        /// Proptest: Pin/Unpin combined with register/drop.
        /// Proves that persistent pins and RAII guards coexist correctly.
        ///
        /// # Anti-Mirroring
        /// Reference min is maintained in an independent BTreeMap<u64, usize>.
        #[test]
        fn prop_snapshot_pin_unpin_interleaving(
            pin_seqs in proptest::collection::vec(0..500u64, 1..20),
            guard_seqs in proptest::collection::vec(0..500u64, 1..20),
        ) {
            use std::collections::BTreeMap;
            let registry = Arc::new(SnapshotRegistry::new());
            let mut ref_counts: BTreeMap<u64, usize> = BTreeMap::new();

            // Pin all
            for &seq in &pin_seqs {
                registry.pin(seq);
                *ref_counts.entry(seq).or_default() += 1;
            }

            // Register guards
            let mut guards = Vec::new();
            for &seq in &guard_seqs {
                guards.push(registry.register(seq));
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

            // Drop all guards
            drop(guards);
            prop_assert_eq!(registry.min_active_seqno(), u64::MAX);
        }
    }
}
