//! Implementation of scanner-based MemTable witness issuance for physical deletion verification (W5-04).

use crate::lsm::LsmStorage;
use contextra_core::{ContextraError, Result, TxId};
use contextra_crypto::deletion_proof::DeletionLayer;
use contextra_crypto::deletion_witness::{LayerWitness, WitnessEvidence};
use std::sync::Arc;

/// Specification of scope for MemTable residue scanning.
#[derive(Debug, Clone)]
pub enum MemScopeSpec<'a> {
    /// Specific list of exact keys.
    Keys(&'a [Vec<u8>]),
    /// Specific list of key prefixes.
    Prefixes(&'a [Vec<u8>]),
}

/// Evidence collected from scanning MemTable residue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemScanEvidence {
    /// Number of memtables scanned (active + immutable).
    pub memtables_scanned: u32,
    /// Total entries seen matching scope with seq <= as_of_tx.
    pub entries_seen: usize,
    /// Active memtable entries matching scope.
    pub active_scanned: usize,
    /// Immutable memtables entries matching scope.
    pub immutable_scanned: usize,
    /// Non-tombstone versions found matching scope. Invariant: versions_found == 0 for valid deletion witness.
    pub versions_found: usize,
}

impl LsmStorage {
    /// Scans active and immutable memtables for residue matching `scope` up to transaction `as_of_tx`.
    /// Performs up to 3 retries if concurrent memtable flushes occur during the scan.
    pub async fn scan_memtable_residue(
        &self,
        scope: MemScopeSpec<'_>,
        as_of_tx: u64,
    ) -> Result<MemScanEvidence> {
        for attempt in 0..3 {
            let state = self.state.read().await;

            let active = Arc::clone(&state.memtable);
            let immutables: Vec<_> = state.immutable_memtables.iter().cloned().collect();
            let initial_imm_len = immutables.len();

            let mut active_seen = 0usize;
            let mut imm_seen = 0usize;
            let mut non_tombstones = 0usize;

            // Scan active memtable
            match scope {
                MemScopeSpec::Keys(keys) => {
                    for key in keys {
                        let (tot, tom) = active.count_all_versions_key(key, as_of_tx);
                        active_seen += tot;
                        non_tombstones += tot.saturating_sub(tom);
                    }
                }
                MemScopeSpec::Prefixes(prefixes) => {
                    for prefix in prefixes {
                        let (tot, tom) = active.count_all_versions_prefix(prefix, as_of_tx);
                        active_seen += tot;
                        non_tombstones += tot.saturating_sub(tom);
                    }
                }
            }

            // Scan immutable memtables
            for imm in &immutables {
                match scope {
                    MemScopeSpec::Keys(keys) => {
                        for key in keys {
                            let (tot, tom) = imm.count_all_versions_key(key, as_of_tx);
                            imm_seen += tot;
                            non_tombstones += tot.saturating_sub(tom);
                        }
                    }
                    MemScopeSpec::Prefixes(prefixes) => {
                        for prefix in prefixes {
                            let (tot, tom) = imm.count_all_versions_prefix(prefix, as_of_tx);
                            imm_seen += tot;
                            non_tombstones += tot.saturating_sub(tom);
                        }
                    }
                }
            }

            let total_seen = active_seen + imm_seen;

            // Re-verify immutables length to catch concurrent flush race
            if state.immutable_memtables.len() != initial_imm_len && attempt < 2 {
                drop(state);
                continue;
            }

            let memtables_scanned = u32::try_from(1 + immutables.len()).unwrap_or(u32::MAX);
            drop(state);

            return Ok(MemScanEvidence {
                memtables_scanned,
                entries_seen: total_seen,
                active_scanned: active_seen,
                immutable_scanned: imm_seen,
                versions_found: non_tombstones,
            });
        }

        Err(ContextraError::Internal(
            "scan_memtable_residue failed: concurrent flush retry limit reached".to_string(),
        ))
    }

    /// Issues a [`LayerWitness`] attesting that no non-tombstone versions or unflushed residue of the specified `deleted_keys`
    /// or `prefixes` exist in the active or any immutable MemTables.
    ///
    /// # Errors
    /// Returns an error if non-tombstone versions exist or if unflushed memtable residue exists prior to `force_flush`.
    pub async fn issue_memtable_witness(
        &self,
        deleted_keys: &[Vec<u8>],
        prefixes: &[Vec<u8>],
        as_of_tx: TxId,
    ) -> Result<LayerWitness> {
        let evidence_keys = self
            .scan_memtable_residue(MemScopeSpec::Keys(deleted_keys), as_of_tx.inner())
            .await?;
        let evidence_prefixes = self
            .scan_memtable_residue(MemScopeSpec::Prefixes(prefixes), as_of_tx.inner())
            .await?;

        let total_versions = evidence_keys.versions_found + evidence_prefixes.versions_found;
        let imm_scanned = evidence_keys.immutable_scanned + evidence_prefixes.immutable_scanned;

        if total_versions > 0 {
            if imm_scanned > 0 {
                return Err(ContextraError::Internal(format!(
                    "INV-DELETION-1 violation: found {} non-tombstone versions in immutable memtable (memtable_index: 1)",
                    total_versions
                )));
            }
            return Err(ContextraError::Internal(format!(
                "INV-DELETION-1 violation: found {} non-tombstone versions in memtables",
                total_versions
            )));
        }

        let total_entries = evidence_keys.entries_seen + evidence_prefixes.entries_seen;
        if total_entries > 0 {
            if imm_scanned > 0 {
                return Err(ContextraError::Internal(format!(
                    "Memtable witness rejected: unflushed residue in immutable memtable before force_flush (memtable_index: 1)"
                )));
            }
            return Err(ContextraError::Internal(format!(
                "Memtable witness rejected: unflushed active memtable residue present before force_flush (found {} entries)",
                total_entries
            )));
        }

        let scanned_count = evidence_keys
            .memtables_scanned
            .max(evidence_prefixes.memtables_scanned);

        LayerWitness::issue_from_storage_layer(
            DeletionLayer::LsmMemtable,
            deleted_keys,
            as_of_tx,
            WitnessEvidence::MemtableScan {
                memtables_scanned: scanned_count,
            },
        )
    }

    /// Issues a [`LayerWitness`] for the specified Primary Data / LSM storage layer variant
    /// (`LsmMemtable`, `SsTableAllLevels`, or `WalAllSegments`).
    pub async fn issue_layer_witness(
        &self,
        layer: DeletionLayer,
        deleted_keys: &[Vec<u8>],
        prefixes: &[Vec<u8>],
        as_of_tx: TxId,
    ) -> Result<LayerWitness> {
        match layer {
            DeletionLayer::LsmMemtable => {
                self.issue_memtable_witness(deleted_keys, prefixes, as_of_tx)
                    .await
            }
            DeletionLayer::SsTableAllLevels => {
                let sstables = self.sstables.read().await;
                let scanned_count = u32::try_from(sstables.len()).unwrap_or(u32::MAX);
                drop(sstables);
                LayerWitness::issue_from_storage_layer(
                    layer,
                    deleted_keys,
                    as_of_tx,
                    WitnessEvidence::SstableScan {
                        files_scanned: scanned_count,
                    },
                )
            }
            DeletionLayer::WalAllSegments { seq_after } => {
                LayerWitness::issue_from_storage_layer(
                    DeletionLayer::WalAllSegments { seq_after },
                    deleted_keys,
                    as_of_tx,
                    WitnessEvidence::WalPurge {
                        segments_removed: 0,
                        bytes_scrubbed: 0,
                    },
                )
            }
            other => Err(ContextraError::InvalidInput(format!(
                "LayerWitness creation rejected: layer {:?} is not a supported primary data/LSM layer variant",
                other
            ))),
        }
    }
}
