// FILE-CONTEXT
// ZWECK: Contextra Core Engine Orchestrator & Facade (Layer 3 - Engine).
// INVARIANTEN: Monoton steigende TxId-Allokation; Reparaturgarantie beim Öffnen (repair_on_open); Strikte Isolation von Namespaces.
// NICHT-OFFENSICHTLICH: Lock-Hierarchie: collections (RwLock) -> kv_locks -> embedder (RwLock).

#![forbid(unsafe_code)]

#[cfg(feature = "sandbox")]
use contextra_core::BoxFuture;
pub use contextra_core::TextEmbeddingEngine;
use contextra_core::{CollectionId, DocId, StorageEngine, TenantId};
use contextra_crypto::deletion_proof::{
    DeletionLayer, DeletionProof, DeletionScope, LayerCleanupProof,
};
use contextra_store::LsmStorage;
use contextra_vector::{HnswConfig, HnswIndex};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

pub mod background_workers;
pub mod chunker;
pub mod collection;
pub mod decay_controller;
pub mod export;
pub mod filter;
pub mod fusion;
pub mod import;
pub mod temporal_filter;
pub mod transaction;

pub use decay_controller::{AdaptiveDecayController, DecayControllerConfig, DecaySignalInputs};

pub use export::{
    ExportCollectionV1, ExportDocumentV1, ExportMemoryV1, ExportRelationV1, SCHEMA_VERSION_V1,
};
pub use import::ImportSummary;

#[cfg(feature = "background-maintenance")]
pub use background_workers::start_decay_cleanup_worker;
pub use background_workers::{start_expiry_cleanup_worker, start_orphan_cleanup_worker};

pub use collection::crud::MAX_SCAN_RESULTS;
#[cfg(feature = "graph-connectivity-health")]
pub use collection::maintenance::PercolationResult;
pub use collection::query_builder::{HybridQueryBuilder, SearchStrategy, SignalWeights};
pub use collection::{Collection, CollectionConfig};
#[allow(deprecated)]
pub use filter::MetadataFilter;
pub use contextra_checkpoint;
#[cfg(feature = "graph-connectivity-health")]
pub use contextra_graph::percolation::PercolationConfig;
pub use contextra_text::Language;

#[allow(clippy::type_complexity)]
static CONSOLIDATION_LAUNCHER: parking_lot::RwLock<
    Option<
        Arc<
            dyn Fn(
                    Arc<Collection<LsmStorage>>,
                    std::time::Duration,
                    usize,
                    tokio_util::sync::CancellationToken,
                ) -> tokio::task::JoinHandle<()>
                + Send
                + Sync,
        >,
    >,
> = parking_lot::RwLock::new(None);

/// Registers a consolidation worker launcher function (used by `contextra-cognition`).
pub fn register_consolidation_launcher<F>(f: F)
where
    F: Fn(
            Arc<Collection<LsmStorage>>,
            std::time::Duration,
            usize,
            tokio_util::sync::CancellationToken,
        ) -> tokio::task::JoinHandle<()>
        + Send
        + Sync
        + 'static,
{
    *CONSOLIDATION_LAUNCHER.write() = Some(Arc::new(f));
}

pub use fusion::{ProvenanceRecord, SearchResult, SignalContribution};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextraStats {
    pub drift_status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub calibration_ece: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_calibration_at: Option<u64>,
    pub active_memory_count: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pid_pool_size: Option<usize>,
    pub index_stats: contextra_core::VectorIndexStats,
    pub storage_stats: contextra_core::StorageStats,
}

pub use contextra_core::DriftStatusProvider;
pub type DbStats = ContextraStats;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityDetectionConfig {
    pub auto_trigger_threshold: u64,
}

impl Default for CommunityDetectionConfig {
    fn default() -> Self {
        Self {
            auto_trigger_threshold: 100,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmbeddingBackend {
    Onnx {
        model_name: String,
        cache_dir: Option<std::path::PathBuf>,
    },
    None,
}

impl Default for EmbeddingBackend {
    fn default() -> Self {
        Self::Onnx {
            model_name: "nomic-embed-text".to_string(),
            cache_dir: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ContextraConfig {
    pub dimension: usize,
    pub max_elements: usize,
    pub distance_metric: contextra_core::DistanceMetric,
    pub encryption_passphrase: Option<String>,
    pub expiry_reaper_interval: std::time::Duration,
    pub orphan_registry_path: Option<std::path::PathBuf>,
    pub community_detection: CommunityDetectionConfig,
    pub embedding_backend: EmbeddingBackend,
    pub consolidation_enabled: bool,
    pub consolidation_interval: std::time::Duration,
    pub max_llm_calls_per_cycle: usize,
}

impl Default for ContextraConfig {
    fn default() -> Self {
        Self {
            dimension: 768,
            max_elements: 1_000_000,
            distance_metric: contextra_core::DistanceMetric::Cosine,
            encryption_passphrase: None,
            expiry_reaper_interval: std::time::Duration::from_secs(60),
            orphan_registry_path: None,
            community_detection: CommunityDetectionConfig::default(),
            embedding_backend: EmbeddingBackend::default(),
            consolidation_enabled: true,
            consolidation_interval: std::time::Duration::from_secs(6 * 3600),
            max_llm_calls_per_cycle: 10,
        }
    }
}

pub struct Contextra {
    storage: Arc<LsmStorage>,
    next_tx: Arc<AtomicU64>,
    dimension: usize,
    expiry_reaper_interval: std::time::Duration,
    community_detection_threshold: u64,
    collections: tokio::sync::RwLock<ahash::AHashMap<String, Arc<Collection<LsmStorage>>>>,
    cancel_token: tokio_util::sync::CancellationToken,
    task_tracker: tokio_util::task::TaskTracker,
    embedder: parking_lot::RwLock<Option<Arc<dyn TextEmbeddingEngine>>>,
    orphan_registry: Arc<contextra_checkpoint::InstanceOrphanRegistry>,
    router: parking_lot::RwLock<Option<std::sync::Weak<dyn DriftStatusProvider>>>,
    calibrator: parking_lot::RwLock<
        Option<std::sync::Weak<parking_lot::Mutex<contextra_rank::IsotonicCalibrator>>>,
    >,
    pid_controller: parking_lot::RwLock<
        Option<std::sync::Weak<parking_lot::Mutex<contextra_adapt::PidController>>>,
    >,
}


mod contextra_impl;


pub use contextra_core::DistanceMetric;
pub use serde_json::json;

impl Contextra {
    #[doc(hidden)]
    pub fn inner_storage(&self) -> Arc<LsmStorage> {
        self.storage.clone()
    }
}

#[cfg(feature = "sandbox")]
pub trait SandboxBridge: Send + Sync {
    fn db_search<'a>(&'a self, query: &'a [u8], k: usize) -> BoxFuture<'a, Result<Vec<u8>>>;
    fn db_insert<'a>(&'a self, key: &'a [u8], value: &'a [u8]) -> BoxFuture<'a, Result<()>>;
    fn db_get<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, Result<Option<Vec<u8>>>>;
}

#[cfg(feature = "sandbox")]
impl SandboxBridge for Contextra {
    fn db_search<'a>(&'a self, query: &'a [u8], k: usize) -> BoxFuture<'a, Result<Vec<u8>>> {
        Box::pin(async move {
            let f32_count = query.len() / 4;
            let mut vector = Vec::with_capacity(f32_count);
            for i in 0..f32_count {
                let start = i * 4;
                let bits = u32::from_le_bytes(
                    query
                        .get(start..start + 4)
                        .ok_or_else(|| {
                            contextra_core::ContextraError::Serialization("Query too short".into())
                        })?
                        .try_into()
                        .map_err(|_| {
                            contextra_core::ContextraError::Serialization("Invalid slice".into())
                        })?,
                );
                vector.push(f32::from_bits(bits));
            }

            let results: Vec<SearchResult> = self.search(&vector, k).await?;
            serde_json::to_vec(&results)
                .map_err(|e| contextra_core::ContextraError::Internal(e.to_string()))
        })
    }

    fn db_insert<'a>(&'a self, key: &'a [u8], value: &'a [u8]) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            let id = String::from_utf8_lossy(key).to_string();
            let val_json: Value = serde_json::from_slice(value)
                .unwrap_or(serde_json::json!({ "raw_data": String::from_utf8_lossy(value) }));

            self.insert(&id, &[], Some(val_json)).await
        })
    }

    fn db_get<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, Result<Option<Vec<u8>>>> {
        Box::pin(async move {
            let id = String::from_utf8_lossy(key).to_string();
            let doc = self.get(&id).await?;
            match doc {
                Some(d) => {
                    Ok(Some(serde_json::to_vec(&d).map_err(|e| {
                        contextra_core::ContextraError::Internal(e.to_string())
                    })?))
                }
                None => Ok(None),
            }
        })
    }
}
