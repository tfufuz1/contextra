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
use crate::stats::StorageStats;
use contextra_ids::{CollectionId, DocId, TenantId, TxId};
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
// MIGRATION(S-70): StorageEngineHandle moved to compat::engine::tenant_storage
#[cfg(feature = "workspace")]
pub use crate::compat::engine::tenant_storage::*;

pub struct TenantScopedStorage<S> {
    #[allow(dead_code)]
    pub(crate) inner: S,
    pub(crate) codec: TenantKeyCodec,
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

    #[allow(dead_code)]
    #[inline]
    pub(crate) fn make_key(&self, key: &[u8]) -> Vec<u8> {
        let prefix = self.codec.scan_prefix();
        let mut physical = Vec::with_capacity(prefix.len() + key.len());
        physical.extend_from_slice(prefix);
        physical.extend_from_slice(key);
        physical
    }

    #[allow(dead_code)]
    #[inline]
    pub(crate) fn strip_key(&self, key: Vec<u8>) -> Option<Vec<u8>> {
        let prefix = self.codec.scan_prefix();
        if key.starts_with(prefix) {
            Some(key[prefix.len()..].to_vec())
        } else {
            None
        }
    }

    #[allow(dead_code)]
    pub(crate) fn filter_strip_batch(&self, batch: Vec<(Vec<u8>, Vec<u8>)>) -> Vec<(Vec<u8>, Vec<u8>)> {
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

#[allow(dead_code)]
pub(crate) fn upper_bound_for_prefix(prefix: &[u8]) -> Option<Vec<u8>> {
    let mut ub = prefix.to_vec();
    while let Some(last) = ub.pop() {
        if last < 0xFF {
            ub.push(last + 1);
            return Some(ub);
        }
    }
    None
}

#[allow(dead_code)]
pub(crate) fn bound_as_ref(bound: &Bound<Vec<u8>>) -> Bound<&[u8]> {
    match bound {
        Bound::Included(v) => Bound::Included(v.as_slice()),
        Bound::Excluded(v) => Bound::Excluded(v.as_slice()),
        Bound::Unbounded => Bound::Unbounded,
    }
}


mod storage_engine;

#[cfg(test)]
mod tests;
