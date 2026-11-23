//! Compact cluster lifecycle table (SP-CLUSTER, E-33/E-38).

use contextra_storage_ports::{ClusterId, Error, Protection, Stamp};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ClusterState { Live, Retired, Shredded, Purged }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClusterEntry {
    pub id: ClusterId,
    pub protection: Protection,
    pub state: ClusterState,
    pub state_stamp: Stamp,
    pub generation: u64,
}

/// Mutation methods require exclusive ownership so the commit sequencer or
/// recovery fold can publish lifecycle changes atomically with their records.
#[derive(Debug, Clone, Default)]
pub struct ClusterTable { entries: BTreeMap<ClusterId, ClusterEntry> }

impl ClusterTable {
    pub fn new() -> Self { Self::default() }

    pub fn register(&mut self, id: ClusterId, protection: Protection) -> Result<&ClusterEntry, Error> {
        if self.entries.contains_key(&id) {
            return Err(Error::Corrupt { place: "duplicate cluster registration" });
        }
        self.entries.insert(id, ClusterEntry {
            id, protection, state: ClusterState::Live, state_stamp: 0, generation: 1,
        });
        self.entries.get(&id).ok_or(Error::Corrupt { place: "cluster registration insertion" })
    }

    pub fn get(&self, id: ClusterId) -> Option<&ClusterEntry> { self.entries.get(&id) }
    pub fn len(&self) -> usize { self.entries.len() }
    pub fn is_empty(&self) -> bool { self.entries.is_empty() }
    pub fn iter(&self) -> impl Iterator<Item = &ClusterEntry> { self.entries.values() }

    pub fn mark_retired(&mut self, id: ClusterId, stamp: Stamp) -> Result<(), Error> {
        if stamp == 0 { return Err(Error::Limit { limit: "retire stamp must be nonzero" }); }
        let entry = self.entries.get(&id).ok_or(Error::Corrupt { place: "unknown cluster" })?;
        if entry.state == ClusterState::Retired {
            return if entry.state_stamp == stamp { Ok(()) } else { Err(Error::Corrupt { place: "cluster retire stamp changed" }) };
        }
        self.transition(id, ClusterState::Retired, stamp)
    }

    pub fn mark_shredded(&mut self, id: ClusterId) -> Result<(), Error> {
        let entry = self.entries.get(&id).ok_or(Error::Corrupt { place: "unknown cluster" })?;
        if entry.protection != Protection::Sealed {
            return Err(Error::Corrupt { place: "plain cluster cannot be cryptographically shredded" });
        }
        let stamp = entry.state_stamp;
        self.transition(id, ClusterState::Shredded, stamp)
    }

    pub fn mark_purged(&mut self, id: ClusterId) -> Result<(), Error> {
        let entry = self.entries.get(&id).ok_or(Error::Corrupt { place: "unknown cluster" })?;
        if entry.state == ClusterState::Live {
            return Err(Error::Corrupt { place: "live cluster cannot be purged" });
        }
        let stamp = entry.state_stamp;
        self.transition(id, ClusterState::Purged, stamp)
    }

    pub fn require_live(&self, id: ClusterId) -> Result<&ClusterEntry, Error> {
        let entry = self.entries.get(&id).ok_or(Error::UnknownCluster)?;
        if entry.state != ClusterState::Live { return Err(Error::ClusterRetired); }
        Ok(entry)
    }

    fn transition(&mut self, id: ClusterId, next: ClusterState, stamp: Stamp) -> Result<(), Error> {
        let entry = self.entries.get_mut(&id).ok_or(Error::Corrupt { place: "unknown cluster" })?;
        let valid = match (entry.state, next) {
            (ClusterState::Live, ClusterState::Retired) => stamp > entry.state_stamp,
            (ClusterState::Retired, ClusterState::Shredded) => entry.protection == Protection::Sealed,
            (ClusterState::Retired, ClusterState::Purged) => true,
            (ClusterState::Shredded, ClusterState::Purged) => true,
            (current, target) if current == target => true,
            _ => false,
        };
        if !valid { return Err(Error::Corrupt { place: "invalid cluster lifecycle transition" }); }
        if entry.state != next {
            let generation = entry.generation.checked_add(1)
                .ok_or(Error::Limit { limit: "cluster generation exhausted" })?;
            entry.state = next;
            if next == ClusterState::Retired { entry.state_stamp = stamp; }
            entry.generation = generation;
        }
        Ok(())
    }
}
