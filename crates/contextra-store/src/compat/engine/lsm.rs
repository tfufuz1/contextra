//! StorageEngine trait implementation for LsmStorage (Bridge adapter).

use crate::lsm::LsmStorage;
use bytes::Bytes;
use contextra_ports::{BoxFuture, StorageEngine, StorageStats};
use contextra_types::{Result, TxId};

impl StorageEngine for LsmStorage {
    fn get<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, Result<Option<Bytes>>> {
        Box::pin(async move {
            self.get(key).await.map_err(Into::into)
        })
    }

    fn get_at_seq<'a>(
        &'a self,
        key: &'a [u8],
        seq_no: u64,
    ) -> BoxFuture<'a, Result<Option<Bytes>>> {
        Box::pin(async move {
            self.get_at_seq(key, seq_no).await.map_err(Into::into)
        })
    }

    fn get_tracked<'a>(
        &'a self,
        tx_id: TxId,
        key: &'a [u8],
    ) -> BoxFuture<'a, Result<Option<Bytes>>> {
        Box::pin(async move {
            crate::lsm::ops::read::get_tracked(self, tx_id, key).await.map_err(Into::into)
        })
    }

    fn get_at_seq_tracked<'a>(
        &'a self,
        tx_id: TxId,
        key: &'a [u8],
        seq: u64,
    ) -> BoxFuture<'a, Result<Option<Bytes>>> {
        Box::pin(async move {
            crate::lsm::ops::read::get_at_seq_tracked(self, tx_id, key, seq).await.map_err(Into::into)
        })
    }

    fn scan_prefix<'a>(
        &'a self,
        prefix: &'a [u8],
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        Box::pin(async move {
            self.scan_prefix(prefix).await.map_err(Into::into)
        })
    }

    fn scan_prefix_tracked<'a>(
        &'a self,
        tx_id: TxId,
        prefix: &'a [u8],
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        Box::pin(async move {
            crate::lsm::ops::read::scan_prefix_tracked(self, tx_id, prefix).await.map_err(Into::into)
        })
    }

    fn supports_ssi_tracking(&self) -> bool {
        true
    }

    fn scan_prefix_bounded<'a>(
        &'a self,
        prefix: &'a [u8],
        limit: usize,
        cursor: Option<&'a [u8]>,
    ) -> BoxFuture<'a, Result<(Vec<(Vec<u8>, Vec<u8>)>, Option<Vec<u8>>)>> {
        Box::pin(async move {
            self.scan_prefix_bounded(prefix, limit, cursor).await.map_err(Into::into)
        })
    }

    fn scan_prefix_at<'a>(
        &'a self,
        prefix: &'a [u8],
        seq_no: u64,
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        Box::pin(async move {
            self.scan_prefix_at(prefix, seq_no).await.map_err(Into::into)
        })
    }

    fn scan_bounded<'a>(
        &'a self,
        start: std::ops::Bound<&'a [u8]>,
        end: std::ops::Bound<&'a [u8]>,
        limit: usize,
        cursor: Option<&'a [u8]>,
    ) -> BoxFuture<'a, Result<(Vec<(Vec<u8>, Vec<u8>)>, Option<Vec<u8>>)>> {
        Box::pin(async move {
            self.scan_bounded(start, end, limit, cursor).await.map_err(Into::into)
        })
    }

    fn scan<'a>(
        &'a self,
        start: std::ops::Bound<&'a [u8]>,
        end: std::ops::Bound<&'a [u8]>,
        limit: Option<usize>,
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        Box::pin(async move {
            self.scan(start, end, limit).await.map_err(Into::into)
        })
    }

    fn put<'a>(&'a self, tx_id: TxId, key: &'a [u8], value: &'a [u8]) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            self.put(tx_id, key, value).await.map_err(Into::into)
        })
    }

    fn put_if_absent<'a>(
        &'a self,
        tx_id: TxId,
        key: &'a [u8],
        value: &'a [u8],
    ) -> BoxFuture<'a, Result<bool>> {
        Box::pin(async move {
            self.put_if_absent(tx_id, key, value).await.map_err(Into::into)
        })
    }

    fn delete_many<'a>(&'a self, tx_id: TxId, keys: Vec<Vec<u8>>) -> BoxFuture<'a, Result<u64>> {
        Box::pin(async move {
            self.delete_many(tx_id, keys).await.map_err(Into::into)
        })
    }

    fn delete_prefix<'a>(&'a self, tx_id: TxId, prefix: &'a [u8]) -> BoxFuture<'a, Result<u64>> {
        Box::pin(async move {
            self.delete_prefix(tx_id, prefix).await.map_err(Into::into)
        })
    }

    fn delete<'a>(&'a self, tx_id: TxId, key: &'a [u8]) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            self.delete(tx_id, key).await.map_err(Into::into)
        })
    }

    fn commit<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            self.commit(tx_id).await.map_err(Into::into)
        })
    }

    fn rollback<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            self.rollback(tx_id).await.map_err(Into::into)
        })
    }

    fn rollback_to_tx<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            crate::lsm::ops::maintenance::rollback_to_tx(self, tx_id).await.map_err(Into::into)
        })
    }

    fn pin_checkpoint<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            self.pin_checkpoint(seq_no).await.map_err(Into::into)
        })
    }

    fn unpin_checkpoint<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            self.unpin_checkpoint(seq_no).await.map_err(Into::into)
        })
    }

    fn flush<'a>(&'a self) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            self.flush().await.map_err(Into::into)
        })
    }

    fn stats<'a>(&'a self) -> BoxFuture<'a, Result<StorageStats>> {
        Box::pin(async move {
            let s = self.stats().await.map_err(|e| contextra_types::ContextraError::from(e))?;
            Ok(s.into())
        })
    }

    fn last_seq_no<'a>(&'a self) -> BoxFuture<'a, Result<u64>> {
        Box::pin(async move {
            self.last_seq_no().await.map_err(Into::into)
        })
    }

    fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, Result<TxId>> {
        Box::pin(async move {
            self.last_tx_id().await.map_err(Into::into)
        })
    }
}

impl From<crate::stats::StorageStats> for StorageStats {
    fn from(s: crate::stats::StorageStats) -> Self {
        Self {
            num_segments: s.num_segments,
            total_size_bytes: s.total_size_bytes,
            memtable_size_bytes: s.memtable_size_bytes,
        }
    }
}

impl From<StorageStats> for crate::stats::StorageStats {
    fn from(s: StorageStats) -> Self {
        Self {
            num_segments: s.num_segments,
            total_size_bytes: s.total_size_bytes,
            memtable_size_bytes: s.memtable_size_bytes,
        }
    }
}
