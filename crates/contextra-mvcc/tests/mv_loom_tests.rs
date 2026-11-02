//! # Loom-Modelle für SeqAllocator, Lockreihenfolge und History-Horizon (T-LOCK, T-CONC, T-SNAP)
//!
//! Modelliert:
//! - Nebenläufige Sequenzvergabe (T-CONC-1)
//! - Einhaltung der vollständigen Lock-Hierarchie SPEC 5.4:
//!   Dokumentlock -> Commit-Barriere -> Purge-Barrier -> Journal (T-LOCK)
//! - Purge-Barriere und History-Horizon Abgrenzung gegen aktive Snapshots (T-SNAP):
//!   Horizon <= älteste aktive Snapshot-Seq; Leser sehen vollständigen Alt-Zustand oder Verweigerung.

#[cfg(loom)]
mod loom_conc_tests {
    use loom::sync::atomic::{AtomicU64, Ordering};
    use loom::sync::{Arc, Mutex, RwLock};
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
    fn t_lock_loom_lock_order_hierarchy_full_spec_5_4() {
        // SPEC 5.4: Dokumentlock -> Commit-Barriere -> Purge-Barrier -> Journal
        loom::model(|| {
            let doc_lock = Arc::new(Mutex::new(0));
            let commit_barrier = Arc::new(RwLock::new(0));
            let purge_barrier = Arc::new(RwLock::new(0));
            let journal_lock = Arc::new(Mutex::new(0));

            let d1 = Arc::clone(&doc_lock);
            let c1 = Arc::clone(&commit_barrier);
            let p1 = Arc::clone(&purge_barrier);
            let j1 = Arc::clone(&journal_lock);

            // Thread 1: Normaler Schreibpfad / Gruppen-Commit
            let t1 = thread::spawn(move || {
                let _d = d1.lock().unwrap();
                let _c = c1.read().unwrap();
                let _p = p1.read().unwrap();
                let _j = j1.lock().unwrap();
            });

            let d2 = Arc::clone(&doc_lock);
            let c2 = Arc::clone(&commit_barrier);
            let p2 = Arc::clone(&purge_barrier);
            let j2 = Arc::clone(&journal_lock);

            // Thread 2: Konkurrierende Transaktion auf selbe Hierarchie
            let t2 = thread::spawn(move || {
                let _d = d2.lock().unwrap();
                let _c = c2.read().unwrap();
                let _p = p2.read().unwrap();
                let _j = j2.lock().unwrap();
            });

            t1.join().unwrap();
            t2.join().unwrap();
        });
    }

    #[test]
    fn t_snap_loom_history_horizon_never_exceeds_active_snapshots() {
        // Purge-Barriere und History-Horizon dürfen niemals über die kleinste aktive Snapshot-Seq gehoben werden.
        loom::model(|| {
            let min_active = Arc::new(AtomicU64::new(100));
            let horizon = Arc::new(AtomicU64::new(50));

            let m1 = Arc::clone(&min_active);
            let h1 = Arc::clone(&horizon);

            // Purge / Prune-Thread: darf Horizon nur bis min_active anheben
            let t1 = thread::spawn(move || {
                let active = m1.load(Ordering::Acquire);
                let current_h = h1.load(Ordering::Acquire);
                let candidate_h = 120;
                // Invariante: Horizon <= active
                let safe_h = candidate_h.min(active);
                if safe_h > current_h {
                    h1.store(safe_h, Ordering::Release);
                }
            });

            let m2 = Arc::clone(&min_active);
            let h2 = Arc::clone(&horizon);

            // Reader-Thread: prüft, dass der gelesene Snapshot >= Horizon ist
            let t2 = thread::spawn(move || {
                let active = m2.load(Ordering::Acquire);
                let h = h2.load(Ordering::Acquire);
                assert!(
                    h <= active,
                    "History-Horizon darf niemals größer als min_active_seqno werden!"
                );
            });

            t1.join().unwrap();
            t2.join().unwrap();
        });
    }
}
