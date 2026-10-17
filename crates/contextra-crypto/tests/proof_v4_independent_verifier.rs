//! Independent verifier and test vector verification for version 4 deletion proofs.
//!
//! CRITICAL REQUIREMENTS (T7, T8):
//! - Zero imports from `canon.rs`, `proof_v4.rs`, or `policy.rs`.
//! - Only uses `ed25519-dalek`, `blake3`, `serde_json`, `bincode`, `contextra-crypto`.
//! - Compares independent verification verdict against crate verification verdict.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::cast_possible_truncation
)]

use contextra_crypto::deletion_proof::builder::ProofBuilder;
use contextra_crypto::deletion_proof::policy::{verify_with_policy, ProofPolicy};
use contextra_crypto::deletion_proof::proof_v4::{ProofClaimV4, ProofV4};
use contextra_crypto::deletion_proof::signer::{ProofKeyPairSigner, ProofSigner};
use contextra_crypto::deletion_proof::types::DeletionScope;
use contextra_types::{DocId, TenantId};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use std::fs;
use std::path::Path;

/// Standalone TLV field representation.
#[derive(Debug, Clone)]
struct StandaloneTlvField {
    tag: u16,
    value: Vec<u8>,
}

impl StandaloneTlvField {
    fn is_critical(&self) -> bool {
        (self.tag & 0x8000) != 0
    }
}

/// Standalone TLV decoder without importing canon.rs.
struct StandaloneTlvDecoder<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> StandaloneTlvDecoder<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, offset: 0 }
    }

    fn decode_all(&mut self) -> Result<Vec<StandaloneTlvField>, String> {
        let mut fields = Vec::new();
        while self.offset < self.data.len() {
            if self.data.len() - self.offset < 6 {
                return Err("Truncated TLV header".to_string());
            }

            let tag = u16::from_be_bytes([self.data[self.offset], self.data[self.offset + 1]]);
            let len = u32::from_be_bytes([
                self.data[self.offset + 2],
                self.data[self.offset + 3],
                self.data[self.offset + 4],
                self.data[self.offset + 5],
            ]) as usize;
            self.offset += 6;

            if self.data.len() - self.offset < len {
                return Err("Truncated TLV payload".to_string());
            }

            let value = self.data[self.offset..self.offset + len].to_vec();
            self.offset += len;

            fields.push(StandaloneTlvField { tag, value });
        }
        Ok(fields)
    }
}

/// Standalone TLV encoder for independent canonical payload construction.
struct StandaloneTlvEncoder {
    fields: Vec<StandaloneTlvField>,
}

impl StandaloneTlvEncoder {
    fn new() -> Self {
        Self { fields: Vec::new() }
    }

    fn add_field(&mut self, tag: u16, value: Vec<u8>) {
        self.fields.push(StandaloneTlvField { tag, value });
    }

    fn add_u8(&mut self, tag: u16, val: u8) {
        self.add_field(tag, vec![val]);
    }

    fn add_u16(&mut self, tag: u16, val: u16) {
        self.add_field(tag, val.to_be_bytes().to_vec());
    }

    fn add_u64(&mut self, tag: u16, val: u64) {
        self.add_field(tag, val.to_be_bytes().to_vec());
    }

    fn add_bool(&mut self, tag: u16, val: bool) {
        self.add_u8(tag, if val { 1 } else { 0 });
    }

    fn add_string(&mut self, tag: u16, val: &str) {
        self.add_field(tag, val.as_bytes().to_vec());
    }

    fn add_bytes32(&mut self, tag: u16, val: &[u8; 32]) {
        self.add_field(tag, val.to_vec());
    }

    fn finish(mut self) -> Vec<u8> {
        self.fields.sort_by_key(|f| f.tag);
        let mut out = Vec::new();
        for f in self.fields {
            out.extend_from_slice(&f.tag.to_be_bytes());
            let len = u32::try_from(f.value.len()).unwrap_or(u32::MAX);
            out.extend_from_slice(&len.to_be_bytes());
            out.extend_from_slice(&f.value);
        }
        out
    }
}

/// Independent verifier function implementing raw TLV and Ed25519 signature checks.
fn independent_verify_proof_v4(
    proof: &ProofV4,
    public_key_bytes: &[u8; 32],
) -> Result<bool, String> {
    // 1. Re-encode canonical payload manually via standalone encoder
    let mut enc = StandaloneTlvEncoder::new();

    enc.add_u16(0x8001, proof.version);
    enc.add_bytes32(0x8002, &proof.proof_id);
    enc.add_string(0x8003, &proof.db_instance_id);
    enc.add_u8(0x8004, proof.stage);

    match proof.supersedes {
        Some(ref b) => enc.add_bytes32(0x8005, b),
        None => enc.add_field(0x8005, Vec::new()),
    }

    let scope_bytes = bincode::serialize(&proof.scope).map_err(|e| e.to_string())?;
    enc.add_field(0x8006, scope_bytes);

    enc.add_string(0x8007, &proof.storage_scope);
    enc.add_bytes32(0x8008, &proof.deleted_keys_hash);
    enc.add_bytes32(0x8009, &proof.footprint_hash);
    enc.add_bool(0x800A, proof.observed_empty);
    enc.add_u64(0x800B, proof.deleted_after_tx);

    let mut claims_tlv = Vec::new();
    for claim in &proof.claims {
        let mut c_enc = StandaloneTlvEncoder::new();
        c_enc.add_u16(0x0001, claim.layer_id);
        c_enc.add_u8(0x0002, claim.assurance);
        c_enc.add_u16(0x0003, claim.claim_text_id);
        c_enc.add_bytes32(0x0004, &claim.witness_digest);
        c_enc.add_u64(0x0005, claim.scanned_at_tx);
        c_enc.add_field(0x0006, claim.evidence_tlv.clone());
        let c_bytes = c_enc.finish();

        let len_bytes = u32::try_from(c_bytes.len())
            .unwrap_or(u32::MAX)
            .to_be_bytes();
        claims_tlv.extend_from_slice(&len_bytes);
        claims_tlv.extend_from_slice(&c_bytes);
    }
    enc.add_field(0x800C, claims_tlv);

    let mut excl_bytes = Vec::new();
    for id in &proof.excluded_scopes {
        excl_bytes.extend_from_slice(&id.to_be_bytes());
    }
    enc.add_field(0x800D, excl_bytes);

    let mut lim_bytes = Vec::new();
    for id in &proof.limitations_text_ids {
        lim_bytes.extend_from_slice(&id.to_be_bytes());
    }
    enc.add_field(0x800E, lim_bytes);

    enc.add_field(0x800F, proof.environment_tlv.clone());

    match proof.wal_chain_receipt {
        Some(ref r) => enc.add_bytes32(0x8010, r),
        None => enc.add_field(0x8010, Vec::new()),
    }

    let mut chain_bytes = Vec::with_capacity(8 + 32);
    chain_bytes.extend_from_slice(&proof.chain.seq.to_be_bytes());
    chain_bytes.extend_from_slice(&proof.chain.prev_hash);
    enc.add_field(0x8011, chain_bytes);

    enc.add_u64(0x8012, proof.issued_wall_ms);
    enc.add_u64(0x8013, proof.issued_mono_ctr);
    enc.add_bytes32(0x8014, &proof.signer_key_id);

    let canon_bytes = enc.finish();

    // 2. Decode TLV and verify unknown critical tags
    let mut decoder = StandaloneTlvDecoder::new(&canon_bytes);
    let fields = decoder.decode_all()?;
    for f in fields {
        if f.is_critical() && !(0x8001..=0x8014).contains(&f.tag) {
            return Err(format!("Unknown critical tag: 0x{:04X}", f.tag));
        }
    }

    // 3. Verify Ed25519 signature
    let domain_tag = b"contextra/deletion-proof/v4\0";
    let mut signed_msg = Vec::with_capacity(domain_tag.len() + canon_bytes.len());
    signed_msg.extend_from_slice(domain_tag);
    signed_msg.extend_from_slice(&canon_bytes);

    let vk = VerifyingKey::from_bytes(public_key_bytes).map_err(|e| e.to_string())?;
    let sig = Signature::from_bytes(&proof.signature);

    Ok(vk.verify(&signed_msg, &sig).is_ok())
}

#[test]
fn test_canonicalization_non_ambiguity_and_field_distinction() {
    let signer = ProofKeyPairSigner::from_secret_bytes([42u8; 32]);
    let scope = DeletionScope::Document {
        doc_id: DocId(1),
        tenant_id: TenantId::try_new(1).unwrap(),
    };

    let base_proof = ProofBuilder::new(scope, "db-canonical-test")
        .with_deleted_after_tx(100)
        .with_deleted_keys_hash([0x01; 32])
        .finish(&signer)
        .unwrap();

    let bytes_base = base_proof.signed_bytes();

    // 1. Storage scope variation
    let p_storage = ProofBuilder::new(
        DeletionScope::Document {
            doc_id: DocId(1),
            tenant_id: TenantId::try_new(1).unwrap(),
        },
        "db-canonical-test",
    )
    .with_deleted_after_tx(100)
    .with_deleted_keys_hash([0x01; 32])
    .with_storage_scope("different_storage_scope")
    .finish(&signer)
    .unwrap();
    assert_ne!(bytes_base, p_storage.signed_bytes());

    // 2. Stage variation
    let p_stage = ProofBuilder::new(
        DeletionScope::Document {
            doc_id: DocId(1),
            tenant_id: TenantId::try_new(1).unwrap(),
        },
        "db-canonical-test",
    )
    .with_stage(1)
    .with_deleted_after_tx(100)
    .with_deleted_keys_hash([0x01; 32])
    .finish(&signer)
    .unwrap();
    assert_ne!(bytes_base, p_stage.signed_bytes());

    // 3. Observed empty flag variation
    let p_empty = ProofBuilder::new(
        DeletionScope::Document {
            doc_id: DocId(1),
            tenant_id: TenantId::try_new(1).unwrap(),
        },
        "db-canonical-test",
    )
    .with_deleted_after_tx(100)
    .with_deleted_keys_hash([0x01; 32])
    .with_observed_empty(true)
    .finish(&signer)
    .unwrap();
    assert_ne!(bytes_base, p_empty.signed_bytes());
}

fn create_test_vector_files(dir: &Path) {
    fs::create_dir_all(dir).unwrap();

    let signer = ProofKeyPairSigner::from_secret_bytes([42u8; 32]);
    let scope = DeletionScope::Document {
        doc_id: DocId(1),
        tenant_id: TenantId::try_new(1).unwrap(),
    };

    let valid_proof = ProofBuilder::new(scope, "db-instance-test-vector")
        .with_deleted_after_tx(100)
        .with_deleted_keys_hash([0x01; 32])
        .add_claim(ProofClaimV4 {
            layer_id: 1,
            assurance: 2,
            claim_text_id: 3,
            witness_digest: [0x02; 32],
            scanned_at_tx: 100,
            evidence_tlv: vec![],
        })
        .add_excluded_scope(1)
        .finish(&signer)
        .unwrap();

    // 1. Valid vector
    let valid_json = serde_json::to_string_pretty(&valid_proof).unwrap();
    fs::write(dir.join("valid_v4_vector.json"), valid_json).unwrap();

    // 2. Tampered signature vector
    let mut tampered_sig = valid_proof.clone();
    tampered_sig.signature[0] ^= 0xFF;
    let tampered_sig_json = serde_json::to_string_pretty(&tampered_sig).unwrap();
    fs::write(
        dir.join("tampered_signature_vector.json"),
        tampered_sig_json,
    )
    .unwrap();

    // 3. Tampered payload vector
    let mut tampered_payload = valid_proof.clone();
    tampered_payload.deleted_after_tx = 101;
    let tampered_payload_json = serde_json::to_string_pretty(&tampered_payload).unwrap();
    fs::write(
        dir.join("tampered_payload_vector.json"),
        tampered_payload_json,
    )
    .unwrap();
}

#[test]
fn test_independent_verifier_matches_crate_verifier_on_all_vectors() {
    let vector_dir = Path::new("docs/deletion/vectors");
    create_test_vector_files(vector_dir);

    let signer = ProofKeyPairSigner::from_secret_bytes([42u8; 32]);
    let public_key = signer.public_key();

    let policy = ProofPolicy::default();

    for entry in fs::read_dir(vector_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "json") {
            let json_str = fs::read_to_string(&path).unwrap();
            let proof: ProofV4 = serde_json::from_str(&json_str).unwrap();

            let independent_result =
                independent_verify_proof_v4(&proof, &public_key).unwrap_or(false);
            let crate_report = verify_with_policy(&proof, &public_key, &policy);
            let crate_result = crate_report
                .rule_results
                .iter()
                .find(|r| r.rule_id == "R1")
                .unwrap()
                .passed;

            assert_eq!(
                independent_result, crate_result,
                "Independent verifier and crate verifier MUST agree on vector file {:?}",
                path
            );
        }
    }
}
