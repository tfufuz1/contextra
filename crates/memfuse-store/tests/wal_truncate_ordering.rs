// FILE-CONTEXT
// ZWECK: Audit-Test B-7 — Proof WAL-Truncate-Ordering & Size-Counter-Integrität.
// INVARIANT 1: WAL-Size-Zähler (self.size) DARF erst NACH erfolgreichem set_len() aktualisiert werden.
// INVARIANT 2: truncate() und append_batch() müssen durch Mutex serialisiert sein.

use memfuse_core::{Result, TxId};
use memfuse_store::wal::{Wal, WalEntry, WalOp};
use std::sync::Arc;
use tempfile::tempdir;

pub fn verify_hmac_chain(entries: &[WalEntry]) -> bool {
    if entries.is_empty() {
        return true;
    }
    for i in 1..entries.len() {
        if entries[i].prev_hmac != entries[i - 1].checksum {
            return false;
        }
    }
    true
}

#[tokio::test]
async fn proof_hmac_chain_valid_after_truncate_and_rewrite() -> Result<()> {
    let dir = tempdir()?;
    let wal_path = dir.path().join("truncate_rewrite_hmac.wal");

    let wal = Wal::open(&wal_path).await?;

    // 1. Write 50 WAL entries
    let mut hmac_at_20 = [0u8; 32];
    for i in 1..=50u64 {
        let op = WalOp::Put {
            tx_id: TxId::new(i),
            key: format!("k_{i}").into_bytes(),
            value: format!("v_{i}").into_bytes(),
        };
        let (batch, _last_hmac) = wal.prepare_batch(vec![(op, i)]).await?;
        wal.append_batch(batch).await?;

        if i == 20 {
            hmac_at_20 = wal.last_hmac_snapshot().await;
        }
    }

    // Record offset at entry 20
    let (offset_at_20, snapshot_hmac_at_20) = wal.find_tx_offset(TxId::new(20)).await?;
    assert_eq!(snapshot_hmac_at_20, hmac_at_20);

    // 2. Truncate back to entry 20
    wal.truncate(offset_at_20, hmac_at_20).await?;

    // 3. Write 10 new entries (seq_no 21..30)
    for i in 21..=30u64 {
        let op = WalOp::Put {
            tx_id: TxId::new(i),
            key: format!("new_k_{i}").into_bytes(),
            value: format!("new_v_{i}").into_bytes(),
        };
        let (batch, _) = wal.prepare_batch(vec![(op, i)]).await?;
        wal.append_batch(batch).await?;
    }

    // 4. Replay and verify HMAC chain
    let replayed = wal.replay().await?;
    let entries: Vec<WalEntry> = replayed.into_iter().map(|(_, e, _)| e).collect();

    assert_eq!(
        entries.len(),
        30,
        "Replayed entry count must be exactly 30 (20 original + 10 rewritten)"
    );

    assert!(
        verify_hmac_chain(&entries),
        "HMAC chain continuity check failed after truncate and rewrite"
    );

    Ok(())
}

#[tokio::test]
async fn proof_size_counter_consistent_after_truncate() -> Result<()> {
    let dir = tempdir()?;
    let wal_path = dir.path().join("size_counter_truncate.wal");

    let wal = Wal::open(&wal_path).await?;

    for i in 1..=5u64 {
        let op = WalOp::Put {
            tx_id: TxId::new(i),
            key: format!("k_{i}").into_bytes(),
            value: format!("v_{i}").into_bytes(),
        };
        let (batch, _) = wal.prepare_batch(vec![(op, i)]).await?;
        wal.append_batch(batch).await?;
    }

    let size_before = wal.size();
    assert!(size_before > 0, "WAL size should be non-zero after writes");

    wal.truncate(0, [0u8; 32]).await?;

    assert_eq!(wal.size(), 0, "In-memory wal.size() must be 0 after truncate(0)");

    let file_meta = tokio::fs::metadata(&wal_path).await?;
    assert_eq!(
        file_meta.len(),
        0,
        "Physical file length on disk must be 0 after truncate(0)"
    );

    Ok(())
}

#[test]
fn proof_flusher_actor_exclusive_after_single_consumer_refactor() {
    let source = include_str!("../src/wal/io.rs");
    let lock_calls: Vec<_> = source
        .lines()
        .enumerate()
        .filter(|(_, l)| l.contains("file.lock()") && !l.trim().starts_with("//"))
        .collect();
    assert!(
        lock_calls.len() <= 2,
        "WAL file.lock() Callsites: {} (max 2 erwartet)",
        lock_calls.len()
    );
}

#[tokio::test]
async fn proof_concurrent_flush_and_truncate_no_panic() -> Result<()> {
    use std::time::Duration;

    let res = tokio::time::timeout(Duration::from_secs(3), async {
        let dir = tempdir()?;
        let wal_path = dir.path().join("concurrent_no_panic.wal");
        let wal = Arc::new(Wal::open(&wal_path).await?);

        let wal_writer = Arc::clone(&wal);
        let writer_handle = tokio::spawn(async move {
            for i in 1..=100u64 {
                let op = WalOp::Put {
                    tx_id: TxId::new(i),
                    key: format!("k_{i}").into_bytes(),
                    value: format!("v_{i}").into_bytes(),
                };
                if let Ok((batch, _)) = wal_writer.prepare_batch(vec![(op, i)]).await {
                    let _ = wal_writer.append_batch(batch).await;
                }
                tokio::task::yield_now().await;
            }
        });

        let wal_truncater = Arc::clone(&wal);
        let truncater_handle = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            let _ = wal_truncater.truncate(4, [0u8; 32]).await;
        });

        let (res_w, res_t) = tokio::join!(writer_handle, truncater_handle);
        res_w.expect("writer task panicked");
        res_t.expect("truncater task panicked");

        Ok::<(), memfuse_core::MemFuseError>(())
    })
    .await;

    assert!(res.is_ok(), "Concurrent flush and truncate timed out or failed");
    res.unwrap()?;
    Ok(())
}

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
