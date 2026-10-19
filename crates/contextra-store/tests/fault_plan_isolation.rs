#![cfg(feature = "fault-injection")]

use contextra_ports::StorageEngine;
use contextra_store::lsm::{LsmConfig, LsmStorage};
use contextra_store::wal::{Wal, WalOp};
use contextra_types::TxId;
use std::sync::atomic::Ordering;
use tempfile::tempdir;

#[tokio::test]
async fn test_two_wal_instances_different_fault_plans_do_not_interfere() -> contextra_types::Result<()> {
    let dir1 = tempdir()?;
    let dir2 = tempdir()?;

    let wal1_path = dir1.path().join("wal1.log");
    let wal2_path = dir2.path().join("wal2.log");

    let wal1 = Wal::open(&wal1_path).await?;
    let wal2 = Wal::open(&wal2_path).await?;

    // Arm fault injection ONLY on wal1
    wal1.fault_plan().fail_append_for_tx.store(u64::MAX, Ordering::SeqCst);

    let op1 = WalOp::Put {
        tx_id: TxId::new(1),
        key: b"k1".to_vec(),
        value: b"v1".to_vec(),
    };
    let (batch1, _) = wal1.prepare_batch(vec![(op1.clone(), 1)]).await?;
    let (batch2, _) = wal2.prepare_batch(vec![(op1.clone(), 1)]).await?;

    // append_batch on wal1 should fail
    let res1 = wal1.append_batch(batch1).await;
    assert!(res1.is_err(), "wal1 must fail when fault injection is set on wal1");

    // append_batch on wal2 must succeed (unaffected by wal1's fault plan)
    let res2 = wal2.append_batch(batch2).await;
    assert!(res2.is_ok(), "wal2 must succeed as its fault plan is clean");

    Ok(())
}

#[tokio::test]
async fn test_two_lsm_storages_different_fault_plans_do_not_interfere() -> contextra_types::Result<()> {
    let dir1 = tempdir()?;
    let dir2 = tempdir()?;

    let config1 = LsmConfig {
        path: dir1.path().to_path_buf(),
        ..Default::default()
    };
    let config2 = LsmConfig {
        path: dir2.path().to_path_buf(),
        ..Default::default()
    };

    let storage1 = LsmStorage::open(config1).await?;
    let storage2 = LsmStorage::open(config2).await?;

    // Arm fault injection ONLY on storage1
    storage1.simulate_wal_append_failure_for_test().await;

    let tx1 = TxId::new(10);
    storage1.put(tx1, b"key1", b"val1").await?;
    storage2.put(tx1, b"key1", b"val1").await?;

    let commit1 = storage1.commit(tx1).await;
    assert!(commit1.is_err(), "storage1 commit must fail due to fault plan on storage1");

    let commit2 = storage2.commit(tx1).await;
    assert!(commit2.is_ok(), "storage2 commit must succeed independently of storage1");

    Ok(())
}
