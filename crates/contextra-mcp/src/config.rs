// FILE-CONTEXT
// STAND:       2026-10-31T00:00:00Z
// ZWECK:       Dynamic Provider Construction for Embedding & LLM backends (Ollama, ONNX, Candle, Mock)
// INVARIANTEN: No default mock provider in production; fail-closed explicit provider configuration; startup embedding probe
// HOTSPOTS:    create_embedding_provider(), create_llm_text_generator(), verify_embedding_provider()
// SIEHE AUCH:  ADR-010, contextra-ports/src/traits/mod.rs

use contextra_ports::{EmbeddingProvider, LlmTextGenerator};
use contextra_types::ContextraError;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Once};

static EMBEDDING_PROVIDER_DEPRECATION_WARN_ONCE: Once = Once::new();
static LLM_PROVIDER_DEPRECATION_WARN_ONCE: Once = Once::new();

/// Returns the list of currently compiled provider identifiers.
pub fn available_compiled_providers() -> Vec<&'static str> {
    let mut list = Vec::new();
    if cfg!(feature = "candle") {
        list.push("candle");
    }
    if cfg!(feature = "ollama") {
        list.push("ollama");
    }
    if cfg!(feature = "onnx") {
        list.push("onnx");
    }
    if cfg!(any(test, feature = "test-utils")) {
        list.push("mock");
    }
    list
}

fn no_provider_error() -> ContextraError {
    let avail = available_compiled_providers();
    let avail_str = if avail.is_empty() {
        "keine einkompiliert (features: candle, ollama, onnx) / none compiled in".to_string()
    } else {
        avail.join(", ")
    };
    ContextraError::InvalidInput(format!(
        "Kein Embedding-Provider konfiguriert / No embedding provider configured. \
         Bitte CONTEXTRA_EMBEDDING_PROVIDER setzen (z.B. CONTEXTRA_EMBEDDING_PROVIDER=ollama). \
         Einkompilierte Provider / Available compiled providers: [{avail_str}]."
    ))
}

fn uncompiled_provider_error(provider: &str) -> ContextraError {
    let avail = available_compiled_providers();
    let avail_str = if avail.is_empty() {
        "keine / none".to_string()
    } else {
        avail.join(", ")
    };
    ContextraError::CapabilityUnsupported {
        capability: format!("{provider} embedding backend"),
        reason: format!(
            "Provider '{provider}' ist nicht einkompiliert / Provider '{provider}' is not compiled in (Feature '{provider}' erforderlich). \
             Verfügbare einkompilierte Provider / Available compiled providers: [{avail_str}]."
        ),
    }
}

fn unknown_provider_error(provider: &str) -> ContextraError {
    let avail = available_compiled_providers();
    let avail_str = if avail.is_empty() {
        "keine / none".to_string()
    } else {
        avail.join(", ")
    };
    ContextraError::InvalidInput(format!(
        "Unbekannter Embedding-Provider '{provider}' / Unknown embedding provider '{provider}'. \
         Verfügbare einkompilierte Provider / Available compiled providers: [{avail_str}]."
    ))
}

/// Startup embedding self-test (probe) verifying provider functionality.
pub fn verify_embedding_provider(
    provider: &dyn EmbeddingProvider,
    expected_dim: Option<usize>,
) -> Result<(), ContextraError> {
    let probe_text = "contextra startup probe";
    let probe_fut = provider.embed(probe_text);

    let vec = match tokio::runtime::Handle::try_current() {
        Ok(handle) => tokio::task::block_in_place(|| handle.block_on(probe_fut)),
        Err(_) => {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|e| {
                    ContextraError::Internal(format!(
                        "Konnte Tokio-Laufzeit für Probe nicht erstellen: {e}"
                    ))
                })?;
            rt.block_on(probe_fut)
        }
    }
    .map_err(|e| {
        ContextraError::Internal(format!(
            "Embedding-Selbsttest Probe fehlgeschlagen / Embedding probe failed: {e}"
        ))
    })?;

    // Probe check 1: Dimension > 0
    if vec.is_empty() {
        return Err(ContextraError::Internal(
            "Embedding-Selbsttest fehlgeschlagen: Vektordimension ist 0 / Embedding self-test failed: vector dimension is 0".to_string(),
        ));
    }

    // Probe check 2: Vector is not completely zero
    let is_all_zero = vec.iter().all(|&x| x == 0f32);
    if is_all_zero {
        return Err(ContextraError::Internal(
            "Embedding-Selbsttest fehlgeschlagen: Vektor ist komplett Null / Embedding self-test failed: vector is all zeros".to_string(),
        ));
    }

    // Probe check 3: Norm is plausible (finite, non-zero, within 1e-6..1e6)
    let norm: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
    if !norm.is_finite() || norm < 1e-6 || norm > 1e6 {
        return Err(ContextraError::Internal(format!(
            "Embedding-Selbsttest fehlgeschlagen: Unplausibler Normwert {norm} / Embedding self-test failed: implausible norm {norm}"
        )));
    }

    // Probe check 4: Dimension matches expected_dim if provided
    if let Some(exp) = expected_dim {
        if vec.len() != exp {
            return Err(ContextraError::Internal(format!(
                "Embedding-Selbsttest fehlgeschlagen: Dimension {got} stimmt nicht mit erwarteter Indexdimension {exp} überein / Dimension mismatch: got {got}, expected {exp}",
                got = vec.len()
            )));
        }
    }

    Ok(())
}

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
            value: String::new(),
            used_unprefixed_fallback: false,
        },
    }
}

/// Embedding provider configuration settings.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EmbeddingConfig {
    /// Provider type ("ollama", "onnx", "candle", or "mock"). Unconfigured when empty.
    pub provider: String,
    /// Base URL for Ollama HTTP API.
    pub ollama_url: String,
    /// Model identifier for Ollama embeddings.
    pub embed_model: String,
    /// Optional path to ONNX model file or directory.
    pub onnx_model_path: Option<PathBuf>,
    /// Optional path to Candle model directory.
    pub candle_model_dir: Option<PathBuf>,
    /// Optional expected dimension for index alignment verification.
    pub expected_dim: Option<usize>,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            provider: String::new(),
            ollama_url: "http://localhost:11434".to_string(),
            embed_model: "nomic-embed-text".to_string(),
            onnx_model_path: None,
            candle_model_dir: None,
            expected_dim: None,
        }
    }
}

impl EmbeddingConfig {
    /// Loads configuration from environment variables with fallbacks.
    pub fn from_env() -> Self {
        let prefixed = std::env::var("CONTEXTRA_EMBEDDING_PROVIDER").ok();
        let unprefixed = std::env::var("EMBEDDING_PROVIDER").ok();
        let resolved = resolve_provider_setting(prefixed.as_deref(), unprefixed.as_deref());

        if resolved.used_unprefixed_fallback {
            EMBEDDING_PROVIDER_DEPRECATION_WARN_ONCE.call_once(|| {
                tracing::warn!(
                    "Ungeprefixte Variable EMBEDDING_PROVIDER ist deprecated, bitte CONTEXTRA_EMBEDDING_PROVIDER verwenden."
                );
            });
        }

        let provider = resolved.value;

        let ollama_url = std::env::var("CONTEXTRA_OLLAMA_URL")
            .unwrap_or_else(|_| "http://localhost:11434".to_string());

        let embed_model = std::env::var("CONTEXTRA_EMBED_MODEL")
            .unwrap_or_else(|_| "nomic-embed-text".to_string());

        let onnx_model_path = std::env::var("CONTEXTRA_ONNX_MODEL_PATH")
            .ok()
            .map(PathBuf::from);

        let candle_model_dir = std::env::var("CONTEXTRA_CANDLE_MODEL_DIR")
            .ok()
            .map(PathBuf::from);

        Self {
            provider,
            ollama_url,
            embed_model,
            onnx_model_path,
            candle_model_dir,
            expected_dim: None,
        }
    }

    pub fn with_expected_dim(mut self, dim: usize) -> Self {
        self.expected_dim = Some(dim);
        self
    }

    pub fn provider_opt(&self) -> Option<&str> {
        let trimmed = self.provider.trim();
        if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("none") {
            None
        } else {
            Some(trimmed)
        }
    }

    /// Instantiates the configured `EmbeddingProvider` as an `Arc<dyn EmbeddingProvider>` and runs startup probe.
    pub fn build_provider(&self) -> Result<Arc<dyn EmbeddingProvider>, ContextraError> {
        let provider = create_embedding_provider(
            self.provider_opt(),
            &self.ollama_url,
            &self.embed_model,
            self.onnx_model_path.as_deref(),
            self.candle_model_dir.as_deref(),
        )?;
        verify_embedding_provider(provider.as_ref(), self.expected_dim)?;
        Ok(provider)
    }
}

/// Dynamically constructs an `EmbeddingProvider` implementation based on provider identifier.
pub fn create_embedding_provider(
    provider_type: Option<&str>,
    ollama_url: &str,
    embed_model: &str,
    onnx_model_path: Option<&Path>,
    candle_model_dir: Option<&Path>,
) -> Result<Arc<dyn EmbeddingProvider>, ContextraError> {
    let provider = match provider_type {
        Some(p) if !p.trim().is_empty() => p.trim().to_lowercase(),
        _ => return Err(no_provider_error()),
    };

    let _ = (ollama_url, embed_model, onnx_model_path, candle_model_dir);

    match provider.as_str() {
        #[cfg(feature = "candle")]
        "candle" => {
            let model_dir = candle_model_dir.ok_or_else(|| {
                ContextraError::InvalidInput(
                    "candle_model_dir is required when embedding provider is 'candle'".to_string(),
                )
            })?;
            let quantization = contextra_infer_candle::model_registry::CandleQuantization::Q4KM;
            let embedder =
                contextra_infer_candle::CandleEmbedClient::from_dir(model_dir, quantization)
                    .map_err(|e| {
                        ContextraError::Internal(format!("Failed to load Candle embed model: {e}"))
                    })?;
            Ok(Arc::new(embedder))
        }
        #[cfg(not(feature = "candle"))]
        "candle" => Err(uncompiled_provider_error("candle")),

        #[cfg(feature = "ollama")]
        "ollama" => {
            let embedder = contextra_infer_ollama::OllamaEmbedder::new(ollama_url, embed_model);
            Ok(Arc::new(embedder))
        }
        #[cfg(not(feature = "ollama"))]
        "ollama" => Err(uncompiled_provider_error("ollama")),

        #[cfg(feature = "onnx")]
        "onnx" => {
            let embedder = if let Some(path) = onnx_model_path {
                contextra_infer_onnx::OnnxEmbedder::from_path(path)
                    .map_err(|e| ContextraError::Internal(format!("Failed to load ONNX model: {e}")))?
            } else {
                let cache_dir = contextra_infer_onnx::default_model_cache_dir("nomic-embed-text");
                contextra_infer_onnx::OnnxEmbedder::load(&cache_dir)
                    .map_err(|e| ContextraError::Internal(format!("Failed to load ONNX model from cache: {e}")))?
            };
            Ok(Arc::new(embedder))
        }
        #[cfg(not(feature = "onnx"))]
        "onnx" => Err(uncompiled_provider_error("onnx")),

        #[cfg(any(test, feature = "test-utils"))]
        "mock" => {
            let embedder = MockEmbedder::new(768);
            Ok(Arc::new(embedder))
        }
        #[cfg(not(any(test, feature = "test-utils")))]
        "mock" => Err(uncompiled_provider_error("mock")),

        other => Err(unknown_provider_error(other)),
    }
}

/// LLM provider configuration settings.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LlmConfig {
    /// Provider type ("ollama", "candle", or "mock"). Unconfigured when empty.
    pub provider: String,
    /// Base URL for Ollama HTTP API.
    pub ollama_url: String,
    /// Model identifier for Ollama LLM.
    pub llm_model: String,
    /// Optional path to Candle model directory.
    pub candle_model_dir: Option<PathBuf>,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: String::new(),
            ollama_url: "http://localhost:11434".to_string(),
            llm_model: "llama3.2:3b".to_string(),
            candle_model_dir: None,
        }
    }
}

impl LlmConfig {
    /// Loads configuration from environment variables with fallbacks.
    pub fn from_env() -> Self {
        let prefixed = std::env::var("CONTEXTRA_LLM_PROVIDER").ok();
        let unprefixed = std::env::var("LLM_PROVIDER").ok();
        let resolved = resolve_provider_setting(prefixed.as_deref(), unprefixed.as_deref());

        if resolved.used_unprefixed_fallback {
            LLM_PROVIDER_DEPRECATION_WARN_ONCE.call_once(|| {
                tracing::warn!(
                    "Ungeprefixte Variable LLM_PROVIDER ist deprecated, bitte CONTEXTRA_LLM_PROVIDER verwenden."
                );
            });
        }

        let provider = resolved.value;

        let ollama_url = std::env::var("CONTEXTRA_OLLAMA_URL")
            .unwrap_or_else(|_| "http://localhost:11434".to_string());

        let llm_model =
            std::env::var("CONTEXTRA_LLM_MODEL").unwrap_or_else(|_| "llama3.2:3b".to_string());

        let candle_model_dir = std::env::var("CONTEXTRA_CANDLE_MODEL_DIR")
            .ok()
            .map(PathBuf::from);

        Self {
            provider,
            ollama_url,
            llm_model,
            candle_model_dir,
        }
    }

    pub fn provider_opt(&self) -> Option<&str> {
        let trimmed = self.provider.trim();
        if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("none") {
            None
        } else {
            Some(trimmed)
        }
    }

    /// Instantiates the configured `LlmTextGenerator` as an `Arc<dyn LlmTextGenerator>`.
    pub fn build_generator(&self) -> Result<Arc<dyn LlmTextGenerator>, ContextraError> {
        create_llm_text_generator(
            self.provider_opt(),
            &self.ollama_url,
            &self.llm_model,
            self.candle_model_dir.as_deref(),
        )
    }
}

/// Dynamically constructs an `LlmTextGenerator` implementation based on provider identifier.
pub fn create_llm_text_generator(
    provider_type: Option<&str>,
    ollama_url: &str,
    llm_model: &str,
    candle_model_dir: Option<&Path>,
) -> Result<Arc<dyn LlmTextGenerator>, ContextraError> {
    let provider = match provider_type {
        Some(p) if !p.trim().is_empty() => p.trim().to_lowercase(),
        _ => {
            return Err(ContextraError::InvalidInput(
                "Kein LLM-Generator konfiguriert / No LLM generator configured.".to_string(),
            ))
        }
    };

    let _ = (ollama_url, llm_model, candle_model_dir);

    match provider.as_str() {
        #[cfg(feature = "candle")]
        "candle" => {
            let model_dir = candle_model_dir.ok_or_else(|| {
                ContextraError::InvalidInput(
                    "candle_model_dir is required when LLM provider is 'candle'".to_string(),
                )
            })?;
            let quantization = contextra_infer_candle::model_registry::CandleQuantization::Q4KM;
            let generator =
                contextra_infer_candle::CandleLlmClient::from_dir(model_dir, quantization)
                    .map_err(|e| {
                        ContextraError::Internal(format!("Failed to load Candle LLM model: {e}"))
                    })?;
            Ok(Arc::new(generator))
        }
        #[cfg(not(feature = "candle"))]
        "candle" => Err(uncompiled_provider_error("candle")),

        #[cfg(feature = "ollama")]
        "ollama" => {
            let config = contextra_infer_ollama::OllamaConfig {
                base_url: ollama_url.to_string(),
                model: llm_model.to_string(),
                ..Default::default()
            };
            let client = contextra_infer_ollama::OllamaClient::with_config(config);
            Ok(Arc::new(client))
        }
        #[cfg(not(feature = "ollama"))]
        "ollama" => Err(uncompiled_provider_error("ollama")),

        #[cfg(any(test, feature = "test-utils"))]
        "mock" => {
            let generator = MockLlmGenerator;
            Ok(Arc::new(generator))
        }
        #[cfg(not(any(test, feature = "test-utils")))]
        "mock" => Err(uncompiled_provider_error("mock")),

        other => Err(unknown_provider_error(other)),
    }
}

/// Router configuration settings.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct RouterConfig {
    /// List of SLM profiles. If non-empty, routing is enabled.
    pub profiles: Vec<contextra_router::SlmProfile>,
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
                    serde_json::from_slice::<Vec<contextra_router::SlmProfile>>(&bytes)
                {
                    profiles = loaded;
                }
            }
        } else if let Ok(json_str) = std::env::var("CONTEXTRA_ROUTER_PROFILES_JSON") {
            if let Ok(loaded) = serde_json::from_str::<Vec<contextra_router::SlmProfile>>(&json_str)
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

/// Fallback Mock Embedder (Test & dev utilities only).
#[cfg(any(test, feature = "test-utils"))]
#[derive(Debug, Clone)]
pub struct MockEmbedder {
    dimension: usize,
    all_zero: bool,
}

#[cfg(any(test, feature = "test-utils"))]
impl MockEmbedder {
    pub fn new(dimension: usize) -> Self {
        Self {
            dimension,
            all_zero: false,
        }
    }

    pub fn with_zero_vectors(dimension: usize) -> Self {
        Self {
            dimension,
            all_zero: true,
        }
    }
}

#[cfg(any(test, feature = "test-utils"))]
impl std::fmt::Display for MockEmbedder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "MockEmbedder(dim={}, TEST_ONLY=true)",
            self.dimension
        )
    }
}

#[cfg(any(test, feature = "test-utils"))]
impl EmbeddingProvider for MockEmbedder {
    fn provider_name(&self) -> &str {
        "mock"
    }

    fn embed<'a>(
        &'a self,
        _text: &'a str,
    ) -> contextra_ports::BoxFuture<'a, Result<Vec<f32>, contextra_ports::EmbeddingError>> {
        tracing::warn!("MockEmbedder embed invoked (TEST/DEV ONLY - NOT FOR PRODUCTION)");
        let dim = self.dimension;
        let all_zero = self.all_zero;
        Box::pin(async move {
            if all_zero {
                Ok(vec![0f32; dim])
            } else {
                let val = 1.0f32 / (dim as f32).sqrt();
                Ok(vec![val; dim])
            }
        })
    }

    fn embedding_dim(&self) -> usize {
        self.dimension
    }
}

/// Fallback Mock LLM Text Generator (Test & dev utilities only).
#[cfg(any(test, feature = "test-utils"))]
#[derive(Debug, Clone, Default)]
pub struct MockLlmGenerator;

#[cfg(any(test, feature = "test-utils"))]
impl std::fmt::Display for MockLlmGenerator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MockLlmGenerator(TEST_ONLY=true)")
    }
}

#[cfg(any(test, feature = "test-utils"))]
impl LlmTextGenerator for MockLlmGenerator {
    fn generate<'a>(
        &'a self,
        prompt: &'a str,
    ) -> contextra_ports::BoxFuture<'a, Result<String, ContextraError>> {
        tracing::warn!("MockLlmGenerator generate invoked (TEST/DEV ONLY - NOT FOR PRODUCTION)");
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
        let res = resolve_provider_setting(Some("onnx"), Some("ollama"));
        assert_eq!(res.value, "onnx");
        assert!(!res.used_unprefixed_fallback);
    }

    #[test]
    fn test_resolve_provider_setting_unprefixed_fallback() {
        let res = resolve_provider_setting(None, Some("ollama"));
        assert_eq!(res.value, "ollama");
        assert!(res.used_unprefixed_fallback);
    }

    #[test]
    fn test_resolve_provider_setting_default_empty() {
        let res = resolve_provider_setting(None, None);
        assert!(res.value.is_empty());
        assert!(!res.used_unprefixed_fallback);
    }

    #[test]
    fn test_embedding_config_defaults_unconfigured() {
        let config = EmbeddingConfig::default();
        assert!(config.provider.is_empty());
        assert!(config.provider_opt().is_none());
        assert_eq!(config.ollama_url, "http://localhost:11434");
        assert_eq!(config.embed_model, "nomic-embed-text");
        assert!(config.onnx_model_path.is_none());
        assert!(config.candle_model_dir.is_none());
        assert!(config.expected_dim.is_none());
    }

    #[test]
    fn test_llm_config_defaults_unconfigured() {
        let config = LlmConfig::default();
        assert!(config.provider.is_empty());
        assert!(config.provider_opt().is_none());
        assert_eq!(config.ollama_url, "http://localhost:11434");
        assert_eq!(config.llm_model, "llama3.2:3b");
        assert!(config.candle_model_dir.is_none());
    }

    #[test]
    fn test_create_embedding_provider_no_provider_error() {
        let res = create_embedding_provider(
            None,
            "http://localhost:11434",
            "nomic-embed-text",
            None,
            None,
        );
        assert!(res.is_err());
        let err = res.err().unwrap();
        assert!(matches!(err, ContextraError::InvalidInput(_)));
        let err_msg = err.to_string();
        assert!(err_msg.contains("Kein Embedding-Provider konfiguriert"));
        assert!(err_msg.contains("No embedding provider configured"));
    }

    #[test]
    fn test_create_llm_generator_no_provider_error() {
        let res = create_llm_text_generator(
            None,
            "http://localhost:11434",
            "llama3.2:3b",
            None,
        );
        assert!(res.is_err());
        let err = res.err().unwrap();
        assert!(matches!(err, ContextraError::InvalidInput(_)));
        let err_msg = err.to_string();
        assert!(err_msg.contains("Kein LLM-Generator konfiguriert"));
    }

    #[test]
    fn test_create_embedding_provider_unknown_error_lists_available() {
        let res = create_embedding_provider(
            Some("invalid_provider"),
            "http://localhost:11434",
            "nomic-embed-text",
            None,
            None,
        );
        assert!(res.is_err());
        let err = res.err().unwrap();
        assert!(matches!(err, ContextraError::InvalidInput(_)));
        let err_msg = err.to_string();
        assert!(err_msg.contains("Unbekannter Embedding-Provider 'invalid_provider'"));
    }

    #[test]
    fn test_create_llm_generator_unknown_error_lists_available() {
        let res = create_llm_text_generator(
            Some("invalid_provider"),
            "http://localhost:11434",
            "llama3.2:3b",
            None,
        );
        assert!(res.is_err());
        let err = res.err().unwrap();
        assert!(matches!(err, ContextraError::InvalidInput(_)));
        let err_msg = err.to_string();
        assert!(err_msg.contains("Unbekannter Embedding-Provider 'invalid_provider'"));
    }

    #[test]
    fn test_mock_embedder_available_in_test() {
        let provider = create_embedding_provider(
            Some("mock"),
            "http://localhost:11434",
            "nomic-embed-text",
            None,
            None,
        )
        .unwrap();
        assert_eq!(provider.provider_name(), "mock");
        assert_eq!(provider.embedding_dim(), 768);
    }

    #[test]
    fn test_mock_embedder_display_and_logging() {
        let mock = MockEmbedder::new(768);
        let display_str = format!("{mock}");
        assert!(display_str.contains("MockEmbedder"));
        assert!(display_str.contains("dim=768"));
        assert!(display_str.contains("TEST_ONLY=true"));
    }

    #[test]
    fn test_mock_llm_generator_display() {
        let generator = MockLlmGenerator;
        let display_str = format!("{generator}");
        assert!(display_str.contains("MockLlmGenerator"));
        assert!(display_str.contains("TEST_ONLY=true"));
    }

    #[test]
    fn test_verify_embedding_provider_zero_vector_fails() {
        let zero_mock = MockEmbedder::with_zero_vectors(768);
        let res = verify_embedding_provider(&zero_mock, None);
        assert!(res.is_err());
        let err_msg = res.err().unwrap().to_string();
        assert!(err_msg.contains("Vektor ist komplett Null"));
    }

    #[test]
    fn test_verify_embedding_provider_dimension_mismatch_fails() {
        let mock = MockEmbedder::new(768);
        let res = verify_embedding_provider(&mock, Some(1536));
        assert!(res.is_err());
        let err_msg = res.err().unwrap().to_string();
        assert!(err_msg.contains("Dimension 768 stimmt nicht mit erwarteter Indexdimension 1536 überein"));
    }

    #[test]
    fn test_verify_embedding_provider_success() {
        let mock = MockEmbedder::new(768);
        let res = verify_embedding_provider(&mock, Some(768));
        assert!(res.is_ok());
    }

    #[test]
    fn test_embedding_config_with_expected_dim() {
        let config = EmbeddingConfig::default().with_expected_dim(768);
        assert_eq!(config.expected_dim, Some(768));
    }

    #[test]
    fn test_available_compiled_providers_contains_mock_in_test() {
        let avail = available_compiled_providers();
        assert!(avail.contains(&"mock"));
    }

    #[test]
    fn test_build_provider_unconfigured_error() {
        let config = EmbeddingConfig::default();
        let res = config.build_provider();
        assert!(res.is_err());
        let err_msg = res.err().unwrap().to_string();
        assert!(err_msg.contains("Kein Embedding-Provider konfiguriert"));
    }

    #[test]
    fn test_build_provider_mock_success() {
        let config = EmbeddingConfig {
            provider: "mock".to_string(),
            ..Default::default()
        };
        let provider = config.build_provider().unwrap();
        assert_eq!(provider.provider_name(), "mock");
    }

    #[test]
    fn test_build_generator_unconfigured_error() {
        let config = LlmConfig::default();
        let res = config.build_generator();
        assert!(res.is_err());
        let err_msg = res.err().unwrap().to_string();
        assert!(err_msg.contains("Kein LLM-Generator konfiguriert"));
    }

    #[test]
    fn test_build_generator_mock_success() {
        let config = LlmConfig {
            provider: "mock".to_string(),
            ..Default::default()
        };
        let generator = config.build_generator().unwrap();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let resp = rt.block_on(generator.generate("hello")).unwrap();
        assert!(resp.contains("[Mock LLM response for: hello]"));
    }

    #[cfg(feature = "candle")]
    #[test]
    fn test_create_embedding_provider_candle_success() {
        let tmp = tempfile::tempdir().unwrap();
        let provider = create_embedding_provider(
            Some("candle"),
            "http://localhost:11434",
            "embed_model",
            None,
            Some(tmp.path()),
        )
        .unwrap();
        assert_eq!(provider.provider_name(), "candle");
    }

    #[cfg(not(feature = "candle"))]
    #[test]
    fn test_create_embedding_provider_candle_uncompiled_error() {
        let res = create_embedding_provider(
            Some("candle"),
            "http://localhost:11434",
            "embed_model",
            None,
            None,
        );
        assert!(res.is_err());
        let err = res.err().unwrap();
        assert!(matches!(
            err,
            ContextraError::CapabilityUnsupported { .. }
        ));
        let msg = err.to_string();
        assert!(msg.contains("Provider 'candle' ist nicht einkompiliert"));
    }

    #[cfg(feature = "ollama")]
    #[test]
    fn test_create_embedding_provider_ollama_success() {
        let provider = create_embedding_provider(
            Some("ollama"),
            "http://localhost:11434",
            "nomic-embed-text",
            None,
            None,
        )
        .unwrap();
        assert_eq!(provider.provider_name(), "ollama");
    }

    #[cfg(not(feature = "ollama"))]
    #[test]
    fn test_create_embedding_provider_ollama_uncompiled_error() {
        let res = create_embedding_provider(
            Some("ollama"),
            "http://localhost:11434",
            "nomic-embed-text",
            None,
            None,
        );
        assert!(res.is_err());
        let err = res.err().unwrap();
        assert!(matches!(
            err,
            ContextraError::CapabilityUnsupported { .. }
        ));
        let msg = err.to_string();
        assert!(msg.contains("Provider 'ollama' ist nicht einkompiliert"));
    }

    #[cfg(not(feature = "onnx"))]
    #[test]
    fn test_create_embedding_provider_onnx_uncompiled_error() {
        let res = create_embedding_provider(
            Some("onnx"),
            "http://localhost:11434",
            "nomic-embed-text",
            None,
            None,
        );
        assert!(res.is_err());
        let err = res.err().unwrap();
        assert!(matches!(
            err,
            ContextraError::CapabilityUnsupported { .. }
        ));
        let msg = err.to_string();
        assert!(msg.contains("Provider 'onnx' ist nicht einkompiliert"));
    }
}
