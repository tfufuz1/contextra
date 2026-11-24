//! Deterministische Catalog-Faltung für die P1-Op-Teilmenge.

use super::ops::{CatalogOp, FileEntry};
use super::record::CatalogRecord;
use contextra_storage_ports::{ClusterId, ClusterLifecycle, Error, Protection, SpaceId, Stamp, SYSTEM_SPACE_ID_MAX};
use std::collections::BTreeMap;

pub type FileMeta = FileEntry;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CatalogCluster {
    pub protection: Protection,
    pub lifecycle: ClusterLifecycle,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraveyardEntry {
    pub ticket: [u8; 16],
    pub file: FileMeta,
}

#[derive(Debug, Clone)]
pub struct CatalogState {
    spaces: BTreeMap<SpaceId, Protection>,
    clusters: BTreeMap<ClusterId, CatalogCluster>,
    files: BTreeMap<u64, FileMeta>,
    graveyard: BTreeMap<([u8; 16], u64), GraveyardEntry>,
    unlink_acks: std::collections::BTreeSet<([u8; 16], u64)>,
    purge_complete: std::collections::BTreeSet<[u8; 16]>,
    replay_floor: (u64, u64),
    redo_mark: (u64, u64, Stamp),
    stamp_limit: Stamp,
    id_limit: u64,
    space_id_limit: u64,
    epoch: u64,
}

impl Default for CatalogState {
    fn default() -> Self {
        Self { spaces: BTreeMap::new(), clusters: BTreeMap::new(), files: BTreeMap::new(),
            graveyard: BTreeMap::new(), unlink_acks: std::collections::BTreeSet::new(),
            purge_complete: std::collections::BTreeSet::new(), replay_floor: (0, 0),
            redo_mark: (0, 0, 0), stamp_limit: 0, id_limit: 0,
            space_id_limit: SYSTEM_SPACE_ID_MAX, epoch: 0 }
    }
}

impl CatalogState {
    pub fn fold(records: &[CatalogRecord]) -> Result<Self, Error> {
        let mut state = Self::default();
        for record in records {
            state.apply_record(record)?;
        }
        Ok(state)
    }

    pub(crate) fn apply_record(&mut self, record: &CatalogRecord) -> Result<(), Error> {
        for op in &record.ops { self.apply(op)?; }
        self.epoch = self.epoch.checked_add(1).ok_or(Error::Limit { limit: "catalog epoch" })?;
        Ok(())
    }

    pub fn apply(&mut self, op: &CatalogOp) -> Result<(), Error> {
        match op {
            CatalogOp::SpaceAdd { space, protection } => match self.spaces.get(space) {
                _ if space.0 > self.space_id_limit => Err(Error::Corrupt { place: "catalog space id not reserved" }),
                Some(existing) if existing == protection => Ok(()),
                Some(_) => Err(Error::Corrupt { place: "catalog conflicting space registration" }),
                None => { self.spaces.insert(*space, *protection); Ok(()) }
            },
            CatalogOp::ClusterRegister { cluster, protection } => match self.clusters.get(cluster) {
                Some(existing) if existing.protection == *protection => Ok(()),
                Some(_) => Err(Error::Corrupt { place: "catalog conflicting cluster registration" }),
                None => { self.clusters.insert(*cluster, CatalogCluster { protection: *protection, lifecycle: ClusterLifecycle::Live }); Ok(()) }
            },
            CatalogOp::ClusterState { cluster, state } => {
                let entry = self.clusters.get_mut(cluster).ok_or(Error::Corrupt { place: "catalog state for unknown cluster" })?;
                if !valid_cluster_transition(entry.protection, entry.lifecycle, *state) {
                    return Err(Error::Corrupt { place: "catalog cluster lifecycle regression" });
                }
                entry.lifecycle = *state;
                Ok(())
            }
            CatalogOp::FilesAdd { file_id, space, cluster, kind, level, bytes, min_stamp, max_stamp } => {
                if !self.spaces.contains_key(space) { return Err(Error::Corrupt { place: "catalog file references unknown space" }); }
                if let Some(cluster) = cluster {
                    if !self.clusters.contains_key(cluster) { return Err(Error::Corrupt { place: "catalog file references unknown cluster" }); }
                }
                let file = FileMeta { file_id: *file_id, space: *space, cluster: *cluster, kind: *kind,
                    level: *level, bytes: *bytes, min_stamp: *min_stamp, max_stamp: *max_stamp };
                match self.files.get(file_id) {
                    Some(existing) if existing == &file => Ok(()),
                    Some(_) => Err(Error::Corrupt { place: "catalog duplicate file id" }),
                    None => {
                        if cluster.is_some_and(|cluster| {
                            self.clusters.get(&cluster).is_some_and(|entry| entry.lifecycle != ClusterLifecycle::Live)
                        }) {
                            return Err(Error::Corrupt { place: "catalog adds file for retired cluster" });
                        }
                        self.files.insert(*file_id, file);
                        Ok(())
                    }
                }
            }
            CatalogOp::FilesRemove { file_id } => {
                let ticket = [0; 16];
                let key = (ticket, *file_id);
                if self.graveyard.contains_key(&key) { return Ok(()); }
                let file = self.files.remove(file_id).ok_or(Error::Corrupt { place: "catalog removes non-live file" })?;
                self.graveyard.insert(key, GraveyardEntry { ticket, file });
                Ok(())
            }
            CatalogOp::ReplayFloor { segment, offset } => {
                if (*segment, *offset) < self.replay_floor { return Err(Error::Corrupt { place: "catalog replay floor regressed" }); }
                self.replay_floor = (*segment, *offset);
                Ok(())
            }
            CatalogOp::RedoMark { segment, offset, stamp } => {
                if (*segment, *offset, *stamp) < self.redo_mark { return Err(Error::Corrupt { place: "catalog redo mark regressed" }); }
                self.redo_mark = (*segment, *offset, *stamp);
                Ok(())
            }
            CatalogOp::ReserveStamps { through } => {
                if *through < self.stamp_limit { return Err(Error::Corrupt { place: "catalog stamp reserve regressed" }); }
                self.stamp_limit = *through;
                Ok(())
            }
            CatalogOp::ReserveIds { through } => {
                if *through < self.id_limit { return Err(Error::Corrupt { place: "catalog id reserve regressed" }); }
                self.id_limit = *through;
                Ok(())
            }
            CatalogOp::ReserveSpaceIds { through } => {
                if *through < self.space_id_limit { return Err(Error::Corrupt { place: "catalog space id reserve regressed" }); }
                self.space_id_limit = *through;
                Ok(())
            }
            CatalogOp::PurgeBatch { ticket, remove_file_id, replacement } => {
                if self.purge_complete.contains(ticket) { return Err(Error::Corrupt { place: "purge batch after completion" }); }
                let key = (*ticket, *remove_file_id);
                if let Some(existing) = self.graveyard.get(&key) {
                    if replacement.as_ref().is_some_and(|new| self.files.get(&new.file_id) != Some(new)) {
                        return Err(Error::Corrupt { place: "purge batch replay mismatch" });
                    }
                    return if existing.file.file_id == *remove_file_id { Ok(()) } else { Err(Error::Corrupt { place: "purge graveyard identity" }) };
                }
                let old = self.files.get(remove_file_id).cloned().ok_or(Error::Corrupt { place: "purge batch removes non-live file" })?;
                if let Some(new) = replacement {
                    validate_file_refs(&self.spaces, &self.clusters, new)?;
                    if new.file_id == *remove_file_id || self.files.contains_key(&new.file_id) {
                        return Err(Error::Corrupt { place: "purge replacement file id" });
                    }
                }
                self.files.remove(remove_file_id);
                if let Some(new) = replacement { self.files.insert(new.file_id, new.clone()); }
                self.graveyard.insert(key, GraveyardEntry { ticket: *ticket, file: old });
                Ok(())
            }
            CatalogOp::UnlinkAck { ticket, file_id } => {
                let key = (*ticket, *file_id);
                if self.unlink_acks.contains(&key) { return Ok(()); }
                self.graveyard.remove(&key).ok_or(Error::Corrupt { place: "unlink ack without graveyard entry" })?;
                self.unlink_acks.insert(key);
                Ok(())
            }
            CatalogOp::PurgeComplete { ticket } => {
                if self.purge_complete.contains(ticket) { return Ok(()); }
                if self.graveyard.keys().any(|(owner, _)| owner == ticket) {
                    return Err(Error::Corrupt { place: "purge complete with pending graveyard" });
                }
                self.purge_complete.insert(*ticket);
                Ok(())
            }
        }
    }

    pub fn spaces(&self) -> &BTreeMap<SpaceId, Protection> { &self.spaces }
    pub fn clusters(&self) -> &BTreeMap<ClusterId, CatalogCluster> { &self.clusters }
    pub fn files(&self) -> &BTreeMap<u64, FileMeta> { &self.files }
    pub fn graveyard(&self) -> &BTreeMap<([u8; 16], u64), GraveyardEntry> { &self.graveyard }
    pub fn purge_is_complete(&self, ticket: &[u8; 16]) -> bool { self.purge_complete.contains(ticket) }
    pub fn replay_floor(&self) -> (u64, u64) { self.replay_floor }
    pub fn redo_mark(&self) -> (u64, u64, Stamp) { self.redo_mark }
    pub fn stamp_limit(&self) -> Stamp { self.stamp_limit }
    pub fn id_limit(&self) -> u64 { self.id_limit }
    pub fn space_id_limit(&self) -> u64 { self.space_id_limit }
    pub fn epoch(&self) -> u64 { self.epoch }
}

fn valid_cluster_transition(protection: Protection, from: ClusterLifecycle, to: ClusterLifecycle) -> bool {
    if from == to { return true; }
    match (from, to) {
        (ClusterLifecycle::Live, ClusterLifecycle::Retired { at }) => at > 0,
        (ClusterLifecycle::Retired { at: retired }, ClusterLifecycle::Shredded { at }) => {
            protection == Protection::Sealed && at == retired
        }
        (ClusterLifecycle::Retired { at: retired }, ClusterLifecycle::Purged { at }) => at == retired,
        (ClusterLifecycle::Shredded { at: shredded }, ClusterLifecycle::Purged { at }) => at == shredded,
        _ => false,
    }
}

fn validate_file_refs(
    spaces: &BTreeMap<SpaceId, Protection>,
    clusters: &BTreeMap<ClusterId, CatalogCluster>,
    file: &FileMeta,
) -> Result<(), Error> {
    if !spaces.contains_key(&file.space) { return Err(Error::Corrupt { place: "catalog file references unknown space" }); }
    if file.cluster.is_some_and(|cluster| !clusters.contains_key(&cluster)) {
        return Err(Error::Corrupt { place: "catalog file references unknown cluster" });
    }
    if file.min_stamp > file.max_stamp { return Err(Error::Corrupt { place: "catalog file stamp range" }); }
    Ok(())
}
