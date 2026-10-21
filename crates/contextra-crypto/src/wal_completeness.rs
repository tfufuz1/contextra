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
/// # Semantik & Bedrohungsmodell
/// - Die Verifikation prüft die exakte Byteweise Gleichheit (`subtle::ConstantTimeEq`) zwischen dem
///   Ende der WAL-HMAC-Kette (`wal_tail_hmac`) und der im Checkpoint-Manifest aufgezeichneten High-Water-Mark.
/// - Ein fehlgeschlagener Vergleich zeigt an, dass nach dem letzten Checkpoint geschriebene WAL-Einträge
///   durch Truncation, Dateisystem-Fehler oder bösartige Rollback-Versuche verloren gegangen oder verfälscht wurden.
/// - **Grenzen:** Diese Funktion ist ein reiner Prädikatsvergleich (Tail == HighWaterMark). Ein Angreifer mit
///   Schreibzugriff auf WAL *und* Checkpoint-Manifest kann beide konsistent zurücksetzen.
///   Der Schutz gegen synchrone Manifest/WAL-Rollbacks wird durch den Audit-Chain-Anker (siehe ADR-042 / Spec §15.6) hergestellt.
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

/// Helper structure representing segment metadata collected during replay for chain validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalSegmentSummary {
    /// Segment sequence number.
    pub segment_seq: u64,
    /// First entry's prev_hmac (or header's prev_segment_hmac).
    pub first_prev_hmac: [u8; 32],
    /// Last entry's HMAC in the segment.
    pub last_hmac: [u8; 32],
    /// Indicates whether any entry in this segment matches the target Checkpoint HWM.
    pub contains_hwm: bool,
}

/// Verifies segment continuity (I-WAL-1) across consecutive WAL segments.
pub fn verify_segment_chain_continuity(
    segments: &[WalSegmentSummary],
) -> Result<(), CryptoError> {
    for window in segments.windows(2) {
        let prev = &window[0];
        let curr = &window[1];
        if curr.segment_seq != prev.segment_seq + 1 {
            return Err(CryptoError::WalTruncationDetected);
        }
        let matches: bool = curr.first_prev_hmac.ct_eq(&prev.last_hmac).into();
        if !matches {
            return Err(CryptoError::WalTruncationDetected);
        }
    }
    Ok(())
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
