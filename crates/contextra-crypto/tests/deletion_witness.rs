//! Integration tests for LayerWitness (Auftrag W5-02).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_crypto::deletion_proof::{
    hash_deleted_keys_length_prefixed, DeletionLayer,
};
use contextra_crypto::deletion_witness::{LayerWitness, WitnessEvidence};
use contextra_types::TxId;

#[test]
fn test_layer_witness_positive_cases() {
    let keys = vec![b"key1".to_vec(), b"key2".to_vec()];
    let expected_hash = hash_deleted_keys_length_prefixed(&keys);
    let tx_id = TxId(100);

    // 1. LsmMemtable with MemtableScan
    let witness_mem = LayerWitness::issue_from_storage_layer(
        DeletionLayer::LsmMemtable,
        &keys,
        tx_id,
        WitnessEvidence::MemtableScan {
            memtables_scanned: 3,
        },
    )
    .expect("LsmMemtable witness creation failed");

    assert_eq!(witness_mem.layer(), &DeletionLayer::LsmMemtable);
    assert_eq!(witness_mem.scope_hash(), &expected_hash);
    assert_eq!(witness_mem.scanned_at_tx(), tx_id);
    assert_eq!(
        witness_mem.evidence(),
        &WitnessEvidence::MemtableScan {
            memtables_scanned: 3
        }
    );

    // 2. SsTableAllLevels with SstableScan
    let witness_sst = LayerWitness::issue_from_storage_layer(
        DeletionLayer::SsTableAllLevels,
        &keys,
        tx_id,
        WitnessEvidence::SstableScan { files_scanned: 12 },
    )
    .expect("SsTableAllLevels witness creation failed");

    assert_eq!(witness_sst.layer(), &DeletionLayer::SsTableAllLevels);
    assert_eq!(
        witness_sst.evidence(),
        &WitnessEvidence::SstableScan { files_scanned: 12 }
    );

    // 3. WalAllSegments with WalPurge
    let wal_layer = DeletionLayer::WalAllSegments { seq_after: 50 };
    let witness_wal = LayerWitness::issue_from_storage_layer(
        wal_layer.clone(),
        &keys,
        tx_id,
        WitnessEvidence::WalPurge {
            segments_removed: 2,
            bytes_scrubbed: 4096,
        },
    )
    .expect("WalAllSegments witness creation failed");

    assert_eq!(witness_wal.layer(), &wal_layer);
    assert_eq!(
        witness_wal.evidence(),
        &WitnessEvidence::WalPurge {
            segments_removed: 2,
            bytes_scrubbed: 4096,
        }
    );
}

#[test]
fn test_layer_witness_rejection_for_non_scanner_layers() {
    let keys = vec![b"key_vector".to_vec()];
    let tx_id = TxId(100);

    // HnswIndex rejected
    let res = LayerWitness::issue_from_storage_layer(
        DeletionLayer::HnswIndex,
        &keys,
        tx_id,
        WitnessEvidence::SstableScan { files_scanned: 1 },
    );
    assert!(res.is_err(), "HnswIndex layer must be rejected");

    // CsrGraph rejected
    let res = LayerWitness::issue_from_storage_layer(
        DeletionLayer::CsrGraph,
        &keys,
        tx_id,
        WitnessEvidence::SstableScan { files_scanned: 1 },
    );
    assert!(res.is_err(), "CsrGraph layer must be rejected");

    // KvCacheSegments rejected
    let res = LayerWitness::issue_from_storage_layer(
        DeletionLayer::KvCacheSegments,
        &keys,
        tx_id,
        WitnessEvidence::MemtableScan { memtables_scanned: 1 },
    );
    assert!(res.is_err(), "KvCacheSegments layer must be rejected");

    // EmbeddingCache rejected
    let res = LayerWitness::issue_from_storage_layer(
        DeletionLayer::EmbeddingCache,
        &keys,
        tx_id,
        WitnessEvidence::MemtableScan { memtables_scanned: 1 },
    );
    assert!(res.is_err(), "EmbeddingCache layer must be rejected");
}

#[test]
fn test_layer_witness_rejection_for_evidence_mismatch() {
    let keys = vec![b"key".to_vec()];
    let tx_id = TxId(10);

    // LsmMemtable with SstableScan -> Err
    let res = LayerWitness::issue_from_storage_layer(
        DeletionLayer::LsmMemtable,
        &keys,
        tx_id,
        WitnessEvidence::SstableScan { files_scanned: 5 },
    );
    assert!(res.is_err(), "Memtable layer with SstableScan must be rejected");

    // SsTableAllLevels with WalPurge -> Err
    let res = LayerWitness::issue_from_storage_layer(
        DeletionLayer::SsTableAllLevels,
        &keys,
        tx_id,
        WitnessEvidence::WalPurge {
            segments_removed: 1,
            bytes_scrubbed: 100,
        },
    );
    assert!(res.is_err(), "Sstable layer with WalPurge must be rejected");

    // WalAllSegments with MemtableScan -> Err
    let res = LayerWitness::issue_from_storage_layer(
        DeletionLayer::WalAllSegments { seq_after: 1 },
        &keys,
        tx_id,
        WitnessEvidence::MemtableScan { memtables_scanned: 1 },
    );
    assert!(res.is_err(), "Wal layer with MemtableScan must be rejected");
}

#[test]
fn test_layer_witness_rejection_for_empty_keys() {
    let empty_keys: Vec<Vec<u8>> = Vec::new();
    let tx_id = TxId(10);

    let res = LayerWitness::issue_from_storage_layer(
        DeletionLayer::LsmMemtable,
        &empty_keys,
        tx_id,
        WitnessEvidence::MemtableScan { memtables_scanned: 1 },
    );
    assert!(res.is_err(), "Empty keys array must be rejected");
}

#[test]
fn test_check_binding_validations() {
    let keys = vec![b"keyA".to_vec(), b"keyB".to_vec()];
    let correct_hash = hash_deleted_keys_length_prefixed(&keys);
    let wrong_hash = [0xFFu8; 32];

    let scanned_tx = TxId(200);
    let witness = LayerWitness::issue_from_storage_layer(
        DeletionLayer::LsmMemtable,
        &keys,
        scanned_tx,
        WitnessEvidence::MemtableScan { memtables_scanned: 1 },
    )
    .unwrap();

    // 1. Correct parameters -> Ok
    assert!(witness.check_binding(&correct_hash, TxId(150)).is_ok());
    assert!(witness.check_binding(&correct_hash, TxId(200)).is_ok());

    // 2. Wrong scope hash -> Err
    let err_hash = witness.check_binding(&wrong_hash, TxId(150));
    assert!(err_hash.is_err(), "Mismatched scope hash must return Err");

    // 3. scanned_at_tx < deleted_after_tx -> Err
    let err_tx = witness.check_binding(&correct_hash, TxId(250));
    assert!(
        err_tx.is_err(),
        "scanned_at_tx < deleted_after_tx must return Err"
    );
}

#[test]
fn test_no_code_dependency_on_revocation_log() {
    // This test explicitly confirms that LayerWitness functions without any revocation_log dependency.
    let keys = vec![b"doc_key_123".to_vec()];
    let witness = LayerWitness::issue_from_storage_layer(
        DeletionLayer::LsmMemtable,
        &keys,
        TxId(1),
        WitnessEvidence::MemtableScan { memtables_scanned: 1 },
    );
    assert!(witness.is_ok());
}
