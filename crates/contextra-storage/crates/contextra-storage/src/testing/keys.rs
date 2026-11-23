//! In-memory KeyProvider for deterministic models and conformance exercises.

use contextra_storage_ports::traits::Entropy;
use contextra_storage_ports::{BatchId, ClusterId, Error, Key32, KeyCreateFailure, KeyCreateResult, KeyLookup, KeyProbe, KeyProvider, MAX_KEYS_PER_BATCH};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use zeroize::Zeroizing;

pub struct MemKeyProvider {
    keys: Mutex<BTreeMap<ClusterId, ClusterKeys>>,
    batches: Mutex<BTreeMap<BatchId, (Vec<ClusterId>, Vec<KeyCreateResult>)>>,
    entropy: Arc<dyn Entropy>,
}

#[derive(Default)]
struct ClusterKeys {
    epochs: BTreeMap<u32, Zeroizing<[u8; 32]>>,
    destroyed: bool,
}

impl MemKeyProvider {
    pub fn new(entropy: Arc<dyn Entropy>) -> Self {
        Self { keys: Mutex::new(BTreeMap::new()), batches: Mutex::new(BTreeMap::new()), entropy }
    }
}

impl KeyProvider for MemKeyProvider {
    fn create_cluster_keys(&self, request_id: BatchId, clusters: &[ClusterId]) -> Result<Vec<KeyCreateResult>, Error> {
        if clusters.len() > MAX_KEYS_PER_BATCH { return Err(Error::Limit { limit: "key create batch <= 256" }); }
        let mut unique = std::collections::BTreeSet::new();
        if clusters.iter().any(|cluster| !unique.insert(*cluster)) {
            return Err(Error::Limit { limit: "duplicate cluster in key create batch" });
        }
        {
            let batches = self.batches.lock().map_err(|_| Error::Poisoned { cause: "MemKeyProvider batch lock" })?;
            if let Some((previous, result)) = batches.get(&request_id) {
                return if previous == clusters { Ok(result.clone()) } else { Err(Error::IdemConflict) };
            }
        }
        // Generate outside the provider-state lock. Races converge at insertion.
        let mut candidates = Vec::with_capacity(clusters.len());
        for _ in clusters {
            let mut key = Zeroizing::new([0u8; 32]);
            self.entropy.fill(&mut *key)?;
            candidates.push(key);
        }
        let mut batches = self.batches.lock().map_err(|_| Error::Poisoned { cause: "MemKeyProvider batch lock" })?;
        if let Some((previous, result)) = batches.get(&request_id) {
            return if previous == clusters { Ok(result.clone()) } else { Err(Error::IdemConflict) };
        }
        let mut state = self.keys.lock().map_err(|_| Error::Poisoned { cause: "MemKeyProvider lock" })?;
        let mut results = Vec::with_capacity(clusters.len());
        for (cluster, candidate) in clusters.iter().zip(candidates) {
            let entry = state.entry(*cluster).or_default();
            if entry.destroyed {
                results.push(KeyCreateResult::Failed { cluster: *cluster, error: KeyCreateFailure::Shredded });
            } else if entry.epochs.contains_key(&1) {
                results.push(KeyCreateResult::Created { cluster: *cluster, epoch: 1 });
            } else {
                entry.epochs.insert(1, Zeroizing::new(*candidate));
                results.push(KeyCreateResult::Created { cluster: *cluster, epoch: 1 });
            }
        }
        batches.insert(request_id, (clusters.to_vec(), results.clone()));
        Ok(results)
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

