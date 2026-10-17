//! Property test for LinUCB anti-windup trace capping under one-sided contexts.

#![cfg(feature = "bandit-routing")]

use contextra_adapt::BanditProfileState;
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_trace_capped_under_onesided_contexts(
        num_updates in 100usize..500usize,
        max_trace in 10.0f64..100.0f64,
    ) {
        let d = 4;
        let mut state = BanditProfileState::cold_start(d, 0.5);
        state.max_trace = max_trace;
        state.gamma = 0.999; // Minimal discounting to push trace growth in unexcited directions

        // Context vector that only excites dimension 0
        let x = vec![1.0f32, 0.0, 0.0, 0.0];

        for _ in 0..num_updates {
            let res = state.update(&x, 1.0, 0.0, false);
            prop_assert!(res.is_ok());

            let mut trace = 0.0f64;
            for i in 0..d {
                trace += state.inv_a[i * d + i] as f64;
            }

            // Trace must strictly not exceed max_trace + float epsilon tolerance
            prop_assert!(
                trace <= max_trace + 1e-4,
                "Trace {} exceeded max_trace {}", trace, max_trace
            );
        }
    }
}
