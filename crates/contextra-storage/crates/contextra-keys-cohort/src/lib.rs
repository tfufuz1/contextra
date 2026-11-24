//! Cohort-rewrap KeyProvider for large-scale multi-tenant cluster fleets (SP-KEYS §3.9, SPEC §9.4).
//!
//! Cluster keys are wrapped under cohort KEKs (`COHORT_MAX <= 1024`). Destroying
//! a cluster removes its wrapped key, generates a fresh random cohort KEK generation
//! for surviving members, rewraps survivors under the new KEK, and drops the old KEK.
#![forbid(unsafe_code)]

use chacha20poly1305::aead::{AeadInPlace, KeyInit};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use contextra_storage_ports::limits::COHORT_MAX;
use contextra_storage_ports::traits::Entropy;
use contextra_storage_ports::{
    BatchId, ClusterId, DestroyBehavior, DestroyReceipt, Error, KekId, Key32, KeyAssurance,
    KeyCreateResult, KeyListEntry, KeyLookup, KeyProvider, KeyProviderInfo, KeyState,
    KeysInBackups, KEY_PROVIDER_BATCH_MAX,
};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use zeroize::Zeroizing;

#[derive(Clone)]
struct WrappedClusterKey {
    nonce: [u8; 24],
    ciphertext_and_tag: Vec<u8>,
}

#[derive(Default)]
struct CohortClusterEntry {
    cohort_id: u32,
    epochs: BTreeMap<u32, WrappedClusterKey>,
    created_utc: u64,
    destroyed: bool,
    disabled: bool,
}

#[derive(Clone)]
struct CohortKekState {
    generation: u64,
    key: Zeroizing<[u8; 32]>,
}

enum CohortTombState {
    Open(Zeroizing<[u8; 32]>),
    Retired,
}

pub struct CohortKeyProvider {
    entries: Mutex<BTreeMap<ClusterId, CohortClusterEntry>>,
    cohort_keks: Mutex<BTreeMap<u32, CohortKekState>>,
    keks: Mutex<BTreeMap<KekId, Zeroizing<[u8; 32]>>>,
    tombs: Mutex<BTreeMap<u32, CohortTombState>>,
    batches: Mutex<HashMap<BatchId, Vec<(ClusterId, u32)>>>,
    utc_override: AtomicU64,
}

fn fill_random(buf: &mut [u8]) -> Result<(), Error> {
    getrandom::fill(buf).map_err(|_| Error::Io {
        role: "keys",
        kind: std::io::ErrorKind::Other,
    })
}

fn current_utc_secs(override_secs: &AtomicU64) -> u64 {
    let ov = override_secs.load(Ordering::Acquire);
    if ov > 0 {
        return ov;
    }
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(1)
        .max(1)
}

fn cohort_aad(cohort_id: u32, generation: u64, cluster: ClusterId, epoch: u32) -> [u8; 32] {
    let mut aad = [0u8; 32];
    aad[..4].copy_from_slice(&cohort_id.to_be_bytes());
    aad[4..12].copy_from_slice(&generation.to_be_bytes());
    aad[12..28].copy_from_slice(cluster.as_bytes());
    aad[28..32].copy_from_slice(&epoch.to_be_bytes());
    aad
}

fn wrap_with_cohort_kek(
    kek: &CohortKekState,
    cohort_id: u32,
    cluster: ClusterId,
    epoch: u32,
    plaintext_key: &[u8; 32],
) -> Result<WrappedClusterKey, Error> {
    let cipher = XChaCha20Poly1305::new_from_slice(&*kek.key)
        .map_err(|_| Error::Corrupt { place: "cohort kek init" })?;
    let mut nonce = [0u8; 24];
    fill_random(&mut nonce)?;
    let aad = cohort_aad(cohort_id, kek.generation, cluster, epoch);
    let mut buf = Zeroizing::new(plaintext_key.to_vec());
    let tag = cipher
        .encrypt_in_place_detached(XNonce::from_slice(&nonce), &aad, &mut buf)
        .map_err(|_| Error::Corrupt { place: "cohort key wrap" })?;
    let mut out = Vec::with_capacity(buf.len() + 16);
    out.extend_from_slice(&buf);
    out.extend_from_slice(&tag);
    Ok(WrappedClusterKey {
        nonce,
        ciphertext_and_tag: out,
    })
}

fn unwrap_with_cohort_kek(
    kek: &CohortKekState,
    cohort_id: u32,
    cluster: ClusterId,
    epoch: u32,
    wrapped: &WrappedClusterKey,
) -> Result<Zeroizing<[u8; 32]>, Error> {
    if wrapped.ciphertext_and_tag.len() != 32 + 16 {
        return Err(Error::Corrupt { place: "cohort wrapped key length" });
    }
    let cipher = XChaCha20Poly1305::new_from_slice(&*kek.key)
        .map_err(|_| Error::Corrupt { place: "cohort kek init" })?;
    let aad = cohort_aad(cohort_id, kek.generation, cluster, epoch);
    let mut pt = Zeroizing::new(wrapped.ciphertext_and_tag[..32].to_vec());
    let tag = chacha20poly1305::Tag::from_slice(&wrapped.ciphertext_and_tag[32..]);
    cipher
        .decrypt_in_place_detached(XNonce::from_slice(&wrapped.nonce), &aad, &mut pt, tag)
        .map_err(|_| Error::Corrupt { place: "cohort key unwrap auth" })?;
    let raw: [u8; 32] = pt
        .as_slice()
        .try_into()
        .map_err(|_| Error::Corrupt { place: "cohort unwrapped key size" })?;
    Ok(Zeroizing::new(raw))
}

impl CohortKeyProvider {
    pub fn new() -> Self {
        Self {
            entries: Mutex::new(BTreeMap::new()),
            cohort_keks: Mutex::new(BTreeMap::new()),
            keks: Mutex::new(BTreeMap::new()),
            tombs: Mutex::new(BTreeMap::new()),
            batches: Mutex::new(HashMap::new()),
            utc_override: AtomicU64::new(0),
        }
    }

    pub fn set_utc_secs_for_test(&self, utc: u64) {
        self.utc_override.store(utc, Ordering::Release);
    }

    pub fn assign_cohort(&self, cluster: ClusterId, cohort_id: u32) -> Result<(), Error> {
        let mut entries = self.entries.lock().map_err(|_| Error::Poisoned {
            cause: "CohortKeyProvider lock",
        })?;
        let active_in_cohort = entries
            .iter()
            .filter(|(id, e)| **id != cluster && e.cohort_id == cohort_id && !e.destroyed)
            .count();
        if active_in_cohort >= COHORT_MAX {
            return Err(Error::Limit {
                limit: "cohort member capacity 1024",
            });
        }
        let entry = entries.entry(cluster).or_default();
        if !entry.epochs.is_empty() && entry.cohort_id != cohort_id {
            return Err(Error::Limit {
                limit: "cannot reassign cohort after key creation",
            });
        }
        entry.cohort_id = cohort_id;
        Ok(())
    }

    pub fn cohort_kek_generation(&self, cohort_id: u32) -> Result<u64, Error> {
        let keks = self.cohort_keks.lock().map_err(|_| Error::Poisoned {
            cause: "CohortKeyProvider kek lock",
        })?;
        Ok(keks.get(&cohort_id).map(|k| k.generation).unwrap_or(0))
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
            keys_in_backups: KeysInBackups::No,
            assurance: KeyAssurance::Kms,
        }
    }

    fn kek(&self, id: KekId) -> Result<Key32, Error> {
        let mut keks = self.keks.lock().map_err(|_| Error::Poisoned {
            cause: "CohortKeyProvider kek lock",
        })?;
        if let Some(existing) = keks.get(&id) {
            return Ok(Key32::from_array(**existing));
        }
        let mut raw = Zeroizing::new([0u8; 32]);
        fill_random(&mut *raw)?;
        keks.insert(id, Zeroizing::new(*raw));
        Ok(Key32::from_array(*raw))
    }

    fn create_cluster_keys(
        &self,
        request_id: BatchId,
        clusters: &[ClusterId],
        entropy: &dyn Entropy,
    ) -> Result<Vec<KeyCreateResult>, Error> {
        if clusters.len() > KEY_PROVIDER_BATCH_MAX {
            return Err(Error::Limit {
                limit: "key provider batch exceeds 256",
            });
        }
        if let Some(prev) = self
            .batches
            .lock()
            .map_err(|_| Error::Poisoned {
                cause: "CohortKeyProvider batch lock",
            })?
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

        let now_utc = current_utc_secs(&self.utc_override);
        let mut entries = self.entries.lock().map_err(|_| Error::Poisoned {
            cause: "CohortKeyProvider lock",
        })?;
        let mut keks = self.cohort_keks.lock().map_err(|_| Error::Poisoned {
            cause: "CohortKeyProvider kek lock",
        })?;

        let mut results = Vec::with_capacity(clusters.len());
        let mut recorded = Vec::new();
        let mut all_ok = true;

        for &cluster in clusters {
            let entry = entries.entry(cluster).or_default();
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
                    result: Err(Error::KeyUnavailable {
                        retry_after_secs: None,
                    }),
                });
                continue;
            }
            let epoch = if let Some(&existing) = entry.epochs.keys().next_back() {
                existing
            } else {
                let mut candidate = Zeroizing::new([0u8; 32]);
                entropy.fill(&mut *candidate)?;
                let cohort_id = entry.cohort_id;
                let kek_state = match keks.get(&cohort_id) {
                    Some(k) => k.clone(),
                    None => {
                        let mut fresh_kek = Zeroizing::new([0u8; 32]);
                        fill_random(&mut *fresh_kek)?;
                        let state = CohortKekState {
                            generation: 1,
                            key: fresh_kek,
                        };
                        keks.insert(cohort_id, state.clone());
                        state
                    }
                };
                let wrapped = wrap_with_cohort_kek(&kek_state, cohort_id, cluster, 1, &candidate)?;
                entry.epochs.insert(1, wrapped);
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
                .map_err(|_| Error::Poisoned {
                    cause: "CohortKeyProvider batch lock",
                })?
                .insert(request_id, recorded);
        }
        Ok(results)
    }

    fn cluster_keys(&self, req: &[(ClusterId, u32)]) -> Result<Vec<KeyLookup>, Error> {
        if req.len() > KEY_PROVIDER_BATCH_MAX {
            return Err(Error::Limit {
                limit: "key provider lookup batch exceeds 256",
            });
        }
        let entries = self.entries.lock().map_err(|_| Error::Poisoned {
            cause: "CohortKeyProvider lock",
        })?;
        let keks = self.cohort_keks.lock().map_err(|_| Error::Poisoned {
            cause: "CohortKeyProvider kek lock",
        })?;
        let mut out = Vec::with_capacity(req.len());
        for &(cluster, epoch) in req {
            let res = match entries.get(&cluster) {
                Some(entry) if entry.disabled => Err(Error::KeyUnavailable {
                    retry_after_secs: None,
                }),
                Some(entry) if entry.destroyed => Ok(None),
                Some(entry) => match entry.epochs.get(&epoch) {
                    Some(wrapped) => {
                        let kek = keks.get(&entry.cohort_id).ok_or(Error::KeyLost)?;
                        let unwrapped =
                            unwrap_with_cohort_kek(kek, entry.cohort_id, cluster, epoch, wrapped)?;
                        Ok(Some(Key32::from_array(*unwrapped)))
                    }
                    None => Ok(None),
                },
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
        if clusters.len() > KEY_PROVIDER_BATCH_MAX {
            return Err(Error::Limit {
                limit: "key provider destroy batch exceeds 256",
            });
        }
        let mut entries = self.entries.lock().map_err(|_| Error::Poisoned {
            cause: "CohortKeyProvider lock",
        })?;
        let mut affected_cohorts = BTreeSet::new();
        let mut absent = 0;
        for c in clusters {
            let entry = entries.entry(*c).or_default();
            if !entry.destroyed {
                affected_cohorts.insert(entry.cohort_id);
            }
            entry.epochs.clear();
            entry.destroyed = true;
            absent += 1;
        }

        // Rewrap all surviving cluster keys in affected cohorts under a fresh random cohort KEK
        let mut keks = self.cohort_keks.lock().map_err(|_| Error::Poisoned {
            cause: "CohortKeyProvider kek lock",
        })?;
        for cohort_id in affected_cohorts {
            let has_survivors = entries
                .values()
                .any(|e| e.cohort_id == cohort_id && !e.destroyed && !e.epochs.is_empty());
            if has_survivors {
                let old_kek = keks.get(&cohort_id).cloned().ok_or(Error::KeyLost)?;
                let mut new_raw = Zeroizing::new([0u8; 32]);
                fill_random(&mut *new_raw)?;
                let new_kek = CohortKekState {
                    generation: old_kek.generation.saturating_add(1),
                    key: new_raw,
                };
                for (cluster, entry) in entries.iter_mut() {
                    if entry.cohort_id == cohort_id && !entry.destroyed && !entry.epochs.is_empty() {
                        let mut rewrapped = BTreeMap::new();
                        for (epoch, wrapped) in &entry.epochs {
                            let pt = unwrap_with_cohort_kek(
                                &old_kek,
                                cohort_id,
                                *cluster,
                                *epoch,
                                wrapped,
                            )?;
                            let nw = wrap_with_cohort_kek(
                                &new_kek,
                                cohort_id,
                                *cluster,
                                *epoch,
                                &pt,
                            )?;
                            rewrapped.insert(*epoch, nw);
                        }
                        entry.epochs = rewrapped;
                    }
                }
                keks.insert(cohort_id, new_kek);
            } else {
                keks.remove(&cohort_id);
            }
        }

        Ok(DestroyReceipt {
            requested_clusters: clusters.len(),
            absent_clusters: absent,
        })
    }

    fn probe(&self, cluster: ClusterId, epoch: u32) -> Result<KeyState, Error> {
        let entries = self.entries.lock().map_err(|_| Error::Poisoned {
            cause: "CohortKeyProvider lock",
        })?;
        Ok(match entries.get(&cluster) {
            Some(e) if e.destroyed => KeyState::Absent,
            Some(e) if e.disabled => KeyState::Disabled,
            Some(e) if e.epochs.contains_key(&epoch) => KeyState::Present,
            _ => KeyState::Absent,
        })
    }

    fn list(&self) -> Result<Vec<KeyListEntry>, Error> {
        let entries = self.entries.lock().map_err(|_| Error::Poisoned {
            cause: "CohortKeyProvider lock",
        })?;
        Ok(entries
            .iter()
            .filter(|(_, e)| !e.destroyed && !e.epochs.is_empty())
            .map(|(c, e)| KeyListEntry {
                cluster: *c,
                created_utc: e.created_utc,
            })
            .collect())
    }

    fn create_tomb_epoch(&self, epoch: u32) -> Result<(), Error> {
        let mut tombs = self.tombs.lock().map_err(|_| Error::Poisoned {
            cause: "CohortKeyProvider tomb lock",
        })?;
        match tombs.get(&epoch) {
            Some(CohortTombState::Open(_)) => Ok(()),
            Some(CohortTombState::Retired) => Err(Error::Shredded),
            None => {
                let mut raw = Zeroizing::new([0u8; 32]);
                fill_random(&mut *raw)?;
                tombs.insert(epoch, CohortTombState::Open(raw));
                Ok(())
            }
        }
    }

    fn tomb_key(&self, epoch: u32) -> Result<Option<Key32>, Error> {
        let tombs = self.tombs.lock().map_err(|_| Error::Poisoned {
            cause: "CohortKeyProvider tomb lock",
        })?;
        match tombs.get(&epoch) {
            Some(CohortTombState::Open(k)) => Ok(Some(Key32::from_array(**k))),
            Some(CohortTombState::Retired) => Ok(None),
            None => Err(Error::KeyLost),
        }
    }

    fn retire_tomb_epoch(&self, epoch: u32) -> Result<KeyState, Error> {
        let mut tombs = self.tombs.lock().map_err(|_| Error::Poisoned {
            cause: "CohortKeyProvider tomb lock",
        })?;
        tombs.insert(epoch, CohortTombState::Retired);
        Ok(KeyState::Absent)
    }
}
