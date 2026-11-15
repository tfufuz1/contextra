//! # Integrationstest: `audit_chain_tests.rs`
//!
//! Validiert die kryptographischen Invarianten und Sicherheitsanforderungen für `AuditChain`.
//! Prüft Blake3-Hash-Verkettung, Manipulationserkennung und Fail-Closed-Verhalten.

#![forbid(unsafe_code)]

use acta_crypto::audit_chain::AuditHeadAnchor;
use acta_crypto::{
    compute_record_commitment, AuditChain, DataClass,
};
use acta_types::{DocId, TxId};

#[test]
fn test_audit_chain_integrity_and_tamper_detection() {
    let mut chain = AuditChain::new();
    let rules_hash = [0xAAu8; 32];

    chain
        .append(
            "schema_v1",
            rules_hash,
            DataClass::Internal,
            DocId(101),
            TxId(1),
            None,
        )
        .unwrap();

    chain
        .append(
            "schema_v1",
            rules_hash,
            DataClass::Confidential,
            DocId(102),
            TxId(2),
            Some([0xBBu8; 32]),
        )
        .unwrap();

    chain
        .append(
            "schema_v1",
            rules_hash,
            DataClass::Public,
            DocId(103),
            TxId(3),
            None,
        )
        .unwrap();

    assert_eq!(chain.len(), 3);
    assert!(chain.verify_chain().unwrap());

    // Tamper with middle entry's doc_id
    let mut tampered_chain = chain.clone();
    tampered_chain.entries.get_mut(1).unwrap().doc_id = DocId(999);
    assert!(!tampered_chain.verify_chain().unwrap());

    // Tamper with middle entry's prev_hash
    let mut tampered_chain2 = chain.clone();
    tampered_chain2.entries.get_mut(1).unwrap().prev_hash = [0xFFu8; 32];
    assert!(!tampered_chain2.verify_chain().unwrap());

    // Tamper with rules_hash
    let mut tampered_chain3 = chain.clone();
    tampered_chain3.entries.get_mut(0).unwrap().rules_hash = [0x11u8; 32];
    assert!(!tampered_chain3.verify_chain().unwrap());
}

#[test]
fn test_audit_chain_anchor_verification() {
    let mut chain = AuditChain::new();
    chain
        .append(
            "schema_v1",
            [0x11u8; 32],
            DataClass::Public,
            DocId(1),
            TxId(10),
            None,
        )
        .unwrap();

    let head_hash = chain.head_hash();
    let anchor = AuditHeadAnchor::new(0, head_hash);
    assert!(chain.verify_chain_with_anchor(&anchor).unwrap());

    // Bad hash in anchor fails
    let bad_anchor = AuditHeadAnchor::new(0, [0xEEu8; 32]);
    assert!(!chain.verify_chain_with_anchor(&bad_anchor).unwrap());

    // Sequence out of bounds fails
    let oob_anchor = AuditHeadAnchor::new(5, head_hash);
    assert!(!chain.verify_chain_with_anchor(&oob_anchor).unwrap());
}

#[test]
fn test_record_commitment_blake3() {
    let salt = b"salt_bytes_32_bytes_long_secret!";
    let attr = b"user@example.com";
    let commitment = compute_record_commitment(salt, attr);
    assert_ne!(commitment, [0u8; 32]);

    let commitment2 = compute_record_commitment(salt, attr);
    assert_eq!(commitment, commitment2);

    let diff_commitment = compute_record_commitment(salt, b"other@example.com");
    assert_ne!(commitment, diff_commitment);
}
