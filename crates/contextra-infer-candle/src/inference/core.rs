// FILE-CONTEXT
// STAND: 2026-09-12T00:00:00Z (SESSION: BACKPRESSURE-CONTRACT-D1)
// ZWECK: Candle LLM text generator client implementing LlmTextGenerator.
// INVARIANTEN: Thread-safe model access via Mutex; spawn_blocking for CPU inference execution.
// Backpressure contract: max_concurrent_inferences limits spawn_blocking calls.
// Callers will experience backpressure (await on permit acquire) rather than Tokio thread pool exhaustion.

//! contextra-candle LLM inference module.
//!
//! Backpressure contract: `max_concurrent_inferences` limits `spawn_blocking` calls.
//! Callers will experience backpressure (await on permit acquire) rather than Tokio thread pool exhaustion.

use crate::gasp::GaspValidator;
#[cfg(feature = "kv-stage-b")]
use crate::model::quantized_llama::ModelWeights;
use crate::model_registry::ModelFingerprint;
use candle_core::quantized::gguf_file;
use candle_core::Device;
use candle_transformers::generation::LogitsProcessor;
#[cfg(not(feature = "kv-stage-b"))]
use candle_transformers::models::quantized_llama::ModelWeights;
use crate::error::{Error, Result};
use crate::calibrator::ConfigFingerprint;
use std::fs::File;
use std::path::Path;
use std::sync::Arc;

/// Inner trait abstracting low-level Candle forward/text-generation execution.
///
/// This trait allows dependency injection for unit testing with mock models
/// without requiring full GGUF binary weights in CI environments.
pub trait CandleModelInner: Send {
    /// Generates text completion for a given prompt using tokenizer and device settings.
    fn generate(
        &mut self,
        prompt: &str,
        tokenizer: &tokenizers::Tokenizer,
        device: &Device,
    ) -> Result<String>;

    /// Generates text stream for a given prompt, invoking `on_token` for each generated token or chunk.
    /// Returns early if `on_token` returns `false`.
    fn generate_stream(
        &mut self,
        prompt: &str,
        tokenizer: &tokenizers::Tokenizer,
        device: &Device,
        on_token: &mut dyn FnMut(String) -> bool,
    ) -> Result<String> {
        let text = self.generate(prompt, tokenizer, device)?;
        on_token(text.clone());
        Ok(text)
    }

    /// Returns the KV layout and RoPE configuration of the underlying model, if supported.
    #[cfg(feature = "kv-stage-b")]
    fn kv_layout(
        &self,
    ) -> Option<(
        crate::kv_state::KvLayout,
        crate::kv_state::RopeConfig,
    )> {
        None
    }

    /// Generates text stream using an optional KV prefix seed state.
    #[cfg(feature = "kv-stage-b")]
    fn generate_stream_with_prefix(
        &mut self,
        prompt: &str,
        tokenizer: &tokenizers::Tokenizer,
        device: &Device,
        seed: Option<crate::inference::PrefixSeed>,
        on_token: &mut dyn FnMut(String) -> bool,
    ) -> Result<crate::inference::PrefixRun> {
        let _ = seed;
        let tokens = tokenizer
            .encode(prompt, true)
            .map_err(|e| Error::InvalidInput(format!("Failed to tokenize prompt: {e}")))?;
        let prompt_tokens = tokens.get_ids().to_vec();
        let output = self.generate_stream(prompt, tokenizer, device, on_token)?;
        Ok(crate::inference::PrefixRun {
            output,
            prompt_tokens,
            reused_tokens: 0,
            exported_prefix: None,
        })
    }
}

/// Default maximum concurrent inference operations for Candle LLM text generation.
pub const DEFAULT_MAX_CONCURRENT_INFERENCES: usize = 4;

/// LLM Text Generator implementation powered by Candle inference engine.
pub struct CandleLlmClient {
    /// Target hardware device (CPU, CUDA, Metal).
    pub device: Device,
    /// Thread-safe mutex wrapping model execution state.
    pub model: Arc<tokio::sync::Mutex<Box<dyn CandleModelInner + Send>>>,
    /// Unique fingerprint identifying model weights and quantization level.
    pub fingerprint: ModelFingerprint,
    /// HuggingFace Tokenizer instance.
    pub tokenizer: tokenizers::Tokenizer,
    /// Maximum concurrent inference operations permitted.
    pub max_concurrent_inferences: usize,
    /// Semaphore enforcing backpressure on concurrent inference calls.
    pub semaphore: Arc<tokio::sync::Semaphore>,
    /// Optional KV-Bridge Adapter connecting retrieval KV cache segments to Candle inference.
    #[cfg(feature = "kv-bridge")]
    pub kv_bridge: Option<crate::KvBridgeAdapter>,
    /// Telemetry counter tracking segment full prefill calculations.
    pub prefill_count: Arc<std::sync::atomic::AtomicU64>,
    /// Telemetry counter tracking segment prefill skips via KV cache hits.
    pub prefill_skip_count: Arc<std::sync::atomic::AtomicU64>,
    /// Optional KV-Prefix Store Context for stage B prefix reuse.
    #[cfg(feature = "kv-stage-b")]
    pub prefix_store: Option<crate::inference::KvPrefixContext>,
    /// Telemetry counter tracking prefill skipped token count.
    #[cfg(feature = "kv-stage-b")]
    pub prefill_skipped_tokens: Arc<std::sync::atomic::AtomicU64>,
}

impl CandleLlmClient {
    /// Creates a new `CandleLlmClient` with default concurrency limits.
    pub fn new(
        device: Device,
        model: Box<dyn CandleModelInner + Send>,
        fingerprint: ModelFingerprint,
        tokenizer: tokenizers::Tokenizer,
    ) -> Self {
        let max_concurrent_inferences = DEFAULT_MAX_CONCURRENT_INFERENCES;
        Self {
            device,
            model: Arc::new(tokio::sync::Mutex::new(model)),
            fingerprint,
            tokenizer,
            max_concurrent_inferences,
            semaphore: Arc::new(tokio::sync::Semaphore::new(max_concurrent_inferences)),
            #[cfg(feature = "kv-bridge")]
            kv_bridge: None,
            prefill_count: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            prefill_skip_count: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            #[cfg(feature = "kv-stage-b")]
            prefix_store: None,
            #[cfg(feature = "kv-stage-b")]
            prefill_skipped_tokens: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }

    /// Returns the total count of segment full prefill calculations performed.
    pub fn prefill_count(&self) -> u64 {
        self.prefill_count.load(std::sync::atomic::Ordering::SeqCst)
    }

    /// Returns the total count of segment prefill skips via KV cache hits.
    pub fn prefill_skip_count(&self) -> u64 {
        self.prefill_skip_count
            .load(std::sync::atomic::Ordering::SeqCst)
    }

    /// Returns the total count of prompt tokens skipped via KV prefix reuse.
    #[cfg(feature = "kv-stage-b")]
    pub fn prefill_skipped_tokens(&self) -> u64 {
        self.prefill_skipped_tokens
            .load(std::sync::atomic::Ordering::SeqCst)
    }

    /// Configures maximum concurrent inference operations for backpressure control.
    pub fn with_max_concurrent_inferences(mut self, limit: usize) -> Self {
        let limit = limit.max(1);
        self.max_concurrent_inferences = limit;
        self.semaphore = Arc::new(tokio::sync::Semaphore::new(limit));
        self
    }

    /// Configures optional `KvBridgeAdapter` for KV cache segment retrieval and injection.
    #[cfg(feature = "kv-bridge")]
    pub fn with_kv_bridge(mut self, adapter: crate::KvBridgeAdapter) -> Self {
        self.kv_bridge = Some(adapter);
        self
    }

    /// Returns a reference to the model's fingerprint.
    pub fn fingerprint(&self) -> &ModelFingerprint {
        &self.fingerprint
    }

    /// Tauscht das zugrundeliegende Modell und dessen Fingerprint zur Laufzeit aus.
    /// Falls ein optionaler `GaspValidator` übergeben wird, wird dessen Kalibrierungsstatus
    /// mit dem neuen Fingerprint invalidiert/aktualisiert (INV-CAL-2).
    pub fn swap_model(
        &mut self,
        new_model: Box<dyn CandleModelInner + Send>,
        new_fingerprint: ModelFingerprint,
        validator: Option<&mut GaspValidator>,
    ) {
        if let Some(mutex) = Arc::get_mut(&mut self.model) {
            *mutex.get_mut() = new_model;
        } else {
            self.model = Arc::new(tokio::sync::Mutex::new(new_model));
        }

        if let Some(val) = validator {
            let mut new_config = val.config().clone();
            new_config.fingerprint = ConfigFingerprint::new(
                &new_fingerprint.model_id,
                &new_fingerprint.quantization,
                "gasp-attribution",
                0.0,
            )
            .with_threshold(new_config.threshold);
            val.refresh_config(new_config);
        }

        self.fingerprint = new_fingerprint;
    }

    /// Loads a `CandleLlmClient` from a model directory.
    pub fn from_dir(
        model_dir: &std::path::Path,
        quantization: crate::model_registry::CandleQuantization,
    ) -> Result<Self> {
        if !model_dir.exists() {
            return Err(Error::ModelLoad {
                reason: format!(
                    "Candle model directory does not exist: {}",
                    model_dir.display()
                ),
            });
        }

        let tokenizer_path = model_dir.join("tokenizer.json");
        if !tokenizer_path.exists() {
            return Err(Error::ModelLoad {
                reason: format!(
                    "Tokenizer file 'tokenizer.json' missing in model directory: {}",
                    model_dir.display()
                ),
            });
        }

        let tokenizer = tokenizers::Tokenizer::from_file(&tokenizer_path).map_err(|e| {
            Error::ModelLoad {
                reason: format!(
                    "Failed to load tokenizer from {}: {e}",
                    tokenizer_path.display()
                ),
            }
        })?;

        if tokenizer.get_vocab_size(true) == 0 {
            return Err(Error::ModelLoad {
                reason: "Tokenizer vocabulary is empty".to_string(),
            });
        }

        let default_gguf = model_dir.join("model.gguf");
        let gguf_path = if default_gguf.exists() {
            default_gguf
        } else {
            let entries = std::fs::read_dir(model_dir).map_err(|e| Error::ModelLoad {
                reason: format!(
                    "Failed to read model directory {}: {e}",
                    model_dir.display()
                ),
            })?;

            let mut gguf_found = None;
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("gguf") {
                    gguf_found = Some(path);
                    break;
                }
            }

            gguf_found.ok_or_else(|| Error::ModelLoad {
                reason: format!(
                    "No GGUF model file found in model directory: {}",
                    model_dir.display()
                ),
            })?
        };

        let fingerprint = crate::model_registry::compute_fingerprint(&gguf_path, &quantization)
            .map_err(|e| Error::ModelLoad {
                reason: format!(
                    "Failed to compute fingerprint for model {}: {e}",
                    gguf_path.display()
                ),
            })?;

        if fingerprint.hash == [0u8; 32] {
            return Err(Error::ModelLoad {
                reason: format!(
                    "Computed model fingerprint hash is all zeros for model {}",
                    gguf_path.display()
                ),
            });
        }

        let device = Device::Cpu;
        let model = QuantizedLlamaModel::load(&gguf_path, &device).map_err(|e| {
            Error::ModelLoad {
                reason: format!("Failed to load quantized GGUF Llama model weights: {e}"),
            }
        })?;

        Ok(Self::new(device, Box::new(model), fingerprint, tokenizer))
    }
}

/// Real quantized Llama / GGUF model execution wrapper.
pub struct QuantizedLlamaModel {
    weights: ModelWeights,
    sample_len: usize,
}

impl QuantizedLlamaModel {
    /// Loads GGUF quantized model weights from the specified file path.
    pub fn load(model_path: &Path, device: &Device) -> Result<Self> {
        // STARTUP-ONLY: kein Hot-Path, spawn_blocking nicht erforderlich
        let mut file = File::open(model_path).map_err(|e| {
            Error::Io(std::io::Error::new(
                e.kind(),
                format!(
                    "Failed to open GGUF weight file {}: {e}",
                    model_path.display()
                ),
            ))
        })?;

        let content = gguf_file::Content::read(&mut file).map_err(|e| {
            Error::Inference(format!(
                "Failed to parse GGUF content header for {}: {e}",
                model_path.display()
            ))
        })?;

        let weights = ModelWeights::from_gguf(content, &mut file, device).map_err(|e| {
            Error::Inference(format!(
                "Failed to build quantized Llama model weights from GGUF {}: {e}",
                model_path.display()
            ))
        })?;

        Ok(Self {
            weights,
            sample_len: 256,
        })
    }
}

impl CandleModelInner for QuantizedLlamaModel {
    #[cfg(feature = "kv-stage-b")]
    fn kv_layout(
        &self,
    ) -> Option<(
        crate::kv_state::KvLayout,
        crate::kv_state::RopeConfig,
    )> {
        if let Some(layer) = self.weights.layers.first() {
            let layout = crate::kv_state::KvLayout {
                n_layer: self.weights.layers.len() as u32,
                n_kv_head: layer.n_kv_head as u32,
                head_dim: layer.head_dim as u32,
                dtype: "f16".to_string(),
            };
            // RoPE base frequency used in precomput_freqs_cis in quantized_llama.rs
            // Hardcoded constant: freq_base = 10000.0
            let rope = crate::kv_state::RopeConfig {
                base: 10000.0,
                scaling: None,
            };
            Some((layout, rope))
        } else {
            None
        }
    }

    fn generate(
        &mut self,
        prompt: &str,
        tokenizer: &tokenizers::Tokenizer,
        device: &Device,
    ) -> Result<String> {
        let mut full_output = String::new();
        self.generate_stream(prompt, tokenizer, device, &mut |chunk| {
            full_output.push_str(&chunk);
            true
        })?;
        Ok(full_output)
    }

    fn generate_stream(
        &mut self,
        prompt: &str,
        tokenizer: &tokenizers::Tokenizer,
        device: &Device,
        on_token: &mut dyn FnMut(String) -> bool,
    ) -> Result<String> {
        #[cfg(feature = "kv-stage-b")]
        {
            let run =
                self.generate_stream_with_prefix(prompt, tokenizer, device, None, on_token)?;
            Ok(run.output)
        }
        #[cfg(not(feature = "kv-stage-b"))]
        {
            let tokens = tokenizer.encode(prompt, true).map_err(|e| {
                Error::InvalidInput(format!("Failed to tokenize prompt: {e}"))
            })?;
            let prompt_tokens = tokens.get_ids();
            if prompt_tokens.is_empty() {
                return Err(Error::InvalidInput(
                    "Encoded prompt tokens cannot be empty".to_string(),
                ));
            }

            let mut logits_processor = LogitsProcessor::new(299792458, Some(0.7), Some(0.9));
            let mut all_tokens = prompt_tokens.to_vec();
            let mut generated_tokens = Vec::new();
            let mut full_output = String::new();

            let mut index_pos = 0;
            for i in 0..self.sample_len {
                let context_len = if i == 0 { all_tokens.len() } else { 1 };
                let input_slice = if i == 0 {
                    all_tokens.clone()
                } else {
                    vec![*all_tokens.last().ok_or_else(|| {
                        Error::Inference(
                            "all_tokens cannot be empty during generation".into(),
                        )
                    })?]
                };

                let input_tensor = candle_core::Tensor::new(&input_slice[..], device)
                    .map_err(|e| {
                        Error::Inference(format!("Failed to create input tensor: {e}"))
                    })?
                    .unsqueeze(0)
                    .map_err(|e| {
                        Error::Inference(format!("Failed to unsqueeze tensor: {e}"))
                    })?;

                let logits = self
                    .weights
                    .forward(&input_tensor, index_pos)
                    .map_err(|e| {
                        Error::Inference(format!("Quantized Llama forward error: {e}"))
                    })?;

                let logits = logits.squeeze(0).map_err(|e| {
                    Error::Inference(format!("Failed to squeeze logits: {e}"))
                })?;
                let logits = logits
                    .get(
                        logits
                            .dim(0)
                            .map_err(|e| Error::Inference(e.to_string()))?
                            - 1,
                    )
                    .map_err(|e| {
                        Error::Inference(format!("Failed to slice logits: {e}"))
                    })?;

                let next_token = logits_processor.sample(&logits).map_err(|e| {
                    Error::Inference(format!("Logits sampling failed: {e}"))
                })?;

                all_tokens.push(next_token);
                generated_tokens.push(next_token);
                index_pos += context_len;

                if let Ok(piece) = tokenizer.decode(&[next_token], true) {
                    if !piece.is_empty() {
                        full_output.push_str(&piece);
                        if !on_token(piece) {
                            break;
                        }
                    }
                }

                if next_token == 2 || next_token == 128001 || next_token == 128009 {
                    break;
                }
            }

            Ok(full_output)
        }
    }

    #[cfg(feature = "kv-stage-b")]
    fn generate_stream_with_prefix(
        &mut self,
        prompt: &str,
        tokenizer: &tokenizers::Tokenizer,
        device: &Device,
        seed: Option<crate::inference::PrefixSeed>,
        on_token: &mut dyn FnMut(String) -> bool,
    ) -> Result<crate::inference::PrefixRun> {
        let tokens = tokenizer
            .encode(prompt, true)
            .map_err(|e| Error::InvalidInput(format!("Failed to tokenize prompt: {e}")))?;
        let prompt_tokens = tokens.get_ids().to_vec();
        if prompt_tokens.is_empty() {
            return Err(Error::InvalidInput(
                "Encoded prompt tokens cannot be empty".to_string(),
            ));
        }

        let mut logits_processor = LogitsProcessor::new(299792458, Some(0.7), Some(0.9));
        let mut all_tokens = prompt_tokens.clone();
        let mut generated_tokens = Vec::new();
        let mut full_output = String::new();

        let mut reused_tokens = 0;
        let mut index_pos = 0;
        let mut prefill_logits = None;

        if let Some(s) = seed {
            if s.matched_len > 0 && s.matched_len < prompt_tokens.len() {
                self.weights.clear_kv_cache();
                if self.weights.import_kv_state(&s.state).is_ok() {
                    let suffix = &prompt_tokens[s.matched_len..];
                    if let Ok(input_tensor) =
                        candle_core::Tensor::new(suffix, device).and_then(|t| t.unsqueeze(0))
                    {
                        if let Ok(logits) = self.weights.forward(&input_tensor, s.matched_len) {
                            prefill_logits = Some(logits);
                            reused_tokens = s.matched_len;
                            index_pos = prompt_tokens.len();
                        }
                    }
                }
            }
        }

        if prefill_logits.is_none() {
            self.weights.clear_kv_cache();
            reused_tokens = 0;
            let input_tensor = candle_core::Tensor::new(&prompt_tokens[..], device)
                .map_err(|e| {
                    Error::Inference(format!("Failed to create input tensor: {e}"))
                })?
                .unsqueeze(0)
                .map_err(|e| {
                    Error::Inference(format!("Failed to unsqueeze tensor: {e}"))
                })?;
            let logits = self.weights.forward(&input_tensor, 0).map_err(|e| {
                Error::Inference(format!("Quantized Llama forward error: {e}"))
            })?;
            prefill_logits = Some(logits);
            index_pos = prompt_tokens.len();
        }

        let exported_prefix = self
            .weights
            .export_kv_state()
            .and_then(|st| st.export_block(0..prompt_tokens.len()))
            .ok();

        let logits = prefill_logits
            .ok_or_else(|| Error::Inference("Missing prefill logits".into()))?;
        let logits = logits
            .squeeze(0)
            .map_err(|e| Error::Inference(format!("Failed to squeeze logits: {e}")))?;
        let logits = logits
            .get(
                logits
                    .dim(0)
                    .map_err(|e| Error::Inference(e.to_string()))?
                    - 1,
            )
            .map_err(|e| Error::Inference(format!("Failed to slice logits: {e}")))?;

        let next_token = logits_processor
            .sample(&logits)
            .map_err(|e| Error::Inference(format!("Logits sampling failed: {e}")))?;

        all_tokens.push(next_token);
        generated_tokens.push(next_token);

        if let Ok(piece) = tokenizer.decode(&[next_token], true) {
            if !piece.is_empty() {
                full_output.push_str(&piece);
                if !on_token(piece) {
                    return Ok(crate::inference::PrefixRun {
                        output: full_output,
                        prompt_tokens,
                        reused_tokens,
                        exported_prefix,
                    });
                }
            }
        }

        if next_token != 2 && next_token != 128001 && next_token != 128009 {
            for _i in 1..self.sample_len {
                let last_token = *all_tokens.last().ok_or_else(|| {
                    Error::Inference("all_tokens cannot be empty during generation".into())
                })?;
                let input_tensor = candle_core::Tensor::new(&[last_token], device)
                    .map_err(|e| {
                        Error::Inference(format!("Failed to create input tensor: {e}"))
                    })?
                    .unsqueeze(0)
                    .map_err(|e| {
                        Error::Inference(format!("Failed to unsqueeze tensor: {e}"))
                    })?;

                let logits = self
                    .weights
                    .forward(&input_tensor, index_pos)
                    .map_err(|e| {
                        Error::Inference(format!("Quantized Llama forward error: {e}"))
                    })?;

                let logits = logits.squeeze(0).map_err(|e| {
                    Error::Inference(format!("Failed to squeeze logits: {e}"))
                })?;
                let logits = logits
                    .get(
                        logits
                            .dim(0)
                            .map_err(|e| Error::Inference(e.to_string()))?
                            - 1,
                    )
                    .map_err(|e| {
                        Error::Inference(format!("Failed to slice logits: {e}"))
                    })?;

                let next_token = logits_processor.sample(&logits).map_err(|e| {
                    Error::Inference(format!("Logits sampling failed: {e}"))
                })?;

                all_tokens.push(next_token);
                generated_tokens.push(next_token);
                index_pos += 1;

                if let Ok(piece) = tokenizer.decode(&[next_token], true) {
                    if !piece.is_empty() {
                        full_output.push_str(&piece);
                        if !on_token(piece) {
                            break;
                        }
                    }
                }

                if next_token == 2 || next_token == 128001 || next_token == 128009 {
                    break;
                }
            }
        }

        Ok(crate::inference::PrefixRun {
            output: full_output,
            prompt_tokens,
            reused_tokens,
            exported_prefix,
        })
    }
}

/// Default inner Candle LLM mock model for unit tests when binary weights are absent.
#[cfg(any(test, feature = "test-utils"))]
pub struct DefaultCandleLlmModel;

#[cfg(any(test, feature = "test-utils"))]
impl CandleModelInner for DefaultCandleLlmModel {
    fn generate(
        &mut self,
        prompt: &str,
        _tokenizer: &tokenizers::Tokenizer,
        _device: &Device,
    ) -> Result<String> {
        Ok(format!("[MockCandle] Response for prompt: {prompt}"))
    }

    fn generate_stream(
        &mut self,
        prompt: &str,
        _tokenizer: &tokenizers::Tokenizer,
        _device: &Device,
        on_token: &mut dyn FnMut(String) -> bool,
    ) -> Result<String> {
        let text = format!("[MockCandle] Response for prompt: {prompt}");
        let words: Vec<&str> = text.split_whitespace().collect();
        for (i, word) in words.iter().enumerate() {
            let chunk = if i == 0 {
                word.to_string()
            } else {
                format!(" {word}")
            };
            if !on_token(chunk) {
                break;
            }
        }
        Ok(text)
    }
}

