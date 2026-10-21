//! Implementation of scanner-based witness issuance for physical deletion verification (W5-04 / C4).

use crate::lsm::LsmStorage;
use crate::sstable::witness_scan::scan_sstables;
use crate::sstable::witness_types::{SstScanEvidence, SstScanInputs, SstScopeSpec};
use crate::wal::witness_scan::{scan_wal_directory, WalScanInputs, WalScopeSpec};
use contextra_crypto::deletion_proof::claim_text::{WitAssurance, WitClaimText};
use contextra_crypto::deletion_proof::evidence::WitLayerEvidence;
use contextra_crypto::deletion_proof::hash_deleted_keys_length_prefixed;
use contextra_crypto::deletion_proof::probe::WitStoreGeneration;
use contextra_crypto::deletion_proof::DeletionLayer;
use contextra_crypto::deletion_witness::{LayerWitness, WitIssueArgs, WitnessEvidence};
use contextra_durable_fs::PendingDeleteJournal;
use contextra_types::{ContextraError, Result, TxId};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

/// Request arguments for v2 multi-layer scanner-based witness issuance.
#[derive(Debug, Clone)]
pub struct ZeuIssueRequest {
    /// Scope keys to audit.
    pub scope_keys: Vec<Vec<u8>>,
    /// Scope prefixes to audit.
    pub scope_prefixes: Vec<Vec<u8>>,
    /// Snapshot transaction ID of deletion request.
    pub as_of_tx: TxId,
    /// Transaction ID at time of scan execution.
    pub scanned_at_tx: TxId,
    /// Assurance level claimed.
    pub assurance: WitAssurance,
    /// Claim text claimed (S0, S1, S2V, S2K, S3, S4).
    pub claim: WitClaimText,
    /// Scanner component identifier name.
    pub scanner_name: &'static str,
    /// Scanner component version.
    pub scanner_version: u32,
    /// Whether key identifiers are claimed.
    pub key_identifiers_claimed: bool,
}

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

    /// Issues multi-layer scanner-based [`LayerWitness`] proofs across MemTable, SSTables, Manifest, and WAL.
    pub async fn issue_witnesses_v2(&self, req: ZeuIssueRequest) -> Result<Vec<LayerWitness>> {
        let seq_before = self.erasure_fence.snapshot_write_seq(&req.scope_prefixes);
        let scope_hash = hash_deleted_keys_length_prefixed(&req.scope_keys);

        let mut result_witnesses = Vec::new();

        // Dynamische Manifest-Generation (1-basiert: 1 + flush_counter)
        let flush_gen = self.flush_counter.load(Ordering::Acquire);
        let manifest_gen = 1 + flush_gen;

        // Dynamisches Auslesen des PendingDeleteJournals (fail-closed bei Read-Error)
        let pending_delete_journal_path = self.config.path.join("pending_delete.journal");
        let pending_delete_files = if pending_delete_journal_path.exists() {
            let journal =
                PendingDeleteJournal::open(&pending_delete_journal_path).map_err(|e| {
                    ContextraError::Storage(format!(
                        "Failed to open pending_delete.journal for witness issuance: {e}"
                    ))
                })?;
            journal.pending().to_vec()
        } else {
            Vec::new()
        };

        // Dynamische Ermittlung von in_flight_commits aus pending_commit_queue
        let in_flight_commits = {
            let queue_guard = self.pending_commit_queue.lock().await;
            if let Some(ref queue) = *queue_guard {
                let shared = queue.shared.lock().unwrap_or_else(|e| e.into_inner());
                u32::try_from(shared.requests.len()).unwrap_or(u32::MAX)
            } else {
                0u32
            }
        };

        // 1. MemTable Witness
        let mem_evidence_keys = self
            .scan_memtable_residue(MemScopeSpec::Keys(&req.scope_keys), req.as_of_tx.inner())
            .await?;
        let mem_evidence_prefixes = self
            .scan_memtable_residue(
                MemScopeSpec::Prefixes(&req.scope_prefixes),
                req.as_of_tx.inner(),
            )
            .await?;

        let total_versions =
            mem_evidence_keys.versions_found + mem_evidence_prefixes.versions_found;
        if total_versions > 0 {
            return Err(ContextraError::InvalidInput(format!(
                "Memtable witness rejected: found {} non-tombstone versions in memtables",
                total_versions
            )));
        }

        let total_entries = mem_evidence_keys.entries_seen + mem_evidence_prefixes.entries_seen;
        let memtables_scanned = mem_evidence_keys
            .memtables_scanned
            .max(mem_evidence_prefixes.memtables_scanned);

        let mem_layer_ev = WitLayerEvidence::Memtable {
            memtables_scanned,
            entries_found: total_entries as u64,
            tombstones_found: total_entries as u64,
        };

        let mem_witness = LayerWitness::issue(WitIssueArgs {
            layer: DeletionLayer::LsmMemtable,
            scope_hash,
            scanned_at_tx: req.scanned_at_tx,
            as_of_tx: req.as_of_tx,
            assurance: req.assurance,
            claim: req.claim,
            footprint_hash: [0u8; 32],
            scope_write_seq: seq_before,
            scanner_name: req.scanner_name,
            scanner_version: req.scanner_version,
            store_generation: WitStoreGeneration(self.flush_counter.load(Ordering::Acquire)),
            layer_evidence: mem_layer_ev,
            key_identifiers_claimed: req.key_identifiers_claimed,
        })?;
        result_witnesses.push(mem_witness);

        // 2. SSTables Witness
        let live_sst_paths: Vec<_> = self
            .sstables
            .read()
            .await
            .iter()
            .map(|s| s.file_path().to_path_buf())
            .collect();

        let sst_inputs = SstScanInputs {
            data_dirs: vec![self.config.path.clone()],
            live_files: live_sst_paths,
            pending_delete_files,
            manifest_generation: manifest_gen,
            as_of_tx: req.as_of_tx.inner(),
            depth: 0,
        };

        let sst_scope = SstScopeSpec {
            explicit: req.scope_keys.clone(),
            prefixes: req.scope_prefixes.clone(),
        };

        let sst_scan_res = scan_sstables(&sst_scope, &sst_inputs, self.key_manager.clone(), || {
            manifest_gen
        })
        .map_err(|e| ContextraError::Storage(e.to_string()))?;
        let violations = Self::validate_sst_evidence(&sst_scan_res, req.key_identifiers_claimed);
        if !violations.is_empty() {
            return Err(ContextraError::InvalidInput(format!(
                "SSTable witness rejected due to invariant violations: {}",
                violations.join("; ")
            )));
        }

        let sst_layer_ev = WitLayerEvidence::SsTables {
            manifest_gen: sst_scan_res.manifest_gen,
            files_in_manifest: sst_scan_res.files_in_manifest as u32,
            files_scanned: sst_scan_res.files_scanned as u32,
            files_pruned_with_verified_range: sst_scan_res.files_pruned_with_verified_range as u32,
            blocks_scanned: sst_scan_res.blocks_scanned,
            bytes_scanned: sst_scan_res.bytes_scanned,
            orphan_files_found: sst_scan_res.orphan_files_found as u32,
            unclassified_files: sst_scan_res.unclassified_files as u32,
            versions_found: sst_scan_res.versions_found,
            identifier_hits_index: if req.key_identifiers_claimed {
                sst_scan_res.identifier_hits_index
            } else {
                0
            },
            tombstones_remaining: sst_scan_res.tombstones_remaining,
        };

        let sst_witness = LayerWitness::issue(WitIssueArgs {
            layer: DeletionLayer::SsTableAllLevels,
            scope_hash,
            scanned_at_tx: req.scanned_at_tx,
            as_of_tx: req.as_of_tx,
            assurance: req.assurance,
            claim: req.claim,
            footprint_hash: [0u8; 32],
            scope_write_seq: seq_before,
            scanner_name: req.scanner_name,
            scanner_version: req.scanner_version,
            store_generation: WitStoreGeneration(manifest_gen),
            layer_evidence: sst_layer_ev,
            key_identifiers_claimed: req.key_identifiers_claimed,
        })?;
        result_witnesses.push(sst_witness);

        // 3. Manifest Witness
        // Dynamische Ermittlung ungeflushter Memtable-Einträge (unpersisted_edits)
        let state = self.state.read().await;
        let mut unpersisted_count = state.memtable.iter().len();
        for imm in &state.immutable_memtables {
            unpersisted_count += imm.iter().len();
        }
        drop(state);

        let unpersisted_edits = u32::try_from(unpersisted_count).unwrap_or(u32::MAX);

        let manifest_layer_ev = WitLayerEvidence::Manifest {
            current_manifest_gen: manifest_gen,
            in_flight_commits,
            unpersisted_edits,
        };

        let manifest_witness = LayerWitness::issue(WitIssueArgs {
            layer: DeletionLayer::Manifest,
            scope_hash,
            scanned_at_tx: req.scanned_at_tx,
            as_of_tx: req.as_of_tx,
            assurance: req.assurance,
            claim: req.claim,
            footprint_hash: [0u8; 32],
            scope_write_seq: seq_before,
            scanner_name: req.scanner_name,
            scanner_version: req.scanner_version,
            store_generation: WitStoreGeneration(manifest_gen),
            layer_evidence: manifest_layer_ev,
            key_identifiers_claimed: req.key_identifiers_claimed,
        })?;
        result_witnesses.push(manifest_witness);

        // 4. WAL Witness
        let wal_inputs = WalScanInputs {
            wal_dir: self.config.path.clone(),
            as_of_tx: req.as_of_tx.inner(),
            encrypted_mode: self.config.encryption.is_encrypted(),
            timeout: Duration::from_secs(10),
            scope: WalScopeSpec {
                key_prefixes: req.scope_prefixes.clone(),
                raw_needles: req.scope_keys.clone(),
            },
            key_identifiers_claimed: req.key_identifiers_claimed,
        };

        let wal_ev = scan_wal_directory(&wal_inputs).await?;
        if !wal_ev.is_valid() {
            return Err(ContextraError::InvalidInput(format!(
                "WAL witness rejected due to invariant violations: {}",
                wal_ev.invariant_violations().join("; ")
            )));
        }

        let wal_layer_ev = WitLayerEvidence::Wal {
            segments_scanned: wal_ev.segments_scanned as u32,
            segments_total: wal_ev.segments_total as u32,
            segments_removed: wal_ev.segments_removed as u32,
            bytes_scrubbed: wal_ev.tail_bytes_raw_scanned,
            unscrubbed_records_found: wal_ev.frames_matching_scope,
        };

        let wal_witness = LayerWitness::issue(WitIssueArgs {
            layer: DeletionLayer::WalAllSegments { seq_after: 0 },
            scope_hash,
            scanned_at_tx: req.scanned_at_tx,
            as_of_tx: req.as_of_tx,
            assurance: req.assurance,
            claim: req.claim,
            footprint_hash: [0u8; 32],
            scope_write_seq: seq_before,
            scanner_name: req.scanner_name,
            scanner_version: req.scanner_version,
            store_generation: WitStoreGeneration(self.segment_counter.load(Ordering::Acquire)),
            layer_evidence: wal_layer_ev,
            key_identifiers_claimed: req.key_identifiers_claimed,
        })?;
        result_witnesses.push(wal_witness);

        // Concurrency mutation check
        let seq_after = self.erasure_fence.snapshot_write_seq(&req.scope_prefixes);
        if seq_before != seq_after {
            return Err(ContextraError::InvalidInput(
                "ConcurrentModification during witness scan".to_string(),
            ));
        }

        Ok(result_witnesses)
    }

    fn validate_sst_evidence(ev: &SstScanEvidence, key_identifiers_claimed: bool) -> Vec<String> {
        let mut violations = Vec::new();
        if ev.unclassified_files > 0 {
            violations.push(format!(
                "Fail-closed: {} unclassified file(s) found",
                ev.unclassified_files
            ));
        }
        if ev.versions_found > 0 {
            violations.push(format!(
                "Claim CT-S2V violation: {} live version(s) found in storage",
                ev.versions_found
            ));
        }
        if key_identifiers_claimed && ev.identifier_hits_index > 0 {
            violations.push(format!(
                "Claim CT-S2K violation: {} index separator/fence hit(s) found",
                ev.identifier_hits_index
            ));
        }
        if key_identifiers_claimed && ev.tombstones_remaining > 0 {
            violations.push(format!(
                "Claim CT-S2K violation: {} tombstone(s) remaining",
                ev.tombstones_remaining
            ));
        }
        if ev.range_hits > 0 {
            violations.push(format!(
                "Claim CT-S2K/CT-S2V violation: {} historical manifest range hit(s) found",
                ev.range_hits
            ));
        }
        violations
    }

    /// Issues a [`LayerWitness`] for the specified Primary Data / LSM storage layer variant.
    /// Legacy interface restricted to `LsmMemtable` to prevent scan-less false positive witnesses.
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
            DeletionLayer::SsTableAllLevels => Err(ContextraError::InvalidInput(
                "Scan-less witness creation rejected: use issue_witnesses_v2 for SsTableAllLevels".to_string(),
            )),
            DeletionLayer::WalAllSegments { .. } => Err(ContextraError::InvalidInput(
                "Scan-less witness creation rejected: use issue_witnesses_v2 for WalAllSegments".to_string(),
            )),
            other => Err(ContextraError::InvalidInput(format!(
                "LayerWitness creation rejected: layer {:?} is not a supported primary data/LSM layer variant",
                other
            ))),
        }
    }
}
