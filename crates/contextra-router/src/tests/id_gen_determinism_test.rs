use crate::DecisionIdGenerator;
use std::sync::Arc;

#[test]
fn test_router_id_gen_determinism() {
    let seed = 777u64;
    let gen1 = Arc::new(DecisionIdGenerator::new(seed));
    let gen2 = Arc::new(DecisionIdGenerator::new(seed));

    let seq1: Vec<u64> = (0..100).map(|_| gen1.next().inner()).collect();
    let seq2: Vec<u64> = (0..100).map(|_| gen2.next().inner()).collect();

    assert_eq!(
        seq1, seq2,
        "Two DecisionIdGenerator instances with identical seeds must produce identical ID sequences"
    );
}
