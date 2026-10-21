// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: Tests für Fehlerbehandlung, PurgeOutcome, WAL-Callback-Fehler und Resumption im PurgeOrchestrator.
// INVARIANTEN:
// - Fail-closed Error Handling: Fehler in record_phase oder WAL-Callback führen zu PurgeOutcome::Failed.
// - State & Fence Consistency: Bei Fehler wird das Erasure Fence freigegeben und das Intent-File bleibt für resume_pending auf Disk.

use contextra_store::compaction::purge_filter::PurgeScope;
use contextra_store::lsm::config::LsmConfig;
use contextra_store::lsm::LsmStorage;
use contextra_store::lsm::purge::orchestrator::PurgeOutcome;
use contextra_store::lsm::purge::{
    resume_pending, MockBackend, OrxFailpointSet, OrxHooks, OrxPurgeIntent, PurgeOrchestrator, PurgePhase,
};
use tempfile::tempdir;

#[tokio::test]
async fn test_record_phase_failpoint_returns_failed_and_preserves_intent() -> std::io::Result<()> {
    let dir = tempdir()?;
    let backend = MockBackend::new();
    let orchestrator = PurgeOrchestrator::new(&backend, dir.path());

    let failpoint_set = OrxFailpointSet::new();
    failpoint_set.activate("after_intent");
    let orchestrator = orchestrator.with_failpoints(failpoint_set);

    let scope = PurgeScope::Keys(vec![b"k1".to_vec()]);

    let hooks = OrxHooks::default();
    let outcome = orchestrator
        .execute_purge_outcome(scope, 100, None, &hooks, false)
        .await;

    match outcome {
        PurgeOutcome::Failed { phase, error } => {
            assert_eq!(phase, PurgePhase::Admitted);
            let err_str = format!("{:?}", error);
            assert!(
                err_str.contains("after_intent") || err_str.contains("FailpointTriggered"),
                "Error should mention failpoint name: {}",
                err_str
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
    assert_eq!(intent.phase, PurgePhase::Admitted);

    Ok(())
}

#[tokio::test]
async fn test_wal_callback_error_returns_failed_and_releases_fence() -> std::io::Result<()> {
    let dir = tempdir()?;
    let backend = MockBackend::new();

    let orchestrator = PurgeOrchestrator::new(&backend, dir.path());

    let mut hooks = OrxHooks::default();
    hooks.wal_retire = Some(Box::new(|| {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Simulated WAL retire failure",
        ))
    }));

    let scope = PurgeScope::Keys(vec![b"k_wal".to_vec()]);
    let outcome = orchestrator
        .execute_purge_outcome(scope, 100, None, &hooks, false)
        .await;

    match outcome {
        PurgeOutcome::Failed { phase, error } => {
            assert_eq!(phase, PurgePhase::WalRetiring);
            let err_str = format!("{:?}", error);
            assert!(
                err_str.contains("Simulated WAL retire failure"),
                "Error msg: {}",
                err_str
            );
        }
        other => panic!("Expected PurgeOutcome::Failed, got {:?}", other),
    }

    let intent_path = dir.path().join("purge.intent");
    assert!(
        intent_path.exists(),
        "Intent file must exist on disk after WAL callback error"
    );

    let intent = OrxPurgeIntent::read_from_file(&intent_path)?.expect("valid intent");
    assert_eq!(intent.phase, PurgePhase::WalRetiring);

    Ok(())
}

#[tokio::test]
async fn test_repetition_after_wal_error_completes_purge() -> std::io::Result<()> {
    let dir = tempdir()?;
    let backend = MockBackend::new();

    let orchestrator = PurgeOrchestrator::new(&backend, dir.path());

    let mut hooks = OrxHooks::default();
    hooks.wal_retire = Some(Box::new(|| {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Simulated WAL retire failure",
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
    hooks.wal_retire = None;

    // Resume execution
    let res_opt = resume_pending_for_storage(&storage, &hooks, None)
        .await
        .expect("Resumption after fixing WAL error should succeed");

    assert!(res_opt.is_some(), "Resumption should complete successfully");

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
