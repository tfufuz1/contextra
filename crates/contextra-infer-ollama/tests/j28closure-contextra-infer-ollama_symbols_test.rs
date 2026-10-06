// ZWECK: J28 Closure Test for contextra-infer-ollama 5 symbols verification.
// SYMBOLE:
// 1. generate_prefix_batch (context_prefixer.rs)
// 2. with_concurrency (embedding.rs)
// 3. is_parsed (importance.rs)
// 4. with_provenance (importance.rs)
// 5. validate_model_available (model_info.rs)

use contextra_infer_ollama::{
    Confidence, ContextPrefixConfig, ContextPrefixEngine, ImportanceAssessment, OllamaClient,
    OllamaEmbedder,
};
use contextra_types::ImportanceScore;

#[tokio::test]
async fn test_j28closure_all_5_symbols_verification() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Setup mock HTTP server for Ollama API endpoints
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let server_url = format!("http://{}", addr);

    tokio::spawn(async move {
        while let Ok((mut socket, _)) = listener.accept().await {
            use tokio::io::{AsyncReadExt, AsyncWriteExt};
            let mut buf = [0u8; 4096];
            let n = socket.read(&mut buf).await.unwrap_or(0);
            let req_str = String::from_utf8_lossy(&buf[..n]);

            let response_body = if req_str.contains("GET /api/tags") {
                serde_json::json!({
                    "models": [
                        { "name": "llama3:latest" },
                        { "name": "nomic-embed-text:latest" }
                    ]
                })
                .to_string()
            } else if req_str.contains("POST /api/chat") {
                serde_json::json!({
                    "message": {
                        "role": "assistant",
                        "content": "Dies ist ein Kontext-Präfix."
                    }
                })
                .to_string()
            } else {
                serde_json::json!({ "status": "ok" }).to_string()
            };

            let response = format!(
                "HTTP/1.1 200 OK\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                response_body.len(),
                response_body
            );
            socket.write_all(response.as_bytes()).await.ok();
        }
    });

    let client = OllamaClient::new(&server_url);

    // Symbol 5: validate_model_available
    client.validate_model_available("llama3").await?;
    let missing_res = client.validate_model_available("missing-model").await;
    assert!(
        missing_res.is_err(),
        "validate_model_available must return Err for missing model"
    );

    // Symbol 1: generate_prefix_batch
    let prefix_engine = ContextPrefixEngine::new(client.clone(), ContextPrefixConfig::default());
    let chunks = vec!["Abschnitt 1", "Abschnitt 2"];
    let prefixes = prefix_engine
        .generate_prefix_batch("Vollständiges Dokument", &chunks)
        .await;
    assert_eq!(prefixes.len(), 2);
    assert_eq!(
        prefixes[0].as_ref().unwrap(),
        "Dies ist ein Kontext-Präfix."
    );
    assert_eq!(
        prefixes[1].as_ref().unwrap(),
        "Dies ist ein Kontext-Präfix."
    );

    // Symbol 2: with_concurrency
    let embedder = OllamaEmbedder::new(&server_url, "nomic-embed-text").with_concurrency(16);
    assert_eq!(embedder.model(), "nomic-embed-text");

    // Symbol 3 & 4: with_provenance & is_parsed
    let assessment_parsed = ImportanceAssessment::with_provenance(
        ImportanceScore::new(0.85),
        Confidence::Parsed,
        "llama3:latest",
        Some(0.88),
    );
    assert!(
        assessment_parsed.is_parsed(),
        "is_parsed must return true when Confidence::Parsed"
    );
    assert_eq!(assessment_parsed.model_id, "llama3:latest");
    assert_eq!(assessment_parsed.calibrated_confidence, Some(0.88));

    let assessment_unparseable = ImportanceAssessment::with_provenance(
        ImportanceScore::default(),
        Confidence::Unparseable,
        "llama3:latest",
        None,
    );
    assert!(
        !assessment_unparseable.is_parsed(),
        "is_parsed must return false when Confidence::Unparseable"
    );

    Ok(())
}
