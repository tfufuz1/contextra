use contextra_crypto::deletion_proof::keys::DeletionProofKeyPair;
use contextra_crypto::ed25519_proof::sign_deletion_proof_v3;
use contextra_mcp::cli::verify_proof::{execute_verify_proof_v4, hex_encode, VerifyProofOptions};
use serde_json::json;
use tempfile::tempdir;

#[test]
fn test_cli_verify_proof_legacy_without_flag_fails() {
    let tmp = tempdir().expect("tempdir");
    let keypair = DeletionProofKeyPair::generate();
    let payload = b"test_legacy_payload_v3";
    let sig = sign_deletion_proof_v3(&keypair, payload);

    let legacy_val = json!({
        "signature": hex_encode(&sig),
        "payload": hex_encode(payload),
        "version": 3
    });

    let proof_file = tmp.path().join("legacy_proof.json");
    std::fs::write(
        &proof_file,
        serde_json::to_string_pretty(&legacy_val).unwrap(),
    )
    .expect("write");

    let opts = VerifyProofOptions {
        proof_path_or_str: proof_file.to_str().unwrap().to_string(),
        pubkey_path_or_str: hex_encode(&keypair.verifying_key_bytes()),
        policy_path: None,
        chain_path: None,
        allow_legacy: false, // Opt-in disabled
    };

    let res = execute_verify_proof_v4(&opts);
    assert!(
        res.is_err(),
        "Legacy proof without --allow-legacy MUST be rejected"
    );
    let err = res.unwrap_err().to_string();
    assert!(
        err.contains("Legacy proof (v1-v3) rejected"),
        "Error message must indicate legacy rejection: {err}"
    );
}

#[test]
fn test_cli_verify_proof_legacy_with_flag_succeeds() {
    let tmp = tempdir().expect("tempdir");
    let keypair = DeletionProofKeyPair::generate();
    let payload = b"test_legacy_payload_v3";
    let sig = sign_deletion_proof_v3(&keypair, payload);

    let legacy_val = json!({
        "signature": hex_encode(&sig),
        "payload": hex_encode(payload),
        "version": 3
    });

    let proof_file = tmp.path().join("legacy_proof_valid.json");
    std::fs::write(
        &proof_file,
        serde_json::to_string_pretty(&legacy_val).unwrap(),
    )
    .expect("write");

    let opts = VerifyProofOptions {
        proof_path_or_str: proof_file.to_str().unwrap().to_string(),
        pubkey_path_or_str: hex_encode(&keypair.verifying_key_bytes()),
        policy_path: None,
        chain_path: None,
        allow_legacy: true, // Opt-in enabled
    };

    let res = execute_verify_proof_v4(&opts);
    assert!(
        res.is_ok(),
        "Legacy proof with --allow-legacy MUST succeed: {:?}",
        res
    );
    assert!(res.unwrap());
}
