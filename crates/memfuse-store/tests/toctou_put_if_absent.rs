use futures::future::join_all;
use memfuse_core::error::MemFuseError;
use memfuse_core::traits::{BoxFuture, StorageEngine, StorageStats};
use memfuse_core::types::*;
use memfuse_core::Result;
use memfuse_store::lsm::{LsmConfig, LsmStorage};
use std::sync::Arc;

struct DummyStorageEngine;

impl StorageEngine for DummyStorageEngine {
    fn get<'a>(&'a self, _: &'a [u8]) -> BoxFuture<'a, Result<Option<bytes::Bytes>>> {
        Box::pin(async move { Ok(None) })
    }
    fn get_at_seq<'a>(
        &'a self,
        _: &'a [u8],
        _: u64,
    ) -> BoxFuture<'a, Result<Option<bytes::Bytes>>> {
        Box::pin(async move { Ok(None) })
    }
    fn put<'a>(&'a self, _: TxId, _: &'a [u8], _: &'a [u8]) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }
    fn delete<'a>(&'a self, _: TxId, _: &'a [u8]) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }
    fn commit<'a>(&'a self, _: TxId) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }
    fn rollback<'a>(&'a self, _: TxId) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }
    fn rollback_to_tx<'a>(&'a self, _: TxId) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }
    fn flush<'a>(&'a self) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }
    fn stats<'a>(&'a self) -> BoxFuture<'a, Result<StorageStats>> {
        Box::pin(async move {
            Ok(StorageStats {
                num_segments: 0,
                total_size_bytes: 0,
                memtable_size_bytes: 0,
            })
        })
    }
    fn last_seq_no<'a>(&'a self) -> BoxFuture<'a, Result<u64>> {
        Box::pin(async move { Ok(0) })
    }
    fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, Result<TxId>> {
        Box::pin(async move { Ok(TxId(0)) })
    }
    fn pin_checkpoint<'a>(&'a self, _: u64) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }
    fn unpin_checkpoint<'a>(&'a self, _: u64) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }
    fn scan_prefix<'a>(&'a self, _: &'a [u8]) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        Box::pin(async move { Ok(vec![]) })
    }
    fn scan<'a>(
        &'a self,
        _: std::ops::Bound<&'a [u8]>,
        _: std::ops::Bound<&'a [u8]>,
        _: Option<usize>,
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        Box::pin(async move { Ok(vec![]) })
    }
}

#[tokio::test]
async fn proof_trait_default_removed() {
    let dummy = DummyStorageEngine;
    let res = dummy.put_if_absent(TxId::new(1), b"key", b"val").await;
    match res {
        Err(MemFuseError::CapabilityUnsupported { capability, .. }) => {
            assert_eq!(capability, "put_if_absent");
        }
        other => panic!(
            "Expected Err(CapabilityUnsupported) for put_if_absent default implementation, got {:?}",
            other
        ),
    }
}

#[test]
fn proof_put_if_absent_default_trait_removed() {
    let source = include_str!("../../memfuse-core/src/traits/storage.rs");
    let has_toctou = source.contains("fn put_if_absent")
        && source.contains("self.get(key)")
        && source.contains("self.put(tx_id");
    assert!(!has_toctou, "REGRESSION B-1: TOCTOU-Default zurückgekehrt");
}

#[tokio::test]
async fn proof_put_if_absent_sees_uncommitted_staged_write() {
    let temp_dir = tempfile::tempdir().expect("tempdir creation failed");
    let config = LsmConfig {
        path: temp_dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::new(config)
        .await
        .expect("LsmStorage::new failed");

    let key = b"mvcc_key";
    let tx_a = TxId::new(200);
    let tx_b = TxId::new(201);

    // TX-A stages put(key, v1) without commit
    storage
        .put(tx_a, key, b"v1")
        .await
        .expect("put staged failed");

    // TX-B calls put_if_absent(key, v2) - must return Ok(false) because it sees staged write
    let put_if_absent_res = storage
        .put_if_absent(tx_b, key, b"v2")
        .await
        .expect("put_if_absent failed");
    assert!(
        !put_if_absent_res,
        "put_if_absent must return false when uncommitted staged write exists"
    );

    // After commit of TX-A, get(key) returns Some(v1)
    storage.commit(tx_a).await.expect("commit tx_a failed");
    let val = storage.get(key).await.expect("get key failed");
    assert_eq!(val, Some(bytes::Bytes::from_static(b"v1")));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn proof_put_if_absent_atomicity_under_contention() {
    let temp_dir = tempfile::tempdir().expect("tempdir creation failed");
    let config = LsmConfig {
        path: temp_dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = Arc::new(
        LsmStorage::new(config)
            .await
            .expect("LsmStorage::new failed"),
    );

    let key = b"contention_key";
    let mut handles = Vec::new();

    for i in 1..=64u64 {
        let s = Arc::clone(&storage);
        let val = format!("val_{i}").into_bytes();
        handles.push(tokio::spawn(async move {
            let tx_id = TxId::new(i);
            let res = s
                .put_if_absent(tx_id, key, &val)
                .await
                .expect("put_if_absent failed");
            if res {
                s.commit(tx_id).await.expect("commit failed");
            } else {
                let _ = s.rollback(tx_id).await;
            }
            res
        }));
    }

    let results = join_all(handles).await;
    let mut success_count = 0;
    let mut false_count = 0;

    for r in results {
        match r.expect("task panicked") {
            true => success_count += 1,
            false => false_count += 1,
        }
    }

    assert_eq!(
        success_count, 1,
        "Exakt genau 1 Task muss Ok(true) von put_if_absent zurückgeben, got {success_count}"
    );
    assert_eq!(
        false_count, 63,
        "Exakt 63 Tasks müssen Ok(false) zurückgeben, got {false_count}"
    );

    let get_res = storage.get(key).await.expect("get failed");
    assert!(
        get_res.is_some(),
        "storage.get(key) muss Some(_) zurückgeben nach erfolgreichem put_if_absent + commit"
    );
}

#[tokio::test]
async fn proof_put_if_absent_atomicity() {
    let temp_dir = tempfile::tempdir().expect("tempdir creation failed");
    let config = LsmConfig {
        path: temp_dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = Arc::new(LsmStorage::new(config).await.unwrap());

    let key = b"atomic_key";
    let tx_a = TxId(100);
    let tx_b = TxId(101);

    let s1 = storage.clone();
    let s2 = storage.clone();

    let task_a = tokio::spawn(async move { s1.put_if_absent(tx_a, key, b"val_a").await });

    let task_b = tokio::spawn(async move { s2.put_if_absent(tx_b, key, b"val_b").await });

    let (res_a, res_b) = tokio::join!(task_a, task_b);
    let ok_a = res_a.unwrap().unwrap();
    let ok_b = res_b.unwrap().unwrap();

    // Exactly one operation should succeed in inserting the key
    assert!(
        (ok_a && !ok_b) || (!ok_a && ok_b),
        "Expected exactly one task to return true from put_if_absent, got ok_a={}, ok_b={}",
        ok_a,
        ok_b
    );
}
