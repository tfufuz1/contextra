use contextra_mcp::config::{create_embedding_provider, create_llm_text_generator};

#[cfg(not(feature = "test-utils"))]
use contextra_types::ContextraError;

#[cfg(not(feature = "test-utils"))]
#[test]
fn release_build_rejects_mock_provider() {
    let embed_res = create_embedding_provider(
        "mock",
        "http://localhost:11434",
        "nomic-embed-text",
        None,
        None,
    );
    match embed_res {
        Err(ContextraError::InvalidInput(msg)) => {
            assert!(
                msg.contains("Unknown embedding provider 'mock'"),
                "Expected unknown provider error, got: {msg}"
            );
        }
        Ok(_) => panic!("Expected InvalidInput error when 'mock' provider is used without test-utils feature, got Ok"),
        Err(other) => panic!("Expected InvalidInput error when 'mock' provider is used without test-utils feature, got error: {other}"),
    }

    let llm_res = create_llm_text_generator(
        "mock",
        "http://localhost:11434",
        "llama3.2:3b",
        None,
    );
    match llm_res {
        Err(ContextraError::InvalidInput(msg)) => {
            assert!(
                msg.contains("Unknown LLM provider 'mock'"),
                "Expected unknown provider error, got: {msg}"
            );
        }
        Ok(_) => panic!("Expected InvalidInput error when 'mock' provider is used without test-utils feature, got Ok"),
        Err(other) => panic!("Expected InvalidInput error when 'mock' provider is used without test-utils feature, got error: {other}"),
    }
}

#[cfg(feature = "test-utils")]
#[test]
fn test_utils_allows_mock_provider() {
    let embed_res = create_embedding_provider(
        "mock",
        "http://localhost:11434",
        "nomic-embed-text",
        None,
        None,
    );
    assert!(embed_res.is_ok(), "Expected mock embedding provider to succeed when test-utils feature is enabled");

    let llm_res = create_llm_text_generator(
        "mock",
        "http://localhost:11434",
        "llama3.2:3b",
        None,
    );
    assert!(llm_res.is_ok(), "Expected mock LLM generator to succeed when test-utils feature is enabled");
}
