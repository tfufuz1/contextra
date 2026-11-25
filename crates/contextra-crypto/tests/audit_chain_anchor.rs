#![forbid(unsafe_code)]

use contextra_crypto::audit_chain::{AuditChain, DataClass};
use contextra_types::{DocId, TxId};

#[test]
fn test_empty_chain_anchor_is_none() {
    let chain = AuditChain::new();
    assert_eq!(chain.anchor(), None);
}

#[test]
fn test_grown_chain_against_older_anchor_is_valid() {
    let mut chain = AuditChain::new();

    chain
        .append(
            "schema_v1",
            [1u8; 32],
            DataClass::Internal,
            DocId(100),
            TxId(1000),
            None,
        )
        .unwrap();

    let anchor = chain.anchor().expect("anchor should exist");

    // Grow the chain further
    chain
        .append(
            "schema_v1",
            [1u8; 32],
            DataClass::Internal,
            DocId(101),
            TxId(1001),
            None,
        )
        .unwrap();

    let res = chain.verify_chain_against_anchor(&anchor).unwrap();
    assert!(res, "Grown chain should be valid against older anchor");
}

#[test]
fn test_rolled_back_shorter_chain_fails_anchor_verification() {
    let mut chain = AuditChain::new();

    chain
        .append(
            "schema_v1",
            [1u8; 32],
            DataClass::Internal,
            DocId(100),
            TxId(1000),
            None,
        )
        .unwrap();

    chain
        .append(
            "schema_v1",
            [1u8; 32],
            DataClass::Internal,
            DocId(101),
            TxId(1001),
            None,
        )
        .unwrap();

    // Take anchor of length-2 chain
    let anchor = chain.anchor().expect("anchor should exist");

    // Simulate backup rollback to length-1 state
    let mut rolled_back_chain = AuditChain::new();
    rolled_back_chain
        .append(
            "schema_v1",
            [1u8; 32],
            DataClass::Internal,
            DocId(100),
            TxId(1000),
            None,
        )
        .unwrap();

    let res = rolled_back_chain
        .verify_chain_against_anchor(&anchor)
        .unwrap();
    assert!(
        !res,
        "Rolled back shorter chain must fail verification against newer anchor"
    );
}

#[test]
fn test_forked_chain_same_length_differing_hash_fails_anchor_verification() {
    let mut original_chain = AuditChain::new();

    original_chain
        .append(
            "schema_v1",
            [1u8; 32],
            DataClass::Internal,
            DocId(100),
            TxId(1000),
            None,
        )
        .unwrap();

    let anchor = original_chain.anchor().expect("anchor should exist");

    // Forked chain at index 0 with different doc_id
    let mut forked_chain = AuditChain::new();
    forked_chain
        .append(
            "schema_v1",
            [1u8; 32],
            DataClass::Internal,
            DocId(999),
            TxId(1000),
            None,
        )
        .unwrap();

    let res = forked_chain.verify_chain_against_anchor(&anchor).unwrap();
    assert!(
        !res,
        "Forked chain with differing entry hash at anchor index must fail"
    );
}

#[test]
fn test_tampered_chain_fails_anchor_verification() {
    let mut chain = AuditChain::new();

    chain
        .append(
            "schema_v1",
            [1u8; 32],
            DataClass::Internal,
            DocId(100),
            TxId(1000),
            None,
        )
        .unwrap();

    let anchor = chain.anchor().expect("anchor should exist");

    // Tamper with data in entry 0
    chain.entries[0].doc_id = DocId(999);

    let res = chain.verify_chain_against_anchor(&anchor).unwrap();
    assert!(!res, "Tampered chain must fail verify_chain_against_anchor");
}
