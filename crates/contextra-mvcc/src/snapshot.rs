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

use crate::error::{ContextraError, Result};
use crate::types::{TenantId, TOMBSTONE_BIT};
use contextra_ports::{Clock, SystemClock};
use parking_lot::Mutex;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Konfigurationsoptionen für die [`SnapshotRegistry`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SnapshotRegistryConfig {
    /// Maximale Anzahl gleichzeitig aktiver Snapshots (Default: 100.000).
    pub max_active_snapshots: usize,
    /// Maximales Alter eines Snapshots (Default: 300 s / 5 min), bevor er als überaltert gilt.
    pub max_snapshot_age: Duration,
}

impl Default for SnapshotRegistryConfig {
    fn default() -> Self {
        Self {
            max_active_snapshots: 100_000,
            max_snapshot_age: Duration::from_secs(300),
        }
    }
}

/// Eindeutige, nicht wiederverwendete Kennung einer Registrierung (Guard oder Pin).
///
/// Wird pro [`SnapshotRegistry`] strikt monoton ab `1` vergeben und besitzt außerhalb
/// dieses Crates keinen Konstruktor.
///
/// # Fehler / # Crash
/// Reiner Werttyp ohne fehlbare Operationen.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct RegistrationId(u64);

/// Art einer aktiven Snapshot-Registrierung in der [`SnapshotRegistry`].
///
/// # Fehler / # Crash
/// Reiner Aufzählungstyp ohne fehlbare Operationen.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RegistrationKind {
    /// RAII-geschützter Lese-Snapshot ([`SnapshotGuard`]).
    Guard,
    /// Dauerhaft gepinnte Sequenznummer (z. B. für Checkpoints oder Backups).
    Pin,
}

/// Bericht über einen mandantenbezogenen Snapshot-Entzug durch
/// [`SnapshotRegistry::revoke_covering_report`].
///
/// Enthält ausschließlich Zähler und Sequenznummern, keine Schlüssel.
///
/// # Fehler / # Crash
/// Reiner Werttyp ohne fehlbare Operationen.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct RevokeReport {
    /// Anzahl der widerrufenen Registrierungen (`Guard` und `Pin`) des Mandanten.
    pub revoked: usize,
    /// Anzahl mandantenloser Registrierungen im Bereich, die bewusst erhalten blieben (K-21).
    pub retained_tenantless: usize,
    /// Kleinste Sequenznummer unter den erhaltenen mandantenlosen Registrierungen im Bereich.
    pub retained_min_seq: Option<u64>,
}

/// Beschreibung einer aktiven Registrierung, die ein Gate-Ende oder eine Kompaktierung blockiert.
///
/// Enthält ausschließlich Metadaten (Sequenz, Art, optionaler Mandant, Alter), keine Schlüssel.
///
/// # Fehler / # Crash
/// Reiner Werttyp ohne fehlbare Operationen.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Blocker {
    /// Sequenznummer der blockierenden Registrierung.
    pub seq: u64,
    /// Art der Registrierung (`Guard` oder `Pin`).
    pub kind: RegistrationKind,
    /// Zugeordneter Mandant oder `None` bei mandantenlosen System-Registrierungen.
    pub tenant: Option<TenantId>,
    /// Alter der Registrierung relativ zur Abfragezeit.
    pub age: Duration,
}

/// Altersrichtlinie für [`SnapshotRegistry::reap_expired`].
///
/// Standardwerte: `max_pin_age = 7 Tage`, `max_guard_age = None` (lebende Guards werden
/// standardmäßig nie zwangsweise entfernt).
///
/// # Fehler / # Crash
/// Reiner Konfigurationstyp ohne fehlbare Operationen.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ReapPolicy {
    /// Maximales Alter eines `Pin`, bevor er bereinigt wird.
    pub max_pin_age: Duration,
    /// Optionales maximales Alter eines `Guard` (`None` = Guards nie bereinigen).
    pub max_guard_age: Option<Duration>,
}

impl Default for ReapPolicy {
    fn default() -> Self {
        Self {
            max_pin_age: Duration::from_secs(7 * 24 * 60 * 60),
            max_guard_age: None,
        }
    }
}

/// Berichtseintrag für eine durch [`SnapshotRegistry::reap_expired`] entfernte Registrierung.
///
/// # Fehler / # Crash
/// Reiner Werttyp ohne fehlbare Operationen.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Reaped {
    /// Sequenznummer der bereinigten Registrierung.
    pub seq: u64,
    /// Art der bereinigten Registrierung (`Guard` oder `Pin`).
    pub kind: RegistrationKind,
    /// Zugeordneter Mandant oder `None` bei mandantenlosen Registrierungen.
    pub tenant: Option<TenantId>,
    /// Alter der Registrierung zum Bereinigungszeitpunkt.
    pub age: Duration,
}

/// Interne Metadaten einer aktiven Snapshot-Registrierung.
#[derive(Debug, Clone)]
struct SnapshotMeta {
    id: RegistrationId,
    kind: RegistrationKind,
    tenant: Option<TenantId>,
    created_at: u64,
    revoked: Arc<AtomicBool>,
}

/// Zentrale Registrierung für aktive Lese-Snapshots im System.
///
/// ### Synchronisationsstrategie:
/// - `active`: Ein sortierter `BTreeMap`, geschützt durch einen `parking_lot::Mutex`.
///   Schlüssel ist die Snapshot-Sequenznummer; Wert ist eine Liste von Metadaten
///   aller aktiven Registrierungen (`Guard` und `Pin`), jeweils mit eindeutiger [`RegistrationId`].
/// - `min_active_seqno`: Atomarer 64-Bit-Ganzzahlwert für lock-freies Auslesen durch Hintergrund-Threads.
/// - `next_id`: Deterministischer, strikt monotoner Zähler ab `1` für eindeutige [`RegistrationId`]s.
/// - `clock`: Zeitport zur Erfassung der Haltedauer aktiver Snapshots.
/// - `config`: Konfigurierte Kapazitäts- und Altersgrenzen.
pub struct SnapshotRegistry {
    active: Mutex<BTreeMap<u64, Vec<SnapshotMeta>>>,
    min_active_seqno: AtomicU64,
    next_id: AtomicU64,
    clock: Arc<dyn Clock>,
    config: SnapshotRegistryConfig,
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
    /// Erstellt eine neue, leere Snapshot-Registry mit Standard-[`SystemClock`] und Standardkonfiguration.
    pub fn new() -> Self {
        Self::with_clock(Arc::new(SystemClock::new()))
    }

    /// Erstellt eine neue Snapshot-Registry mit benutzerdefiniertem [`Clock`]-Port (für Tests).
    pub fn with_clock(clock: Arc<dyn Clock>) -> Self {
        Self::with_clock_and_config(clock, SnapshotRegistryConfig::default())
    }

    /// Erstellt eine neue Snapshot-Registry mit benutzerdefiniertem [`Clock`]-Port und Konfiguration.
    pub fn with_clock_and_config(clock: Arc<dyn Clock>, config: SnapshotRegistryConfig) -> Self {
        Self {
            active: Mutex::new(BTreeMap::new()),
            min_active_seqno: AtomicU64::new(u64::MAX),
            next_id: AtomicU64::new(1),
            clock,
            config,
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

    /// Liefert die aktuelle Konfiguration der Registry.
    pub fn config(&self) -> SnapshotRegistryConfig {
        self.config
    }

    /// Setzt eine neue Konfiguration für die Registry.
    pub fn set_config(&mut self, config: SnapshotRegistryConfig) {
        self.config = config;
    }

    /// Liefert die Untergrenze für Garbage Collection (älteste aktive Snapshot-Sequenz).
    /// Entspricht dem Vertrag für Store-Kompaktierung und ist äquivalent zu `min_read_snapshot()`.
    #[inline]
    pub fn oldest_active_snapshot(&self) -> Option<u64> {
        self.min_read_snapshot()
    }

    /// Versucht, einen Lese-Snapshot zu registrieren.
    ///
    /// Im Gegensatz zu `register` schlägt diese Methode fail-closed mit `ContextraError::LimitExceeded` fehl,
    /// wenn die konfigurierte Maximalanzahl aktiver Snapshots (`max_active_snapshots`) erreicht ist.
    pub fn try_register(self: &Arc<Self>, seq_no: u64) -> Result<SnapshotGuard> {
        let now_nanos = self.clock.monotonic_nanos();
        self.try_register_at(seq_no, now_nanos)
    }

    /// Versucht, einen Lese-Snapshot zu einem expliziten Nanosekunden-Zeitstempel zu registrieren.
    pub fn try_register_at(self: &Arc<Self>, seq_no: u64, at_nanos: u64) -> Result<SnapshotGuard> {
        self.try_register_for_tenant_at(None, seq_no, at_nanos)
    }

    /// Versucht, einen Lese-Snapshot für einen bestimmten Mandanten zu registrieren.
    pub fn try_register_for_tenant(
        self: &Arc<Self>,
        tenant: TenantId,
        seq_no: u64,
    ) -> Result<SnapshotGuard> {
        let now_nanos = self.clock.monotonic_nanos();
        self.try_register_for_tenant_at(Some(tenant), seq_no, now_nanos)
    }

    fn insert_meta_locked(
        &self,
        active: &mut BTreeMap<u64, Vec<SnapshotMeta>>,
        kind: RegistrationKind,
        tenant: Option<TenantId>,
        seq_no: u64,
        at_nanos: u64,
    ) -> (RegistrationId, Arc<AtomicBool>) {
        let id = RegistrationId(self.next_id.fetch_add(1, Ordering::Relaxed));
        let revoked = Arc::new(AtomicBool::new(false));
        active.entry(seq_no).or_default().push(SnapshotMeta {
            id,
            kind,
            tenant,
            created_at: at_nanos,
            revoked: Arc::clone(&revoked),
        });
        self.update_min(active);
        (id, revoked)
    }

    fn register_guard_unbounded(
        self: &Arc<Self>,
        tenant: Option<TenantId>,
        seq_no: u64,
        at_nanos: u64,
    ) -> SnapshotGuard {
        let seq_no = seq_no & !TOMBSTONE_BIT;
        let mut active = self.active.lock();
        let (id, revoked) =
            self.insert_meta_locked(&mut active, RegistrationKind::Guard, tenant, seq_no, at_nanos);
        SnapshotGuard {
            registry: self.clone(),
            id,
            seq_no,
            tenant,
            revoked,
        }
    }

    /// Versucht, einen Lese-Snapshot für einen Mandanten zu einem expliziten Zeitstempel zu registrieren.
    pub fn try_register_for_tenant_at(
        self: &Arc<Self>,
        tenant: Option<TenantId>,
        seq_no: u64,
        at_nanos: u64,
    ) -> Result<SnapshotGuard> {
        let seq_no = seq_no & !TOMBSTONE_BIT;
        let mut active = self.active.lock();

        let total_active: usize = active.values().map(|v| v.len()).sum();
        if total_active >= self.config.max_active_snapshots {
            return Err(ContextraError::limit_exceeded(
                self.config.max_active_snapshots,
                format!(
                    "Snapshot registry capacity exceeded ({} active snapshots >= max {})",
                    total_active, self.config.max_active_snapshots
                ),
            ));
        }

        let (id, revoked) =
            self.insert_meta_locked(&mut active, RegistrationKind::Guard, tenant, seq_no, at_nanos);
        Ok(SnapshotGuard {
            registry: self.clone(),
            id,
            seq_no,
            tenant,
            revoked,
        })
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
        self.register_guard_unbounded(Some(tenant), seq_no, at_nanos)
    }

    /// Registriert einen Lese-Snapshot mit explizitem Zeitstempel ohne Mandantenzuordnung.
    pub fn register_at(self: &Arc<Self>, seq_no: u64, at_nanos: u64) -> SnapshotGuard {
        self.register_guard_unbounded(None, seq_no, at_nanos)
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

    /// Widerruft alle Registrierungen (`Guard` und `Pin`) des Mandanten im Bereich
    /// `seq ∈ range ∧ seq <= gated_seq`.
    ///
    /// Mandantenlose Registrierungen (`tenant == None`) bleiben bewusst erhalten (K-21);
    /// der Kernel behandelt sie über das Gate und nutzt [`SnapshotRegistry::revoke_covering_report`].
    ///
    /// # Fehler / # Crash
    /// Fehlt nie und panikt nicht.
    pub fn revoke_covering(
        &self,
        tenant: TenantId,
        range: std::ops::Range<u64>,
        gated_seq: u64,
    ) -> usize {
        self.revoke_covering_report(tenant, range, gated_seq).revoked
    }

    /// Widerruft alle Registrierungen (`Guard` und `Pin`) des Mandanten im Bereich
    /// `seq ∈ range ∧ seq <= gated_seq` und liefert einen [`RevokeReport`].
    ///
    /// Setzt für jede entfernte Registrierung das atomare `revoked`-Flag (`Release`),
    /// sodass lebende [`SnapshotGuard`]s über [`SnapshotGuard::is_revoked`] abbrechen können.
    ///
    /// Mandantenlose Registrierungen (`tenant == None`) bleiben bewusst erhalten (K-21);
    /// ihre Anzahl und kleinste Sequenznummer im Bereich werden im Bericht ausgewiesen,
    /// damit der Kernel sie über das Gate bzw. `OverlappingSnapshotPinned` behandeln kann.
    ///
    /// # Fehler / # Crash
    /// Fehlt nie und panikt nicht.
    pub fn revoke_covering_report(
        &self,
        tenant: TenantId,
        range: std::ops::Range<u64>,
        gated_seq: u64,
    ) -> RevokeReport {
        if range.is_empty() || gated_seq < range.start {
            return RevokeReport {
                revoked: 0,
                retained_tenantless: 0,
                retained_min_seq: None,
            };
        }

        let upper = (range.end - 1).min(gated_seq);
        let mut active = self.active.lock();
        let mut revoked = 0usize;
        let mut retained_tenantless = 0usize;
        let mut retained_min_seq: Option<u64> = None;
        let mut empty_keys = Vec::new();

        for (&seq, metas) in active.range_mut(range.start..=upper) {
            metas.retain(|meta| {
                if meta.tenant == Some(tenant) {
                    meta.revoked.store(true, Ordering::Release);
                    revoked += 1;
                    false
                } else {
                    if meta.tenant.is_none() {
                        retained_tenantless += 1;
                        retained_min_seq = Some(retained_min_seq.map_or(seq, |m| m.min(seq)));
                    }
                    true
                }
            });
            if metas.is_empty() {
                empty_keys.push(seq);
            }
        }

        for seq in empty_keys {
            active.remove(&seq);
        }

        if revoked > 0 {
            self.update_min(&active);
        }

        RevokeReport {
            revoked,
            retained_tenantless,
            retained_min_seq,
        }
    }

    /// Liefert alle aktiven Registrierungen mit `seq <= up_to`, aufsteigend nach Sequenz.
    /// Höchstens `limit` Einträge.
    ///
    /// Dient der Gate-Ende-Prüfung (I-9) und Diagnose ohne Schlüsselmaterial.
    /// Der `active`-Mutex wird nur für die Dauer des Kopierens gehalten.
    ///
    /// # Fehler / # Crash
    /// Fehlt nie und panikt nicht.
    pub fn blockers_up_to(&self, up_to: u64, limit: usize) -> Vec<Blocker> {
        if limit == 0 {
            return Vec::new();
        }
        let now_nanos = self.clock.monotonic_nanos();
        let active = self.active.lock();
        let mut blockers = Vec::with_capacity(limit.min(active.len()));

        'outer: for (&seq, metas) in active.range(..=up_to) {
            for meta in metas {
                if blockers.len() >= limit {
                    break 'outer;
                }
                blockers.push(Blocker {
                    seq,
                    kind: meta.kind,
                    tenant: meta.tenant,
                    age: Duration::from_nanos(now_nanos.saturating_sub(meta.created_at)),
                });
            }
        }

        blockers
    }

    /// Dauerhaftes Pinnen einer Sequenznummer (z. B. für SAOS Checkpoints oder Backups),
    /// ohne einen RAII-Guard zu halten.
    ///
    /// Muss manuell über [`SnapshotRegistry::unpin`] wieder freigegeben werden.
    pub fn pin(&self, seq_no: u64) {
        let now_nanos = self.clock.monotonic_nanos();
        self.pin_at(seq_no, now_nanos);
    }

    /// Pinnt eine Sequenznummer für einen Mandanten bezogen auf die aktuelle Systemuhr.
    pub fn pin_for_tenant(&self, tenant: TenantId, seq_no: u64) {
        let now_nanos = self.clock.monotonic_nanos();
        self.pin_for_tenant_at(tenant, seq_no, now_nanos);
    }

    /// Pinnt eine Sequenznummer für einen Mandanten zu einem expliziten Zeitstempel.
    pub fn pin_for_tenant_at(&self, tenant: TenantId, seq_no: u64, at_nanos: u64) {
        let seq_no = seq_no & !TOMBSTONE_BIT;
        let mut active = self.active.lock();
        self.insert_meta_locked(&mut active, RegistrationKind::Pin, Some(tenant), seq_no, at_nanos);
    }

    /// Pinnt eine Sequenznummer zu einem expliziten Zeitstempel.
    pub fn pin_at(&self, seq_no: u64, at_nanos: u64) {
        let seq_no = seq_no & !TOMBSTONE_BIT;
        let mut active = self.active.lock();
        self.insert_meta_locked(&mut active, RegistrationKind::Pin, None, seq_no, at_nanos);
    }

    /// Hebt die älteste [`RegistrationKind::Pin`]-Registrierung für `seq_no` auf
    /// und aktualisiert den minimal aktiven Wasserstand.
    ///
    /// Berührt niemals [`RegistrationKind::Guard`]-Registrierungen. Existiert kein
    /// Pin für `seq_no`, ist der Aufruf ein No-op.
    pub fn unpin(&self, seq_no: u64) {
        let seq_no = seq_no & !TOMBSTONE_BIT;
        let mut active = self.active.lock();
        if let Some(metas) = active.get_mut(&seq_no) {
            if let Some((pos, _)) = metas
                .iter()
                .enumerate()
                .filter(|(_, m)| m.kind == RegistrationKind::Pin)
                .min_by_key(|(_, m)| (m.created_at, m.id))
            {
                metas.swap_remove(pos);
                if metas.is_empty() {
                    active.remove(&seq_no);
                }
                self.update_min(&active);
            }
        }
    }

    // Interne Freigabe ausschließlich über die eindeutige RegistrationId.
    // Wird die ID nicht gefunden (bereits entzogen oder gereapt), ist dies ein No-op.
    pub(crate) fn release_by_id(&self, seq_no: u64, id: RegistrationId) {
        let seq_no = seq_no & !TOMBSTONE_BIT;
        let mut active = self.active.lock();
        if let Some(metas) = active.get_mut(&seq_no) {
            if let Some(pos) = metas.iter().position(|m| m.id == id) {
                metas.swap_remove(pos);
                if metas.is_empty() {
                    active.remove(&seq_no);
                }
                self.update_min(&active);
            }
        }
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

    /// Bereinigt überalterte Registrierungen gemäß [`ReapPolicy`] bezogen auf `now_nanos`.
    ///
    /// - `Pin`-Registrierungen älter als `policy.max_pin_age` werden entfernt.
    /// - `Guard`-Registrierungen werden nur entfernt, wenn `policy.max_guard_age = Some(..)`
    ///   gesetzt ist und ihr Alter diesen Schwellenwert überschreitet.
    /// - Für jede entfernte Registrierung wird `revoked = true` (`Release`) gesetzt.
    /// - Aktualisiert `min_active_seqno` und liefert die entfernten Einträge sortiert nach
    ///   `(seq, age)`.
    ///
    /// # Fehler / # Crash
    /// Fehlt nie und panikt nicht.
    pub fn reap_expired(&self, policy: ReapPolicy, now_nanos: u64) -> Vec<Reaped> {
        let max_pin_nanos = u64::try_from(policy.max_pin_age.as_nanos()).unwrap_or(u64::MAX);
        let max_guard_nanos = policy
            .max_guard_age
            .map(|d| u64::try_from(d.as_nanos()).unwrap_or(u64::MAX));

        let mut reaped = Vec::new();
        let mut active = self.active.lock();
        let mut seqs_to_remove = Vec::new();

        for (&seq, metas) in active.iter_mut() {
            metas.retain(|meta| {
                let age_nanos = now_nanos.saturating_sub(meta.created_at);
                let expired = match meta.kind {
                    RegistrationKind::Pin => age_nanos > max_pin_nanos,
                    RegistrationKind::Guard => max_guard_nanos.is_some_and(|max_g| age_nanos > max_g),
                };
                if expired {
                    meta.revoked.store(true, Ordering::Release);
                    reaped.push(Reaped {
                        seq,
                        kind: meta.kind,
                        tenant: meta.tenant,
                        age: Duration::from_nanos(age_nanos),
                    });
                    false
                } else {
                    true
                }
            });
            if metas.is_empty() {
                seqs_to_remove.push(seq);
            }
        }

        for seq in seqs_to_remove {
            active.remove(&seq);
        }

        if !reaped.is_empty() {
            self.update_min(&active);
            reaped.sort_by_key(|r| (r.seq, r.age));
        }

        reaped
    }

    /// Bereinigt und entfernt alle aktiven Snapshot-Pins und Guards, die älter als `max_pin_duration` sind
    /// (ohne Bericht, nur Tests; neue Aufrufer nutzen [`SnapshotRegistry::reap_expired`]).
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
                    meta.revoked.store(true, Ordering::Release);
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

    /// Bereinigt alle überalterten Snapshot-Pins und Guards bezogen auf die aktuelle Uhrzeit
    /// (ohne Bericht, nur Tests; neue Aufrufer nutzen [`SnapshotRegistry::reap_expired`]).
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
    id: RegistrationId,
    seq_no: u64,
    tenant: Option<TenantId>,
    revoked: Arc<AtomicBool>,
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

    /// Prüft, ob dieser Snapshot durch [`SnapshotRegistry::revoke_covering`] oder
    /// [`SnapshotRegistry::reap_expired`] widerrufen wurde (`Acquire`-Load).
    ///
    /// # Fehler / # Crash
    /// Fehlt nie und panikt nicht.
    pub fn is_revoked(&self) -> bool {
        self.revoked.load(Ordering::Acquire)
    }
}

impl Drop for SnapshotGuard {
    // Automatische Deregistrierung beim Droppen des Guards über die eindeutige ID.
    fn drop(&mut self) {
        self.registry.release_by_id(self.seq_no, self.id);
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
        registry.release_by_id(12345, RegistrationId(999_999));
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

    // Prüft try_register und Kapazitätsbegrenzung für SnapshotRegistry.
    #[test]
    fn test_snapshot_registry_try_register_capacity_limit() {
        let clock = Arc::new(ManualClock::new(1_000_000_000));
        let config = SnapshotRegistryConfig {
            max_active_snapshots: 2,
            max_snapshot_age: Duration::from_secs(60),
        };
        let registry = Arc::new(SnapshotRegistry::with_clock_and_config(clock, config));

        assert_eq!(registry.oldest_active_snapshot(), None);

        let g1 = registry.try_register(10).expect("first snapshot succeeds");
        assert_eq!(registry.oldest_active_snapshot(), Some(10));

        let g2 = registry.try_register(20).expect("second snapshot succeeds");
        assert_eq!(registry.oldest_active_snapshot(), Some(10));

        // Third snapshot must fail closed with LimitExceeded
        let res3 = registry.try_register(30);
        assert!(matches!(res3, Err(ContextraError::LimitExceeded { .. })));

        drop(g1);
        assert_eq!(registry.oldest_active_snapshot(), Some(20));

        // Now third snapshot succeeds
        let g3 = registry
            .try_register(30)
            .expect("third snapshot succeeds after drop");
        assert_eq!(registry.oldest_active_snapshot(), Some(20));

        drop(g2);
        drop(g3);
        assert_eq!(registry.oldest_active_snapshot(), None);
    }

    // TST-MVCC-01: Regression B-M1 — Mandant A und B am gleichen seq und gleichem created_at (ManualClock).
    // revoke_covering(A, ...) und anschließendes drop(guard_A) darf Mandant B nicht deregistrieren.
    #[test]
    fn test_tst_mvcc_01_b_m1_revoke_then_drop_preserves_other_tenant() {
        let clock = Arc::new(ManualClock::new(1_000_000_000));
        let registry = Arc::new(SnapshotRegistry::with_clock(clock));
        let tenant_a = TenantId(1);
        let tenant_b = TenantId(2);

        let guard_a = registry.register_for_tenant(tenant_a, 100);
        let guard_b = registry.register_for_tenant(tenant_b, 100);
        assert_eq!(registry.min_active_seqno(), 100);

        let revoked = registry.revoke_covering(tenant_a, 50..150, 100);
        assert_eq!(revoked, 1);
        assert_eq!(registry.min_active_seqno(), 100);

        // Drop des entzogenen Guards darf Mandant B nicht entfernen!
        drop(guard_a);
        assert_eq!(
            registry.min_active_seqno(),
            100,
            "Mandant B muss nach Drop des entzogenen Guard A registriert bleiben"
        );

        drop(guard_b);
        assert_eq!(registry.min_active_seqno(), u64::MAX);
    }

    // TST-MVCC-03: unpin entfernt nur Pin-Metas (älteste zuerst), nie Guards; ohne Pin No-op.
    #[test]
    fn test_tst_mvcc_03_unpin_only_removes_pins_oldest_first() {
        let clock = Arc::new(ManualClock::new(1_000_000_000));
        let registry = Arc::new(SnapshotRegistry::with_clock(clock.clone()));

        let guard = registry.register(100);
        // Ohne Pin ist unpin ein No-op und rührt den Guard nicht an.
        registry.unpin(100);
        assert_eq!(registry.min_active_seqno(), 100);
        assert!(!guard.is_revoked());

        // Zwei Pins auf derselben Sequenz zu verschiedenen Zeitpunkten
        registry.pin(100); // created_at = 1s
        clock.advance(Duration::from_secs(10));
        registry.pin(100); // created_at = 11s

        // Erstes unpin entfernt den ältesten Pin (1s); Guard + jüngerer Pin (11s) bleiben.
        clock.advance(Duration::from_secs(5)); // now = 16s
        registry.unpin(100);
        let blockers = registry.blockers_up_to(100, 10);
        assert_eq!(blockers.len(), 2);
        let pin_ages: Vec<Duration> = blockers
            .iter()
            .filter(|b| b.kind == RegistrationKind::Pin)
            .map(|b| b.age)
            .collect();
        assert_eq!(pin_ages, vec![Duration::from_secs(5)]);

        // Zweites unpin entfernt den verbliebenen Pin; Guard bleibt.
        registry.unpin(100);
        let blockers = registry.blockers_up_to(100, 10);
        assert_eq!(blockers.len(), 1);
        assert_eq!(
            blockers.first().map(|b| b.kind),
            Some(RegistrationKind::Guard)
        );

        // Drittes unpin ohne Pin ist No-op; Guard bleibt aktiv.
        registry.unpin(100);
        assert_eq!(registry.min_active_seqno(), 100);
        assert!(!guard.is_revoked());

        drop(guard);
        assert_eq!(registry.min_active_seqno(), u64::MAX);
    }

    // TST-MVCC-04: revoke_covering entfernt Pin und Guard des Mandanten, lässt mandantenlose Metas stehen,
    // und RevokeReport zählt sie präzise.
    #[test]
    fn test_tst_mvcc_04_revoke_covering_report_pins_guards_and_tenantless() {
        let registry = Arc::new(SnapshotRegistry::new());
        let tenant_a = TenantId(1);
        let tenant_b = TenantId(2);

        let g_a = registry.register_for_tenant(tenant_a, 100);
        registry.pin_for_tenant(tenant_a, 120);
        let g_b = registry.register_for_tenant(tenant_b, 110);
        let g_sys_1 = registry.register(130);
        registry.pin(90);
        let g_a_outside = registry.register_for_tenant(tenant_a, 200);

        let report = registry.revoke_covering_report(tenant_a, 80..150, 140);
        assert_eq!(
            report,
            RevokeReport {
                revoked: 2,
                retained_tenantless: 2,
                retained_min_seq: Some(90),
            }
        );
        assert!(g_a.is_revoked());
        assert!(!g_b.is_revoked());
        assert!(!g_sys_1.is_revoked());
        assert!(!g_a_outside.is_revoked());
        assert_eq!(registry.min_active_seqno(), 90);

        // Leerer Bereich oder gated_seq < range.start
        let empty_report = registry.revoke_covering_report(tenant_a, 100..100, 200);
        assert_eq!(
            empty_report,
            RevokeReport {
                revoked: 0,
                retained_tenantless: 0,
                retained_min_seq: None,
            }
        );

        drop((g_a, g_b, g_sys_1, g_a_outside));
        registry.unpin(90);
        assert_eq!(registry.min_active_seqno(), u64::MAX);
    }

    // TST-MVCC-05: SnapshotGuard::is_revoked wird nach Entzug und nach reap_expired wahr, davor falsch.
    #[test]
    fn test_tst_mvcc_05_guard_is_revoked_on_revoke_and_reap() {
        let clock = Arc::new(ManualClock::new(1_000_000_000));
        let registry = Arc::new(SnapshotRegistry::with_clock(clock.clone()));
        let tenant_a = TenantId(1);

        let g_revoke = registry.register_for_tenant(tenant_a, 100);
        let g_reap = registry.register(200);
        assert!(!g_revoke.is_revoked());
        assert!(!g_reap.is_revoked());

        let revoked = registry.revoke_covering(tenant_a, 50..150, 100);
        assert_eq!(revoked, 1);
        assert!(g_revoke.is_revoked());
        assert!(!g_reap.is_revoked());

        clock.advance(Duration::from_secs(60));
        let policy = ReapPolicy {
            max_pin_age: Duration::from_secs(10),
            max_guard_age: Some(Duration::from_secs(30)),
        };
        let reaped = registry.reap_expired(policy, clock.monotonic_nanos());
        assert_eq!(reaped.len(), 1);
        let first = reaped.first().expect("one reaped guard");
        assert_eq!(first.seq, 200);
        assert_eq!(first.kind, RegistrationKind::Guard);
        assert!(g_reap.is_revoked());
    }

    // TST-MVCC-06: reap_expired mit Default (nur Pins, Guards bleiben), max_guard_age = Some,
    // Sortierung nach (seq, age) und ohne Duplikate.
    #[test]
    fn test_tst_mvcc_06_reap_expired_policy_and_sorting() {
        let clock = Arc::new(ManualClock::new(1_000_000_000));
        let registry = Arc::new(SnapshotRegistry::with_clock(clock.clone()));

        let g_old = registry.register(50);
        registry.pin(300);
        clock.advance(Duration::from_secs(100));
        registry.pin(100);
        clock.advance(Duration::from_secs(50));
        registry.pin(100);

        // Alter bei t = 1s + 8 Tage:
        // g_old: 8d, pin(300): 8d, pin(100)#1: 8d - 100s, pin(100)#2: 8d - 150s
        clock.advance(Duration::from_secs(8 * 24 * 60 * 60 - 150));
        let now = clock.monotonic_nanos();

        // Default-Policy (7 Tage für Pins, None für Guards): Guards bleiben!
        let reaped = registry.reap_expired(ReapPolicy::default(), now);
        assert_eq!(reaped.len(), 3);
        // Sortiert nach seq, dann age: (100, 8d-150s), (100, 8d-100s), (300, 8d)
        let r0 = reaped.first().expect("r0");
        let r1 = reaped.get(1).expect("r1");
        let r2 = reaped.get(2).expect("r2");
        assert_eq!(r0.seq, 100);
        assert_eq!(r1.seq, 100);
        assert!(r0.age < r1.age);
        assert_eq!(r2.seq, 300);
        assert!(reaped.iter().all(|r| r.kind == RegistrationKind::Pin));
        assert!(!g_old.is_revoked());
        assert_eq!(registry.min_active_seqno(), 50);

        // Mit max_guard_age = Some wird auch der alte Guard entfernt.
        let reaped_guards = registry.reap_expired(
            ReapPolicy {
                max_pin_age: Duration::from_secs(1),
                max_guard_age: Some(Duration::from_secs(7 * 24 * 60 * 60)),
            },
            now,
        );
        assert_eq!(reaped_guards.len(), 1);
        let rg0 = reaped_guards.first().expect("rg0");
        assert_eq!(rg0.seq, 50);
        assert_eq!(rg0.kind, RegistrationKind::Guard);
        assert!(g_old.is_revoked());
        assert_eq!(registry.min_active_seqno(), u64::MAX);
    }

    // TST-MVCC-07: blockers_up_to respektiert limit, aufsteigende Reihenfolge, keine Schlüssel.
    #[test]
    fn test_tst_mvcc_07_blockers_up_to_limit_and_order() {
        let clock = Arc::new(ManualClock::new(1_000_000_000));
        let registry = Arc::new(SnapshotRegistry::with_clock(clock.clone()));
        let tenant = TenantId(7);

        let _g300 = registry.register(300);
        let _g100 = registry.register_for_tenant(tenant, 100);
        clock.advance(Duration::from_secs(5));
        registry.pin(200);
        let _g400 = registry.register(400);

        assert!(registry.blockers_up_to(300, 0).is_empty());

        let limited = registry.blockers_up_to(300, 2);
        assert_eq!(limited.len(), 2);
        assert_eq!(
            limited.first().copied(),
            Some(Blocker {
                seq: 100,
                kind: RegistrationKind::Guard,
                tenant: Some(tenant),
                age: Duration::from_secs(5),
            })
        );
        assert_eq!(
            limited.get(1).copied(),
            Some(Blocker {
                seq: 200,
                kind: RegistrationKind::Pin,
                tenant: None,
                age: Duration::from_secs(0),
            })
        );

        let all_up_to_300 = registry.blockers_up_to(300, 10);
        assert_eq!(
            all_up_to_300.iter().map(|b| b.seq).collect::<Vec<_>>(),
            vec![100, 200, 300]
        );
    }

    // TST-MVCC-08: RegistrationId strikt monoton, nach 10^6 Registrierungen eindeutig (mehrere Threads).
    #[test]
    fn test_tst_mvcc_08_registration_id_monotonic_and_unique_1m() {
        let registry = Arc::new(SnapshotRegistry::new());
        let num_threads = 4usize;
        let per_thread = 250_000usize;
        let mut handles = Vec::with_capacity(num_threads);

        for t in 0..num_threads {
            let reg = Arc::clone(&registry);
            handles.push(std::thread::spawn(move || {
                let mut ids = Vec::with_capacity(per_thread);
                let mut prev = RegistrationId(0);
                for i in 0..per_thread {
                    let guard = reg.register(u64::try_from(t * 10 + (i % 10)).unwrap());
                    assert!(guard.id > prev, "ID muss pro Thread strikt monoton steigen");
                    prev = guard.id;
                    ids.push(guard.id);
                    // Guard wird sofort gedroppt, sodass BTreeMap klein bleibt
                }
                ids
            }));
        }

        let mut all_ids = Vec::with_capacity(num_threads * per_thread);
        for h in handles {
            all_ids.extend(h.join().unwrap());
        }
        assert_eq!(all_ids.len(), 1_000_000);
        all_ids.sort_unstable();
        assert!(
            all_ids.windows(2).all(|w| matches!(w, [a, b] if a < b)),
            "Alle 10^6 RegistrationIds müssen paarweise eindeutig sein"
        );
        assert_eq!(registry.min_active_seqno(), u64::MAX);
    }

    // TST-MVCC-09: Nebenläufigkeit von register/drop/revoke/reap/blockers parallel.
    #[test]
    fn test_tst_mvcc_09_concurrent_register_drop_revoke_reap() {
        let clock = Arc::new(ManualClock::new(1_000_000_000));
        let registry = Arc::new(SnapshotRegistry::with_clock(clock.clone()));
        let mut handles = Vec::new();

        for t in 0..4u64 {
            let reg = Arc::clone(&registry);
            let clk = Arc::clone(&clock);
            handles.push(std::thread::spawn(move || {
                let tenant = TenantId(t + 1);
                for i in 0..500u64 {
                    let seq = (i % 50) + 1;
                    let g = reg.register_for_tenant(tenant, seq);
                    if i % 3 == 0 {
                        reg.pin_for_tenant(tenant, seq);
                    }
                    if i % 5 == 0 {
                        let _ = reg.revoke_covering_report(tenant, 1..30, 25);
                    }
                    if i % 7 == 0 {
                        let _ = reg.blockers_up_to(50, 8);
                    }
                    if i % 11 == 0 {
                        clk.advance(Duration::from_secs(5));
                        let _ = reg.reap_expired(
                            ReapPolicy {
                                max_pin_age: Duration::from_secs(10),
                                max_guard_age: None,
                            },
                            clk.monotonic_nanos(),
                        );
                    }
                    drop(g);
                }
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        // Alle verbliebenen Pins mit Alter 0 bereinigen
        clock.advance(Duration::from_secs(100));
        let _ = registry.reap_expired(
            ReapPolicy {
                max_pin_age: Duration::from_secs(1),
                max_guard_age: None,
            },
            clock.monotonic_nanos(),
        );
        assert_eq!(registry.min_active_seqno(), u64::MAX);
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

        // TST-MVCC-02: Doppelfreigabe / beliebige Folgen aus register/pin/revoke/reap/unpin/drop
        // gegen unabhängiges Referenzmodell BTreeMap<RegistrationId, RefMeta>.
        #[test]
        fn test_prop_tst_mvcc_02_double_release_and_state_machine(
            ops in proptest::collection::vec(
                (0u8..6, 0u8..4, 1u64..40, 0usize..20, 1u64..15),
                1..80
            )
        ) {
            #[derive(Clone, Copy)]
            struct RefMeta {
                kind: RegistrationKind,
                tenant: Option<TenantId>,
                seq: u64,
                created_at: u64,
            }

            let clock = Arc::new(ManualClock::new(1_000_000_000));
            let registry = Arc::new(SnapshotRegistry::with_clock(clock.clone()));
            let mut ref_map: BTreeMap<RegistrationId, RefMeta> = BTreeMap::new();
            let mut guards: Vec<(RegistrationId, SnapshotGuard)> = Vec::new();

            for (op, tenant_pick, seq, idx_seed, dt_secs) in ops {
                clock.advance(Duration::from_secs(dt_secs));
                let now = clock.monotonic_nanos();
                let tenant = if tenant_pick == 0 {
                    None
                } else {
                    Some(TenantId(u64::from(tenant_pick)))
                };

                match op {
                    0 => {
                        let g = match tenant {
                            Some(t) => registry.register_for_tenant(t, seq),
                            None => registry.register(seq),
                        };
                        let id = g.id;
                        ref_map.insert(
                            id,
                            RefMeta {
                                kind: RegistrationKind::Guard,
                                tenant,
                                seq,
                                created_at: now,
                            },
                        );
                        guards.push((id, g));
                    }
                    1 => {
                        let id = match tenant {
                            Some(t) => {
                                registry.pin_for_tenant(t, seq);
                                RegistrationId(registry.next_id.load(Ordering::Relaxed) - 1)
                            }
                            None => {
                                registry.pin(seq);
                                RegistrationId(registry.next_id.load(Ordering::Relaxed) - 1)
                            }
                        };
                        ref_map.insert(
                            id,
                            RefMeta {
                                kind: RegistrationKind::Pin,
                                tenant,
                                seq,
                                created_at: now,
                            },
                        );
                    }
                    2 => {
                        if !guards.is_empty() {
                            let idx = idx_seed % guards.len();
                            let (id, g) = guards.swap_remove(idx);
                            drop(g);
                            ref_map.remove(&id);
                        }
                    }
                    3 => {
                        let t = TenantId(u64::from((tenant_pick % 3) + 1));
                        let start = seq;
                        let end = seq + dt_secs;
                        let gated = seq + (dt_secs / 2) + 1;
                        let report = registry.revoke_covering_report(t, start..end, gated);

                        let mut expected_revoked = 0usize;
                        let mut expected_retained = 0usize;
                        let mut expected_min: Option<u64> = None;
                        let mut to_remove = Vec::new();
                        for (&id, meta) in &ref_map {
                            if meta.seq >= start && meta.seq < end && meta.seq <= gated {
                                if meta.tenant == Some(t) {
                                    expected_revoked += 1;
                                    to_remove.push(id);
                                } else if meta.tenant.is_none() {
                                    expected_retained += 1;
                                    expected_min = Some(expected_min.map_or(meta.seq, |m| m.min(meta.seq)));
                                }
                            }
                        }
                        for id in to_remove {
                            ref_map.remove(&id);
                        }
                        prop_assert_eq!(
                            report,
                            RevokeReport {
                                revoked: expected_revoked,
                                retained_tenantless: expected_retained,
                                retained_min_seq: expected_min,
                            }
                        );
                    }
                    4 => {
                        let max_pin_age = Duration::from_secs(20);
                        let max_guard_age = if idx_seed % 2 == 0 {
                            Some(Duration::from_secs(30))
                        } else {
                            None
                        };
                        let reaped = registry.reap_expired(
                            ReapPolicy {
                                max_pin_age,
                                max_guard_age,
                            },
                            now,
                        );
                        let mut expected = Vec::new();
                        let mut to_remove = Vec::new();
                        for (&id, meta) in &ref_map {
                            let age = Duration::from_nanos(now.saturating_sub(meta.created_at));
                            let expired = match meta.kind {
                                RegistrationKind::Pin => age > max_pin_age,
                                RegistrationKind::Guard => max_guard_age.is_some_and(|mg| age > mg),
                            };
                            if expired {
                                to_remove.push(id);
                                expected.push(Reaped {
                                    seq: meta.seq,
                                    kind: meta.kind,
                                    tenant: meta.tenant,
                                    age,
                                });
                            }
                        }
                        for id in to_remove {
                            ref_map.remove(&id);
                        }
                        expected.sort_by_key(|r| (r.seq, r.age));
                        prop_assert_eq!(reaped, expected);
                    }
                    _ => {
                        registry.unpin(seq);
                        if let Some((&oldest_id, _)) = ref_map
                            .iter()
                            .filter(|(_, m)| m.seq == seq && m.kind == RegistrationKind::Pin)
                            .min_by_key(|(&id, m)| (m.created_at, id))
                        {
                            ref_map.remove(&oldest_id);
                        }
                    }
                }

                // Invarianten nach jedem Schritt prüfen
                let expected_min = ref_map.values().map(|m| m.seq).min().unwrap_or(u64::MAX);
                prop_assert_eq!(registry.min_active_seqno(), expected_min);

                for (id, g) in &guards {
                    prop_assert_eq!(g.is_revoked(), !ref_map.contains_key(id));
                }

                let blockers = registry.blockers_up_to(u64::MAX, usize::MAX);
                prop_assert_eq!(blockers.len(), ref_map.len());
            }
        }
    }
}
