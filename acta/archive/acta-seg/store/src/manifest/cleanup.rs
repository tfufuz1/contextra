//! Manifest Validierung, Rekonstruktion und Bereinigung toter SSTables (Ring 1)
//!
//! Berechnet die Menge der gültigen und toten SSTables ohne Auferstehungsrisiko
//! und extrahiert den WAL High-Water-Mark Checkpoint.
//! Verwendet deterministische BTreeMap / BTreeSet, um jegliche Unschärfe durch
//! unvorhersehbare Hash-Iterationen in Kernpfaden vollständig auszuschließen.

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::{Path, PathBuf};

use super::entry::ManifestEntry;
use super::handle::Manifest;

impl Manifest {
    /// Reconstructs the ordered list of currently valid SSTable file names (or path components)
    /// and their rank (shadowing order: smaller rank = older) from a sequence of manifest entries.
    pub fn reconstruct_valid_sstables(entries: &[ManifestEntry]) -> Vec<(PathBuf, u64)> {
        let mut valid_map = BTreeMap::new();
        let mut next_rank = 0u64;

        for entry in entries {
            match entry {
                ManifestEntry::Add { path, .. } => {
                    let key = path
                        .file_name()
                        .map(PathBuf::from)
                        .unwrap_or_else(|| path.clone());
                    valid_map.insert(key, next_rank);
                    next_rank += 1;
                }
                ManifestEntry::Remove { path } => {
                    let key = path
                        .file_name()
                        .map(PathBuf::from)
                        .unwrap_or_else(|| path.clone());
                    valid_map.remove(&key);
                }
                ManifestEntry::RollbackComplete { .. } | ManifestEntry::WalCheckpoint { .. } => {}
                ManifestEntry::Replace { removed, added, .. } => {
                    for p in removed {
                        let key = p
                            .file_name()
                            .map(PathBuf::from)
                            .unwrap_or_else(|| p.clone());
                        valid_map.remove(&key);
                    }
                    let key = added
                        .file_name()
                        .map(PathBuf::from)
                        .unwrap_or_else(|| added.clone());
                    valid_map.insert(key, next_rank);
                    next_rank += 1;
                }
            }
        }

        let mut sorted: Vec<(PathBuf, u64)> = valid_map.into_iter().collect();
        sorted.sort_by_key(|(_, rank)| *rank);
        sorted
    }

    /// Reconstructs the set of SSTable file names (or path components) that are proven dead
    /// (explicitly removed via `Remove` or `Replace.removed`) and are not in the valid set.
    pub fn reconstruct_dead_sstables(entries: &[ManifestEntry]) -> HashSet<PathBuf> {
        let valid = Self::reconstruct_valid_sstables(entries);
        let valid_set: BTreeSet<PathBuf> = valid.into_iter().map(|(p, _)| p).collect();
        let mut dead_set = BTreeSet::new();

        for entry in entries {
            match entry {
                ManifestEntry::Remove { path } => {
                    let key = path
                        .file_name()
                        .map(PathBuf::from)
                        .unwrap_or_else(|| path.clone());
                    if !valid_set.contains(&key) {
                        dead_set.insert(key);
                    }
                }
                ManifestEntry::Replace { removed, .. } => {
                    for p in removed {
                        let key = p
                            .file_name()
                            .map(PathBuf::from)
                            .unwrap_or_else(|| p.clone());
                        if !valid_set.contains(&key) {
                            dead_set.insert(key);
                        }
                    }
                }
                ManifestEntry::Add { .. }
                | ManifestEntry::RollbackComplete { .. }
                | ManifestEntry::WalCheckpoint { .. } => {}
            }
        }

        dead_set.into_iter().collect()
    }

    /// Extracts the most recent WAL high-water-mark HMAC from a list of manifest entries.
    pub fn extract_high_water_mark(entries: &[ManifestEntry]) -> Option<[u8; 32]> {
        for entry in entries.iter().rev() {
            if let ManifestEntry::WalCheckpoint { hmac } = entry {
                return Some(*hmac);
            }
        }
        None
    }
}
