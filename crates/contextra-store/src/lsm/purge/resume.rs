// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: Resume-Schnittstelle zur Wiederaufnahme nicht abgeschlossener Purge-Operationen nach Absturz (Anhang B).
// INVARIANTEN: Erkennt Korruption (Err), führt verbleibende Phasen idempotent aus, reinigt PendingDeleteJournal.

use super::dispose::dispose_obsolete_files;
use super::failpoint::{OrxFailpointSet, OrxPurgeError};
use super::intent::OrxPurgeIntent;
use super::orchestrator::{OrxHooks, PurgeOrchestrator};
use crate::compaction::purge_filter::{PurgeReport, PurgeScope};
use std::path::Path;

/// Resumes any pending or interrupted purge operation present in `storage_dir`.
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

        let report = orchestrator
            .execute_from_intent(intent, scope, hooks, false, None)
            .await?;

        return Ok(Some(report));
    }

    // If no intent remains but pending_delete.journal is present, clean up orphaned files
    if journal_file.exists() {
        let _disp_rep = dispose_obsolete_files(storage_dir, Vec::new(), false).await?;
        return Ok(Some(PurgeReport {
            records_purged: 0,
            bytes_reclaimed: 0,
            sstables_replaced: 0,
        }));
    }

    Ok(None)
}
