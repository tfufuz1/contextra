// FILE-CONTEXT
// ZWECK: MemFuse Core Engine Orchestrator & Facade (Layer 3 - Engine).
// INVARIANTEN: Monoton steigende TxId-Allokation; Reparaturgarantie beim Öffnen (repair_on_open); Strikte Isolation von Namespaces.
// NICHT-OFFENSICHTLICH: Lock-Hierarchie: collections (RwLock) -> kv_locks -> embedder (RwLock).

#![forbid(unsafe_code)]

#[cfg(feature = "sandbox")]
use memfuse_core::BoxFuture;
pub use memfuse_core::TextEmbeddingEngine;
use memfuse_core::{CollectionId, DocId, Result, StorageEngine, TenantId, TxId};
use memfuse_crypto::deletion_proof::{
    DeletionLayer, DeletionProof, DeletionScope, LayerCleanupProof,
};
use memfuse_store::LsmStorage;
use memfuse_vector::{HnswConfig, HnswIndex};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;
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
pub use memfuse_checkpoint;
use memfuse_core::FilterExpr;
#[cfg(feature = "graph-connectivity-health")]
pub use memfuse_graph::percolation::PercolationConfig;
pub use memfuse_text::Language;

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

/// Registers a consolidation worker launcher function (used by `memfuse-cognition`).
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

/// Herkunftsnachweis für ein einzelnes Suchergebnis.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProvenanceRecord {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vector_distance: Option<f32>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bm25_score: Option<f32>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub graph_score: Option<f32>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rerank_score: Option<f32>,

    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub signal_ranks: ahash::AHashMap<String, u32>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_collection: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index_type: Option<String>,

    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub signal_contributions: ahash::AHashMap<String, SignalContribution>,

    #[serde(default)]
    pub coherence_bonus: f32,
}

impl ProvenanceRecord {
    pub fn synthesized_from(source_doc_ids: &[DocId]) -> Self {
        let mut signal_ranks = ahash::AHashMap::new();
        for (idx, id) in source_doc_ids.iter().enumerate() {
            signal_ranks.insert(id.0.to_string(), (idx + 1) as u32);
        }
        ProvenanceRecord {
            index_type: Some("consolidated".to_string()),
            signal_ranks,
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SignalContribution {
    pub raw_score: f32,
    pub rank: u32,
    pub rrf_contribution: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub score: f32,
    pub metadata: Option<Value>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub matched_signals: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provenance: Option<ProvenanceRecord>,
}

impl TryFrom<SearchResult> for memfuse_core::ContextChunk {
    type Error = memfuse_core::MemFuseError;

    fn try_from(r: SearchResult) -> std::result::Result<Self, Self::Error> {
        let doc_id = DocId::from_key(&r.id).map_err(|e| {
            memfuse_core::MemFuseError::InvalidInput(format!(
                "SearchResult-ID '{}' ungültig: {e}",
                r.id
            ))
        })?;
        let content = r
            .metadata
            .as_ref()
            .and_then(|m| m.get("text").or_else(|| m.get("content")))
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .to_string();
        let token_count = crate::chunker::estimate_tokens(&content);
        let links: Vec<memfuse_core::types::domain::MemoryLink> = r
            .metadata
            .as_ref()
            .and_then(|m| m.get("links"))
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();
        Ok(memfuse_core::ContextChunk {
            doc_id,
            content,
            relevance: r.score,
            token_count,
            metadata: r.metadata,
            contextual_prefix: None,
            links,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemFuseStats {
    pub drift_status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub calibration_ece: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_calibration_at: Option<u64>,
    pub active_memory_count: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pid_pool_size: Option<usize>,
    pub index_stats: memfuse_core::VectorIndexStats,
    pub storage_stats: memfuse_core::StorageStats,
}

pub use memfuse_core::DriftStatusProvider;
pub type DbStats = MemFuseStats;

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
pub struct MemFuseConfig {
    pub dimension: usize,
    pub max_elements: usize,
    pub distance_metric: memfuse_core::DistanceMetric,
    pub encryption_passphrase: Option<String>,
    pub expiry_reaper_interval: std::time::Duration,
    pub orphan_registry_path: Option<std::path::PathBuf>,
    pub community_detection: CommunityDetectionConfig,
    pub embedding_backend: EmbeddingBackend,
    pub consolidation_enabled: bool,
    pub consolidation_interval: std::time::Duration,
    pub max_llm_calls_per_cycle: usize,
}

impl Default for MemFuseConfig {
    fn default() -> Self {
        Self {
            dimension: 768,
            max_elements: 1_000_000,
            distance_metric: memfuse_core::DistanceMetric::Cosine,
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

pub struct MemFuse {
    storage: Arc<LsmStorage>,
    next_tx: Arc<AtomicU64>,
    dimension: usize,
    expiry_reaper_interval: std::time::Duration,
    community_detection_threshold: u64,
    collections: tokio::sync::RwLock<ahash::AHashMap<String, Arc<Collection<LsmStorage>>>>,
    cancel_token: tokio_util::sync::CancellationToken,
    task_tracker: tokio_util::task::TaskTracker,
    embedder: parking_lot::RwLock<Option<Arc<dyn TextEmbeddingEngine>>>,
    orphan_registry: Arc<memfuse_checkpoint::InstanceOrphanRegistry>,
    router: parking_lot::RwLock<Option<std::sync::Weak<dyn DriftStatusProvider>>>,
    calibrator: parking_lot::RwLock<
        Option<std::sync::Weak<parking_lot::Mutex<memfuse_calibration::IsotonicCalibrator>>>,
    >,
    pid_controller: parking_lot::RwLock<
        Option<std::sync::Weak<parking_lot::Mutex<memfuse_calibration::PidController>>>,
    >,
}

impl MemFuse {
    #[tracing::instrument(level = "trace", skip(path))]
    pub async fn open(path: impl AsRef<Path>) -> Result<Self> {
        Self::open_with_config(path, MemFuseConfig::default()).await
    }

    #[tracing::instrument(level = "trace", skip(path, config))]
    pub async fn open_with_config(path: impl AsRef<Path>, config: MemFuseConfig) -> Result<Self> {
        if config.dimension == 0 {
            return Err(memfuse_core::MemFuseError::invalid_input(
                "dimension must be > 0",
            ));
        }
        if config.dimension > 65536 {
            return Err(memfuse_core::MemFuseError::invalid_input(
                "dimension exceeds maximum",
            ));
        }

        let lsm_config = memfuse_store::LsmConfig {
            path: path.as_ref().to_path_buf(),
            encryption_passphrase: config.encryption_passphrase.clone(),
            ..Default::default()
        };

        let storage = Arc::new(LsmStorage::new(lsm_config).await?);
        let last_tx = storage.last_tx_id().await?.inner();
        let next_tx = Arc::new(AtomicU64::new(last_tx + 1));

        let dim_key = b"__meta:dimension";
        if let Some(stored_dim_bytes) = storage.get(dim_key).await? {
            if let Ok(s) = std::str::from_utf8(&stored_dim_bytes) {
                if let Ok(stored_dim) = s.parse::<usize>() {
                    if stored_dim != config.dimension {
                        return Err(memfuse_core::MemFuseError::invalid_input(format!(
                            "Dimension mismatch: DB wurde mit dim={} erstellt, \
                             Config fordert dim={}. \
                             Passe MemFuseConfig::dimension an oder nutze eine neue DB.",
                            stored_dim, config.dimension
                        )));
                    }
                }
            }
        } else {
            let tx = TxId::new(0);
            storage
                .put(tx, dim_key, config.dimension.to_string().as_bytes())
                .await?;
            storage.commit(tx).await?;
        }

        let cancel_token = tokio_util::sync::CancellationToken::new();
        let task_tracker = tokio_util::task::TaskTracker::new();

        let orphan_path = config
            .orphan_registry_path
            .clone()
            .unwrap_or_else(|| path.as_ref().join(".orphan_registry.json"));
        let orphan_registry = Arc::new(memfuse_checkpoint::InstanceOrphanRegistry::new(
            &orphan_path,
        ));

        let db = Self {
            storage,
            next_tx,
            dimension: config.dimension,
            expiry_reaper_interval: config.expiry_reaper_interval,
            community_detection_threshold: config.community_detection.auto_trigger_threshold,
            collections: tokio::sync::RwLock::new(ahash::AHashMap::new()),
            cancel_token: cancel_token.clone(),
            task_tracker: task_tracker.clone(),
            embedder: parking_lot::RwLock::new(None),
            orphan_registry,
            router: parking_lot::RwLock::new(None),
            calibrator: parking_lot::RwLock::new(None),
            pid_controller: parking_lot::RwLock::new(None),
        };

        db.initialize_collections().await?;

        let cleaned_intents =
            transaction::cleanup_orphaned_consolidation_intents(db.storage.as_ref(), &db.next_tx)
                .await?;
        tracing::debug!(cleaned_intents, "Orphaned consolidation cleanup on startup");

        db.repair_on_open().await?;

        db.init_embedding_backend(&config.embedding_backend).await?;

        let default_col = db.collection("default").await?;

        if config.consolidation_enabled {
            if let Some(launcher) = CONSOLIDATION_LAUNCHER.read().as_ref() {
                let handle = launcher(
                    default_col,
                    config.consolidation_interval,
                    config.max_llm_calls_per_cycle,
                    cancel_token.clone(),
                );
                task_tracker.spawn(async move {
                    if let Err(e) = handle.await {
                        tracing::warn!(error = %e, "ConsolidationEngine task failed or was cancelled");
                    }
                });
            }
        }

        Ok(db)
    }

    async fn init_embedding_backend(&self, backend: &EmbeddingBackend) -> Result<()> {
        match backend {
            EmbeddingBackend::Onnx {
                model_name,
                cache_dir,
            } => {
                #[cfg(feature = "onnx")]
                {
                    let model_dir = memfuse_infer_onnx::ensure_onnx_model_download(
                        model_name,
                        cache_dir.as_deref(),
                    )
                    .await?;
                    let embedder = memfuse_infer_onnx::OnnxEmbedder::load(model_dir)?;
                    let embedder_arc: Arc<dyn TextEmbeddingEngine> = Arc::new(embedder);
                    self.set_embedder(embedder_arc).await?;
                }
                #[cfg(not(feature = "onnx"))]
                {
                    let _ = (model_name, cache_dir);
                    tracing::debug!(
                        "EmbeddingBackend::Onnx requested, but 'onnx' feature is disabled in this build. \
                         Recompile with feature 'onnx' or 'reranking' to enable local ONNX embeddings."
                    );
                }
            }
            EmbeddingBackend::None => {}
        }
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self))]
    async fn initialize_collections(&self) -> Result<()> {
        let col_idx_prefix = b"__col_idx:\x00";
        let entries = self.storage.scan_prefix(col_idx_prefix).await?;
        for (k, _) in entries {
            let name_bytes = &k[col_idx_prefix.len()..];
            if let Ok(name) = String::from_utf8(name_bytes.to_vec()) {
                let _ = self.collection(&name).await?;
            }
        }
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn repair_on_open(&self) -> Result<()> {
        let timeout_duration = std::time::Duration::from_secs(30);
        match tokio::time::timeout(timeout_duration, self.repair_on_open_internal()).await {
            Ok(res) => res,
            Err(_) => {
                tracing::error!("repair_on_open: Startup recovery timed out after 30 seconds");
                Err(memfuse_core::MemFuseError::Storage(
                    "repair_on_open: Startup recovery timed out after 30s".to_string(),
                ))
            }
        }
    }

    async fn repair_on_open_internal(&self) -> Result<()> {
        let start_time = std::time::Instant::now();

        let collection_name_from_key = |key: &[u8]| -> String {
            if key.starts_with(b"__tx_intent:") {
                "default".to_string()
            } else if key.starts_with(b"__col:") {
                let rest = &key[6..];
                if let Some(pos) = rest.windows(3).position(|w| w == b":\x00\x03") {
                    if let Ok(name) = std::str::from_utf8(&rest[..pos]) {
                        return name.to_string();
                    }
                }
                "default".to_string()
            } else {
                "default".to_string()
            }
        };

        let pending_intents = self.scan_pending_intents().await?;

        if pending_intents.is_empty() {
            return Ok(());
        }

        tracing::warn!(
            "repair_on_open: found {} pending transaction intent(s), initiating recovery",
            pending_intents.len()
        );

        type PendingIntentItem = (Vec<u8>, Option<crate::transaction::CommitIntent>);
        type PendingIntentMap = ahash::AHashMap<String, Vec<PendingIntentItem>>;

        let mut intents_by_col: PendingIntentMap = ahash::AHashMap::new();

        for intent_key in pending_intents {
            let col_name = collection_name_from_key(&intent_key);
            let value_opt = self.storage.get(&intent_key).await?;
            let intent_opt = value_opt.and_then(|val| {
                serde_json::from_slice::<crate::transaction::CommitIntent>(&val).ok()
            });
            intents_by_col
                .entry(col_name)
                .or_default()
                .push((intent_key, intent_opt));
        }

        let mut total_repairs = 0u64;
        let mut repair_errors: Vec<String> = Vec::new();

        for (col_name, col_intents) in intents_by_col {
            let col = match self.collection(&col_name).await {
                Ok(c) => c,
                Err(e) => {
                    let err_msg = format!("Collection '{}' could not be loaded: {}", col_name, e);
                    tracing::error!("repair_on_open: {}", err_msg);
                    repair_errors.push(err_msg.clone());

                    let failed_intent =
                        crate::transaction::CommitIntent::Failed { reason: err_msg };
                    let failed_bytes = serde_json::to_vec(&failed_intent).unwrap_or_default();
                    for (intent_key, _) in col_intents {
                        if let Ok(tx) = self.allocate_tx() {
                            let _ = self.storage.put(tx, &intent_key, &failed_bytes).await;
                            let _ = self.storage.commit(tx).await;
                        }
                    }
                    continue;
                }
            };

            if let Err(e) = col.repair().await {
                let err_msg = format!("'{}': {}", col_name, e);
                tracing::error!(
                    "repair_on_open: Collection '{}' repair failed: {}",
                    col_name,
                    e
                );
                repair_errors.push(err_msg.clone());

                let failed_intent = crate::transaction::CommitIntent::Failed { reason: err_msg };
                let failed_bytes = serde_json::to_vec(&failed_intent).unwrap_or_default();
                for (intent_key, _) in col_intents {
                    if let Ok(tx) = self.allocate_tx() {
                        let _ = self.storage.put(tx, &intent_key, &failed_bytes).await;
                        let _ = self.storage.commit(tx).await;
                    }
                }
            } else {
                total_repairs += 1;

                let committed_intent = crate::transaction::CommitIntent::Committed;
                let committed_bytes = serde_json::to_vec(&committed_intent).unwrap_or_default();

                for (intent_key, intent_opt) in col_intents {
                    let mut should_abort = false;

                    if let Some(crate::transaction::CommitIntent::Pending { ref doc_ids, .. }) =
                        intent_opt
                    {
                        if !doc_ids.is_empty() {
                            let mut any_doc_exists = false;
                            for &doc_id in doc_ids.iter() {
                                let doc_key = col.namespaced_key(&doc_id.inner().to_le_bytes(), 1);
                                if col.storage.get(&doc_key).await.unwrap_or(None).is_some() {
                                    any_doc_exists = true;
                                    break;
                                }
                            }
                            if !any_doc_exists {
                                should_abort = true;
                            }
                        }
                    }

                    let final_bytes = if should_abort {
                        let abort_intent = crate::transaction::CommitIntent::Aborted;
                        serde_json::to_vec(&abort_intent).unwrap_or_default()
                    } else {
                        committed_bytes.clone()
                    };

                    if let Ok(tx) = self.allocate_tx() {
                        if let Err(e) = self.storage.put(tx, &intent_key, &final_bytes).await {
                            tracing::error!(
                                "repair_on_open: failed to write final intent state: {}",
                                e
                            );
                        } else if let Err(e) = self.storage.commit(tx).await {
                            tracing::error!(
                                "repair_on_open: failed to commit final intent state: {}",
                                e
                            );
                        }
                    }
                }
            }
        }

        let elapsed = start_time.elapsed();
        tracing::info!(
            "repair_on_open: completed in {:?} — collections verified: {}",
            elapsed,
            total_repairs
        );

        if !repair_errors.is_empty() {
            return Err(memfuse_core::MemFuseError::Storage(format!(
                "repair_on_open: {} Collection(s) konnten nach Crash nicht \
                 wiederhergestellt werden: {}. \
                 Datenbankintegrität nicht garantiert — manuelle Intervention erforderlich.",
                repair_errors.len(),
                repair_errors.join(", ")
            )));
        }

        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self))]
    async fn scan_pending_intents(&self) -> Result<Vec<Vec<u8>>> {
        let mut pending = Vec::new();

        let mut process_intent = |key: Vec<u8>, value: Vec<u8>| {
            if value == b"pending" {
                pending.push(key);
                return;
            }
            if let Ok(intent) = serde_json::from_slice::<crate::transaction::CommitIntent>(&value) {
                match intent {
                    crate::transaction::CommitIntent::Pending { .. } => pending.push(key),
                    crate::transaction::CommitIntent::Consolidation { .. } => {
                        pending.push(key);
                    }
                    _ => {}
                }
            }
        };

        let default_prefix = b"__tx_intent:";
        let entries = self.storage.scan_prefix(default_prefix).await?;
        for (key, value) in entries {
            process_intent(key, value);
        }

        let col_idx_prefix = b"__col_idx:\x00";
        let col_entries = self.storage.scan_prefix(col_idx_prefix).await?;
        for (k, _) in col_entries {
            let name_bytes = &k[col_idx_prefix.len()..];
            if let Ok(name) = String::from_utf8(name_bytes.to_vec()) {
                let prefix = format!("__col:{}:\x00", name);
                let mut ns_prefix = prefix.into_bytes();
                ns_prefix.push(3);
                let ns_entries = self.storage.scan_prefix(&ns_prefix).await?;
                for (ns_key, ns_value) in ns_entries {
                    process_intent(ns_key, ns_value);
                }
            }
        }

        Ok(pending)
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn collection(&self, name: &str) -> Result<Arc<Collection<LsmStorage>>> {
        self.collection_with_language(name, Language::English).await
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn collection_with_language(
        &self,
        name: &str,
        language: Language,
    ) -> Result<Arc<Collection<LsmStorage>>> {
        if name.len() > 64 {
            return Err(memfuse_core::MemFuseError::invalid_input(
                "Collection name too long (max 64)",
            ));
        }
        if !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            return Err(memfuse_core::MemFuseError::invalid_input(
                "Invalid characters in collection name",
            ));
        }

        let read_guard = self.collections.read().await;
        if let Some(col) = read_guard.get(name) {
            return Ok(Arc::clone(col));
        }
        drop(read_guard);

        let mut write_guard = self.collections.write().await;
        if let Some(col) = write_guard.get(name) {
            return Ok(Arc::clone(col));
        }

        let hnsw_config = HnswConfig {
            dimension: self.dimension,
            ..Default::default()
        };
        let index = Arc::new(HnswIndex::try_new(hnsw_config)?);

        let mut graph = memfuse_graph::CsrGraph::load_from_storage(self.storage.as_ref()).await?;
        graph.set_storage(self.storage.clone());
        let graph_index = Arc::new(graph);

        let mut col = Collection::new(
            name.to_string(),
            Arc::clone(&self.storage),
            index,
            graph_index,
            Arc::clone(&self.next_tx),
            self.dimension,
            language,
        );
        col.set_community_detection_trigger_threshold(self.community_detection_threshold);

        if let Some(emb) = self.embedder.read().as_ref() {
            col = col.with_embedder(Arc::clone(emb));
        }

        if name != "default" {
            let col_idx_key = [b"__col_idx:\x00", name.as_bytes()].concat();
            let tx = self.allocate_tx()?;
            self.storage.put(tx, &col_idx_key, b"{}").await?;
            self.storage.commit(tx).await?;
        }

        col.load_index().await?;
        col.load_text_stats().await?;
        col.migrate_doc_keys_v1().await?;

        let col_arc = Arc::new(col);
        write_guard.insert(name.to_string(), Arc::clone(&col_arc));

        let worker_handle = background_workers::start_expiry_cleanup_worker(
            Arc::clone(&col_arc),
            self.expiry_reaper_interval,
            self.cancel_token.clone(),
        );
        self.task_tracker.spawn(async move {
            if let Err(e) = worker_handle.await {
                tracing::warn!(error = %e, "Expiry cleanup worker task failed or was cancelled");
            }
        });

        Ok(col_arc)
    }

    pub fn allocate_tx(&self) -> Result<TxId> {
        let id = self.next_tx.fetch_add(1, Ordering::SeqCst);
        if id > TxId::MAX_COLLECTION_SEQUENCE {
            return Err(memfuse_core::MemFuseError::Transaction(
                "TxId counter exhausted: MAX_COLLECTION_SEQUENCE range exceeded. Collection must be recreated.".into(),
            ));
        }
        Ok(TxId::new(id))
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn list_collections(&self) -> Result<Vec<String>> {
        let col_idx_prefix = b"__col_idx:\x00";
        let entries = self.storage.scan_prefix(col_idx_prefix).await?;

        let mut names = std::collections::HashSet::new();
        names.insert("default".to_string());

        for (k, _) in entries {
            let name_bytes = &k[col_idx_prefix.len()..];
            if let Ok(name) = String::from_utf8(name_bytes.to_vec()) {
                names.insert(name);
            }
        }

        let guard = self.collections.read().await;
        for name in guard.keys() {
            names.insert(name.clone());
        }

        let mut sorted_names: Vec<String> = names.into_iter().collect();
        sorted_names.sort();
        Ok(sorted_names)
    }

    #[tracing::instrument(level = "trace", skip(self, proof_key))]
    pub async fn drop_collection(
        &self,
        name: &str,
        tenant_id: TenantId,
        proof_key: &[u8],
    ) -> Result<DeletionProof> {
        if name == "default" {
            return Err(memfuse_core::MemFuseError::invalid_input(
                "Cannot drop default collection",
            ));
        }

        let col_data_prefix = format!("__col:{}:", name);
        let txt_data_prefix = format!("__txt:{}:", name);
        let col_idx_key = [b"__col_idx:\x00", name.as_bytes()].concat();

        let mut deleted_keys: Vec<Vec<u8>> = Vec::new();

        let col_entries = self.storage.scan_prefix(col_data_prefix.as_bytes()).await?;
        for (k, _) in col_entries {
            deleted_keys.push(k);
        }

        let txt_entries = self.storage.scan_prefix(txt_data_prefix.as_bytes()).await?;
        for (k, _) in txt_entries {
            deleted_keys.push(k);
        }

        deleted_keys.push(col_idx_key.clone());

        let tx = self.allocate_tx()?;

        self.storage
            .delete_prefix(tx, col_data_prefix.as_bytes())
            .await?;

        self.storage
            .delete_prefix(tx, txt_data_prefix.as_bytes())
            .await?;

        self.storage.delete(tx, &col_idx_key).await?;

        self.storage.commit(tx).await?;

        let remaining_col_data = self.storage.scan_prefix(col_data_prefix.as_bytes()).await?;
        let remaining_txt_data = self.storage.scan_prefix(txt_data_prefix.as_bytes()).await?;

        let mut hasher = blake3::Hasher::new();
        hasher.update(name.as_bytes());
        let hash_bytes = hasher.finalize();
        let col_id_u64 =
            u64::from_le_bytes(hash_bytes.as_bytes()[0..8].try_into().unwrap_or([1; 8]));
        let collection_id = CollectionId::try_new(col_id_u64).unwrap_or(CollectionId::new(1));

        let scope = DeletionScope::Collection {
            collection_id,
            tenant_id,
        };

        let layer_proofs = vec![
            LayerCleanupProof::new_after_verified_empty(
                DeletionLayer::LsmMemtable,
                remaining_col_data.len(),
            ),
            LayerCleanupProof::new_after_verified_empty(
                DeletionLayer::SsTableAllLevels,
                remaining_txt_data.len(),
            ),
        ]
        .into_iter()
        .collect::<Result<Vec<_>>>()
        .map_err(|e| {
            memfuse_core::MemFuseError::Internal(format!(
                "CRITICAL: Collection '{name}' was physically sanitized and committed at tx {}, but DeletionProof generation failed: {e}. Data is permanently deleted.",
                tx.inner()
            ))
        })?;

        let proof = DeletionProof::create(
            scope,
            deleted_keys,
            tx,
            layer_proofs,
            vec![],
            proof_key,
        )
        .map_err(|e| {
            memfuse_core::MemFuseError::Internal(format!(
                "CRITICAL: Collection '{name}' was physically sanitized and committed at tx {}, but DeletionProof generation failed: {e}. Data is permanently deleted.",
                tx.inner()
            ))
        })?;

        self.collections.write().await.remove(name);

        Ok(proof)
    }

    async fn default_col(&self) -> Result<Arc<Collection<LsmStorage>>> {
        self.collection("default").await
    }

    #[tracing::instrument(level = "trace", skip(self, value))]
    pub async fn put_kv(&self, id: &str, value: &Value) -> Result<()> {
        self.default_col().await?.put_kv(id, value).await
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn get_kv(&self, id: &str) -> Result<Option<Value>> {
        self.default_col().await?.get_kv(id).await
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn insert(&self, id: &str, embedding: &[f32], metadata: Option<Value>) -> Result<()> {
        self.default_col()
            .await?
            .insert(id, embedding, metadata)
            .await
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn insert_typed(
        &self,
        collection_name: &str,
        id: &str,
        embedding: &[f32],
        memory_type: memfuse_core::MemoryType,
        metadata: Option<Value>,
    ) -> Result<()> {
        self.collection(collection_name)
            .await?
            .insert_typed(id, embedding, memory_type, metadata)
            .await
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn upsert(&self, id: &str, embedding: &[f32], metadata: Option<Value>) -> Result<()> {
        self.default_col()
            .await?
            .upsert(id, embedding, metadata)
            .await
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn insert_many(&self, docs: &[(String, Vec<f32>, Option<Value>)]) -> Result<()> {
        self.default_col().await?.insert_many(docs).await
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn upsert_many(&self, docs: &[(String, Vec<f32>, Option<Value>)]) -> Result<()> {
        self.default_col().await?.upsert_many(docs).await
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn get(&self, id: &str) -> Result<Option<Document>> {
        self.default_col().await?.get(id).await
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn get_at_snapshot(&self, id: &str, seq_no: u64) -> Result<Option<Document>> {
        self.default_col().await?.get_at_snapshot(id, seq_no).await
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn last_committed_seq(&self) -> Result<u64> {
        self.storage.last_seq_no().await
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn create_snapshot(&self) -> Result<u64> {
        self.storage.last_seq_no().await
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn update(&self, id: &str, embedding: &[f32], metadata: Option<Value>) -> Result<()> {
        self.default_col()
            .await?
            .update(id, embedding, metadata)
            .await
    }

    #[allow(deprecated)]
    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn search(&self, query: &[f32], k: usize) -> Result<Vec<SearchResult>> {
        self.default_col()
            .await?
            .query()
            .embedding(query)
            .k(k)
            .execute()
            .await
    }

    #[deprecated(
        since = "0.1.0",
        note = "Use search_with_filter_expr with memfuse_core::FilterExpr directly"
    )]
    #[allow(deprecated)]
    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn search_with_filter(
        &self,
        query: &[f32],
        k: usize,
        filter: Option<MetadataFilter>,
    ) -> Result<Vec<SearchResult>> {
        let col = self.default_col().await?;
        let mut builder = col.query().vector(query).k(k);
        if let Some(f) = filter {
            builder = builder.metadata_filter(f);
        }
        builder.execute().await
    }

    #[allow(deprecated)]
    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn search_with_filter_expr(
        &self,
        query: &[f32],
        k: usize,
        filter: Option<FilterExpr>,
    ) -> Result<Vec<SearchResult>> {
        let col = self.default_col().await?;
        let mut builder = col.query().embedding(query).k(k);
        if let Some(f) = filter {
            builder = builder.filter(f);
        }
        builder.execute().await
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn insert_text_only(
        &self,
        id: &str,
        text: &str,
        metadata: Option<Value>,
    ) -> Result<()> {
        self.default_col()
            .await?
            .insert_text_only(id, text, metadata)
            .await
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn upsert_text_only(
        &self,
        id: &str,
        text: &str,
        metadata: Option<Value>,
    ) -> Result<()> {
        self.default_col()
            .await?
            .upsert_text_only(id, text, metadata)
            .await
    }

    #[allow(deprecated)]
    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn search_text(&self, text: &str, k: usize) -> Result<Vec<SearchResult>> {
        self.default_col()
            .await?
            .query()
            .text(text)
            .k(k)
            .execute()
            .await
    }

    #[allow(deprecated)]
    #[tracing::instrument(level = "trace", skip(self, filter))]
    pub async fn search_filtered(
        &self,
        query: &[f32],
        k: usize,
        filter: Option<&(dyn Fn(DocId) -> bool + Send + Sync)>,
    ) -> Result<Vec<SearchResult>> {
        #[allow(deprecated)]
        self.default_col()
            .await?
            .search_filtered(query, k, filter)
            .await
    }

    #[allow(deprecated)]
    #[tracing::instrument(level = "trace", skip(self, anchor_entities))]
    pub async fn hybrid_search(
        &self,
        text: &str,
        vector: &[f32],
        k: usize,
        anchor_entities: Option<&[memfuse_core::EntityId]>,
    ) -> Result<Vec<SearchResult>> {
        let col = self.default_col().await?;
        let mut builder = col.query().text(text).vector(vector).k(k);
        if let Some(anchors) = anchor_entities {
            builder = builder.anchors(anchors.iter().copied());
        }
        builder.execute().await
    }

    #[cfg(feature = "reranking")]
    #[allow(deprecated)]
    #[tracing::instrument(level = "trace", skip(self, reranker, anchor_entities))]
    pub async fn hybrid_search_reranked(
        &self,
        text: &str,
        vector: &[f32],
        k: usize,
        reranker: Option<&memfuse_infer_onnx::CrossEncoderReranker>,
        anchor_entities: Option<&[memfuse_core::EntityId]>,
    ) -> Result<Vec<SearchResult>> {
        let col = self.default_col().await?;
        let mut builder = col.query().text(text).vector(vector).k(k);
        if let Some(r) = reranker {
            builder = builder.reranker(r);
        }
        if let Some(anchors) = anchor_entities {
            builder = builder.anchors(anchors.iter().copied());
        }
        builder.execute().await
    }

    #[allow(deprecated)]
    #[tracing::instrument(level = "trace", skip(self, anchor_entities, weights))]
    pub async fn hybrid_search_with_weights(
        &self,
        text: &str,
        vector: &[f32],
        k: usize,
        anchor_entities: Option<&[memfuse_core::EntityId]>,
        weights: Option<&memfuse_core::FusionWeights>,
    ) -> Result<Vec<SearchResult>> {
        let col = self.default_col().await?;
        let mut builder = col.query().text(text).vector(vector).k(k);
        if let Some(w) = weights {
            builder = builder.fusion_weights(w.clone());
        }
        if let Some(anchors) = anchor_entities {
            builder = builder.anchors(anchors.iter().copied());
        }
        builder.execute().await
    }

    #[allow(deprecated)]
    #[tracing::instrument(level = "trace", skip(self, anchor_entities, weights, strategy))]
    pub async fn hybrid_search_with_strategy(
        &self,
        text: &str,
        vector: &[f32],
        k: usize,
        anchor_entities: Option<&[memfuse_core::EntityId]>,
        weights: Option<&memfuse_core::FusionWeights>,
        strategy: Option<&memfuse_core::GraphTraversalStrategy>,
    ) -> Result<Vec<SearchResult>> {
        let col = self.default_col().await?;
        let mut builder = col.query().text(text).vector(vector).k(k);
        if let Some(w) = weights {
            builder = builder.fusion_weights(w.clone());
        }
        if let Some(s) = strategy {
            builder = builder.strategy(s.clone());
        }
        if let Some(anchors) = anchor_entities {
            builder = builder.anchors(anchors.iter().copied());
        }
        builder.execute().await
    }

    #[allow(deprecated)]
    #[tracing::instrument(level = "trace", skip(self, query))]
    pub async fn hybrid_search_with_query(
        &self,
        query: &memfuse_core::HybridQuery,
    ) -> Result<Vec<SearchResult>> {
        self.default_col()
            .await?
            .query()
            .query_config(query)
            .execute()
            .await
    }

    pub async fn delete(&self, id: &str) -> Result<()> {
        self.default_col().await?.delete(id).await
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn relate(&self, from: &str, to: &str, label: &str) -> Result<()> {
        let col = self.default_col().await?;
        col.relate(from, to, label).await
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn relate_bidirectional(&self, from: &str, to: &str, label: &str) -> Result<()> {
        let col = self.default_col().await?;
        col.relate_bidirectional(from, to, label).await
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn scan_prefix(
        &self,
        prefix: &str,
        limit: Option<usize>,
    ) -> Result<Vec<(String, Value)>> {
        self.default_col().await?.scan_prefix(prefix, limit).await
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn len(&self) -> Result<usize> {
        Ok(self.default_col().await?.len().await)
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn is_empty(&self) -> Result<bool> {
        Ok(self.default_col().await?.is_empty().await)
    }

    #[tracing::instrument(level = "trace", skip(self, start, end))]
    pub async fn scan(
        &self,
        start: std::ops::Bound<&[u8]>,
        end: std::ops::Bound<&[u8]>,
        limit: Option<usize>,
    ) -> Result<Vec<(String, Value)>> {
        self.default_col().await?.scan(start, end, limit).await
    }

    pub fn set_router(&self, router: std::sync::Weak<dyn DriftStatusProvider>) {
        *self.router.write() = Some(router);
    }

    pub fn set_calibrator(
        &self,
        calibrator: std::sync::Weak<parking_lot::Mutex<memfuse_calibration::IsotonicCalibrator>>,
    ) {
        *self.calibrator.write() = Some(calibrator);
    }

    pub fn set_pid_controller(
        &self,
        pid_controller: std::sync::Weak<parking_lot::Mutex<memfuse_calibration::PidController>>,
    ) {
        *self.pid_controller.write() = Some(pid_controller);
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn stats(&self) -> Result<MemFuseStats> {
        let default_col = self.default_col().await?;
        let active_memory_count = default_col.len().await;
        let index_stats = default_col.stats().await?;
        let storage_stats = self.storage.stats().await?;

        let drift_status = self
            .router
            .read()
            .as_ref()
            .and_then(|w| w.upgrade())
            .map(|r| r.overall_drift_status())
            .unwrap_or_else(|| "nicht verfügbar".to_string());

        let (calibration_ece, last_calibration_at) = self
            .calibrator
            .read()
            .as_ref()
            .and_then(|w| w.upgrade())
            .map(|c| {
                let guard = c.lock();
                (guard.cached_ece(), guard.last_calibration_at())
            })
            .unwrap_or((None, None));

        let pid_pool_size = self
            .pid_controller
            .read()
            .as_ref()
            .and_then(|w| w.upgrade())
            .map(|p| p.lock().current_pool_size())
            .unwrap_or(None);

        Ok(MemFuseStats {
            drift_status,
            calibration_ece,
            last_calibration_at,
            active_memory_count,
            pid_pool_size,
            index_stats,
            storage_stats,
        })
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn flush(&self) -> Result<()> {
        self.storage.flush().await?;
        Ok(())
    }

    pub fn shutdown(&self) {
        self.cancel_token.cancel();
    }

    pub async fn wait_shutdown(&self) {
        self.shutdown();
        self.task_tracker.close();
        self.task_tracker.wait().await;
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn close(self) -> Result<()> {
        self.wait_shutdown().await;
        self.storage.close().await?;
        self.flush().await?;
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self, embedder))]
    pub async fn with_embedder(self, embedder: Arc<dyn TextEmbeddingEngine>) -> Self {
        {
            let mut guard = self.embedder.write();
            *guard = Some(Arc::clone(&embedder));
        }
        let cols = self.collections.read().await;
        if let Some(col) = cols.get("default") {
            *col.embedder.write() = Some(embedder);
        }
        drop(cols);

        self
    }

    #[tracing::instrument(level = "trace", skip(self, embedder))]
    pub async fn set_embedder(&self, embedder: Arc<dyn TextEmbeddingEngine>) -> Result<()> {
        {
            let mut guard = self.embedder.write();
            *guard = Some(Arc::clone(&embedder));
        }
        let collections_read = self.collections.read().await;
        if let Some(col) = collections_read.get("default") {
            let mut guard = col.embedder.write();
            *guard = Some(embedder);
        }
        Ok(())
    }

    pub fn orphan_registry(&self) -> &Arc<memfuse_checkpoint::InstanceOrphanRegistry> {
        &self.orphan_registry
    }
}

pub use memfuse_core::DistanceMetric;
pub use serde_json::json;

impl MemFuse {
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
impl SandboxBridge for MemFuse {
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
                            memfuse_core::MemFuseError::Serialization("Query too short".into())
                        })?
                        .try_into()
                        .map_err(|_| {
                            memfuse_core::MemFuseError::Serialization("Invalid slice".into())
                        })?,
                );
                vector.push(f32::from_bits(bits));
            }

            let results: Vec<SearchResult> = self.search(&vector, k).await?;
            serde_json::to_vec(&results)
                .map_err(|e| memfuse_core::MemFuseError::Internal(e.to_string()))
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
                        memfuse_core::MemFuseError::Internal(e.to_string())
                    })?))
                }
                None => Ok(None),
            }
        })
    }
}
