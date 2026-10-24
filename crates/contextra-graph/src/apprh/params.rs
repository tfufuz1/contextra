//! Configuration parameters for Averaging-based Personalized PageRank for Hypergraphs (APPRH).

use super::error::ApprhError;
use crate::path_rag::PprParams;

/// Configuration parameters for Averaging-based Personalized PageRank for Hypergraphs (APPRH).
///
/// # Complexity
/// Time complexity of parameter validation is $O(1)$.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprhParams {
    /// Base PPR parameters ($\alpha$ teleport, $\epsilon$ convergence threshold).
    pub ppr: PprParams,
    /// Hyperedge decay factor ($\beta \in (0.0, 1.0]$, default: 0.85).
    pub hyperedge_decay_factor: f32,
    /// Maximum push iterations $\ge 1$ (default: 10_000).
    pub max_iterations: u32,
}

impl Default for ApprhParams {
    fn default() -> Self {
        Self {
            ppr: PprParams::default(),
            hyperedge_decay_factor: 0.85,
            max_iterations: 10_000,
        }
    }
}

impl ApprhParams {
    /// Validates configuration parameters against domain constraints.
    ///
    /// # Errors
    /// Returns [`ApprhError::InvalidParameter`] if any parameter violates domain boundaries or is non-finite.
    ///
    /// # Complexity
    /// Time complexity $O(1)$.
    pub fn validate(&self) -> Result<(), ApprhError> {
        if !self.ppr.alpha.is_finite() || self.ppr.alpha <= 0.0 || self.ppr.alpha >= 1.0 {
            return Err(ApprhError::InvalidParameter(format!(
                "alpha must be in (0.0, 1.0) and finite, got {}",
                self.ppr.alpha
            )));
        }
        if !self.ppr.epsilon.is_finite() || self.ppr.epsilon <= 0.0 {
            return Err(ApprhError::InvalidParameter(format!(
                "epsilon must be positive and finite, got {}",
                self.ppr.epsilon
            )));
        }
        if !self.hyperedge_decay_factor.is_finite()
            || self.hyperedge_decay_factor <= 0.0
            || self.hyperedge_decay_factor > 1.0
        {
            return Err(ApprhError::InvalidParameter(format!(
                "hyperedge_decay_factor must be in (0.0, 1.0] and finite, got {}",
                self.hyperedge_decay_factor
            )));
        }
        if self.max_iterations < 1 {
            return Err(ApprhError::InvalidParameter(format!(
                "max_iterations must be >= 1, got {}",
                self.max_iterations
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
        let params = ApprhParams::default();
        assert!(params.validate().is_ok());
    }

    #[test]
    fn test_invalid_params() {
        let p = ApprhParams {
            ppr: PprParams {
                alpha: 0.0,
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(p.validate().is_err());

        let p = ApprhParams {
            ppr: PprParams {
                epsilon: -1e-4,
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(p.validate().is_err());

        let p = ApprhParams {
            hyperedge_decay_factor: 1.5,
            ..Default::default()
        };
        assert!(p.validate().is_err());

        let p = ApprhParams {
            max_iterations: 0,
            ..Default::default()
        };
        assert!(p.validate().is_err());
    }
}
