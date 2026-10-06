use contextra_crypto::crypto::KeyManager;
use contextra_crypto::deletion_proof::{
    DeletionLayer, DeletionProof, DeletionProofKeyPair, DeletionScope, ExcludedScope,
    LayerCleanupProof, VerificationKey,
};
use contextra_types::{DocId, TenantId, TxId};

fn test_hmac_key() -> Vec<u8> {
    vec![0x42u8; 32]
}

#[test]
fn test_v2_tampered_timestamp_fails_verify() {
    let scope = DeletionScope::Document {
        doc_id: DocId(101),
        tenant_id: TenantId::try_new(1).unwrap(),
    };
    let proof = DeletionProof::create(
        scope,
        vec![b"key_a".to_vec()],
        TxId(10),
        vec![LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0).unwrap()],
        vec![ExcludedScope::LlmParameterMemory],
        &test_hmac_key(),
    )
    .unwrap();

    assert!(proof.verify(&test_hmac_key()).unwrap());

    // Tamper with timestamp
    let mut tampered = proof.clone();
    tampered.timestamp = 123456789;
    assert!(
        !tampered.verify(&test_hmac_key()).unwrap(),
        "Tampered timestamp on new v2 proof MUST fail verification"
    );
}

#[test]
fn test_v2_tampered_audit_chain_position_fails_verify() {
    let scope = DeletionScope::Document {
        doc_id: DocId(102),
        tenant_id: TenantId::try_new(1).unwrap(),
    };
    let proof = DeletionProof::create(
        scope,
        vec![b"key_b".to_vec()],
        TxId(20),
        vec![LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0).unwrap()],
        vec![ExcludedScope::LlmParameterMemory],
        &test_hmac_key(),
    )
    .unwrap();

    assert!(proof.verify(&test_hmac_key()).unwrap());

    // Tamper with audit_chain_position
    let mut tampered = proof.clone();
    tampered.audit_chain_position = Some(999);
    assert!(
        !tampered.verify(&test_hmac_key()).unwrap(),
        "Tampered audit_chain_position on new v2 proof MUST fail verification"
    );
}

#[test]
fn test_v2_tampered_integrity_warning_fails_verify() {
    let scope = DeletionScope::Document {
        doc_id: DocId(103),
        tenant_id: TenantId::try_new(1).unwrap(),
    };
    let proof = DeletionProof::create(
        scope,
        vec![b"key_c".to_vec()],
        TxId(30),
        vec![LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0).unwrap()],
        vec![ExcludedScope::LlmParameterMemory],
        &test_hmac_key(),
    )
    .unwrap();

    assert!(proof.verify(&test_hmac_key()).unwrap());

    // Tamper with integrity_warning
    let mut tampered = proof.clone();
    tampered.integrity_warning = Some("tampered warning".to_string());
    assert!(
        !tampered.verify(&test_hmac_key()).unwrap(),
        "Tampered integrity_warning on new v2 proof MUST fail verification"
    );
}

#[test]
fn test_legacy_v1_v2_proofs_still_verify() {
    let scope = DeletionScope::Tenant {
        tenant_id: TenantId::try_new(1).unwrap(),
    };

    // Construct a legacy v2 HMAC signature using the old formula
    let scope_bytes = bincode::serialize(&scope).unwrap();
    let deleted_keys_hash = [0x11u8; 32];
    let tx_bytes = TxId(5).0.to_le_bytes();
    let covered_layers = vec![DeletionLayer::LsmMemtable];
    let covered_layers_bytes = bincode::serialize(&covered_layers).unwrap();
    let excluded_scopes: Vec<ExcludedScope> = vec![ExcludedScope::LlmParameterMemory];
    let excluded_scopes_bytes = bincode::serialize(&excluded_scopes).unwrap();

    let hmac_key = test_hmac_key();

    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    let mut mac = Hmac::<Sha256>::new_from_slice(&hmac_key).unwrap();
    mac.update(&scope_bytes);
    mac.update(&deleted_keys_hash);
    mac.update(&tx_bytes);
    mac.update(&covered_layers_bytes);
    mac.update(&excluded_scopes_bytes);
    let legacy_signature: [u8; 32] = mac.finalize().into_bytes().into();

    let legacy_v2_proof = DeletionProof {
        signature_version: 2,
        scope: scope.clone(),
        deleted_keys_hash,
        deleted_after_tx: TxId(5),
        timestamp: 0,
        signature: legacy_signature.to_vec(),
        covered_layers,
        excluded_scopes,
        graph_repair: vec![],
        wal_chain_receipt: None,
        audit_chain_position: None,
        integrity_warning: None,
    };

    assert!(
        legacy_v2_proof.verify(&hmac_key).unwrap(),
        "Legacy v2 proof MUST continue to verify via fallback path"
    );

    // Legacy v1 proof
    let mut mac_v1 = Hmac::<Sha256>::new_from_slice(&hmac_key).unwrap();
    mac_v1.update(&scope_bytes);
    mac_v1.update(&deleted_keys_hash);
    mac_v1.update(&tx_bytes);
    let v1_signature: [u8; 32] = mac_v1.finalize().into_bytes().into();

    let legacy_v1_proof = DeletionProof {
        signature_version: 1,
        scope,
        deleted_keys_hash,
        deleted_after_tx: TxId(5),
        timestamp: 0,
        signature: v1_signature.to_vec(),
        covered_layers: vec![],
        excluded_scopes: vec![],
        graph_repair: vec![],
        wal_chain_receipt: None,
        audit_chain_position: None,
        integrity_warning: None,
    };

    assert!(
        legacy_v1_proof.verify(&hmac_key).unwrap(),
        "Legacy v1 proof MUST continue to verify"
    );
}

#[test]
fn test_key_manager_creates_v3_asymmetric_proof() {
    let km = KeyManager::try_new("passphrase-v3-test", b"salt-v3-test-123").unwrap();
    let keypair = DeletionProofKeyPair::generate();
    let scope = DeletionScope::Document {
        doc_id: DocId(42),
        tenant_id: TenantId::try_new(1).unwrap(),
    };

    let proof = KeyManager::create_deletion_proof_v3(
        &keypair,
        scope,
        vec![b"key1".to_vec()],
        TxId(100),
        vec![LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0).unwrap()],
        vec![ExcludedScope::LlmParameterMemory],
        1700000000,
        &[],
    )
    .unwrap();

    assert_eq!(proof.signature_version, 3);
    assert_eq!(proof.signature.len(), 64);

    // Verify via Keypair's verifying_key
    assert!(proof.verify(&keypair.verifying_key).unwrap());
    assert!(proof.verify_external(&keypair.verifying_key).is_ok());

    // Master key alone cannot verify v3 proof (type mismatch)
    let km_hmac_key = km.derive_deletion_proof_key().unwrap();
    let res = proof.verify(&km_hmac_key);
    assert!(res.is_err());
}

#[test]
fn test_downgrade_v3_tampered_version_and_hmac_key_rejected() {
    let keypair = DeletionProofKeyPair::generate();
    let scope = DeletionScope::Document {
        doc_id: DocId(42),
        tenant_id: TenantId::try_new(1).unwrap(),
    };

    let proof = DeletionProof::create_v3(
        scope,
        vec![b"key1".to_vec()],
        TxId(100),
        vec![LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0).unwrap()],
        vec![ExcludedScope::LlmParameterMemory],
        1700000000,
        &[],
        keypair.signing_key(),
    )
    .unwrap();

    assert!(proof.verify(&keypair.verifying_key).unwrap());

    // Attacker tampers signature_version from 3 to 2 and attempts HMAC verification
    let mut tampered = proof.clone();
    tampered.signature_version = 2;

    let hmac_key = test_hmac_key();
    assert!(
        !tampered.verify(&hmac_key).unwrap(),
        "Downgraded v3 proof with tampered signature_version=2 and HMAC key MUST fail verification"
    );

    // Tamper signature_version from 3 to 1
    let mut tampered_v1 = proof.clone();
    tampered_v1.signature_version = 1;
    assert!(
        !tampered_v1.verify(&hmac_key).unwrap(),
        "Downgraded v3 proof with tampered signature_version=1 and HMAC key MUST fail verification"
    );
}

#[test]
fn test_zeroize_overwrites_keypair_bytes_on_drop() {
    let mut keypair = DeletionProofKeyPair::generate();
    let initial_vk_bytes = keypair.verifying_key_bytes();
    assert_ne!(initial_vk_bytes, [0u8; 32]);

    let initial_sk_bytes = keypair.signing_key().to_bytes();
    assert_ne!(initial_sk_bytes, [0u8; 32]);

    // Perform zeroize explicitly
    keypair.zeroize();

    let zeroized_sk_bytes = keypair.signing_key().to_bytes();
    assert_eq!(
        zeroized_sk_bytes, [0u8; 32],
        "Signing key bytes MUST be overwritten with zeros after zeroize()"
    );
}

#[test]
fn test_keypair_debug_redacts_signing_key() {
    let keypair = DeletionProofKeyPair::generate();
    let debug_str = format!("{keypair:?}");
    assert!(debug_str.contains("REDACTED"));
    assert!(!debug_str.contains(&format!("{:?}", keypair.signing_key().to_bytes())));
}

#[test]
fn test_verify_with_key_history_across_key_rotations() {
    let old_key = vec![0x11u8; 32];
    let current_key = vec![0x22u8; 32];
    let future_key = vec![0x33u8; 32];

    let scope = DeletionScope::Tenant {
        tenant_id: TenantId::try_new(10).unwrap(),
    };

    // Proof created with old_key before rotation
    let old_proof = DeletionProof::create(
        scope,
        vec![b"k1".to_vec()],
        TxId(1),
        vec![],
        vec![],
        &old_key,
    )
    .unwrap();

    // Verifying against current_key alone fails
    assert!(!old_proof.verify(&current_key).unwrap());

    // Verifying against key history [current_key, old_key, future_key] succeeds!
    let key_history: Vec<VerificationKey> = vec![
        VerificationKey::from(&current_key),
        VerificationKey::from(&old_key),
        VerificationKey::from(&future_key),
    ];

    assert!(
        old_proof.verify_with_key_history(&key_history).unwrap(),
        "Proof MUST verify successfully against historical key in key history"
    );

    // Verifying against unrelated key history fails
    let wrong_history: Vec<VerificationKey> = vec![
        VerificationKey::from(&current_key),
        VerificationKey::from(&future_key),
    ];
    assert!(!old_proof.verify_with_key_history(&wrong_history).unwrap());
}
