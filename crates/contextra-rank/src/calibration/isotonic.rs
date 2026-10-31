//! # Isotonische Kalibrierung via PAVA (`isotonic`)
//!
//! **Ring:** Ring 0 (Ranking & Kalibrierung)
//!
//! Implementiert die nicht-parametrische isotonische Regression über den
//! Pool-Adjacent Violators Algorithm (PAVA) sowie die ECE-Berechnung.

use contextra_ports::{Clock, SystemClock};
use contextra_types::ConfigFingerprint;
use std::collections::VecDeque;
use std::fmt;
use std::sync::Arc;

// ADR-098 (PAVA Block-Präzision: f64 -> f32 Konvertierung)
#[allow(clippy::cast_possible_truncation)]
fn f64_to_f32(v: f64) -> f32 {
    if !v.is_finite() {
        0.0
    } else {
        v.clamp(f32::MIN as f64, f32::MAX as f64) as f32
    }
}

const ECE_BINS: usize = 10;
const DEFAULT_WARMUP_REQUIRED: u32 = 50;
const DEFAULT_MAX_OBSERVATIONS: usize = 2000;
const REBUILD_THRESHOLD_NEW_OBS: usize = 10;

/// Isotonic Calibrator using Pool-Adjacent Violators Algorithm.
#[derive(Clone)]
// ANCHOR[contextra-rank:ISO-001]
/// Nicht-parametrischer Kalibrierer auf Basis von PAVA.
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
    clock: Arc<dyn Clock>,
}

impl fmt::Debug for IsotonicCalibrator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IsotonicCalibrator")
            .field("observations", &self.observations)
            .field("warmup_required", &self.warmup_required)
            .field("max_observations", &self.max_observations)
            .field("cached_model", &self.cached_model)
            .field("model_dirty", &self.model_dirty)
            .field(
                "observations_since_rebuild",
                &self.observations_since_rebuild,
            )
            .field("fingerprint", &self.fingerprint)
            .field("last_calibration_at", &self.last_calibration_at)
            .field("cached_ece", &self.cached_ece)
            .field("clock", &"<dyn Clock>")
            .finish()
    }
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
            clock: Arc::new(SystemClock::new()),
        }
    }

    /// Sets a custom clock port for deterministic time access.
    pub fn with_clock(mut self, clock: Arc<dyn Clock>) -> Self {
        self.clock = clock;
        self
    }

    /// Creates an `IsotonicCalibrator` with defaults (Warmup: 50, Max Obs: 2000).
    pub fn with_defaults() -> Self {
        Self::new(DEFAULT_WARMUP_REQUIRED, DEFAULT_MAX_OBSERVATIONS)
    }

    /// Records a new observation `(raw_score, outcome)`. Non-finite `raw_score` values are safely ignored.
    // ANCHOR[contextra-rank:ISO-002]
    /// Nimmt eine neue Beobachtung `(raw_score, outcome)` auf.
    pub fn record_outcome(&mut self, raw_score: f32, outcome: bool) {
        if !raw_score.is_finite() {
            tracing::warn!(
                raw_score,
                "IsotonicCalibrator: non-finite raw_score ignored"
            );
            return;
        }
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
        u32::try_from(self.observations.len()).unwrap_or(u32::MAX) >= self.warmup_required
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
    // ANCHOR[contextra-rank:ISO-003]
    /// Liefert die kalibrierte Wahrscheinlichkeit für einen Roh-Score.
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

    // ANCHOR[contextra-rank:ISO-004]
    // Ablauf des PAVA-Rebuilds (Pool-Adjacent Violators Algorithm):
    // 1. Sortieren: Beobachtungen aufsteigend nach raw_score ordnen.
    // 2. Pooling: Identische Scores zusammenfassen und Labels akkumulieren.
    // 3. Block-Merging: Rückwärts iterieren; verletzt ein Block die Monotonie (last <= prev),
    //    werden beide verschmolzen und deren gewichtetes Mittel gebildet.
    // 4. Stützstellen bilden: Blöcke in f32 konvertieren und im Cache hinterlegen.
    // 5. Zeitstempel über Clock-Port abfragen und ECE für Modellgüte aktualisieren.
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
                if last.0.total_cmp(&(score as f64)) == std::cmp::Ordering::Equal {
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
                let (last_avg, prev_avg) = match (
                    blocks.get(n.saturating_sub(1)),
                    blocks.get(n.saturating_sub(2)),
                ) {
                    (Some(l), Some(p)) if l.2 > 0 && p.2 > 0 => {
                        (l.1 / l.2 as f64, p.1 / p.2 as f64)
                    }
                    _ => break,
                };
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
                .map(|&(score, label_sum, count)| {
                    let avg_prob = if count > 0 {
                        label_sum / count as f64
                    } else {
                        0.0
                    };
                    (f64_to_f32(score), f64_to_f32(avg_prob))
                })
                .collect(),
        );
        self.model_dirty = false;
        self.observations_since_rebuild = 0;
        self.last_calibration_at = Some(self.clock.now_unix_nanos() / 1_000_000_000);
        self.cached_ece = self.calculate_ece();
    }

    fn lookup_isotonic(&self, raw_score: f32) -> Option<f32> {
        let model = self.cached_model.as_ref()?;
        if model.is_empty() {
            return None;
        }

        if model.len() == 1 {
            return model.first().map(|(_, p)| *p);
        }

        match model.binary_search_by(|(threshold, _)| threshold.total_cmp(&raw_score)) {
            Ok(idx) => model.get(idx).map(|(_, p)| *p),
            Err(idx) => {
                if idx >= model.len() {
                    model.last().map(|(_, p)| *p)
                } else {
                    model.get(idx).map(|(_, p)| *p)
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

#[cfg(test)]
mod tests {
    use super::*;

    struct FixedTestClock(u64);

    impl Clock for FixedTestClock {
        fn now_unix_nanos(&self) -> u64 {
            self.0
        }

        fn monotonic_nanos(&self) -> u64 {
            0
        }
    }

    #[test]
    fn test_isotonic_calibrator_deterministic_clock() {
        let clock = Arc::new(FixedTestClock(1_700_000_000_000_000_000));
        let mut cal1 = IsotonicCalibrator::new(2, 100).with_clock(clock.clone());
        let mut cal2 = IsotonicCalibrator::new(2, 100).with_clock(clock);

        cal1.record_outcome(0.1, false);
        cal1.record_outcome(0.9, true);

        cal2.record_outcome(0.1, false);
        cal2.record_outcome(0.9, true);

        cal1.force_rebuild();
        cal2.force_rebuild();

        assert_eq!(cal1.last_calibration_at(), Some(1_700_000_000));
        assert_eq!(cal2.last_calibration_at(), Some(1_700_000_000));
        assert_eq!(cal1.last_calibration_at(), cal2.last_calibration_at());
    }
}
