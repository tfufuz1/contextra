//! Schema Versioning structures and utilities for Contextra SSTable/WAL Manifest transitions (ADR-082).
//!
//! # Architektur-Rolle
//! Ring 0 (Kern). Keine I/O- oder Asynchron-Abhängigkeiten.
//!
//! # Zweck
//! Stellt Versionsidentifikatoren für Manifestdateien von SSTables (Sorted String Tables) und
//! WALs (Write-Ahead Logs) sowie die DocId-Bitbreite bereit.

use crate::error::{ContextraError, Result};
use serde::{Deserialize, Serialize};

// ANCHOR[contextra-types:SCHEMA-001]
/// Indicates the DocId bit-width expected or produced by a schema version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum DocIdWidth {
    /// 64-bit DocId format (8-byte BLAKE3 truncation).
    Bit64 = 64,
    /// 128-bit DocId format (16-byte BLAKE3 truncation, ADR-082).
    Bit128 = 128,
}

impl DocIdWidth {
    // ANCHOR[contextra-types:SCHEMA-002]
    /// Returns the active compile-time `DocIdWidth`.
    ///
    /// # Ablauf
    /// Prüft zur Compile-Zeit via `cfg!(feature = "docid-128")`, welche Breite gewählt ist.
    pub fn current() -> Self {
        if cfg!(feature = "docid-128") {
            Self::Bit128
        } else {
            Self::Bit64
        }
    }

    // ANCHOR[contextra-types:SCHEMA-003]
    /// Returns the size in bytes of a single `DocId` value under this width.
    pub fn bytes_len(self) -> usize {
        match self {
            Self::Bit64 => 8,
            Self::Bit128 => 16,
        }
    }
}

// ANCHOR[contextra-types:SCHEMA-004]
/// SSTable and WAL Manifest schema version identifier.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
#[repr(u8)]
pub enum ManifestSchemaVersion {
    /// Schema Version 1 (v0.x legacy 64-bit DocId layout).
    #[default]
    V1 = 1,
    /// Schema Version 2 (ADR-082 128-bit DocId layout).
    V2 = 2,
}

impl ManifestSchemaVersion {
    // ANCHOR[contextra-types:SCHEMA-005]
    /// Returns the corresponding `DocIdWidth` for this schema version.
    pub fn doc_id_width(self) -> DocIdWidth {
        match self {
            Self::V1 => DocIdWidth::Bit64,
            Self::V2 => DocIdWidth::Bit128,
        }
    }

    // ANCHOR[contextra-types:SCHEMA-006]
    /// Returns the numeric version byte.
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    // ANCHOR[contextra-types:SCHEMA-007]
    /// Parses a version from a numeric byte indicator.
    ///
    /// # Fehler
    /// Liefert `ContextraError::Serialization`, falls das übergebene Byte keiner bekannten Version entspricht.
    pub fn from_u8(version_byte: u8) -> Result<Self> {
        match version_byte {
            1 => Ok(Self::V1),
            2 => Ok(Self::V2),
            other => Err(ContextraError::Serialization(format!(
                "Unsupported Manifest schema version byte: {other}"
            ))),
        }
    }

    // ANCHOR[contextra-types:SCHEMA-008]
    /// Returns `true` if this schema version is compatible with the compile-time DocId feature flags.
    pub fn is_compatible_with_current_build(self) -> bool {
        self.doc_id_width() == DocIdWidth::current()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    // NOTE: Prüft Konvertierung zwischen Byte und ManifestSchemaVersion sowie Fehler bei unbekannten Bytes.
    #[test]
    fn test_manifest_schema_version_roundtrip() {
        assert_eq!(
            ManifestSchemaVersion::from_u8(1).unwrap(),
            ManifestSchemaVersion::V1
        );
        assert_eq!(
            ManifestSchemaVersion::from_u8(2).unwrap(),
            ManifestSchemaVersion::V2
        );
        assert!(ManifestSchemaVersion::from_u8(3).is_err());
    }

    // NOTE: Prüft Bytegrößen von 64-Bit und 128-Bit DocIdWidth.
    #[test]
    fn test_doc_id_width_bytes_len() {
        assert_eq!(DocIdWidth::Bit64.bytes_len(), 8);
        assert_eq!(DocIdWidth::Bit128.bytes_len(), 16);
    }
}
