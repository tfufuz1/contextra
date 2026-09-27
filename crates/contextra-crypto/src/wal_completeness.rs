// FILE-CONTEXT
// ZWECK: Verification of WAL log tail completeness against checkpoint manifest high-water-mark to detect truncation attacks.
// INVARIANTEN: Zero-I/O, zero async, zero non-determinism, constant-time HMAC comparison (subtle::ConstantTimeEq).
// HOTSPOTS: [verify_wal_chain_completeness]

#![forbid(unsafe_code)]

//! Modul zur Verifikation der WAL-Ketten-Vollständigkeit.
//!
//! Schützt gegen unbemerkte WAL-Truncation-Angriffe und partielle Log-Dateitrunkierungen.

use crate::error::CryptoError;
use subtle::ConstantTimeEq;

/// Verifiziert, dass der WAL-Tail nicht abgeschnitten wurde, indem der
/// letzte bekannte WAL-HMAC mit der im Checkpoint-Manifest persistierten
/// High-Water-Mark verglichen wird.
///
/// # Fehler
/// `CryptoError::WalTruncationDetected` wenn `wal_tail_hmac != manifest_high_water_mark`.
///
/// # Sicherheitshinweis
/// Diese Funktion bietet keine Garantie gegen einen Angreifer, der sowohl den
/// WAL als auch das Checkpoint-Manifest kontrolliert. Sie schützt gegen
/// teilweise Dateitrunkierungen durch Dateisystem- oder Hardware-Fehler sowie
/// gegen Angreifer, die nur WAL-Schreibzugriff, aber keinen Manifest-Zugriff haben.
pub fn verify_wal_chain_completeness(
    wal_tail_hmac: &[u8; 32],
    manifest_high_water_mark: &[u8; 32],
) -> Result<(), CryptoError> {
    if wal_tail_hmac.ct_eq(manifest_high_water_mark).into() {
        Ok(())
    } else {
        Err(CryptoError::WalTruncationDetected)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_wal_chain_completeness_matching_hmac() {
        let hmac = [0x42u8; 32];
        assert!(verify_wal_chain_completeness(&hmac, &hmac).is_ok());
    }

    #[test]
    fn test_verify_wal_chain_completeness_mismatched_hmac() {
        let hmac_a = [0x42u8; 32];
        let mut hmac_b = [0x42u8; 32];
        hmac_b[31] ^= 0x01;

        let res = verify_wal_chain_completeness(&hmac_a, &hmac_b);
        assert!(matches!(res, Err(CryptoError::WalTruncationDetected)));
    }
}
