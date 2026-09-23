//! Trait and reference implementation for tombstone semantics checking in MemFuse sequence logs and indexes.

// FILE-CONTEXT
// STAND: 2026-09-16T21:00:00Z (SESSION: IP-07)
// ZWECK: Trait und Referenzimplementierung für Tombstone-Semantik (IP-07 / ADR-041).
// INVARIANTEN: Bit 63 der SeqNo kennzeichnet Tombstones (TOMBSTONE_BIT). Trait-Methoden paniken niemals.

use crate::types::TOMBSTONE_BIT;

/// Trait defining tombstone semantics for index sequence numbers and key-value records.
///
/// # Bit-Mask Convention (IP-07 / ADR-041)
/// By convention in MemFuse, sequence numbers (`u64`) reserve the most significant bit (bit 63)
/// as the `TOMBSTONE_BIT` (`1 << 63`). When bit 63 is set (`(seq & TOMBSTONE_BIT) != 0`), the record
/// or index entry represents a soft deletion (tombstone).
///
/// Alternatively, flag bytes (`u8`) can use bit 0 (`flags & 0x01 != 0`) as an explicit tombstone indicator.
///
/// Implementations of this trait provide a unified interface to verify whether sequence/flag
/// tuples represent tombstones and to construct tombstone byte payloads.
///
/// # Zero-Panic Guarantees
/// All methods in this trait perform purely bitwise operations and infallible memory allocations,
/// guaranteeing zero panic across all input ranges.
pub trait TombstoneSemanticsCheck {
    /// Returns `true` if the sequence number or flags indicate a soft-deleted tombstone.
    fn is_tombstone(seq: u64, flags: u8) -> bool;

    /// Encodes a sequence number as an 8-byte little-endian tombstone payload with `TOMBSTONE_BIT` set.
    fn make_tombstone(seq: u64) -> Vec<u8>;
}

/// Default reference implementation of [`TombstoneSemanticsCheck`].
///
/// Uses bit 63 (`TOMBSTONE_BIT = 1 << 63`) in `seq: u64` or bit 0 in `flags: u8` (`flags & 0x01 != 0`)
/// to identify and construct tombstones.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SeqBitTombstone;

impl TombstoneSemanticsCheck for SeqBitTombstone {
    /// Returns `true` if `(seq & TOMBSTONE_BIT) != 0` OR if flag bit 0 (`(flags & 0x01) != 0`) is set.
    #[inline]
    fn is_tombstone(seq: u64, flags: u8) -> bool {
        (seq & TOMBSTONE_BIT) != 0 || (flags & 0x01) != 0
    }

    /// Encodes `seq` with `TOMBSTONE_BIT` set into an 8-byte little-endian byte vector.
    ///
    /// If `seq` already has `TOMBSTONE_BIT` set, bit 63 remains set without double-wrapping or error.
    #[inline]
    fn make_tombstone(seq: u64) -> Vec<u8> {
        (seq | TOMBSTONE_BIT).to_le_bytes().to_vec()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip_property() {
        let test_seqs = [0u64, 1, 42, 1_000_000, max_seq_simulation()];
        for &seq in &test_seqs {
            let bytes = SeqBitTombstone::make_tombstone(seq);
            assert_eq!(bytes.len(), 8);

            let arr: [u8; 8] = [
                bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
            ];
            let encoded_seq = u64::from_le_bytes(arr);
            assert!(
                SeqBitTombstone::is_tombstone(encoded_seq, 0),
                "Encoded sequence must evaluate to tombstone"
            );
            assert_eq!(
                encoded_seq & !TOMBSTONE_BIT,
                seq & !TOMBSTONE_BIT,
                "Stripped sequence number must equal original raw sequence number"
            );
        }
    }

    #[test]
    fn test_negative_cases() {
        let non_tombstones = [
            (0u64, 0u8),
            (1u64, 0u8),
            (42u64, 0u8),
            (1_000_000u64, 0u8),
            (!TOMBSTONE_BIT, 0u8),
            (42u64, 0x02u8),
            (42u64, 0xFEu8),
        ];

        for &(seq, flags) in &non_tombstones {
            assert!(
                !SeqBitTombstone::is_tombstone(seq, flags),
                "Sequence {seq} with flags {flags:#x} should NOT be recognized as tombstone"
            );
        }
    }

    #[test]
    fn test_flags_tombstone_detection() {
        assert!(SeqBitTombstone::is_tombstone(0, 0x01));
        assert!(SeqBitTombstone::is_tombstone(42, 0x01));
        assert!(SeqBitTombstone::is_tombstone(100, 0xFF));
        assert!(!SeqBitTombstone::is_tombstone(100, 0xFE));
    }

    #[test]
    fn test_edge_case_already_tombstoned_seq() {
        let raw_seq = 100u64;
        let already_tombstoned = raw_seq | TOMBSTONE_BIT;

        assert!(SeqBitTombstone::is_tombstone(already_tombstoned, 0));

        let bytes_from_raw = SeqBitTombstone::make_tombstone(raw_seq);
        let bytes_from_tombstoned = SeqBitTombstone::make_tombstone(already_tombstoned);

        assert_eq!(
            bytes_from_raw, bytes_from_tombstoned,
            "make_tombstone on already tombstoned seq should produce identical output"
        );
    }

    fn max_seq_simulation() -> u64 {
        (1u64 << 62) - 1
    }
}
