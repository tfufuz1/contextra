//! # Tests für MV-06, MV-10 und T-RESTORE-3: SeqAllocator-Invariante
//!
//! Prüft die strikte Monotonie der Sequenznummern über Restore-Zyklen:
//! next_seq = max(current_seq, ledger_floor) + 1 (I-20, MV-06).
//! Prüft checked-Arithmetik und Überlaufbehandlung ohne Wrap (MV-10).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_mvcc::error::MvccError;
use contextra_mvcc::seq_log::SeqAllocator;
use contextra_testmacros::ctx_test;
use proptest::prelude::*;

#[ctx_test(id = "T-TXN-1")]
fn mv_06_seq_allocator_next_seq_formula() {
    let allocator = SeqAllocator::new(10, 20);
    // max(10, 20) + 1 = 21
    let s1 = allocator.next_seq().expect("valid next seq");
    assert_eq!(s1, 21);
    assert_eq!(allocator.current_seq(), 21);

    // Jetzt ist current_seq = 21, ledger_floor = 20.
    // max(21, 20) + 1 = 22
    let s2 = allocator.next_seq().expect("valid next seq");
    assert_eq!(s2, 22);
}

#[ctx_test(id = "T-TXN-1")]
fn mv_10_seq_allocator_checked_overflow_error() {
    let allocator = SeqAllocator::new(u64::MAX, 0);
    // Darf niemals wrappen! Muss Err zurückgeben.
    let res = allocator.next_seq();
    assert!(res.is_err(), "Overflow must return Err, never wrap");
    match res.unwrap_err() {
        MvccError::InvalidSequence(seq) => assert_eq!(seq, u64::MAX),
        other => panic!("Unexpected error type: {:?}", other),
    }

    let allocator2 = SeqAllocator::new(0, u64::MAX);
    let res2 = allocator2.next_seq();
    assert!(res2.is_err(), "Overflow via ledger_floor must return Err");
}

#[ctx_test(id = "T-RESTORE-3")]
fn t_restore_3_strict_monotonic_across_restores() {
    let allocator = SeqAllocator::new(100, 100);
    assert_eq!(allocator.next_seq().expect("seq"), 101);
    assert_eq!(allocator.next_seq().expect("seq"), 102);

    // Simulierter Restore: Backup-Stand setzt current_seq zurück auf 50,
    // aber das Audit-/Restore-Ledger hat floor 150.
    allocator.reset_after_restore(50, 150);
    assert_eq!(allocator.current_seq(), 50);
    assert_eq!(allocator.ledger_floor(), 150);

    // Nächste Sequenz muss max(50, 150) + 1 = 151 sein!
    let s = allocator.next_seq().expect("seq");
    assert_eq!(s, 151);
    assert!(
        s > 102,
        "Sequenz muss über Restores hinweg streng monoton steigen"
    );
}

proptest! {
#[ctx_test(id = "T-RESTORE-3")]
    fn t_restore_3_proptest_monotonicity(
        initial_cur in 0u64..1_000_000,
        initial_floor in 0u64..1_000_000,
        restore_cur in 0u64..1_000_000,
        restore_floor in 0u64..1_000_000,
    ) {
        let allocator = SeqAllocator::new(initial_cur, initial_floor);
        let mut last_seq = allocator.current_seq();

        for _ in 0..10 {
            if let Ok(next) = allocator.next_seq() {
                prop_assert!(next > last_seq, "next={next} must be > last={last_seq}");
                last_seq = next;
            }
        }

        let effective_floor = initial_floor.max(restore_floor).max(last_seq);
        allocator.reset_after_restore(restore_cur, effective_floor);

        for _ in 0..10 {
            if let Ok(next) = allocator.next_seq() {
                prop_assert!(next > last_seq, "After restore: next={next} must be > last={last_seq}");
                last_seq = next;
            }
        }
    }
}
