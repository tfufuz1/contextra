// FILE-CONTEXT
// STAND: 2026-09-10T19:16:25Z (SESSION: 3f3e4637)
// ZWECK: Haupt-Routing-Engine für Hybrid-Search-Kontext auf SLM-Profile.
// INVARIANTEN: Atomare Snapshot-Sicherheit bei Hot-Reload, NaN-Safety bei Distanz-Eingaben.
// NICHT-OFFENSICHTLICH: EntityId::from_doc_id Vermeidung von String-Rehashing; Bounded Pending Map.
// SIEHE AUCH: docs/decisions/ADR-020-contextra-brain.md, rules/tag_taxonomy.md

//! Core routing engine for matching hybrid search context to SLM profiles.

use crate::lyapunov::{LyapunovDriftWatcher, LyapunovResult};
use crate::outcome::{DecisionId, DecisionIdGenerator, RoutingOutcome};
use crate::profile::{ProfileCalibrationState, SlmProfile};
use arc_swap::ArcSwap;
use contextra_ports::{
    CommunityResolver, ContextPreparer, DriftStatusProvider as LocalDriftStatusProvider,
    HybridSearchProvider,
};
use contextra_types::{ContextChunk, ContextWindow, ContextraError, EntityId, Result};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[cfg(feature = "bandit-routing")]
use crate::routing_strategy::RoutingStrategy;

mod dispatch_core;
mod lifecycle;
mod outcomes;

#[cfg(test)]
mod tests;

#[cfg(feature = "bandit-routing")]
#[derive(Debug, Clone)]
pub(crate) struct PendingBanditDecision {
    pub context: Vec<f32>,
    pub profile_name: String,
    pub action_idx: u32,
    pub propensity: f32,
    pub created: Instant,
}

#[cfg(feature = "bandit-routing")]
pub(crate) struct BanditExploration {
    pub epsilon: f32,
    pub rng_state: parking_lot::Mutex<u64>,
}

#[cfg(feature = "bandit-routing")]
impl BanditExploration {
    pub fn new(epsilon: f32, seed: u64) -> Self {
        Self {
            epsilon,
            rng_state: parking_lot::Mutex::new(if seed == 0 {
                0x9e37_79b9_7f4a_7c15
            } else {
                seed
            }),
        }
    }

    /// Liefert eine deterministische Pseudo-Zufallszahl in [0.0, 1.0) (SplitMix64).
    pub fn next_sample(&self) -> f32 {
        let mut state = self.rng_state.lock();
        *state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = *state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        let u = z ^ (z >> 31);
        ((u >> 11) as f64 / ((1u64 << 53) as f64)) as f32
    }
}

/// Minimum calibration samples before conformal quantiles are considered reliable.
/// Statistical basis: ≥100 samples required for α=0.1 coverage guarantee per
/// Venn & Gammerman (2005). At 30 samples the quantile interval is too wide
/// to provide meaningful routing signal — the router behaves as a random router.
pub(crate) const CALIBRATION_WARMUP_WINDOW: u32 = 100;

/// Minimum calibration samples required for high confidence coverage (α=0.05).
#[allow(dead_code)]
pub(crate) const CALIBRATION_HIGH_CONFIDENCE_WINDOW: u32 = 200;

/// Maximale TTL für ausstehende Routing-Entscheidungen bevor sie bereinigt werden.
pub(crate) const PENDING_DECISION_TTL: Duration = Duration::from_secs(300);

/// Maximale Kapazität der Map ausstehender Routing-Entscheidungen.
pub(crate) const MAX_PENDING_DECISIONS: usize = 10_000;

/// Calibrated confidence metrics for a routing decision.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfidenceMetrics {
    /// Lower bound of the confidence interval (None when not calibrated).
    pub score_lower: Option<f32>,
    /// Upper bound of the confidence interval (None when not calibrated).
    pub score_upper: Option<f32>,
    /// Whether the score was calibrated via outcome-driven conformal calibration.
    pub calibrated: bool,
    /// Current conformal quantile threshold used for this decision.
    pub quantile_threshold: f32,
    /// Non-conformity score of the decision.
    pub non_conformity_score: f32,
    /// Margin/ratio between best and second best score.
    pub selection_margin: f32,
}

/// Result of a routing operation containing the selected profile, prepared context, confidence, and decision ID.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingDecision {
    /// The target SLM profile selected for the query.
    pub profile: SlmProfile,
    /// The trimmed context window prepared specifically for the selected profile's token budget.
    pub context: ContextWindow,
    /// Calibrated confidence metrics for auditing and cascade control.
    pub confidence: Option<ConfidenceMetrics>,
    /// Eindeutige ID dieser Routing-Entscheidung.
    pub decision_id: DecisionId,
    /// Lyapunov-Drift-Status zum Zeitpunkt der Entscheidung (None wenn InsufficientData).
    pub drift_status: Option<LyapunovResult>,
}

/// Inner state for `RouterEngine` holding active profiles, calibration states, and Lyapunov drift watchers.
///
/// Atomic state swap via `ArcSwap`: profiles, calibration, and watchers
/// are always seen as a consistent unit by all readers.
#[derive(Clone)]
pub struct RouterState {
    pub profiles: Vec<SlmProfile>,
    pub calibration: HashMap<String, ProfileCalibrationState>,
    pub lyapunov_watchers: HashMap<String, LyapunovDriftWatcher>,
}

/// Router engine that routes queries to optimal SLM backends based on community assignment and search scores.
///
/// # Concurrency & Lock Architecture
/// `RouterEngine` utilizes a deliberate dual-state architecture:
/// 1. **`state: ArcSwap<RouterState>`**: Stores active SLM profiles, conformal calibration states,
///    and Lyapunov drift watchers as a single, atomically swapped unit. Readers acquire lock-free
///    snapshots via `state.load()` / `state.load_full()`, ensuring atomic consistency across hot-reloads,
///    routing decisions, and conformal updates without blocking concurrent queries.
/// 2. **`pending_decisions: RwLock<HashMap<DecisionId, (String, Instant)>>`**: Maintained in a separate
///    `parking_lot::RwLock` specifically to decouple transient decision tracking from core router state.
///
/// ### Design Rationale & Safety
/// Keeping `pending_decisions` outside of `RouterState` is a deliberate, audited architectural decision:
/// - **Contention Avoidance**: `pending_decisions` experiences high-frequency write activity (insertions on every
///   routing call, removals on `record_outcome`, periodic pruning in `evict_stale_decisions`). If `pending_decisions`
///   were included inside `RouterState`, every decision track write would require allocating and cloning the entire
///   `RouterState` (profiles, calibration maps, Lyapunov score windows) to perform an `ArcSwap::store` or `rcu`.
/// - **Isolation of Concerns**: Transient pending decision metadata is completely decoupled from statistical
///   calibration and profile routing consistency. Decisions stored in `pending_decisions` are read/written
///   independently per `DecisionId` and do not affect the atomic snapshot guarantees of `RouterState`.
/// - **Concurrency Safety**: This separation presents **zero concurrency risk**. Routing queries read `RouterState`
///   lock-free and briefly acquire a write lock on `pending_decisions` solely to record decision IDs.
///
/// Type-Alias for Backward-Compatibility.
pub type DefaultRouterEngine = RouterEngine;

pub struct RouterEngine {
    pub(crate) search_provider: Arc<dyn HybridSearchProvider>,
    pub(crate) community_resolver: Arc<dyn CommunityResolver>,
    pub(crate) context_preparer: Arc<dyn ContextPreparer>,
    /// Atomic state snapshot via `ArcSwap`: active profiles, conformal calibration, and Lyapunov drift
    /// watchers are maintained together as an immutable, atomically replaceable snapshot.
    pub(crate) state: ArcSwap<RouterState>,
    /// Intentionally kept in a separate `RwLock` outside of `RouterState`.
    ///
    /// **Rationale**: High-frequency writes (decision insertion, `record_outcome` removal, stale eviction)
    /// would trigger full `RouterState` clones if held within `ArcSwap`. Separating `pending_decisions`
    /// eliminates state-cloning overhead while keeping transient outcome tracking isolated from core
    /// routing calibration consistency.
    pub(crate) pending_decisions: RwLock<HashMap<DecisionId, (String, Instant)>>,
    pub(crate) decision_ids: DecisionIdGenerator,
    #[cfg(feature = "bandit-routing")]
    pub(crate) routing_strategy: RoutingStrategy,
    #[cfg(feature = "bandit-routing")]
    pub(crate) bandit_exploration: BanditExploration,
    #[cfg(feature = "bandit-routing")]
    pub(crate) pending_bandit: RwLock<HashMap<DecisionId, PendingBanditDecision>>,
}

pub(crate) const COMMUNITY_RELEVANCE_BOOST: f32 = 1.2;

#[derive(Debug, Clone)]
pub(crate) struct ProfileScoring {
    pub aggregated_score: f32,
    pub max_score: f32,
    pub community_matched: bool,
}

pub(crate) fn score_profile(
    profile: &SlmProfile,
    chunks: &[(ContextChunk, Option<u64>)],
) -> ProfileScoring {
    let mut aggregated_score = 0.0f32;
    let mut max_score = 0.0f32;
    let mut community_matched = profile.domain_communities.is_empty();

    for (chunk, comm_id) in chunks {
        if !chunk.relevance.is_finite() {
            continue;
        }

        let is_match = profile.domain_communities.is_empty()
            || comm_id.is_some_and(|cid| profile.domain_communities.contains(&cid));

        if is_match {
            community_matched = true;
            let boost = if comm_id.is_some_and(|cid| profile.domain_communities.contains(&cid)) {
                COMMUNITY_RELEVANCE_BOOST
            } else {
                1.0
            };
            let boosted_relevance = chunk.relevance * boost;
            aggregated_score += boosted_relevance;
            if boosted_relevance > max_score {
                max_score = boosted_relevance;
            }
        }
    }

    ProfileScoring {
        aggregated_score,
        max_score,
        community_matched,
    }
}

/// Computes the aggregate relevance score for a single profile across chunks.
pub(crate) fn compute_profile_score(
    profile: &SlmProfile,
    chunks: &[(ContextChunk, Option<u64>)],
) -> f32 {
    score_profile(profile, chunks).aggregated_score
}

/// Computes candidate scores for all profiles across chunks.
pub(crate) fn compute_profile_scores(
    profiles: &[SlmProfile],
    chunks: &[(ContextChunk, Option<u64>)],
) -> HashMap<usize, f32> {
    profiles
        .iter()
        .enumerate()
        .map(|(idx, profile)| (idx, compute_profile_score(profile, chunks)))
        .collect()
}

#[allow(dead_code)]
pub(crate) fn compute_max_score(
    profile: &SlmProfile,
    chunks: &[(ContextChunk, Option<u64>)],
) -> f32 {
    score_profile(profile, chunks).max_score
}

#[allow(dead_code)]
pub(crate) fn select_profile_from_chunks(
    profiles: &[SlmProfile],
    chunks: &[(ContextChunk, Option<u64>)],
) -> Result<usize> {
    if chunks.is_empty() {
        return Err(ContextraError::NotFound(
            "Keine gültigen Chunks aus Suchergebnissen ermittelbar".to_string(),
        ));
    }

    if !chunks.iter().any(|(c, _)| c.relevance.is_finite()) {
        tracing::error!(
            "Alle Chunk-Relevanzwerte sind NaN/Inf — mögliche Upstream-Korruption in der Distanzberechnung"
        );
        return Err(ContextraError::NotFound(
            "Alle Chunk-Relevanzwerte sind NaN/Inf — mögliche Upstream-Korruption in der Distanzberechnung".to_string(),
        ));
    }

    let mut profile_scores: HashMap<usize, f32> = HashMap::new();
    let mut any_community_matched = false;

    for (idx, profile) in profiles.iter().enumerate() {
        let scoring = score_profile(profile, chunks);
        if scoring.community_matched {
            any_community_matched = true;
        }

        if scoring.community_matched && scoring.aggregated_score >= profile.min_relevance_score {
            profile_scores.insert(idx, scoring.aggregated_score);
        }
    }

    let best_profile_idx = profile_scores
        .into_iter()
        .max_by(|(idx_a, score_a), (idx_b, score_b)| {
            score_a.total_cmp(score_b).then_with(|| idx_b.cmp(idx_a))
        })
        .map(|(idx, _)| idx);

    match best_profile_idx {
        Some(idx) => Ok(idx),
        None => {
            if any_community_matched {
                Err(ContextraError::NotFound(
                    "Kein SLM-Profil erreicht den erforderlichen min_relevance_score".to_string(),
                ))
            } else {
                Err(ContextraError::NotFound(
                    "Kein SLM-Profil entspricht der Community-Zuordnung".to_string(),
                ))
            }
        }
    }
}

impl LocalDriftStatusProvider for RouterEngine {
    fn overall_drift_status(&self) -> String {
        self.overall_drift_status()
    }
}
