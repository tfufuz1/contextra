use contextra_mcp::config::EmbeddingConfig;
use contextra_types::ContextraError;
use std::sync::Mutex;

static ENV_MUTEX: Mutex<()> = Mutex::new(());

#[test]
fn test_no_env_vars_fails_closed() {
    let _guard = ENV_MUTEX.lock().expect("mutex lock");
    std::env::remove_var("CONTEXTRA_EMBEDDING_PROVIDER");
    std::env::remove_var("EMBEDDING_PROVIDER");

    let config = EmbeddingConfig::from_env();
    let res = config.build_provider();
    assert!(
        res.is_err(),
        "Expected build_provider to fail when no embedding provider is configured"
    );
    match res {
        Err(ContextraError::InvalidInput(msg)) => {
            assert!(
                msg.contains("CONTEXTRA_EMBEDDING_PROVIDER"),
                "Error message must mention CONTEXTRA_EMBEDDING_PROVIDER, got: {msg}"
            );
            assert!(
                msg.contains("EMBEDDING_PROVIDER"),
                "Error message must mention EMBEDDING_PROVIDER, got: {msg}"
            );
            assert!(
                msg.contains("mock"),
                "Error message must mention allowed value 'mock', got: {msg}"
            );
        }
        Err(other) => panic!("Expected ContextraError::InvalidInput, got: {other}"),
        Ok(_) => panic!("Expected build_provider to fail when no embedding provider is configured"),
    }
}

#[test]
fn test_explicit_mock_provider_succeeds() {
    let _guard = ENV_MUTEX.lock().expect("mutex lock");
    std::env::set_var("CONTEXTRA_EMBEDDING_PROVIDER", "mock");
    std::env::remove_var("EMBEDDING_PROVIDER");

    let config = EmbeddingConfig::from_env();
    assert_eq!(config.provider, "mock");
    let res = config.build_provider();
    assert!(
        res.is_ok(),
        "Explicit mock provider should build successfully"
    );

    std::env::remove_var("CONTEXTRA_EMBEDDING_PROVIDER");
}
