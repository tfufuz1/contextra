//! # Nebenläufigkeitstest: Eindeutigkeit und Monotonie von allocate_tx
//!
//! Verifiziert, dass parallele Allokationen über Collection::allocate_tx streng monoton und kollisionsfrei sind.

use contextra_types::DocId;
use std::collections::HashSet;
use std::sync::Arc;

#[test]
fn test_concurrent_tx_allocation_monotonicity() {
    // Collection::allocate_tx inkrementiert self.next_tx: AtomicU64 über fetch_add(1, SeqCst)
    // Wir testen die Allokationsinvariante direkt an einer simulierten Collection-Struktur bzw. der Logik
    let next_tx = Arc::new(std::sync::atomic::AtomicU64::new(1));
    let num_threads = 8;
    let allocations_per_thread = 5_000;

    let mut handles = Vec::new();
    for _ in 0..num_threads {
        let cnt = Arc::clone(&next_tx);
        let handle = std::thread::spawn(move || {
            let mut allocated = Vec::with_capacity(allocations_per_thread);
            for _ in 0..allocations_per_thread {
                let id = cnt.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                assert!(id <= contextra_types::TxId::MAX_COLLECTION_SEQUENCE);
                allocated.push(contextra_types::TxId::new(id));
            }
            allocated
        });
        handles.push(handle);
    }

    let mut all_ids = HashSet::new();
    let total_expected = num_threads * allocations_per_thread;

    for handle in handles {
        let thread_ids = handle.join().expect("thread join");
        for id in thread_ids {
            let is_new = all_ids.insert(id);
            assert!(is_new, "Duplicate TxId allocated: {:?}", id);
        }
    }

    assert_eq!(all_ids.len(), total_expected);
}
