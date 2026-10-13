#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! Integration test matrix for MVCC SSI validation, multi-tenant isolation, fail-closed behavior, and P7/P8 invariants.

use std::sync::Arc;
use std::time::{Duration, Instant};

use contextra_types::{TenantId, TxId};
use contextra_mvcc::{
    CommitDecision, MvccWiring, MvccWiringDeps, ReadSet, SequenceLog,
    SnapshotRegistry, SsiError, TxBuffer, TxSsiState, TxStateStatus,
};

#[test]
fn test_ssi_write_skew_cahill_fekete_rejection() {
    let wiring = MvccWiring::build(MvccWiringDeps::default()).expect("MvccWiring build");
    let seq_log = SequenceLog::new();

    let tenant_id = TenantId::try_new(1).expect("valid tenant");
    let tx1_id = TxId::new(101);
    let tx2_id = TxId::new(102);

    // Initial snapshot sequence is 10
    let snapshot_seq = 10u64;

    // T1 reads key A ("account_a") at snapshot_seq 10, writes key B ("account_b")
    let mut rs1 = ReadSet::new();
    rs1.record_read(b"account_a".to_vec(), snapshot_seq);
    let tx1_state = TxSsiState::new(tx1_id, tenant_id, 15)
        .with_read_set(rs1)
        .with_write_keys(vec![b"account_b".to_vec()]);

    // T2 reads key B ("account_b") at snapshot_seq 10, writes key A ("account_a")
    let mut rs2 = ReadSet::new();
    rs2.record_read(b"account_b".to_vec(), snapshot_seq);
    let tx2_state = TxSsiState::new(tx2_id, tenant_id, 16)
        .with_read_set(rs2)
        .with_write_keys(vec![b"account_a".to_vec()]);

    // T1 commits first -> Accepted
    let decision1 = wiring
        .ssi_commit_gate
        .validate_and_register(&tx1_state, &seq_log)
        .expect("T1 validation decision");
    assert_eq!(decision1, CommitDecision::Accepted { commit_seq: 15 });

    // T2 attempts to commit second -> Must be Aborted due to write-skew ($rw$-antidependency on account_b)
    let decision2 = wiring
        .ssi_commit_gate
        .validate_and_register(&tx2_state, &seq_log)
        .expect("T2 validation decision");

    match decision2 {
        CommitDecision::Aborted { reason } => {
            assert!(
                reason.contains("conflict") || reason.contains("102"),
                "Expected conflict reason, found: {reason}"
            );
        }
        CommitDecision::Accepted { .. } => {
            panic!("T2 write-skew transaction must be rejected by SsiCommitGate!");
        }
    }
}

#[test]
fn test_ssi_serial_schedule_accepted() {
    let wiring = MvccWiring::build(MvccWiringDeps::default()).expect("MvccWiring build");
    let seq_log = SequenceLog::new();
    let tenant_id = TenantId::try_new(1).expect("valid tenant");

    let tx1_id = TxId::new(201);
    let tx2_id = TxId::new(202);

    // T1 reads key A at snapshot 10, writes key B at commit 15
    let mut rs1 = ReadSet::new();
    rs1.record_read(b"key_a".to_vec(), 10);
    let tx1_state = TxSsiState::new(tx1_id, tenant_id, 15)
        .with_read_set(rs1)
        .with_write_keys(vec![b"key_b".to_vec()]);

    let decision1 = wiring
        .ssi_commit_gate
        .validate_and_register(&tx1_state, &seq_log)
        .expect("T1 decision");
    assert_eq!(decision1, CommitDecision::Accepted { commit_seq: 15 });

    // T2 starts after T1 committed (reads at snapshot_seq 15), reads key B, writes key C
    let mut rs2 = ReadSet::new();
    rs2.record_read(b"key_b".to_vec(), 15);
    let tx2_state = TxSsiState::new(tx2_id, tenant_id, 20)
        .with_read_set(rs2)
        .with_write_keys(vec![b"key_c".to_vec()]);

    let decision2 = wiring
        .ssi_commit_gate
        .validate_and_register(&tx2_state, &seq_log)
        .expect("T2 decision");
    assert_eq!(decision2, CommitDecision::Accepted { commit_seq: 20 });
}

#[test]
fn test_two_tenant_isolation() {
    let wiring = MvccWiring::build(MvccWiringDeps::default()).expect("MvccWiring build");
    let seq_log = SequenceLog::new();

    let tenant1 = TenantId::try_new(1).expect("tenant 1");
    let tenant2 = TenantId::try_new(2).expect("tenant 2");

    let tx1_id = TxId::new(301);
    let tx2_id = TxId::new(302);

    // Tenant 1 reads tenant-prefixed key
    let mut rs1 = ReadSet::new();
    rs1.record_read(b"t1:doc_a".to_vec(), 10);
    let tx1_state = TxSsiState::new(tx1_id, tenant1, 15)
        .with_read_set(rs1)
        .with_write_keys(vec![b"t1:doc_b".to_vec()]);

    // Tenant 2 writes tenant-prefixed key at sequence 12
    let mut rs2 = ReadSet::new();
    rs2.record_read(b"t2:doc_x".to_vec(), 10);
    let tx2_state = TxSsiState::new(tx2_id, tenant2, 12)
        .with_read_set(rs2)
        .with_write_keys(vec![b"t2:doc_a".to_vec()]);

    // T2 commits first
    let decision2 = wiring
        .ssi_commit_gate
        .validate_and_register(&tx2_state, &seq_log)
        .expect("T2 decision");
    assert_eq!(decision2, CommitDecision::Accepted { commit_seq: 12 });

    // T1 commits second: no conflict because keyspace prefixes are isolated
    let decision1 = wiring
        .ssi_commit_gate
        .validate_and_register(&tx1_state, &seq_log)
        .expect("T1 decision");
    assert_eq!(decision1, CommitDecision::Accepted { commit_seq: 15 });
}

#[test]
fn test_fail_closed_on_missing_read_tracking_and_unknown_state() {
    let wiring = MvccWiring::build(MvccWiringDeps::default()).expect("MvccWiring build");
    let seq_log = SequenceLog::new();
    let tenant_id = TenantId::try_new(1).expect("valid tenant");

    // Case 1: Read tracking disabled -> Must return SsiError::MissingReadTracking
    let tx_no_tracking = TxSsiState::new(TxId::new(401), tenant_id, 10)
        .with_read_tracking(false);

    let err1 = wiring
        .ssi_commit_gate
        .validate_and_register(&tx_no_tracking, &seq_log)
        .expect_err("Must fail closed when read tracking is disabled");

    assert!(
        matches!(err1, SsiError::MissingReadTracking { tx_id } if tx_id == TxId::new(401)),
        "Expected MissingReadTracking error, found: {err1:?}"
    );

    // Case 2: Invalid status (Aborted or Unknown) -> Must return SsiError::UnknownTxState
    let tx_aborted = TxSsiState::new(TxId::new(402), tenant_id, 10)
        .with_status(TxStateStatus::Aborted);

    let err2 = wiring
        .ssi_commit_gate
        .validate_and_register(&tx_aborted, &seq_log)
        .expect_err("Must fail closed when transaction status is Aborted");

    assert!(
        matches!(err2, SsiError::UnknownTxState { tx_id, status: TxStateStatus::Aborted } if tx_id == TxId::new(402)),
        "Expected UnknownTxState error, found: {err2:?}"
    );
}

#[test]
fn test_p7_lock_hierarchy_deadlock_free_sweeps() {
    let buffer = TxBuffer::<Vec<u8>>::new_with_config(16, Duration::from_secs(30));

    for i in 1..=20u64 {
        let tx = TxId::new(i);
        buffer.begin(tx);
        buffer.register_read(tx, format!("key_{i}").into_bytes(), i * 10);
    }

    assert_eq!(buffer.len(), 20);
    assert!(!buffer.is_empty());
    assert_eq!(buffer.min_read_snapshot(), Some(10));

    let reaped = buffer.reap_orphans();
    assert!(reaped.is_empty());
}

#[test]
fn test_p8_orphan_reaper_snapshot_unpinning_integration() {
    let registry = Arc::new(SnapshotRegistry::new());
    let mut buffer = TxBuffer::<Vec<u8>>::new_with_config(4, Duration::from_millis(10));
    buffer.set_snapshot_registry(registry.clone());

    let tx = TxId::new(501);
    let past_time = Instant::now() - Duration::from_secs(60);
    buffer.begin_with_seq_at(tx, 77, past_time);

    // Confirm that snapshot sequence 77 is active
    assert_eq!(registry.min_active_seqno(), 77);

    // Reap expired orphan transaction
    let reaped = buffer.reap_orphans_at(Instant::now());
    assert_eq!(reaped, vec![tx]);

    // Confirm that snapshot sequence 77 was unpinned from SnapshotRegistry
    assert_eq!(registry.min_active_seqno(), u64::MAX);
}
