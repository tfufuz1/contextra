//! Version 3 Ed25519 deletion proof creation logic.

use super::layer_proof::LayerCleanupProof;
use super::proof::DeletionProof;
use super::types::{
    hash_deleted_keys_length_prefixed, DeletionLayer, DeletionScope, DurabilityProof,
    ExcludedScope, GraphRepairAttestation,
};
use crate::ed25519_proof::SignatureVersion;
use crate::error::CryptoError;
use contextra_types::{error::HnswDeletionError, ContextraError, Result, TxId};

impl DeletionProof {
    /// Erstellt und signiert einen DeletionProof (Version 3) mit Ed25519 nach Layer-Bereinigung.
    ///
    /// # Invarianten (INV-DELETION-1 / INV-DELETION-2)
    /// `DeletionProof::create()` und verbundene Methoden dürfen ERST NACH der vollständigen
    /// physischen Bereinigung der jeweiligen Storage-Schichten aufgerufen werden. Die Schichten
    /// werden durch verifizierte [`LayerCleanupProof`]-Instanzen belegt.
    ///
    /// # Domain Separation & Legacy-Hinweis (`legacy_v4_tag`)
    /// V3-Payloads nutzen den dedizierten Domain-Tag `contextra/deletion-proof/v3\0`.
    /// Beweise, die früher fälschlich mit dem V4-Tag (`contextra/deletion-proof/v4\0`)
    /// signiert wurden, gelten per Sicherheitskontrakt F7 als nicht vertrauenswürdig und ungültig.
    #[cfg_attr(
        not(test),
        deprecated(
            note = "Creation of v3 deletion proofs is forbidden per §20 and ADR-13. Use DeletionProofBuilder for v4 deletion proofs."
        )
    )]
    pub fn create(
        scope: DeletionScope,
        deleted_keys: Vec<Vec<u8>>,
        deleted_after_tx: TxId,
        covered_layers: Vec<LayerCleanupProof>,
        excluded_scopes: Vec<ExcludedScope>,
        proof_key: &[u8],
    ) -> Result<Self> {
        let durability = DurabilityProof::new(deleted_after_tx.0);
        #[allow(deprecated)]
        Self::create_with_durability(
            scope,
            deleted_keys,
            deleted_after_tx,
            covered_layers,
            excluded_scopes,
            Some(&durability),
            proof_key,
        )
    }

    /// Erstellt und signiert einen DeletionProof (Version 3) mit optionaler WAL-Löschquittung.
    ///
    /// # Invarianten (INV-DELETION-1 / INV-DELETION-2)
    /// Erfordert vollständige Bereinigung der zugrundeliegenden Schichten nachgewiesen durch `covered_layers`.
    ///
    /// # Legacy-Hinweis (`legacy_v4_tag`)
    /// V3-Beweise nutzen `contextra/deletion-proof/v3\0`. Mit dem alten Tag signierte Beweise gelten als nicht vertrauenswürdig.
    #[cfg_attr(
        not(test),
        deprecated(
            note = "Creation of v3 deletion proofs is forbidden per §20 and ADR-13. Use DeletionProofBuilder for v4 deletion proofs."
        )
    )]
    pub fn create_with_wal_receipt(
        scope: DeletionScope,
        deleted_keys: Vec<Vec<u8>>,
        deleted_after_tx: TxId,
        covered_layers: Vec<LayerCleanupProof>,
        excluded_scopes: Vec<ExcludedScope>,
        wal_chain_receipt: Option<[u8; 32]>,
        proof_key: &[u8],
    ) -> Result<Self> {
        let durability = DurabilityProof::new(deleted_after_tx.0);
        let signing_key = if proof_key.len() >= 32 {
            let mut key_bytes = [0u8; 32];
            key_bytes.copy_from_slice(&proof_key[..32]);
            ed25519_dalek::SigningKey::from_bytes(&key_bytes)
        } else {
            return Err(ContextraError::Internal(
                "proof_key must be at least 32 bytes for Ed25519 v3 signature".to_string(),
            ));
        };

        #[allow(deprecated)]
        Self::create_with_wal_receipt_v3(
            scope,
            deleted_keys,
            deleted_after_tx,
            covered_layers,
            excluded_scopes,
            wal_chain_receipt,
            0,
            &[],
            Some(&durability),
            &signing_key,
        )
    }

    /// Creates a DeletionProof with explicit DurabilityProof verification.
    ///
    /// Returns `Err(CryptoError::NotDurable)` if `durability_proof` is `None` (e.g. MemoryOnly path).
    ///
    /// # Invarianten (INV-DELETION-1 / INV-DELETION-2)
    /// Aufruf erst nach physischer Bereinigung und Verifikation über `covered_layers`.
    ///
    /// # Legacy-Hinweis (`legacy_v4_tag`)
    /// Beweise mit veraltetem V4-Tag auf V3-Payloads gelten als ungültig und unvertrauenswürdig.
    #[cfg_attr(
        not(test),
        deprecated(
            note = "Creation of v3 deletion proofs is forbidden per §20 and ADR-13. Use DeletionProofBuilder for v4 deletion proofs."
        )
    )]
    pub fn create_with_durability(
        scope: DeletionScope,
        deleted_keys: Vec<Vec<u8>>,
        deleted_after_tx: TxId,
        covered_layers: Vec<LayerCleanupProof>,
        excluded_scopes: Vec<ExcludedScope>,
        durability_proof: Option<&DurabilityProof>,
        proof_key: &[u8],
    ) -> Result<Self> {
        if durability_proof.is_none() {
            return Err(CryptoError::NotDurable.into());
        }

        let signing_key = if proof_key.len() >= 32 {
            let mut key_bytes = [0u8; 32];
            key_bytes.copy_from_slice(&proof_key[..32]);
            ed25519_dalek::SigningKey::from_bytes(&key_bytes)
        } else {
            return Err(ContextraError::Internal(
                "proof_key must be at least 32 bytes for Ed25519 v3 signature".to_string(),
            ));
        };

        #[allow(deprecated)]
        Self::create_v3(
            scope,
            deleted_keys,
            deleted_after_tx,
            covered_layers,
            excluded_scopes,
            0,
            &[],
            durability_proof,
            &signing_key,
        )
    }

    /// Erstellt und signiert einen DeletionProof (Version 3) mit Ed25519.
    ///
    /// # Invarianten (INV-DELETION-1 / INV-DELETION-2)
    /// `covered_layers` muss nachgewiesene Schichten nach physischer Bereinigung enthalten.
    ///
    /// # Legacy-Hinweis (`legacy_v4_tag`)
    /// Beweise, die mit dem alten Tag signiert wurden, gelten als nicht vertrauenswürdig.
    #[cfg_attr(
        not(test),
        deprecated(
            note = "Creation of v3 deletion proofs is forbidden per §20 and ADR-13. Use DeletionProofBuilder for v4 deletion proofs."
        )
    )]
    #[allow(clippy::too_many_arguments)]
    pub fn create_v3(
        scope: DeletionScope,
        deleted_keys: Vec<Vec<u8>>,
        deleted_after_tx: TxId,
        covered_layers: Vec<LayerCleanupProof>,
        excluded_scopes: Vec<ExcludedScope>,
        attested_at: i64,
        graph_repair: &[GraphRepairAttestation],
        durability: Option<&DurabilityProof>,
        signing_key: &ed25519_dalek::SigningKey,
    ) -> Result<Self> {
        #[allow(deprecated)]
        Self::create_v3_with_audit_position(
            scope,
            deleted_keys,
            deleted_after_tx,
            covered_layers,
            excluded_scopes,
            None,
            attested_at,
            graph_repair,
            durability,
            signing_key,
        )
    }

    /// Erstellt und signiert einen DeletionProof (Version 3, Ed25519) inklusive Verweis auf den Audit-Chain-Eintrag.
    #[cfg_attr(
        not(test),
        deprecated(
            note = "Creation of v3 deletion proofs is forbidden per §20 and ADR-13. Use DeletionProofBuilder for v4 deletion proofs."
        )
    )]
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
        durability: Option<&DurabilityProof>,
        signing_key: &ed25519_dalek::SigningKey,
    ) -> Result<Self> {
        #[allow(deprecated)]
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
            durability,
            signing_key,
        )
    }

    /// Erstellt und signiert einen DeletionProof (Version 3, Ed25519) inklusive optionaler WAL-HMAC-Kettenquittung.
    #[cfg_attr(
        not(test),
        deprecated(
            note = "Creation of v3 deletion proofs is forbidden per §20 and ADR-13. Use DeletionProofBuilder for v4 deletion proofs."
        )
    )]
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
        durability: Option<&DurabilityProof>,
        signing_key: &ed25519_dalek::SigningKey,
    ) -> Result<Self> {
        #[allow(deprecated)]
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
            durability,
            signing_key,
        )
    }

    /// Erstellt und signiert einen DeletionProof (Version 3, Ed25519) mit allen optionalen Erweiterungen.
    ///
    /// # Invarianten (INV-DELETION-1 / INV-DELETION-2)
    /// `covered_layers` stellt verifizierte Layer-Claims nach physischer Bereinigung sicher.
    ///
    /// # Legacy-Hinweis (`legacy_v4_tag`)
    /// V3-Payloads nutzen `contextra/deletion-proof/v3\0`. Altbestand mit `contextra/deletion-proof/v4\0` gilt als unvertrauenswürdig.
    #[cfg_attr(
        not(test),
        deprecated(
            note = "Creation of v3 deletion proofs is forbidden per §20 and ADR-13. Use DeletionProofBuilder for v4 deletion proofs."
        )
    )]
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
        durability: Option<&DurabilityProof>,
        signing_key: &ed25519_dalek::SigningKey,
    ) -> Result<Self> {
        let durability_proof =
            durability.ok_or_else(|| ContextraError::from(CryptoError::NotDurable))?;
        if durability_proof.confirmed_seq_no < deleted_after_tx.0 {
            return Err(CryptoError::NotDurable.into());
        }
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
            uncovered_layers: Vec::new(),
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

    /// Erstellt und signiert einen DeletionProof (Version 4, Ed25519) mit allen 7-Schichten-Anforderungen.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn create_v4_internal(
        scope: DeletionScope,
        mut deleted_keys: Vec<Vec<u8>>,
        deleted_after_tx: TxId,
        covered_layers: Vec<LayerCleanupProof>,
        excluded_scopes: Vec<ExcludedScope>,
        wal_chain_receipt: Option<[u8; 32]>,
        audit_chain_position: Option<u64>,
        attested_at: i64,
        graph_repair: &[GraphRepairAttestation],
        durability: Option<&DurabilityProof>,
        signing_key: &ed25519_dalek::SigningKey,
    ) -> Result<Self> {
        let durability_proof =
            durability.ok_or_else(|| ContextraError::from(CryptoError::NotDurable))?;
        if durability_proof.confirmed_seq_no < deleted_after_tx.0 {
            return Err(CryptoError::NotDurable.into());
        }
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
            signature_version: SignatureVersion::V4.as_u8(),
            scope,
            deleted_keys_hash,
            deleted_after_tx,
            timestamp,
            signature: Vec::new(),
            covered_layers,
            uncovered_layers: Vec::new(),
            excluded_scopes,
            graph_repair: graph_repair.to_vec(),
            wal_chain_receipt,
            audit_chain_position,
            integrity_warning: None,
        };

        let payload = proof_stub
            .construct_v4_payload()
            .map_err(|e| ContextraError::Internal(e.to_string()))?;

        use ed25519_dalek::Signer;
        let sig = signing_key.sign(&payload);

        let mut proof = proof_stub;
        proof.signature = sig.to_bytes().to_vec();

        Ok(proof)
    }

    /// Helper to construct the signed payload for signature version 4 (Ed25519, 7-layer coverage).
    ///
    /// # Domain Separation
    /// Prefixes payload strictly with `contextra/deletion-proof/v4\0`.
    pub(super) fn construct_v4_payload(&self) -> std::result::Result<Vec<u8>, CryptoError> {
        let mut payload = Vec::with_capacity(128);
        payload.extend_from_slice(b"contextra/deletion-proof/v4\0");
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

    /// Helper to construct the signed payload for signature version 3 (Ed25519).
    ///
    /// # Domain Separation
    /// Prefixes payload strictly with domain tag `contextra/deletion-proof/v3\0`.
    ///
    /// # Legacy Note (`legacy_v4_tag`)
    /// Beweise, die früher mit dem v4-Tag signiert wurden, gelten per F7-Regel als nicht vertrauenswürdig.
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
    use super::*;
    use contextra_types::{DocId, TenantId};

    fn sample_proof_stub(version: u8) -> DeletionProof {
        DeletionProof {
            signature_version: version,
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
        let proof = sample_proof_stub(SignatureVersion::V3.as_u8());
        let payload = proof.construct_v3_payload().unwrap();
        assert!(payload.starts_with(b"contextra/deletion-proof/v3\0"));
    }

    #[test]
    fn test_v4_payload_domain_tag_prefix() {
        let proof = sample_proof_stub(SignatureVersion::V4.as_u8());
        let payload = proof.construct_v4_payload().unwrap();
        assert!(payload.starts_with(b"contextra/deletion-proof/v4\0"));
    }

    #[test]
    fn test_v3_and_v4_payload_prefixes_are_distinct() {
        let proof_v3_stub = sample_proof_stub(SignatureVersion::V3.as_u8());
        let proof_v4_stub = sample_proof_stub(SignatureVersion::V4.as_u8());

        let payload_v3 = proof_v3_stub.construct_v3_payload().unwrap();
        let payload_v4 = proof_v4_stub.construct_v4_payload().unwrap();

        assert_ne!(payload_v3, payload_v4);
        assert_eq!(&payload_v3[..28], b"contextra/deletion-proof/v3\0");
        assert_eq!(&payload_v4[..28], b"contextra/deletion-proof/v4\0");
    }

    #[test]
    fn test_payload_optional_fields_matching_without_defaults() {
        let mut proof = sample_proof_stub(3);
        let payload_none = proof.construct_v3_payload().unwrap();

        proof.wal_chain_receipt = Some([9u8; 32]);
        proof.audit_chain_position = Some(12345);
        let payload_some = proof.construct_v3_payload().unwrap();

        assert_ne!(payload_none, payload_some);
        assert!(payload_some.len() > payload_none.len());
    }
}
