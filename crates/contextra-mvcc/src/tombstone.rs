// Herkunft: Wörtliche Kopie aus crates/contextra-types/src/tombstone.rs
// gemäß SPEC-v2 §6.2 "mvcc", Commit A (additiv).

//! Trait und Referenzimplementierung für Tombstone-Semantikprüfung in Sequenzprotokollen und Indizes.

use contextra_ids::TOMBSTONE_BIT;

// ANCHOR[contextra-mvcc:TOMB-001]
/// Trait defining tombstone semantics for index sequence numbers and key-value records.
pub trait TombstoneSemanticsCheck {
    /// Returns `true` if the sequence number or flags indicate a soft-deleted tombstone.
    fn is_tombstone(seq: u64, flags: u8) -> bool;

    /// Encodes a sequence number as an 8-byte little-endian tombstone payload with `TOMBSTONE_BIT` set.
    fn make_tombstone(seq: u64) -> Vec<u8>;
}

// ANCHOR[contextra-mvcc:TOMB-002]
/// Default reference implementation of [`TombstoneSemanticsCheck`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SeqBitTombstone;

impl TombstoneSemanticsCheck for SeqBitTombstone {
    // ANCHOR[contextra-mvcc:TOMB-003]
    #[inline]
    fn is_tombstone(seq: u64, flags: u8) -> bool {
        (seq & TOMBSTONE_BIT) != 0 || (flags & 0x01) != 0
    }

    // ANCHOR[contextra-mvcc:TOMB-004]
    #[inline]
    fn make_tombstone(seq: u64) -> Vec<u8> {
        (seq | TOMBSTONE_BIT).to_le_bytes().to_vec()
    }
}
