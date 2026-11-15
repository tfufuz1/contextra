//! # WAL-Vollständigkeitsprüfung (`wal_completeness.rs`)
//!
//! Prüft, ob ein WAL-Segment lückenlos vom Start bis zum Ende erhalten ist.
//!
//! ## Invarianten
//! - Gibt Fehler zurück, wenn die berechnete HMAC-Kette nicht mit dem erwarteten Endwert übereinstimmt.

// FILE-CONTEXT
// ZWECK: Verification of WAL log tail completeness against checkpoint manifest high-water-mark to detect truncation attacks.
// INVARIANTEN: Zero-I/O, zero async, zero non-determinism, constant-time HMAC comparison (subtle::ConstantTimeEq).
// HOTSPOTS: [verify_wal_chain_completeness]

#![forbid(unsafe_code)]

//! Modul zur Verifikation der WAL-Ketten-Vollständigkeit.
//!
//! Schützt gegen unbemerkte WAL-Truncation-Angriffe und partielle Log-Dateitrunkierungen.

// SECTION [epic=crypto-wal,seq=1] Imports
use crate::error::CryptoError;
use subtle::ConstantTimeEq;

// !SECTION

// SECTION [epic=crypto-wal,seq=2] WAL-Vollständigkeitsprüfung
// ANCHOR[acta-crypto:WAL-002] Verifikation des WAL-Tails gegen High-Water-Mark
// NOTE: Erkennt Truncation-Angriffe und partielle Dateikorrumpierung (TS:2026-11-11)
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

// !SECTION

// SECTION [epic=crypto-wal,seq=3] Tests
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
// !SECTION
