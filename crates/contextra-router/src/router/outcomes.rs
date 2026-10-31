//! # Ergebnisrückmeldung und Drift-Tracking der RouterEngine
//!
//! **Ring:** Ring 3 (Applikations- und Koordinationsschicht).
//! **Zweck:** Verwaltung von Feedback zu getroffenen Entscheidungen (`record_outcome`),
//! Aktualisierung der konformalen Kalibrierung und Abfrage des Drift-Status.
//!
//! **Kalibrierungs-Update:**
//! Nach jedem Durchlauf meldet der Agent den Erfolg oder Misserfolg via `record_outcome()` zurück.
//! Bei Fehlschlägen wird der Quantils-Schwellwert adaptiv verschärft, um zukünftige Fehlentscheidungen
//! zu reduzieren.

// FILE-CONTEXT
// STAND: 2026-10-29
// ZWECK: Outcome-Recording, Kalibrierungs- und Drift-Statistiken für RouterEngine.
// INVARIANTEN: Conformal recalibration & Bounded pending decisions eviction.

use super::*;

impl RouterEngine {
    #[cfg(feature = "bandit-routing")]
    /// Liefert die aufgezeichnete Logging-Propensity für eine ausstehende Bandit-Entscheidung.
    // ANCHOR[contextra-router:OTC-001]
    /// Liefert die Aktionswahrscheinlichkeit (Propensity) einer offenen Bandit-Entscheidung.
    pub fn bandit_decision_propensity(&self, id: DecisionId) -> Option<f32> {
        self.pending_bandit.read().get(&id).map(|d| d.propensity)
    }

    /// Gibt aktuelle Kalibrierungsstatistik für alle Profile zurück.
    // ANCHOR[contextra-router:OTC-002]
    /// Liefert eine Momentaufnahme aller aktuellen Profil-Kalibrierungen.
    pub fn calibration_stats(&self) -> BTreeMap<String, ProfileCalibrationState> {
        self.state.load().calibration.clone()
    }

    /// Setzt Kalibrierungsstatistik für ein bestimmtes Profil zurück.
    // ANCHOR[contextra-router:OTC-003]
    /// Setzt die Kalibrierungsstatistik für ein einzelnes Profil zurück.
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
    // ANCHOR[contextra-router:OTC-004]
    /// Fragt den Ljapunow-Drift-Status für ein bestimmtes Profil ab.
    pub fn drift_status(&self, profile_name: &str) -> Option<LyapunovResult> {
        self.state
            .load()
            .lyapunov_watchers
            .get(profile_name)
            .and_then(|w| w.latest_result.clone())
    }

    /// Gives a human-readable summary string of Lyapunov drift status across active profile watchers.
    // ANCHOR[contextra-router:OTC-005]
    /// Liefert eine textuelle Gesamtübersicht des System-Drifts.
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
    // ANCHOR[contextra-router:OTC-006]
    /// Setzt die Referenz-Baseline für die Drift-Erkennung eines Profils.
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
        let now_nanos = self.clock.monotonic_nanos();
        let ttl_nanos = PENDING_DECISION_TTL.as_secs().saturating_mul(1_000_000_000);
        let mut map = self.pending_decisions.write();
        if map.len() >= MAX_PENDING_DECISIONS {
            let sample_ts = map.values().next().map(|(_, ts)| *ts).unwrap_or(0);
            eprintln!("EVICTING: len={}, now={}, sample_ts={}, diff={}, ttl={}", map.len(), now_nanos, sample_ts, now_nanos.saturating_sub(sample_ts), ttl_nanos);
            map.retain(|_, (_, ts)| now_nanos.saturating_sub(*ts) <= ttl_nanos);
        }

        #[cfg(feature = "bandit-routing")]
        {
            let mut bandit_map = self.pending_bandit.write();
            if bandit_map.len() >= MAX_PENDING_DECISIONS {
                bandit_map.retain(|_, decision| {
                    now_nanos.saturating_sub(decision.created_nanos) <= ttl_nanos
                });
            }
        }
    }

    /// Muss vom Aufrufer (Agent-Loop) nach Abschluss des SLM-Aufrufs aufgerufen werden.
    /// Liefert das tatsächliche Ergebnis zurück und trainiert die Kalibrierung.
    // ANCHOR[contextra-router:OTC-007]
    /// Protokolliert das Ergebnis einer Routing-Entscheidung und aktualisiert Kalibrierung und Bandit-Statistiken.
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

    /// Anzahl offener Decisions.
    // ANCHOR[contextra-router:OTC-008]
    /// Liefert die Anzahl offener Entscheidungen, die auf Feedback warten.
    pub fn pending_decision_count(&self) -> usize {
        self.pending_decisions.read().len()
    }

    /// Setzt Kalibrierungsstatistik für alle Profile zurück.
    // ANCHOR[contextra-router:OTC-009]
    /// Setzt alle Kalibrierungen systemweit auf ihre Initialwerte zurück.
    pub fn reset_all_calibration(&self) {
        let current = self.state.load_full();
        let mut new_state = (*current).clone();
        for state in new_state.calibration.values_mut() {
            state.reset();
        }
        self.state.store(Arc::new(new_state));
    }
}
