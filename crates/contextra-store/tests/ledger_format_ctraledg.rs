// FILE-CONTEXT
// STAND: 2026-10-16
// ZWECK: Format-Tests für CTRALEDG v1: Header, Record-Framing, MAC-Chaining, Torn Tail, Corruption, Rollback, Migration.

use contextra_store::erasure_ledger::*;
use tempfile::tempdir;

#[tokio::test]
async fn test_ctraledg_format_and_mac_chaining() {
    let dir = tempdir().unwrap();
    let master_secret = [123u8; 32];
    let key = LedgerKey::derive_from_master_secret(&master_secret);

    let (ledger_opt, status) = ErasureLedger::open_assured(dir.path(), key.clone(), false, None)
        .await
        .unwrap();

    assert_eq!(status, LedgerOpenStatus::Fresh);
    let mut ledger = ledger_opt.unwrap();

    let scope_digest1 = compute_digest(&key, b"scope_1");
    let family_digest1 = compute_digest(&key, b"family_1");

    ledger
        .append_intent(
            10,
            [1u8; 16],
            scope_digest1,
            vec![family_digest1],
            b"prefix:user:".to_vec(),
        )
        .await
        .unwrap();

    let (seq1, mac1) = ledger.read_head_info();
    assert_eq!(seq1, 1);
    assert_ne!(mac1, [0u8; 32]);

    let scope_digest2 = compute_digest(&key, b"scope_2");
    ledger
        .append_purged(20, [2u8; 16], scope_digest2, vec![])
        .await
        .unwrap();

    let (seq2, mac2) = ledger.read_head_info();
    assert_eq!(seq2, 2);
    assert_ne!(mac1, mac2);

    // Reopen and check
    let (reopened_opt, status2) = ErasureLedger::open_assured(dir.path(), key.clone(), true, None)
        .await
        .unwrap();

    assert_eq!(status2, LedgerOpenStatus::Ok);
    let reopened = reopened_opt.unwrap();
    let (r_seq, r_mac) = reopened.read_head_info();
    assert_eq!(r_seq, 2);
    assert_eq!(r_mac, mac2);
}

#[tokio::test]
async fn test_ctraledg_torn_tail_truncation() {
    let dir = tempdir().unwrap();
    let key = LedgerKey::new([42u8; 32]);

    let (ledger_opt, _) = ErasureLedger::open_assured(dir.path(), key.clone(), false, None)
        .await
        .unwrap();
    let mut ledger = ledger_opt.unwrap();

    ledger
        .append_purged(5, [1u8; 16], [0xAA; 32], vec![])
        .await
        .unwrap();

    let seg_path = dir.path().join("segment_0000000000000000.ctraledg");
    let mut bytes = tokio::fs::read(&seg_path).await.unwrap();

    // Append partial frame bytes to simulate torn write at tail
    bytes.extend_from_slice(&[0x10, 0x00, 0x00, 0x00, 0x01, 0x02]);
    tokio::fs::write(&seg_path, &bytes).await.unwrap();

    // Reopen - torn tail must be truncated gracefully
    let (reopened_opt, status) = ErasureLedger::open_assured(dir.path(), key.clone(), true, None)
        .await
        .unwrap();

    assert_eq!(status, LedgerOpenStatus::Ok);
    let reopened = reopened_opt.unwrap();
    let (seq, _) = reopened.read_head_info();
    assert_eq!(seq, 1);
}

#[tokio::test]
async fn test_ctraledg_middle_corruption_fails_closed() {
    let dir = tempdir().unwrap();
    let key = LedgerKey::new([99u8; 32]);

    let (ledger_opt, _) = ErasureLedger::open_assured(dir.path(), key.clone(), false, None)
        .await
        .unwrap();
    let mut ledger = ledger_opt.unwrap();

    ledger
        .append_purged(5, [1u8; 16], [0x11; 32], vec![])
        .await
        .unwrap();
    ledger
        .append_purged(10, [2u8; 16], [0x22; 32], vec![])
        .await
        .unwrap();

    let seg_path = dir.path().join("segment_0000000000000000.ctraledg");
    let mut bytes = tokio::fs::read(&seg_path).await.unwrap();

    // Tamper with byte in the middle of frame 1 body
    bytes[60] ^= 0xFF;
    tokio::fs::write(&seg_path, &bytes).await.unwrap();

    // Reopen - middle corruption must yield LedgerOpenStatus::Corrupt
    let (reopened_opt, status) = ErasureLedger::open_assured(dir.path(), key.clone(), true, None)
        .await
        .unwrap();

    assert!(reopened_opt.is_none());
    assert!(matches!(status, LedgerOpenStatus::Corrupt { .. }));
}

#[tokio::test]
async fn test_ctraledg_head_mac_mismatch_rollback_suspected() {
    let dir = tempdir().unwrap();
    let key = LedgerKey::new([77u8; 32]);

    let (ledger_opt, _) = ErasureLedger::open_assured(dir.path(), key.clone(), false, None)
        .await
        .unwrap();
    let mut ledger = ledger_opt.unwrap();

    ledger
        .append_purged(5, [1u8; 16], [0x33; 32], vec![])
        .await
        .unwrap();

    let head_path = dir.path().join("ledger.head");
    let mut head_bytes = tokio::fs::read(&head_path).await.unwrap();

    // Tamper with head MAC
    let last = head_bytes.len() - 10;
    head_bytes[last] ^= 0xFF;
    tokio::fs::write(&head_path, &head_bytes).await.unwrap();

    let (reopened_opt, status) = ErasureLedger::open_assured(dir.path(), key.clone(), true, None)
        .await
        .unwrap();

    assert!(reopened_opt.is_none());
    assert!(matches!(status, LedgerOpenStatus::RollbackSuspected { .. }));
}

#[tokio::test]
async fn test_ctraledg_compact_segments_removes_plaintext_payload() {
    let dir = tempdir().unwrap();
    let key = LedgerKey::new([88u8; 32]);

    let (ledger_opt, _) = ErasureLedger::open_assured(dir.path(), key.clone(), false, None)
        .await
        .unwrap();
    let mut ledger = ledger_opt.unwrap();

    ledger
        .append_intent(
            15,
            [1u8; 16],
            [0xAA; 32],
            vec![],
            b"secret_plaintext_prefix".to_vec(),
        )
        .await
        .unwrap();

    let compacted = ledger.compact_segments().await.unwrap();
    assert_eq!(compacted, 1);

    let seg_path = dir.path().join("segment_0000000000000000.ctraledg");
    let seg_bytes = tokio::fs::read(&seg_path).await.unwrap();

    let contains_plaintext = seg_bytes
        .windows(b"secret_plaintext_prefix".len())
        .any(|window| window == b"secret_plaintext_prefix");
    assert!(!contains_plaintext, "Plaintext payload must be cleared after compaction");
}
