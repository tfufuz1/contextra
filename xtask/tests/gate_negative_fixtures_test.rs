//! Integration test suite validating negative fixtures for all 11 security & quality gates.
//! Each test case constructs a synthetic negative fixture that violates a gate rule
//! and verifies that the corresponding gate logic detects the violation and returns Outcome::Fail.

use std::fs;
use tempfile::TempDir;
use xtask::core::gate::Outcome;
use xtask::gates::arch::async_gates::scan_async_io;
use xtask::gates::code::code_gates::{scan_code_lints, scan_determinism, scan_panic_surface};
use xtask::gates::gov::security_gates::{
    scan_key_purposes, scan_no_literal_secrets, scan_no_security_stub, scan_security_ports,
};
use xtask::gates::spec::deletion_gates::{
    scan_deletion_honesty, scan_deletion_types_once, scan_witness_mint,
};

fn create_test_root() -> TempDir {
    TempDir::new().expect("Failed to create temporary directory for negative fixtures")
}

#[test]
fn test_negative_fixture_no_security_stub() {
    let temp = create_test_root();
    let root = temp.path();

    let stub_code = r#"
        #[cfg(not(feature = "encryption-at-rest"))]
        pub fn verify_stub() -> Result<bool, String> {
            Ok(true)
        }
    "#;
    let file_path = root.join("crates/test-crate/src/stub_sample.rs");
    fs::create_dir_all(root.join("crates/test-crate/src")).unwrap();
    fs::write(&file_path, stub_code).unwrap();

    let outcome = scan_no_security_stub(root);
    match outcome {
        Outcome::Fail { reason } => {
            assert!(reason.contains("Security stub"));
        }
        other => panic!(
            "Expected Outcome::Fail for no-security-stub, got {:?}",
            other
        ),
    }
}

#[test]
fn test_negative_fixture_no_literal_secrets() {
    let temp = create_test_root();
    let root = temp.path();

    let secret_code = r#"
        pub const SECRET_KEY: &str = "default_test_deletion_proof_key_32_bytes!";
    "#;
    let file_path = root.join("crates/test-crate/src/secret_sample.rs");
    fs::create_dir_all(root.join("crates/test-crate/src")).unwrap();
    fs::write(&file_path, secret_code).unwrap();

    let outcome = scan_no_literal_secrets(root);
    match outcome {
        Outcome::Fail { reason } => {
            assert!(reason.contains("Hardcoded literal proof key"));
        }
        other => panic!(
            "Expected Outcome::Fail for no-literal-secrets, got {:?}",
            other
        ),
    }
}

#[test]
fn test_negative_fixture_security_ports() {
    let temp = create_test_root();
    let root = temp.path();

    let invalid_ports_toml = r#"
        [[port]]
        name = "KmsProvider"
        crate = "contextra-ports"
        trait_path = "contextra_ports::kms::KmsProvider"
        class = "core"
        prod_impl = "non_existent_crate::NonExistentKmsProvider"
        wiring_test = "ports/tests/missing_wiring.rs"
        wp = "WP 5.1"
        inv = "INV-WIRING-2"
    "#;
    let file_path = root.join("governance/security_ports.toml");
    fs::create_dir_all(root.join("governance")).unwrap();
    fs::write(&file_path, invalid_ports_toml).unwrap();

    let outcome = scan_security_ports(root);
    match outcome {
        Outcome::Fail { reason } => {
            assert!(reason.contains("does not exist"));
        }
        other => panic!("Expected Outcome::Fail for security-ports, got {:?}", other),
    }
}

#[test]
fn test_negative_fixture_key_purposes() {
    let temp = create_test_root();
    let root = temp.path();

    let key_purposes_toml = r#"
        [[purpose]]
        label = "contextra/v1/wal"
        purpose = "wal"
    "#;
    fs::create_dir_all(root.join("governance")).unwrap();
    fs::write(root.join("governance/key_purposes.toml"), key_purposes_toml).unwrap();

    let code_with_unregistered_label = r#"
        pub fn derive_key() {
            let label = "contextra/v1/unregistered_purpose_xyz";
        }
    "#;
    let file_path = root.join("crates/test-crate/src/crypto_sample.rs");
    fs::create_dir_all(root.join("crates/test-crate/src")).unwrap();
    fs::write(&file_path, code_with_unregistered_label).unwrap();

    let outcome = scan_key_purposes(root);
    match outcome {
        Outcome::Fail { reason } => {
            assert!(reason.contains("Unregistered HKDF key purpose"));
        }
        other => panic!("Expected Outcome::Fail for key-purposes, got {:?}", other),
    }
}

#[test]
fn test_negative_fixture_witness_mint() {
    let temp = create_test_root();
    let root = temp.path();

    let bad_mint_code = r#"
        pub fn illegal_mint() {
            let witness = LayerWitness { remaining_count: 0 };
        }
    "#;
    let file_path = root.join("crates/contextra-engine/src/illegal_mint.rs");
    fs::create_dir_all(root.join("crates/contextra-engine/src")).unwrap();
    fs::write(&file_path, bad_mint_code).unwrap();

    let outcome = scan_witness_mint(root);
    match outcome {
        Outcome::Fail { reason } => {
            assert!(reason.contains("LayerWitness"));
        }
        other => panic!("Expected Outcome::Fail for witness-mint, got {:?}", other),
    }
}

#[test]
fn test_negative_fixture_deletion_types_once() {
    let temp = create_test_root();
    let root = temp.path();

    let duplicate_type_code = r#"
        #[derive(Debug)]
        pub enum DeletionLayer {
            LsmMemtable,
            Sstable,
        }
    "#;
    let file_path = root.join("crates/contextra-engine/src/duplicate_layer.rs");
    fs::create_dir_all(root.join("crates/contextra-engine/src")).unwrap();
    fs::write(&file_path, duplicate_type_code).unwrap();

    let outcome = scan_deletion_types_once(root);
    match outcome {
        Outcome::Fail { reason } => {
            assert!(reason.contains("Duplicate definition"));
        }
        other => panic!(
            "Expected Outcome::Fail for deletion-types-once, got {:?}",
            other
        ),
    }
}

#[test]
fn test_negative_fixture_deletion_honesty() {
    let temp = create_test_root();
    let root = temp.path();

    let doc_claim =
        "# Contextra README\nDas System garantiert 100% gelöscht und ist DSGVO-konform.\n";
    let doc_path = root.join("README.md");
    fs::write(&doc_path, doc_claim).unwrap();

    let outcome = scan_deletion_honesty(root);
    match outcome {
        Outcome::Fail { reason } => {
            assert!(reason.contains("100% gelöscht"));
        }
        other => panic!(
            "Expected Outcome::Fail for deletion-honesty, got {:?}",
            other
        ),
    }
}

#[test]
fn test_negative_fixture_determinism() {
    let temp = create_test_root();
    let root = temp.path();

    let nondet_code = r#"
        pub fn get_timestamp() {
            let now = std::time::Instant::now();
        }
    "#;
    let file_path = root.join("crates/contextra-vector/src/hnsw.rs");
    fs::create_dir_all(root.join("crates/contextra-vector/src")).unwrap();
    fs::write(&file_path, nondet_code).unwrap();

    let outcome = scan_determinism(root);
    match outcome {
        Outcome::Fail { reason } => {
            assert!(reason.contains("Instant::now"));
        }
        other => panic!("Expected Outcome::Fail for determinism, got {:?}", other),
    }
}

#[test]
fn test_negative_fixture_panic_surface() {
    let temp = create_test_root();
    let root = temp.path();

    let panic_code = r#"
        pub fn do_indexing(slice: &[u8]) {
            let _ = slice.unwrap();
        }
    "#;
    let file_path = root.join("crates/test-crate/src/indexing.rs");
    fs::create_dir_all(root.join("crates/test-crate/src")).unwrap();
    fs::write(&file_path, panic_code).unwrap();

    let outcome = scan_panic_surface(root);
    match outcome {
        Outcome::Fail { reason } => {
            assert!(reason.contains("unwrap"));
        }
        other => panic!("Expected Outcome::Fail for panic-surface, got {:?}", other),
    }
}

#[test]
fn test_negative_fixture_code_lints() {
    let temp = create_test_root();
    let root = temp.path();

    let lint_code = r#"
        pub fn do_io() {
            let _ = file.read(&mut buf);
        }
    "#;
    let file_path = root.join("crates/test-crate/src/io.rs");
    fs::create_dir_all(root.join("crates/test-crate/src")).unwrap();
    fs::write(&file_path, lint_code).unwrap();

    let outcome = scan_code_lints(root);
    match outcome {
        Outcome::Fail { reason } => {
            assert!(reason.contains("Silent let _ = on Result"));
        }
        other => panic!("Expected Outcome::Fail for code-lints, got {:?}", other),
    }
}

#[test]
fn test_negative_fixture_async_io() {
    let temp = create_test_root();
    let root = temp.path();

    let async_io_code = r#"
        pub async fn read_data() {
            let data = std::fs::read("data.bin");
        }
    "#;
    let file_path = root.join("crates/test-crate/src/async_io.rs");
    fs::create_dir_all(root.join("crates/test-crate/src")).unwrap();
    fs::write(&file_path, async_io_code).unwrap();

    let outcome = scan_async_io(root);
    match outcome {
        Outcome::Fail { reason } => {
            assert!(reason.contains("Blocking I/O inside async fn"));
        }
        other => panic!("Expected Outcome::Fail for async-io, got {:?}", other),
    }
}
