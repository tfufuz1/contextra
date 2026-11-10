#![allow(
    deprecated,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

#[allow(unused_imports)]
use super::*;
use crate::types_local::SystemClock;

#[cfg(test)]
fn mint_witness_for_tests(
    layer: &str,
    scanned: u64,
    remaining: u64,
    tech: ScanTechnique,
) -> LayerWitness {
    use crate::types_local::{ScanSession, ScanTarget};

    let target = ScanTarget::new("target", [0x11; 32]);
    let mut session = ScanSession::begin(WitnessLayer::from(layer), target, tech.clone());
    if !matches!(tech, ScanTechnique::FeatureDisabled) {
        session.record_control_match();
        for i in 0..scanned {
            session.record_unit([(i % 255) as u8; 32], i < remaining);
        }
    }
    let clock = SystemClock::new();
    session.finish(&clock).unwrap()
}

#[allow(unused_imports)]
use super::*;
use crate::types_local::{DocId, TenantId, TxId};

fn test_key() -> Vec<u8> {
    vec![0u8; 32]
}

#[test]
fn test_deletion_proof_issue_validation() {
    use crate::types_local::{RequestContext, SagaRecord};

    let clock = SystemClock::new();
    let ctx = RequestContext::new(TenantId::new(1), TxId(100));
    let saga = SagaRecord::new([0xAA; 32]);
    let hmac_key = b"secret_hmac_key_32_bytes_long!!";

    // 1. witnesses empty -> Err
    let res_empty = DeletionProof::issue(
        &ctx,
        &saga,
        &[],
        &[ExcludedScope::LlmParameterMemory],
        CustodyLevel::Local,
        hmac_key.as_slice(),
        &clock,
    );
    assert!(res_empty.is_err(), "empty witnesses MUST return Err");

    // 2. excluded empty -> Err
    let w_valid = vec![
        mint_witness_for_tests("LsmMemtable", 10, 0, ScanTechnique::Structural),
        mint_witness_for_tests("SsTableAllLevels", 50, 0, ScanTechnique::Structural),
        mint_witness_for_tests("WalAllSegments", 100, 0, ScanTechnique::Structural),
    ];
    let res_no_excluded = DeletionProof::issue(
        &ctx,
        &saga,
        &w_valid,
        &[],
        CustodyLevel::Local,
        hmac_key.as_slice(),
        &clock,
    );
    assert!(res_no_excluded.is_err(), "empty excluded MUST return Err");

    // 3. remaining > 0 -> Err
    let w_remaining = vec![mint_witness_for_tests(
        "LsmMemtable",
        10,
        2,
        ScanTechnique::Structural,
    )];
    let res1 = DeletionProof::issue(
        &ctx,
        &saga,
        &w_remaining,
        &[ExcludedScope::LlmParameterMemory],
        CustodyLevel::Local,
        hmac_key.as_slice(),
        &clock,
    );
    assert!(res1.is_err(), "remaining > 0 MUST return Err");

    // 4. ScanMethod::NotMeasured -> Err
    let w_not_measured = vec![mint_witness_for_tests(
        "LsmMemtable",
        0,
        0,
        ScanTechnique::NotMeasured("unmounted".to_string()),
    )];
    let res2 = DeletionProof::issue(
        &ctx,
        &saga,
        &w_not_measured,
        &[ExcludedScope::LlmParameterMemory],
        CustodyLevel::Local,
        hmac_key.as_slice(),
        &clock,
    );
    assert!(res2.is_err(), "NotMeasured MUST return Err");

    // 5. Missing mandatory layer (SsTableAllLevels missing) -> Err
    let w_missing_mandatory = vec![
        mint_witness_for_tests("LsmMemtable", 10, 0, ScanTechnique::Structural),
        mint_witness_for_tests("WalAllSegments", 100, 0, ScanTechnique::Structural),
    ];
    let res_missing = DeletionProof::issue(
        &ctx,
        &saga,
        &w_missing_mandatory,
        &[ExcludedScope::LlmParameterMemory],
        CustodyLevel::Local,
        hmac_key.as_slice(),
        &clock,
    );
    assert!(
        res_missing.is_err(),
        "missing mandatory SsTableAllLevels MUST return Err"
    );

    // 6. Duplicate layer -> Err
    let w_duplicate = vec![
        mint_witness_for_tests("LsmMemtable", 10, 0, ScanTechnique::Structural),
        mint_witness_for_tests("LsmMemtable", 10, 0, ScanTechnique::Structural),
        mint_witness_for_tests("SsTableAllLevels", 50, 0, ScanTechnique::Structural),
        mint_witness_for_tests("WalAllSegments", 100, 0, ScanTechnique::Structural),
    ];
    let res_dup = DeletionProof::issue(
        &ctx,
        &saga,
        &w_duplicate,
        &[ExcludedScope::LlmParameterMemory],
        CustodyLevel::Local,
        hmac_key.as_slice(),
        &clock,
    );
    assert!(res_dup.is_err(), "duplicate layer MUST return Err");

    // 7. Satisfied conditions -> Ok(V5)
    let w_valid_full = vec![
        mint_witness_for_tests("LsmMemtable", 10, 0, ScanTechnique::Structural),
        mint_witness_for_tests("SsTableAllLevels", 50, 0, ScanTechnique::Structural),
        mint_witness_for_tests("WalAllSegments", 100, 0, ScanTechnique::Structural),
        mint_witness_for_tests("HnswIndex", 0, 0, ScanTechnique::FeatureDisabled),
    ];
    let res3 = DeletionProof::issue(
        &ctx,
        &saga,
        &w_valid_full,
        &[ExcludedScope::LlmParameterMemory],
        CustodyLevel::Local,
        hmac_key.as_slice(),
        &clock,
    );
    assert!(
        res3.is_ok(),
        "Satisfied witnesses MUST produce Ok(DeletionProof), got {:?}",
        res3.err()
    );
    let proof = res3.unwrap();
    assert_eq!(proof.signature_version, 5);
    assert_eq!(proof.covered_layers.len(), 4);
    assert!(proof.verify(hmac_key.as_slice()).unwrap());
}

#[test]
fn test_v5_golden_json_serde_and_tamper_rejection() {
    use crate::types_local::{RequestContext, SagaRecord};

    let clock = SystemClock::new();
    let ctx = RequestContext::new(TenantId::new(42), TxId(500));
    let saga = SagaRecord::new([0x33; 32]);
    let keypair = DeletionProofKeyPair::generate();

    let witnesses = vec![
        mint_witness_for_tests("LsmMemtable", 100, 0, ScanTechnique::Structural),
        mint_witness_for_tests("SsTableAllLevels", 500, 0, ScanTechnique::Structural),
        mint_witness_for_tests("WalAllSegments", 1000, 0, ScanTechnique::Structural),
    ];

    let proof = DeletionProof::issue(
        &ctx,
        &saga,
        &witnesses,
        &[
            ExcludedScope::LlmParameterMemory,
            ExcludedScope::ConsolidatedAndDistilled,
        ],
        CustodyLevel::Sovereign,
        keypair.signing_key(),
        &clock,
    )
    .expect("V5 issue must succeed");

    assert_eq!(proof.signature_version, 5);
    assert!(proof.verify(&keypair.verifying_key).unwrap());
    assert!(proof.verify_external(&keypair.verifying_key).is_ok());

    // Serde roundtrip
    let json = serde_json::to_string_pretty(&proof).unwrap();
    assert!(json.contains(r#""signature_version": 5"#));
    assert!(json.contains(r#""custody": "Sovereign""#));
    let de_proof: DeletionProof = serde_json::from_str(&json).unwrap();
    assert_eq!(de_proof, proof);

    // Tamper test: flip signature byte -> verify fails
    let mut tampered_sig = proof.clone();
    tampered_sig.signature[0] ^= 0xFF;
    assert!(!tampered_sig.verify(&keypair.verifying_key).unwrap());
    assert!(tampered_sig
        .verify_external(&keypair.verifying_key)
        .is_err());

    // Tamper test: flip measurement digest -> verify fails
    let mut tampered_digest = proof.clone();
    tampered_digest.measurements[0].digest[0] ^= 0xFF;
    assert!(!tampered_digest.verify(&keypair.verifying_key).unwrap());
    assert!(tampered_digest
        .verify_external(&keypair.verifying_key)
        .is_err());
}

#[test]
fn test_hash_collision_ab_c_vs_a_bc() {
    let keys1 = vec![b"ab".to_vec(), b"c".to_vec()];
    let keys2 = vec![b"a".to_vec(), b"bc".to_vec()];

    let hash1 = hash_deleted_keys_length_prefixed(&keys1);
    let hash2 = hash_deleted_keys_length_prefixed(&keys2);

    assert_ne!(
        hash1, hash2,
        "hash_deleted_keys_length_prefixed MUST produce distinct Blake3 hashes for [\"ab\", \"c\"] vs [\"a\", \"bc\"]"
    );

    // Verify exact encoding bytes fed into hasher:
    // keys1: 4-byte LE len (2) + "ab" + 4-byte LE len (1) + "c"
    let mut expected_bytes1 = Vec::new();
    expected_bytes1.extend_from_slice(&(2u32).to_le_bytes());
    expected_bytes1.extend_from_slice(b"ab");
    expected_bytes1.extend_from_slice(&(1u32).to_le_bytes());
    expected_bytes1.extend_from_slice(b"c");
    let expected_hash1 = *blake3::hash(&expected_bytes1).as_bytes();

    assert_eq!(
        hash1, expected_hash1,
        "hash_deleted_keys_length_prefixed MUST match 4-byte LE length-prefixed Blake3 calculation"
    );
}

#[test]
fn test_v3_create_and_verify() {
    let keypair = DeletionProofKeyPair::generate();
    let scope = DeletionScope::Document {
        doc_id: DocId(42),
        tenant_id: TenantId::new(1),
    };
    let proof = DeletionProof::create_v3(
        scope,
        vec![b"k1".to_vec(), b"k2".to_vec()],
        TxId(100),
        vec![LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0).unwrap()],
        vec![ExcludedScope::LlmParameterMemory],
        1700000000,
        &[],
        keypair.signing_key(),
    )
    .unwrap();

    assert_eq!(proof.signature_version, 3);
    assert_eq!(proof.signature.len(), 64);
    assert!(proof.verify(&keypair.verifying_key).unwrap());
}

#[test]
fn test_v3_wrong_verifying_key_rejects() {
    let keypair1 = DeletionProofKeyPair::generate();
    let keypair2 = DeletionProofKeyPair::generate();

    let scope = DeletionScope::Tenant {
        tenant_id: TenantId::new(1),
    };
    let proof = DeletionProof::create_v3(
        scope,
        vec![b"k1".to_vec()],
        TxId(10),
        vec![],
        vec![],
        1700000000,
        &[],
        keypair1.signing_key(),
    )
    .unwrap();

    assert!(!proof.verify(&keypair2.verifying_key).unwrap());
}

#[test]
fn test_v3_tampered_signature_rejects() {
    let keypair = DeletionProofKeyPair::generate();
    let scope = DeletionScope::Tenant {
        tenant_id: TenantId::new(1),
    };
    let mut proof = DeletionProof::create_v3(
        scope,
        vec![b"k1".to_vec()],
        TxId(10),
        vec![],
        vec![],
        1700000000,
        &[],
        keypair.signing_key(),
    )
    .unwrap();

    proof.signature[0] ^= 0xFF;
    assert!(!proof.verify(&keypair.verifying_key).unwrap());
}

#[test]
fn test_v3_tampered_payload_rejects() {
    let keypair = DeletionProofKeyPair::generate();
    let scope = DeletionScope::Document {
        doc_id: DocId(42),
        tenant_id: TenantId::new(1),
    };
    let proof = DeletionProof::create_with_wal_receipt_v3(
        scope,
        vec![b"k1".to_vec()],
        TxId(10),
        vec![LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0).unwrap()],
        vec![ExcludedScope::LlmParameterMemory],
        Some([0xABu8; 32]),
        1700000000,
        &[],
        keypair.signing_key(),
    )
    .unwrap();

    assert!(proof.verify(&keypair.verifying_key).unwrap());

    // Scope
    let mut tampered = proof.clone();
    tampered.scope = DeletionScope::Document {
        doc_id: DocId(43),
        tenant_id: TenantId::new(1),
    };
    assert!(!tampered.verify(&keypair.verifying_key).unwrap());

    // Keys hash
    let mut tampered = proof.clone();
    tampered.deleted_keys_hash[0] ^= 0xFF;
    assert!(!tampered.verify(&keypair.verifying_key).unwrap());

    // TxId
    let mut tampered = proof.clone();
    tampered.deleted_after_tx = TxId(11);
    assert!(!tampered.verify(&keypair.verifying_key).unwrap());

    // Covered layers
    let mut tampered = proof.clone();
    tampered.covered_layers.push(DeletionLayer::HnswIndex);
    assert!(!tampered.verify(&keypair.verifying_key).unwrap());

    // Excluded scopes
    let mut tampered = proof.clone();
    tampered.excluded_scopes.clear();
    assert!(!tampered.verify(&keypair.verifying_key).unwrap());

    // WAL receipt
    let mut tampered = proof.clone();
    tampered.wal_chain_receipt = Some([0xCDu8; 32]);
    assert!(!tampered.verify(&keypair.verifying_key).unwrap());
}

#[test]
fn test_v3_cannot_verify_with_hmac_key() {
    let keypair = DeletionProofKeyPair::generate();
    let scope = DeletionScope::Tenant {
        tenant_id: TenantId::new(1),
    };
    let proof = DeletionProof::create_v3(
        scope,
        vec![b"k1".to_vec()],
        TxId(10),
        vec![],
        vec![],
        1700000000,
        &[],
        keypair.signing_key(),
    )
    .unwrap();

    let hmac_key = vec![0u8; 32];
    let res = proof.verify(&hmac_key);
    assert!(res.is_err());
    match res {
        Err(ErasureError::Internal(msg)) => {
            assert!(msg.contains("HMAC key provided for Ed25519 signature_version 3 proof"));
        }
        _ => panic!("Expected ErasureError::Internal"),
    }
}

#[test]
fn test_v1_v2_still_verify_after_v3_code() {
    // Run existing v1 & v2 tests logic to ensure zero regression
    let scope = DeletionScope::Tenant {
        tenant_id: TenantId::new(1),
    };

    // v2
    let proof_v2 = DeletionProof::create(
        scope.clone(),
        vec![b"k1".to_vec()],
        TxId(10),
        vec![LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0).unwrap()],
        vec![ExcludedScope::LlmParameterMemory],
        &test_key(),
    )
    .unwrap();
    assert_eq!(proof_v2.signature_version, 2);
    assert!(proof_v2.verify(&test_key()).unwrap());

    // v1
    let scope_bytes = bincode::serialize(&scope).unwrap();
    let deleted_keys_hash = [0u8; 32];
    let tx_bytes = TxId(10).0.to_le_bytes();
    let v1_signature =
        compute_hmac_sha256(&test_key(), &[&scope_bytes, &deleted_keys_hash, &tx_bytes]).unwrap();

    let v1_proof = DeletionProof {
        signature_version: 1,
        scope,
        deleted_keys_hash,
        deleted_after_tx: TxId(10),
        timestamp: 0,
        signature: v1_signature.to_vec(),
        covered_layers: vec![],
        excluded_scopes: vec![],
        graph_repair: vec![],
        wal_chain_receipt: None,
        audit_chain_position: None,
        custody: CustodyLevel::Local,
        measurements: vec![],
        uncovered_layers: vec![],
        integrity_warning: None,
    };
    assert_eq!(v1_proof.signature_version, 1);
    assert!(v1_proof.verify(&test_key()).unwrap());
}

#[test]
fn test_v3_uses_length_prefixed_hash() {
    let keypair = DeletionProofKeyPair::generate();
    let scope = DeletionScope::Tenant {
        tenant_id: TenantId::new(1),
    };

    let proof_ab_c = DeletionProof::create_v3(
        scope.clone(),
        vec![b"ab".to_vec(), b"c".to_vec()],
        TxId(10),
        vec![],
        vec![],
        1700000000,
        &[],
        keypair.signing_key(),
    )
    .unwrap();

    let proof_a_bc = DeletionProof::create_v3(
        scope,
        vec![b"a".to_vec(), b"bc".to_vec()],
        TxId(10),
        vec![],
        vec![],
        1700000000,
        &[],
        keypair.signing_key(),
    )
    .unwrap();

    assert_ne!(proof_ab_c.deleted_keys_hash, proof_a_bc.deleted_keys_hash);
}

#[test]
fn test_deletion_proof_create_and_verify() {
    let tenant = TenantId::new(1);
    let scope = DeletionScope::Document {
        doc_id: DocId(42),
        tenant_id: tenant,
    };
    let keys = vec![b"key1".to_vec(), b"key2".to_vec()];
    let proof = DeletionProof::create(
        scope,
        keys,
        TxId(100),
        vec![
            LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0).unwrap(),
            LayerCleanupProof::new_after_verified_empty(DeletionLayer::HnswIndex, 0).unwrap(),
        ],
        vec![ExcludedScope::LlmParameterMemory],
        &test_key(),
    )
    .unwrap();

    assert!(proof.verify(&test_key()).unwrap());
}

#[test]
fn test_layer_cleanup_proof_rejects_nonzero_remaining_entries() {
    let res = LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 3);
    assert!(res.is_err());
    match res {
        Err(ErasureError::Internal(msg)) => {
            assert!(msg.contains("INV-DELETION-1 violation"));
            assert!(msg.contains("found 3 remaining live entries"));
        }
        _ => panic!("Expected ErasureError::Internal"),
    }
}

#[test]
fn test_deletion_proof_wrong_key_fails_verify() {
    let scope = DeletionScope::Tenant {
        tenant_id: TenantId::new(1),
    };
    let proof = DeletionProof::create(scope, vec![], TxId(1), vec![], vec![], &test_key()).unwrap();

    let wrong_key = vec![1u8; 32];
    assert!(!proof.verify(&wrong_key).unwrap());
}

#[test]
fn test_deletion_proof_tampered_data_fails_verify() {
    let scope = DeletionScope::Tenant {
        tenant_id: TenantId::new(1),
    };
    let proof = DeletionProof::create(
        scope,
        vec![b"k1".to_vec()],
        TxId(10),
        vec![LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0).unwrap()],
        vec![ExcludedScope::LlmParameterMemory],
        &test_key(),
    )
    .unwrap();

    // Tamper with deleted_after_tx
    let mut tampered_tx = proof.clone();
    tampered_tx.deleted_after_tx = TxId(11);
    assert!(!tampered_tx.verify(&test_key()).unwrap());

    // Tamper with deleted_keys_hash
    let mut tampered_hash = proof.clone();
    tampered_hash.deleted_keys_hash[0] ^= 0xFF;
    assert!(!tampered_hash.verify(&test_key()).unwrap());

    // Tamper with scope
    let mut tampered_scope = proof.clone();
    tampered_scope.scope = DeletionScope::Tenant {
        tenant_id: TenantId::new(2),
    };
    assert!(!tampered_scope.verify(&test_key()).unwrap());

    // Untampered original must verify successfully
    assert!(proof.verify(&test_key()).unwrap());
}

#[test]
fn test_deletion_proof_key_order_deterministic() {
    // Gleiche Keys in anderer Reihenfolge → gleicher Hash (weil sortiert)
    let keys_a = vec![b"b".to_vec(), b"a".to_vec()];
    let keys_b = vec![b"a".to_vec(), b"b".to_vec()];

    let proof_a = DeletionProof::create(
        DeletionScope::Tenant {
            tenant_id: TenantId::new(2),
        },
        keys_a,
        TxId(1),
        vec![],
        vec![],
        &test_key(),
    )
    .unwrap();
    let proof_b = DeletionProof::create(
        DeletionScope::Tenant {
            tenant_id: TenantId::new(2),
        },
        keys_b,
        TxId(1),
        vec![],
        vec![],
        &test_key(),
    )
    .unwrap();

    assert_eq!(proof_a.deleted_keys_hash, proof_b.deleted_keys_hash);
}

#[test]
fn test_deletion_proof_audit_export_contains_excluded_scopes() {
    let scope = DeletionScope::Tenant {
        tenant_id: TenantId::new(1),
    };
    let proof = DeletionProof::create(
        scope,
        vec![],
        TxId(1),
        vec![],
        vec![
            ExcludedScope::LlmParameterMemory,
            ExcludedScope::ConsolidatedAndDistilled,
        ],
        &test_key(),
    )
    .unwrap();

    let json = proof.export_for_audit().unwrap();
    assert!(json.contains("LlmParameterMemory"));
    assert!(json.contains("ConsolidatedAndDistilled"));
}

#[test]
fn test_deletion_proof_tenant_id_extraction() {
    let t1 = TenantId::new(10);
    let doc_scope = DeletionScope::Document {
        doc_id: DocId(1),
        tenant_id: t1,
    };
    let p1 =
        DeletionProof::create(doc_scope, vec![], TxId(1), vec![], vec![], &test_key()).unwrap();
    assert_eq!(p1.tenant_id(), t1);

    let t2 = TenantId::new(20);
    let col_scope = DeletionScope::Collection {
        collection_id: CollectionId(5),
        tenant_id: t2,
    };
    let p2 =
        DeletionProof::create(col_scope, vec![], TxId(1), vec![], vec![], &test_key()).unwrap();
    assert_eq!(p2.tenant_id(), t2);

    let t3 = TenantId::new(30);
    let tenant_scope = DeletionScope::Tenant { tenant_id: t3 };
    let p3 =
        DeletionProof::create(tenant_scope, vec![], TxId(1), vec![], vec![], &test_key()).unwrap();
    assert_eq!(p3.tenant_id(), t3);
}

#[test]
fn test_deletion_proof_requires_layer_cleanup_proof() {
    let scope = DeletionScope::Tenant {
        tenant_id: TenantId::new(100),
    };

    let cleanup_proofs = vec![
        LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0).unwrap(),
        LayerCleanupProof::new_after_verified_empty(DeletionLayer::SsTableAllLevels, 0).unwrap(),
        LayerCleanupProof::new_after_verified_empty(DeletionLayer::HnswIndex, 0).unwrap(),
    ];

    let proof = DeletionProof::create(
        scope,
        vec![b"k1".to_vec()],
        TxId(1),
        cleanup_proofs,
        vec![],
        &test_key(),
    )
    .unwrap();

    assert!(proof.verify(&test_key()).unwrap());
    assert_eq!(
        proof.covered_layers,
        vec![
            DeletionLayer::LsmMemtable,
            DeletionLayer::SsTableAllLevels,
            DeletionLayer::HnswIndex,
        ]
    );
}

#[test]
fn test_deletion_proof_tampered_covered_layers_fails_verify() {
    let scope = DeletionScope::Tenant {
        tenant_id: TenantId::new(1),
    };
    let proof = DeletionProof::create(
        scope,
        vec![b"k1".to_vec()],
        TxId(10),
        vec![LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0).unwrap()],
        vec![ExcludedScope::LlmParameterMemory],
        &test_key(),
    )
    .unwrap();

    assert_eq!(proof.signature_version, 2);
    assert!(proof.verify(&test_key()).unwrap());

    let mut tampered_layers = proof.clone();
    tampered_layers
        .covered_layers
        .push(DeletionLayer::KvCacheSegments);

    assert!(!tampered_layers.verify(&test_key()).unwrap());
}

#[test]
fn test_deletion_proof_tampered_excluded_scopes_fails_verify() {
    let scope = DeletionScope::Tenant {
        tenant_id: TenantId::new(1),
    };
    let proof = DeletionProof::create(
        scope,
        vec![b"k1".to_vec()],
        TxId(10),
        vec![LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0).unwrap()],
        vec![
            ExcludedScope::LlmParameterMemory,
            ExcludedScope::ConsolidatedAndDistilled,
        ],
        &test_key(),
    )
    .unwrap();

    assert_eq!(proof.signature_version, 2);
    assert!(proof.verify(&test_key()).unwrap());

    let mut tampered_scopes = proof.clone();
    tampered_scopes.excluded_scopes.pop();

    assert!(!tampered_scopes.verify(&test_key()).unwrap());
}

#[test]
fn test_deletion_proof_v1_backward_compatibility() {
    let scope = DeletionScope::Tenant {
        tenant_id: TenantId::new(1),
    };
    let deleted_after_tx = TxId(10);
    let deleted_keys = vec![b"k1".to_vec()];

    // Construct a v1 signature manually using only the 3 legacy fields
    let mut hasher = blake3::Hasher::new();
    for key in &deleted_keys {
        hasher.update(key);
    }
    let deleted_keys_hash: [u8; 32] = *hasher.finalize().as_bytes();

    let scope_bytes = bincode::serialize(&scope).unwrap();
    let tx_bytes = deleted_after_tx.0.to_le_bytes();

    let v1_signature =
        compute_hmac_sha256(&test_key(), &[&scope_bytes, &deleted_keys_hash, &tx_bytes]).unwrap();

    let v1_proof = DeletionProof {
        signature_version: 1,
        scope,
        deleted_keys_hash,
        deleted_after_tx,
        timestamp: 0,
        signature: v1_signature.to_vec(),
        covered_layers: vec![DeletionLayer::LsmMemtable],
        excluded_scopes: vec![ExcludedScope::LlmParameterMemory],
        graph_repair: vec![],
        wal_chain_receipt: None,
        audit_chain_position: None,
        custody: CustodyLevel::Local,
        measurements: vec![],
        uncovered_layers: vec![],
        integrity_warning: None,
    };

    // v1 proof verifies successfully with original 3 fields intact
    assert!(v1_proof.verify(&test_key()).unwrap());

    // Test serde deserialization of JSON missing signature_version defaults to 1
    let json_missing_version = r#"{
        "scope": {"Tenant": {"tenant_id": 1}},
        "deleted_keys_hash": "#
        .to_string()
        + &serde_json::to_string(&deleted_keys_hash).unwrap()
        + r#",
        "deleted_after_tx": 10,
        "signature": "#
        + &serde_json::to_string(&v1_signature).unwrap()
        + r#",
        "covered_layers": ["LsmMemtable"],
        "excluded_scopes": ["LlmParameterMemory"]
    }"#;

    let deserialized: DeletionProof = serde_json::from_str(&json_missing_version).unwrap();
    assert_eq!(deserialized.signature_version, 1);
    assert!(deserialized.verify(&test_key()).unwrap());
}

#[test]
fn test_deletion_proof_v1_audit_export_integrity_warning() {
    let scope = DeletionScope::Tenant {
        tenant_id: TenantId::new(1),
    };
    let deleted_after_tx = TxId(10);
    let deleted_keys = vec![b"k1".to_vec()];

    let mut hasher = blake3::Hasher::new();
    for key in &deleted_keys {
        hasher.update(key);
    }
    let deleted_keys_hash: [u8; 32] = *hasher.finalize().as_bytes();

    let scope_bytes = bincode::serialize(&scope).unwrap();
    let tx_bytes = deleted_after_tx.0.to_le_bytes();

    let v1_signature =
        compute_hmac_sha256(&test_key(), &[&scope_bytes, &deleted_keys_hash, &tx_bytes]).unwrap();

    let v1_proof = DeletionProof {
        signature_version: 1,
        scope,
        deleted_keys_hash,
        deleted_after_tx,
        timestamp: 0,
        signature: v1_signature.to_vec(),
        covered_layers: vec![DeletionLayer::LsmMemtable],
        excluded_scopes: vec![ExcludedScope::LlmParameterMemory],
        graph_repair: vec![],
        wal_chain_receipt: None,
        audit_chain_position: None,
        custody: CustodyLevel::Local,
        measurements: vec![],
        uncovered_layers: vec![],
        integrity_warning: None,
    };

    let json = v1_proof.export_for_audit().unwrap();
    assert!(
        json.contains("integrity_warning"),
        "v1 export MUST contain integrity_warning"
    );
    assert!(
        json.contains(
            "covered_layers/excluded_scopes are not cryptographically signed in this legacy proof version"
        ),
        "v1 export MUST contain the exact warning message"
    );
}

#[test]
fn test_deletion_proof_v2_v3_audit_export_integrity_warning() {
    let scope = DeletionScope::Tenant {
        tenant_id: TenantId::new(1),
    };
    let proof = DeletionProof::create(
        scope,
        vec![b"k1".to_vec()],
        TxId(10),
        vec![LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0).unwrap()],
        vec![ExcludedScope::LlmParameterMemory],
        &test_key(),
    )
    .unwrap();

    assert_eq!(proof.signature_version, 2);
    let json = proof.export_for_audit().unwrap();
    assert!(
        json.contains("integrity_warning"),
        "v2 export MUST contain integrity_warning per step 5"
    );
}

#[test]
fn test_wal_delete_receipt_computation_and_o1_verification() {
    let integrity_key = b"integrity-key-32-bytes-wal-rec!";
    let prev_hmac = [0x55u8; 32];
    let delete_event = b"delete_event:doc_id=42:tx_id=100";

    let receipt = compute_wal_delete_receipt(&prev_hmac, delete_event, integrity_key).unwrap();

    // O(1) Verification without cleartext
    assert!(verify_wal_delete_receipt(&receipt, &prev_hmac, delete_event, integrity_key).unwrap());

    // Tampered receipt or wrong key fails
    let wrong_key = b"wrong-integrity-key-32-bytes---";
    assert!(!verify_wal_delete_receipt(&receipt, &prev_hmac, delete_event, wrong_key).unwrap());

    let tampered_event = b"delete_event:doc_id=43:tx_id=100";
    assert!(
        !verify_wal_delete_receipt(&receipt, &prev_hmac, tampered_event, integrity_key).unwrap()
    );
}

#[test]
fn test_deletion_proof_with_wal_receipt_creation_and_tamper_check() {
    let scope = DeletionScope::Document {
        doc_id: DocId(42),
        tenant_id: TenantId::new(1),
    };
    let integrity_key = b"integrity-key-32-bytes-wal-rec!";
    let prev_hmac = [0x77u8; 32];
    let delete_event = b"doc_42_delete";

    let receipt = compute_wal_delete_receipt(&prev_hmac, delete_event, integrity_key).unwrap();

    let proof = DeletionProof::create_with_wal_receipt(
        scope,
        vec![b"k42".to_vec()],
        TxId(100),
        vec![LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0).unwrap()],
        vec![ExcludedScope::LlmParameterMemory],
        Some(receipt),
        &test_key(),
    )
    .unwrap();

    assert_eq!(proof.wal_chain_receipt, Some(receipt));
    assert!(proof.verify(&test_key()).unwrap());

    // Tamper with receipt
    let mut tampered_proof = proof.clone();
    tampered_proof.wal_chain_receipt = Some([0xFFu8; 32]);
    assert!(!tampered_proof.verify(&test_key()).unwrap());
}

#[test]
fn test_layer_cleanup_proof_verify_and_create() {
    // Success path: closure returns Ok(true)
    let proof_ok = LayerCleanupProof::verify_and_create(DeletionLayer::HnswIndex, || Ok(true));
    assert!(proof_ok.is_ok());
    assert_eq!(proof_ok.unwrap().layer(), &DeletionLayer::HnswIndex);

    // Verification failed path: closure returns Ok(false)
    let proof_fail = LayerCleanupProof::verify_and_create(DeletionLayer::HnswIndex, || Ok(false));
    assert!(
        matches!(proof_fail, Err(ErasureError::Internal(ref msg)) if msg.contains("verification failed"))
    );

    // Closure returns error
    let proof_err = LayerCleanupProof::verify_and_create(DeletionLayer::HnswIndex, || {
        Err(ErasureError::Internal("db error".to_string()))
    });
    assert!(matches!(proof_err, Err(ErasureError::Internal(ref msg)) if msg == "db error"));
}

#[test]
fn test_verify_external_v3_valid_and_invalid_keys() {
    let keypair = DeletionProofKeyPair::generate();
    let scope = DeletionScope::Document {
        doc_id: DocId(42),
        tenant_id: TenantId::new(1),
    };
    let proof = DeletionProof::create_v3(
        scope,
        vec![b"k1".to_vec()],
        TxId(100),
        vec![LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0).unwrap()],
        vec![ExcludedScope::LlmParameterMemory],
        1700000000,
        &[],
        keypair.signing_key(),
    )
    .unwrap();

    // Valid verifying key
    assert!(proof.verify_external(&keypair.verifying_key).is_ok());

    // Invalid verifying key
    let wrong_keypair = DeletionProofKeyPair::generate();
    assert!(proof.verify_external(&wrong_keypair.verifying_key).is_err());
}

#[test]
fn test_verify_external_v2_hmac() {
    let scope = DeletionScope::Tenant {
        tenant_id: TenantId::new(1),
    };
    let hmac_key = b"secret_hmac_key_32_bytes_long!!";
    let proof = DeletionProof::create(
        scope,
        vec![b"k1".to_vec()],
        TxId(10),
        vec![],
        vec![],
        hmac_key,
    )
    .unwrap();

    let keypair = DeletionProofKeyPair::generate();
    assert!(proof.verify_external(&keypair.verifying_key).is_err());
}

#[test]
fn test_deletion_proof_unsupported_signature_version() {
    let scope = DeletionScope::Tenant {
        tenant_id: TenantId::new(1),
    };
    let mut proof =
        DeletionProof::create(scope, vec![], TxId(1), vec![], vec![], &test_key()).unwrap();

    for unknown_byte in [0x00, 0x04, 0xAA, 0xFF] {
        proof.signature_version = unknown_byte;

        // 1. signature_version_typed() MUST return DeletionProofError::UnsupportedVersion
        let typed_res = proof.signature_version_typed();
        assert_eq!(
            typed_res,
            Err(DeletionProofError::UnsupportedVersion(unknown_byte))
        );

        // 2. verify() MUST reject unknown version fail-closed with error and no panic
        let verify_res = proof.verify(&test_key());
        assert!(
            matches!(verify_res, Err(ErasureError::Internal(ref msg)) if msg.contains(&format!("Unsupported DeletionProof signature_version: {unknown_byte}"))),
            "verify() MUST reject unknown byte 0x{unknown_byte:02X} fail-closed"
        );

        // 3. verify_external() MUST reject unknown version
        let keypair = DeletionProofKeyPair::generate();
        let ext_res = proof.verify_external(&keypair.verifying_key);
        assert!(
            matches!(ext_res, Err(CryptoError::UnsupportedProofVersion(v)) if v == unknown_byte),
            "verify_external() MUST reject unknown byte 0x{unknown_byte:02X} fail-closed"
        );

        // 4. export_for_audit() MUST fail on unknown version
        let audit_res = proof.export_for_audit();
        assert!(
            audit_res.is_err(),
            "export_for_audit() MUST fail on unknown version byte 0x{unknown_byte:02X}"
        );
    }
}

#[test]
fn test_bit_flipped_signature_rejected_for_all_valid_versions() {
    // Befund v17 Teil 10.1 & INV-DELETION-1 Regressionstest:
    // Ein um ein Bit manipulierter Signaturwert schlägt bei allen gültigen Versionen (v1, v2, v3) fehl.
    let scope = DeletionScope::Document {
        doc_id: DocId(42),
        tenant_id: TenantId::new(1),
    };

    // --- Version 1 ---
    let scope_bytes = bincode::serialize(&scope).unwrap();
    let deleted_keys_hash = [0u8; 32];
    let tx_bytes = TxId(10).0.to_le_bytes();
    let v1_signature =
        compute_hmac_sha256(&test_key(), &[&scope_bytes, &deleted_keys_hash, &tx_bytes]).unwrap();

    let mut v1_proof = DeletionProof {
        signature_version: 1,
        scope: scope.clone(),
        deleted_keys_hash,
        deleted_after_tx: TxId(10),
        timestamp: 0,
        signature: v1_signature.to_vec(),
        covered_layers: vec![],
        excluded_scopes: vec![],
        graph_repair: vec![],
        wal_chain_receipt: None,
        audit_chain_position: None,
        custody: CustodyLevel::Local,
        measurements: vec![],
        uncovered_layers: vec![],
        integrity_warning: None,
    };
    assert!(v1_proof.verify(&test_key()).unwrap());
    v1_proof.signature[0] ^= 0x01; // Bit-flip
    assert!(!v1_proof.verify(&test_key()).unwrap());

    // --- Version 2 ---
    let mut v2_proof = DeletionProof::create(
        scope.clone(),
        vec![b"k1".to_vec()],
        TxId(10),
        vec![LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0).unwrap()],
        vec![ExcludedScope::LlmParameterMemory],
        &test_key(),
    )
    .unwrap();
    assert!(v2_proof.verify(&test_key()).unwrap());
    v2_proof.signature[0] ^= 0x01; // Bit-flip
    assert!(!v2_proof.verify(&test_key()).unwrap());

    // --- Version 3 ---
    let keypair = DeletionProofKeyPair::generate();
    let mut v3_proof = DeletionProof::create_v3(
        scope,
        vec![b"k1".to_vec()],
        TxId(10),
        vec![LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0).unwrap()],
        vec![ExcludedScope::LlmParameterMemory],
        1700000000,
        &[],
        keypair.signing_key(),
    )
    .unwrap();
    assert!(v3_proof.verify(&keypair.verifying_key).unwrap());
    assert!(v3_proof.verify_external(&keypair.verifying_key).is_ok());

    v3_proof.signature[0] ^= 0x01; // Bit-flip
    assert!(!v3_proof.verify(&keypair.verifying_key).unwrap());
    assert!(matches!(
        v3_proof.verify_external(&keypair.verifying_key),
        Err(CryptoError::InvalidProofSignature)
    ));
}

#[test]
fn test_verify_external_v3_success() {
    let keypair = DeletionProofKeyPair::generate();
    let scope = DeletionScope::Document {
        doc_id: DocId(42),
        tenant_id: TenantId::new(1),
    };
    let proof = DeletionProof::create_v3(
        scope,
        vec![b"k1".to_vec(), b"k2".to_vec()],
        TxId(100),
        vec![LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0).unwrap()],
        vec![ExcludedScope::LlmParameterMemory],
        1700000000,
        &[],
        keypair.signing_key(),
    )
    .unwrap();

    assert!(proof.verify_external(&keypair.verifying_key).is_ok());
}

#[test]
fn test_v3_tampered_graph_repair_rejects() {
    let keypair = DeletionProofKeyPair::generate();
    let scope = DeletionScope::Document {
        doc_id: DocId(42),
        tenant_id: TenantId::new(1),
    };
    let graph_repair = vec![GraphRepairAttestation {
        doc_id: DocId(42),
        verified_no_ghost_pointers: true,
        attested_at: 1700000000,
    }];
    let proof = DeletionProof::create_v3(
        scope,
        vec![b"k1".to_vec()],
        TxId(10),
        vec![
            LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0).unwrap(),
            LayerCleanupProof::new_after_verified_empty(DeletionLayer::HnswIndex, 0).unwrap(),
        ],
        vec![ExcludedScope::LlmParameterMemory],
        1700000000,
        &graph_repair,
        keypair.signing_key(),
    )
    .unwrap();

    assert!(proof.verify(&keypair.verifying_key).unwrap());
    assert!(proof.verify_external(&keypair.verifying_key).is_ok());

    // Tamper graph_repair field
    let mut tampered = proof.clone();
    tampered.graph_repair[0].verified_no_ghost_pointers = false;

    assert!(!tampered.verify(&keypair.verifying_key).unwrap());
    assert!(matches!(
        tampered.verify_external(&keypair.verifying_key),
        Err(CryptoError::InvalidProofSignature)
    ));
}

#[test]
fn test_verify_external_v3_tampered_payload_returns_invalid_proof_signature() {
    let keypair = DeletionProofKeyPair::generate();
    let scope = DeletionScope::Document {
        doc_id: DocId(42),
        tenant_id: TenantId::new(1),
    };
    let proof = DeletionProof::create_with_wal_receipt_v3(
        scope,
        vec![b"k1".to_vec()],
        TxId(10),
        vec![LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0).unwrap()],
        vec![ExcludedScope::LlmParameterMemory],
        Some([0xABu8; 32]),
        1700000000,
        &[],
        keypair.signing_key(),
    )
    .unwrap();

    // 1. Tamper timestamp / TxId
    let mut tampered = proof.clone();
    tampered.deleted_after_tx = TxId(11);
    assert!(matches!(
        tampered.verify_external(&keypair.verifying_key),
        Err(CryptoError::InvalidProofSignature)
    ));

    // 2. Tamper deleted_keys_hash
    let mut tampered = proof.clone();
    tampered.deleted_keys_hash[0] ^= 0xFF;
    assert!(matches!(
        tampered.verify_external(&keypair.verifying_key),
        Err(CryptoError::InvalidProofSignature)
    ));

    // 3. Tamper scope
    let mut tampered = proof.clone();
    tampered.scope = DeletionScope::Document {
        doc_id: DocId(43),
        tenant_id: TenantId::new(1),
    };
    assert!(matches!(
        tampered.verify_external(&keypair.verifying_key),
        Err(CryptoError::InvalidProofSignature)
    ));

    // 4. Tamper covered_layers
    let mut tampered = proof.clone();
    tampered.covered_layers.push(DeletionLayer::HnswIndex);
    assert!(matches!(
        tampered.verify_external(&keypair.verifying_key),
        Err(CryptoError::InvalidProofSignature)
    ));

    // 5. Tamper excluded_scopes
    let mut tampered = proof.clone();
    tampered.excluded_scopes.clear();
    assert!(matches!(
        tampered.verify_external(&keypair.verifying_key),
        Err(CryptoError::InvalidProofSignature)
    ));

    // 6. Tamper wal_chain_receipt
    let mut tampered = proof.clone();
    tampered.wal_chain_receipt = Some([0xCDu8; 32]);
    assert!(matches!(
        tampered.verify_external(&keypair.verifying_key),
        Err(CryptoError::InvalidProofSignature)
    ));

    // 7. Tamper signature bytes
    let mut tampered = proof.clone();
    tampered.signature[0] ^= 0xFF;
    assert!(matches!(
        tampered.verify_external(&keypair.verifying_key),
        Err(CryptoError::InvalidProofSignature)
    ));

    // 8. Invalid public key
    let wrong_keypair = DeletionProofKeyPair::generate();
    assert!(matches!(
        proof.verify_external(&wrong_keypair.verifying_key),
        Err(CryptoError::InvalidProofSignature)
    ));

    // 9. Malformed signature length
    let mut tampered = proof.clone();
    tampered.signature.truncate(32);
    assert!(matches!(
        tampered.verify_external(&keypair.verifying_key),
        Err(CryptoError::InvalidProofSignature)
    ));
}

#[test]
fn test_verify_external_v1_v2_unsupported_version_error() {
    let keypair = DeletionProofKeyPair::generate();
    let scope = DeletionScope::Tenant {
        tenant_id: TenantId::new(1),
    };

    // v2 HMAC proof
    let proof_v2 = DeletionProof::create(
        scope.clone(),
        vec![b"k1".to_vec()],
        TxId(10),
        vec![],
        vec![],
        &test_key(),
    )
    .unwrap();
    assert_eq!(proof_v2.signature_version, 2);
    assert!(matches!(
        proof_v2.verify_external(&keypair.verifying_key),
        Err(CryptoError::UnsupportedProofVersion(2))
    ));

    // v1 HMAC proof
    let mut proof_v1 = proof_v2.clone();
    proof_v1.signature_version = 1;
    assert!(matches!(
        proof_v1.verify_external(&keypair.verifying_key),
        Err(CryptoError::UnsupportedProofVersion(1))
    ));
}
