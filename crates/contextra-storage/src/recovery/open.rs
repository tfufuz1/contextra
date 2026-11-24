//! Deterministic instance open, anchor rollback check, and catalog/cluster/gate reconstruction (SPEC §10.1, OP1..OP10).
#![forbid(unsafe_code)]

use crate::catalog::file::{CatalogFile, OpenMode};
use crate::catalog::reserve::after_reopen;
use crate::codec::format::IntegrityMode;
use crate::config::Config;
use crate::engine::branches::{BranchTable, DEFAULT_MAX_BRANCHES, DEFAULT_MAX_DEPTH};
use crate::erase::cluster_table::ClusterTable;
use crate::erase::gate::GateIndex;
use crate::erase::links::LinkGraph;
use crate::erase::protocol::EraseCoordinator;
use contextra_storage_ports::traits::{Anchor, FileRole, Fs, FsLock, IoCtx};
use contextra_storage_ports::{
    ClusterId, ClusterLifecycle, Error, InstanceId, SpaceId, Stamp,
};
use std::collections::BTreeMap;
use std::path::Path;

pub struct OpenedInstance {
    pub instance_id: InstanceId,
    pub integrity_mode: IntegrityMode,
    pub instance_lock: Box<dyn FsLock>,
    pub catalog: CatalogFile,
    pub clusters: ClusterTable,
    pub gate_index: GateIndex,
    pub links: LinkGraph,
    pub branches: BTreeMap<ClusterId, BranchTable>,
    pub coordinator: EraseCoordinator,
    pub next_stamp: Stamp,
    pub next_file_id: u64,
    pub next_space_id: u64,
    pub space_protections: BTreeMap<SpaceId, contextra_storage_ports::Protection>,
}

pub fn open_instance(
    fs: &dyn Fs,
    root: &Path,
    config: &Config,
    anchor: &dyn Anchor,
    entropy: &dyn contextra_storage_ports::traits::Entropy,
) -> Result<OpenedInstance, Error> {
    let caps = fs.caps();
    if !caps.durable_file_sync || !caps.durable_dir_sync || !caps.atomic_rename {
        return Err(Error::Unsupported {
            what: "filesystem lacks required durability capabilities",
        });
    }

    let instance_lock = fs.lock(
        &root.join("LOCK"),
        IoCtx {
            role: FileRole::Other,
            purpose: "instance_lock",
        },
    )?;

    let authenticated =
        config.catalog_integrity == crate::catalog::CatalogIntegrity::Authenticated;
    if authenticated && (config.catalog_key.is_none() || config.catalog_mac.is_none()) {
        return Err(Error::Limit {
            limit: "catalog key and MAC must be supplied together",
        });
    }
    if !authenticated && (config.catalog_key.is_some() || config.catalog_mac.is_some()) {
        return Err(Error::Limit {
            limit: "catalog keys require authenticated integrity mode",
        });
    }

    let catalog = CatalogFile::open_current(
        fs,
        root,
        OpenMode::ReadWrite,
        config.catalog_integrity,
        config.catalog_key.as_deref(),
        config.catalog_mac.as_deref(),
    )?;

    let anchor_val = anchor.read()?;
    if anchor_val.catalog_epoch > catalog.state().epoch() {
        return Err(Error::Rollback);
    }

    let mut raw_inst = [0u8; 16];
    entropy.fill(&mut raw_inst)?;
    let instance_id = InstanceId::from_wire(raw_inst);

    let (next_stamp, next_file_id, next_space_id) = after_reopen(
        catalog.state().stamp_limit(),
        catalog.state().id_limit(),
        catalog.state().space_id_limit(),
    )?;

    let mut clusters = ClusterTable::new();
    let mut gate_index = GateIndex::new();
    let links = LinkGraph::default();
    let mut branches = BTreeMap::new();
    let mut retired_to_gate = Vec::new();

    for (cid, cmeta) in catalog.state().clusters() {
        clusters.register(*cid, cmeta.protection)?;
        branches.insert(
            *cid,
            BranchTable::new(0, 0, DEFAULT_MAX_BRANCHES, DEFAULT_MAX_DEPTH)?,
        );
        match cmeta.lifecycle {
            ClusterLifecycle::Live => {}
            ClusterLifecycle::Retired { at } => {
                retired_to_gate.push((*cid, at.max(1)));
            }
            ClusterLifecycle::Shredded { at } => {
                retired_to_gate.push((*cid, at.max(1)));
            }
            ClusterLifecycle::Purged { at } => {
                retired_to_gate.push((*cid, at.max(1)));
            }
        }
    }

    for (cid, at) in retired_to_gate {
        let _ = gate_index.apply_cluster_gate(
            &mut clusters,
            &links,
            &BTreeMap::new(),
            &[cid],
            at,
            at,
            crate::erase::protocol::MAX_GATE_CLUSTERS,
        );
    }

    let mut space_protections = BTreeMap::new();
    for (sid, prot) in catalog.state().spaces() {
        space_protections.insert(*sid, *prot);
    }

    let integrity_mode = match config.catalog_integrity {
        crate::catalog::CatalogIntegrity::Plain => IntegrityMode::Plain,
        crate::catalog::CatalogIntegrity::Authenticated => IntegrityMode::Authenticated,
    };

    Ok(OpenedInstance {
        instance_id,
        integrity_mode,
        instance_lock,
        catalog,
        clusters,
        gate_index,
        links,
        branches,
        coordinator: EraseCoordinator::new(),
        next_stamp,
        next_file_id,
        next_space_id,
        space_protections,
    })
}
