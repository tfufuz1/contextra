// ANCHOR[contextra-mvcc:TEST-LOOM-SEQ-LOG]
//! # Loom-Modell für `SequenceLog` (SPEC-v2 B11)
#![allow(unexpected_cfgs)]

#[cfg(loom)]
mod loom_tests {
    use loom::sync::Arc;
    use loom::sync::RwLock;
    use loom::thread;
    use std::collections::BTreeMap;
    use std::sync::atomic::{AtomicU64, Ordering};

    struct MockSequenceLog {
        counter: AtomicU64,
        entries: RwLock<BTreeMap<u64, (u64, Option<u64>)>>,
    }

    impl MockSequenceLog {
        fn new() -> Self {
            Self {
                counter: AtomicU64::new(1),
                entries: RwLock::new(BTreeMap::new()),
            }
        }

        fn allocate_seq(&self) -> u64 {
            self.counter.fetch_add(1, Ordering::SeqCst)
        }

        fn insert(&self, doc_id: u64) -> u64 {
            let seq = self.allocate_seq();
            let mut entries = self.entries.write().unwrap();
            entries.insert(doc_id, (seq, None));
            seq
        }

        fn delete(&self, doc_id: u64) -> Option<u64> {
            let seq = self.allocate_seq();
            let mut entries = self.entries.write().unwrap();
            if let Some(entry) = entries.get_mut(&doc_id) {
                entry.1 = Some(seq);
                Some(seq)
            } else {
                None
            }
        }

        fn is_visible_at(&self, doc_id: u64, snapshot_seq: u64) -> bool {
            let entries = self.entries.read().unwrap();
            if let Some(&(born, died)) = entries.get(&doc_id) {
                born <= snapshot_seq && died.map_or(true, |d| snapshot_seq < d)
            } else {
                false
            }
        }
    }

    #[test]
    fn test_loom_sequence_log_two_writers_one_reader() {
        loom::model(|| {
            let log = Arc::new(MockSequenceLog::new());
            let l1 = Arc::clone(&log);
            let l2 = Arc::clone(&log);
            let reader = Arc::clone(&log);

            let t1 = thread::spawn(move || {
                l1.insert(42);
            });

            let t2 = thread::spawn(move || {
                l2.insert(43);
            });

            let t_read = thread::spawn(move || {
                let snap = 2;
                let _ = reader.is_visible_at(42, snap);
                let _ = reader.is_visible_at(43, snap);
            });

            t1.join().unwrap();
            t2.join().unwrap();
            t_read.join().unwrap();
        });
    }

    #[test]
    fn test_loom_sequence_log_insert_delete_visibility() {
        loom::model(|| {
            let log = Arc::new(MockSequenceLog::new());
            let writer = Arc::clone(&log);
            let deleter = Arc::clone(&log);
            let reader = Arc::clone(&log);

            let t_write = thread::spawn(move || {
                writer.insert(100);
            });

            let t_del = thread::spawn(move || {
                deleter.delete(100);
            });

            let t_read = thread::spawn(move || {
                let _ = reader.is_visible_at(100, 1);
                let _ = reader.is_visible_at(100, 2);
            });

            t_write.join().unwrap();
            t_del.join().unwrap();
            t_read.join().unwrap();
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
    use contextra_mvcc::{DocId, SequenceLog};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_sequence_log_stress_two_writers_one_reader() {
        let log = Arc::new(parking_lot::RwLock::new(SequenceLog::new()));
        let seq_counter = Arc::new(AtomicU64::new(1));
        let mut handles = Vec::new();

        for i in 0..2u64 {
            let l = Arc::clone(&log);
            let counter = Arc::clone(&seq_counter);
            handles.push(thread::spawn(move || {
                for j in 0..500u64 {
                    let doc_id = DocId::from(i * 1000 + j);
                    let seq = counter.fetch_add(1, Ordering::SeqCst);
                    let mut guard = l.write();
                    guard.record_insert(doc_id, seq);
                }
            }));
        }

        let l_reader = Arc::clone(&log);
        handles.push(thread::spawn(move || {
            for _ in 0..1000 {
                let guard = l_reader.read();
                let _ = guard.is_visible(DocId::from(100u64), 50);
            }
        }));

        for h in handles {
            h.join().expect("thread panicked");
        }
    }
}
