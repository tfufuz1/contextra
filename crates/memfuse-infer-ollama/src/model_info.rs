// FILE-CONTEXT
// STAND: 2026-08-30T18:54:39Z (SESSION: ed7b7b38)
// ZWECK: Modell-Inspektion (/api/show) und Statisches Mapping bekannter Embedding-Dimensionen
// INVARIANTEN: Modellnamen-Validierung vor API-Aufruf; Tag-Stripping bei Dimension-Lookup
// NICHT-OFFENSICHTLICH: known_dimension liefert None für unbekannte Modelle, wodurch validation dynamic wird
// HOTSPOTS: show_model, known_dimension

//! Bekannte Ollama-Modell-Dimensionen für Embedding-Modelle.

use crate::client::OllamaClient;
use memfuse_core::{MemFuseError, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub modelfile: Option<String>,
    pub parameter_size: Option<String>,
    pub quantization_level: Option<String>,
}

impl OllamaClient {
    /// Validates that the model exists in Ollama via GET /api/tags before invoking embeddings/chat.
    pub async fn validate_model_available(&self, model: &str) -> Result<()> {
        crate::client::validate_model_name(model)?;
        if !self.is_model_available(model).await {
            return Err(MemFuseError::NotFound(format!(
                "Ollama model '{}' not found. Run: ollama pull {}",
                model, model
            )));
        }
        Ok(())
    }

    /// Retrieves model info via POST /api/show
    pub async fn show_model(&self, model: &str) -> Result<ModelInfo> {
        crate::client::validate_model_name(model)?;

        #[derive(Serialize)]
        struct ShowRequest<'a> {
            name: &'a str,
        }
        #[derive(Deserialize)]
        struct ShowResponse {
            modelfile: Option<String>,
            details: Option<ModelDetails>,
        }
        #[derive(Deserialize)]
        struct ModelDetails {
            parameter_size: Option<String>,
            quantization_level: Option<String>,
        }

        let url = format!("{}/api/show", self.base_url());
        let req = ShowRequest { name: model };

        let response = self
            .client
            .post(&url)
            .json(&req)
            .send()
            .await
            .map_err(|e| {
                crate::client::classify_reqwest_error(e, self.base_url(), "Ollama /api/show")
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "<body unreadable>".into());
            let lower = body.to_lowercase();
            if lower.contains("model") && lower.contains("not found")
                || status == reqwest::StatusCode::NOT_FOUND
            {
                return Err(MemFuseError::NotFound(format!(
                    "Ollama model '{model}' not found. Run: ollama pull {model}"
                )));
            }
            return Err(MemFuseError::Storage(format!(
                "Ollama show_model HTTP {status}: {body}"
            )));
        }

        let parsed: ShowResponse = response
            .json()
            .await
            .map_err(|e| MemFuseError::Internal(format!("Invalid Ollama show response: {e}")))?;

        let (param_size, quant_level) = match parsed.details {
            Some(d) => (d.parameter_size, d.quantization_level),
            None => (None, None),
        };

        Ok(ModelInfo {
            modelfile: parsed.modelfile,
            parameter_size: param_size,
            quantization_level: quant_level,
        })
    }
}

/// Gibt die Embedding-Dimension für bekannte Modelle zurück.
/// Gibt `None` zurück wenn das Modell unbekannt ist.
pub fn known_dimension(model: &str) -> Option<usize> {
    // Normalisiere: Kleinbuchstaben, Strip Tag (z.B. ":latest")
    let base = model.split(':').next().unwrap_or(model).to_lowercase();
    match base.as_str() {
        "nomic-embed-text" => Some(768),
        "mxbai-embed-large" => Some(1024),
        "all-minilm" => Some(384),
        "snowflake-arctic-embed" => Some(1024),
        "text-embedding-ada-002" => Some(1536), // OpenAI-kompatibel
        "text-embedding-3-small" => Some(1536),
        "text-embedding-3-large" => Some(3072),
        "bge-large-en-v1.5" => Some(1024),
        "bge-m3" => Some(1024),
        _ => None,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_validate_model_available() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap(); // unwrap
        let addr = listener.local_addr().unwrap(); // unwrap
        let server_url = format!("http://{}", addr);

        tokio::spawn(async move {
            while let Ok((mut socket, _)) = listener.accept().await {
                use tokio::io::{AsyncReadExt, AsyncWriteExt};
                let mut buf = [0u8; 1024];
                let _ = socket.read(&mut buf).await;
                let body = serde_json::json!({
                    "models": [
                        { "name": "nomic-embed-text:latest" }
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
        assert!(client
            .validate_model_available("nomic-embed-text")
            .await
            .is_ok());

        let err = client
            .validate_model_available("missing-model")
            .await
            .unwrap_err();
        match err {
            MemFuseError::NotFound(msg) => {
                assert!(msg.contains("Ollama model 'missing-model' not found"));
                assert!(msg.contains("Run: ollama pull missing-model"));
            }
            _ => panic!("Expected MemFuseError::NotFound, got {:?}", err),
        }
    }

    #[tokio::test]
    async fn test_model_info_fetch_offline_returns_io_error() {
        let client = OllamaClient::new("http://127.0.0.1:1");
        let result = client.show_model("llama3.2").await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            MemFuseError::Io(e) => {
                assert_eq!(e.kind(), std::io::ErrorKind::ConnectionRefused);
                assert!(e
                    .to_string()
                    .contains("Ensure Ollama is running (`ollama serve`)"));
            }
            _ => panic!("Expected MemFuseError::Io, got {:?}", err),
        }
    }

    #[test]
    fn test_known_dimension_nomic() {
        assert_eq!(known_dimension("nomic-embed-text"), Some(768));
        assert_eq!(known_dimension("nomic-embed-text:latest"), Some(768));
        assert_eq!(known_dimension("mxbai-embed-large"), Some(1024));
        assert_eq!(known_dimension("all-minilm"), Some(384));
        assert_eq!(known_dimension("snowflake-arctic-embed"), Some(1024));
        assert_eq!(known_dimension("text-embedding-ada-002"), Some(1536));
        assert_eq!(known_dimension("text-embedding-3-small"), Some(1536));
        assert_eq!(known_dimension("text-embedding-3-large"), Some(3072));
        assert_eq!(known_dimension("bge-large-en-v1.5"), Some(1024));
        assert_eq!(known_dimension("bge-m3"), Some(1024));
        assert_eq!(known_dimension("unknown-model-xyz"), None);
    }

    #[tokio::test]
    async fn test_show_model_success_and_error_paths() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server_url = format!("http://{}", addr);

        tokio::spawn(async move {
            if let Ok((mut socket, _)) = listener.accept().await {
                use tokio::io::{AsyncReadExt, AsyncWriteExt};
                let mut buf = [0u8; 1024];
                let _ = socket.read(&mut buf).await;
                let body = serde_json::json!({
                    "modelfile": "FROM nomic-embed-text",
                    "details": {
                        "parameter_size": "137M",
                        "quantization_level": "Q4_0"
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
        let info = client.show_model("nomic-embed-text").await.unwrap();
        assert_eq!(info.modelfile.as_deref(), Some("FROM nomic-embed-text"));
        assert_eq!(info.parameter_size.as_deref(), Some("137M"));
        assert_eq!(info.quantization_level.as_deref(), Some("Q4_0"));
    }

    #[tokio::test]
    async fn test_show_model_http_404_and_500_error_mappings() {
        // HTTP 404 test
        let listener_404 = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr_404 = listener_404.local_addr().unwrap();
        let server_url_404 = format!("http://{}", addr_404);

        tokio::spawn(async move {
            if let Ok((mut socket, _)) = listener_404.accept().await {
                use tokio::io::{AsyncReadExt, AsyncWriteExt};
                let mut buf = [0u8; 1024];
                let _ = socket.read(&mut buf).await;
                let body = r#"{"error":"model 'missing-model' not found"}"#;
                let response = format!(
                    "HTTP/1.1 404 Not Found\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                    body.len(),
                    body
                );
                socket.write_all(response.as_bytes()).await.ok();
            }
        });

        let client_404 = OllamaClient::new(server_url_404);
        let err_404 = client_404.show_model("missing-model").await.unwrap_err();
        assert!(matches!(err_404, MemFuseError::NotFound(_)));

        // HTTP 500 test
        let listener_500 = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr_500 = listener_500.local_addr().unwrap();
        let server_url_500 = format!("http://{}", addr_500);

        tokio::spawn(async move {
            if let Ok((mut socket, _)) = listener_500.accept().await {
                use tokio::io::{AsyncReadExt, AsyncWriteExt};
                let mut buf = [0u8; 1024];
                let _ = socket.read(&mut buf).await;
                let body = "Internal error";
                let response = format!(
                    "HTTP/1.1 500 Internal Server Error\r\nContent-Length: {}\r\n\r\n{}",
                    body.len(),
                    body
                );
                socket.write_all(response.as_bytes()).await.ok();
            }
        });

        let client_500 = OllamaClient::new(server_url_500);
        let err_500 = client_500.show_model("some-model").await.unwrap_err();
        assert!(matches!(err_500, MemFuseError::Storage(_)));
    }

    #[tokio::test]
    async fn test_show_model_invalid_json_returns_internal_error() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server_url = format!("http://{}", addr);

        tokio::spawn(async move {
            if let Ok((mut socket, _)) = listener.accept().await {
                use tokio::io::{AsyncReadExt, AsyncWriteExt};
                let mut buf = [0u8; 1024];
                let _ = socket.read(&mut buf).await;
                let body = "invalid-json";
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                    body.len(),
                    body
                );
                socket.write_all(response.as_bytes()).await.ok();
            }
        });

        let client = OllamaClient::new(server_url);
        let err = client.show_model("test-model").await.unwrap_err();
        assert!(matches!(err, MemFuseError::Internal(_)));
    }

    #[test]
    fn model_info_deserializes_correctly() {
        let json = r#"{"modelfile":"FROM nomic-embed-text","parameter_size":"137M","quantization_level":"Q4_0"}"#;
        let info: ModelInfo = serde_json::from_str(json).unwrap(); // unwrap
        assert_eq!(info.modelfile.as_deref(), Some("FROM nomic-embed-text"));
        assert_eq!(info.parameter_size.as_deref(), Some("137M"));
        assert_eq!(info.quantization_level.as_deref(), Some("Q4_0"));
    }
}
