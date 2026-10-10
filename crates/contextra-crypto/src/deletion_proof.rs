// FILE-CONTEXT
// ZWECK: Cryptographic deletion proof for GDPR Article 17 compliance verification across storage layers.
// INVARIANTEN: INV-DELETION-1: DeletionProof::create() MUST only be invoked AFTER physical layer sanitization.
// NICHT-OFFENSICHTLICH: V1 signiert nur scope, key-hash und tx_id via HMAC-SHA256; V2 signiert scope, key-hash, tx_id, covered_layers, excluded_scopes und wal_receipt via HMAC-SHA256; V3 signiert mit Ed25519 und nutzt längenpräfixierte Key-Hashes. verify() nutzt bei HMAC-Versionen Constant-Time-Vergleiche.
// HOTSPOTS: [40-130]
// STAND: TS:2026-09-07T00:00:00Z

#![forbid(unsafe_code)]

//! Kryptographischer Löschbeweis für die Storage-Ebene.
//!
//! KRITISCHE DECKUNGSGRENZE (Pflicht in Enterprise-Doku und export_for_audit):
//! Dieser Proof deckt AUSSCHLIESSLICH die physischen Storage-Layer ab:
//! `LsmMemtable`, `SsTableAllLevels`, `HnswIndex`, `WalAllSegments`, `CsrGraph`,
//! `KvCacheSegments` und `EmbeddingCache`.
//!
//! Explizit VOM PROOF AUSGESCHLOSSEN (`ExcludedScope`) sind:
//! `ConsolidatedAndDistilled` (Zusammenfassungen für LLM-Fine-Tuning) und
//! `LlmParameterMemory` (LLM-Modellparameter; arXiv:2505.16831).
//! Er KANN NICHT garantieren, dass Wissen aus Fine-Tuning-Zusammenfassungen
//! aus LLM-Parametern entfernbar ist. Referenz: arXiv:2505.16831.
//!
//! INVARIANTE INV-DELETION-1: DeletionProof::create() wird NUR nach
//! physischer Layer-Bereinigung aufgerufen. Proof vor Bereinigung = falsch.

pub mod audit_export;
pub mod keys;
pub mod layer_proof;
pub mod proof;
pub mod proof_v2;
pub mod proof_v3;
pub mod types;
pub mod verify;
pub mod wal_receipt;

pub use keys::*;
pub use layer_proof::{DurabilityProof, LayerCleanupProof};
pub use proof::*;
pub use types::*;
pub use wal_receipt::*;

#[cfg(test)]
mod tests {
    use super::*;
    use contextra_types::{CollectionId, ContextraError, DocId, TenantId, TxId};

    fn test_key() -> Vec<u8> {
        vec![0u8; 32]
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
            tenant_id: TenantId::try_new(1).unwrap(),
        };
        let proof = DeletionProof::create_v3(
            scope,
            vec![b"k1".to_vec(), b"k2".to_vec()],
            TxId(100),
            vec![
                LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0).unwrap(),
            ],
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
            tenant_id: TenantId::try_new(1).unwrap(),
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
            tenant_id: TenantId::try_new(1).unwrap(),
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
            tenant_id: TenantId::try_new(1).unwrap(),
        };
        let proof = DeletionProof::create_with_wal_receipt_v3(
            scope,
            vec![b"k1".to_vec()],
            TxId(10),
            vec![
                LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0).unwrap(),
            ],
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
            tenant_id: TenantId::try_new(1).unwrap(),
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
            tenant_id: TenantId::try_new(1).unwrap(),
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
        let res = proof.verify(VerificationKey::Hmac(&hmac_key));
        assert!(res.is_err());
        match res {
            Err(ContextraError::Internal(msg)) => {
                assert!(msg.contains("HMAC key provided for Ed25519 signature_version 3 proof"));
            }
            _ => panic!("Expected ContextraError::Internal"),
        }
    }


    #[test]
    fn test_v1_v2_still_verify_after_v3_code() {
        // Run existing v1 & v2 tests logic to ensure zero regression
        let scope = DeletionScope::Tenant {
            tenant_id: TenantId::try_new(1).unwrap(),
        };

        // v2 HMAC proof (constructed for verifying legacy support)
        let scope_bytes = bincode::serialize(&scope).unwrap();
        let deleted_keys_hash = hash_deleted_keys_length_prefixed(&[b"k1".to_vec()]);
        let tx_bytes = TxId(10).0.to_le_bytes();

        let stub_v2 = DeletionProof {
            signature_version: 2,
            scope: scope.clone(),
            deleted_keys_hash,
            deleted_after_tx: TxId(10),
            timestamp: 0,
            signature: Vec::new(),
            covered_layers: vec![DeletionLayer::LsmMemtable],
            excluded_scopes: vec![ExcludedScope::LlmParameterMemory],
            graph_repair: vec![],
            wal_chain_receipt: None,
            audit_chain_position: None,
            integrity_warning: None,
        };
        let full_payload = stub_v2.construct_v2_full_payload().unwrap();
        let v2_sig = compute_hmac_sha256(&test_key(), &[&full_payload]).unwrap();

        let mut proof_v2 = stub_v2;
        proof_v2.signature = v2_sig.to_vec();

        assert_eq!(proof_v2.signature_version, 2);
        assert!(proof_v2.verify(VerificationKey::HmacV2(&test_key())).unwrap());

        // v1 HMAC proof
        let v1_signature =
            wal_receipt::compute_hmac_sha256(&test_key(), &[&scope_bytes, &deleted_keys_hash, &tx_bytes])
                .unwrap();

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
            integrity_warning: None,
        };
        assert_eq!(v1_proof.signature_version, 1);
        assert!(v1_proof.verify(VerificationKey::HmacV1(&test_key())).unwrap());
    }

    #[test]
    fn test_v3_uses_length_prefixed_hash() {
        let keypair = DeletionProofKeyPair::generate();
        let scope = DeletionScope::Tenant {
            tenant_id: TenantId::try_new(1).unwrap(),
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
        let tenant = TenantId::try_new(1).unwrap();
        let scope = DeletionScope::Document {
            doc_id: DocId(42),
            tenant_id: tenant,
        };
        let keys = vec![b"key1".to_vec(), b"key2".to_vec()];
        let proof_key = test_key();

        let proof = DeletionProof::create(
            scope,
            keys,
            TxId(100),
            vec![
                LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0).unwrap(),
            ],
            vec![ExcludedScope::LlmParameterMemory],
            &proof_key,
        )
        .unwrap();

        assert_eq!(proof.signature_version, 2);
        assert!(proof.verify(VerificationKey::Hmac(&proof_key)).unwrap());
    }

    #[test]
    fn test_layer_cleanup_proof_rejects_nonzero_remaining_entries() {
        let res = LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 3);
        assert!(res.is_err());
        match res {
            Err(ContextraError::Internal(msg)) => {
                assert!(msg.contains("INV-DELETION-1 violation"));
                assert!(msg.contains("found 3 remaining live entries"));
            }
            _ => panic!("Expected ContextraError::Internal"),
        }
    }

    #[test]
    fn test_deletion_proof_wrong_key_fails_verify() {
        let scope = DeletionScope::Tenant {
            tenant_id: TenantId::try_new(1).unwrap(),
        };
        let proof =
            DeletionProof::create(scope, vec![], TxId(1), vec![], vec![], &test_key()).unwrap();

        let wrong_key = vec![0xFFu8; 32];
        assert!(!proof.verify(VerificationKey::Hmac(&wrong_key)).unwrap());
    }

    #[test]
    fn test_deletion_proof_tampered_data_fails_verify() {
        let scope = DeletionScope::Tenant {
            tenant_id: TenantId::try_new(1).unwrap(),
        };
        let proof_key = test_key();

        let proof = DeletionProof::create(
            scope,
            vec![b"k1".to_vec()],
            TxId(10),
            vec![
                LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0).unwrap(),
            ],
            vec![ExcludedScope::LlmParameterMemory],
            &proof_key,
        )
        .unwrap();

        // Tamper with deleted_after_tx
        let mut tampered_tx = proof.clone();
        tampered_tx.deleted_after_tx = TxId(11);
        assert!(!tampered_tx.verify(VerificationKey::Hmac(&proof_key)).unwrap());

        // Tamper with deleted_keys_hash
        let mut tampered_hash = proof.clone();
        tampered_hash.deleted_keys_hash[0] ^= 0xFF;
        assert!(!tampered_hash.verify(VerificationKey::Hmac(&proof_key)).unwrap());

        // Tamper with scope
        let mut tampered_scope = proof.clone();
        tampered_scope.scope = DeletionScope::Tenant {
            tenant_id: TenantId::try_new(2).unwrap(),
        };
        assert!(!tampered_scope.verify(VerificationKey::Hmac(&proof_key)).unwrap());

        // Untampered original must verify successfully
        assert!(proof.verify(VerificationKey::Hmac(&proof_key)).unwrap());
    }

    #[test]
    fn test_deletion_proof_key_order_deterministic() {
        // Gleiche Keys in anderer Reihenfolge → gleicher Hash (weil sortiert)
        let keys_a = vec![b"b".to_vec(), b"a".to_vec()];
        let keys_b = vec![b"a".to_vec(), b"b".to_vec()];

        let proof_a = DeletionProof::create(
            DeletionScope::Tenant {
                tenant_id: TenantId::try_new(2).unwrap(),
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
                tenant_id: TenantId::try_new(2).unwrap(),
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
            tenant_id: TenantId::try_new(1).unwrap(),
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
        let t1 = TenantId::try_new(10).unwrap();
        let doc_scope = DeletionScope::Document {
            doc_id: DocId(1),
            tenant_id: t1,
        };
        let p1 =
            DeletionProof::create(doc_scope, vec![], TxId(1), vec![], vec![], &test_key()).unwrap();
        assert_eq!(p1.tenant_id(), t1);

        let t2 = TenantId::try_new(20).unwrap();
        let col_scope = DeletionScope::Collection {
            collection_id: CollectionId(5),
            tenant_id: t2,
        };
        let p2 =
            DeletionProof::create(col_scope, vec![], TxId(1), vec![], vec![], &test_key()).unwrap();
        assert_eq!(p2.tenant_id(), t2);

        let t3 = TenantId::try_new(30).unwrap();
        let tenant_scope = DeletionScope::Tenant { tenant_id: t3 };
        let p3 = DeletionProof::create(tenant_scope, vec![], TxId(1), vec![], vec![], &test_key())
            .unwrap();
        assert_eq!(p3.tenant_id(), t3);
    }

    #[test]
    fn test_deletion_proof_requires_layer_cleanup_proof() {
        let scope = DeletionScope::Tenant {
            tenant_id: TenantId::try_new(100).unwrap(),
        };
        let proof_key = test_key();

        let cleanup_proofs = vec![
            LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0).unwrap(),
            LayerCleanupProof::new_after_verified_empty(DeletionLayer::SsTableAllLevels, 0)
                .unwrap(),
            LayerCleanupProof::new_after_verified_empty(DeletionLayer::KvCacheSegments, 0).unwrap(),
        ];

        let proof = DeletionProof::create(
            scope,
            vec![b"k1".to_vec()],
            TxId(1),
            cleanup_proofs,
            vec![],
            &proof_key,
        )
        .unwrap();

        assert!(proof.verify(VerificationKey::Hmac(&proof_key)).unwrap());
        assert_eq!(
            proof.covered_layers,
            vec![
                DeletionLayer::LsmMemtable,
                DeletionLayer::SsTableAllLevels,
                DeletionLayer::KvCacheSegments,
            ]
        );
    }

    #[test]
    fn test_deletion_proof_tampered_covered_layers_fails_verify() {
        let scope = DeletionScope::Tenant {
            tenant_id: TenantId::try_new(1).unwrap(),
        };
        let deleted_keys_hash = hash_deleted_keys_length_prefixed(&[b"k1".to_vec()]);
        let stub_v2 = DeletionProof {
            signature_version: 2,
            scope,
            deleted_keys_hash,
            deleted_after_tx: TxId(10),
            timestamp: 0,
            signature: Vec::new(),
            covered_layers: vec![DeletionLayer::LsmMemtable],
            excluded_scopes: vec![ExcludedScope::LlmParameterMemory],
            graph_repair: vec![],
            wal_chain_receipt: None,
            audit_chain_position: None,
            integrity_warning: None,
        };
        let full_payload = stub_v2.construct_v2_full_payload().unwrap();
        let v2_sig = compute_hmac_sha256(&test_key(), &[&full_payload]).unwrap();

        let mut proof = stub_v2;
        proof.signature = v2_sig.to_vec();

        assert_eq!(proof.signature_version, 2);
        assert!(proof.verify(VerificationKey::HmacV2(&test_key())).unwrap());

        let mut tampered_layers = proof.clone();
        tampered_layers
            .covered_layers
            .push(DeletionLayer::KvCacheSegments);

        assert!(!tampered_layers.verify(VerificationKey::HmacV2(&test_key())).unwrap());
    }

    #[test]
    fn test_deletion_proof_tampered_excluded_scopes_fails_verify() {
        let scope = DeletionScope::Tenant {
            tenant_id: TenantId::try_new(1).unwrap(),
        };
        let deleted_keys_hash = hash_deleted_keys_length_prefixed(&[b"k1".to_vec()]);
        let stub_v2 = DeletionProof {
            signature_version: 2,
            scope,
            deleted_keys_hash,
            deleted_after_tx: TxId(10),
            timestamp: 0,
            signature: Vec::new(),
            covered_layers: vec![DeletionLayer::LsmMemtable],
            excluded_scopes: vec![
                ExcludedScope::LlmParameterMemory,
                ExcludedScope::ConsolidatedAndDistilled,
            ],
            graph_repair: vec![],
            wal_chain_receipt: None,
            audit_chain_position: None,
            integrity_warning: None,
        };
        let full_payload = stub_v2.construct_v2_full_payload().unwrap();
        let v2_sig = compute_hmac_sha256(&test_key(), &[&full_payload]).unwrap();

        let mut proof = stub_v2;
        proof.signature = v2_sig.to_vec();

        assert_eq!(proof.signature_version, 2);
        assert!(proof.verify(VerificationKey::HmacV2(&test_key())).unwrap());

        let mut tampered_scopes = proof.clone();
        tampered_scopes.excluded_scopes.pop();

        assert!(!tampered_scopes.verify(VerificationKey::HmacV2(&test_key())).unwrap());
    }

    #[test]
    fn test_deletion_proof_v1_backward_compatibility() {
        let scope = DeletionScope::Tenant {
            tenant_id: TenantId::try_new(1).unwrap(),
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
            wal_receipt::compute_hmac_sha256(&test_key(), &[&scope_bytes, &deleted_keys_hash, &tx_bytes])
                .unwrap();

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
            tenant_id: TenantId::try_new(1).unwrap(),
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
            wal_receipt::compute_hmac_sha256(&test_key(), &[&scope_bytes, &deleted_keys_hash, &tx_bytes])
                .unwrap();

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
    fn test_deletion_proof_v2_audit_export_no_integrity_warning() {
        let scope = DeletionScope::Tenant {
            tenant_id: TenantId::try_new(1).unwrap(),
        };
        let deleted_keys_hash = hash_deleted_keys_length_prefixed(&[b"k1".to_vec()]);
        let proof = DeletionProof {
            signature_version: 2,
            scope,
            deleted_keys_hash,
            deleted_after_tx: TxId(10),
            timestamp: 0,
            signature: vec![0u8; 32],
            covered_layers: vec![DeletionLayer::LsmMemtable],
            excluded_scopes: vec![ExcludedScope::LlmParameterMemory],
            graph_repair: vec![],
            wal_chain_receipt: None,
            audit_chain_position: None,
            integrity_warning: None,
        };

        assert_eq!(proof.signature_version, 2);
        let json = proof.export_for_audit().unwrap();
        assert!(
            !json.contains("integrity_warning"),
            "v2 export MUST NOT contain integrity_warning"
        );
    }

    #[test]
    fn test_wal_delete_receipt_computation_and_o1_verification() {
        let integrity_key = b"integrity-key-32-bytes-wal-rec!";
        let prev_hmac = [0x55u8; 32];
        let delete_event = b"delete_event:doc_id=42:tx_id=100";

        let receipt = compute_wal_delete_receipt(&prev_hmac, delete_event, integrity_key).unwrap();

        // O(1) Verification without cleartext
        assert!(
            verify_wal_delete_receipt(&receipt, &prev_hmac, delete_event, integrity_key).unwrap()
        );

        // Tampered receipt or wrong key fails
        let wrong_key = b"wrong-integrity-key-32-bytes---";
        assert!(!verify_wal_delete_receipt(&receipt, &prev_hmac, delete_event, wrong_key).unwrap());

        let tampered_event = b"delete_event:doc_id=43:tx_id=100";
        assert!(
            !verify_wal_delete_receipt(&receipt, &prev_hmac, tampered_event, integrity_key)
                .unwrap()
        );
    }

    #[test]
    fn test_deletion_proof_with_wal_receipt_creation_and_tamper_check() {
        let scope = DeletionScope::Document {
            doc_id: DocId(42),
            tenant_id: TenantId::try_new(1).unwrap(),
        };
        let integrity_key = b"integrity-key-32-bytes-wal-rec!";
        let prev_hmac = [0x77u8; 32];
        let delete_event = b"doc_42_delete";
        let proof_key = test_key();

        let receipt = compute_wal_delete_receipt(&prev_hmac, delete_event, integrity_key).unwrap();

        let proof = DeletionProof::create_with_wal_receipt(
            scope,
            vec![b"k42".to_vec()],
            TxId(100),
            vec![
                LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0).unwrap(),
            ],
            vec![ExcludedScope::LlmParameterMemory],
            Some(receipt),
            &proof_key,
        )
        .unwrap();

        assert_eq!(proof.wal_chain_receipt, Some(receipt));
        assert!(proof.verify(VerificationKey::Hmac(&proof_key)).unwrap());

        // Tamper with receipt
        let mut tampered_proof = proof.clone();
        tampered_proof.wal_chain_receipt = Some([0xFFu8; 32]);
        assert!(!tampered_proof.verify(VerificationKey::Hmac(&proof_key)).unwrap());
    }

    #[test]
    fn test_layer_cleanup_proof_verify_and_create() {
        // Success path: closure returns Ok(true)
        let proof_ok = LayerCleanupProof::verify_and_create(DeletionLayer::HnswIndex, || Ok(true));
        assert!(proof_ok.is_ok());
        assert_eq!(proof_ok.unwrap().layer(), &DeletionLayer::HnswIndex);

        // Verification failed path: closure returns Ok(false)
        let proof_fail =
            LayerCleanupProof::verify_and_create(DeletionLayer::HnswIndex, || Ok(false));
        assert!(
            matches!(proof_fail, Err(ContextraError::Internal(ref msg)) if msg.contains("verification failed"))
        );

        // Closure returns error
        let proof_err = LayerCleanupProof::verify_and_create(DeletionLayer::HnswIndex, || {
            Err(ContextraError::Internal("db error".to_string()))
        });
        assert!(matches!(proof_err, Err(ContextraError::Internal(ref msg)) if msg == "db error"));
    }

    #[test]
    fn test_verify_external_v3_valid_and_invalid_keys() {
        let keypair = DeletionProofKeyPair::generate();
        let scope = DeletionScope::Document {
            doc_id: DocId(42),
            tenant_id: TenantId::try_new(1).unwrap(),
        };
        let proof = DeletionProof::create_v3(
            scope,
            vec![b"k1".to_vec()],
            TxId(100),
            vec![
                LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0).unwrap(),
            ],
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
            tenant_id: TenantId::try_new(1).unwrap(),
        };
        let proof = DeletionProof {
            signature_version: 2,
            scope,
            deleted_keys_hash: [0u8; 32],
            deleted_after_tx: TxId(10),
            timestamp: 0,
            signature: vec![0u8; 32],
            covered_layers: vec![],
            excluded_scopes: vec![],
            graph_repair: vec![],
            wal_chain_receipt: None,
            audit_chain_position: None,
            integrity_warning: None,
        };

        let keypair = DeletionProofKeyPair::generate();
        assert!(proof.verify_external(&keypair.verifying_key).is_err());
    }

    #[test]
    fn test_deletion_proof_unsupported_signature_version() {
        let scope = DeletionScope::Tenant {
            tenant_id: TenantId::try_new(1).unwrap(),
        };
        let mut proof =
            DeletionProof::create(scope, vec![], TxId(1), vec![], vec![], &test_key()).unwrap();

        for unknown_byte in [0x00, 0x04, 0xAA, 0xFF] {
            proof.signature_version = unknown_byte;

            // 1. signature_version_typed() MUST return DeletionProofError::UnsupportedVersion
            let typed_res = proof.signature_version_typed();
            assert_eq!(
                typed_res,
                Err(crate::ed25519_proof::DeletionProofError::UnsupportedVersion(unknown_byte))
            );

            // 2. verify() MUST reject unknown version fail-closed with error and no panic
            let verify_res = proof.verify(&test_key());
            assert!(
                matches!(verify_res, Err(ContextraError::Internal(ref msg)) if msg.contains(&format!("Unsupported DeletionProof signature_version: {unknown_byte}"))),
                "verify() MUST reject unknown byte 0x{unknown_byte:02X} fail-closed"
            );

            // 3. verify_external() MUST reject unknown version
            let keypair = DeletionProofKeyPair::generate();
            let ext_res = proof.verify_external(&keypair.verifying_key);
            assert!(
                matches!(ext_res, Err(crate::error::CryptoError::UnsupportedProofVersion(v)) if v == unknown_byte),
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
            tenant_id: TenantId::try_new(1).unwrap(),
        };

        // --- Version 1 ---
        let scope_bytes = bincode::serialize(&scope).unwrap();
        let deleted_keys_hash = [0u8; 32];
        let tx_bytes = TxId(10).0.to_le_bytes();
        let v1_signature =
            wal_receipt::compute_hmac_sha256(&test_key(), &[&scope_bytes, &deleted_keys_hash, &tx_bytes])
                .unwrap();

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
            integrity_warning: None,
        };
        assert!(v1_proof.verify(&test_key()).unwrap());
        v1_proof.signature[0] ^= 0x01; // Bit-flip
        assert!(!v1_proof.verify(&test_key()).unwrap());

        // --- Version 2 ---
        let stub_v2 = DeletionProof {
            signature_version: 2,
            scope: scope.clone(),
            deleted_keys_hash: hash_deleted_keys_length_prefixed(&[b"k1".to_vec()]),
            deleted_after_tx: TxId(10),
            timestamp: 0,
            signature: Vec::new(),
            covered_layers: vec![DeletionLayer::LsmMemtable],
            excluded_scopes: vec![ExcludedScope::LlmParameterMemory],
            graph_repair: vec![],
            wal_chain_receipt: None,
            audit_chain_position: None,
            integrity_warning: None,
        };
        let full_payload = stub_v2.construct_v2_full_payload().unwrap();
        let v2_sig = compute_hmac_sha256(&test_key(), &[&full_payload]).unwrap();
        let mut v2_proof = stub_v2;
        v2_proof.signature = v2_sig.to_vec();

        assert!(v2_proof.verify(VerificationKey::HmacV2(&test_key())).unwrap());
        v2_proof.signature[0] ^= 0x01; // Bit-flip
        assert!(!v2_proof.verify(VerificationKey::HmacV2(&test_key())).unwrap());

        // --- Version 3 ---
        let keypair = DeletionProofKeyPair::generate();
        let mut v3_proof = DeletionProof::create_v3(
            scope,
            vec![b"k1".to_vec()],
            TxId(10),
            vec![
                LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0).unwrap(),
            ],
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
            Err(crate::error::CryptoError::InvalidProofSignature)
        ));
    }

    #[test]
    fn test_verify_external_v3_success() {
        let keypair = DeletionProofKeyPair::generate();
        let scope = DeletionScope::Document {
            doc_id: DocId(42),
            tenant_id: TenantId::try_new(1).unwrap(),
        };
        let proof = DeletionProof::create_v3(
            scope,
            vec![b"k1".to_vec(), b"k2".to_vec()],
            TxId(100),
            vec![
                LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0).unwrap(),
            ],
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
            tenant_id: TenantId::try_new(1).unwrap(),
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
            Err(crate::error::CryptoError::InvalidProofSignature)
        ));
    }

    #[test]
    fn test_verify_external_v3_tampered_payload_returns_invalid_proof_signature() {
        let keypair = DeletionProofKeyPair::generate();
        let scope = DeletionScope::Document {
            doc_id: DocId(42),
            tenant_id: TenantId::try_new(1).unwrap(),
        };
        let proof = DeletionProof::create_with_wal_receipt_v3(
            scope,
            vec![b"k1".to_vec()],
            TxId(10),
            vec![
                LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0).unwrap(),
            ],
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
            Err(crate::error::CryptoError::InvalidProofSignature)
        ));

        // 2. Tamper deleted_keys_hash
        let mut tampered = proof.clone();
        tampered.deleted_keys_hash[0] ^= 0xFF;
        assert!(matches!(
            tampered.verify_external(&keypair.verifying_key),
            Err(crate::error::CryptoError::InvalidProofSignature)
        ));

        // 3. Tamper scope
        let mut tampered = proof.clone();
        tampered.scope = DeletionScope::Document {
            doc_id: DocId(43),
            tenant_id: TenantId::try_new(1).unwrap(),
        };
        assert!(matches!(
            tampered.verify_external(&keypair.verifying_key),
            Err(crate::error::CryptoError::InvalidProofSignature)
        ));

        // 4. Tamper covered_layers
        let mut tampered = proof.clone();
        tampered.covered_layers.push(DeletionLayer::HnswIndex);
        assert!(matches!(
            tampered.verify_external(&keypair.verifying_key),
            Err(crate::error::CryptoError::InvalidProofSignature)
        ));

        // 5. Tamper excluded_scopes
        let mut tampered = proof.clone();
        tampered.excluded_scopes.clear();
        assert!(matches!(
            tampered.verify_external(&keypair.verifying_key),
            Err(crate::error::CryptoError::InvalidProofSignature)
        ));

        // 6. Tamper wal_chain_receipt
        let mut tampered = proof.clone();
        tampered.wal_chain_receipt = Some([0xCDu8; 32]);
        assert!(matches!(
            tampered.verify_external(&keypair.verifying_key),
            Err(crate::error::CryptoError::InvalidProofSignature)
        ));

        // 7. Tamper signature bytes
        let mut tampered = proof.clone();
        tampered.signature[0] ^= 0xFF;
        assert!(matches!(
            tampered.verify_external(&keypair.verifying_key),
            Err(crate::error::CryptoError::InvalidProofSignature)
        ));

        // 8. Invalid public key
        let wrong_keypair = DeletionProofKeyPair::generate();
        assert!(matches!(
            proof.verify_external(&wrong_keypair.verifying_key),
            Err(crate::error::CryptoError::InvalidProofSignature)
        ));

        // 9. Malformed signature length
        let mut tampered = proof.clone();
        tampered.signature.truncate(32);
        assert!(matches!(
            tampered.verify_external(&keypair.verifying_key),
            Err(crate::error::CryptoError::InvalidProofSignature)
        ));
    }

    #[test]
    fn test_verify_external_v1_v2_unsupported_version_error() {
        let keypair = DeletionProofKeyPair::generate();
        let scope = DeletionScope::Tenant {
            tenant_id: TenantId::try_new(1).unwrap(),
        };

        let proof_v2 = DeletionProof {
            signature_version: 2,
            scope: scope.clone(),
            deleted_keys_hash: [0u8; 32],
            deleted_after_tx: TxId(10),
            timestamp: 0,
            signature: vec![0u8; 32],
            covered_layers: vec![],
            excluded_scopes: vec![],
            graph_repair: vec![],
            wal_chain_receipt: None,
            audit_chain_position: None,
            integrity_warning: None,
        };
        assert_eq!(proof_v2.signature_version, 2);
        assert!(matches!(
            proof_v2.verify_external(&keypair.verifying_key),
            Err(crate::error::CryptoError::UnsupportedProofVersion(2))
        ));

        let mut proof_v1 = proof_v2.clone();
        proof_v1.signature_version = 1;
        assert!(matches!(
            proof_v1.verify_external(&keypair.verifying_key),
            Err(crate::error::CryptoError::UnsupportedProofVersion(1))
        ));
    }
}
