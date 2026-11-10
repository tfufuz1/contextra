#![allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]

//! Golden format verification and backward compatibility tests for `contextra-license`.

use std::fs;
use std::path::Path;
use std::sync::Arc;

use contextra_license::{
    FeatureRing, FixedClock, LicenseGate, LicensePayload, OpenFastGate, SignedLicenseGate, TenantId,
};
use ed25519_dalek::VerifyingKey;

#[test]
fn test_golden_license_manifest_and_payload_exact() {
    let golden_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden");
    let manifest_path = golden_dir.join("MANIFEST.sha256");
    assert!(manifest_path.exists(), "MANIFEST.sha256 must exist");

    let manifest_str = fs::read_to_string(&manifest_path).expect("read manifest");
    for line in manifest_str.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        assert_eq!(parts.len(), 2, "Expected '<hash> <filename>'");
        let expected_hash = parts[0];
        let filename = parts[1];

        let _content = fs::read(golden_dir.join(filename)).expect("read golden file");
        let output = std::process::Command::new("sha256sum")
            .arg(golden_dir.join(filename))
            .output()
            .expect("sha256sum failed");
        let out_str = String::from_utf8_lossy(&output.stdout);
        let actual_hash = out_str.split_whitespace().next().unwrap_or("");
        assert_eq!(actual_hash, expected_hash, "Hash mismatch for {filename}");
    }
}

#[test]
fn test_backward_read_license_golden_files() {
    let golden_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden");
    let payload_bytes = fs::read(golden_dir.join("license_payload.bin")).expect("read payload");
    let sig_bytes = fs::read(golden_dir.join("license_signature.bin")).expect("read sig");
    let vk_bytes = fs::read(golden_dir.join("verifying_key.bin")).expect("read vk");
    let decisions_str =
        fs::read_to_string(golden_dir.join("gate_decisions.csv")).expect("read csv");

    let payload: LicensePayload = bincode::deserialize(&payload_bytes).expect("deserialization");
    assert_eq!(payload.tenant_id, TenantId::new(1337));
    assert_eq!(
        payload.allowed_rings,
        vec![FeatureRing::Fast, FeatureRing::Sovereign]
    );

    let mut vk_fixed = [0u8; 32];
    vk_fixed.copy_from_slice(&vk_bytes);
    let verifying_key = VerifyingKey::from_bytes(&vk_fixed).expect("verifying key");

    let mut sig_fixed = [0u8; 64];
    sig_fixed.copy_from_slice(&sig_bytes);

    let test_clock = Arc::new(FixedClock(1_700_000_000 * 1_000_000_000));
    let gate = SignedLicenseGate::from_signed_payload_with_clock(
        &payload_bytes,
        &sig_fixed,
        verifying_key,
        test_clock,
    )
    .expect("gate creation");

    assert_eq!(gate.check_ring(FeatureRing::Fast), Ok(()));
    assert_eq!(gate.check_ring(FeatureRing::Sovereign), Ok(()));
    assert!(gate.check_ring(FeatureRing::Compliance).is_err());

    let open_gate = OpenFastGate;
    assert_eq!(open_gate.check_ring(FeatureRing::Fast), Ok(()));
    assert!(open_gate.check_ring(FeatureRing::Sovereign).is_err());
    assert!(open_gate.check_ring(FeatureRing::Compliance).is_err());

    assert!(decisions_str.contains("signed,fast,true"));
    assert!(decisions_str.contains("signed,sovereign,true"));
    assert!(decisions_str.contains("signed,compliance,false"));
}

#[test]
fn test_golden_tamper_and_truncation_rejects_without_panic() {
    let golden_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden");
    let payload_bytes = fs::read(golden_dir.join("license_payload.bin")).expect("read payload");
    let sig_bytes = fs::read(golden_dir.join("license_signature.bin")).expect("read sig");
    let vk_bytes = fs::read(golden_dir.join("verifying_key.bin")).expect("read vk");

    let mut vk_fixed = [0u8; 32];
    vk_fixed.copy_from_slice(&vk_bytes);
    let verifying_key = VerifyingKey::from_bytes(&vk_fixed).expect("verifying key");

    let mut sig_fixed = [0u8; 64];
    sig_fixed.copy_from_slice(&sig_bytes);

    let test_clock = Arc::new(FixedClock(1_700_000_000 * 1_000_000_000));

    // 1. Bitflip in payload
    if !payload_bytes.is_empty() {
        let mut corrupted_payload = payload_bytes.clone();
        corrupted_payload[0] ^= 0x01;
        let res = SignedLicenseGate::from_signed_payload_with_clock(
            &corrupted_payload,
            &sig_fixed,
            verifying_key,
            test_clock.clone(),
        );
        assert!(
            res.is_err(),
            "Bitflipped payload must return Err, not panic"
        );
    }

    // 2. Truncated payload
    if payload_bytes.len() > 4 {
        let truncated_payload = &payload_bytes[..payload_bytes.len() / 2];
        let res = SignedLicenseGate::from_signed_payload_with_clock(
            truncated_payload,
            &sig_fixed,
            verifying_key,
            test_clock.clone(),
        );
        assert!(res.is_err(), "Truncated payload must return Err, not panic");
    }

    // 3. Bitflip in signature
    let mut corrupted_sig = sig_fixed;
    corrupted_sig[0] ^= 0x01;
    let res = SignedLicenseGate::from_signed_payload_with_clock(
        &payload_bytes,
        &corrupted_sig,
        verifying_key,
        test_clock,
    );
    assert!(
        res.is_err(),
        "Bitflipped signature must return Err, not panic"
    );
}
