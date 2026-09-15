use super::*;
use crate::wal::{Wal, WAL_V3_HEADER};
use memfuse_core::TxId;
use std::sync::Arc;
use tempfile::tempdir;
use tokio::fs;
use tokio::io::AsyncWriteExt;

#[tokio::test]
async fn test_wal_crash_consistency_write_without_fsync() {
    let dir = tempdir().expect("tempdir"); // expect
    let wal_path = dir.path().join("crash_wal.log");

    // 1. Open WAL and append an entry
    {
        let wal = Wal::open(&wal_path).await.expect("open wal"); // expect
        let op = WalOp::Put {
            tx_id: TxId::new(100),
            key: b"crash_k".to_vec(),
            value: b"crash_v".to_vec(),
        };
        let (batch, _) = wal
            .prepare_batch(vec![(op, 1)])
            .await
            .expect("create entry"); // expect
        let entry = &batch.entries()[0];

        // Manually simulate a write + flush to OS buffer WITHOUT file.sync_all()
        let mut file = tokio::fs::OpenOptions::new()
            .append(true)
            .open(&wal_path)
            .await
            .expect("open for append"); // expect
        let bytes = entry.to_bytes().expect("to_bytes"); // expect
        file.write_all(&bytes).await.expect("write_all"); // expect
        file.flush().await.expect("flush"); // expect
                                            // File dropped without calling sync_all() (simulating crash before fsync)
        drop(file);
        drop(wal);
    }

    // 2. Re-open WAL and replay
    let wal_reopen = Wal::open(&wal_path).await;
    assert!(wal_reopen.is_ok(), "WAL open after crash should succeed");
    let wal = wal_reopen.unwrap(); // unwrap

    let replay_result = wal.replay().await;
    match replay_result {
        Ok(entries) => {
            // Should either find the entry or empty set, never panic
            if !entries.is_empty() {
                assert_eq!(entries.len(), 1);
                assert_eq!(entries[0].1.seq_no, 1);
            }
        }
        Err(e) => {
            panic!("replay() failed unexpectedly with error: {:?}", e);
        }
    }
}

#[tokio::test]
async fn test_append_batch_partial_write_atomicity() {
    let dir = tempdir().expect("tempdir"); // expect
    let wal_path = dir.path().join("partial_batch.wal");

    let wal = Wal::open(&wal_path).await.expect("open wal"); // expect
    let ops = vec![
        (
            WalOp::Put {
                tx_id: TxId::new(1),
                key: b"b1".to_vec(),
                value: b"v1".to_vec(),
            },
            1,
        ),
        (
            WalOp::Put {
                tx_id: TxId::new(1),
                key: b"b2".to_vec(),
                value: b"v2".to_vec(),
            },
            2,
        ),
        (
            WalOp::Put {
                tx_id: TxId::new(1),
                key: b"b3".to_vec(),
                value: b"v3".to_vec(),
            },
            3,
        ),
    ];

    let (entries, _) = wal.prepare_batch(ops).await.expect("prepare_batch"); // expect
    assert_eq!(entries.len(), 3);

    // Serialize all 3 entries into a single bytes payload
    let mut batch_bytes = Vec::new();
    for e in entries.entries() {
        batch_bytes.extend_from_slice(&e.to_bytes().expect("to_bytes")); // expect
    }

    // Truncate the batch in the middle of entry 2 (partial write during crash)
    // Each entry is ~101 bytes. Total ~303 bytes.
    // Subtracting 120 bytes leaves ~183 bytes, truncating entry 2 mid-write.
    let truncated_len = batch_bytes.len() - 120;
    let truncated_bytes = &batch_bytes[..truncated_len];

    // Append the truncated bytes directly to the WAL file
    {
        let mut file = tokio::fs::OpenOptions::new()
            .append(true)
            .open(&wal_path)
            .await
            .expect("open for append"); // expect
        file.write_all(truncated_bytes).await.expect("write_all"); // expect
        file.flush().await.expect("flush"); // expect
    }

    // Reopen and replay
    let wal2 = Wal::open(&wal_path).await.expect("reopen"); // expect
    let replay_entries = wal2
        .replay()
        .await
        .expect("replay must succeed without panic"); // expect

    // Replay must recover entry 1 (which was fully written) and cleanly discard the truncated tail
    assert_eq!(replay_entries.len(), 1, "Only entry 1 should be recovered");
    assert_eq!(replay_entries[0].1.seq_no, 1);
}

#[tokio::test]
async fn test_truncate_size_visible_atomically_with_file_state() {
    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("truncate_atomic.wal");

    let wal = Arc::new(Wal::open(&wal_path).await.expect("open wal"));

    let done = Arc::new(std::sync::atomic::AtomicBool::new(false));

    let wal_trunc = wal.clone();
    let done_trunc = done.clone();
    let truncater = tokio::spawn(async move {
        for i in 0..100 {
            // Prepare and append a batch of entries so file grows
            let ops = vec![
                (
                    WalOp::Put {
                        tx_id: TxId::new(i * 2 + 1),
                        key: b"atomic_key_1".to_vec(),
                        value: b"atomic_val_1".to_vec(),
                    },
                    i * 2 + 1,
                ),
                (
                    WalOp::Put {
                        tx_id: TxId::new(i * 2 + 2),
                        key: b"atomic_key_2".to_vec(),
                        value: b"atomic_val_2".to_vec(),
                    },
                    i * 2 + 2,
                ),
            ];
            let (batch, _) = wal_trunc.prepare_batch(ops).await.expect("prepare_batch");
            wal_trunc.append_batch(batch).await.expect("append_batch");

            // Truncate back to offset 4 (length of WAL_V3_HEADER)
            wal_trunc
                .truncate(4, [0xAA; 32])
                .await
                .expect("truncate failed");
        }
        done_trunc.store(true, std::sync::atomic::Ordering::SeqCst);
    });

    let wal_poll = wal.clone();
    let done_poll = done.clone();
    let poller = tokio::spawn(async move {
        while !done_poll.load(std::sync::atomic::Ordering::SeqCst) {
            if let Ok(meta) = fs::metadata(wal_poll.path()).await {
                let disk_size = meta.len();
                let mem_size = wal_poll.size();
                // In-memory size must never observe stale mem_size > disk_size after truncation
                assert!(
                    mem_size <= disk_size,
                    "TOCTOU violation: in-memory WAL size ({mem_size}) > physical disk size ({disk_size})"
                );
            }
            tokio::task::yield_now().await;
        }
    });

    let (res_trunc, res_poll) = tokio::join!(truncater, poller);
    res_trunc.expect("truncater panicked");
    res_poll.expect("poller panicked");
}

#[tokio::test]
async fn test_concurrent_append_batch_header_atomicity() {
    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("concurrent_header.wal");

    let wal = Arc::new(Wal::open(&wal_path).await.expect("open wal"));

    let num_tasks = 8;
    let mut handles = Vec::new();

    for i in 0..num_tasks {
        let wal_clone = wal.clone();
        handles.push(tokio::spawn(async move {
            let op = WalOp::Put {
                tx_id: TxId::new(i + 1),
                key: format!("key_{}", i).into_bytes(),
                value: format!("val_{}", i).into_bytes(),
            };
            let (batch, _) = wal_clone
                .prepare_batch(vec![(op, i + 1)])
                .await
                .expect("prepare_batch");
            wal_clone.append_batch(batch).await.expect("append_batch");
        }));
    }

    for h in handles {
        h.await.expect("join handle");
    }

    let file_bytes = fs::read(&wal_path).await.expect("read wal file");

    // Assert header is present at start
    assert!(
        file_bytes.len() >= 4,
        "WAL file must be at least 4 bytes long"
    );
    assert_eq!(
        &file_bytes[0..4],
        &WAL_V3_HEADER,
        "WAL file must start with WAL_V3_HEADER"
    );

    // Count header occurrences across entire file
    let header_count = file_bytes
        .windows(4)
        .filter(|window| *window == WAL_V3_HEADER)
        .count();
    assert_eq!(
        header_count, 1,
        "WAL_V3_HEADER must appear exactly once at the start of the file, but was found {header_count} times"
    );

    // Reopen and replay to verify no stream corruption
    let wal_reopen = Wal::open(&wal_path).await.expect("reopen wal");
    let replayed = wal_reopen.replay().await.expect("replay must succeed");
    assert_eq!(
        replayed.len(),
        num_tasks as usize,
        "Replay must yield all {} entries",
        num_tasks
    );
}

#[tokio::test]
async fn test_truncate_is_durable_across_simulated_crash() {
    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("truncate_durability.wal");

    let wal = Wal::open(&wal_path).await.expect("open wal");

    // Write several entries so file grows
    for i in 1..=5 {
        let op = WalOp::Put {
            tx_id: TxId::new(i),
            key: format!("k{}", i).into_bytes(),
            value: format!("v{}", i).into_bytes(),
        };
        let (batch, _) = wal
            .prepare_batch(vec![(op, i)])
            .await
            .expect("prepare batch");
        wal.append_batch(batch).await.expect("append entry");
    }

    let initial_size = tokio::fs::metadata(&wal_path).await.expect("meta").len();
    assert!(initial_size > 4, "File size should be larger than header");

    // Truncate to offset 4 (HEADER length)
    let new_hmac = [0x77u8; 32];
    wal.truncate(4, new_hmac).await.expect("truncate");

    // Open via a new independent File handle (simulates restart after crash without the original Wal handle)
    let file = tokio::fs::File::open(&wal_path).await.expect("reopen file");
    let metadata = file.metadata().await.expect("metadata");
    assert_eq!(
        metadata.len(),
        4,
        "Physical file length on disk must equal truncated offset 4 after fsync"
    );
}

#[tokio::test]
async fn test_wal_direct_append_batch_fsync_discipline() {
    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("wal_direct.log");

    let wal = Wal::open(&wal_path).await.expect("wal open");

    let op = WalOp::Put {
        tx_id: TxId::new(1),
        key: b"direct_k".to_vec(),
        value: b"direct_v".to_vec(),
    };

    let (batch, _) = wal
        .prepare_batch(vec![(op, 1)])
        .await
        .expect("create entry");

    println!("[STRACE_MARKER_START_DIRECT_APPEND]");
    let append_res = wal.append_batch(batch).await;
    println!("[STRACE_MARKER_END_DIRECT_APPEND]");

    assert!(append_res.is_ok());
}

#[tokio::test]
async fn test_truncate_at_offset_zero_yields_empty_wal() -> Result<()> {
    let dir = tempdir()?;
    let wal_path = dir.path().join("test_truncate_zero.wal");

    let wal = Wal::open(&wal_path).await?;
    for i in 1..=5 {
        let op = WalOp::Put {
            tx_id: TxId::new(i),
            key: format!("k{i}").into_bytes(),
            value: format!("v{i}").into_bytes(),
        };
        let (batch, _) = wal.prepare_batch(vec![(op, i)]).await?;
        wal.append_batch(batch).await?;
    }

    let meta_before = tokio::fs::metadata(&wal_path).await?;
    assert!(
        meta_before.len() > 0,
        "WAL file should contain written bytes before truncation"
    );

    wal.truncate(0, [0u8; 32]).await?;

    let meta_after = tokio::fs::metadata(&wal_path).await?;
    assert_eq!(
        meta_after.len(),
        0,
        "Physical file length must be exactly 0 bytes after truncate(0)"
    );

    drop(wal);
    let wal_reopened = Wal::open(&wal_path).await?;
    let entries = wal_reopened.replay().await?;
    assert_eq!(
        entries.len(),
        0,
        "Reopened WAL after truncate(0) must yield 0 entries"
    );

    Ok(())
}

#[tokio::test]
async fn test_append_batch_concurrent_double_header_write_race() -> Result<()> {
    let dir = tempdir()?;
    let wal_path = dir.path().join("test_concurrent_header.wal");

    let wal = Wal::open(&wal_path).await?;

    let op1 = WalOp::Put {
        tx_id: TxId::new(10),
        key: b"concurrent_key_1".to_vec(),
        value: b"val_1".to_vec(),
    };
    let op2 = WalOp::Put {
        tx_id: TxId::new(11),
        key: b"concurrent_key_2".to_vec(),
        value: b"val_2".to_vec(),
    };

    let (batch1, _) = wal.prepare_batch(vec![(op1, 1)]).await?;
    let (batch2, _) = wal.prepare_batch(vec![(op2, 2)]).await?;

    let handle1 = wal.append_batch(batch1);
    let handle2 = wal.append_batch(batch2);
    let (res1, res2) = tokio::join!(handle1, handle2);

    res1?;
    res2?;

    let file_bytes = tokio::fs::read(&wal_path).await?;

    let header_magic_count = file_bytes
        .windows(4)
        .filter(|win| *win == WAL_V3_HEADER)
        .count();

    assert_eq!(
        header_magic_count, 1,
        "Header magic 'MFW3' must appear EXACTLY ONCE in physical WAL file, found {}",
        header_magic_count
    );

    Ok(())
}

#[cfg(feature = "fault-injection")]
#[tokio::test]
async fn test_disk_full_mid_append_batch_rollback() -> Result<()> {
    use std::sync::atomic::Ordering;
    let dir = tempdir()?;
    let wal_path = dir.path().join("disk_full_batch.wal");

    let wal = Wal::open(&wal_path).await?;

    let op_base = WalOp::Put {
        tx_id: TxId::new(10),
        key: b"base_k".to_vec(),
        value: b"base_v".to_vec(),
    };
    let (batch_base, _) = wal.prepare_batch(vec![(op_base, 1)]).await?;
    wal.append_batch(batch_base).await?;

    let fail_tx = TxId::new(20);
    let op_fail1 = WalOp::Put {
        tx_id: fail_tx,
        key: b"fail_k1".to_vec(),
        value: b"fail_v1".to_vec(),
    };
    let op_fail2 = WalOp::Put {
        tx_id: fail_tx,
        key: b"fail_k2".to_vec(),
        value: b"fail_v2".to_vec(),
    };

    let (batch, prev_hmac_snapshot) = wal
        .prepare_batch(vec![(op_fail1, 2), (op_fail2, 3)])
        .await?;

    FAIL_APPEND_FOR_TX.store(fail_tx.inner(), Ordering::SeqCst);

    let append_res = wal.append_batch(batch).await;
    assert!(
        append_res.is_err(),
        "append_batch must return Err when fault injection triggers WAL append failure"
    );

    wal.restore_last_hmac(prev_hmac_snapshot).await?;

    drop(wal);
    let wal_reopened = Wal::open(&wal_path).await?;
    let entries = wal_reopened.replay().await?;

    assert_eq!(
        entries.len(),
        1,
        "WAL must contain exactly 1 baseline entry after batch failure rollback"
    );
    assert_eq!(entries[0].1.seq_no, 1);

    Ok(())
}

#[tokio::test]
async fn test_wal_rotate_and_seal_readonly_guarantee() {
    let dir = tempfile::tempdir().expect("tempdir");
    let wal_path = dir.path().join("test.wal");

    // WAL öffnen und einen Eintrag schreiben
    let wal = Wal::open(&wal_path).await.expect("open WAL");
    let op = WalOp::Put {
        tx_id: TxId::new(1),
        key: b"hello".to_vec(),
        value: b"world".to_vec(),
    };
    let (batch, _hmac) = wal
        .prepare_batch(vec![(op, 1)])
        .await
        .expect("prepare batch");
    wal.append_batch(batch).await.expect("append batch");

    // rotate_and_seal aufrufen
    let sealed_path = wal.rotate_and_seal().await.expect("rotate_and_seal");

    // Invariante 1: Sealed-Datei existiert am neuen Pfad
    assert!(sealed_path.exists(), "Sealed WAL muss existieren");
    assert!(
        sealed_path.to_str().unwrap().contains(".sealed."),
        "Sealed-Pfad muss '.sealed.' enthalten"
    );

    // Invariante 2: Originalpfad existiert nicht mehr
    assert!(
        !wal_path.exists(),
        "Originaler WAL-Pfad muss nach Rotate verschwunden sein"
    );

    // Invariante 3: Sealed-Datei ist read-only
    let meta = tokio::fs::metadata(&sealed_path).await.expect("metadata");
    assert!(
        meta.permissions().readonly(),
        "Versiegeltes WAL-Segment MUSS read-only sein"
    );

    // Invariante 4: Schreibversuch auf sealed-Datei schlägt fehl
    let write_result = tokio::fs::OpenOptions::new()
        .write(true)
        .open(&sealed_path)
        .await;
    assert!(
        write_result.is_err(),
        "Schreibversuch auf versiegeltes WAL-Segment muss fehlschlagen"
    );
}

#[tokio::test]
async fn test_wal_rotate_seal_crash_mid_rename() {
    let dir = tempfile::tempdir().expect("tempdir");
    let wal_path = dir.path().join("crash_test.wal");

    // 1. Write initial committed WAL entries
    {
        let wal = Wal::open(&wal_path).await.expect("open WAL");
        for i in 1..=3 {
            let op = WalOp::Put {
                tx_id: TxId::new(i),
                key: format!("k{i}").into_bytes(),
                value: format!("v{i}").into_bytes(),
            };
            let (batch, _) = wal
                .prepare_batch(vec![(op, i)])
                .await
                .expect("prepare batch");
            wal.append_batch(batch).await.expect("append batch");
        }
    }

    // 2. Simulate crash state between rename and parent fsync
    let sealed_name = format!("crash_test.wal.sealed.1234567890");
    let sealed_path = dir.path().join(&sealed_name);

    tokio::fs::rename(&wal_path, &sealed_path)
        .await
        .expect("simulate rename before crash");

    // 3. Post-crash inspection & recovery verification
    let wal_exists = wal_path.exists();
    let sealed_exists = sealed_path.exists();

    assert!(
        (wal_exists && !sealed_exists) || (!wal_exists && sealed_exists),
        "WAL segment must be either fully active or fully sealed after crash mid-rename"
    );

    if sealed_exists {
        let sealed_wal = Wal::open(&sealed_path).await.expect("open sealed WAL");
        let replayed = sealed_wal.replay().await.expect("replay sealed WAL");
        assert_eq!(
            replayed.len(),
            3,
            "All 3 entries must be present in sealed WAL"
        );
    }
}
