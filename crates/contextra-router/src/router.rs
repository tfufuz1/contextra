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
    search_provider: Arc<dyn HybridSearchProvider>,
    community_resolver: Arc<dyn CommunityResolver>,
    context_preparer: Arc<dyn ContextPreparer>,
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

impl RouterEngine {
    /// Creates a new `RouterEngine` instance from decoupled ports.
    pub fn new(
        search_provider: Arc<dyn HybridSearchProvider>,
        community_resolver: Arc<dyn CommunityResolver>,
        context_preparer: Arc<dyn ContextPreparer>,
        profiles: Vec<SlmProfile>,
        calibration_store_path: Option<std::path::PathBuf>,
    ) -> Self {
        let mut calibration: HashMap<String, ProfileCalibrationState> = profiles
            .iter()
            .map(|p| {
                (
                    p.name.clone(),
                    ProfileCalibrationState::new(p.min_relevance_score),
                )
            })
            .collect();

        let lyapunov_watchers: HashMap<String, LyapunovDriftWatcher> = profiles
            .iter()
            .map(|p| (p.name.clone(), LyapunovDriftWatcher::default()))
            .collect();

        if let Some(ref path) = calibration_store_path {
            if let Ok(bytes) = std::fs::read(path) {
                if let Ok(persisted) =
                    serde_json::from_slice::<HashMap<String, ProfileCalibrationState>>(&bytes)
                {
                    // Merge persisted state into defaults (persisted wins for known profiles)
                    for (name, state) in persisted {
                        if calibration.contains_key(&name) {
                            calibration.insert(name, state);
                        }
                        // Unknown profiles (removed from config) are silently dropped
                    }
                }
            }
        }

        let router_state = RouterState {
            profiles,
            calibration,
            lyapunov_watchers,
        };

        Self {
            search_provider,
            community_resolver,
            context_preparer,
            state: ArcSwap::from(Arc::new(router_state)),
            pending_decisions: RwLock::new(HashMap::new()),
            decision_ids: DecisionIdGenerator::new(0),
            #[cfg(feature = "bandit-routing")]
            routing_strategy: RoutingStrategy::Cascade,
            #[cfg(feature = "bandit-routing")]
            bandit_exploration: BanditExploration::new(0.1, 0x1234_5678_9abc_def0),
            #[cfg(feature = "bandit-routing")]
            pending_bandit: RwLock::new(HashMap::new()),
        }
    }

    /// Konfiguriert den Startwert des instanzgebundenen DecisionIdGenerators.
    pub fn with_initial_decision_id(mut self, start: u64) -> Self {
        self.decision_ids = DecisionIdGenerator::new(start);
        self
    }

    #[cfg(feature = "bandit-routing")]
    /// Builder-Methode zur Konfiguration der Routing-Strategie und Bandit-Exploration.
    pub fn with_routing_strategy(
        mut self,
        strategy: RoutingStrategy,
        epsilon: f32,
        seed: u64,
    ) -> Self {
        self.routing_strategy = strategy;
        self.bandit_exploration = BanditExploration::new(epsilon, seed);
        self
    }

    #[cfg(feature = "bandit-routing")]
    /// Liefert die aufgezeichnete Logging-Propensity für eine ausstehende Bandit-Entscheidung.
    pub fn bandit_decision_propensity(&self, id: DecisionId) -> Option<f32> {
        self.pending_bandit.read().get(&id).map(|d| d.propensity)
    }

    /// Validates all profiles and creates a new `RouterEngine` instance.
    pub fn try_new(
        search_provider: Arc<dyn HybridSearchProvider>,
        community_resolver: Arc<dyn CommunityResolver>,
        context_preparer: Arc<dyn ContextPreparer>,
        profiles: Vec<SlmProfile>,
        calibration_store_path: Option<std::path::PathBuf>,
    ) -> Result<Self> {
        for p in &profiles {
            p.validate()?;
        }
        Ok(Self::new(
            search_provider,
            community_resolver,
            context_preparer,
            profiles,
            calibration_store_path,
        ))
    }

    /// Dynamically updates configured SLM profiles at runtime (Hot-Reload).
    pub fn update_profiles(&self, new_profiles: Vec<SlmProfile>) {
        let current = self.state.load_full();
        let mut old_cal = current.calibration.clone();
        let new_cal: HashMap<String, ProfileCalibrationState> = new_profiles
            .iter()
            .map(|p| {
                let mut state = old_cal
                    .remove(&p.name)
                    .unwrap_or_else(|| ProfileCalibrationState::new(p.min_relevance_score));
                state.check_and_invalidate_fingerprint(p.fingerprint.as_ref());
                (p.name.clone(), state)
            })
            .collect();

        let mut old_watchers = current.lyapunov_watchers.clone();
        let new_watchers: HashMap<String, LyapunovDriftWatcher> = new_profiles
            .iter()
            .map(|p| {
                let watcher = old_watchers.remove(&p.name).unwrap_or_default();
                (p.name.clone(), watcher)
            })
            .collect();

        let new_state = RouterState {
            profiles: new_profiles,
            calibration: new_cal,
            lyapunov_watchers: new_watchers,
        };

        self.state.store(Arc::new(new_state));
    }

    /// Validates all profiles and updates configured SLM profiles at runtime (Hot-Reload).
    pub fn try_update_profiles(&self, new_profiles: Vec<SlmProfile>) -> Result<()> {
        for p in &new_profiles {
            p.validate()?;
        }
        self.update_profiles(new_profiles);
        Ok(())
    }

    /// Returns a copy of the active SLM profiles.
    pub fn profiles(&self) -> Vec<SlmProfile> {
        self.state.load().profiles.clone()
    }

    /// Gibt aktuelle Kalibrierungsstatistik für alle Profile zurück.
    pub fn calibration_stats(&self) -> HashMap<String, ProfileCalibrationState> {
        self.state.load().calibration.clone()
    }

    /// Setzt Kalibrierungsstatistik für ein bestimmtes Profil zurück.
    pub fn reset_calibration(&self, profile_name: &str) {
        let current = self.state.load_full();
        if current.calibration.contains_key(profile_name) {
            let mut new_state = (*current).clone();
            if let Some(state) = new_state.calibration.get_mut(profile_name) {
                state.reset();
            }
            self.state.store(Arc::new(new_state));
        }
    }

    /// Gibt den aktuellen Lyapunov-Drift-Status für ein Profil zurück.
    pub fn drift_status(&self, profile_name: &str) -> Option<LyapunovResult> {
        self.state
            .load()
            .lyapunov_watchers
            .get(profile_name)
            .and_then(|w| w.latest_result.clone())
    }

    /// Gives a human-readable summary string of Lyapunov drift status across active profile watchers.
    /// Priority order: "kritisch" > "warnung" > "stabil" > "unbekannt".
    pub fn overall_drift_status(&self) -> String {
        let state = self.state.load();
        if state.lyapunov_watchers.is_empty() {
            return "stabil".to_string();
        }
        let mut has_warning = false;
        let mut has_stable = false;
        for watcher in state.lyapunov_watchers.values() {
            match watcher.status_str() {
                "kritisch" => return "kritisch".to_string(),
                "warnung" => has_warning = true,
                "stabil" => has_stable = true,
                _ => {}
            }
        }
        if has_warning {
            "warnung".to_string()
        } else if has_stable {
            "stabil".to_string()
        } else {
            "unbekannt".to_string()
        }
    }

    /// Setzt die Baseline für den Lyapunov-Drift-Wächter eines bestimmten Profils.
    pub fn set_lyapunov_baseline(&self, profile_name: &str, baseline: &[f32]) -> bool {
        let current = self.state.load_full();
        if current.lyapunov_watchers.contains_key(profile_name) {
            let mut new_state = (*current).clone();
            if let Some(watcher) = new_state.lyapunov_watchers.get_mut(profile_name) {
                watcher.set_baseline(baseline);
            }
            self.state.store(Arc::new(new_state));
            true
        } else {
            false
        }
    }

    fn evict_stale_decisions(&self) {
        let now = Instant::now();
        let cutoff = now.checked_sub(PENDING_DECISION_TTL);
        let mut map = self.pending_decisions.write();
        if map.len() >= MAX_PENDING_DECISIONS {
            if let Some(cutoff) = cutoff {
                map.retain(|_, (_, ts)| *ts > cutoff);
            }
        }

        #[cfg(feature = "bandit-routing")]
        {
            let mut bandit_map = self.pending_bandit.write();
            if bandit_map.len() >= MAX_PENDING_DECISIONS {
                if let Some(cutoff) = cutoff {
                    bandit_map.retain(|_, decision| decision.created > cutoff);
                }
            }
        }
    }

    /// Muss vom Aufrufer (Agent-Loop) nach Abschluss des SLM-Aufrufs aufgerufen werden.
    /// Liefert das tatsächliche Ergebnis zurück und trainiert die Kalibrierung
    /// mit einem echten Ground-Truth-Signal.
    ///
    /// Gibt true zurück wenn die Decision gefunden und verarbeitet wurde,
    /// false wenn die DecisionId unbekannt ist (z.B. nach Restart).
    pub fn record_outcome(&self, decision_id: DecisionId, outcome: RoutingOutcome) -> bool {
        let profile_name = match self.pending_decisions.write().remove(&decision_id) {
            Some((name, _ts)) => name,
            None => {
                tracing::warn!(
                    ?decision_id,
                    "record_outcome: unbekannte DecisionId ignoriert"
                );
                return false;
            }
        };

        let current = self.state.load_full();
        let active_fp = current
            .profiles
            .iter()
            .find(|p| p.name == profile_name)
            .and_then(|p| p.fingerprint.clone());

        let non_conformity = outcome.non_conformity_score();

        let mut new_state = (*current).clone();
        if let Some(state) = new_state.calibration.get_mut(&profile_name) {
            state.check_and_invalidate_fingerprint(active_fp.as_ref());
            if active_fp.is_some() {
                state.recalibrate_conformal(non_conformity);
                tracing::debug!(
                    profile = %profile_name,
                    ?outcome,
                    non_conformity,
                    "Router outcome recorded"
                );
            }
        }

        #[cfg(feature = "bandit-routing")]
        {
            if let Some(pending_bandit_entry) = self.pending_bandit.write().remove(&decision_id) {
                let reward = 1.0 - non_conformity;
                if let Some(profile) = new_state
                    .profiles
                    .iter_mut()
                    .find(|p| p.name == profile_name)
                {
                    #[cfg(feature = "cloud-egress-guard")]
                    let is_cloud = profile.transport.is_cloud();
                    #[cfg(not(feature = "cloud-egress-guard"))]
                    let is_cloud = false;

                    let cost = profile.estimated_cost();
                    if let Some(ref mut bstate) = profile.bandit_state {
                        if let Err(err) =
                            bstate.update(&pending_bandit_entry.context, reward, cost, is_cloud)
                        {
                            tracing::warn!(
                                profile = %pending_bandit_entry.profile_name,
                                action = pending_bandit_entry.action_idx,
                                ?err,
                                "Fehler beim BanditProfileState-Update in record_outcome"
                            );
                        } else {
                            tracing::debug!(
                                profile = %pending_bandit_entry.profile_name,
                                action = pending_bandit_entry.action_idx,
                                propensity = pending_bandit_entry.propensity,
                                reward,
                                cost,
                                "BanditProfileState erfolgreich aktualisiert"
                            );
                        }
                    }
                }
            }
        }

        self.state.store(Arc::new(new_state));
        true
    }

    /// Anzahl offener (noch nicht mit record_outcome() abgeschlossener) Decisions.
    /// Sollte in normaler Laufzeit nahe 0 bleiben.
    pub fn pending_decision_count(&self) -> usize {
        self.pending_decisions.read().len()
    }

    /// Setzt Kalibrierungsstatistik für alle Profile zurück.
    pub fn reset_all_calibration(&self) {
        let current = self.state.load_full();
        let mut new_state = (*current).clone();
        for state in new_state.calibration.values_mut() {
            state.reset();
        }
        self.state.store(Arc::new(new_state));
    }

    /// Routes a query with embedding and text to the best matching SLM profile.
    #[allow(deprecated)]
    pub async fn route(
        &self,
        query_embedding: &[f32],
        query_text: &str,
    ) -> Result<RoutingDecision> {
        self.evict_stale_decisions();

        if query_embedding.iter().any(|v| !v.is_finite()) {
            return Err(ContextraError::InvalidInput(
                "query_embedding contains non-finite values (NaN/Inf)".to_string(),
            ));
        }

        // Snapshot state atomically via ArcSwap to guarantee caller consistency during hot-reloads
        let state_snap = self.state.load_full();
        let profiles = state_snap.profiles.clone();

        if profiles.is_empty() {
            return Err(ContextraError::NotFound(
                "Keine SLM-Profile für Routing konfiguriert".to_string(),
            ));
        }

        // 1. Perform hybrid search with standard fusion weights
        let search_chunks = self
            .search_provider
            .search_hybrid(query_text, query_embedding, 10)
            .await?;

        if search_chunks.is_empty() {
            return Err(ContextraError::NotFound(
                "Keine relevanten Suchergebnisse für Routing gefunden".to_string(),
            ));
        }

        // 2. Identify communities and score candidate profiles
        let mut chunks: Vec<(ContextChunk, Option<u64>)> = Vec::new();

        for chunk in search_chunks {
            let eid = EntityId::from_doc_id(chunk.doc_id);
            let comm_id = self
                .community_resolver
                .get_community(eid)
                .await
                .ok()
                .flatten();
            chunks.push((chunk, comm_id));
        }

        // 3. Perform profile selection, scoring, calibration tracking, and confidence metric generation
        // using an updated state snapshot swapped atomically via ArcSwap.
        let current_state = self.state.load_full();
        let mut new_state = (*current_state).clone();

        let (selected_profile, confidence_metrics, _is_bandit_decision) = {
            let cal = &mut new_state.calibration;

            // 1. Derive effective profiles using calibrated_min_score from calibration state
            let effective_profiles: Vec<SlmProfile> = profiles
                .iter()
                .map(|p| {
                    let mut ep = p.clone();
                    if let Some(state) = cal.get_mut(&p.name) {
                        state.check_and_invalidate_fingerprint(p.fingerprint.as_ref());
                        if state.is_calibrated(p.fingerprint.as_ref()) {
                            ep.min_relevance_score = state.calibrated_min_score;
                        }
                    }
                    ep
                })
                .collect();

            // 2. Profile Selection: Bandit (opt-in) or Cascade (default)
            #[cfg(feature = "bandit-routing")]
            let bandit_selection =
                if matches!(self.routing_strategy, RoutingStrategy::ContextualBandit) {
                    self.select_profile_bandit(&chunks, &effective_profiles, query_embedding)
                } else {
                    None
                };
            #[cfg(not(feature = "bandit-routing"))]
            let bandit_selection: Option<(usize, SlmProfile, u32, f32)> = None;

            let (selected_idx, selected_profile, is_bandit_decision) =
                if let Some((idx, profile, action_idx, propensity)) = bandit_selection {
                    (idx, profile, Some((action_idx, propensity)))
                } else {
                    let (idx, profile, _) =
                        self.select_profile_cascade(&chunks, &effective_profiles, cal)?;
                    (idx, profile, None)
                };

            let profile_scores = compute_profile_scores(&profiles, &chunks);
            let best_score = profile_scores.get(&selected_idx).copied().unwrap_or(0.0);
            let second_best = profile_scores
                .iter()
                .filter(|(idx, _)| **idx != selected_idx)
                .map(|(_, s)| *s)
                .fold(0.0f32, f32::max);

            let confidence_ratio = if second_best > 0.0 {
                (best_score / second_best) as f64
            } else {
                2.0 // Single candidate -> high confidence
            };

            let non_conformity = if best_score > 0.0 {
                (1.0 / confidence_ratio as f32).clamp(0.0, 1.0)
            } else {
                1.0
            };

            if let Some(state) = cal.get_mut(&selected_profile.name) {
                state.times_selected += 1;
                state.cumulative_confidence += confidence_ratio;
            }

            // 4. Construct ConfidenceMetrics from updated state
            let metrics = cal.get(&selected_profile.name).map(|state| {
                let calibrated = state.conformal.window_total >= CALIBRATION_WARMUP_WINDOW as u64;
                if !calibrated {
                    tracing::info!(
                        profile = %selected_profile.name,
                        samples = state.conformal.window_total,
                        required = CALIBRATION_WARMUP_WINDOW,
                        "Router in random-fallback mode: calibration not yet reliable"
                    );
                }
                ConfidenceMetrics {
                    score_lower: if calibrated {
                        Some(best_score * (1.0 - state.conformal.alpha))
                    } else {
                        None
                    },
                    score_upper: if calibrated {
                        Some(best_score * (1.0 + state.conformal.alpha))
                    } else {
                        None
                    },
                    calibrated,
                    quantile_threshold: state.conformal.quantile_threshold,
                    non_conformity_score: non_conformity,
                    selection_margin: confidence_ratio as f32,
                }
            });

            (selected_profile, metrics, is_bandit_decision)
        };

        let decision_id = self.decision_ids.next();
        self.pending_decisions
            .write()
            .insert(decision_id, (selected_profile.name.clone(), Instant::now()));

        #[cfg(feature = "bandit-routing")]
        if let Some((action_idx, propensity)) = _is_bandit_decision {
            self.pending_bandit.write().insert(
                decision_id,
                PendingBanditDecision {
                    context: query_embedding.to_vec(),
                    profile_name: selected_profile.name.clone(),
                    action_idx,
                    propensity,
                    created: Instant::now(),
                },
            );
        }

        // 4. Construct ContextWindow using ContextManager tailored to selected_profile.token_budget and min_relevance_score
        let raw_chunks: Vec<ContextChunk> = chunks.into_iter().map(|(c, _)| c).collect();

        // 5. Update Lyapunov Drift Watcher with non-conformity score
        let non_conformity_score = confidence_metrics
            .as_ref()
            .map(|m| m.non_conformity_score)
            .unwrap_or(1.0);

        let drift_status = {
            let watchers = &mut new_state.lyapunov_watchers;
            if let Some(watcher) = watchers.get_mut(&selected_profile.name) {
                watcher.observe_score(non_conformity_score);
                let res = watcher.analyze();

                if let LyapunovResult::DriftDetected {
                    lyapunov_exponent,
                    ref reason,
                } = res
                {
                    tracing::warn!(
                        profile = %selected_profile.name,
                        lambda = lyapunov_exponent,
                        kl_divergence = reason.kl_divergence,
                        "Lyapunov drift detected — conformal calibration may be stale"
                    );

                    #[cfg(feature = "bandit-routing")]
                    {
                        let k_drift = (1.0 + lyapunov_exponent.max(0.0)).clamp(1.5, 4.0);
                        if let Some(profile_state) = new_state
                            .profiles
                            .iter_mut()
                            .find(|p| p.name == selected_profile.name)
                        {
                            if let Some(ref mut bstate) = profile_state.bandit_state {
                                bstate.on_drift_detected(k_drift);
                            }
                        }
                    }
                }

                match res {
                    LyapunovResult::InsufficientData => None,
                    status => Some(status),
                }
            } else {
                None
            }
        };

        // Store updated state atomically via ArcSwap
        self.state.store(Arc::new(new_state));

        let context_window = self.context_preparer.prepare_context(
            raw_chunks,
            &selected_profile.token_budget,
            selected_profile.min_relevance_score,
        )?;

        Ok(RoutingDecision {
            profile: selected_profile,
            context: context_window,
            confidence: confidence_metrics,
            decision_id,
            drift_status,
        })
    }

    /// Kalibriertes Kaskaden-Routing.
    ///
    /// Algorithmus:
    /// 1. Sortiere Profile absteigend nach min_relevance_score (präzisestes zuerst).
    /// 2. Für jedes Profil in dieser Reihenfolge:
    ///    - Berechne Aggregat-Score der Chunks (existing logic)
    ///    - Hole ConformalCalibrator für dieses Profil aus self.calibration
    ///    - Prüfe: score >= calibrator.quantile_threshold (oder profile.min_relevance_score)
    ///      JA: Dieses Profil nehmen, ConfidenceMetrics::Calibrated
    ///      NEIN: Weiter zum nächsten Profil (Kaskade)
    /// 3. Falls kein Profil den kalibrierten Schwellenwert erfüllt:
    ///    - Nehme das letzte (geringstes min_relevance_score) als sicheren Fallback
    ///    - ConfidenceMetrics::Uncalibrated, tracing::warn! ausgeben
    ///
    /// # Returns
    /// (profil_index, SlmProfile, ConfidenceMetrics)
    pub(crate) fn select_profile_cascade(
        &self,
        chunks: &[(ContextChunk, Option<u64>)],
        profiles: &[SlmProfile],
        calibration: &mut HashMap<String, ProfileCalibrationState>,
    ) -> Result<(usize, SlmProfile, ConfidenceMetrics)> {
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

        if profiles.is_empty() {
            return Err(ContextraError::NotFound(
                "Keine SLM-Profile konfiguriert".to_string(),
            ));
        }

        // Filter profiles by community match eligibility.
        // A profile is eligible if its domain_communities is empty, OR if at least one chunk matches one of its domain_communities.
        let eligible_profiles: Vec<(usize, &SlmProfile)> = profiles
            .iter()
            .enumerate()
            .filter(|(_, profile)| {
                profile.domain_communities.is_empty()
                    || chunks.iter().any(|(_, comm_id)| {
                        comm_id.is_some_and(|cid| profile.domain_communities.contains(&cid))
                    })
            })
            .collect();

        if eligible_profiles.is_empty() {
            return Err(ContextraError::NotFound(
                "Kein SLM-Profil entspricht der Community-Zuordnung".to_string(),
            ));
        }

        // 1. Sort eligible profile indices descending by min_relevance_score (most precise first).
        // Tie-breaking: when min_relevance_scores are equal, candidate score descending, then lower original index.
        let mut sorted_profiles = eligible_profiles.clone();
        sorted_profiles.sort_by(|(idx_a, a), (idx_b, b)| {
            b.min_relevance_score
                .total_cmp(&a.min_relevance_score)
                .then_with(|| {
                    let score_a = compute_profile_score(a, chunks);
                    let score_b = compute_profile_score(b, chunks);
                    score_b.total_cmp(&score_a).then_with(|| idx_a.cmp(idx_b))
                })
        });

        // 2. Cascade evaluation in descending min_relevance_score order
        for &(orig_idx, profile) in &sorted_profiles {
            let score = compute_profile_score(profile, chunks);
            let state = calibration.get_mut(&profile.name);

            let (threshold, is_calibrated) = match state {
                Some(st) => {
                    st.check_and_invalidate_fingerprint(profile.fingerprint.as_ref());
                    if st.is_calibrated(profile.fingerprint.as_ref()) {
                        (st.calibrated_min_score, true)
                    } else {
                        (profile.min_relevance_score, false)
                    }
                }
                None => (profile.min_relevance_score, false),
            };

            if score >= threshold {
                let (quantile, alpha) = match calibration.get(&profile.name) {
                    Some(st) => (st.conformal.quantile_threshold, st.conformal.alpha),
                    None => (profile.min_relevance_score, 0.05),
                };
                let non_conformity = (1.0 - (score / quantile.max(f32::EPSILON))).clamp(0.0, 1.0);
                let selection_margin = if quantile > 0.0 {
                    score / quantile
                } else {
                    1.0
                };

                let confidence = ConfidenceMetrics {
                    score_lower: if is_calibrated {
                        Some(score * (1.0 - alpha))
                    } else {
                        None
                    },
                    score_upper: if is_calibrated {
                        Some(score * (1.0 + alpha))
                    } else {
                        None
                    },
                    calibrated: is_calibrated,
                    quantile_threshold: quantile,
                    non_conformity_score: non_conformity,
                    selection_margin,
                };
                return Ok((orig_idx, profile.clone(), confidence));
            }
        }

        // Während der Warmup-Periode (calibrated == false) wird bewusst konservativ geroutet:
        // das ressourcenschonendste Profil wird gewählt, um Kostenrisiken bei fehlender
        // statistischer Absicherung zu minimieren.
        let &(fallback_idx, fallback_profile) =
            match eligible_profiles.iter().min_by(|(idx_a, a), (idx_b, b)| {
                a.estimated_cost()
                    .total_cmp(&b.estimated_cost())
                    .then_with(|| a.min_relevance_score.total_cmp(&b.min_relevance_score))
                    .then_with(|| idx_a.cmp(idx_b))
            }) {
                Some(p) => p,
                None => {
                    return Err(ContextraError::NotFound(
                        "Keine SLM-Profile konfiguriert".to_string(),
                    ));
                }
            };
        let fallback_score = compute_profile_score(fallback_profile, chunks);
        let state = calibration.get(&fallback_profile.name);
        let (q_threshold, alpha, is_calibrated) = match state {
            Some(st) => (
                st.conformal.quantile_threshold,
                st.conformal.alpha,
                st.is_calibrated(fallback_profile.fingerprint.as_ref()),
            ),
            None => (fallback_profile.min_relevance_score, 0.05, false),
        };
        let non_conformity =
            (1.0 - (fallback_score / q_threshold.max(f32::EPSILON))).clamp(0.0, 1.0);
        let selection_margin = if q_threshold > 0.0 {
            fallback_score / q_threshold
        } else {
            1.0
        };

        tracing::warn!(
            profile = %fallback_profile.name,
            "Kaskaden-Fallback: Kein Profil über Schwellenwert, nutze Profil mit niedrigstem min_relevance_score"
        );

        let confidence = ConfidenceMetrics {
            score_lower: if is_calibrated {
                Some(fallback_score * (1.0 - alpha))
            } else {
                None
            },
            score_upper: if is_calibrated {
                Some(fallback_score * (1.0 + alpha))
            } else {
                None
            },
            calibrated: is_calibrated,
            quantile_threshold: q_threshold,
            non_conformity_score: non_conformity,
            selection_margin,
        };

        Ok((fallback_idx, fallback_profile.clone(), confidence))
    }

    #[cfg(feature = "bandit-routing")]
    /// Contextual Bandit Profilauswahl via LinUCB / Sherman-Morrison (§13.2, §8.5, AK-15).
    ///
    /// Algorithmus:
    /// 1. Filter eligible profiles based on community matching (identical eligibility to cascade).
    /// 2. For each eligible profile, evaluate bandit_state.score(x = query_embedding, cost = estimated_cost(), is_cloud).
    /// 3. Fail-safe: If any eligible profile has bandit_state == None or returns BanditError/NaN,
    ///    issue tracing::warn! and return None (falling back to Cascade without panic).
    /// 4. Greedy choice = profile with highest score (total_cmp, tie-breaker smallest original index).
    /// 5. Randomized logging policy with clamped epsilon (max(epsilon, 0.01 * K).min(1.0)) for propensity >= 0.01 guarantee.
    /// 6. Return Some((selected_orig_idx, selected_profile, action_idx, propensity)).
    pub(crate) fn select_profile_bandit(
        &self,
        chunks: &[(ContextChunk, Option<u64>)],
        profiles: &[SlmProfile],
        query_embedding: &[f32],
    ) -> Option<(usize, SlmProfile, u32, f32)> {
        use contextra_adapt::offpolicy::RandomizedLoggingPolicy;

        if profiles.is_empty() || chunks.is_empty() {
            return None;
        }

        // Filter profiles by community match eligibility (matching select_profile_cascade logic)
        let eligible_profiles: Vec<(usize, &SlmProfile)> = profiles
            .iter()
            .enumerate()
            .filter(|(_, profile)| {
                profile.domain_communities.is_empty()
                    || chunks.iter().any(|(_, comm_id)| {
                        comm_id.is_some_and(|cid| profile.domain_communities.contains(&cid))
                    })
            })
            .collect();

        if eligible_profiles.is_empty() {
            return None;
        }

        let num_actions = eligible_profiles.len() as u32;

        // Evaluate scores for each eligible profile
        let mut profile_scores: Vec<(usize, &SlmProfile, f32)> =
            Vec::with_capacity(eligible_profiles.len());

        for &(orig_idx, profile) in &eligible_profiles {
            let Some(ref bstate) = profile.bandit_state else {
                tracing::warn!(
                    profile = %profile.name,
                    "Bandit-Dispatch Fallback auf Cascade: Profil besitzt keinen bandit_state"
                );
                return None;
            };

            #[cfg(feature = "cloud-egress-guard")]
            let is_cloud = profile.transport.is_cloud();
            #[cfg(not(feature = "cloud-egress-guard"))]
            let is_cloud = false;

            let cost = profile.estimated_cost();
            match bstate.score(query_embedding, cost, is_cloud) {
                Ok(score) if score.is_finite() => {
                    profile_scores.push((orig_idx, profile, score));
                }
                Ok(non_finite) => {
                    tracing::warn!(
                        profile = %profile.name,
                        score = non_finite,
                        "Bandit-Dispatch Fallback auf Cascade: UCB-Score ist nicht-finit (NaN/Inf)"
                    );
                    return None;
                }
                Err(err) => {
                    tracing::warn!(
                        profile = %profile.name,
                        ?err,
                        "Bandit-Dispatch Fallback auf Cascade: BanditError bei Score-Berechnung"
                    );
                    return None;
                }
            }
        }

        // Greedy choice: action index in 0..num_actions with highest score
        let greedy_action_idx = profile_scores
            .iter()
            .enumerate()
            .max_by(
                |(_, (orig_idx_a, _, score_a)), (_, (orig_idx_b, _, score_b))| {
                    score_a
                        .total_cmp(score_b)
                        .then_with(|| orig_idx_b.cmp(orig_idx_a))
                },
            )
            .map(|(action_idx, _)| action_idx as u32)
            .unwrap_or(0);

        // Clamp epsilon to guarantee propensity >= 0.01 per action (AK-15, Spec §8.5)
        // Since uniform exploration distributes epsilon / K across K actions,
        // we require epsilon / K >= 0.01 => epsilon >= 0.01 * K
        let base_epsilon = self.bandit_exploration.epsilon;
        let min_epsilon = 0.01 * (num_actions as f32);
        let clamped_epsilon = base_epsilon.max(min_epsilon).min(1.0);

        let logging_policy = RandomizedLoggingPolicy::new(clamped_epsilon, num_actions);
        let sample = self.bandit_exploration.next_sample();
        let (selected_action_idx, propensity) =
            logging_policy.select_action(greedy_action_idx, sample);

        let selected_action_usize = (selected_action_idx as usize).min(profile_scores.len() - 1);
        let (orig_idx, selected_profile, _score) = profile_scores[selected_action_usize];

        Some((
            orig_idx,
            selected_profile.clone(),
            selected_action_idx,
            propensity,
        ))
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use contextra_types::TokenBudget;

    #[tokio::test]
    async fn test_router_engine_instance_decision_id_independence() {
        let dir = tempfile::tempdir().unwrap();
        let config = contextra_db::ContextraConfig {
            dimension: 4,
            ..Default::default()
        };
        let db = contextra_db::Contextra::open_with_config(dir.path(), config)
            .await
            .unwrap();
        let collection = db.collection("default").await.unwrap();

        let profile = SlmProfile::new(
            "p1",
            "http://localhost:8000/mcp",
            vec![],
            TokenBudget::new(1000, 100),
            0.1,
        );

        let router1 = crate::tests::tests::create_test_router(
            collection.clone(),
            vec![profile.clone()],
            None,
        );
        let router2 = crate::tests::tests::create_test_router(collection, vec![profile], None);

        let id1_a = router1.decision_ids.next();
        let id2_a = router2.decision_ids.next();

        assert_eq!(id1_a.inner(), 0);
        assert_eq!(id2_a.inner(), 0);

        let id1_b = router1.decision_ids.next();
        let id2_b = router2.decision_ids.next();

        assert_eq!(id1_b.inner(), 1);
        assert_eq!(id2_b.inner(), 1);
    }

    #[tokio::test]
    async fn test_calibration_stats_initial_state(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let config = contextra_db::ContextraConfig {
            dimension: 4,
            ..Default::default()
        };
        let db = contextra_db::Contextra::open_with_config(dir.path(), config).await?;
        let collection = db.collection("default").await?;

        let profile1 = SlmProfile::new(
            "p1",
            "http://localhost:1111",
            vec![1],
            TokenBudget::new(1000, 100),
            0.5,
        );
        let profile2 = SlmProfile::new(
            "p2",
            "http://localhost:2222",
            vec![2],
            TokenBudget::new(1000, 100),
            0.8,
        );

        let router =
            crate::tests::tests::create_test_router(collection, vec![profile1, profile2], None);
        let stats = router.calibration_stats();
        assert_eq!(stats.len(), 2);
        assert_eq!(stats["p1"].times_selected, 0);
        assert_eq!(stats["p1"].calibrated_min_score, 0.5);
        assert_eq!(stats["p1"].original_min_score, 0.5);
        assert_eq!(stats["p2"].times_selected, 0);
        assert_eq!(stats["p2"].calibrated_min_score, 0.8);
        assert_eq!(stats["p2"].original_min_score, 0.8);
        Ok(())
    }

    #[test]
    fn test_profile_calibration_state_reset() {
        let mut state = ProfileCalibrationState::new(0.5);
        state.times_selected = 15;
        state.cumulative_confidence = 12.0;
        state.calibrated_min_score = 0.6;
        state.reset();
        assert_eq!(state.times_selected, 0);
        assert_eq!(state.calibrated_min_score, 0.5);
        assert_eq!(state.cumulative_confidence, 1.0);
    }

    #[tokio::test]
    async fn test_reset_calibration_per_profile(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let config = contextra_db::ContextraConfig {
            dimension: 4,
            ..Default::default()
        };
        let db = contextra_db::Contextra::open_with_config(dir.path(), config).await?;
        let collection = db.collection("default").await?;

        let profile = SlmProfile::new(
            "p1",
            "http://localhost:1111",
            vec![1],
            TokenBudget::new(1000, 100),
            0.5,
        );

        let router = crate::tests::tests::create_test_router(collection, vec![profile], None);
        {
            let current = router.state.load_full();
            let mut new_state = (*current).clone();
            if let Some(state) = new_state.calibration.get_mut("p1") {
                state.times_selected = 5;
            }
            router.state.store(Arc::new(new_state));
        }
        assert_eq!(router.calibration_stats()["p1"].times_selected, 5);

        router.reset_calibration("p1");
        assert_eq!(router.calibration_stats()["p1"].times_selected, 0);
        Ok(())
    }

    #[test]
    #[cfg(feature = "bandit-routing")]
    fn test_pending_bandit_eviction_unit() {
        let dir = tempfile::tempdir().unwrap();
        let config = contextra_db::ContextraConfig {
            dimension: 4,
            ..Default::default()
        };
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let db = contextra_db::Contextra::open_with_config(dir.path(), config)
                .await
                .unwrap();
            let collection = db.collection("default").await.unwrap();

            let profile = SlmProfile::new(
                "p1",
                "http://localhost:1111",
                vec![1],
                TokenBudget::new(1000, 100),
                0.5,
            );

            let router = crate::tests::tests::create_test_router(collection, vec![profile], None);
            let old_time = Instant::now() - Duration::from_secs(400);

            {
                let mut map = router.pending_bandit.write();
                for _ in 0..10_000 {
                    map.insert(
                        router.decision_ids.next(),
                        PendingBanditDecision {
                            context: vec![0.0; 4],
                            profile_name: "p1".to_string(),
                            action_idx: 0,
                            propensity: 0.5,
                            created: old_time,
                        },
                    );
                }
                assert_eq!(map.len(), 10_000);
            }

            router.evict_stale_decisions();

            assert_eq!(router.pending_bandit.read().len(), 0);
        });
    }
}
