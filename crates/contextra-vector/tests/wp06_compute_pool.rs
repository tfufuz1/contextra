// FILE-CONTEXT
// ZWECK: Härtungstests für ComputePool (Determinismus, Threadbegrenzung, Drop-Verhalten).
// INVARIANTEN: Deterministische Ausführung; Drop beendet alle Threads sauber ohne Lecks.
// STAND: 2026-11-12

use contextra_vector::compute_pool::ComputePool;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[test]
fn test_compute_pool_deterministic_ordering_independent_of_threads() {
    let input_data: Vec<usize> = (0..100).collect();

    // 1-Worker Ausführung
    let pool_1 = ComputePool::new(1);
    let mut handles_1 = Vec::new();
    for &val in &input_data {
        handles_1.push(pool_1.spawn(move || val * 2));
    }
    let res_1: Vec<usize> = handles_1.into_iter().map(|h| h.join().unwrap()).collect();

    // 8-Worker Ausführung
    let pool_8 = ComputePool::new(8);
    let mut handles_8 = Vec::new();
    for &val in &input_data {
        handles_8.push(pool_8.spawn(move || val * 2));
    }
    let res_8: Vec<usize> = handles_8.into_iter().map(|h| h.join().unwrap()).collect();

    assert_eq!(res_1, res_8);
    assert_eq!(res_1.len(), 100);
}

#[test]
fn test_compute_pool_drop_joins_all_threads_cleanly() {
    let counter = Arc::new(AtomicUsize::new(0));

    {
        let pool = ComputePool::new(4);
        assert_eq!(pool.max_workers(), 4);

        for _ in 0..20 {
            let cnt = Arc::clone(&counter);
            pool.execute(move || {
                cnt.fetch_add(1, Ordering::SeqCst);
            });
        }
        // pool wird hier gedroppt, was alle Worker-Threads joinen muss.
    }

    // Nach dem Drop müssen alle 20 Jobs abgearbeitet sein, da Drop deterministisch auf Beendigung wartet.
    assert_eq!(counter.load(Ordering::SeqCst), 20);
}

#[test]
fn test_compute_pool_panic_propagates_as_vector_error() {
    let pool = ComputePool::new(2);
    let handle = pool.spawn(|| {
        panic!("controlled panic in task");
    });

    let res = handle.join();
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert!(format!("{err}").contains("controlled panic in task"));
}
