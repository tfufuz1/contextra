#![allow(deprecated)]
//! Kanonische IDs, Referenzen und Monotonie-Typen (SPEC 7.2, 7.3, 7.4, 8.1, 8.7, D-08, D-70, I-11, I-12, F.1, F.9, TY-08).

use core::fmt;
use serde::{Deserialize, Serialize};

pub use contextra_ids::*;

/// 16-Byte Lösch-Identifikator (SPEC 7.4, TY-09).
// MIGRATION(S-70): ErasureId gehört in die Erasure-Familie (contextra-erasure)
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[deprecated(
    since = "0.1.0",
    note = "Löschtypen gehören in die Erasure-Familie (contextra-erasure-spi per D-E1)"
)]
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

/// 16-Byte Gate-Kennung für ReadGate-Einträge (SPEC F.9).
// MIGRATION(S-70): GateId gehört in die Erasure-Familie (contextra-erasure)
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[deprecated(
    since = "0.1.0",
    note = "Löschtypen gehören in die Erasure-Familie (contextra-erasure-spi per D-E1)"
)]
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
