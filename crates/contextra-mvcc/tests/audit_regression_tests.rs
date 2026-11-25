use contextra_mvcc::seq_log::{SeqLogEntry, SequenceLog};
use contextra_mvcc::ssi::{ReadSet, SequenceLogSsiValidator, SsiValidator};
use contextra_mvcc::tx_buffer::TxBuffer;
use contextra_mvcc::types::{TxId, TOMBSTONE_BIT};
use contextra_mvcc::ContextraError;

#[test]
fn test_forget_from_restores_earlier_commit_history() {
    let validator = SequenceLogSsiValidator::new();
    let tx = TxId::new(1);

    // Key 'k1' committed at seq 10
    validator.record_commit_key(b"k1", 10);
    // Key 'k1' re-committed at seq 20
    validator.record_commit_key(b"k1", 20);

    // Roll back sequence >= 15
    let removed = validator.forget_from(15);
    assert_eq!(removed, 1);

    // Transaction reading 'k1' at snapshot_seq 8 should conflict with commit at seq 10
    let mut read_set = ReadSet::new();
    read_set.record_read(b"k1".to_vec(), 8);

    let res = validator.validate(tx, &read_set);
    assert!(
        matches!(res, Err(ContextraError::Conflict(_))),
        "Expected conflict for snapshot_seq 8 against restored commit at seq 10, got {:?}",
        res
    );
}

#[test]
fn test_tombstone_bit_masking_in_read_set_and_validator() {
    let validator = SequenceLogSsiValidator::new();
    let tx = TxId::new(2);

    let mut read_set = ReadSet::new();
    // Raw seq number with TOMBSTONE_BIT set
    let raw_snapshot_seq = 10 | TOMBSTONE_BIT;
    read_set.record_read(b"k1".to_vec(), raw_snapshot_seq);

    // min_snapshot_seq must be masked (10)
    assert_eq!(
        read_set.min_snapshot_seq(),
        Some(10),
        "min_snapshot_seq should mask TOMBSTONE_BIT"
    );

    // Commit key at seq 11 (> 10)
    validator.record_commit_key(b"k1", 11);

    let res = validator.validate(tx, &read_set);
    assert!(
        matches!(res, Err(ContextraError::Conflict(_))),
        "Expected conflict for commit_seq 11 > snapshot_seq 10, got {:?}",
        res
    );
}

#[test]
fn test_tombstone_bit_masking_in_tx_buffer() {
    let buffer = TxBuffer::<String>::new();
    let tx = TxId::new(3);
    buffer.begin(tx);

    let raw_seq = 10 | TOMBSTONE_BIT;
    buffer.register_read(tx, b"k1".to_vec(), raw_seq);

    assert_eq!(
        buffer.min_read_snapshot(),
        Some(10),
        "TxBuffer min_read_snapshot must mask TOMBSTONE_BIT"
    );
}

#[test]
fn test_tombstone_bit_masking_in_seq_log() {
    let doc_id = contextra_mvcc::types::DocId::from(100);

    let entry = SeqLogEntry {
        doc_id,
        insert_seq: 10 | TOMBSTONE_BIT,
        delete_seq: None,
    };

    assert!(
        entry.is_visible(10 | TOMBSTONE_BIT),
        "Entry should be visible at insert_seq 10 when both carry TOMBSTONE_BIT"
    );
    assert!(
        entry.is_visible(15),
        "Entry should be visible at snapshot 15"
    );

    let mut log = SequenceLog::new();
    log.record_insert(doc_id, 10 | TOMBSTONE_BIT);
    assert!(
        log.is_visible(doc_id, 10),
        "SequenceLog should mask TOMBSTONE_BIT on insert_seq"
    );
}
