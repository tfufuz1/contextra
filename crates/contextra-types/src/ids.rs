//! Kanonische IDs, Referenzen und Monotonie-Typen (SPEC 7.2, 7.3, 7.4, 8.1, 8.7, D-08, D-70, I-11, I-12, F.1, F.9, TY-08).

use core::fmt;
use serde::{Deserialize, Serialize};

use crate::error::{ContextraError, Result};

/// Maximale Bytelänge für Referenz-Strings / Byte-Identifier.
pub const MAX_REF_LEN: usize = 256;

macro_rules! define_ref_newtype {
    ($name:ident, $doc:expr) => {
        #[doc = $doc]
        #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        pub struct $name(Vec<u8>);

        impl $name {
            /// Erzeugt eine geprüfte Instanz. Verstoß führt zu `ContextraError::InvalidInput`.
            pub fn try_new(bytes: Vec<u8>) -> Result<Self> {
                if bytes.is_empty() {
                    return Err(ContextraError::InvalidInput("empty identifier".into()));
                }
                if bytes.len() > MAX_REF_LEN {
                    return Err(ContextraError::InvalidInput("identifier too long".into()));
                }
                Ok(Self(bytes))
            }

            /// Liefert die Bytes als Slice.
            #[inline]
            pub fn as_bytes(&self) -> &[u8] {
                &self.0
            }

            /// Verbraucht den Wrapper und liefert den Vektor.
            #[inline]
            pub fn into_inner(self) -> Vec<u8> {
                self.0
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(
                    f,
                    "{}([REDACTED {} bytes])",
                    stringify!($name),
                    self.0.len()
                )
            }
        }
    };
}

define_ref_newtype!(TenantRef, "Mandanten-Referenz (SPEC 7.3).");
define_ref_newtype!(CollectionRef, "Collection-Referenz (SPEC 7.3).");
define_ref_newtype!(SubjectRef, "Betroffenen-Referenz (SPEC 7.2, 7.3).");
define_ref_newtype!(OperatorRef, "Opake Betreiberkennung (SPEC 7.3).");
define_ref_newtype!(DocKey, "Dokumentenschlüssel (SPEC 7.2).");

/// Eindeutige Dokumentenadresse: Betroffener ist unveränderlicher Teil der Identität (SPEC 7.2, D-70).
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub struct DocRef {
    /// Optionaler Betroffener (Subject).
    pub subject: Option<SubjectRef>,
    /// Eindeutiger Dokumentenschlüssel innerhalb des Clusters.
    pub key: DocKey,
}

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
