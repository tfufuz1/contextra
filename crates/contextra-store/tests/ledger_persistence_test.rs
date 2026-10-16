// FILE-CONTEXT
// STAND: 2026-10-16
// ZWECK: Persistenz-, Append- und Integrity-Tests für CTRALEDG v1 ErasureLedger.

#![forbid(unsafe_code)]

use contextra_ports::SystemClock;
use contextra_store::erasure_ledger::*;
use tempfile::tempdir;

#[tokio::test]
async fn test_ledger_append_and_open() {
    let dir = tempdir().unwrap();
    let master_secret = [42u8; 32];
    let key = LedgerKey::derive_from_master_secret(&master_secret);
    let clock = SystemClock::new();

    let (ledger_opt, status) =
        ErasureLedger::open_assured(dir.path(), key.clone(), false, None)
            .await
            .unwrap();

    assert_eq!(status, LedgerOpenStatus::Fresh);
    let mut ledger = ledger_opt.unwrap();

    ledger
        .append_intent(100, &clock, [1u8; 16], b"tenant_a", &[b"user:101"], b"payload1")
        .await
        .unwrap();

    ledger
        .append_purged(150, &clock, [2u8; 16], b"tenant_a", &[b"user:"])
        .await
        .unwrap();

    // Reopen ledger
    let (reopened_opt, status2) =
        ErasureLedger::open_assured(dir.path(), key.clone(), true, None)
            .await
            .unwrap();

    assert_eq!(status2, LedgerOpenStatus::Ok);
    let reopened = reopened_opt.unwrap();

    assert_eq!(reopened.records().len(), 2);
    assert_eq!(reopened.records()[0].record_type, RecordType::Intent);
    assert_eq!(reopened.records()[1].record_type, RecordType::Purged);
    assert_eq!(reopened.last_chain_mac(), ledger.last_chain_mac());
}

#[tokio::test]
async fn test_torn_write_tail_truncation() {
    let dir = tempdir().unwrap();
    let master_secret = [99u8; 32];
    let key = LedgerKey::derive_from_master_secret(&master_secret);
    let clock = SystemClock::new();

    let (ledger_opt, _) = ErasureLedger::open_assured(dir.path(), key.clone(), false, None)
        .await
        .unwrap();
    let mut ledger = ledger_opt.unwrap();

    ledger
        .append_intent(10, &clock, [7u8; 16], b"scope", &[b"key1"], b"p1")
        .await
        .unwrap();

    // Append partial garbage to simulate torn write at the tail
    let ledger_path = dir.path().join("erasure.ledger");
    let mut file_bytes = tokio::fs::read(&ledger_path).await.unwrap();
    file_bytes.extend_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF]); // Incomplete frame
    tokio::fs::write(&ledger_path, &file_bytes).await.unwrap();

    // Reopen - should heal tail truncation gracefully
    let (healed_opt, status) =
        ErasureLedger::open_assured(dir.path(), key.clone(), true, None)
            .await
            .unwrap();

    assert_eq!(status, LedgerOpenStatus::Ok);
    let healed = healed_opt.unwrap();
    assert_eq!(healed.records().len(), 1);
}

#[tokio::test]
async fn test_tampered_middle_frame_fails_closed() {
    let dir = tempdir().unwrap();
    let master_secret = [77u8; 32];
    let key = LedgerKey::derive_from_master_secret(&master_secret);
    let clock = SystemClock::new();

    let (ledger_opt, _) = ErasureLedger::open_assured(dir.path(), key.clone(), false, None)
        .await
        .unwrap();
    let mut ledger = ledger_opt.unwrap();

    ledger
        .append_intent(10, &clock, [1u8; 16], b"scope", &[b"k1"], b"p1")
        .await
        .unwrap();
    ledger
        .append_intent(20, &clock, [2u8; 16], b"scope", &[b"k2"], b"p2")
        .await
        .unwrap();

    // Tamper with bytes in the payload section of frame 1
    let ledger_path = dir.path().join("erasure.ledger");
    let mut file_bytes = tokio::fs::read(&ledger_path).await.unwrap();
    assert!(file_bytes.len() > 65);
    file_bytes[65] ^= 0xFF; // Mutate byte in frame body
    tokio::fs::write(&ledger_path, &file_bytes).await.unwrap();

    // Reopen - should fail closed
    let (corrupt_opt, status) =
        ErasureLedger::open_assured(dir.path(), key.clone(), true, None)
            .await
            .unwrap();

    assert!(corrupt_opt.is_none());
    match status {
        LedgerOpenStatus::Corrupt { reason } | LedgerOpenStatus::RollbackSuspected { reason } => {
            assert!(
                reason.contains("Corruption")
                    || reason.contains("CRC32C")
                    || reason.contains("MAC")
                    || reason.contains("does not match")
            );
        }
        _ => panic!("Expected fail-closed status, got {:?}", status),
    }
}
