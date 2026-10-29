// FILE-CONTEXT
// ZWECK: Produktive Adapter-Implementierungen für Port-Traits (`KvBridgeStorage`, `SemanticEmbedder`, `PluginManifest`).

//! Produktive Port-Adapter für den Contextra KV-Cache.
//!
//! Dieses Modul stellt konkrete Implementierungen der Ring-0 Port-Traits bereit:
//! - [`StorageEngineKvBridge`]: Verbindet Tier-2 Persistence über den generischen [`StorageEngine`]-Port.
//! - [`EmbeddingProviderSemanticEmbedder`]: Adaptiert den [`EmbeddingProvider`]-Port für semantische Cache-Lookups.
//! - [`KvCachePluginManifest`]: Bietet ein validiertes Manifest mit Capabilities, Limits und Versionierung.

use std::sync::Arc;

use bytes::Bytes;
use contextra_ports::embedding::EmbeddingProvider;
use contextra_ports::kv_bridge_port::KvBridgeStorage;
use contextra_ports::license::FeatureRing;
use contextra_ports::plugin::{PluginCapability, PluginError, PluginManifest};
use contextra_ports::storage::StorageEngine;
use contextra_ports::BoxFuture;
use contextra_types::{ContextraError, Result as ContextraResult, TxId};

#[cfg(feature = "content-addressed-kv-cache")]
use crate::radix::SemanticEmbedder;

#[cfg(feature = "content-addressed-kv-cache")]
fn block_on_simple<F: std::future::Future>(fut: F) -> F::Output {
    let waker = std::task::Waker::noop();
    let mut cx = std::task::Context::from_waker(waker);
    let mut pinned = std::pin::pin!(fut);
    loop {
        match pinned.as_mut().poll(&mut cx) {
            std::task::Poll::Ready(val) => return val,
            std::task::Poll::Pending => std::thread::yield_now(),
        }
    }
}

/// Produktiver Tier-2 Spill-over Adapter über das generische [`StorageEngine`]-Interface.
///
/// Bietet mandantengetrennte Persistierung von KV-Segmenten im Storage-Layer.
pub struct StorageEngineKvBridge {
    engine: Arc<dyn StorageEngine>,
}

impl StorageEngineKvBridge {
    /// Erstellt eine neue `StorageEngineKvBridge`.
    pub fn new(engine: Arc<dyn StorageEngine>) -> Self {
        Self { engine }
    }
}

impl KvBridgeStorage for StorageEngineKvBridge {
    fn get<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, ContextraResult<Option<Bytes>>> {
        Box::pin(async move { self.engine.get(key).await })
    }

    fn put<'a>(
        &'a self,
        tx_id: TxId,
        key: &'a [u8],
        value: &'a [u8],
    ) -> BoxFuture<'a, ContextraResult<()>> {
        Box::pin(async move { self.engine.put(tx_id, key, value).await })
    }

    fn commit<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, ContextraResult<()>> {
        Box::pin(async move { self.engine.commit(tx_id).await })
    }
}

/// Adaptiert einen generic [`EmbeddingProvider`] für den `SemanticEmbedder`-Trait im KV-Cache.
pub struct EmbeddingProviderSemanticEmbedder {
    provider: Arc<dyn EmbeddingProvider>,
}

impl EmbeddingProviderSemanticEmbedder {
    /// Erstellt einen neuen `EmbeddingProviderSemanticEmbedder`.
    pub fn new(provider: Arc<dyn EmbeddingProvider>) -> Self {
        Self { provider }
    }

    /// Returns a reference to the underlying `EmbeddingProvider`.
    pub fn provider(&self) -> &Arc<dyn EmbeddingProvider> {
        &self.provider
    }
}

#[cfg(feature = "content-addressed-kv-cache")]
impl SemanticEmbedder for EmbeddingProviderSemanticEmbedder {
    fn embed(&self, tokens: &[u32]) -> ContextraResult<Vec<f32>> {
        if tokens.is_empty() {
            return Err(ContextraError::InvalidInput(
                "Cannot compute embedding for empty token sequence".to_string(),
            ));
        }

        let token_text = tokens
            .iter()
            .map(|t| t.to_string())
            .collect::<Vec<_>>()
            .join(" ");

        let provider = Arc::clone(&self.provider);
        let handle = std::thread::spawn(move || {
            let future = provider.embed(&token_text);
            block_on_simple(future)
        });

        handle.join().map_err(|_| {
            ContextraError::Internal("Thread panic during embedding computation".to_string())
        })?
    }
}

/// Validierte Konfiguration für das KV-Cache-Plugin-Manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KvCachePluginConfig {
    /// Plugin version tuple (major, minor, patch).
    pub version: (u32, u32, u32),
    /// Max RAM memory limit in bytes for KV-cache allocation.
    pub max_memory_bytes: u64,
    /// Minimum required feature ring.
    pub feature_ring_required: FeatureRing,
}

impl Default for KvCachePluginConfig {
    fn default() -> Self {
        Self {
            version: (1, 0, 0),
            max_memory_bytes: 512 << 20, // 512 MiB
            feature_ring_required: FeatureRing::Fast,
        }
    }
}

impl KvCachePluginConfig {
    /// Validates the plugin configuration parameters.
    pub fn validate(&self) -> ContextraResult<()> {
        if self.max_memory_bytes < (1 << 20) {
            return Err(ContextraError::InvalidInput(
                "max_memory_bytes must be at least 1 MiB".to_string(),
            ));
        }
        Ok(())
    }
}

/// Concrete, validated [`PluginManifest`] implementation for `contextra-kvcache`.
#[derive(Debug, Clone)]
pub struct KvCachePluginManifest {
    config: KvCachePluginConfig,
}

impl KvCachePluginManifest {
    /// Constructs a new validated `KvCachePluginManifest`.
    pub fn new(config: KvCachePluginConfig) -> ContextraResult<Self> {
        config.validate()?;
        Ok(Self { config })
    }
}

impl PluginManifest for KvCachePluginManifest {
    fn name(&self) -> &str {
        "contextra-kvcache"
    }

    fn requires(&self) -> &[&str] {
        &[]
    }

    fn conflicts(&self) -> &[&str] {
        &[]
    }

    fn feature_ring_required(&self) -> FeatureRing {
        self.config.feature_ring_required
    }

    fn capability(&self) -> PluginCapability {
        PluginCapability {
            name: "contextra-kvcache",
            version: self.config.version,
            ring: 1,
            feature_ring_required: self.config.feature_ring_required,
        }
    }

    fn activate(&self) -> Result<(), PluginError> {
        tracing::info!(
            version = ?self.config.version,
            max_memory_bytes = self.config.max_memory_bytes,
            "Activated contextra-kvcache plugin manifest"
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kv_cache_plugin_manifest_validation() {
        let invalid_config = KvCachePluginConfig {
            version: (1, 0, 0),
            max_memory_bytes: 100, // < 1 MiB -> Invalid
            feature_ring_required: FeatureRing::Fast,
        };
        assert!(KvCachePluginManifest::new(invalid_config).is_err());

        let valid_config = KvCachePluginConfig::default();
        if let Ok(manifest) = KvCachePluginManifest::new(valid_config) {
            assert_eq!(manifest.name(), "contextra-kvcache");
            assert_eq!(manifest.capability().ring, 1);
            assert!(manifest.activate().is_ok());
        }
    }
}
