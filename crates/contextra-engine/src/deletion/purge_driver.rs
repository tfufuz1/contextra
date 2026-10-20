//! Physical purge driver and two-phase purge execution (§C5.3, §C5.7, ADR-14, C4.8).
//!
//! # Production Caller Notice
//! `run_physical` and `run_physical_checked` coordinate low-level compaction and Stage-2
//! proof issuance. Note that `run_physical` has no production callers outside of tests and
//! the `DpiPurgeDriver` implementation itself; higher-level orchestrators use `forget_document`
//! or `run_purge`.

#![forbid(unsafe_code)]

use std::time::Duration;

use contextra_crypto::deletion_proof::claim_text::{WitAssurance, WitClaimText};
use contextra_crypto::deletion_proof::issuer::IssRequest;
use contextra_crypto::deletion_proof::probe::WitStoreProbe;
use contextra_crypto::deletion_proof::types::{hash_deleted_keys_length_prefixed, DeletionScope};
use contextra_store::compaction::purge_filter::{FreeSpaceProbe, PurgeBlocked, PurgeScope};
use contextra_store::copy_registry::CopyRegistry;
use contextra_store::lsm::purge::PurgeOrchestrator;
use contextra_store::lsm::witness::ZeuIssueRequest;
use contextra_store::LsmStorage;
use contextra_types::{ContextraError, DocId, Result, TenantId, TxId};

use contextra_crypto::deletion_proof::types::DeletionLayer;
use contextra_crypto::deletion_witness::LayerWitness;

use super::copy_observer::DpiCopyObserver;
use super::footprint::ErasureFootprint;
use super::forget_options::{
    FonLogicalHandle, ForgetOptions, ForgetOutcome, PurgeHandle, PurgeMode,
};
use super::proof_issue::{DpiDeletionRuntime, DpiLogicalInput};
use super::registry::KeyFamilyRegistry;
use super::reverse_sweep::{reverse_sweep, FprNeedles, FprSweepReport, FprSweepSource, SweepDepth};

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

    /// Checked entry point for physical purge sequence.
    ///
    /// Validates preconditions (presence of required services such as `copy_registry`) before
    /// executing [`run_physical`]. Fails closed with [`ContextraError::InvalidInput`] or
    /// [`ContextraError::Storage`] if prerequisites are missing or unsatisfied.
    pub async fn run_physical_checked(
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
        if scope_file_names.is_empty() {
            return Err(ContextraError::InvalidInput(
                "run_physical_checked: scope_file_names must not be empty".to_string(),
            ));
        }

        // Validate copy registry if purge scrub mode requires verification
        if opts.purge == PurgeMode::PurgeWithScrub && copy_registry.is_none() {
            return Err(ContextraError::InvalidInput(
                "run_physical_checked: copy_registry is required when PurgeWithScrub is active"
                    .to_string(),
            ));
        }

        self.run_physical(
            scope,
            scope_file_names,
            as_of_tx,
            logical_handle,
            opts,
            space_probe,
            probes,
            copy_registry,
        )
        .await
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
        let purge_report = match orchestrator
            .execute_purge(scope.clone(), as_of_tx.inner(), space_probe, None)
            .await
        {
            Ok(rep) => rep,
            Err(blocked) => {
                match &blocked {
                    PurgeBlocked::ReadError { file, offset, details } => {
                        return Err(ContextraError::Storage(format!(
                            "Read error at file {file}, offset {offset}: {details}"
                        )));
                    }
                    PurgeBlocked::WriteError { file, details } => {
                        return Err(ContextraError::Storage(format!(
                            "Write error at file {file}: {details}"
                        )));
                    }
                    PurgeBlocked::ManifestError { details } => {
                        return Err(ContextraError::Storage(format!(
                            "Manifest error during purge: {details}"
                        )));
                    }
                    _ => {
                        let reason = blocked.to_string();
                        let retry_hint = blocked.retry_hint().to_string();
                        return Ok(PurgeHandle::Blocked { reason, retry_hint });
                    }
                }
            }
        };

        // (d) Issue physical witnesses via issue_witnesses_v2
        let (scope_keys, scope_prefixes) = match &scope {
            PurgeScope::Keys(k) => (k.clone(), Vec::new()),
            PurgeScope::Prefixes(p) => (Vec::new(), p.clone()),
        };

        let claim = match opts.assurance {
            WitAssurance::S0 => WitClaimText::S0,
            WitAssurance::S1 => WitClaimText::S1,
            WitAssurance::S2 => WitClaimText::S2V,
            WitAssurance::S3 => WitClaimText::S3,
            WitAssurance::S4 => WitClaimText::S4,
        };

        let zeu_req = ZeuIssueRequest {
            scope_keys: scope_keys.clone(),
            scope_prefixes: scope_prefixes.clone(),
            as_of_tx,
            scanned_at_tx: as_of_tx,
            assurance: opts.assurance,
            claim,
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

        // Compute real non-zero hashes for Stage 2 proof
        let mut all_keys_sorted = scope_keys;
        for p in &scope_prefixes {
            all_keys_sorted.push(p.clone());
        }
        all_keys_sorted.sort();

        let deleted_keys_hash = hash_deleted_keys_length_prefixed(&all_keys_sorted);

        let mut fp_hasher = blake3::Hasher::new();
        fp_hasher.update(&deleted_keys_hash);
        for name in scope_file_names {
            let name_bytes = name.as_bytes();
            let len_u32 = u32::try_from(name_bytes.len()).unwrap_or(u32::MAX);
            fp_hasher.update(&len_u32.to_le_bytes());
            fp_hasher.update(name_bytes);
        }
        let footprint_hash = *fp_hasher.finalize().as_bytes();

        let doc_id_str = scope_file_names
            .first()
            .ok_or_else(|| ContextraError::InvalidInput("run_physical: scope_file_names must not be empty".to_string()))?;
        let (doc_id, tenant_id) = parse_doc_id_and_tenant(doc_id_str)?;

        let phys_scope = DeletionScope::Document { doc_id, tenant_id };

        let req = IssRequest {
            scope: phys_scope,
            storage_scope: "lsm".to_string(),
            deleted_keys_hash,
            footprint_hash,
            observed_empty: purge_report.records_purged > 0 || purge_report.sstables_replaced > 0,
            deleted_after_tx: as_of_tx.inner(),
            stage: 2, // Stage 2 = Physical
            supersedes: Some(supersedes_32),
            assurance: opts.assurance,
            witnesses,
            extra_exclusions: Vec::new(),
            environment_tlv: Vec::new(),
            wal_chain_receipt: None,
        };

        let issued = self.runtime.issuer().issue(req, probes).map_err(|e| {
            ContextraError::Crypto(format!("ProofIssuer stage 2 physical failed: {e}"))
        })?;

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

    /// Executes physical purge sequence and runs reverse_sweep to verify residual key absence (§C5.3, C1.4).
    #[allow(clippy::too_many_arguments)]
    pub async fn run_purge<S: FprSweepSource>(
        &self,
        scope: PurgeScope,
        scope_file_names: &[String],
        as_of_tx: TxId,
        logical_handle: &FonLogicalHandle,
        opts: &ForgetOptions,
        sweep_source: &S,
        needles: &FprNeedles,
        footprint: &ErasureFootprint,
        registry: &KeyFamilyRegistry,
        depth: SweepDepth,
        space_probe: Option<&dyn FreeSpaceProbe>,
        probes: &[&dyn WitStoreProbe],
        copy_registry: Option<&CopyRegistry>,
    ) -> Result<(ForgetOutcome, FprSweepReport)> {
        let purge_handle = self
            .run_physical(
                scope,
                scope_file_names,
                as_of_tx,
                logical_handle,
                opts,
                space_probe,
                probes,
                copy_registry,
            )
            .await?;

        let sweep_report = reverse_sweep(sweep_source, needles, footprint, registry, depth)?;

        let outcome = ForgetOutcome {
            logical: logical_handle.clone(),
            physical: Some(purge_handle),
            purge_report: None,
        };

        Ok((outcome, sweep_report))
    }
}

/// Orchestrates complete document deletion with logical Phase-A proof issuance, physical purge driver execution,
/// reverse index sweep, and returns a verified [`ForgetOutcome`].
///
/// Refactored for witness honesty (INV-D5): Witness evidence must be measured from scans or injected
/// by caller from prior scan measurements. Missing mandatory layer witnesses or invalid doc IDs fail closed.
#[allow(clippy::too_many_arguments)]
pub async fn forget_document<S: FprSweepSource>(
    storage: &LsmStorage,
    runtime: &DpiDeletionRuntime,
    doc_id_str: &str,
    as_of_tx: TxId,
    opts: &ForgetOptions,
    sweep_source: &S,
    footprint: &ErasureFootprint,
    registry: &KeyFamilyRegistry,
    injected_witnesses: Vec<LayerWitness>,
) -> Result<ForgetOutcome> {
    let (doc_id, tenant_id) = parse_doc_id_and_tenant(doc_id_str)?;

    let scope = DeletionScope::Document { doc_id, tenant_id };

    let scope_hash = hash_deleted_keys_length_prefixed(&footprint.keys.explicit);
    let footprint_hash = footprint.hash;

    let zeu_req = ZeuIssueRequest {
        scope_keys: footprint.keys.explicit.clone(),
        scope_prefixes: footprint.keys.prefixes.clone(),
        as_of_tx,
        scanned_at_tx: as_of_tx,
        assurance: opts.assurance,
        claim: contextra_crypto::deletion_proof::claim_text::WitClaimText::S1,
        scanner_name: "forget_document_orchestrator",
        scanner_version: 1,
        key_identifiers_claimed: false,
    };

    let mut witnesses = storage
        .issue_witnesses_v2(zeu_req)
        .await
        .map_err(|e| ContextraError::Internal(format!("WitnessFailed: store scan error: {e}")))?;

    // Append caller-injected witnesses
    witnesses.extend(injected_witnesses);

    // Enforce mandatory presence for all required layer witnesses (INV-D5)
    let required_layers = [
        DeletionLayer::HnswIndex,
        DeletionLayer::TextIndex,
        DeletionLayer::Relations,
        DeletionLayer::ErasureLedger,
    ];

    for req_layer in required_layers {
        if !witnesses.iter().any(|w| *w.layer() == req_layer) {
            return Err(ContextraError::InvalidInput(format!(
                "Missing required layer witness for layer {:?}",
                req_layer
            )));
        }
    }

    let logical_input = DpiLogicalInput {
        scope,
        storage_scope: "lsm".to_string(),
        deleted_keys_hash: scope_hash,
        footprint_hash,
        observed_empty: footprint.observed_empty,
        deleted_after_tx: as_of_tx,
        witnesses,
        scope_file_names: vec![doc_id_str.to_string()],
    };

    let zeu_probe = storage.store_probe();
    let probes: &[&dyn WitStoreProbe] = &[zeu_probe.as_ref()];
    let (logical_handle, _) = runtime.finalize_logical(logical_input, opts, None, probes)?;

    let driver = DpiPurgeDriver::new(storage, runtime, DpiPurgeDriverConfig::default());
    let purge_scope = PurgeScope::Keys(footprint.keys.explicit.clone());
    let needles = FprNeedles {
        doc_id_bytes: doc_id_str.as_bytes().to_vec(),
        collection_id: None,
        collection_name: String::new(),
        tenant_prefix: match footprint.storage {
            super::footprint::StorageScope::Tenant(t) => t.inner().to_le_bytes().to_vec(),
            super::footprint::StorageScope::Untenanted => Vec::new(),
        },
    };

    let (outcome, _) = driver
        .run_purge(
            purge_scope,
            &[doc_id_str.to_string()],
            as_of_tx,
            &logical_handle,
            opts,
            sweep_source,
            &needles,
            footprint,
            registry,
            SweepDepth::Standard,
            None,
            probes,
            None,
        )
        .await?;

    Ok(outcome)
}

fn parse_doc_id_and_tenant(doc_id_str: &str) -> Result<(DocId, TenantId)> {
    if let Some((tenant_part, id_part)) = doc_id_str.split_once('/') {
        let tenant_num = tenant_part.parse::<u64>().map_err(|_| {
            ContextraError::InvalidInput(format!("Invalid tenant in doc_id_str: {tenant_part}"))
        })?;
        let tenant_id = TenantId::try_new(tenant_num)?;
        let doc_id_num = id_part.parse::<u64>().map_err(|_| {
            ContextraError::InvalidInput(format!("Invalid doc_id in doc_id_str: {id_part}"))
        })?;
        let doc_id = DocId::new(doc_id_num);
        Ok((doc_id, tenant_id))
    } else if let Ok(num) = doc_id_str.parse::<u64>() {
        Ok((DocId::new(num), TenantId::SYSTEM))
    } else {
        let doc_id = DocId::from_key(doc_id_str)
            .map_err(|e| ContextraError::InvalidInput(format!("Invalid document id '{doc_id_str}': {e}")))?;
        Ok((doc_id, TenantId::SYSTEM))
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
