// FILE-CONTEXT
// ZWECK: CollectionProfile & DeploymentTier Presets for Contextra (Spec B.1.8, AP-P0-05)
// INVARIANTEN: INV-COLLECTION-PROFILE-1, INV-COLLECTION-PROFILE-2

//! # Modul: CollectionProfile & DeploymentTier Presets (Ring 4)
//!
//! Dieses Modul definiert standardisierte Betriebs-Profile (`CollectionProfile`) und
//! Voreinstellungen (`DeploymentTier`) für Dokumentensammlungen (Collections).
//!
//! ## Ring-Zugehörigkeit & Zweck
//! - **Ring 4 (Fassade)**: Bietet übergeordnete Konfigurationsbündel für typische
//!   Betriebsumgebungen an, von ressourcenbeschränkten Edge-Geräten bis zu hochgradig
//!   regulierten Unternehmensumgebungen (DSGVO, Compliance).
//!
//! ## Wichtigste Typen & Konstanten
//! - [`DEFAULT_AUTO_EXTRACTION_MODE`]: Standardmodus für automatische Entitäten-Extraktion via LLM.
//! - [`ENTERPRISE_REGULATED_AUTO_EXTRACTION_DEFAULT`]: Abweichender Default für regulierte Umgebungen.
//! - [`LsmTuning`]: Feineinstellung für LSM-Speicherstrukturen (Memtable, RAM, Group-Commit).
//! - [`CollectionProfile`]: Kombiniert Performance, KV-Löschmodus, LSM-Tuning und Extraktionsmodus.
//! - [`DeploymentTier`]: Enumeration vordefinierter Betriebsstufen (`EdgeMinimal`, `PowerUserLocal`,
//!   `EnterpriseShared`, `EnterpriseRegulated`).
//!
//! ## Invarianten
//! - **INV-COLLECTION-PROFILE-1**: Wenn kryptografische Löschnachweise (DeletionProof) aktiv sind,
//!   darf `kv_delete_mode` nicht auf `TombstoneOnly` stehen, da reine Tombstones keinen kryptografischen
//!   Löschbeweis (Crypto-Shredding) erbringen können.
//! - **INV-COLLECTION-PROFILE-2**: Alle Voreinstellungen in `DeploymentTier` müssen bei Aufruf von `resolve()`
//!   stets valide sein (`validate().is_ok()`).

use crate::performance_profile::{PerformanceProfile, PerformanceProfileError};
use contextra_ports::license::LicenseGate;
use contextra_store::kv::delete_mode::KvDeleteMode;
pub use contextra_types::types::domain::AutoExtractionMode;

// ANCHOR[contextra:PRF-001]
/// Standardmodus für die automatische LLM-gestützte Metadaten-/Entitäten-Extraktion bei Standard-Tiers.
///
/// Standardmäßig aktiviert (`AutoExtractionMode::Enabled`).
pub const DEFAULT_AUTO_EXTRACTION_MODE: AutoExtractionMode = AutoExtractionMode::Enabled;

// ANCHOR[contextra:PRF-002]
/// Standardmodus für `EnterpriseRegulated`.
///
/// In EnterpriseRegulated wird kein Dokumenttext automatisch an ein Extraktions-LLM
/// gegeben, solange der Betreiber es nicht ausdrücklich aktiviert (Datenminimierung gemäß Art. 5 DSGVO);
/// explizite Aktivierung pro Collection bleibt möglich.
pub const ENTERPRISE_REGULATED_AUTO_EXTRACTION_DEFAULT: AutoExtractionMode =
    AutoExtractionMode::Disabled;

// ANCHOR[contextra:PRF-003]
/// Feineinstellungsparameter für die LSM-Speicher-Engine (Log-Structured Merge-Tree).
///
/// Regelt Speicherbudgets und Flush-Frequenzen, um Systemressourcen optimal auf die Hardware abzustimmen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LsmTuning {
    pub memtable_size_limit: u64,
    pub max_ram_mb: u32,
    pub group_commit_window_micros: u32,
    pub block_cache_shards: u16,
}

// ANCHOR[contextra:PRF-004]
/// Vollständiges Profil zur Konfiguration einer Collection.
///
/// Bündelt Performance-, Persistenz- und Datenschutzaspekte für eine Sammlung.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CollectionProfile {
    pub performance: PerformanceProfile,
    pub kv_delete_mode: KvDeleteMode,
    pub lsm_tuning: LsmTuning,
    pub auto_extraction: AutoExtractionMode,
}

// ANCHOR[contextra:PRF-005]
/// Mögliche Fehler bei der Validierung eines `CollectionProfile`.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum CollectionProfileError {
    #[error("performance profile invalid: {0}")]
    Performance(#[from] PerformanceProfileError),
    #[error("deletion-proof active but KvDeleteMode::TombstoneOnly cannot satisfy a KV-side DeletionProof")]
    KvDeleteMismatch,
}

impl CollectionProfile {
    // ANCHOR[contextra:PRF-006]
    /// Validiert die interne Konsistenz des Profils.
    ///
    /// # Ablauf von `validate`:
    /// 1. Performance-Profil auflösen und dessen Konfiguration prüfen (`resolved.validate()`).
    /// 2. Prüfen, ob `deletion_proof_active` mit `KvDeleteMode::TombstoneOnly` kombiniert wurde (Invariante INV-COLLECTION-PROFILE-1).
    ///
    /// # Fehler
    /// Gibt [`CollectionProfileError::KvDeleteMismatch`] zurück, falls Löschnachweise aktiv sind, aber nur Tombstones konfiguriert wurden.
    pub fn validate(&self) -> Result<(), CollectionProfileError> {
        let resolved = self.performance.resolve();
        resolved.validate()?;
        if resolved.deletion_proof_active && self.kv_delete_mode == KvDeleteMode::TombstoneOnly {
            return Err(CollectionProfileError::KvDeleteMismatch);
        }
        Ok(())
    }

    // ANCHOR[contextra:PRF-007]
    /// Validiert das Profil zusätzlich gegen ein Lizenzgate.
    ///
    /// Stellt sicher, dass das Profil intern konsistent ist und der geforderte Feature-Ring
    /// durch die Lizenz abgedeckt ist.
    pub fn validate_with_license(
        &self,
        gate: &dyn LicenseGate,
    ) -> Result<(), CollectionProfileError> {
        self.validate()?;
        let resolved = self.performance.resolve();
        resolved.enforce_license(gate)?;
        Ok(())
    }
}

// ANCHOR[contextra:PRF-008]
/// Vordefinierte Betriebsstufen (Deployment Tiers) für typische Anwendungsszenarien.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum DeploymentTier {
    EdgeMinimal,
    PowerUserLocal,
    EnterpriseShared,
    EnterpriseRegulated,
}

impl DeploymentTier {
    // ANCHOR[contextra:PRF-009]
    /// Löst die Betriebsstufe in ein konkretes, vorkonfiguriertes [`CollectionProfile`] auf.
    ///
    /// Garantiert stets ein konsistentes Profil gemäß Invariante INV-COLLECTION-PROFILE-2.
    pub fn resolve(self) -> CollectionProfile {
        match self {
            Self::EdgeMinimal => CollectionProfile {
                performance: PerformanceProfile::BareMetal,
                kv_delete_mode: KvDeleteMode::TombstoneOnly,
                lsm_tuning: LsmTuning {
                    memtable_size_limit: 8 << 20,
                    max_ram_mb: 64,
                    group_commit_window_micros: 200,
                    block_cache_shards: 2,
                },
                auto_extraction: DEFAULT_AUTO_EXTRACTION_MODE,
            },
            Self::PowerUserLocal => CollectionProfile {
                performance: PerformanceProfile::Balanced,
                kv_delete_mode: KvDeleteMode::TombstoneOnly,
                lsm_tuning: LsmTuning {
                    memtable_size_limit: 64 << 20,
                    max_ram_mb: 512,
                    group_commit_window_micros: 2_000,
                    block_cache_shards: 8,
                },
                auto_extraction: DEFAULT_AUTO_EXTRACTION_MODE,
            },
            Self::EnterpriseShared => CollectionProfile {
                performance: PerformanceProfile::Balanced,
                kv_delete_mode: KvDeleteMode::CryptoShred,
                lsm_tuning: LsmTuning {
                    memtable_size_limit: 256 << 20,
                    max_ram_mb: 4_096,
                    group_commit_window_micros: 10_000,
                    block_cache_shards: 32,
                },
                auto_extraction: DEFAULT_AUTO_EXTRACTION_MODE,
            },
            Self::EnterpriseRegulated => CollectionProfile {
                performance: PerformanceProfile::Compliance,
                kv_delete_mode: KvDeleteMode::CryptoShred,
                lsm_tuning: LsmTuning {
                    memtable_size_limit: 256 << 20,
                    max_ram_mb: 4_096,
                    group_commit_window_micros: 10_000,
                    block_cache_shards: 32,
                },
                auto_extraction: ENTERPRISE_REGULATED_AUTO_EXTRACTION_DEFAULT,
            },
        }
    }
}
