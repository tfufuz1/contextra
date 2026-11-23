//! Deterministische Catalog-Faltung für die P1-Op-Teilmenge.

use super::ops::{CatalogOp, FileKind};
use super::record::CatalogRecord;
use contextra_storage_ports::{ClusterId, Error, Protection, SpaceId, Stamp};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileMeta {
    pub file_id: u64,
    pub space: SpaceId,
    pub cluster: Option<ClusterId>,
    pub kind: FileKind,
    pub level: u8,
    pub bytes: u64,
    pub min_stamp: Stamp,
    pub max_stamp: Stamp,
}

#[derive(Debug, Clone, Default)]
pub struct CatalogState {
    spaces: BTreeMap<SpaceId, Protection>,
    clusters: BTreeMap<ClusterId, Protection>,
    files: BTreeMap<u64, FileMeta>,
    replay_floor: (u64, u64),
    redo_mark: (u64, u64, Stamp),
    stamp_limit: Stamp,
    id_limit: u64,
    epoch: u64,
}

impl CatalogState {
    pub fn fold(records: &[CatalogRecord]) -> Result<Self, Error> {
        let mut state = Self::default();
        for record in records {
            state.apply_record(record)?;
        }
        Ok(state)
    }

    pub fn apply_record(&mut self, record: &CatalogRecord) -> Result<(), Error> {
        for op in &record.ops { self.apply(op)?; }
        self.epoch = self.epoch.checked_add(1).ok_or(Error::Limit { limit: "catalog epoch" })?;
        Ok(())
    }

    pub fn apply(&mut self, op: &CatalogOp) -> Result<(), Error> {
        match op {
            CatalogOp::SpaceAdd { space, protection } => match self.spaces.get(space) {
                Some(existing) if existing == protection => Ok(()),
                Some(_) => Err(Error::Corrupt { place: "catalog conflicting space registration" }),
                None => { self.spaces.insert(*space, *protection); Ok(()) }
            },
            CatalogOp::ClusterRegister { cluster, protection } => match self.clusters.get(cluster) {
                Some(existing) if existing == protection => Ok(()),
                Some(_) => Err(Error::Corrupt { place: "catalog conflicting cluster registration" }),
                None => { self.clusters.insert(*cluster, *protection); Ok(()) }
            },
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
                    None => { self.files.insert(*file_id, file); Ok(()) }
                }
            }
            CatalogOp::FilesRemove { file_id } => self.files.remove(file_id)
                .map(|_| ()).ok_or(Error::Corrupt { place: "catalog removes non-live file" }),
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
        }
    }

    pub fn spaces(&self) -> &BTreeMap<SpaceId, Protection> { &self.spaces }
    pub fn clusters(&self) -> &BTreeMap<ClusterId, Protection> { &self.clusters }
    pub fn files(&self) -> &BTreeMap<u64, FileMeta> { &self.files }
    pub fn replay_floor(&self) -> (u64, u64) { self.replay_floor }
    pub fn redo_mark(&self) -> (u64, u64, Stamp) { self.redo_mark }
    pub fn stamp_limit(&self) -> Stamp { self.stamp_limit }
    pub fn id_limit(&self) -> u64 { self.id_limit }
    pub fn epoch(&self) -> u64 { self.epoch }
}
