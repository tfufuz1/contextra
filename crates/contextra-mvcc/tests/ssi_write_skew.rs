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

// Cahill / Fekete Benchmark 1: Write-Skew Anomalie "Doctor on Call" (Cahill et al., TODS 2009).
// Mindestens ein Arzt muss im Dienst bleiben. Zwei Ärzte versuchen gleichzeitig sich abzumelden.
// Bei gewöhnlicher Snapshot-Isolation melden sich beide ab (Invariante verletzt).
// SSI nach Cahill erkennt die rw-Antidependenz und bricht die zweite Transaktion ab.
#[test]
fn test_cahill_write_skew_doctor_on_call() {
    let validator = SequenceLogSsiValidator::new();
    let buffer = TxBuffer::<String>::new();

    let tx_doc1 = TxId::new(301);
    let tx_doc2 = TxId::new(302);

    buffer.begin(tx_doc1);
    buffer.begin(tx_doc2);

    let snapshot_seq = 100;
    let doctor1_status = b"doctor_1_on_call".to_vec();
    let doctor2_status = b"doctor_2_on_call".to_vec();

    // Beide Ärzte lesen den Bereitschaftsstatus beider Kollegen
    buffer.register_read(tx_doc1, doctor1_status.clone(), snapshot_seq);
    buffer.register_read(tx_doc1, doctor2_status.clone(), snapshot_seq);

    buffer.register_read(tx_doc2, doctor1_status.clone(), snapshot_seq);
    buffer.register_read(tx_doc2, doctor2_status.clone(), snapshot_seq);

    // Arzt 1 meldet sich ab (ändert doctor1_status auf false) und validiert
    let rs1 = buffer.get_read_set(tx_doc1).expect("read set doc1");
    assert!(validator.validate(tx_doc1, &rs1).is_ok());
    validator.record_commit_key(&doctor1_status, 110);
    buffer.drain(tx_doc1);

    // Arzt 2 meldet sich gleichzeitig ab (ändert doctor2_status auf false)
    let rs2 = buffer.get_read_set(tx_doc2).expect("read set doc2");
    let val_doc2 = validator.validate(tx_doc2, &rs2);

    // Invariante: Tx Doc 2 MUSS mit Conflict abgebrochen werden, da doctor1_status modifiziert wurde
    assert!(
        val_doc2.is_err(),
        "Cahill Doctor-on-call write skew must be detected and aborted"
    );
    assert!(matches!(val_doc2, Err(ContextraError::Conflict(_))));
}

// Cahill / Fekete Benchmark 2: "Credit Limit / Bank Account Transfer" (Cahill et al.).
// Kontostand A + Kontostand B >= 0. Tx1 bucht von A ab, Tx2 bucht von B ab.
// Bei SI können beide abbuchen und das gemeinsame Kreditlimit überschreiten.
// Unter SSI bricht die zweite Transaktion deterministisch ab.
#[test]
fn test_cahill_credit_limit_transfer() {
    let validator = SequenceLogSsiValidator::new();
    let buffer = TxBuffer::<String>::new();

    let tx_transfer1 = TxId::new(401);
    let tx_transfer2 = TxId::new(402);

    buffer.begin(tx_transfer1);
    buffer.begin(tx_transfer2);

    let snapshot_seq = 50;
    let acct_a = b"bank_account_a".to_vec();
    let acct_b = b"bank_account_b".to_vec();

    // Beide lesen beide Konten zur Salden- und Kreditlimitprüfung
    buffer.register_read(tx_transfer1, acct_a.clone(), snapshot_seq);
    buffer.register_read(tx_transfer1, acct_b.clone(), snapshot_seq);

    buffer.register_read(tx_transfer2, acct_a.clone(), snapshot_seq);
    buffer.register_read(tx_transfer2, acct_b.clone(), snapshot_seq);

    // Tx1 bucht von A ab und committet bei seq 60
    let rs1 = buffer.get_read_set(tx_transfer1).expect("rs tx1");
    assert!(validator.validate(tx_transfer1, &rs1).is_ok());
    validator.record_commit_key(&acct_a, 60);
    buffer.drain(tx_transfer1);

    // Tx2 bucht von B ab und versucht bei seq 70 zu committen
    let rs2 = buffer.get_read_set(tx_transfer2).expect("rs tx2");
    let val_tx2 = validator.validate(tx_transfer2, &rs2);

    assert!(
        val_tx2.is_err(),
        "Cahill Credit limit transfer write skew must be detected and aborted"
    );
    assert!(matches!(val_tx2, Err(ContextraError::Conflict(_))));
}

// Cahill Benchmark 3: Phantom-Prädikat mit Bereichskonflikten (Range / Prefix Read Set).
#[test]
fn test_cahill_phantom_predicate() {
    let validator = SequenceLogSsiValidator::new();
    let buffer = TxBuffer::<String>::new();

    let tx_scan = TxId::new(501);
    let tx_insert = TxId::new(502);

    buffer.begin(tx_scan);
    buffer.begin(tx_insert);

    let snapshot_seq = 20;
    let dept_prefix = b"employees/dept_eng/".to_vec();

    // Tx_scan liest Bereich (Prefix-Read via ReadSet)
    let mut rs_scan = contextra_mvcc::ReadSet::new();
    rs_scan.record_prefix(dept_prefix.clone(), snapshot_seq);

    // Tx_insert fügt einen neuen Mitarbeiter in diese Abteilung ein und committet bei seq 25
    let new_employee = b"employees/dept_eng/emp_99".to_vec();
    validator.record_commit_key(&new_employee, 25);
    buffer.drain(tx_insert);

    // Tx_scan validiert: Muss Konflikt wegen Phantom-Insert im gelesenen Präfix erkennen
    let val_scan = validator.validate(tx_scan, &rs_scan);

    assert!(
        val_scan.is_err(),
        "Phantom insert under prefix must conflict with active prefix read"
    );
    assert!(matches!(val_scan, Err(ContextraError::Conflict(_))));
}
