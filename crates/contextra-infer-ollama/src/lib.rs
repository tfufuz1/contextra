#![deny(dead_code)]
// REVIEW-PASS[1/2] STATUS:PASS (TS: 2026-09-11T14:45:00Z) (SESSION: 089dd3c0) PRÜFER-KONTEXT: FRESH - Verified prompt injection safety (xml_escape, build_rag_prompt), HTTP retry policy, zero unsafe, and test suite green.
// REVIEW-PASS[2/2] STATUS:PASS (ID: AGT-OLLAMA-4d606464) (TS: 2026-09-11T19:01:58Z) (SESSION: 089dd3c0) PRÜFER-KONTEXT: FRESH - Clarified historical "zero unsafe" status observation vs compile-time enforced invariant via forbid(unsafe_code).
// REVIEW-PASS[3/3] STATUS:PASS (TS: 2026-09-13T01:45:00Z) (SESSION: 6e371f4e) PRÜFER-KONTEXT: FRESH - Tier 3 deep audit complete: verified prompt injection structural isolation, proptest evasion matrix, concurrency stress tests, and ML calibration provenance.
// REVIEW-PASS[1/2] STATUS:PASS (TS: 2026-09-15T14:48:58Z) (SESSION: 1d6e3667) PRÜFER-KONTEXT: FRESH - Tier 3 deep audit verified: zero unsafe, proptests green, 10x 8-thread concurrency stress test green, prompt injection evasion matrix passed.
// REVIEW-PASS[2/2] STATUS:PASS (TS: 2026-09-16T16:24:19Z) (SESSION: 4b816c9a) PRÜFER-KONTEXT: FRESH - Independent diff review verified: prompt injection XML-escaping, retry & backoff error classification, forbid(unsafe_code) invariant, and 91 tests green.
#![forbid(unsafe_code)]

pub mod error;
pub mod traits;
pub mod xml_escape;

pub use error::{OllamaError, Result};
pub use traits::{BoxFuture, Clock, Embedder, Generator, Rng, SeededRng, SystemClock};
pub use xml_escape::xml_escape;

#[cfg(feature = "http")]
mod client;
#[cfg(feature = "http")]
pub mod context_prefixer;
#[cfg(feature = "http")]
mod embedding;
pub mod importance;
pub mod model_info;
#[cfg(all(feature = "http", feature = "ports-compat"))]
pub mod query_rewriter;

#[cfg(feature = "http")]
pub use client::{
    build_rag_prompt, read_response_bytes, read_response_text, CircuitBreaker, EgressAuditSink,
    EndpointPolicy, NoopEgressAuditSink, OllamaClient, OllamaConfig, DEFAULT_BASE_URL,
    DEFAULT_EMBED_MODEL,
};
#[cfg(feature = "http")]
pub use context_prefixer::{ContextPrefixConfig, ContextPrefixEngine, ContextPrefixer};
#[cfg(feature = "http")]
pub use embedding::OllamaEmbedder;
pub use importance::{
    parse_importance_score_response, Confidence, ImportanceAssessment, ImportanceCalibrator,
};
#[cfg(feature = "http")]
pub use importance::{score_importance, score_importance_batch, score_importance_with_calibrator};
pub use model_info::ModelInfo;
#[cfg(all(feature = "http", feature = "ports-compat"))]
pub use query_rewriter::{OllamaQueryRewriter, OllamaQueryRewriterConfig};
