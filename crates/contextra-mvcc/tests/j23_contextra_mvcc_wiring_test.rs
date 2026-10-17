#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! Integration tests verifying wiring and production reachability for the 13 symbols in campaign J23-contextra-mvcc.

use contextra_mvcc::{
    DocId, IndexOp, ReadSet, SequenceLog, SequenceLogSsiValidator, SnapshotRegistry, SsiValidator,
    TxBuffer, TxId,
};
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::Duration;

#[test]
fn test_j23_seq_log_expired_pins_and_set_max_pin_duration() {
    let seq_log = SequenceLog::new();
    let past_ticks = 100_000_000u64; // 100ms in nanos

    let seq_log_arc = Arc::new(RwLock::new(seq_log));
    let _validator = SequenceLogSsiValidator::builder()
        .with_sequence_log(seq_log_arc.clone())
        .with_max_pin_duration(Duration::from_millis(10))
        .build();

    seq_log_arc.write().pin_snapshot_at(100, past_ticks);

    let min_ret = seq_log_arc.read().min_retention_seq_at(past_ticks + 50_000_000);
    assert_eq!(min_ret, Some(100));
}

#[test]
fn test_j23_snapshot_longest_active_pin_via_ssi() {
    let registry = Arc::new(SnapshotRegistry::new());
    let _g = registry.acquire(|| 42);

    let validator = SequenceLogSsiValidator::builder()
        .with_snapshot_registry(registry.clone())
        .build();

    validator.record_commit_key(b"k1", 10);

    let blocker_populated = validator.diagnose_pruning_blocker();
    assert!(blocker_populated.is_some());
    let info = blocker_populated.unwrap();
    assert_eq!(info.longest_active_snapshot_seq, Some(42));
}

#[test]
fn test_j23_ssi_validator_wiring_and_read_set_maps() {
    let seq_log = Arc::new(RwLock::new(SequenceLog::new()));
    let registry = Arc::new(SnapshotRegistry::new());

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

    assert_eq!(read_set.len(), 2);
    assert!(!read_set.is_empty());
    assert_eq!(read_set.iter().count(), 1);
    assert_eq!(read_set.prefixes_iter().count(), 1);

    validator.record_commit_key(b"key_b", 10);

    assert!(validator.validate(TxId::new(1), &read_set).is_ok());

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

    assert_eq!(buffer.staged_status(b"key1"), Some(true));

    assert_eq!(buffer.get_ops(tx).map(|v| v.len()), Some(1));
    let ops = buffer.drain_kv(tx);
    assert_eq!(ops.len(), 1);
}
