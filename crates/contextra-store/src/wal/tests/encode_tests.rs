use super::*;
use contextra_core::TxId;
use contextra_crypto::crypto::KeyManager;
use std::sync::Arc;
use tempfile::tempdir;
use tokio::fs;

#[test]
fn test_wal_entry_serialization_roundtrip() {
    let op = WalOp::Put {
        tx_id: TxId::new(42),
        key: b"key".to_vec(),
        value: b"value".to_vec(),
    };
    let dummy_key = b"test-integrity-key-32-bytes-long!";
    let entry = WalEntry::try_new(op, 100, dummy_key, [0u8; 32]).expect("try_new"); // expect
    let bytes = entry.to_bytes().expect("serialization failed"); // expect

    // 4 (len) + 4 (crc) + 8 (seq) + 32 (hmac) + 32 (prev) + 1 (op) + 8 (tx) + 4 (klen) + 3 (k) + 4 (vlen) + 5 (v) = 105
    assert_eq!(bytes.len(), 105);
    let total_payload_size = u32::from_le_bytes(bytes[0..4].try_into().expect("valid slice")); // expect
    assert_eq!(total_payload_size, 101); // 4 (crc) + 97 (payload)
}

#[test]
fn test_wal_entry_crc_corruption_detected() {
    let op = WalOp::Put {
        tx_id: TxId::new(1),
        key: b"key".to_vec(),
        value: b"value".to_vec(),
    };
    let dummy_key = b"test-integrity-key-32-bytes-long!";
    let entry = WalEntry::try_new(op, 1, dummy_key, [0u8; 32]).expect("try_new"); // expect

    let mut bytes = entry.to_bytes().expect("serialization failed"); // expect

    // Let's corrupt the payload which is after the length prefix(4) and CRC(4)
    if bytes.len() > 10 {
        bytes[10] ^= 0xFF;
    }

    // Check using from_bytes (skipping the length prefix at the start)
    let result = WalEntry::from_bytes(&bytes[4..]);
    assert!(result.is_err(), "Corruption must be detected by CRC check");
    let err = result.unwrap_err();
    assert!(format!("{}", err).contains("CRC mismatch"));
}

#[test]
fn test_wal_entry_header_fuzzing() {
    let op = WalOp::Put {
        tx_id: TxId::new(1),
        key: b"k".to_vec(),
        value: b"v".to_vec(),
    };
    let integrity_key = b"test-integrity-key-32-bytes-long!";
    let entry = WalEntry::try_new(op, 12345, integrity_key, [0u8; 32]).expect("try_new"); // expect

    let original_bytes = entry.to_bytes().expect("serialization failed"); // expect

    // Systematisch jedes Bit der ersten 12 Bytes flippen
    for byte_idx in 0..12 {
        for bit_idx in 0..8 {
            let mut corrupted_bytes = original_bytes.clone();
            corrupted_bytes[byte_idx] ^= 1 << bit_idx;

            // Testverhalten unterscheidet sich je nach Position
            if byte_idx < 4 {
                // Length prefix corrupted.
                // Das wird normalerweise von Wal::replay abgefangen,
                // aber from_bytes kriegt hier nur den Teil ab Index 4.
                // Wenn wir bytes[0..4] flippen, ändert das für from_bytes(&bytes[4..]) nichts.
                let result = WalEntry::from_bytes(&corrupted_bytes[4..]);
                assert!(
                    result.is_ok(),
                    "Flipping bytes[0..4] should not affect from_bytes(bytes[4..])"
                );
            } else {
                // CRC (4-7) oder SeqNo (8-11) korrumpiert.
                // Das MUSS von from_bytes erkannt werden.
                let result = WalEntry::from_bytes(&corrupted_bytes[4..]);
                assert!(
                    result.is_err(),
                    "Corruption at byte {}, bit {} was NOT detected! result: {:?}",
                    byte_idx,
                    bit_idx,
                    result
                );
            }
        }
    }
}

#[test]
fn test_wal_entry_crc_roundtrip() {
    let op = WalOp::Put {
        tx_id: TxId::new(42),
        key: b"test_key".to_vec(),
        value: b"test_value".to_vec(),
    };
    let dummy_key = b"test-integrity-key-32-bytes-long!";
    let entry = WalEntry::try_new(op, 100, dummy_key, [0u8; 32]).expect("try_new"); // expect

    let bytes = entry.to_bytes().expect("serialization failed"); // expect
    let decoded = WalEntry::from_bytes(&bytes[4..]).expect("Roundtrip must work"); // expect

    assert_eq!(decoded.seq_no, 100);
    if let WalOp::Put { key, value, .. } = decoded.op {
        assert_eq!(key, b"test_key");
        assert_eq!(value, b"test_value");
    } else {
        panic!("Wrong op type");
    }
}

#[tokio::test]
async fn test_batch_encryption_single_nonce_layout() {
    let dir = tempdir().expect("tempdir"); // expect
    let wal_path = dir.path().join("single_nonce_test.wal");

    let km = Arc::new(
        KeyManager::try_new("test_passphrase", b"salt123456789012345678901234567890").expect("km"), // expect
    );
    let wal = Wal::open_with_key_manager(&wal_path, Some(km))
        .await
        .expect("open wal"); // expect

    let ops = vec![
        (
            WalOp::Put {
                tx_id: TxId::new(1),
                key: b"k1".to_vec(),
                value: b"v1".to_vec(),
            },
            100,
        ),
        (
            WalOp::Put {
                tx_id: TxId::new(1),
                key: b"k2".to_vec(),
                value: b"v2".to_vec(),
            },
            101,
        ),
        (
            WalOp::Put {
                tx_id: TxId::new(1),
                key: b"k3".to_vec(),
                value: b"v3".to_vec(),
            },
            102,
        ),
    ];

    let (batch, _) = wal.prepare_batch(ops).await.expect("prepare batch"); // expect
    assert_eq!(batch.len(), 3);

    wal.append_batch(batch).await.expect("append batch"); // expect

    let file_bytes = fs::read(&wal_path).await.expect("read wal file"); // expect

    // Layout:
    // Offset 0..4: WAL_V3_HEADER (b"MFW3")
    // Offset 4..8: batch chunk_len (u32 LE)
    // Offset 8..20: single 12-byte nonce
    // Offset 20..: AES-GCM-SIV ciphertext
    assert_eq!(&file_bytes[0..4], &WAL_V3_HEADER);
    let chunk_len = u32::from_le_bytes(file_bytes[4..8].try_into().unwrap()) as usize; // unwrap
    assert_eq!(file_bytes.len(), 4 + 4 + chunk_len);

    // Verify there is exactly one batch chunk header (12-byte nonce) in the file for N=3 entries
    let nonce_bytes = &file_bytes[8..20];
    assert_eq!(nonce_bytes.len(), 12);
}

#[test]
fn test_wal_op_from_bytes_oversized_key_val() {
    // Construct payload with key_len > 1MB
    let mut payload = vec![0u8; 90];
    // op_type = 0 (Put) at index 72
    payload[72] = 0;
    // tx_id = 1
    payload[73..81].copy_from_slice(&1u64.to_le_bytes());
    // key_len = 2 MB
    payload[81..85].copy_from_slice(&(2 * 1024 * 1024u32).to_le_bytes());

    let crc = crc32fast::hash(&payload);
    let mut data = vec![0u8; 4];
    data[0..4].copy_from_slice(&crc.to_le_bytes());
    data.extend_from_slice(&payload);

    let res = WalEntry::from_bytes(&data);
    assert!(res.is_err());
    if let Err(ContextraError::Serialization(msg)) = res {
        assert!(msg.contains("key_len exceeds 1 MiB limit"));
    } else {
        panic!("Expected Serialization error for key_len limit");
    }
}

#[test]
fn test_wal_entry_from_bytes_invalid_cases() {
    // 1. Too short data (< 94 bytes)
    let short_data = vec![0u8; 50];
    let res = WalEntry::from_bytes(&short_data);
    assert!(matches!(res, Err(ContextraError::Serialization(_))));

    // 2. Invalid WalOp tag (e.g., tag = 255)
    let mut payload = vec![0u8; 90];
    payload[72] = 255; // Invalid tag
    let crc = crc32fast::hash(&payload);
    let mut data = vec![0u8; 4];
    data[0..4].copy_from_slice(&crc.to_le_bytes());
    data.extend_from_slice(&payload);

    let res_op = WalEntry::from_bytes(&data);
    assert!(matches!(res_op, Err(ContextraError::Serialization(_))));
}
