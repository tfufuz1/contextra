//! Atomic logical-erasure gate projection (SPEC §7 and §9.5 ER1).
//!
//! Persistence is owned by the commit sequencer. Apply a gate only after its
//! catalog/control record is durable; recovery replays the same idempotent
//! operation before exposing any user view.

use super::cluster_table::{ClusterState, ClusterTable};
use super::links::LinkGraph;
use super::restrictions::RestrictionTable;
use contextra_storage_ports::{ClusterId, Error, Stamp};
use std::collections::{BTreeMap, BTreeSet, HashMap};

#[derive(Debug, Clone, Default)]
pub struct GateIndex {
    erased_at: HashMap<ClusterId, Stamp>,
    generation: u64,
}

impl GateIndex {
    pub fn new() -> Self { Self::default() }
    pub fn generation(&self) -> u64 { self.generation }
    pub fn erased_at(&self, cluster: ClusterId) -> Option<Stamp> { self.erased_at.get(&cluster).copied() }
    pub fn len(&self) -> usize { self.erased_at.len() }
    pub fn is_empty(&self) -> bool { self.erased_at.is_empty() }

    /// Gate semantics override historical snapshots: once present, a cluster
    /// is never visible through any view, including a view below its gate stamp.
    pub fn check_visible(&self, cluster: ClusterId, _view_stamp: Stamp) -> Result<(), Error> {
        match self.erased_at.get(&cluster) {
            Some(at) => Err(Error::Erased { at: *at }),
            None => Ok(()),
        }
    }

    /// Applies one durable gate frame atomically to both the lifecycle table
    /// and this read filter. The scope is canonicalized and bounded before any
    /// mutation; restrictions are rechecked at the linearization boundary.
    pub fn apply_cluster_gate(
        &mut self,
        clusters: &mut ClusterTable,
        links: &LinkGraph,
        restrictions: &BTreeMap<ClusterId, RestrictionTable>,
        members: &[ClusterId],
        stamp: Stamp,
        monotone_now: u64,
        max_gate_clusters: usize,
    ) -> Result<(), Error> {
        if stamp == 0 { return Err(Error::Limit { limit: "gate stamp must be nonzero" }); }
        if members.is_empty() || members.len() > max_gate_clusters {
            return Err(Error::EraseFanoutExceeded);
        }
        let unique: BTreeSet<_> = members.iter().copied().collect();
        if unique.len() != members.len() { return Err(Error::Limit { limit: "duplicate cluster in gate scope" }); }
        let mut linked_closure = BTreeSet::new();
        for member in &unique {
            linked_closure.extend(links.erase_closure(*member)?);
            if linked_closure.len() > max_gate_clusters { return Err(Error::EraseFanoutExceeded); }
        }
        if linked_closure != unique { return Err(Error::Conflict); }

        // Validate every target first. No mutation occurs if any target is
        // unknown, restricted, non-live, or cannot advance its generation.
        let mut already_gated = 0usize;
        for id in &unique {
            if let Some(existing) = self.erased_at.get(id) {
                if *existing == stamp { already_gated += 1; continue; }
                return Err(Error::Corrupt { place: "cluster gate stamp changed" });
            }
            let entry = clusters.get(*id).ok_or(Error::UnknownCluster)?;
            if entry.state != ClusterState::Live { return Err(Error::ClusterRetired); }
            if entry.generation == u64::MAX { return Err(Error::Limit { limit: "cluster generation exhausted" }); }
            if let Some(restriction) = restrictions.get(id) { restriction.ensure_erase_allowed(monotone_now)?; }
        }
        if already_gated == unique.len() { return Ok(()); }
        if already_gated != 0 { return Err(Error::Corrupt { place: "partially applied cluster gate" }); }
        let new_entries = unique.iter().filter(|id| !self.erased_at.contains_key(id)).count();
        let next_generation = self.generation.checked_add(1).ok_or(Error::Exhausted)?;
        self.erased_at.try_reserve(new_entries).map_err(|_| Error::Limit { limit: "gate index memory" })?;

        // With exclusive mutable access and prevalidated transitions, these
        // state changes cannot fail partway unless the in-memory invariants
        // were already corrupted.
        for id in &unique {
            if !self.erased_at.contains_key(id) { clusters.mark_retired(*id, stamp)?; }
        }
        for id in unique { self.erased_at.insert(id, stamp); }
        self.generation = next_generation;
        Ok(())
    }
}
