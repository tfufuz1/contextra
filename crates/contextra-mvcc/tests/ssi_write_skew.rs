#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_mvcc::{ContextraError, SequenceLogSsiValidator, SsiValidator, TxBuffer, TxId};

#[test]
fn test_classic_write_skew_conflict_detected() {
    let validator = SequenceLogSsiValidator::new();
    let buffer = TxBuffer::<String>::new();

    let tx1 = TxId::new(100);
    let tx2 = TxId::new(101);

    buffer.begin(tx1);
    buffer.begin(tx2);

    // Initial snapshot sequence number for both transactions: seq = 10
    let snapshot_seq = 10;

    let key_a = b"account_a".to_vec();
    let key_b = b"account_b".to_vec();

    // Tx1 reads Key A at snapshot_seq = 10
    buffer.register_read(tx1, key_a.clone(), snapshot_seq);

    // Tx2 reads Key B at snapshot_seq = 10
    buffer.register_read(tx2, key_b.clone(), snapshot_seq);

    // Tx1 stages a write to Key B and commits at commit_seq = 15
    let rs1 = buffer.get_read_set(tx1).expect("tx1 read set");
    // Tx1 validates successfully (nobody modified key_a since seq 10)
    assert!(validator.validate(tx1, &rs1).is_ok());

    // Tx1 commits write to Key B at commit_seq = 15
    validator.record_commit_key(&key_b, 15);
    buffer.drain(tx1);

    // Tx2 stages a write to Key A and attempts to commit at commit_seq = 20
    let rs2 = buffer.get_read_set(tx2).expect("tx2 read set");

    // Tx2 validation MUST fail with Conflict because Key B was modified at commit_seq 15 > snapshot_seq 10
    let validation_res = validator.validate(tx2, &rs2);
    assert!(
        validation_res.is_err(),
        "Tx2 validation must fail due to write skew on key_b"
    );

    match validation_res {
        Err(ContextraError::Conflict(msg)) => {
            assert!(
                msg.contains("account_b") || msg.contains("15") || msg.contains("modified"),
                "Conflict message must detail key mutation: {msg}"
            );
        }
        res => panic!("Expected ContextraError::Conflict, got: {res:?}"),
    }
}

#[test]
fn test_disjoint_read_write_sets_succeed() {
    let validator = SequenceLogSsiValidator::new();
    let buffer = TxBuffer::<String>::new();

    let tx1 = TxId::new(200);
    let tx2 = TxId::new(201);

    buffer.begin(tx1);
    buffer.begin(tx2);

    let snapshot_seq = 10;

    let key_a = b"key_a".to_vec();
    let key_b = b"key_b".to_vec();
    let key_c = b"key_c".to_vec();
    let key_d = b"key_d".to_vec();

    // Tx1 reads Key A and writes Key B
    buffer.register_read(tx1, key_a.clone(), snapshot_seq);

    // Tx2 reads Key C and writes Key D
    buffer.register_read(tx2, key_c.clone(), snapshot_seq);

    // Tx1 validates and commits
    let rs1 = buffer.get_read_set(tx1).expect("tx1 read set");
    assert!(validator.validate(tx1, &rs1).is_ok());
    validator.record_commit_key(&key_b, 15);
    buffer.drain(tx1);

    // Tx2 validates and commits — read set (Key C) is disjoint from Tx1 writes (Key B)
    let rs2 = buffer.get_read_set(tx2).expect("tx2 read set");
    let validation_res2 = validator.validate(tx2, &rs2);
    assert!(
        validation_res2.is_ok(),
        "Disjoint transaction Tx2 must validate successfully"
    );

    validator.record_commit_key(&key_d, 20);
    buffer.drain(tx2);
}
