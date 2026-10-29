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
use zeroize::Zeroizing;

/// Storage encryption mode configuration for persistent database layers.
#[derive(Clone)]
pub enum EncryptionMode {
    /// Storage is encrypted using a key derived from a passphrase.
    Passphrase(Zeroizing<String>),
    /// Storage is explicitly unencrypted.
    Unencrypted,
}

impl std::fmt::Debug for EncryptionMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Passphrase(_) => f.debug_tuple("Passphrase").field(&"[REDACTED]").finish(),
            Self::Unencrypted => write!(f, "Unencrypted"),
        }
    }
}

/// Extension trait providing encryption status helper for [`ContextraConfig`].
pub trait ContextraConfigExt {
    /// Returns `true` if storage encryption is configured with a non-empty passphrase.
    fn is_encrypted(&self) -> bool;
}

impl ContextraConfigExt for ContextraConfig {
    fn is_encrypted(&self) -> bool {
        self.encryption_passphrase
            .as_ref()
            .is_some_and(|pass| !pass.is_empty())
    }
}

/// A builder for configuring and instantiating `Contextra`.
///
/// `ContextraBuilder` acts as the primary builder pattern in the `contextra` facade (Ring 4).
/// It orchestrates subsystem configuration without containing domain business logic.
pub struct ContextraBuilder {
    storage_path: PathBuf,
    config: ContextraConfig,
    encryption_mode: Option<EncryptionMode>,
    encryption_error: Option<String>,
    explicit_user_config: bool,
    embedder: Option<Arc<dyn TextEmbeddingEngine>>,
    license_gate: Arc<dyn LicenseGate>,
    signed_license_error: Option<String>,
    performance_profile: Option<crate::performance_profile::PerformanceProfile>,
}

impl std::fmt::Debug for ContextraBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ContextraBuilder")
            .field("storage_path", &self.storage_path)
            .field("config", &self.config)
            .field("encryption_mode", &self.encryption_mode)
            .field("explicit_user_config", &self.explicit_user_config)
            .field(
                "embedder",
                &self.embedder.as_ref().map(|_| "<dyn TextEmbeddingEngine>"),
            )
            .field("license_gate", &"<dyn LicenseGate>")
            .field("signed_license_error", &self.signed_license_error)
            .field("performance_profile", &self.performance_profile)
            .finish()
    }
}

impl Clone for ContextraBuilder {
    fn clone(&self) -> Self {
        Self {
            storage_path: self.storage_path.clone(),
            config: self.config.clone(),
            encryption_mode: self.encryption_mode.clone(),
            encryption_error: self.encryption_error.clone(),
            explicit_user_config: self.explicit_user_config,
            embedder: self.embedder.clone(),
            license_gate: Arc::clone(&self.license_gate),
            signed_license_error: self.signed_license_error.clone(),
            performance_profile: self.performance_profile,
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
            encryption_mode: None,
            encryption_error: None,
            explicit_user_config: false,
            embedder: None,
            license_gate: Arc::new(OpenFastGate),
            signed_license_error: None,
            performance_profile: None,
        }
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

    /// Sets the storage encryption mode explicitly.
    pub fn with_encryption(mut self, mode: EncryptionMode) -> Self {
        match &mode {
            EncryptionMode::Passphrase(secret) => {
                if secret.is_empty() || secret.trim().is_empty() {
                    self.encryption_error =
                        Some("Encryption passphrase cannot be empty".to_string());
                    return self;
                }
            }
            EncryptionMode::Unencrypted => {}
        }
        self.encryption_mode = Some(mode);
        self.config.encryption_passphrase = None;
        self.encryption_error = None;
        self
    }

    /// Sets the encryption passphrase for persistent storage.
    ///
    /// Converts the passphrase immediately into a zeroized container (`Zeroizing<String>`)
    /// without creating intermediate un-zeroized heap clones.
    pub fn with_encryption_passphrase(self, passphrase: impl Into<String>) -> Self {
        let secret = Zeroizing::new(passphrase.into());
        self.with_encryption(EncryptionMode::Passphrase(secret))
    }

    /// Returns `true` if storage encryption is configured with a passphrase.
    pub fn is_encrypted(&self) -> bool {
        matches!(self.encryption_mode, Some(EncryptionMode::Passphrase(_)))
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
                    Ok(gate) => {
                        self.license_gate = Arc::new(gate);
                        self.signed_license_error = None;
                    }
                    Err(e) => {
                        self.signed_license_error = Some(e.to_string());
                    }
                }
            }
            Err(_) => {
                self.signed_license_error = Some(LicenseError::InvalidSignature.to_string());
            }
        }
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

    /// Sets the complete `ContextraConfig` directly.
    pub fn with_config(mut self, mut config: ContextraConfig) -> Self {
        if let Some(passphrase) = config.encryption_passphrase.take() {
            if !passphrase.is_empty() && !passphrase.trim().is_empty() {
                let secret = Zeroizing::new(passphrase);
                self.encryption_mode = Some(EncryptionMode::Passphrase(secret));
            } else {
                self.encryption_error = Some("Encryption passphrase cannot be empty".to_string());
            }
        }
        self.config = config;
        self.config.encryption_passphrase = None;
        self.explicit_user_config = true;
        self
    }

    /// Builds and initializes the `Contextra` engine instance.
    pub async fn build(self) -> Result<Contextra, ContextraError> {
        if let Some(ref err_msg) = self.encryption_error {
            return Err(ContextraError::invalid_input(err_msg.clone()));
        }

        if let Some(ref err_msg) = self.signed_license_error {
            return Err(ContextraError::PolicyViolation(err_msg.clone()));
        }

        let mut config_for_open = self.config.clone();

        match &self.encryption_mode {
            Some(EncryptionMode::Passphrase(secret)) => {
                if secret.is_empty() || secret.trim().is_empty() {
                    return Err(ContextraError::invalid_input(
                        "Encryption passphrase cannot be empty",
                    ));
                }
                config_for_open.encryption_passphrase = Some(secret.as_str().to_string());
            }
            Some(EncryptionMode::Unencrypted) => {
                config_for_open.encryption_passphrase = None;
            }
            None => {
                tracing::warn!(
                    "Contextra started in implicit unencrypted mode; explicit EncryptionMode selection will be required in a future version"
                );
                config_for_open.encryption_passphrase = None;
            }
        }

        let requested_ring = if let Some(profile) = self.performance_profile {
            let resolved = profile.resolve();
            if self.explicit_user_config {
                if config_for_open.durability_mode != resolved.durability_mode
                    || config_for_open.deletion_proof_active != resolved.deletion_proof_active
                    || config_for_open.vector_delete_mode != resolved.vector_delete_mode
                {
                    return Err(ContextraError::PolicyViolation(
                        "Explicit ContextraConfig conflicts with PerformanceProfile settings"
                            .to_string(),
                    ));
                }
            } else {
                config_for_open.durability_mode = resolved.durability_mode;
                config_for_open.deletion_proof_active = resolved.deletion_proof_active;
                config_for_open.vector_delete_mode = resolved.vector_delete_mode;
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
            config_for_open,
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
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_defaults() {
        let builder = ContextraBuilder::default();
        assert_eq!(builder.config.dimension, 768);
        assert_eq!(builder.storage_path, PathBuf::from("./contextra_data"));
        assert!(builder.embedder.is_none());
        assert!(!builder.is_encrypted());
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
        assert!(builder.is_encrypted());
        assert!(!builder.config.consolidation_enabled);
        assert_eq!(
            builder.config.consolidation_interval,
            Duration::from_secs(300)
        );
    }

    #[test]
    fn test_encryption_passphrase_redaction() {
        let secret_str = "super_secret_master_key_12345";
        let mode = EncryptionMode::Passphrase(Zeroizing::new(secret_str.to_string()));
        let debug_mode_output = format!("{:?}", mode);
        assert!(!debug_mode_output.contains(secret_str));
        assert!(debug_mode_output.contains("[REDACTED]"));

        let builder = ContextraBuilder::new(256).with_encryption_passphrase(secret_str);
        let debug_builder_output = format!("{:?}", builder);
        assert!(!debug_builder_output.contains(secret_str));
        assert!(debug_builder_output.contains("[REDACTED]"));
    }

    #[tokio::test]
    async fn test_empty_passphrase_rejected() {
        let res = ContextraBuilder::new(4)
            .with_encryption_passphrase("")
            .build()
            .await;
        assert!(res.is_err());
        if let Err(ContextraError::InvalidInput(msg)) = res {
            assert!(msg.contains("cannot be empty"));
        } else {
            panic!("Expected InvalidInput error for empty passphrase");
        }

        let res_ws = ContextraBuilder::new(4)
            .with_encryption(EncryptionMode::Passphrase(Zeroizing::new(
                "   ".to_string(),
            )))
            .build()
            .await;
        assert!(res_ws.is_err());
    }

    #[tokio::test]
    async fn test_explicit_unencrypted_mode() -> Result<(), Box<dyn std::error::Error>> {
        let tmp_path =
            std::env::temp_dir().join(format!("contextra_unencrypted_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp_path);

        let builder = ContextraBuilder::new(4)
            .with_storage_path(&tmp_path)
            .with_encryption(EncryptionMode::Unencrypted);

        assert!(!builder.is_encrypted());

        let db = builder.build().await?;
        assert_eq!(db.len().await?, 0);

        let _ = std::fs::remove_dir_all(&tmp_path);
        Ok(())
    }

    #[tokio::test]
    async fn test_implicit_unencrypted_mode_warning() -> Result<(), Box<dyn std::error::Error>> {
        use std::sync::{Arc, Mutex};
        use tracing_subscriber::layer::SubscriberExt;

        #[derive(Clone, Default)]
        struct StringWriter(Arc<Mutex<Vec<u8>>>);
        impl std::io::Write for StringWriter {
            fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                self.0.lock().unwrap().extend_from_slice(buf);
                Ok(buf.len())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }

        let logs = StringWriter::default();
        let logs_clone = logs.clone();
        let subscriber = tracing_subscriber::fmt::layer()
            .with_writer(move || logs_clone.clone())
            .with_ansi(false);
        let dispatch = tracing::Dispatch::new(tracing_subscriber::registry().with(subscriber));

        let tmp_path =
            std::env::temp_dir().join(format!("contextra_implicit_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp_path);

        {
            let _guard = tracing::dispatcher::set_default(&dispatch);
            let db = ContextraBuilder::new(4)
                .with_storage_path(&tmp_path)
                .build()
                .await?;
            assert_eq!(db.len().await?, 0);
        }

        let output = String::from_utf8(logs.0.lock().unwrap().clone())?;
        assert!(
            output.contains("implicit unencrypted mode"),
            "Expected warning log for implicit unencrypted mode, got: {}",
            output
        );

        let _ = std::fs::remove_dir_all(&tmp_path);
        Ok(())
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
