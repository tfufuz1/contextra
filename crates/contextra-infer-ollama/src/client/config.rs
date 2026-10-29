use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Standard Ollama base URL
pub const DEFAULT_BASE_URL: &str = "http://localhost:11434";
/// Standard embedding model for SME usage (multilingual support for German)
pub const DEFAULT_EMBED_MODEL: &str = "nomic-embed-text";
/// Default maximum retry attempts for transient errors
pub const MAX_RETRIES: u32 = 3;
/// Maximum allowed batch size for batch operations to prevent request timeouts and memory exhaustion.
///
/// Bounded to 512 to match `contextra_infer_onnx::MAX_EMBED_BATCH_SIZE` for consistency across embedding
/// backends and to accommodate Ollama's sequential per-text embedding processing latency without
/// exceeding standard HTTP client timeouts.
pub const MAX_BATCH_SIZE: usize = 512;
/// Maximum allowed text length in bytes (10 MB) to prevent OOM or DoS
pub const MAX_TEXT_BYTES: usize = 10_000_000;

/// Configuration options for the Ollama HTTP client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    /// Base URL for the Ollama instance (default: `http://localhost:11434`)
    pub base_url: String,
    /// Model name for embedding generation (default: `nomic-embed-text`)
    pub model: String,
    /// Timeout for individual HTTP requests (default: 30 seconds)
    pub request_timeout: Duration,
    /// Timeout for establishing TCP connection (default: 5 seconds)
    pub connect_timeout: Duration,
    /// Maximum number of retries for transient errors (default: 3)
    pub max_retries: u32,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            base_url: DEFAULT_BASE_URL.to_string(),
            model: DEFAULT_EMBED_MODEL.to_string(),
            request_timeout: Duration::from_secs(30),
            connect_timeout: Duration::from_secs(5),
            max_retries: MAX_RETRIES,
        }
    }
}
