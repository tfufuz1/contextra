#![allow(clippy::unwrap_used, clippy::expect_used)]

use contextra_infer_ollama::{
    score_importance, score_importance_with_calibrator, Confidence, ContextPrefixConfig,
    ContextPrefixEngine, ImportanceAssessment, OllamaClient, OllamaEmbedder,
};
use contextra_types::ImportanceScore;

#[tokio::test]
async fn test_wire_generate_prefix_batch() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server_url = format!("http://{}", addr);

    tokio::spawn(async move {
        while let Ok((mut socket, _)) = listener.accept().await {
            use tokio::io::{AsyncReadExt, AsyncWriteExt};
            let mut buf = [0u8; 4096];
            let _ = socket.read(&mut buf).await;
            let body = serde_json::json!({
                "message": {
                    "role": "assistant",
                    "content": "Batch-Präfix"
                }
            })
            .to_string();
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                body.len(),
                body
            );
            socket.write_all(response.as_bytes()).await.ok();
        }
    });

    let client = OllamaClient::new(server_url);
    let engine = ContextPrefixEngine::new(client, ContextPrefixConfig::default());
    let chunks = vec!["Chunk A", "Chunk B"];
    let results = engine
        .generate_prefix_batch("Volltext Dokument", &chunks)
        .await;

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].as_ref().unwrap(), "Batch-Präfix");
    assert_eq!(results[1].as_ref().unwrap(), "Batch-Präfix");
}

#[test]
fn test_wire_embedder_with_concurrency() {
    let embedder = OllamaEmbedder::with_defaults().with_concurrency(16);
    assert_eq!(embedder.model(), "nomic-embed-text");
    assert_eq!(embedder.config().base_url, "http://localhost:11434");
}

#[tokio::test]
async fn test_wire_importance_assessment_is_parsed_and_with_provenance() {
    let assessment = ImportanceAssessment::with_provenance(
        ImportanceScore::new(0.95),
        Confidence::Parsed,
        "llama3:8b",
        Some(0.92),
    );

    assert!(assessment.is_parsed());
    assert_eq!(assessment.value(), 0.95);
    assert_eq!(assessment.model_id, "llama3:8b");
    assert_eq!(assessment.calibrated_confidence, Some(0.92));

    // Test production call through score_importance_with_calibrator
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server_url = format!("http://{}", addr);

    tokio::spawn(async move {
        if let Ok((mut socket, _)) = listener.accept().await {
            use tokio::io::{AsyncReadExt, AsyncWriteExt};
            let mut buf = [0u8; 4096];
            let _ = socket.read(&mut buf).await;
            let body = serde_json::json!({
                "message": {
                    "role": "assistant",
                    "content": "0.85"
                }
            })
            .to_string();
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                body.len(),
                body
            );
            socket.write_all(response.as_bytes()).await.ok();
        }
    });

    let client = OllamaClient::new(server_url);
    let result = score_importance_with_calibrator(&client, "Valid chunk", None)
        .await
        .unwrap();

    assert!(result.is_parsed());
    assert_eq!(result.value(), 0.85);

    // Unparseable case checks is_parsed() returning false
    let listener_unparseable = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr_unparseable = listener_unparseable.local_addr().unwrap();
    let server_url_unparseable = format!("http://{}", addr_unparseable);

    tokio::spawn(async move {
        if let Ok((mut socket, _)) = listener_unparseable.accept().await {
            use tokio::io::{AsyncReadExt, AsyncWriteExt};
            let mut buf = [0u8; 4096];
            let _ = socket.read(&mut buf).await;
            let body = serde_json::json!({
                "message": {
                    "role": "assistant",
                    "content": "not a number"
                }
            })
            .to_string();
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                body.len(),
                body
            );
            socket.write_all(response.as_bytes()).await.ok();
        }
    });

    let client_unparseable = OllamaClient::new(server_url_unparseable);
    let result_unparseable = score_importance(&client_unparseable, "Valid chunk")
        .await
        .unwrap();

    assert!(!result_unparseable.is_parsed());
}

#[tokio::test]
async fn test_wire_validate_model_available() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server_url = format!("http://{}", addr);

    tokio::spawn(async move {
        while let Ok((mut socket, _)) = listener.accept().await {
            use tokio::io::{AsyncReadExt, AsyncWriteExt};
            let mut buf = [0u8; 1024];
            let _ = socket.read(&mut buf).await;
            let body = serde_json::json!({
                "models": [
                    { "name": "llama3:latest" }
                ]
            })
            .to_string();
            let response = format!(
                "HTTP/1.1 200 OK\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                body.len(),
                body
            );
            socket.write_all(response.as_bytes()).await.ok();
        }
    });

    let client = OllamaClient::new(server_url);

    // Direct invocation on client (model_info.rs impl OllamaClient)
    assert!(client.validate_model_available("llama3").await.is_ok());

    // Indirect invocation via ensure_model_available
    assert!(client.ensure_model_available("llama3").await.is_ok());
}
