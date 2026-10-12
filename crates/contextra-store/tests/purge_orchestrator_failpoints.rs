use contextra_store::compaction::purge_filter::PurgeScope;
use contextra_store::lsm::config::LsmConfig;
use contextra_store::lsm::purge::{
    resume_pending, OrxFailpointSet, OrxHooks, OrxPurgeError, OrxPurgeIntent, PurgeOrchestrator,
    ENGINE_PURGE_FAILPOINTS, STORE_PURGE_FAILPOINTS,
};
use contextra_store::lsm::LsmStorage;
use tempfile::tempdir;

#[tokio::test]
async fn test_store_failpoints_crash_and_resume() -> contextra_core::Result<()> {
    for fp in STORE_PURGE_FAILPOINTS {
        let dir = tempdir()?;
        let config = LsmConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        };
        let storage = LsmStorage::open(config).await?;
        let orchestrator = PurgeOrchestrator::new(&storage);

        let failpoints = OrxFailpointSet::new();
        failpoints.enable(fp);

        let hooks = OrxHooks::new().with_failpoints(failpoints.clone());
        let scope = PurgeScope::Keys(vec![b"k1".to_vec()]);

        let res = orchestrator.execute_purge_with_hooks(scope.clone(), 100, &hooks).await;

        match res {
            Err(OrxPurgeError::Interrupted { point }) => {
                assert_eq!(point, fp, "Triggered failpoint must match enabled point");
            }
            other => panic!("Failpoint {} should trigger Interrupted error, got: {:?}", fp, other),
        }

        let intent_file = dir.path().join("purge.intent");
        assert!(intent_file.exists(), "Intent file must persist on disk after crash at {}", fp);

        let intent = OrxPurgeIntent::load(&intent_file)?;
        assert!(intent.attempt >= 1);

        // Resume after crash
        let clean_hooks = OrxHooks::new();
        let resume_res = resume_pending(&storage, &clean_hooks).await?;
        assert!(resume_res.is_some(), "Resume must succeed after crash at {}", fp);

        assert!(!intent_file.exists(), "Intent file must be unlinked after successful resume from {}", fp);
    }

    Ok(())
}

#[tokio::test]
async fn test_engine_failpoints_inventory_b05() -> contextra_core::Result<()> {
    for fp in ENGINE_PURGE_FAILPOINTS {
        let dir = tempdir()?;
        let config = LsmConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        };
        let storage = LsmStorage::open(config).await?;
        let orchestrator = PurgeOrchestrator::new(&storage);

        let failpoints = OrxFailpointSet::new();
        failpoints.enable(fp);

        let hooks = OrxHooks::new().with_failpoints(failpoints);
        let scope = PurgeScope::Keys(vec![b"k1".to_vec()]);

        let res = orchestrator.execute_purge_with_hooks(scope, 100, &hooks).await;

        match res {
            Err(OrxPurgeError::Interrupted { point }) => {
                assert_eq!(point, fp);
            }
            other => panic!("Engine failpoint {} should trigger Interrupted error, got: {:?}", fp, other),
        }

        let intent_file = dir.path().join("purge.intent");
        assert!(intent_file.exists(), "Engine failpoint {} must preserve intent for B05", fp);
    }

    Ok(())
}
