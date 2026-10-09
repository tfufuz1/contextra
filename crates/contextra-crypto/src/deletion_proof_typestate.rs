// FILE-CONTEXT
// ZWECK: Type-state builder for DeletionProof layer-coverage enforcing complete layer coverage at compile time.
// INVARIANTEN: INV-DELETION-1: Proof construction via type-state builder enforces all 7 storage layers are covered.
// NICHT-OFFENSICHTLICH: Uses 7 phantom type parameters matching all variants of DeletionLayer. finish() is only available when all parameters are Cleaned.

#![forbid(unsafe_code)]

//! Compile-time type-state builder for [`DeletionProof`] layer coverage verification.

use crate::deletion_proof::{
    DeletionLayer, DeletionProof, DeletionScope, ExcludedScope, LayerCleanupProof,
};
use contextra_types::{Result, TxId};
use std::marker::PhantomData;

/// Marker type indicating a storage layer has not yet been attested as cleaned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Missing;

/// Marker type indicating a storage layer has been attested as cleaned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cleaned;

/// Type-state builder enforcing complete storage layer coverage at compile time for [`DeletionProof`].
///
/// # Example
/// ```rust
/// use contextra_crypto::deletion_proof::{DeletionLayer, DeletionScope, ExcludedScope, LayerCleanupProof};
/// use contextra_crypto::deletion_proof_typestate::DeletionProofBuilder;
/// use contextra_types::{DocId, TenantId, TxId};
///
/// # fn example() -> contextra_types::Result<()> {
/// let lsm_proof = LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0)?;
/// let sst_proof = LayerCleanupProof::new_after_verified_empty(DeletionLayer::SsTableAllLevels, 0)?;
/// let hnsw_proof = LayerCleanupProof::new_after_verified_empty(DeletionLayer::HnswIndex, 0)?;
/// let wal_proof = LayerCleanupProof::new_after_verified_empty(DeletionLayer::WalAllSegments { seq_after: 10 }, 0)?;
/// let csr_proof = LayerCleanupProof::new_after_verified_empty(DeletionLayer::CsrGraph, 0)?;
/// let kv_proof = LayerCleanupProof::new_after_verified_empty(DeletionLayer::KvCacheSegments, 0)?;
/// let emb_proof = LayerCleanupProof::new_after_verified_empty(DeletionLayer::EmbeddingCache, 0)?;
///
/// let proof = DeletionProofBuilder::new(
///     DeletionScope::Document { doc_id: DocId(1), tenant_id: TenantId::try_new(10).unwrap() },
///     vec![b"key1".to_vec()],
///     TxId(100),
///     vec![ExcludedScope::LlmParameterMemory],
///     vec![0u8; 32],
/// )
/// .with_lsm_memtable_cleanup(lsm_proof)
/// .with_sstable_all_levels_cleanup(sst_proof)
/// .with_hnsw_index_cleanup(hnsw_proof)
/// .with_wal_all_segments_cleanup(wal_proof)
/// .with_csr_graph_cleanup(csr_proof)
/// .with_kv_cache_segments_cleanup(kv_proof)
/// .with_embedding_cache_cleanup(emb_proof)
/// .finish()?;
/// # Ok(())
/// # }
/// ```
///
/// ## Compile Error Example
/// Attempting to call `.finish()` before all 7 layers are transitioned to [`Cleaned`] will fail at compile time:
/// ```ignore
/// // COMPILE ERROR: method `finish` not found on `DeletionProofBuilder<Cleaned, Missing, Missing, Missing, Missing, Missing, Missing>`
/// let proof = DeletionProofBuilder::new(...)
///     .with_lsm_memtable_cleanup(lsm_proof)
///     // missing other 6 storage layers!
///     .finish();
/// ```
#[derive(Debug)]
pub struct DeletionProofBuilder<
    Lsm = Missing,
    Sst = Missing,
    Hnsw = Missing,
    Wal = Missing,
    Csr = Missing,
    Kv = Missing,
    Emb = Missing,
> {
    scope: DeletionScope,
    deleted_keys: Vec<Vec<u8>>,
    deleted_after_tx: TxId,
    excluded_scopes: Vec<ExcludedScope>,
    wal_chain_receipt: Option<[u8; 32]>,
    graph_repair: Vec<crate::deletion_proof::GraphRepairAttestation>,
    proof_key: Vec<u8>,
    covered_layers: Vec<LayerCleanupProof>,
    _marker: PhantomData<(Lsm, Sst, Hnsw, Wal, Csr, Kv, Emb)>,
}

impl DeletionProofBuilder<Missing, Missing, Missing, Missing, Missing, Missing, Missing> {
    /// Constructs a new type-state builder initialized with all storage layers as [`Missing`].
    pub fn new(
        scope: DeletionScope,
        deleted_keys: Vec<Vec<u8>>,
        deleted_after_tx: TxId,
        excluded_scopes: Vec<ExcludedScope>,
        proof_key: Vec<u8>,
    ) -> Self {
        Self {
            scope,
            deleted_keys,
            deleted_after_tx,
            excluded_scopes,
            wal_chain_receipt: None,
            graph_repair: Vec::new(),
            proof_key,
            covered_layers: Vec::new(),
            _marker: PhantomData,
        }
    }
}

impl<Lsm, Sst, Hnsw, Wal, Csr, Kv, Emb> DeletionProofBuilder<Lsm, Sst, Hnsw, Wal, Csr, Kv, Emb> {
    /// Sets an optional WAL HMAC chain receipt.
    pub fn with_wal_chain_receipt(mut self, receipt: Option<[u8; 32]>) -> Self {
        self.wal_chain_receipt = receipt;
        self
    }

    /// Sets graph repair attestations required for HNSW layer cleanup.
    pub fn with_graph_repair(
        mut self,
        graph_repair: Vec<crate::deletion_proof::GraphRepairAttestation>,
    ) -> Self {
        self.graph_repair = graph_repair;
        self
    }
}

impl<Sst, Hnsw, Wal, Csr, Kv, Emb> DeletionProofBuilder<Missing, Sst, Hnsw, Wal, Csr, Kv, Emb> {
    /// Attests physical cleanup of the LSM memtable layer.
    pub fn with_lsm_memtable_cleanup(
        mut self,
        proof: LayerCleanupProof,
    ) -> DeletionProofBuilder<Cleaned, Sst, Hnsw, Wal, Csr, Kv, Emb> {
        assert!(
            matches!(proof.layer(), DeletionLayer::LsmMemtable),
            "expected LayerCleanupProof for DeletionLayer::LsmMemtable, got {:?}",
            proof.layer()
        );
        self.covered_layers.push(proof);
        DeletionProofBuilder {
            scope: self.scope,
            deleted_keys: self.deleted_keys,
            deleted_after_tx: self.deleted_after_tx,
            excluded_scopes: self.excluded_scopes,
            wal_chain_receipt: self.wal_chain_receipt,
            graph_repair: self.graph_repair,
            proof_key: self.proof_key,
            covered_layers: self.covered_layers,
            _marker: PhantomData,
        }
    }
}

impl<Lsm, Hnsw, Wal, Csr, Kv, Emb> DeletionProofBuilder<Lsm, Missing, Hnsw, Wal, Csr, Kv, Emb> {
    /// Attests physical cleanup of all SSTable levels.
    pub fn with_sstable_all_levels_cleanup(
        mut self,
        proof: LayerCleanupProof,
    ) -> DeletionProofBuilder<Lsm, Cleaned, Hnsw, Wal, Csr, Kv, Emb> {
        assert!(
            matches!(proof.layer(), DeletionLayer::SsTableAllLevels),
            "expected LayerCleanupProof for DeletionLayer::SsTableAllLevels, got {:?}",
            proof.layer()
        );
        self.covered_layers.push(proof);
        DeletionProofBuilder {
            scope: self.scope,
            deleted_keys: self.deleted_keys,
            deleted_after_tx: self.deleted_after_tx,
            excluded_scopes: self.excluded_scopes,
            wal_chain_receipt: self.wal_chain_receipt,
            graph_repair: self.graph_repair,
            proof_key: self.proof_key,
            covered_layers: self.covered_layers,
            _marker: PhantomData,
        }
    }
}

impl<Lsm, Sst, Wal, Csr, Kv, Emb> DeletionProofBuilder<Lsm, Sst, Missing, Wal, Csr, Kv, Emb> {
    /// Attests physical cleanup of the HNSW vector index.
    pub fn with_hnsw_index_cleanup(
        mut self,
        proof: LayerCleanupProof,
    ) -> DeletionProofBuilder<Lsm, Sst, Cleaned, Wal, Csr, Kv, Emb> {
        assert!(
            matches!(proof.layer(), DeletionLayer::HnswIndex),
            "expected LayerCleanupProof for DeletionLayer::HnswIndex, got {:?}",
            proof.layer()
        );
        self.covered_layers.push(proof);
        DeletionProofBuilder {
            scope: self.scope,
            deleted_keys: self.deleted_keys,
            deleted_after_tx: self.deleted_after_tx,
            excluded_scopes: self.excluded_scopes,
            wal_chain_receipt: self.wal_chain_receipt,
            graph_repair: self.graph_repair,
            proof_key: self.proof_key,
            covered_layers: self.covered_layers,
            _marker: PhantomData,
        }
    }
}

impl<Lsm, Sst, Hnsw, Csr, Kv, Emb> DeletionProofBuilder<Lsm, Sst, Hnsw, Missing, Csr, Kv, Emb> {
    /// Attests physical cleanup of all WAL log segments.
    pub fn with_wal_all_segments_cleanup(
        mut self,
        proof: LayerCleanupProof,
    ) -> DeletionProofBuilder<Lsm, Sst, Hnsw, Cleaned, Csr, Kv, Emb> {
        assert!(
            matches!(proof.layer(), DeletionLayer::WalAllSegments { .. }),
            "expected LayerCleanupProof for DeletionLayer::WalAllSegments, got {:?}",
            proof.layer()
        );
        self.covered_layers.push(proof);
        DeletionProofBuilder {
            scope: self.scope,
            deleted_keys: self.deleted_keys,
            deleted_after_tx: self.deleted_after_tx,
            excluded_scopes: self.excluded_scopes,
            wal_chain_receipt: self.wal_chain_receipt,
            graph_repair: self.graph_repair,
            proof_key: self.proof_key,
            covered_layers: self.covered_layers,
            _marker: PhantomData,
        }
    }
}

impl<Lsm, Sst, Hnsw, Wal, Kv, Emb> DeletionProofBuilder<Lsm, Sst, Hnsw, Wal, Missing, Kv, Emb> {
    /// Attests physical cleanup of the CSR knowledge graph.
    pub fn with_csr_graph_cleanup(
        mut self,
        proof: LayerCleanupProof,
    ) -> DeletionProofBuilder<Lsm, Sst, Hnsw, Wal, Cleaned, Kv, Emb> {
        assert!(
            matches!(proof.layer(), DeletionLayer::CsrGraph),
            "expected LayerCleanupProof for DeletionLayer::CsrGraph, got {:?}",
            proof.layer()
        );
        self.covered_layers.push(proof);
        DeletionProofBuilder {
            scope: self.scope,
            deleted_keys: self.deleted_keys,
            deleted_after_tx: self.deleted_after_tx,
            excluded_scopes: self.excluded_scopes,
            wal_chain_receipt: self.wal_chain_receipt,
            graph_repair: self.graph_repair,
            proof_key: self.proof_key,
            covered_layers: self.covered_layers,
            _marker: PhantomData,
        }
    }
}

impl<Lsm, Sst, Hnsw, Wal, Csr, Emb> DeletionProofBuilder<Lsm, Sst, Hnsw, Wal, Csr, Missing, Emb> {
    /// Attests physical cleanup of key-value cache segments.
    pub fn with_kv_cache_segments_cleanup(
        mut self,
        proof: LayerCleanupProof,
    ) -> DeletionProofBuilder<Lsm, Sst, Hnsw, Wal, Csr, Cleaned, Emb> {
        assert!(
            matches!(proof.layer(), DeletionLayer::KvCacheSegments),
            "expected LayerCleanupProof for DeletionLayer::KvCacheSegments, got {:?}",
            proof.layer()
        );
        self.covered_layers.push(proof);
        DeletionProofBuilder {
            scope: self.scope,
            deleted_keys: self.deleted_keys,
            deleted_after_tx: self.deleted_after_tx,
            excluded_scopes: self.excluded_scopes,
            wal_chain_receipt: self.wal_chain_receipt,
            graph_repair: self.graph_repair,
            proof_key: self.proof_key,
            covered_layers: self.covered_layers,
            _marker: PhantomData,
        }
    }
}

impl<Lsm, Sst, Hnsw, Wal, Csr, Kv> DeletionProofBuilder<Lsm, Sst, Hnsw, Wal, Csr, Kv, Missing> {
    /// Attests physical cleanup of the embedding cache.
    pub fn with_embedding_cache_cleanup(
        mut self,
        proof: LayerCleanupProof,
    ) -> DeletionProofBuilder<Lsm, Sst, Hnsw, Wal, Csr, Kv, Cleaned> {
        assert!(
            matches!(proof.layer(), DeletionLayer::EmbeddingCache),
            "expected LayerCleanupProof for DeletionLayer::EmbeddingCache, got {:?}",
            proof.layer()
        );
        self.covered_layers.push(proof);
        DeletionProofBuilder {
            scope: self.scope,
            deleted_keys: self.deleted_keys,
            deleted_after_tx: self.deleted_after_tx,
            excluded_scopes: self.excluded_scopes,
            wal_chain_receipt: self.wal_chain_receipt,
            graph_repair: self.graph_repair,
            proof_key: self.proof_key,
            covered_layers: self.covered_layers,
            _marker: PhantomData,
        }
    }
}

impl DeletionProofBuilder<Cleaned, Cleaned, Cleaned, Cleaned, Cleaned, Cleaned, Cleaned> {
    /// Finalizes the builder and creates a fully attested [`DeletionProof`].
    ///
    /// This method is only available when all 7 storage layers have been marked as [`Cleaned`].
    pub fn finish(self) -> Result<DeletionProof> {
        let signing_key = if self.proof_key.len() >= 32 {
            let mut key_bytes = [0u8; 32];
            key_bytes.copy_from_slice(&self.proof_key[..32]);
            ed25519_dalek::SigningKey::from_bytes(&key_bytes)
        } else {
            return Err(contextra_types::ContextraError::Internal(
                "proof_key must be at least 32 bytes for Ed25519 v3 signature".to_string(),
            ));
        };

        DeletionProof::create_full_v3(
            self.scope,
            self.deleted_keys,
            self.deleted_after_tx,
            self.covered_layers,
            self.excluded_scopes,
            self.wal_chain_receipt,
            None,
            0,
            &self.graph_repair,
            &signing_key,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use contextra_types::{DocId, TenantId};

    fn test_proof_key() -> Vec<u8> {
        vec![0x42u8; 32]
    }

    #[test]
    fn test_typestate_builder_complete_chain_matches_direct_creation() {
        let scope = DeletionScope::Document {
            doc_id: DocId(42),
            tenant_id: TenantId::try_new(1).unwrap(),
        };
        let deleted_keys = vec![b"key1".to_vec(), b"key2".to_vec()];
        let tx_id = TxId(100);
        let excluded = vec![ExcludedScope::LlmParameterMemory];
        let receipt = Some([0xAAu8; 32]);
        let key = test_proof_key();

        let lsm_proof =
            LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0).unwrap();
        let sst_proof =
            LayerCleanupProof::new_after_verified_empty(DeletionLayer::SsTableAllLevels, 0)
                .unwrap();
        let hnsw_proof =
            LayerCleanupProof::new_after_verified_empty(DeletionLayer::HnswIndex, 0).unwrap();
        let wal_proof = LayerCleanupProof::new_after_verified_empty(
            DeletionLayer::WalAllSegments { seq_after: 50 },
            0,
        )
        .unwrap();
        let csr_proof =
            LayerCleanupProof::new_after_verified_empty(DeletionLayer::CsrGraph, 0).unwrap();
        let kv_proof =
            LayerCleanupProof::new_after_verified_empty(DeletionLayer::KvCacheSegments, 0).unwrap();
        let emb_proof =
            LayerCleanupProof::new_after_verified_empty(DeletionLayer::EmbeddingCache, 0).unwrap();

        let graph_repair = vec![crate::deletion_proof::GraphRepairAttestation {
            doc_id: DocId(42),
            verified_no_ghost_pointers: true,
            attested_at: 0,
        }];

        // Direct creation reference
        let expected_proof = DeletionProof::create_full_v3(
            scope.clone(),
            deleted_keys.clone(),
            tx_id,
            vec![
                lsm_proof.clone(),
                sst_proof.clone(),
                hnsw_proof.clone(),
                wal_proof.clone(),
                csr_proof.clone(),
                kv_proof.clone(),
                emb_proof.clone(),
            ],
            excluded.clone(),
            receipt,
            None,
            0,
            &graph_repair,
            Some(&crate::deletion_proof::DurabilityProof::new(100)),
            &ed25519_dalek::SigningKey::from_bytes(key[..32].try_into().unwrap()),
        )
        .unwrap();

        // Typestate builder creation
        let built_proof =
            DeletionProofBuilder::new(scope, deleted_keys, tx_id, excluded, key.clone())
                .with_wal_chain_receipt(receipt)
                .with_graph_repair(graph_repair)
                .with_lsm_memtable_cleanup(lsm_proof)
                .with_sstable_all_levels_cleanup(sst_proof)
                .with_hnsw_index_cleanup(hnsw_proof)
                .with_wal_all_segments_cleanup(wal_proof)
                .with_csr_graph_cleanup(csr_proof)
                .with_kv_cache_segments_cleanup(kv_proof)
                .with_embedding_cache_cleanup(emb_proof)
                .finish()
                .unwrap();

        let sk = ed25519_dalek::SigningKey::from_bytes(key[..32].try_into().unwrap());
        let vk = sk.verifying_key();

        assert_eq!(built_proof, expected_proof);
        assert!(built_proof.verify(crate::deletion_proof::VerificationKey::Ed25519(&vk)).unwrap());
    }

    #[test]
    #[should_panic(expected = "expected LayerCleanupProof for DeletionLayer::LsmMemtable")]
    fn test_typestate_builder_layer_mismatch_panics() {
        let scope = DeletionScope::Tenant {
            tenant_id: TenantId::try_new(1).unwrap(),
        };

        let wrong_proof =
            LayerCleanupProof::new_after_verified_empty(DeletionLayer::HnswIndex, 0).unwrap();

        let _ = DeletionProofBuilder::new(scope, vec![], TxId(1), vec![], test_proof_key())
            .with_lsm_memtable_cleanup(wrong_proof);
    }
}
