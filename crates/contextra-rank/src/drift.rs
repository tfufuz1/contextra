// FILE-CONTEXT
// STAND: 2026-10-11T15:07:28Z (SESSION: 5aad92a6)
// ZWECK: Score-Verteilungs- und Drift-Erkennung für Contextra Search Signals.
// INVARIANTEN: P26-konform (rein synchron, 0 async/tokio), non-finite floats löschen keine bestehenden Messungen, Schwellenwerte sind stets positiv und endlich.
// NICHT-OFFENSICHTLICH: analyse() erfordert mindestens window_size / 2 Beobachtungen und ein gesetztes Baseline-Mean.
// SIEHE AUCH: crates/contextra-rank/AGENTS.md

//! Score distribution and ranking drift detection for Contextra search signals.

use std::collections::VecDeque;

/// Drift status summary representation.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum DriftStatus {
    /// Distribution is stable.
    Stable {
        /// Mean score shift relative to baseline.
        mean_shift: f32,
    },
    /// Drift detected exceeding configured tolerance threshold.
    DriftDetected {
        /// Mean score shift relative to baseline.
        mean_shift: f32,
        /// Threshold that was breached.
        threshold: f32,
    },
    /// Insufficient observations to perform statistical drift detection.
    InsufficientData,
}

impl DriftStatus {
    /// Returns human-readable summary string ("stabil", "warnung", "kritisch", or "unbekannt").
    pub fn as_str(&self) -> &'static str {
        match self {
            DriftStatus::Stable { mean_shift } => {
                if mean_shift.abs() > 0.05 {
                    "warnung"
                } else {
                    "stabil"
                }
            }
            DriftStatus::DriftDetected { mean_shift, .. } => {
                if mean_shift.abs() > 0.20 {
                    "kritisch"
                } else {
                    "warnung"
                }
            }
            DriftStatus::InsufficientData => "unbekannt",
        }
    }
}

/// Drift detector monitoring score shifts over sliding windows.
#[derive(Debug, Clone)]
pub struct DriftDetector {
    window_size: usize,
    threshold: f32,
    baseline_mean: Option<f32>,
    recent_scores: VecDeque<f32>,
}

impl DriftDetector {
    /// Default threshold value used when none or an invalid threshold is provided.
    pub const DEFAULT_THRESHOLD: f32 = 0.15;
    /// Default window size used for score tracking.
    pub const DEFAULT_WINDOW_SIZE: usize = 50;

    /// Creates a new `DriftDetector` with window size and sensitivity threshold.
    ///
    /// If `threshold` is non-finite or non-positive, [`Self::DEFAULT_THRESHOLD`] is used.
    pub fn new(window_size: usize, threshold: f32) -> Self {
        let sanitized_threshold = if threshold.is_finite() && threshold > 0.0 {
            threshold
        } else {
            Self::DEFAULT_THRESHOLD
        };
        let effective_window = window_size.max(5);
        Self {
            window_size: effective_window,
            threshold: sanitized_threshold,
            baseline_mean: None,
            recent_scores: VecDeque::with_capacity(effective_window),
        }
    }

    /// Creates a `DriftDetector` with default parameters (window size 50, threshold 0.15).
    pub fn with_defaults() -> Self {
        Self::new(Self::DEFAULT_WINDOW_SIZE, Self::DEFAULT_THRESHOLD)
    }

    /// Returns the configured sliding window size.
    pub fn window_size(&self) -> usize {
        self.window_size
    }

    /// Returns the active drift threshold.
    pub fn threshold(&self) -> f32 {
        self.threshold
    }

    /// Returns the configured baseline mean, if set.
    pub fn baseline_mean(&self) -> Option<f32> {
        self.baseline_mean
    }

    /// Returns the number of recorded recent score observations.
    pub fn observation_count(&self) -> usize {
        self.recent_scores.len()
    }

    /// Clears all recorded observations while retaining baseline and threshold settings.
    pub fn clear_observations(&mut self) {
        self.recent_scores.clear();
    }

    /// Clears the configured baseline mean score.
    pub fn clear_baseline(&mut self) {
        self.baseline_mean = None;
    }

    /// Sets explicit baseline mean score. Non-finite values are ignored.
    pub fn set_baseline(&mut self, baseline: f32) {
        if baseline.is_finite() {
            self.baseline_mean = Some(baseline);
        }
    }

    /// Records a new score observation and calculates updated drift status.
    /// Non-finite scores are ignored and do not alter recorded history.
    pub fn observe(&mut self, score: f32) -> DriftStatus {
        if !score.is_finite() {
            return self.analyze();
        }

        if self.recent_scores.len() >= self.window_size {
            self.recent_scores.pop_front();
        }
        self.recent_scores.push_back(score);

        self.analyze()
    }

    /// Analyzes current observations against baseline.
    pub fn analyze(&self) -> DriftStatus {
        if self.recent_scores.len() < self.window_size / 2 {
            return DriftStatus::InsufficientData;
        }

        let current_sum: f32 = self.recent_scores.iter().sum();
        let current_mean = current_sum / self.recent_scores.len() as f32;

        let baseline = match self.baseline_mean {
            Some(b) => b,
            None => return DriftStatus::InsufficientData,
        };

        let mean_shift = (current_mean - baseline).abs();
        if mean_shift > self.threshold {
            DriftStatus::DriftDetected {
                mean_shift,
                threshold: self.threshold,
            }
        } else {
            DriftStatus::Stable { mean_shift }
        }
    }

    /// Returns human-readable overall drift status string.
    pub fn overall_drift_status(&self) -> &'static str {
        self.analyze().as_str()
    }
}

impl Default for DriftDetector {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drift_detector_default() {
        let detector = DriftDetector::default();
        assert_eq!(detector.window_size(), DriftDetector::DEFAULT_WINDOW_SIZE);
        assert_eq!(detector.threshold(), DriftDetector::DEFAULT_THRESHOLD);
        assert_eq!(detector.baseline_mean(), None);
        assert_eq!(detector.observation_count(), 0);
    }

    #[test]
    fn test_drift_detector_non_finite_threshold_fallback() {
        let detector_nan = DriftDetector::new(10, f32::NAN);
        assert_eq!(detector_nan.threshold(), DriftDetector::DEFAULT_THRESHOLD);

        let detector_neg = DriftDetector::new(10, -0.5);
        assert_eq!(detector_neg.threshold(), DriftDetector::DEFAULT_THRESHOLD);

        let detector_valid = DriftDetector::new(10, 0.2);
        assert_eq!(detector_valid.threshold(), 0.2);
    }

    #[test]
    fn test_drift_detector_clearing_state() {
        let mut detector = DriftDetector::new(10, 0.1);
        detector.set_baseline(0.5);
        for _ in 0..10 {
            detector.observe(0.51);
        }
        assert_eq!(detector.observation_count(), 10);
        assert_eq!(detector.baseline_mean(), Some(0.5));

        detector.clear_observations();
        assert_eq!(detector.observation_count(), 0);
        assert_eq!(detector.analyze(), DriftStatus::InsufficientData);

        detector.clear_baseline();
        assert_eq!(detector.baseline_mean(), None);
    }

    #[test]
    fn test_drift_detector_non_finite_score_handling() {
        let mut detector = DriftDetector::new(10, 0.1);
        detector.set_baseline(0.5);
        for _ in 0..5 {
            detector.observe(0.5);
        }
        assert_eq!(detector.observation_count(), 5);

        // Observing NaN should not change observation count
        detector.observe(f32::NAN);
        assert_eq!(detector.observation_count(), 5);

        detector.set_baseline(f32::NAN);
        assert_eq!(detector.baseline_mean(), Some(0.5));
    }
}
