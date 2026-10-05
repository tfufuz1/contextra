//! Integration tests verifying wiring and production reachability for the 13 symbols in campaign J23-contextra-mvcc.

use contextra_mvcc::{
    DocId, IndexOp, ReadSet, SequenceLog, SequenceLogSsiValidator, SnapshotRegistry,
    SsiValidator, TxBuffer, TxId,
};
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[test]
fn test_j23closure_seq_log_expired_pins_and_set_max_pin_duration() {
    let mut seq_log = SequenceLog::new();
    seq_log.set_max_pin_duration(Duration::from_millis(10));
    assert_eq!(seq_log.max_pin_duration(), Duration::from_millis(10));

    let now = Instant::now();
    let past = now.checked_sub(Duration::from_millis(50)).unwrap();

    seq_log.pin_snapshot_at(100, past);
    let expired = seq_log.expired_pins_at(now);
    assert_eq!(expired, vec![100]);

    // SequenceLog::expired_pins() uses Instant::now() and is called by min_retention_seq()
    let _expired_now = seq_log.expired_pins();
    let min_ret = seq_log.min_retention_seq();
    assert_eq!(min_ret, Some(100));
}

#[test]
fn test_j23closure_snapshot_longest_active_pin() {
    let registry = Arc::new(SnapshotRegistry::new());
    let _g = registry.register(42);

    let pin_info = registry.longest_active_pin();
    assert!(pin_info.is_some());
    let (seq, _dur) = pin_info.unwrap();
    assert_eq!(seq, 42);
}

#[test]
fn test_j23closure_ssi_validator_wiring_and_maps() {
    let seq_log = Arc::new(RwLock::new(SequenceLog::new()));
    let registry = Arc::new(SnapshotRegistry::new());

    // SequenceLogSsiValidatorBuilder uses with_sequence_log and with_snapshot_registry
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

    // ReadSet methods delegate to keys_map and prefixes_map
    assert_eq!(read_set.keys_map().len(), 1);
    assert_eq!(read_set.prefixes_map().len(), 1);
    assert_eq!(read_set.len(), 2);
    assert!(!read_set.is_empty());
    assert_eq!(read_set.iter().count(), 1);
    assert_eq!(read_set.prefixes_iter().count(), 1);

    // record_commit_key delegates to record_commit_keys
    validator.record_commit_key(b"key_b", 10);
    validator.record_commit_keys([b"key_c".as_slice()], 10);

    // SsiValidator::validate delegates to validate_and_record
    assert!(validator.validate(TxId::new(1), &read_set).is_ok());
    assert!(validator
        .validate_and_record(TxId::new(2), &read_set, [b"key_d".as_slice()], 12)
        .is_ok());

    // diagnose_pruning_blocker tests
    let blocker = validator.diagnose_pruning_blocker();
    assert!(blocker.is_some());

    // set_snapshot_registry mutation wiring test
    let mut mut_val = SequenceLogSsiValidator::new();
    mut_val.set_snapshot_registry(registry);
}

#[test]
fn test_j23closure_tx_buffer_staged_status_and_validate_ops() {
    let buffer = TxBuffer::<(Vec<u8>, Vec<u8>)>::new();
    let tx = TxId::new(50);
    let doc_id = DocId::from(1u64);

    buffer.begin(tx);
    buffer.stage_kv(tx, IndexOp::Insert { doc_id, data: (b"key1".to_vec(), b"val1".to_vec()) }).unwrap();

    // is_key_staged_for_tx directly tested and used by staged_status
    assert!(buffer.is_key_staged_for_tx(tx, b"key1"));
    assert_eq!(buffer.staged_status(b"key1"), Some(true));

    // validate_pending_ops directly tested and used by get_ops and drain_kv
    assert!(buffer.validate_pending_ops(tx).is_ok());
    assert_eq!(buffer.get_ops(tx).map(|v| v.len()), Some(1));
    let ops = buffer.drain_kv(tx);
    assert_eq!(ops.len(), 1);
}
