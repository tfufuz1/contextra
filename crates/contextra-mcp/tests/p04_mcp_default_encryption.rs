#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_mcp::resolve_mcp_encryption_config;
use tempfile::TempDir;

#[test]
fn test_mcp_encryption_at_rest_default_active() {
    let tmp = TempDir::new().expect("temp dir");

    // Clear env vars for test isolation
    std::env::remove_var("CONTEXTRA_ENCRYPTION_PASSPHRASE");
    std::env::remove_var("CONTEXTRA_PASSPHRASE");
    std::env::remove_var("CONTEXTRA_DISABLE_ENCRYPTION");

    // Default resolution without args must return an active passphrase
    let (passphrase, disabled) = resolve_mcp_encryption_config(tmp.path(), &[]);
    assert!(
        !disabled,
        "Encryption at rest must NOT be disabled by default"
    );
    assert!(
        passphrase.is_some(),
        "Encryption at rest must generate or load an active passphrase by default"
    );

    let key_file = tmp.path().join(".contextra_master.key");
    assert!(
        key_file.exists(),
        "Master keyfile .contextra_master.key must be created in db_path"
    );

    // Repeated resolution loads existing keyfile
    let (passphrase2, _) = resolve_mcp_encryption_config(tmp.path(), &[]);
    assert_eq!(passphrase, passphrase2);
}

#[test]
fn test_mcp_encryption_at_rest_opt_out() {
    let tmp = TempDir::new().expect("temp dir");

    let args = vec!["--disable-encryption-at-rest".to_string()];
    let (passphrase, disabled) = resolve_mcp_encryption_config(tmp.path(), &args);

    assert!(disabled, "Opt-out flag must disable encryption at rest");
    assert!(
        passphrase.is_none(),
        "Opt-out must result in None passphrase"
    );
}
