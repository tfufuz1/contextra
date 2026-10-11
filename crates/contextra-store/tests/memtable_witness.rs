use contextra_core::{TenantId, TxId};
use contextra_crypto::deletion_proof::DeletionLayer;
use contextra_crypto::deletion_witness::WitnessEvidence;
use contextra_ports::StorageEngine;
use contextra_store::compaction::CompactionConfig;
use contextra_store::lsm::{LsmConfig, LsmStorage};
use contextra_store::tenant_codec::TenantScopedStorage;
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;

async fn create_test_storage() -> (LsmStorage, TempDir) {
    let tmp = TempDir::new().expect("temp dir");
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        memtable_size_limit: 1024 * 1024,
        max_ram_mb: 64,
        tx_timeout: Duration::from_secs(60),
        compaction: CompactionConfig::default(),
        encryption_passphrase: None,
        ..Default::default()
    };
    let storage = LsmStorage::new(config).await.expect("create storage");
    (storage, tmp)
}

#[tokio::test]
async fn test_a_put_in_active_memtable_err() {
    let (storage, _tmp) = create_test_storage().await;
    let tx1 = TxId::new(1);
    let key = b"key_a".to_vec();

    storage.put(tx1, &key, b"val_a").await.unwrap();
    storage.commit(tx1).await.unwrap();

    let res = storage
        .issue_memtable_witness(&[key.clone()], &[], tx1)
        .await;
    assert!(res.is_err(), "Expected Err when non-tombstone Put version exists in active memtable");
    let err_msg = res.unwrap_err().to_string();
    assert!(
        err_msg.contains("INV-DELETION-1 violation"),
        "Error message should mention INV-DELETION-1: {}",
        err_msg
    );
    assert!(!err_msg.contains("key_a"), "Error message must not leak key bytes");
}

#[tokio::test]
async fn test_b_put_then_delete_put_version_still_in_memtable_err() {
    let (storage, _tmp) = create_test_storage().await;
    let tx1 = TxId::new(1);
    let key = b"key_b".to_vec();

    storage.put(tx1, &key, b"val_b").await.unwrap();
    storage.commit(tx1).await.unwrap();

    let tx2 = TxId::new(2);
    storage.delete(tx2, &key).await.unwrap();
    storage.commit(tx2).await.unwrap();

    let res = storage
        .issue_memtable_witness(&[key.clone()], &[], tx2)
        .await;
    assert!(
        res.is_err(),
        "Expected Err when older Put version is shadowed by tombstone but still in memtable"
    );
    let err_msg = res.unwrap_err().to_string();
    assert!(err_msg.contains("INV-DELETION-1 violation"));
}

#[tokio::test]
async fn test_c_after_force_flush_ok() {
    let (storage, _tmp) = create_test_storage().await;
    let tx1 = TxId::new(1);
    let key = b"key_c".to_vec();

    storage.put(tx1, &key, b"val_c").await.unwrap();
    storage.commit(tx1).await.unwrap();

    storage.force_flush().await.unwrap();

    let witness = storage
        .issue_memtable_witness(&[key.clone()], &[], tx1)
        .await
        .expect("Witness should succeed after force_flush");

    assert_eq!(witness.layer(), &DeletionLayer::LsmMemtable);
    if let WitnessEvidence::MemtableScan { memtables_scanned } = witness.evidence() {
        assert!(
            *memtables_scanned >= 1,
            "Expected memtables_scanned >= 1, got {}",
            memtables_scanned
        );
    } else {
        panic!("Unexpected evidence type: {:?}", witness.evidence());
    }
}

#[tokio::test]
async fn test_d_put_in_immutable_memtable_err() {
    let tmp = TempDir::new().expect("temp dir");
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        memtable_size_limit: 1024,
        max_ram_mb: 64,
        tx_timeout: Duration::from_secs(60),
        compaction: CompactionConfig::default(),
        encryption_passphrase: None,
        ..Default::default()
    };
    let storage = LsmStorage::new(config).await.expect("create storage");

    let tx1 = TxId::new(1);
    let key = b"key_d".to_vec();
    storage.put(tx1, &key, b"val_d").await.unwrap();
    storage.commit(tx1).await.unwrap();

    // Read-lock storage state or inject invalid path to force Phase 3 flush failure
    let count = storage.segment_counter_for_test();
    let seq = storage.next_seq_no_for_test();
    let sst_path = tmp
        .path()
        .join(format!("sst-{:020}-{:06}.sst", seq, count % 1_000_000));
    tokio::fs::create_dir_all(&sst_path).await.unwrap();

    let flush_res = storage.force_flush().await;
    assert!(flush_res.is_err(), "Flush should fail to simulate immutable memtable retention");

    let res = storage
        .issue_memtable_witness(&[key.clone()], &[], tx1)
        .await;
    assert!(
        res.is_err(),
        "Expected Err when Put version is in immutable memtable"
    );
    let err_msg = res.unwrap_err().to_string();
    assert!(
        err_msg.contains("INV-DELETION-1 violation"),
        "Error message: {}",
        err_msg
    );
    assert!(
        err_msg.contains("memtable_index: 1") || err_msg.contains("immutable"),
        "Error should identify immutable memtable: {}",
        err_msg
    );
}

#[tokio::test]
async fn test_e_prefix_case_new_key_under_prefix_err() {
    let (storage, _tmp) = create_test_storage().await;
    let prefix = b"prefix_e:".to_vec();

    let tx1 = TxId::new(1);
    let key1 = b"prefix_e:1".to_vec();
    storage.put(tx1, &key1, b"v1").await.unwrap();
    storage.commit(tx1).await.unwrap();

    let tx2 = TxId::new(2);
    storage.delete(tx2, &key1).await.unwrap();
    storage.commit(tx2).await.unwrap();

    let tx3 = TxId::new(3);
    let key2 = b"prefix_e:2".to_vec();
    storage.put(tx3, &key2, b"v2").await.unwrap();
    storage.commit(tx3).await.unwrap();

    let res = storage
        .issue_memtable_witness(&[key1.clone()], &[prefix.clone()], tx3)
        .await;
    assert!(
        res.is_err(),
        "Expected Err when a new key was written under prefix"
    );
}

#[tokio::test]
async fn test_f_foreign_tenant_different_prefix_untouched_ok() {
    let (storage, _tmp) = create_test_storage().await;

    let storage_arc = Arc::new(storage);
    let tenant_a = TenantId::try_new(10).unwrap();
    let tenant_b = TenantId::try_new(20).unwrap();

    let store_a = TenantScopedStorage::new(storage_arc.clone(), tenant_a);
    let store_b = TenantScopedStorage::new(storage_arc.clone(), tenant_b);

    let tx1 = TxId::new(1);
    let key_b = b"doc1".to_vec();
    store_b.put(tx1, &key_b, b"data_b").await.unwrap();
    store_b.commit(tx1).await.unwrap();

    let witness_a = store_a
        .issue_memtable_witness(&[b"doc1".to_vec()], &[b"doc".to_vec()], tx1)
        .await;
    assert!(
        witness_a.is_ok(),
        "Tenant A witness must be Ok despite Tenant B having active memtable data"
    );
}

#[tokio::test]
async fn test_g_property_like_random_sequence() {
    use rand::Rng;
    let (storage, _tmp) = create_test_storage().await;

    let mut rng = StdRng::seed_from_u64(0x42_DE_AD_BE_EF);
    let target_key = b"target_prop_key".to_vec();
    let target_prefix = b"target_prop_".to_vec();

    let mut in_memtable = false;
    let mut tx_counter = 1u64;

    for _step in 0..50 {
        let op: u8 = rng.gen_range(0..3);
        let tx = TxId::new(tx_counter);
        tx_counter += 1;

        match op {
            0 => {
                storage.put(tx, &target_key, b"v").await.unwrap();
                storage.commit(tx).await.unwrap();
                in_memtable = true;
            }
            1 => {
                storage.delete(tx, &target_key).await.unwrap();
                storage.commit(tx).await.unwrap();
                // delete still leaves a non-tombstone Put version in memtable if Put was present!
            }
            2 => {
                storage.force_flush().await.unwrap();
                in_memtable = false;
            }
            _ => unreachable!(),
        }

        let witness_res = storage
            .issue_memtable_witness(&[target_key.clone()], &[target_prefix.clone()], tx)
            .await;

        if in_memtable {
            assert!(
                witness_res.is_err(),
                "Step: expected witness Err because target key is in memtable"
            );
        } else {
            assert!(
                witness_res.is_ok(),
                "Step: expected witness Ok because memtable was flushed"
            );
        }
    }
}
