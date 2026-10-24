use contextra_core::{ContextraError, TxId};
use contextra_store::wal::{KeyManager, Wal, WalEntry, WalOp, MAX_WAL_ENTRY_SIZE};
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn test_batch_exceeding_64mb_encrypted_succeeds_and_replays() {
    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("large_batch.wal");

    let km = Arc::new(
        KeyManager::try_new("passphrase123", b"salt123456789012345678901234567890").expect("km"),
    );

    let wal = Wal::open_with_key_manager(&wal_path, Some(km.clone()))
        .await
        .expect("open wal");

    // Construct 2 entries of 35 MiB each = 70 MiB batch total (> 64 MiB limit)
    let payload_35mb = vec![0xA5u8; 35 * 1024 * 1024];
    let ops = vec![
        (
            WalOp::Put {
                tx_id: TxId::new(1),
                key: b"large_key_1".to_vec(),
                value: payload_35mb.clone(),
            },
            1,
        ),
        (
            WalOp::Put {
                tx_id: TxId::new(2),
                key: b"large_key_2".to_vec(),
                value: payload_35mb,
            },
            2,
        ),
    ];

    let (batch, _) = wal.prepare_batch(ops).await.expect("prepare batch");
    wal.append_batch(batch)
        .await
        .expect("append_batch for large batch must succeed by greedy chunk splitting");

    // Reopen and replay
    let wal_reopen = Wal::open_with_key_manager(&wal_path, Some(km))
        .await
        .expect("reopen wal");
    let replayed = wal_reopen
        .replay()
        .await
        .expect("replay must succeed for multi-frame batch");

    assert_eq!(replayed.len(), 2);
    assert_eq!(replayed[0].1.seq_no, 1);
    assert_eq!(replayed[1].1.seq_no, 2);
}

#[tokio::test]
async fn test_single_entry_exceeding_64mb_returns_err_before_io() {
    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("oversized_single.wal");

    let km = Arc::new(
        KeyManager::try_new("passphrase123", b"salt123456789012345678901234567890").expect("km"),
    );

    let wal = Wal::open_with_key_manager(&wal_path, Some(km))
        .await
        .expect("open wal");

    let initial_file_len = tokio::fs::metadata(&wal_path).await.expect("meta").len();

    // Single entry of 65 MiB > MAX_WAL_ENTRY_SIZE
    let payload_65mb = vec![0xBBu8; (MAX_WAL_ENTRY_SIZE as usize) + 1_048_576];
    let ops = vec![(
        WalOp::Put {
            tx_id: TxId::new(1),
            key: b"huge_key".to_vec(),
            value: payload_65mb,
        },
        1,
    )];

    let prepare_res = wal.prepare_batch(ops).await;
    if let Ok((batch, _)) = prepare_res {
        let append_res = wal.append_batch(batch).await;
        assert!(
            append_res.is_err(),
            "append_batch must fail for single entry > 64 MiB"
        );
    }

    let final_file_len = tokio::fs::metadata(&wal_path).await.expect("meta").len();

    assert_eq!(
        initial_file_len, final_file_len,
        "Physical WAL file must not be modified when oversized entry append fails"
    );
}

#[test]
fn test_trailing_bytes_in_entry_rejected() {
    let op = WalOp::TxEnd {
        tx_id: TxId::new(10),
        committed: true,
    };
    let dummy_key = b"test-integrity-key-32-bytes-long!";
    let entry = WalEntry::try_new(op, 1, dummy_key, [0u8; 32]).expect("try_new");
    let bytes = entry.to_bytes().expect("to_bytes");

    // Append trailing byte inside payload (and update length and CRC)
    // entry layout: [len: 4][crc: 4][payload...]
    let mut payload = bytes[8..].to_vec();
    payload.push(0xFF); // Trailing byte after valid TxEnd op body

    let new_crc = crc32fast::hash(&payload);
    let mut corrupted = Vec::new();
    corrupted.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    corrupted.extend_from_slice(&new_crc.to_le_bytes());
    corrupted.extend_from_slice(&payload);

    let res = WalEntry::from_bytes(&corrupted[4..]);
    assert!(
        res.is_err(),
        "from_bytes must reject entry with trailing bytes"
    );
}

#[test]
fn test_invalid_committed_byte_rejected() {
    let mut payload = vec![0u8; 73 + 9]; // seq_no(8) + hmac(32) + prev(32) + op_type(1) + tx_id(8) + committed(1)
    payload[72] = 2; // op_type = TxEnd
    payload[73 + 8] = 2; // committed = 2 (invalid, must be 0 or 1)

    let crc = crc32fast::hash(&payload);
    let mut data = Vec::new();
    data.extend_from_slice(&crc.to_le_bytes());
    data.extend_from_slice(&payload);

    let res = WalEntry::from_bytes(&data);
    assert!(
        res.is_err(),
        "from_bytes must reject TxEnd with committed byte != 0 and != 1"
    );
}

#[tokio::test]
async fn test_authenticated_data_parse_error_is_corruption_not_tail() {
    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("auth_corrupt.wal");

    let km = Arc::new(
        KeyManager::try_new("passphrase123", b"salt123456789012345678901234567890").expect("km"),
    );

    // Open WAL once to initialize UUID file and header
    {
        let wal = Wal::open_with_key_manager(&wal_path, Some(km.clone()))
            .await
            .expect("open wal");
        let op = WalOp::Put {
            tx_id: TxId::new(1),
            key: b"k1".to_vec(),
            value: b"v1".to_vec(),
        };
        let (batch, _) = wal.prepare_batch(vec![(op, 1)]).await.expect("prepare");
        wal.append_batch(batch).await.expect("append");
    }

    let uuid_path = dir.path().join("auth_corrupt.wal.uuid");
    let uuid_bytes = tokio::fs::read(&uuid_path).await.expect("read uuid");
    let sub_km = km.derive_file_key(&uuid_bytes).expect("derive sub key");

    // Construct an inner plaintext with invalid op_type = 255
    let mut inner_invalid_entry = vec![0u8; 80];
    inner_invalid_entry[72] = 255; // Invalid op_type
    let crc = crc32fast::hash(&inner_invalid_entry);
    let mut inner_entry_bytes = Vec::new();
    inner_entry_bytes.extend_from_slice(&crc.to_le_bytes());
    inner_entry_bytes.extend_from_slice(&inner_invalid_entry);

    let mut inner_plaintext = Vec::new();
    inner_plaintext.extend_from_slice(&(inner_entry_bytes.len() as u32).to_le_bytes());
    inner_plaintext.extend_from_slice(&inner_entry_bytes);

    let (encrypted, nonce) = sub_km
        .encrypt_auto_nonce(&inner_plaintext)
        .expect("encrypt");

    let mut file_bytes = Vec::new();
    file_bytes.extend_from_slice(b"MFW3");
    let chunk_len = (12 + encrypted.len()) as u32;
    file_bytes.extend_from_slice(&chunk_len.to_le_bytes());
    file_bytes.extend_from_slice(&nonce);
    file_bytes.extend_from_slice(&encrypted);

    tokio::fs::write(&wal_path, &file_bytes)
        .await
        .expect("write wal");

    let wal_open = Wal::open_with_key_manager(&wal_path, Some(km)).await;
    let err = match wal_open {
        Ok(wal) => wal.replay().await.unwrap_err(),
        Err(e) => e,
    };

    assert!(
        matches!(err, ContextraError::WalCorruption { .. }),
        "Authenticated data parse error MUST return WalCorruption, got: {:?}",
        err
    );
}
