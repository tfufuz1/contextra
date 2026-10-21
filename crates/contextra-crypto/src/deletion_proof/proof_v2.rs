//! Version 2 HMAC-SHA256 deletion proof creation logic.

use super::layer_proof::LayerCleanupProof;
use super::proof::DeletionProof;
use super::types::{
    hash_deleted_keys_length_prefixed, DeletionLayer, DeletionScope, ExcludedScope,
};
use super::wal_receipt::compute_hmac_sha256;
use crate::ed25519_proof::SignatureVersion;
use contextra_types::{Result, TxId};

impl DeletionProof {
    /// Erstellt und signiert einen DeletionProof nach Layer-Bereinigung.
    ///
    /// AUFRUFREIHENFOLGE (INV-DELETION-1):
    /// 1. Alle covered_layers physisch bereinigen
    /// 2. WAL-Commit mit Lösch-Intent (P3)
    /// 3. DeletionProof::create() aufrufen
    #[cfg_attr(
        not(test),
        deprecated(
            note = "Creation of v2 deletion proofs is forbidden per §20 and ADR-13. Use DeletionProofBuilder for v4 deletion proofs."
        )
    )]
    pub fn create_v2(
        scope: DeletionScope,
        deleted_keys: Vec<Vec<u8>>,
        deleted_after_tx: TxId,
        covered_layers: Vec<LayerCleanupProof>,
        excluded_scopes: Vec<ExcludedScope>,
        proof_key: &[u8],
    ) -> Result<Self> {
        // LEGACY: Calling deprecated create_v2_with_wal_receipt within legacy v2 creation path.
        #[allow(deprecated)]
        Self::create_v2_with_wal_receipt(
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
    #[cfg_attr(
        not(test),
        deprecated(
            note = "Creation of v2 deletion proofs is forbidden per §20 and ADR-13. Use DeletionProofBuilder for v4 deletion proofs."
        )
    )]
    pub fn create_v2_with_wal_receipt(
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
            uncovered_layers: Vec::new(),
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
        let mut payload = Vec::with_capacity(128);
        payload.extend_from_slice(b"contextra-hmac-v2-full:");

        let mut writer = crate::canonical::CanonicalWriter::new();

        match &self.scope {
            DeletionScope::Document { doc_id, tenant_id } => {
                let mut scope_w = crate::canonical::CanonicalWriter::new();
                scope_w.write_u8(1, 1);
                scope_w.write_u64(2, tenant_id.as_u64());
                scope_w.write_u64(3, doc_id.0);
                writer.write_bytes(1, scope_w.finish()?);
            }
            DeletionScope::Collection { collection_id, tenant_id } => {
                let mut scope_w = crate::canonical::CanonicalWriter::new();
                scope_w.write_u8(1, 2);
                scope_w.write_u64(2, tenant_id.as_u64());
                scope_w.write_u64(3, collection_id.0);
                writer.write_bytes(1, scope_w.finish()?);
            }
            DeletionScope::Tenant { tenant_id } => {
                let mut scope_w = crate::canonical::CanonicalWriter::new();
                scope_w.write_u8(1, 3);
                scope_w.write_u64(2, tenant_id.as_u64());
                writer.write_bytes(1, scope_w.finish()?);
            }
        }

        writer.write_bytes32(2, &self.deleted_keys_hash);
        writer.write_u64(3, self.deleted_after_tx.0);
        writer.write_u64(4, self.timestamp);

        let mut layers_w = crate::canonical::CanonicalWriter::new();
        for (i, layer) in self.covered_layers.iter().enumerate() {
            layers_w.write_u16((i + 1) as u16, layer.layer_id());
        }
        writer.write_bytes(5, layers_w.finish()?);

        let mut excl_w = crate::canonical::CanonicalWriter::new();
        for (i, scope) in self.excluded_scopes.iter().enumerate() {
            excl_w.write_u16((i + 1) as u16, scope.id());
        }
        writer.write_bytes(6, excl_w.finish()?);

        let mut repair_w = crate::canonical::CanonicalWriter::new();
        for (i, rep) in self.graph_repair.iter().enumerate() {
            let mut rep_item = crate::canonical::CanonicalWriter::new();
            rep_item.write_u64(1, rep.doc_id.0);
            rep_item.write_bool(2, rep.verified_no_ghost_pointers);
            rep_item.write_u64(3, rep.attested_at as u64);
            repair_w.write_bytes((i + 1) as u16, rep_item.finish()?);
        }
        writer.write_bytes(7, repair_w.finish()?);

        if let Some(ref receipt) = self.wal_chain_receipt {
            writer.write_bytes32(8, receipt);
        }
        if let Some(pos) = self.audit_chain_position {
            writer.write_u64(9, pos);
        }
        if let Some(ref warning) = self.integrity_warning {
            writer.write_string(10, warning);
        }

        let canon_bytes = writer.finish()?;
        payload.extend_from_slice(&canon_bytes);
        Ok(payload)
    }
}
