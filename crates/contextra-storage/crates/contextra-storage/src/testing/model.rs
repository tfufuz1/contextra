//! Reference in-memory MVCC + Erase + Branch model for differential testing (SP-VERIFY §13.1).

use contextra_storage_ports::{BranchId, ClusterId, Error, SpaceId, Stamp};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq)]
struct EntryVersion {
    stamp: Stamp,
    value: Option<Vec<u8>>, // None = tombstone
}

#[derive(Debug, Clone)]
struct RangeDeleteEntry {
    stamp: Stamp,
    start: Vec<u8>,
    end: Vec<u8>,
}

#[derive(Debug, Clone)]
struct BranchLink {
    parent: Option<BranchId>,
    fork_stamp: Stamp,
    dropped: bool,
}

#[derive(Default)]
pub struct ReferenceModel {
    current_stamp: Stamp,
    cluster_space: BTreeMap<ClusterId, SpaceId>,
    erased_clusters: BTreeMap<ClusterId, Stamp>,
    erased_spaces: BTreeMap<SpaceId, Stamp>,
    branches: BTreeMap<(ClusterId, BranchId), BranchLink>,
    entries: BTreeMap<(ClusterId, BranchId, Vec<u8>), Vec<EntryVersion>>,
    range_deletes: BTreeMap<(ClusterId, BranchId), Vec<RangeDeleteEntry>>,
}

impl ReferenceModel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn current_stamp(&self) -> Stamp {
        self.current_stamp
    }

    pub fn register_cluster(&mut self, space: SpaceId, cluster: ClusterId) {
        self.cluster_space.insert(cluster, space);
        self.branches.entry((cluster, BranchId(0))).or_insert(BranchLink {
            parent: None,
            fork_stamp: 0,
            dropped: false,
        });
    }

    pub fn put(
        &mut self,
        cluster: ClusterId,
        branch: BranchId,
        key: Vec<u8>,
        value: Vec<u8>,
    ) -> Result<Stamp, Error> {
        self.check_cluster_live(cluster)?;
        self.current_stamp += 1;
        let stamp = self.current_stamp;
        self.entries
            .entry((cluster, branch, key))
            .or_default()
            .push(EntryVersion {
                stamp,
                value: Some(value),
            });
        Ok(stamp)
    }

    pub fn delete(
        &mut self,
        cluster: ClusterId,
        branch: BranchId,
        key: Vec<u8>,
    ) -> Result<Stamp, Error> {
        self.check_cluster_live(cluster)?;
        self.current_stamp += 1;
        let stamp = self.current_stamp;
        self.entries
            .entry((cluster, branch, key))
            .or_default()
            .push(EntryVersion { stamp, value: None });
        Ok(stamp)
    }

    pub fn range_delete(
        &mut self,
        cluster: ClusterId,
        branch: BranchId,
        start: Vec<u8>,
        end: Vec<u8>,
    ) -> Result<Stamp, Error> {
        self.check_cluster_live(cluster)?;
        self.current_stamp += 1;
        let stamp = self.current_stamp;
        self.range_deletes
            .entry((cluster, branch))
            .or_default()
            .push(RangeDeleteEntry { stamp, start, end });
        Ok(stamp)
    }

    pub fn fork_branch(
        &mut self,
        cluster: ClusterId,
        parent: BranchId,
        child: BranchId,
        fork_stamp: Stamp,
    ) {
        self.branches.insert(
            (cluster, child),
            BranchLink {
                parent: Some(parent),
                fork_stamp,
                dropped: false,
            },
        );
    }

    pub fn drop_branch(&mut self, cluster: ClusterId, branch: BranchId) {
        if let Some(b) = self.branches.get_mut(&(cluster, branch)) {
            b.dropped = true;
        }
    }

    pub fn gate_cluster(&mut self, cluster: ClusterId, at: Stamp) {
        self.erased_clusters.entry(cluster).or_insert(at);
    }

    pub fn drop_space(&mut self, space: SpaceId, at: Stamp) {
        self.erased_spaces.entry(space).or_insert(at);
        let clusters: Vec<ClusterId> = self
            .cluster_space
            .iter()
            .filter_map(|(c, s)| (*s == space).then_some(*c))
            .collect();
        for c in clusters {
            self.gate_cluster(c, at);
        }
    }

    pub fn get(
        &self,
        cluster: ClusterId,
        branch: BranchId,
        key: &[u8],
        view_stamp: Stamp,
    ) -> Result<Option<Vec<u8>>, Error> {
        self.check_cluster_live(cluster)?;
        let mut cur = Some((branch, view_stamp));
        let mut best: Option<(Stamp, Option<Vec<u8>>)> = None;
        let mut seen = BTreeSet::new();
        while let Some((b, cap)) = cur {
            if !seen.insert(b) {
                break;
            }
            if let Some(link) = self.branches.get(&(cluster, b)) {
                if link.dropped {
                    return Err(Error::BranchClosed);
                }
            }
            if let Some(versions) = self.entries.get(&(cluster, b, key.to_vec())) {
                for v in versions.iter().rev() {
                    if v.stamp <= cap {
                        let shadowed = self
                            .range_deletes
                            .get(&(cluster, b))
                            .is_some_and(|rds| {
                                rds.iter().any(|rd| {
                                    rd.stamp <= cap
                                        && rd.stamp >= v.stamp
                                        && key >= rd.start.as_slice()
                                        && key < rd.end.as_slice()
                                })
                            });
                        if shadowed {
                            best = Some((v.stamp, None));
                        } else {
                            best = Some((v.stamp, v.value.clone()));
                        }
                        break;
                    }
                }
            }
            if best.is_some() {
                break;
            }
            cur = self
                .branches
                .get(&(cluster, b))
                .and_then(|link| link.parent.map(|p| (p, cap.min(link.fork_stamp))));
        }
        Ok(best.and_then(|(_, val)| val))
    }

    fn check_cluster_live(&self, cluster: ClusterId) -> Result<(), Error> {
        if let Some(at) = self.erased_clusters.get(&cluster) {
            return Err(Error::Erased { at: *at });
        }
        if let Some(space) = self.cluster_space.get(&cluster) {
            if let Some(at) = self.erased_spaces.get(space) {
                return Err(Error::Erased { at: *at });
            }
        }
        Ok(())
    }
}
