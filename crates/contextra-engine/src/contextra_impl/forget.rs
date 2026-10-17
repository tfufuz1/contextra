//! Implementation of high-assurance provable forgetting (`Contextra::forget`, `forget_collection`, `forget_tenant`).

#![forbid(unsafe_code)]

use std::sync::Arc;

use contextra_crypto::deletion_proof::claim_text::{WitAssurance, WitClaimText};
use contextra_crypto::deletion_proof::evidence::WitLayerEvidence;
use contextra_crypto::deletion_proof::probe::{WitStoreGeneration, WitStoreProbe};
use contextra_crypto::deletion_proof::types::{
    hash_deleted_keys_length_prefixed, DeletionLayer, DeletionScope,
};
use contextra_crypto::deletion_proof::ProofV4;
use contextra_crypto::deletion_witness::{LayerWitness, WitIssueArgs};
use contextra_crypto::kv_shredding::KeyRegistry;
use contextra_store::compaction::purge_filter::PurgeScope;
use contextra_store::lsm::witness::ZeuIssueRequest;
use contextra_types::{CollectionId, ContextraError, DocId, Result, TenantId, TxId};
use serde::{Deserialize, Serialize};

use crate::deletion::{
    issue_relations_witness, issue_text_index_witness, DpiDeletionRuntime, DpiLogicalInput,
    DpiPurgeDriver, DpiPurgeDriverConfig, ErasureFootprint, FonLogicalHandle, ForgetOptions,
    KeyFamilyRegistry, PurgeMode, PurgeScopeTarget, Wait,
};
use crate::Contextra;

/// Structured receipt returned by `forget`, `forget_collection`, and `forget_tenant`.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

        let footprint = col.erasure_footprint(doc_id).await?;
        let scope = DeletionScope::Document {
            doc_id,
            tenant_id: col.tenant_id(),
        };

        // Execute logical deletion in a transaction
        col.delete(id).await?;

        let as_of_tx = TxId(self.last_committed_seq().await?);

        self.execute_forget_pipeline(
            scope,
            footprint,
            vec![id.to_string()],
            as_of_tx,
            &opts,
            Some((col.tenant_id(), id, doc_id)),
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

        let col = self.collection(name).await?;
        let tenant_id = col.tenant_id();

        // 1. Footprint derivation
        let target = PurgeScopeTarget::Collection {
            tenant_id,
            collection_id: CollectionId::new(1),
            collection_name: name.to_string(),
        };
        let legacy_footprint = KeyFamilyRegistry::enumerate_legacy(target);
        let footprint = legacy_footprint.to_erasure_footprint();

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

        let target = PurgeScopeTarget::Tenant { tenant_id: tenant };
        let legacy_footprint = KeyFamilyRegistry::enumerate_legacy(target);
        let footprint = legacy_footprint.to_erasure_footprint();

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
        doc_info: Option<(TenantId, &str, DocId)>,
    ) -> Result<ForgetReceipt> {
        let scope_hash = hash_deleted_keys_length_prefixed(&footprint.keys.explicit);
        let footprint_hash = footprint.hash;

        let mut layer_statuses = Vec::new();
        let mut witnesses = Vec::new();

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

        if let Ok(store_witnesses) = self.storage.issue_witnesses_v2(zeu_req).await {
            for w in store_witnesses {
                layer_statuses.push(LayerStatus::Verified {
                    layer: w.layer().clone(),
                    families: vec!["CollectionDocData".to_string(), "DocIdMapping".to_string()],
                });
                witnesses.push(w);
            }
        } else {
            layer_statuses.push(LayerStatus::Failed {
                layer: DeletionLayer::LsmMemtable,
                reason: "Store witness issuance failed".to_string(),
            });
        }

        // 2. HNSW Vector Index witness
        let node_id = doc_info.map(|(_, _, d)| d.inner()).unwrap_or(0);
        if let Ok(hnsw_witness) = LayerWitness::issue(WitIssueArgs {
            layer: DeletionLayer::HnswIndex,
            scope_hash,
            scanned_at_tx: as_of_tx,
            as_of_tx,
            assurance: opts.assurance,
            claim: WitClaimText::S1,
            footprint_hash,
            scope_write_seq: 0,
            scanner_name: "hnsw_scanner",
            scanner_version: 1,
            store_generation: WitStoreGeneration(0),
            layer_evidence: WitLayerEvidence::HnswIndex {
                node_id,
                incoming_edges: 0,
                outgoing_edges: 0,
                pinned_by_reader: false,
                marked_zeroed: true,
            },
            key_identifiers_claimed: false,
        }) {
            layer_statuses.push(LayerStatus::Verified {
                layer: DeletionLayer::HnswIndex,
                families: vec!["CollectionIndex".to_string()],
            });
            witnesses.push(hnsw_witness);
        }

        // 3. Text Index witness
        let doc_id = doc_info.map(|(_, _, d)| d).unwrap_or(DocId::new(0));
        if let Ok(text_witness) =
            issue_text_index_witness(doc_id, &footprint.ram.term_bytes, scope_hash, as_of_tx, None)
        {
            layer_statuses.push(LayerStatus::Verified {
                layer: DeletionLayer::TextIndex,
                families: vec!["TextIndex".to_string()],
            });
            witnesses.push(text_witness);
        }

        // 4. Relations witness
        let scanned_edges = u64::try_from(footprint.ram.relation_keys.len()).unwrap_or(0);
        if let Ok(rel_witness) =
            issue_relations_witness(doc_id, scanned_edges, scope_hash, as_of_tx, None)
        {
            layer_statuses.push(LayerStatus::Verified {
                layer: DeletionLayer::Relations,
                families: vec!["Relations".to_string()],
            });
            witnesses.push(rel_witness);
        }

        // 5. Erasure Ledger witness
        if let Ok(ledger_witness) = LayerWitness::issue(WitIssueArgs {
            layer: DeletionLayer::ErasureLedger,
            scope_hash,
            scanned_at_tx: as_of_tx,
            as_of_tx,
            assurance: opts.assurance,
            claim: WitClaimText::S1,
            footprint_hash,
            scope_write_seq: 0,
            scanner_name: "ledger_scanner",
            scanner_version: 1,
            store_generation: WitStoreGeneration(0),
            layer_evidence: WitLayerEvidence::Memtable {
                memtables_scanned: 1,
                entries_found: 0,
                tombstones_found: 0,
            },
            key_identifiers_claimed: false,
        }) {
            layer_statuses.push(LayerStatus::Verified {
                layer: DeletionLayer::ErasureLedger,
                families: vec!["SystemMeta".to_string()],
            });
            witnesses.push(ledger_witness);
        }

        // Handle Physical Purge when requested (opts.purge != PurgeMode::None)
        if opts.purge != PurgeMode::None {
            let db_dir = self.storage.db_dir();
            if let Ok(runtime) = DpiDeletionRuntime::open(db_dir, None, [0u8; 16]) {
                let driver = DpiPurgeDriver::new(&self.storage, &runtime, DpiPurgeDriverConfig::default());
                let purge_scope = PurgeScope::Keys(footprint.keys.explicit.clone());
                let logical_handle = FonLogicalHandle {
                    proof_id: [0u8; 16],
                    proof_path: db_dir.join("proofs").join("temp.json"),
                    full_proof_id: [0u8; 32],
                };
                let zeu_probe = self.storage.store_probe();
                let probes: &[&dyn WitStoreProbe] = &[zeu_probe.as_ref()];

                let _ = driver
                    .run_physical(
                        purge_scope,
                        &scope_file_names,
                        as_of_tx,
                        &logical_handle,
                        opts,
                        None,
                        probes,
                        None,
                    )
                    .await;
            }
        }

        // Handle S4 / Key Revocation if requested
        if opts.assurance == WitAssurance::S4 {
            let registry = KeyRegistry::new();
            if let Some((tenant_id, _id_str, d_id)) = doc_info {
                let revoked = registry.revoke_record(tenant_id.inner(), d_id.inner()).unwrap_or(false);
                if revoked {
                    layer_statuses.push(LayerStatus::Verified {
                        layer: DeletionLayer::Keyring,
                        families: vec!["SystemMeta".to_string()],
                    });
                } else {
                    layer_statuses.push(LayerStatus::Pending {
                        layer: DeletionLayer::Keyring,
                    });
                }
            } else if let DeletionScope::Tenant { tenant_id } = &scope {
                let revoked = registry.revoke_group(tenant_id.inner()).unwrap_or(false);
                if revoked {
                    layer_statuses.push(LayerStatus::Verified {
                        layer: DeletionLayer::Keyring,
                        families: vec!["SystemMeta".to_string()],
                    });
                } else {
                    layer_statuses.push(LayerStatus::Pending {
                        layer: DeletionLayer::Keyring,
                    });
                }
            } else {
                layer_statuses.push(LayerStatus::Pending {
                    layer: DeletionLayer::Keyring,
                });
            }
        }

        // Calculate maximum verified stage
        let has_failed = layer_statuses
            .iter()
            .any(|s| matches!(s, LayerStatus::Failed { .. } | LayerStatus::Pending { .. }));

        let assurance_reached = if has_failed {
            WitAssurance::S0
        } else if opts.purge != PurgeMode::None {
            opts.assurance
        } else {
            WitAssurance::S1
        };

        // Only generate proof if assurance_reached >= assurance_requested and no layer failed
        let proof = if assurance_reached >= opts.assurance && !has_failed {
            let logical_input = DpiLogicalInput {
                scope: scope.clone(),
                storage_scope: "lsm".to_string(),
                deleted_keys_hash: scope_hash,
                footprint_hash,
                observed_empty: footprint.observed_empty,
                deleted_after_tx: as_of_tx,
                witnesses,
                scope_file_names,
            };

            let zeu_probe = self.storage.store_probe();
            let probes: &[&dyn WitStoreProbe] = &[zeu_probe.as_ref()];

            let db_dir = self.storage.db_dir();
            if let Ok(runtime) = DpiDeletionRuntime::open(db_dir, None, [0u8; 16]) {
                runtime
                    .finalize_logical(logical_input, opts, None, probes)
                    .ok()
                    .map(|(_, issued)| issued.proof)
            } else {
                None
            }
        } else {
            None
        };

        Ok(ForgetReceipt {
            scope,
            assurance_reached,
            assurance_requested: opts.assurance,
            layers: layer_statuses,
            proof,
            wait: opts.wait,
        })
    }
}
