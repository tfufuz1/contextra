use contextra_core::{Result, TxId};
use contextra_store::{
    sstable::{BlockBuilder, SstableBuilder},
    wal::{Wal, WalOp},
};
use tempfile::tempdir;

#[tokio::test]
async fn test_j08_block_builder_current_size_and_sstable_set_format_version() {
    let mut bb = BlockBuilder::new(4096);
    let size_empty = bb.current_size();
    assert!(size_empty > 0);

    bb.add(b"key1", b"val1", 1, 100);
    let size_one = bb.current_size();
    assert!(size_one > size_empty);

    let dir = tempdir().expect("tempdir creation failed");
    let sst_path = dir.path().join("test_v3.sst");

    let mut builder = SstableBuilder::create(&sst_path)
        .await
        .expect("create builder");
    builder.set_format_version(3);

    builder
        .add(b"key1", b"val1", 1, 100)
        .await
        .expect("add key");
    let meta = builder.finish().await.expect("finish builder");
    assert!(meta.file_size > 0);
}

#[tokio::test]
async fn test_j08_wal_try_append_batch_and_replay_stream() -> Result<()> {
    let dir = tempdir().expect("tempdir creation failed");
    let wal_path = dir.path().join("test.wal");

    let wal = Wal::open(&wal_path).await?;
    let op = WalOp::Put {
        tx_id: TxId::new(1),
        key: b"k1".to_vec(),
        value: b"v1".to_vec(),
    };

    let (batch, _prev_hmac) = wal.prepare_batch(vec![(op, 1)]).await?;
    wal.try_append_batch(batch).await?;

    let stream_entries = wal.replay_stream().await?;
    assert_eq!(stream_entries.len(), 1);
    assert_eq!(stream_entries[0].0, 1);

    let mmap_entries = wal.replay().await?;
    assert_eq!(mmap_entries.len(), 1);
    assert_eq!(mmap_entries[0].0, 1);

    Ok(())
}
