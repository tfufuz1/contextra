//! Dateitypen und Header-Konstanten (SPEC Anhang F.1, TY-15).

use crate::error::Error;
use serde::{Deserialize, Serialize};

/// Format-Version des Speicherframeworks (SPEC F.1).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub struct FormatVersion(pub u16);

impl FormatVersion {
    /// Aktuelle Standard-Formatversion (Version 2).
    pub const CURRENT: Self = Self(2);
}

/// Algorithmus-IDs eines Dateikopfs (SPEC F.1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct AlgIds(pub [u8; 4]);

/// Dateitypen laut SPEC Anhang F.1 (Werte 1..=10, TY-15).
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FileKind {
    /// 1: Keyslot-Headerdatei (A/B).
    Header = 1,
    /// 2: Write-Ahead-Log-Segment.
    Wal = 2,
    /// 3: File-Vault (Tresor für Dateischlüssel).
    Vault = 3,
    /// 4: Manifest-Segment.
    Manifest = 4,
    /// 5: SSTable-Datendatei (Level 0..N).
    SSTable = 5,
    /// 6: Textindex-Segment.
    TextSegment = 6,
    /// 7: Vektorindex-Snapshot.
    VectorSnapshot = 7,
    /// 8: Gate-Log-Segment.
    GateLog = 8,
    /// 9: Journal-Transaktionsdatei.
    Journal = 9,
    /// 10: Ledger-Auditkette.
    Ledger = 10,
}

impl TryFrom<u8> for FileKind {
    type Error = Error;

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            1 => Ok(Self::Header),
            2 => Ok(Self::Wal),
            3 => Ok(Self::Vault),
            4 => Ok(Self::Manifest),
            5 => Ok(Self::SSTable),
            6 => Ok(Self::TextSegment),
            7 => Ok(Self::VectorSnapshot),
            8 => Ok(Self::GateLog),
            9 => Ok(Self::Journal),
            10 => Ok(Self::Ledger),
            _ => Err(Error::Corrupt),
        }
    }
}
