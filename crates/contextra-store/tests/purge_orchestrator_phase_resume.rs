// FILE-CONTEXT
// STAND: 2026-10-22
// ZWECK: Regressionstest für Phase-Resumption im PurgeOrchestrator.
// INVARIANTEN:
// - Idempotente Wiederaufnahme: Wiederaufnahme ab Phase Finalizing führt den Witness-Hook NICHT erneut aus.
// - Status-Fortschritt & Intent-Bereinigung: Resumption schließt Phase Finalizing und Released erfolgreich ab.

use contextra_store::compaction::purge_filter::PurgeScope;
use contextra_store::lsm::purge::{
    MockBackend, OrxHooks, OrxPurgeIntent, PurgeOrchestrator, PurgePhase,
};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn test_resume_from_finalizing_skips_witness_hook() -> std::io::Result<()> {
    let dir = tempdir()?;
    let backend = MockBackend::new();
    let orchestrator = PurgeOrchestrator::new(&backend, dir.path());

    let scope = PurgeScope::Keys(vec![b"k_resume".to_vec()]);
    let witness_calls = Arc::new(AtomicUsize::new(0));
    let witness_calls_clone = Arc::clone(&witness_calls);

    let mut hooks = OrxHooks::default();
    hooks.witness = Some(Box::new(move || {
        witness_calls_clone.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }));

    // Construct intent starting at Finalizing (Witnessing was already completed)
    let intent = OrxPurgeIntent::new([42u8; 16], &scope, 100, None, None, PurgePhase::Finalizing);

    let outcome = orchestrator
        .execute_from_intent_outcome(intent, scope, &hooks, false, None)
        .await;

    assert!(
        outcome.is_success(),
        "Purge resumption from Finalizing should succeed: {:?}",
        outcome
    );

    assert_eq!(
        witness_calls.load(Ordering::SeqCst),
        0,
        "Witness hook must NOT be re-executed when resuming at Finalizing"
    );

    let intent_path = dir.path().join("purge.intent");
    assert!(
        !intent_path.exists(),
        "Intent file must be removed after successful pipeline completion"
    );

    Ok(())
}
