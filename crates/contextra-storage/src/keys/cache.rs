//! Bounded zeroizing cluster-key cache with generation fencing (INV-KEY-2).

use crate::sync::ranked::Ranked;
use contextra_storage_ports::traits::Clock;
use contextra_storage_ports::{ClusterId, Error, Key32, KeyProvider};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use zeroize::Zeroizing;

const MAX_PREFETCH: usize = 256;

struct CacheEntry {
    key: Zeroizing<[u8; 32]>,
    expires_at: u64,
    last_used: u64,
}

#[derive(Default)]
struct CacheState {
    entries: HashMap<(ClusterId, u32), CacheEntry>,
    generations: HashMap<ClusterId, Arc<AtomicU64>>,
    revoked: HashSet<ClusterId>,
    tick: u64,
}

pub struct KeyCache {
    state: Ranked<CacheState, 6>,
    provider: Arc<dyn KeyProvider>,
    clock: Arc<dyn Clock>,
    ttl_nanos: u64,
    grace_nanos: u64,
    max_clusters: usize,
}

pub struct Key32Handle {
    key: Key32,
    generation: Arc<AtomicU64>,
    expected_generation: u64,
}

impl Key32Handle {
    /// The closure cannot keep a reference to key material beyond this call.
    /// A concurrent invalidation fences the result even if the operation was
    /// already using its bounded temporary key handle.
    pub fn with_key<T>(&self, use_key: impl FnOnce(&Key32) -> Result<T, Error>) -> Result<T, Error> {
        if self.generation.load(Ordering::Acquire) != self.expected_generation {
            return Err(Error::ClusterRetired);
        }
        let result = use_key(&self.key);
        if self.generation.load(Ordering::Acquire) != self.expected_generation {
            return Err(Error::ClusterRetired);
        }
        result
    }
}

impl fmt::Debug for Key32Handle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str("Key32Handle(<redacted>)") }
}

impl KeyCache {
    pub fn new(
        provider: Arc<dyn KeyProvider>,
        clock: Arc<dyn Clock>,
        ttl_nanos: u64,
        grace_nanos: u64,
        max_clusters: usize,
    ) -> Result<Self, Error> {
        if ttl_nanos == 0 || max_clusters == 0 {
            return Err(Error::Limit { limit: "key cache TTL and cluster bound must be nonzero" });
        }
        Ok(Self { state: Ranked::new(CacheState::default()), provider, clock,
            ttl_nanos, grace_nanos, max_clusters })
    }

    pub fn get(&self, cluster: ClusterId, epoch: u32) -> Result<Key32Handle, Error> {
        let now = self.clock.read()?.monotonic_nanos;
        let (generation, generation_snapshot, stale) = {
            let mut state = self.state.lock()?;
            if state.revoked.contains(&cluster) { return Err(Error::ClusterRetired); }
            let generation = generation_for(&mut state, cluster);
            let snapshot = generation.load(Ordering::Acquire);
            state.tick = state.tick.checked_add(1).ok_or(Error::Limit { limit: "key cache LRU clock exhausted" })?;
            let tick = state.tick;
            let entry = state.entries.get_mut(&(cluster, epoch));
            let stale = entry.map(|entry| {
                entry.last_used = tick;
                CacheEntry { key: Zeroizing::new(*entry.key), expires_at: entry.expires_at, last_used: tick }
            });
            if let Some(entry) = stale.as_ref().filter(|entry| now <= entry.expires_at) {
                return Ok(make_handle(&*entry.key, generation, snapshot));
            }
            (generation, snapshot, stale)
        };

        let lookup = self.provider.cluster_key(cluster, epoch);
        let key = match lookup {
            Ok(Some(key)) => key,
            Ok(None) => return Err(Error::KeyLost),
            Err(error) => {
                if error.class() == contextra_storage_ports::ErrorClass::Transient {
                    if let Some(entry) = stale.filter(|entry| within_grace(now, entry.expires_at, self.grace_nanos)) {
                        if generation.load(Ordering::Acquire) != generation_snapshot {
                            return Err(Error::ClusterRetired);
                        }
                        return Ok(make_handle(&*entry.key, Arc::clone(&generation), generation_snapshot));
                    }
                }
                return Err(error);
            }
        };
        let generation_value = generation.load(Ordering::Acquire);
        let expires_at = now.checked_add(self.ttl_nanos).ok_or(Error::Limit { limit: "key cache TTL overflow" })?;
        let result_key = Zeroizing::new(*key.expose());
        {
            let mut state = self.state.lock()?;
            if generation.load(Ordering::Acquire) != generation_value { return Err(Error::ClusterRetired); }
            state.tick = state.tick.checked_add(1).ok_or(Error::Limit { limit: "key cache LRU clock exhausted" })?;
            let tick = state.tick;
            state.entries.insert((cluster, epoch), CacheEntry {
                key: Zeroizing::new(*result_key), expires_at, last_used: tick,
            });
            evict_clusters(&mut state, self.max_clusters, cluster);
        }
        if generation.load(Ordering::Acquire) != generation_value { return Err(Error::ClusterRetired); }
        Ok(make_handle(&*result_key, generation, generation_value))
    }

    /// Fetches bounded groups of misses without holding the cache lock across
    /// provider calls.
    pub fn prefetch(&self, request: &[(ClusterId, u32)]) -> Result<(), Error> {
        for chunk in request.chunks(MAX_PREFETCH) {
        let now = self.clock.read()?.monotonic_nanos;
            let (misses, generations) = {
                let mut state = self.state.lock()?;
                let mut misses = Vec::new();
                let mut generations = Vec::new();
                for (cluster, epoch) in chunk {
                    if state.revoked.contains(cluster) { return Err(Error::ClusterRetired); }
                    let generation = generation_for(&mut state, *cluster);
                    let fresh = state.entries.get(&(*cluster, *epoch))
                        .is_some_and(|entry| now <= entry.expires_at);
                    if !fresh {
                        misses.push((*cluster, *epoch));
                        generations.push((Arc::clone(&generation), generation.load(Ordering::Acquire)));
                    }
                }
                (misses, generations)
            };
            if misses.is_empty() { continue; }
            let expires_at = now.checked_add(self.ttl_nanos).ok_or(Error::Limit { limit: "key cache TTL overflow" })?;
            let mut resolved = Vec::with_capacity(misses.len());
            for (index, (cluster, epoch)) in misses.iter().enumerate() {
                let (generation, expected) = generations.get(index).ok_or(Error::Corrupt { place: "key cache prefetch generation" })?;
                let key = self.provider.cluster_key(*cluster, *epoch)?.ok_or(Error::KeyLost)?;
                if generation.load(Ordering::Acquire) != *expected { return Err(Error::ClusterRetired); }
                resolved.push((*cluster, *epoch, Zeroizing::new(*key.expose())));
            }
            let mut state = self.state.lock()?;
            for (index, (cluster, epoch, key)) in resolved.into_iter().enumerate() {
                let (generation, expected) = generations.get(index).ok_or(Error::Corrupt { place: "key cache prefetch generation" })?;
                if generation.load(Ordering::Acquire) != *expected { return Err(Error::ClusterRetired); }
                state.tick = state.tick.checked_add(1).ok_or(Error::Limit { limit: "key cache LRU clock exhausted" })?;
                let tick = state.tick;
                state.entries.insert((cluster, epoch), CacheEntry { key, expires_at, last_used: tick });
                evict_clusters(&mut state, self.max_clusters, cluster);
            }
        }
        Ok(())
    }

    pub fn invalidate(&self, cluster: ClusterId) -> Result<u64, Error> {
        let mut state = self.state.lock()?;
        state.revoked.insert(cluster);
        let generation = generation_for(&mut state, cluster);
        let mut current = generation.load(Ordering::Acquire);
        loop {
            let next = current.checked_add(1).ok_or(Error::Limit { limit: "cluster key generation exhausted" })?;
            match generation.compare_exchange_weak(current, next, Ordering::AcqRel, Ordering::Acquire) {
                Ok(_) => {
                    state.entries.retain(|(entry_cluster, _), _| *entry_cluster != cluster);
                    return Ok(next);
                }
                Err(observed) => current = observed,
            }
        }
    }
}

fn generation_for(state: &mut CacheState, cluster: ClusterId) -> Arc<AtomicU64> {
    Arc::clone(state.generations.entry(cluster).or_insert_with(|| Arc::new(AtomicU64::new(0))))
}

fn make_handle(key: &[u8; 32], generation: Arc<AtomicU64>, expected_generation: u64) -> Key32Handle {
    Key32Handle { key: Key32::from_array(*key), generation, expected_generation }
}

fn within_grace(now: u64, expires_at: u64, grace_nanos: u64) -> bool {
    now <= expires_at.saturating_add(grace_nanos)
}

fn evict_clusters(state: &mut CacheState, max_clusters: usize, keep: ClusterId) {
    loop {
        let mut clusters: HashMap<ClusterId, u64> = HashMap::new();
        for ((cluster, _), entry) in &state.entries {
            clusters.entry(*cluster).and_modify(|last| *last = (*last).min(entry.last_used))
                .or_insert(entry.last_used);
        }
        if clusters.len() <= max_clusters { return; }
        let victim = clusters.into_iter().filter(|(cluster, _)| *cluster != keep)
            .min_by_key(|(_, last_used)| *last_used).map(|(cluster, _)| cluster);
        let Some(victim) = victim else { return; };
        state.entries.retain(|(cluster, _), _| *cluster != victim);
    }
}
