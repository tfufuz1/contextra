// FILE-CONTEXT
// ZWECK: MemFuse Primary Facade & Composition Root (Ring 4).
// INVARIANTEN: #![forbid(unsafe_code)]; <= 20 pub fn; zero direct dependencies on Ring 0/1 internal crates.

#![forbid(unsafe_code)]

pub mod agent_memory;
pub mod builder;

pub use agent_memory::{AgentMemory, Memory, MemoryId};
pub use builder::MemFuseBuilder;
pub use memfuse_core::error::MemFuseError;
pub use memfuse_core::types::domain::{DocId, ScoredDocument};
pub use memfuse_core::DistanceMetric;
pub use memfuse_db::{
    chunker, execute_background_consolidation, memory_consolidation, Collection, CollectionConfig,
    DriftStatusProvider, EmbeddingBackend, MemFuse, MemFuseConfig, MemFuseStats, SearchResult,
    TextEmbeddingEngine,
};

#[cfg(feature = "router")]
pub use memfuse_router as router;

#[cfg(feature = "router")]
pub use memfuse_rank as rank;

#[cfg(feature = "ollama")]
pub use memfuse_infer_ollama as ollama;

#[cfg(feature = "candle")]
pub use memfuse_infer_candle as candle;

#[cfg(feature = "onnx")]
pub use memfuse_infer_onnx as onnx;

/// Creates a new `MemFuseBuilder` for configuring and instantiating `MemFuse`.
pub fn builder(dimension: usize) -> MemFuseBuilder {
    MemFuseBuilder::new(dimension)
}

/// Opens or creates a `MemFuse` instance at the given storage path using default configuration.
pub async fn open(path: impl AsRef<std::path::Path>) -> Result<MemFuse, MemFuseError> {
    MemFuse::open(path).await
}

/// Opens or creates a `MemFuse` instance at the given storage path with an explicit configuration.
pub async fn open_with_config(
    path: impl AsRef<std::path::Path>,
    config: MemFuseConfig,
) -> Result<MemFuse, MemFuseError> {
    MemFuse::open_with_config(path, config).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_facade_open_and_builder_entrypoints() {
        let base_tmp =
            std::env::temp_dir().join(format!("memfuse_facade_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base_tmp);

        let db1_path = base_tmp.join("db1");
        let db1 = open(&db1_path).await.expect("open db1");
        assert_eq!(db1.len().await.expect("len"), 0);

        let mut config = MemFuseConfig::default();
        config.dimension = 16;
        let db2_path = base_tmp.join("db2");
        let db2 = open_with_config(&db2_path, config).await.expect("open db2");
        assert_eq!(db2.len().await.expect("len"), 0);

        let db3_path = base_tmp.join("db3");
        let db3 = builder(32)
            .with_storage_path(&db3_path)
            .build()
            .await
            .expect("builder db3");
        assert_eq!(db3.len().await.expect("len"), 0);

        let _ = std::fs::remove_dir_all(&base_tmp);
    }
}
