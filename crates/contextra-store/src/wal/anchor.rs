//! FILE-CONTEXT: STAND/ZWECK/INVARIANTEN
//! Stand: 2026-10-21
//! Zweck: WAL-002 High-Water-Mark Anchoring and Anchor Verification (§ADR-N12).
//! Invarianten:
//! - Verification checks expected_hwm against replay set, previous HMAC of first frame, or empty log state.
//! - Never compares expected_hwm tautologically with itself.

use contextra_types::{ContextraError, Result};
use std::collections::HashSet;

/// Result of WAL anchor verification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnchorCheck {
    /// HWM anchor found in replayed frame HMACs.
    HwmInReplay,
    /// First replayed frame's previous HMAC matched HWM anchor (exact continuation after checkpoint/retirement).
    FirstFramePrevMatches,
    /// WAL log was empty after retirement and expected_hwm matches tail HMAC.
    EmptyAfterRetire,
}

/// Verifies that the expected manifest High-Water-Mark (HWM) anchor matches the live WAL state.
///
/// # Errors
/// Returns [`ContextraError::WalTruncationDetected`] if the anchor is missing and truncation occurred.
pub fn verify_wal_anchor(
    expected_hwm: &[u8; 32],
    actual_tail_hmac: &[u8; 32],
    replayed_hmac_set: &HashSet<[u8; 32]>,
    first_frame_prev_hmac: Option<&[u8; 32]>,
    entries_empty: bool,
) -> Result<AnchorCheck> {
    if *expected_hwm == [0u8; 32] {
        return Ok(AnchorCheck::EmptyAfterRetire);
    }

    if replayed_hmac_set.contains(expected_hwm) {
        return Ok(AnchorCheck::HwmInReplay);
    }

    if let Some(prev) = first_frame_prev_hmac {
        if prev == expected_hwm {
            return Ok(AnchorCheck::FirstFramePrevMatches);
        }
    }

    if entries_empty && expected_hwm == actual_tail_hmac {
        return Ok(AnchorCheck::EmptyAfterRetire);
    }

    Err(ContextraError::wal_truncation_detected(
        *expected_hwm,
        *actual_tail_hmac,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_wal_anchor_cases() {
        let hwm = [0xAA; 32];
        let mut set = HashSet::new();

        // Null HWM
        assert_eq!(
            verify_wal_anchor(&[0; 32], &[0; 32], &set, None, true).unwrap(),
            AnchorCheck::EmptyAfterRetire
        );

        // HWM in replay set
        set.insert(hwm);
        assert_eq!(
            verify_wal_anchor(&hwm, &[0xBB; 32], &set, None, false).unwrap(),
            AnchorCheck::HwmInReplay
        );

        // First frame prev matches
        let empty_set = HashSet::new();
        assert_eq!(
            verify_wal_anchor(&hwm, &[0xBB; 32], &empty_set, Some(&hwm), false).unwrap(),
            AnchorCheck::FirstFramePrevMatches
        );

        // Truncation detected
        assert!(verify_wal_anchor(&hwm, &[0xBB; 32], &empty_set, None, false).is_err());
    }
}
