//! Umfassende TDD-, Invarianten- und Sabotagetests fuer contextra-erasure-verify.
//!
//! Entspricht SPEC §8.17, §11.7, Anhang D, F.9, F.10 und Baukasten-Anforderungen.

use contextra_erasure_verify::canonical_json::verify_canonical_json;
use contextra_erasure_verify::codes::InvalidCode;
use contextra_erasure_verify::report::{hex_encode, parse_public_key, verify_report_bytes};
use contextra_erasure_verify::scan::{execute_field_scan, FieldScanOptions};
use ed25519_dalek::{Signer, SigningKey};
use rand::rngs::OsRng;
use serde_json::json;
use std::fs::File;
use std::io::Write;
use tempfile::tempdir;

fn create_sample_keys() -> (SigningKey, ed25519_dalek::VerifyingKey, String) {
    let mut csprng = OsRng;
    let signing_key = SigningKey::generate(&mut csprng);
    let verifying_key = signing_key.verifying_key();
    let fp = {
        use sha2::{Digest, Sha256};
        let mut h = Sha256::new();
        h.update(verifying_key.as_bytes());
        let d = h.finalize();
        hex_encode(&d[0..16])
    };
    (signing_key, verifying_key, fp)
}

fn create_report_json(
    signing_key: &SigningKey,
    fp: &str,
    control_ok: bool,
    matches: u64,
    completed_at: u64,
) -> String {
    let unsigned_val = json!({
        "bytes_rewritten": 0,
        "classes": { "lsm": "Purged" },
        "commitment": "c0ffee",
        "completed_at": completed_at,
        "erasure_id": "era-1",
        "field_measurement": [{
            "class": "M",
            "control_ok": control_ok,
            "duration_us": 12,
            "hash": "abc",
            "layer": "store",
            "matches": matches,
            "method_id": "raw_scan",
            "scanned_bytes": 1024,
            "scanned_files": 1
        }],
        "gated_at": 950,
        "key_coverage": { "store": "Covered" },
        "key_destroyed_at": 980,
        "lab_evidence": "verified",
        "legal_notice": "no-legal-advice",
        "process_hardening": "Enabled",
        "pub_fingerprint": fp,
        "report_version": 5,
        "requested_at": 900,
        "retention_code": "RET-01",
        "residual_risks": [
            "FlashRemanence",
            "FreedBlockRemanence",
            "ReplicaWindow",
            "ExternalCopies",
            "ExternalProcessors",
            "PseudonymCommitment",
            "KeyCompromise"
        ],
        "segments_rewritten": 0,
        "signature_scope_notice": "scope",
        "sla_us": 100000,
        "sla_violation": false,
        "store_id": "store-1",
        "target_type": "Document"
    });

    let unsigned_canonical =
        contextra_erasure_verify::canonical_json::to_canonical_json_string(&unsigned_val).unwrap();

    let mut message = Vec::new();
    message.extend_from_slice(b"ctx/report/v1");
    message.extend_from_slice(unsigned_canonical.as_bytes());
    let sig = signing_key.sign(&message);
    let sig_hex = hex_encode(&sig.to_bytes());

    let mut signed_map = unsigned_val.as_object().unwrap().clone();
    signed_map.insert("signature".to_string(), serde_json::Value::String(sig_hex));
    let signed_val = serde_json::Value::Object(signed_map);
    contextra_erasure_verify::canonical_json::to_canonical_json_string(&signed_val).unwrap()
}

fn create_valid_report_json(signing_key: &SigningKey, fp: &str) -> String {
    create_report_json(signing_key, fp, true, 0, 1000)
}

/// SPEC §8.17 / §11.7: Gueltiger Bericht verifiziert erfolgreich.
#[test]
fn t_verify_strict_signature_valid() {
    let (signing_key, verifying_key, fp) = create_sample_keys();
    let report_json = create_valid_report_json(&signing_key, &fp);

    let parsed_vk = parse_public_key(&verifying_key.to_bytes()).unwrap();
    let res = verify_report_bytes(report_json.as_bytes(), &parsed_vk);
    assert!(res.is_ok());
}

/// SPEC §8.17 / §11.7: Bericht mit manipulierter Signatur wird strikt abgewiesen.
#[test]
fn t_verify_strict_signature_tampered_fails() {
    let (signing_key, verifying_key, fp) = create_sample_keys();
    let report_json = create_valid_report_json(&signing_key, &fp);

    // Letztes Hex-Zeichen der Signatur manipulieren
    let mut tampered = report_json.clone();
    let last_idx = tampered.rfind('"').unwrap();
    tampered.replace_range(last_idx - 1..last_idx, "0");

    let parsed_vk = parse_public_key(&verifying_key.to_bytes()).unwrap();
    let res = verify_report_bytes(tampered.as_bytes(), &parsed_vk);
    assert!(matches!(
        res,
        Err(InvalidCode::BadSignature) | Err(InvalidCode::NonCanonicalJson)
    ));
}

/// SPEC F.10: Floats im JSON sind verboten und muessen abgewiesen werden.
#[test]
fn t_canonical_json_floats_rejected() {
    let json_with_float = json!({
        "score": 1.23,
        "name": "test"
    });
    let s = serde_json::to_string(&json_with_float).unwrap();
    assert_eq!(
        verify_canonical_json(s.as_bytes()),
        Err(InvalidCode::NonCanonicalJson)
    );
}

/// SPEC F.10: Unsortierte JSON-Keys oder Whitespace verletzen Kanonizitaet.
#[test]
fn t_canonical_json_whitespace_rejected() {
    let whitespace_json = b"{ \"a\": 1, \"b\": 2 }";
    assert_eq!(
        verify_canonical_json(whitespace_json),
        Err(InvalidCode::NonCanonicalJson)
    );
}

/// SPEC §11.7: Ein Zeuge ohne bestandene Positivkontrolle macht den Bericht ungueltig.
#[test]
fn t_positive_control_mandatory_in_witness() {
    let (signing_key, verifying_key, fp) = create_sample_keys();
    let report_json = create_report_json(&signing_key, &fp, false, 0, 1000);

    let parsed_vk = parse_public_key(&verifying_key.to_bytes()).unwrap();
    let res = verify_report_bytes(report_json.as_bytes(), &parsed_vk);
    assert_eq!(res, Err(InvalidCode::MissingPositiveControl));
}

/// SPEC §11.7: Ein Zeuge mit Rest-Treffern (matches > 0) fuehrt zu WitnessMatchesNonZero.
#[test]
fn t_witness_matches_nonzero_fails() {
    let (signing_key, verifying_key, fp) = create_sample_keys();
    let report_json = create_report_json(&signing_key, &fp, true, 1, 1000);

    let parsed_vk = parse_public_key(&verifying_key.to_bytes()).unwrap();
    let res = verify_report_bytes(report_json.as_bytes(), &parsed_vk);
    assert_eq!(res, Err(InvalidCode::WitnessMatchesNonZero));
}

/// SPEC §11.7: Verletzung der Zeitstempel-Monotonie wird abgewiesen.
#[test]
fn t_temporal_order_violation_fails() {
    let (signing_key, verifying_key, fp) = create_sample_keys();
    // completed_at < requested_at (800 < 900)
    let report_json = create_report_json(&signing_key, &fp, true, 0, 800);

    let parsed_vk = parse_public_key(&verifying_key.to_bytes()).unwrap();
    let res = verify_report_bytes(report_json.as_bytes(), &parsed_vk);
    assert_eq!(res, Err(InvalidCode::TemporalOrderViolation));
}

/// SPEC §12.1 / §8.17: Feldscan findet eingepflanzten Klartextrest (MK2-Detektion).
#[test]
fn t_residual_scan_planted_needle() {
    let dir = tempdir().unwrap();
    let canary_needle = b"CANARY_TEST_OK";
    let target_needle = b"SECRET_TARGET_DATA";
    let target_hex = hex_encode(target_needle);

    // 1. Canary-Datei anlegen
    let canary_path = dir.path().join("canary.bin");
    let mut cf = File::create(&canary_path).unwrap();
    cf.write_all(canary_needle).unwrap();

    // 2. Kontaminierte Datei mit Zielrest anlegen
    let dirty_path = dir.path().join("segment_001.sst");
    let mut df = File::create(&dirty_path).unwrap();
    df.write_all(b"header_data_prefix_").unwrap();
    df.write_all(target_needle).unwrap();
    df.write_all(b"_trailer").unwrap();

    let scan_opts = FieldScanOptions {
        data_dir: dir.path(),
        target_tag_hex: &target_hex,
        control_needle: Some(canary_needle),
        passphrase_fd: None,
    };

    let res = execute_field_scan(scan_opts);
    assert_eq!(res, Err(InvalidCode::Mk2RawResidue));
}

/// SPEC §12.1 / §8.17: Feldscan findet nicht entwerteten Vault-Schluesselslot (MK3-Detektion).
#[test]
fn t_residual_scan_planted_vault_slot_active() {
    let dir = tempdir().unwrap();
    let canary_needle = b"CANARY_TEST_OK";
    let target_tag = [0x42u8; 16];
    let target_hex = hex_encode(&target_tag);

    // 1. Canary-Datei anlegen
    let canary_path = dir.path().join("canary.bin");
    let mut cf = File::create(&canary_path).unwrap();
    cf.write_all(canary_needle).unwrap();

    // 2. Vault-Datei mit aktivem Slot fuer target_tag anlegen (Format F.3)
    let vault_path = dir.path().join("00.vlt");
    let mut vf = File::create(&vault_path).unwrap();
    let mut vault_bytes = vec![0u8; 128];
    let mut slot = vec![0u8; 64];
    slot[0..16].copy_from_slice(&target_tag);
    slot[48] = 0x01; // AKTIV!
    vault_bytes.extend_from_slice(&slot);

    vf.write_all(&vault_bytes).unwrap();

    let scan_opts = FieldScanOptions {
        data_dir: dir.path(),
        target_tag_hex: &target_hex,
        control_needle: Some(canary_needle),
        passphrase_fd: None,
    };

    let res = execute_field_scan(scan_opts);
    assert_eq!(res, Err(InvalidCode::Mk3KeyPresent));
}

/// SPEC §12.1: Feldscan ohne vorhandene Positivkontrolle ist zwingend ungueltig.
#[test]
fn t_field_scan_missing_positive_control_fails() {
    let dir = tempdir().unwrap();
    let canary_needle = b"NON_EXISTENT_CANARY";
    let target_needle = b"TARGET_NOT_PRESENT";
    let target_hex = hex_encode(target_needle);

    // Beliebige saubere Datei
    let clean_path = dir.path().join("clean.dat");
    let mut f = File::create(&clean_path).unwrap();
    f.write_all(b"clean content without canary").unwrap();

    let scan_opts = FieldScanOptions {
        data_dir: dir.path(),
        target_tag_hex: &target_hex,
        control_needle: Some(canary_needle),
        passphrase_fd: None,
    };

    let res = execute_field_scan(scan_opts);
    assert_eq!(res, Err(InvalidCode::MissingPositiveControl));
}
