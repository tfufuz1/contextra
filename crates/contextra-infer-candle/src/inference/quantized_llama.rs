// FILE-CONTEXT
// STAND: 2026-09-12T00:00:00Z
// ZWECK: Quantized Llama GGUF model execution wrapper.

use crate::inference::config::GenerationConfig;
use crate::inference::traits::CandleModelInner;
#[cfg(feature = "kv-stage-b")]
use crate::model::quantized_llama::ModelWeights;
use candle_core::quantized::gguf_file;
use candle_core::Device;
use candle_transformers::generation::LogitsProcessor;
#[cfg(not(feature = "kv-stage-b"))]
use candle_transformers::models::quantized_llama::ModelWeights;
use contextra_types::{ContextraError, Result};
use std::fs::File;
use std::path::Path;

/// Real quantized Llama / GGUF model execution wrapper.
pub struct QuantizedLlamaModel {
    weights: ModelWeights,
    config: GenerationConfig,
}

impl QuantizedLlamaModel {
    /// Loads GGUF quantized model weights from the specified file path with default generation config.
    pub fn load(model_path: &Path, device: &Device) -> Result<Self> {
        Self::load_with_config(model_path, device, GenerationConfig::default())
    }

    /// Loads GGUF quantized model weights from file path with custom `GenerationConfig`.
    pub fn load_with_config(
        model_path: &Path,
        device: &Device,
        config: GenerationConfig,
    ) -> Result<Self> {
        let mut file = File::open(model_path).map_err(|e| {
            ContextraError::Io(std::io::Error::new(
                e.kind(),
                format!(
                    "Failed to open GGUF weight file {}: {e}",
                    model_path.display()
                ),
            ))
        })?;

        let content = gguf_file::Content::read(&mut file).map_err(|e| ContextraError::ModelLoad {
            path: model_path.display().to_string(),
            reason: format!("Failed to parse GGUF content header: {e}"),
        })?;

        let weights = ModelWeights::from_gguf(content, &mut file, device).map_err(|e| {
            ContextraError::ModelLoad {
                path: model_path.display().to_string(),
                reason: format!("Failed to build quantized Llama model weights: {e}"),
            }
        })?;

        Ok(Self { weights, config })
    }

    /// Configures generation config for sampling parameters and context bounds.
    pub fn with_generation_config(mut self, config: GenerationConfig) -> Self {
        self.config = config;
        self
    }

    /// Truncates the underlying layer-wise KV cache tensors to `pos` tokens.
    /// Used when context window overflow occurs or prefix reuse requires resetting tail cache.
    #[cfg(feature = "kv-stage-b")]
    pub fn truncate_kv_cache(&mut self, pos: usize) -> Result<()> {
        self.weights.truncate_kv_cache(pos)
    }

    /// Clears the underlying layer-wise KV cache tensors completely.
    #[cfg(feature = "kv-stage-b")]
    pub fn clear_kv_cache(&mut self) {
        self.weights.clear_kv_cache();
    }
}

impl CandleModelInner for QuantizedLlamaModel {
    #[cfg(feature = "kv-stage-b")]
    fn kv_layout(
        &self,
    ) -> Option<(
        contextra_ports::kv::KvLayout,
        contextra_ports::kv::RopeConfig,
    )> {
        if let Some(layer) = self.weights.layers.first() {
            let layout = contextra_ports::kv::KvLayout {
                n_layer: self.weights.layers.len() as u32,
                n_kv_head: layer.n_kv_head as u32,
                head_dim: layer.head_dim as u32,
                dtype: "f16".to_string(),
            };
            let rope = contextra_ports::kv::RopeConfig {
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
                ContextraError::InvalidInput(format!("Failed to tokenize prompt: {e}"))
            })?;
            let prompt_tokens = tokens.get_ids();
            if prompt_tokens.is_empty() {
                return Err(ContextraError::InvalidInput(
                    "Encoded prompt tokens cannot be empty".to_string(),
                ));
            }

            self.config.validate_prompt_len(prompt_tokens.len())?;

            let temp = if self.config.temperature <= 0.0 {
                None
            } else {
                Some(self.config.temperature)
            };
            let mut logits_processor =
                LogitsProcessor::new(self.config.seed, temp, self.config.top_p);
            let mut all_tokens = prompt_tokens.to_vec();
            let mut full_output = String::new();

            let sample_len = self.config.max_token_budget;
            let mut index_pos = 0;
            for i in 0..sample_len {
                let context_len = if i == 0 { all_tokens.len() } else { 1 };
                let input_slice = if i == 0 {
                    all_tokens.clone()
                } else {
                    vec![*all_tokens
                        .last()
                        .ok_or_else(|| ContextraError::Internal("all_tokens cannot be empty".into()))?]
                };

                let input_tensor = candle_core::Tensor::new(&input_slice[..], device)
                    .map_err(|e| ContextraError::Internal(format!("Tensor creation err: {e}")))?
                    .unsqueeze(0)
                    .map_err(|e| ContextraError::Internal(format!("Unsqueeze err: {e}")))?;

                let logits = self
                    .weights
                    .forward(&input_tensor, index_pos)
                    .map_err(|e| ContextraError::Internal(format!("Forward err: {e}")))?;

                let logits = logits
                    .squeeze(0)
                    .map_err(|e| ContextraError::Internal(e.to_string()))?;
                let logits = logits
                    .get(
                        logits
                            .dim(0)
                            .map_err(|e| ContextraError::Internal(e.to_string()))?
                            - 1,
                    )
                    .map_err(|e| ContextraError::Internal(e.to_string()))?;

                let next_token = logits_processor
                    .sample(&logits)
                    .map_err(|e| ContextraError::Internal(format!("Sampling failed: {e}")))?;

                all_tokens.push(next_token);
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
            .map_err(|e| ContextraError::InvalidInput(format!("Failed to tokenize prompt: {e}")))?;
        let prompt_tokens = tokens.get_ids().to_vec();
        if prompt_tokens.is_empty() {
            return Err(ContextraError::InvalidInput(
                "Encoded prompt tokens cannot be empty".to_string(),
            ));
        }

        self.config.validate_prompt_len(prompt_tokens.len())?;

        let temp = if self.config.temperature <= 0.0 {
            None
        } else {
            Some(self.config.temperature)
        };
        let mut logits_processor = LogitsProcessor::new(self.config.seed, temp, self.config.top_p);
        let mut all_tokens = prompt_tokens.clone();
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
                .map_err(|e| ContextraError::Internal(format!("Tensor creation err: {e}")))?
                .unsqueeze(0)
                .map_err(|e| ContextraError::Internal(format!("Unsqueeze err: {e}")))?;
            let logits = self
                .weights
                .forward(&input_tensor, 0)
                .map_err(|e| ContextraError::Internal(format!("Forward err: {e}")))?;
            prefill_logits = Some(logits);
            index_pos = prompt_tokens.len();
        }

        let exported_prefix = self
            .weights
            .export_kv_state()
            .and_then(|st| st.export_block(0..prompt_tokens.len()))
            .ok();

        let logits = prefill_logits
            .ok_or_else(|| ContextraError::Internal("Missing prefill logits".into()))?;
        let logits = logits
            .squeeze(0)
            .map_err(|e| ContextraError::Internal(e.to_string()))?;
        let logits = logits
            .get(
                logits
                    .dim(0)
                    .map_err(|e| ContextraError::Internal(e.to_string()))?
                    - 1,
            )
            .map_err(|e| ContextraError::Internal(e.to_string()))?;

        let next_token = logits_processor
            .sample(&logits)
            .map_err(|e| ContextraError::Internal(format!("Sampling failed: {e}")))?;

        all_tokens.push(next_token);

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

        let sample_len = self.config.max_token_budget;
        if next_token != 2 && next_token != 128001 && next_token != 128009 {
            for _i in 1..sample_len {
                let last_token = *all_tokens
                    .last()
                    .ok_or_else(|| ContextraError::Internal("all_tokens cannot be empty".into()))?;
                let input_tensor = candle_core::Tensor::new(&[last_token], device)
                    .map_err(|e| ContextraError::Internal(format!("Tensor creation err: {e}")))?
                    .unsqueeze(0)
                    .map_err(|e| ContextraError::Internal(format!("Unsqueeze err: {e}")))?;

                let logits = self
                    .weights
                    .forward(&input_tensor, index_pos)
                    .map_err(|e| ContextraError::Internal(format!("Forward err: {e}")))?;

                let logits = logits
                    .squeeze(0)
                    .map_err(|e| ContextraError::Internal(e.to_string()))?;
                let logits = logits
                    .get(
                        logits
                            .dim(0)
                            .map_err(|e| ContextraError::Internal(e.to_string()))?
                            - 1,
                    )
                    .map_err(|e| ContextraError::Internal(e.to_string()))?;

                let next_token = logits_processor
                    .sample(&logits)
                    .map_err(|e| ContextraError::Internal(format!("Sampling failed: {e}")))?;

                all_tokens.push(next_token);
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
