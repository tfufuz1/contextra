// FILE-CONTEXT
// STAND: 2026-10-12
// ZWECK: Loom-basierter Concurrency-Test für ErasureFence Scope-Sperren und Dokument-Locks.

#![allow(unexpected_cfgs)]

#[cfg(loom)]
mod loom_tests {
    #[path = "../src/lsm/fence.rs"]
    mod fence;

    use fence::ErasureFence;
    use loom::sync::Arc;
    use loom::thread;

    #[test]
    fn test_loom_erasure_fence_concurrent_scopes_and_locks() {
        let mut builder = loom::model::Builder::new();
        builder.preemption_bound = Some(2);

        builder.check(|| {
            let fence = Arc::new(ErasureFence::new());

            let fence_a = Arc::clone(&fence);
            let t1 = thread::spawn(move || {
                if let Ok(guard) = fence_a.acquire_scope_fence_for(vec![b"scope_1:".to_vec()]) {
                    let _ = fence_a.check_write_allowed_scope(b"scope_1:key1");
                    let _ = fence_a.scope_write_seq_for(&guard);
                    drop(guard);
                }
            });

            let fence_b = Arc::clone(&fence);
            let t2 = thread::spawn(move || {
                if let Ok(guard) = fence_b.lock_documents_guarded(vec![b"doc_1".to_vec()]) {
                    let _ = fence_b.check_write_allowed_scope(b"doc_1");
                    drop(guard);
                }
            });

            t1.join().unwrap();
            t2.join().unwrap();

            // Verification after threads complete: fence returns to clean state
            assert!(fence.check_write_allowed_scope(b"scope_1:key1").is_ok());
            assert!(fence.check_write_allowed_scope(b"doc_1").is_ok());
        });
    }
}

#[cfg(not(loom))]
#[test]
fn test_erasure_fence_concurrency_non_loom() {
    use contextra_store::lsm::fence::ErasureFence;
    use std::sync::Arc;
    use std::thread;

    let fence = Arc::new(ErasureFence::new());

    let fence_a = Arc::clone(&fence);
    let t1 = thread::spawn(move || {
        for _ in 0..100 {
            if let Ok(guard) = fence_a.acquire_scope_fence_for(vec![b"scope_1:".to_vec()]) {
                assert!(fence_a.check_write_allowed_scope(b"scope_1:key1").is_err());
                drop(guard);
            }
        }
    });

    let fence_b = Arc::clone(&fence);
    let t2 = thread::spawn(move || {
        for _ in 0..100 {
            if let Ok(guard) = fence_b.lock_documents_guarded(vec![b"doc_1".to_vec()]) {
                assert!(fence_b.check_write_allowed_scope(b"doc_1").is_err());
                drop(guard);
            }
        }
    });

    t1.join().unwrap();
    t2.join().unwrap();

    assert!(fence.check_write_allowed_scope(b"scope_1:key1").is_ok());
    assert!(fence.check_write_allowed_scope(b"doc_1").is_ok());
}
