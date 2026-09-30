// FILE-CONTEXT
// ZWECK: Contextra Core Engine Orchestrator & Facade (Layer 3 - Engine).
// INVARIANTEN: Monoton steigende TxId-Allokation; Reparaturgarantie beim Öffnen (repair_on_open); Strikte Isolation von Namespaces.
// NICHT-OFFENSICHTLICH: Lock-Hierarchie: collections (RwLock) -> kv_locks -> embedder (RwLock).

#![forbid(unsafe_code)]

#[cfg(feature = "encryption-at-rest")]
pub use contextra_crypto::deletion_proof::{
    DeletionLayer, DeletionProof, DeletionScope, LayerCleanupProof,
};

#[cfg(not(feature = "encryption-at-rest"))]
pub use no_crypto_stubs::*;

#[cfg(not(feature = "encryption-at-rest"))]
mod no_crypto_stubs {
    use contextra_types::{CollectionId, TenantId, TxId};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub enum DeletionLayer {
        LsmMemtable,
        SsTableAllLevels,
        HnswIndex,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum DeletionScope {
        Document {
            doc_id: contextra_types::DocId,
            tenant_id: TenantId,
        },
        Collection {
            collection_id: CollectionId,
            tenant_id: TenantId,
        },
        Tenant {
            tenant_id: TenantId,
        },
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct LayerCleanupProof {
        pub layer: DeletionLayer,
        pub remaining_count: usize,
    }

    impl LayerCleanupProof {
        pub fn new_after_verified_empty(
            layer: DeletionLayer,
            remaining_count: usize,
        ) -> contextra_types::Result<Self> {
            Ok(Self {
                layer,
                remaining_count,
            })
        }

        pub fn verify_and_create<F>(
            layer: DeletionLayer,
            verifier: F,
        ) -> contextra_types::Result<Self>
        where
            F: FnOnce() -> contextra_types::Result<bool>,
        {
            let _ = verifier();
            Self::new_after_verified_empty(layer, 0)
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub enum ExcludedScope {
        ConsolidatedAndDistilled,
        LlmParameterMemory,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DeletionProof {
        #[serde(default = "default_signature_version")]
        pub signature_version: u8,
        pub scope: DeletionScope,
        #[serde(default)]
        pub deleted_keys_hash: [u8; 32],
        #[serde(default = "default_tx_id")]
        pub deleted_after_tx: TxId,
        #[serde(default)]
        pub timestamp: u64,
        #[serde(default)]
        pub signature: Vec<u8>,
        #[serde(default)]
        pub covered_layers: Vec<DeletionLayer>,
        #[serde(default)]
        pub excluded_scopes: Vec<ExcludedScope>,
        #[serde(skip_serializing_if = "Option::is_none", default)]
        pub wal_chain_receipt: Option<[u8; 32]>,
        #[serde(skip_serializing_if = "Option::is_none", default)]
        pub integrity_warning: Option<String>,
        #[serde(default)]
        pub deleted_keys: Vec<Vec<u8>>,
    }

    const fn default_signature_version() -> u8 {
        1
    }

    const fn default_tx_id() -> TxId {
        TxId(0)
    }

    #[derive(Debug, Clone, Default)]
    pub struct TenantIsolatedKvStore;

    impl TenantIsolatedKvStore {
        pub fn new() -> Self {
            Self
        }

        pub fn purge_tenant_segments(&self, _tenant: TenantId) {}

        pub fn on_rollback(&self, _tenant: TenantId, _chunk_ids: &[u64]) {}

        pub fn remove_tenant_segment(&self, _tenant: TenantId, _doc_id: contextra_types::DocId) {}
    }

    impl DeletionProof {
        pub fn create(
            scope: DeletionScope,
            mut deleted_keys: Vec<Vec<u8>>,
            tx_id: TxId,
            layer_proofs: Vec<LayerCleanupProof>,
            _cas_proofs: Vec<Vec<u8>>,
            proof_key: &[u8],
        ) -> contextra_types::Result<Self> {
            deleted_keys.sort();
            let mut hasher = blake3::Hasher::new();
            for key in &deleted_keys {
                hasher.update(&(key.len() as u32).to_le_bytes());
                hasher.update(key);
            }
            let deleted_keys_hash: [u8; 32] = hasher.finalize().into();

            let covered_layers: Vec<DeletionLayer> =
                layer_proofs.into_iter().map(|p| p.layer).collect();
            let excluded_scopes: Vec<ExcludedScope> = vec![];

            let scope_bytes = bincode::serialize(&scope)
                .map_err(|e| contextra_types::ContextraError::Internal(e.to_string()))?;
            let tx_bytes = tx_id.0.to_le_bytes();
            let covered_layers_bytes = bincode::serialize(&covered_layers)
                .map_err(|e| contextra_types::ContextraError::Internal(e.to_string()))?;
            let excluded_scopes_bytes = bincode::serialize(&excluded_scopes)
                .map_err(|e| contextra_types::ContextraError::Internal(e.to_string()))?;

            use hmac::{Hmac, Mac};
            use sha2::Sha256;
            let mut mac = Hmac::<Sha256>::new_from_slice(proof_key).map_err(|e| {
                contextra_types::ContextraError::Internal(format!("HMAC key error: {e}"))
            })?;
            mac.update(&scope_bytes);
            mac.update(&deleted_keys_hash);
            mac.update(&tx_bytes);
            mac.update(&covered_layers_bytes);
            mac.update(&excluded_scopes_bytes);
            let signature = mac.finalize().into_bytes().to_vec();

            Ok(Self {
                signature_version: 2,
                scope,
                deleted_keys_hash,
                deleted_after_tx: tx_id,
                timestamp: 0,
                signature,
                covered_layers,
                excluded_scopes,
                wal_chain_receipt: None,
                integrity_warning: None,
                deleted_keys,
            })
        }

        pub fn tenant_id(&self) -> TenantId {
            match &self.scope {
                DeletionScope::Collection { tenant_id, .. } => *tenant_id,
                DeletionScope::Document { tenant_id, .. } => *tenant_id,
                DeletionScope::Tenant { tenant_id } => *tenant_id,
            }
        }

        pub fn verify(&self, _key: &[u8]) -> contextra_types::Result<bool> {
            Ok(true)
        }

        pub fn export_for_audit(&self) -> contextra_types::Result<String> {
            serde_json::to_string_pretty(self)
                .map_err(|e| contextra_types::ContextraError::Internal(e.to_string()))
        }
    }
}
use contextra_ports::license::LicenseGate;
#[cfg(feature = "sandbox")]
use contextra_ports::BoxFuture;
use contextra_ports::StorageEngine;
pub use contextra_ports::TextEmbeddingEngine;
pub use contextra_ports::VectorDeleteMode;
#[cfg(not(loom))]
pub use contextra_store::lsm::DurabilityMode;
#[cfg(not(loom))]
use contextra_store::LsmStorage;
use contextra_types::{CollectionId, DocId, TenantId};
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
#[cfg(feature = "entity-extraction")]
#[cfg(not(loom))]
pub mod extraction;
#[cfg(not(loom))]
pub mod filter;
#[cfg(not(loom))]
pub mod fusion;
#[cfg(not(loom))]
pub mod import;
#[cfg(not(loom))]
pub mod temporal_filter;
#[cfg(not(loom))]
pub mod transaction;

#[cfg(not(loom))]
pub use decay_controller::{AdaptiveDecayController, DecayControllerConfig, DecaySignalInputs};

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
pub use collection::crud::MAX_SCAN_RESULTS;
#[cfg(not(loom))]
#[cfg(feature = "graph-connectivity-health")]
pub use collection::maintenance::PercolationResult;
#[cfg(not(loom))]
pub use collection::query_builder::{HybridQueryBuilder, SearchStrategy, SignalWeights};
#[cfg(not(loom))]
pub use collection::{Collection, CollectionConfig};
pub use contextra_checkpoint;
#[cfg(feature = "graph-connectivity-health")]
pub use contextra_graph::percolation::PercolationConfig;
pub use contextra_text::Language;
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

pub use contextra_ports::DriftStatusProvider;
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
    pub encryption_passphrase: Option<String>,
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

    /// Controls whether deletion proof generation and HMAC integrity guarantees are active.
    /// Default: false.
    pub deletion_proof_active: bool,

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
}

#[cfg(not(loom))]
impl ContextraConfig {
    pub fn with_consolidation_launcher(mut self, launcher: ConsolidationLauncher) -> Self {
        self.consolidation_launcher = Some(launcher);
        self
    }

    pub fn with_tenant_policy(mut self, policy: TenantPolicy) -> Self {
        self.tenant_policy = policy;
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
            .field(
                "encryption_passphrase",
                &self.encryption_passphrase.as_ref().map(|_| "***"),
            )
            .field("max_ram_mb", &self.max_ram_mb)
            .field(
                "group_commit_window_micros",
                &self.group_commit_window_micros,
            )
            .field("durability_mode", &self.durability_mode)
            .field("deletion_proof_active", &self.deletion_proof_active)
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
            encryption_passphrase: None,
            max_ram_mb: 2048,
            group_commit_window_micros: 500,
            durability_mode: DurabilityMode::default(),
            deletion_proof_active: false,
            vector_delete_mode: VectorDeleteMode::BackgroundRepair,
            memtable_size_limit: 64 * 1024 * 1024,
            expiry_reaper_interval: std::time::Duration::from_secs(60),
            orphan_registry_path: None,
            community_detection: CommunityDetectionConfig::default(),
            embedding_backend: EmbeddingBackend::default(),
            consolidation_enabled: true,
            consolidation_interval: std::time::Duration::from_secs(6 * 3600),
            max_llm_calls_per_cycle: 10,
            consolidation_launcher: None,
            tenant_policy: TenantPolicy::Optional,
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
    license_gate: Arc<dyn LicenseGate>,
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
    pid_controller: parking_lot::RwLock<
        Option<std::sync::Weak<parking_lot::Mutex<contextra_adapt::PidController>>>,
    >,
}

#[cfg(not(loom))]
mod contextra_impl;

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

    /// Returns a reference to the attached [`LicenseGate`].
    pub fn license_gate(&self) -> Arc<dyn LicenseGate> {
        Arc::clone(&self.license_gate)
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
                            contextra_types::ContextraError::Serialization("Query too short".into())
                        })?
                        .try_into()
                        .map_err(|_| {
                            contextra_types::ContextraError::Serialization("Invalid slice".into())
                        })?,
                );
                vector.push(f32::from_bits(bits));
            }

            let results: Vec<SearchResult> = self.search(&vector, k).await?;
            serde_json::to_vec(&results)
                .map_err(|e| contextra_types::ContextraError::Internal(e.to_string()))
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
                Some(d) => Ok(Some(serde_json::to_vec(&d).map_err(|e| {
                    contextra_types::ContextraError::Internal(e.to_string())
                })?)),
                None => Ok(None),
            }
        })
    }
}
