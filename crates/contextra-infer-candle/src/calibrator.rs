//! Lokales Trait `ConfidenceCalibrator` und lokaler `ConfigFingerprint`.

use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ConfigFingerprint {
    pub model_id: String,
    pub quantization: String,
    pub prompt_template_hash: [u8; 32],
    pub temperature_bits: u32,
    #[serde(default)]
    pub threshold_bits: u32,
}

impl ConfigFingerprint {
    pub fn new(
        model_id: impl Into<String>,
        quantization: impl Into<String>,
        prompt_template: &str,
        temperature: f32,
    ) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(prompt_template.as_bytes());
        let hash: [u8; 32] = hasher.finalize().into();

        Self {
            model_id: model_id.into(),
            quantization: quantization.into(),
            prompt_template_hash: hash,
            temperature_bits: temperature.to_bits(),
            threshold_bits: 0,
        }
    }

    pub fn with_threshold(mut self, threshold: f32) -> Self {
        self.threshold_bits = threshold.to_bits();
        self
    }

    #[inline]
    pub fn threshold(&self) -> f32 {
        f32::from_bits(self.threshold_bits)
    }

    #[inline]
    pub fn temperature(&self) -> f32 {
        f32::from_bits(self.temperature_bits)
    }
}

pub trait ConfidenceCalibrator: Send + Sync {
    fn record_outcome(&mut self, raw_score: f32, ground_truth: bool);
    fn calibrated_probability(&mut self, raw_score: f32) -> Option<f32>;
    fn invalidate_on_config_change(&mut self, new_fingerprint: ConfigFingerprint);
    fn observation_count(&self) -> usize;
}

#[derive(Debug, Default, Clone)]
pub struct IdentityCalibrator {
    fingerprint: Option<ConfigFingerprint>,
    observations: usize,
}

impl ConfidenceCalibrator for IdentityCalibrator {
    fn record_outcome(&mut self, _raw_score: f32, _ground_truth: bool) {
        self.observations += 1;
    }
    fn calibrated_probability(&mut self, raw_score: f32) -> Option<f32> {
        Some(raw_score)
    }
    fn invalidate_on_config_change(&mut self, new_fingerprint: ConfigFingerprint) {
        if self.fingerprint.as_ref() != Some(&new_fingerprint) {
            self.observations = 0;
            self.fingerprint = Some(new_fingerprint);
        }
    }
    fn observation_count(&self) -> usize {
        self.observations
    }
}
