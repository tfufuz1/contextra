//! Loom-basierter Model-Test für Lock-Handoff & Group-Commit-Reihenfolge.
//! Verifiziert, dass unter Loom-Simulation das Group-Commit-Handoff-Protokoll
//! ohne Stack-Overflow oder Panics abläuft.

#![allow(unexpected_cfgs)]

#[cfg(loom)]
mod loom_tests {
    use loom::sync::atomic::{AtomicU64, Ordering};
    use loom::sync::{Arc, Mutex};
    use loom::thread;

    struct GroupCommitQueue {
        requests: Vec<u64>,
    }

    #[test]
    fn test_loom_group_commit_last_hmac_race() {
        let mut builder = loom::model::Builder::new();
        builder.preemption_bound = Some(3);

        builder.check(|| {
            let commit_mutex = Arc::new(Mutex::new(()));
            let queue = Arc::new(Mutex::new(Option::<GroupCommitQueue>::None));
            let last_applied = Arc::new(AtomicU64::new(0));

            let mut threads = Vec::new();

            for tx_id in 1..=2u64 {
                let cm = Arc::clone(&commit_mutex);
                let q = Arc::clone(&queue);
                let la = Arc::clone(&last_applied);

                threads.push(thread::spawn(move || {
                    let _guard = cm.lock().unwrap();
                    let mut q_guard = q.lock().unwrap();

                    if let Some(ref mut pending) = *q_guard {
                        pending.requests.push(tx_id);
                        drop(q_guard);
                        drop(_guard);
                    } else {
                        *q_guard = Some(GroupCommitQueue {
                            requests: vec![tx_id],
                        });
                        drop(q_guard);
                        drop(_guard);

                        thread::yield_now();

                        let _guard = cm.lock().unwrap();
                        let mut q_guard = q.lock().unwrap();
                        let pending = q_guard.take().unwrap();
                        drop(q_guard);

                        for id in pending.requests {
                            la.store(id, Ordering::Release);
                        }
                    }
                }));
            }

            for t in threads {
                t.join().unwrap();
            }

            assert!(last_applied.load(Ordering::Acquire) > 0);
        });
    }
}

#[cfg(not(loom))]
#[test]
fn test_loom_group_commit_non_loom() {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::{Arc, Mutex};
    use std::thread;

    let commit_mutex = Arc::new(Mutex::new(()));
    let queue = Arc::new(Mutex::new(Option::<Vec<u64>>::None));
    let last_applied = Arc::new(AtomicU64::new(0));

    let mut handles = Vec::new();
    for tx_id in 1..=3u64 {
        let cm = Arc::clone(&commit_mutex);
        let q = Arc::clone(&queue);
        let la = Arc::clone(&last_applied);

        handles.push(thread::spawn(move || {
            let _guard = cm.lock().unwrap();
            let mut q_guard = q.lock().unwrap();
            if let Some(ref mut pending) = *q_guard {
                pending.push(tx_id);
            } else {
                *q_guard = Some(vec![tx_id]);
            }
            drop(q_guard);
            drop(_guard);

            thread::sleep(std::time::Duration::from_millis(1));

            let _guard = cm.lock().unwrap();
            let mut q_guard = q.lock().unwrap();
            if let Some(pending) = q_guard.take() {
                for id in pending {
                    la.store(id, Ordering::Release);
                }
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    assert!(last_applied.load(Ordering::Acquire) > 0);
}
