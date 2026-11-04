//! WAL-Ketten-Vollständigkeitsprüfung (`chain_check.rs`)
//!
//! Prüft, ob ein WAL-Segment lückenlos vom Start bis zum Ende erhalten ist
//! und ob die berechnete HMAC-Kette mit dem erwarteten Endwert übereinstimmt (SPEC 10.4, P14, P6).

use contextra_types::{ContextraError, Result};

/// Verifiziert, dass der WAL-Tail nicht abgeschnitten oder manipuliert wurde,
/// indem der letzte berechnete WAL-HMAC mit der im Manifest erwarteten
/// High-Water-Mark verglichen wird.
///
/// Delegiert an die kanonische Krypto-Implementierung `contextra_crypto::verify_wal_chain_completeness`.
pub fn verify_wal_chain_completeness(
    wal_tail_hmac: &[u8; 32],
    manifest_high_water_mark: &[u8; 32],
) -> Result<()> {
    match contextra_crypto::verify_wal_chain_completeness(manifest_high_water_mark, wal_tail_hmac) {
        Ok(()) => Ok(()),
        Err(_) => Err(ContextraError::wal_truncation_detected(
            *manifest_high_water_mark,
            *wal_tail_hmac,
        )),
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
        assert!(matches!(
            res,
            Err(ContextraError::WalTruncationDetected { .. })
        ));
    }
}
