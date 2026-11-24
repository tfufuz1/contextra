// FILE-CONTEXT
// ZWECK: Contextra Database Facade & Core Engine (Ring 3/Ring 2 gemäss ADR-0122).
// INVARIANTEN: Abwärtskompatibilität für bestehende Pfade via prelude/advanced; Forbid unsafe code.

#![forbid(unsafe_code)]

//! # Contextra DB — Facade & Storage Engine
//!
//! `contextra-db` bildet die benutzerfreundliche Hauptfassade und Speicher-Engine
//! für Anwendungen, welche die Contextra-Datenbankbibliothek als eingebettete Engine nutzen.

pub mod adapters;
pub mod backends;
#[cfg(not(loom))]
pub mod background_workers;
#[cfg(not(loom))]
pub mod chunker;
pub mod collection;
#[cfg(not(loom))]
pub mod contextra_impl;
#[cfg(not(loom))]
pub mod decay_controller;
#[cfg(not(loom))]
pub mod export;
#[cfg(any(feature = "exp-extraction", feature = "entity-extraction"))]
#[cfg(not(loom))]
pub mod extraction;
#[cfg(not(loom))]
pub mod filter;
#[cfg(not(loom))]
pub mod fusion;
pub mod homeostat;
#[cfg(not(loom))]
pub mod import;
pub(crate) mod license_seam;
#[cfg(feature = "sandbox")]
pub mod merge_operator_adapter;
#[cfg(not(loom))]
pub mod metrics_names;
pub mod multistep;
pub mod orchestration;
pub mod pid_latency_controller;
#[cfg(not(loom))]
pub mod temporal_filter;
#[cfg(not(loom))]
pub mod transaction;

#[cfg(feature = "cognition")]
pub mod cognition;

#[cfg(feature = "volatile-vault")]
pub mod volatile_vault;

use license_seam::LicenseGate;
#[cfg(feature = "sandbox")]
use contextra_ports::BoxFuture;
use contextra_ports::StorageEngine;
pub use contextra_ports::TextEmbeddingEngine;
pub use contextra_ports::VectorDeleteMode;
#[cfg(not(loom))]
pub use contextra_store::lsm::DurabilityMode;
#[cfg(not(loom))]
use contextra_store::LsmStorage;
use contextra_types::TenantId;
use contextra_vector::HnswIndex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;

#[cfg(feature = "sandbox")]
pub use merge_operator_adapter::WasmMergeOperator;

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
pub use backends::Backends;
#[cfg(not(loom))]
pub use backends::DefaultBackends;
#[cfg(not(loom))]
pub use collection::crud::MAX_SCAN_RESULTS;
#[cfg(feature = "graph-connectivity-health")]
pub use collection::maintenance::PercolationConfig;
#[cfg(not(loom))]
#[cfg(feature = "graph-connectivity-health")]
pub use collection::maintenance::PercolationResult;
#[cfg(not(loom))]
pub use collection::query_builder::{HybridQueryBuilder, SearchStrategy, SignalWeights};
pub use collection::{Collection, CollectionConfig};
pub use contextra_checkpoint;
pub use contextra_text::Language;
#[cfg(not(loom))]
#[allow(deprecated)]
pub use filter::MetadataFilter;

#[cfg(not(loom))]
#[allow(clippy::type_complexity)]
/// Funktions-Typdefinition zum Starten von Wissenskonsolidierungs-Workern.
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
/// Repräsentiert ein Dokument mit eindeutiger ID, Text, Vektoreinbettung und Metadaten.
pub struct Document {
    pub id: String,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Aggregierte Betriebs- und Belegungsstatistiken der Engine.
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
/// Alias für `ContextraStats` zur Abwärtskompatibilität.
pub type DbStats = ContextraStats;

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Konfiguration für Community-Erkennungs- und Graph-Clustering-Algorithmen.
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
/// Wählbares Backend zur Berechnung von Text-Embeddings.
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
/// Globale Konfiguration für die Contextra-Engine-Initialisierung.
pub struct ContextraConfig {
    pub dimension: usize,
    pub max_elements: usize,
    pub distance_metric: contextra_types::DistanceMetric,
    pub encryption_passphrase: Option<String>,
    pub max_ram_mb: u64,
    pub group_commit_window_micros: u64,
    pub durability_mode: DurabilityMode,
    pub deletion_proof_active: bool,
    pub vector_delete_mode: VectorDeleteMode,
    pub memtable_size_limit: usize,
    pub expiry_reaper_interval: std::time::Duration,
    pub orphan_registry_path: Option<std::path::PathBuf>,
    pub community_detection: CommunityDetectionConfig,
    pub embedding_backend: EmbeddingBackend,
    pub consolidation_enabled: bool,
    pub consolidation_interval: std::time::Duration,
    pub max_llm_calls_per_cycle: usize,
    pub consolidation_launcher: Option<ConsolidationLauncher>,
    pub tenant_policy: TenantPolicy,
    pub metrics_sink: Option<Arc<dyn contextra_ports::MetricsSink>>,
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

    pub fn with_metrics_sink(mut self, sink: Arc<dyn contextra_ports::MetricsSink>) -> Self {
        self.metrics_sink = Some(sink);
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
            .field(
                "metrics_sink",
                &self.metrics_sink.as_ref().map(|_| "dyn MetricsSink"),
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
            metrics_sink: None,
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
    metrics_sink: parking_lot::RwLock<Arc<dyn contextra_ports::MetricsSink>>,
}

pub use contextra_types::DistanceMetric;
pub use serde_json::json;

#[cfg(not(loom))]
impl Contextra {
    #[doc(hidden)]
    pub fn inner_storage(&self) -> Arc<LsmStorage> {
        self.storage.clone()
    }

    pub fn config(&self) -> &ContextraConfig {
        &self.config
    }

    pub fn license_gate(&self) -> Arc<dyn LicenseGate> {
        Arc::clone(&self.license_gate)
    }
}

#[cfg(feature = "sandbox")]
pub trait SandboxBridge: Send + Sync {
    fn db_search<'a>(
        &'a self,
        query: &'a [u8],
        k: usize,
    ) -> BoxFuture<'a, contextra_types::Result<Vec<u8>>>;
    fn db_insert<'a>(
        &'a self,
        key: &'a [u8],
        value: &'a [u8],
    ) -> BoxFuture<'a, contextra_types::Result<()>>;
    fn db_get<'a>(
        &'a self,
        key: &'a [u8],
    ) -> BoxFuture<'a, contextra_types::Result<Option<Vec<u8>>>>;
}

#[cfg(feature = "sandbox")]
impl SandboxBridge for Contextra {
    fn db_search<'a>(
        &'a self,
        query: &'a [u8],
        k: usize,
    ) -> BoxFuture<'a, contextra_types::Result<Vec<u8>>> {
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

    fn db_insert<'a>(
        &'a self,
        key: &'a [u8],
        value: &'a [u8],
    ) -> BoxFuture<'a, contextra_types::Result<()>> {
        Box::pin(async move {
            let id = String::from_utf8_lossy(key).to_string();
            let val_json: Value = serde_json::from_slice(value)
                .unwrap_or(serde_json::json!({ "raw_data": String::from_utf8_lossy(value) }));

            self.insert(&id, &[], Some(val_json)).await
        })
    }

    fn db_get<'a>(
        &'a self,
        key: &'a [u8],
    ) -> BoxFuture<'a, contextra_types::Result<Option<Vec<u8>>>> {
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

pub mod prelude {
    pub use crate::collection::{self, Collection, CollectionConfig};
    pub use crate::{
        Contextra, ContextraConfig, ContextraStats, DbStats, Document, EmbeddingBackend,
        HybridQueryBuilder, Language, ProvenanceRecord, SearchResult, SearchStrategy,
        SignalContribution, SignalWeights,
    };
    pub use contextra_types::{DistanceMetric, DocId, RequestContext, TenantId, TxId};

    pub use crate::multistep::{MultiStepConfig, MultiStepEngine, MultiStepResult, QueryRewriter};

    #[cfg(feature = "volatile-vault")]
    pub use crate::volatile_vault::*;
}

pub mod advanced {
    pub use crate::decay_controller::{
        AdaptiveDecayController, DecayControllerConfig, DecaySignalInputs,
    };
    pub use crate::homeostat::RerankDeadline;
    #[allow(deprecated)]
    pub use crate::homeostat::RerankPidController;
    pub use crate::pid_latency_controller::{
        LatencyBudgetGuard, PidLatencyController, DEFAULT_TARGET_LATENCY_MS, MAX_SCALING_FACTOR,
        MIN_SCALING_FACTOR,
    };

    pub use crate as engine;
    pub use crate::DurabilityMode;

    pub use crate::{
        background_workers, chunker, export, filter, import, temporal_filter, transaction,
        CommunityDetectionConfig, ExportCollectionV1, ExportDocumentV1, ExportMemoryV1,
        ExportRelationV1, ImportSummary, MAX_SCAN_RESULTS, SCHEMA_VERSION_V1,
    };

    #[cfg(feature = "sandbox")]
    pub use crate::SandboxBridge;
}

pub use advanced::*;
pub use prelude::*;

pub use homeostat::RerankDeadline;
#[allow(deprecated)]
pub use homeostat::RerankPidController;
pub use multistep::{MultiStepConfig, MultiStepEngine, MultiStepResult, QueryRewriter};
pub use pid_latency_controller::{
    LatencyBudgetGuard, PidLatencyController, DEFAULT_TARGET_LATENCY_MS, MAX_SCALING_FACTOR,
    MIN_SCALING_FACTOR,
};

#[cfg(feature = "volatile-vault")]
pub use volatile_vault::*;

#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use super::*;
    use contextra_types::TenantId;
    use std::sync::Arc;
    use tempfile::TempDir;

    async fn test_db(dim: usize) -> (Contextra, TempDir) {
        let tmp = TempDir::new().expect("temp dir");
        let config = ContextraConfig {
            dimension: dim,
            max_elements: 10_000,
            distance_metric: DistanceMetric::Cosine,
            ..Default::default()
        };
        let db = Contextra::open_with_config(tmp.path(), config)
            .await
            .expect("open db");
        (db, tmp)
    }

    #[tokio::test]
    async fn test_insert_search_roundtrip() {
        let (db, _tmp) = test_db(4).await;

        db.insert(
            "doc-1",
            &[1.0, 0.0, 0.0, 0.0],
            Some(json!({"topic": "rust"})),
        )
        .await
        .expect("insert");

        db.insert(
            "doc-2",
            &[0.0, 1.0, 0.0, 0.0],
            Some(json!({"topic": "python"})),
        )
        .await
        .expect("insert");

        db.insert("doc-3", &[0.9, 0.1, 0.0, 0.0], None)
            .await
            .expect("insert");

        let results = db.search(&[1.0, 0.0, 0.0, 0.0], 2).await.expect("search");
        assert_eq!(results.len(), 2);
        assert!(results[0].score > results[1].score);
    }

    #[tokio::test]
    async fn test_insert_search_returns_metadata() {
        let (db, _tmp) = test_db(4).await;

        db.insert(
            "doc-1",
            &[1.0, 0.0, 0.0, 0.0],
            Some(json!({"topic": "rust", "priority": 1})),
        )
        .await
        .expect("insert");

        let results = db.search(&[1.0, 0.0, 0.0, 0.0], 1).await.expect("search");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "doc-1");
        let meta = results[0].metadata.as_ref().expect("metadata should exist");
        assert_eq!(meta["topic"], "rust");
        assert_eq!(meta["priority"], 1);
    }

    #[tokio::test]
    async fn test_get_by_key() {
        let (db, _tmp) = test_db(4).await;

        db.insert(
            "doc-1",
            &[1.0, 0.0, 0.0, 0.0],
            Some(json!({"topic": "rust"})),
        )
        .await
        .expect("insert");

        let doc = db.get("doc-1").await.expect("get").expect("should exist");
        assert_eq!(doc.id, "doc-1");
        assert_eq!(doc.metadata.expect("valid")["topic"], "rust");

        let none = db.get("nonexistent").await.expect("get");
        assert!(none.is_none());
    }

    #[tokio::test]
    async fn test_update() {
        let (db, _tmp) = test_db(4).await;

        db.insert("doc-1", &[1.0, 0.0, 0.0, 0.0], Some(json!({"v": 1})))
            .await
            .expect("insert");

        db.update("doc-1", &[0.0, 1.0, 0.0, 0.0], Some(json!({"v": 2})))
            .await
            .expect("update");

        let doc = db.get("doc-1").await.expect("get").expect("exists");
        assert_eq!(doc.metadata.expect("valid")["v"], 2);

        let results = db.search(&[0.0, 1.0, 0.0, 0.0], 1).await.expect("search");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "doc-1");
    }

    #[tokio::test]
    async fn test_delete() {
        let (db, _tmp) = test_db(4).await;

        db.insert("doc-1", &[1.0, 0.0, 0.0, 0.0], None)
            .await
            .expect("insert");
        assert_eq!(db.len().await.expect("len"), 1);

        db.delete("doc-1").await.expect("delete");
        assert_eq!(db.len().await.expect("len"), 0);

        let doc = db.get("doc-1").await.expect("get");
        assert!(doc.is_none());
    }

    #[tokio::test]
    async fn test_relate() {
        let (db, _tmp) = test_db(4).await;

        db.insert("doc-1", &[1.0, 0.0, 0.0, 0.0], None)
            .await
            .expect("insert");
        db.insert("doc-2", &[0.0, 1.0, 0.0, 0.0], None)
            .await
            .expect("insert");

        db.relate("doc-1", "doc-2", "references")
            .await
            .expect("relate");
    }

    #[tokio::test]
    async fn test_dimension_mismatch() {
        let (db, _tmp) = test_db(4).await;
        let result = db.insert("doc-1", &[1.0, 0.0], None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_empty_search() {
        let (db, _tmp) = test_db(4).await;
        let results = db.search(&[1.0, 0.0, 0.0, 0.0], 5).await.expect("search");
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_relate_and_scan_prefix() {
        let (db, _tmp) = test_db(4).await;

        db.insert("doc-1", &[1.0, 0.0, 0.0, 0.0], None)
            .await
            .expect("insert");
        db.insert("doc-2", &[0.0, 1.0, 0.0, 0.0], None)
            .await
            .expect("insert");
        db.insert("doc-3", &[0.0, 0.0, 1.0, 0.0], None)
            .await
            .expect("insert");

        db.relate_bidirectional("doc-1", "doc-2", "references")
            .await
            .expect("relate");
        db.relate("doc-1", "doc-3", "references")
            .await
            .expect("relate");

        let results = db
            .scan_prefix("__rel:doc-1:references:", None)
            .await
            .expect("scan");
        assert_eq!(results.len(), 2);

        let related_ids: Vec<String> = results
            .into_iter()
            .map(|(_, v)| v["to"].as_str().expect("valid").to_string())
            .collect();
        assert!(related_ids.contains(&"doc-2".to_string()));
        assert!(related_ids.contains(&"doc-3".to_string()));

        let backward_results = db
            .scan_prefix("__rel:doc-2:references:", None)
            .await
            .expect("scan bwd");
        assert_eq!(backward_results.len(), 1);
        assert_eq!(backward_results[0].1["to"], "doc-1");
    }

    #[tokio::test]
    async fn test_stats_aggregation() {
        let (db, _tmp) = test_db(4).await;

        db.insert("doc-1", &[1.0, 0.0, 0.0, 0.0], None)
            .await
            .expect("insert");

        let stats = db.stats().await.expect("stats");
        assert_eq!(stats.index_stats.num_vectors, 1);
        assert_eq!(stats.active_memory_count, 1);
        assert_eq!(stats.drift_status, "nicht verfügbar");
        assert!(stats.storage_stats.memtable_size_bytes > 0);
    }

    #[tokio::test]
    async fn test_stats_live_data_and_weak_reference_lifecycle() {
        let (db, _tmp) = test_db(4).await;

        let stats_unattached = db.stats().await.expect("stats");
        assert_eq!(stats_unattached.drift_status, "nicht verfügbar");
        assert_eq!(stats_unattached.calibration_ece, None);
        assert_eq!(stats_unattached.last_calibration_at, None);
        assert_eq!(stats_unattached.pid_pool_size, None);

        struct DummyRouter;
        impl DriftStatusProvider for DummyRouter {
            fn overall_drift_status(&self) -> String {
                "stabil".to_string()
            }
        }
        let dummy = DummyRouter;
        assert_eq!(dummy.overall_drift_status(), "stabil");

        let router_arc: Arc<dyn DriftStatusProvider> = Arc::new(DummyRouter);
        let calibrator_arc = Arc::new(parking_lot::Mutex::new(
            contextra_rank::IsotonicCalibrator::new(5, 100),
        ));
        {
            let mut cal = calibrator_arc.lock();
            for i in 0..10 {
                cal.record_outcome(i as f32 / 10.0, i > 5);
            }
            cal.force_rebuild();
        }

        db.set_router(Arc::downgrade(&router_arc));
        db.set_calibrator(Arc::downgrade(&calibrator_arc));

        let stats_attached = db.stats().await.expect("stats");
        assert_eq!(stats_attached.drift_status, "stabil");
        assert!(stats_attached.calibration_ece.is_some());
        assert!(stats_attached.last_calibration_at.is_some());

        drop(router_arc);
        drop(calibrator_arc);

        let stats_dropped = db.stats().await.expect("stats after drop");
        assert_eq!(stats_dropped.drift_status, "nicht verfügbar");
        assert_eq!(stats_dropped.calibration_ece, None);
        assert_eq!(stats_dropped.last_calibration_at, None);
        assert_eq!(stats_dropped.pid_pool_size, None);
    }

    #[tokio::test]
    async fn test_integration_end_to_end() {
        let (db, _tmp) = test_db(4).await;

        db.insert(
            "agent-1",
            &[1.0, 0.5, 0.0, 0.0],
            Some(json!({"type": "agent"})),
        )
        .await
        .expect("insert agent");
        db.insert(
            "task-1",
            &[0.9, 0.6, 0.0, 0.0],
            Some(json!({"type": "task"})),
        )
        .await
        .expect("insert task");
        db.insert(
            "task-2",
            &[0.0, 0.0, 1.0, 0.5],
            Some(json!({"type": "task"})),
        )
        .await
        .expect("insert task 2");

        db.relate("agent-1", "task-1", "assigned_to")
            .await
            .expect("relate 1");
        db.relate("agent-1", "task-2", "assigned_to")
            .await
            .expect("relate 2");

        let results = db.search(&[1.0, 0.5, 0.0, 0.0], 2).await.expect("search");
        assert_eq!(results[0].id, "agent-1");
        assert_eq!(results[1].id, "task-1");

        db.update(
            "task-1",
            &[0.1, 0.1, 0.9, 0.9],
            Some(json!({"type": "task", "status": "done"})),
        )
        .await
        .expect("update task");

        let edges = db
            .scan_prefix("__rel:agent-1:assigned_to:", None)
            .await
            .expect("scan");
        assert_eq!(edges.len(), 2);

        db.delete("agent-1").await.expect("delete");

        let get_agent = db.get("agent-1").await.expect("get");
        assert!(get_agent.is_none());
        assert_eq!(db.len().await.expect("len"), 2);
    }

    #[tokio::test]
    async fn test_concurrent_collection_idempotency() {
        let (db, _tmp) = test_db(4).await;
        let db = Arc::new(db);
        let mut handles = Vec::new();

        for _ in 0..10 {
            let db_clone = db.clone();
            handles.push(tokio::spawn(async move {
                db_clone.collection("c").await.expect("collection c")
            }));
        }

        let mut cols = Vec::new();
        for handle in handles {
            cols.push(handle.await.expect("join handle"));
        }

        let first = &cols[0];
        for col in &cols[1..] {
            assert!(
                Arc::ptr_eq(first, col),
                "collection(\"c\") must return the exact same Arc instance"
            );
        }
    }

    #[tokio::test]
    async fn test_collections_are_isolated() {
        let (db, _tmp) = test_db(4).await;
        let col_a = db.collection("a").await.expect("col a");
        let col_b = db.collection("b").await.expect("col b");

        col_a
            .insert("k1", &[1.0, 0.0, 0.0, 0.0], Some(json!({"val": "a"})))
            .await
            .expect("ins a");
        col_b
            .insert("k1", &[0.0, 1.0, 0.0, 0.0], Some(json!({"val": "b"})))
            .await
            .expect("ins b");

        let res_a = col_a.get("k1").await.expect("get a").expect("exists");
        let res_b = col_b.get("k1").await.expect("get b").expect("exists");

        assert_eq!(res_a.metadata.expect("test")["val"], "a");
        assert_eq!(res_b.metadata.expect("test")["val"], "b");

        let search_a = col_a
            .search(&[1.0, 0.0, 0.0, 0.0], 1)
            .await
            .expect("search a");
        assert_eq!(search_a.len(), 1);
        assert_eq!(search_a[0].id, "k1");
        assert_eq!(search_a[0].metadata.as_ref().expect("test")["val"], "a");
    }

    #[tokio::test]
    async fn test_close_and_reopen_100_docs() {
        let tmp = TempDir::new().expect("temp dir");
        let path = tmp.path().to_path_buf();
        let config = ContextraConfig {
            dimension: 4,
            ..Default::default()
        };

        {
            let db = Contextra::open_with_config(&path, config.clone())
                .await
                .expect("open 1");
            for i in 0..100 {
                let id = format!("doc-{}", i);
                let val = (i as f32) / 100.0;
                db.insert(&id, &[val, 1.0 - val, 0.0, 0.0], Some(json!({"idx": i})))
                    .await
                    .expect("insert");
            }
            db.close().await.expect("close");
        }

        {
            let db = Contextra::open_with_config(&path, config)
                .await
                .expect("open 2");
            assert_eq!(db.len().await.expect("len"), 100);
            for i in 0..100 {
                let id = format!("doc-{}", i);
                let doc = db.get(&id).await.expect("get").expect("exists");
                assert_eq!(doc.id, id);
                assert_eq!(doc.metadata.expect("valid")["idx"], i);
            }
            let results = db.search(&[0.5, 0.5, 0.0, 0.0], 10).await.expect("search");
            assert_eq!(results.len(), 10);
        }
    }

    #[tokio::test]
    async fn test_close_and_reopen() {
        let tmp = TempDir::new().expect("temp dir");
        let path = tmp.path().to_path_buf();
        let config = ContextraConfig {
            dimension: 4,
            ..Default::default()
        };

        {
            let db = Contextra::open_with_config(&path, config.clone())
                .await
                .expect("open 1");
            db.insert("doc-1", &[1.0, 0.0, 0.0, 0.0], Some(json!({"v": 1})))
                .await
                .expect("insert");
            db.close().await.expect("close");
        }

        {
            let db = Contextra::open_with_config(&path, config)
                .await
                .expect("open 2");
            let doc = db.get("doc-1").await.expect("get").expect("exists");
            assert_eq!(doc.id, "doc-1");
            assert_eq!(doc.metadata.expect("valid")["v"], 1);
        }
    }

    #[tokio::test]
    async fn test_drop_removes_all_data() {
        let (db, _tmp) = test_db(4).await;
        let tenant_id = TenantId::try_new(1).expect("tenant_id");
        let col = db
            .collection_for_tenant("drop-me", tenant_id)
            .await
            .expect("col");
        col.insert("k1", &[1.0, 0.0, 0.0, 0.0], None)
            .await
            .expect("ins");

        db.drop_collection("drop-me", tenant_id)
            .await
            .expect("drop");

        let col2 = db
            .collection_for_tenant("drop-me", tenant_id)
            .await
            .expect("re-create");
        assert_eq!(col2.len().await, 0);
        assert!(col2.get("k1").await.expect("get").is_none());
    }

    #[tokio::test]
    async fn test_default_collection_compat() {
        let (db, _tmp) = test_db(4).await;
        db.insert("k", &[1.0, 0.0, 0.0, 0.0], Some(json!({"v": 1})))
            .await
            .expect("ins");

        let doc = db.get("k").await.expect("get").expect("exists");
        assert_eq!(doc.id, "k");

        let results = db.search(&[1.0, 0.0, 0.0, 0.0], 1).await.expect("search");
        assert_eq!(results[0].id, "k");
    }

    #[tokio::test]
    async fn test_list_collections() {
        let (db, _tmp) = test_db(4).await;
        db.collection("c1").await.expect("c1");
        db.collection("c2").await.expect("c2");
        db.collection("c3").await.expect("c3");

        let list = db.list_collections().await.expect("list");
        assert!(list.contains(&"default".to_string()));
        assert!(list.contains(&"c1".to_string()));
        assert!(list.contains(&"c2".to_string()));
        assert!(list.contains(&"c3".to_string()));
        assert_eq!(list.len(), 4);
    }

    #[test]
    fn test_contextra_config_defaults_to_onnx_backend() {
        let config = ContextraConfig::default();
        assert_eq!(config.dimension, 768);
        assert_eq!(
            config.embedding_backend,
            EmbeddingBackend::Onnx {
                model_name: "nomic-embed-text".to_string(),
                cache_dir: None,
            }
        );
        assert_eq!(
            EmbeddingBackend::default(),
            EmbeddingBackend::Onnx {
                model_name: "nomic-embed-text".to_string(),
                cache_dir: None,
            }
        );
    }

    #[tokio::test]
    async fn test_open_dimension_mismatch_fails() -> contextra_types::Result<()> {
        let dir = tempfile::tempdir()
            .map_err(|e| contextra_types::ContextraError::InvalidInput(e.to_string()))?;
        let config_768 = ContextraConfig {
            dimension: 768,
            ..Default::default()
        };
        let db = Contextra::open_with_config(dir.path(), config_768).await?;
        db.close().await?;

        let config_1536 = ContextraConfig {
            dimension: 1536,
            ..Default::default()
        };
        let result = Contextra::open_with_config(dir.path(), config_1536).await;
        assert!(result.is_err());
        let err_msg = match result {
            Err(e) => e.to_string(),
            Ok(_) => panic!("expected error"),
        };
        assert!(err_msg.contains("Dimension mismatch"));
        Ok(())
    }

    #[test]
    fn test_provenance_record_serialization_roundtrip() {
        let p = ProvenanceRecord {
            vector_distance: Some(0.42),
            bm25_score: Some(1.23),
            graph_score: None,
            rerank_score: None,
            signal_ranks: {
                let mut m = ahash::AHashMap::new();
                m.insert("vector".to_string(), 1u32);
                m.insert("bm25".to_string(), 3u32);
                m
            },
            source_collection: Some("test_col".to_string()),
            index_type: Some("hnsw".to_string()),
            signal_contributions: ahash::AHashMap::new(),
            coherence_bonus: 0.0,
        };
        let json = serde_json::to_string(&p).expect("serialize");
        let back: ProvenanceRecord = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.vector_distance, Some(0.42));
        assert_eq!(back.bm25_score, Some(1.23));
        assert_eq!(back.graph_score, None);
        assert_eq!(back.source_collection.as_deref(), Some("test_col"));
    }

    #[test]
    fn test_search_result_with_provenance_serialization() {
        let sr = SearchResult {
            id: "doc1".to_string(),
            score: 0.9,
            metadata: None,
            matched_signals: vec!["vector".to_string()],
            provenance: Some(ProvenanceRecord {
                source_collection: Some("my_col".to_string()),
                ..Default::default()
            }),
        };
        let json = serde_json::to_string(&sr).expect("serialize");
        assert!(json.contains("provenance"));
        assert!(json.contains("my_col"));
    }

    #[test]
    fn test_search_result_without_provenance_serialization_omits_field() {
        let sr = SearchResult {
            id: "doc2".to_string(),
            score: 0.5,
            metadata: None,
            matched_signals: vec![],
            provenance: None,
        };
        let json = serde_json::to_string(&sr).expect("serialize");
        assert!(
            !json.contains("provenance"),
            "provenance=None soll im JSON weggelassen werden: {json}"
        );
    }

    #[tokio::test]
    async fn test_provenance_rrf_sum_invariant() {
        let (db, _tmp) = test_db(4).await;
        let col = db.collection("prov_test").await.expect("collection");

        for i in 0..10 {
            let id = format!("doc-{}", i);
            let val = (i as f32) / 10.0;
            col.insert(
                &id,
                &[val, 1.0 - val, 0.0, 0.0],
                Some(json!({ "text": format!("rust memory system {}", i) })),
            )
            .await
            .expect("insert");
        }

        let results = col
            .query()
            .text("rust memory")
            .embedding([0.5, 0.5, 0.0, 0.0])
            .include_provenance(true)
            .k(5)
            .execute()
            .await
            .expect("search");

        assert!(!results.is_empty(), "Results must not be empty");

        for result in &results {
            let prov = result
                .provenance
                .as_ref()
                .expect("Provenance must be present when include_provenance=true");
            let sum_contrib: f32 = prov
                .signal_contributions
                .values()
                .map(|c| c.rrf_contribution)
                .sum();
            assert!(
                (sum_contrib - result.score).abs() < 1e-6,
                "INV-PROV-1 verletzt: Summe der Signal-Beiträge ({}) ≠ RRF-Score ({})",
                sum_contrib,
                result.score
            );
        }
    }

    #[tokio::test]
    async fn test_search_results_have_provenance_is_some() {
        let (db, _tmp) = test_db(4).await;
        let col = db.collection("search_prov_test").await.expect("collection");

        col.insert(
            "doc-1",
            &[1.0, 0.0, 0.0, 0.0],
            Some(json!({"text": "rust search provenance test"})),
        )
        .await
        .expect("insert");

        let vec_results = col
            .search(&[1.0, 0.0, 0.0, 0.0], 1)
            .await
            .expect("vector search");
        assert_eq!(vec_results.len(), 1);
        assert!(
            vec_results[0].provenance.is_some(),
            "Vector search results must contain provenance record"
        );
        let prov = vec_results[0].provenance.as_ref().expect("provenance");
        assert_eq!(prov.source_collection.as_deref(), Some("search_prov_test"));
        assert_eq!(prov.index_type.as_deref(), Some("hnsw"));

        let text_results = col
            .query()
            .text("rust search")
            .embedding([1.0, 0.0, 0.0, 0.0])
            .include_provenance(true)
            .k(1)
            .execute()
            .await
            .expect("text search");
        assert_eq!(text_results.len(), 1);
        assert!(
            text_results[0].provenance.is_some(),
            "Text search results must contain provenance record"
        );
    }
}
