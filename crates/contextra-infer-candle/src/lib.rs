// FILE-CONTEXT
// STAND: 2026-10-21T00:00:00Z
// ZWECK: Crate root for contextra-infer-candle native GGUF inference backend.
// INVARIANTEN: Zero unsafe code in crate; re-exports core inference types and gasp validator.

//! `contextra-infer-candle`: Native Candle GGUF ML Inferenz-Backend für Contextra.
//!
//! # Architektur-Strategie
//!
//! **Strategie B** (mistral.rs-artiger Ansatz):
//! Dieses Crate bietet eine schnell deploybare, native Inferenz-Engine hinter den bestehenden
//! Traits (`LlmTextGenerator`, `EmbeddingProvider`, `TextEmbeddingEngine`).
//! Es kapselt Modell-Laden und Tensor-Inferenz ohne direkten Attention-Level- oder
//! RoPE-Shift-Zugriff auf KV-Cache-Ebene.

#![forbid(unsafe_code)]
#![allow(clippy::arithmetic_side_effects, clippy::indexing_slicing)]

pub mod attention_exporter;
pub mod embedding;
pub mod embedding_provider;
pub mod gguf_loader;
pub mod inference;
pub mod model_registry;

pub mod gasp;

#[cfg(feature = "kv-bridge")]
pub mod kv_bridge;

#[cfg(feature = "kv-stage-b")]
pub mod kv_state;
#[cfg(feature = "kv-stage-b")]
pub mod model;

pub use attention_exporter::*;
pub use embedding::CandleEmbedClient;
pub use embedding_provider::MAX_CANDLE_EMBED_BATCH_SIZE;
pub use gasp::{GaspConfig, GaspValidator, DEFAULT_GROUNDING_THRESHOLD};
#[cfg(feature = "kv-stage-b")]
pub use inference::{QuantizedLlamaModel, QuantizedQwen2Model};
pub use inference::*;
#[cfg(feature = "kv-bridge")]
pub use kv_bridge::KvBridgeAdapter;
#[cfg(feature = "kv-stage-b")]
pub use kv_state::{KvState, LayerKv};
#[cfg(feature = "kv-stage-b")]
pub use model::{quantized_llama, quantized_qwen2};
pub use model_registry::{compute_fingerprint, CandleQuantization, ModelFingerprint};
