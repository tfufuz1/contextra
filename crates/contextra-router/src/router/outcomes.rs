// FILE-CONTEXT
// ZWECK: Outcome-Recording, Kalibrierungs- und Drift-Statistiken für RouterEngine.
// INVARIANTEN: Conformal recalibration & Bounded pending decisions eviction.

use super::*;

impl RouterEngine {
    #[cfg(feature = "bandit-routing")]
    /// Liefert die aufgezeichnete Logging-Propensity für eine ausstehende Bandit-Entscheidung.
    pub fn bandit_decision_propensity(&self, id: DecisionId) -> Option<f32> {
        self.pending_bandit.read().get(&id).map(|d| d.propensity)
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

    pub(super) fn evict_stale_decisions(&self) {
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
}
