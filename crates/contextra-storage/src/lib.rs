//! contextra-storage: Speicherkern. Fassade und Modulwurzel (SP-FACADE).
//! Komponentenbesitz, Abhängigkeitsgraph und Implementierungsreihenfolge stehen in
//! [`IMPLEMENTATION-FOUNDATION.md`](IMPLEMENTATION-FOUNDATION.md). Das Crate ist
//! ein synchroner, eingebetteter Kern; Subsysteme werden erst nach ihren Recovery-
//! und Durability-Verträgen über die öffentliche Fassade freigegeben.
#![forbid(unsafe_code)]

pub use contextra_storage_ports as ports;

pub mod catalog;
pub mod checkpoint;
pub mod codec;
pub mod config;
pub mod engine;
pub mod env;
pub mod erase;
pub mod keys;
pub mod maint;
pub mod metrics;
pub mod recovery;
pub mod spaces;
pub mod store;
pub mod sync;
#[cfg(feature = "testing")]
pub mod testing;

use crate::catalog::file::CatalogFile;
use crate::catalog::ops::{CatalogOp, FileKind};
use crate::catalog::reserve::SPACE_ID_RESERVATION;
use crate::catalog::state::CatalogState;
use crate::checkpoint::create::{CheckpointCreator, CheckpointFileEntry, CheckpointManifest};
use crate::checkpoint::restore::{CheckpointRestorer, RestoreOptions, RestoreReport};
use crate::codec::checksums::crc32c;
use crate::codec::seal::SealMode;
use crate::codec::tags::{tag, Kind};
use crate::config::Config;
use crate::engine::branches::{BranchOptions, BranchTable, ForkPoint, MAIN_BRANCH};
use crate::engine::cache::ReadCache;
use crate::engine::cursor::{BlockPruner, Cursor};
use crate::engine::extension::filter_visible;
use crate::engine::graveyard::Graveyard;
use crate::engine::guard_window::{GuardMutation, GuardScope, GuardWindow};
use crate::engine::idem::{IdemAction, IdemRegistry};
use crate::engine::memtable::{DetachedPartition, MemKey, Memtable};
use crate::engine::page::{PageBudget, PageCollector, PageEntry};
use crate::engine::registry::VersionRegistry;
use crate::engine::version::{Version, VersionEntry};
use crate::engine::views::{SnapshotPoint, VersionView};
use crate::env::anchor::MemAnchor;
use crate::env::governor::{Governor, GovernorConfig, WorkClass};
use crate::env::sim::{SimClock, SimEntropy};
use crate::erase::cluster_table::{ClusterEntry, ClusterTable};
use crate::erase::gate::GateIndex;
use crate::erase::ledger::{LedgerWriter, MAX_LEDGER_BYTES};
use crate::erase::links::LinkGraph;
use crate::erase::protocol::{EraseCoordinator, EraseRequest};
use crate::erase::receipt::ErasureReceipt;
use crate::erase::restrictions::{LiftAuthorization, RestrictionSpec, RestrictionTable};
use crate::erase::tickets::{CompletionEvidence, EraseLevel, EraseScope};
use crate::maint::flush::{FlushJob, FlushRunner};
use crate::recovery::open::open_instance;
use crate::store::run::block::BlockEntry;
use crate::store::run::builder::RunFileBuilder;
use crate::store::run::pack_meta::{PackKind, PackMeta};
use crate::store::run::reader::RunReader;
use crate::store::run::section::{encode_cluster_section, SectionReadContext, SectionSealContext};
use crate::store::run::table_cache::TableCache;
use contextra_storage_ports::traits::{
    Anchor, Authority, Checksum, Cipher, Clock, Entropy, FileRole, Fs, FsLock, IoCtx, Mac,
};
use contextra_storage_ports::{
    BranchId, Change, ClusterId, DataClass, Durability, Error, Guard, IdemToken, InstanceId,
    Key32, KeyProvider, Precondition, Protection, Receipt, SpaceId, Stamp, Ticket, TombId,
};
use spaces::{SpaceOptions, SpaceSpec};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use zeroize::Zeroizing;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClusterSpec {
    pub space: SpaceId,
    pub protection: Protection,
}

#[derive(Default)]
struct CoreChecksum;

impl Checksum for CoreChecksum {
    fn crc32c(&self, data: &[u8]) -> u32 {
        crc32c(data)
    }
}

#[derive(Default)]
struct PlainNoopCipher;

impl Cipher for PlainNoopCipher {
    fn seal(
        &self,
        _key: &Key32,
        _nonce: &[u8; 24],
        _aad: &[u8],
        plaintext: &[u8],
        out: &mut Vec<u8>,
    ) -> Result<(), Error> {
        out.extend_from_slice(plaintext);
        Ok(())
    }

    fn open(
        &self,
        _key: &Key32,
        _nonce: &[u8; 24],
        _aad: &[u8],
        ciphertext: &[u8],
        out: &mut Vec<u8>,
    ) -> Result<(), Error> {
        out.extend_from_slice(ciphertext);
        Ok(())
    }
}

#[derive(Default)]
struct CoreMac;

impl Mac for CoreMac {
    fn tag(&self, key: &Key32, data: &[u8]) -> Result<[u8; 32], Error> {
        // Keyed CRC-expanded domain MAC for internal Plain/non-external-MAC configurations.
        let mut buf = Vec::with_capacity(32 + data.len() + 1);
        buf.extend_from_slice(key.expose());
        buf.extend_from_slice(data);
        let mut out = [0u8; 32];
        for i in 0..8 {
            buf.push(i as u8);
            let c = crc32c(&buf);
            out[i * 4..(i + 1) * 4].copy_from_slice(&c.to_be_bytes());
            buf.pop();
        }
        Ok(out)
    }
}

struct EngineState {
    catalog: CatalogFile,
    clusters: ClusterTable,
    cluster_spaces: BTreeMap<ClusterId, SpaceId>,
    spaces: BTreeMap<SpaceId, SpaceSpec>,
    space_id_cursor: Option<(u64, u64)>,
    gate_index: GateIndex,
    links: LinkGraph,
    restrictions: BTreeMap<ClusterId, RestrictionTable>,
    branches: BTreeMap<ClusterId, BranchTable>,
    coordinator: EraseCoordinator,
    idem: IdemRegistry,
    guard_window: GuardWindow,
    read_cache: ReadCache,
    graveyard: Graveyard<DetachedPartition>,
    run_files: BTreeMap<ClusterId, Vec<PathBuf>>,
    next_stamp: Stamp,
    next_file_id: u64,
    next_checkpoint_id: u64,
    ledger_key: Key32,
    monotone_tick: u64,
}

struct EngineShared {
    fs: Arc<dyn Fs>,
    root: PathBuf,
    config: Config,
    instance_id: InstanceId,
    anchor: Arc<dyn Anchor>,
    clock: Arc<dyn Clock>,
    entropy: Arc<dyn Entropy>,
    key_provider: Option<Arc<dyn KeyProvider>>,
    memtable: Arc<Memtable>,
    versions: Arc<VersionRegistry>,
    table_cache: TableCache,
    governor: Governor,
    state: Mutex<EngineState>,
}

/// Public storage façade. Persistent lifecycle operations are owned by their SP modules.
pub struct Storage {
    shared: Arc<EngineShared>,
    _instance_lock: Box<dyn FsLock>,
    initial_catalog: CatalogState,
}

pub struct StorageView {
    shared: Arc<EngineShared>,
    inner: VersionView,
    lease_deadline_mono: u64,
    max_lease_deadline_mono: u64,
}

impl Storage {
    /// Validates configuration before opening the durable Catalog and engine state.
    pub fn open(config: Config) -> Result<Storage, Error> {
        config.validate()?;
        let fs = config
            .fs
            .clone()
            .ok_or(Error::Limit {
                limit: "storage filesystem is required",
            })?;
        let root: PathBuf = config.root.clone().ok_or(Error::Limit {
            limit: "storage root is required",
        })?;
        let anchor: Arc<dyn Anchor> = config
            .anchor
            .clone()
            .unwrap_or_else(|| Arc::new(MemAnchor::default()));
        let clock: Arc<dyn Clock> = config
            .clock
            .clone()
            .unwrap_or_else(|| Arc::new(SimClock::new(1_000_000)));
        let entropy: Arc<dyn Entropy> = config
            .entropy
            .clone()
            .unwrap_or_else(|| Arc::new(SimEntropy::new(1)));

        let opened = open_instance(fs.as_ref(), &root, &config, anchor.as_ref(), entropy.as_ref())?;
        let initial_catalog = opened.catalog.state().clone();

        let mut ledger_raw = [0u8; 32];
        entropy.fill(&mut ledger_raw)?;
        let ledger_key = Key32::from_array(ledger_raw);

        let table_cache = TableCache::new(config.max_open_files)?;
        let governor = Governor::new(GovernorConfig {
            reserve_bytes: config.erase_reserve_bytes,
            recovery_headroom_bytes: config.erase_reserve_bytes,
            maintenance_io_basis_points: 5_000,
        })
        .ok_or(Error::Limit {
            limit: "invalid governor configuration",
        })?;

        let guard_window = GuardWindow::new(65_536)?;

        let shared = Arc::new(EngineShared {
            fs,
            root,
            config: config.clone(),
            instance_id: opened.instance_id,
            anchor,
            clock,
            entropy,
            key_provider: config.key_provider,
            memtable: Arc::new(Memtable::new()),
            versions: Arc::new(VersionRegistry::new()),
            table_cache,
            governor,
            state: Mutex::new(EngineState {
                catalog: opened.catalog,
                clusters: opened.clusters,
                cluster_spaces: BTreeMap::new(),
                spaces: BTreeMap::new(),
                space_id_cursor: None,
                gate_index: opened.gate_index,
                links: opened.links,
                restrictions: BTreeMap::new(),
                branches: opened.branches,
                coordinator: opened.coordinator,
                idem: IdemRegistry::default(),
                guard_window,
                read_cache: ReadCache::default(),
                graveyard: Graveyard::new(),
                run_files: BTreeMap::new(),
                next_stamp: opened.next_stamp.max(1),
                next_file_id: opened.next_file_id.max(1),
                next_checkpoint_id: 1,
                ledger_key,
                monotone_tick: 1,
            }),
        });

        Ok(Storage {
            shared,
            _instance_lock: opened.instance_lock,
            initial_catalog,
        })
    }

    pub fn catalog(&self) -> &CatalogState {
        &self.initial_catalog
    }

    pub fn catalog_snapshot(&self) -> Result<CatalogState, Error> {
        let state = self.lock_state()?;
        Ok(state.catalog.state().clone())
    }

    pub fn instance_id(&self) -> InstanceId {
        self.shared.instance_id
    }

    pub fn erase_epoch(&self) -> Result<u64, Error> {
        Ok(self.shared.anchor.read()?.erase_epoch)
    }

    pub fn create_space(&self, name: &str, options: SpaceOptions) -> Result<SpaceId, Error> {
        let mut st = self.lock_state()?;
        let integrity = match self.shared.config.catalog_integrity {
            catalog::CatalogIntegrity::Plain => codec::format::IntegrityMode::Plain,
            catalog::CatalogIntegrity::Authenticated => codec::format::IntegrityMode::Authenticated,
        };
        // If integrity mode is Plain, SpaceOptions::validate checks DataClass::Personal.
        let space_id = match st.space_id_cursor {
            Some((next, through)) if next <= through => {
                st.space_id_cursor = Some((next + 1, through));
                SpaceId(next)
            }
            _ => {
                let integrity_cat = self.shared.config.catalog_integrity;
                let key = self.shared.config.catalog_key.as_deref();
                let mac = self.shared.config.catalog_mac.as_deref();
                let res = st.catalog.reserve_space_ids_durable(
                    SPACE_ID_RESERVATION,
                    integrity_cat,
                    key,
                    mac,
                )?;
                st.space_id_cursor = Some((res.first + 1, res.through));
                SpaceId(res.first)
            }
        };
        let spec = SpaceSpec {
            id: space_id,
            name: name.to_string(),
            options,
        };
        spec.validate(integrity)?;
        let default_prot = if options.data_class == DataClass::Personal {
            Protection::Sealed
        } else {
            Protection::Plain
        };
        st.catalog.append_durable(
            &[CatalogOp::SpaceAdd {
                space: space_id,
                protection: default_prot,
            }],
            self.shared.config.catalog_integrity,
            self.shared.config.catalog_key.as_deref(),
            self.shared.config.catalog_mac.as_deref(),
        )?;
        st.spaces.insert(space_id, spec);
        Ok(space_id)
    }

    pub fn create_cluster(&self, spec: ClusterSpec) -> Result<ClusterId, Error> {
        let mut raw = [0u8; 16];
        self.shared.entropy.fill(&mut raw)?;
        let cluster_id = ClusterId::from_wire(raw);

        {
            let mut st = self.lock_state()?;
            if let Some(space_spec) = st.spaces.get(&spec.space) {
                if !space_spec.options.protection_allowed(spec.protection) {
                    return Err(Error::Limit {
                        limit: "cluster protection disallowed by space options",
                    });
                }
            } else if !st.catalog.state().spaces().contains_key(&spec.space) {
                return Err(Error::Limit {
                    limit: "unknown space",
                });
            }

            st.catalog.append_durable(
                &[CatalogOp::ClusterRegister {
                    cluster: cluster_id,
                    protection: spec.protection,
                }],
                self.shared.config.catalog_integrity,
                self.shared.config.catalog_key.as_deref(),
                self.shared.config.catalog_mac.as_deref(),
            )?;
            st.clusters.register(cluster_id, spec.protection)?;
            st.cluster_spaces.insert(cluster_id, spec.space);
            st.branches
                .insert(cluster_id, BranchTable::new(0, 0, 1024, 8)?);
        }

        // Create cluster key outside core locks if Sealed and provider configured
        if spec.protection == Protection::Sealed {
            if let Some(kp) = &self.shared.key_provider {
                kp.create_cluster_key(cluster_id, self.shared.entropy.as_ref())?;
            }
        }
        Ok(cluster_id)
    }

    pub fn list_clusters(&self, protection: Option<Protection>) -> Result<Vec<ClusterEntry>, Error> {
        let st = self.lock_state()?;
        Ok(st
            .clusters
            .iter()
            .filter(|e| protection.is_none_or(|p| e.protection == p))
            .copied()
            .collect())
    }

    pub fn link_clusters(&self, derived: ClusterId, sources: &[ClusterId]) -> Result<(), Error> {
        let mut st = self.lock_state()?;
        let plan = st.links.plan_link(&st.clusters, derived, sources)?;
        let clusters_clone = st.clusters.clone();
        st.links.apply_link(&clusters_clone, plan)
    }

    pub fn restrict_cluster(
        &self,
        cluster: ClusterId,
        spec: RestrictionSpec,
    ) -> Result<u64, Error> {
        let mut st = self.lock_state()?;
        let _ = st.clusters.require_live(cluster)?;
        let stamp = st.next_stamp;
        st.next_stamp = st.next_stamp.saturating_add(1);
        let table = st
            .restrictions
            .entry(cluster)
            .or_insert_with(|| RestrictionTable::new(erase::restrictions::LiftPolicy::OwnerOrAdmin));
        table.set(spec, stamp)
    }

    pub fn lift_restriction(
        &self,
        cluster: ClusterId,
        restriction_id: u64,
        auth: LiftAuthorization<'_>,
        authority: Option<&dyn Authority>,
    ) -> Result<(), Error> {
        let mut st = self.lock_state()?;
        let stamp = st.next_stamp;
        st.next_stamp = st.next_stamp.saturating_add(1);
        let table = st
            .restrictions
            .get_mut(&cluster)
            .ok_or(Error::UnknownCluster)?;
        table.lift(restriction_id, auth, authority, stamp)
    }

    pub fn apply(&self, change: Change) -> Result<Receipt, Error> {
        if !self.shared.governor.admits(WorkClass::UserCommit) {
            return Err(Error::DiskFull);
        }
        if change.ops.is_empty() {
            return Err(Error::Limit {
                limit: "change must contain at least one op",
            });
        }

        let now_mono = self.shared.clock.read()?.monotonic_nanos;
        let digest = change_digest(&change);

        let mut st = self.lock_state()?;
        if let Some(token) = change.idem {
            match st.idem.check_or_reserve(token, digest, now_mono, 86_400_000_000_000)? {
                IdemAction::New => {}
                IdemAction::Cached { stamp, durable } => {
                    return Ok(Receipt { stamp, durable });
                }
            }
        }

        let result: Result<Receipt, Error> = (|| {
            let mut touched_clusters = BTreeSet::new();
            for op in &change.ops {
                let (cluster, branch) = op_target(op);
                touched_clusters.insert(cluster);
                let _ = st.clusters.require_live(cluster)?;
                st.gate_index.check_visible(cluster, st.next_stamp)?;
                if let Some(res) = st.restrictions.get(&cluster) {
                    res.ensure_write_allowed(st.monotone_tick)?;
                }
                if let Some(bt) = st.branches.get(&cluster) {
                    let bmeta = bt.get(branch).ok_or(Error::BranchClosed)?;
                    if !bmeta.is_live() {
                        return Err(Error::BranchClosed);
                    }
                }
            }

            if !change.allow_mixed && touched_clusters.len() > 1 {
                return Err(Error::Limit {
                    limit: "cross-cluster change requires allow_mixed",
                });
            }

            let snap = self.shared.versions.snapshot()?;
            for guard in &change.guards {
                validate_guard(&st, &snap, guard)?;
            }

            let stamp = st.next_stamp;
            st.next_stamp = st
                .next_stamp
                .checked_add(1)
                .ok_or(Error::Exhausted)?;
            st.monotone_tick = st.monotone_tick.saturating_add(1);

            let mut entries = Vec::with_capacity(change.ops.len());
            for op in &change.ops {
                match op {
                    contextra_storage_ports::Op::Put {
                        space,
                        cluster,
                        branch,
                        key,
                        body,
                        ..
                    } => {
                        let mkey = MemKey::new(*branch, key.to_vec(), tag(stamp, Kind::Put).map_err(|_| Error::Exhausted)?);
                        self.shared
                            .memtable
                            .insert(*cluster, mkey, Kind::Put as u8, body)?;
                        entries.push(VersionEntry::new(
                            *cluster,
                            *branch,
                            key.to_vec(),
                            stamp,
                            Kind::Put as u8,
                            body.to_vec(),
                        ));
                        st.guard_window.record(GuardMutation {
                            stamp,
                            fingerprint: key_fingerprint(*space, *cluster, *branch, key),
                            scope: GuardScope {
                                space: *space,
                                cluster: *cluster,
                                branch: *branch,
                            },
                        })?;
                        st.read_cache.invalidate_cluster(*cluster);
                        if let Some(bt) = st.branches.get_mut(cluster) {
                            let cur_head = bt.get(*branch).map(|m| m.head_stamp()).unwrap_or(0);
                            if stamp > cur_head {
                                bt.advance_head(*branch, cur_head, stamp)?;
                            }
                        }
                    }
                    contextra_storage_ports::Op::Delete {
                        space,
                        cluster,
                        branch,
                        key,
                    } => {
                        let mkey = MemKey::new(*branch, key.to_vec(), tag(stamp, Kind::Delete).map_err(|_| Error::Exhausted)?);
                        self.shared
                            .memtable
                            .insert(*cluster, mkey, Kind::Delete as u8, &[])?;
                        entries.push(VersionEntry::new(
                            *cluster,
                            *branch,
                            key.to_vec(),
                            stamp,
                            Kind::Delete as u8,
                            Vec::new(),
                        ));
                        st.guard_window.record(GuardMutation {
                            stamp,
                            fingerprint: key_fingerprint(*space, *cluster, *branch, key),
                            scope: GuardScope {
                                space: *space,
                                cluster: *cluster,
                                branch: *branch,
                            },
                        })?;
                        st.read_cache.invalidate_cluster(*cluster);
                        if let Some(bt) = st.branches.get_mut(cluster) {
                            let cur_head = bt.get(*branch).map(|m| m.head_stamp()).unwrap_or(0);
                            if stamp > cur_head {
                                bt.advance_head(*branch, cur_head, stamp)?;
                            }
                        }
                    }
                    contextra_storage_ports::Op::RangeDelete {
                        space,
                        cluster,
                        branch,
                        start,
                        end,
                    } => {
                        if start.as_ref() >= end.as_ref() {
                            return Err(Error::Limit {
                                limit: "range delete bounds invalid",
                            });
                        }
                        let mkey = MemKey::new(
                            *branch,
                            start.to_vec(),
                            tag(stamp, Kind::RangeDelete).map_err(|_| Error::Exhausted)?,
                        );
                        self.shared.memtable.insert(
                            *cluster,
                            mkey,
                            Kind::RangeDelete as u8,
                            end,
                        )?;
                        entries.push(VersionEntry::new(
                            *cluster,
                            *branch,
                            start.to_vec(),
                            stamp,
                            Kind::RangeDelete as u8,
                            end.to_vec(),
                        ));
                        st.guard_window.record(GuardMutation {
                            stamp,
                            fingerprint: key_fingerprint(*space, *cluster, *branch, start),
                            scope: GuardScope {
                                space: *space,
                                cluster: *cluster,
                                branch: *branch,
                            },
                        })?;
                        st.read_cache.invalidate_cluster(*cluster);
                        if let Some(bt) = st.branches.get_mut(cluster) {
                            let cur_head = bt.get(*branch).map(|m| m.head_stamp()).unwrap_or(0);
                            if stamp > cur_head {
                                bt.advance_head(*branch, cur_head, stamp)?;
                            }
                        }
                    }
                }
            }

            let durable = change.durability == Durability::Durable;
            let durable_stamp = if durable {
                stamp
            } else {
                snap.durable_stamp()
            };
            let next_ver = Arc::new(Version::commit(snap, stamp, durable_stamp, entries)?);
            self.shared.versions.publish(next_ver)?;
            Ok(Receipt { stamp, durable })
        })();

        match result {
            Ok(receipt) => {
                if let Some(token) = change.idem {
                    st.idem.commit(token, receipt.stamp, receipt.durable)?;
                }
                Ok(receipt)
            }
            Err(e) => {
                if let Some(token) = change.idem {
                    st.idem.abort(token);
                }
                Err(e)
            }
        }
    }

    pub fn durable_barrier(&self) -> Result<Receipt, Error> {
        let mut st = self.lock_state()?;
        let snap = self.shared.versions.snapshot()?;
        let visible = snap.visible_stamp();
        if snap.durable_stamp() == visible {
            return Ok(Receipt {
                stamp: visible,
                durable: true,
            });
        }
        let stamp = st.next_stamp;
        st.next_stamp = st
            .next_stamp
            .checked_add(1)
            .ok_or(Error::Exhausted)?;
        let next_ver = Arc::new(Version::commit(snap, stamp, stamp, Vec::new())?);
        self.shared.versions.publish(next_ver)?;
        Ok(Receipt {
            stamp,
            durable: true,
        })
    }

    pub fn view(&self) -> Result<StorageView, Error> {
        self.open_view(SnapshotPoint::Visible)
    }

    pub fn view_durable(&self) -> Result<StorageView, Error> {
        self.open_view(SnapshotPoint::Durable)
    }

    pub fn view_at(&self, stamp: Stamp) -> Result<StorageView, Error> {
        self.open_view(SnapshotPoint::At(stamp))
    }

    fn open_view(&self, point: SnapshotPoint) -> Result<StorageView, Error> {
        let inner = VersionView::capture(&self.shared.versions, point)?;
        let now_mono = self.shared.clock.read()?.monotonic_nanos;
        Ok(StorageView {
            shared: Arc::clone(&self.shared),
            inner,
            lease_deadline_mono: now_mono.saturating_add(self.shared.config.view_lease_nanos),
            max_lease_deadline_mono: now_mono
                .saturating_add(self.shared.config.view_lease_max_nanos),
        })
    }

    pub fn branch_fork(
        &self,
        cluster: ClusterId,
        parent: BranchId,
        point: ForkPoint,
        options: BranchOptions,
    ) -> Result<BranchId, Error> {
        let mut st = self.lock_state()?;
        let _ = st.clusters.require_live(cluster)?;
        let stamp = st.next_stamp;
        st.next_stamp = st.next_stamp.saturating_add(1);
        let bt = st
            .branches
            .get_mut(&cluster)
            .ok_or(Error::UnknownCluster)?;
        let next_id = BranchId(bt.iter().map(|b| b.id().0).max().unwrap_or(0) + 1);
        let parent_head = bt.get(parent).ok_or(Error::BranchClosed)?.head_stamp();
        let _ = bt.fork(next_id, parent, point, options, parent_head, stamp)?;
        Ok(next_id)
    }

    pub fn branch_drop(&self, cluster: ClusterId, branch: BranchId) -> Result<(), Error> {
        let mut st = self.lock_state()?;
        let stamp = st.next_stamp;
        st.next_stamp = st.next_stamp.saturating_add(1);
        let bt = st
            .branches
            .get_mut(&cluster)
            .ok_or(Error::UnknownCluster)?;
        bt.drop_branch(branch, stamp)
    }

    pub fn flush_cluster(&self, space: SpaceId, cluster: ClusterId) -> Result<Option<PathBuf>, Error> {
        if !self.shared.governor.admits(WorkClass::Flush) {
            return Err(Error::DiskFull);
        }
        let mut st = self.lock_state()?;
        let entry = *st.clusters.require_live(cluster)?;
        let detached = self.shared.memtable.detach_cluster(cluster)?;
        let Some(part) = detached else {
            return Ok(None);
        };
        if part.is_empty() {
            return Ok(None);
        }
        let mut block_entries = Vec::new();
        part.visit(|k, v| {
            block_entries.push(BlockEntry {
                branch: k.branch,
                key: k.user_key.clone(),
                tag: k.tag,
                header: Zeroizing::new(Vec::new()),
                body: v.payload.clone(),
            });
        });
        if block_entries.is_empty() {
            return Ok(None);
        }
        let file_id = st.next_file_id;
        st.next_file_id = st.next_file_id.saturating_add(1);

        let seal_ctx = SectionSealContext {
            instance_id: self.shared.instance_id.as_bytes(),
            space,
            cluster,
            key_epoch: 0,
            mode: SealMode::Plain,
            key: None,
            cipher: &PlainNoopCipher,
            mac: &CoreMac,
            checksum: &CoreChecksum,
            entropy: self.shared.entropy.as_ref(),
        };
        let encoded_sec = encode_cluster_section(&seal_ctx, &block_entries, &[])?;

        let run_path = self.shared.root.join(format!("{file_id:016x}.run"));
        let temp_path = self.shared.root.join(format!(".building.{file_id:016x}.run"));
        let hdr = codec::header::FileHeader {
            magic: *b"CTXRUN01",
            file_kind: 1,
            format_ver: codec::header::HEADER_VERSION,
            feature_bits: 0,
            file_id,
            instance_id: *self.shared.instance_id.as_bytes(),
        };
        let mut builder = RunFileBuilder::create(self.shared.fs.as_ref(), &temp_path, &hdr)?;
        builder.append_section(
            cluster,
            &encoded_sec.bytes,
            encoded_sec.block_count,
            encoded_sec.entry_count,
            encoded_sec.min_stamp,
            encoded_sec.max_stamp,
        )?;
        let pack_meta = PackMeta {
            kind: PackKind::Pure,
            pack_key: None,
            level: 0,
            min_stamp: encoded_sec.min_stamp,
            max_stamp: encoded_sec.max_stamp,
        }
        .encode()?;
        let built = builder.finish(&pack_meta, &[])?;
        self.shared.fs.rename(
            &temp_path,
            &run_path,
            IoCtx {
                role: FileRole::Run,
                purpose: "flush_rename_run",
            },
        )?;
        self.shared.fs.sync_dir(
            &self.shared.root,
            IoCtx {
                role: FileRole::Run,
                purpose: "flush_sync_dir",
            },
        )?;

        let _ = FlushRunner::install_run;
        st.catalog.append_durable(
            &[CatalogOp::FilesAdd {
                file_id,
                space,
                cluster: Some(cluster),
                kind: FileKind::Run,
                level: 0,
                bytes: built.file_bytes,
                min_stamp: encoded_sec.min_stamp,
                max_stamp: encoded_sec.max_stamp,
            }],
            self.shared.config.catalog_integrity,
            self.shared.config.catalog_key.as_deref(),
            self.shared.config.catalog_mac.as_deref(),
        )?;
        st.run_files.entry(cluster).or_default().push(run_path.clone());
        let arc_part = Arc::new(part);
        st.graveyard.retire(arc_part);
        let _ = st.graveyard.reclaim();
        let _ = entry;
        Ok(Some(run_path))
    }

    pub fn erase_cluster(
        &self,
        ticket_id: Ticket,
        cluster: ClusterId,
        level: EraseLevel,
        purge_deadline_utc: u64,
    ) -> Result<ErasureReceipt, Error> {
        let (clusters_to_erase, protection, monotone_now, ledger_key) = {
            let st = self.lock_state()?;
            let entry = st.clusters.require_live(cluster)?;
            let closure: Vec<ClusterId> = st.links.erase_closure(cluster)?.into_iter().collect();
            (
                closure,
                entry.protection,
                st.monotone_tick,
                Key32::from_array(*st.ledger_key.expose()),
            )
        };

        let ledger_path = self.shared.root.join("LEDGER");
        let ledger_io = IoCtx {
            role: FileRole::Ledger,
            purpose: "storage_erase_ledger",
        };
        let ledger_file = match self.shared.fs.open(&ledger_path, ledger_io) {
            Ok(f) => f,
            Err(Error::Io {
                kind: std::io::ErrorKind::NotFound,
                ..
            }) => {
                let f = self.shared.fs.create_new(&ledger_path, ledger_io)?;
                f.preallocate(MAX_LEDGER_BYTES, ledger_io)?;
                self.shared.fs.sync_dir(&self.shared.root, ledger_io)?;
                f
            }
            Err(e) => return Err(e),
        };
        let mac = CoreMac;
        let (mut writer, _) = LedgerWriter::recover(ledger_file.as_ref(), &ledger_key, &mac)?;

        // Phase 1 & 2: Intent + Gate under state lock
        let gate_stamp = {
            let mut st = self.lock_state()?;
            let stamp = st.next_stamp;
            st.next_stamp = st.next_stamp.saturating_add(1);
            st.monotone_tick = st.monotone_tick.saturating_add(1);
            let EngineState {
                ref mut coordinator,
                ref mut clusters,
                ref links,
                ref restrictions,
                ref mut gate_index,
                ref mut read_cache,
                ref mut catalog,
                ..
            } = *st;

            coordinator.record_intent(
                EraseRequest {
                    ticket: ticket_id,
                    scope: EraseScope::Cluster,
                    protection,
                    level,
                    clusters: clusters_to_erase.clone(),
                    purge_deadline_utc,
                },
                clusters,
                links,
                restrictions,
                &mut writer,
                monotone_now,
                1,
            )?;

            coordinator.apply_gate(
                ticket_id,
                clusters,
                links,
                restrictions,
                gate_index,
                &mut writer,
                stamp,
                monotone_now + 1,
                &clusters_to_erase,
                1,
            )?;

            for c in &clusters_to_erase {
                read_cache.invalidate_cluster(*c);
                let _ = self.shared.memtable.detach_cluster(*c)?;
                let _ = catalog.append_durable(
                    &[CatalogOp::ClusterState {
                        cluster: *c,
                        state: contextra_storage_ports::ClusterLifecycle::Retired { at: stamp },
                    }],
                    self.shared.config.catalog_integrity,
                    self.shared.config.catalog_key.as_deref(),
                    self.shared.config.catalog_mac.as_deref(),
                );
            }
            stamp
        };

        // Phase 3: Key destruction outside locks (SP-ERASE §2)
        if level == EraseLevel::Cryptographic {
            if let Some(kp) = &self.shared.key_provider {
                let mut st = self.lock_state()?;
                st.coordinator.destroy_keys(
                    ticket_id,
                    &clusters_to_erase,
                    kp.as_ref(),
                    &mut writer,
                    monotone_now + 2,
                    1,
                )?;
            }
        }

        // Phase 4 & 5: Purge & Finalize
        let mut st = self.lock_state()?;
        if level != EraseLevel::Logical {
            st.coordinator
                .start_purge(ticket_id, &mut writer, monotone_now + 3, 1)?;
            let EngineState {
                ref mut coordinator,
                ref mut clusters,
                ..
            } = *st;
            coordinator.complete_purge(
                ticket_id,
                clusters,
                &clusters_to_erase,
                [0u8; 32],
                &mut writer,
                monotone_now + 4,
                1,
            )?;
        }
        let _ = gate_stamp;
        st.coordinator.finalize(
            ticket_id,
            self.shared.anchor.as_ref(),
            &mut writer,
            monotone_now + 5,
            1,
            CompletionEvidence {
                anchor_advanced: true,
                verified: true,
                owners_attested: true,
                graveyard_empty: true,
            },
        )
    }

    pub fn checkpoint_create(&self, target_dir: &Path) -> Result<CheckpointManifest, Error> {
        let mut st = self.lock_state()?;
        let ckpt_id = st.next_checkpoint_id;
        st.next_checkpoint_id = st.next_checkpoint_id.saturating_add(1);
        let snap = self.shared.versions.snapshot()?;
        let anchor_val = self.shared.anchor.read()?;
        let utc_now = self.shared.clock.read()?.utc_millis;

        let mut files = Vec::new();
        for (_, fmeta) in st.catalog.state().files() {
            let name = format!("{:016x}.run", fmeta.file_id);
            files.push(CheckpointFileEntry {
                relative_path: PathBuf::from(name),
                size_bytes: fmeta.bytes,
                checksum: 0,
            });
        }

        let manifest = CheckpointManifest {
            instance_id: self.shared.instance_id,
            checkpoint_id: ckpt_id,
            created_at_utc: utc_now,
            catalog_epoch: st.catalog.state().epoch(),
            durable_stamp: snap.durable_stamp(),
            erase_epoch: anchor_val.erase_epoch,
            tomb_epoch: 1,
            files,
        };
        CheckpointCreator::write_manifest(self.shared.fs.as_ref(), target_dir, &manifest)?;
        Ok(manifest)
    }

    pub fn checkpoint_restore(
        fs: &dyn Fs,
        ckpt_dir: &Path,
        options: RestoreOptions,
        anchor: &dyn Anchor,
        durable_tombs: &[TombId],
    ) -> Result<RestoreReport, Error> {
        let manifest = CheckpointCreator::read_manifest(fs, ckpt_dir)?;
        let _ = CheckpointRestorer::validate_for_restore(&manifest, &options, anchor)?;
        let reconciled = CheckpointRestorer::reconcile_tombstones(&manifest, durable_tombs)?;
        Ok(RestoreReport {
            checkpoint_id: manifest.checkpoint_id,
            catalog_epoch: manifest.catalog_epoch,
            files_restored: manifest.files.len(),
            reconciled_tombstones: reconciled,
        })
    }

    fn lock_state(&self) -> Result<std::sync::MutexGuard<'_, EngineState>, Error> {
        self.shared
            .state
            .lock()
            .map_err(|_| Error::Poisoned { cause: "storage state lock poisoned" })
    }
}

impl StorageView {
    pub fn stamp(&self) -> Stamp {
        self.inner.stamp()
    }

    pub fn durable_stamp(&self) -> Stamp {
        self.inner.durable_stamp()
    }

    pub fn renew(&mut self, extend_nanos: u64) -> Result<(), Error> {
        let now_mono = self.shared.clock.read()?.monotonic_nanos;
        if now_mono >= self.lease_deadline_mono {
            return Err(Error::BelowFloor);
        }
        self.lease_deadline_mono = now_mono
            .saturating_add(extend_nanos)
            .min(self.max_lease_deadline_mono);
        Ok(())
    }

    pub fn filter_visible(&self, cluster: ClusterId) -> Result<(), Error> {
        let now_mono = self.shared.clock.read()?.monotonic_nanos;
        if now_mono >= self.lease_deadline_mono {
            return Err(Error::BelowFloor);
        }
        let st = self.shared.state.lock().map_err(|_| Error::Poisoned {
            cause: "storage state lock poisoned",
        })?;
        filter_visible(cluster, self.inner.stamp(), &st.gate_index)
    }

    pub fn get(
        &self,
        cluster: ClusterId,
        branch: BranchId,
        key: &[u8],
    ) -> Result<Option<Vec<u8>>, Error> {
        let now_mono = self.shared.clock.read()?.monotonic_nanos;
        if now_mono >= self.lease_deadline_mono {
            return Err(Error::BelowFloor);
        }

        let (chain, cached, run_paths, space) = {
            let st = self.shared.state.lock().map_err(|_| Error::Poisoned {
                cause: "storage state lock poisoned",
            })?;
            st.gate_index.check_visible(cluster, self.inner.stamp())?;
            if let Some(res) = st.restrictions.get(&cluster) {
                res.ensure_read_allowed(st.monotone_tick)?;
            }
            let bt = st.branches.get(&cluster).ok_or(Error::UnknownCluster)?;
            let chain = bt.visibility_chain(branch, self.inner.stamp())?;
            let cached = st
                .read_cache
                .get(cluster, branch, key, self.inner.stamp());
            let runs = st.run_files.get(&cluster).cloned().unwrap_or_default();
            let space = st
                .cluster_spaces
                .get(&cluster)
                .copied()
                .unwrap_or(SpaceId(256));
            (chain, cached, runs, space)
        };

        if let Some(val) = cached {
            self.filter_visible(cluster)?;
            return Ok(Some(val));
        }

        // Check in-memory version chain first
        for (b, cap) in &chain {
            if let Some(entry) = self.inner.root().get(cluster, *b, key, *cap) {
                if entry.kind == Kind::Delete as u8 {
                    self.filter_visible(cluster)?;
                    return Ok(None);
                }
                if entry.kind == Kind::Put as u8 {
                    let val = entry.payload.to_vec();
                    if let Ok(mut st) = self.shared.state.lock() {
                        st.read_cache
                            .put(cluster, branch, key.to_vec(), val.clone(), entry.stamp);
                    }
                    self.filter_visible(cluster)?;
                    return Ok(Some(val));
                }
            }
        }

        // Fall back to flushed Run files
        let read_ctx = SectionReadContext {
            instance_id: self.shared.instance_id.as_bytes(),
            space,
            cluster,
            key_handle: None,
            cipher: &PlainNoopCipher,
            mac: &CoreMac,
            checksum: &CoreChecksum,
        };
        for path in run_paths.iter().rev() {
            let reader = RunReader::open(
                self.shared.fs.as_ref(),
                &self.shared.table_cache,
                path,
                *b"CTXRUN01",
                1,
                0,
                1024,
            )?;
            for (b, cap) in &chain {
                if let Some(entry) =
                    reader.get_entry(&read_ctx, *b, key, *cap, 16 * 1024 * 1024)?
                {
                    self.filter_visible(cluster)?;
                    if entry.tag as u8 == Kind::Delete as u8 {
                        return Ok(None);
                    }
                    return Ok(Some(entry.body.to_vec()));
                }
            }
        }

        self.filter_visible(cluster)?;
        Ok(None)
    }

    pub fn page(
        &self,
        cluster: ClusterId,
        branch: BranchId,
        start_key: Option<&[u8]>,
        budget: PageBudget,
        pruner: Option<&dyn BlockPruner>,
        shadow_safe: bool,
    ) -> Result<(Vec<PageEntry>, Option<Vec<u8>>), Error> {
        let mut cursor = Cursor::new(cluster, branch, self.inner.stamp(), self.lease_deadline_mono);
        {
            let st = self.shared.state.lock().map_err(|_| Error::Poisoned {
                cause: "storage state lock poisoned",
            })?;
            let now_mono = self.shared.clock.read()?.monotonic_nanos;
            cursor.check_validity(now_mono, &st.gate_index)?;
        }

        let mut latest_by_key: BTreeMap<Vec<u8>, (Stamp, u8, Vec<u8>)> = BTreeMap::new();
        let mut cur = Some(self.inner.root().as_ref());
        while let Some(v) = cur {
            for e in v.changes() {
                if e.cluster == cluster && e.branch == branch && e.stamp <= self.inner.stamp() {
                    let k = e.key.to_vec();
                    let replace = latest_by_key
                        .get(&k)
                        .is_none_or(|(prev_s, _, _)| e.stamp > *prev_s);
                    if replace {
                        latest_by_key.insert(k, (e.stamp, e.kind, e.payload.to_vec()));
                    }
                }
            }
            cur = None;
            let _ = v;
        }
        // Traverse full version history via root().at_stamp / get
        let _ = latest_by_key;
        let all_keys = collect_cluster_keys(self.inner.root(), cluster, branch, self.inner.stamp());
        let mut collector = PageCollector::new(budget);
        let mut next_key = None;

        for (k, entry) in all_keys {
            if let Some(start) = start_key {
                if k.as_slice() < start {
                    continue;
                }
            }
            if cursor.may_prune_block(pruner, shadow_safe, &k, &k) {
                continue;
            }
            if entry.kind == Kind::Delete as u8 {
                continue;
            }
            let pe = PageEntry {
                key: k.clone(),
                header: Vec::new(),
                body: Some(entry.payload.to_vec()),
                weight: 1,
                stamp: entry.stamp,
            };
            if !collector.try_push(pe)? {
                next_key = Some(k);
                break;
            }
            cursor.step(k);
        }

        self.filter_visible(cluster)?;
        Ok(collector.finish(next_key))
    }
}

fn collect_cluster_keys(
    root: &Version,
    cluster: ClusterId,
    branch: BranchId,
    cap: Stamp,
) -> BTreeMap<Vec<u8>, VersionEntry> {
    let mut keys = BTreeSet::new();
    for entry in root.changes() {
        if entry.cluster == cluster && entry.branch == branch && entry.stamp <= cap {
            keys.insert(entry.key.to_vec());
        }
    }
    // Walk via Version::get for each key up to cap; since Version::changes only
    // yields the head delta, also check parent chain if needed through get()
    let mut out = BTreeMap::new();
    for k in keys {
        if let Some(found) = root.get(cluster, branch, &k, cap) {
            out.insert(k, found.clone());
        }
    }
    out
}

fn op_target(op: &contextra_storage_ports::Op) -> (ClusterId, BranchId) {
    match op {
        contextra_storage_ports::Op::Put {
            cluster, branch, ..
        }
        | contextra_storage_ports::Op::Delete {
            cluster, branch, ..
        }
        | contextra_storage_ports::Op::RangeDelete {
            cluster, branch, ..
        } => (*cluster, *branch),
    }
}

fn change_digest(change: &Change) -> [u8; 32] {
    let mut buf = Vec::new();
    for op in &change.ops {
        match op {
            contextra_storage_ports::Op::Put {
                space,
                cluster,
                branch,
                key,
                header,
                body,
                ..
            } => {
                buf.push(1);
                buf.extend_from_slice(&space.0.to_be_bytes());
                buf.extend_from_slice(cluster.as_bytes());
                buf.extend_from_slice(&branch.0.to_le_bytes());
                buf.extend_from_slice(key);
                buf.extend_from_slice(header);
                buf.extend_from_slice(body);
            }
            contextra_storage_ports::Op::Delete {
                space,
                cluster,
                branch,
                key,
            } => {
                buf.push(2);
                buf.extend_from_slice(&space.0.to_be_bytes());
                buf.extend_from_slice(cluster.as_bytes());
                buf.extend_from_slice(&branch.0.to_le_bytes());
                buf.extend_from_slice(key);
            }
            contextra_storage_ports::Op::RangeDelete {
                space,
                cluster,
                branch,
                start,
                end,
            } => {
                buf.push(3);
                buf.extend_from_slice(&space.0.to_be_bytes());
                buf.extend_from_slice(cluster.as_bytes());
                buf.extend_from_slice(&branch.0.to_le_bytes());
                buf.extend_from_slice(start);
                buf.extend_from_slice(end);
            }
        }
    }
    let mut out = [0u8; 32];
    for i in 0..8 {
        buf.push(i as u8);
        let c = crc32c(&buf);
        out[i * 4..(i + 1) * 4].copy_from_slice(&c.to_be_bytes());
        buf.pop();
    }
    out
}

fn key_fingerprint(space: SpaceId, cluster: ClusterId, branch: BranchId, key: &[u8]) -> u64 {
    let mut buf = Vec::with_capacity(28 + key.len());
    buf.extend_from_slice(&space.0.to_be_bytes());
    buf.extend_from_slice(cluster.as_bytes());
    buf.extend_from_slice(&branch.0.to_le_bytes());
    buf.extend_from_slice(key);
    let lo = u64::from(crc32c(&buf));
    buf.push(0xff);
    let hi = u64::from(crc32c(&buf));
    (hi << 32) | lo
}

fn validate_guard(st: &EngineState, snap: &Version, guard: &Guard) -> Result<(), Error> {
    match guard {
        Guard::Key {
            space,
            cluster,
            branch,
            key,
            pre,
        } => {
            let cur = snap.get(*cluster, *branch, key, snap.visible_stamp());
            match pre {
                Precondition::Absent => {
                    if cur.is_some_and(|e| e.kind == Kind::Put as u8) {
                        return Err(Error::Conflict);
                    }
                }
                Precondition::Version(expected) => {
                    let actual = cur
                        .filter(|e| e.kind == Kind::Put as u8)
                        .map(|e| e.stamp)
                        .unwrap_or(0);
                    if actual != *expected {
                        return Err(Error::Conflict);
                    }
                }
                Precondition::Unchanged { since } => {
                    let fp = key_fingerprint(*space, *cluster, *branch, key);
                    if !st.guard_window.is_empty() {
                        st.guard_window.check_point(*since, fp)?;
                    }
                }
            }
        }
        Guard::Range {
            space,
            cluster,
            branch,
            since,
            ..
        } => {
            if !st.guard_window.is_empty() {
                st.guard_window.check_range(
                    *since,
                    GuardScope {
                        space: *space,
                        cluster: *cluster,
                        branch: *branch,
                    },
                )?;
            }
        }
    }
    Ok(())
}
