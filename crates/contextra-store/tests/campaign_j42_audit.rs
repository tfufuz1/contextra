//! Campaign J-42: Audit Chain Separation & Chain Integrity Verification Test Suite
//! Oracle Source (R4): Cryptographic Hash Chain Invariants (RFC 6962 / Spec §4.2 / ENG-023)
//! counter-test (R10): Verified against entry payload tampering (tampered entry fails chain verification).

use contextra_crypto::{AuditChain, DataClass, DeletionProofKeyPair};
use contextra_types::{DocId, TxId};

#[test]
#[allow(clippy::expect_used)]
fn test_campaign_j42_audit_chain_append_integrity_oracle() {
    let mut chain = AuditChain::new();
    let keypair = DeletionProofKeyPair::generate();
    let rules_hash = [0xAAu8; 32];

    // Append 3 distinct audit entries
    chain
        .append(
            "schema_v1",
            rules_hash,
            DataClass::Public,
            DocId::new(100),
            TxId::new(1),
            None,
        )
        .expect("Append entry 1");

    chain
        .append(
            "schema_v1",
            rules_hash,
            DataClass::Internal,
            DocId::new(200),
            TxId::new(2),
            None,
        )
        .expect("Append entry 2");

    chain
        .append(
            "schema_v1",
            rules_hash,
            DataClass::Confidential,
            DocId::new(300),
            TxId::new(3),
            None,
        )
        .expect("Append entry 3");

    assert_eq!(chain.entries().len(), 3);

    // Oracle Check 1: Chain hash link integrity MUST pass
    let chain_valid = chain.verify_chain().expect("Chain verification");
    assert!(chain_valid, "Untampered chain entries must be valid");

    // Oracle Check 2: Sign head and verify signature
    let head_sig = chain
        .sign_head(keypair.signing_key())
        .expect("Head signature generation must succeed");

    let sig_valid = AuditChain::verify_head_signature(&head_sig, &keypair.verifying_key)
        .expect("Head signature verification function execution");
    assert!(sig_valid, "AuditChain head signature must be valid");
}

#[test]
#[allow(clippy::unwrap_used, clippy::expect_used)]
fn test_campaign_j42_audit_chain_counter_factual_tamper_check() {
    // R10 Counter-Test: Tampering with an entry's payload in a signed chain MUST cause verify_chain() to return false.
    let mut chain = AuditChain::new();
    let _keypair = DeletionProofKeyPair::generate();
    let rules_hash = [0xBBu8; 32];

    chain
        .append(
            "schema_v1",
            rules_hash,
            DataClass::Public,
            DocId::new(1),
            TxId::new(1),
            None,
        )
        .expect("Append");

    // Before tampering, verify_chain is true
    assert!(chain.verify_chain().unwrap());

    // Counter-factual: Modify an entry's schema_id after signing
    chain.entries[0].schema_id = "schema_TAMPERED".to_string();

    // Verification MUST now return false
    let verified = chain.verify_chain().expect("Chain integrity verification");

    assert!(
        !verified,
        "R10 Counter-Fact: Tampering with audit entry payload MUST invalidate chain integrity"
    );
}
