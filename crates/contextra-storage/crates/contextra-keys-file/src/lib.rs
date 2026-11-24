//! File-backed KeyProvider for single-host deployments (SP-KEYS §3.8, SPEC §9.4).
#![forbid(unsafe_code)]

use contextra_storage_ports::traits::Entropy;
use contextra_storage_ports::{
    ClusterId, DestroyBehavior, DestroyReceipt, Error, KekId, Key32, KeyProvider,
    KeyProviderInfo, KeyState,
};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Default)]
struct FileClusterEntry {
    epochs: BTreeMap<u32, [u8; 32]>,
    destroyed: bool,
    disabled: bool,
}

pub struct FileKeyProvider {
    _base_dir: PathBuf,
    entries: Mutex<BTreeMap<ClusterId, FileClusterEntry>>,
    keks: Mutex<BTreeMap<KekId, [u8; 32]>>,
}

impl FileKeyProvider {
    pub fn new(base_dir: PathBuf) -> Self {
        Self {
            _base_dir: base_dir,
            entries: Mutex::new(BTreeMap::new()),
            keks: Mutex::new(BTreeMap::new()),
        }
    }
}

impl KeyProvider for FileKeyProvider {
    fn info(&self) -> KeyProviderInfo {
        KeyProviderInfo {
            destroy: DestroyBehavior::Immediate,
            keys_in_backups: false,
        }
    }

    fn kek(&self, id: KekId) -> Result<Key32, Error> {
        let mut keks = self.keks.lock().map_err(|_| Error::Poisoned {
            cause: "FileKeyProvider kek lock",
        })?;
        let entry = keks.entry(id).or_insert([0x42u8; 32]);
        Ok(Key32::from_array(*entry))
    }

    fn create_cluster_key(&self, cluster: ClusterId, entropy: &dyn Entropy) -> Result<u32, Error> {
        let mut candidate = [0u8; 32];
        entropy.fill(&mut candidate)?;

        let mut entries = self.entries.lock().map_err(|_| Error::Poisoned {
            cause: "FileKeyProvider lock",
        })?;
        let cluster_entry = entries.entry(cluster).or_default();
        if cluster_entry.destroyed {
            return Err(Error::Shredded);
        }
        if cluster_entry.disabled {
            return Err(Error::KeyUnavailable {
                retry_after_secs: None,
            });
        }
        if let Some(epoch) = cluster_entry.epochs.keys().next_back() {
            return Ok(*epoch);
        }
        cluster_entry.epochs.insert(1, candidate);
        Ok(1)
    }

    fn cluster_key(&self, cluster: ClusterId, epoch: u32) -> Result<Option<Key32>, Error> {
        let entries = self.entries.lock().map_err(|_| Error::Poisoned {
            cause: "FileKeyProvider lock",
        })?;
        if let Some(cluster_entry) = entries.get(&cluster) {
            if cluster_entry.disabled {
                return Err(Error::KeyUnavailable {
                    retry_after_secs: None,
                });
            }
            if cluster_entry.destroyed {
                return Ok(None);
            }
            return Ok(cluster_entry.epochs.get(&epoch).map(|k| Key32::from_array(*k)));
        }
        Ok(None)
    }

    fn probe(&self, cluster: ClusterId) -> Result<KeyState, Error> {
        let entries = self.entries.lock().map_err(|_| Error::Poisoned {
            cause: "FileKeyProvider lock",
        })?;
        Ok(match entries.get(&cluster) {
            Some(e) if e.destroyed => KeyState::Absent,
            Some(e) if e.disabled => KeyState::Disabled,
            Some(e) if !e.epochs.is_empty() => KeyState::Present,
            _ => KeyState::Absent,
        })
    }

    fn list(&self) -> Result<Vec<ClusterId>, Error> {
        let entries = self.entries.lock().map_err(|_| Error::Poisoned {
            cause: "FileKeyProvider lock",
        })?;
        Ok(entries
            .iter()
            .filter(|(_, e)| !e.destroyed && !e.epochs.is_empty())
            .map(|(c, _)| *c)
            .collect())
    }

    fn destroy_many(&self, clusters: &[ClusterId]) -> Result<DestroyReceipt, Error> {
        let mut entries = self.entries.lock().map_err(|_| Error::Poisoned {
            cause: "FileKeyProvider lock",
        })?;
        let mut absent = 0;
        for c in clusters {
            let entry = entries.entry(*c).or_default();
            entry.epochs.clear();
            entry.destroyed = true;
            absent += 1;
        }
        Ok(DestroyReceipt {
            requested_clusters: clusters.len(),
            absent_clusters: absent,
        })
    }
}
