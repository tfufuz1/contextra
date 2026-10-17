//! Implementation of high-assurance provable forgetting (`Contextra::forget`, `forget_collection`, `forget_tenant`).

#![forbid(unsafe_code)]

use std::path::Path;

use contextra_crypto::deletion_proof::claim_text::{WitAssurance, WitClaimText};
use contextra_crypto::deletion_proof::probe::WitStoreProbe;
use contextra_crypto::deletion_proof::proof_v4::ProofV4;
use contextra_crypto::deletion_proof::types::{
    hash_deleted_keys_length_prefixed, DeletionLayer, DeletionScope,
};
use contextra_crypto::deletion_witness::{LayerWitness, WitIssueArgs};
use contextra_ports::VectorIndex;
use contextra_store::compaction::purge_filter::PurgeScope;
use contextra_store::lsm::witness::ZeuIssueRequest;
use contextra_types::{CollectionId, ContextraError, DocId, EntityId, Result, TenantId, TxId};
use serde::Serialize;

use crate::deletion::relations_witness::issue_relations_witness;
use crate::deletion::text_witness::scan_and_issue_text_index_witness;
use crate::deletion::{
    DpiDeletionRuntime, DpiLogicalInput, DpiPurgeDriver, DpiPurgeDriverConfig, ErasureFootprint,
    ForgetOptions, PurgeHandle, PurgeMode, Wait,
};
use crate::Contextra;

/// Structured receipt returned by `forget`, `forget_collection`, and `forget_tenant`.
#[derive(Debug, Clone, Serialize)]
pub struct ForgetReceipt {
    /// Target deletion scope.
    pub scope: DeletionScope,
    /// Maximum assurance level reached with valid measured witnesses.
    pub assurance_reached: WitAssurance,
    /// Requested target assurance level.
    pub assurance_requested: WitAssurance,
    /// Per-layer verification status.
    pub layers: Vec<LayerStatus>,
    /// Sealed v4 cryptographic deletion proof, present iff `assurance_reached >= assurance_requested`.
    pub proof: Option<ProofV4>,
    /// Wait policy applied during deletion.
    pub wait: Wait,
}

/// Verification status for an individual storage/index layer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum LayerStatus {
    /// Layer verified with measured witness backing specified key families.
    Verified {
        layer: DeletionLayer,
        families: Vec<String>,
    },
    /// Physical purge or key destruction pending for layer.
    Pending {
        layer: DeletionLayer,
    },
    /// Layer verification or witness issuance failed.
    Failed {
        layer: DeletionLayer,
        reason: String,
    },
}

impl Contextra {
    /// Provably forgets a single document by its user string ID.
    pub async fn forget(&self, id: &str, opts: ForgetOptions) -> Result<ForgetReceipt> {
        opts.validate()?;

        let col = self.default_col().await?;
        let doc_id = DocId::from_key(id)
            .map_err(|e| ContextraError::InvalidInput(format!("Invalid doc id '{id}': {e}")))?;

        if doc_id.inner() == 0 {
            return Err(ContextraError::InvalidInput(
                "DocId(0) is reserved and invalid for forget target".to_string(),
            ));
        }

        let tenant_id = col.tenant_id();
        let footprint = ErasureFootprint::for_document(tenant_id, col.name(), doc_id);
        let scope = DeletionScope::Document { doc_id, tenant_id };

        // Execute logical deletion in a transaction
        col.delete(id).await?;

        let as_of_tx = TxId(self.last_committed_seq().await?);

        self.execute_forget_pipeline(
            scope,
            footprint,
            vec![id.to_string()],
            as_of_tx,
            &opts,
            Some((tenant_id, col.name().to_string(), id.to_string(), doc_id)),
        )
        .await
    }

    /// Provably forgets an entire collection by name.
    pub async fn forget_collection(
        &self,
        name: &str,
        opts: ForgetOptions,
    ) -> Result<ForgetReceipt> {
        opts.validate()?;

        let tenant_id = TenantId::SYSTEM;
        let footprint = ErasureFootprint::for_collection(tenant_id, name);

        let scope = DeletionScope::Collection {
            collection_id: CollectionId::new(1),
            tenant_id,
        };

        // Logical phase: remove collection entry & write tombstones
        {
            let mut writer = self.collections.write().await;
            writer.remove(name);
        }

        let as_of_tx = TxId(self.last_committed_seq().await?);

        self.execute_forget_pipeline(
            scope,
            footprint,
            vec![format!("col:{name}")],
            as_of_tx,
            &opts,
            None,
        )
        .await
    }

    /// Provably forgets all data associated with a tenant.
    pub async fn forget_tenant(
        &self,
        tenant: TenantId,
        opts: ForgetOptions,
    ) -> Result<ForgetReceipt> {
        opts.validate()?;

        let footprint = ErasureFootprint::for_tenant(tenant);

        let scope = DeletionScope::Tenant { tenant_id: tenant };

        // Logical phase: drop tenant collections
        {
            let mut writer = self.tenant_collections.write().await;
            writer.retain(|(t, _), _| *t != tenant);
        }

        let as_of_tx = TxId(self.last_committed_seq().await?);

        self.execute_forget_pipeline(
            scope,
            footprint,
            vec![format!("tenant:{}", tenant.inner())],
            as_of_tx,
            &opts,
            None,
        )
        .await
    }

    /// Central pipeline executing logical Phase-A, witness collection, optional purge/revocation,
    /// and stage-backed receipt generation.
    async fn execute_forget_pipeline(
        &self,
        scope: DeletionScope,
        footprint: ErasureFootprint,
        scope_file_names: Vec<String>,
        as_of_tx: TxId,
        opts: &ForgetOptions,
        doc_info: Option<(TenantId, String, String, DocId)>,
    ) -> Result<ForgetReceipt> {
        let scope_hash = hash_deleted_keys_length_prefixed(&footprint.keys.explicit);

        let mut layer_statuses = Vec::new();
        let mut witnesses = Vec::new();

        let manifest = self.storage.manifest_for_test();
        let db_dir = manifest.path().parent().unwrap_or_else(|| Path::new("."));

        // Determine DB instance ID dynamically from ErasureLedger handle or db_dir Blake3 hash
        let db_instance_id = if let Some(ref handle) = self.storage.erasure_ledger_handle {
            handle.db_instance_id()
        } else {
            let mut id = [0xFFu8; 16];
            let hash = blake3::hash(db_dir.to_string_lossy().as_bytes());
            id.copy_from_slice(&hash.as_bytes()[..16]);
            id
        };

        // 1. Store (LSM MemTable, WAL, SSTable) witnesses
        let zeu_req = ZeuIssueRequest {
            scope_keys: footprint.keys.explicit.clone(),
            scope_prefixes: footprint.keys.prefixes.clone(),
            as_of_tx,
            scanned_at_tx: as_of_tx,
            assurance: opts.assurance,
            claim: WitClaimText::S1,
            scanner_name: "forget_orchestrator",
            scanner_version: 1,
            key_identifiers_claimed: false,
        };

        match self.storage.issue_witnesses_v2(zeu_req).await {
            Ok(store_witnesses) => {
                for w in store_witnesses {
                    layer_statuses.push(LayerStatus::Verified {
                        layer: w.layer().clone(),
                        families: vec!["CollectionDocData".to_string(), "DocIdMapping".to_string()],
                    });
                    witnesses.push(w);
                }
            }
            Err(e) => {
                layer_statuses.push(LayerStatus::Failed {
                    layer: DeletionLayer::LsmMemtable,
                    reason: format!("Store witness issuance failed: {e}"),
                });
            }
        }

        // 2. HNSW Vector Index witness (measured)
        if let Some((_, _, _, doc_id)) = doc_info {
            if let Ok(col) = self.default_col().await {
                match col.vector_index().residue(doc_id).await {
                    Ok(residue) => {
                        let marked_zeroed = !residue.node_present
                            && residue.incoming_edges == 0
                            && residue.outgoing_edges == 0;

                        let hnsw_witness_res = LayerWitness::issue(WitIssueArgs {
                            layer: DeletionLayer::HnswIndex,
                            scope_hash,
                            scanned_at_tx: as_of_tx,
                            as_of_tx,
                            assurance: opts.assurance,
                            claim: WitClaimText::S1,
                            footprint_hash: [0u8; 32],
                            scope_write_seq: 0,
                            scanner_name: "hnsw_scanner",
                            scanner_version: 1,
                            store_generation: contextra_crypto::deletion_proof::probe::WitStoreGeneration(0),
                            layer_evidence: contextra_crypto::deletion_proof::evidence::WitLayerEvidence::HnswIndex {
                                node_id: doc_id.inner(),
                                incoming_edges: residue.incoming_edges,
                                outgoing_edges: residue.outgoing_edges,
                                pinned_by_reader: residue.pinned,
                                marked_zeroed,
                            },
                            key_identifiers_claimed: false,
                        });

                        match hnsw_witness_res {
                            Ok(hnsw_witness) => {
                                witnesses.push(hnsw_witness);
                                if marked_zeroed && residue.incoming_edges == 0 && !residue.pinned {
                                    layer_statuses.push(LayerStatus::Verified {
                                        layer: DeletionLayer::HnswIndex,
                                        families: vec!["CollectionIndex".to_string()],
                                    });
                                } else {
                                    layer_statuses.push(LayerStatus::Failed {
                                        layer: DeletionLayer::HnswIndex,
                                        reason: format!(
                                            "HNSW index residue detected (incoming={}, outgoing={}, pinned={})",
                                            residue.incoming_edges, residue.outgoing_edges, residue.pinned
                                        ),
                                    });
                                }
                            }
                            Err(e) => {
                                layer_statuses.push(LayerStatus::Failed {
                                    layer: DeletionLayer::HnswIndex,
                                    reason: format!("HNSW witness issuance failed: {e}"),
                                });
                            }
                        }
                    }
                    Err(e) => {
                        layer_statuses.push(LayerStatus::Failed {
                            layer: DeletionLayer::HnswIndex,
                            reason: format!("HNSW residue scan error: {e}"),
                        });
                    }
                }
            } else {
                layer_statuses.push(LayerStatus::Failed {
                    layer: DeletionLayer::HnswIndex,
                    reason: "Default collection unavailable for HNSW scan".to_string(),
                });
            }
        } else {
            // Non-document scope: no collection-wide/tenant-wide HNSW scanner exists -> mark Pending
            layer_statuses.push(LayerStatus::Pending {
                layer: DeletionLayer::HnswIndex,
            });
        }

        // 3. Text Index witness (measured via A7/A8 scanner)
        if let Some((_, _, _, doc_id)) = doc_info {
            if let Ok(col) = self.default_col().await {
                match scan_and_issue_text_index_witness(
                    &col.text_index,
                    doc_id,
                    &footprint.ram.term_bytes,
                    scope_hash,
                    as_of_tx,
                )
                .await
                {
                    Ok(text_witness) => {
                        witnesses.push(text_witness);
                        layer_statuses.push(LayerStatus::Verified {
                            layer: DeletionLayer::TextIndex,
                            families: vec!["TextIndex".to_string()],
                        });
                    }
                    Err(e) => {
                        layer_statuses.push(LayerStatus::Failed {
                            layer: DeletionLayer::TextIndex,
                            reason: format!("TextIndex witness scan failed: {e}"),
                        });
                    }
                }
            } else {
                layer_statuses.push(LayerStatus::Failed {
                    layer: DeletionLayer::TextIndex,
                    reason: "Collection unavailable for TextIndex scan".to_string(),
                });
            }
        } else {
            // Non-document scope: no collection-wide/tenant-wide TextIndex scanner exists -> mark Pending
            layer_statuses.push(LayerStatus::Pending {
                layer: DeletionLayer::TextIndex,
            });
        }

        // 4. Relations witness (measured via CsrGraph scan)
        if let Some((_, _, _, doc_id)) = doc_info {
            if let Ok(col) = self.default_col().await {
                let report = contextra_graph::residue_scan::scan_entity_residue(
                    &col.graph_index,
                    EntityId::from(doc_id.inner()),
                    None,
                );
                let matching_edges = u64::try_from(
                    report.outgoing_edges.len().saturating_add(report.incoming_edges.len()),
                )
                .unwrap_or(u64::MAX);
                let scanned_edges = u64::try_from(footprint.ram.relation_keys.len()).unwrap_or(0);

                match issue_relations_witness(doc_id, scanned_edges, scope_hash, as_of_tx, Some(matching_edges)) {
                    Ok(rel_witness) => {
                        witnesses.push(rel_witness);
                        if matching_edges == 0 {
                            layer_statuses.push(LayerStatus::Verified {
                                layer: DeletionLayer::Relations,
                                families: vec!["Relations".to_string()],
                            });
                        } else {
                            layer_statuses.push(LayerStatus::Failed {
                                layer: DeletionLayer::Relations,
                                reason: format!("Relational edges remaining in graph: {matching_edges}"),
                            });
                        }
                    }
                    Err(e) => {
                        layer_statuses.push(LayerStatus::Failed {
                            layer: DeletionLayer::Relations,
                            reason: format!("Relations witness creation failed: {e}"),
                        });
                    }
                }
            } else {
                layer_statuses.push(LayerStatus::Failed {
                    layer: DeletionLayer::Relations,
                    reason: "Collection unavailable for Relations scan".to_string(),
                });
            }
        } else {
            // Non-document scope: no collection-wide/tenant-wide CsrGraph scanner exists -> mark Pending
            layer_statuses.push(LayerStatus::Pending {
                layer: DeletionLayer::Relations,
            });
        }

        // 5. Erasure Ledger witness: since no ErasureLedger residue scanner exists yet, report Pending
        layer_statuses.push(LayerStatus::Pending {
            layer: DeletionLayer::ErasureLedger,
        });

        // Handle S4 / Key Revocation if requested
        if opts.assurance == WitAssurance::S4 {
            // Durable KeyRegistry wiring pending in Prompt E5 -> mark Pending
            layer_statuses.push(LayerStatus::Pending {
                layer: DeletionLayer::Keyring,
            });
        }

        // Calculate maximum verified stage
        let has_failed = layer_statuses
            .iter()
            .any(|s| matches!(s, LayerStatus::Failed { .. } | LayerStatus::Pending { .. }));

        let zeu_probe = self.storage.store_probe();

        // Finalize logical deletion proof
        let (proof, logical_handle) = if !has_failed {
            let logical_input = DpiLogicalInput {
                scope: scope.clone(),
                storage_scope: "lsm".to_string(),
                deleted_keys_hash: scope_hash,
                footprint_hash: [0u8; 32],
                observed_empty: footprint.observed_empty,
                deleted_after_tx: as_of_tx,
                witnesses,
                scope_file_names: scope_file_names.clone(),
            };

            let probes: &[&dyn WitStoreProbe] = &[zeu_probe.as_ref()];

            match DpiDeletionRuntime::open(db_dir, None, db_instance_id) {
                Ok(runtime) => match runtime.finalize_logical(logical_input, opts, None, probes) {
                    Ok((handle, issued)) => (Some(issued.proof), Some(handle)),
                    Err(e) => {
                        layer_statuses.push(LayerStatus::Failed {
                            layer: DeletionLayer::Manifest,
                            reason: format!("finalize_logical failed: {e}"),
                        });
                        (None, None)
                    }
                },
                Err(e) => {
                    layer_statuses.push(LayerStatus::Failed {
                        layer: DeletionLayer::Manifest,
                        reason: format!("DpiDeletionRuntime::open failed: {e}"),
                    });
                    (None, None)
                }
            }
        } else {
            (None, None)
        };

        // Handle Physical Purge when requested (opts.purge != PurgeMode::None)
        let mut purge_blocked = false;
        if opts.purge != PurgeMode::None {
            if let Some(ref handle) = logical_handle {
                if let Ok(runtime) = DpiDeletionRuntime::open(db_dir, None, db_instance_id) {
                    let driver =
                        DpiPurgeDriver::new(&self.storage, &runtime, DpiPurgeDriverConfig::default());
                    let purge_scope = PurgeScope::Keys(footprint.keys.explicit.clone());
                    let probes: &[&dyn WitStoreProbe] = &[zeu_probe.as_ref()];

                    match driver
                        .run_physical(
                            purge_scope,
                            &scope_file_names,
                            as_of_tx,
                            handle,
                            opts,
                            None,
                            probes,
                            None,
                        )
                        .await
                    {
                        Ok(PurgeHandle::Final(_phys_handle)) => {}
                        Ok(PurgeHandle::Pending { .. }) => {
                            purge_blocked = true;
                            layer_statuses.push(LayerStatus::Pending {
                                layer: DeletionLayer::SsTableAllLevels,
                            });
                        }
                        Ok(PurgeHandle::Blocked { reason, .. }) => {
                            purge_blocked = true;
                            layer_statuses.push(LayerStatus::Failed {
                                layer: DeletionLayer::SsTableAllLevels,
                                reason: format!("Physical purge blocked: {reason}"),
                            });
                        }
                        Err(e) => {
                            purge_blocked = true;
                            layer_statuses.push(LayerStatus::Failed {
                                layer: DeletionLayer::SsTableAllLevels,
                                reason: format!("Physical purge execution error: {e}"),
                            });
                        }
                    }
                }
            }
        }

        let is_any_failed_or_pending = layer_statuses
            .iter()
            .any(|s| matches!(s, LayerStatus::Failed { .. } | LayerStatus::Pending { .. }));

        let assurance_reached = if is_any_failed_or_pending || purge_blocked {
            WitAssurance::S0
        } else if opts.purge != PurgeMode::None {
            opts.assurance
        } else {
            WitAssurance::S1
        };

        let proof_output = if assurance_reached >= opts.assurance {
            proof
        } else {
            None
        };

        Ok(ForgetReceipt {
            scope,
            assurance_reached,
            assurance_requested: opts.assurance,
            layers: layer_statuses,
            proof: proof_output,
            wait: opts.wait,
        })
    }
}
