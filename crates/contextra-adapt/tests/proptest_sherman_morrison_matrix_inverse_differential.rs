//! Differential property tests for Sherman-Morrison O(d²) matrix inverse recursion vs independent Gauss-Jordan reinversion (Spec §21, §13.2).

use contextra_adapt::bandit::{BanditImplementation, BanditProfileState};

/// Simple LCG pseudo-random number generator for self-contained property generation without external crates.
struct LcgRng {
    state: u64,
}

impl LcgRng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.state
    }

    fn next_f64_range(&mut self, low: f64, high: f64) -> f64 {
        let val = (self.next_u64() >> 11) as f64 / ((1u64 << 53) as f64);
        low + val * (high - low)
    }

    fn next_bool(&mut self) -> bool {
        (self.next_u64() & 1) == 1
    }

    fn next_usize_range(&mut self, low: usize, high: usize) -> usize {
        if low >= high {
            return low;
        }
        let range = (high - low + 1) as u64;
        low + (self.next_u64() % range) as usize
    }
}

/// Naive Gauss-Jordan matrix inversion in f64 for d x d square matrix in row-major order.
///
/// Implemented completely independently from production code (no reuse of fused_row_update / dot_product_f32).
/// Performs partial pivoting in f64 precision and returns `None` if matrix is singular (pivot < 1e-12).
fn naive_matrix_inverse_f64(a: &[f64], d: usize) -> Option<Vec<f32>> {
    assert_eq!(a.len(), d * d, "Input array must have d * d elements");

    let mut aug = vec![0.0f64; d * 2 * d];
    for i in 0..d {
        for j in 0..d {
            aug[i * 2 * d + j] = a[i * d + j];
        }
        aug[i * 2 * d + d + i] = 1.0;
    }

    for k in 0..d {
        let mut max_row = k;
        let mut max_val = aug[k * 2 * d + k].abs();
        for i in (k + 1)..d {
            let val = aug[i * 2 * d + k].abs();
            if val > max_val {
                max_val = val;
                max_row = i;
            }
        }

        if max_val < 1e-12 {
            return None;
        }

        if max_row != k {
            for j in 0..(2 * d) {
                aug.swap(k * 2 * d + j, max_row * 2 * d + j);
            }
        }

        let pivot = aug[k * 2 * d + k];
        for j in 0..(2 * d) {
            aug[k * 2 * d + j] /= pivot;
        }

        for i in 0..d {
            if i != k {
                let factor = aug[i * 2 * d + k];
                for j in 0..(2 * d) {
                    aug[i * 2 * d + j] -= factor * aug[k * 2 * d + j];
                }
            }
        }
    }

    let mut inv = vec![0.0f32; d * d];
    for i in 0..d {
        for j in 0..d {
            inv[i * d + j] = aug[i * 2 * d + d + j] as f32;
        }
    }
    Some(inv)
}

/// Property test: Sherman-Morrison recursive matrix inversion matches independent Gauss-Jordan reinversion
/// after every update within a 1e-3 float tolerance over 100 randomized trial sequences.
///
/// Tolerance Choice Rationale (Spec §21):
/// A generous tolerance of 1e-3 is chosen because Sherman-Morrison incremental matrix updates
/// and full Gauss-Jordan reinversions accumulate floating-point rounding errors along different
/// numerical pathways over multiple update iterations. A 1e-3 tolerance prevents false-positive
/// test failures from harmless FP divergence while strictly catching algorithmic or recursion bugs.
#[test]
fn prop_sherman_morrison_matches_naive_reinversion() {
    let mut rng = LcgRng::new(0xDEAD_BEEF_1234_5678);

    for case in 0..100 {
        let d = rng.next_usize_range(2, 6);
        let num_updates = rng.next_usize_range(1, 30);

        let mut state = BanditProfileState::cold_start(d, 0.5);
        state.implementation = BanditImplementation::ShermanMorrison;

        // Independent A matrix accumulator in f64 (initialized to Identity matrix I_d, matching cold_start inv_a = I_d)
        let mut a_matrix = vec![0.0f64; d * d];
        for i in 0..d {
            a_matrix[i * d + i] = 1.0;
        }

        for step in 0..num_updates {
            let mut x = vec![0.0f32; d];
            for j in 0..d {
                x[j] = rng.next_f64_range(-99.0, 99.0) as f32;
            }
            let r_outcome = rng.next_f64_range(-10.0, 10.0) as f32;
            let cost = rng.next_f64_range(0.0, 5.0) as f32;
            let is_cloud = rng.next_bool();

            // Determine effective gamma prior to update
            let effective_gamma = if state.drift_steps_remaining > 0 {
                state.drift_gamma
            } else {
                state.gamma
            };

            // Perform bandit update
            let update_res = state.update(&x, r_outcome, cost, is_cloud);
            if update_res.is_err() {
                // Precision matrix drift or dimension mismatch reached
                break;
            }

            // Independent A matrix recursion: A_new = gamma * A_old + x * x^T
            for i in 0..d {
                for j in 0..d {
                    a_matrix[i * d + j] = (effective_gamma as f64) * a_matrix[i * d + j]
                        + (x[i] as f64) * (x[j] as f64);
                }
            }

            // Re-invert independent A matrix with Gauss-Jordan
            if let Some(naive_inv) = naive_matrix_inverse_f64(&a_matrix, d) {
                for i in 0..(d * d) {
                    let diff = (state.inv_a[i] - naive_inv[i]).abs();
                    assert!(
                        diff < 1e-3,
                        "Case {case} Step {step}: Sherman-Morrison inv_a[{i}] ({}) deviated from naive reinversion ({}) by {} (> 1e-3) at dim {}",
                        state.inv_a[i], naive_inv[i], diff, d
                    );
                }
            }
        }
    }
}

/// Targeted test checking error growth rate on nearly collinear feature vectors over 50 updates.
///
/// Verifies that accumulated numerical discrepancy between Sherman-Morrison and full reinversion
/// does not grow monotonically unbounded (error growth factor between checkpoints <= 5).
#[test]
fn test_sherman_morrison_collinear_stability_bounded_growth() {
    let d = 3;
    let mut state = BanditProfileState::cold_start(d, 0.5);
    state.implementation = BanditImplementation::ShermanMorrison;
    state.gamma = 0.9;

    let mut a_matrix = vec![0.0f64; d * d];
    for i in 0..d {
        a_matrix[i * d + i] = 1.0;
    }

    let mut err_10 = 0.0f32;
    let mut err_25 = 0.0f32;
    let mut err_50 = 0.0f32;

    for step in 1..=50 {
        // Moderate collinear vector scenario
        let x = vec![0.5f32, 0.48f32, 0.52f32];

        let effective_gamma = if state.drift_steps_remaining > 0 {
            state.drift_gamma
        } else {
            state.gamma
        };

        state
            .update(&x, 1.0, 0.1, false)
            .expect("Update must succeed");

        for i in 0..d {
            for j in 0..d {
                a_matrix[i * d + j] = (effective_gamma as f64) * a_matrix[i * d + j]
                    + (x[i] as f64) * (x[j] as f64);
            }
        }

        let naive_inv = naive_matrix_inverse_f64(&a_matrix, d)
            .expect("Naive reinversion must succeed for collinear step");

        let mut max_diff = 0.0f32;
        for i in 0..(d * d) {
            let diff = (state.inv_a[i] - naive_inv[i]).abs();
            if diff > max_diff {
                max_diff = diff;
            }
        }

        if step == 10 {
            err_10 = max_diff;
        } else if step == 25 {
            err_25 = max_diff;
        } else if step == 50 {
            err_50 = max_diff;
        }
    }

    println!("err_10: {err_10}, err_25: {err_25}, err_50: {err_50}");

    let max_base_10 = err_10.max(1e-4);
    let max_base_25 = err_25.max(1e-4);

    assert!(
        err_25 <= 5.0 * max_base_10,
        "Error growth from step 10 ({err_10}) to step 25 ({err_25}) exceeded factor 5"
    );
    assert!(
        err_50 <= 5.0 * max_base_25,
        "Error growth from step 25 ({err_25}) to step 50 ({err_50}) exceeded factor 5"
    );
}
