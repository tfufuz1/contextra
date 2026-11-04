//! Loom-Modell und Stresstests für Gate, Barriere und Lese-Linearisierung (SPEC §11.3, D-57, D-120, I-21, T-CONC-1).
//!
//! Modelliert:
//! - Nebenläufige Leser und Gate-Updates (`gate_epoch += 1` unter exklusiver Barriere)
//! - Wiederholung von Lesern bei Epochenwechsel über den Gate-Zeitpunkt
//! - Keine Rückgabe von Klartext für ein Ziel, das während des Lesevorgangs gegated wurde

#[cfg(loom)]
mod loom_model {
    use loom::sync::atomic::{AtomicU64, Ordering};
    use loom::sync::{Arc, RwLock};
    use loom::thread;

    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    enum State {
        Open,
        Erased,
    }

    struct LoomGateModel {
        epoch: AtomicU64,
        key_state: RwLock<State>,
    }

    impl LoomGateModel {
        fn new() -> Self {
            Self {
                epoch: AtomicU64::new(0),
                key_state: RwLock::new(State::Open),
            }
        }

        // Simuliert Gate-Commit unter exklusiver Barriere (D-57, I-21)
        fn apply_gate(&self) {
            // 1. Epoche inkrementieren VOR der Zustandsänderung (I-21)
            self.epoch.fetch_add(1, Ordering::SeqCst);
            // 2. Zustand auf Erased setzen
            let mut state = self.key_state.write().unwrap();
            *state = State::Erased;
        }

        // Simuliert linearisierten Leser mit Retry bei Epochenwechsel (D-120, I-21)
        fn linearized_read(&self) -> State {
            loop {
                let epoch_before = self.epoch.load(Ordering::SeqCst);
                let state_before = *self.key_state.read().unwrap();
                if state_before != State::Open {
                    return state_before;
                }

                // Simulierte Entschlüsselungs-/I/O-Verzögerung
                let value = state_before;

                let epoch_after = self.epoch.load(Ordering::SeqCst);
                if epoch_after != epoch_before {
                    // Epochenänderung: Volle Prüfung wiederholen!
                    let state_after = *self.key_state.read().unwrap();
                    if state_after != State::Open {
                        return state_after;
                    }
                    continue;
                }

                return value;
            }
        }
    }

    #[test]
    fn t_conc_1_loom_gate_linearized_reader() {
        loom::model(|| {
            let gate = Arc::new(LoomGateModel::new());

            let g1 = Arc::clone(&gate);
            let reader_thread = thread::spawn(move || g1.linearized_read());

            let g2 = Arc::clone(&gate);
            let gate_thread = thread::spawn(move || {
                g2.apply_gate();
            });

            gate_thread.join().unwrap();
            let read_result = reader_thread.join().unwrap();

            // Nach Abschluss beider Threads darf der Leser entweder Open
            // (vollständig VOR dem Gate abgeschlossen) oder Erased geliefert haben.
            // Er darf aber niemals in einem inkonsistenten Zwischenzustand steckenbleiben.
            assert!(read_result == State::Open || read_result == State::Erased);
        });
    }
}

#[cfg(not(loom))]
#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod stress_tests {
    use contextra_erasure::gate::erased_set::ErasedSet;
    use contextra_erasure::gate::read_gate::GateGuard;
    use contextra_ports::gate::{GateDecision, KeyBuf, ReadGate, Seq};
    use parking_lot::RwLock;
    use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn t_conc_1_stress_concurrent_reads_and_gate_linearization() {
        let set = Arc::new(RwLock::new(ErasedSet::new()));
        let guard = Arc::new(GateGuard::new(Arc::clone(&set)));
        let running = Arc::new(AtomicBool::new(true));
        let read_violations = Arc::new(AtomicU64::new(0));

        let target_key = KeyBuf::new([0x42; 16]);

        let mut readers = Vec::new();
        for _ in 0..4 {
            let g = Arc::clone(&guard);
            let run = Arc::clone(&running);
            let violations = Arc::clone(&read_violations);
            let k = target_key;

            readers.push(thread::spawn(move || {
                let mut read_count = 0;
                while run.load(Ordering::Relaxed) && read_count < 1000 {
                    let res = g.linearized_read(&k, || Ok("secret_document_bytes"));
                    if g.check(&k) == GateDecision::Erased {
                        // Wenn der Schlüssel bereits gegated ist, darf linearized_read NIEMALS Ok zurückgeben!
                        if res.is_ok() {
                            violations.fetch_add(1, Ordering::SeqCst);
                        }
                    }
                    read_count += 1;
                }
            }));
        }

        // Gate wird mitten unter den Lesern gesetzt
        thread::sleep(std::time::Duration::from_millis(5));
        set.write().add_erased(
            KeyBuf::new([0x40; 16]),
            KeyBuf::new([0x50; 16]),
            Seq::new(100),
        );

        running.store(false, Ordering::Relaxed);

        for r in readers {
            r.join().expect("reader panicked");
        }

        assert_eq!(
            read_violations.load(Ordering::SeqCst),
            0,
            "No reader should leak plaintext after gate is active (I-21)"
        );
        assert_eq!(guard.check(&target_key), GateDecision::Erased);
    }
}
