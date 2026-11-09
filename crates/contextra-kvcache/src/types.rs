//! Domänen- und Identifikationstypen für `contextra-kvcache`.

use crate::error::{KvCacheError, Result};
use serde::{Deserialize, Serialize};

/// Mandanten-Identifier für vollständige Cache-Isolation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TenantId(pub u64);

impl TenantId {
    /// System-Mandant.
    pub const SYSTEM: Self = Self(0);

    /// Erzeugt einen neuen `TenantId` oder gibt einen Fehler zurück, falls 0.
    pub fn try_new(id: u64) -> Result<Self> {
        if id == 0 {
            Err(KvCacheError::InvalidTenant(
                "TenantId darf nicht 0 sein".into(),
            ))
        } else {
            Ok(Self(id))
        }
    }

    /// Erzeugt einen `TenantId` direkt aus einem `u64`.
    pub const fn from_u64(id: u64) -> Self {
        Self(id)
    }

    /// Gibt den inneren `u64`-Wert zurück.
    pub const fn as_u64(&self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for TenantId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Tenant({})", self.0)
    }
}

/// Sequenzschritt / Token-Offset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct StepId(pub u64);

impl StepId {
    /// Erzeugt einen neuen StepId.
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Gibt den numerischen Wert zurück.
    pub const fn as_u64(&self) -> u64 {
        self.0
    }
}

/// Eindeutige ID einer Inferenzanfrage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RequestId(pub [u8; 16]);

impl RequestId {
    /// Erzeugt eine RequestId aus 16 Bytes.
    pub const fn from_bytes(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }

    /// Erzeugt eine RequestId aus einem `u128`.
    pub const fn from_u128(id: u128) -> Self {
        Self(id.to_be_bytes())
    }

    /// Gibt die Bytes zurück.
    pub const fn as_bytes(&self) -> &[u8; 16] {
        &self.0
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CacheDirective {
    /// Automatische Verdrängung (Standard LRU / Attention-basiert).
    Auto,
    /// Block soll dauerhaft oder für eine bestimmte TTL im Cache gepinnt bleiben.
    Pin {
        /// Optionale Time-to-Live in Millisekunden.
        ttl_ms: Option<u64>,
    },
    /// Block nach Inferenzschritt sofort freigeben.
    ReleaseAfterStep,
    /// Niemals im Speicher cachen (sofort verwerfen).
    NeverCache,
}
