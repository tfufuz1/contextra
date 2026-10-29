// FILE-CONTEXT
// STAND: 2026-09-12T00:00:00Z
// ZWECK: Traits for Candle LLM inner model execution.

use candle_core::Device;
use contextra_types::Result;

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
        contextra_ports::kv::KvLayout,
        contextra_ports::kv::RopeConfig,
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
        let tokens = tokenizer.encode(prompt, true).map_err(|e| {
            contextra_types::ContextraError::InvalidInput(format!("Failed to tokenize prompt: {e}"))
        })?;
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

/// Default inner Candle LLM mock model for unit tests when binary weights are absent.
pub struct DefaultCandleLlmModel;

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
