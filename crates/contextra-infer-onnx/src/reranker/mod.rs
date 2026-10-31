// FILE-CONTEXT
// STAND: 2026-10-31
// ZWECK: Cross-Encoder Reranking für Post-RRF Präzisionsverbesserung (Ring 2).
// INVARIANTEN: Falls onnx-Feature inaktiv, greift transparenter Passthrough-Fallback.
// NICHT-OFFENSICHTLICH: OnnxReranker nutzt ein eigenes Arc<Mutex<Session>> getrennt von TextEmbedder.
// SIEHE AUCH: crates/contextra-infer-onnx/AGENTS.md, rules/detect_nested_locks.yml

//! # Submodul `reranker` — Cross-Encoder Reranking
//!
//! **Ring & Klasse:** Ring 2 (Flüchtige Abhängigkeit / Rand `[R]`).
//! **Status:** Optionales ML-Inferenz-Submodul zur Nachbewertung von Suchergebnissen.
//!
//! ## Fachlicher Hintergrund
//! Nach einer hybriden Suche (z. B. Kombination aus Volltextsuche und Vektorsuche via Reciprocal Rank Fusion / RRF)
//! liegen vorab gefilterte Kandidatendokumente vor. Ein Cross-Encoder bewertet ein Paar aus
//! `(Suchanfrage, Dokument)` gemeinsam im selben Transformer-Netzwerk. Dadurch werden Wechselwirkungen
//! zwischen den Wörtern der Anfrage und des Dokuments tief modelliert, was zu deutlich präziseren
//! Relevanz-Scores führt als rein bi-kodierte Vektor-Ähnlichkeiten (Bi-Encoder).
//!
//! ## Funktionsweise & Fallback
//! - **Mit `feature = "onnx"`:** Nutzt die lokale ONNX Runtime (`ort`) und HuggingFace-Tokenizer
//!   (z. B. `bge-reranker-base` oder `ms-marco-MiniLM-L-6-v2`), um logits zu berechnen und via
//!   Platt-Scaling (`PlattScaler`) zu Wahrscheinlichkeiten zu kalibrieren.
//! - **Ohne `feature = "onnx"` (Default):** Transparenter Passthrough-Fallback, der die
//!   übergebene Reihenfolge beibehält und deterministische synthetische Scores vergibt.

mod config;
mod cross_encoder;
#[cfg(feature = "onnx")]
mod onnx;

#[cfg(test)]
mod tests;

// ANCHOR[contextra-infer-onnx:RERANKER-REEXPORTS]
/// Re-Exports der Konfigurationstypen, Builder und Ergebnisstrukturen.
pub use config::{
    OnnxConfig, OnnxRerankerBuilder, PlattScaledSigmoid, RerankConfig, RerankResult, MAX_CANDIDATES,
};
/// Re-Export des primären Reranker-Strukturtyps.
pub use cross_encoder::CrossEncoderReranker;

// REVIEW-PASS[1/2] STATUS:PASS (TS: 2026-09-04T11:42:28Z) (SESSION: 3e5150c8) PRÜFER-KONTEXT: FRESH - Verified CrossEncoderReranker passthrough fallback, candidate limit bounds, and zero-unsafe invariants.
// REVIEW-PASS[2/2] STATUS:PASS (TS: 2026-09-06T11:19:00Z) (SESSION: 8efa6210) PRÜFER-KONTEXT: FRESH - Verified ML domain APM alignment (APM-22, APM-23, APM-24), zero-unsafe in production, and hermetic feature isolation.
