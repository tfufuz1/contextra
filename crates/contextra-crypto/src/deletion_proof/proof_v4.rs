//! Version 4 Proof Format structures, canonical payload generation, and signing (C7, ADR-13).

use super::canon::{TlvDecoder, TlvEncoder};
use super::signer::ProofSigner;
use super::types::DeletionScope;
use crate::error::{CryptoError, Result};
use ed25519_dalek::{Signature, VerifyingKey};
use serde::{Deserialize, Serialize};

/// Domain tag prefix for version 4 deletion proof signature input.
pub const PROOF_V4_DOMAIN_TAG: &[u8] = b"contextra/deletion-proof/v4\0";

/// Domain tag prefix for version 1 proof chain link hashing.
pub const PROOF_CHAIN_DOMAIN_TAG: &[u8] = b"contextra/proof-chain/v1";

/// Individual layer claim in a version 4 deletion proof.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProofClaimV4 {
    /// Layer ID according to Contract V1 (1..7, 20..26).
    pub layer_id: u16,
    /// Assurance level (0..=4 S0..S4).
    pub assurance: u8,
    /// Claim text ID (1 = CT-S0, ..., 8 = CT-RAM-Z).
    pub claim_text_id: u16,
    /// Digest of the layer witness or scan proof ([u8; 32]).
    pub witness_digest: [u8; 32],
    /// TxId at which the layer scan occurred.
    pub scanned_at_tx: u64,
    /// Opaque canonical TLV evidence bytes.
    pub evidence_tlv: Vec<u8>,
}

impl ProofClaimV4 {
    /// Encodes this claim to canonical TLV bytes.
    pub fn to_canonical_tlv(&self) -> Vec<u8> {
        let mut enc = TlvEncoder::new();
        enc.add_u16(0x0001, self.layer_id);
        enc.add_u8(0x0002, self.assurance);
        enc.add_u16(0x0003, self.claim_text_id);
        enc.add_bytes32(0x0004, &self.witness_digest);
        enc.add_u64(0x0005, self.scanned_at_tx);
        enc.add_field(0x0006, self.evidence_tlv.clone());
        enc.finish()
    }
}

/// Chain link tracking sequence and previous chain hash.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProofChainLink {
    /// Monotonic sequence number in the proof chain log.
    pub seq: u64,
    /// BLAKE3 chain hash of the previous proof in the log.
    pub prev_hash: [u8; 32],
}

/// Version 4 Deletion Proof structure (C7, ADR-13).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProofV4 {
    /// Proof format version (must be 4).
    pub version: u16,
    /// Unique proof ID ([u8; 32]).
    pub proof_id: [u8; 32],
    /// Target database instance identifier.
    pub db_instance_id: String,
    /// Proof stage (e.g. 1 = Interim, 2 = Final).
    pub stage: u8,
    /// Proof ID superseded by this proof, if any.
    pub supersedes: Option<[u8; 32]>,
    /// Deletion scope.
    pub scope: DeletionScope,
    /// Human-readable or canonical storage scope description.
    pub storage_scope: String,
    /// Length-prefixed Blake3 hash of deleted keys.
    pub deleted_keys_hash: [u8; 32],
    /// Footprint hash of affected storage bounds.
    pub footprint_hash: [u8; 32],
    /// Flag indicating whether the scope was confirmed empty without key deletion.
    pub observed_empty: bool,
    /// TxId after which physical deletion was completed.
    pub deleted_after_tx: u64,
    /// Per-layer physical sanitization claims.
    pub claims: Vec<ProofClaimV4>,
    /// Explicitly excluded scopes.
    pub excluded_scopes: Vec<u16>,
    /// Limitations text IDs.
    pub limitations_text_ids: Vec<u16>,
    /// Opaque environment TLV metadata.
    pub environment_tlv: Vec<u8>,
    /// WAL chain receipt, if available.
    pub wal_chain_receipt: Option<[u8; 32]>,
    /// Chain link metadata.
    pub chain: ProofChainLink,
    /// Wall clock issuance timestamp in milliseconds since Unix epoch.
    pub issued_wall_ms: u64,
    /// Monotonic issuance counter.
    pub issued_mono_ctr: u64,
    /// Key ID (BLAKE3 hash of public key) of the signer.
    pub signer_key_id: [u8; 32],
    /// Ed25519 signature over signed_bytes (64 bytes).
    #[serde(with = "serde_64_bytes")]
    pub signature: [u8; 64],
}

mod serde_64_bytes {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(bytes: &[u8; 64], serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(bytes)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> std::result::Result<[u8; 64], D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes: Vec<u8> = Deserialize::deserialize(deserializer)?;
        bytes
            .try_into()
            .map_err(|_| serde::de::Error::custom("expected 64 bytes for signature"))
    }
}

/// Encodes a DeletionScope to canonical TLV format with salted BLAKE3 hashes for IDs.
pub fn encode_scope_tlv(scope: &DeletionScope, proof_id: &[u8; 32]) -> Result<Vec<u8>> {
    let salt = *blake3::keyed_hash(b"contextra/deletion-proof/v4/salt", proof_id).as_bytes();
    let mut enc = TlvEncoder::new();

    // Tag 0x0001: Salt (32 bytes)
    enc.add_bytes32(0x0001, &salt);

    match scope {
        DeletionScope::Document { doc_id, tenant_id } => {
            // Tag 0x0002: Scope type 1 (Document)
            enc.add_u8(0x0002, 1);
            // Tag 0x0003: Salted Tenant ID Hash
            let tenant_bytes = tenant_id.as_u64().to_be_bytes();
            let tenant_hash = *blake3::keyed_hash(&salt, &tenant_bytes).as_bytes();
            enc.add_bytes32(0x0003, &tenant_hash);
            // Tag 0x0004: Salted Document ID Hash
            let doc_bytes = doc_id.0.to_be_bytes();
            let id_hash = *blake3::keyed_hash(&salt, &doc_bytes).as_bytes();
            enc.add_bytes32(0x0004, &id_hash);
        }
        DeletionScope::Collection {
            collection_id,
            tenant_id,
        } => {
            // Tag 0x0002: Scope type 2 (Collection)
            enc.add_u8(0x0002, 2);
            // Tag 0x0003: Salted Tenant ID Hash
            let tenant_bytes = tenant_id.as_u64().to_be_bytes();
            let tenant_hash = *blake3::keyed_hash(&salt, &tenant_bytes).as_bytes();
            enc.add_bytes32(0x0003, &tenant_hash);
            // Tag 0x0004: Salted Collection ID Hash
            let coll_bytes = collection_id.0.to_be_bytes();
            let id_hash = *blake3::keyed_hash(&salt, &coll_bytes).as_bytes();
            enc.add_bytes32(0x0004, &id_hash);
        }
        DeletionScope::Tenant { tenant_id } => {
            // Tag 0x0002: Scope type 3 (Tenant)
            enc.add_u8(0x0002, 3);
            // Tag 0x0003: Salted Tenant ID Hash
            let tenant_bytes = tenant_id.as_u64().to_be_bytes();
            let tenant_hash = *blake3::keyed_hash(&salt, &tenant_bytes).as_bytes();
            enc.add_bytes32(0x0003, &tenant_hash);
        }
    }

    Ok(enc.finish())
}

impl super::proof::DeletionProof {
    /// Erstellt und signiert einen DeletionProof (Version 4, Ed25519) mit allen 7-Schichten-Anforderungen.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn create_v4_internal(
        scope: DeletionScope,
        mut deleted_keys: Vec<Vec<u8>>,
        deleted_after_tx: contextra_types::TxId,
        covered_layers: Vec<super::layer_proof::LayerCleanupProof>,
        excluded_scopes: Vec<super::types::ExcludedScope>,
        wal_chain_receipt: Option<[u8; 32]>,
        audit_chain_position: Option<u64>,
        attested_at: i64,
        graph_repair: &[super::types::GraphRepairAttestation],
        durability: Option<&super::types::DurabilityProof>,
        signing_key: &ed25519_dalek::SigningKey,
    ) -> contextra_types::Result<Self> {
        let durability_proof = durability
            .ok_or_else(|| contextra_types::ContextraError::from(CryptoError::NotDurable))?;
        if durability_proof.confirmed_seq_no < deleted_after_tx.0 {
            return Err(CryptoError::NotDurable.into());
        }
        deleted_keys.sort();
        let deleted_keys_hash = super::types::hash_deleted_keys_length_prefixed(&deleted_keys);

        let covered_layers: Vec<super::types::DeletionLayer> =
            covered_layers.into_iter().map(|p| p.layer).collect();

        if covered_layers.contains(&super::types::DeletionLayer::HnswIndex)
            && graph_repair.is_empty()
        {
            return Err(contextra_types::ContextraError::GraphRepairFailed(
                contextra_types::error::HnswDeletionError::VerificationFailed {
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
            signature_version: crate::ed25519_proof::SignatureVersion::V4.as_u8(),
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
            .map_err(|e| contextra_types::ContextraError::Internal(e.to_string()))?;

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
}

impl ProofV4 {
    /// Fallible canonical TLV payload construction without bincode or unwrap_or_default.
    pub fn try_construct_canonical_payload(&self) -> Result<Vec<u8>> {
        let mut enc = TlvEncoder::new();

        enc.add_u16(0x8001, self.version);
        enc.add_bytes32(0x8002, &self.proof_id);
        enc.add_string(0x8003, &self.db_instance_id);
        enc.add_u8(0x8004, self.stage);
        enc.add_option_bytes32(0x8005, self.supersedes.as_ref());

        let scope_bytes = encode_scope_tlv(&self.scope, &self.proof_id)?;
        enc.add_field(0x8006, scope_bytes);

        enc.add_string(0x8007, &self.storage_scope);
        enc.add_bytes32(0x8008, &self.deleted_keys_hash);
        enc.add_bytes32(0x8009, &self.footprint_hash);
        enc.add_bool(0x800A, self.observed_empty);
        enc.add_u64(0x800B, self.deleted_after_tx);

        let mut claims_tlv = Vec::new();
        for claim in &self.claims {
            let claim_bytes = claim.to_canonical_tlv();
            let len = u32::try_from(claim_bytes.len())
                .map_err(|_| CryptoError::Crypto("Claim TLV length overflow".to_string()))?;
            claims_tlv.extend_from_slice(&len.to_be_bytes());
            claims_tlv.extend_from_slice(&claim_bytes);
        }
        enc.add_field(0x800C, claims_tlv);

        let mut excl_bytes = Vec::new();
        for id in &self.excluded_scopes {
            excl_bytes.extend_from_slice(&id.to_be_bytes());
        }
        enc.add_field(0x800D, excl_bytes);

        let mut lim_bytes = Vec::new();
        for id in &self.limitations_text_ids {
            lim_bytes.extend_from_slice(&id.to_be_bytes());
        }
        enc.add_field(0x800E, lim_bytes);

        enc.add_field(0x800F, self.environment_tlv.clone());
        enc.add_option_bytes32(0x8010, self.wal_chain_receipt.as_ref());

        let mut chain_bytes = Vec::with_capacity(8 + 32);
        chain_bytes.extend_from_slice(&self.chain.seq.to_be_bytes());
        chain_bytes.extend_from_slice(&self.chain.prev_hash);
        enc.add_field(0x8011, chain_bytes);

        enc.add_u64(0x8012, self.issued_wall_ms);
        enc.add_u64(0x8013, self.issued_mono_ctr);
        enc.add_bytes32(0x8014, &self.signer_key_id);

        Ok(enc.finish())
    }

    /// Constructs the canonical TLV encoding of all payload fields.
    pub fn construct_canonical_payload(&self) -> Vec<u8> {
        match self.try_construct_canonical_payload() {
            Ok(bytes) => bytes,
            Err(_) => Vec::new(),
        }
    }

    /// Fallible signed bytes construction: PROOF_V4_DOMAIN_TAG || CANONICAL_TLV.
    pub fn try_signed_bytes(&self) -> Result<Vec<u8>> {
        let canon = self.try_construct_canonical_payload()?;
        let mut buf = Vec::with_capacity(PROOF_V4_DOMAIN_TAG.len() + canon.len());
        buf.extend_from_slice(PROOF_V4_DOMAIN_TAG);
        buf.extend_from_slice(&canon);
        Ok(buf)
    }

    /// Computes the exact signed bytes input: PROOF_V4_DOMAIN_TAG || CANONICAL_TLV.
    pub fn signed_bytes(&self) -> Vec<u8> {
        match self.try_signed_bytes() {
            Ok(bytes) => bytes,
            Err(_) => Vec::new(),
        }
    }

    /// Signs this proof using the provided `ProofSigner`, updating `signer_key_id` and `signature`.
    pub fn sign_with_signer(&mut self, signer: &dyn ProofSigner) -> Result<()> {
        self.signer_key_id = signer.key_id();
        let msg = self.try_signed_bytes()?;
        self.signature = signer.sign(&msg)?;
        Ok(())
    }

    /// Computes the chain hash for entry n in the proof chain log:
    /// `BLAKE3(PROOF_CHAIN_DOMAIN_TAG || prev_chain_hash || BLAKE3(signed_bytes || signature))`
    pub fn compute_chain_hash(&self, prev_chain_hash: &[u8; 32]) -> [u8; 32] {
        let signed_msg = self.signed_bytes();
        let mut inner_hasher = blake3::Hasher::new();
        inner_hasher.update(&signed_msg);
        inner_hasher.update(&self.signature);
        let inner_digest = inner_hasher.finalize();

        let mut outer_hasher = blake3::Hasher::new();
        outer_hasher.update(PROOF_CHAIN_DOMAIN_TAG);
        outer_hasher.update(prev_chain_hash);
        outer_hasher.update(inner_digest.as_bytes());
        *outer_hasher.finalize().as_bytes()
    }

    /// Verifies the Ed25519 signature of this proof using `public_key` with strict verification.
    pub fn verify_signature(&self, public_key: &[u8; 32]) -> Result<bool> {
        let expected_key_id = *blake3::hash(public_key).as_bytes();
        if self.signer_key_id != expected_key_id {
            return Ok(false);
        }

        let vk = VerifyingKey::from_bytes(public_key)
            .map_err(|e| CryptoError::Crypto(format!("Invalid public key: {e}")))?;

        let msg = self.try_signed_bytes()?;
        let sig = Signature::from_bytes(&self.signature);

        Ok(vk.verify_strict(&msg, &sig).is_ok())
    }

    /// Verifies that no unknown critical tag (0x8000 bit set) exists in `raw_canonical_tlv`.
    pub fn check_unknown_critical_tags(raw_canonical_tlv: &[u8]) -> Result<()> {
        let mut decoder = TlvDecoder::new(raw_canonical_tlv);
        let fields = decoder.decode_all()?;
        for field in fields {
            if field.is_critical() && !(0x8001..=0x8014).contains(&field.tag) {
                return Err(CryptoError::Crypto(format!(
                    "Unknown critical TLV tag encountered: 0x{:04X}",
                    field.tag
                )));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::super::canon::TlvField;
    use super::*;
    use crate::deletion_proof::signer::ProofKeyPairSigner;
    use contextra_types::{DocId, TenantId};
    use rand::rngs::OsRng;

    fn test_proof_v4() -> ProofV4 {
        let signer = ProofKeyPairSigner::generate(&mut OsRng);
        ProofV4 {
            version: 4,
            proof_id: [1u8; 32],
            db_instance_id: "db-instance-test".to_string(),
            stage: 2,
            supersedes: None,
            scope: DeletionScope::Document {
                doc_id: DocId(42),
                tenant_id: TenantId::try_new(1).expect("tenant"),
            },
            storage_scope: "doc:42".to_string(),
            deleted_keys_hash: [2u8; 32],
            footprint_hash: [3u8; 32],
            observed_empty: false,
            deleted_after_tx: 100,
            claims: vec![ProofClaimV4 {
                layer_id: 1,
                assurance: 1,
                claim_text_id: 2,
                witness_digest: [4u8; 32],
                scanned_at_tx: 100,
                evidence_tlv: vec![0x10, 0x20],
            }],
            excluded_scopes: vec![1, 2],
            limitations_text_ids: vec![],
            environment_tlv: vec![],
            wal_chain_receipt: Some([5u8; 32]),
            chain: ProofChainLink {
                seq: 1,
                prev_hash: [0u8; 32],
            },
            issued_wall_ms: 1700000000000,
            issued_mono_ctr: 10,
            signer_key_id: signer.key_id(),
            signature: [0u8; 64],
        }
    }

    #[test]
    fn test_proof_v4_sign_and_verify_roundtrip() {
        let signer = ProofKeyPairSigner::generate(&mut OsRng);
        let mut proof = test_proof_v4();

        proof.sign_with_signer(&signer).expect("sign failed");

        assert!(proof
            .verify_signature(&signer.public_key())
            .expect("verify failed"));

        let wrong_signer = ProofKeyPairSigner::generate(&mut OsRng);
        assert!(!proof
            .verify_signature(&wrong_signer.public_key())
            .expect("verify failed"));
    }

    #[test]
    fn test_single_field_mutation_changes_signed_bytes() {
        let p1 = test_proof_v4();
        let bytes1 = p1.signed_bytes();

        let mut p2 = p1.clone();
        p2.stage = 1;
        let bytes2 = p2.signed_bytes();

        assert_ne!(bytes1, bytes2, "Mutation of stage MUST change signed_bytes");

        let mut p3 = p1.clone();
        p3.deleted_after_tx = 101;
        let bytes3 = p3.signed_bytes();

        assert_ne!(
            bytes1, bytes3,
            "Mutation of deleted_after_tx MUST change signed_bytes"
        );
    }

    #[test]
    fn test_unknown_critical_tag_rejected() {
        let proof = test_proof_v4();
        let mut canon = proof.construct_canonical_payload();

        // Inject unknown critical tag 0x9999 into payload stream
        let unknown_critical_field = TlvField::new(0x9999, vec![0x01, 0x02]);
        unknown_critical_field.encode(&mut canon);

        let res = ProofV4::check_unknown_critical_tags(&canon);
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("0x9999"));
    }
}
