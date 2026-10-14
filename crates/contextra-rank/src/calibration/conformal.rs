//! Conformal Calibration and Adaptive Conformal Prediction with Propensity Weighting.
//!
//! Provides quantile-based online threshold estimation for target coverage guarantees
//! under covariate shift via propensity weighting.

use contextra_types::ConfigFingerprint;
use thiserror::Error;

/// Minimum number of calibration observations required to exit cold-start ($n_{min} = 30$).
pub const CONFORMAL_MIN_CALIBRATION_SAMPLES: usize = 30;

/// Default step-size parameter $\gamma = 2^{-8} = 1/256$ for Adaptive Conformal Inference (ACI).
pub const ACI_DEFAULT_GAMMA: f32 = 1.0 / 256.0;

/// Result of a Split Conformal threshold calculation.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SplitConformalOutcome {
    /// Calculated quantile threshold $\hat{q}$ or $+\infty$.
    pub threshold: f32,
    /// Whether enough samples existed ($n \ge 30$) to calibrate the threshold.
    pub is_calibrated: bool,
    /// Total calibration sample count $n$.
    pub sample_count: usize,
}

/// Computes the split conformal quantile threshold $\hat{q}$ given non-conformity scores $s_i = 1 - f(d_i)$.
///
/// Implements Spec §9.3:
/// - Input scores must be finite.
/// - Cold start: $n < 30 \implies \hat{q} = +\infty$ and `is_calibrated = false`.
/// - Index calculation: $k = \lceil (n + 1)(1 - \alpha) \rceil$.
/// - If $k > n \implies \hat{q} = +\infty$.
/// - Else $\hat{q} = s_{(k)}$ (the $k$-th smallest score, 1-based index).
pub fn compute_split_conformal_threshold(
    scores: &[f32],
    alpha: f32,
) -> Result<SplitConformalOutcome, ConformalError> {
    if !alpha.is_finite() || alpha <= 0.0 || alpha >= 1.0 {
        return Err(ConformalError::InvalidAlpha(alpha));
    }

    for &s in scores {
        if !s.is_finite() {
            return Err(ConformalError::InvalidNonConformityScore(s));
        }
    }

    let n = scores.len();
    if n < CONFORMAL_MIN_CALIBRATION_SAMPLES {
        return Ok(SplitConformalOutcome {
            threshold: f32::INFINITY,
            is_calibrated: false,
            sample_count: n,
        });
    }

    let mut sorted_scores = scores.to_vec();
    sorted_scores.sort_by(|a, b| a.total_cmp(b));

    let k_ceil = ((n as f64 + 1.0) * (1.0 - alpha as f64)).ceil() as usize;

    let threshold = if k_ceil > n {
        f32::INFINITY
    } else {
        sorted_scores[k_ceil - 1]
    };

    Ok(SplitConformalOutcome {
        threshold,
        is_calibrated: true,
        sample_count: n,
    })
}

/// Adaptive Conformal Inference (ACI) state tracking dynamic miscoverage rate $\alpha_t$ under drift (§9.3).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AciState {
    alpha_target: f32,
    alpha_current: f32,
    gamma: f32,
    step_count: u64,
}

impl AciState {
    /// Creates a new `AciState` with target error rate $\alpha$ and step size $\gamma = 2^{-8}$.
    pub fn new(alpha: f32) -> Result<Self, ConformalError> {
        Self::with_gamma(alpha, ACI_DEFAULT_GAMMA)
    }

    /// Creates an `AciState` with explicit step size $\gamma$.
    pub fn with_gamma(alpha: f32, gamma: f32) -> Result<Self, ConformalError> {
        if !alpha.is_finite() || alpha <= 0.0 || alpha >= 1.0 {
            return Err(ConformalError::InvalidAlpha(alpha));
        }
        if !gamma.is_finite() || gamma <= 0.0 {
            return Err(ConformalError::InvalidLearningRate(gamma));
        }
        Ok(Self {
            alpha_target: alpha,
            alpha_current: alpha,
            gamma,
            step_count: 0,
        })
    }

    /// Returns current active error rate $\alpha_t$.
    pub fn current_alpha(&self) -> f32 {
        self.alpha_current
    }

    /// Returns target error rate $\alpha$.
    pub fn target_alpha(&self) -> f32 {
        self.alpha_target
    }

    /// Returns step size $\gamma$.
    pub fn gamma(&self) -> f32 {
        self.gamma
    }

    /// Returns total ACI update step count.
    pub fn step_count(&self) -> u64 {
        self.step_count
    }

    /// Updates $\alpha_t$ based on coverage error $err_t \in \{0.0, 1.0\}$:
    /// $\alpha_{t+1} = \text{clamp}(\alpha_t + \gamma \cdot (\alpha - err_t), 0.0, 1.0)$.
    pub fn update(&mut self, err_t: f32) -> Result<f32, ConformalError> {
        if !err_t.is_finite() || !(0.0..=1.0).contains(&err_t) {
            return Err(ConformalError::InvalidNonConformityScore(err_t));
        }

        let next = self.alpha_current + self.gamma * (self.alpha_target - err_t);
        self.alpha_current = next.clamp(0.0, 1.0);
        self.step_count += 1;
        Ok(self.alpha_current)
    }
}

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

    /// Creates a bounded `AdaptiveConformalCalibrator` with custom miscoverage level `alpha` and explicit threshold bounds.
    pub fn bounded(
        alpha: f32,
        min_threshold: f32,
        max_threshold: f32,
    ) -> Result<Self, ConformalError> {
        let calibrator = Self::new(alpha)?;
        calibrator.with_bounds(min_threshold, max_threshold)
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

        let calibrator = Self {
            alpha,
            threshold: initial_threshold,
            learning_rate,
            min_threshold: Self::DEFAULT_MIN_THRESHOLD,
            max_threshold: Self::DEFAULT_MAX_THRESHOLD,
            observation_count: 0,
            fingerprint: None,
        };
        calibrator.with_bounds(Self::DEFAULT_MIN_THRESHOLD, Self::DEFAULT_MAX_THRESHOLD)
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
