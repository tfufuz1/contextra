use crate::*;
use contextra_store::LsmStorage;
use contextra_types::{Result, TxId};
use std::sync::Arc;

impl Contextra {
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
            contextra_types::ContextraError::Internal(format!(
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
            contextra_types::ContextraError::Internal(format!(
                "CRITICAL: Collection '{name}' was physically sanitized and committed at tx {}, but DeletionProof generation failed: {e}. Data is permanently deleted.",
                tx.inner()
            ))
        })?;

        self.collections.write().await.remove(name);

        Ok(proof)
    }
}
