use memfuse_core::{Result, TxId};
use memfuse_store::wal::{Wal, WalConfig, WalFlusherConfig, WalOp};
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn proof_wal_size_counter_consistent_after_failed_truncate() -> Result<()> {
    let dir = tempdir()?;
    let wal_path = dir.path().join("failed_truncate.wal");

    let config = WalConfig {
        flusher_config: WalFlusherConfig {
            batch_window_micros: 0,
        },
        ..Default::default()
    };
    let wal = Wal::open_with_config(&wal_path, config).await?;

    // Append an entry to increase size beyond header
    let op = WalOp::Put {
        tx_id: TxId::new(1),
        key: b"test_key".to_vec(),
        value: b"test_value_long_enough_to_have_size".to_vec(),
    };
    let (batch, _) = wal.prepare_batch(vec![(op, 1)]).await?;
    wal.append_batch(batch).await?;

    let original_size = wal.size();
    assert!(
        original_size > 4,
        "Original size should be greater than 4 bytes header"
    );

    // Call truncate with u64::MAX which causes set_len to fail with EFIBG / EINVAL on Linux
    let target_offset = u64::MAX;
    let new_hmac = [0xAAu8; 32];
    let res = wal.truncate(target_offset, new_hmac).await;

    assert!(res.is_err(), "truncate with u64::MAX offset must fail");

    // D-1 check: size counter MUST NOT have been updated if truncate failed
    let size_after_failed_truncate = wal.size();
    assert_eq!(
        size_after_failed_truncate, original_size,
        "WAL size counter ({size_after_failed_truncate}) was modified to target_offset before set_len succeeded! Expected original size ({original_size})."
    );

    Ok(())
}

#[tokio::test]
async fn proof_wal_truncate_ordering_under_concurrent_flush() -> Result<()> {
    let dir = tempdir()?;
    let wal_path = dir.path().join("concurrent_flush_truncate.wal");

    let wal = Arc::new(Wal::open(&wal_path).await?);

    // Spawn concurrent writers and truncaters to test ordering and state consistency
    let mut handles = Vec::new();
    let num_iterations = 50;

    for i in 0..num_iterations {
        let wal_writer = Arc::clone(&wal);
        handles.push(tokio::spawn(async move {
            let op = WalOp::Put {
                tx_id: TxId::new(i + 1),
                key: format!("k_{i}").into_bytes(),
                value: format!("v_{i}").into_bytes(),
            };
            if let Ok((batch, _)) = wal_writer.prepare_batch(vec![(op, i + 1)]).await {
                let _ = wal_writer.append_batch(batch).await;
            }
        }));

        if i % 5 == 0 {
            let wal_truncater = Arc::clone(&wal);
            handles.push(tokio::spawn(async move {
                let _ = wal_truncater.truncate(4, [0xBB; 32]).await;
            }));
        }
    }

    for h in handles {
        let _ = h.await;
    }

    // Post-concurrency sanity check:
    // Physical disk size and in-memory size counter must match exactly
    let disk_size = tokio::fs::metadata(&wal_path).await?.len();
    let mem_size = wal.size();

    assert_eq!(
        mem_size, disk_size,
        "In-memory WAL size ({mem_size}) diverged from physical disk size ({disk_size})"
    );

    Ok(())
}
