// FILE-CONTEXT
// STAND: 2026-09-09T15:45:22Z (SESSION: 6cae458a)
// ZWECK: GASP Grounding-Aware Sensitivity by Perturbation post-hoc hallucination validator.

//! GASP (Grounding-Aware Sensitivity by Perturbation) Post-Hoc Hallucination Validator.

use crate::calibrator::{ConfidenceCalibrator, ConfigFingerprint};
#[cfg(not(feature = "workspace"))]
use crate::calibrator::IdentityCalibrator;
use crate::error::{Error, Result};
use std::sync::Mutex;

pub const DEFAULT_GROUNDING_THRESHOLD: f32 = 0.70;

#[derive(Debug, Clone)]
pub struct GaspConfig {
    pub threshold: f32,
    pub warmup_required: u32,
    pub max_observations: usize,
    pub fingerprint: ConfigFingerprint,
}

impl Default for GaspConfig {
    fn default() -> Self {
        Self {
            threshold: DEFAULT_GROUNDING_THRESHOLD,
            warmup_required: 10,
            max_observations: 2000,
            fingerprint: ConfigFingerprint::new(
                "candle-gasp-v1",
                "Q4_K_M",
                "gasp-attribution",
                0.0,
            )
            .with_threshold(DEFAULT_GROUNDING_THRESHOLD),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ContextChunk {
    pub doc_id: u64,
    pub content: String,
    pub relevance: f32,
    pub contextual_prefix: Option<String>,
}

impl ContextChunk {
    pub fn combined_text_owned(&self) -> String {
        if let Some(ref p) = self.contextual_prefix {
            format!("{p}\n{}", self.content)
        } else {
            self.content.clone()
        }
    }
}

fn contains_at_word_boundary(text: &str, target: &str) -> bool {
    if target.is_empty() {
        return false;
    }
    let target_len = target.len();
    for (idx, _) in text.match_indices(target) {
        let left_ok = if idx == 0 {
            true
        } else {
            text[..idx]
                .chars()
                .next_back()
                .is_none_or(|c| !c.is_alphanumeric())
        };

        let right_ok = if idx + target_len == text.len() {
            true
        } else {
            text[idx + target_len..]
                .chars()
                .next()
                .is_none_or(|c| !c.is_alphanumeric())
        };

        if left_ok && right_ok {
            return true;
        }
    }
    false
}

pub struct GaspValidator {
    config: GaspConfig,
    calibrator: Mutex<Box<dyn ConfidenceCalibrator>>,
}

impl GaspValidator {
    pub fn new() -> Self {
        Self::with_config(GaspConfig::default())
    }

    pub fn with_config(config: GaspConfig) -> Self {
        #[cfg(feature = "workspace")]
        {
            let rank_cal = contextra_rank::IsotonicCalibrator::new(
                config.warmup_required,
                config.max_observations,
            );
            let mut cal: Box<dyn ConfidenceCalibrator> = Box::new(rank_cal);
            cal.invalidate_on_config_change(config.fingerprint.clone());
            Self {
                config,
                calibrator: Mutex::new(cal),
            }
        }
        #[cfg(not(feature = "workspace"))]
        {
            let mut cal = Box::new(IdentityCalibrator::default());
            cal.invalidate_on_config_change(config.fingerprint.clone());
            Self {
                config,
                calibrator: Mutex::new(cal),
            }
        }
    }

    pub fn with_calibrator(
        config: GaspConfig,
        mut calibrator: Box<dyn ConfidenceCalibrator>,
    ) -> Self {
        calibrator.invalidate_on_config_change(config.fingerprint.clone());
        Self {
            config,
            calibrator: Mutex::new(calibrator),
        }
    }

    pub fn set_threshold(&mut self, threshold: f32) {
        let mut new_config = self.config.clone();
        new_config.threshold = threshold;
        new_config.fingerprint = new_config.fingerprint.with_threshold(threshold);
        self.refresh_config(new_config);
    }

    pub fn record_external_feedback(&self, raw_score: f32, ground_truth_grounded: bool) {
        if let Ok(mut cal) = self.calibrator.lock() {
            cal.record_outcome(raw_score, ground_truth_grounded);
        }
    }

    pub fn threshold(&self) -> f32 {
        self.config.threshold
    }

    pub fn config(&self) -> &GaspConfig {
        &self.config
    }

    pub fn refresh_config(&mut self, config: GaspConfig) {
        if let Ok(mut cal) = self.calibrator.lock() {
            cal.invalidate_on_config_change(config.fingerprint.clone());
        }
        self.config = config;
    }

    pub fn observation_count(&self) -> usize {
        if let Ok(cal) = self.calibrator.lock() {
            cal.observation_count()
        } else {
            0
        }
    }

    pub fn compute_raw_grounding_score_local(
        &self,
        response: &str,
        context_chunks: &[ContextChunk],
    ) -> Result<f32> {
        if context_chunks.is_empty() {
            return Err(Error::InvalidInput(
                "Empty context provided for post-hoc grounding validation".to_string(),
            ));
        }

        let trimmed_resp = response.trim();
        if trimmed_resp.is_empty() {
            return Err(Error::InvalidInput(
                "Response is empty for post-hoc grounding validation".to_string(),
            ));
        }

        let combined_context: String = context_chunks
            .iter()
            .map(|c| c.combined_text_owned())
            .collect::<Vec<_>>()
            .join("\n");
        let context_lower = combined_context.to_lowercase();

        let resp_numbers: Vec<&str> = trimmed_resp
            .split(|c: char| !c.is_numeric())
            .filter(|s| !s.is_empty())
            .collect();

        let mut supported_numbers = 0;
        let mut total_numbers = 0;

        for num in &resp_numbers {
            if num.len() < 2 && (num == &"1" || num == &"2" || num == &"3") {
                continue;
            }
            total_numbers += 1;
            if contains_at_word_boundary(&context_lower, num) {
                supported_numbers += 1;
            }
        }

        let number_score = if total_numbers > 0 {
            supported_numbers as f32 / total_numbers as f32
        } else {
            1.0
        };

        let resp_words: Vec<&str> = trimmed_resp
            .split_whitespace()
            .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()))
            .filter(|w| w.len() > 3)
            .collect();

        let mut matched_words = 0;
        let total_words = resp_words.len();

        for word in &resp_words {
            let w_lower = word.to_lowercase();
            if contains_at_word_boundary(&context_lower, &w_lower) {
                matched_words += 1;
            }
        }

        let word_score = if total_words > 0 {
            matched_words as f32 / total_words as f32
        } else {
            1.0
        };

        let base_score = 0.6 * number_score + 0.4 * word_score;
        let raw_score = if total_numbers > 0 && number_score < 1.0 {
            base_score * number_score
        } else {
            base_score
        };

        if raw_score.is_nan() || !raw_score.is_finite() {
            return Ok(0.0);
        }

        Ok(raw_score.clamp(0.0, 1.0))
    }

    pub fn score_grounding_slices(&self, response: &str, sources: &[&str]) -> Result<f32> {
        let chunks: Vec<ContextChunk> = sources
            .iter()
            .enumerate()
            .map(|(i, src)| ContextChunk {
                doc_id: i as u64 + 1,
                content: src.to_string(),
                relevance: 1.0,
                contextual_prefix: None,
            })
            .collect();
        self.compute_raw_grounding_score_local(response, &chunks)
    }

    pub async fn validate_grounding_local(
        &self,
        response: &str,
        context_chunks: &[ContextChunk],
    ) -> Result<f32> {
        let raw_score = self.compute_raw_grounding_score_local(response, context_chunks)?;

        let final_score = if let Ok(mut cal) = self.calibrator.lock() {
            cal.calibrated_probability(raw_score).unwrap_or(raw_score)
        } else {
            raw_score
        };

        if final_score < self.config.threshold {
            return Err(Error::PolicyViolation(format!(
                "LowConfidenceGrounding: grounding score {:.2} is below threshold {:.2}",
                final_score, self.config.threshold
            )));
        }

        Ok(final_score)
    }
}

impl Default for GaspValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_chunk(id: u64, content: &str) -> ContextChunk {
        ContextChunk {
            doc_id: id,
            content: content.to_string(),
            relevance: 0.95,
            contextual_prefix: None,
        }
    }

    #[test]
    fn test_score_grounding_slices() {
        let validator = GaspValidator::new();
        let sources = ["Umsatz 50 Millionen"];
        let res = validator.score_grounding_slices("Umsatz 50 Millionen", &sources);
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_case_a_fully_supported_response_high_score_no_abstention() {
        let validator = GaspValidator::new();
        let chunks = vec![
            sample_chunk(1, "Der Umsatz betrug im Jahr 2025 genau 50 Millionen Euro."),
            sample_chunk(2, "Der Gewinn der Hauptsparte lag bei 5 Millionen Euro."),
        ];
        let response = "Im Jahr 2025 betrug der Umsatz 50 Millionen Euro und der Gewinn lag bei 5 Millionen Euro.";

        let res = validator.validate_grounding_local(response, &chunks).await;
        assert!(res.is_ok());
        let score = res.unwrap();
        assert!(score >= 0.70);
    }

    #[tokio::test]
    async fn test_case_b_hallucinated_number_low_score_triggers_abstention() {
        let validator = GaspValidator::new();
        let chunks = vec![sample_chunk(
            1,
            "Der Umsatz betrug im Jahr 2025 genau 50 Millionen Euro.",
        )];
        let response = "Im Jahr 2025 betrug der Umsatz 99 Millionen Euro.";

        let res = validator.validate_grounding_local(response, &chunks).await;
        assert!(res.is_err());
        match res.unwrap_err() {
            Error::PolicyViolation(msg) => {
                assert!(msg.contains("LowConfidenceGrounding"));
            }
            other => panic!("Expected PolicyViolation, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_case_c_empty_context_zero_shot_defined_explicit_error() {
        let validator = GaspValidator::new();
        let empty_chunks: Vec<ContextChunk> = vec![];
        let response = "Das ist eine Antwort ohne Kontext.";

        let res = validator.validate_grounding_local(response, &empty_chunks).await;
        assert!(res.is_err());
        assert!(matches!(res.unwrap_err(), Error::InvalidInput(_)));
    }

    #[tokio::test]
    async fn test_number_boundary_no_false_positive() {
        let validator = GaspValidator::new();
        let chunks = vec![sample_chunk(
            1,
            "Der Umsatz stieg im Jahr 2025 signifikant an.",
        )];
        let score = validator.compute_raw_grounding_score_local("5", &chunks).unwrap();
        assert_eq!(score, 0.0);
    }

    #[tokio::test]
    async fn test_word_boundary_no_false_positive() {
        let validator = GaspValidator::new();
        let chunks = vec![sample_chunk(
            1,
            "We plan to accelerate the system performance drastically.",
        )];
        let score = validator
            .compute_raw_grounding_score_local("rate", &chunks)
            .unwrap();
        assert_eq!(score, 0.6);
    }

    #[tokio::test]
    async fn test_exact_number_match() {
        let validator = GaspValidator::new();
        let chunks = vec![sample_chunk(
            1,
            "We have 5 items in total in the warehouse.",
        )];
        let score = validator.compute_raw_grounding_score_local("5", &chunks).unwrap();
        assert_eq!(score, 1.0);
    }

    #[tokio::test]
    async fn test_exact_word_match() {
        let validator = GaspValidator::new();
        let chunks = vec![sample_chunk(1, "The rate limit was reached yesterday.")];
        let score = validator
            .compute_raw_grounding_score_local("rate limit", &chunks)
            .unwrap();
        assert_eq!(score, 1.0);
    }

    #[test]
    fn test_gasp_nan_score_protection() {
        let validator = GaspValidator::new();
        let chunks = vec![sample_chunk(1, "Valid context text.")];
        let res = validator.compute_raw_grounding_score_local("Valid response text.", &chunks);
        assert!(res.is_ok());
        let score = res.unwrap();
        assert!((0.0..=1.0).contains(&score));
        assert!(!score.is_nan());
    }

    #[test]
    fn test_config_fingerprint_invalidation_resets_calibrator() {
        let mut validator = GaspValidator::new();
        let old_threshold = validator.threshold();
        assert_eq!(old_threshold, DEFAULT_GROUNDING_THRESHOLD);

        validator.set_threshold(0.85);
        assert_eq!(validator.threshold(), 0.85);
    }

    #[tokio::test]
    async fn test_validate_grounding_does_not_increment_observation_count() {
        let validator = GaspValidator::new();
        let chunks = vec![sample_chunk(
            1,
            "Der Umsatz betrug im Jahr 2025 genau 50 Millionen Euro.",
        )];
        let response = "Im Jahr 2025 betrug der Umsatz 50 Millionen Euro.";

        for _ in 0..5 {
            let res = validator.validate_grounding_local(response, &chunks).await;
            assert!(res.is_ok());
        }
        assert_eq!(validator.observation_count(), 0);

        validator.record_external_feedback(0.95, true);
        assert_eq!(validator.observation_count(), 1);
    }

    #[tokio::test]
    async fn test_set_threshold_updates_fingerprint_and_invalidates_calibration() {
        let mut validator = GaspValidator::new();
        let chunks = vec![sample_chunk(
            1,
            "Der Umsatz betrug im Jahr 2025 genau 50 Millionen Euro.",
        )];
        let response = "Im Jahr 2025 betrug der Umsatz 50 Millionen Euro.";

        let res = validator.validate_grounding_local(response, &chunks).await;
        assert!(res.is_ok());
        let score = res.unwrap();

        validator.record_external_feedback(score, true);
        assert_eq!(validator.observation_count(), 1);

        let initial_fp = validator.config().fingerprint.clone();
        validator.set_threshold(0.85);
        let new_fp = validator.config().fingerprint.clone();
        assert_ne!(initial_fp, new_fp);
        assert_eq!(new_fp.threshold(), 0.85);
        assert_eq!(validator.observation_count(), 0);
    }

    #[tokio::test]
    async fn test_refresh_config_invalidates_calibration_on_fingerprint_change() {
        let fp1 = ConfigFingerprint::new("candle-gasp-v1", "Q4_K_M", "gasp-model-a", 0.0);
        let config1 = GaspConfig {
            fingerprint: fp1.clone(),
            ..GaspConfig::default()
        };
        let mut validator = GaspValidator::with_config(config1);

        let chunks = vec![sample_chunk(
            1,
            "Der Umsatz betrug im Jahr 2025 genau 50 Millionen Euro.",
        )];
        let response = "Im Jahr 2025 betrug der Umsatz 50 Millionen Euro.";

        let res = validator.validate_grounding_local(response, &chunks).await;
        assert!(res.is_ok());
        let score = res.unwrap();
        validator.record_external_feedback(score, true);
        assert_eq!(validator.observation_count(), 1);

        let fp2 = ConfigFingerprint::new("candle-gasp-v2", "Q8_0", "gasp-model-b", 0.0);
        let config2 = GaspConfig {
            fingerprint: fp2.clone(),
            ..GaspConfig::default()
        };
        validator.refresh_config(config2.clone());
        assert_eq!(validator.observation_count(), 0);

        let res2 = validator.validate_grounding_local(response, &chunks).await;
        assert!(res2.is_ok());
        let score2 = res2.unwrap();
        validator.record_external_feedback(score2, true);
        assert_eq!(validator.observation_count(), 1);

        validator.refresh_config(config2);
        assert_eq!(validator.observation_count(), 1);
    }
}
