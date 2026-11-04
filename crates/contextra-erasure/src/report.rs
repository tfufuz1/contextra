//! Berichts-Modul (SPEC §11.7).
//!
//! Zusammenstellung, Kanonisierung und Signierung des Löschberichts.

pub mod build;
pub mod canonical;
pub mod risk;
pub mod signing;

pub use build::{
    build_report, canonicalize_report, FieldMeasurementSection, LayerKeyCoverage,
    ProcessHardeningStatus, ReachStatus, WitnessEntry,
};
pub use canonical::{canonicalize_value, to_canonical_json, to_canonical_json_string};
pub use risk::{evaluate_residual_risks, ResidualRiskEntry};
pub use signing::ReportSigner;

use crate::error::ErasureError;
use ed25519_dalek::{Signer, SigningKey};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Löschbericht vor Signierung.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErasureReport {
    /// Schema-Version (z. B. "ctx/report/v1")
    pub schema_version: String,
    /// Zeitstempel in UTC-Mikrosekunden
    pub timestamp_micros: u64,
    /// Sortierte Felder (BTreeMap für kanonische Ordnung)
    pub fields: BTreeMap<String, String>,
}

impl ErasureReport {
    /// Erstellt einen neuen Bericht.
    pub fn new(schema_version: &str, timestamp_micros: u64) -> Self {
        Self {
            schema_version: schema_version.to_string(),
            timestamp_micros,
            fields: BTreeMap::new(),
        }
    }

    /// Setzt ein Attribut im Bericht.
    pub fn set_field(&mut self, key: &str, val: &str) {
        self.fields.insert(key.to_string(), val.to_string());
    }
}

/// Kanonisiert den Bericht nach SPEC F.10:
/// Sortierte Schlüssel, keine unnötigen Whitespaces, UTF-8.
pub fn canonicalize_json(report: &ErasureReport) -> Result<String, ErasureError> {
    serde_json::to_string(report).map_err(|e| ErasureError::Internal(e.to_string()))
}

/// Signierter Löschbericht mit Base64url-Signatur und Prüffingerabdruck (ER-27).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedReport {
    /// Kanonischer JSON-Inhalt
    pub canonical_body: String,
    /// Signatur als Hex- oder Base64url-String
    pub signature_base64: String,
    /// Öffentlicher Schlüssel Fingerprint (Hex)
    pub pub_fingerprint: String,
}

impl SignedReport {
    /// Signiert den kanonischen Inhalt mit dem Ed25519-Schlüssel.
    pub fn sign(report: ErasureReport, key: &SigningKey) -> Result<Self, ErasureError> {
        let canonical = canonicalize_json(&report)?;
        let mut msg = Vec::new();
        msg.extend_from_slice(b"ctx/report/v1");
        msg.extend_from_slice(canonical.as_bytes());

        let sig = key.sign(&msg);
        let sig_bytes = sig.to_bytes();
        let mut sig_hex = String::new();
        for b in &sig_bytes {
            sig_hex.push_str(&format!("{:02x}", b));
        }

        let pub_key = key.verifying_key();
        let pub_hash = blake3::hash(pub_key.as_bytes());

        Ok(Self {
            canonical_body: canonical,
            signature_base64: sig_hex,
            pub_fingerprint: pub_hash.to_hex().to_string(),
        })
    }

    /// Liefert den Signatur-String.
    pub fn signature_base64(&self) -> &str {
        &self.signature_base64
    }
}
