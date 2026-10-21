// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: Integrationstests für Purge-Orchestrator Resumption aus verschiedenen Phasen und Idempotenz.

use contextra_store::compaction::purge_filter::PurgeScope;
use contextra_store::lsm::purge::{resume_pending, OrxHooks, OrxPurgeIntent, PurgePhase};
use tempfile::tempdir;

#[tokio::test]
async fn test_resume_pending_from_admitted_phase() -> std::io::Result<()> {
    let dir = tempdir()?;
    let intent_path = dir.path().join("purge.intent");

    let scope = PurgeScope::Keys(vec![b"k1".to_vec()]);
    let purge_id = [1u8; 16];

    let intent = OrxPurgeIntent::new(purge_id, &scope, 50, None, None, PurgePhase::Admitted);
    intent.write_to_file(&intent_path)?;

    let hooks = OrxHooks::default();

    let report_opt = resume_pending(dir.path(), &hooks, None)
        .await
        .expect("Resumption should succeed");

    assert!(report_opt.is_some());
    assert!(
        !intent_path.exists(),
        "Intent file should be deleted on completion"
    );

    Ok(())
}

#[tokio::test]
async fn test_resume_pending_from_disposing_phase() -> std::io::Result<()> {
    let dir = tempdir()?;
    let intent_path = dir.path().join("purge.intent");

    let scope = PurgeScope::Keys(vec![b"k1".to_vec()]);
    let purge_id = [2u8; 16];

    let intent = OrxPurgeIntent::new(purge_id, &scope, 50, None, None, PurgePhase::Disposing);
    intent.write_to_file(&intent_path)?;

    let hooks = OrxHooks::default();

    let report_opt = resume_pending(dir.path(), &hooks, None)
        .await
        .expect("Resumption should succeed");

    assert!(report_opt.is_some());
    assert!(!intent_path.exists());

    Ok(())
}

#[tokio::test]
async fn test_double_resume_is_idempotent() -> std::io::Result<()> {
    let dir = tempdir()?;
    let intent_path = dir.path().join("purge.intent");

    let scope = PurgeScope::Keys(vec![b"k1".to_vec()]);
    let purge_id = [3u8; 16];

    let intent = OrxPurgeIntent::new(purge_id, &scope, 50, None, None, PurgePhase::Admitted);
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
