// FILE-CONTEXT
// STAND: 2026-09-12T00:00:00Z
// ZWECK: Generation sampling configuration with explicit RNG seed, temperature, top_p, context length, and token budget limits.

use contextra_types::ContextraError;

/// Sampling and resource bounds configuration for Candle LLM text generation.
#[derive(Debug, Clone)]
pub struct GenerationConfig {
    /// Random number generator seed (default 299792458).
    pub seed: u64,
    /// Sampling temperature (0.0 = greedy deterministic bit-identical sampling).
    pub temperature: f64,
    /// Top-p nucleus sampling probability threshold (None or Some(0.0..=1.0)).
    pub top_p: Option<f64>,
    /// Maximum context length in tokens permitted for input prompts.
    pub max_context_length: usize,
    /// Maximum token budget permitted for generated output.
    pub max_token_budget: usize,
}

impl Default for GenerationConfig {
    fn default() -> Self {
        Self {
            seed: 299792458,
            temperature: 0.7,
            top_p: Some(0.9),
            max_context_length: 4096,
            max_token_budget: 512,
        }
    }
}

impl GenerationConfig {
    /// Creates a deterministic generation configuration (temperature = 0.0).
    pub fn deterministic(seed: u64) -> Self {
        Self {
            seed,
            temperature: 0.0,
            top_p: None,
            max_context_length: 4096,
            max_token_budget: 512,
        }
    }

    /// Sets the maximum context length in tokens.
    pub fn with_max_context_length(mut self, limit: usize) -> Self {
        self.max_context_length = limit;
        self
    }

    /// Sets the maximum token budget for generation.
    pub fn with_max_token_budget(mut self, budget: usize) -> Self {
        self.max_token_budget = budget;
        self
    }

    /// Validates input prompt token length against `max_context_length`.
    pub fn validate_prompt_len(&self, prompt_token_count: usize) -> Result<(), ContextraError> {
        if prompt_token_count > self.max_context_length {
            return Err(ContextraError::LimitExceeded {
                limit: self.max_context_length,
                context: format!(
                    "Prompt token count {prompt_token_count} exceeds maximum context length limit {}",
                    self.max_context_length
                ),
            });
        }
        Ok(())
    }

    /// Validates target sample length against `max_token_budget`.
    pub fn validate_token_budget(&self, requested_tokens: usize) -> Result<(), ContextraError> {
        if requested_tokens > self.max_token_budget {
            return Err(ContextraError::LimitExceeded {
                limit: self.max_token_budget,
                context: format!(
                    "Requested token generation count {requested_tokens} exceeds maximum token budget {}",
                    self.max_token_budget
                ),
            });
        }
        Ok(())
    }
}
