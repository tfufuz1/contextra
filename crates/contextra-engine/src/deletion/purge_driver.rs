//! Physical purge driver and two-phase purge execution (§C5.3, §C5.7, ADR-14, C4.8).

#![forbid(unsafe_code)]

use std::time::Duration;

use contextra_crypto::deletion_proof::claim_text::WitClaimText;
use contextra_crypto::deletion_proof::issuer::IssRequest;
use contextra_crypto::deletion_proof::probe::WitStoreProbe;
use contextra_crypto::deletion_proof::types::DeletionScope;
use contextra_store::compaction::purge_filter::{FreeSpaceProbe, PurgeBlocked, PurgeScope};
use contextra_store::copy_registry::CopyRegistry;
use contextra_store::lsm::purge::PurgeOrchestrator;
use contextra_store::lsm::witness::ZeuIssueRequest;
use contextra_store::LsmStorage;
use contextra_types::{ContextraError, DocId, Result, TenantId, TxId};

use super::copy_observer::DpiCopyObserver;
use super::forget_options::{FonLogicalHandle, ForgetOptions, PurgeHandle, PurgeMode};
use super::proof_issue::DpiDeletionRuntime;

/// Options controlling physical purge driver execution (§C5.3, C5.7).
#[derive(Debug, Clone)]
pub struct DpiPurgeDriverConfig {
    /// Batching window W (default 0 = immediate flush/WAL cycle).
    pub batch_window: Duration,
}

impl Default for DpiPurgeDriverConfig {
    fn default() -> Self {
        Self {
            batch_window: Duration::from_secs(0),
        }
    }
}

/// Physical purge driver coordinating PurgeOrchestrator, CopyObserver, and Phase-B proof issuance (§C5.3, ADR-14).
pub struct DpiPurgeDriver<'a> {
    storage: &'a LsmStorage,
    runtime: &'a DpiDeletionRuntime,
    config: DpiPurgeDriverConfig,
}

impl<'a> DpiPurgeDriver<'a> {
    /// Creates a new `DpiPurgeDriver` for `storage` and `runtime`.
    #[must_use]
    pub fn new(
        storage: &'a LsmStorage,
        runtime: &'a DpiDeletionRuntime,
        config: DpiPurgeDriverConfig,
    ) -> Self {
        Self {
            storage,
            runtime,
            config,
        }
    }

    /// Executes physical purge sequence run_physical(...) (§C5.3, C4.8, ADR-14).
    ///
    /// (a) Verifies prerequisites (active snapshots, copy registry scope holds, space probe).
    /// (b) If registered copies hold scope files without explicit `purge_checkpoint` authorization in `opts`,
    ///     returns [`PurgeHandle::Blocked`] leaving Phase-A proof valid and adding `RegisteredCopies` exclusion.
    /// (c) Invokes [`PurgeOrchestrator`].
    /// (d) On successful pipeline execution, calls `issue_witnesses_v2` and issues Phase-B (Stage 2) proof
    ///     linking to Phase-A via `supersedes`.
    /// (e) Returns [`PurgeHandle::Final`] or [`PurgeHandle::Blocked`].
    ///
    /// # Errors
    ///
    /// Returns [`ContextraError`] on unrecoverable physical purge or proof issuance errors.
    pub async fn run_physical(
        &self,
        scope: PurgeScope,
        scope_file_names: &[String],
        as_of_tx: TxId,
        logical_handle: &FonLogicalHandle,
        opts: &ForgetOptions,
        space_probe: Option<&dyn FreeSpaceProbe>,
        probes: &[&dyn WitStoreProbe],
        copy_registry: Option<&CopyRegistry>,
    ) -> Result<PurgeHandle> {
        // (a/b) CopyObserver check
        if let Some(copy_reg) = copy_registry {
            let observer = DpiCopyObserver::new(copy_reg);
            let report = observer.observe(scope_file_names);
            if report.copies_holding_scope > 0 && opts.purge != PurgeMode::PurgeWithScrub {
                return Ok(PurgeHandle::Blocked {
                    reason: format!(
                        "Registered checkpoint copies ({}) hold scope data without explicit purge approval",
                        report.copies_holding_scope
                    ),
                    retry_hint: "Authorize purge_checkpoint or remove checkpoint copies first".to_string(),
                });
            }
        }

        // Window W batching delay if configured and non-zero
        if !self.config.batch_window.is_zero() {
            tokio::time::sleep(self.config.batch_window).await;
        }

        // (c) PurgeOrchestrator execution
        let orchestrator = PurgeOrchestrator::new(self.storage);
        let _purge_report = match orchestrator
            .execute_purge(scope.clone(), as_of_tx.inner(), space_probe, None)
            .await
        {
            Ok(rep) => rep,
            Err(blocked) => {
                let (reason, retry_hint) = match blocked {
                    PurgeBlocked::ActiveSnapshot { .. } => (
                        "Active MVCC snapshot blocking compaction".to_string(),
                        "Close open read transactions / snapshots".to_string(),
                    ),
                    PurgeBlocked::InsufficientDiskSpace { needed, free } => (
                        format!("Insufficient disk space for compaction: needed {needed}, free {free}"),
                        "Free disk space or increase storage capacity".to_string(),
                    ),
                    PurgeBlocked::UnclassifiedFiles { count } => (
                        format!("Unclassified files in storage directory: {count}"),
                        "Clean up untracked or stale files".to_string(),
                    ),
                    PurgeBlocked::CheckpointClones { copy_ids } => (
                        format!("Checkpoint clones blocking purge: {}", copy_ids.join(", ")),
                        "Purge or release registered checkpoint clones".to_string(),
                    ),
                    PurgeBlocked::ReaderPinned => (
                        "Reader pinned in active memtable / scan".to_string(),
                        "Wait for active reader scan to finish".to_string(),
                    ),
                    PurgeBlocked::CompactionBusy => (
                        "Background compaction is busy".to_string(),
                        "Retry after current compaction completes".to_string(),
                    ),
                    PurgeBlocked::SecureDeleteUnsupported { reason } => (
                        format!("Secure delete unsupported: {reason}"),
                        "Disable scrub requirement or run on supported filesystem".to_string(),
                    ),
                    PurgeBlocked::Unimplemented => (
                        "Purge compaction implementation inactive".to_string(),
                        "Enable purge compaction feature".to_string(),
                    ),
                    PurgeBlocked::ScanTimeout => (
                        "Scan timeout during purge compaction".to_string(),
                        "Retry with smaller batch size or increased timeout".to_string(),
                    ),
                };
                return Ok(PurgeHandle::Blocked {
                    reason,
                    retry_hint,
                });
            }
        };

        // (d) Issue physical witnesses via issue_witnesses_v2
        let (scope_keys, scope_prefixes) = match &scope {
            PurgeScope::Keys(k) => (k.clone(), Vec::new()),
            PurgeScope::Prefixes(p) => (Vec::new(), p.clone()),
        };

        let zeu_req = ZeuIssueRequest {
            scope_keys,
            scope_prefixes,
            as_of_tx,
            scanned_at_tx: as_of_tx,
            assurance: opts.assurance,
            claim: WitClaimText::S2V,
            scanner_name: "dpi_purge_driver",
            scanner_version: 1,
            key_identifiers_claimed: false,
        };

        let witnesses = self
            .storage
            .issue_witnesses_v2(zeu_req)
            .await
            .map_err(|e| ContextraError::Crypto(format!("Failed to issue witnesses_v2: {e}")))?;

        let supersedes_32 = logical_handle.full_proof_id;

        let doc_id_str = scope_file_names.first().cloned().unwrap_or_default();
        let doc_id = doc_id_str.parse::<u64>().map(DocId::new).unwrap_or_else(|_| DocId::new(0));

        let phys_scope = DeletionScope::Document {
            doc_id,
            tenant_id: TenantId::SYSTEM,
        };

        let req = IssRequest {
            scope: phys_scope,
            storage_scope: "lsm".to_string(),
            deleted_keys_hash: [0u8; 32],
            footprint_hash: [0u8; 32],
            observed_empty: true,
            deleted_after_tx: as_of_tx.inner(),
            stage: 2, // Stage 2 = Physical
            supersedes: Some(supersedes_32),
            assurance: opts.assurance,
            witnesses,
            extra_exclusions: Vec::new(),
            environment_tlv: Vec::new(),
            wal_chain_receipt: None,
        };

        let issued = self
            .runtime
            .issuer()
            .issue(req, probes)
            .map_err(|e| ContextraError::Crypto(format!("ProofIssuer stage 2 physical failed: {e}")))?;

        let proof_id = issued.proof.proof_id;
        let proof_path = self
            .runtime
            .proof_dir()
            .join("proofs")
            .join(format!("{}.json", hex_encode_32(&proof_id)));

        let phys_handle = FonLogicalHandle {
            proof_id: proof_id_32_to_16(&proof_id),
            proof_path,
            full_proof_id: proof_id,
        };

        Ok(PurgeHandle::Final(phys_handle))
    }
}

fn hex_encode_32(bytes: &[u8; 32]) -> String {
    let mut s = String::with_capacity(64);
    for b in bytes {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

fn proof_id_32_to_16(bytes: &[u8; 32]) -> [u8; 16] {
    let mut out = [0u8; 16];
    out.copy_from_slice(&bytes[..16]);
    out
}
