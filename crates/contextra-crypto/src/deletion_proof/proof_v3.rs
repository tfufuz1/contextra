//! Legacy Version 3 Ed25519 deletion proof verification payload construction (M1.4).
//!
//! Creation of Version 3 deletion proofs is forbidden per §20 and ADR-13.
//! Legacy v3 deletion proofs remain verifiable for backwards compatibility.

use super::layer_proof::LayerCleanupProof;
use super::proof::DeletionProof;
use super::types::{
    DeletionScope, DurabilityProof, ExcludedScope, GraphRepairAttestation,
};
use crate::error::CryptoError;
use contextra_types::{ContextraError, Result, TxId};

impl DeletionProof {
    /// Creation of Version 3 deletion proofs is forbidden per §20 and ADR-13.
    #[deprecated(note = "Creation of v3 deletion proofs is forbidden per §20 and ADR-13. Use ProofBuilder for v4 deletion proofs.")]
    pub fn create(
        _scope: DeletionScope,
        _deleted_keys: Vec<Vec<u8>>,
        _deleted_after_tx: TxId,
        _covered_layers: Vec<LayerCleanupProof>,
        _excluded_scopes: Vec<ExcludedScope>,
        _proof_key: &[u8],
    ) -> Result<Self> {
        Err(ContextraError::PolicyViolation(
            "Creation of v3 deletion proofs is forbidden per §20 and ADR-13. Use ProofBuilder for v4 deletion proofs.".to_string(),
        ))
    }

    /// Creation of Version 3 deletion proofs is forbidden per §20 and ADR-13.
    #[deprecated(note = "Creation of v3 deletion proofs is forbidden per §20 and ADR-13. Use ProofBuilder for v4 deletion proofs.")]
    pub fn create_with_wal_receipt(
        _scope: DeletionScope,
        _deleted_keys: Vec<Vec<u8>>,
        _deleted_after_tx: TxId,
        _covered_layers: Vec<LayerCleanupProof>,
        _excluded_scopes: Vec<ExcludedScope>,
        _wal_chain_receipt: Option<[u8; 32]>,
        _proof_key: &[u8],
    ) -> Result<Self> {
        Err(ContextraError::PolicyViolation(
            "Creation of v3 deletion proofs is forbidden per §20 and ADR-13. Use ProofBuilder for v4 deletion proofs.".to_string(),
        ))
    }

    /// Creation of Version 3 deletion proofs is forbidden per §20 and ADR-13.
    #[deprecated(note = "Creation of v3 deletion proofs is forbidden per §20 and ADR-13. Use ProofBuilder for v4 deletion proofs.")]
    pub fn create_with_durability(
        _scope: DeletionScope,
        _deleted_keys: Vec<Vec<u8>>,
        _deleted_after_tx: TxId,
        _covered_layers: Vec<LayerCleanupProof>,
        _excluded_scopes: Vec<ExcludedScope>,
        _durability_proof: Option<&DurabilityProof>,
        _proof_key: &[u8],
    ) -> Result<Self> {
        Err(ContextraError::PolicyViolation(
            "Creation of v3 deletion proofs is forbidden per §20 and ADR-13. Use ProofBuilder for v4 deletion proofs.".to_string(),
        ))
    }

    /// Creation of Version 3 deletion proofs is forbidden per §20 and ADR-13.
    #[deprecated(note = "Creation of v3 deletion proofs is forbidden per §20 and ADR-13. Use ProofBuilder for v4 deletion proofs.")]
    pub fn create_v3(
        _scope: DeletionScope,
        _deleted_keys: Vec<Vec<u8>>,
        _deleted_after_tx: TxId,
        _covered_layers: Vec<LayerCleanupProof>,
        _excluded_scopes: Vec<ExcludedScope>,
        _attested_at: i64,
        _graph_repair: &[GraphRepairAttestation],
        _durability: Option<&DurabilityProof>,
        _signing_key: &ed25519_dalek::SigningKey,
    ) -> Result<Self> {
        Err(ContextraError::PolicyViolation(
            "Creation of v3 deletion proofs is forbidden per §20 and ADR-13. Use ProofBuilder for v4 deletion proofs.".to_string(),
        ))
    }

    /// Creation of Version 3 deletion proofs is forbidden per §20 and ADR-13.
    #[deprecated(note = "Creation of v3 deletion proofs is forbidden per §20 and ADR-13. Use ProofBuilder for v4 deletion proofs.")]
    pub fn create_v3_with_audit_position(
        _scope: DeletionScope,
        _deleted_keys: Vec<Vec<u8>>,
        _deleted_after_tx: TxId,
        _covered_layers: Vec<LayerCleanupProof>,
        _excluded_scopes: Vec<ExcludedScope>,
        _audit_chain_position: Option<u64>,
        _attested_at: i64,
        _graph_repair: &[GraphRepairAttestation],
        _durability: Option<&DurabilityProof>,
        _signing_key: &ed25519_dalek::SigningKey,
    ) -> Result<Self> {
        Err(ContextraError::PolicyViolation(
            "Creation of v3 deletion proofs is forbidden per §20 and ADR-13. Use ProofBuilder for v4 deletion proofs.".to_string(),
        ))
    }

    /// Creation of Version 3 deletion proofs is forbidden per §20 and ADR-13.
    #[deprecated(note = "Creation of v3 deletion proofs is forbidden per §20 and ADR-13. Use ProofBuilder for v4 deletion proofs.")]
    pub fn create_with_wal_receipt_v3(
        _scope: DeletionScope,
        _deleted_keys: Vec<Vec<u8>>,
        _deleted_after_tx: TxId,
        _covered_layers: Vec<LayerCleanupProof>,
        _excluded_scopes: Vec<ExcludedScope>,
        _wal_chain_receipt: Option<[u8; 32]>,
        _attested_at: i64,
        _graph_repair: &[GraphRepairAttestation],
        _durability: Option<&DurabilityProof>,
        _signing_key: &ed25519_dalek::SigningKey,
    ) -> Result<Self> {
        Err(ContextraError::PolicyViolation(
            "Creation of v3 deletion proofs is forbidden per §20 and ADR-13. Use ProofBuilder for v4 deletion proofs.".to_string(),
        ))
    }

    /// Creation of Version 3 deletion proofs is forbidden per §20 and ADR-13.
    #[deprecated(note = "Creation of v3 deletion proofs is forbidden per §20 and ADR-13. Use ProofBuilder for v4 deletion proofs.")]
    pub fn create_full_v3(
        _scope: DeletionScope,
        _deleted_keys: Vec<Vec<u8>>,
        _deleted_after_tx: TxId,
        _covered_layers: Vec<LayerCleanupProof>,
        _excluded_scopes: Vec<ExcludedScope>,
        _wal_chain_receipt: Option<[u8; 32]>,
        _audit_chain_position: Option<u64>,
        _attested_at: i64,
        _graph_repair: &[GraphRepairAttestation],
        _durability: Option<&DurabilityProof>,
        _signing_key: &ed25519_dalek::SigningKey,
    ) -> Result<Self> {
        Err(ContextraError::PolicyViolation(
            "Creation of v3 deletion proofs is forbidden per §20 and ADR-13. Use ProofBuilder for v4 deletion proofs.".to_string(),
        ))
    }

    /// Helper to construct the signed payload for signature version 3 (Ed25519) verification.
    ///
    /// # Domain Separation
    /// Prefixes payload strictly with domain tag `contextra/deletion-proof/v3\0`.
    pub(super) fn construct_v3_payload(&self) -> std::result::Result<Vec<u8>, CryptoError> {
        let mut payload = Vec::with_capacity(128);
        payload.extend_from_slice(b"contextra/deletion-proof/v3\0");

        let mut writer = crate::canonical::CanonicalWriter::new();

        match &self.scope {
            DeletionScope::Document { doc_id, tenant_id } => {
                let mut scope_w = crate::canonical::CanonicalWriter::new();
                scope_w.write_u8(1, 1);
                scope_w.write_u64(2, tenant_id.as_u64());
                scope_w.write_doc_id(3, *doc_id);
                writer.write_bytes(1, scope_w.finish().map_err(|e| CryptoError::Crypto(e.to_string()))?);
            }
            DeletionScope::Collection { collection_id, tenant_id } => {
                let mut scope_w = crate::canonical::CanonicalWriter::new();
                scope_w.write_u8(1, 2);
                scope_w.write_u64(2, tenant_id.as_u64());
                scope_w.write_u64(3, collection_id.0);
                writer.write_bytes(1, scope_w.finish().map_err(|e| CryptoError::Crypto(e.to_string()))?);
            }
            DeletionScope::Tenant { tenant_id } => {
                let mut scope_w = crate::canonical::CanonicalWriter::new();
                scope_w.write_u8(1, 3);
                scope_w.write_u64(2, tenant_id.as_u64());
                writer.write_bytes(1, scope_w.finish().map_err(|e| CryptoError::Crypto(e.to_string()))?);
            }
        }

        writer.write_bytes32(2, &self.deleted_keys_hash);
        writer.write_u64(3, self.deleted_after_tx.0);
        writer.write_u64(4, self.timestamp);

        let mut layers_w = crate::canonical::CanonicalWriter::new();
        for (i, layer) in self.covered_layers.iter().enumerate() {
            layers_w.write_u16((i + 1) as u16, layer.layer_id());
        }
        writer.write_bytes(5, layers_w.finish().map_err(|e| CryptoError::Crypto(e.to_string()))?);

        let mut excl_w = crate::canonical::CanonicalWriter::new();
        for (i, scope) in self.excluded_scopes.iter().enumerate() {
            excl_w.write_u16((i + 1) as u16, scope.id());
        }
        writer.write_bytes(6, excl_w.finish().map_err(|e| CryptoError::Crypto(e.to_string()))?);

        let mut repair_w = crate::canonical::CanonicalWriter::new();
        for (i, rep) in self.graph_repair.iter().enumerate() {
            let mut rep_item = crate::canonical::CanonicalWriter::new();
            rep_item.write_doc_id(1, rep.doc_id);
            rep_item.write_bool(2, rep.verified_no_ghost_pointers);
            rep_item.write_u64(3, rep.attested_at as u64);
            repair_w.write_bytes((i + 1) as u16, rep_item.finish().map_err(|e| CryptoError::Crypto(e.to_string()))?);
        }
        writer.write_bytes(7, repair_w.finish().map_err(|e| CryptoError::Crypto(e.to_string()))?);

        if let Some(ref receipt) = self.wal_chain_receipt {
            writer.write_bytes32(8, receipt);
        }
        if let Some(pos) = self.audit_chain_position {
            writer.write_u64(9, pos);
        }

        let canon_bytes = writer.finish().map_err(|e| CryptoError::Crypto(e.to_string()))?;
        payload.extend_from_slice(&canon_bytes);
        Ok(payload)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::types::{DeletionLayer, DeletionScope, ExcludedScope};
    use crate::ed25519_proof::SignatureVersion;
    use contextra_types::{DocId, TenantId, TxId};

    fn sample_v3_proof_stub() -> DeletionProof {
        DeletionProof {
            signature_version: SignatureVersion::V3.as_u8(),
            scope: DeletionScope::Document {
                doc_id: DocId::from(1u64),
                tenant_id: TenantId::try_new(100).unwrap(),
            },
            deleted_keys_hash: [1u8; 32],
            deleted_after_tx: TxId(42),
            timestamp: 1700000000,
            signature: Vec::new(),
            covered_layers: vec![DeletionLayer::LsmMemtable],
            uncovered_layers: Vec::new(),
            excluded_scopes: vec![ExcludedScope::LlmParameterMemory],
            graph_repair: Vec::new(),
            wal_chain_receipt: None,
            audit_chain_position: None,
            integrity_warning: None,
        }
    }

    #[test]
    fn test_v3_payload_domain_tag_prefix() {
        let proof = sample_v3_proof_stub();
        let payload = proof.construct_v3_payload().unwrap();
        assert!(payload.starts_with(b"contextra/deletion-proof/v3\0"));
    }

    #[test]
    fn test_v3_creation_returns_policy_violation() {
        let scope = DeletionScope::Document {
            doc_id: DocId(1),
            tenant_id: TenantId::try_new(100).unwrap(),
        };
        let key = vec![0u8; 32];
        #[allow(deprecated)]
        let res = DeletionProof::create(scope, vec![], TxId(1), vec![], vec![], &key);
        assert!(res.is_err());
        if let Err(contextra_types::ContextraError::PolicyViolation(msg)) = res {
            assert!(msg.contains("forbidden"));
        } else {
            panic!("expected PolicyViolation");
        }
    }
}
