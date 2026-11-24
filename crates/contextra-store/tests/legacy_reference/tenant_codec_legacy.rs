#![allow(dead_code)]
// Origin: crates/contextra-store/src/tenant_codec.rs
// Commit: f92caabaa65d74648d64b96c5fffcfd42f922814
// Lines: 30-130

use contextra_types::{CollectionId, DocId, TenantId};

/// Legacy-Referenz der TenantKeyCodec-Implementierung für Differenztests (G-D).
pub struct TenantKeyCodecLegacy {
    tenant_id: TenantId,
    scan_prefix_cache: Vec<u8>,
}

impl TenantKeyCodecLegacy {
    pub fn new(tenant_id: TenantId) -> Self {
        let scan_prefix_cache = format!("t:{}:", tenant_id.inner()).into_bytes();
        Self {
            tenant_id,
            scan_prefix_cache,
        }
    }

    pub fn encode_chunk_key(&self, collection: &CollectionId, doc_id: DocId) -> Vec<u8> {
        format!(
            "t:{}:{}:chunk:{}",
            self.tenant_id.inner(),
            collection.0,
            doc_id.0
        )
        .into_bytes()
    }

    pub fn encode_graph_key(&self, collection: &CollectionId, entity_id: u64) -> Vec<u8> {
        format!(
            "t:{}:{}:graph:{}",
            self.tenant_id.inner(),
            collection.0,
            entity_id
        )
        .into_bytes()
    }

    pub fn scan_prefix(&self) -> &[u8] {
        &self.scan_prefix_cache
    }

    pub fn collection_prefix(&self, collection: &CollectionId) -> Vec<u8> {
        format!("t:{}:{}:", self.tenant_id.inner(), collection.0).into_bytes()
    }

    pub fn decode_tenant_id(key: &[u8]) -> Option<TenantId> {
        let s = std::str::from_utf8(key).ok()?;
        let rest = s.strip_prefix("t:")?;
        let end = rest.find(':')?;
        let id_slice = rest.get(..end)?;
        let id: u64 = id_slice.parse().ok()?;
        TenantId::try_new(id).ok()
    }

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
            return None;
        }
        let tenant = TenantId::try_new(tenant_raw).ok()?;
        Some((tenant, CollectionId(col_raw), DocId(doc_raw)))
    }
}
