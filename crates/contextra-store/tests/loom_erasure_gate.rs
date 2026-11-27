//! Loom-Verifikation der Gate-Linearisierung (TST-STORE-GATE-6 und TST-STORE-GATE-7, §10).
//!
//! Ausführen mit:
//!   RUSTFLAGS="--cfg loom" cargo test -p contextra-store --release --test loom_erasure_gate

#[cfg(loom)]
mod loom_tests {
    use loom::sync::atomic::{AtomicBool, AtomicU64, Ordering};
    use loom::sync::{Arc, Mutex, RwLock};
    use loom::thread;

    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    enum Phase {
        Open,
        Restricted,
    }

    struct GateModel {
        active: AtomicBool,
        epoch: AtomicU64,
        snap: RwLock<(u64, Phase)>, // (up_to, phase)
    }

    impl GateModel {
        fn new() -> Self {
            Self {
                active: AtomicBool::new(false),
                epoch: AtomicU64::new(0),
                snap: RwLock::new((0, Phase::Open)),
            }
        }

        fn is_blocked(&self) -> bool {
            if !self.active.load(Ordering::Acquire) {
                return false;
            }
            let guard = self.snap.read().unwrap();
            guard.1 == Phase::Restricted
        }

        fn epoch(&self) -> u64 {
            self.epoch.load(Ordering::Acquire)
        }
    }

    /// TST-STORE-GATE-6 (R1): Commit gegen publish_gate.
    /// Jede vergebene Sequenz `<= gated_seq` gilt als "vor dem Gate akzeptiert",
    /// jede Sequenz `> gated_seq` wird durch H-1 abgewiesen.
    #[test]
    fn test_loom_commit_vs_publish_gate() {
        loom::model(|| {
            let commit_mutex = Arc::new(Mutex::new(()));
            let next_seq = Arc::new(AtomicU64::new(1));
            let gate = Arc::new(GateModel::new());
            let committed_seq = Arc::new(AtomicU64::new(0));
            let gated_seq_out = Arc::new(AtomicU64::new(0));

            let cm1 = Arc::clone(&commit_mutex);
            let ns1 = Arc::clone(&next_seq);
            let g1 = Arc::clone(&gate);
            let cs1 = Arc::clone(&committed_seq);
            let t_commit = thread::spawn(move || {
                let _lock = cm1.lock().unwrap();
                if g1.is_blocked() {
                    // H-1: Abgewiesen vor Sequenzvergabe
                    return;
                }
                let seq = ns1.fetch_add(1, Ordering::SeqCst);
                cs1.store(seq, Ordering::Release);
            });

            let cm2 = Arc::clone(&commit_mutex);
            let ns2 = Arc::clone(&next_seq);
            let g2 = Arc::clone(&gate);
            let gs2 = Arc::clone(&gated_seq_out);
            let t_gate = thread::spawn(move || {
                let _lock = cm2.lock().unwrap();
                let gated_seq = ns2.load(Ordering::SeqCst).saturating_sub(1);
                {
                    let mut w = g2.snap.write().unwrap();
                    *w = (gated_seq, Phase::Restricted);
                }
                g2.epoch.store(1, Ordering::Release);
                g2.active.store(true, Ordering::Release);
                gs2.store(gated_seq, Ordering::Release);
            });

            t_commit.join().unwrap();
            t_gate.join().unwrap();

            let c_seq = committed_seq.load(Ordering::Acquire);
            let g_seq = gated_seq_out.load(Ordering::Acquire);
            if c_seq > 0 {
                assert!(
                    c_seq <= g_seq,
                    "Accepted commit seq {c_seq} must be <= gated_seq {g_seq}"
                );
            }
        });
    }

    /// TST-STORE-GATE-7 (R4): Leser mit Epochen-Prüfung (e1/e2) gegen Veröffentlichung.
    #[test]
    fn test_loom_reader_epoch_vs_publish_gate() {
        loom::model(|| {
            let gate = Arc::new(GateModel::new());
            let data_visible = Arc::new(AtomicBool::new(true));

            let g1 = Arc::clone(&gate);
            let d1 = Arc::clone(&data_visible);
            let t_reader = thread::spawn(move || {
                loop {
                    let e1 = g1.epoch();
                    let blocked = g1.is_blocked();
                    let val = if blocked {
                        false
                    } else {
                        d1.load(Ordering::Acquire)
                    };
                    let e2 = g1.epoch();
                    if e1 == e2 {
                        // Konsistenter Read: Wenn e1 == e2 == 1, muss blocked == true sein.
                        if e1 >= 1 {
                            assert!(!val, "Post-gate consistent read must not observe data");
                        }
                        break;
                    }
                }
            });

            let g2 = Arc::clone(&gate);
            let t_pub = thread::spawn(move || {
                {
                    let mut w = g2.snap.write().unwrap();
                    *w = (1, Phase::Restricted);
                }
                g2.epoch.store(1, Ordering::Release);
                g2.active.store(true, Ordering::Release);
            });

            t_reader.join().unwrap();
            t_pub.join().unwrap();
        });
    }
}

#[cfg(not(loom))]
#[test]
fn test_loom_erasure_gate_non_loom_placeholder() {
    // Loom-Tests werden mit `RUSTFLAGS="--cfg loom"` ausgeführt.
}
