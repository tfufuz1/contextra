// FILE-CONTEXT
// STAND: 2026-08-30T18:54:39Z (SESSION: ed7b7b38)
// ZWECK: HTTP-Client für lokale Ollama LLM/Embedding API mit Retry/Timeout/Streaming-Semantik
// INVARIANTEN: Explicite Timeouts für jeden HTTP-Request; Retry nur bei transienten Net-Errors/5xx; Thread-safe Client via reqwest Arc-Pool
// NICHT-OFFENSICHTLICH: Client-Fehler 4xx (400, 404) niemals retrien; Automatischer Fallback auf sequentielles Embedding falls /api/embed fehlt
// HOTSPOTS: embed, try_embed_batch, generate_text, chat_with_rag_streaming

mod config;
mod core;
mod errors;
mod trait_impls;
mod validation;
mod wire;

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests;

pub use config::*;
pub use core::OllamaClient;
#[allow(unused_imports)]
pub(crate) use core::{parse_prompt_template, ParsedPrompt};
pub use errors::*;
pub use validation::*;
