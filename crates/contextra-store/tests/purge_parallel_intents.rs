// FILE-CONTEXT
// STAND: 2026-10-16
// ZWECK: Testet zwei parallele Purge-Operationen mit eigenen Intent-Dateien.

use contextra_store::compaction::purge_filter::PurgeScope;
use contextra_store::lsm::config::LsmConfig;
use contextra_store::lsm::purge::{OrxHooks, OrxPurgeIntent, PurgeOrchestrator, PurgePhase};
use contextra_store::lsm::LsmStorage;
use tempfile::tempdir;

#[tokio::test]
async fn test_parallel_purges_with_distinct_intent_files() -> std::io::Result<()> {
    let dir = tempdir()?;
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::open(config).await.expect("open storage");

    let orchestrator1 = PurgeOrchestrator::new(&storage);
    let orchestrator2 = PurgeOrchestrator::new(&storage);

    let scope1 = PurgeScope::Keys(vec![b"key_purge_1".to_vec()]);
    let scope2 = PurgeScope::Keys(vec![b"key_purge_2".to_vec()]);

    let mut intent1 = OrxPurgeIntent::new([1u8; 16], &scope1, 10, None, None, PurgePhase::P1Intent);
    let mut intent2 = OrxPurgeIntent::new([2u8; 16], &scope2, 20, None, None, PurgePhase::P1Intent);

    orchestrator1.record_phase(&mut intent1, PurgePhase::P1Intent).expect("record intent1");
    orchestrator2.record_phase(&mut intent2, PurgePhase::P1Intent).expect("record intent2");

    let intent1_path = PurgeOrchestrator::intent_path_for_id(dir.path(), &[1u8; 16]);
    let intent2_path = PurgeOrchestrator::intent_path_for_id(dir.path(), &[2u8; 16]);

    assert!(intent1_path.exists(), "intent 1 file must exist at {:?}", intent1_path);
    assert!(intent2_path.exists(), "intent 2 file must exist at {:?}", intent2_path);
    assert_ne!(intent1_path, intent2_path);

    let all_intents = OrxPurgeIntent::read_all_intents(dir.path())?;
    assert_eq!(all_intents.len(), 2);

    let hooks = OrxHooks::default();

    let outcome1 = orchestrator1
        .execute_from_intent_outcome(intent1, scope1, &hooks, false, None)
        .await;
    assert!(outcome1.is_success());

    assert!(!intent1_path.exists(), "intent 1 must be removed after completion");
    assert!(intent2_path.exists(), "intent 2 must still exist");

    let outcome2 = orchestrator2
        .execute_from_intent_outcome(intent2, scope2, &hooks, false, None)
        .await;
    assert!(outcome2.is_success());

    assert!(!intent2_path.exists(), "intent 2 must be removed after completion");

    Ok(())
}
