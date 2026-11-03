//! Kryptografische Signierung des Berichts mittels Ed25519 (SPEC §11.7, D-13, ER-27).
//!
//! Signatur-Domain: `"ctx/report/v1" ‖ kanonische Bytes ohne signature`
//! Felder: `signature` (Base64url), `pub_fingerprint`

use super::SignedReport;

/// Signier-Hilfsfunktionen für Löschberichte (ER-27).
pub struct ReportSigner;

impl ReportSigner {
    /// Erstellt einen signierten Bericht aus kanonischen Bytes (Platzhalter).
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
}
