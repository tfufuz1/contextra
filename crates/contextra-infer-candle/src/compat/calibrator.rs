//! Implementierung von `ConfidenceCalibrator` via `contextra-rank::IsotonicCalibrator`.

use crate::calibrator::ConfidenceCalibrator;
use contextra_rank::IsotonicCalibrator;
use contextra_types::ConfigFingerprint;

impl ConfidenceCalibrator for IsotonicCalibrator {
    fn record_outcome(&mut self, raw_score: f32, ground_truth: bool) {
        self.record_outcome(raw_score, ground_truth);
    }

    fn calibrated_probability(&mut self, raw_score: f32) -> Option<f32> {
        self.calibrated_probability(raw_score)
    }

    fn invalidate_on_config_change(&mut self, new_fingerprint: ConfigFingerprint) {
        self.invalidate_on_config_change(new_fingerprint);
    }

    fn observation_count(&self) -> usize {
        self.observation_count()
    }
}
