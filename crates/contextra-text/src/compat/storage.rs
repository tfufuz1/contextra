//! Kompatibilitätsbrücke für `contextra-ports` und `StorageEngine` (SPEC §6.2 text, B04).
// ANCHOR[contextra-text:COMPAT-001]

use crate::storage::{BoxFuture, PostingStore};
use bytes::Bytes;
use contextra_ports::StorageEngine;
use contextra_ids::TxId;

/// Adapter-Newtype für `StorageEngine` zur Vermeidung von Blanket-Impl-Kollisionen.
pub struct StorageBridge<S>(pub S);

impl<S: StorageEngine> PostingStore for StorageBridge<S> {
    fn get<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, crate::error::Result<Option<Bytes>>> {
        Box::pin(async move { self.0.get(key).await.map_err(crate::error::TextError::from) })
    }

    fn get_at_seq<'a>(
        &'a self,
        key: &'a [u8],
        seq: u64,
    ) -> BoxFuture<'a, crate::error::Result<Option<Bytes>>> {
        Box::pin(async move {
            self.0
                .get_at_seq(key, seq)
                .await
                .map_err(crate::error::TextError::from)
        })
    }

    fn put<'a>(
        &'a self,
        tx: TxId,
        key: &'a [u8],
        value: &'a [u8],
    ) -> BoxFuture<'a, crate::error::Result<()>> {
        Box::pin(async move {
            self.0
                .put(contextra_types::TxId::new(tx.inner()), key, value)
                .await
                .map_err(crate::error::TextError::from)
        })
    }

    fn delete<'a>(&'a self, tx: TxId, key: &'a [u8]) -> BoxFuture<'a, crate::error::Result<()>> {
        Box::pin(async move {
            self.0
                .delete(contextra_types::TxId::new(tx.inner()), key)
                .await
                .map_err(crate::error::TextError::from)
        })
    }

    fn commit<'a>(&'a self, tx: TxId) -> BoxFuture<'a, crate::error::Result<()>> {
        Box::pin(async move {
            self.0
                .commit(contextra_types::TxId::new(tx.inner()))
                .await
                .map_err(crate::error::TextError::from)
        })
    }

    fn rollback<'a>(&'a self, tx: TxId) -> BoxFuture<'a, crate::error::Result<()>> {
        Box::pin(async move {
            self.0
                .rollback(contextra_types::TxId::new(tx.inner()))
                .await
                .map_err(crate::error::TextError::from)
        })
    }

    fn rollback_to_tx<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, crate::error::Result<()>> {
        Box::pin(async move {
            self.0
                .rollback_to_tx(contextra_types::TxId::new(tx_id.inner()))
                .await
                .map_err(crate::error::TextError::from)
        })
    }

    fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, crate::error::Result<TxId>> {
        Box::pin(async move {
            self.0
                .last_tx_id()
                .await
                .map(|t| TxId::new(t.inner()))
                .map_err(crate::error::TextError::from)
        })
    }

    fn last_seq_no<'a>(&'a self) -> BoxFuture<'a, crate::error::Result<u64>> {
        Box::pin(async move {
            self.0
                .last_seq_no()
                .await
                .map_err(crate::error::TextError::from)
        })
    }

    fn scan_prefix<'a>(
        &'a self,
        prefix: &'a [u8],
    ) -> BoxFuture<'a, crate::error::Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        Box::pin(async move {
            self.0
                .scan_prefix(prefix)
                .await
                .map_err(crate::error::TextError::from)
        })
    }

    fn scan_prefix_at<'a>(
        &'a self,
        prefix: &'a [u8],
        seq: u64,
    ) -> BoxFuture<'a, crate::error::Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        Box::pin(async move {
            self.0
                .scan_prefix_at(prefix, seq)
                .await
                .map_err(crate::error::TextError::from)
        })
    }
}

// Bestehende direkte Implementierung für Abwärtskompatibilität beibehalten
impl<T: StorageEngine + ?Sized> PostingStore for T {
    fn get<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, crate::error::Result<Option<Bytes>>> {
        Box::pin(async move { self.get(key).await.map_err(crate::error::TextError::from) })
    }

    fn get_at_seq<'a>(
        &'a self,
        key: &'a [u8],
        seq: u64,
    ) -> BoxFuture<'a, crate::error::Result<Option<Bytes>>> {
        Box::pin(async move {
            self.get_at_seq(key, seq)
                .await
                .map_err(crate::error::TextError::from)
        })
    }

    fn put<'a>(
        &'a self,
        tx: TxId,
        key: &'a [u8],
        value: &'a [u8],
    ) -> BoxFuture<'a, crate::error::Result<()>> {
        Box::pin(async move {
            self.put(contextra_types::TxId::new(tx.inner()), key, value)
                .await
                .map_err(crate::error::TextError::from)
        })
    }

    fn delete<'a>(&'a self, tx: TxId, key: &'a [u8]) -> BoxFuture<'a, crate::error::Result<()>> {
        Box::pin(async move {
            self.delete(contextra_types::TxId::new(tx.inner()), key)
                .await
                .map_err(crate::error::TextError::from)
        })
    }

    fn commit<'a>(&'a self, tx: TxId) -> BoxFuture<'a, crate::error::Result<()>> {
        Box::pin(async move {
            self.commit(contextra_types::TxId::new(tx.inner()))
                .await
                .map_err(crate::error::TextError::from)
        })
    }

    fn rollback<'a>(&'a self, tx: TxId) -> BoxFuture<'a, crate::error::Result<()>> {
        Box::pin(async move {
            self.rollback(contextra_types::TxId::new(tx.inner()))
                .await
                .map_err(crate::error::TextError::from)
        })
    }

    fn rollback_to_tx<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, crate::error::Result<()>> {
        Box::pin(async move {
            self.rollback_to_tx(contextra_types::TxId::new(tx_id.inner()))
                .await
                .map_err(crate::error::TextError::from)
        })
    }

    fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, crate::error::Result<TxId>> {
        Box::pin(async move {
            self.last_tx_id()
                .await
                .map(|t| TxId::new(t.inner()))
                .map_err(crate::error::TextError::from)
        })
    }

    fn last_seq_no<'a>(&'a self) -> BoxFuture<'a, crate::error::Result<u64>> {
        Box::pin(async move {
            self.last_seq_no()
                .await
                .map_err(crate::error::TextError::from)
        })
    }

    fn scan_prefix<'a>(
        &'a self,
        prefix: &'a [u8],
    ) -> BoxFuture<'a, crate::error::Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        Box::pin(async move {
            self.scan_prefix(prefix)
                .await
                .map_err(crate::error::TextError::from)
        })
    }

    fn scan_prefix_at<'a>(
        &'a self,
        prefix: &'a [u8],
        seq: u64,
    ) -> BoxFuture<'a, crate::error::Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        Box::pin(async move {
            self.scan_prefix_at(prefix, seq)
                .await
                .map_err(crate::error::TextError::from)
        })
    }
}
