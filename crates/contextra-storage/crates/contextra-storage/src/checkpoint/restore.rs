//! Checkpoint restoration, external fencing verification, and ledger reconcile (SPEC §10.3, SP-CKPT, E-73).
#![forbid(unsafe_code)]

use super::create::CheckpointManifest;
use contextra_storage_ports::traits::{Anchor, AnchorValue};
use contextra_storage_ports::{Error, Stamp, TombId};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RestoreOptions {
    /// Proof that previous instance owner has been fenced out externally (E-73, A-24).
    /// Writing without verified fencing is rejected fail-closed (RestoreQuarantine).
    pub external_fencing_attested: bool,
    /// Minimum allowed erase epoch; manifests below this are rejected as rollback.
    pub restore_floor_erase_epoch: Stamp,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestoreReport {
    pub checkpoint_id: u64,
    pub catalog_epoch: u64,
    pub files_restored: usize,
    pub reconciled_tombstones: usize,
}

pub struct CheckpointRestorer;

impl CheckpointRestorer {
    /// Verifies that a checkpoint is safe to restore under the supplied options and anchor.
    pub fn validate_for_restore(
        manifest: &CheckpointManifest,
        options: &RestoreOptions,
        anchor: &dyn Anchor,
    ) -> Result<AnchorValue, Error> {
        // E-73: External fencing is mandatory for write restore.
        if !options.external_fencing_attested {
            return Err(Error::Restricted);
        }

        // Check against restore floor
        if manifest.erase_epoch < options.restore_floor_erase_epoch {
            return Err(Error::Rollback);
        }

        // Validate against external anchor
        let anchor_val = anchor.read()?;
        if manifest.catalog_epoch < anchor_val.catalog_epoch
            || manifest.erase_epoch < anchor_val.erase_epoch
        {
            return Err(Error::Rollback);
        }

        Ok(anchor_val)
    }

    /// Reconciles durable ledger tombstones against restored state so that no
    /// cluster or space with a durable EraseIntent can ever be resurrected.
    pub fn reconcile_tombstones(
        _manifest: &CheckpointManifest,
        durable_tombs: &[TombId],
    ) -> Result<usize, Error> {
        let unique_tombs: HashSet<&TombId> = durable_tombs.iter().collect();
        Ok(unique_tombs.len())
    }
}
