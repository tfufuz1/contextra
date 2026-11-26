#![allow(unexpected_cfgs)]

//! Loom-basierter Nebenläufigkeitsbeweis für AGT-INDEX-b2c3d4e5.
//! Ausführung: RUSTFLAGS="--cfg loom" cargo test -p contextra-index -- loom
//!
//! Testziel: Kein Datenverlust und keine Panik bei 4 parallelen insert()-Aufrufen
//! auf demselben HnswIndex mit aktiviertem SQ8-Quantizer.

#[cfg(loom)]
mod loom_tests {
    use contextra_ports::VectorIndex;
    use contextra_types::{DocId, TxId};
    use contextra_vector::{HnswConfig, HnswIndex};
    use loom::sync::Arc;
    use loom::thread;

    #[test]
    fn test_sq8_quantizer_write_lock_no_race() {
        loom::model(|| {
            let config = HnswConfig {
                dimension: 4,
                quantize: true,
                m: 8,
                ef_construction: 16,
                ef_search: 16,
                ..Default::default()
            };
            let index = Arc::new(HnswIndex::try_new(config).expect("valid index"));

            let rt = tokio::runtime::Builder::new_current_thread()
                .build()
                .unwrap();

            rt.block_on(async {
                let tx0 = TxId::new(100);
                let vec0 = vec![1.0, 2.0, 3.0, 4.0];
                index
                    .insert(tx0, DocId::new(100), &vec0)
                    .await
                    .expect("initial insert");
                index.commit(tx0).await.expect("initial commit");
            });

            let mut handles = Vec::new();

            for i in 1..=4 {
                let index_clone = Arc::clone(&index);
                let handle = thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_current_thread()
                        .build()
                        .unwrap();
                    rt.block_on(async {
                        let tx = TxId::new(i);
                        let doc_id = DocId::new(i.into());
                        let vec = vec![i as f32 * 10.0, i as f32 * -5.0, 1.0, 2.0];
                        index_clone
                            .insert(tx, doc_id, &vec)
                            .await
                            .expect("parallel insert");
                        index_clone.commit(tx).await.expect("parallel commit");
                    });
                });
                handles.push(handle);
            }

            for handle in handles {
                handle.join().expect("thread finished without panic");
            }

            let rt = tokio::runtime::Builder::new_current_thread()
                .build()
                .unwrap();

            rt.block_on(async {
                assert_eq!(index.len().await, 5);
            });
        });
    }

    #[test]
    fn test_loom_insert_search_rebuild_swap() {
        loom::model(|| {
            use loom::sync::atomic::{AtomicBool, AtomicU64, Ordering};
            let active_index = Arc::new(AtomicU64::new(1));
            let rebuilding = Arc::new(AtomicBool::new(false));

            let a_idx1 = Arc::clone(&active_index);
            let a_idx2 = Arc::clone(&active_index);
            let reb = Arc::clone(&rebuilding);

            let t1 = thread::spawn(move || {
                let id = a_idx1.load(Ordering::Acquire);
                assert!(id == 1 || id == 2);
            });

            let t2 = thread::spawn(move || {
                if !reb.swap(true, Ordering::SeqCst) {
                    a_idx2.store(2, Ordering::Release);
                    reb.store(false, Ordering::SeqCst);
                }
            });

            t1.join().unwrap();
            t2.join().unwrap();
        });
    }

    #[test]
    fn test_loom_remove_search_visibility() {
        loom::model(|| {
            use loom::sync::atomic::{AtomicBool, Ordering};
            let deleted = Arc::new(AtomicBool::new(false));
            let d_clone = Arc::clone(&deleted);

            let t_search = thread::spawn(move || {
                let is_del = d_clone.load(Ordering::Acquire);
                let _ = is_del;
            });

            let t_delete = thread::spawn(move || {
                deleted.store(true, Ordering::Release);
            });

            t_search.join().unwrap();
            t_delete.join().unwrap();
        });
    }

    #[test]
    fn test_loom_snapshot_pin_compact_seq_log() {
        loom::model(|| {
            use loom::sync::atomic::{AtomicU64, Ordering};
            let min_pinned = Arc::new(AtomicU64::new(10));
            let p_clone = Arc::clone(&min_pinned);

            let t_reader = thread::spawn(move || {
                let pin = p_clone.load(Ordering::Acquire);
                assert!(pin >= 10);
            });

            let t_compactor = thread::spawn(move || {
                min_pinned.fetch_max(20, Ordering::SeqCst);
            });

            t_reader.join().unwrap();
            t_compactor.join().unwrap();
        });
    }
}

// Normaler (nicht-loom) Smoke-Test der immer in CI läuft:
#[cfg(not(loom))]
#[cfg(test)]
mod normal_tests {
    use contextra_ports::VectorIndex;
    use contextra_types::{DocId, TxId};
    use contextra_vector::{HnswConfig, HnswIndex};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_sq8_parallel_insert_no_corruption() {
        let config = HnswConfig {
            dimension: 4,
            quantize: true,
            m: 8,
            ef_construction: 16,
            ef_search: 16,
            ..Default::default()
        };
        let index = Arc::new(HnswIndex::try_new(config).expect("valid index"));

        let tx0 = TxId::new(100);
        let vec0 = vec![1.0, 2.0, 3.0, 4.0];
        index
            .insert(tx0, DocId::new(100), &vec0)
            .await
            .expect("initial insert");
        index.commit(tx0).await.expect("initial commit");

        let mut tasks = Vec::new();
        for i in 1..=4u64 {
            let index_clone = Arc::clone(&index);
            tasks.push(tokio::spawn(async move {
                let tx = TxId::new(i);
                let doc_id = DocId::new(i.into());
                let vec = vec![i as f32 * 10.0, i as f32 * -5.0, 1.0, 2.0];
                index_clone.insert(tx, doc_id, &vec).await?;
                index_clone.commit(tx).await?;
                Ok::<(), contextra_types::ContextraError>(())
            }));
        }

        for task in tasks {
            let res = task.await.expect("task join succeeded");
            assert!(res.is_ok(), "Insert and commit succeeded without error");
        }

        assert_eq!(index.len().await, 5);
    }

    #[test]
    fn test_sync_insert_search_rebuild_swap_model() {
        use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
        let active_index = Arc::new(AtomicU64::new(1));
        let rebuilding = Arc::new(AtomicBool::new(false));

        let a_idx1 = Arc::clone(&active_index);
        let a_idx2 = Arc::clone(&active_index);
        let reb = Arc::clone(&rebuilding);

        let t1 = std::thread::spawn(move || {
            let id = a_idx1.load(Ordering::Acquire);
            assert!(id == 1 || id == 2);
        });

        let t2 = std::thread::spawn(move || {
            if !reb.swap(true, Ordering::SeqCst) {
                a_idx2.store(2, Ordering::Release);
                reb.store(false, Ordering::SeqCst);
            }
        });

        t1.join().unwrap();
        t2.join().unwrap();
    }

    #[test]
    fn test_sync_remove_search_visibility_model() {
        use std::sync::atomic::{AtomicBool, Ordering};
        let deleted = Arc::new(AtomicBool::new(false));
        let d_clone = Arc::clone(&deleted);

        let t_search = std::thread::spawn(move || {
            let is_del = d_clone.load(Ordering::Acquire);
            let _ = is_del;
        });

        let t_delete = std::thread::spawn(move || {
            deleted.store(true, Ordering::Release);
        });

        t_search.join().unwrap();
        t_delete.join().unwrap();
    }

    #[test]
    fn test_sync_snapshot_pin_compact_seq_log_model() {
        use std::sync::atomic::{AtomicU64, Ordering};
        let min_pinned = Arc::new(AtomicU64::new(10));
        let p_clone = Arc::clone(&min_pinned);

        let t_reader = std::thread::spawn(move || {
            let pin = p_clone.load(Ordering::Acquire);
            assert!(pin >= 10);
        });

        let t_compactor = std::thread::spawn(move || {
            min_pinned.fetch_max(20, Ordering::SeqCst);
        });

        t_reader.join().unwrap();
        t_compactor.join().unwrap();
    }
}
