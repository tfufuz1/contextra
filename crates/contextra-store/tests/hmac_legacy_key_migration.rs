// ZWECK: Tests für HMAC Legacy-Key Migration, LegacyKeyStatus und Tamper Detection.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_core::{ContextraError, TxId};
use contextra_store::wal::hmac::{migrate_legacy_key, LegacyKeyStatus};
use contextra_store::wal::{Wal, WalOp};
use tempfile::tempdir;

#[test]
fn test_legacy_key_status_methods_and_debug_format() {
    let standard = LegacyKeyStatus::Standard;
    assert!(standard.is_standard());
    assert!(!standard.is_legacy());
    assert_eq!(format!("{:?}", standard), "Standard");

    let legacy = LegacyKeyStatus::LegacyActive;
    assert!(legacy.is_legacy());
    assert!(!legacy.is_standard());
    assert_eq!(format!("{:?}", legacy), "LegacyActive");
}

#[test]
fn test_migrate_legacy_key_success_and_validations() {
    let legacy_key = Wal::legacy_integrity_key_for_test();
    let new_key = [0x42u8; 32];

    // 1. Valid migration
    let derived = migrate_legacy_key(&legacy_key, &new_key).expect("migration should succeed");
    assert_eq!(derived, new_key);

    // 2. Rejection of invalid legacy key
    let wrong_legacy = [0xFFu8; 32];
    let err_invalid_legacy = migrate_legacy_key(&wrong_legacy, &new_key);
    assert!(err_invalid_legacy.is_err());
    let err_msg = format!("{}", err_invalid_legacy.unwrap_err());
    assert!(err_msg.contains("does not match expected legacy integrity key"));

    // 3. Rejection of invalid new key size
    let short_key = [0x42u8; 16];
    let err_short = migrate_legacy_key(&legacy_key, &short_key);
    assert!(err_short.is_err());
    assert!(format!("{}", err_short.unwrap_err()).contains("must be exactly 32 bytes"));

    // 4. Rejection of identical new key (legacy -> legacy migration forbidden)
    let err_identical = migrate_legacy_key(&legacy_key, &legacy_key);
    assert!(err_identical.is_err());
    assert!(format!("{}", err_identical.unwrap_err()).contains("cannot be identical"));
}

#[tokio::test]
async fn test_wal_legacy_key_roundtrip_migration_and_tamper_detection() {
    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("legacy_migration.wal");

    let legacy_key = Wal::legacy_integrity_key_for_test();
    let new_key = [0x77u8; 32];

    // Migrate key state
    let migrated_key =
        migrate_legacy_key(&legacy_key, &new_key).expect("key migration should succeed");
    assert_eq!(migrated_key, new_key);

    // Open via legacy migration entrypoint
    let wal = Wal::open_for_legacy_migration(&wal_path, None)
        .await
        .expect("open wal for legacy migration");

    let op = WalOp::Put {
        tx_id: TxId::new(1),
        key: b"migration_key".to_vec(),
        value: b"migration_val".to_vec(),
    };

    let (batch, _) = wal
        .prepare_batch(vec![(op, 1)])
        .await
        .expect("prepare batch");
    wal.append_batch(batch).await.expect("append batch");

    drop(wal);

    // Verify re-opening via standard Wal::open succeeds and reads entry (since migration marker was set)
    let wal_read = Wal::open(&wal_path)
        .await
        .expect("re-open wal via standard path post-migration");

    let mut scanned = Vec::new();
    wal_read
        .scan_entries_with_callback(u64::MAX, |_seq, entry, _pos| {
            scanned.push(entry);
            true
        })
        .await
        .expect("scan entries");

    assert_eq!(scanned.len(), 1);

    drop(wal_read);

    // Tamper detection test: corrupt WAL file byte and verify scan fails
    let mut file_bytes = tokio::fs::read(&wal_path).await.expect("read wal file");
    if file_bytes.len() > 12 {
        file_bytes[12] ^= 0xFF; // flip bits in payload
        tokio::fs::write(&wal_path, file_bytes)
            .await
            .expect("write corrupted wal");
    }

    let res_tamper = Wal::open(&wal_path).await;
    assert!(
        matches!(
            res_tamper,
            Err(ContextraError::Serialization(_)) | Err(ContextraError::WalCorruption { .. })
        ),
        "Corrupted migrated WAL segment must fail verification"
    );
}

#[test]
fn test_log_and_debug_output_excludes_key_material() {
    let legacy_key = Wal::legacy_integrity_key_for_test();
    let new_key = [0x99u8; 32];

    let status = LegacyKeyStatus::LegacyActive;
    let debug_str = format!("{:?}", status);

    let legacy_hex = hex_like(&legacy_key);
    let new_hex = hex_like(&new_key);

    assert!(
        !debug_str.contains(&legacy_hex),
        "Debug string must not leak legacy key material"
    );
    assert!(
        !debug_str.contains(&new_hex),
        "Debug string must not leak new key material"
    );
}

fn hex_like(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}
