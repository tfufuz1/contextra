//! Platt-Scaling (Logistic Calibration): `sigmoid(A * logit + B)`.

use contextra_types::ConfigFingerprint;

/// Platt Scaler for parametric logit/score calibration.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PlattScaler {
    a: f32,
    b: f32,
    fingerprint: Option<ConfigFingerprint>,
}

impl Default for PlattScaler {
    fn default() -> Self {
        Self::identity()
    }
}

impl PlattScaler {
    /// Creates a new `PlattScaler` with specified parameters `a` and `b`.
    pub fn new(a: f32, b: f32) -> Self {
        Self {
            a,
            b,
            fingerprint: None,
        }
    }

    /// Uncalibrated default fallback ($A=1.0, B=0.0$).
    pub fn identity() -> Self {
        Self {
            a: 1.0,
            b: 0.0,
            fingerprint: None,
        }
    }

    /// Checks if this scaler is identical to uncalibrated default ($A=1.0, B=0.0$).
    pub fn is_identity(&self) -> bool {
        (self.a - 1.0).abs() < f32::EPSILON && self.b.abs() < f32::EPSILON
    }

    /// Checks if model has been fitted.
    pub fn is_fitted(&self) -> bool {
        !self.is_identity()
    }

    /// Returns current parameters `(a, b)`.
    pub fn params(&self) -> (f32, f32) {
        (self.a, self.b)
    }

    /// Calculates calibrated probability for raw score.
    pub fn predict(&self, raw_score: f32) -> f32 {
        self.transform(raw_score)
    }

    /// Alias for `transform(logit)`.
    pub fn apply(&self, logit: f32) -> f32 {
        self.transform(logit)
    }

    /// Applies Platt Scaling `sigmoid(A * logit + B)`.
    pub fn transform(&self, logit: f32) -> f32 {
        if logit.is_nan() {
            return 0.5;
        }
        let z = self.a * logit + self.b;
        1.0 / (1.0 + (-z).exp())
    }

    /// Resets scaler to identity if ConfigFingerprint changes (P8 Compliance).
    pub fn invalidate_on_config_change(&mut self, new_fingerprint: ConfigFingerprint) {
        if self.fingerprint.as_ref() != Some(&new_fingerprint) {
            tracing::warn!(
                old_fp = ?self.fingerprint,
                new_fp = ?new_fingerprint,
                "PlattScaler: ConfigFingerprint changed — resetting to identity (P8)"
            );
            self.a = 1.0;
            self.b = 0.0;
            self.fingerprint = Some(new_fingerprint);
        }
    }

    /// Fits parameters `A` and `B` via Negative Log-Likelihood minimization with L2 regularization
    /// and target smoothing (Platt, 1999).
    pub fn fit(observations: &[(f32, bool)]) -> Self {
        let valid_obs: Vec<(f32, bool)> = observations
            .iter()
            .copied()
            .filter(|(logit, _)| logit.is_finite())
            .collect();

        if valid_obs.is_empty() {
            return Self::identity();
        }

        let pos_count = valid_obs.iter().filter(|(_, is_rel)| *is_rel).count();
        let neg_count = valid_obs.len() - pos_count;

        let t_pos = (pos_count as f32 + 1.0) / (pos_count as f32 + 2.0);
        let t_neg = 1.0 / (neg_count as f32 + 2.0);

        let mut a = 1.0f32;
        let mut b = 0.0f32;
        let mut lr = 0.05f32;
        let iterations = 300;
        let l2_reg = 0.001f32;

        for _ in 0..iterations {
            let mut grad_a = 0.0f32;
            let mut grad_b = 0.0f32;

            for &(logit, is_rel) in &valid_obs {
                let target = if is_rel { t_pos } else { t_neg };
                let z = a * logit + b;
                let p = 1.0 / (1.0 + (-z).exp());
                let err = p - target;

                grad_a += err * logit;
                grad_b += err;
            }

            let n = valid_obs.len() as f32;
            grad_a = grad_a / n + l2_reg * (a - 1.0);
            grad_b = grad_b / n + l2_reg * b;

            let grad_norm = (grad_a * grad_a + grad_b * grad_b).sqrt();
            if grad_norm > 10.0 {
                grad_a = (grad_a / grad_norm) * 10.0;
                grad_b = (grad_b / grad_norm) * 10.0;
            }

            a -= lr * grad_a;
            b -= lr * grad_b;

            lr *= 0.995;
        }

        if !a.is_finite() || !b.is_finite() {
            return Self::identity();
        }

        Self {
            a,
            b,
            fingerprint: None,
        }
    }
}
