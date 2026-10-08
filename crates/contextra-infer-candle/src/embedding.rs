// FILE-CONTEXT
// STAND: 2026-09-12T00:00:00Z (SESSION: BACKPRESSURE-CONTRACT-D1)
// ZWECK: Native Candle ML vector embedding client implementation.
// INVARIANTEN: Thread safety via Arc<tokio::sync::Mutex<Box<dyn CandleEmbedInner>>>; vector dimension matches model.dim. Zero unsafe code in production via VarBuilder::from_buffered_safetensors.
// NICHT-OFFENSICHTLICH: CandleEmbedInner trait enables mock-based unit testing without binary weights in CI.
// Backpressure contract: max_concurrent_embeddings limits spawn_blocking calls.

use crate::model_registry::ModelFingerprint;
use candle_core::{Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::bert::{BertModel, Config, DTYPE};
use contextra_types::{ContextraError, Result};
use std::path::Path;
use std::sync::Arc;

/// Default maximum concurrent embedding operations for Candle vector embedding.
pub const DEFAULT_MAX_CONCURRENT_EMBEDDINGS: usize = 8;

/// Loads BERT model configuration from `config.json` at `path`.
///
/// Returns an error containing the path if the file is missing or invalid JSON.
pub fn load_bert_config(path: &Path) -> Result<Config> {
    let content = std::fs::read_to_string(path).map_err(|e| {
        ContextraError::InvalidInput(format!(
            "Failed to read BERT config file {}: {e}",
            path.display()
        ))
    })?;

    parse_bert_config_json(&content).map_err(|e| {
        ContextraError::InvalidInput(format!(
            "Failed to parse BERT config JSON from {}: {e}",
            path.display()
        ))
    })
}

fn parse_bert_config_json(json_str: &str) -> std::result::Result<Config, String> {
    let trimmed = json_str.trim();
    if !trimmed.starts_with('{') || !trimmed.ends_with('}') {
        return Err("JSON must be a valid JSON object enclosed in { ... }".to_string());
    }

    let mut config = Config::default();

    let chars: Vec<char> = trimmed.chars().collect();
    let mut i = 1;
    let len = chars.len() - 1;

    while i < len {
        while i < len && (chars[i].is_whitespace() || chars[i] == ',') {
            i += 1;
        }
        if i >= len {
            break;
        }

        if chars[i] != '"' {
            return Err(format!("Expected quoted key at character offset {i}"));
        }
        i += 1;
        let key_start = i;
        while i < len && chars[i] != '"' {
            if chars[i] == '\\' {
                i += 1;
            }
            i += 1;
        }
        if i >= len {
            return Err("Unterminated string key".to_string());
        }
        let key: String = chars[key_start..i].iter().collect();
        i += 1;

        while i < len && chars[i].is_whitespace() {
            i += 1;
        }
        if i >= len || chars[i] != ':' {
            return Err(format!("Expected ':' after key '{key}'"));
        }
        i += 1;

        while i < len && chars[i].is_whitespace() {
            i += 1;
        }
        if i >= len {
            return Err(format!("Expected value for key '{key}'"));
        }

        let value_start = i;
        if chars[i] == '"' {
            i += 1;
            while i < len && chars[i] != '"' {
                if chars[i] == '\\' {
                    i += 1;
                }
                i += 1;
            }
            if i >= len {
                return Err(format!("Unterminated string value for key '{key}'"));
            }
            let val_str: String = chars[value_start + 1..i].iter().collect();
            i += 1;

            if key == "model_type" {
                config.model_type = Some(val_str);
            }
        } else if chars[i] == '[' {
            let mut depth = 1;
            i += 1;
            while i < len && depth > 0 {
                if chars[i] == '[' {
                    depth += 1;
                } else if chars[i] == ']' {
                    depth -= 1;
                } else if chars[i] == '"' {
                    i += 1;
                    while i < len && chars[i] != '"' {
                        if chars[i] == '\\' {
                            i += 1;
                        }
                        i += 1;
                    }
                }
                i += 1;
            }
            if depth != 0 {
                return Err(format!("Unterminated array for key '{key}'"));
            }
        } else if chars[i] == '{' {
            let mut depth = 1;
            i += 1;
            while i < len && depth > 0 {
                if chars[i] == '{' {
                    depth += 1;
                } else if chars[i] == '}' {
                    depth -= 1;
                } else if chars[i] == '"' {
                    i += 1;
                    while i < len && chars[i] != '"' {
                        if chars[i] == '\\' {
                            i += 1;
                        }
                        i += 1;
                    }
                }
                i += 1;
            }
            if depth != 0 {
                return Err(format!("Unterminated object for key '{key}'"));
            }
        } else {
            while i < len && chars[i] != ',' && chars[i] != '}' && !chars[i].is_whitespace() {
                i += 1;
            }
            let token: String = chars[value_start..i].iter().collect();

            match key.as_str() {
                "vocab_size" => {
                    config.vocab_size = token
                        .parse()
                        .map_err(|e| format!("Invalid vocab_size '{token}': {e}"))?;
                }
                "hidden_size" => {
                    config.hidden_size = token
                        .parse()
                        .map_err(|e| format!("Invalid hidden_size '{token}': {e}"))?;
                }
                "num_hidden_layers" => {
                    config.num_hidden_layers = token
                        .parse()
                        .map_err(|e| format!("Invalid num_hidden_layers '{token}': {e}"))?;
                }
                "num_attention_heads" => {
                    config.num_attention_heads = token
                        .parse()
                        .map_err(|e| format!("Invalid num_attention_heads '{token}': {e}"))?;
                }
                "intermediate_size" => {
                    config.intermediate_size = token
                        .parse()
                        .map_err(|e| format!("Invalid intermediate_size '{token}': {e}"))?;
                }
                "hidden_dropout_prob" => {
                    config.hidden_dropout_prob = token
                        .parse()
                        .map_err(|e| format!("Invalid hidden_dropout_prob '{token}': {e}"))?;
                }
                "max_position_embeddings" => {
                    config.max_position_embeddings = token
                        .parse()
                        .map_err(|e| format!("Invalid max_position_embeddings '{token}': {e}"))?;
                }
                "type_vocab_size" => {
                    config.type_vocab_size = token
                        .parse()
                        .map_err(|e| format!("Invalid type_vocab_size '{token}': {e}"))?;
                }
                "initializer_range" => {
                    config.initializer_range = token
                        .parse()
                        .map_err(|e| format!("Invalid initializer_range '{token}': {e}"))?;
                }
                "layer_norm_eps" => {
                    config.layer_norm_eps = token
                        .parse()
                        .map_err(|e| format!("Invalid layer_norm_eps '{token}': {e}"))?;
                }
                "pad_token_id" => {
                    config.pad_token_id = token
                        .parse()
                        .map_err(|e| format!("Invalid pad_token_id '{token}': {e}"))?;
                }
                "use_cache" => {
                    config.use_cache = token
                        .parse()
                        .map_err(|e| format!("Invalid use_cache '{token}': {e}"))?;
                }
                _ => {}
            }
        }
    }

    Ok(config)
}

/// Inner trait abstracting low-level Candle forward execution for vector embeddings.
///
/// Enables mock-based unit testing without loading full ONNX or GGUF files in CI.
pub trait CandleEmbedInner: Send {
    /// Generates an embedding vector for the provided input text.
    fn embed(
        &mut self,
        text: &str,
        tokenizer: &tokenizers::Tokenizer,
        device: &Device,
    ) -> Result<Vec<f32>>;

    /// Returns the vector dimension produced by this embedding model.
    fn dim(&self) -> usize;
}

/// Text embedding client powered by Candle ML backend.
pub struct CandleEmbedClient {
    /// Hardware device (CPU, CUDA, Metal).
    pub device: Device,
    /// Thread-safe mutex wrapping the inner embedding model.
    pub model: Arc<tokio::sync::Mutex<Box<dyn CandleEmbedInner + Send>>>,
    /// Unique fingerprint identifying the embedding model weights and quantization.
    pub fingerprint: ModelFingerprint,
    /// Tokenizer for converting text to token ID tensors.
    pub tokenizer: tokenizers::Tokenizer,
    /// Vector dimension produced by this model.
    pub dim: usize,
    /// Maximum concurrent embedding operations permitted.
    pub max_concurrent_embeddings: usize,
    /// Semaphore enforcing backpressure on concurrent embedding calls.
    pub semaphore: Arc<tokio::sync::Semaphore>,
}

impl CandleEmbedClient {
    /// Creates a new `CandleEmbedClient`.
    pub fn new(
        device: Device,
        model: Box<dyn CandleEmbedInner + Send>,
        fingerprint: ModelFingerprint,
        tokenizer: tokenizers::Tokenizer,
    ) -> Self {
        let dim = model.dim();
        let max_concurrent_embeddings = DEFAULT_MAX_CONCURRENT_EMBEDDINGS;
        let client = Self {
            device,
            model: Arc::new(tokio::sync::Mutex::new(model)),
            fingerprint,
            tokenizer,
            dim,
            max_concurrent_embeddings,
            semaphore: Arc::new(tokio::sync::Semaphore::new(max_concurrent_embeddings)),
        };
        client.with_max_concurrent_embeddings(DEFAULT_MAX_CONCURRENT_EMBEDDINGS)
    }

    /// Erstellt einen Test-Stub-Embedder für Unittests. Nicht für Produktion.
    #[doc(hidden)]
    pub fn with_test_stub_model(dim: usize) -> Self {
        let mock_model = Box::new(DefaultCandleEmbedModel { dim });
        let fingerprint = crate::model_registry::ModelFingerprint {
            hash: [0u8; 32],
            model_id: "test-stub".to_string(),
            quantization: "none".to_string(),
        };
        let tokenizer_bytes = r#"{
            "version": "1.0",
            "truncation": null,
            "padding": null,
            "added_tokens": [],
            "normalizer": null,
            "pre_tokenizer": null,
            "post_processor": null,
            "decoder": null,
            "model": { "type": "BPE", "dropout": null, "unk_token": null, "continuing_subword_prefix": null, "end_of_word_suffix": null, "fuse_unk": false, "vocab": {}, "merges": [] }
        }"#;
        let tokenizer = tokenizers::Tokenizer::from_bytes(tokenizer_bytes.as_bytes())
            .unwrap_or_else(|_| tokenizers::Tokenizer::new(tokenizers::models::bpe::BPE::default()));
        Self::new(Device::Cpu, mock_model, fingerprint, tokenizer)
    }

    /// Configures maximum concurrent embedding operations for backpressure control.
    pub fn with_max_concurrent_embeddings(mut self, limit: usize) -> Self {
        let limit = limit.max(1);
        self.max_concurrent_embeddings = limit;
        self.semaphore = Arc::new(tokio::sync::Semaphore::new(limit));
        self
    }

    /// Returns a reference to the model's fingerprint.
    pub fn fingerprint(&self) -> &ModelFingerprint {
        &self.fingerprint
    }

    /// Loads a `CandleEmbedClient` from a model directory.
    pub fn from_dir(
        model_dir: &std::path::Path,
        quantization: crate::model_registry::CandleQuantization,
    ) -> Result<Self> {
        if !model_dir.exists() {
            return Err(ContextraError::InvalidInput(format!(
                "Candle model directory does not exist: {}",
                model_dir.display()
            )));
        }

        let weights_path = model_dir.join("model.safetensors");
        if !weights_path.exists() {
            return Err(ContextraError::InvalidInput(format!(
                "Missing model weights file model.safetensors in {}",
                model_dir.display()
            )));
        }

        let tokenizer_path = model_dir.join("tokenizer.json");
        let tokenizer = if tokenizer_path.exists() {
            tokenizers::Tokenizer::from_file(&tokenizer_path).map_err(|e| {
                ContextraError::InvalidInput(format!(
                    "Failed to load tokenizer from {}: {e}",
                    tokenizer_path.display()
                ))
            })?
        } else {
            let tokenizer_bytes = r#"{
                "version": "1.0",
                "truncation": null,
                "padding": null,
                "added_tokens": [],
                "normalizer": null,
                "pre_tokenizer": null,
                "post_processor": null,
                "decoder": null,
                "model": { "type": "BPE", "dropout": null, "unk_token": null, "continuing_subword_prefix": null, "end_of_word_suffix": null, "fuse_unk": false, "vocab": {}, "merges": [] }
            }"#;
            tokenizers::Tokenizer::from_bytes(tokenizer_bytes.as_bytes()).map_err(|e| {
                ContextraError::Internal(format!("Failed to parse default tokenizer: {e}"))
            })?
        };

        let gguf_path = model_dir.join("model.gguf");
        let fingerprint = if gguf_path.exists() {
            let meta = crate::gguf_loader::parse_gguf_metadata(&gguf_path)?;
            tracing::debug!(
                "Parsed GGUF metadata for embedding model: arch={}, tensors={}",
                meta.architecture,
                meta.tensor_count
            );
            crate::model_registry::compute_fingerprint(&gguf_path, &quantization)?
        // STARTUP-ONLY: kein Hot-Path, spawn_blocking nicht erforderlich
        } else if let Ok(entries) = std::fs::read_dir(model_dir) {
            let mut gguf_found = None;
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("gguf") {
                    gguf_found = Some(path);
                    break;
                }
            }
            if let Some(path) = gguf_found {
                let meta = crate::gguf_loader::parse_gguf_metadata(&path)?;
                tracing::debug!(
                    "Parsed GGUF metadata for embedding model: arch={}, tensors={}",
                    meta.architecture,
                    meta.tensor_count
                );
                crate::model_registry::compute_fingerprint(&path, &quantization)?
            } else {
                crate::model_registry::ModelFingerprint {
                    hash: [0u8; 32],
                    model_id: model_dir
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                        .to_string(),
                    quantization: quantization.to_string(),
                }
            }
        } else {
            crate::model_registry::ModelFingerprint {
                hash: [0u8; 32],
                model_id: model_dir
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string(),
                quantization: quantization.to_string(),
            }
        };

        let device = Device::Cpu;
        let config_path = model_dir.join("config.json");

        let model: Box<dyn CandleEmbedInner + Send> =
            Box::new(BertEmbedModel::load(&weights_path, &config_path, &device)?);

        Ok(Self::new(device, model, fingerprint, tokenizer))
    }

    /// Loads a `CandleEmbedClient` from a model directory with custom maximum concurrent operations limit.
    pub fn from_dir_with_concurrency(
        model_dir: &std::path::Path,
        quantization: crate::model_registry::CandleQuantization,
        max_concurrent: usize,
    ) -> Result<Self> {
        let client = Self::from_dir(model_dir, quantization)?;
        Ok(client.with_max_concurrent_embeddings(max_concurrent))
    }
}

/// Real BERT transformer text embedding model wrapper.
pub struct BertEmbedModel {
    model: BertModel,
    dim: usize,
}

impl BertEmbedModel {
    /// Loads BERT model weights from a `.safetensors` file and configuration from `config.json`.
    pub fn load(weights_path: &Path, config_path: &Path, device: &Device) -> Result<Self> {
        let config = load_bert_config(config_path)?;

        // STARTUP-ONLY: kein Hot-Path, spawn_blocking nicht erforderlich
        let weights_bytes = std::fs::read(weights_path).map_err(|e| {
            ContextraError::Io(std::io::Error::new(
                e.kind(),
                format!(
                    "Failed to read BERT safetensors weights file {}: {e}",
                    weights_path.display()
                ),
            ))
        })?;
        let vb =
            VarBuilder::from_buffered_safetensors(weights_bytes, DTYPE, device).map_err(|e| {
                ContextraError::Internal(format!(
                    "Failed to load BERT safetensors weights from {}: {e}",
                    weights_path.display()
                ))
            })?;

        let dim = config.hidden_size;
        let model = BertModel::load(vb, &config).map_err(|e| {
            ContextraError::Internal(format!("Failed to initialize BERT model: {e}"))
        })?;

        Ok(Self { model, dim })
    }
}

impl CandleEmbedInner for BertEmbedModel {
    fn embed(
        &mut self,
        text: &str,
        tokenizer: &tokenizers::Tokenizer,
        device: &Device,
    ) -> Result<Vec<f32>> {
        let encoding = tokenizer
            .encode(text, true)
            .map_err(|e| ContextraError::InvalidInput(format!("Tokenizer encoding error: {e}")))?;

        let tokens = encoding.get_ids();
        if tokens.is_empty() {
            return Err(ContextraError::InvalidInput(
                "Cannot embed empty token sequence".to_string(),
            ));
        }

        let token_ids = Tensor::new(tokens, device)
            .map_err(|e| {
                ContextraError::Internal(format!("Failed to create token_ids tensor: {e}"))
            })?
            .unsqueeze(0)
            .map_err(|e| ContextraError::Internal(format!("Failed to unsqueeze token_ids: {e}")))?;

        let token_type_ids = token_ids.zeros_like().map_err(|e| {
            ContextraError::Internal(format!("Failed to create token_type_ids: {e}"))
        })?;

        // Forward pass through BERT model
        let embeddings = self
            .model
            .forward(&token_ids, &token_type_ids, None)
            .map_err(|e| ContextraError::Internal(format!("BERT forward pass error: {e}")))?;

        // Mean pooling over token sequence dimension (dim 1)
        let (_b_sz, seq_len, _hidden_dim) = embeddings
            .dims3()
            .map_err(|e| ContextraError::Internal(format!("Expected 3D embeddings tensor: {e}")))?;

        let sum_embeddings = embeddings
            .sum(1)
            .map_err(|e| ContextraError::Internal(format!("Failed to sum embeddings: {e}")))?;
        let pooled = (sum_embeddings / (seq_len as f64))
            .map_err(|e| ContextraError::Internal(format!("Failed to mean pool embeddings: {e}")))?
            .squeeze(0)
            .map_err(|e| {
                ContextraError::Internal(format!("Failed to squeeze pooled tensor: {e}"))
            })?;

        let vec: Vec<f32> = pooled.to_vec1().map_err(|e| {
            ContextraError::Internal(format!("Failed to convert tensor to vec: {e}"))
        })?;

        // L2 normalization and zero/NaN check (APM-4)
        let norm_sq: f32 = vec.iter().map(|v| v * v).sum();
        let norm = norm_sq.sqrt();

        if norm < 1e-12 || norm.is_nan() || !norm.is_finite() {
            return Err(ContextraError::Internal(format!(
                "Invalid or zero L2 vector norm ({norm}) produced during embedding forward pass"
            )));
        }

        let normalized_vec = vec.into_iter().map(|v| v / norm).collect();
        Ok(normalized_vec)
    }

    fn dim(&self) -> usize {
        self.dim
    }
}

/// Default inner Candle embedding mock model for unit testing when weight files are missing.
struct DefaultCandleEmbedModel {
    /// Vector dimension.
    dim: usize,
}

impl CandleEmbedInner for DefaultCandleEmbedModel {
    fn embed(
        &mut self,
        text: &str,
        _tokenizer: &tokenizers::Tokenizer,
        _device: &Device,
    ) -> Result<Vec<f32>> {
        // Deterministic pseudo-embedding generator derived from input text string hash
        // Ensures distinct non-zero L2-normalized vectors for distinct input texts in mock/test mode
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        let seed = hasher.finish();

        let mut raw_vec = Vec::with_capacity(self.dim);
        for i in 0..self.dim {
            let val =
                ((seed.wrapping_add((i as u64).wrapping_mul(2654435761))) % 1000) as f32 + 1.0;
            raw_vec.push(val);
        }

        let norm_sq: f32 = raw_vec.iter().map(|v| v * v).sum();
        let norm = norm_sq.sqrt();
        if norm < 1e-12 {
            return Err(ContextraError::Internal(
                "Zero norm in mock embedder".to_string(),
            ));
        }

        Ok(raw_vec.into_iter().map(|v| v / norm).collect())
    }

    fn dim(&self) -> usize {
        self.dim
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use contextra_ports::{EmbeddingProvider, TextEmbeddingEngine};

    struct MockEmbedModel {
        dim: usize,
    }

    impl CandleEmbedInner for MockEmbedModel {
        fn embed(
            &mut self,
            _text: &str,
            _tokenizer: &tokenizers::Tokenizer,
            _device: &Device,
        ) -> Result<Vec<f32>> {
            Ok(vec![0.5f32; self.dim])
        }

        fn dim(&self) -> usize {
            self.dim
        }
    }

    #[tokio::test]
    async fn test_candle_embed_client_provider_and_engine() {
        let mock_model = Box::new(MockEmbedModel { dim: 4 });
        let fingerprint = ModelFingerprint {
            hash: [2u8; 32],
            model_id: "embed_model.gguf".to_string(),
            quantization: "Q8_0".to_string(),
        };
        let tokenizer_bytes = r#"{
            "version": "1.0",
            "truncation": null,
            "padding": null,
            "added_tokens": [],
            "normalizer": null,
            "pre_tokenizer": null,
            "post_processor": null,
            "decoder": null,
            "model": { "type": "BPE", "dropout": null, "unk_token": null, "continuing_subword_prefix": null, "end_of_word_suffix": null, "fuse_unk": false, "vocab": {}, "merges": [] }
        }"#;
        let tokenizer = tokenizers::Tokenizer::from_bytes(tokenizer_bytes.as_bytes())
            .map_err(|e| e.to_string())
            .unwrap();

        let client =
            CandleEmbedClient::new(Device::Cpu, mock_model, fingerprint.clone(), tokenizer);

        assert_eq!(client.provider_name(), "candle");
        assert_eq!(client.embedding_dim(), 4);
        assert_eq!(client.fingerprint(), &fingerprint);

        let vec = EmbeddingProvider::embed(&client, "test sentence")
            .await
            .unwrap();
        assert_eq!(vec, vec![0.5f32, 0.5f32, 0.5f32, 0.5f32]);

        // Test blanket TextEmbeddingEngine
        let engine: &dyn TextEmbeddingEngine = &client;
        let vec_engine = engine.embed("another test").await.unwrap();
        assert_eq!(vec_engine, vec![0.5f32; 4]);
    }

    #[test]
    fn test_from_dir_nonexistent() {
        let res = CandleEmbedClient::from_dir(
            std::path::Path::new("/nonexistent/model/dir"),
            crate::model_registry::CandleQuantization::Q4KM,
        );
        assert!(res.is_err());
    }

    #[test]
    fn test_from_dir_missing_safetensors() {
        let temp_dir = tempfile::tempdir().unwrap();
        let res = CandleEmbedClient::from_dir(
            temp_dir.path(),
            crate::model_registry::CandleQuantization::Q4KM,
        );
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_embed_zero_concurrency_limit_clamped_to_one() {
        let mock_model = Box::new(MockEmbedModel { dim: 8 });
        let fingerprint = ModelFingerprint {
            hash: [3u8; 32],
            model_id: "embed_clamped.gguf".to_string(),
            quantization: "Q8_0".to_string(),
        };
        let tokenizer_bytes = r#"{
            "version": "1.0",
            "truncation": null,
            "padding": null,
            "added_tokens": [],
            "normalizer": null,
            "pre_tokenizer": null,
            "post_processor": null,
            "decoder": null,
            "model": { "type": "BPE", "dropout": null, "unk_token": null, "continuing_subword_prefix": null, "end_of_word_suffix": null, "fuse_unk": false, "vocab": {}, "merges": [] }
        }"#;
        let tokenizer = tokenizers::Tokenizer::from_bytes(tokenizer_bytes.as_bytes()).unwrap();

        let client = CandleEmbedClient::new(Device::Cpu, mock_model, fingerprint, tokenizer)
            .with_max_concurrent_embeddings(0);

        assert_eq!(client.max_concurrent_embeddings, 1);
        assert_eq!(client.semaphore.available_permits(), 1);
    }

    #[test]
    fn test_bert_embed_model_load_nonexistent_weights_returns_error() {
        let path = Path::new("/nonexistent/model.safetensors");
        let cfg_path = Path::new("/nonexistent/config.json");
        let device = Device::Cpu;

        let res = BertEmbedModel::load(path, cfg_path, &device);
        assert!(res.is_err());
    }
}
