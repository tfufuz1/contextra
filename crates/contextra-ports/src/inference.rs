//! Inference and model engine port contracts.

pub use crate::embedding::{
    ContextSegment, EmbeddingError, EmbeddingProvider, LlmTextGenerator, LlmTextGeneratorStreaming,
    TextEmbeddingEngine,
};
pub use crate::query_rewriter::QueryRewriter;
pub use crate::reranker::Reranker;

/// Prelude for inference ports.
pub mod prelude {
    pub use super::{
        EmbeddingProvider, LlmTextGenerator, QueryRewriter, Reranker, TextEmbeddingEngine,
    };
}
