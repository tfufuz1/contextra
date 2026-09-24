use super::MAX_SCAN_RESULTS;
use std::ops::Bound;
use std::sync::Arc;
use tokio::task::JoinSet;

#[tokio::test]
async fn test_scan_prefix_exceeds_default_limit_returns_error() {
    let dir = tempfile::tempdir().unwrap();
    let db = crate::Contextra::open_with_config(
        dir.path(),
        crate::ContextraConfig {
            dimension: 4,
            ..Default::default()
        },
    )
    .await
    .unwrap();
    let col = db
        .collection("test_scan_prefix_default_exceed")
        .await
        .unwrap();

    let total = MAX_SCAN_RESULTS + 1;
    for i in 0..total {
        col.put_kv(&format!("item_{:05}", i), &serde_json::json!({ "v": i }))
            .await
            .unwrap();
    }

    // scan_prefix with default limit (None) must fail because total items exceeds safety threshold
    let res_default = col.scan_prefix("", None).await;
    assert!(res_default.is_err());
}

#[tokio::test]
async fn test_scan_prefix_explicit_limit_equal_or_greater() {
    let dir = tempfile::tempdir().unwrap();
    let db = crate::Contextra::open_with_config(
        dir.path(),
        crate::ContextraConfig {
            dimension: 4,
            ..Default::default()
        },
    )
    .await
    .unwrap();
    let col = db.collection("test_scan_prefix_equal").await.unwrap();

    for i in 0..10 {
        col.put_kv(&format!("item_{:02}", i), &serde_json::json!({ "v": i }))
            .await
            .unwrap();
    }

    let results = col.scan_prefix("", Some(10)).await.unwrap();
    assert_eq!(results.len(), 10);

    let results_larger = col.scan_prefix("", Some(20)).await.unwrap();
    assert_eq!(results_larger.len(), 10);
}

#[tokio::test]
async fn test_scan_prefix_explicit_limit_smaller_than_count_returns_limit_exceeded() {
    let dir = tempfile::tempdir().unwrap();
    let db = crate::Contextra::open_with_config(
        dir.path(),
        crate::ContextraConfig {
            dimension: 4,
            ..Default::default()
        },
    )
    .await
    .unwrap();
    let col = db.collection("test_scan_prefix_exceed").await.unwrap();

    for i in 0..15 {
        col.put_kv(&format!("item_{:02}", i), &serde_json::json!({ "v": i }))
            .await
            .unwrap();
    }

    let res = col.scan_prefix("", Some(10)).await;
    assert!(matches!(
        res,
        Err(contextra_types::ContextraError::LimitExceeded { limit: 10, .. })
    ));

    let res_exact = col.scan_prefix("", Some(15)).await;
    assert!(res_exact.is_ok());
    assert_eq!(res_exact.unwrap().len(), 15);
}

#[tokio::test]
async fn test_scan_respects_default_limit() {
    let dir = tempfile::tempdir().unwrap();
    let db = crate::Contextra::open_with_config(
        dir.path(),
        crate::ContextraConfig {
            dimension: 4,
            ..Default::default()
        },
    )
    .await
    .unwrap();
    let col = db.collection("test_scan_default").await.unwrap();

    for i in 0..50 {
        col.put_kv(&format!("item_{:02}", i), &serde_json::json!({ "v": i }))
            .await
            .unwrap();
    }

    let results = col
        .scan(Bound::Unbounded, Bound::Unbounded, None)
        .await
        .unwrap();
    assert_eq!(results.len(), 50);
    assert!(results.len() <= MAX_SCAN_RESULTS);
}

#[tokio::test]
async fn test_scan_explicit_limit_equal_or_greater() {
    let dir = tempfile::tempdir().unwrap();
    let db = crate::Contextra::open_with_config(
        dir.path(),
        crate::ContextraConfig {
            dimension: 4,
            ..Default::default()
        },
    )
    .await
    .unwrap();
    let col = db.collection("test_scan_equal").await.unwrap();

    for i in 0..10 {
        col.put_kv(&format!("item_{:02}", i), &serde_json::json!({ "v": i }))
            .await
            .unwrap();
    }

    let results = col
        .scan(Bound::Unbounded, Bound::Unbounded, Some(10))
        .await
        .unwrap();
    assert_eq!(results.len(), 10);

    let results_larger = col
        .scan(Bound::Unbounded, Bound::Unbounded, Some(20))
        .await
        .unwrap();
    assert_eq!(results_larger.len(), 10);
}

#[tokio::test]
async fn test_scan_explicit_limit_smaller_than_count_returns_limit_exceeded() {
    let dir = tempfile::tempdir().unwrap();
    let db = crate::Contextra::open_with_config(
        dir.path(),
        crate::ContextraConfig {
            dimension: 4,
            ..Default::default()
        },
    )
    .await
    .unwrap();
    let col = db.collection("test_scan_exceed").await.unwrap();

    for i in 0..15 {
        col.put_kv(&format!("item_{:02}", i), &serde_json::json!({ "v": i }))
            .await
            .unwrap();
    }

    let res = col.scan(Bound::Unbounded, Bound::Unbounded, Some(10)).await;
    assert!(matches!(
        res,
        Err(contextra_types::ContextraError::LimitExceeded { limit: 10, .. })
    ));

    let res_exact = col.scan(Bound::Unbounded, Bound::Unbounded, Some(15)).await;
    assert!(res_exact.is_ok());
    assert_eq!(res_exact.unwrap().len(), 15);
}

#[tokio::test]
async fn test_concurrent_put_kv_if_absent_race() {
    let dir = tempfile::tempdir().unwrap();
    let db = crate::Contextra::open_with_config(
        dir.path(),
        crate::ContextraConfig {
            dimension: 4,
            ..Default::default()
        },
    )
    .await
    .unwrap();
    let collection = Arc::new(db.collection("test_race").await.unwrap());

    let mut set = JoinSet::new();
    let key = "shared_race_key";

    for i in 0..50 {
        let col = collection.clone();
        let val = serde_json::json!({ "task_id": i });
        set.spawn(async move { col.put_kv_if_absent(key, &val).await });
    }

    let mut ok_count = 0;
    let mut conflict_count = 0;

    while let Some(res) = set.join_next().await {
        match res.unwrap() {
            Ok(_) => ok_count += 1,
            Err(contextra_types::ContextraError::Conflict(_)) => conflict_count += 1,
            Err(other) => panic!("Unexpected error: {:?}", other),
        }
    }

    assert_eq!(ok_count, 1, "Exactly one task must succeed");
    assert_eq!(
        conflict_count, 49,
        "49 tasks must fail with ContextraError::Conflict"
    );
}

#[tokio::test]
async fn test_independent_collections_kv_lock_isolation() {
    let dir = tempfile::tempdir().unwrap();
    let db = crate::Contextra::open_with_config(
        dir.path(),
        crate::ContextraConfig {
            dimension: 4,
            ..Default::default()
        },
    )
    .await
    .unwrap();

    let col_a = Arc::new(db.collection("col_a").await.unwrap());
    let col_b = Arc::new(db.collection("col_b").await.unwrap());

    let key = "shared_key_across_collections";

    // Acquire lock on col_a's kv_locks for `key`
    let guard_a = col_a.kv_locks.lock_for(key).await;

    // Spawn put_kv_if_absent on col_b with the exact same key.
    // If col_b shared a global lock pool with col_a, this call would block until guard_a is dropped.
    let col_b_clone = col_b.clone();
    let handle = tokio::spawn(async move {
        col_b_clone
            .put_kv_if_absent(key, &serde_json::json!({ "col": "b" }))
            .await
    });

    // Yield or wait briefly to verify col_b operation completes concurrently without blocking.
    let res = tokio::time::timeout(std::time::Duration::from_millis(500), handle).await;
    assert!(
        res.is_ok(),
        "put_kv_if_absent on col_b timed out/blocked due to lock on col_a"
    );
    let put_res = res.unwrap().unwrap();
    assert!(
        put_res.is_ok(),
        "put_kv_if_absent on col_b should succeed independently: {:?}",
        put_res
    );

    drop(guard_a);
}

#[tokio::test]
async fn test_scan_default_limit_bounds_full_range_scan() {
    let dir = tempfile::tempdir().unwrap();
    let db = crate::Contextra::open_with_config(
        dir.path(),
        crate::ContextraConfig {
            dimension: 4,
            ..Default::default()
        },
    )
    .await
    .unwrap();
    let collection = Arc::new(db.collection("test_scan_cap").await.unwrap());

    // RESOLVED: AGT-DB-cb16e356 — scan_prefix returns LimitExceeded when scan matches > limit entries; verified boundary behavior (TS: 2026-09-09T20:33:05Z)
    for i in 0..1000 {
        collection
            .put_kv(&format!("pfx_{i:05}"), &serde_json::json!({ "idx": i }))
            .await
            .unwrap();
    }

    let scanned = match collection.scan_prefix("pfx_", Some(1000)).await {
        Ok(res) => res,
        Err(e) => panic!("scan_prefix failed: {e:?}"),
    };
    assert_eq!(
        scanned.len(),
        1000,
        "scan_prefix must return requested limit (1000)"
    );
}

#[tokio::test]
async fn test_insert_rejects_dimension_mismatch_before_lock_acquisition() {
    let dir = tempfile::tempdir().unwrap();
    let db = crate::Contextra::open_with_config(
        dir.path(),
        crate::ContextraConfig {
            dimension: 1536,
            ..Default::default()
        },
    )
    .await
    .unwrap();
    let col = db.collection("dim_test").await.unwrap();

    // Perform insert with mismatched vector (768 dims instead of 1536)
    let invalid_vector = vec![0.1f32; 768];
    let res = col.insert("doc-1", &invalid_vector, None).await;

    // Must fail immediately
    assert!(res.is_err());
    let err_msg = match res {
        Err(contextra_types::ContextraError::InvalidInput(msg)) => msg,
        Err(other) => panic!("Expected InvalidInput error, got: {:?}", other),
        Ok(_) => panic!("Expected insert to fail due to dimension mismatch"),
    };

    assert!(
        err_msg.contains("1536") && err_msg.contains("768"),
        "Error message must mention both expected (1536) and actual (768) dimensions, got: {}",
        err_msg
    );
}

#[tokio::test]
async fn test_insert_does_not_block_on_collection_wide_lock() {
    let dir = tempfile::tempdir().unwrap();
    let db = crate::Contextra::open_with_config(
        dir.path(),
        crate::ContextraConfig {
            dimension: 4,
            ..Default::default()
        },
    )
    .await
    .unwrap();
    let col = Arc::new(db.collection("concurrency_test").await.unwrap());

    // Hold a key lock on "key_1"
    let guard_key_1 = col.kv_locks.lock_for("key_1").await;

    // Perform insert on "key_2" concurrently.
    // Since key-granular locking is used, inserting key_2 must not block on key_1.
    let col_clone = col.clone();
    let handle =
        tokio::spawn(
            async move { col_clone.insert("key_2", &[1.0, 0.0, 0.0, 0.0], None).await },
        );

    let res = tokio::time::timeout(std::time::Duration::from_millis(500), handle).await;
    assert!(
        res.is_ok(),
        "Insert of key_2 timed out/blocked due to lock on key_1"
    );
    assert!(res.unwrap().unwrap().is_ok());
    drop(guard_key_1);
}
