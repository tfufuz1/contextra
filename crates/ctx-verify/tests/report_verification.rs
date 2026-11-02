//! Integration and contract tests for ctx-verify report verification.
//!
//! SPEC: T-PROOF-1, T-REPORT-1, §8.17, §11.7, Anhang D, F.9, F.10

use ctx_verify::canonical_json::to_canonical_json_string;
use ctx_verify::codes::InvalidCode;
use ctx_verify::journal::JournalChainEntry;
use ctx_verify::report::verify_report_bytes;
use ed25519_dalek::{Signer, SigningKey};
use rand::rngs::OsRng;
use rand::RngCore;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

fn create_valid_test_report() -> (SigningKey, serde_json::Value) {
    let mut csprng = OsRng;
    let mut secret_bytes = [0u8; 32];
    csprng.fill_bytes(&mut secret_bytes);
    let signing_key = SigningKey::from_bytes(&secret_bytes);
    let verifying_key = signing_key.verifying_key();

    let mut classes = BTreeMap::new();
    classes.insert("G".to_string(), "reached".to_string());
    classes.insert("K".to_string(), "reached".to_string());
    classes.insert("P".to_string(), "reached".to_string());

    let mut key_coverage = BTreeMap::new();
    key_coverage.insert("doc_store".to_string(), "full".to_string());
    key_coverage.insert("vector_payload".to_string(), "full".to_string());
    key_coverage.insert("text_postings".to_string(), "unlinked".to_string());

    let witness = serde_json::json!({
        "class": "MK1",
        "control_ok": true,
        "duration_us": 1200,
        "hash": "0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20",
        "layer": "store",
        "matches": 0,
        "method_id": "struct_scan_v1",
        "scanned_bytes": 4096,
        "scanned_files": 2
    });

    // Journal chain steps
    let prev_0 = [0u8; 32];
    let payload_0 = b"entry_0_data";
    let curr_0 = JournalChainEntry::compute_chain_step(&prev_0, payload_0);

    let journal_chain = vec![serde_json::json!({
        "prev_hash_hex": hex::encode(prev_0),
        "raw_entry_hex": hex::encode(payload_0),
        "current_hash_hex": hex::encode(curr_0)
    })];

    let residual_risks = vec![
        "FlashRemanence".to_string(),
        "WrappedKeyRemanence".to_string(),
        "FreedBlockRemanence".to_string(),
        "ReplicaWindow".to_string(),
        "ExternalCopies".to_string(),
        "ExternalProcessors".to_string(),
        "PseudonymCommitment".to_string(),
        "KeyCompromise".to_string(),
    ];

    let pub_fingerprint = {
        let mut hasher = Sha256::new();
        hasher.update(verifying_key.as_bytes());
        let digest = hasher.finalize();
        hex::encode(&digest[0..16])
    };

    let mut val = serde_json::json!({
        "bytes_rewritten": 0,
        "classes": classes,
        "commitment": "00112233445566778899aabbccddeeff",
        "completed_at": 4000,
        "erasure_id": "11112222333344445555666677778888",
        "field_measurement": [witness],
        "gated_at": 2000,
        "journal_chain": journal_chain,
        "key_coverage": key_coverage,
        "key_destroyed_at": 3000,
        "lab_evidence": "aabbccddeeff00112233445566778899aabbccddeeff00112233445566778899",
        "legal_notice": "Keine Rechtsberatung; die rechtliche Wertung obliegt dem Betreiber.",
        "process_hardening": "Full",
        "pub_fingerprint": pub_fingerprint,
        "report_version": 1,
        "requested_at": 1000,
        "residual_risks": residual_risks,
        "retention_code": "Tax10Y",
        "segments_rewritten": 0,
        "signature_scope_notice": "Die Signatur beweist Integrität und Herkunft aus dieser Installation, nicht die Wahrheit des Inhalts.",
        "sla_us": 5000000,
        "sla_violation": false,
        "store_id": "abcdef0123456789abcdef0123456789",
        "target_type": "Document"
    });

    let unsigned_canonical = to_canonical_json_string(&val).unwrap();
    let mut msg = Vec::new();
    msg.extend_from_slice(b"ctx/report/v1");
    msg.extend_from_slice(unsigned_canonical.as_bytes());

    let sig = signing_key.sign(&msg);
    val.as_object_mut().unwrap().insert(
        "signature".to_string(),
        serde_json::Value::String(hex::encode(sig.to_bytes())),
    );

    (signing_key, val)
}

/// SPEC: T-PROOF-1
#[test]
fn t_proof_1_valid_report_accepted() {
    let (signing_key, val) = create_valid_test_report();
    let canonical = to_canonical_json_string(&val).unwrap();
    let res = verify_report_bytes(canonical.as_bytes(), &signing_key.verifying_key(), None);
    assert_eq!(res, Ok(()));
}

/// SPEC: T-PROOF-1
#[test]
fn t_proof_1_byte_manipulation_rejected() {
    let (signing_key, val) = create_valid_test_report();
    let mut canonical = to_canonical_json_string(&val).unwrap().into_bytes();

    // Mutate a byte in the payload
    if let Some(b) = canonical.get_mut(15) {
        *b ^= 0x01;
    }

    let res = verify_report_bytes(&canonical, &signing_key.verifying_key(), None);
    assert!(res.is_err(), "manipulated report must be rejected");
}

/// SPEC: T-PROOF-1
#[test]
fn t_proof_1_missing_positive_control_rejected() {
    let (signing_key, mut val) = create_valid_test_report();
    val["field_measurement"][0]["control_ok"] = serde_json::Value::Bool(false);

    // Resign
    let mut unsigned = val.clone();
    unsigned.as_object_mut().unwrap().remove("signature");
    let unsigned_canonical = to_canonical_json_string(&unsigned).unwrap();
    let mut msg = Vec::new();
    msg.extend_from_slice(b"ctx/report/v1");
    msg.extend_from_slice(unsigned_canonical.as_bytes());
    let sig = signing_key.sign(&msg);
    val.as_object_mut().unwrap().insert(
        "signature".to_string(),
        serde_json::Value::String(hex::encode(sig.to_bytes())),
    );

    let canonical = to_canonical_json_string(&val).unwrap();
    let res = verify_report_bytes(canonical.as_bytes(), &signing_key.verifying_key(), None);
    assert_eq!(res, Err(InvalidCode::MissingPositiveControl));
}

/// SPEC: T-PROOF-1
#[test]
fn t_proof_1_broken_hash_chain_rejected() {
    let (signing_key, mut val) = create_valid_test_report();
    val["journal_chain"][0]["current_hash_hex"] = serde_json::Value::String(
        "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".to_string(),
    );

    // Resign
    let mut unsigned = val.clone();
    unsigned.as_object_mut().unwrap().remove("signature");
    let unsigned_canonical = to_canonical_json_string(&unsigned).unwrap();
    let mut msg = Vec::new();
    msg.extend_from_slice(b"ctx/report/v1");
    msg.extend_from_slice(unsigned_canonical.as_bytes());
    let sig = signing_key.sign(&msg);
    val.as_object_mut().unwrap().insert(
        "signature".to_string(),
        serde_json::Value::String(hex::encode(sig.to_bytes())),
    );

    let canonical = to_canonical_json_string(&val).unwrap();
    let res = verify_report_bytes(canonical.as_bytes(), &signing_key.verifying_key(), None);
    assert_eq!(res, Err(InvalidCode::BrokenHashChain));
}

/// SPEC: T-REPORT-1
#[test]
fn t_report_1_missing_residual_risk_rejected() {
    let (signing_key, mut val) = create_valid_test_report();
    // Remove FlashRemanence
    val["residual_risks"] = serde_json::json!(["KeyCompromise"]);

    // Resign
    let mut unsigned = val.clone();
    unsigned.as_object_mut().unwrap().remove("signature");
    let unsigned_canonical = to_canonical_json_string(&unsigned).unwrap();
    let mut msg = Vec::new();
    msg.extend_from_slice(b"ctx/report/v1");
    msg.extend_from_slice(unsigned_canonical.as_bytes());
    let sig = signing_key.sign(&msg);
    val.as_object_mut().unwrap().insert(
        "signature".to_string(),
        serde_json::Value::String(hex::encode(sig.to_bytes())),
    );

    let canonical = to_canonical_json_string(&val).unwrap();
    let res = verify_report_bytes(canonical.as_bytes(), &signing_key.verifying_key(), None);
    assert_eq!(res, Err(InvalidCode::IncompleteResidualRisks));
}

/// SPEC: T-PROOF-1
#[test]
fn t_proof_1_temporal_order_violation_rejected() {
    let (signing_key, mut val) = create_valid_test_report();
    // completed_at before requested_at
    val["completed_at"] = serde_json::Value::Number(serde_json::Number::from(500));

    // Resign
    let mut unsigned = val.clone();
    unsigned.as_object_mut().unwrap().remove("signature");
    let unsigned_canonical = to_canonical_json_string(&unsigned).unwrap();
    let mut msg = Vec::new();
    msg.extend_from_slice(b"ctx/report/v1");
    msg.extend_from_slice(unsigned_canonical.as_bytes());
    let sig = signing_key.sign(&msg);
    val.as_object_mut().unwrap().insert(
        "signature".to_string(),
        serde_json::Value::String(hex::encode(sig.to_bytes())),
    );

    let canonical = to_canonical_json_string(&val).unwrap();
    let res = verify_report_bytes(canonical.as_bytes(), &signing_key.verifying_key(), None);
    assert_eq!(res, Err(InvalidCode::TemporalOrderViolation));
}
