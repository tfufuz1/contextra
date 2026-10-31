//! # Contextra Rank (`contextra-rank`)
//!
//! **Ring:** Ring 0 (Ranking & Kalibrierung / Kernlogik)
//! **Klasse:** Rand `[R]`
//! **Reifegrad:** Experimentell / Aktiv (Kern v1)
//!
//! ## Zweck des Crates
//! Dieses Crate implementiert Methoden zur Fusion, Kalibrierung und Erklärung
//! von Retrieval-Ergebnissen im Contextra-System. Dazu gehören:
//! - **4-Signal-Fusion:** Verknüpfung von Vektor-, Text- (BM25), Graph- und
//!   Rerank-Scores mittels reziproker Rangfusion (RRF = Reciprocal Rank Fusion)
//!   und score-basierter Normalisierung.
//! - **Kalibrierung:** Umwandlung von Roh-Scores in Wahrscheinlichkeiten via
//!   Platt-Scaling (logistische Regression) und Isotonische Regression
//!   (PAVA = Pool-Adjacent Violators Algorithm).
//! - **Konforme Prädiktion:** Finite-Sample-Abdeckungsgarantien und adaptive
//!   Schwellenwertanpassung bei Verteilungsverschiebung (Covariate Shift).
//! - **Drift-Erkennung:** Überwachung von Score-Verteilungen zur Laufzeit.
//! - **Erklärbarkeit (Explainability):** Umwandlung maschinenlesbarer Herkunftsnachweise
//!   (`ProvenanceRecord`) in strukturierte und deutschsprachige Erklärungen.
//! - **DiBud (Discrete Budgeted):** Jederzeit abbrechbare Streaming-Fusion unter Zeit-
//!   und Kanalbudgets.
//!
//! ## Einbindung und Abhängigkeiten
//! - **Konsumiert (Ring 0):** `contextra-types` (Domain-Typen), `contextra-ports` (Hardware-Abstraktionen).
//! - **Wird eingebunden von:** `contextra`, `contextra-db`, `contextra-engine`,
//!   `contextra-infer-candle`, `contextra-infer-ollama`, `contextra-infer-onnx`,
//!   `contextra-mcp`, `contextra-py`.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

// ANCHOR[contextra-rank:MOD-001]
pub mod calibration;
pub mod drift;
pub mod explain;
pub mod fusion;

#[cfg(feature = "dibud")]
pub mod dibud;

pub use calibration::{
    AdaptiveConformalCalibrator, ConformalCalibrator, ConformalError, IsotonicCalibrator,
    PlattScaler,
};
#[cfg(feature = "dibud")]
pub use dibud::{
    fuse_exact_prefix, fuse_exact_prefix_async, BudgetedChannel, DiBudFusionState, DiBudOutcome,
    DiBudStep, FusionBudget,
};
pub use drift::{DriftDetector, DriftStatus};
pub use explain::{explain, ExplanationEntry, RetrievalExplanation};
pub use fusion::{
    apply_resonance_bonus, fuse_search_results_with_strategy, g, modulate_and_renormalize_weights,
    reciprocal_rank_fusion, score_normalized_fusion_with_options, weighted_reciprocal_rank_fusion,
    weighted_reciprocal_rank_fusion_mrrf, weighted_reciprocal_rank_fusion_with_options,
    weighted_reciprocal_rank_fusion_with_priority, weights_to_signal_factors, BoundedTopK,
    GlobalFusionConfig, GlobalFusionStrategy, MetadataMergePriority, ProvenanceBuilder,
    ProvenanceRecord, ResonanceConfig, SearchResult, SignalCalibrationContext, SignalContribution,
    SignalKind,
};
