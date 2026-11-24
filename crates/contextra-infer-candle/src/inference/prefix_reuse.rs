// FILE-CONTEXT
// STAND: 2026-09-22T00:00:00Z
// ZWECK: Prefix reuse helper structs, context, and key/seed builders for KV cache stage B (Spec §9.2).
// INVARIANTEN: Zero-Panic doctrine; at least 1 token must be computed forward during prefill to obtain logits.

use crate::error::{Error, Result};
use crate::kv_state::{KvBlock, KvState};

/// Seed state for executing prompt generation with an imported KV cache prefix.
#[derive(Debug, Clone)]
pub struct PrefixSeed {
    /// Number of prompt tokens covered by the imported KV state.
    pub matched_len: usize,
    /// Imported and truncated KV state ready for remaining prompt prefill.
    pub state: KvState,
}

/// Execution result of a prefix-assisted generation run.
#[derive(Debug, Clone)]
pub struct PrefixRun {
    /// Generated output string.
    pub output: String,
    /// Tokenized prompt sequence.
    pub prompt_tokens: Vec<u32>,
    /// Number of prompt tokens reused from the KV store.
    pub reused_tokens: usize,
    /// Exported KV block covering the prompt, if produced.
    pub exported_prefix: Option<KvBlock>,
}
