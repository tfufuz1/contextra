// FILE-CONTEXT
// STAND: 2026-09-16T16:13:00Z (SESSION: afafdd44)
// ZWECK: Crate root for contextra-candle native GGUF inference backend.
// INVARIANTEN: Zero unsafe code in crate; re-exports core inference types and gasp validator.

//! `contextra-candle`: Native Candle GGUF ML Inferenz-Backend für Contextra.
//!
//! # Architektur-Strategie
//!
//! **Strategie B** (mistral.rs-artiger Ansatz):
//! Dieses Crate bietet eine schnell deploybare, native Inferenz-Engine hinter den bestehenden
//! Traits (`LlmTextGenerator`, `EmbeddingProvider`, `TextEmbeddingEngine`).
//! Es kapselt Modell-Laden und Tensor-Inferenz ohne direkten Attention-Level- oder
//! RoPE-Shift-Zugriff auf KV-Cache-Ebene.
//!
//! *Hinweis*: **Strategie A** (expliziter RoPE-Shift- und KV-Cache-Bridge-Zugriff für mandantenisolierte
//! Cache-Projektionen) ist ein separates, zukünftiges Vorhaben und NICHT Gegenstand dieser Erstfassung.

#![forbid(unsafe_code)]

// REVIEW-PASS[1/2] (ID: AGT-CANDLE-d495a019) (TS: 2026-09-16T16:13:00Z) (SESSION: afafdd44) PRÜFER-KONTEXT: FRESH

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
pub use inference::QuantizedLlamaModel;
pub use inference::*;
#[cfg(feature = "kv-bridge")]
pub use kv_bridge::KvBridgeAdapter;
#[cfg(feature = "kv-stage-b")]
pub use kv_state::{KvState, LayerKv};
#[cfg(feature = "kv-stage-b")]
pub use model::quantized_llama;
pub use model_registry::{compute_fingerprint, CandleQuantization, ModelFingerprint};
