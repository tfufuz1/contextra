//! PPR Cost Model, Strategy Selection, and ShadowMode Calibration.

use contextra_types::{PprAlgorithm, PprConfig};
use parking_lot::Mutex;
use std::sync::Arc;

/// CSR Graph structural statistics used for PPR cost estimation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GraphStats {
    /// Number of active/committed nodes |V|.
    pub node_count: usize,
    /// Number of directed edges |E|.
    pub edge_count: usize,
    /// Average outgoing degree |E| / |V|.
    pub avg_degree: f64,
}

impl GraphStats {
    /// Creates a new `GraphStats` from explicit counts.
    pub fn new(node_count: usize, edge_count: usize) -> Self {
        let avg_degree = if node_count > 0 {
            edge_count as f64 / node_count as f64
        } else {
            0.0
        };
        Self {
            node_count,
            edge_count,
            avg_degree,
        }
    }
}

/// Cost estimate for PPR evaluation algorithms in estimated edge access units.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PprCostEstimate {
    /// Estimated edge accesses for Andersen-Chung-Lang ForwardPush: $O(1 / (\epsilon \cdot \alpha))$.
    pub forward_push: f64,
    /// Estimated edge accesses for Dense Power Iteration: $O(\text{iterations} \cdot (|E| + |V|))$.
    pub power_iteration: f64,
}

impl PprCostEstimate {
    /// Selects the cheapest algorithm based on estimated cost.
    ///
    /// Tie-break rule: If `forward_push == power_iteration`, `ForwardPush` is deterministically chosen.
    pub fn cheapest_strategy(&self) -> PprAlgorithm {
        if self.forward_push <= self.power_iteration {
            PprAlgorithm::ForwardPush
        } else {
            PprAlgorithm::DensePowerIteration
        }
    }
}

/// A sample recording estimated cost vs measured edge access counters (operations count, no wall-clock time).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PprCostSample {
    /// Algorithm evaluated.
    pub algorithm: PprAlgorithm,
    /// Estimated cost in edge access units.
    pub estimated_cost: f64,
    /// Measured edge accesses (counted operations, not wall-clock).
    pub measured_edge_accesses: u64,
}

/// ShadowMode calibrator that collects `PprCostSample` observations and derives
/// a recommended cost threshold multiplier without automatically applying it.
#[derive(Debug, Clone, Default)]
pub struct PprCostCalibrator {
    samples: Arc<Mutex<Vec<PprCostSample>>>,
}

impl PprCostCalibrator {
    /// Creates a new empty `PprCostCalibrator`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Records a new calibration sample comparing estimated cost vs measured edge accesses.
    pub fn record(&self, sample: PprCostSample) {
        let mut guard = self.samples.lock();
        guard.push(sample);
    }

    /// Returns the total number of recorded calibration samples.
    pub fn sample_count(&self) -> usize {
        self.samples.lock().len()
    }

    /// Derives a recommended cost multiplier (ratio of measured edge accesses to estimated cost).
    ///
    /// Returns `None` if no samples have been recorded.
    /// DOES NOT automatically apply the recommended factor.
    pub fn recommended_multiplier(&self) -> Option<f64> {
        self.evaluate_recommended_multiplier()
    }

    /// Evaluates the recommended cost multiplier across recorded samples.
    pub fn evaluate_recommended_multiplier(&self) -> Option<f64> {
        let guard = self.samples.lock();
        if guard.is_empty() {
            return None;
        }

        let mut total_ratio = 0.0f64;
        let mut valid_samples = 0usize;

        for sample in guard.iter() {
            if sample.estimated_cost > 0.0 {
                let ratio = sample.measured_edge_accesses as f64 / sample.estimated_cost;
                total_ratio += ratio;
                valid_samples += 1;
            }
        }

        if valid_samples == 0 {
            None
        } else {
            Some(total_ratio / valid_samples as f64)
        }
    }

    /// Clears all recorded samples.
    pub fn clear(&self) {
        self.samples.lock().clear();
    }
}

/// Estimates computational costs (in estimated edge accesses) for PPR algorithms.
///
/// # Fallback
/// If `stats` is `None` or `stats.node_count == 0`, uses the spec fallback threshold:
/// `seed_count <= 100 => ForwardPush`, otherwise `DensePowerIteration`.
pub fn estimate_ppr_cost(
    stats: Option<&GraphStats>,
    seed_count: usize,
    config: &PprConfig,
) -> (PprCostEstimate, PprAlgorithm) {
    let alpha = if config.damping_factor.is_nan()
        || config.damping_factor <= 0.0
        || config.damping_factor >= 1.0
    {
        0.15f64
    } else {
        (1.0f64 - config.damping_factor as f64).max(1e-4)
    };

    let epsilon = if config.convergence_epsilon.is_nan() || config.convergence_epsilon <= 0.0 {
        1e-6f64
    } else {
        (config.convergence_epsilon as f64).max(1e-12)
    };

    let max_iters = (config.max_iterations.min(1000) as usize) as f64;

    // Asymptotic estimated edge accesses
    let fp_cost = 1.0f64 / (epsilon * alpha);

    if let Some(st) = stats {
        if st.node_count > 0 {
            let pi_cost = max_iters * (st.edge_count as f64 + st.node_count as f64);
            let estimate = PprCostEstimate {
                forward_push: fp_cost,
                power_iteration: pi_cost,
            };
            let choice = estimate.cheapest_strategy();
            return (estimate, choice);
        }
    }

    // Fallback when stats are not available or node_count is 0
    let fallback_choice = if seed_count <= 100 {
        PprAlgorithm::ForwardPush
    } else {
        PprAlgorithm::DensePowerIteration
    };

    let estimate = PprCostEstimate {
        forward_push: fp_cost,
        power_iteration: match fallback_choice {
            PprAlgorithm::ForwardPush => fp_cost + 1.0,
            _ => fp_cost - 1.0,
        },
    };

    (estimate, fallback_choice)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_estimate_ppr_cost_tie_break_prefers_forward_push() {
        let estimate = PprCostEstimate {
            forward_push: 1000.0,
            power_iteration: 1000.0,
        };
        assert_eq!(estimate.cheapest_strategy(), PprAlgorithm::ForwardPush);
    }

    #[test]
    fn test_fallback_when_stats_none() {
        let config = PprConfig::default();
        let (_, choice_small) = estimate_ppr_cost(None, 50, &config);
        assert_eq!(choice_small, PprAlgorithm::ForwardPush);

        let (_, choice_large) = estimate_ppr_cost(None, 150, &config);
        assert_eq!(choice_large, PprAlgorithm::DensePowerIteration);
    }

    #[test]
    fn test_cost_estimation_sparse_vs_dense() {
        let config = PprConfig::default();
        // Small graph: ForwardPush estimated cost 1 / (1e-6 * 0.15) = 6,666,666.67
        // Small graph PI cost: 100 * (10 + 5) = 1,500
        let small_stats = GraphStats::new(5, 10);
        let (_, choice_small) = estimate_ppr_cost(Some(&small_stats), 1, &config);
        assert_eq!(choice_small, PprAlgorithm::DensePowerIteration);

        // Huge dense graph: PI cost: 100 * (100,000 + 10,000) = 11,000,000 > FP cost 6,666,666
        let large_stats = GraphStats::new(10_000, 100_000);
        let (_, choice_large) = estimate_ppr_cost(Some(&large_stats), 1, &config);
        assert_eq!(choice_large, PprAlgorithm::ForwardPush);
    }

    #[test]
    fn test_ppr_cost_calibrator() {
        let calibrator = PprCostCalibrator::new();
        assert_eq!(calibrator.recommended_multiplier(), None);

        calibrator.record(PprCostSample {
            algorithm: PprAlgorithm::ForwardPush,
            estimated_cost: 1000.0,
            measured_edge_accesses: 1500,
        });

        calibrator.record(PprCostSample {
            algorithm: PprAlgorithm::ForwardPush,
            estimated_cost: 2000.0,
            measured_edge_accesses: 1000,
        });

        // sample 1 ratio: 1500 / 1000 = 1.5
        // sample 2 ratio: 1000 / 2000 = 0.5
        // mean ratio = (1.5 + 0.5) / 2 = 1.0
        let mult = calibrator.recommended_multiplier().unwrap();
        assert!((mult - 1.0).abs() < 1e-6);
    }
}
