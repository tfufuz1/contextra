//! # Compaction Installation (install)
//!
//! SPEC §12.2, §16.5 (INV-CLAIM-1):
//! Ein atomarer Catalog-Edit mit add_files, remove_files, blob_live_delta, reserve.
//! Anschließende Veröffentlichung der neuen Version, Markierung der Eingaben als
//! obsolete und Übergabe an das Graveyard.
//! Keine Integer-Casts (M-02 = 0), keine Panics (M-08 = 0).

use crate::catalog::durable::sync_existing;
use crate::catalog::edit::CatalogEdit;
use crate::catalog::log::CatalogLog;
use crate::catalog::reserve::IdReserve;
use crate::catalog::state::{apply_edit, CatalogState};
use crate::catalog::types::CatalogFileKind;
use crate::engine::compaction::exec::{run_file_name, CompactionExecutionResult};
use contextra_storage_ports::{Cipher, File, Fs, Kdf, Mac};
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

/// Fehler bei der Installation einer Kompaktion.
#[derive(Debug)]
pub enum CompactionInstallError {
    /// Kompaktion wurde durch einen dringenden (urgent) Auftrag abgebrochen.
    Cancelled,
    /// E/A-Fehler.
    Io(io::Error),
    /// Katalogfehler beim Commit.
    Catalog(crate::catalog::types::CatalogError),
}

impl fmt::Display for CompactionInstallError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cancelled => write!(f, "INV-CLAIM-1: Kompaktion wegen Urgent-Konflikt abgebrochen"),
            Self::Io(e) => write!(f, "E/A-Fehler bei Kompaktions-Installation: {e}"),
            Self::Catalog(e) => write!(f, "Katalog-Fehler bei Kompaktions-Installation: {e:?}"),
        }
    }
}

impl std::error::Error for CompactionInstallError {}

impl From<io::Error> for CompactionInstallError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<crate::catalog::types::CatalogError> for CompactionInstallError {
    fn from(e: crate::catalog::types::CatalogError) -> Self {
        Self::Catalog(e)
    }
}

/// Führt die Installation der Kompaktionsergebnisse atomar durch.
pub fn install_compaction(
    fs: &dyn Fs,
    base_dir: &Path,
    catalog_file: &dyn File,
    catalog_log: &mut CatalogLog,
    current_state: &CatalogState,
    exec_res: CompactionExecutionResult,
    id_reserve: &IdReserve,
    cancel_flag: &AtomicBool,
    graveyard: &mut Vec<PathBuf>,
    root_key: Option<&[u8; 32]>,
    cipher: Option<&dyn Cipher>,
    mac: Option<&dyn Mac>,
    kdf: Option<&dyn Kdf>,
) -> Result<CatalogState, CompactionInstallError> {
    // 1. Prüfe Abbruchsignal (cancel_flag) vor Beginn der Installation (INV-CLAIM-1)
    if cancel_flag.load(Ordering::Acquire) {
        // Räume erzeugte .building-Dateien auf
        for (_, p) in &exec_res.created_runs {
            let _ = fs.remove_file(p);
        }
        for (_, p) in &exec_res.created_blobs {
            let _ = fs.remove_file(p);
        }
        return Err(CompactionInstallError::Cancelled);
    }

    // 2. Benenne alle .building-Dateien in ihre endgültigen Namen um
    let mut add_files = Vec::new();

    for (meta, temp_path) in exec_res.created_runs {
        let final_path = base_dir.join(run_file_name(meta.id));
        fs.rename(&temp_path, &final_path)?;
        add_files.push(meta);
    }

    for (meta, temp_path) in exec_res.created_blobs {
        let final_path = base_dir.join(format!("BLOB-{:06}.0000", meta.id));
        fs.rename(&temp_path, &final_path)?;
        add_files.push(meta);
    }

    // sync_dir auf das Elternverzeichnis
    fs.sync_dir(base_dir)?;

    // 3. Baue atomaren CatalogEdit
    let mut edit = CatalogEdit::default();
    edit.add_files = add_files;
    edit.remove_files = exec_res.removed_files.clone();

    for (b_no, delta) in exec_res.blob_live_deltas {
        edit.blob_live_delta.push((b_no, delta));
    }

    edit.reserve = Some((
        id_reserve.run_id_hwm,
        id_reserve.blob_no_hwm,
        id_reserve.segment_id_hwm,
        id_reserve.tmp_id_hwm,
        current_state.keys_generation_min,
    ));

    // 4. Prüfe Abbruchsignal erneut unmittelbar vor dem Katalog-Commit
    if cancel_flag.load(Ordering::Acquire) {
        return Err(CompactionInstallError::Cancelled);
    }

    // 5. Commit über Synced (M-5)
    let synced_edit = sync_existing(fs, catalog_file, base_dir, edit.clone())?;
    catalog_log.commit(catalog_file, synced_edit, root_key, cipher, mac, kdf)?;

    // 6. Neuen Katalogzustand ableiten (atomar)
    let new_state = apply_edit(current_state, &edit)?;

    // 7. Eingabedateien als obsolete kennzeichnen und an Graveyard übergeben
    for (kind, id) in exec_res.removed_files {
        match kind {
            CatalogFileKind::Run => {
                let p = base_dir.join(run_file_name(id));
                graveyard.push(p);
            }
            CatalogFileKind::Blob => {
                let p = base_dir.join(format!("BLOB-{id:06}.0000"));
                graveyard.push(p);
            }
            CatalogFileKind::Segment => {
                let p = base_dir.join(format!("SEGMENT-{id:06}"));
                graveyard.push(p);
            }
        }
    }

    Ok(new_state)
}
