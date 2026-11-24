//! Domänen- und Identifikationstypen für `contextra-kvcache`.

use serde::{Deserialize, Serialize};

/// Mandanten-Identifier (lokaler Typ).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TenantId(pub u64);

impl TenantId {
    /// Reservierter System-Mandant.
    pub const SYSTEM: Self = Self(0);

    /// Erzeugt eine neue `TenantId`.
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Validierte Erzeugung (Mandant 0 ist reserviert/ungültig für reguläre Mandanten).
    pub fn try_new(id: u64) -> Result<Self, crate::error::KvCacheError> {
        if id == 0 {
            Err(crate::error::KvCacheError::InvalidTenant("0 ist kein gültiger Mandant".into()))
        } else {
            Ok(Self(id))
        }
    }

    /// Gibt den inneren raw `u64`-Wert zurück.
    pub const fn inner(self) -> u64 {
        self.0
    }

    /// Gibt den numerischen Wert zurück.
    pub const fn as_u64(&self) -> u64 {
        self.0
    }

    /// Gibt an, ob es sich um den System-Mandanten handelt.
    pub const fn is_system(&self) -> bool {
        self.0 == 0
    }
}

impl std::fmt::Display for TenantId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u64> for TenantId {
    fn from(id: u64) -> Self {
        Self(id)
    }
}

/// Transaktions-Identifier (lokaler Newtype für Speicheroperationen).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TxId(pub u64);

impl TxId {
    /// Erzeugt eine neue `TxId`.
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Gibt den inneren raw `u64`-Wert zurück.
    pub const fn inner(self) -> u64 {
        self.0
    }

    /// Gibt den numerischen Wert zurück.
    pub const fn as_u64(&self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for TxId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TxId({})", self.0)
    }
}

impl From<u64> for TxId {
    fn from(id: u64) -> Self {
        Self(id)
    }
}

/// Sequenzschritt / Token-Offset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct StepId(pub u64);

impl StepId {
    /// Erzeugt einen neuen `StepId`.
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Gibt den inneren raw `u64`-Wert zurück.
    pub const fn inner(self) -> u64 {
        self.0
    }

    /// Gibt den numerischen Wert zurück.
    pub const fn as_u64(&self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for StepId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "StepId({})", self.0)
    }
}

impl From<u64> for StepId {
    fn from(id: u64) -> Self {
        Self(id)
    }
}

/// Eindeutige ID einer Inferenzanfrage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RequestId(pub u64);

impl RequestId {
    /// Erzeugt eine `RequestId` aus einem `u64`.
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Gibt den inneren `u64`-Wert zurück.
    pub const fn inner(self) -> u64 {
        self.0
    }

    /// Gibt den inneren `u64`-Wert zurück.
    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for RequestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RequestId({})", self.0)
    }
}

impl From<u64> for RequestId {
    fn from(id: u64) -> Self {
        Self(id)
    }
}

/// Fingerabdruck eines Modells (32-Byte Hash).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ModelFingerprint(pub [u8; 32]);

impl ModelFingerprint {
    /// Erzeugt einen ModelFingerprint aus 32 Bytes.
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Erzeugt einen ModelFingerprint aus einem String (BLAKE3 gehasht).
    pub fn from_str_hash(s: &str) -> Self {
        let hash = blake3::hash(s.as_bytes());
        Self(*hash.as_bytes())
    }

    /// Gibt die 32 Bytes zurück.
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// Cache-Direktive für granulare Lebenszyklus-Steuerung.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CacheDirective {
    /// Block soll dauerhaft oder für eine bestimmte TTL im Cache gepinnt bleiben.
    Pin {
        /// Optionale maximale Dauer für die Pinning-Gültigkeit.
        ttl: Option<std::time::Duration>,
    },
    /// Niemals im Speicher cachen (sofort verwerfen).
    NeverCache,
    /// Automatische Verdrängung (Standard LRU / Attention-basiert).
    Auto,
    /// Transientes Schritt-Segment, das nach Abschluss des Schritts freigegeben wird.
    ReleaseAfterStep {
        /// Schritt-ID nach dessen Abschluss dieses Segment freigegeben wird.
        step_id: StepId,
    },
}

impl Default for CacheDirective {
    fn default() -> Self {
        Self::Auto
    }
}
