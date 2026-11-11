//! Standalone persistence port for  (N4).

use crate::error::Result;
use crate::types::TxId;
use bytes::Bytes;
use parking_lot::RwLock;
use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

pub type GraphFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub trait GraphStorage: Send + Sync + 'static {
    fn get<'a>(&'a self, key: &'a [u8]) -> GraphFuture<'a, Result<Option<Bytes>>>;
    fn put<'a>(&'a self, tx: TxId, key: &'a [u8], value: &'a [u8]) -> GraphFuture<'a, Result<()>>;
    fn delete<'a>(&'a self, tx: TxId, key: &'a [u8]) -> GraphFuture<'a, Result<()>>;
    fn scan_prefix<'a>(&'a self, prefix: &'a [u8]) -> GraphFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>>;
    fn commit<'a>(&'a self, tx: TxId) -> GraphFuture<'a, Result<()>>;
    fn flush<'a>(&'a self) -> GraphFuture<'a, Result<()>>;
    fn last_tx_id<'a>(&'a self) -> GraphFuture<'a, Result<TxId>>;
}

#[derive(Debug, Default, Clone)]
pub struct InMemoryGraphStorage {
    map: Arc<RwLock<BTreeMap<Vec<u8>, Vec<u8>>>>,
    last_tx: Arc<AtomicU64>,
}

impl InMemoryGraphStorage {
    pub fn new() -> Self { Self::default() }
}

impl GraphStorage for InMemoryGraphStorage {
    fn get<'a>(&'a self, key: &'a [u8]) -> GraphFuture<'a, Result<Option<Bytes>>> {
        let key = key.to_vec();
        Box::pin(async move {
            let res = self.map.read().get(&key).map(|v| Bytes::copy_from_slice(v));
            Ok(res)
        })
    }
    fn put<'a>(&'a self, tx: TxId, key: &'a [u8], value: &'a [u8]) -> GraphFuture<'a, Result<()>> {
        let key = key.to_vec();
        let value = value.to_vec();
        let tx_raw = tx.inner();
        Box::pin(async move {
            self.map.write().insert(key, value);
            self.last_tx.fetch_max(tx_raw, Ordering::SeqCst);
            Ok(())
        })
    }
    fn delete<'a>(&'a self, tx: TxId, key: &'a [u8]) -> GraphFuture<'a, Result<()>> {
        let key = key.to_vec();
        let tx_raw = tx.inner();
        Box::pin(async move {
            self.map.write().remove(&key);
            self.last_tx.fetch_max(tx_raw, Ordering::SeqCst);
            Ok(())
        })
    }
    fn scan_prefix<'a>(&'a self, prefix: &'a [u8]) -> GraphFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        let prefix = prefix.to_vec();
        Box::pin(async move {
            let guard = self.map.read();
            let matches = guard.range(prefix.clone()..)
                .take_while(|(k, _)| k.starts_with(&prefix))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            Ok(matches)
        })
    }
    fn commit<'a>(&'a self, tx: TxId) -> GraphFuture<'a, Result<()>> {
        let tx_raw = tx.inner();
        Box::pin(async move {
            self.last_tx.fetch_max(tx_raw, Ordering::SeqCst);
            Ok(())
        })
    }
    fn flush<'a>(&'a self) -> GraphFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }
    fn last_tx_id<'a>(&'a self) -> GraphFuture<'a, Result<TxId>> {
        Box::pin(async move {
            let val = self.last_tx.load(Ordering::SeqCst);
            Ok(TxId::new(val))
        })
    }
}

pub mod conformance {
    use super::*;

    pub async fn run_all<S: GraphStorage>(storage: S) {
        let tx = TxId::new(1);
        assert_eq!(storage.get(b"nonexistent").await.unwrap(), None);
        storage.put(tx, b"key1", b"val1").await.unwrap();
        assert_eq!(storage.get(b"key1").await.unwrap(), Some(Bytes::from_static(b"val1")));
        storage.put(tx, b"key1", b"val1_updated").await.unwrap();
        assert_eq!(storage.get(b"key1").await.unwrap(), Some(Bytes::from_static(b"val1_updated")));
        storage.delete(tx, b"key1").await.unwrap();
        assert_eq!(storage.get(b"key1").await.unwrap(), None);
        storage.delete(tx, b"key1").await.unwrap();

        let tx2 = TxId::new(2);
        storage.put(tx2, b"pref:c", b"3").await.unwrap();
        storage.put(tx2, b"pref:a", b"1").await.unwrap();
        storage.put(tx2, b"pref:b", b"2").await.unwrap();
        storage.put(tx2, b"other:x", b"9").await.unwrap();

        let scanned = storage.scan_prefix(b"pref:").await.unwrap();
        assert_eq!(scanned.len(), 3);
        assert_eq!(scanned[0], (b"pref:a".to_vec(), b"1".to_vec()));
        assert_eq!(scanned[1], (b"pref:b".to_vec(), b"2".to_vec()));
        assert_eq!(scanned[2], (b"pref:c".to_vec(), b"3".to_vec()));

        let all = storage.scan_prefix(b"").await.unwrap();
        assert_eq!(all.len(), 4);
        assert_eq!(all[0].0, b"other:x");

        let bin_key = vec![0x00, 0xFF, 0x01];
        storage.put(tx2, &bin_key, b"binary").await.unwrap();
        assert_eq!(storage.get(&bin_key).await.unwrap(), Some(Bytes::from_static(b"binary")));

        storage.commit(tx2).await.unwrap();
        storage.flush().await.unwrap();
        assert!(storage.last_tx_id().await.unwrap().inner() >= 2);
    }
}
