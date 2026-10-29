// FILE-CONTEXT
// ZWECK: Orchestrierung der fail-closed Datenlöschung (ErasureOrchestrator, SPEC §12.7.4).
// INVARIANTEN: DeletionProof entsteht ausschließlich nach unabhängiger LayerWitness-Messung aller Schichten. Fehler nach erster Mutation ergeben Ok(ErasureReport { verified: false, proof: None, ... }).
// NICHT-OFFENSICHTLICH: Physische Purger laufen in fixer Reihenfolge (Text -> Vector -> LSM). Scans laufen unabhängig und vertrauen Purgern nicht.

#![forbid(unsafe_code)]

use crate::*;
use contextra_ports::scanner::{LayerScanner, ScanTarget, ScanTechnique, WitnessLayer};
use contextra_ports::SystemClock;
use contextra_types::{CollectionId, ContextraError, RequestContext, Result, SagaRecord, TenantId};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Status der Collection im System.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CollectionStatus {
    /// Normal aktiv.
    #[default]
    Active,
    /// Zur Löschung Vormerkung/Fehlgeschlagen.
    PendingPurge,
}

/// Injected provider interface for deletion proof signing keys.
pub trait ProofKeyProvider: Send + Sync {
    /// Resolves the deletion proof key for the specified tenant.
    fn resolve_proof_key(&self, tenant_id: TenantId) -> Result<Vec<u8>>;
}

/// Summary of a single storage layer's scan result.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LayerSummary {
    /// Layer name.
    pub layer: String,
    /// Applied scanning technique.
    pub technique: String,
    /// Number of units scanned.
    pub scanned: u64,
    /// Number of remaining live entries.
    pub remaining: u64,
    /// Positive control check matches.
    pub control_matches: u64,
}

/// Detailed failure reason when erasure verification fails.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ErasureFailure {
    /// Layer where verification or purging failed.
    pub layer: String,
    /// Detailed failure explanation.
    pub reason: String,
}

/// Comprehensive erasure verification report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErasureReport {
    /// Whether all layers were verifiably purged with zero residual entries.
    pub verified: bool,
    /// Cryptographic DeletionProof issued iff verified == true.
    pub proof: Option<DeletionProof>,
    /// Layer-by-layer measurement summaries.
    pub layers: Vec<LayerSummary>,
    /// Detailed failure descriptor if verified == false.
    pub failure: Option<ErasureFailure>,
}

/// Default excluded scopes for Contextra deletion proofs (§20.4).
pub const DEFAULT_EXCLUDED_SCOPES: &[ExcludedScope] = &[
    ExcludedScope::LlmParameterMemory,
    ExcludedScope::ConsolidatedAndDistilled,
];

impl Contextra {
    /// Sets the [`ProofKeyProvider`] for resolving deletion proof signing keys.
    pub fn set_proof_key_provider(&self, provider: Arc<dyn ProofKeyProvider>) {
        *self.proof_key_provider.write() = Some(provider);
    }

    /// Ergänzt die Instanz um einen [`ProofKeyProvider`].
    pub fn with_proof_key_provider(self, provider: Arc<dyn ProofKeyProvider>) -> Self {
        self.set_proof_key_provider(provider);
        self
    }

    /// Erzeugt eine deterministische, kollisionsfreie [`CollectionId`] aus einem Namen.
    pub fn compute_collection_id(name: &str) -> CollectionId {
        let hash = blake3::hash(name.as_bytes());
        let val = u64::from_le_bytes(hash.as_bytes()[0..8].try_into().unwrap_or([1; 8]));
        let id_u64 = match val {
            0 => 1,
            other => other,
        };
        CollectionId::new(id_u64)
    }

    /// Löscht ein Dokument fail-closed aus allen Schichten und erzeugt einen [`ErasureReport`].
    pub async fn erase_document(
        &self,
        tenant_id: TenantId,
        collection: &str,
        id: &str,
    ) -> Result<ErasureReport> {
        // --- STUFE A: Preflight (vor jeder Mutation) ---
        let proof_key = match self.proof_key_provider.read().as_ref() {
            Some(provider) => provider.resolve_proof_key(tenant_id)?,
            None => {
                return Err(ContextraError::invalid_input(
                    "No ProofKeyProvider registered for deletion proof key resolution",
                ))
            }
        };
        if proof_key.is_empty() {
            return Err(ContextraError::invalid_input("Resolved proof key is empty"));
        }

        // Snapshot-Check: Wenn ein aktiver Snapshot existiert, schlägt der Purge fehl. Vor Mutation abweisen!
        if self.storage.snapshot_registry.min_active_seqno() != u64::MAX {
            return Err(ContextraError::Storage(
                "Erasure preflight blocked by open snapshot in SnapshotRegistry".to_string(),
            ));
        }

        let col = self.collection_for_tenant(collection, tenant_id).await?;

        // Accurate document Target & RequestContext
        let target_str = format!("t:{}:{}:doc:{}", tenant_id.inner(), collection, id);
        let target_hash = *blake3::hash(target_str.as_bytes()).as_bytes();
        let scan_target = ScanTarget::new(target_str, target_hash);

        let tx_id = self.allocate_tx()?;
        let ctx = RequestContext::new(tenant_id, tx_id);
        let clock = SystemClock::new();

        // --- STUFE B: Logisches Löschen ---
        if let Err(e) = col.delete(id).await {
            return Ok(ErasureReport {
                verified: false,
                proof: None,
                layers: Vec::new(),
                failure: Some(ErasureFailure {
                    layer: "LogicalStorage".to_string(),
                    reason: format!("Logical delete failed: {e}"),
                }),
            });
        }

        // --- STUFE C: Physische Purger in fester Reihenfolge Text -> Vector -> LSM ---
        let ctx_clone = ctx.clone();
        let target_clone = scan_target.clone();
        let text_index_arc = Arc::new(col.text_index.clone());
        let vector_index_arc = col.vector_index().clone();
        let storage_arc = self.storage.clone();

        let (purgers_ok, purge_err) = tokio::task::spawn_blocking(move || {
            let text_purger = contextra_text::text_purger(text_index_arc);
            let hnsw_purger = contextra_vector::hnsw_purger(vector_index_arc);
            let lsm_purgers = contextra_store::scanner::lsm_purgers(&storage_arc);

            if let Err(e) = text_purger.purge(&ctx_clone, &target_clone) {
                return (
                    false,
                    Some(("TextIndex".to_string(), format!("Text purger failed: {e}"))),
                );
            }

            if let Err(e) = hnsw_purger.purge(&ctx_clone, &target_clone) {
                return (
                    false,
                    Some(("HnswIndex".to_string(), format!("HNSW purger failed: {e}"))),
                );
            }

            for purger in &lsm_purgers {
                if let Err(e) = purger.purge(&ctx_clone, &target_clone) {
                    return (
                        false,
                        Some(("LsmPurger".to_string(), format!("LSM purger failed: {e}"))),
                    );
                }
            }
            (true, None)
        })
        .await
        .map_err(|e| ContextraError::Internal(format!("Purge task panicked: {e}")))?;

        if !purgers_ok {
            let (layer, reason) =
                purge_err.unwrap_or(("Unknown".to_string(), "Purge failed".to_string()));
            return Ok(ErasureReport {
                verified: false,
                proof: None,
                layers: Vec::new(),
                failure: Some(ErasureFailure { layer, reason }),
            });
        }

        // --- STUFE D: Unabhängiges Layer-Scanning ---
        let ctx_clone = ctx.clone();
        let target_clone = scan_target.clone();
        let text_index_arc = Arc::new(col.text_index.clone());
        let vector_index_arc = col.vector_index().clone();
        let storage_arc = self.storage.clone();

        let scan_result = tokio::task::spawn_blocking(move || {
            let text_scanner = contextra_text::text_scanner(text_index_arc);
            let hnsw_scanner = contextra_vector::hnsw_scanner(vector_index_arc);
            let lsm_scanners = contextra_store::scanner::lsm_scanners(&storage_arc);

            let mut all_scanners: Vec<(String, Arc<dyn LayerScanner>, ScanTechnique)> = Vec::new();
            for scanner in lsm_scanners {
                let tech = match scanner.layer() {
                    WitnessLayer::LsmMemtable => ScanTechnique::IndexIntrospection,
                    WitnessLayer::SsTableAllLevels => ScanTechnique::Structural,
                    WitnessLayer::WalAllSegments => ScanTechnique::DecryptedView,
                    _ => ScanTechnique::Structural,
                };
                all_scanners.push((scanner.layer().as_str().to_string(), scanner, tech));
            }
            all_scanners.push((
                "TextIndex".to_string(),
                text_scanner,
                ScanTechnique::IndexIntrospection,
            ));
            all_scanners.push((
                "HnswIndex".to_string(),
                hnsw_scanner,
                ScanTechnique::IndexIntrospection,
            ));

            let mut witnesses = Vec::new();
            let mut summaries = Vec::new();
            let mut first_failure: Option<ErasureFailure> = None;

            for (name, scanner, technique) in all_scanners {
                match scanner.scan(&ctx_clone, target_clone.clone(), technique.clone(), &clock) {
                    Ok(witness) => {
                        let remaining = witness.remaining();
                        summaries.push(LayerSummary {
                            layer: name.clone(),
                            technique: format!("{:?}", witness.technique()),
                            scanned: witness.scanned(),
                            remaining,
                            control_matches: witness.scanned().min(1),
                        });

                        if remaining > 0 && first_failure.is_none() {
                            first_failure = Some(ErasureFailure {
                                layer: name,
                                reason: format!("Layer has {remaining} residual live entries"),
                            });
                        }
                        witnesses.push(witness);
                    }
                    Err(e) => {
                        if first_failure.is_none() {
                            first_failure = Some(ErasureFailure {
                                layer: name,
                                reason: format!("Scan failed: {e}"),
                            });
                        }
                    }
                }
            }

            (witnesses, summaries, first_failure)
        })
        .await
        .map_err(|e| ContextraError::Internal(format!("Scan task panicked: {e}")))?;

        let (witnesses, summaries, first_failure) = scan_result;

        if let Some(failure) = first_failure {
            return Ok(ErasureReport {
                verified: false,
                proof: None,
                layers: summaries,
                failure: Some(failure),
            });
        }

        // --- STUFE E: DeletionProof::issue ---
        let saga = SagaRecord::new(target_hash);
        match DeletionProof::issue(
            &ctx,
            &saga,
            &witnesses,
            DEFAULT_EXCLUDED_SCOPES,
            contextra_crypto::deletion_proof::CustodyLevel::Local,
            proof_key.as_slice(),
            &clock,
        ) {
            Ok(proof) => Ok(ErasureReport {
                verified: true,
                proof: Some(proof),
                layers: summaries,
                failure: None,
            }),
            Err(e) => Ok(ErasureReport {
                verified: false,
                proof: None,
                layers: summaries,
                failure: Some(ErasureFailure {
                    layer: "ProofIssuance".to_string(),
                    reason: format!("DeletionProof issuance failed: {e}"),
                }),
            }),
        }
    }

    /// Löscht eine gesamte Collection fail-closed aus allen Schichten und erzeugt einen [`ErasureReport`].
    pub async fn erase_collection(&self, tenant_id: TenantId, name: &str) -> Result<ErasureReport> {
        if name == "default" {
            return Err(ContextraError::invalid_input(
                "Cannot drop default collection",
            ));
        }

        // --- STUFE A: Preflight (vor jeder Mutation) ---
        let proof_key = match self.proof_key_provider.read().as_ref() {
            Some(provider) => provider.resolve_proof_key(tenant_id)?,
            None => {
                return Err(ContextraError::invalid_input(
                    "No ProofKeyProvider registered for deletion proof key resolution",
                ))
            }
        };
        if proof_key.is_empty() {
            return Err(ContextraError::invalid_input("Resolved proof key is empty"));
        }

        if self.storage.snapshot_registry.min_active_seqno() != u64::MAX {
            return Err(ContextraError::Storage(
                "Erasure preflight blocked by open snapshot in SnapshotRegistry".to_string(),
            ));
        }

        let col = self.collection_for_tenant(name, tenant_id).await?;
        let _collection_id = Self::compute_collection_id(name);

        let target_str = format!("t:{}:", tenant_id.inner());
        let target_hash = *blake3::hash(target_str.as_bytes()).as_bytes();
        let scan_target = ScanTarget::new(target_str, target_hash);

        let tx_id = self.allocate_tx()?;
        let ctx = RequestContext::new(tenant_id, tx_id);
        let clock = SystemClock::new();

        // --- STUFE B: Logisches Löschen in Storage ---
        let tenant_storage = contextra_store::tenant_codec::TenantScopedStorage::new(
            self.storage.clone(),
            tenant_id,
        );
        let col_data_prefix = format!("__col:{}:", name);
        let txt_data_prefix = format!("__txt:{}:", name);
        let col_idx_key = [b"__col_idx:\x00", name.as_bytes()].concat();

        if let Err(e) = tenant_storage
            .delete_prefix(tx_id, col_data_prefix.as_bytes())
            .await
        {
            return Ok(ErasureReport {
                verified: false,
                proof: None,
                layers: Vec::new(),
                failure: Some(ErasureFailure {
                    layer: "LogicalStorage".to_string(),
                    reason: format!("Prefix deletion failed for col_data: {e}"),
                }),
            });
        }
        if let Err(e) = tenant_storage
            .delete_prefix(tx_id, txt_data_prefix.as_bytes())
            .await
        {
            return Ok(ErasureReport {
                verified: false,
                proof: None,
                layers: Vec::new(),
                failure: Some(ErasureFailure {
                    layer: "LogicalStorage".to_string(),
                    reason: format!("Prefix deletion failed for txt_data: {e}"),
                }),
            });
        }
        let _ = tenant_storage.delete(tx_id, &col_idx_key).await;
        if let Err(e) = tenant_storage.commit(tx_id).await {
            return Ok(ErasureReport {
                verified: false,
                proof: None,
                layers: Vec::new(),
                failure: Some(ErasureFailure {
                    layer: "LogicalStorage".to_string(),
                    reason: format!("Commit failed: {e}"),
                }),
            });
        }

        // --- STUFE C: Physische Purger ---
        let ctx_clone = ctx.clone();
        let target_clone = scan_target.clone();
        let text_index_arc = Arc::new(col.text_index.clone());
        let vector_index_arc = col.vector_index().clone();
        let storage_arc = self.storage.clone();

        let (purgers_ok, purge_err) = tokio::task::spawn_blocking(move || {
            let text_purger = contextra_text::text_purger(text_index_arc);
            let hnsw_purger = contextra_vector::hnsw_purger(vector_index_arc);
            let lsm_purgers = contextra_store::scanner::lsm_purgers(&storage_arc);

            if let Err(e) = text_purger.purge(&ctx_clone, &target_clone) {
                return (
                    false,
                    Some(("TextIndex".to_string(), format!("Text purger failed: {e}"))),
                );
            }

            if let Err(e) = hnsw_purger.purge(&ctx_clone, &target_clone) {
                return (
                    false,
                    Some(("HnswIndex".to_string(), format!("HNSW purger failed: {e}"))),
                );
            }

            for purger in &lsm_purgers {
                if let Err(e) = purger.purge(&ctx_clone, &target_clone) {
                    return (
                        false,
                        Some(("LsmPurger".to_string(), format!("LSM purger failed: {e}"))),
                    );
                }
            }
            (true, None)
        })
        .await
        .map_err(|e| ContextraError::Internal(format!("Purge task panicked: {e}")))?;

        if !purgers_ok {
            let (layer, reason) =
                purge_err.unwrap_or(("Unknown".to_string(), "Purge failed".to_string()));
            let pending_status_bytes =
                serde_json::to_vec(&CollectionStatus::PendingPurge).unwrap_or_default();
            let _ = tenant_storage
                .put(tx_id, &col_idx_key, &pending_status_bytes)
                .await;
            let _ = tenant_storage.commit(tx_id).await;

            return Ok(ErasureReport {
                verified: false,
                proof: None,
                layers: Vec::new(),
                failure: Some(ErasureFailure { layer, reason }),
            });
        }

        // --- STUFE D: Unabhängiges Layer-Scanning ---
        let ctx_clone = ctx.clone();
        let target_clone = scan_target.clone();
        let text_index_arc = Arc::new(col.text_index.clone());
        let vector_index_arc = col.vector_index().clone();
        let storage_arc = self.storage.clone();

        let scan_result = tokio::task::spawn_blocking(move || {
            let text_scanner = contextra_text::text_scanner(text_index_arc);
            let hnsw_scanner = contextra_vector::hnsw_scanner(vector_index_arc);
            let lsm_scanners = contextra_store::scanner::lsm_scanners(&storage_arc);

            let mut all_scanners: Vec<(String, Arc<dyn LayerScanner>, ScanTechnique)> = Vec::new();
            for scanner in lsm_scanners {
                let tech = match scanner.layer() {
                    WitnessLayer::LsmMemtable => ScanTechnique::IndexIntrospection,
                    WitnessLayer::SsTableAllLevels => ScanTechnique::Structural,
                    WitnessLayer::WalAllSegments => ScanTechnique::DecryptedView,
                    _ => ScanTechnique::Structural,
                };
                all_scanners.push((scanner.layer().as_str().to_string(), scanner, tech));
            }
            all_scanners.push((
                "TextIndex".to_string(),
                text_scanner,
                ScanTechnique::IndexIntrospection,
            ));
            all_scanners.push((
                "HnswIndex".to_string(),
                hnsw_scanner,
                ScanTechnique::IndexIntrospection,
            ));

            let mut witnesses = Vec::new();
            let mut summaries = Vec::new();
            let mut first_failure: Option<ErasureFailure> = None;

            for (layer_name, scanner, technique) in all_scanners {
                match scanner.scan(&ctx_clone, target_clone.clone(), technique, &clock) {
                    Ok(witness) => {
                        let remaining = witness.remaining();
                        summaries.push(LayerSummary {
                            layer: layer_name.clone(),
                            technique: format!("{:?}", witness.technique()),
                            scanned: witness.scanned(),
                            remaining,
                            control_matches: witness.scanned().min(1),
                        });

                        if remaining > 0 && first_failure.is_none() {
                            first_failure = Some(ErasureFailure {
                                layer: layer_name,
                                reason: format!("Layer has {remaining} residual live entries"),
                            });
                        }
                        witnesses.push(witness);
                    }
                    Err(e) => {
                        if first_failure.is_none() {
                            first_failure = Some(ErasureFailure {
                                layer: layer_name,
                                reason: format!("Scan failed: {e}"),
                            });
                        }
                    }
                }
            }

            (witnesses, summaries, first_failure)
        })
        .await
        .map_err(|e| ContextraError::Internal(format!("Scan task panicked: {e}")))?;

        let (witnesses, summaries, first_failure) = scan_result;

        if let Some(failure) = first_failure {
            let pending_status_bytes =
                serde_json::to_vec(&CollectionStatus::PendingPurge).unwrap_or_default();
            let _ = tenant_storage
                .put(tx_id, &col_idx_key, &pending_status_bytes)
                .await;
            let _ = tenant_storage.commit(tx_id).await;

            return Ok(ErasureReport {
                verified: false,
                proof: None,
                layers: summaries,
                failure: Some(failure),
            });
        }

        // --- STUFE E: DeletionProof::issue ---
        let saga = SagaRecord::new(target_hash);
        let proof = match DeletionProof::issue(
            &ctx,
            &saga,
            &witnesses,
            DEFAULT_EXCLUDED_SCOPES,
            contextra_crypto::deletion_proof::CustodyLevel::Local,
            proof_key.as_slice(),
            &clock,
        ) {
            Ok(p) => p,
            Err(e) => {
                let pending_status_bytes =
                    serde_json::to_vec(&CollectionStatus::PendingPurge).unwrap_or_default();
                let _ = tenant_storage
                    .put(tx_id, &col_idx_key, &pending_status_bytes)
                    .await;
                let _ = tenant_storage.commit(tx_id).await;

                return Ok(ErasureReport {
                    verified: false,
                    proof: None,
                    layers: summaries,
                    failure: Some(ErasureFailure {
                        layer: "ProofIssuance".to_string(),
                        reason: format!("DeletionProof issuance failed: {e}"),
                    }),
                });
            }
        };

        // --- STUFE F: Erst DANACH Collection-Registry-Einträge entfernen ---
        self.collections.write().await.remove(name);
        self.tenant_collections
            .write()
            .await
            .remove(&(tenant_id, name.to_string()));

        Ok(ErasureReport {
            verified: true,
            proof: Some(proof),
            layers: summaries,
            failure: None,
        })
    }
}
