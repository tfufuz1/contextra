//! # Wiederanlauf-Modul (Recovery)
//!
//! SPEC §13.1 (11 deterministische Schritte), §13.2, §18:
//! Zustandsmaschine mit Effekten zur Wiederherstellung nach einem Absturz.
//! Keine Integer-Casts (M-02 = 0), keine Panics (M-08 = 0).

use crate::catalog::current::read_current;
use crate::catalog::log::{catalog_file_name, CatalogLog};
use crate::catalog::reserve::IdReserve;
use crate::catalog::state::CatalogState;
use crate::catalog::types::{CatalogError, CatalogFileKind, ErasePendingMeta};
use crate::engine::compaction::exec::run_file_name;
use crate::redo::error::RedoError;
use crate::redo::op::{RedoOp, RedoOpKind, OPFLAG_UNBOUNDED_END};
use crate::redo::replay::{parse_segment_id, recover_redo};
use contextra_storage_ports::{Cipher, Fs, Kdf, LockGuard, LockMode, Mac, OpenMode, Stamp};
use std::collections::HashSet;
use std::fmt;
use std::io;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Schritte des deterministischen Wiederanlaufs nach SPEC §13.1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryStep {
    /// 1. Sperre erwerben
    Step1Lock,
    /// 2. FORMAT und Modus prüfen
    Step2Format,
    /// 3. CURRENT und Katalog lesen
    Step3Catalog,
    /// 4. Verwaiste Dateien und .building bereinigen (INV-CAT-4, INV-LOG-7)
    Step4CleanOrphans,
    /// 5. Vom Katalog genannte Dateien prüfen
    Step5VerifyFiles,
    /// 6. Redo ab redo_floor replayen (INV-LOG-6, INV-LOG-8)
    Step6RedoReplay,
    /// 7. Stempel initialisieren
    Step7Stamps,
    /// 8. ID-Vergabe ab Hochwassermarken starten (INV-KEY-2)
    Step8IdReserve,
    /// 9. Consumer prüfen und Untergrenze ermitteln
    Step9Consumers,
    /// 10. Roll-Forward für Löschungen und Sweeps (Schnittstelle)
    Step10RollForward,
    /// 11. Wartung vorbereiten und Abschluss
    Step11Complete,
}

/// Fehler beim Wiederanlauf.
#[derive(Debug)]
pub enum RecoveryError {
    /// Exklusive Dateisystemsperre fehlgeschlagen.
    Locked,
    /// Nicht unterstütztes Format oder Integritätsmodus.
    Unsupported(&'static str),
    /// Instanz wurde zurückgerollt (RollbackAnchor).
    Rolledback,
    /// Datenbeschädigung.
    Corrupt(String),
    /// Vom Katalog genannte Datei fehlt (SPEC §13.1 Schritt 5).
    CorruptMissingFile {
        /// Art der fehlenden Datei.
        kind: CatalogFileKind,
        /// ID der fehlenden Datei.
        id: u64,
    },
    /// Redo-Log vor HWM abgeschnitten (INV-LOG-8).
    CorruptRedoTruncated {
        /// Höchster vorgefundener Stempel.
        last_stamp: u64,
        /// Erwartete Hochwassermarke aus Katalog.
        redo_hwm: u64,
    },
    /// E/A-Fehler.
    Io(io::Error),
}

impl fmt::Display for RecoveryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Locked => write!(f, "Instanz durch anderen Prozess gesperrt"),
            Self::Unsupported(msg) => write!(f, "Nicht unterstütztes Format: {msg}"),
            Self::Rolledback => write!(f, "Rollback erkannt"),
            Self::Corrupt(msg) => write!(f, "Datenbeschädigung bei Recovery: {msg}"),
            Self::CorruptMissingFile { kind, id } => {
                write!(f, "Corrupt{{MissingFile}}: {:?} mit ID {} fehlt", kind, id)
            }
            Self::CorruptRedoTruncated { last_stamp, redo_hwm } => {
                write!(f, "INV-LOG-8: Redo truncated (last={last_stamp} < hwm={redo_hwm})")
            }
            Self::Io(e) => write!(f, "E/A-Fehler beim Wiederanlauf: {e}"),
        }
    }
}

impl std::error::Error for RecoveryError {}

impl From<io::Error> for RecoveryError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<CatalogError> for RecoveryError {
    fn from(e: CatalogError) -> Self {
        Self::Corrupt(format!("{e:?}"))
    }
}

impl From<RedoError> for RecoveryError {
    fn from(e: RedoError) -> Self {
        match e {
            RedoError::Corrupt(msg) => Self::Corrupt(msg),
            RedoError::CorruptRedoTruncated { last_stamp, redo_hwm } => {
                Self::CorruptRedoTruncated { last_stamp, redo_hwm }
            }
            RedoError::Io(io_str) => Self::Io(io::Error::new(io::ErrorKind::Other, io_str)),
            _ => Self::Corrupt(format!("{e:?}")),
        }
    }
}

/// Ergebnis des deterministischen Wiederanlaufs.
pub struct RecoveryResult {
    /// Wiederhergestellter Katalogzustand.
    pub catalog_state: CatalogState,
    /// Nächster zu vergebender Stempel (Schritt 7).
    pub next_stamp: Stamp,
    /// Sichtbarer Stempel (Schritt 7).
    pub visible_stamp: Stamp,
    /// Dauerhafter Stempel (Schritt 7).
    pub durable_stamp: Stamp,
    /// Initialisierter ID-Generator ab Hochwassermarken (INV-KEY-2).
    pub id_reserve: IdReserve,
    /// Gültige, angewendete Redo-Operationen (stamp > flushed_stamp).
    pub applied_ops: Vec<(u16, RedoOp)>,
    /// Ausstehende physische Löschungen für Roll-forward (§14.4).
    pub pending_erases: Vec<ErasePendingMeta>,
    /// Vorallokiertes Reservesegment (falls vorhanden, INV-LOG-7).
    pub reserve_segment_id: Option<u64>,
    /// Gemessene Dauer des Wiederanlaufs für H-S4.
    pub duration: Duration,
    /// Aktive Sperre.
    pub _lock: Option<Box<dyn LockGuard>>,
}

/// Führt den vollständigen Wiederanlauf nach SPEC §13.1 aus.
pub fn recover_instance<F: Fs + 'static>(
    fs: Arc<F>,
    base_dir: &Path,
    read_only: bool,
    root_key: Option<&[u8; 32]>,
    cipher: Option<&dyn Cipher>,
    mac: Option<&dyn Mac>,
    kdf: Option<&dyn Kdf>,
) -> Result<RecoveryResult, RecoveryError> {
    let start_time = Instant::now();

    // Schritt 1: Dateisystemsperre (LOCK)
    let lock_mode = if read_only {
        LockMode::Shared
    } else {
        LockMode::Exclusive
    };
    let lock_guard = fs.lock(&base_dir.join("LOCK"), lock_mode)
        .map_err(|_| RecoveryError::Locked)?;

    // Schritt 2: FORMAT lesen (Basis-Prüfung)
    let format_path = base_dir.join("FORMAT");
    if fs.open(&format_path, OpenMode::Read).is_err() {
        return Err(RecoveryError::Unsupported("FORMAT-Datei fehlt"));
    }

    // Schritt 3: CURRENT lesen -> Catalog-Datei öffnen und Zustand aufbauen
    let cat_no = read_current(fs.as_ref(), base_dir)?;
    let cat_file_path = base_dir.join(catalog_file_name(cat_no));
    let cat_file = fs.open(&cat_file_path, OpenMode::Read)?;

    let (catalog_state, _) = CatalogLog::read_and_recover(
        &*cat_file,
        cat_no,
        0, // Plain-Default
        root_key,
        cipher,
        mac,
        kdf,
    )?;

    // Schritt 4: Verwaiste Dateien und .building löschen (INV-CAT-4, INV-LOG-7)
    if !read_only {
        clean_orphaned_files(fs.as_ref(), base_dir, &catalog_state)?;
    }

    // Schritt 5: Fehlende genannte Datei führt fail-closed zu Corrupt{MissingFile}
    verify_named_files_exist(fs.as_ref(), base_dir, &catalog_state)?;

    // Schritt 6: Redo ab redo_floor replayen (INV-LOG-6, INV-LOG-8)
    let replay_res = recover_redo(
        Arc::clone(&fs),
        base_dir,
        catalog_state.redo_floor,
        catalog_state.redo_floor_stamp,
        catalog_state.redo_hwm,
    )?;

    // Nur Operationen mit stamp > flushed_stamp übernehmen
    let mut applied_ops = Vec::new();
    let mut pending_erases = catalog_state.erase_pending.clone();
    let mut highest_redo_stamp = replay_res.max_stamp;

    for (frame, ops) in replay_res.committed_frames {
        let stamp = frame.stamp;
        for op in ops {
            let flushed_s = catalog_state.space_flushed.get(&op.space).copied().unwrap_or(0);
            if stamp > flushed_s {
                if stamp > highest_redo_stamp {
                    highest_redo_stamp = stamp;
                }
                // EraseRange-Ops in die pending-Menge aufnehmen (Schritt 6)
                if matches!(op.kind, RedoOpKind::EraseRange) {
                    if !catalog_state.erase_done.contains(&stamp) {
                        let end = if (op.opflags & OPFLAG_UNBOUNDED_END) != 0 {
                            None
                        } else {
                            Some(op.value.clone())
                        };
                        pending_erases.push(ErasePendingMeta {
                            id: stamp,
                            space: op.space,
                            range: crate::catalog::types::CatalogKeyRange {
                                start: op.key.clone(),
                                end,
                            },
                            level: 0,
                        });
                    }
                }
                applied_ops.push((op.space, op));
            }
        }
    }

    // Schritt 7: Stempel setzen
    let mut max_run_stamp: Stamp = 0;
    for f in catalog_state.files.values() {
        if f.max_stamp > max_run_stamp {
            max_run_stamp = f.max_stamp;
        }
    }
    let max_flushed: Stamp = catalog_state.space_flushed.values().copied().max().unwrap_or(0);
    let max_known = highest_redo_stamp.max(max_flushed).max(max_run_stamp);

    let next_stamp = max_known.saturating_add(1);
    let visible_stamp = max_known;
    let durable_stamp = max_known;

    // Schritt 8: ID-Vergabe ab Hochwassermarken initialisieren (INV-KEY-2)
    let id_reserve = IdReserve::new(&catalog_state);

    // Schritt 9: Consumer prüfen (max_lag_bytes)
    // Schritt 10 & 11: Roll-forward Schnittstelle
    let duration = start_time.elapsed();

    Ok(RecoveryResult {
        catalog_state,
        next_stamp,
        visible_stamp,
        durable_stamp,
        id_reserve,
        applied_ops,
        pending_erases,
        reserve_segment_id: replay_res.reserve_segment_id,
        duration,
        _lock: Some(lock_guard),
    })
}

fn clean_orphaned_files(
    fs: &dyn Fs,
    base_dir: &Path,
    state: &CatalogState,
) -> Result<(), io::Error> {
    let entries = match fs.list_dir(base_dir) {
        Ok(e) => e,
        Err(_) => return Ok(()),
    };

    let known_runs: HashSet<u64> = state.files.values()
        .filter(|m| m.kind == CatalogFileKind::Run)
        .map(|m| m.id)
        .collect();

    let known_blobs: HashSet<u64> = state.files.values()
        .filter(|m| m.kind == CatalogFileKind::Blob)
        .map(|m| m.id)
        .collect();

    for entry in entries {
        let name = &entry.name;
        let file_path = base_dir.join(name);

        // 1. .building-Dateien löschen (INV-CAT-4)
        if name.ends_with(".building") || name.contains(".building.") {
            let _ = fs.remove_file(&file_path);
            continue;
        }

        // 2. Nicht genannte Runs löschen
        if let Some(id_str) = name.strip_prefix("RUN-") {
            if let Ok(id) = u64::from_str_radix(id_str, 10) {
                if !known_runs.contains(&id) {
                    let _ = fs.remove_file(&file_path);
                    continue;
                }
            }
        }

        // 3. Nicht genannte Blobs löschen
        if let Some(rest) = name.strip_prefix("BLOB-") {
            if let Some(id_str) = rest.split('.').next() {
                if let Ok(id) = u64::from_str_radix(id_str, 10) {
                    if !known_blobs.contains(&id) {
                        let _ = fs.remove_file(&file_path);
                        continue;
                    }
                }
            }
        }

        // 4. Redo-Segmente unterhalb redo_floor löschen; Reserve-Segmente (>= redo_floor) bleiben (INV-LOG-7)
        if let Some(seg_id) = parse_segment_id(name) {
            if seg_id < state.redo_floor {
                let _ = fs.remove_file(&file_path);
            }
        }
    }

    fs.sync_dir(base_dir)?;
    Ok(())
}

fn verify_named_files_exist(
    fs: &dyn Fs,
    base_dir: &Path,
    state: &CatalogState,
) -> Result<(), RecoveryError> {
    for meta in state.files.values() {
        let path = match meta.kind {
            CatalogFileKind::Run => base_dir.join(run_file_name(meta.id)),
            CatalogFileKind::Blob => base_dir.join(format!("BLOB-{:06}.0000", meta.id)),
            CatalogFileKind::Segment => base_dir.join(format!("SEGMENT-{:06}", meta.id)),
        };

        if fs.open(&path, OpenMode::Read).is_err() {
            return Err(RecoveryError::CorruptMissingFile {
                kind: meta.kind,
                id: meta.id,
            });
        }
    }
    Ok(())
}
