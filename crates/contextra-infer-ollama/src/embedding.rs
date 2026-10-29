// FILE-CONTEXT
// STAND: 2026-08-30T18:54:39Z (SESSION: ed7b7b38)
// ZWECK: `TextEmbeddingEngine`-Implementierung für Ollama Embedding API
// INVARIANTEN: Validiert Embedding-Dimensionen beim ersten Aufruf und sperrt diese fest gegen spätere Abweichungen
// NICHT-OFFENSICHTLICH: Mismatch bei Embedding-Dimensionen liefert `EmbeddingError::ComputationFailed` um stille Index-Korruption zu verhindern
// HOTSPOTS: embed, embed_batch

use crate::client::{OllamaClient, OllamaConfig, DEFAULT_BASE_URL, DEFAULT_EMBED_MODEL};
use crate::model_info::known_dimension;
use contextra_ports::{BoxFuture, EmbeddingError, EmbeddingProvider};
use contextra_types::ContextraError;
use parking_lot::Mutex;
use std::sync::Arc;

/// Implementation of `TextEmbeddingEngine` using Ollama's HTTP API.
#[derive(Clone, Debug)]
pub struct OllamaEmbedder {
    client: OllamaClient,
    model: String,
    concurrency: usize,
    expected_dimension: Option<usize>,
    observed_dimension: Arc<Mutex<Option<usize>>>,
}

impl OllamaEmbedder {
    pub fn new(base_url: impl Into<String>, model: impl Into<String>) -> Self {
        let base_url_str = base_url.into();
        let model_str = model.into();
        let config = OllamaConfig {
            base_url: base_url_str,
            model: model_str.clone(),
            ..Default::default()
        };
        let expected_dimension = known_dimension(&model_str);
        Self {
            client: OllamaClient::with_config(config),
            model: model_str,
            concurrency: 8,
            expected_dimension,
            observed_dimension: Arc::new(Mutex::new(None)),
        }
    }

    pub fn with_config(config: OllamaConfig) -> Self {
        let model = config.model.clone();
        let expected_dimension = known_dimension(&model);
        Self {
            client: OllamaClient::with_config(config),
            model,
            concurrency: 8,
            expected_dimension,
            observed_dimension: Arc::new(Mutex::new(None)),
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(DEFAULT_BASE_URL, DEFAULT_EMBED_MODEL)
    }

    pub fn config(&self) -> &OllamaConfig {
        self.client.config()
    }

    pub fn with_concurrency(mut self, concurrency: usize) -> Self {
        self.concurrency = concurrency.max(1);
        self
    }

    pub fn with_expected_dimension(mut self, dim: usize) -> Self {
        self.expected_dimension = Some(dim);
        self
    }

    pub fn model(&self) -> &str {
        &self.model
    }

    fn check_and_lock_dimension(&self, dim: usize) -> std::result::Result<(), EmbeddingError> {
        let mut lock = self.observed_dimension.lock();
        match *lock {
            None => {
                if let Some(expected_dim) = self.expected_dimension {
                    if dim != expected_dim {
                        return Err(EmbeddingError::ComputationFailed(format!(
                            "Ollama returned embedding of dimension {} but expected {}. Model '{}' may have changed. Rebuild the HNSW index.",
                            dim,
                            expected_dim,
                            self.model
                        )));
                    }
                }
                *lock = Some(dim);
                Ok(())
            }
            Some(observed_dim) => {
                if dim != observed_dim {
                    return Err(EmbeddingError::ComputationFailed(format!(
                        "Ollama embedding dimension mismatch: previously observed {}, but got {} on subsequent call for model '{}'. Silent index corruption prevented.",
                        observed_dim,
                        dim,
                        self.model
                    )));
                }
                Ok(())
            }
        }
    }
}

impl EmbeddingProvider for OllamaEmbedder {
    fn provider_name(&self) -> &str {
        "ollama"
    }

    fn embed<'a>(
        &'a self,
        text: &'a str,
    ) -> BoxFuture<'a, std::result::Result<Vec<f32>, EmbeddingError>> {
        Box::pin(async move {
            let vec = self
                .client
                .embed(&self.model, text)
                .await
                .map_err(|e| match e {
                    ContextraError::NotFound(msg) | ContextraError::InvalidInput(msg) => {
                        EmbeddingError::Unavailable(msg)
                    }
                    other => EmbeddingError::ComputationFailed(other.to_string()),
                })?;

            self.check_and_lock_dimension(vec.len())?;
            Ok(vec)
        })
    }

    fn embedding_dim(&self) -> usize {
        if let Some(exp) = self.expected_dimension {
            exp
        } else {
            (*self.observed_dimension.lock()).unwrap_or_default()
        }
    }

    fn embed_batch<'a>(
        &'a self,
        texts: &'a [&'a str],
    ) -> BoxFuture<'a, std::result::Result<Vec<Vec<f32>>, EmbeddingError>> {
        Box::pin(async move {
            if texts.is_empty() {
                return Ok(Vec::new());
            }

            let output =
                self.client
                    .embed_batch(&self.model, texts)
                    .await
                    .map_err(|e| match e {
                        ContextraError::NotFound(msg) | ContextraError::InvalidInput(msg) => {
                            EmbeddingError::Unavailable(msg)
                        }
                        other => EmbeddingError::ComputationFailed(other.to_string()),
                    })?;

            for vec in &output {
                self.check_and_lock_dimension(vec.len())?;
            }

            Ok(output)
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn test_ollama_embedder_constructors_and_getters() {
        let embedder = OllamaEmbedder::with_defaults();
        assert_eq!(embedder.provider_name(), "ollama");
        assert_eq!(embedder.model(), DEFAULT_EMBED_MODEL);
        assert_eq!(embedder.config().base_url, DEFAULT_BASE_URL);
        assert_eq!(embedder.embedding_dim(), 768);

        let config = OllamaConfig {
            base_url: "http://127.0.0.1:11434".into(),
            model: "mxbai-embed-large".into(),
            ..Default::default()
        };
        let embedder_config = OllamaEmbedder::with_config(config)
            .with_concurrency(4)
            .with_expected_dimension(1024);
        assert_eq!(embedder_config.model(), "mxbai-embed-large");
        assert_eq!(embedder_config.embedding_dim(), 1024);
    }

    #[tokio::test]
    async fn test_embed_batch_empty() {
        let client = OllamaClient::new("http://127.0.0.1:1");
        let embedder = OllamaEmbedder {
            client,
            model: "test".into(),
            concurrency: 4,
            expected_dimension: None,
            observed_dimension: Arc::new(Mutex::new(None)),
        };
        let result = embedder.embed_batch(&[]).await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_dimension_validation_mismatch_returns_index_error() {
        use contextra_ports::TextEmbeddingEngine;
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

        let result = TextEmbeddingEngine::embed(&embedder, "test text").await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            ContextraError::Internal(msg) => {
                assert!(msg.contains("Ollama returned embedding of dimension 3 but expected 768"));
                assert!(msg.contains("Model 'nomic-embed-text' may have changed"));
            }
            _ => panic!("Expected ContextraError::Internal, got {:?}", err),
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

        // Create embedder for unknown model without expected dimension
        let embedder = OllamaEmbedder::new(server_url, "custom-unknown-model");
        assert_eq!(embedder.embedding_dim(), 0);

        // First call locks dimension to 4
        let res1 = embedder.embed("first call").await;
        assert!(res1.is_ok());
        assert_eq!(res1.unwrap().len(), 4);
        assert_eq!(embedder.embedding_dim(), 4);

        // Second call with 2 dims MUST fail with dimension mismatch error
        let res2 = embedder.embed("second call").await;
        assert!(res2.is_err());
        let err = res2.unwrap_err();
        match err {
            EmbeddingError::ComputationFailed(msg) => {
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

        let single = embedder.embed("hello").await.unwrap();
        assert_eq!(single, vec![0.1, 0.2]);

        let batch = embedder.embed_batch(&["hello", "world"]).await.unwrap();
        assert_eq!(batch, vec![vec![0.1, 0.2], vec![0.3, 0.4]]);
    }

    #[tokio::test]
    async fn test_embedding_provider_error_mapping_and_batch_dimension_mismatch() {
        let dead_config = OllamaConfig {
            base_url: "http://127.0.0.1:1".into(),
            failure_threshold: 100, // High threshold to prevent tripping breaker during dead port test
            ..Default::default()
        };
        let dead_embedder = OllamaEmbedder::with_config(dead_config);
        let single_err = dead_embedder.embed("hello").await.unwrap_err();
        assert!(matches!(single_err, EmbeddingError::ComputationFailed(_)));

        let batch_err = dead_embedder.embed_batch(&["hello"]).await.unwrap_err();
        assert!(matches!(batch_err, EmbeddingError::ComputationFailed(_)));

        // Test batch dimension mismatch
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
        let batch_dim_err = embedder_mismatch.embed_batch(&["hello"]).await.unwrap_err();
        match batch_dim_err {
            EmbeddingError::ComputationFailed(msg) => {
                assert!(msg.contains("Ollama returned embedding of dimension 2 but expected 4"));
            }
            _ => panic!(
                "Expected EmbeddingError::ComputationFailed, got {:?}",
                batch_dim_err
            ),
        }
    }
}
