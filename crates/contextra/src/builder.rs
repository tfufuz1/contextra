// FILE-CONTEXT
// ZWECK: Contextra Composition Root Builder (Ring 4).
// INVARIANTEN: Zero-panic in production code; orchestriert Subsystem-Konfiguration ohne Geschäftslogik.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use contextra_core::error::ContextraError;
use contextra_core::DistanceMetric;
use contextra_db::{Contextra, ContextraConfig, EmbeddingBackend, TextEmbeddingEngine};
use contextra_license::SignedLicenseGate;
use contextra_ports::license::{FeatureRing, LicenseError, LicenseGate, OpenFastGate};

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
        }
    }
}

impl ContextraBuilder {
    /// Creates a new `ContextraBuilder` with specified vector dimension.
    pub fn new(dimension: usize) -> Self {
        Self {
            storage_path: PathBuf::from("./contextra_data"),
            config: ContextraConfig {
                dimension,
                ..Default::default()
            },
            explicit_user_config: false,
            embedder: None,
            license_gate: Arc::new(OpenFastGate),
            signed_license_error: None,
            performance_profile: None,
            collection_profile: None,
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
        builder = builder.with_consolidation(
            config.consolidation_enabled,
            config.consolidation_interval,
        );
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

    /// Sets the complete `ContextraConfig` directly.
    pub fn with_config(self, config: ContextraConfig) -> Self {
        let storage_path = self.storage_path;
        let embedder = self.embedder;
        let license_gate = self.license_gate;
        let signed_license_error = self.signed_license_error;
        let performance_profile = self.performance_profile;
        let collection_profile = self.collection_profile;

        let mut b = Self::from_config(config);
        b.storage_path = storage_path;
        b.embedder = embedder;
        b.license_gate = license_gate;
        b.signed_license_error = signed_license_error;
        b.performance_profile = performance_profile;
        b.collection_profile = collection_profile;
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

    /// Returns the configured performance profile preset, if any.
    pub fn performance_profile(&self) -> Option<crate::performance_profile::PerformanceProfile> {
        self.performance_profile
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

        let instance = Contextra::open_authorized(
            &self.storage_path,
            self.config,
            &authorized_token,
            self.license_gate,
        )
        .await?;

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
}
