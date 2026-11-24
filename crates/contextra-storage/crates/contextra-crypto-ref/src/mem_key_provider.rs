//! Deterministic in-memory KeyProvider used by conformance and fault tests (SPEC §9.4).

use contextra_storage_ports::traits::Entropy;
use contextra_storage_ports::{
    BatchId, ClusterId, DestroyBehavior, DestroyReceipt, Error, KekId, Key32, KeyAssurance,
    KeyCreateResult, KeyListEntry, KeyLookup, KeyProvider, KeyProviderInfo, KeyState,
    KeysInBackups, KEY_PROVIDER_BATCH_MAX,
};
use std::collections::{BTreeMap, HashMap};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use zeroize::Zeroizing;

#[derive(Default)]
struct ClusterKeys {
    epochs: BTreeMap<u32, Zeroizing<[u8; 32]>>,
    created_utc: u64,
    destroyed: bool,
    disabled: bool,
}

#[derive(Clone)]
enum TombState {
    Open(Zeroizing<[u8; 32]>),
    Retired,
}

pub struct MemKeyProvider {
    keys: Mutex<BTreeMap<ClusterId, ClusterKeys>>,
    keks: Mutex<BTreeMap<KekId, Zeroizing<[u8; 32]>>>,
    batches: Mutex<HashMap<BatchId, Vec<(ClusterId, u32)>>>,
    tomb_epochs: Mutex<BTreeMap<u32, TombState>>,
    entropy: Arc<dyn Entropy>,
    info: KeyProviderInfo,
    utc_secs: AtomicU64,
    offline: std::sync::atomic::AtomicBool,
}

impl MemKeyProvider {
    pub fn new(entropy: Arc<dyn Entropy>) -> Self {
        Self::with_info(
            entropy,
            KeyProviderInfo {
                destroy: DestroyBehavior::Immediate,
                keys_in_backups: KeysInBackups::No,
                assurance: KeyAssurance::Kms,
            },
        )
    }

    pub fn with_info(entropy: Arc<dyn Entropy>, info: KeyProviderInfo) -> Self {
        Self {
            keys: Mutex::new(BTreeMap::new()),
            keks: Mutex::new(BTreeMap::new()),
            batches: Mutex::new(HashMap::new()),
            tomb_epochs: Mutex::new(BTreeMap::new()),
            entropy,
            info,
            utc_secs: AtomicU64::new(1_700_000_000),
            offline: std::sync::atomic::AtomicBool::new(false),
        }
    }

    pub fn set_utc_secs(&self, utc_secs: u64) {
        self.utc_secs.store(utc_secs, Ordering::Release);
    }

    pub fn set_offline(&self, offline: bool) {
        self.offline.store(offline, Ordering::Release);
    }

    /// Changes provider availability for deterministic failure scenarios.
    pub fn set_disabled(&self, cluster: ClusterId, disabled: bool) -> Result<(), Error> {
        let mut keys = self.keys.lock().map_err(|_| Error::Poisoned { cause: "MemKeyProvider lock" })?;
        keys.entry(cluster).or_default().disabled = disabled;
        Ok(())
    }

    /// Installs an explicit epoch for provider conformance scenarios (e.g. key rotation).
    pub fn insert_epoch_for_test(&self, cluster: ClusterId, epoch: u32, key: [u8; 32]) -> Result<(), Error> {
        if epoch == 0 {
            return Err(Error::Limit { limit: "cluster key epoch must be nonzero" });
        }
        let mut keys = self.keys.lock().map_err(|_| Error::Poisoned { cause: "MemKeyProvider lock" })?;
        let entry = keys.entry(cluster).or_default();
        if entry.destroyed {
            return Err(Error::Shredded);
        }
        if entry.created_utc == 0 {
            entry.created_utc = self.utc_secs.load(Ordering::Acquire);
        }
        entry.epochs.insert(epoch, Zeroizing::new(key));
        Ok(())
    }

    /// Simulates unlogged key loss (without destroy/retire) for KeyLost tests.
    pub fn simulate_tomb_key_loss_for_test(&self, epoch: u32) -> Result<(), Error> {
        let mut tombs = self.tomb_epochs.lock().map_err(|_| Error::Poisoned { cause: "MemKeyProvider tomb lock" })?;
        tombs.remove(&epoch);
        Ok(())
    }

    fn ensure_online(&self) -> Result<(), Error> {
        if self.offline.load(Ordering::Acquire) {
            return Err(Error::KeyUnavailable { retry_after_secs: Some(5) });
        }
        Ok(())
    }
}

impl KeyProvider for MemKeyProvider {
    fn info(&self) -> KeyProviderInfo {
        self.info
    }

    fn kek(&self, id: KekId) -> Result<Key32, Error> {
        self.ensure_online()?;
        let mut candidate = Zeroizing::new([0u8; 32]);
        self.entropy.fill(&mut *candidate)?;
        let mut keks = self.keks.lock().map_err(|_| Error::Poisoned { cause: "MemKeyProvider KEK lock" })?;
        let key = keks.entry(id).or_insert(candidate);
        Ok(Key32::from_array(**key))
    }

    fn create_cluster_keys(
        &self,
        request_id: BatchId,
        clusters: &[ClusterId],
        entropy: &dyn Entropy,
    ) -> Result<Vec<KeyCreateResult>, Error> {
        self.ensure_online()?;
        if clusters.len() > KEY_PROVIDER_BATCH_MAX {
            return Err(Error::Limit { limit: "key provider batch exceeds 256" });
        }

        if let Some(prev) = self
            .batches
            .lock()
            .map_err(|_| Error::Poisoned { cause: "MemKeyProvider batch lock" })?
            .get(&request_id)
            .cloned()
        {
            return Ok(prev
                .into_iter()
                .map(|(cluster, epoch)| KeyCreateResult {
                    cluster,
                    result: Ok(epoch),
                })
                .collect());
        }

        let now_utc = self.utc_secs.load(Ordering::Acquire);
        let mut keys = self.keys.lock().map_err(|_| Error::Poisoned { cause: "MemKeyProvider lock" })?;
        let mut results = Vec::with_capacity(clusters.len());
        let mut recorded = Vec::new();
        let mut all_ok = true;

        for &cluster in clusters {
            let entry = keys.entry(cluster).or_default();
            if entry.destroyed {
                all_ok = false;
                results.push(KeyCreateResult {
                    cluster,
                    result: Err(Error::Shredded),
                });
                continue;
            }
            if entry.disabled {
                all_ok = false;
                results.push(KeyCreateResult {
                    cluster,
                    result: Err(Error::KeyUnavailable { retry_after_secs: None }),
                });
                continue;
            }
            let epoch = if let Some(&existing) = entry.epochs.keys().next_back() {
                existing
            } else {
                let mut candidate = Zeroizing::new([0u8; 32]);
                entropy.fill(&mut *candidate)?;
                entry.epochs.insert(1, candidate);
                if entry.created_utc == 0 {
                    entry.created_utc = now_utc;
                }
                1
            };
            recorded.push((cluster, epoch));
            results.push(KeyCreateResult {
                cluster,
                result: Ok(epoch),
            });
        }

        if all_ok {
            self.batches
                .lock()
                .map_err(|_| Error::Poisoned { cause: "MemKeyProvider batch lock" })?
                .insert(request_id, recorded);
        }
        Ok(results)
    }

    fn cluster_keys(&self, req: &[(ClusterId, u32)]) -> Result<Vec<KeyLookup>, Error> {
        self.ensure_online()?;
        if req.len() > KEY_PROVIDER_BATCH_MAX {
            return Err(Error::Limit { limit: "key provider lookup batch exceeds 256" });
        }
        let keys = self.keys.lock().map_err(|_| Error::Poisoned { cause: "MemKeyProvider lock" })?;
        let mut out = Vec::with_capacity(req.len());
        for &(cluster, epoch) in req {
            let res = match keys.get(&cluster) {
                Some(entry) if entry.disabled => Err(Error::KeyUnavailable { retry_after_secs: None }),
                Some(entry) if entry.destroyed => Ok(None),
                Some(entry) => Ok(entry.epochs.get(&epoch).map(|k| Key32::from_array(**k))),
                None => Ok(None),
            };
            out.push(KeyLookup {
                cluster,
                epoch,
                result: res,
            });
        }
        Ok(out)
    }

    fn destroy_many(&self, clusters: &[ClusterId]) -> Result<DestroyReceipt, Error> {
        self.ensure_online()?;
        if clusters.len() > KEY_PROVIDER_BATCH_MAX {
            return Err(Error::Limit { limit: "key provider destroy batch exceeds 256" });
        }
        let mut keys = self.keys.lock().map_err(|_| Error::Poisoned { cause: "MemKeyProvider lock" })?;
        for cluster in clusters {
            let entry = keys.entry(*cluster).or_default();
            entry.epochs.clear();
            entry.destroyed = true;
            entry.disabled = false;
        }
        Ok(DestroyReceipt {
            requested_clusters: clusters.len(),
            absent_clusters: clusters.len(),
        })
    }

    fn probe(&self, cluster: ClusterId, epoch: u32) -> Result<KeyState, Error> {
        self.ensure_online()?;
        let keys = self.keys.lock().map_err(|_| Error::Poisoned { cause: "MemKeyProvider lock" })?;
        Ok(match keys.get(&cluster) {
            Some(entry) if entry.destroyed => KeyState::Absent,
            Some(entry) if entry.disabled => KeyState::Disabled,
            Some(entry) if entry.epochs.contains_key(&epoch) => KeyState::Present,
            _ => KeyState::Absent,
        })
    }

    fn list(&self) -> Result<Vec<KeyListEntry>, Error> {
        self.ensure_online()?;
        let keys = self.keys.lock().map_err(|_| Error::Poisoned { cause: "MemKeyProvider lock" })?;
        Ok(keys
            .iter()
            .filter_map(|(cluster, entry)| {
                (!entry.destroyed && !entry.epochs.is_empty()).then_some(KeyListEntry {
                    cluster: *cluster,
                    created_utc: entry.created_utc,
                })
            })
            .collect())
    }

    fn create_tomb_epoch(&self, epoch: u32) -> Result<(), Error> {
        self.ensure_online()?;
        let mut tombs = self.tomb_epochs.lock().map_err(|_| Error::Poisoned { cause: "MemKeyProvider tomb lock" })?;
        match tombs.get(&epoch) {
            Some(TombState::Open(_)) => Ok(()),
            Some(TombState::Retired) => Err(Error::Shredded),
            None => {
                let mut raw = Zeroizing::new([0u8; 32]);
                self.entropy.fill(&mut *raw)?;
                tombs.insert(epoch, TombState::Open(raw));
                Ok(())
            }
        }
    }

    fn tomb_key(&self, epoch: u32) -> Result<Option<Key32>, Error> {
        self.ensure_online()?;
        let tombs = self.tomb_epochs.lock().map_err(|_| Error::Poisoned { cause: "MemKeyProvider tomb lock" })?;
        match tombs.get(&epoch) {
            Some(TombState::Open(k)) => Ok(Some(Key32::from_array(**k))),
            Some(TombState::Retired) => Ok(None),
            None => Err(Error::KeyLost),
        }
    }

    fn retire_tomb_epoch(&self, epoch: u32) -> Result<KeyState, Error> {
        self.ensure_online()?;
        let mut tombs = self.tomb_epochs.lock().map_err(|_| Error::Poisoned { cause: "MemKeyProvider tomb lock" })?;
        tombs.insert(epoch, TombState::Retired);
        Ok(KeyState::Absent)
    }
}
