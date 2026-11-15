// FILE-CONTEXT
// STAND: 2026-11-08
// ZWECK: HTTP-Client für lokale Ollama LLM/Embedding API mit Retry/Timeout/Streaming-Semantik
// INVARIANTEN: Explicite Timeouts für jeden HTTP-Request; Retry nur bei transienten Net-Errors/5xx; Thread-safe Client via reqwest Arc-Pool
// NICHT-OFFENSICHTLICH: Client-Fehler 4xx (400, 404) niemals retrien; Automatischer Fallback auf sequentielles Embedding falls /api/embed fehlt
// HOTSPOTS: embed, try_embed_batch, generate_text, chat_with_rag_streaming

#[cfg(feature = "client")]
mod circuit_breaker;
pub mod config;
#[cfg(feature = "client")]
mod core;
#[cfg(feature = "client")]
mod errors;
#[cfg(feature = "client")]
pub mod policy;
pub mod validation;
#[cfg(feature = "client")]
mod wire;

#[cfg(all(test, feature = "client"))]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn test_core_parse_prompt_roundtrip() {
        let prompt = "<system>sys</system><instructions>ins</instructions><context>ctx</context><user_query>q</user_query>";
        let parsed = parse_prompt_template(prompt).expect("parse should succeed");
        assert_eq!(parsed.system, "sys");
    }
}

#[cfg(feature = "client")]
pub use circuit_breaker::CircuitBreaker;
pub use config::*;
#[cfg(all(test, feature = "client"))]
pub(crate) use core::{parse_prompt_template, retry_jitter, ParsedPrompt};
#[cfg(feature = "client")]
pub use core::{read_response_bytes, read_response_text, OllamaClient};
#[cfg(feature = "client")]
pub use errors::*;
#[cfg(feature = "client")]
pub use policy::{
    EgressAuditSink, EndpointPolicy, NoopEgressAuditSink, PayloadGate, PermissivePayloadGate,
};
pub use validation::*;
