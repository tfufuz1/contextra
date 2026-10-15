//! End-to-end integration and negative tests for deletion proof issuance, serialization, and autonomous external verification.

#![forbid(unsafe_code)]

use contextra_crypto::deletion_proof::evidence::WitLayerEvidence;
use contextra_crypto::deletion_proof::policy::ProofPolicy;
use contextra_crypto::deletion_proof::probe::{WitStoreGeneration, WitStoreProbe};
use contextra_crypto::deletion_proof::signer::{ProofKeyPairSigner, ProofSigner};
use contextra_crypto::deletion_proof::types::{DeletionLayer, DeletionScope};
use contextra_crypto::deletion_proof::wiring::{DeletionProofWiring, DeletionProofWiringDeps};
use contextra_crypto::deletion_proof::{IssRequest, ProofV4, WitAssurance, WitClaimText};
use contextra_crypto::deletion_witness::{LayerWitness, WitIssueArgs};
use contextra_crypto::error::CryptoError;
use contextra_crypto::kv_shredding::{shred_segment_key, KeyRegistry};
use contextra_crypto::revocation_log::{marker_path_for, RevocationLog, RevocationTarget};
use contextra_ports::SystemClock;
use contextra_types::{DocId, TenantId, TxId};
use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;
use std::collections::HashSet;
use std::sync::Arc;
use tempfile::tempdir;

struct MockStoreProbe {
    generation: u64,
    write_seq: u64,
}

impl WitStoreProbe for MockStoreProbe {
    fn current_generation(&self, _layer: &DeletionLayer) -> WitStoreGeneration {
        WitStoreGeneration(self.generation)
    }

    fn scope_write_seq(&self, _scope_hash: &[u8; 32]) -> u64 {
        self.write_seq
    }

    fn manifest_file_count(&self) -> Option<u32> {
        None
    }
}

fn create_witness(
    layer: DeletionLayer,
    deleted_keys_hash: [u8; 32],
    footprint_hash: [u8; 32],
) -> LayerWitness {
    LayerWitness::issue(WitIssueArgs {
        layer,
        scope_hash: deleted_keys_hash,
        scanned_at_tx: TxId(10),
        as_of_tx: TxId(5),
        assurance: WitAssurance::S0,
        claim: WitClaimText::RamZeroized,
        footprint_hash,
        scope_write_seq: 10,
        scanner_name: "memtable_scanner",
        scanner_version: 1,
        store_generation: WitStoreGeneration(1),
        layer_evidence: WitLayerEvidence::Memtable {
            memtables_scanned: 1,
            entries_found: 0,
            tombstones_found: 0,
        },
        key_identifiers_claimed: false,
    })
    .expect("issue witness failed")
}

#[test]
fn test_e2e_deletion_proof_creation_serialization_verification_standalone() {
    let tmp = tempdir().expect("tempdir failed");
    let signer = Arc::new(ProofKeyPairSigner::generate(&mut OsRng));
    let rev_sk = SigningKey::generate(&mut OsRng);
    let rev_vk = rev_sk.verifying_key();
    let clock = Arc::new(SystemClock::new());

    let deps = DeletionProofWiringDeps {
        storage_dir: tmp.path().to_path_buf(),
        signer: signer.clone(),
        db_instance_id: [0xAB; 16],
        clock,
        revocation_log: None,
        revocation_signing_key: Some(rev_sk),
        revocation_verifying_key: rev_vk,
        has_existing_keys: false,
    };

    let wiring = DeletionProofWiring::build(deps).expect("wiring build failed");
    let public_key = wiring.public_key();

    let deleted_keys_hash = [0x11u8; 32];
    let footprint_hash = [0x22u8; 32];

    let probe = MockStoreProbe {
        generation: 1,
        write_seq: 10,
    };

    let witness = create_witness(
        DeletionLayer::LsmMemtable,
        deleted_keys_hash,
        footprint_hash,
    );

    let req = IssRequest {
        scope: DeletionScope::Document {
            doc_id: DocId(100),
            tenant_id: TenantId::try_new(1).unwrap(),
        },
        storage_scope: "doc:100".to_string(),
        deleted_keys_hash,
        footprint_hash,
        observed_empty: false,
        deleted_after_tx: 10,
        stage: 1,
        supersedes: None,
        assurance: WitAssurance::S0,
        witnesses: vec![witness],
        extra_exclusions: vec![],
        environment_tlv: vec![],
        wal_chain_receipt: None,
    };

    let issued = wiring
        .issue_proof(req, &[&probe])
        .expect("issue proof failed");

    // Audit chain entry was recorded in wiring
    assert!(wiring.audit_anchor().lock().unwrap().is_some());

    // Serialize proof to JSON string and canonical TLV bytes
    let json_str = serde_json::to_string(&issued.proof).expect("serialize json failed");
    let canonical_tlv = issued.proof.construct_canonical_payload();
    assert!(!json_str.is_empty());
    assert!(!canonical_tlv.is_empty());

    // Fresh process context: deserialize proof without DB state or KeyManager
    let deserialized_proof: ProofV4 =
        serde_json::from_str(&json_str).expect("deserialize json failed");

    // Standalone verification requiring strictly public_key + deserialized proof
    let report = DeletionProofWiring::verify_external(&public_key, &deserialized_proof);
    assert!(
        report.is_valid,
        "Issued proof MUST pass standalone external verification; report: {:?}",
        report
    );
}

#[test]
fn test_e2e_negative_tampered_signature_rejected() {
    let tmp = tempdir().expect("tempdir failed");
    let signer = Arc::new(ProofKeyPairSigner::generate(&mut OsRng));
    let rev_sk = SigningKey::generate(&mut OsRng);
    let rev_vk = rev_sk.verifying_key();
    let clock = Arc::new(SystemClock::new());

    let deps = DeletionProofWiringDeps {
        storage_dir: tmp.path().to_path_buf(),
        signer: signer.clone(),
        db_instance_id: [0xCD; 16],
        clock,
        revocation_log: None,
        revocation_signing_key: Some(rev_sk),
        revocation_verifying_key: rev_vk,
        has_existing_keys: false,
    };

    let wiring = DeletionProofWiring::build(deps).expect("wiring build failed");
    let public_key = wiring.public_key();

    let deleted_keys_hash = [0x33u8; 32];
    let footprint_hash = [0x44u8; 32];

    let probe = MockStoreProbe {
        generation: 1,
        write_seq: 10,
    };

    let witness = create_witness(
        DeletionLayer::LsmMemtable,
        deleted_keys_hash,
        footprint_hash,
    );

    let req = IssRequest {
        scope: DeletionScope::Document {
            doc_id: DocId(200),
            tenant_id: TenantId::try_new(1).unwrap(),
        },
        storage_scope: "doc:200".to_string(),
        deleted_keys_hash,
        footprint_hash,
        observed_empty: false,
        deleted_after_tx: 10,
        stage: 1,
        supersedes: None,
        assurance: WitAssurance::S0,
        witnesses: vec![witness],
        extra_exclusions: vec![],
        environment_tlv: vec![],
        wal_chain_receipt: None,
    };

    let issued = wiring
        .issue_proof(req, &[&probe])
        .expect("issue proof failed");

    let mut tampered_proof = issued.proof;
    // Flip bit in signature
    tampered_proof.signature[0] ^= 0xFF;

    let report = DeletionProofWiring::verify_external(&public_key, &tampered_proof);
    assert!(
        !report.is_valid,
        "Tampered signature MUST fail external verification"
    );
    let r1 = report
        .rule_results
        .iter()
        .find(|r| r.rule_id == "R1")
        .unwrap();
    assert!(!r1.passed, "R1 rule MUST fail on tampered signature");
}

#[test]
fn test_e2e_negative_tampered_payload_rejected() {
    let tmp = tempdir().expect("tempdir failed");
    let signer = Arc::new(ProofKeyPairSigner::generate(&mut OsRng));
    let rev_sk = SigningKey::generate(&mut OsRng);
    let rev_vk = rev_sk.verifying_key();
    let clock = Arc::new(SystemClock::new());

    let deps = DeletionProofWiringDeps {
        storage_dir: tmp.path().to_path_buf(),
        signer: signer.clone(),
        db_instance_id: [0xEF; 16],
        clock,
        revocation_log: None,
        revocation_signing_key: Some(rev_sk),
        revocation_verifying_key: rev_vk,
        has_existing_keys: false,
    };

    let wiring = DeletionProofWiring::build(deps).expect("wiring build failed");
    let public_key = wiring.public_key();

    let deleted_keys_hash = [0x55u8; 32];
    let footprint_hash = [0x66u8; 32];

    let probe = MockStoreProbe {
        generation: 1,
        write_seq: 10,
    };

    let witness = create_witness(
        DeletionLayer::LsmMemtable,
        deleted_keys_hash,
        footprint_hash,
    );

    let req = IssRequest {
        scope: DeletionScope::Document {
            doc_id: DocId(300),
            tenant_id: TenantId::try_new(1).unwrap(),
        },
        storage_scope: "doc:300".to_string(),
        deleted_keys_hash,
        footprint_hash,
        observed_empty: false,
        deleted_after_tx: 10,
        stage: 1,
        supersedes: None,
        assurance: WitAssurance::S0,
        witnesses: vec![witness],
        extra_exclusions: vec![],
        environment_tlv: vec![],
        wal_chain_receipt: None,
    };

    let issued = wiring
        .issue_proof(req, &[&probe])
        .expect("issue proof failed");

    let mut tampered_proof = issued.proof;
    // Modify payload without updating signature
    tampered_proof.deleted_after_tx = 9999;

    let report = DeletionProofWiring::verify_external(&public_key, &tampered_proof);
    assert!(
        !report.is_valid,
        "Tampered payload MUST fail external verification"
    );
}

#[test]
fn test_e2e_negative_wrong_public_key_rejected() {
    let tmp = tempdir().expect("tempdir failed");
    let signer_a = Arc::new(ProofKeyPairSigner::generate(&mut OsRng));
    let signer_b = ProofKeyPairSigner::generate(&mut OsRng);

    let rev_sk = SigningKey::generate(&mut OsRng);
    let rev_vk = rev_sk.verifying_key();
    let clock = Arc::new(SystemClock::new());

    let deps = DeletionProofWiringDeps {
        storage_dir: tmp.path().to_path_buf(),
        signer: signer_a,
        db_instance_id: [0x12; 16],
        clock,
        revocation_log: None,
        revocation_signing_key: Some(rev_sk),
        revocation_verifying_key: rev_vk,
        has_existing_keys: false,
    };

    let wiring = DeletionProofWiring::build(deps).expect("wiring build failed");

    let deleted_keys_hash = [0x77u8; 32];
    let footprint_hash = [0x88u8; 32];

    let probe = MockStoreProbe {
        generation: 1,
        write_seq: 10,
    };

    let witness = create_witness(
        DeletionLayer::LsmMemtable,
        deleted_keys_hash,
        footprint_hash,
    );

    let req = IssRequest {
        scope: DeletionScope::Document {
            doc_id: DocId(400),
            tenant_id: TenantId::try_new(1).unwrap(),
        },
        storage_scope: "doc:400".to_string(),
        deleted_keys_hash,
        footprint_hash,
        observed_empty: false,
        deleted_after_tx: 10,
        stage: 1,
        supersedes: None,
        assurance: WitAssurance::S0,
        witnesses: vec![witness],
        extra_exclusions: vec![],
        environment_tlv: vec![],
        wal_chain_receipt: None,
    };

    let issued = wiring
        .issue_proof(req, &[&probe])
        .expect("issue proof failed");

    // Verify proof signed by signer_a using signer_b's public key
    let wrong_public_key = signer_b.public_key();
    let report = DeletionProofWiring::verify_external(&wrong_public_key, &issued.proof);
    assert!(
        !report.is_valid,
        "Verification against wrong public key MUST fail"
    );
}

#[test]
fn test_e2e_negative_untrusted_expired_signer_key_rejected() {
    let tmp = tempdir().expect("tempdir failed");
    let signer = Arc::new(ProofKeyPairSigner::generate(&mut OsRng));

    let rev_sk = SigningKey::generate(&mut OsRng);
    let rev_vk = rev_sk.verifying_key();
    let clock = Arc::new(SystemClock::new());

    let deps = DeletionProofWiringDeps {
        storage_dir: tmp.path().to_path_buf(),
        signer: signer.clone(),
        db_instance_id: [0x99; 16],
        clock,
        revocation_log: None,
        revocation_signing_key: Some(rev_sk),
        revocation_verifying_key: rev_vk,
        has_existing_keys: false,
    };

    let wiring = DeletionProofWiring::build(deps).expect("wiring build failed");

    let deleted_keys_hash = [0x99u8; 32];
    let footprint_hash = [0xAAu8; 32];

    let probe = MockStoreProbe {
        generation: 1,
        write_seq: 10,
    };

    let witness = create_witness(
        DeletionLayer::LsmMemtable,
        deleted_keys_hash,
        footprint_hash,
    );

    let req = IssRequest {
        scope: DeletionScope::Document {
            doc_id: DocId(500),
            tenant_id: TenantId::try_new(1).unwrap(),
        },
        storage_scope: "doc:500".to_string(),
        deleted_keys_hash,
        footprint_hash,
        observed_empty: false,
        deleted_after_tx: 10,
        stage: 1,
        supersedes: None,
        assurance: WitAssurance::S0,
        witnesses: vec![witness],
        extra_exclusions: vec![],
        environment_tlv: vec![],
        wal_chain_receipt: None,
    };

    let issued = wiring
        .issue_proof(req, &[&probe])
        .expect("issue proof failed");

    // Policy with trusted_signer_key_ids set to an expired/different trusted key ID
    let mut trusted_signers = HashSet::new();
    trusted_signers.insert([0xFFu8; 32]); // Different key ID

    let policy = ProofPolicy {
        trusted_signer_key_ids: trusted_signers,
        ..ProofPolicy::default()
    };

    let report = DeletionProofWiring::verify_external_with_policy(
        &wiring.public_key(),
        &issued.proof,
        &policy,
    );

    assert!(
        !report.is_valid,
        "Verification MUST fail when signer key ID is not trusted / expired"
    );
    let r16 = report
        .rule_results
        .iter()
        .find(|r| r.rule_id == "R16")
        .unwrap();
    assert!(
        !r16.passed,
        "R16 rule MUST fail on untrusted/expired signer key ID"
    );
}

#[test]
fn test_e2e_negative_revoked_key_rejected() {
    let registry = KeyRegistry::new();
    let tenant = TenantId::try_new(10).unwrap();
    let segment_id = 9999;

    // Shred segment key
    let shredded = shred_segment_key(&registry, tenant, segment_id).expect("shred failed");
    assert!(shredded);

    // Any access attempt to revoked key MUST return error / key revoked status, NEVER Ok
    let kdf_hdr = contextra_crypto::KdfHeader::generate_default().unwrap();
    let km = contextra_crypto::crypto::KeyManager::try_new_strict(
        "passphrase-strict-policy-ok",
        &kdf_hdr,
    )
    .unwrap();
    let res = registry.get_or_derive(&km, segment_id);
    assert!(
        res.is_err(),
        "Accessing revoked segment key MUST return error"
    );
    match res {
        Err(CryptoError::KeyRevoked(msg)) => {
            assert!(msg.contains("9999"));
        }
        res => panic!("Expected CryptoError::KeyRevoked, got: {:?}", res),
    }
}

#[test]
fn test_e2e_negative_revoked_key_and_missing_log_fails_closed() {
    let tmp = tempdir().expect("tempdir failed");
    let log_path = tmp.path().join("revocation.log");

    let rev_sk = SigningKey::generate(&mut OsRng);
    let rev_vk = rev_sk.verifying_key();
    let clock = Arc::new(SystemClock::new());

    let rev_log = RevocationLog::open_or_create(&log_path, clock.clone(), Some(rev_sk), rev_vk)
        .expect("open_or_create failed");

    let group_target = RevocationTarget::Group(12345);
    rev_log.append(group_target.clone()).expect("append failed");
    assert!(rev_log.is_revoked(&group_target));

    // Remove initialization marker file to test fail-closed behavior
    let marker_path = marker_path_for(&log_path);
    assert!(
        marker_path.exists(),
        "Marker file must exist before deletion"
    );
    std::fs::remove_file(&marker_path).expect("remove marker failed");

    // Opening with has_existing_keys = true when marker is missing MUST fail closed
    let res = RevocationLog::open_or_create_if_fresh(
        &log_path, clock, None, rev_vk, true, // has_existing_keys = true
    );

    assert!(
        res.is_err(),
        "RevocationLog MUST fail-closed with error when marker is missing with existing keys"
    );
    assert!(matches!(res, Err(CryptoError::IntegrityViolation)));
}

#[test]
fn test_e2e_kms_rotation_and_old_proof_verification() {
    let tmp = tempdir().expect("tempdir failed");
    let initial_signer = Arc::new(ProofKeyPairSigner::generate(&mut OsRng));
    let initial_public_key = initial_signer.public_key();

    let rev_sk = SigningKey::generate(&mut OsRng);
    let rev_vk = rev_sk.verifying_key();
    let clock = Arc::new(SystemClock::new());

    let deps = DeletionProofWiringDeps {
        storage_dir: tmp.path().to_path_buf(),
        signer: initial_signer,
        db_instance_id: [0x77; 16],
        clock: clock.clone(),
        revocation_log: None,
        revocation_signing_key: Some(rev_sk.clone()),
        revocation_verifying_key: rev_vk,
        has_existing_keys: false,
    };

    let wiring = DeletionProofWiring::build(deps).expect("initial wiring build failed");

    let deleted_keys_hash = [0xAAu8; 32];
    let footprint_hash = [0xBBu8; 32];
    let probe = MockStoreProbe {
        generation: 1,
        write_seq: 10,
    };
    let witness = create_witness(
        DeletionLayer::LsmMemtable,
        deleted_keys_hash,
        footprint_hash,
    );

    let req = IssRequest {
        scope: DeletionScope::Document {
            doc_id: DocId(700),
            tenant_id: TenantId::try_new(1).unwrap(),
        },
        storage_scope: "doc:700".to_string(),
        deleted_keys_hash,
        footprint_hash,
        observed_empty: false,
        deleted_after_tx: 10,
        stage: 1,
        supersedes: None,
        assurance: WitAssurance::S0,
        witnesses: vec![witness],
        extra_exclusions: vec![],
        environment_tlv: vec![],
        wal_chain_receipt: None,
    };

    let issued_old = wiring
        .issue_proof(req, &[&probe])
        .expect("issue proof with old key failed");

    // Execute Key Rotation: New signer generated
    let new_signer = Arc::new(ProofKeyPairSigner::generate(&mut OsRng));

    let deps_rotated = DeletionProofWiringDeps {
        storage_dir: tmp.path().to_path_buf(),
        signer: new_signer.clone(),
        db_instance_id: [0x77; 16],
        clock,
        revocation_log: Some(wiring.revocation_log()),
        revocation_signing_key: None,
        revocation_verifying_key: rev_vk,
        has_existing_keys: true,
    };

    let rotated_wiring =
        DeletionProofWiring::build(deps_rotated).expect("rotated wiring build failed");
    assert_ne!(rotated_wiring.public_key(), initial_public_key);

    // Old proof remains verifiably valid when checked against the initial_public_key
    let old_report = DeletionProofWiring::verify_external(&initial_public_key, &issued_old.proof);
    assert!(
        old_report.is_valid,
        "Proof issued prior to key rotation MUST remain externally verifiable against historical public key"
    );
}
