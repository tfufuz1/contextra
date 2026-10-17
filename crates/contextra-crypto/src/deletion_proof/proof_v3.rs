//! Legacy Version 3 Ed25519 deletion proof verification payload construction (M1.4).
//!
//! Creation of Version 3 deletion proofs is forbidden per §20 and ADR-13.
//! Legacy v3 deletion proofs remain verifiable for backwards compatibility.

use super::layer_proof::LayerCleanupProof;
use super::proof::DeletionProof;
use super::types::{DeletionScope, DurabilityProof, ExcludedScope, GraphRepairAttestation};
use crate::error::CryptoError;
use contextra_types::{ContextraError, Result, TxId};

impl DeletionProof {
    /// Creation of Version 3 deletion proofs is forbidden per §20 and ADR-13.
    #[deprecated(
        note = "Creation of v3 deletion proofs is forbidden per §20 and ADR-13. Use ProofBuilder for v4 deletion proofs."
    )]
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
    #[deprecated(
        note = "Creation of v3 deletion proofs is forbidden per §20 and ADR-13. Use ProofBuilder for v4 deletion proofs."
    )]
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
    #[deprecated(
        note = "Creation of v3 deletion proofs is forbidden per §20 and ADR-13. Use ProofBuilder for v4 deletion proofs."
    )]
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
    #[deprecated(
        note = "Creation of v3 deletion proofs is forbidden per §20 and ADR-13. Use ProofBuilder for v4 deletion proofs."
    )]
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
    #[deprecated(
        note = "Creation of v3 deletion proofs is forbidden per §20 and ADR-13. Use ProofBuilder for v4 deletion proofs."
    )]
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
    #[deprecated(
        note = "Creation of v3 deletion proofs is forbidden per §20 and ADR-13. Use ProofBuilder for v4 deletion proofs."
    )]
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
    #[deprecated(
        note = "Creation of v3 deletion proofs is forbidden per §20 and ADR-13. Use ProofBuilder for v4 deletion proofs."
    )]
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

        let scope_bytes =
            bincode::serialize(&self.scope).map_err(|e| CryptoError::Crypto(e.to_string()))?;
        let tx_bytes = self.deleted_after_tx.0.to_le_bytes();
        let timestamp_bytes = self.timestamp.to_le_bytes();
        let covered_layers_bytes = bincode::serialize(&self.covered_layers)
            .map_err(|e| CryptoError::Crypto(e.to_string()))?;
        let excluded_scopes_bytes = bincode::serialize(&self.excluded_scopes)
            .map_err(|e| CryptoError::Crypto(e.to_string()))?;
        let graph_repair_bytes = bincode::serialize(&self.graph_repair)
            .map_err(|e| CryptoError::Crypto(e.to_string()))?;

        let receipt_part = match &self.wal_chain_receipt {
            Some(receipt) => receipt.as_slice(),
            None => &[],
        };

        let audit_pos_bytes = self.audit_chain_position.map(|p| p.to_le_bytes());
        let audit_pos_part = match &audit_pos_bytes {
            Some(ref pos_b) => pos_b.as_slice(),
            None => &[],
        };

        payload.extend_from_slice(&scope_bytes);
        payload.extend_from_slice(&self.deleted_keys_hash);
        payload.extend_from_slice(&tx_bytes);
        payload.extend_from_slice(&timestamp_bytes);
        payload.extend_from_slice(&covered_layers_bytes);
        payload.extend_from_slice(&excluded_scopes_bytes);
        payload.extend_from_slice(&graph_repair_bytes);
        payload.extend_from_slice(receipt_part);
        payload.extend_from_slice(audit_pos_part);

        Ok(payload)
    }
}

#[cfg(test)]
mod tests {
    use super::super::types::{DeletionLayer, DeletionScope, ExcludedScope};
    use super::*;
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
