mod golden;

use contextra_store::wal::{Wal, WalOp, WAL_V3_HEADER};
use golden::generate::{golden_dir, generate_all, verify_file_hash};
use std::fs;

#[tokio::test]
async fn test_format_golden_wal() {
    let dir = golden_dir();
    if !dir.join("MANIFEST.txt").exists() {
        generate_all().await;
    }

    // 1. Check file exists and verify hash in MANIFEST.txt
    let wal_filename = "wal_v3.wal";
    let wal_path = dir.join(wal_filename);
    assert!(wal_path.exists(), "Golden WAL V3 file must exist");

    let is_valid_hash = verify_file_hash(&dir, wal_filename);
    assert!(
        is_valid_hash,
        "Golden WAL V3 file hash must match MANIFEST.txt entry"
    );

    // 2. Check binary header (MFW3)
    let bytes = fs::read(&wal_path).expect("read golden wal");
    assert!(bytes.len() >= 4, "WAL file must have at least header length");
    assert_eq!(&bytes[0..4], &WAL_V3_HEADER, "WAL header must match MFW3");

    // 3. Open WAL and replay
    let wal = Wal::open(&wal_path).await.expect("open golden WAL");
    let replayed = wal.replay().await.expect("replay golden WAL");

    assert_eq!(replayed.len(), 3, "Golden WAL must contain 3 entries");

    // Tuple structure returned by replay is (seq_no, entry, pos)
    // Entry 1: Put golden_wal_k1 -> golden_wal_v1
    let (seq1, entry1, _pos1) = &replayed[0];
    assert_eq!(*seq1, 1);
    assert_eq!(entry1.seq_no, 1);
    match &entry1.op {
        WalOp::Put { key, value, tx_id } => {
            assert_eq!(tx_id.inner(), 1);
            assert_eq!(key, b"golden_wal_k1");
            assert_eq!(value, b"golden_wal_v1");
        }
        other => panic!("Expected Put op, got {:?}", other),
    }

    // Entry 2: Put golden_wal_k2 -> golden_wal_v2
    let (seq2, entry2, _pos2) = &replayed[1];
    assert_eq!(*seq2, 2);
    assert_eq!(entry2.seq_no, 2);
    match &entry2.op {
        WalOp::Put { key, value, tx_id } => {
            assert_eq!(tx_id.inner(), 1);
            assert_eq!(key, b"golden_wal_k2");
            assert_eq!(value, b"golden_wal_v2");
        }
        other => panic!("Expected Put op, got {:?}", other),
    }

    // Entry 3: TxEnd committed=true
    let (seq3, entry3, _pos3) = &replayed[2];
    assert_eq!(*seq3, 3);
    assert_eq!(entry3.seq_no, 3);
    match &entry3.op {
        WalOp::TxEnd { tx_id, committed } => {
            assert_eq!(tx_id.inner(), 1);
            assert!(*committed);
        }
        other => panic!("Expected TxEnd op, got {:?}", other),
    }
}
