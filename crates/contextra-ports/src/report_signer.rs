//! Report signing port trait and types (SPEC §3.1, §8.2, §11.7, D-130, F.10, T-SIG-1).
//!
//! # Normative Referenz
//! - **D-130:** Kryptografie-Abnahme und Berichtssignatur.
//! - **F.10:** Berichtsformat (kanonisches JSON), Signatur über `"ctx/report/v1" ‖ kanonische_Bytes`.
//!   `signer_id` (0 = K_sign abgeleitet, 1 = betreibereigener Signierer).
//! - Ports beschreiben nur Verhalten, nicht Implementierung. Standardimplementierung
//!   mit Ed25519 liegt in `crypto` bzw. `erasure`.

use crate::BoxFuture;
use std::fmt;
use std::sync::Arc;

/// Signer ID per SPEC F.10 (0 = internal/derived K_sign, 1 = operator signer).
pub type SignerId = u8;

/// Digital signature over canonical report bytes (SPEC F.10).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportSignature {
    /// Signer identifier (u8: 0 = internal, 1 = operator / HSM).
    pub signer_id: SignerId,
    /// Base64url-encoded signature string without padding (RFC 4648 §5, SPEC F.10).
    pub signature_base64: String,
    /// Hex-encoded SHA-256 fingerprint (first 16 bytes / 32 hex chars) of verifying public key.
    pub pub_fingerprint: String,
}

/// Error class for report signing operations (SPEC D-130, 7.5).
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SignerError {
    /// Signing key unavailable or corrupted.
    #[error("signing key unavailable: {0}")]
    KeyUnavailable(String),

    /// Hardware security module or external signature provider error.
    #[error("external signer failed: {0}")]
    ExternalSignerFailed(String),

    /// Invalid report payload or canonical encoding failure.
    #[error("invalid report payload: {0}")]
    InvalidPayload(String),
}

/// Abstract port trait for signing audit reports (SPEC §8.2, D-130, T-SIG-1).
pub trait ReportSigner: Send + Sync {
    /// Returns the unique numeric signer identifier according to SPEC F.10.
    fn signer_id(&self) -> SignerId;

    /// Signs the canonical report bytes.
    ///
    /// # Parameter
    /// - `canonical_report_bytes`: canonical JSON representation according to SPEC F.10.
    ///
    /// # Fehler
    /// Liefert [`SignerError`], wenn das kryptografische Signieren scheitert.
    ///
    /// # Crash
    /// Reines Berechnungs- oder HSM-Protokoll ohne ungesicherten Seiteneffekt.
    fn sign_report<'a>(
        &'a self,
        canonical_report_bytes: &'a [u8],
    ) -> BoxFuture<'a, std::result::Result<ReportSignature, SignerError>>;
}

impl<T: ReportSigner + ?Sized> ReportSigner for Arc<T> {
    fn signer_id(&self) -> SignerId {
        (**self).signer_id()
    }

    fn sign_report<'a>(
        &'a self,
        canonical_report_bytes: &'a [u8],
    ) -> BoxFuture<'a, std::result::Result<ReportSignature, SignerError>> {
        (**self).sign_report(canonical_report_bytes)
    }
}

impl fmt::Display for ReportSignature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ReportSignature(signer_id={}, fp={})",
            self.signer_id, self.pub_fingerprint
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::FutureExt;

    struct DummySigner;

    impl ReportSigner for DummySigner {
        fn signer_id(&self) -> SignerId {
            0
        }

        fn sign_report<'a>(
            &'a self,
            _canonical_report_bytes: &'a [u8],
        ) -> BoxFuture<'a, std::result::Result<ReportSignature, SignerError>> {
            Box::pin(async move {
                Ok(ReportSignature {
                    signer_id: 0,
                    signature_base64: "dummy_sig".to_string(),
                    pub_fingerprint: "0123456789abcdef".to_string(),
                })
            })
        }
    }

    /// SPEC: T-SIG-1
    #[test]
    fn test_dyn_report_signer() {
        let signer: Arc<dyn ReportSigner> = Arc::new(DummySigner);
        assert_eq!(signer.signer_id(), 0);
        let sig = signer
            .sign_report(b"{}")
            .now_or_never()
            .expect("now or never");
        assert!(sig.is_ok());
    }
}
