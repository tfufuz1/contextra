//! Brücke für Kalibratoren aus `contextra-rank` zu `ConfidenceCalibrator`.

use crate::calibrator::{ConfidenceCalibrator, ConfigFingerprint};
use contextra_rank::IsotonicCalibrator;

/// Adapter um `IsotonicCalibrator` als `ConfidenceCalibrator` zu verwenden.
pub struct RankIsotonicCalibratorAdapter {
    calibrator: IsotonicCalibrator,
    fingerprint: Option<ConfigFingerprint>,
}

impl RankIsotonicCalibratorAdapter {
    /// Erstellt einen neuen Adapter.
    pub fn new(
        warmup_required: u32,
        max_observations: usize,
        fingerprint: ConfigFingerprint,
    ) -> Self {
        let mut cal = IsotonicCalibrator::new(warmup_required, max_observations);
        cal.invalidate_on_config_change(contextra_rank::ConfigFingerprint::from_bytes(
            fingerprint.prompt_template_hash,
        ));
        Self {
            calibrator: cal,
            fingerprint: Some(fingerprint),
        }
    }
}

impl ConfidenceCalibrator for RankIsotonicCalibratorAdapter {
    fn calibrated_probability(&mut self, raw_score: f32) -> Option<f32> {
        self.calibrator.calibrated_probability(raw_score)
    }

    fn record_outcome(&mut self, raw_score: f32, ground_truth: bool) {
        self.calibrator.record_outcome(raw_score, ground_truth);
    }

    fn invalidate_on_config_change(&mut self, new_fingerprint: ConfigFingerprint) {
        self.calibrator.invalidate_on_config_change(
            contextra_rank::ConfigFingerprint::from_bytes(new_fingerprint.prompt_template_hash),
        );
        self.fingerprint = Some(new_fingerprint);
    }

    fn observation_count(&self) -> usize {
        self.calibrator.observation_count()
    }
}
