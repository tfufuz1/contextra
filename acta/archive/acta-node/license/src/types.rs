//! # Crate-lokale Kern-Typen und Traits für die Lizenzprüfung.
//!
//! Stellt fundamentale Schnittstellen und Datenstrukturen zur Verfügung, die für eine
//! autarke Funktionsweise (Klasse A, B1–B12) ohne Abhängigkeit von `acta-base`
//! oder `acta-base` erforderlich sind.

use serde::{Deserialize, Serialize};

// ANCHOR[acta-node:TYPE-FEATURE-RING]
/// Einteilung der Acta-Funktionsringe (§14.6).
///
/// Definiert die Sicherheits- und Isolationsstufen von Acta-Instanzen.
/// Der freie `Fast`-Ring steht ohne Lizenz zur Verfügung.
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

// ANCHOR[acta-node:TYPE-AUTHORIZED-RING]
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

// ANCHOR[acta-node:TYPE-TENANT-ID]
/// Kanonischer 64-Bit-Ganzzahl-Mandantenidentifikator.
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

// ANCHOR[acta-node:TYPE-ERROR]
/// Typisierter Fehler für Lizenz- und Aktivierungsprüfungen (B4,).
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

/// Kanonischer Result-Typ für das `acta-node`-Crate.
pub type Result<T> = core::result::Result<T, LicenseError>;

// ANCHOR[acta-node:TRAIT-CLOCK]
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

// ANCHOR[acta-node:STRUCT-SYSTEM-CLOCK]
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

// ANCHOR[acta-node:STRUCT-FIXED-CLOCK]
/// Feste Test-Uhr für deterministische Tests und Benchmarks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FixedClock(pub u64);

impl Clock for FixedClock {
    fn now_unix_nanos(&self) -> u64 {
        self.0
    }
}

// ANCHOR[acta-node:TRAIT-LICENSE-GATE]
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

// ANCHOR[acta-node:STRUCT-OPEN-FAST-GATE]
/// Standard-Gate ohne Lizenzbeschränkung für den freien `Fast`-Ring.
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
