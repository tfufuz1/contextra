#![cfg_attr(test, allow(clippy::disallowed_methods))]
#![allow(clippy::indexing_slicing)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![deny(dead_code)]
// FILE-CONTEXT
// STAND: 2026-11-09T00:00:00Z
// ZWECK: Crate root for contextra-infer-candle native GGUF inference backend.
// INVARIANTEN: Zero unsafe code in crate; pure-Rust fallback without candle; fail-closed.

//! `contextra-infer-candle`: Native Candle GGUF ML Inferenz-Baustein für Contextra (Klasse A).
//!
//! # Architektur
//! - **Pure Rust Standard**: Ohne aktiviertes Feature `candle` baut dieser Crate autark
//!   mit lokalen Traits (`Embedder`, `TextGenerator`) und Fake-Backends.
//! - **GGUF-Laden**: Robuster, defensiver GGUF-Header- und Metadaten-Parser mit SHA-256 Validierung.
//! - **Optionale Hardwarebeschleunigung**: Unterstützt CUDA und Metal via Feature-Flags.

#![forbid(unsafe_code)]

pub mod compat;
pub mod error;
pub mod traits;

#[cfg(feature = "test-utils")]
pub mod fake;

#[cfg(feature = "candle")]
pub mod gguf_loader;

pub use error::{CandleError, Error, Result};
pub use fake::{FakeEmbedder, FakeTextGenerator};
#[cfg(feature = "candle")]
pub use gguf_loader::{
    parse_gguf_metadata, parse_gguf_metadata_with_checksum, GgufMetadata, MAX_GGUF_METADATA_KVS,
    MAX_GGUF_STRING_LEN, MAX_GGUF_TENSORS,
};
pub use traits::{
    ConfidenceCalibrator, Embedder, GenerationConfig, IdentityCalibrator, KvStateSink,
    KvStateSource, TextGenerator,
};

pub mod attention_exporter;
#[cfg(feature = "candle")]
pub mod embedding;
#[cfg(feature = "candle")]
pub mod embedding_provider;
#[cfg(feature = "candle")]
pub mod inference;
pub mod model_registry;

pub mod gasp;

#[cfg(feature = "kv-bridge")]
pub mod kv_bridge;

#[cfg(all(feature = "candle", feature = "kv-stage-b"))]
pub mod kv_state;
#[cfg(all(feature = "candle", feature = "kv-stage-b"))]
pub mod model;

pub use attention_exporter::*;
#[cfg(feature = "candle")]
pub use embedding::CandleEmbedClient;
#[cfg(feature = "candle")]
pub use embedding_provider::{verify_embedding_provider, MAX_CANDLE_EMBED_BATCH_SIZE};
pub use gasp::{GaspConfig, GaspValidator, DEFAULT_GROUNDING_THRESHOLD};
#[cfg(all(feature = "candle", feature = "kv-stage-b"))]
pub use inference::QuantizedLlamaModel;
#[cfg(feature = "candle")]
pub use inference::*;
#[cfg(feature = "kv-bridge")]
pub use kv_bridge::KvBridgeAdapter;
#[cfg(all(feature = "candle", feature = "kv-stage-b"))]
pub use kv_state::{KvState, LayerKv};
#[cfg(all(feature = "candle", feature = "kv-stage-b"))]
pub use model::quantized_llama;
pub use model_registry::{compute_fingerprint, CandleQuantization, ModelFingerprint};
