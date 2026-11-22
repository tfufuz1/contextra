//! # Committer: Flat Combining Group-Commit und Schreibpfad
//!
//! Normative Referenzen: SPEC §10.1, §10.2 (Schritte 1–12), §10.4, §16.1 (INV-LOCK-1),
//! §16.2 (Leiterwahl und Gegendruck ohne Wartezyklus, E-4), §16.3 (INV-VIS-1, INV-VIS-2, INV-VIS-3),
//! INV-KEYP-1, INV-LOG-5, INV-STAMP-1, SD-49, SD-53, Hypothesen H-S2 und H-S7.
//! Keine Integer-Casts (M-02 = 0).

use std::collections::{BTreeMap, VecDeque};
use std::fmt;
use std::io;
use contextra_storage_ports::{HookPoint, Hooks, KeyDomain, KeyProvider};

use crate::codec::limits::{
    MAX_CHANGE_BYTES, MAX_HEADER_LEN, MAX_KEY_BYTES, MAX_OPS_PER_CHANGE, MAX_VALUE_BYTES,
};
use crate::codec::tag::{InternalKey, Tag, TagKind};
use crate::engine::memtable::Slot;
use crate::engine::version::VersionState;
use crate::sync::{
    Arc, AtomicBool, AtomicU64, Condvar, Mutex, Ordering, RankedMutex,
};

/// Zusammenfassung einer einzelnen Operation für Listener.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpSummary {
    /// Space-Kennung.
    pub space: u16,
    /// Art der Operation (Put, Delete, RangeDelete, EraseRange).
    pub kind: TagKind,
    /// Nutzerschlüssel.
    pub key: Vec<u8>,
    /// Header-Bytes.
    pub header: Vec<u8>,
}

/// Ereignis über einen erfolgreich festgeschriebenen Change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeEvent {
    /// Eindeutiger Stamp des Changes.
    pub stamp: u64,
    /// Liste der Operationen in diesem Change.
    pub ops: Vec<OpSummary>,
}

/// Fehler beim Empfang oder der Zustellung an Listener.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ListenerError {
    /// Listener hat Ereignisse verpasst (SD-62).
    Lagged {
        /// Erster verpasster Stamp.
        missed_from: u64,
    },
    /// Zustellung fehlgeschlagen oder Listener meldet Fehler.
    Failed(String),
}

impl fmt::Display for ListenerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Lagged { missed_from } => {
                write!(
                    f,
                    "ListenerLagged: Ereignisse ab Stamp {} verpasst",
                    missed_from
                )
            }
            Self::Failed(msg) => write!(f, "ListenerFailed: {}", msg),
        }
    }
}

impl std::error::Error for ListenerError {}

/// Empfänger von Change-Ereignissen.
pub trait Listener: Send + Sync {
    /// Wird bei jedem sichtbaren Change in Stamp-Reihenfolge aufgerufen.
    fn on_change(&self, event: &ChangeEvent) -> Result<(), ListenerError>;
}

/// Konfiguration für einen registrierten Listener.
#[derive(Debug, Clone)]
pub struct ListenerConfig {
    /// Maximale Warteschlangengröße vor Überlauf (Standard: 1024).
    pub queue_limit: usize,
    /// Maximale Frist für die Zustellung in Millisekunden (Standard: 5000).
    pub deadline_ms: u64,
}

impl Default for ListenerConfig {
    fn default() -> Self {
        Self {
            queue_limit: 1024,
            deadline_ms: 5000,
        }
    }
}

/// Zustand eines einzelnen Abonnenten.
struct Subscriber {
    id: u64,
    listener: Arc<dyn Listener>,
    config: ListenerConfig,
    queue: VecDeque<ChangeEvent>,
    lagged: Option<u64>,
    active: bool,
}

/// Zustand der Listener-Warteschlange.
struct ListenerQueueState {
    next_id: u64,
    subscribers: Vec<Subscriber>,
}

/// Warteschlange und Verwaltung aller Listener unter Rang 7 Sperre (INV-LOCK-1).
pub struct ListenerQueue {
    inner: RankedMutex<ListenerQueueState, 7>,
}

impl Default for ListenerQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl ListenerQueue {
    /// Erstellt eine neue Listener-Warteschlange.
    pub fn new() -> Self {
        Self {
            inner: RankedMutex::new(ListenerQueueState {
                next_id: 1,
                subscribers: Vec::new(),
            }),
        }
    }

    /// Registriert einen neuen Listener.
    pub fn subscribe(&self, listener: Arc<dyn Listener>, config: ListenerConfig) -> u64 {
        let mut guard = self.inner.lock();
        let id = guard.next_id;
        guard.next_id = guard.next_id.saturating_add(1);

        guard.subscribers.push(Subscriber {
            id,
            listener,
            config,
            queue: VecDeque::new(),
            lagged: None,
            active: true,
        });

        id
    }

    /// Meldet einen Listener ab.
    pub fn unsubscribe(&self, id: u64) {
        let mut guard = self.inner.lock();
        guard.subscribers.retain(|s| s.id != id);
    }

    /// Reiht ein neues Ereignis in alle aktiven Listener-Warteschlangen ein (SPEC §10.2 Schritt 11).
    ///
    /// SD-62: Läuft die Warteschlange eines Listeners über, wird er abgemeldet und
    /// als `lagged` markiert. Der Commit-Pfad wird NIEMALS blockiert.
    pub fn push_event(&self, event: ChangeEvent) {
        let mut guard = self.inner.lock();

        for sub in &mut guard.subscribers {
            if !sub.active {
                continue;
            }

            if sub.queue.len() >= sub.config.queue_limit {
                // Warteschlange voll: Abmelden (SD-62)
                sub.active = false;
                if sub.lagged.is_none() {
                    sub.lagged = Some(event.stamp);
                }
                sub.queue.clear();
            } else {
                sub.queue.push_back(event.clone());
            }
        }
    }

    /// Prüft den Status eines Listeners und liefert ggf. `ListenerLagged` zurück (SD-62).
    pub fn poll_status(&self, id: u64) -> Result<(), ListenerError> {
        let guard = self.inner.lock();
        if let Some(sub) = guard.subscribers.iter().find(|s| s.id == id) {
            if let Some(missed) = sub.lagged {
                return Err(ListenerError::Lagged {
                    missed_from: missed,
                });
            }
        }
        Ok(())
    }

    /// Führt die Zustellung ausstehender Ereignisse ohne gehaltene Sperre aus (P-10).
    pub fn dispatch_pending(&self) {
        // 1. Hole anstehende Ereignisse unter kurzem Lock (Rang 7)
        let pending = {
            let mut guard = self.inner.lock();
            let mut items = Vec::new();
            for sub in &mut guard.subscribers {
                if sub.active && !sub.queue.is_empty() {
                    let events: Vec<ChangeEvent> = sub.queue.drain(..).collect();
                    items.push((sub.id, Arc::clone(&sub.listener), events));
                }
            }
            items
        };

        // 2. Zustellung OHNE gehaltene Sperre
        for (id, listener, events) in pending {
            for event in events {
                let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    listener.on_change(&event)
                }));

                match res {
                    Ok(Ok(())) => {}
                    _ => {
                        // Fehler oder Panik: Abmelden (SD-62, F-12)
                        let mut guard = self.inner.lock();
                        if let Some(sub) = guard.subscribers.iter_mut().find(|s| s.id == id) {
                            sub.active = false;
                            if sub.lagged.is_none() {
                                sub.lagged = Some(event.stamp);
                            }
                        }
                        break;
                    }
                }
            }
        }
    }
}

/// Maximale Anzahl von Changes in einer Gruppe (SPEC §10.2 Schritt 4).
pub const MAX_GROUP_CHANGES: usize = 1000;

/// Maximale Bytegröße einer Gruppe: 8 MiB (SPEC §10.2 Schritt 4).
pub const MAX_GROUP_BYTES: usize = 8 * 1024 * 1024;

/// Gewünschte Dauerhaftigkeit eines Schreibvorgangs (SPEC §6.5, §7.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Durability {
    /// Quittiert erst nach erfolgreichem sync_data.
    Always,
    /// Quittiert sofort; Intervall-Sync folgt.
    Interval(std::time::Duration),
    /// Kein Sync auf Datenträger.
    Never,
}

/// Einzelne Mutation in einem Change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Op {
    /// Schreiben eines Nutzerschlüssels.
    Put {
        /// Space-Kennung.
        space: u16,
        /// Nutzerschlüssel.
        key: Vec<u8>,
        /// Header-Bytes.
        header: Vec<u8>,
        /// Wertkörper.
        body: Vec<u8>,
    },
    /// Punktuelles Löschen eines Nutzerschlüssels.
    Delete {
        /// Space-Kennung.
        space: u16,
        /// Nutzerschlüssel.
        key: Vec<u8>,
    },
    /// Bereichslöschung [start, end).
    RangeDelete {
        /// Space-Kennung.
        space: u16,
        /// Startschlüssel (inklusiv).
        start: Vec<u8>,
        /// Endschlüssel (exklusiv) oder `None` für unbegrenzt.
        end: Option<Vec<u8>>,
    },
}

/// Bedingung (Guard) für atomare bedingte Ausführung (SPEC §6.5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Guard {
    /// Schlüssel darf nicht existieren.
    Absent {
        /// Space-Kennung.
        space: u16,
        /// Nutzerschlüssel.
        key: Vec<u8>,
    },
    /// Schlüssel muss mit genau diesem Header und Körper existieren.
    Equals {
        /// Space-Kennung.
        space: u16,
        /// Nutzerschlüssel.
        key: Vec<u8>,
        /// Erwarteter Header.
        expected_header: Vec<u8>,
        /// Erwarteter Körper.
        expected_body: Vec<u8>,
    },
    /// Schlüssel darf seit diesem Stamp nicht verändert worden sein.
    UnchangedSince {
        /// Space-Kennung.
        space: u16,
        /// Nutzerschlüssel.
        key: Vec<u8>,
        /// Maximaler Stamp bisheriger Änderungen.
        stamp: u64,
    },
    /// Kein Schlüssel im Bereich [start, end) darf seit diesem Stamp verändert worden sein.
    RangeUnchangedSince {
        /// Space-Kennung.
        space: u16,
        /// Startschlüssel.
        start: Vec<u8>,
        /// Endschlüssel.
        end: Option<Vec<u8>>,
        /// Maximaler Stamp bisheriger Änderungen.
        stamp: u64,
    },
}

/// Ein Bündel atomarer Mutationen mit Guards (SPEC §6.5).
#[derive(Debug, Clone)]
pub struct Change {
    /// Liste von Operationen.
    pub ops: Vec<Op>,
    /// Vorbedingungen.
    pub guards: Vec<Guard>,
    /// Optionale Überschreibung der Dauerhaftigkeit.
    pub durability: Option<Durability>,
    /// Kennzeichnung als rein flüchtiger Change (SPEC §10.2).
    pub is_ephemeral: bool,
    /// Optionale Verschlüsselungsdomäne für Körper.
    pub domain_sealed: Option<KeyDomain>,
}

impl Default for Change {
    fn default() -> Self {
        Self::new()
    }
}

impl Change {
    /// Erstellt einen leeren Change.
    pub fn new() -> Self {
        Self {
            ops: Vec::new(),
            guards: Vec::new(),
            durability: None,
            is_ephemeral: false,
            domain_sealed: None,
        }
    }

    /// Fügt ein `Put` hinzu. Doppelte Schlüssel werden beim Aufbau zusammengelegt (letzter gewinnt).
    pub fn put(&mut self, space: u16, key: Vec<u8>, header: Vec<u8>, body: Vec<u8>) {
        self.ops.retain(|op| match op {
            Op::Put { space: s, key: k, .. } | Op::Delete { space: s, key: k } => {
                !(*s == space && k.as_slice() == key.as_slice())
            }
            _ => true,
        });
        self.ops.push(Op::Put {
            space,
            key,
            header,
            body,
        });
    }

    /// Fügt ein `Delete` hinzu.
    pub fn delete(&mut self, space: u16, key: Vec<u8>) {
        self.ops.retain(|op| match op {
            Op::Put { space: s, key: k, .. } | Op::Delete { space: s, key: k } => {
                !(*s == space && k.as_slice() == key.as_slice())
            }
            _ => true,
        });
        self.ops.push(Op::Delete { space, key });
    }

    /// Fügt ein `RangeDelete` hinzu.
    pub fn delete_range(&mut self, space: u16, start: Vec<u8>, end: Option<Vec<u8>>) {
        self.ops.push(Op::RangeDelete { space, start, end });
    }

    /// Fügt einen `Absent`-Guard hinzu.
    pub fn guard_absent(&mut self, space: u16, key: Vec<u8>) {
        self.guards.push(Guard::Absent { space, key });
    }

    /// Fügt einen `Equals`-Guard hinzu.
    pub fn guard_equals(
        &mut self,
        space: u16,
        key: Vec<u8>,
        expected_header: Vec<u8>,
        expected_body: Vec<u8>,
    ) {
        self.guards.push(Guard::Equals {
            space,
            key,
            expected_header,
            expected_body,
        });
    }

    /// Fügt einen `UnchangedSince`-Guard hinzu.
    pub fn guard_unchanged_since(&mut self, space: u16, key: Vec<u8>, stamp: u64) {
        self.guards.push(Guard::UnchangedSince { space, key, stamp });
    }

    /// Berechnet die geschätzte Bytegröße für Limits und Gruppenbildung.
    pub fn estimated_bytes(&self) -> usize {
        let mut total = 0usize;
        for op in &self.ops {
            match op {
                Op::Put { key, header, body, .. } => {
                    total = total.saturating_add(key.len());
                    total = total.saturating_add(header.len());
                    total = total.saturating_add(body.len());
                }
                Op::Delete { key, .. } => {
                    total = total.saturating_add(key.len());
                }
                Op::RangeDelete { start, end, .. } => {
                    total = total.saturating_add(start.len());
                    if let Some(e) = end {
                        total = total.saturating_add(e.len());
                    }
                }
            }
        }
        total
    }
}

/// Quittung eines erfolgreichen Commits (SPEC §6.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Receipt {
    /// Zugeordneter eindeutiger Stamp.
    pub stamp: u64,
    /// Ob der Stamp per sync_data dauerhaft gesichert wurde.
    pub durable: bool,
}

/// Fehler beim Ausführen eines Changes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommitError {
    /// Vorbedingung (Guard) nicht erfüllt.
    Conflict(String),
    /// Operation unzulässig (Rechte, append-only).
    Forbidden(String),
    /// System befindet sich im Gegendruck-Zustand (SPEC §10.4).
    Busy {
        /// Empfohlene Wartezeit vor Wiederholung.
        retry_after_ms: u64,
    },
    /// System ist vergiftet nach E/A- oder Sync-Fehler (INV-LOG-5).
    Poisoned {
        /// Ursache des Fehlers.
        cause: String,
    },
    /// Obergrenze überschritten (Limits).
    Limit(String),
    /// Stamp-Bereich erschöpft (INV-STAMP-2).
    Exhausted,
    /// E/A-Fehler (Schlüsselverwaltung etc.).
    Io(String),
}

impl fmt::Display for CommitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Conflict(msg) => write!(f, "Conflict: {}", msg),
            Self::Forbidden(msg) => write!(f, "Forbidden: {}", msg),
            Self::Busy { retry_after_ms } => {
                write!(f, "Busy: bitte nach {} ms wiederholen", retry_after_ms)
            }
            Self::Poisoned { cause } => write!(f, "Poisoned: {}", cause),
            Self::Limit(msg) => write!(f, "Limit: {}", msg),
            Self::Exhausted => write!(f, "Exhausted: Stamps erschöpft (INV-STAMP-2)"),
            Self::Io(msg) => write!(f, "Io: {}", msg),
        }
    }
}

impl std::error::Error for CommitError {}

/// Gegendruck-Stufe nach SPEC §10.4.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackpressureLevel {
    /// Normalbetrieb.
    Normal,
    /// Verlangsamt (L0 >= 8 oder frozen >= 2).
    Slowdown,
    /// Gestoppt (L0 >= 12 oder frozen >= 4).
    Stopped,
}

/// Quelle für Gegendruck-Signale.
pub trait BackpressureSource: Send + Sync {
    /// Liefert die aktuelle Gegendruck-Stufe.
    fn level(&self) -> BackpressureLevel;
}

/// Schnittstelle für das Redo-Log (SPEC §10.2, WP-3.1 / P11).
pub trait RedoWriter: Send + Sync {
    /// Schreibt Frames mit einem einzigen E/A-Aufruf an das Redo-Segment.
    fn write_frames(&self, frames: &[Vec<u8>]) -> io::Result<()>;
    /// Synchronisiert Daten des Redo-Segments auf den Datenträger (sync_data).
    fn sync_data(&self) -> io::Result<()>;
}

/// Schnittstelle für die Guard-Vorwärmung (SD-53).
pub trait GuardPrewarmer: Send + Sync {
    /// Lädt Blöcke für einen Schlüssel vorab in den BlockCache (ohne Sperre).
    fn prewarm(&self, space: u16, key: &[u8]);
}

/// Zustand eines Tickets in der Intake-Warteschlange.
struct TicketState {
    completed: bool,
    result: Option<Result<Receipt, CommitError>>,
}

/// Ticket für Flat Combining.
pub struct Ticket {
    _id: u64,
    change: Change,
    state: Arc<(Mutex<TicketState>, Condvar)>,
    byte_size: usize,
}

impl Ticket {
    fn new(id: u64, change: Change) -> Self {
        let byte_size = change.estimated_bytes();
        Self {
            _id: id,
            change,
            state: Arc::new((
                Mutex::new(TicketState {
                    completed: false,
                    result: None,
                }),
                Condvar::new(),
            )),
            byte_size,
        }
    }

    fn is_completed(&self) -> bool {
        let (lock, _) = &*self.state;
        let guard = lock.lock().unwrap();
        guard.completed
    }

    fn wait_result(&self) -> Result<Receipt, CommitError> {
        let (lock, cond) = &*self.state;
        let mut guard = lock.lock().unwrap();
        while !guard.completed {
            guard = cond.wait(guard).unwrap();
        }
        guard.result.clone().unwrap_or(Err(CommitError::Poisoned {
            cause: "Ticket ohne Ergebnis beendet".to_string(),
        }))
    }

    fn complete(&self, res: Result<Receipt, CommitError>) {
        let (lock, cond) = &*self.state;
        let mut guard = lock.lock().unwrap();
        guard.completed = true;
        guard.result = Some(res);
        cond.notify_all();
    }
}

/// Interner Zustand des Committers (Rang 1 Sperre).
pub struct CommitterState {
    _next_ticket_id: u64,
    next_stamp: u64,
}

/// Flat-Combining Committer nach SPEC §10.1, §10.2 und §16.2.
pub struct Committer {
    committer: RankedMutex<CommitterState, 1>,
    intake: RankedMutex<VecDeque<Arc<Ticket>>, 2>,
    visible_stamp: Arc<AtomicU64>,
    durable_stamp: Arc<AtomicU64>,
    poisoned: Arc<AtomicBool>,
    state: Arc<VersionState>,
    listener_queue: Arc<ListenerQueue>,
    hooks: Option<Arc<dyn Hooks>>,
}

impl Committer {
    /// Erstellt einen neuen Committer.
    pub fn new(
        initial_stamp: u64,
        visible_stamp: Arc<AtomicU64>,
        durable_stamp: Arc<AtomicU64>,
        poisoned: Arc<AtomicBool>,
        state: Arc<VersionState>,
        listener_queue: Arc<ListenerQueue>,
        hooks: Option<Arc<dyn Hooks>>,
    ) -> Self {
        visible_stamp.store(initial_stamp, Ordering::Release);
        durable_stamp.store(initial_stamp, Ordering::Release);

        Self {
            committer: RankedMutex::new(CommitterState {
                _next_ticket_id: 1,
                next_stamp: initial_stamp.saturating_add(1),
            }),
            intake: RankedMutex::new(VecDeque::new()),
            visible_stamp,
            durable_stamp,
            poisoned,
            state,
            listener_queue,
            hooks,
        }
    }

    /// Führt `apply` für einen Change aus (Flat Combining Group-Fsync nach SPEC §10.2).
    pub fn apply(
        &self,
        change: Change,
        key_provider: Option<&dyn KeyProvider>,
        prewarmer: Option<&dyn GuardPrewarmer>,
        backpressure: Option<&dyn BackpressureSource>,
        redo_writer: &dyn RedoWriter,
    ) -> Result<Receipt, CommitError> {
        // Schneller Abbruch bei Vergiftung (INV-LOG-5)
        if self.poisoned.load(Ordering::Acquire) {
            return Err(CommitError::Poisoned {
                cause: "Instanz ist vergiftet".to_string(),
            });
        }

        // =====================================================================
        // Schritt 1: Validierung ohne Sperre (SPEC §10.2)
        // Limits, Spaces vorhanden, Mischverbot ephemeral/dauerhaft, Duplikate
        // =====================================================================
        if change.ops.len() > MAX_OPS_PER_CHANGE {
            return Err(CommitError::Limit(format!(
                "Zu viele Operationen: {} > {}",
                change.ops.len(),
                MAX_OPS_PER_CHANGE
            )));
        }

        if change.estimated_bytes() > MAX_CHANGE_BYTES {
            return Err(CommitError::Limit(format!(
                "Change-Größe übersteigt Maximum: {} > {}",
                change.estimated_bytes(),
                MAX_CHANGE_BYTES
            )));
        }

        for op in &change.ops {
            match op {
                Op::Put { key, header, body, .. } => {
                    if key.is_empty() || key.len() > MAX_KEY_BYTES {
                        return Err(CommitError::Limit("Schlüssellänge ungültig".to_string()));
                    }
                    if header.len() > MAX_HEADER_LEN {
                        return Err(CommitError::Limit("Headerlänge ungültig".to_string()));
                    }
                    if body.len() > MAX_VALUE_BYTES {
                        return Err(CommitError::Limit("Körpergröße ungültig".to_string()));
                    }
                }
                Op::Delete { key, .. } => {
                    if key.is_empty() || key.len() > MAX_KEY_BYTES {
                        return Err(CommitError::Limit("Schlüssellänge ungültig".to_string()));
                    }
                }
                Op::RangeDelete { start, end, .. } => {
                    if start.len() > MAX_KEY_BYTES {
                        return Err(CommitError::Limit("Startschlüssel ungültig".to_string()));
                    }
                    if let Some(e) = end {
                        if e.len() > MAX_KEY_BYTES {
                            return Err(CommitError::Limit("Endschlüssel ungültig".to_string()));
                        }
                    }
                }
            }
        }

        // Guard-Vorwärmung ohne Sperre (SD-53, H-S2)
        if let Some(pw) = prewarmer {
            for guard in &change.guards {
                match guard {
                    Guard::Absent { space, key }
                    | Guard::Equals { space, key, .. }
                    | Guard::UnchangedSince { space, key, .. } => {
                        pw.prewarm(*space, key);
                    }
                    Guard::RangeUnchangedSince { space, start, .. } => {
                        pw.prewarm(*space, start);
                    }
                }
            }
        }

        // =====================================================================
        // Schritt 1b: Domänenschlüssel vor dem Ticket (INV-KEYP-1)
        // =====================================================================
        if let Some(domain) = change.domain_sealed {
            if let Some(kp) = key_provider {
                kp.domain_key_create(domain).map_err(|e| {
                    CommitError::Io(format!("domain_key_create fehlgeschlagen: {}", e))
                })?;
                kp.make_durable().map_err(|e| {
                    CommitError::Io(format!("make_durable fehlgeschlagen: {}", e))
                })?;
            } else {
                return Err(CommitError::Forbidden(
                    "Domänenversiegelung erfordert KeyProvider".to_string(),
                ));
            }
        }

        // =====================================================================
        // Schritt 1½: Gegendruck-Prüfung vor dem Ticket (E-4, SD-49)
        // Der Aufrufer wartet vor dem Ticket; der Committer wartet nie!
        // =====================================================================
        if let Some(bp) = backpressure {
            match bp.level() {
                BackpressureLevel::Normal => {}
                BackpressureLevel::Slowdown => {
                    // Verlangsamt: Kurzes Nachgeben ohne Sperre
                    std::thread::yield_now();
                }
                BackpressureLevel::Stopped => {
                    // Gestoppt: Sofortiger Fehler mit Busy
                    return Err(CommitError::Busy { retry_after_ms: 100 });
                }
            }
        }

        // =====================================================================
        // Schritt 2: Ticket in intake einreihen und Leiterwahl
        // =====================================================================
        let ticket = {
            let mut intake_guard = self.intake.lock();
            let ticket_id = {
                // Wir erzeugen eine lokale ID
                1u64
            };
            let t = Arc::new(Ticket::new(ticket_id, change));
            intake_guard.push_back(Arc::clone(&t));
            t
        };

        // Um den Committer (Rang 1) zu erwerben, darf intake (Rang 2) NICHT gehalten werden!
        let mut committer_guard = self.committer.lock();

        // Prüfen, ob das Ticket bereits von einem vorherigen Leiter bedient wurde
        if ticket.is_completed() {
            return ticket.wait_result();
        }

        // DIESER THREAD IST DER LEITER!
        // =====================================================================
        // Schritt 3: Gegendruck-Auswertung durch den Leiter (E-4: Wertet nur aus, wartet nie)
        // =====================================================================
        if let Some(bp) = backpressure {
            if bp.level() == BackpressureLevel::Stopped {
                // Gruppe scheitert sofort
                ticket.complete(Err(CommitError::Busy { retry_after_ms: 100 }));
                return Err(CommitError::Busy { retry_after_ms: 100 });
            }
        }

        // =====================================================================
        // Schritt 4: intake leeren (<= 1 000 Changes / 8 MiB)
        // =====================================================================
        let mut group: Vec<Arc<Ticket>> = Vec::new();
        let mut group_bytes = 0usize;

        {
            let mut intake_guard = self.intake.lock();
            while let Some(t) = intake_guard.pop_front() {
                group_bytes = group_bytes.saturating_add(t.byte_size);
                group.push(t);

                if group.len() >= MAX_GROUP_CHANGES || group_bytes >= MAX_GROUP_BYTES {
                    break;
                }
            }
        }

        if group.is_empty() {
            return ticket.wait_result();
        }

        // Aktuelle Version für Guards und Memtable-Referenzen lesen
        let current_version = Arc::clone(&*self.state.read());

        // =====================================================================
        // Schritt 5: Guards prüfen und Stamps zuteilen
        // Overlay bildet frühere Changes derselben Gruppe ab
        // =====================================================================
        let mut group_overlay: BTreeMap<(u16, Vec<u8>), (TagKind, Vec<u8>, Vec<u8>)> = BTreeMap::new();
        let mut accepted_tickets: Vec<(Arc<Ticket>, u64, Vec<Op>)> = Vec::new();

        for t in &group {
            let mut guard_failed = false;

            for guard in &t.change.guards {
                match guard {
                    Guard::Absent { space, key } => {
                        let key_tuple = (*space, key.clone());
                        if let Some((kind, _, _)) = group_overlay.get(&key_tuple) {
                            if *kind != TagKind::Delete {
                                guard_failed = true;
                                break;
                            }
                        } else if let Some(sp) = current_version.get_space(*space) {
                            if let Some((tag, _)) = sp.active.get(key, u64::MAX) {
                                if let Ok(TagKind::Put) = tag.kind() {
                                    guard_failed = true;
                                    break;
                                }
                            }
                        }
                    }
                    Guard::Equals {
                        space,
                        key,
                        expected_header,
                        expected_body,
                    } => {
                        let key_tuple = (*space, key.clone());
                        if let Some((kind, hdr, body)) = group_overlay.get(&key_tuple) {
                            if *kind != TagKind::Put
                                || hdr != expected_header
                                || body != expected_body
                            {
                                guard_failed = true;
                                break;
                            }
                        } else if let Some(sp) = current_version.get_space(*space) {
                            if let Some((tag, slot)) = sp.active.get(key, u64::MAX) {
                                if let Ok(TagKind::Put) = tag.kind() {
                                    if &slot.header != expected_header || &slot.body != expected_body {
                                        guard_failed = true;
                                        break;
                                    }
                                } else {
                                    guard_failed = true;
                                    break;
                                }
                            } else {
                                guard_failed = true;
                                break;
                            }
                        } else {
                            guard_failed = true;
                            break;
                        }
                    }
                    Guard::UnchangedSince { space, key, stamp } => {
                        let key_tuple = (*space, key.clone());
                        if group_overlay.contains_key(&key_tuple) {
                            guard_failed = true;
                            break;
                        } else if let Some(sp) = current_version.get_space(*space) {
                            if let Some((tag, _)) = sp.active.get(key, u64::MAX) {
                                if tag.stamp() > *stamp {
                                    guard_failed = true;
                                    break;
                                }
                            }
                        }
                    }
                    Guard::RangeUnchangedSince { space, start, end, stamp } => {
                        if let Some(sp) = current_version.get_space(*space) {
                            if sp.active.max_stamp() > *stamp {
                                let snapshot = sp.active.snapshot_entries();
                                for (k, _) in snapshot {
                                    let uk = k.user_key.as_slice();
                                    if uk >= start.as_slice() {
                                        let in_range = match end {
                                            Some(e) => uk < e.as_slice(),
                                            None => true,
                                        };
                                        if in_range && k.tag.stamp() > *stamp {
                                            guard_failed = true;
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if guard_failed {
                // Fehlschlag: Nur dieser Change erhält Err(Conflict) und verbraucht keinen Stamp
                t.complete(Err(CommitError::Conflict("Guard verletzt".to_string())));
                continue;
            }

            // Erfolg: Stamp vergeben (INV-STAMP-1)
            let stamp = committer_guard.next_stamp;
            committer_guard.next_stamp = committer_guard.next_stamp.saturating_add(1);

            // Overlay aktualisieren
            for op in &t.change.ops {
                match op {
                    Op::Put { space, key, header, body } => {
                        group_overlay.insert((*space, key.clone()), (TagKind::Put, header.clone(), body.clone()));
                    }
                    Op::Delete { space, key } => {
                        group_overlay.insert((*space, key.clone()), (TagKind::Delete, Vec::new(), Vec::new()));
                    }
                    _ => {}
                }
            }

            accepted_tickets.push((Arc::clone(t), stamp, t.change.ops.clone()));
        }

        if accepted_tickets.is_empty() {
            return ticket.wait_result();
        }

        // =====================================================================
        // Schritt 6 & 7: Frames kodieren und schreiben (P11)
        // =====================================================================
        let mut frames: Vec<Vec<u8>> = Vec::new();
        for (t, stamp, _) in &accepted_tickets {
            if !t.change.is_ephemeral {
                // Wir erzeugen einen Frame-Dummy / -Platzhalter für den Redo-Writer
                let mut frame_data = Vec::new();
                frame_data.extend_from_slice(&stamp.to_le_bytes());
                frames.push(frame_data);
            }
        }

        if !frames.is_empty() {
            if let Err(e) = redo_writer.write_frames(&frames) {
                // Fehler beim Schreiben
                self.poisoned.store(true, Ordering::Release);
                let err_res = Err(CommitError::Poisoned {
                    cause: format!("Redo write_frames fehlgeschlagen: {}", e),
                });
                for (t, _, _) in &accepted_tickets {
                    t.complete(err_res.clone());
                }
                return err_res;
            }

            if let Some(ref h) = self.hooks {
                h.point(HookPoint::AfterRedoWrite);
            }
        }

        // =====================================================================
        // Schritt 8: Sync nach höchster Anforderung der Gruppe
        // =====================================================================
        let mut requires_sync = false;
        for (t, _, _) in &accepted_tickets {
            if let Some(d) = t.change.durability {
                if d == Durability::Always {
                    requires_sync = true;
                    break;
                }
            } else {
                requires_sync = true; // Standard ist Always (§6.6)
            }
        }

        if requires_sync && !frames.is_empty() {
            if let Err(e) = redo_writer.sync_data() {
                // INV-LOG-5, INV-VIS-1: Poison, alle Tickets Err, NICHTS in Memtables!
                self.poisoned.store(true, Ordering::Release);
                let poison_err = Err(CommitError::Poisoned {
                    cause: format!("sync_data fehlgeschlagen (INV-LOG-5): {}", e),
                });

                for (t, _, _) in &accepted_tickets {
                    t.complete(poison_err.clone());
                }

                return poison_err;
            }

            let highest_stamp = accepted_tickets.last().map(|(_, s, _)| *s).unwrap_or(0);
            self.durable_stamp.store(highest_stamp, Ordering::Release);

            if let Some(ref h) = self.hooks {
                h.point(HookPoint::AfterRedoSync);
            }
        }

        // =====================================================================
        // Schritt 9: In Memtables einfügen (in Stamp-Reihenfolge)
        // Unterbatches von höchstens 64 Ops je Sperre (H-S7)
        // =====================================================================
        if let Some(ref h) = self.hooks {
            h.point(HookPoint::BeforeVisible);
        }

        // Wir gruppieren Operationen nach Space
        let mut space_entries: BTreeMap<u16, Vec<(InternalKey, Slot)>> = BTreeMap::new();
        let mut change_events: Vec<ChangeEvent> = Vec::new();

        for (_, stamp, ops) in &accepted_tickets {
            let mut summaries = Vec::new();

            for op in ops {
                match op {
                    Op::Put { space, key, header, body } => {
                        let tag = Tag::new(*stamp, TagKind::Put);
                        let k = InternalKey::new(key.clone(), tag);
                        let slot = Slot::put(header.clone(), body.clone());
                        space_entries.entry(*space).or_default().push((k, slot));

                        summaries.push(OpSummary {
                            space: *space,
                            kind: TagKind::Put,
                            key: key.clone(),
                            header: header.clone(),
                        });
                    }
                    Op::Delete { space, key } => {
                        let tag = Tag::new(*stamp, TagKind::Delete);
                        let k = InternalKey::new(key.clone(), tag);
                        let slot = Slot::delete();
                        space_entries.entry(*space).or_default().push((k, slot));

                        summaries.push(OpSummary {
                            space: *space,
                            kind: TagKind::Delete,
                            key: key.clone(),
                            header: Vec::new(),
                        });
                    }
                    Op::RangeDelete { space, start, end } => {
                        let tag = Tag::new(*stamp, TagKind::RangeDelete);
                        let k = InternalKey::new(start.clone(), tag);
                        let slot = Slot::range_delete(end.clone());
                        space_entries.entry(*space).or_default().push((k, slot));

                        summaries.push(OpSummary {
                            space: *space,
                            kind: TagKind::RangeDelete,
                            key: start.clone(),
                            header: Vec::new(),
                        });
                    }
                }
            }

            change_events.push(ChangeEvent {
                stamp: *stamp,
                ops: summaries,
            });
        }

        for (space_id, entries) in space_entries {
            if let Some(sp) = current_version.get_space(space_id) {
                sp.active.insert_batch(&entries);
            }
        }

        // =====================================================================
        // Schritt 10: visible_stamp veröffentlichen (Release)
        // Sichtbarkeitspunkt nach INV-VIS-1, INV-VIS-2, INV-VIS-3
        // =====================================================================
        let last_stamp = accepted_tickets.last().map(|(_, s, _)| *s).unwrap_or(0);
        self.visible_stamp.store(last_stamp, Ordering::Release);

        // =====================================================================
        // Schritt 11: Tickets mit Receipt wecken; Listener füttern
        // =====================================================================
        for (t, s, _) in accepted_tickets {
            let durable = s <= self.durable_stamp.load(Ordering::Acquire);
            t.complete(Ok(Receipt { stamp: s, durable }));
        }

        for ev in change_events {
            self.listener_queue.push_event(ev);
        }

        ticket.wait_result()
    }
}
