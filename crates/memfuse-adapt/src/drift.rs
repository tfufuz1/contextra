//! Drift-Verdrahtung zwischen Lyapunov-Drift-Watcher und Bandit-Policy (§B.5.2.3 & §8.2).
//!
//! Reagiert auf erkannte Verteilungsverschiebungen (`DriftDetected`) durch Aufruf von
//! `apply_drift_penalty` auf der angegebenen `BanditPolicy`.

use crate::lyapunov::{LyapunovDriftWatcher, LyapunovResult};

/// Adapter zur Kopplung des `LyapunovDriftWatcher` mit einer `BanditPolicy`.
#[derive(Debug, Clone)]
pub struct DriftPolicyBridge {
    /// Der zugrundeliegende Lyapunov-Drift-Wächter.
    pub watcher: LyapunovDriftWatcher,
    /// Multiplikator k_drift für die Alpha-Erhöhung bei Drift.
    pub k_drift: f32,
    /// Maximale Eskalation α_max.
    pub alpha_max: f32,
    /// Accelerated Discounting Gamma γ_drift.
    pub gamma: f32,
}

impl Default for DriftPolicyBridge {
    fn default() -> Self {
        Self {
            watcher: LyapunovDriftWatcher::default(),
            k_drift: 2.0,
            alpha_max: 4.0,
            gamma: 0.95,
        }
    }
}

impl DriftPolicyBridge {
    /// Erstellt eine neue `DriftPolicyBridge` mit benutzerdefinierten Parametern.
    pub fn new(window_size: usize, k_drift: f32, alpha_max: f32, gamma: f32) -> Self {
        Self {
            watcher: LyapunovDriftWatcher::new(window_size),
            k_drift,
            alpha_max,
            gamma,
        }
    }

    /// Nimmt aktuelle Non-Conformity-Scores auf, wertet den Drift-Status aus und
    /// wendet bei erkannter Drift eine Strafe auf die `BanditPolicy` an.
    #[cfg(feature = "bandit-routing")]
    pub fn observe_and_react<P: crate::bandit::BanditPolicy + ?Sized>(
        &mut self,
        current_scores: &[f32],
        policy: &mut P,
    ) -> LyapunovResult {
        let result = self.watcher.update(current_scores);
        result.apply_drift_if_detected(policy, self.k_drift, self.alpha_max, self.gamma);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "bandit-routing")]
    fn test_drift_policy_bridge_triggers_bandit_penalty() {
        use crate::bandit::BanditProfileState;

        let mut bridge = DriftPolicyBridge::new(10, 2.0, 4.0, 0.95);
        let baseline: Vec<f32> = (0..100).map(|i| (i as f32 / 100.0) * 0.2).collect();
        bridge.watcher.set_baseline(&baseline);

        let mut bandit = BanditProfileState::cold_start(2, 0.5);
        let alpha_before = bandit.alpha;

        let mut last_res = LyapunovResult::InsufficientData;
        for i in 0..25 {
            let shift = (i as f32 / 25.0) * 0.8;
            let current: Vec<f32> = (0..50)
                .map(|j| ((j as f32 / 50.0) * 0.2 + shift).clamp(0.0, 1.0))
                .collect();
            last_res = bridge.observe_and_react(&current, &mut bandit);
        }

        assert!(matches!(last_res, LyapunovResult::DriftDetected { .. }));
        assert!(bandit.alpha > alpha_before);
        assert_eq!(bandit.drift_steps_remaining, bandit.drift_decay_window);
    }

    #[test]
    #[cfg(feature = "bandit-routing")]
    fn test_drift_policy_bridge_dispatches_exact_penalty_parameters() {
        use crate::bandit::BanditPolicy;

        struct RecordingPolicy {
            calls: Vec<(f32, f32, f32)>,
        }

        impl BanditPolicy for RecordingPolicy {
            fn apply_drift_penalty(&mut self, k_drift: f32, alpha_max: f32, gamma: f32) {
                self.calls.push((k_drift, alpha_max, gamma));
            }
        }

        let mut bridge = DriftPolicyBridge::new(5, 2.5, 5.0, 0.92);
        let baseline: Vec<f32> = (0..50).map(|i| (i as f32 / 50.0) * 0.1).collect();
        bridge.watcher.set_baseline(&baseline);

        let mut policy = RecordingPolicy { calls: vec![] };

        for i in 0..15 {
            let shift = (i as f32 / 15.0) * 0.9;
            let current: Vec<f32> = (0..30)
                .map(|j| ((j as f32 / 30.0) * 0.1 + shift).clamp(0.0, 1.0))
                .collect();
            let _ = bridge.observe_and_react(&current, &mut policy);
        }

        assert!(
            !policy.calls.is_empty(),
            "apply_drift_penalty must be called on DriftDetected"
        );
        for call in &policy.calls {
            assert_eq!(*call, (2.5, 5.0, 0.92));
        }
    }
}
