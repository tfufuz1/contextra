// FILE-CONTEXT
// STAND: 2026-09-22T00:00:00Z
// ZWECK: Prefix reuse helper structs, context, and key/seed builders for KV cache stage B (Spec §9.2).
// INVARIANTEN: Zero-Panic doctrine; at least 1 token must be computed forward during prefill to obtain logits.

use crate::kv_state::KvState;
use contextra_ports::kv::{KvBlock, KvLayout, KvPrefixHit, KvPrefixStore, PrefixKey, RopeConfig};
use contextra_types::{ContextraError, ModelFingerprint, Result};
use sha2::{Digest, Sha256};
use std::fmt;
use std::sync::Arc;

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

/// Wrapper context holding a thread-safe KV prefix store reference.
#[derive(Clone)]
pub struct KvPrefixContext {
    /// Reference to the underlying KV prefix store port implementation.
    pub store: Arc<dyn KvPrefixStore>,
}

impl fmt::Debug for KvPrefixContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KvPrefixContext")
            .field("store", &"<dyn KvPrefixStore>")
            .finish()
    }
}

/// Constructs a `PrefixKey` for the given model fingerprint, tokenizer, layout, and RoPE config.
pub fn build_prefix_key(
    fingerprint: &ModelFingerprint,
    tokenizer: &tokenizers::Tokenizer,
    layout: KvLayout,
    rope: RopeConfig,
) -> Result<PrefixKey> {
    let json_str = tokenizer.to_string(false).map_err(|e| {
        ContextraError::Internal(format!("Failed to serialize tokenizer to JSON: {e}"))
    })?;
    let mut hasher = Sha256::new();
    hasher.update(json_str.as_bytes());
    let result = hasher.finalize();
    let mut tokenizer_hash = [0u8; 32];
    tokenizer_hash.copy_from_slice(&result);

    Ok(PrefixKey {
        model: fingerprint.clone(),
        tokenizer_hash,
        layout,
        rope,
    })
}

/// Reconstructs a `PrefixSeed` from a `KvPrefixHit` and total prompt token count.
///
/// Ensures at least one prompt token remains to be evaluated forward in the model to yield logits.
pub fn seed_from_hit(hit: &KvPrefixHit, tokens_len: usize) -> Result<PrefixSeed> {
    if tokens_len == 0 {
        return Err(ContextraError::InvalidInput(
            "Cannot seed prefix from empty tokens list".to_string(),
        ));
    }
    let matched_len = hit.matched_tokens.min(tokens_len.saturating_sub(1));
    let mut state = KvState::new(Vec::new(), 0);
    for block in &hit.blocks {
        state.import_block(block)?;
    }
    state.truncate(matched_len)?;
    Ok(PrefixSeed { matched_len, state })
}
