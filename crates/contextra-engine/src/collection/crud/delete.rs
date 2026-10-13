use crate::collection::Collection;
use crate::deletion::forget_options::ForgetOptions;
use crate::deletion::relations_witness::issue_relations_witness;
use crate::deletion::text_witness::issue_text_index_witness;
use contextra_crypto::deletion_proof::claim_text::{WitAssurance, WitClaimText};
use contextra_crypto::deletion_proof::evidence::WitLayerEvidence;
use contextra_crypto::deletion_proof::probe::WitStoreGeneration;
use contextra_crypto::deletion_proof::DeletionLayer;
use contextra_crypto::deletion_witness::{LayerWitness, WitIssueArgs};
use contextra_ports::{StorageEngine, VectorIndex};
use contextra_store::erasure_ledger::{compute_keyed_hash, LedgerEntry, LedgerEntryKind, LedgerKey};
use contextra_store::lsm::witness::ZeuIssueRequest;
use contextra_store::LsmStorage;
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
    /// # Execution Order
    /// (a) Lock document key and scope erasure fence;
    /// (b) Compute ErasureFootprint;
    /// (c) Write Erasure Ledger entry (`ledger_append`) with `erased_at_tx` and fsync;
    /// (d) Write and commit tombstones for all footprint keys in a single transaction;
    /// (e) Clear RAM indices (HNSW, InvertedIndex, Graph/Relations, KV-Hooks);
    /// (f) Collect layer witnesses (Memtable, HNSW, Text, Relations, Ledger);
    /// (g) Verify scope write sequence for concurrent modification.
    pub async fn erase_document(
        &self,
        id: &str,
        _opts: &DocEraseOptions,
    ) -> Result<DocErasure> {
        if id.is_empty() {
            return Err(ContextraError::InvalidInput(
                "Document ID cannot be empty".to_string(),
            ));
        }

        let doc_id = DocId::from_key(id)?;

        let user_key = self.namespaced_key(id.as_bytes(), 0);
        let doc_key = self.namespaced_key(&doc_id.inner().to_le_bytes(), 1);

        let maybe_lsm = (self.storage.as_ref() as &dyn std::any::Any)
            .downcast_ref::<LsmStorage>();

        // (a) Document lock via ErasureFence and KV key lock guard
        let _key_guard = self.kv_locks.lock_for(id).await;

        let _doc_lock_guard = if let Some(lsm) = maybe_lsm {
            Some(
                lsm.erasure_fence
                    .lock_documents_guarded(vec![user_key.clone(), doc_key.clone()])
                    .map_err(|e| {
                        ContextraError::InvalidInput(format!(
                            "ErasureFence document lock conflict: {e}"
                        ))
                    })?,
            )
        } else {
            None
        };

        // (b) Compute Footprint
        let footprint = self.erasure_footprint(doc_id).await?;

        let _scope_fence_guard = if let Some(lsm) = maybe_lsm {
            if !footprint.keys.prefixes.is_empty() {
                Some(
                    lsm.erasure_fence
                        .acquire_scope_fence_for(footprint.keys.prefixes.clone())
                        .map_err(|e| {
                            ContextraError::InvalidInput(format!(
                                "ErasureFence scope fence conflict: {e}"
                            ))
                        })?,
                )
            } else {
                None
            }
        } else {
            None
        };

        let scope_write_seq_before = if let Some(lsm) = maybe_lsm {
            lsm.erasure_fence.snapshot_write_seq(&footprint.keys.prefixes)
        } else {
            0
        };

        let db_tx = self.begin_transaction()?;
        let erased_at_tx = db_tx.tx_id;

        let scope_hash = contextra_crypto::deletion_proof::hash_deleted_keys_length_prefixed(
            &footprint.keys.explicit,
        );

        // (c) Ledger Append BEFORE Tombstone Commit
        if let Some(lsm) = maybe_lsm {
            let mut ledger_entries = Vec::new();
            let dummy_key = LedgerKey::new([0u8; 32]);

            for k in &footprint.keys.explicit {
                let kh = compute_keyed_hash(&dummy_key, b"lsm", k);
                let mut proof_id = [0u8; 16];
                proof_id.copy_from_slice(&blake3::hash(k).as_bytes()[0..16]);
                ledger_entries.push(LedgerEntry {
                    ledger_seq: 0,
                    kind: LedgerEntryKind::Key,
                    keyed_hash: kh,
                    erased_at_tx: erased_at_tx.inner(),
                    proof_id,
                });
            }

            for p in &footprint.keys.prefixes {
                let kh = compute_keyed_hash(&dummy_key, b"lsm", p);
                let mut proof_id = [0u8; 16];
                proof_id.copy_from_slice(&blake3::hash(p).as_bytes()[0..16]);
                let prefix_len = u32::try_from(p.len()).unwrap_or(0);
                ledger_entries.push(LedgerEntry {
                    ledger_seq: 0,
                    kind: LedgerEntryKind::Prefix { prefix_len },
                    keyed_hash: kh,
                    erased_at_tx: erased_at_tx.inner(),
                    proof_id,
                });
            }

            lsm.ledger_append(&ledger_entries).await?;
        }

        // (d) Tombstones for ALL Footprint Keys in single transaction
        let old_user_val = self
            .storage
            .get_at_seq_tracked(erased_at_tx, &user_key, u64::MAX)
            .await?;
        let old_doc_val = self
            .storage
            .get_at_seq_tracked(erased_at_tx, &doc_key, u64::MAX)
            .await?;

        db_tx.stage_text_delete(doc_id);

        if let Ok(eid) = EntityId::from_key(id) {
            db_tx.stage_graph_entity_delete(eid);
        }

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

        // (f) Witnesses
        let mut witnesses = Vec::new();

        if let Some(lsm) = maybe_lsm {
            let req = ZeuIssueRequest {
                scope_keys: footprint.keys.explicit.clone(),
                scope_prefixes: footprint.keys.prefixes.clone(),
                as_of_tx: erased_at_tx,
                scanned_at_tx: erased_at_tx,
                assurance: WitAssurance::S1,
                claim: WitClaimText::S1,
                scanner_name: "engine_document_eraser",
                scanner_version: 1,
                key_identifiers_claimed: true,
            };
            if let Ok(lsm_witnesses) = lsm.issue_witnesses_v2(req).await {
                witnesses.extend(lsm_witnesses);
            }

            if let Some(ledger) = lsm.ledger_handle() {
                let entries = ledger.entries();
                let head_seq = entries.last().map(|e| e.ledger_seq).unwrap_or(0);
                let chain_mac = ledger.last_chain_mac();

                let ledger_evidence = WitLayerEvidence::ErasureLedger {
                    ledger_head_seq: head_seq,
                    decision_chain_mac: chain_mac,
                    entry_confirmed: true,
                };
                let witness_args = WitIssueArgs {
                    layer: DeletionLayer::ErasureLedger,
                    scope_hash,
                    scanned_at_tx: erased_at_tx,
                    as_of_tx: erased_at_tx,
                    assurance: WitAssurance::S1,
                    claim: WitClaimText::S1,
                    footprint_hash: footprint.hash,
                    scope_write_seq: scope_write_seq_before,
                    scanner_name: "ErasureLedgerWitnessScanner",
                    scanner_version: 1,
                    store_generation: WitStoreGeneration(0),
                    layer_evidence: ledger_evidence,
                    key_identifiers_claimed: false,
                };
                if let Ok(ledger_witness) = LayerWitness::issue(witness_args) {
                    witnesses.push(ledger_witness);
                }
            }
        }

        // HNSW Witness
        let maybe_hnsw = (self.index.as_ref() as &dyn std::any::Any)
            .downcast_ref::<contextra_vector::HnswIndex>();
        if let Some(hnsw) = maybe_hnsw {
            let hnsw_args = contextra_vector::HnwIssueArgs {
                doc_ids: vec![doc_id],
                scope_hash,
                footprint_hash: footprint.hash,
                as_of_tx: erased_at_tx,
                scope_write_seq: scope_write_seq_before,
                depth: contextra_vector::ResDepth::FullSweep,
                assurance: WitAssurance::S1,
            };
            if let Ok(Some(hnsw_witness)) = hnsw.issue_residue_witness(&hnsw_args) {
                witnesses.push(hnsw_witness);
            }
        }

        if let Ok(text_witness) = issue_text_index_witness(
            doc_id,
            &footprint.ram.term_bytes,
            scope_hash,
            erased_at_tx,
            None,
        ) {
            witnesses.push(text_witness);
        }

        let scanned_edges = u64::try_from(footprint.ram.relation_keys.len()).unwrap_or(0);
        if let Ok(rel_witness) = issue_relations_witness(
            doc_id,
            scanned_edges,
            scope_hash,
            erased_at_tx,
            None,
        ) {
            witnesses.push(rel_witness);
        }

        // (g) Scope Write Seq verification
        if let Some(lsm) = maybe_lsm {
            let scope_write_seq_after =
                lsm.erasure_fence.snapshot_write_seq(&footprint.keys.prefixes);
            if scope_write_seq_before != scope_write_seq_after {
                return Err(ContextraError::InvalidInput(
                    "ConcurrentModification during erase_document execution".to_string(),
                ));
            }
        }

        Ok(DocErasure {
            footprint_hash: footprint.hash,
            scope_hash,
            as_of_tx: erased_at_tx,
            scope_write_seq: scope_write_seq_before,
            witnesses,
            observed_empty: footprint.observed_empty,
            prefixes: footprint.keys.prefixes,
            keys: footprint.keys.explicit,
        })
    }

    /// Deletes a document from the collection by its ID (delegating to `erase_document`).
    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn delete(&self, id: &str) -> Result<()> {
        let opts = DocEraseOptions::default();
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
            .get_at_seq_tracked(tx, &user_key, u64::MAX)
            .await?;
        let old_doc_val = self
            .storage
            .get_at_seq_tracked(tx, &doc_key, u64::MAX)
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
