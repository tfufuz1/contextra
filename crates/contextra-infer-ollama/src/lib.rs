#![deny(dead_code)]
#![forbid(unsafe_code)]

pub mod client;
pub mod context_prefixer;
#[cfg(feature = "client")]
mod embedding;
pub mod error;
pub mod importance;
pub mod model_info;
#[cfg(all(feature = "client", feature = "ports-compat"))]
pub mod query_rewriter;
pub mod traits;

pub use client::{build_rag_prompt, xml_escape};
#[cfg(feature = "client")]
pub use client::{
    read_response_bytes, read_response_text, CircuitBreaker, EgressAuditSink, EndpointPolicy,
    NoopEgressAuditSink, OllamaClient, OllamaConfig, PayloadGate, PermissivePayloadGate,
    DEFAULT_BASE_URL, DEFAULT_EMBED_MODEL,
};

pub use context_prefixer::{ContextPrefixConfig, ContextPrefixEngine, ContextPrefixer};
#[cfg(feature = "client")]
pub use embedding::OllamaEmbedder;
pub use error::{
    LocalClock, LocalImportanceScore, LocalQueryRewriteOutput, LocalRng, LocalSeededRng,
    OllamaError, Result, SystemClock,
};
pub use importance::{
    parse_importance_score_response, Confidence, ImportanceAssessment, ImportanceCalibrator,
};
#[cfg(feature = "client")]
pub use importance::{score_importance, score_importance_batch, score_importance_with_calibrator};
pub use model_info::{known_dimension, ModelInfo};
#[cfg(all(feature = "client", feature = "ports-compat"))]
pub use query_rewriter::{OllamaQueryRewriter, OllamaQueryRewriterConfig};
pub use traits::{AsyncEmbedder, BoxFuture, Embedder, TextGenerator};
