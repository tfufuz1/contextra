use contextra_crypto::deletion_proof::builder::ProofBuilder;
use contextra_crypto::deletion_proof::proof_store::ProofStore;
use contextra_crypto::deletion_proof::proof_v4::ProofClaimV4;
use contextra_crypto::deletion_proof::signer::{ProofKeyPairSigner, ProofSigner};
use contextra_crypto::deletion_proof::types::DeletionScope;
use contextra_mcp::cli::verify_proof::{execute_verify_proof_v4, hex_encode, VerifyProofOptions};
use contextra_types::{DocId, TenantId};
use rand::rngs::OsRng;
use tempfile::tempdir;

fn create_valid_v4_proof(
    signer: &ProofKeyPairSigner,
) -> contextra_crypto::deletion_proof::proof_v4::ProofV4 {
    let scope = DeletionScope::Document {
        doc_id: DocId(1),
        tenant_id: TenantId::try_new(1).expect("tenant"),
    };
    let mut proof = ProofBuilder::new(scope, "db-v4-test")
        .with_deleted_after_tx(10)
        .with_deleted_keys_hash([0x11; 32])
        .finish(signer)
        .expect("builder finish");

    proof.claims = vec![ProofClaimV4 {
        layer_id: 1,
        assurance: 2,
        claim_text_id: 3,
        witness_digest: [0x22; 32],
        scanned_at_tx: 10,
        evidence_tlv: vec![],
    }];
    proof.excluded_scopes = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
    proof.sign_with_signer(signer).expect("sign failed");
    proof
}

#[test]
fn test_cli_verify_proof_v4_valid_returns_ok() {
    let tmp = tempdir().expect("tempdir");
    let signer = ProofKeyPairSigner::generate(&mut OsRng);
    let proof = create_valid_v4_proof(&signer);

    let proof_json = serde_json::to_string_pretty(&proof).expect("json");
    let proof_file = tmp.path().join("proof.json");
    std::fs::write(&proof_file, proof_json).expect("write proof");

    let pubkey_hex = hex_encode(&signer.public_key());

    let opts = VerifyProofOptions {
        proof_path_or_str: proof_file.to_str().unwrap().to_string(),
        pubkey_path_or_str: pubkey_hex,
        policy_path: None,
        chain_path: None,
        allow_legacy: false,
    };

    let res = execute_verify_proof_v4(&opts);
    assert!(
        res.is_ok(),
        "Valid v4 proof MUST pass verification: {:?}",
        res
    );
    assert!(res.unwrap());
}

#[test]
fn test_cli_verify_proof_v4_tampered_byte_fails() {
    let tmp = tempdir().expect("tempdir");
    let signer = ProofKeyPairSigner::generate(&mut OsRng);
    let mut proof = create_valid_v4_proof(&signer);

    // Tamper with deleted_after_tx after signing
    proof.deleted_after_tx = 999;

    let proof_json = serde_json::to_string_pretty(&proof).expect("json");
    let proof_file = tmp.path().join("proof_tampered.json");
    std::fs::write(&proof_file, proof_json).expect("write proof");

    let pubkey_hex = hex_encode(&signer.public_key());

    let opts = VerifyProofOptions {
        proof_path_or_str: proof_file.to_str().unwrap().to_string(),
        pubkey_path_or_str: pubkey_hex,
        policy_path: None,
        chain_path: None,
        allow_legacy: false,
    };

    let res = execute_verify_proof_v4(&opts);
    assert!(res.is_err(), "Tampered v4 proof MUST fail verification");
    let err = res.unwrap_err().to_string();
    assert!(
        err.contains("R1") || err.contains("Policy"),
        "Error must cite rule ID R1 or Policy: {err}"
    );
}

#[test]
fn test_cli_verify_proof_v4_missing_required_layers_fails_r3_r12() {
    let tmp = tempdir().expect("tempdir");
    let signer = ProofKeyPairSigner::generate(&mut OsRng);
    let mut proof = create_valid_v4_proof(&signer);

    // Remove claims and zero deleted_keys_hash without observed_empty -> triggers R12 failure
    proof.claims.clear();
    proof.deleted_keys_hash = [0u8; 32];
    proof.observed_empty = false;
    proof.sign_with_signer(&signer).expect("sign failed");

    let proof_json = serde_json::to_string_pretty(&proof).expect("json");
    let proof_file = tmp.path().join("proof_no_layers.json");
    std::fs::write(&proof_file, proof_json).expect("write proof");

    let pubkey_hex = hex_encode(&signer.public_key());

    let opts = VerifyProofOptions {
        proof_path_or_str: proof_file.to_str().unwrap().to_string(),
        pubkey_path_or_str: pubkey_hex,
        policy_path: None,
        chain_path: None,
        allow_legacy: false,
    };

    let res = execute_verify_proof_v4(&opts);
    assert!(
        res.is_err(),
        "Proof without required layers MUST fail policy"
    );
    let err = res.unwrap_err().to_string();
    assert!(
        err.contains("R12") || err.contains("Policy"),
        "Error must cite rule ID R12: {err}"
    );
}

#[test]
fn test_cli_verify_proof_v4_chain_gap_fails_r7() {
    let tmp = tempdir().expect("tempdir");
    let signer = ProofKeyPairSigner::generate(&mut OsRng);

    let (mut store, _) = ProofStore::open(tmp.path()).expect("open store");

    let proof1 = create_valid_v4_proof(&signer);
    store.save_proof(&proof1).expect("save 1");

    // Manually break chain.log
    let chain_log_path = tmp.path().join("chain.log");
    let corrupted_log = "0000000000000001 | 0101010101010101010101010101010101010101010101010101010101010101 | 0000000000000000000000000000000000000000000000000000000000000000\n\
                         0000000000000003 | 0202020202020202020202020202020202020202020202020202020202020202 | 0000000000000000000000000000000000000000000000000000000000000000\n";
    std::fs::write(&chain_log_path, corrupted_log).expect("corrupt log");

    let proof_file = tmp.path().join("proof.json");
    std::fs::write(&proof_file, serde_json::to_string_pretty(&proof1).unwrap()).expect("write");

    let opts = VerifyProofOptions {
        proof_path_or_str: proof_file.to_str().unwrap().to_string(),
        pubkey_path_or_str: hex_encode(&signer.public_key()),
        policy_path: None,
        chain_path: Some(tmp.path().to_str().unwrap().to_string()),
        allow_legacy: false,
    };

    let res = execute_verify_proof_v4(&opts);
    assert!(res.is_err(), "Chain gap MUST fail verification");
    let err = res.unwrap_err().to_string();
    assert!(err.contains("R7"), "Error must cite R7: {err}");
}
