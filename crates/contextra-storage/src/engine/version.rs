//! # Versionsverwaltung und unveränderliche Zustandsstrukturen
//!
//! Normative Referenzen: SPEC §10.1, §16.1 (INV-LOCK-1), §16.5 (INV-CLAIM-1).
//! Keine Integer-Casts (M-02 = 0).

use std::collections::BTreeMap;
use crate::engine::memtable::Memtable;
use crate::run::rangedel::RangeMarker;
use crate::sync::{Arc, AtomicBool, Ordering, RankedRwLock};

/// Handle für einen unveränderlichen Run (SSTable) nach P09 und SPEC §16.5.
#[derive(Debug)]
pub struct FileHandle {
    /// Numerische Datei-ID.
    pub id: u64,
    /// Space-ID.
    pub space: u16,
    /// LSM-Level.
    pub level: u8,
    /// Pfad zur Datei auf dem Dateisystem.
    pub path: String,
    /// Minimaler enthaltener Stamp.
    pub min_stamp: u64,
    /// Maximaler enthaltener Stamp.
    pub max_stamp: u64,
    /// Veraltet-Flag für die Claim-Tabelle und Graveyard (§16.5, INV-CLAIM-1).
    pub obsolete: AtomicBool,
}

impl FileHandle {
    /// Erstellt ein neues FileHandle.
    pub fn new(
        id: u64,
        space: u16,
        level: u8,
        path: String,
        min_stamp: u64,
        max_stamp: u64,
    ) -> Self {
        Self {
            id,
            space,
            level,
            path,
            min_stamp,
            max_stamp,
            obsolete: AtomicBool::new(false),
        }
    }

    /// Prüft, ob die Datei als veraltet markiert wurde (§16.5).
    pub fn is_obsolete(&self) -> bool {
        self.obsolete.load(Ordering::Acquire)
    }

    /// Markiert die Datei als veraltet.
    pub fn mark_obsolete(&self) {
        self.obsolete.store(true, Ordering::Release);
    }
}

/// Handle für eine Blob-Datei nach P10 und SPEC §9.6.
#[derive(Debug)]
pub struct BlobHandle {
    /// Blob-Nummer (`blob_no`).
    pub blob_no: u64,
    /// Space-ID.
    pub space: u16,
    /// Generationsnummer.
    pub generation: u32,
    /// Pfad zur Blob-Datei.
    pub path: String,
    /// Veraltet-Flag.
    pub obsolete: AtomicBool,
}

impl BlobHandle {
    /// Erstellt ein neues BlobHandle.
    pub fn new(blob_no: u64, space: u16, generation: u32, path: String) -> Self {
        Self {
            blob_no,
            space,
            generation,
            path,
            obsolete: AtomicBool::new(false),
        }
    }

    /// Prüft, ob die Blob-Datei als veraltet markiert wurde.
    pub fn is_obsolete(&self) -> bool {
        self.obsolete.load(Ordering::Acquire)
    }

    /// Markiert die Blob-Datei als veraltet.
    pub fn mark_obsolete(&self) {
        self.obsolete.store(true, Ordering::Release);
    }
}

/// Zustand eines einzelnen Space in einer Version.
#[derive(Clone)]
pub struct SpaceState {
    /// Aktive Memtable.
    pub active: Arc<Memtable>,
    /// Eingefrorene Memtables (von alt nach neu).
    pub frozen: Vec<Arc<Memtable>>,
    /// Runs geordnet nach Levels: Level 0 (überlappend), Level 1..N (überlappungsfrei).
    pub levels: Vec<Vec<Arc<FileHandle>>>,
    /// Blob-Dateien nach Blob-Nummer geordnet.
    pub blobs: BTreeMap<u64, Arc<BlobHandle>>,
    /// Höchster per Flush gesicherter Stamp.
    pub flushed_stamp: u64,
    /// Aktive physische Bereichslöschungen.
    pub erased_ranges: Vec<RangeMarker>,
}

impl Default for SpaceState {
    fn default() -> Self {
        Self::new()
    }
}

impl SpaceState {
    /// Erstellt einen initialen SpaceState mit einer neuen aktiven Memtable.
    pub fn new() -> Self {
        Self {
            active: Arc::new(Memtable::new()),
            frozen: Vec::new(),
            levels: Vec::new(),
            blobs: BTreeMap::new(),
            flushed_stamp: 0,
            erased_ranges: Vec::new(),
        }
    }

    /// Rotiert die aktive Memtable in die frozen-Liste und erzeugt eine frische aktive Memtable.
    pub fn rotate(&mut self) -> Arc<Memtable> {
        let old_active = Arc::clone(&self.active);
        self.frozen.push(old_active);
        let new_active = Arc::new(Memtable::new());
        self.active = Arc::clone(&new_active);
        new_active
    }
}

/// Unveränderlicher Gesamtzustand aller Spaces (SPEC §10.1).
#[derive(Clone)]
pub struct Version {
    /// Zustände je Space.
    pub spaces: BTreeMap<u16, SpaceState>,
}

impl Default for Version {
    fn default() -> Self {
        Self::new()
    }
}

impl Version {
    /// Erstellt eine leere Version.
    pub fn new() -> Self {
        Self {
            spaces: BTreeMap::new(),
        }
    }

    /// Erstellt eine Version mit gegebenen Spaces.
    pub fn with_spaces(spaces: BTreeMap<u16, SpaceState>) -> Self {
        Self { spaces }
    }

    /// Liefert den Zustand eines Space, falls vorhanden.
    pub fn get_space(&self, space_id: u16) -> Option<&SpaceState> {
        self.spaces.get(&space_id)
    }

    /// Fügt einen neuen Space hinzu oder initialisiert ihn.
    pub fn add_space(&mut self, space_id: u16) {
        self.spaces.entry(space_id).or_default();
    }
}

/// Thread-sicherer Halter für die aktuelle Version (Rang 3 Sperre).
///
/// INV-LOCK-1: Rang 3 Sperre (`state`). Klon-und-Austausch erfolgt in Nanosekunden.
pub type VersionState = RankedRwLock<Arc<Version>, 3>;
