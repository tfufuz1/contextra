// ANCHOR[contextra-mvcc:TEST-SSI-WRITE-SKEW-ROOT]
//! # Write-Skew Isolations- und Konflikttests
//!
//! Demonstriert die klassische Write-Skew-Anomalie und stellt sicher, dass der
//! SSI-Validator die zweite Transaktion deterministisch abbricht.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_mvcc::{ContextraError, SequenceLogSsiValidator, SsiValidator, TxBuffer, TxId};

// Prüft die Erkennung und Abweisung des klassischen Write-Skews bei überkreuzten Lese- und Schreibzugriffen.
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

// Prüft, dass disjunkte Lese- und Schreibmengen ohne Fehlalarm erfolgreich committen.
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

// Prüft die Erkennung von Anti-Dependency-Zyklen (Write-Skew) bei atomarem validate_and_record ohne Storage-Kopplung.
// Szenario: Bankkonto-Invariante Balance(A) + Balance(B) >= 0.
// Beide Transaktionen lesen Konten A und B. Tx1 hebt von A ab, Tx2 hebt von B ab.
// Beide heben so viel ab, dass jede Transaktion für sich gültig wäre, aber gemeinsam die Invariante brechen würden.
#[test]
fn test_storage_independent_anti_dependency_cycle_detection() {
    let validator = SequenceLogSsiValidator::new();
    let tx1 = TxId::new(301);
    let tx2 = TxId::new(302);

    let key_account_a = b"bank:acc_a".to_vec();
    let key_account_b = b"bank:acc_b".to_vec();

    // Beide Transaktionen starten mit logischer Snapshot-Sequenz 10
    let snapshot_seq = 10;

    let mut read_set_1 = contextra_mvcc::ReadSet::new();
    read_set_1.record_read(key_account_a.clone(), snapshot_seq);
    read_set_1.record_read(key_account_b.clone(), snapshot_seq);

    let mut read_set_2 = contextra_mvcc::ReadSet::new();
    read_set_2.record_read(key_account_a.clone(), snapshot_seq);
    read_set_2.record_read(key_account_b.clone(), snapshot_seq);

    // Tx1 modifiziert Account A und validiert/committet atomar bei Sequenz 12
    let write_keys_1 = [key_account_a.as_slice()];
    let res1 = validator.validate_and_record(tx1, &read_set_1, write_keys_1, 12);
    assert!(res1.is_ok(), "Tx1 muss erfolgreich validieren und committen");

    // Tx2 versucht Account B bei Sequenz 13 zu verändern und zu committen.
    // Da Tx2 jedoch Account A gelesen hat (Snapshot 10), welches bei Seq 12 von Tx1 modifiziert wurde,
    // MUSS die Validierung von Tx2 mit einem Konflikt abbrechen (Anti-Dependency-Cycle unterbrochen).
    let write_keys_2 = [key_account_b.as_slice()];
    let res2 = validator.validate_and_record(tx2, &read_set_2, write_keys_2, 13);
    assert!(
        res2.is_err(),
        "Tx2 muss deterministisch abgewiesen werden, um Write-Skew zu verhindern"
    );
    match res2 {
        Err(ContextraError::Conflict(msg)) => {
            assert!(
                msg.contains("bank:acc_a") || msg.contains("modified"),
                "Konfliktmeldung muss den kollidierenden Schlüssel anzeigen: {msg}"
            );
        }
        other => panic!("Erwartete ContextraError::Conflict, erhalten: {other:?}"),
    }
}
