// FILE-CONTEXT
// ZWECK: MemFuse Composition Root Builder (Ring 4).
// INVARIANTEN: Zero-panic in production code; orchestriert Subsystem-Konfiguration ohne Geschäftslogik.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use memfuse_core::error::MemFuseError;
use memfuse_core::DistanceMetric;
use memfuse_db::{EmbeddingBackend, MemFuse, MemFuseConfig, TextEmbeddingEngine};

/// A builder for configuring and instantiating `MemFuse`.
///
/// `MemFuseBuilder` acts as the primary builder pattern in the `memfuse` facade (Ring 4).
/// It orchestrates subsystem configuration without containing domain business logic.
pub struct MemFuseBuilder {
    storage_path: PathBuf,
    config: MemFuseConfig,
    embedder: Option<Arc<dyn TextEmbeddingEngine>>,
}

impl std::fmt::Debug for MemFuseBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MemFuseBuilder")
            .field("storage_path", &self.storage_path)
            .field("config", &self.config)
            .field(
                "embedder",
                &self
                    .embedder
                    .as_ref()
                    .map(|_| "<dyn TextEmbeddingEngine>"),
            )
            .finish()
    }
}

impl Clone for MemFuseBuilder {
    fn clone(&self) -> Self {
        Self {
            storage_path: self.storage_path.clone(),
            config: self.config.clone(),
            embedder: self.embedder.clone(),
        }
    }
}

impl MemFuseBuilder {
    /// Creates a new `MemFuseBuilder` with specified vector dimension.
    pub fn new(dimension: usize) -> Self {
        let mut config = MemFuseConfig::default();
        config.dimension = dimension;
        Self {
            storage_path: PathBuf::from("./memfuse_data"),
            config,
            embedder: None,
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

    /// Sets the complete `MemFuseConfig` directly.
    pub fn with_config(mut self, config: MemFuseConfig) -> Self {
        self.config = config;
        self
    }

    /// Builds and initializes the `MemFuse` engine instance.
    pub async fn build(self) -> Result<MemFuse, MemFuseError> {
        let instance = MemFuse::open_with_config(&self.storage_path, self.config).await?;
        if let Some(embedder) = self.embedder {
            Ok(instance.with_embedder(embedder).await)
        } else {
            Ok(instance)
        }
    }
}

impl Default for MemFuseBuilder {
    fn default() -> Self {
        Self::new(768)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_defaults() {
        let builder = MemFuseBuilder::default();
        assert_eq!(builder.config.dimension, 768);
        assert_eq!(builder.storage_path, PathBuf::from("./memfuse_data"));
        assert!(builder.embedder.is_none());
    }

    #[test]
    fn test_builder_custom_configuration() {
        let builder = MemFuseBuilder::new(128)
            .with_storage_path("/tmp/test_memfuse_db")
            .with_max_elements(50000)
            .with_distance_metric(DistanceMetric::Euclidean)
            .with_encryption_passphrase("secret_pass")
            .with_consolidation(false, Duration::from_secs(300));

        assert_eq!(builder.config.dimension, 128);
        assert_eq!(builder.storage_path, PathBuf::from("/tmp/test_memfuse_db"));
        assert_eq!(builder.config.max_elements, 50000);
        assert_eq!(builder.config.distance_metric, DistanceMetric::Euclidean);
        assert_eq!(
            builder.config.encryption_passphrase,
            Some("secret_pass".to_string())
        );
        assert!(!builder.config.consolidation_enabled);
        assert_eq!(builder.config.consolidation_interval, Duration::from_secs(300));
    }

    #[tokio::test]
    async fn test_builder_build_roundtrip() {
        let tmp_path = std::env::temp_dir().join(format!("memfuse_builder_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp_path);

        let db = MemFuseBuilder::new(4)
            .with_storage_path(&tmp_path)
            .with_distance_metric(DistanceMetric::Cosine)
            .build()
            .await
            .expect("build db");

        assert_eq!(db.len().await.expect("len"), 0);
        let _ = std::fs::remove_dir_all(&tmp_path);
    }
}
