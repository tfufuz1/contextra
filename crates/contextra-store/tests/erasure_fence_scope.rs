// FILE-CONTEXT
// STAND: 2026-10-12
// ZWECK: Tests für präzise Scope-Fences (ErasureFence) und Invarianten.

use contextra_types::TxId;
use contextra_ports::StorageEngine;
use contextra_store::lsm::fence::{ErasureFence, FenceError};
use contextra_store::{LsmConfig, LsmStorage};
use tempfile::tempdir;

#[tokio::test]
async fn test_scope_fence_inside_outside_prefix() {
    let fence = ErasureFence::new();
    let guard = fence
        .acquire_scope_fence_for(vec![b"coll_a:".to_vec()])
        .unwrap();

    // Writing outside prefix works
    assert!(fence.check_write_allowed_scope(b"coll_b:doc1").is_ok());

    // Writing inside prefix is rejected
    assert_eq!(
        fence.check_write_allowed_scope(b"coll_a:doc1"),
        Err(FenceError::CollectionDropping)
    );

    drop(guard);
    // After guard drop, writing inside prefix is allowed again
    assert!(fence.check_write_allowed_scope(b"coll_a:doc1").is_ok());
}

#[tokio::test]
async fn test_scope_fence_prefix_edge_cases() {
    let fence = ErasureFence::new();
    // "a:" vs "ab:"
    let _guard = fence.acquire_scope_fence_for(vec![b"a:".to_vec()]).unwrap();

    // "a:123" is in scope "a:"
    assert_eq!(
        fence.check_write_allowed_scope(b"a:123"),
        Err(FenceError::CollectionDropping)
    );

    // "ab:123" does NOT start with "a:", so it is outside scope
    assert!(fence.check_write_allowed_scope(b"ab:123").is_ok());
}

#[tokio::test]
async fn test_scope_fence_guard_drop_releases() {
    let fence = ErasureFence::new();

    let res: Result<(), FenceError> = (|| {
        let _guard = fence.acquire_scope_fence_for(vec![b"scope:".to_vec()])?;
        assert!(fence.check_write_allowed_scope(b"scope:key").is_err());
        // Simulate error path inside closure
        Err(FenceError::Conflict)
    })();

    assert!(res.is_err());
    // Guard should be dropped when closure returned, releasing fence
    assert!(fence.check_write_allowed_scope(b"scope:key").is_ok());
}

#[tokio::test]
async fn test_scope_fence_overlapping_scopes_conflict() {
    let fence = ErasureFence::new();
    let _guard1 = fence
        .acquire_scope_fence_for(vec![b"coll_a:".to_vec()])
        .unwrap();

    // Overlapping scope "coll_a:sub:" -> Conflict
    let res = fence.acquire_scope_fence_for(vec![b"coll_a:sub:".to_vec()]);
    assert_eq!(res.unwrap_err(), FenceError::Conflict);

    // Disjoint scope "coll_b:" -> Success
    let guard2 = fence.acquire_scope_fence_for(vec![b"coll_b:".to_vec()]);
    assert!(guard2.is_ok());
}

#[tokio::test]
async fn test_doc_lock_atomic_conflict_no_partial_leak() {
    let fence = ErasureFence::new();
    // Pre-lock doc1
    let _guard1 = fence
        .lock_documents_guarded(vec![b"doc1".to_vec()])
        .unwrap();

    // Attempt locking [doc2, doc1] -> should fail cleanly
    let res = fence.lock_documents_guarded(vec![b"doc2".to_vec(), b"doc1".to_vec()]);
    assert_eq!(res.err(), Some(FenceError::Conflict));

    // Verify doc2 was NOT partially locked
    assert!(fence.check_write_allowed_scope(b"doc2").is_ok());
}

#[tokio::test]
async fn test_scope_write_seq_increments_only_in_scope() {
    let fence = ErasureFence::new();
    let guard = fence
        .acquire_scope_fence_for(vec![b"coll_a:".to_vec()])
        .unwrap();

    let initial_seq = fence.scope_write_seq_for(&guard);

    // Attempt write on foreign key -> sequence does not change
    let _ = fence.check_write_allowed_scope(b"coll_b:doc1");
    assert_eq!(fence.scope_write_seq_for(&guard), initial_seq);

    // Attempt write in scope -> sequence increments
    let _ = fence.check_write_allowed_scope(b"coll_a:doc1");
    let new_seq = fence.scope_write_seq_for(&guard);
    assert_ne!(new_seq, initial_seq);

    let snap_seq = fence.snapshot_write_seq(&[b"coll_a:".to_vec()]);
    assert!(snap_seq >= new_seq);
}

#[tokio::test]
async fn test_all_write_paths_respect_fence() {
    let tmp = tempdir().unwrap();
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::open(config).await.unwrap();

    let tx_id = TxId::new(1);

    // 1. Path 'put'
    let _guard = storage
        .erasure_fence
        .acquire_scope_fence_for(vec![b"locked:".to_vec()])
        .unwrap();

    let put_res = storage.put(tx_id, b"locked:key1", b"val1").await;
    assert!(put_res.is_err());
    assert!(storage.put(tx_id, b"unlocked:key1", b"val1").await.is_ok());

    // 2. Path 'put_if_absent'
    let pia_res = storage.put_if_absent(tx_id, b"locked:key2", b"val2").await;
    assert!(pia_res.is_err());
    assert!(storage
        .put_if_absent(tx_id, b"unlocked:key2", b"val2")
        .await
        .unwrap());

    // 3. Path 'delete'
    let del_res = storage.delete(tx_id, b"locked:key1").await;
    assert!(del_res.is_err());
    assert!(storage.delete(tx_id, b"unlocked:key2").await.is_ok());

    // 4. Path 'delete_many' (Batch commit)
    let del_many_res = storage
        .delete_many(tx_id, vec![b"locked:key3".to_vec()])
        .await;
    assert!(del_many_res.is_err());

    // 5. Path 'delete_prefix'
    let del_prefix_res = storage.delete_prefix(tx_id, b"locked:").await;
    assert!(del_prefix_res.is_err());

    // 6. Path 'commit' (when operation staged before fence)
    drop(_guard);
    let tx_staged = TxId::new(2);
    storage
        .put(tx_staged, b"locked:key4", b"val4")
        .await
        .unwrap();

    let _guard_again = storage
        .erasure_fence
        .acquire_scope_fence_for(vec![b"locked:".to_vec()])
        .unwrap();

    let commit_res = storage.commit(tx_staged).await;
    assert!(commit_res.is_err());

    storage.close().await.unwrap();
}
