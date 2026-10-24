use super::*;
use crate::wal::{PreparedBatch, WalConfig, WalEntry, WalOp};
use contextra_core::TxId;
use tempfile::tempdir;

#[tokio::test]
async fn test_wal_flusher_actor_coalescing() -> Result<()> {
    let dir = tempdir()?;
    let wal_path = dir.path().join("test_flusher.wal");

    let wal = Arc::new(Wal::open(&wal_path).await?);

    let num_tasks = 10;
    let mut handles = Vec::new();

    for i in 0..num_tasks {
        let wal_clone = Arc::clone(&wal);
        handles.push(tokio::spawn(async move {
            let op = WalOp::Put {
                tx_id: TxId::new(i + 1),
                key: format!("flusher_k_{i}").into_bytes(),
                value: format!("flusher_v_{i}").into_bytes(),
            };
            let (batch, _) = wal_clone.prepare_batch(vec![(op, i + 1)]).await?;
            wal_clone.append_batch(batch).await
        }));
    }

    for h in handles {
        h.await
            .map_err(|e| ContextraError::Storage(e.to_string()))??;
    }

    let replayed = wal.replay().await?;
    assert_eq!(replayed.len(), num_tasks as usize);

    for (i, (_seq, entry, _pos)) in replayed.iter().enumerate() {
        assert_eq!(entry.seq_no, (i + 1) as u64);
    }

    Ok(())
}

#[tokio::test]
async fn test_flusher_batch_window_coalesces_writes() -> Result<()> {
    let dir = tempdir()?;
    let wal_path = dir.path().join("batch_window_test.wal");

    let wal = Arc::new(
        Wal::open_with_config(
            &wal_path,
            WalConfig {
                flusher_config: WalFlusherConfig {
                    batch_window_micros: 50,
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .await?,
    );

    let num_tasks = 5;
    let mut handles = Vec::new();

    for i in 0u64..num_tasks {
        let wal_clone = Arc::clone(&wal);
        handles.push(tokio::spawn(async move {
            let op = WalOp::Put {
                tx_id: TxId::new(i + 1),
                key: format!("key-{i}").into_bytes(),
                value: b"val".to_vec(),
            };
            let (batch, _) = wal_clone.prepare_batch(vec![(op, i + 1)]).await?;
            wal_clone.append_batch(batch).await
        }));
    }

    for h in handles {
        h.await
            .map_err(|e| ContextraError::Storage(e.to_string()))??;
    }

    let replayed = wal.replay().await?;
    assert_eq!(
        replayed.len(),
        num_tasks as usize,
        "Alle 5 Batches müssen sicher im WAL landen"
    );

    for (i, (_seq, entry, _pos)) in replayed.iter().enumerate() {
        assert_eq!(entry.seq_no, (i + 1) as u64);
    }

    Ok(())
}

#[tokio::test]
async fn test_wal_flusher_actor_no_write_to_sealed() {
    let dir = tempfile::tempdir().expect("tempdir");
    let wal_path = dir.path().join("test_flusher_sealed.wal");

    let wal = Wal::open(&wal_path).await.expect("open WAL");
    let op1 = WalOp::Put {
        tx_id: TxId::new(1),
        key: b"k1".to_vec(),
        value: b"v1".to_vec(),
    };
    let (batch1, _) = wal
        .prepare_batch(vec![(op1, 1)])
        .await
        .expect("prepare batch 1");
    wal.append_batch(batch1).await.expect("append batch 1");

    assert!(
        !wal.is_sealed(),
        "WAL must not be sealed before rotate_and_seal"
    );

    let sealed_path = wal.rotate_and_seal().await.expect("rotate_and_seal");
    assert!(wal.is_sealed(), "WAL must be sealed after rotate_and_seal");

    // Attempting to prepare or append to the sealed WAL segment must return Err
    let op2 = WalOp::Put {
        tx_id: TxId::new(2),
        key: b"k2".to_vec(),
        value: b"v2".to_vec(),
    };
    let prep_res = wal.prepare_batch(vec![(op2, 2)]).await;
    assert!(
        prep_res.is_err(),
        "prepare_batch on sealed WAL must return Err, no panic or silent write"
    );

    let dummy_entry = WalEntry::try_new(
        WalOp::Put {
            tx_id: TxId::new(3),
            key: b"k3".to_vec(),
            value: b"v3".to_vec(),
        },
        3,
        &[0u8; 32],
        [0u8; 32],
    )
    .expect("dummy entry");
    let manual_batch = PreparedBatch(vec![dummy_entry]);
    let append_res = wal.append_batch(manual_batch).await;
    assert!(
        append_res.is_err(),
        "append_batch on sealed WAL must return Err, no panic or silent write"
    );

    assert!(sealed_path.exists(), "Sealed path must exist");
}

#[tokio::test]
async fn test_wal_queue_capacity_zero_rejected() {
    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("invalid_cap.wal");

    let res = Wal::open_with_config(
        &wal_path,
        WalConfig {
            flusher_config: WalFlusherConfig {
                queue_capacity: 0,
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .await;

    assert!(res.is_err(), "queue_capacity == 0 must return an error");
    if let Err(err) = res {
        assert!(
            err.to_string().contains("queue_capacity"),
            "Error message must mention queue_capacity, got: {err}"
        );
    }
}

#[tokio::test]
async fn test_wal_try_append_backpressure() -> Result<()> {
    let dir = tempdir()?;
    let wal_path = dir.path().join("try_append_backpressure.wal");

    // Configure capacity of 1 with batch_window_micros = 100_000 (100ms window)
    let wal = Arc::new(
        Wal::open_with_config(
            &wal_path,
            WalConfig {
                flusher_config: WalFlusherConfig {
                    queue_capacity: 1,
                    batch_window_micros: 100_000,
                },
                ..Default::default()
            },
        )
        .await?,
    );

    // Get flusher_tx directly to fill channel capacity without holding truncate_lock
    let flusher_tx = {
        let guard = wal.flusher_tx.read().unwrap();
        guard.clone().unwrap()
    };

    let (ack1_tx, _ack1_rx) = tokio::sync::oneshot::channel();
    let (ack2_tx, _ack2_rx) = tokio::sync::oneshot::channel();

    // 1st command is popped by flusher loop and enters batch window wait
    flusher_tx
        .send(WalCommand::Append {
            payload: vec![1, 2, 3],
            last_hmac_val: [0u8; 32],
            ack: ack1_tx,
        })
        .await
        .unwrap();

    // 2nd command fills the bounded channel (capacity 1)
    flusher_tx
        .send(WalCommand::Append {
            payload: vec![4, 5, 6],
            last_hmac_val: [0u8; 32],
            ack: ack2_tx,
        })
        .await
        .unwrap();

    // Now try_append_batch when channel queue is full
    let op = WalOp::Put {
        tx_id: TxId::new(10),
        key: b"overflow_key".to_vec(),
        value: b"overflow_val".to_vec(),
    };
    let (overflow_batch, _) = wal.prepare_batch(vec![(op, 10)]).await?;

    let try_res = wal.try_append_batch(overflow_batch).await;
    assert!(
        try_res.is_err(),
        "try_append_batch must return error when queue is full"
    );
    if let Err(err) = try_res {
        assert!(
            err.to_string().contains("backpressure"),
            "Error must mention backpressure, got: {err}"
        );
    }

    Ok(())
}

#[tokio::test]
async fn test_wal_rotate_and_seal_collision_avoidance() -> Result<()> {
    let dir = tempdir()?;
    let wal_path = dir.path().join("test_seal.wal");

    let wal = Wal::open(&wal_path).await?;
    let op1 = WalOp::Put {
        tx_id: TxId::new(1),
        key: b"k1".to_vec(),
        value: b"v1".to_vec(),
    };
    let (batch1, _) = wal.prepare_batch(vec![(op1, 1)]).await?;
    wal.append_batch(batch1).await?;

    // Manually create pre-existing candidate path test_seal.wal.sealed.1
    let existing_sealed_1 = dir.path().join("test_seal.wal.sealed.1");
    tokio::fs::write(&existing_sealed_1, b"PRE_EXISTING_SEALED_CONTENT").await?;

    // Calling rotate_and_seal must skip .sealed.1 and write to .sealed.2
    let sealed_path = wal.rotate_and_seal().await?;
    assert_ne!(
        sealed_path, existing_sealed_1,
        "rotate_and_seal must NOT overwrite existing .sealed.1"
    );
    assert!(
        sealed_path.exists(),
        "New sealed path {:?} must exist",
        sealed_path
    );

    // Content of existing_sealed_1 must be unchanged
    let content = tokio::fs::read(&existing_sealed_1).await?;
    assert_eq!(
        content, b"PRE_EXISTING_SEALED_CONTENT",
        "Pre-existing sealed file must not be modified or overwritten"
    );

    Ok(())
}

#[cfg(feature = "fault-injection")]
#[tokio::test]
async fn test_wal_fsync_failure_poisons_handle() -> Result<()> {
    let dir = tempdir()?;
    let wal_path = dir.path().join("fsync_fail.wal");

    let wal = Wal::open(&wal_path).await?;
    let op1 = WalOp::Put {
        tx_id: TxId::new(1),
        key: b"k1".to_vec(),
        value: b"v1".to_vec(),
    };
    let (batch1, _) = wal.prepare_batch(vec![(op1, 1)]).await?;
    wal.append_batch(batch1).await?;

    // Set FAIL_SYNC_ONCE
    crate::wal::flusher::FAIL_SYNC_ONCE.store(true, std::sync::atomic::Ordering::SeqCst);

    let op2 = WalOp::Put {
        tx_id: TxId::new(2),
        key: b"k2".to_vec(),
        value: b"v2".to_vec(),
    };
    let (batch2, _) = wal.prepare_batch(vec![(op2, 2)]).await?;
    let res = wal.append_batch(batch2).await;

    assert!(res.is_err(), "Append must fail when fsync fails");
    let err_msg = res.unwrap_err().to_string();
    assert!(
        err_msg.contains("WAL fsync failed, outcome unknown"),
        "Error message must contain 'WAL fsync failed, outcome unknown', got: {err_msg}"
    );

    assert!(
        wal.is_poisoned(),
        "WAL handle must be permanently poisoned after fsync failure"
    );

    // Subsequent appends must fail because handle is poisoned
    let op3 = WalOp::Put {
        tx_id: TxId::new(3),
        key: b"k3".to_vec(),
        value: b"v3".to_vec(),
    };
    let (batch3, _) = wal.prepare_batch(vec![(op3, 3)]).await?;
    let res3 = wal.append_batch(batch3).await;
    assert!(res3.is_err(), "Subsequent append must fail on poisoned WAL");

    Ok(())
}

#[cfg(feature = "encryption-at-rest")]
#[tokio::test]
async fn test_wal_rewrite_multi_frame_chunking_encrypted() -> Result<()> {
    let dir = tempdir()?;
    let wal_path = dir.path().join("encrypted_rewrite.wal");

    let km = Arc::new(KeyManager::try_new(
        "passphrase123",
        b"salt123456789012345678901234567890",
    )?);
    let wal = Wal::open_with_key_manager(&wal_path, Some(km.clone())).await?;

    // 2 entries of 35 MiB each -> total 70 MiB > MAX_WAL_ENTRY_SIZE (64 MiB)
    let payload_35mb = vec![0xAB; 35 * 1024 * 1024];
    for i in 1..=2 {
        let op = WalOp::Put {
            tx_id: TxId::new(i),
            key: format!("k{i}").into_bytes(),
            value: payload_35mb.clone(),
        };
        let (batch, _) = wal.prepare_batch(vec![(op, i)]).await?;
        wal.append_batch(batch).await?;
    }

    let replayed = wal.replay().await?;
    assert_eq!(replayed.len(), 2);

    let integrity_key = wal.get_integrity_key()?;

    // Trigger Rewrite command
    let (ack_tx, ack_rx) = tokio::sync::oneshot::channel();
    let flusher_tx = wal.flusher_tx.read().unwrap().clone().unwrap();
    flusher_tx
        .send(WalCommand::Rewrite {
            replayed_entries: replayed,
            integrity_key,
            ack: ack_tx,
        })
        .await
        .unwrap();

    ack_rx.await.unwrap()?;

    drop(wal);

    // Reopen and replay: must successfully parse both 35 MiB entries split across multiple frames
    let wal_reopened = Wal::open_with_key_manager(&wal_path, Some(km)).await?;
    let replayed_after = wal_reopened.replay().await?;
    assert_eq!(replayed_after.len(), 2);
    for (i, (_seq, entry, _pos)) in replayed_after.iter().enumerate() {
        assert_eq!(entry.seq_no, (i + 1) as u64);
        if let WalOp::Put { value, .. } = &entry.op {
            assert_eq!(value.len(), 35 * 1024 * 1024);
        }
    }

    Ok(())
}
