use contextra_ports::StorageEngine;
use contextra_types::TxId;
use contextra_store::compaction::purge_filter::{FreeSpaceProbe, PurgeScope};
use contextra_store::lsm::config::LsmConfig;
use contextra_store::lsm::purge::{OrxHooks, PurgeOrchestrator};
use contextra_store::lsm::LsmStorage;
use tempfile::tempdir;

struct TestFreeSpaceProbe;
impl FreeSpaceProbe for TestFreeSpaceProbe {
    fn available_bytes(&self) -> std::result::Result<Option<u64>, contextra_ports::ProbeError> {
        Ok(Some(100 * 1024 * 1024))
    }

    fn free_space_bytes(&self) -> Option<u64> {
        Some(100 * 1024 * 1024)
    }
}

#[tokio::test]
async fn test_purge_orchestrator_execution() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::open(config).await?;

    let tx1 = TxId::try_new(1)?;
    storage.put(tx1, b"p_key", b"p_val").await?;
    storage.commit(tx1).await?;

    let orchestrator = PurgeOrchestrator::new(&storage, dir.path());
    let scope = PurgeScope::Keys(vec![b"p_key".to_vec()]);
    let probe = TestFreeSpaceProbe;
    let hooks = OrxHooks::default();

    let report = orchestrator
        .execute_purge_internal(scope, 100, Some(&probe), &hooks, false)
        .await
        .expect("Purge orchestration should succeed");

    assert!(report.records_purged >= 1);

    Ok(())
}
