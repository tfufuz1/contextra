//! WAL-Ketten-Vollständigkeitsprüfung (`chain_check.rs`)
//!
//! Prüft, ob ein WAL-Segment lückenlos vom Start bis zum Ende erhalten ist
//! und ob die berechnete HMAC-Kette mit dem erwarteten Endwert übereinstimmt (SPEC 10.4, P14, P6).

use contextra_types::{ContextraError, Result};
use subtle::ConstantTimeEq;

/// Verifiziert, dass der WAL-Tail nicht abgeschnitten oder manipuliert wurde,
/// indem der letzte berechnete WAL-HMAC mit der im Manifest erwarteten
/// High-Water-Mark verglichen wird.
///
/// Truncation, Lücken, vertauschte Reihenfolge und Fremdschlüssel führen
/// zu einem typisierten Fehler (`ContextraError::WalTruncationDetected` oder `WalCorruption`).
pub fn verify_wal_chain_completeness(
    wal_tail_hmac: &[u8; 32],
    manifest_high_water_mark: &[u8; 32],
) -> Result<()> {
    if wal_tail_hmac.ct_eq(manifest_high_water_mark).into() {
        Ok(())
    } else {
        Err(ContextraError::wal_truncation_detected(
            *manifest_high_water_mark,
            *wal_tail_hmac,
        ))
    }
}

/// Validiert eine Sequenz von WAL-Einträgen auf Lücken, Truncation, Vertauschung
/// und Fremdschlüssel/Ketten-Integrität.
pub fn verify_wal_entry_sequence(
    entries: &[(u64, [u8; 32], [u8; 32])], // (seq_no, prev_hmac, checksum)
    expected_start_hmac: &[u8; 32],
    expected_end_hmac: &[u8; 32],
) -> Result<()> {
    let mut current_hmac = *expected_start_hmac;
    let mut last_seq: Option<u64> = None;

    for (seq_no, prev_hmac, checksum) in entries {
        if let Some(prev_seq) = last_seq {
            if *seq_no <= prev_seq {
                return Err(ContextraError::wal_corruption(
                    *seq_no,
                    format!("Swapped or non-monotonic sequence detected: {seq_no} <= {prev_seq}"),
                ));
            }
            if *seq_no > prev_seq + 1 {
                return Err(ContextraError::wal_corruption(
                    *seq_no,
                    format!(
                        "Sequence gap detected: expected {}, got {}",
                        prev_seq + 1,
                        seq_no
                    ),
                ));
            }
        }

        if !bool::from(prev_hmac.ct_eq(&current_hmac)) {
            return Err(ContextraError::wal_corruption(
                *seq_no,
                format!(
                    "Chain linkage mismatch at seq {}: prev_hmac does not match previous link",
                    seq_no
                ),
            ));
        }

        current_hmac = *checksum;
        last_seq = Some(*seq_no);
    }

    verify_wal_chain_completeness(&current_hmac, expected_end_hmac)
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

    #[test]
    fn test_verify_wal_entry_sequence_gaps_detected() {
        let entries = vec![
            (1, [0u8; 32], [1u8; 32]),
            (3, [1u8; 32], [2u8; 32]), // Gap: 1 -> 3
        ];
        let res = verify_wal_entry_sequence(&entries, &[0u8; 32], &[2u8; 32]);
        assert!(matches!(res, Err(ContextraError::WalCorruption { .. })));
    }

    #[test]
    fn test_verify_wal_entry_sequence_swapped_order_detected() {
        let entries = vec![
            (2, [0u8; 32], [1u8; 32]),
            (1, [1u8; 32], [2u8; 32]), // Swapped
        ];
        let res = verify_wal_entry_sequence(&entries, &[0u8; 32], &[2u8; 32]);
        assert!(matches!(res, Err(ContextraError::WalCorruption { .. })));
    }

    #[test]
    fn test_verify_wal_entry_sequence_broken_chain_link_detected() {
        let entries = vec![
            (1, [0u8; 32], [1u8; 32]),
            (2, [99u8; 32], [2u8; 32]), // Broken link: prev is 99 instead of 1
        ];
        let res = verify_wal_entry_sequence(&entries, &[0u8; 32], &[2u8; 32]);
        assert!(matches!(res, Err(ContextraError::WalCorruption { .. })));
    }

    #[test]
    fn test_verify_wal_entry_sequence_truncation_at_end_detected() {
        let entries = vec![(1, [0u8; 32], [1u8; 32]), (2, [1u8; 32], [2u8; 32])];
        // Expected end is 3 instead of 2 (tail truncated)
        let res = verify_wal_entry_sequence(&entries, &[0u8; 32], &[3u8; 32]);
        assert!(matches!(
            res,
            Err(ContextraError::WalTruncationDetected { .. })
        ));
    }
}
