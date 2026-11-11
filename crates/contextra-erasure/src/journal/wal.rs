//! # WAL-Entsorgung und Verankerung (INV-ERA-6 / WP-7).
//!
//! Koordiniert die Entsorgung oder Unlesbarmachung von WAL-Einträgen zu gelöschten Scopes
//! sowie deren dauerhafte Verankerung via Marker.

use crate::error::ErasureError;
use crate::types_local::{ErasureId, KeyScope, RequestContext, WalSink};
use parking_lot::RwLock;
use std::collections::BTreeSet;

/// In-Memory Referenzimplementierung von [`WalSink`].
#[derive(Default)]
pub struct MemoryWalSink {
    purged_scopes: RwLock<BTreeSet<Vec<u8>>>,
    erasure_markers: RwLock<Vec<(ErasureId, Vec<u8>)>>,
}

impl MemoryWalSink {
    /// Erzeugt eine neue `MemoryWalSink`.
    pub fn new() -> Self {
        Self {
            purged_scopes: RwLock::new(BTreeSet::new()),
            erasure_markers: RwLock::new(Vec::new()),
        }
    }

    /// Prüft, ob ein Scope bereits als im WAL bereinigt markiert ist.
    pub fn is_scope_purged(&self, scope: &KeyScope) -> bool {
        let key = bincode::serialize(scope).unwrap_or_else(|_| format!("{:?}", scope).into_bytes());
        self.purged_scopes.read().contains(&key)
    }

    /// Gibt alle registrierten Marker zurück.
    pub fn markers_count(&self) -> usize {
        self.erasure_markers.read().len()
    }
}

impl WalSink for MemoryWalSink {
    fn purge_scope_wal(
        &self,
        _ctx: &RequestContext,
        scope: &KeyScope,
    ) -> Result<u64, ErasureError> {
        let key = bincode::serialize(scope).unwrap_or_else(|_| format!("{:?}", scope).into_bytes());
        self.purged_scopes.write().insert(key);
        // Gibt die simulierte Anzahl entsorgter WAL-Einträge zurück
        Ok(1)
    }

    fn append_erasure_marker(
        &self,
        _ctx: &RequestContext,
        erasure_id: &ErasureId,
        scope: &KeyScope,
    ) -> Result<(), ErasureError> {
        let key = bincode::serialize(scope).unwrap_or_else(|_| format!("{:?}", scope).into_bytes());
        self.erasure_markers.write().push((*erasure_id, key));
        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::types_local::{TenantId, TxId};

    #[test]
    fn test_memory_wal_sink() {
        let sink = MemoryWalSink::new();
        let tenant = TenantId::new(10);
        let ctx = RequestContext::new(tenant, TxId::new(1));
        let scope = KeyScope::Tenant(tenant);
        let id = ErasureId::new([0xEE; 16]);

        assert!(!sink.is_scope_purged(&scope));
        let purged = sink.purge_scope_wal(&ctx, &scope).unwrap();
        assert_eq!(purged, 1);
        assert!(sink.is_scope_purged(&scope));

        sink.append_erasure_marker(&ctx, &id, &scope).unwrap();
        assert_eq!(sink.markers_count(), 1);
    }
}
