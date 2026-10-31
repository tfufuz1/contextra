#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::indexing_slicing,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::await_holding_lock
)]
//! # Integrationstest: Determinismus der ID-Generierung
//!
//! Prüft die deterministische Vergabe interner Checkpoint-Transaktions-IDs.

use contextra_ports::{IdGen, SequentialIdGen};
use std::sync::Arc;

// Testet: test checkpoint id gen determinism.
#[test]
fn test_checkpoint_id_gen_determinism() {
    let seed = 100u64;
    let gen1 = Arc::new(SequentialIdGen::new(seed));
    let gen2 = Arc::new(SequentialIdGen::new(seed));

    let seq1: Vec<u64> = (0..100).map(|_| gen1.next_id()).collect();
    let seq2: Vec<u64> = (0..100).map(|_| gen2.next_id()).collect();

    assert_eq!(
        seq1, seq2,
        "Two IdGen instances with identical seeds must produce identical ID sequences"
    );
}
