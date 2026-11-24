//! Compatibility bridge adapting  to .

use crate::error::{GraphError, Result};
use crate::storage::{GraphFuture, GraphStorage};
use crate::types::TxId;
use bytes::Bytes;
use contextra_ports::StorageEngine;
use std::sync::Arc;

impl<T: StorageEngine + ?Sized> GraphStorage for T {
    fn get<'a>(&'a self, key: &'a [u8]) -> GraphFuture<'a, Result<Option<Bytes>>> {
        Box::pin(async move {
            StorageEngine::get(self, key).await.map_err(GraphError::from)
        })
    }
    fn put<'a>(&'a self, tx: TxId, key: &'a [u8], value: &'a [u8]) -> GraphFuture<'a, Result<()>> {
        Box::pin(async move {
            let tx_w = contextra_types::TxId::new(tx.inner());
            StorageEngine::put(self, tx_w, key, value).await.map_err(GraphError::from)
        })
    }
    fn delete<'a>(&'a self, tx: TxId, key: &'a [u8]) -> GraphFuture<'a, Result<()>> {
        Box::pin(async move {
            let tx_w = contextra_types::TxId::new(tx.inner());
            StorageEngine::delete(self, tx_w, key).await.map_err(GraphError::from)
        })
    }
    fn scan_prefix<'a>(&'a self, prefix: &'a [u8]) -> GraphFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        Box::pin(async move {
            StorageEngine::scan_prefix(self, prefix).await.map_err(GraphError::from)
        })
    }
    fn commit<'a>(&'a self, tx: TxId) -> GraphFuture<'a, Result<()>> {
        Box::pin(async move {
            let tx_w = contextra_types::TxId::new(tx.inner());
            StorageEngine::commit(self, tx_w).await.map_err(GraphError::from)
        })
    }
    fn flush<'a>(&'a self) -> GraphFuture<'a, Result<()>> {
        Box::pin(async move {
            StorageEngine::flush(self).await.map_err(GraphError::from)
        })
    }
    fn last_tx_id<'a>(&'a self) -> GraphFuture<'a, Result<TxId>> {
        Box::pin(async move {
            let tx = StorageEngine::last_tx_id(self).await.map_err(GraphError::from)?;
            Ok(TxId::new(tx.inner()))
        })
    }
}

#[derive(Clone)]
pub struct EngineHandle(pub Arc<dyn StorageEngine>);

impl GraphStorage for EngineHandle {
    fn get<'a>(&'a self, key: &'a [u8]) -> GraphFuture<'a, Result<Option<Bytes>>> {
        Box::pin(async move { self.0.get(key).await.map_err(GraphError::from) })
    }
    fn put<'a>(&'a self, tx: TxId, key: &'a [u8], value: &'a [u8]) -> GraphFuture<'a, Result<()>> {
        Box::pin(async move {
            let tx_w = contextra_types::TxId::new(tx.inner());
            self.0.put(tx_w, key, value).await.map_err(GraphError::from)
        })
    }
    fn delete<'a>(&'a self, tx: TxId, key: &'a [u8]) -> GraphFuture<'a, Result<()>> {
        Box::pin(async move {
            let tx_w = contextra_types::TxId::new(tx.inner());
            self.0.delete(tx_w, key).await.map_err(GraphError::from)
        })
    }
    fn scan_prefix<'a>(&'a self, prefix: &'a [u8]) -> GraphFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        Box::pin(async move { self.0.scan_prefix(prefix).await.map_err(GraphError::from) })
    }
    fn commit<'a>(&'a self, tx: TxId) -> GraphFuture<'a, Result<()>> {
        Box::pin(async move {
            let tx_w = contextra_types::TxId::new(tx.inner());
            self.0.commit(tx_w).await.map_err(GraphError::from)
        })
    }
    fn flush<'a>(&'a self) -> GraphFuture<'a, Result<()>> {
        Box::pin(async move { self.0.flush().await.map_err(GraphError::from) })
    }
    fn last_tx_id<'a>(&'a self) -> GraphFuture<'a, Result<TxId>> {
        Box::pin(async move {
            let tx = self.0.last_tx_id().await.map_err(GraphError::from)?;
            Ok(TxId::new(tx.inner()))
        })
    }
}
