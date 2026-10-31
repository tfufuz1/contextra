// FILE-CONTEXT
// ZWECK: PerformanceProfile Presets for Contextra (Spec B.1.6, AP-P0-05)
// INVARIANTEN: INV-PERF-PROFILE-1, INV-PERF-PROFILE-2

//! # Modul: PerformanceProfile Presets (Ring 4)
//!
//! Dieses Modul definiert standardisierte Leistungs- und Persistenzprofile (`PerformanceProfile`),
//! die das Zusammenspiel von WAL-Durability, Hintergrund-Reparaturen im Vektorindex und
//! Löschnachweisen festlegen.
//!
//! ## Ring-Zugehörigkeit
//! - **Ring 4 (Fassade)**: Vordefinierte Voreinstellungen für Entwickler und Administratoren,
//!   ohne dass Details tieferliegender Schichten (LSM-WAL, HNSW-Graph-Reparatur) manuell konfiguriert
//!   werden müssen.
//!
//! ## Wichtigste Typen
//! - [`PerformanceProfile`]: Voreinstellungen `Compliance`, `Balanced` und `BareMetal`.
//! - [`ResolvedProfileConfig`]: Die konkrete Kombination aus DurabilityMode, VectorDeleteMode,
//!   Löschbeweis-Aktivierung und gefordertem FeatureRing.
//! - [`PerformanceProfileError`]: Mögliche Konfigurations- oder Lizenzfehler bei Profilvalidierung.
//!
//! ## Invarianten
//! - **INV-PERF-PROFILE-1**: Strikte Konsistenz: Volle Durability (`DurabilityMode::Full`) in Verbindung
//!   mit aktiven Löschnachweisen verbietet asynchrone Vektor-Löschung (`VectorDeleteMode::BackgroundRepair`),
//!   da Löschnachweise garantieren müssen, dass Vektoren synchron aus dem HNSW-Graphen getrennt wurden.
//! - **INV-PERF-PROFILE-2**: Lizenz-Erzwingung: Profile mit erhöhtem Feature-Ring (wie `Sovereign` im Compliance-Profil)
//!   dürfen nur instanziiert werden, wenn das hinterlegte [`LicenseGate`] die Berechtigung erteilt.

pub use contextra_ports::license::VectorDeleteMode;
use contextra_ports::license::{FeatureRing, LicenseError, LicenseGate};
use contextra_store::lsm::config::{DurabilityConfigError, DurabilityMode};

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
    /// Standardmäßig wird das `Compliance`-Profil gewählt (Security-by-Default / DSGVO-Konformität).
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
    pub deletion_proof_active: bool,
    pub feature_ring: FeatureRing,
}

// ANCHOR[contextra:PRF-012]
/// Fehler bei der Validierung oder Lizenzierung eines Performance-Profils.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum PerformanceProfileError {
    #[error("durability configuration invalid: {0}")]
    Durability(#[from] DurabilityConfigError),
    #[error("DurabilityMode::Full combined with VectorDeleteMode::BackgroundRepair while deletion-proof is active breaks the deletion-proof guarantee for the vector index")]
    DurabilityVectorMismatch,
    #[error("feature ring {0:?} requires a valid license: {1}")]
    LicenseRequired(FeatureRing, LicenseError),
}

impl PerformanceProfile {
    // ANCHOR[contextra:PRF-013]
    /// Löst das Profil in konkrete Konfigurationsparameter auf.
    ///
    /// # Ablauf von `resolve`:
    /// - `Compliance`: Full Durability, SynchronousRepair, DeletionProof aktiv, Sovereign-Ring.
    /// - `Balanced`: WalNoHmac, BackgroundRepair, DeletionProof inaktiv, Fast-Ring.
    /// - `BareMetal`: MemoryOnly, BackgroundRepair, DeletionProof inaktiv, Fast-Ring.
    pub fn resolve(self) -> ResolvedProfileConfig {
        match self {
            Self::Compliance => ResolvedProfileConfig {
                durability_mode: DurabilityMode::Full,
                vector_delete_mode: VectorDeleteMode::SynchronousRepair,
                deletion_proof_active: true,
                feature_ring: FeatureRing::Sovereign,
            },
            Self::Balanced => ResolvedProfileConfig {
                durability_mode: DurabilityMode::WalNoHmac,
                vector_delete_mode: VectorDeleteMode::BackgroundRepair,
                deletion_proof_active: false,
                feature_ring: FeatureRing::Fast,
            },
            Self::BareMetal => ResolvedProfileConfig {
                durability_mode: DurabilityMode::MemoryOnly,
                vector_delete_mode: VectorDeleteMode::BackgroundRepair,
                deletion_proof_active: false,
                feature_ring: FeatureRing::Fast,
            },
        }
    }
}

impl ResolvedProfileConfig {
    // ANCHOR[contextra:PRF-014]
    /// Prüft den geforderten Feature-Ring gegen das übergebene [`LicenseGate`].
    ///
    /// Der Basisring `FeatureRing::Fast` ist stets lizenzfrei erlaubt. Höhere Ringe
    /// (wie `Sovereign`) fordern eine gültige Autorisierung an.
    pub fn enforce_license(&self, gate: &dyn LicenseGate) -> Result<(), PerformanceProfileError> {
        if self.feature_ring != FeatureRing::Fast {
            gate.check_ring(self.feature_ring)
                .map_err(|e| PerformanceProfileError::LicenseRequired(self.feature_ring, e))?;
        }
        Ok(())
    }

    // ANCHOR[contextra:PRF-015]
    /// Validiert die gegenseitige Kompatibilität der Profilparameter.
    ///
    /// Stellt sicher, dass Durability und Vektor-Löschung nicht in einen ungültigen Zustand geraten
    /// (Invariante INV-PERF-PROFILE-1).
    pub fn validate(&self) -> Result<(), PerformanceProfileError> {
        self.durability_mode
            .validate_against_features(self.deletion_proof_active, self.feature_ring)?;

        if self.durability_mode == DurabilityMode::Full
            && self.vector_delete_mode == VectorDeleteMode::BackgroundRepair
            && self.deletion_proof_active
        {
            return Err(PerformanceProfileError::DurabilityVectorMismatch);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockLicenseGate {
        allowed_ring: FeatureRing,
    }

    impl LicenseGate for MockLicenseGate {
        fn check_ring(&self, ring: FeatureRing) -> Result<(), LicenseError> {
            if ring <= self.allowed_ring {
                Ok(())
            } else {
                Err(LicenseError::InsufficientRing)
            }
        }

        fn authorize(
            &self,
            ring: FeatureRing,
        ) -> Result<contextra_ports::license::GateToken, LicenseError> {
            self.check_ring(ring)?;
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
        assert!(comp.deletion_proof_active);
        assert_eq!(comp.feature_ring, FeatureRing::Sovereign);

        // Balanced
        let bal = PerformanceProfile::Balanced.resolve();
        assert_eq!(bal.durability_mode, DurabilityMode::WalNoHmac);
        assert_eq!(bal.vector_delete_mode, VectorDeleteMode::BackgroundRepair);
        assert!(!bal.deletion_proof_active);
        assert_eq!(bal.feature_ring, FeatureRing::Fast);

        // BareMetal
        let bare = PerformanceProfile::BareMetal.resolve();
        assert_eq!(bare.durability_mode, DurabilityMode::MemoryOnly);
        assert_eq!(bare.vector_delete_mode, VectorDeleteMode::BackgroundRepair);
        assert!(!bare.deletion_proof_active);
        assert_eq!(bare.feature_ring, FeatureRing::Fast);
    }

    #[test]
    fn test_all_resolved_presets_validate_successfully() {
        assert!(PerformanceProfile::Compliance.resolve().validate().is_ok());
        assert!(PerformanceProfile::Balanced.resolve().validate().is_ok());
        assert!(PerformanceProfile::BareMetal.resolve().validate().is_ok());
    }

    #[test]
    fn test_enforce_license_fast_ring_requires_no_gate() {
        let bal = PerformanceProfile::Balanced.resolve();
        let gate = MockLicenseGate {
            allowed_ring: FeatureRing::Fast,
        };
        assert!(bal.enforce_license(&gate).is_ok());

        let bare = PerformanceProfile::BareMetal.resolve();
        assert!(bare.enforce_license(&gate).is_ok());
    }

    #[test]
    fn test_enforce_license_sovereign_ring_with_fast_gate_fails() {
        let comp = PerformanceProfile::Compliance.resolve();
        let gate = MockLicenseGate {
            allowed_ring: FeatureRing::Fast,
        };
        let res = comp.enforce_license(&gate);
        assert!(matches!(
            res,
            Err(PerformanceProfileError::LicenseRequired(
                FeatureRing::Sovereign,
                LicenseError::InsufficientRing
            ))
        ));
    }

    #[test]
    fn test_enforce_license_sovereign_ring_with_sovereign_gate_succeeds() {
        let comp = PerformanceProfile::Compliance.resolve();
        let gate = MockLicenseGate {
            allowed_ring: FeatureRing::Sovereign,
        };
        assert!(comp.enforce_license(&gate).is_ok());
    }

    #[test]
    fn test_durability_vector_mismatch_error() {
        let invalid_config = ResolvedProfileConfig {
            durability_mode: DurabilityMode::Full,
            vector_delete_mode: VectorDeleteMode::BackgroundRepair,
            deletion_proof_active: true,
            feature_ring: FeatureRing::Sovereign,
        };
        assert_eq!(
            invalid_config.validate(),
            Err(PerformanceProfileError::DurabilityVectorMismatch)
        );
    }

    #[test]
    fn test_error_display_and_from_conversion() {
        let mismatch_err = PerformanceProfileError::DurabilityVectorMismatch;
        assert!(mismatch_err
            .to_string()
            .contains("breaks the deletion-proof guarantee"));

        let lic_err = PerformanceProfileError::LicenseRequired(
            FeatureRing::Sovereign,
            LicenseError::InsufficientRing,
        );
        assert!(lic_err.to_string().contains("requires a valid license"));
    }
}
