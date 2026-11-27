use super::ManifestEntry;
use contextra_types::{ContextraError, Result};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// Reduced durable state of the manifest.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ManifestState {
    live: BTreeMap<PathBuf, u64>,
    dead: BTreeSet<PathBuf>,
    rollbacks: BTreeSet<u64>,
    wal_hwm: Option<[u8; 32]>,
}

fn name(path: &Path) -> PathBuf {
    path.file_name()
        .map(PathBuf::from)
        .unwrap_or_else(|| path.to_path_buf())
}

impl ManifestState {
    pub(crate) fn from_entries(entries: &[ManifestEntry]) -> Result<Self> {
        let mut state = Self::default();
        for entry in entries {
            state.apply(entry)?;
        }
        Ok(state)
    }

    pub(crate) fn apply(&mut self, entry: &ManifestEntry) -> Result<()> {
        match entry {
            ManifestEntry::Add { path, max_tx } => {
                let key = name(path);
                self.dead.remove(&key);
                self.live.insert(key, *max_tx);
            }
            ManifestEntry::Remove { path } => {
                let key = name(path);
                self.live.remove(&key);
                self.dead.insert(key);
            }
            ManifestEntry::Replace {
                removed,
                added,
                added_max_tx,
                ..
            } => {
                let removed_names = removed.iter().map(|path| name(path));
                for key in removed_names {
                    if self.live.remove(&key).is_none() {
                        return Err(ContextraError::Storage(format!(
                            "Manifest Replace references non-live SSTable {:?}", key
                        )));
                    }
                    self.dead.insert(key);
                }
                let key = name(added);
                self.dead.remove(&key);
                self.live.insert(key, *added_max_tx);
            }
            ManifestEntry::RollbackComplete { target_tx } => {
                self.rollbacks.insert(*target_tx);
            }
            ManifestEntry::WalCheckpoint { hmac } => self.wal_hwm = Some(*hmac),
        }
        Ok(())
    }

    pub(crate) fn live(&self) -> &BTreeMap<PathBuf, u64> {
        &self.live
    }

    pub(crate) fn dead(&self) -> &BTreeSet<PathBuf> {
        &self.dead
    }

    pub(crate) fn wal_hwm(&self) -> Option<[u8; 32]> {
        self.wal_hwm
    }

    pub(crate) fn snapshot_entries(&self) -> Vec<ManifestEntry> {
        let mut snapshot = self
            .live
            .iter()
            .map(|(name, max_tx)| ManifestEntry::Add {
                path: name.clone(),
                max_tx: *max_tx,
            })
            .collect::<Vec<_>>();
        snapshot.extend(self.dead.iter().cloned().map(|path| ManifestEntry::Remove { path }));
        snapshot.extend(
            self.rollbacks
                .iter()
                .map(|target_tx| ManifestEntry::RollbackComplete {
                    target_tx: *target_tx,
                }),
        );
        if let Some(hmac) = self.wal_hwm {
            snapshot.push(ManifestEntry::WalCheckpoint { hmac });
        }
        snapshot
    }
}
