use contextra_core::{ContextraError, TxId};
use contextra_crypto::KeyManager;
use contextra_store::wal::{Wal, WalOp};
use std::sync::Arc;
use tempfile::tempdir;
use tokio::fs;

#[tokio::test]
async fn test_aead_tail_corruption_mid_file_returns_wal_corruption_not_truncated() {
    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("encrypted_midfile_corruption.wal");

    let km = Arc::new(
        KeyManager::try_new("passphrase123", b"salt123456789012345678901234567890").expect("km"),
    );

    // 1. Write 3 encrypted batches
    {
        let wal = Wal::open_with_key_manager(&wal_path, Some(km.clone()))
            .await
            .expect("open wal");

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
        wal.close().await.expect("close wal");
    }

    // 2. Locate batch 2 and corrupt its AEAD payload in the middle of the file
    let mut data = fs::read(&wal_path).await.expect("read wal file");
    let initial_len = data.len();

    // Flip bit in batch 2 (offset around middle of file)
    let mid_offset = initial_len / 2;
    data[mid_offset] ^= 0xFF;
    fs::write(&wal_path, &data).await.expect("write corrupted wal");

    // 3. Opening / replaying WAL must strictly return WalCorruption, not silent truncation!
    let reopen_res = Wal::open_with_key_manager(&wal_path, Some(km)).await;
    let replay_res = match reopen_res {
        Ok(wal) => wal.replay().await,
        Err(e) => Err(e),
    };

    assert!(
        replay_res.is_err(),
        "Opening/replaying WAL with mid-file AEAD corruption MUST fail"
    );

    let err = replay_res.unwrap_err();
    assert!(
        matches!(err, ContextraError::WalCorruption { .. }),
        "Expected ContextraError::WalCorruption, got: {:?}",
        err
    );
}
