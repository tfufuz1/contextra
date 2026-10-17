// FILE-CONTEXT
// STAND: 2026-10-08 (T-2026-9104)
// ZWECK: Dateiinventur und Klassifizierung für den SSTable Witness Scanner.
// INVARIANTEN: Fail-Closed bei Unknown (unclassified_files > 0 entzieht Claim CT-S2V/S2K).

use super::witness_types::{SstScanInputs, WitnessReadError};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

/// Inventory of all files discovered in store data directories, classified by role.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SstInventory {
    pub live: Vec<PathBuf>,
    pub obsolete_pending: Vec<PathBuf>,
    pub orphan: Vec<PathBuf>,
    pub temp: Vec<PathBuf>,
    pub old_manifest: Vec<PathBuf>,
    pub unknown: Vec<PathBuf>,
}

impl SstInventory {
    /// Builds inventory by scanning `inputs.data_dirs` and comparing against `live_files` and `pending_delete_files`.
    pub fn build(inputs: &SstScanInputs) -> Result<Self, WitnessReadError> {
        let mut inventory = Self::default();

        let live_set: HashSet<PathBuf> = inputs
            .live_files
            .iter()
            .map(|p| normalize_path(p))
            .collect();
        let pending_set: HashSet<PathBuf> = inputs
            .pending_delete_files
            .iter()
            .map(|p| normalize_path(p))
            .collect();

        // Include all explicitly passed live_files as Live
        for path in &inputs.live_files {
            inventory.live.push(path.clone());
        }

        // Include all explicitly passed pending_delete_files as ObsoletePending
        for path in &inputs.pending_delete_files {
            if !live_set.contains(&normalize_path(path)) {
                inventory.obsolete_pending.push(path.clone());
            }
        }

        for data_dir in &inputs.data_dirs {
            if !data_dir.exists() {
                continue;
            }

            let entries = fs::read_dir(data_dir)
                .map_err(|e| WitnessReadError::Io(format!("Failed to read data directory: {e}")))?;

            for entry in entries {
                let entry = entry.map_err(|e| {
                    WitnessReadError::Io(format!("Failed to read directory entry: {e}"))
                })?;

                let path = entry.path();
                if path.is_dir() {
                    continue;
                }

                let norm_path = normalize_path(&path);
                if live_set.contains(&norm_path) || pending_set.contains(&norm_path) {
                    continue; // Already classified above
                }

                let file_name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();

                if file_name.ends_with(".sst") {
                    inventory.orphan.push(path);
                } else if file_name.ends_with(".tmp")
                    || file_name.contains(".tmp.")
                    || file_name.ends_with(".bak")
                {
                    inventory.temp.push(path);
                } else if file_name == "MANIFEST.old"
                    || file_name == "MANIFEST.bak"
                    || file_name.starts_with("MANIFEST.tmp")
                    || file_name.starts_with("MANIFEST-")
                {
                    inventory.old_manifest.push(path);
                } else if file_name.starts_with('.')
                    || file_name == "MANIFEST"
                    || file_name == "SALT"
                    || file_name == "LOCK"
                    || file_name.contains("ledger")
                    || file_name == "purge.intent"
                    || file_name == "pending_delete.journal"
                    || file_name == "integrity.key"
                    || file_name.ends_with(".wal")
                    || file_name.ends_with(".log")
                    || file_name.starts_with("wal")
                    || file_name.ends_with(".wal_integrity_key")
                    || file_name.ends_with(".rekeyed")
                    || file_name.ends_with(".key")
                    || file_name.contains(".uuid")
                    || file_name.contains(".sealed")
                    || file_name.contains(".archived")
                {
                    // Active system metadata, WAL, or ledger files - valid system files, not orphans or unknown
                    continue;
                } else {
                    inventory.unknown.push(path);
                }
            }
        }

        Ok(inventory)
    }
}

fn normalize_path(path: &Path) -> PathBuf {
    match path.canonicalize() {
        Ok(p) => p,
        Err(_) => path.to_path_buf(),
    }
}
