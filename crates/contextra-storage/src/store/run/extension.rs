//! # Extension: Erweiterungsschnittstelle für zusätzliche Run-Blöcke
//!
//! Gemäß SPEC §12.4:
//! Schnittstelle für obere Schichten zur Erzeugung und Zusammenführung
//! blockspezifischer Sekundärindizes (Postings, Zähler, Bloom).
//! Block-Art im Directory ist `0x1000 + id`.
//!
//! **Stabilitätshinweis (SPEC §12.4, A-10):**
//! Die Erweiterungsschnittstelle gilt bis Phase F10 als **instabil**.
//! Keine Integer-Casts (M-02 = 0).

use super::block::block_kind::EXTENSION_BASE;
use super::data::DataEntry;
use std::io;

/// Erweiterungsschnittstelle für anwendungsspezifische Index-Blöcke je Run.
///
/// Implementierungen erzeugen zusätzliche Blöcke beim Bau eines Runs
/// (Flush und Compaction) und führen sie bei der Compaction zusammen.
pub trait Extension: Send + Sync {
    /// Stabile Block-Art (>= 0x1000).
    fn id(&self) -> u16;

    /// Erzeugt den Erweiterungsblock aus den enthaltenen Dateneinträgen.
    fn build(&self, entries: &mut dyn Iterator<Item = &DataEntry>) -> io::Result<Vec<u8>>;

    /// Führt bestehende Erweiterungsblöcke bei der Compaction zusammen.
    fn merge(
        &self,
        inputs: &[&[u8]],
        survivors: &mut dyn Iterator<Item = &DataEntry>,
    ) -> io::Result<Vec<u8>>;
}

/// Hilfsfunktion zur Ermittlung der Directory-Blockart für eine Extension.
pub fn extension_kind(id: u16) -> u16 {
    EXTENSION_BASE.saturating_add(id)
}

/// Prüft, ob eine Blockart ein Erweiterungsblock ist (kind >= 0x1000).
pub const fn is_extension_kind(kind: u16) -> bool {
    kind >= EXTENSION_BASE
}
