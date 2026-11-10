//! Differential test for unix_ms against frozen reference implementations in text, graph, and vector

use contextra_env::{unix_ms, Clock, ManualClock};

// text/contract.rs:16
pub trait TextClock: Send + Sync {
    fn now_unix_ms(&self) -> u64;
}

// graph/contract.rs:15
pub trait GraphClock: Send + Sync {
    fn now_unix_ms(&self) -> u64;
}

// vector/contract.rs:41
pub trait VectorClock: Send + Sync {
    fn now_unix_ms(&self) -> u64;
}

// Reference implementations as found in legacy adapters
struct TextAdapter<C: Clock>(C);
impl<C: Clock> TextClock for TextAdapter<C> {
    fn now_unix_ms(&self) -> u64 {
        self.0.now_unix_nanos() / 1_000_000
    }
}

struct GraphAdapter<C: Clock>(C);
impl<C: Clock> GraphClock for GraphAdapter<C> {
    fn now_unix_ms(&self) -> u64 {
        self.0.now_unix_nanos() / 1_000_000
    }
}

struct VectorAdapter<C: Clock>(C);
impl<C: Clock> VectorClock for VectorAdapter<C> {
    fn now_unix_ms(&self) -> u64 {
        self.0.now_unix_nanos() / 1_000_000
    }
}

#[test]
fn test_unix_ms_differential_parity() {
    let test_nanos = vec![
        0u64,
        999_999u64,
        1_000_000u64,
        1_000_001u64,
        1_500_000_000u64,
        u64::MAX - (u64::MAX % 1_000_000),
    ];

    for nanos in test_nanos {
        let clock = ManualClock::new(nanos, nanos);
        let actual = unix_ms(&clock);

        let text_res = TextAdapter(ManualClock::new(nanos, nanos)).now_unix_ms();
        let graph_res = GraphAdapter(ManualClock::new(nanos, nanos)).now_unix_ms();
        let vector_res = VectorAdapter(ManualClock::new(nanos, nanos)).now_unix_ms();

        assert_eq!(actual, text_res, "Mismatch with text at {} ns", nanos);
        assert_eq!(actual, graph_res, "Mismatch with graph at {} ns", nanos);
        assert_eq!(actual, vector_res, "Mismatch with vector at {} ns", nanos);
    }
}
