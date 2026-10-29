#![allow(clippy::expect_used, clippy::unwrap_used)]

use contextra_infer_ollama::query_rewriter::{OllamaQueryRewriter, OllamaQueryRewriterConfig};
use contextra_infer_ollama::OllamaClient;
use contextra_ports::query_rewriter::QueryRewriter;

#[tokio::test]
async fn test_empty_and_whitespace_query() {
    let client = OllamaClient::new("http://127.0.0.1:11434");
    let rewriter = OllamaQueryRewriter::new(client, OllamaQueryRewriterConfig::default());

    let res_empty = rewriter.rewrite_structured("", &[]).await;
    assert!(res_empty.is_ok());
    if let Ok(vec) = res_empty {
        assert!(vec.is_empty());
    }

    let res_ws = rewriter.rewrite_structured("   \n\t  ", &[]).await;
    assert!(res_ws.is_ok());
    if let Ok(vec) = res_ws {
        assert!(vec.is_empty());
    }
}

#[tokio::test]
async fn test_overlong_query_and_unicode_safety() {
    let client = OllamaClient::new("http://127.0.0.1:11434");
    let config = OllamaQueryRewriterConfig {
        max_query_chars: 10,
        ..Default::default()
    };
    let rewriter = OllamaQueryRewriter::new(client, config);

    let results = rewriter.rewrite_structured("Präfix-🚀-Test", &[]).await;
    assert!(results.is_ok());
    if let Ok(res) = results {
        let output = res.first().expect("expected 1 output");
        assert_eq!(output.text_query.as_deref(), Some("Präfix-🚀-T"));
    }
}

#[tokio::test]
async fn test_fallback_on_network_error() {
    let client = OllamaClient::new("http://127.0.0.1:1");
    let rewriter = OllamaQueryRewriter::new(client, OllamaQueryRewriterConfig::default());

    let results = rewriter
        .rewrite_structured("rust async programming", &[])
        .await;
    assert!(results.is_ok());
    if let Ok(res) = results {
        let output = res.first().expect("expected 1 output");
        assert_eq!(output.text_query.as_deref(), Some("rust async programming"));
        assert!(output.anchor_entities.is_empty());
    }
}
