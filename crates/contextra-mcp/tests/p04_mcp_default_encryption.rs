use tempfile::TempDir;

#[test]
fn test_mcp_encryption_at_rest_default_active() {
    let _tmp = TempDir::new().expect("temp dir");

    // Clear env vars for test isolation
    std::env::remove_var("CONTEXTRA_MCP_ENCRYPTION_PASS");
    std::env::remove_var("CONTEXTRA_ENCRYPTION_PASSPHRASE");

    let cfg = contextra::ContextraConfig::default();
    assert!(cfg.encryption_passphrase().is_some());
}
