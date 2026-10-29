use super::policy::EndpointPolicy;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Standard Ollama base URL
pub const DEFAULT_BASE_URL: &str = "http://localhost:11434";
/// Standard embedding model for SME usage (multilingual support for German)
pub const DEFAULT_EMBED_MODEL: &str = "nomic-embed-text";
/// Default maximum retry attempts for transient errors
pub const MAX_RETRIES: u32 = 3;
/// Maximum allowed batch size for batch operations to prevent request timeouts and memory exhaustion.
pub const MAX_BATCH_SIZE: usize = 512;
/// Maximum allowed text length in bytes (10 MB) to prevent OOM or DoS
pub const MAX_TEXT_BYTES: usize = 10_000_000;
/// Default maximum response size limit in bytes (10 MB)
pub const DEFAULT_MAX_RESPONSE_BYTES: usize = 10_000_000;

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
    /// Timeout for reading HTTP responses (default: 30 seconds)
    pub read_timeout: Duration,
    /// Maximum allowed HTTP response body size in bytes (default: 10 MB)
    pub max_response_bytes: usize,
    /// Maximum number of retries for transient errors (default: 3)
    pub max_retries: u32,
    /// Endpoint policy enforcing loopback and remote allowlists
    pub policy: EndpointPolicy,
    /// Consecutive failure count threshold to trip circuit breaker (default: 5)
    pub failure_threshold: u32,
    /// Time period circuit breaker remains OPEN before resetting (default: 30 seconds)
    pub circuit_reset_timeout: Duration,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            base_url: DEFAULT_BASE_URL.to_string(),
            model: DEFAULT_EMBED_MODEL.to_string(),
            request_timeout: Duration::from_secs(30),
            connect_timeout: Duration::from_secs(5),
            read_timeout: Duration::from_secs(30),
            max_response_bytes: DEFAULT_MAX_RESPONSE_BYTES,
            max_retries: MAX_RETRIES,
            policy: EndpointPolicy::default(),
            failure_threshold: 5,
            circuit_reset_timeout: Duration::from_secs(30),
        }
    }
}
