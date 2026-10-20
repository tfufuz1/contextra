use contextra_ports::IdGen;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

struct SequentialIdGen {
    next: AtomicU64,
}

impl SequentialIdGen {
    fn new(start: u64) -> Self {
        Self {
            next: AtomicU64::new(start),
        }
    }
}

impl IdGen for SequentialIdGen {
    fn next_id(&self) -> u64 {
        self.next.fetch_add(1, Ordering::SeqCst)
    }
}

#[test]
fn test_agent_id_gen_determinism() {
    let seed = 42u64;
    let gen1 = Arc::new(SequentialIdGen::new(seed));
    let gen2 = Arc::new(SequentialIdGen::new(seed));

    let seq1: Vec<u64> = (0..100).map(|_| gen1.next_id()).collect();
    let seq2: Vec<u64> = (0..100).map(|_| gen2.next_id()).collect();

    assert_eq!(
        seq1, seq2,
        "Two IdGen instances with identical seeds must produce identical ID sequences"
    );
}
