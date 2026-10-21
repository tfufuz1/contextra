// FILE-CONTEXT
// ZWECK: Comprehensive test suite for SSTable Format Version 5 with ScopeKeyring encryption (ADR-N27).
// INVARIANTEN: Zero-panic, fail-closed on destroyed/unknown key, AAD block-swap detection, raw canary sweep verification.

#![forbid(unsafe_code)]

use contextra_crypto::{KeyId, KeyManager, KeyScope, ScopeKeyring};
use contextra_mvcc::snapshot::SnapshotFloor;
use contextra_ports::SystemRng;
use contextra_store::compaction::{CompactionConfig, CompactionEngine};
use contextra_store::sstable::{
    create_block_cache, scan_sstables_with_keyring, SstScanInputs, SstScopeSpec, SstableBuilder,
    SstableReader, WitnessReadError, WitnessReader,
};
use contextra_testkit::ManualClock;
use contextra_types::{CollectionId, ContextraError, TenantId};
use std::fs;
use std::sync::Arc;
use tempfile::tempdir;

fn setup_keyring(dir: &std::path::Path) -> (Arc<ScopeKeyring>, KeyScope, KeyId) {
    let (km, _) = KeyManager::try_new_argon2id("test_master_passphrase_32_bytes!").unwrap();
    let clock = Arc::new(ManualClock::new(1_000_000_000));
    let rng = Arc::new(SystemRng::new());
    let keyring = Arc::new(ScopeKeyring::open(dir, &km, clock, rng).unwrap());

    let scope = KeyScope::Collection {
        tenant: TenantId::try_new(1).unwrap(),
        collection: CollectionId::new(100),
        incarnation: 1,
    };

    let (key_id, _handle) = keyring.create_dek(scope.clone()).unwrap();
    (keyring, scope, key_id)
}

#[tokio::test]
async fn test_v5_roundtrip_and_v4_backward_compatibility() {
    let tmp = tempdir().unwrap();
    let (keyring, _scope, key_id) = setup_keyring(tmp.path());
    let cache = create_block_cache(1);

    // 1. Write Format v5 SSTable with ScopeKeyring
    let v5_path = tmp.path().join("v5_data.sst");
    let mut builder5 = SstableBuilder::create_with_scope_keyring(&v5_path, keyring.clone(), key_id)
        .await
        .unwrap();

    builder5
        .add(b"user:001", b"value_alpha", 10, 1)
        .await
        .unwrap();
    builder5
        .add(b"user:002", b"value_beta", 20, 2)
        .await
        .unwrap();
    builder5.finish().await.unwrap();

    // 2. Read back Format v5 SSTable
    let reader5 =
        SstableReader::open_with_keyring(&v5_path, Arc::clone(&cache), Some(keyring.clone()), None)
            .await
            .unwrap();

    assert_eq!(reader5.format_version, 5);
    assert_eq!(reader5.key_id(), Some(key_id));

    let val1 = reader5.get(b"user:001").await.unwrap().unwrap();
    assert_eq!(val1.0.as_ref(), b"value_alpha");

    let val2 = reader5.get(b"user:002").await.unwrap().unwrap();
    assert_eq!(val2.0.as_ref(), b"value_beta");

    // 3. Write Format v4 SSTable (unencrypted legacy)
    let v4_path = tmp.path().join("v4_data.sst");
    let mut builder4 = SstableBuilder::create(&v4_path).await.unwrap();
    builder4
        .add(b"legacy:001", b"value_legacy", 5, 1)
        .await
        .unwrap();
    builder4.finish().await.unwrap();

    // 4. Read back Format v4 SSTable using open_with_keyring
    let reader4 =
        SstableReader::open_with_keyring(&v4_path, Arc::clone(&cache), Some(keyring), None)
            .await
            .unwrap();

    assert_eq!(reader4.format_version, 4);
    assert_eq!(reader4.key_id(), None);

    let val_legacy = reader4.get(b"legacy:001").await.unwrap().unwrap();
    assert_eq!(val_legacy.0.as_ref(), b"value_legacy");
}

#[tokio::test]
async fn test_block_swap_detection_via_aad() {
    let tmp = tempdir().unwrap();
    let (keyring, _scope, key_id) = setup_keyring(tmp.path());
    let cache = create_block_cache(1);

    let sst_path = tmp.path().join("multi_block.sst");
    let mut builder = SstableBuilder::create_with_scope_keyring(&sst_path, keyring.clone(), key_id)
        .await
        .unwrap();

    // Add enough entries to span multiple blocks
    let large_val = vec![b'X'; 3000];
    for i in 0..10 {
        let key = format!("key_{:04}", i);
        builder
            .add(key.as_bytes(), &large_val, i as u64 + 1, 1)
            .await
            .unwrap();
    }
    builder.finish().await.unwrap();

    // Verify normal reader works before tampering
    let reader_ok = SstableReader::open_with_keyring(
        &sst_path,
        Arc::clone(&cache),
        Some(keyring.clone()),
        None,
    )
    .await
    .unwrap();
    assert!(reader_ok.get(b"key_0000").await.unwrap().is_some());

    // Tamper with file: Swap block 0 payload with block 1 payload on disk
    let index_offsets = reader_ok.index().to_vec();
    assert!(index_offsets.len() >= 2, "Expected at least 2 blocks");

    let off0 = index_offsets[0].1;
    let off1 = index_offsets[1].1;
    let off2 = if index_offsets.len() > 2 {
        index_offsets[2].1
    } else {
        reader_ok.index_offset()
    };

    let len0 = (off1 - off0) as usize;
    let len1 = (off2 - off1) as usize;

    let mut raw_bytes = fs::read(&sst_path).unwrap();

    // Swap block bytes
    let block0 = raw_bytes[off0 as usize..(off0 as usize + len0)].to_vec();
    let block1 = raw_bytes[off1 as usize..(off1 as usize + len1)].to_vec();

    // Copy block1 payload into block0 position and block0 payload into block1 position
    let min_len = len0.min(len1);
    raw_bytes[off0 as usize..(off0 as usize + min_len)].copy_from_slice(&block1[..min_len]);
    raw_bytes[off1 as usize..(off1 as usize + min_len)].copy_from_slice(&block0[..min_len]);

    let swapped_path = tmp.path().join("swapped_blocks.sst");
    fs::write(&swapped_path, &raw_bytes).unwrap();

    // Open swapped SSTable and attempt read — opening or reading must fail closed due to AAD mismatch
    let reader_swapped_res =
        SstableReader::open_with_keyring(&swapped_path, create_block_cache(1), Some(keyring), None)
            .await;

    match reader_swapped_res {
        Ok(reader) => {
            let res = reader.get(b"key_0000").await;
            assert!(
                res.is_err(),
                "Reading block with mismatched offset AAD must return error"
            );
        }
        Err(err) => {
            // Decryption failure during open_with_keyring (read_first_key) is also expected and valid
            assert!(
                matches!(
                    err,
                    ContextraError::Crypto(_)
                        | ContextraError::Storage(_)
                        | ContextraError::ParseError(_)
                ),
                "Expected crypto or storage error, got {:?}",
                err
            );
        }
    }
}

#[tokio::test]
async fn test_scope_keyring_destroy_yields_key_destroyed_and_no_canary() {
    let tmp = tempdir().unwrap();
    let (keyring, scope, key_id) = setup_keyring(tmp.path());
    let cache = create_block_cache(1);

    let canary_secret = b"CANARY_CONFIDENTIAL_DSGVO_PURGE_TARGET_998877";

    let sst_path = tmp.path().join("canary_data.sst");
    let mut builder = SstableBuilder::create_with_scope_keyring(&sst_path, keyring.clone(), key_id)
        .await
        .unwrap();

    builder
        .add(b"doc:999", canary_secret, 1, 100)
        .await
        .unwrap();
    builder.finish().await.unwrap();

    // Verify reader works prior to destruction
    let reader_pre = SstableReader::open_with_keyring(
        &sst_path,
        Arc::clone(&cache),
        Some(keyring.clone()),
        None,
    )
    .await
    .unwrap();
    let pre_val = reader_pre.get(b"doc:999").await.unwrap().unwrap();
    assert_eq!(pre_val.0.as_ref(), canary_secret);

    // Verify WitnessReader works prior to destruction
    let mut witness_pre =
        WitnessReader::open_with_keyring(&sst_path, None, Some(keyring.clone())).unwrap();
    let pre_entries = witness_pre.read_all_entries().unwrap();
    assert_eq!(pre_entries.len(), 1);
    assert_eq!(pre_entries[0].value.as_ref(), canary_secret);

    // Execute Crypto-Erase on KeyScope
    let receipt = keyring.destroy(&scope).unwrap();
    assert!(receipt.key_ids.contains(&key_id));

    // 1. Reader open or get must return Err(KeyDestroyed)
    let reader_post_res = SstableReader::open_with_keyring(
        &sst_path,
        create_block_cache(1),
        Some(keyring.clone()),
        None,
    )
    .await;
    match reader_post_res {
        Ok(reader) => {
            let post_res = reader.get(b"doc:999").await;
            assert!(
                matches!(post_res, Err(ContextraError::KeyDestroyed(_))),
                "Read after key destruction must return ContextraError::KeyDestroyed, got {:?}",
                post_res
            );
        }
        Err(err) => {
            assert!(
                matches!(err, ContextraError::KeyDestroyed(_)),
                "Open after key destruction must return ContextraError::KeyDestroyed, got {:?}",
                err
            );
        }
    }

    // 2. WitnessReader must return Err(WitnessReadError::KeyDestroyed("unlesbar (Schlüssel vernichtet)..."))
    let witness_post_res = WitnessReader::open_with_keyring(&sst_path, None, Some(keyring.clone()));
    assert!(
        matches!(witness_post_res, Err(WitnessReadError::KeyDestroyed(_))),
        "WitnessReader after key destruction must return WitnessReadError::KeyDestroyed, got {:?}",
        witness_post_res
    );

    // 3. Scan SSTables witness pipeline must report KeyDestroyed
    let sst_inputs = SstScanInputs {
        data_dirs: vec![tmp.path().to_path_buf()],
        live_files: vec![sst_path.clone()],
        pending_delete_files: Vec::new(),
        manifest_generation: 1,
        as_of_tx: 1000,
        depth: 1,
    };
    let scan_res = scan_sstables_with_keyring(
        &SstScopeSpec::default(),
        &sst_inputs,
        None,
        Some(keyring.clone()),
        || 1,
    );
    assert!(
        matches!(scan_res, Err(WitnessReadError::KeyDestroyed(_))),
        "scan_sstables_with_keyring must fail-closed with KeyDestroyed after crypto-erase, got {:?}",
        scan_res
    );

    // 4. Raw file byte sweep must yield ZERO occurrences of the plaintext canary
    let raw_file_bytes = fs::read(&sst_path).unwrap();
    let canary_found = raw_file_bytes
        .windows(canary_secret.len())
        .any(|window| window == canary_secret);

    assert!(
        !canary_found,
        "CRITICAL SECURITY INVARIANT VIOLATION: Plaintext canary found in raw SSTable byte sweep!"
    );
}

#[tokio::test]
async fn test_compaction_isolation_across_scopes() {
    let tmp = tempdir().unwrap();
    let (km, _) = KeyManager::try_new_argon2id("test_compaction_master_passphrase").unwrap();
    let clock = Arc::new(ManualClock::new(1_000_000_000));
    let rng = Arc::new(SystemRng::new());
    let keyring = Arc::new(ScopeKeyring::open(tmp.path(), &km, clock, rng).unwrap());

    let scope_a = KeyScope::Collection {
        tenant: TenantId::try_new(1).unwrap(),
        collection: CollectionId::new(10),
        incarnation: 1,
    };
    let scope_b = KeyScope::Collection {
        tenant: TenantId::try_new(1).unwrap(),
        collection: CollectionId::new(20),
        incarnation: 1,
    };

    let (key_a, _) = keyring.create_dek(scope_a.clone()).unwrap();
    let (key_b, _) = keyring.create_dek(scope_b.clone()).unwrap();

    let cache = create_block_cache(1);

    // Write input SSTable A under Scope A
    let sst_a_path = tmp.path().join("scope_a_in.sst");
    let mut builder_a =
        SstableBuilder::create_with_scope_keyring(&sst_a_path, keyring.clone(), key_a)
            .await
            .unwrap();
    builder_a
        .add(b"col_a_key1", b"secret_a1", 1, 1)
        .await
        .unwrap();
    builder_a.finish().await.unwrap();

    let reader_a = Arc::new(
        SstableReader::open_with_keyring(
            &sst_a_path,
            Arc::clone(&cache),
            Some(keyring.clone()),
            None,
        )
        .await
        .unwrap(),
    );

    // Write input SSTable B under Scope B
    let sst_b_path = tmp.path().join("scope_b_in.sst");
    let mut builder_b =
        SstableBuilder::create_with_scope_keyring(&sst_b_path, keyring.clone(), key_b)
            .await
            .unwrap();
    builder_b
        .add(b"col_b_key1", b"secret_b1", 1, 1)
        .await
        .unwrap();
    builder_b.finish().await.unwrap();

    let reader_b = Arc::new(
        SstableReader::open_with_keyring(
            &sst_b_path,
            Arc::clone(&cache),
            Some(keyring.clone()),
            None,
        )
        .await
        .unwrap(),
    );

    let floor = Arc::new(contextra_mvcc::SnapshotRegistry::new()) as Arc<dyn SnapshotFloor>;

    // Compact Scope A using engine configured with Scope A keyring + key_id
    let engine_a = CompactionEngine::new(
        CompactionConfig::default(),
        floor.clone(),
        Arc::clone(&cache),
        None,
        Arc::new(contextra_types::ResourceTracker::new(
            contextra_types::ResourceBudget {
                memory_limit: 1024 * 1024,
            },
        )),
        None,
    )
    .with_scope_keyring(keyring.clone(), key_a);

    let output_a_path = tmp.path().join("scope_a_compact.sst");
    engine_a
        .merge_sstables_with_cancel(&[reader_a.clone()], &output_a_path, 0, false, None)
        .await
        .unwrap();

    // Compact Scope B using engine configured with Scope B keyring + key_id
    let engine_b = CompactionEngine::new(
        CompactionConfig::default(),
        floor,
        Arc::clone(&cache),
        None,
        Arc::new(contextra_types::ResourceTracker::new(
            contextra_types::ResourceBudget {
                memory_limit: 1024 * 1024,
            },
        )),
        None,
    )
    .with_scope_keyring(keyring.clone(), key_b);

    let output_b_path = tmp.path().join("scope_b_compact.sst");
    engine_b
        .merge_sstables_with_cancel(&[reader_b.clone()], &output_b_path, 0, false, None)
        .await
        .unwrap();

    // Verify attempting to merge reader_a and reader_b together in a single call fails
    let mixed_output_path = tmp.path().join("mixed_output.sst");
    let mixed_res = engine_a
        .merge_sstables_with_cancel(&[reader_a, reader_b], &mixed_output_path, 0, false, None)
        .await;
    assert!(
        matches!(mixed_res, Err(ContextraError::InvalidInput(_))),
        "Merging SSTables from different key scopes into one file must be rejected fail-closed"
    );

    // Destroy Scope A
    keyring.destroy(&scope_a).unwrap();

    // Verify Scope A compacted file is unreadable (KeyDestroyed)
    let reader_a_compact_res = SstableReader::open_with_keyring(
        &output_a_path,
        create_block_cache(1),
        Some(keyring.clone()),
        None,
    )
    .await;
    match reader_a_compact_res {
        Ok(reader) => {
            let res_a = reader.get(b"col_a_key1").await;
            assert!(
                matches!(res_a, Err(ContextraError::KeyDestroyed(_))),
                "Scope A output after destruction must return KeyDestroyed"
            );
        }
        Err(err) => {
            assert!(
                matches!(err, ContextraError::KeyDestroyed(_)),
                "Scope A open after destruction must return KeyDestroyed, got {:?}",
                err
            );
        }
    }

    // Verify Scope B compacted file remains fully readable!
    let reader_b_compact = SstableReader::open_with_keyring(
        &output_b_path,
        create_block_cache(1),
        Some(keyring),
        None,
    )
    .await
    .unwrap();
    let val_b = reader_b_compact.get(b"col_b_key1").await.unwrap().unwrap();
    assert_eq!(val_b.0.as_ref(), b"secret_b1");
}

#[tokio::test]
async fn test_corrupt_header_or_unknown_key_id_fails() {
    let tmp = tempdir().unwrap();
    let (keyring, _scope, key_id) = setup_keyring(tmp.path());
    let cache = create_block_cache(1);

    let sst_path = tmp.path().join("unknown_key.sst");
    let mut builder = SstableBuilder::create_with_scope_keyring(&sst_path, keyring.clone(), key_id)
        .await
        .unwrap();

    builder.add(b"k1", b"v1", 1, 1).await.unwrap();
    builder.finish().await.unwrap();

    // Create a different keyring without key_id
    let (km2, _) = KeyManager::try_new_argon2id("different_passphrase_32_bytes!!").unwrap();
    let clock2 = Arc::new(ManualClock::new(1_000_000_000));
    let rng2 = Arc::new(SystemRng::new());
    let keyring2 =
        Arc::new(ScopeKeyring::open(tmp.path().join("kr2"), &km2, clock2, rng2).unwrap());

    // Reading with keyring2 where key_id does not exist must fail closed
    let reader_res =
        SstableReader::open_with_keyring(&sst_path, Arc::clone(&cache), Some(keyring2), None).await;
    match reader_res {
        Ok(reader) => {
            let res = reader.get(b"k1").await;
            assert!(res.is_err());
        }
        Err(err) => {
            assert!(
                matches!(err, ContextraError::Crypto(_)),
                "Expected Crypto error for unknown key_id, got {:?}",
                err
            );
        }
    }
}
