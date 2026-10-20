//! Tests for Stufe 6: X-08 Async I/O Non-Blocking Compliance.

use contextra_store::wal::{Wal, WalOp};
use contextra_types::TxId;
use tempfile::tempdir;

#[test]
fn wal_replay_does_not_block_current_thread_runtime() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("build runtime");

    runtime.block_on(async {
        let dir = tempdir().expect("tempdir");
        let wal_path = dir.path().join("wal_async_io.wal");

        let wal = Wal::open(&wal_path).await.expect("open wal");
        let tx1 = TxId::try_new(10).unwrap();
        let ops = vec![(
            WalOp::Put {
                tx_id: tx1,
                key: b"async_k".to_vec(),
                value: b"async_v".to_vec(),
            },
            1,
        )];

        let (batch, _) = wal.prepare_batch(ops).await.expect("prepare");
        wal.append_batch(batch).await.expect("append");
        wal.close().await.expect("close");

        let reopened = Wal::open_read_only(&wal_path, None)
            .await
            .expect("reopen wal");
        let entries = reopened.replay().await.expect("replay");

        assert_eq!(entries.len(), 1);
    });
}
