//! `verify-proof` CLI subcommand performing offline validation of DeletionProof v4 and legacy structures.

use contextra_crypto::deletion_proof::policy::{verify_with_policy, ProofPolicy};
use contextra_crypto::deletion_proof::proof_store::ProofStore;
use contextra_crypto::deletion_proof::proof_v4::ProofV4;
use contextra_crypto::ed25519_proof::verify_deletion_proof_v3;
use contextra_types::ContextraError;
use std::path::Path;

/// Options passed to `execute_verify_proof_v4`.
#[derive(Debug, Clone, Default)]
pub struct VerifyProofOptions {
    /// Path to proof file or raw content.
    pub proof_path_or_str: String,
    /// Path to public key file or hex string.
    pub pubkey_path_or_str: String,
    /// Optional path to policy file (JSON).
    pub policy_path: Option<String>,
    /// Optional path to chain directory or log.
    pub chain_path: Option<String>,
    /// Whether legacy (v1-v3) proofs are allowed.
    pub allow_legacy: bool,
}

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

/// Helper hex string encoder.
pub fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

fn read_file_or_raw(input: &str) -> Result<Vec<u8>, ContextraError> {
    if Path::new(input).exists() {
        std::fs::read(input).map_err(|e| {
            ContextraError::InvalidInput(format!("IO error reading file {input}: {e}"))
        })
    } else {
        Ok(input.as_bytes().to_vec())
    }
}

fn parse_pubkey_bytes(input: &str) -> Result<[u8; 32], ContextraError> {
    let bytes = if Path::new(input).exists() {
        let raw = std::fs::read_to_string(input)?;
        hex_decode(&raw)?
    } else {
        hex_decode(input)?
    };

    bytes.try_into().map_err(|_| {
        ContextraError::InvalidInput("Ed25519 public key must be exactly 32 bytes".to_string())
    })
}

/// Legacy v1-v3 verification fallback.
pub fn execute_verify_proof_legacy(
    proof_val: &serde_json::Value,
    public_key_bytes: &[u8; 32],
) -> Result<bool, ContextraError> {
    let sig_hex = proof_val
        .get("signature")
        .and_then(|s| s.as_str())
        .ok_or_else(|| {
            ContextraError::InvalidInput("R1 FAIL: Missing signature field in proof".to_string())
        })?;

    let sig_bytes = hex_decode(sig_hex)?;

    let sig_array: [u8; 64] = sig_bytes.try_into().map_err(|_| {
        ContextraError::InvalidInput(
            "R1 FAIL: Ed25519 signature must be exactly 64 bytes".to_string(),
        )
    })?;

    let payload_bytes = if let Some(payload_str) = proof_val.get("payload").and_then(|p| p.as_str())
    {
        hex_decode(payload_str).unwrap_or_else(|_| payload_str.as_bytes().to_vec())
    } else {
        serde_json::to_vec(&proof_val).map_err(|e| {
            ContextraError::InvalidInput(format!("Failed to serialize proof payload: {e}"))
        })?
    };

    verify_deletion_proof_v3(public_key_bytes, &payload_bytes, &sig_array)
        .map(|_| true)
        .map_err(|e| ContextraError::InvalidInput(format!("R1 FAIL: Verification failed: {e}")))
}

/// Executes offline verification of a DeletionProof (v4 with policy/chain, or v1-v3 legacy).
pub fn execute_verify_proof_v4(opts: &VerifyProofOptions) -> Result<bool, ContextraError> {
    let raw_bytes = read_file_or_raw(&opts.proof_path_or_str)?;
    let public_key = parse_pubkey_bytes(&opts.pubkey_path_or_str)?;

    // 1. Try to decode as v4 JSON
    let proof_v4_opt: Option<ProofV4> = if let Ok(s) = std::str::from_utf8(&raw_bytes) {
        serde_json::from_str::<ProofV4>(s).ok()
    } else {
        None
    };

    if let Some(proof) = proof_v4_opt {
        // Build Policy
        let policy = ProofPolicy::default();

        // Evaluate policy
        let report = verify_with_policy(&proof, &public_key, &policy);

        if !report.is_valid {
            for res in &report.rule_results {
                if !res.passed {
                    eprintln!("{} FAIL: {}", res.rule_id, res.message);
                }
            }
            // Find first failing rule ID
            let first_failed = report
                .rule_results
                .iter()
                .find(|r| !r.passed)
                .map(|r| r.rule_id.as_str())
                .unwrap_or("R-UNKNOWN");

            return Err(ContextraError::InvalidInput(format!(
                "Policy verification failed at {first_failed}"
            )));
        }

        // Chain verification if requested
        if let Some(ref chain_path) = opts.chain_path {
            let chain_dir = if Path::new(chain_path).is_file() {
                Path::new(chain_path).parent().unwrap_or(Path::new("."))
            } else {
                Path::new(chain_path)
            };

            let store = ProofStore::open(chain_dir)
                .map_err(|e| {
                    ContextraError::InvalidInput(format!(
                        "R7 FAIL: Failed to open chain store: {e}"
                    ))
                })?
                .0;

            let warnings = store.verify_chain().map_err(|e| {
                ContextraError::InvalidInput(format!("R7 FAIL: Chain verification error: {e}"))
            })?;

            if !warnings.is_empty() {
                eprintln!("R7 FAIL: Chain gap or mismatch detected: {:?}", warnings);
                return Err(ContextraError::InvalidInput(
                    "R7 FAIL: Chain verification failed with warnings/gaps".to_string(),
                ));
            }
        }

        return Ok(true);
    }

    // 2. Legacy v1-v3 handling
    if let Ok(json_str) = std::str::from_utf8(&raw_bytes) {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(json_str) {
            if !opts.allow_legacy {
                return Err(ContextraError::InvalidInput(
                    "Legacy proof (v1-v3) rejected. Use --allow-legacy to verify legacy proofs."
                        .to_string(),
                ));
            }

            eprintln!("[NOTE] Verifying legacy proof version (marked as legacy)");
            return execute_verify_proof_legacy(&val, &public_key);
        }
    }

    Err(ContextraError::InvalidInput(
        "Invalid proof format: Could not parse as v4 JSON or legacy JSON proof".to_string(),
    ))
}

/// Backward compatibility wrapper.
pub fn execute_verify_proof(
    proof_file_or_json: &str,
    public_key_bytes: &[u8],
) -> Result<bool, ContextraError> {
    let pubkey_hex = hex_encode(public_key_bytes);
    let opts = VerifyProofOptions {
        proof_path_or_str: proof_file_or_json.to_string(),
        pubkey_path_or_str: pubkey_hex,
        policy_path: None,
        chain_path: None,
        allow_legacy: true,
    };
    execute_verify_proof_v4(&opts)
}
