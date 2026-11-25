#![cfg(feature = "fault-injection")]

use bytes::Bytes;
use contextra_core::{StorageEngine, TxId};
use contextra_store::wal::{DELAY_APPEND_FOR_TX, DELAY_APPEND_MS, FAIL_APPEND_FOR_TX};
use contextra_store::{LsmConfig, LsmStorage};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;
use tempfile::tempdir;

fn copy_dir_all(
    src: impl AsRef<std::path::Path>,
    dst: impl AsRef<std::path::Path>,
) -> std::io::Result<()> {
    std::fs::create_dir_all(&dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

#[tokio::test]
async fn test_k2_flush_swap_race_crash_recovery() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().to_path_buf();

    let mut config = LsmConfig::default();
    config.path = db_path.clone();
    config.group_commit_window_micros = 5000;
    let storage = LsmStorage::open(config.clone()).await.unwrap();

    let tx_id = TxId::new(100);
    storage.put(tx_id, b"key_k2", b"val_k2").await.unwrap();

    DELAY_APPEND_FOR_TX.store(tx_id.inner(), Ordering::SeqCst);
    DELAY_APPEND_MS.store(200, Ordering::SeqCst);

    let storage_clone = Arc::new(storage);
    let s1 = storage_clone.clone();

    let commit_task = tokio::spawn(async move { s1.commit(tx_id).await });

    tokio::time::sleep(Duration::from_millis(50)).await;

    // Trigger parallel flush while commit is delayed at WAL append
    let s2 = storage_clone.clone();
    let flush_task = tokio::spawn(async move { s2.force_flush().await });

    let (commit_res, flush_res) = tokio::join!(commit_task, flush_task);
    commit_res.unwrap().unwrap();
    let _ = flush_res;

    DELAY_APPEND_FOR_TX.store(0, Ordering::SeqCst);
    DELAY_APPEND_MS.store(0, Ordering::SeqCst);

    // Simulate crash without clean shutdown: copy DB directory
    let crash_dir = tempdir().unwrap();
    let crash_path = crash_dir.path().to_path_buf();

    copy_dir_all(&db_path, &crash_path).unwrap();

    // Reopen from crash directory
    let mut reopened_config = config.clone();
    reopened_config.path = crash_path;
    let reopened = LsmStorage::open(reopened_config).await.unwrap();
    let val = reopened.get(b"key_k2").await.unwrap();
    assert_eq!(val, Some(Bytes::from_static(b"val_k2")));
}

#[tokio::test]
async fn test_k2_k3_64_parallel_writers_with_repeated_flushes() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().to_path_buf();

    let mut config = LsmConfig::default();
    config.path = db_path.clone();
    config.group_commit_window_micros = 1000;
    let storage = Arc::new(LsmStorage::open(config.clone()).await.unwrap());

    let mut handles = Vec::new();
    for i in 0..64 {
        let st = storage.clone();
        handles.push(tokio::spawn(async move {
            let tx = TxId::new(1000 + i);
            let k = format!("k_{i}").into_bytes();
            let v = format!("v_{i}").into_bytes();
            st.put(tx, &k, &v).await.unwrap();
            st.commit(tx).await.unwrap();
            (k, v)
        }));
    }

    let st_flush = storage.clone();
    let flush_handle = tokio::spawn(async move {
        for _ in 0..10 {
            tokio::time::sleep(Duration::from_millis(5)).await;
            let _ = st_flush.force_flush().await;
        }
    });

    let mut expected_kvs = Vec::new();
    for h in handles {
        expected_kvs.push(h.await.unwrap());
    }
    let _ = flush_handle.await;

    drop(storage);

    let reopened = LsmStorage::open(config).await.unwrap();
    for (k, v) in expected_kvs {
        let res = reopened.get(&k).await.unwrap();
        assert_eq!(res, Some(Bytes::from(v)));
    }
}

#[tokio::test]
async fn test_h5_follower_timeout_after_leader_acquisition() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().to_path_buf();

    let mut config = LsmConfig::default();
    config.path = db_path;
    config.group_commit_window_micros = 1000;
    config.tx_timeout = Duration::from_millis(50);

    let storage = Arc::new(LsmStorage::open(config).await.unwrap());

    let tx_leader = TxId::new(500);
    let tx_follower = TxId::new(501);

    storage.put(tx_leader, b"k_lead", b"v_lead").await.unwrap();
    storage
        .put(tx_follower, b"k_foll", b"v_foll")
        .await
        .unwrap();

    // Delay WAL append for leader so follower's timeout expires while taken by leader
    DELAY_APPEND_FOR_TX.store(tx_leader.inner(), Ordering::SeqCst);
    DELAY_APPEND_MS.store(150, Ordering::SeqCst);

    let s1 = storage.clone();
    let leader_task = tokio::spawn(async move { s1.commit(tx_leader).await });

    tokio::time::sleep(Duration::from_millis(5)).await;

    let s2 = storage.clone();
    let follower_task = tokio::spawn(async move { s2.commit(tx_follower).await });

    let (r1, r2) = tokio::join!(leader_task, follower_task);
    DELAY_APPEND_FOR_TX.store(0, Ordering::SeqCst);
    DELAY_APPEND_MS.store(0, Ordering::SeqCst);

    let res_leader = r1.unwrap();
    let res_follower = r2.unwrap();

    assert!(
        res_leader.is_ok(),
        "Leader commit should succeed: {:?}",
        res_leader
    );
    assert!(
        res_follower.is_ok(),
        "Follower should not get CommitTimeout after acquisition: {:?}",
        res_follower
    );
}

#[tokio::test]
async fn test_batch_append_error_recovery() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().to_path_buf();

    let mut config = LsmConfig::default();
    config.path = db_path;
    config.group_commit_window_micros = 2000;
    let storage = Arc::new(LsmStorage::open(config.clone()).await.unwrap());

    let tx_fail = TxId::new(900);
    storage.put(tx_fail, b"k_fail", b"v_fail").await.unwrap();

    FAIL_APPEND_FOR_TX.store(tx_fail.inner(), Ordering::SeqCst);

    let commit_res = storage.commit(tx_fail).await;
    FAIL_APPEND_FOR_TX.store(0, Ordering::SeqCst);

    assert!(
        commit_res.is_err(),
        "Commit should fail on injected append error"
    );

    // Next commit should succeed
    let tx_ok = TxId::new(901);
    storage.put(tx_ok, b"k_ok", b"v_ok").await.unwrap();
    let ok_res = storage.commit(tx_ok).await;
    assert!(ok_res.is_ok(), "Subsequent commit should succeed");

    drop(storage);

    // Reopen & verify consistency
    let reopened = LsmStorage::open(config).await.unwrap();
    assert_eq!(reopened.get(b"k_fail").await.unwrap(), None);
    assert_eq!(
        reopened.get(b"k_ok").await.unwrap(),
        Some(Bytes::from_static(b"v_ok"))
    );
}
