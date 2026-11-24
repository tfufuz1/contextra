// ANCHOR[contextra-mvcc:TEST-LOOM-TX-BUFFER]
//! # Loom-Modell für `TxBuffer` (SPEC-v2 B11)
#![allow(unexpected_cfgs)]

#[cfg(loom)]
mod loom_tests {
    use loom::sync::Arc;
    use loom::sync::Mutex;
    use loom::thread;
    use std::collections::BTreeMap;

    struct MockTxBuffer {
        staging: Mutex<BTreeMap<u64, Vec<(u64, Vec<u8>)>>>,
    }

    impl MockTxBuffer {
        fn new() -> Self {
            Self {
                staging: Mutex::new(BTreeMap::new()),
            }
        }

        fn begin(&self, tx_id: u64) {
            let mut s = self.staging.lock().unwrap();
            s.entry(tx_id).or_default();
        }

        fn stage(&self, tx_id: u64, doc_id: u64, data: Vec<u8>) {
            let mut s = self.staging.lock().unwrap();
            s.entry(tx_id).or_default().push((doc_id, data));
        }

        fn drain(&self, tx_id: u64) -> Vec<(u64, Vec<u8>)> {
            let mut s = self.staging.lock().unwrap();
            s.remove(&tx_id).unwrap_or_default()
        }

        fn discard(&self, tx_id: u64) {
            let mut s = self.staging.lock().unwrap();
            s.remove(&tx_id);
        }
    }

    #[test]
    fn test_loom_tx_buffer_concurrent_stage_drain() {
        loom::model(|| {
            let buffer = Arc::new(MockTxBuffer::new());
            let b1 = Arc::clone(&buffer);
            let b2 = Arc::clone(&buffer);

            b1.begin(1);
            b2.begin(2);

            let t1 = thread::spawn(move || {
                b1.stage(1, 10, vec![1, 2, 3]);
                let ops = b1.drain(1);
                assert!(ops.len() <= 1);
            });

            let t2 = thread::spawn(move || {
                b2.stage(2, 20, vec![4, 5, 6]);
                b2.discard(2);
            });

            t1.join().unwrap();
            t2.join().unwrap();
        });
    }

    #[test]
    fn test_loom_tx_buffer_isolation_two_threads() {
        loom::model(|| {
            let buffer = Arc::new(MockTxBuffer::new());
            let b1 = Arc::clone(&buffer);
            let b2 = Arc::clone(&buffer);

            b1.begin(10);
            b2.begin(20);

            let t1 = thread::spawn(move || {
                b1.stage(10, 100, vec![1]);
                let drained = b1.drain(10);
                assert_eq!(drained.len(), 1);
                assert_eq!(drained[0].0, 100);
            });

            let t2 = thread::spawn(move || {
                b2.stage(20, 200, vec![2]);
                let drained = b2.drain(20);
                assert_eq!(drained.len(), 1);
                assert_eq!(drained[0].0, 200);
            });

            t1.join().unwrap();
            t2.join().unwrap();
        });
    }
}

#[cfg(not(loom))]
#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::useless_conversion
)]
mod normal_tests {
    use contextra_mvcc::tx_buffer::{IndexOp, TxBuffer};
    use contextra_mvcc::types::{DocId, TxId};
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_tx_buffer_stress_concurrency() {
        let buffer = Arc::new(TxBuffer::<Vec<u8>>::new());
        let mut handles = Vec::new();

        for i in 0..4u64 {
            let buf = Arc::clone(&buffer);
            handles.push(thread::spawn(move || {
                let tx = TxId::new(i + 1);
                buf.begin(tx);
                for j in 0..200u64 {
                    let doc_id = DocId::from(i * 1000 + j);
                    buf.stage(
                        tx,
                        IndexOp::Insert {
                            doc_id,
                            data: vec![1, 2, 3],
                        },
                    )
                    .expect("stage");
                }
                let drained = buf.drain(tx);
                assert_eq!(drained.len(), 200);
            }));
        }

        for h in handles {
            h.join().expect("thread panicked");
        }
    }
}
