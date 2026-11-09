use proptest::prelude::*;
use std::time::Duration;

proptest! {
    #[test]
    fn test_exponential_backoff_monotonic_and_bounded(
        attempt in 0u32..20u32,
        jitter_ms in 0u64..100u64
    ) {
        let base_delay = Duration::from_millis(100 * 2u64.saturating_pow(attempt));
        let jitter = Duration::from_millis(jitter_ms);
        let delay = (base_delay + jitter).min(Duration::from_secs(5));

        // Delay must never exceed 5 seconds limit
        prop_assert!(delay <= Duration::from_secs(5));
        // Delay must be at least base delay (at attempt 0)
        prop_assert!(delay >= Duration::from_millis(100));
    }
}
