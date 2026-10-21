// FILE-CONTEXT
// STAND: 2026-10-21
// ZWECK: Resume-Schnittstelle zur Wiederaufnahme nicht abgeschlossener Purge-Operationen nach Absturz.

use super::backend::{MockBackend, PurgeBackend};
use super::dispose::{dispose_obsolete_files_with_policy, DisposePolicy, OrxDisposeReport};
use super::failpoint::{OrxFailpointSet, OrxPurgeError};
use super::intent::OrxPurgeIntent;
use super::orchestrator::{OrxHooks, PurgeOrchestrator};
use crate::compaction::purge_filter::{PurgeReport, PurgeScope};
use crate::lsm::engine::LsmStorage;
use contextra_durable_fs::PendingDeleteJournal;
use std::path::Path;

/// Resumes any pending or interrupted purge operation using a generic [`PurgeBackend`].
pub async fn resume_pending_for_backend<B: PurgeBackend>(
    backend: &B,
    storage_dir: &Path,
    hooks: &OrxHooks,
    failpoints: Option<&OrxFailpointSet>,
    dispose_policy: DisposePolicy,
) -> Result<Option<(PurgeReport, OrxDisposeReport)>, OrxPurgeError> {
    let intent_file = storage_dir.join("purge.intent");
    let journal_file = storage_dir.join("pending_delete.journal");

    let intent_opt = OrxPurgeIntent::read_from_file(&intent_file)?;

    if let Some(intent) = intent_opt {
        let scope = if let Some(ref sc) = intent.scope {
            sc.clone()
        } else if let Some(ref recon_fn) = hooks.reconstruct_scope {
            recon_fn(intent.ledger_seq, intent.proof_id)?
        } else {
            PurgeScope::Keys(Vec::new())
        };

        let mut orchestrator = PurgeOrchestrator::new(backend, storage_dir);
        if let Some(fp) = failpoints {
            orchestrator = orchestrator.with_failpoints(fp.clone());
        }

        let secure_delete = dispose_policy != DisposePolicy::Disabled;
        let report = orchestrator
            .execute_from_intent(intent, scope, hooks, secure_delete, None)
            .await?;

        return Ok(Some((report, OrxDisposeReport::default())));
    }

    if journal_file.exists() {
        let journal = PendingDeleteJournal::open(&journal_file)?;
        let obsolete_files = journal.pending().to_vec();

        let disp_rep =
            dispose_obsolete_files_with_policy(storage_dir, obsolete_files, dispose_policy).await?;

        return Ok(Some((
            PurgeReport {
                records_purged: 0,
                bytes_reclaimed: 0,
                sstables_replaced: 0,
                obsolete_files: Vec::new(),
                residual_risks: Vec::new(),
            },
            disp_rep,
        )));
    }

    Ok(None)
}

/// Resumes any pending or interrupted purge operation present in `storage_dir` using an explicit [`DisposePolicy`].
pub async fn resume_pending_with_policy(
    storage_dir: &Path,
    hooks: &OrxHooks,
    failpoints: Option<&OrxFailpointSet>,
    dispose_policy: DisposePolicy,
) -> Result<Option<(PurgeReport, OrxDisposeReport)>, OrxPurgeError> {
    let mock = MockBackend::new();
    resume_pending_for_backend(&mock, storage_dir, hooks, failpoints, dispose_policy).await
}

/// Resumes any pending or interrupted purge operation present in `storage_dir`.
pub async fn resume_pending(
    storage_dir: &Path,
    hooks: &OrxHooks,
    failpoints: Option<&OrxFailpointSet>,
) -> Result<Option<PurgeReport>, OrxPurgeError> {
    let res = resume_pending_with_policy(storage_dir, hooks, failpoints, DisposePolicy::RequireScrub).await?;
    Ok(res.map(|(r, _)| r))
}

/// Resumes any pending or interrupted purge operation bound to active [`LsmStorage`].
pub async fn resume_pending_for_storage(
    storage: &LsmStorage,
    hooks: &OrxHooks,
    failpoints: Option<&OrxFailpointSet>,
) -> Result<Option<PurgeReport>, OrxPurgeError> {
    let result = resume_pending_for_backend(
        storage,
        &storage.config.path,
        hooks,
        failpoints,
        DisposePolicy::RequireScrub,
    )
    .await?;

    Ok(result.map(|(report, _disp_rep)| report))
}

/// Resumes any pending or interrupted purge operation bound to active [`LsmStorage`] with policy.
pub async fn resume_pending_for_storage_with_policy(
    storage: &LsmStorage,
    hooks: &OrxHooks,
    failpoints: Option<&OrxFailpointSet>,
    dispose_policy: DisposePolicy,
) -> Result<Option<(PurgeReport, OrxDisposeReport)>, OrxPurgeError> {
    resume_pending_for_backend(
        storage,
        &storage.config.path,
        hooks,
        failpoints,
        dispose_policy,
    )
    .await
}
