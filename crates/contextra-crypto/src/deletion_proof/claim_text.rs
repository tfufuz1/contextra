//! Geschlossenes ClaimText-Enum und Assurance-Stufen (S0..S4) für Deletion Witnesses (C7).

#![forbid(unsafe_code)]

use serde::Serialize;

/// Assurance-Stufen S0 bis S4 für den Deletion Witness.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[repr(u8)]
pub enum WitAssurance {
    /// S0: Basic Tombstone Committal.
    S0 = 0,
    /// S1: RAM Cleaned & Vector Zeroed.
    S1 = 1,
    /// S2: Persistent SSTable Value/Key Cleaned.
    S2 = 2,
    /// S3: WAL Segments Cleaned.
    S3 = 3,
    /// S4: Key Destroyed / Crypto Erased.
    S4 = 4,
}

impl WitAssurance {
    /// Gibt den numerischen Level (0..=4) der Assurance-Stufe zurück.
    pub fn level(&self) -> u8 {
        *self as u8
    }

    /// Konvertiert einen u8-Level in eine `WitAssurance`.
    pub fn from_level(level: u8) -> Option<Self> {
        match level {
            0 => Some(Self::S0),
            1 => Some(Self::S1),
            2 => Some(Self::S2),
            3 => Some(Self::S3),
            4 => Some(Self::S4),
            _ => None,
        }
    }
}

/// Geschlossenes ClaimText-Enum mit repr(u16)-stabilen IDs gemäß Spezifikation C7.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[repr(u16)]
pub enum WitClaimText {
    /// S0: Tombstones committed; Daten über die Contextra-API nicht mehr les-/findbar.
    S0 = 1,
    /// S1: + RAM-Indizes (Vektor, Text, Relationen) bereinigt; Vektorkopien im Indexspeicher genullt.
    S1 = 2,
    /// S2V: Keine Wertversionen in SSTable-/Manifest-Dateien; Tombstones können verbleiben.
    S2V = 3,
    /// S2K: Keine Werte und keine Schlüsselbezeichner in SSTable-/Manifest-Dateien.
    S2K = 4,
    /// S3: Keine passenden Records in WAL-Segmenten.
    S3 = 5,
    /// S4: Schlüssel vernichtet; Ciphertexte nicht mehr entschlüsselbar.
    S4 = 6,
    /// RamUnreferenced: Cache-/RAM-Einträge nicht mehr referenziert.
    RamUnreferenced = 7,
    /// RamZeroized: Cache-/RAM-Einträge überschrieben.
    RamZeroized = 8,
}

impl WitClaimText {
    /// Gibt die stabile u16 ID des Claims zurück.
    pub fn id(&self) -> u16 {
        *self as u16
    }

    /// Dekodiert eine u16 ID in ein `WitClaimText`.
    pub fn from_id(id: u16) -> Option<Self> {
        match id {
            1 => Some(Self::S0),
            2 => Some(Self::S1),
            3 => Some(Self::S2V),
            4 => Some(Self::S2K),
            5 => Some(Self::S3),
            6 => Some(Self::S4),
            7 => Some(Self::RamUnreferenced),
            8 => Some(Self::RamZeroized),
            _ => None,
        }
    }

    /// Gibt die exakte deutsche Zeichenkette gemäß Spezifikation C7 zurück.
    pub fn text(&self) -> &'static str {
        match self {
            Self::S0 => "Tombstones committed; Daten über die Contextra-API nicht mehr les-/findbar",
            Self::S1 => "+ RAM-Indizes (Vektor, Text, Relationen) bereinigt; Vektorkopien im Indexspeicher genullt",
            Self::S2V => "Keine Wertversionen in SSTable-/Manifest-Dateien; Tombstones können verbleiben",
            Self::S2K => "Keine Werte und keine Schlüsselbezeichner in SSTable-/Manifest-Dateien",
            Self::S3 => "Keine passenden Records in WAL-Segmenten",
            Self::S4 => "Schlüssel vernichtet; Ciphertexte nicht mehr entschlüsselbar",
            Self::RamUnreferenced => "Cache-/RAM-Einträge nicht mehr referenziert",
            Self::RamZeroized => "Cache-/RAM-Einträge überschrieben",
        }
    }

    /// Einzige Quelle für Doc-Truth-Prüfungen.
    pub fn all() -> &'static [Self] {
        &[
            Self::S0,
            Self::S1,
            Self::S2V,
            Self::S2K,
            Self::S3,
            Self::S4,
            Self::RamUnreferenced,
            Self::RamZeroized,
        ]
    }
}
