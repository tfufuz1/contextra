//! Integration tests for Ed25519 WASM Module Admission Control (§4.18).

use contextra_sandbox::{
    admission::Ed25519ModuleVerifier,
    capabilities::{ModulePolicy, WasmCapabilities},
    error::{AdmissionError, SandboxError},
    ModuleVerifier,
};
use ed25519_dalek::SigningKey;

fn sample_wat_bytes() -> Vec<u8> {
    wat::parse_str("(module (func (export \"run\")))").expect("valid WAT payload")
}

#[test]
fn test_ed25519_module_verifier_valid_signature_success() {
    let signing_key = SigningKey::from_bytes(&[1u8; 32]);
    let public_key = signing_key.verifying_key().to_bytes();

    let wasm_bytes = sample_wat_bytes();
    let signed_payload = Ed25519ModuleVerifier::sign_payload(&signing_key, &wasm_bytes);

    let verifier = Ed25519ModuleVerifier::new(vec![public_key]);
    let admitted = verifier
        .verify(&signed_payload)
        .expect("valid signed module must be admitted");

    assert_eq!(admitted.bytes(), &wasm_bytes[..]);
    let expected_digest = WasmCapabilities::compute_sha256(&wasm_bytes);
    assert_eq!(admitted.digest(), &expected_digest);
}

#[test]
fn test_ed25519_module_verifier_tampered_payload_rejected() {
    let signing_key = SigningKey::from_bytes(&[1u8; 32]);
    let public_key = signing_key.verifying_key().to_bytes();

    let wasm_bytes = sample_wat_bytes();
    let mut signed_payload = Ed25519ModuleVerifier::sign_payload(&signing_key, &wasm_bytes);

    // Tamper with payload byte after signature
    let last_idx = signed_payload.len() - 1;
    signed_payload[last_idx] ^= 0xFF;

    let verifier = Ed25519ModuleVerifier::new(vec![public_key]);
    let err = verifier
        .verify(&signed_payload)
        .expect_err("tampered payload must be rejected");

    assert!(
        matches!(err, AdmissionError::InvalidSignature(_)),
        "Expected InvalidSignature, got {err:?}"
    );
}

#[test]
fn test_ed25519_module_verifier_tampered_signature_rejected() {
    let signing_key = SigningKey::from_bytes(&[1u8; 32]);
    let public_key = signing_key.verifying_key().to_bytes();

    let wasm_bytes = sample_wat_bytes();
    let mut signed_payload = Ed25519ModuleVerifier::sign_payload(&signing_key, &wasm_bytes);

    // Flip a bit in the signature header (first 64 bytes)
    signed_payload[0] ^= 0x01;

    let verifier = Ed25519ModuleVerifier::new(vec![public_key]);
    let err = verifier
        .verify(&signed_payload)
        .expect_err("tampered signature header must be rejected");

    assert!(
        matches!(err, AdmissionError::InvalidSignature(_)),
        "Expected InvalidSignature, got {err:?}"
    );
}

#[test]
fn test_ed25519_module_verifier_unsigned_payload_rejected() {
    let wasm_bytes = sample_wat_bytes();
    let public_key = [0u8; 32];

    let verifier = Ed25519ModuleVerifier::new(vec![public_key]);
    let err = verifier
        .verify(&wasm_bytes)
        .expect_err("unsigned raw wasm payload (<64 bytes header) must return UnsignedModule error");

    assert_eq!(err, AdmissionError::UnsignedModule);
}

#[test]
fn test_ed25519_module_verifier_untrusted_key_rejected() {
    let signing_key = SigningKey::from_bytes(&[1u8; 32]);
    let untrusted_signing_key = SigningKey::from_bytes(&[2u8; 32]);
    let trusted_public_key = untrusted_signing_key.verifying_key().to_bytes();

    let wasm_bytes = sample_wat_bytes();
    let signed_payload = Ed25519ModuleVerifier::sign_payload(&signing_key, &wasm_bytes);

    let verifier = Ed25519ModuleVerifier::new(vec![trusted_public_key]);
    let err = verifier
        .verify(&signed_payload)
        .expect_err("module signed with untrusted key must be rejected");

    assert!(
        matches!(err, AdmissionError::InvalidSignature(_)),
        "Expected InvalidSignature, got {err:?}"
    );
}

#[test]
fn test_ed25519_module_verifier_expired_policy_rejected() {
    let signing_key = SigningKey::from_bytes(&[1u8; 32]);
    let public_key = signing_key.verifying_key().to_bytes();

    let wasm_bytes = sample_wat_bytes();
    let signed_payload = Ed25519ModuleVerifier::sign_payload(&signing_key, &wasm_bytes);

    let verifier = Ed25519ModuleVerifier::new(vec![public_key])
        .with_expiration(1000)
        .with_now(1000); // Evaluated at boundary expiration time

    let err = verifier
        .verify(&signed_payload)
        .expect_err("expired policy must be rejected");

    assert!(
        matches!(err, AdmissionError::VerificationFailed(_)),
        "Expected VerificationFailed, got {err:?}"
    );

    // Also test verify_at with None on an expiring policy fails closed
    let verifier_no_now = Ed25519ModuleVerifier::new(vec![public_key]).with_expiration(1000);
    let err_no_now = verifier_no_now
        .verify_at(&signed_payload, None)
        .expect_err("expiring policy without evaluation timestamp must fail closed");
    assert!(
        matches!(err_no_now, AdmissionError::VerificationFailed(_)),
        "Expected VerificationFailed, got {err_no_now:?}"
    );
}

#[test]
fn test_wasm_capabilities_signed_by_policy_integration() {
    let signing_key = SigningKey::from_bytes(&[1u8; 32]);
    let public_key = signing_key.verifying_key().to_bytes();

    let wasm_bytes = sample_wat_bytes();
    let signed_payload = Ed25519ModuleVerifier::sign_payload(&signing_key, &wasm_bytes);

    let caps = WasmCapabilities {
        module_policy: ModulePolicy::SignedBy(vec![public_key]),
        ..Default::default()
    };

    assert!(
        caps.verify_module_policy(&signed_payload).is_ok(),
        "Valid signed module must satisfy SignedBy module policy"
    );

    let unsigned_err = caps.verify_module_policy(&wasm_bytes).expect_err("Unsigned module must fail SignedBy policy");
    assert!(
        matches!(unsigned_err, SandboxError::InvalidModule(_)),
        "Expected InvalidModule error, got {unsigned_err:?}"
    );
}
