//! # Kern-Routing-Dispatch und Kaskaden-/Bandit-Profilauswahl
//!
//! **Ring:** Ring 3 (Applikations- und Koordinationsschicht).
//! **Zweck:** Ausführung der Profilauswahl-Kaskade (`route()`), Kontext-Anreicherung über Hybrid-Suche
//! und Graph-Communities sowie multi-kriterielle Filterung nach Token-Budgets und Konfidenz-Schwellwerten.

// FILE-CONTEXT
// STAND: 2026-10-29
// ZWECK: Kern-Routing-Dispatch und Kaskaden-/Bandit-Profilauswahl für RouterEngine.
// INVARIANTEN: Atomare Snapshot-Sicherheit via ArcSwap, NaN-Safety bei Distanz-Eingaben.

use super::*;

pub(crate) type SelectionResult = (
    SlmProfile,
    Option<ConfidenceMetrics>,
    Option<(u32, f32)>,
);

impl RouterEngine {
    /// Fetches search results and resolves graph communities for each result chunk.
    async fn fetch_and_enrich_chunks(
        &self,
        query_embedding: &[f32],
        query_text: &str,
    ) -> Result<Vec<(ContextChunk, Option<u64>)>> {
        let search_chunks = self
            .search_provider
            .search_hybrid(query_text, query_embedding, 10)
            .await?;

        if search_chunks.is_empty() {
            return Err(ContextraError::NotFound(
                "Keine relevanten Suchergebnisse für Routing gefunden".to_string(),
            ));
        }

        let mut chunks = Vec::with_capacity(search_chunks.len());
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

        Ok(chunks)
    }

    /// Constructs ConfidenceMetrics for a selected profile.
    fn build_confidence_metrics(
        state: &ProfileCalibrationState,
        best_score: f32,
        confidence_ratio: f32,
        non_conformity: f32,
    ) -> ConfidenceMetrics {
        let calibrated = state.conformal.window_total >= CALIBRATION_WARMUP_WINDOW as u64;
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
            selection_margin: confidence_ratio,
        }
    }

    /// Evaluates profile candidate selection and updates calibration confidence stats.
    fn calculate_routing_selection(
        &self,
        profiles: &[SlmProfile],
        chunks: &[(ContextChunk, Option<u64>)],
        _query_embedding: &[f32],
        cal: &mut BTreeMap<String, ProfileCalibrationState>,
    ) -> Result<SelectionResult> {
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

        #[cfg(feature = "bandit-routing")]
        let bandit_selection = if matches!(self.routing_strategy, RoutingStrategy::ContextualBandit)
        {
            self.select_profile_bandit(chunks, &effective_profiles, _query_embedding)
        } else {
            None
        };
        #[cfg(not(feature = "bandit-routing"))]
        let bandit_selection: Option<(usize, SlmProfile, u32, f32)> = None;

        let (selected_idx, selected_profile, is_bandit) =
            if let Some((idx, profile, action_idx, propensity)) = bandit_selection {
                (idx, profile, Some((action_idx, propensity)))
            } else {
                let (idx, profile, _) =
                    self.select_profile_cascade(chunks, &effective_profiles, cal)?;
                (idx, profile, None)
            };

        let profile_scores = compute_profile_scores(profiles, chunks);
        let best_score = profile_scores.get(&selected_idx).copied().unwrap_or(0.0);
        let second_best = profile_scores
            .iter()
            .filter(|(idx, _)| **idx != selected_idx)
            .map(|(_, s)| *s)
            .fold(0.0f32, f32::max);

        let confidence_ratio: f32 = if second_best > 0.0 {
            best_score / second_best
        } else {
            2.0
        };

        let non_conformity = if best_score > 0.0 {
            (1.0 / confidence_ratio.max(f32::EPSILON)).clamp(0.0, 1.0)
        } else {
            1.0
        };

        if let Some(state) = cal.get_mut(&selected_profile.name) {
            state.times_selected += 1;
            state.cumulative_confidence += f64::from(confidence_ratio);
        }

        let metrics = cal
            .get(&selected_profile.name)
            .map(|state| Self::build_confidence_metrics(state, best_score, confidence_ratio, non_conformity));

        Ok((selected_profile, metrics, is_bandit))
    }

    /// Records pending decision entry into pending_decisions and pending_bandit.
    fn record_pending_decision(
        &self,
        decision_id: DecisionId,
        selected_profile_name: &str,
        _query_embedding: &[f32],
        _is_bandit: Option<(u32, f32)>,
        now_nanos: u64,
    ) {
        self.pending_decisions
            .write()
            .insert(decision_id, (selected_profile_name.to_string(), now_nanos));

        #[cfg(feature = "bandit-routing")]
        if let Some((action_idx, propensity)) = _is_bandit {
            self.pending_bandit.write().insert(
                decision_id,
                PendingBanditDecision {
                    context: _query_embedding.to_vec(),
                    profile_name: selected_profile_name.to_string(),
                    action_idx,
                    propensity,
                    created_nanos: now_nanos,
                },
            );
        }
    }

    /// Updates Lyapunov drift watchers for the selected profile and handles drift side effects.
    fn update_lyapunov_drift(
        watchers: &mut BTreeMap<String, LyapunovDriftWatcher>,
        selected_profile_name: &str,
        _profiles: &mut [SlmProfile],
        non_conformity_score: f32,
    ) -> Option<LyapunovResult> {
        let watcher = watchers.get_mut(selected_profile_name)?;
        watcher.observe_score(non_conformity_score);
        let res = watcher.analyze();

        if let LyapunovResult::DriftDetected {
            lyapunov_exponent,
            ref reason,
        } = res
        {
            tracing::warn!(
                profile = %selected_profile_name,
                lambda = lyapunov_exponent,
                kl_divergence = reason.kl_divergence,
                "Lyapunov drift detected — calibration may be stale"
            );

            #[cfg(feature = "bandit-routing")]
            {
                let k_drift = (1.0 + lyapunov_exponent.max(0.0)).clamp(1.5, 4.0);
                if let Some(profile_state) = _profiles.iter_mut().find(|p| p.name == selected_profile_name) {
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

        let state_snap = self.state.load_full();
        let profiles = state_snap.profiles.clone();

        if profiles.is_empty() {
            return Err(ContextraError::NotFound(
                "Keine SLM-Profile für Routing konfiguriert".to_string(),
            ));
        }

        let chunks = self
            .fetch_and_enrich_chunks(query_embedding, query_text)
            .await?;

        let current_state = self.state.load_full();
        let mut new_state = (*current_state).clone();

        let (selected_profile, confidence_metrics, is_bandit_decision) = self
            .calculate_routing_selection(
                &profiles,
                &chunks,
                query_embedding,
                &mut new_state.calibration,
            )?;

        let decision_id = self.decision_ids.next();
        let now_nanos = self.clock.monotonic_nanos();
        self.record_pending_decision(
            decision_id,
            &selected_profile.name,
            query_embedding,
            is_bandit_decision,
            now_nanos,
        );

        let non_conformity_score = confidence_metrics
            .as_ref()
            .map(|m| m.non_conformity_score)
            .unwrap_or(1.0);

        let drift_status = Self::update_lyapunov_drift(
            &mut new_state.lyapunov_watchers,
            &selected_profile.name,
            &mut new_state.profiles,
            non_conformity_score,
        );

        self.state.store(Arc::new(new_state));

        let raw_chunks: Vec<ContextChunk> = chunks.into_iter().map(|(c, _)| c).collect();
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

    /// Filters profiles matching community rules and sorts candidates descending by threshold score.
    fn filter_and_sort_cascade_candidates(
        &self,
        chunks: &[(ContextChunk, Option<u64>)],
        profiles: &[SlmProfile],
    ) -> Result<Vec<(usize, SlmProfile)>> {
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

        let eligible: Vec<(usize, SlmProfile)> = profiles
            .iter()
            .enumerate()
            .filter(|(_, profile)| {
                profile.domain_communities.is_empty()
                    || chunks.iter().any(|(_, comm_id)| {
                        comm_id.is_some_and(|cid| profile.domain_communities.contains(&cid))
                    })
            })
            .map(|(idx, profile)| (idx, profile.clone()))
            .collect();

        if eligible.is_empty() {
            return Err(ContextraError::NotFound(
                "Kein SLM-Profil entspricht der Community-Zuordnung".to_string(),
            ));
        }

        let mut sorted = eligible;
        sorted.sort_by(|(idx_a, a), (idx_b, b)| {
            b.min_relevance_score
                .total_cmp(&a.min_relevance_score)
                .then_with(|| {
                    let score_a = compute_profile_score(a, chunks);
                    let score_b = compute_profile_score(b, chunks);
                    score_b.total_cmp(&score_a).then_with(|| idx_a.cmp(idx_b))
                })
        });

        Ok(sorted)
    }

    /// Evaluates cascade profiles against calibrated thresholds.
    fn evaluate_cascade_thresholds(
        &self,
        sorted_profiles: &[(usize, SlmProfile)],
        chunks: &[(ContextChunk, Option<u64>)],
        calibration: &mut BTreeMap<String, ProfileCalibrationState>,
    ) -> Option<(usize, SlmProfile, ConfidenceMetrics)> {
        for (orig_idx, profile) in sorted_profiles {
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
                return Some((*orig_idx, profile.clone(), confidence));
            }
        }
        None
    }

    /// Evaluates lowest-cost fallback when no cascade profile meets the threshold.
    fn evaluate_cascade_fallback(
        &self,
        eligible: &[(usize, SlmProfile)],
        chunks: &[(ContextChunk, Option<u64>)],
        calibration: &BTreeMap<String, ProfileCalibrationState>,
    ) -> Result<(usize, SlmProfile, ConfidenceMetrics)> {
        let &(fallback_idx, ref fallback_profile) = eligible
            .iter()
            .min_by(|(idx_a, a), (idx_b, b)| {
                a.estimated_cost()
                    .total_cmp(&b.estimated_cost())
                    .then_with(|| a.min_relevance_score.total_cmp(&b.min_relevance_score))
                    .then_with(|| idx_a.cmp(idx_b))
            })
            .ok_or_else(|| ContextraError::NotFound("Keine SLM-Profile konfiguriert".to_string()))?;

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
            "Kaskaden-Fallback: Kein Profil über Schwellenwert"
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

    /// Kalibriertes Kaskaden-Routing.
    pub(crate) fn select_profile_cascade(
        &self,
        chunks: &[(ContextChunk, Option<u64>)],
        profiles: &[SlmProfile],
        calibration: &mut BTreeMap<String, ProfileCalibrationState>,
    ) -> Result<(usize, SlmProfile, ConfidenceMetrics)> {
        let sorted_candidates = self.filter_and_sort_cascade_candidates(chunks, profiles)?;

        if let Some(selected) =
            self.evaluate_cascade_thresholds(&sorted_candidates, chunks, calibration)
        {
            return Ok(selected);
        }

        self.evaluate_cascade_fallback(&sorted_candidates, chunks, calibration)
    }

    #[cfg(feature = "bandit-routing")]
    /// Evaluates bandit scores for eligible profiles.
    fn evaluate_bandit_profile_scores(
        &self,
        eligible_profiles: &[(usize, &SlmProfile)],
        query_embedding: &[f32],
    ) -> Option<Vec<(usize, SlmProfile, f32)>> {
        let mut profile_scores = Vec::with_capacity(eligible_profiles.len());

        for &(orig_idx, profile) in eligible_profiles {
            let Some(ref bstate) = profile.bandit_state else {
                tracing::warn!(profile = %profile.name, "Bandit-Fallback: Profil besitzt keinen bandit_state");
                return None;
            };

            #[cfg(feature = "cloud-egress-guard")]
            let is_cloud = profile.transport.is_cloud();
            #[cfg(not(feature = "cloud-egress-guard"))]
            let is_cloud = false;

            let cost = profile.estimated_cost();
            match bstate.score(query_embedding, cost, is_cloud) {
                Ok(score) if score.is_finite() => {
                    profile_scores.push((orig_idx, profile.clone(), score));
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
                    let contextra_err = ContextraError::from(err);
                    tracing::warn!(
                        profile = %profile.name,
                        err = %contextra_err,
                        "Bandit-Dispatch Fallback auf Cascade: BanditError bei Score-Berechnung"
                    );
                    return None;
                }
            }
        }

        Some(profile_scores)
    }

    #[cfg(feature = "bandit-routing")]
    /// Selects bandit action using randomized logging policy.
    fn choose_bandit_action(
        &self,
        profile_scores: &[(usize, SlmProfile, f32)],
        num_actions: u32,
    ) -> (usize, SlmProfile, u32, f32) {
        use contextra_adapt::offpolicy::RandomizedLoggingPolicy;

        let greedy_action_idx = profile_scores
            .iter()
            .enumerate()
            .max_by(|(_, (idx_a, _, score_a)), (_, (idx_b, _, score_b))| {
                score_a
                    .total_cmp(score_b)
                    .then_with(|| idx_b.cmp(idx_a))
            })
            .map(|(action_idx, _)| action_idx as u32)
            .unwrap_or(0);

        let min_epsilon = 0.01 * (num_actions as f32);
        let clamped_epsilon = self.bandit_exploration.epsilon.max(min_epsilon).min(1.0);

        let logging_policy = RandomizedLoggingPolicy::new(clamped_epsilon, num_actions);
        let sample = self.bandit_exploration.next_sample();
        let (selected_action_idx, propensity) =
            logging_policy.select_action(greedy_action_idx, sample);

        let selected_usize = (selected_action_idx as usize).min(profile_scores.len() - 1);
        let (orig_idx, ref selected_profile, _) = profile_scores[selected_usize];

        (
            orig_idx,
            selected_profile.clone(),
            selected_action_idx,
            propensity,
        )
    }

    #[cfg(feature = "bandit-routing")]
    /// Contextual Bandit Profilauswahl via LinUCB / Sherman-Morrison (§13.2, §8.5, AK-15).
    pub(crate) fn select_profile_bandit(
        &self,
        chunks: &[(ContextChunk, Option<u64>)],
        profiles: &[SlmProfile],
        query_embedding: &[f32],
    ) -> Option<(usize, SlmProfile, u32, f32)> {
        if profiles.is_empty() || chunks.is_empty() {
            return None;
        }

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
        let profile_scores = self.evaluate_bandit_profile_scores(&eligible_profiles, query_embedding)?;
        Some(self.choose_bandit_action(&profile_scores, num_actions))
    }
}
