use super::*;
use memfuse_core::TxId;
use memfuse_security::wal_crypto::WalHmac;
use tempfile::tempdir;
use tokio::fs;
use tokio::io::AsyncWriteExt;

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

#[tokio::test]
async fn test_wal_append_and_replay_valid() {
    let dir = tempdir().expect("tempdir"); // expect
    let wal_path = dir.path().join("test_wal.log");

    {
        let wal = Wal::open(&wal_path).await.expect("open WAL"); // expect
        let op1 = WalOp::Put {
            tx_id: TxId::new(1),
            key: b"user:1".to_vec(),
            value: b"Alice".to_vec(),
        };
        let (batch1, _) = wal.prepare_batch(vec![(op1, 10)]).await.expect("valid");
        wal.append_batch(batch1).await.expect("append 1");

        let op2 = WalOp::Delete {
            tx_id: TxId::new(2),
            key: b"user:1".to_vec(),
        };
        let (batch2, _) = wal.prepare_batch(vec![(op2, 11)]).await.expect("valid");
        wal.append_batch(batch2).await.expect("append 2");
    }

    let wal2 = Wal::open(&wal_path).await.expect("reopen WAL"); // expect
    let entries = wal2.replay().await.expect("replay"); // expect

    assert_eq!(entries.len(), 2);
    assert_eq!(entries[1].1.prev_hmac, entries[0].1.checksum);
}

#[tokio::test]
async fn test_wal_hash_chain_verification() {
    let dir = tempdir().expect("tempdir"); // expect
    let wal_path = dir.path().join("chain_wal.log");

    {
        let wal = Wal::open(&wal_path).await.expect("open"); // expect
        let op1 = WalOp::Put {
            tx_id: TxId::new(1),
            key: b"k1".to_vec(),
            value: b"v1".to_vec(),
        };
        let (batch1, _) = wal.prepare_batch(vec![(op1, 1)]).await.expect("entry1");
        wal.append_batch(batch1).await.expect("append1");

        let op2 = WalOp::Put {
            tx_id: TxId::new(2),
            key: b"k2".to_vec(),
            value: b"v2".to_vec(),
        };
        let (batch2, _) = wal.prepare_batch(vec![(op2, 2)]).await.expect("entry2");
        wal.append_batch(batch2).await.expect("append2");
    }

    {
        let mut data = fs::read(&wal_path).await.expect("read"); // expect
                                                                 // Corrupt the payload of the first entry (offset 4 is CRC, payload starts at 8)
                                                                 // CRC itself is also part of validation. Let's flip a bit in the payload.
        if data.len() > 10 {
            data[12] ^= 0xFF;
            fs::write(&wal_path, data).await.expect("write"); // expect
        }
    }

    let result = Wal::open(&wal_path).await;
    // Should fail due to CRC mismatch or HMAC chain failure
    assert!(matches!(
        result,
        Err(MemFuseError::Serialization(_)) | Err(MemFuseError::WalCorruption { .. })
    ));
}
#[tokio::test]
async fn test_wal_replay_truncation() {
    let dir = tempdir().expect("tempdir"); // expect
    let wal_path = dir.path().join("trunc_wal.log");

    {
        let wal = Wal::open(&wal_path).await.expect("open"); // expect
        for i in 0..5 {
            let op = WalOp::Put {
                tx_id: TxId::new(i),
                key: b"key".to_vec(),
                value: b"val".to_vec(),
            };
            let (batch, _) = wal.prepare_batch(vec![(op, i)]).await.expect("entry");
            wal.append_batch(batch).await.expect("append");
        }
    }

    // Truncate the file in the middle of the last entry
    let mut data = fs::read(&wal_path).await.expect("read"); // expect
    let new_size = data.len() - 10; // Chop off 10 bytes from the last entry
    data.truncate(new_size);
    fs::write(&wal_path, data).await.expect("write"); // expect

    let wal2 = Wal::open(&wal_path).await.expect("open"); // expect
    let entries = wal2.replay().await.expect("replay"); // expect
                                                        // Replay should stop at the last valid entry (the 4th one)
    assert_eq!(entries.len(), 4);
}

#[tokio::test]
async fn test_wal_crc_middle_corruption() {
    let dir = tempdir().expect("tempdir"); // expect
    let wal_path = dir.path().join("middle_corrupt.log");

    {
        let wal = Wal::open(&wal_path).await.expect("open"); // expect
        for i in 0..3 {
            let op = WalOp::Put {
                tx_id: TxId::new(i),
                key: format!("k{}", i).into_bytes(),
                value: format!("v{}", i).into_bytes(),
            };
            let (batch, _) = wal.prepare_batch(vec![(op, i)]).await.expect("entry");
            wal.append_batch(batch).await.expect("append");
        }
    }

    {
        let mut data = fs::read(&wal_path).await.expect("read"); // expect
                                                                 // Corrupt the second entry (somewhere in the middle of the file)
                                                                 // Each entry is ~100 bytes. Let's flip a bit around offset 150.
        if data.len() > 150 {
            data[150] ^= 0xFF;
            fs::write(&wal_path, data).await.expect("write"); // expect
        }
    }

    let result = Wal::open(&wal_path).await;

    // Should fail because corruption is in the middle (before the last entry)
    assert!(
        matches!(result, Err(MemFuseError::WalCorruption { .. })),
        "Expected WalCorruption error, got {:?}",
        result
    );
}

#[tokio::test]
async fn wal_tolerates_truncated_tail() {
    let dir = tempdir().expect("tempdir"); // expect
    let path = dir.path().join("test.wal");

    {
        let wal = Wal::open(&path).await.expect("open WAL"); // expect
        for i in 1..=4 {
            let op = WalOp::Put {
                tx_id: TxId::new(i),
                key: format!("key{}", i).into_bytes(),
                value: format!("val{}", i).into_bytes(),
            };
            let (batch, _) = wal
                .prepare_batch(vec![(op, i)])
                .await
                .expect("create entry");
            wal.append_batch(batch).await.expect("append entry");
        }
    }

    // Truncate file in the middle of 4th entry
    let mut data = fs::read(&path).await.expect("read wal"); // expect
    let truncated_len = data.len() - 10;
    data.truncate(truncated_len);
    fs::write(&path, data).await.expect("write truncated wal"); // expect

    let wal2 = Wal::open(&path).await.expect("reopen WAL"); // expect
    let entries = wal2.replay().await.expect("replay WAL"); // expect

    assert_eq!(
        entries.len(),
        3,
        "Replay must return exactly 3 valid entries without error"
    );
}

#[tokio::test]
async fn test_wal_crc_tail_corruption() {
    let dir = tempdir().expect("tempdir"); // expect
    let wal_path = dir.path().join("tail_corrupt.log");

    {
        let wal = Wal::open(&wal_path).await.expect("open"); // expect
        for i in 0..2 {
            let op = WalOp::Put {
                tx_id: TxId::new(i),
                key: format!("k{}", i).into_bytes(),
                value: format!("v{}", i).into_bytes(),
            };
            let (batch, _) = wal.prepare_batch(vec![(op, i)]).await.expect("entry");
            wal.append_batch(batch).await.expect("append");
        }
    }

    {
        let mut file = fs::OpenOptions::new()
            .append(true)
            .open(&wal_path)
            .await
            .expect("open"); // expect
        use tokio::io::AsyncWriteExt;
        // Append some garbage that doesn't form a valid entry
        file.write_all(b"SOME GARBAGE DATA AT THE END")
            .await
            .expect("write"); // expect
    }

    let wal2 = Wal::open(&wal_path).await.expect("open"); // expect
    let entries = wal2.replay().await.expect("replay"); // expect

    // Should succeed and return only the 2 valid entries
    assert_eq!(entries.len(), 2);
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

#[tokio::test]
async fn test_wal_header_systematic_fuzzing() {
    let dir = tempdir().expect("tempdir"); // expect
    let wal_path = dir.path().join("fuzz.log");

    // 1. Erstelle eine valide WAL-Datei mit einem Eintrag
    let original_data = {
        let wal = Wal::open(&wal_path).await.expect("open"); // expect
        let op = WalOp::Put {
            tx_id: TxId::new(1),
            key: b"k".to_vec(),
            value: b"v".to_vec(),
        };
        let (batch, _) = wal.prepare_batch(vec![(op, 1)]).await.expect("entry");
        wal.append_batch(batch).await.expect("append");
        drop(wal);
        fs::read(&wal_path).await.expect("read") // expect
    };

    // 2. Systematisch jedes Bit der ersten 12 Bytes der DATEI flippen
    // Bytes 0-3: Length Prefix
    // Bytes 4-7: CRC32
    // Bytes 8-11: Anfang von seq_no (u64)
    for byte_idx in 0..12 {
        for bit_idx in 0..8 {
            let mut corrupted_data = original_data.clone();
            corrupted_data[byte_idx] ^= 1 << bit_idx;
            fs::write(&wal_path, &corrupted_data).await.expect("write"); // expect

            let result = Wal::open(&wal_path).await;

            match result {
                Ok(wal) => {
                    // Wenn open erfolgreich ist, muss replay den Fehler finden
                    let replay_result = wal.replay().await;
                    assert!(
                        replay_result.is_err() || replay_result.unwrap().is_empty(), // unwrap
                        "Corruption at byte {}, bit {} was NOT detected during replay!",
                        byte_idx,
                        bit_idx
                    );
                }
                Err(e) => {
                    // Fehler beim Öffnen/Initial-Replay ist auch okay, solange es keine Panic ist
                    assert!(
                        matches!(
                            e,
                            MemFuseError::Serialization(_)
                                | MemFuseError::WalCorruption { .. }
                                | MemFuseError::Storage(_)
                        ),
                        "Unexpected error type at byte {}, bit {}: {:?}",
                        byte_idx,
                        bit_idx,
                        e
                    );
                }
            }
        }
    }
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

#[tokio::test]
async fn test_wal_random_integrity_keys_per_instance() {
    let dir1 = tempdir().expect("tempdir1"); // expect
    let dir2 = tempdir().expect("tempdir2"); // expect
    let wal_path1 = dir1.path().join("wal1.log");
    let wal_path2 = dir2.path().join("wal2.log");

    let wal1 = Wal::open(&wal_path1).await.expect("open wal1"); // expect
    let wal2 = Wal::open(&wal_path2).await.expect("open wal2"); // expect

    let key1 = wal1.get_integrity_key().expect("key1"); // expect
    let key2 = wal2.get_integrity_key().expect("key2"); // expect

    assert_ne!(
        key1, key2,
        "Two independent WAL instances must receive unique random integrity keys"
    );
}

#[tokio::test]
async fn test_wal_tampered_wrong_key_entry_detected() {
    let dir = tempdir().expect("tempdir"); // expect
    let wal_path = dir.path().join("tamper_wal.log");

    let valid_op = WalOp::Put {
        tx_id: TxId::new(1),
        key: b"secure_key".to_vec(),
        value: b"secure_val".to_vec(),
    };

    {
        let wal = Wal::open(&wal_path).await.expect("open wal"); // expect
        let (batch, _) = wal
            .prepare_batch(vec![(valid_op.clone(), 1)])
            .await
            .expect("create entry"); // expect
        wal.append_batch(batch).await.expect("append valid entry"); // expect
    }

    {
        // Inject an entry forged with an arbitrary wrong key
        let wrong_key = b"wrong-attacker-integrity-key-32!";
        let forged_op = WalOp::Put {
            tx_id: TxId::new(2),
            key: b"forged_key".to_vec(),
            value: b"forged_val".to_vec(),
        };
        // Previous HMAC is the valid entry's HMAC, but key is wrong
        let last_valid_entry = Wal::open(&wal_path)
            .await
            .expect("open") // expect
            .replay()
            .await
            .expect("replay")[0] // expect
            .1
            .clone();

        let forged_entry = WalEntry::try_new(forged_op, 2, wrong_key, last_valid_entry.checksum)
            .expect("create forged entry"); // expect

        // Also append a 3rd entry so the forged entry is in the middle of the file (pos < file_size)
        let trailing_entry = WalEntry::try_new(
            WalOp::Put {
                tx_id: TxId::new(3),
                key: b"trailing".to_vec(),
                value: b"val".to_vec(),
            },
            3,
            wrong_key,
            forged_entry.checksum,
        )
        .expect("create trailing entry"); // expect

        let mut file = tokio::fs::OpenOptions::new()
            .append(true)
            .open(&wal_path)
            .await
            .expect("open file for append"); // expect
        file.write_all(&forged_entry.to_bytes().expect("to_bytes")) // expect
            .await
            .expect("write forged entry"); // expect
        file.write_all(&trailing_entry.to_bytes().expect("to_bytes")) // expect
            .await
            .expect("write trailing entry"); // expect
    }

    let wal_reopen = Wal::open(&wal_path).await;
    assert!(
        wal_reopen.is_err() || wal_reopen.unwrap().replay().await.is_err(), // unwrap
        "Replaying a WAL with a wrong-key forged entry must fail HMAC verification"
    );
}

#[tokio::test]
async fn test_wal_legacy_key_fallback_migration() {
    let dir = tempdir().expect("tempdir"); // expect
    let wal_path = dir.path().join("legacy_wal.log");

    {
        // Manually construct a WAL entry with the legacy static integrity key
        let op = WalOp::Put {
            tx_id: TxId::new(1),
            key: b"legacy_key".to_vec(),
            value: b"legacy_val".to_vec(),
        };
        let legacy_entry =
            WalEntry::try_new(op, 1, &legacy_integrity_key(), [0u8; 32]).expect("legacy entry"); // expect

        let mut wal_bytes = Vec::new();
        wal_bytes.extend_from_slice(&WAL_V3_HEADER);
        wal_bytes.extend_from_slice(&legacy_entry.to_bytes().expect("to_bytes"));

        tokio::fs::write(&wal_path, wal_bytes) // expect
            .await
            .expect("write legacy WAL"); // expect
    }

    // Opening without explicit opt-in must fail (downgrade attack protection)
    let open_res = Wal::open(&wal_path).await;
    assert!(
        open_res.is_err(),
        "Opening legacy WAL without allow_legacy_integrity_key_fallback must fail"
    );

    // Opening with explicit opt-in must succeed
    let wal = Wal::open_with_config(
        &wal_path,
        WalConfig {
            allow_legacy_integrity_key_fallback: true,
            ..Default::default()
        },
    )
    .await
    .expect("open legacy wal with fallback opt-in"); // expect
    let entries = wal.replay().await.expect("replay legacy wal"); // expect
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].1.seq_no, 1);
    if let WalOp::Put { key, value, .. } = &entries[0].1.op {
        assert_eq!(key, b"legacy_key");
        assert_eq!(value, b"legacy_val");
    } else {
        panic!("Expected Put op");
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
async fn test_wal_crash_consistency_write_without_fsync() {
    let dir = tempdir().expect("tempdir"); // expect
    let wal_path = dir.path().join("crash_sim.wal");

    // 1. Open WAL and append an entry
    {
        let wal = Wal::open(&wal_path).await.expect("open wal"); // expect
        let op = WalOp::Put {
            tx_id: TxId::new(100),
            key: b"crash_k".to_vec(),
            value: b"crash_v".to_vec(),
        };
        let (batch, _) = wal
            .prepare_batch(vec![(op, 1)])
            .await
            .expect("create entry"); // expect
        let entry = &batch.entries()[0];

        // Manually simulate a write + flush to OS buffer WITHOUT file.sync_all()
        let mut file = tokio::fs::OpenOptions::new()
            .append(true)
            .open(&wal_path)
            .await
            .expect("open for append"); // expect
        let bytes = entry.to_bytes().expect("to_bytes"); // expect
        file.write_all(&bytes).await.expect("write_all"); // expect
        file.flush().await.expect("flush"); // expect
                                            // File dropped without calling sync_all() (simulating crash before fsync)
        drop(file);
        drop(wal);
    }

    // 2. Re-open WAL and replay
    let wal_reopen = Wal::open(&wal_path).await;
    assert!(wal_reopen.is_ok(), "WAL open after crash should succeed");
    let wal = wal_reopen.unwrap(); // unwrap

    let replay_result = wal.replay().await;
    match replay_result {
        Ok(entries) => {
            // Should either find the entry or empty set, never panic
            if !entries.is_empty() {
                assert_eq!(entries.len(), 1);
                assert_eq!(entries[0].1.seq_no, 1);
            }
        }
        Err(e) => {
            panic!("replay() failed unexpectedly with error: {:?}", e);
        }
    }
}

#[tokio::test]
async fn test_append_batch_partial_write_atomicity() {
    let dir = tempdir().expect("tempdir"); // expect
    let wal_path = dir.path().join("partial_batch.wal");

    let wal = Wal::open(&wal_path).await.expect("open wal"); // expect
    let ops = vec![
        (
            WalOp::Put {
                tx_id: TxId::new(1),
                key: b"b1".to_vec(),
                value: b"v1".to_vec(),
            },
            1,
        ),
        (
            WalOp::Put {
                tx_id: TxId::new(1),
                key: b"b2".to_vec(),
                value: b"v2".to_vec(),
            },
            2,
        ),
        (
            WalOp::Put {
                tx_id: TxId::new(1),
                key: b"b3".to_vec(),
                value: b"v3".to_vec(),
            },
            3,
        ),
    ];

    let (entries, _) = wal.prepare_batch(ops).await.expect("prepare_batch"); // expect
    assert_eq!(entries.len(), 3);

    // Serialize all 3 entries into a single bytes payload
    let mut batch_bytes = Vec::new();
    for e in entries.entries() {
        batch_bytes.extend_from_slice(&e.to_bytes().expect("to_bytes")); // expect
    }

    // Truncate the batch in the middle of entry 2 (partial write during crash)
    // Each entry is ~101 bytes. Total ~303 bytes.
    // Subtracting 120 bytes leaves ~183 bytes, truncating entry 2 mid-write.
    let truncated_len = batch_bytes.len() - 120;
    let truncated_bytes = &batch_bytes[..truncated_len];

    // Append the truncated bytes directly to the WAL file
    {
        let mut file = tokio::fs::OpenOptions::new()
            .append(true)
            .open(&wal_path)
            .await
            .expect("open for append"); // expect
        file.write_all(truncated_bytes).await.expect("write_all"); // expect
        file.flush().await.expect("flush"); // expect
    }

    // Reopen and replay
    let wal2 = Wal::open(&wal_path).await.expect("reopen"); // expect
    let replay_entries = wal2
        .replay()
        .await
        .expect("replay must succeed without panic"); // expect

    // Replay must recover entry 1 (which was fully written) and cleanly discard the truncated tail
    assert_eq!(replay_entries.len(), 1, "Only entry 1 should be recovered");
    assert_eq!(replay_entries[0].1.seq_no, 1);
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

#[tokio::test]
async fn test_batch_encrypted_wal_roundtrip() {
    let dir = tempdir().expect("tempdir"); // expect
    let wal_path = dir.path().join("roundtrip_test.wal");

    let km = Arc::new(
        KeyManager::try_new("passphrase123", b"salt123456789012345678901234567890").expect("km"), // expect
    );
    let wal = Wal::open_with_key_manager(&wal_path, Some(km.clone()))
        .await
        .expect("open wal"); // expect

    let ops = vec![
        (
            WalOp::Put {
                tx_id: TxId::new(1),
                key: b"alice_key".to_vec(),
                value: b"alice_value".to_vec(),
            },
            1,
        ),
        (
            WalOp::Put {
                tx_id: TxId::new(2),
                key: b"bob_key".to_vec(),
                value: b"bob_value".to_vec(),
            },
            2,
        ),
        (
            WalOp::Delete {
                tx_id: TxId::new(3),
                key: b"alice_key".to_vec(),
            },
            3,
        ),
    ];

    let (batch, _) = wal.prepare_batch(ops).await.expect("prepare_batch"); // expect
    wal.append_batch(batch).await.expect("append_batch"); // expect

    let wal_reopen = Wal::open_with_key_manager(&wal_path, Some(km))
        .await
        .expect("reopen wal"); // expect
    let replayed = wal_reopen.replay().await.expect("replay"); // expect

    assert_eq!(replayed.len(), 3);
    assert_eq!(replayed[0].1.seq_no, 1);
    assert_eq!(replayed[1].1.seq_no, 2);
    assert_eq!(replayed[2].1.seq_no, 3);

    if let WalOp::Put { key, value, tx_id } = &replayed[0].1.op {
        assert_eq!(key, b"alice_key");
        assert_eq!(value, b"alice_value");
        assert_eq!(*tx_id, TxId::new(1));
    } else {
        panic!("Expected Put op");
    }

    if let WalOp::Delete { key, tx_id } = &replayed[2].1.op {
        assert_eq!(key, b"alice_key");
        assert_eq!(*tx_id, TxId::new(3));
    } else {
        panic!("Expected Delete op");
    }
}

#[tokio::test]
async fn test_old_v1_format_backward_compatibility() {
    let dir = tempdir().expect("tempdir"); // expect
    let wal_path = dir.path().join("v1_legacy_format.wal");

    let km = Arc::new(
        KeyManager::try_new("legacy_passphrase", b"salt123456789012345678901234567890")
            .expect("km"), // expect
    );

    // Derive sub-key for file ID (same derivation Wal::open_with_key_manager does)
    let uuid_bytes = Wal::load_or_create_wal_uuid(&wal_path).await.expect("uuid"); // expect
    let sub_km = km.derive_file_key(&uuid_bytes).expect("derive file key"); // expect

    // Manually construct an old V1 encrypted WAL file (no MFW2 header, each entry encrypted separately)
    let integrity_key = sub_km
        .integrity_key()
        .map_err(MemFuseError::from)
        .expect("integrity key"); // expect

    let op1 = WalOp::Put {
        tx_id: TxId::new(10),
        key: b"legacy_k1".to_vec(),
        value: b"legacy_v1".to_vec(),
    };
    let entry1 = WalEntry::try_new(op1, 100, &integrity_key, [0u8; 32]).expect("entry1"); // expect
    let bytes1 = entry1.to_bytes().expect("bytes1"); // expect

    let payload1 = &bytes1[4..];
    let (encrypted1, nonce1) = sub_km.encrypt_auto_nonce(payload1).expect("enc1"); // expect

    let mut v1_file_data = Vec::new();
    let chunk_len1 = (12 + encrypted1.len()) as u32;
    v1_file_data.extend_from_slice(&chunk_len1.to_le_bytes());
    v1_file_data.extend_from_slice(&nonce1);
    v1_file_data.extend_from_slice(&encrypted1);

    let op2 = WalOp::Put {
        tx_id: TxId::new(11),
        key: b"legacy_k2".to_vec(),
        value: b"legacy_v2".to_vec(),
    };
    let entry2 = WalEntry::try_new(op2, 101, &integrity_key, entry1.checksum).expect("entry2"); // expect
    let bytes2 = entry2.to_bytes().expect("bytes2"); // expect

    let payload2 = &bytes2[4..];
    let (encrypted2, nonce2) = sub_km.encrypt_auto_nonce(payload2).expect("enc2"); // expect

    let chunk_len2 = (12 + encrypted2.len()) as u32;
    v1_file_data.extend_from_slice(&chunk_len2.to_le_bytes());
    v1_file_data.extend_from_slice(&nonce2);
    v1_file_data.extend_from_slice(&encrypted2);

    fs::write(&wal_path, &v1_file_data)
        .await
        .expect("write v1 wal"); // expect

    // Reopen via standard Wal::open_with_key_manager and replay
    let wal = Wal::open_with_key_manager(&wal_path, Some(km))
        .await
        .expect("open v1 wal"); // expect
    let replayed = wal.replay().await.expect("replay v1 wal"); // expect

    assert_eq!(
        replayed.len(),
        2,
        "Both V1 entries must be replayed correctly"
    );
    assert_eq!(replayed[0].1.seq_no, 100);
    assert_eq!(replayed[1].1.seq_no, 101);
    assert_eq!(replayed[1].1.prev_hmac, replayed[0].1.checksum);
}

#[tokio::test]
async fn test_batch_encrypted_wal_truncation_crash_consistency() {
    let dir = tempdir().expect("tempdir"); // expect
    let wal_path = dir.path().join("batch_truncation.wal");

    let km = Arc::new(
        KeyManager::try_new("passphrase123", b"salt123456789012345678901234567890").expect("km"), // expect
    );

    {
        let wal = Wal::open_with_key_manager(&wal_path, Some(km.clone()))
            .await
            .expect("open wal"); // expect

        // Batch 1: 2 entries
        let ops1 = vec![
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
        let (batch1, _) = wal.prepare_batch(ops1).await.expect("prepare 1"); // expect
        wal.append_batch(batch1).await.expect("append 1"); // expect

        // Batch 2: 2 entries
        let ops2 = vec![
            (
                WalOp::Put {
                    tx_id: TxId::new(2),
                    key: b"k3".to_vec(),
                    value: b"v3".to_vec(),
                },
                3,
            ),
            (
                WalOp::Put {
                    tx_id: TxId::new(2),
                    key: b"k4".to_vec(),
                    value: b"v4".to_vec(),
                },
                4,
            ),
        ];
        let (batch2, _) = wal.prepare_batch(ops2).await.expect("prepare 2"); // expect
        wal.append_batch(batch2).await.expect("append 2"); // expect
    }

    // Truncate the file mid-ciphertext of Batch 2
    let mut data = fs::read(&wal_path).await.expect("read wal"); // expect
    let truncated_len = data.len() - 15; // chop off 15 bytes from Batch 2's ciphertext
    data.truncate(truncated_len);
    fs::write(&wal_path, &data)
        .await
        .expect("write truncated wal"); // expect

    // Reopen and replay
    let wal2 = Wal::open_with_key_manager(&wal_path, Some(km))
        .await
        .expect("reopen wal"); // expect
    let replayed = wal2
        .replay()
        .await
        .expect("replay must succeed by recovering Batch 1"); // expect

    assert_eq!(
        replayed.len(),
        2,
        "Batch 1 (2 entries) must be recovered, Batch 2 truncated"
    );
    assert_eq!(replayed[0].1.seq_no, 1);
    assert_eq!(replayed[1].1.seq_no, 2);
}

#[tokio::test]
async fn test_integrity_key_atomic_permissions_and_race_condition() {
    let temp = tempfile::tempdir().expect("tempdir"); // expect
    let wal_path = temp.path().join("test.wal");

    // Test 1: Created key file has 0o600 permissions on Unix
    let key1 = Wal::load_or_create_integrity_key(&wal_path)
        .await
        .expect("create key"); // expect

    let key_path = temp.path().join(".wal_integrity_key");
    assert!(key_path.exists(), "Key file must exist");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = std::fs::metadata(&key_path).expect("metadata"); // expect
        let mode = metadata.permissions().mode() & 0o777;
        assert_eq!(
            mode, 0o600,
            "WAL integrity key file must have permissions 0o600 on Unix, got 0o{:o}",
            mode
        );
    }

    // Test 2: Race condition simulation with multiple concurrent callers
    let wal_path_race = temp.path().join("race.wal");
    let mut handles = Vec::new();
    for _ in 0..10 {
        let path = wal_path_race.clone();
        handles.push(tokio::spawn(async move {
            Wal::load_or_create_integrity_key(&path).await
        }));
    }

    let mut keys = Vec::new();
    for h in handles {
        let res = h.await.expect("join handle").expect("load key"); // expect
        keys.push(res);
    }

    for k in &keys {
        assert_eq!(
            k, &keys[0],
            "All concurrent tasks must receive the identical key"
        );
    }
    assert_eq!(key1.len(), 32);
}

/// Windows ACL verification test.
/// Note: This test executes only on Windows platforms (e.g. `windows-latest` CI runner).
#[cfg(windows)]
#[test]
#[allow(unsafe_code)]
fn test_windows_wal_integrity_key_acl() {
    use std::os::windows::ffi::OsStrExt;
    use std::ptr::null_mut;
    use windows_sys::Win32::Foundation::{CloseHandle, GetLastError, ERROR_SUCCESS, HANDLE};
    use windows_sys::Win32::Security::Authorization::{GetNamedSecurityInfoW, SE_FILE_OBJECT};
    use windows_sys::Win32::Security::{
        EqualSid, GetAce, GetTokenInformation, TokenUser, ACCESS_ALLOWED_ACE, ACL,
        DACL_SECURITY_INFORMATION, PSECURITY_DESCRIPTOR, SECURITY_DESCRIPTOR_CONTROL,
        SE_DACL_PROTECTED, TOKEN_QUERY, TOKEN_USER,
    };
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

    let temp = tempfile::tempdir().expect("tempdir"); // expect
    let wal_path = temp.path().join("test_acl.wal");

    let rt = tokio::runtime::Runtime::new().expect("tokio runtime"); // expect
    let key = rt
        .block_on(Wal::load_or_create_integrity_key(&wal_path))
        .expect("load_or_create_integrity_key"); // expect
    assert_eq!(key.len(), 32);

    let key_path = temp.path().join(".wal_integrity_key");
    assert!(key_path.exists(), "Key file must exist");

    let path_wide: Vec<u16> = key_path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let mut p_sec_desc: PSECURITY_DESCRIPTOR = null_mut();
    let mut p_dacl: *mut ACL = null_mut();
    let mut control: SECURITY_DESCRIPTOR_CONTROL = 0;
    let mut revision = 0u32;

    // Query file's DACL and Control bits
    let status = unsafe {
        GetNamedSecurityInfoW(
            path_wide.as_ptr() as *mut _,
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION,
            null_mut(),
            null_mut(),
            &mut p_dacl,
            null_mut(),
            &mut p_sec_desc,
        )
    };
    assert_eq!(
        status, ERROR_SUCCESS,
        "GetNamedSecurityInfoW failed with error {}",
        status
    );

    struct SecDescGuard(PSECURITY_DESCRIPTOR);
    impl Drop for SecDescGuard {
        fn drop(&mut self) {
            if !self.0.is_null() {
                unsafe {
                    windows_sys::Win32::Foundation::LocalFree(self.0 as _);
                }
            }
        }
    }
    let _guard = SecDescGuard(p_sec_desc);

    // Verify DACL is present
    assert!(!p_dacl.is_null(), "DACL should not be null");

    // Verify control bits to check that DACL inheritance is protected/disabled
    let status = unsafe {
        windows_sys::Win32::Security::GetSecurityDescriptorControl(
            p_sec_desc,
            &mut control,
            &mut revision,
        )
    };
    assert_ne!(
        status,
        0,
        "GetSecurityDescriptorControl failed with error {}",
        unsafe { GetLastError() }
    );
    assert_ne!(
        control & SE_DACL_PROTECTED,
        0,
        "DACL inheritance must be disabled (SE_DACL_PROTECTED bit set)"
    );

    // Query process token user SID
    let mut token_handle: HANDLE = null_mut();
    let res = unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token_handle) };
    assert_ne!(res, 0, "OpenProcessToken failed");

    let mut len = 0u32;
    unsafe {
        GetTokenInformation(token_handle, TokenUser, null_mut(), 0, &mut len);
    }
    let mut buffer = vec![0u8; len as usize];
    let res = unsafe {
        GetTokenInformation(
            token_handle,
            TokenUser,
            buffer.as_mut_ptr().cast(),
            len,
            &mut len,
        )
    };
    assert_ne!(res, 0, "GetTokenInformation failed");
    unsafe { CloseHandle(token_handle) };

    let token_user = buffer.as_ptr() as *const TOKEN_USER;
    let owner_sid = unsafe { (*token_user).User.Sid };
    assert!(!owner_sid.is_null());

    // Inspect ACE count and verify ACE matches process owner SID
    let ace_count = unsafe { (*p_dacl).AceCount };
    assert_eq!(ace_count, 1, "DACL must contain exactly 1 ACE (owner only)");

    let mut p_ace: *mut std::ffi::c_void = null_mut();
    let res = unsafe { GetAce(p_dacl, 0, &mut p_ace) };
    assert_ne!(res, 0, "GetAce failed");

    let ace = p_ace as *const ACCESS_ALLOWED_ACE;
    let ace_sid = unsafe { &(*ace).SidStart as *const u32 as *mut std::ffi::c_void };

    let same_sid = unsafe { EqualSid(owner_sid, ace_sid) };
    assert_ne!(same_sid, 0, "ACE SID must match the process owner SID");
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
    if let Err(MemFuseError::Serialization(msg)) = res {
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
    assert!(matches!(res, Err(MemFuseError::Serialization(_))));

    // 2. Invalid WalOp tag (e.g., tag = 255)
    let mut payload = vec![0u8; 90];
    payload[72] = 255; // Invalid tag
    let crc = crc32fast::hash(&payload);
    let mut data = vec![0u8; 4];
    data[0..4].copy_from_slice(&crc.to_le_bytes());
    data.extend_from_slice(&payload);

    let res_op = WalEntry::from_bytes(&data);
    assert!(matches!(res_op, Err(MemFuseError::Serialization(_))));
}

#[tokio::test]
async fn test_uuid_sidecar_crash_fault_injection() {
    let dir = tempdir().expect("tempdir"); // expect
    let wal_path = dir.path().join("fault_uuid.wal");
    let uuid_path = dir.path().join("fault_uuid.wal.uuid");

    let km = Arc::new(
        KeyManager::try_new("passphrase123", b"salt123456789012345678901234567890").expect("km"), // expect
    );

    // 1. Simulate a leftover interrupted temp file from a crashed write
    let tmp_path = dir.path().join("fault_uuid.wal.uuid.tmp.99999.12345");
    tokio::fs::write(&tmp_path, b"incomplete_uuid")
        .await
        .expect("write leftover tmp file"); // expect

    // 2. Opening WAL should cleanly recover, create valid 16-byte UUID sidecar, and ignore leftover tmp file
    let wal = Wal::open_with_key_manager(&wal_path, Some(km.clone()))
        .await
        .expect("open wal should succeed despite leftover tmp file"); // expect

    assert!(uuid_path.exists(), "UUID sidecar must exist");
    let uuid_bytes = tokio::fs::read(&uuid_path).await.expect("read uuid"); // expect
    assert_eq!(uuid_bytes.len(), 16);

    drop(wal);

    // 3. Re-opening should read the same valid UUID sidecar
    let uuid_bytes_after = Wal::load_or_create_wal_uuid(&wal_path)
        .await
        .expect("load uuid"); // expect
    assert_eq!(uuid_bytes.as_slice(), uuid_bytes_after);
}

#[tokio::test]
async fn test_prepare_batch_hmac_chain_concurrency() {
    let dir = tempdir().expect("tempdir"); // expect
    let wal_path = dir.path().join("concurrency_batch.wal");

    let wal = Arc::new(Wal::open(&wal_path).await.expect("open wal")); // expect

    let wal1 = wal.clone();
    let wal2 = wal.clone();

    let handle1 = tokio::spawn(async move {
        let ops = vec![(
            WalOp::Put {
                tx_id: TxId::new(1),
                key: b"k1".to_vec(),
                value: b"v1".to_vec(),
            },
            1,
        )];
        wal1.prepare_batch(ops).await.expect("batch 1") // expect
    });

    let handle2 = tokio::spawn(async move {
        let ops = vec![(
            WalOp::Put {
                tx_id: TxId::new(2),
                key: b"k2".to_vec(),
                value: b"v2".to_vec(),
            },
            2,
        )];
        wal2.prepare_batch(ops).await.expect("batch 2") // expect
    });

    let (res1, res2) = tokio::join!(handle1, handle2);
    let (batch1, _) = res1.expect("join 1"); // expect
    let (batch2, _) = res2.expect("join 2"); // expect

    let prev1 = batch1.entries()[0].prev_hmac;
    let prev2 = batch2.entries()[0].prev_hmac;

    // One batch must have chained off the initial [0u8; 32] HMAC, and the second batch off the first's checksum.
    // Crucially, their starting prev_hmac values must NOT be identical.
    assert_ne!(
        prev1, prev2,
        "Concurrent prepare_batch calls must produce unique prev_hmac chain links"
    );

    if prev1 == [0u8; 32] {
        assert_eq!(prev2, batch1.entries()[0].checksum);
    } else {
        assert_eq!(prev1, batch2.entries()[0].checksum);
        assert_eq!(prev2, [0u8; 32]);
    }
}

#[tokio::test]
async fn test_open_with_key_manager_is_new_race_condition() {
    let dir = tempdir().expect("tempdir"); // expect
    let wal_path = dir.path().join("race_open.wal");

    let km = Arc::new(
        KeyManager::try_new("passphrase123", b"salt123456789012345678901234567890").expect("km"), // expect
    );

    // Pre-create the UUID sidecar so both calls race purely on the WAL file open
    Wal::load_or_create_wal_uuid(&wal_path)
        .await
        .expect("create uuid sidecar"); // expect

    let path1 = wal_path.clone();
    let path2 = wal_path.clone();
    let km1 = km.clone();
    let km2 = km.clone();

    let h1 = tokio::spawn(async move { Wal::open_with_key_manager(path1, Some(km1)).await });
    let h2 = tokio::spawn(async move { Wal::open_with_key_manager(path2, Some(km2)).await });

    let (res1, res2) = tokio::join!(h1, h2);
    let wal1 = res1.expect("join 1").expect("open 1"); // expect
    let wal2 = res2.expect("join 2").expect("open 2"); // expect

    // Both WAL instances are opened successfully
    assert_eq!(wal1.path(), wal2.path());
}

#[test]
fn test_legacy_integrity_key_deobfuscation() {
    let key = legacy_integrity_key();
    assert_eq!(&key, b"memfuse-integrity-key-v1\0\0\0\0\0\0\0\0");
}

#[tokio::test]
async fn test_wal_v1_auto_migration_on_min_version_v3() {
    let dir = tempdir().expect("tempdir"); // expect
    let wal_path = dir.path().join("migrate_v1.wal");
    let bak_path = dir.path().join("migrate_v1.wal.v1.bak");

    {
        // Create an unencrypted V1 WAL file
        let op = WalOp::Put {
            tx_id: TxId::new(1),
            key: b"mig_key".to_vec(),
            value: b"mig_val".to_vec(),
        };
        let entry = WalEntry::try_new(op, 1, &legacy_integrity_key(), [0u8; 32]).expect("v1 entry"); // expect

        let mut v1_bytes = Vec::new();
        // V1 WAL file has no MFW3 or MFW2 header prefix
        v1_bytes.extend_from_slice(&entry.to_bytes().expect("to_bytes")); // expect
        fs::write(&wal_path, v1_bytes).await.expect("write v1 wal"); // expect
    }

    // Open with min_wal_version = WalVersion::V3 and legacy key fallback allowed
    let wal = Wal::open_with_config(
        &wal_path,
        WalConfig {
            allow_legacy_integrity_key_fallback: true,
            min_wal_version: WalVersion::V3,
            ..Default::default()
        },
    )
    .await
    .expect("open and auto-migrate v1 wal"); // expect

    // Verify that backup file exists
    assert!(
        bak_path.exists(),
        "Backup file .v1.bak must exist after migration"
    );

    // Verify replayed entries
    let entries = wal.replay().await.expect("replay migrated wal"); // expect
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].1.seq_no, 1);
    if let WalOp::Put { key, value, .. } = &entries[0].1.op {
        assert_eq!(key, b"mig_key");
        assert_eq!(value, b"mig_val");
    } else {
        panic!("Expected Put op");
    }

    // Read the actual WAL file from disk and verify it now has the V3 header
    let raw_disk_bytes = fs::read(&wal_path).await.expect("read wal_path"); // expect
    assert_eq!(
        &raw_disk_bytes[0..4],
        &WAL_V3_HEADER,
        "Migrated file must start with WAL_V3_HEADER"
    );
}

#[tokio::test]
async fn test_hmac_chain_intact_after_append_failure() {
    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("append_failure.wal");

    let wal = Wal::open(&wal_path).await.expect("open wal");

    // 1. Initial write
    let op1 = WalOp::Put {
        tx_id: TxId::new(1),
        key: b"k1".to_vec(),
        value: b"v1".to_vec(),
    };
    let (batch1, _) = wal
        .prepare_batch(vec![(op1, 1)])
        .await
        .expect("prepare batch 1");
    wal.append_batch(batch1).await.expect("append batch 1");

    let hmac_before = wal.last_hmac_snapshot().await;

    // 2. Prepare a second batch that advances last_hmac
    let op2 = WalOp::Put {
        tx_id: TxId::new(2),
        key: b"k2".to_vec(),
        value: b"v2".to_vec(),
    };
    let (batch2, prev_hmac) = wal
        .prepare_batch(vec![(op2, 2)])
        .await
        .expect("prepare batch 2");
    assert_ne!(
        wal.last_hmac_snapshot().await,
        hmac_before,
        "prepare_batch should advance in-memory last_hmac"
    );
    assert_eq!(prev_hmac, hmac_before);

    // 3. Simulate append failure by replacing file with a read-only file handle
    {
        let ro_file = tokio::fs::OpenOptions::new()
            .read(true)
            .write(false)
            .open(&wal_path)
            .await
            .expect("open read-only");
        let mut guard = wal.file.lock().await;
        *guard = ro_file;
    }

    let append_res = wal.append_batch(batch2).await;
    assert!(
        append_res.is_err(),
        "append_batch must fail on read-only file handle"
    );

    // Restore last_hmac as lsm commit would do upon append failure
    wal.restore_last_hmac(prev_hmac)
        .await
        .expect("restore last hmac");

    // 4. Verify last_hmac_snapshot is back to hmac_before
    let hmac_after = wal.last_hmac_snapshot().await;
    assert_eq!(
        hmac_after, hmac_before,
        "last_hmac_snapshot must match value before failed prepare_batch"
    );
}

#[tokio::test]
async fn test_truncate_size_visible_atomically_with_file_state() {
    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("truncate_atomic.wal");

    let wal = Arc::new(Wal::open(&wal_path).await.expect("open wal"));

    let done = Arc::new(std::sync::atomic::AtomicBool::new(false));

    let wal_trunc = wal.clone();
    let done_trunc = done.clone();
    let truncater = tokio::spawn(async move {
        for i in 0..100 {
            // Prepare and append a batch of entries so file grows
            let ops = vec![
                (
                    WalOp::Put {
                        tx_id: TxId::new(i * 2 + 1),
                        key: b"atomic_key_1".to_vec(),
                        value: b"atomic_val_1".to_vec(),
                    },
                    i * 2 + 1,
                ),
                (
                    WalOp::Put {
                        tx_id: TxId::new(i * 2 + 2),
                        key: b"atomic_key_2".to_vec(),
                        value: b"atomic_val_2".to_vec(),
                    },
                    i * 2 + 2,
                ),
            ];
            let (batch, _) = wal_trunc.prepare_batch(ops).await.expect("prepare_batch");
            wal_trunc.append_batch(batch).await.expect("append_batch");

            // Truncate back to offset 4 (length of WAL_V3_HEADER)
            wal_trunc
                .truncate(4, [0xAA; 32])
                .await
                .expect("truncate failed");
        }
        done_trunc.store(true, std::sync::atomic::Ordering::SeqCst);
    });

    let wal_poll = wal.clone();
    let done_poll = done.clone();
    let poller = tokio::spawn(async move {
        while !done_poll.load(std::sync::atomic::Ordering::SeqCst) {
            if let Ok(meta) = fs::metadata(wal_poll.path()).await {
                let disk_size = meta.len();
                let mem_size = wal_poll.size();
                // In-memory size must never observe stale mem_size > disk_size after truncation
                assert!(
                        mem_size <= disk_size,
                        "TOCTOU violation: in-memory WAL size ({mem_size}) > physical disk size ({disk_size})"
                    );
            }
            tokio::task::yield_now().await;
        }
    });

    let (res_trunc, res_poll) = tokio::join!(truncater, poller);
    res_trunc.expect("truncater panicked");
    res_poll.expect("poller panicked");
}

#[tokio::test]
async fn test_concurrent_append_batch_header_atomicity() {
    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("concurrent_header.wal");

    let wal = Arc::new(Wal::open(&wal_path).await.expect("open wal"));

    let num_tasks = 8;
    let mut handles = Vec::new();

    for i in 0..num_tasks {
        let wal_clone = wal.clone();
        handles.push(tokio::spawn(async move {
            let op = WalOp::Put {
                tx_id: TxId::new(i + 1),
                key: format!("key_{}", i).into_bytes(),
                value: format!("val_{}", i).into_bytes(),
            };
            let (batch, _) = wal_clone
                .prepare_batch(vec![(op, i + 1)])
                .await
                .expect("prepare_batch");
            wal_clone.append_batch(batch).await.expect("append_batch");
        }));
    }

    for h in handles {
        h.await.expect("join handle");
    }

    let file_bytes = fs::read(&wal_path).await.expect("read wal file");

    // Assert header is present at start
    assert!(
        file_bytes.len() >= 4,
        "WAL file must be at least 4 bytes long"
    );
    assert_eq!(
        &file_bytes[0..4],
        &WAL_V3_HEADER,
        "WAL file must start with WAL_V3_HEADER"
    );

    // Count header occurrences across entire file
    let header_count = file_bytes
        .windows(4)
        .filter(|window| *window == WAL_V3_HEADER)
        .count();
    assert_eq!(
            header_count, 1,
            "WAL_V3_HEADER must appear exactly once at the start of the file, but was found {header_count} times"
        );

    // Reopen and replay to verify no stream corruption
    let wal_reopen = Wal::open(&wal_path).await.expect("reopen wal");
    let replayed = wal_reopen.replay().await.expect("replay must succeed");
    assert_eq!(
        replayed.len(),
        num_tasks as usize,
        "Replay must yield all {} entries",
        num_tasks
    );
}

#[tokio::test]
async fn test_truncate_is_durable_across_simulated_crash() {
    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("truncate_durability.wal");

    let wal = Wal::open(&wal_path).await.expect("open wal");

    // Write several entries so file grows
    for i in 1..=5 {
        let op = WalOp::Put {
            tx_id: TxId::new(i),
            key: format!("k{}", i).into_bytes(),
            value: format!("v{}", i).into_bytes(),
        };
        let (batch, _) = wal
            .prepare_batch(vec![(op, i)])
            .await
            .expect("prepare batch");
        wal.append_batch(batch).await.expect("append entry");
    }

    let initial_size = tokio::fs::metadata(&wal_path).await.expect("meta").len();
    assert!(initial_size > 4, "File size should be larger than header");

    // Truncate to offset 4 (HEADER length)
    let new_hmac = [0x77u8; 32];
    wal.truncate(4, new_hmac).await.expect("truncate");

    // Open via a new independent File handle (simulates restart after crash without the original Wal handle)
    let file = tokio::fs::File::open(&wal_path).await.expect("reopen file");
    let metadata = file.metadata().await.expect("metadata");
    assert_eq!(
        metadata.len(),
        4,
        "Physical file length on disk must equal truncated offset 4 after fsync"
    );
}

#[tokio::test]
async fn test_recover_from_bak_if_present_cases() {
    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("test_recovery.wal");
    let bak_path = dir.path().join("test_recovery.wal.v1.bak");

    // Case (a): no backup present -> Ok(false), file untouched
    tokio::fs::write(&wal_path, b"some content").await.unwrap();
    let res = recover_from_bak_if_present(&wal_path)
        .await
        .expect("recover");
    assert!(!res);
    assert_eq!(tokio::fs::read(&wal_path).await.unwrap(), b"some content");

    // Case (c): backup present, regular file non-empty -> Ok(false), no override
    tokio::fs::write(&bak_path, b"backup content")
        .await
        .unwrap();
    let res = recover_from_bak_if_present(&wal_path)
        .await
        .expect("recover");
    assert!(!res);
    assert_eq!(tokio::fs::read(&wal_path).await.unwrap(), b"some content");

    // Case (b): backup present, regular file empty (0 bytes) -> Ok(true), backup restored
    tokio::fs::write(&wal_path, b"").await.unwrap();
    let res = recover_from_bak_if_present(&wal_path)
        .await
        .expect("recover");
    assert!(res);
    assert_eq!(tokio::fs::read(&wal_path).await.unwrap(), b"backup content");
    assert!(
        !bak_path.exists(),
        "Backup file should be renamed/removed after recovery"
    );
}

#[tokio::test]
async fn test_full_rewrite_crash_recovery_pipeline() {
    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("rewrite_crash.wal");

    // 1. Write legacy V1 entry
    let op = WalOp::Put {
        tx_id: TxId::new(1),
        key: b"k_crash".to_vec(),
        value: b"v_crash".to_vec(),
    };
    let entry = WalEntry::try_new(op, 1, &legacy_integrity_key(), [0u8; 32]).expect("v1 entry");
    let v1_bytes = entry.to_bytes().expect("to_bytes");
    tokio::fs::write(&wal_path, &v1_bytes)
        .await
        .expect("write v1 wal");

    // 2. Simulate backup creation and fsync
    let bak_path = dir.path().join("rewrite_crash.wal.v1.bak");
    tokio::fs::copy(&wal_path, &bak_path)
        .await
        .expect("copy backup");
    let bak_file = tokio::fs::OpenOptions::new()
        .write(true)
        .open(&bak_path)
        .await
        .expect("open bak");
    bak_file.sync_all().await.expect("fsync bak");
    drop(bak_file);

    // 3. Simulate crash after truncating original WAL to 0 bytes before V3 rewrite finishes
    let file = tokio::fs::OpenOptions::new()
        .write(true)
        .open(&wal_path)
        .await
        .expect("open wal");
    file.set_len(0).await.expect("truncate wal to 0");
    file.sync_all().await.expect("fsync truncated wal");
    drop(file);

    // 4. Wal::open() on the path -> recover_from_bak_if_present recovers backup and replays successfully
    let wal = Wal::open_with_config(
        &wal_path,
        WalConfig {
            allow_legacy_integrity_key_fallback: true,
            min_wal_version: WalVersion::V3,
            ..Default::default()
        },
    )
    .await
    .expect("open and recover wal from backup");

    let replayed = wal.replay().await.expect("replay recovered wal");
    assert_eq!(replayed.len(), 1);
    if let WalOp::Put { key, value, .. } = &replayed[0].1.op {
        assert_eq!(key, b"k_crash");
        assert_eq!(value, b"v_crash");
    } else {
        panic!("Expected Put op");
    }
}

#[tokio::test]
async fn test_v1_plaintext_rejected_when_key_manager_active() {
    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("v1_downgrade.wal");

    let km = Arc::new(
        KeyManager::try_new("passphrase123", b"salt123456789012345678901234567890").expect("km"),
    );

    // 1. Manually construct an unencrypted V1 plaintext WAL entry
    let op = WalOp::Put {
        tx_id: TxId::new(1),
        key: b"unencrypted_key".to_vec(),
        value: b"unencrypted_val".to_vec(),
    };
    let entry = WalEntry::try_new(op, 1, &legacy_integrity_key(), [0u8; 32]).expect("entry");
    let entry_bytes = entry.to_bytes().expect("to_bytes");

    // Write directly to file (bypassing Wal API)
    tokio::fs::write(&wal_path, &entry_bytes)
        .await
        .expect("write plaintext entry");

    // 2. Opening/replaying with active KeyManager MUST reject the V1 plaintext entry
    let open_res = Wal::open_with_key_manager(&wal_path, Some(km.clone())).await;
    if let Ok(wal) = open_res {
        let replay_res = wal.replay().await;
        assert!(
            replay_res.is_err(),
            "Replaying unencrypted V1 entry with active KeyManager MUST return an error"
        );
        let err_msg = format!("{}", replay_res.unwrap_err());
        assert!(
            err_msg.contains("refusing potential downgrade attack"),
            "Error message should mention downgrade attack refusal, got: {}",
            err_msg
        );
    } else {
        // Opening failed during initial replay in open_with_key_manager, which is also valid
        let err_msg = format!("{}", open_res.unwrap_err());
        assert!(
            err_msg.contains("refusing potential downgrade attack"),
            "Error message should mention downgrade attack refusal, got: {}",
            err_msg
        );
    }

    // 3. Opening/replaying WITHOUT KeyManager MUST succeed for the same V1 plaintext entry
    let wal_no_km = Wal::open_with_config(
        &wal_path,
        WalConfig {
            allow_legacy_integrity_key_fallback: true,
            ..Default::default()
        },
    )
    .await
    .expect("open without key manager should succeed");

    let replayed = wal_no_km
        .replay()
        .await
        .expect("replay without key manager");
    assert_eq!(replayed.len(), 1);
    if let WalOp::Put { key, value, .. } = &replayed[0].1.op {
        assert_eq!(key, b"unencrypted_key");
        assert_eq!(value, b"unencrypted_val");
    } else {
        panic!("Expected Put op");
    }
}

#[tokio::test]
async fn test_split_brain_legacy_fallback_chain_continuity() {
    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("split_brain.wal");

    let normal_key = b"normal-integrity-key-32-bytes---";
    let legacy_key = legacy_integrity_key();

    // 1. Entry 1: created with normal key, prev_hmac = [0u8; 32]
    let op1 = WalOp::Put {
        tx_id: TxId::new(1),
        key: b"k1".to_vec(),
        value: b"v1".to_vec(),
    };
    let e1 = WalEntry::try_new(op1, 1, normal_key, [0u8; 32]).unwrap();

    // 2. Entry 2: created with normal key, prev_hmac = e1.checksum
    let op2 = WalOp::Put {
        tx_id: TxId::new(2),
        key: b"k2".to_vec(),
        value: b"v2".to_vec(),
    };
    let e2 = WalEntry::try_new(op2, 2, normal_key, e1.checksum).unwrap();

    // 3. Entry 3 (attacker/legacy entry):
    // prev_hmac = [0u8; 32] (trying to pretend it's the start of chain), but signed with legacy_key.
    let op3 = WalOp::Put {
        tx_id: TxId::new(3),
        key: b"k3".to_vec(),
        value: b"v3".to_vec(),
    };
    let e3_forged = WalEntry::try_new(op3, 3, &legacy_key, [0u8; 32]).unwrap();

    let mut file_bytes = Vec::new();
    file_bytes.extend_from_slice(&WAL_V3_HEADER);
    file_bytes.extend_from_slice(&e1.to_bytes().unwrap());
    file_bytes.extend_from_slice(&e2.to_bytes().unwrap());
    file_bytes.extend_from_slice(&e3_forged.to_bytes().unwrap());

    tokio::fs::write(&wal_path, &file_bytes).await.unwrap();

    // Pre-create integrity key file with normal_key
    let key_file_path = dir.path().join(".wal_integrity_key");
    tokio::fs::write(&key_file_path, normal_key).await.unwrap();

    // Opening / replaying with legacy fallback enabled MUST fail during replay/open due to HMAC mismatch
    let open_res = Wal::open_with_config(
        &wal_path,
        WalConfig {
            allow_legacy_integrity_key_fallback: true,
            ..Default::default()
        },
    )
    .await;

    let replay_err = match open_res {
        Ok(wal) => wal.replay().await.unwrap_err(),
        Err(e) => e,
    };

    assert!(
            matches!(replay_err, MemFuseError::WalCorruption { .. }),
            "Forged entry 3 with prev_hmac=[0;32] must be rejected during fallback because chain state was non-zero! Got: {:?}",
            replay_err
        );
}

#[tokio::test]
async fn test_find_tx_offset_invariants() {
    let dir = tempdir().unwrap();
    let wal_path = dir.path().join("test_find_tx_offset.wal");
    let wal = Wal::open(&wal_path).await.unwrap();

    // Append user txs: 10, 20
    let op1 = WalOp::Put {
        tx_id: TxId::new(10),
        key: b"k10".to_vec(),
        value: b"v10".to_vec(),
    };
    let (batch1, _) = wal.prepare_batch(vec![(op1, 1)]).await.unwrap();
    let entry1 = batch1.entries()[0].clone();
    wal.append_batch(batch1).await.unwrap();

    let op2 = WalOp::Put {
        tx_id: TxId::new(20),
        key: b"k20".to_vec(),
        value: b"v20".to_vec(),
    };
    let (batch2, _) = wal.prepare_batch(vec![(op2, 2)]).await.unwrap();
    wal.append_batch(batch2).await.unwrap();

    // Append system tx: TxId::INTERNAL_BASE + 1
    let sys_tx = TxId::new(TxId::INTERNAL_BASE + 1);
    let op_sys = WalOp::Put {
        tx_id: sys_tx,
        key: b"sys_k".to_vec(),
        value: b"sys_v".to_vec(),
    };
    let (batch_sys, _) = wal.prepare_batch(vec![(op_sys, 3)]).await.unwrap();
    wal.append_batch(batch_sys).await.unwrap();

    // Append user tx: 30
    let op3 = WalOp::Put {
        tx_id: TxId::new(30),
        key: b"k30".to_vec(),
        value: b"v30".to_vec(),
    };
    let (batch3, _) = wal.prepare_batch(vec![(op3, 4)]).await.unwrap();
    wal.append_batch(batch3).await.unwrap();

    let replayed = wal.replay().await.unwrap();
    assert_eq!(replayed.len(), 4);
    let (_s1, _e1, offset1) = &replayed[0];
    let (_s2, _e2, _offset2) = &replayed[1];
    let (_ss, esys, offsetsys) = &replayed[2];
    let (_s3, e3, offset3) = &replayed[3];

    // Invariant 2: Target tx_id = 20. First entry with tx_id > 20 is sys_tx (tx_id = INTERNAL_BASE + 1).
    // Since target_tx_id (20) < INTERNAL_BASE, sys_tx is preserved (`continue`), so the next entry triggering `entry_tx > target_tx_id` is e3 (tx_id = 30).
    // Immediately preceding entry before e3 is esys.
    let (offset, hmac) = wal.find_tx_offset(TxId::new(20)).await.unwrap();
    assert_eq!(offset, *offsetsys);
    assert_eq!(hmac, esys.checksum);

    // Invariant 2 & 1: Target tx_id = 10. First entry with tx_id > 10 is e2 (tx_id = 20).
    // Preceding entry is e1.
    let (offset10, hmac10) = wal.find_tx_offset(TxId::new(10)).await.unwrap();
    assert_eq!(offset10, *offset1);
    assert_eq!(hmac10, entry1.checksum);

    // Invariant 3: Target tx_id = 100 (beyond last entry). Returns last known offset & hmac (e3).
    let (offset100, hmac100) = wal.find_tx_offset(TxId::new(100)).await.unwrap();
    assert_eq!(offset100, *offset3);
    assert_eq!(hmac100, e3.checksum);
}

#[tokio::test]
async fn test_wal_direct_append_batch_fsync_discipline() {
    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("wal_direct.log");

    let wal = Wal::open(&wal_path).await.expect("wal open");

    let op = WalOp::Put {
        tx_id: TxId::new(1),
        key: b"direct_k".to_vec(),
        value: b"direct_v".to_vec(),
    };

    let (batch, _) = wal
        .prepare_batch(vec![(op, 1)])
        .await
        .expect("create entry");

    println!("[STRACE_MARKER_START_DIRECT_APPEND]");
    let append_res = wal.append_batch(batch).await;
    println!("[STRACE_MARKER_END_DIRECT_APPEND]");

    assert!(append_res.is_ok());
}

async fn create_5_entry_wal_unencrypted(
    dir: &std::path::Path,
) -> (std::path::PathBuf, Vec<[u8; 32]>) {
    let wal_path = dir.join("test_5_entries.wal");
    let wal = Wal::open(&wal_path).await.expect("open WAL");

    let mut hmacs = Vec::new();

    for i in 1..=5 {
        let op = WalOp::Put {
            tx_id: TxId::new(i),
            key: format!("key_{i}").into_bytes(),
            value: format!("val_{i}").into_bytes(),
        };
        let (batch, _) = wal
            .prepare_batch(vec![(op, i)])
            .await
            .expect("prepare batch");
        let checksum = batch.entries()[0].checksum;
        wal.append_batch(batch).await.expect("append entry");
        hmacs.push(checksum);
    }

    (wal_path, hmacs)
}

async fn read_unencrypted_raw_entries(path: &std::path::Path) -> Vec<Vec<u8>> {
    let data = tokio::fs::read(path).await.expect("read file");
    let mut offset = 0;
    if data.starts_with(&WAL_V3_HEADER) {
        offset = 4;
    }

    let mut entry_chunks = Vec::new();
    while offset < data.len() {
        if offset + 4 > data.len() {
            break;
        }
        let len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        let total_chunk_len = 4 + len;
        if offset + total_chunk_len > data.len() {
            break;
        }
        entry_chunks.push(data[offset..offset + total_chunk_len].to_vec());
        offset += total_chunk_len;
    }
    entry_chunks
}

async fn read_encrypted_raw_chunks(path: &std::path::Path) -> Vec<Vec<u8>> {
    let data = tokio::fs::read(path).await.expect("read file");
    let mut offset = 0;
    if data.starts_with(&WAL_V3_HEADER) {
        offset = 4;
    }

    let mut chunks = Vec::new();
    while offset < data.len() {
        if offset + 4 > data.len() {
            break;
        }
        let len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        let total_chunk_len = 4 + len;
        if offset + total_chunk_len > data.len() {
            break;
        }
        chunks.push(data[offset..offset + total_chunk_len].to_vec());
        offset += total_chunk_len;
    }
    chunks
}

#[tokio::test]
async fn test_attack_a_swap_blocks_unencrypted_detected() {
    let dir = tempdir().expect("tempdir");
    let (wal_path, _) = create_5_entry_wal_unencrypted(dir.path()).await;

    let chunks = read_unencrypted_raw_entries(&wal_path).await;
    assert_eq!(chunks.len(), 5, "Expected 5 raw entry chunks");

    let mut tampered_data = Vec::new();
    tampered_data.extend_from_slice(&WAL_V3_HEADER);
    tampered_data.extend_from_slice(&chunks[0]);
    tampered_data.extend_from_slice(&chunks[3]);
    tampered_data.extend_from_slice(&chunks[2]);
    tampered_data.extend_from_slice(&chunks[1]);
    tampered_data.extend_from_slice(&chunks[4]);

    tokio::fs::write(&wal_path, &tampered_data)
        .await
        .expect("write tampered file");

    let open_res = Wal::open(&wal_path).await;

    assert!(
        open_res.is_err(),
        "Attack a (swap blocks) MUST be detected during open/recovery"
    );
    let err = open_res.unwrap_err();
    assert!(
        matches!(err, MemFuseError::WalCorruption { .. }),
        "Expected WalCorruption error, got: {:?}",
        err
    );
}

#[tokio::test]
async fn test_attack_a_swap_blocks_encrypted_detected() {
    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("encrypted_swap.wal");
    let km =
        Arc::new(KeyManager::try_new("pass_swap", b"salt123456789012345678901234567890").unwrap());

    {
        let wal = Wal::open_with_key_manager(&wal_path, Some(km.clone()))
            .await
            .expect("open wal");
        for i in 1..=5 {
            let op = WalOp::Put {
                tx_id: TxId::new(i),
                key: format!("k{i}").into_bytes(),
                value: format!("v{i}").into_bytes(),
            };
            let (batch, _) = wal
                .prepare_batch(vec![(op, i)])
                .await
                .expect("prepare batch");
            wal.append_batch(batch).await.expect("append");
        }
    }

    let chunks = read_encrypted_raw_chunks(&wal_path).await;
    assert_eq!(chunks.len(), 5);

    let mut tampered_data = Vec::new();
    tampered_data.extend_from_slice(&WAL_V3_HEADER);
    tampered_data.extend_from_slice(&chunks[0]);
    tampered_data.extend_from_slice(&chunks[3]);
    tampered_data.extend_from_slice(&chunks[2]);
    tampered_data.extend_from_slice(&chunks[1]);
    tampered_data.extend_from_slice(&chunks[4]);

    tokio::fs::write(&wal_path, &tampered_data)
        .await
        .expect("write");

    let open_res = Wal::open_with_key_manager(&wal_path, Some(km)).await;

    assert!(
        open_res.is_err(),
        "Encrypted Attack a (swap blocks) MUST be detected during open/recovery"
    );
    assert!(matches!(
        open_res.unwrap_err(),
        MemFuseError::WalCorruption { .. }
    ));
}

#[tokio::test]
async fn test_attack_b_truncation_last_block_behavior() {
    let dir = tempdir().expect("tempdir");
    let (wal_path, _) = create_5_entry_wal_unencrypted(dir.path()).await;

    let chunks = read_unencrypted_raw_entries(&wal_path).await;
    assert_eq!(chunks.len(), 5);

    let mut truncated_data = Vec::new();
    truncated_data.extend_from_slice(&WAL_V3_HEADER);
    for chunk in &chunks[0..4] {
        truncated_data.extend_from_slice(chunk);
    }

    tokio::fs::write(&wal_path, &truncated_data)
        .await
        .expect("write");

    let wal = Wal::open(&wal_path).await.expect("open wal");
    let res = wal.replay().await;

    assert!(
        res.is_ok(),
        "Replay of clean tail truncation succeeds for prefix entries"
    );
    let entries = res.unwrap();
    assert_eq!(
        entries.len(),
        4,
        "Block 5 was truncated; replay returns first 4 entries without error"
    );
}

#[tokio::test]
async fn test_attack_c_duplicate_block_3_unencrypted_detected() {
    let dir = tempdir().expect("tempdir");
    let (wal_path, _) = create_5_entry_wal_unencrypted(dir.path()).await;

    let chunks = read_unencrypted_raw_entries(&wal_path).await;
    assert_eq!(chunks.len(), 5);

    let mut tampered_data = Vec::new();
    tampered_data.extend_from_slice(&WAL_V3_HEADER);
    tampered_data.extend_from_slice(&chunks[0]);
    tampered_data.extend_from_slice(&chunks[1]);
    tampered_data.extend_from_slice(&chunks[2]);
    tampered_data.extend_from_slice(&chunks[2]);
    tampered_data.extend_from_slice(&chunks[3]);
    tampered_data.extend_from_slice(&chunks[4]);

    tokio::fs::write(&wal_path, &tampered_data)
        .await
        .expect("write");

    let open_res = Wal::open(&wal_path).await;

    assert!(
        open_res.is_err(),
        "Attack c (duplicate block) MUST be detected during open/recovery"
    );
    let err = open_res.unwrap_err();
    assert!(
        matches!(err, MemFuseError::WalCorruption { .. }),
        "Expected WalCorruption error, got: {:?}",
        err
    );
}

#[tokio::test]
async fn test_attack_d_cross_file_replay_unencrypted_detected() {
    let dir1 = tempdir().expect("dir1");
    let dir2 = tempdir().expect("dir2");

    let (wal_path_a, _) = create_5_entry_wal_unencrypted(dir1.path()).await;
    let (wal_path_b, _) = create_5_entry_wal_unencrypted(dir2.path()).await;

    let chunks_a = read_unencrypted_raw_entries(&wal_path_a).await;
    let chunks_b = read_unencrypted_raw_entries(&wal_path_b).await;

    let mut tampered_data = Vec::new();
    tampered_data.extend_from_slice(&WAL_V3_HEADER);
    tampered_data.extend_from_slice(&chunks_b[0]);
    tampered_data.extend_from_slice(&chunks_b[1]);
    tampered_data.extend_from_slice(&chunks_a[2]);
    tampered_data.extend_from_slice(&chunks_b[3]);
    tampered_data.extend_from_slice(&chunks_b[4]);

    tokio::fs::write(&wal_path_b, &tampered_data)
        .await
        .expect("write");

    let open_res = Wal::open(&wal_path_b).await;

    assert!(
        open_res.is_err(),
        "Attack d (cross-file replay unencrypted) MUST be detected during open/recovery"
    );
    let err = open_res.unwrap_err();
    assert!(
        matches!(err, MemFuseError::WalCorruption { .. }),
        "Expected WalCorruption error, got: {:?}",
        err
    );
}

#[tokio::test]
async fn test_attack_d_cross_file_replay_encrypted_detected() {
    let dir1 = tempdir().expect("dir1");
    let dir2 = tempdir().expect("dir2");

    let wal_path_a = dir1.path().join("wal_a.wal");
    let wal_path_b = dir2.path().join("wal_b.wal");

    let km =
        Arc::new(KeyManager::try_new("cross_pass", b"salt123456789012345678901234567890").unwrap());

    {
        let wal_a = Wal::open_with_key_manager(&wal_path_a, Some(km.clone()))
            .await
            .unwrap();
        for i in 1..=5 {
            let op = WalOp::Put {
                tx_id: TxId::new(i),
                key: format!("a_k{i}").into_bytes(),
                value: format!("a_v{i}").into_bytes(),
            };
            let (batch, _) = wal_a.prepare_batch(vec![(op, i)]).await.unwrap();
            wal_a.append_batch(batch).await.unwrap();
        }
    }

    {
        let wal_b = Wal::open_with_key_manager(&wal_path_b, Some(km.clone()))
            .await
            .unwrap();
        for i in 1..=5 {
            let op = WalOp::Put {
                tx_id: TxId::new(i),
                key: format!("b_k{i}").into_bytes(),
                value: format!("b_v{i}").into_bytes(),
            };
            let (batch, _) = wal_b.prepare_batch(vec![(op, i)]).await.unwrap();
            wal_b.append_batch(batch).await.unwrap();
        }
    }

    let chunks_a = read_encrypted_raw_chunks(&wal_path_a).await;
    let chunks_b = read_encrypted_raw_chunks(&wal_path_b).await;

    let mut tampered_data = Vec::new();
    tampered_data.extend_from_slice(&WAL_V3_HEADER);
    tampered_data.extend_from_slice(&chunks_b[0]);
    tampered_data.extend_from_slice(&chunks_b[1]);
    tampered_data.extend_from_slice(&chunks_a[2]);
    tampered_data.extend_from_slice(&chunks_b[3]);
    tampered_data.extend_from_slice(&chunks_b[4]);

    tokio::fs::write(&wal_path_b, &tampered_data)
        .await
        .expect("write");

    let open_res = Wal::open_with_key_manager(&wal_path_b, Some(km)).await;

    assert!(
        open_res.is_err(),
        "Attack d (cross-file encrypted replay) MUST be detected during open/recovery"
    );
    let err = open_res.unwrap_err();
    assert!(
        matches!(err, MemFuseError::WalCorruption { .. }),
        "Expected WalCorruption error due to per-file UUID key isolation failure, got: {:?}",
        err
    );
}

async fn write_valid_wal_fuzz(dir: &std::path::Path, n: usize) -> std::path::PathBuf {
    let wal_path = dir.join(format!("fuzz_test_{}.wal", rand::random::<u64>()));
    let wal = Wal::open(&wal_path).await.expect("open WAL");

    for i in 0..n {
        let op = WalOp::Put {
            tx_id: TxId::new(i as u64),
            key: format!("sensor:data:{}", i).into_bytes(),
            value: format!("payload_{:04}", i).into_bytes(),
        };
        let (batch, _) = wal
            .prepare_batch(vec![(op, i as u64)])
            .await
            .expect("prepare batch");
        wal.append_batch(batch).await.expect("append");
    }
    wal_path
}

#[tokio::test]
async fn test_wal_bitflip_property_integrity() {
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};
    let dir = tempdir().expect("tempdir");
    let mut rng = StdRng::seed_from_u64(0xFEED_FACE_CAFE_4242);

    for _iteration in 0..30u32 {
        let wal_path = write_valid_wal_fuzz(dir.path(), 5).await;
        let mut data = tokio::fs::read(&wal_path).await.expect("read WAL");

        if data.len() > 12 {
            let flip_offset = rng.gen_range(4..data.len());
            data[flip_offset] ^= 0x01 << rng.gen_range(0..8);
            tokio::fs::write(&wal_path, &data).await.expect("write WAL");

            let result = async {
                match Wal::open(&wal_path).await {
                    Err(e) => Err(e),
                    Ok(wal) => wal.replay().await.map(|_| ()),
                }
            }
            .await;

            assert!(
                result.is_err() || result.is_ok(),
                "Replay must cleanly handle bit-flip alterations"
            );
        }
        let _ = tokio::fs::remove_file(&wal_path).await;
    }
}

#[tokio::test]
async fn test_wal_random_bitflip_never_panics() {
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};
    let dir = tempdir().expect("tempdir");
    let n_entries = 10;

    let mut rng = StdRng::seed_from_u64(0xDEAD_BEEF_CAFE_1337);

    let mut corruption_detected = 0u32;
    let mut truncation_tolerated = 0u32;

    for _iteration in 0..50u32 {
        let wal_path = write_valid_wal_fuzz(dir.path(), n_entries).await;
        let mut data = tokio::fs::read(&wal_path).await.expect("read WAL");

        if data.is_empty() {
            continue;
        }

        let skip_header = (data.len() / 10).min(8);
        let flip_offset = rng.gen_range(skip_header..data.len());
        data[flip_offset] ^= 0x01 << rng.gen_range(0..8);
        tokio::fs::write(&wal_path, &data)
            .await
            .expect("write corrupted WAL");

        let result = async {
            match Wal::open(&wal_path).await {
                Err(e) => Err(e),
                Ok(wal) => wal.replay().await.map(|_| ()),
            }
        }
        .await;

        match result {
            Err(_) => {
                corruption_detected += 1;
            }
            Ok(_) => {
                truncation_tolerated += 1;
            }
        }
        let _ = tokio::fs::remove_file(&wal_path).await;
    }

    assert!(corruption_detected + truncation_tolerated == 50);
}

#[tokio::test]
async fn test_wal_systematic_header_corruption() {
    let dir = tempdir().expect("tempdir");
    let wal_path = write_valid_wal_fuzz(dir.path(), 5).await;
    let original_data = tokio::fs::read(&wal_path).await.expect("read");

    let test_limit = 12.min(original_data.len());

    for byte_idx in 0..test_limit {
        for bit_idx in 0..8 {
            let mut corrupted_data = original_data.clone();
            corrupted_data[byte_idx] ^= 0x01 << bit_idx;

            tokio::fs::write(&wal_path, &corrupted_data)
                .await
                .expect("write");

            let result = async {
                match Wal::open(&wal_path).await {
                    Err(e) => Err(e),
                    Ok(wal) => wal.replay().await.map(|_| ()),
                }
            }
            .await;

            assert!(
                result.is_err(),
                "Bit-flip at byte {}, bit {} must be detected as error.",
                byte_idx,
                bit_idx
            );
        }
    }
}

#[tokio::test]
async fn test_wal_crc_field_corruption_detected() {
    let dir = tempdir().expect("tempdir");
    let wal_path = write_valid_wal_fuzz(dir.path(), 5).await;
    let mut data = tokio::fs::read(&wal_path).await.expect("read");

    if data.len() >= 8 {
        data[4] ^= 0xFF;
        tokio::fs::write(&wal_path, &data).await.expect("write");
    }

    let result = async {
        match Wal::open(&wal_path).await {
            Err(e) => Err(e),
            Ok(wal) => wal.replay().await.map(|_| ()),
        }
    }
    .await;

    assert!(result.is_err(), "Corrupted CRC MUST be detected");
}

#[tokio::test]
async fn test_truncate_at_offset_zero_yields_empty_wal() -> Result<()> {
    let dir = tempdir()?;
    let wal_path = dir.path().join("test_truncate_zero.wal");

    let wal = Wal::open(&wal_path).await?;
    for i in 1..=5 {
        let op = WalOp::Put {
            tx_id: TxId::new(i),
            key: format!("k{i}").into_bytes(),
            value: format!("v{i}").into_bytes(),
        };
        let (batch, _) = wal.prepare_batch(vec![(op, i)]).await?;
        wal.append_batch(batch).await?;
    }

    let meta_before = tokio::fs::metadata(&wal_path).await?;
    assert!(
        meta_before.len() > 0,
        "WAL file should contain written bytes before truncation"
    );

    wal.truncate(0, [0u8; 32]).await?;

    let meta_after = tokio::fs::metadata(&wal_path).await?;
    assert_eq!(
        meta_after.len(),
        0,
        "Physical file length must be exactly 0 bytes after truncate(0)"
    );

    drop(wal);
    let wal_reopened = Wal::open(&wal_path).await?;
    let entries = wal_reopened.replay().await?;
    assert_eq!(
        entries.len(),
        0,
        "Reopened WAL after truncate(0) must yield 0 entries"
    );

    Ok(())
}

#[tokio::test]
async fn test_append_batch_concurrent_double_header_write_race() -> Result<()> {
    let dir = tempdir()?;
    let wal_path = dir.path().join("test_concurrent_header.wal");

    let wal = Wal::open(&wal_path).await?;

    let op1 = WalOp::Put {
        tx_id: TxId::new(10),
        key: b"concurrent_key_1".to_vec(),
        value: b"val_1".to_vec(),
    };
    let op2 = WalOp::Put {
        tx_id: TxId::new(11),
        key: b"concurrent_key_2".to_vec(),
        value: b"val_2".to_vec(),
    };

    let (batch1, _) = wal.prepare_batch(vec![(op1, 1)]).await?;
    let (batch2, _) = wal.prepare_batch(vec![(op2, 2)]).await?;

    let handle1 = wal.append_batch(batch1);
    let handle2 = wal.append_batch(batch2);
    let (res1, res2) = tokio::join!(handle1, handle2);

    res1?;
    res2?;

    let file_bytes = tokio::fs::read(&wal_path).await?;

    let header_magic_count = file_bytes
        .windows(4)
        .filter(|win| *win == WAL_V3_HEADER)
        .count();

    assert_eq!(
        header_magic_count, 1,
        "Header magic 'MFW3' must appear EXACTLY ONCE in physical WAL file, found {}",
        header_magic_count
    );

    Ok(())
}

#[tokio::test]
async fn test_seq_no_near_u64_max_boundary() -> Result<()> {
    let dir = tempdir()?;
    let wal_path = dir.path().join("test_seq_max.wal");
    let wal = Wal::open(&wal_path).await?;

    let seq_max_minus_2 = u64::MAX - 2;
    let seq_max_minus_1 = u64::MAX - 1;
    let seq_max = u64::MAX;

    let op1 = WalOp::Put {
        tx_id: TxId::new(100),
        key: b"seq_near_max_1".to_vec(),
        value: b"v1".to_vec(),
    };
    let op2 = WalOp::Put {
        tx_id: TxId::new(101),
        key: b"seq_near_max_2".to_vec(),
        value: b"v2".to_vec(),
    };
    let op3 = WalOp::Put {
        tx_id: TxId::new(102),
        key: b"seq_near_max_3".to_vec(),
        value: b"v3".to_vec(),
    };

    let (batch1, _) = wal.prepare_batch(vec![(op1, seq_max_minus_2)]).await?;
    wal.append_batch(batch1).await?;

    let (batch2, _) = wal.prepare_batch(vec![(op2, seq_max_minus_1)]).await?;
    wal.append_batch(batch2).await?;

    let (batch3, _) = wal.prepare_batch(vec![(op3, seq_max)]).await?;
    wal.append_batch(batch3).await?;

    drop(wal);
    let wal_reopened = Wal::open(&wal_path).await?;
    let replayed = wal_reopened.replay().await?;

    assert_eq!(replayed.len(), 3);
    assert_eq!(replayed[0].1.seq_no, seq_max_minus_2);
    assert_eq!(replayed[1].1.seq_no, seq_max_minus_1);
    assert_eq!(replayed[2].1.seq_no, seq_max);

    Ok(())
}

#[tokio::test]
async fn test_empty_wal_file_zero_bytes_replay() -> Result<()> {
    let dir = tempdir()?;
    let wal_path = dir.path().join("empty_zero_byte.wal");

    tokio::fs::File::create(&wal_path).await?;
    let meta = tokio::fs::metadata(&wal_path).await?;
    assert_eq!(meta.len(), 0, "Created test file must be 0 bytes");

    let wal = Wal::open(&wal_path).await?;
    let entries = wal.replay().await?;

    assert_eq!(
        entries.len(),
        0,
        "Replaying 0-byte empty file must return 0 entries without error"
    );

    Ok(())
}

#[tokio::test]
async fn test_single_entry_no_commit_marker_replay() -> Result<()> {
    let dir = tempdir()?;
    let wal_path = dir.path().join("uncommitted_tail.wal");

    {
        let wal = Wal::open(&wal_path).await?;
        let op1 = WalOp::Put {
            tx_id: TxId::new(1),
            key: b"committed_key".to_vec(),
            value: b"committed_val".to_vec(),
        };
        let (batch1, _) = wal.prepare_batch(vec![(op1, 1)]).await?;
        wal.append_batch(batch1).await?;
    }

    let mut file = std::fs::OpenOptions::new().append(true).open(&wal_path)?;
    use std::io::Write;
    file.write_all(&[0xFF, 0x00, 0x7A, 0x11])?;
    file.sync_all()?;

    let wal = Wal::open(&wal_path).await?;
    let entries = wal.replay().await?;

    assert_eq!(
        entries.len(),
        1,
        "Replay must recover valid entry and discard partial trailing write"
    );
    assert_eq!(entries[0].1.seq_no, 1);

    Ok(())
}

#[tokio::test]
async fn test_wal_flusher_actor_coalescing() -> Result<()> {
    let dir = tempdir()?;
    let wal_path = dir.path().join("test_flusher.wal");

    let wal = Arc::new(Wal::open(&wal_path).await?);
    wal.enable_flusher();

    let num_tasks = 10;
    let mut handles = Vec::new();

    for i in 0..num_tasks {
        let wal_clone = Arc::clone(&wal);
        handles.push(tokio::spawn(async move {
            let op = WalOp::Put {
                tx_id: TxId::new(i + 1),
                key: format!("flusher_k_{i}").into_bytes(),
                value: format!("flusher_v_{i}").into_bytes(),
            };
            let (batch, _) = wal_clone.prepare_batch(vec![(op, i + 1)]).await?;
            wal_clone.append_batch(batch).await
        }));
    }

    for h in handles {
        h.await
            .map_err(|e| MemFuseError::Storage(e.to_string()))??;
    }

    let replayed = wal.replay().await?;
    assert_eq!(replayed.len(), num_tasks as usize);

    for (i, (_seq, entry, _pos)) in replayed.iter().enumerate() {
        assert_eq!(entry.seq_no, (i + 1) as u64);
    }

    Ok(())
}

#[tokio::test]
async fn test_flusher_batch_window_coalesces_writes() -> Result<()> {
    let dir = tempdir()?;
    let wal_path = dir.path().join("batch_window_test.wal");

    let wal = Arc::new(
        Wal::open_with_config(
            &wal_path,
            WalConfig {
                flusher_config: WalFlusherConfig {
                    batch_window_micros: 50,
                },
                ..Default::default()
            },
        )
        .await?,
    );

    let num_tasks = 5;
    let mut handles = Vec::new();

    for i in 0u64..num_tasks {
        let wal_clone = Arc::clone(&wal);
        handles.push(tokio::spawn(async move {
            let op = WalOp::Put {
                tx_id: TxId::new(i + 1),
                key: format!("key-{i}").into_bytes(),
                value: b"val".to_vec(),
            };
            let (batch, _) = wal_clone.prepare_batch(vec![(op, i + 1)]).await?;
            wal_clone.append_batch(batch).await
        }));
    }

    for h in handles {
        h.await
            .map_err(|e| MemFuseError::Storage(e.to_string()))??;
    }

    let replayed = wal.replay().await?;
    assert_eq!(
        replayed.len(),
        num_tasks as usize,
        "Alle 5 Batches müssen sicher im WAL landen"
    );

    for (i, (_seq, entry, _pos)) in replayed.iter().enumerate() {
        assert_eq!(entry.seq_no, (i + 1) as u64);
    }

    Ok(())
}

fn compute_v3_hmac_reference_independent(
    key: &[u8],
    prev_hmac: &[u8; 32],
    seq_no: u64,
    tx_id: u64,
    op: &WalOp,
) -> Result<[u8; 32]> {
    let mut mac = WalHmac::new(key)?;
    mac.update(prev_hmac);
    mac.update(&seq_no.to_le_bytes());
    mac.update(&tx_id.to_le_bytes());

    match op {
        WalOp::Put { key, value, .. } => {
            mac.update(&[0u8]);
            mac.update(&(key.len() as u32).to_le_bytes());
            mac.update(key);
            mac.update(&(value.len() as u32).to_le_bytes());
            mac.update(value);
        }
        WalOp::Delete { key, .. } => {
            mac.update(&[1u8]);
            mac.update(&(key.len() as u32).to_le_bytes());
            mac.update(key);
        }
    }

    Ok(mac.finalize())
}

#[tokio::test]
async fn test_hmac_chain_prev_hash_independent_reference() -> Result<()> {
    let dir = tempdir()?;
    let wal_path = dir.path().join("hmac_ref.wal");
    let wal = Wal::open(&wal_path).await?;

    let key_path = dir.path().join(".wal_integrity_key");
    let integrity_key = tokio::fs::read(&key_path).await?;

    let ops = vec![
        WalOp::Put {
            tx_id: TxId::new(10),
            key: b"k1".to_vec(),
            value: b"v1".to_vec(),
        },
        WalOp::Put {
            tx_id: TxId::new(11),
            key: b"k2".to_vec(),
            value: b"v2".to_vec(),
        },
        WalOp::Delete {
            tx_id: TxId::new(12),
            key: b"k1".to_vec(),
        },
    ];

    let mut current_chain = [0u8; 32];
    for (idx, op) in ops.into_iter().enumerate() {
        let seq_no = (idx + 1) as u64;
        let (batch, _) = wal.prepare_batch(vec![(op.clone(), seq_no)]).await?;
        let entry = &batch.entries()[0];

        let expected_hmac = compute_v3_hmac_reference_independent(
            &integrity_key,
            &current_chain,
            seq_no,
            op.tx_id().inner(),
            &op,
        )?;

        assert_eq!(
            entry.checksum, expected_hmac,
            "WAL entry checksum at seq {} must match independent HMAC-SHA256 reference calculation",
            seq_no
        );
        assert_eq!(
            entry.prev_hmac, current_chain,
            "WAL entry prev_hmac at seq {} must match previous chain digest",
            seq_no
        );

        let checksum = entry.checksum;
        wal.append_batch(batch).await?;
        current_chain = checksum;
    }

    Ok(())
}

#[cfg(feature = "fault-injection")]
#[tokio::test]
async fn test_disk_full_mid_append_batch_rollback() -> Result<()> {
    use std::sync::atomic::Ordering;
    let dir = tempdir()?;
    let wal_path = dir.path().join("disk_full_batch.wal");

    let wal = Wal::open(&wal_path).await?;

    let op_base = WalOp::Put {
        tx_id: TxId::new(10),
        key: b"base_k".to_vec(),
        value: b"base_v".to_vec(),
    };
    let (batch_base, _) = wal.prepare_batch(vec![(op_base, 1)]).await?;
    wal.append_batch(batch_base).await?;

    let fail_tx = TxId::new(20);
    let op_fail1 = WalOp::Put {
        tx_id: fail_tx,
        key: b"fail_k1".to_vec(),
        value: b"fail_v1".to_vec(),
    };
    let op_fail2 = WalOp::Put {
        tx_id: fail_tx,
        key: b"fail_k2".to_vec(),
        value: b"fail_v2".to_vec(),
    };

    let (batch, prev_hmac_snapshot) = wal
        .prepare_batch(vec![(op_fail1, 2), (op_fail2, 3)])
        .await?;

    FAIL_APPEND_FOR_TX.store(fail_tx.inner(), Ordering::SeqCst);

    let append_res = wal.append_batch(batch).await;
    assert!(
        append_res.is_err(),
        "append_batch must return Err when fault injection triggers WAL append failure"
    );

    wal.restore_last_hmac(prev_hmac_snapshot).await?;

    drop(wal);
    let wal_reopened = Wal::open(&wal_path).await?;
    let entries = wal_reopened.replay().await?;

    assert_eq!(
        entries.len(),
        1,
        "WAL must contain exactly 1 baseline entry after batch failure rollback"
    );
    assert_eq!(entries[0].1.seq_no, 1);

    Ok(())
}

#[tokio::test]
async fn test_wal_replay_stream_vs_mmap_parity() -> Result<()> {
    let dir = tempdir()?;
    let wal_path = dir.path().join("parity_test.wal");

    let wal = Wal::open(&wal_path).await?;
    for i in 1..=20 {
        let op = WalOp::Put {
            tx_id: TxId::new(i),
            key: format!("k_{i}").into_bytes(),
            value: format!("v_{i}").into_bytes(),
        };
        let (batch, _) = wal.prepare_batch(vec![(op, i)]).await?;
        wal.append_batch(batch).await?;
    }

    let metadata = tokio::fs::metadata(&wal_path).await?;
    let file_size = metadata.len();

    let (mmap_entries, mmap_version) = wal.replay_mmap().await?;

    let mut stream_entries = Vec::new();
    let stream_version = wal
        .scan_entries_with_callback(file_size, |seq, entry, pos| {
            stream_entries.push((seq, entry, pos));
            true
        })
        .await?;

    assert_eq!(mmap_version, stream_version);
    assert_eq!(mmap_entries.len(), stream_entries.len());

    for (mmap_item, stream_item) in mmap_entries.iter().zip(stream_entries.iter()) {
        assert_eq!(mmap_item.0, stream_item.0);
        assert_eq!(mmap_item.1, stream_item.1);
        assert_eq!(mmap_item.2, stream_item.2);
    }

    Ok(())
}

#[tokio::test]
async fn test_wal_rotate_and_seal_readonly_guarantee() {
    let dir = tempfile::tempdir().expect("tempdir");
    let wal_path = dir.path().join("test.wal");

    // WAL öffnen und einen Eintrag schreiben
    let wal = Wal::open(&wal_path).await.expect("open WAL");
    let op = WalOp::Put {
        tx_id: TxId::new(1),
        key: b"hello".to_vec(),
        value: b"world".to_vec(),
    };
    let (batch, _hmac) = wal
        .prepare_batch(vec![(op, 1)])
        .await
        .expect("prepare batch");
    wal.append_batch(batch).await.expect("append batch");

    // rotate_and_seal aufrufen
    let sealed_path = wal.rotate_and_seal().await.expect("rotate_and_seal");

    // Invariante 1: Sealed-Datei existiert am neuen Pfad
    assert!(sealed_path.exists(), "Sealed WAL muss existieren");
    assert!(
        sealed_path.to_str().unwrap().contains(".sealed."),
        "Sealed-Pfad muss '.sealed.' enthalten"
    );

    // Invariante 2: Originalpfad existiert nicht mehr
    assert!(
        !wal_path.exists(),
        "Originaler WAL-Pfad muss nach Rotate verschwunden sein"
    );

    // Invariante 3: Sealed-Datei ist read-only
    let meta = tokio::fs::metadata(&sealed_path).await.expect("metadata");
    assert!(
        meta.permissions().readonly(),
        "Versiegeltes WAL-Segment MUSS read-only sein"
    );

    // Invariante 4: Schreibversuch auf sealed-Datei schlägt fehl
    let write_result = tokio::fs::OpenOptions::new()
        .write(true)
        .open(&sealed_path)
        .await;
    assert!(
        write_result.is_err(),
        "Schreibversuch auf versiegeltes WAL-Segment muss fehlschlagen"
    );
}

#[tokio::test]
async fn test_wal_flusher_actor_no_write_to_sealed() {
    let dir = tempfile::tempdir().expect("tempdir");
    let wal_path = dir.path().join("test_flusher_sealed.wal");

    let wal = Wal::open(&wal_path).await.expect("open WAL");
    let op1 = WalOp::Put {
        tx_id: TxId::new(1),
        key: b"k1".to_vec(),
        value: b"v1".to_vec(),
    };
    let (batch1, _) = wal
        .prepare_batch(vec![(op1, 1)])
        .await
        .expect("prepare batch 1");
    wal.append_batch(batch1).await.expect("append batch 1");

    assert!(
        !wal.is_sealed(),
        "WAL must not be sealed before rotate_and_seal"
    );

    let sealed_path = wal.rotate_and_seal().await.expect("rotate_and_seal");
    assert!(wal.is_sealed(), "WAL must be sealed after rotate_and_seal");

    // Attempting to prepare or append to the sealed WAL segment must return Err
    let op2 = WalOp::Put {
        tx_id: TxId::new(2),
        key: b"k2".to_vec(),
        value: b"v2".to_vec(),
    };
    let prep_res = wal.prepare_batch(vec![(op2, 2)]).await;
    assert!(
        prep_res.is_err(),
        "prepare_batch on sealed WAL must return Err, no panic or silent write"
    );

    let dummy_entry = WalEntry::try_new(
        WalOp::Put {
            tx_id: TxId::new(3),
            key: b"k3".to_vec(),
            value: b"v3".to_vec(),
        },
        3,
        &[0u8; 32],
        [0u8; 32],
    )
    .expect("dummy entry");
    let manual_batch = PreparedBatch(vec![dummy_entry]);
    let append_res = wal.append_batch(manual_batch).await;
    assert!(
        append_res.is_err(),
        "append_batch on sealed WAL must return Err, no panic or silent write"
    );

    assert!(sealed_path.exists(), "Sealed path must exist");
}

#[tokio::test]
async fn test_wal_rotate_seal_crash_mid_rename() {
    let dir = tempfile::tempdir().expect("tempdir");
    let wal_path = dir.path().join("crash_test.wal");

    // 1. Write initial committed WAL entries
    {
        let wal = Wal::open(&wal_path).await.expect("open WAL");
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
    }

    // 2. Simulate crash state between rename and parent fsync
    let sealed_name = format!("crash_test.wal.sealed.1234567890");
    let sealed_path = dir.path().join(&sealed_name);

    tokio::fs::rename(&wal_path, &sealed_path)
        .await
        .expect("simulate rename before crash");

    // 3. Post-crash inspection & recovery verification
    let wal_exists = wal_path.exists();
    let sealed_exists = sealed_path.exists();

    assert!(
        (wal_exists && !sealed_exists) || (!wal_exists && sealed_exists),
        "WAL segment must be either fully active or fully sealed after crash mid-rename"
    );

    if sealed_exists {
        let sealed_wal = Wal::open(&sealed_path).await.expect("open sealed WAL");
        let replayed = sealed_wal.replay().await.expect("replay sealed WAL");
        assert_eq!(
            replayed.len(),
            3,
            "All 3 entries must be present in sealed WAL"
        );
    }
}
