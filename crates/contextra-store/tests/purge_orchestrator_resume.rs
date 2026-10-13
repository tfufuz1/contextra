use contextra_store::lsm::config::LsmConfig;
use contextra_store::lsm::purge::{
    resume_pending, OrxHooks, OrxPurgeIntent, PurgePhase,
};
use contextra_store::lsm::LsmStorage;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn test_resume_pending_no_intent_returns_ok_none() -> contextra_core::Result<()> {
    let dir = tempdir()?;
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::open(config).await?;
    let hooks = OrxHooks::new();

    let res = resume_pending(&storage, &hooks).await?;
    assert!(res.is_none());

    Ok(())
}

#[tokio::test]
async fn test_resume_pending_corrupted_intent_returns_err() -> contextra_core::Result<()> {
    let dir = tempdir()?;
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::open(config).await?;
    let intent_file = dir.path().join("purge.intent");

    std::fs::write(&intent_file, b"corrupted payload without crc")?;

    let hooks = OrxHooks::new();
    let res = resume_pending(&storage, &hooks).await;
    assert!(res.is_err(), "Corrupted intent must return Err");

    Ok(())
}

#[tokio::test]
async fn test_resume_pending_resumes_and_cleans_up() -> contextra_core::Result<()> {
    let dir = tempdir()?;
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::open(config).await?;
    let intent_file = dir.path().join("purge.intent");

    let intent = OrxPurgeIntent::new(
        [7; 16],
        [8; 32],
        1,
        None,
        None,
        100,
        PurgePhase::PurgeCompaction,
    );
    intent.save(&intent_file)?;

    let purge_executed = Arc::new(AtomicBool::new(false));
    let purge_executed_clone = purge_executed.clone();

    let mut hooks = OrxHooks::new();
    hooks.purge = Box::new(move |_scope, _tx| {
        purge_executed_clone.store(true, Ordering::SeqCst);
        Box::pin(async {
            Ok(contextra_store::compaction::purge_filter::PurgeReport {
                records_purged: 5,
                bytes_reclaimed: 500,
                sstables_replaced: 1,
            })
        })
    });

    let report = resume_pending(&storage, &hooks)
        .await?
        .expect("Resume report should be present");

    assert_eq!(report.resumed_from_phase, PurgePhase::PurgeCompaction);
    assert!(purge_executed.load(Ordering::SeqCst), "Purge closure must execute during resume");
    assert!(!intent_file.exists(), "Intent file must be unlinked in Release phase");

    Ok(())
}
