#![cfg(feature = "client")]

use contextra_infer_ollama::{OllamaClient, OllamaConfig, OllamaError};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::test]
async fn test_fake_server_success_embed() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let url = format!("http://{}", addr);

    tokio::spawn(async move {
        if let Ok((mut socket, _)) = listener.accept().await {
            let mut buf = [0u8; 1024];
            let _ = socket.read(&mut buf).await;
            let body = serde_json::json!({
                "embedding": [0.1, 0.2, 0.3]
            })
            .to_string();
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = socket.write_all(response.as_bytes()).await;
        }
    });

    let client = OllamaClient::new(url);
    let emb = client.embed("test-model", "hello").await.unwrap();
    assert_eq!(emb, vec![0.1, 0.2, 0.3]);
}

#[tokio::test]
async fn test_fake_server_5xx_transient_retry_exhausted() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let url = format!("http://{}", addr);

    tokio::spawn(async move {
        while let Ok((mut socket, _)) = listener.accept().await {
            let mut buf = [0u8; 1024];
            let _ = socket.read(&mut buf).await;
            let response = "HTTP/1.1 503 Service Unavailable\r\nContent-Length: 0\r\n\r\n";
            let _ = socket.write_all(response.as_bytes()).await;
        }
    });

    let config = OllamaConfig {
        base_url: url,
        max_retries: 2,
        ..Default::default()
    };
    let client = OllamaClient::with_config(config);
    let res = client.embed("test-model", "hello").await;
    assert!(res.is_err());
}

#[tokio::test]
async fn test_fake_server_response_exceeds_max_bytes() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let url = format!("http://{}", addr);

    tokio::spawn(async move {
        if let Ok((mut socket, _)) = listener.accept().await {
            let mut buf = [0u8; 1024];
            let _ = socket.read(&mut buf).await;
            let body = "x".repeat(500);
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = socket.write_all(response.as_bytes()).await;
        }
    });

    let config = OllamaConfig {
        base_url: url,
        max_response_bytes: 100,
        ..Default::default()
    };
    let client = OllamaClient::with_config(config);
    let res = client.embed("test-model", "hello").await;
    match res {
        Err(OllamaError::InvalidInput(msg)) => {
            assert!(msg.contains("Response size limit exceeded"));
        }
        other => panic!("Expected InvalidInput, got {:?}", other),
    }
}

#[tokio::test]
async fn test_fake_server_404_no_retry() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let url = format!("http://{}", addr);

    tokio::spawn(async move {
        if let Ok((mut socket, _)) = listener.accept().await {
            let mut buf = [0u8; 1024];
            let _ = socket.read(&mut buf).await;
            let response = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n";
            let _ = socket.write_all(response.as_bytes()).await;
        }
    });

    let config = OllamaConfig {
        base_url: url,
        max_retries: 5,
        ..Default::default()
    };
    let client = OllamaClient::with_config(config);
    let res = client.embed("missing-model", "hello").await;
    match res {
        Err(OllamaError::ModelNotFound(_)) => {}
        other => panic!("Expected ModelNotFound, got {:?}", other),
    }
}
