// FILE-CONTEXT
// ZWECK: Contextra Core Engine Orchestrator & Facade (Layer 3 - Engine).
// INVARIANTEN: Monoton steigende TxId-Allokation; Reparaturgarantie beim Öffnen (repair_on_open); Strikte Isolation von Namespaces.
// NICHT-OFFENSICHTLICH: Lock-Hierarchie: collections (RwLock) -> kv_locks -> embedder (RwLock).

#![forbid(unsafe_code)]

pub use contextra_crypto::deletion_proof::{
    DeletionLayer, DeletionProof, DeletionScope, ExcludedScope, LayerCleanupProof,
};

pub mod deletion;
use contextra_ports::StorageEngine;
pub use contextra_ports::TextEmbeddingEngine;
pub use contextra_ports::VectorDeleteMode;
#[cfg(not(loom))]
pub use contextra_store::lsm::config::{EncryptionConfig, KekSource};
#[cfg(not(loom))]
pub use contextra_store::lsm::DurabilityMode;
#[cfg(not(loom))]
use contextra_store::LsmStorage;
use contextra_types::{CollectionId, ContextraError, DocId, TenantId};
use contextra_vector::{HnswConfig, HnswIndex};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

#[cfg(not(loom))]
pub mod background_workers;
#[cfg(not(loom))]
pub mod chunker;
pub mod collection;
#[cfg(not(loom))]
pub mod decay_controller;
#[cfg(not(loom))]
pub mod export;
#[cfg(not(loom))]
pub mod filter;
#[cfg(not(loom))]
pub mod fusion;
#[cfg(not(loom))]
pub mod import;
#[cfg(not(loom))]
pub mod lifecycle;
pub mod limits;
pub mod metrics_names;
pub mod migration;
#[cfg(not(loom))]
pub mod temporal_filter;
#[cfg(not(loom))]
pub mod transaction;

pub use limits::EngineLimits;

#[cfg(not(loom))]
pub use export::{
    ExportCollectionV1, ExportDocumentV1, ExportMemoryV1, ExportRelationV1, SCHEMA_VERSION_V1,
};
#[cfg(not(loom))]
pub use import::ImportSummary;

#[cfg(not(loom))]
#[cfg(feature = "background-maintenance")]
pub use background_workers::start_decay_cleanup_worker;
#[cfg(not(loom))]
pub use background_workers::{start_expiry_cleanup_worker, start_orphan_cleanup_worker};
#[cfg(not(loom))]
pub use lifecycle::{EngineLifecycle, ErasureSupervisor};

#[cfg(not(loom))]
pub use collection::crud::MAX_SCAN_RESULTS;
#[cfg(not(loom))]
pub use collection::query_builder::{HybridQueryBuilder, SearchStrategy, SignalWeights};
pub use collection::search::hybrid::{HybridPipelineStage, HYBRID_PIPELINE_STAGES};
#[cfg(not(loom))]
pub use collection::{Collection, CollectionConfig};
pub use contextra_checkpoint;
pub use contextra_text::Language;
pub use deletion::{
    forget_document, reverse_sweep, DeletionWiring, DeletionWiringDeps, DpiDeletionRuntime,
    DpiPurgeDriver, DpiPurgeDriverConfig, FonLogicalHandle, ForgetOptions, ForgetOutcome,
    PurgeHandle, VerifyDepth, Wait,
};
#[cfg(not(loom))]
#[allow(deprecated)]
pub use filter::MetadataFilter;

#[cfg(not(loom))]
#[allow(clippy::type_complexity)]
pub type ConsolidationLauncher = Arc<
    dyn Fn(
            Arc<Collection<LsmStorage>>,
            std::time::Duration,
            usize,
            tokio_util::sync::CancellationToken,
        ) -> tokio::task::JoinHandle<()>
        + Send
        + Sync,
>;

/// Registers a consolidation worker launcher function (deprecated: set on `ContextraConfig` instead).
#[cfg(not(loom))]
#[deprecated(
    note = "Instanzgebundenen Launcher via ContextraConfig::with_consolidation_launcher setzen (P29)"
)]
pub fn register_consolidation_launcher<F>(_f: F)
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
    tracing::warn!(
        "register_consolidation_launcher is deprecated and has no effect. Set consolidation_launcher on ContextraConfig instead."
    );
}

#[cfg(not(loom))]
pub use fusion::{ProvenanceRecord, SearchResult, SignalContribution};

#[cfg(not(loom))]
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
    pub index_stats: contextra_ports::VectorIndexStats,
    pub storage_stats: contextra_ports::StorageStats,
}

pub use contextra_impl::{ForgetReceipt, LayerStatus};

pub use contextra_ports::{DriftStatusProvider, ReadSnapshot};
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
    Candle {
        model_dir: std::path::PathBuf,
    },
    Ollama {
        endpoint: String,
        model: String,
    },
    None,
}

impl std::str::FromStr for EmbeddingBackend {
    type Err = ContextraError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().trim() {
            "candle" => Ok(Self::Candle {
                model_dir: std::path::PathBuf::new(),
            }),
            "onnx" => Ok(Self::Onnx {
                model_name: "nomic-embed-text".to_string(),
                cache_dir: None,
            }),
            "ollama" => Ok(Self::Ollama {
                endpoint: "http://localhost:11434".to_string(),
                model: "nomic-embed-text".to_string(),
            }),
            "none" => Ok(Self::None),
            other => Err(ContextraError::InvalidInput(format!(
                "Unknown embedding backend '{other}'. Expected 'candle', 'onnx', 'ollama', or 'none'."
            ))),
        }
    }
}

/// Tenant isolation enforcement policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TenantPolicy {
    /// Non-tenant collection access (`collection(...)`) is allowed (Default / Admin / Single-tenant path).
    #[default]
    Optional,
    /// Non-tenant collection access is forbidden; callers MUST use tenant-scoped handles (`collection_for_tenant(...)`).
    Required,
}

#[cfg(not(loom))]
#[derive(Clone)]
pub struct ContextraConfig {
    pub dimension: usize,
    pub max_elements: usize,
    pub distance_metric: contextra_types::DistanceMetric,
    pub encryption: EncryptionConfig,
    /// Maximum RAM the LSM storage engine may use (in MB). Passed directly to LsmConfig::max_ram_mb.
    /// Default: 2048 (matches LsmConfig default).
    pub max_ram_mb: u64,

    /// WAL group-commit window in microseconds. 0 = disabled (immediate single commit).
    /// Passed directly to LsmConfig::group_commit_window_micros.
    /// Default: 500 (matches LsmConfig default).
    pub group_commit_window_micros: u64,

    /// Durability mode for WAL persistence. Passed directly to LsmConfig::durability_mode.
    /// Default: DurabilityMode::Full.
    pub durability_mode: DurabilityMode,

    /// Controls operational erasure mode (§4.6).
    /// Default: ErasureMode::Off.
    pub erasure_mode: contextra_ports::erasure::ErasureMode,

    /// Controls vector index deletion repair behavior.
    /// Default: VectorDeleteMode::BackgroundRepair.
    pub vector_delete_mode: VectorDeleteMode,

    /// MemTable flush threshold in bytes. Passed directly to LsmConfig::memtable_size_limit.
    /// Default: 67_108_864 (64 MiB, matches LsmConfig default).
    pub memtable_size_limit: usize,
    pub expiry_reaper_interval: std::time::Duration,
    pub orphan_registry_path: Option<std::path::PathBuf>,
    pub community_detection: CommunityDetectionConfig,
    pub embedding_backend: EmbeddingBackend,
    pub consolidation_enabled: bool,
    pub consolidation_interval: std::time::Duration,
    pub max_llm_calls_per_cycle: usize,
    pub consolidation_launcher: Option<ConsolidationLauncher>,
    /// Policy controlling whether multi-tenant isolation is required.
    pub tenant_policy: TenantPolicy,
    /// Configured metrics sink for recording engine operational metrics.
    pub metrics_sink: Option<Arc<dyn contextra_ports::MetricsSink>>,
    /// Custom merge operator for compaction / flush.
    pub merge_operator: Option<Arc<dyn contextra_store::MergeOperator>>,
}

#[cfg(not(loom))]
impl ContextraConfig {
    pub fn with_encryption_passphrase(mut self, passphrase: impl Into<String>) -> Self {
        self.encryption = EncryptionConfig::Passphrase(passphrase.into());
        self
    }

    pub fn with_encryption_config(mut self, encryption: EncryptionConfig) -> Self {
        self.encryption = encryption;
        self
    }

    pub fn encryption_passphrase(&self) -> Option<String> {
        self.encryption.passphrase()
    }

    pub fn with_consolidation_launcher(mut self, launcher: ConsolidationLauncher) -> Self {
        self.consolidation_launcher = Some(launcher);
        self
    }

    pub fn with_tenant_policy(mut self, policy: TenantPolicy) -> Self {
        self.tenant_policy = policy;
        self
    }

    pub fn with_metrics_sink(mut self, sink: Arc<dyn contextra_ports::MetricsSink>) -> Self {
        self.metrics_sink = Some(sink);
        self
    }

    pub fn with_merge_operator(mut self, op: Arc<dyn contextra_store::MergeOperator>) -> Self {
        self.merge_operator = Some(op);
        self
    }
}

#[cfg(not(loom))]
impl std::fmt::Debug for ContextraConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ContextraConfig")
            .field("dimension", &self.dimension)
            .field("max_elements", &self.max_elements)
            .field("distance_metric", &self.distance_metric)
            .field("encryption", &self.encryption)
            .field("max_ram_mb", &self.max_ram_mb)
            .field(
                "group_commit_window_micros",
                &self.group_commit_window_micros,
            )
            .field("durability_mode", &self.durability_mode)
            .field("erasure_mode", &self.erasure_mode)
            .field("vector_delete_mode", &self.vector_delete_mode)
            .field("memtable_size_limit", &self.memtable_size_limit)
            .field("expiry_reaper_interval", &self.expiry_reaper_interval)
            .field("orphan_registry_path", &self.orphan_registry_path)
            .field("community_detection", &self.community_detection)
            .field("embedding_backend", &self.embedding_backend)
            .field("consolidation_enabled", &self.consolidation_enabled)
            .field("consolidation_interval", &self.consolidation_interval)
            .field("max_llm_calls_per_cycle", &self.max_llm_calls_per_cycle)
            .field(
                "consolidation_launcher",
                &self.consolidation_launcher.as_ref().map(|_| "Fn(...)"),
            )
            .field("tenant_policy", &self.tenant_policy)
            .field(
                "metrics_sink",
                &self.metrics_sink.as_ref().map(|_| "dyn MetricsSink"),
            )
            .field(
                "merge_operator",
                &self.merge_operator.as_ref().map(|_| "dyn MergeOperator"),
            )
            .finish()
    }
}

#[cfg(not(loom))]
impl Default for ContextraConfig {
    fn default() -> Self {
        Self {
            dimension: 768,
            max_elements: 1_000_000,
            distance_metric: contextra_types::DistanceMetric::Cosine,
            encryption: EncryptionConfig::default(),
            max_ram_mb: 2048,
            group_commit_window_micros: 500,
            durability_mode: DurabilityMode::default(),
            erasure_mode: contextra_ports::erasure::ErasureMode::Off,
            vector_delete_mode: VectorDeleteMode::BackgroundRepair,
            memtable_size_limit: 64 * 1024 * 1024,
            expiry_reaper_interval: std::time::Duration::from_secs(60),
            orphan_registry_path: None,
            community_detection: CommunityDetectionConfig::default(),
            embedding_backend: EmbeddingBackend::None,
            consolidation_enabled: false,
            consolidation_interval: std::time::Duration::from_secs(6 * 3600),
            max_llm_calls_per_cycle: 10,
            consolidation_launcher: None,
            tenant_policy: TenantPolicy::Optional,
            metrics_sink: None,
            merge_operator: None,
        }
    }
}

#[cfg(not(loom))]
pub type TenantCollectionMap = ahash::AHashMap<
    (TenantId, String),
    Arc<Collection<contextra_store::tenant_codec::TenantScopedStorage<Arc<LsmStorage>>>>,
>;

#[cfg(not(loom))]
pub struct Contextra {
    storage: Arc<LsmStorage>,
    next_tx: Arc<AtomicU64>,
    dimension: usize,
    config: ContextraConfig,
    expiry_reaper_interval: std::time::Duration,
    community_detection_threshold: u64,
    collections: tokio::sync::RwLock<ahash::AHashMap<String, Arc<Collection<LsmStorage>>>>,
    tenant_collections: tokio::sync::RwLock<TenantCollectionMap>,
    cancel_token: tokio_util::sync::CancellationToken,
    task_tracker: tokio_util::task::TaskTracker,
    embedder: parking_lot::RwLock<Option<Arc<dyn TextEmbeddingEngine>>>,
    orphan_registry: Arc<contextra_checkpoint::InstanceOrphanRegistry>,
    router: parking_lot::RwLock<Option<std::sync::Weak<dyn DriftStatusProvider>>>,
    calibrator: parking_lot::RwLock<
        Option<std::sync::Weak<parking_lot::Mutex<contextra_rank::IsotonicCalibrator>>>,
    >,
    metrics_sink: parking_lot::RwLock<Arc<dyn contextra_ports::MetricsSink>>,
}

#[cfg(not(loom))]
pub mod contextra_impl;
#[cfg(not(loom))]
pub use contextra_impl::wiring::{EngineWiring, EngineWiringExt};

pub use contextra_types::DistanceMetric;
pub use serde_json::json;

#[cfg(not(loom))]
impl Contextra {
    #[doc(hidden)]
    pub fn inner_storage(&self) -> Arc<LsmStorage> {
        self.storage.clone()
    }

    /// Returns a reference to the active [`ContextraConfig`].
    pub fn config(&self) -> &ContextraConfig {
        &self.config
    }
}

#[cfg(test)]
mod embedding_backend_tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_embedding_backend_from_str_table() {
        let cases = vec![
            ("Candle", true),
            ("CANDLE", true),
            ("candle", true),
            ("onnx", true),
            ("ONNX", true),
            ("ollama", true),
            ("Ollama", true),
            ("none", true),
            ("NONE", true),
            ("mock", false),
            ("MOCK", false),
            ("unknown_backend", false),
            ("", false),
        ];

        for (input, expected_ok) in cases {
            let res = EmbeddingBackend::from_str(input);
            if expected_ok {
                assert!(res.is_ok(), "Expected '{input}' to parse successfully");
                let backend = res.unwrap();
                match input.to_ascii_lowercase().trim() {
                    "candle" => assert!(matches!(backend, EmbeddingBackend::Candle { .. })),
                    "onnx" => assert!(matches!(backend, EmbeddingBackend::Onnx { .. })),
                    "ollama" => assert!(matches!(backend, EmbeddingBackend::Ollama { .. })),
                    "none" => assert!(matches!(backend, EmbeddingBackend::None)),
                    _ => unreachable!(),
                }
            } else {
                assert!(res.is_err(), "Expected '{input}' to return an error");
                let err = res.unwrap_err();
                assert!(matches!(err, ContextraError::InvalidInput(_)));
            }
        }
    }
}
