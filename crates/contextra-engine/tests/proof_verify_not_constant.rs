use contextra_crypto::deletion_proof::{CustodyLevel, DeletionProof};
use contextra_ports::scanner::ScanTechnique;
use contextra_ports::SystemClock;
use contextra_testkit::mint_witness_for_tests;
use contextra_types::{RequestContext, SagaRecord, TenantId, TxId};

#[test]
fn test_proof_verify_with_wrong_key_fails() {
    let clock = SystemClock::new();
    let ctx = RequestContext::new(TenantId::try_new(1).unwrap(), TxId(100));
    let saga = SagaRecord::new([0xAA; 32]);
    let real_key = b"correct_key_32_bytes_long_secret!";
    let wrong_key = b"wrong___key_32_bytes_long_secret!";

    let witness = vec![mint_witness_for_tests(
        "LsmMemtable",
        10,
        0,
        ScanTechnique::Structural,
    )];

    let proof = DeletionProof::issue(
        &ctx,
        &saga,
        &witness,
        &[],
        CustodyLevel::Local,
        real_key.as_slice(),
        &clock,
    )
    .unwrap();

    let is_valid = proof.verify(wrong_key.as_slice()).unwrap_or(false);
    assert!(
        !is_valid,
        "Verification with wrong key must return false, but returned true"
    );
}
