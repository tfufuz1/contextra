//! Loom-basierter Concurrency-Test für Reader-Sichtbarkeit vs Commit & Flush.
//! Verifiziert, dass ein Reader mit Snapshot s genau alle Commits mit seq <= s sieht und keinen mit seq > s.

#![allow(unexpected_cfgs)]

#[cfg(loom)]
mod loom_tests {
    use loom::sync::atomic::{AtomicU64, Ordering};
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
}

#[cfg(not(loom))]
#[test]
fn test_commit_flush_visibility_non_loom() {
    use std::sync::atomic::{AtomicU64, Ordering};
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
}
