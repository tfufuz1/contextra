//! StorageEngine trait implementation for TenantScopedStorage (Bridge adapter).

use bytes::Bytes;
use contextra_ports::{BoxFuture, StorageEngine, StorageStats};
use contextra_types::{Result, TxId};
use std::ops::Bound;
use std::sync::Arc;
use crate::lsm::LsmStorage;
use crate::tenant_codec::{bound_as_ref, upper_bound_for_prefix, TenantScopedStorage};

/// Adapter trait for types that can act as backend for `TenantScopedStorage`.
pub trait StorageEngineHandle: Send + Sync + 'static {
    fn get_handle<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, Result<Option<Bytes>>>;
    fn get_at_seq_handle<'a>(&'a self, key: &'a [u8], seq_no: u64) -> BoxFuture<'a, Result<Option<Bytes>>>;
    fn get_tracked_handle<'a>(&'a self, tx_id: TxId, key: &'a [u8]) -> BoxFuture<'a, Result<Option<Bytes>>>;
    fn get_at_seq_tracked_handle<'a>(&'a self, tx_id: TxId, key: &'a [u8], seq: u64) -> BoxFuture<'a, Result<Option<Bytes>>>;
    fn scan_prefix_tracked_handle<'a>(&'a self, tx_id: TxId, prefix: &'a [u8]) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>>;
    fn supports_ssi_tracking_handle(&self) -> bool;
    fn put_handle<'a>(&'a self, tx_id: TxId, key: &'a [u8], value: &'a [u8]) -> BoxFuture<'a, Result<()>>;
    fn put_if_absent_handle<'a>(&'a self, tx_id: TxId, key: &'a [u8], value: &'a [u8]) -> BoxFuture<'a, Result<bool>>;
    fn put_batch_handle<'a>(&'a self, tx_id: TxId, entries: &'a [(Vec<u8>, Vec<u8>)]) -> BoxFuture<'a, Result<()>>;
    fn delete_handle<'a>(&'a self, tx_id: TxId, key: &'a [u8]) -> BoxFuture<'a, Result<()>>;
    fn delete_many_handle<'a>(&'a self, tx_id: TxId, keys: Vec<Vec<u8>>) -> BoxFuture<'a, Result<u64>>;
    fn delete_prefix_handle<'a>(&'a self, tx_id: TxId, prefix: &'a [u8]) -> BoxFuture<'a, Result<u64>>;
    fn commit_handle<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>>;
    fn rollback_handle<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>>;
    fn rollback_to_tx_handle<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>>;
    fn flush_handle<'a>(&'a self) -> BoxFuture<'a, Result<()>>;
    fn stats_handle<'a>(&'a self) -> BoxFuture<'a, Result<StorageStats>>;
    fn last_seq_no_handle<'a>(&'a self) -> BoxFuture<'a, Result<u64>>;
    fn last_tx_id_handle<'a>(&'a self) -> BoxFuture<'a, Result<TxId>>;
    fn pin_checkpoint_handle<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, Result<()>>;
    fn unpin_checkpoint_handle<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, Result<()>>;
    fn scan_prefix_handle<'a>(&'a self, prefix: &'a [u8]) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>>;
    fn scan_prefix_bounded_handle<'a>(&'a self, prefix: &'a [u8], limit: usize, cursor: Option<&'a [u8]>) -> BoxFuture<'a, Result<(Vec<(Vec<u8>, Vec<u8>)>, Option<Vec<u8>>)>>;
    fn scan_prefix_at_handle<'a>(&'a self, prefix: &'a [u8], seq_no: u64) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>>;
    fn scan_handle<'a>(&'a self, start: Bound<&'a [u8]>, end: Bound<&'a [u8]>, limit: Option<usize>) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>>;
    fn scan_bounded_handle<'a>(&'a self, start: Bound<&'a [u8]>, end: Bound<&'a [u8]>, limit: usize, cursor: Option<&'a [u8]>) -> BoxFuture<'a, Result<(Vec<(Vec<u8>, Vec<u8>)>, Option<Vec<u8>>)>>;
}

impl StorageEngineHandle for LsmStorage {
    fn get_handle<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, Result<Option<Bytes>>> { StorageEngine::get(self, key) }
    fn get_at_seq_handle<'a>(&'a self, key: &'a [u8], seq_no: u64) -> BoxFuture<'a, Result<Option<Bytes>>> { StorageEngine::get_at_seq(self, key, seq_no) }
    fn get_tracked_handle<'a>(&'a self, tx_id: TxId, key: &'a [u8]) -> BoxFuture<'a, Result<Option<Bytes>>> { StorageEngine::get_tracked(self, tx_id, key) }
    fn get_at_seq_tracked_handle<'a>(&'a self, tx_id: TxId, key: &'a [u8], seq: u64) -> BoxFuture<'a, Result<Option<Bytes>>> { StorageEngine::get_at_seq_tracked(self, tx_id, key, seq) }
    fn scan_prefix_tracked_handle<'a>(&'a self, tx_id: TxId, prefix: &'a [u8]) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> { StorageEngine::scan_prefix_tracked(self, tx_id, prefix) }
    fn supports_ssi_tracking_handle(&self) -> bool { StorageEngine::supports_ssi_tracking(self) }
    fn put_handle<'a>(&'a self, tx_id: TxId, key: &'a [u8], value: &'a [u8]) -> BoxFuture<'a, Result<()>> { StorageEngine::put(self, tx_id, key, value) }
    fn put_if_absent_handle<'a>(&'a self, tx_id: TxId, key: &'a [u8], value: &'a [u8]) -> BoxFuture<'a, Result<bool>> { StorageEngine::put_if_absent(self, tx_id, key, value) }
    fn put_batch_handle<'a>(&'a self, tx_id: TxId, entries: &'a [(Vec<u8>, Vec<u8>)]) -> BoxFuture<'a, Result<()>> { StorageEngine::put_batch(self, tx_id, entries) }
    fn delete_handle<'a>(&'a self, tx_id: TxId, key: &'a [u8]) -> BoxFuture<'a, Result<()>> { StorageEngine::delete(self, tx_id, key) }
    fn delete_many_handle<'a>(&'a self, tx_id: TxId, keys: Vec<Vec<u8>>) -> BoxFuture<'a, Result<u64>> { StorageEngine::delete_many(self, tx_id, keys) }
    fn delete_prefix_handle<'a>(&'a self, tx_id: TxId, prefix: &'a [u8]) -> BoxFuture<'a, Result<u64>> { StorageEngine::delete_prefix(self, tx_id, prefix) }
    fn commit_handle<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>> { StorageEngine::commit(self, tx_id) }
    fn rollback_handle<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>> { StorageEngine::rollback(self, tx_id) }
    fn rollback_to_tx_handle<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>> { StorageEngine::rollback_to_tx(self, tx_id) }
    fn flush_handle<'a>(&'a self) -> BoxFuture<'a, Result<()>> { StorageEngine::flush(self) }
    fn stats_handle<'a>(&'a self) -> BoxFuture<'a, Result<StorageStats>> { StorageEngine::stats(self) }
    fn last_seq_no_handle<'a>(&'a self) -> BoxFuture<'a, Result<u64>> { StorageEngine::last_seq_no(self) }
    fn last_tx_id_handle<'a>(&'a self) -> BoxFuture<'a, Result<TxId>> { StorageEngine::last_tx_id(self) }
    fn pin_checkpoint_handle<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, Result<()>> { StorageEngine::pin_checkpoint(self, seq_no) }
    fn unpin_checkpoint_handle<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, Result<()>> { StorageEngine::unpin_checkpoint(self, seq_no) }
    fn scan_prefix_handle<'a>(&'a self, prefix: &'a [u8]) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> { StorageEngine::scan_prefix(self, prefix) }
    fn scan_prefix_bounded_handle<'a>(&'a self, prefix: &'a [u8], limit: usize, cursor: Option<&'a [u8]>) -> BoxFuture<'a, Result<(Vec<(Vec<u8>, Vec<u8>)>, Option<Vec<u8>>)>> { StorageEngine::scan_prefix_bounded(self, prefix, limit, cursor) }
    fn scan_prefix_at_handle<'a>(&'a self, prefix: &'a [u8], seq_no: u64) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> { StorageEngine::scan_prefix_at(self, prefix, seq_no) }
    fn scan_handle<'a>(&'a self, start: Bound<&'a [u8]>, end: Bound<&'a [u8]>, limit: Option<usize>) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> { StorageEngine::scan(self, start, end, limit) }
    fn scan_bounded_handle<'a>(&'a self, start: Bound<&'a [u8]>, end: Bound<&'a [u8]>, limit: usize, cursor: Option<&'a [u8]>) -> BoxFuture<'a, Result<(Vec<(Vec<u8>, Vec<u8>)>, Option<Vec<u8>>)>> { StorageEngine::scan_bounded(self, start, end, limit, cursor) }
}

impl<S: StorageEngine> StorageEngineHandle for Arc<S> {
    fn get_handle<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, Result<Option<Bytes>>> { self.as_ref().get(key) }
    fn get_at_seq_handle<'a>(&'a self, key: &'a [u8], seq_no: u64) -> BoxFuture<'a, Result<Option<Bytes>>> { self.as_ref().get_at_seq(key, seq_no) }
    fn get_tracked_handle<'a>(&'a self, tx_id: TxId, key: &'a [u8]) -> BoxFuture<'a, Result<Option<Bytes>>> { self.as_ref().get_tracked(tx_id, key) }
    fn get_at_seq_tracked_handle<'a>(&'a self, tx_id: TxId, key: &'a [u8], seq: u64) -> BoxFuture<'a, Result<Option<Bytes>>> { self.as_ref().get_at_seq_tracked(tx_id, key, seq) }
    fn scan_prefix_tracked_handle<'a>(&'a self, tx_id: TxId, prefix: &'a [u8]) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> { self.as_ref().scan_prefix_tracked(tx_id, prefix) }
    fn supports_ssi_tracking_handle(&self) -> bool { self.as_ref().supports_ssi_tracking() }
    fn put_handle<'a>(&'a self, tx_id: TxId, key: &'a [u8], value: &'a [u8]) -> BoxFuture<'a, Result<()>> { self.as_ref().put(tx_id, key, value) }
    fn put_if_absent_handle<'a>(&'a self, tx_id: TxId, key: &'a [u8], value: &'a [u8]) -> BoxFuture<'a, Result<bool>> { self.as_ref().put_if_absent(tx_id, key, value) }
    fn put_batch_handle<'a>(&'a self, tx_id: TxId, entries: &'a [(Vec<u8>, Vec<u8>)]) -> BoxFuture<'a, Result<()>> { self.as_ref().put_batch(tx_id, entries) }
    fn delete_handle<'a>(&'a self, tx_id: TxId, key: &'a [u8]) -> BoxFuture<'a, Result<()>> { self.as_ref().delete(tx_id, key) }
    fn delete_many_handle<'a>(&'a self, tx_id: TxId, keys: Vec<Vec<u8>>) -> BoxFuture<'a, Result<u64>> { self.as_ref().delete_many(tx_id, keys) }
    fn delete_prefix_handle<'a>(&'a self, tx_id: TxId, prefix: &'a [u8]) -> BoxFuture<'a, Result<u64>> { self.as_ref().delete_prefix(tx_id, prefix) }
    fn commit_handle<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>> { self.as_ref().commit(tx_id) }
    fn rollback_handle<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>> { self.as_ref().rollback(tx_id) }
    fn rollback_to_tx_handle<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>> { self.as_ref().rollback_to_tx(tx_id) }
    fn flush_handle<'a>(&'a self) -> BoxFuture<'a, Result<()>> { self.as_ref().flush() }
    fn stats_handle<'a>(&'a self) -> BoxFuture<'a, Result<StorageStats>> { self.as_ref().stats() }
    fn last_seq_no_handle<'a>(&'a self) -> BoxFuture<'a, Result<u64>> { self.as_ref().last_seq_no() }
    fn last_tx_id_handle<'a>(&'a self) -> BoxFuture<'a, Result<TxId>> { self.as_ref().last_tx_id() }
    fn pin_checkpoint_handle<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, Result<()>> { self.as_ref().pin_checkpoint(seq_no) }
    fn unpin_checkpoint_handle<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, Result<()>> { self.as_ref().unpin_checkpoint(seq_no) }
    fn scan_prefix_handle<'a>(&'a self, prefix: &'a [u8]) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> { self.as_ref().scan_prefix(prefix) }
    fn scan_prefix_bounded_handle<'a>(&'a self, prefix: &'a [u8], limit: usize, cursor: Option<&'a [u8]>) -> BoxFuture<'a, Result<(Vec<(Vec<u8>, Vec<u8>)>, Option<Vec<u8>>)>> { self.as_ref().scan_prefix_bounded(prefix, limit, cursor) }
    fn scan_prefix_at_handle<'a>(&'a self, prefix: &'a [u8], seq_no: u64) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> { self.as_ref().scan_prefix_at(prefix, seq_no) }
    fn scan_handle<'a>(&'a self, start: Bound<&'a [u8]>, end: Bound<&'a [u8]>, limit: Option<usize>) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> { self.as_ref().scan(start, end, limit) }
    fn scan_bounded_handle<'a>(&'a self, start: Bound<&'a [u8]>, end: Bound<&'a [u8]>, limit: usize, cursor: Option<&'a [u8]>) -> BoxFuture<'a, Result<(Vec<(Vec<u8>, Vec<u8>)>, Option<Vec<u8>>)>> { self.as_ref().scan_bounded(start, end, limit, cursor) }
}

impl<S: StorageEngineHandle> StorageEngine for TenantScopedStorage<S> {
    fn get<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, Result<Option<Bytes>>> {
        Box::pin(async move {
            let physical_key = self.make_key(key);
            self.inner.get_handle(&physical_key).await
        })
    }

    fn get_at_seq<'a>(&'a self, key: &'a [u8], seq_no: u64) -> BoxFuture<'a, Result<Option<Bytes>>> {
        Box::pin(async move {
            let physical_key = self.make_key(key);
            self.inner.get_at_seq_handle(&physical_key, seq_no).await
        })
    }

    fn get_tracked<'a>(&'a self, tx_id: TxId, key: &'a [u8]) -> BoxFuture<'a, Result<Option<Bytes>>> {
        Box::pin(async move {
            let physical_key = self.make_key(key);
            self.inner.get_tracked_handle(tx_id, &physical_key).await
        })
    }

    fn get_at_seq_tracked<'a>(&'a self, tx_id: TxId, key: &'a [u8], seq: u64) -> BoxFuture<'a, Result<Option<Bytes>>> {
        Box::pin(async move {
            let physical_key = self.make_key(key);
            self.inner.get_at_seq_tracked_handle(tx_id, &physical_key, seq).await
        })
    }

    fn scan_prefix_tracked<'a>(&'a self, tx_id: TxId, prefix: &'a [u8]) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        Box::pin(async move {
            let physical_prefix = self.make_key(prefix);
            let raw_res = self.inner.scan_prefix_tracked_handle(tx_id, &physical_prefix).await?;
            Ok(self.filter_strip_batch(raw_res))
        })
    }

    fn supports_ssi_tracking(&self) -> bool {
        self.inner.supports_ssi_tracking_handle()
    }

    fn put<'a>(&'a self, tx_id: TxId, key: &'a [u8], value: &'a [u8]) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            let physical_key = self.make_key(key);
            self.inner.put_handle(tx_id, &physical_key, value).await
        })
    }

    fn put_if_absent<'a>(&'a self, tx_id: TxId, key: &'a [u8], value: &'a [u8]) -> BoxFuture<'a, Result<bool>> {
        Box::pin(async move {
            let physical_key = self.make_key(key);
            self.inner.put_if_absent_handle(tx_id, &physical_key, value).await
        })
    }

    fn put_batch<'a>(&'a self, tx_id: TxId, entries: &'a [(Vec<u8>, Vec<u8>)]) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            let physical_entries: Vec<(Vec<u8>, Vec<u8>)> = entries.iter().map(|(k, v)| (self.make_key(k), v.clone())).collect();
            self.inner.put_batch_handle(tx_id, &physical_entries).await
        })
    }

    fn delete<'a>(&'a self, tx_id: TxId, key: &'a [u8]) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            let physical_key = self.make_key(key);
            self.inner.delete_handle(tx_id, &physical_key).await
        })
    }

    fn delete_many<'a>(&'a self, tx_id: TxId, keys: Vec<Vec<u8>>) -> BoxFuture<'a, Result<u64>> {
        Box::pin(async move {
            let physical_keys: Vec<Vec<u8>> = keys.iter().map(|k| self.make_key(k)).collect();
            self.inner.delete_many_handle(tx_id, physical_keys).await
        })
    }

    fn delete_prefix<'a>(&'a self, tx_id: TxId, prefix: &'a [u8]) -> BoxFuture<'a, Result<u64>> {
        Box::pin(async move {
            let physical_prefix = self.make_key(prefix);
            self.inner.delete_prefix_handle(tx_id, &physical_prefix).await
        })
    }

    fn commit<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>> { self.inner.commit_handle(tx_id) }
    fn rollback<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>> { self.inner.rollback_handle(tx_id) }
    fn rollback_to_tx<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>> { self.inner.rollback_to_tx_handle(tx_id) }
    fn flush<'a>(&'a self) -> BoxFuture<'a, Result<()>> { self.inner.flush_handle() }
    fn stats<'a>(&'a self) -> BoxFuture<'a, Result<StorageStats>> { self.inner.stats_handle() }
    fn last_seq_no<'a>(&'a self) -> BoxFuture<'a, Result<u64>> { self.inner.last_seq_no_handle() }
    fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, Result<TxId>> { self.inner.last_tx_id_handle() }
    fn pin_checkpoint<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, Result<()>> { self.inner.pin_checkpoint_handle(seq_no) }
    fn unpin_checkpoint<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, Result<()>> { self.inner.unpin_checkpoint_handle(seq_no) }

    fn scan_prefix<'a>(&'a self, prefix: &'a [u8]) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        Box::pin(async move {
            let physical_prefix = self.make_key(prefix);
            let raw_res = self.inner.scan_prefix_handle(&physical_prefix).await?;
            Ok(self.filter_strip_batch(raw_res))
        })
    }

    fn scan_prefix_bounded<'a>(&'a self, prefix: &'a [u8], limit: usize, cursor: Option<&'a [u8]>) -> BoxFuture<'a, Result<(Vec<(Vec<u8>, Vec<u8>)>, Option<Vec<u8>>)>> {
        Box::pin(async move {
            let physical_prefix = self.make_key(prefix);
            let physical_cursor = cursor.map(|c| self.make_key(c));
            let (raw_batch, next_cursor) = self.inner.scan_prefix_bounded_handle(&physical_prefix, limit, physical_cursor.as_deref()).await?;
            let stripped_batch = self.filter_strip_batch(raw_batch);
            let stripped_cursor = next_cursor.and_then(|c| self.strip_key(c));
            Ok((stripped_batch, stripped_cursor))
        })
    }

    fn scan_prefix_at<'a>(&'a self, prefix: &'a [u8], seq_no: u64) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        Box::pin(async move {
            let physical_prefix = self.make_key(prefix);
            let raw_res = self.inner.scan_prefix_at_handle(&physical_prefix, seq_no).await?;
            Ok(self.filter_strip_batch(raw_res))
        })
    }

    fn scan<'a>(&'a self, start: Bound<&'a [u8]>, end: Bound<&'a [u8]>, limit: Option<usize>) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        Box::pin(async move {
            let prefix = self.codec.scan_prefix();
            let upper_bound_bytes = upper_bound_for_prefix(prefix);
            let physical_start = match start {
                Bound::Included(k) => Bound::Included(self.make_key(k)),
                Bound::Excluded(k) => Bound::Excluded(self.make_key(k)),
                Bound::Unbounded => Bound::Included(prefix.to_vec()),
            };
            let physical_end = match end {
                Bound::Included(k) => Bound::Included(self.make_key(k)),
                Bound::Excluded(k) => Bound::Excluded(self.make_key(k)),
                Bound::Unbounded => match &upper_bound_bytes {
                    Some(ub) => Bound::Excluded(ub.clone()),
                    None => Bound::Unbounded,
                },
            };
            let raw_res = self.inner.scan_handle(bound_as_ref(&physical_start), bound_as_ref(&physical_end), limit).await?;
            Ok(self.filter_strip_batch(raw_res))
        })
    }

    fn scan_bounded<'a>(&'a self, start: Bound<&'a [u8]>, end: Bound<&'a [u8]>, limit: usize, cursor: Option<&'a [u8]>) -> BoxFuture<'a, Result<(Vec<(Vec<u8>, Vec<u8>)>, Option<Vec<u8>>)>> {
        Box::pin(async move {
            let prefix = self.codec.scan_prefix();
            let upper_bound_bytes = upper_bound_for_prefix(prefix);
            let physical_start = match start {
                Bound::Included(k) => Bound::Included(self.make_key(k)),
                Bound::Excluded(k) => Bound::Excluded(self.make_key(k)),
                Bound::Unbounded => Bound::Included(prefix.to_vec()),
            };
            let physical_end = match end {
                Bound::Included(k) => Bound::Included(self.make_key(k)),
                Bound::Excluded(k) => Bound::Excluded(self.make_key(k)),
                Bound::Unbounded => match &upper_bound_bytes {
                    Some(ub) => Bound::Excluded(ub.clone()),
                    None => Bound::Unbounded,
                },
            };
            let physical_cursor = cursor.map(|c| self.make_key(c));
            let (raw_batch, next_cursor) = self.inner.scan_bounded_handle(bound_as_ref(&physical_start), bound_as_ref(&physical_end), limit, physical_cursor.as_deref()).await?;
            let stripped_batch = self.filter_strip_batch(raw_batch);
            let stripped_cursor = next_cursor.and_then(|c| self.strip_key(c));
            Ok((stripped_batch, stripped_cursor))
        })
    }
}
