//! Conformal Calibration and Adaptive Conformal Prediction with Propensity Weighting.
//!
//! Provides quantile-based online threshold estimation for target coverage guarantees
//! under covariate shift via propensity weighting.

use contextra_types::ConfigFingerprint;
use thiserror::Error;

/// Error type for conformal calibration and threshold updates.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum ConformalError {
    /// Target miscoverage level `alpha` is not finite or outside (0.0, 1.0).
    #[error("Invalid target miscoverage alpha: {0} (must be finite and in (0.0, 1.0))")]
    InvalidAlpha(f32),

    /// Propensity weight is not finite or not strictly positive (> 0.0).
    #[error("Invalid propensity weight: {0} (must be finite and strictly positive > 0.0)")]
    InvalidPropensityWeight(f32),

    /// Non-conformity score is not finite (NaN or Infinity).
    #[error("Invalid non-conformity score: {0} (must be finite)")]
    InvalidNonConformityScore(f32),

    /// Threshold value is not finite (NaN or Infinity).
    #[error("Invalid threshold: {0} (must be finite)")]
    InvalidThreshold(f32),

    /// Learning rate is not finite or not strictly positive (> 0.0).
    #[error("Invalid learning rate: {0} (must be finite and strictly positive > 0.0)")]
    InvalidLearningRate(f32),
}

/// Trait for deterministic, online conformal calibration and threshold estimation.
pub trait ConformalCalibrator {
    /// Deterministically updates the threshold given a non-conformity score and propensity weight.
    ///
    /// # Parameters
    /// - `non_conformity_score`: The non-conformity score $S_t$ of the current sample.
    /// - `propensity_weight`: The propensity weight $w_t > 0$ correcting for covariate shift.
    ///
    /// # Returns
    /// - `Ok(true)` if the update altered the threshold value.
    /// - `Ok(false)` if the threshold value remained unchanged.
    /// - `Err(ConformalError)` if any input parameter is non-finite or non-positive.
    fn update(
        &mut self,
        non_conformity_score: f32,
        propensity_weight: f32,
    ) -> Result<bool, ConformalError>;

    /// Returns the currently valid threshold $\hat{q}$.
    fn threshold(&self) -> f32;
}

/// Adaptive Conformal Calibrator for quantile-based online threshold estimation under covariate shift.
///
/// Implements online stochastic gradient updates on the weighted pinball loss for target coverage
/// level $1 - \alpha$. When a non-conformity score exceeds the current threshold ($S_t > \hat{q}_t$),
/// the threshold increases by $\gamma \cdot w_t \cdot (1 - \alpha)$. Otherwise, it decreases by
/// $\gamma \cdot w_t \cdot \alpha$.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AdaptiveConformalCalibrator {
    alpha: f32,
    threshold: f32,
    learning_rate: f32,
    min_threshold: f32,
    max_threshold: f32,
    observation_count: u64,
    fingerprint: Option<ConfigFingerprint>,
}

impl AdaptiveConformalCalibrator {
    /// Default learning rate $\gamma$ for online quantile updates.
    pub const DEFAULT_LEARNING_RATE: f32 = 0.02;

    /// Default initial threshold $\hat{q}_0$.
    pub const DEFAULT_INITIAL_THRESHOLD: f32 = 0.5;

    /// Default minimum threshold bound.
    pub const DEFAULT_MIN_THRESHOLD: f32 = 0.0;

    /// Default maximum threshold bound.
    pub const DEFAULT_MAX_THRESHOLD: f32 = 1.0;

    /// Creates a new `AdaptiveConformalCalibrator` with target error rate `alpha` in `(0.0, 1.0)`.
    ///
    /// Uses default initial threshold (0.5) and default learning rate (0.02).
    pub fn new(alpha: f32) -> Result<Self, ConformalError> {
        Self::with_params(
            alpha,
            Self::DEFAULT_INITIAL_THRESHOLD,
            Self::DEFAULT_LEARNING_RATE,
        )
    }

    /// Creates an `AdaptiveConformalCalibrator` with custom parameters.
    pub fn with_params(
        alpha: f32,
        initial_threshold: f32,
        learning_rate: f32,
    ) -> Result<Self, ConformalError> {
        if !alpha.is_finite() || alpha <= 0.0 || alpha >= 1.0 {
            return Err(ConformalError::InvalidAlpha(alpha));
        }
        if !initial_threshold.is_finite() {
            return Err(ConformalError::InvalidThreshold(initial_threshold));
        }
        if !learning_rate.is_finite() || learning_rate <= 0.0 {
            return Err(ConformalError::InvalidLearningRate(learning_rate));
        }

        Ok(Self {
            alpha,
            threshold: initial_threshold,
            learning_rate,
            min_threshold: Self::DEFAULT_MIN_THRESHOLD,
            max_threshold: Self::DEFAULT_MAX_THRESHOLD,
            observation_count: 0,
            fingerprint: None,
        })
    }

    /// Sets bounding constraints `[min_threshold, max_threshold]` for the estimated threshold.
    pub fn with_bounds(
        mut self,
        min_threshold: f32,
        max_threshold: f32,
    ) -> Result<Self, ConformalError> {
        if !min_threshold.is_finite() {
            return Err(ConformalError::InvalidThreshold(min_threshold));
        }
        if !max_threshold.is_finite() {
            return Err(ConformalError::InvalidThreshold(max_threshold));
        }
        if min_threshold > max_threshold {
            return Err(ConformalError::InvalidThreshold(min_threshold));
        }
        self.min_threshold = min_threshold;
        self.max_threshold = max_threshold;
        self.threshold = self.threshold.clamp(min_threshold, max_threshold);
        Ok(self)
    }

    /// Returns the target error rate $\alpha$.
    pub fn alpha(&self) -> f32 {
        self.alpha
    }

    /// Returns the learning rate $\gamma$.
    pub fn learning_rate(&self) -> f32 {
        self.learning_rate
    }

    /// Returns the total number of observations processed.
    pub fn observation_count(&self) -> u64 {
        self.observation_count
    }

    /// Resets state if ConfigFingerprint changes (P8 Compliance).
    pub fn invalidate_on_config_change(&mut self, new_fingerprint: ConfigFingerprint) {
        if self.fingerprint.as_ref() != Some(&new_fingerprint) {
            tracing::warn!(
                old_fp = ?self.fingerprint,
                new_fp = ?new_fingerprint,
                obs_count = self.observation_count,
                "AdaptiveConformalCalibrator: ConfigFingerprint changed — resetting (P8)"
            );
            self.threshold =
                Self::DEFAULT_INITIAL_THRESHOLD.clamp(self.min_threshold, self.max_threshold);
            self.observation_count = 0;
            self.fingerprint = Some(new_fingerprint);
        }
    }
}

impl ConformalCalibrator for AdaptiveConformalCalibrator {
    fn update(
        &mut self,
        non_conformity_score: f32,
        propensity_weight: f32,
    ) -> Result<bool, ConformalError> {
        // Zero-Panic Invariants: Validate all inputs before touching state
        if !propensity_weight.is_finite() || propensity_weight <= 0.0 {
            return Err(ConformalError::InvalidPropensityWeight(propensity_weight));
        }
        if !non_conformity_score.is_finite() {
            return Err(ConformalError::InvalidNonConformityScore(
                non_conformity_score,
            ));
        }

        let step = if non_conformity_score > self.threshold {
            self.learning_rate * propensity_weight * (1.0 - self.alpha)
        } else {
            -self.learning_rate * propensity_weight * self.alpha
        };

        if !step.is_finite() {
            return Err(ConformalError::InvalidPropensityWeight(propensity_weight));
        }

        let raw_next = self.threshold + step;
        let next_threshold = raw_next.clamp(self.min_threshold, self.max_threshold);

        let changed = next_threshold != self.threshold;
        if changed {
            self.threshold = next_threshold;
        }

        self.observation_count += 1;
        Ok(changed)
    }

    fn threshold(&self) -> f32 {
        self.threshold
    }
}
