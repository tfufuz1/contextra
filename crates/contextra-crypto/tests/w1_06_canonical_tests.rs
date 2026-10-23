#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

//! Tests for Stage 1: Canonical Encoding and Signature Payloads (w1_06_*).

#![forbid(unsafe_code)]

use contextra_crypto::canonical::{CanonicalReader, CanonicalWriter};
use contextra_crypto::deletion_proof::signer::ProofKeyPairSigner;
use contextra_crypto::deletion_proof::{
    ExcludedScope, ProofBuilder, ProofClaimV4,
};
use contextra_crypto::DeletionScope;
use contextra_types::{DocId, TenantId};
use rand::rngs::OsRng;

#[test]
fn w1_06_proof_canonical_golden() {
    let signer = ProofKeyPairSigner::generate(&mut OsRng);
    let tenant = TenantId::try_new(42).expect("tenant");

    // 1. Document scope
    let scope_doc = DeletionScope::Document {
        doc_id: DocId(101),
        tenant_id: tenant,
    };
    let proof_doc = ProofBuilder::with_required(
        scope_doc,
        "db-inst-1",
        [1u8; 32],
        [2u8; 32],
        10,
        1700000000000,
    )
    .add_claim(ProofClaimV4 {
        layer_id: 1,
        assurance: 2,
        claim_text_id: 3,
        witness_digest: [1u8; 32],
        scanned_at_tx: 10,
        evidence_tlv: vec![0x01],
    })
    .finish(&signer)
    .expect("proof doc");

    let bytes_doc = proof_doc.construct_canonical_payload();
    assert!(!bytes_doc.is_empty());

    // 2. Collection scope + empty excluded_scopes
    let scope_coll = DeletionScope::Collection {
        collection_id: contextra_types::CollectionId(202),
        tenant_id: tenant,
    };
    let proof_coll = ProofBuilder::with_required(
        scope_coll,
        "db-inst-1",
        [1u8; 32],
        [2u8; 32],
        20,
        1700000000000,
    )
    .finish(&signer)
    .expect("proof coll");

    let bytes_coll = proof_coll.construct_canonical_payload();
    assert!(!bytes_coll.is_empty());

    // 3. Tenant scope + non-empty excluded_scopes
    let scope_tenant = DeletionScope::Tenant { tenant_id: tenant };
    let proof_tenant = ProofBuilder::with_required(
        scope_tenant,
        "db-inst-1",
        [1u8; 32],
        [2u8; 32],
        30,
        1700000000000,
    )
    .add_excluded_scope(ExcludedScope::LlmParameterMemory.id())
    .add_excluded_scope(ExcludedScope::ConsolidatedAndDistilled.id())
    .finish(&signer)
    .expect("proof tenant");

    let bytes_tenant = proof_tenant.construct_canonical_payload();
    assert!(!bytes_tenant.is_empty());

    // Canonical bytes must be distinct across different scopes
    assert_ne!(bytes_doc, bytes_coll);
    assert_ne!(bytes_coll, bytes_tenant);
}

#[test]
fn w1_06_proof_canonical_rejects_duplicate_tag() {
    // 1. Writer duplicate tag rejection
    let mut writer = CanonicalWriter::new();
    writer.write_u32(0x0001, 100);
    writer.write_u32(0x0001, 200); // duplicate tag 0x0001
    let res = writer.finish();
    assert!(res.is_err());
    assert!(res.unwrap_err().to_string().contains("duplicate tag"));

    // 2. Reader non-canonical tag order / duplicate tag rejection
    let mut valid_writer = CanonicalWriter::new();
    valid_writer.write_u32(0x0001, 10);
    let valid_bytes = valid_writer.finish().expect("valid finish");

    // Manually construct duplicate bytes: tag=0x0001, len=4, val=10 | tag=0x0001, len=4, val=20
    let mut dup_bytes = valid_bytes.clone();
    dup_bytes.extend_from_slice(&valid_bytes);

    let mut reader = CanonicalReader::new(&dup_bytes);
    let read_res = reader.read_all(&[]);
    assert!(read_res.is_err());
    let err_msg = read_res.unwrap_err().to_string();
    assert!(err_msg.contains("duplicate tag") || err_msg.contains("non-canonical"));
}

#[test]
fn w1_06_proof_canonical_rejects_unknown_critical_tag() {
    let mut writer = CanonicalWriter::new();
    // Critical tag top bit 0x8000 set -> tag 0x8099 is critical
    writer.write_u32(0x8099, 12345);
    let bytes = writer.finish().expect("finish");

    // Pass known_critical_tags = [0x8001, 0x8002] (so 0x8099 is unknown)
    let known_tags = [0x8001, 0x8002];
    let mut reader = CanonicalReader::new(&bytes);
    let res = reader.read_all(&known_tags);

    assert!(res.is_err());
    assert!(res.unwrap_err().to_string().contains("unknown critical tag"));
}
