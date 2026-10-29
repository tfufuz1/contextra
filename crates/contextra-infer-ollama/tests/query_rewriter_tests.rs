use contextra_infer_ollama::{OllamaClient, OllamaQueryRewriter, OllamaQueryRewriterConfig};
use contextra_ports::QueryRewriter;
use contextra_types::ScoredEntry;

#[tokio::test]
async fn test_empty_query_returns_empty() {
    let client = OllamaClient::new("http://127.0.0.1:1");
    let rewriter = OllamaQueryRewriter::new(client, OllamaQueryRewriterConfig::default());

    let results = rewriter.rewrite_structured("", &[]).await.unwrap();
    assert!(results.is_empty());

    let results_spaces = rewriter.rewrite_structured("   \n\t  ", &[]).await.unwrap();
    assert!(results_spaces.is_empty());
}

#[tokio::test]
async fn test_overlong_query_bounded() {
    let client = OllamaClient::new("http://127.0.0.1:1");
    let mut config = OllamaQueryRewriterConfig::default();
    config.max_query_chars = 10;
    let rewriter = OllamaQueryRewriter::new(client, config);

    // Network error on invalid endpoint will fall back to truncated query
    let results = rewriter
        .rewrite_structured("Münchener Oktoberfest 2026", &[])
        .await
        .unwrap();

    assert_eq!(results.len(), 1);
    if let Some(output) = results.get(0) {
        assert_eq!(output.text_query.as_deref(), Some("Münchener "));
    } else {
        panic!("expected 1 output");
    }
}

#[tokio::test]
async fn test_unicode_boundary_handling() {
    let client = OllamaClient::new("http://127.0.0.1:1");
    let mut config = OllamaQueryRewriterConfig::default();
    config.max_query_chars = 7;
    let rewriter = OllamaQueryRewriter::new(client, config);

    // "Präfix-🚀" -> 7 chars is "Präfix-"
    let results = rewriter
        .rewrite_structured("Präfix-🚀-Test", &[])
        .await
        .unwrap();

    assert_eq!(results.len(), 1);
    if let Some(output) = results.get(0) {
        assert_eq!(output.text_query.as_deref(), Some("Präfix-"));
    } else {
        panic!("expected 1 output");
    }
}

#[tokio::test]
async fn test_llm_network_error_fallback() {
    let client = OllamaClient::new("http://127.0.0.1:1");
    let rewriter = OllamaQueryRewriter::new(client, OllamaQueryRewriterConfig::default());

    let results = rewriter
        .rewrite_structured("rust async programming", &[])
        .await
        .unwrap();

    assert_eq!(results.len(), 1);
    if let Some(output) = results.get(0) {
        assert_eq!(output.text_query.as_deref(), Some("rust async programming"));
        assert!(output.semantic_query.is_none());
        assert!(output.anchor_entities.is_empty());
    } else {
        panic!("expected 1 output");
    }
}

#[tokio::test]
async fn test_mock_server_garbage_and_json_responses() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server_url = format!("http://{}", addr);

    tokio::spawn(async move {
        for i in 0..4 {
            if let Ok((mut socket, _)) = listener.accept().await {
                use tokio::io::{AsyncReadExt, AsyncWriteExt};
                let mut buf = [0u8; 4096];
                let n = socket.read(&mut buf).await.unwrap_or(0);
                let req_bytes = buf.get(..n).unwrap_or(&[]);
                let _req_str = String::from_utf8_lossy(req_bytes);

                let response_content = match i {
                    0 => "```json\n{\n  \"text_query\": \"rust async tokio\",\n  \"semantic_query\": \"tokio task execution in rust\",\n  \"anchor_entities\": [\"Rust\", \"Tokio\"]\n}\n```".to_string(),
                    1 => "1. rust concurrency\n2. channels in rust\n- Mutex\n- Condvar".to_string(),
                    2 => "!@#$%^&* NOT VALID JSON OR TEXT !@#$%^&*".to_string(),
                    _ => "".to_string(),
                };

                let body = serde_json::json!({
                    "message": {
                        "role": "assistant",
                        "content": response_content
                    }
                })
                .to_string();

                let http_response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                    body.len(),
                    body
                );
                let _ = socket.write_all(http_response.as_bytes()).await;
            }
        }
    });

    let client = OllamaClient::new(server_url);
    let rewriter = OllamaQueryRewriter::new(client, OllamaQueryRewriterConfig::default());

    let entry = ScoredEntry {
        id: "doc1".into(),
        final_score: 0.9,
        metadata: Some(serde_json::json!({"content": "concurrency in rust"})),
    };

    // Call 0: JSON with code fences
    let res0 = rewriter
        .rewrite_structured("rust async", &[entry.clone()])
        .await
        .unwrap();
    if let Some(out) = res0.get(0) {
        assert_eq!(out.text_query.as_deref(), Some("rust async tokio"));
        assert_eq!(
            out.semantic_query.as_deref(),
            Some("tokio task execution in rust")
        );
        assert_eq!(out.anchor_entities, vec!["Rust", "Tokio"]);
    } else {
        panic!("expected output 0");
    }

    // Call 1: Plain text bullet fallback
    let res1 = rewriter
        .rewrite_structured("rust async", &[entry.clone()])
        .await
        .unwrap();
    if let Some(out) = res1.get(0) {
        assert_eq!(out.text_query.as_deref(), Some("rust concurrency"));
        assert_eq!(out.semantic_query.as_deref(), Some("channels in rust"));
        assert_eq!(out.anchor_entities, vec!["Mutex", "Condvar"]);
    } else {
        panic!("expected output 1");
    }

    // Call 2: Garbage output -> fallback to original query
    let res2 = rewriter
        .rewrite_structured("rust async", &[entry.clone()])
        .await
        .unwrap();
    if let Some(out) = res2.get(0) {
        assert_eq!(out.text_query.as_deref(), Some("rust async"));
    } else {
        panic!("expected output 2");
    }

    // Call 3: Empty response -> fallback to original query
    let res3 = rewriter
        .rewrite_structured("rust async", &[entry.clone()])
        .await
        .unwrap();
    if let Some(out) = res3.get(0) {
        assert_eq!(out.text_query.as_deref(), Some("rust async"));
    } else {
        panic!("expected output 3");
    }
}
