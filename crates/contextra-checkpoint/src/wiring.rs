//! Wiring entrypoint module for `contextra-checkpoint` composition root integration.
//!
//! Provides [`CheckpointWiring`] and [`CheckpointWiringDeps`] for binding instance-scoped checkpoint
//! operations, orphan recovery, hardlink cloning, and clock injection to higher-ring composition roots.

use crate::orphan::InstanceOrphanRegistry;
use crate::store::PersistentCheckpointStore;
use contextra_mvcc::SnapshotRegistry;
use contextra_ports::Clock;
use contextra_types::{Result, TxId};
use std::path::Path;
use std::sync::Arc;

/// Dependencies required to construct [`CheckpointWiring`].
pub struct CheckpointWiringDeps<S: contextra_ports::StorageEngine> {
    pub store: Arc<PersistentCheckpointStore<S>>,
    pub orphan_registry: Arc<InstanceOrphanRegistry>,
    pub clock: Arc<dyn Clock>,
}

/// Instance-bound wiring container for contextra-checkpoint subsystem.
pub struct CheckpointWiring<S: contextra_ports::StorageEngine> {
    store: Arc<PersistentCheckpointStore<S>>,
    orphan_registry: Arc<InstanceOrphanRegistry>,
    clock: Arc<dyn Clock>,
}

impl<S: contextra_ports::StorageEngine> CheckpointWiring<S> {
    /// Builds a new [`CheckpointWiring`] instance from the provided dependencies.
    pub fn build(deps: CheckpointWiringDeps<S>) -> Result<Self> {
        Ok(Self {
            store: deps.store,
            orphan_registry: deps.orphan_registry,
            clock: deps.clock,
        })
    }

    pub fn store(&self) -> &Arc<PersistentCheckpointStore<S>> {
        &self.store
    }

    pub fn orphan_registry(&self) -> &Arc<InstanceOrphanRegistry> {
        &self.orphan_registry
    }

    pub fn clock(&self) -> &Arc<dyn Clock> {
        &self.clock
    }

    /// Recovers all registered/persisted orphaned sequence pins.
    pub async fn recover_orphaned_pins(&self) -> Result<Vec<u64>> {
        self.store.recover_orphaned_pins().await
    }

    /// Recovers all registered/persisted orphaned checkpoints.
    pub async fn recover_orphaned_checkpoints(&self) -> Result<Vec<TxId>> {
        self.store.recover_orphaned_checkpoints().await
    }

    /// Creates a physical hardlink clone of all SSTable files visible for `seq_no`.
    pub async fn create_hardlink_clone(
        &self,
        seq_no: u64,
        source_dir: &Path,
        target_dir: &Path,
        snapshot_registry: &Arc<SnapshotRegistry>,
    ) -> Result<crate::HardlinkCloneResult> {
        self.store
            .create_hardlink_clone(seq_no, source_dir, target_dir, snapshot_registry)
            .await
    }
}
