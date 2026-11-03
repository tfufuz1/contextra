//! Feldmessung der Klassen MK1, MK2 und MK3 (SPEC §12.1, D-35, D-72, ER-26).
//!
//! MK1: Logische/funktionale Prüfung – kein Eintrag in Strukturen des Datenverzeichnisses.
//! MK2: Rohscan inklusive tmp/, alte Segmente, WAL, Snapshots, scope/, Berichte.
//! MK3: Schlüsselabwesenheit – Slot der höchsten Ebene genullt, DEK-Ableitung scheitert.

use contextra_types::erasure::{LayerWitness, MeasureClass};

/// Ergebnis einer MK1-Feldmessung (logische Prüfung).
#[derive(Debug)]
pub struct Mk1Result {
    /// Anzahl geprüfter Strukturen.
    pub checked_structures: u64,
    /// Treffer (sollte 0 sein).
    pub matches: u64,
}

/// Ergebnis einer MK2-Feldmessung (Rohscan).
#[derive(Debug)]
pub struct Mk2Result {
    /// Anzahl geprüfter Dateien.
    pub checked_files: u64,
    /// Geprüfte Bytes.
    pub checked_bytes: u64,
    /// Treffer (sollte 0 sein).
    pub matches: u64,
}

/// Ergebnis einer MK3-Feldmessung (Schlüsselabwesenheit).
#[derive(Debug)]
pub struct Mk3Result {
    /// Slot der höchsten Ebene genullt.
    pub top_slot_zeroed: bool,
    /// DEK-Ableitung gescheitert.
    pub dek_derivation_fails: bool,
    /// F-Slots der Zieldateien nicht mehr vorhanden.
    pub file_slots_absent: bool,
}

impl Mk3Result {
    /// Prüft ob MK3 vollständig bestanden (kein Schlüssel mehr ableitbar).
    pub fn is_clean(&self) -> bool {
        self.top_slot_zeroed && self.dek_derivation_fails && self.file_slots_absent
    }
}
