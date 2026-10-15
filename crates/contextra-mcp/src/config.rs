// FILE-CONTEXT
// STAND:       2026-10-15
// ZWECK:       Dynamic Provider Construction for Embedding & LLM backends (Mock)
// INVARIANTEN: Direct provider instantiation without leaking implementation details; fallback capability checking
// HOTSPOTS:    create_embedding_provider(), create_llm_text_generator()
// SIEHE AUCH:  ADR-010

use contextra::EmbeddingBackend;
use contextra_ports::{EmbeddingProvider, LlmTextGenerator};
use contextra_types::ContextraError;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Once};

static EMBEDDING_PROVIDER_DEPRECATION_WARN_ONCE: Once = Once::new();

/// Resolves the encryption-at-rest passphrase for MCP database initialization.
///
/// # Default Behavior
/// Encryption at rest is **active by default**.
/// 1. Checks `CONTEXTRA_ENCRYPTION_PASSPHRASE` or `CONTEXTRA_PASSPHRASE` environment variables.
/// 2. If CLI args contain `--disable-encryption-at-rest` or `CONTEXTRA_DISABLE_ENCRYPTION=1`,
///    encryption at rest is explicitly disabled and a loud warning log is emitted.
/// 3. Otherwise, generates or loads a persistent 256-bit passphrase keyfile (`.contextra_master.key`)
///    located inside `db_path`.
pub fn resolve_mcp_encryption_config(
    db_path: &Path,
    cli_args: &[String],
) -> (Option<String>, bool) {
    let disabled_by_flag = cli_args.iter().any(|a| a == "--disable-encryption-at-rest");
    let disabled_by_env = std::env::var("CONTEXTRA_DISABLE_ENCRYPTION")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    if disabled_by_flag || disabled_by_env {
        tracing::warn!(
            "SECURITY WARNING: Encryption at rest has been explicitly disabled via opt-out flag (--disable-encryption-at-rest / CONTEXTRA_DISABLE_ENCRYPTION). Stored database segments and WAL files are UNENCRYPTED!"
        );
        return (None, true);
    }

    let env_passphrase = std::env::var("CONTEXTRA_ENCRYPTION_PASSPHRASE")
        .ok()
        .or_else(|| std::env::var("CONTEXTRA_PASSPHRASE").ok())
        .filter(|p| !p.trim().is_empty());

    if let Some(p) = env_passphrase {
        return (Some(p), false);
    }

    // Auto-generate or load persistent master key file in db_path
    let key_file = db_path.join(".contextra_master.key");
    if key_file.exists() {
        if let Ok(key_hex) = std::fs::read_to_string(&key_file) {
            let clean = key_hex.trim().to_string();
            if !clean.is_empty() {
                return (Some(clean), false);
            }
        }
    }

    // Generate fresh key
    if !db_path.exists() {
        let _ = std::fs::create_dir_all(db_path);
    }

    use rand::RngCore;
    let mut key_bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut key_bytes);
    let key_hex = hex_encode(key_bytes);

    if let Err(e) = std::fs::write(&key_file, &key_hex) {
        tracing::warn!(
            error = %e,
            "Failed to save auto-generated master key file at {}",
            key_file.display()
        );
    }

    (Some(key_hex), false)
}

fn hex_encode(bytes: [u8; 32]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

static LLM_PROVIDER_DEPRECATION_WARN_ONCE: Once = Once::new();

/// Ergebnis der Provider-Einstellung-Auflösung.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ResolvedProviderSetting {
    pub value: String,
    pub used_unprefixed_fallback: bool,
}

/// Löste eine Provider-Einstellung ohne Prozess-Env-Zugriff auf.
pub(crate) fn resolve_provider_setting(
    prefixed: Option<&str>,
    unprefixed: Option<&str>,
    default_val: &str,
) -> ResolvedProviderSetting {
    let clean_prefixed = prefixed.map(|s| s.trim()).filter(|s| !s.is_empty());
    let clean_unprefixed = unprefixed.map(|s| s.trim()).filter(|s| !s.is_empty());

    match (clean_prefixed, clean_unprefixed) {
        (Some(p), _) => ResolvedProviderSetting {
            value: p.to_string(),
            used_unprefixed_fallback: false,
        },
        (None, Some(u)) => ResolvedProviderSetting {
            value: u.to_string(),
            used_unprefixed_fallback: true,
        },
        (None, None) => ResolvedProviderSetting {
            value: default_val.to_string(),
            used_unprefixed_fallback: false,
        },
    }
}

/// Embedding provider configuration settings.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct EmbeddingConfig {
    /// Provider type ("mock").
    pub provider: String,
}

impl EmbeddingConfig {
    /// Loads configuration from environment variables with fallbacks.
    pub fn from_env() -> Self {
        let prefixed = std::env::var("CONTEXTRA_EMBEDDING_PROVIDER").ok();
        let unprefixed = std::env::var("EMBEDDING_PROVIDER").ok();
        let resolved = resolve_provider_setting(prefixed.as_deref(), unprefixed.as_deref(), "");

        if resolved.used_unprefixed_fallback {
            EMBEDDING_PROVIDER_DEPRECATION_WARN_ONCE.call_once(|| {
                tracing::warn!(
                    "Ungeprefixte Variable EMBEDDING_PROVIDER ist deprecated, bitte CONTEXTRA_EMBEDDING_PROVIDER verwenden."
                );
            });
        }

        let provider = resolved.value;

        Self { provider }
    }

    /// Instantiates the configured `EmbeddingProvider` as an `Arc<dyn EmbeddingProvider>`.
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
