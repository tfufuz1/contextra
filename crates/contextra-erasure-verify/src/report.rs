//! Unabhaengiger Berichtspruefer fuer signierte Loeschberichte.

use crate::canonical_json::{to_canonical_json_string, verify_canonical_json};
use crate::codes::InvalidCode;
use crate::residual_risk::verify_residual_risks;
use ed25519_dalek::{Signature, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::Path;

/// Zeugeneintrag im Loeschbericht (SPEC §12.2).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

/// Struktur eines signierten Loeschberichts nach SPEC §11.7, F.10.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
    pub pub_fingerprint: String,
    pub signature: String,
}

/// Parst einen Ed25519 Public Key aus 32 rohen Bytes oder 64 Hex-Zeichen.
pub fn parse_public_key(bytes: &[u8]) -> Result<VerifyingKey, InvalidCode> {
    let key_bytes: [u8; 32] = if bytes.len() == 32 {
        let mut k = [0u8; 32];
        k.copy_from_slice(bytes);
        k
    } else {
        let s = std::str::from_utf8(bytes).map_err(|_| InvalidCode::MissingKey)?;
        let trimmed = s.trim();
        if trimmed.len() != 64 {
            return Err(InvalidCode::MissingKey);
        }
        let decoded = hex_decode(trimmed)?;
        let mut k = [0u8; 32];
        k.copy_from_slice(&decoded);
        k
    };

    VerifyingKey::from_bytes(&key_bytes).map_err(|_| InvalidCode::MissingKey)
}

/// Parst einen Hex-String ohne externe Hex-Crate.
pub fn hex_decode(s: &str) -> Result<Vec<u8>, InvalidCode> {
    if s.len() % 2 != 0 {
        return Err(InvalidCode::BadSignature);
    }
    let mut bytes = Vec::with_capacity(s.len() / 2);
    for chunk in s.as_bytes().chunks_exact(2) {
        let h = match chunk.first() {
            Some(b) => parse_hex_nibble(*b)?,
            None => return Err(InvalidCode::BadSignature),
        };
        let l = match chunk.get(1) {
            Some(b) => parse_hex_nibble(*b)?,
            None => return Err(InvalidCode::BadSignature),
        };
        bytes.push((h << 4) | l);
    }
    Ok(bytes)
}

fn parse_hex_nibble(b: u8) -> Result<u8, InvalidCode> {
    match b {
        b'0'..=b'9' => Ok(b - b'0'),
        b'a'..=b'f' => Ok(b - b'a' + 10),
        b'A'..=b'F' => Ok(b - b'A' + 10),
        _ => Err(InvalidCode::BadSignature),
    }
}

pub fn hex_encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(match b >> 4 {
            0..=9 => (b'0' + (b >> 4)) as char,
            10..=15 => (b'a' + (b >> 4) - 10) as char,
            _ => '0',
        });
        out.push(match b & 0x0f {
            0..=9 => (b'0' + (b & 0x0f)) as char,
            10..=15 => (b'a' + (b & 0x0f) - 10) as char,
            _ => '0',
        });
    }
    out
}

/// Verifiziert rohe Berichtsbytes gegen einen uebergebenen VerifyingKey.
pub fn verify_report_bytes(
    raw_bytes: &[u8],
    verifying_key: &VerifyingKey,
) -> Result<(), InvalidCode> {
    // 1. Kanonisches JSON pruefen
    verify_canonical_json(raw_bytes)?;

    // 2. Parsen
    let val: serde_json::Value =
        serde_json::from_slice(raw_bytes).map_err(|_| InvalidCode::NonCanonicalJson)?;

    let obj = match val.as_object() {
        Some(o) => o,
        None => return Err(InvalidCode::NonCanonicalJson),
    };

    let signature_str = match obj.get("signature").and_then(|v| v.as_str()) {
        Some(s) => s,
        None => return Err(InvalidCode::NonCanonicalJson),
    };

    let sig_bytes = hex_decode(signature_str).map_err(|_| InvalidCode::BadSignature)?;
    if sig_bytes.len() != 64 {
        return Err(InvalidCode::BadSignature);
    }
    let mut sig_arr = [0u8; 64];
    sig_arr.copy_from_slice(&sig_bytes);
    let signature = Signature::from_bytes(&sig_arr);

    // Fingerprint pruefen (SHA-256 erste 16 Bytes)
    let expected_fingerprint = {
        let mut hasher = Sha256::new();
        hasher.update(verifying_key.as_bytes());
        let digest = hasher.finalize();
        let slice = match digest.get(0..16) {
            Some(s) => s,
            None => return Err(InvalidCode::BadSignature),
        };
        hex_encode(slice)
    };

    if let Some(fp) = obj.get("pub_fingerprint").and_then(|v| v.as_str()) {
        if fp != expected_fingerprint {
            return Err(InvalidCode::BadSignature);
        }
    } else {
        return Err(InvalidCode::NonCanonicalJson);
    }

    // 3. Ed25519 verify_strict ueber "ctx/report/v1" || canonical_bytes_without_signature
    let mut unsigned_map = obj.clone();
    unsigned_map.remove("signature");
    let unsigned_val = serde_json::Value::Object(unsigned_map);
    let unsigned_canonical = to_canonical_json_string(&unsigned_val)?;

    let mut message = Vec::with_capacity(14 + unsigned_canonical.len());
    message.extend_from_slice(b"ctx/report/v1");
    message.extend_from_slice(unsigned_canonical.as_bytes());

    verifying_key
        .verify_strict(&message, &signature)
        .map_err(|_| InvalidCode::BadSignature)?;

    // 4. Struktur-Validierung
    let report: ErasureReport =
        serde_json::from_value(val).map_err(|_| InvalidCode::NonCanonicalJson)?;

    // 5. Zeitstempel-Monotonie: requested <= gated <= destroyed <= completed
    if !(report.requested_at <= report.gated_at
        && report.gated_at <= report.key_destroyed_at
        && report.key_destroyed_at <= report.completed_at)
    {
        return Err(InvalidCode::TemporalOrderViolation);
    }

    // 6. Zeugen-Pruefung: JEDER Zeuge MUSS positive Kontrolle bestanden haben (control_ok == true)
    // und genau 0 Treffer aufweisen (matches == 0)
    if report.field_measurement.is_empty() {
        return Err(InvalidCode::MissingPositiveControl);
    }

    for w in &report.field_measurement {
        if !w.control_ok {
            return Err(InvalidCode::MissingPositiveControl);
        }
        if w.matches != 0 {
            return Err(InvalidCode::WitnessMatchesNonZero);
        }
    }

    // 7. Restrisiko-Vollstaendigkeit
    verify_residual_risks(&report.residual_risks)?;

    Ok(())
}

/// Liest und verifiziert eine Berichtsdatei.
pub fn verify_report_file(report_path: &Path, pub_key_path: &Path) -> Result<(), InvalidCode> {
    let pub_key_bytes = match std::fs::read(pub_key_path) {
        Ok(b) => b,
        Err(_) => return Err(InvalidCode::MissingKey),
    };
    let verifying_key = parse_public_key(&pub_key_bytes)?;

    let report_bytes = match std::fs::read(report_path) {
        Ok(b) => b,
        Err(_) => return Err(InvalidCode::IoError),
    };

    verify_report_bytes(&report_bytes, &verifying_key)
}
