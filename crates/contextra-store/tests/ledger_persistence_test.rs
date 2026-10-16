// FILE-CONTEXT
// STAND: 2026-10-16
// ZWECK: Tests für Etappe 1 & 3: CTRALEDG v1 Erasure-Ledger Formate, Framing, Chain-MACs, Append, Torn Writes, Manipulation & Fail-Closed Opening.

use contextra_store::erasure_ledger::*;
use tempfile::tempdir;

#[tokio::test]
async fn test_ledger_append_and_open() {
    let dir = tempdir().unwrap();
    let master_secret = [42u8; 32];
    let key = LedgerKey::derive_from_master_secret(&master_secret);

    let (ledger_opt, status) =
        ErasureLedger::open_or_fail_closed(dir.path(), key.clone(), false, None)
            .await
            .unwrap();

    assert_eq!(status, LedgerOpenStatus::Fresh);
    let mut ledger = ledger_opt.unwrap();

    let entry1 = LedgerEntry {
        ledger_seq: 1,
        kind: LedgerEntryKind::Key,
        keyed_hash: compute_keyed_hash(&key, b"tenant_a", b"user:101"),
        erased_at_tx: 100,
        proof_id: [1u8; 16],
    };

    let entry2 = LedgerEntry {
        ledger_seq: 2,
        kind: LedgerEntryKind::Prefix { prefix_len: 5 },
        keyed_hash: compute_keyed_hash(&key, b"tenant_a", b"user:"),
        erased_at_tx: 150,
        proof_id: [2u8; 16],
    };

    ledger
        .append(&[entry1.clone(), entry2.clone()])
        .await
        .unwrap();

    // Reopen ledger
    let (reopened_opt, status2) =
        ErasureLedger::open_or_fail_closed(dir.path(), key.clone(), true, None)
            .await
            .unwrap();

    assert_eq!(status2, LedgerOpenStatus::Ok);
    let reopened = reopened_opt.unwrap();

    assert_eq!(reopened.entries().len(), 2);
    assert_eq!(reopened.entries()[0], entry1);
    assert_eq!(reopened.entries()[1], entry2);
    assert_eq!(reopened.last_chain_mac(), ledger.last_chain_mac());
}

#[tokio::test]
async fn test_torn_write_tail_truncation() {
    let dir = tempdir().unwrap();
    let master_secret = [99u8; 32];
    let key = LedgerKey::derive_from_master_secret(&master_secret);

    let (ledger_opt, _) = ErasureLedger::open_or_fail_closed(dir.path(), key.clone(), false, None)
        .await
        .unwrap();
    let mut ledger = ledger_opt.unwrap();

    let entry = LedgerEntry {
        ledger_seq: 1,
        kind: LedgerEntryKind::Key,
        keyed_hash: compute_keyed_hash(&key, b"scope", b"key1"),
        erased_at_tx: 10,
        proof_id: [7u8; 16],
    };
    ledger.append(&[entry.clone()]).await.unwrap();

    // Append partial garbage to simulate torn write at the tail of CTRALEDG segment
    let seg_path = dir.path().join("segment_0000000000000000.ctraledg");
    let mut file_bytes = tokio::fs::read(&seg_path).await.unwrap();
    file_bytes.extend_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF]); // Incomplete frame length
    tokio::fs::write(&seg_path, &file_bytes).await.unwrap();

    // Reopen - should heal tail truncation gracefully
    let (healed_opt, status) =
        ErasureLedger::open_or_fail_closed(dir.path(), key.clone(), true, None)
            .await
            .unwrap();

    assert_eq!(status, LedgerOpenStatus::Ok);
    let healed = healed_opt.unwrap();
    assert_eq!(healed.entries().len(), 1);
}

#[tokio::test]
async fn test_tampered_middle_frame_fails_closed() {
    let dir = tempdir().unwrap();
    let master_secret = [77u8; 32];
    let key = LedgerKey::derive_from_master_secret(&master_secret);

    let (ledger_opt, _) = ErasureLedger::open_or_fail_closed(dir.path(), key.clone(), false, None)
        .await
        .unwrap();
    let mut ledger = ledger_opt.unwrap();

    let e1 = LedgerEntry {
        ledger_seq: 1,
        kind: LedgerEntryKind::Key,
        keyed_hash: compute_keyed_hash(&key, b"scope", b"k1"),
        erased_at_tx: 10,
        proof_id: [1u8; 16],
    };
    let e2 = LedgerEntry {
        ledger_seq: 2,
        kind: LedgerEntryKind::Key,
        keyed_hash: compute_keyed_hash(&key, b"scope", b"k2"),
        erased_at_tx: 20,
        proof_id: [2u8; 16],
    };
    ledger.append(&[e1, e2]).await.unwrap();

    // Tamper with bytes in the middle of CTRALEDG segment file
    let seg_path = dir.path().join("segment_0000000000000000.ctraledg");
    let mut file_bytes = tokio::fs::read(&seg_path).await.unwrap();
    file_bytes[60] ^= 0xFF; // Mutate byte in the middle of frame 1 payload
    tokio::fs::write(&seg_path, &file_bytes).await.unwrap();

    // Reopen - should fail closed with Corrupt
    let (corrupt_opt, status) =
        ErasureLedger::open_or_fail_closed(dir.path(), key.clone(), true, None)
            .await
            .unwrap();

    assert!(corrupt_opt.is_none());
    match status {
        LedgerOpenStatus::Corrupt { reason } => {
            assert!(reason.contains("Corruption") || reason.contains("CRC32 mismatch") || reason.contains("MAC mismatch"));
        }
        _ => panic!("Expected Corrupt status, got {:?}", status),
    }
}
