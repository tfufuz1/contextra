//! Dateitypen und Header-Konstanten (SPEC Anhang F.1, TY-15).

use crate::error::Error;
use serde::{Deserialize, Serialize};

/// Format-Version des Speicherframeworks (SPEC F.1).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub struct FormatVersion(pub u16);

impl FormatVersion {
    /// Standard-Formatversion v1 (SPEC F.1).
    pub const V1: Self = Self(1);
    /// Aktuelle Standard-Formatversion.
    pub const CURRENT: Self = Self(1);
}

/// Algorithmus-IDs eines Dateikopfs (SPEC F.1: `u8[4] = [aead, kdf_or_0, mac, hash]; v1: [1, 0, 1, 1]`).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct AlgIds(pub [u8; 4]);

impl AlgIds {
    /// Standard-Algorithmen-IDs für v1 (SPEC F.1).
    pub const V1: Self = Self([1, 0, 1, 1]);
}

/// Dateitypen laut SPEC Anhang F.1 (Werte 1..=10, TY-15).
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FileKind {
    /// 1: WAL-Segment.
    Wal = 1,
    /// 2: SSTable-Datendatei.
    SSTable = 2,
    /// 3: Manifest-Log.
    Manifest = 3,
    /// 4: Textsegment.
    TextSegment = 4,
    /// 5: Vektor-Snapshot.
    VectorSnapshot = 5,
    /// 6: Journal.
    Journal = 6,
    /// 7: Gate-Log.
    GateLog = 7,
    /// 8: Ledger.
    Ledger = 8,
    /// 9: Scope-Datei.
    Scope = 9,
    /// 10: Tresor.
    Vault = 10,
}

impl FileKind {
    /// Liefert den numerischen U8-Code des Dateityps nach SPEC F.1.
    #[inline]
    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    /// Erwarteter key_mode nach SPEC F.1 (0 = zufälliger Dateischlüssel, 1 = abgeleitet).
    #[inline]
    pub const fn key_mode(self) -> u8 {
        match self {
            Self::Wal | Self::SSTable | Self::TextSegment | Self::VectorSnapshot | Self::Scope => 0,
            Self::Manifest | Self::Journal | Self::GateLog | Self::Ledger | Self::Vault => 1,
        }
    }
}

impl TryFrom<u8> for FileKind {
    type Error = Error;

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            1 => Ok(Self::Wal),
            2 => Ok(Self::SSTable),
            3 => Ok(Self::Manifest),
            4 => Ok(Self::TextSegment),
            5 => Ok(Self::VectorSnapshot),
            6 => Ok(Self::Journal),
            7 => Ok(Self::GateLog),
            8 => Ok(Self::Ledger),
            9 => Ok(Self::Scope),
            10 => Ok(Self::Vault),
            _ => Err(Error::Corrupt),
        }
    }
}
