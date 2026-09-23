// FILE-CONTEXT
// STAND: 2026-09-12T00:00:00Z (SESSION: BACKPRESSURE-CONTRACT-D1)
// ZWECK: Candle LLM text generator client implementing LlmTextGenerator.
// INVARIANTEN: Thread-safe model access via Mutex; spawn_blocking for CPU inference execution.
// Backpressure contract: max_concurrent_inferences limits spawn_blocking calls.
// Callers will experience backpressure (await on permit acquire) rather than Tokio thread pool exhaustion.

//! memfuse-candle LLM inference module.
//!
//! Backpressure contract: `max_concurrent_inferences` limits `spawn_blocking` calls.
//! Callers will experience backpressure (await on permit acquire) rather than Tokio thread pool exhaustion.

mod core;

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests;

pub use core::*;
