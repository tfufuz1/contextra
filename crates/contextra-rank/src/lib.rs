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
//!
//! ## Minimalbeispiel (RRF-Doctest mit handgerechnetem Wert)
//! ```rust
//! use contextra_rank::fusion::{rrf_fuse, ScoredItem};
//!
//! // Liste 1 (Text):   [doc_1 (Rang 1), doc_2 (Rang 2)]
//! // Liste 2 (Vektor): [doc_2 (Rang 1), doc_1 (Rang 2)]
//! let text = vec![ScoredItem::new("doc_1", 10.0), ScoredItem::new("doc_2", 5.0)];
//! let vector = vec![ScoredItem::new("doc_2", 0.9), ScoredItem::new("doc_1", 0.8)];
//!
//! // Mit k = 60:
//! // doc_1: 1/(60+1) + 1/(60+2) = 1/61 + 1/62 ≈ 0.032522475
//! // doc_2: 1/(60+2) + 1/(60+1) = 1/62 + 1/61 ≈ 0.032522475
//! // Tie-Break: "doc_1" < "doc_2" (lexikografisch aufsteigend)
//! let fused = rrf_fuse(&[text, vector], 60).unwrap_or_default();
//! assert_eq!(fused.len(), 2);
//! assert_eq!(fused[0].id, "doc_1");
//! assert_eq!(fused[1].id, "doc_2");
//! let expected = (1.0 / 61.0_f32) + (1.0 / 62.0_f32);
//! assert!((fused[0].score - expected).abs() < 1e-6);
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#[cfg(feature = "workspace")]
pub mod compat;
pub mod calibration;
pub mod contract;
pub mod drift;
pub mod error;
pub mod explain;
pub mod fusion;
pub mod local_types;
pub mod metrics;
pub mod temporal;

pub use error::{RankError, Result};
pub use local_types::{
    ConfigFingerprint, DocId, FusionStrategy, FusionWeights, SignalFailurePolicy,
    SignalFusionStrategies, MAX_SEARCH_K,
};

#[cfg(feature = "dibud")]
pub mod dibud;

pub use calibration::{
    calibrate_score, AdaptiveConformalCalibrator, Calibrator, Clock, ConformalCalibrator,
    ConformalError, IsotonicCalibrator, PlattScaler, SystemClock,
};
#[cfg(feature = "dibud")]
pub use dibud::{
    fuse_exact_prefix, fuse_exact_prefix_async, BudgetedChannel, DiBudFusionState, DiBudOutcome,
    DiBudStep, FusionBudget,
};
pub use drift::{DriftDetector, DriftStatus};
pub use explain::{explain, ExplanationEntry, RetrievalExplanation};
pub use fusion::{
    fuse_search_results_with_strategy, g, modulate_and_renormalize_weights,
    reciprocal_rank_fusion, rrf_fuse, score_normalized_fusion_with_options,
    weighted_reciprocal_rank_fusion, weighted_reciprocal_rank_fusion_mrrf,
    weighted_reciprocal_rank_fusion_with_options, weighted_reciprocal_rank_fusion_with_priority,
    weights_to_signal_factors, BoundedTopK, GlobalFusionConfig, GlobalFusionStrategy,
    MetadataMergePriority, ProvenanceBuilder, ProvenanceRecord, ScoredItem,
    SearchResult, SignalCalibrationContext, SignalContribution, SignalKind,
};
pub use metrics::{mean_reciprocal_rank, ndcg_at_k, precision_at_k, recall_at_k, reciprocal_rank};

pub use temporal::{
    apply_temporal_validity_filter, apply_temporal_validity_filter_at,
    apply_temporal_validity_filter_with_clock, extract_validity_window,
    sort_candidates_deterministic, FusionResult, ValidityWindow,
};
#[cfg(feature = "exp-resonance")]
pub use fusion::{apply_resonance_bonus, fuse_signals, ResonanceConfig};
pub use fusion::ResonanceModifier;
