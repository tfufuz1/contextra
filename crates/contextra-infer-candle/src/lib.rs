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

pub mod error;
pub mod fake;
pub mod gguf_loader;
pub mod traits;

pub use error::{Error, Result};
pub use fake::{FakeEmbedder, FakeTextGenerator};
pub use gguf_loader::{
    parse_gguf_metadata, parse_gguf_metadata_with_checksum, GgufMetadata, MAX_GGUF_METADATA_KVS,
    MAX_GGUF_STRING_LEN, MAX_GGUF_TENSORS,
};
pub use traits::{Embedder, GenerationConfig, KvStateSink, KvStateSource, TextGenerator};

#[cfg(feature = "candle")]
pub mod attention_exporter;
#[cfg(feature = "candle")]
pub mod embedding;
#[cfg(feature = "candle")]
pub mod embedding_provider;
#[cfg(feature = "candle")]
pub mod inference;
#[cfg(feature = "candle")]
pub mod model_registry;

#[cfg(all(feature = "candle", feature = "experimental"))]
pub mod gasp;

#[cfg(all(feature = "candle", feature = "experimental"))]
pub mod kv_bridge;

#[cfg(all(feature = "candle", feature = "experimental"))]
pub mod kv_state;
#[cfg(all(feature = "candle", feature = "experimental"))]
pub mod model;

#[cfg(feature = "candle")]
pub use attention_exporter::*;
#[cfg(feature = "candle")]
pub use embedding::CandleEmbedClient;
#[cfg(feature = "candle")]
pub use embedding_provider::{verify_embedding_provider, MAX_CANDLE_EMBED_BATCH_SIZE};
#[cfg(all(feature = "candle", feature = "experimental"))]
pub use gasp::{GaspConfig, GaspValidator, DEFAULT_GROUNDING_THRESHOLD};
#[cfg(all(feature = "candle", feature = "experimental"))]
pub use inference::QuantizedLlamaModel;
#[cfg(feature = "candle")]
pub use inference::*;
#[cfg(all(feature = "candle", feature = "experimental"))]
pub use kv_bridge::KvBridgeAdapter;
#[cfg(all(feature = "candle", feature = "experimental"))]
pub use kv_state::{KvState, LayerKv};
#[cfg(all(feature = "candle", feature = "experimental"))]
pub use model::quantized_llama;
#[cfg(feature = "candle")]
pub use model_registry::{compute_fingerprint, CandleQuantization, ModelFingerprint};
