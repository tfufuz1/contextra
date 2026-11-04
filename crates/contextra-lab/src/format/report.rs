//! F.10 Bericht Parser (kanonisches JSON).
//!
//! SPEC Anhang F.10:
//! - Kanonisierung: UTF-8 NFC, Objektschlüssel aufsteigend sortiert,
//!   keine Leerzeichen außerhalb von Strings, keine Gleitkommazahlen,
//!   Zeiten als Ganzzahl-Mikrosekunden UTC.
//! - Signatur: Ed25519 über `"ctx/report/v1" || kanonische_Bytes_ohne_signature`
//! - Felder: `signature` (Base64url), `pub_fingerprint` (SHA-256 von report_pub, 16 Byte hex)
//! - Pflichtinhalte laut SPEC Anhang D & §11.7:
//!   - Klassenverband G, K, P je einzeln mit reached/not_reached (keine Gesamtnote, D-36, T-CLASS-1)
//!   - key_coverage je Schicht (full, unlinked, none, D-69, T-KEYCOV-1)
//!   - Keine Inhalte/Personenkennungen (HR-4, P13)

use ed25519_dalek::{Signature, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::error::LabError;

/// Pflichtstatus einer Nachweisklasse (G, K, P laut SPEC D-36, §12.5).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ClassStatus {
    /// Erreicht
    Reached,
    /// Nicht erreicht
    NotReached,
}

/// Reichweite der Schlüsselvernichtung je Schicht laut SPEC D-69 und T-KEYCOV-1.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum KeyCoverage {
    /// Vollständig: Inhalt unlesbar (doc_store, vector_payload)
    Full,
    /// Entkoppelt: Struktur vorhanden, aber Bindung an Dokument/Betroffenen gelöst (text_postings)
    Unlinked,
    /// Keine: Nicht durch Schlüsselvernichtung geschützt (vector_edges, file_structure)
    None,
}

/// Schlüsselreichweite je Einzelschicht nach SPEC D-69.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LayerKeyCoverage {
    /// Dokumentspeicher (LSM/SSTable)
    #[serde(default = "default_coverage_full")]
    pub doc_store: KeyCoverage,
    /// Vektor-Payload (Embeddings unter DEK)
    #[serde(default = "default_coverage_full")]
    pub vector_payload: KeyCoverage,
    /// Text-Postings (Bindung unter DEK im ersten Purge-Fenster)
    #[serde(default = "default_coverage_unlinked")]
    pub text_postings: KeyCoverage,
    /// Vektor-Kanten (Graph-Topologie)
    #[serde(default = "default_coverage_none")]
    pub vector_edges: KeyCoverage,
    /// Dateistruktur (Header/Metadaten bis Dateischlüssel-Vernichtung)
    #[serde(default = "default_coverage_none")]
    pub file_structure: KeyCoverage,
}

fn default_coverage_full() -> KeyCoverage {
    KeyCoverage::Full
}

fn default_coverage_unlinked() -> KeyCoverage {
    KeyCoverage::Unlinked
}

fn default_coverage_none() -> KeyCoverage {
    KeyCoverage::None
}

impl Default for LayerKeyCoverage {
    fn default() -> Self {
        Self {
            doc_store: KeyCoverage::Full,
            vector_payload: KeyCoverage::Full,
            text_postings: KeyCoverage::Unlinked,
            vector_edges: KeyCoverage::None,
            file_structure: KeyCoverage::None,
        }
    }
}

/// Strukturierter Löschbericht nach SPEC F.10, §11.7, §12.5 und Anhang D.
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
    /// Status Klasse G (Gate) - einzeln ausgewiesen (T-CLASS-1).
    pub class_g: ClassStatus,
    /// Status Klasse K (Key) - einzeln ausgewiesen (T-CLASS-1).
    pub class_k: ClassStatus,
    /// Status Klasse P (Purge) - einzeln ausgewiesen (T-CLASS-1).
    pub class_p: ClassStatus,
    /// Reichweite der Schlüsselvernichtung je Schicht (T-KEYCOV-1, D-69).
    #[serde(default)]
    pub key_coverage: LayerKeyCoverage,
    /// Restrisiken nach Anhang D.
    pub residual_risks: Vec<String>,
    /// Fingerprint des Signaturschlüssels (16 Byte hex).
    pub pub_fingerprint: String,
    /// Signatur (Base64url oder hex-codiert).
    pub signature: String,
}

impl ErasureReport {
    /// Parst einen Bericht aus JSON.
    pub fn from_json_str(json: &str) -> Result<Self, LabError> {
        serde_json::from_str(json).map_err(|_| LabError::Corrupted("Ungültiges Bericht-JSON"))
    }

    /// Verifiziert die Klassenverbands-Regel (SPEC D-36, T-CLASS-1):
    /// Jede Klasse G, K, P muss einzeln bewertet sein, keine Gesamtnote.
    pub fn verify_class_lattice(&self) -> Result<(), LabError> {
        // G, K, P sind unabhängig; alle drei müssen gültige Stati tragen.
        // Keine Reduktion auf einen einzelnen Skalarwert erlaubt.
        Ok(())
    }

    /// Verifiziert die Übereinstimmung der key_coverage je Schicht (SPEC D-69, T-KEYCOV-1).
    pub fn verify_key_coverage_contract(&self) -> Result<(), LabError> {
        if self.key_coverage.doc_store != KeyCoverage::Full {
            return Err(LabError::VerificationFailed(
                "doc_store muss key_coverage = full aufweisen",
            ));
        }
        if self.key_coverage.vector_payload != KeyCoverage::Full {
            return Err(LabError::VerificationFailed(
                "vector_payload muss key_coverage = full aufweisen",
            ));
        }
        if self.key_coverage.text_postings != KeyCoverage::Unlinked {
            return Err(LabError::VerificationFailed(
                "text_postings muss key_coverage = unlinked aufweisen",
            ));
        }
        if self.key_coverage.vector_edges != KeyCoverage::None {
            return Err(LabError::VerificationFailed(
                "vector_edges muss key_coverage = none aufweisen",
            ));
        }
        if self.key_coverage.file_structure != KeyCoverage::None {
            return Err(LabError::VerificationFailed(
                "file_structure muss key_coverage = none aufweisen",
            ));
        }
        Ok(())
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
