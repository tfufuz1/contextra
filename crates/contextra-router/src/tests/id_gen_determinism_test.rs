use contextra_ports::{IdGen, SequentialIdGen};
use std::sync::Arc;

#[test]
fn test_router_id_gen_determinism() {
    let seed = 777u64;
    let gen1 = Arc::new(SequentialIdGen::new(seed));
    let gen2 = Arc::new(SequentialIdGen::new(seed));

    let seq1: Vec<u64> = (0..100).map(|_| gen1.next_id()).collect();
    let seq2: Vec<u64> = (0..100).map(|_| gen2.next_id()).collect();

    assert_eq!(
        seq1, seq2,
        "Two IdGen instances with identical seeds must produce identical ID sequences"
    );
}
