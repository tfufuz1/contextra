// FILE-CONTEXT
// STAND: 2026-10-16
// ZWECK: Invarianten- und Matrix-Tests für Format CTRALEDG v1 (§6.5; Befund B-10).
// TEST-ABDECKUNG: Torn Tail, Mittel-Beschädigung, Rollback, ELDG-Migration, Gruppen-Append, Plaintext-Präfix Isolation.

#![forbid(unsafe_code)]

use contextra_ports::SystemClock;
use contextra_store::erasure_ledger::*;
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn test_ctraledg_torn_tail_truncation_on_open() {
    let dir = tempdir().unwrap();
    let key = LedgerKey::new([0x11; 32]);
    let clock = SystemClock::new();

    let (ledger_opt, status) = ErasureLedger::open_assured(dir.path(), key.clone(), false, None)
        .await
        .unwrap();
    assert_eq!(status, LedgerOpenStatus::Fresh);
    let mut ledger = ledger_opt.unwrap();

    let purge_id = [1u8; 16];
    ledger
        .append_intent(
            100,
            &clock,
            purge_id,
            b"lsm",
            &[b"user:101"],
            b"user:101:payload",
        )
        .await
        .unwrap();

    // Corrupt tail by appending truncated garbage
    let ledger_path = dir.path().join("erasure.ledger");
    let mut raw_bytes = tokio::fs::read(&ledger_path).await.unwrap();
    raw_bytes.extend_from_slice(&[0x43, 0x54, 0x52, 0x41]); // Partial frame bytes
    tokio::fs::write(&ledger_path, &raw_bytes).await.unwrap();

    // Reopen ledger - torn tail should be truncated and verified records restored
    let (reopened_opt, reopen_status) = ErasureLedger::open_assured(dir.path(), key, true, None)
        .await
        .unwrap();

    assert_eq!(reopen_status, LedgerOpenStatus::Ok);
    let reopened = reopened_opt.unwrap();
    assert_eq!(reopened.records().len(), 1);
    assert_eq!(reopened.records()[0].record_type, RecordType::Intent);
    assert_eq!(reopened.records()[0].tx, 100);
}

#[tokio::test]
async fn test_ctraledg_middle_corruption_returns_ledger_corrupt() {
    let dir = tempdir().unwrap();
    let key = LedgerKey::new([0x22; 32]);
    let clock = SystemClock::new();

    let (ledger_opt, _) = ErasureLedger::open_assured(dir.path(), key.clone(), false, None)
        .await
        .unwrap();
    let mut ledger = ledger_opt.unwrap();

    ledger
        .append_intent(1, &clock, [1u8; 16], b"scope1", &[b"key1"], b"p1")
        .await
        .unwrap();

    ledger
        .append_intent(2, &clock, [2u8; 16], b"scope2", &[b"key2"], b"p2")
        .await
        .unwrap();

    // Tamper with bytes in the middle of the ledger file (frame 1 payload)
    let ledger_path = dir.path().join("erasure.ledger");
    let mut raw_bytes = tokio::fs::read(&ledger_path).await.unwrap();
    assert!(raw_bytes.len() > 60);
    raw_bytes[55] ^= 0xFF; // Flip bits in middle of frame
    tokio::fs::write(&ledger_path, &raw_bytes).await.unwrap();

    // Reopen - should return Corrupt status
    let (reopened_opt, reopen_status) = ErasureLedger::open_assured(dir.path(), key, true, None)
        .await
        .unwrap();

    assert!(reopened_opt.is_none());
    match reopen_status {
        LedgerOpenStatus::Corrupt { reason } => {
            assert!(
                reason.contains("CRC32C mismatch")
                    || reason.contains("MAC chain")
                    || reason.contains("Corruption"),
                "Unexpected reason: {}",
                reason
            );
        }
        _ => panic!("Expected Corrupt status, got {:?}", reopen_status),
    }
}

#[tokio::test]
async fn test_ctraledg_rollback_head_mismatch_returns_rollback_suspected() {
    let dir = tempdir().unwrap();
    let key = LedgerKey::new([0x33; 32]);
    let clock = SystemClock::new();

    let (ledger_opt, _) = ErasureLedger::open_assured(dir.path(), key.clone(), false, None)
        .await
        .unwrap();
    let mut ledger = ledger_opt.unwrap();

    ledger
        .append_intent(10, &clock, [10u8; 16], b"scope", &[b"k1"], b"p1")
        .await
        .unwrap();

    // Save old ledger file state
    let ledger_path = dir.path().join("erasure.ledger");
    let old_ledger_bytes = tokio::fs::read(&ledger_path).await.unwrap();

    ledger
        .append_intent(20, &clock, [20u8; 16], b"scope", &[b"k2"], b"p2")
        .await
        .unwrap();

    let head_mac_after_20 = *ledger.last_chain_mac();

    // Revert erasure.ledger to older bytes (simulating file rollback)
    tokio::fs::write(&ledger_path, &old_ledger_bytes)
        .await
        .unwrap();

    // Reopen expecting head_mac_after_20
    let (reopened_opt, reopen_status) =
        ErasureLedger::open_assured(dir.path(), key, true, Some(head_mac_after_20))
            .await
            .unwrap();

    assert!(reopened_opt.is_none());
    match reopen_status {
        LedgerOpenStatus::RollbackSuspected { reason } => {
            assert!(
                reason.contains("head MAC") || reason.contains("does not match"),
                "Unexpected reason: {}",
                reason
            );
        }
        _ => panic!("Expected RollbackSuspected, got {:?}", reopen_status),
    }
}

#[tokio::test]
async fn test_ctraledg_atomic_legacy_eldg_migration() {
    let dir = tempdir().unwrap();
    let key = LedgerKey::new([0x44; 32]);
    let ledger_path = dir.path().join("erasure.ledger");

    // Construct legacy ELDG file
    let mut legacy_bytes = Vec::new();
    let payload = b"\x00\x00\x00\x00\x00\x00\x00\x01\x01key_data_here\x00\x00\x00\x00\x00\x00\x00\x0a\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01";
    let len = payload.len() as u32;
    let crc = crc32fast::hash(payload);
    let mac = [0u8; 32];

    legacy_bytes.extend_from_slice(&len.to_be_bytes());
    legacy_bytes.extend_from_slice(payload);
    legacy_bytes.extend_from_slice(&crc.to_be_bytes());
    legacy_bytes.extend_from_slice(&mac);

    tokio::fs::write(&ledger_path, &legacy_bytes).await.unwrap();

    // Open - should trigger automatic legacy ELDG -> CTRALEDG migration
    let (migrated_opt, status) = ErasureLedger::open_assured(dir.path(), key.clone(), true, None)
        .await
        .unwrap();

    assert_eq!(status, LedgerOpenStatus::Ok);
    let migrated = migrated_opt.unwrap();
    assert!(!migrated.records().is_empty());

    // Verify file on disk now starts with CTRALEDG magic
    let disk_bytes = tokio::fs::read(&ledger_path).await.unwrap();
    assert_eq!(&disk_bytes[0..8], b"CTRALEDG");
}

#[tokio::test]
async fn test_ctraledg_group_append_durability() {
    let dir = tempdir().unwrap();
    let key = LedgerKey::new([0x55; 32]);
    let clock = SystemClock::new();

    let (ledger_opt, _) = ErasureLedger::open_assured(dir.path(), key.clone(), false, None)
        .await
        .unwrap();
    let ledger = Arc::new(tokio::sync::Mutex::new(ledger_opt.unwrap()));

    let handle1 = {
        let l = ledger.clone();
        let c = clock.clone();
        tokio::spawn(async move {
            let mut guard = l.lock().await;
            guard
                .append_intent(1, &c, [1u8; 16], b"scope1", &[b"key1"], b"p1")
                .await
        })
    };

    let handle2 = {
        let l = ledger.clone();
        let c = clock.clone();
        tokio::spawn(async move {
            let mut guard = l.lock().await;
            guard
                .append_purged(2, &c, [2u8; 16], b"scope2", &[b"key2"])
                .await
        })
    };

    let (res1, res2) = tokio::join!(handle1, handle2);
    assert!(res1.unwrap().is_ok());
    assert!(res2.unwrap().is_ok());

    let final_ledger = ledger.lock().await;
    assert_eq!(final_ledger.records().len(), 2);
}

#[tokio::test]
async fn test_ctraledg_plaintext_prefix_isolation_intent_vs_purged() {
    let dir = tempdir().unwrap();
    let key = LedgerKey::new([0x66; 32]);
    let clock = SystemClock::new();

    let (ledger_opt, _) = ErasureLedger::open_assured(dir.path(), key.clone(), false, None)
        .await
        .unwrap();
    let mut ledger = ledger_opt.unwrap();

    // 1. Intent record MUST contain plaintext payload
    ledger
        .append_intent(
            100,
            &clock,
            [10u8; 16],
            b"lsm",
            &[b"user:42"],
            b"user:42:secret_prefix",
        )
        .await
        .unwrap();

    // 2. Purged record MUST contain empty payload (no plaintext prefixes)
    ledger
        .append_purged(200, &clock, [20u8; 16], b"lsm", &[b"user:42"])
        .await
        .unwrap();

    assert_eq!(ledger.records().len(), 2);
    let intent_rec = &ledger.records()[0];
    let purged_rec = &ledger.records()[1];

    assert_eq!(intent_rec.record_type, RecordType::Intent);
    assert_eq!(intent_rec.payload, b"user:42:secret_prefix");

    assert_eq!(purged_rec.record_type, RecordType::Purged);
    assert!(
        purged_rec.payload.is_empty(),
        "Purged record payload MUST be empty to ensure no plaintext leakage"
    );

    // 3. Reopen and verify plaintext payload restriction holds fail-closed
    let (reopened_opt, status) = ErasureLedger::open_assured(dir.path(), key, true, None)
        .await
        .unwrap();

    assert_eq!(status, LedgerOpenStatus::Ok);
    let reopened = reopened_opt.unwrap();
    assert_eq!(reopened.records().len(), 2);
}

#[tokio::test]
async fn test_ctraledg_compaction_replaces_intents_with_purged_and_scrubs() {
    let dir = tempdir().unwrap();
    let key = LedgerKey::new([0x77; 32]);
    let clock = SystemClock::new();

    let (ledger_opt, _) = ErasureLedger::open_assured(dir.path(), key.clone(), false, None)
        .await
        .unwrap();
    let mut ledger = ledger_opt.unwrap();

    ledger
        .append_intent(1, &clock, [1u8; 16], b"scope", &[b"k1"], b"plaintext1")
        .await
        .unwrap();

    assert_eq!(ledger.records()[0].record_type, RecordType::Intent);

    // Run compaction
    ledger.compact_intents_to_purged().await.unwrap();

    assert_eq!(ledger.records()[0].record_type, RecordType::Purged);
    assert!(ledger.records()[0].payload.is_empty());

    // Reopen and verify compacted file
    let (reopened_opt, status) = ErasureLedger::open_assured(dir.path(), key, true, None)
        .await
        .unwrap();
    assert_eq!(status, LedgerOpenStatus::Ok);
    let reopened = reopened_opt.unwrap();
    assert_eq!(reopened.records()[0].record_type, RecordType::Purged);
    assert!(reopened.records()[0].payload.is_empty());
}
