// FILE-CONTEXT
// STAND: 2026-10-16
// ZWECK: Integrationstests für präzise Scope-Fences (ErasureFence) und Invarianten (F-1 bis F-8).

use contextra_ports::StorageEngine;
use contextra_types::TxId;
use contextra_store::lsm::fence::{ErasureFence, FenceError};
use contextra_store::{LsmConfig, LsmStorage};
use tempfile::tempdir;

#[tokio::test]
async fn test_async_runtime_blocking_attempt_rejected() {
    let fence = ErasureFence::new();
    let res = fence.acquire_scope_fence_for(vec![b"coll_a:".to_vec()]);
    assert_eq!(res.unwrap_err(), FenceError::AsyncRuntimeBlockingAttempt);
}

#[test]
fn test_scope_fence_inside_outside_prefix() {
    let fence = ErasureFence::new();
    fence.acquire_scope_fence_for_locked(b"coll_a:".to_vec());

    // Writing outside prefix works
    assert!(fence.check_write_allowed_scope(b"coll_b:doc1").is_ok());

    // Writing inside prefix is rejected
    assert_eq!(
        fence.check_write_allowed_scope(b"coll_a:doc1"),
        Err(FenceError::CollectionDropping)
    );

    fence.release_scope_fence_for_locked(b"coll_a:");
    // After release, writing inside prefix is allowed again
    assert!(fence.check_write_allowed_scope(b"coll_a:doc1").is_ok());
}

#[test]
fn test_scope_fence_prefix_edge_cases() {
    let fence = ErasureFence::new();
    // "a:" vs "ab:"
    fence.acquire_scope_fence_for_locked(b"a:".to_vec());

    // "a:123" is in scope "a:"
    assert_eq!(
        fence.check_write_allowed_scope(b"a:123"),
        Err(FenceError::CollectionDropping)
    );

    // "ab:123" does NOT start with "a:", so it is outside scope
    assert!(fence.check_write_allowed_scope(b"ab:123").is_ok());
}

#[test]
fn test_scope_fence_guard_drop_releases() {
    let fence = ErasureFence::new();

    fence.acquire_scope_fence_for_locked(b"scope:".to_vec());
    assert!(fence.check_write_allowed_scope(b"scope:key").is_err());

    fence.release_scope_fence_for_locked(b"scope:");
    assert!(fence.check_write_allowed_scope(b"scope:key").is_ok());
}

#[test]
fn test_scope_fence_overlapping_scopes_union_ref_count() {
    let fence = ErasureFence::new();
    fence.acquire_scope_fence_for_locked(b"coll_a:".to_vec());

    // Overlapping scope "coll_a:sub:" -> Union with ref_count (F-2/F-3, no Conflict)
    fence.acquire_scope_fence_for_locked(b"coll_a:sub:".to_vec());

    // Disjoint scope "coll_b:" -> Success
    fence.acquire_scope_fence_for_locked(b"coll_b:".to_vec());

    fence.release_scope_fence_for_locked(b"coll_a:");
    // Overlapping "coll_a:sub:" still active via second registration
    assert_eq!(
        fence.check_write_allowed_scope(b"coll_a:sub:doc1"),
        Err(FenceError::CollectionDropping)
    );

    fence.release_scope_fence_for_locked(b"coll_a:sub:");
    // Now coll_a:sub: is released
    assert!(fence.check_write_allowed_scope(b"coll_a:sub:doc1").is_ok());

    fence.release_scope_fence_for_locked(b"coll_b:");
}

#[test]
fn test_doc_lock_atomic_conflict_no_partial_leak() {
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

#[test]
fn test_scope_write_seq_increments_only_in_scope() {
    let fence = ErasureFence::new();
    let initial_seq = fence.scope_write_seq();

    fence.acquire_scope_fence_for_locked(b"coll_a:".to_vec());
    let fenced_seq = fence.scope_write_seq();
    assert!(fenced_seq > initial_seq);

    // Attempt write on foreign key -> sequence does not change
    let _ = fence.check_write_allowed_scope(b"coll_b:doc1");
    assert_eq!(fence.scope_write_seq(), fenced_seq);

    // Attempt write in scope -> sequence increments
    let _ = fence.check_write_allowed_scope(b"coll_a:doc1");
    let new_seq = fence.scope_write_seq();
    assert!(new_seq > fenced_seq);

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

    // 1. Install fence via locked helper / sync call
    storage
        .erasure_fence
        .acquire_scope_fence_for_locked(b"locked:".to_vec());

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
    storage.erasure_fence.release_scope_fence_for_locked(b"locked:");
    let tx_staged = TxId::new(2);
    storage
        .put(tx_staged, b"locked:key4", b"val4")
        .await
        .unwrap();

    storage
        .erasure_fence
        .acquire_scope_fence_for_locked(b"locked:".to_vec());

    let commit_res = storage.commit(tx_staged).await;
    assert!(commit_res.is_err());

    storage.close().await.unwrap();
}
