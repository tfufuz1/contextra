//! `contextra-infer-onnx` — Autarke In-Process Inferenz für Embeddings und Reranking (Klasse A, Ring 2).
//!
//! Stellt isolierte lokale Inferenz-Funktionalität hinter Feature-Flags bereit.
//! Standard-Build ist vollständig Pure-Rust ohne externe Shared-Libraries oder C-FFI.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod config;
pub mod embedder;
pub mod error;
pub mod fakes;
pub mod reranker;
pub mod traits;

// Re-Exports für bequeme Verwendung
pub use config::{
    verify_path_safety, EmbedderConfig, ModelHash, PoolingMode, RerankerConfig,
};
pub use embedder::OnnxEmbedder;
pub use error::{Error, Result};
pub use fakes::{FakeEmbedder, FakeReranker};
pub use reranker::{CrossEncoderReranker, OnnxReranker};
pub use traits::{Embedder, Reranker};

/// Standard-Cacheverzeichnis für heruntergeladene Modelle.
#[must_use]
pub fn default_model_cache_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(".contextra/models")
}
