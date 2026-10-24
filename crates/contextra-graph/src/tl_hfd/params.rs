//! Configuration parameters for Thresholded Local Hyper-Flow Diffusion (TL-HFD, Spec §21.1).

use super::error::TlHfdError;

/// Configuration parameters for Thresholded Local Hyper-Flow Diffusion (TL-HFD).
///
/// # Complexity
/// Time complexity of parameter validation is $O(1)$.
#[derive(Debug, Clone, PartialEq)]
pub struct TlHfdParams {
    /// Regularization parameter $\sigma > 0$ controlling diffusion scale (default: 0.1).
    pub sigma: f32,
    /// Seed injection multiplier $\delta \ge 2.0$ (default: 2.0).
    pub delta: f32,
    /// Boundary conductance exponent $\gamma \ge 0$ (default: 1.0).
    pub gamma: f32,
    /// Maximum number of diffusion iterations $\ge 1$ (default: 50).
    pub max_iterations: u32,
    /// Maximum number of boundary nodes to activate per iteration $\ge 1$ (default: 10).
    pub max_top_k_expansion: usize,
    /// Maximum hyperedge participant size before truncation $\ge 2$ (default: 64).
    pub max_hyperedge_sort_size: usize,
}

impl Default for TlHfdParams {
    fn default() -> Self {
        Self {
            sigma: 0.1,
            delta: 2.0,
            gamma: 1.0,
            max_iterations: 50,
            max_top_k_expansion: 10,
            max_hyperedge_sort_size: 64,
        }
    }
}

impl TlHfdParams {
    /// Validates configuration parameters against spec constraints.
    ///
    /// # Errors
    /// Returns [`TlHfdError::InvalidParameter`] if any parameter violates its domain boundary or is non-finite.
    ///
    /// # Complexity
    /// Time complexity $O(1)$.
    pub fn validate(&self) -> Result<(), TlHfdError> {
        if !self.sigma.is_finite() || self.sigma <= 0.0 {
            return Err(TlHfdError::InvalidParameter(format!(
                "sigma must be positive and finite, got {}",
                self.sigma
            )));
        }
        if !self.delta.is_finite() || self.delta < 2.0 {
            return Err(TlHfdError::InvalidParameter(format!(
                "delta must be >= 2.0 and finite, got {}",
                self.delta
            )));
        }
        if !self.gamma.is_finite() || self.gamma < 0.0 {
            return Err(TlHfdError::InvalidParameter(format!(
                "gamma must be >= 0.0 and finite, got {}",
                self.gamma
            )));
        }
        if self.max_iterations < 1 {
            return Err(TlHfdError::InvalidParameter(format!(
                "max_iterations must be >= 1, got {}",
                self.max_iterations
            )));
        }
        if self.max_top_k_expansion < 1 {
            return Err(TlHfdError::InvalidParameter(format!(
                "max_top_k_expansion must be >= 1, got {}",
                self.max_top_k_expansion
            )));
        }
        if self.max_hyperedge_sort_size < 2 {
            return Err(TlHfdError::InvalidParameter(format!(
                "max_hyperedge_sort_size must be >= 2, got {}",
                self.max_hyperedge_sort_size
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_params_valid() {
        let params = TlHfdParams::default();
        assert!(params.validate().is_ok());
    }

    #[test]
    fn test_invalid_params() {
        let p = TlHfdParams {
            sigma: 0.0,
            ..Default::default()
        };
        assert!(p.validate().is_err());

        let p = TlHfdParams {
            delta: 1.99,
            ..Default::default()
        };
        assert!(p.validate().is_err());

        let p = TlHfdParams {
            gamma: -0.1,
            ..Default::default()
        };
        assert!(p.validate().is_err());

        let p = TlHfdParams {
            max_iterations: 0,
            ..Default::default()
        };
        assert!(p.validate().is_err());

        let p = TlHfdParams {
            max_top_k_expansion: 0,
            ..Default::default()
        };
        assert!(p.validate().is_err());

        let p = TlHfdParams {
            max_hyperedge_sort_size: 1,
            ..Default::default()
        };
        assert!(p.validate().is_err());
    }
}
