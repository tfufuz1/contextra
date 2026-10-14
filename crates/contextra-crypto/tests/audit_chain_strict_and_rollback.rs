// FILE-CONTEXT
// ZWECK: Integration tests for strict Ed25519 verification and rollback detection against external anchors in Blake3 Audit Chain.
// INVARIANTEN: Truncated, reordered, or rolled-back chains are strictly rejected. Non-canonical Ed25519 signatures (S >= L) fail verify_strict.

#![forbid(unsafe_code)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_crypto::{AuditChain, AuditChainHeadSignature, DataClass, DeletionProofKeyPair};
use contextra_types::{DocId, TxId};

#[test]
fn test_verify_against_anchor_valid_chain() {
    let mut chain = AuditChain::new();
    let rules_hash = [0x11u8; 32];

    chain
        .append(
            "schema_v1",
            rules_hash,
            DataClass::Internal,
            DocId(1),
            TxId(100),
            None,
        )
        .unwrap();

    chain
        .append(
            "schema_v1",
            rules_hash,
            DataClass::Confidential,
            DocId(2),
            TxId(101),
            None,
        )
        .unwrap();

    let head_hash = chain.head_hash();
    let head_counter = 1;

    // External anchor matching head_hash and head_counter=1
    assert!(chain
        .verify_against_anchor(&head_hash, head_counter)
        .is_ok());

    // Older index anchor is also verified if entry hash matches
    let entry0_hash = chain.entries[0].entry_hash;
    assert!(chain.verify_against_anchor(&entry0_hash, 0).is_ok());
}

#[test]
fn test_verify_against_anchor_detects_rollback_truncated_chain() {
    let mut full_chain = AuditChain::new();
    let rules_hash = [0x22u8; 32];

    full_chain
        .append(
            "schema_v1",
            rules_hash,
            DataClass::Internal,
            DocId(10),
            TxId(200),
            None,
        )
        .unwrap();

    full_chain
        .append(
            "schema_v1",
            rules_hash,
            DataClass::Internal,
            DocId(11),
            TxId(201),
            None,
        )
        .unwrap();

    full_chain
        .append(
            "schema_v1",
            rules_hash,
            DataClass::Internal,
            DocId(12),
            TxId(202),
            None,
        )
        .unwrap();

    // External anchor recorded at full chain state (counter = 2)
    let anchor_hash = full_chain.head_hash();
    let anchor_counter = 2;

    // Simulate rollback: Tail entries are truncated (e.g. restoring older snapshot with only 1 entry)
    let mut truncated_chain = AuditChain::new();
    truncated_chain
        .append(
            "schema_v1",
            rules_hash,
            DataClass::Internal,
            DocId(10),
            TxId(200),
            None,
        )
        .unwrap();

    // Truncated chain head index is 0, anchor counter is 2 -> MUST fail
    let err = truncated_chain
        .verify_against_anchor(&anchor_hash, anchor_counter)
        .unwrap_err();

    let err_msg = err.to_string();
    assert!(
        err_msg.contains("rollback detected") || err_msg.contains("greater than chain head index"),
        "Unexpected error message: {err_msg}"
    );
}

#[test]
fn test_verify_against_anchor_detects_reordering_or_gaps() {
    let mut chain = AuditChain::new();
    let rules_hash = [0x33u8; 32];

    chain
        .append(
            "schema_v1",
            rules_hash,
            DataClass::Internal,
            DocId(1),
            TxId(1),
            None,
        )
        .unwrap();

    chain
        .append(
            "schema_v1",
            rules_hash,
            DataClass::Internal,
            DocId(2),
            TxId(2),
            None,
        )
        .unwrap();

    let head_hash = chain.head_hash();

    // 1. Reorder entries in chain
    let mut reordered_chain = chain.clone();
    reordered_chain.entries.swap(0, 1);

    assert!(reordered_chain
        .verify_against_anchor(&head_hash, 1)
        .is_err());

    // 2. Tamper entry index gap
    let mut gap_chain = chain.clone();
    gap_chain.entries[1].index = 5;

    assert!(gap_chain.verify_against_anchor(&head_hash, 1).is_err());
}

#[test]
fn test_verify_strict_rejects_non_canonical_signature() {
    let keypair = DeletionProofKeyPair::generate();
    let mut chain = AuditChain::new();

    chain
        .append(
            "schema_v1",
            [0x44u8; 32],
            DataClass::Public,
            DocId(1),
            TxId(1),
            None,
        )
        .unwrap();

    let valid_head_sig = chain.sign_head(keypair.signing_key()).unwrap();

    // Valid signature verifies
    assert!(AuditChain::verify_head_signature(&valid_head_sig, keypair.verifying_key()).unwrap());

    // Construct non-canonical Ed25519 signature where S >= L (e.g., S = L or all 0xFFs in second 32 bytes)
    // L in little-endian bytes:
    let l_bytes: [u8; 32] = [
        0xed, 0xd3, 0xf5, 0x5c, 0x1a, 0x63, 0x12, 0x58, 0xd6, 0x9c, 0xf7, 0xa2, 0xde, 0xf9, 0xde,
        0x14, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x10,
    ];

    let mut non_canonical_sig_bytes = vec![0u8; 64];
    // Copy valid R (first 32 bytes)
    non_canonical_sig_bytes[..32].copy_from_slice(&valid_head_sig.signature[..32]);
    // Set S = L (which is non-canonical, S >= L)
    non_canonical_sig_bytes[32..64].copy_from_slice(&l_bytes);

    let non_canonical_head_sig = AuditChainHeadSignature {
        chain_index: valid_head_sig.chain_index,
        head_hash: valid_head_sig.head_hash,
        signature: non_canonical_sig_bytes,
    };

    // verify_strict must reject S >= L signature
    let res = AuditChain::verify_head_signature(&non_canonical_head_sig, keypair.verifying_key())
        .unwrap();
    assert!(
        !res,
        "Non-canonical Ed25519 signature (S >= L) must be rejected by verify_strict"
    );
}
