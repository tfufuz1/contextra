//! Typ-Konvertierungen und Brücken für KV-Cache-Typen (KvLayout, RopeConfig, KvBlock, PrefixKey).

use crate::error::Result as LocalResult;
use crate::inference::prefix_reuse::PrefixSeed;
use crate::kv_state::{
    KvBlock as LocalKvBlock, KvLayout as LocalKvLayout, KvState, RopeConfig as LocalRopeConfig,
};
use crate::model_registry::ModelFingerprint;
use contextra_ports::kv::{
    KvBlock as PortsKvBlock, KvLayout as PortsKvLayout, KvPrefixHit, KvPrefixStore, PrefixKey,
    RopeConfig as PortsRopeConfig,
};
use contextra_types::{ContextraError, Result};
use sha2::{Digest, Sha256};
use std::fmt;
use std::sync::Arc;

impl From<LocalKvLayout> for PortsKvLayout {
    fn from(l: LocalKvLayout) -> Self {
        PortsKvLayout {
            n_layer: l.n_layer,
            n_kv_head: l.n_kv_head,
            head_dim: l.head_dim,
            dtype: l.dtype,
        }
    }
}

impl From<PortsKvLayout> for LocalKvLayout {
    fn from(l: PortsKvLayout) -> Self {
        LocalKvLayout {
            n_layer: l.n_layer,
            n_kv_head: l.n_kv_head,
            head_dim: l.head_dim,
            dtype: l.dtype,
        }
    }
}

impl From<LocalRopeConfig> for PortsRopeConfig {
    fn from(r: LocalRopeConfig) -> Self {
        PortsRopeConfig {
            base: r.base,
            scaling: r.scaling,
        }
    }
}

impl From<PortsRopeConfig> for LocalRopeConfig {
    fn from(r: PortsRopeConfig) -> Self {
        LocalRopeConfig {
            base: r.base,
            scaling: r.scaling,
        }
    }
}

impl From<LocalKvBlock> for PortsKvBlock {
    fn from(b: LocalKvBlock) -> Self {
        PortsKvBlock {
            block_id: b.block_id,
            data: b.data,
        }
    }
}

impl From<PortsKvBlock> for LocalKvBlock {
    fn from(b: PortsKvBlock) -> Self {
        LocalKvBlock {
            block_id: b.block_id,
            data: b.data,
        }
    }
}

/// Wrapper context holding a thread-safe KV prefix store reference.
#[derive(Clone)]
pub struct KvPrefixContext {
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
    layout: impl Into<PortsKvLayout>,
    rope: impl Into<PortsRopeConfig>,
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
        model: fingerprint.clone().into(),
        tokenizer_hash,
        layout: layout.into(),
        rope: rope.into(),
    })
}

/// Reconstructs a `PrefixSeed` from a `KvPrefixHit` and total prompt token count.
pub fn seed_from_hit(hit: &KvPrefixHit, tokens_len: usize) -> Result<PrefixSeed> {
    if tokens_len == 0 {
        return Err(ContextraError::InvalidInput(
            "Cannot seed prefix from empty tokens list".to_string(),
        ));
    }
    let matched_len = hit.matched_tokens.min(tokens_len.saturating_sub(1));
    let mut state = KvState::new(Vec::new(), 0);
    for block in &hit.blocks {
        let local_block = LocalKvBlock {
            block_id: block.block_id,
            data: block.data.clone(),
        };
        state
            .import_block(&local_block)
            .map_err(|e| ContextraError::Internal(e.to_string()))?;
    }
    state
        .truncate(matched_len)
        .map_err(|e| ContextraError::Internal(e.to_string()))?;
    Ok(PrefixSeed { matched_len, state })
}
