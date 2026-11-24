//! Tombstone-Erkennung und Bit-63-Maskierung (Ring 1)
//!
//! Stellt die Kernkonstante `TOMBSTONE_BIT` und Hilfsfunktionen zur
//! Erkennung gelöschter Schlüssel zur Verfügung.

pub use contextra_ids::TOMBSTONE_BIT;
use std::ops::Bound;

// Compile-time Invariante: TOMBSTONE_BIT muss exakt Bit 63 sein für atomare Bitmaskierung.
const _: () = assert!(TOMBSTONE_BIT == 1u64 << 63);

/// Helper function to check if a sequence number has TOMBSTONE_BIT set.
#[inline]
pub const fn is_tombstone(seq: u64) -> bool {
    (seq & TOMBSTONE_BIT) != 0
}

/// Validates whether start and end key range bounds define a non-empty, valid interval (`start <= end`).
#[inline]
pub fn is_valid_range_bounds(start: Bound<&[u8]>, end: Bound<&[u8]>) -> bool {
    match (start, end) {
        (Bound::Included(s), Bound::Included(e)) => s <= e,
        (Bound::Included(s), Bound::Excluded(e))
        | (Bound::Excluded(s), Bound::Included(e))
        | (Bound::Excluded(s), Bound::Excluded(e)) => s < e,
        _ => true,
    }
}
