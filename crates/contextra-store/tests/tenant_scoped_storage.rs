//! Integration and isolation tests for `TenantScopedStorage`.

use contextra_core::{TenantId, TxId};
use contextra_ports::StorageEngine;
use contextra_store::{tenant_codec::TenantScopedStorage, LsmConfig, LsmStorage};
use proptest::prelude::*;
use std::sync::Arc;
use tempfile::TempDir;

async fn setup_test_lsm() -> (TempDir, Arc<LsmStorage>) {
    let dir = TempDir::new().expect("Failed to create tempdir");
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        memtable_size_limit: 1024 * 1024,
        ..Default::default()
    };
    let lsm = LsmStorage::new(config)
        .await
        .expect("Failed to open LsmStorage");
    (dir, Arc::new(lsm))
}

#[tokio::test]
async fn test_tenant_a_cannot_read_tenant_b() {
    let (_dir, lsm) = setup_test_lsm().await;
    let tenant1 = TenantId::try_new(101).unwrap();
    let tenant2 = TenantId::try_new(102).unwrap();

    let store_a = TenantScopedStorage::new(lsm.clone(), tenant1);
    let store_b = TenantScopedStorage::new(lsm, tenant2);

    let tx1 = TxId(1);
    let tx2 = TxId(2);

    // Put under Tenant A
    store_a.put(tx1, b"key1", b"val_a").await.expect("put key1");
    store_a.commit(tx1).await.expect("commit tx1");

    // Tenant B should not read key1
    let res_b = store_b.get(b"key1").await.expect("get key1");
    assert_eq!(res_b, None, "Tenant B must not read Tenant A key");

    // Tenant A reads key1
    let res_a = store_a.get(b"key1").await.expect("get key1");
    assert_eq!(res_a.as_deref(), Some(&b"val_a"[..]));

    // Put under Tenant B
    store_b.put(tx2, b"key1", b"val_b").await.expect("put key1");
    store_b.commit(tx2).await.expect("commit tx2");

    // Each gets their own value
    let res_a = store_a.get(b"key1").await.expect("get key1");
    let res_b = store_b.get(b"key1").await.expect("get key1");
    assert_eq!(res_a.as_deref(), Some(&b"val_a"[..]));
    assert_eq!(res_b.as_deref(), Some(&b"val_b"[..]));

    // Scan prefix test
    let scan_a = store_a.scan_prefix(b"key").await.expect("scan_a");
    let scan_b = store_b.scan_prefix(b"key").await.expect("scan_b");
    assert_eq!(scan_a, vec![(b"key1".to_vec(), b"val_a".to_vec())]);
    assert_eq!(scan_b, vec![(b"key1".to_vec(), b"val_b".to_vec())]);

    // Delete prefix under Tenant A
    let tx3 = TxId(3);
    let deleted = store_a
        .delete_prefix(tx3, b"key")
        .await
        .expect("delete_prefix");
    assert_eq!(deleted, 1);
    store_a.commit(tx3).await.expect("commit tx3");

    // Tenant A is deleted, Tenant B remains
    assert_eq!(store_a.get(b"key1").await.expect("get"), None);
    assert_eq!(
        store_b.get(b"key1").await.expect("get").as_deref(),
        Some(&b"val_b"[..])
    );
}

#[tokio::test]
async fn test_snapshot_reads() {
    let (_dir, lsm) = setup_test_lsm().await;
    let tenant1 = TenantId::try_new(201).unwrap();
    let store = TenantScopedStorage::new(lsm, tenant1);

    let tx1 = TxId(10);
    store.put(tx1, b"snap_key", b"v1").await.unwrap();
    store.commit(tx1).await.unwrap();

    let seq1 = store.last_seq_no().await.unwrap();

    let tx2 = TxId(11);
    store.put(tx2, b"snap_key", b"v2").await.unwrap();
    store.commit(tx2).await.unwrap();

    let val_at_seq1 = store.get_at_seq(b"snap_key", seq1).await.unwrap();
    assert_eq!(val_at_seq1.as_deref(), Some(&b"v1"[..]));

    let scan_at_seq1 = store.scan_prefix_at(b"snap", seq1).await.unwrap();
    assert_eq!(scan_at_seq1, vec![(b"snap_key".to_vec(), b"v1".to_vec())]);
}

#[tokio::test]
async fn test_tenant_deletion_all() {
    let (_dir, lsm) = setup_test_lsm().await;
    let tenant1 = TenantId::try_new(301).unwrap();
    let tenant2 = TenantId::try_new(302).unwrap();

    let store1 = TenantScopedStorage::new(lsm.clone(), tenant1);
    let store2 = TenantScopedStorage::new(lsm, tenant2);

    let tx1 = TxId(1);
    store1.put(tx1, b"a", b"1").await.unwrap();
    store1.put(tx1, b"b", b"2").await.unwrap();
    store2.put(tx1, b"a", b"3").await.unwrap();
    store1.commit(tx1).await.unwrap();

    let tx2 = TxId(2);
    // Delete all keys for tenant 1 using empty prefix
    let deleted = store1.delete_prefix(tx2, b"").await.unwrap();
    assert_eq!(deleted, 2);
    store1.commit(tx2).await.unwrap();

    assert!(store1.scan_prefix(b"").await.unwrap().is_empty());
    assert_eq!(store2.get(b"a").await.unwrap().as_deref(), Some(&b"3"[..]));
}

proptest! {
    #[test]
    fn scan_prefix_never_crosses_tenant_boundary(
        key_suffix in prop::collection::vec(any::<u8>(), 0..64),
        val_bytes in prop::collection::vec(any::<u8>(), 0..64),
    ) {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async move {
            let (_dir, lsm) = setup_test_lsm().await;
            let tenant1 = TenantId::try_new(501).unwrap();
            let tenant2 = TenantId::try_new(502).unwrap();

            let store1 = TenantScopedStorage::new(lsm.clone(), tenant1);
            let store2 = TenantScopedStorage::new(lsm, tenant2);

            let tx1 = TxId(100);
            store1.put(tx1, &key_suffix, &val_bytes).await.unwrap();
            store1.commit(tx1).await.unwrap();

            // Tenant 2 scanning empty prefix or key prefix must never retrieve tenant 1 key
            let scan2_all = store2.scan_prefix(b"").await.unwrap();
            assert!(scan2_all.is_empty());

            let scan2_suffix = store2.scan_prefix(&key_suffix).await.unwrap();
            assert!(scan2_suffix.is_empty());

            let get2 = store2.get(&key_suffix).await.unwrap();
            assert_eq!(get2, None);
        });
    }
}
