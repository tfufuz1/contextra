//! Kryptografische Signierung des Berichts mittels Ed25519 (SPEC §11.7, D-13, ER-27).
//!
//! Signatur-Domain: `"ctx/report/v1" ‖ kanonische Bytes ohne signature`
//! Felder: `signature` (Base64url), `pub_fingerprint`

use crate::error::ErasureError;
use serde::{Deserialize, Serialize};

/// Signierter Löschbericht (ER-27).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedReport {
    /// Kanonische JSON-Bytes des Berichts (ohne Signatur-Feld).
    pub report_bytes: Vec<u8>,
    /// Ed25519-Signatur über `"ctx/report/v1" ‖ report_bytes` (Base64url).
    pub signature: String,
    /// Fingerabdruck des öffentlichen Schlüssels (hex).
    pub pub_fingerprint: String,
}

impl SignedReport {
    /// Erstellt einen signierten Bericht (stub – produktiv: echter Ed25519-Key aus KeyHierarchy).
    ///
    /// In der vollständigen Implementierung (WP-4.3) wird der Schlüssel aus
    /// `crypto::KeyHierarchy::report_signing_key()` bezogen.
    pub fn sign_stub(report_bytes: Vec<u8>) -> Self {
        // Stub: keine echte Signatur (WP-4.3)
        let domain = b"ctx/report/v1";
        let mut to_sign = Vec::with_capacity(domain.len() + report_bytes.len());
        to_sign.extend_from_slice(domain);
        to_sign.extend_from_slice(&report_bytes);

        // Deterministischer Platzhalter (kein echter Ed25519, WP-4.3)
        let hash = blake3::hash(&to_sign);
        let sig_hex = hash
            .as_bytes()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();

        Self {
            report_bytes,
            signature: sig_hex.clone(),
            pub_fingerprint: sig_hex[..16].to_string(),
        }
    }

    /// Gibt den kanonischen JSON-Bericht als String zurück.
    pub fn report_json(&self) -> Result<String, ErasureError> {
        String::from_utf8(self.report_bytes.clone())
            .map_err(|e| ErasureError::Json(e.to_string()))
    }
}
