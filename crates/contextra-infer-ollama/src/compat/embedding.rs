//! Port-Trait Implementierung `EmbeddingProvider` für `OllamaEmbedder`.

use crate::embedding::OllamaEmbedder;
use crate::error::OllamaError;
use contextra_ports::{BoxFuture as PortsBoxFuture, EmbeddingError as PortsEmbeddingError, EmbeddingProvider};

impl EmbeddingProvider for OllamaEmbedder {
    fn provider_name(&self) -> &str {
        "ollama"
    }

    fn embed<'a>(
        &'a self,
        text: &'a str,
    ) -> PortsBoxFuture<'a, std::result::Result<Vec<f32>, PortsEmbeddingError>> {
        Box::pin(async move {
            let vec = self
                .client()
                .embed(&self.model(), text)
                .await
                .map_err(|e| match e {
                    OllamaError::ModelNotFound(msg) | OllamaError::InvalidInput(msg) => {
                        PortsEmbeddingError::Unavailable(msg)
                    }
                    other => PortsEmbeddingError::ComputationFailed(other.to_string()),
                })?;

            self.check_and_lock_dimension(vec.len())
                .map_err(|e| PortsEmbeddingError::ComputationFailed(e.to_string()))?;
            Ok(vec)
        })
    }

    fn embedding_dim(&self) -> usize {
        self.embedding_dimension()
    }

    fn embed_batch<'a>(
        &'a self,
        texts: &'a [&'a str],
    ) -> PortsBoxFuture<'a, std::result::Result<Vec<Vec<f32>>, PortsEmbeddingError>> {
        Box::pin(async move {
            if texts.is_empty() {
                return Ok(Vec::new());
            }

            let output = self
                .client()
                .embed_batch(&self.model(), texts)
                .await
                .map_err(|e| match e {
                    OllamaError::ModelNotFound(msg) | OllamaError::InvalidInput(msg) => {
                        PortsEmbeddingError::Unavailable(msg)
                    }
                    other => PortsEmbeddingError::ComputationFailed(other.to_string()),
                })?;

            for vec in &output {
                self.check_and_lock_dimension(vec.len())
                    .map_err(|e| PortsEmbeddingError::ComputationFailed(e.to_string()))?;
            }

            Ok(output)
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use crate::client::{OllamaConfig, DEFAULT_BASE_URL, DEFAULT_EMBED_MODEL};
    use std::sync::Arc;

    #[test]
    fn test_ollama_embedder_constructors_and_getters() {
        let embedder = OllamaEmbedder::with_defaults();
        assert_eq!(EmbeddingProvider::provider_name(&embedder), "ollama");
        assert_eq!(embedder.model(), DEFAULT_EMBED_MODEL);
        assert_eq!(embedder.config().base_url, DEFAULT_BASE_URL);
        assert_eq!(EmbeddingProvider::embedding_dim(&embedder), 768);

        let config = OllamaConfig {
            base_url: "http://127.0.0.1:11434".into(),
            model: "mxbai-embed-large".into(),
            ..Default::default()
        };
        let embedder_config = OllamaEmbedder::with_config(config)
            .with_concurrency(4)
            .with_expected_dimension(1024);
        assert_eq!(embedder_config.model(), "mxbai-embed-large");
        assert_eq!(EmbeddingProvider::embedding_dim(&embedder_config), 1024);
    }

    #[tokio::test]
    async fn test_embed_batch_empty() {
        let embedder = OllamaEmbedder::new("http://127.0.0.1:1", "test");
        let result = EmbeddingProvider::embed_batch(&embedder, &[])
            .await
            .unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_dimension_validation_mismatch_returns_index_error() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server_url = format!("http://{}", addr);

        tokio::spawn(async move {
            if let Ok((mut socket, _)) = listener.accept().await {
                use tokio::io::{AsyncReadExt, AsyncWriteExt};
                let mut buf = [0u8; 1024];
                let _ = socket.read(&mut buf).await;
                let body = serde_json::json!({
                    "embedding": [0.1, 0.2, 0.3] // 3 dimensions
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

        let embedder =
            OllamaEmbedder::new(server_url, "nomic-embed-text").with_expected_dimension(768);

        let result = EmbeddingProvider::embed(&embedder, "test text").await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            PortsEmbeddingError::ComputationFailed(msg) => {
                assert!(msg.contains("Ollama returned embedding of dimension 3 but expected 768"));
                assert!(msg.contains("Model 'nomic-embed-text' may have changed"));
            }
            _ => panic!("Expected ComputationFailed, got {:?}", err),
        }
    }

    #[tokio::test]
    async fn test_dynamic_dimension_locking_and_subsequent_mismatch_guard() {
        use std::sync::atomic::{AtomicU32, Ordering};

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server_url = format!("http://{}", addr);
        let call_count = Arc::new(AtomicU32::new(0));
        let count_clone = call_count.clone();

        tokio::spawn(async move {
            while let Ok((mut socket, _)) = listener.accept().await {
                use tokio::io::{AsyncReadExt, AsyncWriteExt};
                let mut buf = [0u8; 1024];
                let _ = socket.read(&mut buf).await;
                let cnt = count_clone.fetch_add(1, Ordering::SeqCst);

                let embedding = if cnt == 0 {
                    vec![0.1, 0.2, 0.3, 0.4] // 4 dims on first call
                } else {
                    vec![0.1, 0.2] // 2 dims on second call
                };

                let body = serde_json::json!({ "embedding": embedding }).to_string();
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                    body.len(),
                    body
                );
                socket.write_all(response.as_bytes()).await.ok();
            }
        });

        let embedder = OllamaEmbedder::new(server_url, "custom-unknown-model");
        assert_eq!(EmbeddingProvider::embedding_dim(&embedder), 0);

        let res1 = EmbeddingProvider::embed(&embedder, "first call").await;
        assert!(res1.is_ok());
        assert_eq!(res1.unwrap().len(), 4);
        assert_eq!(EmbeddingProvider::embedding_dim(&embedder), 4);

        let res2 = EmbeddingProvider::embed(&embedder, "second call").await;
        assert!(res2.is_err());
        let err = res2.unwrap_err();
        match err {
            PortsEmbeddingError::ComputationFailed(msg) => {
                assert!(msg.contains("previously observed 4, but got 2"));
                assert!(msg.contains("Silent index corruption prevented"));
            }
            _ => panic!("Expected ComputationFailed, got {:?}", err),
        }
    }

    #[tokio::test]
    async fn test_embedding_provider_embed_and_batch_mock_success() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server_url = format!("http://{}", addr);

        tokio::spawn(async move {
            while let Ok((mut socket, _)) = listener.accept().await {
                use tokio::io::{AsyncReadExt, AsyncWriteExt};
                let mut buf = [0u8; 2048];
                let n = socket.read(&mut buf).await.unwrap_or(0);
                let req_str = String::from_utf8_lossy(&buf[..n]);

                if req_str.starts_with("POST /api/embeddings ") {
                    let body = serde_json::json!({ "embedding": [0.1, 0.2] }).to_string();
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    socket.write_all(response.as_bytes()).await.ok();
                } else if req_str.starts_with("POST /api/embed ") {
                    let body =
                        serde_json::json!({ "embeddings": [[0.1, 0.2], [0.3, 0.4]] }).to_string();
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    socket.write_all(response.as_bytes()).await.ok();
                }
            }
        });

        let embedder = OllamaEmbedder::new(server_url, "test-model").with_expected_dimension(2);

        let single = EmbeddingProvider::embed(&embedder, "hello").await.unwrap();
        assert_eq!(single, vec![0.1, 0.2]);

        let batch = EmbeddingProvider::embed_batch(&embedder, &["hello", "world"])
            .await
            .unwrap();
        assert_eq!(batch, vec![vec![0.1, 0.2], vec![0.3, 0.4]]);
    }

    #[tokio::test]
    async fn test_embedding_provider_error_mapping_and_batch_dimension_mismatch() {
        let dead_config = OllamaConfig {
            base_url: "http://127.0.0.1:1".into(),
            failure_threshold: 100,
            ..Default::default()
        };
        let dead_embedder = OllamaEmbedder::with_config(dead_config);
        let single_err = EmbeddingProvider::embed(&dead_embedder, "hello")
            .await
            .unwrap_err();
        assert!(matches!(single_err, PortsEmbeddingError::ComputationFailed(_)));

        let batch_err = EmbeddingProvider::embed_batch(&dead_embedder, &["hello"])
            .await
            .unwrap_err();
        assert!(matches!(batch_err, PortsEmbeddingError::ComputationFailed(_)));

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server_url = format!("http://{}", addr);

        tokio::spawn(async move {
            if let Ok((mut socket, _)) = listener.accept().await {
                use tokio::io::{AsyncReadExt, AsyncWriteExt};
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

        let embedder_mismatch =
            OllamaEmbedder::new(server_url, "test-model").with_expected_dimension(4);
        let batch_dim_err = EmbeddingProvider::embed_batch(&embedder_mismatch, &["hello"])
            .await
            .unwrap_err();
        match batch_dim_err {
            PortsEmbeddingError::ComputationFailed(msg) => {
                assert!(msg.contains("Ollama returned embedding of dimension 2 but expected 4"));
            }
            _ => panic!("Expected ComputationFailed, got {:?}", batch_dim_err),
        }
    }
}
