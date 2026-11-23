//! In-memory KeyProvider for deterministic models and conformance exercises.

use contextra_storage_ports::traits::Entropy;
use contextra_storage_ports::{
    Error, Key32, KeyLookup, KeyProbe, KeyProvider, ClusterId,
};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use zeroize::Zeroizing;

pub struct MemKeyProvider {
    keys: Mutex<BTreeMap<ClusterId, ClusterKeys>>,
    entropy: Arc<dyn Entropy>,
}

#[derive(Default)]
struct ClusterKeys {
    epochs: BTreeMap<u32, Zeroizing<[u8; 32]>>,
    destroyed: bool,
}

impl MemKeyProvider {
    pub fn new(entropy: Arc<dyn Entropy>) -> Self {
        Self { keys: Mutex::new(BTreeMap::new()), entropy }
    }
}

impl KeyProvider for MemKeyProvider {
    fn create_cluster_keys(&self, clusters: &[ClusterId]) -> Result<Vec<u32>, Error> {
        // Generate outside the provider-state lock. Races converge at insertion.
        let mut candidates = Vec::with_capacity(clusters.len());
        for _ in clusters {
            let mut key = Zeroizing::new([0u8; 32]);
            self.entropy.fill(&mut *key)?;
            candidates.push(key);
        }
        let mut state = self.keys.lock().map_err(|_| Error::Poisoned { cause: "MemKeyProvider lock" })?;
        if clusters.iter().any(|cluster| state.get(cluster).is_some_and(|keys| keys.destroyed)) {
            return Err(Error::Shredded);
        }
        let mut epochs = Vec::with_capacity(clusters.len());
        for (cluster, candidate) in clusters.iter().zip(candidates) {
            let entry = state.entry(*cluster).or_default();
            if !entry.epochs.contains_key(&1) {
                entry.epochs.insert(1, Zeroizing::new(*candidate));
            }
            epochs.push(1);
        }
        Ok(epochs)
    }

    fn cluster_keys(&self, req: &[(ClusterId, u32)]) -> Result<Vec<KeyLookup>, Error> {
        let state = self.keys.lock().map_err(|_| Error::Poisoned { cause: "MemKeyProvider lock" })?;
        let mut result = Vec::with_capacity(req.len());
        for (cluster, epoch) in req {
            let lookup = match state.get(cluster) {
                Some(keys) if keys.destroyed => KeyLookup::Shredded,
                Some(keys) => match keys.epochs.get(epoch) {
                    Some(key) => KeyLookup::Present(Key32::from_array(**key)),
                    None => KeyLookup::Missing,
                },
                None => KeyLookup::Missing,
            };
            result.push(lookup);
        }
        Ok(result)
    }

    fn probe(&self, cluster: ClusterId, epoch: u32) -> Result<KeyProbe, Error> {
        let state = self.keys.lock().map_err(|_| Error::Poisoned { cause: "MemKeyProvider lock" })?;
        Ok(match state.get(&cluster) {
            Some(keys) if keys.destroyed => KeyProbe::Absent,
            Some(keys) if keys.epochs.contains_key(&epoch) => KeyProbe::Present,
            _ => KeyProbe::Disabled,
        })
    }

    fn destroy_many(&self, clusters: &[ClusterId]) -> Result<(), Error> {
        let mut state = self.keys.lock().map_err(|_| Error::Poisoned { cause: "MemKeyProvider lock" })?;
        for cluster in clusters {
            let entry = state.entry(*cluster).or_default();
            entry.epochs.clear();
            entry.destroyed = true;
        }
        Ok(())
    }
}

