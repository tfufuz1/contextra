#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_mcp::config::{create_embedding_provider, create_llm_text_generator, EmbeddingBackend};

#[test]
fn test_mock_providers_controlled_by_cfg() {
    let none_res = create_embedding_provider(&EmbeddingBackend::None);
    let mock_llm_res = create_llm_text_generator("mock");

    #[cfg(any(test, debug_assertions, feature = "test-utils"))]
    {
        assert!(none_res.is_ok());
        assert!(mock_llm_res.is_ok());
    }

    #[cfg(not(any(test, debug_assertions, feature = "test-utils")))]
    {
        assert!(none_res.is_err());
        assert!(mock_llm_res.is_err());
    }
}
