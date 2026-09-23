// Testpflicht AK-13 (docs/specs/CONTEXTRA_SPEC_v2.md)

use contextra_core::TxId;
use contextra_store::wal::{Wal, WalConfig, WalFlusherConfig, WalOp};
use std::sync::mpsc;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::fs;
use tokio::time::{timeout, Duration};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_wal_try_append_backpressure_when_queue_full() {
    let tmp = TempDir::new().expect("temp dir");
    let wal_path = tmp.path().join("wal_backpressure.log");

    let config = WalConfig {
        flusher_config: WalFlusherConfig {
            batch_window_micros: 0,
            queue_capacity: 1, // Bounded channel capacity = 1
        },
        ..Default::default()
    };

    let wal = Arc::new(
        Wal::open_with_config(&wal_path, config)
            .await
            .expect("open wal"),
    );

    // 1. Write an initial confirmed batch so WAL file has a valid header + entry
    let (batch_init, _) = wal
        .prepare_batch(vec![(
            WalOp::Put {
                tx_id: TxId::new(100),
                key: b"init_key".to_vec(),
                value: b"init_val".to_vec(),
            },
            1,
        )])
        .await
        .expect("prepare batch init");

    wal.append_batch(batch_init).await.expect("init append");

    let file_size = fs::metadata(&wal_path).await.expect("metadata").len();
    assert!(
        file_size > 0,
        "WAL file must contain data for scan callback"
    );

    let (batch1, _) = wal
        .prepare_batch(vec![(
            WalOp::Put {
                tx_id: TxId::new(1),
                key: b"k1".to_vec(),
                value: b"v1".to_vec(),
            },
            2,
        )])
        .await
        .expect("prepare batch 1");

    let (batch2, _) = wal
        .prepare_batch(vec![(
            WalOp::Put {
                tx_id: TxId::new(2),
                key: b"k2".to_vec(),
                value: b"v2".to_vec(),
            },
            3,
        )])
        .await
        .expect("prepare batch 2");

    let (batch3, _) = wal
        .prepare_batch(vec![(
            WalOp::Put {
                tx_id: TxId::new(3),
                key: b"k3".to_vec(),
                value: b"v3".to_vec(),
            },
            4,
        )])
        .await
        .expect("prepare batch 3");

    let (in_scan_tx, in_scan_rx) = mpsc::channel();
    let (resume_scan_tx, resume_scan_rx) = mpsc::channel();
    let resume_scan_rx = Arc::new(std::sync::Mutex::new(resume_scan_rx));

    let wal_scan = Arc::clone(&wal);

    let scan_handle = std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async move {
            wal_scan
                .scan_entries_with_callback(file_size, move |_seq, _entry, _pos| {
                    let _ = in_scan_tx.send(());
                    if let Ok(rx) = resume_scan_rx.lock() {
                        let _ = rx.recv();
                    }
                    true
                })
                .await
        })
    });

    in_scan_rx
        .recv_timeout(Duration::from_secs(5))
        .expect("flusher enters scan callback");

    // Flusher is currently busy in Scan callback.
    // 2. Task 1 calls enqueue_append_batch_locked(batch1): holds truncate_lock, sends batch1 to mpsc queue (occupies flusher buffer)
    let wal_c1 = Arc::clone(&wal);
    let (tx_in_enqueue1, rx_in_enqueue1) = mpsc::channel();
    let (tx_resume_enqueue1, rx_resume_enqueue1) = mpsc::channel();
    let rx_resume_enqueue1 = Arc::new(std::sync::Mutex::new(rx_resume_enqueue1));

    let task_enqueue1 = tokio::spawn(async move {
        let guard = wal_c1.truncate_lock.lock().await;
        let _ = tx_in_enqueue1.send(());
        if let Ok(rx) = rx_resume_enqueue1.lock() {
            let _ = rx.recv();
        }
        wal_c1.enqueue_append_batch_locked(batch1, &guard).await
    });

    let _ = rx_in_enqueue1
        .recv_timeout(Duration::from_secs(5))
        .expect("task_enqueue1 locked");

    // 3. Task 2 calls enqueue_append_batch_locked(batch2): waits for truncate_lock
    let wal_c2 = Arc::clone(&wal);
    let task_enqueue2 = tokio::spawn(async move {
        let guard = wal_c2.truncate_lock.lock().await;
        wal_c2.enqueue_append_batch_locked(batch2, &guard).await
    });

    tokio::time::sleep(Duration::from_millis(20)).await;

    // Release task 1 so it sends batch1 into mpsc queue (fills 1/1 capacity) and drops truncate_lock
    let _ = tx_resume_enqueue1.send(());

    let enqueue1_res = task_enqueue1.await.expect("task_enqueue1 join");
    assert!(enqueue1_res.is_ok(), "enqueue batch1 must succeed");

    // Task 2 now acquires truncate_lock and sends batch2 into mpsc queue (now full)
    let enqueue2_res = task_enqueue2.await.expect("task_enqueue2 join");
    assert!(enqueue2_res.is_ok(), "enqueue batch2 must succeed");

    // 4. Call try_append_batch(batch3) on full queue: must fail immediately with Backpressure
    let res = wal.try_append_batch(batch3).await;
    assert!(
        res.is_err(),
        "try_append_batch on full WAL queue must return Err(Backpressure)"
    );

    let err_msg = res.unwrap_err().to_string();
    assert!(
        err_msg.contains("WAL queue full (backpressure)"),
        "Error message must indicate backpressure, got: {}",
        err_msg
    );

    let _ = resume_scan_tx.send(());
    let _ = scan_handle.join().unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_wal_append_blocks_until_flusher_frees_capacity() {
    let tmp = TempDir::new().expect("temp dir");
    let wal_path = tmp.path().join("wal_blocking.log");

    let config = WalConfig {
        flusher_config: WalFlusherConfig {
            batch_window_micros: 200_000,
            queue_capacity: 1,
        },
        ..Default::default()
    };

    let wal = Arc::new(
        Wal::open_with_config(&wal_path, config)
            .await
            .expect("open wal"),
    );

    let (batch1, _) = wal
        .prepare_batch(vec![(
            WalOp::Put {
                tx_id: TxId::new(1),
                key: b"k1".to_vec(),
                value: b"v1".to_vec(),
            },
            1,
        )])
        .await
        .expect("prepare batch 1");

    let (batch2, _) = wal
        .prepare_batch(vec![(
            WalOp::Put {
                tx_id: TxId::new(2),
                key: b"k2".to_vec(),
                value: b"v2".to_vec(),
            },
            2,
        )])
        .await
        .expect("prepare batch 2");

    let (batch3, _) = wal
        .prepare_batch(vec![(
            WalOp::Put {
                tx_id: TxId::new(3),
                key: b"k3".to_vec(),
                value: b"v3".to_vec(),
            },
            3,
        )])
        .await
        .expect("prepare batch 3");

    let (batch4, _) = wal
        .prepare_batch(vec![(
            WalOp::Put {
                tx_id: TxId::new(4),
                key: b"k4".to_vec(),
                value: b"v4".to_vec(),
            },
            4,
        )])
        .await
        .expect("prepare batch 4");

    // Task 1: Flusher receives batch1 and enters batch window sleep
    let wal_c1 = Arc::clone(&wal);
    let t1 = tokio::spawn(async move { wal_c1.append_batch(batch1).await });
    tokio::time::sleep(Duration::from_millis(5)).await;

    // Task 2: Flusher pops batch2 into batch_payload during batch window
    let wal_c2 = Arc::clone(&wal);
    let t2 = tokio::spawn(async move { wal_c2.append_batch(batch2).await });
    tokio::time::sleep(Duration::from_millis(5)).await;

    // Task 3: Fill channel buffer with batch3
    let wal_c3 = Arc::clone(&wal);
    let t3 = tokio::spawn(async move { wal_c3.append_batch(batch3).await });
    tokio::time::sleep(Duration::from_millis(5)).await;

    // Task 4: Call append_batch on full queue -> must block waiting for flusher channel capacity
    let wal_c4 = Arc::clone(&wal);
    let mut t4 = tokio::spawn(async move { wal_c4.append_batch(batch4).await });

    // Verify task 4 is blocked while queue is full
    let is_still_pending = timeout(Duration::from_millis(30), &mut t4).await.is_err();
    assert!(
        is_still_pending,
        "append_batch on full queue must block until flusher drains capacity"
    );

    // Await all tasks to complete successfully after flusher processes batches
    let r1 = t1.await.expect("task 1 join");
    let r2 = t2.await.expect("task 2 join");
    let r3 = t3.await.expect("task 3 join");
    let r4 = t4.await.expect("task 4 join");

    assert!(r1.is_ok(), "batch 1 append succeeded");
    assert!(r2.is_ok(), "batch 2 append succeeded");
    assert!(r3.is_ok(), "batch 3 append succeeded");
    assert!(
        r4.is_ok(),
        "batch 4 append succeeded after queue capacity freed"
    );
}

#[tokio::test]
async fn test_wal_confirmed_and_unconfirmed_entries_after_crash_recovery() {
    let tmp = TempDir::new().expect("temp dir");
    let wal_path = tmp.path().join("wal_crash_recovery.log");

    // 1. Confirmed append
    {
        let wal = Wal::open(&wal_path).await.expect("open wal");

        let (confirmed_batch, _) = wal
            .prepare_batch(vec![(
                WalOp::Put {
                    tx_id: TxId::new(100),
                    key: b"confirmed_key".to_vec(),
                    value: b"confirmed_val".to_vec(),
                },
                1,
            )])
            .await
            .expect("prepare confirmed batch");

        // append_batch awaits ack confirmation from flusher
        wal.append_batch(confirmed_batch)
            .await
            .expect("confirmed append must succeed");
    }

    // 2. Simulate crash with unconfirmed trailing write on disk
    let mut file_bytes = fs::read(&wal_path).await.expect("read wal file");
    assert!(
        !file_bytes.is_empty(),
        "WAL file should contain header + confirmed entry"
    );

    // Append corrupted/incomplete trailing bytes simulating an unconfirmed in-flight crash
    file_bytes.extend_from_slice(&[0xFF, 0x00, 0xCA, 0xFE, 0xBA, 0xBE]);
    fs::write(&wal_path, &file_bytes)
        .await
        .expect("write wal with trailing garbage");

    // 3. Re-open WAL and recover
    let wal_recovered = Wal::open(&wal_path).await.expect("reopen wal after crash");

    let mut recovered_entries = Vec::new();
    let file_size = fs::metadata(&wal_path).await.expect("metadata").len();

    wal_recovered
        .scan_entries_with_callback(file_size, |_seq, entry, _pos| {
            recovered_entries.push(entry);
            true
        })
        .await
        .expect("scan entries after crash recovery");

    // 4. Verify confirmed entry is present and unconfirmed trailing noise was safely ignored
    assert_eq!(
        recovered_entries.len(),
        1,
        "Exactly 1 confirmed entry must be recovered"
    );

    if let WalOp::Put { tx_id, key, value } = &recovered_entries[0].op {
        assert_eq!(*tx_id, TxId::new(100));
        assert_eq!(key, b"confirmed_key");
        assert_eq!(value, b"confirmed_val");
    } else {
        panic!("Unexpected op type recovered");
    }
}
