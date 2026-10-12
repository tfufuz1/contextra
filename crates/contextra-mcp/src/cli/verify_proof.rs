//! `verify-proof` CLI subcommand performing offline validation of DeletionProof structures.

use contextra_crypto::ed25519_proof::verify_deletion_proof_v3;
use contextra_types::ContextraError;
use std::path::Path;

/// Simple hex string decoder without third-party dependency.
pub fn hex_decode(s: &str) -> Result<Vec<u8>, ContextraError> {
    let s = s.trim();
    if s.len() % 2 != 0 {
        return Err(ContextraError::InvalidInput(
            "Hex string has odd length".to_string(),
        ));
    }
    (0..s.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&s[i..i + 2], 16)
                .map_err(|e| ContextraError::InvalidInput(format!("Invalid hex byte: {e}")))
        })
        .collect()
}

/// Executes offline verification of a DeletionProof JSON file or raw string.
pub fn execute_verify_proof(
    proof_file_or_json: &str,
    public_key_bytes: &[u8],
) -> Result<bool, ContextraError> {
    let raw_json = if Path::new(proof_file_or_json).exists() {
        std::fs::read_to_string(proof_file_or_json)?
    } else {
        proof_file_or_json.to_string()
    };

    let proof_val: serde_json::Value = serde_json::from_str(&raw_json).map_err(|e| {
        ContextraError::InvalidInput(format!("Failed to parse DeletionProof JSON: {e}"))
    })?;

    let sig_hex = proof_val
        .get("signature")
        .and_then(|s| s.as_str())
        .ok_or_else(|| {
            ContextraError::InvalidInput("Missing signature field in proof".to_string())
        })?;

    let sig_bytes = hex_decode(sig_hex)?;

    let sig_array: [u8; 64] = sig_bytes.try_into().map_err(|_| {
        ContextraError::InvalidInput("Ed25519 signature must be exactly 64 bytes".to_string())
    })?;

    let vk_array: [u8; 32] = public_key_bytes.try_into().map_err(|_| {
        ContextraError::InvalidInput("Ed25519 public key must be exactly 32 bytes".to_string())
    })?;

    let payload_bytes = if let Some(payload_str) = proof_val.get("payload").and_then(|p| p.as_str())
    {
        hex_decode(payload_str).unwrap_or_else(|_| payload_str.as_bytes().to_vec())
    } else {
        serde_json::to_vec(&proof_val).map_err(|e| {
            ContextraError::InvalidInput(format!("Failed to serialize proof payload: {e}"))
        })?
    };

    verify_deletion_proof_v3(&vk_array, &payload_bytes, &sig_array)
        .map(|_| true)
        .map_err(|e| ContextraError::InvalidInput(format!("Verification failed: {e}")))
}
