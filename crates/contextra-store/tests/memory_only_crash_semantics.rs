use contextra_core::{StorageEngine, TxId};
use contextra_store::{DurabilityMode, LsmConfig, LsmStorage};
use tempfile::TempDir;

#[tokio::test]
async fn test_memory_only_crash_semantics() {
    let tmp = TempDir::new().expect("temp dir creation");
    let path = tmp.path().to_path_buf();

    let config = LsmConfig {
        path: path.clone(),
        durability_mode: DurabilityMode::MemoryOnly,
        ..Default::default()
    };

    {
        let storage = LsmStorage::new(config.clone())
            .await
            .expect("create MemoryOnly storage");

        let tx1 = TxId::new(1);
        storage.put(tx1, b"key1", b"val1").await.unwrap();
        storage.commit(tx1).await.unwrap();

        assert_eq!(
            storage.get(b"key1").await.unwrap(),
            Some(bytes::Bytes::from_static(b"val1"))
        );
    }

    // Re-open storage after process crash simulation
    {
        let storage = LsmStorage::new(config.clone())
            .await
            .expect("reopen MemoryOnly storage");

        assert_eq!(
            storage.get(b"key1").await.unwrap(),
            None,
            "MemoryOnly mode must lose all un-flushed data upon process restart"
        );
    }
}
