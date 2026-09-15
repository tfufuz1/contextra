// FILE-CONTEXT
// ZWECK: Audit-Test B-7 — Proof WAL-Truncate-Ordering & Size-Counter-Integrität.
// INVARIANT 1: WAL-Size-Zähler (self.size) DARF erst NACH erfolgreichem set_len() aktualisiert werden.
// INVARIANT 2: truncate() und append_batch() müssen durch Mutex serialisiert sein.

use memfuse_core::{Result, TxId};
use memfuse_store::wal::{Wal, WalOp};
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn proof_wal_size_counter_consistent_after_failed_truncate() -> Result<()> {
    let dir = tempdir()?;
    let wal_path = dir.path().join("failed_truncate.wal");

    let wal = Wal::open(&wal_path).await?;

    // 1. Write initial entries to grow WAL file size
    for i in 1..=3 {
        let op = WalOp::Put {
            tx_id: TxId::new(i),
            key: format!("key_{i}").into_bytes(),
            value: format!("val_{i}").into_bytes(),
        };
        let (batch, _) = wal.prepare_batch(vec![(op, i)]).await?;
        wal.append_batch(batch).await?;
    }

    let initial_size = wal.size();
    assert!(initial_size > 4, "WAL size should be greater than header length");

    // 2. Simulate I/O failure for truncate by injecting a read-only file handle into wal.file
    {
        let ro_file = tokio::fs::OpenOptions::new()
            .read(true)
            .write(false)
            .open(&wal_path)
            .await?;
        wal.replace_file_handle_for_test(ro_file).await;
    }

    // 3. Call truncate to offset 4 (should fail due to read-only file handle)
    let truncate_res = wal.truncate(4, [0xBB; 32]).await;
    assert!(
        truncate_res.is_err(),
        "truncate must return Err when underlying file handle is read-only"
    );

    // 4. Invariant Check: wal.size() MUST NOT have been updated to 4
    let size_after_failed_truncate = wal.size();
    assert_eq!(
        size_after_failed_truncate, initial_size,
        "WAL size counter (wal.size()) was prematurely updated before set_len() succeeded! Expected {}, got {}",
        initial_size, size_after_failed_truncate
    );

    Ok(())
}

#[tokio::test]
async fn proof_wal_truncate_ordering_under_concurrent_flush() -> Result<()> {
    let dir = tempdir()?;
    let wal_path = dir.path().join("concurrent_flush_truncate.wal");

    let wal = Arc::new(Wal::open(&wal_path).await?);

    // Write baseline entry
    let op = WalOp::Put {
        tx_id: TxId::new(1),
        key: b"base_k".to_vec(),
        value: b"base_v".to_vec(),
    };
    let (batch, _base_hmac) = wal.prepare_batch(vec![(op, 1)]).await?;
    let last_hmac_1 = batch.entries()[0].checksum;
    wal.append_batch(batch).await?;

    let base_offset = wal.size();

    let done = Arc::new(std::sync::atomic::AtomicBool::new(false));

    let wal_writer = Arc::clone(&wal);
    let done_writer = Arc::clone(&done);
    let writer_handle = tokio::spawn(async move {
        let mut seq = 2u64;
        while !done_writer.load(std::sync::atomic::Ordering::Relaxed) {
            let op = WalOp::Put {
                tx_id: TxId::new(seq),
                key: format!("concurrent_k_{seq}").into_bytes(),
                value: b"concurrent_v".to_vec(),
            };
            if let Ok((batch, _)) = wal_writer.prepare_batch(vec![(op, seq)]).await {
                let _ = wal_writer.append_batch(batch).await;
            }
            seq += 1;
            tokio::task::yield_now().await;
        }
    });

    let wal_truncater = Arc::clone(&wal);
    let done_truncater = Arc::clone(&done);
    let truncater_handle = tokio::spawn(async move {
        for _ in 0..50 {
            tokio::task::yield_now().await;
            let _ = wal_truncater.truncate(base_offset, last_hmac_1).await;
        }
        done_truncater.store(true, std::sync::atomic::Ordering::Relaxed);
    });

    let (res_w, res_t) = tokio::join!(writer_handle, truncater_handle);
    res_w.expect("writer task panicked");
    res_t.expect("truncater task panicked");

    // Perform final truncate to known base_offset
    wal.truncate(base_offset, last_hmac_1).await?;

    // Reopen and replay WAL — must replay without HMAC corruption or invalid entry length errors
    drop(wal);
    let reopened = Wal::open(&wal_path).await?;
    let replayed = reopened.replay().await?;

    assert_eq!(
        replayed.len(),
        1,
        "WAL should replay cleanly to exactly 1 baseline entry after truncate operations"
    );
    assert_eq!(replayed[0].1.seq_no, 1);

    Ok(())
}
