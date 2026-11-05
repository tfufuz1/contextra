use contextra_engine::collection::StripedDocLocks;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// SPEC: T-LOCK-2
#[tokio::test]
async fn test_striped_document_locks_and_delete_race_prevention() {
    let striped_locks = Arc::new(StripedDocLocks::new());

    let doc_tag_1 = [1u8; 16];
    let doc_tag_2 = [2u8; 16];

    let shard_1 = striped_locks.shard_index(&doc_tag_1);
    let shard_2 = striped_locks.shard_index(&doc_tag_2);

    assert!(shard_1 < 1024);
    assert!(shard_2 < 1024);

    let lock_held_during_persistence = Arc::new(AtomicBool::new(false));
    let concurrent_access_detected = Arc::new(AtomicBool::new(false));

    let locks_clone = striped_locks.clone();
    let lock_held = lock_held_during_persistence.clone();
    let conflict = concurrent_access_detected.clone();

    let delete_task = tokio::spawn(async move {
        let lock = locks_clone.get_lock(&doc_tag_1);
        let _guard = lock.lock().await;

        lock_held.store(true, Ordering::SeqCst);
        tokio::time::sleep(Duration::from_millis(50)).await;
        lock_held.store(false, Ordering::SeqCst);
    });

    let writer_task = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(10)).await;
        let lock = striped_locks.get_lock(&doc_tag_1);
        if lock.try_lock().is_ok() {
            conflict.store(true, Ordering::SeqCst);
        }
    });

    let _ = tokio::join!(delete_task, writer_task);
    assert!(!concurrent_access_detected.load(Ordering::SeqCst));
}
