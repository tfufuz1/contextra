//! # Bounded Scan Tests (Ring 1)
//!
//! ZWECK: Pruefung von Bereichs- und Praefix-Scans mit Limits und Cursorn.

use super::*;
use std::ops::Bound;

#[tokio::test]
async fn test_scan_range() {
    let (storage, _tmp) = test_storage().await;

    let tx1 = TxId::new(1);
    storage.put(tx1, b"a", b"1").await.unwrap();
    storage.put(tx1, b"b", b"2").await.unwrap();
    storage.put(tx1, b"c", b"3").await.unwrap();
    storage.put(tx1, b"d", b"4").await.unwrap();
    storage.commit(tx1).await.unwrap();

    let results = storage
        .scan(
            Bound::Included(b"b".as_ref()),
            Bound::Excluded(b"d".as_ref()),
            None,
        )
        .await
        .unwrap();

    assert_eq!(results.len(), 2);
    assert_eq!(results[0], (b"b".to_vec(), b"2".to_vec()));
    assert_eq!(results[1], (b"c".to_vec(), b"3".to_vec()));
}

#[tokio::test]
async fn test_bounded_scan_and_prefix_bounded_limits_candidate_evaluation() {
    let (storage, _tmp) = test_storage().await;
    let tx = TxId::new(1);

    for i in 0..100 {
        let key = format!("k:{:03}", i);
        let val = format!("val:{:03}", i);
        storage
            .put(tx, key.as_bytes(), val.as_bytes())
            .await
            .unwrap();
    }
    storage.commit(tx).await.unwrap();

    let (res_range, cur_range) = storage
        .scan_bounded(Bound::Unbounded, Bound::Unbounded, 10, None)
        .await
        .unwrap();
    assert_eq!(res_range.len(), 10);
    assert_eq!(res_range[0].0, b"k:000");
    assert_eq!(res_range[9].0, b"k:009");
    assert_eq!(cur_range, Some(b"k:009".to_vec()));

    let (res_prefix, cur_prefix) = storage.scan_prefix_bounded(b"k:", 10, None).await.unwrap();
    assert_eq!(res_prefix.len(), 10);
    assert_eq!(res_prefix[0].0, b"k:000");
    assert_eq!(res_prefix[9].0, b"k:009");
    assert_eq!(cur_prefix, Some(b"k:009".to_vec()));
}

#[tokio::test]
async fn test_scan_bounded_respects_accumulator_ceiling_with_wide_range() {
    let (storage, _tmp) = test_storage().await;
    let tx = TxId::new(1);

    for i in 0..100 {
        let key = format!("k:{:03}", i);
        let val = format!("val:{:03}", i);
        storage
            .put(tx, key.as_bytes(), val.as_bytes())
            .await
            .unwrap();
    }
    storage.commit(tx).await.unwrap();

    let (batch, next_cur) = storage
        .scan_bounded(Bound::Unbounded, Bound::Unbounded, 5, None)
        .await
        .unwrap();

    assert_eq!(batch.len(), 5);
    assert_eq!(batch[0].0, b"k:000");
    assert_eq!(batch[4].0, b"k:004");
    assert_eq!(next_cur, Some(b"k:004".to_vec()));
}

#[tokio::test]
async fn test_scan_bounded_pagination_matches_full_scan() {
    let (storage, _tmp) = test_storage().await;
    let tx = TxId::new(1);

    for i in 0..25 {
        let key = format!("k:{:02}", i);
        let val = format!("val:{:02}", i);
        storage
            .put(tx, key.as_bytes(), val.as_bytes())
            .await
            .unwrap();
    }
    storage.commit(tx).await.unwrap();

    let (p1, cur1) = storage
        .scan_bounded(Bound::Unbounded, Bound::Unbounded, 10, None)
        .await
        .unwrap();
    assert_eq!(p1.len(), 10);
    assert_eq!(p1[0].0, b"k:00");
    assert_eq!(p1[9].0, b"k:09");
    assert!(cur1.is_some());
    let cur1_val = cur1.unwrap();
    assert_eq!(cur1_val, b"k:09");

    let (p2, cur2) = storage
        .scan_bounded(Bound::Unbounded, Bound::Unbounded, 10, Some(&cur1_val))
        .await
        .unwrap();
    assert_eq!(p2.len(), 10);
    assert_eq!(p2[0].0, b"k:10");
    assert_eq!(p2[9].0, b"k:19");
    assert!(cur2.is_some());
    let cur2_val = cur2.unwrap();
    assert_eq!(cur2_val, b"k:19");

    let (p3, cur3) = storage
        .scan_bounded(Bound::Unbounded, Bound::Unbounded, 10, Some(&cur2_val))
        .await
        .unwrap();
    assert_eq!(p3.len(), 5);
    assert_eq!(p3[0].0, b"k:20");
    assert_eq!(p3[4].0, b"k:24");
    assert!(cur3.is_none());
}

#[tokio::test]
async fn test_scan_prefix_bounded_pagination() {
    let (storage, _tmp) = test_storage().await;
    let tx = TxId::new(1);

    for i in 0..25 {
        let key = format!("k:{:02}", i);
        let val = format!("val:{:02}", i);
        storage
            .put(tx, key.as_bytes(), val.as_bytes())
            .await
            .unwrap();
    }
    storage.commit(tx).await.unwrap();

    let (p1, cur1) = storage.scan_prefix_bounded(b"k:", 10, None).await.unwrap();
    assert_eq!(p1.len(), 10);
    assert_eq!(p1[0].0, b"k:00");
    assert_eq!(p1[9].0, b"k:09");
    assert!(cur1.is_some());
    let cur1_val = cur1.unwrap();
    assert_eq!(cur1_val, b"k:09");

    let (p2, cur2) = storage
        .scan_prefix_bounded(b"k:", 10, Some(&cur1_val))
        .await
        .unwrap();
    assert_eq!(p2.len(), 10);
    assert_eq!(p2[0].0, b"k:10");
    assert_eq!(p2[9].0, b"k:19");
    assert!(cur2.is_some());
    let cur2_val = cur2.unwrap();
    assert_eq!(cur2_val, b"k:19");

    let (p3, cur3) = storage
        .scan_prefix_bounded(b"k:", 10, Some(&cur2_val))
        .await
        .unwrap();
    assert_eq!(p3.len(), 5);
    assert_eq!(p3[0].0, b"k:20");
    assert_eq!(p3[4].0, b"k:24");
    assert!(cur3.is_none());
}

#[tokio::test]
async fn test_scan_bounded_respects_limit_and_cursor() {
    let (storage, _tmp) = test_storage().await;
    let tx = TxId::new(1);

    for i in 0..25 {
        let key = format!("k:{:02}", i);
        let val = format!("val:{:02}", i);
        storage
            .put(tx, key.as_bytes(), val.as_bytes())
            .await
            .unwrap();
    }
    storage.commit(tx).await.unwrap();

    let (p1, cur1) = storage
        .scan_bounded(Bound::Unbounded, Bound::Unbounded, 10, None)
        .await
        .unwrap();
    assert_eq!(p1.len(), 10);
    assert_eq!(p1[0].0, b"k:00");
    assert_eq!(p1[9].0, b"k:09");
    assert!(cur1.is_some());
    let cur1_val = cur1.unwrap();
    assert_eq!(cur1_val, b"k:09");

    let (p2, cur2) = storage
        .scan_bounded(Bound::Unbounded, Bound::Unbounded, 10, Some(&cur1_val))
        .await
        .unwrap();
    assert_eq!(p2.len(), 10);
    assert_eq!(p2[0].0, b"k:10");
    assert_eq!(p2[9].0, b"k:19");
    assert!(cur2.is_some());
    let cur2_val = cur2.unwrap();
    assert_eq!(cur2_val, b"k:19");

    let (p3, cur3) = storage
        .scan_bounded(Bound::Unbounded, Bound::Unbounded, 10, Some(&cur2_val))
        .await
        .unwrap();
    assert_eq!(p3.len(), 5);
    assert_eq!(p3[0].0, b"k:20");
    assert_eq!(p3[4].0, b"k:24");
    assert!(cur3.is_none());
}

#[tokio::test]
async fn test_scan_bounded_rejects_oversized_internal_merge() {
    let (storage, _tmp) = test_storage().await;
    let tx = TxId::new(1);

    for i in 0..100 {
        let key = format!("k:{:03}", i);
        let val = format!("val:{:03}", i);
        storage
            .put(tx, key.as_bytes(), val.as_bytes())
            .await
            .unwrap();
    }
    storage.commit(tx).await.unwrap();

    let (batch, next_cur) = storage
        .scan_bounded(Bound::Unbounded, Bound::Unbounded, 5, None)
        .await
        .unwrap();
    assert_eq!(batch.len(), 5);
    assert!(next_cur.is_some());
}
