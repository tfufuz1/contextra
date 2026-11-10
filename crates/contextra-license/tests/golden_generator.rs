#![allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]

//! Generator for deterministic golden files in `contextra-license`.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;

use contextra_license::{
    FeatureRing, FixedClock, LicenseGate, LicensePayload, OpenFastGate, SignedLicenseGate, TenantId,
};
use ed25519_dalek::{Signer, SigningKey};

#[test]
#[ignore]
fn generate_license_golden_corpus() {
    let out_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden");
    fs::create_dir_all(&out_dir).expect("Failed to create golden dir");

    // 1. Deterministic keypair
    let seed = [42u8; 32];
    let signing_key = SigningKey::from_bytes(&seed);
    let verifying_key = signing_key.verifying_key();

    // 2. Fixed inputs
    let payload = LicensePayload {
        tenant_id: TenantId::new(1337),
        allowed_rings: vec![FeatureRing::Fast, FeatureRing::Sovereign],
        expires_at: Some(1_800_000_000), // Year 2027
        feature_flags: BTreeMap::from([
            ("audit_export".to_string(), true),
            ("sovereign_storage".to_string(), true),
        ]),
        installation_id_hash: None,
    };

    let payload_bytes = bincode::serialize(&payload).expect("bincode serialize");
    let signature = signing_key.sign(&payload_bytes);

    fs::write(out_dir.join("license_payload.bin"), &payload_bytes).expect("write payload bin");
    fs::write(out_dir.join("license_signature.bin"), signature.to_bytes()).expect("write sig bin");
    fs::write(out_dir.join("verifying_key.bin"), verifying_key.to_bytes()).expect("write vk bin");

    // Gate decisions
    let test_clock = Arc::new(FixedClock(1_700_000_000 * 1_000_000_000));

    let gate = SignedLicenseGate::from_signed_payload_with_clock(
        &payload_bytes,
        &signature.to_bytes(),
        verifying_key,
        test_clock,
    )
    .expect("gate creation");

    let decision_fast = gate.check_ring(FeatureRing::Fast).is_ok();
    let decision_sovereign = gate.check_ring(FeatureRing::Sovereign).is_ok();
    let decision_compliance = gate.check_ring(FeatureRing::Compliance).is_ok();

    let open_gate = OpenFastGate;
    let open_fast = open_gate.check_ring(FeatureRing::Fast).is_ok();
    let open_sov = open_gate.check_ring(FeatureRing::Sovereign).is_ok();
    let open_comp = open_gate.check_ring(FeatureRing::Compliance).is_ok();

    let mut dec_csv = String::new();
    dec_csv.push_str("gate,ring,decision\n");
    dec_csv.push_str(&format!("signed,fast,{decision_fast}\n"));
    dec_csv.push_str(&format!("signed,sovereign,{decision_sovereign}\n"));
    dec_csv.push_str(&format!("signed,compliance,{decision_compliance}\n"));
    dec_csv.push_str(&format!("open_fast,fast,{open_fast}\n"));
    dec_csv.push_str(&format!("open_fast,sovereign,{open_sov}\n"));
    dec_csv.push_str(&format!("open_fast,compliance,{open_comp}\n"));

    fs::write(out_dir.join("gate_decisions.csv"), dec_csv.as_bytes()).expect("write decisions csv");
}
