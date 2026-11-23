//! Immutable read root (SP-MEMVER). Payload/run roots are added by flush phases.

use super::memtable::DetachedPartition;
use contextra_storage_ports::ClusterId;
use contextra_storage_ports::BranchId;
use contextra_storage_ports::{Error, Stamp};
use std::sync::Arc;
use zeroize::Zeroizing;

#[derive(Clone)]
pub struct VersionEntry {
    pub cluster: ClusterId,
    pub branch: BranchId,
    pub key: Zeroizing<Vec<u8>>,
    pub stamp: Stamp,
    pub kind: u8,
    pub payload: Zeroizing<Vec<u8>>,
}

impl VersionEntry {
    pub fn new(cluster: ClusterId, branch: BranchId, key: Vec<u8>, stamp: Stamp, kind: u8, payload: Vec<u8>) -> Self {
        Self { cluster, branch, key: Zeroizing::new(key), stamp, kind, payload: Zeroizing::new(payload) }
    }
}

#[derive(Clone, Default)]
pub struct Version {
    visible_stamp: Stamp,
    durable_stamp: Stamp,
    partitions: Arc<[Arc<DetachedPartition>]>,
    durable_partitions: Arc<[Arc<DetachedPartition>]>,
    gc_floor: Stamp,
    parent: Option<Arc<Version>>,
    changes: Arc<[VersionEntry]>,
}

impl Version {
    pub fn empty() -> Self { Self::default() }

    pub fn new(
        visible_stamp: Stamp,
        durable_stamp: Stamp,
        gc_floor: Stamp,
        partitions: Vec<Arc<DetachedPartition>>,
        durable_partitions: Vec<Arc<DetachedPartition>>,
    ) -> Result<Self, Error> {
        if durable_stamp > visible_stamp || gc_floor > visible_stamp {
            return Err(Error::Limit { limit: "version stamp ordering" });
        }
        Ok(Self { visible_stamp, durable_stamp, partitions: partitions.into(), durable_partitions: durable_partitions.into(), gc_floor, parent: None, changes: Arc::from([]) })
    }

    pub fn commit(
        parent: Arc<Version>,
        visible_stamp: Stamp,
        durable_stamp: Stamp,
        changes: Vec<VersionEntry>,
    ) -> Result<Self, Error> {
        if visible_stamp <= parent.visible_stamp || durable_stamp < parent.durable_stamp || durable_stamp > visible_stamp {
            return Err(Error::Limit { limit: "version commit stamp ordering" });
        }
        if changes.iter().any(|entry| entry.stamp != visible_stamp) {
            return Err(Error::Limit { limit: "version delta stamp mismatch" });
        }
        Ok(Self {
            visible_stamp,
            durable_stamp,
            gc_floor: parent.gc_floor,
            partitions: Arc::clone(&parent.partitions),
            durable_partitions: Arc::clone(&parent.durable_partitions),
            parent: Some(parent),
            changes: changes.into(),
        })
    }

    pub fn visible_stamp(&self) -> Stamp { self.visible_stamp }
    pub fn durable_stamp(&self) -> Stamp { self.durable_stamp }
    pub fn gc_floor(&self) -> Stamp { self.gc_floor }
    pub fn partitions(&self) -> &[Arc<DetachedPartition>] { &self.partitions }
    pub fn durable_partitions(&self) -> &[Arc<DetachedPartition>] { &self.durable_partitions }
    pub fn changes(&self) -> &[VersionEntry] { &self.changes }

    /// Returns the newest point mutation at or below `cap`. Tombstone kinds
    /// are returned to the read layer for visibility resolution.
    pub fn get(&self, cluster: ClusterId, branch: BranchId, key: &[u8], cap: Stamp) -> Option<&VersionEntry> {
        if cap < self.gc_floor { return None; }
        let mut current = Some(self);
        while let Some(version) = current {
            if let Some(entry) = version.changes.iter().rev().find(|entry| {
                entry.stamp <= cap && entry.cluster == cluster && entry.branch == branch && entry.key.as_slice() == key
            }) { return Some(entry); }
            current = version.parent.as_deref();
        }
        None
    }

    /// Resolves point mutations through a cluster-local branch ancestry.
    /// The first branch in the chain wins even when the entry is a tombstone;
    /// callers must apply tombstone, range-delete, and erase-mask semantics
    /// before returning a value. This keeps a child delete from revealing its
    /// parent's older value.
    pub fn get_on_branch<'a>(
        &'a self,
        branches: &super::branches::BranchTable,
        cluster: ClusterId,
        branch: BranchId,
        key: &[u8],
        view_stamp: Stamp,
    ) -> Result<Option<&'a VersionEntry>, Error> {
        for (branch_id, cap) in branches.visibility_chain(branch, view_stamp)? {
            if let Some(entry) = self.get(cluster, branch_id, key, cap) {
                return Ok(Some(entry));
            }
        }
        Ok(None)
    }

    pub fn at_stamp(&self, stamp: Stamp) -> Result<Self, Error> {
        if stamp < self.gc_floor {
            return Err(Error::Limit { limit: "view stamp outside version bounds" });
        }
        Ok(Self {
            visible_stamp: stamp,
            durable_stamp: self.durable_stamp.min(stamp),
            partitions: Arc::clone(&self.partitions),
            durable_partitions: Arc::clone(&self.durable_partitions),
            gc_floor: self.gc_floor,
            parent: self.parent.as_ref().map(Arc::clone),
            changes: Arc::clone(&self.changes),
        })
    }

    pub fn with_gc_floor(&self, gc_floor: Stamp) -> Result<Self, Error> {
        if gc_floor < self.gc_floor || gc_floor > self.visible_stamp {
            return Err(Error::Limit { limit: "version gc floor" });
        }
        Ok(Self {
            visible_stamp: self.visible_stamp,
            durable_stamp: self.durable_stamp,
            partitions: Arc::clone(&self.partitions),
            durable_partitions: Arc::clone(&self.durable_partitions),
            gc_floor,
            parent: self.parent.as_ref().map(Arc::clone),
            changes: Arc::clone(&self.changes),
        })
    }

    /// Rebuilds a bounded immutable delta chain. The newest value at or below
    /// the floor is retained per key, along with every later mutation.
    pub fn compact_history(&self, gc_floor: Stamp) -> Result<Self, Error> {
        if gc_floor < self.gc_floor || gc_floor > self.visible_stamp {
            return Err(Error::Limit { limit: "version compact floor" });
        }
        let mut all = Vec::new();
        let mut current = Some(self);
        while let Some(version) = current {
            all.extend(version.changes.iter().cloned());
            current = version.parent.as_deref();
        }
        all.sort_by_key(|entry| entry.stamp);
        let mut baseline = Vec::<VersionEntry>::new();
        let mut later = Vec::<VersionEntry>::new();
        for entry in all {
            if entry.stamp > gc_floor {
                later.push(entry);
                continue;
            }
            if let Some(previous) = baseline.iter_mut().find(|candidate|
                candidate.cluster == entry.cluster && candidate.branch == entry.branch && candidate.key == entry.key
            ) {
                *previous = entry;
            } else {
                baseline.push(entry);
            }
        }
        baseline.extend(later);
        Ok(Self {
            visible_stamp: self.visible_stamp,
            durable_stamp: self.durable_stamp,
            partitions: Arc::clone(&self.partitions),
            durable_partitions: Arc::clone(&self.durable_partitions),
            gc_floor,
            parent: None,
            changes: baseline.into(),
        })
    }
}
