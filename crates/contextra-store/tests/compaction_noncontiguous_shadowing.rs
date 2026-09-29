use contextra_core::{Result, StorageEngine, TxId};
use contextra_store::compaction::CompactionConfig;
use contextra_store::lsm::{LsmConfig, LsmStorage};
use tempfile::TempDir;

#[tokio::test]
async fn test_noncontiguous_compaction_shadowing_overwrite() -> Result<()> {
    let tmp = TempDir::new().expect("temp dir");
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        compaction: CompactionConfig {
            min_sstables_per_tier: 2,
            size_ratio: 4.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let storage = LsmStorage::new(config).await?;

    let target_key = b"a_target_key";
    let val_v1 = b"value_v1_old";
    let val_v2 = b"value_v2_new";

    // 1. SSTable A: target_key = v1 + filler keys (~100KB)
    let tx1 = TxId::new(1);
    storage.put(tx1, target_key, val_v1).await?;
    for i in 0..300 {
        let key = format!("filler_a_{:04}", i).into_bytes();
        let val = vec![0xAA; 300];
        storage.put(tx1, &key, &val).await?;
    }
    storage.commit(tx1).await?;
    storage.force_flush().await?;

    // 2. SSTable X: target_key = v2 + filler keys (~90KB)
    let tx2 = TxId::new(2);
    storage.put(tx2, target_key, val_v2).await?;
    for i in 0..250 {
        let key = format!("filler_x_{:04}", i).into_bytes();
        let val = vec![0xCC; 300];
        storage.put(tx2, &key, &val).await?;
    }
    storage.commit(tx2).await?;
    storage.force_flush().await?;

    // 3. SSTable B: filler keys only (~100KB)
    let tx3 = TxId::new(3);
    for i in 0..300 {
        let key = format!("filler_b_{:04}", i).into_bytes();
        let val = vec![0xBB; 300];
        storage.put(tx3, &key, &val).await?;
    }
    storage.commit(tx3).await?;
    storage.force_flush().await?;

    // Prior to compaction, reading target_key returns v2
    assert_eq!(
        storage.get(target_key).await?,
        Some(bytes::Bytes::from_static(val_v2)),
        "Pre-compaction read must return v2"
    );

    // Force compaction round
    let compacted = storage.maybe_compact().await?;
    assert!(compacted, "Compaction should be triggered");

    let val_after_compact = storage.get(target_key).await?;

    // AFTER compaction, target_key MUST STILL return v2!
    assert_eq!(
        val_after_compact,
        Some(bytes::Bytes::from_static(val_v2)),
        "Post-compaction read must return v2, not stale v1 or None"
    );

    let val_at_seq = storage.get_at_seq(target_key, u64::MAX).await?;
    assert_eq!(
        val_at_seq,
        Some(bytes::Bytes::from_static(val_v2)),
        "get_at_seq must return v2"
    );

    let prefix_res = storage.scan_prefix(b"a_target_key").await?;
    assert_eq!(prefix_res.len(), 1);
    assert_eq!(prefix_res[0].1, val_v2);

    Ok(())
}
