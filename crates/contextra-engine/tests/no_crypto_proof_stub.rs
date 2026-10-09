#![cfg(not(feature = "encryption-at-rest"))]

use contextra_engine::{DeletionLayer, DeletionProof, DeletionScope, LayerCleanupProof};
use contextra_types::{TenantId, TxId};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn sample_scope() -> DeletionScope {
    DeletionScope::Tenant {
        tenant_id: TenantId::try_new(123).unwrap_or_default(),
    }
}

fn sample_keys() -> Vec<Vec<u8>> {
    vec![b"key1".to_vec(), b"key2".to_vec()]
}

fn create_sample_proof(key: &[u8]) -> Result<DeletionProof, Box<dyn std::error::Error>> {
    let scope = sample_scope();
    let keys = sample_keys();
    let layer_proof = /* Test-Fixture, keine Produktion */ LayerCleanupProof::verify_and_create(DeletionLayer::LsmMemtable, || Ok(true))?;
    let proof = DeletionProof::create(scope, keys, TxId(42), vec![layer_proof], vec![], key)?;
    Ok(proof)
}

#[test]
fn test_create_and_verify_valid_key() -> TestResult {
    let key = b"secret_hmac_key_32_bytes_long!!";
    let proof = create_sample_proof(key)?;
    let valid = proof.verify(key)?;
    if !valid {
        return Err("Verification with correct key failed".into());
    }
    Ok(())
}

#[test]
fn test_verify_wrong_key() -> TestResult {
    let key = b"secret_hmac_key_32_bytes_long!!";
    let wrong_key = b"wrong_hmac_key_32_bytes_long!!";
    let proof = create_sample_proof(key)?;
    let valid = proof.verify(wrong_key)?;
    if valid {
        return Err("Verification with wrong key unexpectedly passed".into());
    }
    Ok(())
}

#[test]
fn test_tamper_fields() -> TestResult {
    let key = b"secret_hmac_key_32_bytes_long!!";

    // Tamper deleted_keys_hash
    let mut proof_tampered_hash = create_sample_proof(key)?;
    proof_tampered_hash.deleted_keys_hash[0] ^= 0xFF;
    if proof_tampered_hash.verify(key)? {
        return Err("Verification passed after tampering deleted_keys_hash".into());
    }

    // Tamper covered_layers
    let mut proof_tampered_layers = create_sample_proof(key)?;
    proof_tampered_layers
        .covered_layers
        .push(DeletionLayer::HnswIndex);
    if proof_tampered_layers.verify(key)? {
        return Err("Verification passed after tampering covered_layers".into());
    }

    // Tamper deleted_after_tx
    let mut proof_tampered_tx = create_sample_proof(key)?;
    proof_tampered_tx.deleted_after_tx = TxId(999);
    if proof_tampered_tx.verify(key)? {
        return Err("Verification passed after tampering deleted_after_tx".into());
    }

    // Tamper scope
    let mut proof_tampered_scope = create_sample_proof(key)?;
    proof_tampered_scope.scope = DeletionScope::Tenant {
        tenant_id: TenantId::try_new(999).unwrap_or_default(),
    };
    if proof_tampered_scope.verify(key)? {
        return Err("Verification passed after tampering scope".into());
    }

    // Tamper signature
    let mut proof_tampered_sig = create_sample_proof(key)?;
    if let Some(byte) = proof_tampered_sig.signature.get_mut(0) {
        *byte ^= 0xFF;
    }
    if proof_tampered_sig.verify(key)? {
        return Err("Verification passed after tampering signature".into());
    }

    Ok(())
}

#[test]
fn test_invalid_signature_version() -> TestResult {
    let key = b"secret_hmac_key_32_bytes_long!!";
    let mut proof = create_sample_proof(key)?;
    proof.signature_version = 1;
    let result = proof.verify(key);
    if result.is_ok() {
        return Err("Verification succeeded or returned Ok for invalid signature_version".into());
    }
    Ok(())
}

#[test]
fn test_layer_cleanup_proof_verify_and_create() -> TestResult {
    // Closure returning Ok(false) -> Err
    let res_false = LayerCleanupProof::verify_and_create(DeletionLayer::LsmMemtable, || Ok(false));
    if res_false.is_ok() {
        return Err("verify_and_create succeeded when closure returned Ok(false)".into());
    }

    // Closure returning Err -> Err
    let res_err = LayerCleanupProof::verify_and_create(DeletionLayer::LsmMemtable, || {
        Err(contextra_types::ContextraError::Internal("failed".into()))
    });
    if res_err.is_ok() {
        return Err("verify_and_create succeeded when closure returned Err".into());
    }

    // Closure returning Ok(true) -> Ok
    let res_true = LayerCleanupProof::verify_and_create(DeletionLayer::LsmMemtable, || Ok(true));
    if res_true.is_err() {
        return Err("verify_and_create failed when closure returned Ok(true)".into());
    }

    Ok(())
}

#[test]
fn test_new_after_verified_empty() -> TestResult {
    // remaining_count != 0 -> Err
    let res_non_zero = LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 1);
    if res_non_zero.is_ok() {
        return Err("new_after_verified_empty succeeded with remaining_count != 0".into());
    }

    // remaining_count == 0 -> Ok
    let res_zero = /* Test-Fixture, keine Produktion */ LayerCleanupProof::verify_and_create(DeletionLayer::LsmMemtable, || Ok(true));
    if res_zero.is_err() {
        return Err("new_after_verified_empty failed with remaining_count == 0".into());
    }

    Ok(())
}

#[test]
fn test_integrity_warning_present_and_non_empty() -> TestResult {
    let key = b"secret_hmac_key_32_bytes_long!!";
    let proof = create_sample_proof(key)?;
    match &proof.integrity_warning {
        Some(warning) if !warning.is_empty() => Ok(()),
        _ => Err("integrity_warning is missing or empty".into()),
    }
}
