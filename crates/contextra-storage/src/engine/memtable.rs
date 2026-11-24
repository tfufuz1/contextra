//! Lock-free ClusterMap → SkipMap memtable (E-42, SD-30).

use crossbeam_skiplist::SkipMap;
use contextra_storage_ports::{BranchId, ClusterId, Error};
use std::cmp::Ordering as CmpOrdering;
use std::collections::HashMap;
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use zeroize::Zeroizing;

pub struct MemKey {
    pub branch: BranchId,
    pub user_key: Zeroizing<Vec<u8>>,
    /// Encoded `(stamp, kind)` tag.
    pub tag: u64,
}

impl PartialEq for MemKey {
    fn eq(&self, other: &Self) -> bool {
        self.branch == other.branch && self.user_key.as_slice() == other.user_key.as_slice() && self.tag == other.tag
    }
}
impl Eq for MemKey {}
impl PartialOrd for MemKey {
    fn partial_cmp(&self, other: &Self) -> Option<CmpOrdering> { Some(self.cmp(other)) }
}
impl Ord for MemKey {
    fn cmp(&self, other: &Self) -> CmpOrdering {
        self.branch.cmp(&other.branch)
            .then_with(|| self.user_key.as_slice().cmp(other.user_key.as_slice()))
            .then_with(|| self.tag.cmp(&other.tag))
    }
}
impl fmt::Debug for MemKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str("MemKey(<redacted>)") }
}

impl MemKey {
    pub fn new(branch: BranchId, user_key: Vec<u8>, tag: u64) -> Self {
        Self { branch, user_key: Zeroizing::new(user_key), tag }
    }
}

pub struct MemValue {
    pub kind: u8,
    pub payload: Zeroizing<Vec<u8>>,
}

struct ClusterPartition {
    entries: SkipMap<MemKey, Arc<MemValue>>,
    approx_bytes: AtomicU64,
}

impl ClusterPartition {
    fn new() -> Self { Self { entries: SkipMap::new(), approx_bytes: AtomicU64::new(0) } }
}

pub struct Memtable {
    partitions: RwLock<HashMap<ClusterId, Arc<ClusterPartition>>>,
}

impl Default for Memtable {
    fn default() -> Self { Self::new() }
}

impl Memtable {
    pub fn new() -> Self { Self { partitions: RwLock::new(HashMap::new()) } }

    pub fn insert(&self, cluster: ClusterId, key: MemKey, kind: u8, payload: &[u8]) -> Result<(), Error> {
        let partition = {
            let mut partitions = self.partitions.write().map_err(|_| Error::Poisoned { cause: "memtable cluster map poisoned" })?;
            Arc::clone(partitions.entry(cluster).or_insert_with(|| Arc::new(ClusterPartition::new())))
        };
        let size = (key.user_key.len() as u64)
            .saturating_add(payload.len() as u64)
            .saturating_add(32);
        partition.entries.insert(key, Arc::new(MemValue { kind, payload: Zeroizing::new(payload.to_vec()) }));
        partition.approx_bytes.fetch_add(size, Ordering::Relaxed);
        Ok(())
    }

    pub fn get(&self, cluster: ClusterId, key: &MemKey) -> Result<Option<Arc<MemValue>>, Error> {
        let partition = self.partitions.read()
            .map_err(|_| Error::Poisoned { cause: "memtable cluster map poisoned" })?
            .get(&cluster).cloned();
        Ok(partition.and_then(|part| part.entries.get(key).map(|entry| Arc::clone(entry.value()))))
    }

    /// Entfernt die Partition O(1); der zurückgegebene Arc wird auf urgent-Lane reclaimed.
    pub fn detach_cluster(&self, cluster: ClusterId) -> Result<Option<DetachedPartition>, Error> {
        let partition = self.partitions.write()
            .map_err(|_| Error::Poisoned { cause: "memtable cluster map poisoned" })?
            .remove(&cluster);
        Ok(partition.map(|partition| DetachedPartition { partition }))
    }

    pub fn approx_bytes(&self) -> Result<u64, Error> {
        let partitions = self.partitions.read().map_err(|_| Error::Poisoned { cause: "memtable cluster map poisoned" })?;
        Ok(partitions.values().fold(0u64, |sum, p| sum.saturating_add(p.approx_bytes.load(Ordering::Relaxed))))
    }

    pub fn cluster_count(&self) -> Result<usize, Error> {
        Ok(self.partitions.read().map_err(|_| Error::Poisoned { cause: "memtable cluster map poisoned" })?.len())
    }
}

pub struct DetachedPartition { partition: Arc<ClusterPartition> }

impl DetachedPartition {
    pub fn approx_bytes(&self) -> u64 { self.partition.approx_bytes.load(Ordering::Acquire) }
    pub fn is_empty(&self) -> bool { self.partition.entries.is_empty() }
    pub fn visit(&self, mut visitor: impl FnMut(&MemKey, &Arc<MemValue>)) {
        for entry in self.partition.entries.iter() {
            visitor(entry.key(), entry.value());
        }
    }
}
