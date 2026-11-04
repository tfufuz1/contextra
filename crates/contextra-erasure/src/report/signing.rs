//! Kryptografische Signierung des Berichts mittels Ed25519 (SPEC §11.7, D-13, ER-27).
//!
//! Signatur-Domain: `"ctx/report/v1" ‖ kanonische Bytes ohne signature`
//! Felder: `signature` (Base64url), `pub_fingerprint`

use super::build::{canonicalize_report, ErasureReport};
use super::SignedReport;
use crate::error::ErasureError;
use contextra_ports::vfs::Vfs;
use ed25519_dalek::{Signer, SigningKey};
use sha2::Digest;
use std::path::Path;

/// Hilfsfunktion für Base64url-Kodierung ohne Padding (F.10, RFC 4648 §5).
fn base64url_encode(data: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut out = String::new();
    let mut i = 0;
    while i < data.len() {
        let b0 = match data.get(i) {
            Some(&b) => b,
            None => break,
        };
        let b1 = data.get(i + 1).copied();
        let b2 = data.get(i + 2).copied();

        let n = ((b0 as u32) << 16) | ((b1.unwrap_or(0) as u32) << 8) | (b2.unwrap_or(0) as u32);
        let c0 = TABLE
            .get(((n >> 18) & 63) as usize)
            .copied()
            .unwrap_or(b'A');
        let c1 = TABLE
            .get(((n >> 12) & 63) as usize)
            .copied()
            .unwrap_or(b'A');
        out.push(c0 as char);
        out.push(c1 as char);

        if b1.is_some() {
            let c2 = TABLE.get(((n >> 6) & 63) as usize).copied().unwrap_or(b'A');
            out.push(c2 as char);
        }
        if b2.is_some() {
            let c3 = TABLE.get((n & 63) as usize).copied().unwrap_or(b'A');
            out.push(c3 as char);
        }
        i = i.saturating_add(3);
    }
    out
}

/// Signier- und Persistierungs-Hilfsfunktionen für Löschberichte (SPEC §11.7, D-13, ER-27).
pub struct ReportSigner;

impl ReportSigner {
    /// Erstellt einen signierten Bericht aus kanonischen Bytes.
    pub fn sign_bytes(
        report_bytes: &[u8],
        signature: String,
        pub_fingerprint: String,
    ) -> SignedReport {
        SignedReport {
            canonical_body: String::from_utf8_lossy(report_bytes).to_string(),
            signature_base64: signature,
            pub_fingerprint,
        }
    }

    /// Signiert einen kanonischen Bericht mit dem Ed25519-Schlüssel (SPEC §11.7, F.10).
    pub fn sign_report(
        report: &ErasureReport,
        key: &SigningKey,
    ) -> Result<SignedReport, ErasureError> {
        let canonical_bytes = canonicalize_report(report)?;
        let canonical_str = String::from_utf8(canonical_bytes.clone())
            .map_err(|e| ErasureError::Json(e.to_string()))?;

        // Signatur-Domain nach SPEC F.10: "ctx/report/v1" ‖ kanonische_Bytes
        let mut msg = Vec::new();
        msg.extend_from_slice(b"ctx/report/v1");
        msg.extend_from_slice(&canonical_bytes);

        let sig = key.sign(&msg);
        let sig_bytes = sig.to_bytes();
        // Base64url ohne Padding gemäß SPEC §11.7, F.10
        let sig_base64url = base64url_encode(&sig_bytes);

        let pub_key = key.verifying_key();
        let pub_hash = sha2::Sha256::digest(pub_key.as_bytes());
        let slice = pub_hash
            .get(..16)
            .ok_or_else(|| ErasureError::Internal("invalid sha256 slice".into()))?;
        let pub_fingerprint = slice
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();

        Ok(SignedReport {
            canonical_body: canonical_str,
            signature_base64: sig_base64url,
            pub_fingerprint,
        })
    }

    /// Schreibt und synchronisiert den signierten Bericht atomar über Vfs (SPEC §11.8).
    pub fn persist_signed_report(
        vfs: &dyn Vfs,
        target_path: &Path,
        signed: &SignedReport,
    ) -> Result<(), ErasureError> {
        if let Some(parent) = target_path.parent() {
            vfs.create_dir_all(parent)
                .map_err(|e| ErasureError::Internal(e.to_string()))?;
        }

        let serialized =
            serde_json::to_vec(signed).map_err(|e| ErasureError::Json(e.to_string()))?;

        vfs.write_atomic(target_path, &serialized)
            .map_err(|e| ErasureError::Internal(e.to_string()))?;

        if let Some(parent) = target_path.parent() {
            vfs.sync_dir(parent)
                .map_err(|e| ErasureError::Internal(e.to_string()))?;
        }

        Ok(())
    }
}
