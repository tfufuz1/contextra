// FILE-CONTEXT
// ZWECK: Contextra Composition Root Builder (Ring 4, EP1).
// INVARIANTEN: Zero-panic in production code; orchestriert Subsystem-Konfiguration ohne Geschäftslogik.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use contextra_core::error::ContextraError;
use contextra_core::DistanceMetric;
use contextra_db::engine::{EngineWiring, EngineWiringExt};
use contextra_db::{Contextra, ContextraConfig, EmbeddingBackend, TextEmbeddingEngine};
use contextra_ports::clock::{Clock, SystemClock};
use contextra_ports::erasure::{ErasureHooks, ErasureMode};
use contextra_ports::id_gen::{IdGen, SystemIdGen};
use contextra_ports::license::{FeatureRing, LicenseGate, OpenFastGate};
use contextra_ports::rng::{Rng, SystemRng};
use contextra_ports::wiring::PortsWiring;
use contextra_store::kv::KvDeleteMode;
use contextra_store::lsm::DurabilityMode;

/// Error types emitted during contextra build/wiring verification (§4.7).
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum BuildError {
    /// Indicates an incomplete wiring configuration where a required component is missing.
    #[error("Incomplete wiring for flag '{flag}': missing {missing}")]
    IncompleteWiring {
        flag: String,
        missing: String,
    },
    /// Indicates that a requested operational mode requires a feature flag that was not compiled.
    #[error("Feature missing for mode '{mode}': feature '{feature}' is required")]
    FeatureMissing {
        mode: String,
        feature: String,
    },
    /// Indicates that a requested feature ring requires enterprise/Welle 3 licensing capabilities.
    #[error("Feature required: {0}")]
    FeatureRequired(String),
    /// Indicates an invalid or policy-violating combination of DurabilityMode and ErasureMode (§4.6).
    #[error("INV-DURABILITY-RING: Incompatible mode combination DurabilityMode::{durability} x ErasureMode::{erasure}")]
    IncompatibleModes {
        durability: String,
        erasure: String,
    },
    /// General engine initialization error.
    #[error("Engine build error: {0}")]
    Engine(String),
}

impl From<BuildError> for ContextraError {
    fn from(err: BuildError) -> Self {
        match err {
            BuildError::IncompleteWiring { flag, missing } => {
                ContextraError::PolicyViolation(format!("Incomplete wiring for flag '{flag}': missing {missing}"))
            }
            BuildError::FeatureMissing { mode, feature } => {
                ContextraError::PolicyViolation(format!("Feature missing for mode '{mode}': feature '{feature}' is required"))
            }
            BuildError::FeatureRequired(msg) => ContextraError::PolicyViolation(msg),
            BuildError::IncompatibleModes { durability, erasure } => {
                ContextraError::PolicyViolation(format!("INV-DURABILITY-RING: Incompatible mode combination DurabilityMode::{durability} x ErasureMode::{erasure}"))
            }
            BuildError::Engine(msg) => ContextraError::Internal(msg),
        }
    }
}

/// A builder for configuring and instantiating `Contextra`.
pub struct ContextraBuilder {
    storage_path: PathBuf,
    config: ContextraConfig,
    explicit_user_config: bool,
    embedder: Option<Arc<dyn TextEmbeddingEngine>>,
    license_gate: Arc<dyn LicenseGate>,
    performance_profile: Option<crate::performance_profile::PerformanceProfile>,
    collection_profile: Option<crate::collection_profile::CollectionProfile>,
    clock: Arc<dyn Clock>,
    rng: Arc<dyn Rng>,
    id_gen: Arc<dyn IdGen>,
    erasure_mode: ErasureMode,
    explicit_erasure_mode: bool,
    erasure_hooks: Option<Arc<dyn ErasureHooks>>,
}

impl std::fmt::Debug for ContextraBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ContextraBuilder")
            .field("storage_path", &self.storage_path)
            .field("config", &self.config)
            .field("explicit_user_config", &self.explicit_user_config)
            .field(
                "embedder",
                &self.embedder.as_ref().map(|_| "<dyn TextEmbeddingEngine>"),
            )
            .field("license_gate", &"<dyn LicenseGate>")
            .field("performance_profile", &self.performance_profile)
            .field("collection_profile", &self.collection_profile)
            .field("erasure_mode", &self.erasure_mode)
            .field(
                "erasure_hooks",
                &self.erasure_hooks.as_ref().map(|_| "<dyn ErasureHooks>"),
            )
            .finish()
    }
}

impl Clone for ContextraBuilder {
    fn clone(&self) -> Self {
        Self {
            storage_path: self.storage_path.clone(),
            config: self.config.clone(),
            explicit_user_config: self.explicit_user_config,
            embedder: self.embedder.clone(),
            license_gate: Arc::clone(&self.license_gate),
            performance_profile: self.performance_profile,
            collection_profile: self.collection_profile,
            clock: Arc::clone(&self.clock),
            rng: Arc::clone(&self.rng),
            id_gen: Arc::clone(&self.id_gen),
            erasure_mode: self.erasure_mode,
            explicit_erasure_mode: self.explicit_erasure_mode,
            erasure_hooks: self.erasure_hooks.clone(),
        }
    }
}

impl ContextraBuilder {
    /// Creates a new `ContextraBuilder` with specified vector dimension.
    pub fn new(dimension: usize) -> Self {
        let storage_path = PathBuf::from("./contextra_data");
        let ports_wiring = PortsWiring::build().unwrap_or_else(|_| PortsWiring {
            clock: Arc::new(SystemClock::new()),
            rng: Arc::new(SystemRng::new()),
            id_gen: Arc::new(SystemIdGen::new()),
        });
        let clock = ports_wiring.clock;
        let rng = ports_wiring.rng;
        let id_gen = ports_wiring.id_gen;
        let license_gate: Arc<dyn LicenseGate> = Arc::new(OpenFastGate);

        let kv_delete_mode = if cfg!(feature = "encryption-at-rest") {
            KvDeleteMode::CryptoShred
        } else {
            KvDeleteMode::TombstoneOnly
        };

        Self {
            storage_path,
            config: ContextraConfig {
                dimension,
                ..Default::default()
            },
            explicit_user_config: false,
            embedder: None,
            license_gate,
            performance_profile: None,
            collection_profile: None,
            clock,
            rng,
            id_gen,
            erasure_mode: ErasureMode::Off,
            explicit_erasure_mode: false,
            erasure_hooks: None,
        }
    }

    /// Creates a `ContextraBuilder` pre-populated from a `ContextraConfig`.
    pub fn from_config(config: ContextraConfig) -> Self {
        let mut builder = Self::new(config.dimension);
        builder = builder.with_max_elements(config.max_elements);
        builder = builder.with_distance_metric(config.distance_metric);
        if let Some(passphrase) = config.encryption_passphrase.clone() {
            builder = builder.with_encryption_passphrase(passphrase);
        }
        builder = builder.with_embedding_backend(config.embedding_backend);
        builder =
            builder.with_consolidation(config.consolidation_enabled, config.consolidation_interval);
        builder.explicit_user_config = true;
        builder
    }

    /// Creates a `ContextraBuilder` configured from a `CollectionProfile`.
    pub fn from_profile(profile: crate::collection_profile::CollectionProfile) -> Self {
        Self::new(768).with_collection_profile(profile)
    }

    /// Creates a `ContextraBuilder` configured for a specific `DeploymentTier`.
    pub fn from_tier(tier: crate::collection_profile::DeploymentTier) -> Self {
        Self::from_profile(tier.resolve())
    }

    /// Sets the storage directory path for the database engine.
    pub fn with_storage_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.storage_path = path.into();
        self
    }

    /// Sets the maximum number of vector elements stored.
    pub fn with_max_elements(mut self, max_elements: usize) -> Self {
        self.config.max_elements = max_elements;
        self
    }

    /// Sets the distance metric for vector similarity search.
    pub fn with_distance_metric(mut self, metric: DistanceMetric) -> Self {
        self.config.distance_metric = metric;
        self
    }

    /// Sets the encryption passphrase for persistent storage.
    pub fn with_encryption_passphrase(mut self, passphrase: impl Into<String>) -> Self {
        self.config.encryption_passphrase = Some(passphrase.into());
        self
    }

    /// Sets the embedding backend configuration.
    pub fn with_embedding_backend(mut self, backend: EmbeddingBackend) -> Self {
        self.config.embedding_backend = backend;
        self
    }

    /// Configures memory consolidation behavior.
    pub fn with_consolidation(mut self, enabled: bool, interval: Duration) -> Self {
        self.config.consolidation_enabled = enabled;
        self.config.consolidation_interval = interval;
        self
    }

    /// Attaches a custom text embedding engine implementation.
    pub fn with_embedder(mut self, embedder: Arc<dyn TextEmbeddingEngine>) -> Self {
        self.embedder = Some(embedder);
        self
    }

    /// Attaches a license gate for feature ring authorization.
    pub fn with_license_gate(mut self, gate: Arc<dyn LicenseGate>) -> Self {
        self.license_gate = gate;
        self
    }

    /// Sets the operational erasure mode (§4.6, §4.7).
    pub fn with_erasure_mode(mut self, mode: ErasureMode) -> Self {
        self.erasure_mode = mode;
        self.explicit_erasure_mode = true;
        self
    }

    /// Attaches custom erasure lifecycle hooks.
    pub fn with_erasure_hooks(mut self, hooks: Arc<dyn ErasureHooks>) -> Self {
        self.erasure_hooks = Some(hooks);
        self
    }

    /// Sets the performance profile preset for the contextra instance.
    pub fn with_performance_profile(
        mut self,
        profile: crate::performance_profile::PerformanceProfile,
    ) -> Self {
        self.performance_profile = Some(profile);
        self
    }

    /// Attaches a `CollectionProfile` and pre-configures performance profile settings.
    pub fn with_collection_profile(
        self,
        profile: crate::collection_profile::CollectionProfile,
    ) -> Self {
        let mut b = self.with_performance_profile(profile.performance);
        b.collection_profile = Some(profile);
        b
    }

    /// Attaches a custom [`Clock`] port.
    pub fn with_clock(mut self, clock: Arc<dyn Clock>) -> Self {
        self.clock = clock;
        self
    }

    /// Attaches a custom [`Rng`] port.
    pub fn with_rng(mut self, rng: Arc<dyn Rng>) -> Self {
        self.rng = rng;
        self
    }

    /// Attaches a custom [`IdGen`] port.
    pub fn with_id_gen(mut self, id_gen: Arc<dyn IdGen>) -> Self {
        self.id_gen = id_gen;
        self
    }

    /// Sets the complete `ContextraConfig` directly.
    pub fn with_config(self, config: ContextraConfig) -> Self {
        let storage_path = self.storage_path;
        let embedder = self.embedder;
        let license_gate = self.license_gate;
        let performance_profile = self.performance_profile;
        let collection_profile = self.collection_profile;
        let clock = self.clock;
        let rng = self.rng;
        let id_gen = self.id_gen;

        let mut b = Self::from_config(config);
        b.storage_path = storage_path;
        b.embedder = embedder;
        b.license_gate = license_gate;
        b.performance_profile = performance_profile;
        b.collection_profile = collection_profile;
        b.clock = clock;
        b.rng = rng;
        b.id_gen = id_gen;
        b
    }

    /// Returns the configured storage path.
    pub fn storage_path(&self) -> &std::path::Path {
        &self.storage_path
    }

    /// Returns a reference to the inner [`ContextraConfig`].
    pub fn config(&self) -> &ContextraConfig {
        &self.config
    }

    /// Returns a reference to the configured license gate.
    pub fn license_gate(&self) -> &Arc<dyn LicenseGate> {
        &self.license_gate
    }

    /// Returns the configured clock port.
    pub fn clock(&self) -> &Arc<dyn Clock> {
        &self.clock
    }

    /// Returns the configured RNG port.
    pub fn rng(&self) -> &Arc<dyn Rng> {
        &self.rng
    }

    /// Returns the configured ID generator port.
    pub fn id_gen(&self) -> &Arc<dyn IdGen> {
        &self.id_gen
    }

    pub fn erasure_mode(&self) -> ErasureMode {
        self.erasure_mode
    }

    pub fn explicit_erasure_mode(&self) -> bool {
        self.explicit_erasure_mode
    }

    pub fn erasure_hooks(&self) -> Option<&Arc<dyn ErasureHooks>> {
        self.erasure_hooks.as_ref()
    }

    /// Returns the configured performance profile preset, if any.
    pub fn performance_profile(&self) -> Option<crate::performance_profile::PerformanceProfile> {
        self.performance_profile
    }

    /// Generates a wiring self-test report for this builder.
    pub fn wiring_report(&self) -> crate::wiring_selftest::WiringReport {
        crate::wiring_selftest::generate_report_for_builder(self)
    }

    /// Builds and initializes the `Contextra` engine instance.
    pub async fn build(mut self) -> Result<Contextra, ContextraError> {
        if let Some(ref collection_profile) = self.collection_profile {
            collection_profile
                .validate_with_license(self.license_gate.as_ref())
                .map_err(|e| ContextraError::PolicyViolation(e.to_string()))?;
        }

        let requested_ring = if let Some(profile) = self.performance_profile {
            let resolved = profile.resolve();
            if self.explicit_user_config {
                if self.config.durability_mode != resolved.durability_mode
                    || self.config.vector_delete_mode != resolved.vector_delete_mode
                {
                    return Err(ContextraError::PolicyViolation(
                        "Explicit ContextraConfig conflicts with PerformanceProfile settings"
                            .to_string(),
                    ));
                }
            } else {
                self.config.durability_mode = resolved.durability_mode;
                self.config.vector_delete_mode = resolved.vector_delete_mode;
                self.erasure_mode = resolved.erasure_mode;
            }
            resolved.feature_ring
        } else {
            FeatureRing::Fast
        };

        if requested_ring != FeatureRing::Fast {
            return Err(BuildError::FeatureRequired(format!(
                "Feature ring {:?} requires Welle 3 license activation",
                requested_ring
            ))
            .into());
        }

        self.config.erasure_mode = self.erasure_mode;

        let authorized_token = self
            .license_gate
            .authorize(requested_ring)
            .map_err(|e| ContextraError::PolicyViolation(e.to_string()))?;

        // Honest verify_wiring check
        let report = crate::wiring_selftest::verify_builder_wiring(&self)?;

        // DurabilityMode x ErasureMode matrix check (§4.6)
        let durability = self.config.durability_mode;
        let erasure = self.erasure_mode;
        match (durability, erasure) {
            (DurabilityMode::Full, _) => {}
            (DurabilityMode::WalNoHmac, ErasureMode::Off) => {}
            (DurabilityMode::WalNoHmac, ErasureMode::Assured) => {
                tracing::warn!(
                    "W-DURABILITY-WAL-NOHMAC: ErasureMode::Assured active under DurabilityMode::WalNoHmac; ledger has independent MAC protection."
                );
            }
            (DurabilityMode::WalNoHmac, ErasureMode::Attested) => {
                return Err(BuildError::IncompatibleModes {
                    durability: format!("{durability:?}"),
                    erasure: format!("{erasure:?}"),
                }
                .into());
            }
            (DurabilityMode::MemoryOnly, ErasureMode::Off) => {}
            (DurabilityMode::MemoryOnly, ErasureMode::Assured | ErasureMode::Attested) => {
                return Err(BuildError::IncompatibleModes {
                    durability: format!("{durability:?}"),
                    erasure: format!("{erasure:?}"),
                }
                .into());
            }
            _ => {}
        }

        let instance = Contextra::open_authorized(
            &self.storage_path,
            self.config,
            &authorized_token,
            self.license_gate,
        )
        .await?;

        let engine_wiring = EngineWiring::default();

        let _ = &report;
        instance.apply(engine_wiring);

        if let Some(embedder) = self.embedder {
            Ok(instance.with_embedder(embedder).await)
        } else {
            Ok(instance)
        }
    }
}

impl Default for ContextraBuilder {
    fn default() -> Self {
        Self::new(768)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_defaults() {
        let builder = ContextraBuilder::default();
        assert_eq!(builder.config.dimension, 768);
        assert_eq!(builder.storage_path, PathBuf::from("./contextra_data"));
        assert!(builder.embedder.is_none());
        assert_eq!(builder.erasure_mode(), ErasureMode::Off);
    }

    #[test]
    fn test_builder_custom_configuration() {
        let builder = ContextraBuilder::new(128)
            .with_storage_path("/tmp/test_contextra_db")
            .with_max_elements(50000)
            .with_distance_metric(DistanceMetric::Euclidean)
            .with_encryption_passphrase("secret_pass")
            .with_consolidation(false, Duration::from_secs(300));

        assert_eq!(builder.config.dimension, 128);
        assert_eq!(
            builder.storage_path,
            PathBuf::from("/tmp/test_contextra_db")
        );
        assert_eq!(builder.config.max_elements, 50000);
        assert_eq!(builder.config.distance_metric, DistanceMetric::Euclidean);
        assert_eq!(
            builder.config.encryption_passphrase,
            Some("secret_pass".to_string())
        );
        assert!(!builder.config.consolidation_enabled);
        assert_eq!(
            builder.config.consolidation_interval,
            Duration::from_secs(300)
        );
    }

    #[tokio::test]
    async fn test_builder_build_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let tmp_path =
            std::env::temp_dir().join(format!("contextra_builder_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp_path);

        let db = ContextraBuilder::new(4)
            .with_storage_path(&tmp_path)
            .with_distance_metric(DistanceMetric::Cosine)
            .build()
            .await?;

        assert_eq!(db.len().await?, 0);
        let _ = std::fs::remove_dir_all(&tmp_path);
        Ok(())
    }

    #[test]
    fn test_builder_ports_attachment() {
        let clock: Arc<dyn Clock> = Arc::new(SystemClock::new());
        let rng: Arc<dyn Rng> = Arc::new(SystemRng::new());
        let id_gen: Arc<dyn IdGen> = Arc::new(SystemIdGen::new());

        let b = ContextraBuilder::new(128)
            .with_clock(clock.clone())
            .with_rng(rng.clone())
            .with_id_gen(id_gen.clone());

        assert!(Arc::ptr_eq(b.clock(), &clock));
        assert!(Arc::ptr_eq(b.rng(), &rng));
        assert!(Arc::ptr_eq(b.id_gen(), &id_gen));
    }

    #[tokio::test]
    async fn test_builder_wiring_report_integration() -> Result<(), Box<dyn std::error::Error>> {
        use crate::wiring_selftest::ContextraWiringReportExt;

        let tmp_path = std::env::temp_dir().join(format!(
            "contextra_wiring_rep_test_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&tmp_path);

        let db = ContextraBuilder::new(4)
            .with_storage_path(&tmp_path)
            .build()
            .await?;

        let report = db.wiring_report();
        assert!(report.is_valid());
        assert!(report.wired_ports.contains(&"Clock".to_string()));
        assert!(report.wired_ports.contains(&"EngineWiring".to_string()));

        let _ = std::fs::remove_dir_all(&tmp_path);
        Ok(())
    }
}
