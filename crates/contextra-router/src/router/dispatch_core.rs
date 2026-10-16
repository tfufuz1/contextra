// FILE-CONTEXT
// ZWECK: Kern-Routing-Dispatch und Kaskaden-/Bandit-Profilauswahl für RouterEngine.
// INVARIANTEN: Atomare Snapshot-Sicherheit via ArcSwap, NaN-Safety bei Distanz-Eingaben.

use super::*;

impl RouterEngine {
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
