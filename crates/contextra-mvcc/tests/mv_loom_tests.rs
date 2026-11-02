//! # Loom-Modelle für SeqAllocator und Lockreihenfolge (T-CONC-1)
//!
//! Modelliert die nebenläufige Sequenzvergabe und die Einhaltung der Lock-Hierarchie:
//! Dokumentlock -> Commit-Barriere -> Purge-Barrier -> Journal.

#[cfg(loom)]
mod loom_conc_tests {
    use loom::sync::atomic::{AtomicU64, Ordering};
    use loom::sync::{Arc, Mutex};
    use loom::thread;

    struct LoomSeqAllocator {
        current_seq: AtomicU64,
        ledger_floor: AtomicU64,
    }

    impl LoomSeqAllocator {
        fn new(current: u64, floor: u64) -> Self {
            Self {
                current_seq: AtomicU64::new(current),
                ledger_floor: AtomicU64::new(floor),
            }
        }

        fn next_seq(&self) -> Option<u64> {
            let mut curr = self.current_seq.load(Ordering::Acquire);
            loop {
                let floor = self.ledger_floor.load(Ordering::Acquire);
                let base = curr.max(floor);
                let next = base.checked_add(1)?;
                match self.current_seq.compare_exchange_weak(
                    curr,
                    next,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                ) {
                    Ok(_) => return Some(next),
                    Err(actual) => curr = actual,
                }
            }
        }
    }

    #[test]
    fn t_conc_1_loom_seq_allocator_concurrent_monotonic() {
        loom::model(|| {
            let alloc = Arc::new(LoomSeqAllocator::new(10, 10));

            let a1 = Arc::clone(&alloc);
            let a2 = Arc::clone(&alloc);

            let t1 = thread::spawn(move || {
                let s1 = a1.next_seq().unwrap();
                assert!(s1 >= 11);
            });

            let t2 = thread::spawn(move || {
                let s2 = a2.next_seq().unwrap();
                assert!(s2 >= 11);
            });

            t1.join().unwrap();
            t2.join().unwrap();

            assert_eq!(alloc.current_seq.load(Ordering::Acquire), 12);
        });
    }

    #[test]
    fn t_conc_1_loom_lock_order_hierarchy() {
        loom::model(|| {
            let doc_lock = Arc::new(Mutex::new(0));
            let commit_barrier = Arc::new(Mutex::new(0));

            let d1 = Arc::clone(&doc_lock);
            let c1 = Arc::clone(&commit_barrier);

            let t1 = thread::spawn(move || {
                let _l1 = d1.lock().unwrap();
                let _l2 = c1.lock().unwrap();
            });

            let d2 = Arc::clone(&doc_lock);
            let c2 = Arc::clone(&commit_barrier);

            let t2 = thread::spawn(move || {
                let _l1 = d2.lock().unwrap();
                let _l2 = c2.lock().unwrap();
            });

            t1.join().unwrap();
            t2.join().unwrap();
        });
    }
}
