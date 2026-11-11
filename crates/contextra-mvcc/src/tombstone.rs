// ANCHOR[id=tombstone-origin]
// Herkunft: Wörtliche Kopie aus crates/contextra-types/src/tombstone.rs
// gemäß SPEC-v2 §6.2 "mvcc", Commit A (additiv).

//! # Tombstone-Semantikprüfung für Sequenzprotokolle und Indizes
//!
//! LINK crates/contextra-mvcc/src/lib.rs
//! LINK crates/contextra-mvcc/src/seq_log.rs
//!
//! Dieses Modul definiert Schnittstellen und Referenztypen, um zu prüfen,
//! ob ein Eintrag im MVCC-Versionsverlauf oder Key-Value-Speicher als Soft-Delete (Tombstone)
//! markiert wurde, und erlaubt die deterministische Erzeugung solcher Tombstones.

use contextra_ids::TOMBSTONE_BIT;

// SECTION Tombstone Interface
// ANCHOR[id=tombstone-semantics-trait]
/// Trait zur Bestimmung und Erzeugung von Tombstones in Sequenzprotokollen und Indizes.
///
/// NOTE: In Contextra wird das höchstwertige Bit (`TOMBSTONE_BIT` = 1 << 63) als
/// Indikator für gelöschte Versionen verwendet. Zusätzlich können Low-Level-Flags
/// (z. B. Bit 0x01) signalisieren, dass ein Key-Value-Record ein Tombstone ist.
pub trait TombstoneSemanticsCheck {
    /// Gibt `true` zurück, falls die Sequenznummer oder die Flags eine gelöschte Version (Tombstone) anzeigen.
    fn is_tombstone(seq: u64, flags: u8) -> bool;

    /// Kodiert eine logische Sequenznummer als 8-Byte Little-Endian Bytevektor mit gesetztem `TOMBSTONE_BIT`.
    fn make_tombstone(seq: u64) -> Vec<u8>;
}
// !SECTION Tombstone Interface

// SECTION Reference Implementation
// ANCHOR[id=seqbit-tombstone-impl]
/// Standard-Referenzimplementierung für [`TombstoneSemanticsCheck`].
///
/// Nutzt bitweise ODER-Verknüpfung mit [`TOMBSTONE_BIT`] sowie Flag-Prüfung auf Bit 0.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SeqBitTombstone;

impl TombstoneSemanticsCheck for SeqBitTombstone {
    // ANCHOR[id=seqbit-is-tombstone]
    #[inline]
    fn is_tombstone(seq: u64, flags: u8) -> bool {
        // NOTE: Bitprüfung ist O(1) und allokationsfrei
        (seq & TOMBSTONE_BIT) != 0 || (flags & 0x01) != 0
    }

    // ANCHOR[id=seqbit-make-tombstone]
    #[inline]
    fn make_tombstone(seq: u64) -> Vec<u8> {
        // NOTE: Setzen des MSB kennzeichnet den Eintrag irreversibel als Soft-Delete
        (seq | TOMBSTONE_BIT).to_le_bytes().to_vec()
    }
}
// !SECTION Reference Implementation
