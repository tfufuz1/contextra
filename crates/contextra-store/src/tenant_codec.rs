// FILE-CONTEXT
// STAND: 2026-09-28T00:00:00Z
// ZWECK: Multi-Tenant Key Isolation & Encoding für LSM Storage Engine
// INVARIANTEN: INV-TENANT-2: scan_prefix(codec.scan_prefix()) liefert ausschließlich Keys des gewählten Tenants
// NICHT-OFFENSICHTLICH: Festes `t:` Präfix verhindert Kollisionen mit Legacy non-tenanted Keys
// SIEHE AUCH: lsm.rs, rules/tag_taxonomy.md

//! TenantKeyCodec — LSM-Key-Isolation via Präfix-Encoding.
//!
//! INVARIANTE INV-TENANT-2: scan_prefix(codec.scan_prefix()) liefert
//! AUSSCHLIESSLICH Keys dieses Tenants. Cross-Tenant-Leak strukturell ausgeschlossen.
//!
//! ENCODING: `t:{tenant_id}:{collection_id}:{doc_type}:{doc_id}`
//! Festes `t:`-Präfix verhindert Kollision mit Legacy-Keys.

use crate::lsm::LsmStorage;
use bytes::Bytes;
use contextra_core::{CollectionId, DocId, StorageStats, TenantId, TxId};
use contextra_ports::{BoxFuture, Result, StorageEngine};
use std::ops::Bound;
use std::sync::Arc;

/// Encodes LSM storage keys with tenant isolation prefixes.
pub struct TenantKeyCodec {
    tenant_id: TenantId,
    /// Vorberechnetes Scan-Präfix ohne Allokation im Hot-Path.
    scan_prefix_cache: Vec<u8>,
}

impl TenantKeyCodec {
    /// Creates a new `TenantKeyCodec` for the given `TenantId`.
    pub fn new(tenant_id: TenantId) -> Self {
        let scan_prefix_cache = format!("t:{}:", tenant_id.inner()).into_bytes();
        Self {
            tenant_id,
            scan_prefix_cache,
        }
    }

    /// Encodes a document chunk key: `t:{tenant}:{collection}:chunk:{doc}`.
    #[inline]
    pub fn encode_chunk_key(&self, collection: &CollectionId, doc_id: DocId) -> Vec<u8> {
        format!(
            "t:{}:{}:chunk:{}",
            self.tenant_id.inner(),
            collection.0,
            doc_id.0
        )
        .into_bytes()
    }

    /// Encodes a graph entity key: `t:{tenant}:{collection}:graph:{entity}`.
    #[inline]
    pub fn encode_graph_key(&self, collection: &CollectionId, entity_id: u64) -> Vec<u8> {
        format!(
            "t:{}:{}:graph:{}",
            self.tenant_id.inner(),
            collection.0,
            entity_id
        )
        .into_bytes()
    }

    /// Globales Scan-Präfix für diesen Tenant (alle Collections): `t:{tenant}:`
    /// Cached — keine Allokation im Hot-Path.
    #[inline]
    pub fn scan_prefix(&self) -> &[u8] {
        &self.scan_prefix_cache
    }

    /// Collection-spezifisches Scan-Präfix: `t:{tenant}:{col}:`
    pub fn collection_prefix(&self, collection: &CollectionId) -> Vec<u8> {
        format!("t:{}:{}:", self.tenant_id.inner(), collection.0).into_bytes()
    }

    /// Dekodiert TenantId aus encoded Key. None bei ungültigem Format.
    /// Kein Unwrap — robuste Nutzung im Recovery-Pfad.
    pub fn decode_tenant_id(key: &[u8]) -> Option<TenantId> {
        let s = std::str::from_utf8(key).ok()?;
        let rest = s.strip_prefix("t:")?;
        let end = rest.find(':')?;
        let id: u64 = rest[..end].parse().ok()?;
        if id == 0 {
            None
        } else {
            Some(TenantId(id))
        }
    }
}

/// Internal trait abstraction enabling `TenantScopedStorage` to wrap both `S: StorageEngine` and `Arc<S>`.
pub trait StorageEngineHandle: Send + Sync + 'static {
    fn get_handle<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, Result<Option<Bytes>>>;
    fn get_at_seq_handle<'a>(
        &'a self,
        key: &'a [u8],
        seq_no: u64,
    ) -> BoxFuture<'a, Result<Option<Bytes>>>;
    fn get_tracked_handle<'a>(
        &'a self,
        tx_id: TxId,
        key: &'a [u8],
    ) -> BoxFuture<'a, Result<Option<Bytes>>>;
    fn get_at_seq_tracked_handle<'a>(
        &'a self,
        tx_id: TxId,
        key: &'a [u8],
        seq: u64,
    ) -> BoxFuture<'a, Result<Option<Bytes>>>;
    #[allow(clippy::type_complexity)]
    fn scan_prefix_tracked_handle<'a>(
        &'a self,
        tx_id: TxId,
        prefix: &'a [u8],
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>>;
    fn supports_ssi_tracking_handle(&self) -> bool;
    fn put_handle<'a>(
        &'a self,
        tx_id: TxId,
        key: &'a [u8],
        value: &'a [u8],
    ) -> BoxFuture<'a, Result<()>>;
    fn put_if_absent_handle<'a>(
        &'a self,
        tx_id: TxId,
        key: &'a [u8],
        value: &'a [u8],
    ) -> BoxFuture<'a, Result<bool>>;
    fn put_batch_handle<'a>(
        &'a self,
        tx_id: TxId,
        entries: &'a [(Vec<u8>, Vec<u8>)],
    ) -> BoxFuture<'a, Result<()>>;
    fn delete_handle<'a>(&'a self, tx_id: TxId, key: &'a [u8]) -> BoxFuture<'a, Result<()>>;
    fn delete_many_handle<'a>(
        &'a self,
        tx_id: TxId,
        keys: Vec<Vec<u8>>,
    ) -> BoxFuture<'a, Result<u64>>;
    fn delete_prefix_handle<'a>(
        &'a self,
        tx_id: TxId,
        prefix: &'a [u8],
    ) -> BoxFuture<'a, Result<u64>>;
    fn commit_handle<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>>;
    fn rollback_handle<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>>;
    fn rollback_to_tx_handle<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>>;
    fn flush_handle<'a>(&'a self) -> BoxFuture<'a, Result<()>>;
    fn stats_handle<'a>(&'a self) -> BoxFuture<'a, Result<StorageStats>>;
    fn last_seq_no_handle<'a>(&'a self) -> BoxFuture<'a, Result<u64>>;
    fn last_tx_id_handle<'a>(&'a self) -> BoxFuture<'a, Result<TxId>>;
    fn pin_checkpoint_handle<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, Result<()>>;
    fn unpin_checkpoint_handle<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, Result<()>>;
    #[allow(clippy::type_complexity)]
    fn scan_prefix_handle<'a>(
        &'a self,
        prefix: &'a [u8],
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>>;
    #[allow(clippy::type_complexity)]
    fn scan_prefix_bounded_handle<'a>(
        &'a self,
        prefix: &'a [u8],
        limit: usize,
        cursor: Option<&'a [u8]>,
    ) -> BoxFuture<'a, Result<(Vec<(Vec<u8>, Vec<u8>)>, Option<Vec<u8>>)>>;
    #[allow(clippy::type_complexity)]
    fn scan_prefix_at_handle<'a>(
        &'a self,
        prefix: &'a [u8],
        seq_no: u64,
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>>;
    #[allow(clippy::type_complexity)]
    fn scan_handle<'a>(
        &'a self,
        start: Bound<&'a [u8]>,
        end: Bound<&'a [u8]>,
        limit: Option<usize>,
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>>;
    #[allow(clippy::type_complexity)]
    fn scan_bounded_handle<'a>(
        &'a self,
        start: Bound<&'a [u8]>,
        end: Bound<&'a [u8]>,
        limit: usize,
        cursor: Option<&'a [u8]>,
    ) -> BoxFuture<'a, Result<(Vec<(Vec<u8>, Vec<u8>)>, Option<Vec<u8>>)>>;
}

impl StorageEngineHandle for LsmStorage {
    fn get_handle<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, Result<Option<Bytes>>> {
        self.get(key)
    }
    fn get_at_seq_handle<'a>(
        &'a self,
        key: &'a [u8],
        seq_no: u64,
    ) -> BoxFuture<'a, Result<Option<Bytes>>> {
        self.get_at_seq(key, seq_no)
    }
    fn get_tracked_handle<'a>(
        &'a self,
        tx_id: TxId,
        key: &'a [u8],
    ) -> BoxFuture<'a, Result<Option<Bytes>>> {
        Box::pin(self.get_tracked(tx_id, key))
    }
    fn get_at_seq_tracked_handle<'a>(
        &'a self,
        tx_id: TxId,
        key: &'a [u8],
        seq: u64,
    ) -> BoxFuture<'a, Result<Option<Bytes>>> {
        Box::pin(self.get_at_seq_tracked(tx_id, key, seq))
    }
    fn scan_prefix_tracked_handle<'a>(
        &'a self,
        tx_id: TxId,
        prefix: &'a [u8],
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        Box::pin(self.scan_prefix_tracked(tx_id, prefix))
    }
    fn supports_ssi_tracking_handle(&self) -> bool {
        self.supports_ssi_tracking()
    }
    fn put_handle<'a>(
        &'a self,
        tx_id: TxId,
        key: &'a [u8],
        value: &'a [u8],
    ) -> BoxFuture<'a, Result<()>> {
        self.put(tx_id, key, value)
    }
    fn put_if_absent_handle<'a>(
        &'a self,
        tx_id: TxId,
        key: &'a [u8],
        value: &'a [u8],
    ) -> BoxFuture<'a, Result<bool>> {
        self.put_if_absent(tx_id, key, value)
    }
    fn put_batch_handle<'a>(
        &'a self,
        tx_id: TxId,
        entries: &'a [(Vec<u8>, Vec<u8>)],
    ) -> BoxFuture<'a, Result<()>> {
        self.put_batch(tx_id, entries)
    }
    fn delete_handle<'a>(&'a self, tx_id: TxId, key: &'a [u8]) -> BoxFuture<'a, Result<()>> {
        self.delete(tx_id, key)
    }
    fn delete_many_handle<'a>(
        &'a self,
        tx_id: TxId,
        keys: Vec<Vec<u8>>,
    ) -> BoxFuture<'a, Result<u64>> {
        self.delete_many(tx_id, keys)
    }
    fn delete_prefix_handle<'a>(
        &'a self,
        tx_id: TxId,
        prefix: &'a [u8],
    ) -> BoxFuture<'a, Result<u64>> {
        self.delete_prefix(tx_id, prefix)
    }
    fn commit_handle<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        self.commit(tx_id)
    }
    fn rollback_handle<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        self.rollback(tx_id)
    }
    fn rollback_to_tx_handle<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        Box::pin(self.rollback_to_tx(tx_id))
    }
    fn flush_handle<'a>(&'a self) -> BoxFuture<'a, Result<()>> {
        self.flush()
    }
    fn stats_handle<'a>(&'a self) -> BoxFuture<'a, Result<StorageStats>> {
        self.stats()
    }
    fn last_seq_no_handle<'a>(&'a self) -> BoxFuture<'a, Result<u64>> {
        self.last_seq_no()
    }
    fn last_tx_id_handle<'a>(&'a self) -> BoxFuture<'a, Result<TxId>> {
        self.last_tx_id()
    }
    fn pin_checkpoint_handle<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, Result<()>> {
        self.pin_checkpoint(seq_no)
    }
    fn unpin_checkpoint_handle<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, Result<()>> {
        self.unpin_checkpoint(seq_no)
    }
    fn scan_prefix_handle<'a>(
        &'a self,
        prefix: &'a [u8],
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        self.scan_prefix(prefix)
    }
    fn scan_prefix_bounded_handle<'a>(
        &'a self,
        prefix: &'a [u8],
        limit: usize,
        cursor: Option<&'a [u8]>,
    ) -> BoxFuture<'a, Result<(Vec<(Vec<u8>, Vec<u8>)>, Option<Vec<u8>>)>> {
        self.scan_prefix_bounded(prefix, limit, cursor)
    }
    fn scan_prefix_at_handle<'a>(
        &'a self,
        prefix: &'a [u8],
        seq_no: u64,
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        self.scan_prefix_at(prefix, seq_no)
    }
    fn scan_handle<'a>(
        &'a self,
        start: Bound<&'a [u8]>,
        end: Bound<&'a [u8]>,
        limit: Option<usize>,
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        self.scan(start, end, limit)
    }
    fn scan_bounded_handle<'a>(
        &'a self,
        start: Bound<&'a [u8]>,
        end: Bound<&'a [u8]>,
        limit: usize,
        cursor: Option<&'a [u8]>,
    ) -> BoxFuture<'a, Result<(Vec<(Vec<u8>, Vec<u8>)>, Option<Vec<u8>>)>> {
        self.scan_bounded(start, end, limit, cursor)
    }
}

impl<S: StorageEngine> StorageEngineHandle for Arc<S> {
    fn get_handle<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, Result<Option<Bytes>>> {
        (**self).get(key)
    }
    fn get_at_seq_handle<'a>(
        &'a self,
        key: &'a [u8],
        seq_no: u64,
    ) -> BoxFuture<'a, Result<Option<Bytes>>> {
        (**self).get_at_seq(key, seq_no)
    }
    fn get_tracked_handle<'a>(
        &'a self,
        tx_id: TxId,
        key: &'a [u8],
    ) -> BoxFuture<'a, Result<Option<Bytes>>> {
        (**self).get_tracked(tx_id, key)
    }
    fn get_at_seq_tracked_handle<'a>(
        &'a self,
        tx_id: TxId,
        key: &'a [u8],
        seq: u64,
    ) -> BoxFuture<'a, Result<Option<Bytes>>> {
        (**self).get_at_seq_tracked(tx_id, key, seq)
    }
    fn scan_prefix_tracked_handle<'a>(
        &'a self,
        tx_id: TxId,
        prefix: &'a [u8],
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        (**self).scan_prefix_tracked(tx_id, prefix)
    }
    fn supports_ssi_tracking_handle(&self) -> bool {
        (**self).supports_ssi_tracking()
    }
    fn put_handle<'a>(
        &'a self,
        tx_id: TxId,
        key: &'a [u8],
        value: &'a [u8],
    ) -> BoxFuture<'a, Result<()>> {
        (**self).put(tx_id, key, value)
    }
    fn put_if_absent_handle<'a>(
        &'a self,
        tx_id: TxId,
        key: &'a [u8],
        value: &'a [u8],
    ) -> BoxFuture<'a, Result<bool>> {
        (**self).put_if_absent(tx_id, key, value)
    }
    fn put_batch_handle<'a>(
        &'a self,
        tx_id: TxId,
        entries: &'a [(Vec<u8>, Vec<u8>)],
    ) -> BoxFuture<'a, Result<()>> {
        (**self).put_batch(tx_id, entries)
    }
    fn delete_handle<'a>(&'a self, tx_id: TxId, key: &'a [u8]) -> BoxFuture<'a, Result<()>> {
        (**self).delete(tx_id, key)
    }
    fn delete_many_handle<'a>(
        &'a self,
        tx_id: TxId,
        keys: Vec<Vec<u8>>,
    ) -> BoxFuture<'a, Result<u64>> {
        (**self).delete_many(tx_id, keys)
    }
    fn delete_prefix_handle<'a>(
        &'a self,
        tx_id: TxId,
        prefix: &'a [u8],
    ) -> BoxFuture<'a, Result<u64>> {
        (**self).delete_prefix(tx_id, prefix)
    }
    fn commit_handle<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        (**self).commit(tx_id)
    }
    fn rollback_handle<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        (**self).rollback(tx_id)
    }
    fn rollback_to_tx_handle<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        (**self).rollback_to_tx(tx_id)
    }
    fn flush_handle<'a>(&'a self) -> BoxFuture<'a, Result<()>> {
        (**self).flush()
    }
    fn stats_handle<'a>(&'a self) -> BoxFuture<'a, Result<StorageStats>> {
        (**self).stats()
    }
    fn last_seq_no_handle<'a>(&'a self) -> BoxFuture<'a, Result<u64>> {
        (**self).last_seq_no()
    }
    fn last_tx_id_handle<'a>(&'a self) -> BoxFuture<'a, Result<TxId>> {
        (**self).last_tx_id()
    }
    fn pin_checkpoint_handle<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, Result<()>> {
        (**self).pin_checkpoint(seq_no)
    }
    fn unpin_checkpoint_handle<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, Result<()>> {
        (**self).unpin_checkpoint(seq_no)
    }
    fn scan_prefix_handle<'a>(
        &'a self,
        prefix: &'a [u8],
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        (**self).scan_prefix(prefix)
    }
    fn scan_prefix_bounded_handle<'a>(
        &'a self,
        prefix: &'a [u8],
        limit: usize,
        cursor: Option<&'a [u8]>,
    ) -> BoxFuture<'a, Result<(Vec<(Vec<u8>, Vec<u8>)>, Option<Vec<u8>>)>> {
        (**self).scan_prefix_bounded(prefix, limit, cursor)
    }
    fn scan_prefix_at_handle<'a>(
        &'a self,
        prefix: &'a [u8],
        seq_no: u64,
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        (**self).scan_prefix_at(prefix, seq_no)
    }
    fn scan_handle<'a>(
        &'a self,
        start: Bound<&'a [u8]>,
        end: Bound<&'a [u8]>,
        limit: Option<usize>,
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        (**self).scan(start, end, limit)
    }
    fn scan_bounded_handle<'a>(
        &'a self,
        start: Bound<&'a [u8]>,
        end: Bound<&'a [u8]>,
        limit: usize,
        cursor: Option<&'a [u8]>,
    ) -> BoxFuture<'a, Result<(Vec<(Vec<u8>, Vec<u8>)>, Option<Vec<u8>>)>> {
        (**self).scan_bounded(start, end, limit, cursor)
    }
}

/// Tenant-aware `StorageEngine` wrapper implementing strict key-level isolation (INV-TENANT-2).
pub struct TenantScopedStorage<S> {
    inner: S,
    codec: TenantKeyCodec,
}

impl<S> TenantScopedStorage<S> {
    /// Creates a new `TenantScopedStorage` wrapping an underlying `StorageEngine` or shared pointer.
    pub fn new(inner: S, tenant_id: TenantId) -> Self {
        Self {
            inner,
            codec: TenantKeyCodec::new(tenant_id),
        }
    }

    /// Returns the immutable `TenantId` bound to this storage wrapper.
    pub fn tenant_id(&self) -> TenantId {
        self.codec.tenant_id
    }

    #[inline]
    fn make_key(&self, key: &[u8]) -> Vec<u8> {
        let prefix = self.codec.scan_prefix();
        let mut physical = Vec::with_capacity(prefix.len() + key.len());
        physical.extend_from_slice(prefix);
        physical.extend_from_slice(key);
        physical
    }

    #[inline]
    fn strip_key(&self, key: Vec<u8>) -> Option<Vec<u8>> {
        let prefix = self.codec.scan_prefix();
        if key.starts_with(prefix) {
            Some(key[prefix.len()..].to_vec())
        } else {
            None
        }
    }

    fn filter_strip_batch(&self, batch: Vec<(Vec<u8>, Vec<u8>)>) -> Vec<(Vec<u8>, Vec<u8>)> {
        let prefix = self.codec.scan_prefix();
        batch
            .into_iter()
            .filter_map(|(k, v)| {
                if k.starts_with(prefix) {
                    Some((k[prefix.len()..].to_vec(), v))
                } else {
                    None
                }
            })
            .collect()
    }
}

fn upper_bound_for_prefix(prefix: &[u8]) -> Option<Vec<u8>> {
    let mut ub = prefix.to_vec();
    while let Some(last) = ub.pop() {
        if last < 0xFF {
            ub.push(last + 1);
            return Some(ub);
        }
    }
    None
}

fn bound_as_ref(bound: &Bound<Vec<u8>>) -> Bound<&[u8]> {
    match bound {
        Bound::Included(v) => Bound::Included(v.as_slice()),
        Bound::Excluded(v) => Bound::Excluded(v.as_slice()),
        Bound::Unbounded => Bound::Unbounded,
    }
}

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

    fn get_tracked<'a>(&'a self, tx_id: TxId, key: &'a [u8]) -> BoxFuture<'a, Result<Option<Bytes>>> {
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
            self.inner.delete_prefix_handle(tx_id, &physical_prefix).await
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
                .scan_prefix_bounded_handle(
                    &physical_prefix,
                    limit,
                    physical_cursor.as_deref(),
                )
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_roundtrip() {
        let tenant = TenantId::try_new(7).unwrap();
        let codec = TenantKeyCodec::new(tenant);
        let col = CollectionId(42);
        let doc = DocId(99);
        let key = codec.encode_chunk_key(&col, doc);
        assert_eq!(TenantKeyCodec::decode_tenant_id(&key), Some(tenant));
    }

    #[test]
    fn test_cross_tenant_isolation() {
        let codec_a = TenantKeyCodec::new(TenantId::try_new(1).unwrap());
        let codec_b = TenantKeyCodec::new(TenantId::try_new(2).unwrap());
        let col = CollectionId(1);
        let doc = DocId(1);
        let key_a = codec_a.encode_chunk_key(&col, doc);
        // Key von Tenant A darf NICHT mit Präfix von Tenant B matchen
        assert!(!key_a.starts_with(codec_b.scan_prefix()));
    }

    #[test]
    fn test_scan_prefix_cached_no_alloc() {
        let codec = TenantKeyCodec::new(TenantId::try_new(42).unwrap());
        assert!(codec.scan_prefix().starts_with(b"t:42:"));
    }

    #[test]
    fn test_decode_invalid_key_returns_none() {
        assert!(TenantKeyCodec::decode_tenant_id(b"invalid").is_none());
        assert!(TenantKeyCodec::decode_tenant_id(b"t:0:col:chunk:1").is_none());
        // SYSTEM
    }

    #[test]
    fn test_upper_bound_calculation() {
        let prefix = b"t:101:";
        let ub = upper_bound_for_prefix(prefix).unwrap();
        assert_eq!(ub, b"t:101;");
    }
}
