//! # Mandantentrennung & Key-Codec (Ring 1)
//!
//! Definiert [`TenantKeyCodec`] und [`TenantScopedStorage`], um Daten strikt
//! mandantenfaehig zu praefigieren und logisch zu isolieren.

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
use contextra_ports::StorageStats;
use contextra_ports::{BoxFuture, Result, StorageEngine};
use contextra_types::{CollectionId, DocId, TenantId, TxId};
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
        let id_slice = rest.get(..end)?;
        let id: u64 = id_slice.parse().ok()?;
        TenantId::try_new(id).ok()
    }

    /// Dekodiert einen Chunk-Key vollständig in (TenantId, CollectionId, DocId).
    /// Invariante: decode(encode(x)) == x (Injektivität).
    pub fn decode_chunk_key(key: &[u8]) -> Option<(TenantId, CollectionId, DocId)> {
        let s = std::str::from_utf8(key).ok()?;
        let rest = s.strip_prefix("t:")?;
        let mut parts = rest.split(':');
        let tenant_raw: u64 = parts.next()?.parse().ok()?;
        let col_raw: u64 = parts.next()?.parse().ok()?;
        let doc_type = parts.next()?;
        if doc_type != "chunk" {
            return None;
        }
        let doc_raw: u64 = parts.next()?.parse().ok()?;
        if parts.next().is_some() {
            return None; // keine nachfolgenden Segmente erlaubt
        }
        let tenant = TenantId::try_new(tenant_raw).ok()?;
        Some((tenant, CollectionId(col_raw), DocId(doc_raw)))
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


mod storage_engine;

#[cfg(test)]
mod tests;
