// FILE-CONTEXT
// STAND: 2026-09-15T16:10:40Z (SESSION: ec33599e)
// ZWECK: Cross-Encoder Reranking für Post-RRF Präzisionsverbesserung.
// INVARIANTEN: Falls onnx-Feature inaktiv, greift transparenter Passthrough-Fallback.
// NICHT-OFFENSICHTLICH: OnnxReranker nutzt ein eigenes Arc<Mutex<Session>> getrennt von TextEmbedder.
// SIEHE AUCH: crates/contextra-embed/AGENTS.md, rules/detect_nested_locks.yml

//! Cross-Encoder Reranking für Post-RRF Präzisionsverbesserung.
//!
//! Implementiert Contextra Post-Fusion Cross-Encoder Reranking: nach RRF-Fusion
//! werden die Top-K Kandidaten durch ein lokales ONNX Cross-Encoder-Modell
//! neu bewertet.
//!
//! Aktivierung: Feature-Flag `onnx` erforderlich.
//! Modell: bge-reranker-base oder ms-marco-MiniLM-L-6-v2 (ONNX-Export).

mod config;
mod cross_encoder;
#[cfg(feature = "onnx")]
mod onnx;

#[cfg(test)]
mod tests;

pub use config::{PlattScaledSigmoid, RerankConfig, RerankResult, MAX_CANDIDATES};
pub use cross_encoder::CrossEncoderReranker;

// REVIEW-PASS[1/2] STATUS:PASS (TS: 2026-09-04T11:42:28Z) (SESSION: 3e5150c8) PRÜFER-KONTEXT: FRESH - Verified CrossEncoderReranker passthrough fallback, candidate limit bounds, and zero-unsafe invariants.
// REVIEW-PASS[2/2] STATUS:PASS (TS: 2026-09-06T11:19:00Z) (SESSION: 8efa6210) PRÜFER-KONTEXT: FRESH - Verified ML domain APM alignment (APM-22, APM-23, APM-24), zero-unsafe in production, and hermetic feature isolation.
