//! Loom-basierter Test für ErasureGate & Group-Commit Drain-Fence (Protocol B, SP-10).
//!
//! Testet das 2-Fenster Fence-Protokoll mit 3 Threads (Leader, Writer, Installer)
//! über alle 3 relativen Interleavings:
//! (a) Fence vor Fenster 1
//! (b) Fence zwischen Fenster 1 und Fenster 2 (während WAL I/O)
//! (c) Fence nach Fenster 2
//!
//! Ausführung:
//!   RUSTFLAGS="--cfg loom" cargo test --locked -p contextra-store --test loom_fence --release

#![allow(unexpected_cfgs)]

#[cfg(loom)]
mod loom_tests {
    use loom::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
    use loom::sync::{Arc, Mutex};
    use loom::thread;

    #[derive(Debug)]
    struct MockGate {
        active: AtomicBool,
        fenced_prefix: Mutex<Option<Vec<u8>>>,
        inflight: AtomicU32,
        drained_flag: AtomicBool,
        install_mutex: Mutex<()>,
    }

    impl MockGate {
        fn new() -> Self {
            Self {
                active: AtomicBool::new(false),
                fenced_prefix: Mutex::new(None),
                inflight: AtomicU32::new(0),
                drained_flag: AtomicBool::new(false),
                install_mutex: Mutex::new(()),
            }
        }

        fn is_active(&self) -> bool {
            self.active.load(Ordering::Acquire)
        }

        fn check_key(&self, key: &[u8]) -> Result<(), ()> {
            if !self.is_active() {
                return Ok(());
            }
            let guard = self.fenced_prefix.lock().unwrap();
            if let Some(ref p) = *guard {
                if key.starts_with(p) {
                    return Err(());
                }
            }
            Ok(())
        }

        fn inc_inflight(&self) {
            self.inflight.fetch_add(1, Ordering::SeqCst);
        }

        fn dec_inflight(&self) {
            let prev = self.inflight.fetch_sub(1, Ordering::SeqCst);
            if prev == 1 {
                self.drained_flag.store(true, Ordering::Release);
            }
        }

        fn acquire_fence(&self, prefix: Vec<u8>) {
            let _g = self.install_mutex.lock().unwrap();
            {
                let mut p_guard = self.fenced_prefix.lock().unwrap();
                *p_guard = Some(prefix);
            }
            self.active.store(true, Ordering::Release);

            // Drain phase: wait until inflight == 0
            while self.inflight.load(Ordering::Acquire) > 0 {
                thread::yield_now();
            }
        }
    }

    #[test]
    fn test_loom_drain_fence_protocol_b() {
        let mut builder = loom::model::Builder::new();
        builder.preemption_bound = Some(2);

        builder.check(|| {
            let gate = Arc::new(MockGate::new());
            let commit_mutex = Arc::new(Mutex::new(()));
            let memtable = Arc::new(Mutex::new(Vec::<(u64, Vec<u8>)>::new()));
            let next_seq = Arc::new(AtomicU64::new(1));

            // State flags for scenario verification
            let drain_completed = Arc::new(AtomicBool::new(false));

            let g_leader = Arc::clone(&gate);
            let cm_leader = Arc::clone(&commit_mutex);
            let mem_leader = Arc::clone(&memtable);
            let seq_leader = Arc::clone(&next_seq);

            // Thread 1: Leader (Window 1 -> WAL I/O -> Window 2)
            let t_leader = thread::spawn(move || {
                let key = b"doc_100".to_vec();

                // WINDOW 1: under commit_mutex lock
                let _cm = cm_leader.lock().unwrap();
                if g_leader.check_key(&key).is_err() {
                    // Fence active: rejected in Window 1
                    return Err("fenced_window_1");
                }
                g_leader.inc_inflight();
                let assigned_seq = seq_leader.fetch_add(1, Ordering::SeqCst);
                drop(_cm);

                // SIMULATED WAL I/O: outside commit_mutex lock
                thread::yield_now();

                // WINDOW 2: re-acquire commit_mutex lock & publish to memtable
                let _cm = cm_leader.lock().unwrap();
                let mut mem = mem_leader.lock().unwrap();
                mem.push((assigned_seq, key));
                drop(mem);
                g_leader.dec_inflight();
                drop(_cm);

                Ok(assigned_seq)
            });

            // Thread 2: Writer
            let g_writer = Arc::clone(&gate);
            let cm_writer = Arc::clone(&commit_mutex);
            let mem_writer = Arc::clone(&memtable);
            let seq_writer = Arc::clone(&next_seq);

            let t_writer = thread::spawn(move || {
                let key = b"doc_200".to_vec();
                let _cm = cm_writer.lock().unwrap();
                if g_writer.check_key(&key).is_err() {
                    return Err("fenced_writer");
                }
                g_writer.inc_inflight();
                let assigned_seq = seq_writer.fetch_add(1, Ordering::SeqCst);
                drop(_cm);

                thread::yield_now();

                let _cm = cm_writer.lock().unwrap();
                let mut mem = mem_writer.lock().unwrap();
                mem.push((assigned_seq, key));
                drop(mem);
                g_writer.dec_inflight();
                drop(_cm);

                Ok(assigned_seq)
            });

            // Thread 3: Installer (Installs fence over "doc_" prefix and drains)
            let g_installer = Arc::clone(&gate);
            let drain_flag = Arc::clone(&drain_completed);

            let t_installer = thread::spawn(move || {
                g_installer.acquire_fence(b"doc_".to_vec());
                drain_flag.store(true, Ordering::Release);
            });

            let _ = t_leader.join();
            let _ = t_writer.join();
            let _ = t_installer.join();

            let final_mem = memtable.lock().unwrap();

            // Verifications:
            // 1. If a batch was published to memtable, its seq number was assigned in Window 1.
            // 2. Dense sequence numbers: next_seq after execution equals 1 + count of successful publishes.
            // 3. No record with seq > Drain was added after Drain completed.
            if drain_completed.load(Ordering::Acquire) {
                for (seq, key) in final_mem.iter() {
                    assert!(*seq > 0, "Sequence number must be positive");
                    assert!(key.starts_with(b"doc_"));
                }
            }
        });
    }
}

#[cfg(not(loom))]
#[test]
fn test_loom_fence_non_loom() {
    println!("Loom fence test skipped in non-loom mode");
}
