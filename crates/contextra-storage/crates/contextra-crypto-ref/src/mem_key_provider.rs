//! Deterministic in-memory KeyProvider used by conformance and fault tests.

use contextra_storage_ports::traits::Entropy;
use contextra_storage_ports::{
    ClusterId, DestroyBehavior, DestroyReceipt, Error, KekId, Key32, KeyProvider,
    KeyProviderInfo, KeyState,
};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use zeroize::Zeroizing;

pub struct MemKeyProvider {
    keys: Mutex<BTreeMap<ClusterId, ClusterKeys>>,
    keks: Mutex<BTreeMap<KekId, Zeroizing<[u8; 32]>>>,
    entropy: Arc<dyn Entropy>,
}

#[derive(Default)]
struct ClusterKeys {
    epochs: BTreeMap<u32, Zeroizing<[u8; 32]>>,
    destroyed: bool,
    disabled: bool,
}

impl MemKeyProvider {
    pub fn new(entropy: Arc<dyn Entropy>) -> Self {
        Self { keys: Mutex::new(BTreeMap::new()), keks: Mutex::new(BTreeMap::new()), entropy }
    }

    /// Changes provider availability for deterministic failure scenarios.
    pub fn set_disabled(&self, cluster: ClusterId, disabled: bool) -> Result<(), Error> {
        let mut keys = self.keys.lock().map_err(|_| Error::Poisoned { cause: "MemKeyProvider lock" })?;
        keys.entry(cluster).or_default().disabled = disabled;
        Ok(())
    }

    /// Installs an explicit epoch for provider conformance scenarios.
    pub fn insert_epoch_for_test(&self, cluster: ClusterId, epoch: u32, key: [u8; 32]) -> Result<(), Error> {
        if epoch == 0 { return Err(Error::Limit { limit: "cluster key epoch must be nonzero" }); }
        let mut keys = self.keys.lock().map_err(|_| Error::Poisoned { cause: "MemKeyProvider lock" })?;
        let entry = keys.entry(cluster).or_default();
        if entry.destroyed { return Err(Error::Shredded); }
        entry.epochs.insert(epoch, Zeroizing::new(key));
        Ok(())
    }
}

impl KeyProvider for MemKeyProvider {
    fn info(&self) -> KeyProviderInfo {
        KeyProviderInfo { destroy: DestroyBehavior::Immediate, keys_in_backups: false }
    }

    fn kek(&self, id: KekId) -> Result<Key32, Error> {
        let mut candidate = Zeroizing::new([0u8; 32]);
        self.entropy.fill(&mut *candidate)?;
        let mut keks = self.keks.lock().map_err(|_| Error::Poisoned { cause: "MemKeyProvider KEK lock" })?;
        let key = keks.entry(id).or_insert(candidate);
        Ok(Key32::from_array(**key))
    }

    fn create_cluster_key(&self, cluster: ClusterId, entropy: &dyn Entropy) -> Result<u32, Error> {
        {
            let keys = self.keys.lock().map_err(|_| Error::Poisoned { cause: "MemKeyProvider lock" })?;
            if let Some(entry) = keys.get(&cluster) {
                if entry.destroyed { return Err(Error::Shredded); }
                if entry.disabled { return Err(Error::KeyUnavailable { retry_after_secs: None }); }
                if let Some(epoch) = entry.epochs.keys().next_back() { return Ok(*epoch); }
            }
        }
        let mut candidate = Zeroizing::new([0u8; 32]);
        entropy.fill(&mut *candidate)?;
        let mut keys = self.keys.lock().map_err(|_| Error::Poisoned { cause: "MemKeyProvider lock" })?;
        let entry = keys.entry(cluster).or_default();
        if entry.destroyed { return Err(Error::Shredded); }
        if entry.disabled { return Err(Error::KeyUnavailable { retry_after_secs: None }); }
        if let Some(epoch) = entry.epochs.keys().next_back() { return Ok(*epoch); }
        entry.epochs.insert(1, Zeroizing::new(*candidate));
        Ok(1)
    }

    fn cluster_key(&self, cluster: ClusterId, epoch: u32) -> Result<Option<Key32>, Error> {
        let keys = self.keys.lock().map_err(|_| Error::Poisoned { cause: "MemKeyProvider lock" })?;
        if keys.get(&cluster).is_some_and(|entry| entry.disabled) {
            return Err(Error::KeyUnavailable { retry_after_secs: None });
        }
        Ok(keys.get(&cluster).and_then(|entry| entry.epochs.get(&epoch))
            .map(|key| Key32::from_array(**key)))
    }

    fn probe(&self, cluster: ClusterId) -> Result<KeyState, Error> {
        let keys = self.keys.lock().map_err(|_| Error::Poisoned { cause: "MemKeyProvider lock" })?;
        Ok(match keys.get(&cluster) {
            Some(entry) if entry.destroyed => KeyState::Absent,
            Some(entry) if entry.disabled => KeyState::Disabled,
            Some(entry) if !entry.epochs.is_empty() => KeyState::Present,
            _ => KeyState::Absent,
        })
    }

    fn list(&self) -> Result<Vec<ClusterId>, Error> {
        let keys = self.keys.lock().map_err(|_| Error::Poisoned { cause: "MemKeyProvider lock" })?;
        Ok(keys.iter().filter_map(|(cluster, entry)| {
            (!entry.destroyed && !entry.epochs.is_empty()).then_some(*cluster)
        }).collect())
    }

    fn destroy_many(&self, clusters: &[ClusterId]) -> Result<DestroyReceipt, Error> {
        let mut keys = self.keys.lock().map_err(|_| Error::Poisoned { cause: "MemKeyProvider lock" })?;
        for cluster in clusters {
            let entry = keys.entry(*cluster).or_default();
            entry.epochs.clear();
            entry.destroyed = true;
            entry.disabled = false;
        }
        Ok(DestroyReceipt { requested_clusters: clusters.len(), absent_clusters: clusters.len() })
    }
}
