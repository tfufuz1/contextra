use contextra_core::{DocId, StorageEngine, TxId};
use contextra_store::lsm::{LsmConfig, LsmStorage};
use tempfile::tempdir;

#[tokio::test]
async fn test_kv_engine_put_get_default_docid() {
    let dir = tempdir().expect("tempdir");
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };

    let storage = LsmStorage::new(config).await.expect("open storage");
    let tx1 = TxId::new(1);

    let key1 = b"test_key_alpha";
    let val1 = b"value_alpha";

    storage.put(tx1, key1, val1).await.expect("put key1");
    storage.commit(tx1).await.expect("commit tx1");

    let retrieved = storage.get(key1).await.expect("get key1");
    assert_eq!(retrieved.as_deref(), Some(&val1[..]));

    #[cfg(not(feature = "docid-128"))]
    {
        let hash = blake3::hash(key1);
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&hash.as_bytes()[..8]);
        let expected_doc_id = DocId::new(u64::from_le_bytes(bytes));
        assert_eq!(std::mem::size_of::<DocId>(), 8);
        assert_eq!(expected_doc_id.inner(), u64::from_le_bytes(bytes));
    }

    #[cfg(feature = "docid-128")]
    {
        let hash = blake3::hash(key1);
        let mut bytes = [0u8; 16];
        bytes.copy_from_slice(&hash.as_bytes()[..16]);
        let expected_doc_id = DocId::new(u128::from_le_bytes(bytes));
        assert_eq!(std::mem::size_of::<DocId>(), 16);
        assert_eq!(expected_doc_id.inner(), u128::from_le_bytes(bytes));
    }
}

#[cfg(feature = "docid-128")]
#[tokio::test]
async fn test_kv_engine_docid_128_uniqueness() {
    let dir = tempdir().expect("tempdir");
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };

    let storage = LsmStorage::new(config).await.expect("open storage");
    let tx = TxId::new(10);

    let key_a = b"doc_key_128_a";
    let key_b = b"doc_key_128_b";

    storage.put(tx, key_a, b"val_a").await.expect("put key_a");
    storage.put(tx, key_b, b"val_b").await.expect("put key_b");
    storage.commit(tx).await.expect("commit tx");

    let hash_a = blake3::hash(key_a);
    let hash_b = blake3::hash(key_b);

    let mut bytes_a = [0u8; 16];
    bytes_a.copy_from_slice(&hash_a.as_bytes()[..16]);
    let doc_id_a = DocId::new(u128::from_le_bytes(bytes_a));

    let mut bytes_b = [0u8; 16];
    bytes_b.copy_from_slice(&hash_b.as_bytes()[..16]);
    let doc_id_b = DocId::new(u128::from_le_bytes(bytes_b));

    assert_ne!(doc_id_a.inner(), doc_id_b.inner());
    assert_eq!(std::mem::size_of::<DocId>(), 16);
}
