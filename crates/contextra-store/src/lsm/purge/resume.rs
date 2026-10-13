// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: Resume-Schnittstelle zur Wiederaufnahme nicht abgeschlossener Purge-Operationen nach Absturz (Anhang B).
// INVARIANTEN: Erkennt Korruption (Err), führt verbleibende Phasen idempotent aus, reinigt PendingDeleteJournal.

use super::dispose::{dispose_obsolete_files_with_policy, DisposePolicy};
use super::failpoint::{OrxFailpointSet, OrxPurgeError};
use super::intent::OrxPurgeIntent;
use super::orchestrator::{OrxHooks, PurgeOrchestrator};
use crate::compaction::purge_filter::{PurgeReport, PurgeScope};
use contextra_durable_fs::PendingDeleteJournal;
use std::path::Path;

/// Resumes any pending or interrupted purge operation present in `storage_dir` using default safe policy [`DisposePolicy::RequireScrub`].
///
/// Evaluates saved `purge.intent` and `pending_delete.journal`.
/// Returns `Ok(None)` if no pending operation or journal remains.
/// Returns `Ok(Some(report))` upon successful resumption and completion.
/// Propagates `Err` if intent or journal is corrupted or execution fails.
pub async fn resume_pending(
    storage_dir: &Path,
    hooks: &OrxHooks,
    failpoints: Option<&OrxFailpointSet>,
) -> Result<Option<PurgeReport>, OrxPurgeError> {
    resume_pending_with_policy(storage_dir, hooks, failpoints, DisposePolicy::RequireScrub).await
}

/// Resumes any pending or interrupted purge operation present in `storage_dir` using a specified [`DisposePolicy`].
pub async fn resume_pending_with_policy(
    storage_dir: &Path,
    hooks: &OrxHooks,
    failpoints: Option<&OrxFailpointSet>,
    policy: DisposePolicy,
) -> Result<Option<PurgeReport>, OrxPurgeError> {
    let intent_file = storage_dir.join("purge.intent");
    let journal_file = storage_dir.join("pending_delete.journal");

    let intent_opt = OrxPurgeIntent::read_from_file(&intent_file)?;

    if let Some(intent) = intent_opt {
        let scope = if let Some(ref recon_fn) = hooks.reconstruct_scope {
            recon_fn(intent.ledger_seq, intent.proof_id)?
        } else {
            PurgeScope::Keys(Vec::new())
        };

        let mut orchestrator = PurgeOrchestrator::new_standalone(storage_dir);
        if let Some(fp) = failpoints {
            orchestrator = orchestrator.with_failpoints(fp.clone());
        }

        let secure_delete = policy != DisposePolicy::Disabled;
        let report = orchestrator
            .execute_from_intent(intent, scope, hooks, secure_delete, None)
            .await?;

        return Ok(Some(report));
    }

    // If no intent remains but pending_delete.journal is present, clean up orphaned files
    if journal_file.exists() {
        let journal = PendingDeleteJournal::open(&journal_file)?;
        let obsolete_files = journal.pending().to_vec();

        let _disp_rep =
            dispose_obsolete_files_with_policy(storage_dir, obsolete_files, policy).await?;
        return Ok(Some(PurgeReport {
            records_purged: 0,
            bytes_reclaimed: 0,
            sstables_replaced: 0,
        }));
    }

    Ok(None)
}
