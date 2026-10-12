// FILE-CONTEXT
// STAND: 2026-10-12
// ZWECK: Resume-Logik für abgebrochene/abgestürzte Purge-Operationen (C5.1 / Anhang B).
// INVARIANTEN: Korruption der Intent-Datei schlägt fehl (Err, nie Ok(None)). Jede Phase ist idempotent.

use super::orchestrator::{OrxHooks, PurgeOrchestrator};
use super::plan::PurgePhase;

use crate::compaction::purge_filter::PurgeScope;
use crate::lsm::engine::LsmStorage;
use contextra_core::{ContextraError, Result};

/// Summary report of a resumed purge operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrxResumeReport {
    /// Initial phase recorded in `purge.intent` before resume.
    pub resumed_from_phase: PurgePhase,
}

/// Checks for `purge.intent` or `pending_delete.journal` in `storage.config.path` and resumes pending purge work.
pub async fn resume_pending(
    storage: &LsmStorage,
    hooks: &OrxHooks<'_>,
) -> Result<Option<OrxResumeReport>> {
    let intent_file = storage.config.path.join("purge.intent");
    let journal_file = storage.config.path.join("pending_delete.journal");

    if !intent_file.exists() {
        if journal_file.exists() {
            // Pending delete journal exists without intent file; process pending deletes
            let obs_files = (hooks.obsolete_files)();
            let _ = super::dispose::execute_dispose(journal_file, obs_files, hooks.secure_delete)
                .await
                .map_err(|e| ContextraError::Storage(format!("Dispose resume failed: {}", e)))?;
            return Ok(Some(OrxResumeReport {
                resumed_from_phase: PurgePhase::Dispose,
            }));
        }
        return Ok(None);
    }

    // Load intent file. Failure to load/CRC mismatch MUST yield Err, never Ok(None).
    let intent = super::intent::OrxPurgeIntent::load(&intent_file)
        .map_err(|e| ContextraError::Storage(format!("Corrupted purge intent: {}", e)))?;

    let initial_phase = intent.phase;

    let scope = PurgeScope::Keys(vec![]);

    let orchestrator = PurgeOrchestrator::new(storage);
    orchestrator
        .execute_from(intent, scope, hooks)
        .await
        .map_err(|e| ContextraError::Storage(format!("Resumed purge failed: {}", e)))?;

    Ok(Some(OrxResumeReport {
        resumed_from_phase: initial_phase,
    }))
}
