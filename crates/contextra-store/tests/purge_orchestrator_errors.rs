// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: Tests für Fehlerbehandlung, PurgeOutcome, WAL-Callback-Fehler und Resumption im PurgeOrchestrator.
// INVARIANTEN:
// - Fail-closed Error Handling: Fehler in record_phase oder WAL-Callback führen zu PurgeOutcome::Failed.
// - State & Fence Consistency: Bei Fehler wird das Erasure Fence freigegeben und das Intent-File bleibt für resume_pending auf Disk.

use contextra_store::compaction::purge_filter::PurgeScope;
use contextra_store::lsm::purge::orchestrator::PurgeOutcome;
use contextra_store::lsm::purge::{
    resume_pending, OrxFailpointSet, OrxHooks, OrxPurgeIntent, PurgeOrchestrator, PurgePhase,
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn test_record_phase_failpoint_returns_failed_and_preserves_intent() -> std::io::Result<()> {
    let dir = tempdir()?;
    let orchestrator = PurgeOrchestrator::new_standalone(dir.path());

    let failpoint_set = OrxFailpointSet::new();
    failpoint_set.activate("after_intent");
    let orchestrator = orchestrator.with_failpoints(failpoint_set);

    let scope = PurgeScope::Keys(vec![b"k1".to_vec()]);
    let hooks = OrxHooks::default();

    let outcome = orchestrator
        .execute_purge_outcome(scope, 100, None, &hooks, false)
        .await;

    match outcome {
        PurgeOutcome::Failed { phase, error_msg } => {
            assert_eq!(phase, PurgePhase::P1Intent);
            assert!(
                error_msg.contains("after_intent"),
                "Error message should mention failpoint name: {}",
                error_msg
            );
        }
        other => panic!("Expected PurgeOutcome::Failed, got {:?}", other),
    }

    let intent_path = dir.path().join("purge.intent");
    assert!(
        intent_path.exists(),
        "Intent file must remain intact on disk after record_phase error"
    );

    let intent = OrxPurgeIntent::read_from_file(&intent_path)?.expect("valid intent on disk");
    assert_eq!(intent.phase, PurgePhase::P1Intent);

    Ok(())
}

#[tokio::test]
async fn test_wal_callback_error_returns_failed_and_releases_fence() -> std::io::Result<()> {
    let dir = tempdir()?;
    let orchestrator = PurgeOrchestrator::new_standalone(dir.path());

    let fence_acquired = Arc::new(AtomicBool::new(false));
    let fence_released = Arc::new(AtomicBool::new(false));

    let acq_clone = Arc::clone(&fence_acquired);
    let rel_clone = Arc::clone(&fence_released);

    let mut hooks = OrxHooks::default();
    hooks.fence_acquire = Some(Box::new(move || {
        acq_clone.store(true, Ordering::SeqCst);
        Ok(())
    }));
    hooks.fence_release = Some(Box::new(move || {
        rel_clone.store(true, Ordering::SeqCst);
        Ok(())
    }));

    // WAL callback fails with IO error
    hooks.wal_retire = Some(Box::new(|| {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "WAL callback simulated failure",
        ))
    }));

    let scope = PurgeScope::Keys(vec![b"k_wal".to_vec()]);
    let outcome = orchestrator
        .execute_purge_outcome(scope, 100, None, &hooks, false)
        .await;

    match outcome {
        PurgeOutcome::Failed { phase, error_msg } => {
            assert_eq!(phase, PurgePhase::P6WalCallback);
            assert!(
                error_msg.contains("WAL callback simulated failure"),
                "Error msg: {}",
                error_msg
            );
        }
        other => panic!("Expected PurgeOutcome::Failed, got {:?}", other),
    }

    assert!(
        fence_acquired.load(Ordering::SeqCst),
        "Fence must have been acquired"
    );
    assert!(
        fence_released.load(Ordering::SeqCst),
        "Fence must be released upon failure"
    );

    let intent_path = dir.path().join("purge.intent");
    assert!(
        intent_path.exists(),
        "Intent file must exist on disk after WAL callback error"
    );

    let intent = OrxPurgeIntent::read_from_file(&intent_path)?.expect("valid intent");
    assert_eq!(intent.phase, PurgePhase::P6WalCallback);

    Ok(())
}

#[tokio::test]
async fn test_repetition_after_wal_error_completes_purge() -> std::io::Result<()> {
    let dir = tempdir()?;
    let orchestrator = PurgeOrchestrator::new_standalone(dir.path());

    let mut hooks = OrxHooks::default();
    hooks.wal_retire = Some(Box::new(|| {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Temporary WAL error",
        ))
    }));

    let scope = PurgeScope::Keys(vec![b"k_retry".to_vec()]);

    let outcome_1 = orchestrator
        .execute_purge_outcome(scope, 100, None, &hooks, false)
        .await;

    assert!(
        matches!(outcome_1, PurgeOutcome::Failed { .. }),
        "First execution should fail"
    );

    // Fix WAL callback for retry
    let wal_executed = Arc::new(AtomicBool::new(false));
    let wal_exec_clone = Arc::clone(&wal_executed);
    hooks.wal_retire = Some(Box::new(move || {
        wal_exec_clone.store(true, Ordering::SeqCst);
        Ok(())
    }));

    // Resume execution
    let res_opt = resume_pending(dir.path(), &hooks, None)
        .await
        .expect("Resumption after fixing WAL error should succeed");

    assert!(res_opt.is_some(), "Resumption should complete successfully");
    assert!(
        wal_executed.load(Ordering::SeqCst),
        "WAL callback must have been executed during resumption"
    );

    let intent_path = dir.path().join("purge.intent");
    assert!(
        !intent_path.exists(),
        "Intent file must be removed after successful completion"
    );

    // Second resumption must return None idempotently
    let res_opt_2 = resume_pending(dir.path(), &hooks, None)
        .await
        .expect("Second resumption should succeed idempotently");
    assert!(res_opt_2.is_none(), "Second resumption must return None");

    Ok(())
}
