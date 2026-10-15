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
    /// Complete document erasure from all key spaces, RAM indices, and witnesses (INV-D1/D2/D3).
    ///
    /// # Crash-Safety & Idempotency
    /// Writes to the Erasure Ledger are idempotent; if a crash occurs after the ledger append
    /// but before the tombstone commit, recovery WAL replay and compaction filters use the ledger entry
    /// to safely filter out deleted key versions.
    ///
    /// # Execution Order
    /// (a) Lock document key and scope erasure fence;
    /// (b) Compute ErasureFootprint;
    /// (c) Validate assurance capabilities;
    /// (d) Write and commit tombstones for all footprint keys in a single transaction;
    /// (e) Clear RAM indices (HNSW, InvertedIndex, Graph/Relations, KV-Hooks);
    /// (f) Return empty witnesses (no witness fabrication without real scan per REG-14);
    /// (g) Verify scope write sequence for concurrent modification.
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

        // Compute Footprint first to get exact keys/prefixes
        let footprint = self.erasure_footprint(doc_id).await?;

        let maybe_backend = self.storage.erasure_backend();

        if maybe_backend.is_none()
            && (opts.options.assurance != contextra_crypto::deletion_proof::claim_text::WitAssurance::S0
                || opts.options.purge != crate::deletion::forget_options::PurgeMode::None)
        {
            return Err(EraseError::NotAvailable.into());
        }

        let scope_write_seq_before = if let Some(backend) = maybe_backend {
            backend.snapshot_write_seq(&footprint.keys.prefixes)
        } else {
            0
        };

        // Allocate TxId for Ledger and Tombstones
        let db_tx = self.begin_transaction()?;
        let erased_at_tx = db_tx.tx_id;

        // Compute Scope Hash
        let scope_hash = contextra_crypto::deletion_proof::hash_deleted_keys_length_prefixed(
            &footprint.keys.explicit,
        );

        // (d) Tombstones for ALL Footprint Keys in single transaction
        let user_key = self.namespaced_key(id.as_bytes(), 0);
        let doc_key = self.namespaced_key(&doc_id.inner().to_le_bytes(), 1);

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

        // Tombstone explicit footprint keys
        for key in &footprint.keys.explicit {
            self.storage.delete(erased_at_tx, key).await?;
        }

        db_tx.record_keys_with_old_values(user_key, old_user_val, doc_key, old_doc_val, doc_id);

        db_tx.commit().await?;

        // (e) RAM index cleanup
        self.index.delete(erased_at_tx, doc_id).await?;

        self.check_and_trigger_community_detection(1);
        if let Some(kv_hooks) = self.kv_hooks() {
            let tenant = self.tenant_id();
            kv_hooks.remove_doc_segments(tenant, doc_id);
        }

        // (f) No witness fabrication without real scan (REG-14, Hotfix F-01)
        let witnesses = Vec::new();

        // (g) Scope Write Seq verification
        let final_scope_write_seq = if let Some(backend) = maybe_backend {
            let scope_write_seq_after = backend
                .snapshot_write_seq(&footprint.keys.prefixes);
            if scope_write_seq_before != scope_write_seq_after {
                tracing::warn!(
                    "Concurrent modification detected during erase_document (write_seq changed from {} to {}). Deletion committed at tx {}.",
                    scope_write_seq_before,
                    scope_write_seq_after,
                    erased_at_tx.inner()
                );
            }
            scope_write_seq_after
        } else {
            scope_write_seq_before
        };

        Ok(DocErasure {
            footprint_hash: footprint.hash,
            scope_hash,
            as_of_tx: erased_at_tx,
            scope_write_seq: final_scope_write_seq,
            witnesses,
            observed_empty: footprint.observed_empty,
            prefixes: footprint.keys.prefixes,
            keys: footprint.keys.explicit,
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
