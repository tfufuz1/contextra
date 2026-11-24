#![cfg(feature = "signing")]

//! # Integrationstest: `deletion_proof_signature_version_serde.rs`
//!
//! Validiert die kryptographischen Invarianten und Sicherheitsanforderungen für `deletion_proof_signature_version_serde`.
//! Prüft Zero-Panic, deterministische Ausführung und Fail-Closed-Verhalten.

use contextra_erasure::deletion_proof::{
    DeletionLayer, DeletionProof, DeletionScope, ExcludedScope, LayerCleanupProof,
};
use contextra_erasure::ed25519_proof::{
    DeletionProofError, DeletionProofKeyPair, SignatureVersion,
};
use contextra_erasure::types_local::{DocId, TenantId, TxId};

#[test]
fn test_signature_version_try_from_u8() {
    assert_eq!(SignatureVersion::try_from(1), Ok(SignatureVersion::V1));
    assert_eq!(SignatureVersion::try_from(2), Ok(SignatureVersion::V2));
    assert_eq!(SignatureVersion::try_from(3), Ok(SignatureVersion::V3));

    assert_eq!(
        SignatureVersion::try_from(0),
        Err(DeletionProofError::UnsupportedVersion(0))
    );
    assert_eq!(
        SignatureVersion::try_from(4),
        Err(DeletionProofError::UnsupportedVersion(4))
    );
    assert_eq!(
        SignatureVersion::try_from(255),
        Err(DeletionProofError::UnsupportedVersion(255))
    );
}

#[test]
fn test_deletion_proof_typed_signature_version_accessor() {
    let mut proof = DeletionProof {
        signature_version: 1,
        scope: DeletionScope::Tenant {
            tenant_id: TenantId(1),
        },
        deleted_keys_hash: [0u8; 32],
        deleted_after_tx: TxId(10),
        timestamp: 0,
        signature: vec![],
        covered_layers: vec![],
        excluded_scopes: vec![],
        graph_repair: vec![],
        wal_chain_receipt: None,
        audit_chain_position: None,
        custody: contextra_erasure::deletion_proof::CustodyLevel::Local,
        measurements: vec![],
        uncovered_layers: vec![],
        integrity_warning: None,
    };

    assert_eq!(proof.signature_version_typed(), Ok(SignatureVersion::V1));

    proof.signature_version = 2;
    assert_eq!(proof.signature_version_typed(), Ok(SignatureVersion::V2));

    proof.signature_version = 3;
    assert_eq!(proof.signature_version_typed(), Ok(SignatureVersion::V3));

    proof.signature_version = 0;
    assert_eq!(
        proof.signature_version_typed(),
        Err(DeletionProofError::UnsupportedVersion(0))
    );

    proof.signature_version = 4;
    assert_eq!(
        proof.signature_version_typed(),
        Err(DeletionProofError::UnsupportedVersion(4))
    );
}

#[test]
fn test_golden_json_v1_v2_v3_serde_roundtrip() {
    let tenant_id = TenantId(42);
    let scope = DeletionScope::Document {
        doc_id: DocId(101),
        tenant_id,
    };

    // Golden JSON V1 (without signature_version field, serde default should be 1)
    let golden_v1_json = r#"{
  "scope": {
    "Document": {
      "doc_id": 101,
      "tenant_id": 42
    }
  },
  "deleted_keys_hash": [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
  "deleted_after_tx": 10,
  "timestamp": 0,
  "signature": [1,2,3,4],
  "covered_layers": [],
  "excluded_scopes": []
}"#;

    let proof_v1: DeletionProof = serde_json::from_str(golden_v1_json).unwrap();
    assert_eq!(proof_v1.signature_version, 1);
    assert_eq!(proof_v1.signature_version_typed(), Ok(SignatureVersion::V1));

    // Golden JSON V2
    let proof_v2_orig = DeletionProof::create(
        scope.clone(),
        vec![b"key_a".to_vec()],
        TxId(20),
        vec![LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0).unwrap()],
        vec![ExcludedScope::LlmParameterMemory],
        b"hmac_key_32_bytes_long_secret!!",
    )
    .unwrap();

    let json_v2 = serde_json::to_string_pretty(&proof_v2_orig).unwrap();
    assert!(json_v2.contains(r#""signature_version": 2"#));
    let proof_v2_de: DeletionProof = serde_json::from_str(&json_v2).unwrap();
    assert_eq!(proof_v2_de.signature_version, 2);
    assert_eq!(
        proof_v2_de.signature_version_typed(),
        Ok(SignatureVersion::V2)
    );
    assert!(proof_v2_de
        .verify(b"hmac_key_32_bytes_long_secret!!")
        .unwrap());

    // Golden JSON V3
    let mut rng = rand::rngs::OsRng;
    let keypair = DeletionProofKeyPair::generate_with_rng(&mut rng);
    let proof_v3_orig = DeletionProof::create_v3(
        scope,
        vec![b"key_a".to_vec()],
        TxId(30),
        vec![LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0).unwrap()],
        vec![ExcludedScope::LlmParameterMemory],
        1700000000,
        &[],
        keypair.signing_key(),
    )
    .unwrap();

    let json_v3 = serde_json::to_string_pretty(&proof_v3_orig).unwrap();
    assert!(json_v3.contains(r#""signature_version": 3"#));
    let proof_v3_de: DeletionProof = serde_json::from_str(&json_v3).unwrap();
    assert_eq!(proof_v3_de.signature_version, 3);
    assert_eq!(
        proof_v3_de.signature_version_typed(),
        Ok(SignatureVersion::V3)
    );
    assert!(proof_v3_de.verify(&keypair.verifying_key).unwrap());

    // Golden JSON V5
    use contextra_erasure::types_local::ScanTechnique;
    use contextra_erasure::types_local::SystemClock;
    fn mint_witness_for_tests(
        layer: &str,
        scanned: u64,
        remaining: u64,
        tech: contextra_erasure::types_local::ScanTechnique,
    ) -> contextra_erasure::types_local::ScanWitness {
        use contextra_erasure::types_local::{ScanSession, ScanTarget, SystemClock, WitnessLayer};

        let target = ScanTarget::new("target", [0x11; 32]);
        let mut session = ScanSession::begin(WitnessLayer::from(layer), target, tech.clone());
        if !matches!(
            tech,
            contextra_erasure::types_local::ScanTechnique::FeatureDisabled
        ) {
            session.record_control_match();
            for i in 0..scanned {
                session.record_unit([(i % 255) as u8; 32], i < remaining);
            }
        }
        let clock = SystemClock::new();
        session.finish(&clock).unwrap()
    }
    use contextra_erasure::types_local::{RequestContext, SagaRecord};

    let clock = SystemClock::new();
    let ctx = RequestContext::new(tenant_id, TxId(50));
    let saga = SagaRecord::new([0x55; 32]);
    let witnesses = vec![
        mint_witness_for_tests("LsmMemtable", 100, 0, ScanTechnique::Structural),
        mint_witness_for_tests("SsTableAllLevels", 500, 0, ScanTechnique::Structural),
        mint_witness_for_tests("WalAllSegments", 1000, 0, ScanTechnique::Structural),
    ];

    let proof_v5_orig = DeletionProof::issue(
        &ctx,
        &saga,
        &witnesses,
        &[
            ExcludedScope::LlmParameterMemory,
            ExcludedScope::ConsolidatedAndDistilled,
        ],
        contextra_erasure::deletion_proof::CustodyLevel::Sovereign,
        keypair.signing_key(),
        &clock,
    )
    .expect("V5 issue must succeed");

    let json_v5 = serde_json::to_string_pretty(&proof_v5_orig).unwrap();
    assert!(json_v5.contains(r#""signature_version": 5"#));
    assert!(json_v5.contains(r#""custody": "Sovereign""#));
    let proof_v5_de: DeletionProof = serde_json::from_str(&json_v5).unwrap();
    assert_eq!(proof_v5_de.signature_version, 5);
    assert_eq!(
        proof_v5_de.signature_version_typed(),
        Ok(SignatureVersion::V5)
    );
    assert!(proof_v5_de.verify(&keypair.verifying_key).unwrap());
    assert!(proof_v5_de.verify_external(&keypair.verifying_key).is_ok());
}

#[test]
fn test_verify_rejects_unknown_versions_fail_closed() {
    let mut proof = DeletionProof {
        signature_version: 0,
        scope: DeletionScope::Tenant {
            tenant_id: TenantId(1),
        },
        deleted_keys_hash: [0u8; 32],
        deleted_after_tx: TxId(10),
        timestamp: 0,
        signature: vec![],
        covered_layers: vec![],
        excluded_scopes: vec![],
        graph_repair: vec![],
        wal_chain_receipt: None,
        audit_chain_position: None,
        custody: contextra_erasure::deletion_proof::CustodyLevel::Local,
        measurements: vec![],
        uncovered_layers: vec![],
        integrity_warning: None,
    };

    assert!(proof.verify(b"secret").is_err());

    proof.signature_version = 4;
    assert!(proof.verify(b"secret").is_err());
}
