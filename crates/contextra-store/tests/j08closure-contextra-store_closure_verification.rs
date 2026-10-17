use contextra_core::{Result, TxId};
use contextra_store::sstable::BlockBuilder;
use contextra_store::wal::hmac::LegacyKeyStatus;
use contextra_store::wal::{Wal, WalOp};
use tempfile::tempdir;

#[tokio::test]
async fn test_j08closure_block_builder_and_sstable_builder() -> Result<()> {
    // 1. BlockBuilder::current_size
    let mut bb = BlockBuilder::new(4096);
    let size_empty = bb.current_size();
    assert!(size_empty > 0, "Empty BlockBuilder size should be > 0");

    bb.add(b"key1", b"value1", 1, 100);
    let size_one = bb.current_size();
    assert!(
        size_one > size_empty,
        "BlockBuilder current_size should grow after add"
    );

    // 2. SstableBuilder::set_format_version & SstableBuilder::current_size
    let dir = tempdir().expect("tempdir");
    let sstable_path = dir.path().join("test.sst");
    let mut builder = contextra_store::sstable::SstableBuilder::create(&sstable_path).await?;
    builder.set_format_version(3);
    assert!(builder.current_size() > 0);
    builder.add(b"k1", b"v1", 10, 100).await?;
    assert!(builder.current_size() > 0);
    let meta = builder.finish().await?;
    assert_eq!(meta.first_key.as_ref(), b"k1");
    assert_eq!(meta.last_key.as_ref(), b"k1");

    Ok(())
}

#[test]
fn test_j08closure_legacy_key_status_is_standard() {
    let standard = LegacyKeyStatus::Standard;
    let legacy = LegacyKeyStatus::LegacyActive;

    assert!(standard.is_standard());
    assert!(!legacy.is_standard());
    assert!(legacy.is_legacy());
    assert!(!standard.is_legacy());
}

#[tokio::test]
async fn test_j08closure_wal_try_append_batch_and_replay_stream() -> Result<()> {
    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("test_wal.log");

    let wal = Wal::open(&wal_path).await?;
    assert!(wal.legacy_key_status().is_standard());

    // Prepare a batch
    let ops = vec![
        (
            WalOp::Put {
                tx_id: TxId::new(1),
                key: b"k1".to_vec(),
                value: b"v1".to_vec(),
            },
            1,
        ),
        (
            WalOp::Put {
                tx_id: TxId::new(1),
                key: b"k2".to_vec(),
                value: b"v2".to_vec(),
            },
            2,
        ),
    ];
    let (batch, _last_hmac) = wal.prepare_batch(ops).await?;

    // Test try_append_batch
    wal.try_append_batch(batch).await?;

    // Test try_append_batch_locked
    let truncate_guard = wal.truncate_lock.lock().await;
    let (batch2, _) = wal
        .prepare_batch(vec![(
            WalOp::TxEnd {
                tx_id: TxId::new(1),
                committed: true,
            },
            3,
        )])
        .await?;
    wal.try_append_batch_locked(batch2, &truncate_guard).await?;
    drop(truncate_guard);

    // Test replay_stream
    let stream_entries = wal.replay_stream().await?;
    assert_eq!(stream_entries.len(), 3);
    assert_eq!(stream_entries[0].0, 1);
    assert_eq!(stream_entries[1].0, 2);
    assert_eq!(stream_entries[2].0, 3);

    Ok(())
}
