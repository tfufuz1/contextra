use contextra_core::{Result, TxId};
use contextra_store::{
    sstable::{BlockBuilder, SstableBuilder},
    wal::{hmac::LegacyKeyStatus, Wal, WalEntry, WalOp},
};
use tempfile::tempdir;

#[tokio::test]
async fn test_j08closure_block_builder_and_sstable_builder_version() -> Result<()> {
    let mut block_builder = BlockBuilder::new(4096);
    let size_before = block_builder.current_size();
    assert!(size_before > 0);

    block_builder.add(b"k1", b"v1", 10, 100);
    let size_after = block_builder.current_size();
    assert!(size_after > size_before);

    let dir = tempdir().expect("tempdir creation failed");
    let sst_path = dir.path().join("v3_format.sst");

    let mut sst_builder = SstableBuilder::create(&sst_path).await?;
    sst_builder.set_format_version(3);
    sst_builder.add(b"k1", b"v1", 10, 100).await?;

    let meta = sst_builder.finish().await?;
    assert!(meta.file_size > 0);

    Ok(())
}

#[tokio::test]
async fn test_j08closure_wal_checksum_methods_and_legacy_key_status() -> Result<()> {
    let op = WalOp::Put {
        tx_id: TxId::new(1),
        key: b"k1".to_vec(),
        value: b"v1".to_vec(),
    };
    let integrity_key = [7u8; 32];
    let prev_hmac = [0u8; 32];

    let cs = WalEntry::compute_checksum(&op, 1, &integrity_key, prev_hmac)?;
    let cs_v3 = WalEntry::compute_checksum_v3(&op, 1, &integrity_key, prev_hmac)?;
    assert_eq!(cs, cs_v3);

    let cs_v2 = WalEntry::compute_checksum_v2(&op, 1, &integrity_key, prev_hmac)?;
    assert_ne!(cs_v2, [0u8; 32]);

    let standard_status = LegacyKeyStatus::Standard;
    assert!(standard_status.is_standard());
    assert!(!standard_status.is_legacy());

    let legacy_status = LegacyKeyStatus::LegacyActive;
    assert!(!legacy_status.is_standard());
    assert!(legacy_status.is_legacy());

    Ok(())
}

#[tokio::test]
async fn test_j08closure_wal_try_append_batch_and_replay_stream() -> Result<()> {
    let dir = tempdir().expect("tempdir creation failed");
    let wal_path = dir.path().join("test_try_append.wal");

    let wal = Wal::open(&wal_path).await?;
    let op = WalOp::Put {
        tx_id: TxId::new(100),
        key: b"key_a".to_vec(),
        value: b"val_a".to_vec(),
    };

    let (batch, _prev_hmac) = wal.prepare_batch(vec![(op, 1001)]).await?;

    wal.try_append_batch(batch.clone()).await?;

    let truncate_guard = wal.truncate_lock.lock().await;
    let (batch2, _) = wal
        .prepare_batch(vec![(
            WalOp::Put {
                tx_id: TxId::new(101),
                key: b"key_b".to_vec(),
                value: b"val_b".to_vec(),
            },
            1002,
        )])
        .await?;
    wal.try_append_batch_locked(batch2, &truncate_guard).await?;
    drop(truncate_guard);

    let stream_entries = wal.replay_stream().await?;
    assert_eq!(stream_entries.len(), 2);
    assert_eq!(stream_entries[0].0, 1001);
    assert_eq!(stream_entries[1].0, 1002);

    Ok(())
}
