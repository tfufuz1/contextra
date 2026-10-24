use contextra_core::TxId;
use contextra_store::wal::{Wal, WalEntry, WalOp, WAL_V3_HEADER};
use tempfile::tempdir;
use tokio::fs;

#[tokio::test]
async fn test_wal_no_hmac_multi_batch_replay_and_corruption() {
    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("wal_no_hmac.log");

    // Construct unchained zero-HMAC entries representing WalNoHmac mode
    let zero_key = [0u8; 32];
    let zero_chain = [0u8; 32];

    let op1 = WalOp::Put {
        tx_id: TxId::new(100),
        key: b"nohmac_key_1".to_vec(),
        value: b"nohmac_val_1".to_vec(),
    };
    let entry1 = WalEntry::try_new(op1, 1, &zero_key, zero_chain).expect("entry 1");

    let op2 = WalOp::Put {
        tx_id: TxId::new(101),
        key: b"nohmac_key_2".to_vec(),
        value: b"nohmac_val_2".to_vec(),
    };
    let entry2 = WalEntry::try_new(op2, 2, &zero_key, entry1.checksum).expect("entry 2");

    let mut wal_bytes = Vec::new();
    wal_bytes.extend_from_slice(&WAL_V3_HEADER);
    wal_bytes.extend_from_slice(&entry1.to_bytes().expect("bytes 1"));
    wal_bytes.extend_from_slice(&entry2.to_bytes().expect("bytes 2"));

    fs::write(&wal_path, &wal_bytes).await.expect("write wal");

    // Open WAL with zero integrity key to simulate WalNoHmac replay
    let key_path = dir.path().join(".wal_integrity_key");
    fs::write(&key_path, zero_key)
        .await
        .expect("write zero key");

    let wal = Wal::open(&wal_path).await.expect("open wal");
    let replayed = wal.replay().await.expect("replay wal");

    assert_eq!(replayed.len(), 2);
    assert_eq!(replayed[0].1.seq_no, 1);
    assert_eq!(replayed[1].1.seq_no, 2);

    // Corrupt single entry payload bit
    let mut corrupted_bytes = wal_bytes.clone();
    if corrupted_bytes.len() > 16 {
        corrupted_bytes[16] ^= 0xFF;
    }
    fs::write(&wal_path, &corrupted_bytes)
        .await
        .expect("write corrupted wal");

    let reopen_res = Wal::open(&wal_path).await;
    assert!(
        reopen_res.is_err(),
        "Corrupted entry in WalNoHmac mode must be rejected during replay"
    );
}
