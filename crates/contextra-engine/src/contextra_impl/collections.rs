use crate::*;
use contextra_store::LsmStorage;
use contextra_types::{Result, TxId};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Status of physical purge attempt during collection drop.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PhysicalPurgeStatus {
    /// Physical purge was successfully performed.
    Performed {
        /// Total number of records purged.
        records_purged: u64,
        /// Total bytes reclaimed during physical compaction.
        bytes_reclaimed: u64,
    },
    /// Physical purge was not performed or was blocked.
    NotPerformed {
        /// Reason why physical purge was not executed or was blocked.
        reason: String,
    },
}

/// Report summarizing the result of dropping a collection, including deletion proof and physical purge status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DropCollectionReport {
    /// Cryptographic deletion proof attesting to logical (and optionally physical) deletion.
    pub proof: DeletionProof,
    /// Physical purge execution status.
    pub physical_purge: PhysicalPurgeStatus,
}

fn hex_encode_32_col(bytes: &[u8; 32]) -> String {
    let mut s = String::with_capacity(64);
    for b in bytes {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

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

        if name != "default" {
            let col_idx_key = [b"__col_idx:\x00", name.as_bytes()].concat();
            let tx = self.allocate_tx()?;
            tenant_storage.put(tx, &col_idx_key, b"{}").await?;
            tenant_storage.commit(tx).await?;
        }

        col.load_index().await?;
        col.load_text_stats().await?;
        col.migrate_doc_keys_v1().await?;

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
        TxId::try_new(id)
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

    /// Drops a collection and returns a [`DropCollectionReport`] summarizing deletion proof and physical purge status.
    ///
    /// # Limitations / Non-Guarantees
    /// - Physical purge execution depends on LSM compaction availability (`LsmStorage::purge_keys`).
    /// - If physical purge is blocked or unimplemented, `physical_purge` is set to [`PhysicalPurgeStatus::NotPerformed`]
    ///   and `covered_layers` in the deletion proof strictly claims only logical [`DeletionLayer::LsmMemtable`].
    /// - Does NOT guarantee hardware physical SSD flash controller level sanitization (wear leveling / TRIM depend on OS & hardware).
    #[tracing::instrument(level = "trace", skip(self, proof_key))]
    pub async fn drop_collection_with_report(
        &self,
        name: &str,
        tenant_id: TenantId,
        proof_key: &[u8],
    ) -> Result<DropCollectionReport> {
        if proof_key.len() < 32 || proof_key.iter().all(|&b| b == 0) {
            return Err(contextra_types::ContextraError::invalid_input(
                "Proof key must be at least 32 bytes long and cannot be all zeroes",
            ));
        }

        if name == "default" {
            return Err(contextra_types::ContextraError::invalid_input(
                "Cannot drop default collection",
            ));
        }

        let _collection_id = derive_collection_id(name)?;

        // Schließe das Scan-Commit-Fenster: Lock-Reihenfolge collections -> kv_locks -> embedder (INV-DELETION-1).
        // Entfernen aus der Registry vor dem Scannen verhindert, dass ein nebenläufiger Schreiber
        // eine neue Referenz zur Collection holt und während des Löschens schreibt.
        self.collections.write().await.remove(name);
        self.tenant_collections
            .write()
            .await
            .remove(&(tenant_id, name.to_string()));

        let tenant_storage = contextra_store::tenant_codec::TenantScopedStorage::new(
            self.storage.clone(),
            tenant_id,
        );

        let col_data_prefix = format!("__col:{}:", name);
        let txt_data_prefix = format!("__txt:{}:", name);
        let col_idx_key = [b"__col_idx:\x00", name.as_bytes()].concat();

        let tx = self.allocate_tx()?;

        // Scanne Schlüssel am konsistenten Snapshot-Stand tx.inner()
        let col_entries = tenant_storage
            .scan_prefix_at(col_data_prefix.as_bytes(), tx.inner())
            .await?;
        let mut deleted_keys: Vec<Vec<u8>> = Vec::new();
        for (k, _) in col_entries {
            deleted_keys.push(k);
        }

        let txt_entries = tenant_storage
            .scan_prefix_at(txt_data_prefix.as_bytes(), tx.inner())
            .await?;
        for (k, _) in txt_entries {
            deleted_keys.push(k);
        }

        deleted_keys.push(col_idx_key.clone());

        tenant_storage
            .delete_prefix(tx, col_data_prefix.as_bytes())
            .await?;

        tenant_storage
            .delete_prefix(tx, txt_data_prefix.as_bytes())
            .await?;

        tenant_storage.delete(tx, &col_idx_key).await?;

        tenant_storage.commit(tx).await?;

        let remaining_col_data = tenant_storage
            .scan_prefix(col_data_prefix.as_bytes())
            .await?;
        let remaining_txt_data = tenant_storage
            .scan_prefix(txt_data_prefix.as_bytes())
            .await?;

        let total_remaining = remaining_col_data.len() + remaining_txt_data.len();
        if total_remaining != 0 {
            return Err(contextra_types::ContextraError::Internal(format!(
                "CRITICAL: Collection '{name}' drop incomplete: {total_remaining} keys remained after delete"
            )));
        }

        let mut hasher = blake3::Hasher::new();
        hasher.update(name.as_bytes());
        let hash_bytes = hasher.finalize();
        let slice8: [u8; 8] = hash_bytes.as_bytes()[0..8].try_into().map_err(|_| {
            contextra_types::ContextraError::Internal(
                "Failed to extract 8 bytes for collection ID".into(),
            )
        })?;
        let col_id_u64 = u64::from_le_bytes(slice8);
        let collection_id = CollectionId::try_new(col_id_u64)?;

        let scope = DeletionScope::Collection {
            collection_id,
            tenant_id,
        };

        let mut sorted_deleted_keys = deleted_keys.clone();
        sorted_deleted_keys.sort();
        let deleted_keys_hash =
            contextra_crypto::deletion_proof::types::hash_deleted_keys_length_prefixed(
                &sorted_deleted_keys,
            );

        let proof_stub = DeletionProof {
            signature_version: 4,
            scope,
            deleted_keys_hash,
            deleted_after_tx: tx,
            timestamp: 0,
            signature: Vec::new(),
            covered_layers: vec![DeletionLayer::LsmMemtable],
            uncovered_layers: Vec::new(),
            excluded_scopes: vec![],
            graph_repair: Vec::new(),
            wal_chain_receipt: None,
            audit_chain_position: None,
            integrity_warning: None,
        };

        let mut payload = Vec::with_capacity(128);
        payload.extend_from_slice(b"contextra/deletion-proof/v4\0");
        let scope_bytes = bincode::serialize(&proof_stub.scope)
            .map_err(|e| contextra_types::ContextraError::Internal(e.to_string()))?;
        let tx_bytes = proof_stub.deleted_after_tx.0.to_le_bytes();
        let timestamp_bytes = proof_stub.timestamp.to_le_bytes();
        let covered_layers_bytes = bincode::serialize(&proof_stub.covered_layers)
            .map_err(|e| contextra_types::ContextraError::Internal(e.to_string()))?;
        let excluded_scopes_bytes = bincode::serialize(&proof_stub.excluded_scopes)
            .map_err(|e| contextra_types::ContextraError::Internal(e.to_string()))?;
        let graph_repair_bytes = bincode::serialize(&proof_stub.graph_repair)
            .map_err(|e| contextra_types::ContextraError::Internal(e.to_string()))?;

        payload.extend_from_slice(&scope_bytes);
        payload.extend_from_slice(&proof_stub.deleted_keys_hash);
        payload.extend_from_slice(&tx_bytes);
        payload.extend_from_slice(&timestamp_bytes);
        payload.extend_from_slice(&covered_layers_bytes);
        payload.extend_from_slice(&excluded_scopes_bytes);
        payload.extend_from_slice(&graph_repair_bytes);

        let key_bytes: [u8; 32] = proof_key[..32].try_into().map_err(|_| {
            contextra_types::ContextraError::invalid_input("Proof key must be at least 32 bytes")
        })?;
        let signer =
            contextra_crypto::deletion_proof::signer::ProofKeyPairSigner::from_secret_bytes(
                key_bytes,
            );
        use contextra_crypto::deletion_proof::signer::ProofSigner;
        let sig = signer
            .sign(&payload)
            .map_err(|e| contextra_types::ContextraError::Internal(e.to_string()))?;

        let proof = DeletionProof {
            signature: sig.to_vec(),
            ..proof_stub
        };

        // Execute DpiPurgeDriver and reverse_sweep for dropped collection (§C5.3, C1.4)
        let footprint = deletion::ErasureFootprint::for_collection(tenant_id, name);
        let registry = deletion::KeyFamilyRegistry::default();
        let sweep_needles = deletion::FprNeedles {
            doc_id_bytes: Vec::new(),
            collection_id: Some(collection_id),
            collection_name: name.to_string(),
            tenant_prefix: tenant_id.inner().to_le_bytes().to_vec(),
        };

        let mut pid16 = [0u8; 16];
        pid16.copy_from_slice(&proof.deleted_keys_hash[..16]);

        let fon_handle = deletion::FonLogicalHandle {
            proof_id: pid16,
            proof_path: std::path::PathBuf::from(format!(
                "proofs/{}.json",
                hex_encode_32_col(&proof.deleted_keys_hash)
            )),
            full_proof_id: proof.deleted_keys_hash,
        };

        let forget_opts = deletion::ForgetOptions::default();

        let tenant_prefix = format!("t:{}:", tenant_id.inner());
        let purge_scope = contextra_store::compaction::purge_filter::PurgeScope::Prefixes(vec![
            format!("{}{}", tenant_prefix, col_data_prefix).into_bytes(),
            format!("{}{}", tenant_prefix, txt_data_prefix).into_bytes(),
        ]);

        let db_dir = self.storage.db_dir().to_path_buf();
        let db_instance_id = if let Some(ref handle) = self.storage.erasure_ledger_handle {
            handle.db_instance_id()
        } else {
            let mut id = [0xFFu8; 16];
            let hash = blake3::hash(db_dir.to_string_lossy().as_bytes());
            id.copy_from_slice(&hash.as_bytes()[..16]);
            id
        };

        let signer_key_path = db_dir.join("proof_ed25519.key");
        let deps = deletion::DeletionWiringDeps {
            storage: self.storage.clone(),
            db_dir,
            signer_cfg_path: Some(signer_key_path),
            db_instance_id,
            ring_mode: deletion::wiring::RingMode::Sovereign,
        };

        let physical_purge = match deletion::DeletionWiring::build(deps) {
            Ok(deletion_wiring) => {
                let driver = deletion_wiring.purge_driver();
                let zeu_probe = self.storage.store_probe();
                let probes: &[&dyn contextra_crypto::deletion_proof::probe::WitStoreProbe] =
                    &[zeu_probe.as_ref()];

                match driver
                    .run_purge(
                        purge_scope,
                        &[name.to_string()],
                        tx,
                        &fon_handle,
                        &forget_opts,
                        &deleted_keys,
                        &sweep_needles,
                        &footprint,
                        &registry,
                        deletion::SweepDepth::Standard,
                        None,
                        probes,
                        None,
                    )
                    .await
                {
                    Ok((outcome, _sweep_report)) => match outcome.physical {
                        Some(deletion::PurgeHandle::Final(_phys_handle)) => {
                            PhysicalPurgeStatus::Performed {
                                records_purged: u64::try_from(deleted_keys.len()).unwrap_or(0),
                                bytes_reclaimed: 0,
                            }
                        }
                        Some(deletion::PurgeHandle::Blocked { reason, .. }) => {
                            PhysicalPurgeStatus::NotPerformed {
                                reason: format!("Physical purge blocked: {reason}"),
                            }
                        }
                        Some(deletion::PurgeHandle::Pending { .. }) => {
                            PhysicalPurgeStatus::NotPerformed {
                                reason: "Physical purge pending execution".to_string(),
                            }
                        }
                        None => PhysicalPurgeStatus::NotPerformed {
                            reason: "Physical purge skipped or unexecuted".to_string(),
                        },
                    },
                    Err(e) => PhysicalPurgeStatus::NotPerformed {
                        reason: format!("Physical purge execution error: {e}"),
                    },
                }
            }
            Err(e) => PhysicalPurgeStatus::NotPerformed {
                reason: format!("Deletion wiring build error: {e}"),
            },
        };

        Ok(DropCollectionReport {
            proof,
            physical_purge,
        })
    }

    /// Drops a collection and issues a [`DeletionProof`] attesting to logical LSM key space deletion.
    ///
    /// # Limitations / Non-Guarantees
    /// - Claims ONLY layers verified by an explicit scan (strictly [`DeletionLayer::LsmMemtable`] unless physical purge succeeded).
    /// - Does NOT guarantee hardware physical SSD flash controller level sanitization (wear leveling / TRIM depend on OS & hardware).
    /// - Attests strictly to logical deletion (tombstones committed and verified empty in active MemTable/LSM key space scans).
    #[tracing::instrument(level = "trace", skip(self, proof_key))]
    pub async fn drop_collection(
        &self,
        name: &str,
        tenant_id: TenantId,
        proof_key: &[u8],
    ) -> Result<DeletionProof> {
        let report = self
            .drop_collection_with_report(name, tenant_id, proof_key)
            .await?;
        Ok(report.proof)
    }
}
