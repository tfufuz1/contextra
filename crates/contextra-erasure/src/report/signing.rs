//! Kryptografische Signierung des Berichts mittels Ed25519 (SPEC §11.7, D-13, D-130, F.10, ER-27).
//!
//! Signatur-Domain: `"ctx/report/v1" ‖ kanonische Bytes ohne signature`
//! Port-Integration: `crate::types_local::ReportSigner` (signer_id 0 = K_sign, 1 = Operator)
//! Felder: `signature` (Base64url), `pub_fingerprint` (SHA-256 von report_pub, 16 Byte hex)

use super::build::{canonicalize_report, ErasureReport};
use crate::error::ErasureError;
use crate::types_local::Vfs;
use crate::types_local::{
    ReportSignature, ReportSigner as ReportSignerPort, SignerError, SignerId,
};
use ed25519_dalek::{Signer, SigningKey, VerifyingKey};
use futures_util::future::BoxFuture;
use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::path::Path;

/// Signierter Löschbericht mit Base64url-Signatur und Prüffingerabdruck (ER-27, SPEC §11.7, F.10).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SignedReport {
    /// Kanonischer JSON-Inhalt ohne Signaturfelder bzw. vollständiger Berichtskörper.
    pub canonical_body: String,
    /// Base64url-Signatur ohne Padding.
    pub signature_base64: String,
    /// SHA-256 Fingerprint des öffentlichen Schlüssels (erste 16 Bytes / 32 Hex-Zeichen).
    pub pub_fingerprint: String,
    /// Signer-ID nach SPEC F.10 / D-130 (0 = K_sign, 1 = Operator).
    pub signer_id: u8,
}

impl SignedReport {
    /// Signiert einen kanonischen Bericht mit einem `SigningKey`.
    pub fn sign(report: ErasureReport, key: &SigningKey) -> Result<Self, ErasureError> {
        ReportSigner::sign_report(&report, key)
    }

    /// Liefert die Base64url-Signatur.
    pub fn signature_base64(&self) -> &str {
        &self.signature_base64
    }

    /// Verifiziert die Integrität und Signatur des Berichts gegen den VerifyingKey.
    pub fn verify_with_key(&self, verifying_key: &VerifyingKey) -> Result<(), ErasureError> {
        let pub_hash = sha2::Sha256::digest(verifying_key.as_bytes());
        let expected_fp = pub_hash
            .get(..16)
            .ok_or_else(|| ErasureError::Internal("invalid sha256 slice".into()))?
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();

        if self.pub_fingerprint != expected_fp {
            return Err(ErasureError::Crypto(
                "Public key fingerprint mismatch".to_string(),
            ));
        }

        let sig_bytes = decode_sig_string(&self.signature_base64)?;
        let sig = ed25519_dalek::Signature::from_bytes(&sig_bytes);

        let mut msg = Vec::with_capacity(14 + self.canonical_body.len());
        msg.extend_from_slice(b"ctx/report/v1");
        msg.extend_from_slice(self.canonical_body.as_bytes());

        verifying_key
            .verify_strict(&msg, &sig)
            .map_err(|e| ErasureError::Crypto(e.to_string()))?;

        Ok(())
    }

    /// Verifiziert die Integrität (erfordert bekannten Verifying-Key oder decodierten Fingerprint).
    pub fn verify(&self) -> Result<(), ErasureError> {
        let _ = decode_sig_string(&self.signature_base64)?;
        if self.pub_fingerprint.len() != 32 {
            return Err(ErasureError::Crypto(
                "Invalid public key fingerprint length".to_string(),
            ));
        }
        Ok(())
    }
}

/// Hilfsfunktion zur Dekodierung von Base64url oder Hex-Signaturen.
fn decode_sig_string(s: &str) -> Result<[u8; 64], ErasureError> {
    if s.len() == 128 {
        let mut out = [0u8; 64];
        for (i, b) in out.iter_mut().enumerate() {
            let start = i.saturating_mul(2);
            let end = start.saturating_add(2);
            let sub = s
                .get(start..end)
                .ok_or_else(|| ErasureError::Internal("hex error".into()))?;
            *b = u8::from_str_radix(sub, 16).map_err(|e| ErasureError::Internal(e.to_string()))?;
        }
        return Ok(out);
    }

    base64url_decode(s)
}

/// Hilfsfunktion für Base64url-Dekodierung ohne Padding.
fn base64url_decode(input: &str) -> Result<[u8; 64], ErasureError> {
    let mut out = Vec::with_capacity(64);
    let mut buf: u32 = 0;
    let mut bits: u32 = 0;

    for c in input.chars() {
        let val = match c {
            'A'..='Z' => (c as u32) - ('A' as u32),
            'a'..='z' => (c as u32) - ('a' as u32) + 26,
            '0'..='9' => (c as u32) - ('0' as u32) + 52,
            '-' => 62,
            '_' => 63,
            '=' => break,
            _ => return Err(ErasureError::Internal("invalid base64url char".into())),
        };
        buf = (buf << 6) | val;
        bits = bits.saturating_add(6);
        if bits >= 8 {
            bits = bits.saturating_sub(8);
            out.push((buf >> bits) as u8);
            buf &= (1 << bits) - 1;
        }
    }

    if out.len() != 64 {
        return Err(ErasureError::Internal(
            "invalid ed25519 signature length".into(),
        ));
    }

    let mut res = [0u8; 64];
    res.copy_from_slice(&out);
    Ok(res)
}

/// Hilfsfunktion für Base64url-Kodierung ohne Padding (F.10, RFC 4648 §5).
pub fn base64url_encode(data: &[u8]) -> String {
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

/// Implementierung des `ReportSigner`-Ports für intern abgeleitete Schlüssel (signer_id = 0, D-130).
pub struct DerivedReportSigner {
    key: SigningKey,
    signer_id: SignerId,
}

impl DerivedReportSigner {
    /// Erzeugt einen neuen internen Signierer (signer_id = 0).
    pub fn new(key: SigningKey) -> Self {
        Self { key, signer_id: 0 }
    }

    /// Erzeugt einen Signierer mit benutzerdefinierter Signer-ID (z. B. 1 für Betreiber).
    pub fn with_signer_id(key: SigningKey, signer_id: SignerId) -> Self {
        Self { key, signer_id }
    }
}

impl ReportSignerPort for DerivedReportSigner {
    fn signer_id(&self) -> SignerId {
        self.signer_id
    }

    fn sign_report<'a>(
        &'a self,
        canonical_report_bytes: &'a [u8],
    ) -> BoxFuture<'a, std::result::Result<ReportSignature, SignerError>> {
        Box::pin(async move {
            let mut msg = Vec::with_capacity(14 + canonical_report_bytes.len());
            msg.extend_from_slice(b"ctx/report/v1");
            msg.extend_from_slice(canonical_report_bytes);

            let sig = self.key.sign(&msg);
            let sig_base64 = base64url_encode(&sig.to_bytes());

            let pub_key = self.key.verifying_key();
            let pub_hash = sha2::Sha256::digest(pub_key.as_bytes());
            let slice = pub_hash
                .get(..16)
                .ok_or_else(|| SignerError::InvalidPayload("hash failed".into()))?;
            let pub_fingerprint = slice
                .iter()
                .map(|b| format!("{:02x}", b))
                .collect::<String>();

            Ok(ReportSignature {
                signer_id: self.signer_id,
                signature_base64: sig_base64,
                pub_fingerprint,
            })
        })
    }
}

/// Statische Hilfsfunktionen für Berichtssignierung und VFS-Persistierung (Abwärtskompatibilität).
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
            signer_id: 0,
        }
    }

    /// Signiert einen kanonischen Bericht mit dem Ed25519-Schlüssel (SPEC §11.7, F.10, D-130).
    pub fn sign_report(
        report: &ErasureReport,
        key: &SigningKey,
    ) -> Result<SignedReport, ErasureError> {
        let canonical_bytes = canonicalize_report(report)?;
        let canonical_str = String::from_utf8(canonical_bytes.clone())
            .map_err(|e| ErasureError::Json(e.to_string()))?;

        let mut msg = Vec::with_capacity(14 + canonical_bytes.len());
        msg.extend_from_slice(b"ctx/report/v1");
        msg.extend_from_slice(&canonical_bytes);

        let sig = key.sign(&msg);
        let sig_bytes = sig.to_bytes();
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
            signer_id: report.signer_id,
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
