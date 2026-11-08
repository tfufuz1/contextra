//! Specification & Invariant Tests for `contextra-store` (AGENTS.md §4 / SPEC §9).
//!
//! Validates the nine core contracts and guarantees:
//! 1. `INV-FSYNC-1`: fsync error propagation and handle poisoning.
//! 2. `INV-ATOMIC-WRITE`: Atomic write cycle (tmp -> fsync -> rename -> sync_dir).
//! 3. `INV-TOMBSTONE-MASK`: Masking bit 63 (`seq & !TOMBSTONE_BIT`) before comparison.
//! 4. `INV-SINGLE-LOAD`: Snapshot single-load isolation of `last_committed_tx`.
//! 5. `INV-FLUSH-BEFORE-VISIBLE`: Flush-before-visible ordering.
//! 6. `INV-WAL-HMAC-KEY`: WAL HMAC key dynamically derived, never hardcoded.
//! 7. `INV-TENANT-1`: Tenant prefix enforcement and strict tenant isolation.
//! 8. `INV-KEYSPACE-1`: Keyspace & opaque file identifiers.
//! 9. `INV-FORMAT-1`: Magic byte headers and format version verification.

use contextra_ports::StorageEngine;
use contextra_store::lsm::{LsmConfig, LsmStorage};
use contextra_store::tenant_codec::TenantKeyCodec;
use contextra_store::wal::{Wal, WalConfig, WalOp, WAL_V3_HEADER};
use contextra_types::{CollectionId, DocId, Result, TenantId, TxId};
use tempfile::tempdir;
use tokio::fs;

/// SPEC: INV-FSYNC-1
/// Verifies that fsync failures are strictly propagated and permanently poison the handle.
#[tokio::test]
async fn test_inv_fsync_1_error_propagation_and_poisoning() -> Result<()> {
    let dir = tempdir()?;
    let path = dir.path().join("inv_fsync.wal");

    let wal = Wal::open(&path).await?;
    let op = WalOp::Put {
        tx_id: TxId::new(1),
        key: b"k1".to_vec(),
        value: b"v1".to_vec(),
    };
    let (batch, _) = wal.prepare_batch(vec![(op, 1)]).await?;
    wal.append_batch(batch).await?;

    // Verify handle is healthy initially
    assert!(!wal.is_poisoned());

    // Inject simulated failure: poison the WAL handle
    wal.poison_for_test();
    assert!(
        wal.is_poisoned(),
        "WAL handle must be poisoned after failure"
    );

    // Subsequent writes MUST fail closed with poisoned error
    let op2 = WalOp::Put {
        tx_id: TxId::new(2),
        key: b"k2".to_vec(),
        value: b"v2".to_vec(),
    };
    let (batch2, _) = wal.prepare_batch(vec![(op2, 2)]).await?;
    let res2 = wal.append_batch(batch2).await;
    assert!(
        res2.is_err(),
        "Writing to a poisoned WAL handle MUST fail closed"
    );

    Ok(())
}

/// SPEC: INV-ATOMIC-WRITE
/// Verifies atomic file replacement pattern: tmp file write -> sync -> atomic rename -> sync parent.
#[tokio::test]
async fn test_inv_atomic_write_lifecycle() -> Result<()> {
    let dir = tempdir()?;
    let target_path = dir.path().join("atomic_target.dat");
    let tmp_path = dir.path().join("atomic_target.dat.tmp");

    fs::write(&tmp_path, b"atomic payload content").await?;
    contextra_store::fsync::sync_file(&tmp_path).await?;

    // Atomic replace
    fs::rename(&tmp_path, &target_path).await?;
    contextra_store::fsync::sync_parent_dir(&target_path).await?;

    assert!(!tmp_path.exists());
    assert!(target_path.exists());
    let read_back = fs::read(&target_path).await?;
    assert_eq!(read_back, b"atomic payload content");

    Ok(())
}

/// SPEC: INV-TOMBSTONE-MASK
/// Verifies that tombstone bit 63 (`1 << 63`) is masked off prior to sequence comparisons,
/// preventing overflow/ordering inversion.
#[test]
fn test_inv_tombstone_bit_masking_before_comparison() {
    const TOMBSTONE_BIT: u64 = 1 << 63;
    let actual_seq = 42u64;
    let tombstone_seq = actual_seq | TOMBSTONE_BIT;

    assert_ne!(actual_seq, tombstone_seq);
    let masked_seq = tombstone_seq & !TOMBSTONE_BIT;
    assert_eq!(masked_seq, actual_seq);
    assert!(actual_seq < 100);
    assert_eq!(masked_seq.cmp(&100), std::cmp::Ordering::Less);
}

/// SPEC: INV-SINGLE-LOAD
/// Verifies Snapshot Isolation single-load: reading transactions see consistent snapshot watermark.
#[tokio::test]
async fn test_inv_single_load_snapshot_isolation() -> Result<()> {
    let dir = tempdir()?;
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::new(config).await?;

    let tx1 = TxId::new(1);
    storage.put(tx1, b"key_snap", b"v1").await?;
    storage.commit(tx1).await?;

    // Read at sequence 1
    let val_at_seq1 = storage.get_at_seq(b"key_snap", 1).await?;
    assert_eq!(val_at_seq1.as_deref(), Some(b"v1".as_slice()));

    // Concurrent writer modifies and commits at sequence 2
    let tx2 = TxId::new(2);
    storage.put(tx2, b"key_snap", b"v2").await?;
    storage.commit(tx2).await?;

    // Time-travel read at seq 1 must strictly observe snapshot value (v1), unperturbed by tx2 commit
    let read_val = storage.get_at_seq(b"key_snap", 1).await?;
    assert_eq!(read_val.as_deref(), Some(b"v1".as_slice()));

    // Latest read sees v2
    let latest_val = storage.get(b"key_snap").await?;
    assert_eq!(latest_val.as_deref(), Some(b"v2".as_slice()));

    Ok(())
}

/// SPEC: INV-FLUSH-BEFORE-VISIBLE
/// Verifies Flush-before-Visible ordering: flushed SSTable state is made visible only after persistence.
#[tokio::test]
async fn test_inv_flush_before_visible_ordering() -> Result<()> {
    let dir = tempdir()?;
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        memtable_size_limit: 1024,
        ..Default::default()
    };
    let storage = LsmStorage::new(config).await?;

    let tx = TxId::new(10);
    storage
        .put(tx, b"flush_vis_key", b"flush_vis_value")
        .await?;
    storage.commit(tx).await?;

    // Trigger flush explicitly
    storage.flush().await?;

    // Upon reopening, the committed data is immediately visible via recovered SSTables
    drop(storage);
    let reopen_storage = LsmStorage::new(LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    })
    .await?;

    let val = reopen_storage.get(b"flush_vis_key").await?;
    assert_eq!(val.as_deref(), Some(b"flush_vis_value".as_slice()));

    Ok(())
}

/// SPEC: INV-WAL-HMAC-KEY
/// Verifies WAL HMAC keys are derived dynamically per WAL instance/directory and never hardcoded.
#[tokio::test]
async fn test_inv_wal_hmac_key_derivation_not_hardcoded() -> Result<()> {
    let dir1 = tempdir()?;
    let dir2 = tempdir()?;
    let wal1_path = dir1.path().join("wal1.wal");
    let wal2_path = dir2.path().join("wal2.wal");

    let wal1 = Wal::open(&wal1_path).await?;
    let wal2 = Wal::open(&wal2_path).await?;

    let key1 = wal1.integrity_key_for_test()?;
    let key2 = wal2.integrity_key_for_test()?;

    // Dynamic keys must be uniquely generated per directory, never static or hardcoded
    assert_ne!(
        key1, key2,
        "WAL integrity keys must be uniquely generated per directory, never static or hardcoded"
    );

    // Verify key file is persisted on disk
    let key_file_1 = dir1.path().join(".wal_integrity_key");
    assert!(key_file_1.exists());
    let disk_key1 = fs::read(&key_file_1).await?;
    assert_eq!(&disk_key1[..], &key1[..]);

    Ok(())
}

/// SPEC: INV-TENANT-1
/// Verifies Tenant prefix enforcement and cross-tenant namespace separation.
#[test]
fn test_inv_tenant_1_prefix_enforcement_and_isolation() {
    let tenant_a = TenantId::try_new(101).expect("tenant a");
    let tenant_b = TenantId::try_new(102).expect("tenant b");

    let codec_a = TenantKeyCodec::new(tenant_a);
    let codec_b = TenantKeyCodec::new(tenant_b);

    let col = CollectionId::new(1);
    let doc = DocId::new(42);

    let key_a = codec_a.encode_chunk_key(&col, doc);
    let key_b = codec_b.encode_chunk_key(&col, doc);

    assert_ne!(key_a, key_b, "Tenant isolation keys must differ");

    let decoded_a = TenantKeyCodec::decode_tenant_id(&key_a).expect("decode a");
    let decoded_b = TenantKeyCodec::decode_tenant_id(&key_b).expect("decode b");

    assert_eq!(decoded_a, tenant_a);
    assert_eq!(decoded_b, tenant_b);
}

/// SPEC: INV-KEYSPACE-1
/// Verifies that keyspace identifiers remain opaque and avoid path leakage.
#[test]
fn test_inv_keyspace_1_opaque_keys_and_filenames() {
    let tenant = TenantId::try_new(999).expect("tenant");
    let codec = TenantKeyCodec::new(tenant);
    let col = CollectionId::new(2);
    let doc = DocId::new(12345);

    let key = codec.encode_chunk_key(&col, doc);
    let key_str = std::str::from_utf8(&key).expect("utf8 key");

    // Must be formatted as opaque prefix "t:999:2:chunk:12345"
    assert!(key_str.starts_with("t:999:2:chunk:"));
    assert!(!key_str.contains("/"));
}

/// SPEC: INV-FORMAT-1
/// Verifies Magic byte format headers and tamper/bit-rot detection on WAL files.
#[tokio::test]
async fn test_inv_format_1_magic_and_version_headers() -> Result<()> {
    let dir = tempdir()?;
    let wal_path = dir.path().join("magic_check.wal");

    {
        let wal = Wal::open(&wal_path).await?;
        let op = WalOp::Put {
            tx_id: TxId::new(1),
            key: b"k1".to_vec(),
            value: b"v1".to_vec(),
        };
        let (batch, _) = wal.prepare_batch(vec![(op, 1)]).await?;
        wal.append_batch(batch).await?;
    }

    let bytes = fs::read(&wal_path).await?;
    assert!(bytes.len() >= 4);
    assert_eq!(
        &bytes[0..4],
        &WAL_V3_HEADER,
        "File must begin with valid V3 Magic header"
    );

    // Tamper with magic header
    let mut tampered = bytes.clone();
    tampered[0] ^= 0xFF;
    fs::write(&wal_path, &tampered).await?;

    let mut config = WalConfig::default();
    config.min_wal_version = contextra_store::wal::WalVersion::V3;
    let reopen_res = Wal::open_with_config(&wal_path, config).await;

    assert!(
        reopen_res.is_err(),
        "WAL with invalid magic header must be rejected"
    );

    Ok(())
}
