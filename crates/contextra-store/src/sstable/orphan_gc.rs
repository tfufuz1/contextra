// FILE-CONTEXT
// STAND: 2026-10-08 (T-2026-9104)
// ZWECK: Orphan- und Temp-Datei Bereinigung (D4.0) unter Verwendung von contextra_durable_fs::durable_remove.
// INVARIANTEN: Unknown-Dateien werden NIEMALS gelöscht; Laufende Operationen werden geschützt.

use super::witness_inventory::SstInventory;
use super::witness_types::WitnessReadError;
use contextra_durable_fs::durable_remove;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

/// Plan for GC removal of orphaned SSTables and temporary files.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OrphanPlan {
    pub files_to_remove: Vec<PathBuf>,
}

/// Receipt detailing the result of an orphan GC execution.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OrphanGcReceipt {
    pub files_removed: u64,
    pub bytes_freed: u64,
}

/// Plans orphan and temp file removal, excluding any files associated with running operations.
/// UNKNOWN files are NEVER included in the removal plan!
pub fn plan_orphan_removal(inventory: &SstInventory, running_ops: &[PathBuf]) -> OrphanPlan {
    let running_set: HashSet<PathBuf> = running_ops
        .iter()
        .map(|p| normalize_path(p))
        .collect();

    let mut files_to_remove = Vec::new();

    // Check Orphans
    for file_path in &inventory.orphan {
        let norm = normalize_path(file_path);
        if !running_set.contains(&norm) {
            files_to_remove.push(file_path.clone());
        }
    }

    // Check Temps
    for file_path in &inventory.temp {
        let norm = normalize_path(file_path);
        if !running_set.contains(&norm) {
            files_to_remove.push(file_path.clone());
        }
    }

    // Note: inventory.unknown is deliberately IGNORED! Unknown files are NEVER removed.

    OrphanPlan { files_to_remove }
}

/// Executes orphan removal using durable_remove for APM-1 durability compliance.
pub fn remove_orphans(plan: &OrphanPlan) -> Result<OrphanGcReceipt, WitnessReadError> {
    let mut receipt = OrphanGcReceipt::default();

    for file_path in &plan.files_to_remove {
        if !file_path.exists() {
            continue;
        }

        let file_bytes = match fs::metadata(file_path) {
            Ok(meta) => meta.len(),
            Err(_) => 0,
        };

        durable_remove(file_path).map_err(|e| {
            WitnessReadError::Io(format!("Failed to durably remove orphan file: {e}"))
        })?;

        receipt.files_removed += 1;
        receipt.bytes_freed += file_bytes;
    }

    Ok(receipt)
}

fn normalize_path(path: &Path) -> PathBuf {
    match path.canonicalize() {
        Ok(p) => p,
        Err(_) => path.to_path_buf(),
    }
}
