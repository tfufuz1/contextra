//! Trait und Referenzimplementierung für Tombstone-Semantikprüfung in Sequenzprotokollen und Indizes.
//!
//! # Zweck & Architektur-Rolle: Ring 0
//!
//! In Multi-Version Concurrency Control (MVCC) und LSM-Tree-Speichern werden Datensätze
//! bei einer Löschung nicht physisch überschrieben, sondern mit einem Löschmarker ("Tombstone")
//! versehen.
//!
//! Gemäß ADR-041 und IP-07 wird dafür das höchstwertige Bit 63 der Sequenznummer (`u64`) reserviert.
//! Ist Bit 63 gesetzt (`TOMBSTONE_BIT`), handelt es sich um eine Soft-Deletion.
//! Alternativ kann ein Flag-Byte mit gesetztem Bit 0 (`0x01`) als Indikator dienen.

// FILE-CONTEXT
// STAND: 2026-09-16T21:00:00Z (SESSION: IP-07)
// ZWECK: Trait und Referenzimplementierung für Tombstone-Semantik (IP-07 / ADR-041).
// INVARIANTEN: Bit 63 der SeqNo kennzeichnet Tombstones (TOMBSTONE_BIT). Trait-Methoden paniken niemals.

use crate::types::TOMBSTONE_BIT;

// ANCHOR[contextra-types:TOMB-001]
/// Trait defining tombstone semantics for index sequence numbers and key-value records.
///
/// # Bit-Mask Convention (IP-07 / ADR-041)
/// By convention in Contextra, sequence numbers (`u64`) reserve the most significant bit (bit 63)
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
// ANCHOR[contextra-types:TOMB-001]
pub trait TombstoneSemanticsCheck {
    /// Returns `true` if the sequence number or flags indicate a soft-deleted tombstone.
    fn is_tombstone(seq: u64, flags: u8) -> bool;

    /// Encodes a sequence number as an 8-byte little-endian tombstone payload with `TOMBSTONE_BIT` set.
    fn make_tombstone(seq: u64) -> Vec<u8>;
}

// ANCHOR[contextra-types:TOMB-002]
/// Default reference implementation of [`TombstoneSemanticsCheck`].
///
/// Uses bit 63 (`TOMBSTONE_BIT = 1 << 63`) in `seq: u64` or bit 0 in `flags: u8` (`flags & 0x01 != 0`)
/// to identify and construct tombstones.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
// ANCHOR[contextra-types:TOMB-002]
pub struct SeqBitTombstone;

impl TombstoneSemanticsCheck for SeqBitTombstone {
    // ANCHOR[contextra-types:TOMB-003]
    /// Returns `true` if `(seq & TOMBSTONE_BIT) != 0` OR if flag bit 0 (`(flags & 0x01) != 0`) is set.
    ///
    /// # Ablauf
    /// 1. Prüfen, ob Bit 63 in `seq` gesetzt ist.
    /// 2. Prüfen, ob Bit 0 in `flags` gesetzt ist.
    /// 3. Liefert `true`, wenn mindestens eine Bedingung wahr ist.
    #[inline]
    fn is_tombstone(seq: u64, flags: u8) -> bool {
        (seq & TOMBSTONE_BIT) != 0 || (flags & 0x01) != 0
    }

    // ANCHOR[contextra-types:TOMB-004]
    /// Encodes `seq` with `TOMBSTONE_BIT` set into an 8-byte little-endian byte vector.
    ///
    /// If `seq` already has `TOMBSTONE_BIT` set, bit 63 remains set without double-wrapping or error.
    ///
    /// # Ablauf
    /// Verodert `seq` mit `TOMBSTONE_BIT` und serialisiert das Ergebnis in 8 Little-Endian-Bytes.
    #[inline]
    fn make_tombstone(seq: u64) -> Vec<u8> {
        (seq | TOMBSTONE_BIT).to_le_bytes().to_vec()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    // NOTE: Prüft Roundtrip von Sequenznummern zu Tombstone-Bytes und zurück.
    #[test]
    fn test_roundtrip_property() {
        let test_seqs = [0u64, 1, 42, 1_000_000, max_seq_simulation()];
        for &seq in &test_seqs {
            let bytes = SeqBitTombstone::make_tombstone(seq);
            assert_eq!(bytes.len(), 8);

            let arr: [u8; 8] = bytes.as_slice().try_into().expect("8 bytes");
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

    // NOTE: Prüft, dass Sequenzen ohne Tombstone-Bit oder -Flags nicht als Tombstone gewertet werden.
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

    // NOTE: Prüft die Tombstone-Erkennung über das Flags-Byte (Bit 0).
    #[test]
    fn test_flags_tombstone_detection() {
        assert!(SeqBitTombstone::is_tombstone(0, 0x01));
        assert!(SeqBitTombstone::is_tombstone(42, 0x01));
        assert!(SeqBitTombstone::is_tombstone(100, 0xFF));
        assert!(!SeqBitTombstone::is_tombstone(100, 0xFE));
    }

    // NOTE: Prüft Idempotenz bei bereits gesetztem Tombstone-Bit.
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

    // NOTE: Hilfsfunktion zur Erzeugung einer maximalen regulären Sequenznummer.
    fn max_seq_simulation() -> u64 {
        (1u64 << 62) - 1
    }
}
