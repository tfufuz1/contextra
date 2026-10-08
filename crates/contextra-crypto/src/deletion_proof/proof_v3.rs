//! Version 3 Ed25519 deletion proof creation logic.

use super::layer_proof::LayerCleanupProof;
use super::proof::DeletionProof;
use super::types::{
    hash_deleted_keys_length_prefixed, DeletionLayer, DeletionScope, ExcludedScope,
    GraphRepairAttestation,
};
use crate::ed25519_proof::SignatureVersion;
use crate::error::CryptoError;
use contextra_types::{error::HnswDeletionError, ContextraError, Result, TxId};

impl DeletionProof {
    /// Erstellt und signiert einen DeletionProof (Version 3) mit Ed25519.
    #[allow(clippy::too_many_arguments)]
    pub fn create_v3(
        scope: DeletionScope,
        deleted_keys: Vec<Vec<u8>>,
        deleted_after_tx: TxId,
        covered_layers: Vec<LayerCleanupProof>,
        excluded_scopes: Vec<ExcludedScope>,
        attested_at: i64,
        graph_repair: &[GraphRepairAttestation],
        signing_key: &ed25519_dalek::SigningKey,
    ) -> Result<Self> {
        Self::create_v3_with_audit_position(
            scope,
            deleted_keys,
            deleted_after_tx,
            covered_layers,
            excluded_scopes,
            None,
            attested_at,
            graph_repair,
            signing_key,
        )
    }

    /// Erstellt und signiert einen DeletionProof (Version 3, Ed25519) inklusive Verweis auf den Audit-Chain-Eintrag.
    #[allow(clippy::too_many_arguments)]
    pub fn create_v3_with_audit_position(
        scope: DeletionScope,
        deleted_keys: Vec<Vec<u8>>,
        deleted_after_tx: TxId,
        covered_layers: Vec<LayerCleanupProof>,
        excluded_scopes: Vec<ExcludedScope>,
        audit_chain_position: Option<u64>,
        attested_at: i64,
        graph_repair: &[GraphRepairAttestation],
        signing_key: &ed25519_dalek::SigningKey,
    ) -> Result<Self> {
        Self::create_full_v3(
            scope,
            deleted_keys,
            deleted_after_tx,
            covered_layers,
            excluded_scopes,
            None,
            audit_chain_position,
            attested_at,
            graph_repair,
            signing_key,
        )
    }

    /// Erstellt und signiert einen DeletionProof (Version 3, Ed25519) inklusive optionaler WAL-HMAC-Kettenquittung.
    #[allow(clippy::too_many_arguments)]
    pub fn create_with_wal_receipt_v3(
        scope: DeletionScope,
        deleted_keys: Vec<Vec<u8>>,
        deleted_after_tx: TxId,
        covered_layers: Vec<LayerCleanupProof>,
        excluded_scopes: Vec<ExcludedScope>,
        wal_chain_receipt: Option<[u8; 32]>,
        attested_at: i64,
        graph_repair: &[GraphRepairAttestation],
        signing_key: &ed25519_dalek::SigningKey,
    ) -> Result<Self> {
        Self::create_full_v3(
            scope,
            deleted_keys,
            deleted_after_tx,
            covered_layers,
            excluded_scopes,
            wal_chain_receipt,
            None,
            attested_at,
            graph_repair,
            signing_key,
        )
    }

    /// Erstellt und signiert einen DeletionProof (Version 3, Ed25519) mit allen optionalen Erweiterungen.
    #[allow(clippy::too_many_arguments)]
    pub fn create_full_v3(
        scope: DeletionScope,
        mut deleted_keys: Vec<Vec<u8>>,
        deleted_after_tx: TxId,
        covered_layers: Vec<LayerCleanupProof>,
        excluded_scopes: Vec<ExcludedScope>,
        wal_chain_receipt: Option<[u8; 32]>,
        audit_chain_position: Option<u64>,
        attested_at: i64,
        graph_repair: &[GraphRepairAttestation],
        signing_key: &ed25519_dalek::SigningKey,
    ) -> Result<Self> {
        deleted_keys.sort();
        let deleted_keys_hash = hash_deleted_keys_length_prefixed(&deleted_keys);

        let covered_layers: Vec<DeletionLayer> =
            covered_layers.into_iter().map(|p| p.layer).collect();

        if covered_layers.contains(&DeletionLayer::HnswIndex) && graph_repair.is_empty() {
            return Err(ContextraError::GraphRepairFailed(
                HnswDeletionError::VerificationFailed {
                    remaining_pointers: 1,
                },
            ));
        }

        let timestamp = if attested_at >= 0 {
            #[allow(clippy::cast_sign_loss)]
            {
                attested_at as u64
            }
        } else {
            0u64
        };

        let proof_stub = Self {
            signature_version: SignatureVersion::V3.as_u8(),
            scope,
            deleted_keys_hash,
            deleted_after_tx,
            timestamp,
            signature: Vec::new(),
            covered_layers,
            excluded_scopes,
            graph_repair: graph_repair.to_vec(),
            wal_chain_receipt,
            audit_chain_position,
            integrity_warning: None,
        };

        let payload = proof_stub
            .construct_v3_payload()
            .map_err(|e| ContextraError::Internal(e.to_string()))?;

        use ed25519_dalek::Signer;
        let sig = signing_key.sign(&payload);

        let mut proof = proof_stub;
        proof.signature = sig.to_bytes().to_vec();

        Ok(proof)
    }

    /// Helper to construct the signed payload for signature version 3 (Ed25519).
    pub(super) fn construct_v3_payload(&self) -> std::result::Result<Vec<u8>, CryptoError> {
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
        let receipt_bytes = self.wal_chain_receipt.unwrap_or([0u8; 32]);
        let receipt_part = if self.wal_chain_receipt.is_some() {
            receipt_bytes.as_slice()
        } else {
            &[]
        };

        let audit_pos_bytes = self.audit_chain_position.map(|p| p.to_le_bytes());
        let audit_pos_part = if let Some(ref pos_b) = audit_pos_bytes {
            pos_b.as_slice()
        } else {
            &[]
        };

        let mut payload = Vec::with_capacity(
            scope_bytes.len()
                + 32
                + tx_bytes.len()
                + timestamp_bytes.len()
                + covered_layers_bytes.len()
                + excluded_scopes_bytes.len()
                + graph_repair_bytes.len()
                + receipt_part.len()
                + audit_pos_part.len(),
        );
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
