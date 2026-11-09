// FILE-CONTEXT
// STAND: 2026-11-08
// ZWECK: HTTP-Client für lokale Ollama LLM/Embedding API mit Retry/Timeout/Streaming-Semantik
// INVARIANTEN: Explicite Timeouts für jeden HTTP-Request; Retry nur bei transienten Net-Errors/5xx; Thread-safe Client via reqwest Arc-Pool
// NICHT-OFFENSICHTLICH: Client-Fehler 4xx (400, 404) niemals retrien; Automatischer Fallback auf sequentielles Embedding falls /api/embed fehlt
// HOTSPOTS: embed, try_embed_batch, generate_text, chat_with_rag_streaming

#[cfg(feature = "http")]
mod circuit_breaker;
pub mod config;
#[cfg(feature = "http")]
mod core;
#[cfg(feature = "http")]
mod errors;
#[cfg(feature = "http")]
pub mod policy;
#[cfg(all(feature = "http", feature = "ports-compat"))]
mod trait_impls;
pub mod validation;
#[cfg(feature = "http")]
mod wire;

#[cfg(all(test, feature = "http"))]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests;

#[cfg(feature = "http")]
pub use circuit_breaker::CircuitBreaker;
pub use config::*;
#[cfg(all(test, feature = "http"))]
#[allow(unused_imports)]
pub(crate) use core::{parse_prompt_template, retry_jitter, ParsedPrompt};
#[cfg(feature = "http")]
pub use core::{read_response_bytes, read_response_text, OllamaClient};
#[cfg(feature = "http")]
pub use errors::*;
#[cfg(feature = "http")]
pub use policy::*;
pub use validation::*;
