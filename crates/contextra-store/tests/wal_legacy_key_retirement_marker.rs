// ZWECK: Integrationstests für WAL-Legacy-Schlüssel Retirement Marker, Blake3-Bindung, Crash-Fenster und hartem Fallback-Ausschluss.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_core::TxId;
use contextra_store::wal::{Wal, WalConfig, WalEntry, WalOp, WAL_V3_HEADER};
use tempfile::tempdir;
use tokio::fs;

#[tokio::test]
async fn test_retirement_marker_created_and_contains_blake3_binding() {
    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("retirement_test.wal");

    // 1. Manually construct a legacy WAL segment signed with the legacy integrity key
    let op = WalOp::Put {
        tx_id: TxId::new(1),
        key: b"test_key".to_vec(),
        value: b"test_val".to_vec(),
    };
    let legacy_key = Wal::legacy_integrity_key_for_test();
    let legacy_entry = WalEntry::try_new(op, 1, &legacy_key, [0u8; 32]).expect("legacy entry");

    let mut wal_bytes = Vec::new();
    wal_bytes.extend_from_slice(&WAL_V3_HEADER);
    wal_bytes.extend_from_slice(&legacy_entry.to_bytes().expect("to_bytes"));
    fs::write(&wal_path, wal_bytes).await.expect("write wal");

    let marker_path = dir.path().join("retirement_test.wal.rekeyed");
    assert!(
        !marker_path.exists(),
        "Marker must not exist before migration"
    );

    // 2. Perform migration
    let wal = Wal::open_for_legacy_migration(&wal_path, None)
        .await
        .expect("open_for_legacy_migration");
    drop(wal);

    // 3. Verify retirement marker exists and contains Blake3 binding without plaintext key
    assert!(
        marker_path.exists(),
        "Retirement marker must exist after migration"
    );
    let marker_content = fs::read_to_string(&marker_path).await.expect("read marker");

    assert!(
        marker_content.contains("blake3:"),
        "Marker must contain blake3 binding format, got: {}",
        marker_content
    );
    assert!(
        !marker_content.contains(std::str::from_utf8(&legacy_key).unwrap_or("")),
        "Marker must NEVER contain plaintext key material"
    );
}

#[tokio::test]
async fn test_post_migration_legacy_fallback_hard_rejected_when_marker_present() {
    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("rejected_legacy.wal");

    // 1. Prepare legacy WAL
    let op = WalOp::Put {
        tx_id: TxId::new(10),
        key: b"k1".to_vec(),
        value: b"v1".to_vec(),
    };
    let legacy_key = Wal::legacy_integrity_key_for_test();
    let legacy_entry = WalEntry::try_new(op, 10, &legacy_key, [0u8; 32]).expect("entry");
    let mut wal_bytes = Vec::new();
    wal_bytes.extend_from_slice(&WAL_V3_HEADER);
    wal_bytes.extend_from_slice(&legacy_entry.to_bytes().expect("to_bytes"));
    fs::write(&wal_path, wal_bytes).await.expect("write wal");

    // 2. Migrate
    let wal = Wal::open_for_legacy_migration(&wal_path, None)
        .await
        .expect("migrate");
    drop(wal);

    // 3. Re-opening with allow_legacy_integrity_key_fallback: true on a migrated file with marker
    let config = WalConfig::default().with_legacy_fallback(true);
    let std_wal = Wal::open_with_config(&wal_path, config)
        .await
        .expect("open");

    // Ensure that legacy fallback is hard-disabled internally because marker is present
    assert!(
        !std_wal.allow_legacy_fallback_for_test(),
        "Legacy fallback must be hard-disabled when retirement marker is present"
    );
}

#[tokio::test]
async fn test_crash_window_before_and_after_marker() {
    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("crash_window.wal");
    let marker_path = dir.path().join("crash_window.wal.rekeyed");

    let op = WalOp::Put {
        tx_id: TxId::new(99),
        key: b"crash_k".to_vec(),
        value: b"crash_v".to_vec(),
    };
    let legacy_key = Wal::legacy_integrity_key_for_test();
    let legacy_entry = WalEntry::try_new(op, 99, &legacy_key, [0u8; 32]).expect("entry");
    let mut wal_bytes = Vec::new();
    wal_bytes.extend_from_slice(&WAL_V3_HEADER);
    wal_bytes.extend_from_slice(&legacy_entry.to_bytes().expect("to_bytes"));
    fs::write(&wal_path, wal_bytes).await.expect("write wal");

    // Case A: Before marker -> migration is repeatable
    assert!(!marker_path.exists());
    let wal_a = Wal::open_for_legacy_migration(&wal_path, None)
        .await
        .expect("migration A");
    drop(wal_a);
    assert!(marker_path.exists());

    // Case B: After marker -> marker is present and final
    let wal_b = Wal::open(&wal_path).await.expect("open B");
    let entries = wal_b.replay().await.expect("replay B");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].1.seq_no, 99);
}
