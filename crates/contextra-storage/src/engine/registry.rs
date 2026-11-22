//! # Registry: Sichten, Halteregeln, Konsumenten, Untergrenze und Claims
//!
//! Normative Referenzen: SPEC §10.3, §16.1 (INV-LOCK-1), §16.4 (INV-RET-1), §16.5 (INV-CLAIM-1, INV-SEG-1).
//! INV-SEG-1: Ein Segment wird nur mit gültigem Siegel-Footer und per Edit lebendig; Rahmenprüfung schützt vor Beschädigung.
//! Keine Integer-Casts (M-02 = 0).

use std::collections::BTreeMap;
use std::fmt;
use contextra_storage_ports::{FileKind, FileRef};
use crate::engine::version::FileHandle;
use crate::sync::{Arc, AtomicBool, AtomicU64, Ordering, RankedMutex};

/// Fehler der Registry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistryError {
    /// Angefordertes Zeitalter wurde bereits verworfen (`stamp < retention_floor`).
    HistoryPruned {
        /// Angefragter Stamp.
        requested: u64,
        /// Aktuelle Untergrenze.
        floor: u64,
    },
    /// Angefragter Stamp liegt in der Zukunft (`stamp > visible_stamp`).
    FutureStamp {
        /// Angefragter Stamp.
        requested: u64,
        /// Aktueller sichtbarer Stamp.
        visible: u64,
    },
    /// Claim-Konflikt oder Datei veraltet.
    ClaimFailed(String),
}

impl fmt::Display for RegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HistoryPruned { requested, floor } => {
                write!(
                    f,
                    "HistoryPruned: Angefragter Stamp {} liegt unter der Untergrenze {}",
                    requested, floor
                )
            }
            Self::FutureStamp { requested, visible } => {
                write!(
                    f,
                    "FutureStamp: Angefragter Stamp {} liegt über dem sichtbaren Stamp {}",
                    requested, visible
                )
            }
            Self::ClaimFailed(msg) => write!(f, "ClaimFailed: {}", msg),
        }
    }
}

impl std::error::Error for RegistryError {}

/// Halteregel (Hold) zum Schutz historischer Stände vor Compaction/Retention.
#[derive(Debug, Clone)]
pub struct Hold {
    /// Eindeutige ID des Holds.
    pub id: u64,
    /// Eigentümer des Holds (z. B. Branch-Name oder Transaktions-ID).
    pub owner: String,
    /// Zu schützender Mindest-Stamp.
    pub stamp: u64,
    /// Optionaler Ablaufzeitpunkt in Millisekunden (TTL).
    pub expires_at_ms: Option<u64>,
}

/// Registrierter Konsument für Ereignisse / History.
#[derive(Debug, Clone)]
pub struct Consumer {
    /// Eindeutige ID des Konsumenten.
    pub id: u64,
    /// Name des Konsumenten.
    pub name: String,
    /// Zuletzt konsumierter Stamp.
    pub consumed_stamp: u64,
}

/// Zeile in der Claim-Tabelle (SPEC §16.5).
#[derive(Debug, Clone)]
pub struct Claim {
    /// ID des Jobs, der den Anspruch hält.
    pub job_id: u64,
    /// Abbruchflag des Jobs.
    pub cancel_flag: Arc<AtomicBool>,
    /// Ob der Job auf der urgent-Lane läuft.
    pub is_urgent: bool,
}

/// Interner Zustand der Registry.
pub struct RegistryState {
    next_id: u64,
    views: BTreeMap<u64, u64>,
    holds: BTreeMap<u64, Hold>,
    consumers: BTreeMap<u64, Consumer>,
    retention_floor: u64,
    claims: BTreeMap<FileRef, Claim>,
}

impl RegistryState {
    fn new() -> Self {
        Self {
            next_id: 1,
            views: BTreeMap::new(),
            holds: BTreeMap::new(),
            consumers: BTreeMap::new(),
            retention_floor: 0,
            claims: BTreeMap::new(),
        }
    }

    fn compute_floor(&self, visible_stamp: u64) -> u64 {
        let mut min_val = visible_stamp;

        for &v_stamp in self.views.values() {
            if v_stamp < min_val {
                min_val = v_stamp;
            }
        }

        for hold in self.holds.values() {
            if hold.stamp < min_val {
                min_val = hold.stamp;
            }
        }

        for consumer in self.consumers.values() {
            if consumer.consumed_stamp < min_val {
                min_val = consumer.consumed_stamp;
            }
        }

        min_val
    }
}

/// Thread-sichere Registry unter Rang 5 Sperre (INV-LOCK-1).
pub struct Registry {
    state: RankedMutex<RegistryState, 5>,
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

impl Registry {
    /// Erstellt eine neue Registry.
    pub fn new() -> Self {
        Self {
            state: RankedMutex::new(RegistryState::new()),
        }
    }

    /// Registriert eine neue View beim aktuellen `visible_stamp` (INV-RET-1).
    ///
    /// Berechnet die neue Untergrenze und trägt die View atomar unter der Registry-Sperre ein.
    pub fn register_view(&self, visible_stamp: &AtomicU64) -> (u64, u64) {
        let mut guard = self.state.lock();
        let v = visible_stamp.load(Ordering::Acquire);

        let floor = guard.compute_floor(v);
        if floor > guard.retention_floor {
            guard.retention_floor = floor;
        }

        let id = guard.next_id;
        guard.next_id = guard.next_id.saturating_add(1);
        guard.views.insert(id, v);

        (id, v)
    }

    /// Meldet eine zuvor registrierte View ab.
    pub fn unregister_view(&self, view_id: u64) {
        let mut guard = self.state.lock();
        guard.views.remove(&view_id);
    }

    /// Registriert eine historische View zu einem bestimmten Stamp (SPEC §16.4).
    ///
    /// Prüft atomar unter der Registry-Sperre: `stamp >= retention_floor`.
    /// Scheitert mit `HistoryPruned`, falls der Stand bereits verworfen wurde (INV-RET-1).
    pub fn view_at(
        &self,
        visible_stamp: &AtomicU64,
        requested_stamp: u64,
    ) -> Result<(u64, u64), RegistryError> {
        let mut guard = self.state.lock();
        let v = visible_stamp.load(Ordering::Acquire);

        let floor = guard.compute_floor(v);
        if floor > guard.retention_floor {
            guard.retention_floor = floor;
        }

        if requested_stamp < guard.retention_floor {
            return Err(RegistryError::HistoryPruned {
                requested: requested_stamp,
                floor: guard.retention_floor,
            });
        }

        if requested_stamp > v {
            return Err(RegistryError::FutureStamp {
                requested: requested_stamp,
                visible: v,
            });
        }

        let id = guard.next_id;
        guard.next_id = guard.next_id.saturating_add(1);
        guard.views.insert(id, requested_stamp);

        Ok((id, requested_stamp))
    }

    /// Liefert die aktuelle Untergrenze `retention_floor`.
    ///
    /// Monoton nicht fallend nach INV-RET-1.
    pub fn retention_floor(&self, visible_stamp: &AtomicU64) -> u64 {
        let mut guard = self.state.lock();
        let v = visible_stamp.load(Ordering::Acquire);
        let floor = guard.compute_floor(v);
        if floor > guard.retention_floor {
            guard.retention_floor = floor;
        }
        guard.retention_floor
    }

    /// Fügt einen Hold hinzu.
    pub fn add_hold(&self, owner: String, stamp: u64, expires_at_ms: Option<u64>) -> u64 {
        let mut guard = self.state.lock();
        let id = guard.next_id;
        guard.next_id = guard.next_id.saturating_add(1);
        guard.holds.insert(
            id,
            Hold {
                id,
                owner,
                stamp,
                expires_at_ms,
            },
        );
        id
    }

    /// Entfernt einen Hold.
    pub fn drop_hold(&self, hold_id: u64) {
        let mut guard = self.state.lock();
        guard.holds.remove(&hold_id);
    }

    /// Registriert einen Consumer.
    pub fn register_consumer(&self, name: String, initial_stamp: u64) -> u64 {
        let mut guard = self.state.lock();
        let id = guard.next_id;
        guard.next_id = guard.next_id.saturating_add(1);
        guard.consumers.insert(
            id,
            Consumer {
                id,
                name,
                consumed_stamp: initial_stamp,
            },
        );
        id
    }

    /// Aktualisiert den Fortschritt eines Consumers.
    pub fn advance_consumer(&self, consumer_id: u64, stamp: u64) {
        let mut guard = self.state.lock();
        if let Some(c) = guard.consumers.get_mut(&consumer_id) {
            if stamp > c.consumed_stamp {
                c.consumed_stamp = stamp;
            }
        }
    }

    /// Beansprucht Dateien in der Claim-Tabelle (SPEC §16.5, INV-CLAIM-1).
    ///
    /// All-or-Nothing: Prüft für ALLE Dateien:
    /// 1. `!obsolete`
    /// 2. Nicht bereits von einem anderen Job beansprucht (außer ein `urgent`-Job bricht `background` ab).
    /// Erst wenn alle frei sind, werden alle Zeilen eingetragen.
    pub fn claim(
        &self,
        job_id: u64,
        files: &[Arc<FileHandle>],
        is_urgent: bool,
        cancel_flag: Arc<AtomicBool>,
    ) -> Result<(), RegistryError> {
        let mut guard = self.state.lock();

        // 1. Prüfung für alle Dateien
        for handle in files {
            if handle.is_obsolete() {
                return Err(RegistryError::ClaimFailed(format!(
                    "INV-CLAIM-1: Datei {} ist bereits obsolete",
                    handle.id
                )));
            }

            let file_ref = FileRef {
                kind: FileKind::Run,
                id: handle.id,
            };

            if let Some(existing) = guard.claims.get(&file_ref) {
                if existing.job_id != job_id {
                    if is_urgent && !existing.is_urgent {
                        // Urgent-Job darf Background-Job abbrechen (§12.0)
                        existing.cancel_flag.store(true, Ordering::Release);
                    } else {
                        return Err(RegistryError::ClaimFailed(format!(
                            "INV-CLAIM-1: Datei {} bereits beansprucht durch Job {}",
                            handle.id, existing.job_id
                        )));
                    }
                }
            }
        }

        // 2. Alle eintragen (All-or-Nothing)
        for handle in files {
            let file_ref = FileRef {
                kind: FileKind::Run,
                id: handle.id,
            };
            guard.claims.insert(
                file_ref,
                Claim {
                    job_id,
                    cancel_flag: Arc::clone(&cancel_flag),
                    is_urgent,
                },
            );
        }

        Ok(())
    }

    /// Gibt Claims für die angegebenen Dateien frei.
    pub fn release_claims(&self, files: &[Arc<FileHandle>]) {
        let mut guard = self.state.lock();
        for handle in files {
            let file_ref = FileRef {
                kind: FileKind::Run,
                id: handle.id,
            };
            guard.claims.remove(&file_ref);
        }
    }
}
