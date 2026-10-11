//! Implementation of scanner-based MemTable witness issuance for physical deletion verification (W5-04).

use crate::lsm::LsmStorage;
use contextra_core::{ContextraError, Result, TxId};
use contextra_crypto::deletion_proof::DeletionLayer;
use contextra_crypto::deletion_witness::{LayerWitness, WitnessEvidence};
use std::sync::Arc;

impl LsmStorage {
    /// Issues a [`LayerWitness`] attesting that no non-tombstone versions of the specified `deleted_keys`
    /// or `prefixes` exist in the active or any immutable MemTables.
    ///
    /// # Lock Hierarchy
    /// Holds the `state` read lock (`tokio::sync::RwLock<LsmState>`) for the duration of the
    /// in-memory inspection. No disk I/O is performed while holding the lock.
    ///
    /// # Errors
    /// Returns `ContextraError::Internal` with INV-DELETION-1 violation details if any non-tombstone version
    /// is detected. Key bytes are strictly omitted from error messages.
    pub async fn issue_memtable_witness(
        &self,
        deleted_keys: &[Vec<u8>],
        prefixes: &[Vec<u8>],
        as_of_tx: TxId,
    ) -> Result<LayerWitness> {
        let state = self.state.read().await;

        let mut memtables = Vec::with_capacity(1 + state.immutable_memtables.len());
        memtables.push(Arc::clone(&state.memtable));
        for imm in &state.immutable_memtables {
            memtables.push(Arc::clone(imm));
        }

        for (idx, memtable) in memtables.iter().enumerate() {
            let mut violation_count = 0usize;

            for key in deleted_keys {
                violation_count += memtable.count_non_tombstone_key(key);
            }

            for prefix in prefixes {
                violation_count += memtable.count_non_tombstone_prefix(prefix);
            }

            if violation_count > 0 {
                return Err(ContextraError::Internal(format!(
                    "INV-DELETION-1 violation: found {} non-tombstone versions in memtable_index: {}",
                    violation_count, idx
                )));
            }
        }

        let scanned_count = u32::try_from(memtables.len()).unwrap_or(u32::MAX);
        drop(state);

        LayerWitness::issue_from_storage_layer(
            DeletionLayer::LsmMemtable,
            deleted_keys,
            as_of_tx,
            WitnessEvidence::MemtableScan {
                memtables_scanned: scanned_count,
            },
        )
    }
}
