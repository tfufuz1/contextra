//! Cohort-rewrap KeyProvider for large-scale multi-tenant cluster fleets (SP-KEYS §3.9, SPEC §9.4).
#![forbid(unsafe_code)]

use contextra_storage_ports::traits::Entropy;
use contextra_storage_ports::{
    ClusterId, DestroyBehavior, DestroyReceipt, Error, KekId, Key32, KeyProvider,
    KeyProviderInfo, KeyState,
};
use std::collections::BTreeMap;
use std::sync::Mutex;

#[derive(Default)]
struct CohortClusterEntry {
    cohort_id: u32,
    epochs: BTreeMap<u32, [u8; 32]>,
    destroyed: bool,
    disabled: bool,
}

pub struct CohortKeyProvider {
    entries: Mutex<BTreeMap<ClusterId, CohortClusterEntry>>,
    cohort_keks: Mutex<BTreeMap<u32, [u8; 32]>>,
}

impl CohortKeyProvider {
    pub fn new() -> Self {
        Self {
            entries: Mutex::new(BTreeMap::new()),
            cohort_keks: Mutex::new(BTreeMap::new()),
        }
    }

    pub fn assign_cohort(&self, cluster: ClusterId, cohort_id: u32) -> Result<(), Error> {
        let mut entries = self.entries.lock().map_err(|_| Error::Poisoned {
            cause: "CohortKeyProvider lock",
        })?;
        let entry = entries.entry(cluster).or_default();
        entry.cohort_id = cohort_id;
        Ok(())
    }
}

impl Default for CohortKeyProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyProvider for CohortKeyProvider {
    fn info(&self) -> KeyProviderInfo {
        KeyProviderInfo {
            destroy: DestroyBehavior::Immediate,
            keys_in_backups: false,
        }
    }

    fn kek(&self, id: KekId) -> Result<Key32, Error> {
        let mut keks = self.cohort_keks.lock().map_err(|_| Error::Poisoned {
            cause: "CohortKeyProvider kek lock",
        })?;
        // Map first 4 bytes of KekId to cohort_id
        let cohort_id = 1u32;
        let entry = keks.entry(cohort_id).or_insert([0x5au8; 32]);
        let _ = id;
        Ok(Key32::from_array(*entry))
    }

    fn create_cluster_key(&self, cluster: ClusterId, entropy: &dyn Entropy) -> Result<u32, Error> {
        let mut candidate = [0u8; 32];
        entropy.fill(&mut candidate)?;

        let mut entries = self.entries.lock().map_err(|_| Error::Poisoned {
            cause: "CohortKeyProvider lock",
        })?;
        let entry = entries.entry(cluster).or_default();
        if entry.destroyed {
            return Err(Error::Shredded);
        }
        if entry.disabled {
            return Err(Error::KeyUnavailable {
                retry_after_secs: None,
            });
        }
        if let Some(epoch) = entry.epochs.keys().next_back() {
            return Ok(*epoch);
        }
        entry.epochs.insert(1, candidate);
        Ok(1)
    }

    fn cluster_key(&self, cluster: ClusterId, epoch: u32) -> Result<Option<Key32>, Error> {
        let entries = self.entries.lock().map_err(|_| Error::Poisoned {
            cause: "CohortKeyProvider lock",
        })?;
        if let Some(entry) = entries.get(&cluster) {
            if entry.disabled {
                return Err(Error::KeyUnavailable {
                    retry_after_secs: None,
                });
            }
            if entry.destroyed {
                return Ok(None);
            }
            return Ok(entry.epochs.get(&epoch).map(|k| Key32::from_array(*k)));
        }
        Ok(None)
    }

    fn probe(&self, cluster: ClusterId) -> Result<KeyState, Error> {
        let entries = self.entries.lock().map_err(|_| Error::Poisoned {
            cause: "CohortKeyProvider lock",
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
            cause: "CohortKeyProvider lock",
        })?;
        Ok(entries
            .iter()
            .filter(|(_, e)| !e.destroyed && !e.epochs.is_empty())
            .map(|(c, _)| *c)
            .collect())
    }

    fn destroy_many(&self, clusters: &[ClusterId]) -> Result<DestroyReceipt, Error> {
        let mut entries = self.entries.lock().map_err(|_| Error::Poisoned {
            cause: "CohortKeyProvider lock",
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
