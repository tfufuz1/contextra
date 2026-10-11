//! Version 2 HMAC-SHA256 deletion proof creation logic.

use super::layer_proof::LayerCleanupProof;
use super::proof::DeletionProof;
use super::types::{hash_deleted_keys_length_prefixed, DeletionLayer, DeletionScope, ExcludedScope};
use super::wal_receipt::compute_hmac_sha256;
use crate::ed25519_proof::SignatureVersion;
use contextra_types::{ContextraError, Result, TxId};

impl DeletionProof {
    /// Erstellt und signiert einen DeletionProof nach Layer-Bereinigung.
    ///
    /// AUFRUFREIHENFOLGE (INV-DELETION-1):
    /// 1. Alle covered_layers physisch bereinigen
    /// 2. WAL-Commit mit Lösch-Intent (P3)
    /// 3. DeletionProof::create() aufrufen
    pub fn create(
        scope: DeletionScope,
        deleted_keys: Vec<Vec<u8>>,
        deleted_after_tx: TxId,
        covered_layers: Vec<LayerCleanupProof>,
        excluded_scopes: Vec<ExcludedScope>,
        proof_key: &[u8],
    ) -> Result<Self> {
        Self::create_with_wal_receipt(
            scope,
            deleted_keys,
            deleted_after_tx,
            covered_layers,
            excluded_scopes,
            None,
            proof_key,
        )
    }

    /// Erstellt und signiert einen DeletionProof (Version 2) inklusive optionaler WAL-HMAC-Kettenquittung.
    pub fn create_with_wal_receipt(
        scope: DeletionScope,
        mut deleted_keys: Vec<Vec<u8>>,
        deleted_after_tx: TxId,
        covered_layers: Vec<LayerCleanupProof>,
        excluded_scopes: Vec<ExcludedScope>,
        wal_chain_receipt: Option<[u8; 32]>,
        proof_key: &[u8],
    ) -> Result<Self> {
        // Keys sortieren für deterministischen Hash
        deleted_keys.sort();

        let deleted_keys_hash = hash_deleted_keys_length_prefixed(&deleted_keys);

        let covered_layers: Vec<DeletionLayer> =
            covered_layers.into_iter().map(|p| p.layer).collect();

        let proof_stub = Self {
            signature_version: SignatureVersion::V2.as_u8(),
            scope,
            deleted_keys_hash,
            deleted_after_tx,
            timestamp: 0,
            signature: Vec::new(),
            covered_layers,
            excluded_scopes,
            graph_repair: Vec::new(),
            wal_chain_receipt,
            audit_chain_position: None,
            integrity_warning: None,
        };

        let full_payload = proof_stub.construct_v2_full_payload()?;
        let signature = compute_hmac_sha256(proof_key, &[&full_payload])?;

        let mut proof = proof_stub;
        proof.signature = signature.to_vec();

        Ok(proof)
    }

    /// Constructs the full, length-prefixed HMAC payload for version 2 proofs binding all fields.
    pub(super) fn construct_v2_full_payload(&self) -> Result<Vec<u8>> {
        let scope_bytes =
            bincode::serialize(&self.scope).map_err(|e| ContextraError::Internal(e.to_string()))?;
        let tx_bytes = self.deleted_after_tx.0.to_le_bytes();
        let timestamp_bytes = self.timestamp.to_le_bytes();
        let covered_layers_bytes = bincode::serialize(&self.covered_layers)
            .map_err(|e| ContextraError::Internal(e.to_string()))?;
        let excluded_scopes_bytes = bincode::serialize(&self.excluded_scopes)
            .map_err(|e| ContextraError::Internal(e.to_string()))?;
        let graph_repair_bytes = bincode::serialize(&self.graph_repair)
            .map_err(|e| ContextraError::Internal(e.to_string()))?;

        let mut payload = Vec::with_capacity(128 + scope_bytes.len() + covered_layers_bytes.len());
        payload.extend_from_slice(b"contextra-hmac-v2-full:");

        let scope_len = u32::try_from(scope_bytes.len()).unwrap_or(u32::MAX);
        payload.extend_from_slice(&scope_len.to_le_bytes());
        payload.extend_from_slice(&scope_bytes);

        payload.extend_from_slice(&self.deleted_keys_hash);
        payload.extend_from_slice(&tx_bytes);
        payload.extend_from_slice(&timestamp_bytes);

        let covered_len = u32::try_from(covered_layers_bytes.len()).unwrap_or(u32::MAX);
        payload.extend_from_slice(&covered_len.to_le_bytes());
        payload.extend_from_slice(&covered_layers_bytes);

        let excluded_len = u32::try_from(excluded_scopes_bytes.len()).unwrap_or(u32::MAX);
        payload.extend_from_slice(&excluded_len.to_le_bytes());
        payload.extend_from_slice(&excluded_scopes_bytes);

        let graph_len = u32::try_from(graph_repair_bytes.len()).unwrap_or(u32::MAX);
        payload.extend_from_slice(&graph_len.to_le_bytes());
        payload.extend_from_slice(&graph_repair_bytes);

        match &self.wal_chain_receipt {
            Some(receipt) => {
                payload.push(1u8);
                payload.extend_from_slice(receipt);
            }
            None => {
                payload.push(0u8);
            }
        }

        match self.audit_chain_position {
            Some(pos) => {
                payload.push(1u8);
                payload.extend_from_slice(&pos.to_le_bytes());
            }
            None => {
                payload.push(0u8);
            }
        }

        match &self.integrity_warning {
            Some(warning) => {
                payload.push(1u8);
                let warn_bytes = warning.as_bytes();
                let warn_len = u32::try_from(warn_bytes.len()).unwrap_or(u32::MAX);
                payload.extend_from_slice(&warn_len.to_le_bytes());
                payload.extend_from_slice(warn_bytes);
            }
            None => {
                payload.push(0u8);
            }
        }

        Ok(payload)
    }
}
