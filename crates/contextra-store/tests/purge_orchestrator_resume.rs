// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: Integrationstests für Purge-Orchestrator Resumption aus verschiedenen Phasen und Idempotenz.

use contextra_store::compaction::purge_filter::PurgeScope;
use contextra_store::lsm::purge::{resume_pending, OrxHooks, OrxPurgeIntent, PurgePhase};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn test_resume_pending_from_p1_intent_phase() -> std::io::Result<()> {
    let dir = tempdir()?;
    let intent_path = dir.path().join("purge.intent");

    let scope = PurgeScope::Keys(vec![b"k1".to_vec()]);
    let intent = OrxPurgeIntent::new([1u8; 16], &scope, 50, None, None, PurgePhase::P1Intent);
    intent.write_to_file(&intent_path)?;

    let wal_retired = Arc::new(AtomicBool::new(false));
    let wal_retired_clone = Arc::clone(&wal_retired);

    let mut hooks = OrxHooks::default();
    hooks.wal_retire = Some(Box::new(move || {
        wal_retired_clone.store(true, Ordering::SeqCst);
        Ok(())
    }));

    let report_opt = resume_pending(dir.path(), &hooks, None)
        .await
        .expect("Resumption should succeed");

    assert!(report_opt.is_some());
    assert!(wal_retired.load(Ordering::SeqCst), "WAL retire hook should have executed");
    assert!(!intent_path.exists(), "Intent file should be deleted on completion");

    Ok(())
}

#[tokio::test]
async fn test_resume_pending_from_p5_dispose_phase() -> std::io::Result<()> {
    let dir = tempdir()?;
    let intent_path = dir.path().join("purge.intent");

    let scope = PurgeScope::Keys(vec![b"k1".to_vec()]);
    let intent = OrxPurgeIntent::new([2u8; 16], &scope, 50, None, None, PurgePhase::P5Dispose);
    intent.write_to_file(&intent_path)?;

    let witness_called = Arc::new(AtomicBool::new(false));
    let witness_called_clone = Arc::clone(&witness_called);

    let mut hooks = OrxHooks::default();
    hooks.witness = Some(Box::new(move || {
        witness_called_clone.store(true, Ordering::SeqCst);
        Ok(())
    }));

    let report_opt = resume_pending(dir.path(), &hooks, None)
        .await
        .expect("Resumption should succeed");

    assert!(report_opt.is_some());
    assert!(witness_called.load(Ordering::SeqCst));
    assert!(!intent_path.exists());

    Ok(())
}

#[tokio::test]
async fn test_double_resume_is_idempotent() -> std::io::Result<()> {
    let dir = tempdir()?;
    let intent_path = dir.path().join("purge.intent");

    let scope = PurgeScope::Keys(vec![b"k1".to_vec()]);
    let intent = OrxPurgeIntent::new([3u8; 16], &scope, 50, None, None, PurgePhase::P1Intent);
    intent.write_to_file(&intent_path)?;

    let hooks = OrxHooks::default();

    // First resumption completes the purge and deletes intent
    let report_1 = resume_pending(dir.path(), &hooks, None)
        .await
        .expect("First resumption should succeed");
    assert!(report_1.is_some());
    assert!(!intent_path.exists());

    // Second resumption on already completed state is idempotent and returns Ok(None)
    let report_2 = resume_pending(dir.path(), &hooks, None)
        .await
        .expect("Second resumption should succeed");
    assert!(report_2.is_none());

    Ok(())
}
