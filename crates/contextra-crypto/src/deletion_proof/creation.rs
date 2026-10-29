// FILE-CONTEXT
// ZWECK: Generation logic for cryptographic deletion proofs (v1, v2, v3, v5, WAL receipts).
// INVARIANTEN: INV-DELETION-1: Proof creation requires verified layer sanitization witnesses.

#![forbid(unsafe_code)]

use crate::deletion_proof::types::*;
use crate::ed25519_proof::SignatureVersion;
use crate::error::CryptoError;
use contextra_types::{ContextraError, Result, TxId};

impl DeletionProof {
    /// Issues a new `DeletionProof` based on verified `LayerWitness` measurements.
    #[allow(clippy::too_many_arguments)]
    pub fn issue<'a>(
        ctx: &contextra_types::RequestContext,
        saga: &contextra_types::SagaRecord,
        witnesses: &[contextra_ports::scanner::LayerWitness],
        excluded: &[ExcludedScope],
        custody: CustodyLevel,
        key: impl Into<SigningKeyHandle<'a>>,
        clock: &dyn contextra_ports::Clock,
    ) -> Result<Self> {
        for w in witnesses {
            if w.remaining() > 0 {
                return Err(ContextraError::Internal(format!(
                    "INV-WITNESS-MINT-1 violation: layer '{}' has {} remaining live entries",
                    w.layer(),
                    w.remaining()
                )));
            }
            match w.method() {
                contextra_types::ScanMethod::Structural
                | contextra_types::ScanMethod::FeatureDisabled => {}
                contextra_types::ScanMethod::NotMeasured(reason) => {
                    return Err(ContextraError::Internal(format!(
                        "INV-WITNESS-MINT-1 violation: layer '{}' was not measured ({})",
                        w.layer(),
                        reason
                    )));
                }
            }
        }

        if witnesses.is_empty() {
            return Err(ContextraError::Internal(
                "REQ-PRF-1 violation: witnesses list cannot be empty for deletion proof issuance"
                    .to_string(),
            ));
        }

        if excluded.is_empty() {
            return Err(ContextraError::Internal(
                "REQ-PRF-2 violation: excluded scopes list cannot be empty".to_string(),
            ));
        }

        use crate::deletion_proof::LayerWitnessExt;

        let mut covered_layers = Vec::new();
        let mut measurements = Vec::new();
        let mut seen_layers = std::collections::HashSet::new();

        for w in witnesses {
            if w.remaining() > 0 {
                return Err(ContextraError::Internal(format!(
                    "INV-WITNESS-MINT-1 violation: layer '{}' has {} remaining live entries",
                    w.layer(),
                    w.remaining()
                )));
            }

            match w.method() {
                contextra_types::ScanMethod::Structural
                | contextra_types::ScanMethod::FeatureDisabled => {}
                contextra_types::ScanMethod::NotMeasured(reason) => {
                    return Err(ContextraError::Internal(format!(
                        "INV-WITNESS-MINT-1 violation: layer '{}' was not measured ({})",
                        w.layer(),
                        reason
                    )));
                }
            }

            if let contextra_ports::scanner::ScanTechnique::NotMeasured(reason) = w.technique() {
                return Err(ContextraError::Internal(format!(
                    "INV-WITNESS-MINT-1 violation: layer '{}' was not measured ({})",
                    w.layer(),
                    reason
                )));
            }

            let witness_layer = w.witness_layer().clone();

            if !seen_layers.insert(witness_layer.clone()) {
                return Err(ContextraError::Internal(format!(
                    "INV-WITNESS-MINT-1 violation: duplicate witness layer '{:?}' in witnesses list",
                    witness_layer
                )));
            }

            let mapped_layer = map_witness_layer_to_deletion_layer(&witness_layer, w)?;

            covered_layers.push(mapped_layer.clone());

            measurements.push(LayerWitnessMeasurement {
                layer: mapped_layer,
                technique: w.technique().clone(),
                scanned: w.scanned(),
                remaining: w.remaining(),
                control_matches: w.control_matches(),
                digest: *w.digest(),
            });
        }

        let has_memtable = covered_layers.iter().any(|l| matches!(l, DeletionLayer::LsmMemtable));
        let has_sstable = covered_layers.iter().any(|l| matches!(l, DeletionLayer::SsTableAllLevels));
        let has_wal = covered_layers.iter().any(|l| matches!(l, DeletionLayer::WalAllSegments { .. }));

        if !has_memtable {
            return Err(ContextraError::Internal(
                "INV-WITNESS-MINT-1 violation: mandatory layer LsmMemtable missing from witnesses"
                    .to_string(),
            ));
        }
        if !has_sstable {
            return Err(ContextraError::Internal(
                "INV-WITNESS-MINT-1 violation: mandatory layer SsTableAllLevels missing from witnesses"
                    .to_string(),
            ));
        }
        if !has_wal {
            return Err(ContextraError::Internal(
                "INV-WITNESS-MINT-1 violation: mandatory layer WalAllSegments missing from witnesses"
                    .to_string(),
            ));
        }

        let mut uncovered_layers = Vec::new();
        if !covered_layers.iter().any(|l| matches!(l, DeletionLayer::HnswIndex)) {
            uncovered_layers.push(UncoveredLayer {
                layer: DeletionLayer::HnswIndex,
                reason: "Scope does not contain vector HnswIndex data".to_string(),
            });
        }
        if !covered_layers.iter().any(|l| matches!(l, DeletionLayer::TextIndex)) {
            uncovered_layers.push(UncoveredLayer {
                layer: DeletionLayer::TextIndex,
                reason: "Scope does not contain full-text TextIndex data".to_string(),
            });
        }

        let scope = DeletionScope::Tenant {
            tenant_id: ctx.tenant_id,
        };
        let timestamp = clock.now_unix_nanos() / 1_000_000_000;

        let proof_stub = Self {
            signature_version: SignatureVersion::V5.as_u8(),
            scope,
            deleted_keys_hash: saga.deleted_keys_hash,
            deleted_after_tx: ctx.tx_id,
            timestamp,
            signature: Vec::new(),
            covered_layers,
            excluded_scopes: excluded.to_vec(),
            graph_repair: Vec::new(),
            wal_chain_receipt: None,
            audit_chain_position: None,
            custody,
            measurements,
            uncovered_layers,
            integrity_warning: None,
        };

        let payload = proof_stub
            .construct_v5_payload()
            .map_err(|e| ContextraError::Internal(e.to_string()))?;

        let mut proof = proof_stub;

        match key.into() {
            SigningKeyHandle::Ed25519(sk) => {
                use ed25519_dalek::Signer;
                let sig = sk.sign(&payload);
                proof.signature = sig.to_bytes().to_vec();
            }
            SigningKeyHandle::Hmac(hmac_key) => {
                let sig = compute_hmac_sha256(hmac_key, &[&payload])?;
                proof.signature = sig.to_vec();
            }
        }

        Ok(proof)
    }

    /// Erstellt und signiert einen DeletionProof nach Layer-Bereinigung.
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
        deleted_keys.sort();

        let deleted_keys_hash = hash_deleted_keys_length_prefixed(&deleted_keys);

        let scope_bytes =
            bincode::serialize(&scope).map_err(|e| ContextraError::Internal(e.to_string()))?;
        let tx_bytes = deleted_after_tx.0.to_le_bytes();

        let covered_layers: Vec<DeletionLayer> =
            covered_layers.into_iter().map(|p| p.layer).collect();

        let covered_layers_bytes = bincode::serialize(&covered_layers)
            .map_err(|e| ContextraError::Internal(e.to_string()))?;
        let excluded_scopes_bytes = bincode::serialize(&excluded_scopes)
            .map_err(|e| ContextraError::Internal(e.to_string()))?;

        let receipt_bytes = wal_chain_receipt.unwrap_or([0u8; 32]);
        let receipt_part = if wal_chain_receipt.is_some() {
            receipt_bytes.as_slice()
        } else {
            &[]
        };

        let signature = compute_hmac_sha256(
            proof_key,
            &[
                &scope_bytes,
                &deleted_keys_hash,
                &tx_bytes,
                &covered_layers_bytes,
                &excluded_scopes_bytes,
                receipt_part,
            ],
        )?;

        Ok(Self {
            signature_version: SignatureVersion::V2.as_u8(),
            scope,
            deleted_keys_hash,
            deleted_after_tx,
            timestamp: 0,
            signature: signature.to_vec(),
            covered_layers,
            excluded_scopes,
            graph_repair: Vec::new(),
            wal_chain_receipt,
            audit_chain_position: None,
            custody: CustodyLevel::Local,
            measurements: Vec::new(),
            uncovered_layers: Vec::new(),
            integrity_warning: None,
        })
    }

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
        Self::create_with_wal_receipt_v3(
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
                contextra_types::error::HnswDeletionError::VerificationFailed {
                    remaining_pointers: 1,
                },
            ));
        }

        let timestamp = if attested_at >= 0 {
            attested_at as u64
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
            custody: CustodyLevel::Local,
            measurements: Vec::new(),
            uncovered_layers: Vec::new(),
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

    /// Helper to construct the signed payload for signature version 5.
    pub(crate) fn construct_v5_payload(&self) -> std::result::Result<Vec<u8>, CryptoError> {
        let version_bytes = self.signature_version.to_le_bytes();
        let scope_bytes =
            bincode::serialize(&self.scope).map_err(|e| CryptoError::Crypto(e.to_string()))?;
        let tx_bytes = self.deleted_after_tx.0.to_le_bytes();
        let timestamp_bytes = self.timestamp.to_le_bytes();
        let custody_bytes =
            bincode::serialize(&self.custody).map_err(|e| CryptoError::Crypto(e.to_string()))?;
        let covered_layers_bytes = bincode::serialize(&self.covered_layers)
            .map_err(|e| CryptoError::Crypto(e.to_string()))?;
        let measurements_bytes = bincode::serialize(&self.measurements)
            .map_err(|e| CryptoError::Crypto(e.to_string()))?;
        let uncovered_bytes = bincode::serialize(&self.uncovered_layers)
            .map_err(|e| CryptoError::Crypto(e.to_string()))?;
        let excluded_scopes_bytes = bincode::serialize(&self.excluded_scopes)
            .map_err(|e| CryptoError::Crypto(e.to_string()))?;

        let mut payload = Vec::with_capacity(
            version_bytes.len()
                + scope_bytes.len()
                + 32
                + tx_bytes.len()
                + timestamp_bytes.len()
                + custody_bytes.len()
                + covered_layers_bytes.len()
                + measurements_bytes.len()
                + uncovered_bytes.len()
                + excluded_scopes_bytes.len(),
        );

        payload.extend_from_slice(&version_bytes);
        payload.extend_from_slice(&scope_bytes);
        payload.extend_from_slice(&self.deleted_keys_hash);
        payload.extend_from_slice(&tx_bytes);
        payload.extend_from_slice(&timestamp_bytes);
        payload.extend_from_slice(&custody_bytes);
        payload.extend_from_slice(&covered_layers_bytes);
        payload.extend_from_slice(&measurements_bytes);
        payload.extend_from_slice(&uncovered_bytes);
        payload.extend_from_slice(&excluded_scopes_bytes);

        Ok(payload)
    }

    /// Helper to construct the signed payload for signature version 3 (Ed25519).
    pub(crate) fn construct_v3_payload(&self) -> std::result::Result<Vec<u8>, CryptoError> {
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

/// Erzeugt eine WAL-Löschquittung H(hmac_prev || delete_event) für den Nachweis
/// auf WAL-Ebene gemäß DSGVO Art. 17.
pub fn compute_wal_delete_receipt(
    prev_hmac: &[u8; 32],
    delete_event_payload: &[u8],
    integrity_key: &[u8],
) -> Result<[u8; 32]> {
    compute_hmac_sha256(
        integrity_key,
        &[b"contextra-wal-delete-v1", prev_hmac, delete_event_payload],
    )
}

pub(crate) fn compute_hmac_sha256(key: &[u8], data_parts: &[&[u8]]) -> Result<[u8; 32]> {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    let mut mac = Hmac::<Sha256>::new_from_slice(key)
        .map_err(|e| ContextraError::Internal(format!("HMAC key error: {e}")))?;
    for part in data_parts {
        mac.update(part);
    }
    Ok(mac.finalize().into_bytes().into())
}
