use crate::collection::Collection;
use crate::deletion::forget_options::ForgetOptions;
use crate::deletion::forget_options::VerifyDepth;
use contextra_crypto::deletion_witness::LayerWitness;
use contextra_ports::erasure::EraseError;
use contextra_ports::{StorageEngine, VectorIndex};
use contextra_types::{ContextraError, DocId, EntityId, Result, TxId};
use serde::Serialize;

/// Execution options for document erasure (§C9).
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct DocEraseOptions {
    /// Generic forget options controlling assurance, purge mode, and wait policy.
    pub options: ForgetOptions,
}

/// Result of a complete document erasure operation.
#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct DocErasure {
    /// Hash over the ErasureFootprint.
    pub footprint_hash: [u8; 32],
    /// Hash over the scope keys and prefixes.
    pub scope_hash: [u8; 32],
    /// Commit transaction ID of the erasure operation.
    pub as_of_tx: TxId,
    /// Erasure fence scope write sequence number.
    pub scope_write_seq: u64,
    /// Layer witnesses confirming removal across storage layers and RAM indices.
    #[serde(skip)]
    pub witnesses: Vec<LayerWitness>,
    /// Whether the document was observed to be empty before deletion.
    pub observed_empty: bool,
    /// List of key prefixes covered by the erasure.
    pub prefixes: Vec<Vec<u8>>,
    /// List of explicit keys covered by the erasure.
    pub keys: Vec<Vec<u8>>,
}

impl<S: StorageEngine, V: VectorIndex> Collection<S, V> {
    /// Logical document erasure and tombstone committal for a single document (Guarantee Level C1: logical, single document only).
    ///
    /// # Guarantee Level & Truth (Doc-Truth, S12)
    /// Provides Level C1 logical deletion by committing tombstones for document keys and clearing associated RAM indices.
    /// Physical purge, erasure ledger logging, and real storage witness scans are not performed at this stage.
    ///
    /// # Execution Order
    /// (a) Validate input document ID and erase options;
    /// (b) Lock document key;
    /// (c) Verify scope write sequence for concurrent modification before tombstone commit;
    /// (d) Write and commit tombstones in a single transaction;
    /// (e) Clear RAM indices (HNSW, InvertedIndex, KV-Hooks if configured);
    /// (f) Return `DocErasure` result with empty witnesses (witnesses are issued only when real scans are active).
    pub async fn erase_document(&self, id: &str, opts: &DocEraseOptions) -> Result<DocErasure> {
        if id.is_empty() {
            return Err(ContextraError::InvalidInput(
                "Document ID cannot be empty".to_string(),
            ));
        }

        // Validate execution options (§C9)
        opts.options.validate()?;

        // Assurance above S0 requires real storage scans which are implemented in M5.
        // Return CapabilityUnsupported per REG-14 until real scans are wired.
        if opts.options.assurance
            > contextra_crypto::deletion_proof::claim_text::WitAssurance::S0
        {
            return Err(ContextraError::capability_unsupported(
                "real_scan_erasure",
                "Assurance levels above S0 require real storage scans",
            ));
        }

        if opts.options.verify_depth == VerifyDepth::FullSweep {
            return Err(ContextraError::capability_unsupported(
                "full_sweep_verify_depth",
                "FullSweep verify depth requires real storage scans and erasure ledger",
            ));
        }

        let doc_id = DocId::from_key(id)?;

        // (a) Key lock
        let _key_guard = self.kv_locks.lock_for(id).await;

        let is_off_mode = opts.options.assurance == contextra_crypto::deletion_proof::claim_text::WitAssurance::S0
            && opts.options.purge == crate::deletion::forget_options::PurgeMode::None;

        let maybe_backend = self.storage.erasure_backend();

        if maybe_backend.is_none() && !is_off_mode {
            return Err(EraseError::NotAvailable.into());
        }

        let user_key = self.namespaced_key(id.as_bytes(), 0);
        let doc_key = self.namespaced_key(&doc_id.inner().to_le_bytes(), 1);

        let (footprint_hash, scope_hash, prefixes, explicit_keys, observed_empty, scope_prefixes_ref) = if is_off_mode {
            // Mode Off: Direct deletion without footprint computation / storage scan / heap allocations.
            let explicit = vec![user_key.clone(), doc_key.clone()];
            let scope_hash = contextra_crypto::deletion_proof::hash_deleted_keys_length_prefixed(&explicit);
            let prefixes = Vec::<Vec<u8>>::new();
            ([0u8; 32], scope_hash, prefixes, explicit, false, Vec::<Vec<u8>>::new())
        } else {
            let footprint = self.erasure_footprint(doc_id).await?;
            let scope_hash = contextra_crypto::deletion_proof::hash_deleted_keys_length_prefixed(&footprint.keys.explicit);
            (
                footprint.hash,
                scope_hash,
                footprint.keys.prefixes.clone(),
                footprint.keys.explicit,
                footprint.observed_empty,
                footprint.keys.prefixes,
            )
        };

        let scope_write_seq_before = if let Some(backend) = maybe_backend.as_ref() {
            backend.snapshot_write_seq(&scope_prefixes_ref)
        } else {
            0
        };

        // Allocate TxId for Ledger and Tombstones
        let db_tx = self.begin_transaction()?;
        let erased_at_tx = db_tx.tx_id;

        // Check concurrent modification before commit
        if let Some(backend) = maybe_backend.as_ref() {
            let scope_write_seq_check = backend.snapshot_write_seq(&scope_prefixes_ref);
            if scope_write_seq_before != scope_write_seq_check {
                return Err(ContextraError::Conflict(
                    "ConcurrentModification detected before tombstone commit".to_string(),
                ));
            }
        }

        // (d) Tombstones for explicit keys in single transaction
        let old_user_val = self
            .storage
            .get_at_seq_tracked(erased_at_tx, &user_key, erased_at_tx.inner())
            .await?;
        let old_doc_val = self
            .storage
            .get_at_seq_tracked(erased_at_tx, &doc_key, erased_at_tx.inner())
            .await?;

        db_tx.stage_text_delete(doc_id);

        match EntityId::from_key(id) {
            Ok(eid) => db_tx.stage_graph_entity_delete(eid),
            Err(_) => {}
        }

        // Direct storage.delete calls
        for key in &explicit_keys {
            self.storage.delete(erased_at_tx, key).await?;
        }

        db_tx.record_keys_with_old_values(user_key, old_user_val, doc_key, old_doc_val, doc_id);

        db_tx.commit().await?;

        // After commit succeeds, post-commit operations must not fail the deletion call.
        if let Err(e) = self.index.delete(erased_at_tx, doc_id).await {
            tracing::error!("Post-commit vector index delete failed for doc {}: {:?}", id, e);
        }

        self.check_and_trigger_community_detection(1);
        if let Some(kv_hooks) = self.kv_hooks() {
            let tenant = self.tenant_id();
            kv_hooks.remove_doc_segments(tenant, doc_id);
        }

        // (f) No witness fabrication without real scan (REG-14, Hotfix F-01)
        let witnesses = Vec::new();

        let final_scope_write_seq = if let Some(backend) = maybe_backend.as_ref() {
            backend.snapshot_write_seq(&scope_prefixes_ref)
        } else {
            scope_write_seq_before
        };

        Ok(DocErasure {
            footprint_hash,
            scope_hash,
            as_of_tx: erased_at_tx,
            scope_write_seq: final_scope_write_seq,
            witnesses,
            observed_empty,
            prefixes,
            keys: explicit_keys,
        })
    }

    /// Deletes a document from the collection by its ID (delegating to `erase_document`).
    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn delete(&self, id: &str) -> Result<()> {
        let opts = DocEraseOptions {
            options: ForgetOptions {
                assurance: contextra_crypto::deletion_proof::claim_text::WitAssurance::S0,
                ..ForgetOptions::default()
            },
        };
        self.erase_document(id, &opts).await.map(|_| ())
    }

    pub async fn delete_op(
        &self,
        db_tx: &mut crate::transaction::DbTransaction<S, V>,
        id: &str,
    ) -> Result<()> {
        let doc_id = DocId::from_key(id)?;
        let doc_key = self.namespaced_key(&doc_id.inner().to_le_bytes(), 1);
        let user_key = self.namespaced_key(id.as_bytes(), 0);

        let tx = db_tx.tx_id;

        let old_user_val = self
            .storage
            .get_at_seq_tracked(tx, &user_key, tx.inner())
            .await?;
        let old_doc_val = self
            .storage
            .get_at_seq_tracked(tx, &doc_key, tx.inner())
            .await?;

        db_tx.stage_text_delete(doc_id);

        match EntityId::from_key(id) {
            Ok(eid) => db_tx.stage_graph_entity_delete(eid),
            Err(_) => {}
        }

        self.storage.delete(tx, &user_key).await?;
        self.storage.delete(tx, &doc_key).await?;

        db_tx.record_keys_with_old_values(user_key, old_user_val, doc_key, old_doc_val, doc_id);

        self.index.delete(tx, doc_id).await?;

        Ok(())
    }
}
