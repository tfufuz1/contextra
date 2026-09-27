//! Drift-Verdrahtung zwischen Lyapunov-Drift-Watcher und Bandit-Policy (§B.5.2.3 & §8.2).
//!
//! Reagiert auf erkannte Verteilungsverschiebungen (`DriftDetected`) durch Aufruf von
//! `apply_drift_penalty` auf der angegebenen `BanditPolicy`.

use serde::{Deserialize, Serialize};
use crate::lyapunov::{LyapunovDriftWatcher, LyapunovResult};

/// Drift detection signal returned by a [`DriftDetector`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DriftSignal {
    /// Signal distribution is stable.
    Stable,
    /// Distributional drift detected.
    Detected,
}

/// Generic interface for sequential O(1) drift detectors.
pub trait DriftDetector {
    /// Observes a reward or non-conformity score and returns the updated drift signal.
    fn observe(&mut self, reward: f32) -> DriftSignal;
    /// Returns `true` if drift is currently detected.
    fn is_drifting(&self) -> bool;
}

/// Catoni-M-estimator based robust drift detector (arXiv:2505.20051 & arXiv:2501.10974).
///
/// Uses influence function $\psi(x) = \frac{x}{1 + |x|}$ for $O(1)$ robust mean estimation,
/// paired with CUSUM sequential change-point accumulation.
/// Deterministic (P28) and robust against isolated reward outliers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatoniDriftDetector {
    /// Robust mean estimate $\mu$.
    pub catoni_mu: f64,
    /// CUSUM sequential change detection accumulator.
    pub cusum_sum: f64,
    /// Threshold for declaring drift detection.
    pub threshold: f64,
    /// Learning rate $\alpha$ for Catoni M-estimator updates.
    pub alpha: f64,
    /// Baseline target mean reward.
    pub baseline_reward: f64,
}

impl Default for CatoniDriftDetector {
    fn default() -> Self {
        Self::new(0.5, 2.0, 0.05)
    }
}

impl CatoniDriftDetector {
    /// Creates a new `CatoniDriftDetector` with specified baseline, threshold, and learning rate.
    pub fn new(baseline_reward: f64, threshold: f64, alpha: f64) -> Self {
        Self {
            catoni_mu: baseline_reward,
            cusum_sum: 0.0,
            threshold: threshold.max(1e-6),
            alpha: alpha.clamp(0.001, 0.5),
            baseline_reward,
        }
    }

    /// Resets the CUSUM accumulator and restores `catoni_mu` to baseline.
    pub fn reset(&mut self) {
        self.cusum_sum = 0.0;
        self.catoni_mu = self.baseline_reward;
    }
}

impl DriftDetector for CatoniDriftDetector {
    fn observe(&mut self, reward: f32) -> DriftSignal {
        let x = reward as f64 - self.catoni_mu;
        // Influence function ψ(x) = x / (1 + |x|)
        let psi = x / (1.0 + x.abs());
        self.catoni_mu += self.alpha * psi;

        let slack = 0.05;
        let dev = (self.catoni_mu - self.baseline_reward).abs();
        if dev > slack {
            self.cusum_sum += dev - slack;
        } else {
            self.cusum_sum = (self.cusum_sum - 0.01).max(0.0);
        }

        if self.is_drifting() {
            DriftSignal::Detected
        } else {
            DriftSignal::Stable
        }
    }

    fn is_drifting(&self) -> bool {
        self.cusum_sum >= self.threshold
    }
}

/// Ensemble combination of [`LyapunovDriftWatcher`] and [`CatoniDriftDetector`].
///
/// Triggers drift response ONLY if BOTH detectors independently detect drift,
/// significantly reducing false positive rates under noisy or outlier-prone reward streams.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnsembleDriftWatcher {
    /// Primary Lyapunov drift watcher instance.
    pub lyapunov: LyapunovDriftWatcher,
    /// Secondary Catoni robust drift detector instance.
    pub catoni: CatoniDriftDetector,
}

impl Default for EnsembleDriftWatcher {
    fn default() -> Self {
        Self::new(LyapunovDriftWatcher::default(), CatoniDriftDetector::default())
    }
}

impl EnsembleDriftWatcher {
    /// Creates a new `EnsembleDriftWatcher`.
    pub fn new(lyapunov: LyapunovDriftWatcher, catoni: CatoniDriftDetector) -> Self {
        Self { lyapunov, catoni }
    }

    /// Observes a new reward / score and decides if drift is detected by BOTH detectors.
    pub fn observe_and_decide(&mut self, reward: f32) -> bool {
        let l = self.lyapunov.observe_score(reward);
        let c = self.catoni.observe(reward);
        l.is_drift_detected() && matches!(c, DriftSignal::Detected)
    }

    /// Sets the baseline distribution for the Lyapunov watcher and baseline mean for the Catoni detector.
    pub fn set_baseline(&mut self, baseline_scores: &[f32]) {
        self.lyapunov.set_baseline(baseline_scores);
        if !baseline_scores.is_empty() {
            let mean = baseline_scores.iter().map(|&v| v as f64).sum::<f64>()
                / baseline_scores.len() as f64;
            self.catoni.baseline_reward = mean;
            self.catoni.catoni_mu = mean;
            self.catoni.cusum_sum = 0.0;
        }
    }
}

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
    fn test_catoni_drift_detector_stable_stream() {
        let mut detector = CatoniDriftDetector::new(0.5, 1.0, 0.05);
        for _ in 0..100 {
            let sig = detector.observe(0.5);
            assert_eq!(sig, DriftSignal::Stable);
        }
        assert!(!detector.is_drifting());
    }

    #[test]
    fn test_catoni_drift_detector_robust_to_single_outlier() {
        let mut detector = CatoniDriftDetector::new(0.5, 1.0, 0.05);
        for _ in 0..50 {
            detector.observe(0.5);
        }
        // Extreme single outlier
        let sig = detector.observe(100.0);
        assert_eq!(sig, DriftSignal::Stable);
        assert!(!detector.is_drifting());
    }

    #[test]
    fn test_catoni_drift_detector_detects_sustained_shift() {
        let mut detector = CatoniDriftDetector::new(0.5, 0.5, 0.1);
        for _ in 0..20 {
            detector.observe(0.5);
        }
        assert!(!detector.is_drifting());

        let mut detected = false;
        for _ in 0..30 {
            if detector.observe(0.9) == DriftSignal::Detected {
                detected = true;
                break;
            }
        }
        assert!(detected, "CatoniDriftDetector should detect sustained shift");
    }

    #[test]
    fn test_ensemble_drift_watcher_requires_both() {
        let mut ensemble = EnsembleDriftWatcher::default();
        let baseline: Vec<f32> = (0..100).map(|i| (i as f32 / 100.0) * 0.2).collect();
        ensemble.set_baseline(&baseline);

        // Single outlier: neither or only one triggers drift
        let result = ensemble.observe_and_decide(100.0);
        assert!(!result, "Ensemble must not trigger drift on single outlier");
    }

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
