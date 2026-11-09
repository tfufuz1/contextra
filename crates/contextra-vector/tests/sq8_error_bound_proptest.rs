// FILE-CONTEXT
// ZWECK: Proptest für die mathematische Fehlergrenze von SQ8 (INV-SQ8-ERROR-BOUND-1).

use contextra_vector::quantize::{ScalarQuantizer, SQ8_MAX_ERROR_FACTOR_PER_DIM};
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_sq8_reconstruction_error_bound_proptest(
        dim in 1usize..32,
        min_val in -100.0f32..0.0f32,
        range in 0.1f32..200.0f32,
        t in 0.0f32..1.0f32
    ) {
        let max_val = min_val + range;
        let batch_vec1 = vec![min_val; dim];
        let batch_vec2 = vec![max_val; dim];
        let batch = [batch_vec1.as_slice(), batch_vec2.as_slice()];

        let sq = ScalarQuantizer::try_train(&batch, dim).unwrap();

        // Target value strictly inside [min_val, max_val]
        let val = min_val + range * t;
        let test_vec = vec![val; dim];

        let q = sq.quantize(&test_vec).unwrap();
        let deq = sq.dequantize(&q).unwrap();

        for i in 0..dim {
            let max_bound = sq.max_reconstruction_error_per_dim(i).unwrap();
            let expected_bound = (sq.maxes()[i] - sq.mins()[i]) * SQ8_MAX_ERROR_FACTOR_PER_DIM;
            prop_assert!((max_bound - expected_bound).abs() < 1e-6);

            let actual_err = (test_vec[i] - deq[i]).abs();
            prop_assert!(actual_err <= max_bound + 1e-4, "Dimension {} error {} exceeded bound {}", i, actual_err, max_bound);
        }
    }
}
