//! MemFuse — Embedded hybrid-search for AI agents (Facade)
//!
//! This crate provides the primary Composition Root for MemFuse.

#![forbid(unsafe_code)]

pub use memfuse_core::error::MemFuseError;
pub use memfuse_core::types::domain::{DocId, ScoredDocument};
pub use memfuse_db::{
    chunker, execute_background_consolidation, memory_consolidation, Collection, CollectionConfig,
    DriftStatusProvider, MemFuse, MemFuseConfig,
};

#[cfg(feature = "router")]
pub use memfuse_router as router;

#[cfg(feature = "router")]
pub use memfuse_calibration as calibration;

#[cfg(feature = "ollama")]
pub use memfuse_ollama as ollama;

/// A builder for creating a `MemFuse` instance.
pub struct MemFuseBuilder {
    storage_path: std::path::PathBuf,
    config: MemFuseConfig,
}

impl MemFuseBuilder {
    /// Creates a new builder for MemFuse with the given dimension.
    pub fn new(dimension: usize) -> Self {
        let mut config = MemFuseConfig::default();
        config.dimension = dimension;
        Self {
            storage_path: std::path::PathBuf::from("./memfuse_data"),
            config,
        }
    }

    /// Sets the path for the underlying storage engine.
    pub fn with_storage_path(mut self, path: impl Into<std::path::PathBuf>) -> Self {
        self.storage_path = path.into();
        self
    }

    /// Builds the `MemFuse` instance.
    pub async fn build(self) -> Result<MemFuse, MemFuseError> {
        MemFuse::open_with_config(self.storage_path, self.config).await
    }
}
