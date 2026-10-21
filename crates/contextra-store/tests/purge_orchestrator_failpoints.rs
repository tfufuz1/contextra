// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: Integrationstests für alle 14 Purge Failpoints und Resumption (Spezifikation Anhang B).

use contextra_store::compaction::purge_filter::PurgeScope;
use contextra_store::lsm::config::LsmConfig;
use contextra_store::lsm::purge::{
    resume_pending, OrxFailpointSet, OrxHooks, OrxPurgeIntent, PurgeOrchestrator, PurgePhase,
    CANONICAL_FAILPOINTS,
};
use contextra_store::lsm::LsmStorage;
use tempfile::tempdir;

#[tokio::test]
async fn test_all_14_failpoints_preserves_state_and_resumes() -> std::io::Result<()> {
    for fp_name in CANONICAL_FAILPOINTS {
        let dir = tempdir()?;
        let config = LsmConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        };
        let storage = LsmStorage::open(config).await.expect("open storage");

        let orchestrator = PurgeOrchestrator::new(&storage);
        let failpoint_set = OrxFailpointSet::new();
        failpoint_set.activate(fp_name);

        let scope = PurgeScope::Keys(vec![b"k_fp".to_vec()]);
        let orchestrator = orchestrator.with_failpoints(failpoint_set);

        let hooks = OrxHooks::default();
        let res = orchestrator
            .execute_purge_internal(scope, 100, None, &hooks, false)
            .await;

        assert!(
            res.is_err(),
            "Failpoint {} should interrupt execution",
            fp_name
        );

        let intent_path = dir.path().join("purge.intent");

        // Depending on failpoint location, intent file must exist or be cleaned up
        if *fp_name != "after_intent_cleanup" && *fp_name != "p0_init" {
            assert!(
                intent_path.exists(),
                "Intent file must remain intact at failpoint {}",
                fp_name
            );
            let intent = OrxPurgeIntent::read_from_file(&intent_path)?.expect("valid intent");
            assert!(
                intent.phase <= PurgePhase::Released,
                "Phase at failpoint {} must be recorded",
                fp_name
            );

            // First resume execution and verify clean completion
            let res_opt = resume_pending(dir.path(), &hooks, None)
                .await
                .expect("Resumption after failpoint should succeed");

            assert!(
                res_opt.is_some(),
                "Resumption after failpoint {} should complete",
                fp_name
            );
            assert!(
                !intent_path.exists(),
                "Intent file must be removed after successful resumption for {}",
                fp_name
            );

            // Second resume must be idempotent and return Ok(None)
            let res_opt_2 = resume_pending(dir.path(), &hooks, None)
                .await
                .expect("Second resumption should succeed idempotently");
            assert!(
                res_opt_2.is_none(),
                "Second resumption for {} should return None",
                fp_name
            );
        }
    }

    Ok(())
}
