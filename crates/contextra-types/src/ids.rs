//! Kanonische IDs und Monotonie-Typen (SPEC 7.4, 8.7, I-12, F.1, F.9, TY-08).

use core::fmt;
use serde::{Deserialize, Serialize};

/// 16-Byte Lösch-Identifikator (SPEC 7.4, TY-09).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ErasureId(pub [u8; 16]);

impl ErasureId {
    /// Erzeugt eine `ErasureId` aus rohen 16 Bytes.
    pub const fn new(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }

    /// Liefert die 16 Bytes als Slice.
    pub const fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }
}

impl fmt::Debug for ErasureId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ErasureId({:02x}{:02x}{:02x}{:02x}..)",
            self.0[0], self.0[1], self.0[2], self.0[3]
        )
    }
}

/// 16-Byte Dateikennung (SPEC F.1).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct FileId(pub [u8; 16]);

impl FileId {
    /// Erzeugt eine `FileId` aus rohen 16 Bytes.
    pub const fn new(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }

    /// Liefert die 16 Bytes als Slice.
    pub const fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }
}

impl fmt::Debug for FileId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "FileId({:02x}{:02x}{:02x}{:02x}..)",
            self.0[0], self.0[1], self.0[2], self.0[3]
        )
    }
}

/// 16-Byte Speicherinstanz-Kennung (SPEC F.1).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct StoreId(pub [u8; 16]);

impl StoreId {
    /// Erzeugt eine `StoreId` aus rohen 16 Bytes.
    pub const fn new(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }

    /// Liefert die 16 Bytes als Slice.
    pub const fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }
}

impl fmt::Debug for StoreId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "StoreId({:02x}{:02x}{:02x}{:02x}..)",
            self.0[0], self.0[1], self.0[2], self.0[3]
        )
    }
}

/// 16-Byte Gate-Kennung für ReadGate-Einträge (SPEC F.9).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct GateId(pub [u8; 16]);

impl GateId {
    /// Erzeugt eine `GateId` aus rohen 16 Bytes.
    pub const fn new(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }

    /// Liefert die 16 Bytes als Slice.
    pub const fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }
}

impl fmt::Debug for GateId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "GateId({:02x}{:02x}{:02x}{:02x}..)",
            self.0[0], self.0[1], self.0[2], self.0[3]
        )
    }
}

/// Streng monoton steigende Sequenznummer (SPEC 8.7, I-12, TY-08).
/// Kein Sentinelwert und kein u64::MAX-Pfad; `Option<Seq>` modelliert „kein Snapshot“.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub struct Seq(pub u64);

impl Seq {
    /// Erzeugt eine neue Sequenznummer.
    pub const fn new(val: u64) -> Self {
        Self(val)
    }

    /// Liefert den numerischen Wert als u64.
    pub const fn as_u64(&self) -> u64 {
        self.0
    }
}

/// Mikrosekunden-Zeitstempel in UTC (SPEC 7.4).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub struct Timestamp(pub u64);

impl Timestamp {
    /// Erzeugt einen neuen Zeitstempel aus Mikrosekunden seit Unix-Epoche.
    pub const fn from_micros(micros: u64) -> Self {
        Self(micros)
    }

    /// Liefert den Mikrosekunden-Wert.
    pub const fn as_micros(&self) -> u64 {
        self.0
    }
}
