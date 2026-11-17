// FILE-CONTEXT
// STAND: 2026-09-09T15:45:22Z
// ZWECK: Embedding provider definitions and constants.

/// Maximum batch size for Candle GGUF embeddings.
pub const MAX_CANDLE_EMBED_BATCH_SIZE: usize = 256;

#[cfg(feature = "workspace")]
pub use crate::compat::embedding::verify_embedding_provider;
