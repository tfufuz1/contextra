use crate::collection::Collection;
use crate::deletion::forget_options::ForgetOptions;
use crate::deletion::forget_options::VerifyDepth;
use contextra_crypto::deletion_witness::LayerWitness;
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
    /// Deletes a document from the collection by its ID.
    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn delete(&self, id: &str) -> Result<()> {
        // Retain per-document lock across begin_transaction, delete_op, and commit/rollback (TOCTOU safety)
        let _guard = self.kv_locks.lock_for(id).await;
        let mut db_tx = self.begin_transaction()?;

        match self.delete_op(&mut db_tx, id).await {
            Ok(_) => {
                db_tx.commit().await?;
                self.check_and_trigger_community_detection(1);
                Ok(())
            }
            Err(e) => {
                if let Err(rollback_err) = db_tx.rollback().await {
                    tracing::error!("[INV-DB-3] Failed to rollback delete: {}", rollback_err);
                }
                Err(e)
            }
        }
    }

    /// Logical document erasure and tombstone committal for a single document (Guarantee Level C1: logical, single document only).
    pub async fn erase_document(&self, id: &str, opts: &DocEraseOptions) -> Result<DocErasure> {
        if id.is_empty() {
            return Err(ContextraError::InvalidInput(
                "Document ID cannot be empty".to_string(),
            ));
        }

        opts.options.validate()?;

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

        let _key_guard = self.kv_locks.lock_for(id).await;

        let is_off_mode = opts.options.assurance == contextra_crypto::deletion_proof::claim_text::WitAssurance::S0
            && opts.options.purge == crate::deletion::forget_options::PurgeMode::None;

        let maybe_backend = self.storage.erasure_capabilities();

        if maybe_backend.is_none() && !is_off_mode {
            return Err(ContextraError::capability_unsupported(
                "erasure",
                "Erasure backend not available",
            ));
        }

        let user_key = self.namespaced_key(id.as_bytes(), 0);
        let doc_key = self.namespaced_key(&doc_id.inner().to_le_bytes(), 1);

        let (footprint_hash, scope_hash, prefixes, explicit_keys, observed_empty, _scope_prefixes_ref) = if is_off_mode {
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

        let scope_write_seq_before = 0u64;

        let db_tx = self.begin_transaction()?;
        let erased_at_tx = db_tx.tx_id;

        let old_user_val = self
            .storage
            .get_at_seq_tracked(erased_at_tx, &user_key, erased_at_tx.inner())
            .await?;
        let old_doc_val = self
            .storage
            .get_at_seq_tracked(erased_at_tx, &doc_key, erased_at_tx.inner())
            .await?;

        db_tx.stage_text_delete(doc_id);

        if let Ok(eid) = EntityId::from_key(id) {
            db_tx.stage_graph_entity_delete(eid);
        }

        for key in &explicit_keys {
            self.storage.delete(erased_at_tx, key).await?;
        }

        db_tx.record_keys_with_old_values(user_key, old_user_val, doc_key, old_doc_val, doc_id);

        db_tx.commit().await?;

        if let Err(e) = self.index.delete(erased_at_tx, doc_id).await {
            tracing::error!("Post-commit vector index delete failed for doc {}: {:?}", id, e);
        }

        self.check_and_trigger_community_detection(1);

        let witnesses = Vec::new();
        let final_scope_write_seq = scope_write_seq_before;

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

        if let Ok(eid) = EntityId::from_key(id) {
            db_tx.stage_graph_entity_delete(eid);
        }

        self.storage.delete(tx, &user_key).await?;
        self.storage.delete(tx, &doc_key).await?;

        db_tx.record_keys_with_old_values(user_key, old_user_val, doc_key, old_doc_val, doc_id);

        self.index.delete(tx, doc_id).await?;

        Ok(())
    }
}
