// FILE-CONTEXT
// ZWECK: Unabhängiges Dateisystem-Orakel für WAL size vs. physische Dateigröße.
// ORACLE SOURCE: std::fs::metadata(wal.path()).len() (Betriebssystem-Dateisystem).
// INVARIANTEN:
// 1. Sequentiell (ohne Nebenläufigkeit): nach jedem awaited append_batch und truncate
//    gilt: wal.size() == std::fs::metadata(wal.path()).unwrap().len().
// 2. Nebenläufig (Append-Phase): Bei richtiger Lese-Reihenfolge (erst wal.size(), dann fs::metadata)
//    gilt während Append: mem_size <= disk_size (weil fetch_add nach dem Schreiben erfolgt).

use contextra_core::{Result, TxId};
use contextra_store::wal::{Wal, WalOp};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn oracle_sequential_wal_size_matches_disk_metadata() -> Result<()> {
    let dir = tempdir()?;
    let wal_path = dir.path().join("sequential_fs_oracle.wal");

    let wal = Wal::open(&wal_path).await?;

    // Initial state check (Oracle = std::fs::metadata)
    let disk_size_init = std::fs::metadata(&wal_path)?.len();
    assert_eq!(
        wal.size(),
        disk_size_init,
        "Initial wal.size() must match physical disk size (Oracle)"
    );

    for i in 1..=20u64 {
        // Append batch
        let op1 = WalOp::Put {
            tx_id: TxId::new(i * 2 - 1),
            key: format!("oracle_k_{i}_1").into_bytes(),
            value: format!("oracle_v_{i}_1").into_bytes(),
        };
        let op2 = WalOp::Put {
            tx_id: TxId::new(i * 2),
            key: format!("oracle_k_{i}_2").into_bytes(),
            value: format!("oracle_v_{i}_2").into_bytes(),
        };
        let (batch, _) = wal.prepare_batch(vec![(op1, i * 2 - 1), (op2, i * 2)]).await?;
        wal.append_batch(batch).await?;

        // Oracle verification after completed append
        let disk_size_after_append = std::fs::metadata(&wal_path)?.len();
        let mem_size_after_append = wal.size();
        assert_eq!(
            mem_size_after_append, disk_size_after_append,
            "After completed append_batch, wal.size() ({mem_size_after_append}) must match disk size ({disk_size_after_append})"
        );
        assert!(
            mem_size_after_append > 4,
            "WAL size after append must be larger than header"
        );

        // Truncate back to offset 4 (HEADER)
        wal.truncate(4, [0xCC; 32]).await?;

        // Oracle verification after completed truncate
        let disk_size_after_truncate = std::fs::metadata(&wal_path)?.len();
        let mem_size_after_truncate = wal.size();

        assert_eq!(
            mem_size_after_truncate, 4,
            "wal.size() must be exactly 4 after truncate(4)"
        );
        assert_eq!(
            disk_size_after_truncate, 4,
            "Physical file size must be exactly 4 after truncate(4)"
        );
        assert_eq!(
            mem_size_after_truncate, disk_size_after_truncate,
            "After completed truncate, wal.size() ({mem_size_after_truncate}) must equal disk size ({disk_size_after_truncate})"
        );
    }

    Ok(())
}

#[tokio::test]
async fn oracle_concurrent_append_wal_size_read_order_invariant() -> Result<()> {
    let dir = tempdir()?;
    let wal_path = dir.path().join("concurrent_append_fs_oracle.wal");

    let wal = Arc::new(Wal::open(&wal_path).await?);
    let done = Arc::new(AtomicBool::new(false));

    let wal_writer = Arc::clone(&wal);
    let done_writer = Arc::clone(&done);
    let writer_task = tokio::spawn(async move {
        for i in 1..=100u64 {
            let op = WalOp::Put {
                tx_id: TxId::new(i),
                key: format!("concurrent_k_{i}").into_bytes(),
                value: format!("concurrent_v_{i}").into_bytes(),
            };
            if let Ok((batch, _)) = wal_writer.prepare_batch(vec![(op, i)]).await {
                wal_writer.append_batch(batch).await.expect("append_batch");
            }
            tokio::task::yield_now().await;
        }
        done_writer.store(true, Ordering::SeqCst);
    });

    let wal_poller = Arc::clone(&wal);
    let done_poller = Arc::clone(&done);
    let poller_task = tokio::spawn(async move {
        while !done_poller.load(Ordering::SeqCst) {
            // Correct order: 1st read wal.size() (mem_size), 2nd read fs::metadata (disk_size).
            // During appends, file write happens BEFORE fetch_add.
            // So mem_size read at t1 is <= disk_size read at t2.
            let mem_size = wal_poller.size();
            if let Ok(meta) = tokio::fs::metadata(wal_poller.path()).await {
                let disk_size = meta.len();
                assert!(
                    mem_size <= disk_size,
                    "Monotonic append invariant violated: mem_size ({mem_size}) > disk_size ({disk_size})"
                );
            }
            tokio::task::yield_now().await;
        }
    });

    let (res_w, res_p) = tokio::join!(writer_task, poller_task);
    res_w.expect("writer panicked");
    res_p.expect("poller panicked");

    // Final sequential verification after concurrent appends finish
    let final_disk_size = std::fs::metadata(&wal_path)?.len();
    assert_eq!(
        wal.size(),
        final_disk_size,
        "Final wal.size() must equal physical disk size after all appends complete"
    );

    Ok(())
}
