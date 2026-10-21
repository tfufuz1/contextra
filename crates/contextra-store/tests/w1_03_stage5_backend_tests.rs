// FILE-CONTEXT
// STAND: 2026-10-21
// ZWECK: Stage 5 mandatory tests for PurgeBackend and recovery resumption.

use contextra_store::compaction::purge_filter::PurgeScope;
use contextra_store::lsm::purge::{
    resume_pending_for_backend, MockBackend, OrxHooks, OrxPurgeIntent, PurgePhase,
};
use tempfile::tempdir;

#[tokio::test]
async fn resume_with_mock_backend_calls_purge_keys() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let mock = MockBackend::new();

    let scope = PurgeScope::Keys(vec![b"key_mock".to_vec()]);
    let intent = OrxPurgeIntent::new([11u8; 16], &scope, 100, None, None, PurgePhase::Admitted);
    intent.write_to_file(&dir.path().join("purge.intent"))?;

    let hooks = OrxHooks::default();
    let res = resume_pending_for_backend(
        &mock,
        dir.path(),
        &hooks,
        None,
        contextra_store::lsm::purge::dispose::DisposePolicy::Disabled,
    )
    .await?;

    assert!(res.is_some(), "Resume with mock backend must return report");
    assert_eq!(
        mock.purge_keys_calls.load(std::sync::atomic::Ordering::SeqCst),
        1,
        "MockBackend.purge_keys must be invoked during resume"
    );
    assert!(!dir.path().join("purge.intent").exists(), "Intent file must be removed");

    Ok(())
}

#[tokio::test]
async fn resume_detects_residue_and_keeps_intent() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let mut mock = MockBackend::new();
    mock.residue_detected = true;

    let scope = PurgeScope::Keys(vec![b"key_residue".to_vec()]);
    let intent = OrxPurgeIntent::new([12u8; 16], &scope, 100, None, None, PurgePhase::Admitted);
    intent.write_to_file(&dir.path().join("purge.intent"))?;

    let hooks = OrxHooks::default();
    let res = resume_pending_for_backend(
        &mock,
        dir.path(),
        &hooks,
        None,
        contextra_store::lsm::purge::dispose::DisposePolicy::Disabled,
    )
    .await;

    assert!(res.is_err(), "Resume must fail when residue is detected");
    assert!(
        dir.path().join("purge.intent").exists(),
        "Intent file must remain on disk when residue is detected"
    );

    Ok(())
}

#[tokio::test]
async fn resume_executes_purge() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let mock = MockBackend::new();

    let scope = PurgeScope::Keys(vec![b"k_exec".to_vec()]);
    let intent = OrxPurgeIntent::new([13u8; 16], &scope, 100, None, None, PurgePhase::Admitted);
    intent.write_to_file(&dir.path().join("purge.intent"))?;

    let hooks = OrxHooks::default();
    let res = resume_pending_for_backend(
        &mock,
        dir.path(),
        &hooks,
        None,
        contextra_store::lsm::purge::dispose::DisposePolicy::Disabled,
    )
    .await?;

    assert!(res.is_some(), "Resume must execute purge rather than no-op");
    Ok(())
}
