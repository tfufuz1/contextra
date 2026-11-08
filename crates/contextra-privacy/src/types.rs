//! `types` — Lokale Typ- und Traitabstraktionen für Zeit, Mandantenisolation und Egress-Policy (Klasse A).
//!
//! # Zweck
//! Stellt sicher, dass `contextra-privacy` als autarker Baustein ohne interne Abhängigkeiten
//! zu `contextra-ports` oder `contextra-types` funktioniert (SPEC v4.0 §4, B1–B4).
//! Bei aktiviertem transitionalen Feature `ports-compat` stehen Konvertierungen zu den
//! Workspace-Typen bereit.

use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

/// Trait für monotone und reale Zeitabfragen ohne Abhängigkeit von `contextra-ports`.
pub trait Clock: Send + Sync {
    /// Liefert die aktuelle Zeit in Unix-Nanosekunden.
    fn now_unix_nanos(&self) -> u64;
    /// Liefert einen monotonen Nanosekunden-Zähler.
    fn monotonic_nanos(&self) -> u64;
}

/// Standardmäßige Implementierung von [`Clock`] auf Basis von `std::time::SystemTime`.
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemClock;

impl SystemClock {
    /// Erstellt eine neue [`SystemClock`].
    pub fn new() -> Self {
        Self
    }
}

impl Clock for SystemClock {
    #[allow(clippy::disallowed_methods, clippy::cast_possible_truncation)]
    fn now_unix_nanos(&self) -> u64 {
        SystemTime::now() // ALLOW-DETERMINISM: Wall clock baseline in standalone SystemClock
            .duration_since(UNIX_EPOCH)
            .map(|d| u64::try_from(d.as_nanos()).unwrap_or(u64::MAX))
            .unwrap_or(0)
    }

    fn monotonic_nanos(&self) -> u64 {
        self.now_unix_nanos()
    }
}

#[cfg(feature = "ports-compat")]
impl<T: contextra_ports::Clock + ?Sized> Clock for T {
    fn now_unix_nanos(&self) -> u64 {
        contextra_ports::Clock::now_unix_nanos(self)
    }

    fn monotonic_nanos(&self) -> u64 {
        contextra_ports::Clock::monotonic_nanos(self)
    }
}

/// Numerischer Mandanten-Identifikator (64-Bit).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[repr(transparent)]
pub struct TenantId(pub u64);

impl TenantId {
    /// Systemmandant (0).
    pub const SYSTEM: Self = Self(0);

    /// Erstellt einen neuen `TenantId`.
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Versucht, einen Mandanten zu erzeugen (0 ist reserviert/Fehler für Benutzerdaten).
    pub fn try_new(id: u64) -> Result<Self, TenantScopeViolation> {
        if id == 0 {
            Err(TenantScopeViolation::InvalidZero)
        } else {
            Ok(Self(id))
        }
    }

    /// Liefert den numerischen Wert.
    pub const fn as_u64(&self) -> u64 {
        self.0
    }
}

impl fmt::Display for TenantId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TenantId({})", self.0)
    }
}

impl From<u64> for TenantId {
    fn from(val: u64) -> Self {
        Self(val)
    }
}

/// Fehler bei Mandanten-Scope-Verletzungen.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TenantScopeViolation {
    /// Mandanten-Scope-Ungleichheit.
    #[error("tenant scope mismatch: expected {expected:?}, got {actual:?}")]
    Mismatch {
        /// Erwarteter Mandant.
        expected: TenantId,
        /// Tatsächlicher Mandant.
        actual: TenantId,
    },
    /// Ungültiger Null-Mandant oder unzulässige Operation.
    #[error("tenant ID cannot be 0")]
    InvalidZero,
}

/// An einen Mandanten gebundener Datenbehälter zur Durchsetzung von Isolation.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TenantScoped<T> {
    tenant_id: TenantId,
    data: T,
}

impl<T> TenantScoped<T> {
    /// Erstellt einen neuen `TenantScoped`-Container.
    pub const fn new(tenant_id: TenantId, data: T) -> Self {
        Self { tenant_id, data }
    }

    /// Liefert die gebundene Mandanten-ID.
    pub const fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }

    /// Gibt eine Referenz auf die inneren Daten zurück.
    pub const fn inner(&self) -> &T {
        &self.data
    }

    /// Prüft den Mandanten und entpackt die inneren Daten.
    pub fn into_inner_checked(self, expected: &TenantId) -> Result<T, TenantScopeViolation> {
        if self.tenant_id == *expected {
            Ok(self.data)
        } else {
            Err(TenantScopeViolation::Mismatch {
                expected: *expected,
                actual: self.tenant_id,
            })
        }
    }
}

/// Entscheidung einer Egress-Richtlinienauswertung.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EgressDecision {
    /// Anfrage ohne Modifikation gestattet.
    Allow,
    /// Anfrage gestattet, aber Payload muss bereinigt / maskiert werden.
    Redact(String),
    /// Anfrage verweigert.
    Deny(String),
}

/// Crate-eigener dynamischer Trait für Egress-Richtlinien.
pub trait EgressPolicy: Send + Sync {
    /// Bewertet, ob der ausgehende Payload die Richtlinie erfüllt.
    fn evaluate(&self, payload: &str) -> EgressDecision;
}

#[cfg(feature = "ports-compat")]
mod compat {
    use super::*;

    impl From<TenantId> for contextra_types::TenantId {
        fn from(t: TenantId) -> Self {
            contextra_types::TenantId(t.0)
        }
    }

    impl From<contextra_types::TenantId> for TenantId {
        fn from(t: contextra_types::TenantId) -> Self {
            TenantId(t.0)
        }
    }

    impl From<TenantScopeViolation> for contextra_types::TenantScopeViolation {
        fn from(v: TenantScopeViolation) -> Self {
            match v {
                TenantScopeViolation::Mismatch { expected, actual } => {
                    contextra_types::TenantScopeViolation::Mismatch {
                        expected: expected.into(),
                        actual: actual.into(),
                    }
                }
                TenantScopeViolation::InvalidZero => {
                    contextra_types::TenantScopeViolation::Mismatch {
                        expected: contextra_types::TenantId(0),
                        actual: contextra_types::TenantId(0),
                    }
                }
            }
        }
    }

    impl From<contextra_types::TenantScopeViolation> for TenantScopeViolation {
        fn from(v: contextra_types::TenantScopeViolation) -> Self {
            match v {
                contextra_types::TenantScopeViolation::Mismatch { expected, actual } => {
                    TenantScopeViolation::Mismatch {
                        expected: expected.into(),
                        actual: actual.into(),
                    }
                }
            }
        }
    }

    impl<T> From<TenantScoped<T>> for contextra_types::TenantScoped<T> {
        fn from(s: TenantScoped<T>) -> Self {
            contextra_types::TenantScoped::new(s.tenant_id.into(), s.data)
        }
    }
}
