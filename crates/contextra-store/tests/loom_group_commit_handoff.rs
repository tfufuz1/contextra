//! Loom-basierter isolierter Modelltest für Lock-Handoff & Group-Commit-Reihenfolge.
//!
//! Testet das Synchronisationsmuster zwischen nebenläufigen Writern beim Group-Commit
//! in einer minimalen, abstrahierten Form, damit Loom den Zustandsraum vollständig
//! explorieren kann.
//!
//! Ausführung:
//!   RUSTFLAGS="--cfg loom" cargo test -p contextra-store --release --test loom_group_commit_handoff

#![allow(unexpected_cfgs)]

#[cfg(loom)]
mod loom_tests {
    use loom::sync::atomic::{AtomicBool, Ordering};
    use loom::sync::{Arc, Mutex};
    use loom::thread;

    #[test]
    fn commit_mutex_handoff_no_lost_write() {
        loom::model(|| {
            let state = Arc::new(Mutex::new(Vec::<u64>::new()));
            let s1 = state.clone();
            let s2 = state.clone();

            let t1 = thread::spawn(move || {
                let mut g = s1.lock().unwrap();
                g.push(1);
                drop(g);
                let g = s1.lock().unwrap();
                assert!(g.contains(&1));
            });

            let t2 = thread::spawn(move || {
                let mut g = s2.lock().unwrap();
                g.push(2);
                drop(g);
                let g = s2.lock().unwrap();
                assert!(g.contains(&2));
            });

            t1.join().unwrap();
            t2.join().unwrap();

            let final_state = state.lock().unwrap();
            assert_eq!(final_state.len(), 2);
        });
    }

    #[test]
    fn commit_mutex_handoff_preserves_commit_order() {
        loom::model(|| {
            let wal_log = Arc::new(Mutex::new(Vec::<u64>::new()));
            let commit_mutex = Arc::new(Mutex::new(()));
            let w1_started = Arc::new(AtomicBool::new(false));

            let wal1 = Arc::clone(&wal_log);
            let cm1 = Arc::clone(&commit_mutex);
            let flag1 = Arc::clone(&w1_started);

            let t1 = thread::spawn(move || {
                let _g = cm1.lock().unwrap();
                flag1.store(true, Ordering::Release);
                let mut log = wal1.lock().unwrap();
                log.push(100);
            });

            let wal2 = Arc::clone(&wal_log);
            let cm2 = Arc::clone(&commit_mutex);
            let flag2 = Arc::clone(&w1_started);

            let t2 = thread::spawn(move || {
                while !flag2.load(Ordering::Acquire) {
                    thread::yield_now();
                }
                let _g = cm2.lock().unwrap();
                let mut log = wal2.lock().unwrap();
                assert_eq!(
                    log.len(),
                    1,
                    "Writer 1 must have written to WAL log before Writer 2 acquires lock"
                );
                assert_eq!(log[0], 100);
                log.push(200);
            });

            t1.join().unwrap();
            t2.join().unwrap();

            let final_wal = wal_log.lock().unwrap();
            assert_eq!(*final_wal, vec![100, 200]);
        });
    }
}

#[cfg(not(loom))]
#[test]
fn test_commit_mutex_handoff_non_loom() {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Mutex};
    use std::thread;

    let wal_log = Arc::new(Mutex::new(Vec::<u64>::new()));
    let commit_mutex = Arc::new(Mutex::new(()));
    let w1_started = Arc::new(AtomicBool::new(false));

    let wal1 = Arc::clone(&wal_log);
    let cm1 = Arc::clone(&commit_mutex);
    let flag1 = Arc::clone(&w1_started);

    let t1 = thread::spawn(move || {
        let _g = cm1.lock().unwrap();
        flag1.store(true, Ordering::Release);
        let mut log = wal1.lock().unwrap();
        log.push(100);
    });

    let wal2 = Arc::clone(&wal_log);
    let cm2 = Arc::clone(&commit_mutex);
    let flag2 = Arc::clone(&w1_started);

    let t2 = thread::spawn(move || {
        while !flag2.load(Ordering::Acquire) {
            thread::yield_now();
        }
        let _g = cm2.lock().unwrap();
        let mut log = wal2.lock().unwrap();
        assert_eq!(log.len(), 1);
        assert_eq!(log[0], 100);
        log.push(200);
    });

    t1.join().unwrap();
    t2.join().unwrap();

    let final_wal = wal_log.lock().unwrap();
    assert_eq!(*final_wal, vec![100, 200]);
}
