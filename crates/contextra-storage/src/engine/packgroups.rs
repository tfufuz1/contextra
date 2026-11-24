//! Packgroup policies, clustering, and expiration batching (SPEC §9.8, SP-CLUSTER, SD-02).
#![forbid(unsafe_code)]

use contextra_storage_ports::{ClusterId, Error, Protection};
use std::collections::BTreeSet;

pub const MAX_PACKGROUP_GATE_BATCH: usize = 4096;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PackKey(pub [u8; 16]);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackPolicy {
    Pure,
    Packed,
    Staging,
}

#[derive(Debug, Clone)]
pub struct PackGroup {
    pub key: PackKey,
    pub policy: PackPolicy,
    pub protection: Protection,
    pub members: BTreeSet<ClusterId>,
}

impl PackGroup {
    pub fn new(key: PackKey, policy: PackPolicy, protection: Protection) -> Self {
        Self {
            key,
            policy,
            protection,
            members: BTreeSet::new(),
        }
    }

    pub fn add_member(&mut self, cluster: ClusterId) -> Result<(), Error> {
        self.members.insert(cluster);
        Ok(())
    }

    pub fn remove_member(&mut self, cluster: &ClusterId) -> bool {
        self.members.remove(cluster)
    }

    pub fn len(&self) -> usize {
        self.members.len()
    }

    pub fn is_empty(&self) -> bool {
        self.members.is_empty()
    }
}

/// Splits expired packgroup clusters into deterministic batches of <= 4096 clusters,
/// ensuring each batch can be processed as an independent erase ticket (SPEC §9.8).
pub fn split_pack_expired(
    clusters: &[ClusterId],
    max_batch: usize,
) -> Vec<Vec<ClusterId>> {
    let limit = if max_batch == 0 || max_batch > MAX_PACKGROUP_GATE_BATCH {
        MAX_PACKGROUP_GATE_BATCH
    } else {
        max_batch
    };

    let mut batches = Vec::new();
    for chunk in clusters.chunks(limit) {
        batches.push(chunk.to_vec());
    }
    batches
}
