//! Ed25519 report signing and verification (D-13, F.10).

use crate::error::{CryptoError, Result};
use crate::hierarchy::SignSeed;
use crate::labels;
use contextra_ports::report_signer::{ReportSignature, ReportSigner, SignerError, SignerId};
use contextra_ports::BoxFuture;
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use sha2::{Digest, Sha256};

/// Helper for base64url encoding without padding (RFC 4648 §5, SPEC F.10).
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

        let n = ((b0 as u32) << 16)
            | (match b1 {
                Some(b) => (b as u32) << 8,
                None => 0,
            })
            | match b2 {
                Some(b) => b as u32,
                None => 0,
            };

        let c0 = match TABLE.get(((n >> 18) & 63) as usize) {
            Some(&c) => c,
            None => b'A',
        };
        let c1 = match TABLE.get(((n >> 12) & 63) as usize) {
            Some(&c) => c,
            None => b'A',
        };
        out.push(c0 as char);
        out.push(c1 as char);

        if b1.is_some() {
            let c2 = match TABLE.get(((n >> 6) & 63) as usize) {
                Some(&c) => c,
                None => b'A',
            };
            out.push(c2 as char);
        }
        if b2.is_some() {
            let c3 = match TABLE.get((n & 63) as usize) {
                Some(&c) => c,
                None => b'A',
            };
            out.push(c3 as char);
        }
        i = i.saturating_add(3);
    }
    out
}

/// Computes hex fingerprint from public verifying key (SHA-256 first 16 bytes / 32 hex chars).
fn pub_fingerprint_hex(verifying_key: &VerifyingKey) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let digest = Sha256::digest(verifying_key.as_bytes());
    let mut s = String::with_capacity(32);
    if let Some(prefix) = digest.get(..16) {
        for &b in prefix {
            let high = match HEX.get((b >> 4) as usize) {
                Some(&c) => c as char,
                None => '0',
            };
            let low = match HEX.get((b & 0x0f) as usize) {
                Some(&c) => c as char,
                None => '0',
            };
            s.push(high);
            s.push(low);
        }
    }
    s
}

/// Standard internal report signer using derived `K_sign` (SPEC §8.2, §11.7, D-80, D-130, F.10).
pub struct DerivedReportSigner {
    signing_key: SigningKey,
    pub_fingerprint: String,
}

impl DerivedReportSigner {
    /// Creates a new `DerivedReportSigner` from a strongly-typed `SignSeed`.
    pub fn new(seed: &SignSeed) -> Self {
        let signing_key = SigningKey::from_bytes(seed.0.as_bytes());
        let pub_fingerprint = pub_fingerprint_hex(&signing_key.verifying_key());
        Self {
            signing_key,
            pub_fingerprint,
        }
    }

    /// Exposes verifying key.
    pub fn verifying_key(&self) -> VerifyingKey {
        self.signing_key.verifying_key()
    }
}

impl ReportSigner for DerivedReportSigner {
    fn signer_id(&self) -> SignerId {
        0
    }

    fn sign_report<'a>(
        &'a self,
        canonical_report_bytes: &'a [u8],
    ) -> BoxFuture<'a, std::result::Result<ReportSignature, SignerError>> {
        let mut msg =
            Vec::with_capacity(labels::REPORT_PREFIX.len() + canonical_report_bytes.len());
        msg.extend_from_slice(labels::REPORT_PREFIX);
        msg.extend_from_slice(canonical_report_bytes);
        let sig = self.signing_key.sign(&msg);
        let signature_base64 = base64url_encode(&sig.to_bytes());
        let report_sig = ReportSignature {
            signer_id: self.signer_id(),
            signature_base64,
            pub_fingerprint: self.pub_fingerprint.clone(),
        };
        Box::pin(async move { Ok(report_sig) })
    }
}

/// Signs a canonical report message with prefix ctx/report/v1.
///
/// # Inputs
/// * `seed`: Strongly-typed `SignSeed` (32 bytes) derived from the master key.
/// * `canon`: Canonical report byte slice to be signed.
///
/// # Properties & Security
/// Prepends `labels::REPORT_PREFIX` (`ctx/report/v1`) to prevent cross-protocol signature
/// confusion. Generates a 64-byte Ed25519 signature.
pub fn sign_report(seed: &SignSeed, canon: &[u8]) -> [u8; 64] {
    let signing_key = SigningKey::from_bytes(seed.0.as_bytes());
    let mut msg = Vec::with_capacity(labels::REPORT_PREFIX.len() + canon.len());
    msg.extend_from_slice(labels::REPORT_PREFIX);
    msg.extend_from_slice(canon);
    signing_key.sign(&msg).to_bytes()
}

/// Verifies a canonical report signature against an Ed25519 public key using verify_strict (RFC 8032).
pub fn verify_report(pubkey: &[u8; 32], canon: &[u8], sig: &[u8; 64]) -> Result<()> {
    let verifying_key = VerifyingKey::from_bytes(pubkey).map_err(|_| CryptoError::Auth)?;
    let signature = Signature::from_bytes(sig);
    let mut msg = Vec::with_capacity(labels::REPORT_PREFIX.len() + canon.len());
    msg.extend_from_slice(labels::REPORT_PREFIX);
    msg.extend_from_slice(canon);
    verifying_key
        .verify_strict(&msg, &signature)
        .map_err(|_| CryptoError::Auth)
}
