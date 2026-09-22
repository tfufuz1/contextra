//! Score distribution and ranking drift detection for MemFuse search signals.

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
        Self {
            window_size: window_size.max(5),
            threshold: threshold.abs(),
            baseline_mean: None,
            recent_scores: VecDeque::with_capacity(window_size),
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
