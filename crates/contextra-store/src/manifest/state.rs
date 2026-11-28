use super::entry::SstName;
use super::ManifestEntry;
use contextra_types::{ContextraError, Result};
use std::collections::{BTreeMap, BTreeSet};

/// Reduced durable state of the manifest.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ManifestState {
    live: BTreeMap<SstName, u64>,
    dead: BTreeSet<SstName>,
    rollbacks: BTreeSet<u64>,
    wal_hwm: Option<[u8; 32]>,
}

impl ManifestState {
    pub(crate) fn from_entries(entries: &[ManifestEntry]) -> Result<Self> {
        let mut state = Self::default();
        for entry in entries {
            state.apply(entry)?;
        }
        Ok(state)
    }

    /// Wie [`Self::apply`], prüft aber zusätzlich Einträge, die neu geschrieben werden (nicht
    /// beim Laden alter Manifeste): `Remove` muss eine dem Manifest bekannte Datei betreffen.
    /// Ein erneutes `Remove` einer bereits toten Datei bleibt idempotent erlaubt.
    pub(crate) fn apply_checked(&mut self, entry: &ManifestEntry) -> Result<()> {
        if let ManifestEntry::Remove { path } = entry {
            let key = SstName::try_from_path(path)?;
            if !self.live.contains_key(&key) && !self.dead.contains(&key) {
                return Err(ContextraError::Storage(format!(
                    "Manifest Remove references unknown SSTable {key:?}"
                )));
            }
        }
        self.apply(entry)
    }

    pub(crate) fn apply(&mut self, entry: &ManifestEntry) -> Result<()> {
        match entry {
            ManifestEntry::Add { path, max_tx } => {
                let key = SstName::try_from_path(path)?;
                self.dead.remove(&key);
                self.live.insert(key, *max_tx);
            }
            ManifestEntry::Remove { path } => {
                let key = SstName::try_from_path(path)?;
                self.live.remove(&key);
                self.dead.insert(key);
            }
            ManifestEntry::Replace {
                removed,
                added,
                added_max_tx,
                ..
            } => {
                let removed_names = removed
                    .iter()
                    .map(|path| SstName::try_from_path(path))
                    .collect::<Result<Vec<_>>>()?;
                for key in removed_names {
                    if self.live.remove(&key).is_none() {
                        return Err(ContextraError::Storage(format!(
                            "Manifest Replace references non-live SSTable {:?}",
                            key
                        )));
                    }
                    self.dead.insert(key);
                }
                let key = SstName::try_from_path(added)?;
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

    pub(crate) fn live(&self) -> &BTreeMap<SstName, u64> {
        &self.live
    }

    pub(crate) fn dead(&self) -> &BTreeSet<SstName> {
        &self.dead
    }

    pub(crate) fn wal_hwm(&self) -> Option<[u8; 32]> {
        self.wal_hwm
    }

    pub(crate) fn snapshot_frame(&self) -> Result<Vec<u8>> {
        super::entry::encode_snapshot_frame(&self.live, &self.dead, &self.rollbacks, self.wal_hwm)
    }
}
