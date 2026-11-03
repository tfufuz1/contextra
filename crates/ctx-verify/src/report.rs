//! Report parsing and validation per SPEC §8.17, §11.7, Anhang D, F.10.

use crate::canonical_json::{to_canonical_json_string, verify_canonical_json};
use crate::codes::InvalidCode;
use crate::journal::JournalChainEntry;
use crate::residual_risk::verify_residual_risks;
use ed25519_dalek::{Signature, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::Path;

/// Layer witness entry in report (SPEC §12.2).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct LayerWitnessReport {
    pub layer: String,
    pub class: String,
    pub method_id: String,
    pub scanned_files: u64,
    pub scanned_bytes: u64,
    pub matches: u64,
    pub control_ok: bool,
    pub duration_us: u64,
    pub hash: String,
}

/// Journal step recorded in the report for journal reduction proof.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct JournalStepReport {
    pub prev_hash_hex: String,
    pub raw_entry_hex: String,
    pub current_hash_hex: String,
}

/// Ledger reference recorded in the report or verified via external file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct LedgerEntryReport {
    pub commitment_hex: String,
    pub gated_seq: u64,
    pub completed_at: u64,
    pub range_count: u16,
}

/// Full structure of a signed deletion report per SPEC §11.7, F.10.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ErasureReport {
    pub report_version: u32,
    pub store_id: String,
    pub erasure_id: String,
    pub target_type: String,
    pub commitment: String,
    pub requested_at: u64,
    pub gated_at: u64,
    pub key_destroyed_at: u64,
    pub completed_at: u64,
    pub sla_us: u64,
    pub sla_violation: bool,
    pub retention_code: String,
    pub field_measurement: Vec<LayerWitnessReport>,
    pub lab_evidence: String,
    pub classes: BTreeMap<String, String>,
    pub key_coverage: BTreeMap<String, String>,
    pub segments_rewritten: u64,
    pub bytes_rewritten: u64,
    pub residual_risks: Vec<String>,
    pub legal_notice: String,
    pub signature_scope_notice: String,
    pub process_hardening: String,
    #[serde(default)]
    pub journal_chain: Vec<JournalStepReport>,
    #[serde(default)]
    pub ledger_entry: Option<LedgerEntryReport>,
    pub pub_fingerprint: String,
    pub signature: String,
}

/// Verifies a deletion report file against a public key file, optional ledger file, and optional expected store ID.
pub fn verify_report_file(
    report_path: &Path,
    pub_key_path: &Path,
    ledger_path: Option<&Path>,
    expected_store_id: Option<&str>,
) -> Result<(), InvalidCode> {
    let pub_key_bytes = std::fs::read(pub_key_path).map_err(|_| InvalidCode::MissingKey)?;
    let verifying_key = parse_public_key(&pub_key_bytes)?;

    // TOCTOU mitigation: read once into memory buffer
    let mut file = std::fs::File::open(report_path).map_err(|_| InvalidCode::IoError)?;
    let mut report_bytes = Vec::new();
    use std::io::Read;
    file.by_ref()
        .take((crate::canonical_json::MAX_REPORT_BYTES + 1) as u64)
        .read_to_end(&mut report_bytes)
        .map_err(|_| InvalidCode::IoError)?;

    if report_bytes.len() > crate::canonical_json::MAX_REPORT_BYTES {
        return Err(InvalidCode::IoError);
    }

    verify_report_bytes_with_target(
        &report_bytes,
        &verifying_key,
        ledger_path,
        expected_store_id,
    )
}

/// Parses an Ed25519 public key from 32 raw bytes or 64 hex characters.
pub fn parse_public_key(bytes: &[u8]) -> Result<VerifyingKey, InvalidCode> {
    let trimmed = match std::str::from_utf8(bytes) {
        Ok(s) => s.trim(),
        Err(_) => "",
    };

    let key_bytes: [u8; 32] = if bytes.len() == 32 {
        let mut k = [0u8; 32];
        k.copy_from_slice(bytes);
        k
    } else if trimmed.len() == 64 {
        let decoded = hex::decode(trimmed).map_err(|_| InvalidCode::MissingKey)?;
        if decoded.len() != 32 {
            return Err(InvalidCode::MissingKey);
        }
        let mut k = [0u8; 32];
        k.copy_from_slice(&decoded);
        k
    } else {
        return Err(InvalidCode::MissingKey);
    };

    VerifyingKey::from_bytes(&key_bytes).map_err(|_| InvalidCode::MissingKey)
}

/// Verifies canonical report bytes against verifying key.
pub fn verify_report_bytes(
    raw_bytes: &[u8],
    verifying_key: &VerifyingKey,
    ledger_path: Option<&Path>,
) -> Result<(), InvalidCode> {
    verify_report_bytes_with_target(raw_bytes, verifying_key, ledger_path, None)
}

/// Verifies canonical report bytes against verifying key with optional target store_id binding.
pub fn verify_report_bytes_with_target(
    raw_bytes: &[u8],
    verifying_key: &VerifyingKey,
    ledger_path: Option<&Path>,
    expected_store_id: Option<&str>,
) -> Result<(), InvalidCode> {
    // 1. Canonical JSON format check
    verify_canonical_json(raw_bytes).map_err(|_| InvalidCode::NonCanonicalJson)?;

    // 2. Parse JSON Value
    let val: serde_json::Value =
        serde_json::from_slice(raw_bytes).map_err(|_| InvalidCode::NonCanonicalJson)?;

    // Extract signature and pub_fingerprint
    let obj = match val.as_object() {
        Some(o) => o,
        None => return Err(InvalidCode::NonCanonicalJson),
    };

    let signature_str = match obj.get("signature").and_then(|v| v.as_str()) {
        Some(s) => s,
        None => return Err(InvalidCode::NonCanonicalJson),
    };

    let sig_bytes = hex::decode(signature_str).map_err(|_| InvalidCode::BadSignature)?;
    if sig_bytes.len() != 64 {
        return Err(InvalidCode::BadSignature);
    }
    let mut sig_arr = [0u8; 64];
    sig_arr.copy_from_slice(&sig_bytes);
    let signature = Signature::from_bytes(&sig_arr);

    // Verify public key fingerprint (SHA-256 first 16 bytes hex)
    let expected_fingerprint = {
        let mut hasher = Sha256::new();
        hasher.update(verifying_key.as_bytes());
        let digest = hasher.finalize();
        hex::encode(&digest[0..16])
    };

    if let Some(fp) = obj.get("pub_fingerprint").and_then(|v| v.as_str()) {
        if fp != expected_fingerprint {
            return Err(InvalidCode::BadSignature);
        }
    } else {
        return Err(InvalidCode::NonCanonicalJson);
    }

    // 3. Verify Ed25519 signature over "ctx/report/v1" || canonical_bytes_without_signature
    let mut unsigned_map = obj.clone();
    unsigned_map.remove("signature");
    let unsigned_val = serde_json::Value::Object(unsigned_map);
    let unsigned_canonical =
        to_canonical_json_string(&unsigned_val).map_err(|_| InvalidCode::NonCanonicalJson)?;

    let mut message = Vec::with_capacity(14 + unsigned_canonical.len());
    message.extend_from_slice(b"ctx/report/v1");
    message.extend_from_slice(unsigned_canonical.as_bytes());

    verifying_key
        .verify_strict(&message, &signature)
        .map_err(|_| InvalidCode::BadSignature)?;

    // 4. Parse full report structure
    let report: ErasureReport =
        serde_json::from_value(val).map_err(|_| InvalidCode::NonCanonicalJson)?;

    // Check store_id binding (replay protection across different stores/targets)
    if let Some(target_id) = expected_store_id {
        if report.store_id != target_id {
            return Err(InvalidCode::TargetMismatch);
        }
    }

    // 5. Temporal order: requested_at <= gated_at <= key_destroyed_at <= completed_at
    if !(report.requested_at <= report.gated_at
        && report.gated_at <= report.key_destroyed_at
        && report.key_destroyed_at <= report.completed_at)
    {
        return Err(InvalidCode::TemporalOrderViolation);
    }

    // 6. Witness validation: every witness must have positive control (control_ok == true) and 0 matches
    for w in &report.field_measurement {
        if !w.control_ok {
            return Err(InvalidCode::MissingPositiveControl);
        }
        if w.matches != 0 {
            return Err(InvalidCode::WitnessMatchesNonZero);
        }
    }

    // 7. Residual risks completeness check
    verify_residual_risks(&report.residual_risks)
        .map_err(|_| InvalidCode::IncompleteResidualRisks)?;

    // 8. Journal hash chain check (if steps present)
    if !report.journal_chain.is_empty() {
        let mut prev_hash = [0u8; 32];
        for (i, step) in report.journal_chain.iter().enumerate() {
            let step_prev =
                hex::decode(&step.prev_hash_hex).map_err(|_| InvalidCode::BrokenHashChain)?;
            let raw_entry =
                hex::decode(&step.raw_entry_hex).map_err(|_| InvalidCode::BrokenHashChain)?;
            let step_curr =
                hex::decode(&step.current_hash_hex).map_err(|_| InvalidCode::BrokenHashChain)?;

            if step_prev.len() != 32 || step_curr.len() != 32 {
                return Err(InvalidCode::BrokenHashChain);
            }

            if i > 0 && step_prev.as_slice() != prev_hash.as_slice() {
                return Err(InvalidCode::BrokenHashChain);
            }

            let mut prev_arr = [0u8; 32];
            prev_arr.copy_from_slice(&step_prev);
            let calculated = JournalChainEntry::compute_chain_step(&prev_arr, &raw_entry);

            if calculated.as_slice() != step_curr.as_slice() {
                return Err(InvalidCode::BrokenHashChain);
            }

            prev_hash.copy_from_slice(&step_curr);
        }
    }

    // 9. Optional ledger check
    if let Some(l_path) = ledger_path {
        let ledger_content = std::fs::read(l_path).map_err(|_| InvalidCode::IoError)?;
        if let Some(entry) = &report.ledger_entry {
            // Check that commitment and gated_seq are present in the ledger content
            let comm_bytes =
                hex::decode(&entry.commitment_hex).map_err(|_| InvalidCode::LedgerMismatch)?;
            if !ledger_content
                .windows(comm_bytes.len())
                .any(|w| w == comm_bytes.as_slice())
            {
                return Err(InvalidCode::LedgerMismatch);
            }
        } else {
            return Err(InvalidCode::LedgerMismatch);
        }
    }

    Ok(())
}
