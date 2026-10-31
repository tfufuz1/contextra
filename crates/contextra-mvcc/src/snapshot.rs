// ANCHOR[contextra-mvcc:SNAPSHOT-REGISTRY-ROOT]
//! # Snapshot-Registry (`SnapshotRegistry`) für MVCC-sichere Leseoperationen
//!
//! Die `SnapshotRegistry` verwaltet alle aktuell aktiven Lese-Snapshots (Schnappschüsse)
//! und berechnet atomar die minimal aktive Sequenznummer (`min_active_seqno`).
//!
//! ## Warum ist das essenziell? (GC-Schutz)
//! In einem MVCC-System erzeugen Löschungen sogenannte Tombstones (Grabsteine).
//! Der Garbage-Collector (LSM Compaction) darf Tombstones und ältere Datenversionen erst dann
//! physisch von der Festplatte entfernen, wenn kein aktiver Leser mehr existiert,
//! dessen Snapshot-Sequenznummer vor oder bei dieser Änderung liegt.
//! Solange ein [`SnapshotGuard`] existiert, garantiert die Registry, dass `min_active_seqno`
//! nicht über dessen `seq_no` ansteigt.
//!
//! ## Synchronisations- und Speicherarchitektur
//! - **Schreiber über Mutex**: Änderungen an den aktiven Snapshots erfordern den `active`-Mutex.
//!   Da Snapshot-Registrierungen kurz sind, bleibt die Lock-Haltedauer minimal.
//! - **Lock-freie Leser via AtomicU64 (Acquire/Release)**: Leser wie die LSM-Kompaktierung fragen
//!   `min_active_seqno()` extrem häufig ab. Dies geschieht vollständig lock-frei mittels
//!   `AtomicU64::load(Ordering::Acquire)`. Die Registry aktualisiert diesen Wert nach jeder
//!   Änderung mittels `AtomicU64::store(..., Ordering::Release)`.

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

// ANCHOR[contextra-mvcc:SNAPSHOT-REGISTRY-STRUCT]
/// Zentrale Registrierung für aktive Lese-Snapshots im System.
///
/// ### Synchronisationsstrategie:
/// - `active`: Ein sortierter `BTreeMap`, geschützt durch einen `parking_lot::Mutex`.
///   Schlüssel ist die Snapshot-Sequenznummer; Wert ist eine Liste von Nanosekunden-Zeitstempeln
///   aller Guards, die diesen Snapshot halten (Referenzzählung).
/// - `min_active_seqno`: Atomarer 64-Bit-Ganzzahlwert für lock-freies Auslesen durch Hintergrund-Threads.
/// - `clock`: Zeitport zur Erfassung der Haltedauer aktiver Snapshots.
pub struct SnapshotRegistry {
    active: Mutex<BTreeMap<u64, Vec<u64>>>,
    min_active_seqno: AtomicU64,
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
    // ANCHOR[contextra-mvcc:SNAPSHOT-REGISTRY-NEW]
    /// Erstellt eine neue, leere Snapshot-Registry mit Standard-[`SystemClock`].
    pub fn new() -> Self {
        Self::with_clock(Arc::new(SystemClock::new()))
    }

    /// Erstellt eine neue Snapshot-Registry mit benutzerdefiniertem [`Clock`]-Port (für Tests).
    pub fn with_clock(clock: Arc<dyn Clock>) -> Self {
        Self {
            active: Mutex::new(BTreeMap::new()),
            min_active_seqno: AtomicU64::new(u64::MAX),
            clock,
        }
    }

    /// Liefert eine Referenz auf den aktiven Zeitport ([`Clock`]).
    pub fn clock(&self) -> &Arc<dyn Clock> {
        &self.clock
    }

    // ANCHOR[contextra-mvcc:SNAPSHOT-REGISTRY-REGISTER]
    /// Registriert einen Lese-Snapshot und liefert einen RAII-Guard ([`SnapshotGuard`]) zurück.
    ///
    /// Sobald der Guard den Gültigkeitsbereich verlässt (Drop), wird der Snapshot
    /// automatisch deregistriert.
    ///
    /// ### Ablauf:
    /// 1. Zeitstempel über die Systemuhr erfassen.
    /// 2. Delegieren an [`SnapshotRegistry::register_at`].
    pub fn register(self: &Arc<Self>, seq_no: u64) -> SnapshotGuard {
        let now_nanos = self.clock.monotonic_nanos();
        self.register_at(seq_no, now_nanos)
    }

    /// Registriert einen Snapshot mit explizitem Zeitstempel (für deterministische Tests).
    ///
    /// ### Ablauf:
    /// 1. Tombstone-Bit aus der Sequenznummer entfernen.
    /// 2. Mutex auf `active` sperren.
    /// 3. Zeitstempel in die Liste für `seq_no` eintragen.
    /// 4. Minimalen aktiven Wasserstand via `update_min` atomar aktualisieren.
    /// 5. RAII-Guard erzeugen und zurückgeben.
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

    // ANCHOR[contextra-mvcc:SNAPSHOT-REGISTRY-MIN-ACTIVE]
    /// Liefert die kleinste Sequenznummer aller derzeit aktiven Snapshots (`u64::MAX`, falls keine aktiv).
    ///
    /// ### Nebenläufigkeit:
    /// Vollständig sperr- und allokationsfrei über atomares Laden mit `Ordering::Acquire`.
    /// Synchronisiert mit dem `Ordering::Release`-Speichern in `update_min`.
    ///
    /// ### Invariante:
    /// Der zurückgegebene Wert ist kleiner oder gleich der Sequenznummer jedes existierenden
    /// aktiven `SnapshotGuard`. Die LSM-Kompaktierung nutzt dies als untere Löschgrenze.
    #[inline]
    pub fn min_active_seqno(&self) -> u64 {
        self.min_active_seqno.load(Ordering::Acquire)
    }

    // ANCHOR[contextra-mvcc:SNAPSHOT-REGISTRY-PIN]
    /// Dauerhaftes Pinnen einer Sequenznummer (z. B. für SAOS Checkpoints oder Backups),
    /// ohne einen RAII-Guard zu halten.
    ///
    /// Muss manuell über [`SnapshotRegistry::unpin`] wieder freigegeben werden.
    pub fn pin(&self, seq_no: u64) {
        let now_nanos = self.clock.monotonic_nanos();
        self.pin_at(seq_no, now_nanos);
    }

    /// Pinnt eine Sequenznummer zu einem spezifischen Zeitstempel.
    pub fn pin_at(&self, seq_no: u64, at_nanos: u64) {
        let seq_no = seq_no & !TOMBSTONE_BIT;
        let mut active = self.active.lock();
        active.entry(seq_no).or_default().push(at_nanos);
        self.update_min(&active);
    }

    // ANCHOR[contextra-mvcc:SNAPSHOT-REGISTRY-UNPIN]
    /// Hebt einen dauerhaften Pin auf und bereinigt den minimal aktiven Wasserstand.
    pub fn unpin(&self, seq_no: u64) {
        self.release(seq_no);
    }

    // Interne Freigabe ohne Zeitstempelabgleich.
    pub(crate) fn release(&self, seq_no: u64) {
        self.release_at(seq_no, None);
    }

    // Interne Freigabe einer Registrierung unter Berücksichtigung des spezifischen Erstellungs-Zeitstempels.
    // Ablauf:
    // 1. Tombstone-Bit ausmaskieren.
    // 2. Zeitstempel aus dem Vektor entfernen (wenn gefunden swap_remove, sonst pop).
    // 3. Ist der Vektor für die Sequenznummer leer, Schlüssel aus der BTreeMap löschen.
    // 4. Minimalaktiven Wasserstand neu berechnen und atomar veröffentlichen.
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
            // NOTE: Unpinning oder Releasing einer ungetrackten Sequenznummer ist ein No-op (Regel 2: Zero-Panic).
        }
        self.update_min(&active);
    }

    // ANCHOR[contextra-mvcc:SNAPSHOT-REGISTRY-LONGEST-PIN]
    /// Findet den am längsten aktiven Snapshot bezüglich des Nanosekunden-Zeitstempels `now_nanos`.
    ///
    /// Liefert `Some((seq_no, duration))` oder `None`, falls aktuell keine Snapshots aktiv sind.
    /// Dient der Diagnose von "hängenden" Lese-Transaktionen, die die Garbage Collection blockieren.
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

    /// Findet den am längsten aktiven Snapshot bezogen auf die aktuelle Systemuhr.
    pub fn longest_active_pin(&self) -> Option<(u64, Duration)> {
        self.longest_active_pin_at(self.clock.monotonic_nanos())
    }

    // ANCHOR[contextra-mvcc:SNAPSHOT-REGISTRY-REAP-EXPIRED]
    /// Bereinigt und entfernt alle aktiven Snapshot-Pins, die älter als `max_pin_duration` sind.
    ///
    /// ### Ablauf:
    /// 1. Berechne Schwellenwert in Nanosekunden.
    /// 2. Iteriere über alle aktiven Snapshots und entferne überalterte Zeitstempel.
    /// 3. Entferne leere Einträge vollständig aus der Map.
    /// 4. Aktualisiere `min_active_seqno` atomar.
    /// 5. Gib die sortierte, deduplizierte Liste aller entfernten Sequenznummern zurück.
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

    /// Bereinigt alle überalterten Snapshot-Pins bezogen auf die aktuelle Uhrzeit.
    pub fn reap_expired_pins(&self, max_pin_duration: Duration) -> Vec<u64> {
        let now_nanos = self.clock.monotonic_nanos();
        self.reap_expired_pins_at(max_pin_duration, now_nanos)
    }

    // Aktualisiert die atomare minimale Sequenznummer.
    // NOTE: u64::MAX ist der korrekte Standardwert, wenn keine Snapshots aktiv sind.
    // Dadurch darf die LSM-Compaction ALLE Tombstones aufräumen, da alle Datensätze seq_no < u64::MAX haben.
    // Ordering::Release stellt sicher, dass alle Vorab-Änderungen an der Datenstruktur für Leser sichtbar werden.
    fn update_min(&self, active: &BTreeMap<u64, Vec<u64>>) {
        let min = active.keys().next().copied().unwrap_or(u64::MAX);
        self.min_active_seqno.store(min, Ordering::Release);
    }
}

// ANCHOR[contextra-mvcc:SNAPSHOT-GUARD]
/// RAII-Guard (Resource Acquisition Is Initialization) für einen aktiven Lese-Snapshot.
///
/// Hält einen Snapshot in der [`SnapshotRegistry`] gepinnt, solange die Instanz existiert.
/// Beim Verlassen des Gültigkeitsbereichs (`Drop`) wird der Snapshot automatisch deregistriert.
pub struct SnapshotGuard {
    registry: Arc<SnapshotRegistry>,
    seq_no: u64,
    created_at: Option<u64>,
}

impl SnapshotGuard {
    /// Liefert die Sequenznummer, die durch diesen Guard geschützt wird.
    pub fn seq_no(&self) -> u64 {
        self.seq_no
    }
}

impl Drop for SnapshotGuard {
    // Automatische Deregistrierung beim Droppen des Guards.
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

    // Prüft den Basis-Lebenszyklus eines SnapshotGuards: Registrierung, Minimum-Berechnung und automatisches Zurücksetzen beim Drop.
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

    // Prüft die korrekte Bestimmung der minimal aktiven Sequenznummer bei mehreren parallelen Snapshots.
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

    // Prüft das manuelle Pinnen und Entpinnen von Snapshots in Kombination mit RAII-Guards.
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

    // Prüft, dass gesetzte Tombstone-Bits in Sequenznummern bei der Registrierung transparent maskiert werden.
    #[test]
    fn test_seq_no_tombstone_masking() {
        let registry = Arc::new(SnapshotRegistry::new());
        // seq_no with tombstone bit set
        let seq = 100 | crate::types::TOMBSTONE_BIT;
        let guard = registry.register(seq);

        assert_eq!(guard.seq_no(), 100);
        assert_eq!(registry.min_active_seqno(), 100);
    }

    // Prüft die Referenzzählung bei mehrfacher Registrierung derselben Sequenznummer.
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

    // Prüft, dass das Entpinnen einer nie registrierten Sequenznummer fehlertolerant ohne Panic ignoriert wird.
    #[test]
    fn test_unpin_unknown_seq_no_does_not_panic() {
        let registry = Arc::new(SnapshotRegistry::new());
        // Unpinning a sequence number that was never pinned or registered must not panic
        registry.unpin(999);
        assert_eq!(registry.min_active_seqno(), u64::MAX);
    }

    // Prüft die Panic-Freiheit der Snapshot-Registry bei unerwarteten Freigabe-Aufrufen.
    #[test]
    fn test_snapshot_panic_free_safety() {
        let registry = Arc::new(SnapshotRegistry::new());
        assert_eq!(registry.min_active_seqno(), u64::MAX);
        registry.release(12345);
        registry.unpin(12345);
        assert_eq!(registry.min_active_seqno(), u64::MAX);
    }

    // Prüft Doppelregistrierung und sequentiellen Drop zweier Guards auf gleicher Sequenznummer.
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

    // Prüft das automatische Bereinigen überalterter Pins nach Ablauf der maximalen Haltedauer.
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
        // Eigenschaftstest: Verifiziert die minimale aktive Sequenznummer über zufällige Registrierungen.
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
        // Eigenschaftstest: Stresstest für Registrierung und Deregistrierung in zufälliger Reihenfolge gegen ein unabhängiges Modell.
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
        // Eigenschaftstest: Prüft die korrekte Koexistenz von persistenten Pins und RAII-Guards.
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
