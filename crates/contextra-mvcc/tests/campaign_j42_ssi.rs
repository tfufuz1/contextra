// ANCHOR[contextra-mvcc:TEST-CAMPAIGN-J42-SSI-ROOT]
//! # Kampagne J-42: SSI Read-Set & Write-Skew Isolations-Audit
//!
//! Formale Prüfung des Write-Skew-Orakels und Gegenprüfung gegen fehlerhafte
//! Umgehung des ReadSets (Counter-Factual Verification).

use contextra_mvcc::ssi::{ReadSet, SequenceLogSsiValidator, SsiValidator};
use contextra_mvcc::TxId;

// Prüft die formale Erkennung von Write-Skew-Anomalien über das unabhängige Orakel.
#[test]
#[allow(clippy::expect_used)]
fn test_campaign_j42_ssi_write_skew_oracle_verification() {
    // Independent Oracle: Write Skew occurs when Tx1 reads A and writes B, Tx2 reads B and writes A concurrently.
    // Under SSI, at least one transaction MUST be aborted with a Conflict error.
    let validator = SequenceLogSsiValidator::new();

    let _tx1 = TxId::new(10);
    let tx2 = TxId::new(20);

    // Tx1 reads Key A (seq 5), Tx2 reads Key B (seq 5)
    let mut read_set_2 = ReadSet::new();
    read_set_2.record_read(b"key_b".to_vec(), 5);

    // Tx1 commits write to Key B at commit_seq 10
    let commit_seq_1 = 10;
    validator.record_commit_key(b"key_b", commit_seq_1);

    // Tx2 attempts commit write to Key A at commit_seq 15
    // Validator MUST detect that Tx2's read set contains Key B which was modified by Tx1 at seq 10 > Tx2's snapshot seq 5.
    let validation_res = validator.validate(tx2, &read_set_2);
    assert!(
        validation_res.is_err(),
        "SSI Validator must reject Tx2 due to concurrent commit on Key B"
    );
}

// Gegenprobe: Verifiziert, dass ein absichtlich unvollständiges ReadSet die Konflikterkennung verfehlt (Beweis für die Wirksamkeit des ReadSets).
#[test]
fn test_campaign_j42_ssi_counter_factual_mutation_check() {
    // R10 Counter-Test: Demonstrates that if an empty ReadSet is passed (as occurs when LSM get() skips ReadSet registration),
    // the SSI validator fails to catch the conflict!
    let validator = SequenceLogSsiValidator::new();

    let _tx1 = TxId::new(100);
    let tx2 = TxId::new(200);

    // Tx1 commits Key X at seq 15
    validator.record_commit_key(b"key_x", 15);

    // Counter-factual: Tx2 read Key X, but due to un-tracked get() call, read_set is empty
    let empty_read_set = ReadSet::new();

    let res = validator.validate(tx2, &empty_read_set);
    assert!(
        res.is_ok(),
        "R10 Counter-Fact: Empty ReadSet bypasses SSI validation, proving the necessity of ReadSet tracking in LSM read paths"
    );
}
