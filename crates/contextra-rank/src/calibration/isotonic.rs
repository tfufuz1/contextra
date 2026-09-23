//! Isotonic Calibration via PAVA (Pool-Adjacent Violators Algorithm).

use contextra_types::ConfigFingerprint;
use std::collections::VecDeque;

const ECE_BINS: usize = 10;
const DEFAULT_WARMUP_REQUIRED: u32 = 50;
const DEFAULT_MAX_OBSERVATIONS: usize = 2000;
const REBUILD_THRESHOLD_NEW_OBS: usize = 10;

/// Isotonic Calibrator using Pool-Adjacent Violators Algorithm.
#[derive(Debug, Clone)]
pub struct IsotonicCalibrator {
    observations: VecDeque<(f32, bool)>,
    warmup_required: u32,
    max_observations: usize,
    cached_model: Option<Vec<(f32, f32)>>, // (max_score_in_block, calibrated_prob)
    model_dirty: bool,
    observations_since_rebuild: usize,
    fingerprint: Option<ConfigFingerprint>,
    last_calibration_at: Option<u64>,
    cached_ece: Option<f32>,
}

impl IsotonicCalibrator {
    /// Creates a new `IsotonicCalibrator`.
    pub fn new(warmup_required: u32, max_observations: usize) -> Self {
        Self {
            observations: VecDeque::new(),
            warmup_required,
            max_observations,
            cached_model: None,
            model_dirty: true,
            observations_since_rebuild: 0,
            fingerprint: None,
            last_calibration_at: None,
            cached_ece: None,
        }
    }

    /// Creates an `IsotonicCalibrator` with defaults (Warmup: 50, Max Obs: 2000).
    pub fn with_defaults() -> Self {
        Self::new(DEFAULT_WARMUP_REQUIRED, DEFAULT_MAX_OBSERVATIONS)
    }

    /// Records a new observation `(raw_score, outcome)`.
    pub fn record_outcome(&mut self, raw_score: f32, outcome: bool) {
        if self.observations.len() >= self.max_observations {
            self.observations.pop_front();
        }
        self.observations.push_back((raw_score, outcome));
        self.model_dirty = true;
        self.observations_since_rebuild += 1;
    }

    /// Returns current observation count.
    pub fn observation_count(&self) -> usize {
        self.observations.len()
    }

    /// Checks if warmup observations threshold has been reached.
    pub fn is_calibrated(&self) -> bool {
        self.observations.len() as u32 >= self.warmup_required
    }

    /// Returns timestamp (UNIX epoch in seconds) of last model rebuild.
    pub fn last_calibration_at(&self) -> Option<u64> {
        self.last_calibration_at
    }

    /// Returns cached Expected Calibration Error (ECE).
    pub fn cached_ece(&self) -> Option<f32> {
        self.cached_ece
    }

    /// Returns calibrated probability if warm, or `None` if below warmup requirement.
    pub fn calibrated_probability(&mut self, raw_score: f32) -> Option<f32> {
        if !self.is_calibrated() {
            return None;
        }
        if self.model_dirty
            && (self.cached_model.is_none()
                || self.observations_since_rebuild >= REBUILD_THRESHOLD_NEW_OBS)
        {
            self.rebuild_model();
        }
        self.lookup_isotonic(raw_score)
    }

    /// Forces immediate PAVA model rebuild.
    pub fn force_rebuild(&mut self) {
        if self.model_dirty {
            self.rebuild_model();
        }
    }

    /// Resets observations if ConfigFingerprint changes (P8 Compliance).
    pub fn invalidate_on_config_change(&mut self, new_fingerprint: ConfigFingerprint) {
        if self.fingerprint.as_ref() != Some(&new_fingerprint) {
            tracing::warn!(
                old_fp = ?self.fingerprint,
                new_fp = ?new_fingerprint,
                obs_count = self.observations.len(),
                "IsotonicCalibrator: ConfigFingerprint changed — resetting (P8)"
            );
            self.observations.clear();
            self.cached_model = None;
            self.model_dirty = true;
            self.observations_since_rebuild = 0;
            self.fingerprint = Some(new_fingerprint);
            self.cached_ece = None;
        }
    }

    fn rebuild_model(&mut self) {
        let mut sorted: Vec<(f32, f32)> = self
            .observations
            .iter()
            .map(|&(score, outcome)| (score, if outcome { 1.0 } else { 0.0 }))
            .collect();
        sorted.sort_by(|a, b| a.0.total_cmp(&b.0));

        let mut aggregated: Vec<(f64, f64, usize)> = Vec::with_capacity(sorted.len());
        for (score, label) in sorted {
            if let Some(last) = aggregated.last_mut() {
                if (last.0 as f32).total_cmp(&score) == std::cmp::Ordering::Equal {
                    last.1 += label as f64;
                    last.2 += 1;
                    continue;
                }
            }
            aggregated.push((score as f64, label as f64, 1));
        }

        let mut blocks: Vec<(f64, f64, usize)> = Vec::with_capacity(aggregated.len());

        for (score, label_sum, count) in aggregated {
            blocks.push((score, label_sum, count));

            while blocks.len() >= 2 {
                let n = blocks.len();
                let last_avg = blocks[n - 1].1 / blocks[n - 1].2 as f64;
                let prev_avg = blocks[n - 2].1 / blocks[n - 2].2 as f64;
                if last_avg <= prev_avg {
                    if let Some(last) = blocks.pop() {
                        if let Some(prev) = blocks.last_mut() {
                            prev.0 = last.0;
                            prev.1 += last.1;
                            prev.2 += last.2;
                        }
                    }
                } else {
                    break;
                }
            }
        }

        self.cached_model = Some(
            blocks
                .iter()
                .map(|(score, label_sum, count)| {
                    (*score as f32, (label_sum / *count as f64) as f32)
                })
                .collect(),
        );
        self.model_dirty = false;
        self.observations_since_rebuild = 0;
        self.last_calibration_at = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        );
        self.cached_ece = self.calculate_ece();
    }

    fn lookup_isotonic(&self, raw_score: f32) -> Option<f32> {
        let model = self.cached_model.as_ref()?;
        if model.is_empty() {
            return None;
        }

        if model.len() == 1 {
            return Some(model[0].1);
        }

        match model.binary_search_by(|(threshold, _)| threshold.total_cmp(&raw_score)) {
            Ok(idx) => Some(model[idx].1),
            Err(idx) => {
                if idx >= model.len() {
                    model.last().map(|(_, p)| *p)
                } else {
                    Some(model[idx].1)
                }
            }
        }
    }

    /// Calculates Expected Calibration Error over 10 bins.
    pub fn expected_calibration_error(&mut self) -> Option<f32> {
        if !self.is_calibrated() {
            return None;
        }
        if self.model_dirty {
            self.rebuild_model();
        }
        self.cached_ece
    }

    fn calculate_ece(&self) -> Option<f32> {
        if !self.is_calibrated() {
            return None;
        }

        let n = self.observations.len() as f32;
        if n == 0.0 {
            return Some(0.0);
        }

        let probs_and_outcomes: Vec<(f32, bool)> = self
            .observations
            .iter()
            .filter_map(|&(score, outcome)| self.lookup_isotonic(score).map(|prob| (prob, outcome)))
            .collect();

        let bin_width = 1.0 / ECE_BINS as f32;
        let mut ece = 0.0f32;

        for bin_idx in 0..ECE_BINS {
            let lo = bin_idx as f32 * bin_width;
            let hi = lo + bin_width;

            let bin_obs: Vec<(f32, bool)> = probs_and_outcomes
                .iter()
                .filter(|&&(prob, _)| {
                    prob >= lo && (prob < hi || (bin_idx == ECE_BINS - 1 && prob <= hi))
                })
                .copied()
                .collect();

            if bin_obs.is_empty() {
                continue;
            }

            let bin_n = bin_obs.len() as f32;
            let avg_confidence = bin_obs.iter().map(|(p, _)| p).sum::<f32>() / bin_n;
            let avg_accuracy = bin_obs.iter().filter(|(_, o)| *o).count() as f32 / bin_n;
            ece += (bin_n / n) * (avg_confidence - avg_accuracy).abs();
        }
        Some(ece)
    }
}
