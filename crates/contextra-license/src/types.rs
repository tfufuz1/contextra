//! # Crate-lokale Kern-Typen und Traits für die Lizenzprüfung (§14.6, §15, SPEC v4.0).
//!
//! Stellt fundamentale Schnittstellen und Datenstrukturen zur Verfügung, die für eine
//! autarke Funktionsweise (Klasse A, B1–B12) ohne Abhängigkeit von `contextra-ports`
//! oder `contextra-types` erforderlich sind.

use serde::{Deserialize, Serialize};

// ANCHOR[contextra-license:TYPE-FEATURE-RING]
/// Einteilung der Contextra-Funktionsringe (§14.6).
///
/// Definiert die Sicherheits- und Isolationsstufen von Contextra-Instanzen.
/// Der freie `Fast`-Ring steht ohne Lizenz zur Verfügung (INV-LICENSE-2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum FeatureRing {
    /// Fast-Ring: Freie Open-Source-Kernfunktionen ohne Lizenzprüfung.
    Fast,
    /// Sovereign-Ring: Erweiterte Funktionen zur Datensouveränität (Air-Gapped, On-Premise).
    Sovereign,
    /// Compliance-Ring: Strenge Auditierbarkeit, Löschketten und regulatorische Garantien.
    Compliance,
}

impl std::fmt::Display for FeatureRing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Fast => write!(f, "Fast"),
            Self::Sovereign => write!(f, "Sovereign"),
            Self::Compliance => write!(f, "Compliance"),
        }
    }
}

// ANCHOR[contextra-license:TYPE-AUTHORIZED-RING]
/// Bestätigungstoken für einen erfolgreich autorisierten Feature-Ring (§15).
///
/// Kann nur durch erfolgreiches Durchlaufen eines [`LicenseGate`] erzeugt werden.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuthorizedRing {
    ring: FeatureRing,
}

impl AuthorizedRing {
    /// Erzeugt ein neues Autorisierungs-Token für den angegebenen Ring.
    pub fn new(ring: FeatureRing) -> Self {
        Self { ring }
    }

    /// Gibt den autorisierten [`FeatureRing`] zurück.
    pub fn ring(&self) -> FeatureRing {
        self.ring
    }
}

// ANCHOR[contextra-license:TYPE-TENANT-ID]
/// Kanonischer 64-Bit-Ganzzahl-Mandantenidentifikator (SPEC §2, B3).
///
/// Zur Gewährleistung binärer Kompatibilität mit gespeicherten Lizenznutzlasten
/// als transparenter `u64`-Wrapper modelliert.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub struct TenantId(pub u64);

impl TenantId {
    /// SYSTEM-Mandant (`0`).
    pub const SYSTEM: Self = Self(0);
    /// Impliziter Standard-Mandant.
    pub const DEFAULT: Self = Self(0);

    /// Erzeugt eine neue `TenantId`.
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Liefert den numerischen Wert als `u64`.
    pub const fn as_u64(&self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for TenantId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "tenant-{}", self.0)
    }
}

impl From<u64> for TenantId {
    fn from(val: u64) -> Self {
        Self(val)
    }
}

impl From<TenantId> for u64 {
    fn from(id: TenantId) -> Self {
        id.0
    }
}

// ANCHOR[contextra-license:TYPE-ERROR]
/// Typisierter Fehler für Lizenz- und Aktivierungsprüfungen (B4, SPEC §4.1).
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum LicenseError {
    /// Kein gültiger Aktivierungsdatensatz für den angeforderten Feature-Ring vorhanden.
    #[error("no valid activation found for feature ring {0:?}")]
    NotActivated(FeatureRing),
    /// Die kryptografische Ed25519-Signaturprüfung ist fehlgeschlagen.
    #[error("activation signature verification failed")]
    InvalidSignature,
    /// Der Aktivierungsdatensatz ist zum angegebenen Unix-Zeitstempel (Sekunden) abgelaufen.
    #[error("activation expired at {0}")]
    Expired(i64),
}

/// Kanonischer Result-Typ für das `contextra-license`-Crate.
pub type Result<T> = core::result::Result<T, LicenseError>;

// ANCHOR[contextra-license:TRAIT-CLOCK]
/// Crate-lokaler Trait für deterministische Zeitabfragen (B11, P28).
///
/// Verhindert direkte und nicht-deterministische Betriebssystem-Zeitaufrufe im Kern.
pub trait Clock: Send + Sync {
    /// Liefert den aktuellen Unix-Zeitstempel in Nanosekunden.
    fn now_unix_nanos(&self) -> u64;

    /// Optionaler monotoner Zähler für Kompatibilität mit Port-Uhren.
    fn monotonic_nanos(&self) -> u64 {
        0
    }
}

/// Funktions-Wrapper für `Clock` aus einer Closure (B11).
pub struct FnClock<F>(pub F);

impl<F> Clock for FnClock<F>
where
    F: Fn() -> u64 + Send + Sync,
{
    fn now_unix_nanos(&self) -> u64 {
        (self.0)()
    }
}

// ANCHOR[contextra-license:STRUCT-SYSTEM-CLOCK]
/// Standard-Wanduhr basierend auf der Systemzeit für produktive Umgebungen.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SystemClock;

impl SystemClock {
    /// Erzeugt eine neue Instanz von `SystemClock`.
    pub fn new() -> Self {
        Self
    }
}

impl Clock for SystemClock {
    #[allow(clippy::disallowed_methods, clippy::cast_possible_truncation)]
    fn now_unix_nanos(&self) -> u64 {
        std::time::SystemTime::now() // ALLOW-DETERMINISM: Wall-clock time provider (P28 exception for SystemClock)
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0)
    }
}

// ANCHOR[contextra-license:STRUCT-FIXED-CLOCK]
/// Feste Test-Uhr für deterministische Tests und Benchmarks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FixedClock(pub u64);

impl Clock for FixedClock {
    fn now_unix_nanos(&self) -> u64 {
        self.0
    }
}

// ANCHOR[contextra-license:TRAIT-LICENSE-GATE]
/// Trait für die Offline-Prüfung und Autorisierung von Feature-Ringen (§15).
pub trait LicenseGate: Send + Sync {
    /// Prüft, ob der angeforderte Feature-Ring aktiviert und zugänglich ist.
    fn check_ring(&self, ring: FeatureRing) -> Result<()>;

    /// Autorisiert den angeforderten Feature-Ring und liefert bei Erfolg ein [`AuthorizedRing`]-Token.
    fn authorize(&self, requested: FeatureRing) -> Result<AuthorizedRing> {
        self.check_ring(requested)?;
        Ok(AuthorizedRing::new(requested))
    }
}

// ANCHOR[contextra-license:STRUCT-OPEN-FAST-GATE]
/// Standard-Gate ohne Lizenzbeschränkung für den freien `Fast`-Ring (INV-LICENSE-2).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct OpenFastGate;

impl LicenseGate for OpenFastGate {
    fn check_ring(&self, ring: FeatureRing) -> Result<()> {
        match ring {
            FeatureRing::Fast => Ok(()),
            other => Err(LicenseError::NotActivated(other)),
        }
    }
}

// ============================================================================
// Transitionale Kompatibilitäts-Adapter (Feature: ports-compat)
// ============================================================================

#[cfg(feature = "ports-compat")]
mod compat {
    use super::*;

    impl From<FeatureRing> for contextra_ports::license::FeatureRing {
        fn from(ring: FeatureRing) -> Self {
            match ring {
                FeatureRing::Fast => contextra_ports::license::FeatureRing::Fast,
                FeatureRing::Sovereign => contextra_ports::license::FeatureRing::Sovereign,
                FeatureRing::Compliance => contextra_ports::license::FeatureRing::Compliance,
            }
        }
    }

    impl From<contextra_ports::license::FeatureRing> for FeatureRing {
        fn from(ring: contextra_ports::license::FeatureRing) -> Self {
            match ring {
                contextra_ports::license::FeatureRing::Fast => FeatureRing::Fast,
                contextra_ports::license::FeatureRing::Sovereign => FeatureRing::Sovereign,
                contextra_ports::license::FeatureRing::Compliance => FeatureRing::Compliance,
                _ => FeatureRing::Compliance,
            }
        }
    }

    impl From<LicenseError> for contextra_ports::license::LicenseError {
        fn from(err: LicenseError) -> Self {
            match err {
                LicenseError::NotActivated(ring) => {
                    contextra_ports::license::LicenseError::NotActivated(ring.into())
                }
                LicenseError::InvalidSignature => {
                    contextra_ports::license::LicenseError::InvalidSignature
                }
                LicenseError::Expired(ts) => contextra_ports::license::LicenseError::Expired(ts),
            }
        }
    }

    impl From<contextra_ports::license::LicenseError> for LicenseError {
        fn from(err: contextra_ports::license::LicenseError) -> Self {
            match err {
                contextra_ports::license::LicenseError::NotActivated(ring) => {
                    LicenseError::NotActivated(ring.into())
                }
                contextra_ports::license::LicenseError::InvalidSignature => {
                    LicenseError::InvalidSignature
                }
                contextra_ports::license::LicenseError::Expired(ts) => LicenseError::Expired(ts),
            }
        }
    }

    impl From<TenantId> for contextra_types::TenantId {
        fn from(id: TenantId) -> Self {
            contextra_types::TenantId(id.0)
        }
    }

    impl From<contextra_types::TenantId> for TenantId {
        fn from(id: contextra_types::TenantId) -> Self {
            TenantId(id.0)
        }
    }

    impl contextra_ports::license::LicenseGate for OpenFastGate {
        fn check_ring(
            &self,
            ring: contextra_ports::license::FeatureRing,
        ) -> std::result::Result<(), contextra_ports::license::LicenseError> {
            LicenseGate::check_ring(self, ring.into()).map_err(Into::into)
        }
    }

    impl<T: contextra_ports::clock::Clock + ?Sized> Clock for T {
        fn now_unix_nanos(&self) -> u64 {
            contextra_ports::clock::Clock::now_unix_nanos(self)
        }
    }
}
