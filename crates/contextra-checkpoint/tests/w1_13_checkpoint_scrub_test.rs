// FILE-CONTEXT
// STAND: 2026-10-21 (W1-13)
// ZWECK: Stage 4 Tests for Checkpoint-Scrub feature gating and copy removal.

use contextra_checkpoint::PersistentCheckpointStore;
use contextra_store::{LsmConfig, LsmStorage};
use contextra_types::TxId;
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn checkpoint_scrub_disabled_reports_unsupported() {
    let dir = tempdir().unwrap();
    let config = LsmConfig {
        path: dir.path().join("db"),
        ..Default::default()
    };
    let lsm = Arc::new(LsmStorage::open(config).await.unwrap());
    let store = PersistentCheckpointStore::open(lsm, "test_ns").await.unwrap();

    #[cfg(not(feature = "checkpoint"))]
    {
        let res_coll = store.scrub_collection_checkpoints("col1").await;
        assert!(res_coll.is_err());
        match res_coll {
            Err(contextra_types::ContextraError::CapabilityUnsupported { capability, .. }) => {
                assert_eq!(capability, "checkpoint");
            }
            _ => panic!("Expected ContextraError::CapabilityUnsupported"),
        }

        let res_tenant = store.scrub_tenant_checkpoints("tenant1").await;
        assert!(res_tenant.is_err());

        let res_purge = store.purge_checkpoint_by_name("cp1").await;
        assert!(res_purge.is_err());
    }

    #[cfg(feature = "checkpoint")]
    {
        let count = store.scrub_collection_checkpoints("col1").await.unwrap();
        assert_eq!(count, 0);
    }
}

#[tokio::test]
async fn checkpoint_scrub_removes_copies_raw_sweep() {
    let dir = tempdir().unwrap();
    let config = LsmConfig {
        path: dir.path().join("db"),
        ..Default::default()
    };
    let lsm = Arc::new(LsmStorage::open(config).await.unwrap());
    let store = PersistentCheckpointStore::open(lsm, "test_ns").await.unwrap();

    let tx = TxId::try_new(10).unwrap();
    let _cp = store
        .create_checkpoint("cp_to_scrub", "col_scrub", 10, tx, serde_json::json!({}))
        .await
        .unwrap();

    let listed = store.list_checkpoints().await.unwrap();
    assert_eq!(listed.len(), 1);

    #[cfg(feature = "checkpoint")]
    {
        let count = store.scrub_collection_checkpoints("col_scrub").await.unwrap();
        assert_eq!(count, 1);

        let remaining = store.list_checkpoints().await.unwrap();
        assert_eq!(remaining.len(), 0);
    }
}
