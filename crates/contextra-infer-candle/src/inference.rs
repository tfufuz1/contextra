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

// TODO(contextra-plan): Plan for Candle ML inference path: Verify real GGUF forward-passes, backpressure semaphore limits, and non-constant output proofs across variable prompts [ref:eigenbau-candle-tensor]

mod core;

#[cfg(feature = "kv-stage-b")]
mod prefix_reuse;

#[cfg(test)]
mod tests;

pub use core::*;

#[cfg(feature = "kv-stage-b")]
pub use prefix_reuse::*;
