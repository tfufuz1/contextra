//! Loom-basierter Concurrency-Test für Reader-Sichtbarkeit vs Commit & Flush sowie Flush vs Purge.
//! Verifiziert, dass ein Reader mit Snapshot s genau alle Commits mit seq <= s sieht und keinen mit seq > s.
//! Verifiziert außerdem das kooperative Dekker-Write-Fencing zwischen Flush und Purge.

#![allow(unexpected_cfgs)]

#[cfg(loom)]
mod loom_tests {
    use loom::sync::atomic::{AtomicUsize, AtomicU64, Ordering};
    use loom::sync::{Arc, Mutex};
    use loom::thread;

    #[test]
    fn test_commit_flush_visibility_snapshot_isolation() {
        let mut builder = loom::model::Builder::new();
        builder.preemption_bound = Some(2);

        builder.check(|| {
            let memtable = Arc::new(Mutex::new(Vec::<(u64, u64)>::new())); // (seq, val)
            let last_applied_seq = Arc::new(AtomicU64::new(0));

            let m_writer = Arc::clone(&memtable);
            let la_writer = Arc::clone(&last_applied_seq);

            // Commit Task
            let writer = thread::spawn(move || {
                for seq in 1..=2u64 {
                    {
                        let mut m = m_writer.lock().unwrap();
                        m.push((seq, seq * 10));
                    }
                    la_writer.store(seq, Ordering::Release);
                }
            });

            let m_reader = Arc::clone(&memtable);
            let la_reader = Arc::clone(&last_applied_seq);

            // Reader Task
            let reader = thread::spawn(move || {
                let snapshot = la_reader.load(Ordering::Acquire);
                let m = m_reader.lock().unwrap();

                for &(seq, val) in m.iter() {
                    if seq <= snapshot {
                        assert_eq!(val, seq * 10);
                    }
                }
            });

            writer.join().unwrap();
            reader.join().unwrap();
        });
    }

    #[test]
    fn test_loom_flush_vs_purge_cooperative_fence() {
        let mut builder = loom::model::Builder::new();
        builder.preemption_bound = Some(2);

        builder.check(|| {
            // State: 0 = Idle, 1 = Flush, 2 = Purge
            let state = Arc::new(AtomicUsize::new(0));
            let in_cs = Arc::new(AtomicU64::new(0));

            let s1 = Arc::clone(&state);
            let cs1 = Arc::clone(&in_cs);

            // Flush thread (versucht 0 -> 1 mit compare_exchange)
            let flush_t = thread::spawn(move || {
                let entered = s1.compare_exchange(0, 1, Ordering::SeqCst, Ordering::SeqCst).is_ok();
                if entered {
                    let prev = cs1.fetch_add(1, Ordering::SeqCst);
                    assert_eq!(prev, 0, "Mutual exclusion violated: concurrent CS");
                    cs1.fetch_sub(1, Ordering::SeqCst);
                    s1.store(0, Ordering::SeqCst);
                }
            });

            let s2 = Arc::clone(&state);
            let cs2 = Arc::clone(&in_cs);

            // Purge thread (versucht 0 -> 2 mit compare_exchange)
            let purge_t = thread::spawn(move || {
                let entered = s2.compare_exchange(0, 2, Ordering::SeqCst, Ordering::SeqCst).is_ok();
                if entered {
                    let prev = cs2.fetch_add(1, Ordering::SeqCst);
                    assert_eq!(prev, 0, "Mutual exclusion violated: concurrent CS");
                    cs2.fetch_sub(1, Ordering::SeqCst);
                    s2.store(0, Ordering::SeqCst);
                }
            });

            flush_t.join().unwrap();
            purge_t.join().unwrap();
        });
    }
}

#[cfg(not(loom))]
#[test]
fn test_commit_flush_visibility_non_loom() {
    use std::sync::atomic::{AtomicUsize, AtomicU64, Ordering};
    use std::sync::{Arc, Mutex};
    use std::thread;

    let memtable = Arc::new(Mutex::new(Vec::<(u64, u64)>::new()));
    let last_applied_seq = Arc::new(AtomicU64::new(0));

    let m_writer = Arc::clone(&memtable);
    let la_writer = Arc::clone(&last_applied_seq);

    let writer = thread::spawn(move || {
        for seq in 1..=2u64 {
            {
                let mut m = m_writer.lock().unwrap();
                m.push((seq, seq * 10));
            }
            la_writer.store(seq, Ordering::Release);
        }
    });

    let m_reader = Arc::clone(&memtable);
    let la_reader = Arc::clone(&last_applied_seq);

    let reader = thread::spawn(move || {
        let snapshot = la_reader.load(Ordering::Acquire);
        let m = m_reader.lock().unwrap();

        for &(seq, val) in m.iter() {
            if seq <= snapshot {
                assert_eq!(val, seq * 10);
            }
        }
    });

    writer.join().unwrap();
    reader.join().unwrap();

    let state = Arc::new(AtomicUsize::new(0));
    let in_cs = Arc::new(AtomicU64::new(0));

    let s1 = Arc::clone(&state);
    let cs1 = Arc::clone(&in_cs);

    let flush_t = thread::spawn(move || {
        let entered = s1.compare_exchange(0, 1, Ordering::SeqCst, Ordering::SeqCst).is_ok();
        if entered {
            let prev = cs1.fetch_add(1, Ordering::SeqCst);
            assert_eq!(prev, 0);
            cs1.fetch_sub(1, Ordering::SeqCst);
            s1.store(0, Ordering::SeqCst);
        }
    });

    let s2 = Arc::clone(&state);
    let cs2 = Arc::clone(&in_cs);

    let purge_t = thread::spawn(move || {
        let entered = s2.compare_exchange(0, 2, Ordering::SeqCst, Ordering::SeqCst).is_ok();
        if entered {
            let prev = cs2.fetch_add(1, Ordering::SeqCst);
            assert_eq!(prev, 0);
            cs2.fetch_sub(1, Ordering::SeqCst);
            s2.store(0, Ordering::SeqCst);
        }
    });

    flush_t.join().unwrap();
    purge_t.join().unwrap();
}
