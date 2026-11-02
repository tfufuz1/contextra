//! F.10 Bericht Parser (kanonisches JSON).
//!
//! SPEC Anhang F.10:
//! - Kanonisierung: UTF-8 NFC, Objektschlüssel aufsteigend sortiert,
//!   keine Leerzeichen außerhalb von Strings, keine Gleitkommazahlen,
//!   Zeiten als Ganzzahl-Mikrosekunden UTC.
//! - Signatur: Ed25519 über `"ctx/report/v1" || kanonische_Bytes_ohne_signature`
//! - Felder: `signature` (Base64url), `pub_fingerprint` (SHA-256 von report_pub, 16 Byte hex)
//! - Pflichtinhalte laut SPEC Anhang D & §11.7:
//!   - Klassen G, K, P je mit reached/not_reached
//!   - key_coverage je Schicht
//!   - Keine Inhalte/Personenkennungen

use ed25519_dalek::{Signature, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::error::LabError;

/// Pflichtstatus einer Nachweisklasse (G, K, P).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ClassStatus {
    /// Erreicht
    Reached,
    /// Nicht erreicht
    NotReached,
}

/// Strukturierter Löschbericht nach SPEC F.10 und Anhang D.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ErasureReport {
    /// Eindeutige Vorgangskennung.
    pub erasure_id: String,
    /// Zeitpunkt der Anfrage in Mikrosekunden UTC.
    pub requested_at_micros: u64,
    /// Zeitpunkt der Sperre (Gate).
    pub gated_at_micros: u64,
    /// Zeitpunkt des Abschlusses.
    pub completed_at_micros: u64,
    /// Status Klasse G (Gate).
    pub class_g: ClassStatus,
    /// Status Klasse K (Key).
    pub class_k: ClassStatus,
    /// Status Klasse P (Purge).
    pub class_p: ClassStatus,
    /// Restrisiken nach Anhang D.
    pub residual_risks: Vec<String>,
    /// Fingerprint des Signaturschlüssels (16 Byte hex).
    pub pub_fingerprint: String,
    /// Signatur (Base64url).
    pub signature: String,
}

impl ErasureReport {
    /// Parst einen Bericht aus JSON.
    pub fn from_json_str(json: &str) -> Result<Self, LabError> {
        serde_json::from_str(json).map_err(|_| LabError::Corrupted("Ungültiges Bericht-JSON"))
    }

    /// Verifiziert die Ed25519-Signatur des Berichts über:
    /// `"ctx/report/v1" || kanonische_Bytes_ohne_signature`
    pub fn verify_signature(
        &self,
        raw_json_canonical_without_sig: &[u8],
        report_pub: &[u8; 32],
    ) -> Result<(), LabError> {
        let digest = Sha256::digest(report_pub);
        let fp_bytes = digest
            .get(0..16)
            .ok_or(LabError::Corrupted("Digest Fehler"))?;
        let expected_fp: String = fp_bytes.iter().map(|b| format!("{b:02x}")).collect();
        if self.pub_fingerprint != expected_fp {
            return Err(LabError::VerificationFailed(
                "Pubkey Fingerprint stimmt nicht überein",
            ));
        }

        let mut signed_data = Vec::with_capacity(14 + raw_json_canonical_without_sig.len());
        signed_data.extend_from_slice(b"ctx/report/v1");
        signed_data.extend_from_slice(raw_json_canonical_without_sig);

        let verifying_key = VerifyingKey::from_bytes(report_pub)
            .map_err(|_| LabError::Corrupted("Ungültiger Ed25519 öffentlicher Schlüssel"))?;

        let sig_bytes = if self.signature.len() == 128 {
            let mut out = [0u8; 64];
            for (i, byte_ref) in out.iter_mut().enumerate() {
                let start = i.saturating_mul(2);
                let end = start.saturating_add(2);
                let s = self.signature.get(start..end).unwrap_or("");
                *byte_ref =
                    u8::from_str_radix(s, 16).map_err(|_| LabError::Corrupted("Hex decode"))?;
            }
            out
        } else {
            return Err(LabError::Corrupted("Signaturformat nicht unterstützt"));
        };

        let sig = Signature::from_bytes(&sig_bytes);
        verifying_key
            .verify_strict(&signed_data, &sig)
            .map_err(|_| LabError::VerificationFailed("Bericht-Signatur ungültig"))?;

        Ok(())
    }
}
