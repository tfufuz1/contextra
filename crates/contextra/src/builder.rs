// FILE-CONTEXT
// ZWECK: Contextra Composition Root Builder (Ring 4).
// INVARIANTEN: Zero-panic in production code; orchestriert Subsystem-Konfiguration ohne Geschäftslogik.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use contextra_core::error::ContextraError;
use contextra_core::DistanceMetric;
use contextra_db::engine::{EngineWiring, EngineWiringExt};
use contextra_db::{Contextra, ContextraConfig, EmbeddingBackend, TextEmbeddingEngine};
use contextra_license::signed_gate::SignedLicenseGate;
use contextra_license::wiring::{LicenseWiring, LicenseWiringDeps};
use contextra_license::LicenseGateMode;
use contextra_ports::clock::{Clock, SystemClock};
use contextra_ports::id_gen::{IdGen, SystemIdGen};
use contextra_ports::license::{FeatureRing, LicenseError, LicenseGate};
use contextra_ports::rng::{Rng, SystemRng};
use contextra_ports::wiring::PortsWiring;

/// A builder for configuring and instantiating `Contextra`.
///
/// `ContextraBuilder` acts as the primary builder pattern in the `contextra` facade (Ring 4).
/// It orchestrates subsystem configuration without containing domain business logic.
pub struct ContextraBuilder {
    storage_path: PathBuf,
    config: ContextraConfig,
    explicit_user_config: bool,
    embedder: Option<Arc<dyn TextEmbeddingEngine>>,
    license_gate: Arc<dyn LicenseGate>,
    signed_license_error: Option<String>,
    performance_profile: Option<crate::performance_profile::PerformanceProfile>,
    collection_profile: Option<crate::collection_profile::CollectionProfile>,
    clock: Arc<dyn Clock>,
    rng: Arc<dyn Rng>,
    id_gen: Arc<dyn IdGen>,
    router_enabled: bool,
    adapt_enabled: bool,
    privacy_enabled: bool,
    sandbox_enabled: bool,
    audit_export_enabled: bool,
    avv_generator_enabled: bool,
    kms_enabled: bool,
    kv_shredding_enabled: bool,
    checkpoint_enabled: bool,

    #[cfg(feature = "sovereign")]
    pub sovereign_wiring: Option<Arc<crate::wiring_sovereign::SovereignWiringBundle>>,
    #[cfg(feature = "audit-export")]
    pub audit_wiring: Option<Arc<crate::wiring_audit::AuditWiringBundle>>,
    #[cfg(feature = "onnx")]
    pub onnx_wiring: Option<Arc<crate::wiring_onnx::OnnxWiringBundle>>,
    #[cfg(feature = "sandbox")]
    pub sandbox_wiring: Option<Arc<crate::wiring_sandbox::SandboxWiringBundle>>,
    #[cfg(feature = "cognition")]
    pub cognition_wiring: Option<Arc<crate::wiring_cognition::CognitionWiringBundle>>,
    #[cfg(feature = "router")]
    pub retrieval_wiring: Option<Arc<crate::wiring_retrieval::RetrievalWiringBundle>>,
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
            .field("signed_license_error", &self.signed_license_error)
            .field("performance_profile", &self.performance_profile)
            .field("collection_profile", &self.collection_profile)
            .field("router_enabled", &self.router_enabled)
            .field("adapt_enabled", &self.adapt_enabled)
            .field("privacy_enabled", &self.privacy_enabled)
            .field("sandbox_enabled", &self.sandbox_enabled)
            .field("audit_export_enabled", &self.audit_export_enabled)
            .field("avv_generator_enabled", &self.avv_generator_enabled)
            .field("kms_enabled", &self.kms_enabled)
            .field("kv_shredding_enabled", &self.kv_shredding_enabled)
            .field("checkpoint_enabled", &self.checkpoint_enabled)
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
            signed_license_error: self.signed_license_error.clone(),
            performance_profile: self.performance_profile,
            collection_profile: self.collection_profile,
            clock: Arc::clone(&self.clock),
            rng: Arc::clone(&self.rng),
            id_gen: Arc::clone(&self.id_gen),
            router_enabled: self.router_enabled,
            adapt_enabled: self.adapt_enabled,
            privacy_enabled: self.privacy_enabled,
            sandbox_enabled: self.sandbox_enabled,
            audit_export_enabled: self.audit_export_enabled,
            avv_generator_enabled: self.avv_generator_enabled,
            kms_enabled: self.kms_enabled,
            kv_shredding_enabled: self.kv_shredding_enabled,
            checkpoint_enabled: self.checkpoint_enabled,
            #[cfg(feature = "sovereign")]
            sovereign_wiring: self.sovereign_wiring.clone(),
            #[cfg(feature = "audit-export")]
            audit_wiring: self.audit_wiring.clone(),
            #[cfg(feature = "onnx")]
            onnx_wiring: self.onnx_wiring.clone(),
            #[cfg(feature = "sandbox")]
            sandbox_wiring: self.sandbox_wiring.clone(),
            #[cfg(feature = "cognition")]
            cognition_wiring: self.cognition_wiring.clone(),
            #[cfg(feature = "router")]
            retrieval_wiring: self.retrieval_wiring.clone(),
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

        let license_gate = LicenseWiring::build(LicenseWiringDeps {
            mode: LicenseGateMode::Enforced,
            storage_path: Some(storage_path.clone()),
            clock: Some(clock.clone()),
            ..Default::default()
        })
        .map(|w| w.into_gate())
        .unwrap_or_else(|_| Arc::new(SignedLicenseGate::no_activation_with_clock(clock.clone())));

        Self {
            storage_path,
            config: ContextraConfig {
                dimension,
                ..Default::default()
            },
            explicit_user_config: false,
            embedder: None,
            license_gate,
            signed_license_error: None,
            performance_profile: None,
            collection_profile: None,
            clock,
            rng,
            id_gen,
            router_enabled: true,
            adapt_enabled: true,
            privacy_enabled: true,
            sandbox_enabled: true,
            audit_export_enabled: true,
            avv_generator_enabled: true,
            kms_enabled: true,
            kv_shredding_enabled: true,
            checkpoint_enabled: true,

            #[cfg(feature = "sovereign")]
            sovereign_wiring: None,
            #[cfg(feature = "audit-export")]
            audit_wiring: None,
            #[cfg(feature = "onnx")]
            onnx_wiring: None,
            #[cfg(feature = "sandbox")]
            sandbox_wiring: None,
            #[cfg(feature = "cognition")]
            cognition_wiring: None,
            #[cfg(feature = "router")]
            retrieval_wiring: None,
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

    /// Creates a `ContextraBuilder` initialized with a cryptographic signed license gate (§14.6, §15).
    pub fn from_signed_license(
        payload_bytes: &[u8],
        signature: &[u8; 64],
        verifying_key_bytes: &[u8; 32],
    ) -> Self {
        Self::new(768).with_signed_license(payload_bytes, signature, verifying_key_bytes)
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
        self.signed_license_error = None;
        self
    }

    /// Attaches a cryptographic signed license gate (§14.6, §15).
    pub fn with_signed_license(
        mut self,
        payload_bytes: &[u8],
        signature: &[u8; 64],
        verifying_key_bytes: &[u8; 32],
    ) -> Self {
        match contextra_license::signed_gate::VerifyingKey::from_bytes(verifying_key_bytes) {
            Ok(key) => {
                match SignedLicenseGate::from_signed_payload(payload_bytes, signature, key) {
                    Ok(gate) => self.with_license_gate(Arc::new(gate)),
                    Err(e) => {
                        self.signed_license_error = Some(e.to_string());
                        self
                    }
                }
            }
            Err(_) => {
                self.signed_license_error = Some(LicenseError::InvalidSignature.to_string());
                self
            }
        }
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

    /// Configures the builder to use permissive development mode (`LicenseGateMode::DevelopmentOpen`).
    ///
    /// Allows Fast ring bypass without activation.
    /// Only available in test/debug builds or when the `dev-open-gate` feature is enabled.
    #[cfg(any(test, debug_assertions, feature = "dev-open-gate"))]
    pub fn with_development_open_gate(mut self) -> Self {
        if let Ok(wiring) = LicenseWiring::build(LicenseWiringDeps {
            mode: LicenseGateMode::DevelopmentOpen,
            storage_path: Some(self.storage_path.clone()),
            clock: Some(self.clock.clone()),
            ..Default::default()
        }) {
            self.license_gate = wiring.into_gate();
            self.signed_license_error = None;
        }
        self
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

    /// Configures whether the router subsystem is enabled.
    pub fn with_router(mut self, enabled: bool) -> Self {
        self.router_enabled = enabled;
        self
    }

    /// Disables the router subsystem.
    pub fn without_router(self) -> Self {
        self.with_router(false)
    }

    /// Configures whether the adapt subsystem is enabled.
    pub fn with_adapt(mut self, enabled: bool) -> Self {
        self.adapt_enabled = enabled;
        self
    }

    /// Disables the adapt subsystem.
    pub fn without_adapt(self) -> Self {
        self.with_adapt(false)
    }

    /// Configures whether the privacy/egress subsystem is enabled.
    pub fn with_privacy(mut self, enabled: bool) -> Self {
        self.privacy_enabled = enabled;
        self
    }

    /// Disables the privacy/egress subsystem.
    pub fn without_privacy(self) -> Self {
        self.with_privacy(false)
    }

    /// Configures whether the WASM sandbox subsystem is enabled.
    pub fn with_sandbox(mut self, enabled: bool) -> Self {
        self.sandbox_enabled = enabled;
        self
    }

    /// Disables the WASM sandbox subsystem.
    pub fn without_sandbox(self) -> Self {
        self.with_sandbox(false)
    }

    /// Configures whether the audit export subsystem is enabled.
    pub fn with_audit_export(mut self, enabled: bool) -> Self {
        self.audit_export_enabled = enabled;
        self
    }

    /// Disables the audit export subsystem.
    pub fn without_audit_export(self) -> Self {
        self.with_audit_export(false)
    }

    /// Configures whether the AVV generator subsystem is enabled.
    pub fn with_avv_generator(mut self, enabled: bool) -> Self {
        self.avv_generator_enabled = enabled;
        self
    }

    /// Disables the AVV generator subsystem.
    pub fn without_avv_generator(self) -> Self {
        self.with_avv_generator(false)
    }

    /// Configures whether the KMS subsystem is enabled.
    pub fn with_kms(mut self, enabled: bool) -> Self {
        self.kms_enabled = enabled;
        self
    }

    /// Disables the KMS subsystem.
    pub fn without_kms(self) -> Self {
        self.with_kms(false)
    }

    /// Configures whether the KV shredding subsystem is enabled.
    pub fn with_kv_shredding(mut self, enabled: bool) -> Self {
        self.kv_shredding_enabled = enabled;
        self
    }

    /// Disables the KV shredding subsystem.
    pub fn without_kv_shredding(self) -> Self {
        self.with_kv_shredding(false)
    }

    /// Configures whether the checkpoint subsystem is enabled.
    pub fn with_checkpoint(mut self, enabled: bool) -> Self {
        self.checkpoint_enabled = enabled;
        self
    }

    /// Disables the checkpoint subsystem.
    pub fn without_checkpoint(self) -> Self {
        self.with_checkpoint(false)
    }

    /// Sets the complete `ContextraConfig` directly.
    pub fn with_config(self, config: ContextraConfig) -> Self {
        let storage_path = self.storage_path;
        let embedder = self.embedder;
        let license_gate = self.license_gate;
        let signed_license_error = self.signed_license_error;
        let performance_profile = self.performance_profile;
        let collection_profile = self.collection_profile;
        let clock = self.clock;
        let rng = self.rng;
        let id_gen = self.id_gen;

        let mut b = Self::from_config(config);
        b.storage_path = storage_path;
        b.embedder = embedder;
        b.license_gate = license_gate;
        b.signed_license_error = signed_license_error;
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

    pub fn router_enabled(&self) -> bool {
        self.router_enabled
    }

    pub fn adapt_enabled(&self) -> bool {
        self.adapt_enabled
    }

    pub fn privacy_enabled(&self) -> bool {
        self.privacy_enabled
    }

    pub fn sandbox_enabled(&self) -> bool {
        self.sandbox_enabled
    }

    pub fn audit_export_enabled(&self) -> bool {
        self.audit_export_enabled
    }

    pub fn avv_generator_enabled(&self) -> bool {
        self.avv_generator_enabled
    }

    pub fn kms_enabled(&self) -> bool {
        self.kms_enabled
    }

    pub fn kv_shredding_enabled(&self) -> bool {
        self.kv_shredding_enabled
    }

    pub fn checkpoint_enabled(&self) -> bool {
        self.checkpoint_enabled
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
        if let Some(ref err_msg) = self.signed_license_error {
            return Err(ContextraError::PolicyViolation(err_msg.clone()));
        }

        if let Some(ref collection_profile) = self.collection_profile {
            collection_profile
                .validate_with_license(self.license_gate.as_ref())
                .map_err(|e| ContextraError::PolicyViolation(e.to_string()))?;
        }

        let requested_ring = if let Some(profile) = self.performance_profile {
            let resolved = profile.resolve();
            if self.explicit_user_config {
                if self.config.durability_mode != resolved.durability_mode
                    || self.config.deletion_proof_active != resolved.deletion_proof_active
                    || self.config.vector_delete_mode != resolved.vector_delete_mode
                {
                    return Err(ContextraError::PolicyViolation(
                        "Explicit ContextraConfig conflicts with PerformanceProfile settings"
                            .to_string(),
                    ));
                }
            } else {
                self.config.durability_mode = resolved.durability_mode;
                self.config.deletion_proof_active = resolved.deletion_proof_active;
                self.config.vector_delete_mode = resolved.vector_delete_mode;
            }
            resolved.feature_ring
        } else {
            FeatureRing::Fast
        };

        let authorized_token = self
            .license_gate
            .authorize(requested_ring)
            .map_err(|e| ContextraError::PolicyViolation(e.to_string()))?;

        let has_clock = self.clock.now_unix_nanos() >= 0;
        let has_license_gate = true;
        let license_gate_is_noop = requested_ring != FeatureRing::Fast
            && self.license_gate.check_ring(FeatureRing::Sovereign).is_ok()
            && self.license_gate.check_ring(FeatureRing::Compliance).is_ok();

        let _report = crate::wiring_selftest::verify_wiring(
            has_clock,
            has_license_gate,
            license_gate_is_noop,
        )?;

        let instance = Contextra::open_authorized(
            &self.storage_path,
            self.config,
            &authorized_token,
            self.license_gate,
        )
        .await?;

        #[cfg_attr(not(feature = "router"), allow(unused_mut))]
        let mut engine_wiring = EngineWiring::default();

        #[cfg(feature = "router")]
        let _router_wiring_keepalive = if self.router_enabled {
            // Dummy providers for router wiring when concrete providers are not available.
            // The router subsystem requires these trait objects but the builder may not
            // have them — use no-op stubs so the router can at least be instantiated.
            struct NoOpHybridSearch;
            impl contextra_ports::HybridSearchProvider for NoOpHybridSearch {
                fn search_hybrid<'a>(
                    &'a self,
                    _query_text: &'a str,
                    _query_embedding: &'a [f32],
                    _top_k: usize,
                ) -> contextra_ports::BoxFuture<'a, contextra_core::Result<Vec<contextra_ports::ContextChunk>>> {
                    Box::pin(async move { Ok(Vec::new()) })
                }
            }

            struct NoOpCommunityResolver;
            impl contextra_ports::CommunityResolver for NoOpCommunityResolver {
                fn get_community<'a>(
                    &'a self,
                    _entity_id: contextra_ports::EntityId,
                ) -> contextra_ports::BoxFuture<'a, contextra_core::Result<Option<u64>>> {
                    Box::pin(async move { Ok(None) })
                }
            }

            struct NoOpContextPreparer;
            impl contextra_ports::ContextPreparer for NoOpContextPreparer {
                fn prepare_context(
                    &self,
                    _chunks: Vec<contextra_ports::ContextChunk>,
                    _budget: &contextra_ports::TokenBudget,
                    _relevance_threshold: f32,
                ) -> contextra_core::Result<contextra_ports::ContextWindow> {
                    Ok(contextra_ports::ContextWindow {
                        chunks: Vec::new(),
                        total_tokens: 0,
                        truncated: false,
                    })
                }
            }

            let router_deps = contextra_router::RouterWiringDeps {
                search_provider: Arc::new(NoOpHybridSearch),
                community_resolver: Arc::new(NoOpCommunityResolver),
                context_preparer: Arc::new(NoOpContextPreparer),
                catalog: Vec::new(),
                clock: self.clock.clone(),
                arm_registry: None,
                calibration_store_path: None,
                transport: None,
                route_timeout: None,
            };
            if let Ok(router_wiring) = contextra_router::RouterWiring::build(router_deps) {
                let arc_rw = Arc::new(router_wiring);
                engine_wiring.router = Some(Arc::downgrade(&arc_rw) as _);
                Some(arc_rw)
            } else {
                None
            }
        } else {
            None
        };

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
    fn test_builder_subsystem_levers() {
        let b = ContextraBuilder::new(128)
            .without_router()
            .without_adapt()
            .without_privacy()
            .without_sandbox()
            .without_audit_export()
            .without_avv_generator()
            .without_kms()
            .without_kv_shredding()
            .without_checkpoint();

        assert!(!b.router_enabled());
        assert!(!b.adapt_enabled());
        assert!(!b.privacy_enabled());
        assert!(!b.sandbox_enabled());
        assert!(!b.audit_export_enabled());
        assert!(!b.avv_generator_enabled());
        assert!(!b.kms_enabled());
        assert!(!b.kv_shredding_enabled());
        assert!(!b.checkpoint_enabled());

        let report = b.wiring_report();
        assert_eq!(report.missing_ports.len(), 9);

        let b = b
            .with_router(true)
            .with_adapt(true)
            .with_privacy(true)
            .with_sandbox(true)
            .with_audit_export(true)
            .with_avv_generator(true)
            .with_kms(true)
            .with_kv_shredding(true)
            .with_checkpoint(true);

        assert!(b.router_enabled());
        assert!(b.adapt_enabled());
        assert!(b.privacy_enabled());
        assert!(b.sandbox_enabled());
        assert!(b.audit_export_enabled());
        assert!(b.avv_generator_enabled());
        assert!(b.kms_enabled());
        assert!(b.kv_shredding_enabled());
        assert!(b.checkpoint_enabled());

        let report2 = b.wiring_report();
        assert!(report2.missing_ports.is_empty());
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
