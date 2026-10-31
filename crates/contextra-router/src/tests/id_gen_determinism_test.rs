//! # Tests für die deterministische Generierung von Entscheidungs-IDs
//!
//! **Ring:** Ring 3 (Applikations- und Testschicht).
//! **Zweck:** Tests für die deterministische Generierung von Entscheidungs-IDs.

// ANCHOR[contextra-router:TST-IDG-001]
#![allow(
    clippy::indexing_slicing,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::cast_possible_truncation,
    clippy::disallowed_methods,
    clippy::disallowed_types,
    deprecated
)]
use contextra_ports::{IdGen, SequentialIdGen};
use std::sync::Arc;

// ANCHOR[contextra-router:TST-IDG-T01]
// Test `test_router_id_gen_determinism`: Prüft das erwartete Verhalten und Invarianten für tests für die deterministische generierung von entscheidungs-ids.
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
