//! # Contextra Rank (`contextra-rank`)
//!
//! **Ring:** Ring 2 (Ranking & Fusion)
//! **Klasse:** Rand `[R]`
//! **Reifegrad:** Aktiv (Kern v1, SPEC §8.13, D-26)
//!
//! ## Zweck des Crates
//! Dieses Crate implementiert die deterministische Reciprocal Rank Fusion (RRF)
//! für Hybridsuche im Contextra-System:
//! - Kanonische Schnittstelle: [`rrf_fuse`]
//! - Standard-Glättungsfaktor $k = 60$
//! - Deterministischer Tie-Break (primär Score absteigend, sekundär `doc_tag` / `id` aufsteigend)
//! - Permutationsinvarianz bezüglich der Listenreihenfolge
//! - Keine Hintergrund-Threads, kein `unsafe` (`#![forbid(unsafe_code)]`).

#![forbid(unsafe_code)]
#![warn(missing_docs)]

// Begründung: Veraltete Wrapper bleiben für Abwärtskompatibilität zu contextra-engine erhalten
// bis diese umgestellt ist (siehe ÜBERGABE).
#[allow(deprecated)]
pub use fusion::{
    fuse_search_results_with_signal_strategies, fuse_search_results_with_strategy,
    reciprocal_rank_fusion, weighted_reciprocal_rank_fusion, weighted_reciprocal_rank_fusion_mrrf,
    weighted_reciprocal_rank_fusion_with_options, weighted_reciprocal_rank_fusion_with_priority,
};

// ANCHOR[contextra-rank:MOD-001]
pub mod explain;
pub mod fusion;

pub use explain::{explain, ExplanationEntry, RetrievalExplanation};
pub use fusion::{
    apply_resonance_bonus, fuse_signals, g, modulate_and_renormalize_weights, rrf_fuse,
    score_normalized_fusion_with_options, weights_to_signal_factors, BoundedTopK, Hit,
    MetadataMergePriority, ProvenanceBuilder, ProvenanceRecord, ResonanceConfig, SearchReport,
    SearchResult, Signal, SignalContribution, SignalFailurePolicy, SignalKind, DEFAULT_RRF_K,
};
