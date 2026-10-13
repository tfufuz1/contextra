// FILE-CONTEXT
// STAND:       2026-10-11
// ZWECK:       Integration test suite for MCP CLI commands (init, doctor, verify-proof, confirm).

use contextra_mcp::cli::doctor::{DoctorCheckStatus, DoctorChecker, SystemDoctor};
use contextra_mcp::cli::{execute_doctor, execute_init, execute_verify_proof, InitOptions};
use rand::SeedableRng;
use std::path::Path;
use tempfile::tempdir;

#[test]
fn test_cli_init_requires_key_file_unattended() {
    let tmp = tempdir().unwrap();
    let opts = InitOptions {
        db_path: tmp.path().to_path_buf(),
        key_file: None,
    };

    // Unattended init without key file or keyring must fail closed (INV-KDF-1)
    let res = execute_init(&opts);
    assert!(res.is_err());
    assert!(res.unwrap_err().to_string().contains("INV-KDF-1"));

    // Init with explicit key file succeeds
    let key_file_path = tmp.path().join("master.key");
    let opts2 = InitOptions {
        db_path: tmp.path().to_path_buf(),
        key_file: Some(key_file_path.clone()),
    };
    assert!(execute_init(&opts2).is_ok());
    assert!(key_file_path.exists());
    assert_eq!(std::fs::metadata(&key_file_path).unwrap().len(), 32);
}

struct UncheckableChecker;
impl DoctorChecker for UncheckableChecker {
    fn name(&self) -> &'static str {
        "MockUncheckable"
    }
    fn check(&self, _db_path: &Path) -> DoctorCheckStatus {
        DoctorCheckStatus::NotCheckable(
            "Component not checkable without active runtime".to_string(),
        )
    }
}

#[test]
fn test_cli_doctor_outputs_not_checkable() {
    let tmp = tempdir().unwrap();
    let doctor = SystemDoctor::new().with_checker(Box::new(UncheckableChecker));

    let results = doctor.run_diagnostics(tmp.path());
    let mock_res = results.iter().find(|(name, _)| *name == "MockUncheckable");
    assert!(mock_res.is_some());

    match &mock_res.unwrap().1 {
        DoctorCheckStatus::NotCheckable(msg) => {
            assert!(msg.contains("not checkable"));
        }
        other => panic!("Expected NotCheckable, got {:?}", other),
    }

    assert!(execute_doctor(tmp.path()).is_ok());
}

#[test]
fn test_cli_verify_proof_offline() {
    // Generate valid Ed25519 keypair and proof payload
    let mut rng = rand::rngs::StdRng::seed_from_u64(12345);
    let keypair = contextra_crypto::ed25519_proof::DeletionProofKeyPair::generate(&mut rng);

    let payload = b"canonical_proof_payload_bytes";
    let sig = contextra_crypto::ed25519_proof::sign_deletion_proof_v3(&keypair, payload);

    let mut sig_hex = String::new();
    for byte in sig {
        sig_hex.push_str(&format!("{byte:02x}"));
    }

    let mut payload_hex = String::new();
    for byte in payload {
        payload_hex.push_str(&format!("{byte:02x}"));
    }

    let proof_json = serde_json::json!({
        "signature": sig_hex,
        "payload": payload_hex,
    })
    .to_string();

    let vk_bytes = keypair.verifying_key_bytes();

    // Verify valid proof
    let res = execute_verify_proof(&proof_json, &vk_bytes);
    assert!(res.is_ok());

    // Tampered public key fails
    let mut wrong_vk = vk_bytes;
    wrong_vk[0] ^= 0xFF;
    let res_tampered = execute_verify_proof(&proof_json, &wrong_vk);
    assert!(res_tampered.is_err());
}
