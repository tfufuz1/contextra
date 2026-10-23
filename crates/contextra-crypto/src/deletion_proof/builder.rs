//! Builder for constructing version 4 deletion proofs (ADR-13).

use super::proof_v4::{ProofChainLink, ProofClaimV4, ProofV4};
use super::signer::ProofSigner;
use super::types::DeletionScope;
use crate::error::Result;

/// Fluent builder for constructing version 4 deletion proofs.
pub struct ProofBuilder {
    proof_id: Option<[u8; 32]>,
    db_instance_id: String,
    stage: u8,
    supersedes: Option<[u8; 32]>,
    scope: DeletionScope,
    storage_scope: String,
    deleted_keys_hash: [u8; 32],
    footprint_hash: [u8; 32],
    observed_empty: bool,
    deleted_after_tx: u64,
    claims: Vec<ProofClaimV4>,
    excluded_scopes: Vec<u16>,
    limitations_text_ids: Vec<u16>,
    environment_tlv: Vec<u8>,
    wal_chain_receipt: Option<[u8; 32]>,
    chain_seq: u64,
    prev_chain_hash: [u8; 32],
    issued_wall_ms: u64,
    issued_mono_ctr: u64,
}

impl ProofBuilder {
    /// Creates a new builder initialized with all mandatory parameters.
    pub fn with_required(
        scope: DeletionScope,
        db_instance_id: impl Into<String>,
        deleted_keys_hash: [u8; 32],
        footprint_hash: [u8; 32],
        deleted_after_tx: u64,
        issued_wall_ms: u64,
    ) -> Self {
        Self {
            proof_id: None,
            db_instance_id: db_instance_id.into(),
            stage: 2, // Default Final
            supersedes: None,
            scope,
            storage_scope: String::new(),
            deleted_keys_hash,
            footprint_hash,
            observed_empty: false,
            deleted_after_tx,
            claims: Vec::new(),
            excluded_scopes: Vec::new(),
            limitations_text_ids: Vec::new(),
            environment_tlv: Vec::new(),
            wal_chain_receipt: None,
            chain_seq: 1,
            prev_chain_hash: [0u8; 32],
            issued_wall_ms,
            issued_mono_ctr: 1,
        }
    }

    /// Creates a new builder initialized with scope and db instance ID.
    #[deprecated(note = "ProofBuilder::new is deprecated. Use ProofBuilder::with_required instead.")]
    pub fn new(scope: DeletionScope, db_instance_id: impl Into<String>) -> Self {
        Self {
            proof_id: None,
            db_instance_id: db_instance_id.into(),
            stage: 2, // Default Final
            supersedes: None,
            scope,
            storage_scope: String::new(),
            deleted_keys_hash: [0u8; 32],
            footprint_hash: [0u8; 32],
            observed_empty: false,
            deleted_after_tx: 0,
            claims: Vec::new(),
            excluded_scopes: Vec::new(),
            limitations_text_ids: Vec::new(),
            environment_tlv: Vec::new(),
            wal_chain_receipt: None,
            chain_seq: 1,
            prev_chain_hash: [0u8; 32],
            issued_wall_ms: 1,
            issued_mono_ctr: 1,
        }
    }

    /// Sets explicit stage (1 = Interim, 2 = Final).
    pub fn with_stage(mut self, stage: u8) -> Self {
        self.stage = stage;
        self
    }

    /// Sets supersedes proof ID reference.
    pub fn with_supersedes(mut self, supersedes: [u8; 32]) -> Self {
        self.supersedes = Some(supersedes);
        self
    }

    /// Sets storage scope description string.
    pub fn with_storage_scope(mut self, storage_scope: impl Into<String>) -> Self {
        self.storage_scope = storage_scope.into();
        self
    }

    /// Sets length-prefixed BLAKE3 hash of deleted keys.
    pub fn with_deleted_keys_hash(mut self, deleted_keys_hash: [u8; 32]) -> Self {
        self.deleted_keys_hash = deleted_keys_hash;
        self
    }

    /// Sets footprint hash.
    pub fn with_footprint_hash(mut self, footprint_hash: [u8; 32]) -> Self {
        self.footprint_hash = footprint_hash;
        self
    }

    /// Sets observed_empty flag.
    pub fn with_observed_empty(mut self, observed_empty: bool) -> Self {
        self.observed_empty = observed_empty;
        self
    }

    /// Sets deleted_after_tx sequence number.
    pub fn with_deleted_after_tx(mut self, tx: u64) -> Self {
        self.deleted_after_tx = tx;
        self
    }

    /// Adds a layer sanitization claim.
    pub fn add_claim(mut self, claim: ProofClaimV4) -> Self {
        self.claims.push(claim);
        self
    }

    /// Adds an excluded scope ID.
    pub fn add_excluded_scope(mut self, scope_id: u16) -> Self {
        self.excluded_scopes.push(scope_id);
        self
    }

    /// Sets opaque environment TLV metadata bytes.
    pub fn with_environment_tlv(mut self, environment_tlv: Vec<u8>) -> Self {
        self.environment_tlv = environment_tlv;
        self
    }

    /// Sets WAL chain receipt.
    pub fn with_wal_chain_receipt(mut self, receipt: [u8; 32]) -> Self {
        self.wal_chain_receipt = Some(receipt);
        self
    }

    /// Sets chain sequence and previous chain hash.
    pub fn with_chain_link(mut self, seq: u64, prev_hash: [u8; 32]) -> Self {
        self.chain_seq = seq;
        self.prev_chain_hash = prev_hash;
        self
    }

    /// Sets wall millisecond timestamp and monotonic counter.
    pub fn with_timestamps(mut self, wall_ms: u64, mono_ctr: u64) -> Self {
        self.issued_wall_ms = wall_ms;
        self.issued_mono_ctr = mono_ctr;
        self
    }

    /// Validates builder internal consistency, sorts/deduplicates claims and excluded scopes, and signs the proof.
    pub fn finish_checked(mut self, signer: &dyn ProofSigner) -> Result<ProofV4> {
        if self.issued_wall_ms == 0 {
            return Err(crate::error::CryptoError::Crypto(
                "ProofBuilder: issued_wall_ms (timestamp) must be non-zero".to_string(),
            ));
        }

        if self.chain_seq < 1 {
            return Err(crate::error::CryptoError::Crypto(
                "ProofBuilder: chain_seq must be >= 1".to_string(),
            ));
        }

        // Sort claims by layer_id and deduplicate duplicate layer IDs
        self.claims.sort_by_key(|c| c.layer_id);
        self.claims.dedup_by_key(|c| c.layer_id);

        // Sort and deduplicate excluded_scopes
        self.excluded_scopes.sort();
        self.excluded_scopes.dedup();

        let mut proof_id = self.proof_id.unwrap_or([0u8; 32]);
        if proof_id == [0u8; 32] {
            let mut hasher = blake3::Hasher::new();
            hasher.update(self.db_instance_id.as_bytes());
            hasher.update(&[self.stage]);
            hasher.update(&self.deleted_after_tx.to_be_bytes());
            hasher.update(&self.deleted_keys_hash);
            proof_id = *hasher.finalize().as_bytes();
        }

        let mut proof = ProofV4 {
            version: 4,
            proof_id,
            db_instance_id: self.db_instance_id,
            stage: self.stage,
            supersedes: self.supersedes,
            scope: self.scope,
            storage_scope: self.storage_scope,
            deleted_keys_hash: self.deleted_keys_hash,
            footprint_hash: self.footprint_hash,
            observed_empty: self.observed_empty,
            deleted_after_tx: self.deleted_after_tx,
            claims: self.claims,
            excluded_scopes: self.excluded_scopes,
            limitations_text_ids: self.limitations_text_ids,
            environment_tlv: self.environment_tlv,
            wal_chain_receipt: self.wal_chain_receipt,
            chain: ProofChainLink {
                seq: self.chain_seq,
                prev_hash: self.prev_chain_hash,
            },
            issued_wall_ms: self.issued_wall_ms,
            issued_mono_ctr: self.issued_mono_ctr,
            signer_key_id: signer.key_id(),
            signature: [0u8; 64],
        };

        proof.sign_with_signer(signer)?;
        Ok(proof)
    }

    /// Finishes building and signs the version 4 deletion proof.
    pub fn finish(self, signer: &dyn ProofSigner) -> Result<ProofV4> {
        self.finish_checked(signer)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use crate::deletion_proof::signer::ProofKeyPairSigner;
    use contextra_types::{DocId, TenantId};
    use rand::rngs::OsRng;

    #[test]
    fn test_proof_builder_finishes_v4_only() {
        let signer = ProofKeyPairSigner::generate(&mut OsRng);
        let scope = DeletionScope::Document {
            doc_id: DocId(101),
            tenant_id: TenantId::try_new(1).expect("tenant"),
        };

        let proof = ProofBuilder::with_required(scope, "db-test", [0xAA; 32], [0u8; 32], 50, 1)
            .finish(&signer)
            .expect("finish failed");

        assert_eq!(proof.version, 4);
        assert_eq!(proof.db_instance_id, "db-test");
        assert_eq!(proof.deleted_after_tx, 50);
        assert_eq!(proof.deleted_keys_hash, [0xAA; 32]);
        assert!(proof
            .verify_signature(&signer.public_key())
            .expect("verify failed"));
    }
}
