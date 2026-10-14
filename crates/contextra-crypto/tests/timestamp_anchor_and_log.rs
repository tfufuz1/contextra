// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: Integrationstests für RFC 3161 Zeitanker und RFC 9162 Transparenzlog.

use contextra_crypto::deletion_proof::timestamp_anchor::{
    AnchorQualification, DerTstInfoParser, LocalMonotonicAnchor, NoOpTsaSignatureVerifier,
    Rfc3161TimeAnchor, TimeAnchor,
};
use contextra_crypto::deletion_proof::transparency_log::{
    hash_salted_proof_id, verify_consistency_proof, verify_inclusion_proof, TransparencyLog,
};
use proptest::prelude::*;

#[test]
fn test_local_monotonic_anchor_basic() {
    let anchor = LocalMonotonicAnchor::default();
    let msg_imprint = b"test_imprint_bytes_1234567890";

    let receipt = anchor
        .anchor(msg_imprint)
        .expect("Anchoring should succeed");

    assert_eq!(receipt.message_imprint, msg_imprint);
    assert_eq!(receipt.qualification, AnchorQualification::NotQualified);
    assert_eq!(receipt.anchor_type, "LocalMonotonic");
    assert!(!receipt.serial_number.is_empty());
}

#[test]
fn test_local_monotonic_anchor_empty_input_fails() {
    let anchor = LocalMonotonicAnchor::default();
    let err = anchor.anchor(b"").unwrap_err();
    assert!(err.to_string().contains("cannot be empty"));
}

#[test]
fn test_der_parser_rejects_truncated_or_empty_inputs() {
    assert!(DerTstInfoParser::parse_tst_info(b"").is_err());
    assert!(DerTstInfoParser::parse_tst_info(b"\x30\x05\x02\x01").is_err());

    // Indefinite length (0x80)
    assert!(DerTstInfoParser::parse_tst_info(b"\x30\x80\x02\x01\x01").is_err());

    // Oversized input (> 64KB)
    let huge_der = vec![0u8; 65537];
    assert!(DerTstInfoParser::parse_tst_info(&huge_der).is_err());
}

#[test]
fn test_rfc3161_anchor_parse_and_verify() {
    let mut der = Vec::new();
    let imprint_bytes = b"01234567890123456789012345678901"; // 32 bytes
    let mut mi_seq = Vec::new();
    let alg_oid = [0x60, 0x86, 0x48, 0x01, 0x65, 0x03, 0x04, 0x02, 0x01];
    mi_seq.push(0x30);
    mi_seq.push(11);
    mi_seq.push(0x06);
    mi_seq.push(alg_oid.len() as u8);
    mi_seq.extend_from_slice(&alg_oid);
    mi_seq.push(0x04);
    mi_seq.push(32);
    mi_seq.extend_from_slice(imprint_bytes);

    let mut payload = Vec::new();
    payload.extend_from_slice(&[0x02, 0x01, 0x01]);
    payload.extend_from_slice(&[0x06, 0x03, 0x2A, 0x03, 0x04]);
    payload.push(0x30);
    payload.push(mi_seq.len() as u8);
    payload.extend_from_slice(&mi_seq);
    payload.extend_from_slice(&[0x02, 0x01, 0x2A]);
    let gen_time_bytes = b"20261013120000Z";
    payload.push(0x18);
    payload.push(gen_time_bytes.len() as u8);
    payload.extend_from_slice(gen_time_bytes);

    der.push(0x30);
    der.push(payload.len() as u8);
    der.extend_from_slice(&payload);

    let verifier = NoOpTsaSignatureVerifier;
    let rfc_anchor = Rfc3161TimeAnchor::new(verifier);

    let receipt = rfc_anchor
        .parse_and_verify(&der, b"dummy_sig", imprint_bytes)
        .expect("Valid TSTInfo must parse and verify successfully");

    assert_eq!(receipt.policy_oid, "1.2.3.4");
    assert_eq!(receipt.message_imprint, imprint_bytes);
    assert_eq!(receipt.serial_number, vec![0x2A]);
    assert_eq!(receipt.qualification, AnchorQualification::Qualified);
    assert_eq!(receipt.anchor_type, "RFC3161-TSA");
}

#[test]
fn test_transparency_log_salted_hashes_only() {
    let salt = b"c_sprng_salt_1234567890_32bytes!!";
    let proof_id1 = b"proof_uuid_0001";
    let proof_id2 = b"proof_uuid_0002";

    let leaf1 = hash_salted_proof_id(proof_id1, salt);
    let leaf2 = hash_salted_proof_id(proof_id2, salt);

    let log = TransparencyLog::new(vec![leaf1, leaf2]);
    assert_eq!(log.tree_size(), 2);

    let root = log.root_hash();
    assert_ne!(root, [0u8; 32]);
}

#[test]
fn test_transparency_log_inclusion_proof_valid_and_tampered() {
    let salt = b"salt_key_1234567890_32bytes_sec!";
    let sizes = [1, 2, 3, 4, 7, 8, 15, 16];

    for &sz in &sizes {
        let leaves: Vec<[u8; 32]> = (0..sz)
            .map(|i| hash_salted_proof_id(format!("proof_{}", i).as_bytes(), salt))
            .collect();

        let log = TransparencyLog::new(leaves.clone());
        let root = log.root_hash();

        for i in 0..sz {
            let proof = log.inclusion_proof(i).expect("Inclusion proof generation");
            let valid = verify_inclusion_proof(&leaves[i], &proof, &root)
                .expect("Inclusion proof verification");
            assert!(
                valid,
                "Inclusion proof for leaf {} in tree size {} must be valid",
                i, sz
            );

            // Tampered leaf hash must fail verification
            let mut tampered_leaf = leaves[i];
            tampered_leaf[0] ^= 0xFF;
            let tampered_valid = verify_inclusion_proof(&tampered_leaf, &proof, &root)
                .expect("Verification must return result");
            assert!(!tampered_valid, "Tampered leaf hash must not verify");
        }
    }
}

#[test]
fn test_transparency_log_consistency_proof_and_rollback_detection() {
    let salt = b"salt_key_1234567890_32bytes_sec!";
    let leaves_v1: Vec<[u8; 32]> = (0..4)
        .map(|i| hash_salted_proof_id(format!("proof_{}", i).as_bytes(), salt))
        .collect();

    let leaves_v2: Vec<[u8; 32]> = (0..7)
        .map(|i| hash_salted_proof_id(format!("proof_{}", i).as_bytes(), salt))
        .collect();

    let log_v1 = TransparencyLog::new(leaves_v1);
    let log_v2 = TransparencyLog::new(leaves_v2);

    let root_v1 = log_v1.root_hash();
    let root_v2 = log_v2.root_hash();

    let consistency_proof = log_v2
        .consistency_proof(4)
        .expect("Consistency proof generation");

    assert_eq!(consistency_proof.first_tree_size, 4);
    assert_eq!(consistency_proof.second_tree_size, 7);

    let is_consistent = verify_consistency_proof(&consistency_proof, &root_v1, &root_v2)
        .expect("Consistency verification should execute");
    assert!(is_consistent, "Log v1 and v2 must be consistent");

    // Tampered root_v1 must fail consistency check
    let mut tampered_root_v1 = root_v1;
    tampered_root_v1[0] ^= 0xFF;
    let is_tampered_consistent =
        verify_consistency_proof(&consistency_proof, &tampered_root_v1, &root_v2)
            .expect("Consistency verification execution");
    assert!(
        !is_tampered_consistent,
        "Tampered root_v1 must fail consistency verification"
    );

    // Rollback attempt (claiming tree size > current size must fail)
    let err = log_v2.consistency_proof(8);
    assert!(err.is_err(), "Tree size > current size must fail");
}

proptest! {
    #[test]
    fn proptest_der_parser_never_panics_on_arbitrary_bytes(data in proptest::collection::vec(any::<u8>(), 0..1024)) {
        let _ = DerTstInfoParser::parse_tst_info(&data);
    }
}
