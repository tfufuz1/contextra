use crate::*;
use contextra_core::{Result, TxId};
use contextra_store::LsmStorage;
use std::path::Path;
use std::sync::Arc;

impl Contextra {
    #[tracing::instrument(level = "trace", skip(path))]
    pub async fn open(path: impl AsRef<Path>) -> Result<Self> {
        Self::open_with_config(path, ContextraConfig::default()).await
    }

    #[tracing::instrument(level = "trace", skip(path, config))]
    pub async fn open_with_config(path: impl AsRef<Path>, config: ContextraConfig) -> Result<Self> {
        if config.dimension == 0 {
            return Err(contextra_core::ContextraError::invalid_input(
                "dimension must be > 0",
            ));
        }
        if config.dimension > 65536 {
            return Err(contextra_core::ContextraError::invalid_input(
                "dimension exceeds maximum",
            ));
        }

        let lsm_config = contextra_store::LsmConfig {
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
                        return Err(contextra_core::ContextraError::invalid_input(format!(
                            "Dimension mismatch: DB wurde mit dim={} erstellt, \
                             Config fordert dim={}. \
                             Passe ContextraConfig::dimension an oder nutze eine neue DB.",
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
        let orphan_registry = Arc::new(contextra_checkpoint::InstanceOrphanRegistry::new(
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
                    let model_dir = contextra_infer_onnx::ensure_onnx_model_download(
                        model_name,
                        cache_dir.as_deref(),
                    )
                    .await?;
                    let embedder = contextra_infer_onnx::OnnxEmbedder::load(model_dir)?;
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
                Err(contextra_core::ContextraError::Storage(
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
            return Err(contextra_core::ContextraError::Storage(format!(
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

}
