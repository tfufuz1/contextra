use super::*;
use std::ops::Bound;
use contextra_ports::StorageStats;

impl<S: StorageEngineHandle> StorageEngine for TenantScopedStorage<S> {
    fn get<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, Result<Option<Bytes>>> {
        Box::pin(async move {
            let physical_key = self.make_key(key);
            self.inner.get_handle(&physical_key).await
        })
    }

    fn get_at_seq<'a>(
        &'a self,
        key: &'a [u8],
        seq_no: u64,
    ) -> BoxFuture<'a, Result<Option<Bytes>>> {
        Box::pin(async move {
            let physical_key = self.make_key(key);
            self.inner.get_at_seq_handle(&physical_key, seq_no).await
        })
    }

    fn get_tracked<'a>(
        &'a self,
        tx_id: TxId,
        key: &'a [u8],
    ) -> BoxFuture<'a, Result<Option<Bytes>>> {
        Box::pin(async move {
            let physical_key = self.make_key(key);
            self.inner.get_tracked_handle(tx_id, &physical_key).await
        })
    }

    fn get_at_seq_tracked<'a>(
        &'a self,
        tx_id: TxId,
        key: &'a [u8],
        seq: u64,
    ) -> BoxFuture<'a, Result<Option<Bytes>>> {
        Box::pin(async move {
            let physical_key = self.make_key(key);
            self.inner
                .get_at_seq_tracked_handle(tx_id, &physical_key, seq)
                .await
        })
    }

    fn scan_prefix_tracked<'a>(
        &'a self,
        tx_id: TxId,
        prefix: &'a [u8],
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        Box::pin(async move {
            let physical_prefix = self.make_key(prefix);
            let raw_res = self
                .inner
                .scan_prefix_tracked_handle(tx_id, &physical_prefix)
                .await?;
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

    fn put_if_absent<'a>(
        &'a self,
        tx_id: TxId,
        key: &'a [u8],
        value: &'a [u8],
    ) -> BoxFuture<'a, Result<bool>> {
        Box::pin(async move {
            let physical_key = self.make_key(key);
            self.inner
                .put_if_absent_handle(tx_id, &physical_key, value)
                .await
        })
    }

    fn put_batch<'a>(
        &'a self,
        tx_id: TxId,
        entries: &'a [(Vec<u8>, Vec<u8>)],
    ) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            let physical_entries: Vec<(Vec<u8>, Vec<u8>)> = entries
                .iter()
                .map(|(k, v)| (self.make_key(k), v.clone()))
                .collect();
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
            self.inner
                .delete_prefix_handle(tx_id, &physical_prefix)
                .await
        })
    }

    fn commit<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        self.inner.commit_handle(tx_id)
    }

    fn rollback<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        self.inner.rollback_handle(tx_id)
    }

    fn rollback_to_tx<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        self.inner.rollback_to_tx_handle(tx_id)
    }

    fn flush<'a>(&'a self) -> BoxFuture<'a, Result<()>> {
        self.inner.flush_handle()
    }

    fn stats<'a>(&'a self) -> BoxFuture<'a, Result<StorageStats>> {
        self.inner.stats_handle()
    }

    fn last_seq_no<'a>(&'a self) -> BoxFuture<'a, Result<u64>> {
        self.inner.last_seq_no_handle()
    }

    fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, Result<TxId>> {
        self.inner.last_tx_id_handle()
    }

    fn pin_checkpoint<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, Result<()>> {
        self.inner.pin_checkpoint_handle(seq_no)
    }

    fn unpin_checkpoint<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, Result<()>> {
        self.inner.unpin_checkpoint_handle(seq_no)
    }

    fn scan_prefix<'a>(
        &'a self,
        prefix: &'a [u8],
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        Box::pin(async move {
            let physical_prefix = self.make_key(prefix);
            let raw_res = self.inner.scan_prefix_handle(&physical_prefix).await?;
            Ok(self.filter_strip_batch(raw_res))
        })
    }

    fn scan_prefix_bounded<'a>(
        &'a self,
        prefix: &'a [u8],
        limit: usize,
        cursor: Option<&'a [u8]>,
    ) -> BoxFuture<'a, Result<(Vec<(Vec<u8>, Vec<u8>)>, Option<Vec<u8>>)>> {
        Box::pin(async move {
            let physical_prefix = self.make_key(prefix);
            let physical_cursor = cursor.map(|c| self.make_key(c));
            let (raw_batch, next_cursor) = self
                .inner
                .scan_prefix_bounded_handle(&physical_prefix, limit, physical_cursor.as_deref())
                .await?;
            let stripped_batch = self.filter_strip_batch(raw_batch);
            let stripped_cursor = next_cursor.and_then(|c| self.strip_key(c));
            Ok((stripped_batch, stripped_cursor))
        })
    }

    fn scan_prefix_at<'a>(
        &'a self,
        prefix: &'a [u8],
        seq_no: u64,
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        Box::pin(async move {
            let physical_prefix = self.make_key(prefix);
            let raw_res = self
                .inner
                .scan_prefix_at_handle(&physical_prefix, seq_no)
                .await?;
            Ok(self.filter_strip_batch(raw_res))
        })
    }

    fn scan<'a>(
        &'a self,
        start: Bound<&'a [u8]>,
        end: Bound<&'a [u8]>,
        limit: Option<usize>,
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
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

            let raw_res = self
                .inner
                .scan_handle(
                    bound_as_ref(&physical_start),
                    bound_as_ref(&physical_end),
                    limit,
                )
                .await?;

            Ok(self.filter_strip_batch(raw_res))
        })
    }

    fn scan_bounded<'a>(
        &'a self,
        start: Bound<&'a [u8]>,
        end: Bound<&'a [u8]>,
        limit: usize,
        cursor: Option<&'a [u8]>,
    ) -> BoxFuture<'a, Result<(Vec<(Vec<u8>, Vec<u8>)>, Option<Vec<u8>>)>> {
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

            let (raw_batch, next_cursor) = self
                .inner
                .scan_bounded_handle(
                    bound_as_ref(&physical_start),
                    bound_as_ref(&physical_end),
                    limit,
                    physical_cursor.as_deref(),
                )
                .await?;

            let stripped_batch = self.filter_strip_batch(raw_batch);
            let stripped_cursor = next_cursor.and_then(|c| self.strip_key(c));
            Ok((stripped_batch, stripped_cursor))
        })
    }
}
