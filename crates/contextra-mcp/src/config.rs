// FILE-CONTEXT
// STAND:       2026-09-10T19:25:24Z (SESSION: ae8c2fb9)
// ZWECK:       Dynamic Provider Construction for Embedding & LLM backends (Ollama, ONNX, Candle, Mock)
// INVARIANTEN: Direct provider instantiation without leaking implementation details; fallback capability checking
// HOTSPOTS:    create_embedding_provider(), create_llm_text_generator()
// SIEHE AUCH:  ADR-010, contextra-core/src/traits/mod.rs

use contextra::EmbeddingBackend;
use contextra_ports::{EmbeddingProvider, LlmTextGenerator};
use contextra_types::ContextraError;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::{Arc, Once};

static EMBEDDING_PROVIDER_DEPRECATION_WARN_ONCE: Once = Once::new();
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
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EmbeddingConfig {
    /// Provider type ("ollama", "onnx", or "candle").
    pub provider: String,
    /// Base URL for Ollama HTTP API.
    pub ollama_url: String,
    /// Model identifier for Ollama embeddings.
    pub embed_model: String,
    /// Optional path to ONNX model file or directory.
    pub onnx_model_path: Option<PathBuf>,
    /// Optional path to Candle model directory.
    pub candle_model_dir: Option<PathBuf>,
    /// Candle model quantization grade (e.g. Q4KM, Q8_0, F16).
    pub candle_quantization: contextra_infer_candle::model_registry::CandleQuantization,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            provider: "mock".to_string(),
            ollama_url: "http://localhost:11434".to_string(),
            embed_model: "nomic-embed-text".to_string(),
            onnx_model_path: None,
            candle_model_dir: None,
            candle_quantization: contextra_infer_candle::model_registry::CandleQuantization::Q4KM,
        }
    }
}

impl EmbeddingConfig {
    /// Loads configuration from environment variables with fallbacks.
    pub fn from_env() -> Self {
        let prefixed = std::env::var("CONTEXTRA_EMBEDDING_PROVIDER").ok();
        let unprefixed = std::env::var("EMBEDDING_PROVIDER").ok();
        let resolved = resolve_provider_setting(prefixed.as_deref(), unprefixed.as_deref(), "mock");

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

        let quantization_str = std::env::var("CONTEXTRA_CANDLE_QUANTIZATION")
            .or_else(|_| std::env::var("CANDLE_QUANTIZATION"))
            .unwrap_or_else(|_| "Q4KM".to_string());

        let candle_quantization =
            contextra_infer_candle::model_registry::CandleQuantization::from_str(&quantization_str)
                .unwrap_or(contextra_infer_candle::model_registry::CandleQuantization::Q4KM);

        Self {
            provider,
            ollama_url,
            embed_model,
            onnx_model_path,
            candle_model_dir,
            candle_quantization,
        }
    }

    /// Instantiates the configured `EmbeddingProvider` as an `Arc<dyn EmbeddingProvider>`.
    pub fn build_provider(&self) -> Result<Arc<dyn EmbeddingProvider>, ContextraError> {
        let trimmed_provider = self.provider.trim();
        if trimmed_provider.is_empty() {
            return Err(ContextraError::InvalidInput(
                "Kein Embedding-Provider konfiguriert. Bitte CONTEXTRA_EMBEDDING_PROVIDER (oder EMBEDDING_PROVIDER) auf 'ollama', 'onnx', 'candle' oder 'mock' setzen.".to_string(),
            ));
        }

        let mut backend = EmbeddingBackend::from_str(trimmed_provider)?;

        match &mut backend {
            EmbeddingBackend::Candle {
                model_dir,
                quantization,
            } => {
                let dir = self.candle_model_dir.clone().ok_or_else(|| {
                    ContextraError::InvalidInput(
                        "candle_model_dir is required when embedding provider is 'candle'"
                            .to_string(),
                    )
                })?;
                *model_dir = dir;
                *quantization = self.candle_quantization;
            }
            EmbeddingBackend::Onnx {
                model_name,
                cache_dir: _,
            } => {
                let path = self.onnx_model_path.clone().ok_or_else(|| {
                    ContextraError::InvalidInput(
                        "onnx_model_path is required when embedding provider is 'onnx'".to_string(),
                    )
                })?;
                *model_name = path.to_string_lossy().to_string();
            }
            EmbeddingBackend::Ollama { endpoint, model } => {
                *endpoint = self.ollama_url.clone();
                *model = self.embed_model.clone();
            }
            EmbeddingBackend::None => {}
        }

        create_embedding_provider(&backend)
    }
}

/// Dynamically constructs an `EmbeddingProvider` implementation based on an `EmbeddingBackend`.
pub fn create_embedding_provider(
    backend: &EmbeddingBackend,
) -> Result<Arc<dyn EmbeddingProvider>, ContextraError> {
    match provider_type.to_lowercase().trim() {
        #[cfg(feature = "ollama")]
        EmbeddingBackend::Ollama { endpoint, model } => {
            let embedder = contextra_infer_ollama::OllamaEmbedder::new(endpoint, model);
            Ok(Arc::new(embedder))
        }
        #[cfg(not(feature = "ollama"))]
        EmbeddingBackend::Ollama { .. } => Err(ContextraError::CapabilityUnsupported {
            capability: "ollama embedding backend".to_string(),
            reason: "contextra-mcp was built without the 'ollama' feature".to_string(),
        }),
        #[cfg(feature = "onnx")]
        EmbeddingBackend::Onnx { model_name, .. } => {
            let embedder = contextra_infer_onnx::OnnxEmbedder::from_path(Path::new(model_name))?;
            Ok(Arc::new(embedder))
        }
        #[cfg(not(feature = "onnx"))]
        EmbeddingBackend::Onnx { .. } => Err(ContextraError::CapabilityUnsupported {
            capability: "onnx".to_string(),
            reason: "ONNX support is disabled in this build. Recompile with feature flag 'onnx'."
                .to_string(),
        }),
        #[cfg(feature = "candle")]
        EmbeddingBackend::Candle {
            model_dir,
            quantization,
        } => {
            let embedder =
                contextra_infer_candle::CandleEmbedClient::from_dir(model_dir, *quantization)
                    .map_err(|e| {
                        ContextraError::Internal(format!("Failed to load Candle embed model: {e}"))
                    })?;
            Ok(Arc::new(embedder))
        }
        #[cfg(not(feature = "candle"))]
        "candle" => {
            let _ = (ollama_url, embed_model, onnx_model_path, candle_model_dir);
            Err(ContextraError::CapabilityUnsupported {
                capability: "candle embedding backend".to_string(),
                reason: "contextra-mcp was built without the 'candle' feature".to_string(),
            })
        }
        #[cfg(any(test, feature = "test-utils"))]
        "mock" => {
            let embedder = contextra_ports::MockEmbedder::new(768);
            Ok(Arc::new(embedder))
        }
        other => Err(ContextraError::InvalidInput(format!(
            "Unknown embedding provider '{other}'. Expected 'ollama', 'onnx', or 'candle'."
        ))),
    }
}

/// LLM provider configuration settings.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LlmConfig {
    /// Provider type ("ollama", or "candle").
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
            provider: "mock".to_string(),
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
        let resolved = resolve_provider_setting(prefixed.as_deref(), unprefixed.as_deref(), "mock");

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

    // TODO(wiring): Facade-Anbindung in contextra/src/builder.rs folgt in separatem Task
    /// Instantiates the configured `LlmTextGenerator` as an `Arc<dyn LlmTextGenerator>`.
    pub fn build_generator(&self) -> Result<Arc<dyn LlmTextGenerator>, ContextraError> {
        create_llm_text_generator(
            &self.provider,
            &self.ollama_url,
            &self.llm_model,
            self.candle_model_dir.as_deref(),
        )
    }
}

/// Dynamically constructs an `LlmTextGenerator` implementation based on provider identifier.
pub fn create_llm_text_generator(
    provider_type: &str,
    ollama_url: &str,
    llm_model: &str,
    candle_model_dir: Option<&Path>,
) -> Result<Arc<dyn LlmTextGenerator>, ContextraError> {
    match provider_type.to_lowercase().trim() {
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
        "ollama" => {
            let _ = (ollama_url, llm_model);
            Err(ContextraError::CapabilityUnsupported {
                capability: "ollama LLM backend".to_string(),
                reason: "contextra-mcp was built without the 'ollama' feature".to_string(),
            })
        }
        #[cfg(feature = "candle")]
        "candle" => {
            let _ = (ollama_url, llm_model);
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
        "candle" => {
            let _ = (ollama_url, llm_model, candle_model_dir);
            Err(ContextraError::CapabilityUnsupported {
                capability: "candle LLM backend".to_string(),
                reason: "contextra-mcp was built without the 'candle' feature".to_string(),
            })
        }
        #[cfg(any(test, feature = "test-utils"))]
        "mock" => {
            let generator = MockLlmGenerator;
            Ok(Arc::new(generator))
        }
        other => Err(ContextraError::InvalidInput(format!(
            "Unknown LLM provider '{other}'. Expected 'ollama' or 'candle'."
        ))),
    }
}

/// Router configuration settings.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
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
        let res = resolve_provider_setting(Some("onnx"), Some("ollama"), "mock");
        assert_eq!(res.value, "onnx");
        assert!(!res.used_unprefixed_fallback);
    }

    #[test]
    fn test_resolve_provider_setting_unprefixed_fallback() {
        let res = resolve_provider_setting(None, Some("ollama"), "mock");
        assert_eq!(res.value, "ollama");
        assert!(res.used_unprefixed_fallback);
    }

    #[test]
    fn test_resolve_provider_setting_default() {
        let res = resolve_provider_setting(None, None, "mock");
        assert_eq!(res.value, "mock");
        assert!(!res.used_unprefixed_fallback);
    }

    #[test]
    fn test_embedding_config_defaults() {
        let config = EmbeddingConfig::default();
        assert_eq!(config.provider, "mock");
        assert_eq!(config.ollama_url, "http://localhost:11434");
        assert_eq!(config.embed_model, "nomic-embed-text");
        assert!(config.onnx_model_path.is_none());
        assert!(config.candle_model_dir.is_none());
    }

    #[test]
    fn test_llm_config_defaults() {
        let config = LlmConfig::default();
        assert_eq!(config.provider, "mock");
        assert_eq!(config.ollama_url, "http://localhost:11434");
        assert_eq!(config.llm_model, "llama3.2:3b");
        assert!(config.candle_model_dir.is_none());
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

    #[cfg(feature = "ollama")]
    #[test]
    fn test_create_embedding_provider_ollama() {
        let backend = EmbeddingBackend::Ollama {
            endpoint: "http://localhost:11434".to_string(),
            model: "nomic-embed-text".to_string(),
        };
        let provider = create_embedding_provider(&backend).unwrap();
        assert_eq!(provider.provider_name(), "ollama");
    }

    #[test]
    fn test_create_embedding_provider_unknown_error() {
        let config = EmbeddingConfig {
            provider: "invalid_provider".to_string(),
            ..Default::default()
        };
        let res = config.build_provider();
        assert!(matches!(res, Err(ContextraError::InvalidInput(_))));
    }

    #[test]
    fn test_create_llm_generator_mock() {
        let generator =
            create_llm_text_generator("mock", "http://localhost:11434", "llama3.2:3b", None)
                .unwrap();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let response = rt.block_on(generator.generate("hello")).unwrap();
        assert!(response.contains("[Mock LLM response for: hello]"));
    }

    #[cfg(feature = "ollama")]
    #[test]
    fn test_create_llm_generator_ollama() {
        let generator =
            create_llm_text_generator("ollama", "http://localhost:11434", "llama3.2:3b", None);
        assert!(generator.is_ok());
    }

    #[test]
    fn test_create_llm_generator_unknown_error() {
        let res = create_llm_text_generator(
            "invalid_provider",
            "http://localhost:11434",
            "llama3.2:3b",
            None,
        );
        assert!(matches!(res, Err(ContextraError::InvalidInput(_))));
    }

    #[cfg(feature = "candle")]
    #[test]
    fn test_create_embedding_provider_candle_success() {
        let tmp = tempfile::tempdir().unwrap();
        let backend = EmbeddingBackend::Candle {
            model_dir: tmp.path().to_path_buf(),
            quantization: contextra_infer_candle::model_registry::CandleQuantization::Q4KM,
        };
        let provider = create_embedding_provider(&backend).unwrap();
        assert_eq!(provider.provider_name(), "candle");
    }

    #[cfg(feature = "candle")]
    #[test]
    fn test_create_embedding_provider_candle_missing_dir_error() {
        let config = EmbeddingConfig {
            provider: "candle".to_string(),
            candle_model_dir: None,
            ..Default::default()
        };
        let res = config.build_provider();
        match res {
            Err(err) => {
                assert!(matches!(err, ContextraError::InvalidInput(_)));
                assert!(err.to_string().contains("candle_model_dir is required"));
            }
            Ok(_) => panic!("Expected error for missing candle_model_dir"),
        }
    }

    #[cfg(not(feature = "candle"))]
    #[test]
    fn test_create_embedding_provider_candle_unsupported_error() {
        let backend = EmbeddingBackend::Candle {
            model_dir: PathBuf::from("/tmp"),
            quantization: contextra_infer_candle::model_registry::CandleQuantization::Q4KM,
        };
        let res = create_embedding_provider(&backend);
        assert!(matches!(
            res,
            Err(ContextraError::CapabilityUnsupported { .. })
        ));
    }

    #[cfg(feature = "candle")]
    #[test]
    fn test_create_llm_generator_candle_success() {
        let tmp = tempfile::tempdir().unwrap();
        let generator = create_llm_text_generator(
            "candle",
            "http://localhost:11434",
            "llama3.2:3b",
            Some(tmp.path()),
        )
        .unwrap();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let response = rt.block_on(generator.generate("hello")).unwrap();
        assert!(response.contains("Response for prompt: hello"));
    }

    #[cfg(feature = "candle")]
    #[test]
    fn test_create_llm_generator_candle_missing_dir_error() {
        let res =
            create_llm_text_generator("candle", "http://localhost:11434", "llama3.2:3b", None);
        match res {
            Err(err) => {
                assert!(matches!(err, ContextraError::InvalidInput(_)));
                assert!(err.to_string().contains("candle_model_dir is required"));
            }
            Ok(_) => panic!("Expected error for missing candle_model_dir"),
        }
    }

    #[cfg(not(feature = "candle"))]
    #[test]
    fn test_create_llm_generator_candle_unsupported_error() {
        let res =
            create_llm_text_generator("candle", "http://localhost:11434", "llama3.2:3b", None);
        assert!(matches!(
            res,
            Err(ContextraError::CapabilityUnsupported { .. })
        ));
    }
}
