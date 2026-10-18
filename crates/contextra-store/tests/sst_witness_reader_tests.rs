// FILE-CONTEXT
// STAND: 2026-10-08 (T-2026-9104)
// ZWECK: Differenzialtests und Fehlerbehandlungstests für den SSTable WitnessReader.

use bytes::Bytes;
use contextra_store::sstable::{
    BlockCache, SstableBuilder, SstableReader, WitnessReadError, WitnessReader,
};
use contextra_store::wal::KeyManager;
use rand::Rng;
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn test_witness_reader_differential_against_production_reader() {
    let mut rng = rand::thread_rng();

    for run in 0..100 {
        let dir = tempdir().expect("tempdir");
        let sst_path = dir.path().join(format!("diff_test_{run}.sst"));

        let version = if run % 2 == 0 { 4 } else { 3 };
        let mut builder = SstableBuilder::create_with_version(&sst_path, version)
            .await
            .expect("create builder");

        let entry_count = rng.gen_range(5..150);
        let mut expected_entries = Vec::new();

        for i in 0..entry_count {
            let key_str = format!("key_{:05}_{i}", rng.gen_range(0..1000));
            let val_str = format!("value_{:08}_{i}", rng.gen_range(0..10000));
            let raw_seq = (i as u64) + 1;
            let is_tombstone = rng.gen_bool(0.15);
            let seq = if is_tombstone {
                raw_seq | contextra_types::TOMBSTONE_BIT
            } else {
                raw_seq
            };
            let tx_id = (i as u64) * 2;

            let key_bytes = key_str.as_bytes();
            let val_bytes = val_str.as_bytes();

            if builder.add(key_bytes, val_bytes, seq, tx_id).await.is_ok() {
                expected_entries.push((
                    Bytes::copy_from_slice(key_bytes),
                    Bytes::copy_from_slice(val_bytes),
                    seq,
                    tx_id,
                    is_tombstone,
                ));
            }
        }

        if expected_entries.is_empty() {
            continue;
        }

        // Sort expected entries by key ASC, raw_seq DESC (SSTable ordering)
        expected_entries.sort_by(|a, b| {
            let key_cmp = a.0.cmp(&b.0);
            if key_cmp == std::cmp::Ordering::Equal {
                let seq_a = a.2 & !contextra_types::TOMBSTONE_BIT;
                let seq_b = b.2 & !contextra_types::TOMBSTONE_BIT;
                seq_b.cmp(&seq_a)
            } else {
                key_cmp
            }
        });

        let _meta = builder.finish().await.expect("finish builder");

        // Read using production SstableReader
        let cache = Arc::new(BlockCache::new(8 * 1024 * 1024));
        let prod_reader = SstableReader::open(&sst_path, cache)
            .await
            .expect("open prod reader");
        let prod_entries = prod_reader.iter().await.expect("prod iter");

        // Read using WitnessReader
        let mut witness_reader = WitnessReader::open(&sst_path, None).expect("open witness reader");
        let witness_first_key = witness_reader
            .read_first_key()
            .expect("witness read_first_key");
        let witness_entries = witness_reader
            .read_all_entries()
            .expect("witness read_all_entries");

        assert_eq!(
            prod_entries.len(),
            witness_entries.len(),
            "Run {run}: Entry count mismatch between prod and witness reader"
        );

        assert_eq!(
            witness_reader.metadata().first_key,
            prod_reader.first_key(),
            "Run {run}: First key mismatch"
        );
        assert_eq!(
            witness_first_key,
            prod_reader.first_key(),
            "Run {run}: Explicit read_first_key mismatch"
        );
        assert_eq!(
            witness_reader.metadata().last_key,
            prod_reader.last_key(),
            "Run {run}: Last key mismatch"
        );

        for (idx, ((p_key, p_val, p_seq), w_entry)) in
            prod_entries.iter().zip(witness_entries.iter()).enumerate()
        {
            assert_eq!(p_key, &w_entry.key, "Run {run} idx {idx}: Key mismatch");
            assert_eq!(p_val, &w_entry.value, "Run {run} idx {idx}: Value mismatch");
            assert_eq!(*p_seq, w_entry.seq_no, "Run {run} idx {idx}: Seq mismatch");
            assert_eq!(
                (*p_seq & contextra_types::TOMBSTONE_BIT) != 0,
                w_entry.is_tombstone,
                "Run {run} idx {idx}: Tombstone mismatch"
            );
        }
    }
}

#[tokio::test]
async fn test_witness_reader_corrupt_file_handling() {
    let dir = tempdir().expect("tempdir");

    // Case 1: Corrupt magic bytes
    let bad_magic_path = dir.path().join("bad_magic.sst");
    let mut data = vec![0u8; 100];
    data[96..100].copy_from_slice(&0xDEADBEEFu32.to_le_bytes());
    tokio::fs::write(&bad_magic_path, &data)
        .await
        .expect("write bad magic");

    let res = WitnessReader::open(&bad_magic_path, None);
    assert!(res.is_err());
    let err = match res {
        Err(e) => e,
        Ok(_) => panic!("Expected error for bad magic"),
    };
    assert!(
        matches!(err, WitnessReadError::CorruptFile(_)),
        "Expected CorruptFile for bad magic, got: {err}"
    );

    // Case 2: Truncated file (< 12 bytes)
    let truncated_path = dir.path().join("truncated.sst");
    tokio::fs::write(&truncated_path, b"too_short")
        .await
        .expect("write truncated");
    let res_trunc = WitnessReader::open(&truncated_path, None);
    assert!(res_trunc.is_err());
    let err_trunc = match res_trunc {
        Err(e) => e,
        Ok(_) => panic!("Expected error for truncated file"),
    };
    assert!(
        matches!(err_trunc, WitnessReadError::CorruptFile(_)),
        "Expected CorruptFile for truncated file, got: {err_trunc}"
    );
}

#[tokio::test]
async fn test_witness_reader_encrypted_sstable_handling() {
    let dir = tempdir().expect("tempdir");
    let enc_path = dir.path().join("encrypted.sst");

    let km = Arc::new(
        KeyManager::try_new("test_secret_passphrase", b"salt_12345678901234567890123456")
            .expect("create key manager"),
    );
    let mut builder = SstableBuilder::create_with_key_manager(&enc_path, Some(Arc::clone(&km)))
        .await
        .expect("create enc builder");

    builder
        .add(b"secret_key_123", b"secret_value_456", 1, 100)
        .await
        .expect("add enc");
    builder.finish().await.expect("finish enc");

    // 1. Reading with correct key manager succeeds
    let mut enc_reader =
        WitnessReader::open(&enc_path, Some(Arc::clone(&km))).expect("open with km");
    let entries = enc_reader.read_all_entries().expect("read enc entries");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].key.as_ref(), b"secret_key_123");
    assert_eq!(entries[0].value.as_ref(), b"secret_value_456");

    // 2. Reading without key manager detects encryption and returns NotWitnessable on entry read
    let mut unauth_reader = WitnessReader::open(&enc_path, None).expect("open without km");
    assert!(unauth_reader.is_encrypted());

    let unauth_err = unauth_reader.read_all_entries().unwrap_err();
    assert!(
        matches!(unauth_err, WitnessReadError::NotWitnessable(_)),
        "Expected NotWitnessable when reading encrypted SSTable without key, got: {unauth_err}"
    );
}

#[test]
fn test_witness_no_key_bytes_in_error_or_debug() {
    let secret_key = b"SUPER_SECRET_USER_KEY_999";
    let secret_val = b"SUPER_SECRET_USER_VAL_111";

    let entry = contextra_store::sstable::WitnessEntry {
        key: Bytes::copy_from_slice(secret_key),
        value: Bytes::copy_from_slice(secret_val),
        seq_no: 42,
        tx_id: 10,
        is_tombstone: false,
    };

    let debug_str = format!("{entry:?}");
    assert!(!debug_str.contains("SUPER_SECRET"));
    assert!(debug_str.contains("key_len: 25"));
    assert!(debug_str.contains("val_len: 25"));

    let meta = contextra_store::sstable::WitnessSstMetadata {
        first_key: Bytes::copy_from_slice(secret_key),
        last_key: Bytes::copy_from_slice(secret_key),
        file_size: 1024,
        min_tx_id: 1,
        max_tx_id: 10,
        min_seq: 1,
        max_seq: 42,
        format_version: 4,
    };

    let meta_debug = format!("{meta:?}");
    assert!(!meta_debug.contains("SUPER_SECRET"));
    assert!(meta_debug.contains("first_key_len: 25"));
}
