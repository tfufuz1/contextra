use contextra_store::lsm::{LsmConfig, LsmStorage};
use contextra_store::wal::Wal;
use contextra_types::TxId;
use tempfile::tempdir;
use tokio::fs;

#[tokio::test]
async fn repro_wal_tail_truncation_undetected_when_self_compared() {
    let dir = tempdir().expect("tempdir");
    let db_path = dir.path().to_path_buf();
    let config = LsmConfig {
        path: db_path.clone(),
        ..Default::default()
    };

    fs::write(db_path.join("SALT"), &[0u8; 32])
        .await
        .expect("write salt");

    // 1. Boot engine, write entry 1, flush to SSTable (which creates MANIFEST with WalCheckpoint HWM)
    {
        let storage = LsmStorage::new(config.clone()).await.expect("new storage");
        let tx1 = storage.begin_transaction_at(TxId::try_new(10).unwrap()).await.expect("tx1");
        storage.put_in_tx(&tx1, b"key1", b"val1").await.expect("put1");
        storage.commit_transaction(tx1).await.expect("commit1");
        storage.flush().await.expect("flush");

        // 2. Write entry 2 after flush into active WAL
        let tx2 = storage.begin_transaction_at(TxId::try_new(20).unwrap()).await.expect("tx2");
        storage.put_in_tx(&tx2, b"key2", b"val2").await.expect("put2");
        storage.commit_transaction(tx2).await.expect("commit2");

        storage.close().await.expect("close storage");
    }

    // 3. Truncate active WAL to remove entry 2 (after flush)
    let wal_path = db_path.join("wal.log");
    if wal_path.exists() {
        let wal_read = Wal::open(&wal_path).await.expect("open wal read");
        let entries = wal_read.replay().await.expect("replay entries");
        if entries.len() >= 2 {
            let (_, _, offset_first) = entries[0];
            drop(wal_read);
            let mut wal_bytes = fs::read(&wal_path).await.expect("read wal");
            wal_bytes.truncate(offset_first as usize);
            fs::write(&wal_path, wal_bytes).await.expect("write truncated wal");
        } else {
            drop(wal_read);
        }
    }

    // 4. Reopen engine after truncation
    let reopen_res = LsmStorage::new(config).await;

    // Proof of self-comparison vulnerability on unpatched code:
    // With `check_hmac = expected_hwm`, reopen_res succeeds despite truncated active WAL!
    println!("REPRO_RESULT: reopen success = {}", reopen_res.is_ok());
}
