//! Integration tests verifying wiring and production reachability for the 13 symbols in campaign J23-contextra-mvcc.

use contextra_mvcc::{
    DocId, IndexOp, ReadSet, SequenceLog, SequenceLogSsiValidator, SnapshotRegistry, SsiValidator,
    TxBuffer, TxId,
};
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[test]
fn test_j23_seq_log_expired_pins_and_set_max_pin_duration() {
    let seq_log = SequenceLog::new();
    let now = Instant::now();
    let past = now.checked_sub(Duration::from_millis(50)).unwrap();

    // Set pin duration via SsiValidator builder which calls set_max_pin_duration on SequenceLog
    let seq_log_arc = Arc::new(RwLock::new(seq_log));
    let _validator = SequenceLogSsiValidator::builder()
        .with_sequence_log(seq_log_arc.clone())
        .with_max_pin_duration(Duration::from_millis(10))
        .build();

    seq_log_arc.write().pin_snapshot_at(100, past);

    // Call min_retention_seq on the enclosing SequenceLog type, which invokes expired_pins()
    let min_ret = seq_log_arc.read().min_retention_seq();
    assert_eq!(min_ret, Some(100));
}

#[test]
fn test_j23_snapshot_longest_active_pin_via_ssi() {
    let registry = Arc::new(SnapshotRegistry::new());
    let _g = registry.register(42);

    let validator = SequenceLogSsiValidator::builder()
        .with_snapshot_registry(registry.clone())
        .build();

    // Populate seq_index with committed write
    validator.record_commit_key(b"k1", 10);

    // Enclosing call diagnose_pruning_blocker internally calls reg.longest_active_pin()
    let blocker_populated = validator.diagnose_pruning_blocker();
    assert!(blocker_populated.is_some());
    let info = blocker_populated.unwrap();
    assert_eq!(info.longest_active_snapshot_seq, Some(42));
}

#[test]
fn test_j23_ssi_validator_wiring_and_read_set_maps() {
    let seq_log = Arc::new(RwLock::new(SequenceLog::new()));
    let registry = Arc::new(SnapshotRegistry::new());

    // Enclosing builder constructs and wires with_sequence_log & with_snapshot_registry
    let validator = SequenceLogSsiValidator::builder()
        .with_max_tracked_keys(10)
        .with_sequence_log(seq_log.clone())
        .with_snapshot_registry(registry.clone())
        .with_max_pin_duration(Duration::from_secs(120))
        .build();

    assert_eq!(validator.max_tracked_keys(), 10);

    let mut read_set = ReadSet::new();
    read_set.record_read(b"key_a".to_vec(), 10);
    read_set.record_prefix(b"pref_".to_vec(), 10);

    // Enclosing ReadSet calls (len, is_empty, iter, prefixes_iter) internally delegate to keys_map & prefixes_map
    assert_eq!(read_set.len(), 2);
    assert!(!read_set.is_empty());
    assert_eq!(read_set.iter().count(), 1);
    assert_eq!(read_set.prefixes_iter().count(), 1);

    // Enclosing record_commit_key delegates to record_commit_keys
    validator.record_commit_key(b"key_b", 10);

    // Enclosing validate delegates to validate_and_record
    assert!(validator.validate(TxId::new(1), &read_set).is_ok());

    // set_snapshot_registry mutation wiring
    let mut mut_val = SequenceLogSsiValidator::new();
    mut_val.set_snapshot_registry(registry);
}

#[test]
fn test_j23_tx_buffer_staged_status_and_drain_wiring() {
    let buffer = TxBuffer::<(Vec<u8>, Vec<u8>)>::new();
    let tx = TxId::new(50);
    let doc_id = DocId::from(1u64);

    buffer.begin(tx);
    buffer
        .stage_kv(
            tx,
            IndexOp::Insert {
                doc_id,
                data: (b"key1".to_vec(), b"val1".to_vec()),
            },
        )
        .unwrap();

    // Enclosing staged_status calls is_key_staged_for_tx
    assert_eq!(buffer.staged_status(b"key1"), Some(true));

    // Enclosing get_ops and drain calls validate_pending_ops
    assert_eq!(buffer.get_ops(tx).map(|v| v.len()), Some(1));
    let ops = buffer.drain_kv(tx);
    assert_eq!(ops.len(), 1);
}
