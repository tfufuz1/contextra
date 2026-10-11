// FILE-CONTEXT
// STAND: 2026-10-11T15:03:39Z (SESSION: 46330525)
// ZWECK: Drift-Statistiken und Schwellenwert-Überwachung für Ranking-Scores.
// INVARIANTEN: Strikte Behandlung von non-finite Scores (f32::is_finite) und Schwellenwerten (threshold > 0.0).
// NICHT-OFFENSICHTLICH: Non-finite Scores und Schwellenwerte triggern keinesfalls Panics, sondern fallen auf Default/InsufficientData zurück.
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
    /// Creates a new `DriftDetector` with window size and sensitivity threshold.
    pub fn new(window_size: usize, threshold: f32) -> Self {
        let sanitized_threshold = if threshold.is_finite() && threshold > 0.0 {
            threshold
        } else {
            0.15
        };
        Self {
            window_size: window_size.max(5),
            threshold: sanitized_threshold,
            baseline_mean: None,
            recent_scores: VecDeque::with_capacity(window_size.max(5)),
        }
    }

    /// Creates a `DriftDetector` with default parameters (window size 50, threshold 0.15).
    pub fn with_defaults() -> Self {
        Self::new(50, 0.15)
    }

    /// Sets explicit baseline mean score.
    pub fn set_baseline(&mut self, baseline: f32) {
        if baseline.is_finite() {
            self.baseline_mean = Some(baseline);
        }
    }

    /// Records a new score observation and calculates updated drift status.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drift_detector_threshold_sanitization() {
        let detector_nan = DriftDetector::new(10, f32::NAN);
        assert_eq!(detector_nan.threshold, 0.15);

        let detector_inf = DriftDetector::new(10, f32::INFINITY);
        assert_eq!(detector_inf.threshold, 0.15);

        let detector_neg = DriftDetector::new(10, -0.10);
        assert_eq!(detector_neg.threshold, 0.15);

        let detector_valid = DriftDetector::new(10, 0.05);
        assert_eq!(detector_valid.threshold, 0.05);
    }

    #[test]
    fn test_drift_detector_observe_and_analyze() {
        let mut detector = DriftDetector::new(6, 0.10);
        detector.set_baseline(0.5);

        // Insufficient data initially (< window_size / 2 = 3)
        assert_eq!(detector.observe(0.5), DriftStatus::InsufficientData);
        assert_eq!(detector.observe(0.5), DriftStatus::InsufficientData);

        // Third observation triggers Stable
        assert_eq!(
            detector.observe(0.5),
            DriftStatus::Stable { mean_shift: 0.0 }
        );
        assert_eq!(detector.overall_drift_status(), "stabil");

        // Non-finite scores are ignored
        assert_eq!(
            detector.observe(f32::NAN),
            DriftStatus::Stable { mean_shift: 0.0 }
        );

        // Shift mean above threshold (0.10)
        detector.observe(0.8);
        detector.observe(0.8);
        detector.observe(0.8);

        match detector.analyze() {
            DriftStatus::DriftDetected {
                mean_shift,
                threshold,
            } => {
                assert!(mean_shift > 0.10);
                assert_eq!(threshold, 0.10);
            }
            other => panic!("expected DriftDetected, got {:?}", other),
        }
        assert_eq!(detector.overall_drift_status(), "warnung");
    }

    #[test]
    fn test_drift_status_as_str() {
        assert_eq!(DriftStatus::InsufficientData.as_str(), "unbekannt");
        assert_eq!(DriftStatus::Stable { mean_shift: 0.01 }.as_str(), "stabil");
        assert_eq!(DriftStatus::Stable { mean_shift: 0.08 }.as_str(), "warnung");
        assert_eq!(
            DriftStatus::DriftDetected {
                mean_shift: 0.12,
                threshold: 0.10
            }
            .as_str(),
            "warnung"
        );
        assert_eq!(
            DriftStatus::DriftDetected {
                mean_shift: 0.25,
                threshold: 0.10
            }
            .as_str(),
            "kritisch"
        );
    }
}
