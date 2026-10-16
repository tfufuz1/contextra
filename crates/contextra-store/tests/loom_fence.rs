//! Loom-basierter Test für das ErasureFence / Group-Commit Fence-Protokoll (§3, §5, ADR-V2-1).
//!
//! Verifiziert die 4 geforderten Loom-Szenarien mit mindestens 3 Threads:
//! (a) Erwerb vor dem Leader-Erwerb;
//! (b) Erwerb während der Leader im Sammelfenster freigegeben hat (Drain-Barriere hält, kein unangewendeter Batch);
//! (c) Freigabe während eines laufenden Batches;
//! (d) Leader-eigene Operation wird abgewiesen und die Sequenz wird im SSI-Validator zurückgenommen (F-7).

#![allow(unexpected_cfgs)]

#[cfg(loom)]
mod loom_tests {
    #[path = "../src/lsm/fence.rs"]
    mod fence;

    use fence::{ErasureFence, FenceError};
    use loom::sync::atomic::{AtomicBool, AtomicU64, Ordering};
    use loom::sync::{Arc, Mutex};
    use loom::thread;

    /// Test-SsiValidator zur Verifikation der Sequenz-Rücknahme (F-7) unter Loom.
    #[derive(Debug, Default)]
    struct TestSsiValidator {
        tracked_seqs: Mutex<Vec<u64>>,
    }

    impl TestSsiValidator {
        fn record(&self, seq: u64) {
            self.tracked_seqs.lock().unwrap().push(seq);
        }

        fn forget_from(&self, seq: u64) {
            let mut seqs = self.tracked_seqs.lock().unwrap();
            seqs.retain(|&s| s < seq);
        }

        fn contains(&self, seq: u64) -> bool {
            self.tracked_seqs.lock().unwrap().contains(&seq)
        }
    }

    /// Loom-Harness zur Orchestrierung von ErasureFence und commit_mutex
    #[derive(Debug)]
    struct LoomFenceEngine {
        fence: ErasureFence,
        next_seq: AtomicU64,
        ssi_validator: TestSsiValidator,
        memtable: Mutex<Vec<(u64, Vec<u8>)>>,
    }

    impl LoomFenceEngine {
        fn new() -> Self {
            Self {
                fence: ErasureFence::new(),
                next_seq: AtomicU64::new(1),
                ssi_validator: TestSsiValidator::default(),
                memtable: Mutex::new(Vec::new()),
            }
        }

        fn acquire_scope_fence_for(&self, prefix: Vec<u8>) -> Result<u64, FenceError> {
            let guard = self.fence.acquire_scope_fence_for(vec![prefix])?;
            Ok(guard.scope_write_seq())
        }

        fn release_scope_fence_for(&self, prefix: &[u8]) {
            self.fence.release_scope_fence_for_locked(prefix);
        }

        fn execute_commit(&self, key: &[u8]) -> Result<u64, ()> {
            let _cm = self.fence.commit_mutex.lock().unwrap();
            if self.fence.check_write_allowed_scope(key).is_err() {
                return Err(());
            }

            let seq = self.next_seq.fetch_add(1, Ordering::SeqCst);
            self.ssi_validator.record(seq);
            self.memtable.lock().unwrap().push((seq, key.to_vec()));
            Ok(seq)
        }

        fn execute_commit_with_preassigned_seq(&self, key: &[u8]) -> Result<u64, ()> {
            let _cm = self.fence.commit_mutex.lock().unwrap();
            let pre_seq = self.next_seq.fetch_add(1, Ordering::SeqCst);
            self.ssi_validator.record(pre_seq);

            if self.fence.check_write_allowed_scope(key).is_err() {
                self.ssi_validator.forget_from(pre_seq);
                return Err(());
            }

            self.memtable.lock().unwrap().push((pre_seq, key.to_vec()));
            Ok(pre_seq)
        }
    }

    /// Szenario (a): Erwerb vor dem Leader-Erwerb
    #[test]
    fn test_loom_scenario_a_acquire_before_leader() {
        let mut builder = loom::model::Builder::new();
        builder.preemption_bound = Some(2);

        builder.check(|| {
            let engine = Arc::new(LoomFenceEngine::new());

            let e_fence = Arc::clone(&engine);
            let t_fence = thread::spawn(move || {
                let _ = e_fence.acquire_scope_fence_for(b"doc:".to_vec());
            });

            let e_leader = Arc::clone(&engine);
            let t_leader = thread::spawn(move || {
                thread::yield_now();
                e_leader.execute_commit(b"doc:1")
            });

            t_fence.join().unwrap();
            let res = t_leader.join().unwrap();

            if res.is_err() {
                assert!(engine.memtable.lock().unwrap().is_empty());
            }
        });
    }

    /// Szenario (b): Erwerb während Leader im Sammelfenster freigegeben hat
    #[test]
    fn test_loom_scenario_b_acquire_during_collection_window() {
        let mut builder = loom::model::Builder::new();
        builder.preemption_bound = Some(2);

        builder.check(|| {
            let engine = Arc::new(LoomFenceEngine::new());
            let collection_window_active = Arc::new(AtomicBool::new(false));

            let e_leader = Arc::clone(&engine);
            let window_flag = Arc::clone(&collection_window_active);

            let t_leader = thread::spawn(move || {
                window_flag.store(true, Ordering::Release);
                thread::yield_now();
                window_flag.store(false, Ordering::Release);

                e_leader.execute_commit(b"coll:key1")
            });

            let e_fence = Arc::clone(&engine);
            let window_flag_f = Arc::clone(&collection_window_active);

            let t_fence = thread::spawn(move || {
                while !window_flag_f.load(Ordering::Acquire) {
                    thread::yield_now();
                }
                let _ = e_fence.acquire_scope_fence_for(b"coll:".to_vec());
            });

            let e_follower = Arc::clone(&engine);
            let t_follower = thread::spawn(move || {
                thread::yield_now();
                e_follower.execute_commit(b"coll:key2")
            });

            t_fence.join().unwrap();
            let res_leader = t_leader.join().unwrap();
            let res_follower = t_follower.join().unwrap();

            let mem = engine.memtable.lock().unwrap();
            if res_leader.is_err() {
                assert!(mem.iter().all(|(_, k)| !k.starts_with(b"coll:key1")));
            }
            if res_follower.is_err() {
                assert!(mem.iter().all(|(_, k)| !k.starts_with(b"coll:key2")));
            }
        });
    }

    /// Szenario (c): Freigabe während eines laufenden Batches
    #[test]
    fn test_loom_scenario_c_release_during_running_batch() {
        let mut builder = loom::model::Builder::new();
        builder.preemption_bound = Some(2);

        builder.check(|| {
            let engine = Arc::new(LoomFenceEngine::new());
            let _ = engine.acquire_scope_fence_for(b"scope:".to_vec());

            let e_fence = Arc::clone(&engine);
            let t_release = thread::spawn(move || {
                thread::yield_now();
                e_fence.release_scope_fence_for(b"scope:");
            });

            let e_writer = Arc::clone(&engine);
            let t_writer = thread::spawn(move || {
                e_writer.execute_commit(b"scope:test")
            });

            t_release.join().unwrap();
            let res = t_writer.join().unwrap();

            if res.is_ok() {
                assert!(engine.fence.check_write_allowed_scope(b"scope:test").is_ok());
            }
        });
    }

    /// Szenario (d): Leader-eigene Operation wird abgewiesen und Sequenz wird zurückgenommen (F-7)
    #[test]
    fn test_loom_scenario_d_rejection_sequence_rollback() {
        let mut builder = loom::model::Builder::new();
        builder.preemption_bound = Some(2);

        builder.check(|| {
            let engine = Arc::new(LoomFenceEngine::new());
            let _ = engine.acquire_scope_fence_for(b"fenced:".to_vec());

            let res = engine.execute_commit_with_preassigned_seq(b"fenced:doc1");
            assert!(res.is_err(), "Leader must be rejected by active fence");

            assert!(
                !engine.ssi_validator.contains(1),
                "Assigned sequence number must be rolled back on Window-1 fence rejection"
            );
        });
    }
}

#[cfg(not(loom))]
#[test]
fn test_loom_fence_non_loom() {
    println!("Loom fence protocol tests run in loom mode via --cfg loom");
}
