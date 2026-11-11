//! Implementierung von `ConfidenceCalibrator` via `contextra-rank::IsotonicCalibrator`.

use crate::calibrator::{ConfidenceCalibrator, ConfigFingerprint as LocalConfigFingerprint};
use contextra_rank::IsotonicCalibrator;

impl ConfidenceCalibrator for IsotonicCalibrator {
    fn record_outcome(&mut self, raw_score: f32, ground_truth: bool) {
        self.record_outcome(raw_score, ground_truth);
    }

    fn calibrated_probability(&mut self, raw_score: f32) -> Option<f32> {
        self.calibrated_probability(raw_score)
    }

    fn invalidate_on_config_change(&mut self, new_fingerprint: LocalConfigFingerprint) {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(new_fingerprint.model_id.as_bytes());
        hasher.update(new_fingerprint.quantization.as_bytes());
        hasher.update(new_fingerprint.prompt_template_hash);
        hasher.update(new_fingerprint.temperature_bits.to_le_bytes());
        hasher.update(new_fingerprint.threshold_bits.to_le_bytes());
        let hash: [u8; 32] = hasher.finalize().into();

        let rank_fp = contextra_rank::ConfigFingerprint::from_bytes(hash);
        self.invalidate_on_config_change(rank_fp);
    }

    fn observation_count(&self) -> usize {
        self.observation_count()
    }
}
