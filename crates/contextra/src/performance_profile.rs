// FILE-CONTEXT
// ZWECK: PerformanceProfile Presets for Contextra (Spec B.1.6, AP-P0-05)
// INVARIANTEN: INV-PERF-PROFILE-1, INV-PERF-PROFILE-2

//! # Modul: PerformanceProfile Presets (Ring 4)
//!
//! Dieses Modul definiert standardisierte Leistungs- und Persistenzprofile (`PerformanceProfile`),
//! die das Zusammenspiel von WAL-Durability, Hintergrund-Reparaturen im Vektorindex und
//! Integritätsnachweisen festlegen.
//!
//! ## Ring-Zugehörigkeit
//! - **Ring 4 (Fassade)**: Vordefinierte Voreinstellungen für Entwickler und Administratoren,
//!   ohne dass Details tieferliegender Schichten (LSM-WAL, HNSW-Graph-Reparatur) manuell konfiguriert
//!   werden müssen.
//!
//! ## Wichtigste Typen
//! - [`PerformanceProfile`]: Voreinstellungen `Compliance`, `Balanced` und `BareMetal`.
//! - [`DurabilityMode`]: Bestimmt den Persistenz- und Flush-Modus (Full, WalNoHmac, MemoryOnly).
//! - [`ResolvedProfileConfig`]: Die konkrete Kombination aus DurabilityMode, VectorDeleteMode,
//!   Integritätsnachweis-Aktivierung und gefordertem FeatureRing.
//! - [`PerformanceProfileError`]: Mögliche Konfigurations- oder Lizenzfehler bei Profilvalidierung.
//!
//! ## Invarianten
//! - **INV-PERF-PROFILE-1**: Strikte Konsistenz: Volle Durability (`DurabilityMode::Full`) in Verbindung
//!   mit aktiven Integritätsnachweisen verbietet asynchrone Vektor-Bereinigung (`VectorDeleteMode::BackgroundRepair`),
//!   da Integritätsnachweise gewährleisten müssen, dass Vektoren synchron aus dem HNSW-Graphen getrennt wurden.
//! - **INV-PERF-PROFILE-2**: Lizenz-Erzwingung: Veraltet. Lizenzprüfung wurde entfernt,
//!   alle Profile sind lizenzfrei nutzbar.

pub use contextra_ports::license::VectorDeleteMode;
use contextra_ports::license::FeatureRing;
#[allow(deprecated)]
use contextra_ports::license::LicenseGate;
use serde::{Deserialize, Serialize};

/// Durability mode for WAL persistence and integrity checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum DurabilityMode {
    Full,
    WalNoHmac,
    MemoryOnly,
}

impl Default for DurabilityMode {
    fn default() -> Self {
        Self::Full
    }
}

/// Fehler bei der Validierung des DurabilityMode gegen Ring- und Integritätsfeatures.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum DurabilityConfigError {
    #[error("DurabilityMode::{mode:?} is incompatible with feature '{feature}': {reason}")]
    IncompatibleCombination {
        mode: DurabilityMode,
        feature: &'static str,
        reason: &'static str,
    },
}

impl From<DurabilityConfigError> for contextra_types::ContextraError {
    fn from(err: DurabilityConfigError) -> Self {
        contextra_types::ContextraError::DurabilityConfig(err.to_string())
    }
}

impl DurabilityMode {
    /// Validiert die Kombination aus DurabilityMode, Integritätsnachweisen und Feature-Ring.
    pub fn validate_against_features(
        self,
        integrity_proof_active: bool,
        feature_ring: FeatureRing,
    ) -> Result<(), DurabilityConfigError> {
        match self {
            Self::Full => Ok(()),
            Self::WalNoHmac if integrity_proof_active => {
                Err(DurabilityConfigError::IncompatibleCombination {
                    mode: self,
                    feature: "integrity-proof",
                    reason: "no HMAC integrity chain to anchor the proof",
                })
            }
            Self::MemoryOnly
                if integrity_proof_active || feature_ring == FeatureRing::Sovereign =>
            {
                Err(DurabilityConfigError::IncompatibleCombination {
                    mode: self,
                    feature: "integrity-proof / FeatureRing::Sovereign",
                    reason: "no persistence layer exists to verify integrity from",
                })
            }
            _ => Ok(()),
        }
    }
}

// ANCHOR[contextra:PRF-010]
/// Vordefinierte Leistungsprofile für Contextra-Instanzen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum PerformanceProfile {
    Compliance,
    Balanced,
    BareMetal,
}

impl Default for PerformanceProfile {
    /// Standardmäßig wird das `Compliance`-Profil gewählt (Security-by-Default / maximale Schutzstufe).
    fn default() -> Self {
        Self::Compliance
    }
}

// ANCHOR[contextra:PRF-011]
/// Vollständig aufgelöste Konfigurationsparameter eines Performance-Profils.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolvedProfileConfig {
    pub durability_mode: DurabilityMode,
    pub vector_delete_mode: VectorDeleteMode,
    pub integrity_proof_active: bool,
    pub feature_ring: FeatureRing,
}

// ANCHOR[contextra:PRF-012]
/// Fehler bei der Validierung oder Lizenzierung eines Performance-Profils.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum PerformanceProfileError {
    #[error("durability configuration invalid: {0}")]
    Durability(#[from] DurabilityConfigError),
    #[error("DurabilityMode::Full combined with VectorDeleteMode::BackgroundRepair while integrity-proof is active breaks the integrity guarantee for the vector index")]
    DurabilityVectorMismatch,
    #[deprecated(note = "license gating was removed; unused")]
    #[error("feature ring {0:?} requires a valid license: {1}")]
    LicenseRequired(FeatureRing, contextra_ports::license::LicenseError),
}

impl PerformanceProfile {
    // ANCHOR[contextra:PRF-013]
    /// Löst das Profil in konkrete Konfigurationsparameter auf.
    ///
    /// # Ablauf von `resolve`:
    /// - `Compliance`: Full Durability, SynchronousRepair, IntegrityProof aktiv, Sovereign-Ring.
    /// - `Balanced`: WalNoHmac, BackgroundRepair, IntegrityProof inaktiv, Fast-Ring.
    /// - `BareMetal`: MemoryOnly, BackgroundRepair, IntegrityProof inaktiv, Fast-Ring.
    pub fn resolve(self) -> ResolvedProfileConfig {
        match self {
            Self::Compliance => ResolvedProfileConfig {
                durability_mode: DurabilityMode::Full,
                vector_delete_mode: VectorDeleteMode::SynchronousRepair,
                integrity_proof_active: true,
                feature_ring: FeatureRing::Sovereign,
            },
            Self::Balanced => ResolvedProfileConfig {
                durability_mode: DurabilityMode::WalNoHmac,
                vector_delete_mode: VectorDeleteMode::BackgroundRepair,
                integrity_proof_active: false,
                feature_ring: FeatureRing::Fast,
            },
            Self::BareMetal => ResolvedProfileConfig {
                durability_mode: DurabilityMode::MemoryOnly,
                vector_delete_mode: VectorDeleteMode::BackgroundRepair,
                integrity_proof_active: false,
                feature_ring: FeatureRing::Fast,
            },
        }
    }
}

impl ResolvedProfileConfig {
    // ANCHOR[contextra:PRF-014]
    /// Prüft den geforderten Feature-Ring gegen das übergebene [`LicenseGate`].
    ///
    /// # Veraltet
    /// Lizenzprüfung wurde entfernt; diese Methode ist ein No-op und liefert stets `Ok(())`.
    #[deprecated(note = "license gating was removed; no-op")]
    pub fn enforce_license(&self, _gate: &dyn LicenseGate) -> Result<(), PerformanceProfileError> {
        // Direkter Durchlauf: Alle Ringe sind lizenzfrei autorisiert.
        Ok(())
    }

    // ANCHOR[contextra:PRF-015]
    /// Validiert die gegenseitige Kompatibilität der Profilparameter.
    ///
    /// Stellt sicher, dass Durability und Vektor-Bereinigung nicht in einen inkonsistenten Zustand geraten
    /// (Invariante INV-PERF-PROFILE-1).
    pub fn validate(&self) -> Result<(), PerformanceProfileError> {
        self.durability_mode
            .validate_against_features(self.integrity_proof_active, self.feature_ring)?;

        if self.durability_mode == DurabilityMode::Full
            && self.vector_delete_mode == VectorDeleteMode::BackgroundRepair
            && self.integrity_proof_active
        {
            return Err(PerformanceProfileError::DurabilityVectorMismatch);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockLicenseGate;

    impl LicenseGate for MockLicenseGate {
        fn check_ring(&self, _ring: FeatureRing) -> Result<(), contextra_ports::license::LicenseError> {
            Ok(())
        }

        fn authorize(
            &self,
            ring: FeatureRing,
        ) -> Result<contextra_ports::license::GateToken, contextra_ports::license::LicenseError> {
            Ok(contextra_ports::license::GateToken::for_test(ring))
        }
    }

    #[test]
    fn test_performance_profile_default() {
        assert_eq!(
            PerformanceProfile::default(),
            PerformanceProfile::Compliance
        );
    }

    #[test]
    fn test_performance_profile_resolve_values() {
        // Compliance
        let comp = PerformanceProfile::Compliance.resolve();
        assert_eq!(comp.durability_mode, DurabilityMode::Full);
        assert_eq!(comp.vector_delete_mode, VectorDeleteMode::SynchronousRepair);
        assert!(comp.integrity_proof_active);
        assert_eq!(comp.feature_ring, FeatureRing::Sovereign);

        // Balanced
        let bal = PerformanceProfile::Balanced.resolve();
        assert_eq!(bal.durability_mode, DurabilityMode::WalNoHmac);
        assert_eq!(bal.vector_delete_mode, VectorDeleteMode::BackgroundRepair);
        assert!(!bal.integrity_proof_active);
        assert_eq!(bal.feature_ring, FeatureRing::Fast);

        // BareMetal
        let bare = PerformanceProfile::BareMetal.resolve();
        assert_eq!(bare.durability_mode, DurabilityMode::MemoryOnly);
        assert_eq!(bare.vector_delete_mode, VectorDeleteMode::BackgroundRepair);
        assert!(!bare.integrity_proof_active);
        assert_eq!(bare.feature_ring, FeatureRing::Fast);
    }

    #[test]
    fn test_all_resolved_presets_validate_successfully() {
        assert!(PerformanceProfile::Compliance.resolve().validate().is_ok());
        assert!(PerformanceProfile::Balanced.resolve().validate().is_ok());
        assert!(PerformanceProfile::BareMetal.resolve().validate().is_ok());
    }

    #[test]
    #[allow(deprecated)]
    fn test_enforce_license_always_succeeds_as_noop() {
        let comp = PerformanceProfile::Compliance.resolve();
        let bal = PerformanceProfile::Balanced.resolve();
        let bare = PerformanceProfile::BareMetal.resolve();
        let gate = MockLicenseGate;

        assert!(comp.enforce_license(&gate).is_ok());
        assert!(bal.enforce_license(&gate).is_ok());
        assert!(bare.enforce_license(&gate).is_ok());
    }

    #[test]
    fn test_durability_vector_mismatch_error() {
        let invalid_config = ResolvedProfileConfig {
            durability_mode: DurabilityMode::Full,
            vector_delete_mode: VectorDeleteMode::BackgroundRepair,
            integrity_proof_active: true,
            feature_ring: FeatureRing::Sovereign,
        };
        assert_eq!(
            invalid_config.validate(),
            Err(PerformanceProfileError::DurabilityVectorMismatch)
        );
    }

    #[test]
    #[allow(deprecated)]
    fn test_error_display_and_from_conversion() {
        let mismatch_err = PerformanceProfileError::DurabilityVectorMismatch;
        assert!(mismatch_err
            .to_string()
            .contains("breaks the integrity guarantee"));

        let lic_err = PerformanceProfileError::LicenseRequired(
            FeatureRing::Sovereign,
            contextra_ports::license::LicenseError::InsufficientRing,
        );
        assert!(lic_err.to_string().contains("requires a valid license"));
    }
}

impl From<DurabilityMode> for contextra_db::DurabilityMode {
    fn from(m: DurabilityMode) -> Self {
        match m {
            DurabilityMode::Full => contextra_db::DurabilityMode::Full,
            DurabilityMode::WalNoHmac => contextra_db::DurabilityMode::WalNoHmac,
            DurabilityMode::MemoryOnly => contextra_db::DurabilityMode::MemoryOnly,
        }
    }
}

impl From<contextra_db::DurabilityMode> for DurabilityMode {
    fn from(m: contextra_db::DurabilityMode) -> Self {
        match m {
            contextra_db::DurabilityMode::Full => DurabilityMode::Full,
            contextra_db::DurabilityMode::WalNoHmac => DurabilityMode::WalNoHmac,
            contextra_db::DurabilityMode::MemoryOnly => DurabilityMode::MemoryOnly,
            _ => DurabilityMode::Full,
        }
    }
}
