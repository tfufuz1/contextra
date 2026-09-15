use memfuse_core::error::MemFuseError;
use memfuse_core::traits::{BoxFuture, StorageEngine, StorageStats};
use memfuse_core::types::*;
use memfuse_core::Result;
use memfuse_store::lsm::{LsmConfig, LsmStorage};
use std::sync::Arc;

struct DummyStorageEngine;

impl StorageEngine for DummyStorageEngine {
    fn get<'a>(&'a self, _: &'a [u8]) -> BoxFuture<'a, Result<Option<Vec<u8>>>> {
        Box::pin(async move { Ok(None) })
    }
    fn get_at_seq<'a>(&'a self, _: &'a [u8], _: u64) -> BoxFuture<'a, Result<Option<Vec<u8>>>> {
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
    let res = dummy.put_if_absent(TxId(1), b"key", b"val").await;
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

#[tokio::test]
async fn proof_put_if_absent_atomicity() {
    let config = LsmConfig::default();
    let storage = Arc::new(LsmStorage::new(config).await.unwrap());

    let key = b"atomic_key";
    let tx_a = TxId(100);
    let tx_b = TxId(101);

    let s1 = storage.clone();
    let s2 = storage.clone();

    let task_a = tokio::spawn(async move {
        s1.put_if_absent(tx_a, key, b"val_a").await
    });

    let task_b = tokio::spawn(async move {
        s2.put_if_absent(tx_b, key, b"val_b").await
    });

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
