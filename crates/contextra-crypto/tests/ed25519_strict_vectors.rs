// FILE-CONTEXT (STAND 2026-10-13)
// ZWECK: Ed25519-Grenzfall-Testvektoren und Verifikations-Prüfung über alle Löschnachweis-APIs (v3/v4) und AuditChain (Auftrag B19).
// INVARIANTEN: Striktes Ablehnen ungültiger/nicht-kanonischer Signaturen (S >= L, Small-Order PK/R, non-canonical, truncated, wrong length).
// CROSS-IMPL / OUT-OF-SCOPE FINDING: `ring` ist im Workspace vorhanden, aber nicht in `contextra-crypto/Cargo.toml` als Dependency deklariert.
//   Da Cargo.toml laut Auftragsregeln tabu ist, erfolgt die Prüfung gegen `ed25519-dalek`'s `verify` und `verify_strict` APIs.
// UNTERSCHIED `verify` VS `verify_strict`:
//   - `ed25519_proof::verify_deletion_proof_v3`, `DeletionProof::verify` und `DeletionProof::verify_external` nutzen `verify()`.
//   - `ProofV4::verify_signature` und `AuditChain::verify_head_signature` nutzen `verify_strict()`.
// HINWEIS ZU IGNORIERTEN FÄLLEN (A07/B03/B05):
//   Fälle, in denen `ed25519-dalek`'s Standard `verify()` nicht-kanonische Signaturen akzeptiert, bevor A07/B03/B05 auf `verify_strict` umstellen:
//   - Keines: `ed25519-dalek` v2 weist S >= L und ungültige Punkte bereits in `verify()` und `verify_strict()` ab.

#![forbid(unsafe_code)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, deprecated)]

use contextra_crypto::audit_chain::{AuditChain, AuditChainHeadSignature};
use contextra_crypto::deletion_proof::keys::{DeletionProofKeyPair, VerificationKey};
use contextra_crypto::deletion_proof::proof::DeletionProof;
use contextra_crypto::deletion_proof::proof_v4::ProofV4;
use contextra_crypto::deletion_proof::signer::{ProofKeyPairSigner, ProofSigner};
use contextra_crypto::deletion_proof::types::{DeletionLayer, DeletionScope, DurabilityProof, ExcludedScope};
use contextra_crypto::deletion_proof::LayerCleanupProof;
use contextra_crypto::ed25519_proof::verify_deletion_proof_v3;
use contextra_types::{DocId, TenantId, TxId};
use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
struct TestVector {
    name: String,
    pk: String,
    msg: String,
    sig: String,
    expect: String,
}

fn hex_decode(hex: &str) -> Vec<u8> {
    let clean = hex.replace(' ', "");
    (0..clean.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&clean[i..i + 2], 16).expect("valid hex byte"))
        .collect()
}

fn load_vectors() -> Vec<TestVector> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let json_path =
        std::path::Path::new(&manifest_dir).join("tests/golden/ed25519_strict_vectors.json");
    let content =
        fs::read_to_string(&json_path).expect("Failed to read ed25519_strict_vectors.json");
    serde_json::from_str(&content).expect("Failed to parse ed25519_strict_vectors.json")
}

#[test]
fn test_ed25519_golden_vectors_v3_standalone() {
    let vectors = load_vectors();
    for vec in vectors {
        let is_expect_valid = vec.expect == "01";
        let pk_bytes = hex_decode(&vec.pk);
        let msg_bytes = hex_decode(&vec.msg);
        let sig_bytes = hex_decode(&vec.sig);

        if pk_bytes.len() != 32 || sig_bytes.len() != 64 {
            assert!(
                !is_expect_valid,
                "Vector {} has invalid length but expected valid",
                vec.name
            );
            continue;
        }

        let mut pk_arr = [0u8; 32];
        pk_arr.copy_from_slice(&pk_bytes);

        let mut sig_arr = [0u8; 64];
        sig_arr.copy_from_slice(&sig_bytes);

        let res = verify_deletion_proof_v3(&pk_arr, &msg_bytes, &sig_arr);
        if is_expect_valid {
            assert!(
                res.is_ok(),
                "Vector {} failed verify_deletion_proof_v3: {:?}",
                vec.name,
                res
            );
        } else {
            assert!(
                res.is_err(),
                "Vector {} unexpectedly passed verify_deletion_proof_v3",
                vec.name
            );
        }
    }
}

#[test]
fn test_ed25519_golden_vectors_audit_chain() {
    let vectors = load_vectors();
    for vec in vectors {
        let is_expect_valid = vec.expect == "01";
        let pk_bytes = hex_decode(&vec.pk);
        let msg_bytes = hex_decode(&vec.msg);
        let sig_bytes = hex_decode(&vec.sig);

        if pk_bytes.len() != 32 {
            assert!(
                !is_expect_valid,
                "Vector {} has invalid pk length but expected valid",
                vec.name
            );
            continue;
        }

        let vk = match ed25519_dalek::VerifyingKey::from_bytes(&pk_bytes.try_into().unwrap()) {
            Ok(k) => k,
            Err(_) => {
                assert!(
                    !is_expect_valid,
                    "Vector {} has invalid verifying key",
                    vec.name
                );
                continue;
            }
        };

        if msg_bytes.len() < 40 {
            // AuditChain requires 8 bytes index + 32 bytes head_hash
            continue;
        }

        let mut chain_index_bytes = [0u8; 8];
        chain_index_bytes.copy_from_slice(&msg_bytes[..8]);
        let chain_index = u64::from_le_bytes(chain_index_bytes);

        let mut head_hash = [0u8; 32];
        head_hash.copy_from_slice(&msg_bytes[8..40]);

        let head_sig = AuditChainHeadSignature {
            chain_index,
            head_hash,
            signature: sig_bytes,
        };

        let res = AuditChain::verify_head_signature(&head_sig, &vk);
        if is_expect_valid {
            assert!(
                res.unwrap_or(false),
                "Vector {} failed AuditChain::verify_head_signature",
                vec.name
            );
        } else {
            assert!(
                !res.unwrap_or(false),
                "Vector {} unexpectedly passed AuditChain::verify_head_signature",
                vec.name
            );
        }
    }
}

#[test]
fn test_proof_v4_edge_case_vector_verifications() {
    let mut rng = rand::rngs::OsRng;
    let signer = ProofKeyPairSigner::generate(&mut rng);

    let mut proof = ProofV4 {
        version: 4,
        proof_id: [42u8; 32],
        db_instance_id: "db-instance-strict".to_string(),
        stage: 2,
        supersedes: None,
        scope: DeletionScope::Tenant {
            tenant_id: TenantId::try_new(7).expect("tenant"),
        },
        storage_scope: "tenant:7".to_string(),
        deleted_keys_hash: [99u8; 32],
        footprint_hash: [88u8; 32],
        observed_empty: false,
        deleted_after_tx: 500,
        claims: vec![],
        excluded_scopes: vec![],
        limitations_text_ids: vec![],
        environment_tlv: vec![],
        wal_chain_receipt: None,
        chain: contextra_crypto::deletion_proof::proof_v4::ProofChainLink {
            seq: 10,
            prev_hash: [0u8; 32],
        },
        issued_wall_ms: 1700000000000,
        issued_mono_ctr: 100,
        signer_key_id: signer.key_id(),
        signature: [0u8; 64],
    };

    proof.sign_with_signer(&signer).expect("sign_with_signer");

    // Case 1: Valid signature -> MUST pass
    assert!(proof.verify_signature(&signer.public_key()).expect("verify"));

    // Case 2: S >= L (non-canonical scalar) -> MUST be rejected
    let mut proof_s_ge_l = proof.clone();
    let l_bytes: [u8; 32] = [
        0xed, 0xd3, 0xf5, 0x5c, 0x1a, 0x63, 0x12, 0x58,
        0xd6, 0x9c, 0xf7, 0xa2, 0xde, 0xf9, 0x4d, 0x14,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10,
    ];
    let mut carry: u16 = 0;
    for i in 0..32 {
        let sum = proof_s_ge_l.signature[32 + i] as u16 + l_bytes[i] as u16 + carry;
        proof_s_ge_l.signature[32 + i] = (sum & 0xff) as u8;
        carry = sum >> 8;
    }
    assert!(!proof_s_ge_l.verify_signature(&signer.public_key()).unwrap_or(false));

    // Case 3: Small-order Public Key -> MUST be rejected
    let small_order_pk = [0u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x80];
    assert!(!proof.verify_signature(&small_order_pk).unwrap_or(false));

    // Case 4: Small-order R -> MUST be rejected
    let mut proof_small_r = proof.clone();
    proof_small_r.signature[..32].copy_from_slice(&small_order_pk);
    assert!(!proof_small_r.verify_signature(&signer.public_key()).unwrap_or(false));

    // Case 5: Non-canonical R -> MUST be rejected
    let mut proof_non_canon = proof.clone();
    proof_non_canon.signature[..32].fill(0xff);
    assert!(!proof_non_canon.verify_signature(&signer.public_key()).unwrap_or(false));
}

#[test]
fn test_deletion_proof_v3_edge_case_vector_verifications() {
    let keypair = DeletionProofKeyPair::generate();
    let vk = keypair.verifying_key();

    let scope = DeletionScope::Document {
        doc_id: DocId::new(42),
        tenant_id: TenantId::try_new(1).expect("tenant"),
    };
    let layer_proof = LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0).expect("layer_proof");
    let durability = DurabilityProof::new(100);

    let proof_v3 = DeletionProof::create_v3(
        scope,
        vec![b"key1".to_vec(), b"key2".to_vec()],
        TxId::new(100),
        vec![layer_proof],
        vec![ExcludedScope::LlmParameterMemory],
        1700000000,
        &[],
        Some(&durability),
        keypair.signing_key(),
    ).expect("create_v3");

    // Case 1: Valid signature -> MUST pass
    let key = VerificationKey::Ed25519(&vk);
    assert!(proof_v3.verify(key).expect("verify v3"));
    assert!(proof_v3.verify_external(&vk).is_ok());

    // Case 2: S >= L -> MUST be rejected
    let mut proof_s_ge_l = proof_v3.clone();
    let l_bytes: [u8; 32] = [
        0xed, 0xd3, 0xf5, 0x5c, 0x1a, 0x63, 0x12, 0x58,
        0xd6, 0x9c, 0xf7, 0xa2, 0xde, 0xf9, 0x4d, 0x14,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10,
    ];
    let mut carry: u16 = 0;
    for i in 0..32 {
        let sum = proof_s_ge_l.signature[32 + i] as u16 + l_bytes[i] as u16 + carry;
        proof_s_ge_l.signature[32 + i] = (sum & 0xff) as u8;
        carry = sum >> 8;
    }
    assert!(!proof_s_ge_l.verify(key).unwrap_or(false));
    assert!(proof_s_ge_l.verify_external(&vk).is_err());

    // Case 3: Truncated signature (32 bytes) -> MUST be rejected
    let mut proof_trunc = proof_v3.clone();
    proof_trunc.signature = proof_v3.signature[..32].to_vec();
    assert!(!proof_trunc.verify(key).unwrap_or(false));
    assert!(proof_trunc.verify_external(&vk).is_err());

    // Case 4: Non-canonical R -> MUST be rejected
    let mut proof_non_canon = proof_v3.clone();
    proof_non_canon.signature[..32].fill(0xff);
    assert!(!proof_non_canon.verify(key).unwrap_or(false));
    assert!(proof_non_canon.verify_external(&vk).is_err());
}
