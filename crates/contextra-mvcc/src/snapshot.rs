//! # Snapshot-Registry (`SnapshotRegistry`) für MVCC-sichere Leseoperationen
//!
//! Die `SnapshotRegistry` verwaltet alle aktuell aktiven Lese-Snapshots (Schnappschüsse)
//! und berechnet atomar die minimal aktive Sequenznummer (`min_active_seqno`).
//!
//! ## Warum ist das essenziell? (GC-Schutz)
//! In einem MVCC-System erzeugen Entfernungen (Deletions) sogenannte Tombstones (Grabsteine).
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
// ZWECK: SnapshotRegistry für MVCC-sichere Reads und minimal aktive Sequenznummern.
// INVARIANTEN: Solange SnapshotGuard lebt -> keine Tombstone-GC für seq >= guard.seq_no.
// HOTSPOTS: 40-110
// NICHT-OFFENSICHTLICH: Lock-freie Reads von min_active_seqno() via AtomicU64 (Acquire/Release).
// SIEHE AUCH: rules/tag_taxonomy.md, DECISIONS.md (ADR-024)

// INVARIANT: Snapshot-Registry schützt Reads vor Compaction-GC.
// INVARIANTE: Solange SnapshotGuard lebt → keine Tombstone-GC für seq >= guard.seq_no.
// RAII-PATTERN: Drop deregistriert automatisch. unwrap_or(u64::MAX) ist KORREKT.

use crate::types::{TenantId, TOMBSTONE_BIT};
use contextra_ports::{Clock, SystemClock};
use parking_lot::Mutex;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Zentrale Registrierung für aktive Lese-Snapshots im System.
///
/// ### Synchronisationsstrategie:
/// - `active`: Ein sortierter `BTreeMap`, geschützt durch einen `parking_lot::Mutex`.
///   Schlüssel ist die Snapshot-Sequenznummer; Wert ist eine Liste von Nanosekunden-Zeitstempeln
///   aller Guards, die diesen Snapshot halten (Referenzzählung).
/// - `min_active_seqno`: Atomarer 64-Bit-Ganzzahlwert für lock-freies Auslesen durch Hintergrund-Threads.
/// - `clock`: Zeitport zur Erfassung der Haltedauer aktiver Snapshots.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SnapshotMeta {
    tenant: Option<TenantId>,
    created_at: u64,
}

/// Zentrale Registrierung für aktive Lese-Snapshots im System.
pub struct SnapshotRegistry {
    active: Mutex<BTreeMap<u64, Vec<SnapshotMeta>>>,
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

    /// Erstellt eine neue Snapshot-Registry mit einer Zeitstempel-Closure (State-Passing per `Fn() -> u64`).
    pub fn with_time_fn<F>(time_fn: F) -> Self
    where
        F: Fn() -> u64 + Send + Sync + 'static,
    {
        struct ClosureClock<F>(F);
        impl<F: Fn() -> u64 + Send + Sync + 'static> Clock for ClosureClock<F> {
            fn now_unix_nanos(&self) -> u64 {
                (self.0)()
            }
            fn monotonic_nanos(&self) -> u64 {
                (self.0)()
            }
        }
        Self::with_clock(Arc::new(ClosureClock(time_fn)))
    }

    /// Liefert eine Referenz auf den aktiven Zeitport ([`Clock`]).
    pub fn clock(&self) -> &Arc<dyn Clock> {
        &self.clock
    }

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

    /// Registriert einen Lese-Snapshot für einen bestimmten Mandanten.
    pub fn register_for_tenant(self: &Arc<Self>, tenant: TenantId, seq_no: u64) -> SnapshotGuard {
        let now_nanos = self.clock.monotonic_nanos();
        self.register_for_tenant_at(tenant, seq_no, now_nanos)
    }

    /// Registriert einen Lese-Snapshot für einen bestimmten Mandanten mit explizitem Zeitstempel.
    pub fn register_for_tenant_at(
        self: &Arc<Self>,
        tenant: TenantId,
        seq_no: u64,
        at_nanos: u64,
    ) -> SnapshotGuard {
        let seq_no = seq_no & !TOMBSTONE_BIT;
        let mut active = self.active.lock();
        active.entry(seq_no).or_default().push(SnapshotMeta {
            tenant: Some(tenant),
            created_at: at_nanos,
        });
        self.update_min(&active);
        SnapshotGuard {
            registry: self.clone(),
            seq_no,
            created_at: Some(at_nanos),
            tenant: Some(tenant),
        }
    }

    /// Registriert einen Lese-Snapshot mit explizitem Zeitstempel ohne Mandantenzuordnung.
    pub fn register_at(self: &Arc<Self>, seq_no: u64, at_nanos: u64) -> SnapshotGuard {
        let seq_no = seq_no & !TOMBSTONE_BIT;
        let mut active = self.active.lock();
        active.entry(seq_no).or_default().push(SnapshotMeta {
            tenant: None,
            created_at: at_nanos,
        });
        self.update_min(&active);
        SnapshotGuard {
            registry: self.clone(),
            seq_no,
            created_at: Some(at_nanos),
            tenant: None,
        }
    }

    /// Liefert die kleinste Sequenznummer aller derzeit aktiven Snapshots (`u64::MAX`, falls keine aktiv).
    #[inline]
    pub fn min_active_seqno(&self) -> u64 {
        self.min_active_seqno.load(Ordering::Acquire)
    }

    /// Liefert die kleinste Snapshot-Sequenznummer als `Option<u64>` (ohne Sentinel).
    #[inline]
    pub fn min_read_snapshot(&self) -> Option<u64> {
        let val = self.min_active_seqno.load(Ordering::Acquire);
        if val == u64::MAX {
            None
        } else {
            Some(val)
        }
    }

    /// Berechnet die minimale Sequenznummer für die Kompaktierung (GC-Grenze) ohne Sentinel.
    #[inline]
    pub fn min_for_compaction(&self, current_seq: u64) -> u64 {
        self.min_read_snapshot().unwrap_or(current_seq)
    }

    /// Widerruft Snapshots im Bereich für einen Mandanten (Tenant-Isolation).
    pub fn revoke_covering(
        &self,
        tenant: TenantId,
        range: std::ops::Range<u64>,
        gated_seq: u64,
    ) -> usize {
        let mut active = self.active.lock();
        let mut revoked = 0;
        let mut empty_keys = Vec::new();

        for (&seq, metas) in active.iter_mut() {
            if seq >= range.start && seq < range.end && seq <= gated_seq {
                let initial_len = metas.len();
                metas.retain(|meta| meta.tenant != Some(tenant));
                revoked += initial_len - metas.len();
                if metas.is_empty() {
                    empty_keys.push(seq);
                }
            }
        }

        for seq in empty_keys {
            active.remove(&seq);
        }

        self.update_min(&active);
        revoked
    }

    /// Dauerhaftes Pinnen einer Sequenznummer (z. B. für SAOS Checkpoints oder Backups),
    /// ohne einen RAII-Guard zu halten.
    ///
    /// Muss manuell über [`SnapshotRegistry::unpin`] wieder freigegeben werden.
    pub fn pin(&self, seq_no: u64) {
        let now_nanos = self.clock.monotonic_nanos();
        self.pin_at(seq_no, now_nanos);
    }

    /// Pinnt eine Sequenznummer zu einem spezifischen Zeitstempel.
    /// Pinnt eine Sequenznummer für einen Mandanten bezogen auf die aktuelle Systemuhr.
    pub fn pin_for_tenant(&self, tenant: TenantId, seq_no: u64) {
        let now_nanos = self.clock.monotonic_nanos();
        self.pin_for_tenant_at(tenant, seq_no, now_nanos);
    }

    /// Pinnt eine Sequenznummer für einen Mandanten zu einem expliziten Zeitstempel.
    pub fn pin_for_tenant_at(&self, tenant: TenantId, seq_no: u64, at_nanos: u64) {
        let seq_no = seq_no & !TOMBSTONE_BIT;
        let mut active = self.active.lock();
        active.entry(seq_no).or_default().push(SnapshotMeta {
            tenant: Some(tenant),
            created_at: at_nanos,
        });
        self.update_min(&active);
    }

    /// Pinnt eine Sequenznummer zu einem expliziten Zeitstempel.
    pub fn pin_at(&self, seq_no: u64, at_nanos: u64) {
        let seq_no = seq_no & !TOMBSTONE_BIT;
        let mut active = self.active.lock();
        active.entry(seq_no).or_default().push(SnapshotMeta {
            tenant: None,
            created_at: at_nanos,
        });
        self.update_min(&active);
    }

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
        self.release_for_tenant_at(None, seq_no, created_at);
    }

    pub(crate) fn release_for_tenant_at(
        &self,
        tenant: Option<TenantId>,
        seq_no: u64,
        created_at: Option<u64>,
    ) {
        let seq_no = seq_no & !TOMBSTONE_BIT;
        let mut active = self.active.lock();
        if let Some(metas) = active.get_mut(&seq_no) {
            if let Some(ts) = created_at {
                if let Some(pos) = metas
                    .iter()
                    .position(|m| m.created_at == ts && (tenant.is_none() || m.tenant == tenant))
                {
                    metas.swap_remove(pos);
                } else if let Some(pos) = metas.iter().position(|m| m.created_at == ts) {
                    metas.swap_remove(pos);
                } else {
                    metas.pop();
                }
            } else if let Some(tenant_id) = tenant {
                if let Some(pos) = metas.iter().position(|m| m.tenant == Some(tenant_id)) {
                    metas.swap_remove(pos);
                } else {
                    metas.pop();
                }
            } else {
                metas.pop();
            }
            if metas.is_empty() {
                active.remove(&seq_no);
            }
        }
        self.update_min(&active);
    }

    /// Findet den am längsten aktiven Snapshot bezüglich des Nanosekunden-Zeitstempels `now_nanos`.
    ///
    /// Liefert `Some((seq_no, duration))` oder `None`, falls aktuell keine Snapshots aktiv sind.
    /// Dient der Diagnose von "hängenden" Lese-Transaktionen, die die Garbage Collection blockieren.
    pub fn longest_active_pin_at(&self, now_nanos: u64) -> Option<(u64, Duration)> {
        let active = self.active.lock();
        let mut longest: Option<(u64, Duration)> = None;

        for (&seq_no, metas) in active.iter() {
            for meta in metas {
                let duration = Duration::from_nanos(now_nanos.saturating_sub(meta.created_at));
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
        for (&seq_no, metas) in active.iter_mut() {
            metas.retain(|meta| {
                if now_nanos.saturating_sub(meta.created_at) > max_nanos {
                    expired_seqs.push(seq_no);
                    false
                } else {
                    true
                }
            });
            if metas.is_empty() {
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
    fn update_min(&self, active: &BTreeMap<u64, Vec<SnapshotMeta>>) {
        let min = active.keys().next().copied().unwrap_or(u64::MAX);
        self.min_active_seqno.store(min, Ordering::Release);
    }
}

/// RAII-Guard (Resource Acquisition Is Initialization) für einen aktiven Lese-Snapshot.
///
/// Hält einen Snapshot in der [`SnapshotRegistry`] gepinnt, solange die Instanz existiert.
/// Beim Verlassen des Gültigkeitsbereichs (`Drop`) wird der Snapshot automatisch deregistriert.
pub struct SnapshotGuard {
    registry: Arc<SnapshotRegistry>,
    seq_no: u64,
    created_at: Option<u64>,
    tenant: Option<TenantId>,
}

impl SnapshotGuard {
    /// Liefert die Sequenznummer, die durch diesen Guard geschützt wird.
    pub fn seq_no(&self) -> u64 {
        self.seq_no
    }

    /// Liefert den zugeordneten Mandanten, falls angegeben.
    pub fn tenant(&self) -> Option<TenantId> {
        self.tenant
    }
}

impl Drop for SnapshotGuard {
    // Automatische Deregistrierung beim Droppen des Guards.
    fn drop(&mut self) {
        self.registry
            .release_for_tenant_at(self.tenant, self.seq_no, self.created_at);
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
    fn test_min_read_snapshot_and_min_for_compaction_and_tenant_revoke() {
        let registry = Arc::new(SnapshotRegistry::new());
        assert_eq!(registry.min_read_snapshot(), None);
        assert_eq!(registry.min_for_compaction(500), 500);

        let tenant_a = TenantId(1);
        let tenant_b = TenantId(2);

        let g_a = registry.register_for_tenant(tenant_a, 100);
        let g_b = registry.register_for_tenant(tenant_b, 100);
        assert_eq!(registry.min_read_snapshot(), Some(100));

        // Revoke only tenant A in range 50..150
        let revoked_a = registry.revoke_covering(tenant_a, 50..150, 100);
        assert_eq!(revoked_a, 1);
        // Tenant B is still active at 100!
        assert_eq!(registry.min_read_snapshot(), Some(100));

        // Revoking again for tenant A does nothing
        let revoked_a_again = registry.revoke_covering(tenant_a, 50..150, 100);
        assert_eq!(revoked_a_again, 0);
        assert_eq!(registry.min_read_snapshot(), Some(100));

        // Revoke tenant B
        let revoked_b = registry.revoke_covering(tenant_b, 50..150, 100);
        assert_eq!(revoked_b, 1);
        assert_eq!(registry.min_read_snapshot(), None);
        assert_eq!(registry.min_for_compaction(600), 600);

        drop(g_a);
        drop(g_b);
    }

    // Beweist strikte Tenant-Isolation: Widerruf für Mandant A berührt ausschließlich Snapshots von Mandant A.
    // Snapshots von Mandant B im selben Sequenzbereich bleiben vollständig aktiv und schützen weiterhin min_active_seqno.
    #[test]
    fn test_tenant_isolation_revocation_guarantee() {
        let registry = Arc::new(SnapshotRegistry::new());
        let tenant_a = TenantId(10);
        let tenant_b = TenantId(20);

        // Registriere zwei Guards auf exakt derselben Sequenznummer (150) für verschiedene Mandanten.
        let guard_a = registry.register_for_tenant(tenant_a, 150);
        let guard_b = registry.register_for_tenant(tenant_b, 150);
        assert_eq!(guard_a.tenant(), Some(tenant_a));
        assert_eq!(guard_b.tenant(), Some(tenant_b));
        assert_eq!(registry.min_read_snapshot(), Some(150));
        assert_eq!(registry.min_active_seqno(), 150);

        // Widerrufe Snapshots von Tenant A im Bereich 100..200
        let revoked_count = registry.revoke_covering(tenant_a, 100..200, 150);
        assert_eq!(
            revoked_count, 1,
            "Exakt 1 Snapshot für Mandant A muss widerrufen werden"
        );

        // Mandant B ist nach wie vor aktiv auf Sequenz 150!
        assert_eq!(
            registry.min_read_snapshot(),
            Some(150),
            "Mandant B muss unverändert aktiv bleiben und min_read_snapshot schützen"
        );
        assert_eq!(registry.min_active_seqno(), 150);

        // Erneuter Widerruf für Mandant A im selben Bereich bewirkt nichts mehr
        let revoked_again = registry.revoke_covering(tenant_a, 100..200, 150);
        assert_eq!(revoked_again, 0);

        // Erst wenn auch Mandant B widerrufen wird, wird die Sequenznummer freigegeben
        let revoked_b = registry.revoke_covering(tenant_b, 100..200, 150);
        assert_eq!(revoked_b, 1);
        assert_eq!(
            registry.min_read_snapshot(),
            None,
            "Nach Widerruf aller Mandanten gibt min_read_snapshot None zurück"
        );
        assert_eq!(
            registry.min_active_seqno(),
            u64::MAX,
            "min_active_seqno signalisiert keine aktiven Leser"
        );

        drop(guard_a);
        drop(guard_b);
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
        fn test_prop_snapshot_registry_min_active(
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
        fn test_prop_snapshot_register_unregister_stress(
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
        fn test_prop_snapshot_pin_unpin_interleaving(
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
