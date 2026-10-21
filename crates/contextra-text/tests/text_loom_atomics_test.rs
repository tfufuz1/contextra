#![allow(unexpected_cfgs)]

//! Loom & Non-Loom Atomic Visibility Test for InvertedIndex Statistics

#[cfg(loom)]
mod loom_tests {
    use loom::sync::atomic::{AtomicU64, Ordering};
    use loom::sync::Arc;
    use loom::thread;

    struct AtomicStatsModel {
        total_docs: AtomicU64,
        total_tokens: AtomicU64,
        avg_doc_len_x1000: AtomicU64,
    }

    impl AtomicStatsModel {
        fn new() -> Self {
            Self {
                total_docs: AtomicU64::new(0),
                total_tokens: AtomicU64::new(0),
                avg_doc_len_x1000: AtomicU64::new(0),
            }
        }

        fn commit_doc(&self, num_tokens: u64) {
            let docs = self.total_docs.fetch_add(1, Ordering::SeqCst) + 1;
            let tokens = self.total_tokens.fetch_add(num_tokens, Ordering::SeqCst) + num_tokens;
            let avg = (tokens as f64 / docs as f64 * 1000.0) as u64;
            self.avg_doc_len_x1000.store(avg, Ordering::SeqCst);
        }

        fn read_stats(&self) -> (u64, u64) {
            let docs = self.total_docs.load(Ordering::Acquire);
            let avg = self.avg_doc_len_x1000.load(Ordering::Acquire);
            (docs, avg)
        }
    }

    #[test]
    fn test_loom_index_stats_atomic_visibility() {
        loom::model(|| {
            let model = Arc::new(AtomicStatsModel::new());

            let model_writer1 = Arc::clone(&model);
            let h1 = thread::spawn(move || {
                model_writer1.commit_doc(100);
            });

            let model_writer2 = Arc::clone(&model);
            let h2 = thread::spawn(move || {
                model_writer2.commit_doc(200);
            });

            let model_reader = Arc::clone(&model);
            let h3 = thread::spawn(move || {
                let (docs, _avg) = model_reader.read_stats();
                assert!(
                    docs <= 2,
                    "Reader MUST observe valid total_docs within bounds"
                );
            });

            h1.join().unwrap();
            h2.join().unwrap();
            h3.join().unwrap();

            let (docs, _avg) = model.read_stats();
            assert_eq!(docs, 2, "Final committed total_docs MUST equal 2");
        });
    }
}

#[cfg(not(loom))]
#[cfg(test)]
mod normal_tests {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Arc;

    struct AtomicStatsModel {
        total_docs: AtomicU64,
        total_tokens: AtomicU64,
        avg_doc_len_x1000: AtomicU64,
    }

    impl AtomicStatsModel {
        fn new() -> Self {
            Self {
                total_docs: AtomicU64::new(0),
                total_tokens: AtomicU64::new(0),
                avg_doc_len_x1000: AtomicU64::new(0),
            }
        }

        fn commit_doc(&self, num_tokens: u64) {
            let docs = self.total_docs.fetch_add(1, Ordering::SeqCst) + 1;
            let tokens = self.total_tokens.fetch_add(num_tokens, Ordering::SeqCst) + num_tokens;
            let avg = (tokens as f64 / docs as f64 * 1000.0) as u64;
            self.avg_doc_len_x1000.store(avg, Ordering::SeqCst);
        }

        fn read_stats(&self) -> (u64, u64) {
            let docs = self.total_docs.load(Ordering::Acquire);
            let avg = self.avg_doc_len_x1000.load(Ordering::Acquire);
            (docs, avg)
        }
    }

    #[tokio::test]
    async fn test_index_stats_atomic_visibility_non_loom() {
        let model = Arc::new(AtomicStatsModel::new());

        let mut handles = Vec::new();
        for i in 1..=10u64 {
            let m = Arc::clone(&model);
            handles.push(tokio::spawn(async move {
                m.commit_doc(i * 10);
            }));
        }

        for h in handles {
            h.await.expect("task finishes cleanly");
        }

        let (docs, avg) = model.read_stats();
        assert_eq!(docs, 10, "Total docs must equal 10 after 10 commits");
        assert!(avg > 0, "Avg doc length must be strictly positive");
    }
}
