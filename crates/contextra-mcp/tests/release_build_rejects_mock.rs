use contextra::EmbeddingBackend;
use contextra_mcp::config::{
    create_embedding_provider, create_llm_text_generator, EmbeddingConfig,
};

#[cfg(not(feature = "test-utils"))]
use contextra_types::ContextraError;

#[cfg(not(feature = "test-utils"))]
#[test]
fn release_build_rejects_mock_provider() {
    let config = EmbeddingConfig {
        provider: "mock".into(),
        ..Default::default()
    };
    let embed_res = config.build_provider();
    match embed_res {
        Err(ContextraError::InvalidInput(_)) => {}
        Ok(_) => panic!("Expected InvalidInput error when 'mock' provider is used without test-utils feature, got Ok"),
        Err(other) => panic!("Expected InvalidInput error when 'mock' provider is used without test-utils feature, got error: {other}"),
    }

    let llm_res = create_llm_text_generator("mock", "http://localhost:11434", "llama3.2:3b", None);
    match llm_res {
        Err(ContextraError::InvalidInput(_)) => {}
        Ok(_) => panic!("Expected InvalidInput error when 'mock' LLM provider is used without test-utils feature, got Ok"),
        Err(other) => panic!("Expected InvalidInput error when 'mock' LLM provider is used without test-utils feature, got error: {other}"),
    }

    let none_res = create_embedding_provider(&EmbeddingBackend::None);
    match none_res {
        Err(ContextraError::InvalidInput(_)) => {}
        Ok(_) => panic!("Expected InvalidInput error when EmbeddingBackend::None is used in release build, got Ok"),
        Err(other) => panic!("Expected InvalidInput error when EmbeddingBackend::None is used in release build, got error: {other}"),
    }
}

#[cfg(feature = "test-utils")]
#[test]
fn test_utils_allows_mock_provider() {
    let config = EmbeddingConfig {
        provider: "mock".into(),
        ..Default::default()
    };
    let embed_res = config.build_provider();
    assert!(
        embed_res.is_ok(),
        "Expected mock embedding provider to succeed when test-utils feature is enabled"
    );

    let llm_res = create_llm_text_generator("mock", "http://localhost:11434", "llama3.2:3b", None);
    assert!(
        llm_res.is_ok(),
        "Expected mock LLM generator to succeed when test-utils feature is enabled"
    );

    let none_res = create_embedding_provider(&EmbeddingBackend::None);
    assert!(
        none_res.is_ok(),
        "Expected EmbeddingBackend::None to succeed when test-utils feature is enabled"
    );
}
