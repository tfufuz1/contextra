// campaign_stub_ollama.rs — Campaign stub tests for contextra-infer-ollama (J-19)
// Oracle source: Local HTTP mock server simulating Ollama API edge cases.

use contextra_infer_ollama::{OllamaClient, OllamaConfig};
use contextra_types::ContextraError;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::test]
async fn campaign_stub_success_200() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server_url = format!("http://{}", addr);

    tokio::spawn(async move {
        if let Ok((mut socket, _)) = listener.accept().await {
            let mut buf = [0u8; 1024];
            let _ = socket.read(&mut buf).await;
            let body = serde_json::json!({ "embedding": [0.1, 0.2, 0.3] }).to_string();
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                body.len(),
                body
            );
            socket.write_all(response.as_bytes()).await.ok();
        }
    });

    let client = OllamaClient::new(server_url);
    let res = client.embed("nomic-embed-text", "hello").await;
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), vec![0.1, 0.2, 0.3]);
}

#[tokio::test]
async fn campaign_stub_server_500_error() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server_url = format!("http://{}", addr);

    tokio::spawn(async move {
        while let Ok((mut socket, _)) = listener.accept().await {
            let mut buf = [0u8; 1024];
            let _ = socket.read(&mut buf).await;
            let response = "HTTP/1.1 500 Internal Server Error\r\nContent-Length: 5\r\n\r\nERROR";
            socket.write_all(response.as_bytes()).await.ok();
        }
    });

    let config = OllamaConfig {
        base_url: server_url,
        max_retries: 2,
        ..Default::default()
    };
    let client = OllamaClient::with_config(config);
    let res = client.embed("nomic-embed-text", "hello").await;
    assert!(res.is_err());
}

#[tokio::test]
async fn campaign_stub_timeout() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server_url = format!("http://{}", addr);

    tokio::spawn(async move {
        if let Ok((mut socket, _)) = listener.accept().await {
            let mut buf = [0u8; 1024];
            let _ = socket.read(&mut buf).await;
            tokio::time::sleep(Duration::from_millis(200)).await;
            let body = serde_json::json!({ "embedding": [0.1] }).to_string();
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                body.len(),
                body
            );
            socket.write_all(response.as_bytes()).await.ok();
        }
    });

    let config = OllamaConfig {
        base_url: server_url,
        request_timeout: Duration::from_millis(50),
        max_retries: 1,
        ..Default::default()
    };
    let client = OllamaClient::with_config(config);
    let res = client.embed("nomic-embed-text", "hello").await;
    assert!(res.is_err());
}

#[tokio::test]
async fn campaign_stub_malformed_json() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server_url = format!("http://{}", addr);

    tokio::spawn(async move {
        if let Ok((mut socket, _)) = listener.accept().await {
            let mut buf = [0u8; 1024];
            let _ = socket.read(&mut buf).await;
            let body = "{ broken_json: [1, 2, ";
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                body.len(),
                body
            );
            socket.write_all(response.as_bytes()).await.ok();
        }
    });

    let client = OllamaClient::new(server_url);
    let res = client.embed("nomic-embed-text", "hello").await;
    assert!(res.is_err());
    assert!(matches!(res.unwrap_err(), ContextraError::Internal(_)));
}

#[tokio::test]
async fn campaign_stub_empty_response() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server_url = format!("http://{}", addr);

    tokio::spawn(async move {
        if let Ok((mut socket, _)) = listener.accept().await {
            let mut buf = [0u8; 1024];
            let _ = socket.read(&mut buf).await;
            let body = "{}";
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                body.len(),
                body
            );
            socket.write_all(response.as_bytes()).await.ok();
        }
    });

    let client = OllamaClient::new(server_url);
    let res = client.embed("nomic-embed-text", "hello").await;
    assert!(res.is_err());
    assert!(matches!(res.unwrap_err(), ContextraError::Internal(_)));
}

#[tokio::test]
async fn campaign_stub_batch_dimension_mismatch() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server_url = format!("http://{}", addr);

    tokio::spawn(async move {
        if let Ok((mut socket, _)) = listener.accept().await {
            let mut buf = [0u8; 1024];
            let _ = socket.read(&mut buf).await;
            let body = serde_json::json!({ "embeddings": [[0.1, 0.2]] }).to_string();
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                body.len(),
                body
            );
            socket.write_all(response.as_bytes()).await.ok();
        }
    });

    let client = OllamaClient::new(server_url);
    let res = client
        .try_embed_batch("nomic-embed-text", &["item1", "item2"])
        .await;
    assert!(res.is_err());
    assert!(matches!(res.unwrap_err(), ContextraError::Internal(_)));
}

#[tokio::test]
async fn campaign_stub_huge_response() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server_url = format!("http://{}", addr);

    tokio::spawn(async move {
        if let Ok((mut socket, _)) = listener.accept().await {
            let mut buf = [0u8; 1024];
            let _ = socket.read(&mut buf).await;
            let huge_embedding: Vec<f32> = vec![0.1; 100_000];
            let body = serde_json::json!({ "embedding": huge_embedding }).to_string();
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                body.len(),
                body
            );
            socket.write_all(response.as_bytes()).await.ok();
        }
    });

    let client = OllamaClient::new(server_url);
    let res = client.embed("nomic-embed-text", "hello").await;
    assert!(res.is_ok());
    assert_eq!(res.unwrap().len(), 100_000);
}
