// FILE-CONTEXT
// ZWECK: Layer-4 EgressGuard (Re-export from contextra-privacy)

pub use contextra_privacy::egress_guard::{
    EgressGuard, TextSearchEngine, TextSearchResult, DEFAULT_EGRESS_GUARD_MIN_BYTES,
    DEFAULT_EGRESS_GUARD_THRESHOLD, DEFAULT_EGRESS_GUARD_TIMEOUT,
};

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_egress_guard_defaults() {
        assert_eq!(DEFAULT_EGRESS_GUARD_TIMEOUT, Duration::from_millis(200));
        assert_eq!(DEFAULT_EGRESS_GUARD_THRESHOLD, 0.85);
        assert_eq!(DEFAULT_EGRESS_GUARD_MIN_BYTES, 128);
    }
}
