use crate::collection::crud::{AutoExtractionConfig, AutoExtractionMode};
use crate::*;
use contextra_store::LsmStorage;
use contextra_types::{Result, TxId};
use std::sync::Arc;

/// Derives a [`CollectionId`] from a raw `u64` hash prefix value.
///
/// # Errors
/// Returns [`ContextraError::InvalidInput`](contextra_types::ContextraError::InvalidInput) if `prefix` is 0.
pub fn collection_id_from_hash_prefix(prefix: u64) -> Result<CollectionId> {
    CollectionId::try_new(prefix)
}

/// Derives a [`CollectionId`] deterministically from a collection name using the first 8 bytes of BLAKE3 hash.
///
/// # Errors
/// Returns [`ContextraError::InvalidInput`](contextra_types::ContextraError::InvalidInput) if `name` is empty or if derived ID is 0.
pub fn derive_collection_id(name: &str) -> Result<CollectionId> {
    if name.is_empty() {
        return Err(contextra_types::ContextraError::invalid_input(
            "Collection name cannot be empty",
        ));
    }
    let mut hasher = blake3::Hasher::new();
    hasher.update(name.as_bytes());
    let hash_bytes = hasher.finalize();
    let col_id_u64 =
        u64::from_le_bytes(hash_bytes.as_bytes()[0..8].try_into().map_err(|_| {
            contextra_types::ContextraError::invalid_input("Hash truncation failed")
        })?);
    collection_id_from_hash_prefix(col_id_u64)
}

impl Contextra {
    /// Derives a [`CollectionId`] from a raw `u64` hash prefix value.
    pub fn collection_id_from_hash_prefix(prefix: u64) -> Result<CollectionId> {
        collection_id_from_hash_prefix(prefix)
    }

    /// Derives a [`CollectionId`] deterministically from a collection name using BLAKE3.
    pub fn derive_collection_id(name: &str) -> Result<CollectionId> {
        derive_collection_id(name)
    }
    pub async fn collection_for_tenant(
        &self,
        name: &str,
        tenant_id: TenantId,
    ) -> Result<Arc<Collection<contextra_store::tenant_codec::TenantScopedStorage<Arc<LsmStorage>>>>>
    {
        self.collection_with_language_and_tenant(name, Language::English, tenant_id)
            .await
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn collection_with_language_and_tenant(
        &self,
        name: &str,
        language: Language,
        tenant_id: TenantId,
    ) -> Result<Arc<Collection<contextra_store::tenant_codec::TenantScopedStorage<Arc<LsmStorage>>>>>
    {
        if name.len() > 64 {
            return Err(contextra_types::ContextraError::invalid_input(
                "Collection name too long (max 64)",
            ));
        }
        if !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            return Err(contextra_types::ContextraError::invalid_input(
                "Invalid characters in collection name",
            ));
        }

        let cache_key = (tenant_id, name.to_string());

        let read_guard = self.tenant_collections.read().await;
        if let Some(col) = read_guard.get(&cache_key) {
            return Ok(Arc::clone(col));
        }
        drop(read_guard);

        let tenant_storage = Arc::new(contextra_store::tenant_codec::TenantScopedStorage::new(
            self.storage.clone(),
            tenant_id,
        ));

        let hnsw_config = HnswConfig {
            dimension: self.dimension,
            ..Default::default()
        };
        let index = Arc::new(HnswIndex::try_new(hnsw_config)?);

        let mut graph =
            contextra_graph::CsrGraph::load_from_storage(tenant_storage.as_ref()).await?;
        graph.set_storage(tenant_storage.clone());
        let graph_index = Arc::new(graph);

        let mut col = Collection::new(
            name.to_string(),
            tenant_storage.clone(),
            index,
            graph_index,
            Arc::clone(&self.next_tx),
            self.dimension,
            language,
        );
        col.set_community_detection_trigger_threshold(self.community_detection_threshold);
        col.set_metrics_sink(self.metrics_sink.read().clone());

        if let Some(emb) = self.embedder.read().as_ref() {
            col = col.with_embedder(Arc::clone(emb));
        }

        if let Some(hooks) = self.kv_hooks() {
            col.set_kv_hooks(hooks);
        }

        if name != "default" {
            let col_idx_key = [b"__col_idx:\x00", name.as_bytes()].concat();
            let tx = self.allocate_tx()?;
            tenant_storage.put(tx, &col_idx_key, b"{}").await?;
            tenant_storage.commit(tx).await?;
        }

        col.load_index().await?;
        col.load_text_stats().await?;
        col.migrate_doc_keys_v1().await?;

        let auto_cfg =
            AutoExtractionConfig::for_regulated(false).with_mode(AutoExtractionMode::Enabled);
        col.set_auto_extraction_config(auto_cfg);

        let col_arc = Arc::new(col);

        let mut write_guard = self.tenant_collections.write().await;
        if let Some(existing) = write_guard.get(&cache_key) {
            return Ok(Arc::clone(existing));
        }
        write_guard.insert(cache_key, Arc::clone(&col_arc));

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

    /// Retrieves or creates an un-tenanted [`Collection`] handle (Admin / Single-Tenant path).
    ///
    /// # Errors
    ///
    /// Returns [`ContextraError::InvalidInput`] if [`TenantPolicy::Required`] is configured.
    pub async fn collection(&self, name: &str) -> Result<Arc<Collection<LsmStorage>>> {
        self.collection_with_language(name, Language::English).await
    }

    /// Retrieves or creates an un-tenanted [`Collection`] handle with specified language (Admin / Single-Tenant path).
    ///
    /// # Errors
    ///
    /// Returns [`ContextraError::InvalidInput`] if [`TenantPolicy::Required`] is configured.
    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn collection_with_language(
        &self,
        name: &str,
        language: Language,
    ) -> Result<Arc<Collection<LsmStorage>>> {
        if self.config.tenant_policy == TenantPolicy::Required {
            return Err(contextra_types::ContextraError::invalid_input(
                "Non-tenant collection access is forbidden when TenantPolicy::Required is active. Use collection_for_tenant instead.",
            ));
        }

        if name.len() > 64 {
            return Err(contextra_types::ContextraError::invalid_input(
                "Collection name too long (max 64)",
            ));
        }
        if !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            return Err(contextra_types::ContextraError::invalid_input(
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

        let mut graph = contextra_graph::CsrGraph::load_from_storage(self.storage.as_ref()).await?;
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
        col.set_metrics_sink(self.metrics_sink.read().clone());

        if let Some(emb) = self.embedder.read().as_ref() {
            col = col.with_embedder(Arc::clone(emb));
        }

        if let Some(hooks) = self.kv_hooks() {
            col.set_kv_hooks(hooks);
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

        let auto_cfg =
            AutoExtractionConfig::for_regulated(false).with_mode(AutoExtractionMode::Enabled);
        col.set_auto_extraction_config(auto_cfg);

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
            return Err(contextra_types::ContextraError::Transaction(
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

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn list_collections_for_tenant(&self, tenant_id: TenantId) -> Result<Vec<String>> {
        let tenant_storage = contextra_store::tenant_codec::TenantScopedStorage::new(
            self.storage.clone(),
            tenant_id,
        );
        let col_idx_prefix = b"__col_idx:\x00";
        let entries = tenant_storage.scan_prefix(col_idx_prefix).await?;

        let mut names = std::collections::HashSet::new();
        names.insert("default".to_string());

        for (k, _) in entries {
            let name_bytes = &k[col_idx_prefix.len()..];
            if let Ok(name) = String::from_utf8(name_bytes.to_vec()) {
                names.insert(name);
            }
        }

        let mut sorted_names: Vec<String> = names.into_iter().collect();
        sorted_names.sort();
        Ok(sorted_names)
    }

    /// Drops a collection and issues a [`DeletionProof`] attesting to logical LSM key space deletion.
    ///
    /// # Limitations / Non-Guarantees
    /// - Does NOT guarantee immediate physical purge or removal of SSTable files, WAL segments, HNSW index files, or CSR graph data files from disk storage.
    /// - Does NOT guarantee hardware physical SSD flash controller level sanitization (wear leveling / TRIM depend on OS & hardware).
    /// - Attests strictly to logical deletion (tombstones committed and verified empty in active MemTable/LSM key space scans).
    #[tracing::instrument(level = "trace", skip(self, proof_key))]
    pub async fn drop_collection(
        &self,
        name: &str,
        tenant_id: TenantId,
        proof_key: &[u8],
    ) -> Result<DeletionProof> {
        if name == "default" {
            return Err(contextra_types::ContextraError::invalid_input(
                "Cannot drop default collection",
            ));
        }

        let collection_id = derive_collection_id(name)?;

        let tenant_storage = contextra_store::tenant_codec::TenantScopedStorage::new(
            self.storage.clone(),
            tenant_id,
        );

        let col_data_prefix = format!("__col:{}:", name);
        let txt_data_prefix = format!("__txt:{}:", name);
        let col_idx_key = [b"__col_idx:\x00", name.as_bytes()].concat();

        let mut deleted_keys: Vec<Vec<u8>> = Vec::new();

        let col_entries = tenant_storage
            .scan_prefix(col_data_prefix.as_bytes())
            .await?;
        for (k, _) in col_entries {
            deleted_keys.push(k);
        }

        let txt_entries = tenant_storage
            .scan_prefix(txt_data_prefix.as_bytes())
            .await?;
        for (k, _) in txt_entries {
            deleted_keys.push(k);
        }

        deleted_keys.push(col_idx_key.clone());

        let tx = self.allocate_tx()?;

        tenant_storage
            .delete_prefix(tx, col_data_prefix.as_bytes())
            .await?;

        tenant_storage
            .delete_prefix(tx, txt_data_prefix.as_bytes())
            .await?;

        tenant_storage.delete(tx, &col_idx_key).await?;

        tenant_storage.commit(tx).await?;

        // Durability Audit Note (Review P02 / F-03 / Invariante I-2 & I-5):
        // `tenant_storage.commit(tx).await` executes `LsmStorage::commit`, which dispatches WAL ops to the WAL flusher
        // actor and awaits oneshot ACK (`file.sync_all()`) before returning `Ok(())` (both in single commit and group
        // commit leader/follower paths). Thus, disk durability (`fsync`) of all deletion tombstones is strictly
        // guaranteed BEFORE `DeletionProof::create` is invoked and returned to caller.

        let remaining_col_data = tenant_storage
            .scan_prefix(col_data_prefix.as_bytes())
            .await?;
        let remaining_txt_data = tenant_storage
            .scan_prefix(txt_data_prefix.as_bytes())
            .await?;

        let scope = DeletionScope::Collection {
            collection_id,
            tenant_id,
        };

        // LSM deletion via `delete_prefix` is tombstone-only.
        // Post-commit prefix scans verify logical key space emptiness in active MemTable / LSM state.
        // SSTable compaction and WAL truncation are asynchronous background operations; existing SSTable files
        // and WAL segments may still contain physical bytes until full compaction/truncation occurs.
        // Therefore, drop_collection claims ONLY `DeletionLayer::LsmMemtable`.
        // `SsTableAllLevels` or `WalAllSegments` may only be re-added together with a real physical verifier that inspects disk storage.
        let layer_proofs = vec![LayerCleanupProof::new_after_verified_empty(
            DeletionLayer::LsmMemtable,
            remaining_col_data.len() + remaining_txt_data.len(),
        )]
        .into_iter()
        .collect::<Result<Vec<_>>>()
        .map_err(|e| {
            contextra_types::ContextraError::Internal(format!(
                "CRITICAL: Collection '{name}' was tombstoned and committed at tx {}, but DeletionProof generation failed: {e}. Data is permanently deleted.",
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
            contextra_types::ContextraError::Internal(format!(
                "CRITICAL: Collection '{name}' was tombstoned and committed at tx {}, but DeletionProof generation failed: {e}. Data is permanently deleted.",
                tx.inner()
            ))
        })?;

        self.collections.write().await.remove(name);
        self.tenant_collections
            .write()
            .await
            .remove(&(tenant_id, name.to_string()));

        Ok(proof)
    }

    /// Purges all cached KV segments and stored collection data for the given tenant across the engine.
    ///
    /// # Limitations / Non-Guarantees (SCHRITT 4)
    /// - Does NOT purge unmanaged hardware RAM or OS memory pages outside zeroized buffers.
    /// - Does NOT purge raw LLM internal model activations or external prompt residues outside Contextra.
    /// - Does NOT guarantee hardware physical SSD flash controller level sanitization (wear leveling / TRIM depend on OS & hardware).
    /// - Does NOT clear external unmanaged block caches.
    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn purge_tenant(&self, tenant_id: TenantId) -> Result<()> {
        // 0. Query collections for tenant to log and inspect before purge
        let _tenant_cols = self
            .list_collections_for_tenant(tenant_id)
            .await
            .unwrap_or_default();

        // 1. Notify KV lifecycle hooks on Contextra and across active collections
        if let Some(hooks) = self.kv_hooks() {
            hooks.purge_tenant(tenant_id);
        }

        let read_guard = self.tenant_collections.read().await;
        for ((t, _), col) in read_guard.iter() {
            if *t == tenant_id {
                if let Some(hooks) = col.kv_hooks() {
                    hooks.purge_tenant(tenant_id);
                }
            }
        }
        drop(read_guard);

        // 2. Delete all LSM storage entries for tenant
        let tenant_storage = contextra_store::tenant_codec::TenantScopedStorage::new(
            self.storage.clone(),
            tenant_id,
        );
        let tx = self.allocate_tx()?;
        tenant_storage.delete_prefix(tx, b"").await?;
        tenant_storage.commit(tx).await?;

        // 3. Remove cached collection handles for this tenant
        let mut write_guard = self.tenant_collections.write().await;
        write_guard.retain(|(t, _), _| *t != tenant_id);

        Ok(())
    }
}
