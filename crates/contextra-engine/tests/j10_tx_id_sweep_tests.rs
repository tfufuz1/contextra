use contextra_types::{ContextraError, TxId};
use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Barrier};
use std::thread;

/// Invariant AGT-GRAPH-001:
/// Valid TxId values must originate strictly from either:
/// 1) Collection sequence range: `[0, MAX_COLLECTION_SEQUENCE]` (`0` is TxId::INVALID)
/// 2) Internal system range: `[INTERNAL_BASE, u64::MAX]`
/// Any value in the gap (`MAX_COLLECTION_SEQUENCE < id < INTERNAL_BASE`) must be rejected
/// cleanly with `ContextraError::Transaction`.
#[test]
fn test_tx_id_boundary_semantics_and_range_gap() {
    // 1. Valid origin lower bound & sentinel
    let tx_zero = TxId::try_new(0);
    assert!(
        tx_zero.is_ok(),
        "TxId(0) must be a valid origin (TxId::INVALID sentinel)"
    );
    assert_eq!(tx_zero.unwrap(), TxId::INVALID);

    // 2. Valid origin sequence start
    let tx_one = TxId::try_new(1);
    assert!(
        tx_one.is_ok(),
        "TxId(1) must be a valid collection sequence origin"
    );
    assert_eq!(tx_one.unwrap().inner(), 1);

    // 3. Valid origin sequence upper bound
    let tx_max_seq = TxId::try_new(TxId::MAX_COLLECTION_SEQUENCE);
    assert!(
        tx_max_seq.is_ok(),
        "TxId(MAX_COLLECTION_SEQUENCE) must be valid"
    );
    assert_eq!(tx_max_seq.unwrap().inner(), TxId::MAX_COLLECTION_SEQUENCE);

    // 4. Invalid gap boundary lower end
    let err_gap_lower = TxId::try_new(TxId::MAX_COLLECTION_SEQUENCE + 1);
    assert!(
        matches!(err_gap_lower, Err(ContextraError::Transaction(_))),
        "TxId in gap lower bound must return ContextraError::Transaction"
    );

    // 5. Invalid gap wall-clock nanosecond timestamp simulation (~1.7e18)
    let wall_clock_nanos = 1_700_000_000_000_000_000u64;
    let err_gap_wall_clock = TxId::try_new(wall_clock_nanos);
    assert!(
        matches!(err_gap_wall_clock, Err(ContextraError::Transaction(_))),
        "Wall-clock nanos in domain gap must be rejected per AGT-GRAPH-001"
    );

    // 6. Invalid gap boundary upper end
    let err_gap_upper = TxId::try_new(TxId::INTERNAL_BASE - 1);
    assert!(
        matches!(err_gap_upper, Err(ContextraError::Transaction(_))),
        "TxId directly below INTERNAL_BASE must return ContextraError::Transaction"
    );

    // 7. Valid origin internal base lower bound
    let tx_internal_base = TxId::try_new(TxId::INTERNAL_BASE);
    assert!(
        tx_internal_base.is_ok(),
        "TxId(INTERNAL_BASE) must be valid system origin"
    );
    assert_eq!(tx_internal_base.unwrap(), TxId::internal());

    // 8. Valid origin internal u64::MAX
    let tx_u64_max = TxId::try_new(u64::MAX);
    assert!(
        tx_u64_max.is_ok(),
        "TxId(u64::MAX) must be valid system origin"
    );
    assert_eq!(tx_u64_max.unwrap().inner(), u64::MAX);
}

/// Verifies that concurrent `allocate_tx` allocations across N threads produce unique
/// transaction IDs strictly within the collection range `[1, MAX_COLLECTION_SEQUENCE]`
/// without panics or sequence overlap.
#[test]
fn test_parallel_allocate_tx_uniqueness_and_bounds() {
    const NUM_THREADS: usize = 8;
    const ITREATIONS_PER_THREAD: usize = 500;

    struct TestAllocCounter {
        next_tx: AtomicU64,
    }

    impl TestAllocCounter {
        fn allocate_tx(&self) -> Result<TxId, ContextraError> {
            let id = self.next_tx.fetch_add(1, Ordering::SeqCst);
            if id > TxId::MAX_COLLECTION_SEQUENCE {
                return Err(ContextraError::Transaction(
                    "TxId counter exhausted: MAX_COLLECTION_SEQUENCE range exceeded.".into(),
                ));
            }
            TxId::try_new(id)
        }
    }

    let counter = Arc::new(TestAllocCounter {
        next_tx: AtomicU64::new(1),
    });

    let barrier = Arc::new(Barrier::new(NUM_THREADS));
    let mut handles = Vec::with_capacity(NUM_THREADS);

    for t in 0..NUM_THREADS {
        let counter_ref = Arc::clone(&counter);
        let barrier_ref = Arc::clone(&barrier);

        handles.push(thread::spawn(move || {
            barrier_ref.wait();
            let mut local_allocated = Vec::with_capacity(ITREATIONS_PER_THREAD);
            for _ in 0..ITREATIONS_PER_THREAD {
                let tx = counter_ref
                    .allocate_tx()
                    .unwrap_or_else(|e| panic!("Thread {t} failed to allocate TxId: {e}"));
                assert!(
                    tx.inner() >= 1 && tx.inner() <= TxId::MAX_COLLECTION_SEQUENCE,
                    "Allocated TxId {} out of valid collection sequence range",
                    tx.inner()
                );
                assert!(tx.is_valid_origin(), "Allocated TxId must be valid origin");
                local_allocated.push(tx.inner());
            }
            local_allocated
        }));
    }

    let mut all_ids = HashSet::with_capacity(NUM_THREADS * ITREATIONS_PER_THREAD);
    for h in handles {
        let thread_ids = h.join().expect("Thread panicked");
        for id in thread_ids {
            let is_new = all_ids.insert(id);
            assert!(is_new, "Duplicate TxId {id} allocated across threads");
        }
    }

    assert_eq!(
        all_ids.len(),
        NUM_THREADS * ITREATIONS_PER_THREAD,
        "Total allocated unique TxIds must match exact requested count"
    );

    // Verify counter exhaustion behavior near MAX_COLLECTION_SEQUENCE
    let exhausted_counter = TestAllocCounter {
        next_tx: AtomicU64::new(TxId::MAX_COLLECTION_SEQUENCE + 1),
    };
    let res = exhausted_counter.allocate_tx();
    assert!(
        matches!(res, Err(ContextraError::Transaction(_))),
        "Counter beyond MAX_COLLECTION_SEQUENCE must return error"
    );
}

/// Verifies that lifecycle initialization (TxId::try_new(0)) and export score
/// evaluation (TxId::try_new(u64::MAX)) operate without panics.
#[test]
fn test_export_and_lifecycle_tx_id_construction() {
    // Lifecycle initial TxId construction
    let init_tx = TxId::try_new(0).expect("Initial lifecycle TxId(0) must succeed");
    assert_eq!(init_tx, TxId::INVALID);

    // Export max TxId evaluation construction
    let export_tx = TxId::try_new(u64::MAX).expect("Export evaluation TxId(u64::MAX) must succeed");
    assert_eq!(export_tx.inner(), u64::MAX);
    assert!(export_tx.is_valid_origin());
}
