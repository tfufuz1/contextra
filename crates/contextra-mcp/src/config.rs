use contextra_ports::{EmbeddingProvider, LlmTextGenerator};
use contextra_types::ContextraError;
use std::path::PathBuf;
use std::sync::{Arc, Once};

static LLM_PROVIDER_DEPRECATION_WARN_ONCE: Once = Once::new();

/// Embedding backend options.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum EmbeddingBackend {
    None,
    LocalOnnx,
    Candle,
    Ollama,
}

/// Resolved provider string helper.
pub struct ResolvedProviderSetting {
    pub value: String,
    pub used_unprefixed_fallback: bool,
}

pub fn resolve_provider_setting(
    prefixed: Option<&str>,
    unprefixed: Option<&str>,
    default_val: &str,
) -> ResolvedProviderSetting {
    if let Some(p) = prefixed {
        if !p.trim().is_empty() {
            return ResolvedProviderSetting {
                value: p.to_string(),
                used_unprefixed_fallback: false,
            };
        }
    }
    if let Some(u) = unprefixed {
        if !u.trim().is_empty() {
            return ResolvedProviderSetting {
                value: u.to_string(),
                used_unprefixed_fallback: true,
            };
        }
    }
    ResolvedProviderSetting {
        value: default_val.to_string(),
        used_unprefixed_fallback: false,
    }
}

/// Embedding configuration settings.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct EmbeddingConfig {
    /// Provider type.
    pub provider: String,
}

impl EmbeddingConfig {
    /// Loads configuration from environment variables with fallbacks.
    pub fn from_env() -> Self {
        let prefixed = std::env::var("CONTEXTRA_EMBEDDING_PROVIDER").ok();
        let unprefixed = std::env::var("EMBEDDING_PROVIDER").ok();
        let resolved = resolve_provider_setting(prefixed.as_deref(), unprefixed.as_deref(), "");
        Self {
            provider: resolved.value,
        }
    }

    /// Instantiates the configured `EmbeddingProvider`.
    pub fn build_provider(&self) -> Result<Arc<dyn EmbeddingProvider>, ContextraError> {
        let trimmed_provider = self.provider.trim();
        if trimmed_provider.is_empty() {
            let msg = "Kein Embedding-Provider konfiguriert. Bitte CONTEXTRA_EMBEDDING_PROVIDER (oder EMBEDDING_PROVIDER) auf 'mock' setzen.".to_string();
            return Err(ContextraError::InvalidInput(msg));
        }

        if trimmed_provider.eq_ignore_ascii_case("mock") {
            #[cfg(any(test, debug_assertions, feature = "test-utils"))]
            {
                tracing::warn!("Embedding-Provider 'mock' ist explizit gesetzt. Es findet keine echte semantische Suche statt!");
                let embedder = contextra_ports::MockEmbedder::new(768);
                return Ok(Arc::new(embedder));
            }
            #[cfg(not(any(test, debug_assertions, feature = "test-utils")))]
            {
                return Err(ContextraError::InvalidInput(
                    "Embedding-Provider 'mock' ist in Release-Builds deaktiviert".to_string(),
                ));
            }
        }

        Err(ContextraError::InvalidInput(format!(
            "Unknown embedding provider '{trimmed_provider}'. Expected 'mock'."
        )))
    }
}

/// Dynamically constructs an `EmbeddingProvider` implementation based on an `EmbeddingBackend`.
pub fn create_embedding_provider(
    backend: &EmbeddingBackend,
) -> Result<Arc<dyn EmbeddingProvider>, ContextraError> {
    match backend {
        #[cfg(any(test, debug_assertions, feature = "test-utils"))]
        EmbeddingBackend::None => {
            tracing::warn!("Embedding-Provider 'mock' / 'none' ist explizit gesetzt. Es findet keine echte semantische Suche statt!");
            let embedder = contextra_ports::MockEmbedder::new(768);
            Ok(Arc::new(embedder))
        }
        #[cfg(not(any(test, debug_assertions, feature = "test-utils")))]
        EmbeddingBackend::None => Err(ContextraError::InvalidInput(
            "Kein Embedding-Backend konfiguriert".to_string(),
        )),
        _ => Err(ContextraError::InvalidInput(
            "Kein externes Embedding-Backend verfuegbar".to_string(),
        )),
    }
}

/// LLM provider configuration settings.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LlmConfig {
    /// Provider type ("mock").
    pub provider: String,
    /// Model identifier for LLM.
    pub llm_model: String,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: "mock".to_string(),
            llm_model: "mock".to_string(),
        }
    }
}

impl LlmConfig {
    /// Loads configuration from environment variables with fallbacks.
    pub fn from_env() -> Self {
        let prefixed = std::env::var("CONTEXTRA_LLM_PROVIDER").ok();
        let unprefixed = std::env::var("LLM_PROVIDER").ok();
        let resolved = resolve_provider_setting(prefixed.as_deref(), unprefixed.as_deref(), "mock");

        if resolved.used_unprefixed_fallback {
            LLM_PROVIDER_DEPRECATION_WARN_ONCE.call_once(|| {
                tracing::warn!(
                    "Ungeprefixte Variable LLM_PROVIDER ist deprecated, bitte CONTEXTRA_LLM_PROVIDER verwenden."
                );
            });
        }

        let provider = resolved.value;
        let llm_model =
            std::env::var("CONTEXTRA_LLM_MODEL").unwrap_or_else(|_| "mock".to_string());

        Self {
            provider,
            llm_model,
        }
    }

    /// Instantiates the configured `LlmTextGenerator` as an `Arc<dyn LlmTextGenerator>`.
    pub fn build_generator(&self) -> Result<Arc<dyn LlmTextGenerator>, ContextraError> {
        create_llm_text_generator(&self.provider)
    }
}

/// Dynamically constructs an `LlmTextGenerator` implementation based on provider identifier.
pub fn create_llm_text_generator(
    provider_type: &str,
) -> Result<Arc<dyn LlmTextGenerator>, ContextraError> {
    match provider_type.to_lowercase().trim() {
        #[cfg(any(test, debug_assertions, feature = "test-utils"))]
        "mock" => {
            let generator = MockLlmGenerator;
            Ok(Arc::new(generator))
        }
        #[cfg(not(any(test, debug_assertions, feature = "test-utils")))]
        "mock" => Err(ContextraError::InvalidInput(
            "LLM-Provider 'mock' ist in Release-Builds deaktiviert".to_string(),
        )),
        other => Err(ContextraError::InvalidInput(format!(
            "Unknown LLM provider '{other}'. Expected 'mock'."
        ))),
    }
}

/// Router configuration settings.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct RouterConfig {
    /// List of SLM profiles. If non-empty, routing is enabled.
    pub profiles: Vec<contextra::router::SlmProfile>,
    /// Optional path to routing profiles JSON file.
    pub profiles_path: Option<PathBuf>,
    /// Optional path to persistent calibration state.
    pub calibration_store_path: Option<PathBuf>,
}

impl RouterConfig {
    /// Loads configuration from environment variables with fallbacks.
    pub fn from_env() -> Self {
        let profiles_path = std::env::var("CONTEXTRA_ROUTER_PROFILES_PATH")
            .ok()
            .map(PathBuf::from);

        let calibration_store_path = std::env::var("CONTEXTRA_ROUTER_CALIBRATION_PATH")
            .ok()
            .map(PathBuf::from);

        let mut profiles = Vec::new();
        if let Some(ref path) = profiles_path {
            if let Ok(bytes) = std::fs::read(path) {
                if let Ok(loaded) =
                    serde_json::from_slice::<Vec<contextra::router::SlmProfile>>(&bytes)
                {
                    profiles = loaded;
                }
            }
        } else if let Ok(json_str) = std::env::var("CONTEXTRA_ROUTER_PROFILES_JSON") {
            if let Ok(loaded) =
                serde_json::from_str::<Vec<contextra::router::SlmProfile>>(&json_str)
            {
                profiles = loaded;
            }
        }

        Self {
            profiles,
            profiles_path,
            calibration_store_path,
        }
    }
}

/// IPC and transport security configuration.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IpcConfig {
    /// Custom path for the IPC socket or endpoint file.
    pub endpoint_path: Option<PathBuf>,
    /// Maximum line length for framing (default 8 MiB = 8 * 1024 * 1024).
    pub max_line_bytes: usize,
    /// Expected OS user ID for peer credential validation (if None, defaults to current process UID).
    pub expected_uid: Option<u32>,
}

/// Resolves or generates the master encryption passphrase for MCP DB opening.
pub fn resolve_mcp_encryption_config(
    db_path: &std::path::Path,
    args: &[String],
) -> (Option<String>, bool) {
    if args.iter().any(|a| a == "--disable-encryption-at-rest")
        || std::env::var("CONTEXTRA_DISABLE_ENCRYPTION").is_ok()
    {
        return (None, true);
    }

    if let Ok(p) = std::env::var("CONTEXTRA_ENCRYPTION_PASSPHRASE") {
        if !p.trim().is_empty() {
            return (Some(p), false);
        }
    }

    if let Ok(p) = std::env::var("CONTEXTRA_PASSPHRASE") {
        if !p.trim().is_empty() {
            return (Some(p), false);
        }
    }

    let key_path = db_path.join(".contextra_master.key");
    if key_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&key_path) {
            let trimmed = content.trim().to_string();
            if !trimmed.is_empty() {
                return (Some(trimmed), false);
            }
        }
    }

    let mut key_bytes = [0u8; 32];
    use rand::RngCore;
    rand::rngs::OsRng.fill_bytes(&mut key_bytes);
    let generated = key_bytes.iter().map(|b| format!("{b:02x}")).collect::<String>();
    let _ = std::fs::create_dir_all(db_path);
    let _ = std::fs::write(&key_path, &generated);

    (Some(generated), false)
}

pub const DEFAULT_MAX_LINE_BYTES: usize = 8 * 1024 * 1024; // 8 MiB

impl Default for IpcConfig {
    fn default() -> Self {
        Self {
            endpoint_path: None,
            max_line_bytes: DEFAULT_MAX_LINE_BYTES,
            expected_uid: None,
        }
    }
}

impl IpcConfig {
    /// Loads IPC configuration from environment variables.
    pub fn from_env() -> Self {
        let endpoint_path = std::env::var("CONTEXTRA_IPC_ENDPOINT")
            .ok()
            .map(PathBuf::from);

        let max_line_bytes = std::env::var("CONTEXTRA_IPC_MAX_LINE_BYTES")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(DEFAULT_MAX_LINE_BYTES);

        let expected_uid = std::env::var("CONTEXTRA_IPC_EXPECTED_UID")
            .ok()
            .and_then(|s| s.parse::<u32>().ok());

        Self {
            endpoint_path,
            max_line_bytes,
            expected_uid,
        }
    }

    /// Validates configuration parameters, failing closed on invalid values.
    pub fn validate(&self) -> Result<(), ContextraError> {
        if self.max_line_bytes == 0 {
            return Err(ContextraError::InvalidInput(
                "max_line_bytes must be strictly greater than 0".to_string(),
            ));
        }
        Ok(())
    }
}

/// Fallback Mock LLM Text Generator.
#[derive(Debug, Clone, Default)]
pub struct MockLlmGenerator;

impl LlmTextGenerator for MockLlmGenerator {
    fn generate<'a>(
        &'a self,
        prompt: &'a str,
    ) -> contextra_ports::BoxFuture<'a, Result<String, ContextraError>> {
        let response = format!("[Mock LLM response for: {prompt}]");
        Box::pin(async move { Ok(response) })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_provider_setting_prefixed_wins() {
        let res = resolve_provider_setting(Some("mock"), Some("fallback"), "default");
        assert_eq!(res.value, "mock");
        assert!(!res.used_unprefixed_fallback);
    }

    #[test]
    fn test_resolve_provider_setting_unprefixed_fallback() {
        let res = resolve_provider_setting(None, Some("fallback"), "default");
        assert_eq!(res.value, "fallback");
        assert!(res.used_unprefixed_fallback);
    }

    #[test]
    fn test_resolve_provider_setting_default() {
        let res = resolve_provider_setting(None, None, "");
        assert_eq!(res.value, "");
        assert!(!res.used_unprefixed_fallback);
    }

    #[test]
    fn test_embedding_config_defaults() {
        let config = EmbeddingConfig::default();
        assert_eq!(config.provider, "");
    }

    #[test]
    fn test_create_embedding_provider_unconfigured_error() {
        let config = EmbeddingConfig {
            provider: "".to_string(),
        };
        let res = config.build_provider();
        match res {
            Err(ContextraError::InvalidInput(msg)) => {
                assert!(msg.contains("CONTEXTRA_EMBEDDING_PROVIDER"));
                assert!(msg.contains("EMBEDDING_PROVIDER"));
                assert!(msg.contains("mock"));
            }
            Err(other) => panic!("Expected ContextraError::InvalidInput, got: {other}"),
            Ok(_) => panic!("Expected create_embedding_provider to fail when unconfigured"),
        }
    }

    #[test]
    fn test_llm_config_defaults() {
        let config = LlmConfig::default();
        assert_eq!(config.provider, "mock");
        assert_eq!(config.llm_model, "mock");
    }

    #[test]
    fn test_llm_config_build_generator() {
        let config = LlmConfig::default();
        let generator = config.build_generator().unwrap();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let response = rt.block_on(generator.generate("test prompt")).unwrap();
        assert!(response.contains("[Mock LLM response for: test prompt]"));
    }

    #[test]
    fn test_create_embedding_provider_mock() {
        let provider = create_embedding_provider(&EmbeddingBackend::None).unwrap();
        assert_eq!(provider.provider_name(), "mock");
        assert_eq!(provider.embedding_dim(), 768);
    }

    #[test]
    fn test_create_embedding_provider_unknown_error() {
        let config = EmbeddingConfig {
            provider: "invalid_provider".to_string(),
        };
        let res = config.build_provider();
        assert!(matches!(res, Err(ContextraError::InvalidInput(_))));
    }

    #[test]
    fn test_create_llm_generator_mock() {
        let generator = create_llm_text_generator("mock").unwrap();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let response = rt.block_on(generator.generate("hello")).unwrap();
        assert!(response.contains("[Mock LLM response for: hello]"));
    }

    #[test]
    fn test_create_llm_generator_unknown_error() {
        let res = create_llm_text_generator("invalid_provider");
        assert!(matches!(res, Err(ContextraError::InvalidInput(_))));
    }
}
